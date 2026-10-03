//! ADR-0010 bounded metadata admission using exact synthetic consumer output.

use calendarweave::admission::{
    AuthorizationError, AuthorizedCalendarService, CalendarAuthorizationPort,
    CalendarAuthorizationRequest, ExternalIdentity,
};
use calendarweave::{
    CalendarError, CalendarEvent, CalendarPort, EventClass, InMemoryCalendarService, TenantId,
};

const BASE: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//CalendarWeave//Metadata test//EN\r\nBEGIN:VEVENT\r\nUID:metadata@example.test\r\nDTSTAMP:20261003T000000Z\r\nDTSTART:20261004T090000Z\r\nDTEND:20261004T100000Z\r\nSUMMARY:Synthetic metadata\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

fn payload(lines: &str) -> String {
    BASE.replace("END:VEVENT", &format!("{lines}\r\nEND:VEVENT"))
}

fn create(input: &str) -> Result<CalendarEvent, CalendarError> {
    let tenant = TenantId::parse("metadata-test").unwrap();
    let mut service = InMemoryCalendarService::new();
    let collection = service.create_collection(&tenant, "Metadata").unwrap();
    service.create_event(&tenant, &collection.collection_ref, input)
}

macro_rules! consumer_fixture {
    ($name:ident, $file:literal, $class:ident) => {
        #[test]
        fn $name() {
            let input = include_str!(concat!("fixtures/saju_event_metadata/", $file, ".ics"));
            let tenant = TenantId::parse("synthetic-consumer").unwrap();
            let outsider = TenantId::parse("synthetic-outsider").unwrap();
            let mut service = InMemoryCalendarService::new();
            let collection = service
                .create_collection(&tenant, "Consumer fixture")
                .unwrap();
            let created = service
                .create_event(&tenant, &collection.collection_ref, input)
                .expect("exact real serializer output must be admitted");
            assert_eq!(created.icalendar.as_bytes(), input.as_bytes());
            assert_eq!(created.classification(), Ok(EventClass::$class));
            assert_eq!(created.uid, "4242d64e05db4852f1f0b726b4e6f621@saju-caldav");
            assert_eq!(
                service.create_event(&tenant, &collection.collection_ref, input),
                Ok(created.clone())
            );
            assert_eq!(
                service.get_event(&tenant, &collection.collection_ref, &created.event_ref),
                Ok(created.clone())
            );
            assert_eq!(
                service.list_events(&tenant, &collection.collection_ref),
                Ok(vec![created.clone()])
            );
            assert_eq!(
                service.get_event(&outsider, &collection.collection_ref, &created.event_ref),
                Err(CalendarError::NotFound)
            );
            assert_eq!(
                service.create_event(&outsider, &collection.collection_ref, input),
                Err(CalendarError::NotFound)
            );
            assert_eq!(
                service.list_events(&outsider, &collection.collection_ref),
                Err(CalendarError::NotFound)
            );
            assert_eq!(
                service.update_event(
                    &outsider,
                    &collection.collection_ref,
                    &created.event_ref,
                    &created.etag,
                    input
                ),
                Err(CalendarError::NotFound)
            );
            assert_eq!(
                service.update_event(
                    &tenant,
                    &collection.collection_ref,
                    &created.event_ref,
                    &created.etag,
                    input
                ),
                Ok(created.clone())
            );
            let replacement = input.replace("TRANSP:TRANSPARENT", "TRANSP:OPAQUE");
            assert_eq!(
                service.create_event(&tenant, &collection.collection_ref, &replacement),
                Err(CalendarError::StaleRevision)
            );
            assert_eq!(
                service.update_event(
                    &tenant,
                    &collection.collection_ref,
                    &created.event_ref,
                    "\"stale\"",
                    &replacement
                ),
                Err(CalendarError::StaleRevision)
            );
            let updated = service
                .update_event(
                    &tenant,
                    &collection.collection_ref,
                    &created.event_ref,
                    &created.etag,
                    &replacement,
                )
                .unwrap();
            assert_eq!(updated.revision, 2);
            assert_ne!(updated.etag, created.etag);
            assert_eq!(updated.icalendar.as_bytes(), replacement.as_bytes());
            assert_eq!(
                service.update_event(
                    &tenant,
                    &collection.collection_ref,
                    &created.event_ref,
                    &created.etag,
                    input
                ),
                Err(CalendarError::StaleRevision)
            );
            assert_eq!(
                service.get_event(&tenant, &collection.collection_ref, &created.event_ref),
                Ok(updated)
            );
        }
    };
}

consumer_fixture!(normal_public, "normal-public", Public);
consumer_fixture!(normal_private, "normal-private", Private);
consumer_fixture!(normal_confidential, "normal-confidential", Confidential);
consumer_fixture!(folded_public, "folded-public", Public);
consumer_fixture!(folded_private, "folded-private", Private);
consumer_fixture!(folded_confidential, "folded-confidential", Confidential);
consumer_fixture!(escaped_public, "escaped-public", Public);
consumer_fixture!(escaped_private, "escaped-private", Private);
consumer_fixture!(escaped_confidential, "escaped-confidential", Confidential);
consumer_fixture!(overlong_public, "overlong-public", Public);
consumer_fixture!(overlong_private, "overlong-private", Private);
consumer_fixture!(overlong_confidential, "overlong-confidential", Confidential);

#[test]
fn description_preserves_empty_escaped_and_folded_text() {
    for line in [
        "DESCRIPTION:",
        "DESCRIPTION: ",
        "DESCRIPTION:한글 café 😀: \"quoted\"\\,semi\\;slash\\\\line\\nnext\\Nend",
        "DESCRIPTION:한글 café 😀\r\n continued\r\n\ttext",
    ] {
        let input = payload(line);
        assert_eq!(
            create(&input).unwrap().icalendar.as_bytes(),
            input.as_bytes()
        );
    }
}

#[test]
fn transparency_omission_and_standard_values_preserve_exact_content() {
    for lines in [
        "DESCRIPTION:No explicit transparency",
        "TRANSP:OPAQUE",
        "TRANSP:opaque",
        "TRANSP:oPaQuE",
        "TRANSP:TRANSPARENT",
        "TRANSP:transparent",
        "TRANSP:tRaNsPaReNt",
    ] {
        let input = payload(lines);
        assert_eq!(
            create(&input).unwrap().icalendar.as_bytes(),
            input.as_bytes()
        );
    }
}

#[test]
fn both_metadata_properties_reject_duplicates_before_dependency_overwrite() {
    for lines in [
        "DESCRIPTION:first\r\nDESCRIPTION:second",
        "DESCRIPTION:first\r\nDESCRIP\r\n TION:second",
        "DESCRIPTION:first\r\nDESCRIP\r\n\tTION:second",
        "TRANSP:OPAQUE\r\nTRANSP:TRANSPARENT",
        "TRANSP:OPAQUE\r\nTRA\r\n NSP:TRANSPARENT",
        "TRANSP:OPAQUE\r\nTRA\r\n\tNSP:TRANSPARENT",
    ] {
        assert_eq!(
            create(&payload(lines)),
            Err(CalendarError::MalformedCalendar),
            "{lines:?}"
        );
    }
}

#[test]
fn logical_content_lines_preserve_valid_multi_indent_folds() {
    for indent in [" ", "\t", "  ", "\t\t", " \t", "\t "] {
        let input = payload(&format!(
            "DESCRIPTION:first\r\n{indent}DESCRIPTION:still text\r\n{indent}CLASS:PUBLIC\r\nCLASS:PRIVATE"
        ));
        let event = create(&input).unwrap();
        assert_eq!(event.icalendar.as_bytes(), input.as_bytes());
        assert_eq!(event.classification(), Ok(EventClass::Private));
    }
    let input = payload("DESCRIP\r\n TION:text\r\nTRA\r\n\tNSP:OPAQUE");
    assert_eq!(
        create(&input).unwrap().icalendar.as_bytes(),
        input.as_bytes()
    );
}

macro_rules! blank_line_overwrite {
    ($name:ident, $first:literal, $second:literal) => {
        #[test]
        fn $name() {
            let mut admitted = Vec::new();
            for blank_lines in [1, 2, 3] {
                for indent in ["", " ", "\t", "  ", "\t\t", " \t", "\t "] {
                    for second in [$second.to_owned(), $second.to_ascii_lowercase()] {
                        let lines = format!(
                            "{}{}\r\n{indent}{second}",
                            $first,
                            "\r\n".repeat(blank_lines)
                        );
                        let result = create(&payload(&lines));
                        if result != Err(CalendarError::MalformedCalendar) {
                            admitted.push((lines, result));
                        }
                    }
                }
            }
            assert!(admitted.is_empty(), "non-malformed results: {admitted:#?}");
        }
    };
}

blank_line_overwrite!(
    logical_content_lines_reject_description_overwrite,
    "DESCRIPTION;LANGUAGE=ko:first",
    "DESCRIPTION:second"
);
blank_line_overwrite!(
    logical_content_lines_reject_transparency_overwrite,
    "TRANSP:BUSY",
    "TRANSP:OPAQUE"
);
blank_line_overwrite!(
    logical_content_lines_reject_class_overwrite,
    "CLASS:PRIVATE",
    "CLASS:PUBLIC"
);
blank_line_overwrite!(
    logical_content_lines_reject_status_overwrite,
    "STATUS:BUSY",
    "STATUS:CONFIRMED"
);

#[test]
fn logical_content_lines_reject_required_singleton_overwrite() {
    for second in [
        "UID:replacement@example.test",
        "DTSTAMP:20261003T000000Z",
        "DTSTART:20261004T090000Z",
        "DTEND:20261004T100000Z",
        "SUMMARY:replacement",
        "SEQUENCE:1\r\n\r\n  SEQUENCE:2",
    ] {
        for indent in ["  ", "\t\t", " \t", "\t "] {
            let input = payload(&format!("DESCRIPTION:text\r\n\r\n{indent}{second}"));
            assert_eq!(
                create(&input),
                Err(CalendarError::MalformedCalendar),
                "{input:?}"
            );
        }
    }
}

#[test]
fn logical_content_lines_reject_blank_and_whitespace_only_lines() {
    for blank in ["", " ", "\t", "  ", "\t\t", " \t", "\t "] {
        for boundary in [
            "BEGIN:VCALENDAR",
            "BEGIN:VEVENT",
            "END:VEVENT",
            "END:VCALENDAR",
        ] {
            let input = BASE.replace(boundary, &format!("\r\n{blank}\r\n{boundary}"));
            assert_eq!(
                create(&input),
                Err(CalendarError::MalformedCalendar),
                "{input:?}"
            );
        }
        let input = format!("{BASE}\r\n{blank}\r\n");
        assert_eq!(
            create(&input),
            Err(CalendarError::MalformedCalendar),
            "{input:?}"
        );
    }
}

#[test]
fn logical_content_lines_reject_indented_component_boundaries() {
    for boundary in [
        "BEGIN:VCALENDAR",
        "BEGIN:VEVENT",
        "END:VEVENT",
        "END:VCALENDAR",
    ] {
        for indent in ["  ", "\t\t", " \t", "\t "] {
            let input = BASE.replace(boundary, &format!("\r\n{indent}{boundary}"));
            assert_eq!(
                create(&input),
                Err(CalendarError::MalformedCalendar),
                "{input:?}"
            );
        }
    }
}

#[test]
fn bare_cr_cannot_hide_unsupported_description_parameters() {
    assert_eq!(
        create(&payload(
            "DESCRIPTION;LANGUAGE=ko:first\r\n\rDESCRIPTION:second"
        )),
        Err(CalendarError::MalformedCalendar)
    );
}

#[test]
fn bare_cr_cannot_hide_invalid_transparency() {
    assert_eq!(
        create(&payload("TRANSP:BUSY\r\n\rTRANSP:OPAQUE")),
        Err(CalendarError::MalformedCalendar)
    );
}

#[test]
fn bare_cr_cannot_bypass_other_singleton_property_guards() {
    for lines in [
        "STATUS:BUSY\r\n\rSTATUS:CONFIRMED",
        "CLASS;X-HINT=local:PRIVATE\r\n\rCLASS:PUBLIC",
        "\rUID:replacement@example.test",
        "\rDTSTAMP:20261003T000000Z",
        "\rDTSTART:20261004T090000Z",
        "\rDTEND:20261004T100000Z",
        "\rSUMMARY:replacement",
    ] {
        assert_eq!(
            create(&payload(lines)),
            Err(CalendarError::MalformedCalendar),
            "{lines:?}"
        );
    }
}

#[test]
fn bare_cr_and_lf_are_malformed_without_normalizing_valid_folds() {
    for separator in ["\r", "\n", "\r\r\n", "\n\r\n"] {
        let input = payload(&format!("DESCRIPTION:first{separator}second"));
        assert_eq!(create(&input), Err(CalendarError::MalformedCalendar));
    }
    for separator in ["\r\n", "\r\n ", "\r\n\t"] {
        let input = payload(&format!("DESCRIPTION:first{separator}TRANSP:OPAQUE"));
        assert_eq!(
            create(&input).unwrap().icalendar.as_bytes(),
            input.as_bytes()
        );
    }
}

#[test]
fn invalid_transparency_values_fail_malformed() {
    for value in [
        "",
        "BUSY",
        "X-FREE",
        "OPAQUE ",
        " TRANSPARENT",
        "OPAQUE,TRANSPARENT",
    ] {
        assert_eq!(
            create(&payload(&format!("TRANSP:{value}"))),
            Err(CalendarError::MalformedCalendar),
            "{value:?}"
        );
    }
}

#[test]
fn metadata_parameters_remain_explicitly_unsupported() {
    for line in [
        "DESCRIPTION;LANGUAGE=ko:text",
        "DESCRIPTION;ALTREP=\"https://example.test/text\":text",
        "DESCRIPTION;VALUE=TEXT:text",
        "DESCRIPTION;VALUE=URI:https://example.test/text",
        "DESCRIPTION;X-HINT=local:text",
        "TRANSP;VALUE=TEXT:OPAQUE",
        "TRANSP;X-HINT=local:TRANSPARENT",
    ] {
        assert_eq!(
            create(&payload(line)),
            Err(CalendarError::UnsupportedCapability),
            "{line}"
        );
    }
}

#[test]
fn metadata_does_not_allow_unknown_event_or_recurrence_properties() {
    for line in [
        "ATTENDEE:mailto:synthetic@example.test",
        "X-UNKNOWN:hint",
        "RRULE:FREQ=DAILY",
    ] {
        assert_eq!(
            create(&payload(&format!(
                "DESCRIPTION:text\r\nTRANSP:TRANSPARENT\r\n{line}"
            ))),
            Err(CalendarError::UnsupportedCapability)
        );
    }
    let timezone = payload("DESCRIPTION:text\r\nTRANSP:OPAQUE").replace(
        "BEGIN:VEVENT",
        "BEGIN:VTIMEZONE\r\nTZID:Asia/Seoul\r\nEND:VTIMEZONE\r\nBEGIN:VEVENT",
    );
    assert_eq!(create(&timezone), Err(CalendarError::MalformedCalendar));
}

// Synthetic decisions exercise the actual admission service, not a real issuer or PDP.
struct SyntheticDecision(Result<TenantId, AuthorizationError>);
impl CalendarAuthorizationPort for SyntheticDecision {
    fn authorize(
        &self,
        _identity: &ExternalIdentity,
        _request: &CalendarAuthorizationRequest<'_>,
    ) -> Result<TenantId, AuthorizationError> {
        self.0.clone()
    }
}

#[test]
fn metadata_never_overrides_denied_or_unavailable_authorization() {
    let identity =
        ExternalIdentity::parse("https://identity.example.test", "synthetic-user").unwrap();
    let input = include_str!("fixtures/saju_event_metadata/normal-public.ics");
    for (decision, expected) in [
        (AuthorizationError::Denied, CalendarError::Unauthorized),
        (
            AuthorizationError::Unavailable,
            CalendarError::AuthorizationUnavailable,
        ),
    ] {
        let mut service = AuthorizedCalendarService::new(
            SyntheticDecision(Err(decision)),
            InMemoryCalendarService::new(),
        );
        assert_eq!(
            service.create_event(&identity, "absent", input),
            Err(expected.clone())
        );
        assert_eq!(
            service.update_event(&identity, "absent", "absent", "\"stale\"", input),
            Err(expected)
        );
    }
}

#[test]
fn authorized_service_admits_nonempty_exact_consumer_content() {
    let identity =
        ExternalIdentity::parse("https://identity.example.test", "synthetic-user").unwrap();
    let mut service = AuthorizedCalendarService::new(
        SyntheticDecision(Ok(TenantId::parse("synthetic-consumer").unwrap())),
        InMemoryCalendarService::new(),
    );
    let collection = service.create_collection(&identity, "Metadata").unwrap();
    let input = include_str!("fixtures/saju_event_metadata/normal-private.ics");
    let created = service
        .create_event(&identity, &collection.collection_ref, input)
        .unwrap();
    assert_eq!(
        service.get_event(&identity, &collection.collection_ref, &created.event_ref),
        Ok(created)
    );
    assert_eq!(
        service
            .list_events(&identity, &collection.collection_ref)
            .unwrap()[0]
            .icalendar,
        input
    );
}
