# SVPP-EXPANSION — the SystemVerilog preprocessing / expansion stage (the missing front-end link for NEXSIM)

## Metadata

- Tree ID: `SVPP-EXPANSION`
- Status: `proposed` (own-now / **build-later**, director 2026-06-08). Sequenced STRICTLY AFTER the locked
  program (all existing parsers cert-coverage clean + `UNKNOWN`=0). Activates (frontier `.1` SCOPING) when
  prioritized — not PNT-eligible until then.
- Family / slice-id prefix: `PGEN-SVPP-EXPANSION-<NNNN>`
- Roadmap lane: front-end workbench / downstream-consumer service (NEXSIM SV front-end) — composes the
  `systemverilog_preprocessor` (svpp) directive parse with the `systemverilog` (sv) parser
- Created: `2026-06-08`
- Owner: repo-local workflow
- Decision record: [[project_svpp_expansion_stage_for_nexsim]]
- Disciplines: [[feedback_research_grounded_sota_no_trial_and_revert]] (survey IEEE 1800 §22 +
  slang/Verible/Verilator preprocessing models BEFORE building), [[feedback_tools_first_no_guessing]],
  [[feedback_ast_pipeline_parser_agnostic]] (parser-agnostic where the svpp directive model is general),
  [[feedback_always_signoff_decisions]].

## The frame (why this tree exists)

PGEN's svpp parser parses SystemVerilog preprocessor *directives* into a typed AST (`` `define `` /
`` `ifdef `` / `` `include `` / macro bodies / condition exprs) and passes non-directive code through as
`non_directive_text`; it does **not** expand macros, resolve conditionals, or inline includes (no
substitution engine). The sv parser parses SV source directly and only *tolerates* a directive inline via
the opaque `compiler_directive` rule — it does **not** expand it either (`systemverilog.ebnf:178`). So the
two parsers do **not** compose into the classic preprocess→parse pipeline a downstream consumer expects;
an **expansion stage** must sit between them.

Real SystemVerilog cannot be elaborated without preprocessing (macros expand into arbitrary tokens;
conditionals select live code; includes pull in package/interface/typedef definitions). So for NEXSIM (the
primary SV+VHDL integration target — a simulator/elaborator-facing consumer), an un-expanded parse tree is
not a usable design representation. Per the director (2026-06-08): **PGEN ought to deliver the whole
package; the SV + SVPP front-end stack is not release-complete for NEXSIM without the expansion stage.**

The svpp + sv parsers remain valid deliverables AS PARSERS; the expansion stage is a SEPARATE deliverable —
a **transformation** over the svpp directive AST, not a parser — consistent with PGEN's "parsers from EBNF"
doctrine and its front-end-workbench / compiler-elaborator-enablement direction.

## Goal

Deliver a SystemVerilog preprocessing / expansion stage that, given SV source, produces **expanded,
sv-parseable source** (or an equivalent expanded token stream) by composing the svpp directive parse with
macro substitution, conditional resolution, and `` `include `` inlining — so the full SV front-end
(svpp parse → expand → sv parse) is a usable, trustworthy service for downstream consumers (NEXSIM first).

## Non-goals

- A new parser. This is a transformation/service over the EXISTING svpp directive AST + macro table.
- VHDL (no C-style preprocessor — SV-specific).
- Full elaboration / typing / binding (owned by the compiler-elaborator lane). Expansion stops at producing
  sv-parseable expanded source; semantic elaboration is downstream.
- Changing the svpp or sv grammars' accept sets (the expansion consumes svpp output and targets the existing
  sv parser).

## Acceptance criteria

- **Macro substitution:** object-like and function-like macros (with arguments), `` `" `` stringize,
  `` ` ` `` token-paste, nested/recursive expansion with a cycle/recursion guard; `` `undef `` honored;
  predefined/built-in macros (`` `__FILE__ `` / `` `__LINE__ `` / `` `line `` …) where in scope.
- **Conditional resolution:** `` `ifdef `` / `` `ifndef `` / `` `elsif `` / `` `else `` / `` `endif ``,
  arbitrarily nested; only the live branch survives.
- **`include` inlining:** path resolution (search list), recursion/cycle guard, line/file provenance.
- **Composition proof:** the expanded output re-parses cleanly through the `systemverilog` parser; round-trip
  + coverage proof; provenance/source-map preserved enough for downstream diagnostics (anchor fidelity).
- **Parser-agnostic where general:** the directive-AST→expansion machinery should reuse the svpp directive
  model generically where possible (not a one-off SV hack).
- **Lockstep on landing:** SV + SVPP integration contracts + release policy updated to document the front-end
  pipeline (svpp parse → expand → sv parse); book chapter; this tree; decision record.

## Task tree

- ID: `SVPP-EXPANSION.1`
  Status: `pending` (activates when the locked program is met)
  Goal: SCOPING + SOTA survey — pin the exact expansion model against IEEE 1800-2017 §22 (compiler
  directives) and the slang / Verible / Verilator preprocessing implementations; decide the architecture
  (transformation over the svpp AST → expanded source vs. token stream; where it lives in the pipeline;
  provenance/source-map carrier); enumerate the build leaves (`.2` macro substitution, `.3` conditional
  resolution, `.4` `include` inlining, `.5` integration + composition proof). Pure docs.
- ID: `SVPP-EXPANSION.2`–`.5`
  Status: `pending` (defined at `.1` SCOPING)
  Goal: the build — macro substitution (`.2`), conditional resolution (`.3`), `` `include `` inlining (`.4`),
  integration + expanded-output→sv-parse composition proof + contract/book lockstep (`.5`).

## Current frontier

- (none active) — this tree is `proposed` / build-later. It activates at `.1` SCOPING only after the locked
  program (all existing parsers cert-coverage clean + `UNKNOWN`=0) is achieved (director 2026-06-08).

## Decisions

- **(director 2026-06-08)** PGEN must deliver the SV expansion stage for NEXSIM; the SV+SVPP front-end is not
  release-complete without it. The svpp + sv parsers remain valid AS PARSERS; expansion is a separate
  transformation deliverable. See [[project_svpp_expansion_stage_for_nexsim]].
- **(director 2026-06-08, sequencing)** Build-later: sequenced strictly AFTER the locked program (every
  existing parser cert-coverage WIRED + clean + `UNKNOWN`=0). Do not pivot to building this until then.
- **(director 2026-06-09, ARCHITECTURE — go all-in PGEN-native)** The expansion stage is a **parse-tree
  transformation over the enhanced `svpp.ebnf` AST** — NOT a separate token-stream engine and NOT a separate
  grammar. RATIONALE (objective, no fundamental blocker found): the industry token-stream model exists to
  dodge one wall — *macros cross SV structural boundaries, so you can't preprocess over a parse tree of the
  SV grammar*. svpp's AST is **not** an SV syntax tree; it is a flat, source-ordered `pp_item*` **structured
  token/directive stream** (directives with internal structure + opaque `non_directive_text` runs), so it
  NEVER hits that wall (it makes no SV-structural assumptions). Walking svpp's `pp_item` list with a stateful
  macro table IS token-stream preprocessing, with the directive grammar recognized DECLARATIVELY (EBNF =
  single source of truth) instead of ad-hoc scanner code. Every hard case maps cleanly: boundary-crossing
  macro fragments + conditional-split constructs are text runs to substitute/keep/drop; token-paste/stringize
  + rescan/recursion are engine logic over the modeled macro-body atoms; balanced function-macro args are a
  context-free `macro_actual_args` rule; cross-boundary re-lexing (`` `HALF ``→`3.14` must not fuse with a
  following ` e2`) is handled by PGEN's **lexical-faithfulness pillar (LEXICAL-ANNOTATIONS)** — an area PGEN
  is AHEAD; provenance is structured (AST spans). The token-stream model's only genuine edges
  (performance / streaming / malformed-input tolerance) are NON-FUNCTIONAL — none a capability/correctness
  blocker, and all consistent with PGEN's parse-then-transform architecture + correctness-before-speed.
  GAINS going PGEN-native: directive grammar validated by the well-formedness linter + cert-coverage;
  expansion is a testable, proof-first transformation over a validated AST; structured provenance;
  architectural consistency (the transformation dual; the macro table is the same stateful left-to-right
  processing the semantic store already does). OBLIGATIONS (not blockers): `svpp.ebnf` must be a total/correct
  grammar over the directive+text surface (proven by cert-coverage/linter); macro-usage recognition + output
  must be lexically faithful (we have the pillar); full-AST performance on huge files is monitor-and-optimize-
  later. The `.1` SCOPING read of IEEE 1800 §22 + slang/Verible/Verilator now CONFIRMS no SV-specific corner
  breaks this + settles implementation details (output form, provenance carrier, token-paste/rescan corners)
  — it does NOT re-litigate the architecture.
  - **PROS/CONS (recorded 2026-06-09, balanced).** *Token-stream (industry) PROS:* streaming performance/low
    memory (single lexical pass, no full AST); conceptual simplicity (lexer + substitution loop);
    language-agnostic (grammar-blind directive filter); malformed-input tolerance; direct token-level
    differential alignment with slang/Verible/Verilator. *Token-stream CONS for PGEN:* ad-hoc directive
    recognition (NOT EBNF-validated → drifts from single-source-of-truth; linter/cert-coverage don't apply);
    a SECOND separate technology beside the EBNF/AST pipeline (more surface, inconsistent); flatter
    provenance; no reuse of the existing svpp grammar/AST. *Parse-tree (PGEN-native) PROS:* EBNF = single
    source of truth (directive grammar validated by linter + cert-coverage + well-formedness); proof-first /
    testable transformation over a validated AST (shape contracts, round-trip, deterministic gates);
    structured provenance (AST spans — what downstream linters/elaborators need); architectural consistency
    (transformation dual; macro table reuses semantic-store-style stateful processing); lexical-faithfulness
    already solved (LEXICAL-ANNOTATIONS). *Parse-tree CONS:* heavier (build+walk AST vs streaming —
    mitigable, correctness-first); a grammar-totality obligation on svpp.ebnf (provable via cert-coverage;
    `non_directive_text` catch-all is near-total); the same subtle rescan/paste semantics must be encoded
    (true of BOTH models). **WEIGHING:** every token-stream pro is NON-FUNCTIONAL (performance/simplicity/
    agnosticism/tolerance/reference-alignment) — none is a capability the parse-tree lacks; every parse-tree
    pro is a SIGNOFF-QUALITY/doctrine win (PGEN's differentiator). The parse-tree cons are real but
    non-functional + mitigable. ⇒ parse-tree, all-in.

## Open questions

- **Grammar home: enhance `svpp.ebnf` vs. a separate "expression parser" EBNF? (director 2026-06-08).**
  RECORDED ANALYSIS + LEAN (to confirm at `.1` with SOTA): expansion is ~90% TRANSFORMATION LOGIC over the
  structure svpp ALREADY parses — macro table (`pp_define`/`macro_formals`/`macro_body` incl. stringize +
  token-paste atoms), conditional STRUCTURE (`pp_conditional`), and `` `include `` paths need NO grammar
  work; substitution semantics, conditional evaluation, and include inlining are engine logic, NOT grammar.
  The ONE genuine grammar gap is recognizing **macro-usage sites** (`` `NAME `` / `` `NAME(args) ``) inside
  the body/non-directive text — today buried in opaque `non_directive_text`. That is a LEXICAL-level
  enhancement to `svpp.ebnf` (split `non_directive_text` into `(plain_text | macro_usage)*` + add a
  `macro_actual_args` rule), NOT a full SV parse and NOT a separate grammar. **LEAN: enhance `svpp.ebnf`; a
  separate "expression parser EBNF" is almost certainly unnecessary** because (1) the macro/usage model is
  lexical and belongs in svpp's domain (EBNF-single-source-of-truth), and (2) standard SV preprocessor
  conditionals (`` `ifdef ``/`` `ifndef ``/`` `elsif ``) take a single **macro identifier**, NOT a C-`#if`
  boolean expression (IEEE 1800-2017 §22.6) — so there is no expression language to parse. ⚠️ SCOPING CHECK:
  svpp's current `condition_expr` models a RICHER form (`||`/`&&`/`!`/`?:` atoms) than the standard —
  confirm at `.1` whether that is a deliberate tool-extension or over-modeling.
- **Transformation model (the deeper fork under the grammar question): parse-tree vs. token-stream.** A
  parse-tree transformation walks the (enhanced) svpp AST — PGEN-native, EBNF-as-source-of-truth, reuses
  svpp (the LEAN). A token-stream transformation is the industry SOTA (slang / Verible / Verilator
  preprocess at the token level, because macros can expand across structural boundaries and preprocessing
  is inherently lexical). `.1` SCOPING decides this with IEEE 1800 §22 + the slang/Verible/Verilator
  implementations read FIRST (research-grounded discipline) — do not pick speculatively.
- Output form: expanded source TEXT (re-fed to the sv parser) vs. an expanded token stream handed to the sv
  parser directly — decide at `.1` with provenance/source-map requirements in mind (tied to the
  transformation-model fork above).
- Reuse boundary: how much of the substitution/resolution machinery can be parser-agnostic (a general
  directive-AST→expansion service) vs. SV-specific.

## Blockers

- Sequencing gate (not a defect): build is deferred until the locked program (all existing parsers
  cert-coverage clean + `UNKNOWN`=0) is met. Unblock condition: locked program achieved + director activates.

## Verification log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-08` | (tree creation) | director directive captured + tool-verified gap (svpp parses directives but does not expand; sv does not expand, tolerates `compiler_directive` inline; no composition stage) | tree registered (`PGEN-SVPP-EXPANSION-0001`, docs-only); build deferred behind the locked program |

## Commit log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| (tree creation) | `PGEN-SVPP-EXPANSION-0001` | tree + decision record created; registered in `docs/TASK_TREE.md` Proposed Task Trees; sequenced after the locked program |

## Changelog

- `2026-06-08` — tree created (`proposed`, own-now/build-later) per the director directive; decision record
  [[project_svpp_expansion_stage_for_nexsim]] written; sequenced strictly after the locked program.
