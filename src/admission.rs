//! Fail-closed authorization admission for calendar-resource operations.
//!
//! This module is an Anti-Corruption Layer between an external identity and
//! authorization authority and the Calendar Resource Core. It consumes scoped
//! identity evidence; it does not validate tokens, implement an identity
//! provider, or move authorization policy into calendar domain entities.

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
    /// Create a collection in the identity's tenant scope.
    CreateCollection,
    /// Create a calendar event in an authorized tenant-scoped collection.
    CreateEvent,
    /// Read calendar events from an authorized tenant-scoped collection.
    ReadEvents,
    /// Conditionally revise an existing calendar event.
    UpdateEvent,
}

/// External principal identity bound to one authorized tenant scope.
///
/// Issuer and subject are retained together because an OpenID Connect subject
/// is only unique within its issuer. CalendarWeave deliberately treats both as
/// opaque external identifiers rather than inventing local identity semantics.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopedIdentity {
    issuer: String,
    subject: String,
    tenant_id: TenantId,
}

impl ScopedIdentity {
    /// Build an externally verified principal bound to one tenant scope.
    ///
    /// The byte limits are defensive admission bounds, not an attempt to
    /// redefine OpenID Connect or JWT identifier syntax. Spaces and other
    /// ordinary Unicode are therefore preserved while control characters are
    /// rejected at this boundary.
    ///
    /// # Errors
    ///
    /// Returns [`CalendarError::InvalidInput`] for an empty, over-limit, or
    /// control-character-bearing issuer or subject.
    pub fn parse(
        issuer: &str,
        subject: &str,
        tenant_id: TenantId,
    ) -> Result<Self, CalendarError> {
        if !bounded_identifier(issuer, MAX_ISSUER_BYTES)
            || !bounded_identifier(subject, MAX_SUBJECT_BYTES)
        {
            return Err(CalendarError::InvalidInput);
        }
        Ok(Self {
            issuer: issuer.to_owned(),
            subject: subject.to_owned(),
            tenant_id,
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

    /// Return the tenant scope admitted for this external principal.
    #[must_use]
    pub fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }
}

/// External authorization port consumed before calendar-domain processing.
///
/// A concrete adapter may consult Keyverse-scoped claims or another approved
/// policy service, but the domain core remains independent of that provider.
pub trait CalendarAuthorizationPort {
    /// Decide whether an identity may perform one Calendar Resource Core action.
    ///
    /// # Errors
    ///
    /// Returns [`AuthorizationError::Denied`] for a completed negative policy
    /// decision and [`AuthorizationError::Unavailable`] when no safe decision
    /// can be established.
    fn authorize(
        &self,
        identity: &ScopedIdentity,
        action: CalendarAction,
    ) -> Result<(), AuthorizationError>;
}

/// Admission service that authorizes before delegating to a calendar port.
///
/// The wrapper does not expose its inner calendar adapter, preventing callers
/// from accidentally bypassing the admission decision through this surface.
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

    /// Create a collection after an affirmative external authorization decision.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed authorization error or the delegated calendar-port
    /// validation/storage error.
    pub fn create_collection(
        &mut self,
        identity: &ScopedIdentity,
        display_name: &str,
    ) -> Result<CalendarCollection, CalendarError> {
        self.authorize(identity, CalendarAction::CreateCollection)?;
        self.calendar
            .create_collection(identity.tenant_id(), display_name)
    }

    /// Create an event only after authorization succeeds.
    ///
    /// Authorization intentionally precedes calendar parsing so denied callers
    /// cannot use parser behavior or resource existence as an information side
    /// channel through this application boundary.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed authorization error or the delegated calendar-port
    /// validation/storage error.
    pub fn create_event(
        &mut self,
        identity: &ScopedIdentity,
        collection_ref: &str,
        icalendar: &str,
    ) -> Result<CalendarEvent, CalendarError> {
        self.authorize(identity, CalendarAction::CreateEvent)?;
        self.calendar
            .create_event(identity.tenant_id(), collection_ref, icalendar)
    }

    /// Conditionally revise an event after authorization succeeds.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed authorization error or the delegated stale-write,
    /// validation, not-found, or storage error.
    pub fn update_event(
        &mut self,
        identity: &ScopedIdentity,
        collection_ref: &str,
        event_ref: &str,
        if_match: &str,
        icalendar: &str,
    ) -> Result<CalendarEvent, CalendarError> {
        self.authorize(identity, CalendarAction::UpdateEvent)?;
        self.calendar.update_event(
            identity.tenant_id(),
            collection_ref,
            event_ref,
            if_match,
            icalendar,
        )
    }

    /// List events after a positive read authorization decision.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed authorization error or the delegated not-found or
    /// storage error.
    pub fn list_events(
        &self,
        identity: &ScopedIdentity,
        collection_ref: &str,
    ) -> Result<Vec<CalendarEvent>, CalendarError> {
        self.authorize(identity, CalendarAction::ReadEvents)?;
        self.calendar
            .list_events(identity.tenant_id(), collection_ref)
    }

    /// Read one event after a positive read authorization decision.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed authorization error or the delegated not-found or
    /// storage error.
    pub fn get_event(
        &self,
        identity: &ScopedIdentity,
        collection_ref: &str,
        event_ref: &str,
    ) -> Result<CalendarEvent, CalendarError> {
        self.authorize(identity, CalendarAction::ReadEvents)?;
        self.calendar
            .get_event(identity.tenant_id(), collection_ref, event_ref)
    }

    fn authorize(
        &self,
        identity: &ScopedIdentity,
        action: CalendarAction,
    ) -> Result<(), CalendarError> {
        self.authorization
            .authorize(identity, action)
            .map_err(|error| match error {
                AuthorizationError::Denied => CalendarError::Unauthorized,
                AuthorizationError::Unavailable => CalendarError::AuthorizationUnavailable,
            })
    }
}

fn bounded_identifier(value: &str, max_bytes: usize) -> bool {
    !value.is_empty() && value.len() <= max_bytes && !value.chars().any(char::is_control)
}
