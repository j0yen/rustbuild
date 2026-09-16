//! PRD-rustbuild-hermetic-local-route AC6 (P1, R5) — with build-skill's
//! REAL `burst-lane-bin/cargo` shim first on PATH and
//! `BUILD_BURST_ENABLED=1` in the parent env (the "routing is mandatory"
//! shape from the incident this PRD fixes), the producer still builds
//! locally on a one-file crate: `route_observed: "local"`, verdict
//! `pass`, and `route.log` gains a `local` line, never a `burst` one.
//!
//! RedBaron-only (needs a real `~/.claude/skills/build` checkout with the
//! real shim scripts) and does not require a live burst box:
//! `BUILD_BURST_ENABLED=1` alone only makes `burst_configured()` true —
//! this producer's own `BURST_LANE=0` pin (R1) makes the shim take its
//! `cause=burst-lane-disabled` fallthrough before it ever calls
//! `burst-lane.sh run`.
//!
//! `#[ignore]`d: mutates process-global PATH/env and spawns a real `cargo
//! build`. Run with
//! `cargo test --test hermetic_route_ac6_real_shim_local -- --ignored`.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, unsafe_code)]

mod fixtures;
use fixtures::*;

use autobuilder_extended_gates::run_producer;

#[cfg(target_os = "linux")]
#[test]
#[ignore]
fn hermetic_route_ac6_real_shim_local() {
    let Ok(home) = std::env::var("HOME") else {
        eprintln!("skip: $HOME not set");
        return;
    };
    let shim_dir = std::path::Path::new(&home).join(".claude/skills/build/scripts/burst-lane-bin");
    if !shim_dir.join("cargo").is_file() {
        eprintln!(
            "skip: build-skill's burst-lane-bin/cargo shim not found at {}",
            shim_dir.display()
        );
        return;
    }

    let tmp = tempfile::tempdir().unwrap();
    let project = tmp.path();
    init_git(project);
    scaffold_trivial_lib(project);

    let route_log = project.join("target/autobuilder/route.log");
    std::fs::create_dir_all(route_log.parent().unwrap()).unwrap();

    let orig_path = std::env::var("PATH").unwrap_or_default();
    // SAFETY: one #[test] fn in this whole (ignored) integration-test
    // binary — no concurrent thread reads/writes these vars while this
    // scope holds them.
    unsafe {
        std::env::set_var("PATH", format!("{}:{orig_path}", shim_dir.display()));
        std::env::set_var("BUILD_BURST_ENABLED", "1");
        std::env::set_var("BURST_ROUTE_LOG", &route_log);
    }
    let result = run_producer("hermetic-build", project);
    // SAFETY: see above.
    unsafe {
        std::env::set_var("PATH", &orig_path);
        std::env::remove_var("BUILD_BURST_ENABLED");
        std::env::remove_var("BURST_ROUTE_LOG");
    }
    result.unwrap();

    let v = read_receipt(project, "hermetic-build-receipt.json");
    assert_eq!(verdict_of(&v), "pass", "receipt: {v:?}");
    assert_eq!(
        v.get("route_observed").and_then(serde_json::Value::as_str),
        Some("local"),
        "receipt: {v:?}"
    );

    let log = std::fs::read_to_string(&route_log).unwrap_or_default();
    assert!(log.contains(" local "), "expected a local route.log line from the real shim: {log:?}");
    assert!(!log.contains(" burst "), "expected no burst route.log line: {log:?}");
}
