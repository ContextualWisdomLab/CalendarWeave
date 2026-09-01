# CalendarWeave product and technical gap baseline

## Snapshot

CalendarWeave protected `main` is currently a seed repository. The accepted target boundary is being documented in Draft PR #1; no production CalDAV/iCalendar runtime, persistence, provider adapter or released consumer contract is present on protected `main` yet.

The primary product risk is therefore not missing UI polish. It is **authority ambiguity across existing CWL calendar implementations** and the absence of an executable CalendarWeave contract that lets consumers migrate safely.

## Authority baseline

| Responsibility | Owner | Current state |
| --- | --- | --- |
| Generic calendar collections/resources and revisions | CalendarWeave | Accepted target architecture; implementation missing |
| RFC 5545 iCalendar / RFC 4791 CalDAV semantics | CalendarWeave | Accepted target architecture; implementation missing |
| Provider calendar adapters and sync/revision evidence | CalendarWeave | Planned; no released contract |
| Workspace commitment/conflict/resolution policy | Naruon | Existing product responsibility; must remain outside CalendarWeave |
| Calendar/evidence composition in lineage workflows | LineageWeave | Existing read-model responsibility; compatibility projection currently routes through Naruon |
| Saju candidate rules/scoring/explanation | saju-caldav | Existing product responsibility |
| Current generic Radicale/CalDAV runtime for saju-caldav | saju-caldav compatibility path | Target migration to CalendarWeave only after parity |
| Deterministic Four Pillars calculation/reporting | four-pillars | Separate product responsibility |
| Identity/federation | Keyverse | External identity owner |

## Open gaps and owner paths

| Gap | Owner path | Status / acceptance |
| --- | --- | --- |
| CalendarWeave has architecture but no executable calendar core | CalendarWeave #2 | Build real collection + VEVENT create/list/get with versioned port, RFC fixtures, tenant/security tests and 100% statement/branch coverage |
| Naruon currently owns Google Calendar SDK/iCalendar mechanics in generic service paths | Naruon #978 and #1508 | Preserve Naruon commitment policy; introduce CalendarPort/ACL after CalendarWeave release; migrate generic provider/protocol responsibility only after parity |
| `saju-caldav` currently owns Radicale/CalDAV platform responsibility | saju-caldav #43 | Characterize current behavior, add CalendarWeave publisher port, prove parity, then remove generic CalDAV platform ownership |
| LineageWeave merged compatibility projection names Naruon as calendar authority | LineageWeave #900 | Keep compatibility v1 until CalendarWeave read contract exists; later consume CalendarWeave observations and Naruon decision evidence as separate typed sources |
| Protected-main documentation could overstate target model as shipped persistence | CalendarWeave PR #1 | Docs must label logical model and accepted architecture separately from implemented protected-main state |
| No executable architectural fitness preventing foreign domain imports | CalendarWeave #2 | Fail if CalendarWeave imports Naruon commitment policy, saju scoring, LineageWeave ontology or Four Pillars computation; fail if consumers read CalendarWeave tables directly |
| No released package/API/event compatibility policy | CalendarWeave #2 | Versioned consumer contract required before migration |

## Required development order

1. Finish PR #1 as an accurate architecture/ADR baseline without claiming implementation.
2. Implement CalendarWeave #2 test-first as the dependency root.
3. Publish immutable/versioned consumer fixtures and release evidence.
4. Naruon adds CalendarPort/ACL and proves parity before removing Google/iCalendar generic plumbing.
5. `saju-caldav` adds CalendarPublisherPort/ACL and proves Radicale-path parity before removing generic CalDAV runtime responsibility.
6. LineageWeave adds the CalendarWeave observation contract and keeps Naruon scheduling-decision evidence separate.
7. Add organization architectural-fitness checks preventing a second generic calendar source of truth.

## Quality gates

- SOLID responsibility boundaries follow the Context Map; no provider SDK dependency in the calendar domain core.
- DDD paths reflect calendar-resource responsibility, with provider integrations behind ports/adapters.
- Every production behavior change starts from a failing executable test or contract fixture.
- Owned production statement and branch coverage: 100%.
- Missing, one-line or vacuous shipped docstrings/rustdoc: 0.
- Real recurrence/timezone/malformed-input fixtures and stale-write/concurrency/tenant-isolation tests.
- No consumer direct access to CalendarWeave application tables.
- 3NF and descriptive multiword snake_case objects for relational persistence.
- Security, SBOM/provenance, package/container, rollback/recovery and exact-head review/check evidence before release.

## Evidence references

- CalendarWeave PR #1 — Context Map and ADR baseline.
- CalendarWeave #2 — executable Calendar Resource Core dependency root.
- Naruon #978 — scheduling/commitment buyer outcome with CalendarWeave ownership correction.
- Naruon #1508 — executable bounded-context/path migration, including CalendarWeave ACL.
- saju-caldav #43 — Radicale/CalDAV migration after parity.
- LineageWeave #900 — migration from temporary Naruon calendar authority to CalendarWeave resource authority.
