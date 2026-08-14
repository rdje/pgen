# `ENGINE-UNIVERSAL-SERVICES.17` slice 8 — the guard PGEN EMITS, executed

Owned by `docs/tasks/ENGINE-UNIVERSAL-SERVICES.md` leaf `.17`, slice 8.

Slice 7 closed with the sentence that defines this bank's burden, and it was a correction to its own
first draft:

> What this slice emits has **never been executed**: `guard_effectiveness` measured a HAND-WRITTEN
> `g7`, and slice 7 asserts PGEN's emission matches it in the gen-AST. Nothing has generated a parser
> from PGEN's own guarded output and run an input through it.

This bank runs it.

## Run it

```bash
bash docs/tasks/artifacts/engine_universal_services/guard_parses/probe.sh
```

`64/64 as declared`, rc 0, ~15 minutes — most of it four `parseability_probe` rebuilds. There is no
`--interp-only` shortcut; see *Both oracles* below for why that is a soundness point here and not a
cost decision.

## What is in it

**Two grammars, two arms each.** Both grammars are **pre-rewrite**: still left recursive, no helper
rules, no lookaheads, every line one a grammar author would write. That is the difference from the
sibling `guard_effectiveness/` bank, whose `g0`–`g7` are hand-eliminated post-rewrite shapes.

| arm | how the parser is generated | what PGEN does with the grammar |
|---|---|---|
| **A** | the same generator line **plus** `--indirect-lr-admit-starvation-safe-only` | `starvation-safe candidates: 0/3` ⇒ **nothing absorbed**, the cycle survives |
| **B** | `make -C rust focus_scratch` — the shipped path | `prim` absorbed **and** its surviving starvation site guarded |

Both arms of a grammar compile the **same** `generated/scratch.json`, produced by arm A. The
admission is the only difference.

⛔⛔ **THE ARMS SWAPPED WHICH ONE CARRIES THE FLAG AT `.17` SLICE 9, AND NOT ONE EXPECTATION MOVED.**
This bank was written while the narrow admission shipped: arm A was the bare `make` path and arm B
opted in with `--indirect-lr-admit-guard-feasible`. Slice 9 flipped the shipped criterion, so the
lever inverted and the arms traded places. **Every parse verdict and every structural count is
byte-identical to what slice 8 recorded** — which is the strongest statement this file can make about
the flip: what used to require a flag is what `make` now produces, and the rows that proved it did
not have to be renegotiated.

⭐ The **banner** rows are the only two that inverted, and necessarily: the warning follows the
non-shipped policy, and the non-shipped policy changed sides. Arm A now prints it; arm B does not.

## Why there are TWO grammars, and it is the bank's most important line

`s1_guard_source.ebnf` has two holders:

```text
scratch := kw_k eq outer_cast semi     # the residual-BEARING holder — needs the guard
         | kw_k eq prim semi           # the residual-FREE holder — must stay unguarded
```

⛔⛔ **That second alternative can parse `e1`–`e6` on its own**, through the
eliminated-but-**unguarded** `prim`. So on S1 those six rows pass whether the guard is emitted or
not. Measured rather than reasoned: deleting the trailing-lookahead emission from `apply_plan`,
rebuilding, and re-running the whole bank left it **GREEN**. *A plant that cannot fail is a row that
cannot check.*

⭐ `.17` slice 4 had already drawn this line one ladder down — `guard_effectiveness/probe.sh` runs
`g4` and `g5` on `e1` and `e7` **only**, *"because their second alternative absorbs `e2`–`e6` on its
own, so those rows would pass under BOTH hypotheses and prove nothing"*. S1 inherited `g7`'s shape
and not that lesson. (Neither did `g7` itself — see that bank's README, corrected by this slice.)

`s2_holder_only.ebnf` is the fix: **S1 minus the second alternative**, nothing else changed. Now
`e1`–`e6` have exactly one route to a parse and each tests a guard position:

| row | what it can falsify |
|---|---|
| `2B e1` | the **LOOP** guard — the `*` must stop at zero iterations or the holder starves |
| `2B e2` | …and must still take exactly one iteration when the input has a real chain |
| `2B e5` | the **TRAILING** guard — the over-long SEED must be refused so `kw` wins the tournament |
| `1B e7` | the **CALL-SITE SCOPING** — `k = n;` must still reach `prim` unguarded |

⇒ **neither grammar alone measures the design.** S1 proves the guard does no damage where it must do
none; S2 proves it does the work where it must. `probe.sh` refuses to pass if the S2 arm did not run.

## The four results

**1. ⭐⭐ The emitted guard PARSES.** Every input ACCEPTs on arm B of both grammars, on real generated
parsers. That includes `2B e5` (the over-long SEED, which the loop guard provably cannot reach) and
`1B e7` (`k = n;`, which the shared-rule shape `g4` REJECTS). One design doing both is its whole
claim, and until this bank it had only ever been shown on grammars a person wrote.

**2. ⭐⭐ Six GEN rows FLIP.** `e2` / `e4` / `e6` on **both** grammars go **REJECT → ACCEPT** between
the arms. That is `.13`'s founding defect (*"the runtime guard REJECTS the derivation"*) closed by
PGEN's own emission. `probe.sh` fails if the flip count reaches zero: a bank where both arms accept
everything is describing a grammar, not measuring an admission.

**3. ⭐⭐ The HOP CLONE is exercised end-to-end**, which is `.17` acceptance (d)'s standing
obligation. Both holders name `ct`, not `prim`, so the transparent chain is two rules long
(`chain: ct > prim`, `max_hops=1`) and PGEN must emit `prim_lr_guard0_ct`. Rows `1B/2B hop_clone`
assert that function is **in the emitted parser**. SystemVerilog's own dry run never reaches this
rule — all three of its chains have `chain:` of length 1 — so before this bank the hop-clone half had
unit-test coverage on synthetics only.

**4. ⛔⛔ The un-eliminated arms diverge, and they diverge in OPPOSITE directions.** `.14`'s title is
*"the INTERPRETER and the generated parser disagree … in BOTH directions"*, and this bank reproduces
both in one run, on two grammars that differ by one line:

| | `1A` (two holders) | `2A` (one holder) |
|---|---|---|
| `e2` `k = n'(n)'(n);` | GEN **REJECT** · INTERP ACCEPT | both REJECT |
| `e1` `k = n'(n);` | both ACCEPT | GEN **ACCEPT** · INTERP **REJECT** |

The mechanism is one difference: the interpreter has no cycle guard, only a whole-stack depth ceiling
(`parse_harness_interpreter.rs:746-749`), and under the `longest_match` default it must *evaluate*
the cyclic alternative in order to compare it. On S1 the surviving second entry alternative rescues
the parse after that evaluation blows the ceiling; S2 has nothing to rescue it. ⇒ **which direction
`.14` bites in is a property of the surrounding grammar, not of the cycle** — and this is the
smallest reproducer of either direction in the repository: six rows, two ~20-line grammars, no
SystemVerilog.

⛔ **The `2A` INTERP cells were the one place this bank's expectations were corrected**, and it is
recorded rather than quietly edited: they were first written `ACCEPT`×6 by extrapolation from `1A`
and measured `REJECT`×6. The extrapolation was the error. They are a **pin of a known-divergent
oracle**, not a design claim — every design claim lives in the GEN column, and all 26 GEN cells were
predicted before the first run and measured correct.

## Both oracles, always

Every parse row is measured on the real generated parser (`GEN`, via the scratch slot — authoritative
BY CONSTRUCTION) **and** on `--interpret-parse` (`INTERP` — authoritative BY VERIFICATION only).

⛔ Unlike the sibling banks, a row here declares **two** verdicts, one per oracle. It has to: on the
A arms they legitimately differ (result 4). A bank that failed on any disagreement could not measure
the un-eliminated arms at all, and a bank that declared one verdict for both would have to choose an
oracle to be wrong about. So the divergence is declared, and the run derives and prints its count.

⛔ **This is also why there is no `--interp-only` mode.** The interpreter reports ACCEPT on *every*
row of *every* arm. A run that skipped `GEN` would conclude the shipped admission already parses
everything — it would not under-report, it would MISREPORT, which is `.13` slice 3's recorded lesson
in this exact rule family.

## The instrument states its own input

Both `INTERP` columns read ACCEPT on every row, so *nothing in the verdicts distinguishes an arm's
interpreter run from its sibling's*. A bank that silently dropped the flag from a B arm would look
identical. The `banner` rows close that: they assert the opt-in warning is **absent** on the shipped
runs and **present** on the widened ones, so the flag is proven to have reached the pass rather than
assumed to have.

## Structural rows, and why they are read off the artifact

`guard_rule_count`, `eliminated` and `hop_clone` are counted by rule-function name in
`generated/scratch_parser.rs` — the compiled artifact, never the plan (`.17` slice 7 RESULT 4:
*a report about a thing must be computed from that thing*).

⛔ The first draft of those anchors was `^\s*fn parse_…`, and the codegen emits `pub fn parse_…`. They
matched nothing and reported `guard_rule_count=0` on an arm that has three of them. One run caught
it, because the row **declares 3** — which is the whole reason this bank carries structural
expectations instead of printing what it found. *"Found nothing"* and *"there is nothing"* are the
same output; only a declared non-zero separates them.

## Scratch-slot obligation

Every arm loads a source into `grammars/scratch/scratch.ebnf`. `generated/scratch_parser.rs` is
git-ignored, so restoring the fixture **without regenerating** leaves a clean-looking tree and two RED
gates — `.13` slice 4b's finding, which cost a day. `probe.sh`'s `restore_scratch` trap therefore
checks out **and** re-runs `focus_scratch` on every exit path including Ctrl-C.

⭐ It also rebuilds `parseability_probe`, which the sibling banks do not. That binary **compiles the
scratch parser in**, so a restored artifact next to a stale binary is a third inconsistent state —
one that no `git status` shows. Costs one build; leaves no trap.

## Ground truth, in both directions

⛔ Every total — the ACCEPT/REJECT split, the flip count, the divergence count — is DERIVED and
printed by the run, never stored here (`docs/DERIVED_STATE_CONTAINMENT.md` R1/R3). A prose count of a
table is a claim with no gate behind it, and this family has rotted four of them.

⛔ **Do not adjust an expectation to match a new measurement.** `.17`'s decision rests on these rows;
a bank that edits its own expectations is a bank that cannot notice it broke.

## What this bank does NOT claim

It is the **first leg** of the ratchet, not the flip.

- ⛔ Nothing here changes what ships. The shipped admission is untouched; every one of the 11
  generated parsers is byte-identical across this slice, and `indirect_guard_chains=0` on every
  family.
- ⛔ It is measured on **synthetics**, not on SystemVerilog. `.17` slice 9 flips the admission and
  owes the two-sided repro ratchet plus a corpus re-measure — and, per acceptance (d), must either
  exercise a multi-hop chain on the real grammar or measure and state that its own picks are still
  all `max_hops=0`.
- ⛔ It says nothing about parse-time COST, which is the second non-negotiable and `.17`'s open
  pricing question.
