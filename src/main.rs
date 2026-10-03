//! Local operator entry point for the versioned Calendar Resource Core.
//!
//! This executable trusts the operating-system user and an already-authorized
//! tenant scope. It is not a network authentication or `CalDAV` boundary.

use std::{
    env,
    fmt::Write as _,
    io::{self, Write},
    process::ExitCode,
};

use calendarweave::{CalendarPort, TenantId, postgres_store::PostgresCalendarService};

const HELP: &str = "CalendarWeave local operator (v0.1)\n\nCommands:\n  init\n  create-collection TENANT NAME\n  create-event TENANT COLLECTION FILE\n  list-events TENANT COLLECTION\n  get-event TENANT COLLECTION EVENT\n  update-event TENANT COLLECTION EVENT ETAG FILE\n\nSet CALENDARWEAVE_DATABASE_URL to a PostgreSQL connection string.\nThis is not a CalDAV or authentication service. The OS operator must\nestablish an authorized tenant scope before using these commands.\n";

/// Run one trusted local operation and return a shell-visible result.
///
/// Output errors remain failures even when the database operation completed.
/// Operators must inspect stored state before retrying an uncertain write.
fn main() -> ExitCode {
    match execute(&env::args().skip(1).collect::<Vec<_>>()) {
        Ok(output) => match io::stdout().write_all(output.as_bytes()) {
            Ok(()) => ExitCode::SUCCESS,
            Err(_) => ExitCode::FAILURE,
        },
        Err(error) => {
            let _ = writeln!(io::stderr(), "calendarweave: {error}");
            ExitCode::FAILURE
        }
    }
}

/// A parsed local command with an exact, closed positional argument shape.
///
/// Raw tenant strings are validated by the retained calendar value object.
enum Operation<'a> {
    Init,
    CreateCollection(&'a str, &'a str),
    ListEvents(&'a str, &'a str),
    CreateEvent(&'a str, &'a str, &'a str),
    GetEvent(&'a str, &'a str, &'a str),
    UpdateEvent(&'a str, &'a str, &'a str, &'a str, &'a str),
}

/// Validate the command before connecting, then invoke the existing port.
///
/// Only explicit `init` can replay schema DDL; reads and writes never migrate.
fn execute(arguments: &[String]) -> Result<String, String> {
    let arguments = arguments.iter().map(String::as_str).collect::<Vec<_>>();
    let operation = match arguments.as_slice() {
        ["--help" | "-h"] => return Ok(HELP.to_owned()),
        ["init"] => Operation::Init,
        ["create-collection", tenant, name] => Operation::CreateCollection(tenant, name),
        ["list-events", tenant, collection] => Operation::ListEvents(tenant, collection),
        ["create-event", tenant, collection, file] => {
            Operation::CreateEvent(tenant, collection, file)
        }
        ["get-event", tenant, collection, event] => Operation::GetEvent(tenant, collection, event),
        ["update-event", tenant, collection, event, etag, file] => {
            Operation::UpdateEvent(tenant, collection, event, etag, file)
        }
        _ => return Err("unknown command; use --help".to_owned()),
    };
    let database = env::var("CALENDARWEAVE_DATABASE_URL")
        .map_err(|_| "CALENDARWEAVE_DATABASE_URL is required".to_owned())?;
    validate_local_database(&database)?;
    let mut service =
        PostgresCalendarService::connect(&database).map_err(|error| calendar_error(&error))?;
    match operation {
        Operation::Init => {
            service.migrate().map_err(|error| calendar_error(&error))?;
            Ok("calendar store initialized\n".to_owned())
        }
        Operation::CreateCollection(tenant, name) => {
            let tenant = TenantId::parse(tenant).map_err(|error| calendar_error(&error))?;
            let collection = service
                .create_collection(&tenant, name)
                .map_err(|error| calendar_error(&error))?;
            Ok(format!("{}\n", collection.collection_ref))
        }
        Operation::ListEvents(tenant, collection) => {
            let tenant = TenantId::parse(tenant).map_err(|error| calendar_error(&error))?;
            let events = service
                .list_events(&tenant, collection)
                .map_err(|error| calendar_error(&error))?;
            let mut output = String::new();
            for event in events {
                writeln!(output, "{}\t{}", event.event_ref, event.etag)
                    .expect("writing into a String cannot fail");
            }
            Ok(output)
        }
        Operation::CreateEvent(tenant, collection, file) => {
            let tenant = TenantId::parse(tenant).map_err(|error| calendar_error(&error))?;
            let payload = read_calendar(file)?;
            let event = service
                .create_event(&tenant, collection, &payload)
                .map_err(|error| calendar_error(&error))?;
            Ok(format!("{}\t{}\n", event.event_ref, event.etag))
        }
        Operation::GetEvent(tenant, collection, event) => {
            let tenant = TenantId::parse(tenant).map_err(|error| calendar_error(&error))?;
            service
                .get_event(&tenant, collection, event)
                .map(|event| event.icalendar)
                .map_err(|error| calendar_error(&error))
        }
        Operation::UpdateEvent(tenant, collection, event, etag, file) => {
            let tenant = TenantId::parse(tenant).map_err(|error| calendar_error(&error))?;
            let payload = read_calendar(file)?;
            let event = service
                .update_event(&tenant, collection, event, etag, &payload)
                .map_err(|error| calendar_error(&error))?;
            Ok(format!("{}\t{}\n", event.event_ref, event.etag))
        }
    }
}

/// Reject unverified remote targets before the retained non-TLS adapter connects.
///
/// DNS names, host overrides and implicit host defaults are deliberately closed.
fn validate_local_database(database: &str) -> Result<(), String> {
    use postgres::config::Host;
    const ERROR: &str = "local database target required (UNIX socket or loopback IP)";
    let config = database
        .parse::<postgres::Config>()
        .map_err(|_| ERROR.to_owned())?;
    if !config.get_hostaddrs().is_empty() || config.get_hosts().is_empty() {
        return Err(ERROR.to_owned());
    }
    for host in config.get_hosts() {
        let local = match host {
            Host::Tcp(address) => address
                .parse::<std::net::IpAddr>()
                .is_ok_and(|ip| ip.is_loopback()),
            Host::Unix(path) => path.is_absolute(),
        };
        if !local {
            return Err(ERROR.to_owned());
        }
    }
    Ok(())
}

/// Load the operator-selected calendar file without rewriting its line endings.
///
/// I/O errors return a bounded message that contains no path or file content.
fn read_calendar(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|_| "unable to read calendar file".to_owned())
}

/// Map a domain failure to a stable operator code without storage diagnostics.
///
/// Database credentials and untrusted calendar bytes never enter this message.
fn calendar_error(error: &calendarweave::CalendarError) -> String {
    use calendarweave::CalendarError;
    match error {
        CalendarError::InvalidInput => "InvalidInput",
        CalendarError::Unauthorized => "Unauthorized",
        CalendarError::AuthorizationUnavailable => "AuthorizationUnavailable",
        CalendarError::NotFound => "NotFound",
        CalendarError::MalformedCalendar => "MalformedCalendar",
        CalendarError::UnsupportedCapability => "UnsupportedCapability",
        CalendarError::StaleRevision => "StaleRevision",
        CalendarError::StorageUnavailable => "StorageUnavailable",
    }
    .to_owned()
}

#[cfg(test)]
#[path = "../tests/support/operator_errors.rs"]
mod operator_errors;
