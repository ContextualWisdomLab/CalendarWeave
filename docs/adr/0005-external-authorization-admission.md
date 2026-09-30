# ADR-0005: External authorization admission before calendar processing

- Status: Accepted candidate; not protected-main or released truth
- Date: 2026-09-02
- Decision owner: CalendarWeave Calendar Resource Core
- Related: issue #2, PR #6, ADR-0002 through ADR-0004

## Context

CalendarWeave's executable core already carries a tenant scope through every collection and event operation, but that is not equivalent to authenticating or authorizing a caller. A commercial service boundary must fail closed before calendar parsing or resource lookup and must not duplicate the ContextualWisdomLab identity control plane inside the calendar domain.

JWT and OpenID Connect subject identifiers are issuer-scoped. RFC 7519 defines `sub` as a case-sensitive StringOrURI that is locally unique within the issuer or globally unique; OpenID Connect Core likewise defines a locally unique, never-reassigned subject identifier and distinguishes public and pairwise subject types. CalendarWeave therefore retains issuer and subject together instead of treating subject text as globally unique or imposing an invented character whitelist.

A tenant identifier supplied by an external caller is not authorization evidence. The authorization authority must derive the tenant scope that is permitted for the verified principal, requested action, and exact calendar-resource context.

## Decision

CalendarWeave adds an application-layer Anti-Corruption Layer with four contracts:

1. `ExternalIdentity` carries only opaque externally verified `issuer` + `subject` identity. It contains no caller-selected tenant scope and rejects only empty, defensively over-limit, or control-character-bearing identity strings. These byte limits are service-safety bounds, not identity-standard syntax rules.
2. `CalendarAuthorizationRequest` carries the typed `CalendarAction` plus the exact opaque collection/event references required for resource-scoped policy evaluation. Create-collection carries no resource reference; collection operations carry the collection reference; event reads/updates carry both collection and event references.
3. `CalendarAuthorizationPort` asks an external policy authority to authorize that identity/request pair. A successful decision returns the `TenantId` that the policy authority actually admitted; deny and unavailable outcomes remain explicit and fail closed.
4. `AuthorizedCalendarService` obtains that authorization-derived tenant before delegating to the underlying `CalendarPort`. In particular, event-create authorization happens before iCalendar parsing, so a denied caller cannot use parser behavior as an admission oracle. The caller cannot provide a tenant argument through this public admission surface.

The admission wrapper does not expose its inner calendar adapter. Tenant-bound resource operations delegate only the `TenantId` returned by `CalendarAuthorizationPort`, preserving the existing absent/cross-tenant indistinguishability at the Calendar Resource Core boundary while preventing a permissive policy adapter from accidentally blessing a caller-forged tenant scope.

A concrete Keyverse integration belongs in an infrastructure/service adapter that verifies the relevant issuer/token/session contract before constructing `ExternalIdentity`, then evaluates the exact `CalendarAuthorizationRequest` and derives the authorized tenant from trusted identity/policy state. No Keyverse implementation bytes, raw credentials, or caller-selected tenant become CalendarWeave domain state.

## DDD impact

The Calendar Resource Core remains the core subdomain. Authorization admission is a supporting application boundary around that core, with Keyverse/another approved identity-policy service as an upstream external bounded context. The context-map relationship is conformist only for verified identity facts and ACL-mediated for authorization decisions. Calendar collections and event revisions remain the transactional aggregates; admission decisions do not enlarge their transaction boundary.

Ubiquitous language introduced by this decision:

- **External identity** — externally verified issuer/subject evidence with no CalendarWeave tenant authority of its own.
- **Calendar authorization request** — a typed operation plus the exact collection/event resource references required for policy evaluation.
- **Authorization decision** — external allow/deny/unavailable outcome; an allow result derives the admitted tenant scope.
- **Admission boundary** — the application service that maps external authorization into fail-closed CalendarWeave errors and then delegates to the Calendar Resource Core using only the authorization-derived tenant.

## Security and privacy consequences

- Denied and unavailable authorization states never fall through to calendar parsing or mutation.
- A caller cannot self-assert `TenantId` through `ExternalIdentity` or `AuthorizedCalendarService`; tenant scope is derived by the trusted authorization adapter for each exact request.
- Collection/event references are presented to the authorization authority so resource-scoped grants need not degrade into tenant-wide grants.
- The core receives no raw bearer token, provider credential, or customer identity secret.
- Issuer and subject are operational identity evidence; this decision does not authorize logging them or attendee/calendar PII in ordinary telemetry.
- The existing non-masking boundary remains: fields required to perform calendar work stay usable under least privilege, tenant/purpose isolation, encryption, retention and access/audit controls rather than destructive masking.
- This candidate does not claim service authentication, token verification, authorization-policy persistence, SOC 2/CSAP attestation, deployment, or audit-retention completion.

## Alternatives rejected

### Local CalendarWeave identity provider

Rejected because it duplicates Keyverse responsibility and creates a second credential/session authority inside a reusable calendar module.

### Accept a tenant identifier as sufficient authorization

Rejected because possession of a tenant string is scope data, not proof that the principal may perform the requested action. Tenant scope is therefore absent from `ExternalIdentity` and must be returned by a trusted authorization decision.

### Authorize only by coarse action

Rejected because an action-only decision cannot enforce collection- or event-scoped grants. The authorization request carries the exact opaque target references needed by the policy authority.

### Parse the calendar payload before authorization

Rejected because it spends parser resources for a denied caller and exposes a distinguishable pre-authorization behavior surface.

### Character-whitelist the external subject

Rejected because external subject syntax is issuer/application specific. CalendarWeave needs bounded opaque handling, not an invented identity grammar.

## Verification

PR #6 preserves a test-first sequence. Its regression contract covers deny-before-parse ordering, unavailable-authorization failure, bounded opaque issuer/subject handling, issuer+subject identity distinction, prevention of caller-selected tenant scope, exact collection/event authorization context, cross-tenant resource isolation and successful CRUD/ETag behavior through the wrapper. Exact-current-head repository and central checks remain mandatory; predecessor-head results do not transfer.

## References

Jones, M., Bradley, J., & Sakimura, N. (2015). *JSON Web Token (JWT)* (RFC 7519). RFC Editor. https://doi.org/10.17487/RFC7519

OpenID Foundation. (2014). *OpenID Connect Core 1.0 incorporating errata set 2*. https://openid.net/specs/openid-connect-core-1_0.html
