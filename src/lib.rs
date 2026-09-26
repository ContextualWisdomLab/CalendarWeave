//! `CalendarWeave`'s first executable calendar-resource contract.
//!
//! The crate owns generic calendar collections and RFC 5545 event resources.
//! It deliberately ships no provider adapter or consumer-specific policy.

use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;

use chrono::{NaiveDate, NaiveDateTime};
use icalendar::{Calendar, CalendarComponent, Component, Property};
use uuid::Uuid;

pub mod postgres_store;

const ALLOWED_EVENT_PROPERTIES: [&str; 7] = [
    "UID", "DTSTAMP", "DTSTART", "DTEND", "SUMMARY", "SEQUENCE", "STATUS",
];

/// A bounded failure returned by the calendar-resource application port.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CalendarError {
    /// An identity or display value does not satisfy the public contract.
    InvalidInput,
    /// The resource is absent or belongs to another tenant.
    NotFound,
    /// The payload is not the supported RFC 5545 structure.
    MalformedCalendar,
    /// The payload requests a calendar capability not published by v1.
    UnsupportedCapability,
    /// The same UID already identifies different immutable event content.
    StaleRevision,
    /// Durable storage could not complete the operation safely.
    StorageUnavailable,
}

/// A validated tenant scope used for every collection and event operation.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TenantId(String);

impl TenantId {
    /// Validate a bounded opaque tenant identifier.
    ///
    /// `CalendarWeave` does not interpret identity-provider claims; its caller
    /// supplies the already-authorized tenant scope through this value.
    ///
    /// # Errors
    ///
    /// Returns [`CalendarError::InvalidInput`] when the value is empty, longer
    /// than 128 bytes, or contains characters outside the opaque ID profile.
    pub fn parse(value: &str) -> Result<Self, CalendarError> {
        if value.is_empty()
            || value.len() > 128
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"-_.:".contains(&byte))
        {
            return Err(CalendarError::InvalidInput);
        }
        Ok(Self(value.to_owned()))
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

/// Public evidence that a tenant-scoped collection was created.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CalendarCollection {
    /// Opaque reference safe for consumers to retain.
    pub collection_ref: String,
    /// Human-readable collection name supplied by the authorized caller.
    pub display_name: String,
}

/// Public read projection for one immutable event revision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CalendarEvent {
    /// Opaque `CalendarWeave` event reference, distinct from RFC UID.
    pub event_ref: String,
    /// Opaque containing collection reference.
    pub collection_ref: String,
    /// RFC 5545 globally unique event identifier.
    pub uid: String,
    /// Buyer-visible RFC 5545 summary.
    pub summary: String,
    /// RFC 5545 event status; an omitted property means confirmed.
    pub status: EventStatus,
    /// Monotonic `CalendarWeave` revision; initial creation is revision one.
    pub revision: u64,
    /// Strong revision token suitable for an eventual If-Match boundary.
    pub etag: String,
    /// Original validated RFC 5545 calendar representation.
    pub icalendar: String,
}

/// Supported RFC 5545 VEVENT status values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventStatus {
    /// The event is confirmed, including when STATUS is omitted.
    Confirmed,
    /// The event is tentative.
    Tentative,
    /// The event is cancelled; consuming conflict policy decides occupancy.
    Cancelled,
}

/// Versioned application port consumed by service or package adapters.
pub trait CalendarPort {
    /// Create a tenant-owned collection with an opaque reference.
    ///
    /// # Errors
    ///
    /// Returns [`CalendarError::InvalidInput`] for an empty or overlong name.
    fn create_collection(
        &mut self,
        tenant_id: &TenantId,
        display_name: &str,
    ) -> Result<CalendarCollection, CalendarError>;

    /// Validate and create one v1 RFC 5545 event resource.
    ///
    /// # Errors
    ///
    /// Returns a bounded error for an absent tenant collection, malformed or
    /// unsupported RFC payload, or conflicting immutable UID revision.
    fn create_event(
        &mut self,
        tenant_id: &TenantId,
        collection_ref: &str,
        icalendar: &str,
    ) -> Result<CalendarEvent, CalendarError>;

    /// Replace one event only when the caller presents its current strong `ETag`.
    ///
    /// # Errors
    ///
    /// Returns [`CalendarError::NotFound`] for absent or cross-tenant resources,
    /// [`CalendarError::StaleRevision`] for a stale `ETag`, and a bounded input
    /// error when the replacement is malformed or changes the immutable UID.
    fn update_event(
        &mut self,
        tenant_id: &TenantId,
        collection_ref: &str,
        event_ref: &str,
        if_match: &str,
        icalendar: &str,
    ) -> Result<CalendarEvent, CalendarError>;

    /// List authorized event revisions without exposing another tenant.
    ///
    /// # Errors
    ///
    /// Returns [`CalendarError::NotFound`] for absent and cross-tenant scopes.
    fn list_events(
        &self,
        tenant_id: &TenantId,
        collection_ref: &str,
    ) -> Result<Vec<CalendarEvent>, CalendarError>;

    /// Get one authorized event revision by opaque reference.
    ///
    /// # Errors
    ///
    /// Returns [`CalendarError::NotFound`] for absent and cross-tenant scopes.
    fn get_event(
        &self,
        tenant_id: &TenantId,
        collection_ref: &str,
        event_ref: &str,
    ) -> Result<CalendarEvent, CalendarError>;
}

#[derive(Debug)]
struct CollectionRecord {
    tenant_id: TenantId,
    collection: CalendarCollection,
    events: BTreeMap<String, CalendarEvent>,
    event_ref_by_uid: HashMap<String, String>,
}

/// In-process adapter for the published application port.
///
/// It provides executable consumer fixtures without claiming durable storage
/// or CalDAV/WebDAV behavior that has not shipped yet.
#[derive(Debug, Default)]
pub struct InMemoryCalendarService {
    collections: HashMap<String, CollectionRecord>,
}

impl InMemoryCalendarService {
    /// Construct an empty isolated calendar-resource adapter.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn collection(
        &self,
        tenant_id: &TenantId,
        collection_ref: &str,
    ) -> Result<&CollectionRecord, CalendarError> {
        self.collections
            .get(collection_ref)
            .filter(|record| &record.tenant_id == tenant_id)
            .ok_or(CalendarError::NotFound)
    }

    fn collection_mut(
        &mut self,
        tenant_id: &TenantId,
        collection_ref: &str,
    ) -> Result<&mut CollectionRecord, CalendarError> {
        self.collections
            .get_mut(collection_ref)
            .filter(|record| &record.tenant_id == tenant_id)
            .ok_or(CalendarError::NotFound)
    }
}

impl CalendarPort for InMemoryCalendarService {
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
        self.collections.insert(
            collection.collection_ref.clone(),
            CollectionRecord {
                tenant_id: tenant_id.clone(),
                collection: collection.clone(),
                events: BTreeMap::new(),
                event_ref_by_uid: HashMap::new(),
            },
        );
        Ok(collection)
    }

    fn create_event(
        &mut self,
        tenant_id: &TenantId,
        collection_ref: &str,
        icalendar: &str,
    ) -> Result<CalendarEvent, CalendarError> {
        let record = self.collection_mut(tenant_id, collection_ref)?;
        let parsed = parse_event(icalendar)?;
        if let Some(event_ref) = record.event_ref_by_uid.get(&parsed.uid) {
            let existing = &record.events[event_ref];
            return if existing.icalendar == icalendar {
                Ok(existing.clone())
            } else {
                Err(CalendarError::StaleRevision)
            };
        }
        let event_ref = format!("evt_{}", Uuid::new_v4().simple());
        let event = CalendarEvent {
            event_ref: event_ref.clone(),
            collection_ref: record.collection.collection_ref.clone(),
            uid: parsed.uid.clone(),
            summary: parsed.summary,
            status: parsed.status,
            revision: 1,
            etag: format!("\"{event_ref}:1\""),
            icalendar: icalendar.to_owned(),
        };
        record
            .event_ref_by_uid
            .insert(parsed.uid, event_ref.clone());
        record.events.insert(event_ref, event.clone());
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
        let record = self.collection_mut(tenant_id, collection_ref)?;
        let existing = record
            .events
            .get(event_ref)
            .cloned()
            .ok_or(CalendarError::NotFound)?;
        if existing.etag != if_match {
            return Err(CalendarError::StaleRevision);
        }
        let parsed = parse_event(icalendar)?;
        if parsed.uid != existing.uid {
            return Err(CalendarError::InvalidInput);
        }
        if existing.icalendar == icalendar {
            return Ok(existing);
        }
        let revision = existing
            .revision
            .checked_add(1)
            .ok_or(CalendarError::StaleRevision)?;
        let updated = CalendarEvent {
            event_ref: existing.event_ref,
            collection_ref: existing.collection_ref,
            uid: existing.uid,
            summary: parsed.summary,
            status: parsed.status,
            revision,
            etag: format!("\"{event_ref}:{revision}\""),
            icalendar: icalendar.to_owned(),
        };
        record.events.insert(event_ref.to_owned(), updated.clone());
        Ok(updated)
    }

    fn list_events(
        &self,
        tenant_id: &TenantId,
        collection_ref: &str,
    ) -> Result<Vec<CalendarEvent>, CalendarError> {
        Ok(self
            .collection(tenant_id, collection_ref)?
            .events
            .values()
            .cloned()
            .collect())
    }

    fn get_event(
        &self,
        tenant_id: &TenantId,
        collection_ref: &str,
        event_ref: &str,
    ) -> Result<CalendarEvent, CalendarError> {
        self.collection(tenant_id, collection_ref)?
            .events
            .get(event_ref)
            .cloned()
            .ok_or(CalendarError::NotFound)
    }
}

pub(crate) struct ParsedEvent {
    pub(crate) uid: String,
    pub(crate) summary: String,
    pub(crate) status: EventStatus,
}

pub(crate) fn parse_event(input: &str) -> Result<ParsedEvent, CalendarError> {
    if !input.ends_with("\r\n") || input.replace("\r\n", "").contains('\n') {
        return Err(CalendarError::MalformedCalendar);
    }
    validate_singleton_properties(input)?;
    let calendar = Calendar::from_str(input).map_err(|_| CalendarError::MalformedCalendar)?;
    if calendar.property_value("VERSION") != Some("2.0")
        || calendar.property_value("PRODID").is_none_or(str::is_empty)
        || calendar.property_value("METHOD").is_some()
    {
        return Err(CalendarError::MalformedCalendar);
    }
    let mut components = calendar.iter();
    let (Some(CalendarComponent::Event(event)), None) = (components.next(), components.next())
    else {
        return Err(CalendarError::MalformedCalendar);
    };
    if !event.multi_properties().is_empty()
        || !event.components().is_empty()
        || event
            .properties()
            .keys()
            .any(|property| !ALLOWED_EVENT_PROPERTIES.contains(&property.as_str()))
    {
        return Err(CalendarError::UnsupportedCapability);
    }
    let uid = required_text(event.properties().get("UID"))?;
    let summary = required_text(event.properties().get("SUMMARY"))?;
    let status = parse_status(event.properties().get("STATUS"))?;
    validate_utc("DTSTAMP", event.properties().get("DTSTAMP"))?;
    validate_interval(
        event.properties().get("DTSTART"),
        event.properties().get("DTEND"),
    )?;
    if let Some(sequence) = event.properties().get("SEQUENCE")
        && (!sequence.params().is_empty() || sequence.value().parse::<u32>().is_err())
    {
        return Err(CalendarError::MalformedCalendar);
    }
    Ok(ParsedEvent {
        uid,
        summary,
        status,
    })
}

pub(crate) fn validated_display_name(display_name: &str) -> Result<String, CalendarError> {
    let display_name = display_name.trim();
    if display_name.is_empty() || display_name.len() > 200 {
        return Err(CalendarError::InvalidInput);
    }
    Ok(display_name.to_owned())
}

fn validate_singleton_properties(input: &str) -> Result<(), CalendarError> {
    const REQUIRED_ONCE: [&str; 7] = [
        "VERSION", "PRODID", "UID", "DTSTAMP", "DTSTART", "DTEND", "SUMMARY",
    ];
    for required in REQUIRED_ONCE {
        if input
            .split("\r\n")
            .filter(|line| property_name(line) == Some(required))
            .count()
            != 1
        {
            return Err(CalendarError::MalformedCalendar);
        }
    }
    for optional in ["SEQUENCE", "STATUS"] {
        if input
            .split("\r\n")
            .filter(|line| property_name(line) == Some(optional))
            .count()
            > 1
        {
            return Err(CalendarError::MalformedCalendar);
        }
    }
    Ok(())
}

fn parse_status(property: Option<&Property>) -> Result<EventStatus, CalendarError> {
    let Some(property) = property else {
        return Ok(EventStatus::Confirmed);
    };
    if !property.params().is_empty() {
        return Err(CalendarError::MalformedCalendar);
    }
    match property.value() {
        "CONFIRMED" => Ok(EventStatus::Confirmed),
        "TENTATIVE" => Ok(EventStatus::Tentative),
        "CANCELLED" => Ok(EventStatus::Cancelled),
        _ => Err(CalendarError::MalformedCalendar),
    }
}

fn property_name(line: &str) -> Option<&str> {
    (!line.starts_with([' ', '\t']))
        .then(|| line.split([';', ':']).next())
        .flatten()
}

fn required_text(property: Option<&Property>) -> Result<String, CalendarError> {
    let property = property.ok_or(CalendarError::MalformedCalendar)?;
    if !property.params().is_empty() || property.value().trim().is_empty() {
        return Err(CalendarError::MalformedCalendar);
    }
    Ok(property.value().to_owned())
}

fn validate_utc(_name: &str, property: Option<&Property>) -> Result<(), CalendarError> {
    let property = property.ok_or(CalendarError::MalformedCalendar)?;
    if !property.params().is_empty()
        || NaiveDateTime::parse_from_str(property.value(), "%Y%m%dT%H%M%SZ").is_err()
    {
        return Err(CalendarError::MalformedCalendar);
    }
    Ok(())
}

fn validate_interval(
    start: Option<&Property>,
    end: Option<&Property>,
) -> Result<(), CalendarError> {
    let start = start.ok_or(CalendarError::MalformedCalendar)?;
    let end = end.ok_or(CalendarError::MalformedCalendar)?;
    if start.params().contains_key("TZID") || end.params().contains_key("TZID") {
        return Err(CalendarError::UnsupportedCapability);
    }
    let start_is_date = start
        .params()
        .get("VALUE")
        .is_some_and(|value| value.value() == "DATE");
    let end_is_date = end
        .params()
        .get("VALUE")
        .is_some_and(|value| value.value() == "DATE");
    if start_is_date != end_is_date {
        return Err(CalendarError::MalformedCalendar);
    }
    if start_is_date {
        let start = NaiveDate::parse_from_str(start.value(), "%Y%m%d")
            .map_err(|_| CalendarError::MalformedCalendar)?;
        let end = NaiveDate::parse_from_str(end.value(), "%Y%m%d")
            .map_err(|_| CalendarError::MalformedCalendar)?;
        return (end > start)
            .then_some(())
            .ok_or(CalendarError::MalformedCalendar);
    }
    if !start.params().is_empty() || !end.params().is_empty() {
        return Err(CalendarError::UnsupportedCapability);
    }
    let start = NaiveDateTime::parse_from_str(start.value(), "%Y%m%dT%H%M%SZ")
        .map_err(|_| CalendarError::UnsupportedCapability)?;
    let end = NaiveDateTime::parse_from_str(end.value(), "%Y%m%dT%H%M%SZ")
        .map_err(|_| CalendarError::UnsupportedCapability)?;
    (end > start)
        .then_some(())
        .ok_or(CalendarError::MalformedCalendar)
}
