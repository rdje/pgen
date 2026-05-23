#!/usr/bin/env python3
"""SV-EXH-PROOF audit tool — defect class A2 (LRM-optional vs active grammar).

Compares `(...)?` quantifier counts per rule between
`grammars/systemverilog_lrm_profiled_generated.ebnf` (auto-extracted from
the IEEE 1800 Annex A markdown) and `grammars/systemverilog.ebnf` (the
active grammar with manual corrections). Flags rules where the
LRM-generated version has MORE `?` than the active — candidates for a
dropped optional clause.

Caveat — the LRM uses `[ X ]` for BOTH EBNF-optional AND literal SV bracket
syntax (`arr[i]`, `bins b[3]`, `foreach (arr[i])`). The extractor treats all
`[ X ]` as optional, mis-encoding literal-bracket cases. The active grammar
correctly rewrites literal-bracket cases to `lbrack X rbrack`. So the
"drops" this tool flags are MOSTLY legitimate fixes (active is correct), not
defects. Per-candidate manual LRM-source comparison is needed to distinguish
genuine defects from legitimate corrections.

Run periodically; failure to verify a candidate is a likely new instance of
A2 or a regression on a prior correction.

See: docs/reference/SV_EXH_PROOF_DEFECT_TAXONOMY.md §A2.
"""
import re, sys
from pathlib import Path

REPO = Path("/Users/richarddje/Documents/github/pgen")
LRM = REPO / "grammars/systemverilog_lrm_profiled_generated.ebnf"
ACT = REPO / "grammars/systemverilog.ebnf"

# Rule extractor: matches `rule_name :=` at column 0, then the body until the
# next column-0 `rule_name :=` (or end-of-file). Strips comments + trivia
# directives for the count.
RULE_RE = re.compile(r'^([a-z_][a-z_0-9]*) :=', re.M)
ANNOTATION_RE = re.compile(r'^@[a-z_]+:.*$', re.M)
COMMENT_RE = re.compile(r'^\s*#.*$', re.M)

def parse(path: Path) -> dict[str, str]:
    src = path.read_text()
    # strip comments + @annotations BEFORE rule split
    src = COMMENT_RE.sub("", src)
    src = ANNOTATION_RE.sub("", src)
    matches = list(RULE_RE.finditer(src))
    rules = {}
    for i, m in enumerate(matches):
        end = matches[i + 1].start() if i + 1 < len(matches) else len(src)
        rules[m.group(1)] = src[m.start():end]
    return rules

def question_count(body: str) -> int:
    """Count `)?` (close-paren-question) — the EBNF optional-group form."""
    # Strip strings + return-annotation bodies (`-> { ... }`) which may contain
    # unrelated `?` (e.g. in identifiers / comments).
    # For a count comparison, conservatively strip:
    # 1. Quoted strings.
    body = re.sub(r'"[^"]*"', '""', body)
    # 2. Return-annotation `-> {...}` bodies (nested braces — single-pass via greedy strip
    #    then balance-aware would be ideal; here, use a simple per-line filter on
    #    arrow-prefixed continuation lines).
    body = re.sub(r'->\s*\{[^}]*\}', '', body, flags=re.S)
    body = re.sub(r'->\s*\[[^\]]*\]', '', body, flags=re.S)
    body = re.sub(r'->\s*\"[^\"]*\"', '', body)
    body = re.sub(r'->\s*\$[a-zA-Z0-9_.]+', '', body)
    return len(re.findall(r'\)\?', body))

lrm = parse(LRM)
act = parse(ACT)
print(f"[a2-oracle] LRM-generated rules: {len(lrm)}  active rules: {len(act)}", file=sys.stderr)

common = sorted(set(lrm) & set(act))
print(f"[a2-oracle] common rules: {len(common)}", file=sys.stderr)

# Find candidates: LRM has more `?` than active
candidates = []
for name in common:
    lc = question_count(lrm[name])
    ac = question_count(act[name])
    if lc > ac:
        candidates.append((name, lc, ac, lc - ac))

candidates.sort(key=lambda x: (-x[3], x[0]))
print(f"\n[a2-oracle] {len(candidates)} candidate rules with dropped `?` quantifiers:")
print(f"{'RULE':<60} LRM ACT DROP")
for name, lc, ac, diff in candidates:
    print(f"  {name:<60} {lc:>3} {ac:>3} {diff:>3}")
