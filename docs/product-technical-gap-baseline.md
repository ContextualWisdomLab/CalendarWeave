# CalendarWeave product and technical gap baseline

## Snapshot

Protected `main` remains the seed commit `d972ccae6225716bdff7210a1fed808c01d32689`; CalendarWeave still has no released runtime, package, service, container, CalDAV endpoint, or consumer migration. The active same-repository stack is candidate evidence: PR #1 architecture/ownership, #3 Rust Calendar Resource Core, #4 PostgreSQL persistence, #5 bounded IANA `TZID`, #6 authorization admission, #7 logical recovery, and #8 bounded RFC 5545 `DURATION` interoperability.

The repository remains genuinely early-stage because the buyer-facing workflow is not installable or operated and foundational service authentication, release/deployment, CalDAV/provider parity, privacy/audit operation, measured recovery, and downstream migration are still missing. The bounded `DURATION` slice materially narrows a real producer-interoperability gap without widening the product boundary.

## Product responsibility and DDD boundary

| Responsibility | Owner / bounded context | Current evidence |
| --- | --- | --- |
| Calendar collections, events, UID/revision/ETag invariants, RFC 5545 resource semantics | CalendarWeave / Calendar Resource Core | PR #3 plus stacked #5/#8 candidates; not released |
| Calendar operation admission | CalendarWeave / Authorization Admission | PR #6 candidate; tenant-free issuer/subject evidence, exact resource request, authorization-derived tenant |
| Identity/federation and external authorization policy | Keyverse | External authority behind `CalendarAuthorizationPort`; no copied identity/policy store |
| Relational durability/concurrency | CalendarWeave / PostgreSQL adapter | PR #4 candidate, 3NF append-only revisions and row-locked conditional updates |
| Logical recovery | CalendarWeave / operations boundary | PR #7 candidate; checksum-before-restore and invariant drill, not PITR/HA/RPO/RTO evidence |
| CalDAV/provider interoperability and synchronization | CalendarWeave / interoperability adapters | Target responsibility; no released endpoint/provider parity |
| Workspace commitment/conflict/resolution policy | Naruon | Supporting consumer context behind a versioned Calendar Port/ACL |
| Calendar/evidence composition | LineageWeave | Read-only composition/deep-link responsibility; no calendar store or mathematical computation |
| Saju scoring/explanation/publication intent | `saju-caldav` | Separate domain; generic CalDAV compatibility migrates only after CalendarWeave parity |
| Deterministic Four Pillars computation | `four-pillars` | Separate mathematical product responsibility |

Core subdomain: governed calendar-resource semantics and mutation/revision invariants. Supporting subdomains: authorization admission plus CalDAV/provider interoperability. Generic/external capabilities: identity/federation, PostgreSQL, telemetry, deployment platform. `CalendarCollection` owns collection-scoped membership; `CalendarEvent` exposes the current immutable-UID revision projection. `TenantId` and tenant-free `ExternalIdentity` are value objects. `CalendarAuthorizationRequest` carries action and opaque resource references only. External identity/provider DTOs terminate behind ACLs.

Item-level transactions remain minimal. Event create is idempotent by collection + RFC UID. Conditional update locks one event row and advances one revision only under the expected strong ETag. Recovery is outside ordinary aggregate transactions and must restore rather than redefine these invariants.

## Current feature specification

The candidate v1 Calendar Resource Core supports tenant-scoped collection create and VEVENT create/update/list/get. Supported VEVENTs require RFC 5545 `VERSION:2.0`, `PRODID`, UID, UTC `DTSTAMP`, start, summary, and exactly one explicit interval form. Standard confirmed/tentative/cancelled status and non-negative `SEQUENCE` are bounded optional fields.

Time/interval candidate behavior is explicit:

- `DTEND` remains supported for UTC, all-day DATE, and matching bounded IANA `TZID` intervals, with non-increasing intervals rejected.
- PR #8 adds positive RFC 5545 `DURATION` as the alternative to `DTEND`; both present, neither present, or duplicate interval fields fail closed under the CalendarWeave v1 profile.
- `DURATION` accepts RFC 5545 week/day/hour/minute/second lexical ordering, including explicit `+`; the optional-minute grammar is covered explicitly (`PT1H30S` and `P1DT1H30S`), while negative, zero, years, calendar months, fractions, mixed week/date forms, and reordered units fail closed.
- DATE `DTSTART` accepts only day/week duration forms as required by RFC 5545.
- The existing bounded profile continues to reject floating local time, unknown/ambiguous/nonexistent named local starts, `VTIMEZONE`, recurrence, and uninterpreted duration parameters until separately versioned. PR #8 now contains direct DURATION regressions for unknown, ambiguous, and nonexistent named starts rather than relying only on predecessor TZID tests.
- Nominal duration is preserved in the original iCalendar payload; this slice does not invent a fixed-second end across DST discontinuities.

Persistence remains 3NF with descriptive multiword `snake_case` objects: `calendar_collection`, `calendar_event`, and `calendar_event_revision`. No duration table or denormalized computed-end column is added; both adapters use the shared parser and the durable revision stores `icalendar_payload`.

## Exact-stack evidence and status

| Lane | Exact evidence observed in this iteration | Status / next verification |
| --- | --- | --- |
| protected `main` | `d972ccae6225716bdff7210a1fed808c01d32689` | Seed only; protected by active organization required-workflow/review ruleset |
| PR #1 `docs/adr-baseline` | `d9393fd7e4e6e3ad72d0f09acdf656d6569555f6` | Draft architecture parent; exact SAST/Security runs observed queued; documentation is not executable-product completion evidence |
| PR #3 `feat/calendar-resource-core-v1` | `e4d3defec07fa00cd909ed676ea88c5c898d32db` | Exact repository `Tests` run `33532894750` completed success; Draft→Ready mutation was retried live and is blocked by connector GraphQL `Repository.fullDatabaseId` schema failure |
| PR #4 `feat/postgres-calendar-store-v1` | `77f8b66560c999385eae90ff038643e2e948fabf` | Exact `Tests` run `33532977605` completed success; same live Ready-mutation connector failure |
| PR #5 `feat/tzid-calendar-interval-v1` | `b68a1c566f0fee520459f297a2f94a2ffa5bac24` | Exact `Tests` run `33533050155` completed success; same live Ready-mutation connector failure |
| PR #6 `feat/authorization-admission-v1` | `9b3500633b4b7c7a9ac1e43dda10140ec0f1aedc` | Open/non-Draft/mergeable; exact `Tests` run `33563830224` still queued; unresolved review thread correctly remains open because it asks for hosted exact-head check evidence |
| PR #7 `feat/postgres-recovery-v1` | `1473e0ae8e9ddbe3343190941f6125dbcac03bcc` | Draft/mergeable; exact `Tests` run `33569539054` still queued; recovery cannot be promoted without execution evidence |
| PR #8 `feat/rfc5545-duration-v1` | pre-baseline head `7e356a6f4eaa0fb60722b23e6c9953fcdea9df02`; this baseline commit advances the head | Test-first DURATION lane; exact-head `Tests` run `33580394726` had rust/coverage/recovery all queued with zero steps and no assigned runner, so it becomes historical after this commit; re-read the new exact head and reacquire checks/reviews |
| Central runner acquisition | ContextualWisdomLab/.github #712 | Current organization-level queue evidence continues to show explicit Ubuntu jobs unassigned; do not rewrite leaf runner selectors or claim queued as passing |

The live CalendarWeave organization ruleset requires an approving review, stale-review dismissal, review-thread resolution, and central required workflows on the protected default branch. No self-approval, admin bypass, required-check weakening, force-push, or destructive rebase is permitted for commercialization progress.

## PR #8 test-first and research traceability

The primary RED contract is commit `7b22940c1b26f69a79b66a50de72e0a436821600`, which added `tests/rfc5545_duration.rs` while production still excluded `DURATION` and required `DTEND`. The production parser implementation followed in `0606f8b72e97c21b3d64945dc8b22a4688ea6358`. A later exact RFC grammar audit found that RFC 5545 permits the hour form to omit minutes while still including seconds; `c48b2019031d579646ad7aa1f57beab492d85acd` added `PT1H30S`/`P1DT1H30S` plus named-timezone edge regressions before `7e356a6f4eaa0fb60722b23e6c9953fcdea9df02` repaired the parser. This preserves RED→GREEN ordering for the discovered edge defect independently of hosted-runner availability.

ADR-0007 and `docs/doctoring/rfc5545-duration-baseline.md` bind the behavior to RFC 5545 sections 3.3.6, 3.6.1, and 3.8.2.5. RFC 5545 defines `DTEND` and `DURATION` as mutually exclusive VEVENT alternatives, defines `DURATION` as positive, requires DATE-start durations to be day/week forms, and distinguishes nominal day/week duration across time-scale discontinuities. CalendarWeave's additional requirement that one explicit interval form be present is an intentional bounded v1 product restriction, not represented as full RFC conformance.

## Open issue state

Issue #2 remains the canonical commercialization tracker and must stay open. It maps to real product, security, reliability, interoperability, release, and ecosystem gaps. PR #8 addresses only the explicit `DURATION` portion of the RFC 5545 capability gap. The issue now contains a current PR #8 progress comment; no current evidence proves a versioned release, production authentication, CalDAV/provider parity, measured disaster recovery, privacy/audit operation, or downstream migration, so closure would be false.

## Commercialization gaps

| Gap | Owner | Current evidence | Smallest next action | Completion evidence |
| --- | --- | --- | --- | --- |
| Real service authentication / Keyverse integration | CalendarWeave + Keyverse boundary | PR #6 proves only an in-process authorization ACL | Implement concrete verified issuer/token/session adapter without copying identity policy into CalendarWeave | Invalid signature/issuer/algorithm/audience/subject/time, tenant/resource mismatch, dependency failure, and successful authorized operation fixtures |
| Operated durability | CalendarWeave deployment boundary | PR #7 logical restore candidate only | Add encrypted retained remote backups, migration rollback, WAL/PITR where required, monitoring, and measured exercises | Exact measured RPO/RTO/PITR/restore evidence; no logical-backup overclaim |
| `VTIMEZONE` and remaining RFC 5545 profile | Calendar Resource Core | PR #5 bounded IANA registry lookup + PR #8 DURATION | Add standards-backed `VTIMEZONE` slice test-first; keep floating/recurrence separate until justified | Real RFC fixtures and edge cases under exact-head coverage |
| CalDAV/provider parity | CalendarWeave interoperability | No endpoint/provider adapter released | Add protocol ACL only after core contract stabilizes | Real CalDAV/provider interoperability fixtures and reversible migration evidence |
| Privacy/content + authorization audit | CalendarWeave + deployment | Calendar text and backup artifacts may contain necessary PII | Define purpose/retention/access/export/audit controls and encrypted backup access | CSAP/SOC 2-oriented control map and operated evidence without certification claims |
| Release/package/service | CalendarWeave | No versioned public artifact | Define package/service contract, compose deployment, SBOM/provenance, rollback | Immutable versioned artifact/service plus real install/call path |
| Consumer migration | Naruon / `saju-caldav` / LineageWeave | Compatibility implementations remain | Characterization tests then versioned ACLs after release | No direct table coupling; parity/security/failure semantics and reversible cutover |
| Hosted exact-head verification | ContextualWisdomLab/.github #712 | #6/#7/#8 jobs queued/unassigned | Continue central runner-capacity/root-cause lane; do not create leaf no-op churn | Current-head terminal successful repository + semantic/security checks |
| Draft lifecycle mutation | GitHub connector | #3/#4/#5 executable with successful exact Tests but Ready mutation errors on `fullDatabaseId`; the later #8 Ready attempt also encountered a transient GraphQL rate limit | Retry only when the connector/API is healthy; meanwhile preserve PR state and do not bypass governance | Successful ordinary Ready transition and downstream semantic review dispatch |

## Quality, security, persistence, and operability invariants

- Behavior changes begin with executable RED contracts; owned production statement/branch coverage and public-doc coverage target 100%.
- No deprecation-warning suppression, production synthetic data, self-approval, force-push, destructive rebase, or protection weakening.
- Calendar math/psychometrics do not belong here; LineageWeave remains free of mathematical computation that belongs in a dedicated mathematical owner.
- Authorization precedes untrusted calendar parsing/mutation. Cross-tenant and absent-resource observations stay indistinguishable at the core boundary.
- No raw bearer token/provider credential becomes a Calendar Resource attribute or ordinary telemetry field.
- Necessary calendar PII is protected by least privilege, purpose/tenant isolation, encryption, retention and audit rather than masking that destroys calendar utility. Test/docs identities remain synthetic/anonymized.
- Relational persistence stays normalized and item-level UPSERT/idempotency semantics remain explicit. Event writes lock only the item that needs serialization.
- Logical backup digest verification is integrity evidence, not encryption, signature, provenance, PITR, HA, or RPO/RTO evidence.
- Web p95/load targets do not apply until a web/service surface exists; once one does, asynchronous handling and realistic k6 evidence become release gates.

## Required development order

1. Reacquire exact-head repository and central semantic/security evidence for #6, #7, and #8 while continuing independent work instead of waiting on the runner queue.
2. Progress #3/#4/#5 and then #8 to Ready when the connector mutation is healthy; never substitute admin bypass or self-approval.
3. Establish concrete Keyverse/service authentication and operated recovery/release evidence.
4. Add the next standards-backed `VTIMEZONE` capability and then CalDAV/provider interoperability fixtures, keeping recurrence/floating semantics explicitly versioned.
5. Migrate Naruon, `saju-caldav`, and LineageWeave only after released parity evidence exists.

## Evidence references

- CalendarWeave PRs #1, #3, #4, #5, #6, #7, #8 and issue #2.
- ADR-0001 through ADR-0007.
- `docs/doctoring/identity-authorization-admission-baseline.md`.
- `docs/doctoring/postgresql-logical-recovery-baseline.md`.
- `docs/doctoring/rfc5545-duration-baseline.md`.
- ContextualWisdomLab/.github #712 for current hosted-runner acquisition evidence.
- RFC 5545: Desruisseaux, B. (Ed.). (2009). *Internet calendaring and scheduling core object specification (iCalendar)*. RFC Editor. https://doi.org/10.17487/RFC5545
