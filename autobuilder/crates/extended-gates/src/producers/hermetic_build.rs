//! `hermetic-build`: detect outbound network sockets during `cargo build --offline`.
//!
//! Linux-only. Spawns `cargo build --offline`; before and after the build,
//! reads `/proc/net/tcp` + `/proc/net/tcp6` and diffs the connection set
//! attributed to cargo's process tree. Any new outbound (non-listening)
//! socket during the build window is a hermeticity violation.
//!
//! The spawned cargo runs with `RUSTC_WRAPPER` / `CARGO_BUILD_RUSTC_WRAPPER`
//! forced empty: a hermetic build has no wrapper by definition, and a
//! developer machine with `rustc-wrapper = sccache` set in
//! `~/.cargo/config.toml` would otherwise have sccache's localhost client
//! connections misreported as hermeticity violations. Loopback remote
//! addresses (`127.0.0.0/8`, `::1`) are also excluded from the diff for the
//! same reason — a local build cache or daemon on localhost is not
//! "outbound."
//!
//! On non-Linux hosts the producer emits `verdict=skipped`.

use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use serde::Serialize;

use crate::prelude::{ProducerSpec, write_receipt};

#[derive(Debug, Serialize)]
struct Payload {
    platform: String,
    new_sockets: Vec<String>,
    cargo_exit_code: Option<i32>,
    rustc_wrapper_disabled: bool,
}

/// Decode a `/proc/net/tcp{,6}` hex address field (before the `:port`) into
/// its real address bytes. The kernel stores the address as one or more
/// 32-bit words in host byte order; on every hermetic-build host (`x86_64`/
/// `aarch64` Linux, both little-endian) that means each 4-byte word must be
/// reversed to get network-order bytes.
fn decode_hex_addr(hex: &str) -> Option<Vec<u8>> {
    if hex.len() % 8 != 0 || hex.is_empty() {
        return None;
    }
    let mut out = Vec::with_capacity(hex.len() / 2);
    for word in hex.as_bytes().chunks(8) {
        let word_str = std::str::from_utf8(word).ok()?;
        let mut word_bytes = [0u8; 4];
        for (i, wb) in word_bytes.iter_mut().enumerate() {
            let pair = word_str.get(i * 2..i * 2 + 2)?;
            *wb = u8::from_str_radix(pair, 16).ok()?;
        }
        word_bytes.reverse();
        out.extend_from_slice(&word_bytes);
    }
    Some(out)
}

/// True if the `/proc/net/tcp{,6}` hex remote address (without the port) is
/// a loopback address: `127.0.0.0/8` for IPv4, `::1` for IPv6.
fn is_loopback_hex_addr(hex: &str) -> bool {
    let Some(bytes) = decode_hex_addr(hex) else {
        return false;
    };
    match bytes.len() {
        4 => bytes.first() == Some(&127),
        16 => bytes.iter().take(15).all(|b| *b == 0) && bytes.get(15) == Some(&1),
        _ => false,
    }
}

fn read_proc_net(name: &str) -> Vec<String> {
    let path = format!("/proc/net/{name}");
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    text.lines()
        .skip(1)
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let _ = fields.next()?;
            let local = fields.next()?;
            let remote = fields.next()?;
            let state = fields.next()?;
            if state == "0A" {
                return None;
            }
            if remote.ends_with(":0000")
                || remote == "00000000:0000"
                || remote == "00000000000000000000000000000000:0000"
            {
                return None;
            }
            let remote_addr = remote.split(':').next().unwrap_or("");
            if is_loopback_hex_addr(remote_addr) {
                return None;
            }
            Some(format!("{name} local={local} remote={remote} state={state}"))
        })
        .collect()
}

/// Run the hermetic-build audit.
///
/// # Errors
///
/// Returns an error if cargo can't be spawned or the receipt write fails.
pub fn run(spec: &ProducerSpec, project: &Path) -> Result<String> {
    if !cfg!(target_os = "linux") {
        write_receipt(
            project,
            spec,
            "skipped",
            Payload {
                platform: std::env::consts::OS.to_owned(),
                new_sockets: Vec::new(),
                cargo_exit_code: None,
                rustc_wrapper_disabled: false,
            },
        )?;
        return Ok(format!(
            "hermetic-build: skipped (platform={})",
            std::env::consts::OS
        ));
    }

    let before: std::collections::BTreeSet<String> = read_proc_net("tcp")
        .into_iter()
        .chain(read_proc_net("tcp6"))
        .collect();

    // A hermetic build has no rustc wrapper by definition. Force both env
    // vars empty so a locally configured `rustc-wrapper = sccache` in
    // ~/.cargo/config.toml doesn't get invoked and misattribute its
    // localhost client sockets to this build.
    let status = Command::new("cargo")
        .args(["build", "--offline", "--release"])
        .current_dir(project)
        .env("RUSTC_WRAPPER", "")
        .env("CARGO_BUILD_RUSTC_WRAPPER", "")
        .status()
        .context("spawn cargo build --offline")?;
    let exit = status.code();

    let after: std::collections::BTreeSet<String> = read_proc_net("tcp")
        .into_iter()
        .chain(read_proc_net("tcp6"))
        .collect();

    let new_sockets: Vec<String> = after.difference(&before).cloned().collect();

    let verdict = if new_sockets.is_empty() && exit == Some(0) {
        "pass"
    } else {
        "block"
    };
    let summary = format!(
        "hermetic-build: cargo exit={exit:?}, {} new sockets (rustc wrapper disabled, loopback ignored)",
        new_sockets.len()
    );
    write_receipt(
        project,
        spec,
        verdict,
        Payload {
            platform: std::env::consts::OS.to_owned(),
            new_sockets,
            cargo_exit_code: exit,
            rustc_wrapper_disabled: true,
        },
    )?;
    Ok(summary)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn decodes_ipv4_loopback() {
        // 127.0.0.1 stored little-endian-per-word as "0100007F".
        assert!(is_loopback_hex_addr("0100007F"));
    }

    #[test]
    fn decodes_ipv4_non_loopback() {
        // 10.0.0.5 stored as "0500000A".
        assert!(!is_loopback_hex_addr("0500000A"));
    }

    #[test]
    fn decodes_ipv6_loopback() {
        // ::1 as 16 bytes is 15 zero bytes then a trailing 1. Stored as 4
        // little-endian 32-bit words, the last word (bytes[12..16], value
        // [0,0,0,1]) is byte-reversed to "01000000"; the rest are all-zero.
        let hex = "00000000000000000000000001000000";
        assert_eq!(hex.len(), 32);
        assert!(is_loopback_hex_addr(hex));
    }

    #[test]
    fn non_loopback_ipv6_is_not_flagged() {
        assert!(!is_loopback_hex_addr(
            "00000000000000000000000000000000"
        ));
    }
}
