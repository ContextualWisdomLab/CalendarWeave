# Run the CalendarWeave local operator

This candidate contains an executable Rust operator and the same Calendar Port
used by package consumers. It is not a CalDAV server, a public API, or a released
Keyverse integration. Do not expose the CLI through a web shell or treat a tenant
argument as authentication. Only a trusted OS operator may invoke it.

## Prerequisites

- Rust 1.97.1 or newer, with Cargo.
- An operator-owned, disposable PostgreSQL 18 database for evaluation.
- A UNIX socket or an explicit loopback IP. DNS names (including `localhost`),
  implicit default hosts, `hostaddr` overrides, and remote IPs are rejected.
- A trusted authorization decision establishing the tenant scope before invocation.

The retained PostgreSQL adapter has no TLS transport. UNIX socket access relies
on OS permissions. Loopback TCP is appropriate only inside a separately trusted
same-host boundary; it is not evidence of encrypted remote database support.

## Build and initialize

Run these commands from this candidate checkout, not the seed `main` branch:

```sh
cargo build --release --locked
./target/release/calendarweave --help

# Use your existing local PostgreSQL operator and an owned evaluation database.
# Replace the socket directory and database below with your actual local values.
export CALENDARWEAVE_DATABASE_URL='host=/actual/postgres/socket dbname=calendarweave_eval'
./target/release/calendarweave init
```

Only `init` applies the idempotent migration. All other commands require an
already-initialized store and never automatically alter the schema. Do not point
`init` at an unrelated or production database. The command writes CalendarWeave
collections, events, and revision tables in the selected database.

If password authentication is needed, inject the connection setting through your
approved secret-management boundary. Do not put it in command arguments, commit
it, use shell tracing, or paste it into logs. The CLI's bounded errors do not echo
the connection string; that does not make process environment inspection safe.

## Create, list, and retrieve an event

The CLI expects UTF-8 RFC 5545 content with CRLF line endings and a final CRLF.
The following sample creates those exact line endings without a provider login:

```sh
printf '%s\r\n' \
  'BEGIN:VCALENDAR' \
  'VERSION:2.0' \
  'PRODID:-//CalendarWeave//Operator Quickstart//EN' \
  'BEGIN:VEVENT' \
  'UID:operator-quickstart@example.test' \
  'DTSTAMP:20261003T000000Z' \
  'DTSTART;TZID=Asia/Seoul:20261004T090000' \
  'DURATION:PT1H' \
  'SUMMARY:CalendarWeave operator evaluation' \
  'CLASS:PRIVATE' \
  'END:VEVENT' \
  'END:VCALENDAR' > event.ics

collection=$(./target/release/calendarweave create-collection eval-tenant 'Evaluation calendar')
./target/release/calendarweave create-event eval-tenant "$collection" event.ics
./target/release/calendarweave list-events eval-tenant "$collection"
```

Creation returns two tab-separated fields: the opaque event reference and the
strong ETag, for example `evt_<opaque-id>` and `"evt_<opaque-id>:1"`. Copy the actual
returned values into the following variables; these placeholders are not data:

```sh
event='ACTUAL_RETURNED_EVENT_REFERENCE'
etag='ACTUAL_RETURNED_ETAG_INCLUDING_DOUBLE_QUOTES'
./target/release/calendarweave get-event eval-tenant "$collection" "$event" > stored.ics
cmp event.ics stored.ics
```

`get-event` writes the original calendar bytes, not a metadata wrapper. A second
identical create with the same UID returns the same reference and ETag. Different
content under an existing UID is rejected instead of silently overwritten.

## Conditional update

Edit `event.ics` while preserving its UID, supported profile and CRLF format.
Then present the exact current ETag:

```sh
./target/release/calendarweave update-event eval-tenant "$collection" "$event" "$etag" event.ics
```

A changed payload increments the revision and returns a new ETag. An identical
payload with the current ETag is a no-op. An old ETag fails with `StaleRevision`.
The CLI does not accept wildcard replacement. Another tenant sees `NotFound` for
this collection or event; the operator must still establish authority to use the
tenant string in the first place.

## Failure and recovery behavior

Ordinary application failures exit nonzero with a bounded code on stderr and no
success payload on stdout. Unsupported recurrence, floating time, `VTIMEZONE`,
unrecognized event properties, malformed content, and conflicting revisions
fail closed.

Output failures also exit nonzero, but can emit partial stdout and no stderr.
A stdout failure can occur after a write committed. Do not blindly repeat
collection creation after an uncertain result: it has no caller-supplied
idempotency key. Inspect stored state through the trusted boundary. Event create
is idempotent only for the same collection, UID and exact content. Updates use
strong ETags, not an automatic retry loop.

Logical backup and restore are separate commands under `ops/postgres/`. Read
ADR-0006 before using them. SHA-256 integrity and a successful logical restore do
not prove encryption, retained remote backups, PITR, HA, or measured RPO/RTO.

## Verify the candidate

Use only a deliberately selected disposable database. The CLI tests fail rather
than silently skip when `CALENDARWEAVE_TEST_DATABASE_URL` is missing:

```sh
export CALENDARWEAVE_TEST_DATABASE_URL="$CALENDARWEAVE_DATABASE_URL"
cargo fmt --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features
RUSTDOCFLAGS='-D warnings' cargo doc --locked --all-features --no-deps
cargo package --locked
```

`cargo package` verifies an installable source archive. It does not publish to
crates.io, create a GitHub release, prove production security, or authorize a
consumer migration. Public HTTP/CalDAV delivery still requires the exact
Keyverse resource authorization integration described in the product gap record.
