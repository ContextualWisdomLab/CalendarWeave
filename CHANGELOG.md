# Changelog

## Unreleased

- Seeded buyer README and ADR baseline so CalendarWeave is a real repo, not an empty org stub.
- Next: Draft CalDAV create/list/get that reproduces a real VEVENT (UID, DTSTART, DTEND, SUMMARY).
- Add the candidate Rust Calendar Resource Core v1 application port with
  tenant-scoped collection and strict VEVENT create/list/get behavior.
- Preserve standard confirmed, tentative, and cancelled VEVENT status without
  importing consumer conflict policy.
- Add tenant-safe strong-ETag conditional update with immutable UID and
  authorization-before-parse error ordering.
- Add a PostgreSQL 3NF persistence candidate with restart-stable create
  idempotency, append-only revisions, and serialized ETag concurrency.
- Fail closed for malformed, cross-tenant, stale-revision, and unsupported
  calendar requests with 100% owned line and branch coverage.
