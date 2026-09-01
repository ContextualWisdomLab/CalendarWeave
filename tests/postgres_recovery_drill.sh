#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
backup_script="$repo_root/ops/postgres/backup_calendarweave.sh"
restore_script="$repo_root/ops/postgres/restore_calendarweave.sh"

[[ -f "$backup_script" ]] || { echo "missing production backup contract: $backup_script" >&2; exit 1; }
[[ -f "$restore_script" ]] || { echo "missing production restore contract: $restore_script" >&2; exit 1; }

postgres_container="$(docker ps --format '{{.ID}} {{.Image}}' | awk '$2 ~ /^postgres:18\.4-alpine/ {print $1; exit}')"
[[ -n "$postgres_container" ]] || { echo "PostgreSQL 18.4 service container not found" >&2; exit 1; }

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

cat >"$tmp_dir/pg_dump" <<EOF
#!/usr/bin/env bash
exec docker exec "$postgres_container" pg_dump "\$@"
EOF
cat >"$tmp_dir/pg_restore" <<EOF
#!/usr/bin/env bash
exec docker exec -i "$postgres_container" pg_restore "\$@"
EOF
chmod 700 "$tmp_dir/pg_dump" "$tmp_dir/pg_restore"

source_url="postgres://postgres:postgres@localhost:5432/calendarweave_test"
restore_url="postgres://postgres:postgres@localhost:5432/calendarweave_restore"
tamper_url="postgres://postgres:postgres@localhost:5432/calendarweave_tamper"
admin_url="postgres://postgres:postgres@localhost:5432/postgres"
backup_path="$tmp_dir/calendarweave.dump"

# Establish one real calendar aggregate and revision through the same migration
# shipped with the PostgreSQL adapter. The values are synthetic and anonymous.
docker exec -i "$postgres_container" psql "$source_url" < "$repo_root/migrations/0001_calendar_resource_store.sql"
docker exec -i "$postgres_container" psql "$source_url" <<'SQL'
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
docker exec "$postgres_container" psql "$admin_url" -v ON_ERROR_STOP=1 \
    -c 'CREATE DATABASE calendarweave_restore'
docker exec "$postgres_container" psql "$admin_url" -v ON_ERROR_STOP=1 \
    -c 'CREATE DATABASE calendarweave_tamper'

CALENDARWEAVE_RESTORE_DATABASE_URL="$restore_url" \
CALENDARWEAVE_BACKUP_PATH="$backup_path" \
PG_RESTORE_BIN="$tmp_dir/pg_restore" \
bash "$restore_script"

restored="$(docker exec "$postgres_container" psql "$restore_url" -At -v ON_ERROR_STOP=1 <<'SQL'
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
constraint_count="$(docker exec "$postgres_container" psql "$restore_url" -At -v ON_ERROR_STOP=1 <<'SQL'
SELECT count(*)
FROM pg_constraint
WHERE conname IN (
    'calendar_event_collection_uid_unique',
    'calendar_event_current_revision_foreign_key'
);
SQL
)"
[[ "$constraint_count" == '2' ]]

# Tampering must be detected before pg_restore can mutate a target database.
printf 'tamper' >> "$backup_path"
if CALENDARWEAVE_RESTORE_DATABASE_URL="$tamper_url" \
   CALENDARWEAVE_BACKUP_PATH="$backup_path" \
   PG_RESTORE_BIN="$tmp_dir/pg_restore" \
   bash "$restore_script"; then
    echo 'tampered backup unexpectedly restored' >&2
    exit 1
fi

tamper_table_count="$(docker exec "$postgres_container" psql "$tamper_url" -At -v ON_ERROR_STOP=1 <<'SQL'
SELECT count(*)
FROM information_schema.tables
WHERE table_schema = 'public'
  AND table_name IN ('calendar_collection', 'calendar_event', 'calendar_event_revision');
SQL
)"
[[ "$tamper_table_count" == '0' ]]

# Validation failures must happen before the restore executable is called.
cat >"$tmp_dir/pg_restore_sentinel" <<EOF
#!/usr/bin/env bash
touch "$tmp_dir/restore_invoked"
exit 0
EOF
chmod 700 "$tmp_dir/pg_restore_sentinel"

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
