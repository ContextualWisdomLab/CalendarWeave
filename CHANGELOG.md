# Changelog

## Unreleased

- Reframed the repository README around CalendarWeave's customer-facing calendar-resource value, current release boundary, integration responsibilities, architecture, quality posture, and next actions without advertising unreleased runtime capabilities.
- Established the repository's original source and documentation under Apache License 2.0 after verifying the seed and architecture branch contain organization-owned documentation and no inherited third-party source license.
- Seeded the customer-facing README and ADR baseline so CalendarWeave is a real product repository rather than an empty organization stub.
- Add the candidate Rust Calendar Resource Core v1 application port with tenant-scoped collection and strict VEVENT create/list/get behavior.
- Preserve standard confirmed, tentative, and cancelled VEVENT status without importing consumer conflict policy.
- Add tenant-safe strong-ETag conditional update with immutable UID and authorization-before-parse error ordering.
- Add a PostgreSQL 3NF persistence candidate with restart-stable item-level create idempotency, append-only revisions, and serialized ETag concurrency.
- Validate bounded matching IANA `TZID` intervals through the shared parser, rejecting unknown, mixed, mismatched, ambiguous, nonexistent, and non-increasing local-time intervals.
- Add a fail-closed external authorization admission candidate in which `ExternalIdentity` carries only verified issuer/subject evidence, `CalendarAuthorizationRequest` carries exact resource context, and the trusted authorization decision derives the tenant used by the Calendar Resource Core; callers cannot self-assert tenant scope through the admission API.
- Record RFC 7519/OpenID Connect identity traceability so issuer plus subject jointly identify an external principal and defensive API bounds do not become an invented subject-character grammar.
- Enforce resource-scoped authorization inputs so collection/event grants do not collapse into tenant-wide permission, while deny/unavailable states still authorize-before-parse and fail closed.
- Fail closed for malformed, cross-tenant, stale-revision, unsupported, denied, and authorization-unavailable calendar requests with exact-head coverage gates.
- Replace repository-local `ubuntu-latest` selectors with explicit `ubuntu-24.04`, preserving PostgreSQL service coverage, after the same hosted-runner starvation signature proven by central `.github` #1618; add a permanent two-job selector regression.
- Refresh the product/technical gap baseline from live architecture, implementation, persistence, time-semantics, authorization, review-control and operability evidence without promoting candidate branches to shipped evidence.
- Next: establish concrete service/Keyverse authentication, operated backup/recovery, standards-backed `DURATION`/`VTIMEZONE` capability slices, versioned release evidence, and consumer migration gates.
