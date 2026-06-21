# GRAMMAR-WELLFORMED.H.12.5.7.1 — M3 property/sequence temporal-operator WHY+WHERE + parse/reject adjudication

Tools-first WHY+WHERE for the M3 cluster the `H.12.5.7` leaf opened — the property/sequence
temporal-operator rules the certificate-coverage plannable-witness pass leaves `UNKNOWN`. The leaf
mandated *"Tools-first WHY+WHERE in the plannable-witness pass FIRST"*; this slice is that
investigation, and it **re-scopes** `H.12.5.7` by adjudicating each M3 rule to one of the attribution
rule's two causes (generator-reach gap vs grammar/parser defect).

> Slice `PGEN-GRAMMAR-WELLFORMED-0116` (**PURE-DOCS INVESTIGATION** — no grammar/Rust/generated/
> release/schema/ledger change; clippy not invoked). Status: `done`.
> Reads with [[feedback_why_and_where_before_solution]], [[feedback_tools_first_no_guessing]],
> [[feedback_no_codebase_change_without_tool_backed_facts]], [[feedback_be_alert_root_cause_fishy_immediately]],
> [[project_grammar_wellformedness_contract]], [[feedback_uvm_is_valid_sv]], [[feedback_always_signoff_decisions]].
> Parent: `GRAMMAR-WELLFORMED.H.12.5.7` (split by this slice). Opened by `H.12.5.4` (`-0083`),
> re-enumerated by `H.12.5.6.1` (`-0112`).

## Baseline (reproduced tools-first)

`PGEN_CERT_COVERAGE_DUMP_ALL=1 ast_pipeline grammars/systemverilog.ebnf --report-certificate-coverage
--grammar-profile sv_2017 --entry-rule systemverilog_file --count 40 --seed 0` (DEBUG binary):

```
CERTIFICATE-COVERAGE: grammar='systemverilog' entry='systemverilog_file' samples=40
  total=1289 proof=1 witness=1221 UNKNOWN=67 fully_certified=false (sample_parse_failures=0)
  plannable-rule reach pass: 1059 targeted; 927 witnessed, 41 parsed-but-routed-elsewhere,
                             437 probe samples did not re-parse, 29 generation failures
  19 UNKNOWN rules have NO reach path (the no_path set → H.12.6, NON-defects)
```

Confirms the `-0115` baseline (`UNKNOWN=67`, the M2a constraint reach-honesty pass already landed).

## The M3 cluster in the residual `67`

The property/sequence temporal-operator rules still `UNKNOWN` (extracted from the `DUMP_ALL` list):

| Rule | SV construct | grammar site |
|---|---|---|
| `kw_accept_on` / `kw_reject_on` / `kw_sync_accept_on` / `kw_sync_reject_on` | `accept_on (e) p` … | `property_expr_sv_2017` (prefix branch) |
| `kw_eventually` / `kw_s_eventually` | `eventually [r] p` / `s_eventually p` | `property_expr_sv_2017` (prefix) |
| `kw_nexttime` / `kw_s_nexttime` | `nexttime p` / `s_nexttime p` | `property_expr_sv_2017` (prefix) |
| `kw_s_always` | `s_always [r] p` | `property_expr_sv_2017` (prefix) |
| `kw_constant` | `nexttime ( constant e )? p` | inside the `nexttime` prefix branch |
| `property_case_item` | `case (e) i: p; endcase` item | `property_expr_sv_2017` case branch |
| `kw_until` / `kw_until_with` / `kw_s_until` / `kw_s_until_with` | `p until q` … | `property_expr_sv_2017` (**infix**, left-recursive) |
| `kw_intersect` / `kw_within` | `s intersect t` / `s within t` | `sequence_expr` (**infix**, left-recursive) |

(`kw_always`/`kw_s_eventually`-family duplicates aside, `kw_always` itself is **not** `UNKNOWN` — it
witnesses via the *procedural* `always` block, e.g. the plannable probe
`checker\foo ;always\foo ;endchecker`. The M3 set is exactly the keywords with **no** non-temporal use.
`property_qualifier`, `repeat_range`, `constant_cast`, `constraint_set`, `with_covergroup_expression` in
the `67` are NOT temporal — they are class-property / covergroup optional-gating or constraint residuals,
out of `H.12.5.7` scope.)

## The adjudication (`parseability_probe`, the attribution rule)

Minimal each-operator samples (`module m; logic a,b,x; assert property (<op>); endmodule`, profile
`2017`) through the **real** SV parser — *grammar-first suspicion* per the well-formedness contract: ask
the parser whether the construct is even acceptable BEFORE adding generator machinery.

| Construct | parser | verdict |
|---|---|---|
| `s_eventually a`, `eventually a`, `nexttime a`, `s_nexttime a`, `always a`, `s_always a` | **PASS** | reachable ⇒ generator-reach gap |
| `accept_on (a) b`, `reject_on (a) b`, `sync_accept_on (a) b`, `sync_reject_on (a) b` | **PASS** | reachable ⇒ generator-reach gap |
| `case (x) 1: a; endcase` (property_case_item) | **PASS** | reachable ⇒ generator-reach gap |
| `a until b`, `a s_until b`, `a until_with b`, `a s_until_with b` | **FAIL** `furthest=47` | parser **rejects** ⇒ grammar/parser defect |
| `a or b`, `a and b`, `a ##1 b` (control, the known H.12.5.8 class) | **FAIL** `furthest=47/49` | parser **rejects** ⇒ grammar/parser defect |

The `until`-family rejects at the **identical** byte position (the operator, right after the parsed
`a`) as the already-known `a or b` / `a and b` / `a ##1 b` defect. `eventually [1:2] a` /
`s_always [1:2] a` (the *bounded* forms) also fail, but the *unbounded* `eventually a` / `s_always a`
forms parse — so the keyword itself is witnessable; the bracket-range form is a separate, narrower
question deferred with the fix.

**Adjudication (attribution rule):**
- **Prefix operators** (`accept_on`, `eventually`, `nexttime`, `reject_on`, `s_always`,
  `s_eventually`, `s_nexttime`, `sync_accept_on`, `sync_reject_on`, `kw_constant`, `property_case_item`)
  → **generator-reach gap** (cause 1) → stays in `H.12.5.7` (lane 1).
- **Infix operators** (`until`, `until_with`, `s_until`, `s_until_with` at property level; `intersect`,
  `within` at sequence level) → **grammar/parser defect** (cause 2): they are the indirectly
  left-recursive `property_expr`/`sequence_expr` infix branches the parser cannot accept — the SAME
  LR-elimination defect as `a or b`/`a ##1 b`. They **cannot** be witnessed by any generator-reach
  improvement (per the contract: chasing a parser-rejected fragment in the generator is wasted effort).
  → **folded into `H.12.5.8`** (lane 2).

## WHERE (the plannable-witness pass, code-grounded)

`PGEN_CERT_COVERAGE_DEBUG_PROBES=1` (same run): for the **prefix** operators and the property
`until`-family there is **no `[plannable-probe]` line at all**, yet none of them appear in the
`19` `no_path` set. By the pass's own accounting in
`run_plannable_witness_pass` (`rust/src/ast_pipeline/stimuli_generator.rs:3026`):

- `set_reach_plan_for_rule_mode` returning **false** ⇒ `rule_attempted=false` ⇒ pushed to
  `report.no_path` (`:3204`). These rules are **not** in `no_path`, so a reach path **was** found.
- the debug print fires only inside the `witness_check` closure, i.e. only on a **successfully generated**
  sample (`Ok`). On a generation `Err`, `report.generation_failures += 1` and **no** print (`:3145-3160`).

So a rule that is *not in `no_path`* **and** *has no probe line* is, by construction, in the dump's
**"29 generation failures"**: the reach plan steers to the target's reference site, but
`generate_from_entry_with_optional_timeout` (`:3113`) **dead-ends/times out** building the forced
construct → no candidate is ever produced → the rule stays `UNKNOWN`. This is exactly the leaf's framing
("synthesizes NO candidate at all — cannot reach into the property/sequence-expression grammar").

Contrast: the **sequence-level** infix `intersect`/`within` **do** generate (e.g.
`sequence\foo ;5357.4…intersect 4248.5…endsequence`) but probe `parsed=false` — the LR-elim parse
defect, exactly as the `parseability_probe` adjudication predicts.

## WHY (root mechanism)

The reach path to a property temporal keyword forces descent through
`assertion_item → concurrent_assertion_statement → assert_property_statement → property_spec →
property_expr → property_expr_sv_2017 → <branch>`. `property_expr` is **indirectly left-recursive**
(`property_expr → property_expr_sv_2017 → property_expr` via the infix branches `property_expr kw_or
property_expr`, `property_expr kw_until property_expr`, …), so the transform pipeline LR-eliminates it
into a large `_lr_base`/`_lr_suffix` ordered choice. Building the *minimal forced construct* through that
deep, indirectly-recursive chain exceeds the plannable pass's per-target depth/visit budget (or the
forced branch interacts with the indirect recursion), so generation `Err`s before any sample is emitted.

This is the **deep-operator-chain reach class** the `rtl_const_expr` constructive-reach precedent
(`UNKNOWN 3` → `0`) and the RTL-FE-CLOSURE.5.x refinements (per-target two-tier budget; fire a
self-recursive forced branch once; prefer the non-self-recursive top-level alternative) were built for —
but `property_expr` is *deeper* and *indirectly* recursive, so those refinements do not yet cover it.
The fix direction is therefore a lane-1 **generator** extension, not a grammar change.

## OUTCOME — `H.12.5.7` re-scoped (split)

- **`H.12.5.7.1`** (this slice, `done`): WHY+WHERE + adjudication. PURE-DOCS, no behaviour change —
  SV stays `UNKNOWN=67`, the only non-fully-certified shipped grammar.
- **`H.12.5.7.2`** (`pending`, the new lane-1 frontier leaf): design + implement the plannable-reach
  extension so property/sequence-expression descent constructs a minimal witness for the **prefix**
  temporal operators (`accept_on`/`eventually`/`nexttime`/`reject_on`/`s_always`/`s_eventually`/
  `s_nexttime`/`sync_accept_on`/`sync_reject_on`/`kw_constant`/`property_case_item`). Model on the
  `rtl_const_expr` constructive-reach + RTL-FE-CLOSURE.5.x precedents. Generator-only, parser-agnostic,
  strictly additive (decisive A/B + global cert + `spf` at seeds 0/7/42; commit only an improvement).
- **`H.12.5.8`** (lane 2) **broadened** to explicitly include the property-level infix `until`-family
  (`until`/`until_with`/`s_until`/`s_until_with`) alongside the already-listed sequence binary operators
  (`and`/`or`/`intersect`/`within`/`##`) — all the indirectly-left-recursive infix property/sequence
  branches share the one LR-elimination parse defect.

## VERIFICATION (this slice)

- Tools-only investigation; no code/grammar/generated change ⇒ no clippy, no regen, no gate run.
- Determinism: the cert baseline (`UNKNOWN=67`, `witness=1221`, `spf=0`) matches `-0115` at seed 0;
  the `parseability_probe` verdicts are parser-deterministic.
- Live status UNCHANGED (SV remains `Mostly Done`, `UNKNOWN=67`).

## Artifacts (scratch — not tracked)

- `/tmp/h1257/cert_dump_seed0.txt` — `DUMP_ALL` enumeration of the `67`.
- `/tmp/h1257/cert_debugprobes.txt` — `DEBUG_PROBES` per-rule probe verdicts (intersect/within
  `parsed=false`; prefix/until = no line = generation failures).
- `/tmp/h1257/probes/*.sv` — the minimal per-operator `parseability_probe` adjudication samples.
