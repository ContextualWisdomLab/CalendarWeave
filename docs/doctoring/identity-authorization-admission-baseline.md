# Identity and authorization admission evidence baseline

## Scope

This note binds the CalendarWeave authorization-admission candidate to standards and exact source responsibilities. It is engineering/research traceability, not a certification, deployment, identity-provider, or release claim.

## Standards-derived identity contract

RFC 7519 defines the JWT `sub` claim as a case-sensitive StringOrURI that is locally unique in the issuer or globally unique. OpenID Connect Core 1.0 likewise defines the Subject Identifier as locally unique and never reassigned within the issuer, with public and pairwise subject types. CalendarWeave therefore treats an external principal as the pair `(issuer, subject)` and does not invent a global-subject assumption.

CalendarWeave does not reinterpret issuer or subject syntax. `ScopedIdentity` retains both values as opaque strings and applies only defensive service bounds: non-empty, finite byte length and no control characters. Ordinary Unicode and spaces are accepted. Those bounds protect this API surface from unbounded/control-bearing input; they are not a replacement for the issuer's identity grammar.

## Exact implementation boundary

PR #6 introduces the following candidate surfaces:

- `src/admission.rs::ScopedIdentity`: issuer + subject + admitted `TenantId` value object;
- `src/admission.rs::CalendarAuthorizationPort`: external allow/deny/unavailable decision port over typed `CalendarAction` values;
- `src/admission.rs::AuthorizedCalendarService`: fail-closed application ACL that authorizes before delegating to `CalendarPort`;
- `src/lib.rs::CalendarError::{Unauthorized, AuthorizationUnavailable}`: separates a completed deny decision from inability to establish authorization.

The wrapper intentionally has no public escape hatch to the inner calendar adapter. An event create is authorized before the supplied iCalendar text is parsed. Successful calls continue to pass only `identity.tenant_id()` into the Calendar Resource Core, preserving the existing cross-tenant/absent-resource indistinguishability in the underlying port.

## Test-first evidence

The branch preserves a RED-first executable contract in `tests/authorization_admission.rs`, followed by production implementation. The current contract covers:

1. denied create-event authorization wins over malformed-calendar parsing;
2. authorization dependency unavailability fails closed distinctly;
3. bounded opaque issuer/subject handling, including ordinary spaces and rejection of empty, over-limit and control-bearing values;
4. the issuer+subject joint-principal invariant;
5. cross-tenant get/list isolation through the admitted service; and
6. successful create/list/get/conditional-update behavior with revision and ETag preservation.

Hosted exact-head checks remain authoritative for compilation, lint, rustdoc and quantitative coverage. A queued, cancelled, stale-head or predecessor-head run is not passing evidence.

## Keyverse and ecosystem boundary

Keyverse remains the ContextualWisdomLab identity/federation and authorization control-plane owner. CalendarWeave does not parse/verify bearer tokens, store passwords, mint sessions, copy Keyverse policy tables, or consume mutable Keyverse implementation bytes. A future infrastructure adapter may translate a successfully verified Keyverse principal and authorization decision into these admission types through a versioned contract.

Naruon, LineageWeave and `saju-caldav` must likewise consume CalendarWeave through released/versioned ports or ACLs after parity evidence; they do not obtain CalendarWeave table authority from this candidate.

## Security evidence boundary

This candidate establishes only an in-process authorization admission contract. It does **not** yet prove:

- HTTP/service authentication or token validation;
- authorization-decision persistence or durable audit retention;
- service-to-service credential rotation;
- rate limiting, network admission or abuse resistance;
- production Keyverse integration;
- operated backup/recovery;
- CSAP certification or SOC 2 attestation; or
- a released consumer migration path.

Those remain explicit commercialization gaps and must not be inferred from source-level admission tests.

## References

Jones, M., Bradley, J., & Sakimura, N. (2015). *JSON Web Token (JWT)* (RFC 7519). RFC Editor. https://doi.org/10.17487/RFC7519

OpenID Foundation. (2014). *OpenID Connect Core 1.0 incorporating errata set 2*. https://openid.net/specs/openid-connect-core-1_0.html
