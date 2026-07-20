# `PGEN-RGX-0078-0199` result

Verdict: **LANDED — strict geomean ratchet PASS.**

The checkpoint's duplicate `scope_len` word is removed:
`SemanticRuntimeCheckpoint` is now a 6-word `Copy` value. The public
`scope_len()` accessor is preserved and answers from `chain_len` — the same
value, because `scopes` and `active_chain` are maintained in lockstep at
every mutation site (`new()`/reset, `open_scope`, `close_scope`, the rollback
rebuild, `apply_delta`; audit table in `design_prereg.md`). The two
`scopes`-side debug asserts and `commit()` now compare against `chain_len`,
doubling as the mechanical lockstep tripwire, and a focused test pins the
accessor/chain-depth lockstep across open/close/checkpoint/rollback.

Adjudication (binding `-0197` ratchet, same-session A/B, `adjudication.txt`):

- unrounded canonical PCRE2 external-corpus geomean
  **1210.6606132712145 → 1193.1574265537554 ns = −1.4458%** — strict
  decrease PASS;
- verdict flips **0/2,189**;
- candidate corpus MAX **469,125 ≤ 483,583 ns** (same worst cell
  `pcre2:testdata/testinput2:line_725`);
- floor validation −1.35% vs the banked ≈1,937.4 ns bench (custody OK);
  bench steering +0.41%, inside noise, non-gating.

Honest magnitude caveat (recorded per the `-0197` noise clause): the
measured −17.5 ns is ≈20× the 0.859310862 ns exact-current classifier
pricing, and the −1.45% magnitude sits inside the ≈2.3% observed
session-noise span. The direction gate adjudicates — the strict same-session
compare passed — while the magnitude carries bounded confidence; candidates
for the surplus are cross-site codegen/register-pressure effects of the
narrower `Copy` value at the ~20k checkpoint/rollback sites per MAX-cell
parse.

Correctness battery (all green, changed-lib vintage): dual-feature lib
**1002/0 (29 ignored)** including the ALL-11 interpreter↔generated
byte-identical oracle and the new lockstep test; cert ×3 seeds 0/7/42
byte-exact `268/9/259/0 fully_certified=true` (`sample_parse_failures=0`);
`ast_shape_contract_gate`; `duality_hunt_gate`;
`regex_pcre2_compile_oracle_gate`; `regex_typed_differential_gate`
(post-gate hash-check: `generated/regex_parser.rs` = `f85f2121…`, restore
verified); strict-source clippy.

Custody (`custody.txt`): base probe `0a7346ce…` — byte-identical (`cmp`) to
the preserved `-0198` candidate `preserved_probes/regex_perf_probe_g3reset_0a7346ce`,
copied to scratch + SHA-banked before any source change; candidate
`8aef8541…` preserved as
`preserved_probes/regex_perf_probe_ckptword_8aef8541`; LIB-ONLY change —
both probes embed the SAME regex artifact `f85f2121…` (asserted in-run);
serialized memory-guarded caffeinated runs, order-alternated bench rounds,
full per-cell JSONLs banked both sides.

Floor re-baselined: **corpus geomean 1,193.2 ns** (from 1,227.7); bench
floor and settled MAX unchanged. The <1 µs call-off bar now needs −16.19%.

Per the one-fix/fresh-session directive this session stops after the clean
commit. NEXT (brand-new session) = `PGEN-RGX-0078-0200` — generated ID-only
recursion-name path (fixed entry/exit, rollback, and growth measured
together as one representation fix).
