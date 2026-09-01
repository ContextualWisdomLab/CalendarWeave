use calendarweave::admission::{
    AuthorizationError, AuthorizedCalendarService, CalendarAction, CalendarAuthorizationPort,
    CalendarAuthorizationRequest, ExternalIdentity,
};
use calendarweave::{CalendarError, InMemoryCalendarService, TenantId};

const EVENT: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//ContextualWisdomLab//CalendarWeave Test//EN\r\nBEGIN:VEVENT\r\nUID:event-1@example.test\r\nDTSTAMP:20260901T000000Z\r\nDTSTART:20260902T090000Z\r\nDTEND:20260902T100000Z\r\nSUMMARY:Customer planning session\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
const UPDATED_EVENT: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//ContextualWisdomLab//CalendarWeave Test//EN\r\nBEGIN:VEVENT\r\nUID:event-1@example.test\r\nDTSTAMP:20260901T000000Z\r\nDTSTART:20260902T090000Z\r\nDTEND:20260902T100000Z\r\nSUMMARY:Updated customer planning session\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

#[derive(Clone, Debug)]
struct StubAuthorization {
    denied_action: Option<CalendarAction>,
    denied_collection_ref: Option<String>,
    unavailable: bool,
}

impl StubAuthorization {
    fn allow_all() -> Self {
        Self {
            denied_action: None,
            denied_collection_ref: None,
            unavailable: false,
        }
    }

    fn deny(action: CalendarAction) -> Self {
        Self {
            denied_action: Some(action),
            denied_collection_ref: None,
            unavailable: false,
        }
    }

    fn deny_collection(action: CalendarAction, collection_ref: &str) -> Self {
        Self {
            denied_action: Some(action),
            denied_collection_ref: Some(collection_ref.to_owned()),
            unavailable: false,
        }
    }

    fn unavailable() -> Self {
        Self {
            denied_action: None,
            denied_collection_ref: None,
            unavailable: true,
        }
    }
}

impl CalendarAuthorizationPort for StubAuthorization {
    fn authorize(
        &self,
        identity: &ExternalIdentity,
        request: &CalendarAuthorizationRequest<'_>,
    ) -> Result<TenantId, AuthorizationError> {
        assert_eq!(identity.issuer(), "https://identity.example.test");
        if self.unavailable {
            return Err(AuthorizationError::Unavailable);
        }
        if self.denied_action == Some(request.action())
            && self
                .denied_collection_ref
                .as_deref()
                .is_none_or(|expected| request.collection_ref() == Some(expected))
        {
            return Err(AuthorizationError::Denied);
        }
        let tenant = match identity.subject() {
            "customer-user-01" => "tenant-a",
            "customer-user-02" => "tenant-b",
            _ => return Err(AuthorizationError::Denied),
        };
        TenantId::parse(tenant).map_err(|_| AuthorizationError::Unavailable)
    }
}

fn identity(subject: &str) -> ExternalIdentity {
    ExternalIdentity::parse("https://identity.example.test", subject)
        .expect("identity fixture is valid")
}

#[test]
fn admission_denies_before_parsing_untrusted_calendar_payload() {
    let mut service = AuthorizedCalendarService::new(
        StubAuthorization::deny(CalendarAction::CreateEvent),
        InMemoryCalendarService::new(),
    );
    let identity = identity("customer-user-01");

    assert_eq!(
        service.create_event(&identity, "missing-collection", "not an iCalendar payload"),
        Err(CalendarError::Unauthorized)
    );
}

#[test]
fn admission_distinguishes_authorization_dependency_failure() {
    let mut service = AuthorizedCalendarService::new(
        StubAuthorization::unavailable(),
        InMemoryCalendarService::new(),
    );

    assert_eq!(
        service.create_collection(&identity("customer-user-01"), "Customer calendar"),
        Err(CalendarError::AuthorizationUnavailable)
    );
}

#[test]
fn authorization_denial_covers_every_published_calendar_action() {
    let identity = identity("customer-user-01");

    let mut create_collection = AuthorizedCalendarService::new(
        StubAuthorization::deny(CalendarAction::CreateCollection),
        InMemoryCalendarService::new(),
    );
    assert_eq!(
        create_collection.create_collection(&identity, "Customer calendar"),
        Err(CalendarError::Unauthorized)
    );

    let mut create_event = AuthorizedCalendarService::new(
        StubAuthorization::deny(CalendarAction::CreateEvent),
        InMemoryCalendarService::new(),
    );
    assert_eq!(
        create_event.create_event(&identity, "missing-collection", "not an iCalendar payload"),
        Err(CalendarError::Unauthorized)
    );

    let read_events = AuthorizedCalendarService::new(
        StubAuthorization::deny(CalendarAction::ReadEvents),
        InMemoryCalendarService::new(),
    );
    assert_eq!(
        read_events.list_events(&identity, "missing-collection"),
        Err(CalendarError::Unauthorized)
    );
    assert_eq!(
        read_events.get_event(&identity, "missing-collection", "missing-event"),
        Err(CalendarError::Unauthorized)
    );

    let mut update_event = AuthorizedCalendarService::new(
        StubAuthorization::deny(CalendarAction::UpdateEvent),
        InMemoryCalendarService::new(),
    );
    assert_eq!(
        update_event.update_event(
            &identity,
            "missing-collection",
            "missing-event",
            "\"missing:1\"",
            "not an iCalendar payload",
        ),
        Err(CalendarError::Unauthorized)
    );
}

#[test]
fn authorization_unavailability_covers_every_published_calendar_action() {
    let identity = identity("customer-user-01");
    let mut service = AuthorizedCalendarService::new(
        StubAuthorization::unavailable(),
        InMemoryCalendarService::new(),
    );

    assert_eq!(
        service.create_collection(&identity, "Customer calendar"),
        Err(CalendarError::AuthorizationUnavailable)
    );
    assert_eq!(
        service.create_event(&identity, "missing-collection", "not an iCalendar payload"),
        Err(CalendarError::AuthorizationUnavailable)
    );
    assert_eq!(
        service.list_events(&identity, "missing-collection"),
        Err(CalendarError::AuthorizationUnavailable)
    );
    assert_eq!(
        service.get_event(&identity, "missing-collection", "missing-event"),
        Err(CalendarError::AuthorizationUnavailable)
    );
    assert_eq!(
        service.update_event(
            &identity,
            "missing-collection",
            "missing-event",
            "\"missing:1\"",
            "not an iCalendar payload",
        ),
        Err(CalendarError::AuthorizationUnavailable)
    );
}

#[test]
fn resource_context_reaches_authorization_before_domain_lookup() {
    let identity = identity("customer-user-01");
    let service = AuthorizedCalendarService::new(
        StubAuthorization::deny_collection(CalendarAction::ReadEvents, "restricted-collection"),
        InMemoryCalendarService::new(),
    );

    assert_eq!(
        service.list_events(&identity, "restricted-collection"),
        Err(CalendarError::Unauthorized)
    );
    assert_eq!(
        service.get_event(&identity, "restricted-collection", "event-1"),
        Err(CalendarError::Unauthorized)
    );
    assert_eq!(
        service.list_events(&identity, "different-missing-collection"),
        Err(CalendarError::NotFound)
    );
}

#[test]
fn authorization_request_exposes_exact_resource_context() {
    let mut service = AuthorizedCalendarService::new(
        InspectingAuthorization,
        InMemoryCalendarService::new(),
    );
    let identity = identity("customer-user-01");

    assert_eq!(
        service.update_event(
            &identity,
            "calendar-123",
            "event-456",
            "\"event-456:1\"",
            "not an iCalendar payload",
        ),
        Err(CalendarError::NotFound)
    );
}

#[derive(Clone, Copy, Debug)]
struct InspectingAuthorization;

impl CalendarAuthorizationPort for InspectingAuthorization {
    fn authorize(
        &self,
        _identity: &ExternalIdentity,
        request: &CalendarAuthorizationRequest<'_>,
    ) -> Result<TenantId, AuthorizationError> {
        assert_eq!(request.action(), CalendarAction::UpdateEvent);
        assert_eq!(request.collection_ref(), Some("calendar-123"));
        assert_eq!(request.event_ref(), Some("event-456"));
        TenantId::parse("tenant-a").map_err(|_| AuthorizationError::Unavailable)
    }
}

#[test]
fn external_identity_validation_is_bounded_and_opaque() {
    assert!(ExternalIdentity::parse("https://identity.example.test", "subject with spaces").is_ok());
    assert_eq!(
        ExternalIdentity::parse("", "subject-01"),
        Err(CalendarError::InvalidInput)
    );
    assert_eq!(
        ExternalIdentity::parse("https://identity.example.test", ""),
        Err(CalendarError::InvalidInput)
    );
    assert_eq!(
        ExternalIdentity::parse("https://identity.example.test", &"s".repeat(513)),
        Err(CalendarError::InvalidInput)
    );
    assert_eq!(
        ExternalIdentity::parse("https://identity.example.test", "subject\u{0000}bad"),
        Err(CalendarError::InvalidInput)
    );
}

#[test]
fn issuer_and_subject_jointly_identify_the_external_principal() {
    let first = ExternalIdentity::parse(
        "https://identity.example.test",
        "customer-user-01",
    )
    .expect("identity fixture is valid");
    let second = ExternalIdentity::parse(
        "https://other-identity.example.test",
        "customer-user-01",
    )
    .expect("identity fixture is valid");

    assert_ne!(first, second);
}

#[test]
fn authorized_tenant_is_derived_by_the_policy_adapter_not_the_caller() {
    let mut service = AuthorizedCalendarService::new(
        StubAuthorization::allow_all(),
        InMemoryCalendarService::new(),
    );
    let tenant_a_identity = identity("customer-user-01");
    let tenant_b_identity = identity("customer-user-02");
    let collection = service
        .create_collection(&tenant_a_identity, "Customer calendar")
        .expect("collection creation succeeds");
    let created = service
        .create_event(&tenant_a_identity, &collection.collection_ref, EVENT)
        .expect("event creation succeeds");

    assert_eq!(
        service.get_event(
            &tenant_b_identity,
            &collection.collection_ref,
            &created.event_ref,
        ),
        Err(CalendarError::NotFound)
    );
    assert_eq!(
        service.list_events(&tenant_b_identity, &collection.collection_ref),
        Err(CalendarError::NotFound)
    );
}

#[test]
fn admitted_crud_preserves_calendar_port_revision_contract() {
    let mut service = AuthorizedCalendarService::new(
        StubAuthorization::allow_all(),
        InMemoryCalendarService::new(),
    );
    let identity = identity("customer-user-01");
    let collection = service
        .create_collection(&identity, "Customer calendar")
        .expect("collection creation succeeds");
    let created = service
        .create_event(&identity, &collection.collection_ref, EVENT)
        .expect("event creation succeeds");

    let listed = service
        .list_events(&identity, &collection.collection_ref)
        .expect("list succeeds");
    assert_eq!(listed, vec![created.clone()]);
    assert_eq!(
        service
            .get_event(&identity, &collection.collection_ref, &created.event_ref)
            .expect("get succeeds"),
        created
    );

    let updated = service
        .update_event(
            &identity,
            &collection.collection_ref,
            &created.event_ref,
            &created.etag,
            UPDATED_EVENT,
        )
        .expect("conditional update succeeds");
    assert_eq!(updated.revision, 2);
    assert_ne!(updated.etag, created.etag);
}
