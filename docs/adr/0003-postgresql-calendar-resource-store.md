# ADR-0003: PostgreSQL Calendar Resource Store v1

- Status: Proposed implementation candidate
- Date: 2026-09-01
- Depends on: ADR-0002

## Context

ADR-0002 proves the application contract only inside one process. Its
in-memory adapter cannot preserve identity across restart or serialize two
writers that present the same event ETag. CalendarWeave issue #2 requires a
durable production boundary before any consumer migration claim.

## Decision

Add a PostgreSQL adapter for the existing `CalendarPort`. PostgreSQL owns the
concurrency boundary: collection and event identity use unique constraints,
each update locks the event row, and an update appends one immutable revision
only when the caller's strong ETag matches the current revision. The adapter
checks tenant ownership before parsing caller payloads and returns the same
`NotFound` outcome for absent and cross-tenant resources.

The relational model is third normal form:

- `calendar_collection` owns collection identity, tenant scope, and display
  name;
- `calendar_event` owns stable event identity, collection membership, RFC UID,
  and the current revision pointer;
- `calendar_event_revision` owns revision-specific summary, status, and
  preserved iCalendar payload.

A deferred composite foreign key requires the current pointer to identify an
existing revision at commit. History is append-only; update inserts the next
revision and advances the event pointer in one transaction. The strong ETag is
derived from stable event reference plus revision instead of being stored
twice. Consumers use the application port and never read these tables directly.

## Failure and recovery

Database transport, constraint, and transaction failures return the bounded
`StorageUnavailable` outcome without exposing SQL or connection detail. A
failed transaction commits neither the current-pointer change nor the new
revision. Native PostgreSQL backup, restore, and point-in-time recovery remain
deployment responsibilities; this candidate does not claim that an operated
backup schedule exists.

## Verification

Real PostgreSQL integration tests cover migration replay, restart-stable
create idempotency, tenant isolation, current-revision reads, and two-connection
stale-writer rejection. Owned production statement and branch coverage remains
100%. The in-memory adapter remains a fast conformance fixture, not the
durability claim.

## Consequences

The candidate establishes durable identity and database-serialized ETag
updates. It still lacks an authenticated network service, Keyverse admission,
CalDAV sync, provider mappings, backup-operation evidence, and consumer parity;
therefore issue #2 remains open.

## References

PostgreSQL Global Development Group. (2026). *PostgreSQL 18 documentation:
Explicit locking*. https://www.postgresql.org/docs/18/explicit-locking.html

PostgreSQL Global Development Group. (2026). *PostgreSQL 18 documentation:
Constraints*. https://www.postgresql.org/docs/18/ddl-constraints.html
