//! Loopback-only diagnostic HTTP transport, not an authenticated service.
use std::{
    io::{self, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    time::Duration,
};

pub(crate) fn serve(address: &str, once: bool) -> Result<(), String> {
    let address = address
        .parse::<SocketAddr>()
        .map_err(|_| "explicit loopback IP:PORT required".to_owned())?;
    if !address.ip().is_loopback() {
        return Err("explicit loopback IP:PORT required".to_owned());
    }
    let listener = TcpListener::bind(address).map_err(|_| "listener unavailable".to_owned())?;
    writeln!(
        io::stdout(),
        "LISTENING {}",
        listener.local_addr().map_err(|_| "listener unavailable")?
    )
    .map_err(|_| "listener publication failed".to_owned())?;
    io::stdout()
        .flush()
        .map_err(|_| "listener publication failed".to_owned())?;
    loop {
        let (mut stream, _) = listener
            .accept()
            .map_err(|_| "listener unavailable".to_owned())?;
        exchange(&mut stream).map_err(|_| "transport unavailable".to_owned())?;
        if once {
            return Ok(());
        }
    }
}

fn exchange(stream: &mut TcpStream) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    let mut request = Vec::new();
    let response = loop {
        let Some(remaining) = deadline.checked_duration_since(std::time::Instant::now()) else {
            break ("408 Request Timeout", "RequestTimeout\n");
        };
        stream.set_read_timeout(Some(remaining))?;
        let mut byte = [0];
        match stream.read(&mut byte) {
            Ok(0) => break ("400 Bad Request", "UnsupportedRequest\n"),
            Ok(_) => request.push(byte[0]),
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                ) =>
            {
                break ("408 Request Timeout", "RequestTimeout\n");
            }
            Err(error) => return Err(error),
        }
        if request.ends_with(b"\r\n\r\n") {
            break response_for(&request);
        }
        if request.len() >= 8192
            || (!request.windows(2).any(|pair| pair == b"\r\n") && request.len() > 2048)
        {
            break ("400 Bad Request", "UnsupportedRequest\n");
        }
    };
    let (status, body) = response;
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

fn response_for(request: &[u8]) -> (&'static str, &'static str) {
    const BAD: (&str, &str) = ("400 Bad Request", "UnsupportedRequest\n");
    let Ok(request) = std::str::from_utf8(request) else {
        return BAD;
    };
    if !request.is_ascii()
        || !request.ends_with("\r\n\r\n")
        || request.len() > 8192
        || request
            .split("\r\n")
            .next()
            .is_none_or(|line| line.len() > 2048)
    {
        return BAD;
    }
    let mut lines = request.split("\r\n");
    let fields = lines
        .next()
        .unwrap_or_default()
        .split(' ')
        .collect::<Vec<_>>();
    let [method, path, "HTTP/1.1"] = fields.as_slice() else {
        return BAD;
    };
    if *method != "GET" {
        return ("405 Method Not Allowed", "UnsupportedMethod\n");
    }
    let segments = path.split('/').collect::<Vec<_>>();
    let ["", "calendars", collection, event] = segments.as_slice() else {
        return BAD;
    };
    if !resource_segment(collection) || !resource_segment(event) {
        return BAD;
    }
    let mut host_seen = false;
    let mut names = std::collections::HashSet::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let Some((name, value)) = line.split_once(':') else {
            return BAD;
        };
        let name = name.to_ascii_lowercase();
        let value = value.trim_matches(' ');
        if value.is_empty()
            || value.bytes().any(|byte| byte.is_ascii_control())
            || !names.insert(name.clone())
        {
            return BAD;
        }
        match name.as_str() {
            "host" => host_seen = true,
            "authorization" | "user-agent" | "accept" => {}
            "connection" if value.eq_ignore_ascii_case("close") => {}
            "content-length" if value == "0" => {}
            _ => return BAD,
        }
    }
    if !host_seen {
        return BAD;
    }
    let result = calendarweave::http_bootstrap::unconfigured_get_event::<
        calendarweave::postgres_store::PostgresCalendarService,
        _,
    >(None, collection, event, || {
        let database = std::env::var("CALENDARWEAVE_DATABASE_URL")
            .map_err(|_| calendarweave::CalendarError::StorageUnavailable)?;
        super::validate_local_database(&database)
            .map_err(|_| calendarweave::CalendarError::StorageUnavailable)?;
        calendarweave::postgres_store::PostgresCalendarService::connect(&database)
    });
    debug_assert!(matches!(
        result,
        Err(calendarweave::CalendarError::AuthorizationUnavailable)
    ));
    ("503 Service Unavailable", "AuthorizationUnavailable\n")
}

fn resource_segment(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_-".contains(&byte))
}

#[cfg(test)]
#[path = "../tests/support/http_parser.rs"]
mod http_parser;
