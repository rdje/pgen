# GENERATED-LINT-CORRECTNESS: the generated-parser clippy stage can never be made strict, so a real correctness lint there would be invisible forever

## Metadata

- Tree ID: `GENERATED-LINT-CORRECTNESS`
- Status: `active` (opened 2026-07-27, session #213, by **direct director order** —
  *"take the right decision, you have information to take the proper signoff, sota
  decision"*, on being shown the 291-error generated-clippy stage)
- Family / slice-id prefix: `PGEN-GENERATED-LINT-CORRECTNESS-<NNNN>`
- Created: `2026-07-27`
- Owner: repo-local workflow
- **Frontier: `.2`** (`.1` `done` 2026-07-27, session #214 —
  `PGEN-GENERATED-LINT-CORRECTNESS-0002`)
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

### `.2` — sweep for other statically-degenerate emissions (`todo`)

- **Status: `todo`** — frontier. `.1` fixed the two CONSTANTS whose emissions the clippy
  run happened to surface. Other configuration constants may emit the same shape without
  ever having been caught, because most degenerate forms (`"longest_match" == "ordered"`)
  are **not lint-visible at all** — `.1` measured 20,910 such emissions that no lint
  would ever have reported.
- ⚠️ Do the sweep **at the codegen sites**, not by reading generated output: the pattern
  is "a `#interpolated` constant used in a runtime conditional", and it is enumerable in
  the generator source. Reading 150 MB of emitted Rust is the wrong instrument.
- ⛔ **Sweep all THREE emitters** (`ast_based_generator.rs`,
  `ast_based_generator/cascade.rs`, `ast_based_generator/scan.rs`) — `.1` found the
  charter had named only one, and a `rust/src/ast_pipeline/*.rs` glob does not descend
  into the subdirectory. Any grep that does not name all three is unsound by
  construction.
- Known inputs already measured by `.1`, **deliberately left for this leaf**:
  - `#associativity_mode` — emitted as `match "left" { "right" => …, "nonassoc" => …,
    _ => false }`, statically decided in all three emitters. Not lint-flagged, so it is
    pure dead weight. ⚠️ Folding it makes the emitted `nonassoc_tie` binding's `mut`
    conditional (`ast_based_generator.rs:4854` today), or it trades the fold for an
    `unused_mut` warning.
  - `#allow_layout_skip_for_terminals` — emitted as `if true {` / `if false {` at
    `ast_based_generator.rs:6914/:6929/:8417/:8431`. Same class, no lint fires.

### `.3` — promote the generated-clippy correctness subset to a gate (`todo`)

- **Status: `todo`**, blocked on `.2` (`.1` is `done` — the count IS 0 today, but nothing
  yet stops it drifting back up). Once the count is 0, make it **stay** 0:
  run the generated stage strictly for the **correctness category only** (not all of
  clippy — the 84k style warnings on generated code are genuinely not worth chasing and
  gating on them would be noise, not signal).
- Wire it where the other maintained gates live so it runs in `ci_workflow_local_gate`,
  and record the chosen lint subset in the contract so it cannot silently narrow.

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
