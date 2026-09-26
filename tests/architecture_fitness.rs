//! Architecture checks that keep foreign product policy out of the core.

const MANIFEST: &str = include_str!("../Cargo.toml");
const CORE: &str = include_str!("../src/lib.rs");

#[test]
fn core_has_no_foreign_domain_or_provider_dependency() {
    for forbidden in [
        "naruon",
        "lineageweave",
        "saju",
        "four-pillars",
        "google-calendar",
        "microsoft-graph",
        "caldav-client",
    ] {
        assert!(!MANIFEST.to_ascii_lowercase().contains(forbidden));
        assert!(!CORE.to_ascii_lowercase().contains(forbidden));
    }
}

#[test]
fn durable_vertical_does_not_claim_a_network_or_provider_boundary() {
    for unshipped in ["axum", "actix-web", "reqwest"] {
        assert!(!MANIFEST.to_ascii_lowercase().contains(unshipped));
    }
}
