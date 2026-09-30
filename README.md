# CalendarWeave

[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/ContextualWisdomLab/CalendarWeave)

**Governed calendar resources, iCalendar semantics, and interoperable scheduling infrastructure.**

CalendarWeave is the ContextualWisdomLab calendar-resource product: the reusable home for versioned calendar collections and events, iCalendar semantics, revisions and synchronization evidence, timezone and recurrence behavior, CalDAV interoperability, and provider-facing calendar adapters.

It exists so products can consume one explicit calendar contract instead of each embedding its own calendar store, provider SDK behavior, or protocol interpretation.

## Why CalendarWeave

Calendar systems become difficult to integrate when resource identity, revisions, time semantics, provider behavior, and business decisions are mixed together. CalendarWeave keeps the reusable calendar-resource layer narrow and governed.

| Need | CalendarWeave responsibility |
| --- | --- |
| Stable calendar data | Calendar collections, resources, event identity and revision evidence |
| Interoperability | RFC 5545 iCalendar and RFC 4791 CalDAV semantics as versioned capabilities |
| Correct time behavior | Explicit timezone, interval and later recurrence semantics that fail closed when unsupported or ambiguous |
| Provider integration | Calendar-provider adapters behind product-owned ports rather than provider DTOs in the domain model |
| Safe composition | Versioned contracts for consumers instead of direct database coupling |
| Auditability | Synchronization, mutation and authorization evidence at the calendar boundary |

## Current status

CalendarWeave is under active foundation development. The protected default branch is still a seed repository; this implementation stack defines and proves candidate product behavior, not a released runtime.

The implementation stack is building a tenant-scoped Calendar Resource Core, durable persistence, strict RFC 5545 time semantics, authorization admission, and recoverability evidence before any production service, CalDAV endpoint, provider parity, or consumer-migration claim is made. Candidate behavior in open pull requests is not protected-main or release evidence.

### What you can rely on today

- the calendar-resource ownership boundary and Context Map are documented;
- target interoperability and migration constraints are explicit;
- unsupported or not-yet-released capabilities are identified rather than implied;
- no production endpoint, package, container image, or deployment is advertised from this branch.

### Executable candidate evidence

The implementation stack contains a Rust v0.1 application port for tenant-scoped collection creation and strict RFC 5545 VEVENT create, conditional-update, list, and get behavior. It accepts UTC, all-day, and bounded matching-IANA-`TZID` intervals; the newest candidate also accepts positive RFC 5545 `DURATION` values as the explicit alternative to `DTEND`, including DATE-start day/week restrictions. The core preserves standard event status, returns opaque event references with revision/ETag evidence, and fails closed for malformed, cross-tenant, stale-revision, ambiguous/nonexistent local-time, and unsupported requests.

The durable-store slice adds a 3NF PostgreSQL adapter with restart-stable event identity, append-only revisions, item-level create idempotency by collection plus RFC UID, and row-locked strong-ETag updates. A later candidate adds a digest-verified logical backup/restore drill, but that is not an operated disaster-recovery, PITR, HA, or RPO/RTO claim.

Authorization admission uses externally verified issuer/subject evidence without allowing the caller to choose a tenant. The trusted authorization decision derives tenant scope for the exact calendar action/resource request before parsing or mutation. This is candidate application-boundary evidence, not a released authentication service or Keyverse token-verification adapter.

These executable candidates are not protected-main, a released package or service, a CalDAV endpoint, provider parity, `VTIMEZONE`/recurrence support, or a consumer migration contract.

## First releasable vertical

The first release target is intentionally small and testable:

1. create a tenant-scoped calendar collection;
2. create and retrieve an RFC 5545 VEVENT with stable identity and revision evidence;
3. preserve supported timezone and explicit `DTEND`/`DURATION` interval semantics fail-closed;
4. persist the resource through a durable adapter; and
5. expose a versioned application contract that other products can consume without database coupling.

CalDAV service exposure, recurrence, scheduling, provider adapters, authentication, operated backup/recovery, and broader synchronization capabilities become customer-facing only when their executable contracts and release evidence exist.

## Start here

There is no supported installation command yet because the current protected product is not released. To evaluate or integrate CalendarWeave without inventing a runtime contract, use the documentation in this order:

1. Read [`ARCHITECTURE.md`](ARCHITECTURE.md) for the product boundary and integration ownership map.
2. Read the [documentation home](docs/index.md) for the current architecture, research, and gap evidence.
3. Review [`docs/adr/0001-context-map-and-calendar-ownership.md`](docs/adr/0001-context-map-and-calendar-ownership.md) before designing a consumer adapter.
4. Treat open implementation pull requests as candidate evidence until their behavior reaches the protected default branch and a versioned release is published.

A copy-paste install or API quickstart will be added only when there is a released executable contract to install and call.

## Integration context

CalendarWeave owns generic calendar resources. Adjacent products retain their own domain authority and integrate through explicit ports or Anti-Corruption Layers.

| Product | Boundary with CalendarWeave |
| --- | --- |
| Naruon | Owns workspace commitment meaning, conflict/resolution policy, approval and customer workflow; consumes calendar resources through a versioned port/ACL |
| LineageWeave | Owns read-only lineage/evidence composition and deep links; does not own calendar persistence or provider credentials |
| `saju-caldav` | Owns saju-specific candidate calculation, scoring, explanation and publication intent; its current generic CalDAV path remains compatibility infrastructure until parity is proven |
| `four-pillars` | Owns deterministic Four Pillars calculation/reporting; that computation stays outside CalendarWeave |
| Keyverse | Owns identity/federation and external authorization policy; CalendarWeave consumes verified principal evidence and a resource-aware authorization decision rather than implementing a local identity provider |

Consumers must not read CalendarWeave application tables directly or import provider DTOs as their domain model. The migration path is contract-first, parity-tested, and reversible until the replacement is proven.

## Architecture at a glance

```text
Calendar clients / product consumers
               |
        versioned Calendar Port
               |
               v
+--------------------------------------+
|             CalendarWeave            |
|                                      |
|  calendar collections & resources    |
|  iCalendar / CalDAV semantics        |
|  revision & synchronization evidence |
|  authorization / audit boundary      |
+-------------------+------------------+
                    |
            ports / adapters
                    |
          +---------+---------+
          |                   |
          v                   v
   durable storage      provider gateways
```

Provider SDKs and legacy calendar implementations belong behind adapters. Business-specific scheduling decisions, scoring, lineage semantics, identity administration, and unrelated workflow policy stay outside this bounded context.

## Quality and release posture

CalendarWeave's development contract requires behavior to be established test-first, owned production statement and branch coverage to reach 100%, public documentation to be usable without code archaeology, and protocol/security behavior to be exercised with realistic fixtures before release.

Those are development and release requirements, not claims that the current seed branch already satisfies a production-readiness certification. Current gaps and acceptance evidence are tracked in [`docs/product-technical-gap-baseline.md`](docs/product-technical-gap-baseline.md).

## Documentation

- [Architecture](ARCHITECTURE.md) — product responsibility, Context Map and migration design.
- [Documentation home](docs/index.md) — navigation for product and technical evidence.
- [Architecture decisions](docs/adr/) — accepted decisions and rejected alternatives.
- [Product and technical gap baseline](docs/product-technical-gap-baseline.md) — current gaps, owner paths and acceptance conditions.
- [Standards and research doctoring](docs/doctoring/) — RFC and research basis used by the design.
- [Changelog](CHANGELOG.md) — notable repository changes.

## Contributing and support

Before changing calendar behavior or ownership boundaries, read [`AGENTS.md`](AGENTS.md), [`CLAUDE.md`](CLAUDE.md), the architecture, and the applicable ADR. Keep consumer-specific policy outside CalendarWeave, preserve fail-closed capability boundaries, and update tests, public contracts, documentation, and migration evidence together when behavior changes.

Use the repository issue tracker for product defects, interoperability gaps, and integration questions. Do not infer support for an unreleased capability from an open branch or design document.

## License

CalendarWeave source and documentation are licensed under the [Apache License 2.0](LICENSE). Third-party dependencies and external services retain their own licenses and terms; they do not change the license of CalendarWeave's original source unless an applicable inbound obligation explicitly requires it.
