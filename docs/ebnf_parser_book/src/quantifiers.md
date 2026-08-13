# Quantifiers

A quantifier repeats the element immediately to its left. PGEN implements a **single, unified quantifier
engine** (the "Layer-0" engine, `rust/src/ast_pipeline/mod.rs::parse_quantifier_bounds` +
`rust/src/ast_based_generator.rs`), so every quantifier form has the same per-iteration semantics.

## The forms

| Form | Meaning | `(min, max)` |
| --- | --- | --- |
| `?` | zero or one (optional) | `(0, 1)` |
| `*` | zero or more | `(0, ∞)` |
| `+` | one or more | `(1, ∞)` |
| `{n}` | exactly `n` | `(n, n)` |
| `{n,m}` | between `n` and `m` | `(n, m)` |
| `{n,}` | at least `n` | `(n, ∞)` |
| `{,m}` | at most `m` | `(0, m)` |

All seven are first-class and consumed by the codegen, and each form is differentially proven: the
parse-harness structural combinator suite (`make -C rust parse_harness_combinator_gate`) runs an
isolating grammar per form through both the grammar-AST interpreter and the real generated parser and
asserts byte-identical behavior — including the exact accept windows of the four bounded forms
(`item{2}` accepts `xx` and nothing else; `item{2,3}` accepts the 2..=3 window; …).

> Historical note: before `BOUNDED-QUANT.1` (2026-07-07) the four bounded forms were *half-wired* —
> the stimuli generator honored them but parser codegen aborted with `Unknown quantifier`, so a
> grammar using `x{2,3}` could not be compiled. The fix unified the two quantifier-bounds decoders
> into one (`rust/src/ast_pipeline/mod.rs::parse_quantifier_bounds`), which now also decodes the
> brace-stripped spelling the EBNF frontend carries in the raw AST. (No shipped grammar used a
> bounded form — `grammars/regex.ebnf`'s `{n,m}`-looking text lives inside *regex literals*, i.e. the
> regex language's own quantifiers, not EBNF quantifiers.)

```ebnf
optional_sign  := ( "+" | "-" )?
zero_or_more   := digit*
one_or_more    := digit+
exactly_four   := hex_digit{4}
two_to_eight   := hex_digit{2,8}
at_least_two   := item{2,}
up_to_ten      := item{,10}
```

## Per-iteration atomicity (Layer-0)

The unified engine wraps **every** iteration of a quantifier in an atomic try: if an iteration fails
partway, the parser cleanly backtracks to the position just before that iteration. For the minimum-count
forms (`+`, `{n}`, `{n,…}`), the whole quantifier rolls back to before the quantifier on a min-count
failure. This makes the repetition forms behave uniformly — there is no asymmetry between the first and
subsequent iterations.

A zero-length guard prevents an inner rule that matches the empty string from looping forever, and a large
safety limit backstops pathological repetition.

## Repetition is POSSESSIVE — a quantifier never gives an iteration back

Per-iteration atomicity is about a **failed** iteration. It is *not* backtracking into the loop count.
Once an iteration has succeeded, nothing later in the sequence can make the quantifier hand it back:

```ebnf
# Measured: this rule rejects BOTH "ab" and "aaab".
bt := "a"* "ab"
```

On `"ab"`, `"a"*` consumes the `a`, the loop stops, and `"ab"` is then attempted against the leftover
`b` and fails. A regex engine (or a CFG parser) would retry with a shorter run; PGEN does not — the
emitted loop breaks on the first failing iteration and enforces only the *minimum* count. So:

> **Never write `body* closer` where `body` can also match `closer`.** The loop will swallow the
> closer and the rule can never complete.

Two idioms fix it, both proven to work:

```ebnf
# 1. STATIC closer — guard the loop element with a negative lookahead, so the loop
#    stops of its own accord and no give-back is needed.
quoted := "q" "/" ( !"/" builtin_any_char )* "/"

# 2. DYNAMIC closer (the closer is not known until parse time — a Raku-style
#    user-chosen delimiter, a here-document terminator). Register it in the semantic
#    store on the way in and gate the body against it. See The Semantic Store.
@emit_fact:  { kind: qdelim, name: $body, family: d }
open      := delim -> { body: $1 }
@predicate:  { name: lacks_fact, args: [qdelim, $body], phase: post }
body_char := builtin_any_char -> { body: $1 }
@predicate:  { name: has_fact, args: [qdelim, $body], phase: post }
close     := delim -> { body: $1 }
quoted    := "q" open body_char* close
```

⚠️ Idiom 2 carries one bound worth knowing before you rely on it: the fact store is **monotone within
a parse** — there is no retraction directive, and `@close_scope` does not retire facts — so a *second*
quoted string in the same input still sees the first one's delimiter fact. It is correct for one
instance per parse, and the repeated-instance case is a tracked gap
(`docs/tasks/LANG-CAPABILITY-AUDIT.md` `.3b`/`.4`).

### ⛔ A stop-guard belongs to the CALL SITE, not to the rule

Idiom 1 is written inline for a reason. The guard encodes *"leave something for what comes after
me"* — which is a fact about **this** use of the loop, not about the loop. Factor the guarded loop
into its own rule and reuse it from a caller that wants the whole run, and you have broken that
caller:

```ebnf
# Reserve one `a` for the trailing `"a"`. Measured: ACCEPTS "aaa" and "a".
ok  := ( "a" &"a" )* "a"

# The SAME loop, reached from a holder with nothing after it.
# Measured: REJECTS "aaa" — the guard reserves an `a` that nobody claims,
# so the parse cannot consume the input. Delete the `&"a"` and it accepts.
run := ( "a" &"a" )*
top := run
```

So: **write the guard where the closer is**, and if two callers of one rule disagree about whether a
residual follows, they need two rules. This is the authoring-tier form of a constraint the engine hits
too — PGEN's automatic left-recursion elimination has to shear a separate clone for exactly this
reason (`docs/tasks/ENGINE-UNIVERSAL-SERVICES.md` `.17`).

## Quantifying a group

A quantifier binds to the single element on its left — use a group to repeat more than one element:

```ebnf
# repeat the (',' expression) pair — the classic comma-separated-list idiom
arg_list := expression ( "," expression )*

# WRONG: '+' binds only to 'b'
ab := a | b+        # == a, OR one-or-more b
# RIGHT:
ab := ( a | b )+    # == one-or-more of (a or b)
```

## Returning a quantified group

A quantified element produces a **list** of its iterations. The return-annotation language has dedicated
operators to collect, spread, and flatten those lists — `$N*`, `$N**`, and the extraction-spread `$N::n*`
— described in [Return Annotations](return-annotations.md). The common pattern:

```ebnf
items := item ( "," item )* -> [ $1, $2* ]      # [first, ...rest]
```

## Not implemented: the probability quantifier `@N%`

The meta-grammar `grammars/ebnf.ebnf` describes a probability quantifier (`@75%`). It is **parsed but not
consumed by the codegen** — it has no effect on a generated parser or stimuli generator. Do not use it;
steer stimuli weighting through the [semantic-annotation](semantic-annotations.md) `@…` directives
instead.
