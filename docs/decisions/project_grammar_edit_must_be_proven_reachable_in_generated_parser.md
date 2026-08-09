---
name: project-grammar-edit-must-be-proven-reachable-in-generated-parser
description: "A grammar edit is not landed when the .ebnf reads right and regeneration succeeds — it is landed when the new production has a CALLER in the generated parser. A column-0 `#` comment inside an alternation list terminates the rule and silently DROPS every `|` arm below it: no error, no warning, and the generated parser simply carries fewer alternatives. Verify the arm count, not the diff."
metadata:
  node_type: memory
  type: project
id: project-grammar-edit-must-be-proven-reachable-in-generated-parser
title: A grammar arm is landed only when it has a CALLER in the generated parser — a column-0 comment inside an alternation silently drops it
date: 2026-08-09
answers:
  - "I added an alternative to a grammar rule, regenerated, and nothing changed — why"
  - "my new EBNF rule is in the generated parser but never fires"
  - "can I put a comment between the alternatives of a rule"
  - "how do I prove a grammar edit actually reached the generated parser"
  - "why did the parser behave identically after I edited the .ebnf"
reverify: |
  # Every arm-above comment in every grammar must be INDENTED, never at column 0.
  python3 - <<'PY'
  import pathlib
  bad=[]
  for g in sorted(pathlib.Path("grammars").glob("*.ebnf")):
      L=g.read_text().splitlines()
      for i,l in enumerate(L):
          if l.lstrip().startswith("|"):
              j=i-1
              while j>=0 and not L[j].strip(): j-=1
              if j>=0 and L[j].startswith("#"): bad.append((str(g), i+1))
  print("column-0 comment directly above a | arm:", len(bad), bad)
  PY
---

**Measured 2026-08-09 (`SV-CORPUS-GRAD.3.14b`, `PGEN-SV-CORPUS-GRAD-0033`).** A new
alternative was added to `non_port_module_item` and `class_item` in
`grammars/systemverilog.ebnf`, the parser was regenerated, the release probe was rebuilt —
and the measured behaviour was **identical to before**, on all 8 target files. No error, no
warning, exit code 0 at every stage.

The rule itself was in `generated/systemverilog_parser.rs` (65 references) and reachable
only through the dynamic `parse_rule_by_name` dispatch table. It had **no caller**.
`cascade_match_non_port_module_item` carried **8 alternatives, not 9**.

**Root cause: a `#` comment at COLUMN 0 inside an alternation list TERMINATES the rule, and
every `|` arm below it is dropped from the generated parser with no diagnostic.** The
explanatory comment had been written between the arms.

Discriminating pair, measured in both directions:

| form | site | arm in generated parser |
|---|---|---|
| comment at **column 0** between arms | the failed first attempt | **DROPPED** — no caller emitted |
| comment **indented** to the continuation column | `net_declaration_sv_2017` in the same grammar | **LIVE** — `checked_nettype_identifier`, `wildcard_escape_nettype_identifier`, `interconnect_net_declaration_sv_only` all present |

**Why:** the failure is silent *and* it fails in the ACCEPTING direction. A dropped
alternative only narrows the language the parser accepts, so no pass-rate, no corpus delta,
no AST-shape gate and no test that exercises other constructs can reveal it. The only symptom
is "my change did nothing" — which reads like a wrong hypothesis about the grammar, and sends
you back to re-read the EBNF you already wrote correctly. It cost a full regenerate +
release-rebuild cycle before the toolbox pinned it.

**How to apply:**

1. **Keep commentary ABOVE the rule, never between its arms.** If a specific arm needs a
   note, indent the comment to the arm's continuation column — that form is absorbed and is
   already used throughout `systemverilog.ebnf`.
2. **After any grammar edit, prove REACHABILITY before measuring behaviour.** Grep the
   generated parser for a *caller* of the new rule, not merely for its name:
   `grep -c "parse_<new_rule>()" generated/<family>_parser.rs` — the rule's own definition
   and the dispatch table always mention it, so a nonzero name count proves nothing.
   Better: count the host rule's alternatives in `cascade_match_<host>` and check the count
   moved.
3. **"Regeneration succeeded" is not "the edit landed."** Treat a no-op measurement after a
   grammar change as a REACHABILITY question first and a semantics question second — the
   toolbox answers it in one grep, while re-reading the EBNF cannot answer it at all
   ([[feedback-why-and-where-before-solution]]).
4. Repo-wide audit at the time of writing: 17 grammars, 8 comment-above-arm sites, **all 8
   indented and verified live, 0 column-0 occurrences** — this is a latent hazard, not
   current damage. Whether column-0 termination is the INTENDED frontend semantics, and
   whether it should emit a diagnostic, is tracked as `SV-CORPUS-GRAD.3.14c`.
