//! Durable `PostgreSQL` adapter for the Calendar Resource Core application port.

use std::cell::RefCell;

use postgres::{Client, NoTls, Row, Transaction};
use uuid::Uuid;

use crate::{
    CalendarCollection, CalendarError, CalendarEvent, CalendarPort, EventStatus, TenantId,
    parse_event, validated_display_name,
};

const MIGRATION: &str = include_str!("../migrations/0001_calendar_resource_store.sql");

/// PostgreSQL-backed implementation of the versioned calendar application port.
pub struct PostgresCalendarService {
    client: RefCell<Client>,
}

impl PostgresCalendarService {
    /// Connect to one `PostgreSQL` database without logging the connection string.
    ///
    /// # Errors
    ///
    /// Returns [`CalendarError::StorageUnavailable`] when the connection fails.
    pub fn connect(connection_string: &str) -> Result<Self, CalendarError> {
        Client::connect(connection_string, NoTls)
            .map(|client| Self {
                client: RefCell::new(client),
            })
            .map_err(storage_error)
    }

    /// Replay the idempotent Calendar Resource Store migration.
    ///
    /// # Errors
    ///
    /// Returns [`CalendarError::StorageUnavailable`] when `PostgreSQL` rejects it.
    pub fn migrate(&self) -> Result<(), CalendarError> {
        self.client
            .borrow_mut()
            .batch_execute(MIGRATION)
            .map_err(storage_error)
    }
}

impl CalendarPort for PostgresCalendarService {
    fn create_collection(
        &mut self,
        tenant_id: &TenantId,
        display_name: &str,
    ) -> Result<CalendarCollection, CalendarError> {
        let display_name = validated_display_name(display_name)?;
        let collection = CalendarCollection {
            collection_ref: format!("cal_{}", Uuid::new_v4().simple()),
            display_name,
        };
        self.client
            .borrow_mut()
            .execute(
                "INSERT INTO calendar_collection
                    (collection_reference, tenant_reference, display_name)
                 VALUES ($1, $2, $3)",
                &[
                    &collection.collection_ref,
                    &tenant_id.as_str(),
                    &collection.display_name,
                ],
            )
            .map_err(storage_error)?;
        Ok(collection)
    }

    fn create_event(
        &mut self,
        tenant_id: &TenantId,
        collection_ref: &str,
        icalendar: &str,
    ) -> Result<CalendarEvent, CalendarError> {
        let mut client = self.client.borrow_mut();
        let mut transaction = client.transaction().map_err(storage_error)?;
        authorized_collection_transaction(&mut transaction, tenant_id, collection_ref)?;
        let parsed = parse_event(icalendar)?;
        let event_ref = format!("evt_{}", Uuid::new_v4().simple());
        let inserted = transaction
            .execute(
                "INSERT INTO calendar_event
                    (event_reference, collection_reference, calendar_uid, current_revision_number)
                 VALUES ($1, $2, $3, 1)
                 ON CONFLICT (collection_reference, calendar_uid) DO NOTHING",
                &[&event_ref, &collection_ref, &parsed.uid],
            )
            .map_err(storage_error)?;
        if inserted == 0 {
            let existing = find_event_by_uid(&mut transaction, collection_ref, &parsed.uid)?
                .ok_or(CalendarError::StorageUnavailable)?;
            return if existing.icalendar == icalendar {
                transaction.commit().map_err(storage_error)?;
                Ok(existing)
            } else {
                Err(CalendarError::StaleRevision)
            };
        }
        let event = event_value(event_ref, collection_ref.to_owned(), parsed, 1, icalendar);
        insert_revision(&mut transaction, &event)?;
        transaction.commit().map_err(storage_error)?;
        Ok(event)
    }

    fn update_event(
        &mut self,
        tenant_id: &TenantId,
        collection_ref: &str,
        event_ref: &str,
        if_match: &str,
        icalendar: &str,
    ) -> Result<CalendarEvent, CalendarError> {
        let mut client = self.client.borrow_mut();
        let mut transaction = client.transaction().map_err(storage_error)?;
        let uid = lock_event(&mut transaction, tenant_id, collection_ref, event_ref)?;
        let existing = find_current_event(&mut transaction, collection_ref, event_ref)?
            .ok_or(CalendarError::NotFound)?;
        if existing.etag != if_match {
            return Err(CalendarError::StaleRevision);
        }
        let parsed = parse_event(icalendar)?;
        if parsed.uid != uid {
            return Err(CalendarError::InvalidInput);
        }
        if existing.icalendar == icalendar {
            transaction.commit().map_err(storage_error)?;
            return Ok(existing);
        }
        let revision = existing
            .revision
            .checked_add(1)
            .ok_or(CalendarError::StaleRevision)?;
        let updated = event_value(
            event_ref.to_owned(),
            collection_ref.to_owned(),
            parsed,
            revision,
            icalendar,
        );
        insert_revision(&mut transaction, &updated)?;
        let revision_number = i64::try_from(revision).map_err(|_| CalendarError::StaleRevision)?;
        transaction
            .execute(
                "UPDATE calendar_event
                 SET current_revision_number = $2
                 WHERE event_reference = $1",
                &[&event_ref, &revision_number],
            )
            .map_err(storage_error)?;
        transaction.commit().map_err(storage_error)?;
        Ok(updated)
    }

    fn list_events(
        &self,
        tenant_id: &TenantId,
        collection_ref: &str,
    ) -> Result<Vec<CalendarEvent>, CalendarError> {
        let mut client = self.client.borrow_mut();
        authorized_collection(&mut client, tenant_id, collection_ref)?;
        client
            .query(
                &format!("{EVENT_SELECT} WHERE event_record.collection_reference = $1 ORDER BY event_record.event_reference"),
                &[&collection_ref],
            )
            .map_err(storage_error)?
            .iter()
            .map(row_event)
            .collect()
    }

    fn get_event(
        &self,
        tenant_id: &TenantId,
        collection_ref: &str,
        event_ref: &str,
    ) -> Result<CalendarEvent, CalendarError> {
        let mut client = self.client.borrow_mut();
        authorized_collection(&mut client, tenant_id, collection_ref)?;
        client
            .query_opt(
                &format!("{EVENT_SELECT} WHERE event_record.collection_reference = $1 AND event_record.event_reference = $2"),
                &[&collection_ref, &event_ref],
            )
            .map_err(storage_error)?
            .map_or(Err(CalendarError::NotFound), |row| row_event(&row))
    }
}

const EVENT_SELECT: &str = "SELECT
    event_record.event_reference,
    event_record.collection_reference,
    event_record.calendar_uid,
    revision_record.summary_text,
    revision_record.status_code,
    revision_record.revision_number,
    revision_record.icalendar_payload
FROM calendar_event AS event_record
JOIN calendar_event_revision AS revision_record
  ON revision_record.event_reference = event_record.event_reference
 AND revision_record.revision_number = event_record.current_revision_number";

fn authorized_collection_transaction(
    transaction: &mut Transaction<'_>,
    tenant_id: &TenantId,
    collection_ref: &str,
) -> Result<(), CalendarError> {
    transaction
        .query_opt(
            "SELECT collection_reference FROM calendar_collection
             WHERE collection_reference = $1 AND tenant_reference = $2",
            &[&collection_ref, &tenant_id.as_str()],
        )
        .map_err(storage_error)?
        .ok_or(CalendarError::NotFound)
        .map(|_| ())
}

fn authorized_collection(
    client: &mut Client,
    tenant_id: &TenantId,
    collection_ref: &str,
) -> Result<(), CalendarError> {
    client
        .query_opt(
            "SELECT collection_reference FROM calendar_collection
             WHERE collection_reference = $1 AND tenant_reference = $2",
            &[&collection_ref, &tenant_id.as_str()],
        )
        .map_err(storage_error)?
        .ok_or(CalendarError::NotFound)
        .map(|_| ())
}

fn lock_event(
    transaction: &mut Transaction<'_>,
    tenant_id: &TenantId,
    collection_ref: &str,
    event_ref: &str,
) -> Result<String, CalendarError> {
    transaction
        .query_opt(
            "SELECT event_record.calendar_uid
             FROM calendar_event AS event_record
             JOIN calendar_collection AS collection_record
               ON collection_record.collection_reference = event_record.collection_reference
             WHERE event_record.event_reference = $1
               AND event_record.collection_reference = $2
               AND collection_record.tenant_reference = $3
             FOR UPDATE OF event_record",
            &[&event_ref, &collection_ref, &tenant_id.as_str()],
        )
        .map_err(storage_error)?
        .ok_or(CalendarError::NotFound)
        .map(|row| row.get(0))
}

fn find_event_by_uid(
    transaction: &mut Transaction<'_>,
    collection_ref: &str,
    uid: &str,
) -> Result<Option<CalendarEvent>, CalendarError> {
    transaction
        .query_opt(
            &format!("{EVENT_SELECT} WHERE event_record.collection_reference = $1 AND event_record.calendar_uid = $2"),
            &[&collection_ref, &uid],
        )
        .map_err(storage_error)?
        .map(|row| row_event(&row))
        .transpose()
}

fn find_current_event(
    transaction: &mut Transaction<'_>,
    collection_ref: &str,
    event_ref: &str,
) -> Result<Option<CalendarEvent>, CalendarError> {
    transaction
        .query_opt(
            &format!("{EVENT_SELECT} WHERE event_record.collection_reference = $1 AND event_record.event_reference = $2"),
            &[&collection_ref, &event_ref],
        )
        .map_err(storage_error)?
        .map(|row| row_event(&row))
        .transpose()
}

fn insert_revision(
    transaction: &mut Transaction<'_>,
    event: &CalendarEvent,
) -> Result<(), CalendarError> {
    let revision = i64::try_from(event.revision).map_err(|_| CalendarError::StaleRevision)?;
    transaction
        .execute(
            "INSERT INTO calendar_event_revision
                (event_reference, revision_number, summary_text,
                 status_code, icalendar_payload)
             VALUES ($1, $2, $3, $4, $5)",
            &[
                &event.event_ref,
                &revision,
                &event.summary,
                &status_text(event.status),
                &event.icalendar,
            ],
        )
        .map_err(storage_error)?;
    Ok(())
}

fn event_value(
    event_ref: String,
    collection_ref: String,
    parsed: crate::ParsedEvent,
    revision: u64,
    icalendar: &str,
) -> CalendarEvent {
    CalendarEvent {
        etag: format!("\"{event_ref}:{revision}\""),
        event_ref,
        collection_ref,
        uid: parsed.uid,
        summary: parsed.summary,
        status: parsed.status,
        revision,
        icalendar: icalendar.to_owned(),
    }
}

fn row_event(row: &Row) -> Result<CalendarEvent, CalendarError> {
    let status = status_from_text(row.get(4))?;
    let revision = revision_from_i64(row.get(5))?;
    Ok(CalendarEvent {
        event_ref: row.get(0),
        collection_ref: row.get(1),
        uid: row.get(2),
        summary: row.get(3),
        status,
        revision,
        etag: format!("\"{}:{}\"", row.get::<_, String>(0), revision),
        icalendar: row.get(6),
    })
}

fn status_from_text(status: &str) -> Result<EventStatus, CalendarError> {
    Ok(match status {
        "CONFIRMED" => EventStatus::Confirmed,
        "TENTATIVE" => EventStatus::Tentative,
        "CANCELLED" => EventStatus::Cancelled,
        _ => return Err(CalendarError::StorageUnavailable),
    })
}

fn revision_from_i64(revision: i64) -> Result<u64, CalendarError> {
    u64::try_from(revision).map_err(|_| CalendarError::StorageUnavailable)
}

const fn status_text(status: EventStatus) -> &'static str {
    match status {
        EventStatus::Confirmed => "CONFIRMED",
        EventStatus::Tentative => "TENTATIVE",
        EventStatus::Cancelled => "CANCELLED",
    }
}

fn storage_error<T>(_error: T) -> CalendarError {
    CalendarError::StorageUnavailable
}

#[cfg(test)]
mod tests {
    use super::{revision_from_i64, status_from_text};
    use crate::{CalendarError, EventStatus};

    #[test]
    fn stored_projection_values_fail_closed() {
        assert_eq!(status_from_text("CONFIRMED"), Ok(EventStatus::Confirmed));
        assert_eq!(status_from_text("TENTATIVE"), Ok(EventStatus::Tentative));
        assert_eq!(status_from_text("CANCELLED"), Ok(EventStatus::Cancelled));
        assert_eq!(
            status_from_text("unknown"),
            Err(CalendarError::StorageUnavailable)
        );
        assert_eq!(revision_from_i64(1), Ok(1));
        assert_eq!(
            revision_from_i64(-1),
            Err(CalendarError::StorageUnavailable)
        );
    }
}
