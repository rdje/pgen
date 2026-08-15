#!/usr/bin/env bash
# scripts/check_scratch_slot_header.sh
#
# DOCTRINE `SCRATCH-SLOT-HEADER` (structural) — PARSE-HARNESS.11.
#
#   The blessed throwaway slot's OPERATING MANUAL is not part of the throwaway.
#
# ⛔ THE DEFECT THIS CLOSES — MEASURED, NOT HYPOTHETICAL (2026-08-15, session #236).
# `grammars/scratch/scratch.ebnf` is a tracked file whose BODY is meant to be overwritten freely
# (TOOLBOX.md 1.3) and whose 32-line HEADER is the only place that records how the slot works: the
# regeneration command, the ⛔ rebuild-ast_pipeline-AFTER ordering trap, that `generated/scratch*`
# is git-ignored, that THIS file is tracked, and how to restore it. An agent loading a probe
# grammar replaced the whole file — header included — and the loss was invisible: the correct
# workflow ends in `git checkout grammars/scratch/scratch.ebnf`, so the commit shows NO diff there
# and no commit-time gate can ever see it.
#
# ⛔⛔ AND WHY THE FIX IS NOT ANOTHER SENTENCE IN THE HEADER. The header ALREADY said *"Edit the
# grammar body below"*, in its own HOW TO USE section, and it was overwritten anyway. Adding
# *"do not delete this header"* would be a second suggestion in a file whose first suggestion had
# just been ignored — and `DOCTRINE_ENFORCEMENT.md` §1 is that a rule nothing checks is a
# suggestion. So the remedy is a check, fired at the layer where the loss actually happens.
#
# ⭐⭐ TWO TIERS, BECAUSE THE TWO FAILURE MODES ARE VISIBLE AT DIFFERENT MOMENTS.
#
#   STRUCTURAL (default; the doctrine, every commit) — the header block exists and still carries
#     the four operational anchors that make it a manual rather than a comment. This catches a
#     header removal that reaches a commit. It deliberately does NOT compare against HEAD, so a
#     deliberate rewording of the header is free.
#
#   --probe-time (called from `make focus_scratch` and from preserve_scratch_probe.sh) — ALSO
#     requires the header to be byte-identical to the committed one. This is the tier that catches
#     the real incident, because it fires while the probe grammar is still in the slot: at that
#     moment the body is supposed to differ and the header is not. ⭐ Derived from
#     `git show HEAD:` rather than from a second copy of the header, so there is nothing to drift.
#     A deliberate header edit sets PGEN_SCRATCH_HEADER_EDIT=1 to pass this tier.
#
# ⭐ `--restore-header` turns the refusal into a one-command repair: re-attach the committed header
# to whatever body is currently in the slot. A guard that only says NO invites a hand
# reconstruction, and a hand-reconstructed manual is how a manual quietly loses a line.
#
# CONTRACT (DOCTRINE_ENFORCEMENT.md §4): exit code is the verdict (0 holds / 1 breach / 2 refuses);
# explains on stderr; deterministic; mutates nothing except under the explicit --restore-header;
# resolves the repo root from its own location.
#
# Usage:
#   bash scripts/check_scratch_slot_header.sh                  # structural (the doctrine)
#   bash scripts/check_scratch_slot_header.sh --probe-time     # + byte-identical to HEAD
#   bash scripts/check_scratch_slot_header.sh --restore-header # re-attach the committed header
#   bash scripts/check_scratch_slot_header.sh --self-test      # prove every refusal fires
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$ROOT" || exit 2

SLOT="grammars/scratch/scratch.ebnf"

# ⛔ The anchors are COMMANDS and PATHS, not sentences. A prose-phrase list would have to enumerate
# wordings, and enumeration is precisely what made `LIVE-DOC-CURRENCY`'s instrument B measure the
# same population as 10, then 16, then 18. These four are the operational content: without any one
# of them the next reader cannot regenerate, cannot avoid the ordering trap, does not know the
# artifacts are untracked, and cannot restore the slot.
ANCHORS=(
  "focus_scratch"                          # how to regenerate
  "parseability_probe --parse scratch"     # how to drive it
  "git-ignored"                            # that generated/scratch* never enters the tracked set
  "git checkout grammars/scratch/scratch.ebnf"  # how to restore
)

note() { printf 'scratch-slot-header: %s\n' "$1" >&2; }

# header_of <file> — the leading banner-delimited comment block: line 1 must be a `# ===` banner,
# and the block ends at the next one. Prints nothing (rc 1) when that shape is absent.
header_of() {
  awk 'NR==1 { if ($0 !~ /^# =+$/) exit 1 } { print } NR>1 && /^# =+$/ { found=1; exit 0 }
       END { if (!found) exit 1 }' "$1"
}

structural() {
  local rc=0 hdr
  if [ ! -f "$SLOT" ]; then note "$SLOT does not exist"; return 1; fi
  if [ ! -s "$SLOT" ]; then note "$SLOT is empty"; return 1; fi
  if ! hdr="$(header_of "$SLOT")"; then
    note "$SLOT has no banner-delimited header block."
    note "  The slot's operating manual is gone — it is the only record of how to regenerate,"
    note "  restore and drive the slot. Repair it with:"
    note "    bash scripts/check_scratch_slot_header.sh --restore-header"
    return 1
  fi
  local a
  for a in "${ANCHORS[@]}"; do
    case "$hdr" in
      *"$a"*) ;;
      *) note "the header no longer names \`$a\` — it has stopped being an operating manual"; rc=1 ;;
    esac
  done
  return $rc
}

probe_time() {
  # ⭐ Compared against the COMMITTED header, derived — never against a copy kept in this script.
  local live committed
  if ! live="$(header_of "$SLOT" 2>/dev/null)"; then return 1; fi   # structural() already reported
  if ! committed="$(git show "HEAD:$SLOT" 2>/dev/null | header_of /dev/stdin)"; then
    note "could not read the committed header from HEAD:$SLOT — skipping the byte-identity tier"
    return 0
  fi
  [ "$live" = "$committed" ] && return 0
  note "the slot's header DIFFERS from the committed one while a probe body is loaded."
  note "  The header is not part of the throwaway: overwrite the BODY, keep the manual."
  note "  Repair (re-attaches the committed header to your current body):"
  note "    bash scripts/check_scratch_slot_header.sh --restore-header"
  note "  Deliberately editing the header? Re-run with PGEN_SCRATCH_HEADER_EDIT=1."
  return 1
}

restore_header() {
  local committed body tmp
  committed="$(git show "HEAD:$SLOT" 2>/dev/null | header_of /dev/stdin)" || {
    note "HEAD:$SLOT has no header block to restore from"; return 2; }
  # the body is everything AFTER the live header block, or the whole file when it has none
  if header_of "$SLOT" >/dev/null 2>&1; then
    body="$(awk 'skip { print; next } NR==1 { next } /^# =+$/ { skip=1 }' "$SLOT")"
  else
    body="$(cat "$SLOT")"
  fi
  tmp="$(mktemp "${TMPDIR:-/tmp}/scratch_hdr.XXXXXX")" || return 2
  { printf '%s\n' "$committed"; printf '%s\n' "$body"; } > "$tmp"
  mv "$tmp" "$SLOT" || return 2
  note "restored the committed header; your probe body is preserved below it."
  return 0
}

self_test() {
  # GROUND TRUTH over trust: a guard nobody has seen refuse is indistinguishable from one that
  # cannot refuse. Every arm runs against a throwaway copy, never the real slot.
  local tmpdir pass=0 fail=0 saved
  tmpdir="$(mktemp -d "$ROOT/rust/target/scratch_header_selftest.XXXXXX")" || return 2
  saved="$tmpdir/slot.orig"; cp "$SLOT" "$saved"
  arm() { # arm <label> <expect-rc> ; slot content already in place
    local label="$1" want="$2" rc
    structural; rc=$?
    if [ "$rc" -eq "$want" ]; then printf '  ✅ %s (rc %d)\n' "$label" "$rc"; pass=$((pass+1))
    else printf '  ❌ %s: rc %d, wanted %d\n' "$label" "$rc" "$want"; fail=$((fail+1)); fi
  }
  printf 'scratch-slot-header --self-test:\n'
  arm "GREEN: the real slot passes" 0
  # the incident, reproduced exactly: header replaced by a probe body
  printf '@entry: true\nscratch := "x"\n' > "$SLOT";              arm "RED: header deleted outright" 1
  # a header that is present but has stopped being a manual
  { printf '# ==========\n# a header\n# ==========\n'; printf '@entry: true\nscratch := "x"\n'; } > "$SLOT"
  arm "RED: banner present, all four anchors gone" 1
  # one anchor missing is still a breach — the manual is only as good as its weakest instruction
  { header_of "$saved" | grep -v 'git checkout grammars/scratch/scratch.ebnf'; tail -n +2 "$saved"; } > "$SLOT"
  arm "RED: exactly one anchor removed" 1
  : > "$SLOT";                                                     arm "RED: empty slot" 1
  # --restore-header must repair the incident, and the repaired slot must then pass
  printf '@entry: true\nscratch := "probe body"\n' > "$SLOT"
  restore_header >/dev/null 2>&1
  arm "GREEN after --restore-header on a header-less slot" 0
  if grep -q 'scratch := "probe body"' "$SLOT"; then
    printf '  ✅ --restore-header preserved the probe body\n'; pass=$((pass+1))
  else printf '  ❌ --restore-header LOST the probe body\n'; fail=$((fail+1)); fi
  cp "$saved" "$SLOT"; rm -rf "$tmpdir"
  arm "GREEN: the real slot is restored intact" 0
  printf 'scratch-slot-header: self-test %d passed, %d failed\n' "$pass" "$fail"
  [ "$fail" -eq 0 ]
}

case "${1:-}" in
  --self-test)      self_test; exit $? ;;
  --restore-header) restore_header; exit $? ;;
  --probe-time)
    structural || exit 1
    [ "${PGEN_SCRATCH_HEADER_EDIT:-0}" = "1" ] && {
      note "OK (header edit explicitly declared via PGEN_SCRATCH_HEADER_EDIT=1)"; exit 0; }
    probe_time || exit 1
    exit 0 ;;
  "") structural || exit 1
      echo "scratch-slot-header: OK ($SLOT keeps its operating manual: ${#ANCHORS[@]} anchors)"
      exit 0 ;;
  *)  note "unknown argument: $1"; exit 2 ;;
esac
