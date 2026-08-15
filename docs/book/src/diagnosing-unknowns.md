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
| Does a file parse? Where does it fail? | `parseability_probe --parse` | `parseability_probe --parse <g> f --profile P` (every family's error carries `furthest_position` — the deep locus, not the shallow one; see [the chapter](parseability-probe-debug.md#furthest-position-error-diagnostic)) |
| What AST did the parse produce? | `--parse-dump-ast-pretty` | `parseability_probe --parse-dump-ast-pretty <g> f out.json --profile P` |
| Which rules dominate a slow/stuck parse? | `--dump-rule-call-counts` | `parseability_probe --parse <g> f --dump-rule-call-counts 20` |
| Why did a `@predicate` reject a branch? | semantic trace | `PGEN_TRACE_VERBOSITY=debug parseability_probe --parse <g> f --trace-rules <rule>` |
| **Is a (regex) reject grammar-owned or validator-owned (load-bearing)?** | the message-source probe | `parseability_probe --parse regex f --profile pcre2` → classify by the reject MESSAGE (see [below](#is-a-reject-grammar-owned-or-validator-owned-the-message-source-probe)) |
| **What is the cert-coverage proof/witness/`UNKNOWN` split?** | `--report-certificate-coverage` | `ast_pipeline g.ebnf --report-certificate-coverage --grammar-profile P --entry-rule R --count 40 --seed 0` |
| **Union cert-coverage across entries/profiles (certify a rule witnessed-or-proven in ANY supported config)?** | `--cert-union-config` | append `--cert-union-config <entry>[:<profile>]` (repeatable) to the cert command → extra `CERTIFICATE-COVERAGE-UNION:` line |
| **The FULL list of `UNKNOWN` rules (not the truncated 25)?** | `PGEN_CERT_COVERAGE_DUMP_ALL=1` | prefix the cert command |
| **WHY each `UNKNOWN` rule failed to witness (the forced sample + verdict)?** | `PGEN_CERT_COVERAGE_DEBUG_PROBES=1` | prefix the cert command → `[plannable-probe]` lines |
| The reach path (BFS hop chain) the planner installs for a target? | `PGEN_REACH_PATH_DUMP=1` | prefix any generation/cert command |
| **The plan forced branch N — did the render OBEY, or fall back to a sibling silently?** | `PGEN_REACH_FORCED_OVERRIDE_DUMP=1` | prefix any generation/cert command → `[forced-override]` lines |
| **Which residual `UNKNOWN`s are profile-excluded by construction (vs genuine)?** | `PGEN_CERT_RESIDUAL_CLASSIFICATION=1` | prefix the cert command → `RESIDUAL-CLASSIFICATION` block |
| Is an `UNKNOWN` a dead rule, a reach gap, or a store-gate rejection? | the 3-step protocol below | dump-all → debug-probes → semantic trace |
| The normalized grammar IR the generators consume? | `--dump-gen-ast` | `ast_pipeline g.ebnf --generate-parser --dump-gen-ast gen.json …` |
| Static well-formedness (LR / non-terminating / shadowing)? | `--lint-grammar` | `ast_pipeline g.ebnf --lint-grammar` (add `PGEN_LINT_DUMP_ALL=1` to print every finding of every class) |
| `left_recursion_unhandled=N` — which rule absorbs the chain, and what does it cost? | `--report-indirect-lr-plan` | `ast_pipeline g.ebnf --report-indirect-lr-plan` |
| Packrat memo hit/miss statistics? | `PGEN_REPORT_MEMO_STATS=1` | prefix a parse/generate command |
| EXACT per-rule entry counts for a parse (machine-readable JSON)? | `--dump-rule-entry-counts-json` | `parseability_probe --parse <g> f --dump-rule-entry-counts-json c.json` |
| How much parse work is DISCARDED (failed speculation), per rule? | `--dump-rule-outcome-counts-json` | `parseability_probe --parse <g> f --dump-rule-outcome-counts-json o.json` |
| **Did the whole SV parser get SLOWER — and would anything have told me?** ⛔ never answer with an ad-hoc timing script against a remembered number | the parse-cost **ratchet** (doctrine `PARSE-COST-RATCHET`) | `bash scripts/check_parse_cost_ratchet.sh` (~0.4 s identity tier) · `make -C rust sv_parse_cost_ratchet` (~2.5 min) |
| **WHERE does the parse time actually go?** ⛔ no counter can answer this — they all route to the PROTOCOL graph, and a production parse runs the FUSED one | `/usr/bin/sample` on a BARE parse | `parseability_probe --parse systemverilog big.sv --profile sv_2017 &` then `/usr/bin/sample $! 4 1 -f prof.txt` |
| Which rules a derived DFA scanner could fuse + the measured ceiling? the choice-site / merged-choice surface? | `--report-fusibility-census` | `ast_pipeline g.ebnf --report-fusibility-census [--fusibility-entry-counts c.json] [--fusibility-outcome-counts o.json]` |
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
| `true` | `false` | The forced sample parsed but routed through **other** rules | **three different causes — see below** |
| `false` | — | The forced sample **did not parse at all** | malformed forced sample (often a store-gate rejection) |

⛔ **`parsed=true witnessed_target=false` is not one cause, and calling it "a reach gap" is how a
wrong diagnosis gets written down confidently.** The sample tells you where generation *ended up*; it
never tells you what the plan *asked for*. Always run the reach-path dump (below) before concluding:

| What the hop dump shows | Cause | Where the fix lives |
|---|---|---|
| chain missing, short, or routed through the wrong carrier | **plan-side** — a genuine reach/routing gap | the reach planner |
| chain complete and correct, sample ignores it | **render-side** — generation *was* steered and then failed, and `generate_or`'s forced-first-**with-fallback** silently rendered a sibling | whatever made the forced descent fail (a re-firing forced quantifier, a depth budget) |
| sample renders the target construct correctly, still not witnessed | **parser-side** — the parser never *commits* to the target, typically a longest-match sibling spanning the same syntax | the grammar shape, or seed selection in the witness planner |

The last row is easy to misread: use `--parse-dump-ast-pretty` and check the discriminator `kind`,
because rule **entry** counts will show the target entered (speculatively) and tell you nothing.

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

## Is a reject grammar-owned or validator-owned? The message-source probe

Most PGEN parsers drive acceptance **entirely** from their generated parser. One
family — **regex** — additionally runs an out-of-band *compile-contract validator*
(`validate_regex_compile_contract`, `rust/src/regex_compile_validation.rs`) **after**
the generated parser accepts, to reject a handful of PCRE2-invalid patterns the
grammar does not yet reject on its own. Why that validator exists, and why it is
being migrated away one check at a time, is
[THE EBNF IS THE SINGLE SOURCE OF TRUTH](quality-and-closure-model.md#-the-ebnf-is-the-single-source-of-truth-for-the-accepted-language);
its regex-specific reference (architecture, the remaining check families, the
migration roadmap) is the regex parser book's **The Compile-Contract Validator**
chapter.

The registry runs the two layers **in series** (`parser_registry.rs` calls
`parse_full_regex()` first, then — only if the grammar accepted — the validator). That
ordering makes the **rejection message name its own source**, and a one-line probe
reads it off with zero guessing:

| What you observe | Who rejected | Meaning |
|---|---|---|
| exit `0` | nobody | **ACCEPT** — grammar and validator both passed |
| `Parser did not consume full input …` | the **grammar** | **GRAMMAR-reject** — the validator was never reached (it is *shadowed* / dead for this input) |
| any other message (e.g. `… character class …`, `unknown POSIX character class name`) | the **validator** | **VALIDATOR-reject** — the grammar *accepted* and only the validator rejected → the reject is **load-bearing** |

**"Load-bearing" is the pivotal concept.** A validator-owned reject is load-bearing
precisely because *deleting the validator would make that input **accept*** — the
grammar alone does not reject it. That is the exact test the EBNF-source-of-truth
migration turns on: the capstone that deletes the validator is safe only once *every*
load-bearing reject has been re-encoded in the grammar. (A session #72 measurement
found 8 check families / 54 hand-writable inputs still load-bearing — recorded in the
decision `project_regex_validator_deletion_blocked_load_bearing`.)

### Running the probe

The release `parseability_probe` embeds the current grammar **and** applies the
validator, so it is the single authoritative command. Feed each candidate pattern as a
file and classify by the message — do this on **both** profiles (`pcre2`, `relaxed`)
when the check is profile-sensitive:

```bash
PROBE=./rust/target/release/parseability_probe
classify() {   # $1 = pattern, $2 = profile (pcre2 | relaxed)
  printf '%s' "$1" > /tmp/rgx_in.txt
  out=$("$PROBE" --parse regex /tmp/rgx_in.txt --profile "$2" 2>&1); rc=$?
  if   [ "$rc" -eq 0 ];                                     then echo "ACCEPT"
  elif echo "$out" | grep -q "did not consume full input"; then echo "GRAMMAR-reject"
  else                                                          echo "VALIDATOR-reject"   # load-bearing
  fi
}
classify '[[:foo:]]' pcre2      # → VALIDATOR-reject (grammar accepts a bad POSIX name; only the validator rejects)
classify '[\B]'      pcre2      # → VALIDATOR-reject (escape-in-class the grammar does not yet reject)
classify '\pL'       pcre2      # → ACCEPT
```

⚠️ **Classify by MESSAGE, never by exit code alone.** A grammar reject and a validator
reject are *both* non-zero exits — a first-pass classifier that reads only the exit
code reports every reject as the same thing (this exact "too-good-to-be-true, all
shadowed" mistake was made and caught in session #72). The message text is the only
thing that distinguishes the two layers.

### The two jobs the probe does

1. **Scope a validator→grammar migration.** Before encoding a validator check in the
   EBNF, run the probe over that check's inputs: the ones that come back
   `VALIDATOR-reject` are the **load-bearing set** the grammar must learn to reject; the
   ones already `GRAMMAR-reject` are shadowed (nothing to migrate).
2. **Prove a migration is behavior-neutral.** After the grammar edit + regen, re-run the
   probe: every migrated input must flip `VALIDATOR-reject → GRAMMAR-reject`, and every
   *control* (the inputs that must stay valid) must stay `ACCEPT`. Pair that with the
   `regex_pcre2_compile_oracle_gate` tuple staying byte-identical **against your own
   pre-change run** — the corpus-wide tuple evolves as fidelity fixes land (the gate
   asserts bounds, so it stays green while the tuple improves; it is
   `2189/1879/262/48` as of `REGEX-PCRE2-FIDELITY.DOCSYNC.1`, 2026-07-17), so the
   proof is before→after equality, never equality to a constant from this book — and
   you have proven the accept/reject **set** is unchanged — only the reject's source
   (and its message) moved from the validator into the grammar.

This is the technique behind every `REGEX-PCRE2-FIDELITY.4.x` slice.

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

### Why this step is not optional

`ENGINE-UNIVERSAL-SERVICES.10` is the worked example. Two SystemVerilog rules
(`select_expression_lr_suffix`, `block_event_expression_lr_suffix`) reported
`parsed=true witnessed_target=false`, and their probe samples looked exactly like a routing
failure — one rendered `cross f, f { option.f = 3.5; }`, the `option` sibling that never enters
`bins_selection` at all. From the samples alone the cause was recorded as *"the reach planner cannot
route to a rule that did not exist when it built its graph"*, and that sentence reached both a task
leaf and a **tracked gate contract**.

The hop dump falsified it in one command. The chain was complete and correct — 21 hops ending
`("bins_selection","root/s3"), ("select_expression","root/s1/q")`, every OR steered and the
left-recursion quantifier forced. The planner was never wrong; the *render* was, one stack frame
away: a forced quantifier re-fired on every recursive re-entry of its own rule, so the forced descent
never terminated and the enclosing choice quietly fell back to a sibling.

Pair it with the generator's own decision trace when the path crosses a `?`/`*`:

```bash
PGEN_TRACE_VERBOSITY=high ./rust/target/debug/ast_pipeline grammars/<g>.ebnf \
  --report-certificate-coverage --count 1 --seed 0 2>&1 | grep "Quantifier decision"
```

A forced site prints `candidates=[1]` — exactly **one** repeat count, so a failure below it has no
fallback of its own and propagates up to the nearest choice, which *does* fall back. Counting those
lines is how a runaway forcing loop is caught: 119 at a single site in one probe was the `.10`
signature, against 5 after the fix.

---

## Forced-branch override dump (`PGEN_REACH_FORCED_OVERRIDE_DUMP`)

The reach-path dump above answers *"what did the planner decide?"*. This one answers the question
that follows it — *"and did the render actually obey?"*.

```bash
PGEN_REACH_FORCED_OVERRIDE_DUMP=1 ./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf \
  --report-certificate-coverage --grammar-profile sv_2017 \
  --entry-rule systemverilog_file --count 40 --seed 0 2>&1 | grep forced-override
```

When a reach-plan-forced branch fails, `generate_or` falls back to a sibling and returns `Ok`. That
fallback is *correct for termination and wrong for observability*: the directive is lost with no
trace at any verbosity, so a probe that "did not witness" is indistinguishable from a probe that was
never actually driven down the intended branch. The flag emits two paired records:

```
[forced-override] rule='select_expression_lr_suffix' path='root' forced_branch=2/3 outcome=failed
    reason="Stimuli generation depth exceeded max_depth=67 while expanding rule 'real_number'"
[forced-override] rule='select_expression_lr_suffix' path='root' forced_branch=2/3
    outcome=overridden rendered_branch=0
```

`outcome=failed` carries **why** the directive was lost — the generator's own error string;
`outcome=overridden` carries **what** was substituted. An `overridden` line with no `failed` line for
the same site means the forced branch was never attempted at all, which is a different bug (the
self-recursion suppression, or exhausted bypass fuel) in a different place.

### Why this instrument had to exist

`ENGINE-UNIVERSAL-SERVICES.11` is the worked example, and its point is that the *existing* record was
unreachable. The per-branch `failure_reasons` map does hold this answer — but
`--report-certificate-coverage` returns before any coverage artifact is written, and
`--coverage-output` cannot even be requested alongside it (it requires `--generate-stimuli`). So on
the one path where residual `UNKNOWN`s actually live, the reason was being computed, filed, and
discarded at process exit. Two candidate mechanisms sat in the task leaf with no way to choose
between them; the flag named the winner on its first run.

⚠️ **Read the reason, then question the budget.** `max_depth=67` above is
`reach_prefix_budget + min_derivation_depths[rule]` — the depth of the rule's *shallowest*
alternative, on a pass that was at that moment forcing a much deeper one. "Needs more budget" and
"the budget is scoped to the wrong thing" produce the identical symptom, so read where the number
came from, not just that it was hit. And a bigger global `--max-depth` is not the answer either: on
SystemVerilog it buys `UNKNOWN 1→0` while `sample_parse_failures` climbs `0→8→17` — witness samples
the real parser then rejects.

That particular budget is now correct: the target-own witness tier funds the alternative it forces
(`min_full_derivation_depth_of_node(alternative) + 1`), SystemVerilog's recognized union basis
reaches `UNKNOWN = 0` with an empty residual, and no `[forced-override]` line remains for that rule.
The mandatory-child and seed-sibling tiers still use the flat rule-scoped budget, so if you meet this
signature again, that is the first place to look.

---

## Residual classification under a profile (`PGEN_CERT_RESIDUAL_CLASSIFICATION`)

Under a dialect profile (e.g. `verilog_2005` on the SystemVerilog grammar), most of
the cert residual is `UNKNOWN` **because the profile excludes it** — gated SV-only
rules whose every referencing rule was pruned, and store-gated use-sites whose fact
*producers* are profile-unreachable. This read-only surface classifies the residual
mechanically, so the genuinely-actionable remainder stands out:

```bash
PGEN_CERT_RESIDUAL_CLASSIFICATION=1 PGEN_CERT_COVERAGE_DUMP_ALL=1 \
  ./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf \
  --report-certificate-coverage --grammar-profile verilog_2005 \
  --entry-rule systemverilog_file --count 40 --seed 0 \
  --cert-union-config sv_multi_entry_root:verilog_2005 \
  --cert-union-config library_text:verilog_2005
```

An extra block is printed after the `UNKNOWN` list (the default output is
byte-identical when the variable is unset — the analysis does not even run):

```
  RESIDUAL-CLASSIFICATION (read-only; PGEN_CERT_RESIDUAL_CLASSIFICATION): profile='verilog_2005' entry_universe=["systemverilog_file", "sv_multi_entry_root", "library_text"] store_analysis=active
    profile_entry_unreachable (…): […]         ← not positively reachable from ANY declared entry
    store_unproducible_under_profile (…): […]  ← a mandatory positive store-gate no live rule can feed
    genuine (…): […]                           ← the honest remainder to witness or adjudicate
```

Two sound, pure analyses back the classes. **P1** computes positive reachability from
the *declared entry universe* (the cert entry plus every `--cert-union-config` entry
present in the active tree — pass the alternate entries, or an entry-relative cohort
like `library_text`'s will honestly show as unreachable *from the single entry*) with
satisfiability-honest edges: a reference contributes nothing through an unsatisfiable
alternative, a profile-pruned mandatory sibling, or a lookahead. **P2** is a fixpoint
composed with P1: a fact-kind is producible only if some live rule emits it, a live
rule whose rule-level `@predicate` *requires* a positive fact-query (`has_fact`,
`fact_attribute_equals`, `fact_count_at_least` ≥ 1 — never `lacks_fact` or negations)
on an unproducible kind is dead, and deadness cascades until stable. If any live rule
carries `@import_from_library` the store analysis reports itself `DEGRADED-INERT`
(external artifacts could inject facts, so no unproducibility claim is safe).

The classification is diagnostic only — it never changes generation, the reach
passes, or the headline numbers. Promoting the two classes to checkable per-profile
`proof` certificates is tracked separately (`VERILOG-2005-PROFILE.6.7`).

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
| Static well-formedness | `ast_pipeline g.ebnf --lint-grammar` (`PGEN_LINT_DUMP_ALL=1` uncaps every class) | the DERIVED left-recursion verdict — `left_recursion_eliminated` (what the pass rewrote, by name) vs `left_recursion_unhandled` (cycles it declined; the runtime guard REJECTS those derivations, `A2.6`) — plus non-terminating errors and ordered-choice shadowing |
| **Indirect-LR survey** | `ast_pipeline g.ebnf --report-indirect-lr-plan [--indirect-lr-plan-json out.json]` (`PGEN_INDIRECT_LR_DUMP_ALL=1` uncaps routes/sites/declines) | for each cycle the lint reports as `left_recursion_unhandled`: which rule could absorb the chain (`X := X_lr_base ( X_lr_suffix )*`), the SUFFIX it would iterate, the CLONE cost, and each **starvation site** — a rule holding the candidate at its left corner with a non-empty residual, which PGEN's greedy non-backtracking `*` can starve. ⛔ The lint names the cycle, never the fix, and eliminating at the rule it names FIRST is a measured regression (`ENGINE-UNIVERSAL-SERVICES.13`) ⭐ Since slice 5 the header ALSO reports what the pass DID — `indirect_eliminated_base_rules=/indirect_clone_rules=/indirect_refusals=`, then one `✅ absorbed at '<rule>'` per rewrite and one `⛔ REFUSED '<rule>': <reason>` per decline. ⛔ Read that first: the survey runs on the POST-elimination grammar, so a knot the pass absorbed is simply absent, and a candidate still printed `MAY-ABSORB` is one the pass REFUSED. `--no-eliminate-indirect-left-recursion` gives you the before column on the same binary. |
| Memo statistics | `PGEN_REPORT_MEMO_STATS=1 parseability_probe --parse <g> f` | packrat hit/miss counts (perf triage) |
| Per-rule entry counts (exact, JSON) | `parseability_probe --parse <g> f --dump-rule-entry-counts-json c.json` | every rule-method entry (successful and backtracked) — the machine-readable dual of the live dashboard; deterministic, so a re-runnable cost-model oracle |
| Per-rule outcome counts (raw + committed + memo hits, JSON) | `parseability_probe --parse <g> f --dump-rule-outcome-counts-json o.json` | the same raw counters PLUS the committed (surviving) histogram from the transactional coverage stack PLUS per-rule memo-HIT counts — `raw − committed` = failed-speculation work per rule (committed keeps C3-B successful losers); `raw − memo_hits` = body executions; the choice-site + inline census's dynamic input |
| Fusibility census (derived-scanner gate) | `ast_pipeline g.ebnf --report-fusibility-census [--fusibility-entry-counts c1.json,…] [--fusibility-outcome-counts o1.json,…] [--fusibility-census-json out.json]` | per-rule scanner-compilability tiers, maximal fusible roots, disqualification histogram, regex-atom (`match_regex`) site count, the CHOICE-SITE census (every Or site's token-shaped branch subset), the DEGENERACY census (which rule-top-level sites qualify for the P2 degenerate-tournament byte-switch, with named blockers), the INLINE census (which rules are inline-eligible wrapper frames under the P1 gates, with named blockers + wrapper classes), the QUANT census (which min-0 quantified sites qualify for FIRST-guarded attempt elision, with named blockers), the CASCADE census (which rules can live inside a fused direct-coded region under the D2 effect-freedom gate, with roots/boundary call-outs), the CASCADE-PLAN (the shared D2-A acyclic-sub-region emission plan codegen consumes: sub-roots/internal/effect-reaching), the CASCADE-PLAN-B (the cyclic-spine increment's plan — every eligible rule fused, sub-roots = the full-fold roots, `thin_memo` = the cycle-participating fused rules that keep memo protection), the BOUNDARY-SCANNER-PLAN (the shared D3 gate the scan emitter consumes: which PLAN-B protocol boundaries get a direct-coded frameless `scan_<rule>` on the bare path, classed `text` / `span_transform` / `shaped_object`, with every dropped candidate's reasons named), and — joined with entry/outcome counts — the measured fusion ceiling, the discarded-work (merged-choice) kill surface, the degenerate-dispatch exposure, the inline frame exposure incl. the memo-hit share, the quantified-site attributable exposure, and the cascade-fold exposure with its post-fold committed floor (`PGEN_FUSIBILITY_DUMP_ALL=1` for per-rule + per-site verdicts) |
| **Per-rule memo insert/evict/replay census** | `memo_insert_evict_census.py t.log --verify o.json --rules r1,r2` (see [The Parseability Probe](parseability-probe-debug.md#is-the-memo-actually-serving-this-rule-per-rule-insert--evict--replay-census)) | splits the FUSED `rule_memo_hit_counts` into success replays vs cached failures vs stale-tainted evictions — ⛔ the aggregate hides whether packrat is working at all: 208 "hits" on one rule were 208 cached failures and **0** success replays. Cross-checks itself against the atomic counters and refuses on mismatch |
| **SV parse-cost ratchet** | `bash scripts/check_parse_cost_ratchet.sh` · `make -C rust sv_parse_cost_ratchet` · `make -C rust sv_parse_cost_rebaseline` | the standing guard (doctrine `PARSE-COST-RATCHET`) over a pinned 192-file corpus sample. ⭐ It binds on **exact integers** — rule entries, committed entries, memo hits — not wall clock, which is advisory on a ±50 % band that never fails the gate: a wall-clock-primary ratchet inherits the defect that once published `~11 %` for a **+24.3 %** regression. Tier 1 (~0.4 s, every commit) re-hashes the three inputs the baseline names and is a *proof*, not a sample: the binding metric is an exact function of exactly those. ⛔ **Declared blind spot** — the counters tick only in the PROTOCOL graph, so they cannot price the fused `cascade_*` graph the +24.3 % was measured on; the report states the bound on every run |
| **LR-family classifier check** (all ten families) | `python3 stimuli/sv/corpus_parse_cost.py --verify-families` | ~1 s, reads the generated artifacts rather than running a parse. Runs the parse-cost ratchet's left-recursion-family predicate over **every** generated parser's own `RULE_NAMES` registry and REFUSES on any declared name containing `_lr` that no emission shape claims. ⭐ Use it after touching either eliminator (`indirect_lr_elimination.rs`, `ast_pipeline/mod.rs`): the predicate is derived from their emission sites, so a new emitted shape silently under-counts the family until this refuses. Live: 10 parsers, **131** declared LR names, 131 classified — 127 in SystemVerilog, 2 each in the two annotation families, 0 elsewhere. ⛔ It is also the gate on the published SV declared-name count, which is derived here rather than carried (it was published as 128 and is 127). Rides the ratchet's tier 2, so it is **on-demand**, not every-commit |
| **Sampling a parse** (`/usr/bin/sample`) | `parseability_probe --parse <g> big.sv --profile P &` then `/usr/bin/sample $! 4 1 -f prof.txt` | the ONLY instrument that observes the FUSED `cascade_*` graph — every counter-based tool routes the parse to the PROTOCOL graph by construction. ⛔ **Four silent traps**, all measured: the denominator is the **worker thread** (the main thread is 100 % `__ulock_wait`, so the process total halves every percentage); **self-time answers "where is the CPU", not "who caused it"** (measured on SV's LR machinery: self ≈ 2 %, inclusive ≈ 24 %); the **linker folds** the generated parsers' identical helpers, so a per-family symbol name can be a lie; and a report has **four sections** — ingesting `Total number in stack …` as call-graph rows produced an **8×** wrong number whose conservation control stayed green. Full detail in `TOOLBOX.md` 3.8 |
| Well-formedness gen-AST (tests) | `PGEN_WELLFORMEDNESS_GEN_AST=<path> cargo test --lib …` | the gen-AST a well-formedness test loads |

---

## Cross-references

- [Debugging With `parseability_probe`](parseability-probe-debug.md) — trace
  levels, `--trace-rules`, the call-count dashboard, furthest-position, the
  predicate self-explaining trace (the parser-level detail this chapter builds on).
- [Grammar Well-Formedness & Well-Definedness](grammar-wellformedness.md) — what
  proof / witness / `UNKNOWN` mean and why `UNKNOWN`→0 is the trustworthiness number.
- [The Quality & Closure Model → THE EBNF IS THE SINGLE SOURCE OF TRUTH](quality-and-closure-model.md#-the-ebnf-is-the-single-source-of-truth-for-the-accepted-language)
  — why the one out-of-band validator exists, why it is a defect class being migrated
  away, and the load-bearing / deletability distinction the message-source probe measures.
- [The Semantic Store: Parser Memory](semantic-store.md) — the facts/scope model the
  store-gates query.
- KM card `cert-coverage-unknown-diagnostics` (`docs/knowledge/`) — the same
  protocol as a retrieval-indexed knowledge card.
- Decision `feedback_systematically_use_debug_toolbox` (`docs/decisions/`) — the
  standing directive that makes this protocol mandatory.
