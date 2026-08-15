#!/usr/bin/env python3
"""Find every ARM64 `BL` call site in __text whose target is a given vmaddr, and
name the enclosing symbol for each.

`ENGINE-UNIVERSAL-SERVICES.20` slice 4 (open audit item 1) / `.21` acceptance (e).

⛔ PROMOTED OUT OF `rust/target/audit_scratch/` BY `.21` (e). This is the producer of the published
"code folding is the LINKER, proven three ways" claim — 2 505 BL sites of which 1 964 are
SystemVerilog, and nine pre-link per-family copies collapsing to one. TOOLBOX 3.8's TRAP 3 is that a
per-family symbol name in a profile can be a LIE after folding, so this is the instrument that says
whether a profile row means what it says. It cannot be a scratch file.

Decide ICF (one shared copy, called from every family) vs INLINED-AWAY (SystemVerilog's copy folded
into its callers, leaving `sample` to attribute to the nearest preceding symbol). The two hypotheses
make OPPOSITE predictions about the call graph:

  ICF          => SystemVerilog rule methods contain `bl <target>` to that address.
  INLINED-AWAY => no SystemVerilog code calls it at all.

Usage: bl_callers.py <binary> <target-hex-vmaddr> [<target-hex-vmaddr> ...]
"""
import bisect
import subprocess
import sys

BIN = sys.argv[1]
TARGETS = {int(a, 16) for a in sys.argv[2:]}

# __text placement, read from the load commands rather than assumed.
vmaddr = fileoff = size = None
out = subprocess.run(["otool", "-l", BIN], capture_output=True, text=True).stdout
lines = out.splitlines()
for i, ln in enumerate(lines):
    if ln.strip() == "sectname __text":
        blk = lines[i:i + 8]
        for b in blk:
            b = b.strip()
            if b.startswith("addr "):
                vmaddr = int(b.split()[1], 16)
            elif b.startswith("size "):
                size = int(b.split()[1], 16)
            elif b.startswith("offset "):
                fileoff = int(b.split()[1])
        break
assert None not in (vmaddr, fileoff, size), "could not locate __text"
print(f"__text vmaddr=0x{vmaddr:x} fileoff={fileoff} size=0x{size:x}", file=sys.stderr)

with open(BIN, "rb") as fh:
    fh.seek(fileoff)
    text = fh.read(size)

# Symbol table (address -> name), sorted, so a call site maps to its enclosing function.
syms = []
nm = subprocess.run(["nm", "-C", BIN], capture_output=True, text=True).stdout
for ln in nm.splitlines():
    parts = ln.split(" ", 2)
    if len(parts) == 3 and parts[0].strip() and parts[1] in ("t", "T"):
        try:
            syms.append((int(parts[0], 16), parts[2]))
        except ValueError:
            pass
syms.sort()
addrs = [a for a, _ in syms]
print(f"text symbols: {len(syms)}", file=sys.stderr)


def enclosing(a):
    i = bisect.bisect_right(addrs, a) - 1
    return syms[i][1] if i >= 0 else "<none>"


hits = {t: [] for t in TARGETS}
n = len(text) // 4
for k in range(n):
    w = int.from_bytes(text[4 * k:4 * k + 4], "little")
    if (w >> 26) != 0b100101:          # BL
        continue
    imm = w & 0x03FFFFFF
    if imm & 0x02000000:
        imm -= 0x04000000
    a = vmaddr + 4 * k
    tgt = a + (imm << 2)
    if tgt in TARGETS:
        hits[tgt].append(a)

for t in sorted(TARGETS):
    cs = hits[t]
    print(f"\n=== target 0x{t:x} — {len(cs)} BL call sites ===")
    fams = {}
    for a in cs:
        name = enclosing(a)
        fam = "<other>"
        if "generated_parsers::" in name:
            fam = name.split("generated_parsers::", 1)[1].split("::", 1)[0]
        fams[fam] = fams.get(fam, 0) + 1
    for fam, c in sorted(fams.items(), key=lambda kv: -kv[1]):
        print(f"  {c:6d}  caller family: {fam}")
    for a in cs[:8]:
        print(f"    e.g. 0x{a:x}  in  {enclosing(a)}")
