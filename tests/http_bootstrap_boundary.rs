//! Lazy bootstrap refusal, not real issuer acceptance or authenticated parity.
use calendarweave::{
    CalendarError, InMemoryCalendarService, admission::ExternalIdentity,
    http_bootstrap::unconfigured_get_event,
};
use std::cell::Cell;

#[test]
fn unavailable_runtime_must_precede_factory_acquisition() {
    let acquisitions = Cell::new(0);
    let factory = || {
        acquisitions.set(acquisitions.get() + 1);
        Ok(InMemoryCalendarService::new())
    };
    // This fixture is already-trusted in-process evidence, never a wire verifier.
    let identity =
        ExternalIdentity::parse("https://verified.example.test", "verified-subject").unwrap();
    assert_eq!(
        unconfigured_get_event(Some(&identity), "cal_exact", "evt_exact", factory),
        Err(CalendarError::AuthorizationUnavailable)
    );
    assert_eq!(
        acquisitions.get(),
        0,
        "unavailable runtime must not acquire storage"
    );
}

#[test]
fn absent_wire_identity_does_not_invoke_failing_storage_factory() {
    assert_eq!(
        unconfigured_get_event::<InMemoryCalendarService, _>(
            None,
            "cal_exact",
            "evt_exact",
            || panic!("must not acquire storage")
        ),
        Err(CalendarError::AuthorizationUnavailable)
    );
}
