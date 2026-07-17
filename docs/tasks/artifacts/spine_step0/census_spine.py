#!/usr/bin/env python3
"""RGX-0078.5.i.13 THIN-MEMO SPINE STEP-0 — the composite-spine census (PGEN-RGX-0078-0116).

STATIC half: scan the canonical regex artifact (hash-pinned) for the thin-memo carrier
population — every `cascade_match_*` fn containing the `__pgen_thin_key` machinery — and
inventory the per-entry machinery each carrier pays (string-keyed check_cycle, guard
enter/exit, thin-memo probe + epoch validation, tournament checkpoint, ThinDerivMemoEntry
insert with 2x segment `.to_vec()` on success).

DYNAMIC half: aggregate the fresh 8-pattern `--dump-rule-outcome-counts-json` dumps
(PROTOCOL-graph counts; the CASCADE-PLAN-B boundary join maps them onto the bare-graph
cascade populations — identical parse decisions by construction) into per-pattern and
corpus totals for the carrier rules.

Run from the repo root:
    python3 docs/tasks/artifacts/spine_step0/census_spine.py \
        > docs/tasks/artifacts/spine_step0/census_spine.txt

Inputs (must exist):
    generated/regex_parser.rs                              (canonical artifact)
    rust/target/generated_logs/spine_step0/<p>.outcome.json (the 8 fresh dumps)
"""

import hashlib
import json
import re
import sys

ARTIFACT = "generated/regex_parser.rs"
DUMP_DIR = "rust/target/generated_logs/spine_step0"
PATTERNS = [
    "literal_simple",
    "digit_sequence",
    "character_class",
    "alternation",
    "capture_groups",
    "anchor_complex",
    "email_basic",
    "url_simple",
]


def static_census():
    data = open(ARTIFACT, "rb").read()
    digest = hashlib.sha256(data).hexdigest()[:8]
    src = data.decode("utf-8")
    fn_re = re.compile(r"^\s*fn (\w+)\(")
    cur = None
    fns = {}
    for ln in src.split("\n"):
        m = fn_re.match(ln)
        if m:
            cur = m.group(1)
            fns.setdefault(
                cur,
                {"thin": 0, "cycle": 0, "ckpt": 0, "enter": 0, "to_vec": 0, "insert": 0},
            )
        elif cur is not None:
            rec = fns[cur]
            if "__pgen_thin_key" in ln:
                rec["thin"] += 1
            if "check_cycle" in ln:
                rec["cycle"] += 1
            if ".checkpoint()" in ln:
                rec["ckpt"] += 1
            if "recursion_guard.enter" in ln:
                rec["enter"] += 1
            if ".to_vec()" in ln:
                rec["to_vec"] += 1
            if "ThinDerivMemoEntry {" in ln:
                rec["insert"] += 1
    carriers = sorted(
        f for f, v in fns.items() if f.startswith("cascade_match_") and v["thin"] > 0
    )
    cascade_total = sum(1 for f in fns if f.startswith("cascade_match_"))
    return digest, fns, carriers, cascade_total


def main():
    digest, fns, carriers, cascade_total = static_census()
    rules = [f[len("cascade_match_"):] for f in carriers]

    print("THIN-MEMO SPINE CENSUS — RGX-0078.5.i.13 STEP-0 (PGEN-RGX-0078-0116)")
    print(f"artifact: {ARTIFACT} sha256[:8]={digest}")
    print()
    print("== STATIC: the thin-memo carrier population ==")
    print(
        f"cascade_match_* fns total={cascade_total}; thin-memo carriers (cyclic fused "
        f"internals)={len(carriers)}"
    )
    print(
        "per-entry machinery (every carrier): string-keyed recursion_guard.check_cycle"
        " + guard enter/exit + (RULE_ID,pos) HashMap thin-memo probe + epoch/deferred"
        " validation + ThinDerivMemoEntry insert on EVERY completion (success AND"
        " failure); success additionally clones the event+boundary tape segments"
        " (.to_vec() x2)."
    )
    ckpt_carriers = [f for f in carriers if fns[f]["ckpt"] > 0]
    print(
        f"carriers with a tournament checkpoint() preamble: {len(ckpt_carriers)} "
        f"({', '.join(f[len('cascade_match_'):] for f in ckpt_carriers)})"
    )
    print()
    print("carrier list (rule / thin_refs / to_vec_sites / insert_sites / ckpt):")
    for f in carriers:
        v = fns[f]
        print(
            f"  {f[len('cascade_match_'):]:38s} thin={v['thin']} to_vec={v['to_vec']}"
            f" insert={v['insert']} ckpt={v['ckpt']}"
        )

    print()
    print("== DYNAMIC: the 8-pattern outcome-dump aggregation (PROTOCOL-graph counts;")
    print("   boundary-join onto the bare cascade graph per the .5.i.9 method) ==")
    header = (
        f"{'pattern':16s} {'total_e':>8s} {'spine_e':>8s} {'spine%':>7s}"
        f" {'atom_e':>7s} {'atom_h':>7s} {'piece_e':>8s} {'piece_h':>8s} {'other_sp':>9s}"
    )
    print(header)
    corpus = {"total": 0, "spine": 0, "atom_e": 0, "atom_h": 0, "piece_e": 0,
              "piece_h": 0, "spine_h": 0, "spine_c": 0}
    per_pattern = {}
    for p in PATTERNS:
        d = json.load(open(f"{DUMP_DIR}/{p}.outcome.json"))
        ent = d["rule_entry_counts"]
        hit = d["rule_memo_hit_counts"]
        com = d["rule_committed_counts"]
        spine_e = sum(ent.get(r, 0) for r in rules)
        spine_h = sum(hit.get(r, 0) for r in rules)
        spine_c = sum(com.get(r, 0) for r in rules)
        atom_e, atom_h = ent.get("atom", 0), hit.get("atom", 0)
        piece_e, piece_h = ent.get("piece", 0), hit.get("piece", 0)
        other = spine_e - atom_e - piece_e
        tot = d["total_entries"]
        per_pattern[p] = {"total": tot, "spine": spine_e, "atom": atom_e,
                          "piece": piece_e, "spine_h": spine_h, "spine_c": spine_c}
        print(
            f"{p:16s} {tot:8d} {spine_e:8d} {100.0*spine_e/tot:6.1f}%"
            f" {atom_e:7d} {atom_h:7d} {piece_e:8d} {piece_h:8d} {other:9d}"
        )
        corpus["total"] += tot
        corpus["spine"] += spine_e
        corpus["spine_h"] += spine_h
        corpus["spine_c"] += spine_c
        corpus["atom_e"] += atom_e
        corpus["atom_h"] += atom_h
        corpus["piece_e"] += piece_e
        corpus["piece_h"] += piece_h
    print("-" * len(header))
    print(
        f"{'CORPUS':16s} {corpus['total']:8d} {corpus['spine']:8d}"
        f" {100.0*corpus['spine']/corpus['total']:6.1f}% {corpus['atom_e']:7d}"
        f" {corpus['atom_h']:7d} {corpus['piece_e']:8d} {corpus['piece_h']:8d}"
        f" {corpus['spine']-corpus['atom_e']-corpus['piece_e']:9d}"
    )
    print()
    print(
        f"corpus spine committed={corpus['spine_c']} memo_hits={corpus['spine_h']}"
        f" (hit rate {100.0*corpus['spine_h']/corpus['spine']:.1f}% of spine entries;"
        f" body executions = {corpus['spine']-corpus['spine_h']})"
    )
    print()
    print("== per-carrier corpus totals (entries/committed/hits; zero-entry carriers named) ==")
    agg = {}
    for p in PATTERNS:
        d = json.load(open(f"{DUMP_DIR}/{p}.outcome.json"))
        for r in rules:
            a = agg.setdefault(r, [0, 0, 0])
            a[0] += d["rule_entry_counts"].get(r, 0)
            a[1] += d["rule_committed_counts"].get(r, 0)
            a[2] += d["rule_memo_hit_counts"].get(r, 0)
    live = {r: v for r, v in agg.items() if v[0] > 0}
    dead = sorted(r for r, v in agg.items() if v[0] == 0)
    for r, v in sorted(live.items(), key=lambda kv: -kv[1][0]):
        print(f"  {r:38s} e={v[0]:5d} c={v[1]:5d} h={v[2]:5d}")
    print(f"  zero-entry carriers on this corpus ({len(dead)}): {', '.join(dead)}")

    # machine-readable extract for the pricing fit
    out = {
        "artifact_sha8": digest,
        "carriers": rules,
        "per_pattern": per_pattern,
        "corpus": corpus,
    }
    with open("docs/tasks/artifacts/spine_step0/census_spine.json", "w") as f:
        json.dump(out, f, indent=1, sort_keys=True)
    print()
    print("machine-readable extract: docs/tasks/artifacts/spine_step0/census_spine.json")
    return 0


if __name__ == "__main__":
    sys.exit(main())
