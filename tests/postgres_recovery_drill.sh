#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
backup_script="$repo_root/ops/postgres/backup_calendarweave.sh"
restore_script="$repo_root/ops/postgres/restore_calendarweave.sh"

[[ -f "$backup_script" ]] || { echo "missing production backup contract: $backup_script" >&2; exit 1; }
[[ -f "$restore_script" ]] || { echo "missing production restore contract: $restore_script" >&2; exit 1; }

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

# CI runs the drill against the pinned PostgreSQL 18.4 service container. A
# developer may instead point it at a disposable local cluster that they own by
# setting CALENDARWEAVE_RECOVERY_LOCAL_SOCKET_DIR (UNIX socket directory),
# CALENDARWEAVE_RECOVERY_LOCAL_PORT and CALENDARWEAVE_RECOVERY_LOCAL_PG_BIN_DIR;
# the calendarweave_test database must already exist there.
if [[ -n "${CALENDARWEAVE_RECOVERY_LOCAL_SOCKET_DIR:-}" ]]; then
    local_pg_bin_dir="${CALENDARWEAVE_RECOVERY_LOCAL_PG_BIN_DIR:?CALENDARWEAVE_RECOVERY_LOCAL_PG_BIN_DIR is required in local mode}"
    local_port="${CALENDARWEAVE_RECOVERY_LOCAL_PORT:-5432}"
    database_url() {
        printf 'postgresql://postgres@/%s?host=%s&port=%s' \
            "$1" "$CALENDARWEAVE_RECOVERY_LOCAL_SOCKET_DIR" "$local_port"
    }
    run_psql() { "$local_pg_bin_dir/psql" "$@"; }
    cat >"$tmp_dir/pg_dump" <<EOF
#!/usr/bin/env bash
exec "$local_pg_bin_dir/pg_dump" "\$@"
EOF
    cat >"$tmp_dir/pg_restore" <<EOF
#!/usr/bin/env bash
exec "$local_pg_bin_dir/pg_restore" "\$@"
EOF
else
    postgres_container="$(docker ps --format '{{.ID}} {{.Image}}' | awk '$2 ~ /^postgres:18\.4-alpine/ {print $1; exit}')"
    [[ -n "$postgres_container" ]] || { echo "PostgreSQL 18.4 service container not found" >&2; exit 1; }
    database_url() { printf 'postgres://postgres:postgres@localhost:5432/%s' "$1"; }
    run_psql() { run_psql "$@"; }
    cat >"$tmp_dir/pg_dump" <<EOF
#!/usr/bin/env bash
exec docker exec "$postgres_container" pg_dump "\$@"
EOF
    cat >"$tmp_dir/pg_restore" <<EOF
#!/usr/bin/env bash
exec docker exec -i "$postgres_container" pg_restore "\$@"
EOF
fi
chmod 700 "$tmp_dir/pg_dump" "$tmp_dir/pg_restore"

# Records that a restore executable was invoked without touching any database.
cat >"$tmp_dir/pg_restore_sentinel" <<EOF
#!/usr/bin/env bash
touch "$tmp_dir/restore_invoked"
exit 0
EOF
chmod 700 "$tmp_dir/pg_restore_sentinel"

source_url="$(database_url calendarweave_test)"
restore_url="$(database_url calendarweave_restore)"
tamper_url="$(database_url calendarweave_tamper)"
admin_url="$(database_url postgres)"
backup_path="$tmp_dir/calendarweave.dump"

# Establish one real calendar aggregate and revision through the same migration
# shipped with the PostgreSQL adapter. The values are synthetic and anonymous.
run_psql "$source_url" < "$repo_root/migrations/0001_calendar_resource_store.sql"
run_psql "$source_url" <<'SQL'
BEGIN;
INSERT INTO calendar_collection (collection_reference, tenant_reference, display_name)
VALUES ('collection_recovery_fixture', 'tenant_recovery_fixture', 'Recovery fixture');
INSERT INTO calendar_event (
    event_reference,
    collection_reference,
    calendar_uid,
    current_revision_number
) VALUES (
    'event_recovery_fixture',
    'collection_recovery_fixture',
    'event-recovery-fixture@example.test',
    1
);
INSERT INTO calendar_event_revision (
    event_reference,
    revision_number,
    summary_text,
    status_code,
    icalendar_payload
) VALUES (
    'event_recovery_fixture',
    1,
    'Recovery fixture event',
    'CONFIRMED',
    'BEGIN:VCALENDAR\nVERSION:2.0\nBEGIN:VEVENT\nUID:event-recovery-fixture@example.test\nDTSTART:20260902T010000Z\nDTEND:20260902T013000Z\nSUMMARY:Recovery fixture event\nEND:VEVENT\nEND:VCALENDAR\n'
);
COMMIT;
SQL

CALENDARWEAVE_DATABASE_URL="$source_url" \
CALENDARWEAVE_BACKUP_PATH="$backup_path" \
PG_DUMP_BIN="$tmp_dir/pg_dump" \
bash "$backup_script"

[[ -s "$backup_path" ]]
[[ -s "$backup_path.sha256" ]]
[[ "$(stat -c '%a' "$backup_path")" == "600" ]]
[[ "$(stat -c '%a' "$backup_path.sha256")" == "600" ]]

# Restore only into explicitly named recovery databases; never overwrite the
# source database during the drill.
run_psql "$admin_url" -v ON_ERROR_STOP=1 \
    -c 'CREATE DATABASE calendarweave_restore'
run_psql "$admin_url" -v ON_ERROR_STOP=1 \
    -c 'CREATE DATABASE calendarweave_tamper'

CALENDARWEAVE_RESTORE_DATABASE_URL="$restore_url" \
CALENDARWEAVE_BACKUP_PATH="$backup_path" \
PG_RESTORE_BIN="$tmp_dir/pg_restore" \
bash "$restore_script"

restored="$(run_psql "$restore_url" -At -v ON_ERROR_STOP=1 <<'SQL'
SELECT concat_ws('|',
    c.tenant_reference,
    c.collection_reference,
    e.event_reference,
    e.calendar_uid,
    e.current_revision_number,
    r.revision_number,
    r.summary_text,
    r.status_code
)
FROM calendar_collection AS c
JOIN calendar_event AS e USING (collection_reference)
JOIN calendar_event_revision AS r
  ON r.event_reference = e.event_reference
 AND r.revision_number = e.current_revision_number;
SQL
)"
[[ "$restored" == 'tenant_recovery_fixture|collection_recovery_fixture|event_recovery_fixture|event-recovery-fixture@example.test|1|1|Recovery fixture event|CONFIRMED' ]]

# The restored schema must preserve item-level idempotency and current-revision
# referential integrity rather than only recovering payload rows.
constraint_count="$(run_psql "$restore_url" -At -v ON_ERROR_STOP=1 <<'SQL'
SELECT count(*)
FROM pg_constraint
WHERE conname IN (
    'calendar_event_collection_uid_unique',
    'calendar_event_current_revision_foreign_key'
);
SQL
)"
[[ "$constraint_count" == '2' ]]

# Verification custody: the digest must be calculated from a private copy that
# lives in an owner-only directory, and that copy must be removed on success and
# on a checksum-mismatch exit. The recording SHA-256 executable captures the
# path and modes it was given without changing the reported digest.
cat >"$tmp_dir/sha256_recording" <<EOF
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "\$1" >"$tmp_dir/hashed_path"
stat -c '%a' "\$(dirname "\$1")" >"$tmp_dir/hashed_dir_mode"
stat -c '%a' "\$1" >"$tmp_dir/hashed_file_mode"
sha256sum "\$@"
EOF
chmod 700 "$tmp_dir/sha256_recording"
assert_private_custody() {
    local hashed_path
    hashed_path="$(cat "$tmp_dir/hashed_path")"
    if [[ "$hashed_path" == "$1" ]]; then
        echo 'digest was calculated by reopening the operator backup pathname' >&2
        exit 1
    fi
    [[ "$(cat "$tmp_dir/hashed_dir_mode")" == '700' ]] || { echo 'private copy directory is not mode 0700' >&2; exit 1; }
    [[ "$(cat "$tmp_dir/hashed_file_mode")" == '600' ]] || { echo 'private copy is not mode 0600' >&2; exit 1; }
    [[ ! -e "$hashed_path" && ! -e "$(dirname "$hashed_path")" ]] || { echo 'private copy survived restore exit' >&2; exit 1; }
}
run_psql "$admin_url" -v ON_ERROR_STOP=1 -c 'CREATE DATABASE calendarweave_custody'
custody_url="$(database_url calendarweave_custody)"
CALENDARWEAVE_RESTORE_DATABASE_URL="$custody_url" \
CALENDARWEAVE_BACKUP_PATH="$backup_path" \
PG_RESTORE_BIN="$tmp_dir/pg_restore" \
SHA256_BIN="$tmp_dir/sha256_recording" \
bash "$restore_script"
assert_private_custody "$backup_path"

mismatch_backup="$tmp_dir/mismatch.dump"
cp -- "$backup_path" "$mismatch_backup"
printf '%064d\n' 0 >"$mismatch_backup.sha256"
chmod 600 "$mismatch_backup" "$mismatch_backup.sha256"
rm -f "$tmp_dir/restore_invoked" "$tmp_dir/hashed_path"
mismatch_status=0
CALENDARWEAVE_RESTORE_DATABASE_URL="$tamper_url" \
CALENDARWEAVE_BACKUP_PATH="$mismatch_backup" \
PG_RESTORE_BIN="$tmp_dir/pg_restore_sentinel" \
SHA256_BIN="$tmp_dir/sha256_recording" \
bash "$restore_script" || mismatch_status=$?
[[ "$mismatch_status" -eq 65 ]] || { echo "checksum mismatch returned $mismatch_status instead of 65" >&2; exit 1; }
[[ ! -e "$tmp_dir/restore_invoked" ]]
assert_private_custody "$mismatch_backup"

# The private copy is also removed when pg_restore fails or the restore process
# is terminated by a signal while pg_restore runs.
cat >"$tmp_dir/pg_restore_failing" <<'EOF'
#!/usr/bin/env bash
cat >/dev/null
exit 1
EOF
cat >"$tmp_dir/pg_restore_terminating" <<'EOF'
#!/usr/bin/env bash
cat >/dev/null
kill -TERM "$PPID"
exit 0
EOF
chmod 700 "$tmp_dir/pg_restore_failing" "$tmp_dir/pg_restore_terminating"
for interrupted_restore in pg_restore_failing pg_restore_terminating; do
    rm -f "$tmp_dir/hashed_path"
    interrupted_status=0
    CALENDARWEAVE_RESTORE_DATABASE_URL="$tamper_url" \
    CALENDARWEAVE_BACKUP_PATH="$backup_path" \
    PG_RESTORE_BIN="$tmp_dir/$interrupted_restore" \
    SHA256_BIN="$tmp_dir/sha256_recording" \
    bash "$restore_script" || interrupted_status=$?
    [[ "$interrupted_status" -ne 0 ]] || { echo "$interrupted_restore unexpectedly succeeded" >&2; exit 1; }
    assert_private_custody "$backup_path"
done

# Time-of-check/time-of-use: the archive that pg_restore consumes must be the
# exact bytes whose digest was verified. Build a second, valid archive whose
# event summary differs, then use an injected SHA-256 executable that reports
# the real digest of what it was given and immediately renames the substitute
# archive over the operator-supplied backup pathname. A restore that reopens the
# pathname after verification would load the unverified substitute.
substitute_backup="$tmp_dir/substitute.dump"
run_psql "$source_url" -v ON_ERROR_STOP=1 \
    -c "UPDATE calendar_event_revision SET summary_text = 'Substituted unverified event' WHERE event_reference = 'event_recovery_fixture'"
"$tmp_dir/pg_dump" --format=custom --no-owner --no-privileges "$source_url" >"$substitute_backup"
run_psql "$source_url" -v ON_ERROR_STOP=1 \
    -c "UPDATE calendar_event_revision SET summary_text = 'Recovery fixture event' WHERE event_reference = 'event_recovery_fixture'"
[[ -s "$substitute_backup" ]]
if cmp -s "$substitute_backup" "$backup_path"; then
    echo 'substitute archive must differ from the verified archive' >&2
    exit 1
fi

cat >"$tmp_dir/sha256_then_swap" <<EOF
#!/usr/bin/env bash
set -euo pipefail
sha256sum "\$@"
if [[ ! -e "$tmp_dir/swap_done" ]]; then
    : >"$tmp_dir/swap_done"
    mv -f -- "$substitute_backup" "$backup_path"
fi
EOF
chmod 700 "$tmp_dir/sha256_then_swap"
run_psql "$admin_url" -v ON_ERROR_STOP=1 -c 'CREATE DATABASE calendarweave_swap'
swap_url="$(database_url calendarweave_swap)"

swap_status=0
CALENDARWEAVE_RESTORE_DATABASE_URL="$swap_url" \
CALENDARWEAVE_BACKUP_PATH="$backup_path" \
PG_RESTORE_BIN="$tmp_dir/pg_restore" \
SHA256_BIN="$tmp_dir/sha256_then_swap" \
bash "$restore_script" || swap_status=$?
[[ -e "$tmp_dir/swap_done" ]] || { echo 'swap hook was not exercised' >&2; exit 1; }

swap_summaries="$(run_psql "$swap_url" -At -v ON_ERROR_STOP=1 <<'SQL'
SELECT coalesce(string_agg(summary_text, ',' ORDER BY summary_text), '<none>')
FROM calendar_event_revision
WHERE to_regclass('public.calendar_event_revision') IS NOT NULL;
SQL
)" || swap_summaries='<no restored schema>'
if [[ "$swap_status" -eq 0 ]]; then
    if [[ "$swap_summaries" != 'Recovery fixture event' ]]; then
        echo "TOCTOU: restore reported success but loaded '$swap_summaries' instead of the verified archive" >&2
        exit 1
    fi
else
    echo "TOCTOU: restore of a verified archive failed with status $swap_status after the pathname was swapped" >&2
    exit 1
fi

# Tampering must be detected before pg_restore can mutate a target database.
printf 'tamper' >> "$backup_path"
if CALENDARWEAVE_RESTORE_DATABASE_URL="$tamper_url" \
   CALENDARWEAVE_BACKUP_PATH="$backup_path" \
   PG_RESTORE_BIN="$tmp_dir/pg_restore" \
   bash "$restore_script"; then
    echo 'tampered backup unexpectedly restored' >&2
    exit 1
fi

tamper_table_count="$(run_psql "$tamper_url" -At -v ON_ERROR_STOP=1 <<'SQL'
SELECT count(*)
FROM information_schema.tables
WHERE table_schema = 'public'
  AND table_name IN ('calendar_collection', 'calendar_event', 'calendar_event_revision');
SQL
)"
[[ "$tamper_table_count" == '0' ]]

# Validation failures must happen before the restore executable is called.

validation_backup="$tmp_dir/validation.dump"
printf 'validation archive' >"$validation_backup"
printf 'not-a-sha256\n' >"$validation_backup.sha256"
rm -f "$tmp_dir/restore_invoked"
if CALENDARWEAVE_RESTORE_DATABASE_URL="$tamper_url" \
   CALENDARWEAVE_BACKUP_PATH="$validation_backup" \
   PG_RESTORE_BIN="$tmp_dir/pg_restore_sentinel" \
   bash "$restore_script"; then
    echo 'malformed checksum unexpectedly accepted' >&2
    exit 1
fi
[[ ! -e "$tmp_dir/restore_invoked" ]]

real_archive="$tmp_dir/real.dump"
printf 'symlink validation archive' >"$real_archive"
real_digest="$(sha256sum "$real_archive" | awk '{print $1}')"

symlink_archive="$tmp_dir/symlink-archive.dump"
ln -s "$real_archive" "$symlink_archive"
printf '%s\n' "$real_digest" >"$symlink_archive.sha256"
rm -f "$tmp_dir/restore_invoked"
if CALENDARWEAVE_RESTORE_DATABASE_URL="$tamper_url" \
   CALENDARWEAVE_BACKUP_PATH="$symlink_archive" \
   PG_RESTORE_BIN="$tmp_dir/pg_restore_sentinel" \
   bash "$restore_script"; then
    echo 'symlinked backup archive unexpectedly accepted' >&2
    exit 1
fi
[[ ! -e "$tmp_dir/restore_invoked" ]]

regular_archive="$tmp_dir/regular-archive.dump"
printf 'checksum symlink archive' >"$regular_archive"
regular_digest="$(sha256sum "$regular_archive" | awk '{print $1}')"
printf '%s\n' "$regular_digest" >"$tmp_dir/real-checksum.sha256"
ln -s "$tmp_dir/real-checksum.sha256" "$regular_archive.sha256"
rm -f "$tmp_dir/restore_invoked"
if CALENDARWEAVE_RESTORE_DATABASE_URL="$tamper_url" \
   CALENDARWEAVE_BACKUP_PATH="$regular_archive" \
   PG_RESTORE_BIN="$tmp_dir/pg_restore_sentinel" \
   bash "$restore_script"; then
    echo 'symlinked backup checksum unexpectedly accepted' >&2
    exit 1
fi
[[ ! -e "$tmp_dir/restore_invoked" ]]

# Backup input validation must fail without publishing an artifact.
if CALENDARWEAVE_DATABASE_URL="$source_url" \
   CALENDARWEAVE_BACKUP_PATH='relative-calendarweave.dump' \
   PG_DUMP_BIN=/bin/true \
   bash "$backup_script"; then
    echo 'relative backup path unexpectedly accepted' >&2
    exit 1
fi

empty_backup="$tmp_dir/empty.dump"
if CALENDARWEAVE_DATABASE_URL="$source_url" \
   CALENDARWEAVE_BACKUP_PATH="$empty_backup" \
   PG_DUMP_BIN=/bin/true \
   bash "$backup_script"; then
    echo 'empty pg_dump output unexpectedly published' >&2
    exit 1
fi
[[ ! -e "$empty_backup" ]]
[[ ! -e "$empty_backup.sha256" ]]
