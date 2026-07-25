# LEX-ADJACENCY: a NO-LAYOUT lexical boundary that BOTH the parser and the generator honour

## Metadata

- Tree ID: `LEX-ADJACENCY`
- Status: `active` (opened 2026-07-26, session #207) — **design-first; no code
  until the design leaf `.1` is adjudicated.**
- Family / slice-id prefix: `PGEN-LEX-ADJACENCY-<NNNN>`
- Roadmap lane: cross-cutting parser capability — a MISSING PGEN PRIMITIVE,
  surfaced independently by TWO grammar families and measured on BOTH sides of
  the engine. Feeds the HORIZON goal ("every primitive duality-complete",
  [[project_horizon_universal_parser]]).
- Created: `2026-07-26`
- Owner: repo-local workflow
- Parser-AGNOSTIC ([[feedback_ast_pipeline_parser_agnostic]]) — the capability is
  a general EBNF/engine primitive, NOT a SystemVerilog special case.

## The gap, stated exactly

PGEN today has **two unconditional, opposite layout behaviours**:

| side | behaviour | measured |
|---|---|---|
| PARSE | layout is skipped before EVERY terminal | all **1,798** `match_regex` call sites in the generated SV parser pass `skip_leading_whitespace = true`; `grep -c "match_regex(.*false)"` = **0**. There is no per-token opt-out. |
| GENERATE | a separator is inserted between EVERY pair of sequence elements | probe `gen_probe.ebnf`: `num unit` emits `A 7904 ns`; the SAME structure with the unit tokens' `trivia` prefix REMOVED still emits `B 8918 s`. Removing `trivia` is not sufficient — the separator is independent of it. |

⇒ **A grammar cannot say "these two adjacent elements admit no layout between
them."** The only way to express lexical adjacency is to collapse the construct
into ONE terminal — which destroys the LRM's nonterminal structure and flattens
the AST (a regex terminal binds exactly one value; codegen emits
`ParseContent::Terminal(matched_str)` and there is no capture-group → `$N`
mapping).

## Why this is a real capability gap and not a grammar-authoring nit

Real language standards specify lexical adjacency **across a production
boundary**, routinely. IEEE 1800-2017 Annex A alone has four such footnotes, and
three of them are only satisfied today *because the construct happens to be
expressible as one token*:

| footnote | constraint | PGEN today |
|---|---|---|
| 33 | "Embedded spaces are illegal" (number sub-tokens) | one regex — OK by luck of shape |
| 44 | "The unsigned number or fixed-point number in `time_literal` shall not be followed by a white_space" | ⛔ **VIOLATED** — `time_literal := number time_unit` is two rules |
| 48 | "The apostrophe in `unbased_unsized_literal` shall not be followed by white_space" | one regex — OK |
| 50 | "The `$` in a `system_tf_identifier` shall not be followed by white_space" | one regex — OK |

Footnote 44 is the case where the LRM *also* names a sub-nonterminal
(`time_unit ::= s | ms | us | ns | ps | fs`), so it cannot be collapsed without
deleting an LRM production — which
[[feedback_no_rule_deletion_without_lrm_proof]] forbids outright.

**TWO independent families have now hit the same missing primitive:**

1. `rtl_frontend.ebnf:375` (2026, already in the record): *"A separate
   `!/[A-Za-z0-9_$]/` lookahead rule cannot work here: generated regex terminals
   always consume leading layout first … Closing that needs a precise no-layout
   lexical boundary assertion (a parse-enforced `[>!]` follow-restriction, or a
   no-layout regex) — a future parser-agnostic capability, out of scope here."*
   That entry saw the PARSE half only.
2. `SV-CORPUS-GRAD.3.11` (this session) — the GENERATE half, measured, plus a
   fully written and parser-verified fix that cannot land because of it.

## What `.3.11` proved about the cost of NOT having it

The `time_literal` defect is not cosmetic. Because `s`/`ms`/`us`/`ns`/`ps`/`fs`
are ordinary identifier spellings, the missing adjacency constraint makes PGEN
**reject legal real-world code** — `#1 ps[idx] = 1'b1;`, `#2 s = ~s;`,
`a ##1 s ##1 b` — and it owns **2 of the 382** tracked
`unexplained_rejects_valid` corpus rows. The strict fix measures at
**+5 corpus passes, 0 pass→fail, rejects-valid 382 → 380**. It is blocked purely
on this primitive.

## Leaves

### `.1` — Design: what the primitive IS (read-only + design doc)

- **Status: `todo`.** Adjudicate the surface AND both engine halves together —
  a parse-only or generate-only answer is what produced the current gap.
- **Candidate surfaces to price (not a menu to hand the director — pick one and
  justify it):**
  - a sequence-level adjacency operator (e.g. `a . b` / `a ~ b`) meaning "no
    layout between `a` and `b`";
  - a rule-level `@lexical` annotation meaning "no layout anywhere inside this
    rule's body", which reads closest to how an LRM states it (a time literal
    IS one lexical token) and is the smallest surface;
  - a per-terminal no-skip form (the `match_regex(..., false)` path already
    exists in codegen and is currently never emitted).
- **Both halves are mandatory, and the acceptance test is the duality:**
  1. PARSER — the boundary is enforced (interior layout rejects);
  2. GENERATOR — no separator is emitted across the boundary, so generated
     samples re-parse (`sample_parse_failures = 0`);
  3. the CERTIFICATE machinery still witnesses every sub-rule (the `@sample`
     route failed exactly here: pinning `time_literal` collapsed generator
     coverage to **rules 3/13, branches 0/6**).
- **⛔ EBNF-NATIVE, non-negotiable** — `scripts/check_doctrines.sh` rejects an
  out-of-band acceptance mechanism wired outside the EBNF
  ([[project_ebnf_is_single_source_of_truth]]).
- **Prior art in-repo to reuse, not re-derive:** the `@profiles` annotation is
  the proof that an EBNF-native, gate-verified, engine-wide switch is achievable;
  `.3.11`'s `generator_shape_probe.txt` is the ready-made A/B harness for the
  generate half.

### `.2` — Implement + gate

- **Status: `todo`**, blocked on `.1`. Engine tier, so it needs the design record
  first per the fix hierarchy (annotations > store > grammar > engine).
- Wire a gate that proves BOTH settings behave as declared, on a synthetic
  grammar (the `.1` probe) *and* on a real one.

### `.3` — Land the blocked consumers

- **Status: `todo`**, blocked on `.2`. Two known consumers, both already
  diagnosed, both with acceptance evidence already banked:
  - `SV-CORPUS-GRAD.3.11` — the grammar edit is written verbatim in that leaf and
    its before/after matrix + corpus lanes re-run unchanged.
  - `rtl_frontend` — the `keyword$…` identifier residual recorded at
    `grammars/rtl_frontend.ebnf:375`.
- Re-sweep IEEE 1800-2017 Annex A footnotes 33 / 48 / 50 afterwards: they are
  correct TODAY only because their constructs collapse into one token, so they
  should be re-expressed with the primitive if that is what the LRM actually
  means — a fidelity question to decide, not to assume.

## Acceptance Criteria (tree)

- The primitive is EBNF-native, parser-agnostic, and honoured by BOTH the parser
  and the stimuli generator, proven by a gate.
- `SV-CORPUS-GRAD.3.11` lands strictly with its banked evidence re-run green:
  38/38 matrix rows, 0 pass→fail, rejects-valid 382 → 380,
  `sample_parse_failures = 0`, `time_unit:5555` still defined AND reachable, and
  the `{value, unit}` AST object unchanged.
- No LRM production is deleted and no cert contract is re-baselined to absorb a
  duality break.

## Evidence

`docs/tasks/artifacts/sv_corpus_grad/time_literal_ws_diag/design_adjudication.txt`
(the four routes, each measured and each blocked) and
`.../generator_shape_probe.txt` (the three-shape generator A/B/C run).
