#!/usr/bin/env bash
# scripts/run_with_memory_guard.sh — run a job under a host-RAM guard (OPS-MEMSAFE.1).
#
# Mechanical enforcement of the HOST-RAM BUDGET DIRECTIVE
# (docs/decisions/feedback_host_ram_budget_all_jobs.md, director 2026-07-14):
# on this 24 GB single-project machine, NO spawned job may exhaust host RAM.
#
# What it does:
#   1. PRE-FLIGHT: refuses to launch the job if system-wide free memory is already
#      below the floor (never launch into a pressured system).
#   2. Runs the command in its OWN PROCESS GROUP and samples the job's process-tree
#      RSS (process-group members ∪ parent-child descendant closure) on an interval.
#   3. KILLS the whole tree (TERM → grace → KILL) and writes an observable breach
#      MARKER + an always-on log line when:
#        - the tree's summed RSS exceeds the budget (default 12288 MB ≈ half RAM), or
#        - system-wide free memory drops below the floor (default 10%), or
#        - the optional wall-clock timeout expires.
#   4. Always writes a completion marker (composes with the background-job
#      observability doctrine: completion marker + bounded timeout + liveness).
#
# Exit codes (mechanically branchable by callers):
#   child's own exit code   — the job completed on its own (guard transparent)
#   96  preflight-refused   — system already below the free floor; job NEVER started
#   97  rss-budget breach   — tree RSS exceeded --budget-mb
#   98  free-floor breach   — system free % dropped below --floor-pct mid-run
#   99  timeout             — --timeout-s expired
#   130 guard-interrupted   — the guard itself received INT/TERM (tree killed too)
#   2   usage error
#
# Usage:
#   scripts/run_with_memory_guard.sh [options] -- <command> [args...]
# Options:
#   --budget-mb N    process-tree RSS budget in MB        (default 12288)
#   --floor-pct N    minimum system-wide free memory %    (default 10)
#   --interval-s N   sampling interval in seconds         (default 5)
#   --timeout-s N    wall-clock timeout in seconds, 0=off (default 0)
#   --marker FILE    marker file path (default rust/target/generated_logs/memory_guard/guard.<pid>.marker)
#   --log FILE       also append guard lines to FILE
#
# Verbosity: PGEN_TRACE_VERBOSITY ∈ none|low|medium|high|debug (default low).
#   Periodic sample lines print at medium+; per-PID breakdown at debug.
#   Breaches, kills, and warnings print UNCONDITIONALLY — severity is never gated
#   by verbosity (docs/decisions/feedback_severity_never_gated_by_verbosity.md).
#
# Test seam (deterministic floor-path testing only):
#   PGEN_MEMORY_GUARD_FAKE_FREE_PCT_FILE=<file> — read the free % from <file>
#   (re-read every sample) instead of `memory_pressure -Q`.
#
# Honest limit: a descendant that re-parents AND changes its own process group
# escapes both accounting and the kill. Jobs PGEN spawns (make/cargo/bench/parse)
# do neither. This is bounded-blast-radius enforcement, not a kernel limit.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

BUDGET_MB=12288
FLOOR_PCT=10
INTERVAL_S=5
TIMEOUT_S=0
MARKER=""
LOG_FILE=""

usage() { sed -n '2,50p' "${BASH_SOURCE[0]}" | grep -E '^# ?' | sed 's/^# \{0,1\}//'; }

is_uint() { [[ "$1" =~ ^[0-9]+$ ]]; }

while [ $# -gt 0 ]; do
  case "$1" in
    --budget-mb)  BUDGET_MB="${2:-}"; shift 2 ;;
    --floor-pct)  FLOOR_PCT="${2:-}"; shift 2 ;;
    --interval-s) INTERVAL_S="${2:-}"; shift 2 ;;
    --timeout-s)  TIMEOUT_S="${2:-}"; shift 2 ;;
    --marker)     MARKER="${2:-}"; shift 2 ;;
    --log)        LOG_FILE="${2:-}"; shift 2 ;;
    --help|-h)    usage; exit 0 ;;
    --)           shift; break ;;
    *) echo "memory-guard: unknown option '$1' (use -- before the command)" >&2; exit 2 ;;
  esac
done
if [ $# -eq 0 ]; then echo "memory-guard: no command given (usage: $0 [options] -- cmd args...)" >&2; exit 2; fi
for v in "$BUDGET_MB" "$FLOOR_PCT" "$INTERVAL_S" "$TIMEOUT_S"; do
  is_uint "$v" || { echo "memory-guard: option values must be non-negative integers (got '$v')" >&2; exit 2; }
done
[ "$INTERVAL_S" -ge 1 ] || { echo "memory-guard: --interval-s must be >= 1" >&2; exit 2; }

BUDGET_KB=$(( BUDGET_MB * 1024 ))
CMD_STR="$*"

# ---- logging ---------------------------------------------------------------
VERBOSITY="${PGEN_TRACE_VERBOSITY:-low}"
verbosity_rank() {
  case "$1" in
    none) echo 0 ;; low) echo 1 ;; medium) echo 2 ;; high) echo 3 ;; debug) echo 4 ;;
    *) echo 1 ;;
  esac
}
VRANK="$(verbosity_rank "$VERBOSITY")"

emit() { # emit <line> — unconditional (severity: warning/error/breach) + log file
  printf 'memory-guard: %s\n' "$1" >&2
  [ -n "$LOG_FILE" ] && printf 'memory-guard: %s\n' "$1" >> "$LOG_FILE"
  return 0
}
info() { # info <min-rank> <line> — verbosity-gated INFO only
  if [ "$VRANK" -ge "$1" ]; then emit "$2"; fi
  return 0
}

# ---- marker ----------------------------------------------------------------
write_marker() { # write_marker <status> <reason> <exit> <peak_rss_kb> <last_free_pct> <elapsed_s>
  local dir; dir="$(dirname "$MARKER")"
  mkdir -p "$dir" 2>/dev/null || true
  {
    printf 'status=%s\n'        "$1"
    printf 'reason=%s\n'        "$2"
    printf 'exit=%s\n'          "$3"
    printf 'budget_mb=%s\n'     "$BUDGET_MB"
    printf 'floor_pct=%s\n'     "$FLOOR_PCT"
    printf 'timeout_s=%s\n'     "$TIMEOUT_S"
    printf 'peak_rss_mb=%s\n'   "$(( ${4:-0} / 1024 ))"
    printf 'last_free_pct=%s\n' "${5:--}"
    printf 'elapsed_s=%s\n'     "${6:-0}"
    printf 'cmd=%s\n'           "$CMD_STR"
    printf 'ended_at=%s\n'      "$(date '+%Y-%m-%dT%H:%M:%S%z')"
  } > "$MARKER"
}

# ---- system free % ---------------------------------------------------------
free_pct() {
  local out
  if [ -n "${PGEN_MEMORY_GUARD_FAKE_FREE_PCT_FILE:-}" ]; then
    out="$(cat "$PGEN_MEMORY_GUARD_FAKE_FREE_PCT_FILE" 2>/dev/null | tr -d '[:space:]')"
  else
    out="$(memory_pressure -Q 2>/dev/null | sed -n 's/^System-wide memory free percentage: \([0-9][0-9]*\)%$/\1/p')"
  fi
  if is_uint "${out:-}"; then printf '%s\n' "$out"; else printf '\n'; fi
}

# ---- process-tree RSS ------------------------------------------------------
# One ps snapshot; sum RSS over (pgid == root) ∪ (ppid-descendant closure of root).
#
# ⛔ Membership tests MUST be `(p in mark)`, NEVER `mark[p]` — in awk, merely
# READING `mark[p]` auto-vivifies the key, after which `ppid[p] in mark` is true
# for every process on the system and the "tree" becomes the whole process
# table. That exact defect made the first guard build kill every user process
# on the host (OPS-MEMSAFE.1, 2026-07-14 13:04 session kill).
#
# FAIL-SAFE (defense in depth): if the walk ever marks pid 1, the guard itself,
# or an ancestor of the guard, the sample is INSANE — emit nothing so the
# caller treats it as a failed sample and NEVER drives a kill from it.
TREE_WALK_AWK='
  { pid[$1]=1; ppid[$1]=$2; pgid[$1]=$3; rss[$1]=$4 }
  END {
    mark[root]=1
    for (p in pid) if (pgid[p]==root) mark[p]=1
    changed=1
    while (changed) {
      changed=0
      for (p in pid) if (!(p in mark) && (ppid[p] in mark)) { mark[p]=1; changed=1 }
    }
    anc[guard]=1
    a=guard
    while ((a in ppid) && !(ppid[a] in anc) && ppid[a] > 0) { a=ppid[a]; anc[a]=1 }
    if (1 in mark) exit 0
    for (p in anc) if (p in mark) exit 0
    total=0; n=0; out=""
    for (p in mark) if (p in pid) { total+=rss[p]; n++; out = out p " " }
    if (mode=="rss") print total, n
    else print out
  }'

sample_tree_rss_kb() { # sample_tree_rss_kb <root-pid> → "TOTAL_KB NPIDS" (empty on ps failure/insane sample)
  ps -ax -o pid=,ppid=,pgid=,rss= 2>/dev/null | awk -v root="$1" -v guard="$$" -v mode=rss "$TREE_WALK_AWK"
}

list_tree_pids() { # list_tree_pids <root-pid> → space-separated pid list (same membership rule; empty on insane sample)
  ps -ax -o pid=,ppid=,pgid=,rss= 2>/dev/null | awk -v root="$1" -v guard="$$" -v mode=pids "$TREE_WALK_AWK"
}

# ---- kill escalation -------------------------------------------------------
# The kernel-scoped process-group kill (kill -- -root) is the PRIMARY kill; the
# walked pid list only supplements it (descendants that changed group but kept
# their ppid chain). Belt-and-braces: never signal pid <= 1 or the guard itself,
# even if the walk somehow returned them.
GRACE_S=5
signal_tree() { # signal_tree <sig> <root-pid> <pids...>
  local sig="$1" root="$2" p
  shift 2
  kill "-$sig" -- "-$root" 2>/dev/null
  for p in "$@"; do
    [ "$p" -gt 1 ] 2>/dev/null && [ "$p" != "$$" ] && [ "$p" != "$root" ] && kill "-$sig" "$p" 2>/dev/null
  done
  return 0
}
terminate_tree() { # terminate_tree <root-pid>
  local root="$1" pids waited
  pids="$(list_tree_pids "$root")"
  # shellcheck disable=SC2086
  signal_tree TERM "$root" $pids
  waited=0
  while [ "$waited" -lt "$GRACE_S" ] && kill -0 "$root" 2>/dev/null; do
    sleep 1; waited=$(( waited + 1 ))
  done
  pids="$(list_tree_pids "$root")"
  # shellcheck disable=SC2086
  signal_tree KILL "$root" $pids
}

# ---- pre-flight ------------------------------------------------------------
PRE_FREE="$(free_pct)"
if [ -z "$PRE_FREE" ]; then
  emit "WARNING: could not read system free memory (memory_pressure) — pre-flight check skipped"
elif [ "$PRE_FREE" -lt "$FLOOR_PCT" ]; then
  # marker path may reference the child pid; none exists yet — use the guard's pid
  MARKER="${MARKER:-$ROOT/rust/target/generated_logs/memory_guard/guard.$$.marker}"
  emit "PRE-FLIGHT REFUSED: system free ${PRE_FREE}% < floor ${FLOOR_PCT}% — NOT launching: $CMD_STR"
  write_marker "preflight-refused" "free-floor" "-" 0 "$PRE_FREE" 0
  emit "marker written: $MARKER"
  exit 96
fi

# ---- launch (own process group via job control) ----------------------------
set -m
"$@" &
CHILD=$!
set +m
MARKER="${MARKER:-$ROOT/rust/target/generated_logs/memory_guard/guard.$CHILD.marker}"
info 1 "started pid=$CHILD pgid=$CHILD budget=${BUDGET_MB}MB floor=${FLOOR_PCT}% interval=${INTERVAL_S}s timeout=${TIMEOUT_S}s free=${PRE_FREE:-?}% cmd: $CMD_STR"

PEAK_RSS_KB=0
LAST_FREE="$PRE_FREE"
START_S=$SECONDS

breach() { # breach <reason> <detail> <exit-code>
  trap - INT TERM   # no re-entry while we are already tearing the tree down
  emit "BREACH ($1): $2 — killing process tree of pid $CHILD (TERM, ${GRACE_S}s grace, KILL)"
  write_marker "killed" "$1" "$3" "$PEAK_RSS_KB" "${LAST_FREE:--}" "$(( SECONDS - START_S ))"
  terminate_tree "$CHILD"
  wait "$CHILD" 2>/dev/null
  emit "process tree killed; marker written: $MARKER"
  exit "$3"
}

on_guard_signal() {
  trap - INT TERM   # no re-entry while we are already tearing the tree down
  emit "guard received INT/TERM — killing the guarded tree (pid $CHILD)"
  write_marker "killed" "guard-interrupted" "130" "$PEAK_RSS_KB" "${LAST_FREE:--}" "$(( SECONDS - START_S ))"
  terminate_tree "$CHILD"
  wait "$CHILD" 2>/dev/null
  exit 130
}
trap on_guard_signal INT TERM

# ---- monitor loop ----------------------------------------------------------
while kill -0 "$CHILD" 2>/dev/null; do
  sleep "$INTERVAL_S" &
  wait $! 2>/dev/null   # interruptible sleep (so traps fire promptly)
  kill -0 "$CHILD" 2>/dev/null || break

  elapsed=$(( SECONDS - START_S ))

  if [ "$TIMEOUT_S" -gt 0 ] && [ "$elapsed" -ge "$TIMEOUT_S" ]; then
    breach "timeout" "elapsed ${elapsed}s >= timeout ${TIMEOUT_S}s" 99
  fi

  sample="$(sample_tree_rss_kb "$CHILD")"
  if [ -n "$sample" ]; then
    rss_kb="${sample%% *}"
    npids="${sample##* }"
    if is_uint "$rss_kb"; then
      [ "$rss_kb" -gt "$PEAK_RSS_KB" ] && PEAK_RSS_KB="$rss_kb"
      info 2 "sample: tree_rss=$(( rss_kb / 1024 ))MB pids=$npids elapsed=${elapsed}s free=${LAST_FREE:-?}%"
      if [ "$VRANK" -ge 4 ]; then
        ps -o pid,pgid,rss,command -g "$CHILD" 2>/dev/null | sed 's/^/memory-guard[debug]:   /' >&2 || true
      fi
      if [ "$rss_kb" -gt "$BUDGET_KB" ]; then
        breach "rss-budget" "tree RSS $(( rss_kb / 1024 ))MB > budget ${BUDGET_MB}MB (pids=$npids)" 97
      fi
    fi
  else
    emit "WARNING: process-tree RSS sample failed (ps) — retrying next interval"
  fi

  f="$(free_pct)"
  if [ -n "$f" ]; then
    LAST_FREE="$f"
    if [ "$f" -lt "$FLOOR_PCT" ]; then
      breach "free-floor" "system free ${f}% < floor ${FLOOR_PCT}%" 98
    fi
  else
    emit "WARNING: could not read system free memory this interval — floor check skipped"
  fi
done

# ---- normal completion -----------------------------------------------------
wait "$CHILD"
RC=$?
trap - INT TERM
write_marker "completed" "none" "$RC" "$PEAK_RSS_KB" "${LAST_FREE:--}" "$(( SECONDS - START_S ))"
info 1 "completed exit=$RC peak_tree_rss=$(( PEAK_RSS_KB / 1024 ))MB elapsed=$(( SECONDS - START_S ))s; marker: $MARKER"
exit "$RC"
