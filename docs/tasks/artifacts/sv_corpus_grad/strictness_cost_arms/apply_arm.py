#!/usr/bin/env python3
"""SV-CORPUS-GRAD.13c.2k (a)+(b) — build one experimental ARM of `grammars/systemverilog.ebnf`.

⛔ WHY A SCRIPT AND NOT A PATCH FILE. The rise `.13c.2k` measured is CONFLATED across two grammar
changes, and the leaf's remaining work is a set of COMPARISONS between spellings. A comparison
whose arms were hand-edited is a comparison nobody can re-run — and `.13c.2k`'s own headline
number (`43` raw references) was wrong precisely because it came from a hand count
(`raw_identifier_census/census.py` measured 47). So every arm here is DERIVED: design A's 45 call
sites come from the census's own signal, re-derived at apply time, and the script REFUSES if the
count it rewrites differs from the count the census reports.

THE ARMS
  t_only     `.13c.2t` alone — `assignment_pattern_key := simple_type | data_type | default`.
             Isolates the cost of the LRM-union widening from the identifier guard.
  designB    `t_only` + `.13c.2k` as BUILT and held back: the reserved-word exclusion moves INTO
             `identifier`'s simple branch and `non_keyword_identifier` becomes a bare alias.
             One edit, guards all 47 references including the negation site.
  designC    `t_only` + `.13c.2k` with `non_keyword_identifier` doing the work ITSELF instead of
             delegating to `identifier`, and the 45 call sites pointed at it. ⛔ Built because
             `designA` LANDED and `PARSE-COST-RATCHET` refused it on the OTHER binding counter:
             `entries` fell 0.89 % but `committed` ROSE +51,487, attributed to the unit as one
             extra COMMITTED FRAME per committed identifier at the 45 rewritten sites
             (`non_keyword_identifier` wrapping `identifier`). This arm removes that frame — the
             guarded rule matches the token directly, so the chain is the same LENGTH as HEAD's
             at the rewritten sites and one frame SHORTER at the ~100 already-guarded ones.
  designA    `t_only` + `.13c.2k` spelled at the CALL SITES: `identifier` and
             `non_keyword_identifier` keep their HEAD definitions and the 45 unguarded
             non-negation references are rewritten to `non_keyword_identifier`.
             ⛔ This arm exists because `PARSE-COST-RATCHET`'s "record it as irreducible" outcome
             requires a comparison, and `.13c.2k` could only offer a reasoned one — the exact
             error `-0231`/`-0234` were opened to correct.

⭐ THE TWO DESIGNS ARE NOT ACCEPT-SET EQUAL, AND THE DIFFERENCE IS EXACTLY ONE SITE.
`rooted_tf_call_sv_only`'s `!( identifier )` firewall is the census's sole `negation` member:
guarding `identifier` (design B) makes that lookahead succeed on keyword-headed input, i.e. WIDENS
there, while design A leaves it untouched. That is a measurable difference, not a footnote, and it
is what the correctness arm of this comparison has to settle.

USAGE   python3 …/apply_arm.py --arm {t_only,designB,designA,designC} [--check]
        python3 …/apply_arm.py --restore          # git-checkout the grammar back to HEAD
EXIT    0 = applied (or --check passed) · 2 = refused (anchor text not found, count mismatch)
"""

from __future__ import annotations

import argparse
import importlib.util
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
if not (ROOT / "grammars").is_dir():
    raise SystemExit(f"apply_arm: not at the repo root (derived {ROOT}) — fix the parents[] depth")
GRAMMAR = ROOT / "grammars/systemverilog.ebnf"
CENSUS = ROOT / "docs/tasks/artifacts/sv_corpus_grad/raw_identifier_census/census.py"

TARGET = "identifier"
GUARD = "non_keyword_identifier"

# ── `.13c.2t` — the LRM union. Anchored on the exact HEAD text so a moved grammar refuses. ──────
T_BEFORE = (
    'assignment_pattern_key := simple_type        -> {kind: "type",     body: $1}\n'
    '                        | kw_default_7505d64a -> {kind: "default"}\n'
)
T_AFTER = (
    '# SV-CORPUS-GRAD.13c.2t (EXPERIMENTAL ARM — see strictness_cost_arms/apply_arm.py): the UNION\n'
    '# of the standard\'s two normative statements about this key, which disagree identically in\n'
    '# 1800-2017 and 1800-2023. Annex A gives `simple_type | default`; §10.9.2 clause text gives\n'
    '# "\'{data_type: default_value}" and the worked example `\'{int:1, default:0, string:""}`.\n'
    '# The sets are NOT nested — `ps_parameter_identifier` is in `simple_type` only, `string` /\n'
    '# `chandle` / `event` / `class_type` / `type_reference` in `data_type` only — so neither alone\n'
    '# is compliant. `simple_type` stays FIRST so every construct that parses today keeps its arm.\n'
    'assignment_pattern_key := simple_type        -> {kind: "type",      body: $1}\n'
    '                        | data_type          -> {kind: "data_type", body: $1}\n'
    '                        | kw_default_7505d64a -> {kind: "default"}\n'
)

# ── `.13c.2k` design C — `non_keyword_identifier` matches the token itself. ─────────────────────
C_GUARD_AFTER = (
    '# SV-CORPUS-GRAD.13c.2k design C (EXPERIMENTAL ARM): the guarded rule does the work ITSELF\n'
    '# rather than delegating to `identifier`, so guarding a position costs no extra parse frame.\n'
    'non_keyword_identifier := escaped_identifier -> {body: $1}\n'
    '                       | !reserved_non_keyword_identifier simple_identifier -> {body: $2}\n'
)

# ── `.13c.2k` design B — the exclusion moves into `identifier`. ─────────────────────────────────
B_BEFORE = (
    'identifier := escaped_identifier -> {body: $1}\n'
    '           | simple_identifier  -> {body: $1}\n'
)
B_AFTER = (
    '# SV-CORPUS-GRAD.13c.2k design B (EXPERIMENTAL ARM): IEEE 1800-2017 §5.6.2 forbids a reserved\n'
    '# word from being a `simple_identifier`, and A.9.3 writes `identifier ::= simple_identifier |\n'
    '# escaped_identifier` — so the exclusion is a property of `identifier` itself, not of the 45\n'
    '# places that reference it. `escaped_identifier` is deliberately NOT guarded: §5.6.1 makes\n'
    '# `\\module ` a legal identifier distinct from the keyword.\n'
    'identifier := escaped_identifier -> {body: $1}\n'
    '           | !reserved_non_keyword_identifier simple_identifier -> {body: $2}\n'
)
B_GUARD_BEFORE = (
    'non_keyword_identifier := !reserved_non_keyword_identifier identifier\n'
    '                       -> {body: $2.body}\n'
)
B_GUARD_AFTER = (
    '# SV-CORPUS-GRAD.13c.2k design B (EXPERIMENTAL ARM): now an ALIAS — the prefix moved into\n'
    '# `identifier` above. The returned shape (`{body: X}`) is unchanged.\n'
    'non_keyword_identifier := identifier\n'
    '                       -> {body: $1.body}\n'
)


def load_census():
    spec = importlib.util.spec_from_file_location("raw_identifier_census", CENSUS)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def replace_once(text: str, before: str, after: str, what: str) -> str:
    n = text.count(before)
    if n != 1:
        raise SystemExit(f"apply_arm: anchor for {what} occurs {n} times, expected 1 — refusing")
    return text.replace(before, after)


def rewrite_call_sites(text: str, census) -> tuple[str, int]:
    """Rewrite every non-negation, non-guard reference to `identifier` -> `non_keyword_identifier`.

    Derived, not listed: the census's own per-rule classification decides which rules are in
    scope, and the count is asserted against it.
    """
    kinds, _sites = census.signal_b()
    in_scope = {r for r, ks in kinds.items() if any(k in ("alias", "inline") for k in ks)}
    expected = sum(1 for ks in kinds.values() for k in ks if k in ("alias", "inline"))

    word = census.WORD
    out_lines: list[str] = []
    current = None
    rewrites = 0
    for raw in text.splitlines(keepends=True):
        line = raw.rstrip("\n")
        stripped = line.lstrip()
        if stripped.startswith("#") or stripped.startswith("//"):
            out_lines.append(raw)
            continue
        head = census.RULE_HEAD.match(line)
        if head:
            current = head.group(1)
        if current not in in_scope:
            out_lines.append(raw)
            continue
        # Only the region BEFORE a return directive can hold a rule reference; `-> {kind:
        # "identifier"}` on the same line must not be touched.
        cut = line.find("->")
        head_part, tail_part = (line[:cut], line[cut:]) if cut >= 0 else (line, "")
        if "#" in head_part or "//" in head_part:
            raise SystemExit(f"apply_arm: unexpected comment inside a rewritten body: {line!r}")
        new_head, n = word.subn(GUARD, head_part)
        rewrites += n
        out_lines.append(new_head + tail_part + ("\n" if raw.endswith("\n") else ""))

    if rewrites != expected:
        raise SystemExit(f"apply_arm: rewrote {rewrites} references, census says {expected} — refusing")
    return "".join(out_lines), rewrites


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--arm", choices=("t_only", "designB", "designA", "designC"))
    ap.add_argument("--restore", action="store_true")
    ap.add_argument("--check", action="store_true", help="apply in memory and report, write nothing")
    a = ap.parse_args()

    if a.restore:
        subprocess.run(["git", "checkout", "--", "grammars/systemverilog.ebnf"], cwd=ROOT, check=True)
        print("apply_arm: grammar restored to HEAD")
        return 0
    if not a.arm:
        ap.print_help()
        return 2

    dirty = subprocess.run(["git", "diff", "--quiet", "--", "grammars/systemverilog.ebnf"],
                           cwd=ROOT).returncode
    if dirty:
        raise SystemExit("apply_arm: the grammar is already modified — --restore first, refusing "
                         "to stack arms (a stacked arm silently measures two changes as one)")

    text = GRAMMAR.read_text(encoding="utf-8")
    text = replace_once(text, T_BEFORE, T_AFTER, ".13c.2t")
    note = ".13c.2t only"
    if a.arm == "designB":
        text = replace_once(text, B_BEFORE, B_AFTER, ".13c.2k design B (identifier)")
        text = replace_once(text, B_GUARD_BEFORE, B_GUARD_AFTER, ".13c.2k design B (alias)")
        note = ".13c.2t + .13c.2k design B (guard inside `identifier`)"
    elif a.arm == "designA":
        text, n = rewrite_call_sites(text, load_census())
        note = f".13c.2t + .13c.2k design A ({n} call sites rewritten)"
    elif a.arm == "designC":
        text, n = rewrite_call_sites(text, load_census())
        text = replace_once(text, B_GUARD_BEFORE, C_GUARD_AFTER, ".13c.2k design C (self-matching)")
        note = f".13c.2t + .13c.2k design C ({n} call sites rewritten, guard self-matching)"

    if a.check:
        print(f"apply_arm: {a.arm} would apply cleanly — {note} (nothing written)")
        return 0
    GRAMMAR.write_text(text, encoding="utf-8")
    print(f"apply_arm: arm={a.arm} APPLIED — {note}")
    print("  ⛔ regenerate the parser and rebuild the probe before measuring; the grammar alone "
          "changes nothing an instrument can see")
    return 0


if __name__ == "__main__":
    sys.exit(main())
