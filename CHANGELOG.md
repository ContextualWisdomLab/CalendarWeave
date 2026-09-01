# Changelog

## Unreleased

- Reframed the repository README around CalendarWeave's customer-facing calendar-resource value, current release boundary, integration responsibilities, architecture, quality posture, and next actions without advertising unreleased runtime capabilities.
- Established the repository's original source and documentation under Apache License 2.0 after verifying the seed and architecture branch contain organization-owned documentation and no inherited third-party source license.
- Seeded the customer-facing README and ADR baseline so CalendarWeave is a real product repository rather than an empty organization stub.
- Add the candidate Rust Calendar Resource Core v1 application port with tenant-scoped collection and strict VEVENT create/list/get behavior.
- Preserve standard confirmed, tentative, and cancelled VEVENT status without importing consumer conflict policy.
- Add tenant-safe strong-ETag conditional update with immutable UID and authorization-before-parse error ordering.
- Add a PostgreSQL 3NF persistence candidate with restart-stable item-level create idempotency, append-only revisions, and serialized ETag concurrency.
- Fail closed for malformed, cross-tenant, stale-revision, and unsupported calendar requests with 100% owned line and branch coverage.
- Replace repository-local `ubuntu-latest` selectors with explicit `ubuntu-24.04`, preserving PostgreSQL service coverage, after the same hosted-runner starvation signature proven by central `.github` #1618; add a permanent two-job selector regression.
- Refresh the product/technical gap baseline from live architecture, implementation, persistence, review-control and operability evidence without promoting candidate branches to shipped evidence.
- Next: advance timezone/interval parity, external authorization admission, operated backup/recovery, versioned release evidence, and consumer migration gates.
