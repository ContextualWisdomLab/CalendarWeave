//! Execute the local operator entry point rather than a simulated calendar API.

use std::{
    env,
    process::{Command, Output},
};

#[path = "support/operator_database.rs"]
mod operator_database;

fn database_cli(arguments: &[&str]) -> Output {
    let database = env::var("CALENDARWEAVE_TEST_DATABASE_URL")
        .expect("operator database test requires an explicitly disposable database");
    Command::new(env!("CARGO_BIN_EXE_calendarweave"))
        .args(arguments)
        .env("CALENDARWEAVE_DATABASE_URL", database)
        .output()
        .unwrap()
}

#[test]
fn explicit_init_and_collection_creation_work_in_separate_processes() {
    let initialized = database_cli(&["init"]);
    assert!(
        initialized.status.success(),
        "{}",
        String::from_utf8_lossy(&initialized.stderr)
    );
    assert_eq!(initialized.stdout, b"calendar store initialized\n");
    assert!(database_cli(&["init"]).status.success());
    let created = database_cli(&["create-collection", "cli-tenant", "CLI calendar"]);
    assert!(
        created.status.success(),
        "{}",
        String::from_utf8_lossy(&created.stderr)
    );
    let reference = String::from_utf8(created.stdout).unwrap();
    assert!(reference.starts_with("cal_"));
    assert_eq!(reference.trim().len(), 36);
    let listed = database_cli(&["list-events", "cli-tenant", reference.trim()]);
    assert!(listed.status.success());
    assert!(listed.stdout.is_empty());
}

#[test]
fn events_round_trip_and_conditional_update_through_real_processes() {
    let initialized = database_cli(&["init"]);
    assert!(initialized.status.success());
    let collection = database_cli(&["create-collection", "cli-crud", "CRUD calendar"]);
    assert!(collection.status.success());
    let collection = String::from_utf8(collection.stdout).unwrap();
    let collection = collection.trim();
    let root = env::temp_dir().join(format!("cw-cli-{}", uuid::Uuid::new_v4().simple()));
    std::fs::create_dir(&root).unwrap();
    let file = root.join("event.ics");
    let payload = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//CalendarWeave//Operator Test//EN\r\nBEGIN:VEVENT\r\nUID:cli-crud@example.test\r\nDTSTAMP:20261003T000000Z\r\nDTSTART;TZID=Asia/Seoul:20261004T090000\r\nDURATION:PT1H\r\nSUMMARY:Local operator test\r\nCLASS:PRIVATE\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
    std::fs::write(&file, payload).unwrap();
    let file = file.to_str().unwrap();
    let created = database_cli(&["create-event", "cli-crud", collection, file]);
    assert!(
        created.status.success(),
        "{}",
        String::from_utf8_lossy(&created.stderr)
    );
    let created = String::from_utf8(created.stdout).unwrap();
    let (event, etag) = created.trim().split_once('\t').unwrap();
    assert!(event.starts_with("evt_"));
    assert_eq!(etag, format!("\"{event}:1\""));
    let duplicate = database_cli(&["create-event", "cli-crud", collection, file]);
    assert_eq!(duplicate.stdout, created.as_bytes());
    let fetched = database_cli(&["get-event", "cli-crud", collection, event]);
    assert!(fetched.status.success());
    assert_eq!(fetched.stdout, payload.as_bytes());
    let listed = database_cli(&["list-events", "cli-crud", collection]);
    assert_eq!(listed.stdout, created.as_bytes());
    let outsider = database_cli(&["get-event", "cli-outsider", collection, event]);
    assert!(!outsider.status.success());
    assert!(outsider.stdout.is_empty());
    assert_eq!(outsider.stderr, b"calendarweave: NotFound\n");
    let changed = payload.replace("Local operator test", "Updated local event");
    std::fs::write(file, &changed).unwrap();
    let updated = database_cli(&["update-event", "cli-crud", collection, event, etag, file]);
    assert!(updated.status.success());
    assert_eq!(
        updated.stdout,
        format!("{event}\t\"{event}:2\"\n").as_bytes()
    );
    let stale = database_cli(&["update-event", "cli-crud", collection, event, etag, file]);
    assert!(!stale.status.success());
    assert_eq!(stale.stderr, b"calendarweave: StaleRevision\n");
    assert_eq!(
        database_cli(&["get-event", "cli-crud", collection, event]).stdout,
        changed.as_bytes()
    );
    for (arguments, error) in [
        (vec!["create-collection", "!", "name"], "InvalidInput"),
        (vec!["list-events", "!", collection], "InvalidInput"),
        (vec!["create-event", "!", collection, file], "InvalidInput"),
        (vec!["get-event", "!", collection, event], "InvalidInput"),
        (
            vec!["update-event", "!", collection, event, etag, file],
            "InvalidInput",
        ),
        (
            vec!["create-event", "cli-crud", collection, "/missing-file"],
            "unable to read calendar file",
        ),
        (
            vec![
                "update-event",
                "cli-crud",
                collection,
                event,
                etag,
                "/missing-file",
            ],
            "unable to read calendar file",
        ),
    ] {
        let output = database_cli(&arguments);
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert_eq!(
            output.stderr,
            format!("calendarweave: {error}\n").as_bytes()
        );
    }
    std::fs::write(file, "malformed").unwrap();
    assert_eq!(
        database_cli(&["create-event", "cli-crud", collection, file]).stderr,
        b"calendarweave: MalformedCalendar\n"
    );
    let unsupported = payload.replace(
        "SUMMARY:Local operator test",
        "SUMMARY:Local operator test\r\nRRULE:FREQ=DAILY",
    );
    std::fs::write(file, unsupported).unwrap();
    assert_eq!(
        database_cli(&["create-event", "cli-crud", collection, file]).stderr,
        b"calendarweave: UnsupportedCapability\n"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn errors_are_nonzero_and_do_not_publish_credentials_or_mutate_schema() {
    for args in [
        vec![],
        vec!["serve"],
        vec!["init", "extra"],
        vec!["create-event", "tenant"],
    ] {
        let output = cli(&args);
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
    }
    assert!(cli(&["-h"]).status.success());
    let missing = cli(&["init"]);
    assert!(!missing.status.success());
    assert_eq!(
        missing.stderr,
        b"calendarweave: CALENDARWEAVE_DATABASE_URL is required\n"
    );
    for database in [
        "not a database connection credential-sentinel",
        "user=operator password=credential-sentinel",
        "host=remote.example.test user=operator password=credential-sentinel",
        "host=127.0.0.1 hostaddr=192.0.2.4 user=operator password=credential-sentinel",
        "host=localhost user=operator password=credential-sentinel",
        "host=/local/socket hostaddr=192.0.2.4 user=operator password=credential-sentinel",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_calendarweave"))
            .arg("init")
            .env("CALENDARWEAVE_DATABASE_URL", database)
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(!String::from_utf8_lossy(&output.stderr).contains("credential-sentinel"));
        assert!(String::from_utf8_lossy(&output.stderr).contains("local database target required"));
    }
    let unavailable = Command::new(env!("CARGO_BIN_EXE_calendarweave"))
        .arg("init")
        .env(
            "CALENDARWEAVE_DATABASE_URL",
            "host=127.0.0.1 port=1 user=operator password=credential-sentinel connect_timeout=1",
        )
        .output()
        .unwrap();
    assert!(!unavailable.status.success());
    assert_eq!(unavailable.stderr, b"calendarweave: StorageUnavailable\n");
    let unavailable_v6 = Command::new(env!("CARGO_BIN_EXE_calendarweave"))
        .arg("init")
        .env(
            "CALENDARWEAVE_DATABASE_URL",
            "host=::1 port=1 user=operator connect_timeout=1",
        )
        .output()
        .unwrap();
    assert_eq!(
        unavailable_v6.stderr,
        b"calendarweave: StorageUnavailable\n"
    );
}

#[test]
fn uri_and_keyword_targets_use_authoritative_postgres_host_parsing() {
    use postgres::config::Host;

    // These are parser/validator tests: none of these rejected targets may
    // reach DNS or a remote connection attempt, even if a local host comes first.
    for database in [
        "postgresql://operator:credential-sentinel@127.0.0.1/db?host=remote.example.test",
        "postgresql://operator:credential-sentinel@127.0.0.1/db?host=localhost",
        "postgresql://operator:credential-sentinel@127.0.0.1/db?hostaddr=192.0.2.4",
        "postgresql://operator:credential-sentinel@127.0.0.1/db?hostaddr=127.0.0.1",
        "postgresql://operator:credential-sentinel@127.0.0.1,192.0.2.4/db",
        "postgresql://operator:credential-sentinel@192.0.2.4,127.0.0.1/db",
        "postgresql://operator:credential-sentinel@%2Flocal%2Fsocket,remote.example.test/db",
        "postgresql://operator:credential-sentinel@%2Flocal%2Fsocket/db?hostaddr=192.0.2.4",
        "host=127.0.0.1,192.0.2.4 user=operator password=credential-sentinel",
        "host=192.0.2.4,127.0.0.1 user=operator password=credential-sentinel",
        "host=/local/socket,remote.example.test user=operator password=credential-sentinel",
    ] {
        let parsed = database
            .parse::<postgres::Config>()
            .unwrap_or_else(|_| panic!("target fixture must parse before testing rejection"));
        assert!(!parsed.get_hosts().is_empty());
        let output = Command::new(env!("CARGO_BIN_EXE_calendarweave"))
            .arg("init")
            .env("CALENDARWEAVE_DATABASE_URL", database)
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert_eq!(
            output.stderr,
            b"calendarweave: local database target required (UNIX socket or loopback IP)\n"
        );
    }
    // Valid local URIs pass validation, then fail only at an unavailable local
    // endpoint. Port 1 cannot touch the owned server or any real database.
    for database in [
        "postgresql://operator:credential-sentinel@127.0.0.1:1/db?connect_timeout=1",
        "postgresql://operator:credential-sentinel@[::1]:1/db?connect_timeout=1",
        "postgresql://operator:credential-sentinel@127.0.0.1:1/db?host=127.0.0.2&connect_timeout=1",
        "postgresql://operator:credential-sentinel@%2Fcalendarweave-nonexistent-socket:1/db?connect_timeout=1",
    ] {
        let parsed = database
            .parse::<postgres::Config>()
            .unwrap_or_else(|_| panic!("local URI fixture must parse"));
        assert!(parsed.get_hostaddrs().is_empty());
        assert!(parsed.get_hosts().iter().all(|host| {
            match host {
                Host::Tcp(host) => host
                    .parse::<std::net::IpAddr>()
                    .is_ok_and(|address| address.is_loopback()),
                #[cfg(unix)]
                Host::Unix(path) => path.is_absolute(),
            }
        }));
        let output = Command::new(env!("CARGO_BIN_EXE_calendarweave"))
            .arg("init")
            .env("CALENDARWEAVE_DATABASE_URL", database)
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert_eq!(output.stderr, b"calendarweave: StorageUnavailable\n");
    }
}

// Boolean comparisons deliberately keep Config, DSNs and credentials out of
// panic diagnostics, including when a non-credential field differs.
fn assert_same_database_config(actual: &postgres::Config, expected: &postgres::Config) {
    macro_rules! same {
        ($getter:ident) => {
            assert!(
                actual.$getter() == expected.$getter(),
                concat!(stringify!($getter), " must survive the fixture roundtrip")
            );
        };
    }
    same!(get_hosts);
    same!(get_ports);
    same!(get_hostaddrs);
    same!(get_user);
    same!(get_password);
    same!(get_dbname);
    same!(get_options);
    same!(get_application_name);
    same!(get_ssl_mode);
    same!(get_ssl_negotiation);
    same!(get_connect_timeout);
    same!(get_tcp_user_timeout);
    same!(get_keepalives);
    same!(get_keepalives_idle);
    same!(get_keepalives_interval);
    same!(get_keepalives_retries);
    same!(get_target_session_attrs);
    same!(get_channel_binding);
    same!(get_load_balance_hosts);
}

// These fixture checks intentionally use boolean assertions so a failure never
// publishes connection identity, options, credentials or subprocess output.
#[allow(clippy::manual_assert_eq)]
#[cfg(unix)]
#[test]
fn database_forms_roundtrip_multiple_absolute_socket_hosts() {
    let original = "host='/fixture/first socket,/fixture/second%socket' port=55573,55574 user='fixture user' dbname='fixture database' password='synthetic\\'\\\\%&=,+ secret' options='-c fixture=\\'\\\\%&=,+ value' application_name='fixture\\'\\\\%&=,+ 앱' connect_timeout=3 tcp_user_timeout=4 keepalives=0 keepalives_idle=5 keepalives_interval=6 keepalives_retries=7 sslmode=disable sslnegotiation=direct target_session_attrs=read-only channel_binding=disable load_balance_hosts=random"
        .parse::<postgres::Config>()
        .unwrap_or_else(|_| panic!("synthetic socket configuration must parse"));
    assert!(original.get_hosts().len() == 2);
    assert!(original.get_ports() == [55573, 55574]);
    for database in operator_database::database_forms(&original) {
        let roundtrip = database
            .parse::<postgres::Config>()
            .unwrap_or_else(|_| panic!("serialized socket configuration must parse"));
        assert_same_database_config(&roundtrip, &original);
    }
}

// Boolean diagnostics are part of this credential-safe fixture contract.
#[allow(clippy::manual_assert_eq)]
#[test]
fn database_forms_roundtrip_loopback_host_address_and_individual_port_lists() {
    // Both source grammars feed the real parser. Each position deliberately has
    // a distinct host/address/port so collapsing, sorting or pairing drift fails.
    for source in [
        "host=127.0.0.2,::1,127.0.0.1 hostaddr=127.0.0.2,::1,127.0.0.1 port=55571,55572,55573 user=fixture dbname=fixture connect_timeout=8 tcp_user_timeout=9 keepalives=1 keepalives_idle=10 keepalives_interval=11 keepalives_retries=12 sslmode=require sslnegotiation=postgres target_session_attrs=read-write channel_binding=require load_balance_hosts=disable",
        "postgresql://fixture@127.0.0.2:55571,[::1]:55572,127.0.0.1:55573/fixture?hostaddr=127.0.0.2%2C%3A%3A1%2C127.0.0.1&connect_timeout=8&tcp_user_timeout=9&keepalives=1&keepalives_idle=10&keepalives_interval=11&keepalives_retries=12&sslmode=require&sslnegotiation=postgres&target_session_attrs=read-write&channel_binding=require&load_balance_hosts=disable",
    ] {
        let mut original = source
            .parse::<postgres::Config>()
            .unwrap_or_else(|_| panic!("synthetic loopback configuration must parse"));
        original
            .password("synthetic'\\%&=,+ secret 앱")
            .options("-c fixture='\\%&=,+ existing value 앱")
            .application_name("fixture'\\%&=,+ name 앱");
        assert!(original.get_hosts().len() == 3);
        assert!(original.get_hostaddrs().len() == 3);
        assert!(original.get_ports() == [55571, 55572, 55573]);
        for database in operator_database::database_forms(&original) {
            let roundtrip = database
                .parse::<postgres::Config>()
                .unwrap_or_else(|_| panic!("serialized loopback configuration must parse"));
            assert_same_database_config(&roundtrip, &original);
            let read_only = operator_database::read_only_database(&database)
                .parse::<postgres::Config>()
                .unwrap_or_else(|_| panic!("read-only loopback configuration must parse"));
            let mut expected = original.clone();
            expected.options(&format!(
                "{} -c default_transaction_read_only=on",
                original.get_options().unwrap()
            ));
            assert_same_database_config(&read_only, &expected);
        }
    }
}

// Keep server/CLI failure diagnostics bounded rather than printing their values.
#[allow(clippy::manual_assert_eq)]
#[cfg(unix)]
#[test]
fn duplicate_owned_host_forms_connect_before_read_only_migration_rejection() {
    use postgres::config::Host;

    let mut original = env::var("CALENDARWEAVE_TEST_DATABASE_URL")
        .unwrap_or_else(|_| panic!("owned database must be explicitly configured"))
        .parse::<postgres::Config>()
        .unwrap_or_else(|_| panic!("owned database configuration must parse"));
    // Duplicate only the explicitly configured target: the local owned UNIX
    // fixture or CI's explicit loopback IP, never a default or inferred host.
    assert!(original.get_hosts().len() == 1);
    assert!(original.get_ports().len() == 1);
    match original.get_hosts()[0].clone() {
        Host::Unix(path) => {
            assert!(path.is_absolute());
            original.host_path(path);
        }
        Host::Tcp(host) => {
            assert!(
                host.parse::<std::net::IpAddr>()
                    .is_ok_and(|ip| ip.is_loopback())
            );
            original.host(&host);
        }
    }
    let port = original.get_ports()[0];
    original.port(port);
    assert!(original.get_hosts().len() == 2);
    assert!(original.get_ports().len() == 2);
    original.options(&format!(
        "{} -c statement_timeout=5000",
        original.get_options().unwrap_or("")
    ));
    let mut baseline = original
        .connect(postgres::NoTls)
        .unwrap_or_else(|_| panic!("original duplicate socket configuration must connect"));
    let intended_name: String = baseline
        .query_one("SELECT current_database()", &[])
        .unwrap_or_else(|_| panic!("owned database identity can be checked"))
        .get(0);
    for database in operator_database::database_forms(&original) {
        let roundtrip = database
            .parse::<postgres::Config>()
            .unwrap_or_else(|_| panic!("serialized duplicate socket configuration must parse"));
        assert_same_database_config(&roundtrip, &original);
        let mut connection = roundtrip
            .connect(postgres::NoTls)
            .unwrap_or_else(|_| panic!("serialized duplicate socket configuration must connect"));
        let name: String = connection
            .query_one("SELECT current_database()", &[])
            .unwrap_or_else(|_| panic!("connected database identity can be checked"))
            .get(0);
        assert!(
            name == intended_name,
            "duplicate socket forms must retain database identity"
        );
        let read_only = operator_database::read_only_database(&database);
        let configured = read_only
            .parse::<postgres::Config>()
            .unwrap_or_else(|_| panic!("read-only duplicate socket configuration must parse"));
        let mut expected = original.clone();
        expected.options(&format!(
            "{} -c default_transaction_read_only=on",
            original.get_options().unwrap()
        ));
        assert_same_database_config(&configured, &expected);
        let mut connection = configured
            .connect(postgres::NoTls)
            .unwrap_or_else(|_| panic!("read-only duplicate socket configuration must connect"));
        let row = connection
            .query_one("SELECT current_database(), current_setting('default_transaction_read_only'), current_setting('statement_timeout')", &[])
            .unwrap_or_else(|_| panic!("read-only database identity and existing options can be checked"));
        assert!(row.get::<_, String>(0) == intended_name);
        assert!(row.get::<_, String>(1) == "on");
        assert!(row.get::<_, String>(2) == "5s");
        let rejected = Command::new(env!("CARGO_BIN_EXE_calendarweave"))
            .arg("init")
            .env("CALENDARWEAVE_DATABASE_URL", read_only)
            .output()
            .unwrap_or_else(|_| panic!("owned read-only CLI can be executed"));
        assert!(!rejected.status.success());
        assert!(rejected.stdout.is_empty());
        assert!(rejected.stderr == b"calendarweave: StorageUnavailable\n");
    }
}

#[test]
fn storage_and_output_failures_are_nonzero() {
    let original = env::var("CALENDARWEAVE_TEST_DATABASE_URL")
        .unwrap()
        .parse::<postgres::Config>()
        .unwrap_or_else(|_| panic!("valid owned database configuration"));
    let mut original_connection = original
        .connect(postgres::NoTls)
        .unwrap_or_else(|_| panic!("owned database must actually connect"));
    let intended_name: String = original_connection
        .query_one("SELECT current_database()", &[])
        .unwrap_or_else(|_| panic!("owned database name can be checked"))
        .get(0);
    // Always exercise both grammars, even when the environment supplies only one.
    // Also retain the original form to cover URI authority without a query.
    for database in operator_database::database_forms(&original)
        .into_iter()
        .chain([env::var("CALENDARWEAVE_TEST_DATABASE_URL").unwrap()])
    {
        let read_only = operator_database::read_only_database(&database);
        let configured = read_only
            .parse::<postgres::Config>()
            .unwrap_or_else(|_| panic!("valid read-only fixture configuration"));
        assert!(
            configured.get_dbname() == original.get_dbname(),
            "read-only fixture must preserve the intended database name"
        );
        assert!(
            configured.get_hosts() == original.get_hosts(),
            "read-only fixture must preserve hosts"
        );
        assert!(
            configured.get_ports() == original.get_ports(),
            "read-only fixture must preserve ports"
        );
        assert!(
            configured.get_user() == original.get_user(),
            "read-only fixture must preserve user"
        );
        assert!(
            configured.get_password() == original.get_password(),
            "read-only fixture must preserve credentials without publishing them"
        );
        assert!(
            configured
                .get_options()
                .is_some_and(|options| options.contains("default_transaction_read_only=on")),
            "read-only fixture must carry server options"
        );
        let mut expected = original.clone();
        expected.options(&format!(
            "{} -c default_transaction_read_only=on",
            original.get_options().unwrap_or("")
        ));
        assert_same_database_config(&configured, &expected);
        let mut connected = configured
            .connect(postgres::NoTls)
            .unwrap_or_else(|_| panic!("read-only fixture must actually connect"));
        let setting: String = connected
            .query_one("SHOW default_transaction_read_only", &[])
            .unwrap_or_else(|_| panic!("read-only setting can be read"))
            .get(0);
        assert_eq!(setting, "on");
        let name: String = connected
            .query_one("SELECT current_database()", &[])
            .unwrap_or_else(|_| panic!("connected database can be checked"))
            .get(0);
        assert!(
            name == intended_name,
            "read-only fixture must connect to intended database"
        );
        // A live read-only session and correct identity precede the CLI assertion:
        // StorageUnavailable alone cannot distinguish failed connect from failed DDL.
        let rejected_migration = Command::new(env!("CARGO_BIN_EXE_calendarweave"))
            .arg("init")
            .env("CALENDARWEAVE_DATABASE_URL", read_only)
            .output()
            .unwrap();
        assert!(!rejected_migration.status.success());
        assert!(rejected_migration.stdout.is_empty());
        assert_eq!(
            rejected_migration.stderr,
            b"calendarweave: StorageUnavailable\n"
        );
    }
}

#[test]
fn ordinary_storage_and_help_output_failures_are_nonzero() {
    // This test owns its schema prerequisite; sibling tests may run in any order.
    assert!(database_cli(&["init"]).status.success());
    assert_eq!(
        database_cli(&["create-collection", "cli-failure", " "]).stderr,
        b"calendarweave: InvalidInput\n"
    );
    assert_eq!(
        database_cli(&["list-events", "cli-failure", "cal_missing"]).stderr,
        b"calendarweave: NotFound\n"
    );
    #[cfg(unix)]
    {
        use std::{os::fd::OwnedFd, os::unix::net::UnixStream, process::Stdio};
        let (writer, reader) = UnixStream::pair().unwrap();
        drop(reader);
        let output = Command::new(env!("CARGO_BIN_EXE_calendarweave"))
            .arg("--help")
            .stdout(Stdio::from(OwnedFd::from(writer)))
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(output.stderr.is_empty());
    }
}

#[cfg(unix)]
#[test]
fn committed_collection_survives_broken_stdout_and_documentation_warns_against_retry() {
    use std::{os::fd::OwnedFd, os::unix::net::UnixStream, process::Stdio};

    assert!(database_cli(&["init"]).status.success());
    let database = env::var("CALENDARWEAVE_TEST_DATABASE_URL").unwrap();
    let mut connection = postgres::Client::connect(&database, postgres::NoTls)
        .unwrap_or_else(|_| panic!("owned database must actually connect"));
    let tenant = format!("cli-output-{}", uuid::Uuid::new_v4().simple());
    let name = "Committed despite broken stdout";
    let count = |connection: &mut postgres::Client| -> i64 {
        connection.query_one(
            "SELECT count(*) FROM calendar_collection WHERE tenant_reference = $1 AND display_name = $2",
            &[&tenant, &name],
        ).unwrap_or_else(|_| panic!("committed rows can be checked")).get(0)
    };
    assert_eq!(count(&mut connection), 0);
    let (writer, reader) = UnixStream::pair().unwrap();
    drop(reader);
    // Invoke the write exactly once. A closed peer fails stdout after SQL commit.
    let output = Command::new(env!("CARGO_BIN_EXE_calendarweave"))
        .args(["create-collection", &tenant, name])
        .env("CALENDARWEAVE_DATABASE_URL", database)
        .stdout(Stdio::from(OwnedFd::from(writer)))
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(count(&mut connection), 1);
    let row = connection.query_one(
        "SELECT collection_reference FROM calendar_collection WHERE tenant_reference = $1 AND display_name = $2",
        &[&tenant, &name],
    ).unwrap_or_else(|_| panic!("committed collection can be read back"));
    let reference: String = row.get(0);
    assert!(reference.starts_with("cal_"));
    assert_eq!(reference.len(), 36);

    let documentation = include_str!("../docs/local-operator-quickstart.md");
    assert!(
        !documentation.contains("Failures exit nonzero with a bounded code on stderr"),
        "failure documentation must not promise stderr for output failures"
    );
    assert!(documentation.contains("Ordinary application failures exit nonzero"));
    assert!(documentation.contains("partial stdout"));
    assert!(documentation.contains("no stderr"));
    assert!(documentation.contains("Do not blindly repeat"));
}

fn cli(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_calendarweave"))
        .args(arguments)
        .env_remove("CALENDARWEAVE_DATABASE_URL")
        .output()
        .expect("Cargo can execute the CLI contract")
}

#[test]
fn help_exposes_a_real_local_operator_without_claiming_a_network_service() {
    let output = cli(&["--help"]);
    assert!(
        output.status.success(),
        "operator executable is missing: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let help = String::from_utf8(output.stdout).unwrap();
    assert!(help.contains("create-collection TENANT NAME"));
    assert!(help.contains("create-event TENANT COLLECTION FILE"));
    assert!(help.contains("get-event TENANT COLLECTION EVENT"));
    assert!(help.contains("local operator"));
    assert!(help.contains("not a CalDAV or authentication service"));
}
