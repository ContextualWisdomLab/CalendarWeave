# CalendarWeave

CalendarWeave is ContextualWisdomLab's governed calendar-resource product for interoperable calendar collections, events, revisions, and iCalendar/CalDAV semantics.

## What is being built

The first release target is a small, independently testable calendar core that can create and retrieve tenant-scoped RFC 5545 events, preserve stable resource identity and revision evidence, enforce timezone and explicit interval semantics, and persist resources without exposing application database internals to consumers. The current candidate stack accepts bounded `DTEND` intervals and positive RFC 5545 `DURATION` intervals while broader recurrence, `VTIMEZONE`, CalDAV, provider, service-authentication, and release capabilities remain gated.

## Current status

The project is in active foundation development. A public production endpoint, production deployment, and consumer migration are not yet claimed. Supported behavior becomes authoritative only after the relevant work reaches the protected default branch and is released with its required verification evidence.

## Start here

- [Repository overview](../README.md)
- [Architecture](../ARCHITECTURE.md)
- [Architecture decisions](adr/)
- [RFC 5545 DURATION traceability](doctoring/rfc5545-duration-baseline.md)
- [Product and technical gap baseline](product-technical-gap-baseline.md)
- [DeepWiki](https://deepwiki.com/ContextualWisdomLab/CalendarWeave)

## Integration boundary

CalendarWeave owns generic calendar-resource and interoperability behavior. Consumer-specific scheduling decisions, domain calculations, and workflow policy remain with the consuming products and connect through versioned contracts rather than direct database access.
