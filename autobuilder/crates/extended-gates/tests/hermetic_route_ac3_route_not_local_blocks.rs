//! PRD-rustbuild-hermetic-local-route AC3 (P0) — a shim that routes
//! despite the pin (R1) is a named block, not a silent pass and not a
//! misattributed-egress block: `verdict: "block"`, `cause:
//! "route-not-local"`, and the journal summary line ends `route=burst`.
//!
//! The fake `cargo` here is otherwise completely quiet (no network
//! activity, exit 0) — the ONLY reason this run blocks is the `burst`
//! line it appends to `route.log`, isolating R2's route-attestation check
//! from ordinary socket-attribution blocking (that's AC2).
//!
//! `#[ignore]`d: mutates process-global PATH; sound here only because this
//! file compiles to its own single-test integration binary. Run with
//! `cargo test --test hermetic_route_ac3_route_not_local_blocks -- --ignored`.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, unsafe_code)]

mod fixtures;
use fixtures::*;

use std::os::unix::fs::PermissionsExt;

use autobuilder_extended_gates::run_producer;

#[cfg(target_os = "linux")]
#[test]
#[ignore]
fn hermetic_route_ac3_route_not_local_blocks() {
    let tmp = tempfile::tempdir().unwrap();
    let project = tmp.path();
    init_git(project);

    let bin_dir = project.join("fakebin");
    std::fs::create_dir_all(&bin_dir).unwrap();
    let fake_cargo = bin_dir.join("cargo");
    std::fs::write(
        &fake_cargo,
        "#!/usr/bin/env bash\n\
         set -u\n\
         mkdir -p target/autobuilder\n\
         printf '%s burst cargo build\\n' \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\" >> target/autobuilder/route.log\n\
         exit 0\n",
    )
    .unwrap();
    std::fs::set_permissions(&fake_cargo, std::fs::Permissions::from_mode(0o755)).unwrap();

    let orig_path = std::env::var("PATH").unwrap_or_default();
    // SAFETY: one #[test] fn in this whole (ignored) integration-test
    // binary — no concurrent thread reads/writes PATH while this scope
    // holds it.
    unsafe {
        std::env::set_var("PATH", format!("{}:{orig_path}", bin_dir.display()));
    }
    let result = run_producer("hermetic-build", project);
    // SAFETY: see above.
    unsafe {
        std::env::set_var("PATH", &orig_path);
    }
    let summary = result.unwrap();
    assert!(summary.ends_with("route=burst"), "journal summary: {summary:?}");

    let v = read_receipt(project, "hermetic-build-receipt.json");
    assert_eq!(verdict_of(&v), "block", "receipt: {v:?}");
    assert_eq!(
        v.get("cause").and_then(serde_json::Value::as_str),
        Some("route-not-local"),
        "receipt: {v:?}"
    );
    assert_eq!(
        v.get("route_observed").and_then(serde_json::Value::as_str),
        Some("burst"),
        "receipt: {v:?}"
    );
}
