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
# ⭐⭐ THE ROSTER IS 11, NOT 10, SINCE `ENGINE-UNIVERSAL-SERVICES.16` — THE ARTIFACT NOTHING COULD SEE
# ROT IS NOW COVERED. `generated/ebnf.rs` is the META-PARSER: arm 2 of the frontend differential, and
# the artifact `--features ebnf_dual_run` compiles in. `rust/Makefile` seeds it under `if [ ! -f … ]`
# and NEVER regenerates it, so its only staleness check was that branch's `else`: *"does it still
# compile"* — the weakest of the four candidates this header already rejects. `.16` measured the local
# copy dated 2026-07-30 while the grammar and the code generator had both moved.
# ⛔ MEASURED END TO END, because the two checks disagree exactly where it matters. With one comment
# line appended to `generated/ebnf.rs` — still valid Rust, still not what the source produces:
#     cargo build --features ebnf_dual_run --bin ast_pipeline   ->  rc 0, 0 rustc errors  (sees nothing)
#     this gate                                                 ->  `ebnf DOES NOT re-derive from
#         HEAD: live 219a07581ef9… (11 668 785 B) vs fresh 6a37b20a17a3… (11 668 701 B)`, rc 1
# ⚠️ `.16` acceptance (b) asked for *"a gate that FAILS when the artifact is older than its inputs"*.
# That is deliberately NOT what shipped: mtime is the wrong instrument for this question in this
# repository, measured — GNU Make 3.81 compares mtimes at WHOLE SECONDS, so a prerequisite rewritten
# inside the same second is invisible and a rule is skipped at exit 0 (`CI-PARITY-GATE-ROT.32`, 10 of
# 10 families exposed on the json→parser edge). A gate keyed on that comparison inherits the blind
# spot. Re-derive-and-diff answers what mtime approximates, in both directions.
# ⚠️ `ebnf` is NOT in `GENERATED_PARSER_FAMILIES` and must not be: its recipe takes the BOOTSTRAP
# flags with the ORDINARY binary, and the frontend binary that produces every other family's `.json`
# is compiled FROM it. It is a cohort of its own here, which needs no `focus_ebnf` target inside that
# cycle.
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
#   - ✅ **Cargo sees the CRATE, not the RECIPE — CLOSED by `ENGINE-UNIVERSAL-SERVICES.33`.** This
#     bound used to read: *"this script MIRRORS the Makefile's generator flags in
#     `rederive_and_compare` rather than reading them from it."* It no longer mirrors them —
#     `derive_generator_recipe` READS `RUST_GENERATOR` / `RUST_GENERATOR_BOOTSTRAP` out of
#     `rust/Makefile`, the way the family roster was already read, and REFUSES (exit 2) on any shape
#     it cannot resolve rather than falling back to a default. See THE RECIPE IS DERIVED below.
#   - ⚠️ It MUTATES `rust/target/`: announced on every run, and the reason this lives in tier 2 (on
#     demand) and never in tier 1.
# Neither surviving bound is the mechanism that produced the defect — that was `make` skipping a
# rebuild on GNU Make 3.81's whole-second mtime comparison (`CI-PARITY-GATE-ROT.37`), which cargo
# detects exactly.
#
# ⭐⭐ THE RECIPE IS DERIVED FROM `rust/Makefile`, NOT MIRRORED (`ENGINE-UNIVERSAL-SERVICES.33`).
# ⛔⛔ AND THE FALSE PASS THE MIRROR ALLOWED WAS MEASURED END TO END, NOT REASONED ABOUT — including
# the step that made it permanent. With `--indirect-lr-admit-starvation-safe-only` added to
# `RUST_GENERATOR` (a real emission-affecting flag; the repository's own docstring says *"NOT the
# shipped policy, and nothing in `rust/Makefile` passes this"*) and the artifacts left alone, which
# is the ORDINARY state during a recipe change:
#
#   1. tier 1 BREACHED correctly — `rust/Makefile` is inside `emission_sha`, so the alarm fired;
#   2. the operator did exactly what the breach message instructs, `--rebaseline`, and tier 2
#      re-derived with its own STALE hard-coded flags, matched, printed
#      `✓ systemverilog re-derives byte-identically`, and RECORDED the baseline;
#   3. tier 1 then reported `OK … tier 2 last proved them byte-identical to HEAD`.
#
# What `make` would actually have emitted for SystemVerilog under that recipe: **130 878 616 B**
# (`d518dec16abb…`) against the `143 072 420 B` (`592bccec3bfc…`) on disk — **12 193 804 B** and a
# different left-recursion admission policy. ⇒ tier 1 does not save the check from a stale mirror;
# it ROUTES THE OPERATOR INTO the false pass, and `--rebaseline` launders it into the baseline that
# silences tier 1. A mirror inside the oracle is worse than a mirror beside it.
#
# ⚠️ THE FIRST PERTURBATION CHOSEN FOR THAT DEMONSTRATION WAS VACUOUS, AND ONLY A CONTROL CAUGHT IT.
# Dropping `--eliminate-left-recursion` from `RUST_GENERATOR` produces a **byte-identical** artifact
# for json / regex / vhdl / systemverilog, because `main.rs:1104` reads
# `if args.eliminate_left_recursion { config.eliminate_left_recursion = true; }` over a field that
# `PipelineConfig::default()` already sets to `true`, and there is no negating flag — so the flag in
# the shipping recipe **cannot change emission**. A demonstration built on it would have "reproduced"
# a false pass that was not false. Routed as a census finding, not fixed here.
#
# HOW THE DERIVATION REFUSES RATHER THAN GUESSES. `derive_generator_recipe <VAR> <expected-binary>`
# takes the single `^<VAR> = ` definition, asserts word 0 is the expected `$(RUST_AST_PIPELINE…)`
# reference, and returns the remaining words as the flag list. It REFUSES (exit 2) on: no definition,
# more than one definition, a different leading binary reference, an empty flag list, a non-flag
# token, or a flag carrying an unresolved make expansion (`$(…)`/`${…}`) — because resolving one
# would be a second implementation of make's expansion, which is the very class this leaf exists to
# remove. Every refusal is fired by `--self-test`.
#
# ⭐ AND THE CALL SITES ARE CHECKED TOO, because reading the variable is only half the recipe: a
# `$(RUST_GENERATOR) … -o …` line could add a generator flag of its own and the variable would still
# read clean. All 21 call sites are uniform today; `assert_call_sites_add_no_flags` holds them that
# way and refuses on the first that is not.
#
# ✅ THE BOUND `.33` STATED HERE IS CLOSED BY `ENGINE-UNIVERSAL-SERVICES.16`. It read: *"a recipe that
# bypasses `$(RUST_GENERATOR…)` entirely is outside this derivation — `rust/Makefile:980` seeds
# `generated/ebnf.rs` with the bootstrap flags spelled inline, a THIRD copy of the flag list inside
# the Makefile itself."* That copy is gone: the flags now live in `GENERATOR_FLAGS` /
# `GENERATOR_FLAGS_BOOTSTRAP`, which all three spellings reference, and this check reads THOSE.
# ⚠️ WHAT REMAINS is narrower and named: the derivation understands a composed variable of the exact
# shape `<binary> $(<flag-variable>)` and REFUSES anything else, so a recipe wrapped in `env …` or
# built by `$(if …)` is a refusal rather than a silent mis-read. Resolving those would need a general
# resolver for make's expansion — the duplicate implementation rejected above.
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

# The recipe's home. ⛔ NOT a knob: the only way to point this elsewhere is the two-variable
# self-test pair below, so an escape hatch cannot reach an ordinary run (the rule this script's
# `PGEN_PARSE_COST_ALLOW_PROBE_MISMATCH` sibling states: a hatch that reaches the gate is a hole).
MAKEFILE="rust/Makefile"
if [ "${PGEN_GENREPRO_SELFTEST:-}" = 1 ] && [ -n "${PGEN_GENREPRO_SELFTEST_MAKEFILE:-}" ]; then
  MAKEFILE="$PGEN_GENREPRO_SELFTEST_MAKEFILE"
fi

# The two annotation parsers come from `ast_pipeline_bootstrap` in --bootstrap-mode; the eight
# families from `ast_pipeline`. Both rosters are the Makefile's, mirrored here and CHECKED against
# it by tier 2 so the mirror cannot drift silently.
PAIR=(return_annotation semantic_annotation)
FAMILIES=(json regex systemverilog systemverilog_preprocessor vhdl rtl_const_expr rtl_frontend scratch)
# ⭐⭐ THE SEED COHORT — `generated/ebnf.rs` (`ENGINE-UNIVERSAL-SERVICES.16`). It is the META-PARSER:
# the artifact `--features ebnf_dual_run` compiles in as arm 2 of the frontend differential, and the
# one every grammar is read through on that path. ⛔ IT WAS THE ONE ARTIFACT NOTHING COULD SEE ROT,
# and that is measured, not suspected: `rust/Makefile`'s bootstrap generates it under
# `if [ ! -f … ]` and then NEVER regenerates it, so its only staleness check was the `else` branch's
# *"does it still compile"* — which this doctrine's own header names as the weakest of the four
# candidates it rejected. `.16` measured the local copy dated 2026-07-30 while the grammar and the
# code generator had both moved. ⇒ a stale arm 2 makes `ebnf_dual_run_diff` compare today's
# hand-written frontend against a fortnight-old meta-parser: green locally, a different comparison on
# a cold runner.
# ⚠️ IT IS NOT IN `GENERATED_PARSER_FAMILIES` AND MUST NOT BE. Its recipe takes the BOOTSTRAP FLAGS
# with the ORDINARY binary (the annotation pair does not exist when it is seeded), and the frontend
# binary that produces every other family's `.json` is itself compiled FROM it — so it is a cohort of
# its own here rather than an eighth family there. Joining the roster would need a `focus_ebnf` target
# inside that cycle; joining THIS gate needs neither, and re-derive-and-diff is strictly stronger than
# the mtime comparison `.16` asked for (see the ⛔ note in run_tier2).
SEED=(ebnf)

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

# ⛔ THE SEED ARTIFACT DOES NOT FOLLOW THE `<family>_parser.rs` NAMING, and hard-coding that pattern
# everywhere is how it stayed outside this gate. One helper, so a caller cannot get it wrong.
artifact_rel() {
  case "$1" in
    ebnf) printf '%s\n' "$GENERATED/ebnf.rs" ;;
    *)    printf '%s\n' "$GENERATED/$1_parser.rs" ;;
  esac
}
# The `-o` spelling `rust/Makefile` passes, relative to `rust/` — mimicked exactly (see the mimic-tree
# note on rederive_and_compare).
artifact_out() {
  case "$1" in
    ebnf) printf '../generated/ebnf.rs\n' ;;
    *)    printf '../generated/%s_parser.rs\n' "$1" ;;
  esac
}

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

# ── the generator recipe, DERIVED from the Makefile (`ENGINE-UNIVERSAL-SERVICES.33`) ──────────────
#
# derive_generator_recipe <flag-variable> <composed-variable> <expected leading binary reference>
#   Prints the generator FLAGS space-separated on ONE line. Refuses with exit 2 rather than falling
#   back to a default, because a default IS the mirror this replaces: a fallback that happens to be
#   right today is indistinguishable from a fallback that is silently wrong tomorrow, and the wrong
#   direction here PASSES (see the header's measured chain).
#   ⛔ ONE LINE, and read back with `read -ra`, deliberately: a `mapfile` from a process substitution
#   reports the status of `mapfile`, NOT of the refusing subshell, so every `die` below would be
#   printed and then IGNORED — and `mapfile` is bash 4+, which the git hook's shell need not be. A
#   space-separated line is lossless here because a token containing whitespace cannot survive the
#   `read -ra` split above it.
#
#   ⭐⭐ IT READS THE FLAG LIST, THEN PROVES THE VARIABLE THE CALL SITES USE IS BUILT FROM IT
#   (`ENGINE-UNIVERSAL-SERVICES.16`). `rust/Makefile` now separates `GENERATOR_FLAGS` from
#   `RUST_GENERATOR = $(RUST_AST_PIPELINE) $(GENERATOR_FLAGS)`, because the `generated/ebnf.rs` seed
#   needs the BOOTSTRAP FLAGS with the ORDINARY binary and so could not share either composed
#   variable — that third inline copy is what `.33`'s census found and what `.16` needed removed.
#   Reading the flag list alone would be weaker than the mirror it replaced: a `RUST_GENERATOR` that
#   stopped referencing `$(GENERATOR_FLAGS)` would leave this check re-deriving with flags nothing
#   passes. So the composed line is asserted to be EXACTLY `<binary> $(<flag-variable>)`.
GEN_FLAGS=()            # GENERATOR_FLAGS           — the eight families
GEN_FLAGS_BOOTSTRAP=()  # GENERATOR_FLAGS_BOOTSTRAP — the annotation pair AND the ebnf seed

# read_make_variable <name> — the single `^<name> = ` definition, or a refusal.
read_make_variable() {
  local var="$1" defs
  [ -f "$MAKEFILE" ] || die "$MAKEFILE is absent, so the generator recipe cannot be derived. This gate re-derives every artifact through the Makefile's recipe; it will not guess one."
  # ⛔ EXACTLY ONE definition. Two would mean the LAST one wins in make while `sed` prints both, so
  # picking either is a coin toss dressed as a derivation.
  defs=$(grep -c "^${var} = " "$MAKEFILE")
  case "$defs" in
    1) ;;
    0) die "$MAKEFILE has no '^${var} = ' definition, so the generator recipe cannot be derived. If the variable was renamed, teach this check the new name — do NOT let it fall back to a hard-coded flag list." ;;
    *) die "$MAKEFILE defines ${var} $defs times. make takes the last; a derivation that picked one would be guessing. Collapse them to one definition." ;;
  esac
  sed -n "s/^${var} = //p" "$MAKEFILE"
}

derive_generator_recipe() {
  local flag_var="$1" composed_var="$2" want_bin="$3" line composed tok
  local -a words

  line=$(read_make_variable "$flag_var") || exit 2
  read -ra words <<< "$line"   # word-split on IFS, and NOT glob-expanded (unlike `set -- $line`)
  [ "${#words[@]}" -ge 1 ] || die "$MAKEFILE's ${flag_var} is defined but empty — a generator invocation with no --generate-parser cannot produce a parser, so this is a malformed recipe rather than an empty one."

  for tok in "${words[@]}"; do
    case "$tok" in
      *'$('*|*'${'*) die "$MAKEFILE's ${flag_var} carries the unresolved make expansion '$tok'. Resolving it here would be a SECOND implementation of make's expansion — the duplication ENGINE-UNIVERSAL-SERVICES.33 removed — so this refuses instead. Inline the value, or extend the derivation deliberately." ;;
      -*) ;;
      *) die "$MAKEFILE's ${flag_var} carries the non-flag token '$tok'. A positional argument in the flag list means the shape changed (an input file moved into it?), and appending it to a command line would build a different invocation." ;;
    esac
  done

  # ⛔ AND THE COMPOSED VARIABLE MUST STILL BE BUILT FROM THAT LIST. Without this, a `RUST_GENERATOR`
  # edited to spell its flags inline again would leave this check re-deriving with a list nothing
  # passes — the mirror restored, silently, in the passing direction.
  composed=$(read_make_variable "$composed_var") || exit 2
  [ "$composed" = "$want_bin \$($flag_var)" ] || die "$MAKEFILE's ${composed_var} is '${composed}', not '${want_bin} \$(${flag_var})'. This check re-derives with ${flag_var}'s list, so the variable the recipes actually invoke must be exactly that binary plus that list — otherwise the two can diverge and this gate would compare against a command line \`make\` does not run."

  printf '%s\n' "${words[*]}"
}

# ⭐ READING THE VARIABLE IS ONLY HALF THE RECIPE. A `$(RUST_GENERATOR) $(X_JSON) -o $(X_PARSER)`
# line could add a generator flag of its own, and the variable would still read clean — so the
# derived flag list would be short while `make` passed more. All 21 call sites are uniform today
# (measured, `.33` census); this holds them that way.
assert_call_sites_add_no_flags() {
  local offenders
  offenders=$(MAKEFILE="$MAKEFILE" python3 - <<'PY'
import os, re, sys

path = os.environ["MAKEFILE"]
ref = re.compile(r"\$\(RUST_GENERATOR(?:_BOOTSTRAP)?\)")
bad = []
for lineno, raw in enumerate(open(path, encoding="utf-8", errors="replace"), 1):
    line = raw.rstrip("\n")
    if line.lstrip().startswith("#"):
        continue
    m = ref.search(line)
    if not m or line.startswith("RUST_GENERATOR"):
        continue                       # the definitions themselves are derive_generator_recipe's job
    tail = line[m.end():]
    # Everything the recipe passes BEFORE `-o` is a generator argument. A shell tail after the
    # output path (`2>&1 | tee …`) is not, so the scan stops at the first `-o`.
    head = tail.split(" -o ", 1)[0] if " -o " in tail else tail
    extra = [t for t in head.split() if t.startswith("-")]
    if " -o " not in tail:
        bad.append(f"{lineno}: no ' -o ' in the invocation, so its argument list cannot be bounded: {line.strip()}")
    elif extra:
        bad.append(f"{lineno}: adds generator flag(s) {' '.join(extra)}: {line.strip()}")
print("\n".join(bad))
PY
  ) || die "the call-site scan of $MAKEFILE could not run (python3 failed), so this gate cannot know which flags make passes"
  [ -z "$offenders" ] || die "$MAKEFILE passes generator flags at a CALL SITE, which the derived recipe variable does not see:
$(printf '%s\n' "$offenders" | sed 's/^/        /')
      This gate re-derives with the variable's flags alone, so its comparison would measure a different command line than \`make\` runs. Move the flag into the recipe variable, or teach the derivation to read call sites."
}

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
  local fam="$1" kind="$2" tool="$3"   # kind = pair|family|seed
  local rel; rel=$(artifact_rel "$fam")
  local out; out=$(artifact_out "$fam")
  local live="$ROOT/$rel"
  local json="$ROOT/$GENERATED/${fam}.json"
  local fresh="$WORK/root/${rel}"

  [ -f "$live" ] || { breach "$rel is absent while other artifacts are present — regenerate with \`make -C rust SHELL=/bin/bash regenerate_generated_parsers\`"; return; }
  [ -f "$json" ] || { breach "$GENERATED/${fam}.json is absent, so ${fam} cannot be re-derived"; return; }

  # ⭐ THE FLAGS ARE THE MAKEFILE'S, READ FROM IT (`ENGINE-UNIVERSAL-SERVICES.33`) — never a copy
  # kept here in step with it. `run_tier2` derives them once and refuses before reaching this loop.
  # ⛔ The SEED cohort takes the BOOTSTRAP flags with the ORDINARY binary, which is exactly why the
  # Makefile had to grow a flag-only variable: no composed variable expresses that pair.
  local -a args=("${GEN_FLAGS[@]}")
  case "$kind" in pair|seed) args=("${GEN_FLAGS_BOOTSTRAP[@]}") ;; esac

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
    || die "cannot compare ${fam}: the shared helper REFUSED on $live (ambiguous embedded path). Absence is fine and reports 0; ambiguity is not."
  fresh_sites=$(python3 "$ROOT/scripts/compare_generated_parsers.py" --sites "$fresh") \
    || die "cannot compare ${fam}: the shared helper REFUSED on $fresh (ambiguous embedded path). Absence is fine and reports 0; ambiguity is not."
  live_spelling=$(python3 "$ROOT/scripts/compare_generated_parsers.py" --spelling "$live")
  fresh_spelling=$(python3 "$ROOT/scripts/compare_generated_parsers.py" --spelling "$fresh")
  # ⛔⛔ THE POLARITY OF THIS TEST INVERTED AT `ENGINE-UNIVERSAL-SERVICES.31` (e), AND LEAVING IT
  # ALONE WOULD HAVE MADE THE GATE REFUSE ON EVERY CORRECT TREE. `live_sites = 0` used to mean
  # *"the helper could not derive anything — something is wrong"*. It now means *"this artifact
  # embeds no output path at all"*, which is the STRONGEST possible state: with nothing to
  # normalise, the byte comparison below is unconditionally about the source. What the comparison
  # actually needs is only that the two sides AGREE, so that is all this asserts.
  if [ "$live_sites" != "$fresh_sites" ] || [ "$live_spelling" != "$fresh_spelling" ]; then
    die "cannot compare ${fam}: embedded -o sites live=$live_sites ('$live_spelling') fresh=$fresh_sites ('$fresh_spelling'). The two sides were not written through the same path spelling, so any verdict would measure the PATH (TOOLBOX 5.6), not the source."
  fi
  # ⭐ AND THE RETIRED PROPERTY GETS A TRIPWIRE RATHER THAN A DELETION. No generated parser should
  # embed its `-o` path any more; if one starts again, the emitter has regressed toward the defect
  # `.31` (e) removed — an artifact whose SIZE is a function of its own output path, and a
  # diagnostic label naming a file the position does not index. Reported loudly, not silently
  # tolerated, exactly as `LIVE-DOC-CURRENCY`'s dormant instrument B stays wired for re-introduction.
  if [ "$live_sites" != 0 ]; then
    breach "$fam embeds its -o path $live_sites time(s) — since ENGINE-UNIVERSAL-SERVICES.31 (e) a generated parser embeds it ZERO times. The emitter has started writing its output path into the artifact again; see TOOLBOX.md 5.6."
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
  # ⛔ THE RECIPE IS DERIVED FIRST, BEFORE `generated_present`, and it can only REFUSE. A malformed
  # recipe is a breach whether or not `generated/` exists, and putting it first means every refusal
  # arm costs nothing — no bootstrap build, no codegen.
  local recipe
  recipe=$(derive_generator_recipe GENERATOR_FLAGS RUST_GENERATOR '$(RUST_AST_PIPELINE)') || exit 2
  read -ra GEN_FLAGS <<< "$recipe"
  recipe=$(derive_generator_recipe GENERATOR_FLAGS_BOOTSTRAP RUST_GENERATOR_BOOTSTRAP '$(RUST_AST_PIPELINE_BOOTSTRAP)') || exit 2
  read -ra GEN_FLAGS_BOOTSTRAP <<< "$recipe"
  assert_call_sites_add_no_flags
  printf 'generated-reproducibility: recipe DERIVED from %s — families: %s | pair + ebnf seed: %s\n' \
    "$MAKEFILE" "${GEN_FLAGS[*]}" "${GEN_FLAGS_BOOTSTRAP[*]}"

  generated_present || { skipped="${skipped} every artifact"; note "NOT EVALUATED — $GENERATED/ holds no generated parser. Regenerate with \`make -C rust SHELL=/bin/bash regenerate_generated_parsers\`, then re-run."; return 0; }

  # The roster must match the Makefile's, or this gate silently checks a subset.
  local mk_fams
  mk_fams=$(sed -n 's/^GENERATED_PARSER_FAMILIES = //p' "$MAKEFILE")
  [ -n "$mk_fams" ] || die "could not read GENERATED_PARSER_FAMILIES from $MAKEFILE — the roster mirror cannot be checked, so this gate would silently check a subset"
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

  # ⭐⭐ THE SEED COHORT (`ENGINE-UNIVERSAL-SERVICES.16`) — same generator as the families, BOOTSTRAP
  # flags, and it rides the currency proof above rather than a second one.
  # ⛔ THIS IS WHAT `.16` ASKED FOR, AND IT IS DELIBERATELY NOT WHAT `.16` ASKED FOR. Its acceptance
  # (b) wanted *"a gate that FAILS when the artifact is older than its inputs"* — an mtime comparison.
  # This repository has since MEASURED mtime to be the wrong instrument for exactly this question:
  # `/usr/bin/make` here is GNU Make 3.81, which compares mtimes at WHOLE SECONDS, so a prerequisite
  # rewritten inside the same second is invisible and a rule is skipped at exit 0
  # (`CI-PARITY-GATE-ROT.32`, 10 of 10 families exposed on the json→parser edge). A gate keyed on the
  # same comparison inherits the same blind spot. Re-derive-and-diff answers the question mtime only
  # approximates, in both directions, and it is what the other ten artifacts already get.
  for f in "${SEED[@]}"; do rederive_and_compare "$f" seed "$ROOT/$pipeline"; done
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
    for fam in "${PAIR[@]}" "${FAMILIES[@]}" "${SEED[@]}"; do
      local p; p=$(artifact_rel "$fam")
      local j="$GENERATED/${fam}.json"
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
  for fam in "${PAIR[@]}" "${FAMILIES[@]}" "${SEED[@]}"; do
    local rec_p rec_i
    rec_p=$(sed -n "s/.*\"$fam\": { \"parser_sha\": \"\([0-9a-f]*\)\".*/\1/p" "$BASELINE")
    rec_i=$(sed -n "s/.*\"$fam\":.*\"input_sha\": \"\([0-9a-f]*\)\".*/\1/p" "$BASELINE")
    if [ -z "$rec_p" ] || [ -z "$rec_i" ]; then
      breach "$fam has no recorded row in $BASELINE, so its reproducibility is UNPROVEN while its siblings' is. Re-record with \`make -C rust SHELL=/bin/bash generated_reproducibility_rebaseline\`."
      continue
    fi
    local p; p=$(artifact_rel "$fam")
    local j="$GENERATED/${fam}.json"
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

  # ── `ENGINE-UNIVERSAL-SERVICES.33` — the DERIVED generator recipe ───────────────────────────────
  # Every arm below perturbs a SCRATCH copy of `rust/Makefile` and is reached through the two-variable
  # self-test pair, so no arm can be triggered from an ordinary run and none of them touches the real
  # Makefile. ⭐ They are cheap by construction: `run_tier2` derives the recipe BEFORE it builds
  # anything, so a refusal costs no bootstrap build and no codegen.
  # ⛔ THE PERTURBATION TEXT TRAVELS THROUGH THE ENVIRONMENT, NEVER THROUGH A RE-EXPANDED STRING.
  # Every string involved contains `$(…)`, so interpolating it into a `python3 -c "…"` argument would
  # hand it to bash's command substitution a second time. And `mk` REFUSES when its target text is
  # not found: a perturbation that silently matched nothing would leave an arm testing an unmodified
  # Makefile and reporting ✓ — a control that cannot go RED.
  # ⛔⛔ `out` IS ASSIGNED ON ITS OWN LINE, AND THE ONE-LINE FORM WAS A REAL BUG THIS ARM SET CAUGHT.
  # `local name="$1" out="$T/Makefile.$name"` expands every word BEFORE `local` runs, so `$name` was
  # read before it was assigned: under `set -u` the direct call died with `name: unbound variable`,
  # and the seven arms reached through `arm_mk` "passed" only because bash's dynamic scoping handed
  # them `arm_mk`'s own `name`. Seven controls agreeing for an accidental reason, found by the eighth.
  mk() { # mk <slug> <old-literal> <new-literal>  -> prints the scratch Makefile path
    local slug="$1" out
    out="$T/Makefile.$(printf '%s' "$slug" | tr -cs 'A-Za-z0-9' '_')"
    MK_SRC="$MAKEFILE" MK_OUT="$out" MK_OLD="$2" MK_NEW="$3" python3 - <<'PY' || return 1
import os, sys
s = open(os.environ["MK_SRC"], encoding="utf-8").read()
old, new = os.environ["MK_OLD"], os.environ["MK_NEW"]
if old not in s:
    sys.exit("self-test: the Makefile text to perturb is not present: " + old[:90])
open(os.environ["MK_OUT"], "w", encoding="utf-8").write(s.replace(old, new, 1))
PY
    printf '%s\n' "$out"
  }
  # ⛔ The perturbation targets are the FLAG-ONLY variable and the COMPOSED variable that must be
  # built from it (`ENGINE-UNIVERSAL-SERVICES.16`). `mk` refuses when its target text is absent, so
  # renaming either variable in `rust/Makefile` makes these arms report ✗ rather than pass quietly —
  # which is how this block stayed honest when the Makefile was refactored under it.
  RECIPE='GENERATOR_FLAGS = --generate-parser --eliminate-left-recursion'
  COMPOSED='RUST_GENERATOR = $(RUST_AST_PIPELINE) $(GENERATOR_FLAGS)'
  CALLSITE='$(RUST_GENERATOR) $(JSON_JSON)'
  NARROW='--indirect-lr-admit-starvation-safe-only'
  arm_mk() { # arm_mk <name> <expected-rc> <old-literal> <new-literal>
    local name="$1" want="$2" path
    path=$(mk "$name" "$3" "$4") || { printf '  ✗ %-46s could not build the scratch Makefile\n' "$name" >&2; bad=$((bad + 1)); arms=$((arms + 1)); return; }
    arm "$name" "$want" env PGEN_GENREPRO_SELFTEST=1 PGEN_GENREPRO_SELFTEST_MAKEFILE="$path" bash "$0" --verify
  }

  arm_mk "REFUSE(2): flag variable is absent"          2 "$RECIPE" '# GENERATOR_FLAGS removed by the self-test'
  arm_mk "REFUSE(2): flag variable defined twice"      2 "$RECIPE" "$RECIPE"$'\n'"$RECIPE"
  arm_mk "REFUSE(2): a flag holds a make expansion"    2 "$RECIPE" "$RECIPE"' --profile=$(SOME_PROFILE)'
  arm_mk "REFUSE(2): a non-flag token in the flags"    2 "$RECIPE" "$RECIPE"' extra_positional.json'
  arm_mk "REFUSE(2): the flag list is EMPTY"           2 "$RECIPE" 'GENERATOR_FLAGS ='
  # ⭐ THE COMPOSITION ARM — reading the flag list alone would be WEAKER than the mirror it replaced:
  # a `RUST_GENERATOR` that stopped referencing `$(GENERATOR_FLAGS)` would leave this gate re-deriving
  # with flags nothing passes, silently and in the passing direction.
  arm_mk "REFUSE(2): composed var not built from flags" 2 "$COMPOSED" 'RUST_GENERATOR = $(RUST_AST_PIPELINE) --generate-parser --eliminate-left-recursion'
  arm_mk "REFUSE(2): composed var names another binary" 2 "$COMPOSED" 'RUST_GENERATOR = env PGEN_X=1 $(RUST_AST_PIPELINE) $(GENERATOR_FLAGS)'
  arm_mk "REFUSE(2): a CALL SITE adds a flag"          2 "$CALLSITE" '$(RUST_GENERATOR) '"$NARROW"' $(JSON_JSON)'

  # ⭐⭐ THE LOAD-BEARING ARM: the derived flags REACH the generator, so a recipe change cannot be
  # ignored. Before `.33` the flag list was hard-coded, and a recipe carrying an argument the
  # generator does not accept was invisible — the gate re-derived with its own list and printed
  # `TIER 2 OK`. ⚠️ HONEST SCOPE: it proves the flags are LIVE, not that any particular flag changes
  # emission. The end-to-end emission demonstration (`--indirect-lr-admit-starvation-safe-only` on
  # `RUST_GENERATOR`: `make` would emit a 130 878 616 B SystemVerilog parser against the 143 072 420 B
  # on disk, and the pre-fix gate declared it byte-identical and then RECORDED that) is a one-shot
  # measurement in `docs/tasks/artifacts/engine_universal_services/es33_makefile_mirror/`, not
  # something to re-spend a 130 MB codegen on per run — the same split `.32` made for cargo.
  arms=$((arms + 1))
  live_mk=$(mk live "$RECIPE" "$RECIPE"' --pgen-es33-selftest-not-a-real-flag')
  PGEN_GENREPRO_SELFTEST=1 PGEN_GENREPRO_SELFTEST_MAKEFILE="$live_mk" bash "$0" --verify >"$T/live" 2>&1
  live_rc=$?
  if [ "$live_rc" != 0 ] && grep -q 'codegen FAILED' "$T/live" && ! grep -q 'TIER 2 OK' "$T/live"; then
    printf '  ✓ %-46s rc=%s\n' "RED: the DERIVED flags reach the generator" "$live_rc"
  else
    printf '  ✗ %-46s rc=%s — a recipe change is being ignored, so the mirror is back\n' \
      "RED: the DERIVED flags reach the generator" "$live_rc" >&2; bad=$((bad + 1))
  fi

  # GREEN: the real Makefile derives exactly the recipe the project ships. Asserted on the derived
  # TEXT rather than on an exit code, because a derivation that silently returned the wrong flags
  # would still exit 0.
  arms=$((arms + 1))
  if grep -q 'recipe DERIVED from rust/Makefile — families: --generate-parser --eliminate-left-recursion | pair + ebnf seed: --generate-parser --bootstrap-mode --eliminate-left-recursion' "$T/currency_ok"; then
    printf '  ✓ %-46s exact\n' "GREEN: derived recipe == the shipped recipe"
  else
    printf '  ✗ %-46s the derived recipe is not the shipped one\n' "GREEN: derived recipe == the shipped recipe" >&2; bad=$((bad + 1))
  fi

  # ⭐ GREEN: the SEED cohort is actually REACHED (`ENGINE-UNIVERSAL-SERVICES.16`). Without this arm
  # the cohort could be silently skipped and every other arm would still be green — which is the
  # failure `.32` found in this same suite (a run that checked 2 of 10 announcing itself as full).
  arms=$((arms + 1))
  if grep -qE '✓ ebnf +re-derives byte-identically' "$T/currency_ok"; then
    printf '  ✓ %-46s reached\n' "GREEN: the ebnf SEED artifact is checked"
  else
    printf '  ✗ %-46s the seed-only artifact was not re-derived — .16 is not closed\n' \
      "GREEN: the ebnf SEED artifact is checked" >&2; bad=$((bad + 1))
  fi

  cp "$T/baseline.orig" "$BASELINE"; rm -rf "$T"
  printf '\n%d/%d arms behaved as designed\n' "$((arms - bad))" "$arms"
  [ "$bad" = 0 ] || printf 'generated-reproducibility: --self-test FAILED\n' >&2
  exit $(( bad > 0 ? 1 : 0 ))
fi

tier1
exit "$fail"
