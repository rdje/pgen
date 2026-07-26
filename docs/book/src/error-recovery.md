# Error Recovery

By default a PGEN parser is **all-or-nothing**: the first construct it cannot
parse fails the whole parse, and you get one error at the furthest position
reached. That is the right default for a signoff-grade tool — a parser that
guesses is a parser you cannot trust.

Some consumers need the other behaviour. An IDE wants to report *five* syntax
errors, not the first one. A linter wants to keep checking the rest of the file
after a broken statement. For those, PGEN ships **opt-in error recovery**: a rule
can declare that, when it fails, the parser should skip forward to a known
landmark and carry on.

> **Status.** Recovery is a shipped steering surface, but it was **unusable in
> every configuration** until `LANG-CAPABILITY-AUDIT.8` (2026-07-26): the
> generated budget arguments were emitted with the wrong type, so a grammar that
> used `@recover` either failed code generation or produced a parser that did not
> compile. See [The bug that hid this feature](#the-bug-that-hid-this-feature).

## The directives

Recovery is configured with six [semantic annotations](annotation-system.md),
all attached to a rule:

| directive | payload | meaning |
|---|---|---|
| `@recover` | `true` / `false` | **The master switch.** Nothing below has any effect without it. |
| `@sync` | list of strings | Landmark tokens to resynchronize *on* (e.g. `";"`, `"end"`). |
| `@panic_until` | list of strings | Landmark tokens to skip *to*, checked **before** `@sync`. |
| `@recover_budget` | integer | Max recoveries **for this rule** in one parse. Omit = unbounded. |
| `@recover_parse_budget` | integer | Max recoveries **across the whole parse**. Omit = unbounded. |
| `@recover_global_budget` | integer | Max recoveries **across the parser's lifetime**. Omit = unbounded. |

A minimal recovering grammar:

```ebnf
@recover: true
@sync: [";"]
stmt := "a" ";"
      | "b" ";"

start := stmt+
```

## What recovery actually does

When a rule with `@recover: true` fails **every** branch, the generated parser
calls its `recover_with_hints` helper, which:

1. **Checks the budgets.** If any of the three is set and already exhausted,
   recovery declines and the rule fails normally. An unset budget is unbounded.
2. **Searches forward** from the rule's start position for the nearest landmark.
   Every `@panic_until` token is considered first (priority 0), then every
   `@sync` token (priority 1); among candidates the **earliest position** wins,
   with the priority breaking ties.
3. **Moves the parser past the landmark** and returns success. The rule yields an
   empty sequence node, so the surrounding grammar continues from a sane point
   instead of unwinding.
4. **Records a `RecoveryEvent`** — rule name, start, previous and new position,
   which marker kind fired (`PanicUntil`, `Sync`, or `EofFallback`), and where.
   Downstream consumers read these to report every error, not just the first.

With tracing on you see the decision directly:

```text
🛟 Recovery for rule 'stmt': moved parser from 0 to 2 using sync token at 1
🛟 Rule 'stmt' recovered from branch failure using sync=[;] panic_until=[]
   budget(rule=unbounded, parse=unbounded, global=unbounded)
```

That trace is from the probe in `LANG-CAPABILITY-AUDIT.8`: input `X;` against
the grammar above. Without `@recover` the parser rejects it; with `@recover` the
`stmt` rule resynchronizes past the `;` and the parse completes.

## Two limits you must know

### `@recover` only applies to multi-branch rules

Recovery is emitted into the **branch-tournament failure path**, which only
exists for a rule with two or more alternatives. On a single-branch rule the
directive is accepted, passes `--lint-grammar`, and emits **no recovery call at
all**. Measured:

```text
stmt := "a" ";"                   branch-count=1   recover CALL-SITES=0
stmt := "a" ";" | "b" ";"         branch-count=2   recover CALL-SITES=1
```

If you need recovery on a rule that has one production, give it a second
alternative or move the directive to an enclosing choice rule. This is a real
sharp edge — the grammar looks like it recovers and does not.

### Recovery skips text; it does not repair it

The recovered rule produces an **empty node**. Recovery buys you *continued
parsing*, not a reconstructed AST for the broken region. Consumers should treat a
`RecoveryEvent` as "there is a hole here", and the span between
`previous_position` and `new_position` as the text that was discarded.

## What non-users pay

Nothing. Per
[the zero-cost capability rule](quality-and-closure-model.md), this was measured
rather than asserted:

```text
rec_none   helper-definitions=1   CALL-SITES=0
rec_all    helper-definitions=1   CALL-SITES=1
```

The `recover_with_hints` helper is emitted unconditionally but is **inert** — only
a rule that opted in ever calls it. A grammar with no `@recover` produces a
parser byte-identical to one generated before the feature existed; all eleven
shipped PGEN parsers were re-generated and byte-compared to prove exactly that.

## The bug that hid this feature

Worth recording, because it explains why no PGEN grammar uses recovery today and
because the failure mode generalizes.

The three budgets are `Option<usize>` in the generator. They were interpolated
straight into the `quote!` template that emits the `recover_with_hints(...)`
call. `ToTokens for Option<T>` forwards to the payload and emits **nothing** for
`None`, so:

- **no budget set** → an *empty argument slot*:
  `recover_with_hints("stmt", parse_start, &[], &[], , , ,)` — code generation
  aborted with *"expected an expression"*;
- **all three set** → the *bare payload* `4usize` against a parameter declared
  `Option<usize>` — code generation succeeded and `rustc` then rejected the
  generated parser with `error[E0308]`.

So every configuration was broken, and the "working" one was broken one layer
later than anyone had looked. The fix spells both arms explicitly (`Some(#limit)`
/ `None`); the runtime helper already implemented "unbounded" as `None` and did
not change.

**The transferable lesson:** an `Option<T>` must never be interpolated into
`quote!` directly — it is silently wrong in *both* arms. A repository-wide sweep
found no other instance, and the sweep was validated by confirming it flags these
three at the pre-fix commit.

## Related

- [Annotation System](annotation-system.md) — how semantic directives attach to rules
- [Diagnostic & Debug Toolbox](diagnosing-unknowns.md) — tracing a parse decision
- [Quality and Closure Model](quality-and-closure-model.md) — the zero-cost rule
