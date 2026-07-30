#!/usr/bin/env bash
# docs/tasks/artifacts/readme_policy/run_layer_c_reconcile_probes.sh
# README-POLICY.7 — RED / GREEN / CONTROL probes for the layer-C index reconcile in
# scripts/check_memory_architecture.sh (E2.5).
#
# ⭐ WHY CONTROLS, NOT JUST RED FIXTURES. The reconcile was adopted to replace a check that
# asserted only "the index has more than zero rows" — which passed at 135 records / 133 rows,
# two records invisible to their own index with the doctrine green. A replacement that is
# ALSO blind would look identical from the outside, so two arms execute the alternatives:
#   CTRL-1 runs the REAL RETIRED check, extracted from `git show HEAD:`, against a fixture the
#          new one rejects — and requires it to PASS. That is what proves the change was
#          necessary rather than cosmetic.
#   CTRL-2 runs a BARE-BASENAME matcher (the shape adopted from the spine repo) against a
#          record that has NO row but IS mentioned in a neighbouring row's prose — and
#          requires it to PASS while the shipped row-anchored form REJECTS. That is what
#          proves the row anchoring is load-bearing rather than stylistic. Measured in the
#          live index: `project_json_full_standard_proof.md` already occurs twice.
#
# ⛔ PATH-DEPTH TRAP: the guard resolves ROOT one level up from its own directory, so the
# fixture guard must sit at <fixture>/scripts/, never deeper.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"; cd "$ROOT"

GUARD_REL="scripts/check_memory_architecture.sh"
[ -f "$GUARD_REL" ] || { echo "probe: REFUSED — $GUARD_REL not found (ROOT=$ROOT)" >&2; exit 2; }

pass=0; fail=0
WORK="$(mktemp -d)"; trap 'rm -rf "$WORK"' EXIT

# Mini-root satisfying every OTHER leg (layer A within caps, bootstrap files, layer B), so
# only the layer-C reconcile varies.
#   $1 = name · $2 = guard to install
make_root() {
  local d="$WORK/$1" guard="$2"
  rm -rf "$d"; mkdir -p "$d/scripts" "$d/docs/tasks" "$d/docs/decisions"
  cp "$guard" "$d/scripts/check_memory_architecture.sh"
  printf '# standard\n'                 > "$d/MEMORY_ARCHITECTURE.md"
  printf 'see MEMORY_ARCHITECTURE.md\n' > "$d/AGENTS.md"
  printf 'see MEMORY_ARCHITECTURE.md\n' > "$d/CLAUDE.md"
  printf '# trees\n'                    > "$d/docs/TASK_TREE.md"
  printf '# MEMORY\n- state: ok\n'      > "$d/MEMORY.md"
  printf '# a\n' > "$d/docs/decisions/alpha.md"
  printf '# b\n' > "$d/docs/decisions/beta.md"
  {
    printf '# Decision Records\n\n| Record | Category | Summary |\n| --- | --- | --- |\n'
    printf '| [alpha.md](alpha.md) | project | the alpha record |\n'
    printf '| [beta.md](beta.md) | project | the beta record |\n'
  } > "$d/docs/decisions/INDEX.md"
  printf '%s' "$d"
}

probe() { # label · root · expected exit (or "nonzero") · expected substring
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

echo "== layer-C reconcile probes (check_memory_architecture.sh E2.5) =="

# ---------------------------------------------------------------- GREEN: a reconciled index
d="$(make_root green "$ROOT/$GUARD_REL")"
probe GREEN-1 "$d" 0 "memory-arch: OK"

# ---------------------------------------------------------------- RED-1: record with no row
d="$(make_root red1 "$ROOT/$GUARD_REL")"
printf '# g\n' > "$d/docs/decisions/gamma.md"          # on disk, absent from the index
probe RED-1 "$d" nonzero "record gamma.md has NO row"

# ---------------------------------------------------------------- RED-2: row with no record
d="$(make_root red2 "$ROOT/$GUARD_REL")"
printf '| [delta.md](delta.md) | project | names a record that does not exist |\n' \
  >> "$d/docs/decisions/INDEX.md"
probe RED-2 "$d" nonzero "which does not exist"

# ---------------------------------------------------------------- ⭐ CTRL-1: the retired check
# The REAL previous implementation, executed — not paraphrased — over the RED-1 fixture.
if git -C "$ROOT" show "HEAD:$GUARD_REL" > "$WORK/old_guard.sh" 2>/dev/null; then
  d="$(make_root ctrl1 "$WORK/old_guard.sh")"
  printf '# g\n' > "$d/docs/decisions/gamma.md"
  out="$(bash "$d/scripts/check_memory_architecture.sh" 2>&1)"; rc=$?
  if [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -qF 'memory-arch: OK'; then
    printf '  ✓ %-10s exit=0  the RETIRED check reports OK with a record missing from the index ⇒ BLIND\n' "CTRL-1"
    pass=$((pass+1))
  else
    printf '  ✗ %-10s exit=%s — expected the retired check to PASS the RED-1 fixture\n' "CTRL-1" "$rc"
    printf '%s\n' "$out" | sed 's/^/        /'; fail=$((fail+1))
  fi
else
  echo "  ! CTRL-1 UNJUDGEABLE — could not read HEAD:$GUARD_REL"; fail=$((fail+1))
fi

# ---------------------------------------------------------------- ⭐⭐ CTRL-2: bare basename
# The adopted spine version matches a basename ANYWHERE in the index. Build the exact shape
# that defeats it — a record with no row of its own, named inside a NEIGHBOUR's prose — and
# show the bare matcher passes it while the shipped row-anchored form rejects.
d="$(make_root ctrl2 "$ROOT/$GUARD_REL")"
printf '# g\n' > "$d/docs/decisions/gamma.md"
# gamma.md gets NO row, but is mentioned in beta's row prose (a cross-reference, as the live
# index genuinely does for project_json_full_standard_proof.md).
sed -i.bak 's|the beta record|the beta record, superseding gamma.md|' "$d/docs/decisions/INDEX.md"
rm -f "$d/docs/decisions/INDEX.md.bak"
bare_pass=0
grep -qF 'gamma.md' "$d/docs/decisions/INDEX.md" && bare_pass=1     # bedrock's matcher: PASSES
row_pass=0
grep -qE '^\| \[gamma\.md\]\(gamma\.md\)' "$d/docs/decisions/INDEX.md" && row_pass=1
out="$(bash "$d/scripts/check_memory_architecture.sh" 2>&1)"; rc=$?
if [ "$bare_pass" -eq 1 ] && [ "$row_pass" -eq 0 ] && [ "$rc" -ne 0 ]; then
  printf '  ✓ %-10s bare-basename matcher PASSES (prose mention) while the row-anchored form REJECTS ⇒ anchoring is load-bearing\n' "CTRL-2"
  pass=$((pass+1))
else
  printf '  ✗ %-10s bare=%s row=%s guard_exit=%s — fixture does not isolate the anchoring\n' \
    "CTRL-2" "$bare_pass" "$row_pass" "$rc"; fail=$((fail+1))
fi

# ---------------------------------------------------------------- CTRL-3: not vacuously green
d="$(make_root ctrl3 "$ROOT/$GUARD_REL")"; rm -f "$d/docs/decisions/INDEX.md"
probe CTRL-3 "$d" nonzero "INDEX.md"

echo
printf 'probes: %d pass / %d fail\n' "$pass" "$fail"
[ "$fail" -eq 0 ]
