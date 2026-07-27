# GENERATED-LINT-CORRECTNESS: the generated-parser clippy stage can never be made strict, so a real correctness lint there would be invisible forever

## Metadata

- Tree ID: `GENERATED-LINT-CORRECTNESS`
- Status: `active` (opened 2026-07-27, session #213, by **direct director order** —
  *"take the right decision, you have information to take the proper signoff, sota
  decision"*, on being shown the 291-error generated-clippy stage)
- Family / slice-id prefix: `PGEN-GENERATED-LINT-CORRECTNESS-<NNNN>`
- Created: `2026-07-27`
- Owner: repo-local workflow
- **Frontier: `.3`** (`.1` + `.2` `done` 2026-07-27, session #214 —
  `PGEN-GENERATED-LINT-CORRECTNESS-0002` / `-0003`)
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

### `.3` — promote the generated-clippy correctness subset to a gate (`todo`)

- **Status: `todo`** — frontier. `.1` and `.2` are `done`, so the count IS 0 today and
  the whole enumerable class is closed — but nothing yet stops it drifting back up.
  Make it **stay** 0:
  run the generated stage strictly for the **correctness category only** (not all of
  clippy — the 84k style warnings on generated code are genuinely not worth chasing and
  gating on them would be noise, not signal).
- Wire it where the other maintained gates live so it runs in `ci_workflow_local_gate`,
  and record the chosen lint subset in the contract so it cannot silently narrow.
- ⭐ **Also owned by this leaf, routed from `.2`: two measured weaknesses in
  `scripts/check_diagnosis_evidence.sh`.** (1) Its `DIAGNOSIS_SIG` list models correctness,
  performance and build-integrity defects but **not codegen-emission defects**, whose
  WHY+WHERE is an emission-site enumeration plus an artifact census — so a fully
  evidence-backed leaf of that family fails the check. (2) The signature is grepped across
  **all staged task files**, so an unrelated co-staged tree file can satisfy it for a leaf
  that carries no signature of its own (measured: that is how `.1` passed). Fixing (2)
  means scoping the grep to the leaf's own file; fixing (1) means adding a fourth token
  group with its own decision record, as the performance and build-integrity groups each
  got.

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

## Commit log

| slice | leaf | commit subject |
|---|---|---|
| `PGEN-GENERATED-LINT-CORRECTNESS-0001` | (tree opened) | the 291 generated-clippy errors adjudicated — not defects, but they make a real check unrunnable |
| `PGEN-GENERATED-LINT-CORRECTNESS-0002` | `.1` | the degenerate branch-policy / layout-skip emissions are folded at codegen — 291 clippy errors → 0, 22.3 MB off the shipped parsers |
| `PGEN-GENERATED-LINT-CORRECTNESS-0003` | `.2` | the sweep closes the class — associativity / negative-case / terminal-layout folded too (9,679 → 0, another 9.0 MB), and `@associativity` gets its first oracle |
