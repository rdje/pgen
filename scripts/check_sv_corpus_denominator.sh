#!/usr/bin/env bash
# scripts/check_sv_corpus_denominator.sh
#
# DOCTRINE `SV-CORPUS-DENOMINATOR` (evidence + structural) — SV-CORPUS-GRAD.13b.
#
#   A defect bar without its denominator beside it is not a claim about the corpus.
#
# The SV graduation bar counts DIVERGENCES (today 318). That answers *how many defects do we know
# about* and is silent on *what fraction of the corpus was asked a question it could answer* — and
# the honest answer is 46.3 % adjudicated, with 4 398 rows (26.9 %) failing inside a deferral
# nobody has looked at (`SV-CORPUS-GRAD.13`/`.13a`). The same bar over a fully adjudicated corpus
# would be a different claim, and only that one supports signoff.
#
# ⭐ WHY THIS IS A DOCTRINE AND NOT A NOTE — IT IS A MEASURED ROT, NOT A HYPOTHETICAL.
# The census instrument landed in `PGEN-SV-CORPUS-GRAD-0202` and its artifact was committed. One
# day later `.13a` re-ran it and the tracked artifact was already WRONG: it published
# `match 5 804` / bar **319** while the manifest at HEAD said `5 805` / **318** (the `.12c.1`
# Latin-1 decoder fix moved one row). Nothing in the repository could see it:
#
#   $ git ls-files 'scripts/*.sh' 'rust/scripts/*.sh' '.github/workflows/*.yml' 'rust/Makefile' \
#       '.githooks/*' | xargs grep -ln corpus_verdict_coverage
#   (none — the instrument was invoked by NOTHING)
#
# ⇒ a number written into a tracked file with no gate behind it is wrong the next time it is
# quoted. This check re-derives it instead.
#
# WHAT IT ASSERTS
#   (1) FRESHNESS — the tracked census artifacts are byte-identical to a fresh re-run of the
#       instrument. ⛔ The re-run goes to a SCRATCH directory: a check that regenerates the file
#       it then compares always passes.
#   (2) CO-PUBLICATION — every designated LIVE surface carries the derived
#       `adjudicated/routed/no-verdict/dark/axis-2-bar` tuple, so the bar cannot be re-published
#       without its denominator moving with it. A stale anchor fails; so does a bar that moved.
#
# ⛔ THE CLASS→BUCKET MAP IS NOT DUPLICATED HERE. This check INVOKES the instrument rather than
# re-implementing its bucketing — a second copy is a second thing to drift, and the instrument
# already refuses on an adjudication class it has no bucket for.
#
# ⚠️ HONEST LIMIT, stated rather than hidden (DOCTRINE_ENFORCEMENT.md §3). This re-derives the
# published NUMBERS from tracked inputs; it does not re-adjudicate the corpus. Whether a row's
# EXPECTED verdict is right is `.2`/`.3`'s subject, and whether a deferral is honest is `.13c`'s.
# ⚠️ The vendored corpora are git SUBMODULES, so a hosted checkout without them cannot run the
# DARK-half backtick census. In that case the corpus-dependent leg is reported as NOT EVALUATED —
# loudly, never as a pass (`docs/decisions/feedback_a_check_that_cannot_run_must_say_so.md` is the
# founding principle of this tree).
#
# ANCHOR SPELLING (one home): a live surface carries a line containing
#   Live verdict-coverage tuple … `A/R/N/D/B`
# The MARKER is the discriminator, exactly as `check_regex_oracle_anchor_sync.sh` uses bold: an
# era-dated citation of an old number in prose is history and is deliberately out of scope
# (supersede-don't-mutate).
#
# CONTRACT (DOCTRINE_ENFORCEMENT.md §4): exit code is the verdict; explains on stderr;
# deterministic; does not mutate tracked state; path-agnostic; fast (~1 s, no cargo, no network).
#
# Usage:
#   bash scripts/check_sv_corpus_denominator.sh
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$ROOT" || exit 1

exec python3 - "$@" <<'PYEOF'
import os, re, shutil, subprocess, sys

ROOT = os.getcwd()
INSTRUMENT = "stimuli/sv/corpus_verdict_coverage.py"
ART = "docs/tasks/artifacts/sv_corpus_grad/verdict_coverage"
SCRATCH = "rust/target/sv_corpus_denominator_gate"
CORPUS_PROBE = "stimuli/sv/subs/opentitan"
LIVE_SURFACES = ["docs/book/src/grammar-wellformedness.md"]

ANCHOR_MARK = "Live verdict-coverage tuple"
TUPLE_RE = re.compile(r"\b\d+/\d+/\d+/\d+/\d+\b")

failures = []


def note(msg):
    failures.append(msg)


def anchor_tuples(text):
    """Every tuple inside a PARAGRAPH that carries the live-anchor MARKER. A tuple elsewhere in
    the document is an era-dated citation and is deliberately invisible here.

    ⚠️ Paragraph-scoped, not line-scoped, and that is not cosmetic: the book wraps its prose, so
    the marker and the tuple legitimately land on different lines. A line-scoped reader read the
    anchor as MISSING while it sat two words away — measured on the first run of this check.
    ⇒ ALL tuples in the marker paragraph are returned, so an unrelated number sharing that
    paragraph fails loudly instead of being silently averaged into agreement."""
    out = []
    for para in re.split(r"\n[ \t]*\n", text):
        if ANCHOR_MARK not in para:
            continue
        out.extend(m.group(0) for m in TUPLE_RE.finditer(para))
    return out


# --- GROUND TRUTH, every invocation (microseconds) ----------------------------------------------
# An instrument with no ground truth is a confident guess
# (docs/decisions/feedback_instrument_needs_ground_truth.md). Both the positive control AND the
# negatives are pinned, and a MISS refuses (exit 2) rather than reporting a clean tree.
def self_check():
    cases = [
        # (text, expected)
        ("x **Live verdict-coverage tuple — `1/2/3/4/5`** y", ["1/2/3/4/5"]),
        # the same digits with no marker = history, must NOT be picked up
        ("the bar rose to `1/2/3/4/5` in 2026", []),
        # marker with no tuple = no anchor (the call site treats that as missing, which it is)
        ("Live verdict-coverage tuple — pending", []),
        # a 4-field tuple is not this tuple
        ("Live verdict-coverage tuple — `1/2/3/4`", []),
        # two anchors, both seen (so disagreement is detectable rather than averaged)
        ("Live verdict-coverage tuple `1/2/3/4/5`\nLive verdict-coverage tuple `9/9/9/9/9`",
         ["1/2/3/4/5", "9/9/9/9/9"]),
    ]
    misses = 0
    for text, want in cases:
        got = anchor_tuples(text)
        if got != want:
            print(f"sv-corpus-denominator: CONTROL MISSED: {text!r} want={want} got={got}",
                  file=sys.stderr)
            misses += 1
    if misses:
        print(f"sv-corpus-denominator: the anchor reader does not discriminate "
              f"({misses} control(s) missed); refusing", file=sys.stderr)
        sys.exit(2)


self_check()

# --- preconditions ------------------------------------------------------------------------------
for p in (INSTRUMENT, f"{ART}/coverage.tsv", f"{ART}/strata.tsv", f"{ART}/coverage.md"):
    if not os.path.isfile(p):
        print(f"sv-corpus-denominator: FAIL — required file missing: {p}", file=sys.stderr)
        sys.exit(1)

# The vendored corpora are submodules; without them the backtick census cannot run.
corpus_present = os.path.isdir(CORPUS_PROBE) and bool(os.listdir(CORPUS_PROBE))

# --- leg 1: re-run the instrument into a SCRATCH directory --------------------------------------
shutil.rmtree(SCRATCH, ignore_errors=True)
os.makedirs(SCRATCH, exist_ok=True)
cmd = [sys.executable, INSTRUMENT, "--outdir", SCRATCH]
if not corpus_present:
    cmd.append("--no-corpus-scan")
proc = subprocess.run(cmd, capture_output=True, text=True)
if proc.returncode != 0:
    print("sv-corpus-denominator: FAIL — the census instrument refused to run:", file=sys.stderr)
    print((proc.stderr or proc.stdout).strip(), file=sys.stderr)
    sys.exit(1)


def read_bytes(p):
    with open(p, "rb") as fh:
        return fh.read()


def strata_without_tick_column(raw):
    """strata.tsv minus its corpus-dependent last column, so the freshness comparison still
    binds when the submodules are absent instead of being skipped wholesale."""
    out = []
    for line in raw.decode("utf-8").splitlines():
        out.append("\t".join(line.split("\t")[:5]))
    return "\n".join(out)


# coverage.tsv is scan-independent, so it is compared in BOTH modes.
compared, unevaluated = [], []
for name in ("coverage.tsv",):
    if read_bytes(f"{ART}/{name}") != read_bytes(f"{SCRATCH}/{name}"):
        note(f"the tracked {ART}/{name} is NOT what the instrument produces today — it is STALE. "
             f"Re-run: python3 {INSTRUMENT}")
    compared.append(name)

if corpus_present:
    for name in ("coverage.md", "strata.tsv"):
        if read_bytes(f"{ART}/{name}") != read_bytes(f"{SCRATCH}/{name}"):
            note(f"the tracked {ART}/{name} is NOT what the instrument produces today — it is "
                 f"STALE. Re-run: python3 {INSTRUMENT}")
        compared.append(name)
else:
    if (strata_without_tick_column(read_bytes(f"{ART}/strata.tsv"))
            != strata_without_tick_column(read_bytes(f"{SCRATCH}/strata.tsv"))):
        note(f"the tracked {ART}/strata.tsv disagrees with the instrument on its "
             f"corpus-INDEPENDENT columns — it is STALE. Re-run: python3 {INSTRUMENT}")
    compared.append("strata.tsv (columns 1-5 only)")
    unevaluated.append("coverage.md and strata.tsv's DARK-half backtick column: the vendored "
                       f"corpora are submodules and {CORPUS_PROBE} is empty here")

# --- derive the published tuple from the FRESH artifacts ----------------------------------------
buckets, classes = {}, {}
with open(f"{SCRATCH}/coverage.tsv", encoding="utf-8") as fh:
    fh.readline()
    for line in fh:
        if not line.strip():
            continue
        bucket, cls, rows, _pct = line.rstrip("\n").split("\t")
        buckets[bucket] = buckets.get(bucket, 0) + int(rows)
        classes[cls] = int(rows)

dark = 0
with open(f"{SCRATCH}/strata.tsv", encoding="utf-8") as fh:
    fh.readline()
    for line in fh:
        if not line.strip():
            continue
        dark += int(line.rstrip("\n").split("\t")[4])

bar = (classes.get("divergence:unexplained_rejects_valid", 0)
       + classes.get("divergence:unexplained_accepts_invalid", 0))
derived = "/".join(str(n) for n in (buckets.get("ADJUDICATED", 0), buckets.get("ROUTED", 0),
                                    buckets.get("NO VERDICT", 0), dark, bar))

# --- leg 2: every LIVE surface must carry exactly that tuple ------------------------------------
for surface in LIVE_SURFACES:
    if not os.path.isfile(surface):
        note(f"designated live surface missing: {surface}")
        continue
    with open(surface, encoding="utf-8") as fh:
        found = anchor_tuples(fh.read())
    if not found:
        note(f"{surface} carries NO live denominator anchor. Add a line containing "
             f"'{ANCHOR_MARK}' and the tuple `{derived}` "
             "(adjudicated/routed/no-verdict/dark/axis-2-bar) so the bar is never published "
             "without its denominator.")
        continue
    wrong = sorted(set(t for t in found if t != derived))
    if wrong:
        note(f"{surface} publishes verdict-coverage tuple(s) {wrong} but the census derives "
             f"{derived}. The bar and its denominator move TOGETHER — update the anchor.")

# --- the bar's TRUST BOUND is one link away, and the link is CHECKED ------------------------------
# ⛔ CORPUS-KEY-AUDIT.2(b), decided 2026-08-24. This doctrine holds the bar equal to its DENOMINATOR.
# One layer under that sits a different question — what the denominator's expectations REST ON —
# and the answer is uncomfortable: 93.8 % of the SV answer key rests on evidence OUTSIDE the
# standard, both non-clause classes having already produced a measured defect. ⭐ The tuple is NOT
# widened to carry that number: several live surfaces are held equal to it, so a sixth field is a
# contract change for every one of them. Naming the artifact costs nothing and puts a reader of the
# bar one link from its bound. ⛔ The pointer is EXISTENCE-CHECKED rather than printed blind — an
# unchecked path in a passing gate's own success line is how a reference rots into a dangling one,
# which is the whole family of defect this repository's currency doctrines exist to stop.
TRUST_BOUND = "docs/tasks/artifacts/corpus_key_audit/provenance.md"
if not os.path.isfile(os.path.join(ROOT, TRUST_BOUND)):
    note(f"the bar's trust-bound artifact is missing: {TRUST_BOUND}. The denominator says how much "
         "of the corpus was ASKED a question; that artifact says how much of the answer key rests "
         "on evidence outside the standard. Re-derive it: python3 "
         "docs/tasks/artifacts/corpus_key_audit/key_provenance_census.py "
         "(CORPUS-KEY-INTEGRITY tier A4 holds it current).")

# --- verdict -------------------------------------------------------------------------------------
if unevaluated:
    for u in unevaluated:
        print(f"sv-corpus-denominator: NOT EVALUATED — {u}", file=sys.stderr)

if failures:
    print(f"sv-corpus-denominator: {len(failures)} breach(es):", file=sys.stderr)
    for f in failures:
        print(f"  ✗ {f}", file=sys.stderr)
    sys.exit(1)

print(f"sv-corpus-denominator: OK (tuple {derived} = adjudicated/routed/no-verdict/dark/bar; "
      f"artifacts fresh: {', '.join(compared)}; trust bound: {TRUST_BOUND})")
PYEOF
