# PostgreSQL logical recovery research baseline

**Evidence date:** 2026-09-02  
**Implementation lane:** `feat/postgres-recovery-v1`

## Question

What is the smallest standards/vendor-backed recovery contract that proves the
current CalendarWeave PostgreSQL schema can be reconstructed without claiming a
production disaster-recovery posture that has not been operated?

## Authoritative evidence and interpretation

PostgreSQL 18 documents `pg_dump` as a consistent logical export mechanism and
states that custom-format archives are intended for `pg_restore`, are portable
across architectures, and support selective/reordered restoration. CalendarWeave
therefore uses custom format for the first executable logical-recovery slice.
This is intentionally database-scoped rather than a claim to back up PostgreSQL
roles, tablespaces, cluster configuration or every operational dependency.

PostgreSQL 18 documents `pg_restore --single-transaction` as an all-or-nothing
restore mode and notes that it implies exit-on-error. CalendarWeave uses that
mode for this bounded schema so a restore failure does not leave a partially
applied target. PostgreSQL also documents that an omitted archive filename makes
`pg_restore` read standard input, which permits the recovery script to verify the
artifact digest before the archive stream reaches `pg_restore`.

PostgreSQL's backup guidance distinguishes logical dumps from continuous
archiving and point-in-time recovery. WAL archiving plus a base backup is the
mechanism that supports recovery to a chosen point in time and materially lower
recovery-point loss. CalendarWeave therefore does **not** convert successful
logical restore into an RPO/RTO, PITR, HA, failover or disaster-recovery claim.
Those require deployment-specific storage, WAL, monitoring, retention, key
management and measured recovery exercises.

## Source-to-code traceability

| Evidence | CalendarWeave contract | Exact source |
| --- | --- | --- |
| Custom-format dumps are designed for `pg_restore` | Produce `--format=custom`; omit source ownership and ACL restoration | `ops/postgres/backup_calendarweave.sh` |
| Single-transaction restore is all-or-nothing | Restore only after digest verification with `--single-transaction` | `ops/postgres/restore_calendarweave.sh` |
| Restore is useful only if product invariants survive | Assert collection/event/revision values, collection+UID uniqueness and current-revision FK after restore | `tests/postgres_recovery_drill.sh` |
| Backup content needs protection | `umask 077`, mode `0600`, reject symlink evidence and verify SHA-256 before restore | backup/restore scripts and recovery drill |
| Logical dump is not PITR | Keep RPO/RTO/WAL/PITR/HA open in commercialization baseline | ADR-0006 and `docs/product-technical-gap-baseline.md` |

The SHA-256 sidecar is an application-level integrity check for accidental or
unapproved artifact substitution in this bounded workflow; it is not a digital
signature, encryption mechanism or provenance system. A production backup store
still needs authenticated access, encryption/key management, retention and
independent durability controls.

## APA 7th references

PostgreSQL Global Development Group. (2026). *PostgreSQL 18 documentation:
pg_dump*. https://www.postgresql.org/docs/18/app-pgdump.html

PostgreSQL Global Development Group. (2026). *PostgreSQL 18 documentation:
pg_restore*. https://www.postgresql.org/docs/18/app-pgrestore.html

PostgreSQL Global Development Group. (2026). *PostgreSQL 18 documentation:
Continuous archiving and point-in-time recovery (PITR)*.
https://www.postgresql.org/docs/18/continuous-archiving.html
