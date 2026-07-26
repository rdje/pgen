# ANNOTATION-PLACEMENT: every semantic annotation accepted in BOTH placements, and NEVER silently dropped

## Metadata

- Tree ID: `ANNOTATION-PLACEMENT`
- Status: `active` (opened 2026-07-26, session #208, by **direct director directive**)
  — **design-first; no code until `.1` is adjudicated.**
- Family / slice-id prefix: `PGEN-ANNOTATION-PLACEMENT-<NNNN>`
- Roadmap lane: cross-cutting **EBNF expressiveness** — the meta-capability that
  gates every other capability axis
  ([[project_ebnf_steers_the_engine_at_full_granularity]],
  [[project_horizon_universal_parser]]).
- Created: `2026-07-26`
- Owner: repo-local workflow
- Parser-AGNOSTIC ([[feedback_ast_pipeline_parser_agnostic]]).

## The directive (director, 2026-07-26, session #208, verbatim)

> *"By default semantic annotations are rule-based, that is defined just before a
> rule. some semantic annotations, if not all (can't say) are also accept in-line,
> intra-sequence of rule's references. What I want and what future languages parser
> will need is the for any semantic annotation to be accepted before-rule and
> intra-sequence in branches or in-branch. They shall all be accepted in these 2
> places. PGEN engine may support their occurrence in either locations or places,
> but syntaxically both locations shall be fully supported. If PGEN doesn't know to
> deal with a particular semantic annotation in one type of location, it can either
> emit a warning message and continue or balk and stop with a clear message of why
> it stop."*

Two separable requirements:

1. **UNIVERSAL SYNTACTIC ACCEPTANCE.** *Every* semantic annotation is accepted in
   *both* placements — before-rule and intra-sequence (in-branch / mid-sequence).
   No annotation is placement-restricted at the syntax level.
2. **NO SILENT DROP (the diagnostic contract).** The engine need not *implement*
   every (annotation × placement) cell. But where it does not, it must **either
   warn and continue, or halt with a clear message naming why** — never accept the
   grammar, report success, and do nothing.

Requirement 2 is the safety-critical half: a silently-ignored annotation makes a
grammar *look* like it enforces something it does not.

## The measured current state (`.1`, session #208)

Re-run: `bash docs/tasks/artifacts/annotation_placement/run_placement_probe.sh`
(read-only, debug `ast_pipeline` only). The probe puts the **same** `@emit_fact` in
all three placements.

| placement | syntactically accepted | reaches the generated parser | fires at parse time | diagnostic when it does not |
|---|---|---|---|---|
| **before-rule** | ✅ | ✅ | ✅ | n/a |
| **branch-start (in-branch)** | ✅ | ✅ | ✅ — landed by `INLINE-ACTIONS.2` (`PGEN-INLINE-ACTIONS-0002`) | n/a |
| **mid-sequence (intra-sequence)** | ✅ | ⛔ **ABSENT** (`marker_mid` → **0** occurrences) | ⛔ no | ⛔ **NONE — lint exit 0, generate exit 0, zero output** |

⇒ **Requirement 1 is already satisfied for `@emit_fact`** (the meta-grammar
`grammars/ebnf.ebnf:117-130` admits all three, and the frontend parses them).
⇒ **Requirement 2 is VIOLATED**, in exactly the shape the director named: the
mid-sequence directive is extracted into `branch_mid_sequence_semantic_annotations`
and then never compiled by `compile_semantic_runtime_annotations`, so it is
**silently discarded with a success exit code**.

⚠️ **Scope honesty — the matrix is measured for ONE annotation, not all.** The probe
covers `@emit_fact`. Whether every *other* annotation (`@predicate`, `@open_scope`,
`@close_scope`, `@transform`, `@sample`, `@profiles`, `@branch_policy`, `@priority`,
`@associativity`, `@quantified_separator`, `@whitespace_sensitive`,
`@default_profile`, `@profile_alias`, `@import_from_library`, …) is accepted and
honoured in both placements is **UNMEASURED**. Enumerating that matrix is `.1`'s
job — and per this session's standing lesson, it must be *measured*, not inferred
from prose.

⛔ **A stale-claim correction this leaf makes:** earlier in session #208 (commit
`3b8ac3e8`) I stated that branch-start `EmitFact` "hits a no-op arm
(`ast_based_generator.rs:3092-3099`)". That was true at `INLINE-ACTIONS.1` (2026-06-09)
and was **fixed by `INLINE-ACTIONS.2`**; I quoted the tree's `.1`-era table instead of
re-measuring. Corrected here and in the affected records. (Third instance in one
session of repeating a doc claim without re-running it — see the Decisions section.)

## ⭐ A SECOND annotation family with the same placement question — LEXICAL annotations

The directive says "semantic annotations", but PGEN has a **second** annotation
family with an identical placement story: the **lexical** annotations
`[> … ]` / `[>! … ]` ([[project_lexical_annotations_fourth_pillar]], the 4th pillar).
Measured this session
(`artifacts/annotation_placement/lexical_annotation_reach.sh`):

| | finding |
|---|---|
| **consumption** | ⛔ **GENERATOR-ONLY** — parser codegen `ast_based_generator.rs` = **0** hits vs stimuli generator = **17**. The constraint shapes rendered output; it does not *enforce* anything at parse time. |
| **inline form** | ⛔ **designed 2026-06-06 (with the director), never implemented** — an inline lexical annotation binds the **PRECEDING** item (mirroring inline semantic, which binds the following one). This is the per-seam placement the #208 directive asks for. |
| **meta-grammar** | ⛔ **absent from `grammars/ebnf.ebnf`** — `annotation_list := semantic_annotation+` (`:74`) admits only `@`-forms, so PGEN's own meta-grammar cannot describe a grammar using a feature PGEN ships. `systemverilog_preprocessor.ebnf` (12 uses) is not parseable by `ebnf.ebnf`. |

⇒ **This tree's matrix must cover BOTH families** (semantic × lexical) × (before-rule
× intra-sequence). The lexical family is the one with a real, shipped consumer and a
designed-but-unbuilt inline form, so it is the more urgent column.

⇒ The meta-grammar gap is a **self-hosting defect** — a notation PGEN ships but
cannot describe in its own EBNF — and is called out for director routing rather than
silently absorbed here.

### ⚠️ ROOT-CAUSED (same session, after the director flagged the result as surprising)

The claim was first derived structurally (no `[>` production in `ebnf.ebnf`). Run to
ground in `rust/src/ebnf_frontend.rs:37-43`, it is **confirmed and ALREADY KNOWN to
the code** — and it is *less* alarming than the bare statement implies. Verbatim:

> *"LEXICAL-ANNOTATIONS.3c — when a grammar declares before-rule lexical
> follow-restriction directives (`[> …]` / `[>! …]`), the generated `EbnfParser`
> (built from the seed `grammars/ebnf.ebnf`) **does not yet know the construct, so
> its cross-check would spuriously fail.** Gate the soft cross-check on their
> presence, exactly as `has_inline_semantic_annotations` already does for inline
> semantic annotations."*

**Correct severity, stated honestly:**

- ✅ **Production parsing is NOT broken.** The authoritative path is the hand-written
  `scan_top_level_rules` + `convert_scanned_rule`; the generated `EbnfParser` is only
  a **soft cross-check** (a verification step layered on top).
- ⛔ **But the cross-check DISABLES ITSELF exactly where it would be most valuable.**
  `ebnf_frontend.rs:61` — `if !has_inline_semantic_annotations && !has_lexical_annotations`
  — so self-hosting verification is skipped for **precisely the two newest, least-verified
  constructs**: inline semantic annotations and lexical annotations. Coverage is silently
  strongest where the language is oldest and absent where it is newest.
- ⛔ **`grammars/ebnf.ebnf` describes a STRICT SUBSET of the language PGEN accepts.**
  That matters beyond the cross-check, because the `ebnf` meta-grammar has its own
  shipped book billed as *"the grammar-author's reference for the EBNF input
  language"* — so the reference under-documents the real surface.

⭐ **Note the shape: this is the SAME failure pattern as the silent-drop finding above** —
a safety mechanism that quietly stands down rather than reporting. Both belong to this
tree'"'"'s diagnostic contract: *a check that cannot run must say so, not return green.*

## Relationship to `INLINE-ACTIONS`

[`INLINE-ACTIONS`](INLINE-ACTIONS.md) owns *making inline ACTION directives fire*:
`.2` landed branch-start, and `.3` (mid-sequence wiring) is `deferred`. **This tree
is not a duplicate** — it owns the *general contract* over the whole
(annotation × placement) matrix plus the **diagnostic guarantee**, which
`INLINE-ACTIONS` never covered (its Non-Goals explicitly park mid-sequence, and
nothing there requires a diagnostic when a placement is unsupported).

Division of labour:
- **`ANNOTATION-PLACEMENT`** — the matrix, the contract, and *"never silently drop"*.
- **`INLINE-ACTIONS.3`** — the mid-sequence *wiring* for action directives, one cell
  of that matrix.

⭐ The diagnostic contract is deliverable **independently of, and far more cheaply
than**, any wiring: it converts today's silent trap into a loud, actionable message
without implementing a single new cell.

## Leaves

### `.1` — Measure the full (annotation × placement) matrix + design the contract

- **Status: `todo`** — read-only + design.
- Enumerate every semantic annotation from `semantic_directive_registry.rs`
  (`DIRECTIVES`) and probe each in both placements. Output: a per-cell verdict —
  **honoured** / **accepted-but-inert** / **dropped** / **rejected**.
- ⚠️ The dangerous class is **accepted-but-inert** and **dropped**: both look like
  success today.
- Design the diagnostic contract: which cells warn-and-continue vs halt; the exact
  message shape (must name the annotation, the placement, and *why*); and where it
  lives (`--lint-grammar` finding, codegen diagnostic, or both — noting that
  `--lint-grammar` already has the severity vocabulary and is the natural home).
- Must respect [[project_capability_growth_is_zero_cost_and_neutral]]: the check is
  a **codegen/lint-time** diagnostic, so a bare parse pays nothing by construction.

### `.2` — Land the diagnostic contract (no silent drop)

- **Status: `todo`**, blocked on `.1`. The cheap, high-value half of the directive.
- Every unsupported cell produces a warning or a hard error naming annotation +
  placement + reason. Red-path verified (a probe grammar per class).
- Zero behaviour change for grammars whose annotations are all honoured
  (byte-identical parsers).

### `.2a` — Make a SKIPPED self-hosting cross-check LOUD (director-approved 2026-07-26)

- **Status: `todo`** — director ruled YES on this item. Small, concrete, and the
  cheapest instance of this tree's contract.
- **The defect (measured, `LEX-ADJACENCY`/`ANNOTATION-PLACEMENT` root-cause):**
  `rust/src/ebnf_frontend.rs:61` gates the generated-`EbnfParser` cross-check on
  `if !has_inline_semantic_annotations && !has_lexical_annotations`, so self-hosting
  verification **silently stands down for precisely the two newest, least-verified
  constructs** — and reports success either way.
- **The fix (tier: diagnostics, not engine semantics):** when the cross-check is
  skipped, say so — naming the grammar and the construct that caused the skip
  (e.g. `self-hosting cross-check SKIPPED for 'systemverilog_preprocessor': the
  seed ebnf.ebnf does not model lexical annotations`). Route through the shared
  trace/diagnostic surface at a severity that is **not maskable by verbosity**
  (the `DIAG-SEVERITY` doctrine already enforces that property).
- **Acceptance:** a grammar WITH the constructs emits the notice; a grammar without
  them is byte-identical and silent; the notice names grammar + construct + reason.
  Verify on `systemverilog_preprocessor.ebnf` (12 lexical annotations) as the
  positive control and any annotation-free grammar as the negative.
- **Cost model** ([[project_capability_growth_is_zero_cost_and_neutral]]): frontend
  diagnostic only — zero parse-time and zero generated-parser impact.
- ⚠️ Needs a debug `ast_pipeline` rebuild to verify; not a paper leaf.

### `.4` — Meta-grammar completeness: `ebnf.ebnf` must describe the language PGEN accepts (director-approved 2026-07-26)

- **Status: `todo`** — director ruled YES on this item.
- **The defect (measured):** `grammars/ebnf.ebnf` has **no** production for the
  lexical-annotation notation `[> … ]` / `[>! … ]`; `annotation_list :=
  semantic_annotation+` (`:74`) admits only `@`-forms. So PGEN's own meta-grammar
  **cannot describe a grammar that uses a feature PGEN ships** —
  `systemverilog_preprocessor.ebnf` uses it 12×.
- **Why it is worth doing (two payoffs, not one):**
  1. it **restores self-hosting verification** for those grammars — `.2a` makes the
     gap loud, this leaf makes it *go away*, so the cross-check runs again;
  2. it fixes the **`ebnf` book's under-documentation** — that book is billed as
     *"the grammar-author's reference for the EBNF input language"* while the
     meta-grammar it documents is a strict subset of the real surface.
- **Scope note:** the inline lexical form (binding the PRECEDING element) is part of
  the same surface and should be modelled here even if the engine does not yet honour
  it — with `.2`'s diagnostic contract making the unhonoured cell loud rather than
  silent. That is the correct order: **describe the language, then report honestly
  about what is wired.**
- **⛔ Do NOT confuse with the 27 unreachable productions** found by
  `LANG-CAPABILITY-AUDIT.1` — that is the inverse defect (declared-but-unwired) and
  is adjudicated there. This leaf is *shipped-but-undeclared*. Both are meta-grammar
  fidelity; they are opposite directions and must not be merged.

### `.3` — Close the matrix cells that should be honoured

- **Status: `todo`**, blocked on `.1`'s verdicts. Per-cell leaves, prioritized by
  what real languages need. Consumes / coordinates with `INLINE-ACTIONS.3`.
- ⭐ **Known first consumer:** the per-seam lexical-adjacency directive
  ([`LEX-ADJACENCY`](LEX-ADJACENCY.md)) — whose surface the director reopened
  precisely because it must be expressible intra-sequence.

## Acceptance Criteria (tree)

- Every semantic annotation is syntactically accepted before-rule and
  intra-sequence.
- **No annotation is ever silently ignored** — every unsupported cell warns or halts
  with a message naming the annotation, the placement, and the reason.
- The matrix is a *measured*, re-runnable artifact, not a prose claim.
- Diagnostics are codegen/lint-time ⇒ zero bare-parse cost; grammars using only
  honoured cells are byte-identical.

## Evidence

- `docs/tasks/artifacts/annotation_placement/run_placement_probe.sh` — re-runnable driver
- `docs/tasks/artifacts/annotation_placement/placement_measurement.txt` — captured output
- `docs/tasks/artifacts/annotation_placement/placement_probe.ebnf` — the 3-placement probe

## Decisions

- **The diagnostic contract ships before the wiring.** Making a silent failure loud
  is independent of implementing any cell, is far cheaper, and removes the
  correctness trap immediately. Wiring follows, per-cell, on demand.
- **Standing lesson reinforced (3rd instance in session #208).** A doc's description
  of engine behaviour goes stale; `INLINE-ACTIONS`' `.1`-era table said branch-start
  effects were inert, `.2` fixed it, and I repeated the stale line. **Re-measure
  before citing engine behaviour** — see [[project_ebnf_steers_the_engine_at_full_granularity]]
  and the retracted-simulator-claim precedent in
  [[feedback_sv_strict_lrm_compliance_default]].
