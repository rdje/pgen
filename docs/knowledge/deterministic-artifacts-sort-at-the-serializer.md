---
id: deterministic-artifacts-sort-at-the-serializer
title: A byte-unreproducible JSON artifact is a WRITER defect — canonicalize at the serializer, never by changing a hot-path field type
answers:
  - "why do two runs of the same binary on the same inputs write different bytes but equal content"
  - "how do I tell a generation non-determinism from a serialization non-determinism"
  - "why is one artifact of a run byte-unstable while the others are byte-identical"
  - "how do I make a serde struct containing a HashMap serialize reproducibly"
  - "why not just change the HashMap field to a BTreeMap"
  - "how do I write a control that proves a map serializes in sorted order"
  - "which grammar should I reproduce a determinism defect on"
tags: [determinism, serialization, artifacts, stimuli, coverage, root-cause]
date: 2026-08-08
status: current
evidence: rust/src/ast_pipeline/stimuli_generator.rs (`serialize_string_map_sorted`, `serialize_string_maps_sorted`, and the three `#[serde(serialize_with)]` sites on `StimuliCoverageMetrics::rule_success_hits` / `::branch_groups` / `BranchCoverageGroup::failure_reasons`); rust/src/ast_pipeline/unified_return_ast.rs (`serialize_properties_sorted`, the first instance of the same idiom); docs/tasks/SV-EXH-PROOF.md leaf .7.4.6.16; docs/tasks/PARSE-HARNESS.md §18 (.5.3, the typed-AST instance); docs/book/src/stimuli-and-quality.md "All Four Closed-Loop Artifacts Are Byte-Reproducible"
reverify: "cargo test --features 'generated_parsers ebnf_dual_run' --lib coverage_metrics_serialize_with_sorted; for i in 1 2 3; do ./rust/target/debug/ast_pipeline grammars/regex.ebnf --generate-stimuli --count 40 --seed 0 --coverage-output rust/target/km_cov_$i.json --output rust/target/km_stim_$i.txt >/dev/null; done; cmp rust/target/km_cov_1.json rust/target/km_cov_2.json && cmp rust/target/km_cov_1.json rust/target/km_cov_3.json && echo BYTE-REPRODUCIBLE"
---

**Two runs, same binary, same inputs, same file SIZE — and `cmp` says they differ.** That signature is
almost always a *writer* defect, not a generator defect, and the two have opposite fixes. Separate
them with one measurement before touching anything:

| observation | verdict |
|---|---|
| `json.load(a) == json.load(b)` is **True**, key SETS equal, key ORDERS differ | **serialization** — the data is deterministic, the emitter is not |
| every OTHER artifact of the same run is already byte-identical | confirms the generator (and its RNG) is not the suspect |
| canonical (`sort_keys`) `sha256` of both files is equal | the divergence is *entirely* ordering |

A std `HashMap` iterates in a per-instance order, so any `#[derive(Serialize)]` reaching one emits a
different key order in every process. Measured instance: PGEN's closed-loop coverage artifact came
out at an identical **77 227 B** across three `regex` runs while `cmp` split them at *"char 210,
line 10"* — 269 `rule_success_hits` keys and 143 `branch_groups` keys, same set, different order.

**The fix is a `#[serde(serialize_with = …)]` that collects into a `BTreeMap` of REFERENCES**, one
per map-bearing field (plus a `collect_seq` variant for a `Vec<HashMap<…>>`). Deserialization is
untouched — a JSON object read back into a `HashMap` is order-insensitive — so every existing reader
is unaffected.

⛔ **Do NOT "fix" it by changing the field to a `BTreeMap`.** In PGEN two of the three maps are read
once per OR decision on the hot generation path; a `BTreeMap` field would buy a *serialization*
property with log-n lookups on every generated sample — a cost paid by everyone, forever, for a
property only the artifact writer needs ([[project_capability_growth_is_zero_cost_and_neutral]]).
A serialize-side sort costs one collect per artifact WRITE, a handful per run.

⭐ **Search for the idiom before writing one.** This repo already had exactly this serializer —
`unified_return_ast::serialize_properties_sorted`, added when the interpreter's typed AST diverged
from the generated parser on map order — with the reasoning in its own doc comment. Cloning its
shape keeps ONE idiom in the codebase instead of two that drift
([[feedback_read_prior_art_before_designing]]).

⭐ **Reproduce on the CHEAPEST grammar that exhibits it.** The defect belonged to a 4–7-minute
SystemVerilog replay stage; `regex` reproduced it in seconds *and* gave a sharper diagnosis, because
a 269-key map is small enough to compare key by key. Spend the expensive subject only on proving the
fix on the real artifact.

⛔ **A control whose VALUES move with its ordering is not a control.** The natural test — build the
same entries under opposite insertion orders and assert byte-identical output — is silently wrong if
each value is derived from the insertion INDEX: reversing then changes the *data*, so the test
compares two different documents and "fails for the right reason" by accident. Derive test values
from the KEY. And assert the keys come out **sorted**, not merely equal: equality alone is satisfied
by two maps that happened to iterate alike. Both halves were needed to make PGEN's oracle go RED
with the `serialize_with` attributes detached and GREEN with them.

See also [[feedback_instrument_needs_ground_truth]] (a control needs a pinned positive AND negative)
and [[closed-loop-residual-ratchet]] (what the now-comparable artifacts feed).
