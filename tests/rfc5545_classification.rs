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
fn class_defaults_to_public_and_exposes_standard_privacy_values() {
    for (class_line, expected) in [
        (None, EventClass::Public),
        (Some("CLASS:PUBLIC"), EventClass::Public),
        (Some("CLASS:PRIVATE"), EventClass::Private),
        (Some("CLASS:CONFIDENTIAL"), EventClass::Confidential),
    ] {
        let event = create_event(&payload(class_line)).expect("standard CLASS must be accepted");
        assert_eq!(event.classification, expected);
    }
}

#[test]
fn class_is_singleton_parameter_free_and_standard_value_only() {
    for class_lines in [
        "CLASS:PRIVATE\r\nCLASS:PUBLIC",
        "CLASS;X-SYNTHETIC=1:PRIVATE",
        "CLASS:SECRET",
        "CLASS:private",
        "CLASS:",
    ] {
        assert_eq!(
            create_event(&payload(Some(class_lines))),
            Err(CalendarError::MalformedCalendar),
            "invalid CLASS must fail closed: {class_lines}"
        );
    }
}
