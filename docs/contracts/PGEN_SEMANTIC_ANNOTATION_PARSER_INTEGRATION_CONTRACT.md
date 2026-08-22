# docs/contracts/PGEN_SEMANTIC_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md

## Purpose
Define the current downstream integration contract for PGEN's `semantic_annotation` parser family.

## Source Of Truth
- Main grammar source:
  - `grammars/semantic_annotation.ebnf`
- Bootstrap-safe grammar source:
  - `grammars/builtin_semantic_annotation.ebnf`
- Tracked generated artifacts:
  - `generated/semantic_annotation.json`
  - `generated/semantic_annotation_parser.rs`
- Public host API:
  - `rust/src/embedding_api.rs`
- Normative semantic/contract docs:
  - `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md`
  - `docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md`

## Stable Integration Surface
- Annotation family:
  - `semantic`
- Stable host entry points:
  - `parse_annotation(...)`
  - `parse_annotation_result(...)`
  - `parse_annotation_named(...)`
- Stable family selector:
  - `AnnotationFamily::Semantic`
- Stable backend selectors:
  - `ParserBackend::Bootstrap`
  - `ParserBackend::Generated`
- Stable diagnostics:
  - `E_BACKEND_UNAVAILABLE`
  - `E_PARSE_FAILURE`
  - `E_INPUT_TOO_LARGE`
  - `E_INVALID_LIMITS`
  - `E_INVALID_ARGUMENT`

## Build / Availability Requirements
- Bootstrap backend is part of the published contract.
- Generated backend is part of the published contract when the generated annotation parser is available.
- Downstream consumers should use the embedding API surface rather than directly depending on internal generated parser types.

## Validation / Release Gates
- `make -C rust SHELL=/bin/bash annotation_contract_gate`
- `make -C rust SHELL=/bin/bash semantic_usage_gate`
- `make -C rust SHELL=/bin/bash semantic_runtime_contract_gate`
- `make -C rust SHELL=/bin/bash semantic_full_contract_gate`

## Scope / Non-Goals
- This contract covers parser-family selection, acceptance/rejection, diagnostics, and the current bootstrap/generated host surface.
- Semantic-runtime meaning, steering leverage, and aggregate proof obligations are governed by `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md` and `docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md`, not by this file alone.
- `semantic_annotation` does not currently have a separate top-level live-status row; track its maturity through the annotation proof spine and the docs above.
- When reporting downstream bugs, follow `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`; accepted released-parser bugs should then be logged in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`.

## Recent Additions

### 2026-08-23 — map KEYS: eight of nine key shapes were REJECTED; the accept set WIDENS (`GRAMMAR-WELLFORMED.H.16.6b`, `PGEN-GRAMMAR-WELLFORMED-0173`)

`map_entry`'s key was declared as a full `annotation_value`. This grammar spells `=>` in **four**
places across **three** roles — the map arrow (`map_entry`), the implication operator
(`implication_expr`), the lambda arrow (`lambda_expression`, both alternatives) and the function-type
arrow (`function_type`) — so under PEG's ordered choice a key allowed to be any of those consumed the
map's own arrow and the entry then had none left to match. The exact characterisation, measured: **a
map entry parsed if and only if its `key => value` was not itself a valid `annotation_value`.** Of nine
key shapes only the string-keyed one worked (`{"a" => "b"}`); `{1 => 2}`, `{a => b}`, `{[a] => b}`,
`{(a) => b}`, `{(a, b) => c}`, `{true => false}`, `{(Foo) => Bar}` and `{Foo => Bar}` were rejected.

The key is now a dedicated `map_key` — `annotation_value` minus exactly the three arrow-consuming
reaches, and nothing else. **Values are unchanged**, so a lambda or implication remains legal on the
right of the arrow (`{a => (x) => y}` parses).

⭐ **Strictly a WIDEN for consumers, measured on both axes** — this is not a "should be safe" claim:
- `ACCEPT-SET-LEDGER:` over 1 211 inputs (62 discriminating probes, 149 real annotation lines
  extracted from the tracked grammars, 1 000 generated stimuli at seeds 0/7/42/123/999):
  **25 newly accepted, 0 newly rejected.**
- `AST-IDENTITY-SWEEP:` over every input both grammars accept: **1 156 / 1 156 byte-identical,
  0 typed ASTs moved** — at both the entry rule and `annotation_value`, on an instrument proven able
  to fire (7/7 and 32 moves on deliberately AST-shape-only red arms).
- The declared-annotation inventory grows 152 → 170 (the four new key rules' branches). **No existing
  rule's declared annotation changed**, checked entry-by-entry.
- Certificate coverage `115/0/84/31` → **`119/0/90/29`** at seeds 0/7/42: four rules added and all four
  witnessed, `array_type` and `optional_type` newly reachable, **nothing newly UNKNOWN**, and
  `sample_parse_failures` 1 → **0** at seed 7.

A consumer that only READ map entries sees strictly more of them. A consumer that relied on
`{1 => 2}` being a parse ERROR is the only one affected, and that behaviour was a defect.

### 2026-06-10 — whole-body-group branch shapes RESTORED (regression window 2026-05-14 → 2026-06-10; ledger `SEMANN-0001`)

A 2026-05-14 engine refinement (the inner→outer branch-index remap) collapsed whole-body parens-group branch annotations onto branch 0, so **43** of this grammar's declared branch annotations (inventory 108→151 once corrected) silently stopped applying to branches 1+ — e.g. `annotation_name` branch 1, `annotation_value` branches 1–3, and `boolean_literal`'s second alternative produced raw passthrough instead of their declared typed shapes in parsers regenerated inside the window.

Fixed by `BRANCH-BROADCAST-FIX.2` (`PGEN-BRANCH-BROADCAST-FIX-0002`): trailing annotations on whole-body groups broadcast to every runtime branch again. Branch-0 inputs are byte-identical; only the previously-buggy branch-1+ shapes change (raw passthrough → declared typed shape). Regression locks: the AST shape-contract manifest now embeds the full 151-row declared-annotation inventory with both the pipeline-artifact and raw-IR crosscheck comparisons active, plus the engine's focused broadcast unit tests.

### 2026-05-20 — `SV-EXH-PROOF.3.3.4.a.1` / `.a.2` (`PGEN-SV-EXH-PROOF-0026` / `0027`): rule-reference syntax — dotted + indexed, depth-unbounded

The `$<ref>` reference shape accepted in semantic-annotation directive payloads is extended (strictly additive — every prior `$name` / `$1` reference parses byte-identically):

```
rule_reference   ::= "$" head segment*
head             ::= /[a-zA-Z_][a-zA-Z0-9_]*/        # named
                  |  /[0-9]+/                          # positional, 1-indexed
segment          ::= "." /[a-zA-Z_][a-zA-Z0-9_]*/     # dotted property
                  |  "[" /[0-9]+/ "]"                  # non-negative integer index
```

Examples now accepted: `$name.body`, `$1.body.subkey`, `$items[0]`, `$matrix[0][1]`, `$a.b[0].c[1].d.e[2].r.z`.

Subset boundary fixed: dotted property + non-negative integer indexing only. NOT full JSONPath (no filters / wildcards / recursive descent / negative indices / range slices). Each excluded feature would require its own normative leaf.

Two-surface lockstep. Both the EBNF-language surface (`grammars/semantic_annotation.ebnf::rule_reference_name`) and the grammar-directive-payload runtime (`unified_semantic_ast.rs::StructuredSemanticValueParser::parse_rule_reference`) are extended in lockstep. Authors writing directives inside grammar `.ebnf` files hit the runtime surface; freestanding annotation strings through the embedding API hit the EBNF surface. Both accept the same set after these slices.

Durable no-depth-limit guarantee. The reference depth is structurally unbounded at every layer (EBNF `*`, hand-rolled `loop`, lexer, resolver iterator). Locked by two regression tests exercising 64 segments each — see `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md` "Rule Reference Syntax (Normative)" for the normative pin and the failure-direction.

Strict trailing-dot / strict-bracket policy. Malformed forms (bare `.`, `[` with no `<digits>]`) roll back to before the offending segment; the surrounding payload parser then handles the leftover or falls back to `Raw`.

## Notable Recent Shape Changes

### 2026-08-11 — LR-eliminated rules now return the DECLARED AST (`ENGINE-UNIVERSAL-SERVICES.8`)

`type_reference` is left-recursive through four wrapper alternatives (`union_type`,
`intersection_type`, `array_type`, `optional_type`), so PGEN eliminates the recursion. Until now the
elimination's own internal record — `{initial, suffixes, type: "_pgen_lr_chain", wrapper_specs}` —
was published *as the rule's typed AST* instead of the declared `{type: "union_type", types: […]}`
and its siblings. The engine now folds the chain back through the author's annotations, so an
LR-eliminated rule is indistinguishable from a hand-written one.

⭐ **Published-surface impact: NONE.** This family's stable surface returns a verdict plus
diagnostics and carries no AST. Only a consumer walking the raw typed-AST JSON of an LR-eliminated
rule sees a difference, and for such a consumer this is a buggy→correct fix, not a versioned
evolution — the previous value was never a declared shape.

Regression locks: the annotation-shape gate now FAILS any emitted value whose `type:` carries the
engine-reserved `_pgen_` prefix (proven to fire by a RED probe), and the combinator suite's
`left_recursion_folded_ast` case asserts the exact declared left-nested value.
