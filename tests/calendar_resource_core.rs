//! Contract tests for the first tenant-scoped calendar-resource vertical.

use calendarweave::{CalendarError, CalendarPort, EventStatus, InMemoryCalendarService, TenantId};

const UTC_EVENT: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//ContextualWisdomLab//CalendarWeave v1//EN\r\nBEGIN:VEVENT\r\nUID:synthetic-event-1@example.test\r\nDTSTAMP:20260901T000000Z\r\nDTSTART:20260902T090000Z\r\nDTEND:20260902T100000Z\r\nSUMMARY:Synthetic planning review\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

const ALL_DAY_EVENT: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//ContextualWisdomLab//CalendarWeave v1//EN\r\nBEGIN:VEVENT\r\nUID:synthetic-all-day@example.test\r\nDTSTAMP:20260901T000000Z\r\nDTSTART;VALUE=DATE:20260902\r\nDTEND;VALUE=DATE:20260903\r\nSUMMARY:Synthetic all-day review\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

fn tenant(value: &str) -> TenantId {
    TenantId::parse(value).expect("synthetic tenant is valid")
}

#[test]
fn tenant_scoped_create_list_and_get_preserve_the_event() {
    let mut service = InMemoryCalendarService::new();
    let tenant = tenant("synthetic-tenant-a");
    let collection = service
        .create_collection(&tenant, "Synthetic calendar")
        .expect("collection creation succeeds");
    let event_with_sequence = UTC_EVENT.replace(
        "SUMMARY:Synthetic planning review",
        "SUMMARY:Synthetic planning review\r\nSEQUENCE:1",
    );
    let created = service
        .create_event(&tenant, &collection.collection_ref, &event_with_sequence)
        .expect("valid UTC VEVENT is accepted");

    assert_eq!(created.uid, "synthetic-event-1@example.test");
    assert_eq!(created.summary, "Synthetic planning review");
    assert_eq!(created.status, EventStatus::Confirmed);
    assert_eq!(created.revision, 1);
    assert!(created.etag.starts_with('"') && created.etag.ends_with('"'));
    assert_eq!(
        service
            .list_events(&tenant, &collection.collection_ref)
            .unwrap(),
        vec![created.clone()]
    );
    assert_eq!(
        service
            .get_event(&tenant, &collection.collection_ref, &created.event_ref)
            .unwrap(),
        created
    );
}

#[test]
fn event_status_is_preserved_without_importing_conflict_policy() {
    let tenant = tenant("synthetic-tenant-a");
    let mut service = InMemoryCalendarService::new();
    let collection = service
        .create_collection(&tenant, "Synthetic calendar")
        .unwrap();
    for (status_text, expected) in [
        ("CONFIRMED", EventStatus::Confirmed),
        ("TENTATIVE", EventStatus::Tentative),
        ("CANCELLED", EventStatus::Cancelled),
    ] {
        let payload = UTC_EVENT
            .replace("synthetic-event-1", &format!("synthetic-{status_text}"))
            .replace(
                "SUMMARY:Synthetic planning review",
                &format!("SUMMARY:Synthetic planning review\r\nSTATUS:{status_text}"),
            );
        assert_eq!(
            service
                .create_event(&tenant, &collection.collection_ref, &payload)
                .unwrap()
                .status,
            expected
        );
    }

    for payload in [
        UTC_EVENT.replace(
            "SUMMARY:Synthetic planning review",
            "SUMMARY:Synthetic planning review\r\nSTATUS:UNKNOWN",
        ),
        UTC_EVENT.replace(
            "SUMMARY:Synthetic planning review",
            "SUMMARY:Synthetic planning review\r\nSTATUS;X-TEST=1:CONFIRMED",
        ),
        UTC_EVENT.replace(
            "SUMMARY:Synthetic planning review",
            "SUMMARY:Synthetic planning review\r\nSTATUS:CONFIRMED\r\nSTATUS:CANCELLED",
        ),
    ] {
        assert_eq!(
            service.create_event(&tenant, &collection.collection_ref, &payload),
            Err(CalendarError::MalformedCalendar)
        );
    }
}

#[test]
fn cross_tenant_access_is_indistinguishable_from_an_unknown_collection() {
    let mut service = InMemoryCalendarService::new();
    let owner = tenant("synthetic-tenant-a");
    let outsider = tenant("synthetic-tenant-b");
    let collection = service
        .create_collection(&owner, "Synthetic calendar")
        .unwrap();
    let event = service
        .create_event(&owner, &collection.collection_ref, UTC_EVENT)
        .unwrap();

    assert_eq!(
        service.list_events(&outsider, &collection.collection_ref),
        Err(CalendarError::NotFound)
    );
    assert_eq!(
        service.get_event(&outsider, &collection.collection_ref, &event.event_ref),
        Err(CalendarError::NotFound)
    );
    assert_eq!(
        service.create_event(&outsider, &collection.collection_ref, "not iCalendar"),
        Err(CalendarError::NotFound)
    );
    assert_eq!(
        service.update_event(
            &outsider,
            &collection.collection_ref,
            &event.event_ref,
            &event.etag,
            "not iCalendar",
        ),
        Err(CalendarError::NotFound)
    );
}

#[test]
fn conditional_update_is_idempotent_and_rejects_stale_writers() {
    let tenant = tenant("synthetic-tenant-a");
    let mut service = InMemoryCalendarService::new();
    let collection = service
        .create_collection(&tenant, "Synthetic calendar")
        .unwrap();
    let created = service
        .create_event(&tenant, &collection.collection_ref, UTC_EVENT)
        .unwrap();
    let changed_payload =
        UTC_EVENT.replace("Synthetic planning review", "Synthetic revised review");

    let updated = service
        .update_event(
            &tenant,
            &collection.collection_ref,
            &created.event_ref,
            &created.etag,
            &changed_payload,
        )
        .unwrap();
    assert_eq!(updated.event_ref, created.event_ref);
    assert_eq!(updated.revision, 2);
    assert_ne!(updated.etag, created.etag);
    assert_eq!(updated.summary, "Synthetic revised review");
    assert_eq!(
        service.update_event(
            &tenant,
            &collection.collection_ref,
            &created.event_ref,
            &created.etag,
            &changed_payload,
        ),
        Err(CalendarError::StaleRevision)
    );
    assert_eq!(
        service
            .update_event(
                &tenant,
                &collection.collection_ref,
                &updated.event_ref,
                &updated.etag,
                &changed_payload,
            )
            .unwrap(),
        updated
    );
    assert_eq!(
        service.update_event(
            &tenant,
            &collection.collection_ref,
            &updated.event_ref,
            &updated.etag,
            &changed_payload.replace("synthetic-event-1", "synthetic-event-2"),
        ),
        Err(CalendarError::InvalidInput)
    );
    assert_eq!(
        service.update_event(
            &tenant,
            &collection.collection_ref,
            &updated.event_ref,
            &updated.etag,
            "not iCalendar",
        ),
        Err(CalendarError::MalformedCalendar)
    );
}

#[test]
fn malformed_or_unsupported_calendar_input_fails_closed() {
    let mut service = InMemoryCalendarService::new();
    let tenant = tenant("synthetic-tenant-a");
    let collection = service
        .create_collection(&tenant, "Synthetic calendar")
        .unwrap();

    assert_eq!(
        service.create_event(&tenant, &collection.collection_ref, "not iCalendar"),
        Err(CalendarError::MalformedCalendar)
    );
    assert_eq!(
        service.create_event(
            &tenant,
            &collection.collection_ref,
            &UTC_EVENT.replace(
                "DTSTART:20260902T090000Z",
                "DTSTART;TZID=Asia/Seoul:20260902T090000"
            )
        ),
        Err(CalendarError::UnsupportedCapability)
    );
}

#[test]
fn duplicate_uid_is_idempotent_only_for_identical_content() {
    let mut service = InMemoryCalendarService::new();
    let tenant = tenant("synthetic-tenant-a");
    let collection = service
        .create_collection(&tenant, "Synthetic calendar")
        .unwrap();
    let first = service
        .create_event(&tenant, &collection.collection_ref, UTC_EVENT)
        .unwrap();
    assert_eq!(
        service
            .create_event(&tenant, &collection.collection_ref, UTC_EVENT)
            .unwrap(),
        first
    );

    let changed = UTC_EVENT.replace("Synthetic planning review", "Changed summary");
    assert_eq!(
        service.create_event(&tenant, &collection.collection_ref, &changed),
        Err(CalendarError::StaleRevision)
    );
}

#[test]
fn identity_and_collection_inputs_are_bounded() {
    assert_eq!(TenantId::parse(""), Err(CalendarError::InvalidInput));
    assert_eq!(
        TenantId::parse("tenant with spaces"),
        Err(CalendarError::InvalidInput)
    );
    assert_eq!(
        TenantId::parse(&"a".repeat(129)),
        Err(CalendarError::InvalidInput)
    );

    let tenant = tenant("synthetic-tenant-a");
    let mut service = InMemoryCalendarService::new();
    assert_eq!(
        service.create_collection(&tenant, "  "),
        Err(CalendarError::InvalidInput)
    );
    assert_eq!(
        service.create_collection(&tenant, &"a".repeat(201)),
        Err(CalendarError::InvalidInput)
    );
}

#[test]
fn all_day_dates_use_an_exclusive_end() {
    let tenant = tenant("synthetic-tenant-a");
    let mut service = InMemoryCalendarService::new();
    let collection = service
        .create_collection(&tenant, "Synthetic calendar")
        .unwrap();
    assert!(
        service
            .create_event(&tenant, &collection.collection_ref, ALL_DAY_EVENT)
            .is_ok()
    );

    let reversed = ALL_DAY_EVENT.replace("DTEND;VALUE=DATE:20260903", "DTEND;VALUE=DATE:20260901");
    assert_eq!(
        service.create_event(&tenant, &collection.collection_ref, &reversed),
        Err(CalendarError::MalformedCalendar)
    );
    let mixed = ALL_DAY_EVENT.replace("DTEND;VALUE=DATE:20260903", "DTEND:20260903T000000Z");
    assert_eq!(
        service.create_event(&tenant, &collection.collection_ref, &mixed),
        Err(CalendarError::MalformedCalendar)
    );
}

#[test]
fn unknown_resources_and_owner_resources_share_not_found() {
    let tenant = tenant("synthetic-tenant-a");
    let mut service = InMemoryCalendarService::new();
    let collection = service
        .create_collection(&tenant, "Synthetic calendar")
        .unwrap();
    assert_eq!(
        service.list_events(&tenant, "cal_unknown"),
        Err(CalendarError::NotFound)
    );
    assert_eq!(
        service.get_event(&tenant, &collection.collection_ref, "evt_unknown"),
        Err(CalendarError::NotFound)
    );
    assert_eq!(
        service.create_event(&tenant, "cal_unknown", UTC_EVENT),
        Err(CalendarError::NotFound)
    );
    assert_eq!(
        service.update_event(
            &tenant,
            &collection.collection_ref,
            "evt_unknown",
            "\"evt_unknown:1\"",
            UTC_EVENT,
        ),
        Err(CalendarError::NotFound)
    );
}

#[test]
fn required_rfc_properties_and_supported_profile_fail_closed() {
    let tenant = tenant("synthetic-tenant-a");
    let mut service = InMemoryCalendarService::new();
    let collection = service
        .create_collection(&tenant, "Synthetic calendar")
        .unwrap();
    let malformed = [
        UTC_EVENT.replace("\r\n", "\n"),
        UTC_EVENT.replace("VERSION:2.0\r\n", "VERSION:2.0\nX-BROKEN:1\r\n"),
        UTC_EVENT.replace("VERSION:2.0\r\n", ""),
        UTC_EVENT.replace("VERSION:2.0", "VERSION:1.0"),
        UTC_EVENT.replace(
            "PRODID:-//ContextualWisdomLab//CalendarWeave v1//EN",
            "PRODID:",
        ),
        UTC_EVENT.replace(
            "PRODID:-//ContextualWisdomLab//CalendarWeave v1//EN\r\n",
            "PRODID:-//ContextualWisdomLab//CalendarWeave v1//EN\r\nMETHOD:PUBLISH\r\n",
        ),
        UTC_EVENT.replace("BEGIN:VEVENT\r\n", ""),
        UTC_EVENT.replace("UID:synthetic-event-1@example.test", "UID:"),
        UTC_EVENT.replace(
            "UID:synthetic-event-1@example.test",
            "UID;X-TEST=1:synthetic-event-1@example.test",
        ),
        UTC_EVENT.replace("SUMMARY:Synthetic planning review\r\n", ""),
        UTC_EVENT.replace(
            "SUMMARY:Synthetic planning review",
            "SUMMARY:Synthetic planning review\r\nSUMMARY:Duplicate summary",
        ),
        UTC_EVENT.replace("DTSTAMP:20260901T000000Z", "DTSTAMP:local-time"),
        UTC_EVENT.replace(
            "DTSTAMP:20260901T000000Z",
            "DTSTAMP;X-TEST=1:20260901T000000Z",
        ),
        UTC_EVENT.replace("DTSTAMP:20260901T000000Z\r\n", ""),
        UTC_EVENT.replace("DTSTART:20260902T090000Z\r\n", ""),
        UTC_EVENT.replace("DTEND:20260902T100000Z\r\n", ""),
        UTC_EVENT.replace("DTEND:20260902T100000Z", "DTEND:20260902T080000Z"),
        ALL_DAY_EVENT.replace("DTSTART;VALUE=DATE:20260902", "DTSTART;VALUE=DATE:not-a-date"),
        ALL_DAY_EVENT.replace("DTEND;VALUE=DATE:20260903", "DTEND;VALUE=DATE:not-a-date"),
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//ContextualWisdomLab//CalendarWeave v1//EN\r\nEND:VCALENDAR\r\n".to_owned(),
        UTC_EVENT.replace(
            "END:VCALENDAR\r\n",
            "BEGIN:VEVENT\r\nUID:synthetic-event-2@example.test\r\nDTSTAMP:20260901T000000Z\r\nDTSTART:20260902T110000Z\r\nDTEND:20260902T120000Z\r\nSUMMARY:Second synthetic event\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
        ),
        UTC_EVENT
            .replace("BEGIN:VEVENT", "BEGIN:VTODO")
            .replace("END:VEVENT", "END:VTODO"),
        UTC_EVENT.replace(
            "SUMMARY:Synthetic planning review",
            "SUMMARY:Synthetic planning review\r\nSEQUENCE:1\r\nSEQUENCE:2",
        ),
        UTC_EVENT.replace(
            "SUMMARY:Synthetic planning review",
            "SUMMARY:Synthetic planning review\r\nSEQUENCE:not-a-number",
        ),
        UTC_EVENT.replace(
            "SUMMARY:Synthetic planning review",
            "SUMMARY:Synthetic planning review\r\nSEQUENCE;X-TEST=1:1",
        ),
    ];
    for payload in malformed {
        assert_eq!(
            service.create_event(&tenant, &collection.collection_ref, &payload),
            Err(CalendarError::MalformedCalendar)
        );
    }

    let unsupported = [
        UTC_EVENT.replace(
            "SUMMARY:Synthetic planning review",
            "SUMMARY:Synthetic planning review\r\nLOCATION:Room 1",
        ),
        UTC_EVENT.replace(
            "SUMMARY:Synthetic planning review",
            "SUMMARY:Synthetic planning review\r\nATTENDEE:mailto:a@example.test\r\nATTENDEE:mailto:b@example.test",
        ),
        UTC_EVENT.replace(
            "SUMMARY:Synthetic planning review",
            "SUMMARY:Synthetic planning review\r\nBEGIN:VALARM\r\nACTION:DISPLAY\r\nTRIGGER:-PT5M\r\nDESCRIPTION:Synthetic reminder\r\nEND:VALARM",
        ),
        UTC_EVENT.replace("DTSTART:20260902T090000Z", "DTSTART:20260902T090000"),
        UTC_EVENT.replace(
            "DTSTART:20260902T090000Z",
            "DTSTART;X-TEST=1:20260902T090000Z",
        ),
        UTC_EVENT.replace(
            "DTEND:20260902T100000Z",
            "DTEND;TZID=Asia/Seoul:20260902T100000",
        ),
        UTC_EVENT.replace("DTEND:20260902T100000Z", "DTEND;X-TEST=1:20260902T100000Z"),
        UTC_EVENT.replace("DTEND:20260902T100000Z", "DTEND:not-a-date-time"),
    ];
    for payload in unsupported {
        assert_eq!(
            service.create_event(&tenant, &collection.collection_ref, &payload),
            Err(CalendarError::UnsupportedCapability)
        );
    }
}
