# `PGEN-RGX-0078-0212` result — the RESULT-CARRIER SLIMMING unit (LANDED)

Leaf `RGX-0078.5.j.4`, session #184, 2026-07-21. The `-0211`-banked ONE fix,
executed design-prereg-first under the `-0197` ratchet. **LANDED — the NINTH
ratchet fix: pooled-paired corpus geomean −2.3606%, flips 0/2,189, MAX
372,542 ≤ 425,000; floor of record → 1,004.4 ns (single-sweep-comparable) /
bench ≈1,579.7 ns ≈314×; the FIRST sub-1 µs raw corpus geomean readings.**

## The unit as landed

`ParseNode` 72 → 48 B (−33.3%), compile-time-pinned (`const` size asserts in
`rust/src/ast_pipeline/mod.rs`):

1. `rule_name: &'static str` → **`&'static &'static str`** (16→8 B). Chosen
   over the sketched u16-id+table (equal 48 B by alignment; zero
   table/serde/state machinery — serde's `&T` delegation + `&&str`
   `PartialEq`/`Debug` delegation keep every derived behavior observably
   identical). Emitted spelling = the static-promoted literal `&"name"`; the
   interpreter interns via the new `intern_ref`.
2. `Quantified(…, &'static str)` → `(…, &'static &'static str)` — shrinks
   `ParseContent` 40 → 32 B (u8 kind REFUTED: open kind set, e.g. `"0,127"`).
3. `span: Range<usize>` → **`Span { start: u32, end: u32 }`** (16→8 B, Copy;
   wire-identical `{"start","end"}` serde shape; custom `Debug` prints
   `start..end`). Honest bound: the generated `parse`/`parse_from` entries
   REFUSE inputs > `u32::MAX` bytes with an explicit error (one branch per
   PARSE; such inputs were never practically parseable).

Scope: lib core (`mod.rs`) + ~160 mechanical consumer sites + the live
emitters (`ast_based_generator.rs`, `cascade.rs`, `cascade/value.rs`,
`scan.rs`, the string-template emitters `unified_return_ast.rs` /
`return_annotation_handler.rs`) + the interpreter. `PgenValue`, the store,
memo protocol, tape, and every parse decision untouched.

## Migration executed — the cold-bootstrap recipe (proven again)

`generated/` emptied (base vintage SHA-banked: `base_artifact_shas.txt`) →
licensed ebnf seed (Steps A–C, `PGEN_ALLOW_BOOTSTRAP_ANNOTATION_FALLBACK=1`)
→ annotation pair (bootstrap = canonical mode) → `generated_parsers` wave
binary → `focus_*` × 8 → dual-feature build LAST (peak 10,081 MB ≤ 16,384
guard) → **ebnf fixed point BYTE-IDENTICAL (path-normalized) under the final
all-11 binary**. Post-regen SHAs: `artifacts_post_regen.sha256` (regex
`1b5bdcf0…`, ebnf `d4c257cf…`).

## Correctness evidence (all green at the candidate vintage)

- **Wire-format byte-identity oracle (explicit):** 44/44 base-vs-candidate
  typed-AST JSON dumps BYTE-IDENTICAL (43 regex corpus cells + 1 json;
  base side captured with the pre-change debug probe embedding the base
  artifacts); the 7 rejected cells identical on both sides
  (`wire_oracle_result.txt`).
- **Battery** (`battery.sh`, all guarded): dual-feature lib suite
  **1006/0 (29 ignored)** incl. the ALL-11 interpreter↔generated
  byte-identical oracle; cert ×3 seeds 0/7/42 byte-exact
  `268/9/259/0 fully_certified` (spf=0); `ast_shape_contract_gate`,
  `duality_hunt_gate`, `regex_pcre2_compile_oracle_gate`,
  `clippy_on_rust_change` (source-strict) — overall=0.

## Perf adjudication — see `adjudication.txt` (two runs)

- Run 1 INVALIDATED by tool-pinned foreign-job contamination (base
  +15.62% same-binary drift; the pre-registered addendum records the breach
  + the hardened protocol BEFORE rerun numbers — commit `094a3ad7`).
- Run 2 (interleaved base₁→cand₁→base₂→cand₂): custody direct legs PASS
  (base −0.99%/−0.92% vs floor; base₁↔base₂ −0.08%); **pooled-paired
  1018.404087075073 → 994.3638657282925 ns = −2.3606%**; flips 0/2,189;
  MAX 372,542 ≤ 425,000; EVERY band improves (0.985/0.968/0.969/0.972 —
  proportional, no flat add). Bench steering −1.25%. Delivered inside the
  prereg's −1.0…−3.4% predicted band (third consecutive in-band delivery).

## Custody

- Base probe `a4067793…` SHA-asserted before every run; candidate
  `8d392176…` preserved as
  `preserved_probes/regex_perf_probe_carrier48_8d392176` = the NEXT
  session's immediate-parent A/B base.
- Artifacts: base vintage regex `438bb931`/ebnf `47266fb0` → candidate
  vintage regex `1b5bdcf0`/ebnf `d4c257cf` (all 11 banked pre/post).
- Custody NOTE (deviation, recorded): run 2's snapshot leg fired (a foreign
  ~100%-of-one-core test binary ran through all four sweeps); adjudicated on
  the direct floor-agreement legs, which measure the distortion the snapshot
  leg proxies — recorded in `adjudication.txt`.

## Reproduction

```sh
R0212_SCRATCH=<scratch> docs/tasks/artifacts/result_carrier_slimming/run_ab.sh       # the original gate
R0212_SCRATCH=<scratch> docs/tasks/artifacts/result_carrier_slimming/rerun_sweeps.sh # the hardened rerun
python3 docs/tasks/artifacts/result_carrier_slimming/analyze_ab.py                   # run-1 analysis
```
