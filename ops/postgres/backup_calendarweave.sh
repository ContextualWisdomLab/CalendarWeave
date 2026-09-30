#!/usr/bin/env bash
set -euo pipefail

: "${CALENDARWEAVE_DATABASE_URL:?CALENDARWEAVE_DATABASE_URL is required}"
: "${CALENDARWEAVE_BACKUP_PATH:?CALENDARWEAVE_BACKUP_PATH is required}"

pg_dump_bin="${PG_DUMP_BIN:-pg_dump}"
sha256_bin="${SHA256_BIN:-sha256sum}"
backup_path="$CALENDARWEAVE_BACKUP_PATH"
checksum_path="${backup_path}.sha256"

case "$backup_path" in
    /*) ;;
    *)
        echo "CALENDARWEAVE_BACKUP_PATH must be an absolute path" >&2
        exit 64
        ;;
esac

backup_dir="$(dirname "$backup_path")"
if [[ ! -d "$backup_dir" ]]; then
    echo "backup directory does not exist: $backup_dir" >&2
    exit 66
fi

# A logical backup may contain calendar content and principal-linked evidence.
# Keep every newly-created artifact private by default and publish by atomic
# rename only after pg_dump and checksum generation succeed.
umask 077
temporary_dump="$(mktemp "${backup_path}.tmp.XXXXXX")"
temporary_checksum="$(mktemp "${checksum_path}.tmp.XXXXXX")"
cleanup() {
    rm -f -- "$temporary_dump" "$temporary_checksum"
}
trap cleanup EXIT

"$pg_dump_bin" \
    --format=custom \
    --no-owner \
    --no-privileges \
    "$CALENDARWEAVE_DATABASE_URL" >"$temporary_dump"

if [[ ! -s "$temporary_dump" ]]; then
    echo "pg_dump produced an empty archive" >&2
    exit 65
fi

chmod 600 "$temporary_dump"
digest="$($sha256_bin "$temporary_dump" | awk 'NR == 1 {print $1}')"
if [[ ! "$digest" =~ ^[[:xdigit:]]{64}$ ]]; then
    echo "failed to calculate a SHA-256 digest for the backup" >&2
    exit 65
fi
printf '%s\n' "${digest,,}" >"$temporary_checksum"
chmod 600 "$temporary_checksum"

# Replacing the dump before its checksum can expose only a short-lived
# fail-closed mismatch to readers; restore always verifies the digest before
# invoking pg_restore.
mv -f -- "$temporary_dump" "$backup_path"
mv -f -- "$temporary_checksum" "$checksum_path"
trap - EXIT
