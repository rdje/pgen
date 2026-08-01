# GENERATED-LINT-CORRECTNESS: the generated-parser clippy stage can never be made strict, so a real correctness lint there would be invisible forever

## Metadata

- Tree ID: `GENERATED-LINT-CORRECTNESS`
- Status: `active` (opened 2026-07-27, session #213, by **direct director order** —
  *"take the right decision, you have information to take the proper signoff, sota
  decision"*, on being shown the 291-error generated-clippy stage)
- Family / slice-id prefix: `PGEN-GENERATED-LINT-CORRECTNESS-<NNNN>`
- Created: `2026-07-27`
- Owner: repo-local workflow
- **Frontier: `.7`** (opened 2026-08-01 — a SIXTH diagnosis family for the generator/coverage
  diagnostics, **director-approved CONDITIONALLY**: *"if you find a misbehavior in the enforcer"*.
  Measure first; `.4` refuted its own chartered family by pricing it, and a refusal is a valid
  outcome here too). `.1`–`.6` all `done` (2026-07-27, sessions #214–#216); `.5` + `.6` closed by
  `PGEN-GENERATED-LINT-CORRECTNESS-0007` on direct director order.
- Opened by: `LANG-CAPABILITY-AUDIT.10.4`, whose commit-workflow clippy run surfaced it.
  Deliberately NOT absorbed into that leaf — it is a separate defect class with a
  different owner (codegen emission shape, not the builtin allowlist).

## ⭐ THE ADJUDICATION (measured first, decided second)

⛔ **The 291 errors are NOT parser defects, and the generated output must NOT be
"fixed" to satisfy the lint.** Both classes were root-caused to their codegen sites:

| lint (count) | emitted in | codegen site | why it fires |
|---|---|---|---|
| `clippy::eq_op` — *"equal expressions as operands to `==`"* (158) | `generated/systemverilog_parser.rs` | `ast_based_generator.rs:4687` — `} else if #branch_policy_mode == "priority_first" {` (⚠️ **and two further emitters the charter missed** — see `.1`) | `#branch_policy_mode` is the grammar's configured branch policy, interpolated as a **string literal at codegen time**. When the grammar's policy *is* `priority_first`, the emission is literally `"priority_first" == "priority_first"` — a tautology. |
| `clippy::eq_op` (132) | `generated/rtl_frontend_parser.rs` | same | same |
| `clippy::overly_complex_bool_expr` — *"this boolean expression contains a logic bug"* (1) | `generated/systemverilog_preprocessor_parser.rs:41040` | `ast_based_generator.rs:7016` — `if skip_leading_whitespace && #allow_layout_skip_for_regexes {` | `#allow_layout_skip_for_regexes` is a **compile-time bool**, `false` for the whitespace-sensitive preprocessor grammar, so the guard reads `skip_leading_whitespace && false` and the block is **intentionally** dead for that grammar. |

⇒ in both cases the "tautology" / "dead branch" **IS the intended specialization**: codegen
resolves a per-grammar configuration constant and emits the already-decided form.
`clippy` is correct that the *expression* is degenerate and wrong that it is a bug.

⚠️ **The real gap is not the 291 — it is what the 291 make impossible.** `eq_op` and
`overly_complex_bool_expr` are **correctness-category** lints (`deny` by default). While
they fire on every build, the generated stage can never be run strictly
(`PGEN_CLIPPY_GENERATED_STRICT=1`), which means **a genuine correctness lint appearing in
a generated parser would be permanently invisible** — buried in 84k warnings and 291
pre-existing errors that nobody can triage. That is a live blind spot over the largest
shipped artifacts in the repo (`generated/systemverilog_parser.rs` is 150 MB).

This is the same shape as `LANG-CAPABILITY-AUDIT.10.1`'s finding about the linter
allowlist: **a check that cannot be run strictly is a check that reports nothing.**

## The decision (signoff-grade, director-delegated)

**Emit the folded form, do not silence the lint.** Ranked, with the reasoning:

1. ⭐ **PREFERRED — fold at codegen time.** The condition is already decided when the
   parser is generated, so emit the decided branch instead of the comparison: drop the
   `else if` chain arm entirely when `#branch_policy_mode` is known, and omit the
   guarded block entirely when `#allow_layout_skip_for_regexes` is `false`. This is
   strictly better on every axis — the lint stops firing **because the degenerate code
   is gone**, the generated artifacts shrink (relevant to the ⭐ SPEED north-star: the
   SV parser is 150 MB and these arms are in hot branch-selection code), and rustc was
   folding them anyway so runtime behaviour is unchanged.
2. **ACCEPTABLE FALLBACK — a scoped `#[allow(...)]` on the emitted item**, if (1) turns
   out to entangle the branch-policy emission more than it is worth. Scoped to the
   specific lint on the specific item, never a crate-level blanket.
3. ⛔ **REJECTED — leave it.** That is the status quo, and it is what makes the strict
   stage unreachable. It also silently trains every future reader to ignore the stage.
4. ⛔ **REJECTED — silence the whole generated stage / lower the lints to `warn`
   crate-wide.** That converts a measurable gap into an unmeasurable one and would hide
   a future genuine correctness lint, which is the entire problem being solved.

**Acceptance for the tree:** `PGEN_CLIPPY_GENERATED_STRICT=1` passes on all 11 generated
parsers, with byte-identity **deliberately NOT expected** (the generated output changes
by construction — that is the point), so the proof obligation is instead: every parser
still compiles, the full gate battery stays green, and parse verdicts are unchanged on
the tracked corpora.

## Leaves

### `.1` — fold the two known emission sites and re-measure (`done`)

- **Status: `done`** — `PGEN-GENERATED-LINT-CORRECTNESS-0002`, session #214 (2026-07-27).
  **CODE CHANGE**: `rust/src/ast_pipeline/ast_based_generator.rs` +
  `rust/src/ast_pipeline/ast_based_generator/cascade.rs` +
  `rust/src/ast_pipeline/ast_based_generator/scan.rs`. All 11 generated parsers
  regenerated; **byte-identity DELIBERATELY NOT expected** (the emitted output changing
  IS the deliverable) — the proof obligation is unchanged parse verdicts, and it is met.
- **`PGEN_CLIPPY_GENERATED_STRICT=1` NOW PASSES: 291 → 0 errors.** The check that could
  never be run strictly can now be run strictly.

#### ⭐⭐ THE CHARTER NAMED ONE EMITTER; THERE ARE **THREE**

The charter located the `eq_op` source at `ast_based_generator.rs:4687` and stopped.
Folding only that site took `generated/json_parser.rs` from 90 degenerate comparisons to
**45 — exactly half**, and the residue had the *identical* 2:1 `ordered`:`priority_first`
ratio. That is not a leftover, it is a **duplicate emitter**, and the measurement said so
before any reading did. The reason the first grep missed them: they live in
`rust/src/ast_pipeline/ast_based_generator/` — a **subdirectory**, invisible to
`grep 'branch_policy_mode' rust/src/ast_pipeline/*.rs`.

| emitter | graph | folded sites |
|---|---|---|
| `ast_based_generator.rs` | protocol / memoized | `:4499` + `:4685` (`== "ordered"`), `:4687` (`== "priority_first"`) |
| `ast_based_generator/cascade.rs` | fused cascade (bare) | `:964`/`:966` (`should_take_chain`), `:1016` + `:1096` (the two ordered guards — island and non-island) |
| `ast_based_generator/scan.rs` | scan (match-only) | `:584`/`:586` (`should_take_chain`), `:626` (ordered guard) |

⇒ **a codegen surface that exists in three graphs must be swept in all three**; this is
now the standing instruction for leaf `.2`.

#### The fix

Every site had the same shape: a **codegen-time constant** interpolated as a string
literal and then re-asked with `==` in the emitted parser. The fold resolves it in the
generator and emits **only the decided form**:

- `branch_policy` (`SemanticBranchPolicy::{LongestMatch, Ordered, PriorityFirst}`) now
  selects ONE `should_take` cascade instead of emitting all three behind statically
  decided comparisons, and the `ordered` keep-first-winner guard is emitted **only under
  `ordered`**. `branch_policy_mode` (the `as_str()` spelling) survives in
  `ast_based_generator.rs` because it is still interpolated into emitted TRACE strings,
  where a literal is exactly what is wanted; in `cascade.rs`/`scan.rs` (which emit no
  trace strings) the binding had no consumer left and was deleted.
- `allow_layout_skip_for_regexes` (from `@whitespace_sensitive:`) now selects between two
  `match_regex` preludes. For a whitespace-sensitive grammar the layout-skip block, the
  `can_match_empty` flag it fed, and `consume_layout_for_regex` itself are **not
  emitted** — otherwise the fold would have traded one lint for a `dead_code` warning.
  The `skip_leading_whitespace` parameter is emitted as `_skip_leading_whitespace` in
  that arm for the same reason.

#### Measured, before → after

| | before | after |
|---|---|---|
| `PGEN_CLIPPY_GENERATED_STRICT=1` generated errors | **291** | **0** |
| statically-degenerate emissions, all 11 artifacts | **21,201** | **0** |
| shipped artifact bytes | 252,782,743 (241.1 MB) | 230,524,147 (219.8 MB) — **−22.3 MB, −8.8%** |
| `systemverilog_parser.rs` | 149,751,672 | 136,452,772 (**−13.3 MB**) |
| new source warnings (`cargo check --lib`) | 42 | 42 |
| new generated warnings (lib build) | 35,981 | 35,981 |

The 291 clippy errors were the **identical-operand subset** of those 21,201 — the other
20,910 (`"longest_match" == "ordered"` and friends) were never lint-visible at all, which
is why the size win is an order of magnitude larger than the error count suggested. This
is directly on the ⭐ SPEED north-star: the removed arms sit in hot branch-selection code.

#### ⭐ The decisive oracle — the interpreter did NOT change

`parse_harness_combinator_gate` compares the **interpreter** (which still evaluates
`SemanticBranchPolicy` at runtime, from the enum) against the **compile-and-run generated
parser** (now folded at codegen), byte-for-byte, on per-combinator isolating grammars.
All four branch-policy discrimination rows are `CLEAN`:

```
choice_longest_default     CLEAN  samples=4 diverge=0 anchor_miss=0
choice_longest_explicit    CLEAN  samples=3 diverge=0 anchor_miss=0
choice_ordered             CLEAN  samples=3 diverge=0 anchor_miss=0
choice_priority_first      CLEAN  samples=3 diverge=0 anchor_miss=0
```

No other instrument in the repo isolates "did folding the policy change which branch
wins?" this exactly — a green suite here is worth more than any amount of reading.

#### ⚠️ FOUND WHILE VERIFYING — `ast_dump_contract_gate` HAS BEEN RED SINCE `7219547c`

Not caused by this leaf, and **proven so mechanically rather than asserted**: the refusal
string (*"no entry rule is declared"*) is in `rust/src/main.rs` at HEAD, introduced by
`7219547c` (`PGEN-QUANT-PLUS-ITER-0004`, the `@entry: true` mandate); this leaf's diff
touches only the three codegen emitters; and the gate dies on its FIRST step at grammar
**load**, before any parser is emitted.

⭐ **Why the `-0004` migration could not see it**: that leaf migrated 58 tracked `.ebnf`
files + 68 synthetic harness grammars. This fixture is a **raw-AST JSON literal
heredoc'd inside `rust/scripts/ast_dump_contract_gate.sh`** (`grammar_name` `mini` /
`mini_large`) — not an `.ebnf` file at all, so an `.ebnf`-shaped sweep is structurally
blind to it. ⭐ **Why nobody noticed**: `ast_dump_contract_gate` is referenced by no
aggregate (`sota_exit_gate`, `ci_workflow_local_gate`) and no CI workflow, so nothing
re-ran it after the mandate landed.

⇒ **the same disease this tree exists to treat, one layer up** — the tree's thesis is *a
check that cannot be run strictly reports nothing*; here is *a check that cannot be run
at all, and nothing noticed for a day*. Routed to `QUANT-PLUS-ITER.4` (which already owns
the `@entry` provenance bound). Deliberately NOT fixed here: it belongs to another tree,
and the code-change doctrine requires the owning leaf first.

#### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — census of the shipped artifacts:
      `"priority_first" == "priority_first"` × **158** in `systemverilog_parser.rs` +
      × **132** in `rtl_frontend_parser.rs` (= 290 `clippy::eq_op`), plus
      `skip_leading_whitespace && false` × **1** in
      `systemverilog_preprocessor_parser.rs` (`clippy::overly_complex_bool_expr`) —
      **291**, exactly the charter's census, so `PGEN_CLIPPY_GENERATED_STRICT=1` could
      never pass. Driver
      `docs/tasks/artifacts/generated_lint_correctness/run_degenerate_emission_sweep.sh`;
      `TOTAL_DEGENERATE_before=21201`.
- [x] **ROOT CAUSE (WHY + WHERE)** — a codegen-time constant interpolated as a string
      literal and compared at "runtime" in the emitted parser, at **three** emitters:
      `ast_based_generator.rs:4499/:4685/:4687` (protocol graph),
      `ast_based_generator/cascade.rs:964/:966/:1016/:1096` (fused cascade graph),
      `ast_based_generator/scan.rs:584/:586/:626` (scan graph); plus
      `ast_based_generator.rs:7006/:7016` for `#allow_layout_skip_for_regexes`. The
      third-emitter discovery was **forced by measurement**, not reading: folding only
      the charter's site left `json_parser.rs` at exactly half its degenerate count with
      an unchanged 2:1 ratio.
- [x] **FIX** — emit the decided form (fix-hierarchy tier: **codegen**, the only tier
      that can express "this constant is already known"). No lint suppression, no
      `#[allow]`, no grammar change, no runtime cost.
- [x] **ADDRESSED (verified)** — `PGEN_CLIPPY_GENERATED_STRICT=1
      make -C rust SHELL=/opt/homebrew/bin/bash clippy_on_rust_change` → **exit 0**,
      generated stage `pass`, `errors=0 eq_op=0 overly_complex_bool_expr=0` in
      `rust/target/clippy_gate/logs/clippy_generated_all_targets.log` (**291 → 0**);
      degenerate emissions **21,201 → 0**; artifacts **241.1 MB → 219.8 MB**.
- [x] **NO REGRESSION** — `parse_harness_combinator_gate` **27/27 CLEAN** (incl. all four
      `choice_*` branch-policy rows, `diverge=0`); `parse_harness_semantic_gate`
      **36/36 CLEAN**; `parse_harness_equivalence_gate` **4 passed / 0 failed**
      (interpreter == generated, byte-identical, seeds 0/7/42); lib suite **1027 passed /
      0 failed / 29 ignored** (the `-0004` baseline exactly);
      `rtl_frontend_generated_contract_gate` **green** (both probe and handwritten
      replay); certificate coverage `UNKNOWN=0 fully_certified=true
      sample_parse_failures=0` and deterministic at seeds 0/7/42 for **json / regex /
      vhdl / systemverilog_preprocessor / rtl_frontend / rtl_const_expr /
      systemverilog** — the last two being precisely the grammars the fold changed most
      (158 + 132 of the 290 `eq_op` sites); `regex_pcre2_compile_oracle_gate` green;
      clippy source-strict **0 errors**; source warnings **42 → 42** and generated
      warnings **35,981 → 35,981** (zero new); `scripts/check_doctrines.sh` **9/9 PASS**.
- [x] **LOCKSTEP** — book chapter `docs/book/src/quality-and-closure-model.md` (the
      generated-parser lint posture), `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`,
      `docs/TASK_TREE.md`. No release / schema / ledger / integration-contract movement:
      the generated parsers' observable behaviour is unchanged (that is what the oracles
      above prove), only their emitted text is smaller.

Evidence: `docs/tasks/artifacts/generated_lint_correctness/verification_capture.txt`,
`.../degenerate_emission_sweep_capture.txt`,
`.../run_degenerate_emission_sweep.sh`.

### `.2` — sweep for other statically-degenerate emissions (`done`)

- **Status: `done`** — `PGEN-GENERATED-LINT-CORRECTNESS-0003`, session #214.
  **CODE CHANGE**: the three emitters again, plus
  `rust/src/parse_harness_combinator_suite.rs` (three new rows + a discrimination proof
  + one re-pinned test). All 11 parsers regenerated; byte-identity again deliberately
  NOT expected.
- **The sweep is CLOSED, and it was enumerable exactly as the charter predicted.** Every
  `#`-interpolated identifier used in emitted control flow, across all six files that
  emit parser code, was classified — the full table is in
  `docs/tasks/artifacts/generated_lint_correctness/verification_capture_leaf2.txt`.
  Four constants were degenerate and are folded; three interpolations
  (`#guard_condition`, `#failure_condition`, `#base_code`/`#base_value`) are genuine
  runtime expressions and are correct as-is; the rest are payloads (trace-string
  arguments, or values passed to a shared helper whose conditional is over a real
  runtime PARAMETER — `#negative_case_enabled` into `inlined_frame_call` is the case to
  not "fix").

| constant | emitted as | sites | emissions |
|---|---|---|---|
| `#associativity_mode` | `match "left" { "right" => …, _ => false }` | all 3 emitters | **7,064** |
| `#negative_case_enabled` | `if false { self.record_negative_case_failure(…) }` | protocol, once per rule | **2,555** |
| `#allow_layout_skip_for_terminals` | `if true { self.consume_layout_for_terminal(…) }` | protocol ×4 | **60** together |
| `#allow_trailing_layout` | `if true { …("<EOF>") }` | protocol ×2 | |

  Measured: **9,679 → 0**; artifacts **219.8 MB → 210.8 MB (−4.1%)**;
  cumulative with `.1`, **30,880** statically-decided expressions removed and
  **252.8 MB → 221.1 MB (−12.5%)**. `PGEN_CLIPPY_GENERATED_STRICT=1` still passes with
  **0 errors**, and the generated stage's style noise went **80,402 → 78,858**.
- ⛔ **Three knock-ons that would have traded one lint for others** — each fixed by
  emitting less, never by an `#[allow]`:
  - `consume_layout_for_terminal` loses BOTH call families when a grammar is
    whitespace-sensitive on terminals AND on trailing layout (`grammars/regex.ebnf` is
    the shipped case), so its emission is now gated on `terminals || trailing`;
  - `nonassoc_tie` and its `if nonassoc_tie { … }` failure arm are emitted ONLY under
    `nonassoc`, where the flag can actually become true;
  - `best_branch_index` is read only by the `right`/`nonassoc` tie-breaks and the
    branch-start-effect lookup, so it (and, in `scan.rs`, `current_branch_index` with
    it) is emitted only when something reads it. ⭐ `cascade.rs` needed no such gate —
    its tape writes `DerivEvent::OrWinner(best_branch_index)` unconditionally. The three
    emitters are NOT interchangeable and each had to be read.

#### ⚠️⚠️ THE FOLD SHIPPED 7,482 FRESH INSTANCES OF THE CLASS IT REMOVES

The first left-associative fold emitted `} else if candidate_priority < best_priority {
false } else { false }` — `clippy::needless_bool`, **7,482 times**, taking the generated
stage from 80,402 to **93,822** warnings. Merging the two arms then left
`else if candidate_priority > best_priority { true } else { false }`, which is the OTHER
`needless_bool` phrasing and still 7,482. Only collapsing the tail to the comparison
itself (`else { candidate_priority > best_priority }` — `<` and `==` both answer "do not
take") cleared it.

⭐ **THE LESSON, and it generalizes past this tree: the leaf's acceptance was an ERROR
count (291 → 0), and the error count was GREEN through all three of those states.** A
fold that watches only the gate's verdict can ship thousands of instances of the very
shape it exists to remove. **Re-measure the whole lint census, not the gate's verdict.**

#### ⭐ THE ORACLE GAP THIS LEAF HAD TO CLOSE BEFORE IT COULD TICK ITS OWN BOX

Before `.2`, **nothing in the repository exercised `@associativity`** — no combinator
row, no semantic row, no integration test, and no tracked grammar declares it
(`grammars/ebnf.ebnf` only *documents* the directive). Ticking NO-REGRESSION on the
largest fold in this leaf would have meant ticking it on instruments that structurally
could not see the change — the exact failure this tree is named after.

Three rows now sit in `parse_harness_combinator_suite`, sharing one grammar whose two
alts TIE on consumed length and priority so only the tie-break can pick a winner:

```
@associativity: <left|right|nonassoc>
start := pair | fused
pair  := "x" "y"
fused := "xy"
```

⭐ **And a DISCRIMINATION proof, because agreement alone would be vacuous** — both sides
compile from the same generator, so a fold that inverted or dropped the directive could
still make them agree. `SampleOutcome` gained an `interp_ast` field, and the gate now
asserts (a) the `(left, right, nonassoc)` verdicts on `"xy"` are
`(accept, accept, REJECT)`, so the tie is real and `nonassoc` genuinely fails the whole
choice, and (b) `left`'s typed AST **differs from** `right`'s, so the rows can actually
see an inversion.

#### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — census of the `.1`-vintage artifacts:
      `match "left" {` × **7,064**, `if false {` × **2,555**, `if true {` × **60** =
      **9,679** statically-decided expressions still shipped. None of them is
      lint-visible (`eq_op` needs identical operands), which is why `.1`'s green error
      count did not close the class.
- [x] **ROOT CAUSE (WHY + WHERE)** — the same shape as `.1`: a codegen-time constant
      interpolated into emitted control flow. Enumerated mechanically **at the codegen
      sites**, across all six files that emit parser code, with:

      ```
      grep -nE '(^|[^a-zA-Z_])(if|else if|while|match)[[:space:]]+#[a-z_]+|
                &&[[:space:]]*#[a-z_]+|\|\|[[:space:]]*#[a-z_]+|#[a-z_]+[[:space:]]*(==|!=)' \
        rust/src/ast_pipeline/ast_code_generator.rs \
        rust/src/ast_pipeline/ast_based_generator.rs \
        rust/src/ast_pipeline/ast_return_transform.rs \
        rust/src/ast_pipeline/ast_based_generator/scan.rs \
        rust/src/ast_pipeline/ast_based_generator/cascade/value.rs \
        rust/src/ast_pipeline/ast_based_generator/cascade.rs
      ```

      cross-referenced against every interpolated identifier whose generator-side binding
      is a compile-time `bool`/`&'static str`. WHERE, by file:line: `#associativity_mode`
      (`ast_based_generator.rs:4256`, `cascade.rs:962`, `scan.rs:578`),
      `#negative_case_enabled` (`ast_based_generator.rs:3990`),
      `#allow_layout_skip_for_terminals` (`:6958/:6973/:8507/:8521`) and
      `#allow_trailing_layout` (`:2212/:2240`). The artifact-side confirmation (WHY it
      matters, and how much) is `run_degenerate_emission_sweep.sh --capture-before`:
      `TOTAL_DEGENERATE_before=9679`. The post-fold behavioural confirmation used
      `--report-certificate-coverage` on all seven claim-carrying grammars (see the
      NO-REGRESSION box).
- [x] **FIX** — emit the decided form (tier: **codegen**), plus the three conditional
      bindings the fold makes unread. No lint suppression, no grammar change.
- [x] **ADDRESSED (verified)** — degenerate emissions **9,679 → 0**; artifacts
      **219.8 MB → 210.8 MB**; `PGEN_CLIPPY_GENERATED_STRICT=1` still **exit 0 /
      0 errors**, generated-stage warnings **80,402 → 78,858** and
      `clippy::needless_bool` **0** (after the two-step correction above).
- [x] **NO REGRESSION** — `parse_harness_combinator_gate` **30/30 CLEAN** (the four
      `choice_*` rows AND the three NEW `assoc_*` rows, `diverge=0`) with the
      discrimination proof holding; `parse_harness_semantic_gate` **36/36 CLEAN**;
      `parse_harness_equivalence_gate` **4 passed / 0 failed**; lib suite **1027 passed
      / 0 failed / 29 ignored**; `rtl_frontend_generated_contract_gate` and
      `rtl_const_expr_cert_gate` green;
      `./rust/target/debug/ast_pipeline grammars/<g>.ebnf --report-certificate-coverage
      --count 40 --seed {0,7,42}` gives `UNKNOWN=0 fully_certified=true
      sample_parse_failures=0 proof_reverify_failures=0` on all **7** claim-carrying
      grammars, identical across the three seeds; `regex_pcre2_compile_oracle_gate` green;
      clippy source-strict **0 errors**; source warnings **42 → 42** and generated
      rustc warnings **35,981 → 35,964**; `scripts/check_doctrines.sh` **9/9 PASS**.
- [x] **LOCKSTEP** — book chapter `docs/book/src/quality-and-closure-model.md`,
      `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`, `docs/TASK_TREE.md`,
      `docs/reference/RUST_CODEBASE_ANALYSIS.md`. No release / schema / ledger /
      integration-contract movement — observable parse behaviour is unchanged.

⚠️ **A FOURTH TEST WAS PINNING THE SHAPE — re-pinned, not deleted.**
`semantic_usage_codegen_emits_priority_and_associativity_tiebreak_logic` asserted
`rendered.contains("match \"right\"")`, i.e. it required the emitted parser to contain
the degenerate `match` over an associativity literal. It went red the moment the fold
landed — the honest outcome — and is the FOURTH occurrence of the banked shape *"a
`contains` assertion over rendered tokens tests SPELLING, not BEHAVIOUR"*. Now it
requires `current_branch_index > best_branch_index` and FORBIDS `match "right"`.

⚠️⚠️ **OPS — THIS FINDING WAS WRONG AND IS CORRECTED IN PLACE (same session).** Two
heavy jobs were killed by the OS (`signal: 15, SIGTERM`) at process-tree peaks of
**15,079 MB** and **15,253 MB**. The first version of this note concluded *"the 16 GB
guard budget is no longer enough — use `--budget-mb 18432`"*.

⛔ **That is refuted by the guard's own markers**, which had already been written when
the claim was made:

| marker | budget_mb | peak_rss_mb | reason | exit | last_free_pct |
|---|---|---|---|---|---|
| `guard.24252` (`make focus_*`) | 16384 | 15,079 | `none` | 2 | 83 |
| `guard.54226` (`parse_harness_combinator_gate`) | **18432** | 15,253 | `none` | 2 | 82 |

**The second kill happened with the "fix" already in effect.** Neither was a guard kill:
`reason=none` in both, both peaks are *below* their budget, and system-free was 82–83%,
so the RSS budget AND the system-free floor both saw nothing. The mechanism is macOS
memory pressure acting on a spike the guard's 5-second sampler never observed, on a 24 GB
host whose swap is **4,096 MB total with 2,509 MB already used** (~1.5 GB free).

⛔ **Raising the budget makes the guard LESS protective, not more** — it licenses 18.4 GB
on a machine that starts killing near 15. The re-runs succeeded because less incremental
work remained (a lower peak) and transient pressure had eased, not because of the budget.

⚠️ **This exact failure mode was ALREADY RECORDED** in `DEVELOPMENT_NOTES.md` (`reason=none`,
periodic sampling blind to spikes, the capped swap). It was re-discovered here and given
the wrong remedy — a RE-MEASURE failure of
[`docs/decisions/feedback_read_prior_art_before_designing.md`] on a fact one grep away.
⇒ routed to **`OPS-MEMSAFE.4`**, which owns the real question (make the guard SEE spikes,
and decide what the binding constraint on this host actually is).

⛔ The first kill left the artifact tree in a MIXED vintage (2 of 11 regenerated); the
whole regeneration was redone rather than patched, because a mixed tree makes every
downstream measurement unattributable. **That part of the note stands.**

⚠️ **A GAP IN THE DOCTRINE ENFORCER, found by this leaf and recorded rather than worked
around.** `scripts/check_diagnosis_evidence.sh` requires the ROOT CAUSE box to be backed
by a `DIAGNOSIS_SIG` token, and its list models three defect families — correctness
(`CERTIFICATE-COVERAGE:` / `[plannable-probe]` / `--trace-rules` / `furthest_position=`),
performance (`self-time` / `flamegraph`), and build integrity (`error[E….]` /
`could not compile`). A **codegen-emission** defect is a fourth family: there is no parse
to trace, no run to sample and no compiler error — the WHY+WHERE comes from enumerating
the emission sites in the generator and censusing the emitted artifacts. Two consequences,
both measured:

1. This leaf's genuine root-cause instrument (the grep above + the census driver) matches
   no token in the list, so the leaf initially FAILED the check despite being fully
   evidence-backed.
2. ⛔ **`.1` passed only because the check greps *all* staged task files, and its commit
   also staged `docs/tasks/QUANT-PLUS-ITER.md`, which carries such tokens for its own
   unrelated reasons.** A co-staged file can therefore satisfy the signature requirement
   for a leaf that does not carry it — the check is weaker than it reads.

Neither is fixed here (that is an enforcer change, and it needs its own leaf); both are
routed to `.3`, which is already the "make the check real" leaf. The boxes above are
backed by the `--report-certificate-coverage` runs, which are real and were run, but the
honest note is that they are the NO-REGRESSION evidence, not the root-cause instrument.

Evidence:
`docs/tasks/artifacts/generated_lint_correctness/verification_capture_leaf2.txt`,
`.../degenerate_emission_sweep_capture_leaf2.txt`,
`.../run_degenerate_emission_sweep.sh` (extended with the `.2` patterns).

### `.3` — promote the generated-clippy correctness subset to a gate (`done`)

- **Status: `done`** — `PGEN-GENERATED-LINT-CORRECTNESS-0004`, session #215 (2026-07-27).
  **NOT a code change** in the enforcer's sense (no `grammars/*.ebnf`, no `rust/src/*`, no
  `generated/*`, no `ast_shape_contract/*.json`): gate scripts, a tracked contract, a
  `Makefile` lane, a workflow, the doctrine enforcer, and docs. **All 11 generated parsers
  are untouched** — nothing in this leaf can move parse behaviour, which is why the proof
  obligation below is gate-shaped rather than oracle-shaped.

#### ⭐⭐⭐ THE HOLE WAS NOT "TOO NARROW" — THE CHECK WAS NEVER RUN BY ANYTHING

The charter assumed the strict stage merely needed narrowing to the correctness category.
Measurement said otherwise. A repo-wide sweep for `PGEN_CLIPPY_GENERATED_STRICT` found it is
set by **no gate, no aggregate, no CI workflow, and no `Makefile` target** — every hit is
prose (`COMMIT.md`, `PGEN_USER_GUIDE.md`, `DEVELOPMENT_NOTES.md`, task records). Its default
was `0`. And `clippy_on_rust_change` — the only thing that runs the generated stage at all —
is itself referenced by no aggregate and no CI workflow.

⇒ when `.1` + `.2` drove the count 291 → 0, **nothing was left holding it there.** The
0 was unguarded from the moment it landed. That is the same shape `.1` found in
`ast_dump_contract_gate` ("belongs to no aggregate and no CI workflow, so nothing re-ran
it"), one layer up: not *"cannot run strictly"* but *"is never asked to"*.

#### ⭐⭐ WHAT THE MEASUREMENT SAID ABOUT THE SUBSET ITSELF

`clippy::correctness` is **deny-by-default**, so the existing strict flag was *already*
gating exactly the correctness category — the narrowing the charter asked for was, in
effect, already the semantics. Censused at `730419a2` over the generated stage:

| category | lints in group | firing today | where |
|---|---|---|---|
| `clippy::correctness` | 68 | **0** | — |
| `clippy::suspicious` | 82 | **2** | BOTH in `rust/src/`, neither in `generated/` |
| everything else | — | 78,858 warnings | overwhelmingly `generated/*.rs` |

The two `suspicious` hits are `clippy::empty_line_after_doc_comments` at
`rust/src/ast_pipeline/ast_based_generator.rs:6228` and
`clippy::unnecessary_get_then_check` at `rust/src/ast_pipeline/mod.rs:5292`. They belong to
the SOURCE lane, so `suspicious` is recorded in the contract as **considered-and-deferred**
with that measurement and a named revisit condition — not silently omitted.

⇒ the real deliverables are (a) make the check actually run, and (b) pin the subset so it
cannot narrow **without anyone noticing**, which a deny-by-default reliance cannot do: if a
future clippy demotes `eq_op` out of `correctness`, the stage silently stops failing on it.

#### ⭐⭐⭐ THE VACUITY TRAP — a generated-code gate that lints NOTHING and exits 0

`rust/build.rs:90-168` sets each `has_generated_<name>_parser` cfg **only when the artifact
`is_file()`** (verified in source, all 8 cfg-guarded parsers), and `generated/` is untracked
(`.gitignore:24`). Therefore in a clean checkout, on a CI runner, or inside the
`ci_workflow_local_gate` export dir (`copy_tracked_worktree` copies `git ls-files` only),
`--features generated_parsers` compiles with **zero** cfg-guarded generated parsers — and a
naive "lint the generated stage" step would lint no generated code and **exit 0**.

That is the tree's own unifying principle turned on the tree's own gate, so the gate carries
two anti-vacuity checks and **REFUSES with exit 2** — never a pass — when either fails:

1. **artifact-presence** — every contract-required artifact exists and is non-empty.
2. **cfg-census** — every required artifact's `has_generated_*` cfg appears in **cargo's own**
   `{"reason":"build-script-executed",…,"cfgs":[…]}` JSON message for *this* run. Measured:
   cargo replays that message even from a warm cache, so it is cargo's testimony that the
   artifact was compiled INTO the linted unit, not a re-reading of the disk.

⚠️ **An asymmetry found while wiring it:** `return_annotation_parser.rs` and
`semantic_annotation_parser.rs` are `include!`d in `rust/src/lib.rs:72,78` by hard relative
path with **no cfg guard**, so they emit no cfg — their absence is a hard *compile error*
rather than a silent exclusion. That is a stronger guarantee, and it is why the contract
marks them `cfg_guarded: false` instead of pretending the census covers all ten.

#### ⭐ A THIRD VACUITY MODE, CHECKED AND CLEARED

A cached clippy verdict would also lint nothing. Measured on the GREEN run: the `pgen` lib
target reports **`fresh: false`** (only third-party deps and build scripts were cached), so
the analysis really ran. And it will keep running when it matters — rustc's dep-info for the
`generated_parsers` build (`rust/target/debug/deps/pgen-f17101ed87693af9.d`) lists **all 11**
generated artifacts, so any change to an emission invalidates the fingerprint.

#### What was built

| surface | what it does |
|---|---|
| `rust/test_data/grammar_quality/generated_clippy_correctness_contract_v0.json` | pins the 68-lint `clippy::correctness` roster BY NAME + the 11 artifacts + the two anti-vacuity checks + the deferred-`suspicious` record |
| `rust/scripts/generated_clippy_correctness_gate.sh` | the gate. Exit **0** pass / **1** findings / **2** REFUSE (could not run soundly). Denies the *group* (so future clippy additions are covered) **and** every pinned *name* (so a departure is loud) |
| `make -C rust generated_clippy_correctness_gate` | the repo-standard lane |
| `make -C rust generated_clippy_correctness_policy` (`--policy-only`) | the no-cargo subset: presence + roster integrity |
| `rust/scripts/clippy_on_rust_change.sh` | **`PGEN_CLIPPY_GENERATED_STRICT` now defaults to `1`**, plus the `--policy-only` stage; the policy files themselves now trigger the flow |
| `.github/workflows/generated-clippy-correctness-gate.yml` | hosted lane — **regenerates the parsers first**, because a bare checkout would otherwise only prove the refusal |
| `rust/scripts/ci_workflow_local_gate.sh` | `audit_generated_clippy_correctness_surface` |

⭐ **Why the commit workflow does NOT get a second clippy pass.** Correctness lints are
deny-by-default, so the generated stage `clippy_on_rust_change` already runs IS the finding
check once the flag defaults to `1`. The one thing that stage cannot see is a lint being
**demoted out of deny-by-default** — and roster integrity catches that with no cargo at all.
So the commit workflow gets the full guarantee for **zero** added build cost, and the
explicit-deny run stays available as its own gate. (This is the zero-cost acceptance test
from the governing invariant applied to a gate rather than to a parser primitive.)

⛔ **The charter's "so it runs in `ci_workflow_local_gate`" is NOT achievable as written, and
this is measured, not assumed.** That gate exports `git ls-files` only, so its export dir has
no `generated/` — the gate would REFUSE there by design. It is therefore wired as a **surface
audit** (contract pinned, both anti-vacuity checks present, strict-by-default not silently
reverted, the `Makefile` lanes present), with the real runs in the commit workflow, the
standalone target, and the hosted workflow.

#### ⚠️⚠️ FOUND WHILE WIRING — `ci_workflow_local_gate` HAS BEEN UNABLE TO COMPLETE FOR 1,371 COMMITS

Registering the new audit meant running the gate, and it **died on its FIRST audit**:
`audit_static_include_paths` asserts `assert_tracked "generated/ebnf.rs"`. Nothing under
`generated/` is tracked — `git ls-files generated/ | wc -l` = **0** — and commit `0ed2b2ad`
*"Slice 5: stop tracking generated/\* in git"* (2026-04-29) is an ancestor of `HEAD` with
**1,371 commits** since. Seven `assert_tracked "generated/…"` call sites assert a condition
repository policy guarantees can never hold.

**Repaired here, because otherwise this leaf's own wiring would be unverifiable dead code** —
the exact disease the tree treats. New helper `assert_generated_artifact` checks *presence on
disk* (the condition `build.rs` actually tests) and refuses with the regeneration command.
Verified: `audit_static_include_paths` now passes, and
`audit_generated_clippy_correctness_surface` was executed in isolation → **PASS**.

⛔ **The gate still does not complete, and the rest is NOT this leaf's to fix.** Measured
after the repair: **8 of its 31 audit functions fail**, each for an independent, unrelated
reason —

| failing audit | reason |
|---|---|
| `audit_top_level_docs_surface` | allowlist missing 4 live files (`docs/TASK_TREE.md`, `docs/TASK_TREE_README.md`, `docs/POST_SV_AUDIT_LEDGER.md`, `docs/SV_EXH_PROOF_BASELINE.md`) |
| `audit_contract_docs_surface` / `audit_reference_docs_surface` | allowlist drift |
| `audit_active_docs_rehome_paths` | live docs still cite pre-rehome paths |
| `audit_embedding_api_surface` | pins `EMBEDDING_API_VERSION = "1.2.0"`; the source has moved on |
| `audit_ebnf_frontend_conversion_surface` | forbids `ebnf_to_json.pl` in `rust/Makefile`; it is there |
| `audit_rtl_frontend_generated_contract_surface` | expects `expected_rule_texts`, superseded by the `0.2.0` typed-AST migration |
| `audit_sv_formal_exhaustive_closure_surface` | pins contract prose that has since changed |

Each needs its own adjudication (is the AUDIT stale, or is the REPO wrong?) — routed to the
new tree `CI-PARITY-GATE-ROT`. ⭐ **Third instance in this tree of a maintained check that
nothing runs**, and the most consequential: `README.md` names
`make -C rust ci_workflow_local_gate` as *the* way to prove workflow parity **while hosted
Actions are paused**.

#### The two enforcer weaknesses routed from `.2`

Both fixed, plus a third the fix's own probe found. Full rationale + limits:
`docs/decisions/project_codegen_emission_root_cause_signature.md`.

1. **Fourth `DIAGNOSIS_SIG` group — CODEGEN-EMISSION.**
   `GENERATED-CLIPPY-CORRECTNESS:|clippy::[a-z_]{3,}|PGEN_CLIPPY_GENERATED_STRICT`. A
   generator emitting the wrong code has no parse to trace, no run to sample and no compiler
   error, so groups 1–3 could not back it. Tokens are verbatim tool output on the
   `error[EXXXX]` footing; bare `generated/…_parser.rs` and `make focus_<g>` were
   **deliberately excluded** as too loose.
2. **BOX-SCOPED evidence.** The signature must now sit inside the ticked box's own bullet.
   This closes BOTH the `.2`-routed cross-FILE leak *and* — ⭐ **found by searching prior art
   first, per `DESIGN-PRIOR-ART`** — the incidental-PROSE leak already recorded as a deferred
   watch item in `docs/decisions/project_build_integrity_compiler_root_cause_signature.md`
   (*"scope the grep to the ticked box's own bullet"*). One mechanism, two holes.
3. ⚠️ **The probes caught a real bug in the first implementation.** It matched the box
   KEYWORD against the whole body, so a box merely *mentioning* "root cause" could stand in
   for the real one — probe RED-2 went green when it should have gone red. The keyword now
   matches the **header line only** (reusing the original proven regex); the signature
   matches the body. **The probes earned their keep by failing the first attempt.**

⚠️ **A FIFTH signature family is visible and deliberately NOT added.** Replaying the new rule
over every tracked leaf: **56 carry a ticked ROOT CAUSE box, 26 are box-scoped-backed, 30 are
not.** Sampling them, many cite a `file.rs:NNN` plus the culprit expression or a controlled
3-arm differential — genuine WHY+WHERE that no group models (e.g. `QUANT-PLUS-ITER.md:723`,
`` `ast_based_generator.rs:616-621` … three-arm control isolates the start symbol ``).
Widening to "any file:line" would weaken the gate to "cite a line number", so the question is
routed to `.4` rather than answered by loosening a regex. The 30 are already-committed leaves
and the rule binds only NEW commits — but the number is the honest price, recorded up front.

#### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — `.1` + `.2` left the generated correctness count at **0**, with
      nothing holding it: `PGEN_CLIPPY_GENERATED_STRICT` defaulted to `0` and a repo-wide
      sweep found **zero** non-prose setters (no gate, no aggregate, no CI workflow, no make
      target).
- [x] **ROOT CAUSE (WHY + WHERE)** — three distinct WHYs, each tool-pinned.
      (a) *Never run:* `rust/scripts/clippy_on_rust_change.sh:15`,
      `GENERATED_STRICT="${PGEN_CLIPPY_GENERATED_STRICT:-0}"`, plus
      `grep -rn PGEN_CLIPPY_GENERATED_STRICT --include=*.sh --include=*.yml --include=Makefile`
      returning only that one line.
      (b) *Would pass vacuously:* `rust/build.rs:90-168` gates every
      `has_generated_<n>_parser` cfg on `is_file()`, and `git ls-files generated/ | wc -l` is
      **0**, so the artifacts are absent wherever the tree is not locally regenerated.
      (c) *Would narrow silently:* the subset was implicit in clippy's deny-by-default
      levels; censused mechanically from `clippy-driver -Whelp`, `clippy::correctness` holds
      **68** lints including the two anchors `clippy::eq_op` and
      `clippy::overly_complex_bool_expr` that produced the 291.
      The instrument that reports this defect family is the lint lane itself — the
      `GENERATED-CLIPPY-CORRECTNESS:` gate output and `clippy::<lint>` paths quoted above are
      its verbatim signature.
- [x] **FIX** — build the gate + contract + `Makefile` lanes + hosted workflow + local surface
      audit; default `PGEN_CLIPPY_GENERATED_STRICT` to `1`; add the `--policy-only` stage to
      the commit workflow; repair the 7 false `assert_tracked "generated/…"` assertions. No
      lint suppression, no `#[allow]`, no Rust source or grammar touched.
- [x] **ADDRESSED (verified)** — the gate runs and passes for real:
      `artifact-presence 10/10 required + 1/1 optional`,
      `roster-integrity 68 pinned, all still in clippy::correctness`,
      `cfg-census 8/8 cfg-guarded required artifacts confirmed compiled in by cargo`,
      `findings: total=0 (expected 0)`, `findings: in generated/=0 (expected 0)`,
      **exit 0**, peak process-tree RSS **11,908 MB**, elapsed **130 s** under
      `scripts/run_with_memory_guard.sh --budget-mb 14336`. Non-vacuity of that GREEN
      independently confirmed: `pgen` lib `fresh: false` in the run's own
      `compiler-artifact` stream.
- [x] **NO REGRESSION** — `scripts/check_doctrines.sh` **9/9 PASS**;
      gate probes `run_generated_clippy_gate_probes.sh` **6/6** (RED-A missing artifact,
      RED-B roster departure, RED-C contract self-inconsistency, RED-D a real
      `clippy::eq_op` finding, RED-E the vacuity refusal, CTRL clean → pass);
      enforcer probes `run_diag_evidence_probes.sh` **6/6** (RED-1 cross-file, RED-2
      out-of-box prose, RED-3 unticked, GREEN-1 correctness family, GREEN-2 codegen family,
      CTRL-1 no code staged);
      `audit_generated_clippy_correctness_surface` executed in isolation → PASS and
      `audit_static_include_paths` repaired from fail → pass;
      **no `grammars/*.ebnf`, no `rust/src/*`, no `generated/*` and no
      `ast_shape_contract/*.json` in the diff**, so all 11 generated parsers are
      **byte-identical by construction** and no parse verdict can have moved — the reason
      the oracle battery is not re-run here.
- [x] **LOCKSTEP** — `docs/decisions/project_codegen_emission_root_cause_signature.md` (new)
      + `docs/decisions/INDEX.md`, `docs/tasks/CI-PARITY-GATE-ROT.md` (new),
      `docs/TASK_TREE.md`, `README.md`, `COMMIT.md`, `PGEN_USER_GUIDE.md`,
      `docs/book/src/quality-and-closure-model.md`, `CHANGES.md`,
      `DEVELOPMENT_NOTES.md`, `MEMORY.md`, `LIVE_ACHIEVEMENT_STATUS.md`. No release /
      schema / ledger / integration-contract movement.

### `.4` — the fifth diagnosis family: is a controlled differential a valid ROOT-CAUSE signature? (`done`)

- **Status: `done`** (2026-07-27, session #216, `PGEN-GENERATED-LINT-CORRECTNESS-0006`).
- ⛔⛔ **DIRECTOR-SCHEDULED FOR A FRESH SESSION (2026-07-27, session #215, verbatim: *"Do this
  … at the next fresh session"*).** The director delegated the DECISION itself — *"You know the
  objective of the project, the north star too, so decide"* — and then scheduled the work for a
  clean context. **Order to execute in: this leaf first, then `CI-PARITY-GATE-ROT.1`, then
  `CI-PARITY-GATE-ROT.2`.** Nothing here was started; no partial state existed to reconcile.

#### ⛔⛔ THE DECISION — **NO fifth family of the chartered shape. The charter's premise is REFUTED.**

The mandated first step was *"OPEN AND READ THAT 144 BEFORE DESIGNING"*. All 144 `RGX-0078.md`
unbacked boxes were read, plus all 110 boxes matching no descriptive probe. Then the hypothesis was
**priced against the corpus instead of being adopted**:

| candidate | admits |
|---|---|
| **SITE ∧ CONTRAST — the charter's own hypothesis** | **2 of 304 (0.7%)** |
| CONTRAST alone | 15 of 304 (4%) |
| SITE alone (⛔ forbidden — "cite a line number") | 134 of 304 (44%) |
| corrected macOS/native profiler vocabulary | 11 of 304 (3.6%) |

⇒ the conjunction is **one sample generalised into a rule**. `.3` chose `QUANT-PLUS-ITER.md:723`
and called it *"the cleanest example"* — designing from the cleanest sample rather than the corpus,
the exact failure [[feedback_read_prior_art_before_designing]] names. ⭐ **This leaf's own charter
warned about that failure mode and would have committed it had the hypothesis been implemented as
written.** `.3`'s companion hypothesis about `RGX-0078` is refuted too: only **6 of its 144** boxes
mention `/usr/bin/sample` at all.

#### ⭐⭐ THE REAL SHAPE: 94% is PLACEMENT, and no regex can fix a placement gap

| | count |
|---|---|
| **OUT-OF-BOX** — the leaf HAS tool evidence, just not inside the ticked bullet | **288 of 304 (94%)** |
| **NO-EVIDENCE** — the whole leaf file carries no diagnosis signature at all | 16 of 304 (5%) |

`.3` correctly moved the bar from *"the leaf shows tool output"* to *"the ticked bullet quotes tool
output"*; 94% of a corpus written under the OLD rule satisfies the former only. That is why every
candidate priced at 0.7–3.6% — **they were all answering the wrong question.**

⇒ **the 288 are LEFT AS-IS and deliberately NOT back-filled** (the leaf spec asked for a plain
answer): the rule binds new commits only, and rewriting 49 historical records to satisfy a rule that
post-dates them is back-dating the record — refused on the `LEX-ADJACENCY.1` precedent, where the
retro-fit was **marked as such so the failure stays visible**.

#### ✅ WHAT DID SHIP — two narrow, non-weakening corrections

**(1) Group 2's tool vocabulary was factually WRONG for this repository.** It named
`cargo flamegraph` / `self-time` (generic Rust) and backed exactly **2** boxes repo-wide, while the
SPEED tree — 153 ROOT CAUSE boxes — measures with macOS `/usr/bin/sample`, `otool -tV` annotated
disassembly, `spindump`/`filtercalltree`, a process-local `ITIMER_PROF` sampler and PGEN's own
`--dump-rule-outcome-counts-json`. Same token class as `cargo flamegraph` ⇒ a **correction of a bar
aimed at the wrong tools, not a relaxation**. Left alone, a SPEED leaf using this repo's real
profiler is forced to waive — a gate teaching authors to bypass it.

**(2) A genuine FIFTH family exists, and it is OPS / BUILD-FLOW — not "controlled differential".**
The 16 no-evidence boxes are dominated by defects in the repo's own operational surface: a Makefile
recipe swallowing a nonzero exit (`RGX-0090`), a version gate passing vacuously (`RGX-0091`), an
`execve` list overflowing `ARG_MAX` (`SV-REPLAY-DEBT`), awk auto-vivification in the memory guard
(`OPS-MEMSAFE`), tracking-state hygiene (`REPO-HYGIENE`). No parse to trace, no run to sample, no
rustc error (it is shell/make), no codegen emission — **the identical argument that admitted groups
3 and 4.**

⭐⭐ **THE FAMILY WAS REQUESTED BY THE CORPUS ITSELF AND NOBODY READ IT.**
`docs/tasks/RGX-0090.md:131` carries a hand-written waiver note *inside the ticked box* (now routed
and CLOSED — see `GENERATED-LINT-CORRECTNESS.6`), verbatim:
*"like RGX-0091 this is a BUILD-FLOW defect — the parse/perf diagnosis-toolbox signatures do not
apply"*; `RGX-0091.md:119` wrote *"Diagnosis tool signatures: `grep -n …`"* and got no credit. An
author telling the gate it does not model their defect class is the strongest possible evidence of
a missing family — and it sat unread. ⛔ Bare `grep` is **deliberately excluded**: measured, it
matches **16** boxes on prose like *"verified by grep"* — a claim, not tool output, and the
"cite a line number" degradation in another costume.

**Calibration (why a 7-box family is not too small).** Measured at `2eed59b6` in a pristine
worktree, so this leaf's own checklist box cannot inflate its own justification: group 1 backs
**48**, group 2 backed **2**, group 3 **2**, group 4 **1**. Group 3 was admitted on the strength of
ONE leaf and group 4 on ONE. The corrected group 2 goes **2 → 14**; the ops family backs **7** —
the third-largest family in the gate, more than groups 3 and 4 combined.

#### Acceptance checklist (enforced)

- [x] **ROOT CAUSE (WHY + WHERE)** — WHY the 302/304 boxes fail is NOT a missing signature family:
      measured with `run_root_cause_box_census.sh` (which sources `DIAGNOSIS_SIG`/`ROOT_KW` live
      from the enforcer so it cannot measure a different rule than the gate applies),
      **288 of 304 (94%) sit in a leaf that DOES carry a signature, outside the ticked bullet** —
      a placement consequence of `.3`'s box-scoping, unclosable by any token set (charter
      hypothesis priced at 2/304). WHERE the two real defects are: `scripts/check_diagnosis_evidence.sh`
      `DIAGNOSIS_SIG` — group 2 names `cargo flamegraph`/`self-time` while the SPEED corpus uses
      `/usr/bin/sample`/`otool -tV` (group 2 backed 2 boxes repo-wide), and no group models
      ops/build-flow defects, which `docs/tasks/RGX-0090.md:131` says in a hand-written in-box
      waiver note (that class is now mechanized — `GENERATED-LINT-CORRECTNESS.6`). Ops evidence is
      itself tool-backed: `git ls-files generated/` → 0,
      `git rev-list --count` over 2,618 commits, `make -n` on the swallowing recipe.
- [x] **ADDRESSED (verified)** — before → after on the symptom, produced not asserted:
      `run_diag_evidence_family5_probes.sh` accepts `PGEN_DIAG_CHECK_OVERRIDE`, so the identical
      arms replay against the pre-`.4` enforcer. **BEFORE (`2eed59b6`): all 5 GREEN arms BLOCKED
      (8 passed / 5 failed). AFTER: 13/13.** Corpus effect: backed boxes **52 → 68**, no-evidence
      residual **16 → 9**.
- [x] **NO REGRESSION** — the 4 RED and 4 CONTROL arms are **identical before and after** (that is
      the non-weakening proof): a bare `file.rs:NNN` citation still BLOCKS, *"verified by grep"*
      still BLOCKS, an out-of-box ops signature still BLOCKS (box-scoping binds the new family),
      unticked still BLOCKS. `.3`'s own driver re-run **6/6**. All 9 enforced doctrines PASS;
      `mdbook_docs_gate` GREEN. No `grammars/*.ebnf`, no `rust/src/*`, no `generated/*` touched ⇒
      all 11 parsers byte-identical BY CONSTRUCTION; no release/schema/ledger/contract movement.

#### ⚠️ Honest limits (stated, not discovered later)

1. The gate still **cannot check that the signature matches the defect class** — a parser leaf can
   satisfy it with `git ls-files` exactly as it already can with `error[E0308]`. Pre-existing
   documented limit, widened by one family, not a new hole.
2. **The census over-counts what the gate polices.** The checklist is only demanded when a
   `code_changed` path is staged, so "304 unbacked boxes" is *not* 304 commits that would have been
   blocked.
3. **Box-scoping has almost no operational history** — it landed at `5c5a0ca0`, two commits before
   this one.

#### ⛔ ROUTED, NOT FIXED → new leaf `.5`

Measured across all **2,618** commits while pricing the ops family: **521** touched the ops surface
(`scripts/`, `rust/scripts/`, Makefiles, `.githooks/`, `.github/workflows/`, `rust/build.rs`) and
**397 of them (76%) staged no `code_changed` path**, so the acceptance checklist was never required.
That includes `5c5a0ca0` itself — a new gate, a tracked contract, Makefile lanes, a CI workflow and
a change to the doctrine enforcer, with no checklist demanded. Not fixed here: widening
`code_changed` has real blast radius and is its own leaf, per the `DOCTRINE-GAP-OWNERSHIP` rule
(own it, do not fix it in the wrong leaf).

⭐⭐ **AND THIS COMMIT IS ITSELF ONE OF THE 397 — verified, not inferred.** Running the live gate
over this leaf's own staged set returns, verbatim:
`diag-evidence: OK (no code change staged; task-acceptance checklist not required)`. A slice that
**edits the acceptance enforcer itself** is not policed by it. The checklist above was therefore
written and earned voluntarily; it is backed (the census counts it — group 5 goes 7 → 8 with this
leaf's own box included), but nothing would have blocked its absence. There is no sharper statement
of why `.5` exists.

#### 🗄️ THE CHARTER AS WRITTEN — SUPERSEDED BY THE DECISION ABOVE, kept verbatim

⛔ Preserved rather than deleted, on this project's own discipline that a superseded conclusion
stays visible instead of being back-dated. Everything below is what `.3` proposed and what `.4`
was chartered to implement; the measurement above refuted it. Two specific claims here are now
known false: the `302` box count is `304` at this commit, and the *"either … or"* framing of the
`RGX-0078` 144 is neither branch. ⭐ Its final bullet — *"the leaf should say plainly whether they
are back-filled or left as-is"* — IS answered: **left as-is.**

#### ⭐ THE CENSUS, REFINED — read this before deciding (measured at `5c5a0ca0`)

The `.3` record's *"30 of 56 leaf files"* is the **file-level** number under the enforcer's
actual rule (a file passes if **any** one of its ticked ROOT CAUSE boxes carries a signature).
Censusing **individual boxes** instead gives a sharper and more alarming picture:

| measure | count |
|---|---|
| unbacked ROOT CAUSE **boxes** | **302** |
| distinct files holding ≥1 unbacked box | **49** |
| files where NO box is backed (what the enforcer sees today) | **30 of 56** |

⚠️ **The distribution is the finding, not the total.** `docs/tasks/RGX-0078.md` alone holds
**144** of the 302 — and that is the SPEED campaign, whose leaves are supposed to be
*profiler*-backed by signature group 2 (`self-time` / `call-graph attribution` /
`flamegraph`). A tree that large failing its own designated group is evidence that the gap is
**not** merely "one unmodelled family": either the SPEED leaves diagnose by differential rather
than by profiler, or group 2's tokens do not match how profiler work is actually written up
here. ⛔ **Do not design the fifth family until that 144 is opened and read** — it is over
double the sample everything else would be inferred from, and inferring from the other 158
would repeat this session's own banked failure (designing from a summary instead of the source,
[[feedback_read_prior_art_before_designing]]).
Other concentrations: `REGEX-PCRE2-FIDELITY.md` 25, `SV-CORPUS-GRAD.md` 19,
`VERILOG-2005-PROFILE.md` 11, `PARSE-HARNESS.md` 9, `LANG-CAPABILITY-AUDIT.md` 9.

#### 💡 WORKING HYPOTHESIS (a starting point, explicitly NOT a decision)

The shape that keeps recurring is a **controlled differential**: a precisely named site plus an
N-arm experiment holding everything else fixed, whose arms produce *different outcomes*. That is
a genuine causal isolation — arguably stronger than a trace line, which shows correlation.

⇒ candidate definition, and the reason it may be admissible without weakening anything: make
the fifth group a **CONJUNCTION**, unlike groups 1–4 which are single-token alternations —

1. a **SITE**: a real repo path with a line number (`[\w./-]+\.(rs|ebnf|sh|json):[0-9]+`), **AND**
2. a **CONTRAST**: a quoted outcome difference between arms (e.g. `ACCEPT→REJECT`, `N → 0`,
   `diverge=`, a before→after verdict pair).

Requiring **both** is a *stricter* bar than any existing group, so it admits real differentials
while never degrading to "cite a line number" — the failure mode `.3` refused. ⛔ Still
forbidden: a bare `file\.rs:[0-9]+` alternation.

**Whoever takes this must RED/GREEN it like `.3` did**: fabricated trust-me prose containing a
plausible file:line must FAIL, and a sample of the real 302 must PASS. `.3`'s own probes caught
a real bug in its first implementation — do not trust the regex without the arms.
- Box-scoping measured that **30 of 56** leaf files carrying a ticked ROOT CAUSE box do not
  have a signature inside that box. Sampling shows a recurring, genuinely-diagnostic shape
  that no group models: a **source citation plus a controlled experiment** — `file.rs:NNN`,
  the culprit expression quoted, and an N-arm differential that isolates one variable
  (`QUANT-PLUS-ITER.md:723` is the cleanest example).
- The question to adjudicate: is that a FIFTH signature family deserving its own token group
  (as correctness, performance, build-integrity and codegen-emission each got), or is it
  prose that should be required to cite a tool? ⛔ Do **not** resolve it by adding a loose
  `file\.rs:[0-9]+` token — that degrades the gate to "cite a line number", which is exactly
  the weakening every prior extension refused.
- Whatever is decided, the 30 files are already committed and the rule binds only new
  commits; the leaf should say plainly whether they are back-filled or left as-is.

#### Evidence

- `docs/tasks/artifacts/generated_lint_correctness/run_root_cause_box_census.sh` — the census
  instrument (`--census` / `--classify` / `--placement` / `--dump*` / `--sig` / `--alt`). Sources
  `DIAGNOSIS_SIG` + `ROOT_KW` live from the enforcer, so the census and the gate can never diverge.
- `docs/tasks/artifacts/generated_lint_correctness/run_diag_evidence_family5_probes.sh` — 13 arms
  (4 RED / 5 GREEN / 4 CONTROL) with the `PGEN_DIAG_CHECK_OVERRIDE` before→after replay.
- `docs/tasks/artifacts/generated_lint_correctness/family5_decision_capture.txt` — the full
  measurement log behind every number above.
- `docs/decisions/project_ops_build_flow_root_cause_signature.md` — the doctrine record.
- ⚠️ Fixed in passing, because `.4` would otherwise have made it lie: `.3`'s
  `run_diag_evidence_probes.sh` carried a **hand-copied duplicate** of `DIAGNOSIS_SIG`/`HDR_RE`;
  correcting group 2 would have left that sweep measuring the stale rule — the
  duplicated-metadata class inside the very driver that verifies the enforcer. Now sourced.

### `.5` — the acceptance gate is blind to the ops surface: 397 of 521 commits never faced it (`done`)

- **Status: `done`** (2026-07-27, session #216, `PGEN-GENERATED-LINT-CORRECTNESS-0007`).
  ⛔⛔ **ROUTED IN `.4`, THEN FIXED HERE ON A DIRECT DIRECTOR ORDER** — verbatim: *"Why did let that
  slide. You are the guarantor of the integrity of the repository. You are the guarantor that all
  rules, doctrines of the project are strictly followed to the T."* The rebuke is recorded because
  it is correct: `.4` MEASURED this hole and then deferred it to a leaf instead of closing it, which
  is the same *"record it and move on"* failure `DOCTRINE-GAP-OWNERSHIP` exists to stop — committed
  by the very session that opened that tree.

#### The fix

`code_changed` in `scripts/check_diagnosis_evidence.sh` now treats **the proof surface as part of
the code**: `scripts/check_*.sh`, `rust/scripts/*.sh`, `Makefile` / `rust/Makefile`, `.githooks/*`,
`.github/workflows/*.yml`, `rust/build.rs`. **A change to a gate is a change to what "verified"
MEANS.**

⭐ **Ordering was load-bearing: `.4` had to land first.** Widening the predicate without the
ops/build-flow diagnosis family would have bound ~397 commits' worth of change classes to a gate
whose signatures could not express their evidence — manufacturing waivers wholesale. With `.4` in
place such a defect is root-caused with `git ls-files` / `make -n` / `shellcheck` / an errno, so the
widening is satisfiable rather than punitive.

⚠️ **Scoped deliberately narrower than "anything under `scripts/`"** — only the machinery deciding
whether other checks run. Corpora, fixtures and ordinary tools stay unbound, pinned by CTRL-3/4/5.

#### Acceptance checklist (enforced)

- [x] **ROOT CAUSE (WHY + WHERE)** — WHY: `code_changed` enumerated only parser artifacts, so a
      change to the machinery that decides whether the parser is ever checked was invisible to the
      acceptance gate. WHERE: `scripts/check_diagnosis_evidence.sh` `code_changed` case list.
      Tool-backed with `git diff-tree`/`git log` over all 2,618 commits: **521 touched the proof
      surface, 397 (76%) staged no path from the old list** ⇒ the gate answered *"no code change
      staged"*. Every rot class of sessions #214–#216 lives on that surface and no other
      (`ast_dump_contract_gate`, `PGEN_CLIPPY_GENERATED_STRICT`, `ci_workflow_local_gate`).
- [x] **ADDRESSED (verified)** — `run_proof_surface_scope_probes.sh` **12/12**, replayed against the
      pre-`.5` enforcer via `PGEN_DIAG_CHECK_OVERRIDE`: **BEFORE all 5 RED arms passed straight
      through as "no code change staged"; AFTER all 5 BLOCK.** Re-measured blast radius: commits
      bound **1,104 → 1,501**, i.e. exactly the **397** measured hole, no more.
- [x] **NO REGRESSION** — CTRL-1/2 (rust/src, unchanged behaviour) and CTRL-3/4/5 (docs, corpus,
      ordinary tool — the over-binding boundary) are **identical before and after**. 10/10 doctrines
      PASS; clippy N/A (no Rust changed). No `grammars/*.ebnf`, no `rust/src/*`, no `generated/*` ⇒
      all 11 parsers byte-identical by construction.

⚠️ **The CONTROL arms caught a real bug in the probe harness, not the predicate** — `new_repo`
staged its own copy of the enforcer, which now correctly matches `scripts/check_*.sh`, so every arm
looked like a proof-surface change and CTRL-3/4/5 failed for a reason unrelated to the rule under
test. Fixed by committing the enforcer in the probe repo's base commit. **Same lesson as `.3`:
write the control arms, and believe them when they fail.**

⚠️⚠️ **KNOCK-ON, found only by re-running EVERY driver after the commit — and it is the same bug in
two more places.** `.3`'s and `.4`'s probe harnesses build their throwaway repos the same way, so
widening the predicate silently broke **their** "no code change staged" CONTROL arms too
(`run_diag_evidence_probes` 5/1, `run_diag_evidence_family5_probes` 12/1). Neither is a defect in
the rule — all three harnesses were staging a copy of the enforcer that `.5` now correctly counts as
a proof-surface change. Both fixed with the same base-commit; all four enforcer drivers re-verified
green (**6/6, 13/13, 12/12, 8/8**).
⭐ **The lesson is about verification scope, not about the bug:** a change to a shared enforcer
invalidates every harness that *exercises* it, not only the one written alongside it. Re-running the
single driver for the current leaf would have reported a clean 12/12 and shipped two broken
controls. **When the thing you changed is used by other checks, re-run all of them.**

### `.6` — a waiver is a bug report about the gate: mechanize it (`done`)

- **Status: `done`** (2026-07-27, session #216, `PGEN-GENERATED-LINT-CORRECTNESS-0007`), on the
  director's question: *"What should do here to correct that and prevent that it does not happen
  ever again."*
- **The class.** `RGX-0090.md:131` carried a hand-written waiver *inside a ticked ROOT CAUSE box* —
  *"the parse/perf diagnosis-toolbox signatures do not apply"* — and it sat unread for months;
  `RGX-0091.md:147` did the same and even wrote *"Diagnosis tool signatures: `grep -n` …"*. Both
  were RIGHT. `.4` re-derived the identical gap from scratch.
- ⭐ **THE LAW: an author writing a waiver IS the gate reporting a missing capability** — the
  highest-signal defect report a gate can receive, because it comes from someone who did the work,
  hit the boundary, and wrote down where it was. Treating it as corner-cutting inverts the
  diagnosis: the author complied; the instrument was incomplete.
- **SHIPPED: the 10th enforced doctrine `WAIVER-ROUTING`** (`scripts/check_waiver_routing.sh`,
  registered in `scripts/check_doctrines.sh`, mirrored in `DOCTRINE_ENFORCEMENT.md` §10). A staged
  leaf that ADDS such a claim must name an owning leaf/slice id nearby, or the commit is blocked
  with the line quoted.
- ⛔ **It must not punish honesty** (constraint inherited from `DOCTRINE-GAP-OWNERSHIP.2`):
  forbidding the language would delete the signal — authors would stop writing the note, which is
  strictly worse than an unread one. The waiver stays **legal**; it just names an owner.
- ⚠️ **Two boundaries learned by USING it, both of which would have made it wrong:**
  (1) ⭐ **scope ≠ capability** — the first draft fired on a dozen honest *"this slice is pure-docs,
  so the code-change gate does not apply"* statements, which are NOT bug reports; narrowed to
  claims about the **signature surface**; (2) ⭐ **markdown wraps** — same-line discharge was
  unsatisfiable for any wrapped paragraph (it failed on the two real waivers being routed) and
  would have pushed authors toward deleting the waiver to pass ⇒ ±6-line window, kept small.
- **Corpus swept and the founding cases ROUTED IN PLACE**: 6 waiver-shaped lines, **4 routed**
  (was 1), 2 residual meta-prose. `RGX-0090`/`RGX-0091` annotated as **CLOSED by `.4`**, with the
  retro-fit explicitly marked (`LEX-ADJACENCY.1` precedent — the failure stays visible).
- Verified: probes **8/8** (3 RED incl. the **verbatim RGX-0090 text** / 2 GREEN / 3 CONTROL incl.
  an untouched historical waiver proving the record is never retro-bound); the sweep **sources the
  enforcer's own regexes** so it cannot measure a different rule than the gate applies; 10/10
  doctrines PASS.
- Decision record: `docs/decisions/project_waiver_is_a_gate_bug_report.md`.


## Evidence

- `rust/target/clippy_gate/logs/clippy_generated_all_targets.log` — the source
  measurement (untracked build output; regenerate with
  `make -C rust SHELL=/opt/homebrew/bin/bash clippy_on_rust_change`, which needs
  `scripts/run_with_memory_guard.sh --budget-mb 16384` per
  `docs/decisions/feedback_host_ram_budget_all_jobs.md` — it peaks ≈12.5 GB and the
  12288 default kills it).
- Error census, measured 2026-07-26 at commit `779dca06`:
  `158` `eq_op` in `systemverilog_parser.rs`, `132` `eq_op` in `rtl_frontend_parser.rs`,
  `1` `overly_complex_bool_expr` in `systemverilog_preprocessor_parser.rs` = **291**.
  Zero errors in any `rust/src/**` file (the strict source stage is green).
- Provenance note: the count is **pre-existing**, not introduced by `.10.4` — that leaf
  left all 11 generated parsers byte-identical, so the same bytes carried the same 291
  errors before it.
- `.1` (session #214) evidence, tracked:
  - `docs/tasks/artifacts/generated_lint_correctness/run_degenerate_emission_sweep.sh` —
    the driver (3 modes: `--capture-before` / `--capture-after` / `--report`); it censuses
    every statically-decided form the three emitters can produce, across all 11 artifacts,
    at their canonical output paths.
  - `docs/tasks/artifacts/generated_lint_correctness/degenerate_emission_sweep_capture.txt`
    — the before → after capture (**21,201 → 0**; 241.1 MB → 219.8 MB).
  - `docs/tasks/artifacts/generated_lint_correctness/verification_capture.txt` — the full
    ADDRESSED + NO-REGRESSION verification log, including the mechanical proof that the
    `ast_dump_contract_gate` failure found during verification predates this leaf.

### `.7` — the sixth family is REFUSED at 0%, and the REAL misbehaviour is that box-scoping was VACUOUS (`done`)

- **Status: `done`** (2026-08-01, `PGEN-GENERATED-LINT-CORRECTNESS-0009`, CODE — the enforcer).
  Opened by `SV-EXH-PROOF.7.4.6.9`'s `ROUTED_FINDING_2026-08-01`, whose ROOT-CAUSE box could not
  be backed by any of the five existing families.

#### ⛔⛔ THE TWO ANSWERS

**(1) NO sixth family. Priced at 0 % of the corpus** — below `.4`'s own refused 2/304.

| candidate sixth family | newly admits |
|---|---|
| `failure_reasons` / `top_failure_reasons` / `reachable_branch_debt` / `branch_groups` | **0 of 307 (0 %)** |
| + `--dump-gen-ast`, `--gap-report-*`, `--report-k-path-coverage`, `depth_exceeded`, … | 1 of 307 (0 %) |
| widest plausible generator/coverage vocabulary | 3 of 307 (0 %) |

The corpus shape `.4` found is unchanged and sharper: **412** ticked ROOT CAUSE boxes, 307
unbacked, of which **299 (97 %) are OUT-OF-BOX** and only **8 (2 %) NO-EVIDENCE**. A placement
gap, and no token set closes a placement gap. ⇒ **the refusal IS the result**, exactly as `.4`'s
was, and the director's *"if you find a misbehavior"* condition is what made refusing available.

**(2) ⭐⭐ A REAL misbehaviour, and it makes `.3`'s hardening vacuous.** `.3` scoped the SIGNATURE
to its own box but never scoped the BOX to the change: `box_matches` returned true when **any**
ticked box in **any** staged task file satisfied a requirement. So a tree already holding one
compliant leaf supplies the checklist for every later leaf, forever, free.

| probe | pre-`.7` | post-`.7` |
|---|---|---|
| **H1** new leaf, NO checklist, same file as one old completed leaf | **PASSED** | BLOCKED |
| **H2** owning leaf in file B, backed boxes only in unrelated co-staged A | **PASSED** | BLOCKED |
| **H3** the three boxes satisfied only across two files | **PASSED** | BLOCKED |

**33 tracked task files carry ≥1 backed ROOT CAUSE box** = a standing free pass. ⭐ And in the
sharpest available statement of it — the `.4` precedent, verified not inferred — **this leaf's own
commit would have satisfied box 1 on four historical boxes (`.2`/`.3`/`.4`/`.5`) without writing a
single line of checklist.**

#### THE RULE THAT SHIPPED, AND WHY IT IS NOT PUNITIVE

A box may satisfy a requirement only if it sits in a **leaf section** (headings of level ≤ 3, so a
`#### Acceptance Checklist` block belongs to its `###` leaf) that the staged change **touches**,
and all three must be met within **one** file. Deliberately permissive *inside* a leaf: a
follow-up commit editing any part of the same leaf keeps its checklist (GREEN-2), and a
deletion-only edit still counts (GREEN-3) — otherwise ordinary multi-commit leaves would be forced
to waive, which is how a gate teaches authors to bypass it (`.4`/`.6`).

⚠️ **Trailing blank lines are trimmed off a section, and that is load-bearing.** A new leaf is
APPENDED, and an append begins with the blank separator line that syntactically still belongs to
the PREVIOUS section — so without the trim, writing a brand-new leaf with no checklist "touches"
the finished leaf above it and inherits its boxes. The first implementation had exactly that hole
and **RED-H1 caught the rule being vacuous against the most common edit in the repository.**

#### THE STRUCTURAL CANDIDATES — each RED-probed or DISPROVEN, none assumed

| candidate (from the charter) | verdict |
|---|---|
| `unchecked()` not box-scoped ⇒ an unrelated leaf's unticked box blocks a legitimate commit | **DISPROVEN, population 0** — no tracked task file carries an unticked ROOT-CAUSE box. Fails *closed*, so left alone deliberately; an unticked box must block wherever it sits. |
| `box_body` exits on `^#` ⇒ a fenced block hides a real signature | **DISPROVEN, 0 boxes** repo-wide (measured against a fence-aware body rule). |
| token width (`\botool\b`, `flamegraph`, `clippy::[a-z_]{3,}`) | `ROOT_KW`'s bare `\bwhy\b` over-matches **4** headers — all `**FIX**`/`**LOCKSTEP**`/`**REPRODUCE**` boxes — but **none carries a signature**, so none can satisfy the gate today. A latent fails-open widening; **routed, not fixed.** |
| *(found here, not chartered)* the satisfying box need not belong to the change | **REPRODUCED — H1/H2/H3. This is the misbehaviour.** |

#### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — `run_diag_evidence_leaf_scope_probes.sh` against the pre-`.7`
      enforcer: **6 passed, 3 failed** — RED-H1/H2/H3 each print
      `diag-evidence: OK (task leaf passes the acceptance checklist…)` and `exit=0` on a leaf that
      wrote no checklist at all.
- [x] **ROOT CAUSE (WHY + WHERE)** — ops/build-flow signature (family 5).
      **WHERE:** `scripts/check_diagnosis_evidence.sh`, `box_matches()` — it loops
      `for f in "${staged_tasks[@]}"` and `return 0`s on the first file with a qualifying box, so
      the box is never tied to the change. **WHY it went unseen:** `.3` closed signature→box
      co-location and its record claims cross-FILE leakage was closed with it; a whole borrowed
      **box** still crossed files freely. Measured with `git log`-driven replay —
      `git rev-list -400 HEAD` + `git diff-tree --no-commit-id -r -p -U0` per commit —
      **138 code-change commits pass today and 7 pass ONLY by borrowing**; and
      `git ls-files 'docs/tasks/*.md'` shows **33 files** carrying a standing free pass.
- [x] **FIX** — declarative/ops tier: one script. `added_ranges` (parses `-U0` hunk headers, and
      maps a pure-deletion hunk to its insertion point) + `section_touched` (headings ≤ 3, with
      the trailing-blank trim) + `box_matches` → `box_matches_in_file` with **separable**
      present / signature-backed / change-owned stages so a breach is reported at the stage it
      failed. No grammar, no `rust/src/`, no `generated/*`.
- [x] **ADDRESSED (verified)** — before → after on the same 9 arms, produced not asserted via
      `PGEN_DIAG_CHECK_OVERRIDE`: **BEFORE 6/9 (RED-H1/H2/H3 all pass the gate) → AFTER 9/9.**
      Blast radius re-measured on the shipped rule: of the 138 passing code-change commits in the
      last 400, **7 newly fail — and all 7 were read individually and are genuine instances of the
      defect** (5 whose own NO REGRESSION box carries no gate signature, 2 that wrote no
      qualifying box at all) ⇒ **0 false positives**.
- [x] **NO REGRESSION** — GREEN-1/2/3 and CTRL-1/2/3 are **byte-identical before and after** (the
      non-weakening proof); `.3`'s driver re-runs **6/6** and `.4`'s **13/13**; all **17** enforced
      doctrines PASS; `bash -n` clean on both changed scripts. No `grammars/*.ebnf`, no
      `rust/src/*`, no `generated/*` touched ⇒ all 11 parsers **byte-identical BY CONSTRUCTION**,
      and no release / schema / ledger / contract movement. Census re-runs with its new controls
      green at **413/106/307** — the pre-change reading was 412/105/307, and the `+1 box / +1
      backed / unbacked unchanged` delta is exactly this leaf's own ROOT CAUSE box entering the
      corpus, which is the arithmetic the controls exist to make checkable rather than assumed.
- [x] **LOCKSTEP** — `TOOLBOX.md` (the box-scoping paragraph now states the leaf-scoping rule),
      `DOCTRINE_ENFORCEMENT.md` §6.1 leg 2 + the `TASK-ACCEPTANCE` row, the book chapter
      *Quality and Closure Model*, `docs/decisions/project_acceptance_box_must_be_written_by_the_change.md`
      + `INDEX.md` (143/143), `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`.

#### ⭐⭐ THE FIX IS PROVEN BINDING ON THIS COMMIT — not just on synthetic probes

`.4` established the discipline of testing a doctrine change against **its own commit** rather than
only against fixtures. Run here on the real staged set, with `.7`'s own checklist temporarily
removed while `.1`–`.6` kept their ticked, backed boxes:

```
- ROOT CAUSE box is ticked and backed, but it belongs to a LEAF THIS CHANGE DID NOT TOUCH —
  an already-finished leaf cannot supply the checklist for new work.
- ADDRESSED box is ticked and backed, but it belongs to a LEAF THIS CHANGE DID NOT TOUCH …
- NO REGRESSION box is ticked and backed, but it belongs to a LEAF THIS CHANGE DID NOT TOUCH …
   exit=1
```

Under the pre-`.7` enforcer that identical state returns
`diag-evidence: OK (task leaf passes the acceptance checklist…)`. ⇒ the commit that closes the
borrow hole is itself held by the rule it adds, and would have been exempt from it an hour earlier.

#### ⭐ The census instrument had NO ground truth — and it published `.4`'s headline numbers

`run_root_cause_box_census.sh` sourced `DIAGNOSIS_SIG` live from the enforcer (so it could not
measure a *different rule*) but had nothing pinning its own **box arithmetic**. `.4` published
304/288/16 from it; `.7` measures 412/299/8 — a corpus that genuinely grew, but **nothing in the
tool could have distinguished growth from a silent regression.** It now runs four pinned controls
on every invocation — POS-1 signature inside the box, NEG-1 signature only elsewhere in the file,
NEG-2 keyword in the body but not the header (`.3`'s RED-2, a bug that was once real), NEG-3
signature past a `#` heading — and **REFUSES with exit 2** on a miss rather than printing a
plausible number ([[feedback_instrument_needs_ground_truth]]).

⭐ The controls earned their keep twice before any number here was trusted: they **caught a bug in
the fixture itself** (NEG-2's header accidentally carried the keyword), and both fire on mutation
— deleting the `^#` body terminator gives `3 2`, replacing `box_body` with a whole-file read gives
`3 3`, against the pinned `3 1`.

#### Evidence

- `docs/tasks/artifacts/generated_lint_correctness/run_diag_evidence_leaf_scope_probes.sh` —
  9 arms (3 RED / 3 GREEN / 3 CONTROL) with the `PGEN_DIAG_CHECK_OVERRIDE` before→after replay.
- `docs/tasks/artifacts/generated_lint_correctness/run_root_cause_box_census.sh` — the census,
  now with `ground_truth_controls()` run before any mode emits a number.
- `docs/decisions/project_acceptance_box_must_be_written_by_the_change.md` — the doctrine record.

#### ⛔ ROUTED, NOT FIXED → new leaf `.8`

`NOREGRESS_SIG` carries the **identical wrong-vocabulary defect group 2 had** and `.4` corrected:
it names only parser-side global gates (`seeds 0/7/42`, `byte-identical`, `external corpus`,
`shape-contract`, `spf=0`, `fully_certified`, `clippy`), so an ops/build-flow change has no
natural no-regression token. **120 of 416 (29 %)** NO REGRESSION boxes are unbacked, and 5 of the
7 commits this leaf newly blocks are exactly that shape. ⭐ **That gap is *why* the borrow hole
went unnoticed — the hole was silently absorbing it.** Not fixed here: it needs `.4`'s pricing
discipline against the whole corpus before any token is adopted, and per `DOCTRINE-GAP-OWNERSHIP`
it is its own leaf. An honest idiom exists meanwhile (*"no grammar/rust/generated touched ⇒ all 11
parsers byte-identical BY CONSTRUCTION"*, which `.4` itself used), so closing the borrow does
**not** force a waiver.

#### 🗄️ THE CHARTER AS WRITTEN — SUPERSEDED BY THE TWO ANSWERS ABOVE, kept verbatim

⛔ Preserved rather than deleted, on this project's own discipline that a superseded conclusion
stays visible instead of being back-dated (the `.4` precedent). Everything below is what this leaf
was chartered to do. ⭐ Its own instruction — *"a REFUSAL is an equally valid outcome of this
leaf"* — is what the measurement returned for the family question; and its structural-candidate
list is what led to the misbehaviour that WAS found, though to none of the three it named.


#### ⛔ THE DIRECTOR'S APPROVAL, VERBATIM AND CONDITIONAL (2026-08-01)

> *"Fix the enforcer carefully because of \[the routed finding] **if you find a misbehavior in the
> enforcer**. You have my approval, but as always be careful, sota, signoff decision and be highly
> professional."*

⭐ **THE APPROVAL IS TO FIX A MISBEHAVIOUR — IT IS NOT A MANDATE TO ADD A FAMILY.** "If you find"
is the whole instruction. Establishing whether a misbehaviour exists is this leaf's work; a family
added without one is a gate widened to let content through, which is the same move as raising a cap
to land content ([[feedback_done_bar_is_first_tier_only]]'s sibling principle, and the reason
`MEMORY.md`/`README.md` carry two caps each).

#### THE OBSERVATION THAT OPENED IT

`scripts/check_diagnosis_evidence.sh` accepts five ROOT-CAUSE families (`DIAGNOSIS_SIG`). Every
correctness token in it names a PARSER-side instrument (cert-coverage, `[plannable-probe]`,
predicate-rejection trace, `furthest_position=`, `--lint-grammar`, the rule-counter dumps). The
instrument that diagnosed all 79 targets of the SV class-A residual — the per-branch
`failure_reasons` record, which `TOOLBOX.md` §6.1 calls *"the FIRST thing to read on any 'why is
this coverage target still residual' question"* — matches **none** of them, and neither does the
normalized gen-AST (`--dump-gen-ast`) the static depth instruments consume.

#### ⛔⛔ PRIOR ART THAT MUST BE READ BEFORE ANY DESIGN — `.4` REFUTED ITS OWN CHARTERED FAMILY

`.4` was chartered to add a fifth family and **decided against it**, by PRICING the candidate
against the corpus instead of adopting it: the chartered shape admitted **2 of 304 (0.7%)** boxes,
and the real gap turned out to be **94% PLACEMENT** (the leaf had tool evidence, just not inside the
ticked bullet) — *"no regex can fix a placement gap"*. It also caught itself designing from *"the
cleanest example"* rather than the corpus, the exact failure
[[feedback_read_prior_art_before_designing]] names. ⇒ **The same pricing is mandatory here, and a
REFUSAL is an equally valid outcome of this leaf.**

#### THE PRICING ALREADY DONE (initial, and it points AWAY from a new family)

`grep -rlE "failure_reasons|top_failure_reasons" docs/tasks/*.md` returns **one** file
(`SV-EXH-PROOF.md`) against **403** ticked ROOT-CAUSE boxes repo-wide. On `.4`'s own standard that
is one data point, not corpus pressure — which is precisely why `SV-EXH-PROOF.7.4.6.9` did **not**
touch the enforcer and cited only evidence it genuinely produced.

#### WHAT THIS LEAF MUST DO (measure first — TOOLBOX-FIRST, anti-spin)

1. **Build the census instrument, with ground truth.** Reproduce the enforcer's OWN logic
   (`box_body` + `box_matches`, box-scoped) over every ticked ROOT-CAUSE box in `docs/tasks/*.md`
   and report, per box, whether its own body matches `DIAGNOSIS_SIG`. ⛔ Pin a positive and a
   negative control inside it and REFUSE on a miss ([[feedback_instrument_needs_ground_truth]]);
   `.4`'s numbers exist to cross-check against.
2. **Classify the misses the way `.4` did** — `OUT-OF-BOX` (evidence exists elsewhere in the leaf)
   vs `NO-EVIDENCE` vs *genuinely inexpressible* (the defect class has no token in any family).
   Only the third category can justify a sixth family, and only if it is a POPULATION.
3. **Hunt for STRUCTURAL misbehaviour independently of the family question** — this is where a
   real "misbehavior" is most likely to be, and it is what the director's condition points at.
   Candidates to test, each RED/GREEN, none assumed:
   - the `unchecked()` path is **not** box-scoped the way `checked()` is: it scans every staged
     task file, so an unticked ROOT-CAUSE box in an UNRELATED leaf of the same tree may block a
     legitimate commit. `.3` closed cross-file leakage in the POSITIVE direction only — the
     negative direction was never re-examined.
   - `box_body` exits on any line starting with `#`, so a fenced code block containing a shell
     comment or a Markdown heading truncates the body early and can hide a real signature.
   - token width: `\botool\b` and `flamegraph` can match ordinary prose; `clippy::[a-z_]{3,}`
     matches a lint named in passing.
4. **Then decide**, and record the decision either way with its numbers. A refusal is a result.

#### Acceptance

`a census instrument with BOTH controls firing; the candidate PRICED against the whole corpus (not
a sample) with the OUT-OF-BOX / NO-EVIDENCE / inexpressible split published; every structural
candidate in (3) either REPRODUCED with a RED probe or DISPROVEN on evidence; if a change lands, the
enforcer's own RED/GREEN pair proven to fire in BOTH directions before it is trusted
([[feedback_instrument_needs_ground_truth]]) and no previously-passing leaf silently starts failing;
if no change lands, the refusal recorded with the numbers that justify it.`

⛔ **Scope guard:** this leaf may change `scripts/check_diagnosis_evidence.sh` — the proof surface,
so it is a CODE change and needs its own acceptance checklist, satisfiable via the **ops/build-flow**
family (`bash -n`, `make -n`, `git log -S`, `shellcheck`). It may **not** re-tick or re-word any
existing leaf's boxes to make them pass.


### `.8` — `NOREGRESS_SIG` names only parser-side gates: 120 of 416 boxes unbacked (`pending`)

- **Status: `pending`** (routed in from `.7`, 2026-08-01, nothing started — no partial state).
- **The finding, measured in `.7`:** 416 ticked NO REGRESSION boxes repo-wide, **296 backed / 120
  unbacked (29 %)**. The unbacked population is dominated by ops/build-flow and gate/docs slices
  whose honest no-regression evidence is *"probes N/N"*, *"both failure directions falsified"*,
  *"`bash -n` clean"*, *"the pass path is untouched"* — none of which `NOREGRESS_SIG` names. This
  is the **identical shape** as group 2's wrong-vocabulary defect that `.4` corrected for
  ROOT CAUSE, now on the NO-REGRESSION axis.
- ⛔ **Do NOT adopt a token before pricing it against the whole corpus.** `.4` refused its own
  chartered family at 2/304 by doing exactly that; `.7` refused a sixth at 0/307. A shallow probe
  already shows the tempting tokens are weak or wrong: `untouched` matches 36 — but that is
  *prose*, and admitting it is the *"verified by grep"* degradation `.4` explicitly refused.
- **Why it is not urgent:** an honest idiom exists today — *"no grammar/rust/generated touched ⇒
  all 11 parsers byte-identical BY CONSTRUCTION"* — which `.4` itself used, so no author is forced
  to waive. The gap costs expressiveness, not compliance.
- **Acceptance:** the candidate priced against all 416 boxes with the OUT-OF-BOX / NO-EVIDENCE
  split published; RED arms proving no prose-only form is admitted; the census instrument extended
  to the NO-REGRESSION axis **with its own ground-truth controls**; a refusal recorded with its
  numbers if that is the answer.


### `.9` — `ROOT_KW`'s bare `\bwhy\b` is a fails-open widening: narrow it (`pending` — DECIDED, priced, not yet implemented)

- **Status: `pending`** (opened 2026-08-01 by `.7`, which found it and deliberately did NOT fix it
  in-leaf to keep that leaf's blast radius controlled). **The director delegated the call**
  (*"Please decide yourself … provided you strictly stick to sota, signoff and highly
  professional"*), and the decision taken is **FIX IT** — the pricing below is why.

#### THE DEFECT

`ROOT_KW='root cause|why ?\+ ?where|\bwhy\b'`. The third alternative matches the bare WORD, so a
box that is not a ROOT CAUSE box at all can satisfy the ROOT CAUSE requirement. Measured: **4**
ticked headers match via bare `\bwhy\b` only, and every one is a `**FIX**` / `**LOCKSTEP**` /
`**REPRODUCE**` box (e.g. `- [x] **FIX** — … Why no lower tier: …`).

⛔ **This is FAILS-OPEN, and on the director's own named step.** A `**FIX**` box routinely quotes a
command, so a leaf carrying no ROOT CAUSE box at all can pass box 1 on its FIX box. That is
categorically different from the two candidates `.7` DISPROVED (`unchecked` cross-leaf blocking and
`box_body` `^#` truncation), which both fail *closed*. `.7`'s leaf-scoping narrows the exposure — the
box must now be one the change wrote — but does not remove it.

#### THE PRICING — a NARROWING, so the bar is "does it break anything", measured at zero

Candidate `ROOT_KW='root cause|why ?[-+/&] ?where'` (drops the bare word; still accepts the
template spelling and the obvious `/`, `-`, `&` connectors), priced against every ticked ROOT CAUSE
box in `docs/tasks/`:

| measure | result |
|---|---|
| boxes dropped by the candidate | **4** |
| of which **BACKED** (i.e. would break a currently-passing leaf) | **0** |
| files losing their last backed ROOT CAUSE box | **0** |
| unticked headers (the false-BLOCK surface) — current → candidate | **0 → 0** |

⇒ zero cost in both directions. 408 of the 412 ticked headers already match `root cause`, and 400
match `why + where`, because `TOOLBOX.md`'s template spells it `**ROOT CAUSE (WHY + WHERE)**`.

⚠️ **Note the asymmetry in the standard, deliberately.** `.4` and `.7` refused *widenings* (2/304 and
0/307) because a widening needs corpus PRESSURE to justify. This is a *narrowing* that closes a
soundness hole, so the question is not "is there a population demanding it" but "does it break
anything" — and the answer is measured, not argued.

#### WHAT IS LEFT TO DO (nothing measured is missing; this is implementation only)

1. Change `ROOT_KW` in `scripts/check_diagnosis_evidence.sh` to the candidate above.
2. RED/GREEN probes proving BOTH directions before trusting it (`.3`'s standing rule): a `**FIX**`
   box whose header says *"why"* and whose body quotes `--trace-rules`, with no ROOT CAUSE box,
   must **BLOCK** (it PASSES today); a real `**ROOT CAUSE (WHY + WHERE)**` box must still ALLOW.
3. Re-run `.3` (6), `.4` (13) and `.7` (9) drivers; census controls; all 17 doctrines.
4. Expect the census to read **408** ticked ROOT CAUSE boxes, not 412 — that delta IS the fix, and
   the census ground-truth controls must stay green across it.
5. ⛔ Mind `run_root_cause_box_census.sh`: it sources `ROOT_KW` live from the enforcer, so it
   follows automatically — **do not hand-copy the regex** (`.4` fixed exactly that duplication bug
   in `.3`'s probe driver).

#### Acceptance

`the candidate priced against the whole corpus with the 0-backed-boxes-dropped result reproduced;
a RED arm proving a FIX-box-with-"why" no longer satisfies box 1, and a GREEN arm proving the
template spelling still does; no previously-passing leaf newly failing; all 17 doctrines PASS.`

## Commit log

| slice | leaf | commit subject |
|---|---|---|
| `PGEN-GENERATED-LINT-CORRECTNESS-0001` | (tree opened) | the 291 generated-clippy errors adjudicated — not defects, but they make a real check unrunnable |
| `PGEN-GENERATED-LINT-CORRECTNESS-0002` | `.1` | the degenerate branch-policy / layout-skip emissions are folded at codegen — 291 clippy errors → 0, 22.3 MB off the shipped parsers |
| `PGEN-GENERATED-LINT-CORRECTNESS-0003` | `.2` | the sweep closes the class — associativity / negative-case / terminal-layout folded too (9,679 → 0, another 9.0 MB), and `@associativity` gets its first oracle |
| `PGEN-GENERATED-LINT-CORRECTNESS-0007` | `.5` + `.6` | the acceptance gate now sees the proof surface (397 blind commits closed), and a waiver is mechanized as the bug report about the gate that it is |
| `PGEN-GENERATED-LINT-CORRECTNESS-0006` | `.4` | the chartered fifth family is REFUTED at 2 of 304 — the gap is 94% placement, group 2 named the wrong profilers, and the real fifth family (ops/build-flow) was requested by the corpus itself in an unread waiver note |
| `PGEN-GENERATED-LINT-CORRECTNESS-0008` | `.7` (opened) | the sixth diagnosis family is CHARTERED, not granted — the director's approval is conditional and the pricing already points AWAY |
| `PGEN-GENERATED-LINT-CORRECTNESS-0009` | `.7` + `.8` | the sixth family is REFUSED at 0 of 307 — and the real misbehaviour is that box-scoping was VACUOUS: any ticked box in any staged task file satisfied the checklist, so 33 trees carried a standing free pass |
| `PGEN-GENERATED-LINT-CORRECTNESS-0010` | `.9` (opened) | the bare `\bwhy\b` over-match is a FAILS-OPEN widening on the named step — decided (director-delegated) and priced at 0 backed boxes dropped, implementation pending |
