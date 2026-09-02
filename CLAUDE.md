# CalendarWeave contributor context

Use [`AGENTS.md`](AGENTS.md) as the authoritative repository instruction set and [`ARCHITECTURE.md`](ARCHITECTURE.md) plus accepted ADRs as the product boundary.

CalendarWeave owns the generic calendar-resource bounded context. Keep Naruon commitment/conflict policy, LineageWeave evidence composition, `saju-caldav` domain-specific scoring, Four Pillars computation, and Keyverse identity/federation policy outside this repository. External identity admitted to CalendarWeave is tenant-free issuer/subject evidence; the trusted authorization decision derives the tenant for the exact calendar action/resource request.

Implement RFC behavior test-first. The current candidate profile accepts explicit `DTEND` intervals and, under ADR-0007, positive RFC 5545 `DURATION` intervals as mutually exclusive alternatives. DATE starts accept only day/week duration forms; unsupported recurrence, floating time, `VTIMEZONE`, provider and CalDAV capabilities must continue to fail closed until separately versioned and verified.

Preserve descriptive multiword `snake_case` persistence, 3NF, item-level idempotency, immutable UID/revision/ETag invariants, authorization-before-parse, exact-head evidence, and ordinary governance. Do not self-approve, force-push, destructively rebase, bypass protection to obtain a merge, or introduce `COPILOT_GITHUB_TOKEN`.
