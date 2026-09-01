# CalendarWeave

[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/ContextualWisdomLab/CalendarWeave)

**Governed calendar resources, iCalendar semantics, and interoperable scheduling infrastructure.**

CalendarWeave is the ContextualWisdomLab calendar-resource product: a focused home for versioned calendar collections, events, revisions, timezone/recurrence semantics, CalDAV interoperability, and provider-facing calendar adapters.

## Current status

CalendarWeave is under active foundation development. The current work is establishing a tenant-scoped Calendar Resource Core, durable persistence, and strict RFC 5545 time semantics before any production service or migration claim is made.

The repository does **not** yet advertise a released CalDAV endpoint or production deployment. Follow the repository releases and architecture documentation for supported capabilities as they become available.

## Product direction

The first releasable vertical is intentionally small and testable:

1. create a tenant-scoped calendar collection;
2. create and retrieve an RFC 5545 VEVENT with stable identity and revision evidence;
3. preserve timezone and interval semantics fail-closed;
4. persist the resource through a durable adapter; and
5. expose a versioned application contract that other products can consume without database coupling.

## Documentation

- [Architecture](ARCHITECTURE.md)
- [Documentation home](docs/index.md)
- [Architecture decisions](docs/adr/)

CalendarWeave keeps generic calendar-resource responsibility separate from consumer-specific workflow or domain policy. Those integration boundaries are documented in the architecture and ADRs rather than exposed as customer-facing repository instructions.
