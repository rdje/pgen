# Codegen Mental Model

A correct mental model of what the generators do with your grammar makes surprising parses and shapes
debuggable. This chapter sketches the path from EBNF construct to generated Rust, without requiring you to
read the codegen source. The generators are `rust/src/ast_based_generator.rs` (parser) and
`rust/src/ast_return_transform.rs` (return shaping).

## The generated parser is a recursive-descent PEG

Each rule becomes a function that tries to match at the current input position and either succeeds
(advancing the position and returning a value) or backtracks. The semantics are **PEG** semantics:

- **ordered choice** `|` tries alternatives left-to-right and commits to the first match — no global
  ambiguity, no longest-match across alternatives (see [Rules and Expressions](rules-and-expressions.md));
- **sequences** match elements in order, backtracking the whole sequence if any element fails;
- **lookaheads** `&` / `!` test without consuming (see [Lookaheads](lookaheads.md));
- a **packrat memo** caches per-rule, per-position results so repeated work is avoided; the memo also
  replays semantic-store side effects on a cache hit, so context-aware gating stays correct under
  backtracking.

This is why alternative ordering and prefix-sharing matter so much: the parser does what you *wrote*, in
the order you wrote it.

## Terminals → matchers

| EBNF construct | Generated matcher (conceptually) |
| --- | --- |
| `"lit"` / `'lit'` | `match_string("lit")` |
| `r"lit"` | `match_string` with no escape decoding |
| `/regex/` | `match_regex(/regex/)` (the embedded Rust regex engine) |
| `any_char` / `builtin_any_char` | `match_any_char()` — a native UTF-8 char matcher, no regex engine |
| `ascii_char` / `builtin_ascii_char` | a native ASCII char matcher |
| rule reference | a call to that rule's function |

The `any_char` built-ins are what let a grammar be regex-engine-independent (the `REGEX-SELF-HOSTING`
property of `grammars/regex.ebnf`).

## Quantifiers → the unified loop

Every quantifier (`?` `*` `+` `{n}` `{n,m}` `{n,}` `{,m}`) compiles to **one** loop shape with
per-iteration atomicity (the Layer-0 engine, [Quantifiers](quantifiers.md)). A quantified element yields a
list of iteration results, which the return annotation then collects/flattens.

## Optionals and grouping → normalization

`[ expr ]` is normalized to `( expr )?` by the frontend before codegen (see [Terminals](terminals.md)).
Left-recursion is eliminated as a normalization pass when requested. To **see** the post-normalization IR
the generators consume, dump it:

```bash
./rust/target/debug/ast_pipeline grammars/foolang.ebnf --generate-parser \
    --dump-gen-ast gen_ast.json --dump-gen-ast-pretty --eliminate-left-recursion --output /tmp/p.rs
```

## Return annotations → tree construction

After a rule's body matches, the captured positionals (`$1`, `$2`, …) feed the return transform, which
builds the literal / object / array / reference result. With no `-> …`, the
[implicit / passthrough policy](return-policy.md) applies.

## Semantic annotations → runtime guards and store ops

`@…` directives compile into runtime hooks around the rule: a `@predicate` becomes an accept/reject guard
evaluated against the semantic store; `@emit_fact` / `@open_scope` become store writes; `@profiles`
becomes a profile guard so a rule only participates under the selected grammar profile (e.g. `sv_2017` vs
`sv_2023`). See [Semantic Annotations](semantic-annotations.md).

## Well-formedness and trustworthiness

Two proofs back the codegen and are the grammar author's safety net:

- **`--lint-grammar`** — static well-formedness: left-recursion info, non-terminating **ERRORS**, and
  ordered-choice **shadowing WARNINGS** (a later alternative that an earlier one already subsumes). Run it
  after every grammar edit.
- **`--report-certificate-coverage`** — for every rule, is it covered by a verified unreachability
  **PROOF** or a verified reachability **WITNESS**? `UNKNOWN=0` with no failures, deterministic at seeds
  `0/7/42`, is the objective "trustworthy on this grammar" number.

Both are detailed in `TOOLBOX.md` and the platform book's
[Grammar Well-Formedness](../../book/src/grammar-wellformedness.md) chapter. The doctrine: an unreachable
or non-terminating rule is a grammar **defect** to fix, not a residual to accept.
