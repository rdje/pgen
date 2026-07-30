#!/usr/bin/env bash
# docs/tasks/artifacts/readme_policy/run_layer_a_cap_probes.sh
# README-POLICY.2 — RED / GREEN / CONTROL probes for the layer-A byte cap added to
# scripts/check_memory_architecture.sh.
#
# ⭐ WHY THE CONTROL ARMS ARE THE POINT, NOT AN EXTRA.
#   A size guard run over a file that has just been trimmed returns 0 whether it WORKS or is
#   BLIND. So the probes do not merely construct oversized fixtures:
#     CTRL-1 replays the REAL pre-trim `HEAD:MEMORY.md` (60 lines / 138,403 bytes — the actual
#            artifact, never a lookalike) against the NEW guard and requires REJECTION.
#     CTRL-2 replays that SAME real file against the REAL OLD guard, extracted from
#            `git show HEAD:scripts/check_memory_architecture.sh`, and requires it to PASS.
#            ⭐⭐ This is the arm that proves the change was necessary rather than arguing it:
#            the retired guard is executed, not paraphrased, and it returns `memory-arch: OK`
#            over a 138 KB "bounded resume pointer". Session #228's discipline — a BEFORE/AFTER
#            must be EXTRACTED from the real script, never re-implemented from memory.
#     CTRL-3 shows the byte-only RED fixture would PASS a line-only check ⇒ non-redundant caps.
#     CTRL-4 proves the harness is not vacuously green: an unrelated leg of the same doctrine
#            still fires in the mini-root, so a passing GREEN-1 means the guard RAN.
#
# ⚠️ An ABSENT/PASS assertion is meaningless until you prove the haystack IS the haystack
#   (session #227). CTRL-2 therefore asserts the OLD guard printed its own success line, not
#   merely that it exited 0 — a guard that died early for an unrelated reason would also be 0.
#
# Each probe builds a SYNTHETIC mini-root so the guard's BASH_SOURCE-derived ROOT lands on the
# fixture and the real repository files are never touched. ⛔ PATH-DEPTH TRAP (cost two earlier
# sessions, twice in the same directory): the guard resolves ROOT ONE level up from its own
# directory, so the guard must sit at <fixture>/scripts/, never deeper.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"; cd "$ROOT"

GUARD_REL="scripts/check_memory_architecture.sh"
[ -f "$GUARD_REL" ] || { echo "probe: REFUSED — $GUARD_REL not found (ROOT=$ROOT)" >&2; exit 2; }

pass=0; fail=0
WORK="$(mktemp -d)"; trap 'rm -rf "$WORK"' EXIT

# Build a mini-root satisfying every OTHER leg of the doctrine, so only the caps vary.
#   $1 = fixture name · $2 = path to MEMORY.md content ("" to omit) · $3 = guard to install
make_root() {
  local d="$WORK/$1" mem="$2" guard="$3"
  rm -rf "$d"; mkdir -p "$d/scripts" "$d/docs/tasks" "$d/docs/decisions"
  cp "$guard" "$d/scripts/check_memory_architecture.sh"
  printf '# standard\n'                      > "$d/MEMORY_ARCHITECTURE.md"
  printf 'see MEMORY_ARCHITECTURE.md\n'      > "$d/AGENTS.md"
  printf 'see MEMORY_ARCHITECTURE.md\n'      > "$d/CLAUDE.md"
  printf '# trees\n'                         > "$d/docs/TASK_TREE.md"
  printf '# index\n| [a.md](a.md) | x | y |\n' > "$d/docs/decisions/INDEX.md"
  printf '# a\n'                             > "$d/docs/decisions/a.md"
  [ -n "$mem" ] && cp "$mem" "$d/MEMORY.md"
  printf '%s' "$d"
}

# $1 = label · $2 = root · $3 = expected exit (or "nonzero") · $4 = expected substring
probe() {
  local label="$1" dir="$2" want="$3" want_txt="${4:-}" out rc ok=1
  out="$(bash "$dir/scripts/check_memory_architecture.sh" 2>&1)"; rc=$?
  case "$want" in
    nonzero) [ "$rc" -ne 0 ] || ok=0 ;;
    *)       [ "$rc" -eq "$want" ] || ok=0 ;;
  esac
  if [ -n "$want_txt" ] && ! printf '%s' "$out" | grep -qF "$want_txt"; then ok=0; fi
  if [ "$ok" -eq 1 ]; then
    printf '  ✓ %-10s exit=%s  %s\n' "$label" "$rc" "${want_txt:-<any message>}"; pass=$((pass+1))
  else
    printf '  ✗ %-10s exit=%s (wanted %s / %s)\n' "$label" "$rc" "$want" "${want_txt:-<any>}"
    printf '%s\n' "$out" | sed 's/^/        /'; fail=$((fail+1))
  fi
}

echo "== layer-A cap probes (scripts/check_memory_architecture.sh) =="

# ---------------------------------------------------------------- GREEN-1: the live pointer
d="$(make_root green "$ROOT/MEMORY.md" "$ROOT/$GUARD_REL")"
probe GREEN-1 "$d" 0 "memory-arch: OK"

# ---------------------------------------------------------------- CTRL-4: not vacuously green
# Remove one unrelated leg (the layer-B index) from an otherwise-passing root. If GREEN-1 above
# were passing because the guard never really ran, this would pass too.
d="$(make_root ctrl4 "$ROOT/MEMORY.md" "$ROOT/$GUARD_REL")"; rm -f "$d/docs/TASK_TREE.md"
probe CTRL-4 "$d" nonzero "docs/TASK_TREE.md"

# ---------------------------------------------------------------- the REAL pre-trim artifact
if git -C "$ROOT" show HEAD:MEMORY.md > "$WORK/before_memory.md" 2>/dev/null; then
  b_lines=$(wc -l < "$WORK/before_memory.md" | tr -d ' ')
  b_bytes=$(wc -c < "$WORK/before_memory.md" | tr -d ' ')
  b_long=$(awk '{ if (length($0)+1 > m) m = length($0)+1 } END { print m }' "$WORK/before_memory.md")
  printf '  (real pre-trim fixture: HEAD:MEMORY.md = %s lines, %s bytes, longest line %s bytes)\n' \
    "$b_lines" "$b_bytes" "$b_long"

  # CTRL-1 — the NEW guard must reject it, and specifically on BYTES.
  d="$(make_root ctrl1 "$WORK/before_memory.md" "$ROOT/$GUARD_REL")"
  probe CTRL-1 "$d" nonzero "bytes (> cap"

  # ⭐⭐ CTRL-2 — the REAL OLD guard, extracted from git, must PASS the very same file.
  if git -C "$ROOT" show "HEAD:$GUARD_REL" > "$WORK/old_guard.sh" 2>/dev/null; then
    d="$(make_root ctrl2 "$WORK/before_memory.md" "$WORK/old_guard.sh")"
    out="$(bash "$d/scripts/check_memory_architecture.sh" 2>&1)"; rc=$?
    # Assert the haystack IS the haystack: it must have RUN and reported its own success.
    if [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -qF 'memory-arch: OK'; then
      printf '  ✓ %-10s exit=0  the RETIRED guard reports OK over %s bytes ⇒ it was BLIND\n' \
        "CTRL-2" "$b_bytes"; pass=$((pass+1))
    else
      printf '  ✗ %-10s exit=%s — expected the retired guard to PASS the pre-trim file\n' "CTRL-2" "$rc"
      printf '%s\n' "$out" | sed 's/^/        /'; fail=$((fail+1))
    fi
  else
    echo "  ! CTRL-2 UNJUDGEABLE — could not read HEAD:$GUARD_REL"; fail=$((fail+1))
  fi
else
  echo "  ! CTRL-1/CTRL-2 UNJUDGEABLE — could not read HEAD:MEMORY.md"; fail=$((fail+1))
fi

# ---------------------------------------------------------------- RED-1: the line cap
{ for i in $(seq 1 80); do echo "- line $i"; done; } > "$WORK/many_lines.md"
d="$(make_root red1 "$WORK/many_lines.md" "$ROOT/$GUARD_REL")"
probe RED-1 "$d" nonzero "lines (> cap"

# ---------------------------------------------------------------- RED-2 / CTRL-3: the byte cap
# Shape: FEW lines, enormous bytes — the exact class that defeated the line-only guard.
{ for i in $(seq 1 10); do printf -- '- entry %s: ' "$i"; head -c 1200 /dev/zero | tr '\0' 'x'; printf '\n'; done
} > "$WORK/fat.md"
f_lines=$(wc -l < "$WORK/fat.md" | tr -d ' '); f_bytes=$(wc -c < "$WORK/fat.md" | tr -d ' ')
printf '  (RED-2 fixture: %s lines — well UNDER the line cap — but %s bytes)\n' "$f_lines" "$f_bytes"
d="$(make_root red2 "$WORK/fat.md" "$ROOT/$GUARD_REL")"
probe RED-2 "$d" nonzero "bytes (> cap"

# CTRL-3 — a line-only check passes that same fixture ⇒ the byte cap is load-bearing.
line_cap="$(sed -nE 's/^CAP="\$\{MEMORY_POINTER_LINE_CAP:-([0-9]+)\}"/\1/p' "$ROOT/$GUARD_REL" | head -1)"
if [ -n "$line_cap" ] && [ "$f_lines" -le "$line_cap" ]; then
  printf '  ✓ %-10s a line-only check PASSES the RED-2 fixture (%s <= %s) ⇒ byte cap non-redundant\n' \
    "CTRL-3" "$f_lines" "$line_cap"; pass=$((pass+1))
else
  printf '  ✗ %-10s fixture does not isolate the byte cap (lines=%s cap=%s)\n' \
    "CTRL-3" "$f_lines" "${line_cap:-?}"; fail=$((fail+1))
fi

# ---------------------------------------------------------------- ABSENT: a missing pointer
d="$(make_root absent "" "$ROOT/$GUARD_REL")"
probe ABSENT-1 "$d" nonzero "MEMORY.md (layer A resume pointer) is missing"

echo
printf 'probes: %d pass / %d fail\n' "$pass" "$fail"
[ "$fail" -eq 0 ]
