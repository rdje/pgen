# Schema and Versioning

## Grammar version

The value language is versioned in its grammar header: `grammars/semantic_annotation.ebnf` declares
**version `2.0`**. The steering semantics are a *living* contract — the normative spec
(`docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md`) and the steering control matrix
(`docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md`) evolve through the SC-01…SC-13 control
slices, each promoted to a Tier-4 gate-enforced contract.

## What is contract-stable

Per `docs/contracts/PGEN_SEMANTIC_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`, the stable parse surface
is:

- the annotation family selector `AnnotationFamily::Semantic`;
- the entry points `parse_annotation` / `parse_annotation_result` / `parse_annotation_named`;
- the backend selectors `ParserBackend::Bootstrap` / `ParserBackend::Generated`;
- the diagnostic codes `E_BACKEND_UNAVAILABLE`, `E_PARSE_FAILURE`, `E_INPUT_TOO_LARGE`,
  `E_INVALID_LIMITS`, `E_INVALID_ARGUMENT`;
- the parsed-annotation node shapes documented in [AST Envelope](ast-envelope.md).

`semantic_annotation` does **not** carry a separate top-level live-status row; track its maturity
through the annotation proof spine and the normative docs.

## Proof gates

| Gate | Proves |
| --- | --- |
| `make -C rust SHELL=/bin/bash annotation_contract_gate` | the aggregate annotation contract spine (validator coverage + built-in/shared suites + SC slices + robustness/stimuli) |
| `make -C rust SHELL=/bin/bash semantic_usage_gate` | the semantic-annotation leverage contract (the steering actually moves parser/stimuli behavior) |
| `make -C rust SHELL=/bin/bash semantic_runtime_contract_gate` | the semantic runtime / typed-AST contract checks |
| `make -C rust SHELL=/bin/bash semantic_full_contract_gate` | the focused aggregate semantic proof surface (runtime + round-trip + differential regression) |

The thirteen control slices SC-01…SC-13 are each gate-enforced (per-slice contract gates with
differential bootstrap/generated parity checks).

## Notable shape changes

### 2026-06-10 — whole-body-group branch shapes RESTORED (ledger `SEMANN-0001`)

A 2026-05-14 engine refinement (the inner→outer branch-index remap) collapsed whole-body parens-group
branch annotations onto branch 0, so **43** of this grammar's declared branch annotations (inventory
108→151 once corrected) silently stopped applying to branches 1+ — e.g. `annotation_name` branch 1,
`annotation_value` branches 1–3, and `boolean_literal`'s second alternative produced raw passthrough
instead of their declared typed shapes in parsers regenerated inside the window.

`BRANCH-BROADCAST-FIX.2` restored it — trailing annotations on whole-body groups broadcast to every
runtime branch again. Branch-0 inputs are byte-identical; only the previously-buggy branch-1+ shapes
change (raw passthrough → declared typed shape). The fix is regression-locked by the AST shape-contract
manifest's full 151-row declared-annotation inventory (both pipeline-artifact and raw-IR crosschecks)
plus the engine's focused broadcast unit tests.

### 2026-05-20 — rule-reference syntax: dotted + indexed, depth-unbounded (`SV-EXH-PROOF.3.3.4.a.1`/`.a.2`)

The `$<ref>` reference shape was extended (strictly additive — every prior `$name` / `$1` reference
parses byte-identically) to accept dotted property segments (`.field`) and non-negative integer index
segments (`[i]`), chained to unbounded depth: `$name.body`, `$1.body.subkey`, `$items[0]`,
`$matrix[0][1]`, `$a.b[0].c[1].d.e[2].r.z`. It is a subset — **not** full JSONPath (no filters,
wildcards, recursive descent, negative indices, or range slices). Both runtime surfaces (the EBNF
language and the directive-payload runtime) were extended in lockstep; the depth is structurally
unbounded at every layer, locked by 64-segment regression tests. See [Annotation Values and
References](values-and-references.md).
