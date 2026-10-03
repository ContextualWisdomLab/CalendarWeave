# ADR-0009: Execute the retained calendar port through a trusted local operator

- Status: Accepted candidate; not released or production-service evidence
- Date: 2026-10-03
- Related: Issue #2, ADR-0002 through ADR-0008

## Context

The default branch is a README seed while PRs #1 and #6–#9 carry executable
calendar, persistence, admission and recovery behavior. A user asked for working
software, not another description of candidate PRs. The existing domain and
PostgreSQL implementations already support a complete local calendar path but
had no installed executable entry point.

The organization has real identity and edge implementation work. Keyverse main
at `7d9151cd2da260e118020c938c7358e2ee75d541` defines the RP verification contract;
`saju-caldav` main at `fa72a2c8b988abeb7efacc886cbb3fd8849314af` implements a JWT
verifier. These facts do not complete CalendarWeave authorization. Keyverse PR103
at `5ac33256229321e9fccbb14a460c7d6de984444a` returns software/menu decisions, but
not an issuer/action/calendar-resource-bound authorized tenant. Its decision
routes require operator authentication, not a CalendarWeave-scoped runtime
credential. No such route is invented here, and no operator credential is
embedded in CalendarWeave.

## Decision

1. Preserve the complete CalendarWeave stack and package preflight in a normal
   integration branch without rewriting or mutating predecessor branches.
2. Add `calendarweave` as a local OS-operator executable over the existing
   `PostgresCalendarService`. Keep package consumption through `CalendarPort`
   and external admission through `AuthorizedCalendarService` unchanged.
3. Publish exact commands for explicit schema initialization, collection create,
   event create/list/get and strong-ETag update. Do not create a web listener,
   accept trusted identity headers, or label the CLI an authenticated service.
4. Obtain connection settings only from `CALENDARWEAVE_DATABASE_URL`; emit bounded
   domain/error codes rather than provider diagnostics or calendar content.
5. The retained storage adapter is non-TLS. Reject implicit/DNS/remote hosts and
   hostaddr overrides; allow absolute UNIX sockets or explicit loopback IPs only.
   This is a target boundary, not a claim that loopback is encrypted.
6. Apply migrations only through explicit `init`, never as a hidden side effect
   of ordinary reads or writes.
7. Exercise the actual binary in separate processes against disposable real
   PostgreSQL. Require byte-preserving readback, restart-stable references,
   idempotent event creation, tenant isolation and stale-write rejection.
8. Keep statement/branch gates unchanged and exercise output/storage failures.
   A failure to emit output after a committed write is an uncertain caller
   result, not permission to blindly repeat a non-idempotent operation.

## Non-goals and next dependency

This is an intermediate usable artifact, not a reduction of CalendarWeave's
standalone product objective. HTTP/CalDAV delivery, actual Keyverse token
acceptance, purpose-bound authorization audit, production deployment, operated
recovery, versioned release, and consumer parity remain open.

Keyverse must own the missing runtime resource decision contract. It must bind
verified issuer/subject, the exact CalendarWeave action and collection/event
references to an authorized tenant, authenticate a least-privilege runtime
caller, and fail closed when membership or resource binding cannot be proved.
CalendarWeave must validate that actual contract before admitting network callers;
it must not infer tenant authority from an ordinary software/menu allow.

## Verification

`tests/operator_cli.rs` executes the shipped binary rather than mocked API
responses. Retained core, admission, PostgreSQL and standards tests remain active.
`tests/support/operator_errors.rs` checks every bounded error code without
expanding the production source coverage denominator with test-only code.
The source archive must include this support module and the migration assets.
See the local operator quickstart for exact commands and evidence boundaries.
