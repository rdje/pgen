---
id: cert-coverage-unknown-diagnostics
title: Diagnosing a certificate-coverage UNKNOWN — the systematic 3-step debug protocol
answers:
  - "why is a cert-coverage rule UNKNOWN"
  - "how do I find out why a rule is not witnessed in certificate-coverage"
  - "how do I dump the full list of UNKNOWN rules (not the truncated 25)"
  - "how do I dump the forced witness samples the cert pass generates"
  - "what env var shows why a store-gated rule fails to witness"
  - "how do I tell a dead rule from a reach gap from a store-gate rejection"
  - "how do I trace which @predicate rejected a witness sample"
  - "how do I see the reach path the witness planner installs for a target"
  - "which debug env vars exist for certificate-coverage and the witness pass"
  - "how do I prove an UNKNOWN regression is not a depth problem"
tags: [cert-coverage, debug, unknown, witness, store-gate, diagnostics, tooling]
date: 2026-06-22
status: current
evidence: rust/src/main.rs (PGEN_CERT_COVERAGE_DUMP_ALL @2710, PGEN_CERT_COVERAGE_DEBUG_PROBES @2523); rust/src/ast_pipeline/stimuli_generator.rs (PGEN_REACH_PATH_DUMP @2770, the [plannable-probe] emitter); docs/book/src/diagnosing-unknowns.md
reverify: `grep -n 'PGEN_CERT_COVERAGE_DUMP_ALL\|PGEN_CERT_COVERAGE_DEBUG_PROBES' rust/src/main.rs; grep -n 'PGEN_REACH_PATH_DUMP\|plannable-probe' rust/src/ast_pipeline/stimuli_generator.rs`
---

A certificate-coverage `UNKNOWN` = a rule neither proven-unreachable nor
witnessed-reachable. There are exactly **three** causes and **three tool steps**
that tell them apart with zero guessing. Run them in order BEFORE proposing any
fix (standing directive — never eyeball the grammar, never guess). Full prose +
recipes: `docs/book/src/diagnosing-unknowns.md`. Cert/`.ebnf` modes need the
debug binary built `--features "generated_parsers ebnf_dual_run"`.

## Step 0 — full list + honest number
```
PGEN_CERT_COVERAGE_DUMP_ALL=1 ./rust/target/debug/ast_pipeline grammars/<g>.ebnf \
  --report-certificate-coverage --grammar-profile <P> --entry-rule <R> --count 40 --seed 0
```
Prints ALL UNKNOWN rules (default truncates to "25 of N") + the
`WARNING ... NO reach path from the entry` dead-rule-candidate list. Confirm
determinism at seeds 0/7/42 (a wobbling number is a non-deterministic-metric bug).

## Step 1 — WHY each UNKNOWN failed to witness
```
PGEN_CERT_COVERAGE_DEBUG_PROBES=1 ... --report-certificate-coverage ... > /tmp/probe.txt 2>&1
grep "rule='<target>'" /tmp/probe.txt
```
Each target emits `[plannable-probe] rule='X' parsed=<b> witnessed_target=<b> sample="..."`:
- `parsed=true witnessed_target=true` → witnessed (not residual).
- `parsed=true witnessed_target=false` → forced sample parsed but routed through OTHER rules = **reach/routing gap**.
- `parsed=false` → forced sample did not parse = **malformed forced sample**, usually a store-gate rejection.

## Step 2 — exact rejection for a `parsed=false` rule
Feed the forced sample from Step 1 back through the parser with the scoped semantic trace:
```
printf '<the sample>\n' > /tmp/s.sv
PGEN_TRACE_VERBOSITY=debug ./rust/target/release/parseability_probe \
  --parse <g> /tmp/s.sv --profile <P> --trace-rules <target> 2>&1 | grep -iE "🚫|has_fact|fact_attribute|NEGATIVE"
```
Names the rejection, e.g. `🚫 Rule 'known_unscoped_covergroup_type_identifier' rejected by post
predicate 'fact_attribute_equals [type_name,"\foo",declaration_family,covergroup]' ↪ NEGATIVE:
... none matched name "\foo"` ⇒ the witness sample used an undeclared identifier; the gate's
precondition fact was never generated (the store-aware-generation / declare-then-use gap).

## Cause → fix
| Signal | Cause | Fix lane |
|---|---|---|
| `NO reach path from the entry` (Step 0) | dead-rule candidate (often belongs to another entry, e.g. `library_text`) | adjudicate via `--lint-grammar`; not a generation bug |
| `parsed=true witnessed_target=false` | reach/routing gap (sibling steals bytes) | reach-planner / grammar-shape |
| `parsed=false` + `🚫 rejected by post predicate` | store-gate rejection (precondition fact never generated) | store-aware witness generation (`STORE-AWARE-GEN`) |

## Companions
- `PGEN_REACH_PATH_DUMP=1` (prefix any gen/cert cmd) → the BFS hop chain the planner installs per target (see which path stole the bytes for a `witnessed_target=false`).
- NOT-depth check: re-run cert at `--max-depth 24/32/40`; unchanged UNKNOWN set ⇒ the per-target budget is adequate, cause is a forcing/store-gate bug, not depth.
- `PGEN_WITNESS_NO_PURDOM=1` (A/B the ordering), `PGEN_WITNESS_TIMEOUT_FLOOR_MS` (per-target budget floor).
- Related: [[sv-cert-coverage-predicate-gated-false-witness]], [[sv-store-fact-scope-and-canonical-name-coupling]], [[prove-rule-dead-or-reachable]], [[ast-pipeline-cli-reference]].
