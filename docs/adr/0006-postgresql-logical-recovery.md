# ADR-0006: Prove logical PostgreSQL recovery before release

**Status:** Accepted  
**Date:** 2026-09-02

## Context

CalendarWeave's PostgreSQL adapter preserves collection identity, item-level UID
idempotency, append-only event revisions and conditional current-revision
integrity, but persistence is not commercially operable until those invariants
can be recovered from an independently stored artifact. A process restart is
not backup evidence, and a successful `pg_dump` command is not restore evidence.

PostgreSQL 18 documents `pg_dump` archive formats as portable inputs to
`pg_restore`; custom format is a flexible archive format and `pg_restore
--single-transaction` makes a restore all-or-nothing when the target is small
enough for that transaction boundary. PostgreSQL also warns that logical dumps
are generally not the complete answer for regular production backup: sustained
production recovery, low recovery-point objectives, and point-in-time recovery
require an operator-designed physical/WAL strategy.

## Decision

1. CalendarWeave ships a bounded logical-backup script using PostgreSQL custom
   format with ownership and privilege restoration disabled. It is a recovery
   artifact for the CalendarWeave database contract, not a cluster-wide backup.
2. Backup files and their SHA-256 evidence are created with owner-only
   permissions. A failed dump is never published as a successful artifact.
3. Restore rejects a missing, symbolic-link, malformed-checksum or
   checksum-mismatched artifact **before** invoking `pg_restore`.
4. A verified restore uses one PostgreSQL transaction so a restore error cannot
   leave a partially applied CalendarWeave schema/data set in the target used by
   this bounded workflow.
5. The executable recovery drill restores into a separate empty database and
   proves both data and relational invariants: collection/event/current revision
   identity, collection+UID uniqueness, and the current-revision foreign key.
6. RPO, RTO, retention, physical base backups, WAL archiving/PITR, encryption at
   rest, remote object-store durability, key management, HA and failover remain
   deployment decisions. CalendarWeave must measure and document those before a
   buyer-facing availability/recovery claim; this ADR does not invent a
   rule-of-thumb target.

## Transaction and security boundary

The backup operation does not participate in application write transactions.
`pg_dump` obtains a PostgreSQL-consistent logical snapshot. Restore owns one
recovery transaction in a dedicated target database; it does not widen normal
item-level Calendar Resource Core mutation boundaries. The scripts accept
operator-supplied connection and artifact locations and do not copy credentials
into CalendarWeave persistence or source control.

Calendar content can contain PII required for calendar work, so backup evidence
is protected rather than masked. Test and documentation fixtures use anonymous
`.example.test` identities only.

## Consequences

This slice converts "PostgreSQL exists" into executable logical recovery
evidence while preserving the existing 3NF model and item-level UPSERT/ETag
contracts. It deliberately leaves production backup cadence and PITR as open
commercialization gates because PostgreSQL's own documentation distinguishes
logical dumps from continuous WAL-based recovery.

## Verification

`tests/postgres_recovery_drill.sh` is the acceptance contract. It was committed
before `ops/postgres/backup_calendarweave.sh` and
`ops/postgres/restore_calendarweave.sh`, and the repository `Tests` workflow
runs it against the same pinned PostgreSQL 18.4 service used by persistence
verification. Hosted exact-head execution remains authoritative; queued or
predecessor-head runs are non-passing.

Research-to-source interpretation and APA 7th references are recorded in
`docs/doctoring/postgresql-logical-recovery-baseline.md`.
