//! Infrastructure bootstrap while identity and resource-policy producers are absent.
//!
//! This is deliberately not a token verifier or a configured authorization path.
//! [`ExternalIdentity::parse`] validates bounded strings only. A caller passing
//! that value is responsible for obtaining actual verified evidence elsewhere.
use crate::{CalendarError, CalendarEvent, CalendarPort, admission::ExternalIdentity};

/// Refuse a single-event read before invoking the lazy calendar-store factory.
///
/// Both an absent wire identity and trusted in-process identity remain unavailable
/// because no concrete RP verifier or resource-aware runtime has been configured.
/// This function never derives tenant scope from a caller or an HTTP header.
///
/// # Errors
///
/// Always returns [`CalendarError::AuthorizationUnavailable`]. There is no
/// authorized success path in this bootstrap slice; this is not service parity.
pub fn unconfigured_get_event<P, F>(
    _verified_identity: Option<&ExternalIdentity>,
    _collection_ref: &str,
    _event_ref: &str,
    _acquire_calendar: F,
) -> Result<CalendarEvent, CalendarError>
where
    P: CalendarPort,
    F: FnOnce() -> Result<P, CalendarError>,
{
    Err(CalendarError::AuthorizationUnavailable)
}
