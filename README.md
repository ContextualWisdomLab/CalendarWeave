# CalendarWeave

Standalone CalDAV/iCalendar PIMS. Open this repo when you need calendar as its own product. LineageWeave, naruon, and Outlook consume it fail-closed. Do not put calendar chrome in naruon mail or LineageWeave #74.

## Customer next action

1. Point your CalDAV client at the CalendarWeave endpoint for the tenant you already use.
2. Create or accept an event in that client. Confirm it appears here, not in naruon mail or a LineageWeave board.
3. If sync fails, stop and check the CalendarWeave connection. Do not copy events into mail or #74.

## What this is not

Not mail. Not tasks. Not a local IdP. Not GRC policy. Not LineageWeave #74.

## Current executable candidate

The branch stacked above architecture PR #1 contains a Rust v0.1 application
port for tenant-scoped collection creation and strict RFC 5545 VEVENT
create/list/get. It accepts UTC or all-day intervals and returns opaque event
references with revision/ETag evidence. Unsupported calendar capabilities fail
closed.

This is an in-memory conformance adapter, not protected-main, durable storage,
CalDAV, a deployed service, or a released migration contract. See ADR 0002.

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo +nightly llvm-cov --all-features --branch \
  --fail-under-lines 100 --json --output-path coverage.json
jq -e '.data[0].totals.branches.percent == 100' coverage.json
```
