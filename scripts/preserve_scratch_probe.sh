#!/usr/bin/env bash
# preserve_scratch_probe.sh — snapshot the `scratch` slot's CURRENT grammar into a tracked task
# artifact, so a synthetic a leaf claims to have reproduced on survives the restore.
#
# ⛔ WHY THIS EXISTS (GENERATED-LINT-CORRECTNESS.13, 2026-08-12).
# `grammars/scratch/scratch.ebnf` is a blessed THROWAWAY slot (TOOLBOX.md 1.3): you overwrite it
# with a synthetic, drive the toolbox, then `git checkout` it back. The restore is total — the probe
# grammar leaves NO trace in git, by construction. `PGEN-ENGINE-UNIVERSAL-SERVICES-0005` proved a
# defect on an 11-rule synthetic and restored the slot; the grammar survived only as a sentence, and
# session #218 spent longer rebuilding it from that sentence than mechanism 2's fix took.
#
# ⛔ AND WHY THIS IS A TOOL AND NOT A GATE — refused on measurement, not on taste.
# A gate would have to detect "an author built and destroyed a probe", and that act leaves no
# artifact to key on: a correct workflow restores the slot, so the commit shows NO diff there.
# The only remaining signal is PROSE, and it was priced over the whole corpus: a trigger on
# scratch-slot phrasing inside ticked acceptance boxes fires on 24 boxes, of which 22 would fail —
# but most of those are legitimate NON-obligations (`PARSE-HARNESS.2` is the leaf that CREATED the
# slot and its "probe grammar" IS the tracked fixture; `CI-PARITY-GATE-ROT.24` only names
# `focus_scratch` to demonstrate a build-flow trap and never had a probe grammar at all). A gate
# that is 91 % false positives teaches authors to waive it, which is the exact failure
# `GENERATED-LINT-CORRECTNESS.6`/`.12` document. Two chartered additions to the acceptance checker
# were already REFUSED on pricing (`.4` at 2/304, `.7` at 0/307); this is the third.
#
# ⇒ the real defect was FRICTION, not a missing rule: preserving meant inventing a path, creating a
# directory and remembering to `git add`, while the instruction at the moment of loss (TOOLBOX 1.3)
# said only "restore the default fixture". This script makes preserving one command, and
# `make focus_scratch` now prints it at the exact moment the author is holding the probe.
#
# USAGE
#   scripts/preserve_scratch_probe.sh <TREE-ID> <probe-name> [--note "one line"]
#   scripts/preserve_scratch_probe.sh --self-test
#
#   TREE-ID     the owning task tree, e.g. ENGINE-UNIVERSAL-SERVICES (case-insensitive; the
#               artifact directory is derived from it, matching docs/tasks/artifacts/ convention)
#   probe-name  a short slug describing what the probe isolates, e.g. seed_shadowing
#
# Writes  docs/tasks/artifacts/<tree_snake>/<probe-name>.ebnf  and `git add`s it, then tells you the
# path to cite in the leaf. Refuses rather than guesses: an unmodified slot, an empty slot, a name
# that is not a safe slug, or an existing artifact with different content all stop with an
# actionable message instead of silently writing something wrong.
set -euo pipefail

repo_root() { git rev-parse --show-toplevel 2>/dev/null || { echo "not a git repository" >&2; exit 2; }; }

ROOT="$(repo_root)"
SLOT_REL="grammars/scratch/scratch.ebnf"

die() { printf '❌ preserve-scratch-probe: %s\n' "$1" >&2; exit 1; }

# slug_of_tree <TREE-ID> — ENGINE-UNIVERSAL-SERVICES -> engine_universal_services
slug_of_tree() { printf '%s' "$1" | tr '[:upper:]' '[:lower:]' | tr '-' '_'; }

# The slot is UNMODIFIED when it is byte-identical to its committed state. That is the one
# condition under which there is provably nothing to preserve — and it is also the state a
# confused caller is most likely to be in (they already restored), so it gets its own message.
slot_is_unmodified() {
  git -C "$ROOT" diff --quiet -- "$SLOT_REL" 2>/dev/null
}

preserve() {
  local tree="$1" name="$2" note="${3:-}"
  [ -n "$tree" ] || die "missing <TREE-ID>. Usage: scripts/preserve_scratch_probe.sh <TREE-ID> <probe-name>"
  [ -n "$name" ] || die "missing <probe-name>. Usage: scripts/preserve_scratch_probe.sh <TREE-ID> <probe-name>"
  [[ "$name" =~ ^[a-z0-9][a-z0-9_]*$ ]] || die "probe-name '$name' must be a lowercase slug ([a-z0-9_], not starting with '_')."
  [ -f "$ROOT/$SLOT_REL" ] || die "$SLOT_REL does not exist — is this the PGEN repository?"
  [ -s "$ROOT/$SLOT_REL" ] || die "$SLOT_REL is empty — there is no probe grammar to preserve."

  if slot_is_unmodified; then
    die "$SLOT_REL is byte-identical to its committed state, so it holds the DEFAULT FIXTURE, not a probe.
   Preserve BEFORE you restore. If you already ran 'git checkout $SLOT_REL', the probe is gone —
   recover it from your shell history or rebuild it, then re-run this."
  fi

  # ⛔ PARSE-HARNESS.11 — refuse to preserve a slot whose operating manual has been destroyed.
  # This script's own instruction to the next reader is `cp <artifact> <slot>`, so an artifact
  # captured from a header-less slot RE-CREATES the loss every time somebody follows it.
  if ! bash "$ROOT/scripts/check_scratch_slot_header.sh" --probe-time; then
    die "$SLOT_REL has lost its header block, so preserving it would bake the loss into a tracked
   artifact — this script tells the next reader to \`cp\` it straight back over the slot.
   Repair first:  bash scripts/check_scratch_slot_header.sh --restore-header"
  fi

  local dir_rel="docs/tasks/artifacts/$(slug_of_tree "$tree")"
  local out_rel="$dir_rel/$name.ebnf"
  mkdir -p "$ROOT/$dir_rel"

  if [ -f "$ROOT/$out_rel" ] && ! cmp -s "$ROOT/$SLOT_REL" "$ROOT/$out_rel"; then
    die "$out_rel already exists with DIFFERENT content. Pick another <probe-name> rather than
   overwriting another leaf's evidence."
  fi

  {
    printf '# PRESERVED SCRATCH PROBE — %s / %s\n' "$tree" "$name"
    printf '# Snapshot of the `scratch` slot (TOOLBOX.md 1.3) taken before restoring the default\n'
    printf '# fixture, so the synthetic this leaf reproduces on can be re-run by the next reader:\n'
    printf '#   cp %s %s\n' "$out_rel" "$SLOT_REL"
    printf '#   make -C rust SHELL=/bin/bash focus_scratch\n'
    printf '#   (cd rust && cargo build --features "generated_parsers ebnf_dual_run" --bin ast_pipeline)\n'
    printf '# ⛔ Rebuild ast_pipeline AFTER focus_scratch: the focus target builds the binary BEFORE it\n'
    printf '#    regenerates the parser, so a binary built by that run judges the PREVIOUS grammar.\n'
    [ -n "$note" ] && printf '# NOTE: %s\n' "$note"
    printf '\n'
    cat "$ROOT/$SLOT_REL"
  } > "$ROOT/$out_rel"

  git -C "$ROOT" add -- "$out_rel"

  printf '✅ preserved: %s\n' "$out_rel"
  printf '   staged with git add. Cite that path in the leaf'"'"'s REPRODUCE box, then restore the slot:\n'
  printf '     git checkout %s\n' "$SLOT_REL"
}

# --self-test — prove the four refusals actually fire, on a throwaway clone of the slot state.
# Ground truth over trust: a guard nobody has seen refuse is indistinguishable from one that
# cannot (the TOOLBOX standard applied to this script itself).
SELFTEST_TMP=""
# ⛔ NOT a `local` — an EXIT trap fires AFTER the function returns, so a local would be out of scope
# by then and `set -u` would abort the script with "tmp: unbound variable" *after* reporting 4/4
# passed. Measured: the first cut did exactly that and exited 1 on an all-green run.
cleanup_selftest() { [ -n "$SELFTEST_TMP" ] && rm -rf "$SELFTEST_TMP"; return 0; }

self_test() {
  local rc pass=0 fail=0 tmp
  mkdir -p "$ROOT/rust/target"
  SELFTEST_TMP="$(mktemp -d "$ROOT/rust/target/preserve_scratch_probe_selftest.XXXXXX")"
  tmp="$SELFTEST_TMP"
  trap cleanup_selftest EXIT

  check() { # check <label> <expect-rc> <cmd...>
    local label="$1" want="$2"; shift 2
    set +e; "$@" >"$tmp/out.txt" 2>&1; rc=$?; set -e
    if [ "$rc" -eq "$want" ]; then printf '  ✅ %s (rc=%d)\n' "$label" "$rc"; pass=$((pass+1))
    else printf '  ❌ %s: expected rc=%d, got %d\n     %s\n' "$label" "$want" "$rc" "$(head -2 "$tmp/out.txt")"; fail=$((fail+1)); fi
  }

  printf 'preserve-scratch-probe --self-test (4 controls)\n'
  check "missing <probe-name> refuses"          1 "$0" SOME-TREE
  check "unsafe probe-name refuses"             1 "$0" SOME-TREE "Bad-Name"
  # An unmodified slot must refuse — this is the control that matters, because it is the state a
  # caller who already restored is in, and silently writing the default fixture would be worse
  # than failing (it would look like preserved evidence and prove nothing).
  if slot_is_unmodified; then
    check "unmodified slot refuses (nothing to preserve)" 1 "$0" SOME-TREE probe_selftest
  else
    printf '  ⚠️  unmodified-slot control SKIPPED — the slot is currently dirty.\n'
    printf '      Re-run --self-test on a clean slot to exercise it.\n'
  fi
  check "slug derivation is stable"             0 bash -c "[ \"\$($0 --print-slug ENGINE-UNIVERSAL-SERVICES)\" = engine_universal_services ]"

  printf '%d passed, %d failed\n' "$pass" "$fail"
  [ "$fail" -eq 0 ]
}

case "${1:-}" in
  --self-test)  self_test; exit $? ;;
  --print-slug) slug_of_tree "${2:?--print-slug needs a TREE-ID}"; exit 0 ;;
  -h|--help|"") sed -n '2,44p' "$0"; exit 0 ;;
esac

TREE="$1"; shift
NAME="${1:-}"; [ $# -gt 0 ] && shift
NOTE=""
if [ "${1:-}" = "--note" ]; then NOTE="${2:-}"; fi
preserve "$TREE" "$NAME" "$NOTE"
