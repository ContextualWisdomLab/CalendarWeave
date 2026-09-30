# CalendarWeave agents

CalendarWeave owns the reusable **generic calendar-resource** bounded context. It is the target CalDAV/iCalendar PIMS for ContextualWisdomLab, but protected `main` is still a seed until an executable production contract is merged and released.

## Context Map

- **CalendarWeave:** calendar collections/resources, iCalendar/CalDAV semantics, provider adapters, revisions/sync evidence, calendar authorization/audit.
- **Naruon:** workspace commitment meaning, conflict/resolution policy, cross-context evidence, approval/correction and buyer workflow. Consume CalendarWeave through a versioned Calendar Port/ACL.
- **LineageWeave:** read-only calendar/evidence composition and deep links. Never own a calendar store or provider credentials.
- **saju-caldav:** saju-specific profile/candidate/scoring/explanation and publication intent. Its current Radicale stack is a compatibility path until CalendarWeave proves production parity; generic CalDAV ownership then migrates here.
- **four-pillars:** deterministic Four Pillars calculation/reporting. Never move that computation into CalendarWeave.
- **Keyverse:** identity/federation and external authorization policy. CalendarWeave accepts tenant-free verified issuer/subject evidence, derives tenant scope only from the trusted resource-aware authorization decision, and does not implement a local IdP.

## Do

- Implement RFC 5545 VEVENT and RFC 4791 CalDAV collection semantics test-first.
- Treat `DTEND` and positive RFC 5545 `DURATION` as the bounded v1 interval alternatives under ADR-0007; keep unsupported recurrence, floating time, `VTIMEZONE`, and unhandled parameters fail-closed until explicitly versioned.
- Add RFC 5546 iTIP, RFC 6578 sync and RFC 6638 scheduling only as explicit versioned capabilities with discovery/fail-closed behavior.
- Keep standalone service and module/package consumption paths both possible through published contracts.
- Organize source by real bounded-context responsibility; provider SDKs belong behind adapters/Anti-Corruption Layers, never inside domain entities.
- Use two-or-more-word snake_case 3NF names when relational persistence is introduced.
- Treat PII as purpose-limited access plus audit. Do not blanket-mask attendee or organizer fields required for calendar work.
- Cite RFCs and material research in `docs/doctoring` with APA 7th.
- Maintain `ARCHITECTURE.md`, ADRs and `docs/product-technical-gap-baseline.md` so protected-main truth and target architecture are clearly distinguished.
- Require RED before production behavior, 100% owned production statement/branch coverage, and beginner-readable multi-line shipped documentation.

## Do not

- Embed calendar chrome or Naruon commitment logic here.
- Copy Naruon Google Calendar code or `saju-caldav` Radicale code as the implementation strategy; extract required behavior into consumer parity fixtures instead.
- Put mail/threading, project/task semantics, LineageWeave ontology, Four Pillars calculation, saju scoring, GRC registries or identity-provider behavior in this repository.
- Let consumers read CalendarWeave application tables directly or leak provider DTOs into their domain models.
- Delete an existing consumer compatibility path before a released CalendarWeave contract proves parity.
- Self-approve, force-push, bypass protection, request Copilot, or use `COPILOT_GITHUB_TOKEN`.

## Migration discipline

1. Implement and release CalendarWeave capability first.
2. Consumer writes characterization tests for its current behavior.
3. Consumer adds a versioned port/ACL and proves parity/security/failure semantics.
4. Remove duplicated generic calendar/provider code only after parity.
5. Add architecture-fitness tests preventing the duplication from returning.

## Merge

Keep foundational product PRs Draft until the corresponding capability is executable. Documentation establishes ownership but is not product-completion evidence. Merge classification must use exact-current-head checks/reviews and live repository policy.
