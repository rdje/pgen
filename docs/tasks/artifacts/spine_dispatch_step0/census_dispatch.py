#!/usr/bin/env python3
"""RGX-0078.5.i.16 FIRST-BYTE-INDEXED SPINE DISPATCH STEP-0 — the census (PGEN-RGX-0078-0121).

STATIC half: scan the canonical regex artifact (hash-pinned) for every inline tournament
instance inside the `cascade_match_*` fns (the bare-graph fast path). For each tournament
(keyed by its `RollbackLabel::C3bBranchCleanup { rule, branch, total }` labels) extract the
per-branch first-byte guard structure:
  - guarded branch: `if parse_start < input.len() && matches!(bytes[parse_start], SET)`
    (optionally a FIRST2 second-byte conjunct `&& matches!(bytes[parse_start+1], SET2)`)
  - unguarded branch: no byte test — always attempted.
Any guard shape the parser does not recognize is flagged LOUDLY (never silently skipped).
From the byte-1 sets, compute the per-byte survivor table surv(b) = unguarded + #branches
whose byte-1 set contains b (a byte-2 conjunct stays an in-branch test under indexed
dispatch, so it does not reduce byte-1 survivors).

DYNAMIC half (PROTOCOL-graph observability; the CASCADE-PLAN-B boundary join maps the
counts onto the bare-graph populations — identical parse decisions by construction, the
standing `.5.i.9`/`.5.i.13` caveat):
  - the 8 fresh `--dump-rule-entry-counts-json` / `--dump-rule-outcome-counts-json` dumps
    (cmp-proven byte-identical to the `-0116` banked dumps),
  - the 8 debug traces: `Entering branch k/N for rule 'r' at position p` lines fire INSIDE
    `try_parse`, i.e. only for guard-PASSING attempts => the per-rule SURVIVOR population;
    `Memo miss for rule <id> at position <p>` lines = protocol body executions (the
    population that pays the full sequential guard chain today).
CROSS-VALIDATION: for every traced attempt position, the static surv(input_byte[p]) is
compared against the observed per-(rule,position) attempt count.

Run from the repo root:
    python3 docs/tasks/artifacts/spine_dispatch_step0/census_dispatch.py \
        > docs/tasks/artifacts/spine_dispatch_step0/census_dispatch.txt

Inputs (must exist):
    generated/regex_parser.rs                                        (canonical artifact)
    docs/tasks/artifacts/spec_step0/<p>.rx                           (the 8 bench inputs)
    rust/target/generated_logs/spine_dispatch_step0/<p>.entry.json   (fresh dumps)
    rust/target/generated_logs/spine_dispatch_step0/<p>.outcome.json (fresh dumps)
    rust/target/generated_logs/spine_dispatch_step0/<p>.trace.txt    (fresh traces)
"""

import hashlib
import json
import re
import sys
from collections import defaultdict

ARTIFACT = "generated/regex_parser.rs"
RX_DIR = "docs/tasks/artifacts/spec_step0"
DUMP_DIR = "rust/target/generated_logs/spine_dispatch_step0"
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

FN_RE = re.compile(r"^    fn (\w+)\(")
LABEL_RE = re.compile(
    r'C3bBranchCleanup \{\s*$'
)
LABEL_RULE_RE = re.compile(r'rule: "([^"]+)"')
LABEL_BRANCH_RE = re.compile(r"branch: (\d+)usize")
LABEL_TOTAL_RE = re.compile(r"total: (\d+)usize")
GUARD_HEAD_RE = re.compile(r"if parse_start(?: \+ 1)? < parser\.input\.len\(\)")
MATCH_IDX_RE = re.compile(r"matches!\(\s*parser\.input\.as_bytes\(\) \[parse_start(?P<plus> \+ 1)?\],")
SKIP_CHECK = 'if "longest_match" == "ordered" && __pgen_best_found'
BYTE_RE = re.compile(r"(\d+)u8")


def load_artifact():
    data = open(ARTIFACT, "rb").read()
    digest = hashlib.sha256(data).hexdigest()[:8]
    return digest, data.decode("utf-8").split("\n")


def rule_id_map(lines):
    m = {}
    for ln in lines:
        mo = re.match(r"    const RULE_(\w+): RuleId = (\d+)u16;", ln)
        if mo:
            m[int(mo.group(2))] = mo.group(1).lower()
    return m


def static_census(lines):
    """Extract every tournament instance in every cascade_match_* fn.

    Returns: instances = list of dicts {fn, rule, total, branches:[{branch, guard}]}
    where guard = None (unguarded) or {"b1": set, "b2": set|None, "raw_ok": bool}.
    Also returns loud_problems: list of strings for any unparsed shape.
    """
    problems = []
    cur_fn = None
    # per-fn scan state
    pending_guard = None  # accumulating guard text
    guard_text = ""
    open_guard_depth = 0
    last_guard = None  # most recent completed guard awaiting a skip-check
    pending_branch = None  # guard association awaiting its rollback label
    instances = []  # completed tournaments keyed later
    open_branches = []  # (fn, guard) association queue in occurrence order
    branch_records = []  # (fn, seq, rule, branch, total, guard)
    seq_in_fn = 0
    collecting_label = False
    label_text = ""
    guard_queue = []  # guards (or None) in skip-check occurrence order per fn

    for i, ln in enumerate(lines):
        mfn = FN_RE.match(ln)
        if mfn:
            cur_fn = mfn.group(1)
            seq_in_fn = 0
            pending_guard = None
            guard_text = ""
            last_guard = None
            guard_queue = []
            collecting_label = False
            continue
        if cur_fn is None or not cur_fn.startswith("cascade_match_"):
            continue
        s = ln.strip()

        if collecting_label:
            label_text += " " + s
            if "}" in s:
                collecting_label = False
                r = LABEL_RULE_RE.search(label_text)
                b = LABEL_BRANCH_RE.search(label_text)
                t = LABEL_TOTAL_RE.search(label_text)
                if not (r and b and t):
                    problems.append(
                        f"{cur_fn} line {i+1}: unparsable rollback label: {label_text!r}"
                    )
                else:
                    g = guard_queue.pop(0) if guard_queue else "NO_SKIPCHECK_SEEN"
                    branch_records.append(
                        {
                            "fn": cur_fn,
                            "line": i + 1,
                            "rule": r.group(1),
                            "branch": int(b.group(1)),
                            "total": int(t.group(1)),
                            "guard": g,
                        }
                    )
            continue

        if pending_guard is not None:
            guard_text += " " + s
            if s.endswith("{"):
                # guard head closed: parse byte sets
                sets = {"b1": None, "b2": None}
                ok = True
                parts = list(MATCH_IDX_RE.finditer(guard_text))
                if not parts:
                    ok = False
                for k, mo in enumerate(parts):
                    endpos = (
                        parts[k + 1].start() if k + 1 < len(parts) else len(guard_text)
                    )
                    seg = guard_text[mo.end():endpos]
                    byteset = frozenset(int(x) for x in BYTE_RE.findall(seg))
                    key = "b2" if mo.group("plus") else "b1"
                    if sets[key] is not None:
                        ok = False
                    sets[key] = byteset
                if "!matches!" in guard_text:
                    ok = False
                if not ok or sets["b1"] is None:
                    problems.append(
                        f"{cur_fn} line {i+1}: unparsed guard shape: {guard_text[:200]!r}"
                    )
                    last_guard = "UNPARSED"
                else:
                    last_guard = sets
                pending_guard = None
                guard_text = ""
            continue

        if GUARD_HEAD_RE.search(s) and "matches!" in "".join(
            lines[i : i + 2]
        ):
            # begin accumulating a guard (may span many lines)
            if s.endswith("{"):
                # single-line guard (rare) — handle inline
                pending_guard = None
                guard_text = s
                sets = {"b1": None, "b2": None}
                parts = list(MATCH_IDX_RE.finditer(guard_text))
                okk = bool(parts)
                for k, mo in enumerate(parts):
                    endpos = (
                        parts[k + 1].start() if k + 1 < len(parts) else len(guard_text)
                    )
                    seg = guard_text[mo.end():endpos]
                    byteset = frozenset(int(x) for x in BYTE_RE.findall(seg))
                    key = "b2" if mo.group("plus") else "b1"
                    sets[key] = byteset
                last_guard = sets if okk and sets["b1"] is not None else "UNPARSED"
                if last_guard == "UNPARSED":
                    problems.append(f"{cur_fn} line {i+1}: unparsed 1-line guard")
                guard_text = ""
            else:
                pending_guard = True
                guard_text = s
            continue

        if s.startswith(SKIP_CHECK):
            guard_queue.append(last_guard)  # None = unguarded
            last_guard = None
            continue

        if LABEL_RE.search(s):
            collecting_label = True
            label_text = ""
            continue

    # group branch records into tournament instances: consecutive records in the same
    # fn with the same (rule, total) and ascending branch numbers form one instance
    grouped = []
    curi = None
    for rec in branch_records:
        if (
            curi is not None
            and rec["fn"] == curi["fn"]
            and rec["rule"] == curi["rule"]
            and rec["total"] == curi["total"]
            and rec["branch"] == curi["branches"][-1]["branch"] + 1
        ):
            curi["branches"].append(rec)
        else:
            if curi is not None:
                grouped.append(curi)
            curi = {
                "fn": rec["fn"],
                "rule": rec["rule"],
                "total": rec["total"],
                "branches": [rec],
            }
    if curi is not None:
        grouped.append(curi)

    for inst in grouped:
        if len(inst["branches"]) != inst["total"]:
            problems.append(
                f"{inst['fn']} rule={inst['rule']}: extracted "
                f"{len(inst['branches'])} branches but label total={inst['total']}"
            )
    return grouped, problems


def survivors_table(inst):
    """Per-byte survivor count for a tournament instance (byte-1 discrimination only)."""
    surv = [0] * 256
    unguarded = 0
    for br in inst["branches"]:
        g = br["guard"]
        if g is None or g in ("UNPARSED", "NO_SKIPCHECK_SEEN"):
            unguarded += 1
        else:
            for b in g["b1"]:
                surv[b] += 1
    return [s + unguarded for s in surv], unguarded


def main():
    digest, lines = load_artifact()
    idmap = rule_id_map(lines)
    instances, problems = static_census(lines)

    print("FIRST-BYTE-INDEXED SPINE DISPATCH CENSUS — RGX-0078.5.i.16 STEP-0"
          " (PGEN-RGX-0078-0121)")
    print(f"artifact: {ARTIFACT} sha256[:8]={digest}")
    print()
    if problems:
        print("⛔ LOUD PROBLEMS (unparsed shapes — census incomplete until resolved):")
        for p in problems:
            print(f"  {p}")
        print()

    # aggregate per rule: a rule's tournament may be emitted inline in several parents
    by_rule = defaultdict(list)
    for inst in instances:
        by_rule[inst["rule"]].append(inst)

    print("== STATIC: cascade tournament-site inventory (bare-graph fast path) ==")
    print(f"tournament instances total={len(instances)} across "
          f"{len(set(i['fn'] for i in instances))} cascade fns; distinct rules="
          f"{len(by_rule)}")
    print()
    print("per-rule structure (instances / branches K / byte-guarded G / unguarded U /"
          " first2-guarded / mean+max byte-1 survivors over the 191 seen bench bytes):")

    # collect the byte alphabet actually seen across bench inputs for a focused survivor stat
    bench_bytes = []
    inputs = {}
    for p in PATTERNS:
        raw = open(f"{RX_DIR}/{p}.rx", "rb").read().rstrip(b"\n")
        inputs[p] = raw
        bench_bytes.extend(raw)

    rule_static = {}
    for rule, insts in sorted(by_rule.items()):
        i0 = insts[0]
        # structural identity across instances of the same rule
        sig0 = [
            (br["guard"] if not isinstance(br["guard"], dict) else
             (sorted(br["guard"]["b1"]), sorted(br["guard"]["b2"]) if br["guard"]["b2"] else None))
            for br in i0["branches"]
        ]
        identical = all(
            [
                (br["guard"] if not isinstance(br["guard"], dict) else
                 (sorted(br["guard"]["b1"]), sorted(br["guard"]["b2"]) if br["guard"]["b2"] else None))
                for br in inst["branches"]
            ]
            == sig0
            for inst in insts
        )
        surv, unguarded = survivors_table(i0)
        first2 = sum(
            1 for br in i0["branches"]
            if isinstance(br["guard"], dict) and br["guard"]["b2"]
        )
        guarded = sum(1 for br in i0["branches"] if isinstance(br["guard"], dict))
        seen_surv = [surv[b] for b in bench_bytes]
        # distinct byte->branch-mask patterns (sizes a two-level class-compressed LUT)
        masks = set()
        for b in range(256):
            mask = tuple(
                bi
                for bi, br in enumerate(i0["branches"])
                if not isinstance(br["guard"], dict) or b in br["guard"]["b1"]
            )
            masks.add(mask)
        rule_static[rule] = {
            "instances": len(insts),
            "K": i0["total"],
            "G": guarded,
            "U": unguarded,
            "first2": first2,
            "surv_table": surv,
            "identical_across_instances": identical,
            "distinct_masks": len(masks),
        }
        print(
            f"  {rule:32s} inst={len(insts)} K={i0['total']:3d} G={guarded:3d}"
            f" U={unguarded} f2={first2}"
            f" surv(bench-bytes) mean={sum(seen_surv)/len(seen_surv):5.2f}"
            f" max={max(seen_surv)}"
            f" masks={len(masks)}"
            f"{'' if identical else '  ⚠️ INSTANCES DIFFER'}"
        )

    # ---------- DYNAMIC ----------
    print()
    print("== DYNAMIC: 8-pattern populations (PROTOCOL-graph counts; boundary-join"
          " onto the cascade per the standing caveat) ==")
    trace_re = re.compile(
        r"Entering branch (\d+)/(\d+) for rule '([^']+)' at position (\d+)"
    )
    miss_re = re.compile(r"Memo miss for rule (\d+) at position (\d+)")

    per_pattern = {}
    xval_bad = 0
    xval_tot = 0
    for p in PATTERNS:
        entry_dump = json.load(open(f"{DUMP_DIR}/{p}.entry.json"))
        entries = entry_dump.get("rule_entry_counts", entry_dump)
        dump_total_entries = entry_dump.get("total_entries")
        attempts = defaultdict(int)  # rule -> survivor attempts
        attempts_pos = defaultdict(int)  # (rule,pos) -> attempts in ONE tournament exec? (aggregated)
        execs = defaultdict(int)  # rule -> protocol body executions (memo misses)
        execs_pos = defaultdict(int)  # (rule,pos) -> executions
        totals_seen = {}
        for ln in open(f"{DUMP_DIR}/{p}.trace.txt"):
            mo = trace_re.search(ln)
            if mo:
                br, tot, rule, pos = (
                    int(mo.group(1)),
                    int(mo.group(2)),
                    mo.group(3),
                    int(mo.group(4)),
                )
                attempts[rule] += 1
                attempts_pos[(rule, pos)] += 1
                totals_seen[rule] = tot
                continue
            mo = miss_re.search(ln)
            if mo:
                rid, pos = int(mo.group(1)), int(mo.group(2))
                rname = idmap.get(rid, f"id{rid}")
                execs[rname] += 1
                execs_pos[(rname, pos)] += 1
        # zero-survivor executions: tournament execs at (rule,pos) cells with NO
        # guard-passing attempt — the population an indexed mask==0 fast-fail would
        # spare the whole tournament preamble (checkpoint + OrWinner push + epilogue)
        zero_surv_execs = sum(
            n
            for (rule, pos), n in execs_pos.items()
            if rule in rule_static and attempts_pos.get((rule, pos), 0) == 0
        )
        per_pattern[p] = {
            "attempts": dict(attempts),
            "execs": dict(execs),
            "entries": entries,
            "total_entries": dump_total_entries
            if dump_total_entries is not None
            else sum(entries.values()),
            "totals_seen": totals_seen,
            "zero_surv_execs": zero_surv_execs,
        }

        # cross-validate survivors: attempts at (rule,pos) vs static surv(byte at pos)
        raw = inputs[p]
        for (rule, pos), n in attempts_pos.items():
            if rule not in rule_static:
                continue
            st = rule_static[rule]
            if pos < len(raw):
                pred = st["surv_table"][raw[pos]]
            else:
                pred = st["U"]
            # n counts attempts across possibly MULTIPLE executions at this position;
            # the per-execution attempt count must be an integer multiple of pred
            # (or ≤ pred when tournament execution count at pos is 1)
            xval_tot += 1
            if pred == 0 and n > 0:
                xval_bad += 1
                print(f"  ⚠️ XVAL {p} {rule}@{pos}: static surv=0 but {n} attempts")
            elif pred > 0 and n % pred != 0 and n > pred:
                xval_bad += 1
                print(
                    f"  ⚠️ XVAL {p} {rule}@{pos}: attempts={n} not a multiple of"
                    f" static surv={pred}"
                )
    print(f"static↔dynamic survivor cross-validation: {xval_tot} (rule,pos) cells"
          f" checked, {xval_bad} inconsistent")
    print()

    # ---------- the priced surface ----------
    print("== THE PRICED SURFACE: per-pattern guard-test load (today) vs"
          " indexed-dispatch load ==")
    print("today per exec: G sequential byte-set tests (+U unguarded bodies always"
          " attempted); indexed: 1 LUT lookup + survivors-only iteration")
    hdr = (f"{'pattern':16s} {'execs*':>7s} {'tests_today':>12s} {'attempts':>9s}"
           f" {'lut_lookups':>12s} {'tests_elided':>13s} {'ap_elided':>10s}"
           f" {'zero_surv_e':>12s}")
    print(hdr)
    print("  (*execs = protocol memo-miss body executions of rules that have a cascade"
          " tournament)")
    census_json = {"artifact": digest, "rules": {}, "patterns": {}}
    for rule, st in rule_static.items():
        census_json["rules"][rule] = {
            "instances": st["instances"],
            "K": st["K"],
            "G": st["G"],
            "U": st["U"],
            "first2": st["first2"],
            "identical_across_instances": st["identical_across_instances"],
        }
    corpus = defaultdict(float)
    for p in PATTERNS:
        d = per_pattern[p]
        execs_t = 0
        tests_today = 0
        attempts_t = 0
        ap_elided = 0  # the atom+piece share of elided tests (the visible-share-bounded part)
        for rule, st in rule_static.items():
            e = d["execs"].get(rule, 0)
            a = d["attempts"].get(rule, 0)
            execs_t += e
            tests_today += e * st["G"]
            attempts_t += a
            if rule in ("atom", "piece") and e:
                ap_elided += e * st["G"] - e
        lut = execs_t
        elided = tests_today - lut
        total_entries = d["total_entries"]
        census_json["patterns"][p] = {
            "execs": execs_t,
            "tests_today": tests_today,
            "attempts": attempts_t,
            "lut_lookups": lut,
            "tests_elided": elided,
            "ap_elided": ap_elided,
            "zero_surv_execs": d["zero_surv_execs"],
            "total_entries": total_entries,
            "per_rule": {
                rule: {
                    "execs": d["execs"].get(rule, 0),
                    "attempts": d["attempts"].get(rule, 0),
                }
                for rule in rule_static
                if d["execs"].get(rule, 0) or d["attempts"].get(rule, 0)
            },
        }
        corpus["execs"] += execs_t
        corpus["tests"] += tests_today
        corpus["attempts"] += attempts_t
        corpus["elided"] += elided
        corpus["ap_elided"] += ap_elided
        corpus["zero"] += d["zero_surv_execs"]
        print(f"  {p:16s} {execs_t:7d} {tests_today:12d} {attempts_t:9d}"
              f" {lut:12d} {elided:13d} {ap_elided:10d} {d['zero_surv_execs']:12d}")
    print(f"  {'CORPUS':16s} {int(corpus['execs']):7d} {int(corpus['tests']):12d}"
          f" {int(corpus['attempts']):9d} {int(corpus['execs']):12d}"
          f" {int(corpus['elided']):13d} {int(corpus['ap_elided']):10d}"
          f" {int(corpus['zero']):12d}")

    with open(
        "docs/tasks/artifacts/spine_dispatch_step0/census_dispatch.json", "w"
    ) as f:
        json.dump(census_json, f, indent=1, sort_keys=True)
    print()
    print("census_dispatch.json written.")
    if problems:
        print("⛔ REMINDER: loud problems above — resolve before pricing.")
        sys.exit(1)


if __name__ == "__main__":
    main()
