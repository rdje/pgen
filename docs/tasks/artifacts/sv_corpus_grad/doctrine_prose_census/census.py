#!/usr/bin/env python3
"""SV-CORPUS-GRAD.13v(c) — which NUMBERS in a doctrine's prose does its own enforcer never mention?

A doctrine's description has (at least) four homes — the registry row in `scripts/check_doctrines.sh`,
the §10 row in `DOCTRINE_ENFORCEMENT.md`, the derived book section (equal to §10 by construction), and
the enforcer's own header — and the meta-checks hold only the NAME roster equal. This lists, per
doctrine, every number (≥ 2 digits, thin-space/comma digit groups kept whole, optional %) that the
registry row or the §10 row cites and the enforcer script's TEXT does not contain, with context.

⛔ IT IS A LEAD GENERATOR, NOT A DEFECT COUNT. Most absent numbers are FOUNDING-MEASUREMENT history
(the incident that founded the doctrine) and are legitimately prose-only; a number an enforcer reads
from a register (FLOW-INTEGRITY's 30-minute floor) is also absent from its text and correct. The
finding is the LIVE subset — a count the prose states in the present tense that the tree can move —
and that adjudication is read, not computed (the leaf records it). Deterministic; reads the tree only.
"""
import re, sys
NUM = re.compile(r'(?<![A-Za-z0-9_./:-])(\d{1,3}(?:[  ,]\d{3})+(?:\.\d+)?|\d{2,}(?:\.\d+)?)(?:\s?%)?(?![A-Za-z0-9_./:-])')
def nums(t):
    out=set()
    for m in NUM.finditer(t):
        n=m.group(0).strip()
        out.add(re.sub(r'[  ,]', '', n))          # canonical digits, % kept
    return out
def present(n, src):
    digits=n.rstrip('%').strip()
    # the enforcer may spell the same number with any digit-group separator or none
    forms={digits, re.sub(r'(\d)(?=(\d{3})+$)', r'\1 ', digits), re.sub(r'(\d)(?=(\d{3})+$)', r'\1,', digits), re.sub(r'(\d)(?=(\d{3})+$)', r'\1 ', digits)}
    return any(f in src for f in forms)
drv=open('scripts/check_doctrines.sh',encoding='utf-8').read()
body=re.search(r"^DOCTRINES=\(\n(.*?)^\)$", drv, re.S|re.M).group(1)
rows=[]
for line in body.splitlines():
    line=line.strip()
    if not line.startswith('"'): continue
    inner=line.strip('"'); left,script=inner.rsplit('|',1); did,proves=left.split('|',1); rows.append((did,proves,script))
s10={}
for l in open('DOCTRINE_ENFORCEMENT.md',encoding='utf-8'):
    mm=re.match(r"^\| `([A-Z0-9+-]+)` \| [^|]+ \| `([^`]+)` \| (.*) \|\s*$", l)
    if mm: s10[mm.group(1)]=mm.group(3)
rows_reg=rows_s10=abs_reg=abs_s10=0
for did,proves,script in rows:
    try: src=open(script,encoding='utf-8').read()
    except FileNotFoundError: src=''
    hit=False
    for home,text in (("REG",proves),("S10",s10.get(did,''))):
        for m in NUM.finditer(text):
            n=re.sub(r'[  ,]','',m.group(0).strip())
            if present(n, src): continue
            ctx=text[max(0,m.start()-45):m.end()+35].replace('\n',' ')
            print(f"{did:<26} {home} {n:>10} | …{ctx}…")
            if home=="REG": abs_reg+=1; hit=True
            else: abs_s10+=1
        if home=="REG" and hit: rows_reg+=1
        if home=="S10" and any(not present(re.sub(r'[  ,]','',m.group(0).strip()),src) for m in NUM.finditer(text)): rows_s10+=1
print(f"DOCTRINE-PROSE-CENSUS: rows={len(rows)} rows_with_registry_number_absent={rows_reg} "
      f"rows_with_s10_number_absent={rows_s10} absent_numbers registry={abs_reg} s10={abs_s10}", file=sys.stderr)
