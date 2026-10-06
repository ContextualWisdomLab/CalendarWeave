//! First standalone transport contracts; no database or identity credentials.
use std::process::Command;

#[test]
fn help_exposes_explicit_loopback_serve() {
    let output = Command::new(env!("CARGO_BIN_EXE_calendarweave"))
        .arg("--help")
        .env_remove("CALENDARWEAVE_DATABASE_URL")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("serve LOOPBACK_IP:PORT")
    );
}

#[test]
fn loopback_wire_default_and_idle_deadline_are_executable() {
    for mode in ["get", "idle"] {
        let output = Command::new("python3")
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .arg("-B")
            .arg("-W")
            .arg("error")
            .arg(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/support/http_wire_probe.py"
            ))
            .arg(env!("CARGO_BIN_EXE_calendarweave"))
            .arg(mode)
            .env_remove("CALENDARWEAVE_TEST_DATABASE_URL")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "wire probe failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        println!(
            "wire mode={mode}: {}",
            String::from_utf8(output.stdout).unwrap()
        );
    }
}

#[test]
fn actor_free_fixture_custody_regressions_are_distributed() {
    let output = Command::new("python3")
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .arg("-B")
        .arg("-W")
        .arg("error")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/test_http_wire_custody.py"
        ))
        .arg("-v")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "custody regressions failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn explicit_listener_rejects_dns_wildcard_remote_and_bad_shapes() {
    for address in [
        "localhost:0",
        "0.0.0.0:0",
        "[::]:0",
        "192.0.2.1:0",
        "[::ffff:127.0.0.1]:0",
        "127.0.0.1",
        "127.0.0.1:99999",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_calendarweave"))
            .args(["serve", address, "--once"])
            .env(
                "CALENDARWEAVE_DATABASE_URL",
                "invalid-synthetic-do-not-connect",
            )
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert_eq!(
            output.stderr,
            b"calendarweave: explicit loopback IP:PORT required\n"
        );
    }
}
