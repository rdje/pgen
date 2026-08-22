#!/usr/bin/env python3
"""Arm evaluators for probe.sh (SV-CORPUS-GRAD.13c.2x.9). Kept beside the script rather than
inlined so each arm is independently runnable and reviewable."""
import collections
import json
import re
import sys

TARGET = "known_unscoped_property_identifier"


def outcome(path):
    """ARM 0 — `entries/committed` for the target rule, from --dump-rule-outcome-counts-json."""
    try:
        d = json.load(open(path))
    except Exception:
        return "-/-"
    return "%s/%s" % (d.get("rule_entry_counts", {}).get(TARGET, 0),
                      d.get("rule_committed_counts", {}).get(TARGET, 0))


def route(path):
    """ARM 1 — which alternative of `ps_or_hierarchical_property_identifier` the WINNING tree kept.

    `prop_primary_*`'s property_instance branch is `{kind:"instance", body:{name,args}}` with NO
    `abbrev`; `seq_unary`'s sequence_instance branch adds `abbrev`, which is what separates them.
    """
    try:
        d = json.load(open(path))
    except Exception:
        return "none"
    routes = []

    def walk(o):
        if isinstance(o, dict):
            if o.get("kind") == "instance" and set(o.keys()) == {"kind", "body"}:
                n = (o.get("body") or {}).get("name")
                if isinstance(n, dict) and isinstance(n.get("body"), dict):
                    k = set(n["body"].keys())
                    if k == {"body"}:
                        routes.append("known_unscoped")
                    elif "root" in k:
                        routes.append("hierarchical")
                    elif k == {"scope", "name"}:
                        routes.append("scoped")
                    else:
                        routes.append("unrecognized")
            for v in o.values():
                walk(v)
        elif isinstance(o, list):
            for v in o:
                walk(v)

    walk(d)
    return "+".join(sorted(set(routes))) if routes else "none"


def tournament(path):
    """ARM 3 — who WON `prop_primary_sv_2017`'s 27-branch longest-match tournament, and by how much.

    `sel=` is codegen's own selection line, which is what TOOLBOX Protocol D step 2 says to read:
    `🏁 Rule 'R' selected branch N/M consuming K chars (priority=…, branch_policy=longest_match)`.
    `b1`/`b26` are the furthest successful extents of branch 1 (`sequence_expr`) and branch 26
    (`property_instance`), which is what makes an equal-extent TIE visible as a tie.
    """
    best = collections.defaultdict(int)
    seen = set()
    sel = []
    try:
        fh = open(path, errors="replace")
    except Exception:
        return "sel=- b1=- b26=-"
    with fh:
        for line in fh:
            m = re.search(r"Leaving branch (\d+)/27 for rule 'prop_primary_sv_2017' "
                          r"at position (\d+) \(success\)", line)
            if m:
                br, pos = int(m.group(1)), int(m.group(2))
                seen.add(br)
                best[br] = max(best[br], pos)
                continue
            m = re.search(r"selected branch (\d+)/27 consuming (\d+) chars", line)
            if m:
                sel.append("%s@%s" % (m.group(1), m.group(2)))
    return "sel=%s b1=%s b26=%s" % (",".join(sorted(set(sel))) or "-",
                                    best[1] if 1 in seen else "-",
                                    best[26] if 26 in seen else "-")


if __name__ == "__main__":
    print({"outcome": outcome, "route": route, "tournament": tournament}[sys.argv[1]](sys.argv[2]))
