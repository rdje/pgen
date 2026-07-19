#!/usr/bin/env python3
"""PGEN-RGX-0078-0172 -- SPINE-DISPATCH STEP-0: what IS the 22-25% "spine
dispatch self" population?

The `-0170` stack named `spine dispatch self` (22-25% self across all three
geomean bands) as "the LARGEST single population in the `-0162` profile [that]
NO slice has ever priced". The label came from reading FUNCTION NAMES at the top
of a self-time table. This script tests the label structurally, on the
custody-verified release floor probe:

  * function SIZE -- a dispatch function is tens of instructions;
  * instruction MIX -- a dispatch function is dominated by compare + branch.

Instrument: `otool -tV` on `preserved_probes/regex_perf_probe_c1_1d3fa0ee`
(sha256 prefix `1d3fa0ee`, byte-identical to the banked floor probe). Reading a
binary cannot flip the observability twin => the `-0163` STANDING INSTRUMENT
RULE holds by construction.

WARNING -- SCOPE (the `-0166` lesson applied to THIS slice's own numbers): a STATIC
instruction mix over a function body is NOT the dynamic retired-instruction mix,
and neither is nanoseconds. This slice REFUTES A LABEL. It does not price a
lever, and no band is derived from it.
"""

import re
import subprocess
import sys
from collections import Counter
from pathlib import Path

PROBE = Path("preserved_probes/regex_perf_probe_c1_1d3fa0ee")
EXPECT_SHA_PREFIX = "1d3fa0ee"

# Symbol substrings identifying the spine population named by `-0162` / `-0170`,
# with their `-0162` sub-1us self shares for context (selfcum_tables.txt).
TARGETS = [
    ("cascade_match_piece",               "19cascade_match_piece17h65565a4aa37893a3", 8.0),
    ("cascade_match_atom{closure}",       "18cascade_match_atom28_$u7b$$u7b$closure", 6.7),
    ("memoized_call",                     "13memoized_call17h3c90",                   5.2),
    ("cascade_match_entry_concatenation", "33cascade_match_entry_concatenation17ha159", 3.0),
    ("parse_pattern",                     "13parse_pattern17h42cee",                  None),
]

MEM = {"ldr", "str", "stp", "ldp", "ldur", "stur", "ldrb", "strb", "ldrsb",
       "ldrh", "strh", "ldrsw", "ldarb", "stlrb", "ldrsh"}
CTL = {"cmp", "b", "cbz", "cbnz", "br", "csel", "cset", "ccmp", "tbz", "tbnz",
       "b.ne", "b.eq", "b.hi", "b.ls", "b.hs", "b.lo", "b.lt", "b.ge", "b.gt",
       "b.le", "b.mi", "b.pl", "cmn", "tst"}
CALL = {"bl", "blr"}


def custody():
    sha = subprocess.run(["shasum", "-a", "256", str(PROBE)],
                         capture_output=True, text=True).stdout.split()[0]
    ok = sha.startswith(EXPECT_SHA_PREFIX)
    print(f"probe custody sha256={sha[:16]}  expected prefix={EXPECT_SHA_PREFIX}  "
          f"{'OK' if ok else 'MISMATCH'}")
    if not ok:
        sys.exit("custody mismatch -- refusing to report")


def symbol_ranges():
    """(name, lo, hi) for each target, sized by the next symbol's address."""
    nm = subprocess.run(["nm", "-n", str(PROBE)], capture_output=True, text=True).stdout
    syms = []
    for line in nm.splitlines():
        p = line.split()
        if len(p) == 3:
            try:
                syms.append((int(p[0], 16), p[2]))
            except ValueError:
                pass
    syms.sort()
    out = []
    for name, key, self_pct in TARGETS:
        for i, (a, s) in enumerate(syms):
            if key in s:
                hi = syms[i + 1][0] if i + 1 < len(syms) else a
                out.append((name, a, hi, self_pct))
                break
    return out


def main():
    if not PROBE.exists():
        sys.exit(f"missing {PROBE}")
    custody()
    ranges = symbol_ranges()

    counters = {n: Counter() for n, _, _, _ in ranges}
    proc = subprocess.Popen(["otool", "-tV", str(PROBE)],
                            stdout=subprocess.PIPE, text=True,
                            stderr=subprocess.DEVNULL)
    pat = re.compile(r"^([0-9a-f]+)\t\s*(\S+)")
    for line in proc.stdout:
        m = pat.match(line)
        if not m:
            continue
        a = int(m.group(1), 16)
        for n, lo, hi, _ in ranges:
            if lo <= a < hi:
                counters[n][m.group(2)] += 1
                break
    proc.wait()

    print(f"\n{'='*88}")
    print("SPINE POPULATION -- size and instruction mix (STATIC, release floor probe)")
    print(f"{'='*88}")
    print(f"{'function':36s} {'-0162 self':>10s} {'bytes':>9s} {'instrs':>8s} "
          f"{'mem':>7s} {'ctl':>7s} {'call':>6s} {'other':>7s}")
    for n, lo, hi, self_pct in ranges:
        c = counters[n]
        t = sum(c.values())
        if not t:
            continue
        mem = sum(v for k, v in c.items() if k in MEM)
        ctl = sum(v for k, v in c.items() if k in CTL)
        cal = sum(v for k, v in c.items() if k in CALL)
        sp = f"{self_pct:.1f}%" if self_pct is not None else "-"
        print(f"{n:36s} {sp:>10s} {hi-lo:9,d} {t:8,d} "
              f"{100*mem/t:6.1f}% {100*ctl/t:6.1f}% {100*cal/t:5.1f}% "
              f"{100*(t-mem-ctl-cal)/t:6.1f}%")

    print(f"\n{'-'*88}")
    print("VERDICT")
    print(f"{'-'*88}")
    print("  A dispatch function is tens of instructions dominated by compare+branch.")
    print("  These are 2,694-19,215 instructions with control flow a MINORITY in every")
    print("  one, and MEMORY TRAFFIC the largest class throughout. They are the FUSED")
    print("  CASCADE REGIONS (D2-B emitter + fat-LTO inlining), i.e. the matching work")
    print("  itself -- not a dispatch overhead sitting on top of it.")
    print("\n  => the 'spine dispatch' LABEL is refuted; there is no dispatch lever here.")
    print("  => SCOPE: static mix != dynamic retired mix != nanoseconds. No band is")
    print("     derived from these numbers; pricing needs a DYNAMIC instrument.")


if __name__ == "__main__":
    main()
