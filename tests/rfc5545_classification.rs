//! RFC 5545 CLASS privacy contracts for CalendarWeave consumer parity.

use calendarweave::{
    CalendarError, CalendarEvent, CalendarPort, EventClass, InMemoryCalendarService, TenantId,
};

fn payload(class_line: Option<&str>) -> String {
    let class_line = class_line.map_or(String::new(), |line| format!("{line}\r\n"));
    format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//ContextualWisdomLab//CalendarWeave v1//EN\r\nBEGIN:VEVENT\r\nUID:synthetic-class@example.test\r\nDTSTAMP:20260902T000000Z\r\nDTSTART:20260903T090000Z\r\nDTEND:20260903T100000Z\r\nSUMMARY:Synthetic privacy review\r\n{class_line}END:VEVENT\r\nEND:VCALENDAR\r\n"
    )
}

fn create_event(input: &str) -> Result<CalendarEvent, CalendarError> {
    let tenant = TenantId::parse("synthetic-class-tenant").expect("test tenant is valid");
    let mut service = InMemoryCalendarService::new();
    let collection = service
        .create_collection(&tenant, "Synthetic privacy calendar")
        .expect("test collection is valid");
    service.create_event(&tenant, &collection.collection_ref, input)
}

#[test]
fn class_defaults_to_public_and_exposes_standard_privacy_values_case_insensitively() {
    for (class_line, expected) in [
        (None, EventClass::Public),
        (Some("CLASS:PUBLIC"), EventClass::Public),
        (Some("CLASS:private"), EventClass::Private),
        (Some("CLASS:Confidential"), EventClass::Confidential),
    ] {
        let event = create_event(&payload(class_line)).expect("standard CLASS must be accepted");
        assert_eq!(event.classification(), Ok(expected));
    }
}

#[test]
fn unknown_registered_or_experimental_class_values_fail_private() {
    for class_line in [
        "CLASS:RESTRICTED",
        "CLASS:X-SYNTHETIC-RESTRICTED",
        "CLASS;X-SYNTHETIC-HINT=LOCAL:X-SYNTHETIC-RESTRICTED",
    ] {
        let event = create_event(&payload(Some(class_line)))
            .expect("RFC token CLASS values must remain interoperable");
        assert_eq!(event.classification(), Ok(EventClass::Private));
    }
}

#[test]
fn class_is_singleton_and_rejects_non_token_values() {
    for class_lines in [
        "CLASS:PRIVATE\r\nCLASS:PUBLIC",
        "CLASS:",
        "CLASS:NOT PRIVATE",
        "CLASS:PRIVATE_VALUE",
    ] {
        assert_eq!(
            create_event(&payload(Some(class_lines))),
            Err(CalendarError::MalformedCalendar),
            "invalid CLASS must fail closed: {class_lines}"
        );
    }
}

#[test]
fn classification_accessor_fails_closed_for_a_forged_projection() {
    let mut event = create_event(&payload(Some("CLASS:PRIVATE"))).expect("valid test event");
    event.icalendar = payload(Some("CLASS:NOT PRIVATE"));
    assert_eq!(event.classification(), Err(CalendarError::MalformedCalendar));
}

#[test]
fn classification_accessor_revalidates_the_full_bounded_event_profile() {
    let mut event = create_event(&payload(Some("CLASS:PRIVATE"))).expect("valid test event");
    event.icalendar = event.icalendar.replace(
        "SUMMARY:Synthetic privacy review\r\n",
        "SUMMARY:Synthetic privacy review\r\nATTENDEE:mailto:synthetic@example.test\r\n",
    );
    assert_eq!(
        event.classification(),
        Err(CalendarError::UnsupportedCapability),
        "a forged projection must not recover a trusted CLASS value from an otherwise unsupported event"
    );
}