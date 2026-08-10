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

### `.4` — an inline `->` return annotation inside a parenthesized GROUP is silently re-read as that element's QUANTIFIER (routed in by `SV-CORPUS-GRAD.3.25`, 2026-08-10)

- **Status: `todo`.** A **second, independent** frontend mis-read, found the same way the
  column-0 one was — by an author writing a legal-looking construct and the IR disagreeing.
  Same class (*the frontend reads something other than what the file says*), different
  mechanism, so it is a sibling leaf rather than part of `.2`.

- ⭐⭐ **UPGRADED 2026-08-10 (director challenge: *"an inline return annotation inside a
  parenthesized group?"*) — THE CONSTRUCT IS LEGAL BY THE PROJECT'S OWN SPEC, so this is a
  frontend ⟷ meta-grammar DIVERGENCE, not an author error.** The first write-up left that open;
  the question forced the check, and the check inverted the reading. Two independent confirmations:

  **(i) The meta-grammar declares it.** `grammars/ebnf.ebnf` derives the construct in three hops:
  ```ebnf
  grouped_expression := "(" rule_expression ")"          # :316
  rule_expression    := alternation                      # :124
  alternation        := sequence (return_annotation? "|" sequence)*   # :134
  ```
  A group contains a full `alternation`, and `alternation` admits a **per-branch `->` before each
  `|`** (added deliberately — the comment at `:127`–`:133` says so, mirroring
  `ebnf_frontend.rs`'s own inline-at-`->` placement). ⇒ `( b -> {…} | c )` is spec-legal.

  **(ii) The parser GENERATED from that meta-grammar reads it correctly** — measured, not
  inferred, via the 1.9 envelope driver
  (`ebnf_dual_run_diff --input <synthetic> --emit-ast-json arm2.json`):
  ```json
  {"type":"grouped","expression":{"type":"alternation","alternatives":[
     {"type":"sequence","elements":[{"type":"non_terminal","name":{"name":"b"}}]},
     [{"type":"return_annotation","expression":{"type":"object_return", …"bee"…}},"|",
      {"type":"sequence","elements":[{"type":"non_terminal","name":{"name":"c"}}]}]]}}
  ```
  **`quantified` node count: 0.** Arm 2 builds a `return_annotation`; arm 1 (the hand-written
  `pgen::ebnf_frontend`, which is what actually reads every grammar today) builds
  `Quantified{element: b, quantifier: "kind: \"bee\""}`.

- ⛔ **Honest bound on legality — the LAST branch of a group is a separate question.**
  `alternation` places `return_annotation?` **before** the `|`, and neither `grouped_expression`
  nor `rule_expression` carries a trailing annotation slot; only `rule_definition` (`:104`) does.
  So inside a group, a `->` on a **non-last** branch is derivable and a `->` on the **last** branch
  is not. The synthetic above deliberately exercises the *non-last* (legal) position, so the
  divergence is proven on ground the spec unambiguously grants. The SV edit that surfaced this
  used both positions, which is why it is not itself the clean witness.

- ⚠️ **Why the existing `.1.9` envelope differential did not already catch it.** `ebnf` is one of
  the 6 ENVELOPE-EQUIVALENT grammars (913/913 token positions). That number is over the **shipped**
  grammars, and **not one of them uses a per-branch `->` inside a group** — so the differential is
  green and blind simultaneously. The instrument is sound; its corpus does not reach here. ⇒ the
  regression proof for this leaf must be a **synthetic** case added to the differential's inputs,
  not a re-run over `grammars/`.
- **MEASURED — reproduced on a 5-rule synthetic, so it is FAMILY-NEUTRAL, not an SV artifact**
  (the `ROUTING-EVIDENCE` question, answered before routing). Grammar:

  ```ebnf
  @entry: true
  top := a ( b -> {kind: "bee"} | c ) d  -> {kind: "top"}
  ```
  `ast_pipeline <g>.ebnf --generate-parser --dump-gen-ast … --dump-gen-ast-pretty` yields

  ```json
  {"Or": {"alternatives": [
     {"Quantified": {"element": {"Atom": … "b"}, "quantifier": "kind: \"bee\""}},
     {"Atom": … "c"}]}}
  ```
  ⇒ the annotation payload `{kind: "bee"}` was consumed as the **quantifier string** of `b`.
  Confirmed identically inside `grammars/systemverilog.ebnf` (`quantifier: "kind: \"plus\""`).
- ⭐ **HONEST SEVERITY — this one fails CLOSED, and that distinction is the point.** Codegen
  then aborts with `Error: Failed to generate parser using AST-based generator / Unknown
  quantifier: kind: "plus"` (measured on BOTH the synthetic and the full SV grammar, exit 1).
  So — unlike `.1`/`.2` — **no wrong parser can ship from this**. The defect is diagnostic
  quality, not correctness:
  1. the error names a **quantifier the author never wrote**, at no source location, so it
     reads as an internal generator failure rather than "your annotation is in the wrong place";
  2. **`--lint-grammar` passes the grammar** — it reports only an oblique
     `always_succeeds_alternatives` NOTE on an unrelated-looking node path
     (`root/s1/q/s0`), which is the tool an author would reach for first;
  3. it is therefore a **latent trap for BOUNDED-QUANT-class work** — TOOLBOX §1.7 records the
     same `Unknown quantifier` message arising from a *real* quantifier-decoding half-wire, so
     the two causes are indistinguishable from the message alone.
- **The decision is now narrower than the first write-up assumed.** *Is it legal?* is answered for
  the non-last branch — the meta-grammar derives it and the generated meta-parser builds it — so
  the frontend must **parse it as an annotation** there, not reject it. Two questions remain:
  1. **The last branch inside a group** (`( a -> {…} | b -> {…} )`) is currently underivable. Do
     we extend `grouped_expression` with a trailing `return_annotation?`, or is the trailing
     position deliberately reserved to `rule_definition`? Whichever is chosen, write it down as a
     rule of the dialect (`.3`) — the current state is implicit in two disagreeing parsers.
  2. **Which frontend is the spec?** `EBNF-SOURCE-OF-TRUTH` says the EBNF is the single source of
     truth; here the *meta-grammar* and the *hand-written reader of it* disagree, and the
     hand-written one wins by default because it is what runs. That is the general defect behind
     this leaf, and it is bigger than one construct.
  ⛔ What must not continue in any case is silently reinterpreting the construct as a **different
  syntactic category** (a quantifier) rather than parsing or refusing it.
- **Workaround until fixed** (used by `SV-CORPUS-GRAD.3.25`, and worth documenting in `.3`):
  lift the group to a **named rule** and annotate its alternatives there.

## Acceptance Criteria (tree)

1. A tracked, deterministic gate fails when a grammar's source alternatives and its IR
   alternatives disagree, for any rule in any tracked grammar (`.1`).
2. The frontend no longer terminates a rule body at a column-0 comment, and the five
   synthetic cases are pinned as a regression test (`.2`).
3. The EBNF dialect's comment/continuation rule is documented in the book (`.3`).
4. Re-running the two censuses above finds zero truncating sites and zero source⟷IR
   alternative-count disagreements.
5. An inline `->` annotation inside a parenthesized group either parses as an annotation or
   is rejected with a located, self-explaining error — never re-read as a quantifier (`.4`).
