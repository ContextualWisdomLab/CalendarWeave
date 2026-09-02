# RFC 5545 VEVENT DURATION research baseline

## Scope

This note binds CalendarWeave's bounded VEVENT `DURATION` implementation to the standards text used for the exact implementation branch. It is implementation traceability, not a claim of full RFC 5545 or CalDAV conformance.

## Standards findings

RFC 5545 section 3.6.1 defines `DTEND` and `DURATION` as alternative VEVENT properties: either may occur, but they MUST NOT occur together. The same section states that a DATE-valued `DTSTART` paired with `DURATION` requires a day or week duration. RFC 5545 section 3.8.2.5 defines the `DURATION` property as a positive duration. Section 3.3.6 defines the duration value grammar: an optional sign followed by `P` and either a week form, a day form optionally followed by time, or a time form; years and calendar-month duration designators are not part of this iCalendar value type.

The duration data-type text also distinguishes nominal week/day durations from accurate hour/minute/second durations. Across daylight-saving or other time-scale discontinuities, exact-duration computation must account for the discontinuity and add greatest-order components first. CalendarWeave therefore validates and preserves `DURATION` without converting it to a guessed fixed-second end timestamp in this slice.

## Source-to-code traceability

| Standard requirement / product restriction | Executable owner | Verification |
| --- | --- | --- |
| `DTEND` and `DURATION` are mutually exclusive | `src/lib.rs::validate_singleton_properties` and `validate_event_interval` | `tests/rfc5545_duration.rs` both-present rejection |
| CalendarWeave bounded v1 continues to require one explicit interval form | same | neither-present rejection; ADR-0007 records this intentional profile restriction |
| VEVENT `DURATION` is positive | `positive_duration` | negative and all-zero regressions |
| RFC duration lexical ordering, no years/months/fractions | `positive_duration`, `duration_time_nonzero` | malformed grammar table |
| DATE `DTSTART` requires day/week duration | `validate_duration_interval` | all-day time-duration rejection plus day/week acceptance |
| Existing UTC/IANA start semantics remain shared | `validate_datetime_start`, `named_datetime` | UTC and `Asia/Seoul` acceptance; inherited named-timezone edge contracts |
| Uninterpreted `DURATION` parameters are not silently accepted | `validate_duration_interval` | `UnsupportedCapability` regression |
| No persistence duplication | existing `calendar_event_revision.icalendar_payload` contract | shared parser precedes both adapters; no migration added |

## Commercialization boundary

This slice improves producer interoperability but does not implement implicit VEVENT duration semantics when both `DTEND` and `DURATION` are absent, `VTIMEZONE`, floating local time, recurrence expansion, free/busy calculation, CalDAV transport, provider adapters, or scheduling. Those capabilities need their own standards-backed executable contracts before they can be advertised.

## References

Desruisseaux, B. (Ed.). (2009). *Internet calendaring and scheduling core object specification (iCalendar)* (RFC 5545). RFC Editor. https://doi.org/10.17487/RFC5545

RFC Editor. (2024). *Errata ID 6109 for RFC 5545*. https://www.rfc-editor.org/errata/eid6109
