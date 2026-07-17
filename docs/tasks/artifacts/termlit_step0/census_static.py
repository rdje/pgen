#!/usr/bin/env python3
"""RGX-0078.5.i.11 STEP-0 — static terminal-literal call-site census.

Classifies every emitted `match_string("…")` call site in every generated
parser artifact by literal byte length, ASCII purity, and enclosing-fn
context. Run from the repo root:

    python3 docs/tasks/artifacts/termlit_step0/census_static.py

The regex artifact hash is printed so the census is pinned to its vintage
(canonical D3 artifact = d2decb51).
"""

import glob
import hashlib
import re
from collections import Counter

FN_RE = re.compile(r"^\s*(?:pub )?fn (\w+)")
CALL_RE = re.compile(r'match_string\("((?:[^"\\]|\\.)*)"\)')


def unescape_rust(s: str) -> str:
    out = []
    i = 0
    while i < len(s):
        c = s[i]
        if c == "\\" and i + 1 < len(s):
            n = s[i + 1]
            if n == "u" and i + 2 < len(s) and s[i + 2] == "{":
                j = s.index("}", i)
                out.append(chr(int(s[i + 3 : j], 16)))
                i = j + 1
                continue
            if n == "x":
                out.append(chr(int(s[i + 2 : i + 4], 16)))
                i += 4
                continue
            mp = {"n": "\n", "t": "\t", "r": "\r", "\\": "\\", '"': '"', "0": "\0", "'": "'"}
            out.append(mp.get(n, n))
            i += 2
            continue
        out.append(c)
        i += 1
    return "".join(out)


def context_bucket(fn_name):
    if fn_name is None:
        return "<module>"
    if fn_name.startswith("cascade_match"):
        return "cascade_match"
    if fn_name.startswith("cascade_build"):
        return "cascade_build"
    if fn_name.startswith("scan_"):
        return "scan"
    if fn_name.startswith("parse_"):
        return "protocol_body"
    return "helper"


def census(path):
    src = open(path, encoding="utf-8").read()
    cur_fn = None
    sites = []
    for line in src.split("\n"):
        m = FN_RE.match(line)
        if m:
            cur_fn = m.group(1)
        for mm in CALL_RE.finditer(line):
            lit = unescape_rust(mm.group(1))
            blen = len(lit.encode("utf-8"))
            sites.append(
                (
                    context_bucket(cur_fn),
                    blen,
                    all(ord(c) < 128 for c in lit),
                )
            )
    return sites


def len_bucket(blen):
    if blen == 1:
        return "1B"
    if blen <= 4:
        return "2-4B"
    return ">=5B"


def main():
    print("== RGX-0078.5.i.11 static terminal-literal call-site census ==")
    total_row = Counter()
    for path in sorted(glob.glob("generated/*.rs")):
        name = path.split("/")[-1]
        sha8 = hashlib.sha256(open(path, "rb").read()).hexdigest()[:8]
        sites = census(path)
        if not sites:
            print(f"{name:44s} [{sha8}] sites=0")
            continue
        by_len = Counter(len_bucket(b) for _, b, _ in sites)
        by_ctx = Counter(c for c, _, _ in sites)
        non_ascii = sum(1 for _, _, a in sites if not a)
        short_ascii = sum(1 for _, b, a in sites if a and b <= 8)
        n = len(sites)
        total_row.update(
            {"sites": n, "1B": by_len["1B"], "2-4B": by_len["2-4B"], ">=5B": by_len[">=5B"],
             "non_ascii": non_ascii, "short_ascii_le8": short_ascii}
        )
        print(
            f"{name:44s} [{sha8}] sites={n:5d}  "
            f"1B={by_len['1B']:5d} ({100.0*by_len['1B']/n:4.1f}%)  "
            f"2-4B={by_len['2-4B']:4d}  >=5B={by_len['>=5B']:4d}  "
            f"non_ascii={non_ascii}  ascii_le8={short_ascii} ({100.0*short_ascii/n:5.1f}%)"
        )
        print(f"{'':56s} contexts: {dict(sorted(by_ctx.items()))}")
    n = total_row["sites"]
    print(
        f"{'TOTAL (all artifacts)':44s} {'':10s} sites={n:5d}  "
        f"1B={total_row['1B']:5d} ({100.0*total_row['1B']/n:4.1f}%)  "
        f"2-4B={total_row['2-4B']:4d}  >=5B={total_row['>=5B']:4d}  "
        f"non_ascii={total_row['non_ascii']}  ascii_le8={total_row['short_ascii_le8']}"
        f" ({100.0*total_row['short_ascii_le8']/n:5.1f}%)"
    )


if __name__ == "__main__":
    main()
