# `ENGINE-UNIVERSAL-SERVICES.17` slice 1 — what PGEN's combinators actually give back

Owning leaf: [`docs/tasks/ENGINE-UNIVERSAL-SERVICES.md`](../../../ENGINE-UNIVERSAL-SERVICES.md) `.17`.
Re-run everything with one command from the repository root:

```bash
bash docs/tasks/artifacts/engine_universal_services/quantifier_policy/probe.sh
# QUANTIFIER-POLICY-CONTROLS: 7/7 as declared — the give-back laws hold as recorded.
```

## Why this bank exists

`.17` opened with a design note asserting that closing SystemVerilog's cast/call knot needs a
**re-enterable `*`**, and that doing so *"reintroduces exactly the backtracking PEG removed to buy
its memoization guarantee."* Both halves are statements about the engine, and
[[feedback_read_prior_art_before_designing]] says an engine claim is **re-measured, never quoted**.
Measuring them changed the design space in two ways the note did not anticipate — so the numbers
live here, re-runnable, rather than in prose.

## The measured laws

| # | grammar | input | verdict | the law it pins |
|---|---|---|---|---|
| Q1 | `( "a" )* "a"` | `aaa` | **REJECT** | the quantifier is **possessive** — it never gives an iteration back |
| Q2 | `( "a" \| "ab" ) "c"` | `abc` | **ACCEPT** | the **choice does give back** a successful-but-losing alternative |
| Q3 | `( "a" \| "ab" )` | `ab` | **ACCEPT** | the default `@branch_policy` is `longest_match`, not first-match commit |
| Q4a | `( "a" &"a" )* "a"` | `aaa` | **ACCEPT** | a per-iteration **stop-guard** closes Q1's starvation with no engine change |
| Q4b | `( "a" &"a" )* "a"` | `a` | **ACCEPT** | …and does not break the zero-iteration case |
| Q5 | `star_rule := ( "a" &"a" )*`, no residual | `aaa` | **REJECT** | that guard is **context-dependent** — rule-global is wrong |
| Q5b | Q5 with the guard removed | `aaa` | **ACCEPT** | Q5's one-difference control: the REJECT is the guard's doing |

Q1 and Q2 are a deliberate **one-difference pair**. Both are "a sub-match succeeds, then the element
after it starves"; the only variable is whether the sub-match came from a quantifier or a choice.
The same engine answers differently. ⇒ **PGEN's blocker is an asymmetry between two of its own
combinators, not a property of PEG.**

## The two findings that move the design

**1. A multi-attempt protocol is not a new cost class for PGEN — it is the default at every
choice.** Q3 shows the engine evaluates *all* alternatives and keeps the longest
(`rust/src/ast_pipeline/ast_based_generator.rs:4273`, comment *"Multi-branch - evaluate all branches
and keep the longest successful match"*; selection is priority → longest → associativity at `:4387`).
The second non-negotiable — *costs are REJECTED, not traded* — has therefore never been read as
*"never attempt twice"*. It has been satisfied by **static elision**: codegen proves the tournament
unnecessary and emits a degenerate dispatch instead (`:4556`). ⇒ the honest question for `.17` is not
*"may the engine attempt more than once?"* but *"can the extra attempts be confined, at codegen time,
to the sites that provably need them?"* — which is the question PGEN has already answered once.

**2. The starvation has a PEG-native fix that costs no backtracking at all.** Q4 closes Q1 with a
lookahead guard; the loop still never re-enters at a lower count, it simply never takes the fatal
iteration. Q5 states its price precisely: the guard is only correct when the holder actually wants a
residual, so it belongs on the **call site**, not on the rule — a sheared clone reached only from the
holder. The `.13` eliminator already emits sheared clones, so the injection point exists.

The emitted quantifier loop already carries a guard slot of exactly this shape: `#quant_guard_tokens`
(`:6006`, the `RGX-0078.5.i.7` Q-GUARD) breaks the loop on a static byte-set test at every iteration
boundary, with furthest-position parity proven. ⛔ **That precedent is architectural, not semantic**:
the Q-GUARD elides an attempt that would have *failed anyway* (sound by construction), whereas a
follow-restriction guard refuses an attempt that would have *succeeded*. It changes the accepted
language, so it carries a soundness burden the Q-GUARD never had. The slot is reusable; the licence
is not.

## Honest bounds

- **Oracle.** Q1–Q5 run through `--interpret-parse`, which is authoritative *by verification*, not by
  construction. Its one measured hole is un-eliminated left recursion (`.14`); no case here contains
  any recursion, so all seven sit on the structural surface the combinator suite pins byte-identical
  to the compile-and-run oracle (35/35 CLEAN — choice under each `branch_policy`, `*`, lookahead
  `&`/`!`, sequence-backtrack). Q1 and Q4 were additionally confirmed on the **real generated
  parser** through the scratch slot — see below.
- **Scale.** These are two-terminal reductions of the knot. They pin the *laws*; they do not
  establish that the cast/call knot's guard is statically computable, which is `.17`'s next open
  question and needs `--report-indirect-lr-plan` on the shipped grammar, not a synthetic.
- **Not measured here.** What a re-enterable `*` would cost on a real corpus. Nobody has built that
  arm, and under *"costs are rejected, not traded"* the burden sits on the arm that adds parse-time
  work — the same posture
  [[project_indirect_left_recursion_is_eliminated_at_generation_not_grown_at_runtime]] took toward
  runtime seed growing.

## Generated-parser confirmation (Q1, Q4)

The two load-bearing cases were re-run through the scratch slot, which is authoritative **by
construction** (the shipped codegen + runtime, the same semantics as any registered parser):

```bash
cp docs/tasks/artifacts/engine_universal_services/quantifier_policy/q1_possessive_star.ebnf \
   grammars/scratch/scratch.ebnf
make -C rust SHELL=/bin/bash focus_scratch
scripts/run_with_memory_guard.sh --budget-mb 16384 --timeout-s 2400 -- \
  bash -c 'cd rust && cargo build --release --features generated_parsers --bin parseability_probe'
printf 'aaa' > rust/target/es17/aaa.txt
./rust/target/release/parseability_probe --parse scratch rust/target/es17/aaa.txt
```

⛔ **Restore BOTH halves of the slot afterwards**, not just the tracked one — `git checkout` cannot
restore the git-ignored `generated/scratch_parser.rs`, and a half-restore left two gates RED for a
day (`.13` slice 4b):

```bash
git checkout grammars/scratch/scratch.ebnf
make -C rust SHELL=/bin/bash focus_scratch      # regenerate the artifact FROM the restored fixture
```

Results are recorded in the leaf's slice-1 section.
