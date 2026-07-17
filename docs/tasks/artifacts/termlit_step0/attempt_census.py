#!/usr/bin/env python3
"""RGX-0078.5.i.11 STEP-0 — dynamic per-pattern terminal-literal attempt census.

`match_string` emits exactly ONE `🔤 Attempting to match terminal '<lit>'`
debug-trace line per attempt (generated runtime, before the compare), so a
`PGEN_TRACE_VERBOSITY=debug` parse of each bench pattern IS the attempt
counter — no instrument build needed. Uses the DEBUG `parseability_probe`
(D3 vintage) and the `spec_step0/*.rx` bench-pattern files (byte-identical
to the `regex_perf_probe` PATTERNS table).

GRAPH CAVEAT (recorded in the leaf): a traced parse routes the PROTOCOL
graph; the bench runs the BARE graph. Populations align because scan
helpers and fused cascades route constant literals through the SAME
`match_string` primitive; residual deltas (protocol memo-hits vs bare
re-scans) are bounded via the profile cross-check in `pricing_model.py`.

Run from the repo root:

    python3 docs/tasks/artifacts/termlit_step0/attempt_census.py

Raw trace logs land under rust/target/generated_logs/termlit_step0/ (on-disk
scratch); the census summary is written next to this script.
"""

import collections
import json
import os
import re
import subprocess

PROBE = "rust/target/debug/parseability_probe"
RX_DIR = "docs/tasks/artifacts/spec_step0"
LOG_DIR = "rust/target/generated_logs/termlit_step0"
OUT_JSON = "docs/tasks/artifacts/termlit_step0/attempt_census.json"
OUT_TXT = "docs/tasks/artifacts/termlit_step0/attempt_census.txt"

PATTERN_NAMES = [
    "literal_simple",
    "digit_sequence",
    "character_class",
    "alternation",
    "capture_groups",
    "url_simple",
    "email_basic",
    "anchor_complex",
]

LINE_RE = re.compile(r"Attempting to match terminal '(.*)' at position \d+ \(end: \d+\)")


def main():
    os.makedirs(LOG_DIR, exist_ok=True)
    out = {}
    lines = ["== RGX-0078.5.i.11 dynamic terminal-literal attempt census (traced protocol graph) =="]
    for name in PATTERN_NAMES:
        rx = os.path.join(RX_DIR, f"{name}.rx")
        env = dict(os.environ, PGEN_TRACE_VERBOSITY="debug")
        r = subprocess.run(
            [PROBE, "--parse", "regex", rx],
            env=env,
            capture_output=True,
            text=True,
        )
        blob = r.stdout + r.stderr
        open(os.path.join(LOG_DIR, f"{name}.trace.log"), "w").write(blob)
        ok = "parse_full passed" in blob
        lits = collections.Counter()
        for line in blob.splitlines():
            m = LINE_RE.search(line)
            if m:
                lits[m.group(1)] += 1
        by_len = collections.Counter()
        for lit, cnt in lits.items():
            blen = len(lit.encode("utf-8"))
            key = "1B" if blen == 1 else ("2-4B" if blen <= 4 else ">=5B")
            by_len[key] += cnt
        non_ascii = sum(cnt for lit, cnt in lits.items() if any(ord(c) >= 128 for c in lit))
        total = sum(lits.values())
        out[name] = {
            "accepted": ok,
            "attempts_total": total,
            "attempts_1B": by_len["1B"],
            "attempts_2_4B": by_len["2-4B"],
            "attempts_ge5B": by_len[">=5B"],
            "attempts_non_ascii": non_ascii,
            "top_literals": lits.most_common(10),
        }
        lines.append(
            f"{name:18s} accepted={ok} total={total:5d}  "
            f"1B={by_len['1B']:5d}  2-4B={by_len['2-4B']:4d}  >=5B={by_len['>=5B']:3d}  "
            f"non_ascii={non_ascii}  top={lits.most_common(5)}"
        )
    grand = sum(v["attempts_total"] for v in out.values())
    g1 = sum(v["attempts_1B"] for v in out.values())
    lines.append(
        f"{'TOTAL':18s} attempts={grand}  1B={g1} ({100.0*g1/grand:.1f}%)  "
        f"2-4B={sum(v['attempts_2_4B'] for v in out.values())}  "
        f">=5B={sum(v['attempts_ge5B'] for v in out.values())}  "
        f"non_ascii={sum(v['attempts_non_ascii'] for v in out.values())}"
    )
    json.dump(out, open(OUT_JSON, "w"), indent=1)
    open(OUT_TXT, "w").write("\n".join(lines) + "\n")
    print("\n".join(lines))


if __name__ == "__main__":
    main()
