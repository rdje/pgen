# `SV-CORPUS-GRAD.13c.2b` — the constant size cast: diagnosed, then FIXED

⭐⭐ **THE DEFECT IS FIXED, AND THESE INSTRUMENTS NOW HOLD THE FIX IN PLACE.**
`ENGINE-UNIVERSAL-SERVICES.17` slice 9 (`PGEN-ENGINE-UNIVERSAL-SERVICES-0032`) flipped the indirect
left-recursion eliminator from *"absorb a knot only if it is starvation-safe"* to *"absorb it if it
is safe **or** if a call-site follow-restriction guard makes it safe"*, which absorbs the
`casting_type` knot behind `casting_type_lr_guard1[loop]` at `cast alt#0`. `PGEN-SV-CORPUS-GRAD-0279`
re-baselined this directory against that tree and closed the leaf.

⛔ **RE-BASELINING WAS NOT BOOKKEEPING.** Two of the three scripts asserted the PRE-FIX verdicts, so
on a *correct* tree they exited non-zero — an instrument that is already red can no longer detect
anything, and a red instrument nobody can act on is how a real regression gets waved through. Both
now assert the post-fix truth, and both were proven able to go red again (below).

| script | question | verdict at HEAD |
|---|---|---|
| `constant_primary_lrm_alternative_audit.py` | is an LRM alternative MISSING from the grammar? | **no** — 15/15, 16/16, 5/5, order-identical (unchanged by the fix; it never asserted a verdict) |
| `casting_type_edge_matrix.sh` | is the knot's admission still shipping? | **5 ACCEPT** — re-baselined from 3 ACCEPT / 2 REJECT |
| `corpus_row_cast_bisect.py` | do the 2 attributed corpus rows still parse, and if not WHY? | **both ACCEPT as shipped**, casts intact (22 and 11 of them) |

## 1. The alternative-list audit — the leaf's own worry, answered NO

The leaf opened suspecting a missing alternative: *"a constant-expression path that is missing ONE
`constant_primary` alternative is unlikely to be missing only that one."* It is missing **none**.

```text
=== constant_primary_sv_2017 — shipped 15 alternatives vs LRM-extracted 15   (all ==, in order)
=== constant_primary_sv_2023 — shipped 16 alternatives vs LRM-extracted 16   (all ==, in order)
=== casting_type            — shipped 5  alternatives vs LRM-extracted 5     (all ==, in order)
VERDICT: AGREE — no missing LRM alternative
```

Arm 2 is `grammars/systemverilog_lrm_profiled_generated.ebnf`, machine-extracted from IEEE 1800
Annex A — so this is a fidelity **proof**, not an assertion. `constant_cast` is alternative 12 of
`constant_primary_sv_2017` and has been all along; the derivation is declared and simply
unreachable.

## 2. The edge matrix — one grammar edge, not one construct class

```text
control_size_cast_in_statement.sv            ACCEPT   8'(1) in a STATEMENT expression
control_simple_type_cast_in_constant.sv      ACCEPT   int'(1) — casting_type = simple_type
control_param_name_cast_in_constant.sv       ACCEPT   W'(1)  — casting_type = ps_type_identifier
defect_constant_size_cast.sv                 REJECT   8'(1) in a CONSTANT expression
defect_constant_size_cast_corpus_shape.sv    REJECT   512'({…}) — the OpenTitan corpus shape
```

⭐ Rows 2 and 3 are what make the diagnosis precise, and both were **absent** from the leaf's
original framing. A size cast in a constant expression is fine when its casting type is a keyword
(`int`) or a name (`W`) — those resolve on `casting_type` branch 1/5 (`simple_type`). Only a
**numeric** size leaves `constant_primary` as the sole viable alternative, and that is the one the
indirect left-recursive cycle makes unreachable at the seed position.

## 3. The corpus bisect — from a PRICING question to a DIFFERENTIAL attributor

It opened as pricing. `.13c.2` had *attributed* 2 DARK corpus rows to this construct, and an
attribution is not a measurement — a file can be blocked by several constructs at once, so
*"unblocks 2 rows"* would only have been caught being wrong when the fix landed and the number did
not move. Removing only the `N'` prefix (so `512'({ … })` becomes the legal parenthesised
concatenation `({ … })`, every other byte still parsed at full strength) measured it instead:

```text
top_darjeeling_rnd_cnst_pkg.sv   22 size casts removed   REJECT furthest_position=5899 → ACCEPT
top_earlgrey_rnd_cnst_pkg.sv     11 size casts removed   REJECT furthest_position=5906 → ACCEPT
VERDICT: the numeric size cast is the SOLE blocker of both rows          ← the pre-fix run
```

⛔ **That question is now permanently answered, so keeping it would have kept a script that can only
fail.** Both rows parse with their casts intact, so `before` can never be a rejection again. The
script measures the same two arms and reads them as a differential, which says strictly more than
the boolean it replaced:

| as shipped (casts intact) | casts stripped | verdict |
|---|---|---|
| ACCEPT | ACCEPT | ✅ the construct parses — the post-fix baseline |
| REJECT | ACCEPT | ⛔ the SIZE CAST regressed — removing it recovers the row |
| REJECT | REJECT | ⛔ something ELSE in the row broke — this leaf is not the owner |

⇒ a red run does not merely say *"a row moved"*, it says whether the numeric size cast is the thing
that moved. ⛔ And it still **refuses** when a row carries no numeric size cast at all: both rows are
vendored OpenTitan sources, and a re-vendor that dropped the construct would leave two arms passing
for a reason unrelated to this defect — a test that cannot fail, reported as a test that passed.

At HEAD:

```text
top_darjeeling_rnd_cnst_pkg.sv   numeric size casts 22   as shipped ACCEPT   casts stripped ACCEPT
top_earlgrey_rnd_cnst_pkg.sv     numeric size casts 11   as shipped ACCEPT   casts stripped ACCEPT
VERDICT: both rows parse as shipped, casts intact — the post-fix baseline
```

## 4. The RED controls — every refusal arm was fired, not assumed

A re-baselined instrument that has only ever been seen green is an assertion, not a check. Each arm
was driven red on the shipped baseline (scratch inputs under `rust/target/`, removed afterwards):

| instrument | arm driven | observed |
|---|---|---|
| `casting_type_edge_matrix.sh` | arms 4/5 given a genuinely invalid input | exit **1**, both arms named `⛔ MISMATCH`, controls 1–3 stayed green |
| `corpus_row_cast_bisect.py` | a row carrying no numeric size cast | `REFUSE: … carries no numeric size cast — the row moved` |
| `corpus_row_cast_bisect.py` | shipped REJECT / stripped ACCEPT | `⛔ REGRESSION IN THE SIZE CAST` |
| `corpus_row_cast_bisect.py` | shipped REJECT / stripped REJECT | `⛔ NOT THE CAST … this leaf is not the owner` |

⭐ The matrix's three **control** arms are what make its red readable: they never depended on the
knot, so a red confined to arms 4/5 is this construct and a red that reaches a control is wider.

## 5. ⛔ A defect found IN these instruments while re-baselining them — the repo-root walk could not terminate

`casting_type_edge_matrix.sh` resolved the repository root with

```bash
while [ ! -f CLAUDE.md ] || [ ! -d grammars ]; do cd .. || exit 1; done
```

`cd ..` at `/` **succeeds** and is a no-op, so the `|| exit 1` escape can never fire: run from
outside a checkout the loop spins forever instead of refusing. Measured directly — 201+ iterations
with `pwd=/` and no error; the shipped line run off-root timed out at exit 124 with no output at
all. Its Python sibling `_repo_root.py` in this same directory never had the bug, because
`Path.parents` is finite and it raises `REFUSE: no repository root above …`.

Census over the repository: **3 tracked scripts** carried the pattern, all under
`docs/tasks/artifacts/sv_corpus_grad/` — this one, plus `config_use_param_override/`'s
`census_use_clause_spellings.sh` and `frontend_truncation_probe.sh`, whose copies were strictly
worse (a bare `cd ..` with no `|| exit` at all). All three now walk with an explicit `[ "$d" = "/" ]`
terminator and refuse by name, matching `_repo_root.py`'s contract:

```text
before   exit 124 (timed out — unbounded spin, no output)
after    exit 2   REFUSE: no repository root (CLAUDE.md + grammars/) above <path>   ×3
```

⭐ It surfaced because a RED control was run off-root — the check that was only supposed to prove
the *matrix* could fail is what proved the *walker* could not stop. Directive 12 (no hard-coded
depth) is satisfied either way; what was missing was that a walk needs its own terminator.

## Re-run

```bash
python3 docs/tasks/artifacts/sv_corpus_grad/constant_size_cast/constant_primary_lrm_alternative_audit.py
bash    docs/tasks/artifacts/sv_corpus_grad/constant_size_cast/casting_type_edge_matrix.sh
python3 docs/tasks/artifacts/sv_corpus_grad/constant_size_cast/corpus_row_cast_bisect.py
```

Each exits non-zero if its claim stops holding. ⛔ **They now guard a fix rather than reproduce a
defect**: a non-zero exit here means the guarded indirect-LR admission has been backed out, not that
progress was made. Start at `left_recursion_unhandled=0` for
`ast_pipeline grammars/systemverilog.ebnf --lint-grammar`, and check that nothing in the build
passes `--indirect-lr-admit-starvation-safe-only`.
