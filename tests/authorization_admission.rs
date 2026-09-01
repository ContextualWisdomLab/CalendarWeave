use calendarweave::admission::{
    AuthorizationError, AuthorizedCalendarService, CalendarAction, CalendarAuthorizationPort,
    ScopedIdentity,
};
use calendarweave::{CalendarError, InMemoryCalendarService, TenantId};

const EVENT: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//ContextualWisdomLab//CalendarWeave Test//EN\r\nBEGIN:VEVENT\r\nUID:event-1@example.test\r\nDTSTAMP:20260901T000000Z\r\nDTSTART:20260902T090000Z\r\nDTEND:20260902T100000Z\r\nSUMMARY:Customer planning session\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
const UPDATED_EVENT: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//ContextualWisdomLab//CalendarWeave Test//EN\r\nBEGIN:VEVENT\r\nUID:event-1@example.test\r\nDTSTAMP:20260901T000000Z\r\nDTSTART:20260902T090000Z\r\nDTEND:20260902T100000Z\r\nSUMMARY:Updated customer planning session\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

#[derive(Clone, Copy, Debug)]
struct StubAuthorization {
    denied_action: Option<CalendarAction>,
    unavailable: bool,
}

impl StubAuthorization {
    fn allow_all() -> Self {
        Self {
            denied_action: None,
            unavailable: false,
        }
    }

    fn deny(action: CalendarAction) -> Self {
        Self {
            denied_action: Some(action),
            unavailable: false,
        }
    }

    fn unavailable() -> Self {
        Self {
            denied_action: None,
            unavailable: true,
        }
    }
}

impl CalendarAuthorizationPort for StubAuthorization {
    fn authorize(
        &self,
        identity: &ScopedIdentity,
        action: CalendarAction,
    ) -> Result<(), AuthorizationError> {
        assert_eq!(identity.issuer(), "https://identity.example.test");
        assert_eq!(identity.subject(), "customer-user-01");
        assert!(identity.tenant_id().as_ref().starts_with("tenant-"));
        if self.unavailable {
            return Err(AuthorizationError::Unavailable);
        }
        if self.denied_action == Some(action) {
            return Err(AuthorizationError::Denied);
        }
        Ok(())
    }
}

fn identity(tenant: &str) -> ScopedIdentity {
    ScopedIdentity::parse(
        "https://identity.example.test",
        "customer-user-01",
        TenantId::parse(tenant).expect("tenant fixture is valid"),
    )
    .expect("identity fixture is valid")
}

#[test]
fn admission_denies_before_parsing_untrusted_calendar_payload() {
    let mut service = AuthorizedCalendarService::new(
        StubAuthorization::deny(CalendarAction::CreateEvent),
        InMemoryCalendarService::new(),
    );
    let identity = identity("tenant-a");
    let collection = service
        .create_collection(&identity, "Customer calendar")
        .expect("collection creation is authorized");

    assert_eq!(
        service.create_event(&identity, &collection.collection_ref, "not an iCalendar payload"),
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
        service.create_collection(&identity("tenant-a"), "Customer calendar"),
        Err(CalendarError::AuthorizationUnavailable)
    );
}

#[test]
fn scoped_identity_validation_is_bounded_and_opaque() {
    let tenant = TenantId::parse("tenant-a").expect("tenant fixture is valid");
    assert!(
        ScopedIdentity::parse(
            "https://identity.example.test",
            "subject with spaces",
            tenant.clone()
        )
        .is_ok()
    );
    assert_eq!(
        ScopedIdentity::parse("", "subject-01", tenant.clone()),
        Err(CalendarError::InvalidInput)
    );
    assert_eq!(
        ScopedIdentity::parse(
            "https://identity.example.test",
            "",
            tenant.clone()
        ),
        Err(CalendarError::InvalidInput)
    );
    assert_eq!(
        ScopedIdentity::parse(
            "https://identity.example.test",
            &"s".repeat(513),
            tenant.clone()
        ),
        Err(CalendarError::InvalidInput)
    );
    assert_eq!(
        ScopedIdentity::parse(
            "https://identity.example.test",
            "subject\u{0000}bad",
            tenant
        ),
        Err(CalendarError::InvalidInput)
    );
}

#[test]
fn issuer_and_subject_jointly_identify_the_external_principal() {
    let tenant = TenantId::parse("tenant-a").expect("tenant fixture is valid");
    let first = ScopedIdentity::parse(
        "https://identity.example.test",
        "customer-user-01",
        tenant.clone(),
    )
    .expect("identity fixture is valid");
    let second = ScopedIdentity::parse(
        "https://other-identity.example.test",
        "customer-user-01",
        tenant,
    )
    .expect("identity fixture is valid");

    assert_ne!(first, second);
}

#[test]
fn admitted_operations_stay_bound_to_the_identity_tenant() {
    let mut service = AuthorizedCalendarService::new(
        StubAuthorization::allow_all(),
        InMemoryCalendarService::new(),
    );
    let tenant_a = identity("tenant-a");
    let tenant_b = identity("tenant-b");
    let collection = service
        .create_collection(&tenant_a, "Customer calendar")
        .expect("collection creation succeeds");
    let created = service
        .create_event(&tenant_a, &collection.collection_ref, EVENT)
        .expect("event creation succeeds");

    assert_eq!(
        service.get_event(&tenant_b, &collection.collection_ref, &created.event_ref),
        Err(CalendarError::NotFound)
    );
    assert_eq!(
        service.list_events(&tenant_b, &collection.collection_ref),
        Err(CalendarError::NotFound)
    );
}

#[test]
fn admitted_crud_preserves_calendar_port_revision_contract() {
    let mut service = AuthorizedCalendarService::new(
        StubAuthorization::allow_all(),
        InMemoryCalendarService::new(),
    );
    let identity = identity("tenant-a");
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
