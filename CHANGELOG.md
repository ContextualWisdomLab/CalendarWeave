# Changelog

## Unreleased

- Fix a time-of-check/time-of-use defect in `ops/postgres/restore_calendarweave.sh`: the archive is now opened exactly once, copied into an owner-only private copy (`0600` file in a `0700` `mktemp -d` directory), and that copy is both hashed and fed to `pg_restore`, so replacing the backup pathname after verification can no longer restore unverified bytes. The private copy is removed on every exit path; exit codes 65/66 and symlink rejection are unchanged.
- Extend the recovery drill with a real PostgreSQL swap-after-digest regression, private-custody/cleanup checks (success, mismatch, `pg_restore` failure, `TERM`), and an opt-in owned local-cluster mode; CI still runs the docker service-container path.

- Add the ADR-0010 source-candidate Calendar Port v0.1 metadata profile: optional single parameter-free DESCRIPTION and case-insensitive OPAQUE/TRANSPARENT, original-byte preservation, folded-duplicate rejection, and explicit unsupported parameters without changing schema or dependencies.
- Retain twelve exact synthetic consumer serializer fixtures with provenance, metadata boundary tests, and real admission-wrapper controls using synthetic authorization decisions; no release, deployed identity, free/busy, or consumer migration is claimed.

- Add a working local PostgreSQL operator executable over the retained Calendar Port: explicit initialization, collection creation, original-byte VEVENT create/list/get, and strong-ETag update.
- Preserve the complete CLASS/DURATION/admission/recovery stack and package preflight without rewriting predecessor branches.
- Reject implicit, DNS, remote and hostaddr-override database targets before the non-TLS operator adapter connects; retain only explicit loopback IPs and absolute UNIX sockets.
- Exercise the real executable in separate processes with disposable PostgreSQL, including durable readback, idempotency, stale revisions, cross-tenant denial, storage failure, credential containment and broken output.
- Add the local operator quickstart and ADR-0009, distinguishing usable candidate artifacts from authenticated service, release, operated recovery and consumer migration.

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
- Fail closed for malformed, cross-tenant, stale-revision, unsupported, denied, and authorization-unavailable calendar requests with 100% owned line and branch coverage.
- Prove that an exhausted `u64` event revision fails closed without replacing the stored event, and reject malformed named-timezone end values through the public parser contract.
- Add a test-first PostgreSQL logical recovery candidate: owner-only custom-format backup artifacts, SHA-256 verification before restore, single-transaction restore, and a separate-database drill that proves calendar data plus current-revision and collection+UID relational invariants survive recovery while tampered artifacts fail before target mutation.
- Record PostgreSQL 18 recovery traceability in ADR-0006 and doctoring; keep WAL/PITR, backup-store encryption/retention, HA/failover and measured RPO/RTO as explicit deployment commercialization gaps rather than inferred claims.
- Add a test-first bounded RFC 5545 `DURATION` VEVENT slice under ADR-0007: positive day/week/date-time duration grammar, explicit `DTEND`/`DURATION` mutual exclusion, DATE-start day/week rules, UTC/IANA start reuse, and fail-closed unsupported duration parameters without adding a parallel persistence model.
- Add a test-first RFC 5545 `CLASS` privacy-intent slice under ADR-0008: omitted events project as `PUBLIC`, standard classifications are case-insensitive, unknown registered/experimental tokens fail-private, extension parameters remain interoperable, and authorization stays separate from descriptive calendar metadata.
- Repair the earlier `STATUS` projection to honor RFC 5545 case-insensitive enumerated values, with an exact failing hosted contract preceding the bounded parser fix.
- Reject folded duplicate singleton properties by counting the same unfolded RFC 5545 content lines consumed by the dependency parser, preventing a second folded `CLASS` from replacing the first value.
- Route `DTEND`/`DURATION` mutual exclusion through one interval validator and cover hour-minute, minute-second, malformed day-time, suffix, and start-parameter edges without weakening the RFC profile.
- Add `CLAUDE.md` as a contributor-context pointer and align `AGENTS.md` with the tenant-free identity / authorization-derived tenant contract so contributor guidance matches the executable admission boundary.
- Replace repository-local `ubuntu-latest` selectors with explicit `ubuntu-24.04`, preserving PostgreSQL service coverage, after the same hosted-runner starvation signature proven by central `.github` #1618; add a permanent two-job selector regression.
- Refresh the product/technical gap baseline from live architecture, implementation, persistence, time-semantics, authorization, review-control and operability evidence without promoting candidate branches to shipped evidence.
- Next: establish concrete service/Keyverse authentication, measured production recovery/PITR posture, standards-backed `VTIMEZONE` capability, versioned release evidence, and consumer migration gates.
