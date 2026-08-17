#!/usr/bin/env bash
# scripts/check_generated_reproducibility.sh — doctrine GENERATED-REPRODUCIBILITY.
#
# ⛔ THE RULE: every artifact in the untracked `generated/` tree must be what HEAD's tracked source
#    produces. Not "compiles". Not "hasn't changed since someone recorded a hash". What the source
#    produces, today.
#
# WHY IT EXISTS — MEASURED, NOT IMAGINED (`ENGINE-UNIVERSAL-SERVICES.29`, 2026-08-16). Both
# annotation parsers in `generated/` carried a line the tracked code generator CANNOT emit:
#
#     self.coverage_deltas.clear();
#
# `grep -c` over `rust/src/ast_pipeline/ast_based_generator.rs` -> 0; `git log -S` -> no commit
# ever; and the emitter's own comment at that site defends the omission deliberately. They had been
# written from an uncommitted editor state ~80 minutes before the commit that finalised the emission.
# ⛔ Those two artifacts are the pair `rust/src/lib.rs` includes by LITERAL PATH and the annotation
# backend links — the pair that participates in generating EVERY other parser. The project's layer-A
# resume pointer recorded *"`generated/` FRESH"* throughout, and nothing disagreed.
#
# ⛔⛔ WHY NO EXISTING CHECK COULD SEE IT, each one measured rather than assumed:
#   - a recorded HASH proves only that an artifact has not moved since someone recorded the hash;
#   - `fixed_point_gate` proves REGENERATION CONVERGES across its own cycles — it OVERWRITES whatever
#     was on disk in cycle 1, so a stale pre-existing artifact is invisible to it by construction;
#   - `PARSE-COST-RATCHET` tier 1 re-hashes the SV parser, which detects MOVEMENT, never STALENESS;
#   - a green build proves the artifact COMPILES, not that it is current.
# Re-derive and diff is the only thing that answers the question. That is tier 2.
#
# TWO TIERS, and the cheap one is a PROOF rather than a shortcut — the same architecture
# `PARSE-COST-RATCHET` uses, for the same reason:
#
#   TIER 1  IDENTITY, every commit, <1 s, no build.
#           Re-hash the three things each artifact is a function of — the artifact itself, its input
#           JSON, and a digest over the tracked EMISSION SOURCES + the recipe that invokes them.
#           If none moved, the last tier-2 verification still describes this tree and the artifacts
#           CANNOT have become stale. If any moved, the baseline no longer describes the tree and
#           the gate demands a re-verify instead of guessing which way.
#
#   TIER 2  THE ORACLE, on demand (`make -C rust SHELL=/bin/bash generated_reproducibility_gate`).
#           Re-derive every artifact through the tracked recipe and demand byte-identity, then
#           rewrite the baseline. `--verify` runs it here; `--rebaseline` records the result.
#
# ⛔⛔ TIER 2 MUST PROVE ITS GENERATOR IS CURRENT, AND UNTIL `ENGINE-UNIVERSAL-SERVICES.32` IT DID
# NOT — SO IT COULD PASS BY CONSTRUCTION. The annotation PAIR has always been re-derived by an
# `ast_pipeline_bootstrap` built from HEAD in this gate's own scratch dir. The EIGHT FAMILIES were
# re-derived by whatever `rust/target/debug/ast_pipeline` happened to be on disk, guarded only for
# its FEATURE surface and its PRESENCE — never its CURRENCY. A stale generator therefore produced
# BOTH sides of the comparison, and byte-identity between two outputs of one stale tool is
# guaranteed. Measured, not argued: with the pair correct, the eight families left at a previous
# emission and a matching stale binary in place, this script printed
#
#     ✓ systemverilog   re-derives byte-identically (34738 sites)
#     generated-reproducibility: TIER 2 OK — every checked artifact is what HEAD produces
#
# at exit 0, over eight artifacts that HEAD's emitter does not produce (it emits 1 embedded site,
# not 34 738). ⭐ The run's own output contained the disproof of its headline: the two PAIR rows
# read `1 sites` because their generator came from HEAD, and the eight family rows did not.
#
# ⭐⭐ THE FIX IS TO ASK CARGO, AND THE REASON IT IS CARGO IS THE INTERESTING PART. Three candidates
# were priced (`ENGINE-UNIVERSAL-SERVICES.32`):
#   (A) build `ast_pipeline` into THIS gate's own CARGO_TARGET_DIR, as the pair already does —
#       sound, but a COLD ~216 MB build every run. Rejected on price.
#   (B) have `build.rs` publish the `emission_sha` into the binary, the shape `.24` used for the
#       parser fingerprint — ⛔ REJECTED ON DESIGN, not price: the gate derives `emission_sha` from
#       `git ls-files`, and a `build.rs` cannot, so this needs a SECOND implementation of one digest
#       that must agree with the first. Two things that must agree can drift, and this repository
#       has paid for that four times (the `2.741` classifier, the carried `43 615`, the four stale
#       prose copies of one number, the `128`-vs-`127` denominator).
#   (C) ⭐ INVOKE CARGO on the tree's own target dir. Cargo is the authority on *"is this binary
#       current with these sources"* — that is its entire job — so there is no second implementation
#       to drift, no digest to keep in lockstep, and no false positive when a file is touched but
#       unchanged (which a mtime comparison would report as staleness). Measured cost when the
#       binary is already current: **0.8 s**. When it is not, it does the only correct thing and
#       rebuilds. ADOPTED.
# ⚠️ HONEST BOUNDS of (C). ⛔⛔ ONE OF THESE WAS PUBLISHED WRONG AND IS RETRACTED HERE
# (`ENGINE-UNIVERSAL-SERVICES.32` slice 2, under director challenge). The first draft of this block
# claimed *"it cannot detect a binary hand-COPIED over cargo's output path, because cargo keys on
# its own fingerprint of the sources rather than on the output bytes."* That was reasoned, not
# measured, and it is FALSE: `target/debug/ast_pipeline` is a hardlink/copy of the real artifact in
# `target/debug/deps/`, and cargo re-establishes it on every invocation. Measured twice, two
# different perturbations — replacing it with a DIFFERENT valid binary, and TRUNCATING it to 1 000
# bytes — cargo restored it to byte-identity in **0.57 s** without recompiling.
# ⭐ The consequence is the one that matters: the fix therefore DOES close the constructed
# demonstration (M1) that opened this leaf. Re-run end to end with the json artifact left at the
# previous emission and its un-hoisted generator hand-placed over cargo's output, the gate no longer
# passes — cargo repairs the binary, the fresh re-derivation comes out with 1 embedded site against
# the artifact's 208, and the site-count assertion REFUSES with exit 2 and an actionable message
# where the pre-fix code printed `✓ json re-derives byte-identically (208 sites)`.
# ⚠️ Publishing an unmeasured bound is the same defect as publishing an unmeasured number, and the
# direction here was CONSERVATIVE — it understated the fix — which is precisely why nothing would
# have caught it: an author re-reading it would find nothing to disagree with.
#
# The bounds that DO survive, each stated as what it is:
#   - **Working tree, not `git show HEAD:`** — it proves the binary is current with the tracked
#     files as they stand, which is the same notion of "HEAD" tier 1's `emission_sha` already uses.
#     Uncommitted emitter edits are therefore inside the guarantee, not outside it.
#   - ⛔ **Cargo sees the CRATE, not the RECIPE.** `rust/Makefile` is in `emission_sha` but is not a
#     cargo input, and this script MIRRORS the Makefile's generator flags
#     (`--generate-parser --eliminate-left-recursion`, `+ --bootstrap-mode` for the pair) in
#     `rederive_and_compare` rather than reading them from it. Two implementations of one recipe
#     that must agree — the same class this leaf REJECTED candidate (B) for — sitting inside the
#     check it hardened. Routed as `ENGINE-UNIVERSAL-SERVICES.33`, not fixed here.
#   - ⚠️ It MUTATES `rust/target/`: announced on every run, and the reason this lives in tier 2 (on
#     demand) and never in tier 1.
# Neither surviving bound is the mechanism that produced the defect — that was `make` skipping a
# rebuild on GNU Make 3.81's whole-second mtime comparison (`CI-PARITY-GATE-ROT.37`), which cargo
# detects exactly.
#
# ⭐ HONEST BOUND, stated before the check is trusted rather than after: tier 1 proves *"nothing that
# could have changed the artifacts has changed"*, NOT *"the artifacts are correct"*. It inherits
# whatever tier 2 last established. That is a real limitation and it is the same one
# `PARSE-COST-RATCHET` states about itself; it is sound because the identity set is chosen to be
# OVER-inclusive (see EMISSION SOURCES below), so its failure direction is a spurious re-verify,
# never a silent pass.
#
# ⭐ THE EMISSION SOURCE SET IS DERIVED, NEVER HAND-LISTED — `git ls-files rust/src/ast_pipeline`
# plus `rust/Makefile` (the recipe is part of what determines the artifact: it chooses the flags and
# the `-o` spelling). A file added to the code generator tomorrow joins the identity by construction.
# ⛔ Over-inclusive on purpose: a change to a code-generator file that happens not to affect emission
# costs one re-verify. Under-inclusive would cost silent staleness, which is the defect this exists
# to prevent.
#
# ⛔ `generated/` IS NOT TRACKED, so a fresh clone has none of it. That is reported as NOT EVALUATED
# — loudly, never as a pass and never as a failure
# (`docs/decisions/feedback_a_check_that_cannot_run_must_say_so.md`).
#
# USAGE
#   bash scripts/check_generated_reproducibility.sh              # tier 1 (the doctrine)
#   bash scripts/check_generated_reproducibility.sh --verify     # tier 2, re-derive + compare
#   bash scripts/check_generated_reproducibility.sh --rebaseline # tier 2, then record the result
#   bash scripts/check_generated_reproducibility.sh --self-test  # fire every refusal arm
# EXIT
#   0 = the doctrine holds (or NOT EVALUATED, reported loudly)
#   1 = a breach: the baseline no longer describes this tree, or an artifact does not re-derive
#   2 = the check could not run correctly (malformed baseline, unusable comparison) — never a pass

set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$ROOT"

BASELINE="rust/test_data/grammar_quality/generated_reproducibility_v0.json"
GENERATED="generated"
WORK="rust/target/generated_reproducibility"

# The two annotation parsers come from `ast_pipeline_bootstrap` in --bootstrap-mode; the eight
# families from `ast_pipeline`. Both rosters are the Makefile's, mirrored here and CHECKED against
# it by tier 2 so the mirror cannot drift silently.
PAIR=(return_annotation semantic_annotation)
FAMILIES=(json regex systemverilog systemverilog_preprocessor vhdl rtl_const_expr rtl_frontend scratch)

fail=0
# ⛔⛔ SKIPPED COHORTS MUST REACH THE HEADLINE (`ENGINE-UNIVERSAL-SERVICES.32`). A NOT-EVALUATED
# cohort returns 0, and the caller used to print *"TIER 2 OK — every checked artifact is what HEAD
# produces"* regardless — so a run that checked 2 of 10 artifacts announced itself in the same words
# as a run that checked all 10. Found by this leaf's own RED arm on its first execution, against the
# very check being hardened. The headline now NAMES what it skipped, and `--rebaseline` REFUSES on a
# partial run rather than recording a baseline half of which nothing verified.
skipped=""
note()  { printf 'generated-reproducibility: %s\n' "$1" >&2; }
die()   { note "$1"; exit 2; }
breach(){ note "$1"; fail=1; }

sha_file() { shasum -a 256 "$1" 2>/dev/null | cut -d' ' -f1; }

# emission_sha — the digest tier 1 keys on. DERIVED from git, so it cannot rot into a hand-list.
emission_sha() {
  local files
  files=$(git ls-files 'rust/src/ast_pipeline/*.rs' 'rust/src/ast_pipeline/**/*.rs' rust/Makefile 2>/dev/null | LC_ALL=C sort)
  [ -n "$files" ] || die "the emission source set is EMPTY — git ls-files matched nothing, so the identity would be vacuous"
  # Hash the CONTENT of each file with its path, so a rename is a change.
  { while IFS= read -r f; do printf '%s  %s\n' "$(sha_file "$f")" "$f"; done <<< "$files"; } \
    | shasum -a 256 | cut -d' ' -f1
}

# generated_present — `generated/` may be entirely absent (fresh clone) or partially built.
generated_present() { [ -d "$GENERATED" ] && ls "$GENERATED"/*_parser.rs >/dev/null 2>&1; }

# ── tier 2: re-derive one artifact into a mimic tree and compare ──────────────────────────────────
#
# ⭐ THE MIMIC TREE IS LOAD-BEARING, NOT TIDINESS. A generated parser embeds its own `-o` path
# (TOOLBOX 5.6 — once per artifact since `ENGINE-UNIVERSAL-SERVICES.31` slice 2, and 36 346 times in
# SystemVerilog before it), so re-deriving to a scratch filename changes the artifact's SIZE for
# reasons that have nothing to do with the source. ⛔ The hoist shrank that effect by four orders of
# magnitude and did NOT remove it — one site is still one byte per character — and it made the
# failure HARDER to notice, which is why the assertion below stays exactly as strict as it was. The
# mimic `<work>/root/{generated,rust}` tree, entered from `<work>/root/rust`, makes the emitted
# string byte-identical to what `rust/Makefile` passes — and the site counts are ASSERTED, never
# assumed, because this exact trap has inverted three published readings in this repository.
rederive_and_compare() {
  local fam="$1" kind="$2" tool="$3"   # kind = pair|family
  local out="../generated/${fam}_parser.rs"
  local live="$ROOT/$GENERATED/${fam}_parser.rs"
  local json="$ROOT/$GENERATED/${fam}.json"
  local fresh="$WORK/root/generated/${fam}_parser.rs"

  [ -f "$live" ] || { breach "$GENERATED/${fam}_parser.rs is absent while other artifacts are present — regenerate with \`make -C rust SHELL=/bin/bash regenerate_generated_parsers\`"; return; }
  [ -f "$json" ] || { breach "$GENERATED/${fam}.json is absent, so ${fam} cannot be re-derived"; return; }

  local args=(--generate-parser --eliminate-left-recursion)
  [ "$kind" = pair ] && args=(--generate-parser --bootstrap-mode --eliminate-left-recursion)

  ( cd "$WORK/root/rust" && "$tool" "$json" "${args[@]}" -o "$out" ) >"$WORK/${fam}.log" 2>&1 \
    || { breach "codegen FAILED for ${fam} — see $WORK/${fam}.log"; return; }

  # ⭐ The spelling is DERIVED FROM EACH ARTIFACT by the shared helper
  # (`scripts/compare_generated_parsers.py`, `ENGINE-UNIVERSAL-SERVICES.25` (c)) — never taken from
  # `$out`. A caller-supplied spelling is the defect the helper exists to remove: the short spelling
  # is a SUBSTRING of the long one, so counting the short one over a long-spelling artifact returns
  # the full count and reads as agreement. Here both sides are written through `$out` today, so this
  # is a hardening rather than a repair; the helper also REFUSES (exit 2) on an artifact whose
  # embedded path is absent or ambiguous, instead of returning a number nobody can interpret.
  local live_sites fresh_sites live_spelling fresh_spelling
  live_sites=$(python3 "$ROOT/scripts/compare_generated_parsers.py" --sites "$live") \
    || die "cannot compare ${fam}: the shared helper REFUSED to derive an embedded -o path from $live"
  fresh_sites=$(python3 "$ROOT/scripts/compare_generated_parsers.py" --sites "$fresh") \
    || die "cannot compare ${fam}: the shared helper REFUSED to derive an embedded -o path from $fresh"
  live_spelling=$(python3 "$ROOT/scripts/compare_generated_parsers.py" --spelling "$live")
  fresh_spelling=$(python3 "$ROOT/scripts/compare_generated_parsers.py" --spelling "$fresh")
  if [ "$live_sites" = 0 ] || [ "$live_sites" != "$fresh_sites" ] || [ "$live_spelling" != "$fresh_spelling" ]; then
    die "cannot compare ${fam}: embedded -o sites live=$live_sites ('$live_spelling') fresh=$fresh_sites ('$fresh_spelling'). The two sides were not written through the same path spelling, so any verdict would measure the PATH (TOOLBOX 5.6), not the source."
  fi

  local a b
  a=$(sha_file "$live"); b=$(sha_file "$fresh")
  if [ "$a" = "$b" ]; then
    printf '  ✓ %-30s re-derives byte-identically (%s sites)\n' "$fam" "$live_sites"
    printf '%s %s\n' "$fam" "$a" >> "$WORK/verified.txt"
  else
    breach "$(printf '%-30s DOES NOT re-derive from HEAD: live %s… (%s B) vs fresh %s… (%s B)' \
      "$fam" "${a:0:12}" "$(wc -c < "$live" | tr -d ' ')" "${b:0:12}" "$(wc -c < "$fresh" | tr -d ' ')")"
    diff -u0 "$live" "$fresh" | sed -n '3,15p' | sed 's/^/        /' >&2
    printf '%s %s\n' "$fam" "MISMATCH" >> "$WORK/verified.txt"
  fi
  rm -f "$fresh"
}

run_tier2() {
  generated_present || { skipped="${skipped} every artifact"; note "NOT EVALUATED — $GENERATED/ holds no generated parser. Regenerate with \`make -C rust SHELL=/bin/bash regenerate_generated_parsers\`, then re-run."; return 0; }

  # The roster must match the Makefile's, or this gate silently checks a subset.
  local mk_fams
  mk_fams=$(sed -n 's/^GENERATED_PARSER_FAMILIES = //p' rust/Makefile)
  [ -n "$mk_fams" ] || die "could not read GENERATED_PARSER_FAMILIES from rust/Makefile — the roster mirror cannot be checked, so this gate would silently check a subset"
  local f
  for f in $mk_fams; do
    case " ${FAMILIES[*]} " in *" $f "*) ;; *) die "rust/Makefile builds family '$f' which this gate does not check — the roster has drifted; add it to FAMILIES";; esac
  done

  rm -rf "$WORK"; mkdir -p "$WORK/root/generated" "$WORK/root/rust"
  : > "$WORK/verified.txt"

  printf 'generated-reproducibility: TIER 2 — re-deriving every artifact from HEAD\n'

  # The annotation pair, through a bootstrap generator built from HEAD into its own target dir so
  # the tree's own binaries are never disturbed by a check.
  printf 'building ast_pipeline_bootstrap from HEAD…\n'
  ( cd rust && CARGO_TARGET_DIR="$ROOT/$WORK/boot" cargo build \
      --bin ast_pipeline_bootstrap --no-default-features --features bootstrap ) >"$WORK/boot.log" 2>&1 \
    || { skipped="${skipped} the annotation pair"; note "NOT EVALUATED — ast_pipeline_bootstrap does not build from HEAD (see $WORK/boot.log). That is a larger finding than this gate measures."; return 0; }
  for f in "${PAIR[@]}"; do rederive_and_compare "$f" pair "$ROOT/$WORK/boot/debug/ast_pipeline_bootstrap"; done

  # ⭐⭐ THE FAMILIES' GENERATOR MUST BE PROVEN CURRENT, NOT MERELY PRESENT AND WELL-FEATURED
  # (`ENGINE-UNIVERSAL-SERVICES.32`; see the header for the measured false pass this closes and for
  # why cargo rather than a second digest). Cargo is asked whether the binary is current with the
  # tree's sources; when it is, this costs ~0.8 s, and when it is not, rebuilding is the only
  # correct response. ⚠️ It MUTATES `rust/target/` and can therefore take minutes on a genuinely
  # stale tree — announced, because a check that silently spends five minutes teaches bypassing.
  local pipeline="rust/target/debug/ast_pipeline"
  printf 'ensuring ast_pipeline is CURRENT with the tree (cargo decides; ~0.8 s if it already is)…\n'
  if [ "${PGEN_GENREPRO_SELFTEST_FAIL_CURRENCY:-}" = 1 ] && [ "${PGEN_GENREPRO_SELFTEST:-}" = 1 ]; then
    # SELF-TEST ONLY, and gated on TWO variables so it cannot be reached from an ordinary run:
    # exercise the refusal path without spending a real rebuild. It proves the gate REFUSES when the
    # currency step fails — it does NOT re-prove that cargo detects staleness, which is cargo's own
    # contract and is covered by the one-shot end-to-end measurement recorded in `.32`.
    skipped="${skipped} the 8 family artifacts"; note "NOT EVALUATED for the ${#FAMILIES[@]} family artifacts — the generator currency step FAILED (self-test injection). Without it, a stale generator would produce BOTH sides of the comparison and this gate would pass by construction."
    return 0
  fi
  if ! ( cd rust && cargo build --features "generated_parsers ebnf_dual_run" --bin ast_pipeline ) \
        >"$WORK/pipeline_build.log" 2>&1; then
    skipped="${skipped} the 8 family artifacts"; note "NOT EVALUATED for the ${#FAMILIES[@]} family artifacts — ast_pipeline does not build from the current tree (see $WORK/pipeline_build.log). A gate that continued here would re-derive with a STALE generator and compare it against artifacts that same generator produced, which passes by construction."
    return 0
  fi
  if [ ! -x "$pipeline" ]; then
    skipped="${skipped} the 8 family artifacts"; note "NOT EVALUATED for the ${#FAMILIES[@]} family artifacts — $pipeline is absent even after a successful build. Build it with \`cd rust && cargo build --features \"generated_parsers ebnf_dual_run\" --bin ast_pipeline\`, then re-run."
    return 0
  fi
  # ⭐ The feature-surface arm is KEPT even though cargo now builds with the right features: it is
  # free, it has its own RED arm, and it is the only thing that would notice if the invocation above
  # ever drifted away from the feature set the families need.
  local surface; surface=$("$pipeline" --report-feature-surface 2>&1)
  case "$surface" in
    *"ebnf_dual_run=true generated_parsers=true"*) ;;
    *) skipped="${skipped} the 8 family artifacts"; note "NOT EVALUATED for the ${#FAMILIES[@]} family artifacts — $pipeline is UNDER-FEATURED ($surface). It cannot generate any parser at all, and a comparison loop that reads a missing file as 'no difference' would report a clean pass."; return 0 ;;
  esac
  for f in "${FAMILIES[@]}"; do rederive_and_compare "$f" family "$ROOT/$pipeline"; done
}

write_baseline() {
  local esha; esha=$(emission_sha)
  {
    printf '{\n'
    printf '  "_comment": "GENERATED-REPRODUCIBILITY baseline. DERIVED — never hand-edit. Rewrite with: bash scripts/check_generated_reproducibility.sh --rebaseline",\n'
    printf '  "_what_this_proves": "every artifact below re-derived BYTE-IDENTICALLY from HEAD at the recorded commit. Tier 1 re-hashes these three fields and demands a re-verify the moment any of them moves.",\n'
    printf '  "verified_at_commit": "%s",\n' "$(git rev-parse HEAD)"
    printf '  "emission_sha": "%s",\n' "$esha"
    printf '  "artifacts": {\n'
    local first=1 fam
    for fam in "${PAIR[@]}" "${FAMILIES[@]}"; do
      local p="$GENERATED/${fam}_parser.rs" j="$GENERATED/${fam}.json"
      [ -f "$p" ] && [ -f "$j" ] || continue
      [ "$first" = 1 ] || printf ',\n'; first=0
      printf '    "%s": { "parser_sha": "%s", "input_sha": "%s" }' "$fam" "$(sha_file "$p")" "$(sha_file "$j")"
    done
    printf '\n  }\n}\n'
  } > "$BASELINE"
  printf 'generated-reproducibility: baseline rewritten (%s)\n' "$BASELINE"
}

# ── argument dispatch ─────────────────────────────────────────────────────────────────────────────
case "${1:-}" in
  --verify)
    run_tier2
    if [ "$fail" = 0 ]; then
      if [ -n "$skipped" ]; then
        printf 'generated-reproducibility: TIER 2 PARTIAL — what ran is what HEAD produces, but NOT EVALUATED for:%s. This is NOT a clean verification.\n' "$skipped"
      else
        printf 'generated-reproducibility: TIER 2 OK — every checked artifact is what HEAD produces\n'
      fi
    fi
    exit "$fail" ;;
  --rebaseline)
    run_tier2
    [ "$fail" = 0 ] || { note "refusing to rebaseline: tier 2 found a breach. Regenerate the artifacts, do not record the divergence."; exit 1; }
    [ -z "$skipped" ] || { note "refusing to rebaseline: tier 2 was NOT EVALUATED for:$skipped. Recording a baseline whose rows nothing verified is exactly the 'proven by a hash somebody wrote down' failure this doctrine exists to prevent."; exit 1; }
    generated_present || die "refusing to rebaseline from an absent generated/ tree"
    write_baseline; exit 0 ;;
  --self-test) ;;   # handled below
  "") ;;
  *) die "unknown argument '$1' (expected --verify, --rebaseline, --self-test, or none)" ;;
esac

# ── TIER 1 — the doctrine ─────────────────────────────────────────────────────────────────────────
tier1() {
  [ -f "$BASELINE" ] || die "$BASELINE is missing — record it with \`bash scripts/check_generated_reproducibility.sh --rebaseline\`"

  if ! generated_present; then
    note "NOT EVALUATED — $GENERATED/ holds no generated parser (it is untracked, so a fresh clone has none). Regenerate with \`make -C rust SHELL=/bin/bash regenerate_generated_parsers\`."
    return 0
  fi

  local recorded_emission live_emission
  recorded_emission=$(sed -n 's/.*"emission_sha": "\([0-9a-f]*\)".*/\1/p' "$BASELINE")
  [ -n "$recorded_emission" ] || die "$BASELINE carries no emission_sha — it is malformed, and comparing against an empty string would PASS by construction"
  live_emission=$(emission_sha)

  if [ "$recorded_emission" != "$live_emission" ]; then
    # ⛔ SAY WHAT MOVED, and do not overstate it. The identity covers the code generator AND the
    # recipe that invokes it (flags, the `-o` spelling), and it is deliberately over-inclusive — so
    # a change here means "the artifacts MAY no longer be what this tree produces", not "they are
    # stale". The gate's job is to refuse to guess which; tier 2 is what decides.
    breach "the EMISSION SOURCES moved (recorded ${recorded_emission:0:12}…, live ${live_emission:0:12}…).
      The digest covers the tracked code generator ($(git ls-files 'rust/src/ast_pipeline/*.rs' 'rust/src/ast_pipeline/**/*.rs' | wc -l | tr -d ' ') files) AND rust/Makefile, which chooses the
      flags and the -o spelling. Something in that set changed, so what is in $GENERATED/ MAY no
      longer be what this tree produces — this tier cannot tell which, by design, and will not guess.
      Re-derive and re-record — do NOT edit the baseline:
        make -C rust SHELL=/bin/bash generated_reproducibility_rebaseline"
    return 0
  fi

  local checked=0 fam
  for fam in "${PAIR[@]}" "${FAMILIES[@]}"; do
    local rec_p rec_i
    rec_p=$(sed -n "s/.*\"$fam\": { \"parser_sha\": \"\([0-9a-f]*\)\".*/\1/p" "$BASELINE")
    rec_i=$(sed -n "s/.*\"$fam\":.*\"input_sha\": \"\([0-9a-f]*\)\".*/\1/p" "$BASELINE")
    if [ -z "$rec_p" ] || [ -z "$rec_i" ]; then
      breach "$fam has no recorded row in $BASELINE, so its reproducibility is UNPROVEN while its siblings' is. Re-record with \`make -C rust SHELL=/bin/bash generated_reproducibility_rebaseline\`."
      continue
    fi
    local p="$GENERATED/${fam}_parser.rs" j="$GENERATED/${fam}.json"
    [ -f "$p" ] && [ -f "$j" ] || { breach "$p or $j is absent while other artifacts are present — the tree is half-generated"; continue; }
    local live_p live_i; live_p=$(sha_file "$p"); live_i=$(sha_file "$j")
    if [ "$live_p" != "$rec_p" ] || [ "$live_i" != "$rec_i" ]; then
      breach "$fam moved since it was last PROVEN to re-derive from HEAD (parser ${live_p:0:12}… vs ${rec_p:0:12}…, input ${live_i:0:12}… vs ${rec_i:0:12}…).
      Tier 1 detects MOVEMENT; only tier 2 can say whether the new state is correct:
        make -C rust SHELL=/bin/bash generated_reproducibility_gate"
      continue
    fi
    checked=$((checked + 1))
  done

  [ "$fail" = 0 ] && printf 'generated-reproducibility: OK (%d artifacts unmoved, emission sources unmoved since %s — tier 2 last proved them byte-identical to HEAD)\n' \
    "$checked" "$(sed -n 's/.*"verified_at_commit": "\([0-9a-f]\{7\}\).*/\1/p' "$BASELINE")"
  return 0
}

# ── --self-test: every refusal arm must be OBSERVED firing ────────────────────────────────────────
# A control never seen RED is not known to work (`docs/CLAIM_VERIFICATION.md` §3 leg 2). Each arm
# perturbs a real input in a scratch copy of the baseline and asserts the expected exit code.
if [ "${1:-}" = "--self-test" ]; then
  arms=0; bad=0
  T="rust/target/generated_reproducibility_selftest"; rm -rf "$T"; mkdir -p "$T"
  cp "$BASELINE" "$T/baseline.orig"
  arm() { # arm <name> <expected-rc> <command…>
    local name="$1" want="$2"; shift 2
    "$@" >"$T/out" 2>&1; local got=$?
    arms=$((arms + 1))
    if [ "$got" = "$want" ]; then printf '  ✓ %-46s rc=%s\n' "$name" "$got"
    else printf '  ✗ %-46s rc=%s (expected %s)\n' "$name" "$got" "$want" >&2; bad=$((bad + 1)); fi
  }
  printf 'generated-reproducibility: --self-test\n'

  arm "GREEN control: the tree as it stands"          0 bash "$0"

  python3 - "$BASELINE" "$T/baseline.orig" <<'PY'
import re, sys
p, orig = sys.argv[1], sys.argv[2]
s = open(orig).read()
open(p, "w").write(re.sub(r'"emission_sha": "[0-9a-f]+"', '"emission_sha": "deadbeef"', s))
PY
  arm "RED: emission sources no longer match"          1 bash "$0"

  python3 - "$BASELINE" "$T/baseline.orig" <<'PY'
import re, sys
s = open(sys.argv[2]).read()
open(sys.argv[1], "w").write(re.sub(r'("json": \{ "parser_sha": ")[0-9a-f]+', r'\1' + "0"*64, s))
PY
  arm "RED: one artifact moved since it was proven"    1 bash "$0"

  python3 - "$BASELINE" "$T/baseline.orig" <<'PY'
import re, sys
s = open(sys.argv[2]).read()
open(sys.argv[1], "w").write(re.sub(r'\s*"json": \{[^}]*\},?', '', s, count=1))
PY
  arm "RED: an artifact has no recorded row at all"    1 bash "$0"

  python3 - "$BASELINE" "$T/baseline.orig" <<'PY'
import re, sys
s = open(sys.argv[2]).read()
open(sys.argv[1], "w").write(re.sub(r'"emission_sha": "[0-9a-f]+",\n', '', s))
PY
  arm "REFUSE(2): baseline carries no emission_sha"    2 bash "$0"

  cp "$T/baseline.orig" "$BASELINE"
  arm "REFUSE(2): unknown argument"                    2 bash "$0" --nonsense
  arm "GREEN control again: baseline restored"         0 bash "$0"

  # ── `ENGINE-UNIVERSAL-SERVICES.32` — the generator-currency refusal path ────────────────────────
  # ⛔ EXIT CODE CANNOT DISCRIMINATE HERE: a NOT-EVALUATED note returns 0, exactly like a pass, which
  # is the whole hazard this arm guards. It therefore asserts on OUTPUT — the family rows must be
  # ABSENT and the refusal note present — rather than on rc.
  # ⚠️ HONEST SCOPE, so nobody reads more into it than it proves: this fires the gate's REFUSAL when
  # the currency step fails. It does NOT re-prove that cargo detects a stale generator — that is
  # cargo's own contract, and the end-to-end demonstration (pair correct, families stale, matching
  # stale binary, gate printing "every checked artifact is what HEAD produces" over eight artifacts
  # that were not) is a one-shot measurement recorded in `.32`, not something to re-run per commit.
  arms=$((arms + 1))
  PGEN_GENREPRO_SELFTEST=1 PGEN_GENREPRO_SELFTEST_FAIL_CURRENCY=1 bash "$0" --verify >"$T/currency" 2>&1
  if grep -q 'the generator currency step FAILED' "$T/currency" \
     && ! grep -q '✓ systemverilog' "$T/currency" \
     && ! grep -q 'TIER 2 OK' "$T/currency"; then
    printf '  ✓ %-46s refused\n' "RED: family generator not proven current"
  else
    printf '  ✗ %-46s did NOT refuse — a stale generator would pass by construction\n' \
      "RED: family generator not proven current" >&2; bad=$((bad + 1))
  fi
  # And the same run WITHOUT the injection must reach the families, or the arm above is vacuous.
  arms=$((arms + 1))
  bash "$0" --verify >"$T/currency_ok" 2>&1
  if grep -q '✓ systemverilog' "$T/currency_ok" && grep -q 'TIER 2 OK' "$T/currency_ok"; then
    printf '  ✓ %-46s reached\n' "GREEN pair: the families ARE checked normally"
  else
    printf '  ✗ %-46s the un-injected run did not reach the families, so the RED arm proves nothing\n' \
      "GREEN pair: the families ARE checked normally" >&2; bad=$((bad + 1))
  fi

  cp "$T/baseline.orig" "$BASELINE"; rm -rf "$T"
  printf '\n%d/%d arms behaved as designed\n' "$((arms - bad))" "$arms"
  [ "$bad" = 0 ] || printf 'generated-reproducibility: --self-test FAILED\n' >&2
  exit $(( bad > 0 ? 1 : 0 ))
fi

tier1
exit "$fail"
