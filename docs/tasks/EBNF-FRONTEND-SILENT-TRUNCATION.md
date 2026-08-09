# EBNF-FRONTEND-SILENT-TRUNCATION: a column-0 `#` comment inside a rule body ENDS the rule and silently discards every following alternative

## Metadata

- Tree ID: `EBNF-FRONTEND-SILENT-TRUNCATION`
- Status: **`active`** (created 2026-08-09, session #219, routed in by
  `SV-CORPUS-GRAD.3.19` — which lost two `use_clause` alternatives to this and only
  noticed because its own repro matrix went RED in a way the fix could not explain)
- Roadmap lane: **cross-family engine/frontend integrity** — it defends
  [[project_ebnf_is_single_source_of_truth]] directly: if the frontend can drop part of a
  rule without saying so, the EBNF is not the source of truth, a silently smaller language
  is. Serves every family, not just SV.
- Created: `2026-08-09`
- Owner: repo-local workflow

## The defect, in one line

A `#` comment line **at column 0** appearing inside a rule body terminates the rule.
Every alternative after it is **silently discarded** — no error, no warning, no lint
finding, and no movement in any census the repo currently runs.

## MEASURED — the discriminating experiment (`SV-CORPUS-GRAD.3.19`, evidence banked at `docs/tasks/artifacts/sv_corpus_grad/config_use_param_override/frontend_truncation.txt`)

Five one-rule synthetic grammars, each dumped through
`ast_pipeline <g>.ebnf --generate-stimuli --count 1 --seed 0 --dump-gen-ast <out>.json`
and read at the IR level (`grammar_tree.scratch.Or.alternatives`). All five declare the
same three alternatives, so the expected answer is **3** every time:

| case | shape | IR alternatives | verdict |
|---|---|---|---|
| `C_baseline` | no comment at all | **3** | control |
| `D_indented_comment` | comment INDENTED, between alts 1 and 2 | **3** | fine — this is the house style |
| `B_hash_in_string` | `@probe_sample: "x #(y)"` — a `#` inside a STRING | **3** | fine — strings are respected |
| `A_comment_between_alts` | comment at **column 0**, between alts 1 and 2 | **1** (node degrades from `Or` to `Sequence`) | ⛔ 2 alternatives lost |
| `E_col0_comment_then_alts` | comment at **column 0**, between alts 2 and 3 | **2** | ⛔ 1 alternative lost |

⭐ The pair `D` vs `A` is what makes this a *finding* rather than a guess: the two files
differ **only in the leading whitespace of a comment line**, and one of them silently
deletes half the rule. And `B` rules out the tempting wrong root cause (that `#` is
mishandled generally — it is not; only the column-0 line position matters).

## ⛔ Why this is severe out of proportion to its size

1. **It deletes LANGUAGE, silently, from the single source of truth.** The lost
   alternatives are not flagged as unreachable, unused, or malformed. They simply stop
   existing.
2. **Every instrument the repo owns was blind to it.** On the real `use_clause` case,
   with two alternatives gone: `--lint-grammar` clean; `defined_rule_count` **1477,
   unchanged**; `--dump-rule-profiles` **byte-identical** across all 1 477 rules and all
   three profiles (1354/1373/1122); `sv_syntax_closure_gate`'s census unmoved. The only
   thing that caught it was a hand-written repro matrix whose *already-passing* control
   rows flipped to REJECT — i.e. it was caught by luck of good practice, not by a gate.
3. **The failure is silent in the safe direction only by accident.** Here it removed
   alternatives, so the parser under-accepted and a corpus row went red. A truncation that
   removed a *strictness* alternative (a `@predicate`-gated arm, a negative lookahead)
   would make the parser **over-accept** and nothing in the repo would go red at all.
4. **It is invisible in review.** A reviewer reading the grammar diff sees a correct
   alternative added; the file says one thing and the parser does another.

## MEASURED — is any live language missing today? **No.**

Two censuses run at diagnosis time, both over `grammars/**/*.ebnf`:

- **Textual:** column-0 comments sitting inside a rule body with a continuation line after
  them — **0** across every tracked grammar (once `.3.19`'s own edit was indented).
- **Structural (SV, the release-critical grammar):** source `|`-alternative count vs IR
  alternative count for all **10** rules carrying an interior comment — `source_text_item`
  5/5, `data_type` 15/15, `net_declaration_sv_2017` 4/4, `net_declaration_sv_2023` 4/4,
  `primary_hier_scope_prefix` 3/3, `scoped_or_hierarchical_tf_identifier` 2/2,
  `use_clause` 5/5, `variable_lvalue_scope` 3/3, `minus_minus` 1/1, `rparen` 1/1 —
  **0 alternatives lost**. Every pre-existing interior comment is INDENTED and intact; only
  `.3.19`'s own (since-fixed) column-0 block was truncating.
  Both censuses are the tracked, re-runnable
  `docs/tasks/artifacts/sv_corpus_grad/config_use_param_override/truncation_census.py`
  (output `truncation_census.txt`), which exits non-zero on any truncating site or any
  `source > IR` disagreement.

⇒ this tree is a **guardrail**, not a repair. Nothing shipped is currently wrong because
of it. That is exactly why it must be gated now rather than remembered.

## Leaves

### `.1` — the mechanism-agnostic ALTERNATIVE-COUNT gate (recommended first; cheap, family-neutral)

- **Status: `todo`** — a deterministic check that, for every rule in every tracked
  grammar, the count of alternatives the SOURCE declares equals the count the IR carries.
  Mechanism-agnostic by construction: it catches this defect and any future frontend
  drop, without needing to know how the drop happens. The SV half of it already ran by
  hand in `.3.19` (the table above), so the instrument is proven before it is written.
- **Honest design note:** "count the source alternatives" needs a real tokenizer, not a
  line regex — a `|` inside a string, a regex token, or a character class must not count.
  The clean implementation reuses the frontend's own tokenizer and compares two passes,
  rather than re-implementing lexing in a script.

### `.2` — fix the frontend so a column-0 comment does not terminate a rule body

- **Status: `todo`** — the actual repair in `pgen::ebnf_frontend`. ⛔ **Decide the
  intended language first, then implement**: is a rule body continued by *indentation*, or
  by the next line starting with `|`/`->`? The current behaviour is a third thing
  (indentation, with a comment line at column 0 acting as a terminator), which is what
  makes it surprising. Whatever is chosen must be written down as a rule of the EBNF
  dialect, not left implicit in the parser.
- Regression proof: the five synthetic cases above become a pinned test, `A`/`E` flipping
  from truncated to complete.

### `.3` — say it out loud in the EBNF dialect documentation

- **Status: `todo`** — the book's EBNF-dialect chapter must state the comment/continuation
  rule explicitly, with the `D` vs `A` pair as the worked example. Until `.2` lands, that
  documentation is the only thing standing between a grammar author and a silent language
  loss, so this leaf is worth landing even before the fix.

## Acceptance Criteria (tree)

1. A tracked, deterministic gate fails when a grammar's source alternatives and its IR
   alternatives disagree, for any rule in any tracked grammar (`.1`).
2. The frontend no longer terminates a rule body at a column-0 comment, and the five
   synthetic cases are pinned as a regression test (`.2`).
3. The EBNF dialect's comment/continuation rule is documented in the book (`.3`).
4. Re-running the two censuses above finds zero truncating sites and zero source⟷IR
   alternative-count disagreements.
