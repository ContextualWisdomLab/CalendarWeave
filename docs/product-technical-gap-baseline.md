# CalendarWeave product and technical gap baseline

## Snapshot

Protected `main` remains a seed repository with no released CalendarWeave runtime or consumer contract. PRs #3/#4/#5 have merged into the open PR #1 branch, which now contains the Rust core, PostgreSQL adapter and bounded IANA `TZID` intervals. PR #6 adds a fail-closed external authorization admission candidate; PR #7 adds a test-first logical PostgreSQL backup/restore and recovery-invariant candidate. PRs #8/#9 remain stacked `DURATION` and `CLASS` candidates. None is shipped-product evidence.

The highest commercialization risks have moved from absence of a core model toward release/operability and interoperability: concrete service authentication/Keyverse integration, measured production recovery including WAL/PITR and RPO/RTO, broader RFC 5545/CalDAV capability, privacy/audit operation, versioned packaging/service evidence, and consumer parity/migration remain open. PR #7 narrows operated durability but does not turn logical restore into a disaster-recovery claim.

## Product responsibility and DDD boundary

| Responsibility | Owner / bounded context | Current evidence |
| --- | --- | --- |
| Calendar collections, resources, stable resource identity, revisions and ETags | CalendarWeave / Calendar Resource Core | Rust application port from #3 is integrated on PR #1; not released |
| Calendar operation admission | CalendarWeave / Authorization Admission | PR #6 candidate, tenant-free external identity + exact resource-aware authorization request + authorization-derived tenant; no token verifier/service authentication claim |
| Identity/federation and external authorization policy | Keyverse | External authority; CalendarWeave consumes verified issuer/subject identity and resource-aware policy decisions through an ACL |
| Relational durability and concurrency | CalendarWeave / Calendar Resource persistence adapter | 3NF PostgreSQL adapter from #4 is integrated on PR #1; PR #7 adds logical backup/restore evidence; no measured production RPO/RTO/PITR/HA claim |
| RFC 5545 time semantics | CalendarWeave / iCalendar semantics | UTC/all-day and bounded matching IANA `TZID` from #3/#5 are integrated on PR #1; `DURATION` is a candidate in #8, while `VTIMEZONE`, floating time and recurrence remain unavailable |
| CalDAV/provider interoperability and synchronization | CalendarWeave / interoperability adapters | Target responsibility; no released endpoint/provider-parity evidence |
| Workspace commitment/conflict/resolution policy | Naruon | Stays outside CalendarWeave behind a versioned CalendarPort/ACL |
| Calendar/evidence composition | LineageWeave | Read-model/evidence responsibility only; no CalendarWeave persistence ownership or mathematical computation |
| Saju calculation/scoring/explanation | `saju-caldav` | Separate domain; current generic CalDAV compatibility path migrates only after parity |
| Deterministic Four Pillars computation | `four-pillars` | Separate mathematical product responsibility |

Core subdomain: governed calendar-resource semantics and mutation/revision invariants. Supporting subdomains: authorization admission plus CalDAV/provider interoperability/synchronization evidence. Generic/external capabilities: identity/federation, database engine, telemetry and deployment platform. `CalendarCollection` owns collection-scoped event membership; event revision transitions preserve immutable UID plus conditional ETag invariants. `ExternalIdentity` is a tenant-free value object, not a calendar aggregate. `CalendarAuthorizationRequest` carries only the typed action and opaque resource references needed for authorization. Authorization decisions derive the tenant used by the core and do not widen item-level transaction boundaries. Backup/restore tooling is an operations boundary outside ordinary aggregate transactions and must recover, not redefine, the persisted invariants. External identity/provider DTOs and consumer decision policy remain behind ACLs.

## Exact-stack evidence and status

| Lane | Exact evidence at this update | Status / next verification |
| --- | --- | --- |
| PR #1 `docs/adr-baseline` | PRs #3/#4/#5 merged into this branch | Executable core, PostgreSQL and bounded `TZID` are integrated but not on protected `main`; exact-head required checks and review govern its merge |
| PRs #3/#4/#5 | Closed and merged into PR #1 branch | Their successful exact-head Rust and coverage checks supported branch integration; PR #1 needs its own checks before protected-main merge |
| PR #6 `feat/authorization-admission-v1` | Open candidate based on PR #1 | Tenant-free verified identity evidence and resource-aware authorization are not yet integrated into PR #1; exact-head hosted checks and semantic review remain required |
| PR #7 `feat/postgres-recovery-v1` | Open candidate stacked after #6 | RED recovery contract preceded production scripts; exact-head rust, coverage and recovery checks remain required |
| PRs #8/#9 | Open candidates stacked after #7 | `DURATION` and `CLASS` are not yet integrated into PR #1; exact-head hosted checks remain required |
| Central runner acquisition | ContextualWisdomLab/.github #712 / owner control plane | Fresh #6 and #7 jobs use explicit `ubuntu-24.04` yet remain queued with no steps, proving the current blocker is not the earlier floating-selector defect alone; exact canary IDs are recorded centrally rather than converted into leaf no-op churn |
| Central stacked review | ContextualWisdomLab/.github organization control plane | Existing bounded organization sweep and non-default-branch stacked OpenCode ruleset path are documented; verify current-head review receipts rather than duplicating the workflow |

The observed pre-fix CalendarWeave job state was queued with `runner_id=0`, empty runner name and zero executed steps. The exact heads for #3/#4/#5 completed on explicit Ubuntu 24.04 before integration, while current #6/#7 checks are still required. No predecessor-head check or review transfers to a later head.

## Commercialization gaps

| Gap | Owner | Evidence | Action | Exact-head status / next verification |
| --- | --- | --- | --- | --- |
| Authorization admission → real service authentication | CalendarWeave #2 + Keyverse integration boundary | PR #6 supplies tenant-free `ExternalIdentity`, exact `CalendarAuthorizationRequest`, deny/unavailable errors, authorization-derived `TenantId`, resource-scoped decision context and authorize-before-parse ordering | Add concrete infrastructure adapter that verifies the approved Keyverse issuer/token/session contract and derives admitted tenant from trusted identity/policy state without moving identity policy into CalendarWeave | Reacquire PR #6 exact-head tests/reviews; then prove invalid issuer/signature/algorithm/audience/subject/expiry/issued-at, tenant/resource mismatch and dependency failure semantics with real contract fixtures |
| Operated durability | CalendarWeave #2 / ADR-0003 / ADR-0006 | PR #7 adds custom-format logical backup, owner-only artifact/checksum, checksum-before-restore, one-transaction restore and a separate-database recovery drill for calendar data plus relational invariants | Execute the exact-head recovery drill; then add deployment-owned encrypted/retained remote backup storage, WAL/PITR where required, migration rollback, monitoring and measured recovery exercises | Logical recovery candidate only until exact-head hosted recovery is terminal successful; RPO/RTO/PITR/HA remain explicitly unclaimed |
| RFC 5545 capability parity | CalendarWeave #2 / ADR-0004 | Bounded IANA `TZID` support exists | Add standards-backed `DURATION`/`VTIMEZONE` slices test-first; keep unsupported recurrence/floating semantics fail-closed | RFC fixtures and edge cases pass at 100% owned statement/branch coverage |
| CalDAV/provider parity | CalendarWeave #2 | No CalDAV endpoint/provider adapter is shipped | Introduce protocol/application ACLs only after core contracts stabilize | Real interoperability fixtures plus reversible consumer migration proof |
| Privacy/content + authorization audit semantics | CalendarWeave #2 | Calendar text, principal evidence and logical backup artifacts may contain PII; no operated retention/access/audit evidence | Define purpose, retention, access/export/audit and non-masking controls where masking breaks calendar work; protect backup storage with least privilege, encryption and key management | CSAP/SOC 2 design controls mapped without certification claims; anonymized tests/docs; durable audit and backup access operation verified |
| Release/package/service contract | CalendarWeave #2 | No versioned release/package/container/service | Define versioned public port, compose deployability, SBOM/provenance and rollback | Immutable release artifact/service plus compatibility policy and real install/call path |
| Consumer migration | Naruon / `saju-caldav` / LineageWeave | Existing compatibility owners remain | Add explicit ACLs after CalendarWeave release; prove parity before deleting legacy paths | No direct table coupling; reversible migration and downstream acceptance evidence |
| Central stacked semantic review | ContextualWisdomLab/.github control plane | Central docs already describe bounded stacked OpenCode dispatch and ruleset `21732164` for non-default branches in evaluate mode | Verify #6/#7 receive current-head semantic review receipts; repair central workflow only if live evidence proves the existing path fails | Current-head OpenCode/Noema evidence plus ordinary governance before stack merge |
| Review lifecycle | CalendarWeave PR stack | PRs #3/#4/#5 were marked Ready through the CLI and merged into PR #1; PR #1 now contains executable candidate code, while #6/#7 remain Draft | Use ordinary Ready transitions only when the capability is executable and reviewable; bind every merge to current head and ruleset evidence | No connector error or predecessor-head result substitutes for current checks/review |

## Persistence and recovery invariants

- Executable relational objects use descriptive multiword `snake_case`: `calendar_collection`, `calendar_event`, `calendar_event_revision` and semantic columns/constraints/indexes.
- Current schema is 3NF for the implemented slice: collection identity, event identity/current revision reference and append-only revision bodies are separated.
- Item-level create idempotency is explicit by collection plus RFC UID; conditional updates use row locking and strong ETag/revision checks so competing writers cannot both advance one expected revision.
- PR #7 restores into a separate target and verifies the collection/event/current-revision values, `calendar_event_collection_uid_unique` and `calendar_event_current_revision_foreign_key`; backup/recovery does not create a parallel data model.
- Backup artifacts and checksum evidence are private by default (`0600`), restore rejects symlink evidence and a mismatched SHA-256 before `pg_restore`, and the bounded logical restore runs as one transaction. The digest is an integrity check, not encryption/signature/provenance.
- Cross-tenant and absent-resource observations remain indistinguishable at the core boundary; PR #6 checks authorization before parsing/mutation through the admitted application service.
- `ExternalIdentity` contains issuer plus subject only; subject syntax remains opaque aside from defensive length/control bounds backed by RFC 7519/OpenID Connect research traceability.
- The tenant used by `CalendarPort` is returned by `CalendarAuthorizationPort` for the exact action/resource request; a caller cannot self-assert tenant scope through the public admission API.
- No raw bearer token/provider credential becomes a Calendar Resource attribute or ordinary telemetry field.
- No production path consumes synthetic demo data; the recovery drill's `.example.test` fixture is test-only and anonymous. No mathematical/psychometric computation belongs in CalendarWeave or LineageWeave.

## Quality, operability and release gates

- Behavior changes start with RED executable contracts. PR #7 committed `tests/postgres_recovery_drill.sh` and wired the recovery job before production backup/restore scripts; PR #6 likewise preserves a test-only first commit before production admission code and later security regressions.
- Touched production code targets 100% owned statement/branch coverage plus complete rustdoc/docstring coverage; exact-head hosted checks are authoritative.
- No deprecation-warning suppression, self-approval, force-push, destructive rebase or governance-gate weakening.
- Exact-head checks, live reviews/threads, rulesets and concurrent writer state are re-read after every branch move; stale/queued/cancelled evidence is non-passing.
- The application boundary must remain fail-closed when authorization is denied or unavailable, must not depend on resource/parser behavior before authorization, and must not accept caller-selected tenant authority.
- Logical restore must remain fail-closed on missing/malformed/tampered evidence and prove post-restore relational invariants. A successful logical drill alone cannot satisfy production recovery/PITR/RPO/RTO gates.
- Release additionally requires security/SBOM/provenance, compose-compatible operability, rollback/recovery, real service authentication, realistic protocol/consumer evidence and PII/audit controls.

## Required development order

1. Reacquire exact-head repository and central semantic review evidence for PR #6 and PR #7; progress the architecture/core/persistence/time/admission/recovery stack only through ordinary governance and dependency order.
2. Establish concrete Keyverse/service authentication without duplicating the identity provider or authorization policy store; bind verified principal and policy state to the tenant returned by the admission port.
3. Turn the logical recovery candidate into measured deployment recovery: durable encrypted backup storage, migration rollback, WAL/PITR where required, monitoring and evidence-backed RPO/RTO; define the versioned service/package contract.
4. Add the next standards-backed calendar semantic slice (`DURATION`/`VTIMEZONE`) and CalDAV/provider interoperability fixtures.
5. Migrate Naruon, `saju-caldav` and LineageWeave through explicit ACLs only after release/parity evidence exists.

## Evidence references

- CalendarWeave PR #1 — Context Map and ownership ADR baseline.
- CalendarWeave #2 — executable Calendar Resource Core dependency root and commercialization tracker; remains open until release/consumer gates are proven.
- CalendarWeave ADR-0002 / ADR-0003 / ADR-0004 / ADR-0005 / ADR-0006 — core, PostgreSQL, bounded timezone, authorization-admission and logical-recovery decisions.
- `docs/doctoring/identity-authorization-admission-baseline.md` — RFC 7519/OpenID Connect research-to-source traceability for PR #6.
- `docs/doctoring/postgresql-logical-recovery-baseline.md` — PostgreSQL 18 logical backup/restore and PITR boundary traceability for PR #7.
- ContextualWisdomLab/.github #712 — current hosted-runner acquisition/capacity owner lane, including exact CalendarWeave #6/#7 queue canaries.
- ContextualWisdomLab/.github organization required-workflow rollout — stacked OpenCode control-plane path for non-default branches.
- Naruon #978 / #1508, `saju-caldav` #43 and LineageWeave #900 — downstream migration boundaries.
