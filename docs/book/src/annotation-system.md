# Annotation System

Annotations are one of the defining differences between PGEN and a simpler grammar-to-parser tool.

## Two Annotation Families

### Return annotations

Return annotations shape the AST that generated parsers return. They are the normative way to control parse-result structure instead of treating the generated tree as fixed.

### Semantic annotations

Semantic annotations steer parser-generation behavior and related transformation/runtime choices in the Rust AST pipeline.

They also now have a stricter same-line scanner contract. Inline rule-body annotations consume only their own payload:

- quoted payloads,
- balanced structured payloads such as `{...}`, `[...]`, or `(...)`,
- or a scalar token payload.

They do not get to swallow the rest of the rule body. That matters because branch-local hints like `@sample: "..." alpha | beta` are only useful if `alpha | beta` still survives as real branch syntax after tokenization.

That steering now includes more than regex-target tweaks. Literalish directives such as `@sample`, `@literal`, `@example`, and legacy `@stimulus` can now be used as parser-proven stimuli seeds for:

- regex atoms,
- non-regex non-OR rule expansions,
- and inline branch-local OR alternatives.

PGEN also now has a narrower replay-only variant: `@probe_sample`.

- `@sample` is the ordinary always-on literalish steering tool.
- `@probe_sample` is for target-drive replay.
- `@probe_sample` only short-circuits when that rule is the active generation entry, so it can help probe broad dependency rules without collapsing ordinary top-level generation transitively.

That widened the annotation system from "token-shape nudges" into a real narrow branch-steering surface for coverage-guided replay, while still keeping the project rule that sample hints must be justified by parser-backed evidence rather than sprayed across a grammar blindly.

#### Grammar-level layout policy: `@whitespace_sensitive` (WS-DIRECTIVE)

By default a generated parser is whitespace-INSENSITIVE — it auto-skips layout (whitespace + unclaimed comment introducers) before terminals and regex tokens and consumes trailing layout. A whitespace-sensitive language (regex is the canonical case: a space is a literal atom) declares its policy **in the grammar** with the grammar-level `@whitespace_sensitive: true` directive, or granularly with `{ terminals: …, regex_tokens: …, trailing: … }` (`grammars/systemverilog_preprocessor.ebnf` declares `{ regex_tokens: true }`). The directive is compile-time only (the policy is burned into the emitted parser; the parse-harness interpreter derives its layout policy from the same compiled declaration), conflicting duplicate declarations are hard errors, and malformed payloads are linted (`W_SEM_INVALID_WHITESPACE_SENSITIVE_PAYLOAD`) and rejected at generation. This replaced the historical engine-internal gate that keyed layout on the grammar's *file name* — any grammar, including a scratch/probe grammar, can now be whitespace-sensitive. Grammar-author reference: the ebnf parser book's *Semantic Annotations* chapter (Layout policy section); proof cases: the structural combinator suite's three `layout_*` cases.

#### Grammar-level default dialect profile: `@default_profile` (DEFAULT-PROFILE)

A grammar that gates rules by dialect profile (`@profiles`) can declare what an **unspecified** requested profile resolves to, with the grammar-level `@default_profile: <name>` directive. `grammars/regex.ebnf` declares `@default_profile: pcre2`, making regex PCRE2-faithful by default: the `@profiles: ["relaxed"]`-gated constructs (`\u`, …) are excluded unless the `relaxed` profile is requested explicitly. The declared default is carried by the artifact itself — the generated parser embeds a `DEFAULT_GRAMMAR_PROFILE` constant, its constructor starts on it, and `set_grammar_profile(None)` *restores* it rather than falling back to a permissive unset state — and the generation-side profile filter and the parse-harness interpreter resolve from the same compiled declaration. An explicit requested profile always wins; without the directive, an unspecified profile keeps today's permissive guard (all rules active). Conflicting duplicates and malformed payloads are hard errors, linted early as `W_SEM_INVALID_DEFAULT_PROFILE_PAYLOAD`. This replaced the historical engine-internal gates that keyed the regex→`pcre2` default on the grammar's *name* — any grammar can now declare its own default. Grammar-author reference: the ebnf parser book's *Semantic Annotations* chapter (Default profile section); proof cases: the structural combinator suite's `profile_unspecified_permissive` / `profile_default_gate` pair.

#### Grammar-level profile request aliases: `@profile_alias` (PROFILE-ALIAS)

A profiled grammar can also declare which request **spellings** resolve to its canonical profile names, with the grammar-level `@profile_alias: { "<spelling>": <canonical>, … }` map directive. `grammars/systemverilog.ebnf` declares `2017`/`ieee1800-2017`/`ieee_1800_2017` → `sv_2017` (plus the `2023` and `1364-2005` groups), which is why `--profile 2017` works everywhere. Multiple declarations **merge** (an alias table is naturally written in groups); re-declaring a spelling with a different target is a hard error. Every alias target must be a profile the grammar actually declares (the union of its `@profiles` lists and its `@default_profile`), and a spelling may not shadow a canonical name — so typos and alias-chains are compile errors, linted early as `W_SEM_INVALID_PROFILE_ALIAS_PAYLOAD`. The map is carried by the artifact itself — the generated parser embeds a sorted `GRAMMAR_PROFILE_ALIASES` constant and resolves spellings case-insensitively inside `set_grammar_profile` — and the registry's profile oracle, the generation-side profile filter, and the parse-harness interpreter resolve from the same compiled declaration. An undeclared spelling passes through un-coerced (it simply matches no `@profiles` list). This replaced the historical engine alias tables (a `"systemverilog"`-name-gated match in the parser registry and a global spelling table on the generation side) — any grammar can now declare its own request spellings. Grammar-author reference: the ebnf parser book's *Semantic Annotations* chapter (Profile aliases section); proof cases: the structural combinator suite's `profile_alias_resolves` / `profile_alias_unknown_passthrough` pair.

#### ⭐ A `@profiles` gate and a negative lookahead: the guard does NOT vanish (`ENGINE-UNIVERSAL-SERVICES.46`)

**If you gate a rule `X` out of a profile, a `!X` negative lookahead elsewhere keeps constraining
under that profile.** That is deliberate, and it is worth stating because the naive reading is the
opposite. A lookahead is a **constraint**, not a production — it derives nothing — so removing its
subject from a dialect must not loosen it. Concretely, a negative lookahead's body is evaluated with
`@profiles` gating **ignored** under exactly the profiles in which that body would otherwise be
unsatisfiable.

⛔ **It did not always work this way, and the bug is worth knowing about because it is the shape to
look for elsewhere.** Gating does not delete a rule; it makes the rule's parse method backtrack
unconditionally. A negative lookahead fails only when its body *matched*. Composed, `!X` used to
succeed **vacuously** the moment `X` was gated away — so the *narrower* profile accepted strings the
wider one rejected, silently, with `--lint-grammar` reporting every counter at zero. ⭐⭐⭐ That broke
the invariant every dialect-profile system depends on: **gating a rule out of a profile must only
ever REMOVE strings from the language, never ADD them.**

⭐⭐ **The condition is satisfiability, and the distinction matters when you write a profiled
grammar.** A `@profiles` gate gets used two ways:

| use | shape | what gating does to `!X` |
|---|---|---|
| **narrowing** | the rule is absent from the narrow dialect, nothing replaces it | the guard would be **deleted** — so the engine bypasses the gate inside the lookahead and the guard survives |
| **selection** | sibling rules, one per dialect, behind a dispatcher | the guard is **switched**, never deleted — one alternative is always live, so the engine leaves it alone |

`grammars/systemverilog.ebnf` uses both. Its keyword guard is selection:

```ebnf
reserved_non_keyword_identifier := reserved_non_keyword_identifier_sv      # @profiles ["sv_2017","sv_2023"]
                                 | reserved_non_keyword_identifier_v2005   # @profiles ["verilog_2005"]
non_keyword_identifier := escaped_identifier | !reserved_non_keyword_identifier simple_identifier
```

`class` is a legal IEEE 1364-2005 identifier and an IEEE 1800 keyword, and `reg class;` parses under
`--profile 1364-2005` and is refused under `--profile 2017` — which only works because the guard
switches rather than unions. A repair that bypassed the gate *everywhere* would have broken exactly
that.

⚠️ **One case remains vacuous and cannot be helped by any static rule: a requested profile the
grammar never declares.** `set_grammar_profile` passes an unknown spelling through un-coerced, and in
that state every `@profiles`-gated rule is absent. The engine treats such a profile as maximally
narrow — a `!X` whose body is unsatisfiable there is bypassed too — but a grammar author should
request a declared profile.

`--lint-grammar` reports both polarities as notes (`profile_gated_negative_lookaheads`,
`profile_gated_positive_lookaheads`) so the sites are visible; see
[Grammar Well-Formedness § the profile-gated lookahead arm](grammar-wellformedness.md) and, for the
standing proof lane, [The Parse Harness § the profile-gate monotonicity probe](parse-harness.md).

#### Rule-level stimuli separator cohesion: `@quantified_separator` (STIMULI-SIGNOFF.12)

A line-oriented (or otherwise junction-sensitive) rule can declare how the **stimuli generator** separates its **stacked quantified renderings**, with the rule-level `@quantified_separator` directive. `grammars/systemverilog_preprocessor.ebnf` declares `{ insert: "\n", satisfied_by: ["\n", "\r\n"] }` on `pp_item`: every directive's trailing newline is optional in that grammar, so two stacked `pp_item` renderings could fuse into one line (a `` `define `` body eats to end-of-line and would swallow the next directive) — the generator therefore inserts `"\n"` between adjacent renderings unless the junction already carries a line break, and `"\r\n"` also counts as one *because the grammar's own `newline` token is `/\r?\n/`*. That last part is the design point: alternate junction spellings are **grammar knowledge, declared in the grammar** — the engine hardcodes no language-specific spellings. The shorthand `@quantified_separator: "<sep>"` means "satisfied only by itself"; the object form requires `satisfied_by` to contain `insert`. The directive binds to the **quantified** rule (the item), so every container that stacks it inherits the policy. It steers stimuli generation only — it compiles to zero runtime directives and the annotated rule keeps the fast-path parser emission, so declaring it is emit-neutral for the generated parser (pinned by a codegen test). Malformed payloads are linted (`W_SEM_INVALID_QUANTIFIED_SEPARATOR_PAYLOAD`) and fail generation loudly at the first join that would need them. This replaced the last per-language literal in the stimuli generator (a hardcoded `systemverilog_preprocessor` + container-rule-name gate) — any grammar, including a scratch/probe grammar, can now declare separator cohesion. Grammar-author reference: the ebnf parser book's *Semantic Annotations* chapter (Quantified separator section).

#### Generation-side store gates: `@gen_emit_fact` / `@gen_predicate` (STIMULI-SIGNOFF.13.4)

Some store constraints are **true of the language but unsound to enforce at parse time**, because the reference target may legally appear *later* in the input. The motivating case is the regex scan-substring capture list: PCRE2 validates `(*scs:('name'))` against the **full-pattern** named-capture inventory, and forward references are legal (`(*scs:('a'))(?<a>x)` compiles) — so a parse-time `@predicate has_fact(...)` would wrongly reject valid patterns. Yet a **generator** that renders such a reference blindly emits mostly-invalid samples (the generator⟷parser duality break the hunter finds). The fix is the generation-side dual pair: `@gen_emit_fact` (payload = the exact `@emit_fact` schema) registers a fact into the **generation-time** store the instant the annotated rule renders — bind it to a rule whose whole render *is* the registered value, like regex's `capture_name` — and `@gen_predicate` (payload = the exact `@predicate` schema) makes the annotated rule render a value **drawn from the live store**: `has_fact(K, $ref)` draws a live `K`-fact *name*, `fact_count_at_least(K, $ref)` draws an integer in `1..=count(K)`, and zero live facts make the rule cleanly **ungeneratable** (the generation tournament backtracks to a sibling; the certificate-coverage witness pass arms an upstream producer prelude instead). The drawn subset — references to *already-generated* targets — is sound by construction; forward-referencing-only positions are honestly not generated, while the **parseable language is untouched**: both directives are stimuli-steering only, compile to zero runtime directives, and the annotated rules keep the fast-path parser emission (pinned by codegen tests). Payload lint: `W_SEM_INVALID_GEN_EMIT_FACT_PAYLOAD` / `W_SEM_INVALID_GEN_PREDICATE_PAYLOAD`, through the same payload parsers as the parse-time pair. First consumer: `grammars/regex.ebnf`'s scs capture list (`scs_capture_name` / `scs_capture_number`), which closed the last observed regex duality-break class. Grammar-author reference: the ebnf parser book's *Semantic Annotations* chapter (Generation-side store gates section).

#### Semantic refs resolve against the rule's *produced* structure (SEMREF-SHAPED)

A semantic-directive argument reference (`$name`, `$a.b`) on rule **X** resolves against **the structure X produces** — not X's internal parse plumbing:

- if X has a `->` return annotation that yields an object, `$name` is a field/path lookup into that **shaped** structure down to a scalar leaf. This is **shaped-only**: the rule's declared output *is* its semantic surface; the engine does not reach back into raw parse content for a `->` rule (an absent/non-scalar field is an unresolved ref — a grammar-author error, surfaced loudly, never silently mis-parsed).
- if X has **no** `->`, resolution is the unchanged raw sub-rule-name search.

This lets a single rule legitimately carry both a `->` and a content-`$ref`-bearing `@predicate`/`@emit_fact` (e.g. `numeric_backreference … -> {…, index:$2}` gated by `@predicate fact_count_at_least(regex_capture_group, $index)` reading its own produced `index`). The capability is parser-agnostic and keyed purely on the content variant, so no-`->` rules are byte-identical to prior behavior. First consumer: the regex `\NN` PCRE2 octal-vs-backref disambiguation (downstream report `PGEN-RGX-0084`).

Together, these two annotation families make PGEN a parser platform rather than only a parser emitter.

#### Rule-reference syntax in semantic-directive payloads (SEMREF-SYNTAX)

Semantic-directive payloads (`@emit_fact: { … }`, `@predicate: { … }`, `@export_to_library: { … }`, `@import_from_library: { … }`, `@open_scope: { … }`, `@close_scope: { … }`) can reference values via four shapes — all four are accepted to **unbounded depth** with no static cap:

| Form | Meaning | Example |
|---|---|---|
| `$<ident>` | Named ref into the current rule's shaped output (`->` rule) OR sub-rule-name descendant search (raw rule) | `$body` |
| `$<int>` | Positional ref to the N'th child (1-indexed) of the current rule's parse content | `$1` |
| `$text` / `$0` | Whole-match ref — the rule's entire matched source text as one string `Terminal` (native, Rust-regex-free equivalent of a `/.../` capture; Perl5 convention `$0`=whole match, `$1..$N`=captures; lets `octal_digit+ -> $text` yield `"777"` instead of a structured `Quantified`). Valid only as a base value — `$0::first`/`$0.x`/`$0[i]` is an error (`E_RET_WHOLE_MATCH_NOT_COMPOSABLE`). REGEX-SELF-HOSTING.3/.3a. | `digit+ -> $0` |
| `$<head>.<ident>(\.<ident>)*` | Dotted property-access chain over the shaped JSON object-key path / raw sub-rule tree | `$name.body`, `$1.body.subkey` |
| `$<head>(\.<ident>\|\[<int>\])*` | Mixed dotted + non-negative-integer indexed-access chain | `$items[0].name`, `$matrix[0][1]`, `$a.b[0].c[1].d.e[2].r.z` |

**Subset boundary** is fixed: dotted property access + non-negative integer array indexing only. **NOT** full JSONPath — no filters (`[?(@.foo)]`), no wildcards (`*`), no recursive descent (`..`). Each of those would be its own future leaf with its own runtime semantics and depth analysis.

**Durable no-depth-limit guarantee.** The reference depth is structurally unbounded at every layer of the implementation — EBNF regex `*` quantifier (strict-math unbounded), hand-rolled `parse_rule_reference` loop with no max-iteration cap, runtime resolver iterator-based walk over the lexed segment vector. Locked by two regression tests (one for pure-dotted depth at 64 segments, one for mixed dotted + indexed at 64 segments) — any future "cap the depth for safety/perf" proposal must fire those tests and justify the cap in its own leaf, or back off.

**Strict-bracket / strict-trailing-dot policy.** Malformed forms — a bare `.` not followed by an identifier, or a `[` not followed by `<digits>]` — roll back to before the offending segment, leaving the leftover for the surrounding parser to handle. The reference parser never silently swallows a malformed segment; the surrounding payload parser then errors or falls back to `Raw`.

**Resolution path.** Resolution dispatches on the rule's content variant: shaped-JSON content (rules with `->`) walks the object/array via `serde_json::Value::get`, which polymorphically accepts both `&str` (property) and `usize` (index); raw-tree content (rules without `->`) uses `find_semantic_named_descendant` for property segments and `find_semantic_indexed_child` for `[N]` segments. The walk is iterative across the lexed segment vector — no recursion on the reference depth itself.

#### Architecture note: which parser parses what

There are **two distinct surfaces** for parsing semantic-annotation source, and they are easy to confuse:

1. **`grammars/semantic_annotation.ebnf` → `generated/semantic_annotation_parser.rs`** — this is the **EBNF-language surface**: it parses semantic-annotation `*.ebnf` source text (or freestanding annotation strings passed through the `parse_annotation` embedding API). This is what `grammars/semantic_annotation.ebnf` formally defines.
2. **`rust/src/ast_pipeline/unified_semantic_ast.rs::StructuredSemanticValueParser`** (hand-rolled) — this is the **runtime path for grammar directive payloads** (the `{ … }` after `@directive:` inside any grammar `.ebnf` file). It is invoked via `UnifiedSemanticAST::parse_bootstrap` → `parse_structured_payload` whenever the AST pipeline reads a grammar's annotations during parser generation.

Changes to "what `$<ref>` accepts" must therefore touch **both** surfaces in lockstep so the language definition (the EBNF) stays consistent with the runtime that actually parses payloads from real grammars. The `SV-EXH-PROOF.3.3.4.a.1` slice surfaced this distinction — initial regex-only edits to `semantic_annotation.ebnf` were no-ops for grammar directive payloads until the hand-rolled `parse_rule_reference` was also extended. Full bootstrap-mode details: see `docs/BOOTSTRAP_MODE_SPECIFICATION.md`.

#### Rule-span value comparison: `value_compare` (RULE-SPAN-VALUE-CONSTRAINT)

Most `@predicate` built-ins read the semantic store. `value_compare` instead gates a rule on a
**comparison between two of the rule's own resolved captures** — `@predicate: { name: value_compare,
args: [$lhs, <op>, $rhs], phase: post }`, where `<op>` is a word-form operator (`lt`/`le`/`gt`/`ge`/
`eq`/`ne`). The comparison is value-oriented (numeric for all ops when both operands parse as
integers — `05` equals `5`, `05 < 4` is false — deterministic lexical/textual fallback otherwise),
so a grammar can declaratively own an accept/reject rule that previously required out-of-band Rust
(the motivating case is a counted-quantifier `{min,max}` order rule like PCRE2's `{5,4}` reject).

This is a **rule-span** constraint (it spans two *different* captures), categorically distinct from
the **atom-scoped** value guards `@range`/`@len`/`@enum`/`@regex`, each of which constrains a *single*
atom's matched text against a constant. It is parser-agnostic and enabled for every grammar — a
grammar opts in simply by writing the predicate — and evaluates byte-identically in the generated
parser and the parse-harness interpreter (both call the shared semantic runtime). Full semantics: the
*Value-comparison predicates* section of [The Semantic Store](semantic-store.md).

A sibling built-in `value_compare_codepoint` takes the same shape but compares its operands by their
decoded **Unicode code point** — each operand is decoded as a single character literal (a bare scalar
or a standard C/Perl char escape: `\xHH`/`\x{H..}`, `\o{O..}`/`\NNN`, `\cX`, `\a \b \e \f \n \r \t`,
or `\X`) before comparing — for constraints like a character-class range order (`[z-a]`, `[\x{100}-z]`)
where a textual comparison would mis-order an escape spelling against a bare character. See the same
Semantic Store section.

## Semantic Seeds, Linters, And Front-End Workbenches

The next major widening for semantic annotations is not "more random annotation flexibility." It is a disciplined semantic-seed layer that downstream tools can trust.

The intended model is:

- the grammar emits local semantic seeds,
- the parser preserves source fidelity and provenance,
- later attribution passes compute broader meaning such as binding, typing, and flow,
- and downstream rule engines consume that attributed model rather than guessing from raw parse trees.

That matters first for HDL signoff-style consumers, but it is not an HDL-only idea. If PGEN lands the right semantic-seed, provenance, and export infrastructure, the same platform work should help:

- linters,
- compiler front-ends,
- elaborators,
- and other downstream semantic tools built on PGEN-backed grammars.

The detailed planning surfaces for those adjacent lanes now live in:

- `docs/reference/PGEN_LINTER_ENABLEMENT_ROADMAP.md`
- `docs/reference/PGEN_COMPILER_ELABORATOR_ENABLEMENT_ROADMAP.md`

## Why They Matter

Without annotations, grammar-driven generation can still produce parsers. With annotations, PGEN can also control:

- AST shape,
- transformation behavior,
- steering metadata,
- downstream usability of generated parsers.

That is why annotation grammars are core platform surfaces, not optional extras.

## Bootstrap Reality

Annotations also sit at the center of one of PGEN's historic bootstrapping constraints.

Because annotation parsers are needed by the generation pipeline itself, PGEN carries bootstrap-safe annotation grammar contracts so those parsers can be generated without circular dependency on themselves.

This is why the docs distinguish between:

- bootstrap-safe built-in annotation grammars,
- full main annotation grammars,
- generated parser steady-state behavior.

## Left recursion is folded back into *your* shape

You never write around PGEN's left-recursion elimination, and you never see it in the AST. Write the
standard's own binary production:

```ebnf
expr := add | sub | term
add  := expr "+" term  -> {type: "add", lhs: $1, rhs: $3}
sub  := expr "-" term  -> {type: "sub", lhs: $1, rhs: $3}
term := "n"            -> {type: "num"}
```

PGEN rewrites `expr` internally into `expr_lr_base (expr_lr_suffix)*` — and then **folds the result
back through your annotations**, left-associatively, so `n+n-n` parses to exactly what you declared:

```json
{ "type": "sub",
  "lhs": {"type": "add", "lhs": {"type": "num"}, "rhs": {"type": "num"}},
  "rhs": {"type": "num"} }
```

`$1` in a left-recursive alternative is the accumulated left operand; the remaining `$N` are that
alternative's own positions. An LR-eliminated rule is **indistinguishable from a hand-written one**,
which is the whole point: the responsibility boundary says the engine absorbs what is common to all
EBNFs, and left recursion is a property of *parsing*, not of your language.

**Honest bounds.** A left-recursive alternative's annotation may not use `$text`/`$0`, a quantified
extraction (`$N::first`), or an out-of-range positional — those cannot be replayed over a folded
chain, so PGEN **refuses them at generation time**, naming the rule and the construct, rather than
emitting an AST that does not match your declaration.

> ⛔ Historical note worth keeping (`ENGINE-UNIVERSAL-SERVICES.8`): before this fold existed, an
> LR-eliminated rule published the eliminator's own record —
> `{initial, suffixes, type: "_pgen_lr_chain", wrapper_specs}` — instead of the declared shape, in
> every grammar, for the life of the feature. Every differential gate stayed green, because they
> assert the interpreter and the generated parser are byte-identical **to each other**, and both
> leaked identically. *Two implementations agreeing is not evidence either one is right.* The gate
> that closes it is negative space: a value carrying an engine-reserved `_pgen_` discriminator now
> fails the annotation-shape gate outright.

## Proof Expectations

Annotation support is not considered real just because syntax exists. It is expected to have:

- validator coverage,
- shared/built-in suite coverage,
- round-trip or comparable contract evidence,
- maintained aggregate gates,
- an **annotation-vs-emitted-AST** check: the AST a parser returns is verified against what the
  grammar declared, not merely against a second implementation of the same engine.

## Spread Operators: `*` and `**`

Return annotations support two spread variants for unpacking multi-element values into an array literal.

### `*` — single-level spread

```ebnf
items := first rest* -> [$1, $2*]
# If `rest` matched [a, b, c], result is [first, a, b, c].
```

`$N*` iterates the Sequence/Quantified bound to `$N` and pushes each child node into the parent array. Each pushed node retains its own `content` — no recursion. This is the right operator for "flatten one level of repetition into a sibling array."

### `**` — flatten-spread (recursive-spread)

```ebnf
concatenation = piece+ -> [$1**]

piece = piece_quoted_run_quantified -> $1
      | atom quantifier? -> {type: "piece", atom: $1, quantifier: $2}

piece_quoted_run_quantified
   = "\\Q" quoted_run_inner_piece* quoted_literal_char "\\E" quantifier
   -> [$2**, {type: "piece", atom: $3, quantifier: $5}]
```

`$N**` is like `$N*`, but for each pushed child whose `content` is itself a `Sequence`/`Quantified`, it unwraps **one level** and pushes that wrapper's children inline. This is the operator to reach for when a child rule may produce *either* a single value *or* an array of values that should appear flat under the parent's accumulator.

The `regex.ebnf` example above is the canonical motivating case (PGEN-RGX-0074): the `piece` rule has two branches, one returning a single piece object (`{type:"piece", ...}`) and one returning an array of piece objects (the multi-char `\Q...\E quantifier` case). At the parent level, `concatenation = piece+ -> [$1**]` flattens both shapes uniformly so consumers see a flat array of pieces regardless of which `piece` branch matched.

### When to pick which

- **Use `*`** when the spread base's iterations are themselves your final array elements (the typical case — e.g. `(',' item)*` extracted-and-spread).
- **Use `**`** only when one of the iterations may itself wrap another array of items that need to flatten. Don't reach for it preemptively — `*` is the correct default and covers most patterns.

### Bootstrap caveat

The hand-written bootstrap parser (`UnifiedReturnAST::parse_bootstrap` in `rust/src/ast_pipeline/unified_return_ast.rs`) historically parses `$N**` as nested `Spread(Spread($N))` — semantically different from `FlattenSpread`. Bootstrap-chain grammars (`return_annotation.ebnf`, `semantic_annotation.ebnf`, `builtin_*.ebnf`) do not use `**`, so this divergence is benign. Tooling that calls `parse_bootstrap` on a `**`-using annotation (e.g. `auto_return_annotation_shape_gate`) maps to `ShapeKind::Passthrough` and skips shape verification gracefully. If a future bootstrap grammar needs `**`, the bootstrap parser will need to be aligned in a separate slice.

### The `null` keyword and the loud-refusal guard (RGX-0078.5.i.1.t2)

Two hardenings close the bootstrap-drift class exposed by the RGX-0078.5.i.1.t1 incident (a
feature-gated regen silently degraded annotation `null` literals to the *string* `"null"` in the
emitted parser):

- **`null` lowers faithfully.** `parse_bootstrap` now recognizes the bare keyword `null` as
  `NullLiteral` → `serde_json::Value::Null`, exactly like the canonical generated-parser path
  (`null_literal := 'null' -> {type: "null"}` in `return_annotation.ebnf`). Previously it fell into
  the identifier branch and serialized as the string `"null"`. Identifiers merely prefixed by the
  keyword (`nullable`, `null_value`, …) still parse as identifiers.
- **The silent fallback is refused loudly.** A pipeline running in **non-bootstrap** mode but built
  **without** `--features generated_parsers` used to fall back to the hand-rolled bootstrap
  annotation parsers with only a debug-gated warning — the exact flow that produced the drift. It
  now **hard-errors** (`REFUSED: … annotation … needs the generated annotation backend …`) for both
  the return and semantic annotation lanes, unless `PGEN_ALLOW_BOOTSTRAP_ANNOTATION_FALLBACK=1`
  explicitly opts in (the legitimate cold-bootstrap / chicken-and-egg recovery flow). The opt-in
  prints a once-per-process, verbosity-independent banner declaring the emitted artifacts
  **NON-CANONICAL** until re-derived via `make -C rust focus_<grammar>` and verified by
  `make -C rust parse_harness_equivalence_gate`. The canonical flows are unaffected: `make focus_*`
  builds the pipeline with `generated_parsers`, the annotation parsers regenerate under
  `--bootstrap-mode` (explicitly licensed), and the `ebnf_dual_run`-only frontend binary only
  performs standalone raw-AST export, which parses no annotations.

## Implicit `-> $1` default — what it does and what it doesn't

When a rule body is a **single Atom** (one terminal, one regex, one rule-reference) or a **single-element Sequence**, and the author has not declared a return annotation, the codegen synthesizes an implicit `-> $1` so the matched value flows through cleanly. This is what lets `boolean_literal := 'true' | 'false'` produce a clean string output without forcing per-branch `-> $1` everywhere.

The implicit default does **not** fire on:

- **Multi-element Sequence bodies** (e.g. `'(' expression ')'`). Picking which `$N` to surface would be an arbitrary author decision; require an explicit declaration.
- **Quantified bodies** (`X+`, `X*`, `X?`). The natural reading of `$1` here is "the whole capture group" — i.e. passthrough — and that is just what no-transform produces. The earlier (now-fixed) shape that synthesized `-> $1` on Quantified bodies emitted an `elements[0].content.clone()` extraction that silently dropped every match past the first; the regression is documented in the codegen-tightening tracker entry on 2026-04-30. Authors who want a different shape on a Quantified body declare it explicitly (`concatenation = piece+ -> [$1*]`).
- **Or-bodies as a whole.** Each Or branch is judged independently against the same rule (single Atom → implicit; single-element Sequence → implicit; multi-element Sequence → no synthetic; Quantified → no synthetic).

Synthetic defaults are codegen-only — they never appear in the inventory artifact, and the grammar-author-written annotations remain the visible declared surface.

## Parens-grouped Or with trailing annotation — broadcast

Annotation authors can write a single trailing return annotation on a parens-grouped Or and have it apply to **every** alternative inside the group:

```ebnf
RULE = ( A | B | C ) -> ann
```

is semantically equivalent to:

```ebnf
RULE = A -> ann
     | B -> ann
     | C -> ann
```

The trailing form is preferred when every branch produces the same shape (e.g. all branches use `-> $1` for positional passthrough, or all branches lift sub-rule results identically).

Per PEG/EBNF precedence rules, this **only** fires when the annotation immediately follows a `group_close`. For un-grouped alternations:

```ebnf
RULE = A | B -> ann
```

`-> ann` binds to `B` alone (the last alternative), as before — author-visible per-branch precedence is unchanged.

Bug history: pre-task-#38, parens-grouped-Or trailing annotations only applied to branch 0 inside the group; branches 1+ silently fell through to raw passthrough. Empirically caught by the `string_literal := ('"' ... '"' | "'" ... "'") -> {type:"string", value:$2}` rule in `grammars/return_annotation.ebnf` — single-quoted strings produced raw `Sequence` while double-quoted produced typed `Json`. The fix is in `extract_rule_annotations` (`rust/src/ast_pipeline/mod.rs`) and the cross-checker `extract_declared_annotations_from_json` (`rust/src/ast_shape_contract.rs`); both now track the branch range of the just-closed group via a stack and broadcast the trailing annotation to that range. The disambiguation cases (`(A|B) | C -> ann` → `ann` on `C` only; `A | (B|C) -> ann` → `ann` on `B` and `C`; nested groups) all behave per the principle "the annotation applies to the alternatives inside the just-closed group ONLY when it directly follows the group's close."

A 2026-05-14 refinement (the inner→outer branch-index remap, added so annotations on groups that are *sub-parts* of a sequence — `id ( a | b )* -> ann`, `( a | b )? id -> ann` — stop being silently dropped) accidentally **regressed the broadcast for whole-body groups**: in `RULE = ( A | B ) -> ann` the remap collapsed both branches' annotations onto branch 0, reintroducing the exact pre-#38 defect (single-quoted `string_literal` shipped as raw `Sequence` again). Fixed on 2026-06-10 (`BRANCH-BROADCAST-FIX.2`): the engine now distinguishes the **whole-body-group** shape — the rule body is exactly one top-level parens group, so the group's alternatives *are* the rule's runtime branches and the annotation indices stay group-local — from groups inside a larger sequence, which keep the 2026-05-14 outer remap. The discriminator is purely structural (the first syntax token is a `group_open` whose matching `group_close` is the last syntax token) and is shared by the pipeline and the cross-checker. The measured blast radius of the correction is confined to the two annotation grammars: `return_annotation` (`string_literal` branch 1 typed again) and `semantic_annotation` (43 whole-body-group branch annotations — e.g. `annotation_name`, `annotation_value`, `boolean_literal` — now correctly broadcast); every other shipped grammar regenerates semantically unchanged.

A companion defect was fixed the same day (`BRANCH-BROADCAST-FIX.3`): a **branch-level `$text`** (MatchedText) inside a *multi-branch* rule used to return the **empty string** — the generated tournament arm rolled the parser position back to the rule start before evaluating the branch's transform, so the `$text` span sliced `start..start`. The branch transform is now evaluated before that rollback, so branch-level `$text` returns the exact matched span. `$text` is the only transform form that reads the parser position; object/array/property transforms read the captured content and were never affected.

## Phase 2: Eliminate Stringification Roundtrips In Return-Annotation Transforms (retargeted)

### Earlier framing was wrong

An earlier framing said return + semantic annotations had drifted to a "post-parse transform" applied by `UnifiedReturnAST::parse_generated_return_annotation` walking a generic `ParseNode` tree at parse time, and that Phase 2 had to "restore inline application." A direct read of the codebase shows that framing is wrong:

- Return annotations are already applied inline at runtime. [rust/src/ast_pipeline/ast_based_generator.rs](../../../rust/src/ast_pipeline/ast_based_generator.rs) emits `result = #transform;` directly inside each rule's parse function whenever the rule carries a return annotation, with the transform tokens produced by [rust/src/ast_pipeline/ast_return_transform.rs](../../../rust/src/ast_pipeline/ast_return_transform.rs).
- `UnifiedReturnAST::parse_generated_return_annotation` is a build-time parser of annotation source text (e.g. `-> $1.foo`); it runs during PGEN code generation, not during downstream input parsing.
- Semantic annotations already use a typed structured carrier (`UnifiedSemanticValue` / `SemanticRuntimeValue`) — they do not carry an analogous problem.

The earlier framing also claimed PGEN-RGX-0073 closure depended on "moving annotations inline for the regex grammar." That claim was retracted after a direct check: [generated/regex_parser.rs](../../../generated/regex_parser.rs) currently has zero hits for `json_obj`, `serde_json::to_string`, or `serde_json::json!(`, and the `parse_regex` / `parse_piece` functions emit raw `ParseContent::Quantified(...)` / `ParseContent::Sequence(...)` with no `result = #transform` step. The regex grammar's two object-literal annotations (`-> {type: "regex", pattern: $1}` and `-> {type: "piece", atom: $1, quantifier: $2}`) are silently dropped at codegen for that grammar today. Whatever the dominant cost in the regex parser is, it is not a stringification roundtrip — that code is not present in the regex parser at all.

### What is actually wrong

The real defect is in how return-annotation object literals and property/array access are carried at runtime in the generated parsers that *do* emit transforms (notably [generated/return_annotation_parser.rs](../../../generated/return_annotation_parser.rs)):

- [`generate_object_transform`](../../../rust/src/ast_pipeline/ast_return_transform.rs) builds a `serde_json::Value` then `serde_json::to_string`s it and wraps the resulting `String` in `ParseContent::TransformedTerminal(String)`. From that point on the shaped value lives as JSON-encoded text inside a string variant.
- [`generate_property_access`](../../../rust/src/ast_pipeline/ast_return_transform.rs) deserialises that string back into `serde_json::Value`, looks up a property, and re-stringifies before wrapping again. Each property access pays serialise → parse → serialise.
- [`generate_array_transform`](../../../rust/src/ast_pipeline/ast_return_transform.rs) builds a `ParseContent::Sequence(Vec<ParseNode>)` with synthetic `element_N` rule names and zero spans, which is a different shape than the JSON-string carrier and does not compose cleanly with property access.
- `parse_content_to_string` falls back to `format!("{:?}", other)` (Debug formatter) for any non-trivial `ParseContent`, so structured shapes degrade silently into Rust Debug strings rather than failing visibly.

That serialise/parse/serialise roundtrip — and the Debug-format fallback — is the "stringification nonsense" Phase 2 retargets to remove.

### What Phase 2 now does

Phase 2 introduces a typed structured carrier in `ParseContent` so return-annotation transforms operate on values, not on serialised strings.

1. Extend `ParseContent` with a typed structured variant (e.g. `Json(serde_json::Value)`).
2. `generate_object_transform` builds `serde_json::Value::Object(...)` directly and wraps it as the new variant. No `to_string`.
3. `generate_array_transform` builds `serde_json::Value::Array(...)` and wraps it as the new variant. The synthetic-`ParseNode` array shape is no longer needed for array literals (it remains for sequence captures the grammar already produces).
4. `generate_property_access` and `generate_array_access` operate in place on the typed value (`value.get(prop)` / `value.get(idx)`). No `from_str`, no `to_string`.
5. `parse_content_to_string` is rewritten to handle the typed variant explicitly, and the `format!("{:?}", other)` Debug fallback is removed in favour of structured handling.
6. `parse_full_<entry>_typed` (M1's seam) returns the typed value directly rather than `serde_json::to_value(&node)`-wrapping a string-encoded tree.

Semantic annotations are out of scope of this work — they already use a typed carrier and are not affected.

### What this phase is not

- It is not a "move post-parse to inline" phase. Inline application is already in place.
- It is not on the critical path for PGEN-RGX-0073 closure as currently understood. The regex parser does not carry the stringify roundtrip today, so removing it cannot directly speed regex parsing. It is, however, a precondition for any future typed-shape API on regex, because turning the dropped annotations on without first fixing the carrier would just import the stringify cost into regex.
- It is not a `serde_json::Value` lock-in for the public API. `parse_full_<entry>_typed` keeps its `ParseResult<serde_json::Value>` signature; only the internal carrier changes.

### Separate defect surfaced during this investigation

The regex grammar declares object-literal return annotations that never reach the generated regex parser. That is a silent codegen drop, separate from the stringification work, and it is not closed by Phase 2 alone. It is tracked as a follow-up to investigate after the typed carrier lands.

### Commit cadence

Phase 2 lands in two commits:

1. Documentation retarget — replaces the wrong "post-parse" framing across the book chapter, live tracker, and continuity docs. No code or test changes.
2. Code change — introduces the typed structured carrier, rewrites the affected codegen helpers, regenerates the affected tracked parsers, and adds a focused differential test that asserts byte-identical wire-JSON output for the existing return + semantic annotation contract corpora before and after the change.

The earlier M1 commit (`4450b93`) remains useful: the typed-entry-skeleton flag (originally `--inline-annotations`, renamed in slice 4 to `--emit-typed-entry-skeleton` for honesty about its scope) and the `parse_full_<entry>_typed` skeleton are the right seam for surfacing the typed value through the public API; only the Phase 2 narrative attached to that commit was wrong.

## Primary Source Docs

- `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md`
- `docs/reference/PGEN_LINTER_ENABLEMENT_ROADMAP.md`
- `docs/reference/PGEN_COMPILER_ELABORATOR_ENABLEMENT_ROADMAP.md`
- `docs/RETURN_ANNOTATIONS_REFERENCE.md`
- `docs/contracts/PGEN_RETURN_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`
- `docs/contracts/PGEN_SEMANTIC_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`
- `README.md`
