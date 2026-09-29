#!/usr/bin/env bash
# scripts/check_per_frame_census_currency.sh
#
# DOCTRINE `PER-FRAME-CENSUS-CURRENCY` (structural + evidence) — ENGINE-UNIVERSAL-SERVICES.83.
#
#   A number DERIVED from a tracked measurement and published on several live surfaces is held equal
#   to a fresh derivation on every commit, and the measurement says which tree it describes.
#
# THE NUMBER. `.79`(a)'s per-frame stack-cost census: two tracked measurements
# (docs/tasks/artifacts/engine_universal_services/eus79a_per_frame_census/census_{debug,release}_96MiB.txt
# — one row per family/profile, `bytes_per_frame` read off the depth probe at its refusal) and the
# MARGIN table `derive_margins.sh` derives from them against the engine's OWN constants
# (`DEDICATED_PARSE_STACK_BYTES`, `STACK_HEADROOM_RESERVE_DIVISOR`, read out of the Rust source on
# every run). Its headline — the tightest debug margin (2.12x at systemverilog/verilog_2005 when this
# shipped), the spread across families, the separation to the nearest other family, the tightest
# release margin and the debug/release ratio at the binding cell — is what a downstream embedder
# sizes a host on, and it is published on FOUR surfaces: the platform book's embedding chapter, the
# engine's own doc comment on `first_stack_sample_depth`, the SystemVerilog integration contract and
# the knowledge card that generalises the lesson.
#
# ⛔ WHY A DOCTRINE — MEASURED, NOT HYPOTHETICAL. `-0145` (2026-08-30) published the census and
# updated the tracker only; all four surfaces carried the superseded "2.37x on SystemVerilog, every
# other family unmeasured" for one commit, caught by `-0146`'s self-audit. `-0147` then claimed
# "no doctrine catches this class" on the strength of two doctrines out of thirty-one, and its own
# census refuted it: SIX registered doctrines already hold a derived number to its designated
# surfaces (SV-CORPUS-DENOMINATOR, PARSE-COST-RATCHET, BASELINE-IDENTITY, GRAMMAR-CERT-CURRENCY,
# PUBLISHED-VERSION-CURRENCY, REGEX-ORACLE-ANCHOR-SYNC). This check registers ONE MORE NUMBER with
# that proven pattern — the ask `.83` narrowed itself to.
#
# WHAT IT ASSERTS, in four arms, each blind to what the next sees:
#   (1) STRUCTURE — the identity record names both census files; each exists with the census's
#       six-column shape and every family row names a tracked grammar; the deriver is executable.
#   (2) IDENTITY — `check_baseline_identity.sh --verify` on the record (ONE digest definition, never
#       a third copy). Its declared inputs are the two census files and the two engine files the
#       deriver reads its constants from. A moved input is a STALE stamp: the anchors may still be
#       right, but nobody re-derived them, and re-deriving costs a tenth of a second — so a stale
#       stamp FAILS with the one-command remedy, rather than being budgeted the way a grammar-keyed
#       baseline must be (BASELINE-IDENTITY learned that one comment line can block every commit;
#       here nothing that cheap can move an input).
#   (3) CO-PUBLICATION — `derive_margins.sh` is RUN on both census files and every designated
#       surface carries the derived headline in ONE anchor spelling; the book's two tables are held
#       ROW BY ROW (family, B/frame, margin; release B/frame, release margin, debug B/frame, ratio),
#       because the book is the director's stated review surface and a table row is where a stale
#       number hides best.
#   (4) DRIFT NOTE — the record says at which commit each census was MEASURED; the commits since
#       that touched the code that shapes a parser frame (the codegen, the recursion guard, the stack
#       module) are COUNTED and printed. ⚠️ A NOTE, never a failure: no drift episode has been
#       measured for this quantity, so no budget can honestly be derived from it
#       (CORPUS-CENSUS-CURRENCY derived its 72 from the one episode it had). The count says how old
#       the measurement is; re-measuring is `per_frame_census.sh` and it is priced in the README.
#
# ANCHOR SPELLING (one home): a line containing the marker `Per-frame census anchor` and, in the
# FIRST code span after the marker, exactly the tuple `--tuple` prints — the SV-CORPUS-DENOMINATOR
# discipline: the marker is the discriminator, and an era-dated old number in prose is history
# (supersede-don't-mutate), deliberately out of scope.
#
# ⚠️ HONEST LIMITS, stated rather than discovered: the frame-shaping input set is DECLARED, not proven
# exhaustive (the whole crate and the rustc version can move a frame); the verdicts re-derive the
# published NUMBERS from the tracked census, they do not re-measure the census.
#
# CONTRACT (DOCTRINE_ENFORCEMENT.md §4): exit code is the verdict (0 holds / 1 breach / 2 refuses);
# explains on stderr; deterministic; mutates nothing (`--self-test` writes under rust/target/ only);
# resolves the repo root from its own location; fast (~0.3 s: two shell derivations and four
# hashes, no cargo, no network).
#
# Usage:
#   bash scripts/check_per_frame_census_currency.sh              # the doctrine
#   bash scripts/check_per_frame_census_currency.sh --tuple      # print the derived anchor tuple
#   bash scripts/check_per_frame_census_currency.sh --self-test  # GREEN on the tree + three RED controls
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$ROOT" || exit 2
SELF="scripts/check_per_frame_census_currency.sh"
export PFC_SELF="$SELF"

exec python3 - "$@" <<'PYEOF'
import json, os, re, shutil, subprocess, sys

ROOT = os.getcwd()
TAG = "PER-FRAME-CENSUS"
SELF = os.environ["PFC_SELF"]
DIR = "docs/tasks/artifacts/engine_universal_services/eus79a_per_frame_census"
RECORD = os.environ.get("PFC_RECORD", DIR + "/census_identity.json")
DERIVER = DIR + "/derive_margins.sh"
VERIFIER = "scripts/check_baseline_identity.sh"
MARK = "Per-frame census anchor"
# The self-test seam: a directory mirroring the surface paths, so a RED control mutates COPIES.
SURFACE_ROOT = os.environ.get("PFC_SURFACE_ROOT", ROOT)
SURFACES = [
    "docs/book/src/embedding-and-downstream-integration.md",
    "rust/src/dedicated_parse_stack.rs",
    "docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md",
    "docs/knowledge/a-threshold-that-is-correct-at-one-parameter-value-will-rot.md",
]
BOOK = SURFACES[0]
DEBUG_HDR = "| family / profile | B/frame | margin at the 256 MiB reference |"
RELEASE_HDR = "| family / profile | release B/frame | release margin | debug B/frame | debug → release |"
COMMIT_RE = re.compile(r"^[0-9a-f]{40}$")

failures, notes = [], []
def fail(msg): failures.append(msg)
def note(msg): notes.append(msg)

def sh(args, **kw):
    return subprocess.run(args, capture_output=True, text=True, **kw)

# ── (1) STRUCTURE ────────────────────────────────────────────────────────────────────────────────
def load_record():
    p = os.path.join(ROOT, RECORD)
    if not os.path.isfile(p):
        fail(f"the census identity record is missing: {RECORD}"); return None
    try:
        rec = json.load(open(p, encoding="utf-8"))
    except ValueError as exc:
        fail(f"{RECORD} is not parseable JSON: {exc}"); return None
    cens = rec.get("censuses")
    if not isinstance(cens, dict) or set(cens) != {"census_debug_96MiB.txt", "census_release_96MiB.txt"}:
        fail(f"{RECORD}: `censuses` must name exactly census_debug_96MiB.txt and census_release_96MiB.txt")
        return None
    for name, meta in cens.items():
        c = meta.get("measured_at_commit", "")
        if not COMMIT_RE.match(c):
            fail(f"{RECORD}: {name}.measured_at_commit must be a 40-hex commit"); continue
        if sh(["git", "cat-file", "-e", c + "^{commit}"]).returncode != 0:
            fail(f"{RECORD}: {name}.measured_at_commit {c[:9]} is not a commit of this repository")
        if not re.match(r"^\d{4}-\d\d-\d\d$", str(meta.get("measured_on", ""))):
            fail(f"{RECORD}: {name}.measured_on must be an ISO date")
    fs = rec.get("frame_shaping_inputs")
    if not isinstance(fs, list) or not fs or not all(isinstance(x, str) for x in fs):
        fail(f"{RECORD}: `frame_shaping_inputs` must be a non-empty list of repo-relative paths")
    else:
        for x in fs:
            if not os.path.isfile(os.path.join(ROOT, x)):
                fail(f"{RECORD}: frame_shaping_inputs names a file that does not exist: {x}")
    return rec

def check_census_shape(name):
    p = os.path.join(ROOT, DIR, name)
    if not os.path.isfile(p):
        fail(f"census file missing: {DIR}/{name}"); return
    lines = [l for l in open(p, encoding="utf-8").read().splitlines() if l.strip()]
    if not lines or lines[0].split() != ["FAMILY", "PROFILE", "LEVELS", "FRAMES", "CAUSE", "BYTES_PER_FRAME"]:
        fail(f"{DIR}/{name}: the header is not the census's six columns"); return
    for l in lines[1:]:
        cols = l.split()
        if len(cols) != 6:
            fail(f"{DIR}/{name}: a row does not have six columns: {l!r}"); continue
        fam = cols[0]
        g = os.path.join(ROOT, "grammars", fam + ".ebnf")
        if not os.path.isfile(g):
            fail(f"{DIR}/{name}: family `{fam}` names no tracked grammar (grammars/{fam}.ebnf)")

# ── (3a) DERIVATION ──────────────────────────────────────────────────────────────────────────────
ROW_RE = re.compile(r"^(\S+)\s+(\S+)\s+(\d+)\s+(\d+)\s+([0-9]+\.[0-9]{2})x$")
def derive(name):
    if not os.access(os.path.join(ROOT, DERIVER), os.X_OK):
        fail(f"the deriver is missing or not executable: {DERIVER}"); return None
    r = sh(["bash", DERIVER, DIR + "/" + name])
    if r.returncode != 0:
        fail(f"{DERIVER} refused {name} (rc {r.returncode}): {r.stderr.strip()[:300]}"); return None
    out = r.stdout
    rows = {}
    for l in out.splitlines():
        m = ROW_RE.match(l.strip())
        if m:
            rows[(m.group(1), m.group(2))] = (int(m.group(3)), m.group(5))
    m_t = re.search(r"tightest margin ([0-9.]+)x\s+\((\S+)\)", out)
    m_s = re.search(r"^spread\s+([0-9.]+)x", out, re.M)
    m_sep = re.search(r"^separation\s+([0-9.]+)x", out, re.M)
    m_n = re.search(r"^numeric cells\s+(\d+)", out, re.M)
    if not (rows and m_t and m_s and m_sep and m_n):
        fail(f"{DERIVER} printed a table this check cannot read for {name} — its output format moved; "
             "update the reader here in the same commit"); return None
    if int(m_n.group(1)) != len(rows):
        fail(f"{name}: the deriver counts {m_n.group(1)} numeric cells but printed {len(rows)} rows")
    return {"rows": rows, "tight": m_t.group(1), "tight_cell": m_t.group(2),
            "spread": m_s.group(1), "sep": m_sep.group(1)}

def tuple_of(rec, d, r):
    cell = tuple(r["tight_cell"].split("/", 1))
    if cell not in d["rows"] or cell not in r["rows"]:
        fail(f"the release-tightest cell {r['tight_cell']} has no debug reading, so the debug/release "
             "ratio at the binding cell cannot be derived"); return None
    ratio = d["rows"][cell][0] / r["rows"][cell][0]
    cd = rec["censuses"]["census_debug_96MiB.txt"]; cr = rec["censuses"]["census_release_96MiB.txt"]
    return (f"debug {d['tight']}x @ {d['tight_cell']} · spread {d['spread']}x · separation {d['sep']}x"
            f" · release {r['tight']}x @ {r['tight_cell']} · debug/release {ratio:.2f}x @ {r['tight_cell']}"
            f" · measured {cd['measured_at_commit'][:9]} {cd['measured_on']} (debug)"
            f" / {cr['measured_at_commit'][:9]} {cr['measured_on']} (release)")

# ── (3b) CO-PUBLICATION ──────────────────────────────────────────────────────────────────────────
def anchors_in(text):
    """One entry per marker line: the tuple in the first code span on THAT line that starts with
    `debug ` — the anchor spelling is marker and tuple on one line, so a wrapped explanation or a
    doctrine name in an earlier span can never be read as the tuple (both happened on this
    check's first run, against its author's own four anchors)."""
    found = []
    for l in text.splitlines():
        if MARK in l:
            after = l.split(MARK, 1)[1]
            spans = [m.group(1).strip() for m in re.finditer(r"`([^`]+)`", after)]
            tup = next((sp for sp in spans if sp.startswith("debug ")), "")
            found.append(tup)
    return found

def check_surfaces(tup):
    for s in SURFACES:
        p = os.path.join(SURFACE_ROOT, s)
        if not os.path.isfile(p):
            fail(f"designated live surface missing: {s}"); continue
        found = anchors_in(open(p, encoding="utf-8").read())
        if not found:
            fail(f"{s} carries NO per-frame census anchor. Add a line containing '{MARK}' followed by "
                 f"the tuple in a code span: `{tup}`"); continue
        if "" in found:
            fail(f"{s} carries the marker '{MARK}' on a line with no tuple code span — the anchor "
                 "spelling is the marker and the tuple `debug …` on ONE line")
        wrong = [f for f in found if f and f != tup]
        if wrong:
            fail(f"{s} publishes a per-frame census anchor that is NOT what the census derives:\n"
                 f"        published: {wrong[0]}\n        derived  : {tup}\n"
                 "      A moved constant or a re-measured census moves every surface — republish the anchor.")

def digits(s):
    return int(re.sub(r"[^0-9]", "", s))
def margin_str(s):
    return s.replace("*", "").replace("×", "").replace("x", "").strip()
def cell_of(s):
    names = re.findall(r"`([^`]+)`", s)
    if len(names) == 1: return (names[0], "<default>")
    if len(names) == 2: return (names[0], names[1])
    return None

def table_rows(text, header):
    lines = text.splitlines()
    try:
        i = next(k for k, l in enumerate(lines) if l.strip().lstrip("> ").strip() == header)
    except StopIteration:
        return None
    rows = []
    for l in lines[i + 1:]:
        body = l.strip().lstrip("> ").strip()
        if not body.startswith("|"): break
        if re.match(r"^\|\s*-", body): continue
        cells = [c.strip() for c in body.strip("|").split("|")]
        rows.append(cells)
    return rows

def check_book_tables(d, r):
    p = os.path.join(SURFACE_ROOT, BOOK)
    if not os.path.isfile(p): return
    text = open(p, encoding="utf-8").read()
    rows = table_rows(text, DEBUG_HDR)
    if rows is None:
        fail(f"{BOOK}: the debug census table header `{DEBUG_HDR}` is not found — the book's table "
             "must keep the header this check locates it by")
    else:
        seen = {}
        for cells in rows:
            if len(cells) != 3: fail(f"{BOOK}: debug table row does not have 3 cells: {cells}"); continue
            key = cell_of(cells[0])
            if key is None: fail(f"{BOOK}: debug table row names no family/profile: {cells[0]}"); continue
            seen[key] = (digits(cells[1]), margin_str(cells[2]))
        want = {k: (v[0], v[1]) for k, v in d["rows"].items()}
        for k in sorted(set(seen) | set(want)):
            if k not in want: fail(f"{BOOK}: debug table publishes {k[0]}/{k[1]} but the census has no numeric cell for it")
            elif k not in seen: fail(f"{BOOK}: debug table lacks the census cell {k[0]}/{k[1]} ({want[k][0]} B/frame, {want[k][1]}x)")
            elif seen[k] != want[k]:
                fail(f"{BOOK}: debug table row {k[0]}/{k[1]} publishes {seen[k]} but the census derives {want[k]} (B/frame, margin)")
    rows = table_rows(text, RELEASE_HDR)
    if rows is None:
        fail(f"{BOOK}: the release census table header `{RELEASE_HDR}` is not found")
    else:
        seen = {}
        for cells in rows:
            if len(cells) != 5: fail(f"{BOOK}: release table row does not have 5 cells: {cells}"); continue
            key = cell_of(cells[0])
            if key is None: fail(f"{BOOK}: release table row names no family/profile: {cells[0]}"); continue
            seen[key] = (digits(cells[1]), margin_str(cells[2]), digits(cells[3]), margin_str(cells[4]))
        want = {}
        for k, (bpf, m) in r["rows"].items():
            if k in d["rows"]:
                want[k] = (bpf, m, d["rows"][k][0], f"{d['rows'][k][0] / bpf:.2f}")
        for k in sorted(set(seen) | set(want)):
            if k not in want: fail(f"{BOOK}: release table publishes {k[0]}/{k[1]} but the census has no numeric release+debug pair for it")
            elif k not in seen: fail(f"{BOOK}: release table lacks the census cell {k[0]}/{k[1]}")
            elif seen[k] != want[k]:
                fail(f"{BOOK}: release table row {k[0]}/{k[1]} publishes {seen[k]} but the census derives {want[k]} "
                     "(release B/frame, release margin, debug B/frame, debug/release)")

# ── (2) IDENTITY ─────────────────────────────────────────────────────────────────────────────────
def check_identity():
    if not os.access(os.path.join(ROOT, VERIFIER), os.X_OK):
        fail(f"the identity verifier is missing or not executable: {VERIFIER}"); return
    r = sh(["bash", VERIFIER, "--verify", RECORD])
    if r.returncode == 0: return
    why = (r.stderr.strip() or r.stdout.strip()).splitlines()
    head = "\n".join("        " + l for l in why[:6])
    if r.returncode == 1:
        fail(f"the census identity record is STALE — an input it declares moved since it was stamped:\n{head}\n"
             "      Re-derive (bash " + DERIVER + " <census>), republish every anchor, then re-stamp:\n"
             "        bash " + VERIFIER + " --stamp " + RECORD + " --confirmed-by \"<this run>\"")
    else:
        fail(f"the identity verifier REFUSED the record (rc {r.returncode}):\n{head}")

# ── (4) DRIFT NOTE ───────────────────────────────────────────────────────────────────────────────
def drift_notes(rec):
    fs = rec.get("frame_shaping_inputs") or []
    if not fs: return
    for name, meta in sorted(rec["censuses"].items()):
        c = meta.get("measured_at_commit", "")
        if not COMMIT_RE.match(c): continue
        r = sh(["git", "rev-list", "--count", f"{c}..HEAD", "--"] + fs)
        n = r.stdout.strip() if r.returncode == 0 else "?"
        note(f"{name} was MEASURED at {c[:9]} ({meta.get('measured_on')}, {meta.get('build_mode')}); "
             f"{n} commit(s) since have touched the code that shapes a parser frame — the numbers describe "
             f"THAT tree; re-measure with {DIR}/per_frame_census.sh (no drift budget has been measured, so this is a note)")

# ── main ─────────────────────────────────────────────────────────────────────────────────────────
def run():
    rec = load_record()
    if rec is None: return 2
    for name in ("census_debug_96MiB.txt", "census_release_96MiB.txt"):
        check_census_shape(name)
    d = derive("census_debug_96MiB.txt"); r = derive("census_release_96MiB.txt")
    if failures or d is None or r is None: return 2 if (d is None or r is None) else 1
    tup = tuple_of(rec, d, r)
    if tup is None: return 2
    if len(sys.argv) > 1 and sys.argv[1] == "--tuple":
        print(tup); return 0
    check_identity()
    check_surfaces(tup)
    check_book_tables(d, r)
    drift_notes(rec)
    for n in notes: print(f"{TAG}: NOTE — {n}")
    if failures:
        print(f"{TAG}: {len(failures)} breach(es):", file=sys.stderr)
        for f in failures: print(f"  ✗ {f}", file=sys.stderr)
        return 1
    print(f"{TAG}: OK (anchor `{tup}`; the four surfaces and the book's {len(d['rows'])}+{len(r['rows'])} table rows "
          f"equal a fresh derivation; identity fresh)")
    return 0

def self_test():
    scratch = os.path.join(ROOT, "rust/target/per_frame_census_self_test")
    shutil.rmtree(scratch, ignore_errors=True); os.makedirs(scratch)
    def rerun(env_extra):
        env = dict(os.environ); env.update(env_extra)
        return sh(["bash", SELF], env=env)
    g = rerun({})
    if g.returncode != 0:
        print(f"{TAG} self-test: the GREEN control failed on the real tree:\n{g.stderr}", file=sys.stderr); return 1
    tup = sh(["bash", SELF, "--tuple"]).stdout.strip()
    def mirror(sub):
        root = os.path.join(scratch, sub)
        for s in SURFACES:
            os.makedirs(os.path.dirname(os.path.join(root, s)), exist_ok=True)
            shutil.copyfile(os.path.join(ROOT, s), os.path.join(root, s))
        return root
    # RED 1 — one surface's anchor tuple no longer equals the derivation.
    root = mirror("red1"); p = os.path.join(root, SURFACES[2])
    t = open(p, encoding="utf-8").read()
    assert tup in t, "the GREEN tree's contract must carry the tuple"
    open(p, "w", encoding="utf-8").write(t.replace(tup, tup.replace("debug ", "debug 9", 1), 1))
    r1 = rerun({"PFC_SURFACE_ROOT": root})
    if r1.returncode == 0 or SURFACES[2] not in r1.stderr:
        print(f"{TAG} self-test: ✗ RED 1 did not fire or did not name the surface:\n{r1.stderr}", file=sys.stderr); return 1
    # RED 2 — a book table row's margin is wrong while the anchor is right.
    root = mirror("red2"); p = os.path.join(root, BOOK)
    t = open(p, encoding="utf-8").read()
    m = re.search(r"\| \*\*([0-9]+\.[0-9]{2})×\*\* \|", t)
    assert m, "the GREEN book must carry a bold tightest-margin cell"
    open(p, "w", encoding="utf-8").write(t[:m.start()] + "| **9.99×** |" + t[m.end():])
    r2 = rerun({"PFC_SURFACE_ROOT": root})
    if r2.returncode == 0 or "table row" not in r2.stderr:
        print(f"{TAG} self-test: ✗ RED 2 did not fire on a wrong table row:\n{r2.stderr}", file=sys.stderr); return 1
    # RED 3 — the identity record's recorded digest of an input no longer matches (a stale stamp).
    rec_copy = "rust/target/per_frame_census_self_test/red3_record.json"
    rec = json.load(open(os.path.join(ROOT, RECORD), encoding="utf-8"))
    inputs = rec["identity"]["inputs"]
    k = sorted(inputs)[0]; v = inputs[k]
    digest = v["sha256"] if isinstance(v, dict) else v
    flipped = ("0" if digest[0] != "0" else "1") + digest[1:]
    if isinstance(v, dict): v["sha256"] = flipped
    else: inputs[k] = flipped
    json.dump(rec, open(os.path.join(ROOT, rec_copy), "w", encoding="utf-8"), indent=2)
    r3 = rerun({"PFC_RECORD": rec_copy})
    if r3.returncode == 0 or "STALE" not in r3.stderr:
        print(f"{TAG} self-test: ✗ RED 3 did not fire on a stale identity stamp:\n{r3.stderr}", file=sys.stderr); return 1
    print(f"{TAG} self-test: 4/4 as declared (GREEN on the tree; RED 1 a moved anchor refused naming its surface; "
          "RED 2 a wrong book table row refused; RED 3 a stale identity stamp refused)")
    return 0

if len(sys.argv) > 1 and sys.argv[1] == "--self-test":
    sys.exit(self_test())
sys.exit(run())
PYEOF
