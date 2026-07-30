#!/usr/bin/env bash
# docs/tasks/artifacts/readme_policy/run_readme_stability_probes.sh
# README-POLICY.1 — RED / GREEN / CONTROL probes for scripts/check_readme_stability.sh.
#
# ⭐ WHY A CONTROL ARM IS PART OF THE CHECK, not an extra: a size guard over an already-trimmed
# file returns 0 whether it is working or blind. CTRL-1 replays the REAL pre-trim README from
# git (git show HEAD:README.md — the actual 510-line / 48,811-byte artifact, never a lookalike)
# and requires the guard to REJECT it. Without that arm, a clean sweep and a broken detector are
# indistinguishable.
#
# ⭐⭐ CTRL-2 IS THE DESIGN JUSTIFICATION: it proves the byte cap is NOT redundant with the line
# cap, by constructing the exact shape that defeated the line-only guard elsewhere in this repo
# (few lines, enormous bytes — the MEMORY.md / README.md:115 class) and showing a line-only
# check passes it while the shipped both-caps check rejects it.
#
# Each probe builds a SYNTHETIC mini-root (<tmp>/scripts/, <tmp>/README.md, <tmp>/docs/reference/)
# so the guard's BASH_SOURCE-derived ROOT lands on the fixture and the real repository README is
# never touched. (The path-depth trap that cost two earlier sessions: the guard resolves ROOT one
# level up from its own directory, so the fixture MUST sit at <tmp>/, not deeper.)
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"; cd "$ROOT"

GUARD="scripts/check_readme_stability.sh"
POLICY_REL="docs/reference/PGEN_README_STABILITY_POLICY.md"
[ -f "$GUARD" ] || { echo "probe: REFUSED — $GUARD not found (ROOT=$ROOT)" >&2; exit 2; }

pass=0; fail=0
WORK="$(mktemp -d)"; trap 'rm -rf "$WORK"' EXIT

# Build a synthetic mini-root containing the guard, a policy doc, and a README fixture.
# $1 = fixture dir name, $2 = path to README content (or "" to omit), $3 = 1 to include policy
make_root() {
  local d="$WORK/$1"; shift
  local readme="$1"; shift
  local with_policy="$1"; shift
  rm -rf "$d"; mkdir -p "$d/scripts" "$d/docs/reference"
  cp "$ROOT/$GUARD" "$d/scripts/"
  [ -n "$readme" ] && cp "$readme" "$d/README.md"
  [ "$with_policy" = "1" ] && printf '# policy\n' > "$d/$POLICY_REL"
  printf '%s' "$d"
}

# $1 = label, $2 = root dir, $3 = expected exit (or "nonzero"), $4 = expected substring ("" = any)
probe() {
  local label="$1" dir="$2" want="$3" want_txt="${4:-}"
  local out rc
  out="$(bash "$dir/scripts/check_readme_stability.sh" 2>&1)"; rc=$?
  local ok=1
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

echo "== README stability probes =="

# ---------------------------------------------------------------- GREEN: the live landing page
d="$(make_root green "$ROOT/README.md" 1)"
probe GREEN-1 "$d" 0 "readme-stability: OK"

# ---------------------------------------------------------------- CTRL-1: the REAL pre-trim file
# Not a synthetic oversized file — the actual artifact the policy was adopted against.
if git -C "$ROOT" show HEAD:README.md > "$WORK/before_readme.md" 2>/dev/null; then
  b_lines=$(wc -l < "$WORK/before_readme.md" | tr -d ' ')
  b_bytes=$(wc -c < "$WORK/before_readme.md" | tr -d ' ')
  printf '  (CTRL-1 fixture: HEAD:README.md = %s lines, %s bytes)\n' "$b_lines" "$b_bytes"
  d="$(make_root ctrl1 "$WORK/before_readme.md" 1)"
  probe CTRL-1 "$d" nonzero "> cap"
else
  echo "  ! CTRL-1 UNJUDGEABLE — could not read HEAD:README.md"; fail=$((fail+1))
fi

# ---------------------------------------------------------------- RED-1: line cap alone
{ printf '# t\n%s\n' "$POLICY_REL"; for i in $(seq 1 300); do echo "line $i"; done; } > "$WORK/long.md"
d="$(make_root red1 "$WORK/long.md" 1)"
probe RED-1 "$d" nonzero "lines (> cap"

# ---------------------------------------------------------------- RED-2 / CTRL-2: byte cap alone
# ⭐ THE ARM THAT MATTERS. Shape: FEW lines, enormous bytes — the exact class that passes a
# line-only guard (README.md:115 was 4,369 bytes on one line; MEMORY.md passes 60/60 lines at
# 149,779 bytes). Built well under the LINE cap so only the BYTE cap can reject it.
{ printf '# t\n%s\n' "$POLICY_REL"
  for i in $(seq 1 20); do printf 'bullet %s: ' "$i"; head -c 900 /dev/zero | tr '\0' 'x'; printf '\n'; done
} > "$WORK/fat.md"
fat_lines=$(wc -l < "$WORK/fat.md" | tr -d ' '); fat_bytes=$(wc -c < "$WORK/fat.md" | tr -d ' ')
printf '  (RED-2 fixture: %s lines — UNDER the 220 line cap — but %s bytes)\n' "$fat_lines" "$fat_bytes"
d="$(make_root red2 "$WORK/fat.md" 1)"
probe RED-2 "$d" nonzero "bytes (> cap"

# CTRL-2: a LINE-ONLY check (the shape check_memory_architecture.sh uses today) PASSES that same
# fixture ⇒ the byte cap is load-bearing, not decorative.
if [ "$fat_lines" -le 220 ]; then
  printf '  ✓ %-10s line-only check would PASS the RED-2 fixture (%s <= 220) ⇒ byte cap is non-redundant\n' \
    "CTRL-2" "$fat_lines"; pass=$((pass+1))
else
  printf '  ✗ %-10s fixture exceeds the line cap too; it does not isolate the byte cap\n' "CTRL-2"; fail=$((fail+1))
fi

# ---------------------------------------------------------------- RED-3: changelog leakage
{ printf '# t\n%s\n' "$POLICY_REL"; echo "- demoted to Mostly Done on 2026-07-29 (DONE-BAR.2b)"; } > "$WORK/dated.md"
d="$(make_root red3 "$WORK/dated.md" 1)"
probe RED-3 "$d" nonzero "date-stamped"

# ---------------------------------------------------------------- RED-4: policy link dropped
printf '# t\nno link here\n' > "$WORK/nolink.md"
d="$(make_root red4 "$WORK/nolink.md" 1)"
probe RED-4 "$d" nonzero "no longer links"

# ---------------------------------------------------------------- REFUSE: a skip is never a pass
d="$(make_root ref1 "" 1)"
probe REFUSE-1 "$d" 2 "REFUSED"
d="$(make_root ref2 "$ROOT/README.md" 0)"
probe REFUSE-2 "$d" 2 "REFUSED"

echo
printf 'probes: %d pass / %d fail\n' "$pass" "$fail"
[ "$fail" -eq 0 ]
