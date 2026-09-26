# CalendarWeave product and technical gap baseline

## Snapshot

Protected `main` remains a seed repository with no released CalendarWeave runtime or consumer contract. The active same-repository stack is executable candidate evidence rather than shipped-product evidence: PR #1 defines the ownership/context baseline; PR #3 implements the Rust Calendar Resource Core; PR #4 adds durable PostgreSQL persistence; PR #5 adds bounded IANA `TZID` interval validation.

The highest commercialization risks are now release/operability and interoperability gaps, not absence of a core model: external authorization admission, operated backup/recovery, broader RFC 5545/CalDAV capability, privacy/content semantics, versioned packaging/service evidence, and consumer parity/migration remain open.

## Product responsibility and DDD boundary

| Responsibility | Owner / bounded context | Current evidence |
| --- | --- | --- |
| Calendar collections, resources, stable resource identity, revisions and ETags | CalendarWeave / Calendar Resource | PR #3 candidate, Rust application port; not released |
| Relational durability and concurrency | CalendarWeave / Calendar Resource persistence adapter | PR #4 candidate, 3NF PostgreSQL with append-only revisions and row-locked conditional writes; not operated recovery evidence |
| RFC 5545 time semantics | CalendarWeave / iCalendar semantics | UTC/all-day in #3; bounded matching IANA `TZID` intervals in #5; `VTIMEZONE`, `DURATION`, floating time and recurrence still unavailable |
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
| PR #1 `docs/adr-baseline` | `d9393fd7e4e6e3ad72d0f09acdf656d6569555f6` | Architecture/ADR parent; Draft, no released runtime claim |
| PR #3 `feat/calendar-resource-core-v1` | runner repair predecessor `d415d835068891dea028210d9ff3f602f60b15dd` | Repository `Tests` completed success after pinning both jobs to `ubuntu-24.04`; this documentation commit must reacquire exact-head evidence |
| PR #4 `feat/postgres-calendar-store-v1` | `da35a30fe9a9f5902d4e7fd795fd01a0ce560378` | Current parent propagation includes PostgreSQL services plus explicit Ubuntu 24.04 runners; exact-head Tests are required |
| PR #5 `feat/tzid-calendar-interval-v1` | `0aadc06f15c550c9b117d3a30bf156326cb63dcb` | Current parent propagation includes bounded `TZID`; exact-head Tests are required |
| Central runner cause | ContextualWisdomLab/.github #1618 merged | Organization evidence proved floating `ubuntu-latest` jobs could remain `runner_id=0` while explicit Ubuntu 24.04 executed; CalendarWeave now has a permanent local selector regression |

The observed pre-fix CalendarWeave job state was queued with `runner_id=0`, empty runner name and zero executed steps. The repaired #3 exact predecessor acquired GitHub-hosted Ubuntu 24.04 runners and completed Rust, tests, rustdoc and 100% line/branch coverage gates successfully. No predecessor-head check or review transfers to a later head.

## Commercialization gaps

| Gap | Owner | Evidence | Action | Acceptance / next verification |
| --- | --- | --- | --- | --- |
| External authorization admission | CalendarWeave #2 | Core receives scoped tenant identity but is not an authenticated service | Define admission port/ACL consuming Keyverse-scoped identity; add tenant/authorization edge cases | Unauthorized/cross-tenant behavior proven through executable service boundary without local IdP duplication |
| Operated durability | CalendarWeave #2 / ADR-0003 | PostgreSQL restart and concurrency candidate exists | Add backup/restore, migration rollback and failure-recovery evidence | Recovery drill preserves collection/event/revision invariants and documented RPO/RTO assumptions |
| RFC 5545 capability parity | CalendarWeave #2 / ADR-0004 | Bounded IANA `TZID` support exists | Add standards-backed `DURATION`/`VTIMEZONE` slices test-first; keep unsupported recurrence/floating semantics fail-closed | RFC fixtures and edge cases pass at 100% owned statement/branch coverage |
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
