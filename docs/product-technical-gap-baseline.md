# CalendarWeave product and technical gap baseline

## Snapshot

Protected `main` remains the seed `d972ccae6225716bdff7210a1fed808c01d32689`; the live repository is non-fork and protected main still contains no released runtime, package, service, container, CalDAV endpoint, provider adapter, or consumer migration. PRs #3/#4/#5 have merged into the open PR #1 branch, which now carries the Rust core, PostgreSQL adapter and bounded IANA `TZID`. PR #6 authorization admission, #7 logical recovery, #8 bounded RFC 5545 `DURATION`, and #9 RFC 5545 `CLASS` privacy intent remain stacked candidates.

CalendarWeave remains genuinely very early-stage because the buyer-facing workflow is not installable or operated and foundational service authentication, release/deployment, CalDAV/provider parity, privacy/audit operation, measured recovery, and downstream migration are still missing. PR #9 is a bounded commercialization intervention because `saju-caldav` parity explicitly needs privacy-classification semantics and RFC 5545 provides a standard contract that can be added without widening CalendarWeave into disclosure policy.

## Product responsibility and DDD boundary

| Responsibility | Owner / bounded context | Current evidence |
| --- | --- | --- |
| Calendar collections, events, UID/revision/ETag invariants, RFC 5545 resource semantics | CalendarWeave / Calendar Resource Core | #3/#5 content is integrated on PR #1; #8/#9 are later semantic candidates; none is released |
| Calendar privacy intent | CalendarWeave / Calendar Resource Core | #9 `EventClass` projection from canonical iCalendar; not authorization |
| Calendar operation admission | CalendarWeave / Authorization Admission | #6 candidate; tenant-free issuer/subject evidence, exact resource request, authorization-derived tenant |
| Identity/federation and external authorization policy | Keyverse | External authority behind `CalendarAuthorizationPort`; no copied identity/policy store |
| Relational durability/concurrency | CalendarWeave / PostgreSQL adapter | #4 content is integrated on PR #1, with 3NF append-only revisions and row-locked conditional updates |
| Logical recovery | CalendarWeave / operations boundary | #7 candidate; checksum-before-restore and invariant drill, not PITR/HA/RPO/RTO evidence |
| CalDAV/provider interoperability and synchronization | CalendarWeave / interoperability adapters | Target responsibility; no released endpoint/provider parity |
| Workspace commitment/conflict/resolution policy | Naruon | Supporting consumer context behind a versioned Calendar Port/ACL |
| Calendar/evidence composition | LineageWeave | Read-only composition/deep-link responsibility; no calendar store or mathematical computation |
| Saju scoring/explanation/publication intent | `saju-caldav` | Separate domain; generic CalDAV compatibility migrates only after CalendarWeave parity |
| Deterministic Four Pillars computation | `four-pillars` | Separate mathematical product responsibility |

Core subdomain: governed calendar-resource semantics and mutation/revision invariants. Supporting subdomains: Authorization Admission plus CalDAV/provider interoperability. Generic/external capabilities: identity/federation, PostgreSQL, telemetry and deployment platform. `CalendarCollection` owns collection-scoped membership. `CalendarEvent` is the current immutable-UID revision projection. `TenantId` and tenant-free `ExternalIdentity` are value objects. `EventClass` is descriptive privacy intent. `CalendarAuthorizationRequest` carries action and opaque resource references only. External identity/provider representations terminate behind ACLs.

Transactions remain item-scoped: create is idempotent by collection + RFC UID; conditional update locks one event row and advances one revision under the expected strong ETag. `CLASS` is derived from the canonical immutable `icalendar_payload`; #9 deliberately adds no duplicate persistence column or transaction boundary.

## Current feature specification

The candidate Calendar Resource Core supports tenant-scoped collection create and VEVENT create/update/list/get. Supported VEVENTs require RFC 5545 `VERSION:2.0`, `PRODID`, UID, UTC `DTSTAMP`, start, summary, and exactly one explicit interval form. Case-insensitive standard confirmed/tentative/cancelled status and non-negative `SEQUENCE` are bounded optional fields.

Time and privacy behavior is explicit:

- `DTEND` supports UTC, all-day DATE, and matching bounded IANA `TZID` intervals, rejecting non-increasing intervals.
- #8 accepts positive RFC 5545 `DURATION` as the alternative to `DTEND`; both, neither, duplicate, negative/zero, calendar-month/year, fractional, reordered, and unsupported-parameter forms fail closed under the bounded v1 profile.
- DATE `DTSTART` accepts only day/week duration forms. Named-timezone starts reuse the existing ambiguity/nonexistence/unknown-zone fail-closed contract.
- #9 accepts one optional RFC 5545 `CLASS`. Omission projects as `PUBLIC`; `PUBLIC`/`PRIVATE`/`CONFIDENTIAL` are case-insensitive; valid unrecognized IANA/experimental token values project as `PRIVATE`; IANA/non-standard parameters remain interoperable; duplicate, empty or non-token values fail malformed.
- Singleton validation unfolds RFC 5545 folded content lines before counting properties, so a folded second `CLASS` cannot overwrite the first value inside the dependency parser.
- `CalendarEvent::classification()` revalidates the same complete bounded event profile before returning a classification, so a manually forged projection with an unsupported property or other invalid raw payload cannot recover apparently trusted privacy metadata.
- `CLASS` is calendar-owner intent only. A public value cannot override denied/unavailable authorization, and private/confidential values do not themselves grant or enforce access.
- Floating local time, `VTIMEZONE`, recurrence/free-busy expansion and unversioned provider/CalDAV capabilities remain outside the candidate profile.

Persistence remains 3NF with descriptive multiword `snake_case` objects: `calendar_collection`, `calendar_event`, and `calendar_event_revision`. Canonical event content remains `icalendar_payload`; neither DURATION nor CLASS introduces a parallel relational source of truth.

## Exact stack evidence observed in this iteration

| Lane | Exact head / evidence | Current status / next verification |
| --- | --- | --- |
| protected `main` | `d972ccae6225716bdff7210a1fed808c01d32689` | seed only; no released product surface |
| #1 `docs/adr-baseline` | `5c85765adb3e3d9514e387921103d2fda7944c18`; Tests `36686207005` and SAST `36686206972` succeeded; Security Scan `36686206813` failed closed on Dependency Review HTTP 403; CodeQL `36686206923` skipped | Executable core, PostgreSQL and bounded `TZID` are integrated but not on protected `main`; `.github#810` owns the dependency-review availability blocker and independent approval is still absent |
| PRs #3/#4/#5 | Closed and merged into PR #1 branch | Their successful exact-head Rust and coverage checks supported branch integration; PR #1 needs its own checks before protected-main merge |
| #6 `feat/authorization-admission-v1` | `3b2b3a532df4663dd8d2af9e4b6cca444fdfee93`; Tests `36686514860` succeeded and its hosted-evidence thread is resolved | Independent approval is absent; the central Security/SAST/CodeQL suite did not materialize on the stacked base |
| #7 `feat/postgres-recovery-v1` | `f31728a5a9009e65bc560dabdf450fa940bd809a`; Tests `36686710710` succeeded | RED recovery contract preceded production scripts; independent approval and stacked-base central scans remain required |
| #8 `feat/rfc5545-duration-v1` | `88844ad55c94cf3ed4f362bf4bea5ef579a20513`; Tests `36688373491` succeeded | Test-first `DURATION` lane and exact-head repository checks are GREEN; independent approval and stacked-base central scans remain required |
| #9 `feat/rfc5545-class-v1` | `65dee82a88ad5f12973d32843eddd45bb887781f`; Tests `36892959099` passed `rust`, `coverage`, and `recovery`. Test-only `ffea407601571da76498cd1deb5bc84cae27aa5f` exposes folded duplicate `CLASS`; production repair `dcd510a424b2c2d137260e71684558f6a502d390` reuses the dependency parser's unfolding contract | Local Rust 1.97.1 RED→GREEN is observed. Hosted run `37111484172` for the RED head failed before all three jobs acquired a runner (`steps=[]`, no log artifact), so it is not product evidence. Exact-head hosted checks, independent approval, and stacked-base central scans remain required |
| central security foundation | ContextualWisdomLab/.github #810 and #2073 | #810 owns fail-closed Dependency Review availability; #2073 owns missing exact-head central scans for feature-base stacked PRs. Leaf branches must not copy workflows, inherit predecessor receipts or weaken gates |

The live governance path requires exact-current-head checks/reviews. PRs #3/#4/#5 were marked Ready through the CLI and merged into PR #1 without self-approval, admin bypass or protection weakening.

## PR #9 TDD and research traceability

The initial RED commit `b91602811a231c726ab5fbc5a2e0a1af894e9346` added `tests/rfc5545_classification.rs` before production CLASS support. A standards audit then corrected overly narrow assumptions before final production behavior: `18d3ea264d2f0c3bfeea10e5af6fa01ff4bbe706` requires case-insensitive standard values, extension-parameter interoperability, and fail-private handling for valid unknown registered/experimental values. `707fcecdb45a273028b2d7966888c3a507d268d5` implements that corrected Rust contract.

A subsequent exact-head static review found that the public classification accessor re-parsed only singleton/component/classification structure rather than the entire supported event profile. That meant a manually forged public `CalendarEvent` could retain a valid `CLASS` while gaining an unsupported property and still return a classification. `bfe078a556677ec99d76c87533bcbd5967836da6` added the RED regression first; `b12f95dee6b3a73f87a56316a48830f45ae3612b` then made the accessor reuse `parse_event` and moved the parsed classification into `ParsedEvent`, so classification now inherits the same fail-closed full-profile validation as event admission.

The same standards review exposed an older interoperability defect outside the new `CLASS` projection: `STATUS` matched only uppercase spellings even though RFC 5545 enumerated values are case-insensitive. `d34cf18fb8037be144c44012c51c1d3f41065651` added the focused RED contract; hosted Tests `36892549369` failed in both `rust` and `coverage` with `MalformedCalendar` while recovery remained green. `4e26597bb52650dd64a5d40ca92376194bfb0b1f` changes only the three standard `STATUS` comparisons to ASCII case-insensitive matching and preserves duplicate, parameterized and unknown-value rejection.

Exact-head review then found that duplicate-singleton validation counted physical lines before RFC 5545 unfolding while `icalendar` 0.17.13 unfolded before parsing. A folded second `CLASS` could therefore replace the first parsed value. Test-only `ffea407601571da76498cd1deb5bc84cae27aa5f` adds space- and tab-continuation regressions; local Rust 1.97.1 execution observed the expected RED `Ok(CalendarEvent)` result. `dcd510a424b2c2d137260e71684558f6a502d390` makes the shared singleton counter reuse `icalendar::parser::unfold`. The focused regression, all six classification tests, the 48-test all-feature suite, formatting, warning-fatal Clippy, and warning-fatal rustdoc pass locally. Hosted run `37111484172` did not execute a step and is retained only as runner-start incident evidence.

ADR-0008 and `docs/doctoring/rfc5545-class-privacy-baseline.md` bind the candidate to RFC 5545 sections 3.1 and 3.8.1.3. RFC 5545 defines omitted CLASS as PUBLIC, permits IANA/non-standard parameters, requires unrecognized iana-token/x-name values to be treated like PRIVATE, and explicitly warns that CLASS is owner intent rather than an enforcement statement. The same RFC states enumerated values are case-insensitive. These semantics are represented directly rather than replaced with a local heuristic.

The classification projection is derived from the validated immutable event payload. This keeps persistence normalized and avoids introducing a synchronization invariant between a second classification column and canonical iCalendar content.

## 2026-10-03 executable integration increment

A dedicated integration branch preserves PR #9 at `6f5c674` and the package
preflight at `e913958` through normal Git ancestry. It adds a trusted local
PostgreSQL operator executable without changing the Calendar Port or external
admission contract. The default branch and predecessor PR branches are unchanged.
See ADR-0009 and `docs/local-operator-quickstart.md` for actual commands.

Real disposable PostgreSQL and separate executable processes verify initialization,
collection/event creation, byte-preserving readback, idempotency, conditional
updates, tenant isolation, storage failures and bounded credential-free errors.
The local coverage receipt for the CLI tree reports 1082/1082 lines and 184/184
explicit branches, with all four production source files included. It is local
candidate evidence, not current-head hosted checks, review approval or release.

The organization already contains authentication implementations. Keyverse main
at `7d9151cd` defines the RP contract; saju-caldav main at `fa72a2c8` implements
JWT/JWKS verification. Keyverse PR103 at `5ac33256` has an operator-protected
software/menu PDP candidate, not the least-privilege CalendarWeave runtime API
that binds verified issuer/subject, exact action and calendar references to an
authorized tenant. That missing contract remains a Keyverse-owned integration
dependency. No software/menu allow is promoted to calendar-resource authority.

This increment does not narrow the standalone-product objective to a CLI.
HTTP/CalDAV, real issuer acceptance, purpose-bound authorization audit, production
release/deployment, operated recovery and consumer parity remain incomplete.

## 2026-10-03 bounded metadata source increment

ADR-0010 extends the Calendar Port v0.1 source candidate with optional single
parameter-free `DESCRIPTION` and `TRANSP`. The original payload remains canonical;
no schema or dependency changes are introduced. Standard transparency values are
case-insensitive; omission is preserved, with RFC default `OPAQUE` but no busy-policy
projection. All parameters remain `UnsupportedCapability`, including RFC-permitted
`LANGUAGE`/`ALTREP`. Description fixtures prove parser acceptance and preservation,
not a complete independent TEXT grammar validator.

Twelve exact synthetic consumer serializer outputs and the metadata tests exercise
this slice, revision/tenant boundaries, and the real admission wrapper with synthetic
authorization decisions. Historical native RED/GREEN receipts are distinct from
normal Cargo acceptance; final exact-source gate receipts must be read separately.
Independent review, hosted CI, package release, authenticated standalone service,
CalDAV collections, DELETE, unconditional PUT, full consumer parity, and cutover
remain open. No existing consumer compatibility implementation is removed.

## Open issue state

Issue #2 remains the canonical commercialization tracker and stays open. #9 addresses only the generic RFC 5545 CLASS portion of `saju-caldav` parity. Current evidence still does not prove released CalDAV/provider parity, concrete service authentication, operated disaster recovery, privacy/retention/export/audit controls, versioned distribution, or consumer cutover.

## Commercialization gaps

| Gap | Owner | Current evidence | Smallest next action | Completion evidence |
| --- | --- | --- | --- | --- |
| Service authentication / Keyverse integration | CalendarWeave + Keyverse | #6 proves an in-process authorization ACL only | implement a concrete verified issuer/token/session adapter behind the existing port | invalid signature/issuer/algorithm/audience/subject/time, dependency failure, tenant/resource mismatch and authorized fixtures |
| Operated durability | CalendarWeave deployment | #7 logical restore candidate only | encrypted retained remote backups, WAL/PITR where required, rollback/monitoring and measured exercises | exact measured RPO/RTO/PITR/restore evidence; no logical-backup overclaim |
| `VTIMEZONE` / remaining RFC 5545 profile | Calendar Resource Core | bounded IANA zone + DURATION + CLASS candidates | add standards-backed `VTIMEZONE` slice test-first | real RFC fixtures and DST edges under exact-head coverage |
| CalDAV/provider parity | CalendarWeave interoperability | no endpoint/provider adapter released | add protocol/provider ACL after the core contract stabilizes | real provider/CalDAV fixtures and reversible migration evidence |
| Privacy/content + authorization audit | CalendarWeave + deployment | CLASS intent candidate, necessary calendar PII, logical backup | define purpose/retention/access/export/audit and encrypted backup access | CSAP/SOC 2-oriented control map plus operated evidence without certification claims |
| Release/package/service | CalendarWeave | no versioned artifact | define package/service contract, compose deployment, SBOM/provenance and rollback | immutable versioned artifact/service plus real install/call path |
| Consumer migration | Naruon / `saju-caldav` / LineageWeave | compatibility implementations remain | characterization tests then versioned ACLs after release | parity/security/failure semantics, no direct table coupling, reversible cutover |
| Hosted exact-head verification | ContextualWisdomLab/.github #712 | CalendarWeave #6/#7/#8/#9 runner-backed jobs can remain queued/unassigned | repair central queue-amplification owner path and revalidate unchanged leaf heads | terminal current-head repository + semantic/security evidence |
| Review lifecycle | CalendarWeave PR stack | #3/#4/#5 were marked Ready through the CLI and merged into PR #1; #6/#7/#8/#9 remain Draft | advance each executable candidate with current-head checks and review evidence, without bypass or no-op churn | ordinary Ready state and downstream independent review dispatch |

## Quality, security, persistence and operability invariants

- Behavior changes begin with executable RED contracts; owned production statement/branch coverage and public-doc coverage target 100%.
- No deprecation-warning suppression, production synthetic data, self-approval, force-push, destructive rebase, routine bypass, or protection weakening.
- CalendarWeave and LineageWeave contain no mathematical/psychometric computation that belongs in dedicated mathematical owners.
- Authorization precedes untrusted calendar parsing/mutation. Cross-tenant and absent-resource observations remain indistinguishable at the core boundary.
- RFC 5545 `CLASS` is not access-control authority; valid unknown tokens fail-private to avoid accidental widening, public classification projections must pass the full supported-event validator before being trusted, and singleton counts must use the same unfolded content-line representation as the parser.
- No raw bearer token/provider credential becomes a Calendar Resource attribute or ordinary telemetry field.
- Necessary calendar PII is protected through least privilege, purpose/tenant isolation, encryption, retention, export/access audit and test anonymization rather than blanket masking that breaks calendar work.
- Relational persistence stays normalized; item-level idempotency/UPSERT semantics remain explicit; writes lock only the required item.
- Logical-backup digest verification is integrity evidence, not encryption, provenance, PITR, HA or RPO/RTO evidence.
- Web p95/k6 gates become applicable only when a web/service surface exists; no absent web surface is represented as load-tested.
- The revision counter must fail closed at `u64::MAX` without replacing the current event; named-timezone parsing must reject malformed local end values before interval comparison.
- Exact-head checks, live reviews/threads, rulesets and concurrent writer state are re-read after every branch move; stale/queued/cancelled evidence is non-passing.
- Logical restore must remain fail-closed on missing/malformed/tampered evidence and prove post-restore relational invariants. A successful logical drill alone cannot satisfy production recovery/PITR/RPO/RTO gates.
- Release additionally requires security/SBOM/provenance, compose-compatible operability, rollback/recovery, real service authentication, realistic protocol/consumer evidence and PII/audit controls.

## Required development order

1. Reacquire exact-head repository and semantic/security evidence for #6/#7/#8/#9 while independently reducing real product gaps.
2. Review PR #1's integrated core and advance #6/#7/#8/#9 only after their current-head checks; preserve stack order and ordinary review gates.
3. Repair the central queue-amplification owner path without leaf churn, then revalidate unchanged CalendarWeave heads.
4. Establish concrete Keyverse/service authentication and operated recovery/release evidence.
5. Add the next standards-backed `VTIMEZONE` capability, then CalDAV/provider interoperability fixtures.
6. Migrate Naruon, `saju-caldav`, and LineageWeave only after released parity evidence exists.

## Evidence references

- CalendarWeave PRs #1, #3, #4, #5, #6, #7, #8, #9 and issue #2.
- ADR-0001 through ADR-0008.
- `docs/doctoring/identity-authorization-admission-baseline.md`.
- `docs/doctoring/postgresql-logical-recovery-baseline.md`.
- `docs/doctoring/rfc5545-duration-baseline.md`.
- `docs/doctoring/rfc5545-class-privacy-baseline.md`.
- ContextualWisdomLab/.github #712 for organization runner-queue causal evidence.
- Desruisseaux, B. (Ed.). (2009). *Internet calendaring and scheduling core object specification (iCalendar)* (RFC 5545). RFC Editor. https://doi.org/10.17487/RFC5545
