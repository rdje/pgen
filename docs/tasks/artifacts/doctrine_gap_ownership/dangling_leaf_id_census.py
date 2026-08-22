#!/usr/bin/env python3
"""DOCTRINE-GAP-OWNERSHIP.8 — does every `<TREE>.<leaf>` id named by the task-tree INDEX
resolve to a leaf-defining heading in that tree's file?

The `[[wikilink]]` sibling of this question is `.7`. This is the SAME defect class in the
OTHER namespace: a reference that reaches nothing, fails silently, in the passing
direction, and still reads like a reference.

⛔⛔ THE HEADING GRAMMAR IS THE WHOLE DIFFICULTY, AND A NAIVE CENSUS OVER-REPORTS BADLY.
A leaf heading is NOT `### `<id>``. In practice it is

    ### `.1` — the INVENTORY: ...                 (bare)
    ### ⛔⛔ `.40` NEW `todo` — **THE COLD-CLONE ...  (status emoji BEFORE the id)
    ### ⚠️ `.41` NEW `todo` — FOUR more gates ...
    #### `.3a` — the `<array_access>` arm ...      (deeper level)
    ### `H.17.1` — **REPAIR THE 5 LIVE ...          (alpha-prefixed trees: no leading dot)

A pattern anchored as `^#{2,6} \x60` — i.e. requiring the backtick IMMEDIATELY after the
hashes — silently skips every emoji-prefixed heading. Measured 2026-08-22: that mistake
reported CI-PARITY-GATE-ROT `.40`/`.41`/`.42` as MISSING when all three are present
(lines 249/146/186), and inflated the repo-wide population to 38. The prefix is FREE TEXT;
match the FIRST backticked token on the heading line, never a fixed offset.

Read-only. Exit code = the MISSING count (the only bucket that is a defect), so a future
ratchet keys on it — same contract as `.7`'s census.
"""
import io, os, re, sys, collections

TASKS = 'docs/tasks'
INDEX = 'docs/TASK_TREE.md'

# ⛔⛔ THERE IS NO SINGLE LEAF-DEFINITION CONVENTION IN THIS REPOSITORY. Measured 2026-08-22,
# THREE are in live use, and a census that knows only one reports the other two as MISSING:
#   (A) backticked id in a heading, optional free-text/emoji prefix
#         ### `.1` — ...            ### ⛔⛤ `.40` NEW `todo` — ...      #### `.3a` — ...
#   (B) numbered prose heading naming the FULLY-QUALIFIED id, NOT backticked
#         ## 23. PARSE-HARNESS.11 — the blessed scratch slot's OPERATING MANUAL ...
#   (C) a body field rather than a heading (SV-EXH-PROOF)
#         - ID: `SV-EXH-PROOF.3.3.4.b.6.2.15`
# All three are matched below. The plurality IS the finding: "is this leaf owned?" is not
# mechanically answerable while three spellings coexist, which is why no gate reads it.
HEADING = re.compile(r'^#{2,6}\s+[^`\n]*`([^`\n]+)`', re.M)
# A tree file is one that defines at least one leaf heading.
def is_tree(text): return HEADING.search(text) is not None
# Leaf-id shaped: starts with a digit, or a letter immediately followed by a digit / dot-digit.
LEAFISH = re.compile(r'^(?:\d|[A-Za-z]\d|[A-Za-z]\.\d)')

def main():
    trees, headings = [], {}
    for fn in sorted(os.listdir(TASKS)):
        if not fn.endswith('.md'):
            continue
        text = io.open(os.path.join(TASKS, fn), encoding='utf-8').read()
        if not is_tree(text):
            continue
        name = fn[:-3]
        trees.append(name)
        defined = {m.group(1).strip().lstrip('.') for m in HEADING.finditer(text)}
        # (B) numbered prose heading naming the fully-qualified id
        for m in re.finditer(r'^#{2,6}\s+[^\n]*?' + re.escape(name) + r'\.([A-Za-z0-9]+(?:\.[A-Za-z0-9]+)*)', text, re.M):
            defined.add(m.group(1).strip().lstrip('.'))
        # (C) an `- ID: `<fully-qualified id>`` body field
        for m in re.finditer(r'^\s*-\s*ID:\s*`' + re.escape(name) + r'\.([^`\n]+)`', text, re.M):
            defined.add(m.group(1).strip().lstrip('.'))
        headings[name] = defined
    trees.sort(key=len, reverse=True)          # longest-first: SV-CORPUS-GRAD before SV-CORPUS

    idx = io.open(INDEX, encoding='utf-8').read()
    refs = collections.defaultdict(set)
    for t in trees:
        for m in re.finditer(re.escape(t) + r'\.([A-Za-z0-9]+(?:\.[A-Za-z0-9]+)*)', idx):
            refs[t].add(m.group(1))

    buckets = collections.defaultdict(list)
    for t in sorted(refs):
        for suf in sorted(refs[t]):
            if suf == 'md' or suf.endswith('.md'):
                buckets['NOT-A-LEAF-ID'].append((t, suf)); continue
            if not LEAFISH.match(suf):
                buckets['NOT-A-LEAF-ID'].append((t, suf)); continue
            if suf.lstrip('.') in headings[t]:
                buckets['RESOLVED'].append((t, suf)); continue
            # a leaf may be written up in a sibling DETAIL document rather than in the tree
            stem = suf.replace('.', '')
            detail = [f for f in os.listdir(TASKS)
                      if f.startswith(t + '-') and f.endswith('.md') and stem.lower() in f.lower().replace('-', '')]
            buckets['DETAIL-DOC' if detail else 'MISSING'].append((t, suf))

    order = ['MISSING', 'DETAIL-DOC', 'RESOLVED', 'NOT-A-LEAF-ID']
    total = sum(len(buckets[b]) for b in order)
    print('LEAF-ID-CENSUS: scanned %d `<TREE>.<leaf>` references in %s against %d tree files'
          % (total, INDEX, len(trees)))
    print('  bucket           count   meaning')
    meaning = {
        'MISSING':       '⛔ the defect bucket — an id the index names with no leaf heading',
        'DETAIL-DOC':    'written up in a sibling detail document, not in the tree file',
        'RESOLVED':      'resolves to a leaf-defining heading in its tree',
        'NOT-A-LEAF-ID': 'false positive by construction (`<TREE>.md` links, prose)',
    }
    for b in order:
        print('  %-15s %5d   %s' % (b, len(buckets[b]), meaning[b]))
    if buckets['MISSING']:
        print('\nMISSING detail:')
        for t, suf in buckets['MISSING']:
            print('  %-32s %s' % (t, suf))
    return len(buckets['MISSING'])

if __name__ == '__main__':
    sys.exit(main())
