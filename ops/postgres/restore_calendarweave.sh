#!/usr/bin/env bash
set -euo pipefail

: "${CALENDARWEAVE_RESTORE_DATABASE_URL:?CALENDARWEAVE_RESTORE_DATABASE_URL is required}"
: "${CALENDARWEAVE_BACKUP_PATH:?CALENDARWEAVE_BACKUP_PATH is required}"

pg_restore_bin="${PG_RESTORE_BIN:-pg_restore}"
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

# Restore never follows a symlink for either evidence file. The checksum is
# verified before pg_restore starts, so corrupted or substituted evidence
# cannot partially mutate the target database through this script.
if [[ ! -f "$backup_path" || -L "$backup_path" ]]; then
    echo "backup archive is missing or not a regular file" >&2
    exit 66
fi
if [[ ! -f "$checksum_path" || -L "$checksum_path" ]]; then
    echo "backup checksum is missing or not a regular file" >&2
    exit 66
fi

IFS= read -r expected_digest <"$checksum_path" || true
if [[ ! "$expected_digest" =~ ^[0-9a-f]{64}$ ]]; then
    echo "backup checksum file is malformed" >&2
    exit 65
fi

actual_digest="$($sha256_bin "$backup_path" | awk 'NR == 1 {print $1}')"
actual_digest="${actual_digest,,}"
if [[ "$actual_digest" != "$expected_digest" ]]; then
    echo "backup checksum mismatch; restore aborted before database mutation" >&2
    exit 65
fi

"$pg_restore_bin" \
    --single-transaction \
    --no-owner \
    --no-privileges \
    --dbname="$CALENDARWEAVE_RESTORE_DATABASE_URL" \
    <"$backup_path"
