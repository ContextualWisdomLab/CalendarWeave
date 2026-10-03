# ADR-0008: RFC 5545 CLASS privacy profile

- Status: Accepted candidate
- Date: 2026-09-02
- Owner: Calendar Resource Core
- Related: issue #2, PR #9, ADR-0002, ADR-0005, ADR-0007

## Context

CalendarWeave must preserve generic calendar privacy intent before `saju-caldav` or other consumers can migrate to its versioned calendar-resource boundary. RFC 5545 `CLASS` is the interoperable property for this intent, but it is not an authorization mechanism. Treating `CLASS:PRIVATE` as access control would collapse Calendar Resource Core metadata into Authorization Admission and would be both architecturally wrong and contrary to RFC 5545.

The existing candidate stores the complete validated iCalendar payload in each immutable revision. Adding a second relational classification column would duplicate source-of-truth data before any query requirement justifies denormalization.

## Decision

CalendarWeave's bounded VEVENT profile accepts one optional `CLASS` property.

1. An omitted `CLASS` projects as `PUBLIC`.
2. `PUBLIC`, `PRIVATE`, and `CONFIDENTIAL` are matched case-insensitively, consistent with the general RFC 5545 rule for enumerated property values.
3. Syntactically valid unrecognized registered or experimental token values project conservatively as `PRIVATE`, as RFC 5545 requires.
4. IANA and non-standard parameters on `CLASS` remain parseable and do not change the classification projection. Unknown parameters are not converted into local authorization policy.
5. Duplicate, empty, or syntactically invalid class values fail closed as malformed calendar input.
6. `EventClass` is a read projection derived from the validated immutable iCalendar payload. The persistence schema remains unchanged.
7. `CLASS` expresses calendar-owner access intent only. Authentication, tenant derivation, resource authorization, disclosure enforcement, and audit remain the responsibility of Authorization Admission and its external policy authority.

## Consequences

The candidate gains a bounded consumer-parity privacy contract without introducing a parallel persistence model or product-specific policy. Consumers can distinguish standard privacy intent through a typed API while unknown extensions are not accidentally treated as public.

This decision does not establish retention policy, export controls, provider-specific sharing semantics, CalDAV ACLs, service authentication, or SOC 2/CSAP compliance evidence. Those remain explicit commercialization gaps.

## Verification

PR #9 carries RED-first fixtures for omitted and standard values, case-insensitive enumeration, unknown registered/experimental token values, extension parameters, duplicate/invalid inputs, and a forged read projection. Exact-head repository checks and independent semantic/security review are required before ordinary merge progression.

## References

RFC 5545 sections 3.1 and 3.8.1.3 are the normative basis. See `docs/doctoring/rfc5545-class-privacy-baseline.md` for research traceability.