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

⭐ **A label is not a distance.** The table above says where each family sits on the ladder; it
says nothing about how far it still has to climb, which is the question a reader actually has. That
distance — the state of each leg of the bar, the corpus pass counts against a named denominator, and
the remaining work — is published per family, derived, on that family's own book page. The first is
[SystemVerilog → Status and Distance to `Done`](../../systemverilog_parser_book/src/status-and-distance.md),
produced by `scripts/report_family_status.sh`. **All ten parser books now carry that page**, and
the `FAMILY-STATUS-CURRENCY` doctrine (`scripts/check_family_status.sh`, run by
`scripts/check_doctrines.sh` on every commit) re-derives all ten blocks and refuses when a
published one has drifted — so a number on those pages cannot go stale without failing a commit.
The cross-family view of the same derivation is [below](#the-distance-behind-the-labels).

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

### The Distance Behind The Labels

⛔ **DERIVED — never edited by hand.** This section is produced by
`bash scripts/report_family_status.sh --cross-family --markdown`, the same pass that produces
each family's own status page, and the `FAMILY-STATUS-CURRENCY` doctrine re-derives it on every
commit and fails that commit when what is published here differs. To change a number here, fix
the tree it is derived from.

<!-- BEGIN DERIVED: scripts/report_family_status.sh --cross-family --markdown -->

**10 families, one page — the distance behind the labels above.** Every figure here is derived by `scripts/report_family_status.sh` in the SAME pass that produces each family's own *Status and Distance to `Done`* page, so the two surfaces cannot disagree about a number; `FAMILY-STATUS-CURRENCY` (`scripts/check_family_status.sh`) re-derives and diffs both on every commit.

**0 of 10 families meet the three-leg `Done` bar, and 0 of the 30 legs across the roster are met.**

| Family | Claimed status | Legs met | 1 | 2 | 3 | Trees owning its remaining work |
|---|---|---:|:-:|:-:|:-:|---:|
| [`ebnf`](../../ebnf_parser_book/src/status-and-distance.md) | In Progress | 0 of 3 | ⚠️ | ⛔ | ⛔ | 1 |
| [`json`](../../json_parser_book/src/status-and-distance.md) | Mostly Done | 0 of 3 | ⚠️ | ⛔ | ⛔ | 1 |
| [`regex`](../../regex_parser_book/src/status-and-distance.md) | In Progress | 0 of 3 | ⚠️ | ⛔ | ⛔ | 5 |
| [`return_annotation`](../../return_annotation_parser_book/src/status-and-distance.md) | Mostly Done | 0 of 3 | ⚠️ | ⛔ | ⛔ | 1 |
| [`rtl_const_expr`](../../rtl_const_expr_parser_book/src/status-and-distance.md) | Mostly Done | 0 of 3 | ⚠️ | ⛔ | ⛔ | 1 |
| [`rtl_frontend`](../../rtl_frontend_parser_book/src/status-and-distance.md) | Mostly Done | 0 of 3 | ⚠️ | ⛔ | ⛔ | 0 |
| [`semantic_annotation`](../../semantic_annotation_parser_book/src/status-and-distance.md) | Mostly Done | 0 of 3 | ⚠️ | ⛔ | ⛔ | 1 |
| [`systemverilog`](../../systemverilog_parser_book/src/status-and-distance.md) | Mostly Done | 0 of 3 | ⚠️ | ⛔ | ⛔ | 4 |
| [`systemverilog_preprocessor`](../../systemverilog_preprocessor_parser_book/src/status-and-distance.md) | Provisional (corpus pending) | 0 of 3 | ⚠️ | ⛔ | ⛔ | 1 |
| [`vhdl`](../../vhdl_parser_book/src/status-and-distance.md) | Provisional (corpus pending) | 0 of 3 | ⚠️ | ⛔ | ⛔ | 0 |

Leg 1 is the stimuli-generator proof, leg 2 is *gates green and invoked* — every covering gate green AND actually invoked by something — and leg 3 is an officially-recognized external corpus passing. ✅ met · ⚠️ unproven · ⛔ unmet. **Why** a leg reads as it does is on that family's own page, one click away; this table is the comparison, not the explanation.

**Corpus.** 3 of the 10 families have a corpus count at all. The rows below are that family's own denominator — there is no roster-wide corpus figure, and the `NOT MEASURED` rows are exactly what is missing, each naming the leaf that owes it.

| Family | Corpus rows | Passing | Known defects | What the number rests on |
|---|---:|---:|---:|---|
| `ebnf` | ⛔ NOT MEASURED | ⛔ | ⛔ | the language is authored by PGEN itself, so no third-party corpus exists or ever will. NOT a measurement gap -- leg 3 is unreachable by construction and the qualifier is `Provisional (ceiling)`. |
| `json` | ⛔ NOT MEASURED | ⛔ | ⛔ | whether a corpus for the full standard counts against this SUBSET language is an open ruling; until it is made, a corpus count would be measured against a denominator nobody has agreed to. Owed by `DONE-BAR.3`. |
| `regex` | 2 189 | 1 879 (85.8 %) | 262 | every pattern spec pcre2test 10.47 declares in testdata/testinput2 that the normalizer can pair soundly with its recorded verdict, counted by `regex_corpus_bundle/scripts/normalize_pcre2_compile_oracle.py` and held current by `bash scripts/check_corpus_census.sh regex` |
| `return_annotation` | ⛔ NOT MEASURED | ⛔ | ⛔ | the language is authored by PGEN itself, so no third-party corpus exists or ever will. NOT a measurement gap -- leg 3 is unreachable by construction and the qualifier is `Provisional (ceiling)`. |
| `rtl_const_expr` | ⛔ NOT MEASURED | ⛔ | ⛔ | whether a corpus for the full standard counts against this SUBSET language is an open ruling; until it is made, a corpus count would be measured against a denominator nobody has agreed to. Owed by `DONE-BAR.3`. |
| `rtl_frontend` | ⛔ NOT MEASURED | ⛔ | ⛔ | whether a corpus for the full standard counts against this SUBSET language is an open ruling; until it is made, a corpus count would be measured against a denominator nobody has agreed to. Owed by `DONE-BAR.3`. |
| `semantic_annotation` | ⛔ NOT MEASURED | ⛔ | ⛔ | the language is authored by PGEN itself, so no third-party corpus exists or ever will. NOT a measurement gap -- leg 3 is unreachable by construction and the qualifier is `Provisional (ceiling)`. |
| `systemverilog` | 16 336 | 5 899 (36.1 %) | 174 | every row of the SystemVerilog corpus adjudication manifests over stimuli/sv/subs + stimuli/sv/uvm, counted by `stimuli/sv/corpus_verdict_coverage.py` and held current by `bash scripts/check_sv_corpus_denominator.sh` |
| `systemverilog_preprocessor` | ⛔ NOT MEASURED | ⛔ | ⛔ | the language is standardized, so a corpus exists in the world, but none is wired into this repository. An UNMET leg, never an inapplicable one. Owed by `CORPUS-GRAD-ALL.1`. |
| `vhdl` | 13 720 | 4 370 (31.9 %) ⚠️ raw | ⛔ n/a | every .vhd / .vhdl file under the ten vendored VHDL corpora in stimuli/vhdl/subs, counted by `stimuli/run_external_corpus.sh vhdl` and held current by `bash scripts/check_corpus_census.sh vhdl`. ⚠️ RAW parse outcomes: no answer key, so the pass share is NOT a conformance share and no defect count exists. Adjudication owed by `CORPUS-GRAD-ALL.2` |

⛔ in a numeric cell means **NOT MEASURED**: there is no count. That is not the same as a count of zero, and it is not the same as nothing being wrong — it is the absence of an instrument, which is itself the work the named leaf owes.

**What remains, across the whole index.** The task-tree index carries 113 trees and **62 are OPEN**. Every row is adjudicated to exactly one bucket — the producer refuses to render at all if a row carries no attribution or an attribution names no row — so the buckets below are a partition, and they sum to that total.

| Where the open work sits | Open trees | What that bucket is |
|---|---:|---|
| Attributed to one family | 15 | one of the `families` keys of done_bar_family_register_v0.json - this tree's remaining work is work on THAT parser |
| `cross-cutting` | 26 | engine, codegen, annotation, stimuli or corpus-doctrine work that can gate ANY family. Counted for every family, but reported as a COUNT rather than inlined, so a family's own list stays readable |
| `process` | 14 | governance, continuity, documentation, ops and repo hygiene. Real work, but not on any parser's path to `Done` |
| `future-family` | 4 | a parser family PGEN does not ship yet (Phase S and later). No current family's bar depends on it |
| `platform` | 3 | platform expansion around the parsers - a linter, an elaborator workbench, MCP control - rather than a parser |
| **Total** | **62** | of the 113 trees the index carries |

⚠️ **These counts are an UPPER BOUND, and the reason is measured.** A tree's status is prose in two places — the index row and the tree file's own metadata — and they DISAGREE on 3 of the 113 rows while 6 index rows carry no status word this reader recognises. Every one of those is counted OPEN, because over-reporting how much is left is the only safe direction for a page that says how much is left. Reconciling them is `DONE-BAR-VISIBILITY.8`; raising the item from a TREE to a LEAF is `DONE-BAR-VISIBILITY.7`.

<!-- END DERIVED -->

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
