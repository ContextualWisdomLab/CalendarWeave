# CalendarWeave product and technical gap baseline

## Snapshot

Protected `main` remains the seed `d972ccae6225716bdff7210a1fed808c01d32689`; the live repository is non-fork and protected main still contains no released runtime, package, service, container, CalDAV endpoint, provider adapter, or consumer migration. PRs #3/#4/#5 have merged into the open PR #1 branch, which now carries the Rust core, PostgreSQL adapter and bounded IANA `TZID`. PR #6 authorization admission, #7 logical recovery, #8 bounded RFC 5545 `DURATION`, and #9 RFC 5545 `CLASS` privacy intent remain stacked candidates.

CalendarWeave remains genuinely very early-stage because the buyer-facing workflow is not installable or operated and foundational service authentication, release/deployment, CalDAV/provider parity, privacy/audit operation, measured recovery, and downstream migration are still missing. PR #9 is a bounded commercialization intervention because `saju-caldav` parity explicitly needs privacy-classification semantics and RFC 5545 provides a standard contract that can be added without widening CalendarWeave into disclosure policy.

## Product responsibility and DDD boundary

| Responsibility | Owner / bounded context | Current evidence |
| --- | --- | --- |
| Calendar collections, events, UID/revision/ETag invariants, RFC 5545 resource semantics | CalendarWeave / Calendar Resource Core | #3/#5 content is integrated on PR #1; #8/#9 are later semantic candidates; none is released |
| Calendar privacy intent | CalendarWeave / Calendar Resource Core | #9 `EventClass` projection from canonical iCalendar; not authorization |
| Calendar operation admission | CalendarWeave / Authorization Admission | #6 candidate; tenant-free issuer/subject evidence, exact resource request, authorization-derived tenant |
| Identity/federation and external authorization policy | Keyverse | External authority behind `CalendarAuthorizationPort`; no copied identity/policy store |
| Relational durability/concurrency | CalendarWeave / PostgreSQL adapter | #4 content is integrated on PR #1, with 3NF append-only revisions and row-locked conditional updates |
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
- `CalendarEvent::classification()` revalidates the same complete bounded event profile before returning a classification, so a manually forged projection with an unsupported property or other invalid raw payload cannot recover apparently trusted privacy metadata.
- `CLASS` is calendar-owner intent only. A public value cannot override denied/unavailable authorization, and private/confidential values do not themselves grant or enforce access.
- Floating local time, `VTIMEZONE`, recurrence/free-busy expansion and unversioned provider/CalDAV capabilities remain outside the candidate profile.

Persistence remains 3NF with descriptive multiword `snake_case` objects: `calendar_collection`, `calendar_event`, and `calendar_event_revision`. Canonical event content remains `icalendar_payload`; neither DURATION nor CLASS introduces a parallel relational source of truth.

## Exact stack evidence observed in this iteration

| Lane | Exact head / evidence | Current status / next verification |
| --- | --- | --- |
| protected `main` | `d972ccae6225716bdff7210a1fed808c01d32689` | seed only; no released product surface |
| #1 `docs/adr-baseline` | PRs #3/#4/#5 merged into this branch; current candidate covers revision exhaustion and malformed named-timezone end values | Executable core, PostgreSQL and bounded `TZID` are integrated but not on protected `main`; replacement exact-head required checks and review govern its merge |
| PRs #3/#4/#5 | Closed and merged into PR #1 branch | Their successful exact-head Rust and coverage checks supported branch integration; PR #1 needs its own checks before protected-main merge |
| #6 `feat/authorization-admission-v1` | Open candidate based on PR #1 | Current-head hosted checks and semantic review remain required; the open review thread requests hosted check evidence |
| #7 `feat/postgres-recovery-v1` | Open candidate stacked after #6 | RED recovery contract preceded production scripts; current-head rust, coverage and recovery checks remain required |
| #8 `feat/rfc5545-duration-v1` | Open candidate stacked after #7 | Test-first `DURATION` lane; current-head rust, coverage and recovery checks remain required |
| #9 `feat/rfc5545-class-v1` | Open candidate stacked after #8 | RED-first `CLASS` implementation and forged-projection hardening; current-head checks and review remain required |
| central runner acquisition | ContextualWisdomLab/.github #712 | current central evidence identifies avoidable COMMENTED-review scheduler wakeups as one causal queue-amplification defect; do not churn leaf heads or declare queued jobs passing |

The live governance path requires exact-current-head checks/reviews. PRs #3/#4/#5 were marked Ready through the CLI and merged into PR #1 without self-approval, admin bypass or protection weakening.

## PR #9 TDD and research traceability

The initial RED commit `b91602811a231c726ab5fbc5a2e0a1af894e9346` added `tests/rfc5545_classification.rs` before production CLASS support. A standards audit then corrected overly narrow assumptions before final production behavior: `18d3ea264d2f0c3bfeea10e5af6fa01ff4bbe706` requires case-insensitive standard values, extension-parameter interoperability, and fail-private handling for valid unknown registered/experimental values. `707fcecdb45a273028b2d7966888c3a507d268d5` implements that corrected Rust contract.

A subsequent exact-head static review found that the public classification accessor re-parsed only singleton/component/classification structure rather than the entire supported event profile. That meant a manually forged public `CalendarEvent` could retain a valid `CLASS` while gaining an unsupported property and still return a classification. `bfe078a556677ec99d76c87533bcbd5967836da6` added the RED regression first; `b12f95dee6b3a73f87a56316a48830f45ae3612b` then made the accessor reuse `parse_event` and moved the parsed classification into `ParsedEvent`, so classification now inherits the same fail-closed full-profile validation as event admission.

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
| Review lifecycle | CalendarWeave PR stack | #3/#4/#5 were marked Ready through the CLI and merged into PR #1; #6/#7/#8/#9 remain Draft | advance each executable candidate with current-head checks and review evidence, without bypass or no-op churn | ordinary Ready state and downstream independent review dispatch |

## Quality, security, persistence and operability invariants

- Behavior changes begin with executable RED contracts; owned production statement/branch coverage and public-doc coverage target 100%.
- No deprecation-warning suppression, production synthetic data, self-approval, force-push, destructive rebase, routine bypass, or protection weakening.
- CalendarWeave and LineageWeave contain no mathematical/psychometric computation that belongs in dedicated mathematical owners.
- Authorization precedes untrusted calendar parsing/mutation. Cross-tenant and absent-resource observations remain indistinguishable at the core boundary.
- RFC 5545 `CLASS` is not access-control authority; valid unknown tokens fail-private to avoid accidental widening, and public classification projections must pass the full supported-event validator before being trusted.
- No raw bearer token/provider credential becomes a Calendar Resource attribute or ordinary telemetry field.
- Necessary calendar PII is protected through least privilege, purpose/tenant isolation, encryption, retention, export/access audit and test anonymization rather than blanket masking that breaks calendar work.
- Relational persistence stays normalized; item-level idempotency/UPSERT semantics remain explicit; writes lock only the required item.
- Logical-backup digest verification is integrity evidence, not encryption, provenance, PITR, HA or RPO/RTO evidence.
- Web p95/k6 gates become applicable only when a web/service surface exists; no absent web surface is represented as load-tested.
- The revision counter must fail closed at `u64::MAX` without replacing the current event; named-timezone parsing must reject malformed local end values before interval comparison.
- Exact-head checks, live reviews/threads, rulesets and concurrent writer state are re-read after every branch move; stale/queued/cancelled evidence is non-passing.
- Logical restore must remain fail-closed on missing/malformed/tampered evidence and prove post-restore relational invariants. A successful logical drill alone cannot satisfy production recovery/PITR/RPO/RTO gates.
- Release additionally requires security/SBOM/provenance, compose-compatible operability, rollback/recovery, real service authentication, realistic protocol/consumer evidence and PII/audit controls.

## Required development order

1. Reacquire exact-head repository and semantic/security evidence for #6/#7/#8/#9 while independently reducing real product gaps.
2. Review PR #1's integrated core and advance #6/#7/#8/#9 only after their current-head checks; preserve stack order and ordinary review gates.
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
