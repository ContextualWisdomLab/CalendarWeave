# RFC 5545 CLASS privacy baseline

## Question

What privacy semantics can CalendarWeave expose from RFC 5545 `CLASS` without turning descriptive calendar metadata into local authorization policy or adding a redundant persistence model?

## Normative evidence

RFC 5545 section 3.8.1.3 defines `CLASS` for `VEVENT`, `VTODO`, and `VJOURNAL`. The property is optional and single-valued for a VEVENT. Its standard values are `PUBLIC`, `PRIVATE`, and `CONFIDENTIAL`; omission defaults to `PUBLIC`. The grammar also permits registered IANA tokens and experimental `X-` names, and applications must treat unrecognized such values the same as `PRIVATE`.

RFC 5545 explicitly says access classification cannot by itself serve as an enforcement statement for a receiving calendar system. It captures the calendar owner's intended access scope and must be combined with authentication, authorization, access rights, roles, and other security controls. CalendarWeave therefore keeps `CLASS` in Calendar Resource Core as descriptive intent while ADR-0005's Authorization Admission remains the enforcement boundary.

RFC 5545 section 3.1 makes property names, property parameters, enumerated property values, and parameter values case-insensitive. Accordingly, CalendarWeave accepts case variants of the three standard `CLASS` values. The `CLASS` grammar permits IANA and non-standard parameters, so the bounded profile preserves their interoperability rather than rejecting a standards-valid event merely because an extension parameter is present.

## Implementation mapping

PR #9 derives a typed `EventClass` from the existing validated immutable `icalendar_payload` instead of adding a second relational field. This keeps the 3NF calendar revision model unchanged and avoids a synchronization invariant between a classification column and the canonical payload.

The parser contract is deliberately fail-private for valid but unknown token values: a new IANA or experimental classification cannot silently become public. Duplicate, empty, or non-token values fail as malformed input. The typed accessor also revalidates singleton/component structure so a manually forged projection cannot manufacture a trusted classification from malformed raw content.

This slice does **not** implement CalDAV ACL semantics, provider sharing controls, purpose/retention policy, encryption policy, authorization decisions, or audit evidence. Those require separate product and operational controls.

## Test traceability

- RED contract: `b91602811a231c726ab5fbc5a2e0a1af894e9346`.
- Standards-corrected RED contract: `18d3ea264d2f0c3bfeea10e5af6fa01ff4bbe706`.
- Standards-correct production implementation: `707fcecdb45a273028b2d7966888c3a507d268d5`.
- Candidate tests: `tests/rfc5545_classification.rs`.

Hosted exact-head checks and independent semantic/security review remain required; this research note is not runtime verification.

## Reference

Desruisseaux, B. (Ed.). (2009). *Internet calendaring and scheduling core object specification (iCalendar)* (RFC 5545). RFC Editor. https://doi.org/10.17487/RFC5545