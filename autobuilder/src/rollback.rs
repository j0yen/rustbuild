//! Stage 4 — rollback-plan receipt.
//!
//! Walks every commit in `<base>..HEAD` and verifies that `git revert` would
//! apply cleanly onto `HEAD`. Emits `target/autobuilder/rollback.md` (human-
//! readable) and `target/autobuilder/receipts/rollback-plan.json` (the gate
//! receipt). Uses `git merge-tree --write-tree` so the check is read-only and
//! never touches the working tree.

use crate::receipt;
use anyhow::{Context, Result, anyhow};
use clap::Args as ClapArgs;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, ClapArgs)]
pub(crate) struct Args {
    /// Project directory. The git repo at this path is the one inspected.
    #[arg(long, default_value = ".")]
    pub project: PathBuf,

    /// Range start: commits in `<base>..HEAD` are checked. Defaults to the
    /// newest `v<major>.<minor>.<patch>` tag reachable from HEAD by
    /// first-parent; if no such tag exists, falls back to the crate's
    /// initial commit. An explicit `--base` always wins over the default.
    #[arg(long)]
    pub base: Option<String>,
}

#[derive(Debug, Serialize)]
struct CommitEntry {
    sha: String,
    short_sha: String,
    subject: String,
    parent_count: usize,
    revertable: bool,
    note: String,
}

#[derive(Debug, Serialize)]
struct ReceiptDoc {
    schema: &'static str,
    head_sha: String,
    base_ref: String,
    base_sha: String,
    /// The `v<major>.<minor>.<patch>` tag the default-base resolution
    /// picked, when it picked one. `None` when `--base` was given
    /// explicitly, or when no matching tag exists on first-parent history.
    base_tag: Option<String>,
    /// Set to `"base=initial (no tags)"` when the default resolution found
    /// no matching tag and fell back to the crate's initial commit.
    base_note: Option<String>,
    rollback_md: String,
    commit_count: usize,
    revertable_count: usize,
    blocking_count: usize,
    verdict: &'static str,
    commits: Vec<CommitEntry>,
    captured_at: String,
    receipt_digest: String,
}

/// Resolution of the rollback base when `--base` was not given explicitly.
struct DefaultBase {
    /// Ref (tag name or commit sha) to diff against.
    git_ref: String,
    /// The matching tag, when one was found.
    tag: Option<String>,
    /// Human-readable note for the receipt/markdown when no tag was found.
    note: Option<String>,
}

/// A tag name shaped exactly like `v<major>.<minor>.<patch>` (all-digit
/// components, no pre-release/build suffix).
fn is_version_tag(tag: &str) -> bool {
    let Some(rest) = tag.strip_prefix('v') else {
        return false;
    };
    let parts: Vec<&str> = rest.split('.').collect();
    parts.len() == 3 && parts.iter().all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
}

/// Sort key for version tags: `v1.2.3` -> `(1, 2, 3)`. Malformed components
/// (should not occur given `is_version_tag` already filtered) sort as 0.
fn version_key(tag: &str) -> (u64, u64, u64) {
    let rest = tag.strip_prefix('v').unwrap_or(tag);
    let mut parts = rest.split('.').map(|p| p.parse::<u64>().unwrap_or(0));
    let major = parts.next().unwrap_or(0);
    let minor = parts.next().unwrap_or(0);
    let patch = parts.next().unwrap_or(0);
    (major, minor, patch)
}

/// Walk first-parent history from HEAD backward and return the newest
/// `v<major>.<minor>.<patch>` tag encountered (i.e. the first commit, in
/// that walk, that any such tag points at). `None` when no commit on the
/// first-parent path carries a matching tag.
fn newest_reachable_tag(project: &Path) -> Result<Option<String>> {
    let commits = run_git(project, &["rev-list", "--first-parent", "HEAD"])?;
    for sha in commits.lines() {
        let sha = sha.trim();
        if sha.is_empty() {
            continue;
        }
        let tags_out = run_git(project, &["tag", "--points-at", sha])?;
        let mut candidates: Vec<&str> = tags_out
            .lines()
            .map(str::trim)
            .filter(|t| is_version_tag(t))
            .collect();
        if candidates.is_empty() {
            continue;
        }
        candidates.sort_by_key(|t| std::cmp::Reverse(version_key(t)));
        if let Some(best) = candidates.first() {
            return Ok(Some((*best).to_owned()));
        }
    }
    Ok(None)
}

/// Resolve the default rollback base (used when `--base` was not given):
/// the newest reachable version tag, or the crate's initial commit.
fn resolve_default_base(project: &Path) -> Result<DefaultBase> {
    if let Some(tag) = newest_reachable_tag(project)? {
        return Ok(DefaultBase {
            git_ref: tag.clone(),
            tag: Some(tag),
            note: None,
        });
    }
    let initial = run_git(project, &["rev-list", "--max-parents=0", "HEAD"])?;
    let initial_sha = initial
        .lines()
        .next()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| anyhow!("could not resolve initial commit for {}", project.display()))?;
    Ok(DefaultBase {
        git_ref: initial_sha,
        tag: None,
        note: Some("base=initial (no tags)".to_owned()),
    })
}

#[allow(clippy::needless_pass_by_value)] // owned `Args` matches the clap-dispatched subcommand contract
pub(crate) fn run(args: Args) -> Result<()> {
    let project = args
        .project
        .canonicalize()
        .with_context(|| format!("project path not found: {}", args.project.display()))?;

    let head_sha = git_rev_parse(&project, "HEAD")?;

    let (base_ref, base_tag, base_note) = if let Some(explicit) = &args.base {
        (explicit.clone(), None, None)
    } else {
        let resolved = resolve_default_base(&project)?;
        (resolved.git_ref, resolved.tag, resolved.note)
    };
    let base_sha = git_rev_parse(&project, &base_ref)
        .with_context(|| format!("could not resolve base {base_ref} in {}", project.display()))?;

    let commits = list_commits(&project, &base_ref)?;
    let mut entries: Vec<CommitEntry> = Vec::with_capacity(commits.len());
    let mut blocking = 0usize;
    for sha in &commits {
        let entry = check_commit(&project, sha)?;
        if !entry.revertable {
            blocking += 1;
        }
        entries.push(entry);
    }

    // P1: warn (never block) when a tag-derived base is more than 5 commits
    // behind HEAD — the receipt still passes/blocks on revert-cleanliness
    // alone.
    if base_tag.is_some() && entries.len() > 5 {
        eprintln!(
            "rollback-plan: warning: {} commits since the newest tag ({}); ship more often, or tag",
            entries.len(),
            base_tag.as_deref().unwrap_or("?")
        );
    }

    let rollback_md_rel = PathBuf::from("target/autobuilder/rollback.md");
    let rollback_md_abs = project.join(&rollback_md_rel);
    let base_info = BaseInfo {
        git_ref: &base_ref,
        sha: &base_sha,
        tag: base_tag.as_deref(),
        note: base_note.as_deref(),
    };
    write_rollback_md(&rollback_md_abs, &head_sha, &base_info, &entries)?;

    let revertable_count = entries.iter().filter(|e| e.revertable).count();
    let verdict = if blocking == 0 { "pass" } else { "block" };

    let doc = ReceiptDoc {
        schema: "autobuilder.rollback_plan_receipt.v1",
        head_sha: head_sha.clone(),
        base_ref: base_ref.clone(),
        base_sha,
        base_tag: base_tag.clone(),
        base_note: base_note.clone(),
        rollback_md: rollback_md_rel.to_string_lossy().into_owned(),
        commit_count: entries.len(),
        revertable_count,
        blocking_count: blocking,
        verdict,
        commits: entries,
        captured_at: receipt::now_rfc3339()?,
        receipt_digest: String::new(),
    };
    let value = serde_json::to_value(&doc)?;
    let receipt_path = project.join("target/autobuilder/receipts/rollback-plan.json");
    receipt::write(&receipt_path, value)?;

    println!(
        "rollback-plan: head={head_sha} base={base_ref}{} commits={} revertable={revertable_count} verdict={verdict}",
        base_tag
            .as_deref()
            .map(|t| format!(" base_tag={t}"))
            .or_else(|| base_note.clone().map(|n| format!(" {n}")))
            .unwrap_or_default(),
        doc.commit_count
    );

    if blocking > 0 {
        return Err(anyhow!(
            "{blocking} of {} commits are not git-revert-clean; see {}",
            doc.commit_count,
            rollback_md_rel.display()
        ));
    }
    Ok(())
}

fn git_rev_parse(project: &Path, refname: &str) -> Result<String> {
    let out = run_git(project, &["rev-parse", refname])?;
    Ok(out.trim().to_owned())
}

fn list_commits(project: &Path, base: &str) -> Result<Vec<String>> {
    // `--first-parent` keeps the receipt focused on the main-line history;
    // any merge commit is treated as a single revertable unit (with `-m 1`).
    let range = format!("{base}..HEAD");
    let out = run_git(project, &["rev-list", "--first-parent", &range])?;
    Ok(out.lines().map(str::to_owned).collect())
}

fn check_commit(project: &Path, sha: &str) -> Result<CommitEntry> {
    let subject = run_git(project, &["log", "-1", "--format=%s", sha])?
        .trim()
        .to_owned();
    let parents_str = run_git(project, &["log", "-1", "--format=%P", sha])?;
    let parents: Vec<&str> = parents_str.split_whitespace().collect();
    let parent_count = parents.len();

    // The mainline parent used as `theirs` for the revert-merge.
    let Some(first_parent) = parents.first() else {
        return Ok(CommitEntry {
            sha: sha.to_owned(),
            short_sha: short(sha),
            subject,
            parent_count: 0,
            revertable: false,
            note: "root commit (no parent) cannot be reverted".to_owned(),
        });
    };

    let merge_arg = format!("--merge-base={sha}");
    let (status, _stdout, stderr) = run_git_capturing(
        project,
        &["merge-tree", "--write-tree", &merge_arg, "HEAD", first_parent],
    )?;

    let revertable = status == 0;
    let note = if revertable {
        if parent_count > 1 {
            "merge commit; revert with `git revert -m 1`".to_owned()
        } else {
            "clean revert".to_owned()
        }
    } else {
        match stderr.lines().next() {
            Some(line) if !line.trim().is_empty() => format!("conflicts: {}", line.trim()),
            _ => "conflicts during revert".to_owned(),
        }
    };

    Ok(CommitEntry {
        sha: sha.to_owned(),
        short_sha: short(sha),
        subject,
        parent_count,
        revertable,
        note,
    })
}

/// Bundles the base-resolution fields for `write_rollback_md` so it stays
/// under clippy's argument-count lint.
struct BaseInfo<'a> {
    git_ref: &'a str,
    sha: &'a str,
    tag: Option<&'a str>,
    note: Option<&'a str>,
}

fn write_rollback_md(
    path: &Path,
    head_sha: &str,
    base: &BaseInfo<'_>,
    entries: &[CommitEntry],
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut out = String::new();
    out.push_str("# Rollback plan\n\n");
    out.push_str(&format!("HEAD: `{head_sha}`\n"));
    out.push_str(&format!("Base: `{}` (`{}`)\n", base.git_ref, base.sha));
    if let Some(tag) = base.tag {
        out.push_str(&format!("Base tag: `{tag}`\n"));
    }
    if let Some(note) = base.note {
        out.push_str(&format!("Base note: {note}\n"));
    }
    out.push('\n');
    out.push_str("Reverts are listed newest → oldest. Each `git revert` was\n");
    out.push_str("dry-run via `git merge-tree --write-tree` against current HEAD,\n");
    out.push_str("so the working tree was not touched during verification.\n\n");
    if entries.is_empty() {
        out.push_str("No commits in range.\n");
    } else {
        out.push_str("| # | sha | revertable | command | subject |\n");
        out.push_str("|---|---|---|---|---|\n");
        for (i, e) in entries.iter().enumerate() {
            let cmd = if e.parent_count > 1 {
                format!("`git revert -m 1 {}`", e.short_sha)
            } else {
                format!("`git revert {}`", e.short_sha)
            };
            let mark = if e.revertable { "✓" } else { "✗" };
            let subject = e.subject.replace('|', "\\|");
            out.push_str(&format!(
                "| {n} | `{sha}` | {mark} | {cmd} | {subj} |\n",
                n = i + 1,
                sha = e.short_sha,
                subj = subject,
            ));
        }
        out.push_str("\n## Notes\n\n");
        for e in entries {
            out.push_str(&format!("- `{}` — {}\n", e.short_sha, e.note));
        }
    }
    fs::write(path, out)?;
    Ok(())
}

fn run_git(project: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(project)
        .args(args)
        .output()
        .with_context(|| format!("failed to spawn git {args:?}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("git {:?} failed: {}", args, stderr.trim()));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn run_git_capturing(
    project: &Path,
    args: &[&str],
) -> Result<(i32, String, String)> {
    let output = Command::new("git")
        .arg("-C")
        .arg(project)
        .args(args)
        .output()
        .with_context(|| format!("failed to spawn git {args:?}"))?;
    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    Ok((code, stdout, stderr))
}

fn short(sha: &str) -> String {
    sha.get(..7.min(sha.len())).unwrap_or(sha).to_owned()
}
