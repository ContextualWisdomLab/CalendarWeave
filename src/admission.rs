//! Fail-closed authorization admission for calendar-resource operations.
//!
//! This module is an Anti-Corruption Layer between external identity and
//! authorization authorities and the Calendar Resource Core. The caller
//! supplies only externally verified issuer/subject identity evidence. The
//! authorization adapter derives the authorized tenant and receives the exact
//! requested calendar-resource context; callers cannot self-assert a tenant
//! through the admission service.

use crate::{CalendarCollection, CalendarError, CalendarEvent, CalendarPort, TenantId};

const MAX_ISSUER_BYTES: usize = 2_048;
const MAX_SUBJECT_BYTES: usize = 512;

/// A bounded result from the external authorization dependency.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthorizationError {
    /// The identity is known but is not permitted to perform the requested action.
    Denied,
    /// Authorization could not be established because its dependency was unavailable.
    Unavailable,
}

/// Calendar Resource Core actions evaluated at the admission boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CalendarAction {
    /// Create a collection in the tenant scope returned by authorization.
    CreateCollection,
    /// Create a calendar event in an authorized collection.
    CreateEvent,
    /// Read calendar events from an authorized collection or event resource.
    ReadEvents,
    /// Conditionally revise an authorized calendar event.
    UpdateEvent,
}

/// Externally verified principal identity before tenant authorization.
///
/// Issuer and subject are retained together because an OpenID Connect subject
/// is only unique within its issuer. The value contains no tenant scope: the
/// trusted authorization adapter must derive that scope for each request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalIdentity {
    issuer: String,
    subject: String,
}

impl ExternalIdentity {
    /// Build opaque identity evidence from an already verified issuer/subject pair.
    ///
    /// The byte limits are defensive admission bounds, not an attempt to
    /// redefine OpenID Connect or JWT identifier syntax. Spaces and ordinary
    /// Unicode are preserved while control characters are rejected.
    ///
    /// # Errors
    ///
    /// Returns [`CalendarError::InvalidInput`] for an empty, over-limit, or
    /// control-character-bearing issuer or subject.
    pub fn parse(issuer: &str, subject: &str) -> Result<Self, CalendarError> {
        if !bounded_identifier(issuer, MAX_ISSUER_BYTES)
            || !bounded_identifier(subject, MAX_SUBJECT_BYTES)
        {
            return Err(CalendarError::InvalidInput);
        }
        Ok(Self {
            issuer: issuer.to_owned(),
            subject: subject.to_owned(),
        })
    }

    /// Return the opaque issuer supplied by the external identity authority.
    #[must_use]
    pub fn issuer(&self) -> &str {
        &self.issuer
    }

    /// Return the opaque subject supplied by the external identity authority.
    #[must_use]
    pub fn subject(&self) -> &str {
        &self.subject
    }
}

/// Exact calendar resource context presented to the authorization authority.
///
/// `collection_ref` and `event_ref` are opaque resource references. They let a
/// policy adapter enforce resource-scoped grants without exposing calendar
/// persistence or forcing CalendarWeave to own the external policy model.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CalendarAuthorizationRequest<'a> {
    action: CalendarAction,
    collection_ref: Option<&'a str>,
    event_ref: Option<&'a str>,
}

impl<'a> CalendarAuthorizationRequest<'a> {
    fn create_collection() -> Self {
        Self {
            action: CalendarAction::CreateCollection,
            collection_ref: None,
            event_ref: None,
        }
    }

    fn create_event(collection_ref: &'a str) -> Self {
        Self {
            action: CalendarAction::CreateEvent,
            collection_ref: Some(collection_ref),
            event_ref: None,
        }
    }

    fn list_events(collection_ref: &'a str) -> Self {
        Self {
            action: CalendarAction::ReadEvents,
            collection_ref: Some(collection_ref),
            event_ref: None,
        }
    }

    fn get_event(collection_ref: &'a str, event_ref: &'a str) -> Self {
        Self {
            action: CalendarAction::ReadEvents,
            collection_ref: Some(collection_ref),
            event_ref: Some(event_ref),
        }
    }

    fn update_event(collection_ref: &'a str, event_ref: &'a str) -> Self {
        Self {
            action: CalendarAction::UpdateEvent,
            collection_ref: Some(collection_ref),
            event_ref: Some(event_ref),
        }
    }

    /// Return the typed Calendar Resource action being requested.
    #[must_use]
    pub fn action(&self) -> CalendarAction {
        self.action
    }

    /// Return the exact opaque collection reference when the action targets one.
    #[must_use]
    pub fn collection_ref(&self) -> Option<&str> {
        self.collection_ref
    }

    /// Return the exact opaque event reference when the action targets one.
    #[must_use]
    pub fn event_ref(&self) -> Option<&str> {
        self.event_ref
    }
}

/// External authorization port consumed before calendar-domain processing.
///
/// A concrete adapter may validate Keyverse-issued claims and consult an
/// approved policy service. Its successful result is the tenant scope that was
/// actually authorized for this identity, action, and resource context.
pub trait CalendarAuthorizationPort {
    /// Authorize one exact Calendar Resource request and derive its tenant scope.
    ///
    /// # Errors
    ///
    /// Returns [`AuthorizationError::Denied`] for a completed negative policy
    /// decision and [`AuthorizationError::Unavailable`] when no safe decision
    /// can be established.
    fn authorize(
        &self,
        identity: &ExternalIdentity,
        request: &CalendarAuthorizationRequest<'_>,
    ) -> Result<TenantId, AuthorizationError>;
}

/// Admission service that authorizes before delegating to a calendar port.
///
/// The wrapper does not expose its inner calendar adapter, preventing callers
/// from bypassing admission through this surface. The tenant passed into the
/// core always comes from the authorization result rather than caller input.
pub struct AuthorizedCalendarService<A, P> {
    authorization: A,
    calendar: P,
}

impl<A, P> AuthorizedCalendarService<A, P>
where
    A: CalendarAuthorizationPort,
    P: CalendarPort,
{
    /// Compose an external authorization adapter with one calendar port.
    #[must_use]
    pub fn new(authorization: A, calendar: P) -> Self {
        Self {
            authorization,
            calendar,
        }
    }

    /// Create a collection after authorization derives the permitted tenant.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed authorization error or the delegated calendar-port
    /// validation/storage error.
    pub fn create_collection(
        &mut self,
        identity: &ExternalIdentity,
        display_name: &str,
    ) -> Result<CalendarCollection, CalendarError> {
        let tenant_id = self.authorize(identity, &CalendarAuthorizationRequest::create_collection())?;
        self.calendar.create_collection(&tenant_id, display_name)
    }

    /// Create an event only after resource-aware authorization succeeds.
    ///
    /// Authorization intentionally precedes collection lookup and calendar
    /// parsing so denied callers cannot use resource or parser behavior as an
    /// information side channel through this application boundary.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed authorization error or the delegated calendar-port
    /// validation/storage error.
    pub fn create_event(
        &mut self,
        identity: &ExternalIdentity,
        collection_ref: &str,
        icalendar: &str,
    ) -> Result<CalendarEvent, CalendarError> {
        let tenant_id = self.authorize(
            identity,
            &CalendarAuthorizationRequest::create_event(collection_ref),
        )?;
        self.calendar
            .create_event(&tenant_id, collection_ref, icalendar)
    }

    /// Conditionally revise an event after exact resource authorization succeeds.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed authorization error or the delegated stale-write,
    /// validation, not-found, or storage error.
    pub fn update_event(
        &mut self,
        identity: &ExternalIdentity,
        collection_ref: &str,
        event_ref: &str,
        if_match: &str,
        icalendar: &str,
    ) -> Result<CalendarEvent, CalendarError> {
        let tenant_id = self.authorize(
            identity,
            &CalendarAuthorizationRequest::update_event(collection_ref, event_ref),
        )?;
        self.calendar.update_event(
            &tenant_id,
            collection_ref,
            event_ref,
            if_match,
            icalendar,
        )
    }

    /// List events after collection-scoped read authorization succeeds.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed authorization error or the delegated not-found or
    /// storage error.
    pub fn list_events(
        &self,
        identity: &ExternalIdentity,
        collection_ref: &str,
    ) -> Result<Vec<CalendarEvent>, CalendarError> {
        let tenant_id = self.authorize(
            identity,
            &CalendarAuthorizationRequest::list_events(collection_ref),
        )?;
        self.calendar.list_events(&tenant_id, collection_ref)
    }

    /// Read one event after exact collection/event authorization succeeds.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed authorization error or the delegated not-found or
    /// storage error.
    pub fn get_event(
        &self,
        identity: &ExternalIdentity,
        collection_ref: &str,
        event_ref: &str,
    ) -> Result<CalendarEvent, CalendarError> {
        let tenant_id = self.authorize(
            identity,
            &CalendarAuthorizationRequest::get_event(collection_ref, event_ref),
        )?;
        self.calendar
            .get_event(&tenant_id, collection_ref, event_ref)
    }

    fn authorize(
        &self,
        identity: &ExternalIdentity,
        request: &CalendarAuthorizationRequest<'_>,
    ) -> Result<TenantId, CalendarError> {
        self.authorization
            .authorize(identity, request)
            .map_err(|error| match error {
                AuthorizationError::Denied => CalendarError::Unauthorized,
                AuthorizationError::Unavailable => CalendarError::AuthorizationUnavailable,
            })
    }
}

fn bounded_identifier(value: &str, max_bytes: usize) -> bool {
    !value.is_empty() && value.len() <= max_bytes && !value.chars().any(char::is_control)
}
