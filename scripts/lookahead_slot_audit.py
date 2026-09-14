#!/usr/bin/env python3
"""scripts/lookahead_slot_audit.py — doctrine LOOKAHEAD-SLOT-TEMPLATES (SV-CORPUS-GRAD.3.61).

⛔ WHAT THIS PROVES. A return template never binds a LOOKAHEAD child. In the engine a lookahead (`!x`,
`&x`, `!( … )`) is a CHILD of the sequence it stands in — it occupies a positional slot and binds `[]`
(the interpreter's `/l` node path; the generated parser keeps a successful lookahead's pushes) — so
`$N` counts it, and inside a repeated group `( !( … ) comma item )*` the guard is child 1, the comma
child 2 and the item child 3: `$2::2*` takes the COMMA. The engine's own documentation says `$<int>`
is "the N'th child (1-indexed) of the current rule's parse content"; a lookahead is content.

⛔ WHY IT EXISTS, measured rather than imagined (`SV-CORPUS-GRAD.3.61`, 2026-09-10). Ten sites in
`grammars/systemverilog.ebnf` bound a lookahead and SHIPPED that way, every one green under every
gate the tree has: `type_assignment_sv_2017` (and its three twins) wrote `{name: $1, value: $2}`
behind a leading `!kw_type`, so every `parameter type T = int` on every release from `1.0.79` to
`1.0.229` read `name: []`, `value: T` and LOST the default type; the `.3.35` type lists (`1.0.210`)
wrote `[$1, $2::2*]` over a guard-opened group and emitted `{"kind": "comma"}` in place of every
assignment after the first; the `.13c.2s` argument lists (`1.0.191`) wrote `[$2::2*]` over
`( comma !dot ( arg )? )*` and filled `ordered_tail` with the guard's empty nodes. The accept set
never moved, the AST identity sweeps compared each release with the one before — consistently wrong
trees are byte-identical — and the certificate reads reachability, not bindings. Nothing in the tree
compared a template's indices with the sequence's lookahead slots; this does.

THREE FINDING KINDS, each a shape the engine can never bind usefully:
  la_slot      — the template references `$p` and child p of the alternative is a lookahead (binds `[]`).
  slice_child  — `$N::K*` where child K of the repeated group at slot N is a lookahead, a separator
                 terminal (`comma`, `semi`, `colon`) or beyond the group's arity.
  raw_guarded  — a group that OPENS with a lookahead is bound RAW (`$N` with no `::K*` extraction), so
                 the envelope carries the guard's empty node beside every item.

SCOPE, stated as a list rather than a promise: the grammars written in the space-separated token
style this tokenizer reads (a rule per `name :=`, `#` and `//` comments, `/…/` regex terminals,
`( … )` groups with a trailing quantifier). The bootstrap-chain grammars (`ebnf.ebnf`,
`return_annotation.ebnf`, `semantic_annotation.ebnf`, `builtin_*.ebnf`) spell tokens as `'('` and
break sequences across lines inside one parenthesis; measured 2026-09-10 this tokenizer reads
`'(' expression ')'` as ONE element there, so its slot numbering is not the engine's and a green
reading there would be an instrument blind spot, not evidence. They are OUT OF SCOPE here, and the
engine-side lint routed by `.3.61` is where they will be read. An out-of-scope grammar passed
explicitly is audited anyway and says so.

Usage:
  python3 scripts/lookahead_slot_audit.py                 # the default scope, exit 0 = clean, 1 = findings
  python3 scripts/lookahead_slot_audit.py GRAMMAR...      # explicit grammars
  python3 scripts/lookahead_slot_audit.py --self-test     # replays the three finding kinds + a clean control (exit 7 on a miss)
Prints `LOOKAHEAD-SLOT-AUDIT: grammars=N alternatives=A with_lookaheads=L findings=F` and one line per finding.
"""
from __future__ import annotations
import pathlib, re, sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
DEFAULT_SCOPE = [
    "grammars/systemverilog.ebnf", "grammars/systemverilog_preprocessor.ebnf", "grammars/vhdl.ebnf",
    "grammars/rtl_frontend.ebnf", "grammars/rtl_const_expr.ebnf", "grammars/regex.ebnf", "grammars/json.ebnf",
    "grammars/systemverilog_2017_lrm_extracted.ebnf", "grammars/systemverilog_2023_lrm_extracted.ebnf",
    "grammars/verilog_2005_lrm_extracted.ebnf", "grammars/systemverilog_lrm_profiled_generated.ebnf",
    "grammars/systemverilog_lrm_profiled_wrapper.ebnf",
]
OUT_OF_SCOPE = {"ebnf.ebnf", "return_annotation.ebnf", "semantic_annotation.ebnf",
                "builtin_return_annotation.ebnf", "builtin_semantic_annotation.ebnf"}
SEPARATORS = {"comma", "semi", "colon"}
INLINE_DIRECTIVE = re.compile(r'@[a-z_]+:\s*"(?:[^"\\]|\\.)*"')


def strip_comments(text: str) -> str:
    out = []
    for line in text.splitlines():
        s = line.lstrip()
        if s.startswith("#") or s.startswith("@"):
            continue
        line = INLINE_DIRECTIVE.sub("", line)
        line = re.sub(r"//.*$", "", line)
        out.append(line)
    return "\n".join(out)


def top_elems(seq: str) -> list[str]:
    """Split a sequence into its top-level elements (a group is one element; a bare `!`/`&` glues forward)."""
    out, depth, cur, i = [], 0, "", 0
    while i < len(seq):
        c = seq[i]
        if c == "/" and depth == 0 and (cur == "" or cur in ("!", "&")):
            j = i + 1
            while j < len(seq) and seq[j] != "/":
                if seq[j] == "\\":
                    j += 1
                j += 1
            cur += seq[i:j + 1]; i = j + 1; continue
        if c == '"':
            j = seq.find('"', i + 1)
            if j < 0:
                j = len(seq) - 1
            cur += seq[i:j + 1]; i = j + 1; continue
        if c == "(":
            depth += 1
        if c == ")":
            depth -= 1
        if c.isspace() and depth == 0:
            if cur:
                out.append(cur); cur = ""
        else:
            cur += c
        i += 1
    if cur:
        out.append(cur)
    glued: list[str] = []
    for e in out:
        if glued and glued[-1] in ("!", "&"):
            glued[-1] += e
        else:
            glued.append(e)
    return glued


def split_alts(body: str) -> list[str]:
    out, depth, cur = [], 0, ""
    for c in body:
        if c == "(":
            depth += 1
        if c == ")":
            depth -= 1
        if c == "|" and depth == 0:
            out.append(cur); cur = ""
        else:
            cur += c
    out.append(cur)
    return out


def group_children(g: str) -> list[str] | None:
    if not g.startswith("("):
        return None
    inner = re.sub(r"\)\s*[*+?]?$", "", g[1:]).strip()
    return top_elems(inner)


def is_lookahead(e: str) -> bool:
    return e[:1] in "!&"


def audit_text(text: str, label: str) -> tuple[int, int, list[str]]:
    """Returns (alternatives, alternatives_with_lookaheads, findings)."""
    txt = strip_comments(text)
    findings: list[str] = []
    n_alts = n_la = 0
    for block in re.split(r"\n(?=[A-Za-z_][A-Za-z0-9_]* *:=)", txt):
        m = re.match(r"([A-Za-z_][A-Za-z0-9_]*) *:=(.*)", block, re.S)
        if not m:
            continue
        name, body = m.group(1), m.group(2)
        for alt in split_alts(body):
            if "->" not in alt:
                continue
            n_alts += 1
            seq, tmpl = alt.split("->", 1)
            els = top_elems(seq.strip())
            la = {i + 1 for i, e in enumerate(els) if is_lookahead(e)}
            guarded_groups = {i + 1 for i, e in enumerate(els) if e.startswith("(") and re.match(r"\(\s*[!&]", e)}
            # an alternative "has lookaheads" when one stands at top level or anywhere inside a group
            has_any = bool(la or guarded_groups or any(re.search(r"[\s(][!&]", e) for e in els))
            if has_any:
                n_la += 1
            refs = sorted({int(x) for x in re.findall(r"\$(\d+)", tmpl)})
            for r in refs:
                if r in la:
                    findings.append(f"{label}:{name}: la_slot ${r} is the lookahead `{els[r-1]}` — it binds [] (template: {tmpl.strip()[:80]})")
            for n_, k_ in re.findall(r"\$(\d+)::(\d+)\*", tmpl):
                n_, k_ = int(n_), int(k_)
                if n_ > len(els):
                    continue
                kids = group_children(els[n_ - 1])
                if kids is None:
                    continue
                if k_ > len(kids):
                    findings.append(f"{label}:{name}: slice_child ${n_}::{k_}* but the group has {len(kids)} children")
                elif is_lookahead(kids[k_ - 1]):
                    findings.append(f"{label}:{name}: slice_child ${n_}::{k_}* takes the lookahead `{kids[k_-1]}` — it binds []")
                elif kids[k_ - 1] in SEPARATORS:
                    findings.append(f"{label}:{name}: slice_child ${n_}::{k_}* takes the separator `{kids[k_-1]}` (the guard shifted the item to child {len(kids)})")
            for gslot in sorted(guarded_groups):
                if gslot in refs and f"${gslot}::" not in tmpl:
                    findings.append(f"{label}:{name}: raw_guarded ${gslot} binds the guard-opened group `{els[gslot-1][:50]}` raw — the envelope carries the guard's []")
    return n_alts, n_la, findings


SELF_TEST_GRAMMAR = """
# the four shapes .3.61 shipped, and a clean control
bad_slot := !kw_x ident ( assign value )? -> {name: $1, value: $2}
bad_slice := item ( !( comma ident ident ) comma item )* -> [$1, $2::2*]
bad_slice_guard := head ( comma !dot ( arg )? )* -> {head: $1, tail: [$2::2*]}
bad_raw := kw_type item ( !( comma kw_string ) comma item )* -> {first: $2, tail: $3}
good_slot := !kw_x ident ( assign value )? -> {name: $2, value: $3}
good_slice := item ( !( comma ident ident ) comma item )* -> [$1, $2::3*]
good_guard_last := lparen a ( comma b &comma )* comma c rparen -> {outs: [$2, $3::2*], last: $5}
good_plain := a ( comma a )* -> [$1, $2::2*]
"""


def self_test() -> int:
    n_alts, n_la, f = audit_text(SELF_TEST_GRAMMAR, "self")
    want = {"bad_slot": "la_slot", "bad_slice": "slice_child", "bad_slice_guard": "slice_child", "bad_raw": "raw_guarded"}
    got = {}
    for line in f:
        rule, kind = line.split(":")[1], line.split(": ")[1].split(" ")[0]
        got.setdefault(rule, set()).add(kind)
    ok = True
    for rule, kind in want.items():
        if kind not in got.get(rule, set()):
            print(f"SELF-TEST MISS: {rule} should report {kind}, reported {sorted(got.get(rule, set()))}"); ok = False
    for rule in ("good_slot", "good_slice", "good_guard_last", "good_plain"):
        if rule in got:
            print(f"SELF-TEST FALSE POSITIVE: {rule} reported {sorted(got[rule])}"); ok = False
    print(f"LOOKAHEAD-SLOT-AUDIT self-test: alternatives={n_alts} with_lookaheads={n_la} findings={len(f)} expected_rules={len(want)} {'OK' if ok else 'FAILED'}")
    return 0 if ok else 7


def main(argv: list[str]) -> int:
    if "--self-test" in argv:
        return self_test()
    paths = [pathlib.Path(a) for a in argv] or [ROOT / p for p in DEFAULT_SCOPE]
    total_alts = total_la = 0
    findings: list[str] = []
    for p in paths:
        if not p.exists():
            print(f"LOOKAHEAD-SLOT-AUDIT: missing grammar {p}", file=sys.stderr); return 2
        if p.name in OUT_OF_SCOPE:
            print(f"note: {p.name} is outside the default scope (quoted-token style) — audited on request, read its findings with that in mind")
        a, l, f = audit_text(p.read_text(encoding="utf-8"), str(p.relative_to(ROOT)) if p.is_relative_to(ROOT) else str(p))
        total_alts += a; total_la += l; findings += f
    for line in findings:
        print("  ✗", line)
    print(f"LOOKAHEAD-SLOT-AUDIT: grammars={len(paths)} alternatives={total_alts} with_lookaheads={total_la} findings={len(findings)}")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
