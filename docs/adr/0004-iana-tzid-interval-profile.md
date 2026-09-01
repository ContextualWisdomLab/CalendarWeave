# ADR-0004: IANA TZID interval profile

- Status: Proposed implementation candidate
- Date: 2026-09-01
- Depends on: ADR-0002

## Context

ADR-0002 accepts only UTC and all-day intervals. CalendarWeave issue #2 also
requires named-timezone semantics before consumers can characterize migration
parity. Full `VTIMEZONE`, floating time, recurrence, and provider timezone
mapping are separate capabilities and need not be invented for this slice.

## Decision

Extend the shared event parser with one bounded named-timezone profile:

- `DTSTART` and `DTEND` may each carry exactly one identical `TZID` parameter;
- the parameter value must be a known IANA timezone identifier;
- both local date-times must resolve to one unambiguous instant in that zone;
- the resolved end instant must be later than the resolved start instant;
- mixed UTC/named time, different identifiers, unknown identifiers, ambiguous
  local times, and nonexistent local times fail closed;
- all-day `DATE` values cannot carry `TZID`.

The accepted source remains unchanged in the revision payload. CalendarWeave
uses the pinned IANA timezone database only to validate and order instants. It
does not claim full `VTIMEZONE` interoperability: calendars containing a
`VTIMEZONE` component remain unsupported, as do floating time, `DURATION`, and
recurrence. Consumers needing portable custom timezone definitions must wait
for a later explicit capability.

An unknown timezone is `UnsupportedCapability`; malformed, mixed, mismatched,
ambiguous, nonexistent, or non-increasing intervals are `MalformedCalendar`.
Authorization continues to run before parsing in every adapter.

## Consequences

The in-memory and PostgreSQL adapters gain the same named-timezone semantics
through the existing shared parser without a schema or public-model change.
This advances the first resource vertical but does not establish CalDAV,
provider, service-authentication, or consumer-parity evidence.

## References

Desruisseaux, B. (2009). *Internet calendaring and scheduling core object
specification* (RFC 5545, Section 3.2.19). RFC Editor.
https://www.rfc-editor.org/rfc/rfc5545

Internet Assigned Numbers Authority. (2026). *Time zone database*.
https://www.iana.org/time-zones
