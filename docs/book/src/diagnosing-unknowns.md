# Diagnostic & Debug Toolbox

> **Read this BEFORE investigating any `UNKNOWN`, rejected parse, reach gap, or
> "why isn't this witnessed" question.** PGEN ships a deep, self-explaining debug
> surface. The standing rule (director directive, 2026-06-22) is that you reach
> for these tools **systematically and first** — never eyeball a grammar, never
> guess a root cause, never offer a strategy menu before the toolbox has shown
> you the exact mechanism and location. If the existing tools cannot surface the
> WHY and WHERE, the next step is to *build* a tool — not to speculate.

This chapter is the **master index** of every diagnostic capability. The
parser-level surface (trace levels, call-count dashboard, furthest-position) has
its own deep chapter — [Debugging With `parseability_probe`](parseability-probe-debug.md).
This chapter adds the surfaces that one does *not* cover: the **certificate-coverage
`UNKNOWN` diagnosis workflow**, the reach-path dump, the witness-pass knobs, and
the generation-input / memo observability.

---

## Debug toolbox at a glance

| I want to know… | Tool | One-liner |
|---|---|---|
| Does a file parse? Where does it fail? | `parseability_probe --parse` | `parseability_probe --parse <g> f --profile P` (error carries `furthest_position`) |
| What AST did the parse produce? | `--parse-dump-ast-pretty` | `parseability_probe --parse-dump-ast-pretty <g> f out.json --profile P` |
| Which rules dominate a slow/stuck parse? | `--dump-rule-call-counts` | `parseability_probe --parse <g> f --dump-rule-call-counts 20` |
| Why did a `@predicate` reject a branch? | semantic trace | `PGEN_TRACE_VERBOSITY=debug parseability_probe --parse <g> f --trace-rules <rule>` |
| **What is the cert-coverage proof/witness/`UNKNOWN` split?** | `--report-certificate-coverage` | `ast_pipeline g.ebnf --report-certificate-coverage --grammar-profile P --entry-rule R --count 40 --seed 0` |
| **Union cert-coverage across entries/profiles (certify a rule witnessed-or-proven in ANY supported config)?** | `--cert-union-config` | append `--cert-union-config <entry>[:<profile>]` (repeatable) to the cert command → extra `CERTIFICATE-COVERAGE-UNION:` line |
| **The FULL list of `UNKNOWN` rules (not the truncated 25)?** | `PGEN_CERT_COVERAGE_DUMP_ALL=1` | prefix the cert command |
| **WHY each `UNKNOWN` rule failed to witness (the forced sample + verdict)?** | `PGEN_CERT_COVERAGE_DEBUG_PROBES=1` | prefix the cert command → `[plannable-probe]` lines |
| The reach path (BFS hop chain) the planner installs for a target? | `PGEN_REACH_PATH_DUMP=1` | prefix any generation/cert command |
| Is an `UNKNOWN` a dead rule, a reach gap, or a store-gate rejection? | the 3-step protocol below | dump-all → debug-probes → semantic trace |
| The normalized grammar IR the generators consume? | `--dump-gen-ast` | `ast_pipeline g.ebnf --generate-parser --dump-gen-ast gen.json …` |
| Static well-formedness (LR / non-terminating / shadowing)? | `--lint-grammar` | `ast_pipeline g.ebnf --lint-grammar` |
| Packrat memo hit/miss statistics? | `PGEN_REPORT_MEMO_STATS=1` | prefix a parse/generate command |
| Witness-pass tuning (A/B, budget, ordering)? | `PGEN_WITNESS_*` | see [witness knobs](#witness-pass-knobs) |

> Build note: cert-coverage and any `.ebnf`-direct mode need the debug binary
> built `--features "generated_parsers ebnf_dual_run"`. `parseability_probe` is the
> release binary built `--features generated_parsers`.

---

## The systematic `UNKNOWN` diagnosis protocol

A certificate-coverage `UNKNOWN` means a rule is neither proven-unreachable nor
witnessed-reachable. There are exactly three causes, and **three tool steps tell
them apart with zero guessing.** Run them in order.

### Step 0 — get the honest number and the full list

```bash
PGEN_CERT_COVERAGE_DUMP_ALL=1 \
  ./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf \
  --report-certificate-coverage --grammar-profile sv_2017 \
  --entry-rule systemverilog_file --count 40 --seed 0
```

The summary line is authoritative:

```
CERTIFICATE-COVERAGE: grammar='systemverilog' ... total=1300 witness=1244 UNKNOWN=55
  (sample_parse_failures=0, proof_reverify_failures=0)
```

`PGEN_CERT_COVERAGE_DUMP_ALL=1` prints **all** `UNKNOWN` rules (the default report
truncates to "25 of N shown") plus the `WARNING ... NO reach path from the entry`
list — the dead-rule candidates. **Always confirm determinism** by re-running at
seeds `0`, `7`, `42`; a number that wobbles is a non-deterministic-metric bug, not
a coverage result.

### Step 1 — see WHY each `UNKNOWN` failed to witness

```bash
PGEN_CERT_COVERAGE_DEBUG_PROBES=1 \
  ./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf \
  --report-certificate-coverage --grammar-profile sv_2017 \
  --entry-rule systemverilog_file --count 40 --seed 0 \
  > /tmp/probe.txt 2>&1
grep "rule='known_unscoped_covergroup_type_identifier'" /tmp/probe.txt
```

Each targeted rule emits a `[plannable-probe]` line:

```
[plannable-probe] rule='known_unscoped_covergroup_type_identifier' \
   parsed=false witnessed_target=false sample="localparam \foo \foo ;"
```

Read the two flags:

| `parsed` | `witnessed_target` | Meaning | Cause class |
|---|---|---|---|
| `true` | `true` | Witnessed — not in the residual | (OK) |
| `true` | `false` | The forced sample parsed but routed through **other** rules | reach/routing gap |
| `false` | — | The forced sample **did not parse at all** | malformed forced sample (often a store-gate rejection) |

Aggregate the whole pass to see the dominant failure mode:

```bash
echo "parsed=false:                    $(grep -c 'parsed=false' /tmp/probe.txt)"
echo "parsed=true witnessed=false:     $(grep -c 'parsed=true witnessed_target=false' /tmp/probe.txt)"
echo "parsed=true witnessed=true (OK): $(grep -c 'parsed=true witnessed_target=true' /tmp/probe.txt)"
```

### Step 2 — for a `parsed=false` rule, get the exact rejection

Feed the **forced sample** from Step 1 straight through the parser with the
semantic trace scoped to the suspect rule:

```bash
printf 'localparam \\foo \\foo ;\n' > /tmp/s.sv
PGEN_TRACE_VERBOSITY=debug ./rust/target/release/parseability_probe \
  --parse systemverilog /tmp/s.sv --profile sv_2017 \
  --trace-rules known_unscoped_covergroup_type_identifier 2>&1 | grep -iE "🚫|has_fact|fact_attribute|NEGATIVE"
```

The trace names the rejection exactly:

```
🚫 Rule 'known_unscoped_covergroup_type_identifier' rejected by post predicate
   'fact_attribute_equals [type_name, "\foo", declaration_family, covergroup]'
   ↪ NEGATIVE: 3 facts of kind 'type_name' exist (none matched name "\foo")
```

That is a **store-gate rejection**: the witness sample used `\foo` as a type
without a prior declaration that emits the `type_name`/`covergroup` fact the gate
requires. (See [The Semantic Store](semantic-store.md) and the
`@predicate`/`@emit_fact` directives in the [Annotation System](annotation-system.md).)

### Interpreting the three causes

| What Step 1/2 showed | It is a… | What to do |
|---|---|---|
| `WARNING ... NO reach path from the entry` (Step 0) | **dead-rule candidate** — unreachable from this entry (often belongs to another entry, e.g. `library_text`) | adjudicate via `--lint-grammar`; not a generation bug |
| `parsed=true witnessed_target=false` | **reach/routing gap** — the planner reaches the context but routes through a sibling | a reach-planner / grammar-shape fix |
| `parsed=false` + a `🚫 rejected by post predicate` trace | **store-gate rejection** — the gate's precondition fact was never generated | store-aware witness generation (declare-then-use), the `STORE-AWARE-GEN` lane |

This protocol replaces eyeballing the grammar. It is mandatory before proposing
any fix for an `UNKNOWN`.

---

## Reach-path dump (`PGEN_REACH_PATH_DUMP`)

```bash
PGEN_REACH_PATH_DUMP=1 ./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf \
  --report-certificate-coverage --grammar-profile sv_2017 \
  --entry-rule systemverilog_file --count 40 --seed 0 2>&1 | grep -i reach
```

Prints the BFS hop chain (`reach_hops`) the planner installs to steer generation
toward each target rule — the entry→target rule path with the chosen branch at
each hop. Use it when a `parsed=true witnessed_target=false` result needs you to
see *which* path the planner took (and therefore which sibling stole the bytes).

---

## Witness-pass knobs

These tune the plannable-witness pass; default-off / default-floor, so production
runs are byte-identical without them. Use for A/B isolation.

| Variable | Effect | Use |
|---|---|---|
| `PGEN_WITNESS_NO_PURDOM=1` | Disable Purdom shortest-derivation ordering | A/B: is the ordering the cause? |
| `PGEN_WITNESS_TIMEOUT_FLOOR_MS` | Per-target witness budget floor (default 200 ms) | Give a deep target more budget |
| `PGEN_GENERATION_STEPS_PER_MS` | Steps→time calibration for the step-budget | Bound a pathological deep generation |
| `PGEN_CERT_DIVERSE_GENERATION_TIMEOUT_MS` | Per-sample step-budget for the **diverse** (PASS-1) generation (default 4000 ms = 4 000 000 steps) | Keep the cert from hanging on a deeply-recursive grammar; set `0` for the legacy unbounded pass |

The diverse-pass budget (`PGEN_CERT_DIVERSE_GENERATION_TIMEOUT_MS`) is a
deterministic safety bound, not a tuning knob: its default is far above any
well-behaved grammar's per-sample cost, so the cert headlines stay
byte-identical, but it deterministically cuts a deep-recursion runaway (e.g.
`rtl_const_expr` / `conditional_expr` at `--max-depth` 40/48, which spin
unboundedly without it) in ~15 s instead of hanging.

To confirm a witness-reach regression is **not** depth, re-run the cert at
`--max-depth 24`, `32`, `40` — if the `UNKNOWN` set is unchanged, the per-target
budget is adequate and the cause is elsewhere (a forcing/store-gate bug).

---

## Generation-input & memo observability

| Tool | Command | Shows |
|---|---|---|
| Normalized grammar IR | `ast_pipeline g.ebnf --generate-parser --dump-gen-ast gen.json --dump-gen-ast-pretty …` | the exact AST the generators consume (after LR-elimination etc.) |
| Static well-formedness | `ast_pipeline g.ebnf --lint-grammar` | left-recursion info, non-terminating errors, ordered-choice shadowing |
| Memo statistics | `PGEN_REPORT_MEMO_STATS=1 parseability_probe --parse <g> f` | packrat hit/miss counts (perf triage) |
| Well-formedness gen-AST (tests) | `PGEN_WELLFORMEDNESS_GEN_AST=<path> cargo test --lib …` | the gen-AST a well-formedness test loads |

---

## Cross-references

- [Debugging With `parseability_probe`](parseability-probe-debug.md) — trace
  levels, `--trace-rules`, the call-count dashboard, furthest-position, the
  predicate self-explaining trace (the parser-level detail this chapter builds on).
- [Grammar Well-Formedness & Well-Definedness](grammar-wellformedness.md) — what
  proof / witness / `UNKNOWN` mean and why `UNKNOWN`→0 is the trustworthiness number.
- [The Semantic Store: Parser Memory](semantic-store.md) — the facts/scope model the
  store-gates query.
- KM card `cert-coverage-unknown-diagnostics` (`docs/knowledge/`) — the same
  protocol as a retrieval-indexed knowledge card.
- Decision `feedback_systematically_use_debug_toolbox` (`docs/decisions/`) — the
  standing directive that makes this protocol mandatory.
