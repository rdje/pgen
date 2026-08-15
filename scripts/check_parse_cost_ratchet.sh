#!/usr/bin/env bash
# scripts/check_parse_cost_ratchet.sh
#
# DOCTRINE `PARSE-COST-RATCHET` (structural + deterministic-oracle) — ENGINE-UNIVERSAL-SERVICES.20
# acceptance (d), under the 2026-08-14 ruling C.
#
#   A cost nothing measures is a cost that grows.
#
# ⛔ THE DEFECT THIS CLOSES — MEASURED, NOT HYPOTHETICAL. `.17` slice 9 shipped the guarded
# left-recursion admission and it cost **+24.3 % parse time** on the SV corpus. Across that
# slowdown, EVERY gate this repository has was GREEN: the generated lint, the two-sided repro
# ratchet, the corpus pass/fail count, and all registered doctrines. Nothing in the tree measured
# parse cost at all, so nothing could report that it had moved.
#
# ⭐⭐ TWO TIERS, AND THE CHEAP ONE IS SOUND RATHER THAN A SHORTCUT.
#
#   TIER 1 (every run, ~2 s) — IDENTITY, on everything this gate PUBLISHES, in four arms:
#     (1) BASELINE IDENTITY. Re-hash the four inputs the baseline names: the grammar, the
#       generated parser, the instrument, and a digest over the sampled corpus files themselves.
#       The binding metric is an exact function of exactly those inputs (verified: deterministic
#       across repeated release runs, and byte-identical between the debug and release probes).
#       ⇒ if none of them moved, the measurement CANNOT have moved. This is a proof, not a
#       sampling heuristic — and when it fails it does not guess, it demands a re-measure.
#     (2) THE LR-FAMILY CLASSIFIER (`--verify-families`, ~0.1 s). Every generated parser's
#       declared `_lr` rule names must be classified by the shipped predicate.
#     (3) THE PUBLISHED FAMILY SHARE (`--verify-family-share`, ~1 s). The corpus-wide share the
#       blind-spot bound below is computed from is re-hashed against the tracked derivation.
#     (4) CO-PUBLICATION. Every designated live surface must carry that derived share and factor,
#       so the bound cannot be quoted anywhere in the repository after it has gone stale.
#
#   TIER 2 (on demand, ~2.5 min) — THE RATCHET ITSELF. Re-run the instrument into a SCRATCH
#     directory and compare. Entries/committed/memo-hits must not RISE. Wall clock is reported
#     against a wide band and never fails the gate.
#
# ⛔ ARMS 2-4 WERE ON DEMAND UNTIL `ENGINE-UNIVERSAL-SERVICES.21` ACCEPTANCE (f), AND THAT IS THE
# DEFECT (f) CLOSES. `--verify-families` rode tier 2, so it ran when an operator chose to
# re-measure and never on a commit; the family share was a hand-carried constant referenced only
# by the file that defined it and guarded by a comment reading *"re-derive it, do not edit this
# line"*. `.21` slice 1 had replaced a WRONG unwatched number with a RIGHT unwatched number —
# `docs/CLAIM_VERIFICATION.md` §6 names that as an anti-pattern in its own right. All three now
# ride the every-run tier, because they cost seconds and the thing they guard rots in silence.
#
# ⛔ WHY THE PARSER HASH IS THE TRIPWIRE AND NOT AN INCONVENIENCE. `generated/
# systemverilog_parser.rs` changes exactly when the parser's behaviour can change. Making the
# baseline stale on that event is what forces the re-measure at the moment the cost could have
# moved — instead of at some later moment nobody schedules. That is the direct fix for the
# `SV-CORPUS-GRAD.13i` finding: six tracked oracles carried a self-describing "instrument
# identity" block, only ONE was gate-checked, and four were measurably stale.
#
# ⛔ THE SAMPLE-INPUT DIGEST IS NOT REDUNDANT WITH THE PARSER HASH. The corpora are git
# SUBMODULES. A submodule bump changes what is measured without touching one byte of PGEN, so a
# parser hash alone would report a fresh baseline over a corpus that had changed underneath it.
#
# ⚠️ HONEST LIMIT, stated rather than discovered later (DOCTRINE_ENFORCEMENT.md §3). The binding
# metric observes the PROTOCOL graph. A production parse with no diagnostic consumer runs the
# FUSED `cascade_*` graph, which ticks no per-rule counters — and the wall-clock regression was
# measured there. So this metric is materially LESS sensitive to that regression than wall clock
# is, by a factor this gate re-derives rather than states: the share and the factor live in
# `docs/tasks/artifacts/…/parse_cost_ratchet/family_share.json` and arm 3 above re-hashes them
# every run. ⛔ The bound READ ~35× until `ENGINE-UNIVERSAL-SERVICES.21`, because the classifier
# that measured it counted only `_lr_base`/`_lr_suffix` — no `_lr_seed`, and in a family named for
# the GUARD, no `_lr_guard` rule at all: 75.1 % of the family's entries were uncounted, and the
# gate was UNDER-claiming its own sensitivity by ~4×. ⭐ The digits are deliberately NOT repeated
# in this comment any more: a gate that both publishes a number and checks it is checking itself.
# What binds is STRUCTURAL work, exactly; this gate does not claim to price the fused graph. The
# wall-clock advisory is the only view of the other graph and is machine-dependent, which is
# precisely why it advises and does not bind. Neither metric alone is sufficient, and the report
# says so on every run.
#
# ⚠️ The vendored corpora are submodules, so a hosted checkout without them cannot measure at all.
# That is reported as NOT EVALUATED — loudly, never as a pass
# (docs/decisions/feedback_a_check_that_cannot_run_must_say_so.md).
#
# CONTRACT (DOCTRINE_ENFORCEMENT.md §4): exit code is the verdict (0 holds / 1 breach / 2 refuses);
# explains on stderr; deterministic; does not mutate tracked state; path-agnostic.
#
# Usage:
#   bash scripts/check_parse_cost_ratchet.sh                  # tier 1 (identity)
#   PGEN_PARSE_COST_REMEASURE=1 bash scripts/check_parse_cost_ratchet.sh    # + tier 2 (ratchet)
#   PGEN_PARSE_COST_REBASELINE=1 bash scripts/check_parse_cost_ratchet.sh   # promote a new baseline
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$ROOT" || exit 2

exec python3 - "$@" <<'PYEOF'
import hashlib, json, os, re, shutil, subprocess, sys

ROOT = os.getcwd()
INSTRUMENT = "stimuli/sv/corpus_parse_cost.py"
MANIFEST = "stimuli/sv/parse_cost_sample.tsv"
ART = "docs/tasks/artifacts/engine_universal_services/parse_cost_ratchet"
STATE = os.environ.get("PGEN_PARSE_COST_STATE_DIR", "rust/target/parse_cost_ratchet")
GRAMMAR_FILE = "grammars/systemverilog.ebnf"
GENERATED_PARSER = "generated/systemverilog_parser.rs"
GENERATED_DIR = "generated"
FAMILY_SHARE = f"{ART}/family_share.json"

# ── the CO-PUBLICATION surfaces (`ENGINE-UNIVERSAL-SERVICES.21` acceptance (f)) ─────────────────
#
# ⛔ THE NUMBER'S FAILURE MODE WAS NEVER "somebody edits the constant". It was that ONE figure was
# hand-copied into four documents and went stale in all of them at once, silently, because no
# reader could tell a live claim from a quotation. Gating the constant alone would fix the file
# nobody was reading from. ⇒ every surface that states the bound as a LIVE fact carries the
# derived pair, and this gate holds them equal to the artifact — the second leg
# `SV-CORPUS-DENOMINATOR` uses, for the same reason.
#
# ⚠️ `check_parse_cost_ratchet.sh` is deliberately NOT in this list: an assertion about a file
# cannot live inside that file as a literal
# (docs/decisions/reference_self_referential_assertion_is_unsound.md). Its header now cites the
# artifact instead of repeating the digits, which removes the copy rather than checking it.
LIVE_SURFACES = [
    "TOOLBOX.md",
    "DOCTRINE_ENFORCEMENT.md",
    "docs/knowledge/a-deterministic-counter-cannot-see-a-per-entry-cost-rise.md",
    f"{ART}/cost.md",
]
SHARE_ANCHOR_MARK = "Live LR-family share"
SHARE_TUPLE_RE = re.compile(r"`(\d+\.\d+/\d+\.\d+)`")

REMEASURE = os.environ.get("PGEN_PARSE_COST_REMEASURE", "0") == "1"
REBASELINE = os.environ.get("PGEN_PARSE_COST_REBASELINE", "0") == "1"

# ⛔ The advisory band is WIDE on purpose. It is compared across whatever machine happens to run
# it, and this leaf's own history is the argument: the first cost figure recorded here, `~11 %`,
# was wrong because it compared wall clock across "materially faster machine conditions". A tight
# band on a machine-dependent number manufactures false alarms, and a gate that cries wolf is
# a gate people learn to bypass.
ADVISORY_BAND_PCT = 50.0

failures, notes, unevaluated = [], [], []


def fail(msg):
    failures.append(msg)


def sha256_of(path):
    h = hashlib.sha256()
    with open(path, "rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


# ── the identity reader, as a pure function so it can carry ground truth ────────────────────────
IDENT_ROW = re.compile(r"^\|\s*([a-z ]+?)\s*\|\s*`([^`]+)`\s*\|\s*`([0-9a-f]{64})`\s*\|\s*$")


def read_identity(text):
    """Every `| label | \\`path\\` | \\`sha256\\` |` row of the baseline's identity table.

    ⛔ Anchored on a 64-hex-digit third cell rather than on "a row in the identity section".
    A section-scoped reader silently returns nothing when the heading is renamed, and returning
    nothing would read as "no identity to check" — a miss in the passing direction, which is the
    failure shape this whole tree exists to remove.
    """
    out = {}
    for line in text.splitlines():
        m = IDENT_ROW.match(line)
        if m:
            out[m.group(1).strip()] = (m.group(2), m.group(3))
    return out


def share_anchors(text):
    """Every `share/factor` tuple inside a PARAGRAPH carrying the live-share MARKER.

    ⛔ Marker-scoped for the same reason `check_sv_corpus_denominator.sh` is: this bound has an
    ERA — it was published as `0.681/35.7` before `.21` corrected the classifier — and a
    historical citation is history, not a stale live claim (supersede-don't-mutate). Only a
    paragraph that declares itself live is read.
    ⚠️ Paragraph-scoped rather than line-scoped, because the book and TOOLBOX wrap their prose, so
    the marker and the tuple legitimately land on different lines. ALL tuples in a marker
    paragraph are returned, so an unrelated number sharing it fails loudly instead of being
    silently averaged into agreement.
    """
    out = []
    for para in re.split(r"\n[ \t]*\n", text):
        if SHARE_ANCHOR_MARK not in para:
            continue
        out.extend(m.group(1) for m in SHARE_TUPLE_RE.finditer(para))
    return out


def self_check():
    """GROUND TRUTH, re-run on every invocation (microseconds). Positive controls AND negatives;
    a MISS refuses (exit 2) rather than reporting a clean tree
    (docs/decisions/feedback_instrument_needs_ground_truth.md)."""
    h = "0" * 64
    cases = [
        (f"| grammar | `grammars/systemverilog.ebnf` | `{h}` |", {"grammar": ("grammars/systemverilog.ebnf", h)}),
        (f"| generated parser | `generated/x.rs` | `{h}` |", {"generated parser": ("generated/x.rs", h)}),
        # a table row that is NOT an identity row must not be absorbed
        ("| rule entries | **416,841,264** | more work |", {}),
        # a truncated / non-hex digest is not a sha256 and must not pass as one
        ("| grammar | `g.ebnf` | `deadbeef` |", {}),
        # prose that merely mentions a hash is not a row
        (f"the sha256 is `{h}` today", {}),
    ]
    misses = 0
    for text, want in cases:
        got = read_identity(text)
        if got != want:
            print(f"parse-cost-ratchet: CONTROL MISSED: {text!r} want={want} got={got}",
                  file=sys.stderr)
            misses += 1
    anchor_cases = [
        (f"x **{SHARE_ANCHOR_MARK} `2.741/8.9`** y", ["2.741/8.9"]),
        # the marker and the tuple wrapped onto separate lines of one paragraph
        (f"{SHARE_ANCHOR_MARK} —\nthe pair is `2.741/8.9` today", ["2.741/8.9"]),
        # ⭐ the SAME digits with no marker are an era-dated citation, and must stay invisible
        ("the bound was `0.681/35.7` before .21 corrected it", []),
        # a marker with no tuple is a missing anchor, which the call site treats as missing
        (f"{SHARE_ANCHOR_MARK} — pending re-derivation", []),
        # unbackticked digits are prose, not an anchor
        (f"{SHARE_ANCHOR_MARK} 2.741/8.9", []),
        # two anchors, both seen — so disagreement fails rather than being averaged
        (f"{SHARE_ANCHOR_MARK} `2.741/8.9`\n{SHARE_ANCHOR_MARK} `9.999/9.9`",
         ["2.741/8.9", "9.999/9.9"]),
        # a paragraph WITHOUT the marker is not read even if it is adjacent to one
        (f"{SHARE_ANCHOR_MARK} `2.741/8.9`\n\nelsewhere `1.234/5.6` is quoted", ["2.741/8.9"]),
    ]
    for text, want in anchor_cases:
        got = share_anchors(text)
        if got != want:
            print(f"parse-cost-ratchet: CONTROL MISSED: {text!r} want={want} got={got}",
                  file=sys.stderr)
            misses += 1
    if misses:
        print(f"parse-cost-ratchet: the identity/anchor readers do not discriminate "
              f"({misses} control(s) missed); refusing", file=sys.stderr)
        sys.exit(2)


self_check()

# ── preconditions ───────────────────────────────────────────────────────────────────────────────
for p in (INSTRUMENT, MANIFEST, f"{ART}/cost.md", f"{ART}/entries.tsv", f"{ART}/advisory.json"):
    if not os.path.isfile(p):
        print(f"parse-cost-ratchet: REFUSING — required file missing: {p}", file=sys.stderr)
        sys.exit(2)

with open(f"{ART}/cost.md", encoding="utf-8") as fh:
    baseline_md = fh.read()
ident = read_identity(baseline_md)
if not ident:
    print(f"parse-cost-ratchet: REFUSING — {ART}/cost.md carries NO instrument-identity rows. "
          f"A baseline that cannot say what produced it cannot be checked for staleness.",
          file=sys.stderr)
    sys.exit(2)

# The sampled corpus files, from the pinned manifest.
sample = []
with open(MANIFEST, encoding="utf-8") as fh:
    for line in fh:
        line = line.rstrip("\n")
        if line.strip() and not line.startswith("#"):
            parts = line.split("\t")
            if len(parts) == 2:
                sample.append(parts[1])

corpus_present = bool(sample) and all(os.path.isfile(os.path.join(ROOT, f)) for f in sample)

# ── TIER 1: identity ────────────────────────────────────────────────────────────────────────────
def sample_input_digest(files):
    h = hashlib.sha256()
    for rel in files:
        h.update(rel.encode("utf-8"))
        h.update(b"\0")
        with open(os.path.join(ROOT, rel), "rb") as fh:
            for chunk in iter(lambda: fh.read(1 << 20), b""):
                h.update(chunk)
        h.update(b"\0")
    return h.hexdigest()


live = {}
if os.path.isfile(GRAMMAR_FILE):
    live["grammar"] = sha256_of(GRAMMAR_FILE)
else:
    unevaluated.append(f"{GRAMMAR_FILE} is absent")
if os.path.isfile(GENERATED_PARSER):
    live["generated parser"] = sha256_of(GENERATED_PARSER)
else:
    unevaluated.append(f"{GENERATED_PARSER} is absent — generated/ is not tracked; regenerate "
                       f"with `make -C rust SHELL=/bin/bash regenerate_generated_parsers`")
# ⛔ THE INSTRUMENT IS AN INPUT TOO (`ENGINE-UNIVERSAL-SERVICES.21`). The BINDING counters are an
# exact function of grammar+parser+inputs and the instrument only reads them — but `entries.tsv`
# also publishes `lr_entries`/`lr_committed` and `cost.md` a family share, and those ARE functions
# of the instrument's classifier. Correcting that classifier staled every published family number
# while this tier reported `fresh`. A baseline's identity must name everything its ARTIFACT
# depends on, not everything its headline metric depends on.
if os.path.isfile(INSTRUMENT):
    live["instrument"] = sha256_of(INSTRUMENT)
else:
    unevaluated.append(f"{INSTRUMENT} is absent — the instrument that produced the baseline")
if corpus_present:
    live["sample inputs"] = sample_input_digest(sample)
else:
    unevaluated.append("the vendored SV corpora are git submodules and the pinned sample is not "
                       "fully checked out here")

# ⛔ A REBASELINE IS THE ACT THAT RESOLVES AN IDENTITY DIVERGENCE, SO IT CANNOT BE BLOCKED BY ONE
# (`ENGINE-UNIVERSAL-SERVICES.21`). Deriving these as hard failures on every path deadlocked the
# only supported way to adopt a NEW identity input: the `instrument` row cannot exist until a
# rebaseline writes it, and the rebaseline refused to write while the row was missing. Under
# `PGEN_PARSE_COST_REBASELINE=1` both conditions are reported as NOTES and the re-measure decides;
# on every other path they stay hard failures, unchanged. ⭐ This is not a relaxation of the gate:
# a rebaseline is an explicit, deliberate, env-gated operator act whose entire purpose is to
# declare "this tree is the new reference", and it still refuses if the RATCHET itself breaches.
def identity_problem(msg):
    (notes if REBASELINE else failures).append(
        (msg + "  [reported as a note: PGEN_PARSE_COST_REBASELINE=1 is set, and adopting the "
                "current tree is what a rebaseline is for]") if REBASELINE else msg)


stale = []
for label, digest in live.items():
    if label not in ident:
        identity_problem(f"the baseline's identity table has no `{label}` row, so that input "
                         f"is unguarded")
        continue
    if ident[label][1] != digest:
        stale.append(label)

if stale:
    identity_problem("the parse-cost BASELINE IS STALE — it no longer describes this tree.\n"
         + "".join(f"        {lbl}: baseline `{ident[lbl][1][:16]}…` vs live `{live[lbl][:16]}…`\n"
                   for lbl in stale)
         + "      The binding metric is an exact function of these inputs, so a change here is a\n"
           "      change in what the parser costs. Re-measure — do not edit the number:\n"
           "        make -C rust SHELL=/bin/bash sv_parse_cost_ratchet")

# ── TIER 1, arm 2: the LR-family CLASSIFIER, over every generated parser ────────────────────────
#
# ⛔ MOVED HERE FROM TIER 2 BY `.21` ACCEPTANCE (f). It reads `generated/`, which the baseline
# identity does not enumerate, so it was hung off the on-demand re-measure — and an arm that runs
# only when an operator chooses to re-measure runs approximately never. Measured cost of running
# it on every commit instead: 0.1 s. There was never a cost argument for deferring it, only an
# architectural one, and `GATE-REACHABILITY` is the record of what happens to checks nobody calls.
# ⛔ `generated/` is NOT tracked, so its absence is NOT EVALUATED — never a pass, never a failure
# (docs/decisions/feedback_a_check_that_cannot_run_must_say_so.md).
gen_present = (os.path.isdir(GENERATED_DIR)
               and any(n.endswith(".rs") for n in os.listdir(GENERATED_DIR)))
if gen_present:
    fam = subprocess.run([sys.executable, INSTRUMENT, "--verify-families"],
                         capture_output=True, text=True)
    if fam.returncode != 0:
        fail("the LR-family classifier does not classify every declared `_lr` rule name across "
             "the generated parsers:\n"
             + "".join(f"        {l}\n" for l in (fam.stderr or fam.stdout).strip().splitlines()[:6])
             + "      Re-derive it from the eliminators' emission sites — never from this list.")
else:
    unevaluated.append(f"{GENERATED_DIR}/ holds no generated parser, so the LR-family classifier "
                       f"could not be run over any declared rule names; regenerate with "
                       f"`make -C rust SHELL=/bin/bash regenerate_generated_parsers`")

# ── TIER 1, arm 3: the PUBLISHED FAMILY SHARE ───────────────────────────────────────────────────
#
# The corpus-wide share the blind-spot bound is computed from. Derived by a ~70 s full-corpus
# census into a tracked artifact; re-hashed here against its four recorded inputs in ~1 s.
share_proc = subprocess.run([sys.executable, INSTRUMENT, "--verify-family-share"],
                            capture_output=True, text=True)
if share_proc.returncode != 0:
    fail("the published LR-family share no longer describes this tree:\n"
         + "".join(f"        {l}\n"
                   for l in (share_proc.stderr or share_proc.stdout).strip().splitlines()[:8]))

# ── TIER 1, arm 4: CO-PUBLICATION of that share on every designated live surface ────────────────
derived_pair = None
if os.path.isfile(FAMILY_SHARE):
    try:
        with open(FAMILY_SHARE, encoding="utf-8") as fh:
            fs = json.load(fh)
        derived_pair = f"{fs['corpus_family_share_pct']}/{fs['blind_spot_factor']}"
    except (OSError, KeyError, json.JSONDecodeError) as exc:
        fail(f"{FAMILY_SHARE} could not be read for the co-publication check: {exc}")
else:
    fail(f"{FAMILY_SHARE} is missing — the published family share has no tracked derivation. "
         f"Re-derive it:\n"
         f"        make -C rust SHELL=/bin/bash sv_parse_cost_family_share")


# ⛔⛔ ONE OF THESE SURFACES IS WRITTEN BY THE REBASELINE ITSELF, AND THAT IS A DEADLOCK IF IT IS
# TREATED LIKE THE OTHERS. `cost.md` is REGENERATED by tier 2; its anchor therefore cannot be
# correct until a rebaseline copies the fresh report in, and `if REBASELINE and not failures`
# refuses to copy while any failure stands. That is precisely the bootstrap deadlock `.21` slice 1
# found and fixed for the `instrument` identity row — the same shape, one surface over. ⇒ under
# `PGEN_PARSE_COST_REBASELINE=1` a stale anchor in a REGENERATED artifact is a NOTE and the copy
# decides; the hand-written surfaces stay hard failures on every path, unchanged.
REGENERATED_SURFACES = {f"{ART}/cost.md"}


def copublication_problem(surface, msg):
    if REBASELINE and surface in REGENERATED_SURFACES:
        notes.append(msg + "  [note, not a failure: this surface is REGENERATED by the rebaseline "
                           "now running, and refusing here would block the act that fixes it]")
    else:
        fail(msg)


if derived_pair:
    for surface in LIVE_SURFACES:
        if not os.path.isfile(surface):
            copublication_problem(surface, f"designated live surface missing: {surface}")
            continue
        with open(surface, encoding="utf-8") as fh:
            found = share_anchors(fh.read())
        if not found:
            copublication_problem(
                surface,
                f"{surface} carries NO live LR-family-share anchor. Add a paragraph containing "
                f"'{SHARE_ANCHOR_MARK}' and the pair `{derived_pair}` "
                f"(corpus-entry share % / blind-spot factor), so the bound cannot be quoted here "
                f"after it has gone stale.")
            continue
        wrong = sorted(set(t for t in found if t != derived_pair))
        if wrong:
            copublication_problem(
                surface,
                f"{surface} publishes LR-family share pair(s) {wrong} but the tracked derivation "
                f"says {derived_pair}. The share and its blind-spot factor move TOGETHER — "
                f"update the anchor, or re-derive if the tree moved.")

# ── TIER 2: the ratchet ─────────────────────────────────────────────────────────────────────────
BINDING = ("entries", "committed", "memo_hits")


def totals_of(tsv_path):
    """Sum the binding columns of an entries.tsv. Refuses on an unexpected header rather than
    unpacking positionally — a consumer that outlives its producer's schema is gate-flow failure
    §7.8, and a positional read of a renamed column fails SILENTLY with a plausible number."""
    with open(tsv_path, encoding="utf-8") as fh:
        header = fh.readline().rstrip("\n").split("\t")
        want = ["sub_corpus", "tier", "path", "accepted", "entries", "committed", "memo_hits",
                "lr_entries", "lr_committed"]
        if header != want:
            print(f"parse-cost-ratchet: REFUSING — {tsv_path} header is {header}, expected {want}. "
                  f"The instrument's schema moved; fix this reader rather than trusting a "
                  f"positional unpack.", file=sys.stderr)
            sys.exit(2)
        idx = {name: i for i, name in enumerate(header)}
        tot = {k: 0 for k in BINDING}
        rows = {}
        for line in fh:
            if not line.strip():
                continue
            c = line.rstrip("\n").split("\t")
            for k in BINDING:
                tot[k] += int(c[idx[k]])
            rows[c[idx["path"]]] = (c[idx["accepted"]], int(c[idx["entries"]]))
        return tot, rows


if REMEASURE or REBASELINE:
    if not corpus_present:
        unevaluated.append("tier 2 (the ratchet) could not run: the pinned sample is not checked out")
    elif not os.path.isfile(GENERATED_PARSER):
        unevaluated.append("tier 2 (the ratchet) could not run: the generated parser is absent")
    else:
        scratch = os.path.join(ROOT, STATE, "scratch")
        shutil.rmtree(scratch, ignore_errors=True)
        os.makedirs(scratch, exist_ok=True)
        print("parse-cost-ratchet: re-measuring the pinned sample (~2.5 min) ...", file=sys.stderr)
        proc = subprocess.run([sys.executable, INSTRUMENT, "--outdir", scratch],
                              capture_output=True, text=True)
        if proc.returncode != 0:
            print("parse-cost-ratchet: REFUSING — the instrument would not run:", file=sys.stderr)
            print((proc.stderr or proc.stdout).strip(), file=sys.stderr)
            sys.exit(2)

        # ⭐ `.21` acceptance (g)'s `--verify-families` USED to run here, and (f) moved it to
        # tier 1 above. The honest limit it carried — *"tier 2 is on demand, so this arm runs when
        # an operator re-measures, NOT on every commit"* — was the whole reason it had to move.

        base_tot, base_rows = totals_of(f"{ART}/entries.tsv")
        new_tot, new_rows = totals_of(f"{scratch}/entries.tsv")

        # ⭐ An accept-flip is reported SEPARATELY from a cost move. Conflating them is how a
        # correctness change gets read as a performance regression (or hides one): a file that
        # stops parsing does dramatically different work, and that is a `.13`/`.17` question,
        # not a `.20` one.
        flips = [p for p in base_rows if p in new_rows and base_rows[p][0] != new_rows[p][0]]
        if flips:
            notes.append(f"{len(flips)} sampled file(s) CHANGED ACCEPT VERDICT since the baseline "
                         f"(e.g. {flips[0]}). That is a correctness move; the cost comparison "
                         f"below is reported but is no longer like-for-like.")

        for k in BINDING:
            b, n = base_tot[k], new_tot[k]
            if n > b:
                fail(f"BINDING metric `{k}` ROSE {b:,} -> {n:,} (+{100.0 * (n - b) / b:.2f} %). "
                     f"Costs are REJECTED, not traded — attribute it and eliminate it, or record "
                     f"it as irreducible with the measurement that proves it (`.20` acceptance).")
            elif n < b:
                notes.append(f"BINDING metric `{k}` FELL {b:,} -> {n:,} "
                             f"({100.0 * (n - b) / b:+.2f} %) — an improvement. Promote it "
                             f"deliberately so the ratchet tightens: "
                             f"PGEN_PARSE_COST_REBASELINE=1 bash scripts/check_parse_cost_ratchet.sh")

        # advisory, never a failure
        with open(f"{ART}/advisory.json", encoding="utf-8") as fh:
            base_adv = json.load(fh)
        with open(f"{scratch}/advisory.json", encoding="utf-8") as fh:
            new_adv = json.load(fh)
        bp, np_ = base_adv.get("total_parse_ms", 0), new_adv.get("total_parse_ms", 0)
        if bp > 0:
            delta = 100.0 * (np_ - bp) / bp
            verdict = "within" if abs(delta) <= ADVISORY_BAND_PCT else "OUTSIDE"
            print(f"parse-cost-ratchet: ADVISORY (does not bind) — floor-adjusted parse wall clock "
                  f"{bp:.0f} ms -> {np_:.0f} ms ({delta:+.1f} %), {verdict} the ±{ADVISORY_BAND_PCT:.0f} % "
                  f"band. Machine-dependent by construction: it is the only view of the FUSED "
                  f"cascade_* graph, which the binding counters cannot see.", file=sys.stderr)

        if REBASELINE and not failures:
            for name in ("entries.tsv", "cost.md", "advisory.json"):
                shutil.copyfile(os.path.join(scratch, name), os.path.join(ROOT, ART, name))
            print(f"parse-cost-ratchet: REBASELINED — {ART} now records this measurement.",
                  file=sys.stderr)
elif not stale:
    notes.append("tier 2 (the re-measure) did not run, and did not need to: every input the "
                 "binding metric depends on is byte-identical to the baseline's, so the "
                 "measurement cannot have moved. Force it with PGEN_PARSE_COST_REMEASURE=1.")

# ── verdict ─────────────────────────────────────────────────────────────────────────────────────
for u in unevaluated:
    print(f"parse-cost-ratchet: NOT EVALUATED — {u}", file=sys.stderr)
for n in notes:
    print(f"parse-cost-ratchet: note — {n}", file=sys.stderr)

if failures:
    print(f"parse-cost-ratchet: {len(failures)} breach(es):", file=sys.stderr)
    for f in failures:
        print(f"  ✗ {f}", file=sys.stderr)
    sys.exit(1)

checked = ", ".join(sorted(live)) or "nothing (see NOT EVALUATED above)"
print(f"parse-cost-ratchet: OK (identity fresh for: {checked}; "
      f"{len(sample)} pinned sample files)")
PYEOF
