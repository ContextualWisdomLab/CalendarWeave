# ADR-0005: External authorization admission before calendar processing

- Status: Accepted candidate; not protected-main or released truth
- Date: 2026-09-02
- Decision owner: CalendarWeave Calendar Resource Core
- Related: issue #2, PR #6, ADR-0002 through ADR-0004

## Context

CalendarWeave's executable core already carries a tenant scope through every collection and event operation, but that is not equivalent to authenticating or authorizing a caller. A commercial service boundary must fail closed before calendar parsing or resource lookup and must not duplicate the ContextualWisdomLab identity control plane inside the calendar domain.

JWT and OpenID Connect subject identifiers are issuer-scoped. RFC 7519 defines `sub` as a case-sensitive StringOrURI that is locally unique within the issuer or globally unique; OpenID Connect Core likewise defines a locally unique, never-reassigned subject identifier and distinguishes public and pairwise subject types. CalendarWeave therefore retains issuer and subject together instead of treating subject text as globally unique or imposing an invented character whitelist.

## Decision

CalendarWeave adds an application-layer Anti-Corruption Layer with three contracts:

1. `ScopedIdentity` carries opaque external `issuer` + `subject` identity and the already-admitted `TenantId`. It rejects only empty, defensively over-limit, or control-character-bearing identity strings. These byte limits are service-safety bounds, not identity-standard syntax rules.
2. `CalendarAuthorizationPort` asks an external policy authority for a typed `CalendarAction` decision. The Calendar Resource Core does not validate bearer tokens, mint sessions, implement an IdP, or persist a second authorization-policy store.
3. `AuthorizedCalendarService` requires an affirmative authorization decision before delegating to the underlying `CalendarPort`. In particular, event-create authorization happens before iCalendar parsing, so a denied caller cannot use parser behavior as an admission oracle. Authorization-provider unavailability is distinguishable from a completed deny decision and fails closed.

The admission wrapper does not expose its inner calendar adapter. Tenant-bound resource operations continue to delegate the identity's `TenantId`, preserving the existing absent/cross-tenant indistinguishability at the Calendar Resource Core boundary.

A concrete Keyverse integration belongs in an infrastructure/service adapter that verifies the relevant issuer/token/session contract before constructing `ScopedIdentity` and translating policy decisions into `AuthorizationError`. No Keyverse implementation bytes or credentials become CalendarWeave domain state.

## DDD impact

The Calendar Resource Core remains the core subdomain. Authorization admission is a supporting application boundary around that core, with Keyverse/another approved identity-policy service as an upstream external bounded context. The context-map relationship is conformist only for verified identity facts and ACL-mediated for authorization decisions. Calendar collections and event revisions remain the transactional aggregates; admission decisions do not enlarge their transaction boundary.

Ubiquitous language introduced by this decision:

- **Scoped identity** — external issuer/subject principal plus one admitted tenant scope.
- **Calendar action** — a typed operation requested at the Calendar Resource boundary.
- **Authorization decision** — external allow/deny/unavailable outcome evaluated before domain processing.
- **Admission boundary** — the application service that maps external authorization into fail-closed CalendarWeave errors and then delegates to the Calendar Resource Core.

## Security and privacy consequences

- Denied and unavailable authorization states never fall through to calendar parsing or mutation.
- The core receives no raw bearer token, provider credential, or customer identity secret.
- Issuer and subject are operational identity evidence; this decision does not authorize logging them or attendee/calendar PII in ordinary telemetry.
- The existing non-masking boundary remains: fields required to perform calendar work stay usable under least privilege, tenant/purpose isolation, encryption, retention and access/audit controls rather than destructive masking.
- This candidate does not claim service authentication, token verification, authorization-policy persistence, SOC 2/CSAP attestation, deployment, or audit-retention completion.

## Alternatives rejected

### Local CalendarWeave identity provider

Rejected because it duplicates Keyverse responsibility and creates a second credential/session authority inside a reusable calendar module.

### Accept a tenant identifier as sufficient authorization

Rejected because possession of a tenant string is scope data, not proof that the principal may perform the requested action.

### Parse the calendar payload before authorization

Rejected because it spends parser resources for a denied caller and exposes a distinguishable pre-authorization behavior surface.

### Character-whitelist the external subject

Rejected because external subject syntax is issuer/application specific. CalendarWeave needs bounded opaque handling, not an invented identity grammar.

## Verification

PR #6 preserves a test-first sequence. Its regression contract covers deny-before-parse ordering, unavailable-authorization failure, bounded opaque issuer/subject handling, issuer+subject identity distinction, cross-tenant resource isolation and successful CRUD/ETag behavior through the wrapper. Exact-current-head repository and central checks remain mandatory; predecessor-head results do not transfer.

## References

Jones, M., Bradley, J., & Sakimura, N. (2015). *JSON Web Token (JWT)* (RFC 7519). RFC Editor. https://doi.org/10.17487/RFC7519

OpenID Foundation. (2014). *OpenID Connect Core 1.0 incorporating errata set 2*. https://openid.net/specs/openid-connect-core-1_0.html
