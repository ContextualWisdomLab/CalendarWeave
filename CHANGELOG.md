# Changelog

## Unreleased

- Reframed the repository README around CalendarWeave's customer-facing calendar-resource value, current release boundary, integration responsibilities, architecture, quality posture, and next actions without advertising unreleased runtime capabilities.
- Established the repository's original source and documentation under Apache License 2.0 after verifying the seed and architecture branch contain organization-owned documentation and no inherited third-party source license.
- Seeded the customer-facing README and ADR baseline so CalendarWeave is a real product repository rather than an empty organization stub.
- Add the candidate Rust Calendar Resource Core v1 application port with tenant-scoped collection and strict VEVENT create/list/get behavior.
- Preserve standard confirmed, tentative, and cancelled VEVENT status without importing consumer conflict policy.
- Add tenant-safe strong-ETag conditional update with immutable UID and authorization-before-parse error ordering.
- Fail closed for malformed, cross-tenant, stale-revision, and unsupported calendar requests with 100% owned line and branch coverage.
- Replace the repository-local floating `ubuntu-latest` selectors with explicit `ubuntu-24.04` after the same hosted-runner starvation signature proven by central `.github` #1618; add a permanent regression requiring both Rust and coverage jobs to keep the explicit image. The repaired exact predecessor head acquired runners and completed both jobs successfully.
- Refresh the product/technical gap baseline from the live stack: executable core, durable PostgreSQL and bounded IANA timezone candidates are distinguished from protected-main/release evidence, while authorization, recovery, RFC/CalDAV, privacy, packaging and consumer migration remain open.
- Next: advance the candidate stack through current-head semantic review, external authorization, operated recovery, standards-backed calendar capability, versioned release evidence, and consumer migration gates.
