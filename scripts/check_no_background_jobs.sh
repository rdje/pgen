#!/usr/bin/env bash
# HANDOFF BACKGROUND-JOB CENSUS — director standing rule, 2026-08-30.
#
#   "at a handoff point, that is whenever I am ready to /exit, ensure there is
#    no background job running."
#
# WHY THIS EXISTS. PGEN-ENGINE-UNIVERSAL-SERVICES-0134 opened on a tree that layer A
# said was clean and that was in fact being REWRITTEN under it: a chain.sh from a
# session that no longer existed had been running for 1 h 24 m and was overwriting
# three TRACKED artifacts of an already-CLOSED leaf, at a different measurement
# configuration, with its log gone with the session. A background job outlives the
# session that spawned it, and its output paths are tracked files.
#
# ⛔ SCOPE — this is a HANDOFF check, deliberately NOT a doctrine and NOT wired into
# .githooks/pre-commit. A commit may legitimately land while a job runs: the pause-signal
# directive's own rule 3 says to commit completed work FIRST and then wait. Making this
# block commits would punish exactly the behaviour that directive asks for. It is run
# before declaring a workspace handoff-ready, and nowhere else.
#
# HONEST LIMIT, stated so a green verdict is not over-read. A candidate is claimed only if its
# command line names this checkout OR its own cwd is inside it, so a sibling project's build is
# NOT reported (measured: the first cut reported two unrelated `linkedspec` processes, which is
# how an operator learns to ignore a census). Consequences of that choice, both real: a job whose
# cwd is elsewhere AND whose command line never spells the repo path is MISSED, and a process
# whose cwd cannot be read is treated as NOT ours rather than as ours. It excludes the caller's
# own ancestry so it never reports itself. ⚠️ THAT ANCESTRY WALK CAN TRUNCATE: if an intermediate
# shell has already exited, the orphaned caller is reparented and the chain stops early — OBSERVED
# once while building this (the harness's own wrapper shell was reported), and not reliably
# reproducible afterwards. So two shapes are ALSO excluded BY NAME as belt-and-braces — this script
# itself, and the harness wrapper (`.claude/shell-snapshots`) — rather than resting on the walk
# alone. Neither exclusion can hide a real project job, which is the property that matters.
# It is an operator instrument, not an oracle.
#
# Usage:  bash scripts/check_no_background_jobs.sh
# Exit:   0 = nothing project-owned is running · 1 = candidates listed · 2 = usage error

set -uo pipefail

case "${1:-}" in
  -h|--help) sed -n '2,28p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 0 ;;
  "") : ;;
  *) printf 'usage: bash scripts/check_no_background_jobs.sh\n' >&2; exit 2 ;;
esac

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)" || exit 2

# The caller's own ancestry: this script, the shell that ran it, the harness above that.
# Without this the census reports itself and can never go green.
ancestry() {
  local p="${1:-$$}" guard=0
  while [ -n "$p" ] && [ "$p" -gt 1 ] 2>/dev/null && [ "$guard" -lt 64 ]; do
    printf '%s\n' "$p"
    p="$(ps -o ppid= -p "$p" 2>/dev/null | tr -d '[:space:]')"
    guard=$((guard + 1))
  done
}
EXCLUDE=" $(ancestry "$$" | tr '\n' ' ') "
[ -n "${PGEN_HANDOFF_DEBUG:-}" ] && printf 'debug: self=%s EXCLUDE=[%s]\n' "$$" "$EXCLUDE" >&2

# ONE snapshot, so the processes spawned to filter it are not themselves in it.
SNAP="$(ps -Ao pid=,etime=,command= 2>/dev/null)"

# Stage 1 — broad CANDIDATE match. Deliberately loose; stage 2 does the discriminating.
PATTERN="${REPO_ROOT}|rust/target/(release|debug)/|docs/tasks/artifacts/.*\.(sh|py)|scripts/[a-z_]*\.sh|parse_alloc_census_probe|parseability_probe|run_with_memory_guard|cargo (build|test|run)|mdbook (build|serve)"

# Stage 2 — is this candidate OURS? A command line naming this checkout settles it. Otherwise
# the discriminator is the process's own WORKING DIRECTORY: a relative invocation like
# `cargo test --manifest-path rust/Cargo.toml` is meaningful only relative to a cwd, and a job
# belonging to a sibling checkout has a cwd outside this repo. Without this arm the census
# reports the operator's UNRELATED projects and teaches them to ignore it.
owned_by_this_repo() {
  local pid="$1" cmd="$2" cwd=""
  case "$cmd" in *"$REPO_ROOT"*) return 0 ;; esac
  cwd="$(lsof -a -p "$pid" -d cwd -Fn 2>/dev/null | sed -n 's/^n//p' | head -1)"
  [ -n "$cwd" ] || return 1                       # cwd unknowable -> do not claim it is ours
  case "$cwd" in "$REPO_ROOT"|"$REPO_ROOT"/*) return 0 ;; esac
  return 1
}

HITS=""
FOREIGN=0
while IFS= read -r line; do
  line="${line#"${line%%[![:space:]]*}"}"          # ⛔ ps PADS its pid column; without this trim
  [ -n "$line" ] || continue                       #    the pid extracted below is EMPTY and the
  pid="${line%% *}"                                #    ancestry exclusion silently never fires
  case "$pid" in ''|*[!0-9]*) continue ;; esac
  case "$EXCLUDE" in *" $pid "*) continue ;; esac
  cmd="${line#* }"; cmd="${cmd#* }"
  if owned_by_this_repo "$pid" "$cmd"; then
    HITS="${HITS}${line}"$'\n'
  else
    FOREIGN=$((FOREIGN + 1))
  fi
done < <(printf '%s\n' "$SNAP" | grep -aEi -- "$PATTERN" \
           | grep -av '[[:space:]]grep[[:space:]]' \
           | grep -av 'check_no_background_jobs\.sh' \
           | grep -av '\.claude/shell-snapshots')

HITS="$(printf '%s' "$HITS" | sed '/^$/d')"

if [ -z "$HITS" ]; then
  printf 'handoff: OK — no project-owned background job is running (repo %s)\n' "$REPO_ROOT"
  exit 0
fi

printf 'handoff: %s project-owned process(es) STILL RUNNING — not handoff-ready:\n\n' \
  "$(printf '%s\n' "$HITS" | wc -l | tr -d ' ')"
printf '%s\n' "$HITS" | sed 's/^/  /'
printf '\nKill them (and their children) before declaring the workspace handoff-ready.\n'
printf 'A job that outlives this session can rewrite TRACKED files with nobody watching\n'
printf '(PGEN-ENGINE-UNIVERSAL-SERVICES-0134). Verify the census is empty afterwards.\n'
exit 1
