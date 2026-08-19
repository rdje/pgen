#!/usr/bin/env python3
"""SV-CORPUS-GRAD.13c.2k (a)+(b) — measure ONE parser arm's cost over the pinned sample, and
diff two arms per RULE.

⛔ WHY THIS EXISTS AND WHY IT IS NOT `price.py`. `.13c.2k` priced its fix before writing it
(`price.py`) and the prediction was **refuted by 16,547,053 entries in the flattering
direction**: the model said −8,958,174 and the parser said +7,588,879. The attribution that
explained the miss had to be reconstructed afterwards from a fixed ten-rule WATCH list, and two
of the three rules that carried the miss were only in that list by luck. ⇒ an arm measurement
must record **every** rule, once, so the attribution is a query against stored data rather than
a re-run with a better guess. `price.py` stays what it is — a PREDICTION instrument. This is the
MEASUREMENT instrument, and it deliberately stores more than any one question needs.

⛔ AND IT EXISTS BECAUSE THE RISE IS CURRENTLY **CONFLATED**. The refuted measurement was taken
on a parser carrying BOTH `.13c.2t` (a `data_type` alternative in `assignment_pattern_key`) and
`.13c.2k` (the identifier guard). A single number over two changes cannot say which one bought
it, and `PARSE-COST-RATCHET`'s two legal outcomes — eliminate, or record as irreducible with the
measurement that proves it — are both unavailable while the split is unknown.

WHAT AN ARM IS: a built `parseability_probe` plus the generated parser inside it. The probe's own
`--parser-fingerprint` is recorded in every arm file, so two arm files can never be compared
without the comparison knowing whether the executables differed. The grammar sha, the sample sha
and the probe path are recorded for the same reason.

MEASURED QUANTITY: the exact integers `PARSE-COST-RATCHET` binds on (TOOLBOX 3.7) — total rule
entries, total memo hits, total committed entries — summed over the pinned 192-file sample
(`stimuli/sv/parse_cost_sample.tsv`), plus the FULL per-rule breakdown from TOOLBOX 3.5
(`rule_entry_counts`, `rule_memo_hit_counts`, `rule_committed_counts`) and the PER-FILE record
including each file's `accepted` verdict. Wall clock is NOT measured here: it is advisory for this
ratchet and has been published wrong twice (`ENGINE-UNIVERSAL-SERVICES.20`).

⛔ THE PER-FILE VERDICT IS NOT A LUXURY — schema 1 omitted it and cost this slice a wrong turn.
Comparing two arms on TOTALS alone, `committed` moved +51,611 between two spellings of the same
fix and there was no way to tell a re-routed derivation from a file whose VERDICT had flipped.
Two arms of the same fix are only comparable if they accept the same inputs, and that is a
per-file fact. Schema 2 records it; `--compare` refuses to mix schemas.

REFUSALS (exit 2), because a partial measurement compared against a whole one is a silent lie:
  * the probe is missing, or prints no `--parser-fingerprint` payload;
  * the sample is empty;
  * ANY sample file fails to produce a dump — the arm is refused, not reported short.

USAGE
  python3 …/arm_cost.py --measure --arm <name> --probe <path> [--outdir <dir>] [--jobs N]
  python3 …/arm_cost.py --compare <base.json> <arm.json> [--top N]
EXIT
  0 = measured / compared · 2 = refused
"""

from __future__ import annotations

import argparse
import concurrent.futures
import hashlib
import importlib.util
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
if not (ROOT / "grammars").is_dir():
    raise SystemExit(f"arm_cost: not at the repo root (derived {ROOT}) — fix the parents[] depth")

SAMPLE = ROOT / "stimuli/sv/parse_cost_sample.tsv"
GRAMMAR = ROOT / "grammars/systemverilog.ebnf"
DEFAULT_OUTDIR = ROOT / "docs/tasks/artifacts/sv_corpus_grad/strictness_cost_arms"
GRAMMAR_NAME = "systemverilog"
PROFILE = "sv_2017"
TIMEOUT_S = 300


# ⛔ AN ARM'S IDENTITY MUST NOT DEPEND ON TREE STATE. The first schema-2 re-measure of `designA`
# ran after the driver's trap had restored the grammar, so the arm file recorded HEAD's grammar
# sha against a designA parser — a field that says the wrong thing while the parser fingerprint
# beside it says the right one. The arm's grammar sha is therefore DERIVED from the arm NAME via
# `apply_arm.py`'s in-memory transform (the same derivation `arm_graph.py` uses), and the live
# grammar is recorded separately, as what it is.
ARM_ALIASES = {"arm0_head": "head", "arm0_head_debug": "head", "arm0_head_release": "head",
               "head": "head", "t_only": "t_only", "designB": "designB", "designA": "designA", "designC": "designC"}


def derived_arm_grammar_sha(arm: str) -> str | None:
    canonical = ARM_ALIASES.get(arm)
    if canonical is None:
        return None
    spec = importlib.util.spec_from_file_location(
        "arm_graph", Path(__file__).resolve().parent / "arm_graph.py")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return hashlib.sha256(mod.arm_text(canonical).encode("utf-8")).hexdigest()


def sha256_of(path: Path) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def sample_files() -> list[str]:
    rows: list[str] = []
    for line in SAMPLE.read_text(encoding="utf-8").splitlines():
        if not line or line.startswith("#"):
            continue
        parts = line.split("\t")
        if len(parts) >= 2:
            rows.append(parts[1])
    return rows


def probe_fingerprint(probe: Path) -> str | None:
    try:
        out = subprocess.run([str(probe), "--parser-fingerprint"],
                             capture_output=True, timeout=120, text=True).stdout
        return json.loads(out).get("parsers", {}).get(GRAMMAR_NAME)
    except Exception:
        return None


def measure_one(args: tuple[Path, str]) -> tuple[str, dict | None]:
    probe, rel = args
    fd, tmp = tempfile.mkstemp(suffix=".json", dir=str(ROOT / "rust" / "target"))
    os.close(fd)
    try:
        subprocess.run(
            [str(probe), "--parse", GRAMMAR_NAME, str(ROOT / rel), "--profile", PROFILE,
             "--dump-rule-outcome-counts-json", tmp],
            capture_output=True, timeout=TIMEOUT_S,
        )
        if os.path.getsize(tmp) == 0:
            return rel, None
        with open(tmp, encoding="utf-8") as fh:
            return rel, json.load(fh)
    except (subprocess.TimeoutExpired, json.JSONDecodeError, OSError):
        return rel, None
    finally:
        try:
            os.unlink(tmp)
        except OSError:
            pass


def do_measure(a: argparse.Namespace) -> int:
    probe = Path(a.probe) if a.probe else (ROOT / "rust/target/release/parseability_probe")
    if not probe.exists():
        print(f"arm_cost: no probe at {probe} — build it first", file=sys.stderr)
        return 2
    fp = probe_fingerprint(probe)
    if not fp:
        print(f"arm_cost: {probe} printed no --parser-fingerprint payload; refusing", file=sys.stderr)
        return 2

    files = sample_files()
    if not files:
        print("arm_cost: EMPTY sample — refusing", file=sys.stderr)
        return 2

    entries: dict[str, int] = {}
    hits: dict[str, int] = {}
    committed: dict[str, int] = {}
    per_file: dict[str, dict] = {}
    total_entries = total_hits = total_committed = 0
    failed: list[str] = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=a.jobs) as ex:
        for rel, d in ex.map(measure_one, [(probe, f) for f in files]):
            if d is None:
                failed.append(rel)
                continue
            total_entries += int(d.get("total_entries", 0))
            total_hits += int(d.get("total_memo_hits", 0))
            total_committed += int(d.get("total_committed", 0))
            per_file[rel] = {
                "accepted": bool(d.get("accepted", False)),
                "entries": int(d.get("total_entries", 0)),
                "memo_hits": int(d.get("total_memo_hits", 0)),
                "committed": int(d.get("total_committed", 0)),
            }
            for r, v in d.get("rule_entry_counts", {}).items():
                entries[r] = entries.get(r, 0) + int(v)
            for r, v in d.get("rule_memo_hit_counts", {}).items():
                hits[r] = hits.get(r, 0) + int(v)
            for r, v in d.get("rule_committed_counts", {}).items():
                committed[r] = committed.get(r, 0) + int(v)

    if failed:
        print(f"arm_cost: {len(failed)} of {len(files)} sample files produced NO dump — refusing "
              f"(a short arm compared against a whole one is a silent lie). First: {failed[:3]}",
              file=sys.stderr)
        return 2

    rec = {
        "schema": 2,
        "arm": a.arm,
        "profile": PROFILE,
        "files": len(files),
        "probe": os.path.relpath(probe, ROOT),
        "probe_parser_fingerprint": fp,
        "arm_grammar_sha256": derived_arm_grammar_sha(a.arm),
        "live_grammar_sha256_at_measure_time": sha256_of(GRAMMAR),
        "sample_sha256": sha256_of(SAMPLE),
        "total_entries": total_entries,
        "total_memo_hits": total_hits,
        "total_committed": total_committed,
        "accepted_files": sum(1 for v in per_file.values() if v["accepted"]),
        "rule_entries": entries,
        "rule_memo_hits": hits,
        "rule_committed": committed,
        "per_file": per_file,
    }
    outdir = Path(a.outdir) if a.outdir else DEFAULT_OUTDIR
    outdir.mkdir(parents=True, exist_ok=True)
    out = outdir / f"{a.arm}.json"
    out.write_text(json.dumps(rec, indent=1, sort_keys=True) + "\n", encoding="utf-8")

    print(f"ARM-COST: arm={a.arm} files={len(files)} profile={PROFILE}")
    print(f"  entries   = {total_entries:>14,}")
    print(f"  memo_hits = {total_hits:>14,}")
    print(f"  committed = {total_committed:>14,}")
    print(f"  rules seen= {len(entries):>14,}")
    print(f"  accepted  = {rec['accepted_files']:>14,} / {len(files)} sample files")
    print(f"  parser    = {fp}")
    print(f"  grammar   = {rec['arm_grammar_sha256']}  (DERIVED from the arm name, not "
          f"from the tree)")
    print(f"  -> {os.path.relpath(out, ROOT)}")
    return 0


def do_compare(a: argparse.Namespace) -> int:
    base = json.loads(Path(a.compare[0]).read_text(encoding="utf-8"))
    arm = json.loads(Path(a.compare[1]).read_text(encoding="utf-8"))
    for k in ("files", "sample_sha256", "profile"):
        if base[k] != arm[k]:
            print(f"arm_cost: arms disagree on {k} ({base[k]} vs {arm[k]}) — refusing",
                  file=sys.stderr)
            return 2
    if base.get("schema", 1) != arm.get("schema", 1):
        print(f"arm_cost: schema {base.get('schema', 1)} vs {arm.get('schema', 1)} — refusing to "
              f"mix; re-measure the older arm", file=sys.stderr)
        return 2

    print(f"ARM-DIFF: {base['arm']}  ->  {arm['arm']}    ({base['files']} files, {base['profile']})")
    if base["probe_parser_fingerprint"] == arm["probe_parser_fingerprint"]:
        print("  ⚠️ SAME parser fingerprint in both arms — one of them was measured with the "
              "WRONG probe unless the change is genuinely parser-neutral")
    print()
    print(f"  {'counter':<12} {'before':>14} {'after':>14} {'delta':>14} {'pct':>9}")
    for k, label in (("total_entries", "entries"), ("total_memo_hits", "memo_hits"),
                     ("total_committed", "committed")):
        b, x = base[k], arm[k]
        pct = (100.0 * (x - b) / b) if b else 0.0
        print(f"  {label:<12} {b:>14,} {x:>14,} {x - b:>+14,} {pct:>+8.3f}%")
    if "per_file" in base and "per_file" in arm:
        flips = [(f, base["per_file"][f]["accepted"], arm["per_file"][f]["accepted"])
                 for f in base["per_file"]
                 if f in arm["per_file"]
                 and base["per_file"][f]["accepted"] != arm["per_file"][f]["accepted"]]
        if flips:
            print(f"\n  ⛔ {len(flips)} sample file(s) CHANGED VERDICT — the arms do not accept the "
                  f"same language, so their costs are not comparable as two spellings of one fix:")
            for f, b, x in flips[:20]:
                print(f"     {('ACCEPT' if b else 'REJECT')} -> {('ACCEPT' if x else 'REJECT')}  {f}")
        else:
            print(f"\n  ✅ all {len(base['per_file'])} sample files keep their verdict — the arms "
                  f"accept the same language over the sample")
    d_e = arm["total_entries"] - base["total_entries"]
    d_c = arm["total_committed"] - base["total_committed"]
    if d_e == 0 and d_c == 0:
        verdict = "no movement at all"
    elif abs(d_c) * 20 < abs(d_e):
        verdict = "FAILED SPECULATION — the accepted derivations did not get more expensive"
    else:
        verdict = "mixed: committed moved too"
    print(f"\n  entries − committed moves {d_e - d_c:+,} ⇒ {verdict}")

    keys = set(base["rule_entries"]) | set(arm["rule_entries"])
    movers = sorted(
        ((r, arm["rule_entries"].get(r, 0) - base["rule_entries"].get(r, 0)) for r in keys),
        key=lambda t: -abs(t[1]))
    shown = [m for m in movers if m[1] != 0][: a.top]
    print(f"\n  {'rule':<46} {'before':>13} {'after':>13} {'delta':>13}")
    for r, d in shown:
        print(f"  {r:<46} {base['rule_entries'].get(r, 0):>13,} "
              f"{arm['rule_entries'].get(r, 0):>13,} {d:>+13,}")
    rest = sum(d for _, d in movers if d != 0) - sum(d for _, d in shown)
    print(f"  {'(all other moving rules)':<46} {'':>13} {'':>13} {rest:>+13,}")
    print(f"  {'TOTAL over rules':<46} {'':>13} {'':>13} "
          f"{sum(d for _, d in movers):>+13,}   (must equal entries delta {d_e:+,})")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--measure", action="store_true")
    ap.add_argument("--arm")
    ap.add_argument("--probe")
    ap.add_argument("--outdir")
    ap.add_argument("--jobs", type=int, default=max(1, (os.cpu_count() or 4) - 2))
    ap.add_argument("--compare", nargs=2, metavar=("BASE", "ARM"))
    ap.add_argument("--top", type=int, default=20)
    a = ap.parse_args()
    if a.measure:
        if not a.arm:
            print("arm_cost: --measure needs --arm <name>", file=sys.stderr)
            return 2
        return do_measure(a)
    if a.compare:
        return do_compare(a)
    ap.print_help()
    return 2


if __name__ == "__main__":
    sys.exit(main())
