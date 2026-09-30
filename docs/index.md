# CalendarWeave

CalendarWeave is ContextualWisdomLab's governed calendar-resource product for interoperable calendar collections, events, revisions, and iCalendar/CalDAV semantics.

## What is being built

The first release target is a small, independently testable calendar core that can create and retrieve tenant-scoped RFC 5545 events, preserve stable resource identity and revision evidence, enforce timezone and interval semantics, and persist resources without exposing application database internals to consumers.

## Current status

The project is in active foundation development. A public production endpoint, production deployment, and consumer migration are not yet claimed. Supported behavior becomes authoritative only after the relevant work reaches the protected default branch and is released with its required verification evidence.

## Start here

- [Repository overview](../README.md)
- [Architecture](../ARCHITECTURE.md)
- [Architecture decisions](adr/)
- [DeepWiki](https://deepwiki.com/ContextualWisdomLab/CalendarWeave)

## Integration boundary

CalendarWeave owns generic calendar-resource and interoperability behavior. Consumer-specific scheduling decisions, domain calculations, and workflow policy remain with the consuming products and connect through versioned contracts rather than direct database access.
