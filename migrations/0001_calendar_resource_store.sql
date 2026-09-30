BEGIN;

SELECT pg_advisory_xact_lock(1128354385);

CREATE TABLE IF NOT EXISTS calendar_collection (
    collection_reference text PRIMARY KEY,
    tenant_reference text NOT NULL,
    display_name text NOT NULL,
    CONSTRAINT calendar_collection_display_name_check
        CHECK (
            octet_length(display_name) BETWEEN 1 AND 200
            AND display_name = btrim(display_name)
        ),
    CONSTRAINT calendar_collection_tenant_reference_check
        CHECK (
            octet_length(tenant_reference) BETWEEN 1 AND 128
            AND tenant_reference ~ '^[A-Za-z0-9_.:-]+$'
        )
);

CREATE TABLE IF NOT EXISTS calendar_event (
    event_reference text PRIMARY KEY,
    collection_reference text NOT NULL
        REFERENCES calendar_collection (collection_reference),
    calendar_uid text NOT NULL,
    current_revision_number bigint NOT NULL,
    CONSTRAINT calendar_event_collection_uid_unique
        UNIQUE (collection_reference, calendar_uid),
    CONSTRAINT calendar_event_current_revision_number_check
        CHECK (current_revision_number > 0),
    CONSTRAINT calendar_event_current_revision_unique
        UNIQUE (event_reference, current_revision_number)
);

CREATE TABLE IF NOT EXISTS calendar_event_revision (
    event_reference text NOT NULL
        REFERENCES calendar_event (event_reference),
    revision_number bigint NOT NULL,
    summary_text text NOT NULL,
    status_code text NOT NULL,
    icalendar_payload text NOT NULL,
    CONSTRAINT calendar_event_revision_primary_key
        PRIMARY KEY (event_reference, revision_number),
    CONSTRAINT calendar_event_revision_number_check
        CHECK (revision_number > 0),
    CONSTRAINT calendar_event_revision_status_code_check
        CHECK (status_code IN ('CONFIRMED', 'TENTATIVE', 'CANCELLED'))
);

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'calendar_event_current_revision_foreign_key'
          AND conrelid = 'calendar_event'::regclass
    ) THEN
        ALTER TABLE calendar_event
            ADD CONSTRAINT calendar_event_current_revision_foreign_key
            FOREIGN KEY (event_reference, current_revision_number)
            REFERENCES calendar_event_revision (event_reference, revision_number)
            DEFERRABLE INITIALLY DEFERRED;
    END IF;
END
$$;

CREATE INDEX IF NOT EXISTS calendar_collection_tenant_lookup
    ON calendar_collection (tenant_reference, collection_reference);

CREATE INDEX IF NOT EXISTS calendar_event_collection_lookup
    ON calendar_event (collection_reference, event_reference);

COMMIT;
