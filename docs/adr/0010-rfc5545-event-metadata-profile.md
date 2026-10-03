# ADR-0010: Bounded RFC 5545 event metadata profile

- Status: Source candidate; not merged or released
- Date: 2026-10-03
- Owner: Calendar Resource Core
- Related: ADR-0002, ADR-0005, ADR-0008, ADR-0009
- Contract: Calendar Port v0.1 metadata admission increment

## Context

Exact synthetic outputs from the existing `saju-caldav` serializer contain
`DESCRIPTION` and `TRANSP`. Previously the bounded core rejected those properties.
Generic metadata admission belongs in CalendarWeave; saju explanation content and
publication intent remain consumer-owned. No consumer compatibility path is removed.

## Decision

The source candidate admits at most one parameter-free `DESCRIPTION` and one
parameter-free `TRANSP` within an otherwise supported VEVENT.

- `DESCRIPTION` is preserved in the original iCalendar bytes, including empty,
  escaped, Unicode, and folded examples accepted by the retained parser. This
  increment does not introduce a complete independent RFC TEXT grammar validator,
  text decoder, HTML renderer, or typed description accessor.
- `TRANSP` accepts `OPAQUE` or `TRANSPARENT`, ASCII case-insensitively. Empty,
  unknown, whitespace-padded, or combined values fail `MalformedCalendar`.
- Omitted `TRANSP` remains omitted in the stored bytes. RFC 5545 defines its
  default as `OPAQUE`; this increment does not calculate free/busy, project busy
  policy, or add a typed transparency accessor.
- Bare CR or LF fails `MalformedCalendar` before property parsing. Valid CRLF
  content lines and CRLF followed by SPACE or HTAB folds remain supported;
  admission does not normalize original bytes.
- Empty logical content lines and logical lines starting with SPACE or HTAB
  fail `MalformedCalendar` before the dependency can skip their whitespace.
  The final CRLF terminator is not an extra empty line. Valid folds remove only
  one SPACE or HTAB; additional indentation inside DESCRIPTION remains text.
- Duplicate properties fail `MalformedCalendar` before dependency overwrite.
  Singleton counting uses the same validated unfolded content lines and compares
  property names ASCII case-insensitively without changing retained bytes.
- Every parameter on either property returns `UnsupportedCapability`, including
  `LANGUAGE`, `ALTREP`, `VALUE`, IANA, and experimental parameters. This is an
  intentionally narrower profile, not a claim that RFC-permitted parameters are
  invalid iCalendar. Parameter support requires another explicit contract slice.
- Unknown event properties, recurrence, floating time, and `VTIMEZONE` remain
  outside the retained profile. Existing interval, status, classification, UID,
  strong-ETag, tenant isolation, and authorization ordering remain unchanged.

Admission uses the shared validator. Original payloads remain the sole metadata
source of truth; no storage schema, dependencies, or parallel policy model change.
Metadata cannot grant access or override denied/unavailable authorization.

## Verification and limits

`tests/rfc5545_event_metadata.rs` carries twelve exact serializer-output fixtures
with provenance, preservation and conditional-revision assertions, plus description,
transparency, duplicate, parameter, unsupported-profile, and authorization controls.
The authorization decisions are synthetic inputs to the real admission wrapper;
they are not a real issuer, token verifier, or deployed policy decision point.
The original nineteen-test native RED/GREEN evidence is retained outside the
package. Additional transparency positives exercise already-implemented behavior;
no new production behavior follows those characterization tests.

Normal Cargo, PostgreSQL-backed full-suite, coverage, and extracted-package results
must be reported from actual receipts. This decision itself proves none of those
gates. Independent exact-byte review, hosted CI, release, authenticated standalone
service, CalDAV collection/DELETE/unconditional PUT, full consumer parity, and
consumer cutover remain open.

## References

See `docs/doctoring/rfc5545-event-metadata-baseline.md` for the retrieved normative
sections, the bounded-profile distinction, and APA 7th source attribution.
