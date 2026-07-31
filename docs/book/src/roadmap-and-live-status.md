# Roadmap and Live Status

PGEN uses two different but connected planning surfaces:

- the living roadmap,
- the live status claim.

Understanding the difference between them is important, because they answer different questions.

## The Roadmap Answers "What Are We Building Toward?"

The roadmap is the forward-looking contract for the project. It captures:

- mission and doctrine,
- execution preferences,
- retained strategic decisions,
- deferred backlog ordering,
- parser-family steering,
- the current preferred next moves.

In other words, the roadmap is where PGEN explains how it wants to advance.

## The Live Status Answers "Where Are We Right Now?"

Every parser family carries exactly one hand-authored status claim, `claimed_status`, held in the
done-bar register:

```
rust/test_data/grammar_quality/done_bar_family_register_v0.json
```

**This chapter is the human-readable view of that register**, and a gate holds the two equal — see
*How this page stays true* below.

### Why the claim lives in a register, not a prose file

Until `LIVE-MEANS-LIVE`, the claim was a table cell in a root-level Markdown file called
`LIVE_ACHIEVEMENT_STATUS.md`. That file was measured at **1 547 057 bytes, of which 94.7 % was a
dated changelog** — 856 accumulated tracker notes wrapped around 14 rows of actual status. The file
named *live* had become a museum.

The cause was structural rather than anybody's discipline: a free-form Markdown file has no schema
and nothing bounded what could accrete around a value that is one word long. The fix was to move the
value into a field a schema defines. **A JSON field cannot accumulate 856 tracker notes.**

⭐ The claim stayed **hand-authored** through that move, deliberately. It is one arm of a two-arm
check: a human writes what PGEN *claims*, the family-status gates compute what is *true* from proof
surfaces, and a mismatch fails the gate. Generating the claim from the gates would make the
comparison pass by construction and it could never fail again.

## The Status Vocabulary

The ladder has six labels, and it is deliberately stricter than casual status language:

`Done` · `Provisional (ceiling)` · `Provisional (corpus pending)` · `Mostly Done` · `In Progress` ·
`Not Started`

The rules behind them — the three-leg `Done` bar, why `Provisional` **ships** and is always
qualified, and why `ceiling` is hard to claim on purpose — are in
[Quality and Closure Model](quality-and-closure-model.md#what-done-means--the-three-leg-bar). They
are not restated here, so there is no second copy to drift.

## Current Family Status

<!-- LIVE-STATUS-SNAPSHOT:BEGIN — held equal to done_bar_family_register_v0.json by
     scripts/check_published_version_currency.sh (PUBLISHED-VERSION-CURRENCY). Edit the register
     first; this table is the published view of it, never the source. -->

| Family | Claimed status | Language owner | Leg-3 corpus surface |
|---|---|---|---|
| `systemverilog` | Mostly Done | external-standard | *(none declared)* |
| `systemverilog_preprocessor` | Provisional (corpus pending) | external-standard | *(none declared)* |
| `vhdl` | Provisional (corpus pending) | external-standard | *(none declared)* |
| `regex` | In Progress | external-standard | *(none declared)* |
| `return_annotation` | Mostly Done | pgen | *(none declared)* |
| `rtl_frontend` | Mostly Done | unadjudicated | *(none declared)* |
| `rtl_const_expr` | Mostly Done | unadjudicated | *(none declared)* |
| `ebnf` | In Progress | pgen | *(none declared)* |
| `json` | Mostly Done | unadjudicated | *(none declared)* |
| `semantic_annotation` | Mostly Done | pgen | *(none declared)* |

<!-- LIVE-STATUS-SNAPSHOT:END -->

Reading the table honestly:

- **No family is `Done`, and that is the current truth, not an oversight.** Leg 3 of the bar — an
  officially-recognized external corpus, asserted as a pass — has **no declared surface for any
  family yet**, so `Done` is unreachable across the board. Wiring the first one is `DONE-BAR.3`.
- **`language owner` decides which `Provisional` qualifier a family can ever hold.** `pgen` ⇒
  `(ceiling)`, because no third-party corpus for PGEN's own languages exists or ever will;
  `external-standard` ⇒ `(corpus pending)`, because one exists in the world and wiring it is
  outstanding work.
- **`unadjudicated` is not a shrug — it blocks.** Three families delimit a *subset* of somebody
  else's standard (`rtl_frontend` and `rtl_const_expr` of IEEE 1800, `json` of ECMA-404/RFC 8259),
  so whether a corpus for the full standard counts is a genuine open ruling that `DONE-BAR.3` owes.
  A family-status gate asked to judge one of these **refuses** rather than guessing, because the
  comfortable label is the one that closes the row.

### Grammars that are not families

`grammars/*.ebnf` holds 17 tracked grammars; 10 are families. The other 7 each carry a recorded
disposition in the same register, because a grammar in neither list **blocks the audit** rather than
being silently invisible:

| Grammar | Disposition |
|---|---|
| `builtin_return_annotation`, `builtin_semantic_annotation` | `bootstrap_contract` — bootstrap-safe contracts that break the annotation-parser cycle |
| `systemverilog_2017_lrm_extracted`, `systemverilog_2023_lrm_extracted`, `verilog_2005_lrm_extracted` | `lrm_extraction_input` — machine-extracted LRM grammar text, consumed as input |
| `systemverilog_lrm_profiled_generated` | `derived_artifact` — generated from the extractions, not authored |
| `systemverilog_lrm_profiled_wrapper` | `lrm_extraction_harness` — an `include(…)` wrapper over the extractions |

The admission rule is pinned to an independent source: a grammar is a family **iff**
`rust/src/parser_registry.rs` ships a registered generated parser for it, the one exception being the
two `builtin_*` contracts. See
[Quality and Closure Model → Why the roster is derived this way round](quality-and-closure-model.md#why-the-roster-is-derived-this-way-round).

## How This Page Stays True

Three mechanisms, in the order they fire:

1. **`scripts/check_published_version_currency.sh`** (the `PUBLISHED-VERSION-CURRENCY` doctrine, run
   by `scripts/check_doctrines.sh` on every commit) holds the *Current Family Status* table above
   equal to the register's `claimed_status`, family for family. An edit to one without the other
   fails the commit. It holds `PGEN_USER_GUIDE.md`'s published regex status to the same register by
   the same rule.
2. **The three family-status gates** (`regex_parser_family_status_gate`,
   `sv_parser_family_status_gate`, `vhdl_parser_family_status_gate`) compute each family's status
   from proof surfaces and fail when the register's claim disagrees. That is the two-arm check.
3. **`bash scripts/audit_done_bar.sh`** audits every leg of the bar for every family, derives its
   roster from `grammars/*.ebnf`, and refuses rather than guessing when it cannot judge.

## Phase Progress vs Family Maturity

These are separate axes, and conflating them is the usual reading error. A roadmap phase can be
advancing while a parser family it touches is not closed, and a closed family stays closed while
roadmap work continues elsewhere. The same proof-first doctrine applies to every family; what
differs between them is how much of the proof surface has landed, not the standard being applied.

Roadmap phase progress is tracked in
`docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`; family maturity is the table above.

## How To Read Current PGEN Status

1. This chapter for the per-family snapshot.
2. `docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md` for the rationale and next steps behind it.
3. The family-specific contracts and gates when the question is about one concrete parser surface.

For platform lanes broader than one family, the main roadmap points to dedicated side roadmaps — the
linter-enablement lane and the compiler-and-elaborator workbench lane are current examples.

## Primary Source Docs

- `rust/test_data/grammar_quality/done_bar_family_register_v0.json` — the status claim itself
- `docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`
- `docs/reference/PGEN_LINTER_ENABLEMENT_ROADMAP.md`
- `docs/reference/PGEN_COMPILER_ELABORATOR_ENABLEMENT_ROADMAP.md`
- `README.md`
