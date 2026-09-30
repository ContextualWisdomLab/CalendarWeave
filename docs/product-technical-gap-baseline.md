# CalendarWeave product and technical gap baseline

## Snapshot

Protected `main` remains a seed repository with no released CalendarWeave runtime or consumer contract. PRs #3, #4 and #5 have merged into the open PR #1 branch, which now contains the Rust Calendar Resource Core, PostgreSQL persistence and bounded IANA `TZID` intervals. PRs #6–#9 remain stacked candidates for authorization admission, recovery, `DURATION` and `CLASS`. None of this branch content is shipped-product evidence.

The highest commercialization risks are now release/operability and interoperability gaps, not absence of a core model: external authorization admission, operated backup/recovery, broader RFC 5545/CalDAV capability, privacy/content semantics, versioned packaging/service evidence, and consumer parity/migration remain open.

## Product responsibility and DDD boundary

| Responsibility | Owner / bounded context | Current evidence |
| --- | --- | --- |
| Calendar collections, resources, stable resource identity, revisions and ETags | CalendarWeave / Calendar Resource | Rust application port from #3 is integrated on PR #1; not released |
| Relational durability and concurrency | CalendarWeave / Calendar Resource persistence adapter | 3NF PostgreSQL adapter from #4 is integrated on PR #1; operated recovery is a separate candidate in #7 |
| RFC 5545 time semantics | CalendarWeave / iCalendar semantics | UTC/all-day and bounded matching IANA `TZID` intervals from #3/#5 are integrated on PR #1; `DURATION` is a candidate in #8, while `VTIMEZONE`, floating time and recurrence remain unavailable |
| CalDAV/provider interoperability and synchronization | CalendarWeave / interoperability adapters | Target responsibility; no released endpoint/provider-parity evidence |
| Workspace commitment/conflict/resolution policy | Naruon | Stays outside CalendarWeave behind a versioned CalendarPort/ACL |
| Calendar/evidence composition | LineageWeave | Read-model/evidence responsibility only; no CalendarWeave persistence ownership |
| Saju calculation/scoring/explanation | `saju-caldav` | Separate domain; current generic CalDAV compatibility path migrates only after parity |
| Deterministic Four Pillars computation | `four-pillars` | Separate mathematical product responsibility |
| Identity/federation | Keyverse | External identity authority; CalendarWeave consumes scoped identity |

Core subdomain: governed calendar-resource semantics and mutation/revision invariants. Supporting subdomains: CalDAV/provider interoperability and synchronization evidence. Generic/external capabilities: identity/federation, database engine, telemetry and deployment platform. The Calendar Resource aggregate owns collection-scoped event identity and revision transitions; external provider DTOs and consumer-specific decision policy remain behind ACLs. Transaction boundaries stay item-level and concurrency-safe.

## Exact-stack evidence and status

| Lane | Exact evidence at this update | Status / next verification |
| --- | --- | --- |
| PR #1 `docs/adr-baseline` | PRs #3/#4/#5 merged into this branch; current candidate covers revision exhaustion and malformed named-timezone end values | Executable core, PostgreSQL and bounded `TZID` are integrated but not on protected `main`; replacement exact-head required checks and review govern its merge |
| PRs #3/#4/#5 | Closed and merged into PR #1 branch | Their successful exact-head Rust and coverage checks supported branch integration; PR #1 needs its own checks before protected-main merge |
| PRs #6/#7/#8/#9 | Open candidate stack, with #6 based on PR #1 | Authorization admission, recovery, `DURATION` and `CLASS` are not yet integrated into PR #1; new-head hosted checks are pending |
| Central runner cause | ContextualWisdomLab/.github #1618 merged | Organization evidence proved floating `ubuntu-latest` jobs could remain `runner_id=0` while explicit Ubuntu 24.04 executed; CalendarWeave now has a permanent local selector regression |

The observed pre-fix CalendarWeave job state was queued with `runner_id=0`, empty runner name and zero executed steps. The repaired #3 exact predecessor acquired GitHub-hosted Ubuntu 24.04 runners and completed Rust, tests, rustdoc and 100% line/branch coverage gates successfully. No predecessor-head check or review transfers to a later head.

## Commercialization gaps

| Gap | Owner | Evidence | Action | Acceptance / next verification |
| --- | --- | --- | --- | --- |
| External authorization admission | CalendarWeave #2 | Core receives scoped tenant identity but is not an authenticated service; PR #6 is an external-admission candidate | Validate PR #6 against hosted gates, then add a real trusted identity/policy adapter and audit evidence | Unauthorized/cross-tenant behavior proven through executable service boundary without local IdP duplication |
| Operated durability | CalendarWeave #2 / ADR-0003 | PostgreSQL restart and concurrency candidate exists; PR #7 adds a logical restore drill | Validate PR #7, then add backup-store operations, migration rollback and failure-recovery evidence | Recovery drill preserves collection/event/revision invariants and documented RPO/RTO assumptions |
| RFC 5545 capability parity | CalendarWeave #2 / ADR-0004 | Bounded IANA `TZID` support exists; PRs #8/#9 carry `DURATION` and `CLASS` candidates | Validate those slices, add standards-backed `VTIMEZONE` test-first, and keep unsupported recurrence/floating semantics fail-closed | RFC fixtures and edge cases pass at 100% owned statement/branch coverage |
| CalDAV/provider parity | CalendarWeave #2 | No CalDAV endpoint or provider adapter is shipped | Introduce protocol/application ACLs only after core contracts stabilize | Real interoperability fixtures plus reversible consumer migration proof |
| Privacy/content semantics | CalendarWeave #2 | Calendar text may contain PII; no operated policy evidence | Document purpose, retention, access/audit and non-masking boundary where masking breaks calendar work | CSAP/SOC 2 design controls mapped without claiming certification; tests/docs anonymize real persons/institutions |
| Release/package/service contract | CalendarWeave #2 | No versioned release/package/container/service | Define versioned public port, compose deployability, SBOM/provenance and rollback | Immutable release artifact/service plus compatibility policy and install/call path |
| Consumer migration | Naruon / `saju-caldav` / LineageWeave | Existing compatibility owners remain | Add explicit ACLs after CalendarWeave release; prove parity before deleting legacy paths | No direct table coupling; reversible migration and downstream acceptance evidence |
| Central stacked semantic review | ContextualWisdomLab/.github control plane | Organization required workflows protect default branches; central scheduler documents bounded stacked OpenCode dispatch | Keep exact-head scheduler review path active; do not weaken default-branch rulesets | Current-head OpenCode/Noema evidence plus ordinary governance before merge |

## Persistence and invariants

- Relational objects use descriptive multiword `snake_case`: `calendar_collection`, `calendar_event`, `calendar_event_revision` and corresponding semantic columns/constraints/indexes.
- Schema is 3NF for the current slice: collection identity, event identity/current revision reference, and append-only revision bodies are separated.
- Item-level create idempotency is explicit by collection plus RFC UID; conditional updates use row locking and strong ETag/revision checks so competing writers cannot both advance one expected revision.
- Cross-tenant and absent-resource observations remain indistinguishable at the application boundary; authorization is checked before payload parsing where required.
- No production path consumes synthetic demo data and no mathematical/psychometric computation belongs in CalendarWeave.

## Quality and release gates

- Behavior changes start with RED executable contracts; the runner-selector incident has a permanent regression test.
- The revision counter must fail closed at `u64::MAX` without replacing the current event; named-timezone parsing must reject malformed local end values before interval comparison.
- Touched production code targets 100% owned statement and branch coverage plus complete rustdoc/docstring coverage.
- No deprecation-warning suppression or governance-gate weakening.
- Exact-head checks, live reviews/threads, rulesets and concurrent writer state are re-read after every branch move; stale evidence is non-passing.
- Release additionally requires security/SBOM/provenance, compose-compatible operability, rollback/recovery, external authorization and realistic protocol/consumer evidence.

## Required development order

1. Land the architecture/core/persistence/time-semantics stack through ordinary exact-head checks and independent review; do not close #2 from candidate branches alone.
2. Add the next standards-backed calendar semantic slice (`DURATION`/`VTIMEZONE`) or external authorization admission, whichever yields the smallest independently verifiable vertical.
3. Establish operated PostgreSQL recovery and versioned service/package evidence.
4. Add CalDAV/provider interoperability and consumer parity fixtures.
5. Migrate Naruon, `saju-caldav` and LineageWeave through explicit ACLs only after release/parity evidence exists.

## Evidence references

- CalendarWeave PR #1 — Context Map and ownership ADR baseline.
- CalendarWeave #2 — executable Calendar Resource Core dependency root and commercialization tracker.
- CalendarWeave ADR-0002 / ADR-0003 / ADR-0004 — core, PostgreSQL and bounded timezone decisions.
- ContextualWisdomLab/.github PR #1618 — hosted-runner selector root-cause repair and A/B evidence.
- Naruon #978 / #1508, `saju-caldav` #43 and LineageWeave #900 — downstream migration boundaries.
