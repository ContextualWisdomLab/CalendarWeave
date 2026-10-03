# RFC 5545 event metadata traceability

## Normative source

The RFC Editor HTML was inspected on 2026-10-03. Section 3.8.1.5 defines
`DESCRIPTION` as TEXT, permits `LANGUAGE`, `ALTREP`, and extension parameters,
and permits only one instance within VEVENT. Section 3.8.2.7 defines single
VEVENT `TRANSP`, the `OPAQUE`/`TRANSPARENT` enumeration, and default `OPAQUE`.[1]
The general enumeration case-insensitivity rule is in section 3.1.[1]
Section 3.1 was retrieved again for CW-METADATA-001 on 2026-10-04 (KST).
It defines CRLF-delimited content lines and folds using CRLF followed by SPACE
or HTAB. Bare CR or LF is malformed framing, not an unsupported property
parameter. The shared admission guard rejects either before property parsing,
without normalizing retained payload bytes.[1]
Section 3.1 was independently inspected again for CW-METADATA-002 on
2026-10-04 (KST). Its content-line grammar requires a nonempty name at the
start of each unfolded logical line. The shared guard rejects empty logical
lines and leading SPACE/HTAB before parsing; only the final CRLF terminator
is excluded. Singleton names are compared ASCII case-insensitively. Valid
multi-indented folds remain part of the value, not new properties.[1]

## Candidate interpretation

ADR-0010 adds parameter-free admission and original-byte preservation, not full
RFC 5545 conformance. Its rejection of all parameters is a versioned support
boundary. In particular, RFC-permitted `LANGUAGE` and `ALTREP` must not be
reported as intrinsically invalid iCalendar.

The existing parser accepts the tested DESCRIPTION escaping/folding/Unicode
examples. There is no new complete TEXT validator or text-rendering capability.
The TRANSP validator checks standard values but does not turn them into a
free/busy query, availability calculation, or consumer commitment policy.
Omission remains byte-preserved, rather than inserting the RFC default.

## Evidence and ownership

Twelve fixtures are exact outputs of the existing consumer serializer over
synthetic inputs. Their provenance file identifies original hashes and collection
method. They prove this metadata admission slice, not all consumer operations,
real identity verification, deployed authorization, or migration readiness.
CalendarWeave owns generic property admission; consumers own their explanation
content and business scheduling meaning. The canonical revision remains the
source of truth; no metadata columns or dependency changes are introduced.

## References (APA 7th)

Desruisseaux, B. (Ed.). (2009). *Internet calendaring and scheduling core object
specification (iCalendar)* (RFC 5545). RFC Editor.
https://doi.org/10.17487/RFC5545

## Sources

[1] RFC 5545 iCalendar — https://www.rfc-editor.org/rfc/rfc5545.html
