# ADR-0002: Calendar Resource Core v1 application port

- Status: Proposed implementation candidate
- Date: 2026-09-01

## Context

ADR-0001 assigns generic calendar-resource authority to CalendarWeave, but the
repository has no executable contract. Consumer migration cannot begin from
architecture prose alone. The first vertical needs real RFC 5545 input,
tenant isolation, opaque identity, revision evidence, and an application port
without prematurely claiming WebDAV, provider synchronization, or durable
service operation.

## Decision

CalendarWeave v0.1 introduces a Rust library application port with an
in-process reference adapter. The port creates a tenant-owned calendar
collection and creates, lists, or gets immutable event revisions through
opaque collection and event references. Every lookup includes the tenant
scope; an unknown resource and another tenant's resource both return the same
`NotFound` outcome.

The accepted RFC 5545 profile is deliberately narrow:

- exactly one `VCALENDAR` with `VERSION:2.0` and non-empty `PRODID`;
- exactly one `VEVENT` with one each of `UID`, `DTSTAMP`, `DTSTART`, `DTEND`,
  and `SUMMARY`;
- UTC `DATE-TIME` intervals or all-day `DATE` intervals with an exclusive end;
- optional singleton `SEQUENCE` and `STATUS`; omitted `STATUS` means
  `CONFIRMED`, while `CONFIRMED`, `TENTATIVE`, and `CANCELLED` are preserved;
- CRLF input and preserved validated source representation;
- identical repeated create by collection plus UID is idempotent;
- changed content for an existing UID fails as `StaleRevision` rather than
  overwriting evidence.

CalendarWeave does not decide whether a cancelled or tentative event occupies
time; that remains consumer conflict policy. TZID/floating time, recurrence,
attendees, alarms, scheduling, provider
mapping, sync, update/delete, and CalDAV/WebDAV are unavailable capabilities.
The parser rejects them explicitly instead of silently discarding fields.

The application port is the consumer contract. The in-memory adapter is an
executable reference and conformance fixture, not durable persistence and not
a production service claim. A later ADR must define storage, concurrency,
authorization admission, HTTP/CalDAV protocol, and recovery before release.

## Dependency decision

- `icalendar` 0.17 parses RFC 5545 component and property structure;
- `chrono` 0.4 validates and orders the bounded UTC/date values;
- `uuid` 1.26 generates opaque v4 resource references.

CalendarWeave performs its own semantic and multiplicity validation because
parser acceptance alone does not enforce this product profile and duplicate
singleton properties may otherwise be overwritten.

## Quality contract

Production source must pass stable Rust 1.97.1 formatting, Clippy, tests, and
documentation checks. A separately pinned nightly coverage job measures LLVM
branch coverage and requires 100% lines and branches; this does not change the
production compiler pin. Synthetic RFC fixtures cover tenant denial, malformed
input, unsupported capabilities, duplicate singleton rejection, UTC and
all-day intervals, idempotency, and stale revision behavior.

## Consequences

Consumers can begin writing versioned contract fixtures against a real narrow
port. They still cannot migrate provider or CalDAV ownership because the
adapter is non-durable and no network service or released artifact exists.
Unsupported RFC features fail closed, which favors evidence integrity over
illusory compatibility.

## References

Desruisseaux, B. (2009). *Internet calendaring and scheduling core object
specification (iCalendar)* (RFC 5545). RFC Editor.
https://doi.org/10.17487/RFC5545

Daboo, C., Desruisseaux, B., & Dusseault, L. M. (2007). *Calendaring
extensions to WebDAV (CalDAV)* (RFC 4791). RFC Editor.
https://doi.org/10.17487/RFC4791
