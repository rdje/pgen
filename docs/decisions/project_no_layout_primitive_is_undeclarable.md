---
name: project-no-layout-primitive-is-undeclarable
description: "MEASURED FACT (LEX-ADJACENCY.1, 2026-07-26, session #208): PGEN's no-layout lexical-adjacency capability is NOT missing — it SHIPS on both engine halves and is merely UN-DECLARABLE. PARSE: `match_regex(…, false)` is live (10 of 20 sites in `generated/return_annotation_parser.rs`) behind a hard-coded rule-NAME `matches!` arm naming `string_content_double`/`string_content_single`; SystemVerilog's `0 of 1,798` reflects rule NAMES, not a missing path. GENERATE: interior separator suppression is live as `atomic_token_depth` but INFERRED from the return shape (`-> $text`/`$0` or `@transform`), so asking for it costs the AST object AND fuses the rule's LEFT-exterior boundary. The required (interior closed, exterior separated) state is unreachable only because one `is_atomic` bool drives two already-independent consumers. Supersedes `SV-CORPUS-GRAD.3.11`'s 'ROUTE 4 … DOES NOT EXIST'."
metadata:
  node_type: memory
  type: project
  created: 2026-07-26
---

**Measured fact, not an opinion.** Re-run: `bash docs/tasks/artifacts/lex_adjacency/run_probes.sh`
(deterministic, `cmp`-clean). Full design: [`LEX-ADJACENCY-design.md`](../tasks/LEX-ADJACENCY-design.md).

## What is actually true

`SV-CORPUS-GRAD.3.11` concluded that a no-layout lexical boundary "DOES NOT EXIST" on either side
of the engine, and opened the [`LEX-ADJACENCY`](../tasks/LEX-ADJACENCY.md) tree to build one.
**Both halves already exist and both already ship.** What is missing is any way for a *grammar* to
ask for them:

| half | the capability | how it is requested today | why a new consumer cannot use it |
|---|---|---|---|
| **PARSE** | `match_regex(pattern, /*skip_leading_whitespace=*/ false)` — **live**, emitted at **10 of 20** sites in `generated/return_annotation_parser.rs` | a hard-coded rule-NAME allowlist in codegen: `!matches!(rule_name, "string_content_double" \| "string_content_single")` (`ast_based_generator.rs:5551`, mirrors at `cascade.rs:1397`/`:2060`, `cascade/value.rs:878`/`:1107`, `scan.rs:468`, plus the census constant `fusibility_census.rs:60`) | those two names belong to `return_annotation.ebnf`; no grammar can add itself to an engine `matches!` arm |
| **GENERATE** | interior separator suppression (`atomic_token_depth`, `stimuli_generator.rs:9638`) | **INFERRED** from the return shape — `rule_is_lexically_atomic` (`:13023`): a `-> $text`/`$0` return on every branch, or a `@transform` directive | requesting it costs the rule's declared AST object, AND it also fuses the rule's LEFT-exterior boundary |

⚠️ The PARSE-side name-gate is a live **`EBNF-SOURCE-OF-TRUTH` breach** — an acceptance-affecting
behaviour keyed on a rule-NAME literal inside the engine (see
[[project_ebnf_is_single_source_of_truth]]). Retiring it is a doctrine win independent of any
grammar defect.

## The gap is a 2×2, not an absence

Measured at seed 0 on the banked probe grammars:

| setting | interior seam | exterior seam | emitted |
|---|---|---|---|
| ordinary object return | open | separated | `<A>timeunit 0 ns;` |
| `-> $text` (inferred atomic) | closed | **fused** | `<D>timeunit02s;` |
| `@transform` (inferred atomic) | closed | **fused** | `E0105fs` |
| **what IEEE 1800-2017 fn 44 needs** | **closed** | **separated** | `timeunit 10ns;` |

The fourth row is unreachable because `stimuli_generator.rs:9421` derives both effects from a single
`is_atomic` bool — **while `append_generated_segment` (`:13052`) already consumes them as two
independent conditions** (`atomic_token_depth` and `segment_from_atomic_rule`). The missing state is
therefore already representable; only the coupling prevents it.

The fused row is not cosmetic: `/timeunit\b/` cannot be followed by a digit, so `<D>timeunit02s;`
does not re-parse. Reaching for the inferred atomicity naively trades one gen↔parse duality break
for another.

## How to apply

- **Do not rebuild this capability.** The remaining work is to *declare* and *scope* it — the
  decided surface is a rule-level `@lexical_token: true` (deep, interior-only), owned by
  `LEX-ADJACENCY.2`.
- **Keep the two atomicity kinds distinct.** The inferred `$text`/`@transform` atomicity is
  exterior-**fusing** on purpose, because it serves token **fragments** (`"R"` + `digits` → `R12`,
  `hex_escape` `"x"` + payload → `xAB`). A **complete** token like `time_literal` needs
  exterior-**separating**. Changing the inferred behaviour would regress regex and VHDL.
- **The parse half must also neutralise EXPLICIT layout**, not just the implicit skip: SV writes
  `kw_ns_7320d5b7 := trivia /ns\b/` (`grammars/systemverilog.ebnf:6458`), so flipping
  `skip_leading_whitespace` alone still accepts `10 ns`.
- **Emit the parse half statically.** A runtime depth counter would tax all 1,798 terminal sites;
  the closure it must specialize is tiny — `time_literal` reaches **14 of 1,475 rules** — so
  static specialization costs a bare parse nothing. See [[feedback_correctness_before_speed]].

## The generalizable lesson (why this was missed)

Two search failures, both cheap to avoid next time:

1. **A negative capability claim must be quantified over every grammar, not one.**
   `grep -c "match_regex(.*false)"` = 0 in the SV parser is a fact about SV; the same grep over
   `generated/return_annotation_parser.rs` returns 10. One grammar's zero never licenses "PGEN
   cannot X".
2. **A capability reachable only as a side effect of an unrelated declaration is invisible to
   search.** Nobody looks for a *layout* behaviour under *return annotations*. That invisibility is
   itself the argument for making it declarable.

Related: [[project_lexical_annotations_fourth_pillar]] (the pillar and its `[> ]`/`[>! ]` notation),
[[feedback_ast_pipeline_parser_agnostic]], [[feedback_no_rule_deletion_without_lrm_proof]] (why
collapsing `time_literal` into one terminal is not available).
