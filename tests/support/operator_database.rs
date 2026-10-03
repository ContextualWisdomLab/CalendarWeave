//! Test-only connection fixtures using the same parser as the operator.

use std::fmt::Write as _;

use postgres::{
    Config,
    config::{ChannelBinding, Host, LoadBalanceHosts, SslMode, SslNegotiation, TargetSessionAttrs},
};

/// Serialize the parsed identity into both supported DSN grammars. Other
/// effective connection settings are retained, never inferred from string parts.
pub fn database_forms(config: &Config) -> [String; 2] {
    let mut fields = Vec::new();
    for (key, value) in [
        ("user", config.get_user()),
        ("dbname", config.get_dbname()),
        ("options", config.get_options()),
        ("application_name", config.get_application_name()),
    ] {
        if let Some(value) = value {
            fields.push((key, value.to_owned()));
        }
    }
    if let Some(password) = config.get_password() {
        let password = std::str::from_utf8(password)
            .unwrap_or_else(|_| panic!("parsed test DSN password is UTF-8"));
        fields.push(("password", password.to_owned()));
    }
    let hosts = config
        .get_hosts()
        .iter()
        .map(|host| match host {
            Host::Tcp(host) => host.clone(),
            #[cfg(unix)]
            Host::Unix(path) => path
                .to_str()
                .expect("parsed test socket path is UTF-8")
                .to_owned(),
        })
        .collect::<Vec<_>>();
    // The pinned URI parser appends one host per query field on UNIX; it does
    // not split a decoded comma list. Repeated host fields also append in the
    // keyword grammar, preserving the same host order and port pairing.
    fields.extend(hosts.into_iter().map(|host| ("host", host)));
    if !config.get_hostaddrs().is_empty() {
        fields.push((
            "hostaddr",
            config
                .get_hostaddrs()
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(","),
        ));
    }
    if !config.get_ports().is_empty() {
        fields.push((
            "port",
            config
                .get_ports()
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(","),
        ));
    }
    for (key, duration) in [
        ("connect_timeout", config.get_connect_timeout().copied()),
        ("tcp_user_timeout", config.get_tcp_user_timeout().copied()),
        ("keepalives_idle", Some(config.get_keepalives_idle())),
        ("keepalives_interval", config.get_keepalives_interval()),
    ] {
        if let Some(duration) = duration {
            // libpq grammar represents these durations as whole seconds.
            fields.push((key, duration.as_secs().to_string()));
        }
    }
    if let Some(retries) = config.get_keepalives_retries() {
        fields.push(("keepalives_retries", retries.to_string()));
    }
    fields.push(("keepalives", u8::from(config.get_keepalives()).to_string()));
    append_transport_fields(config, &mut fields);
    serialize_forms(&fields)
}

fn append_transport_fields(config: &Config, fields: &mut Vec<(&str, String)>) {
    fields.push((
        "sslmode",
        match config.get_ssl_mode() {
            SslMode::Disable => "disable",
            SslMode::Prefer => "prefer",
            SslMode::Require => "require",
            _ => panic!("unsupported test TLS mode"),
        }
        .to_owned(),
    ));
    fields.push((
        "sslnegotiation",
        match config.get_ssl_negotiation() {
            SslNegotiation::Direct => "direct",
            SslNegotiation::Postgres => "postgres",
            _ => panic!("unsupported test TLS negotiation"),
        }
        .to_owned(),
    ));
    fields.push((
        "target_session_attrs",
        match config.get_target_session_attrs() {
            TargetSessionAttrs::Any => "any",
            TargetSessionAttrs::ReadWrite => "read-write",
            TargetSessionAttrs::ReadOnly => "read-only",
            _ => panic!("unsupported test session attributes"),
        }
        .to_owned(),
    ));
    fields.push((
        "channel_binding",
        match config.get_channel_binding() {
            ChannelBinding::Disable => "disable",
            ChannelBinding::Prefer => "prefer",
            ChannelBinding::Require => "require",
            _ => panic!("unsupported test channel binding"),
        }
        .to_owned(),
    ));
    fields.push((
        "load_balance_hosts",
        match config.get_load_balance_hosts() {
            LoadBalanceHosts::Disable => "disable",
            LoadBalanceHosts::Random => "random",
            _ => panic!("unsupported test host balancing"),
        }
        .to_owned(),
    ));
}

fn serialize_forms(fields: &[(&str, String)]) -> [String; 2] {
    let keyword = fields
        .iter()
        .map(|(key, value)| {
            format!(
                "{key}='{}'",
                value.replace('\\', "\\\\").replace('\'', "\\'")
            )
        })
        .collect::<Vec<_>>()
        .join(" ");
    let uri = format!(
        "postgresql:///?{}",
        fields
            .iter()
            .map(|(key, value)| format!("{key}={}", percent_encode(value)))
            .collect::<Vec<_>>()
            .join("&")
    );
    [keyword, uri]
}

/// libpq URI query encoding; parsing remains `postgres::Config`'s responsibility.
fn percent_encode(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || b"-._~".contains(&byte) {
            encoded.push(char::from(byte));
        } else {
            write!(encoded, "%{byte:02X}").expect("writing to a string cannot fail");
        }
    }
    encoded
}

pub fn read_only_database(database: &str) -> String {
    let config = database
        .parse::<Config>()
        .unwrap_or_else(|_| panic!("valid owned database configuration"));
    let options = format!(
        "{} -c default_transaction_read_only=on",
        config.get_options().unwrap_or("")
    );
    if database.starts_with("postgres://") || database.starts_with("postgresql://") {
        let separator = if database.contains('?') { '&' } else { '?' };
        format!("{database}{separator}options={}", percent_encode(&options))
    } else {
        format!(
            "{database} options='{}'",
            options.replace('\\', "\\\\").replace('\'', "\\'")
        )
    }
}
