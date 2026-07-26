# LEX-ADJACENCY: a NO-LAYOUT lexical boundary that BOTH the parser and the generator honour

## Metadata

- Tree ID: `LEX-ADJACENCY`
- Status: `active` (opened 2026-07-26, session #207) — **design-first; no code
  until the design leaf `.1` is adjudicated.** ⭐ **`.1` DONE**
  (`PGEN-LEX-ADJACENCY-0001`, session #208, 2026-07-26) — the design is
  adjudicated and recorded in
  [`LEX-ADJACENCY-design.md`](LEX-ADJACENCY-design.md); frontier is now `.2`.
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

⛔ **CORRECTED by `.1` (session #208) — read this before the original text below.**
The gap is **not** that the capability is absent. Both halves exist and both ship;
what is absent is any way for a **grammar** to ask for them. See
[`LEX-ADJACENCY-design.md`](LEX-ADJACENCY-design.md) §0–§1 for the tool output.

| side | the capability that EXISTS | how it is requested today | why `time_literal` cannot use it |
|---|---|---|---|
| PARSE | `match_regex(pattern, /*skip_leading_whitespace=*/ false)` — **live**: `generated/return_annotation_parser.rs` emits it at **10 of 20** sites | a hard-coded rule-NAME allowlist in codegen (`ast_based_generator.rs:5551` + 5 mirrors + the census constant) naming exactly `string_content_double` / `string_content_single` | those two names belong to `return_annotation.ebnf`; no grammar can add itself to an engine `matches!` arm |
| GENERATE | interior separator suppression (`atomic_token_depth`, `stimuli_generator.rs:9638`) | **INFERRED** from the return shape — `-> $text`/`$0` on every branch, or `@transform` (`rule_is_lexically_atomic`, `:13023`) | requesting it costs the `{value, unit}` AST object, AND it also fuses the rule's LEFT-exterior boundary |

⇒ The honest statement of the gap: **a grammar cannot DECLARE "this production is
one lexical token".** The engine can already do it; the EBNF cannot ask.

⇒ Consequently `.2` is *decouple and scope two existing mechanisms*, not *invent a
primitive* — and it retires an `EBNF-SOURCE-OF-TRUTH` breach (an acceptance-affecting
rule-NAME literal inside the engine) as a by-product.

<details>
<summary>Original (session #207) framing — superseded on the PARSE row, kept for the record</summary>

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

**Where it went wrong:** the SV `0` count is correct, but the inference from it
("there is no per-token opt-out") did not check whether any OTHER grammar emits
the `false` arm — `return_annotation` does. The generate row is correct as
measured; what it missed is that a THIRD trigger (`-> $text` / `@transform`)
suppresses the separator, and was never probed.

</details>

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

- **Status: `done`** (`PGEN-LEX-ADJACENCY-0001`, session #208, 2026-07-26).
  **Read-only** — no code, grammar, or generated artifact touched.
- **The design record:** [`LEX-ADJACENCY-design.md`](LEX-ADJACENCY-design.md).
  Evidence bundle: `docs/tasks/artifacts/lex_adjacency/`.
- **The decision, in one line:** add a rule-level **`@lexical_token: true`**
  semantic annotation meaning *"this rule denotes ONE lexical token"* — **deep**
  (transitive over the rule's derivation) and **interior-only** (the rule's own
  outer boundaries keep separating normally). Parse half emitted **statically**
  (specialized no-layout twins over the rule's closure), so a bare parse pays
  nothing.
- **⭐ The finding that reframed the tree:** the primitive is not missing, it is
  **un-declarable**. Both halves already ship — the parse half is live in
  `generated/return_annotation_parser.rs` (10 of 20 `match_regex` sites pass
  `false`), gated by a hard-coded rule-NAME `matches!` arm; the generate half is
  live as `atomic_token_depth`, inferred from the return shape. `.3.11`'s
  "ROUTE 4 … DOES NOT EXIST" is corrected.
- **The measured gap (the whole design in one table)** — `run_probes.sh`, seed 0:

  | setting | interior seam | exterior seam | emitted | verdict |
  |---|---|---|---|---|
  | ordinary object return (today) | open ✗ | separated ✓ | `<A>timeunit 0 ns;` | the `.3.11` defect |
  | `-> $text` (inferred atomic) | closed ✓ | fused ✗ | `<D>timeunit02s;` | breaks a different boundary |
  | `@transform` (inferred atomic) | closed ✓ | fused ✗ | `E0105fs` | same + coerces the AST |
  | **what the LRM requires** | closed ✓ | separated ✓ | `timeunit 10ns;` | **unreachable today** |

  Unreachable because `stimuli_generator.rs:9421` drives BOTH effects from one
  `is_atomic` bool — while `append_generated_segment` already consumes them as two
  independent conditions. That asymmetry is why `.2` is small.
- **⭐ Why this route succeeds where `.3.11`'s `@sample` route failed** (the tree's
  acceptance condition 3, measured): `@sample` collapsed generator coverage to
  **rules 3/13, branches 0/6**; the atomic route holds **rules 11/11, branches
  5/5** — every unit branch still generated, so nothing is stranded toward cert
  `UNKNOWN`.
- **Static emission is affordable — measured, not assumed** (`closure_probe.py`):
  `time_literal`'s transitive closure is **14 of 1,475 rules (<1%)**, so
  specialization costs ~13 extra rule methods and **zero** runtime cost. A runtime
  depth counter was **rejected on the ⭐ speed north star** (it would tax all 1,798
  terminal sites for a primitive used by two rules).
- **⚠️ Two traps handed forward, both named before implementation:**
  1. the parse half must ALSO drop **explicit** layout elements — SV writes
     `kw_ns_7320d5b7 := trivia /ns\b/` (`systemverilog.ebnf:6458`), so flipping
     `skip_leading_whitespace` alone would still accept `10 ns`;
  2. "deep" closes seams some LRM rules deliberately leave **open** (§5.7.1
     based-literal seams, the `.3.10` law) — the footnote 33/48/50 re-sweep in `.3`
     must re-check per rule, never apply the directive mechanically.

- **Acceptance Checklist (enforced)**
  - [x] **REPRODUCE / ISSUE** — `bash docs/tasks/artifacts/lex_adjacency/run_probes.sh`:
    the control variant emits `<A>timeunit 0 ns;` (interior seam open — the
    footnote-44 over-acceptance `.3.11` root-caused), and no setting of the engine
    produces the required `timeunit 10ns;`.
  - [x] **ROOT CAUSE (WHY + WHERE)** — both halves located and named:
    PARSE `ast_based_generator.rs:5551` `!matches!(rule_name, "string_content_double" | "string_content_single")`
    + 5 mirrors (`cascade.rs:1397`, `cascade.rs:2060`, `cascade/value.rs:878`,
    `cascade/value.rs:1107`, `scan.rs:468`) + `fusibility_census.rs:60`;
    GENERATE `stimuli_generator.rs:13023 rule_is_lexically_atomic`, coupled at
    `:9421` into `:9638` (interior) and `:9650` (exterior). Live-emission proof:
    `return_annotation_parser.rs` no-skip=10/20 vs `systemverilog_parser.rs`
    no-skip=0/1798.
  - [x] **FIX** — N/A code-wise (design leaf). The design's fix-hierarchy tier is
    **declarative (annotation)** — the highest tier — with the engine change
    confined to honouring it.
  - [x] **ADDRESSED (verified)** — the design is decided and falsifiable, not a
    menu: each rejected candidate carries a measurement (per-element annotations
    are not generator-visible; `@sample` coverage 3/13 vs 11/11; closure 14/1475).
    Probe outputs are deterministic — `run_probes.sh` and `closure_probe.py` both
    re-run **byte-identical** (`cmp` clean).
  - [x] **NO REGRESSION** — nothing executable changed: no `grammars/`, `rust/`,
    codegen, generated artifact, or contract manifest touched
    (`git status` shows only `docs/`). Probes ran against a fresh scratchpad
    grammar and the read-only `--dump-gen-ast` IR; no stray artifact landed in the
    repo (checked — the `.3.10` stray-JSON incident).
  - [x] **LOCKSTEP** — tree framing table corrected in place, design doc added,
    `docs/TASK_TREE.md` frontier updated, `MEMORY.md` / `CHANGES.md` /
    `DEVELOPMENT_NOTES.md` updated, and the `SV-CORPUS-GRAD.3.11` blocker note
    re-pointed at the corrected finding. The book chapter
    (`docs/book/src/lexical-annotations.md`) documents the **inferred** form today
    and is a declared `.2` lockstep obligation — deferred deliberately, because
    publishing an annotation the engine does not yet accept would be book drift.

<details>
<summary>Original charter for this leaf (session #207) — satisfied above</summary>

- Adjudicate the surface AND both engine halves together —
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

</details>

### `.2` — Implement + gate

- **Status: `todo`** — **UNBLOCKED** by `.1`; this is the **current frontier**.
- ⭐⭐ **DIRECTOR GO (2026-07-26, session #208).** The scope question `.1` raised
  (*is strict fidelity here worth an engine change, given the deferred
  dialect-tolerance switch?*) is **answered: build it.** Verbatim: *"PGEN SV needs
  to be 100% compliant to the LRM by default. We can add dialect-tolerance later if
  need be, but we need to work towards LRM full compliance and strictness, no
  exception, no compromise and non-negotiable — but still allowing
  dialect-tolerance, if need be at a later time in the future."* ⇒ strictness is
  not contingent on anything; tolerance, if ever built, is an ADDITIVE opt-in on
  top of a compliant default ([[feedback_sv_strict_lrm_compliance_default]]).
  ⛔ **Provenance note:** the apparent tension this leaf was weighed against rested
  on an UNVERIFIED engineer claim ("mainstream simulators accept `10 ns`") that had
  already been measured FALSE in session #206 — **0 spaced vs 273 tight across
  16,336 real-world files** — and then leaked back into the resume pointer. There
  was never a real trade-off here.
- ⭐ **DIRECTOR REAFFIRMATION, same session — and it is exactly this leaf's design:**
  *"even dialect tolerance or any other feature shall be user controllable via the
  EBNF, remember the sole source of truth."* `@lexical_token` is EBNF-declared by
  construction, and step 3 below **retires** a hard-coded rule-NAME `matches!` arm —
  the commonest disguise of an [[project_ebnf_is_single_source_of_truth]] breach.
  That retirement is therefore not incidental cleanup; it is the directive applied.
  Engine tier, so the design record (`LEX-ADJACENCY-design.md`) had to come first
  per the fix hierarchy (annotations > store > grammar > engine). The directive
  itself is annotation-tier; only its *honouring* is engine-tier.
- **Scope, as decided by `.1`** — decouple + scope two mechanisms that already
  work, do not invent one:
  1. **declare** — accept rule-level `@lexical_token: true` (validator + lint);
  2. **generate** — set the INTERIOR suppression (`atomic_token_depth`) without
     the EXTERIOR cohesion flag (`last_terminal_from_atomic_rule`) — the third
     state in the design's §4 table;
  3. **parse** — emit specialized no-layout twins over the annotated rule's
     transitive closure, dropping BOTH implicit layout skipping AND explicit
     layout elements (`trivia`), and **retire** the
     `string_content_double`/`string_content_single` name-gate at all 6 sites +
     the census constant, converting `return_annotation.ebnf` to the directive;
  4. **lint** — hard-error a `@lexical_token` rule whose closure is unbounded or
     re-enters a non-lexical rule (keeps the emission static, keeps the primitive
     honest).
- **⚠️ `return_annotation.ebnf` is the migration's own regression test** — it is
  the one grammar with a known-good before/after (`no-skip=10 of 20`). Convert it
  first; a byte-identical generated parser is the proof the new path reproduces
  the old gate exactly.
- Wire a gate that proves BOTH settings behave as declared, on a synthetic
  grammar (the `.1` probes are ready to reuse) *and* on a real one. The full
  acceptance list is `LEX-ADJACENCY-design.md` §6.
- **Inertness is a hard requirement:** a grammar without the directive must be
  byte-identical (the `@quantified_separator` precedent — an empty policy map
  makes the path inert).

#### Decomposition (session #208) — `.2` as a single leaf is beyond a safe slice

`.2` spans the directive registry, the stimuli generator, six codegen sites, a new
lint, a gate, and a real-grammar migration, and the parse half needs heavy
rebuilds. Per the batch rule ("stop when a task expands beyond a safe slice") it is
cut into four independently-committable sub-leaves, each one complete and verifiable
on its own:

| sub-leaf | scope | verified by | build cost |
|---|---|---|---|
| **`.2.1`** | **declare + GENERATE half.** Registry entry (`ParserAndStimuliSteering`), `compile_lexical_tokens` policy map, validator wiring, and the generator decoupling — set `atomic_token_depth` WITHOUT `last_terminal_from_atomic_rule` (the design's third state). | the `.1` probe grammars: the annotated rule must emit `timeunit 10ns;` — interior closed, exterior separated. Inertness: every un-annotated grammar byte-identical. | debug `ast_pipeline` only — no parser regen |
| **`.2.2`** | **PARSE half.** Static no-layout twins over the annotated rule's closure, dropping BOTH implicit layout skipping and explicit `trivia` elements. | interior layout REJECTS, tight ACCEPTS, on a probe grammar. | codegen + parser regen |
| **`.2.3`** | **retire the name-gate.** Remove `matches!(rule_name, "string_content_double" \| "string_content_single")` at all 6 sites + `fusibility_census.rs:60`, and declare `@lexical_token` in `return_annotation.ebnf` instead. | ⭐ the migration's own regression test: `generated/return_annotation_parser.rs` must keep `no-skip=10 of 20`, and the generated parser should be byte-identical. This is ruling (2) of the #208 directive applied. | parser regen |
| **`.2.4`** | **lint + gate.** Hard-error an unbounded / non-lexical closure; wire a standing gate proving both settings behave as declared, on a synthetic grammar AND a real one. | the gate itself, red-path verified. | gate run |

Sequencing rationale: `.2.1` is safe to land alone because a generate-honoured /
parse-unhonoured directive makes the **parser strictly more permissive than the
generator** — which is not a duality break (the generator's output still parses),
only incomplete strictness. The reverse order would break the duality gate, which is
exactly how `.3.11` died.

##### `.2.1` — declare + honour the GENERATE half

- **Status: `todo`** — the immediate next slice.

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

**`.1` (session #208) — the design leaf.** Re-runnable, deterministic
(`cmp`-clean across re-runs), read-only:

| artifact | proves |
|---|---|
| [`LEX-ADJACENCY-design.md`](LEX-ADJACENCY-design.md) | the adjudicated design |
| `artifacts/lex_adjacency/run_probes.sh` | driver for the (interior × exterior) matrix + the 7 engine sites |
| `artifacts/lex_adjacency/design_measurements.txt` | its captured output |
| `artifacts/lex_adjacency/probe_atomic_triggers.ebnf` | `-> $text` and `@transform` both close the interior seam |
| `artifacts/lex_adjacency/probe_exterior_boundary.ebnf` | …and both fuse the exterior seam against a keyword (`<D>timeunit02s;`) |
| `artifacts/lex_adjacency/closure_probe.py` | closure sizer over the `--dump-gen-ast` IR |
| `artifacts/lex_adjacency/closure_measurement.txt` | `time_literal` closure = **14 / 1,475** ⇒ static emission is affordable |

**`.3.11` (session #207) — the upstream diagnosis this tree was opened from.**
`docs/tasks/artifacts/sv_corpus_grad/time_literal_ws_diag/design_adjudication.txt`
(the four routes, each measured and each blocked) and
`.../generator_shape_probe.txt` (the three-shape generator A/B/C run).
⛔ Its "ROUTE 4 … DOES NOT EXIST" verdict is **superseded by `.1`** — the routes it
measured were each genuinely blocked, but a fifth (the inferred-atomicity trigger)
was never probed, and the parse-half claim did not check grammars other than SV.
The artifact is retained verbatim as measured evidence; the correction lives here
and in the design doc, not by rewriting it.
