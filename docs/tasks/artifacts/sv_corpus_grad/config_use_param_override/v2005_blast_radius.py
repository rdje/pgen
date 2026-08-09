#!/usr/bin/env python3
"""`SV-CORPUS-GRAD.3.20` — how many `verilog_2005`-lane corpus rows can this tightening move?

⛔ A TIGHTENING THAT MOVES NOTHING AND A FIX THAT SILENTLY DID NOTHING LOOK IDENTICAL in a
corpus census. So the blast radius is derived BEFORE the edit, from the corpus text and the
grammar's own reachability, and the corpus run afterwards is a CONFIRMATION of a falsifiable
prediction rather than the source of the claim.

The argument is structural, not statistical:

  1. `use_clause` is referenced from exactly two places in `grammars/systemverilog.ebnf`, both
     inside `config_rule_statement` (`inst_clause use_clause semi` / `cell_clause use_clause semi`).
  2. `config_rule_statement` is referenced from exactly one place: `config_declaration`, which
     begins with the `config` keyword.
  ⇒ a file with no `config` token anywhere cannot reach `use_clause` at all, whatever it contains.

Both premises are RE-DERIVED from the grammar text here rather than quoted, so the script fails
loudly if a future edit adds another reference path (`[[feedback_instrument_needs_ground_truth]]`).
The token scan then reports how many lane files contain `config`, and therefore how many rows this
change can possibly touch.

Usage (paths repo-root-relative, directive 12):
  python3 docs/tasks/artifacts/sv_corpus_grad/config_use_param_override/v2005_blast_radius.py
"""

import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from _repo_root import repo_root  # noqa: E402

ROOT = repo_root()
GRAMMAR = ROOT / "grammars/systemverilog.ebnf"
LANE = ROOT / "stimuli/sv/characterization/v2005_lane_files.tsv"

# A bare identifier occurrence, so `use_clause_param_override_sv_only` never matches `use_clause`.
def word(name: str) -> re.Pattern:
    return re.compile(rf"(?<![A-Za-z0-9_$]){re.escape(name)}(?![A-Za-z0-9_$])")


def referencing_rules(text: str, target: str) -> set:
    """Rules whose BODY references `target` (a rule body runs to the next column-0 definition)."""
    out, current = set(), None
    define = re.compile(r"^([A-Za-z_][A-Za-z0-9_]*)\s*:=")
    ref = word(target)
    for line in text.splitlines():
        m = define.match(line)
        if m:
            current = m.group(1)
            rest = line[m.end():]
        else:
            rest = line
        if current and current != target and not rest.lstrip().startswith("#") and ref.search(rest):
            out.add(current)
    return out


def ground_truth_controls(text: str) -> None:
    """Prove the reachability derivation can still SEE a reference before trusting it.

    ⛔ This is not ceremony. While this script was being written its third premise check
    REFUSED on the live grammar for the wrong reason — it looked for a rule literally named
    `kw_config`, and the grammar spells it `kw_config_dfba7aad`. A guard that fires for a
    bookkeeping reason is one edit away from a guard that never fires at all, so the two
    directions are pinned here and any miss aborts before a number is published
    ([[feedback_instrument_needs_ground_truth]]).
    """
    positive = referencing_rules(text, "use_clause")
    if "config_rule_statement" not in positive:
        raise SystemExit("REFUSE (control): the POSITIVE control failed — the derivation cannot "
                         "see `config_rule_statement`'s reference to `use_clause`, so its empty "
                         "answers would be meaningless.")
    # NEGATIVE: a name that occurs nowhere must produce no referencing rules.
    if referencing_rules(text, "pgen_control_rule_that_does_not_exist"):
        raise SystemExit("REFUSE (control): the NEGATIVE control failed — the derivation reports "
                         "references to a rule name that does not occur in the grammar.")
    # PLANTED: inject one reference from an unrelated rule and require the derivation to catch it.
    planted = text.replace("always_construct := always_keyword statement",
                           "always_construct := always_keyword statement use_clause", 1)
    if planted == text:
        raise SystemExit("REFUSE (control): the planted-mutation anchor `always_construct` is gone; "
                         "re-anchor the control rather than dropping it.")
    if "always_construct" not in referencing_rules(planted, "use_clause"):
        raise SystemExit("REFUSE (control): the PLANTED control failed — a deliberately injected "
                         "`use_clause` reference was not detected, so 'only reachable via config' "
                         "would be an unearned claim.")
    print("## ground-truth controls: positive OK, negative OK, planted-mutation OK")


def main() -> int:
    text = GRAMMAR.read_text(encoding="utf-8")
    ground_truth_controls(text)
    print()

    # --- leg 1: re-derive the reachability premise from the grammar, do not quote it -------------
    via_use = referencing_rules(text, "use_clause")
    via_rule_stmt = referencing_rules(text, "config_rule_statement")
    print("## reachability of `use_clause`, re-derived from the grammar")
    print(f"  rules referencing `use_clause`           : {sorted(via_use)}")
    print(f"  rules referencing `config_rule_statement`: {sorted(via_rule_stmt)}")
    if via_use != {"config_rule_statement"} or via_rule_stmt != {"config_declaration"}:
        print("REFUSE: the reachability premise no longer holds — `use_clause` is reachable from "
              "somewhere other than `config_declaration -> config_rule_statement`, so a "
              "`config`-token scan no longer bounds the blast radius.", file=sys.stderr)
        return 2
    # The keyword rules carry a hash suffix (`kw_config_dfba7aad`), so match the family prefix.
    opener = text.split("config_declaration :=", 1)[1].split("\n", 1)[0].split()
    if not opener or not re.fullmatch(r"kw_config(_[0-9a-f]+)?", opener[0]):
        print(f"REFUSE: `config_declaration` no longer opens with the `config` keyword "
              f"(first element is {opener[:1]}).", file=sys.stderr)
        return 2
    print("  => a file with no `config` token cannot reach `use_clause`. Premise holds.")
    print()

    # --- leg 2: scan the lane ---------------------------------------------------------------
    rows = [l.split("\t") for l in LANE.read_text(encoding="utf-8",
                                                  errors="replace").splitlines()[1:]]
    cfg, use, missing = [], [], 0
    cfg_re, use_re = word("config"), word("use")
    for _suite, _rel, repo_path in rows:
        p = ROOT / repo_path
        if not p.is_file():
            missing += 1
            continue
        t = p.read_text(encoding="utf-8", errors="replace")
        if cfg_re.search(t):
            cfg.append(repo_path)
        if use_re.search(t):
            use.append(repo_path)

    print("## verilog_2005 lane scan")
    print(f"  lane files                       : {len(rows)}  (unreadable: {missing})")
    print(f"  containing the token `use`       : {len(use)}")
    print(f"  containing the token `config`    : {len(cfg)}")
    for h in cfg[:40]:
        print(f"      config in {h}")
    print()
    print(f"=> MAXIMUM corpus rows this tightening can move in the verilog_2005 lane: {len(cfg)}")
    print("   (the `use` count is deliberately printed alongside: it is large and IRRELEVANT — "
          "`use` outside a `config` block is prose, a comment or an identifier.)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
