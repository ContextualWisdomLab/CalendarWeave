# Try the loopback HTTP bootstrap

## What this candidate does

This is a real standalone TCP listener with a deliberately small HTTP request
profile. It is a diagnostic prototype, **not an authenticated service**, a TLS
endpoint, a general-purpose HTTP server, or a completed CalDAV implementation.
The route below is a new transport contract, not a CalDAV resource-discovery API.

The only admitted request shape is:

```text
GET /calendars/{collection}/{event} HTTP/1.1
Host: localhost
```

Collection and event references must be nonempty, at most 128 ASCII bytes each,
and contain only letters, digits, underscores or hyphens. They are resource
references, never tenant authority. Every otherwise-supported GET currently
returns `503 Service Unavailable` with `AuthorizationUnavailable` and no calendar
payload or ETag. An Authorization header does not grant access.

## Build and start

Use the installed Rust 1.97.1 toolchain and the repository's unchanged lockfile:

```sh
cargo +1.97.1 build --locked --offline --jobs 1
./target/debug/calendarweave serve 127.0.0.1:8080
```

No database URL, token, operator credential or `.env` file is needed for `serve`.
The command dispatches before the local operator's database environment lookup,
connection and migration. Only the existing explicit `init` command migrates.
The HTTP path passes a lazy PostgreSQL factory to the unavailable-runtime gate;
that gate does not invoke the factory, even for trusted in-process identity.

Use an explicit loopback IP. DNS names, wildcard addresses, remote addresses and
IPv4-mapped IPv6 addresses are rejected. IPv6 loopback uses `[::1]:8080`.
Port zero asks the kernel to choose a free port. The process prints the actual
address as `LISTENING 127.0.0.1:PORT`; read this publication instead of guessing
or retrying guessed ports.

`--once` accepts one connection and then exits. This is a public diagnostic
lifecycle option, not a secret authentication bypass. Without it the listener
accepts connections sequentially until the operator stops it. It does not spawn
worker processes. A socket/publication error can stop the prototype.

## Exercise the built executable

The repository's bounded Python standard-library probe reads the actual address,
sends one request, captures the response, waits its exact child and closes its
pipes. It sets only an invalid synthetic database URL. Python 3 is required for
this probe and the corresponding integration test, not for the Rust service.

```sh
python3 tests/support/http_wire_probe.py ./target/debug/calendarweave
python3 tests/support/http_wire_probe.py ./target/debug/calendarweave idle
```

The first probe expects 503. The second opens an idle connection and expects 408
when the two-second total header deadline expires. These are denial and transport
observations, not evidence of a verified identity producer or authorized read.

## Fixture custody and diagnostic command policy

The probe owns only its one immediate Rust child. The Rust server does not
spawn descendants. Immediately after the cooperative Popen constructor returns,
its exact handle is retained before PID receipts and selector construction.
One absolute fifteen-second cutoff covers body capture and cleanup waits.
Signal, exact wait and each resource close are attempted independently. The
original body exception remains primary; cleanup exceptions are retained as
objects without formatting. A close that raises, including after its effect,
is always NONPASS. It is never retried merely because a closed flag looks true.

If exact wait is unknown, closing pipes is not permission to exit. The CLI
keeps that same Python owner alive and refuses another launch. Its existing
stdin accepts `status` and `reconcile`; `exit`, interruption and EOF do not
release custody. EOF idles safely with the owner alive. Only an acknowledged
exact integer wait reconciles the child. The original failed probe still
returns nonzero after reconciliation. Do not kill this interpreter to clear
an unresolved wait. The Rust integration caller deliberately waits for it;
no universal bounded-cleanup guarantee is claimed.

The historical scratch `run.py` is retired from resumed command execution.
Build and test commands run directly as terminal-tracked Cargo jobs. Cargo
remains the actual parent of its compiler and test processes and exits
normally. An outer observation timeout must leave the tracked job alive;
transfer it to the parent agent before ending a worker if it is still running.
Never treat killing a Cargo or probe leader as settlement of descendants.

The actor-free distributed tests inject only constructor-returned fake Popen,
selector and socket handles into the actual publication/request caller. Native
spawn, fork, signal and socket audit events are denied. They prove caller
control flow, not a reproduced leak or native failure settlement. Native
constructor return gaps, arbitrary FFI, interpreter crash and OS death are
outside this finite contract. The probe child environment is an explicit
minimal allowlist, not a copy of provider or database credentials.

```sh
python3 -B -W error tests/support/test_http_wire_custody.py -v
cargo +1.97.1 test --locked --offline --jobs 1 --test http_transport
```

## Deliberate HTTP limits

The request line is limited to 2,048 bytes and the complete header block to 8,192
bytes, including framing. Header reading has one two-second total deadline, not a
fresh two seconds per byte. Writes have a two-second socket timeout. Responses
have an explicit length, `Cache-Control: no-store`, and `Connection: close`.
Only HTTP/1.1 with CRLF framing is supported. `Host` is required exactly once.
Allowed headers are Host, Authorization, User-Agent, Accept, Connection: close,
and Content-Length: 0; duplicates and other headers fail closed.

Body framing, nonzero Content-Length, Transfer-Encoding, upgrades, folded headers,
encoded paths, queries and caller tenant headers are not supported. Unsupported
request profiles return 400; other methods return 405. The connection always
closes after one response. No keep-alive, pipelined request processing, HTTP/2,
chunking, TLS, concurrent serving or production denial-of-service protection is
claimed. Extra queued bytes are never processed as another request.

## What must happen next

A real Keyverse RP verification adapter still needs an agreed runtime issuer,
audience, token/trust configuration and verified producer API. String validation
by `ExternalIdentity::parse` is not authentication. A separate resource-aware
policy decision must bind verified issuer/subject, read action and exact resource
to its authorized tenant before any factory is acquired. Software/menu allow
must not become calendar permission. No local IdP or synthetic token allow is
introduced here, and missing dependencies must not be mislabeled as invalid-token
401 responses.

Authenticated PostgreSQL CRUD, full CalDAV, real authorized positives, service
hardening, deployment, consumer parity, release and independent approval remain
open. The existing exact statement/branch coverage gate is not waived; successful
unit tests or native response probes do not measure that gate.
