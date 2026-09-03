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

### ✅✅ `.1` — **`done`** 2026-09-03 (`PGEN-EBNF-FRONTEND-SILENT-TRUNCATION-0004`; doctrine **36**, `EBNF-ALTERNATIVE-PARITY`) — the mechanism-agnostic ALTERNATIVE-COUNT gate (recommended first; cheap, family-neutral)

- **Status: was `todo`; DONE — see the discharge below.** Original text: a deterministic check that, for every rule in every tracked
  grammar, the count of alternatives the SOURCE declares equals the count the IR carries.
  Mechanism-agnostic by construction: it catches this defect and any future frontend
  drop, without needing to know how the drop happens. The SV half of it already ran by
  hand in `.3.19` (the table above), so the instrument is proven before it is written.
- **Honest design note:** "count the source alternatives" needs a real tokenizer, not a
  line regex — a `|` inside a string, a regex token, or a character class must not count.
  The clean implementation reuses the frontend's own tokenizer and compares two passes,
  rather than re-implementing lexing in a script.

### ✅✅ `.2` — **`done`** 2026-09-03 (`PGEN-EBNF-FRONTEND-SILENT-TRUNCATION-0005`) — fix the frontend so a column-0 comment does not terminate a rule body

- ⭐ **THE LANGUAGE IS ALREADY DECIDED — by the meta-grammar (2026-09-03, `.1`'s gate measured it).** Through
  `EBNF-ALTERNATIVE-PARITY`, `a := b\n# a column-0 comment\n   | c` reads **source 2 vs IR 1**: the parser
  generated from `grammars/ebnf.ebnf` continues the rule past the comment line and takes `| c` as the second
  alternative; the hand-written frontend ends the body. The same principle `.6` applied to `//` applies here —
  `EBNF-SOURCE-OF-TRUTH`: implement what `ebnf.ebnf` says, do not choose. The bank keeps this shape as a LIVE
  RED (`alternative_parity/probe.sh` arm 4); when `.2` lands that arm flips and must be rewritten as a GREEN.
- **Status: `todo`** — the actual repair in `pgen::ebnf_frontend`. ⛔ ~~Decide the
  intended language first, then implement~~ (decided above): is a rule body continued by *indentation*, or
  by the next line starting with `|`/`->`? The current behaviour is a third thing
  (indentation, with a comment line at column 0 acting as a terminator), which is what
  makes it surprising. Whatever is chosen must be written down as a rule of the EBNF
  dialect, not left implicit in the parser.
- Regression proof: the five synthetic cases above become a pinned test, `A`/`E` flipping
  from truncated to complete.

### ✅✅ `.3` — **`done`** 2026-09-03 (`PGEN-EBNF-FRONTEND-SILENT-TRUNCATION-0005`, with `.2`) — say it out loud in the EBNF dialect documentation

- **Status: was `todo`; DONE** — `docs/book/src/grammar-wellformedness.md` (the meta-grammar comment section) states the rule — *what ends a rule body is what the meta-grammar says ends it: the next rule header, `@` annotation, `[>` directive or include directive; a blank line or a comment line at any column is trivia* — with the `D` vs `A` pair as the worked example. Original text: the book's EBNF-dialect chapter must state the comment/continuation
  rule explicitly, with the `D` vs `A` pair as the worked example. Until `.2` lands, that
  documentation is the only thing standing between a grammar author and a silent language
  loss, so this leaf is worth landing even before the fix.

### ✅ `.4` — **`done`** 2026-09-03 (`PGEN-EBNF-FRONTEND-SILENT-TRUNCATION-0006`: REFUSED at the byte, never misread; implementing the construct is `.4b`) — an inline `->` return annotation inside a parenthesized GROUP is silently re-read as that element's QUANTIFIER (routed in by `SV-CORPUS-GRAD.3.25`, 2026-08-10)

- **Status: was `todo`; DONE as a refusal — see the discharge below.** Original text: A **second, independent** frontend mis-read, found the same way the
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

#### PRIOR ART (searched 2026-08-10, on the director's question *"should we support inline return annotations? would it increase expressiveness?"*)

Searched `grammars/ebnf.ebnf`, `docs/decisions/`, `docs/tasks/`, `docs/book/`. Three neighbours,
none of which owns this case:

| owner | direction | this case? |
|---|---|---|
| `LANG-CAPABILITY-AUDIT.1` — 27 unreachable `ebnf.ebnf` productions | **declared but unreachable + unwired** (`parametric_rule`, `error_production`, …) | ✘ — `alternation`'s `return_annotation?` is *reachable*, one of the 104 |
| `ANNOTATION-PLACEMENT.4` — meta-grammar completeness | **shipped but undeclared** (lexical `[> … ]`, used 12× by svpp) | ✘ — the inverse; `.4`'s own closing bullet forbids merging the two |
| `ANNOTATION-PLACEMENT` directive (2026-07-26) | placement universality + **no silent drop**, for *semantic* annotations | ~ — the *principle* applies verbatim, the directive's scope does not |

⇒ **a THIRD fidelity direction, and the sharpest of the three: DECLARED, REACHABLE, honoured by
the generated meta-parser — and mis-read by the hand-written frontend that actually runs.** The
other two have one healthy half; here both halves of the self-hosting story look healthy and the
production reader still disagrees. Worth naming as its own class.

#### ⭐ THE ANSWER TO *"would supporting it increase expressiveness?"* — **NO. Measured, not argued.**

A branch whose whole body is one rule reference and which carries no annotation of its own is
**AST-transparent**: the referenced rule's value *is* the branch's value, and the emitted AST is
byte-identical across the lift ([[a-bare-rule-reference-alternative-is-ast-transparent]];
`SV-CORPUS-GRAD.3.20` proved it 15/15 cmp-identical). So `( A -> a | B -> b )` and
`g := A -> a | B -> b` + a reference to `g` denote the **same tree**. The lift is a total,
mechanical, shape-neutral transformation ⇒ inline group annotations are **pure sugar**; the set of
expressible ASTs does not grow by one shape.

**What it WOULD buy is census hygiene and frame cost, which are not nothing in this repo:**
- every lift is a real rule, and rules are **charged for** — `.3.25` paid +3 rules, a cert-census
  move (`expected_total` 1355→1358) and a contract re-baseline under the `CI-PARITY-GATE-ROT.22`
  tripwire, for what is semantically one annotation;
- each lifted rule costs a **frame + memo entry per occurrence** — precisely the wrapper class
  `RGX-0078.5.i.4`'s `INLINE-CENSUS` (`pass_through` / `alternation_leaf`) exists to collapse
  again. We add wrappers by hand and then spend engine work removing them.

**⛔ Recommendation: do NOT build it as a language feature.** Reasons, in order:
1. **Zero expressiveness gain** (above), against a documented house-pattern workaround.
2. **It would ship half a rule.** `alternation` places `return_annotation?` *before* the `|`, so
   the **last** branch of a group is underivable. Implementing only the derivable half gives
   authors a rule with an arbitrary-looking exception; making it symmetric means changing the
   meta-grammar too — a language change, not a bug fix.
3. **`@profiles` is rule-level**, so the most common reason to split a group — dialect gating —
   needs a named rule regardless. Inline annotations would not have saved `.3.25`'s edit.
4. **Wrong order of work.** This frontend has *two* known silent mis-reads (`.1` deletes
   alternatives; `.4` re-categorises an annotation). Adding surface to a reader with two open
   silent-corruption defects is backwards.

**⭐ Do the cheap half instead — make it AGREE or REFUSE.** The engine need not honour the cell,
but it must not silently reinterpret it as a different syntactic category. That is
`ANNOTATION-PLACEMENT`'s requirement 2 (the no-silent-drop diagnostic contract) applied to
*return* annotations, which is a bounded frontend change with a real safety payoff and **no new
language surface**. If the answer later becomes "support it", that decision belongs in
`ANNOTATION-PLACEMENT` as a placement-matrix cell, not here.

### `.4b` — implement per-branch return annotations INSIDE a group (opened 2026-09-03 by `.4`, `PGEN-EBNF-FRONTEND-SILENT-TRUNCATION-0006`) — `todo`, ROUTED to the generator lane

- **What `.4` left**: the construct `( b -> {…} | c )` is legal by the meta-grammar and REFUSED by the frontend with a
  located message; the lift into a named rule is AST-transparent but is charged for (a real rule: cert census,
  a frame + memo entry per occurrence — `.4`'s own accounting above). Implementing it means (i) an IR shape for a
  return annotation attached to an alternative of a NESTED alternation, (ii) `extract_rule_annotations` attaching by
  (group, branch) rather than top-level branch index, (iii) the generator emitting the per-branch builder inside
  the group's alternation, (iv) the refusal in `tokenize_rule_expression` becoming the parse. ⛔ Only when a grammar
  author needs it: today 0 tracked sites, and the director's 2026-08-10 question ("would it increase
  expressiveness?") is answered by the lift — no expressiveness is missing, only convenience.

### ✅✅ `.5` — **`done`** 2026-09-03 (`PGEN-EBNF-FRONTEND-SILENT-TRUNCATION-0007`) — the four GRAMMAR readers refuse a non-UTF-8 `.ebnf` file (routed in by `SV-CORPUS-GRAD.12c.1`, 2026-08-11)

- **Status: was `todo`, parked; DONE — see the discharge below.** Original text: PARKED on the standing SV lane lock — not on a technical blocker. It is
  routed here, and not merely attributed to a named owner, because
  `DOCTRINE-GAP-OWNERSHIP` is explicit that naming an owner is not routing.
- ⚠️ **Naming honesty, stated rather than glossed:** this tree is titled *silent* truncation, and
  this defect is **loud** — the read fails with an error. It is filed here because this tree is the
  repository's home for **EBNF frontend input-handling integrity**, and the sibling relationship is
  real: both are *the frontend not reading what the file says*. If the tree is ever renamed, this
  is one of the reasons.
- **THE DEFECT.** `SV-CORPUS-GRAD.12c.1` enumerated every text reader in `rust/src/` and
  categorised them (artifact:
  `docs/tasks/artifacts/sv_corpus_grad/source_text_readers/enumeration.md`). Four read
  **grammar** text and all four call `fs::read_to_string`, so a `.ebnf` file that is ISO-8859-1 —
  a `©` in a header comment is enough — is refused whole:
  `rust/src/ebnf_frontend.rs:16` (top-level), `rust/src/ebnf_frontend.rs:563` (`@include`d),
  `rust/src/bin/ebnf_dual_run_diff.rs:288`, `rust/src/parser_registry.rs:3805`.
- **WHY IT IS NOT URGENT, measured rather than assumed.** Every grammar tracked in this repo is
  UTF-8 (the same `.12c.1` census that found 13 non-UTF-8 files under `stimuli/sv/subs` found none
  under `grammars/`), so nothing in the repo fails today. It is a **user-facing** limitation: a
  third-party grammar authored on a Windows/Latin-1 toolchain is rejected for its copyright line.
- **THE FIX IS MECHANICAL AND ALREADY BUILT.** `pgen::source_text::read_source_file` landed with
  `.12c.1`; this is the same call swap four times, plus a fixture in the two non-UTF-8 encodings.
  ⛔ Do **not** widen it to the ~35 category-C readers (JSON manifests, generated Rust, reports):
  those are UTF-8 by construction and a decode failure there is corruption that must stay loud.
- **RE-OPEN / SCHEDULE WHEN** the SV lane lock lifts, or sooner if a user reports a rejected
  grammar.

### ✅✅ `.6` — **`done`** 2026-09-03 (`PGEN-EBNF-FRONTEND-SILENT-TRUNCATION-0002`; SV release `1.0.205`, ledger `SV-0078`) — was, opened 2026-09-01 by `LRM-GRAMMAR-FIDELITY.1e` / `PGEN-LRM-GRAMMAR-FIDELITY-0001`: **an INDENTED `#` comment on the line after a return annotation is captured INTO the annotation token: the token is misclassified `return_scalar`, and a comment-only edit there moves the `ebnf_raw_ast` digest that three doctrines key on**

**ROUTING EVIDENCE** (`ROUTING-EVIDENCE` doctrine — what was MEASURED before routing here):

- **REPRODUCES OUTSIDE THE FAMILY IT WAS FOUND IN — on a three-rule synthetic grammar.**
  `a := b -> {kind: "x"}\n     # a comment after the annotation\n   | c -> {kind: "y"}` read through
  `bash scripts/check_baseline_identity.sh --raw-ast` gives
  `['return_scalar', '{kind: "x"}\n     # a comment after the annotation']`; the same grammar
  without the comment line gives `['return_object', '{kind: "x"}']`. Nothing SystemVerilog about it.
- **MEASURED ON THE SHIPPED SV GRAMMAR**: **14** annotation tokens in **12** rules carry a `#`
  comment or a newline inside the payload and every one is classified `return_scalar`
  (`parseable_source_item`, `covergroup_declaration_sv_2023`, `block_data_type`, `data_type` ×2,
  `net_declaration_sv_2017` ×2, `net_declaration_sv_2023` ×2, `net_type`,
  `primary_hier_scope_prefix`, `scoped_or_hierarchical_tf_identifier`, `variable_lvalue_scope`,
  `less_than`). Census: the `raw_ast` tokens of kind `return_*` whose text contains `#` or `\n`.
- **THE CONSEQUENCE, MEASURED WITH A CONTROL**: editing one word of the comment after `net_type`'s
  `-> {kind: "trireg"}` in a copy of the shipped grammar moves `--digest ebnf_raw_ast` from
  `ea76b211c200…` to `3e36c001d84d…`; editing a COLUMN-0 comment elsewhere in the same file leaves
  it at `ea76b211c200…`. So `scripts/check_baseline_identity.sh:78-79` — *"comments never reach
  it"* — is false for this shape, and a comment-only edit at any of those 12 sites re-keys
  `BASELINE-IDENTITY`, `SV-CONTRACT-CURRENCY` tier A and `PARSE-COST-RATCHET`: the
  `SV-CORPUS-GRAD.13c.2x.4` cost with a mechanism attached.
- **WHY + WHERE**: `rust/src/ebnf_frontend.rs:713` `extract_inline_return_annotation_payload` —
  by its own doc comment the payload *"spans from the first non-whitespace character after `->`
  to the next top-level `|` … or to end-of-expression"*, and the rule body it scans is the
  multi-line body `collect_rule_body` assembles (`:664`), in which only a COLUMN-0 `#` terminates
  the rule. An indented comment between the closing `}` and the next `|` is therefore payload
  text. `classify_return_annotation` (`:1472`) then sees a string that starts with `{` and does
  not end with `}` → `return_scalar`. The regression test at `:1660-1701` pins the OTHER
  over-grab (subsequent `|` branches on the same line, *"the over-grab bug used to misclassify as
  return_scalar"*) and does not cover a following comment line.
- **DOES IT REACH THE GENERATED PARSER?** Not observably: the shape contract holds
  `{kind: "trireg"}` and every gate was green through `-0316`, so the code generator must re-read
  the payload up to the balanced `}`. The defect is in the ENVELOPE — the thing three doctrines
  hash — and in the envelope's classification.
- **OWES**: (a) end the inline payload at the balanced closing delimiter of an object / array
  payload and skip comment text before the next `|` (or strip `#`-to-end-of-line inside the
  payload scanner when not in a string / regex); (b) a regression arm on the three-rule grammar
  above — `return_object` with payload exactly `{kind: "x"}`; (c) re-derive the SV `ebnf_raw_ast`
  digest and re-stamp the three baselines ONCE (the digest WILL move — the 14 tokens change —
  and that is the fix landing, so the stamp must say so); (d) then the `.13c.2x.4` measurement:
  a comment-only edit after an annotation must leave the digest unmoved.
- ⚠️ **PARKED**: engine/frontend lane, not SV-blocking (the generated parser is right; the cost
  is one avoidable re-derivation per comment edit at 12 sites, and `SV-CORPUS-GRAD.13o` pays it once
  for the grammar-comment correction it carries).

### ✅✅ `.6` DISCHARGED — **both comment spellings are excluded from inline return-annotation payloads, `//` is a line comment everywhere the meta-grammar says it is, and the fix exposed three commented-out alternatives the frontend had kept LIVE in the shipped parser** (2026-09-03, `PGEN-EBNF-FRONTEND-SILENT-TRUNCATION-0002`; ENGINE tier — `rust/src/ebnf_frontend.rs`; grammar BYTES unchanged; ONE generated parser moves in ONE rule proven unreachable from the registered entry; SV release `1.0.205`, ledger `SV-0078`)

**RE-MEASURED FIRST.** On today's grammar the census still read **14** `return_*` tokens carrying a `#`/`//` comment
or a newline, every one classified `return_scalar`, in **11** rules (the leaf said 12: `data_type` and both
`net_declaration_sv_*` carry two tokens each — 14 over 11). The frozen list is
`docs/tasks/artifacts/ebnf_frontend_silent_truncation/comment_payload/before_census.json`. Two of the 14 tails
began with `//`, which the dialect's frontend did not know as a comment at all.

**THE SECOND HALF OF THE DEFECT, FOUND BY THE MEASUREMENT.** `grammars/ebnf.ebnf:583-586` reads
`line_comment := ("#" | "//") comment_content` — `//` is valid EBNF by the project's own meta-grammar
(`EBNF-SOURCE-OF-TRUTH`: the meta-grammar is the authority on what an author can write). `pgen::ebnf_frontend`
recognised only `#`. Census of the shipped SV grammar's **77** column-0 `//` lines: **35** top-level runs (after a
blank line — dropped by `scan_top_level_rules` as unrecognised lines, silently), **2** inside rule bodies after an
annotation (swallowed into the payload), **0** mid-body. And the before/after envelope diff showed exactly ONE rule
whose non-annotation tokens moved: `parseable_source_item` — `semi -> {kind: "semi"}` followed by three `//` lines
that CONTAIN `|` (`| package_import_declaration`, `| timeunits_declaration`, `| compiler_directive`, commented out
in `795e9954`, 2026-04-25): the old payload scan stopped at the first `|`, the tokenizer read each next `//` as an
EMPTY regex literal, and the generator built **four** alternatives. Measured on release `1.0.204` with
`--entry-rule systemverilog_parseable_file`: `import p::*;` **rc 0**, `timeunit 1ns;` **rc 0** (`sv_2017`/`sv_2023`),
`` `timescale 1ns/1ps `` **rc 0** on all three profiles; `;` rc 0 (control). The rule and its entry are
`unreachable_from_entry` in `rule_fire_partition_sv_2017.tsv` (certificate-coverage PROOF), so no default-entry
input could ever have observed it — and any `--entry-rule` consumer could.

**WHAT LANDED.** (a) `extract_inline_return_annotation_payload` builds the payload with every line comment cut
out (the tracker's `line_comment` flag; a comment's newline is kept so a multi-line object keeps its lines);
`//` opens a line comment in `DelimiterTracker::consume` (before the regex-literal arm), in
`tokenize_rule_expression` (like `#`), in `collect_rule_body` (a column-0 `//` terminates the body exactly as a
column-0 `#` does today — `.2`'s open question about that terminator applies to both spellings) and in
`scan_top_level_rules`. (b) Three regression tests: the three-rule grammar with a `#` and a `//` comment after
`-> {kind: "x"}` → `["return_object", "{kind: \"x\"}"]` exactly, the scalar form `-> $1`, and `//` in every
position (no regex token; a column-0 `//` ends the body). ⛔ `cargo test --lib` runs NONE of the module's 23
tests: `lib.rs:14` gates `ebnf_frontend` behind `#[cfg(feature = "ebnf_dual_run")]` — the first run reported
*"0 passed; 1030 filtered out"* and looked green; `cargo test --lib --features ebnf_dual_run -- ebnf_frontend::tests`
→ **23 passed**. (c) The SV `ebnf_raw_ast` digest moves ONCE, with the fix: `891b02cb654f… → bd64ae30002c…`, and the
`.13c.2x.4` measurement now holds — a one-word edit of the comment after `net_type`'s `-> {kind: "trireg"}` leaves
it unmoved (bank arm 3). Every carrier re-stamped through its own producer (below). (d) Regenerated all eleven
parsers (`regenerate_generated_parsers` + the `ebnf.rs` seed + `focus_scratch`): **ten byte-identical**,
`generated/systemverilog_parser.rs` `62fec0ab… → c82018f9…` in `parse_parseable_source_item` alone (it references
`parse_semi()` only). (e) Release ceremony: register RELEASE row, ledger `SV-0078`, contract note + version
`1.0.205` + digest, parser-book changelog, `CHANGES.md`; the repro ratchet gained an `entry` column and four rows
(`accepts_invalid_parseable_entry_{import,timeunit,directive}.sv` + `control_parseable_entry_semi.sv`) were
pinned against `1.0.204` BEFORE the rebuild (`checked=270 failures=0`) and flipped to `invalid` after it.
(f) **`SV-CONTRACT-CURRENCY` learned that the digest is a function of the frontend.** Its tier A re-derives
every grammar commit since the register's genesis with the CURRENT frontend and demands a row with that digest;
under the new reading every one of the 23 historical revisions produced a digest no row records — *"the SV
grammar moved in commit(s) 6d1a18b3 cb78d01f …"* — while not one grammar byte had. Rewriting 23 historical
digests would have falsified what each contract section described on its day, so the register now declares
`# frontend_rekey: f1d33865` (the last commit keyed by the previous reading) and tier A skips commits at or
before a rekey epoch, printing `rekey_epochs=1 history_skipped_by_rekey=23` in its summary — the exemption is a
number, not a silence. Registry line, `DOCTRINE_ENFORCEMENT.md` §10 and the derived book chapter carry the same
sentence (`report_doctrines --check` equal).
(g) **The rule-fire partition producer could not read a fully certified report.** `make sv_rule_fire_partition`
(tier 2 of `SV-RULE-FIRE-PARTITION`) refused with *"the certificate report carries no UNKNOWN rule list"*:
`ast_pipeline` prints that list only when it is non-empty (`rust/src/main.rs`, `if !report.unknown.is_empty()`),
and the SV grammar reads `CERTIFICATE-COVERAGE: … UNKNOWN=0 fully_certified=true` — so the instrument had been
unrunnable on its own best case, and the tracked partition dated **2026-08-19** while the corpus rule-coverage
artifact it must agree with described 9 781 accepted files against today's 9 794. `rule_fire_partition.py`
now reads `UNKNOWN=0` from the headline as the empty list; re-derived: `coverage_gap` 60 → 64,
`lr_synthetic` 57 → 14, `unreachable_from_entry` 18 (both parseable rules still in it), 98 never-fired rules,
0 unadjudicated — `sv-rule-fire-partition: OK`. The generated-reproducibility baseline was rewritten by its
own `--rebaseline` (the SV parser's new digest; the per-family `input_sha` values churn by design, since
`generated/<family>.json` embeds a `generated_at` timestamp). ⛔ *Corrected 2026-09-03 by `.2` (`-0005`): that churn
is not "by design", it is a defect — a byte-identical regeneration tripped tier 1 — and the input digest is now
timestamp-free.*
(h) **Re-stamped clean** (`PGEN-EBNF-FRONTEND-SILENT-TRUNCATION-0003`): both lanes re-run on the clean tree
`e042b3e8` — the reports had been measured at `f1d33865+dirty` because the identity rows must be written by the
run that produced them. Only the `Measured at` lines (and the machine-dependent slowest-file timing) moved;
`outcome-identity: OK` ×2, `match=5851 unexplained=268`, tuple `7556/2606/6174/4013/268`, denominator gate OK.
(i) `.1` (`-0004`) re-measured the frontend⟷meta-parser differential after this fix: `divergence_total` 175 → 169,
agreement 98.76 % → 98.81 %, `parseable_source_item` no longer diverges.
The same commit republishes the book's grammar-certification table: `GRAMMAR-CERT-CURRENCY` refused because
`systemverilog_recognized_cert_union_contract.json` led the published table by 33 commits (budget 10) — rot accrued
before this tree touched anything, and the budget tripped here; re-derived by its producer (`--check` equal).

**ACCEPTANCE CHECKLIST (TOOLBOX 3.5)**
- [x] **ROOT CAUSE (WHY + WHERE)** — `git ls-files grammars/*.ebnf | wc -l` = 18 tracked grammars; `bash scripts/check_baseline_identity.sh
  --raw-ast grammars/systemverilog.ebnf` with the pre-fix frontend showed 14 `return_scalar` tokens whose text
  ended in a comment, and one rule (`parseable_source_item`) whose token stream carried `["regex", ""]` where the
  grammar has `//`. WHY: `extract_inline_return_annotation_payload` returned the raw slice up to the next top-level
  `|` (comments "tracked" but not removed), and the frontend implemented one of the meta-grammar's two line-comment
  spellings. WHERE: `rust/src/ebnf_frontend.rs` — payload extractor, `DelimiterTracker::consume`,
  `tokenize_rule_expression`, `collect_rule_body`, `scan_top_level_rules`.
- [x] **ADDRESSED (verified)** — bank `docs/tasks/artifacts/ebnf_frontend_silent_truncation/comment_payload/probe.sh` **8/8**: census
  over all 15 readable tracked grammars (0 dirty tokens; the 3 LRM snapshots refused, exactly those), the founding
  synthetic in both spellings, the comment-edit invariance, `//` in every position, the feature-gate pin, the pinned
  entry rows, and the historical RED re-derived from the frozen census (all 14 rules now `return_object` with the
  comment cut off). 23/23 unit tests.
- [x] **NO REGRESSION** — ten of eleven generated parsers byte-identical after full regeneration (`ebnf.rs` seed and
  `scratch` included); the SV parser moves in one function only. **Corpus, per-path and two-sided, both lanes on
  the rebuilt release probe: ZERO verdicts moved** — `results.tsv` and `positions.tsv` re-hash to the SAME digests
  the reports carried before the rebuild (16 336 `sv_2017` rows: 9 811 pass / 6 525 fail; 2 606 `verilog_2005`
  rows: 2 300 / 306), the adjudication manifests, the v2005 lane list and every verdict-coverage artifact are
  byte-identical (`git status` names none of them), `match=5851 unexplained=268` and the tuple
  `7556/2606/6174/4013/268` unchanged (`sv-corpus-denominator: OK`). Parse cost: the LR-family share
  **2.756 % reproduces exactly** (24 585 940 of 892 133 560 entries over 16 336 files); the pinned-sample ratchet
  rebaselined with its identity fresh for `generated parser`, `grammar raw ast`, `instrument`, `sample inputs`
  (wall-clock advisory −7.6 %, machine-dependent, does not bind). Repro ratchet `checked=270 failures=0` after the
  flip. `SV-CONTRACT-CURRENCY` tiers A–D green (`history_skipped_by_rekey=23`). Reproducibility baseline rewritten
  by `--rebaseline`; `sv-rule-fire-partition: OK` (98 never-fired rules, 0 unadjudicated); `clippy_on_rust_change`
  ✅ (0 warnings in the changed file); ALL 35 doctrines PASS on the staged set.

### ✅✅ `.1` DISCHARGED — **`EBNF-ALTERNATIVE-PARITY` is doctrine 36: for every rule in every tracked grammar the meta-parser's alternative count equals the frontend IR's, on every commit, in 4 s; its founding RED is the `.6` phantom (source 1 vs IR 4) and it carries `.2`'s defect as a LIVE RED today** (2026-09-03, `PGEN-EBNF-FRONTEND-SILENT-TRUNCATION-0004`; DOCTRINE tier — one new gate, registry/§10/book in lockstep; ZERO Rust / grammar / codegen / generated bytes)

**WHY THIS AND WHY NOW.** `.6` found `parseable_source_item` shipping with four alternatives where the file
declares one — and the instrument built for exactly this class, the frontend⟷meta-parser envelope differential
(`LANG-CAPABILITY-AUDIT.10.6`), had SEEN that rule diverge: at token 4 (`return_scalar` vs `return_object`), after
which, by its own design, every later token is UNVERIFIED — the three phantom alternatives sat in the unverified
tail of a 98.76 %-agreement report. A per-rule COUNT is orthogonal to token position: it reads 1 vs 4. This is the
leaf's original design ("count the source alternatives with a real tokenizer, compare two passes") with the two
readers that already exist: arm 2 = the parser generated from `grammars/ebnf.ebnf` (`ebnf_dual_run_diff
--emit-ast-json`; a rule's top `alternation` list IS the count), arm 1 = `ast_pipeline --emit-raw-ast-json`
(top-level `|` operators + 1, depth over `group_open`/`group_close`). Nothing re-implemented.

**WHAT LANDED.** `scripts/check_ebnf_alternative_parity.sh` — population `git ls-files 'grammars/*.ebnf'`
(seam `PGEN_ALTERNATIVE_PARITY_GRAMMARS`); PAIRING first (same rule names, same order — reported, never guessed
around), then PARITY per rule; NOT EVALUATED **by name and counted** for a grammar either reader refuses (the
three `*_lrm_extracted.ebnf` snapshots are raw LRM syntax) or that carries an include directive (arm 1 resolves
it, arm 2 stops — `systemverilog_lrm_profiled_wrapper.ebnf`, IR 1 401 rules vs source 3); a population with
nothing comparable REFUSES (rc 2), a missing binary is NOT EVALUATED with the build line, never a pass.
`--self-test` drives both counters and every verdict cell on literals — including the frozen pre-fix
`parseable_source_item` envelope (**source 1 vs IR 4**) and the `.3.19` direction (IR 1 vs source 3) — plus a
real-arm mutation (drop one top-level `|` → *source 3 vs IR 2*). Registered as doctrine **36**: registry line,
`DOCTRINE_ENFORCEMENT.md` §10, `gate-flow.md` count markers 35 → 36, the derived book chapter re-rendered
(`report_doctrines --check` equal). On the tree: **14 grammars / 4 075 rules agree, 4 NOT EVALUATED by name,
4 s**; `grammars/systemverilog.ebnf` 1 512 rules agree — `parseable_source_item` 1 vs 1 since `.6`.

⭐ **IT ALREADY HAS A LIVE RED.** `a := b\n# a column-0 comment\n   | c` — the `.2` shape — reads **source 2 vs
IR 1** today: the meta-parser continues the rule past the comment line, the frontend ends the body. That settles
`.2`'s "decide the intended language first": `ebnf.ebnf` already decided it (the same `EBNF-SOURCE-OF-TRUTH`
reading `.6` applied to `//`). The bank holds the shape as arm 4 and will have to flip when `.2` lands. Bank
`docs/tasks/artifacts/ebnf_frontend_silent_truncation/alternative_parity/probe.sh` → **8/8**.

**MEASURED FOR THE RECORD — the differential after `.6`.** Re-run with the rebuilt `ebnf_dual_run_diff` on
`grammars/systemverilog.ebnf`: agreement 98.76 % → **98.81 %**, `divergence_total` 175 → **169**, kind divergences
174 → 168, unverified tail 1 831 → 1 773, and `parseable_source_item` no longer diverges — the six divergences the
`.6` fix removed were the two `//` sites and four of the comment-carrying payloads the differential could see.

**ACCEPTANCE CHECKLIST (TOOLBOX 3.5)**
- [x] **ROOT CAUSE (WHY + WHERE)** — `git ls-files 'scripts/check_*.sh' | xargs grep -l 'alternatives'` found no gate
  comparing per-rule alternative counts; the only frontend⟷source instrument is token-positional and, by design,
  counts nothing after a rule's first divergence (`tokens_unverified` 1 831 on the pre-fix SV report). WHERE: a
  new `scripts/check_ebnf_alternative_parity.sh` over two existing readers.
- [x] **ADDRESSED (verified)** — self-test 10/10 on literals incl. the frozen pre-fix envelope; bank **8/8** with a
  LIVE RED (arm 4), a refusal RED with text (arm 5), a GREEN sibling on the same seam (arm 6), the include
  NOT-EVALUATED path (arm 7), and the tree GREEN naming exactly its four NOT-EVALUATED grammars (arm 2).
- [x] **NO REGRESSION** — no grammar, Rust or generated byte moved (generated parsers byte-identical to
  `bc98c9a0`); `bash scripts/check_doctrines.sh` **ALL 36 PASS**; doctrine cost 4 s; `report_doctrines --check`
  equal on the re-rendered chapter; the other 35 doctrines unchanged.

`promotion:` knowledge card `docs/knowledge/a-differential-that-stops-at-the-first-divergence-cannot-count-what-follows.md`.

### ✅✅ `.2` + `.3` DISCHARGED — **a rule body now ends where `grammars/ebnf.ebnf` says it ends — at the next rule header, `@` annotation, `[>` directive or include directive — and never at a blank or comment line; the rule is written down in the book; the one tracked site whose reading changed was a dead annotation, removed with its explanation so every generated parser stays byte-identical** (2026-09-03, `PGEN-EBNF-FRONTEND-SILENT-TRUNCATION-0005`; ENGINE tier — `rust/src/ebnf_frontend.rs::collect_rule_body`; one grammar-text edit that changes NO reading)

**THE LANGUAGE WAS DECIDED BY THE META-GRAMMAR, AND DOCTRINE 36 MEASURED IT.** `grammar_file :=
(include_directive | semantic_annotation | grammar_rule | comment | whitespace)*`, `alternation := sequence
(return_annotation? "|" sequence)*` with `@stop_at_rule_boundary` on `sequence`: a rule runs to the next
top-level element; blank lines and comment lines are trivia. Six shapes through `EBNF-ALTERNATIVE-PARITY`
before the change: **A** (column-0 `#` between alternatives) and **B** (blank line between alternatives) read
*source 2 vs IR 1*; **C** (comment then a rule-level `->`), **D** (indented comment), **E** (column-0 `@` between
rules) and **F** (comment then a new rule header) agreed. So the frontend's third language — "indentation, with
a column-0 blank or comment line as a terminator" — was simply wrong on A and B, and `.2`'s "decide the
intended language first" had an answer the tree already owned (`EBNF-SOURCE-OF-TRUTH`, the same reading `.6`
applied to `//`).

**MEASURED BEFORE THE CHANGE — what the new reading touches.** A census over every tracked grammar for a
column-0 blank/comment gap inside a rule followed by a continuation line: **exactly one site** —
`grammars/return_annotation.ebnf:13-14`, a rule-level `-> $2` after a blank line under the entry rule
`return_annotation := arrow expression | arrow | expression` (there since `bc9c4b6c`, 2026-02-19). The old
frontend ended the body at the blank line and dropped `-> $2` as a stray top-level line, so the shipped
return-annotation parser never carried it — and `$2` is out of range for two of the three alternatives. Under
the new reading it would have become LIVE and changed the entry rule's AST for every annotation the code
generator parses. Ruled: the shipped behaviour is kept — the dead line is replaced by an explanation in the
grammar; the old frontend's digest of the grammar is **identical** before and after that edit
(`44ef9139d3af…` both ways, measured), so the edit changes no reading, and the language question it raises
(should the entry rule return the expression?) is a return-annotation-family decision, routed to that family's
owner rather than decided by a frontend fix.

**WHAT LANDED.** `collect_rule_body` no longer breaks on a column-0 blank line or a column-0 `#`/`//` line;
it breaks on `@`, `[>`, an include directive and a rule header, and trims trailing trivia so the comments
between two rules belong to neither. One regression test pins the tree's five founding cases (`C_baseline`,
`D_indented_comment`, `B_hash_in_string`, `A_col0_comment_after_alt1`, `E_col0_comment_after_alt2` — all
THREE alternatives now), the blank-line and `//` shapes, a comment before a rule-level `->`, and the
terminators (a header, an `@` annotation and a `[>` directive after a gap start the NEXT element; the
`@` binds the rule that follows). The `.6` test that had asserted the old terminator flips with it.
Doctrine 36's bank arm 4 — the live RED — flips to GREEN on both shapes (`source 2 vs IR 2`); the `.6` bank's
arm 4 likewise.

**ACCEPTANCE CHECKLIST (TOOLBOX 3.5)**
- [x] **ROOT CAUSE (WHY + WHERE)** — `git ls-files 'grammars/*.ebnf'` (18 tracked grammars) censused for a column-0
  blank/comment gap inside a rule followed by a continuation line: **1** site (`return_annotation.ebnf:13-14`);
  through doctrine 36, shapes A and B read `source 2 vs IR 1`, C–F agree. WHY: `collect_rule_body` treated a
  column-0 blank or comment line as a body terminator — a language the meta-grammar does not have. WHERE:
  `rust/src/ebnf_frontend.rs::collect_rule_body`.
- [x] **ADDRESSED (verified)** — frontend tests **24/24** (`--features ebnf_dual_run`; the new test pins the five
  founding cases + blank-line/`//` shapes + the terminators); `EBNF-ALTERNATIVE-PARITY` on the tree OK
  (14 grammars / 4 075 rules; the two live-RED shapes now `2 vs 2`); banks `alternative_parity/probe.sh` **8/8**
  and `comment_payload/probe.sh` **8/8** with their flipped arms.
- [x] **NO REGRESSION** — every generated parser **byte-identical** after full regeneration (11 of 11, `ebnf.rs` seed
  and `scratch` included, digests compared against the pre-change record); `grammars/return_annotation.ebnf`
  reads identically to the old frontend's reading of the original text (`ebnf_raw_ast` `44ef9139…` before and
  after, measured with both frontends); the SV digest `bd64ae30…` unmoved; `clippy_on_rust_change` ✅;
  `IDENTITY-CARRIERS: OK`; baseline identity within budget.

⛔ **FOUND BY THE PROOF ITSELF — `GENERATED-REPRODUCIBILITY` tier 1 read a byte-identical regeneration as
MOVEMENT.** After the eleven-parser re-derivation (every `parser_sha` identical) the doctrine reported *"ebnf
moved … parser `d4a1be61…` vs `d4a1be61…`, input `45a2fed3…` vs `f0d9eea2…`"*: its `input_sha` hashed
`generated/<family>.json` whole, and `ast_pipeline` stamps that file with `metadata.generated_at`. A digest that
moves when nothing moved is not an identity. Fixed at the definition: `input_sha_file()` digests the canonical JSON
with `metadata.generated_at` removed (three call sites — the baseline writer, `baseline_current`, tier 1), the
header records the measurement, and the baseline is re-derived once through the doctrine's own `--rebaseline`
(tier 2 re-proves all eleven artifacts) so its recorded `input_sha` values are the new, timestamp-free ones. The
`-0002` slice had already noticed the churn and called it "by design"; the proof run here showed it is a defect —
a rebaseline per regeneration for a change that never happened.

`promotion: declined (the transferable rule — census the blast radius of a reading change before making it, and treat each site as a decision — is the application of the existing envelope-diff and meta-grammar-authority cards; per-slice history stays here)`

### ✅ `.4` DISCHARGED — **an inline `->` inside a parenthesized or optional group is REFUSED at the byte, naming the rule, the meta-grammar clause and the lift workaround — never again re-read as a quantifier; the construct itself (legal by `ebnf.ebnf`, absent from the IR) is `.4b`** (2026-09-03, `PGEN-EBNF-FRONTEND-SILENT-TRUNCATION-0006`; ENGINE tier — `tokenize_rule_expression`'s `-` arm; ZERO grammar / codegen / generated bytes: no tracked grammar uses the shape)

**MEASURED FIRST, through both readers.** `a := (b -> {kind: "x"} | c) d`: the frontend's envelope read
`["group_open","("], ["rule_reference","b"], ["quantifier","kind: \"x\""], …` — the `->` skipped character by
character at group depth 1 and the `{…}` payload taken by the `'{'` arm as a braced quantifier — while the parser
generated from `grammars/ebnf.ebnf` read a `return_annotation` on the first alternative of the grouped alternation
(`grouped_expression := "(" rule_expression ")"`, `alternation := sequence (return_annotation? "|" sequence)*`).
The envelope differential showed 1 divergence; doctrine 36 could not see it (the alternative COUNT agrees — the
misread changes a token's category, not the count). Census over the tracked grammars: **0** sites use the shape
(`SV-CORPUS-GRAD.3.25` had already lifted its case into a named rule), so nothing shipped depends on the
misread and nothing regenerates. Frozen: `grouped_annotation/before_envelope.json`.

**RULED: refuse now, implement in `.4b`.** The tree's acceptance criterion 5 offers two honest endings — parse it
as an annotation, or reject it with a located, self-explaining error — and only the second is a frontend change:
the IR carries return annotations per TOP-LEVEL branch (`extract_rule_annotations` attaches by branch index) and
the generator has no shape for a per-branch annotation inside a group, so "parse it" is a generator feature, not
a frontend repair. The `-` arm now returns, at group depth > 0, an error carrying the byte offset, the depth, the
meta-grammar's clause and the workaround (*lift the group into a named rule and annotate its alternatives there* —
AST-transparent, `.3.25`), and `convert_scanned_rule` wraps it with the rule name. Strings and regex literals
containing `->` inside a group are untouched (controls). The divergence is now LOUD in both directions: the
meta-parser accepts, the frontend refuses and says why.

**ACCEPTANCE CHECKLIST (TOOLBOX 3.5)**
- [x] **ROOT CAUSE (WHY + WHERE)** — `git ls-files 'grammars/*.ebnf'` censused for `->` at paren depth > 0 outside
  strings/regex/comments: **0** sites; the frontend's envelope for the synthetic (`ast_pipeline --emit-raw-ast-json`)
  carried `["quantifier", "kind: \"x\""]` where the meta-parser (`ebnf_dual_run_diff --emit-ast-json`) carried a
  `return_annotation` inside `grouped` (1 divergence in the envelope differential; doctrine 36 blind to it — the
  count agrees). WHY: `tokenize_rule_expression`'s `-` arm skipped `->` at group depth > 0 and the `'{'` arm then
  read the payload as a braced quantifier. WHERE: `rust/src/ebnf_frontend.rs`, the `-` arm; the rule name is added by
  `convert_scanned_rule`.
- [x] **ADDRESSED (verified)** — frontend tests **25/25** (`--features ebnf_dual_run`; the new test covers the
  parenthesized, optional and nested forms, asserts the rule name, the byte and the workaround in the message, and
  three controls); bank `grouped_annotation/probe.sh` **8/8** (RED text ×2, lifted GREEN, string/regex controls, the
  frozen pre-fix envelope, the doctrine-36 seam, the meta-parser still accepting).
- [x] **NO REGRESSION** — no tracked grammar uses the shape, so no generated parser can change: the eleven generated
  parsers are untouched and byte-identical to `8c58582e` (`GENERATED-REPRODUCIBILITY` tier 1: 11 artifacts unmoved,
  emission sources unmoved); SV `ebnf_raw_ast` `bd64ae30…` unmoved; `EBNF-ALTERNATIVE-PARITY` OK (14 grammars /
  4 075 rules); the `.1`/`.6` banks 8/8 each; `IDENTITY-CARRIERS` unaffected.

### ✅✅ `.5` DISCHARGED — **every grammar reader decodes a `.ebnf` file the way every other user source is decoded (`pgen::source_text::read_source_file`: UTF-8 ± BOM, UTF-16 with BOM, else ISO-8859-1), a non-UTF-8 grammar reads to the SAME envelope as its UTF-8 twin, and corruption stays loud with a file offset** (2026-09-03, `PGEN-EBNF-FRONTEND-SILENT-TRUNCATION-0007`; ENGINE tier — four call swaps; ZERO grammar / codegen / generated bytes: every tracked grammar is UTF-8, so no envelope moves)

**MEASURED FIRST.** Encoding census over the tracked grammars: all UTF-8, none with a BOM — so nothing in the
repository fails today, exactly as `.12c.1` found. The refusal, on fixtures: `ast_pipeline
rust/test_data/ebnf_frontend/encoding_latin1.ebnf --emit-raw-ast-json` → *"Caused by: stream did not contain
valid UTF-8"*, rc 1; the UTF-16LE twin identically. The four sites are the ones `.12c.1`'s enumeration named:
`ebnf_frontend.rs` top-level read and `@include`d read, `ebnf_dual_run_diff.rs`, and `parser_registry.rs`'s
standalone-annotation scan of every tracked grammar.

**WHAT LANDED.** The four `fs::read_to_string` calls became `read_source_file(...)` (`.text`), the decoder's
one-line `non_utf8_notice` goes to stderr as its contract asks, and nothing else widened — the ~35 category-C
readers (JSON manifests, generated Rust, reports) keep `read_to_string`, where a decode failure is corruption
that must stay loud. Fixtures under `rust/test_data/ebnf_frontend/`: the same five-line grammar as UTF-8,
ISO-8859-1 (`©` = `0xA9` in the header comment), UTF-16LE with BOM, a UTF-8 includer of the Latin-1 file, and a
corruption twin (a UTF-8 BOM over an invalid byte). One frontend test: the Latin-1 and UTF-16LE twins parse to
the UTF-8 twin's `raw_ast` byte-for-byte, and the includer carries the included rules.

**ACCEPTANCE CHECKLIST (TOOLBOX 3.5)**
- [x] **ROOT CAUSE (WHY + WHERE)** — `git ls-files 'grammars/*.ebnf'` decoded as UTF-8: 18/18 clean, 0 BOMs (nothing in
  the tree fails today); on the Latin-1 fixture the pre-fix `ast_pipeline --emit-raw-ast-json` refused with *"stream
  did not contain valid UTF-8"* (rc 1; UTF-16LE identically). WHY: four grammar readers used `fs::read_to_string`,
  which refuses anything that is not UTF-8, while the repository's user-source decoder already existed. WHERE: the
  four sites of `.12c.1`'s enumeration (`ebnf_frontend.rs` top-level and `@include`d reads, `ebnf_dual_run_diff.rs`,
  `parser_registry.rs`'s grammar scan).
- [x] **ADDRESSED (verified)** — frontend + registry tests **26/26** (`--features ebnf_dual_run`; the new test reads the
  Latin-1 and UTF-16LE twins to the UTF-8 twin's `raw_ast` byte-for-byte and the includer's rules); bank
  `non_utf8_grammar/probe.sh` **7/7** (digest equality ×3 `9cc63f78…`, the stderr notice with the offending
  offset, the includer, the corruption RED with a file offset, the dual-run reader, doctrine 36 pairing both readers
  on the Latin-1 file, and the structural pin that no grammar reader calls `fs::read_to_string(` any more).
- [x] **NO REGRESSION** — every tracked grammar is UTF-8, so no envelope moves: SV `ebnf_raw_ast` `bd64ae30…` unmoved,
  `EBNF-ALTERNATIVE-PARITY` OK (14 grammars / 4 075 rules), `GENERATED-REPRODUCIBILITY` tier 1 OK (11 artifacts
  unmoved — the generated parsers are byte-identical to `0dada5df`); the `.1`/`.4`/`.6` banks 8/8 each.

`promotion: declined (the rule — one decoder for user-authored text, refusal for generated text — is .12c.1's own, recorded with the decoder; this is its application to the grammar readers)`

⛔ **CORRECTION 2026-09-03 (`PGEN-EBNF-FRONTEND-SILENT-TRUNCATION-0008`).** The `-0007` commit and the box above claimed
bank **7/7** while the bank as committed read **6/7 and then crashed** — my last edit to arm 7 (counting calls, not the
comment) dropped the line that set `m`, and under `set -u` the arm died with *"m: unbound variable"* before the summary
line. I committed the number I intended, not the number the bank printed — the `-0344` lesson (a control that fails for
the wrong reason) in its simplest form: read the bank's last line before writing it down. Repaired in `-0008`; **7/7**
measured after the repair, nothing else moved.

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
6. A grammar file that is valid EBNF but not UTF-8 is read, not refused, on all four grammar
   readers, with the encoding reported (`.5`).
