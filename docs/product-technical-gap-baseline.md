# CalendarWeave product and technical gap baseline

## Snapshot

Protected `main` remains the seed `d972ccae6225716bdff7210a1fed808c01d32689`; the live repository is non-fork and protected main still contains no released runtime, package, service, container, CalDAV endpoint, provider adapter, or consumer migration. The current candidate stack is PR #1 architecture/ownership, #3 Rust Calendar Resource Core, #4 PostgreSQL persistence, #5 bounded IANA `TZID`, #6 authorization admission, #7 logical recovery, #8 bounded RFC 5545 `DURATION`, and #9 RFC 5545 `CLASS` privacy intent.

CalendarWeave remains genuinely very early-stage because the buyer-facing workflow is not installable or operated and foundational service authentication, release/deployment, CalDAV/provider parity, privacy/audit operation, measured recovery, and downstream migration are still missing. PR #9 is a bounded commercialization intervention because `saju-caldav` parity explicitly needs privacy-classification semantics and RFC 5545 provides a standard contract that can be added without widening CalendarWeave into disclosure policy.

## Product responsibility and DDD boundary

| Responsibility | Owner / bounded context | Current evidence |
| --- | --- | --- |
| Calendar collections, events, UID/revision/ETag invariants, RFC 5545 resource semantics | CalendarWeave / Calendar Resource Core | #3 plus stacked #5/#8/#9 candidates; not released |
| Calendar privacy intent | CalendarWeave / Calendar Resource Core | #9 `EventClass` projection from canonical iCalendar; not authorization |
| Calendar operation admission | CalendarWeave / Authorization Admission | #6 candidate; tenant-free issuer/subject evidence, exact resource request, authorization-derived tenant |
| Identity/federation and external authorization policy | Keyverse | External authority behind `CalendarAuthorizationPort`; no copied identity/policy store |
| Relational durability/concurrency | CalendarWeave / PostgreSQL adapter | #4 candidate, 3NF append-only revisions and row-locked conditional updates |
| Logical recovery | CalendarWeave / operations boundary | #7 candidate; checksum-before-restore and invariant drill, not PITR/HA/RPO/RTO evidence |
| CalDAV/provider interoperability and synchronization | CalendarWeave / interoperability adapters | Target responsibility; no released endpoint/provider parity |
| Workspace commitment/conflict/resolution policy | Naruon | Supporting consumer context behind a versioned Calendar Port/ACL |
| Calendar/evidence composition | LineageWeave | Read-only composition/deep-link responsibility; no calendar store or mathematical computation |
| Saju scoring/explanation/publication intent | `saju-caldav` | Separate domain; generic CalDAV compatibility migrates only after CalendarWeave parity |
| Deterministic Four Pillars computation | `four-pillars` | Separate mathematical product responsibility |

Core subdomain: governed calendar-resource semantics and mutation/revision invariants. Supporting subdomains: Authorization Admission plus CalDAV/provider interoperability. Generic/external capabilities: identity/federation, PostgreSQL, telemetry and deployment platform. `CalendarCollection` owns collection-scoped membership. `CalendarEvent` is the current immutable-UID revision projection. `TenantId` and tenant-free `ExternalIdentity` are value objects. `EventClass` is descriptive privacy intent. `CalendarAuthorizationRequest` carries action and opaque resource references only. External identity/provider representations terminate behind ACLs.

Transactions remain item-scoped: create is idempotent by collection + RFC UID; conditional update locks one event row and advances one revision under the expected strong ETag. `CLASS` is derived from the canonical immutable `icalendar_payload`; #9 deliberately adds no duplicate persistence column or transaction boundary.

## Current feature specification

The candidate Calendar Resource Core supports tenant-scoped collection create and VEVENT create/update/list/get. Supported VEVENTs require RFC 5545 `VERSION:2.0`, `PRODID`, UID, UTC `DTSTAMP`, start, summary, and exactly one explicit interval form. Standard confirmed/tentative/cancelled status and non-negative `SEQUENCE` are bounded optional fields.

Time and privacy behavior is explicit:

- `DTEND` supports UTC, all-day DATE, and matching bounded IANA `TZID` intervals, rejecting non-increasing intervals.
- #8 accepts positive RFC 5545 `DURATION` as the alternative to `DTEND`; both, neither, duplicate, negative/zero, calendar-month/year, fractional, reordered, and unsupported-parameter forms fail closed under the bounded v1 profile.
- DATE `DTSTART` accepts only day/week duration forms. Named-timezone starts reuse the existing ambiguity/nonexistence/unknown-zone fail-closed contract.
- #9 accepts one optional RFC 5545 `CLASS`. Omission projects as `PUBLIC`; `PUBLIC`/`PRIVATE`/`CONFIDENTIAL` are case-insensitive; valid unrecognized IANA/experimental token values project as `PRIVATE`; IANA/non-standard parameters remain interoperable; duplicate, empty or non-token values fail malformed.
- `CLASS` is calendar-owner intent only. A public value cannot override denied/unavailable authorization, and private/confidential values do not themselves grant or enforce access.
- Floating local time, `VTIMEZONE`, recurrence/free-busy expansion and unversioned provider/CalDAV capabilities remain outside the candidate profile.

Persistence remains 3NF with descriptive multiword `snake_case` objects: `calendar_collection`, `calendar_event`, and `calendar_event_revision`. Canonical event content remains `icalendar_payload`; neither DURATION nor CLASS introduces a parallel relational source of truth.

## Exact stack evidence observed in this iteration

| Lane | Exact head / evidence | Current status / next verification |
| --- | --- | --- |
| protected `main` | `d972ccae6225716bdff7210a1fed808c01d32689` | seed only; no released product surface |
| #1 `docs/adr-baseline` | `d9393fd7e4e6e3ad72d0f09acdf656d6569555f6` | open Draft architecture parent; documentation is not executable completion evidence |
| #3 `feat/calendar-resource-core-v1` | `e4d3defec07fa00cd909ed676ea88c5c898d32db` | predecessor exact repository Tests were terminal success; Ready transition was retried in this iteration and is currently blocked by GitHub GraphQL rate limiting, not bypassed |
| #4 `feat/postgres-calendar-store-v1` | `77f8b66560c999385eae90ff038643e2e948fabf` | exact repository Tests previously terminal success; preserve stack order and reacquire lifecycle mutation when API is healthy |
| #5 `feat/tzid-calendar-interval-v1` | `b68a1c566f0fee520459f297a2f94a2ffa5bac24` | exact repository Tests previously terminal success; preserve stack order and ordinary review gates |
| #6 `feat/authorization-admission-v1` | `9b3500633b4b7c7a9ac1e43dda10140ec0f1aedc` | live REST confirms open/non-Draft exact head; Tests run `33563830224` remains queued, so no pass is claimed |
| #7 `feat/postgres-recovery-v1` | `1473e0ae8e9ddbe3343190941f6125dbcac03bcc` | live REST confirms open Draft exact head; recovery/rust/coverage exact-head jobs remain required and queued/unassigned |
| #8 `feat/rfc5545-duration-v1` | `7521b6b39170c29b71d11fc90d48e9000f7bcce8` | live REST confirms open Draft exact head; current-head Tests remains required before Ready/merge progression |
| #9 `feat/rfc5545-class-v1` | pre-baseline head `2052e46cb03bc4c8db8404c2edfe4b5cd9a75cc6`; this baseline commit advances the head | RED-first CLASS implementation + ADR/research/architecture/README/contributor alignment; exact-head checks must regenerate after this commit |
| central runner acquisition | ContextualWisdomLab/.github #712 | current central evidence identifies avoidable COMMENTED-review scheduler wakeups as one causal queue-amplification defect; do not churn leaf heads or declare queued jobs passing |

The live governance path requires exact-current-head checks/reviews. This run attempted the safe Draft→Ready transition for #3; the GraphQL API returned a rate-limit error, so no repeated mutation storm, self-approval, admin bypass, or protection weakening was used.

## PR #9 TDD and research traceability

The initial RED commit `b91602811a231c726ab5fbc5a2e0a1af894e9346` added `tests/rfc5545_classification.rs` before production CLASS support. A standards audit then corrected overly narrow assumptions before final production behavior: `18d3ea264d2f0c3bfeea10e5af6fa01ff4bbe706` requires case-insensitive standard values, extension-parameter interoperability, and fail-private handling for valid unknown registered/experimental values. `707fcecdb45a273028b2d7966888c3a507d268d5` implements that corrected Rust contract.

ADR-0008 and `docs/doctoring/rfc5545-class-privacy-baseline.md` bind the candidate to RFC 5545 sections 3.1 and 3.8.1.3. RFC 5545 defines omitted CLASS as PUBLIC, permits IANA/non-standard parameters, requires unrecognized iana-token/x-name values to be treated like PRIVATE, and explicitly warns that CLASS is owner intent rather than an enforcement statement. The same RFC states enumerated values are case-insensitive. These semantics are represented directly rather than replaced with a local heuristic.

The classification projection is derived from the validated immutable event payload. This keeps persistence normalized and avoids introducing a synchronization invariant between a second classification column and canonical iCalendar content.

## Open issue state

Issue #2 remains the canonical commercialization tracker and stays open. #9 addresses only the generic RFC 5545 CLASS portion of `saju-caldav` parity. Current evidence still does not prove released CalDAV/provider parity, concrete service authentication, operated disaster recovery, privacy/retention/export/audit controls, versioned distribution, or consumer cutover.

## Commercialization gaps

| Gap | Owner | Current evidence | Smallest next action | Completion evidence |
| --- | --- | --- | --- | --- |
| Service authentication / Keyverse integration | CalendarWeave + Keyverse | #6 proves an in-process authorization ACL only | implement a concrete verified issuer/token/session adapter behind the existing port | invalid signature/issuer/algorithm/audience/subject/time, dependency failure, tenant/resource mismatch and authorized fixtures |
| Operated durability | CalendarWeave deployment | #7 logical restore candidate only | encrypted retained remote backups, WAL/PITR where required, rollback/monitoring and measured exercises | exact measured RPO/RTO/PITR/restore evidence; no logical-backup overclaim |
| `VTIMEZONE` / remaining RFC 5545 profile | Calendar Resource Core | bounded IANA zone + DURATION + CLASS candidates | add standards-backed `VTIMEZONE` slice test-first | real RFC fixtures and DST edges under exact-head coverage |
| CalDAV/provider parity | CalendarWeave interoperability | no endpoint/provider adapter released | add protocol/provider ACL after the core contract stabilizes | real provider/CalDAV fixtures and reversible migration evidence |
| Privacy/content + authorization audit | CalendarWeave + deployment | CLASS intent candidate, necessary calendar PII, logical backup | define purpose/retention/access/export/audit and encrypted backup access | CSAP/SOC 2-oriented control map plus operated evidence without certification claims |
| Release/package/service | CalendarWeave | no versioned artifact | define package/service contract, compose deployment, SBOM/provenance and rollback | immutable versioned artifact/service plus real install/call path |
| Consumer migration | Naruon / `saju-caldav` / LineageWeave | compatibility implementations remain | characterization tests then versioned ACLs after release | parity/security/failure semantics, no direct table coupling, reversible cutover |
| Hosted exact-head verification | ContextualWisdomLab/.github #712 | CalendarWeave #6/#7/#8/#9 runner-backed jobs can remain queued/unassigned | repair central queue-amplification owner path and revalidate unchanged leaf heads | terminal current-head repository + semantic/security evidence |
| Draft lifecycle mutation | GitHub API/control plane | #3/#4/#5 are mechanically mature enough for review dispatch but mutation is currently rate-limited | retry once API health returns, without bypass or no-op churn | ordinary Ready state and downstream independent review dispatch |

## Quality, security, persistence and operability invariants

- Behavior changes begin with executable RED contracts; owned production statement/branch coverage and public-doc coverage target 100%.
- No deprecation-warning suppression, production synthetic data, self-approval, force-push, destructive rebase, routine bypass, or protection weakening.
- CalendarWeave and LineageWeave contain no mathematical/psychometric computation that belongs in dedicated mathematical owners.
- Authorization precedes untrusted calendar parsing/mutation. Cross-tenant and absent-resource observations remain indistinguishable at the core boundary.
- RFC 5545 `CLASS` is not access-control authority; valid unknown tokens fail-private to avoid accidental widening.
- Necessary calendar PII is protected through least privilege, purpose/tenant isolation, encryption, retention, export/access audit and test anonymization rather than blanket masking that breaks calendar work.
- Relational persistence stays normalized; item-level idempotency/UPSERT semantics remain explicit; writes lock only the required item.
- Logical-backup digest verification is integrity evidence, not encryption, provenance, PITR, HA or RPO/RTO evidence.
- Web p95/k6 gates become applicable only when a web/service surface exists; no absent web surface is represented as load-tested.

## Required development order

1. Reacquire exact-head repository and semantic/security evidence for #6/#7/#8/#9 while independently reducing real product gaps.
2. Transition #3/#4/#5 to Ready when GitHub GraphQL mutation health permits; preserve stack order and ordinary review gates.
3. Repair the central queue-amplification owner path without leaf churn, then revalidate unchanged CalendarWeave heads.
4. Establish concrete Keyverse/service authentication and operated recovery/release evidence.
5. Add the next standards-backed `VTIMEZONE` capability, then CalDAV/provider interoperability fixtures.
6. Migrate Naruon, `saju-caldav`, and LineageWeave only after released parity evidence exists.

## Evidence references

- CalendarWeave PRs #1, #3, #4, #5, #6, #7, #8, #9 and issue #2.
- ADR-0001 through ADR-0008.
- `docs/doctoring/identity-authorization-admission-baseline.md`.
- `docs/doctoring/postgresql-logical-recovery-baseline.md`.
- `docs/doctoring/rfc5545-duration-baseline.md`.
- `docs/doctoring/rfc5545-class-privacy-baseline.md`.
- ContextualWisdomLab/.github #712 for organization runner-queue causal evidence.
- Desruisseaux, B. (Ed.). (2009). *Internet calendaring and scheduling core object specification (iCalendar)* (RFC 5545). RFC Editor. https://doi.org/10.17487/RFC5545