#!/usr/bin/env bash
# scripts/check_parse_cost_ratchet.sh
#
# DOCTRINE `PARSE-COST-RATCHET` (structural + deterministic-oracle) — ENGINE-UNIVERSAL-SERVICES.20
# acceptance (d), under the 2026-08-14 ruling C.
#
#   A cost nothing measures is a cost that grows.
#
# ⛔ THE DEFECT THIS CLOSES — MEASURED, NOT HYPOTHETICAL. `.17` slice 9 shipped the guarded
# left-recursion admission, and the repository could not say what it had cost. EVERY gate here was
# GREEN across it: the generated lint, the two-sided repro ratchet, the corpus pass/fail count, and
# all registered doctrines. Nothing in the tree measured parse cost at all, so nothing could report
# that it had moved. What the change DID cost, on the metric this gate binds: **+10.59 % rule
# entries** (`docs/tasks/artifacts/engine_universal_services/guard_ab_entries.txt`).
#
# ⛔⛔ AND THE NUMBER THAT COMMISSIONED THIS GATE IS REFUTED, WHICH ARGUES FOR IT RATHER THAN
# AGAINST IT (`ENGINE-UNIVERSAL-SERVICES.20` slice 5 / `.26`, 2026-08-16). This header used to say
# the admission cost **+24.3 % parse time**. It is not reproducible from the raw data of the runs
# that produced it — seven estimator × era combinations put ARM2/ARM1 in [0.9909, 1.0433] — and the
# mechanism was a fixed-arm-order measurement artifact. A wall-clock figure nobody could re-derive
# was carried for three sessions, through five slices and a director-facing ruling, and was wrong
# by ~20 points. That is the case for a ratchet keyed on numbers a reader can re-derive.
#
# ⭐⭐ TWO TIERS, AND THE CHEAP ONE IS SOUND RATHER THAN A SHORTCUT.
#
#   TIER 1 (every run, ~2 s) — IDENTITY, on everything this gate PUBLISHES, in five arms:
#     (1) BASELINE IDENTITY. Re-hash the four inputs the baseline names: the grammar, the
#       generated parser, the instrument, and a digest over the sampled corpus files themselves.
#       The binding metric is an exact function of exactly those inputs (verified: deterministic
#       across repeated release runs, and byte-identical between the debug and release probes).
#       ⇒ if none of them moved, the measurement CANNOT have moved. This is a proof, not a
#       sampling heuristic — and when it fails it does not guess, it demands a re-measure.
#     (2) THE LR-FAMILY CLASSIFIER (`--verify-families`, ~0.1 s). Every generated parser's
#       declared `_lr` rule names must be classified by the shipped predicate.
#     (3) THE PUBLISHED FAMILY SHARE (`--verify-family-share`, ~1 s). The corpus-wide LR-family
#       share is re-hashed against the tracked derivation, and its raw counts must reproduce it.
#     (4) CO-PUBLICATION. Every designated live surface must carry that derived share, so it
#       cannot be quoted anywhere in the repository after it has gone stale.
#     (5) THE PROBE FINGERPRINT (`--verify-probe-fingerprint`, ~0.02 s). The four identity rows
#       are all SOURCES; the thing that produces the numbers is an untracked build artifact.
#       `rust/build.rs` now hashes every generated parser it resolves and the probe reports which
#       one it embeds, so this arm can ask the question the identity table could not. ⛔ At TIER 1
#       a mismatch is a NOTE, not a breach: this tier computes no measurement, so a stale probe
#       misleads nobody here. It becomes a REFUSAL at tier 2, which does.
#
#   TIER 2 (on demand, ~2.5 min) — THE RATCHET ITSELF. Re-run the instrument into a SCRATCH
#     directory and compare. Entries/committed/memo-hits must not RISE. Wall clock is reported
#     against a wide band and never fails the gate. ⛔ PRE-FLIGHTED on arm 5: a probe embedding a
#     different parser REFUSES before one file is measured.
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
# ⛔⛔ AND NONE OF THE FOUR IS THE EXECUTABLE — WHICH IS THE HOLE ARM 5 CLOSES
# (`ENGINE-UNIVERSAL-SERVICES.24`). All four rows are SOURCES. The numbers are produced by
# `rust/target/release/parseability_probe`, an untracked build artifact nothing hashed and nothing
# tied to the parser it was compiled from. `.20` slice 4 built one from a guard-suppressed
# experimental arm and this gate printed *"the measurement cannot have moved"* while `nm … |
# grep -c _lr_guard` read 0 against a pinned parser declaring 6. ⭐ The gap fails in the PASSING
# direction, which is why it is closed rather than documented: on four sampled files that wrong
# binary reports 762,345 rule entries where the shipped one reports 11,240,430, and a FALL is not
# a breach here — it is a note inviting a rebaseline that would lower the ratchet permanently to a
# number no real parser produces.
#
# ⚠️ HONEST LIMIT, stated rather than discovered later (DOCTRINE_ENFORCEMENT.md §3). The binding
# metric observes the PROTOCOL graph; a production parse with no diagnostic consumer runs the FUSED
# `cascade_*` graph, which ticks no per-rule counters. And a counter counts EVENTS — a rise in the
# cost PER event is invisible to it on any graph. ⭐ That is a PROPERTY of the metric, not a
# measurement of it, which is why it is stated here without a number: it cannot go stale.
#
# ⛔⛔ IT USED TO CARRY A NUMBER — *"at least ~8.9× less sensitive than wall clock"* — AND BOTH OF
# ITS TERMS WERE WRONG (`ENGINE-UNIVERSAL-SERVICES.26`). It was `24.3 / 2.741`: a refuted
# wall-clock numerator, over a denominator that was the wrong quantity independently of that.
# Sensitivity is how much the counter MOVED, not how large the rule family is — and the flip moved
# the binding counters **+10.59 %**, 3.49× larger than the whole family's own entry count. The
# factor is RETIRED, not re-computed: it fails under every reading available (0.41× on the point
# estimate, 1.82× on the most adversarial pairing), and `.20` (b) established that no admissible
# wall-clock figure for the change exists to rebuild it from. ⛔ The share ITSELF is a correctly
# measured live quantity and is unaffected; it stays co-published and gated by arms 3-4.
# ⚠️ It READ 0.681 %/~35× until `.21`, because the classifier that measured it counted only
# `_lr_base`/`_lr_suffix` — no `_lr_seed`, and in a family named for the GUARD, no `_lr_guard` rule
# at all: 75.1 % of the family's entries were uncounted. ⭐ The digits are deliberately NOT
# repeated in this comment: a gate that both publishes a number and checks it is checking itself.
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
# derived share, and this gate holds them equal to the artifact — the second leg
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
# ⛔ ONE VALUE, NOT A PAIR, SINCE `ENGINE-UNIVERSAL-SERVICES.26`. The anchor was
# `share/blind-spot-factor`; the factor is retired because both of its terms were wrong (see the
# honest-limit block above). ⭐ A surface left on the old `2.741/8.9` form does not silently half-
# match: `2.741` is not followed by a closing backtick there, so the paragraph yields NO anchor and
# the call site reports it as MISSING. Failing loudly on the superseded form is the point.
SHARE_TUPLE_RE = re.compile(r"`(\d+\.\d+)`")

# ⛔ ONE PREDICATE, IMPORTED — NOT A SECOND SPELLING (`SV-CORPUS-GRAD.13c.2w`). The containment
# invariant below and the arm toolkit's `containment.py` both call
# `scripts/parse_cost_containment.py`. Coding it twice is exactly
# [[one-metric-name-two-predicates-is-a-contract-defect]], measured in this repository on
# `unreachable_rules` three days ago: one metric name, two implementations, both right about
# different populations, and a contract nobody could adjudicate.
sys.path.insert(0, os.path.join(ROOT, "scripts"))
try:
    import parse_cost_containment as CONTAINMENT
    CONTAINMENT_IMPORT_ERROR = None
except Exception as exc:  # pragma: no cover - a missing module is reported, never silently skipped
    CONTAINMENT, CONTAINMENT_IMPORT_ERROR = None, exc

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
        (f"x **{SHARE_ANCHOR_MARK} `2.741`** y", ["2.741"]),
        # the marker and the value wrapped onto separate lines of one paragraph
        (f"{SHARE_ANCHOR_MARK} —\nthe share is `2.741` today", ["2.741"]),
        # ⭐ the SAME digits with no marker are an era-dated citation, and must stay invisible
        ("the share was `0.681` before .21 corrected it", []),
        # a marker with no value is a missing anchor, which the call site treats as missing
        (f"{SHARE_ANCHOR_MARK} — pending re-derivation", []),
        # unbackticked digits are prose, not an anchor
        (f"{SHARE_ANCHOR_MARK} 2.741", []),
        # ⛔ THE SUPERSEDED PAIR FORM MUST NOT HALF-MATCH (`.26`). A surface still carrying
        # `2.741/8.9` yields nothing, so it is reported MISSING rather than silently agreeing on
        # its first component — the retired factor cannot ride along unnoticed.
        (f"{SHARE_ANCHOR_MARK} `2.741/8.9`", []),
        # two anchors, both seen — so disagreement fails rather than being averaged
        (f"{SHARE_ANCHOR_MARK} `2.741`\n{SHARE_ANCHOR_MARK} `9.999`", ["2.741", "9.999"]),
        # a paragraph WITHOUT the marker is not read even if it is adjacent to one
        (f"{SHARE_ANCHOR_MARK} `2.741`\n\nelsewhere `1.234` is quoted", ["2.741"]),
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
# ⛔⛔ THE GRAMMAR IS KEYED SEMANTICALLY, NOT BY BYTES (`ENGINE-UNIVERSAL-SERVICES.38`).
# The EBNF frontend strips comments, so a comment-only edit moved the file's sha while leaving
# `generated/systemverilog_parser.rs` — and therefore every binding counter — byte-identical.
# Measured 2026-08-20: that made THIS tier fail and BLOCKED EVERY COMMIT, for a change that
# provably cannot move a number. ⭐ And the byte row bought nothing: the parser is keyed by bytes
# just below, and any grammar change able to move a counter necessarily moves the parser — so the
# grammar row could only fire alongside it, or ALONE AND FALSELY.
# ⛔ ONE DEFINITION: this shells out to `check_baseline_identity.sh --digest`, which is also what
# BASELINE-IDENTITY uses, rather than becoming a third in-repo copy of the same hash.
if os.path.isfile(GRAMMAR_FILE):
    _d = subprocess.run([os.path.join(ROOT, "scripts", "check_baseline_identity.sh"),
                         "--digest", "ebnf_raw_ast",
                         os.path.relpath(GRAMMAR_FILE, ROOT)],
                        capture_output=True, text=True)
    if _d.returncode == 0 and re.fullmatch(r"[0-9a-f]{64}", _d.stdout.strip() or ""):
        live["grammar raw ast"] = _d.stdout.strip()
    else:
        unevaluated.append(f"the semantic (raw_ast) digest of {GRAMMAR_FILE} could not be derived "
                           f"— build it with `make -C rust ast_pipeline`")
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
derived_share = None
if os.path.isfile(FAMILY_SHARE):
    try:
        with open(FAMILY_SHARE, encoding="utf-8") as fh:
            fs = json.load(fh)
        derived_share = str(fs["corpus_family_share_pct"])
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


if derived_share:
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
                f"'{SHARE_ANCHOR_MARK}' and the value `{derived_share}` (corpus-entry share %), "
                f"so the share cannot be quoted here after it has gone stale. ⛔ If this surface "
                f"still carries the pre-`ENGINE-UNIVERSAL-SERVICES.26` pair form "
                f"`{derived_share}/<factor>`, that is what you are seeing: the blind-spot factor "
                f"is RETIRED — both of its terms were wrong — and the anchor is the share alone.")
            continue
        wrong = sorted(set(t for t in found if t != derived_share))
        if wrong:
            copublication_problem(
                surface,
                f"{surface} publishes LR-family share(s) {wrong} but the tracked derivation says "
                f"{derived_share}. Update the anchor, or re-derive if the tree moved.")

# ── TIER 1, arm 5: THE PROBE FINGERPRINT — which parser does the EXECUTABLE embed? ──────────────
#
# ⛔ THE ESCAPE HATCH MUST NOT REACH THE GATE. `PGEN_PARSE_COST_ALLOW_PROBE_MISMATCH=1` exists so
# an operator can deliberately measure an experimental arm; it is stripped from the environment
# here because this is the path whose output becomes the TRACKED reference, and an escape hatch
# that reaches the gate is not an escape hatch, it is a hole.
GATE_ENV = {k: v for k, v in os.environ.items()
            if k != "PGEN_PARSE_COST_ALLOW_PROBE_MISMATCH"}


def probe_fingerprint_check():
    """`(code, message)` — 0 matches / 1 differs / 3 NOT EVALUATED, with the instrument's own text."""
    proc = subprocess.run([sys.executable, INSTRUMENT, "--verify-probe-fingerprint"],
                          capture_output=True, text=True, env=GATE_ENV)
    return proc.returncode, (proc.stderr or proc.stdout).strip()


probe_fp_code, probe_fp_msg = probe_fingerprint_check()
if probe_fp_code == 1:
    # ⛔ A NOTE AND NOT A FAILURE, DELIBERATELY. Tier 1 computes no measurement, so a probe built
    # from another parser cannot mislead anything this tier reports — and failing every commit
    # for a stale untracked build artifact is how a gate teaches people to bypass it. It becomes
    # a hard refusal in tier 2, which is the tier that measures.
    # ⛔ The HEADLINE says "could not be confirmed", not "embeds a different parser". Arm 5 has two
    # distinct negative verdicts — the probe predates the flag, or it embeds another parser — and
    # they call for different acts. The instrument's own line, quoted below, says which.
    notes.append("the probe on disk could NOT be confirmed to embed the parser this baseline "
                 "pins. Tier 1 measures nothing, so nothing here is wrong because of it — but a "
                 "re-measure would be, and tier 2 will REFUSE until it is resolved "
                 "(ENGINE-UNIVERSAL-SERVICES.24):\n"
                 + "".join(f"        {l}\n" for l in probe_fp_msg.splitlines()[:8]))
elif probe_fp_code == 3:
    unevaluated.append("the probe fingerprint could not be checked: no parseability_probe is "
                       "built (it is an untracked build artifact)")
elif probe_fp_code != 0:
    fail(f"`{INSTRUMENT} --verify-probe-fingerprint` exited {probe_fp_code}, which is not one of "
         f"its declared codes (0 matches / 1 differs / 3 not evaluated):\n"
         + "".join(f"        {l}\n" for l in probe_fp_msg.splitlines()[:8]))

# ── TIER 2: the ratchet ─────────────────────────────────────────────────────────────────────────
BINDING = ("entries", "committed", "memo_hits")

# ── ENGINE-UNIVERSAL-SERVICES.36 — the TYPED ACCEPTANCE of an attributed rise ────────────────────
#
# ⛔ This gate's own breach message has always ended "… or record it as irreducible with the
# measurement that proves it (`.20` acceptance)", and there was nowhere to record it: every rise
# failed, and `PGEN_PARSE_COST_REBASELINE=1` refuses while a failure stands. So the only outcomes
# were "eliminate the cost" and "bypass the gate" — the same asymmetry `SV-CORPUS-GRAD.13c.2k`
# found in the repro manifest, where one direction of a claim was expressible and the other was not.
# An instrument that leaves a legitimate outcome unrepresentable teaches people to route around it.
#
# ⛔⛔ AN ACCEPTANCE IS NOT A WAIVER. `accepted_rises.tsv` names the EXACT from/to integers plus an
# INVARIANT that is CODE HERE, and the invariant is re-evaluated against this run's own numbers. A
# row therefore cannot cover a different rise, cannot cover a rise of a different SHAPE without a
# code change carrying its own leaf, and cannot outlive its own justification: if the invariant
# stops holding, the gate fails and says so.
ACCEPTED_RISES = f"{ART}/accepted_rises.tsv"


def _pure_memo_lookups(base_tot, new_tot, ctx):
    """The rise is entirely MEMO LOOKUPS: every added rule entry was served from the memo table
    and none of them committed.

    ⭐ Why this is a real invariant and not a comfortable reading: `entries` counts rule-method
    ENTRIES, and an entry answered by the memo table is counted identically to one that parses.
    So `delta_entries == delta_memo_hits` with `delta_committed == 0` says, exactly, that the
    change asked N more questions the parser had already answered and did no new work. That is
    the converse of this doctrine's founding lesson (a deterministic counter cannot see a
    per-ENTRY cost rise): it also cannot see that a rise is pure cache traffic.
    """
    d_entries = new_tot["entries"] - base_tot["entries"]
    d_memo = new_tot["memo_hits"] - base_tot["memo_hits"]
    d_committed = new_tot["committed"] - base_tot["committed"]
    if d_committed != 0:
        return False, f"committed moved by {d_committed:+,} — real work changed, not just lookups"
    if d_entries != d_memo:
        return False, (f"entries moved {d_entries:+,} but memo hits moved {d_memo:+,} — "
                       f"{d_entries - d_memo:+,} of the added entries were NOT memo hits")
    return True, (f"entries {d_entries:+,} == memo hits {d_memo:+,}, committed flat — "
                  f"{d_entries:,} more cached lookups, zero new parsing work")


def _unmatched_terminal_alternatives(base_tot, new_tot, ctx):
    """The rise is entirely NEW TERMINAL ALTERNATIVES that never matched: each costs its own rule
    body plus one MEMOIZED `trivia` lookup at the same position, and none of them committed.

    ⭐ Why the arithmetic is structural and not numerology. Every keyword terminal in this
    repository's grammars is spelled `kw_X := trivia /re/`, so ADMITTING one more alternative into
    an ordered choice costs, at each position the choice is attempted, exactly two entries: the
    terminal's own body (which always runs — a terminal is tried at most once per position, so it
    takes no memo hit) and one entry for the shared `trivia` prefix, which the FIRST alternative
    already resolved and which is therefore a memo HIT. N alternatives over P positions is
    `2*N*P` entries and `N*P` memo hits. Hence `delta_entries == 2 * delta_memo_hits`.

    ⛔ AND `committed` MUST NOT MOVE, which is what stops this being a licence for speculation:
    it says the added alternatives never actually matched anything in the sample. The day one of
    them does, `committed` moves, this invariant FAILS, and the acceptance correctly stops
    covering the rise — the row cannot outlive its own justification.

    ⚠️ HONEST BOUND: this explains the SHAPE of a rise, not that the rise was unavoidable. It is
    the right acceptance only when the alternatives are required for the grammar to derive its
    standard's language at all, and when eliminating them would change what the parser accepts.
    """
    d_entries = new_tot["entries"] - base_tot["entries"]
    d_memo = new_tot["memo_hits"] - base_tot["memo_hits"]
    d_committed = new_tot["committed"] - base_tot["committed"]
    if d_committed != 0:
        return False, (f"committed moved by {d_committed:+,} — one of the added alternatives now "
                       f"MATCHES, so this is no longer unmatched speculation")
    if d_memo <= 0:
        return False, f"memo hits moved {d_memo:+,} — a terminal alternative always adds a memoized `trivia` lookup"
    if d_entries != 2 * d_memo:
        return False, (f"entries moved {d_entries:+,} but 2 x memo hits is {2 * d_memo:+,} — the rise "
                       f"is not one body plus one memoized trivia lookup per added alternative")
    return True, (f"entries {d_entries:+,} == 2 x memo hits {d_memo:+,}, committed flat — "
                  f"{d_memo:,} added terminal-alternative attempts, none of which matched")


def _unmatched_lookahead_terminals(base_tot, new_tot, ctx):
    """The rise is entirely NEGATIVE-LOOKAHEAD TERMINALS: a `!kw` guard added to a sequence, which
    at each position it is reached costs the terminal's own body plus one MEMOIZED `trivia` lookup
    and commits nothing.

    ⭐ The arithmetic is the same as `_unmatched_terminal_alternatives` and for the same structural
    reason — every terminal here is `kw_X := trivia /re/`, so one attempt is two entries (the body,
    which always runs, and the shared `trivia` prefix an earlier element already resolved, hence a
    memo HIT). N guards over P reached positions is `2*N*P` entries and `N*P` memo hits.

    ⛔ IT IS A SEPARATE INVARIANT BECAUSE `committed == 0` MEANS SOMETHING WEAKER HERE, and importing
    the other one's name would have imported a guarantee that does not hold. For an added ALTERNATIVE,
    `Δcommitted == 0` says the alternative never matched — the day it does, the acceptance correctly
    expires. A negative lookahead NEVER commits its subject, matched or not: the subject is parsed
    speculatively and rolled back either way. So here `Δcommitted == 0` says something different and
    still worth having: **no rule anywhere committed more or fewer frames**, i.e. the guard changed
    what the parser REFUSES without redirecting the search — which is the failure mode
    [[a-strictness-fix-redirects-the-search-not-just-the-predicate]] names, where a rejection buys
    more speculation elsewhere. That is the property being asserted, and it is asserted about the
    whole run rather than about the guard.

    ⚠️ HONEST BOUND, stated rather than implied: like its sibling this explains the SHAPE of a rise,
    not that the rise was unavoidable. It is the right acceptance only when the guard is what makes
    the grammar derive its standard's language — removing it must be shown to reintroduce a measured
    defect — and when no cheaper spelling exists. ⛔ It cannot see a rise CONFINED to the guard's own
    sub-graph versus one spread across the grammar; three totals cannot express that distinction.
    The per-rule predicate that CAN is `contained_in_introduced_subgraph` below
    (`SV-CORPUS-GRAD.13c.2w`), which reads `rule_costs.tsv` and the measured grammar's own
    reference graph rather than three sums.
    """
    d_entries = new_tot["entries"] - base_tot["entries"]
    d_memo = new_tot["memo_hits"] - base_tot["memo_hits"]
    d_committed = new_tot["committed"] - base_tot["committed"]
    if d_committed != 0:
        return False, (f"committed moved by {d_committed:+,} — the guard REDIRECTED THE SEARCH, "
                       f"which is the one thing this acceptance asserts it did not")
    if d_memo <= 0:
        return False, f"memo hits moved {d_memo:+,} — a lookahead terminal always adds a memoized `trivia` lookup"
    if d_entries != 2 * d_memo:
        return False, (f"entries moved {d_entries:+,} but 2 x memo hits is {2 * d_memo:+,} — the rise "
                       f"is not one body plus one memoized trivia lookup per guard attempt")
    return True, (f"entries {d_entries:+,} == 2 x memo hits {d_memo:+,}, committed flat — "
                  f"{d_memo:,} guard attempts, and no rule committed a different number of frames")


def _contained_in_introduced_subgraph(base_tot, new_tot, ctx):
    """The rise is CONFINED to the sub-graph the change introduced: no accepted derivation got
    dearer, no rule lost entries, and every rule whose entries rose is reachable — in the measured
    GRAMMAR's own reference graph — from one of the rules the acceptance row declares.

    ⛔⛔ WHY IT IS NOT A FOURTH ARITHMETIC IDENTITY. The three invariants above are exact identities
    over `entries`/`memo_hits`/`committed`. The rise this one exists for — `SV-0065`, restoring the
    `randomize_call` alternative IEEE 1800 A.8.2 gives `primary`, without which
    `std::randomize(a,b) with { … }` is unreachable from every expression — measures
    `Δentries +1,917,021`, `Δmemo +1,012,779`, `Δcommitted 0`, i.e. `Δentries/Δmemo = 1.893`. No
    coded identity holds and a fourth identity over the same three numbers would be numerology:
    **three totals cannot distinguish "the added alternative speculates inside its own sub-graph"
    from "the parser now speculates everywhere", and that distinction is the whole question.**

    ⭐ THE EVIDENCE IS PER-RULE AND WAS ALREADY BEING THROWN AWAY. `.13c.2w` (a) made the
    instrument keep it (`rule_costs.tsv`); (b) freezes the reference graph beside the numbers, from
    the grammar the run measured — never from `generated/systemverilog.json`, a floating build
    artifact whose staleness produced a containment verdict computed against the WRONG arm in
    `.13c.2k`.

    ⭐ THE SCOPE LIVES IN THE ROW, so a row cannot silently widen it: `introduced` is a 7th column
    of `accepted_rises.tsv`, the acceptance covers one exact from/to, and this predicate is
    re-derived on every re-measure. Perturbing the declared set changes the verdict, which is what
    the probe's arms fire.

    ⚠️⚠️ HONEST BOUND, IN THE INVARIANT'S OWN DOCSTRING BECAUSE THAT IS WHERE IT WILL BE READ:
    containment says the rise is CONFINED to the construct that caused it. It does NOT say the rise
    was UNAVOIDABLE. `SV-CORPUS-GRAD.13c.2k` is the standing proof that a contained-looking cost can
    still have a strictly cheaper spelling, and no predicate over totals or per-rule counts can see
    that — only another ARM can. So this is the right acceptance exactly when the added construct is
    required for the grammar to derive its standard's language AND the cheaper spellings have been
    built and measured.
    """
    if CONTAINMENT is None:
        raise RuntimeError(f"scripts/parse_cost_containment.py could not be imported "
                           f"({CONTAINMENT_IMPORT_ERROR}), so this acceptance cannot be evaluated")
    graph_path = ctx["graph"]
    if not os.path.isfile(graph_path):
        raise RuntimeError(f"the measured grammar's reference graph is missing "
                           f"({os.path.relpath(graph_path, ROOT)}) — re-measure with an instrument "
                           f"that emits it")
    with open(graph_path, encoding="utf-8") as fh:
        graph = json.load(fh)
    # ⛔ THE GRAPH MUST DESCRIBE THE GRAMMAR THAT WAS MEASURED. It carries the `ebnf_raw_ast`
    # digest tier 1 re-hashes, so this is a re-derivation and not a trusted label.
    if ctx["grammar_digest"] and graph.get("grammar_raw_ast_sha256") != ctx["grammar_digest"]:
        raise RuntimeError(
            f"the reference graph was derived from grammar raw_ast "
            f"`{str(graph.get('grammar_raw_ast_sha256'))[:16]}…` but this tree's is "
            f"`{ctx['grammar_digest'][:16]}…` — a containment verdict computed against the wrong "
            f"grammar is the `.13c.2k` defect this leaf exists to close")
    base_rules = CONTAINMENT.read_rule_costs(ctx["base_rule_costs"])
    new_rules = CONTAINMENT.read_rule_costs(ctx["new_rule_costs"])
    d_committed = new_tot["committed"] - base_tot["committed"]
    holds, detail, _facts = CONTAINMENT.containment(
        base_rules, new_rules, graph["edges"], ctx["introduced"], d_committed)
    return holds, detail


INVARIANTS = {
    "pure_memo_lookups": _pure_memo_lookups,
    "unmatched_terminal_alternatives": _unmatched_terminal_alternatives,
    "unmatched_lookahead_terminals": _unmatched_lookahead_terminals,
    "contained_in_introduced_subgraph": _contained_in_introduced_subgraph,
}

# ⛔ WHICH INVARIANTS TAKE A SCOPE, AND THE CHECK IS TWO-SIDED. A containment row with no
# `introduced` set cannot be evaluated; a row naming a set for an invariant that ignores it reads as
# a scoped acceptance and is not one. Both REFUSE, so the column can never become decoration.
INVARIANTS_TAKING_INTRODUCED = {"contained_in_introduced_subgraph"}
NO_INTRODUCED = "-"


def load_accepted_rises():
    """Rows of `accepted_rises.tsv`, keyed by (metric, from, to). Refuses on an unknown invariant
    or a malformed row rather than skipping it — a row this gate cannot evaluate must not read as
    an acceptance."""
    path = os.path.join(ROOT, ACCEPTED_RISES)
    if not os.path.exists(path):
        return {}, []
    rows, problems = {}, []
    with open(path, encoding="utf-8") as fh:
        header = None
        for lineno, raw in enumerate(fh, 1):
            line = raw.rstrip("\n")
            if not line.strip() or line.lstrip().startswith("#"):
                continue
            fields = line.split("\t")
            if header is None:
                header = fields
                want = ["metric", "from", "to", "invariant", "leaf", "why", "introduced"]
                if header[:7] != want:
                    problems.append(f"{ACCEPTED_RISES}:{lineno} unexpected header {header[:7]}, "
                                    f"expected {want}")
                    return {}, problems
                continue
            if len(fields) < 7:
                problems.append(f"{ACCEPTED_RISES}:{lineno} has {len(fields)} field(s), expected 7")
                continue
            metric, frm, to, invariant, leaf, why, introduced = fields[:7]
            if metric not in BINDING:
                problems.append(f"{ACCEPTED_RISES}:{lineno} unknown metric `{metric}`")
                continue
            if invariant not in INVARIANTS:
                problems.append(f"{ACCEPTED_RISES}:{lineno} names invariant `{invariant}`, which "
                                f"is not coded in this gate — accepting a rise of a NEW SHAPE is a "
                                f"code change with its own task leaf")
                continue
            if not leaf.strip():
                problems.append(f"{ACCEPTED_RISES}:{lineno} names no owning leaf")
                continue
            # ⛔ THE SCOPE COLUMN IS TWO-SIDED (`SV-CORPUS-GRAD.13c.2w` (c)). A scoped invariant
            # with no scope cannot be evaluated, and an unscoped invariant carrying one reads as a
            # narrower acceptance than the gate will actually apply. Both refuse.
            scope = [] if introduced.strip() in ("", NO_INTRODUCED) else introduced.split()
            if invariant in INVARIANTS_TAKING_INTRODUCED and not scope:
                problems.append(f"{ACCEPTED_RISES}:{lineno} accepts under `{invariant}`, which is "
                                f"scoped, but declares no `introduced` rule set — the scope of the "
                                f"rise would be whatever this gate guessed")
                continue
            if invariant not in INVARIANTS_TAKING_INTRODUCED and scope:
                problems.append(f"{ACCEPTED_RISES}:{lineno} declares introduced={scope} for "
                                f"`{invariant}`, which ignores it — a row must not carry a scope "
                                f"its invariant never applies (write `{NO_INTRODUCED}`)")
                continue
            try:
                rows[(metric, int(frm), int(to))] = (invariant, leaf, why, scope)
            except ValueError:
                problems.append(f"{ACCEPTED_RISES}:{lineno} from/to are not integers")
    return rows, problems



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
    elif probe_fp_code == 1:
        # ⛔ REFUSE BEFORE MEASURING, not after (`ENGINE-UNIVERSAL-SERVICES.24` (b)). Running the
        # instrument first and judging the numbers afterwards is how the wrong binary got measured
        # in the first place: the numbers look plausible, and on this ratchet a FALL reads as an
        # improvement worth promoting.
        fail("tier 2 REFUSES to re-measure: the probe that would produce the numbers cannot be "
             "confirmed to embed the parser this baseline pins.\n"
             + "".join(f"        {l}\n" for l in probe_fp_msg.splitlines()[:8])
             + "      Rebuild the probe and re-run. A measurement taken with the wrong binary "
               "cannot be compared with this baseline, and a FALL here would read as an "
               "improvement worth promoting.")
    elif probe_fp_code == 3:
        unevaluated.append("tier 2 (the ratchet) could not run: no parseability_probe is built")
    else:
        scratch = os.path.join(ROOT, STATE, "scratch")
        shutil.rmtree(scratch, ignore_errors=True)
        os.makedirs(scratch, exist_ok=True)
        print("parse-cost-ratchet: re-measuring the pinned sample (~2.5 min) ...", file=sys.stderr)
        proc = subprocess.run([sys.executable, INSTRUMENT, "--outdir", scratch],
                              capture_output=True, text=True, env=GATE_ENV)
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

        accepted, accept_problems = load_accepted_rises()
        for problem in accept_problems:
            fail(f"the accepted-rise record is unusable: {problem}")

        for k in BINDING:
            b, n = base_tot[k], new_tot[k]
            if n > b:
                row = accepted.get((k, b, n))
                if row is None:
                    fail(f"BINDING metric `{k}` ROSE {b:,} -> {n:,} (+{100.0 * (n - b) / b:.2f} %). "
                         f"Costs are REJECTED, not traded — attribute it and eliminate it, or record "
                         f"it as irreducible with the measurement that proves it (`.20` acceptance): "
                         f"one row in {ACCEPTED_RISES} naming this exact from/to plus a coded "
                         f"invariant that explains it.")
                    continue
                invariant, leaf, why, scope = row
                ctx = {"introduced": scope,
                       "base_rule_costs": os.path.join(ROOT, ART, "rule_costs.tsv"),
                       "new_rule_costs": os.path.join(scratch, "rule_costs.tsv"),
                       "graph": os.path.join(scratch, "rule_graph.json"),
                       "grammar_digest": live.get("grammar raw ast")}
                try:
                    holds, detail = INVARIANTS[invariant](base_tot, new_tot, ctx)
                except Exception as exc:
                    # ⛔ AN UN-EVALUABLE ACCEPTANCE IS A BREACH, NEVER A PASS. The alternative is a
                    # measured rise that nobody checked reading GREEN, which is the exact shape
                    # this whole doctrine was founded to remove.
                    fail(f"BINDING metric `{k}` ROSE {b:,} -> {n:,} and {ACCEPTED_RISES} accepts "
                         f"it under `{invariant}` ({leaf}) — but that invariant could NOT BE "
                         f"EVALUATED on this measurement: {exc}")
                    continue
                if not holds:
                    fail(f"BINDING metric `{k}` ROSE {b:,} -> {n:,} and {ACCEPTED_RISES} accepts "
                         f"it under `{invariant}` ({leaf}) — but that invariant NO LONGER HOLDS on "
                         f"this measurement: {detail}. The acceptance's own justification is gone; "
                         f"re-attribute the rise, do not re-word the row.")
                    continue
                notes.append(f"BINDING metric `{k}` ROSE {b:,} -> {n:,} "
                             f"(+{100.0 * (n - b) / b:.2f} %) — ACCEPTED as attributed by "
                             f"{ACCEPTED_RISES} under `{invariant}` ({leaf})"
                             + (f", scoped to `{' '.join(scope)}`" if scope else "")
                             + f", and the invariant was RE-DERIVED on this run: {detail}. "
                               f"Why: {why}")
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
            # ⛔ `rule_graph.json` is DELIBERATELY NOT PROMOTED. It is an exact function of a
            # tracked grammar, and a field a command answers exactly is looked up, never stored
            # (`docs/DERIVED_STATE_CONTAINMENT.md` R1/R3). `rule_costs.tsv` IS promoted: it is a
            # measurement, and nothing but a re-run can produce it.
            for name in ("entries.tsv", "rule_costs.tsv", "cost.md", "advisory.json"):
                shutil.copyfile(os.path.join(scratch, name), os.path.join(ROOT, ART, name))
            print(f"parse-cost-ratchet: REBASELINED — {ART} now records this measurement.",
                  file=sys.stderr)
elif not stale:
    # ⛔ THIS NOTE HAS BEEN WRONG ONCE AND NARROW TWICE, AND THE HISTORY IS THE ARGUMENT FOR
    # KEEPING IT EXACT. It originally read "…so the measurement cannot have moved" — a claim about
    # the whole PIPELINE made by a table that pins four INPUTS. `.20` slice 4 demonstrated the gap
    # live: a release probe built from a guard-suppressed arm, this note printing verbatim, and
    # `nm … | grep -c _lr_guard` reading 0 against a pinned parser declaring 6.
    # `.24` (c) narrowed it to the inputs and NAMED the unpinned executable; `.24` (b) then closed
    # the gap, so the sentence below is the first version that describes a property the gate
    # actually holds — the probe is now fingerprinted (arm 5) and tier 2 refuses on a mismatch.
    # ⛔ It still does not claim the FUSED graph: the binding counters observe the PROTOCOL graph,
    # which is a limit of the metric and not of this check.
    probe_clause = {
        0: ("and the probe that would take the measurement provably embeds that same parser "
            "(arm 5, ENGINE-UNIVERSAL-SERVICES.24)"),
        1: ("⛔ but the probe on disk could NOT be confirmed to embed it — see the note above; "
            "tier 2 will REFUSE until that is resolved"),
        3: ("⚠️ the probe is not built, so arm 5 could not confirm which parser a re-measure "
            "would use; tier 2 would report the same"),
    }.get(probe_fp_code, "⚠️ arm 5 did not return a verdict this reader knows")
    notes.append(f"tier 2 (the re-measure) did not run, and did not need to: every INPUT the "
                 f"binding metric depends on is byte-identical to the baseline's, {probe_clause}. "
                 f"Force a re-measure with PGEN_PARSE_COST_REMEASURE=1.")

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
