# `ENGINE-UNIVERSAL-SERVICES.13` — the indirect-left-recursion probe set

Five six-to-nine-rule synthetics that reproduce SystemVerilog **knot A**, price the two ways of
eliminating it, prove the ENGINE does it (P4), and pin the shape that must NOT be absorbed (P5).
Owning leaf: `docs/tasks/ENGINE-UNIVERSAL-SERVICES.md` `.13` slices 3, 5 and 5b. Driver: [`probe.sh`](probe.sh) (both
oracles) — `probe.sh --interp-only` for the cheap column alone.

> ⭐⭐ **P4 is the one to read now.** P2 and P3 are *hand-eliminated* grammars: they answered "which
> rule should absorb the chain?" before any engine could. P4 is P1's grammar with nothing rewritten
> — only the return annotations the real grammars already carry — and the engine's own
> `indirect_lr_elimination` pass turns it into P3's shape. ⛔ P1 stays exactly as it is, and it is
> still REFUSED by the pass: it declares no annotations, and a hop with a residual and no declared
> AST has an engine-default shape no composed template can reproduce. That refusal is the honest
> bound, printed by name.
>
> ⛔⛔ **AND THIS PROBE SET HAS A MEASURED BLIND SPOT — it is faithful to the DEFECT and blind to the
> FIX.** P3's own header says it: *"`ct` and `cast_expr` are GONE here only because this synthetic
> reaches them from nowhere else. SystemVerilog reaches `casting_type` from `cast` too, so a real
> transformation must ADD the clone and KEEP the originals."* Keeping the original is what leaves a
> **starved** holder standing: after `constant_primary` absorbs the chain, `casting_type` (a bare
> reference, hence consumption-transparent) inherits its greed and `cast := casting_type tick lparen
> expression rparen` can no longer match. On the shipped grammar `initial k = 8'(1);` went
> **ACCEPT → REJECT** — a regression none of P1–P4 can exhibit, caught only by
> `stimuli/sv/run_adjudication_repros.py`'s CONTROL row. ✅ **Closed by slice 5b:**
> `p5_transparent_holder.ebnf` is that fifth synthetic, and
> `a_transparent_holder_that_outlives_the_rewrite_starves_the_base_rule` pins it — RED-proven,
> since the pre-5b criterion left all 11 other tests green.

## Why a synthetic at all

Knot A in the shipped grammar is four rules deep inside a 1 481-rule file, and every question about
it ("does the seed matter?", "which rule should absorb the chain?") costs a SystemVerilog
regeneration to ask. These six rules carry the same cycle edge for edge:

| synthetic | SystemVerilog | file:line |
|---|---|---|
| `prim := lit \| cast_expr` | `constant_primary := primary_literal \| … \| constant_cast` | `systemverilog.ebnf:1641` → `:1572` |
| `cast_expr := ct "'" "(" lit ")"` | `constant_cast := casting_type tick lparen constant_expression rparen` | `systemverilog.ebnf:1539` |
| `ct := kw \| prim` | `casting_type := simple_type \| constant_primary \| …` | `systemverilog.ebnf:1032` |

The only compression is the bare-rule-reference chain: SystemVerilog reaches `constant_cast` from
`constant_primary` through `constant_primary_sv_2017`, two hops where the synthetic has one. Nothing
on the cycle's shape changes — `--lint-grammar` reports the same single surviving cycle.

## The measurement (2026-08-12, session #222)

`probe.sh`, both oracles, five inputs. `t'(n)` is ONE cast level and needs no recursion (it is
seeded by `ct`'s own `kw` alternative); `n'(n)` and `t'(n)'(n)` each need the cycle once;
`t'(n)'(n)'(n)` needs it twice.

```text
PROBE                            INPUT            GEN        INTERP     AGREE
p1_knot_a_defect                 n                accept     accept     yes
p1_knot_a_defect                 t'(n)            accept     reject@3   DIVERGE
p1_knot_a_defect                 n'(n)            reject@0   accept     DIVERGE
p1_knot_a_defect                 t'(n)'(n)        reject@3   reject@7   yes
p1_knot_a_defect                 t'(n)'(n)'(n)    reject@3   reject@11  yes
p2_eliminated_at_inner_rule      n                accept     accept     yes
p2_eliminated_at_inner_rule      t'(n)            reject@5   reject@5   yes
p2_eliminated_at_inner_rule      n'(n)            reject@5   reject@5   yes
p2_eliminated_at_inner_rule      t'(n)'(n)        reject@9   reject@9   yes
p2_eliminated_at_inner_rule      t'(n)'(n)'(n)    reject@13  reject@13  yes
p3_eliminated_at_consumer_rule   n                accept     accept     yes
p3_eliminated_at_consumer_rule   t'(n)            accept     accept     yes
p3_eliminated_at_consumer_rule   n'(n)            accept     accept     yes
p3_eliminated_at_consumer_rule   t'(n)'(n)        accept     accept     yes
p3_eliminated_at_consumer_rule   t'(n)'(n)'(n)    accept     accept     yes
```

`GEN` is the real generated parser through the scratch slot (authoritative **by construction**);
`INTERP` is `ast_pipeline --interpret-parse` (authoritative **by verification**, and see the
divergence below).

### P1 — the defect reproduces, and the reproduction is FAITHFUL

`GEN`'s P1 row is the SystemVerilog signature exactly: one cast level parses, every level that needs
the cycle does not. The named mechanism, from `--trace-rules ct,prim,cast_expr` on `t'(n)'(n)`:

```text
💥 Infinite recursion detected in rule 'prim' at position 0
🔙 Speculative parse failed with error 'InvalidSyntax { message: "Infinite recursion detected" }',
   backtracked to position 0 (rule=ct)
🏁 Rule 'ct' selected branch 1/2 consuming 1 chars     ← kw = "t", the ONLY branch left
✅ Exiting rule 'cast_expr' successfully - advanced from 0 to 5     ← "t'(n)"
🏁 Rule 'prim' selected branch 2/2 consuming 5 chars   ← and there it stops
```

The guard kills `ct`'s branch 2/2 (`prim`) at the seed position, so `ct` can only ever be its own
non-recursive alternative. `prim` therefore tops out at exactly **one** cast level and the second
`'(n)` has no derivation at all. Both of `.13`'s real victims are visible in one table: `t'(n)'(n)`
is `int'(2)'(3)`, and `n'(n)` — where the seed comes from `prim`'s own alternative rather than
`ct`'s — is `SV-CORPUS-GRAD.13c.2b`'s `8'(1)`.

### P2 — eliminating at the INNER cycle rule is a REGRESSION, not a fix

⛔ This is the row that changes the design. P2 rewrites `ct` (i.e. `casting_type` — the rule the
lint names first) to `ct_base ( ct_suffix )*`, which is what a generalized
`detect_left_recursive_chain_plan` would do if it simply followed the bare-reference chain. The
result is **worse than the defect**: `t'(n)` went `accept` → `reject`, and nothing else recovered.

The mechanism is PGEN's `*`, which is greedy and does **not** backtrack its iteration count
(`generated/systemverilog_parser.rs:7463` — `loop { if let Some(node) = try_parse(…) { … } else { break } }`,
no re-entry at a lower count). So the eliminated `ct` swallows the whole cast chain, and
`cast_expr`'s own trailing `"'" "(" lit ")"` can never match. The verdict carries its own proof:
`reject@5` on a five-byte input means the parser *reached the end* and then needed four more bytes.

⇒ **The rule that absorbs the chain must be the rule the consumer asks for.** Eliminating at
`casting_type` cannot help, because nothing outside the cycle ever asks for a `casting_type` — a
SystemVerilog expression asks for a `constant_primary`.

### P3 — eliminating at the CONSUMER rule works, and names the cost

P3 rewrites `prim` (i.e. `constant_primary`) instead, and all five inputs accept on both oracles,
including the two-traversal `t'(n)'(n)'(n)`. The transformation it hand-writes is the specification
for acceptance (c)/(d):

- the consumer rule becomes `base ( suffix )*`;
- `suffix` is the cycle's residual — everything after the leading back-reference in the rule that
  closes the cycle (`constant_cast`'s `tick lparen constant_expression rparen`);
- `base` is every non-cyclic left corner, **including a CLONE of each intermediate rule with the
  cycle edge removed** (`cast_expr_seed` here; in SystemVerilog, `constant_cast` re-emitted over a
  `casting_type` shorn of its `constant_primary` arm).

That clone is the blow-up term ANTLR4's objection is about, and P3 is what makes it concrete rather
than rhetorical: it is one rule per intermediate on the cycle path, not an exponential closure —
but it is also a **new rule name in the typed AST**, which is why acceptance (d) has to re-check
`ast_shape_contract` and not merely the parse verdicts.

### ⛔ The interpreter DIVERGES from the generated parser on P1 — in both directions

Two of P1's five rows disagree, and the cheap oracle is wrong on both:

| input | GEN (generated parser) | INTERP (interpreter) |
|---|---|---|
| `t'(n)` | accept | **reject@3** |
| `n'(n)` | **reject@0** | accept |

P2 and P3 agree on every row, so the divergence is specific to a **surviving** cycle: the generated
parsers block on `check_cycle_id`, whose verdict names one blocking frame, while the interpreter has
no cycle guard at all and reaches the same situations through a whole-stack depth ceiling
(`parse_harness_interpreter.rs:746-749`, where the mechanism difference is documented — its
*observable* consequence was not). `PARSE-HARNESS.6.1`'s one indirect-LR case
(`recursion_guarded_memo_isolation`) is green because its cycle happens to be escapable one hop in.
Tracked as `ENGINE-UNIVERSAL-SERVICES.14`; until it closes, no claim about an un-eliminated cycle
may rest on the interpreter column alone.

## Files

| file | what it is |
|---|---|
| `p1_knot_a_defect.ebnf` | the defect — knot A, unmodified |
| `p2_eliminated_at_inner_rule.ebnf` | hand-eliminated at `ct` (`casting_type`) — the regression |
| `p3_eliminated_at_consumer_rule.ebnf` | hand-eliminated at `prim` (`constant_primary`) — the target shape |
| `p4_knot_a_annotated.ebnf` | ⭐ slice 5's acceptance probe — P1 plus the annotations the real grammars carry, so the **engine** eliminates it |
| `p5_transparent_holder.ebnf` | ⛔ slice 5b — P4 plus an OUTSIDE holder of the transparent rule: the one shape P1–P4 cannot exhibit, and the one that regressed |
| `probe.sh` | the dual-oracle driver; restores the scratch slot on any exit |
| `survey/` | slice 4's per-grammar census of the survey instrument |

Every `.ebnf` here names its entry rule `scratch`, so it loads into the scratch slot verbatim
(`cp <file> grammars/scratch/scratch.ebnf`). The entry sits **outside** the cycle, as `source_text`
does in SystemVerilog — with `prim` itself as the entry the outermost `prim` occupies the guard's
`(rule, position)` slot before the derivation starts, and the probe measures the entry choice
instead of the cycle.
