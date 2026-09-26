//! Real `PostgreSQL` acceptance for durable calendar identity and concurrency.

use std::{env, sync::Barrier, thread};

use calendarweave::{
    CalendarError, CalendarPort, EventStatus, TenantId, postgres_store::PostgresCalendarService,
};

const UTC_EVENT: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//ContextualWisdomLab//CalendarWeave PostgreSQL Test//EN\r\nBEGIN:VEVENT\r\nUID:synthetic-postgres-event@example.test\r\nDTSTAMP:20260901T000000Z\r\nDTSTART:20260902T090000Z\r\nDTEND:20260902T100000Z\r\nSUMMARY:Synthetic durable review\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

fn database_url() -> Option<String> {
    env::var("CALENDARWEAVE_TEST_DATABASE_URL").ok()
}

fn tenant(value: &str) -> TenantId {
    TenantId::parse(value).expect("synthetic tenant is valid")
}

#[test]
fn postgres_store_preserves_identity_scope_and_etag_concurrency() {
    let Some(database_url) = database_url() else {
        return;
    };
    let run_id = uuid::Uuid::new_v4().simple().to_string();
    let owner = tenant(&format!("synthetic-owner-{run_id}"));
    let outsider = tenant(&format!("synthetic-outsider-{run_id}"));
    let mut first = PostgresCalendarService::connect(&database_url).unwrap();
    first.migrate().unwrap();
    first.migrate().unwrap();
    let collection = first
        .create_collection(&owner, "Synthetic durable calendar")
        .unwrap();
    let created = first
        .create_event(&owner, &collection.collection_ref, UTC_EVENT)
        .unwrap();

    let mut restarted = PostgresCalendarService::connect(&database_url).unwrap();
    restarted.migrate().unwrap();
    assert_eq!(
        restarted
            .create_event(&owner, &collection.collection_ref, UTC_EVENT)
            .unwrap(),
        created
    );
    assert_eq!(
        restarted
            .get_event(&owner, &collection.collection_ref, &created.event_ref)
            .unwrap(),
        created
    );
    assert_eq!(
        restarted
            .list_events(&owner, &collection.collection_ref)
            .unwrap(),
        vec![created.clone()]
    );

    assert_eq!(
        restarted.list_events(&outsider, &collection.collection_ref),
        Err(CalendarError::NotFound)
    );
    assert_eq!(
        restarted.create_event(&outsider, &collection.collection_ref, "not iCalendar"),
        Err(CalendarError::NotFound)
    );
    assert_eq!(
        restarted.update_event(
            &outsider,
            &collection.collection_ref,
            &created.event_ref,
            &created.etag,
            "not iCalendar",
        ),
        Err(CalendarError::NotFound)
    );

    let changed = UTC_EVENT.replace("Synthetic durable review", "Synthetic revised review");
    let updated = restarted
        .update_event(
            &owner,
            &collection.collection_ref,
            &created.event_ref,
            &created.etag,
            &changed,
        )
        .unwrap();
    assert_eq!(updated.revision, 2);
    assert_eq!(updated.event_ref, created.event_ref);
    assert_eq!(
        first.update_event(
            &owner,
            &collection.collection_ref,
            &created.event_ref,
            &created.etag,
            UTC_EVENT,
        ),
        Err(CalendarError::StaleRevision)
    );
    assert_eq!(
        restarted
            .update_event(
                &owner,
                &collection.collection_ref,
                &updated.event_ref,
                &updated.etag,
                &changed,
            )
            .unwrap(),
        updated
    );
}

#[test]
fn postgres_store_rejects_conflicting_uid_content_and_uid_replacement() {
    let Some(database_url) = database_url() else {
        return;
    };
    let run_id = uuid::Uuid::new_v4().simple().to_string();
    let owner = tenant(&format!("synthetic-uid-owner-{run_id}"));
    let mut service = PostgresCalendarService::connect(&database_url).unwrap();
    service.migrate().unwrap();
    let collection = service
        .create_collection(&owner, "Synthetic UID calendar")
        .unwrap();
    let created = service
        .create_event(&owner, &collection.collection_ref, UTC_EVENT)
        .unwrap();
    assert_eq!(
        service.create_event(
            &owner,
            &collection.collection_ref,
            &UTC_EVENT.replace("Synthetic durable review", "Conflicting duplicate"),
        ),
        Err(CalendarError::StaleRevision)
    );
    assert_eq!(
        service.update_event(
            &owner,
            &collection.collection_ref,
            &created.event_ref,
            &created.etag,
            &UTC_EVENT.replace("synthetic-postgres-event", "changed-uid"),
        ),
        Err(CalendarError::InvalidInput)
    );
}

#[test]
fn concurrent_identical_create_converges_on_one_durable_event() {
    let Some(database_url) = database_url() else {
        return;
    };
    let run_id = uuid::Uuid::new_v4().simple().to_string();
    let owner = tenant(&format!("synthetic-race-owner-{run_id}"));
    let mut setup = PostgresCalendarService::connect(&database_url).unwrap();
    setup.migrate().unwrap();
    let collection = setup
        .create_collection(&owner, "Synthetic race calendar")
        .unwrap();
    let barrier = std::sync::Arc::new(Barrier::new(2));
    let mut handles = Vec::new();
    for _ in 0..2 {
        let database_url = database_url.clone();
        let owner = owner.clone();
        let collection_ref = collection.collection_ref.clone();
        let barrier = barrier.clone();
        handles.push(thread::spawn(move || {
            let mut service = PostgresCalendarService::connect(&database_url).unwrap();
            barrier.wait();
            service
                .create_event(&owner, &collection_ref, UTC_EVENT)
                .unwrap()
        }));
    }
    let left = handles.remove(0).join().unwrap();
    let right = handles.remove(0).join().unwrap();
    assert_eq!(left, right);
    assert_eq!(
        setup
            .list_events(&owner, &collection.collection_ref)
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn concurrent_different_create_keeps_one_uid_revision() {
    let Some(database_url) = database_url() else {
        return;
    };
    let run_id = uuid::Uuid::new_v4().simple().to_string();
    let owner = tenant(&format!("synthetic-create-owner-{run_id}"));
    let mut setup = PostgresCalendarService::connect(&database_url).unwrap();
    setup.migrate().unwrap();
    let collection = setup
        .create_collection(&owner, "Synthetic create calendar")
        .unwrap();
    let barrier = std::sync::Arc::new(Barrier::new(2));
    let mut handles = Vec::new();
    for summary in ["Synthetic left create", "Synthetic right create"] {
        let database_url = database_url.clone();
        let owner = owner.clone();
        let collection_ref = collection.collection_ref.clone();
        let barrier = barrier.clone();
        handles.push(thread::spawn(move || {
            let mut service = PostgresCalendarService::connect(&database_url).unwrap();
            let payload = UTC_EVENT.replace("Synthetic durable review", summary);
            barrier.wait();
            service.create_event(&owner, &collection_ref, &payload)
        }));
    }
    let outcomes = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(outcomes.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(
        outcomes
            .iter()
            .filter(|result| **result == Err(CalendarError::StaleRevision))
            .count(),
        1
    );
    let stored = setup
        .list_events(&owner, &collection.collection_ref)
        .unwrap();
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].revision, 1);
}

#[test]
fn concurrent_updates_with_one_etag_allow_exactly_one_writer() {
    let Some(database_url) = database_url() else {
        return;
    };
    let run_id = uuid::Uuid::new_v4().simple().to_string();
    let owner = tenant(&format!("synthetic-update-owner-{run_id}"));
    let mut setup = PostgresCalendarService::connect(&database_url).unwrap();
    setup.migrate().unwrap();
    let collection = setup
        .create_collection(&owner, "Synthetic update calendar")
        .unwrap();
    let created = setup
        .create_event(&owner, &collection.collection_ref, UTC_EVENT)
        .unwrap();
    let barrier = std::sync::Arc::new(Barrier::new(2));
    let mut handles = Vec::new();
    for summary in ["Synthetic left writer", "Synthetic right writer"] {
        let database_url = database_url.clone();
        let owner = owner.clone();
        let collection_ref = collection.collection_ref.clone();
        let event_ref = created.event_ref.clone();
        let etag = created.etag.clone();
        let barrier = barrier.clone();
        handles.push(thread::spawn(move || {
            let mut service = PostgresCalendarService::connect(&database_url).unwrap();
            let payload = UTC_EVENT.replace("Synthetic durable review", summary);
            barrier.wait();
            service.update_event(&owner, &collection_ref, &event_ref, &etag, &payload)
        }));
    }
    let outcomes = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(outcomes.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(
        outcomes
            .iter()
            .filter(|result| **result == Err(CalendarError::StaleRevision))
            .count(),
        1
    );
    assert_eq!(
        setup
            .get_event(&owner, &collection.collection_ref, &created.event_ref)
            .unwrap()
            .revision,
        2
    );
}

#[test]
fn postgres_projection_preserves_all_supported_status_values() {
    let Some(database_url) = database_url() else {
        return;
    };
    let run_id = uuid::Uuid::new_v4().simple().to_string();
    let owner = tenant(&format!("synthetic-status-owner-{run_id}"));
    let mut service = PostgresCalendarService::connect(&database_url).unwrap();
    service.migrate().unwrap();
    let collection = service
        .create_collection(&owner, "Synthetic status calendar")
        .unwrap();
    for (status_text, expected) in [
        ("CONFIRMED", EventStatus::Confirmed),
        ("TENTATIVE", EventStatus::Tentative),
        ("CANCELLED", EventStatus::Cancelled),
    ] {
        let event = UTC_EVENT
            .replace(
                "synthetic-postgres-event",
                &format!("synthetic-{status_text}-{run_id}"),
            )
            .replace(
                "SUMMARY:Synthetic durable review",
                &format!("SUMMARY:Synthetic durable review\r\nSTATUS:{status_text}"),
            );
        assert_eq!(
            service
                .create_event(&owner, &collection.collection_ref, &event)
                .unwrap()
                .status,
            expected
        );
        let stored = service
            .list_events(&owner, &collection.collection_ref)
            .unwrap();
        assert!(stored.iter().any(|candidate| candidate.status == expected));
    }
}

#[test]
fn unavailable_database_returns_a_bounded_error() {
    assert!(matches!(
        PostgresCalendarService::connect("postgresql://127.0.0.1:1/unavailable"),
        Err(CalendarError::StorageUnavailable)
    ));
}
