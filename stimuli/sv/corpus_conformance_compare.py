#!/usr/bin/env python3
"""Compare a FRESH adjudication of the SystemVerilog external corpus against the TRACKED one
(DONE-BAR.3b — the SV external-corpus CONFORMANCE gate's judgement).

Two different bits, deliberately kept apart:

  * `regressions` — rows where the recorded answer-key agreement got WORSE: a tracked `match` that
    no longer matches, or a tracked `divergence:explained_*` that is now `divergence:unexplained_*`.
    The gate's EXIT CODE is this bit: any regression fails it. (`observed` pass→fail flips on rows
    with no verdict are reported as `observed_pass_to_fail`, never counted as regressions — they have
    no key to regress against; the corpus-census and denominator doctrines own them.)
  * `full_pass` — the DONE-BAR leg-3 assertion, *"the external corpus, PASSING"*: every row has a
    verdict and no unexplained divergence remains. False today (268 known defects, 6 174 no-verdict
    rows) and it stays false until they are burnt down; the register's `pass_query` reads THIS bit.
    ⛔ An artifact's PRESENCE is not a pass — DONE-BAR.3b measured the audit taking it for one.

The row sets must be identical (a corpus file that vanished or appeared is a corpus change, not a
parser verdict) — otherwise the comparison REFUSES (exit 2).

Usage:
    python3 stimuli/sv/corpus_conformance_compare.py --tracked A.tsv --fresh B.tsv \
        [--tracked-v2005 A5.tsv --fresh-v2005 B5.tsv] --out comparison.json
    python3 stimuli/sv/corpus_conformance_compare.py --self-test
"""
import argparse, csv, importlib.util, json, os, sys

MATCH = "match"
UNEXPLAINED = "divergence:unexplained_"
EXPLAINED = "divergence:explained_"


def _bucket_of():
    """The class->bucket map, IMPORTED from the verdict-coverage instrument — never re-declared here
    (SV-CORPUS-DENOMINATOR's own rule: a second copy of the map is a second thing to drift). The first
    cut counted every `deferred:*` row as no-verdict and read 8 780 where the published tuple says
    6 174 + 2 606 ROUTED (the v2005-lane rows, answered in the other lane)."""
    here = os.path.dirname(os.path.abspath(__file__))
    spec = importlib.util.spec_from_file_location("pgen_corpus_verdict_coverage", os.path.join(here, "corpus_verdict_coverage.py"))
    mod = importlib.util.module_from_spec(spec); sys.modules["pgen_corpus_verdict_coverage"] = mod; spec.loader.exec_module(mod)
    return lambda cls: mod.CLASSES[cls][0] if cls in mod.CLASSES else None


BUCKET_OF = None


def read_manifest(path):
    with open(path, encoding="utf-8") as fh:
        return {(r["suite"], r["relpath"]): r for r in csv.DictReader(fh, delimiter="\t")}


def _prefix_bucket(cls):
    """The verilog_2005 lane's buckets. That lane has no coverage instrument of its own (the published
    tuple is the sv_2017 lane's, and its ROUTED rows ARE this lane), so buckets come from the class
    prefix, stated here: `deferred:*` = NO VERDICT, `divergence:*` and `match` = ADJUDICATED."""
    if cls == MATCH or cls.startswith("divergence:"):
        return "ADJUDICATED"
    if cls.startswith("deferred:"):
        return "NO VERDICT"
    return None


def compare(tracked, fresh, lane="sv_2017"):
    """Pure. tracked/fresh: {(suite, relpath): row} with `observed` and `adjudication`. `lane`
    selects the bucket map: the sv_2017 lane uses the verdict-coverage instrument's CLASSES (imported,
    refusing an unbucketed class); the verilog_2005 lane uses the prefix rule above."""
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
    global BUCKET_OF
    if lane == "sv_2017":
        if BUCKET_OF is None:
            BUCKET_OF = _bucket_of()
        bucket = BUCKET_OF
    else:
        bucket = _prefix_bucket
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
    return {
        "rows": n, "match": match, "explained": explained, "unexplained": unexplained,
        "routed": routed, "no_verdict": no_verdict, "adjudicated": adjudicated,
        "adjudicated_pass_share": (round(match / adjudicated, 6) if adjudicated else None),
        "regressions": len(regressions), "regression_rows": regressions[:50],
        "improvements": len(improvements), "improvement_rows": improvements[:50],
        "observed_pass_to_fail": len(observed_p2f), "observed_fail_to_pass": len(observed_f2p),
        "full_pass": (unexplained == 0 and no_verdict == 0 and n > 0),
        "tracked_equals_fresh": all(tracked[k]["adjudication"] == fresh[k]["adjudication"]
                                    and tracked[k]["observed"] == fresh[k]["observed"] for k in tracked),
    }


def assert_tracked(tracked, lane="sv_2017"):
    """DONE-BAR.3c: the leg-3 assertion over ONE manifest — `compare(m, m)` restricted to the fields the
    assertion is made of. `full_pass` is the comparator's own definition (every row adjudicated, zero
    unexplained divergences), so a TRACKED-ONLY reader and the artifact's reader ask one question."""
    c = compare(tracked, tracked, lane=lane)
    if "refused" in c:
        return {"refused": c["refused"], "lane": lane}
    return {k: c[k] for k in ("rows", "match", "explained", "unexplained", "routed", "no_verdict",
                              "adjudicated", "adjudicated_pass_share", "full_pass")} | {"lane": lane, "source": "tracked manifest"}


def self_test():
    ok = True
    def arm(label, cond):
        nonlocal ok
        print(("  ✓ " if cond else "  ✗ ") + label); ok = ok and cond
    r = lambda obs, adj: {"observed": obs, "adjudication": adj}
    base = {("s", "a"): r("pass", MATCH), ("s", "b"): r("fail", "divergence:unexplained_rejects_valid"),
            ("s", "c"): r("fail", "divergence:explained_svpp_include"), ("s", "d"): r("pass", "deferred:chained_only")}
    same = compare(base, base)
    arm("identical manifests: 0 regressions, tracked_equals_fresh, full_pass False (an unexplained and a deferred row remain)",
        same["regressions"] == 0 and same["tracked_equals_fresh"] and same["full_pass"] is False and same["unexplained"] == 1 and same["no_verdict"] == 1)
    routed = {("s", "a"): r("pass", MATCH), ("s", "v"): r("pass", "deferred:v2005_profile_lane")}
    rt = compare(routed, routed)
    arm("a ROUTED row (deferred:v2005_profile_lane) is counted as routed, not no-verdict — the instrument's bucket map, imported",
        rt["routed"] == 1 and rt["no_verdict"] == 0)
    arm("an adjudication class with no bucket REFUSES on the sv_2017 lane (the instrument's own refusal, not a silent count)",
        "refused" in compare({("s", "a"): r("pass", "deferred:made_up")}, {("s", "a"): r("pass", "deferred:made_up")}))
    v5 = {("s", "a"): r("pass", MATCH), ("s", "i"): r("pass", "deferred:impl_varying_v2005")}
    v = compare(v5, v5, lane="verilog_2005")
    arm("the verilog_2005 lane buckets by prefix: deferred:impl_varying_v2005 is no-verdict, nothing routed, not refused",
        "refused" not in v and v["no_verdict"] == 1 and v["routed"] == 0)
    worse = dict(base); worse[("s", "a")] = r("fail", "divergence:unexplained_rejects_valid")
    w = compare(base, worse)
    arm("RED: a tracked match that stops matching is a regression (exit-code bit)", w["regressions"] == 1 and w["regression_rows"][0]["relpath"] == "a")
    worse2 = dict(base); worse2[("s", "c")] = r("fail", "divergence:unexplained_rejects_valid")
    arm("RED: explained -> unexplained is a regression", compare(base, worse2)["regressions"] == 1)
    better = dict(base); better[("s", "b")] = r("pass", MATCH)
    b = compare(base, better)
    arm("improvement counted, not a regression; observed fail->pass counted", b["regressions"] == 0 and b["improvements"] == 1 and b["observed_fail_to_pass"] == 1)
    flip = dict(base); flip[("s", "d")] = r("fail", "deferred:chained_only")
    fl = compare(base, flip)
    arm("an observed pass->fail on a NO-VERDICT row is reported, never a regression (no key)", fl["regressions"] == 0 and fl["observed_pass_to_fail"] == 1)
    full = {("s", "a"): r("pass", MATCH), ("s", "c"): r("fail", "divergence:explained_svpp_include")}
    arm("full_pass is true only with 0 unexplained AND 0 no-verdict rows (explained rows allowed)", compare(full, full)["full_pass"] is True)
    arm("a row-set difference REFUSES the comparison", "refused" in compare(base, {k: v for k, v in base.items() if k != ("s", "d")}))
    arm("presence is not a pass: the same artifact shape can read full_pass False and True", same["full_pass"] is False and compare(full, full)["full_pass"] is True)
    at = assert_tracked(base)
    arm("DONE-BAR.3c: --assert-tracked over one manifest is compare(m, m)'s assertion — full_pass False with 1 unexplained + 1 no-verdict, counts carried",
        at["full_pass"] is False and at["unexplained"] == 1 and at["no_verdict"] == 1 and at["rows"] == 4 and at["adjudicated"] == 3 and at["source"] == "tracked manifest")
    arm("DONE-BAR.3c: --assert-tracked reads True on an all-adjudicated manifest, and REFUSES (never scores) an unbucketed class",
        assert_tracked(full)["full_pass"] is True and "refused" in assert_tracked({("s", "a"): r("pass", "deferred:made_up")}))
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
                         "over ONE tracked manifest alone — the same `compare()` with the manifest as both sides, "
                         "so the audit's TRACKED-ONLY reading can evaluate the register's pass_query without an artifact")
    ap.add_argument("--lane", default="sv_2017")
    a = ap.parse_args()
    if a.self_test:
        self_test()
    if a.assert_tracked:
        out = assert_tracked(read_manifest(a.assert_tracked), lane=a.lane)
        json.dump(out, sys.stdout, indent=1); print()
        sys.exit(2 if "refused" in out else 0)
    if not (a.tracked and a.fresh and a.out):
        ap.error("--tracked, --fresh and --out are required")
    out = {"sv_2017": compare(read_manifest(a.tracked), read_manifest(a.fresh))}
    if a.tracked_v2005 and a.fresh_v2005:
        out["verilog_2005"] = compare(read_manifest(a.tracked_v2005), read_manifest(a.fresh_v2005), lane="verilog_2005")
    refused = [k for k, v in out.items() if "refused" in v]
    lanes = [v for v in out.values() if "refused" not in v]
    out["regressions_total"] = sum(v["regressions"] for v in lanes)
    out["improvements_total"] = sum(v["improvements"] for v in lanes)
    out["full_pass"] = bool(lanes) and not refused and all(v["full_pass"] for v in lanes)
    json.dump(out, open(a.out, "w", encoding="utf-8"), indent=1)
    if refused:
        for k in refused:
            print(f"corpus-conformance: REFUSING lane {k}: {out[k]['refused']}", file=sys.stderr)
        sys.exit(2)
    for k, v in out.items():
        if isinstance(v, dict):
            print(f"corpus-conformance[{k}]: rows={v['rows']} match={v['match']} explained={v['explained']} "
                  f"unexplained={v['unexplained']} routed={v['routed']} no_verdict={v['no_verdict']} regressions={v['regressions']} "
                  f"improvements={v['improvements']} observed_pass_to_fail={v['observed_pass_to_fail']} full_pass={v['full_pass']}")
    if out["regressions_total"]:
        print(f"corpus-conformance: FAIL — {out['regressions_total']} regression(s) against the recorded answer-key agreement", file=sys.stderr)
        sys.exit(1)
    print(f"corpus-conformance: OK — 0 regressions; full_pass={out['full_pass']} (leg 3 reads this bit, not the exit code)")


if __name__ == "__main__":
    main()
