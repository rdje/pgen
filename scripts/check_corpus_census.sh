#!/usr/bin/env bash
# scripts/check_corpus_census.sh
#
# DOCTRINE `CORPUS-CENSUS-CURRENCY` (evidence + structural) — DONE-BAR-VISIBILITY.3b.
#
#   A corpus pass rate nobody re-derives is a number about a tree that no longer exists.
#
# ⭐ WHY THIS IS A DOCTRINE AND NOT A NOTE — IT IS A MEASURED ROT, NOT A HYPOTHETICAL.
# `stimuli/vhdl/characterization/characterization.md` is TRACKED and carries, in its own words:
#
#     "Re-hash these three inputs; if any hash differs from the row below, this report no longer
#      describes your tree and the honest act is to re-measure, not to quote."
#
# Nothing in the repository re-hashed them. Measured 2026-08-30, at the moment this doctrine was
# written: the recorded generated-parser digest was `a90ae37b…` while the tree carried
# `48a5d294…`, and a re-measure moved the census from 4 335 / 9 385 (31.6 %) to
# 4 370 / 9 350 (31.9 %) — 35 rows, every one of them fail→pass and not one pass→fail, over 418
# commits of which 72 touched `rust/src` and ZERO touched `grammars/vhdl.ebnf`. So the published
# VHDL pass rate had been wrong for 22 days, in the flattering direction, on a 56-second
# measurement, because no instrument asked it to re-run. That is `CORPUS-GRAD-ALL.2.1`'s own
# closing sentence — *"nothing asked it to"* — still true one campaign later.
#
# WHAT IT ASSERTS, per family declared in `corpus_census` of
# `rust/test_data/grammar_quality/family_status_block_v0.json` with `checked_by` naming this file.
# Each arm is blind to what the next one sees, which is why there are five:
#
#   (1) STRUCTURE — the report carries the tables this doctrine reads, in the shape it reads them.
#       A missing table REFUSES; it is not silently scored zero.
#   (2) ARITHMETIC — pass+fail+timeout+crash = files, on the totals row AND on every sub-corpus
#       row; the sub-corpus column sums equal the totals row; each published pass-rate equals its
#       own row's quotient. This is the arm that catches a HAND-EDITED number, which is not
#       hypothetical either: `LIVE-MEANS-LIVE.1a` measured 25 of 61 hand-maintained fields wrong.
#   (3) GRAMMAR IDENTITY — the report's declared grammar digest re-hashes equal. HARD FAIL, and
#       affordable by measurement rather than by hope: `grammars/vhdl.ebnf` has moved TWELVE times
#       in the life of this repository, and a grammar edit is exactly the event after which a
#       corpus pass rate must not be quoted until it is re-run.
#   (4) DENOMINATOR CURRENCY — the file count under each declared sub-corpus is re-counted from
#       the working tree and held equal to the row that publishes it. A denominator is the half of
#       a pass rate nobody checks; this is the arm that notices a submodule bump.
#   (5) ENGINE STALENESS, ON A DERIVED BUDGET — the generated parser is regenerated from unchanged
#       grammar text whenever the ENGINE moves, so a digest mismatch here is NOT a failure: it is
#       ordinary, frequent, and making it blocking would tax most commits in this repository with
#       a re-measure. `BASELINE-IDENTITY` learned that the hard way — ONE COMMENT LINE in the SV
#       grammar blocked every commit. So this arm counts commits touching the declared engine
#       paths since the census's own recorded HEAD, prints a NOTE inside the budget and FAILS past
#       it. ⛔ THE BUDGET IS THE ONE MEASURED DRIFT EPISODE, NOT A ROUND NUMBER: 72 engine commits
#       produced 35 moved rows (0.26 %), one direction. ⚠️ HONEST LIMIT, stated rather than
#       hidden: that is the size at which drift was DETECTED, not a size proven safe — drift may
#       have crossed earlier and nobody was looking. What the budget buys is a bounded
#       re-measurement cost (~56 s per 72 engine commits) instead of an unbounded silence.
#
# ⛔ WHY THERE IS NO SECOND ARTIFACT. The obvious design — a machine-readable census JSON beside
# the report — was rejected: two tracked files holding the same counts is a second thing to drift,
# and this doctrine exists because a number drifted. The tracked report IS the census, this gate
# re-derives from it, and `scripts/report_family_status.sh` publishes from the same file, so the
# gate and the book cannot read different numbers.
#
# ⛔ DIGESTS ARE NOT RE-IMPLEMENTED HERE. `scripts/check_baseline_identity.sh --digest bytes FILE`
# is the repository's one definition of a file digest and is called for every hash this gate
# compares. A second implementation is a second thing to disagree.
#
# ⚠️ HONEST LIMIT (DOCTRINE_ENFORCEMENT.md §3). This proves the published census is INTERNALLY
# sound, describes the grammar in this tree, and counts the corpus that is in this tree. It does
# NOT re-run the parse — that is 56 s and a release binary, which is not a per-commit cost — and
# it says nothing about whether a parse-FAIL is the CORRECT outcome: the VHDL corpus has no answer
# key, so its pass rate is RAW, and `characterization.md` says so in its own opening paragraph.
#
# ⚠️ The vendored corpora are git SUBMODULES, so a checkout without them cannot run arm (4). It is
# then reported NOT EVALUATED — loudly, never as a pass
# (`docs/decisions/feedback_a_check_that_cannot_run_must_say_so.md`). Untracked build products
# (`generated/`, `rust/target/`) are treated the same way.
#
# CONTRACT (DOCTRINE_ENFORCEMENT.md §4): exit code is the verdict; explains on stderr;
# deterministic; does not mutate tracked state; path-agnostic; fast (~0.5 s, no cargo, no network).
#
# Usage:
#   bash scripts/check_corpus_census.sh                # every family this gate owns
#   bash scripts/check_corpus_census.sh vhdl           # one family, and print its tuple
#   bash scripts/check_corpus_census.sh --self-test    # drive every refusal arm + a green control
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$ROOT" || exit 2

exec python3 - "$@" <<'PYEOF'
import contextlib, io, json, os, re, subprocess, sys

ROOT = os.getcwd()
# ⛔ THE SCHEMA PATH IS A SEAM, and it is the SAME seam `scripts/report_family_status.sh` reads —
# one variable, not two, so a control that redirects the producer redirects its gate with it. It
# exists so the RED arms can be driven against a CONSTRUCTED declaration pointing at a mutated COPY
# of the real report, instead of mutating a tracked file and hoping the restore runs.
SCHEMA = (os.environ.get("PGEN_FAMILY_STATUS_SCHEMA")
          or "rust/test_data/grammar_quality/family_status_block_v0.json")
SELF = "scripts/check_corpus_census.sh"
DIGESTER = "scripts/check_baseline_identity.sh"
TAG = "corpus-census"

failures, notes, unevaluated = [], [], []


def refuse(msg):
    print(f"{TAG}: REFUSE — {msg}", file=sys.stderr)
    sys.exit(2)


# =================================================================================================
# The report reader. Everything below is a pure function of the report TEXT, so the self-test can
# drive every arm from constructed text without touching a tracked file.
# =================================================================================================
TOTALS_COLS = ("files", "pass", "fail", "timeout", "crash", "rate")
SHA256_RE = re.compile(r"^[0-9a-f]{64}$")


def _cells(line):
    """The cells of one GFM row. Leading and trailing pipes are BOTH optional in GFM, so they are
    stripped rather than counted — reading a trailing-pipe-less row by position is the defect
    TABLE-ARITY-RATCHET pins, and it made this repository's own instrument publish 25 for a
    population of 26."""
    s = line.strip()
    if s.startswith("|"):
        s = s[1:]
    if s.endswith("|"):
        s = s[:-1]
    return [c.strip() for c in s.split("|")]


def _section(text, heading):
    """The body of one `## heading` section, up to the next heading of the same or higher level.

    ⛔ SECTION-SCOPED, NOT DOCUMENT-SCOPED, and that is not cosmetic: `characterization.md` carries
    FOUR tables and three of them have a numeric first column. A document-wide 'first row of six
    numbers' reader finds the deadline-proximity table's rows and calls them totals.

    ⛔ THE HEADING MATCH IS EXACT, NOT A SUFFIX, and the self-test pins it there: the first cut used
    `endswith`, so `## Not Totals` matched `Totals` and a report that had LOST its totals section
    was read as still having one. Caught by the control that renames the heading — which is the
    only reason it is not in the shipped gate."""
    out, taking = [], False
    for line in text.splitlines():
        if re.match(r"^#{1,6}\s", line):
            if taking:
                break
            taking = (line.startswith("## ")
                      and line[3:].strip().lower() == heading.lower())
            continue
        if taking:
            out.append(line)
    return "\n".join(out)


def _rows(body):
    """Data rows of the first table in `body` — header and the `---` separator dropped."""
    rows, seen_sep = [], False
    for line in body.splitlines():
        if "|" not in line:
            if rows:
                break
            continue
        cells = _cells(line)
        if all(re.fullmatch(r":?-{2,}:?", c) for c in cells):
            seen_sep = True
            continue
        if seen_sep:
            rows.append(cells)
    return rows


def _int(cell, where):
    if not re.fullmatch(r"\d+", cell):
        refuse(f"{where}: expected a whole number, found {cell!r}")
    return int(cell)


def _rate(cell, where):
    m = re.fullmatch(r"(\d+(?:\.\d+)?)\s*%", cell)
    if not m:
        refuse(f"{where}: expected a percentage like `31.9%`, found {cell!r}")
    return float(m.group(1))


def parse_report(text, where="report"):
    """The census, as data. REFUSES on a shape it does not recognise rather than scoring what it
    managed to find — a census that silently reads three of ten sub-corpora is worse than none."""
    totals_rows = _rows(_section(text, "Totals"))
    if len(totals_rows) != 1:
        refuse(f"{where}: the `## Totals` section must carry exactly one data row, found "
               f"{len(totals_rows)}")
    if len(totals_rows[0]) != 6:
        refuse(f"{where}: the totals row must carry 6 cells "
               f"(files/pass/fail/timeout/crash/pass-rate), found {len(totals_rows[0])}")
    t = totals_rows[0]
    totals = dict(zip(TOTALS_COLS, [_int(c, f"{where}: totals") for c in t[:5]]
                      + [_rate(t[5], f"{where}: totals pass-rate")]))

    subs = []
    sub_rows = _rows(_section(text, "Per sub-corpus"))
    if not sub_rows:
        refuse(f"{where}: the `## Per sub-corpus` section carries no data row")
    for r in sub_rows:
        if len(r) != 7:
            refuse(f"{where}: sub-corpus row {r[0]!r} must carry 7 cells, found {len(r)}")
        subs.append(dict(name=r[0],
                         **dict(zip(TOTALS_COLS,
                                    [_int(c, f"{where}: sub-corpus {r[0]}") for c in r[1:6]]
                                    + [_rate(r[6], f"{where}: sub-corpus {r[0]} pass-rate")]))))

    identity = {}
    for r in _rows(_section(text, "Instrument identity (what produced this number)")):
        if len(r) != 3:
            refuse(f"{where}: instrument-identity row {r[0]!r} must carry 3 cells "
                   f"(input/path/sha256), found {len(r)}")
        label, path, sha = r[0], r[1].strip("`"), r[2].strip("`")
        if not SHA256_RE.fullmatch(sha):
            refuse(f"{where}: instrument-identity row {label!r} carries {sha!r}, not a sha256")
        identity[label] = (path, sha)
    if not identity:
        refuse(f"{where}: no `## Instrument identity` table — the report cannot say which tree it "
               f"describes, which is the whole question this doctrine asks")

    m = re.search(r"Measured at `HEAD` = `([0-9a-f]+)(\+dirty)?`", text)
    head = m.group(1) if m else None
    return dict(totals=totals, subs=subs, identity=identity, head=head, dirty=bool(m and m.group(2)))


def arithmetic_breaches(census, where="report"):
    """Arm (2). Returns a list of strings; empty means the report agrees with itself."""
    out = []

    def check_row(label, row):
        parts = row["pass"] + row["fail"] + row["timeout"] + row["crash"]
        if parts != row["files"]:
            out.append(f"{where}: {label} — pass+fail+timeout+crash = {parts} but files = "
                       f"{row['files']}; the outcome classes do not partition the row")
        if row["files"]:
            want = 100.0 * row["pass"] / row["files"]
            # One ulp of the published single decimal, so a rounding-mode difference between awk
            # and python is not reported as a defect while a typed number still is.
            if abs(want - row["rate"]) > 0.05 + 1e-9:
                out.append(f"{where}: {label} — publishes {row['rate']}% but "
                           f"{row['pass']}/{row['files']} = {want:.4f}%")

    check_row("totals", census["totals"])
    for s in census["subs"]:
        check_row(f"sub-corpus `{s['name']}`", s)
    for col in ("files", "pass", "fail", "timeout", "crash"):
        got = sum(s[col] for s in census["subs"])
        if got != census["totals"][col]:
            out.append(f"{where}: the sub-corpus rows sum to {got} in column `{col}` while the "
                       f"totals row publishes {census['totals'][col]}")
    return out


# =================================================================================================
# GROUND TRUTH, before any verdict is trusted. An instrument with no ground truth is a confident
# guess (docs/decisions/feedback_instrument_needs_ground_truth.md). Every RED arm is driven, a
# GREEN control is driven, and a MISS refuses rather than reporting a clean tree.
# =================================================================================================
GREEN_REPORT = """# x

## Instrument identity (what produced this number)

| input | repo-root-relative path | sha256 |
|---|---|---|
| grammar | `grammars/x.ebnf` | `ZERODIGEST` |

Measured at `HEAD` = `0123456789abcdef0123456789abcdef01234567` (2026-01-01).

## Deadline proximity and serial re-confirmation

| population | files |
|---|---|
| `timeout` after serial re-confirmation | 0 |

## Totals

| files parsed | pass | fail | timeout | crash | pass-rate |
|---|---|---|---|---|---|
| 100 | 40 | 60 | 0 | 0 | 40.0% |

## Per sub-corpus

| sub-corpus | files | pass | fail | timeout | crash | pass-rate |
|---|---|---|---|---|---|---|
| a | 60 | 30 | 30 | 0 | 0 | 50.0% |
| b | 40 | 10 | 30 | 0 | 0 | 25.0% |
""".replace("ZERODIGEST", "0" * 64)


def self_test():
    misses = []

    def want(label, text, predicate):
        # ⛔ THE DRIVEN CONTROL'S OWN STDERR IS SWALLOWED, DELIBERATELY. A refusal prints before it
        # exits, so without this every GREEN run of this gate emitted five REFUSE lines from its
        # own controls — a gate whose passing output reads like a failure is one people learn to
        # ignore. What the control asserts is that the refusal HAPPENED, never its wording.
        try:
            with contextlib.redirect_stderr(io.StringIO()):
                c = parse_report(text, where="control")
                got = arithmetic_breaches(c, where="control")
        except SystemExit:
            got = "REFUSED"
        if not predicate(got):
            misses.append(f"{label}: got {got!r}")

    # GREEN control: the arms must be able to say nothing is wrong, or their silence proves nothing.
    want("green control", GREEN_REPORT, lambda g: g == [])

    # The deadline-proximity table sits BETWEEN identity and totals and has a numeric column; the
    # green control carries it precisely so a document-scoped reader is caught here.
    c = parse_report(GREEN_REPORT, where="control")
    if c["totals"]["files"] != 100 or len(c["subs"]) != 2:
        misses.append(f"section scoping: read totals={c['totals']} subs={len(c['subs'])}")
    if c["head"] != "0123456789abcdef0123456789abcdef01234567":
        misses.append(f"HEAD reader: got {c['head']!r}")
    if c["identity"].get("grammar") != ("grammars/x.ebnf", "0" * 64):
        misses.append(f"identity reader: got {c['identity']!r}")

    # RED: the outcome classes stop partitioning the totals row.
    want("totals partition", GREEN_REPORT.replace("| 100 | 40 | 60 | 0 | 0 | 40.0% |",
                                                  "| 100 | 41 | 60 | 0 | 0 | 41.0% |"),
         lambda g: any("do not partition" in x for x in g))
    # RED: a sub-corpus row stops partitioning.
    want("sub-corpus partition", GREEN_REPORT.replace("| a | 60 | 30 | 30 | 0 | 0 | 50.0% |",
                                                      "| a | 60 | 31 | 30 | 0 | 0 | 51.7% |"),
         lambda g: any("sub-corpus `a`" in x and "do not partition" in x for x in g))
    # RED: a hand-typed pass rate that no quotient supports.
    want("typed pass-rate", GREEN_REPORT.replace("| 100 | 40 | 60 | 0 | 0 | 40.0% |",
                                                 "| 100 | 40 | 60 | 0 | 0 | 94.0% |"),
         lambda g: any("publishes 94.0%" in x for x in g))
    # …and the tolerance must be exactly half of the published digit, from both sides. A rounded
    # quotient is legitimate (awk and python may not round a tie the same way); a tenth of a point
    # is not, and a tolerance loose enough to swallow one would swallow a typed number too.
    want("a rounded quotient is tolerated",
         GREEN_REPORT.replace("| 100 | 40 | 60 | 0 | 0 | 40.0% |",
                              "| 99 | 33 | 66 | 0 | 0 | 33.3% |")
                     .replace("| a | 60 | 30 | 30 | 0 | 0 | 50.0% |",
                              "| a | 59 | 23 | 36 | 0 | 0 | 39.0% |")
                     .replace("| b | 40 | 10 | 30 | 0 | 0 | 25.0% |",
                              "| b | 40 | 10 | 30 | 0 | 0 | 25.0% |"),
         lambda g: g == [])
    want("a one-tenth-of-a-point disagreement is NOT tolerated",
         GREEN_REPORT.replace("| 100 | 40 | 60 | 0 | 0 | 40.0% |",
                              "| 100 | 40 | 60 | 0 | 0 | 40.1% |"),
         lambda g: any("publishes 40.1%" in x for x in g))
    # RED: the column sums stop matching the totals row (the arm that catches a dropped row).
    want("column sum", GREEN_REPORT.replace("| b | 40 | 10 | 30 | 0 | 0 | 25.0% |", ""),
         lambda g: any("column `files`" in x for x in g))
    # RED: the shapes this reader must refuse rather than score.
    want("no totals table", GREEN_REPORT.replace("## Totals", "## Not Totals"),
         lambda g: g == "REFUSED")
    want("no sub-corpus table", GREEN_REPORT.replace("## Per sub-corpus", "## Something else"),
         lambda g: g == "REFUSED")
    want("no identity table",
         GREEN_REPORT.replace("## Instrument identity (what produced this number)", "## Nope"),
         lambda g: g == "REFUSED")
    want("identity carries a non-sha",
         GREEN_REPORT.replace("`" + "0" * 64 + "`", "`not-a-digest`"),
         lambda g: g == "REFUSED")
    want("totals row of the wrong arity",
         GREEN_REPORT.replace("| 100 | 40 | 60 | 0 | 0 | 40.0% |", "| 100 | 40 | 60 | 0 | 40.0% |"),
         lambda g: g == "REFUSED")

    if misses:
        print(f"{TAG}: the reader does not discriminate ({len(misses)} control(s) missed):",
              file=sys.stderr)
        for m in misses:
            print(f"  ✗ {m}", file=sys.stderr)
        sys.exit(2)
    print(f"{TAG}: self-test OK — 14 controls, every refusal arm and the green control observed "
          f"firing")


# =================================================================================================
# The live check.
# =================================================================================================
def digest(path):
    """The repository's ONE definition of a file digest, called rather than re-implemented."""
    p = subprocess.run(["bash", DIGESTER, "--digest", "bytes", path],
                       capture_output=True, text=True, cwd=ROOT)
    if p.returncode != 0:
        refuse(f"`{DIGESTER} --digest bytes {path}` exited {p.returncode}: "
               f"{(p.stderr or p.stdout).strip()}")
    out = p.stdout.strip().split()
    if not out or not SHA256_RE.fullmatch(out[0]):
        refuse(f"`{DIGESTER} --digest bytes {path}` printed {p.stdout.strip()!r}, not a sha256")
    return out[0]


def check_family(family, decl):
    """Returns the census tuple for `family`, appending to the module-level verdict lists."""
    kind = decl.get("kind")
    if kind != "raw_parse":
        refuse(f"'{family}' declares census kind {kind!r}, which this gate does not implement. A "
               f"kind with no reader must REFUSE rather than be skipped: an unimplemented census "
               f"that reports nothing is indistinguishable from a census that passed.")
    report_path = decl["report"]
    if not os.path.isfile(report_path):
        failures.append(f"{family}: the tracked census report is missing: {report_path}")
        return None
    with open(report_path, encoding="utf-8") as fh:
        text = fh.read()

    # (1) + (2) structure and arithmetic — always evaluable, tracked-only.
    census = parse_report(text, where=report_path)
    failures.extend(arithmetic_breaches(census, where=report_path))
    if census["dirty"]:
        notes.append(f"{family}: the census records a DIRTY tree ({census['head']}+dirty), so its "
                     f"recorded HEAD does not identify the state it measured")

    regen = decl["regenerate"]

    # (3) grammar identity — HARD, tracked, and the event after which a pass rate must not be quoted.
    label = decl["grammar_identity_row"]
    if label not in census["identity"]:
        failures.append(f"{family}: {report_path} declares no `{label}` row in its instrument "
                        f"identity table, so nothing pins the language it measured")
    else:
        path, recorded = census["identity"][label]
        if not os.path.isfile(path):
            failures.append(f"{family}: {report_path} names `{path}` as its {label}, and that file "
                            f"does not exist in this tree")
        elif digest(path) != recorded:
            failures.append(
                f"{family}: the {label} `{path}` has MOVED since this census was measured "
                f"(report: {recorded[:12]}…, tree: {digest(path)[:12]}…). A corpus pass rate is a "
                f"statement about a language; re-measure before it is quoted again: {regen}")

    # (4) denominator currency — needs the vendored corpora, which are submodules.
    root = decl["corpus_root"]
    exts = tuple(decl["file_extensions"])
    present = [s for s in census["subs"] if os.path.isdir(os.path.join(root, s["name"]))
               and os.listdir(os.path.join(root, s["name"]))]
    if len(present) != len(census["subs"]):
        missing = [s["name"] for s in census["subs"] if s not in present]
        unevaluated.append(f"{family}: the denominator arm — {len(missing)} of "
                           f"{len(census['subs'])} sub-corpora are absent or empty here "
                           f"({', '.join(missing)}); the vendored corpora are git submodules")
    for s in present:
        n = 0
        for dirpath, _dirnames, filenames in os.walk(os.path.join(root, s["name"])):
            n += sum(1 for f in filenames if f.endswith(exts))
        if n != s["files"]:
            failures.append(
                f"{family}: sub-corpus `{s['name']}` publishes {s['files']} files but "
                f"{os.path.join(root, s['name'])} holds {n} matching {'/'.join(exts)} today — the "
                f"DENOMINATOR moved, so every share derived from it is about a different corpus. "
                f"Re-measure: {regen}")

    # (5) engine staleness, on a derived budget.
    plabel = decl["parser_identity_row"]
    ppath, precorded = census["identity"].get(plabel, (None, None))
    if ppath is None:
        failures.append(f"{family}: {report_path} declares no `{plabel}` row, so the engine-drift "
                        f"arm has nothing to read")
    elif not os.path.isfile(ppath):
        unevaluated.append(f"{family}: the engine-drift arm — `{ppath}` is an untracked build "
                           f"product and is absent here")
    elif digest(ppath) == precorded:
        pass
    else:
        budget = decl["engine_staleness_budget_commits"]
        paths = decl["engine_paths"]
        head = census["head"]
        p = subprocess.run(["git", "rev-list", "--count", f"{head}..HEAD", "--"] + paths,
                           capture_output=True, text=True, cwd=ROOT)
        if p.returncode != 0:
            unevaluated.append(f"{family}: the engine-drift budget — the census's recorded HEAD "
                               f"`{head}` is not in this clone")
            return tuple_of(census)
        n = int(p.stdout.strip() or 0)
        msg = (f"{family}: the {plabel} `{ppath}` differs from the census's ({precorded[:12]}… vs "
               f"{digest(ppath)[:12]}…) after {n} commit(s) touching {', '.join(paths)}. The "
               f"grammar is unchanged, so this is ENGINE drift")
        if n > budget:
            failures.append(f"{msg} — past the budget of {budget}, which is the one drift episode "
                            f"this repository has measured (72 engine commits moved 35 of 13 720 "
                            f"rows). Re-measure: {regen}")
        else:
            notes.append(f"{msg}; {budget - n} commit(s) of budget left before this becomes a "
                         f"failure. Re-measure any time: {regen}")
    return tuple_of(census)


def tuple_of(census):
    t = census["totals"]
    return "/".join(str(t[c]) for c in ("files", "pass", "fail", "timeout", "crash"))


# --- drive ---------------------------------------------------------------------------------------
args = [a for a in sys.argv[1:]]
if "--self-test" in args:
    self_test()
    sys.exit(0)

if not os.path.isfile(SCHEMA):
    refuse(f"required file missing: {SCHEMA}")
with open(SCHEMA, encoding="utf-8") as fh:
    schema = json.load(fh)

owned = {f: d for f, d in schema.get("corpus_census", {}).items()
         if d.get("checked_by") == SELF}
if not owned:
    refuse(f"no family in `corpus_census` of {SCHEMA} names {SELF} as its `checked_by`. A gate "
           f"with an empty population reports a clean tree for free, which is the "
           f"GATE-REACHABILITY defect wearing a census.")

wanted = [a for a in args if not a.startswith("-")]
for a in wanted:
    if a not in owned:
        refuse(f"'{a}' is not a family this gate owns ({', '.join(sorted(owned))})")
selected = wanted or sorted(owned)

self_test()

tuples = {}
for family in selected:
    t = check_family(family, owned[family])
    if t:
        tuples[family] = t

for u in unevaluated:
    print(f"{TAG}: NOT EVALUATED — {u}", file=sys.stderr)
for n in notes:
    print(f"{TAG}: NOTE — {n}", file=sys.stderr)

if failures:
    print(f"{TAG}: {len(failures)} breach(es):", file=sys.stderr)
    for f in failures:
        print(f"  ✗ {f}", file=sys.stderr)
    sys.exit(1)

for family in selected:
    if family in tuples:
        print(f"{TAG}[{family}]: OK (tuple {tuples[family]} = "
              f"files/pass/fail/timeout/crash; RAW parse outcomes, no answer key)")
PYEOF
