#!/usr/bin/env bash
# scripts/run_with_memory_guard.sh — run a job under a host-RAM guard (OPS-MEMSAFE.1).
#
# Mechanical enforcement of the HOST-RAM BUDGET DIRECTIVE
# (docs/decisions/feedback_host_ram_budget_all_jobs.md, director 2026-07-14):
# on this 24 GB single-project machine, NO spawned job may exhaust host RAM.
#
# What it does:
#   1. PRE-FLIGHT: refuses to launch the job if system-wide free memory OR free disk
#      space (on the guard's working filesystem) is already below its floor
#      (never launch into a pressured system — OPS-MEMSAFE.2 added the disk axis
#      after the 2026-07-18 ENOSPC build death), or if the HOST is already in trouble
#      from something outside the job (item 5).
#   2. Runs the command in its OWN PROCESS GROUP and samples the job's process-tree
#      RSS (process-group members ∪ parent-child descendant closure) on an interval.
#   3. KILLS the whole tree (TERM → grace → KILL) and writes an observable breach
#      MARKER + an always-on log line when:
#        - the tree's summed RSS exceeds the budget (default 12288 MB ≈ half RAM), or
#        - system-wide free memory drops below the floor (default 10%), or
#        - free disk on the working filesystem drops below --disk-floor-gb (default 8), or
#        - the optional wall-clock timeout expires.
#   4. Always writes a completion marker (composes with the background-job
#      observability doctrine: completion marker + bounded timeout + liveness).
#   5. HOST HAZARD + HOST PRESSURE (OPS-MEMSAFE.6, after the 2026-10-20 incident): the
#      three kill axes see only what the job owns, and "free %" is not pressure —
#      measured that day: free=51% while swap stood at 22.06 of 23.55 GB and a
#      root-owned fseventsd (PPID 1, never inside any job tree) held 9–10 GB for hours
#      while two guarded mirror runs tripped nothing. So the guard (a) refuses to
#      LAUNCH while fseventsd's RSS is over --fseventsd-limit-mb or the swap-out rate
#      across a short window is at or over --swapout-floor, naming the cause; (b) while
#      the job runs, samples the swap-out rate, names the largest process OUTSIDE the
#      job's tree when the host is under pressure, and records both in the marker;
#      (c) warns when the budget exceeds half of physical RAM. It NEVER kills on host
#      pressure: a guard kills only what it owns (the 2026-07-14 session kill is why),
#      and what it records is exactly what a kill rule would need (the tree's RSS at
#      the pressure peak, the unowned consumer) — see docs/tasks/OPS-MEMSAFE.md leaf .6.
#
# Exit codes (mechanically branchable by callers):
#   child's own exit code   — the job completed on its own (guard transparent)
#   96  preflight-refused   — free RAM or free disk already below its floor, or a host
#                             hazard / host pressure present at launch; job NEVER
#                             started (marker reason= free-floor | disk-floor |
#                             host-hazard | host-pressure)
#   97  rss-budget breach   — tree RSS exceeded --budget-mb
#   98  free-floor breach   — system free % dropped below --floor-pct mid-run
#   95  disk-floor breach   — free disk dropped below --disk-floor-gb mid-run
#   99  timeout             — --timeout-s expired
#   130 guard-interrupted   — the guard itself received INT/TERM (tree killed too)
#   94  (reserved)          — a swap-out breach kill; NOT implemented: OPS-MEMSAFE.6
#                             observes and records, the kill rule waits on that data
#   2   usage error
#
# Usage:
#   scripts/run_with_memory_guard.sh [options] -- <command> [args...]
# Options:
#   --budget-mb N     process-tree RSS budget in MB        (default 12288)
#   --floor-pct N     minimum system-wide free memory %    (default 10)
#   --disk-floor-gb N minimum free disk (GB) on the guard's working filesystem,
#                     0=disable the disk axis               (default 8)
#   --interval-s N    sampling interval in seconds         (default 5)
#   --timeout-s N     wall-clock timeout in seconds, 0=off (default 0)
#   --fseventsd-limit-mb N  pre-flight: refuse to launch while the FSEvents journal daemon
#                     holds more than N MB RSS, 0=off           (default 512)
#   --swapout-floor N swap-out pages/s that count as host pressure, 0=off (default 1000):
#                     pre-flight refuses to launch at or over it across a 3 s window; in
#                     flight, two consecutive samples at or over it print ONE warning per
#                     episode naming the largest process outside the job's tree —
#                     recorded in the marker, never a kill
#   --marker FILE     marker file path (default rust/target/generated_logs/memory_guard/guard.<pid>.marker)
#   --log FILE        also append guard lines to FILE
#
# Verbosity: PGEN_TRACE_VERBOSITY ∈ none|low|medium|high|debug (default low).
#   Periodic sample lines print at medium+; per-PID breakdown at debug.
#   Breaches, kills, and warnings print UNCONDITIONALLY — severity is never gated
#   by verbosity (docs/decisions/feedback_severity_never_gated_by_verbosity.md).
#
# Test seams (deterministic floor-path testing only):
#   PGEN_MEMORY_GUARD_FAKE_FREE_PCT_FILE=<file>     — read the free RAM % from <file>
#   (re-read every sample) instead of `memory_pressure -Q`.
#   PGEN_MEMORY_GUARD_FAKE_DISK_FREE_GB_FILE=<file> — read the free disk GB from
#   <file> (re-read every sample) instead of `df -Pk .`.
#   PGEN_MEMORY_GUARD_FAKE_SWAPOUTS_FILE=<file>     — read the cumulative swap-out page
#   count from <file> (re-read every sample) instead of `vm_stat`.
#   PGEN_MEMORY_GUARD_FAKE_FSEVENTSD_RSS_MB_FILE=<file> — read the FSEvents daemon's RSS
#   (MB) from <file> instead of `pgrep` + `ps`.
#   PGEN_MEMORY_GUARD_FAKE_MEMSIZE_MB_FILE=<file>   — read physical RAM (MB) from <file>
#   instead of `sysctl -n hw.memsize`.
#
# Honest limit: a descendant that re-parents AND changes its own process group
# escapes both accounting and the kill. Jobs PGEN spawns (make/cargo/bench/parse)
# do neither. This is bounded-blast-radius enforcement, not a kernel limit.
#
# PARSE-BEFORE-RUN (OPS-MEMSAFE.6, after push-mirror run 10 of 2026-10-20): the whole
# body below is ONE function, called on the last line, and every path out of it is an
# `exit`. bash reads a script incrementally, by file offset, as it executes; the guard
# of run 10 had its script rewritten in place under it (by an actor outside the session,
# discarded by git at 12:33:03) and died with a syntax error at the monitor loop's end,
# 6 h 48 min in, losing the run's marker. With the body in a function, bash has parsed
# the entire file before the job is launched and never reads it again.
main() {
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

BUDGET_MB=12288
FLOOR_PCT=10
DISK_FLOOR_GB=8
INTERVAL_S=5
TIMEOUT_S=0
MARKER=""
LOG_FILE=""
FSEVENTSD_LIMIT_MB=512
SWAPOUT_FLOOR_PS=1000
PRE_SWAP_WINDOW_S=3
# host-pressure observation state (OPS-MEMSAFE.6) — every marker carries it
SWAP_AXIS_ON=1
SWAP_AXIS_STATE="on"
SWAP_PREV=""
SWAP_PREV_S=0
SWAP_HIGH_RUN=0
LAST_SWAP_PS=""
PEAK_SWAP_PS=""
PRESSURE_SAMPLES=0
TREE_MB_AT_PEAK_SWAP=""
UNOWNED_TOP=""
PRE_FSEVENTSD=""
PRE_SWAP_PS=""

usage() { sed -n '2,/^set -uo pipefail$/p' "${BASH_SOURCE[0]}" | grep -E '^# ?' | sed 's/^# \{0,1\}//'; }

is_uint() { [[ "$1" =~ ^[0-9]+$ ]]; }

while [ $# -gt 0 ]; do
  case "$1" in
    --budget-mb)  BUDGET_MB="${2:-}"; shift 2 ;;
    --floor-pct)  FLOOR_PCT="${2:-}"; shift 2 ;;
    --disk-floor-gb) DISK_FLOOR_GB="${2:-}"; shift 2 ;;
    --interval-s) INTERVAL_S="${2:-}"; shift 2 ;;
    --timeout-s)  TIMEOUT_S="${2:-}"; shift 2 ;;
    --fseventsd-limit-mb) FSEVENTSD_LIMIT_MB="${2:-}"; shift 2 ;;
    --swapout-floor) SWAPOUT_FLOOR_PS="${2:-}"; shift 2 ;;
    --marker)     MARKER="${2:-}"; shift 2 ;;
    --log)        LOG_FILE="${2:-}"; shift 2 ;;
    --help|-h)    usage; exit 0 ;;
    --)           shift; break ;;
    *) echo "memory-guard: unknown option '$1' (use -- before the command)" >&2; exit 2 ;;
  esac
done
if [ $# -eq 0 ]; then echo "memory-guard: no command given (usage: $0 [options] -- cmd args...)" >&2; exit 2; fi
for v in "$BUDGET_MB" "$FLOOR_PCT" "$DISK_FLOOR_GB" "$INTERVAL_S" "$TIMEOUT_S" "$FSEVENTSD_LIMIT_MB" "$SWAPOUT_FLOOR_PS"; do
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
    printf 'disk_floor_gb=%s\n' "$DISK_FLOOR_GB"
    printf 'timeout_s=%s\n'     "$TIMEOUT_S"
    printf 'peak_rss_mb=%s\n'   "$(( ${4:-0} / 1024 ))"
    printf 'last_free_pct=%s\n' "${5:--}"
    printf 'last_disk_free_gb=%s\n' "${7:--}"
    printf 'host_pressure_axis=%s\n' "$SWAP_AXIS_STATE"
    printf 'swapout_floor_ps=%s\n' "$SWAPOUT_FLOOR_PS"
    printf 'last_swapout_ps=%s\n'  "${LAST_SWAP_PS:--}"
    printf 'peak_swapout_ps=%s\n'  "${PEAK_SWAP_PS:--}"
    printf 'pressure_samples=%s\n' "$PRESSURE_SAMPLES"
    printf 'tree_mb_at_peak_swapout=%s\n' "${TREE_MB_AT_PEAK_SWAP:--}"
    printf 'unowned_top=%s\n'      "${UNOWNED_TOP:--}"
    printf 'fseventsd_rss_mb=%s\n' "${PRE_FSEVENTSD:--}"
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

# ---- free disk (GB, guard's working filesystem) ----------------------------
# POSIX `df -Pk .` guarantees line 2 column 4 = available 1K-blocks. The guard
# is launched from the directory the job builds under (repo convention), so
# "." is the filesystem the job will fill.
free_disk_gb() {
  local out
  if [ -n "${PGEN_MEMORY_GUARD_FAKE_DISK_FREE_GB_FILE:-}" ]; then
    out="$(cat "$PGEN_MEMORY_GUARD_FAKE_DISK_FREE_GB_FILE" 2>/dev/null | tr -d '[:space:]')"
  else
    out="$(df -Pk . 2>/dev/null | awk 'NR==2 { print int($4 / 1048576) }')"
  fi
  if is_uint "${out:-}"; then printf '%s\n' "$out"; else printf '\n'; fi
}

# ---- host pressure: the swap-out counter (OPS-MEMSAFE.6) --------------------
# `memory_pressure -Q`'s free % counts purgeable and cached pages as free, so it read
# 51 % while swap stood at 22.06 of 23.55 GB (2026-10-20). vm_stat's cumulative
# swap-out page count is the figure that moved; its RATE (pages/s) is the axis.
swapouts_pages() {
  local out
  if [ -n "${PGEN_MEMORY_GUARD_FAKE_SWAPOUTS_FILE:-}" ]; then
    out="$(cat "$PGEN_MEMORY_GUARD_FAKE_SWAPOUTS_FILE" 2>/dev/null | tr -d '[:space:]')"
  else
    out="$(vm_stat 2>/dev/null | awk '/^Swapouts:/ { gsub(/[^0-9]/, "", $2); print $2 }')"
  fi
  if is_uint "${out:-}"; then printf '%s\n' "$out"; else printf '\n'; fi
}

# ---- host hazard: the FSEvents journal daemon's RSS (MB) ---------------------
# Root-owned, PPID 1, outside every job tree; it held 9–10 GB for hours on 2026-10-20.
# Empty when there is no such daemon here (a Linux runner) — then NOT EVALUATED, never a verdict.
fseventsd_rss_mb() {
  local out="" pid
  if [ -n "${PGEN_MEMORY_GUARD_FAKE_FSEVENTSD_RSS_MB_FILE:-}" ]; then
    out="$(cat "$PGEN_MEMORY_GUARD_FAKE_FSEVENTSD_RSS_MB_FILE" 2>/dev/null | tr -d '[:space:]')"
  else
    pid="$(pgrep -x fseventsd 2>/dev/null | head -n 1)"
    [ -n "$pid" ] && out="$(ps -o rss= -p "$pid" 2>/dev/null | awk '{ printf "%d", $1 / 1024 }')"
  fi
  if is_uint "${out:-}"; then printf '%s\n' "$out"; else printf '\n'; fi
}

# ---- physical RAM (MB) — the budget's reference ------------------------------
physical_ram_mb() {
  local out
  if [ -n "${PGEN_MEMORY_GUARD_FAKE_MEMSIZE_MB_FILE:-}" ]; then
    out="$(cat "$PGEN_MEMORY_GUARD_FAKE_MEMSIZE_MB_FILE" 2>/dev/null | tr -d '[:space:]')"
  else
    out="$(sysctl -n hw.memsize 2>/dev/null | awk '{ printf "%d", $1 / 1048576 }')"
  fi
  if is_uint "${out:-}"; then printf '%s\n' "$out"; else printf '\n'; fi
}

# ---- the largest process OUTSIDE a pid set -----------------------------------
# top_unowned "<pids to exclude>" → "name:MB" of the biggest process that is neither in
# the set nor the guard (empty when ps fails). Evidence only: it names what the guard
# cannot own, so a pressure episode is attributable after the fact.
top_unowned() {
  ps -ax -o pid=,rss=,comm= 2>/dev/null | awk -v excl="$1" -v guard="$$" '
    BEGIN { n = split(excl, e, " "); for (i = 1; i <= n; i++) if (e[i] != "") own[e[i]] = 1; own[guard] = 1 }
    {
      if ($1 in own) next
      if ($2 + 0 > best) {
        best = $2 + 0
        name = $0
        sub(/^[ \t]*[0-9]+[ \t]+[0-9]+[ \t]+/, "", name)
        sub(/.*\//, "", name)
      }
    }
    END { if (best > 0) printf "%s:%d\n", name, best / 1024 }'
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
PRE_DISK="$(free_disk_gb)"
if [ -z "$PRE_FREE" ]; then
  emit "WARNING: could not read system free memory (memory_pressure) — pre-flight check skipped"
elif [ "$PRE_FREE" -lt "$FLOOR_PCT" ]; then
  # marker path may reference the child pid; none exists yet — use the guard's pid
  MARKER="${MARKER:-$ROOT/rust/target/generated_logs/memory_guard/guard.$$.marker}"
  emit "PRE-FLIGHT REFUSED: system free ${PRE_FREE}% < floor ${FLOOR_PCT}% — NOT launching: $CMD_STR"
  write_marker "preflight-refused" "free-floor" "-" 0 "$PRE_FREE" 0 "${PRE_DISK:--}"
  emit "marker written: $MARKER"
  exit 96
fi
if [ "$DISK_FLOOR_GB" -gt 0 ]; then
  if [ -z "$PRE_DISK" ]; then
    emit "WARNING: could not read free disk space (df) — disk pre-flight check skipped"
  elif [ "$PRE_DISK" -lt "$DISK_FLOOR_GB" ]; then
    MARKER="${MARKER:-$ROOT/rust/target/generated_logs/memory_guard/guard.$$.marker}"
    emit "PRE-FLIGHT REFUSED: free disk ${PRE_DISK}GB < floor ${DISK_FLOOR_GB}GB — NOT launching: $CMD_STR"
    write_marker "preflight-refused" "disk-floor" "-" 0 "${PRE_FREE:--}" 0 "$PRE_DISK"
    emit "marker written: $MARKER"
    exit 96
  fi
fi

# ---- pre-flight: host hazard + host pressure (OPS-MEMSAFE.6) -----------------
# Both axes above see only what the job will own, and a root-owned daemon outside every
# job tree is invisible to them; "free %" read 51 % while swap stood at 22 of 23.5 GB
# (2026-10-20). Refuse to launch into that, and name the cause instead of grinding.
PRE_FSEVENTSD="$(fseventsd_rss_mb)"
if [ "$FSEVENTSD_LIMIT_MB" -gt 0 ]; then
  if [ -z "$PRE_FSEVENTSD" ]; then
    info 2 "host-hazard pre-flight NOT EVALUATED: no fseventsd RSS readable here (no such daemon, or no pgrep/ps)"
  elif [ "$PRE_FSEVENTSD" -gt "$FSEVENTSD_LIMIT_MB" ]; then
    MARKER="${MARKER:-$ROOT/rust/target/generated_logs/memory_guard/guard.$$.marker}"
    emit "PRE-FLIGHT REFUSED: host hazard fseventsd=${PRE_FSEVENTSD}MB > ${FSEVENTSD_LIMIT_MB}MB — a leaking FSEvents journal daemon starves this job and the whole host, and no axis of this guard can see or stop it (docs/decisions/feedback_host_ram_budget_all_jobs.md) — NOT launching: $CMD_STR"
    write_marker "preflight-refused" "host-hazard" "-" 0 "${PRE_FREE:--}" 0 "${PRE_DISK:--}"
    emit "marker written: $MARKER"
    exit 96
  fi
fi
if [ "$SWAPOUT_FLOOR_PS" -gt 0 ]; then
  s0="$(swapouts_pages)"
  if [ -z "$s0" ]; then
    emit "WARNING: could not read the swap-out counter (vm_stat) — host-pressure axis NOT EVALUATED on this host"
    SWAP_AXIS_ON=0; SWAP_AXIS_STATE="not-evaluated"
  else
    sleep "$PRE_SWAP_WINDOW_S"
    s1="$(swapouts_pages)"
    if [ -n "$s1" ] && [ "$s1" -ge "$s0" ]; then
      PRE_SWAP_PS=$(( (s1 - s0) / PRE_SWAP_WINDOW_S ))
      if [ "$PRE_SWAP_PS" -ge "$SWAPOUT_FLOOR_PS" ]; then
        MARKER="${MARKER:-$ROOT/rust/target/generated_logs/memory_guard/guard.$$.marker}"
        PEAK_SWAP_PS="$PRE_SWAP_PS"; LAST_SWAP_PS="$PRE_SWAP_PS"; PRESSURE_SAMPLES=1
        UNOWNED_TOP="$(top_unowned "")"
        emit "PRE-FLIGHT REFUSED: host pressure — swap-out ${PRE_SWAP_PS} pages/s >= floor ${SWAPOUT_FLOOR_PS} across ${PRE_SWAP_WINDOW_S}s (free=${PRE_FREE:-?}% is not pressure; the largest process on the host: ${UNOWNED_TOP:-?}) — NOT launching: $CMD_STR"
        write_marker "preflight-refused" "host-pressure" "-" 0 "${PRE_FREE:--}" 0 "${PRE_DISK:--}"
        emit "marker written: $MARKER"
        exit 96
      fi
    fi
    SWAP_PREV="${s1:-$s0}"; SWAP_PREV_S=$SECONDS
  fi
else
  SWAP_AXIS_ON=0; SWAP_AXIS_STATE="off"
fi
RAM_MB="$(physical_ram_mb)"
if [ -n "$RAM_MB" ] && [ "$BUDGET_MB" -gt $(( RAM_MB / 2 )) ]; then
  emit "WARNING: budget ${BUDGET_MB}MB exceeds half of physical RAM (${RAM_MB}MB) — the guard protects the host LESS at this budget (OPS-MEMSAFE.4); derive it from measured peaks, budget = min(max_peak x 1.2, RAM / 2), and reduce the peak rather than raise the budget"
fi

# ---- launch (own process group via job control) ----------------------------
set -m
"$@" &
CHILD=$!
set +m
MARKER="${MARKER:-$ROOT/rust/target/generated_logs/memory_guard/guard.$CHILD.marker}"
info 1 "started pid=$CHILD pgid=$CHILD budget=${BUDGET_MB}MB floor=${FLOOR_PCT}% disk_floor=${DISK_FLOOR_GB}GB interval=${INTERVAL_S}s timeout=${TIMEOUT_S}s free=${PRE_FREE:-?}% disk=${PRE_DISK:-?}GB fseventsd=${PRE_FSEVENTSD:-?}MB swapout=${PRE_SWAP_PS:-?}/s (floor ${SWAPOUT_FLOOR_PS}, axis ${SWAP_AXIS_STATE}) cmd: $CMD_STR"

PEAK_RSS_KB=0
LAST_FREE="$PRE_FREE"
LAST_DISK="$PRE_DISK"
START_S=$SECONDS

breach() { # breach <reason> <detail> <exit-code>
  trap - INT TERM   # no re-entry while we are already tearing the tree down
  emit "BREACH ($1): $2 — killing process tree of pid $CHILD (TERM, ${GRACE_S}s grace, KILL)"
  write_marker "killed" "$1" "$3" "$PEAK_RSS_KB" "${LAST_FREE:--}" "$(( SECONDS - START_S ))" "${LAST_DISK:--}"
  terminate_tree "$CHILD"
  wait "$CHILD" 2>/dev/null
  emit "process tree killed; marker written: $MARKER"
  exit "$3"
}

on_guard_signal() {
  trap - INT TERM   # no re-entry while we are already tearing the tree down
  emit "guard received INT/TERM — killing the guarded tree (pid $CHILD)"
  write_marker "killed" "guard-interrupted" "130" "$PEAK_RSS_KB" "${LAST_FREE:--}" "$(( SECONDS - START_S ))" "${LAST_DISK:--}"
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

  if [ "$DISK_FLOOR_GB" -gt 0 ]; then
    d="$(free_disk_gb)"
    if [ -n "$d" ]; then
      LAST_DISK="$d"
      if [ "$d" -lt "$DISK_FLOOR_GB" ]; then
        breach "disk-floor" "free disk ${d}GB < floor ${DISK_FLOOR_GB}GB" 95
      fi
    else
      emit "WARNING: could not read free disk space this interval — disk-floor check skipped"
    fi
  fi

  # host pressure (OPS-MEMSAFE.6): observed and recorded, never a kill — the job may
  # not be the cause, and a guard kills only what it owns.
  if [ "$SWAP_AXIS_ON" -eq 1 ]; then
    s_now="$(swapouts_pages)"
    if [ -n "$s_now" ]; then
      if [ -n "$SWAP_PREV" ] && [ "$s_now" -ge "$SWAP_PREV" ]; then
        dt=$(( SECONDS - SWAP_PREV_S )); [ "$dt" -lt 1 ] && dt=1
        rate=$(( (s_now - SWAP_PREV) / dt ))
        LAST_SWAP_PS="$rate"
        if [ -z "$PEAK_SWAP_PS" ] || [ "$rate" -gt "$PEAK_SWAP_PS" ]; then   # a run of zeros has a peak of 0, not "-"
          PEAK_SWAP_PS="$rate"; TREE_MB_AT_PEAK_SWAP=$(( ${rss_kb:-0} / 1024 ))
        fi
        if [ "$rate" -ge "$SWAPOUT_FLOOR_PS" ]; then
          PRESSURE_SAMPLES=$(( PRESSURE_SAMPLES + 1 ))
          SWAP_HIGH_RUN=$(( SWAP_HIGH_RUN + 1 ))
          if [ "$SWAP_HIGH_RUN" -eq 2 ]; then
            top="$(top_unowned "$(list_tree_pids "$CHILD")")"
            [ -n "$top" ] && UNOWNED_TOP="$top"
            emit "WARNING (host-pressure): swap-out ${rate} pages/s >= floor ${SWAPOUT_FLOOR_PS} on two consecutive samples — tree_rss=$(( ${rss_kb:-0} / 1024 ))MB of budget ${BUDGET_MB}MB, free=${LAST_FREE:-?}% (free % is not pressure), largest process outside the tree: ${top:-?} — observed and recorded, NOT a kill (a guard kills only what it owns)"
          fi
        else
          SWAP_HIGH_RUN=0
        fi
      fi
      SWAP_PREV="$s_now"; SWAP_PREV_S=$SECONDS
    else
      emit "WARNING: could not read the swap-out counter this interval — host-pressure sample skipped"
    fi
  fi
done

# ---- normal completion -----------------------------------------------------
wait "$CHILD"
RC=$?
trap - INT TERM
write_marker "completed" "none" "$RC" "$PEAK_RSS_KB" "${LAST_FREE:--}" "$(( SECONDS - START_S ))" "${LAST_DISK:--}"
info 1 "completed exit=$RC peak_tree_rss=$(( PEAK_RSS_KB / 1024 ))MB elapsed=$(( SECONDS - START_S ))s peak_swapout=${PEAK_SWAP_PS:--}/s pressure_samples=${PRESSURE_SAMPLES}; marker: $MARKER"
exit "$RC"
}

main "$@"
