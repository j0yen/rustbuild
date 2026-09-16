//! PRD-rustbuild-hermetic-local-route AC1 (P0) — R1's local pin actually
//! reaches the cargo child's own environment, and the receipt names the
//! pin.
//!
//! Installs a fake `cargo` on PATH that dumps its own environment to a
//! file instead of building anything — this AC is about what
//! hermetic-build hands its child, not about a real build.
//!
//! `#[ignore]`d like the other hermetic-build ACs: mutates process-global
//! `PATH`/env vars, which is sound here only because this file compiles to
//! its own single-test integration binary (no sibling test in this
//! process to race with). Run with
//! `cargo test --test hermetic_route_ac1_env_pin_recorded -- --ignored`.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, unsafe_code)]

mod fixtures;
use fixtures::*;

use std::os::unix::fs::PermissionsExt;

use autobuilder_extended_gates::run_producer;

#[cfg(target_os = "linux")]
#[test]
#[ignore]
fn hermetic_route_ac1_env_pin_recorded() {
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
         {\n\
         \x20 echo \"BUILD_BURST_ENABLED=${BUILD_BURST_ENABLED-<unset>}\"\n\
         \x20 echo \"BURST_LANE=${BURST_LANE-<unset>}\"\n\
         \x20 if [ -z \"${BURST_LANE_SH+x}\" ]; then echo \"BURST_LANE_SH=<unset>\"; else echo \"BURST_LANE_SH=$BURST_LANE_SH\"; fi\n\
         } > env_dump.txt\n\
         exit 0\n",
    )
    .unwrap();
    std::fs::set_permissions(&fake_cargo, std::fs::Permissions::from_mode(0o755)).unwrap();

    let orig_path = std::env::var("PATH").unwrap_or_default();
    // SAFETY: one #[test] fn in this whole (ignored) integration-test
    // binary — no concurrent thread reads or writes these vars while this
    // scope holds them.
    unsafe {
        std::env::set_var("PATH", format!("{}:{orig_path}", bin_dir.display()));
        std::env::set_var("BUILD_BURST_ENABLED", "1");
        std::env::set_var("BURST_LANE", "1");
        std::env::set_var("BURST_LANE_SH", "/tmp/should-be-unset-by-the-pin");
    }

    let result = run_producer("hermetic-build", project);

    // SAFETY: see above.
    unsafe {
        std::env::set_var("PATH", &orig_path);
        std::env::remove_var("BUILD_BURST_ENABLED");
        std::env::remove_var("BURST_LANE");
        std::env::remove_var("BURST_LANE_SH");
    }
    result.unwrap();

    let dump = std::fs::read_to_string(project.join("env_dump.txt")).unwrap();
    assert!(dump.contains("BUILD_BURST_ENABLED=0"), "child env not pinned: {dump}");
    assert!(dump.contains("BURST_LANE=0"), "child env not pinned: {dump}");
    assert!(dump.contains("BURST_LANE_SH=<unset>"), "BURST_LANE_SH not unset: {dump}");

    let v = read_receipt(project, "hermetic-build-receipt.json");
    assert_eq!(
        v.get("cargo_route").and_then(serde_json::Value::as_str),
        Some("local-pinned"),
        "receipt: {v:?}"
    );
    assert_eq!(
        v.get("route_pinned").and_then(serde_json::Value::as_bool),
        Some(true),
        "receipt: {v:?}"
    );
}
