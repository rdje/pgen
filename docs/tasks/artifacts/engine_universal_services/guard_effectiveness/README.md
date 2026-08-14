# `ENGINE-UNIVERSAL-SERVICES.17` slices 4 + 6 — the guard EFFECTIVENESS bank

Owned by `docs/tasks/ENGINE-UNIVERSAL-SERVICES.md` leaf `.17`, slice 4.

Slice 3 closed with one sentence naming this slice's burden:

> the guard's *effectiveness* — that a trivia-aware structural lookahead refuses exactly the fatal
> iteration on real SystemVerilog text — is **still unmeasured**, and is slice 4's burden.

This bank discharges it, and in doing so refutes an engine law recorded in a decision record and
finds a second starvation mechanism the design did not know about.

## Run it

```bash
bash docs/tasks/artifacts/engine_universal_services/guard_effectiveness/probe.sh
bash docs/tasks/artifacts/engine_universal_services/guard_effectiveness/probe.sh --interp-only
```

`21/21 as declared`, rc 0. `--interp-only` skips the scratch-slot cycle (~6 min per grammar) and can
never be the basis of a claim about the engine on its own — see *Both oracles* below.

## What is in it

Two independent ladders, each a strict one-difference chain.

### Ladder 1 — the guard, on the post-rewrite shape (`g0` / `g1` / `g2`, inputs `e1`–`e6`)

`P5`'s transparent-holder shape (`.13` slice 5b) hand-eliminated at `prim`, **plus SystemVerilog's
own layout model** — a nullable `trivia` rule leading every token, copied in shape from
`grammars/systemverilog.ebnf:618-624`. No earlier synthetic in this family carried layout, which is
why no earlier probe could have found E3.

| | `prim` | models |
|---|---|---|
| `g0_unguarded` | `prim_base ( prim_suffix )*` | the shipped post-rewrite base rule — the control that must starve |
| `g1_byte_guard` | `prim_base ( prim_suffix &residual_first_byte )*` | slice 2's **cheap** form: a byte drawn from `FIRST(residual) = { ' , / }` |
| `g2_structural_guard` | `prim_base ( prim_suffix &( tick lparen lit rparen ) )*` | slice 2's **mandated** form: the residual itself, sub-parsed |
| `g6_trailing_guard_alone` | `prim_base ( prim_suffix )* &( … )` | the TRAILING position alone — `g2`'s exact complement |
| `g3_trailing_guard` | `prim_base ( prim_suffix &( … ) )* &( … )` | **both** positions — the only rung that closes all six inputs |
| `g4_trailing_guard_needs_a_clone` | `g3` **plus a second, residual-free holder** | the guard on the SHARED rule ⇒ `e7` (`k = n;`) breaks |
| `g5_trailing_guard_clone_control` | `g4` with the trailing guard removed | `g4`'s one-difference control — the REJECT is the guard, not the shape |
| `g7_guarded_clone_chain` | `g4`'s holders, the two lookaheads moved onto a **clone chain** | ⭐ `.17` slice 6 — the shape the decision actually specifies |

⭐⭐ **`g4` ↔ `g7` is the pair that closes the ladder**, and until slice 6 it was missing: `g3` had
the power, `g4` showed the damage, and nothing expressed the combination the decision names — the
guard on a clone reached only from the residual-bearing holder. `g7` keeps `ct` and `prim`
untouched, adds `ct_guard := kw | prim_guard` and the guarded `prim_guard`, and repoints only
`outer_cast`'s left corner. It accepts all six starvation inputs **and** `e7`, the row `g4` rejects.
It is also the rule-for-rule TARGET the eliminator's guard planner must synthesize, so that planner
can be checked against a measured grammar rather than against a design note.

The rule-for-rule mapping onto `grammars/systemverilog.ebnf` is in `g0_unguarded.ebnf`'s header.
`cast := casting_type tick lparen expression rparen` is the holder; `casting_type := … |
constant_primary` is the transparent hop; `constant_primary := …_lr_base ( …_lr_suffix )*` is the
possessive loop.

### Ladder 2 — the give-back controls (`c1` / `c2` / `c3`, input `abc`)

Three files that re-adjudicate slice 1's FINDING 1. `c1` is the discriminating shape slice 1 never
ran, `c2` is its one-difference control, `c3` reproduces slice 1's own Q2 so the refutation is not an
argument about a case this bank never measured.

## The three results

**1. The structural guard works; the byte guard does not.** `g2` closes every loop starvation in the
ladder. `g1` closes them too *except* `e3` — `k = n'(n)/*c*/;`, the same input with a comment at the
iteration boundary. `/` is in `FIRST(residual)` because `trivia` is nullable and leads `tick`, so the
byte test passes exactly where it had to refuse. ⇒ slice 2's *"a trivia-aware structural lookahead is
effectively mandatory rather than a refinement"* is now measured rather than argued, on a shape
carrying the real layout model.

⭐ `g1` is written as the **charitable** form deliberately — a lookahead over a regex, so the
engine's layout skipper runs first. The emitted Q-GUARD reads `parser.input.as_bytes()[position]`
raw (`ast_based_generator.rs:6014-6020`) and would also mis-fire on plain whitespace. The finding is
a lower bound on the cheap form's failure.

**2. ⛔⛔ Option (iii) is NECESSARY BUT NOT SUFFICIENT — `e5` is starved by a CHOICE, not by the
loop.** `k = t'(n);` rejects under all three grammars. Bisected: it still rejects with the quantifier
deleted outright (`prim := prim_base`), so no guard on the `*` can reach it. The over-long match
comes from `prim_base`'s sheared-clone alternative winning the `ct` tournament, and the holder then
has no residual left. The shipped analogue is exact — `casting_type := simple_type | constant_primary
| …` (`:1032`) over a `constant_primary` whose post-rewrite base carries the `constant_cast` clone.

⛔ **This paragraph said *"Routed to leaf `.19`"* until `.17` slice 6, and no such leaf exists.**
Slice 4's decision (d) explicitly REFUSED to route the seed starvation out — *"it is not a separate
defect; it is the same follow restriction at the same call site in a second position, fixed by the
same clone"* — and this line was written before that decision and never swept.
⭐ Note also that this paragraph names **`constant_primary`** as the base carrying the clone, and it
is right: `.17` slice 5 measured `casting_type` at `seed_routes=0/10` (its closers have one
alternative each, so the shear leaves no clone) against `constant_primary` at `10/10`. The leaf's own
slice-4 prose transposed the shape onto `casting_type`; this README did not.

**3. ⛔⛔ `[[project_pgen_gives_back_at_the_choice_but_not_at_the_quantifier]]` is REFUTED.** The
choice does not give back either. Slice 1's Q2 — `( "a" | "ab" ) "c"` on `abc` ⇒ ACCEPT — is
non-discriminating: under `longest_match` the longest alternative `"ab"` wins outright and `"c"` then
matches the one remaining byte, so the verdict is predicted identically by *"the choice gives back"*
and by *"the choice commits to the longest"*. `c1` runs the shape that separates them and it
**REJECTS**. Located: the tournament keeps a single winner —
`let mut best_content: Option<ParseContent<'input>> = None;` (`ast_based_generator.rs:5037`),
consumed once at `:5102` — so a successful-but-losing alternative is discarded, not retained, and
there is nothing to retry.

## Both oracles, always

Every row is measured on the real generated parser (`GEN`, via the scratch slot — authoritative BY
CONSTRUCTION) **and** on `--interpret-parse` (`INTERP` — authoritative BY VERIFICATION only). A row
where they disagree prints `DIVERGE(.14)` and fails the bank. This is not caution for its own sake:
`.13` slice 3 ran the interpreter alone on this exact rule family and got a table wrong in **both**
directions. A single-oracle run here does not under-report, it MISREPORTS.

## Ground truth, in both directions

Of the 44 rows, **28 must ACCEPT and 16 must REJECT**; every row declares its verdict in
`probe.sh`'s `CASES` table and the script compares against it. ⛔ This paragraph said *"8 rows must
ACCEPT and 13 must REJECT"* until `.17` slice 6 — a count of an earlier, smaller bank that was never
updated when slice 4 grew it, and wrong for that bank too. Nothing derives from it (the script
compares row by row), which is exactly why it rotted unnoticed. Falsifiability is proven rather than asserted — flipping
`g2_structural_guard e3` from `ACCEPT` to `REJECT` exits **rc 1** naming that row
(`g2_structural_guard e3 … ACCEPT REJECT ⛔` → `GUARD-EFFECTIVENESS: MISMATCH`), and the script was
restored to a byte-identical hash afterwards.

⛔ **Do not adjust an expectation to match a new measurement.** `.17`'s decision rests on these exact
rows; a bank that edits its own expectations is a bank that cannot notice it broke.

## Scratch-slot obligation

The `GEN` arm loads each grammar into `grammars/scratch/scratch.ebnf`. `generated/scratch_parser.rs`
is git-ignored, so restoring the fixture **without regenerating** leaves a clean-looking tree and two
RED gates — `.13` slice 4b's finding, which cost a day. `probe.sh`'s `restore_scratch` trap therefore
checks out **and** re-runs `focus_scratch`, on every exit path including Ctrl-C, and only when the
`GEN` arm actually ran.
