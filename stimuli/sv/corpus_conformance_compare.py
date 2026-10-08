#!/usr/bin/env python3
"""Compare a FRESH adjudication of the SystemVerilog external corpus against the TRACKED one
(DONE-BAR.3b — the SV external-corpus CONFORMANCE gate's judgement).

Two different bits, deliberately kept apart:

  * `regressions` — rows where the recorded answer-key agreement got WORSE: a tracked `match` that
    no longer matches, or a tracked `divergence:explained_*` that is now `divergence:unexplained_*`.
    The gate's EXIT CODE is this bit: any regression fails it. (`observed` pass→fail flips on rows
    with no verdict are reported as `observed_pass_to_fail`, never counted as regressions — they have
    no key to regress against; the corpus-census and denominator doctrines own them.)
  * `full_pass` — the DONE-BAR leg-3 assertion, *"the external corpus, PASSING"*: zero unexplained
    divergences, and every row adjudicated UNDER ITS DECLARED READING; the register's `pass_query`
    reads THIS bit. ⛔ An artifact's PRESENCE is not a pass — DONE-BAR.3b measured the audit taking
    it for one.

    ⛔ DONE-BAR.3f (2026-10-08): until this leaf the bit was `unexplained == 0 and no_verdict == 0`
    over the RAW sv_2017 lane, and that was UNREACHABLE BY CONSTRUCTION — no parser could ever meet
    it. The raw lane's NO VERDICT bucket holds the five PERMANENT classes (SV-CORPUS-GRAD.13f: rows
    no parser, key or corpus work can ever give a verdict — a $readmemh image, Verilog-AMS, a verdict
    the suite's own contract leaves to the implementation — bounded by a ceilings register the
    coverage instrument refuses to exceed) and the `deferred:chained_only` rows the raw lane defers
    to the preprocess→parse pipeline BY DESIGN (SVPP-EXPANSION.6c.1). Measured on the tracked
    manifest that day: 6 245 no-verdict rows, of which 716 permanent and 4 933 chained_only — 5 649
    rows the bit demanded a verdict for that no work on the parser could supply. The verilog_2005
    lane carried 29 more (impl_varying / NI) under the same arguments and no bound at all.

    So the bit now reads each row under its DECLARED reading: the pipeline re-adjudication
    (`stimuli/sv/characterization/pipeline_readjudication.tsv`, held current by
    PIPELINE-LANE-CURRENCY) for a row the pipeline re-read, the raw class otherwise — through the
    coverage instrument's own `pipeline_reading` (ONE definition; the denominator gate's "under the
    pipeline" tuple is the same function) — and the PERMANENT classes of both lanes are named,
    counted and held under their ceilings (above one: REFUSED) instead of being demanded. Nothing is
    relabelled: a permanent row stays NO VERDICT in every published table; `burnable_no_verdict` is
    what still stands between the corpus and the bar, and the raw-lane bit survives by name as
    `full_pass_raw_lane` so the change of meaning is visible.

The row sets must be identical (a corpus file that vanished or appeared is a corpus change, not a
parser verdict) — otherwise the comparison REFUSES (exit 2).

Usage:
    python3 stimuli/sv/corpus_conformance_compare.py --tracked A.tsv --fresh B.tsv \\
        [--tracked-v2005 A5.tsv --fresh-v2005 B5.tsv] --out comparison.json
    python3 stimuli/sv/corpus_conformance_compare.py --assert-tracked A.tsv [--assert-tracked-v2005 A5.tsv]
    python3 stimuli/sv/corpus_conformance_compare.py --self-test
`--readjudication` / `--ceilings` default to the tracked table and register; the declared reading is
NOT EVALUATED (never a pass) without the table and REFUSES without the register.
"""
import argparse, csv, importlib.util, json, os, sys

MATCH = "match"
UNEXPLAINED = "divergence:unexplained_"
EXPLAINED = "divergence:explained_"
HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
DEFAULT_READJUDICATION = os.path.join(HERE, "characterization", "pipeline_readjudication.tsv")
DEFAULT_CEILINGS = os.path.join(ROOT, "docs", "tasks", "artifacts", "sv_corpus_grad", "verdict_coverage",
                                "permanent_deferral_ceilings.json")
COV = None


def _coverage():
    """The verdict-coverage instrument, IMPORTED — never re-declared here (SV-CORPUS-DENOMINATOR's
    own rule: a second copy of the class map, the pipeline overlay or the permanent set is a second
    thing to drift). The first cut of this script counted every `deferred:*` row as no-verdict and
    read 8 780 where the published tuple says 6 174 + 2 606 ROUTED (the v2005-lane rows, answered in
    the other lane)."""
    global COV
    if COV is None:
        spec = importlib.util.spec_from_file_location("pgen_corpus_verdict_coverage", os.path.join(HERE, "corpus_verdict_coverage.py"))
        mod = importlib.util.module_from_spec(spec); sys.modules["pgen_corpus_verdict_coverage"] = mod; spec.loader.exec_module(mod)
        COV = mod
    return COV


def _bucket_of():
    cov = _coverage()
    return lambda cls: cov.CLASSES[cls][0] if cls in cov.CLASSES else None


def read_manifest(path):
    with open(path, encoding="utf-8") as fh:
        return {(r["suite"], r["relpath"]): r for r in csv.DictReader(fh, delimiter="\t")}


def read_readjudication(path):
    """(suite, relpath) -> {verdict, pipeline_status, raw_class} from the re-adjudication table
    (`stimuli/sv/pipeline_readjudication.py`); None when the table is absent — the declared reading
    is then NOT EVALUATED, which is never a pass."""
    if not path or not os.path.isfile(path):
        return None
    with open(path, encoding="utf-8") as fh:
        return {(r["suite"], r["relpath"]): {"verdict": r["verdict"], "pipeline_status": r["pipeline_status"],
                                             "raw_class": r["raw_class"]} for r in csv.DictReader(fh, delimiter="\t")}


def read_ceilings(path):
    """{"sv_2017": {class: ceiling}, "verilog_2005": {class: ceiling}} from the .13f register; None
    when it is absent — a permanent class the assertion excludes must be bounded, so the reading
    then REFUSES."""
    if not path or not os.path.isfile(path):
        return None
    with open(path, encoding="utf-8") as fh:
        reg = json.load(fh)
    lane = lambda section: {c: int(v["ceiling"]) for c, v in reg.get(section, {}).items() if isinstance(v, dict) and "ceiling" in v}
    return {"sv_2017": lane("ceilings"), "verilog_2005": lane("ceilings_verilog_2005")}


def _prefix_bucket(cls):
    """The verilog_2005 lane's buckets. That lane has no coverage instrument of its own (the published
    tuple is the sv_2017 lane's, and its ROUTED rows ARE this lane), so buckets come from the class
    prefix, stated here: `deferred:*` = NO VERDICT, `divergence:*` and `match` = ADJUDICATED."""
    if cls == MATCH or cls.startswith("divergence:"):
        return "ADJUDICATED"
    if cls.startswith("deferred:"):
        return "NO VERDICT"
    return None


def _bound_permanent(counts_by_class, permanent, ceilings, lane):
    """(permanent_total, by_class) or {"refused": why}: every PERMANENT class of the lane must have a
    ceiling and must sit at or under it (SV-CORPUS-GRAD.13f: a permanent deferral may not grow
    silently, and a class this assertion stops demanding a verdict for must be bounded)."""
    if ceilings is None:
        return {"refused": "no permanent-deferral ceilings register (docs/tasks/artifacts/sv_corpus_grad/"
                           "verdict_coverage/permanent_deferral_ceilings.json) — a PERMANENT class the "
                           "assertion excludes must be bounded"}
    lane_ceil = ceilings.get(lane, {})
    missing = [c for c in permanent if c not in lane_ceil]
    if missing:
        return {"refused": f"the ceilings register records no {lane} ceiling for {missing}; every PERMANENT "
                           f"class must be bounded (SV-CORPUS-GRAD.13f)"}
    by_class = {c: counts_by_class.get(c, 0) for c in permanent}
    over = [(c, n, lane_ceil[c]) for c, n in by_class.items() if n > lane_ceil[c]]
    if over:
        return {"refused": f"a PERMANENT {lane} class holds more rows than its recorded ceiling — "
                           + ", ".join(f"`{c}` {n} > {ceil}" for c, n, ceil in over)
                           + " (SV-CORPUS-GRAD.13f: name the argument the new rows join and raise the "
                             "ceiling in the register; never raise it to make a run green)"}
    return sum(by_class.values()), by_class


def declared_reading(fresh, readj, ceilings):
    """Pure. The sv_2017 rows read under their DECLARED reading (DONE-BAR.3f). Returns the declared
    counts, or {"refused": why} when the reading cannot be scored honestly."""
    cov = _coverage()
    not_evaluated = {"declared_evaluated": False, "declared_adjudicated": 0, "declared_routed": 0,
                     "declared_no_verdict": 0, "declared_bar": 0, "declared_dark": 0, "permanent_no_verdict": 0,
                     "permanent_by_class": {}, "burnable_no_verdict": 0, "readjudication_rows": 0,
                     "readjudication_stale": 0, "readjudication_stale_rows": []}
    if readj is None:
        return not_evaluated | {"declared_reading": "NOT EVALUATED — no pipeline re-adjudication table "
                                "(stimuli/sv/characterization/pipeline_readjudication.tsv); a reading that cannot "
                                "be evaluated is not a pass"}
    # a table row whose recorded raw class differs from the fresh adjudication describes a row the
    # fresh run no longer has: it is STALE, counted, never scored under the pipeline
    stale = sorted(k for k in fresh if k in readj and readj[k]["raw_class"] != fresh[k]["adjudication"])
    stale_set = set(stale)
    overlay = {k: (v["verdict"], v["pipeline_status"]) for k, v in readj.items() if k in fresh and k not in stale_set}
    rows = [{"suite": k[0], "relpath": k[1], "adjudication": r["adjudication"], "observed": r["observed"]}
            for k, r in sorted(fresh.items())]
    try:
        counts, by_bucket, bar, dark = cov.pipeline_reading(rows, overlay)
    except SystemExit as e:
        return {"refused": f"the coverage instrument refused the pipeline reading: {e}"}
    bound = _bound_permanent(counts, cov.PERMANENT, ceilings, "sv_2017")
    if isinstance(bound, dict):
        return bound
    permanent, by_class = bound
    return {"declared_evaluated": True,
            "declared_reading": "the pipeline re-adjudication for a row the pipeline re-read, the raw class otherwise "
                                "(corpus_verdict_coverage.pipeline_reading — the denominator gate's tuple under the pipeline)",
            "declared_adjudicated": by_bucket.get("ADJUDICATED", 0), "declared_routed": by_bucket.get("ROUTED", 0),
            "declared_no_verdict": by_bucket.get("NO VERDICT", 0), "declared_bar": bar, "declared_dark": dark,
            "permanent_no_verdict": permanent, "permanent_by_class": by_class,
            "burnable_no_verdict": by_bucket.get("NO VERDICT", 0) - permanent,
            "readjudication_rows": len(overlay), "readjudication_stale": len(stale),
            "readjudication_stale_rows": [{"suite": k[0], "relpath": k[1], "table_raw_class": readj[k]["raw_class"],
                                           "fresh": fresh[k]["adjudication"]} for k in stale[:20]]}


FULL_PASS_REQUIRES = ("zero unexplained divergences on the raw lane and zero bar rows under the declared reading; "
                      "every row adjudicated under its declared reading (the pipeline re-adjudication for a row the "
                      "pipeline re-read, the raw class otherwise) except the PERMANENT classes, which are named and "
                      "held under the ceilings register; a re-adjudication table current with the fresh raw classes; "
                      "the verilog_2005 lane: zero unexplained and every non-permanent row adjudicated (DONE-BAR.3f)")


def compare(tracked, fresh, lane="sv_2017", readj=None, ceilings=None):
    """Pure. tracked/fresh: {(suite, relpath): row} with `observed` and `adjudication`. `lane`
    selects the bucket map: the sv_2017 lane uses the verdict-coverage instrument's CLASSES (imported,
    refusing an unbucketed class) and its declared reading needs `readj` (the re-adjudication table)
    and `ceilings` (the permanent register); the verilog_2005 lane uses the prefix rule above and the
    register's own section."""
    if set(tracked) != set(fresh):
        only_t = sorted(set(tracked) - set(fresh))[:5]; only_f = sorted(set(fresh) - set(tracked))[:5]
        return {"refused": f"row sets differ: {len(set(tracked)-set(fresh))} only in tracked (e.g. {only_t}), "
                           f"{len(set(fresh)-set(tracked))} only in fresh (e.g. {only_f})"}
    regressions, improvements, observed_p2f, observed_f2p = [], [], [], []
    for k in sorted(tracked):
        t, f = tracked[k]["adjudication"], fresh[k]["adjudication"]
        if t == MATCH and f != MATCH:
            regressions.append({"suite": k[0], "relpath": k[1], "tracked": t, "fresh": f})
        elif t.startswith(EXPLAINED) and f.startswith(UNEXPLAINED):
            regressions.append({"suite": k[0], "relpath": k[1], "tracked": t, "fresh": f})
        elif t != MATCH and f == MATCH:
            improvements.append({"suite": k[0], "relpath": k[1], "tracked": t, "fresh": f})
        elif t.startswith(UNEXPLAINED) and f.startswith(EXPLAINED):
            improvements.append({"suite": k[0], "relpath": k[1], "tracked": t, "fresh": f})
        to, fo = tracked[k]["observed"], fresh[k]["observed"]
        if to == "pass" and fo != "pass":
            observed_p2f.append({"suite": k[0], "relpath": k[1], "adjudication": f})
        elif to != "pass" and fo == "pass":
            observed_f2p.append({"suite": k[0], "relpath": k[1], "adjudication": f})
    bucket = _bucket_of() if lane == "sv_2017" else _prefix_bucket
    n = len(fresh)
    match = sum(1 for r in fresh.values() if r["adjudication"] == MATCH)
    unexplained = sum(1 for r in fresh.values() if r["adjudication"].startswith(UNEXPLAINED))
    explained = sum(1 for r in fresh.values() if r["adjudication"].startswith(EXPLAINED))
    unbucketed = sorted({r["adjudication"] for r in fresh.values() if bucket(r["adjudication"]) is None})
    if unbucketed:
        return {"refused": f"adjudication class(es) with no bucket in corpus_verdict_coverage.CLASSES: {unbucketed}"}
    routed = sum(1 for r in fresh.values() if bucket(r["adjudication"]) == "ROUTED")
    no_verdict = sum(1 for r in fresh.values() if bucket(r["adjudication"]) == "NO VERDICT")
    adjudicated = match + unexplained + explained
    raw_full_pass = (unexplained == 0 and no_verdict == 0 and n > 0)
    out = {
        "rows": n, "match": match, "explained": explained, "unexplained": unexplained,
        "routed": routed, "no_verdict": no_verdict, "adjudicated": adjudicated,
        "adjudicated_pass_share": (round(match / adjudicated, 6) if adjudicated else None),
        "regressions": len(regressions), "regression_rows": regressions[:50],
        "improvements": len(improvements), "improvement_rows": improvements[:50],
        "observed_pass_to_fail": len(observed_p2f), "observed_fail_to_pass": len(observed_f2p),
        "full_pass_raw_lane": raw_full_pass,
        "tracked_equals_fresh": all(tracked[k]["adjudication"] == fresh[k]["adjudication"]
                                    and tracked[k]["observed"] == fresh[k]["observed"] for k in tracked),
    }
    if lane == "sv_2017":
        d = declared_reading(fresh, readj, ceilings)
        if "refused" in d:
            return d
        out.update(d)
        out["full_pass"] = bool(d["declared_evaluated"] and n > 0 and unexplained == 0 and d["declared_bar"] == 0
                                and d["burnable_no_verdict"] == 0 and d["readjudication_stale"] == 0)
    else:
        # the verilog_2005 lane has no pipeline re-adjudication (its rows are the sv_2017 lane's ROUTED rows,
        # read raw under the 1364-2005 profile): its PERMANENT classes are bounded in the register's own
        # section and every other no-verdict row is burnable
        cov = _coverage()
        by_class = {}
        for r in fresh.values():
            by_class[r["adjudication"]] = by_class.get(r["adjudication"], 0) + 1
        bound = _bound_permanent(by_class, cov.PERMANENT_V2005, ceilings, "verilog_2005")
        if isinstance(bound, dict):
            return bound
        permanent, per_class = bound
        out.update({"declared_reading": "the raw lane (no pipeline re-adjudication exists for the verilog_2005 lane)",
                    "permanent_no_verdict": permanent, "permanent_by_class": per_class,
                    "burnable_no_verdict": no_verdict - permanent})
        out["full_pass"] = bool(n > 0 and unexplained == 0 and no_verdict - permanent == 0)
    out["full_pass_requires"] = FULL_PASS_REQUIRES
    return out


ASSERTION_FIELDS = ("rows", "match", "explained", "unexplained", "routed", "no_verdict", "adjudicated",
                    "adjudicated_pass_share", "full_pass", "full_pass_raw_lane", "declared_evaluated",
                    "declared_reading", "declared_adjudicated", "declared_routed", "declared_no_verdict",
                    "declared_bar", "permanent_no_verdict", "permanent_by_class", "burnable_no_verdict",
                    "readjudication_stale", "full_pass_requires")


def assert_tracked(tracked, lane="sv_2017", readj=None, ceilings=None):
    """DONE-BAR.3c: the leg-3 assertion over ONE manifest — `compare(m, m)` restricted to the fields the
    assertion is made of. `full_pass` is the comparator's own definition, so a TRACKED-ONLY reader and
    the artifact's reader ask one question."""
    c = compare(tracked, tracked, lane=lane, readj=readj, ceilings=ceilings)
    if "refused" in c:
        return {"refused": c["refused"], "lane": lane}
    return {k: c[k] for k in ASSERTION_FIELDS if k in c} | {"lane": lane, "source": "tracked manifest"}


def assert_tracked_both(tracked, tracked_v2005, readj=None, ceilings=None):
    """DONE-BAR.3f: the assertion over BOTH tracked manifests, shaped like the gate artifact's top level
    (`full_pass` = both lanes; the counts summed) so the audit's `pass_query` reads the same bit from
    the tracked tree and from `summary.json`."""
    a = assert_tracked(tracked, "sv_2017", readj, ceilings)
    b = assert_tracked(tracked_v2005, "verilog_2005", readj, ceilings)
    if "refused" in a or "refused" in b:
        return {"refused": "; ".join(f"{x['lane']}: {x['refused']}" for x in (a, b) if "refused" in x),
                "sv_2017": a, "verilog_2005": b}
    # `rows` is the CORPUS (the sv_2017 manifest): the verilog_2005 manifest's rows are that lane's ROUTED
    # rows, answered under the 1364-2005 profile, not more corpus — so on the real manifests
    # declared_adjudicated + permanent_no_verdict + burnable_no_verdict == rows
    summed = ("unexplained", "permanent_no_verdict", "burnable_no_verdict")
    return {"sv_2017": a, "verilog_2005": b, "full_pass": a["full_pass"] and b["full_pass"],
            "full_pass_raw_lane": a["full_pass_raw_lane"] and b["full_pass_raw_lane"],
            "rows": a["rows"], "routed": a["routed"], "routed_answered_in_verilog_2005": b["rows"],
            "declared_bar": a["declared_bar"], "readjudication_stale": a["readjudication_stale"],
            "declared_adjudicated": a["declared_adjudicated"] + b["adjudicated"],
            "source": "tracked manifests"} | {k: a[k] + b[k] for k in summed}


def self_test():
    ok = True
    cov = _coverage()
    CEIL = {"sv_2017": {c: 10 ** 9 for c in cov.PERMANENT}, "verilog_2005": {c: 10 ** 9 for c in cov.PERMANENT_V2005}}
    design = sorted(cov.DESIGN_SUITES)[0]
    assert "verilator" not in cov.DESIGN_SUITES
    def arm(label, cond):
        nonlocal ok
        print(("  ✓ " if cond else "  ✗ ") + label); ok = ok and cond
    r = lambda obs, adj: {"observed": obs, "adjudication": adj}
    rj = lambda verdict, status, raw: {"verdict": verdict, "pipeline_status": status, "raw_class": raw}
    base = {("s", "a"): r("pass", MATCH), ("s", "b"): r("fail", "divergence:unexplained_rejects_valid"),
            ("s", "c"): r("fail", "divergence:explained_svpp_include"), ("s", "d"): r("pass", "deferred:chained_only")}
    same = compare(base, base, readj={}, ceilings=CEIL)
    arm("identical manifests: 0 regressions, tracked_equals_fresh, full_pass False (an unexplained and a deferred row remain)",
        same["regressions"] == 0 and same["tracked_equals_fresh"] and same["full_pass"] is False and same["unexplained"] == 1 and same["no_verdict"] == 1)
    routed = {("s", "a"): r("pass", MATCH), ("s", "v"): r("pass", "deferred:v2005_profile_lane")}
    rt = compare(routed, routed, readj={}, ceilings=CEIL)
    arm("a ROUTED row (deferred:v2005_profile_lane) is counted as routed, not no-verdict — the instrument's bucket map, imported",
        rt["routed"] == 1 and rt["no_verdict"] == 0 and rt["declared_routed"] == 1 and rt["full_pass"] is True)
    arm("an adjudication class with no bucket REFUSES on the sv_2017 lane (the instrument's own refusal, not a silent count)",
        "refused" in compare({("s", "a"): r("pass", "deferred:made_up")}, {("s", "a"): r("pass", "deferred:made_up")}, readj={}, ceilings=CEIL))
    v5 = {("s", "a"): r("pass", MATCH), ("s", "i"): r("pass", "deferred:impl_varying_v2005")}
    v = compare(v5, v5, lane="verilog_2005", ceilings=CEIL)
    arm("the verilog_2005 lane buckets by prefix: deferred:impl_varying_v2005 is no-verdict, nothing routed, not refused",
        "refused" not in v and v["no_verdict"] == 1 and v["routed"] == 0)
    worse = dict(base); worse[("s", "a")] = r("fail", "divergence:unexplained_rejects_valid")
    w = compare(base, worse, readj={}, ceilings=CEIL)
    arm("RED: a tracked match that stops matching is a regression (exit-code bit)", w["regressions"] == 1 and w["regression_rows"][0]["relpath"] == "a")
    worse2 = dict(base); worse2[("s", "c")] = r("fail", "divergence:unexplained_rejects_valid")
    arm("RED: explained -> unexplained is a regression", compare(base, worse2, readj={}, ceilings=CEIL)["regressions"] == 1)
    better = dict(base); better[("s", "b")] = r("pass", MATCH)
    b = compare(base, better, readj={}, ceilings=CEIL)
    arm("improvement counted, not a regression; observed fail->pass counted", b["regressions"] == 0 and b["improvements"] == 1 and b["observed_fail_to_pass"] == 1)
    flip = dict(base); flip[("s", "d")] = r("fail", "deferred:chained_only")
    fl = compare(base, flip, readj={}, ceilings=CEIL)
    arm("an observed pass->fail on a NO-VERDICT row is reported, never a regression (no key)", fl["regressions"] == 0 and fl["observed_pass_to_fail"] == 1)
    full = {("s", "a"): r("pass", MATCH), ("s", "c"): r("fail", "divergence:explained_svpp_include")}
    arm("full_pass is true only with 0 unexplained AND 0 burnable no-verdict rows (explained rows allowed)", compare(full, full, readj={}, ceilings=CEIL)["full_pass"] is True)
    arm("a row-set difference REFUSES the comparison", "refused" in compare(base, {k: v for k, v in base.items() if k != ("s", "d")}, readj={}, ceilings=CEIL))
    arm("presence is not a pass: the same artifact shape can read full_pass False and True", same["full_pass"] is False and compare(full, full, readj={}, ceilings=CEIL)["full_pass"] is True)
    at = assert_tracked(base, readj={}, ceilings=CEIL)
    arm("DONE-BAR.3c: --assert-tracked over one manifest is compare(m, m)'s assertion — full_pass False with 1 unexplained + 1 no-verdict, counts carried",
        at["full_pass"] is False and at["unexplained"] == 1 and at["no_verdict"] == 1 and at["rows"] == 4 and at["adjudicated"] == 3 and at["source"] == "tracked manifest")
    arm("DONE-BAR.3c: --assert-tracked reads True on an all-adjudicated manifest, and REFUSES (never scores) an unbucketed class",
        assert_tracked(full, readj={}, ceilings=CEIL)["full_pass"] is True and "refused" in assert_tracked({("s", "a"): r("pass", "deferred:made_up")}, readj={}, ceilings=CEIL))
    # --- DONE-BAR.3f: the declared reading ---------------------------------------------------------
    perm_only = {("s", "a"): r("pass", MATCH), ("s", "p"): r("fail", "deferred:not_source_text")}
    po = compare(perm_only, perm_only, readj={}, ceilings=CEIL)
    arm("3f RED-vs-GREEN on one manifest: a PERMANENT row alone — the raw-lane bit stays False (full_pass_raw_lane) while full_pass is True, the row counted permanent, not burnable",
        po["full_pass_raw_lane"] is False and po["full_pass"] is True and po["permanent_no_verdict"] == 1
        and po["permanent_by_class"]["deferred:not_source_text"] == 1 and po["burnable_no_verdict"] == 0 and po["no_verdict"] == 1)
    chained = {("s", "a"): r("pass", MATCH), (design, "x.sv"): r("fail", "deferred:chained_only")}
    ch_ok = compare(chained, chained, readj={(design, "x.sv"): rj("chained:accepted_under_expansion", "pass", "deferred:chained_only")}, ceilings=CEIL)
    arm("3f a chained_only row of a DESIGN suite the pipeline accepts is adjudicated under the declared reading — full_pass True, declared_adjudicated 2, burnable 0",
        ch_ok["full_pass"] is True and ch_ok["declared_adjudicated"] == 2 and ch_ok["burnable_no_verdict"] == 0 and ch_ok["no_verdict"] == 1 and ch_ok["readjudication_rows"] == 1)
    chained_nd = {("s", "a"): r("pass", MATCH), ("verilator", "x.v"): r("fail", "deferred:chained_only")}
    ch_nd = compare(chained_nd, chained_nd, readj={("verilator", "x.v"): rj("chained:accepted_under_expansion", "pass", "deferred:chained_only")}, ceilings=CEIL)
    arm("3f the same verdict outside DESIGN_SUITES keeps NO VERDICT (no per-file expectation) — burnable 1, full_pass False",
        ch_nd["full_pass"] is False and ch_nd["burnable_no_verdict"] == 1 and ch_nd["declared_no_verdict"] == 1)
    ch_none = compare(chained, chained, readj={}, ceilings=CEIL)
    arm("3f a chained_only row the pipeline never re-read is burnable — full_pass False, burnable 1",
        ch_none["full_pass"] is False and ch_none["burnable_no_verdict"] == 1 and ch_none["readjudication_rows"] == 0)
    ch_stale = compare(chained, chained, readj={(design, "x.sv"): rj("chained:accepted_under_expansion", "pass", MATCH)}, ceilings=CEIL)
    arm("3f RED: a table row whose recorded raw class differs from the fresh adjudication is STALE — counted, not scored, full_pass False",
        ch_stale["full_pass"] is False and ch_stale["readjudication_stale"] == 1 and ch_stale["readjudication_rows"] == 0
        and ch_stale["readjudication_stale_rows"][0]["table_raw_class"] == MATCH and ch_stale["burnable_no_verdict"] == 1)
    barred = {("s", "a"): r("pass", MATCH), ("s", "e"): r("fail", "divergence:explained_svpp_include")}
    br = compare(barred, barred, readj={("s", "e"): rj("refuted:parser_question", "fail", "divergence:explained_svpp_include")}, ceilings=CEIL)
    arm("3f RED: a pipeline bar row (refuted:parser_question) fails full_pass with raw unexplained 0 — the declared bar is read",
        br["full_pass"] is False and br["unexplained"] == 0 and br["declared_bar"] == 1)
    over = {"sv_2017": dict(CEIL["sv_2017"], **{"deferred:not_source_text": 0}), "verilog_2005": CEIL["verilog_2005"]}
    arm("3f RED: a PERMANENT class above its ceiling REFUSES (never scores) — the .13f bound, on the comparator too",
        "refused" in compare(perm_only, perm_only, readj={}, ceilings=over))
    arm("3f RED: no ceilings register REFUSES; a missing lane ceiling REFUSES",
        "refused" in compare(perm_only, perm_only, readj={}, ceilings=None)
        and "refused" in compare(perm_only, perm_only, readj={}, ceilings={"sv_2017": {}, "verilog_2005": {}}))
    ne = compare(full, full, readj=None, ceilings=CEIL)
    arm("3f no re-adjudication table: the declared reading is NOT EVALUATED and full_pass is False even on an all-adjudicated manifest",
        ne["full_pass"] is False and ne["declared_evaluated"] is False and ne["declared_reading"].startswith("NOT EVALUATED") and ne["full_pass_raw_lane"] is True)
    v5p = {("s", "a"): r("pass", MATCH), ("s", "i"): r("pass", "deferred:impl_varying_v2005"), ("s", "n"): r("fail", "deferred:ni_unimplemented")}
    vp = compare(v5p, v5p, lane="verilog_2005", ceilings=CEIL)
    v5b = dict(v5p); v5b[("s", "t")] = r("fail", "deferred:negative_stage_triage_v2005")
    vb = compare(v5b, v5b, lane="verilog_2005", ceilings=CEIL)
    arm("3f the verilog_2005 lane: its two PERMANENT classes bounded by the register's own section — full_pass True with only them; a triage row is burnable and fails it",
        vp["full_pass"] is True and vp["permanent_no_verdict"] == 2 and vp["full_pass_raw_lane"] is False
        and vb["full_pass"] is False and vb["burnable_no_verdict"] == 1 and "declared_adjudicated" not in vb)
    arm("3f RED: the verilog_2005 lane's ceiling binds too",
        "refused" in compare(v5p, v5p, lane="verilog_2005", ceilings={"sv_2017": CEIL["sv_2017"], "verilog_2005": {"deferred:impl_varying_v2005": 0, "deferred:ni_unimplemented": 9}}))
    both = assert_tracked_both(perm_only, v5p, readj={}, ceilings=CEIL)
    both_red = assert_tracked_both(perm_only, v5b, readj={}, ceilings=CEIL)
    arm("3f --assert-tracked over BOTH manifests is the artifact's top-level shape: full_pass = both lanes, rows / permanent / burnable summed; one burnable v2005 row turns it False",
        both["full_pass"] is True and both["rows"] == 2 and both["routed_answered_in_verilog_2005"] == 3 and both["permanent_no_verdict"] == 3 and both["burnable_no_verdict"] == 0
        and both_red["full_pass"] is False and both_red["burnable_no_verdict"] == 1 and both_red["sv_2017"]["full_pass"] is True)
    arm("3f a refusal in either lane is the combined assertion's refusal",
        "refused" in assert_tracked_both(perm_only, v5p, readj={}, ceilings=None))
    print(f"corpus_conformance_compare: self-test {'OK' if ok else 'FAILED'}")
    sys.exit(0 if ok else 1)


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--self-test", action="store_true")
    ap.add_argument("--tracked"); ap.add_argument("--fresh")
    ap.add_argument("--tracked-v2005"); ap.add_argument("--fresh-v2005")
    ap.add_argument("--out")
    ap.add_argument("--assert-tracked", metavar="MANIFEST",
                    help="DONE-BAR.3c: print, as JSON on stdout, the leg-3 assertion (`full_pass` and its counts) "
                         "over the tracked manifest alone — the same `compare()` with the manifest as both sides, "
                         "so the audit's TRACKED-ONLY reading can evaluate the register's pass_query without an artifact")
    ap.add_argument("--assert-tracked-v2005", metavar="MANIFEST_V2005",
                    help="DONE-BAR.3f: with --assert-tracked, the verilog_2005 manifest too; the JSON is then the "
                         "artifact's top-level shape (`full_pass` = both lanes)")
    ap.add_argument("--lane", default="sv_2017")
    ap.add_argument("--readjudication", default=DEFAULT_READJUDICATION,
                    help="the pipeline re-adjudication table (DONE-BAR.3f; default: the tracked one; absent → the "
                         "declared reading is NOT EVALUATED, which is never a pass)")
    ap.add_argument("--ceilings", default=DEFAULT_CEILINGS,
                    help="the permanent-deferral ceilings register (SV-CORPUS-GRAD.13f; absent → REFUSED)")
    a = ap.parse_args()
    if a.self_test:
        self_test()
    readj, ceilings = read_readjudication(a.readjudication), read_ceilings(a.ceilings)
    if a.assert_tracked:
        if a.assert_tracked_v2005:
            out = assert_tracked_both(read_manifest(a.assert_tracked), read_manifest(a.assert_tracked_v2005), readj, ceilings)
        else:
            out = assert_tracked(read_manifest(a.assert_tracked), lane=a.lane, readj=readj, ceilings=ceilings)
        json.dump(out, sys.stdout, indent=1); print()
        sys.exit(2 if "refused" in out else 0)
    if not (a.tracked and a.fresh and a.out):
        ap.error("--tracked, --fresh and --out are required")
    out = {"sv_2017": compare(read_manifest(a.tracked), read_manifest(a.fresh), readj=readj, ceilings=ceilings)}
    if a.tracked_v2005 and a.fresh_v2005:
        out["verilog_2005"] = compare(read_manifest(a.tracked_v2005), read_manifest(a.fresh_v2005), lane="verilog_2005", ceilings=ceilings)
    refused = [k for k, v in out.items() if "refused" in v]
    lanes = [v for v in out.values() if "refused" not in v]
    out["regressions_total"] = sum(v["regressions"] for v in lanes)
    out["improvements_total"] = sum(v["improvements"] for v in lanes)
    out["full_pass"] = bool(lanes) and not refused and all(v["full_pass"] for v in lanes)
    out["full_pass_raw_lane"] = bool(lanes) and not refused and all(v["full_pass_raw_lane"] for v in lanes)
    out["permanent_no_verdict"] = sum(v["permanent_no_verdict"] for v in lanes)
    out["burnable_no_verdict"] = sum(v["burnable_no_verdict"] for v in lanes)
    out["full_pass_requires"] = FULL_PASS_REQUIRES
    json.dump(out, open(a.out, "w", encoding="utf-8"), indent=1)
    if refused:
        for k in refused:
            print(f"corpus-conformance: REFUSING lane {k}: {out[k]['refused']}", file=sys.stderr)
        sys.exit(2)
    for k, v in out.items():
        if isinstance(v, dict):
            declared = (f" declared={v['declared_adjudicated']}/{v['declared_routed']}/{v['declared_no_verdict']}/bar={v['declared_bar']}"
                        f" stale={v['readjudication_stale']}" if "declared_adjudicated" in v else "")
            print(f"corpus-conformance[{k}]: rows={v['rows']} match={v['match']} explained={v['explained']} "
                  f"unexplained={v['unexplained']} routed={v['routed']} no_verdict={v['no_verdict']} regressions={v['regressions']} "
                  f"improvements={v['improvements']} observed_pass_to_fail={v['observed_pass_to_fail']}{declared} "
                  f"permanent={v['permanent_no_verdict']} burnable={v['burnable_no_verdict']} "
                  f"full_pass={v['full_pass']} (raw lane {v['full_pass_raw_lane']})")
    if out["regressions_total"]:
        print(f"corpus-conformance: FAIL — {out['regressions_total']} regression(s) against the recorded answer-key agreement", file=sys.stderr)
        sys.exit(1)
    print(f"corpus-conformance: OK — 0 regressions; full_pass={out['full_pass']} "
          f"(leg 3 reads this bit, not the exit code; {out['burnable_no_verdict']} row(s) still await a verdict under their "
          f"declared reading, {out['permanent_no_verdict']} permanent and bounded)")


if __name__ == "__main__":
    main()
