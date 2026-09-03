//! Stage 2 — Scaffold. Materializes a Rust project tree from
//! `~/.claude/skills/autobuilder/templates/scaffold/` and instantiates the
//! per-AC test files + `AUTOBUILDER_PROGRAM.md` from the intent-card.
//!
//! Substitutions:
//!   `{{intent_slug}}`  → `intent_card.intent_slug`
//!   `{{target_kind}}`  → `intent_card.hard_constraints.target_kind`
//!
//! For every AC in `acceptance_criteria`, a `tests/acceptance_<id>.rs`
//! file is generated. The shipped `tests/acceptance_template.rs` is then
//! removed so the iteration loop doesn't see a stale placeholder.

use anyhow::{Context, Result, anyhow};
use clap::Args as ClapArgs;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[derive(Debug, ClapArgs)]
pub(crate) struct Args {
    /// Path to the validated intent-card.json.
    #[arg(long = "intent-card")]
    pub intent_card: PathBuf,

    /// Output directory for the materialized project. Must not exist.
    #[arg(long)]
    pub out: PathBuf,

    /// Scaffold template directory (overrides the skill default).
    #[arg(
        long,
        default_value = "~/.claude/skills/autobuilder/templates/scaffold"
    )]
    pub template: String,

    /// `AUTOBUILDER_PROGRAM.md.tmpl` location (overrides the skill default).
    #[arg(
        long = "program-template",
        default_value = "~/.claude/skills/autobuilder/templates/AUTOBUILDER_PROGRAM.md.tmpl"
    )]
    pub program_template: String,
}

#[allow(clippy::needless_pass_by_value)] // owned `Args` matches the clap-dispatched subcommand contract
#[allow(clippy::too_many_lines)] // single linear materialization pipeline; splitting hides the order
pub(crate) fn run(args: Args) -> Result<()> {
    let card_text = fs::read_to_string(&args.intent_card)
        .with_context(|| format!("missing intent-card at {}", args.intent_card.display()))?;
    let card: Value =
        serde_json::from_str(&card_text).context("intent-card is not valid JSON")?;

    let slug = card
        .get("intent_slug")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("intent-card lacks `intent_slug` (run `autobuilder intake` first)"))?
        .to_owned();
    let target_kind = card
        .pointer("/hard_constraints/target_kind")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("intent-card lacks /hard_constraints/target_kind"))?
        .to_owned();
    let acs = card
        .get("acceptance_criteria")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("intent-card lacks `acceptance_criteria`"))?
        .clone();

    let template_root = expand_tilde(&args.template);
    let program_tmpl = expand_tilde(&args.program_template);
    if !template_root.is_dir() {
        return Err(anyhow!(
            "scaffold template not found at {}",
            template_root.display()
        ));
    }
    // Atomically claim --out by creating it ourselves. `create_dir` (NOT
    // `create_dir_all`) is the only call that fails with AlreadyExists when
    // a concurrent process plants a dir between our check and our use.
    // No exists()-then-create_dir_all() race here.
    match fs::create_dir(&args.out) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            return Err(anyhow!(
                "--out {} already exists; pick a fresh path",
                args.out.display()
            ));
        }
        Err(e) => {
            return Err(anyhow!(
                "could not create --out {}: {e}",
                args.out.display()
            ));
        }
    }

    let subs = [("{{intent_slug}}", slug.as_str()), ("{{target_kind}}", target_kind.as_str())];
    let mut files_written = 0usize;
    copy_tree(&template_root, &args.out, &subs, &mut files_written)?;

    // Vendor the audit rules into the crate. `scripts/audit.sh` falls back
    // to `rules/audit-checks.sh` when `~/.claude` is absent (every CI
    // runner), so every scaffolded crate needs its own copy rather than
    // relying on the skill checkout that only exists on a build box.
    vendor_rules(&template_root, &args.out, &mut files_written)?;

    // For lib targets, drop the scaffold's `src/main.rs` stub. The template
    // emits both `src/main.rs` and `src/lib.rs` so the same materialization
    // works for `target_kind == "cli"`; for lib crates the main.rs is dead
    // weight that adds clippy `print_stderr` surface and confuses readers
    // about which file is the primary one. Counter is decremented so the
    // printed "N files" reflects what's actually on disk.
    if target_kind == "lib" {
        let main_rs = args.out.join("src/main.rs");
        if main_rs.is_file() {
            fs::remove_file(&main_rs)
                .with_context(|| format!("could not remove {}", main_rs.display()))?;
            files_written = files_written.saturating_sub(1);
        }
    }

    // Replace the shipped placeholder with one acceptance file per AC.
    let template_test = args.out.join("tests/acceptance_template.rs");
    if template_test.is_file() {
        fs::remove_file(&template_test).with_context(|| {
            format!("could not remove placeholder {}", template_test.display())
        })?;
    }
    let tests_dir = args.out.join("tests");
    fs::create_dir_all(&tests_dir)?;
    for ac in &acs {
        write_ac_test(&tests_dir, ac, &slug, &target_kind)?;
    }

    // Instantiate AUTOBUILDER_PROGRAM.md from the .tmpl.
    if program_tmpl.is_file() {
        let body = fs::read_to_string(&program_tmpl)
            .with_context(|| format!("missing program template at {}", program_tmpl.display()))?;
        let rendered = substitute(&body, &subs);
        let agent_dir = args.out.join("agent");
        fs::create_dir_all(&agent_dir)?;
        let dest = agent_dir.join("AUTOBUILDER_PROGRAM.md");
        fs::write(&dest, rendered)?;
        files_written += 1;
    }

    // Drop the intent-card itself into the new project for self-reference.
    let agent_dir = args.out.join("agent");
    fs::create_dir_all(&agent_dir)?;
    fs::copy(&args.intent_card, agent_dir.join("intent-card.json"))
        .with_context(|| "could not copy intent-card.json into agent/")?;

    println!(
        "scaffold: {} ({} target, {} ACs) → {} ({} files)",
        slug,
        target_kind,
        acs.len(),
        args.out.display(),
        files_written
    );
    Ok(())
}

fn copy_tree(
    src: &Path,
    dst: &Path,
    subs: &[(&str, &str)],
    files_written: &mut usize,
) -> Result<()> {
    fs::create_dir_all(dst)
        .with_context(|| format!("could not create {}", dst.display()))?;
    let entries = fs::read_dir(src)
        .with_context(|| format!("could not read {}", src.display()))?;
    for entry in entries {
        let entry = entry?;
        let name = entry.file_name();
        let src_path = entry.path();
        let dst_path = dst.join(&name);
        let ft = entry.file_type()?;
        if ft.is_dir() {
            copy_tree(&src_path, &dst_path, subs, files_written)?;
        } else if ft.is_file() {
            copy_file_with_subs(&src_path, &dst_path, subs)?;
            *files_written += 1;
        }
        // Skip symlinks / sockets / etc. — templates should not contain them.
    }
    Ok(())
}

fn copy_file_with_subs(src: &Path, dst: &Path, subs: &[(&str, &str)]) -> Result<()> {
    if is_textual(src) {
        let body = fs::read_to_string(src)
            .with_context(|| format!("could not read {}", src.display()))?;
        let rendered = substitute(&body, subs);
        fs::write(dst, rendered)
            .with_context(|| format!("could not write {}", dst.display()))?;
    } else {
        fs::copy(src, dst)
            .with_context(|| format!("could not copy {} → {}", src.display(), dst.display()))?;
    }
    Ok(())
}

/// Vendors `rules/audit-checks.sh` and `rules/hlt-rules.toml` from the skill
/// checkout into the new project's `rules/`. The skill checkout is the
/// sibling of `templates/scaffold/` (i.e. `template_root/../../rules`) —
/// `--template` always points at `<skill_root>/templates/scaffold`.
///
/// `scripts/audit.sh` has no `~/.claude` on a CI runner, so it falls back to
/// this vendored copy — see its header comment. Non-fatal when the skill
/// checkout can't be located (e.g. a minimal `--template` fixture in tests):
/// prints a warning and leaves the project without `rules/` rather than
/// failing the whole scaffold.
fn vendor_rules(template_root: &Path, out: &Path, files_written: &mut usize) -> Result<()> {
    let Some(skill_root) = template_root.parent().and_then(Path::parent) else {
        eprintln!(
            "scaffold: could not derive a skill root from template path {}; skipping rules/ vendoring",
            template_root.display()
        );
        return Ok(());
    };
    let rules_src_dir = skill_root.join("rules");
    let audit_src = rules_src_dir.join("audit-checks.sh");
    let hlt_src = rules_src_dir.join("hlt-rules.toml");
    if !audit_src.is_file() || !hlt_src.is_file() {
        eprintln!(
            "scaffold: vendored rules not found under {} (no skill checkout detected); scripts/audit.sh will need ~/.claude/skills/rustbuild at build/CI time",
            rules_src_dir.display()
        );
        return Ok(());
    }

    let rules_dst_dir = out.join("rules");
    fs::create_dir_all(&rules_dst_dir)
        .with_context(|| format!("could not create {}", rules_dst_dir.display()))?;

    let sha = git_short_sha(skill_root).unwrap_or_else(|_| "unknown".to_owned());
    let header = format!(
        "# VENDORED COPY — canonical source: j0yen/rustbuild skill/rules/audit-checks.sh @ {sha}\n\
         # CI has no ~/.claude, so the crate carries it; refresh by re-copying.\n"
    );
    let audit_body = fs::read_to_string(&audit_src)
        .with_context(|| format!("could not read {}", audit_src.display()))?;
    let audit_dst = rules_dst_dir.join("audit-checks.sh");
    fs::write(&audit_dst, insert_after_shebang(&audit_body, &header))
        .with_context(|| format!("could not write {}", audit_dst.display()))?;
    set_executable(&audit_dst)?;
    *files_written += 1;

    let hlt_dst = rules_dst_dir.join("hlt-rules.toml");
    fs::copy(&hlt_src, &hlt_dst)
        .with_context(|| format!("could not copy {} → {}", hlt_src.display(), hlt_dst.display()))?;
    *files_written += 1;

    Ok(())
}

/// Inserts `header` immediately after the shebang line (if any); otherwise
/// prepends it. Assumes `header` already ends in a newline.
fn insert_after_shebang(body: &str, header: &str) -> String {
    if let Some(rest) = body.strip_prefix("#!") {
        if let Some(nl) = rest.find('\n') {
            let (shebang_line, tail) = rest.split_at(nl + 1);
            return format!("#!{shebang_line}{header}{tail}");
        }
    }
    format!("{header}{body}")
}

/// Sets the executable bit on a vendored script. `fs::copy` would have
/// preserved the source's mode, but we rewrite the file's contents (to
/// insert the vendoring header) via `fs::write`, which does not.
#[cfg(unix)]
fn set_executable(path: &Path) -> Result<()> {
    let mut perms = fs::metadata(path)
        .with_context(|| format!("could not stat {}", path.display()))?
        .permissions();
    let mode = perms.mode() | 0o111;
    perms.set_mode(mode);
    fs::set_permissions(path, perms)
        .with_context(|| format!("could not chmod +x {}", path.display()))?;
    Ok(())
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) -> Result<()> {
    Ok(())
}

/// Short SHA of the rustbuild checkout backing `skill_root` (a symlink into
/// the repo, or the repo itself, or a self-cloned copy under
/// `~/.local/share/autobuilder/`). Used for the vendored-rules header so a
/// crate's `rules/audit-checks.sh` can be traced back to the source commit.
fn git_short_sha(skill_root: &Path) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(skill_root)
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .context("failed to spawn git rev-parse --short HEAD")?;
    if !output.status.success() {
        return Err(anyhow!(
            "git rev-parse --short HEAD in {} failed: {}",
            skill_root.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn is_textual(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some(
            "rs" | "toml" | "md" | "yml" | "yaml" | "sh" | "json" | "txt" | "lock"
        )
    ) || path.file_name().and_then(|n| n.to_str()).is_some_and(|n| {
        matches!(
            n,
            ".gitignore" | ".gitattributes" | "rust-toolchain.toml" | "clippy.toml" | "deny.toml"
        )
    })
}

fn substitute(body: &str, subs: &[(&str, &str)]) -> String {
    let mut out = body.to_owned();
    for (k, v) in subs {
        out = out.replace(k, v);
    }
    out
}

fn write_ac_test(tests_dir: &Path, ac: &Value, slug: &str, target_kind: &str) -> Result<()> {
    let id = ac
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("acceptance_criteria entry missing `id`"))?;
    let level = ac
        .get("level")
        .and_then(Value::as_str)
        .unwrap_or("MUST");
    let description = ac
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or("(no description)");
    let test_hint = ac
        .get("test")
        .and_then(Value::as_str)
        .unwrap_or("(no test predicate)");
    let id_lower = id.to_ascii_lowercase();
    let fn_name = format!("acceptance_{id_lower}");
    // The file is generated with two distinct ownership zones:
    // 1. The //! header block — AC metadata (id, level, description, test
    //    predicate). Mirror of the intent-card. Read-only for edit-agent.
    //    Changing an AC means filing agent/intent_card_amendment_request.json
    //    and re-scaffolding.
    // 2. The #[test] fn body — owned by the edit-agent. Iter-N replaces the
    //    panic stub with a real assertion that verifies the AC description.
    //    The file-level attribute lets the body use unwrap/expect freely
    //    (test-only ergonomics; tests/*.rs files are not in src/).
    let body = format!(
        "//! Acceptance test for {id} ({level}) — generated by `autobuilder scaffold`.\n\
         //!\n\
         //! Project: {slug} ({target_kind})\n\
         //! AC description: {description}\n\
         //! Test predicate: {test_hint}\n\
         //!\n\
         //! Ownership split: this //! header block (AC metadata) is\n\
         //! read-only — to change an AC, file\n\
         //! agent/intent_card_amendment_request.json and re-scaffold. The\n\
         //! `fn {fn_name}` body BELOW is owned by the edit-agent: replace\n\
         //! the panic stub with a real assertion that verifies the AC\n\
         //! description above.\n\
         \n\
         #![allow(clippy::unwrap_used, clippy::expect_used, clippy::doc_markdown)]\n\
         \n\
         #[test]\n\
         fn {fn_name}() {{\n\
         \x20   // edit-agent: replace this stub with a real assertion. The\n\
         \x20   // panic keeps the test failing until you do, so the loop\n\
         \x20   // sees a real Stage 3 signal.\n\
         \x20   panic!(\"AC {id} not yet implemented — see file header\");\n\
         }}\n"
    );
    let path = tests_dir.join(format!("acceptance_{id_lower}.rs"));
    fs::write(&path, body)
        .with_context(|| format!("could not write {}", path.display()))?;
    Ok(())
}

fn expand_tilde(s: &str) -> PathBuf {
    if let Some(rest) = s.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return Path::new(&home).join(rest);
        }
    }
    PathBuf::from(s)
}
