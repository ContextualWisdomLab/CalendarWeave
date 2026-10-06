//! Parser tests live outside infrastructure production source.
use super::response_for;

#[test]
fn oversized_request_line_and_headers_fail_closed() {
    let line = format!(
        "GET /calendars/{}/evt HTTP/1.1\r\nHost: localhost\r\n\r\n",
        "a".repeat(2048)
    );
    assert_eq!(response_for(line.as_bytes()).0, "400 Bad Request");
    let headers = format!(
        "GET /calendars/cal/evt HTTP/1.1\r\nHost: localhost\r\nAuthorization: {}\r\n\r\n",
        "a".repeat(8192)
    );
    assert_eq!(response_for(headers.as_bytes()).0, "400 Bad Request");
}

#[test]
fn unsupported_method_is_not_admitted_as_a_read() {
    assert_eq!(
        response_for(b"PUT /calendars/cal_exact/evt_exact HTTP/1.1\r\nHost: localhost\r\n\r\n"),
        ("405 Method Not Allowed", "UnsupportedMethod\n")
    );
}

#[test]
fn unsupported_request_profile_fails_closed() {
    for request in [
        b"GET /other HTTP/1.1\r\nHost: localhost\r\n\r\n".as_slice(),
        b"GET /calendars/cal_exact/evt_exact?tenant=mine HTTP/1.1\r\nHost: localhost\r\n\r\n",
        b"GET /calendars/cal_exact/%65vt HTTP/1.1\r\nHost: localhost\r\n\r\n",
        b"GET /calendars/../evt_exact HTTP/1.1\r\nHost: localhost\r\n\r\n",
        b"GET /calendars/cal_exact/evt_exact HTTP/2.0\r\nHost: localhost\r\n\r\n",
        b"GET /calendars/cal_exact/evt_exact HTTP/1.1\r\n\r\n",
        b"GET /calendars/cal_exact/evt_exact HTTP/1.1\r\nHost: one\r\nHost: two\r\n\r\n",
        b"GET /calendars/cal_exact/evt_exact HTTP/1.1\r\nHost: localhost\r\nContent-Length: 1\r\n\r\nx",
        b"GET /calendars/cal_exact/evt_exact HTTP/1.1\r\nHost: localhost\r\nTransfer-Encoding: chunked\r\n\r\n",
        b"GET /calendars/cal_exact/evt_exact HTTP/1.1\r\nHost: localhost\r\nUpgrade: h2c\r\n\r\n",
        b"GET /calendars/cal_exact/evt_exact HTTP/1.1\r\nHost: localhost\r\n folded: no\r\n\r\n",
        b"GET /calendars/cal_exact/evt_exact HTTP/1.1\r\nHost: localhost\r\nX-Tenant: mine\r\n\r\n",
    ] {
        assert_eq!(response_for(request), ("400 Bad Request", "UnsupportedRequest\n"));
    }
}
