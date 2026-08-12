# LRM-GRAMMAR-FIDELITY: make the LRM-markdown→EBNF extraction SOTA + a standing guardrail so silent construct-drops (the `|->`/`|=>` class) can never recur

## Metadata

- Tree ID: `LRM-GRAMMAR-FIDELITY`
- Status: **`active`** (created 2026-07-23, session #198, on the director's
  directive — verbatim: *"What do you [think] about task-tree tracking this
  'the LRM-markdown→EBNF extractor' and at a later point in time to make this
  extractor sota, top-notch and put in place all the necessary guard rails for
  this problem to never happen again. Does it make any sense, your take?"* —
  raised mid-`SV-CORPUS-GRAD.3.3` after that leaf found the SVA `|->`/`|=>`
  operators had been silently dropped by the extractor; see
  [[project_lrm_extractor_sota_and_guardrails]])
- Roadmap lane: cross-family grammar-fidelity infrastructure — serves the
  **Nexsim SV delivery** (the `.9` 100%-LRM-coverage mandate,
  [[project_sv_corpus_100pct_lrm_coverage_mandate]]), the **VHDL campaign**
  ([[project_all_parsers_done_requires_corpus_graduation]]), and the
  **horizon goal** (parse any precisely-described language,
  [[project_horizon_universal_parser]]). Sibling of `SV-CORPUS-GRAD` (which
  finds extraction drops REACTIVELY, one corpus family at a time); this tree
  finds and prevents them PROACTIVELY, at the source.
- Created: `2026-07-23`
- Owner: repo-local workflow

## Why this tree exists — the `|->`/`|=>` signature

`SV-CORPUS-GRAD.3.3` (`SV-0039`) found that the two most fundamental SVA
operators — overlapped `|->` and non-overlapped `|=>` (IEEE 1800-2017 A.2.10) —
had been **entirely absent from `grammars/systemverilog.ebnf` since the
beginning** (`git log -S` empty). Root cause: the LRM-markdown→EBNF extractor
split the `|`-prefixed operators on `|`, which is the EBNF **alternation
metacharacter**, leaving the mangled `implies` (`->`) / `or_assign` (`|=`)
remnants in `prop_primary`.

This is a **class**, not a one-off. Every LRM operator/construct whose literal
collides with an EBNF metacharacter is suspect the same way. SV operators that
collide with `| ( ) * ? [ ] { }`: `|->`, `|=>`, `[*N]`, `[->N]`, `[=N]`, `(* … *)`
(attributes), `->>`, `##`. The current burn-down (`SV-CORPUS-GRAD.3.x`) finds
these **reactively** — O(defects), one corpus family at a time. A proactive
audit + a standing guardrail is **O(1)** for the whole class.

### Second confirmed instance — the `ref` reserved-word omission (`SV-CORPUS-GRAD.3.4`, `SV-0040`, 2026-07-23)

A wider face of the same LRM-fidelity gap surfaced during `SV-CORPUS-GRAD.3.4`:
`ref` — a genuine SV `port_direction` (`port_direction_sv_only := kw_ref`) and
an **IEEE 1800 Table B.1 reserved keyword** — was absent from
`reserved_non_keyword_identifier_sv`, so it wrongly matched `port_identifier`.
It fell through the `SV-0036` net (which appended only the
`verilog_2005`-reserved delta to the SV list; `ref` is SV-*only*-reserved, so it
was never in that delta). This is the same class — an LRM-mandated terminal
(here a reserved keyword) missing from the shipped grammar — surfacing
**reactively** only because a modport fix happened to expose it. The `.1`
coverage audit's scope should therefore include not just Annex A **productions**
but the **reserved-keyword table (Annex B)** cross-checked against the
grammar's `reserved_non_keyword_identifier_sv`/`_v2005` lists, per profile — a
bounded, high-yield check that would have named `ref` (and any siblings)
proactively.

## The sharpening caveat (the honest architecture note)

The extractor is **no longer on the live regeneration path.** The shipped
`grammars/systemverilog.ebnf` is a hand-annotated flattened *synthesis*
(`@profiles`, return annotations, store predicates) that a re-extraction would
destroy. So "make the extractor SOTA" *alone* does not fix the shipped grammar.
That splits the work into two pieces with different leverage:

1. **Highest-leverage, extractor-independent:** a standing **LRM-Annex-A ⟷
   shipped-grammar coverage gate** — prove every LRM production/operator is
   reachable in `systemverilog.ebnf`. This finds all the remaining
   `|->`-siblings in one pass and prevents recurrence, regardless of how the
   grammar was authored. It is the proactive twin of `SV-CORPUS-GRAD.7a`'s
   rule-coverage instrument, closing the loop from the *other* direction
   (LRM→grammar, not corpus→grammar), and directly feeds the `.9` mandate.
2. **Systemic capstone:** the SOTA extractor + an extraction-fidelity gate —
   valuable as (a) a re-derivable *oracle* to diff the shipped grammar against,
   and (b) the faithful path for future LRM-based families (VHDL; the horizon
   goal). The generalizable guardrail: any markdown→EBNF extractor must
   escape/detect operator literals that collide with EBNF metachars, with a
   round-trip check that the emitted grammar's operator terminals ⊇ the LRM's
   operator table.

## Ground rules

- **TOOLBOX-first + task-acceptance on every landing leaf** (root cause +
  addressed + no regression, evidence-backed).
- **Never game coverage:** N/A-with-cause only for LRM clauses/productions with
  no parse surface (the ratified principle, cf. `SV-CORPUS-GRAD.9`).
- **Heavy runs under `scripts/run_with_memory_guard.sh`**.

## Leaves

### `.1` — The LRM-Annex-A ⟷ shipped-grammar coverage audit (the recommended first leaf)

- **Status: `todo`** — a read-only, tool-backed sweep that enumerates every
  IEEE 1800-2017/2023 Annex A production and operator terminal (from the in-repo
  LRM markdown `docs/systemverilog/2017`/`2023`) and checks each is
  **reachable/representable** in `grammars/systemverilog.ebnf`. Output: the
  list of LRM constructs with NO grammar derivation path (the proactive
  `|->`-sibling worklist) + the metachar-collision operators specifically
  audited (`|->`, `|=>`, `[*]`, `[->]`, `[=]`, `(* *)`, `->>`, `##`). Each gap
  becomes an `SV-CORPUS-GRAD.3.x`-style fix leaf. This is the highest-yield,
  bounded first step — it surfaces the rest of the class before sessions are
  spent finding them one corpus row at a time.

### `.1a` — the BRACE half of `.1`'s sweep is DONE and closed; the BRACKET half needs a discriminator that does not exist yet (routed in by `SV-CORPUS-GRAD.13c.2e`, 2026-08-12)

- **Status: `todo`** — and it arrives with **half its charter already discharged**, which is why it
  is a leaf rather than a note.
- ⭐ **WHAT LANDED.** `SV-CORPUS-GRAD.13c.2e` (`PGEN-SV-CORPUS-GRAD-0214`, ledger `SV-0053`) built
  the brace half of the sweep `SV-0049` asked for and left undone. `SV-0049`'s own changelog entry
  says it plainly: *"THE CLASS FINDING: this is the **seventh** logged instance of the
  dropped-delimiter class … The reactive posture does not converge — an exhaustive Annex-A
  bracket/brace sweep is what closes the class."* It then fixed its instance and did not sweep, and
  **the eighth instance shipped three days later, one clause away**. The instrument now exists:
  `docs/tasks/artifacts/sv_corpus_grad/intersect_braces/lrm_brace_transcription_sweep.py`, run over
  BOTH the 2017 and 2023 extracted LRMs, verdict **2 mis-transcribed → 0**.
- ⛔ **WHY THE BRACE HALF WAS TRACTABLE, stated because it is the whole difficulty of what remains.**
  Braces admitted a *decidable* discriminator: a `{ X }` whose inner nonterminal is itself a LIST
  (`*_list`) can only be literal syntax, because repeating a comma-separated list with no separator
  between repetitions is not something any clause means. That rule separates the three literal-brace
  productions (`expression_or_dist`, `inside_expression`, `select_condition`) from the 49 genuine
  repetitions with no judgement calls.
- ⛔⛔ **THE DISCRIMINATOR DOES NOT TRANSFER TO BRACKETS — MEASURED, NOT ASSUMED.** The naive
  transfer (LRM `[ X ]` that the grammar renders as an optional group with no `lbrack` in the rule)
  returns **83 rows / 45 distinct productions**, and the population is visibly dominated by genuine
  optional metasyntax: `function_declaration ::= function [ lifetime ] …`,
  `implicit_data_type ::= [ signing ] { packed_dimension }`, `[ port_direction ]`,
  `[ name_of_instance ]` ×7. Two were read against the LRM and both are ordinary optionals. ⇒ A
  bracket sweep keyed this way would be **~45 candidates of which the true positives are a handful**
  — a ratio that teaches waivers, the failure mode `GENERATED-LINT-CORRECTNESS.6`/`.12` document.
- **WHAT THE LEAF ACTUALLY OWES:** find the bracket discriminator before running the sweep. The
  known bracket instances point at a shape rather than a name — `SV-0044` `cycle_delay_range`
  (`##[1:3]`), `.3.8`'s literal `[ ]`, and the `[*]` / `[->]` / `[=]` operator family `.1` already
  lists — so the candidate signature is *the bracketed content is a RANGE or an operator glyph, not
  a nonterminal that stands alone elsewhere*. That is a hypothesis, and it must be tested against
  the 45 the naive rule returns before any fix is proposed.
- **ROUTING EVIDENCE** (`ROUTING-EVIDENCE` doctrine — what was MEASURED to place it here).
  (1) *Does it reproduce outside the family?* The **mechanism** is family-agnostic — any
  standard whose BNF reuses its metacharacters as literals is exposed, and this tree's charter is
  exactly the LRM-extractor class. (2) *What was measured?* The 2/0 brace verdict and the 83/45
  bracket sizing above, both from tracked instruments over tracked LRM markdown. (3) *Why not
  finish it in `13c.2e`?* Because the brace half had a discriminator and the bracket half does not:
  bolting an undiscriminated 45-row sweep onto a leaf whose evidence is otherwise exact would have
  made the leaf's own claim ("the class is closed") false. ⚠️ **HONEST BOUND recorded on both
  sides:** `.13c.2e` closes the BRACE class only, and says so.

### `.1b` — ⭐ a SECOND fidelity axis the `.1` charter cannot see: **Annex A ⟷ the clauses' own normative EXAMPLES** (routed in by `SV-CORPUS-GRAD.3.19`, 2026-08-09)

- **Status: `todo`** (opened by `SV-CORPUS-GRAD.3.19`; that leaf is the worked existence proof
  and carries the full evidence bundle at
  `docs/tasks/artifacts/sv_corpus_grad/config_use_param_override/`).
- **WHY THIS IS NOT `.1`.** `.1` audits *shipped grammar vs Annex A* — it finds productions
  PGEN failed to transcribe or transcribed wrongly, which is the whole seven-instance
  "dropped-delimiter" class (`SV-0002` `stream_concatenation`, `trans_range_list`,
  `boolean_abbrev`, the six bounded-property operators, `value_range`, `cycle_delay_range`
  `SV-0044`, `expression_or_dist` `SV-0049`). ⛔ **It is structurally blind to a defect where
  PGEN's transcription of Annex A is CORRECT and Annex A itself is wrong**, because Annex A is
  its oracle. `.3.19` is exactly that: `use_clause` is transcribed faithfully — all four
  alternatives, `[lib.]` optional, `[: config]` optional — and it still rejected 8 corpus rows
  and 7 verbatim LRM example lines, because IEEE 1800's own clause 33.4.3 writes
  `instance top use #(.WIDTH(32));` (7 times, in BOTH the 2017 and 2023 revisions) while its
  Annex A `use_clause` has no `#` at all.
- **THE INSTRUMENT this needs** (and it is different from `.1`'s): extract the fenced/indented
  CODE EXAMPLES out of the LRM clause bodies and run each through the shipped parser under the
  matching profile. An example the standard prints as legal that the parser rejects is either a
  grammar gap or an Annex-A defect — and the two are told apart by asking whether Annex A
  derives it. This is a corpus PGEN already owns and has never used: `docs/systemverilog/2017`
  and `docs/systemverilog/2023` are in-repo.
- **WHY IT IS WORTH DOING RATHER THAN LOGGING.** `.3.19` found its instance reactively, from a
  corpus cluster, in a clause nobody had audited. There is no reason to believe clause 33 is the
  only place the standard contradicts its own Annex A — and every such site is, by construction,
  invisible to both `.1` and to the reactive corpus lane until some vendored file happens to use
  it. Annex A's own preamble licenses the reading: *"The normative text description contained
  within the clauses … provide additional details on the syntax."*
- **HONEST BOUND to carry into the design:** LRM example blocks are not all self-contained
  compilable units (many are fragments, some are deliberately erroneous — the ispras suite marks
  these `! TYPE: NEGATIVE`), so the instrument needs an admission policy before it can produce a
  pass rate, or it will report fragments as defects. Start report-only.

### `.1c` — ⛔ the strictness switch's declared evidence base is a BIASED SAMPLE: the accepts-invalid population cannot see over-acceptance the corpus never probes negatively (routed in by `SV-CORPUS-GRAD.3.26`, 2026-08-10)

- **Status: `todo`** — and ⛔ **UNWITNESSED**: opened 2026-08-10 by `SV-CORPUS-GRAD.3.26` on a
  worked counter-example that was **retracted the same day**. The structural argument below is
  intact; it currently has **zero** confirmed instances. Corrected forward by
  `SV-CORPUS-GRAD.3.26d` (`PGEN-SV-CORPUS-GRAD-0192`).
- **THE CLAIM BEING CHALLENGED.** `SV-CORPUS-GRAD`'s strictness directive (2026-07-25, constraint 2)
  designates the accepts-invalid population — **21 rows (`sv_2017`) + 14 (`verilog_2005`) = 35** —
  as the design input for the deferred strictness switch, on the stated ground that those are
  *"by construction, every place PGEN currently accepts what the standard forbids."*
- **THE HYPOTHESIS (structural, and it uses no SV-specific fact).** That population is built from
  corpus rows whose **answer key** says `must_reject`. A tolerance the entire ecosystem shares is
  precisely the one for which no suite writes a negative test — so if such a tolerance exists in the
  shipped grammar, the census is **blind to it by construction**. ⇒ the population would measure
  *over-acceptance that some suite happens to probe negatively*, not over-acceptance; a switch tuned
  on those 35 rows would be tuned on bucket (b) and ship blind to bucket (a), its actual customers.
  Same shape as the standing *"no cut heuristic is a census"* lesson.
- ⛔⛔ **THE ORIGINAL WITNESS IS RETRACTED — and how it failed is itself the leaf's most useful
  evidence.** The leaf was opened citing `empty_unpacked_array_concatenation`'s `'{ }` arm as an
  over-acceptance the census could never see. **`'{ }` is legal SystemVerilog**: §11.4.12 writes the
  construct's own delimiter pair as `'{ }`, Annex M/VPI names the operator
  `vpiAssignmentPatternOp 75 /* '{} assignment pattern */`, and no LRM text forbids it (measured
  exhaustively over both revisions with `pymupdf`, `SV-CORPUS-GRAD.3.26b`/`.3.26c`). It is not
  over-acceptance at all, so it never belonged in this population — and the reason it looked like
  over-acceptance is the **same** methodological error this leaf is about, one level up: a verdict
  read off Annex-A derivability instead of off the standard's text. See
  [[feedback_sv_strict_lrm_compliance_default]] § BOUNDING RULING — *"non-LRM" is a CITATION, never
  an inference.*
- ⚠️ **CONSEQUENCE FOR THIS LEAF, stated plainly rather than papered over:** an "over-acceptance
  audit" that classifies by *Annex-A non-derivability* would have produced exactly the retracted
  finding — it would flag `'{ }`, `q[a:$]` and `use #(...)`, all three legal, all three already
  measured. **Annex-A non-derivability is therefore a KNOWN-BAD classifier for this instrument**,
  and any design that starts from it is starting from a refuted premise. Whatever `.1` builds must
  separate *"the annex does not derive it"* (frequent, and benign three times over) from *"the
  standard forbids it"* (which requires a located sentence). ⭐ **That distinction is the real
  deliverable of this leaf**, and it is worth more than the finding that opened it.
- **REPRODUCES OUTSIDE SV — the routing test (`ROUTING-EVIDENCE`).** The defect is in the *method*,
  not the SV grammar: any family whose over-acceptance census is derived from suite answer keys
  inherits it, which is every family in `CORPUS-GRAD-ALL`. Nothing about the argument uses IEEE 1800
  — it uses only "the census key is the suite's expectation, and suites do not write negative tests
  for what everyone accepts." That is why it lands here (cross-family fidelity infrastructure) and
  not in the SV corpus tree.
- **THE INSTRUMENT THIS NEEDS**, and it is a sibling of `.1`'s, not of the corpus lane's: enumerate
  what the shipped grammar accepts that the transcribed Annex A cannot derive, *without* asking any
  corpus whether it minds. `.1`'s Annex-A ⟷ shipped-grammar comparison already builds the machinery
  for one direction (productions PGEN failed to transcribe); this is the same comparison read the
  other way round. ⛔ **But its output is a CANDIDATE LIST, never a verdict list** — see the
  known-bad-classifier note above. Each candidate needs a second, text-level adjudication (is there
  a sentence forbidding it?) before it can be called over-acceptance, and the expected outcome for
  most candidates, on this repository's own measured record, is *"legal; the annex is incomplete"*.
- **SEED ROWS:** ⛔ **none — the leaf has no confirmed instance.** The three constructs measured so
  far in this class (`'{ }` §11.4.12/Annex M; `q[a:$]` A.8.4 footnote 42, `SV-CORPUS-GRAD.3.25`;
  `use #(...)` clause 33.4.3, `.3.19`) are all **legal**, i.e. all three are negative controls for
  the instrument, not seed rows. ⭐ That is a genuinely useful starting set: **an audit that flags
  any of the three has mis-classified**, which gives the instrument a ground-truth control before
  it publishes a single number.

### `.1d` — ⭐ the extractor's OPERATOR-NAME table collides with an Annex A KEYWORD: SVA `implies` does not exist in the grammar (`todo`, routed in by `ENGINE-UNIVERSAL-SERVICES.13` slice 1, 2026-08-12)

- **THE COLLISION, at its source.** `tools/extract_systemverilog_lrm_profiles.py:133` maps the
  operator glyph `"->"` to the rule name `implies` (its `PUNCTUATION_TOKEN_NAMES` table). Annex A
  *also* spells a reserved keyword `implies` — `property_expr ::= … | property_expr implies
  property_expr` (A.2.10; the operator is normative in §16.12.7). Both land on the same identifier,
  so the shipped grammar carries exactly one rule for the two of them:
  `implies := trivia "->"` (`grammars/systemverilog.ebnf:6388`).
- **MEASURED, three ways, none of them a reading of the code:**
  - `grep -c kw_implies grammars/systemverilog.ebnf` → **0**, while `kw_iff_ee1c009e` → **18**. The
    sibling operator in the *same* production (`property_expr iff property_expr`) was extracted as a
    keyword; `implies` was not. That asymmetry inside one production is the signature.
  - `property p; a implies b; endproperty` → **REJECT**, `furthest_position=39`.
  - `property p; a -> b; endproperty` → ACCEPT, but traced it resolves through
    `prop_primary_sv_2017 selected branch 1/30` = `sequence_expr`, i.e. as the ordinary binary
    expression `a -> b` (A.8.6) — which is legal SV and *not* the property operator. So the `->`
    accept is not evidence the property operator works.
- **WHY IT MATTERS:** an over-REJECTION of LRM-legal SystemVerilog on a normative SVA operator, from
  precisely the metachar/name-collision silent-drop class this tree was created to own. It is
  invisible to `--lint-grammar` (the name resolves, so `undefined_references=0`) and invisible to the
  corpus lanes unless a file happens to use the keyword.
- **OWED:** (1) confirm the keyword against the LRM text (this routing is grounded in the extractor
  table + the `iff` asymmetry, not in a reading of the standard's keyword list); (2) a sweep — the
  same table maps `<->` to `iff_arrow`, `=>` to `sequence_implies`, `*>` to `full_path_arrow`, so ask
  whether any OTHER entry shadows an Annex A keyword or nonterminal, exactly as `.13c.2d` asked for
  the `kw_*`-terminal-named-like-a-nonterminal shape; (3) the fix (a real `kw_implies_*` terminal for
  the keyword, the punctuation rule renamed), which is **accept-widening** and therefore needs the
  full ceremony; (4) a negative reproducer so the collision cannot silently return.
- ⚠️ **SEQUENCING:** the property-operator half is currently masked — `property_expr implies
  property_expr` is *also* unreachable through `ENGINE-UNIVERSAL-SERVICES.13`'s cycle SV-6/SV-7, so
  fixing the keyword alone will not make `a implies b` parse at property level until `.13` lands.
  Fix the naming anyway (it is a distinct defect), but expect the flip to need both.
- ⛔ **SCHEDULE (a NAMED TRIGGER, because `todo` is not a schedule —
  [[feedback_every_finding_is_owned_and_scheduled_never_just_logged]]).** Split in two, deliberately:
  - **(i) the SWEEP — do it NOW-eligible, no blocker.** Ask of every entry in
    `PUNCTUATION_TOKEN_NAMES` whether its assigned name shadows an Annex A keyword or nonterminal
    (`<->`→`iff_arrow`, `=>`→`sequence_implies`, `*>`→`full_path_arrow`, …). Read-only, zero grammar
    bytes, and it SIZES the class instead of fixing one instance — the
    [[feedback_a_named_call_site_is_a_category_of_call_sites]] discipline. This half is the one to
    pick up whenever the SV lane wants a bounded read-only slice.
  - **(ii) the FIX — triggered by `ENGINE-UNIVERSAL-SERVICES.13` landing.** It is accept-widening
    (a new `kw_implies_*` terminal + the punctuation rule renamed), so it needs the full ceremony,
    and its own acceptance flip (`property p; a implies b; endproperty` → ACCEPT) cannot be
    demonstrated until `.13` unblocks the property cascade. Landing it before then would mean
    shipping an accept-widening change whose acceptance test still fails for an unrelated reason.
- ⛔ **IN-LANE, not a lane-lock exception.** This is an over-rejection of LRM-legal SystemVerilog by
  the SV parser, so it belongs to the SV release bar; it is not "another family's finding" being
  smuggled past [[project_nexsim_sv_signoff_delivery_focus]].

### `.2` — The standing coverage gate

- **Status: `todo`** — promote `.1`'s audit into a deterministic `make` gate
  (`lrm_grammar_coverage_gate`) that fails when a required LRM production/
  operator has no grammar reachability, wired so the class cannot silently
  recur. Complements `SV-CORPUS-GRAD.7a` (corpus→grammar) with the LRM→grammar
  direction.

### `.3` — SOTA the extractor + an extraction-fidelity gate (the systemic capstone)

- **Status: `todo`** — make `tools/` LRM-markdown→EBNF extraction faithful for
  metachar-colliding operator literals (escape/detect `| ( ) * ? [ ] { }`), and
  add a fidelity gate: a fresh extraction's operator-terminal set ⊇ the LRM's
  operator table, with a round-trip proof. Valuable as a re-derivable oracle
  and the faithful path for future LRM families (VHDL; the horizon goal). Own
  scope; sequenced after the SV delivery push per the director's "at a later
  point in time".

### `.4` — Generalize to the other LRM-based families

- **Status: `todo`** — apply the `.1`/`.2` audit+gate to VHDL (IEEE 1076) and
  verilog_2005 (IEEE 1364-2005), then any future LRM-based grammar. The
  guardrail is language-agnostic (the metachar-collision class is universal).

## Sequencing (recorded, director-gated)

The director framed the SOTA-extractor work as "at a later point in time" —
after the current SV delivery push. My recorded engineering recommendation
(see the decision record) is that **the `.1` coverage audit is worth pulling
forward**, because it will surface the rest of the `|->`-class proactively
rather than reactively. `.3` (the extractor rewrite) can wait. The director
decides when each leaf opens. This session created the tree (the "task-tree
track this" directive) and did NOT start any leaf — the SV `.3.x` burn-down
remains the active frontier.

**Updated 2026-08-12 (session #221).** Four leaves have since been routed in by SV work and each
carries its own named trigger rather than a bare `todo`: `.1a` (the bracket half of `.1`'s sweep,
needs a discriminator that does not exist yet), `.1b` (Annex A ⟷ the clauses' normative examples),
`.1c` (the strictness switch's biased evidence base), and `.1d` (the `implies` keyword collision —
sweep half do-able now, fix half triggered by `ENGINE-UNIVERSAL-SERVICES.13` landing). The tree is
therefore no longer "created but unstarted": it is the standing destination for LRM-fidelity findings
the SV burn-down surfaces, and `.1d` is the first that is an outright over-rejection of legal SV.

## Acceptance Criteria (tree)

1. A tool-backed audit proving, per LRM production/operator, whether it is
   reachable in the shipped grammar (gaps named + routed to fix leaves).
2. A standing deterministic gate (`.2`) that fails on a missing required LRM
   construct, so the silent-drop class cannot recur.
3. (Capstone) A SOTA extractor + fidelity gate handling metachar-colliding
   operator literals, usable as a re-derivable oracle and for future families.
4. Full lockstep (books/contracts/tracker) and every landing leaf through the
   TOOLBOX acceptance checklist.
