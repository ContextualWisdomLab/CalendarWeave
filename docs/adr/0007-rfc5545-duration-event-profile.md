# ADR-0007: Bounded RFC 5545 VEVENT DURATION profile

- Status: Accepted candidate
- Date: 2026-09-02
- Owners: Calendar Resource Core
- Supersedes: none

## Context

CalendarWeave's current executable VEVENT profile requires an explicit `DTEND`. That is narrower than RFC 5545, which allows `DURATION` as the event-length alternative and forbids `DTEND` and `DURATION` from appearing together. The missing alternative prevents common duration-based iCalendar producers from using the shared Calendar Resource Core even when the event otherwise fits the published v1 profile.

The repository remains an early-stage candidate stack rather than a released calendar service. The safest commercialization step is therefore a small parser-level capability that is shared by both the in-memory and PostgreSQL adapters and does not introduce recurrence, `VTIMEZONE`, floating time, provider SDKs, or a second persistence representation.

## Decision

CalendarWeave accepts a `DURATION` property as the alternative to `DTEND` for the bounded v1 VEVENT profile.

The executable invariants are:

1. Exactly one of `DTEND` or `DURATION` is required by the CalendarWeave v1 profile. RFC 5545 permits a VEVENT with neither, but CalendarWeave continues to require an explicit interval contract until implicit one-day/zero-duration semantics are deliberately versioned.
2. `DTEND` and `DURATION` are mutually exclusive and each is a singleton.
3. A VEVENT `DURATION` must be positive. A leading `+` or no sign is accepted; negative and zero durations are rejected.
4. The accepted lexical grammar is RFC 5545 `dur-value`: weeks, days, or the ordered day/hour/minute/second forms. Years, calendar months, fractional units, mixed week/date forms, and reordered components are rejected.
5. When `DTSTART` has `VALUE=DATE`, `DURATION` is restricted to `dur-day` or `dur-week` as required by RFC 5545.
6. UTC and the existing bounded IANA `TZID` start profiles are accepted. Existing fail-closed handling of unknown, ambiguous, nonexistent, floating, or parameter-mismatched starts remains unchanged.
7. RFC 5545 allows IANA and non-standard parameters on `DURATION`; CalendarWeave v1 does not interpret them yet and reports them as `UnsupportedCapability` rather than silently ignoring them.
8. CalendarWeave validates and preserves the original canonical resource payload. It does not invent a computed end timestamp. Future free/busy or recurrence expansion that needs an exact end must apply RFC 5545 nominal-duration rules, including calendar discontinuities and greatest-order-first addition.

## Domain and persistence impact

`CalendarEvent` remains the same aggregate projection: immutable UID, collection membership, current revision, strong ETag, summary/status projection, and original validated iCalendar payload. `DURATION` is an iCalendar semantic, not a new aggregate or persistence table. PostgreSQL continues to store the original `icalendar_payload`; therefore no migration, denormalized duration cache, or transaction-boundary change is introduced.

The change stays in the Calendar Resource Core parser so in-memory and durable adapters cannot diverge on the acceptance contract. Consumer conflict policy and free/busy interpretation remain outside this slice.

## Verification

The regression contract is committed before production behavior and covers UTC, named IANA timezone, DATE all-day, explicit positive sign, week/day/date-time shapes, mutual exclusion, missing interval, duplicate property, zero/negative values, grammar ordering, all-day restrictions, and unsupported parameters.

Exact-head repository tests, coverage, central semantic reviews, and ordinary governance remain required before integration. Hosted-runner queue starvation is incomplete evidence, not a pass.

## Consequences

This closes a common RFC 5545 interoperability gap without broadening CalendarWeave into recurrence or timezone-component ownership. `VTIMEZONE`, floating time, recurrence, CalDAV endpoints, provider parity, real service authentication, production recovery objectives, and release/consumer migration remain separate commercialization gaps.

## References

Desruisseaux, B. (Ed.). (2009). *Internet calendaring and scheduling core object specification (iCalendar)* (RFC 5545). RFC Editor. https://doi.org/10.17487/RFC5545
