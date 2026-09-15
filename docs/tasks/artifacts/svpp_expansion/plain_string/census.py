#!/usr/bin/env python3
"""SVPP-EXPANSION.5(k) census (the pre-registration instrument, tracked; run: python3 docs/tasks/artifacts/svpp_expansion/plain_string/census.py): function-like `define bodies (continuations joined) in which a FORMAL
appears as a whole identifier inside a PLAIN "..." string literal that is outside any `"..`" region.
IEEE 1800-2017 22.5.1 :370-372: the characters of such a string "become part of the string in the
replaced macro text" (no substitution); PGEN substituted there until PGEN-SVPP-EXPANSION-0027. Per suite: files, defines, sites."""
import os, re, sys
os.chdir(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "..", "..", ".."))
from collections import Counter, defaultdict
roots = ["stimuli/sv/subs", "stimuli/sv/uvm"]
exts = (".sv", ".v", ".svh", ".vh")
def suite_of(p):
    if p.startswith("stimuli/sv/uvm/"): return "uvm-core"
    return p.split("/")[3]
def plain_strings(body):
    r"""yield (start,end) of plain "..." literals outside `"..`" regions; `\`" is skipped whole"""
    i, n = 0, len(body); out = []
    in_region = False
    while i < n:
        if body.startswith('`\\`"', i): i += 4; continue
        if body.startswith('`"', i): in_region = not in_region; i += 2; continue
        if body[i] == '"' and not in_region:
            j = i + 1
            while j < n and body[j] != '"':
                if body[j] == '\\': j += 1
                j += 1
            out.append((i + 1, min(j, n))); i = j + 1; continue
        i += 1
    return out
ident = re.compile(r'[A-Za-z_][A-Za-z0-9_$]*')
files = defaultdict(int); defines = Counter(); sites = Counter(); examples = defaultdict(list)
for root in roots:
    for dp, dn, fn in os.walk(root):
        for f in fn:
            if not f.endswith(exts): continue
            p = os.path.join(dp, f)
            try:
                with open(p, encoding="utf-8", errors="replace") as fh:
                    text = fh.read()
            except Exception:
                continue
            if '`define' not in text: continue
            joined = text.replace("\\\n", " ")
            hit_file = False
            for line in joined.split("\n"):
                m = re.match(r'\s*`define\s+([A-Za-z_][A-Za-z0-9_$]*)\(([^)]*)\)', line)
                if not m: continue
                formals = set()
                for part in m.group(2).split(','):
                    name = part.split('=')[0].strip()
                    if name: formals.add(name)
                body = line[m.end():]
                if '"' not in body: continue
                n_sites = 0
                for (a, b) in plain_strings(body):
                    for w in ident.findall(body[a:b]):
                        if w in formals: n_sites += 1
                if n_sites:
                    s = suite_of(p); hit_file = True
                    defines[s] += 1; sites[s] += n_sites
                    if len(examples[s]) < 3: examples[s].append((p, m.group(1), body.strip()[:110]))
            if hit_file: files[suite_of(p)] += 1
print("suite\tfiles\tdefines\tsites")
for s in sorted(files): print(f"{s}\t{files[s]}\t{defines[s]}\t{sites[s]}")
print(f"TOTAL\t{sum(files.values())}\t{sum(defines.values())}\t{sum(sites.values())}")
for s in sorted(examples):
    for p, name, body in examples[s]: print(f"  {s}: {p}  `define {name}(...) {body}")
