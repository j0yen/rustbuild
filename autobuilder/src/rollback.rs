//! Stage 4 — rollback-plan receipt.
//!
//! Two rollback models, selected per crate (PRD-autobuilder-rollback-tag-aware):
//!
//! - `revert-commits` (default): walks every commit in `<base>..HEAD` and
//!   verifies that `git revert` would apply cleanly onto `HEAD`. The right
//!   criterion for a library crate. Unchanged from the pre-PRD behavior.
//! - `redeploy-tag`: for a deploy-gated service whose real rollback is
//!   redeploying the previous tagged version (never a git revert), the
//!   verdict instead depends on the base tag existing, the v-tag lineage
//!   from base to HEAD being contiguous, and HEAD being tagged or taggable.
//!   Merge commits and non-revert-clean commits never affect this verdict.
//!
//! Emits `target/autobuilder/rollback.md` (human-readable) and
//! `target/autobuilder/receipts/rollback-plan.json` (the gate receipt,
//! schema `autobuilder.rollback_plan_receipt.v2`; `v1` is still accepted by
//! the gate during the transition). `revert-commits` mode uses
//! `git merge-tree --write-tree` so the check is read-only and never
//! touches the working tree; `redeploy-tag` mode never runs that check at
//! all.

use crate::receipt;
use anyhow::{Context, Result, anyhow};
use clap::Args as ClapArgs;
use regex::Regex;
use serde::{Deserialize, Serialize};
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
    /// In `redeploy-tag` mode, `--base` names the previous deployed
    /// version's tag directly.
    #[arg(long)]
    pub base: Option<String>,

    /// Print the resolved rollback model, base tag, and rollback target on
    /// one human-readable line, in addition to the normal run.
    #[arg(long)]
    pub explain: bool,

    /// One-time helper: print the `rollback_model` config line to paste
    /// into `agent/AUTOBUILDER_PROGRAM.md` (or `agent/intent-card.json`) to
    /// opt a deploy-gated crate into `redeploy-tag` mode. Exits immediately
    /// after printing; performs no git inspection and writes no receipt.
    #[arg(long)]
    pub migrate_note: bool,
}

/// The rollback verification strategy a crate uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RollbackModel {
    /// Today's behavior: every commit in `base..HEAD` must `git revert`
    /// cleanly. The right model for a library crate.
    RevertCommits,
    /// The 2026-09-05 fix-forward ruling for deploy-gated services:
    /// rollback means redeploying the previous tagged version, so the
    /// verdict depends on tag lineage, not per-commit revertability.
    RedeployTag,
}

impl RollbackModel {
    fn as_str(self) -> &'static str {
        match self {
            Self::RevertCommits => "revert-commits",
            Self::RedeployTag => "redeploy-tag",
        }
    }

    fn parse(raw: &str) -> Option<Self> {
        match raw {
            "revert-commits" => Some(Self::RevertCommits),
            "redeploy-tag" => Some(Self::RedeployTag),
            _ => None,
        }
    }
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

/// `redeploy-tag`-mode rollback target: the previous tagged version, its
/// commit, and the redeploy command form.
#[derive(Debug, Clone, Serialize)]
struct RollbackTarget {
    tag: String,
    sha: String,
    redeploy_command: String,
}

/// One version-bump commit found while walking tag lineage: the version it
/// bumped to, and the matching tag when one exists (`None` is the
/// `tag-lineage-gap` signal).
#[derive(Debug, Clone, Serialize)]
struct TagLineageEntry {
    version: String,
    tag: Option<String>,
    commit_sha: String,
    commit_short: String,
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
    /// In `redeploy-tag` mode this is always the previous deployed tag when
    /// one was resolved.
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
    /// v2: which rollback model produced this receipt. Always present.
    rollback_model: &'static str,
    /// v2: `redeploy-tag` only — the previous tag to redeploy to on pass.
    #[serde(skip_serializing_if = "Option::is_none")]
    rollback_target: Option<RollbackTarget>,
    /// v2: `redeploy-tag` only — every version-bump commit found in range
    /// and whether it carries a matching tag.
    #[serde(skip_serializing_if = "Option::is_none")]
    tag_lineage: Option<Vec<TagLineageEntry>>,
    /// v2: `redeploy-tag` only — machine-readable block reason
    /// (`no-previous-tag` | `tag-lineage-gap` | `head-untagged`).
    #[serde(skip_serializing_if = "Option::is_none")]
    block_reason: Option<&'static str>,
    /// v2: `redeploy-tag` only — human detail for `block_reason`.
    #[serde(skip_serializing_if = "Option::is_none")]
    block_detail: Option<String>,
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

/// The `[package]` fields this producer reads from a `Cargo.toml`. Unknown
/// keys elsewhere in the document are ignored by serde's default behavior.
#[derive(Debug, Deserialize)]
struct CargoPackageFields {
    name: Option<String>,
    version: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CargoTomlDoc {
    package: Option<CargoPackageFields>,
}

/// Read and parse `Cargo.toml` as it existed at `sha`, via `git show` (never
/// touches the working tree). `None` on any failure (missing file at that
/// commit, invalid TOML, no `[package]` table) — callers treat that as "not
/// determinable", not a hard error.
fn read_cargo_toml_at(project: &Path, sha: &str) -> Option<CargoTomlDoc> {
    let (status, stdout, _stderr) =
        run_git_capturing(project, &["show", &format!("{sha}:Cargo.toml")]).ok()?;
    if status != 0 {
        return None;
    }
    toml::from_str(&stdout).ok()
}

fn cargo_version_at(project: &Path, sha: &str) -> Option<String> {
    read_cargo_toml_at(project, sha).and_then(|c| c.package).and_then(|p| p.version)
}

fn cargo_name_at(project: &Path, sha: &str) -> Option<String> {
    read_cargo_toml_at(project, sha).and_then(|c| c.package).and_then(|p| p.name)
}

/// An explicit `rollback_model` declaration, raw and unvalidated (so the
/// caller can name a bad value verbatim — AC9), plus which file it came
/// from for the error message.
fn explicit_rollback_model(project: &Path) -> Result<Option<(String, &'static str)>> {
    let card_path = project.join("agent/intent-card.json");
    if let Ok(text) = fs::read_to_string(&card_path) {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(s) = value.get("rollback_model").and_then(serde_json::Value::as_str) {
                return Ok(Some((s.to_owned(), "agent/intent-card.json")));
            }
        }
    }
    let program_path = project.join("agent/AUTOBUILDER_PROGRAM.md");
    if let Ok(text) = fs::read_to_string(&program_path) {
        let re = Regex::new(r#"(?m)^\s*rollback_model\s*[:=]\s*"?([A-Za-z0-9_-]+)"?\s*$"#)?;
        if let Some(cap) = re.captures(&text) {
            if let Some(m) = cap.get(1) {
                return Ok(Some((m.as_str().to_owned(), "agent/AUTOBUILDER_PROGRAM.md")));
            }
        }
    }
    Ok(None)
}

/// Infer `redeploy-tag` when the crate carries a deploy manifest — the
/// signal that it ships as a redeployable tagged artifact rather than a
/// library. Convention: `agent/deploy-manifest.toml` (mirrors
/// `agent/AUTOBUILDER_PROGRAM.md`'s home under `agent/`); presence alone is
/// the signal today, fields inside are not yet parsed. Absent that signal,
/// the default stays `revert-commits` — the guard against a silent global
/// weakening (PRD Technical Considerations). See the README's "Rollback
/// models" section (PRD-rollback-redeploy-tag-onboard) for the full
/// onboarding schema and both opt-in paths.
fn infer_rollback_model(project: &Path) -> RollbackModel {
    if project.join("agent/deploy-manifest.toml").is_file() {
        RollbackModel::RedeployTag
    } else {
        RollbackModel::RevertCommits
    }
}

/// Resolve the crate's declared `rollback_model`: an explicit setting in
/// `agent/intent-card.json` or `agent/AUTOBUILDER_PROGRAM.md`, else
/// inferred from a deploy manifest, else the default `revert-commits`.
fn resolve_rollback_model(project: &Path) -> Result<RollbackModel> {
    if let Some((raw, source)) = explicit_rollback_model(project)? {
        return RollbackModel::parse(&raw).ok_or_else(|| {
            anyhow!(
                "invalid rollback_model {raw:?} declared in {source}; expected \"revert-commits\" or \"redeploy-tag\""
            )
        });
    }
    Ok(infer_rollback_model(project))
}

#[allow(clippy::needless_pass_by_value)] // owned `Args` matches the clap-dispatched subcommand contract
pub(crate) fn run(args: Args) -> Result<()> {
    let project = args
        .project
        .canonicalize()
        .with_context(|| format!("project path not found: {}", args.project.display()))?;

    let model = resolve_rollback_model(&project)?;

    if args.migrate_note {
        println!("rollback_model: {}", RollbackModel::RedeployTag.as_str());
        return Ok(());
    }

    let head_sha = git_rev_parse(&project, "HEAD")?;

    match model {
        RollbackModel::RevertCommits => run_revert_commits(&project, &args, &head_sha),
        RollbackModel::RedeployTag => run_redeploy_tag(&project, &args, &head_sha),
    }
}

/// `revert-commits` mode: byte-for-byte the pre-PRD logic (same base
/// resolution, same per-commit merge-tree revert check, same verdict rule).
/// Only the receipt's schema and the (always-empty-for-this-mode) v2
/// fields are new.
#[allow(clippy::too_many_lines)] // linear producer pipeline unchanged from the pre-PRD version; splitting hides the flow
fn run_revert_commits(project: &Path, args: &Args, head_sha: &str) -> Result<()> {
    let (base_ref, base_tag, base_note) = if let Some(explicit) = &args.base {
        (explicit.clone(), None, None)
    } else {
        let resolved = resolve_default_base(project)?;
        (resolved.git_ref, resolved.tag, resolved.note)
    };
    let base_sha = git_rev_parse(project, &base_ref)
        .with_context(|| format!("could not resolve base {base_ref} in {}", project.display()))?;

    let commits = list_commits(project, &base_ref)?;
    let mut entries: Vec<CommitEntry> = Vec::with_capacity(commits.len());
    let mut blocking = 0usize;
    for sha in &commits {
        let entry = check_commit(project, sha)?;
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
    write_rollback_md(&rollback_md_abs, head_sha, &base_info, &entries)?;

    let revertable_count = entries.iter().filter(|e| e.revertable).count();
    let verdict = if blocking == 0 { "pass" } else { "block" };

    if args.explain {
        println!(
            "rollback-plan --explain: model=revert-commits base={base_ref}{} target=git-revert({base_ref}..HEAD, {} commits)",
            base_tag
                .as_deref()
                .map(|t| format!(" base_tag={t}"))
                .unwrap_or_default(),
            entries.len(),
        );
    }

    let doc = ReceiptDoc {
        schema: "autobuilder.rollback_plan_receipt.v2",
        head_sha: head_sha.to_owned(),
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
        rollback_model: RollbackModel::RevertCommits.as_str(),
        rollback_target: None,
        tag_lineage: None,
        block_reason: None,
        block_detail: None,
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

/// Walk first-parent history strictly after `base_sha` up to and including
/// `HEAD`, recording every commit where the crate's `Cargo.toml` `version`
/// changed from its first-parent predecessor, and whether that commit
/// carries the matching `v<version>` tag. Mirrors `newest_reachable_tag`'s
/// own first-parent traversal (PRD Technical Considerations).
fn walk_tag_lineage(project: &Path, base_sha: &str) -> Result<Vec<TagLineageEntry>> {
    let mut commits = list_commits(project, base_sha)?; // newest -> oldest
    commits.reverse(); // oldest -> newest reads chronologically in the receipt
    let mut lineage = Vec::new();
    let mut prev_version = cargo_version_at(project, base_sha);
    for sha in &commits {
        let version = cargo_version_at(project, sha);
        if version != prev_version {
            if let Some(v) = &version {
                let tags_out = run_git(project, &["tag", "--points-at", sha])?;
                let want = format!("v{v}");
                let tag = tags_out.lines().map(str::trim).find(|t| *t == want).map(str::to_owned);
                lineage.push(TagLineageEntry {
                    version: v.clone(),
                    tag,
                    commit_sha: sha.clone(),
                    commit_short: short(sha),
                });
            }
        }
        prev_version = version;
    }
    Ok(lineage)
}

/// Whether HEAD can serve as a `redeploy-tag` rollback endpoint.
enum HeadTagStatus {
    /// HEAD already carries the matching version tag.
    Tagged,
    /// HEAD's version has no tag yet, but the tag name is free to create
    /// (e.g. at ship time via `ship-tag.sh`). Treated as passing.
    Taggable,
    /// HEAD cannot be tagged: its version is unreadable, or the tag name it
    /// wants is already claimed by a different commit.
    Blocked(String),
}

fn head_tag_status(project: &Path, head_sha: &str, version_tag: &str) -> Result<HeadTagStatus> {
    let at_head = run_git(project, &["tag", "--points-at", head_sha])?;
    if at_head.lines().map(str::trim).any(|t| t == version_tag) {
        return Ok(HeadTagStatus::Tagged);
    }
    let (status, _out, _err) = run_git_capturing(
        project,
        &["rev-parse", "--verify", "-q", &format!("refs/tags/{version_tag}")],
    )?;
    if status == 0 {
        return Ok(HeadTagStatus::Blocked(format!(
            "tag {version_tag} already exists on a different commit; cannot redeploy-tag HEAD"
        )));
    }
    Ok(HeadTagStatus::Taggable)
}

/// Bundles the base-resolution fields so helper functions stay under
/// clippy's argument-count lint (shared with `write_rollback_md`).
struct BaseInfo<'a> {
    git_ref: &'a str,
    sha: &'a str,
    tag: Option<&'a str>,
    note: Option<&'a str>,
}

/// `redeploy-tag` mode (PRD-autobuilder-rollback-tag-aware): verdict is
/// `pass` when the base tag exists, the tag lineage from base to HEAD is
/// contiguous, and HEAD is tagged or taggable — regardless of merge
/// commits or revert-cleanliness in the range.
#[allow(clippy::too_many_lines)] // linear producer pipeline: base resolution -> lineage walk -> head-tag check -> receipt; splitting hides the flow
fn run_redeploy_tag(project: &Path, args: &Args, head_sha: &str) -> Result<()> {
    let (base_ref, base_tag, base_note) = if let Some(explicit) = &args.base {
        (explicit.clone(), None, None)
    } else {
        let resolved = resolve_default_base(project)?;
        (resolved.git_ref, resolved.tag, resolved.note)
    };
    let base_sha = git_rev_parse(project, &base_ref)
        .with_context(|| format!("could not resolve base {base_ref} in {}", project.display()))?;

    let rollback_md_rel = PathBuf::from("target/autobuilder/rollback.md");
    let rollback_md_abs = project.join(&rollback_md_rel);

    // Edge case: zero tags reachable and no explicit --base — there is
    // nothing to roll back to yet.
    if args.base.is_none() && base_tag.is_none() {
        let base = BaseInfo {
            git_ref: &base_ref,
            sha: &base_sha,
            tag: None,
            note: base_note.as_deref(),
        };
        return finish_redeploy_block(
            project,
            &rollback_md_abs,
            &rollback_md_rel,
            head_sha,
            &base,
            "no-previous-tag",
            "no previous version tag is reachable from HEAD; nothing to redeploy to yet",
            &[],
            args.explain,
        );
    }

    let previous_tag = base_tag.clone().unwrap_or_else(|| base_ref.clone());

    let lineage = walk_tag_lineage(project, &base_sha)?;
    if let Some(gap) = lineage.iter().find(|e| e.tag.is_none()) {
        let base = BaseInfo {
            git_ref: &base_ref,
            sha: &base_sha,
            tag: Some(&previous_tag),
            note: base_note.as_deref(),
        };
        let detail = format!(
            "version {} (commit {}) was bumped in range but never tagged",
            gap.version, gap.commit_short
        );
        return finish_redeploy_block(
            project,
            &rollback_md_abs,
            &rollback_md_rel,
            head_sha,
            &base,
            "tag-lineage-gap",
            &detail,
            &lineage,
            args.explain,
        );
    }

    let head_version = cargo_version_at(project, head_sha);
    let head_status = match &head_version {
        None => HeadTagStatus::Blocked("HEAD has no readable Cargo.toml [package].version".to_owned()),
        Some(v) => head_tag_status(project, head_sha, &format!("v{v}"))?,
    };
    if let HeadTagStatus::Blocked(detail) = head_status {
        let base = BaseInfo {
            git_ref: &base_ref,
            sha: &base_sha,
            tag: Some(&previous_tag),
            note: base_note.as_deref(),
        };
        return finish_redeploy_block(
            project,
            &rollback_md_abs,
            &rollback_md_rel,
            head_sha,
            &base,
            "head-untagged",
            &detail,
            &lineage,
            args.explain,
        );
    }

    let package = cargo_name_at(project, head_sha).unwrap_or_else(|| {
        project
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("crate")
            .to_owned()
    });
    let target = RollbackTarget {
        tag: previous_tag.clone(),
        sha: base_sha.clone(),
        redeploy_command: format!("{package}-deploy redeploy --tag {previous_tag}"),
    };

    let base = BaseInfo {
        git_ref: &base_ref,
        sha: &base_sha,
        tag: Some(&previous_tag),
        note: base_note.as_deref(),
    };
    write_redeploy_md(&rollback_md_abs, head_sha, &base, "pass", None, Some(&target), &lineage)?;

    if args.explain {
        println!(
            "rollback-plan --explain: model=redeploy-tag base_tag={previous_tag} target=redeploy {}@{} via `{}`",
            target.tag,
            short(&target.sha),
            target.redeploy_command
        );
    }

    let doc = ReceiptDoc {
        schema: "autobuilder.rollback_plan_receipt.v2",
        head_sha: head_sha.to_owned(),
        base_ref: base_ref.clone(),
        base_sha: base_sha.clone(),
        base_tag: Some(previous_tag.clone()),
        base_note: base_note.clone(),
        rollback_md: rollback_md_rel.to_string_lossy().into_owned(),
        commit_count: lineage.len(),
        revertable_count: 0,
        blocking_count: 0,
        verdict: "pass",
        commits: Vec::new(),
        captured_at: receipt::now_rfc3339()?,
        receipt_digest: String::new(),
        rollback_model: RollbackModel::RedeployTag.as_str(),
        rollback_target: Some(target.clone()),
        tag_lineage: Some(lineage.clone()),
        block_reason: None,
        block_detail: None,
    };
    let value = serde_json::to_value(&doc)?;
    let receipt_path = project.join("target/autobuilder/receipts/rollback-plan.json");
    receipt::write(&receipt_path, value)?;

    println!(
        "rollback-plan: head={head_sha} model=redeploy-tag base_tag={previous_tag} target={} verdict=pass",
        target.tag
    );

    Ok(())
}

/// Shared block-path finisher for `redeploy-tag` mode: writes the markdown
/// and v2 receipt, prints `--explain` and the summary line, and returns the
/// blocking `Err`. Shared by all three block reasons so the receipt shape
/// stays identical regardless of which check failed.
#[allow(clippy::too_many_arguments)] // every field is required for both the receipt and the markdown; a context struct would just relocate the sprawl
fn finish_redeploy_block(
    project: &Path,
    rollback_md_abs: &Path,
    rollback_md_rel: &Path,
    head_sha: &str,
    base: &BaseInfo<'_>,
    reason: &'static str,
    detail: &str,
    lineage: &[TagLineageEntry],
    explain: bool,
) -> Result<()> {
    write_redeploy_md(
        rollback_md_abs,
        head_sha,
        base,
        "block",
        Some((reason, detail)),
        None,
        lineage,
    )?;

    if explain {
        println!(
            "rollback-plan --explain: model=redeploy-tag base={}{} target=blocked({reason}: {detail})",
            base.git_ref,
            base.tag.map(|t| format!(" base_tag={t}")).unwrap_or_default(),
        );
    }

    let doc = ReceiptDoc {
        schema: "autobuilder.rollback_plan_receipt.v2",
        head_sha: head_sha.to_owned(),
        base_ref: base.git_ref.to_owned(),
        base_sha: base.sha.to_owned(),
        base_tag: base.tag.map(str::to_owned),
        base_note: base.note.map(str::to_owned),
        rollback_md: rollback_md_rel.to_string_lossy().into_owned(),
        commit_count: lineage.len(),
        revertable_count: 0,
        blocking_count: 0,
        verdict: "block",
        commits: Vec::new(),
        captured_at: receipt::now_rfc3339()?,
        receipt_digest: String::new(),
        rollback_model: RollbackModel::RedeployTag.as_str(),
        rollback_target: None,
        tag_lineage: if lineage.is_empty() { None } else { Some(lineage.to_vec()) },
        block_reason: Some(reason),
        block_detail: Some(detail.to_owned()),
    };
    let value = serde_json::to_value(&doc)?;
    let receipt_path = project.join("target/autobuilder/receipts/rollback-plan.json");
    receipt::write(&receipt_path, value)?;

    println!("rollback-plan: head={head_sha} model=redeploy-tag verdict=block reason={reason}");

    Err(anyhow!("redeploy-tag rollback check blocked ({reason}): {detail}"))
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

/// `redeploy-tag` mode's markdown counterpart to `write_rollback_md`.
#[allow(clippy::too_many_arguments)] // mirrors write_rollback_md's BaseInfo bundling; the remaining fields are each independently optional
fn write_redeploy_md(
    path: &Path,
    head_sha: &str,
    base: &BaseInfo<'_>,
    verdict: &str,
    block: Option<(&str, &str)>,
    target: Option<&RollbackTarget>,
    lineage: &[TagLineageEntry],
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut out = String::new();
    out.push_str("# Rollback plan (redeploy-tag)\n\n");
    out.push_str(&format!("HEAD: `{head_sha}`\n"));
    out.push_str(&format!("Base: `{}` (`{}`)\n", base.git_ref, base.sha));
    if let Some(tag) = base.tag {
        out.push_str(&format!("Base tag: `{tag}`\n"));
    }
    if let Some(note) = base.note {
        out.push_str(&format!("Base note: {note}\n"));
    }
    out.push_str(&format!("\nVerdict: **{verdict}**\n"));
    if let Some((reason, detail)) = block {
        out.push_str(&format!("Block reason: `{reason}` — {detail}\n"));
    }
    if let Some(t) = target {
        out.push_str(&format!(
            "\nRollback target: redeploy `{}` (`{}`) via `{}`\n",
            t.tag, t.sha, t.redeploy_command
        ));
    }
    out.push_str("\nRollback here means redeploying the previous tagged version, not\n");
    out.push_str("reverting commits — merge commits and interleaved history do not\n");
    out.push_str("affect this verdict (PRD-autobuilder-rollback-tag-aware).\n");
    if !lineage.is_empty() {
        out.push_str("\n## Tag lineage\n\n");
        out.push_str("| version | tag | commit |\n");
        out.push_str("|---|---|---|\n");
        for e in lineage {
            let tag = e.tag.as_deref().unwrap_or("**MISSING**");
            out.push_str(&format!("| {} | {} | `{}` |\n", e.version, tag, e.commit_short));
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
