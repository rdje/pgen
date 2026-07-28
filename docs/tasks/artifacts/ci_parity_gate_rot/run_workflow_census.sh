#!/usr/bin/env bash
# docs/tasks/artifacts/ci_parity_gate_rot/run_workflow_census.sh
#
# CI-PARITY-GATE-ROT.3 — run EVERY `run_workflow` replay of the local workflow-parity gate
# individually and report PASS/FAIL + wall time per workflow.
#
# ⭐ WHY THIS EXISTS. `.1` fixed the gate's AUDIT phase (23/8 → 32/32) and the tree then had to
# state plainly that *"audit phase 32/32" is NOT "the gate completes"*: `main()` continues into
# `copy_tracked_worktree` + ELEVEN `run_workflow` replays that had never been executed once in
# this campaign. `run_workflow` calls `fail`, which `exit`s, so the gate can only ever name ONE
# broken replay per run — the identical fail-fast blindness that hid a third of `.1`'s stale
# assertions. This driver isolates each replay in its own subshell so the WHOLE census is visible.
#
# ⭐ IT SOURCES THE GATE, IT DOES NOT RE-IMPLEMENT IT. The replay list, the four arguments of each
# replay, the `assert_workflow_command` drift check, the export-dir cwd and the `CARGO_NET_OFFLINE`
# handling are all the gate's own definitions, taken live. A driver that hand-copies them measures
# a different thing than the gate applies — the defect `GENERATED-LINT-CORRECTNESS.4` had to fix in
# `.3`'s probe driver. The only overrides are:
#   - `run_workflow`  → wrapped (original kept as `orig_run_workflow`) so a failure is RECORDED,
#                       not fatal;
#   - `audit_*`       → no-ops, because `.1`/`.1b` already prove the audit phase 32/32 and this
#                       leaf is about the phase AFTER it. Re-running them here would only make the
#                       census slower, not truer.
# `copy_tracked_worktree` is NOT overridden: the export dir — `git ls-files` output ONLY, with
# `generated/` untracked and therefore ABSENT — is the very condition under test.
#
# ⛔ REPRODUCTION TRAP INHERITED FROM `.1`'s run_audit_census.sh, and it applies unchanged:
# `ci_workflow_local_gate.sh` locates itself with `${BASH_SOURCE[0]}/../..`, so the stripped copy
# MUST live in `rust/scripts/` or ROOT_DIR resolves one directory off and every replay runs against
# a phantom tree. The tell is a doubled `rust/rust/target/...`.
#
# ⛔ HEAVY. The replays invoke full `make` gates with cargo builds. Run under the host-RAM guard:
#   scripts/run_with_memory_guard.sh --budget-mb 12288 --timeout-s 7200 -- \
#     bash docs/tasks/artifacts/ci_parity_gate_rot/run_workflow_census.sh
#
# Scope it with the gate's own filter (comma-separated workflow names) to stage the cost:
#   PGEN_CI_WORKFLOW_LOCAL_FILTER=mdbook-docs-gate,annotation-contract-gate bash <this>
#
# Exit 0 always (a measuring instrument, not a gate); the census is the output.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"; cd "$ROOT"

GATE="rust/scripts/ci_workflow_local_gate.sh"
[ -f "$GATE" ] || { echo "census: ✗ gate not found: $GATE" >&2; exit 1; }

# Retain the run dir: the per-replay logs ARE the evidence this leaf is collecting.
export PGEN_CI_WORKFLOW_LOCAL_KEEP_RUNS=1

# The stripped copy must sit at the SAME DEPTH as the original (trap above).
PROBE="rust/scripts/.ciparity_workflow_probe.sh"
sed 's/^main "\$@"$//' "$GATE" > "$PROBE"
trap 'rm -f "$ROOT/$PROBE"' EXIT

# shellcheck disable=SC1090
. "$ROOT/$PROBE"
set +e

# ⛔ THIRD REPRODUCTION TRAP, and this one was found by the repo being DIRTY afterwards.
# `bash` keeps ONE EXIT trap per shell, and the gate installs its own on line 67
# (`trap cleanup_run_dir_on_exit EXIT`). Sourcing it therefore SILENTLY REPLACES the driver's
# `rm -f "$PROBE"` trap, and the stripped copy survives in `rust/scripts/` — untracked cruft in a
# TRACKED directory, i.e. an instrument that leaves the working tree unclean every time it runs.
# `.1`'s run_audit_census.sh has the identical latent bug for the identical reason. Re-install a
# trap that does BOTH, in that order, so the probe is removed whatever the gate's own cleanup does.
trap 'rm -f "$ROOT/$PROBE"; cleanup_run_dir_on_exit' EXIT

# Keep the gate's own replay body; only its fatality is removed.
eval "$(declare -f run_workflow | sed '1s/^run_workflow/orig_run_workflow/')"

# The gate's closing line asserts a verdict this driver deliberately does NOT enforce — it keeps
# walking past a failure. Left alone, a transcript would read "all selected local workflow commands
# passed" directly above a FAIL row. Rewrite that ONE line; every other `note` is preserved.
eval "$(declare -f note | sed '1s/^note/orig_note/')"
note() {
  if [ "$*" = "all selected local workflow commands passed" ]; then
    orig_note "census: workflow phase walked to completion (the verdict is the table below)"
    return 0
  fi
  orig_note "$@"
}

CENSUS_PASS=0
CENSUS_FAIL=0
CENSUS_SKIP=0
CENSUS_ROWS=()

run_workflow() {
  local workflow_name="$1"
  local started elapsed rc out

  # ⛔ FOURTH TRAP, and the gate's OWN new check is what exposed it.
  # `orig_run_workflow` maintains `WORKFLOW_ROSTER` / `WORKFLOWS_RUN_COUNT` so the gate can refuse a
  # run that replayed nothing — but this driver calls it inside `$( ( … ) )`, and a SUBSHELL cannot
  # write its parent's variables. The very isolation that lets the census survive a failing replay
  # discards the bookkeeping, so `assert_workflow_selection_was_real` saw 0 and failed the driver.
  # Mirrored HERE, in the parent, rather than stubbing the assertion out: the census should be
  # subject to the same "did anything actually run?" check as the gate it measures.
  WORKFLOW_ROSTER="$WORKFLOW_ROSTER $workflow_name"

  if ! is_selected "$workflow_name"; then
    CENSUS_SKIP=$((CENSUS_SKIP + 1))
    CENSUS_ROWS+=("SKIP  $workflow_name")
    return 0
  fi
  WORKFLOWS_RUN_COUNT=$((WORKFLOWS_RUN_COUNT + 1))
  started=$SECONDS
  out=$( (set +e; orig_run_workflow "$@") 2>&1 ); rc=$?
  elapsed=$((SECONDS - started))
  if [ "$rc" -eq 0 ]; then
    CENSUS_PASS=$((CENSUS_PASS + 1))
    CENSUS_ROWS+=("$(printf 'PASS  %-42s %5ds' "$workflow_name" "$elapsed")")
    printf 'PASS  %-42s %5ds\n' "$workflow_name" "$elapsed"
  else
    CENSUS_FAIL=$((CENSUS_FAIL + 1))
    CENSUS_ROWS+=("$(printf 'FAIL  %-42s %5ds' "$workflow_name" "$elapsed")")
    printf 'FAIL  %-42s %5ds\n' "$workflow_name" "$elapsed"
    printf '%s\n' "$out" | tail -n 6 | sed 's/^/        /'
  fi
  return 0
}

# `.1`/`.1b` already prove the audit phase 32/32; this leaf owns the phase after it.
for a in $(grep -oE '^  audit_[a-z0-9_]+' "$GATE" | tr -d ' '); do
  eval "$a() { :; }"
done

# ⭐ OPTIONAL PREPARATION, run INSIDE the export dir before the replays.
#
# The unprepared census answers *"can the workflow phase run at all?"* — measured 3 PASS / 8 FAIL,
# with 6 of the 8 dying on the SAME rustc error before their gate logic executes. That first wall
# hides everything behind it: an 8-way identical failure tells you nothing about whether the gates
# themselves would pass. Set PGEN_CENSUS_PREPARE_CMD to knock the wall down and measure LAYER 2.
#
#   PGEN_CENSUS_PREPARE_CMD='make -C rust SHELL=/bin/bash annotation_parsers' bash <this>
#
# ⛔ The preparation is RECORDED in the census header, never implicit — a census run with a
# different starting tree is a different measurement, and one that does not say so is the
# vacuity class this tree exists to remove.
CENSUS_PREPARE="${PGEN_CENSUS_PREPARE_CMD:-}"

printf '%s\n' "=============================================================================="
printf 'PER-WORKFLOW CENSUS of %s\n' "$GATE"
printf 'filter:  %s\n' "${PGEN_CI_WORKFLOW_LOCAL_FILTER:-<all>}"
printf 'prepare: %s\n' "${CENSUS_PREPARE:-<none — export dir exactly as git ls-files leaves it>}"
printf '%s\n' "=============================================================================="

if [ -n "$CENSUS_PREPARE" ]; then
  copy_tracked_worktree
  printf 'prepare: running in %s\n' "$EXPORT_DIR"
  prep_started=$SECONDS
  # ⚠️ Bounded to the tail: the cold-clone sequence runs the generator with `--debug --trace`
  # (rust/Makefile:93-94) and captured IN FULL it measured 7.1 GB for one preparation. `make`
  # stops at the failing step, so the tail is where a failure's evidence is.
  if ( cd "$EXPORT_DIR"; export CARGO_NET_OFFLINE="$CARGO_OFFLINE_RAW"; eval "$CENSUS_PREPARE" ) 2>&1 \
       | tail -c 4194304 >"$LOG_DIR/00-prepare.log"; then
    printf 'prepare: OK (%ds, %s)\n' "$((SECONDS - prep_started))" "$LOG_DIR/00-prepare.log"
  else
    printf 'prepare: FAILED (%ds) — the census below measures an UNPREPARED tree\n' \
      "$((SECONDS - prep_started))"
    tail -n 8 "$LOG_DIR/00-prepare.log" | sed 's/^/        /'
  fi
  # `main` re-exports on top; harmless (cp -a overwrites) and it keeps the gate's own sequence
  # intact, but it would ERASE the preparation. Neutralise the second export instead of
  # re-implementing main().
  copy_tracked_worktree() { note "exporting tracked working tree into $EXPORT_DIR (already done — preserving preparation)"; }
fi

main

printf '%s\n' "------------------------------------------------------------------------------"
printf '%s\n' "${CENSUS_ROWS[@]}"
printf '%s\n' "------------------------------------------------------------------------------"
printf 'workflows=%d  PASS=%d  FAIL=%d  SKIP=%d\n' \
  "$((CENSUS_PASS + CENSUS_FAIL + CENSUS_SKIP))" "$CENSUS_PASS" "$CENSUS_FAIL" "$CENSUS_SKIP"
printf 'logs: %s\n' "$LOG_DIR"
exit 0
