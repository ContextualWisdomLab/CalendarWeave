//! RFC 5545 DURATION profile contracts for bounded CalendarWeave VEVENTs.

use calendarweave::{
    CalendarError, CalendarEvent, CalendarPort, InMemoryCalendarService, TenantId,
};

fn payload(start: &str, interval_lines: &str) -> String {
    format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//ContextualWisdomLab//CalendarWeave v1//EN\r\nBEGIN:VEVENT\r\nUID:synthetic-duration@example.test\r\nDTSTAMP:20260901T000000Z\r\n{start}\r\n{interval_lines}\r\nSUMMARY:Synthetic duration review\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n"
    )
}

fn create_event(input: &str) -> Result<CalendarEvent, CalendarError> {
    let tenant = TenantId::parse("synthetic-duration-tenant").expect("test tenant is valid");
    let mut service = InMemoryCalendarService::new();
    let collection = service
        .create_collection(&tenant, "Synthetic duration calendar")
        .expect("test collection is valid");
    service.create_event(&tenant, &collection.collection_ref, input)
}

#[test]
fn positive_duration_is_an_interval_alternative_to_dtend() {
    for input in [
        payload("DTSTART:20260902T090000Z", "DURATION:PT30M"),
        payload("DTSTART:20260902T090000Z", "DURATION:PT1H30S"),
        payload("DTSTART:20260902T090000Z", "DURATION:+PT1H0M0S"),
        payload("DTSTART:20260902T090000Z", "DURATION:P15DT5H0M20S"),
        payload("DTSTART:20260902T090000Z", "DURATION:P1DT1H30S"),
        payload(
            "DTSTART;TZID=Asia/Seoul:20260902T090000",
            "DURATION:P1DT2H",
        ),
        payload("DTSTART;VALUE=DATE:20260902", "DURATION:P1D"),
        payload("DTSTART;VALUE=DATE:20260902", "DURATION:P2W"),
    ] {
        assert!(
            create_event(&input).is_ok(),
            "duration should be accepted: {input}"
        );
    }
}

#[test]
fn duration_and_dtend_are_mutually_exclusive_and_the_bounded_profile_requires_one() {
    let both = payload(
        "DTSTART:20260902T090000Z",
        "DTEND:20260902T100000Z\r\nDURATION:PT1H",
    );
    let neither = payload("DTSTART:20260902T090000Z", "SEQUENCE:1");
    let duplicate = payload(
        "DTSTART:20260902T090000Z",
        "DURATION:PT30M\r\nDURATION:PT45M",
    );

    for input in [both, neither, duplicate] {
        assert_eq!(create_event(&input), Err(CalendarError::MalformedCalendar));
    }
}

#[test]
fn duration_must_be_positive_and_match_the_rfc5545_duration_grammar() {
    for value in [
        "-PT30M",
        "PT0S",
        "P0D",
        "P0W",
        "P",
        "PT",
        "P1W1D",
        "P1Y",
        "P1M",
        "PT1.5H",
        "PT1H30S5M",
        "PT1M2H",
    ] {
        let input = payload("DTSTART:20260902T090000Z", &format!("DURATION:{value}"));
        assert_eq!(
            create_event(&input),
            Err(CalendarError::MalformedCalendar),
            "invalid duration should fail closed: {value}"
        );
    }
}

#[test]
fn date_start_accepts_only_day_or_week_duration_shapes() {
    for value in ["PT24H", "P1DT1H", "PT30M"] {
        let input = payload(
            "DTSTART;VALUE=DATE:20260902",
            &format!("DURATION:{value}"),
        );
        assert_eq!(
            create_event(&input),
            Err(CalendarError::MalformedCalendar),
            "DATE DTSTART requires dur-day or dur-week: {value}"
        );
    }
}

#[test]
fn duration_reuses_named_timezone_start_fail_closed_semantics() {
    for malformed_start in [
        "DTSTART;TZID=America/New_York:20261101T013000",
        "DTSTART;TZID=America/New_York:20260308T023000",
    ] {
        assert_eq!(
            create_event(&payload(malformed_start, "DURATION:PT1H")),
            Err(CalendarError::MalformedCalendar)
        );
    }
    assert_eq!(
        create_event(&payload(
            "DTSTART;TZID=Synthetic/Unknown:20260902T090000",
            "DURATION:PT1H"
        )),
        Err(CalendarError::UnsupportedCapability)
    );
}

#[test]
fn duration_parameters_remain_outside_the_bounded_v1_profile() {
    let input = payload(
        "DTSTART:20260902T090000Z",
        "DURATION;X-SYNTHETIC=1:PT1H",
    );
    assert_eq!(
        create_event(&input),
        Err(CalendarError::UnsupportedCapability)
    );
}
