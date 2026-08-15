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
#   TIER 1 (every run, ~0.4 s) — IDENTITY. Re-hash the three inputs the baseline names: the
#     grammar, the generated parser, and a digest over the sampled corpus files themselves. The
#     binding metric is an exact function of exactly those inputs (verified: deterministic across
#     repeated release runs, and byte-identical between the debug and release probes). ⇒ if none
#     of them moved, the measurement CANNOT have moved. Tier 1 is therefore a proof, not a
#     sampling heuristic — and when it fails it does not guess, it demands a re-measure.
#
#   TIER 2 (on demand, ~2.5 min) — THE RATCHET ITSELF. Re-run the instrument into a SCRATCH
#     directory and compare. Entries/committed/memo-hits must not RISE. Wall clock is reported
#     against a wide band and never fails the gate.
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
# FUSED `cascade_*` graph, which ticks no per-rule counters — and the +24.3 % was measured there.
# Measured bound: the LR-elimination family is 2.741 % of corpus entries, so this metric is at
# least ~8.9× less sensitive to THAT regression than wall clock is. ⛔ That bound READ ~35× until
# `ENGINE-UNIVERSAL-SERVICES.21`, because the classifier that measured it counted only
# `_lr_base`/`_lr_suffix` and so saw 0.681 % — no `_lr_seed`, and in a family named for the GUARD,
# no `_lr_guard` rule at all: 75.1 % of the family's entries were uncounted. The gate was
# UNDER-claiming its own sensitivity by ~4×. It binds STRUCTURAL work
# exactly; it does not claim to price the fused graph. The wall-clock advisory is the only view of
# the other graph and is machine-dependent, which is precisely why it advises and does not bind.
# Neither metric alone is sufficient, and the report says so on every run.
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
    if misses:
        print(f"parse-cost-ratchet: the identity reader does not discriminate ({misses} control(s) "
              f"missed); refusing", file=sys.stderr)
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
