#!/usr/bin/env python3
"""SV-CORPUS-GRAD.3.42 — INLINE-ALT-CENSUS: every inline alternation `( a | b )` sitting inside a group
(bare / ? / * / +) of a rule that carries at least one return template — the shape
ENGINE-UNIVERSAL-SERVICES.96 types with the ENCLOSING rule's templates by branch index.
The ENGINE-UNIVERSAL-SERVICES.96 shape: the inner group's branch N is typed with the enclosing
rule's template N when one exists (measured 2026-09-06 on 1-, 2- and 3-alternative synthetics).
Textual, on the .ebnf; validated against known positives (see --self-test)."""
import re, sys

def strip_comments(text):
    out=[]
    for line in text.splitlines():
        # a '#' starts a comment unless inside quotes or a regex literal; good enough for this grammar
        i=0; inq=None; res=''
        while i < len(line):
            c=line[i]
            if inq:
                res+=c
                if c=='\\' and i+1<len(line): res+=line[i+1]; i+=2; continue
                if c==inq: inq=None
            else:
                if c in '"\'': inq=c; res+=c
                elif c=='/' and (i==0 or line[i-1] in ' \t(|'):
                    # regex literal /.../ — copy verbatim to the closing unescaped /
                    j=i+1
                    while j < len(line):
                        if line[j]=='\\': j+=2; continue
                        if line[j]=='/': break
                        j+=1
                    res+=line[i:j+1]; i=j+1; continue
                elif c=='#': break
                else: res+=c
            i+=1
        out.append(res)
    return '\n'.join(out)

def split_rules(text):
    # a rule starts at column 0 with `name :=`; annotations (@...) precede it
    rules=[]; cur=None
    for line in text.splitlines():
        m=re.match(r'^([A-Za-z_][A-Za-z0-9_]*)\s*:=(.*)$', line)
        if m:
            if cur: rules.append(cur)
            cur=[m.group(1), m.group(2)]
        elif cur is not None:
            if re.match(r'^@', line) or re.match(r'^\S', line) and not line.startswith(' '):
                # an annotation line or a non-indented line ends the rule body
                if line.strip()=='' : continue
                if re.match(r'^@', line): rules.append(cur); cur=None; continue
            cur[1]+='\n'+line
    if cur: rules.append(cur)
    return rules

def split_top(body, sep='|'):
    """split on top-level separator, respecting ( ) { } [ ] and quotes/regex."""
    parts=[]; depth=0; cur=''; i=0; inq=None
    while i < len(body):
        c=body[i]
        if inq:
            cur+=c
            if c=='\\' and i+1<len(body): cur+=body[i+1]; i+=2; continue
            if c==inq: inq=None
            i+=1; continue
        if c in '"\'': inq=c; cur+=c; i+=1; continue
        if c=='/' and (i==0 or body[i-1] in ' \t(|'):
            j=i+1
            while j < len(body):
                if body[j]=='\\': j+=2; continue
                if body[j]=='/': break
                j+=1
            cur+=body[i:j+1]; i=j+1; continue
        if c in '({[': depth+=1
        elif c in ')}]': depth-=1
        if c==sep and depth==0:
            parts.append(cur); cur=''
        else: cur+=c
        i+=1
    parts.append(cur)
    return parts

def split_alt(alt):
    """returns (sequence_text, template_text or None)"""
    parts=split_top(alt, '-')
    # find '->' at top level
    depth=0; i=0; inq=None
    while i < len(alt)-1:
        c=alt[i]
        if inq:
            if c=='\\': i+=2; continue
            if c==inq: inq=None
            i+=1; continue
        if c in '"\'': inq=c; i+=1; continue
        if c=='/' and (i==0 or alt[i-1] in ' \t(|'):
            j=i+1
            while j < len(alt):
                if alt[j]=='\\': j+=2; continue
                if alt[j]=='/': break
                j+=1
            i=j+1; continue
        if c in '({[': depth+=1
        elif c in ')}]': depth-=1
        if depth==0 and alt[i:i+2]=='->':
            return alt[:i], alt[i+2:]
        i+=1
    return alt, None

def elements(seq):
    """top-level elements of a sequence: returns list of (text, is_group_alternation, n_branches, quant)"""
    els=[]; i=0; n=len(seq)
    while i < n:
        c=seq[i]
        if c.isspace(): i+=1; continue
        if c=='@':
            # annotation like @probe_sample: "x" — skip token + value
            m=re.match(r'@[a-z_]+:\s*("[^"]*"|\S+)', seq[i:])
            i+= m.end() if m else 1; continue
        if c in '&!':
            # lookahead prefix: treat with next element
            i+=1; continue
        if c=='(':
            depth=0; j=i; inq=None
            while j < n:
                d=seq[j]
                if inq:
                    if d=='\\': j+=2; continue
                    if d==inq: inq=None
                    j+=1; continue
                if d in '"\'': inq=d; j+=1; continue
                if d=='/' and (j==0 or seq[j-1] in ' \t(|'):
                    k=j+1
                    while k < n:
                        if seq[k]=='\\': k+=2; continue
                        if seq[k]=='/': break
                        k+=1
                    j=k+1; continue
                if d=='(': depth+=1
                elif d==')':
                    depth-=1
                    if depth==0: break
                j+=1
            inner=seq[i+1:j]; quant=''
            k=j+1
            if k < n and seq[k] in '?*+': quant=seq[k]; k+=1
            branches=split_top(inner,'|')
            els.append((seq[i:k], len(branches)>1, len(branches), quant)); i=k; continue
        # terminal / regex / rule ref
        if c=='"' or c=="'":
            j=i+1
            while j<n and seq[j]!=c:
                if seq[j]=='\\': j+=1
                j+=1
            j+=1
        elif c=='/':
            j=i+1
            while j<n:
                if seq[j]=='\\': j+=2; continue
                if seq[j]=='/': break
                j+=1
            j+=1
        else:
            m=re.match(r'[A-Za-z_$][A-Za-z0-9_]*', seq[i:])
            j=i+(m.end() if m else 1)
        quant=''
        if j<n and seq[j] in '?*+': quant=seq[j]; j+=1
        els.append((seq[i:j], False, 1, quant)); i=j
    return els

def census(path):
    text=strip_comments(open(path).read())
    rows=[]
    for name, body in split_rules(text):
        alts=split_top(body,'|')
        parsed=[split_alt(a) for a in alts]
        n_templates=sum(1 for _,t in parsed if t is not None)
        if n_templates==0: continue
        for ai,(seq,tmpl) in enumerate(parsed):
            els=elements(seq)
            for ei,(txt,is_alt,nb,quant) in enumerate(els):
                if not is_alt: continue
                dollar=ei+1
                referenced = tmpl is not None and re.search(r'\$%d\b'%dollar, tmpl) is not None
                # an UNANNOTATED alternative keeps the raw sequence envelope, so a corrupted element
                # in it is VISIBLE (measured 2026-09-06: `s := t -> {...} | ( a b | c ) d` on `A.m`
                # yields element_0 = {"body": "A", "kind": "tagged"}) -- treat as referenced
                if tmpl is None: referenced = True
                # inner branch k is typed with template k when template k exists
                shadowed=[k for k in range(nb) if k < len(parsed) and parsed[k][1] is not None]
                rows.append(dict(rule=name, alt=ai+1, alts=len(alts), element=dollar, quant=quant or 'bare',
                                 branches=nb, referenced=referenced, shadowed_branches=shadowed, group=txt.strip()[:120],
                                 template=(tmpl or '').strip()[:100]))
    return rows

def self_test():
    """RED controls: the tracked synthetics that REPRODUCE the class must each read exposed=1, their lifted
    twins exposed=0. A miss exits 7 (MISCALIBRATED) rather than reporting a clean grammar."""
    import os
    d=os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', '..', 'engine_universal_services', 'es96_inline_alt_template')
    expect={'optional_alt.ebnf':1,'inline_alt.ebnf':1,'optional_alt_single.ebnf':1,'unannot_alt.ebnf':1,
            'lifted.ebnf':0,'optional_alt_single_lifted.ebnf':0,'unannot_alt_lifted.ebnf':0}
    bad=[]
    for f,n in expect.items():
        rows=census(os.path.join(d,f)); got=sum(1 for r in rows if r['referenced'] and r['shadowed_branches'])
        print(f"  self-test {f:32s} exposed={got} expected={n} {'ok' if got==n else 'MISS'}")
        if got!=n: bad.append(f)
    if bad:
        print(f"INLINE-ALT-CENSUS: MISCALIBRATED — {len(bad)} control(s) did not fire: {bad}", file=sys.stderr); sys.exit(7)
    print(f"INLINE-ALT-CENSUS: self-test ok ({len(expect)} controls)")

if __name__=='__main__':
    if len(sys.argv)>1 and sys.argv[1]=='--self-test':
        self_test(); sys.exit(0)
    path=sys.argv[1] if len(sys.argv)>1 else 'grammars/systemverilog.ebnf'
    if not os.path.exists(path) if (os:=__import__('os')) else False:
        print(f"INLINE-ALT-CENSUS: REFUSED — cannot read {path}", file=sys.stderr); sys.exit(2)
    rows=census(path)
    exposed=[r for r in rows if r['referenced'] and r['shadowed_branches']]
    print(f"INLINE-ALT-CENSUS: grammar={path} annotated_rules_with_inline_alternation_sites={len(rows)} exposed(referenced_by_template AND a_branch_index_has_a_template)={len(exposed)}")
    for r in sorted(rows, key=lambda r:(not (r['referenced'] and r['shadowed_branches']), r['rule'])):
        tag=('EXPOSED(raw-envelope)' if not r['template'] else 'EXPOSED ') if (r['referenced'] and r['shadowed_branches']) else ('unreferenced' if not r['referenced'] else 'no_template_at_branch_index')
        print(f"  {tag:28s} {r['rule']} alt {r['alt']}/{r['alts']} ${r['element']} {r['quant']} branches={r['branches']} shadowed={r['shadowed_branches']} :: {r['group']}  -> {r['template']}")
