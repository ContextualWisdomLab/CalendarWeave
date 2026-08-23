# CalendarWeave agents

Own calendar only. This repo is the CalDAV/iCalendar PIMS. naruon, LineageWeave GNB, and Outlook consume it fail-closed.

## Do

- Implement RFC 5545 VEVENT and RFC 4791 CalDAV collection semantics.
- Keep standalone service and module-import paths both working.
- Use two-or-more-word snake_case 3NF names (`calendar_events`, `calendar_collections`, `event_attendees`).
- Treat PII as purpose-limited access plus audit. Do not mask attendee or organizer fields.
- Cite RFCs and papers in `docs/doctoring` with APA 7th.

## Do not

- Embed calendar chrome in naruon mail or LineageWeave #74.
- Copy saju-caldav, local IdP, GRC registries, mail, or tasks here.
- Self-approve, squash to main, request Copilot, or use COPILOT_GITHUB_TOKEN.
- Launch Cloud Agents into an empty tree; seed files first.

## Merge

Independent current-head OpenCode APPROVE is the naruon-family receipt. Copilot is forbidden. Keep first product PRs Draft until CalDAV create/list/get reproduces a real VEVENT.
