#!/usr/bin/env bash
# ship-tag.sh — tag the shipped commit of a Rust crate with v<version> from
# its Cargo.toml, and push the tag. PRD-rustbuild-tag-rollback-base.
#
# Single-crate mode (req 1):
#   ship-tag.sh <crate-dir> [prd-slug]
#
#   Reads the crate version from <crate-dir>/Cargo.toml. Refuses (exit 1,
#   creates nothing) if the working tree is dirty, or if v<version> already
#   exists at a commit other than HEAD. Creates an annotated tag v<version>
#   at HEAD (message includes the PRD slug when given) and pushes it with
#   `git push --follow-tags`. If v<version> already exists AT HEAD, this is
#   a no-op (exit 0) — safe to call a second time.
#
# Backfill mode (req 4, one-time / human-run):
#   ship-tag.sh --backfill [--yes] [root...]
#
#   Scans the given roots (default: ~/wintermute and ~/repos) for git repos
#   with a Cargo.toml up to 3 levels deep, lists each crate's name, version,
#   and HEAD, then tags every crate that has no v<version> tag yet — after
#   an interactive y/N confirmation (skip the prompt with --yes) — and
#   pushes. Crates with a dirty tree, an unreadable version, a version not
#   shaped like <major>.<minor>.<patch>, or an existing v<version> tag at a
#   different commit are listed with a status and skipped, never blocking
#   the rest of the batch.
#
# Non-goal: no semantic-version enforcement beyond the plain
# v<major>.<minor>.<patch> shape (pre-release tags like v0.6.0-rc1 are an
# open question — see the PRD — this script refuses them rather than guess).

set -uo pipefail

usage() {
  cat <<'EOF'
usage:
  ship-tag.sh <crate-dir> [prd-slug]
  ship-tag.sh --backfill [--yes] [root...]
EOF
}

# get_crate_version <dir>
# Prints the version string from <dir>/Cargo.toml's [package] section
# (following a same-file `version.workspace = true` to [workspace.package]
# if needed). Prints nothing and returns 1 if it can't be determined.
get_crate_version() {
  local dir="$1" toml="$1/Cargo.toml" v
  [ -f "$toml" ] || return 1

  v="$(awk '
    /^\[package\]/ { in_pkg=1; next }
    /^\[/          { in_pkg=0 }
    in_pkg && /^[[:space:]]*version[[:space:]]*=[[:space:]]*"/ {
      match($0, /"[^"]*"/)
      print substr($0, RSTART+1, RLENGTH-2)
      exit
    }
  ' "$toml")"

  if [ -z "$v" ]; then
    if awk '
      /^\[package\]/ { in_pkg=1; next }
      /^\[/          { in_pkg=0 }
      in_pkg && /^[[:space:]]*version\.workspace[[:space:]]*=[[:space:]]*true/ { found=1 }
      END { exit !found }
    ' "$toml"; then
      v="$(awk '
        /^\[workspace\.package\]/ { in_wp=1; next }
        /^\[/                     { in_wp=0 }
        in_wp && /^[[:space:]]*version[[:space:]]*=[[:space:]]*"/ {
          match($0, /"[^"]*"/)
          print substr($0, RSTART+1, RLENGTH-2)
          exit
        }
      ' "$toml")"
    fi
  fi

  [ -n "$v" ] || return 1
  printf '%s\n' "$v"
}

# is_plain_semver <version>
is_plain_semver() {
  printf '%s' "$1" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+$'
}

# ship_one <crate-dir> [prd-slug]
ship_one() {
  local crate_dir="$1" slug="${2:-}" abs_dir version tag head_sha tag_sha msg

  abs_dir="$(cd "$crate_dir" 2>/dev/null && pwd)" || {
    echo "ship-tag: no such directory: $crate_dir" >&2
    return 1
  }

  if ! git -C "$abs_dir" rev-parse --git-dir >/dev/null 2>&1; then
    echo "ship-tag: refusing — $abs_dir is not a git repo" >&2
    return 1
  fi

  if [ -n "$(git -C "$abs_dir" status --porcelain)" ]; then
    echo "ship-tag: refusing — working tree at $abs_dir is dirty" >&2
    return 1
  fi

  version="$(get_crate_version "$abs_dir")" || {
    echo "ship-tag: refusing — could not read version from $abs_dir/Cargo.toml" >&2
    return 1
  }

  if ! is_plain_semver "$version"; then
    echo "ship-tag: refusing — version '$version' in $abs_dir is not a plain v<major>.<minor>.<patch> shape" >&2
    return 1
  fi

  tag="v$version"
  head_sha="$(git -C "$abs_dir" rev-parse HEAD)"

  if git -C "$abs_dir" rev-parse -q --verify "refs/tags/$tag" >/dev/null 2>&1; then
    tag_sha="$(git -C "$abs_dir" rev-parse "refs/tags/$tag^{commit}")"
    if [ "$tag_sha" = "$head_sha" ]; then
      echo "ship-tag: $tag already at HEAD ($head_sha) in $abs_dir — no-op"
      return 0
    fi
    echo "ship-tag: refusing — $tag already exists at $tag_sha in $abs_dir, HEAD is $head_sha" >&2
    return 1
  fi

  msg="ship $tag"
  [ -n "$slug" ] && msg="ship $tag (PRD-$slug)"
  git -C "$abs_dir" tag -a "$tag" -m "$msg"

  if git -C "$abs_dir" remote get-url origin >/dev/null 2>&1; then
    git -C "$abs_dir" push --follow-tags
    # Belt-and-suspenders: --follow-tags only forwards tags reachable from
    # refs it actually updates, which can be a no-op push when the branch
    # is already current. Push the tag explicitly too (idempotent).
    git -C "$abs_dir" push origin "refs/tags/$tag" >/dev/null 2>&1 || true
    echo "ship-tag: created and pushed $tag at $head_sha in $abs_dir"
  else
    echo "ship-tag: created $tag at $head_sha in $abs_dir (no 'origin' remote — not pushed)"
  fi
}

# backfill [--yes] [root...]
backfill() {
  local yes=0 roots=() root cargo_toml dir toplevel cur name version head_sha tag tag_sha
  while [ $# -gt 0 ]; do
    case "$1" in
      --yes) yes=1; shift ;;
      *) roots+=("$1"); shift ;;
    esac
  done
  [ ${#roots[@]} -eq 0 ] && roots=("$HOME/wintermute" "$HOME/repos")

  declare -A seen_dir=()
  for root in "${roots[@]}"; do
    [ -d "$root" ] || continue
    while IFS= read -r -d '' cargo_toml; do
      dir="$(dirname "$cargo_toml")"
      git -C "$dir" rev-parse --git-dir >/dev/null 2>&1 || continue
      toplevel="$(git -C "$dir" rev-parse --show-toplevel 2>/dev/null)" || continue
      cur="${seen_dir[$toplevel]:-}"
      if [ -z "$cur" ] || [ "${#dir}" -lt "${#cur}" ]; then
        seen_dir["$toplevel"]="$dir"
      fi
    done < <(find "$root" -maxdepth 3 -name Cargo.toml -print0 2>/dev/null)
  done

  if [ ${#seen_dir[@]} -eq 0 ]; then
    echo "ship-tag --backfill: no crates found under: ${roots[*]}"
    return 0
  fi

  echo "ship-tag --backfill: crate list"
  printf '%-32s %-12s %-10s %s\n' "crate" "version" "head" "status"
  local to_tag=()
  for toplevel in "${!seen_dir[@]}"; do
    dir="${seen_dir[$toplevel]}"
    name="$(basename "$toplevel")"
    head_sha="$(git -C "$toplevel" rev-parse --short HEAD 2>/dev/null || echo '?')"
    if ! version="$(get_crate_version "$dir")"; then
      printf '%-32s %-12s %-10s %s\n' "$name" "-" "$head_sha" "no-version"
      continue
    fi
    if ! is_plain_semver "$version"; then
      printf '%-32s %-12s %-10s %s\n' "$name" "$version" "$head_sha" "bad-shape"
      continue
    fi
    tag="v$version"
    if git -C "$toplevel" rev-parse -q --verify "refs/tags/$tag" >/dev/null 2>&1; then
      tag_sha="$(git -C "$toplevel" rev-parse --short "refs/tags/$tag^{commit}")"
      if [ "$tag_sha" = "$head_sha" ]; then
        printf '%-32s %-12s %-10s %s\n' "$name" "$version" "$head_sha" "already-tagged"
      else
        printf '%-32s %-12s %-10s %s\n' "$name" "$version" "$head_sha" "tag-at-other-commit:$tag_sha"
      fi
      continue
    fi
    if [ -n "$(git -C "$toplevel" status --porcelain)" ]; then
      printf '%-32s %-12s %-10s %s\n' "$name" "$version" "$head_sha" "dirty-skip"
      continue
    fi
    printf '%-32s %-12s %-10s %s\n' "$name" "$version" "$head_sha" "untagged"
    to_tag+=("$toplevel")
  done

  if [ ${#to_tag[@]} -eq 0 ]; then
    echo "ship-tag --backfill: nothing to tag"
    return 0
  fi

  echo
  echo "ship-tag --backfill: will create+push tags for ${#to_tag[@]} crate(s):"
  for dir in "${to_tag[@]}"; do
    echo "  - $dir"
  done

  if [ "$yes" -ne 1 ]; then
    local ans
    read -r -p "Proceed? [y/N] " ans
    case "$ans" in
      y|Y|yes|YES) ;;
      *)
        echo "ship-tag --backfill: aborted, nothing tagged"
        return 1
        ;;
    esac
  fi

  local rc=0 dir
  for dir in "${to_tag[@]}"; do
    ship_one "$dir" "" || rc=1
  done
  return "$rc"
}

main() {
  case "${1:-}" in
    --backfill)
      shift
      backfill "$@"
      ;;
    -h|--help)
      usage
      ;;
    "")
      usage
      exit 1
      ;;
    *)
      ship_one "$1" "${2:-}"
      ;;
  esac
}

main "$@"
