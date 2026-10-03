use super::calendar_error;
use calendarweave::CalendarError;

#[test]
fn every_bounded_domain_error_has_a_secret_free_operator_code() {
    for (error, expected) in [
        (CalendarError::InvalidInput, "InvalidInput"),
        (CalendarError::Unauthorized, "Unauthorized"),
        (
            CalendarError::AuthorizationUnavailable,
            "AuthorizationUnavailable",
        ),
        (CalendarError::NotFound, "NotFound"),
        (CalendarError::MalformedCalendar, "MalformedCalendar"),
        (
            CalendarError::UnsupportedCapability,
            "UnsupportedCapability",
        ),
        (CalendarError::StaleRevision, "StaleRevision"),
        (CalendarError::StorageUnavailable, "StorageUnavailable"),
    ] {
        assert_eq!(calendar_error(&error), expected);
    }
}
