# Rust package release preflight baseline

## Scope

This note records the packaging checks used to make CalendarWeave easier to publish as a public Rust crate once its protected-main product contract and release gates are satisfied. It is packaging traceability only; it does not claim that `calendarweave` is already published on crates.io or that an unreleased branch is supported for installation.

## Authoritative Cargo behavior

Current Cargo exposes package metadata fields including `description`, `readme`, `repository`, `homepage`, `documentation`, `keywords`, `categories`, `license`, and `rust-version`. CalendarWeave now supplies the subset that is already truthful for the repository: description, Apache-2.0 license, Rust version, repository URL, README, and product-discovery keywords. It deliberately does not advertise a docs.rs URL, homepage, or release registry artifact that does not yet exist.

`cargo package` assembles the distributable `.crate` archive and then extracts and builds the archive from scratch unless verification is explicitly disabled. Cargo's documentation also states that `--locked` fails when the lock file is absent or would need to change, which is useful as a deterministic CI preflight. The CalendarWeave CI therefore runs `cargo package --locked` after format, clippy, tests, and docs in the Rust job. The command verifies packageability but does not publish anything and does not prove source provenance merely because `.cargo_vcs_info.json` is present.

## Product contract

- The public package name remains `calendarweave` and the repository/product name remains `CalendarWeave`.
- Package metadata must describe only currently supported repository facts; it must not imply a production endpoint or registry release.
- A successful package preflight is necessary but not sufficient for a release. Ordinary protected-branch review/checks, exact versioning, immutable release artifacts, provenance/SBOM, rollback guidance, and a real installation/call path remain release gates.
- Release automation must use ordinary repository credentials/secrets and must not introduce `COPILOT_GITHUB_TOKEN`.

## Verification map

| Contract | Evidence |
| --- | --- |
| Human-usable registry metadata is present | `[package]` in `Cargo.toml` includes repository, README, keywords, description, license and rust-version |
| Archive can be assembled and rebuilt from packaged contents | `.github/workflows/tests.yml` runs `cargo package --locked` without `--no-verify` |
| Dependency resolution does not silently rewrite the lock file during preflight | `--locked` on package preflight |
| Packaging is not confused with publishing | no `cargo publish` step is added by this slice |
| Packaging is not confused with provenance | product-gap baseline retains provenance/SBOM as a separate release gate |

## References

The Rust Project Developers. (2026). *cargo package — Assemble the local package into a distributable tarball*. The Cargo Book. https://doc.rust-lang.org/cargo/commands/cargo-package.html

The Rust Project Developers. (2026). *Manifest metadata*. Cargo documentation. https://doc.rust-lang.org/stable/nightly-rustc/cargo/core/manifest/struct.ManifestMetadata.html
