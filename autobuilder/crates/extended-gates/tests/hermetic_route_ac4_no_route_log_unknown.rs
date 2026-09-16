//! PRD-rustbuild-hermetic-local-route AC4 (P0) — with no `route.log` at
//! all, the producer still passes on a quiet build, `route_observed` is
//! the honest `"unknown"` (no shim ever attested either way — distinct
//! from AC2/AC3's `"local"`/`"burst"`, both of which require a
//! `route.log` to exist), and `ignore_rules` is unchanged by any of this.
//!
//! `#[ignore]`d: mutates process-global PATH; sound here only because this
//! file compiles to its own single-test integration binary. Run with
//! `cargo test --test hermetic_route_ac4_no_route_log_unknown -- --ignored`.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, unsafe_code)]

mod fixtures;
use fixtures::*;

use std::os::unix::fs::PermissionsExt;

use autobuilder_extended_gates::run_producer;

#[cfg(target_os = "linux")]
#[test]
#[ignore]
fn hermetic_route_ac4_no_route_log_unknown() {
    let tmp = tempfile::tempdir().unwrap();
    let project = tmp.path();
    init_git(project);

    let bin_dir = project.join("fakebin");
    std::fs::create_dir_all(&bin_dir).unwrap();
    let fake_cargo = bin_dir.join("cargo");
    // Quiet: no route.log write, no network activity, exit 0.
    std::fs::write(&fake_cargo, "#!/usr/bin/env bash\nexit 0\n").unwrap();
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
    result.unwrap();

    assert!(
        !project.join("target/autobuilder/route.log").exists(),
        "fixture must not have written a route.log"
    );

    let v = read_receipt(project, "hermetic-build-receipt.json");
    assert_eq!(verdict_of(&v), "pass", "receipt: {v:?}");
    assert_eq!(
        v.get("route_observed").and_then(serde_json::Value::as_str),
        Some("unknown"),
        "receipt: {v:?}"
    );
    assert!(v.get("cause").is_none(), "cause must be omitted on a pass: {v:?}");
    let ignore_rules: Vec<&str> = v
        .get("ignore_rules")
        .and_then(serde_json::Value::as_array)
        .expect("ignore_rules must be an array")
        .iter()
        .map(|x| x.as_str().expect("ignore_rules entries are strings"))
        .collect();
    assert_eq!(ignore_rules, vec!["loopback", "unix-domain"], "receipt: {v:?}");
}
