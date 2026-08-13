# `ENGINE-UNIVERSAL-SERVICES.17` slice 2 — what a call-site guard would cost on the SHIPPED grammar

Slice 1 left option **(iii)** — a call-site follow-restriction guard synthesized by the eliminator
onto the sheared clone — as the front-runner, with one open question the leaf required be answered
**against the shipped grammar, never a synthetic**:

> is the holder's residual FIRST set statically computable at each of the 28 rows, and does the
> guard stay sound when that residual is nullable or when two holders of the same clone disagree?

`ast_pipeline <g> --report-indirect-lr-plan` now answers all three. This directory holds the
captured census and a self-checking bank that pins it.

| file | what it is |
|---|---|
| `probe.sh` | the pinned census — 9 declared cases, rc 1 on any disagreement |
| `census_systemverilog.txt` | full `PGEN_INDIRECT_LR_DUMP_ALL=1` report, `grammars/systemverilog.ebnf` |
| `census_systemverilog_lrm_profiled_wrapper.txt` | the same for the LRM-profiled wrapper |

Re-run: `bash docs/tasks/artifacts/engine_universal_services/guard_feasibility/probe.sh`

## The guard, and the criterion the census evaluates

After the `.13` rewrite the base reads `X := X_lr_base ( X_lr_suffix )*` with a **possessive** `*`
(slice 1: PGEN emits Perl's `a*+`). A holder `H := X residual` starves when the loop eats text
`residual` needed. The guard is slice 1's measured Q4 repair, scoped to the call site:

```text
X_guarded := X_lr_base ( X_lr_suffix &FIRST(residual) )*
```

Commit an iteration only when the position after it can still start `residual`. Five outcomes, and
the reason each is a *separate* bucket rather than a yes/no:

| verdict | condition | what it means for (iii) |
|---|---|---|
| `guardable` | competing, `FIRST(suffix) ⊆ FIRST(residual)` | the guard is sound and refuses no iteration earlier than the fatal one — see the `~` qualifier below before reading it as "closed" |
| `no_competition` | `FIRST(suffix) ∩ FIRST(residual) = ∅` | the greedy loop can never take text this holder needed — **no guard owed** |
| `residual_nullable` | `residual` can match empty | the holder cannot starve here at all — **no guard owed** |
| `guard_incomplete` | they compete and `FIRST(suffix) ⊄ FIRST(residual)` | the guard would cut the loop short at an intermediate iteration ⇒ it trades one under-acceptance for another |
| `undecidable` | a FIRST set is `unresolved`, or the suffix is nullable | nothing may be concluded in either direction |

**Why `guardable` is sound in both directions.** Completeness: at any position where the loop
continues, the suffix matched, so that position starts with `FIRST(suffix)`; containment then makes
the guard pass there, so the only iteration it can refuse is the last one — the fatal one. No
over-acceptance: `L(X_guarded) ⊆ L(X)`, so the holder still accepts only strings in
`L(X)·L(residual)` — the guard *recovers* derivations the declarative grammar already licensed, it
never invents one. Every FIRST set here is the shared over-approximation
(`ast_pipeline/first_set.rs`), and both tests use it in the sound direction: disjointness of
over-approximated sets implies disjointness of the true sets, and the containment test compares
against exactly the set the emitted guard would evaluate.

## The three answers

**1. Is FIRST statically computable at every row? YES — `undecidable = 0`** at all **185** surviving
starvation sites (126 SystemVerilog + 59 wrapper). Not one site needs a conservative decline.

**2. Nullable residuals — 29 of 126 (SV) and 19 of 59 (wrapper).** A nullable residual is not a
soundness problem for the guard; it means **no guard is owed**, because the holder succeeds on the
empty match and cannot starve. ⛔ It is also a measured **over-count in the shipped structural
criterion**, which counts any syntactically non-empty residual. Example, now legible because this
slice also fixed the report's group parenthesisation:

```text
constant_expression alt#0 — residual '( binary_operator attribute_instance* constant_expression_operand )*
                                      ( question attribute_instance* constant_expression colon constant_expression )?'
   [guard=residual_nullable first={!%&*+-/<=>?^|~} +ε~ hops=1]
```

⚠️ And it is a hole in the transitive walk, recorded rather than closed: `rules_transparent_to`
follows only **syntactically** empty residuals, so a nullable-but-non-empty holder stops the chain
and a hazard one hop further out is never searched from there.

**3. Do two holders of the same clone disagree? At the rules that matter, NO.** 17 of 28 SV
candidates need exactly **one** guard variant. Every multi-variant row (2–6 variants) is a
non-dominator member of one of the two knots, and every one of them is `BLOCKED` for an independent
reason, so the multiplier never has to be paid.

## The headline — and the row that changes another leaf's premise

| grammar | starvation-safe (shipped criterion) | **guard-feasible (option (iii))** |
|---|---|---|
| `systemverilog` | **0 / 28** | **16 / 28** |
| `systemverilog_lrm_profiled_wrapper` | 3 / 18 | **13 / 18** |
| `ebnf` and all others | — (fully absorbed / no cycles) | — |

Both of SystemVerilog's remaining knots are guard-feasible **at their dominators**, each with one
variant:

```text
[candidate] constant_primary   guard: FEASIBLE  suffix_first={'/}~  variants=1 {'/}~  max_hops=1
    ⛔ starved by cast alt#0 — residual 'tick lparen expression rparen'   [guard=guardable first={'/}~ hops=1]

[candidate] property_expr      guard: FEASIBLE  suffix_first={-/}~  variants=1 {-/}~  max_hops=0
    ⛔ starved by prop_primary_sv_2017 alt#13 — residual 'implies property_expr' [guard=guardable first={-/}~ hops=0]
    ⛔ starved by prop_primary_sv_2023 alt#13 — residual 'implies property_expr' [guard=guardable first={-/}~ hops=0]
```

⭐⭐ The second row is the one to read twice. `.15` was opened on slice 4's finding that the SVA
property knot **has no starvation-safe base rule**, and concluded it needs `.3`'s
precedence-declaration service. The census says the *starvation* blocker is closable at the
dominator `property_expr` with **one guard variant and ZERO clone hops** — the cheapest row in the
whole census. ⛔ That is a claim about the starvation blocker only: whether the
annotation-composability check also passes at `property_expr` is **not** measured here (the wrapper
refuses `property_expr` for a missing return annotation on `property_expr_sv_2017` alternative 5)
and is slice 3's first check.

## ⛔⛔ THE QUALIFIER ON EVERY NUMBER ABOVE — feasible is not closed, and it is measured at 157/157

`guard-feasible 16/28` means *a guard is expressible and provably **sound** at 16 candidates*. It
does **not** mean 16 knots close. The report marks the difference with `~`:

```text
guard: FEASIBLE  suffix_first={'/}~  variants=1 {'/}~  max_hops=1
```

`~` = the byte set is an **over-approximation**, so the guard passes at positions where the residual
cannot actually start and therefore silently declines to refuse the fatal iteration. It never
becomes unsound — `L(X_guarded) ⊆ L(X)` still holds, so an over-permissive guard can only fail to
fire, never over-accept — but it is not a proof.

**Measured: 157 of 157 sites are over-approximated. Not one exact guard exists on either grammar**
(probe case C7, pinned absolutely against the uncapped report). Two compounding causes:

1. **Structural** — `FirstSetSummary::byte_decided` is true only for single-byte-decided shapes, so
   any multi-element residual (`tick lparen expression rparen`) is approximate by construction.
2. **Layout** — `trivia := (line_comment | block_comment)*` (`grammars/systemverilog.ebnf:619`) is
   nullable and leads every token, so **`/` is in the FIRST set of every token**. Concretely,
   `int'(2)/*c*/'(3)` slips a byte-test guard.

⇒ **This flips slice 3's starting point.** The byte test is not the cheap win it looked like after
slice 1; on this grammar a **trivia-aware structural lookahead** (`&( residual )`, exact, at the
price of a per-iteration sub-parse instead of one byte compare) is effectively mandatory. Slice 3
prices that trade — it is not decided here — but it may no longer assume the cheap form suffices.

⭐ The same layout fact is why **`no_competition = 0`**: with `/` in every set, no two token-led
FIRST sets are ever disjoint, so the disjointness refinement is sound, implemented, and structurally
unable to fire here. Pinned at 0 in both directions (case C5) so a future layout-model change is
noticed rather than assumed.

## ⛔ Two things this slice deliberately did NOT do

**A. It did not let a report edit reach the parser.** `render_elements` is documented *"report-only
— nothing parses this back"*, and that comment is false: `indirect_lr_elimination.rs:483` decides an
ambiguity refusal by comparing two of its strings. Adding the group parentheses there would have
made that comparison strictly finer — *refuse less, absorb more* — a shipped-parser change riding on
a cosmetic fix. So `render_elements` is **frozen byte-for-byte** and the human-facing
`render_elements_display` is separate, which makes "nothing the parser executes changed" true **by
construction rather than by measurement**. A unit test pins the collision in the frozen direction
(`( "a" "b" )?` and `"a" "b"?` must still render identically) and its separation in the display one.
The coupling itself is routed as `.18`.

**B. It did not judge a candidate on partial evidence.** Every test here reads `suffix_first` as an
over-approximation; a union over a *truncated* route set is an **under**-approximation, which makes
`FIRST(suffix) ⊆ FIRST(residual)` pass too easily and yields a false `FEASIBLE` that nothing
downstream could notice. Truncation therefore poisons the summary to `unresolved`, forcing every
site to `undecidable`. It costs nothing today — the widest knot enumerates 80 routes against a
budget of 128 — and it is exercised by a test through a budget-parameterised entry point, because an
unreachable branch that must fail safe is exactly the branch a release trusts and never runs.
