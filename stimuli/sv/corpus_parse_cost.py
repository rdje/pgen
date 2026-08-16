#!/usr/bin/env python3
"""stimuli/sv/corpus_parse_cost.py — the SV parser's PARSE COST, on two metrics.

`ENGINE-UNIVERSAL-SERVICES.20` acceptance (d), under the 2026-08-14 ruling C.

⛔ WHY THIS EXISTS. `.17` slice 9 shipped the guarded left-recursion admission and the tree
could not say what it had cost. Every gate the repository has was GREEN across it — the
generated lint, the two-sided repro ratchet, the corpus pass/fail count and all registered
doctrines — because nothing here measured parse cost at all. *A cost nothing measures is a cost
that grows.* This instrument is the thing that measures it.

⛔⛔ AND THE FIGURE THAT COMMISSIONED IT WAS ITSELF UNMEASURED — REFUTED 2026-08-16 BY `.20`
SLICE 5 (`PGEN-ENGINE-UNIVERSAL-SERVICES-0050`). This file used to open by stating that the
admission cost **+24.3 % parse time**. That number is not reproducible from the raw data of the
runs that produced it: seven estimator × era combinations put ARM2/ARM1 in **[0.9909, 1.0433]**,
including the pre-`.22`(e) data of the very run that reported it. Its mechanism was a
FIXED-ARM-ORDER artifact — ARM 1 ran LAST in both contaminating passes, and on a session-drifting
host the last arm looks fastest. ⭐ That strengthens the case for this instrument instead of
weakening it: a wall-clock number nobody could re-derive was believed for three sessions, drove
five slices and a director-facing ruling, and was wrong by ~20 points. Everything published here
is re-derivable by construction. ⛔ The DETERMINISTIC costs of the same change STAND and are the
ones this instrument can see: **+10.59 % rule entries** (`guard_ab_entries.txt`) and +9.3 %
parser bytes.

⭐⭐ TWO METRICS, TWO GRAPHS, AND ONLY ONE OF THEM BINDS — which is the whole design.

PGEN runs a parse through one of two graphs (the "observability twin", TOOLBOX 2.1 / 3.4
ROUTING). A BARE parse runs the fused `cascade_*` functions; a parse with any diagnostic
consumer attached runs the PROTOCOL graph. They are different code, and the wall-clock claim
that opened `.20` was made on the fused one.

| metric | graph observed | machine-dependent? | verdict role |
|---|---|---|---|
| total rule ENTRIES | PROTOCOL (`parse_*`) | no — exact integer | **BINDING** |
| bare-parse wall clock | FUSED (`cascade_*`) | yes | ADVISORY, wide band |

- **Entries bind** because they are an exact integer: verified deterministic across repeated
  release runs AND byte-identical between the debug and release probes. A number that cannot
  drift with machine load is the only kind a standing ratchet can key on. This matters
  concretely: the first cost figure this leaf recorded, `~11 %`, was WRONG because it compared
  wall clock across "materially faster machine conditions". A wall-clock-primary ratchet
  inherits exactly that defect.
- **Wall clock advises** because it is the only view of the fused graph — and it is the half
  that cannot survive a machine change or a hosted runner. `.20` slice 5 is the proof of that
  claim, not an illustration of it: the one wall-clock figure this leaf ever published did not
  survive re-analysis of its own raw data.

⛔ THE BLIND SPOT IS DECLARED, NOT DISCOVERED LATER. The entry counters tick only in the
PROTOCOL graph. The fused twins of the very rules this leaf is about — `cascade_match_
casting_type_lr_suffix`, `cascade_build_select_expression_lr_base`, … — tick nothing. So the
binding metric CANNOT describe fused-graph cost, and the advisory one cannot be trusted to a
few percent. Neither is sufficient alone; the report says so on every run rather than
publishing one number that quietly means less than it appears to.

⚠️ THE PROCESS FLOOR IS SUBTRACTED, BECAUSE IT MEASURABLY DOMINATES. A `parseability_probe`
invocation on a one-line module takes ~9.8 ms before it parses anything (fork + exec + the SV
stdlib preload). The median corpus file's whole wall time is ~18 ms. An unadjusted per-file
wall-clock advisory is therefore mostly a measurement of `fork`, and would drift with the
loader rather than with the parser. The floor is re-measured in the SAME run — never assumed —
and subtracted, and the advisory sample is restricted to files where the remaining signal
dominates.

MODES
  (default)                measure the pinned sample -> the tracked baseline artifacts
  --census                 full-corpus entry census; the input to --select
  --select                 derive the pinned sample manifest FROM a census, deterministically
  --verify-families        run the LR-family predicate over EVERY generated parser's declared
                           rule names and refuse on any unclassified `_lr` name (`.21` (g)).
                           Reads the artifacts, not a parse: no probe, ~0.1 s.
  --verify-family-share    re-hash every input the carried corpus family share is a function of
                           and fail when one moved (`.21` (f)). ~0.8 s — the every-run tier.
  --rederive-family-share  DERIVE that share from a full-corpus census (~70 s) into the tracked
                           artifact. Reports disagreement with the carried constant; never
                           edits it.

USAGE
  python3 stimuli/sv/corpus_parse_cost.py --outdir <dir>
  python3 stimuli/sv/corpus_parse_cost.py --census --outdir <dir>
  python3 stimuli/sv/corpus_parse_cost.py --select --census-tsv <f> --out <manifest>
  python3 stimuli/sv/corpus_parse_cost.py --verify-families
  python3 stimuli/sv/corpus_parse_cost.py --verify-family-share
  python3 stimuli/sv/corpus_parse_cost.py --rederive-family-share

CONTRACT: deterministic on the binding metric; repo-root-relative paths everywhere; refuses
(exit 2) rather than reporting a clean measurement it could not take.
"""

from __future__ import annotations

import argparse
import concurrent.futures
import hashlib
import json
import os
import re
import statistics
import subprocess
import sys
import tempfile
import time

ROOT = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))

GRAMMAR = "systemverilog"
PROFILE = "sv_2017"
GRAMMAR_FILE = "grammars/systemverilog.ebnf"
GENERATED_PARSER = "generated/systemverilog_parser.rs"
DEFAULT_MANIFEST = "stimuli/sv/parse_cost_sample.tsv"
# ⛔ THE INSTRUMENT IS PART OF ITS OWN BASELINE'S IDENTITY (`ENGINE-UNIVERSAL-SERVICES.21`).
# The first identity block named three inputs — grammar, parser, sample bytes — on the sound
# argument that the BINDING counters are an exact function of exactly those. True, and
# insufficient: `entries.tsv` also publishes `lr_entries`/`lr_committed`, and `cost.md` a family
# share, all of which are functions of THIS FILE's classifier. When `.21` corrected that
# classifier, every one of those published numbers went stale and the identity tier still
# printed `fresh` — the same "an identity block nothing reads" defect `SV-CORPUS-GRAD.13i` is a
# record of, one input short instead of one gate short.
INSTRUMENT_FILE = "stimuli/sv/corpus_parse_cost.py"

# ── the corpus-wide family share, and why it is a GATED constant rather than a carried one ───
#
# This instrument measures the 192-file SAMPLE. The bound it publishes is a FULL-CORPUS figure,
# so it cannot be derived from a sample run and must be carried. It was prose inside the report
# writer until `ENGINE-UNIVERSAL-SERVICES.21`, which is how a single number ended up hand-copied
# into four surfaces and stale in all of them at once. One constant, one provenance string, one
# place to correct.
#
# ⛔⛔ AND THAT WAS NOT ENOUGH, WHICH IS WHAT ACCEPTANCE (f) IS A RECORD OF. Slice 1 replaced a
# WRONG unwatched number with a RIGHT unwatched number: the corrected `2.741` was hand-carried,
# referenced only by the file that defines it, and guarded by a COMMENT reading *"re-derive it,
# do not edit this line"*. A comment is prose, and `DOCTRINE_ENFORCEMENT.md` §1 is that a rule
# nothing checks is a suggestion — so the corrected figure would go stale exactly the way
# `0.681` did, silently, the next time the grammar moved.
#
# ⇒ IT IS NOW GATED (`docs/CLAIM_VERIFICATION.md` §5B — derived or gated, never carried):
#
#   `--rederive-family-share`  re-derives it from a full-corpus census (~70 s) and writes the
#                              TRACKED artifact below, naming every input the number depends on.
#   `--verify-family-share`    re-hashes those inputs (~0.8 s) and fails when any of them moved,
#                              so the carried figure cannot outlive the tree it describes. Wired
#                              into `PARSE-COST-RATCHET`'s every-run tier, not left on demand.
#
# ⛔ The re-derivation deliberately does NOT rewrite this constant. Generating the claim from the
# same run that measures it makes the comparison pass by construction — the failure mode
# `COMMIT.md` names for the DONE-BAR register's `claimed_status`. The instrument reports the
# disagreement; a human adopts it.
CORPUS_FAMILY_SHARE_PCT = "2.741"
FAMILY_SHARE_ARTIFACT = (
    "docs/tasks/artifacts/engine_universal_services/parse_cost_ratchet/family_share.json"
)
# v1 carried two fields this instrument no longer computes — `blind_spot_factor` and
# `wall_clock_regression_pct`. The schema is BUMPED rather than silently shrunk so a v1 artifact
# meets the reader's explicit refusal instead of being read for keys it happens not to have.
FAMILY_SHARE_SCHEMA = "pgen.parse_cost.family_share/v2"

# ── the RETIRED blind-spot factor, and why no number replaces it (`.26`, 2026-08-16) ──────────
#
# ⛔⛔ THIS INSTRUMENT PUBLISHED A SENSITIVITY BOUND OF `~8.9×` AND BOTH OF ITS TERMS WERE WRONG.
# It was derived as `WALL_CLOCK_REGRESSION_PCT / CORPUS_FAMILY_SHARE_PCT` = 24.3 / 2.741.
#
#   NUMERATOR — the `+24.3 %` is REFUTED (`.20` slice 5, `PGEN-ENGINE-UNIVERSAL-SERVICES-0050`):
#     not reproducible from the raw data of the runs that produced it, a fixed-arm-order artifact.
#   DENOMINATOR — the family SHARE was the wrong quantity, INDEPENDENTLY of the numerator. "How
#     sensitive is this counter to that change" is answered by how much the counter MOVED, not by
#     how large the family is. The bound reasoned that the flip's entry delta must be *"strictly
#     smaller"* than the 2.741 % share because the rules it replaced were themselves entered —
#     an INFERENCE, and a tracked artifact in this same leaf measures it: `guard_ab_entries.txt`
#     gives ARM 1 812 963 769 -> ARM 2 899 064 022, a delta of 86 100 253 = +10.59 %, which is
#     3.49x LARGER than the family's own 24 644 435 entries. The counters SAW that change plainly.
#
# ⇒ THE FACTOR IS RETIRED, NOT RE-COMPUTED. Under the point-estimate wall clock (<= +4.33 %) the
# factor would be 0.41x — the counter moving ~2.4x MORE than the clock — and under the most
# adversarial pairing available (+19.3 %) only 1.82x. It fails under every reading, and no
# admissible wall-clock figure for the change exists at all (`.20` (b): worst spread 65.2 s
# against a 0.6 s admissibility bar). Publishing any ratio here would be publishing an unearned
# number, which is the exact defect this whole leaf is a record of.
#
# ⭐ WHAT SURVIVES IS THE STRUCTURAL LIMIT, AND IT NEEDS NO NUMBER. These counters tick only in
# the PROTOCOL graph; a production parse runs the FUSED `cascade_*` graph. And a counter counts
# EVENTS — a rise in the cost PER event is invisible to it by construction, on any graph. That is
# a property of the metric, not a measurement, so it cannot go stale and does not expire.
FLIP_ENTRY_DELTA_PCT = "10.59"
FLIP_ENTRY_DELTA_PROVENANCE = (
    "ARM 1 812 963 769 -> ARM 2 899 064 022 rule entries over 16 335 corpus files, "
    "`docs/tasks/artifacts/engine_universal_services/guard_ab_entries.txt` "
    "(`ENGINE-UNIVERSAL-SERVICES.20` (b), three-arm A/B)"
)
# ⛔ CITED, NOT GATED — deliberately, and the reason is stated rather than left as an omission.
# It is a HISTORICAL A/B: ARM 1 is a parser built from the pre-flip admission policy, and that
# binary does not exist in this tree. Nothing here can re-derive the delta, so pinning it would
# compare a constant against itself and call the tautology a check
# (`docs/decisions/reference_self_referential_assertion_is_unsound.md`). The live, re-derivable
# number is the family SHARE below, and that one IS gated.


def format_family_provenance(art: dict) -> str:
    """The raw counts behind the carried share — READ from the tracked derivation, never carried.

    ⛔ THIS WAS A CARRIED STRING AND IT WENT STALE EXACTLY AS THE REST OF THIS FILE PREDICTS. It
    read *"24 644 435 of 899 064 022 entries over 16 335 files"* while the artifact it claims to
    describe said 24 650 497 / 899 264 997 / 16 336 — `.22`(e) moved the census and `.22`(f)
    re-derived the artifact, leaving this one sentence behind. ⛔ Every arm of
    `PARSE-COST-RATCHET` stayed GREEN over it, and not by luck: the gate compares the SHARE, and
    both count pairs round to the same `2.741 %`. A description nothing re-derives is prose, so
    this is now a projection of the artifact rather than a second copy of it.

    ⭐ No gate arm is added for the copy, because deriving it REMOVES the copy. The residual
    question — could the artifact be re-derived to different counts without `cost.md` following? —
    is closed by construction: the census is an exact function of the identity inputs, so
    different counts imply a moved input, which stales `cost.md`'s own identity table and forces
    the rebaseline that regenerates this line. (`.22`(c) is what makes that hold: a silently
    dropped file would change the counts with no input moving, and an undeclared drop now
    refuses.)
    """
    lr, total = art.get("lr_entries"), art.get("total_entries")
    files = art.get("files_measured")
    if not isinstance(lr, int) or not isinstance(total, int) or not isinstance(files, int):
        die(f"{FAMILY_SHARE_ARTIFACT} is missing the raw counts the report cites "
            f"(lr_entries/total_entries/files_measured). Re-derive it:\n"
            f"    make -C rust SHELL=/bin/bash sv_parse_cost_family_share")
    if total <= 0:
        die(f"{FAMILY_SHARE_ARTIFACT} records total_entries={total}: a share with no denominator "
            f"is not a measurement. Re-derive it.")
    # Thin-space grouping, matching every other large number this repository publishes. Applied
    # per NUMBER rather than to the finished sentence: a blanket `replace(",", " ")` would also
    # eat the prose commas, which is how a formatter quietly rewrites the text around it.
    def grouped(n: int) -> str:
        return f"{n:,}".replace(",", " ")

    return (f"{grouped(lr)} of {grouped(total)} entries over {grouped(files)} files, "
            f"`ENGINE-UNIVERSAL-SERVICES.21`; the previous 0.681 % counted only "
            f"`_lr_base`/`_lr_suffix`")

# ── the DECLARED-name population, and why it is gated rather than carried ────────────────────
#
# `ENGINE-UNIVERSAL-SERVICES.21` acceptance (g). The family share above is measured over corpus
# ENTRIES; this is the other population — the LR rule names the parser DECLARES, entered or not.
# Slice 1 published it as **128** and that was wrong by one: its own decomposition (97 matched +
# 24 `_lr_seed` + 6 `_lr_guard`) sums to 127, and three independent surfaces of the generated
# parser agree on 127 — the `RULE_NAMES` registry, the `fn parse_*` names, and the bare string
# literals, set-identical in all three.
#
# ⛔ It is a GATED constant, not a carried one (`docs/CLAIM_VERIFICATION.md` §5B): `--verify-families`
# re-derives it from `generated/` and REFUSES on drift. A number nothing re-derives goes stale
# silently, which is the defect this whole leaf is a record of.
SV_DECLARED_LR_RULES = 127
SV_DECLARED_LR_PROVENANCE = (
    "`generated/systemverilog_parser.rs` RULE_NAMES registry (1 608 entries), "
    "`ENGINE-UNIVERSAL-SERVICES.21` acceptance (g); slice 1 published 128"
)

GENERATED_DIR = "generated"
# ⛔ The blessed THROWAWAY slot (TOOLBOX.md 1.3). Its rule set is whatever probe grammar happens to
# be loaded, so including it would make this mode's verdict depend on the last thing somebody
# debugged. Excluded by name, with the reason stated, rather than silently filtered.
SCRATCH_ARTIFACT = "generated/scratch_parser.rs"

DEFAULT_PROBE = "rust/target/release/parseability_probe"
FALLBACK_PROBE = "rust/target/debug/parseability_probe"
CORPUS_MANIFEST = "stimuli/sv/characterization/durations.tsv"

PER_FILE_TIMEOUT_S = 120

# ── the LR-family classifier ────────────────────────────────────────────────────────────────
#
# ⛔⛔ DERIVED FROM THE EMISSION SITES, NOT FROM GREPPING TODAY'S GENERATED PARSER
# (`ENGINE-UNIVERSAL-SERVICES.21`). The first version of this classifier was written from the
# shape the leaf's prose described — `X := X_lr_base ( X_lr_suffix )*` — and it matched **97 of
# the 128** LR rule names the SV parser declares, missing **75.1 %** of the family's corpus
# entries (18 518 719 of 24 644 435). It counted no `_lr_seed` rule and, in a predicate whose
# own name says GUARDED, **not one `_lr_guard` rule**. Two eliminators emit these names, and
# between them there are EIGHT shapes:
#
#   rust/src/ast_pipeline/indirect_lr_elimination.rs   (the guarded/indirect pass)
#     :915   {base}_lr_base
#     :916   {base}_lr_suffix
#     :862   {base}_lr_suffix_r{index}
#     :1018  {base}_lr_seed_{rule}
#     :1188  {base}_lr_guard{variant}
#     :1190  {base}_lr_guard{variant}_suffix
#     :1241  {base}_lr_guard{variant}_{hop}
#   rust/src/ast_pipeline/mod.rs                       (the direct pass)
#     :3244  {rule}_lr_alt{n}
#     :3347  {rule}_lr_base
#     :3349  {rule}_lr_suffix
#
# ⛔ AND BOTH ALLOCATORS APPEND `_{index}` ON COLLISION (`indirect_lr_elimination.rs::allocate`,
# `mod.rs::allocate_synthetic_rule_name`), so an END-anchored pattern is wrong by construction:
# a single name collision would silently drop `X_lr_base_1` out of the family. The previous
# `_lr_base$` form had exactly that hole.
#
# ⇒ the predicate matches the TOKEN at a segment boundary rather than at end-of-string:
# `_lr_<token>` followed by anything that is not a lowercase letter (a digit, `_`, or the end).
#
# ⛔ Still not a substring test, and that half of the original reasoning was right and is kept:
# `_lr_base` is a PREFIX of `_lr_baseline`, so a bare substring match would absorb any future
# rule merely starting that way — a miss in the FLATTERING direction, inflating the family so
# the admission looks like it already owns cost it does not. `(?![a-z])` is what refuses it, and
# the controls below pin that case together with one positive per emitted shape.
#
# ⭐⭐ `_lr_alt` IS COVERED DELIBERATELY, AND ITS REACHABILITY IS NOW MEASURED RATHER THAN ASSUMED
# (`ENGINE-UNIVERSAL-SERVICES.21` acceptance (g)). Slice 1 covered it on the EMITTER's authority —
# the control string was typed from reading `mod.rs:3244`, and no parser has ever emitted one, so a
# mis-read of that `format!` would have reproduced as a passing control. Three scratch-slot probes
# settle it (`docs/tasks/artifacts/engine_universal_services/lr_alt_*.ebnf`):
#
#   A  a WELL-FORMED direct-LR grammar (`expr := expr "+" term | expr "-" term | term`) hoists two
#      `_lr_alt` rules, the planner consumes both, and `retract_consumed_normalization_rules`
#      (mod.rs:3139) deletes them ⇒ the generated parser declares `expr_lr_base`/`expr_lr_suffix`
#      and NO `_lr_alt` rule. So the shape is unreachable in a rule-entry dump by construction.
#   B  the one shape that leaves a hoist UNCONSUMED — one inline direct alternative plus one bare
#      reference to an indirect wrapper, which makes `base_alternatives` empty (mod.rs:3335) so the
#      planner returns None — is REFUSED by grammar well-formedness before codegen, and the refusal
#      NAMES the rule: `rule 'expr_lr_alt1' has no finite terminal derivation`. ⭐ That diagnostic is
#      the first real observation of the emitted name, from an oracle this instrument did not build.
#   C  with the only direct alternative at position 1 the engine emits `expr_lr_alt2`, which
#      separates "the suffix is the ALTERNATIVE's index+1" from "it is a running count of hoists".
#      The first is confirmed; the second is refuted.
#
# ⇒ two independent mechanisms keep `_lr_alt` out of every shipped parser today, and the predicate
# still matches it, because the predicate must describe the EMITTER rather than this month's
# grammar. That is the whole lesson of the defect — but the honest statement is that this arm is
# DEFENSIVE COVERAGE, not a live dump shape.
#
# ⛔ The shape token is a NAMED group. `--verify-families` reports a per-shape breakdown, and with a
# positional group that reporting silently depends on this pattern's internal layout — a coupling
# to the one thing this leaf is a record of somebody editing. Measured: a RED probe that swapped in
# slice 1's old pattern (whose group 1 is `(_r\d+)?`, optional) crashed the reporter on `None`
# instead of reporting the 30 names it fails to classify. A name makes the dependency explicit and
# lets a predicate without it degrade to `?` rather than abort.
LR_FAMILY_RE = re.compile(r"_lr_(?P<shape>base|suffix|seed|guard|alt)(?![a-z])")


def is_lr_family(rule: str) -> bool:
    """Does this rule name belong to a left-recursion-elimination family?"""
    return LR_FAMILY_RE.search(rule) is not None


def _self_check() -> None:
    """GROUND TRUTH, re-run on every invocation (microseconds).

    An instrument with no ground truth is a confident guess
    (docs/decisions/feedback_instrument_needs_ground_truth.md). Both the POSITIVE controls and
    the NEGATIVES are pinned, and a miss REFUSES rather than publishing a measurement.
    """
    # ⛔⛔ ONE POSITIVE PER EMITTED SHAPE, AND THE SHAPE LIST COMES FROM THE EMISSION SITES.
    # The previous suite had ten cases and every one of them was drawn from the `_lr_base` /
    # `_lr_suffix` half. It therefore PASSED while the classifier was blind to `_lr_seed`,
    # `_lr_guard` and `_lr_alt` — 75.1 % of the family's corpus entries. A control suite that
    # refuses on an unclassified NAME cannot help when the missing names were never imagined:
    # ⭐ the controls must enumerate the PRODUCER's shapes, not the consumer's expectations.
    cases = [
        # ── indirect_lr_elimination.rs, one per emission site ────────────────────────────────
        ("casting_type_lr_base", True),                          # :915
        ("casting_type_lr_suffix", True),                        # :916
        ("casting_type_lr_suffix_r0", True),                     # :862
        ("property_expr_lr_suffix_r10", True),                   # :862
        ("casting_type_lr_seed_constant_primary_sv_2017", True),  # :1018
        ("property_expr_lr_seed_prop_or_sv_2023", True),          # :1018
        ("casting_type_lr_guard1", True),                        # :1188
        ("casting_type_lr_guard0_suffix", True),                 # :1190
        ("casting_type_lr_guard0_constant_primary", True),       # :1241
        # ── ast_pipeline/mod.rs (the DIRECT pass) ────────────────────────────────────────────
        # ⭐ OBSERVED, not typed (`.21` acceptance (g)). Until slice 2 this arm was a single string
        # read off `mod.rs:3244` that no parser had ever emitted, so a mis-read of that `format!`
        # would have reproduced here as a passing control. Both names below were emitted by the
        # engine's own well-formedness diagnostic on the preserved probes
        # `docs/tasks/artifacts/engine_universal_services/lr_alt_survives_unconsumed.ebnf` (`_alt1`)
        # and `…/lr_alt_index_discriminator.ebnf` (`_alt2`, the alternative at position 1).
        ("expr_lr_alt1", True),                                  # :3244, observed (probe B)
        ("expr_lr_alt2", True),                                  # :3244, observed (probe C)
        ("incomplete_class_scoped_type_sv_2023_lr_base", True),  # :3347
        # ⛔ allocate()/allocate_synthetic_rule_name() append `_{index}` on a name COLLISION, so
        # an END-anchored pattern drops these silently. This is why the predicate anchors on a
        # segment boundary rather than on end-of-string.
        ("casting_type_lr_base_1", True),
        ("casting_type_lr_seed_constant_primary_2", True),
        # the rule the admission REPLACED is not itself in the family
        ("casting_type", False),
        # ⛔ the near-miss this classifier is anchored for — `_lr_base` prefixes `_lr_baseline`
        ("something_lr_baseline", False),
        ("something_lr_seedling", False),
        ("something_lr_guarded", False),
        ("something_lr_altitude", False),
        ("lr_base_helper", False),
        # an unrelated rule that merely contains the token
        ("suffix_rule", False),
        ("module_declaration", False),
    ]
    misses = [(r, want, is_lr_family(r)) for r, want in cases if is_lr_family(r) != want]
    if misses:
        for rule, want, got in misses:
            print(f"parse-cost: CONTROL MISSED: {rule!r} want={want} got={got}", file=sys.stderr)
        print("parse-cost: the LR-family classifier does not discriminate; refusing", file=sys.stderr)
        sys.exit(2)


def die(msg: str, code: int = 2) -> None:
    print(f"parse-cost: {msg}", file=sys.stderr)
    sys.exit(code)


def sha256_of(path: str) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def resolve_probe(explicit: str | None) -> str:
    """The parse binary. Either build is legitimate: entry counts are build-mode-independent
    (verified — the debug and release probes emit byte-identical dumps), so a tree with only a
    debug build can still run the BINDING metric. The advisory one says which build it used,
    because that number is not build-independent at all."""
    if explicit:
        p = explicit if os.path.isabs(explicit) else os.path.join(ROOT, explicit)
        if not os.access(p, os.X_OK):
            die(f"--probe {explicit} is not executable")
        return p
    for rel in (DEFAULT_PROBE, FALLBACK_PROBE):
        p = os.path.join(ROOT, rel)
        if os.access(p, os.X_OK):
            return p
    die("no parseability_probe found. Build one:\n"
        "  (cd rust && cargo build --release --features generated_parsers --bin parseability_probe)")
    raise AssertionError("unreachable")


def sub_corpus_of(rel: str) -> str:
    m = re.search(r"/subs/([^/]+)/", rel)
    if m:
        return m.group(1)
    if "/stimuli/sv/uvm/" in f"/{rel}":
        return "uvm-core"
    return "(unknown)"


# ── the NO-DUMP roster: a dropped file is DECLARED or it REFUSES (`.22` acceptance (c)) ─────
#
# ⛔ THE DEFECT THIS CLOSES IS SILENCE, NOT THE DROP. A corpus file the probe cannot dump is a
# file the measurement does not cover. Before `ENGINE-UNIVERSAL-SERVICES.22` every such row was
# folded into an integer, the integer was printed, and the run continued — so `.20` slice 1's
# *"zero no-dump rows"* and *"16 336/16 336 agree"* were published while ONE file was silently
# absent, and that one file measurably moved 5 of the 192 PINNED sample rows (`.21` slice 1
# RESULT 5). A count is not a roster: it cannot say WHICH file, WHY, or whether it is the same
# file as yesterday.
#
# ⭐ THE FIX IS THE `SV-CORPUS-DENOMINATOR` POSTURE, NOT A BIGGER TIMEOUT: publish the roster,
# and REFUSE on drift. A known, owned drop is declared in a tracked file with the leaf that owns
# fixing it; anything else stops the run. ⛔ Raising `PER_FILE_TIMEOUT_S` would convert a REFUSAL
# into a longer wait for the same wrong answer — the blow-up is exponential (measured ×4.14 per
# else-if arm), so no timeout is large enough and every timeout is arbitrary.

NODUMP_ROSTER = (
    "docs/tasks/artifacts/engine_universal_services/parse_cost_ratchet/corpus_nodump.tsv"
)


class NoDump:
    """One corpus file the probe produced no dump for, WITH the reason it did not."""

    __slots__ = ("path", "reason", "detail")

    def __init__(self, path: str, reason: str, detail: str) -> None:
        self.path, self.reason, self.detail = path, reason, detail

    def __repr__(self) -> str:  # pragma: no cover - diagnostics only
        return f"NoDump({self.path!r}, {self.reason!r})"


def read_nodump_roster() -> dict[str, str]:
    """The DECLARED no-dump files: `path -> reason`. Absent roster = an empty roster, which is
    strict rather than permissive — an undeclared drop then refuses."""
    path = os.path.join(ROOT, NODUMP_ROSTER)
    out: dict[str, str] = {}
    if not os.path.isfile(path):
        return out
    with open(path, encoding="utf-8") as fh:
        for n, line in enumerate(fh, 1):
            line = line.rstrip("\n")
            if not line.strip() or line.startswith("#"):
                continue
            parts = line.split("\t")
            # ⛔ The column header is DATA to a naive reader, and it was: the bidirectional check
            # duly reported that a file named `path` "dumps fine". Skipped explicitly rather than
            # by commenting it out in the file, because the next author will write the header
            # again and the reader is the only place that can be sure.
            if parts[0] == "path":
                continue
            if len(parts) < 3:
                die(f"{NODUMP_ROSTER}:{n}: expected at least 3 tab-separated fields "
                    f"(path, reason, owning-leaf), got {len(parts)}")
            out[parts[0]] = parts[1]
    return out


def adjudicate_nodump(nodump: list[NoDump], scope: str, full_corpus: bool = False) -> None:
    """REFUSE (exit 2) on any no-dump file the roster does not declare — and, over the FULL
    corpus, on any roster row that no longer describes a real drop.

    ⛔ Refuses rather than warning. A warning printed into a 16 336-file run's stderr is a count
    with extra characters — this whole leaf exists because that is what happened.

    ⭐⭐ THE CHECK IS BIDIRECTIONAL, AND THE SECOND DIRECTION WAS ADDED THE DAY THE FIRST ROW WENT
    STALE. `.22` slice 1 declared one file; slice 2 FIXED it, and the row would have sat there
    forever describing a defect that no longer exists — a permanent, invisible licence to drop that
    exact file if it ever regressed. That is `GATE-REACHABILITY`'s *"a register entry that no longer
    names an orphan fails too"* and `SV-CORPUS-DENOMINATOR`'s bidirectional reconcile, applied here.
    ⛔ Only over the FULL corpus: a scoped run (the pinned 192-file sample) legitimately does not
    touch most of the corpus, so a row it does not exercise is not evidence of anything.
    """
    roster = read_nodump_roster()
    undeclared = [n for n in nodump if n.path not in roster]
    for n in nodump:
        mark = "declared" if n.path in roster else "⛔ UNDECLARED"
        print(f"parse-cost: no-dump [{mark}] {n.path} — {n.reason}: {n.detail}", file=sys.stderr)
    if undeclared:
        die(f"{len(undeclared)} of {len(nodump)} no-dump file(s) in the {scope} are NOT declared "
            f"in {NODUMP_ROSTER}, so this measurement would silently omit them.\n"
            f"  Either fix the defect that stops the dump, or DECLARE the file with the leaf that "
            f"owns fixing it — one tab-separated row: path\\treason\\towning-leaf\\tnote.\n"
            f"  ⛔ Do NOT raise PER_FILE_TIMEOUT_S: `ENGINE-UNIVERSAL-SERVICES.22` measured the "
            f"blow-up as exponential (×4.14 per else-if arm), so no timeout is large enough.")
    if full_corpus:
        observed = {n.path for n in nodump}
        stale = sorted(p for p in roster if p not in observed)
        if stale:
            die(f"{len(stale)} row(s) in {NODUMP_ROSTER} name a file that DUMPS FINE in this "
                f"{scope}: {', '.join(stale)}.\n"
                f"  The defect they declare is gone. Delete the row — a stale declaration is a "
                f"standing licence for that file to be dropped again, silently, by a future "
                f"regression nobody would then have to justify.")


# ── measurement: the BINDING metric ─────────────────────────────────────────────────────────

def measure_one_entries(args: tuple[str, str]) -> tuple | NoDump:
    """One file -> its exact counter row. None when the probe produced no dump.

    ⭐ USES THE OUTCOME DUMP (TOOLBOX 3.5), NOT THE PLAIN ENTRY DUMP (3.4), AND THAT IS A
    MEASURED CHOICE. The outcome dump is a strict superset — raw entries, COMMITTED entries and
    memo hits — and it was measured to cost the SAME 0.04 s on the same file, so the extra two
    dimensions are free. They matter because `raw − committed` is the parse's FAILED-SPECULATION
    work, which is the mechanism a structural GUARD spends its cost through: measured on
    `ast_reg_pkg.sv`, 95.4 % of all rule entries are rolled back, and the guarded-admission
    family is 100 % rolled back (committed = 0). A ratchet on raw entries alone would be blind
    to a guard that doubles its probing and commits the same amount.
    ⚠️ Verified before being relied on, not assumed: the two dumps report an IDENTICAL
    `total_entries`, so this baseline and the census that selected its sample stay comparable.
    """
    probe, rel = args
    fd, tmp = tempfile.mkstemp(suffix=".json", dir=os.path.join(ROOT, "rust", "target"))
    os.close(fd)
    # ⛔ THE FAILURE REASON IS RETURNED, NOT DISCARDED (`ENGINE-UNIVERSAL-SERVICES.22` (c)).
    # Every one of these paths used to `return None`, so a 120 s TIMEOUT — the symptom of a real
    # engine defect — was indistinguishable from a missing file. The census then counted the row
    # into a `nodump` list, printed the count, and continued: that is how slice 1's *"zero no-dump
    # rows"* and *"16 336/16 336 agree"* became unreproducible with nothing failing.
    try:
        subprocess.run(
            [probe, "--parse", GRAMMAR, os.path.join(ROOT, rel), "--profile", PROFILE,
             "--dump-rule-outcome-counts-json", tmp],
            capture_output=True, timeout=PER_FILE_TIMEOUT_S,
        )
        if os.path.getsize(tmp) == 0:
            return NoDump(rel, "empty-dump",
                          "the probe exited without writing a dump (no timeout)")
        with open(tmp, encoding="utf-8") as fh:
            d = json.load(fh)
    except subprocess.TimeoutExpired:
        return NoDump(rel, "timeout",
                      f"the probe did not finish within {PER_FILE_TIMEOUT_S} s")
    except json.JSONDecodeError as exc:
        return NoDump(rel, "malformed-dump", f"the dump is not valid JSON: {exc}")
    except OSError as exc:
        return NoDump(rel, "os-error", str(exc))
    finally:
        try:
            os.unlink(tmp)
        except OSError:
            pass
    entries = d.get("rule_entry_counts", {})
    committed = d.get("rule_committed_counts", {})
    lr_e = sum(v for k, v in entries.items() if is_lr_family(k))
    lr_c = sum(v for k, v in committed.items() if is_lr_family(k))
    return (rel, "yes" if d.get("accepted") else "no",
            int(d.get("total_entries", 0)), int(d.get("total_committed", 0)),
            int(d.get("total_memo_hits", 0)), lr_e, lr_c)


def measure_entries(probe: str, files: list[str],
                    jobs: int) -> tuple[list[tuple], list[NoDump]]:
    """Entries for every file, in parallel.

    ⭐ Parallelism is SOUND here and is not for the advisory metric: an entry count is an exact
    property of (parser, input), so contention cannot move it. The wall-clock pass below is
    serial for exactly the opposite reason.
    """
    rows: list[tuple] = []
    nodump: list[NoDump] = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=jobs) as ex:
        for res in ex.map(measure_one_entries, [(probe, f) for f in files]):
            if isinstance(res, NoDump):
                nodump.append(res)
            else:
                rows.append(res)
    rows.sort(key=lambda r: (sub_corpus_of(r[0]), r[0]))
    return rows, sorted(nodump, key=lambda n: n.path)


# ── measurement: the ADVISORY metric ────────────────────────────────────────────────────────

def _bare_parse_ms(probe: str, path: str) -> float:
    t0 = time.perf_counter()
    subprocess.run([probe, "--parse", GRAMMAR, path, "--profile", PROFILE],
                   capture_output=True, timeout=PER_FILE_TIMEOUT_S)
    return (time.perf_counter() - t0) * 1000.0


def measure_process_floor(probe: str, repeats: int) -> float:
    """The per-invocation cost that is NOT parsing: fork + exec + the SV stdlib preload.

    ⛔ Re-measured every run, never a constant in this file. It is a property of the machine and
    the binary, and baking a measured-once number into the instrument is how a measurement stops
    describing the tree it runs in.
    """
    with tempfile.NamedTemporaryFile("w", suffix=".sv", delete=False,
                                     dir=os.path.join(ROOT, "rust", "target")) as fh:
        fh.write("module pgen_parse_cost_floor_probe; endmodule\n")
        tiny = fh.name
    try:
        return statistics.median([_bare_parse_ms(probe, tiny) for _ in range(repeats)])
    finally:
        try:
            os.unlink(tiny)
        except OSError:
            pass


def measure_wallclock(probe: str, files: list[str], repeats: int) -> dict:
    """Bare-parse wall clock over the advisory sample — SERIAL, median-of-K, floor-subtracted.

    ⛔ Serial on purpose. Under `-P 8` a file competes with seven siblings for the same cores,
    which is how a timing number becomes a statement about the machine instead of the parser —
    the confound that produced this leaf's wrong `~11 %`.
    """
    floor = measure_process_floor(probe, repeats)
    per_file = []
    for rel in files:
        med = statistics.median([_bare_parse_ms(probe, os.path.join(ROOT, rel))
                                 for _ in range(repeats)])
        per_file.append((rel, med, max(0.0, med - floor)))
    total_adj = sum(a for _, _, a in per_file)
    total_raw = sum(m for _, m, _ in per_file)
    return {
        "files": len(files),
        "repeats": repeats,
        "process_floor_ms": round(floor, 3),
        "total_raw_ms": round(total_raw, 1),
        "total_parse_ms": round(total_adj, 1),
        "floor_share_pct": round(100.0 * (total_raw - total_adj) / total_raw, 1) if total_raw else 0.0,
        "per_file": [{"path": r, "median_ms": round(m, 2), "parse_ms": round(a, 2)}
                     for r, m, a in per_file],
    }


# ── identity ────────────────────────────────────────────────────────────────────────────────

def sample_input_digest(files: list[str]) -> str:
    """One digest over the sample's INPUTS — the manifest order plus every file's bytes.

    ⛔ This is the half of the identity that a parser hash cannot cover. Entries are a function
    of (parser, grammar profile, INPUT). The corpora are git submodules, so a submodule bump can
    move the measurement without touching a single byte of PGEN — and the cheap identity leg of
    the gate would have reported a fresh baseline over a corpus that had changed underneath it.
    """
    h = hashlib.sha256()
    for rel in files:
        h.update(rel.encode("utf-8"))
        h.update(b"\0")
        with open(os.path.join(ROOT, rel), "rb") as fh:
            for chunk in iter(lambda: fh.read(1 << 20), b""):
                h.update(chunk)
        h.update(b"\0")
    return h.hexdigest()


def identity(files: list[str]) -> dict:
    ident = {}
    for label, rel in (("grammar", GRAMMAR_FILE), ("generated_parser", GENERATED_PARSER),
                       ("instrument", INSTRUMENT_FILE)):
        path = os.path.join(ROOT, rel)
        if not os.path.isfile(path):
            die(f"required input missing: {rel}\n"
                f"  generated/ is not tracked — regenerate it:\n"
                f"    make -C rust SHELL=/bin/bash regenerate_generated_parsers")
        ident[label] = {"path": rel, "sha256": sha256_of(path)}
    ident["sample_inputs"] = {"path": DEFAULT_MANIFEST, "sha256": sample_input_digest(files)}
    return ident


# ── the pinned sample ───────────────────────────────────────────────────────────────────────

def read_manifest(path: str) -> list[tuple[str, str]]:
    """The pinned sample: `tier \t repo-root-relative path`, comments and blanks skipped."""
    if not os.path.isfile(path):
        die(f"sample manifest missing: {os.path.relpath(path, ROOT)}")
    out = []
    with open(path, encoding="utf-8") as fh:
        for n, line in enumerate(fh, 1):
            line = line.rstrip("\n")
            if not line.strip() or line.startswith("#"):
                continue
            parts = line.split("\t")
            if len(parts) != 2:
                die(f"{os.path.relpath(path, ROOT)}:{n}: expected 2 tab-separated fields, got "
                    f"{len(parts)}")
            out.append((parts[0], parts[1]))
    if not out:
        die(f"{os.path.relpath(path, ROOT)} lists no files")
    return out


def select_sample(census_tsv: str, hot: int, lr: int, breadth: int) -> list[tuple[str, str]]:
    """Derive the pinned sample from a full-corpus census, DETERMINISTICALLY.

    ⛔ The derivation lives in this tracked instrument rather than in a shell one-liner, so the
    sample can be re-derived and audited instead of being a list somebody once produced. Three
    tiers, because one selection rule cannot serve both jobs the ratchet has:

      hot      the heaviest files by entries — where parse cost actually concentrates, so a
               regression in the hot path cannot hide behind thousands of trivial files;
      lr       the heaviest files by GUARDED-ADMISSION entries — the mechanism `.20` is about,
               so the ratchet is sensitive to the specific thing that regressed;
      breadth  a deterministic stride across everything else — so the ratchet is not blind to a
               regression that lands somewhere the first two tiers do not look.
    """
    rows = []
    with open(census_tsv, encoding="utf-8") as fh:
        for line in fh:
            parts = line.rstrip("\n").split("\t")
            if len(parts) != 5 or parts[2] not in ("True", "False"):
                continue
            rows.append((parts[1], int(parts[3]), int(parts[4])))
    if not rows:
        die(f"census {census_tsv} yielded no usable rows")
    chosen: dict[str, str] = {}
    for rel, _e, _l in sorted(rows, key=lambda r: (-r[1], r[0]))[:hot]:
        chosen[rel] = "hot"
    for rel, _e, lrv in sorted(rows, key=lambda r: (-r[2], r[0])):
        if len(chosen) >= hot + lr:
            break
        if lrv > 0 and rel not in chosen:
            chosen[rel] = "lr"
    # ⛔ BREADTH IS STRATIFIED PER SUB-CORPUS, NOT A FLAT STRIDE ACROSS THE WHOLE LIST. A flat
    # stride is proportional, so it silently drops the SMALL sub-corpora entirely: the first cut
    # covered 12 of the 14 vendored suites and missed `scr1` and `uvm-core` outright. Coverage
    # that depends on a suite being big enough to survive a stride is coverage by luck. Every
    # sub-corpus present therefore gets at least one row, and the remaining budget is filled by
    # a deterministic stride within each suite.
    rest: dict[str, list[str]] = {}
    for rel in sorted(r[0] for r in rows if r[0] not in chosen):
        rest.setdefault(sub_corpus_of(rel), []).append(rel)
    if breadth > 0 and rest:
        per = max(1, breadth // len(rest))
        for sub in sorted(rest):
            pool = rest[sub]
            stride = max(1, len(pool) // per)
            for rel in pool[::stride][:per]:
                chosen[rel] = "breadth"
    # ⛔ (tier, path) — the manifest's own column order. `chosen` is keyed by path, so returning
    # `.items()` directly would emit (path, tier) and every row would be swapped; the first cut
    # did exactly that and wrote a 200-row manifest whose tier column held file paths.
    order = {"hot": 0, "lr": 1, "breadth": 2}
    return sorted(((tier, rel) for rel, tier in chosen.items()),
                  key=lambda tr: (order[tr[0]], tr[1]))


# ── reporting ───────────────────────────────────────────────────────────────────────────────

def write_entries_tsv(path: str, rows: list[tuple], tiers: dict[str, str]) -> None:
    """⛔ FAILED SPECULATION IS NOT A COLUMN. It is exactly `entries − committed`, and a field a
    reader can derive exactly is looked up, never stored (docs/DERIVED_STATE_CONTAINMENT.md
    R1/R3) — a stored copy is one more thing to drift out of agreement with its own inputs."""
    with open(path, "w", encoding="utf-8") as fh:
        fh.write("sub_corpus\ttier\tpath\taccepted\tentries\tcommitted\tmemo_hits"
                 "\tlr_entries\tlr_committed\n")
        for rel, accepted, ent, com, memo, lr_e, lr_c in rows:
            fh.write(f"{sub_corpus_of(rel)}\t{tiers.get(rel, '?')}\t{rel}\t{accepted}"
                     f"\t{ent}\t{com}\t{memo}\t{lr_e}\t{lr_c}\n")


def write_report(path: str, rows: list[tuple], ident: dict, nodump: list[str],
                 tiers: dict[str, str]) -> None:
    # ⛔ The corpus-wide family figures are PROJECTED from the tracked derivation, not restated.
    # This report measures the 192-file SAMPLE and cannot derive a corpus number; before `.26` it
    # carried one as a string, and that string was stale (see `format_family_provenance`). The
    # reader refuses on a missing or wrong-schema artifact rather than writing a report that
    # cites numbers it did not obtain.
    family_share = read_family_share_artifact(FAMILY_SHARE_ARTIFACT)
    total = sum(r[2] for r in rows)
    committed = sum(r[3] for r in rows)
    memo = sum(r[4] for r in rows)
    lr_total = sum(r[5] for r in rows)
    lr_committed = sum(r[6] for r in rows)
    accepted = sum(1 for r in rows if r[1] == "yes")
    acc_entries = sum(r[2] for r in rows if r[1] == "yes")
    acc_committed = sum(r[3] for r in rows if r[1] == "yes")
    by_tier: dict[str, list[int]] = {}
    for row in rows:
        by_tier.setdefault(tiers.get(row[0], "?"), []).append(row[2])

    L = []
    A = L.append
    A("# SV Parse-Cost Ratchet — the BINDING baseline")
    A("")
    A("`ENGINE-UNIVERSAL-SERVICES.20` acceptance (d). Produced by")
    A("`stimuli/sv/corpus_parse_cost.py`; held by `scripts/check_parse_cost_ratchet.sh`.")
    A("")
    A("> ⛔ **This file is byte-compared by the gate.** It carries only values that are exact")
    A("> functions of the tree — no wall clock, no timestamp, no `HEAD`. That is not tidiness:")
    A("> a baseline whose bytes move on their own cannot be diffed, and a baseline nobody")
    A("> diffs is the staleness defect this leaf's own `.13i` is a record of.")
    A("")
    A("## What binds, and what it cannot see")
    A("")
    A("| metric | graph observed | machine-dependent | role |")
    A("|---|---|---|---|")
    A("| total rule entries | PROTOCOL (`parse_*`) | no | **BINDING** |")
    A("| bare-parse wall clock | FUSED (`cascade_*`) | yes | advisory, wide band |")
    A("")
    A("⛔ **The binding metric's blind spot, declared rather than discovered.** Requesting the")
    A("entry dump routes the parse to the PROTOCOL graph (TOOLBOX 3.4 ROUTING). A production")
    A("parse with no diagnostic consumer runs the FUSED `cascade_*` graph, which ticks no")
    A("per-rule counters — including the fused twins of the very rules this leaf is about")
    A("(`cascade_match_casting_type_lr_suffix`, `cascade_build_select_expression_lr_base`, …).")
    A("The +24.3 % that opened `.20` was measured on the fused graph. So this number guards")
    A("**structural work** and cannot, on its own, describe that cost. The wall-clock advisory")
    A("in `advisory.json` is the only view of the other graph, and it is machine-dependent —")
    A("which is why it advises and does not bind. **Neither metric alone is sufficient.**")
    A("")
    A("## Instrument identity (what produced this number)")
    A("")
    A("> Re-hash these four inputs. If any differs, **this baseline no longer describes your")
    A("> tree** and the honest act is to re-measure, not to quote. The gate re-hashes them on")
    A("> every run — unlike the six oracles `SV-CORPUS-GRAD.13i` found carrying an identity")
    A("> block that nothing read, four of which were measurably stale.")
    A("")
    A("⛔ **`instrument` is the fourth input, and it was added because its absence was a real")
    A("hole** (`ENGINE-UNIVERSAL-SERVICES.21`). The first block named three, on the sound")
    A("argument that the BINDING counters are an exact function of exactly those. Sound, and")
    A("insufficient: the family columns below are functions of the instrument's own classifier,")
    A("so correcting that classifier staled every published family number while the identity")
    A("tier still reported `fresh`.")
    A("")
    A("| input | repo-root-relative path | sha256 |")
    A("|---|---|---|")
    for label in ("grammar", "generated_parser", "instrument", "sample_inputs"):
        e = ident[label]
        A(f"| {label.replace('_', ' ')} | `{e['path']}` | `{e['sha256']}` |")
    A("")
    A("`sample inputs` digests the manifest ORDER plus every sampled file's bytes: the corpora")
    A("are git submodules, so a bump can move this measurement without touching one byte of")
    A("PGEN, and a parser hash alone would not notice.")
    A("")
    A("## The binding numbers")
    A("")
    A("Four exact integers. Each was verified deterministic across repeated release runs AND")
    A("byte-identical between the debug and release probes before being made binding.")
    A("")
    A("| quantity | value | what a rise means |")
    A("|---|---:|---|")
    A(f"| sample files measured | {len(rows)} | — |")
    A(f"| accepted / rejected | {accepted} / {len(rows) - accepted} | a correctness move, not a cost move |")
    A(f"| **rule entries** | **{total:,}** | more rule-method entries: structural work grew |")
    A(f"| **committed entries** | **{committed:,}** | more surviving work |")
    A(f"| **failed speculation** (`entries − committed`) | **{total - committed:,}** | more probing waste — the mechanism a GUARD spends through |")
    A(f"| **memo hits** | **{memo:,}** | memo behaviour moved |")
    A("")
    A(f"Failed speculation is **{100.0 * (total - committed) / total:.1f} %** of all rule entries in")
    A("this sample: the parse is overwhelmingly probing work, so a guard that probes more shows")
    A("up here long before it shows up in the raw entry count.")
    A("")
    A("⚠️ **`committed` is only meaningful for an ACCEPTED parse** (TOOLBOX 3.5) — a rejected parse")
    A("commits nothing durable, so its entries are ALL speculation by construction and it drags")
    A("the whole-sample ratio up. The accepted-only sub-total is published beside it so neither is")
    A(f"mistaken for the other: over the {accepted} accepted files, entries {acc_entries:,} and")
    A(f"committed {acc_committed:,} — {100.0 * (acc_entries - acc_committed) / acc_entries:.1f} %")
    A("failed speculation even where the parse succeeded.")
    A("")
    A("## The left-recursion-elimination family")
    A("")
    A("⛔ **This section counted a QUARTER of its own subject until `ENGINE-UNIVERSAL-SERVICES.21`.**")
    A("The classifier was written from the shape the prose described (`X_lr_base ( X_lr_suffix )*`)")
    A(f"and matched 97 of the {SV_DECLARED_LR_RULES} LR rule names the parser declares — no "
      "`_lr_seed`, and in a")
    A("heading that said GUARDED, not one `_lr_guard` rule. It is now derived from the two")
    A("eliminators' emission sites; see the classifier's own comment for the eight shapes.")
    A("")
    A("| quantity | value |")
    A("|---|---:|")
    A(f"| family entries (`_lr_base`/`_lr_suffix`/`_lr_seed`/`_lr_guard`/`_lr_alt`) | {lr_total:,} |")
    A(f"| of those, committed | {lr_committed:,} |")
    A(f"| family share of all entries in this sample | {100.0 * lr_total / total:.3f} % |")
    A("")
    A("⭐ Published so `.20` acceptance (b)'s third A/B arm computes its delta straight off this")
    A("artifact instead of re-deriving it.")
    A("")
    A(f"⭐ The family commits **{lr_committed:,}** of its {lr_total:,} entries — "
      f"**{100.0 * lr_committed / lr_total:.3f} %**. The elimination machinery is, to three")
    A("significant figures, **pure speculation**: it is entered, it probes, it rolls back. That is")
    A("the expected shape of a structural guard and it is stated here so a later reader does not")
    A("mistake the family's entry count for productive work.")
    A("")
    A("⛔⛔ **AND IT CARRIES A FINDING THAT BOUNDS THIS WHOLE INSTRUMENT.** Across the full")
    A(f"corpus the family takes **{CORPUS_FAMILY_SHARE_PCT} %** of all rule entries")
    A(f"({format_family_provenance(family_share)}).")
    A("")
    A("⛔ **What this metric cannot see, stated as a property rather than as a number.** These")
    A("counters tick only in the PROTOCOL graph, and a counter counts EVENTS — a rise in the cost")
    A("PER event is invisible to it on any graph. So the binding numbers guard **structural work**")
    A("exactly and price nothing. That limit is a property of the metric, so it cannot go stale.")
    A("")
    A("⭐ **On the one change this ratchet was built for, the counters were not the blind half.**")
    A(f"The guarded admission moved them **+{FLIP_ENTRY_DELTA_PCT} %** — "
      f"**3.49× larger** than the whole")
    A("family's own entry count, and far outside any band a ratchet could hide. The wall clock,")
    A("meanwhile, produced no admissible figure for the same change at all.")
    A(f"({FLIP_ENTRY_DELTA_PROVENANCE}.)")
    A("")
    A("⛔⛔ **This section published a sensitivity bound of `~8.9×`, and BOTH terms were wrong.**")
    A(f"Retired by `ENGINE-UNIVERSAL-SERVICES.26`. It was `24.3 / {CORPUS_FAMILY_SHARE_PCT}`. The")
    A("numerator, a `+24.3 %` wall-clock regression, is REFUTED — not reproducible from the raw")
    A("data of the runs that produced it (`.20` slice 5). The denominator was the wrong quantity")
    A("independently of that: sensitivity is how much the counter MOVED, not how large the family")
    A("is, and the bound's *\"the flip's entry delta is strictly smaller\"* was an INFERENCE that a")
    A("tracked artifact in its own leaf had already refuted. ⇒ **the factor is retired, not")
    A("re-computed**: it fails under every available reading (0.41× on the point estimate, 1.82× on")
    A("the most adversarial pairing), and no admissible wall-clock figure exists to rebuild it")
    A("from. `.20` acceptance (a)'s profile is what attributes fused-graph cost; this ratchet stops")
    A("structural work growing unwatched meanwhile.")
    A("")
    # ⛔ THE CO-PUBLICATION ANCHOR (`.21` acceptance (f)). `PARSE-COST-RATCHET` holds every
    # designated live surface equal to the tracked derivation, so this bound cannot be quoted
    # anywhere in the repository after it has gone stale. The marker is what separates a LIVE
    # claim from the era-dated citation in the very next paragraph.
    A(f"**Live LR-family share `{CORPUS_FAMILY_SHARE_PCT}`** (corpus-entry share %), derived by")
    A("`python3 stimuli/sv/corpus_parse_cost.py --rederive-family-share` into")
    A(f"`{FAMILY_SHARE_ARTIFACT}` and re-hashed against its four recorded inputs on every run.")
    A("")
    A("⚠️ This anchor was the PAIR `2.741/8.9` until `.26` retired the second element. The share")
    A("itself is unaffected — it is a correctly measured quantity, and the two eras of it are on")
    A("record: it read **0.681 %** until `.21`, computed by a classifier that saw 97 of the")
    A("parser's 127 LR rule names. What `.26` removed is the ratio built on top of it.")
    A("")
    A("## Per tier")
    A("")
    A("| tier | files | entries | what it is for |")
    A("|---|---:|---:|---|")
    why = {
        "hot": "the heaviest files — where parse cost concentrates",
        "lr": "the heaviest guarded-admission files — the mechanism `.20` owns",
        "breadth": "a deterministic stride across the rest — so the ratchet is not blind elsewhere",
    }
    for tier in ("hot", "lr", "breadth"):
        if tier in by_tier:
            A(f"| `{tier}` | {len(by_tier[tier])} | {sum(by_tier[tier]):,} | {why[tier]} |")
    A("")
    if nodump:
        A("## ⚠️ Files that produced NO dump")
        A("")
        A("Reported, never dropped: a file the probe could not dump is a file this baseline does")
        A("not cover, and silently shrinking the sample is how a ratchet loosens itself.")
        A(f"⛔ Since `ENGINE-UNIVERSAL-SERVICES.22` each row also carries WHY, and an undeclared")
        A(f"drop REFUSES the run (roster: `{NODUMP_ROSTER}`).")
        A("")
        A("| file | reason | detail |")
        A("|---|---|---|")
        for n in nodump:
            A(f"| `{n.path}` | `{n.reason}` | {n.detail} |")
        A("")
    A("_Per-file rows: `entries.tsv` — 9 columns (sub-corpus, tier, path, accepted, entries,")
    A("committed, memo_hits, lr_entries, lr_committed), sorted by (sub-corpus, path) so two runs")
    A("diff directly. Failed speculation is deliberately NOT a column: it is exactly")
    A("`entries − committed` and is derived, never stored._")
    with open(path, "w", encoding="utf-8") as fh:
        fh.write("\n".join(L) + "\n")


# ── the DECLARED-name cross-family check (`.21` acceptance (g)) ─────────────────────────────
#
# ⛔ WHY THIS EXISTS. The classifier above is derived from the two ELIMINATORS, which are
# engine-universal — nothing in them is SystemVerilog-specific. So a mis-derivation would mis-price
# every family whose knot is guard-feasible, not just this one, and slice 1 checked exactly one
# family. This mode runs the predicate over every generated parser's own rule registry and REFUSES
# if any declared name containing `_lr` is left unclassified.
#
# ⭐ The roster is DERIVED from the artifacts (every `generated/*.rs` that declares a `RULE_NAMES`
# registry IS a generated parser), never hand-listed: a hand list is one more copy to drift, and a
# new family would silently escape the check the day it lands.

RULE_NAMES_DECL = "const RULE_NAMES"
# ⛔ Anchored on the FULL declaration, because the real corpus contains two near-misses that a
# loose `RULE_COUNT` match absorbs: `const RULE_COUNTED_QUANTIFIER: RuleId` (a prefix collision, 3
# of them in the regex parser) and `const THIN_RULE_COUNT: usize` (a suffix collision). Both are
# pinned as negative controls below — measured from the artifacts, not imagined.
RULE_COUNT_RE = re.compile(r"^\s*const RULE_COUNT: usize = (\d+)usize;")


def read_rule_registry(path: str) -> tuple[list[str], int]:
    """A generated parser's own canonical rule roster: `(RULE_NAMES, RULE_COUNT)`.

    ⛔ The registry is read rather than the `fn parse_*` names, because it is the parser's own
    self-declaration and it ships its own length. That length is the reader's ground truth: a
    truncated scan produces fewer names than `RULE_COUNT` and REFUSES, so this function cannot
    quietly return a short list — which is the failure mode that would make an unclassified name
    look like an absent one.
    """
    names: list[str] = []
    declared: int | None = None
    capturing = False
    buf: list[str] = []
    with open(path, encoding="utf-8", errors="replace") as fh:
        for line in fh:
            if declared is None:
                m = RULE_COUNT_RE.match(line)
                if m:
                    declared = int(m.group(1))
            if not capturing and RULE_NAMES_DECL in line and "&[" in line:
                capturing = True
                line = line.split("&[", 1)[1]
            if capturing:
                buf.append(line)
                if "];" in line:
                    capturing = False
                    if declared is not None:
                        break
    if not buf:
        die(f"{os.path.relpath(path, ROOT)} declares no `RULE_NAMES` registry — this reader has "
            f"outlived the codegen's shape; fix the reader rather than reporting an empty roster")
    if declared is None:
        die(f"{os.path.relpath(path, ROOT)} declares `RULE_NAMES` but no `RULE_COUNT` — the "
            f"reader has no length to check itself against")
    names = re.findall(r'"([^"]+)"', "".join(buf).split("];")[0])
    if len(names) != declared:
        die(f"{os.path.relpath(path, ROOT)}: read {len(names)} rule names but the parser declares "
            f"RULE_COUNT = {declared}. The scan is truncated or the shape moved; refusing rather "
            f"than classifying a partial roster")
    return names, declared


def _self_check_registry_reader() -> None:
    """GROUND TRUTH for the registry reader, run on every invocation. Positives, the two real
    near-miss negatives, and the length check proven to REFUSE."""
    import io

    def read_text(text: str) -> tuple[list[str], int]:
        path = os.path.join(ROOT, "rust", "target", "_parse_cost_registry_control.rs")
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with io.open(path, "w", encoding="utf-8") as fh:
            fh.write(text)
        try:
            return read_rule_registry(path)
        finally:
            try:
                os.unlink(path)
            except OSError:
                pass

    good = ('    const RULE_COUNT: usize = 2usize;\n'
            '    const RULE_NAMES: &\'static [&\'static str] = &["a", "b_lr_base"];\n')
    names, declared = read_text(good)
    ok = names == ["a", "b_lr_base"] and declared == 2
    # the two collisions that really occur in `generated/` must NOT be read as the declaration
    near_miss = ('    const RULE_COUNTED_QUANTIFIER: RuleId = 20u16;\n'
                 '    const THIN_RULE_COUNT: usize = 28usize;\n'
                 '    const RULE_COUNT: usize = 1usize;\n'
                 '    const RULE_NAMES: &\'static [&\'static str] = &["only"];\n')
    names2, declared2 = read_text(near_miss)
    ok = ok and names2 == ["only"] and declared2 == 1
    if not ok:
        print("parse-cost: CONTROL MISSED: the RULE_NAMES/RULE_COUNT reader does not discriminate; "
              f"refusing (got {names}/{declared} and {names2}/{declared2})", file=sys.stderr)
        sys.exit(2)


def family_artifacts() -> list[str]:
    """Every generated parser, derived from the artifacts themselves and sorted for determinism."""
    d = os.path.join(ROOT, GENERATED_DIR)
    if not os.path.isdir(d):
        die(f"{GENERATED_DIR}/ is absent — it is not tracked; regenerate it:\n"
            f"    make -C rust SHELL=/bin/bash regenerate_generated_parsers")
    out = []
    for name in sorted(os.listdir(d)):
        if not name.endswith(".rs"):
            continue
        rel = f"{GENERATED_DIR}/{name}"
        if rel == SCRATCH_ARTIFACT:
            continue
        with open(os.path.join(d, name), encoding="utf-8", errors="replace") as fh:
            head = fh.read(1 << 22)
        if RULE_NAMES_DECL in head:
            out.append(rel)
    if not out:
        die(f"no generated parser under {GENERATED_DIR}/ declares a RULE_NAMES registry")
    return out


def run_verify_families() -> int:
    """`.21` (g): the predicate, over every generated parser's DECLARED rule names."""
    _self_check_registry_reader()
    artifacts = family_artifacts()
    unclassified: list[tuple[str, str]] = []
    rows = []
    for rel in artifacts:
        names, declared = read_rule_registry(os.path.join(ROOT, rel))
        # ⛔ The population is every name CONTAINING `_lr`, not every name the predicate matches.
        # Scoping it to what the predicate already accepts would make the check pass by
        # construction — the exact shape of the ten controls that could only confirm themselves.
        lr_names = [n for n in names if "_lr" in n]
        missed = [n for n in lr_names if not is_lr_family(n)]
        unclassified.extend((rel, n) for n in missed)
        shapes: dict[str, int] = {}
        for n in lr_names:
            m = LR_FAMILY_RE.search(n)
            if m:
                shape = m.groupdict().get("shape") or "?"
                shapes[shape] = shapes.get(shape, 0) + 1
        rows.append((rel, declared, len(lr_names), len(lr_names) - len(missed), shapes))

    width = max(len(r[0]) for r in rows)
    print(f"{'generated parser':<{width}}  {'rules':>6} {'_lr':>5} {'classified':>10}  shapes")
    for rel, declared, lr_n, cls, shapes in rows:
        shape_s = " ".join(f"{k}={v}" for k, v in sorted(shapes.items())) or "—"
        print(f"{rel:<{width}}  {declared:>6} {lr_n:>5} {cls:>10}  {shape_s}")
    total_lr = sum(r[2] for r in rows)
    print(f"\n{len(rows)} generated parsers, {total_lr} declared LR rule names, "
          f"{total_lr - len(unclassified)} classified.")

    if unclassified:
        for rel, n in unclassified:
            print(f"parse-cost: UNCLASSIFIED LR RULE NAME: {n!r} in {rel}", file=sys.stderr)
        print(f"parse-cost: {len(unclassified)} declared rule name(s) contain `_lr` but no emission "
              f"shape claims them. The classifier has fallen behind an eliminator — re-derive it "
              f"from the emission sites, never from this list.", file=sys.stderr)
        return 1

    # ⛔ The SV declared count is GATED, not carried (`docs/CLAIM_VERIFICATION.md` §5B): the number
    # is published in four surfaces and slice 1's copy was already wrong by one.
    sv = next((r for r in rows if r[0] == GENERATED_PARSER), None)
    if sv is None:
        print(f"parse-cost: NOT EVALUATED — {GENERATED_PARSER} is absent, so the pinned declared-LR "
              f"count could not be re-derived", file=sys.stderr)
    elif sv[2] != SV_DECLARED_LR_RULES:
        print(f"parse-cost: the SystemVerilog parser now declares {sv[2]} LR rule names, but "
              f"SV_DECLARED_LR_RULES pins {SV_DECLARED_LR_RULES} ({SV_DECLARED_LR_PROVENANCE}). "
              f"Re-derive the published figure and update the constant deliberately.",
              file=sys.stderr)
        return 1
    else:
        print(f"parse-cost: SV declared LR rule names = {sv[2]}, matching the pinned constant.")
    return 0


def corpus_files() -> tuple[list[str], int]:
    """The corpus the census walks: (files present on disk, files the manifest names).

    ⛔ Both numbers are returned rather than just the first. The vendored corpora are git
    SUBMODULES, so "present" and "named" diverge on a checkout without them — and silently
    measuring the subset that happens to be there is how a shrinking denominator reads as a
    clean run (`docs/decisions/feedback_a_check_that_cannot_run_must_say_so.md`).
    """
    manifest = os.path.join(ROOT, CORPUS_MANIFEST)
    if not os.path.isfile(manifest):
        die(f"corpus manifest missing: {CORPUS_MANIFEST}")
    listed = sorted({l.split("\t")[2] for l in open(manifest, encoding="utf-8") if l.strip()})
    present = [f for f in listed if os.path.isfile(os.path.join(ROOT, f))]
    return present, len(listed)


# ── the family share: derived by one mode, gated by another (`.21` acceptance (f)) ───────────
#
# ⛔ WHY TWO MODES AND NOT ONE. Re-deriving costs a full-corpus census (~70 s measured), which is
# too slow to run on every commit and therefore exactly the kind of check that gets scheduled and
# then never runs — `GATE-REACHABILITY`'s founding sentence. So the expensive derivation writes a
# TRACKED artifact naming every input it consumed, and a cheap tier re-hashes those inputs. If
# none of them moved, the recorded share CANNOT have moved: that tier is a proof, not a sample.
# It is the same two-tier argument `PARSE-COST-RATCHET` already makes for the binding counters,
# reused deliberately rather than invented again.

FAMILY_SHARE_IDENTITY_LABELS = ("grammar", "generated parser", "classifier", "corpus inputs")


def classifier_digest() -> str:
    """The identity of the CLASSIFIER, as its exact pattern string.

    ⛔⛔ DELIBERATELY NARROWER THAN A HASH OF THIS FILE, AND THE TRADE IS STATED RATHER THAN
    DISCOVERED LATER. The share is a function of (grammar, parser, corpus, predicate). Hashing the
    whole instrument would also fire on a comment edit, and a gate that fires on prose is a gate
    that teaches waivers — the failure `GENERATED-LINT-CORRECTNESS.6`/`.12` are records of, and
    the reason the third addition to that checker was refused on measurement.
    ⚠️ HONEST LIMIT: this pins the PREDICATE, not the census pipeline around it. A change to how
    entries are counted (`measure_one_entries`'s dump flag, the summation) would not stale this
    artifact. It is not unguarded — `PARSE-COST-RATCHET`'s own baseline carries a whole-file
    `instrument` hash, so a pipeline edit stales THAT and puts the operator in the re-measure path
    already. The two identity blocks are complements, and neither alone covers the file.
    """
    return hashlib.sha256(LR_FAMILY_RE.pattern.encode("utf-8")).hexdigest()


def family_share_identity() -> tuple[dict, list[str]]:
    """(the identity rows that could be computed, the reasons the others could not).

    Never silently partial: a missing input is RETURNED as a reason, so the caller can report NOT
    EVALUATED rather than pass on a check it did not perform."""
    ident, missing = {}, []
    for label, rel in (("grammar", GRAMMAR_FILE), ("generated parser", GENERATED_PARSER)):
        path = os.path.join(ROOT, rel)
        if os.path.isfile(path):
            ident[label] = {"path": rel, "sha256": sha256_of(path)}
        else:
            missing.append(f"{rel} is absent — generated/ is not tracked; regenerate it with "
                           f"`make -C rust SHELL=/bin/bash regenerate_generated_parsers`"
                           if rel == GENERATED_PARSER else f"{rel} is absent")
    ident["classifier"] = {"path": f"{INSTRUMENT_FILE}:LR_FAMILY_RE",
                           "sha256": classifier_digest()}
    present, listed = corpus_files()
    if present and len(present) == listed:
        ident["corpus inputs"] = {"path": CORPUS_MANIFEST, "sha256": sample_input_digest(present)}
    else:
        missing.append(f"the vendored SV corpora are git submodules and only {len(present)} of "
                       f"{listed} manifest-named files are checked out here")
    return ident, missing


def read_family_share_artifact(rel: str) -> dict:
    path = rel if os.path.isabs(rel) else os.path.join(ROOT, rel)
    if not os.path.isfile(path):
        die(f"{rel} is missing — the corpus family share has no tracked derivation to be checked "
            f"against. Re-derive it:\n"
            f"    make -C rust SHELL=/bin/bash sv_parse_cost_family_share")
    try:
        with open(path, encoding="utf-8") as fh:
            art = json.load(fh)
    except (OSError, json.JSONDecodeError) as exc:
        die(f"{rel} is unreadable: {exc}")
    if art.get("schema") != FAMILY_SHARE_SCHEMA:
        die(f"{rel} declares schema {art.get('schema')!r}, expected {FAMILY_SHARE_SCHEMA!r}. Fix "
            f"this reader rather than trusting fields it may not have written.")
    return art


def _self_check_family_share() -> None:
    """GROUND TRUTH for the share arithmetic and its projection, every invocation.

    ⛔ The controls that matter here are the RED ones. A verifier that only ever agrees with
    itself is what `.21` is a record of: ten cases drawn from the same prose as the classifier
    they tested. Each case below is a specific way this gate could be wrong in the PASSING
    direction, and each must be caught.

    ⛔ Until `.26` these controls exercised `blind_spot_factor()`, a derivation that is now
    RETIRED. They are not deleted along with it — they are re-pointed at what the instrument
    still computes: the share the artifact's own raw counts imply, and the projection of those
    counts into the report. Losing the arms with the arithmetic would have left the replacement
    unguarded, which is the shape of defect this leaf exists to stop repeating.
    """
    misses: list[str] = []

    # (lr_entries, total_entries) -> the share those counts imply, to the published precision.
    # ⭐ Case 2 is the LIVE artifact's own counts and case 1 is the counts the report USED to
    # carry: both round to 2.741, which is exactly why the stale sentence was invisible to a gate
    # that compares the share. Recorded as a control so the reason stays visible.
    share_cases = [
        ((24_644_435, 899_064_022), "2.741"),   # the pre-`.22`(e) counts the report carried
        ((24_650_497, 899_264_997), "2.741"),   # the live artifact's counts
        ((681, 100_000), "0.681"),              # the pre-`.21` era reproduces at this precision
        ((1, 1), "100.000"),                    # a family that is the whole corpus
        ((0, 1_000), "0.000"),                  # and an empty family is 0, never a crash
    ]
    for (lr, total), want in share_cases:
        got = f"{100.0 * lr / total:.3f}"
        if got != want:
            misses.append(f"share({lr}/{total}) want={want} got={got}")

    # The projection into the report must read the ARTIFACT, and must refuse rather than print a
    # placeholder when a count it cites is absent — the failure mode that let a stale sentence
    # survive four sessions was a projection that never looked.
    projected = format_family_provenance(
        {"lr_entries": 24_650_497, "total_entries": 899_264_997, "files_measured": 16_336})
    for token in ("24 650 497", "899 264 997", "16 336"):
        if token not in projected:
            misses.append(f"provenance projection dropped {token!r}: {projected!r}")
    if "," not in projected:
        misses.append(f"provenance projection ate its prose commas: {projected!r}")

    if misses:
        for m in misses:
            print(f"parse-cost: CONTROL MISSED: {m}", file=sys.stderr)
        print("parse-cost: the family-share arithmetic/projection does not reproduce its own "
              "ground truth; refusing", file=sys.stderr)
        sys.exit(2)


def run_verify_family_share(artifact: str = FAMILY_SHARE_ARTIFACT) -> int:
    """`.21` (f) TIER 1 (~0.8 s, every run): is the carried share still describing this tree?

    `artifact` is a parameter so every refusal below can be driven RED against a mutated COPY
    (`docs/tasks/artifacts/engine_universal_services/family_share_gate/probe.sh`) without ever
    touching the tracked one. A control never observed failing is not known to work.
    """
    _self_check_family_share()
    art = read_family_share_artifact(artifact)

    failures: list[str] = []
    # ── leg 1: the CARRIED constant must equal the DERIVED artifact ──────────────────────────
    # This is the whole point of (f). Before it, `CORPUS_FAMILY_SHARE_PCT` was referenced only by
    # the file that defines it, so editing it to any value at all was a silent, unopposed act.
    if str(art.get("corpus_family_share_pct")) != CORPUS_FAMILY_SHARE_PCT:
        failures.append(
            f"the carried CORPUS_FAMILY_SHARE_PCT = {CORPUS_FAMILY_SHARE_PCT} % does NOT match the "
            f"derived {artifact} value {art.get('corpus_family_share_pct')} %. "
            f"One of the two is stale; the artifact is the measurement, so adopt it deliberately "
            f"or re-derive:\n"
            f"        make -C rust SHELL=/bin/bash sv_parse_cost_family_share")
    # ⛔ NEW IN `.26`, replacing the retired blind-spot-factor leg: the artifact's own RAW COUNTS
    # must reproduce the share it declares. Before this, `corpus_family_share_pct` was the only
    # field anything compared, so `lr_entries`/`total_entries` could disagree with it — or with
    # each other across a re-derivation — and every arm stayed green. The report now CITES those
    # counts, so they are a published claim and are checked like one.
    if str(art.get("corpus_family_share_pct")) == CORPUS_FAMILY_SHARE_PCT:
        lr, total = art.get("lr_entries"), art.get("total_entries")
        if not isinstance(lr, int) or not isinstance(total, int) or total <= 0:
            failures.append(
                f"{artifact} declares a share of {CORPUS_FAMILY_SHARE_PCT} % but carries no usable "
                f"raw counts (lr_entries={lr!r}, total_entries={total!r}). A share with no "
                f"numerator and denominator cannot be re-derived by a reader.")
        else:
            implied = f"{100.0 * lr / total:.3f}"
            if implied != CORPUS_FAMILY_SHARE_PCT:
                failures.append(
                    f"{artifact} records {lr:,} / {total:,} entries, which is {implied} % — not "
                    f"the {CORPUS_FAMILY_SHARE_PCT} % it declares. The counts and the share come "
                    f"from one census and cannot disagree; re-derive:\n"
                    f"        make -C rust SHELL=/bin/bash sv_parse_cost_family_share")

    # ── leg 2: identity — every input the recorded number is a function of ───────────────────
    live, missing = family_share_identity()
    recorded = art.get("identity", {})
    for label in FAMILY_SHARE_IDENTITY_LABELS:
        if label not in recorded:
            failures.append(f"{artifact} has no `{label}` identity row, so that input "
                            f"is unguarded — re-derive the artifact")
    stale = [lbl for lbl, row in live.items()
             if lbl in recorded and recorded[lbl].get("sha256") != row["sha256"]]
    if stale:
        failures.append(
            "the corpus family share NO LONGER DESCRIBES THIS TREE — "
            + ", ".join(f"`{lbl}` moved" for lbl in sorted(stale)) + ".\n"
            + "".join(f"        {lbl}: artifact `{recorded[lbl]['sha256'][:16]}…` vs live "
                      f"`{dict(live)[lbl]['sha256'][:16]}…`\n" for lbl in sorted(stale))
            + f"      The share is an exact function of these inputs. Re-derive it (~70 s) — do "
              f"not edit the number:\n"
              f"        make -C rust SHELL=/bin/bash sv_parse_cost_family_share")

    for m in missing:
        print(f"parse-cost: NOT EVALUATED — {m}", file=sys.stderr)
    if failures:
        for f in failures:
            print(f"parse-cost: ✗ {f}", file=sys.stderr)
        return 1
    checked = ", ".join(sorted(live))
    print(f"parse-cost: family share {CORPUS_FAMILY_SHARE_PCT} % "
          f"({art['lr_entries']:,} / {art['total_entries']:,} entries) — identity fresh for: "
          f"{checked}")
    return 0


def run_rederive_family_share(probe: str, jobs: int, out_path: str) -> int:
    """`.21` (f) TIER 2 (~70 s, on demand): the full-corpus census that DERIVES the share."""
    _self_check_family_share()
    files, listed = corpus_files()
    if not files or len(files) != listed:
        die(f"only {len(files)} of {listed} manifest-named corpus files are present. The vendored "
            f"corpora are git submodules; a partial corpus derives a partial share:\n"
            f"    git submodule update --init --recursive")
    ident, missing = family_share_identity()
    if missing:
        die("cannot derive the family share — " + "; ".join(missing))

    print(f"parse-cost: deriving the LR-family share over {len(files)} corpus files at -j{jobs} "
          f"(~70 s) ...", file=sys.stderr)
    t0 = time.perf_counter()
    rows, nodump = measure_entries(probe, files, jobs)
    adjudicate_nodump(nodump, "full-corpus family-share census", full_corpus=True)
    if not rows:
        die("every corpus file failed to produce a dump — refusing to publish an empty share")
    total = sum(r[2] for r in rows)
    lr_total = sum(r[5] for r in rows)
    if total <= 0:
        die("the census summed zero rule entries — refusing to publish a share with no denominator")
    share = f"{100.0 * lr_total / total:.3f}"
    elapsed = time.perf_counter() - t0

    art = {
        "schema": FAMILY_SHARE_SCHEMA,
        "corpus_family_share_pct": share,
        "lr_entries": lr_total,
        "total_entries": total,
        "files_measured": len(rows),
        "files_nodump": len(nodump),
        "nodump": [{"path": n.path, "reason": n.reason} for n in nodump],
        "files_in_manifest": listed,
        "identity": ident,
        "classifier_pattern": LR_FAMILY_RE.pattern,
        "derivation": ("python3 stimuli/sv/corpus_parse_cost.py --rederive-family-share "
                       f"--out {out_path}"),
        "note": ("The share of all PROTOCOL-graph rule entries taken by the left-recursion "
                 "elimination family over the whole SV corpus — `PARSE-COST-RATCHET`'s one "
                 "co-published live number, quoted verbatim on every designated live surface. "
                 "⛔ v1 of this schema also carried `blind_spot_factor` and "
                 "`wall_clock_regression_pct`; ENGINE-UNIVERSAL-SERVICES.26 RETIRED both. The "
                 "factor was wall_clock / this share, and both terms were wrong: the +24.3 % "
                 "numerator is refuted (.20 slice 5, not reproducible from its own raw data), and "
                 "the share was the wrong denominator independently of that — sensitivity is how "
                 "much the counter MOVED (+10.59 %, guard_ab_entries.txt), not how large the "
                 "family is. No ratio replaces it: no admissible wall-clock figure for the change "
                 "exists. ⛔ Derived, never hand-edited; `--verify-family-share` re-hashes every "
                 "identity row above and re-checks lr_entries/total_entries against the declared "
                 "share on every run."),
    }
    os.makedirs(os.path.dirname(os.path.join(ROOT, out_path)) or ".", exist_ok=True)
    with open(os.path.join(ROOT, out_path), "w", encoding="utf-8") as fh:
        json.dump(art, fh, indent=2, sort_keys=True)
        fh.write("\n")

    # ⛔ The no-dump count is PRINTED, not swallowed. `ENGINE-UNIVERSAL-SERVICES.22` is a record
    # of exactly one file being dropped silently by this census and perturbing five pinned sample
    # rows before anybody noticed it was gone.
    print(f"parse-cost: derived family share {share} % "
          f"({lr_total:,} of {total:,} entries over {len(rows)} files, {len(nodump)} no-dump) "
          f"in {elapsed:.0f} s -> {out_path}", file=sys.stderr)
    if share != CORPUS_FAMILY_SHARE_PCT:
        print(f"parse-cost: ⚠️ the carried CORPUS_FAMILY_SHARE_PCT is {CORPUS_FAMILY_SHARE_PCT} % "
              f"and the census derives {share} %. This mode deliberately does NOT edit the "
              f"constant — a claim generated by the run that measures it agrees with that run by "
              f"construction. Adopt it deliberately, then re-run `--verify-family-share`.",
              file=sys.stderr)
        return 1
    print(f"parse-cost: the carried constant {CORPUS_FAMILY_SHARE_PCT} % reproduces exactly.",
          file=sys.stderr)
    return 0


# ── modes ───────────────────────────────────────────────────────────────────────────────────

def run_census(probe: str, outdir: str, jobs: int) -> int:
    files, _listed = corpus_files()
    if not files:
        die("no corpus files found — the vendored corpora are git submodules; check them out")
    print(f"parse-cost: census over {len(files)} files at -j{jobs} ...", file=sys.stderr)
    rows, nodump = measure_entries(probe, files, jobs)
    adjudicate_nodump(nodump, "full-corpus census", full_corpus=True)
    os.makedirs(outdir, exist_ok=True)
    out = os.path.join(outdir, "census.tsv")
    with open(out, "w", encoding="utf-8") as fh:
        for rel, accepted, ent, _com, _memo, lr_e, _lr_c in rows:
            fh.write(f"{sub_corpus_of(rel)}\t{rel}\t{'True' if accepted == 'yes' else 'False'}"
                     f"\t{ent}\t{lr_e}\n")
    print(f"parse-cost: census -> {out} ({len(rows)} rows, {len(nodump)} no-dump, all declared)",
          file=sys.stderr)
    return 0


def run_measure(probe: str, outdir: str, manifest_path: str, jobs: int, repeats: int) -> int:
    pinned = read_manifest(manifest_path)
    tiers = {rel: tier for tier, rel in pinned}
    files = [rel for _t, rel in pinned]

    missing = [f for f in files if not os.path.isfile(os.path.join(ROOT, f))]
    if missing:
        die(f"{len(missing)} pinned sample file(s) are absent, e.g. {missing[0]}\n"
            f"  The corpora are git submodules. Check them out, or re-derive the sample:\n"
            f"    git submodule update --init --recursive")

    ident = identity(files)
    rows, nodump = measure_entries(probe, files, jobs)
    adjudicate_nodump(nodump, "pinned 192-file sample")
    if not rows:
        die("every sampled file failed to produce a dump — refusing to publish an empty baseline")

    os.makedirs(outdir, exist_ok=True)
    write_entries_tsv(os.path.join(outdir, "entries.tsv"), rows, tiers)
    write_report(os.path.join(outdir, "cost.md"), rows, ident, nodump, tiers)

    advisory_files = [rel for tier, rel in pinned if tier == "hot"]
    adv = measure_wallclock(probe, advisory_files, repeats)
    adv["probe"] = os.path.relpath(probe, ROOT)
    adv["note"] = ("ADVISORY ONLY — machine-dependent, measured on the FUSED cascade_* graph "
                   "that the BINDING entry counters cannot see. Compared with a wide band; a "
                   "breach warns, never fails.")
    with open(os.path.join(outdir, "advisory.json"), "w", encoding="utf-8") as fh:
        json.dump(adv, fh, indent=2, sort_keys=True)
        fh.write("\n")

    total = sum(r[2] for r in rows)
    com = sum(r[3] for r in rows)
    print(f"parse-cost: {len(rows)} files — BINDING entries={total:,} committed={com:,} "
          f"failed_speculation={total - com:,} memo_hits={sum(r[4] for r in rows):,} "
          f"(lr-family {sum(r[5] for r in rows):,}); ADVISORY parse={adv['total_parse_ms']:.0f} ms "
          f"over {adv['files']} hot files (floor {adv['process_floor_ms']:.1f} ms/invocation)",
          file=sys.stderr)
    return 0


def main() -> int:
    _self_check()
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--outdir", default=None)
    ap.add_argument("--manifest", default=os.path.join(ROOT, DEFAULT_MANIFEST))
    ap.add_argument("--probe", default=None)
    ap.add_argument("--jobs", type=int, default=8)
    ap.add_argument("--repeats", type=int, default=3,
                    help="median-of-K for the advisory wall clock (default 3)")
    ap.add_argument("--verify-families", action="store_true",
                    help="run the LR-family predicate over every generated parser's declared "
                         "rule names; refuse on any unclassified `_lr` name (`.21` (g))")
    ap.add_argument("--verify-family-share", action="store_true",
                    help="re-hash every input the carried corpus family share depends on and "
                         "fail when one moved (`.21` (f)); ~0.8 s, needs no probe")
    ap.add_argument("--rederive-family-share", action="store_true",
                    help="derive the corpus family share from a full-corpus census (~70 s) into "
                         "the tracked artifact (`.21` (f))")
    ap.add_argument("--family-share-artifact", default=FAMILY_SHARE_ARTIFACT,
                    help="which family-share artifact --verify-family-share checks; a parameter "
                         "so every refusal can be driven RED against a mutated COPY")
    ap.add_argument("--census", action="store_true", help="full-corpus entry census")
    ap.add_argument("--select", action="store_true", help="derive the sample manifest")
    ap.add_argument("--census-tsv", default=None)
    ap.add_argument("--out", default=None)
    ap.add_argument("--hot", type=int, default=40)
    ap.add_argument("--lr", type=int, default=40)
    ap.add_argument("--breadth", type=int, default=120)
    args = ap.parse_args()

    if args.verify_families:
        # ⛔ No probe needed: this reads the generated ARTIFACTS, not a parse.
        return run_verify_families()

    if args.verify_family_share:
        # ⛔ No probe needed either: this hashes inputs, it does not measure.
        return run_verify_family_share(args.family_share_artifact)

    if args.select:
        if not args.census_tsv or not args.out:
            die("--select needs --census-tsv and --out")
        sample = select_sample(args.census_tsv, args.hot, args.lr, args.breadth)
        with open(args.out, "w", encoding="utf-8") as fh:
            fh.write("# stimuli/sv/parse_cost_sample.tsv — the PINNED parse-cost sample\n")
            fh.write("#\n")
            fh.write("# ENGINE-UNIVERSAL-SERVICES.20 acceptance (d). Two tab-separated fields:\n")
            fh.write("#   tier \\t repo-root-relative path\n")
            fh.write("#\n")
            fh.write("# Derived deterministically from a full-corpus entry census — re-derive with:\n")
            fh.write("#   python3 stimuli/sv/corpus_parse_cost.py --census --outdir <dir>\n")
            fh.write("#   python3 stimuli/sv/corpus_parse_cost.py --select \\\n")
            fh.write("#       --census-tsv <dir>/census.tsv --out stimuli/sv/parse_cost_sample.tsv\n")
            fh.write("#\n")
            fh.write(f"# tiers: hot={args.hot} (heaviest by entries) lr={args.lr} (heaviest by\n")
            fh.write(f"# guarded-admission entries) breadth={args.breadth} (deterministic stride).\n")
            fh.write("#\n")
            fh.write("# ⛔ PINNED, not re-derived per run. A sample that re-selects itself from the\n")
            fh.write("# tree it is measuring can shed the file that regressed and report an\n")
            fh.write("# improvement. Re-derivation is a deliberate, reviewed act.\n")
            for tier, rel in sample:
                fh.write(f"{tier}\t{rel}\n")
        print(f"parse-cost: wrote {len(sample)} rows -> {args.out}", file=sys.stderr)
        return 0

    probe = resolve_probe(args.probe)
    if args.rederive_family_share:
        return run_rederive_family_share(probe, args.jobs, args.out or FAMILY_SHARE_ARTIFACT)
    if args.census:
        if not args.outdir:
            die("--census needs --outdir")
        return run_census(probe, args.outdir, args.jobs)
    if not args.outdir:
        die("--outdir is required")
    return run_measure(probe, args.outdir, args.manifest, args.jobs, args.repeats)


if __name__ == "__main__":
    sys.exit(main())
