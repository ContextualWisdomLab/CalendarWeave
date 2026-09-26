# CalendarWeave product and technical gap baseline

## Snapshot

Protected `main` remains a seed repository with no released CalendarWeave runtime or consumer contract. PRs #3/#4/#5 have merged into the open PR #1 branch, which now contains the Rust core, PostgreSQL adapter and bounded IANA `TZID` intervals. PR #6 adds a fail-closed external authorization admission candidate, while PRs #7/#8/#9 remain stacked recovery, `DURATION` and `CLASS` candidates. None is shipped-product evidence.

The highest commercialization risks have moved from absence of a core model toward release/operability and interoperability: concrete service authentication/Keyverse integration, operated backup/recovery, broader RFC 5545/CalDAV capability, privacy/audit operation, versioned packaging/service evidence, and consumer parity/migration remain open.

## Product responsibility and DDD boundary

| Responsibility | Owner / bounded context | Current evidence |
| --- | --- | --- |
| Calendar collections, resources, stable resource identity, revisions and ETags | CalendarWeave / Calendar Resource Core | Rust application port from #3 is integrated on PR #1; not released |
| Calendar operation admission | CalendarWeave / Authorization Admission | PR #6 candidate, tenant-free external identity + exact resource-aware authorization request + authorization-derived tenant; no token verifier/service authentication claim |
| Identity/federation and external authorization policy | Keyverse | External authority; CalendarWeave consumes verified issuer/subject identity and resource-aware policy decisions through an ACL |
| Relational durability and concurrency | CalendarWeave / Calendar Resource persistence adapter | 3NF PostgreSQL adapter from #4 is integrated on PR #1; operated recovery is a separate candidate in #7 |
| RFC 5545 time semantics | CalendarWeave / iCalendar semantics | UTC/all-day and bounded matching IANA `TZID` from #3/#5 are integrated on PR #1; `DURATION` is a candidate in #8, while `VTIMEZONE`, floating time and recurrence remain unavailable |
| CalDAV/provider interoperability and synchronization | CalendarWeave / interoperability adapters | Target responsibility; no released endpoint/provider-parity evidence |
| Workspace commitment/conflict/resolution policy | Naruon | Stays outside CalendarWeave behind a versioned CalendarPort/ACL |
| Calendar/evidence composition | LineageWeave | Read-model/evidence responsibility only; no CalendarWeave persistence ownership or mathematical computation |
| Saju calculation/scoring/explanation | `saju-caldav` | Separate domain; current generic CalDAV compatibility path migrates only after parity |
| Deterministic Four Pillars computation | `four-pillars` | Separate mathematical product responsibility |

Core subdomain: governed calendar-resource semantics and mutation/revision invariants. Supporting subdomains: authorization admission plus CalDAV/provider interoperability/synchronization evidence. Generic/external capabilities: identity/federation, database engine, telemetry and deployment platform. `CalendarCollection` owns collection-scoped event membership; event revision transitions preserve immutable UID plus conditional ETag invariants. `ExternalIdentity` is a tenant-free value object, not a calendar aggregate. `CalendarAuthorizationRequest` carries only the typed action and opaque resource references needed for authorization. Authorization decisions derive the tenant used by the core and do not widen item-level transaction boundaries. External identity/provider DTOs and consumer decision policy remain behind ACLs.

## Exact-stack evidence and status

| Lane | Exact evidence at this update | Status / next verification |
| --- | --- | --- |
| PR #1 `docs/adr-baseline` | PRs #3/#4/#5 merged into this branch | Executable core, PostgreSQL and bounded `TZID` are integrated but not on protected `main`; exact-head required checks and review govern its merge |
| PRs #3/#4/#5 | Closed and merged into PR #1 branch | Their successful exact-head Rust and coverage checks supported branch integration; PR #1 needs its own checks before protected-main merge |
| PR #6 `feat/authorization-admission-v1` | Open candidate based on PR #1 | Tenant-free verified identity evidence and resource-aware authorization are not yet integrated into PR #1; exact-head hosted checks and semantic review remain required |
| PRs #7/#8/#9 | Open candidate stack after #6 | Logical recovery, `DURATION` and `CLASS` are not yet integrated into PR #1; new-head hosted checks remain required |
| Central runner cause | ContextualWisdomLab/.github #1618 merged | Organization evidence proved floating `ubuntu-latest` starvation while explicit Ubuntu 24.04 executed; CalendarWeave has a permanent local selector regression |
| Central stacked review | ContextualWisdomLab/.github organization control plane | Existing bounded organization sweep and non-default-branch stacked OpenCode ruleset path are documented; verify current-head review receipts rather than duplicating the workflow |

The observed pre-fix CalendarWeave job state was queued with `runner_id=0`, empty runner name and zero executed steps. The repaired exact heads for #3/#4/#5 completed on explicit Ubuntu 24.04. No predecessor-head check or review transfers to a later head.

## Commercialization gaps

| Gap | Owner | Evidence | Action | Exact-head status / next verification |
| --- | --- | --- | --- | --- |
| Authorization admission → real service authentication | CalendarWeave #2 + Keyverse integration boundary | PR #6 supplies tenant-free `ExternalIdentity`, exact `CalendarAuthorizationRequest`, deny/unavailable errors, authorization-derived `TenantId`, resource-scoped decision context and authorize-before-parse ordering | Add concrete infrastructure adapter that verifies the approved Keyverse issuer/token/session contract and derives admitted tenant from trusted identity/policy state without moving identity policy into CalendarWeave | Reacquire PR #6 exact-head tests/reviews; then prove token/service admission and failure semantics with real contract fixtures |
| Operated durability | CalendarWeave #2 / ADR-0003 | PostgreSQL restart and concurrency candidate exists; PR #7 adds a logical restore drill | Validate PR #7, then add backup-store operations, migration rollback and failure-recovery evidence | Recovery drill preserves collection/event/revision invariants with documented RPO/RTO assumptions |
| RFC 5545 capability parity | CalendarWeave #2 / ADR-0004 | Bounded IANA `TZID` support exists; PRs #8/#9 carry `DURATION` and `CLASS` candidates | Validate those slices, add standards-backed `VTIMEZONE` test-first, and keep unsupported recurrence/floating semantics fail-closed | RFC fixtures and edge cases pass at 100% owned statement/branch coverage |
| CalDAV/provider parity | CalendarWeave #2 | No CalDAV endpoint/provider adapter is shipped | Introduce protocol/application ACLs only after core contracts stabilize | Real interoperability fixtures plus reversible consumer migration proof |
| Privacy/content + authorization audit semantics | CalendarWeave #2 | Calendar text and principal evidence may contain PII; no operated retention/access/audit evidence | Define purpose, retention, access/export/audit and non-masking controls where masking breaks calendar work | CSAP/SOC 2 design controls mapped without certification claims; anonymized tests/docs; durable audit operation verified |
| Release/package/service contract | CalendarWeave #2 | No versioned release/package/container/service | Define versioned public port, compose deployability, SBOM/provenance and rollback | Immutable release artifact/service plus compatibility policy and real install/call path |
| Consumer migration | Naruon / `saju-caldav` / LineageWeave | Existing compatibility owners remain | Add explicit ACLs after CalendarWeave release; prove parity before deleting legacy paths | No direct table coupling; reversible migration and downstream acceptance evidence |
| Central stacked semantic review | ContextualWisdomLab/.github control plane | Central docs already describe bounded stacked OpenCode dispatch and ruleset `21732164` for non-default branches in evaluate mode | Verify #6 receives current-head semantic review/receipt; repair central workflow only if live evidence proves the existing path fails | Current-head OpenCode/Noema evidence plus ordinary governance before stack merge |
| Review lifecycle | CalendarWeave PR stack | PRs #3/#4/#5 were marked Ready through the CLI and merged into PR #1; PR #1 now contains executable candidate code | Keep PR #1's review and checks bound to its current head before protected-main merge | No connector error or predecessor-head result substitutes for current ruleset evidence |

## Persistence and invariants

- Executable relational objects use descriptive multiword `snake_case`: `calendar_collection`, `calendar_event`, `calendar_event_revision` and semantic columns/constraints/indexes.
- Current schema is 3NF for the implemented slice: collection identity, event identity/current revision reference and append-only revision bodies are separated.
- Item-level create idempotency is explicit by collection plus RFC UID; conditional updates use row locking and strong ETag/revision checks so competing writers cannot both advance one expected revision.
- Cross-tenant and absent-resource observations remain indistinguishable at the core boundary; PR #6 checks authorization before parsing/mutation through the admitted application service.
- `ExternalIdentity` contains issuer plus subject only; subject syntax remains opaque aside from defensive length/control bounds backed by RFC 7519/OpenID Connect research traceability.
- The tenant used by `CalendarPort` is returned by `CalendarAuthorizationPort` for the exact action/resource request; a caller cannot self-assert tenant scope through the public admission API.
- No raw bearer token/provider credential becomes a Calendar Resource attribute or ordinary telemetry field.
- No production path consumes synthetic demo data and no mathematical/psychometric computation belongs in CalendarWeave or LineageWeave.

## Quality, operability and release gates

- Behavior changes start with RED executable contracts; PR #6 preserves a test-only first commit before production admission code and subsequent security regressions cover tenant/resource authorization scope.
- Touched production code targets 100% owned statement/branch coverage plus complete rustdoc/docstring coverage; exact-head hosted checks are authoritative.
- No deprecation-warning suppression, self-approval, force-push, destructive rebase or governance-gate weakening.
- Exact-head checks, live reviews/threads, rulesets and concurrent writer state are re-read after every branch move; stale/queued/cancelled evidence is non-passing.
- The application boundary must remain fail-closed when authorization is denied or unavailable, must not depend on resource/parser behavior before authorization, and must not accept caller-selected tenant authority.
- Release additionally requires security/SBOM/provenance, compose-compatible operability, rollback/recovery, real service authentication, realistic protocol/consumer evidence and PII/audit controls.

## Required development order

1. Reacquire exact-head repository and central semantic review evidence for PR #6; progress the architecture/core/persistence/time/admission stack only through ordinary governance and dependency order.
2. Establish concrete Keyverse/service authentication without duplicating the identity provider or authorization policy store; bind verified principal and policy state to the tenant returned by the admission port.
3. Establish operated PostgreSQL recovery and versioned service/package evidence.
4. Add the next standards-backed calendar semantic slice (`DURATION`/`VTIMEZONE`) and CalDAV/provider interoperability fixtures.
5. Migrate Naruon, `saju-caldav` and LineageWeave through explicit ACLs only after release/parity evidence exists.

## Evidence references

- CalendarWeave PR #1 — Context Map and ownership ADR baseline.
- CalendarWeave #2 — executable Calendar Resource Core dependency root and commercialization tracker; remains open until release/consumer gates are proven.
- CalendarWeave ADR-0002 / ADR-0003 / ADR-0004 / ADR-0005 — core, PostgreSQL, bounded timezone and authorization-admission decisions.
- `docs/doctoring/identity-authorization-admission-baseline.md` — RFC 7519/OpenID Connect research-to-source traceability for PR #6.
- ContextualWisdomLab/.github PR #1618 — hosted-runner selector root-cause repair and A/B evidence.
- ContextualWisdomLab/.github organization required-workflow rollout — stacked OpenCode control-plane path for non-default branches.
- Naruon #978 / #1508, `saju-caldav` #43 and LineageWeave #900 — downstream migration boundaries.
