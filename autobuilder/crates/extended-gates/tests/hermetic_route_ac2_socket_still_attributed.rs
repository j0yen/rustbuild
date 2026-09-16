//! PRD-rustbuild-hermetic-local-route AC2 (P0) — the local pin (R1) never
//! weakens ordinary socket attribution: a build-tree connection off a
//! *locally* routed cargo still blocks, and the receipt says the route
//! itself was fine (`route_observed: "local"`) — the block is about the
//! build's own egress, not about routing.
//!
//! The fake `cargo` here also appends one `local` line to `route.log`
//! before opening its socket, the same shape a real `burst-lane-bin/cargo`
//! writes when `BURST_LANE=0` disables it (see that script's `route_log`
//! calls) — this is what a genuinely-pinned, actually-local cargo child
//! looks like from hermetic-build's side, as opposed to AC4's "no
//! route.log at all" case.
//!
//! Uses a UDP `connect()` (routing-table lookup only, no handshake, no
//! listener needed) rather than a literal TCP handshake, for the same
//! flakiness reason `hermetic_scope_ac2_buildtree_child_attributed.rs`
//! gives: a real TCP SYN to a port nothing is listening on gets an
//! instant RST from the same host's own IP stack, too short-lived for a
//! 150ms sample tick to reliably catch.
//!
//! `#[ignore]`d: mutates process-global PATH; sound here only because this
//! file compiles to its own single-test integration binary. Run with
//! `cargo test --test hermetic_route_ac2_socket_still_attributed -- --ignored`.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, unsafe_code)]

mod fixtures;
use fixtures::*;

use std::os::unix::fs::PermissionsExt;

use autobuilder_extended_gates::run_producer;

#[cfg(target_os = "linux")]
#[test]
#[ignore]
fn hermetic_route_ac2_socket_still_attributed() {
    let Some(ip) = local_nonloopback_ip() else {
        eprintln!("skip: host has no non-loopback interface with a route");
        return;
    };
    let port = 54323;

    let tmp = tempfile::tempdir().unwrap();
    let project = tmp.path();
    init_git(project);

    let bin_dir = project.join("fakebin");
    std::fs::create_dir_all(&bin_dir).unwrap();
    let fake_cargo = bin_dir.join("cargo");
    std::fs::write(
        &fake_cargo,
        format!(
            "#!/usr/bin/env bash\n\
             set -u\n\
             mkdir -p target/autobuilder\n\
             printf '%s %s build local no-session %s\\n' \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\" \"$$\" \"$PWD\" >> target/autobuilder/route.log\n\
             exec 3<>/dev/udp/{ip}/{port}\n\
             sleep 0.4\n\
             exec 3<&-\n\
             exit 0\n"
        ),
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
    result.unwrap();

    let v = read_receipt(project, "hermetic-build-receipt.json");
    assert_eq!(verdict_of(&v), "block", "receipt: {v:?}");
    let new_sockets = v
        .get("new_sockets")
        .and_then(serde_json::Value::as_array)
        .expect("new_sockets must be an array");
    assert!(!new_sockets.is_empty(), "expected the fake cargo's connection to be attributed: {v:?}");
    assert_eq!(
        v.get("route_observed").and_then(serde_json::Value::as_str),
        Some("local"),
        "receipt: {v:?}"
    );
}
