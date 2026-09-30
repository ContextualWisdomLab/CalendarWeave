# ADR-0001: CalendarWeave context map and calendar ownership

- Status: Accepted architecture; implementation pending
- Date: 2026-09-01

## Context

Calendar functionality currently appears in several ContextualWisdomLab products. CalendarWeave is seeded as a standalone CalDAV/iCalendar PIMS, while Naruon already contains Google Calendar writeback, iCalendar task serialization, conflict policy and calendar UI; `saju-caldav` owns a product-specific recommendation engine together with a Radicale CalDAV runtime; LineageWeave exposes a calendar destination but intentionally does not own a calendar kernel.

Without an explicit Context Map, these implementations can evolve into multiple authoritative calendar graphs and duplicate generic provider/protocol code. That violates the intended DDD ownership and makes protocol correctness, synchronization, identity, privacy and migration harder to verify.

## Decision

CalendarWeave is the authoritative **generic calendar-resource bounded context** once its executable production contract is released. It owns calendar collections/resources, RFC 5545 iCalendar semantics, CalDAV interoperability, provider calendar adapters, revision/synchronization evidence and tenant-scoped calendar authorization/audit.

Consumer products retain the business semantics that explain why they interact with a calendar:

- **Naruon** owns workspace commitment semantics, scheduling/conflict policy, cross-context evidence, recommendation, user approval/correction and buyer-facing explanation. It consumes CalendarWeave through a versioned Calendar Port/Anti-Corruption Layer and does not own a second authoritative generic calendar graph.
- **LineageWeave** consumes calendar projections/deep links for lineage and ontology workflows. It does not own calendar persistence or scheduling protocol state.
- **saju-caldav** owns birth/profile inputs, saju-specific candidate calculation, candidate scoring/explanation and the intent to publish a selected candidate. Generic CalDAV collection/server/provider behavior migrates to CalendarWeave after production parity is proven.
- **four-pillars** remains a deterministic Four Pillars calculation/report product. CalendarWeave does not absorb its calculations; possible reuse between `four-pillars` and `saju-caldav` is a separate owner-boundary decision.
- **Keyverse** remains the identity provider. CalendarWeave validates scoped identity and never implements a local IdP.

A consumer may keep immutable calendar evidence snapshots required to explain a historical decision, but those snapshots are not authoritative mutable calendar state.

## Migration rule

This ADR does not authorize deleting existing working calendar code today because protected CalendarWeave `main` does not yet implement the target contract.

Migration is test-first and dependency-root first:

1. CalendarWeave publishes a versioned executable calendar contract and real interoperability fixtures.
2. A consumer adds characterization tests for its existing behavior before changing implementation.
3. A consumer introduces a narrow CalendarWeave adapter/ACL and proves parity for identity, recurrence/timezone, provider revision, authorization, privacy and failure semantics.
4. Only after released parity may duplicated generic calendar/provider/protocol code be removed from the consumer.
5. Architectural fitness tests prevent reintroduction of the generic responsibility.

No consumer may access CalendarWeave application tables directly.

## Consequences

### Positive

- one authoritative owner for reusable calendar protocol and resource semantics;
- Naruon can specialize on commitment decisions instead of provider/calendar plumbing;
- `saju-caldav` can keep its domain-specific recommendation semantics without permanently owning a second CalDAV platform;
- LineageWeave remains a consumer rather than a distributed calendar monolith;
- calendar providers can be replaced behind CalendarWeave adapters without changing consumer domain models.

### Costs and risks

- CalendarWeave must reach production parity before consumer deletion is safe;
- Naruon and `saju-caldav` require explicit compatibility migrations rather than direct source moves;
- existing data and provider identifiers need stable mapping/receipt contracts;
- failure to distinguish immutable evidence snapshots from mutable authoritative state could recreate duplication.

## Rejected alternatives

### Keep every product's own provider/calendar stack

Rejected because reusable RFC/provider behavior and synchronization truth would drift between products.

### Move Naruon commitment/conflict policy into CalendarWeave

Rejected because confirmed/tentative/desired commitment meaning, private evidence bridges and recommendation policy are Naruon workspace semantics rather than generic calendar-resource semantics.

### Move saju-specific scoring into CalendarWeave

Rejected because cultural/astrological candidate selection is product-specific domain logic and would contaminate the generic calendar bounded context.

### Delete consumer implementations immediately

Rejected because CalendarWeave protected `main` has no production implementation yet; architecture intent is not migration evidence.

## Verification

The implementation program must provide:

- real RFC 5545 VEVENT recurrence/timezone fixtures;
- CalDAV create/list/get/update/delete and query interoperability;
- ETag/sync-token/scheduling-tag stale-write tests where supported;
- tenant/purpose authorization and cross-tenant denial tests;
- Naruon contract tests proving commitment/conflict semantics remain local while calendar state is consumed through CalendarWeave;
- `saju-caldav` parity tests proving its event content/privacy/idempotency behavior survives CalendarWeave publication;
- architectural fitness tests forbidding direct consumer access to CalendarWeave persistence and forbidding duplicated generic provider/protocol ownership after migration;
- 100% owned production statement and branch coverage plus beginner-readable multi-line documentation for shipped code.

## References

Daboo, C., Desruisseaux, B., & Dusseault, L. M. (2007). *Calendaring extensions to WebDAV (CalDAV)* (RFC 4791). RFC Editor. https://doi.org/10.17487/RFC4791

Desruisseaux, B. (2009). *Internet calendaring and scheduling core object specification (iCalendar)* (RFC 5545). RFC Editor. https://doi.org/10.17487/RFC5545

Daboo, C. (2010). *iCalendar transport-independent interoperability protocol (iTIP)* (RFC 5546). RFC Editor. https://doi.org/10.17487/RFC5546

Daboo, C., & Quillaud, A. (2012). *Collection synchronization for WebDAV* (RFC 6578). RFC Editor. https://doi.org/10.17487/RFC6578

Daboo, C., & Desruisseaux, B. (2012). *Scheduling extensions to CalDAV* (RFC 6638). RFC Editor. https://doi.org/10.17487/RFC6638
