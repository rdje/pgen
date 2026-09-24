#!/usr/bin/env python3
"""SV-CORPUS-GRAD.21 — the Annex A LITERAL-BRACKET census, read against the grammar.

IEEE prints literal tokens in bold and EBNF optionality in plain brackets; the PDF-to-text extraction
under docs/{systemverilog/{2017,2023},verilog/2005}/md keeps only the characters, so a bold `[` and an
optional `[` are the same byte. This instrument enumerates EVERY bracket pair of every Annex A
alternative in the three editions, classifies each pair as LITERAL or OPTIONAL — a generic rule where
the characters alone decide (an empty `[ ]`, an operator-led `[*`/`[+`/`[->`/`[$`), a curated table
where only the clause decides (dimensions, selects, ranges, repetition, the property operators) — and
for every LITERAL pair (a) cites the clause's own example that spells the brackets (a regex over the
edition's section texts, never Annex A itself) and (b) checks that the grammar renders the pair with
`lbrack`/`rbrack` in the production's rule (or one it references) rather than as `( … )?`.

  python3 docs/tasks/artifacts/sv_corpus_grad/annex_a_brackets/census.py            # rewrite census.tsv, print the summary
  python3 docs/tasks/artifacts/sv_corpus_grad/annex_a_brackets/census.py --check    # refuse if census.tsv differs from a fresh derivation
"""
import re, sys, os, glob, collections
ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', '..', '..', '..', '..'))
os.chdir(ROOT)
SRC = {
    '1800-2017': ('docs/systemverilog/2017/md/section-41-data-read-api.md', 'docs/systemverilog/2017/md'),
    '1800-2023': ('docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md', 'docs/systemverilog/2023/md'),
    '1364-2005': ('docs/verilog/2005/md/section-Annex_A-normative-formal-syntax-definition.md', 'docs/verilog/2005/md'),
}
GRAMMAR = 'grammars/systemverilog.ebnf'
PROBE_BIN = 'rust/target/release/parseability_probe'
PROBE_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'probes')
# ── the reproducers: one minimal construct per literal-bracket family, parsed on every profile the
# edition maps to; ACCEPT with the brackets is the PROOF that the grammar renders them (the rendering
# column above is a hint over the rule text, the probe is the measurement). A REJECT is a finding.
PROBE = {
 'packed_dimension': 'dims.sv', 'unpacked_dimension': 'dims.sv', 'associative_dimension': 'dims.sv', 'queue_dimension': 'dims.sv',
 'unsized_dimension': 'dims.sv', 'dimension': 'dims.sv', 'range': 'dims.sv', 'dynamic_array_new': 'dyn_new.sv',
 'enum_name_declaration': 'enum_range.sv', 'bins_or_options': 'bins.sv', 'trans_range_list': 'bins.sv',
 'covergroup_value_range': 'bins.sv', 'value_range': 'inside_dist.sv', 'open_value_range': 'inside_dist.sv',
 'cycle_delay_range': 'seq_ops.sv', 'consecutive_repetition': 'seq_ops.sv', 'non_consecutive_repetition': 'seq_ops.sv',
 'goto_repetition': 'seq_ops.sv', 'boolean_abbrev': 'seq_ops.sv', 'sequence_abbrev': 'seq_ops.sv', 'property_expr': 'prop_ops.sv',
 'stream_expression': 'stream.sv', 'select': 'selects.sv', 'constant_select': 'selects.sv', 'bit_select': 'selects.sv',
 'constant_bit_select': 'selects.sv', 'primary': 'selects.sv', 'constant_primary': 'selects.sv', 'module_path_primary': 'sp_path_select.sv',
 'net_lvalue': 'v2005_ports.v', 'variable_lvalue': 'v2005_hier.v', 'hierarchical_identifier': 'v2005_hier.v', 'port_reference': 'v2005_ports.v',
 'specify_input_terminal_descriptor': 'sp_setup_select.sv', 'specify_output_terminal_descriptor': 'sp_path_select.sv',
 'delayed_data': 'sp_setuphold_delayed.sv', 'delayed_reference': 'sp_setuphold_delayed.sv', 'edge_control_specifier': 'sp_width_edge2.sv',
}
# 1364-2005's productions probe with 1364 spellings (the 1800 reproducers use SystemVerilog types)
PROBE_1364 = {'dimension': 'dims_v2005.v', 'range': 'dims_v2005.v', 'primary': 'selects_v2005.v', 'constant_primary': 'selects_v2005.v',
              'module_path_primary': 'sp_path_select.sv', 'net_lvalue': 'selects_v2005.v', 'variable_lvalue': 'selects_v2005.v',
              'hierarchical_identifier': 'v2005_hier.v', 'port_reference': 'v2005_ports.v'}
# where no clause example carries the bracket, the literal reading's provenance is named instead
NOTE = {'delayed_data': 'no clause example spells the bracket; the literal reading is SV-CORPUS-GRAD.3.63 (bold in the PDF, measured against the corpus) — the probe is the proof',
        'delayed_reference': 'no clause example spells the bracket; the literal reading is SV-CORPUS-GRAD.3.63 (bold in the PDF, measured against the corpus) — the probe is the proof'}
EDITION_PROFILES = {'1800-2017': ['sv_2017'], '1800-2023': ['sv_2023'], '1364-2005': ['verilog_2005']}
_probe_cache = {}
def probe(fname, profile):
    key = (fname, profile)
    if key in _probe_cache: return _probe_cache[key]
    path = os.path.join(PROBE_DIR, fname)
    if not os.path.exists(path): v = 'NO-PROBE-FILE'
    elif not os.access(PROBE_BIN, os.X_OK): v = 'NOT-EVALUATED'
    else:
        import subprocess
        r = subprocess.run([PROBE_BIN, '--parse', 'systemverilog', path, '--profile', profile], capture_output=True, text=True)
        m = re.search(r'furthest_position=(\d+)', r.stdout + r.stderr)
        v = 'ACCEPT' if r.returncode == 0 else ('REJECT@' + (m.group(1) if m else '?'))
    _probe_cache[key] = v; return v
OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'census.tsv')

def productions(path):
    lines = open(path, encoding='utf-8').read().split('\n'); out = []; cur = None; buf = []
    for l in lines:
        m = re.match(r'^([a-z_][a-z_0-9]*?)([0-9,]*)\s*::=\s*(.*)$', l)
        if m:
            if cur: out.append((cur, buf))
            cur = m.group(1); buf = [m.group(3)] if m.group(3).strip() else []; continue
        if cur is None: continue
        t = l.rstrip()
        if not t.strip(): continue
        # a prose / heading line ends the block (Annex A alternatives start with `|` or are indented continuations)
        if re.match(r'^(#|\*\*|[A-Z][a-z])', t.strip()) and '::=' not in t and not t.strip().startswith('|'):
            out.append((cur, buf)); cur = None; buf = []; continue
        buf.append(t)
    if cur: out.append((cur, buf))
    return out

def alternatives(buf):
    alts = []; cur = ''
    for l in buf:
        t = l.strip()
        if t.startswith('|'): alts.append(cur); cur = t[1:]
        else: cur += ' ' + t
    alts.append(cur); return [a.strip() for a in alts if a.strip()]

def pairs(alt):
    toks = alt.split(); stack = []; res = []
    for i, tk in enumerate(toks):
        if tk == '[': stack.append(i)
        elif tk == ']' and stack:
            j = stack.pop(); res.append((j, ' '.join(toks[j + 1:i]), toks[j - 1] if j > 0 else '^', len(stack)))
    res.sort(key=lambda r: r[0]); return res

# ── the curated table: production -> list of (inner-regex, verdict, clause evidence regex) ──────────
# A pair matches the FIRST row whose inner-regex matches its inner text. Generic rules run first.
T = {
 'packed_dimension':      [(r'.*', 'LITERAL', r'\[\s*\d+\s*:\s*\d+\s*\]')],
 'unpacked_dimension':    [(r'.*', 'LITERAL', r'\]\s*;')],
 'associative_dimension': [(r'.*', 'LITERAL', r'\[\s*(string|\*|int|bit)')],
 'queue_dimension':       [(r'^:', 'OPTIONAL', None), (r'.*', 'LITERAL', r'\[\s*\$\s*\]')],
 'unsized_dimension':     [(r'.*', 'LITERAL', r'\[\s*\]')],
 'dimension':             [(r'.*', 'LITERAL', r'\[\s*\d+\s*:\s*\d+\s*\]')],
 'range':                 [(r'.*', 'LITERAL', r'\[\s*\d+\s*:\s*\d+\s*\]')],
 'dynamic_array_new':     [(r'^expression$', 'LITERAL', r'new\s*\['), (r'.*', 'OPTIONAL', None)],
 'enum_name_declaration': [(r'^integral_number', 'LITERAL', r'enum\b.*\w\[\d+(:\d+)?\]'), (r'.*', 'OPTIONAL', None)],
 'bins_or_options':       [(r'^covergroup_expression$', 'LITERAL', r'bins\s+\w+\s*\[\s*\d*\s*\]'), (r'^$', 'LITERAL', r'bins\s+\w+\s*\[\s*\]'), (r'.*', 'OPTIONAL', None)],
 'covergroup_value_range':[(r'.*:.*|.*\+/-.*|.*\+%-.*', 'LITERAL', r'bins\b.*\{\s*\[\s*[\w$]+\s*:'), (r'.*', 'OPTIONAL', None)],
 'value_range':           [(r'.*:.*|.*\+/-.*|.*\+%-.*', 'LITERAL', r'(inside|dist)\s*\{[^}]*\['), (r'.*', 'OPTIONAL', None)],
 'open_value_range':      [(r'.*:.*|.*\+/-.*|.*\+%-.*', 'LITERAL', r'(inside|dist)\s*\{[^}]*\['), (r'.*', 'OPTIONAL', None)],
 'cycle_delay_range':     [(r'^cycle_delay_const_range_expression$', 'LITERAL', r'##\s*\['), (r'.*', 'OPTIONAL', None)],
 'property_expr':         [(r'^(constant\s*_?expression|constant_range|cycle_delay_const_range_expression)$', 'LITERAL', 'PREV'), (r'.*', 'OPTIONAL', None)],
 'stream_expression':     [(r'^array_range_expression$', 'LITERAL', r'with\s*\['), (r'.*', 'OPTIONAL', None)],
 'select':                [(r'^part_select_range$', 'LITERAL', r'\w\[\d+\s*:\s*\d+\]'), (r'.*', 'OPTIONAL', None)],
 'constant_select':       [(r'^constant_part_select_range$', 'LITERAL', r'\w\[\d+\s*:\s*\d+\]'), (r'.*', 'OPTIONAL', None)],
 'bit_select':            [(r'.*', 'LITERAL', r'\w\[\w+\]')],
 'constant_bit_select':   [(r'.*', 'LITERAL', r'\w\[\w+\]')],
 'primary':               [(r'^range_expression$', 'LITERAL', r'\w\[\d+\s*:\s*\d+\]'), (r'^\{ \[ expression \] \} \[ range_expression \]$', 'OPTIONAL', None), (r'^expression$', 'LITERAL', r'\w\[\w+\]'), (r'.*', 'OPTIONAL', None)],
 'constant_primary':      [(r'^range_expression$|^constant_range_expression$', 'LITERAL', r'\w\[\d+\s*:\s*\d+\]'), (r'^constant_expression$|^expression$', 'LITERAL', r'\w\[\w+\]'), (r'.*', 'OPTIONAL', None)],
 'module_path_primary':   [(r'^constant_expression$|^expression$', 'LITERAL', r'\w\[\w+\]'), (r'^constant_range_expression$|^range_expression$', 'LITERAL', r'\w\[\d+\s*:\s*\d+\]'), (r'.*', 'OPTIONAL', None)],
 'net_lvalue':            [(r'^constant_expression$|^expression$', 'LITERAL', r'\w\[\w+\]'), (r'^constant_range_expression$|^range_expression$', 'LITERAL', r'\w\[\d+\s*:\s*\d+\]'), (r'.*', 'OPTIONAL', None)],
 'variable_lvalue':       [(r'^expression$', 'LITERAL', r'\w\[\w+\]'), (r'^range_expression$', 'LITERAL', r'\w\[\d+\s*:\s*\d+\]'), (r'.*', 'OPTIONAL', None)],
 'hierarchical_identifier': [(r'^constant_expression$', 'LITERAL', r'\w\[\d+\]\.\w'), (r'.*', 'OPTIONAL', None)],
 'port_reference':        [(r'^constant_range_expression$', 'LITERAL', r'\w\[\d+\s*:\s*\d+\]'), (r'.*', 'OPTIONAL', None)],
 'specify_input_terminal_descriptor':  [(r'^constant_range_expression$', 'LITERAL', r'\$\w+\s*\(\s*\w+\s*\['), (r'.*', 'OPTIONAL', None)],
 'specify_output_terminal_descriptor': [(r'^constant_range_expression$', 'LITERAL', r'\$\w+\s*\(\s*\w+\s*\['), (r'.*', 'OPTIONAL', None)],
 'delayed_data':          [(r'.*', 'LITERAL', r'\$(setuphold|recrem)\s*\(.*\w\[')],
 'delayed_reference':     [(r'.*', 'LITERAL', r'\$(setuphold|recrem)\s*\(.*\w\[')],
 'edge_control_specifier':[(r'.*', 'LITERAL', r'edge\s*\[')],
 'trans_range_list':      [(r'.*', 'LITERAL', r'\[\s*(\*|->|=)\s*\d')],
 'consecutive_repetition':[(r'.*', 'LITERAL', r'\[\*')],
 'non_consecutive_repetition': [(r'.*', 'LITERAL', r'\[=')],
 'goto_repetition':       [(r'.*', 'LITERAL', r'\[->')],
}
GENERIC = [
    (lambda inner, prev, prod: inner.strip() == '', 'LITERAL', 'empty `[ ]`', r'\[\s*\]'),
    (lambda inner, prev, prod: re.match(r'^\$', inner.strip()) is not None, 'LITERAL', 'operator-led `[$`', r'\[\s*\$'),
    (lambda inner, prev, prod: re.match(r'^(\*|\+)(\s|$)', inner.strip()) is not None, 'LITERAL', 'operator-led `[*` / `[+`', r'\[\s*[*+]'),
    (lambda inner, prev, prod: re.match(r'^(->|–>)', inner.strip()) is not None, 'LITERAL', 'operator-led `[->`', r'\[->'),
    (lambda inner, prev, prod: re.match(r'^=', inner.strip()) is not None and prod in ('non_consecutive_repetition', 'trans_range_list', 'boolean_abbrev', 'sequence_abbrev'), 'LITERAL', 'operator-led `[=` (repetition)', r'\[='),
]

def classify(prod, inner, prev):
    for pred, verdict, why, ev in GENERIC:
        if pred(inner, prev, prod): return verdict, 'generic: ' + why, ev
    for rx, verdict, ev in T.get(prod, []):
        if re.match(rx, inner.strip()):
            return verdict, ('table: ' + prod) if verdict == 'LITERAL' else 'table: EBNF optionality inside ' + prod, ev
    return 'OPTIONAL', 'default: EBNF optionality', None

def evidence(section_dir, annex_path, rx, prev=''):
    if not rx: return ''
    if rx == 'PREV': rx = r'\b' + re.escape(prev) + r'\s*\['
    best = None
    for f in sorted(glob.glob(section_dir + '/*.md')):
        if os.path.abspath(f) == os.path.abspath(annex_path): continue
        for n, l in enumerate(open(f, encoding='utf-8'), 1):
            if re.search(rx, l):
                hit = f"{f}:{n}: {l.strip()[:90]}"
                if re.search(r'[;)]', l): return hit          # a code-like line wins
                best = best or hit
    return best or 'NO-EVIDENCE'

g_text = open(GRAMMAR, encoding='utf-8').read()
g_rules = {}
for m in re.finditer(r'^([A-Za-z_][A-Za-z0-9_]*)\s*:=(.*?)(?=^\S[^\n]*:=|\Z)', g_text, re.M | re.S):
    g_rules[m.group(1)] = m.group(2)
ALIAS = {'dimension': 'unpacked_dimension', 'range': 'packed_dimension'}   # 1364-2005 names the grammar folds into the 1800 ones
def rendered(prod):
    prod = ALIAS.get(prod, prod)
    cands = [r for r in g_rules if r == prod or re.match(re.escape(prod) + r'(_sv_2017|_sv_2023|_v2005|_sv_only)$', r)]
    if not cands: return 'no-rule', ''
    # breadth-first through referenced rules, three hops: `lbrack` reached = the production renders a bracket token
    for c in cands:
        seen = {c}; frontier = [c]
        for hop in range(3):
            nxt = []
            for r in frontier:
                if re.search(r'\blbrack\b', g_rules[r]): return ('rendered' if r == c else f'rendered(via {r}, hop {hop})'), c
                for ref in sorted(set(re.findall(r'\b([a-z_][a-z0-9_]*)\b', g_rules[r]))):   # sorted: the hint must be deterministic
                    if ref in g_rules and ref not in seen: seen.add(ref); nxt.append(ref)
            frontier = nxt
    return 'hint:no-lbrack-within-3-hops', ','.join(cands[:4])

rows = []
for ed, (annex, secdir) in SRC.items():
    for prod, buf in productions(annex):
        for ai, alt in enumerate(alternatives(buf)):
            for pi, (pos, inner, prev, depth) in enumerate(pairs(alt)):
                verdict, why, ev = classify(prod, inner, prev)
                evd = evidence(secdir, annex, ev, prev) if verdict == 'LITERAL' else ''
                if evd == 'NO-EVIDENCE' and prod in NOTE: evd = 'NOTE: ' + NOTE[prod]
                rend, rule = rendered(prod) if verdict == 'LITERAL' else ('', '')
                pr = ''
                if verdict == 'LITERAL':
                    f = (PROBE_1364.get(prod) if ed == '1364-2005' else None) or PROBE.get(prod)
                    pr = (f + ': ' + ' '.join(f'{p}={probe(f, p)}' for p in EDITION_PROFILES[ed])) if f else 'NO-PROBE'
                rows.append([ed, prod, str(ai), str(pi), str(depth), prev, inner.replace('\t', ' ')[:100], verdict, why, evd, rend, rule, pr])
hdr = ['edition', 'production', 'alt', 'pair', 'depth', 'prev_token', 'inner', 'verdict', 'basis', 'clause_evidence', 'grammar', 'grammar_rule', 'probe']
tsv = '\n'.join(['\t'.join(hdr)] + ['\t'.join(r) for r in rows]) + '\n'
if '--check' in sys.argv:
    cur = open(OUT, encoding='utf-8').read() if os.path.exists(OUT) else ''
    if cur != tsv: print('annex-a-brackets: ✗ census.tsv differs from a fresh derivation — re-run without --check', file=sys.stderr); sys.exit(1)
    print('annex-a-brackets: OK (census equals a fresh derivation)'); sys.exit(0)
open(OUT, 'w', encoding='utf-8').write(tsv)
c = collections.Counter((r[0], r[7]) for r in rows)
print('ANNEX-A-BRACKETS: pairs=%d' % len(rows), ' '.join(f'{ed}:{v}={n}' for (ed, v), n in sorted(c.items())))
lit = [r for r in rows if r[7] == 'LITERAL']
print('literal pairs=%d productions=%d no_evidence=%d unrendered_hint=%d no_rule=%d no_probe=%d probe_reject=%d' % (len(lit), len({(r[0], r[1]) for r in lit}), sum(1 for r in lit if r[9] == 'NO-EVIDENCE'), sum(1 for r in lit if r[10].startswith('hint:')), sum(1 for r in lit if r[10] == 'no-rule'), sum(1 for r in lit if r[12] == 'NO-PROBE'), sum(1 for r in lit if 'REJECT' in r[12])))
for r in lit:
    if r[9] == 'NO-EVIDENCE' or r[10] == 'no-rule' or r[12] == 'NO-PROBE' or 'REJECT' in r[12]: print('  ', r[0], r[1], f'alt{r[2]}/pair{r[3]}', f'[{r[6][:36]}]', r[10], '|', r[9][:50], '|', r[12])
