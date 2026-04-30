# Glossary

Definitions for terms used throughout this book and in the surrounding PGEN documentation. Where a term has both a PGEN-specific meaning and a general regex/parsing meaning, the PGEN-specific one is given.

## Annotation language

The expression language used inside `-> ...` clauses in EBNF rules. Syntactic forms documented in `grammars/return_annotation.ebnf` and `grammars/builtin_return_annotation.ebnf`. Includes positional refs (`$1`, `$2`, ...), object literals, array literals, string/number/boolean literals, the `null` literal, the spread operator `*`, and the flatten-spread `**`.

## AST envelope

The recursive `ParseNode { rule_name, content, span }` structure. "Envelope" = the structural shell that wraps every parser output. Pre-1.1.30 the envelope was the entire AST; 1.1.30+ adds typed-Json content as an inner shape.

## Atom

A grammar-level concept and a rule name in `regex.ebnf`. The `atom` rule's right-hand side is a 25-way Or covering everything that can appear at a "single regex unit" position: a literal char, an escape, a character class, a group, an anchor, a backreference, etc. See [Atom Subtree](rules-atom.md).

## Atom-fallback

The behavior where `piece`'s branch 0 (`piece_quoted_run_quantified`) fails for degenerate inputs (zero or one quoted char) and the parser falls through to `piece`'s branch 1 (`atom quantifier?`), matching the entire `\Q...\E` block as a single `quoted_literal` atom. Documented in [\Q...\E Quoted Literals](examples-quoted-literal.md).

## Bootstrap mode

A build mode where the EBNF grammar parser (used to read other grammars) is itself built without a generated parser available. Activated by the `has_generated_ebnf_parser` cfg flag being absent (set in `build.rs` only when `generated/ebnf.rs` exists). Used in cold-clone builds. See [Build Recipe](build-recipe.md).

## Builtin return annotation parser

A hand-written parser in `rust/src/ast_pipeline/builtin_return_annotation_parser.rs` that handles the subset of the return-annotation language sufficient to bootstrap. Its accepted grammar is documented in `grammars/builtin_return_annotation.ebnf`. The full annotation language is parsed by the generated parser (built from `grammars/return_annotation.ebnf`) once available.

## Byte-equivalence

The invariant that `parse_full_regex(input).content.to_json_value() == parse_regex_typed(input)`. Maintained across every PGEN release. See [Schema Versioning](schema-versioning.md).

## Char class / character class

Both terms refer to `[abc]`-style regex syntax. The book uses "character class" in prose. The grammar rule is `char_class`. Inside character classes, sub-rules are prefixed with `class_` (e.g. `class_literal`, `class_range`).

## Class item

A single element within a character class body. Branches: `class_range`, `class_literal`, `class_escape`, `quoted_class_literal`, `posix_class`, `stray_class_end_quote`. See [Character Class Subtree](rules-char-class.md).

## Cold clone / cold-clone build

Building PGEN from a fresh `git clone`, where no `generated/*` files exist yet and no prior build state is available. The bootstrap recipe (`make regex_parser_bootstrap` or `make regex_parser_fresh`) handles this case. See [Build Recipe](build-recipe.md).

## Concatenation

A grammar rule and a regex concept. The `concatenation` rule produces a flat sequence of pieces — `concatenation = piece+ -> [$1**]`. The `**` flatten-spread is what produces the flat array (without it, the output would be a deeply-nested chain of Quantified wrappers).

## Counted quantifier

A quantifier of the form `{n}`, `{n,}`, `{,m}`, or `{n,m}`. Parsed by the `counted_quantifier` rule (`-> $3`, lifts the body's typed shape), with body parsed by `counted_quantifier_body` (4 explicit branches each emitting a typed `{min, max}` object — `min` is always integer, `max` is integer-or-null for the unbounded form). Both rules annotated in slices 3+4 of the typed-shape campaign.

## Discriminator

The field in a Json object whose value tells the consumer which kind of object this is. In the regex AST, the convention is `"type"`: `"regex"`, `"piece"`, etc. See [The Json Carrier](json-carrier.md).

## EBNF

Extended Backus-Naur Form. PGEN's grammar source language. The regex grammar lives at `grammars/regex.ebnf`. The grammar of EBNF itself lives at `grammars/ebnf.ebnf` (parsed by the generator-of-generators, called the EBNF frontend in this codebase).

## EBNF frontend

The Rust pipeline that reads a `.ebnf` file and produces an `IR` (intermediate representation) that downstream tools (like the AST pipeline / codegen) consume. Located in `rust/src/ebnf_frontend/`. Replaces the old `ebnf_to_json.pl` Perl frontend, which had limitations and is no longer used.

## Flatten-spread

The `**` operator in the return annotation language. Like `*` (spread), but unwraps one level of nested array structure. Used in `concatenation = piece+ -> [$1**]` to produce a flat piece array instead of a nested-Quantified shape. Added in 1.1.31.

## Greediness mode

For quantifiers, the suffix that determines match strategy: nothing = greedy (default), `?` = lazy (minimum match), `+` = possessive (no backtrack). The `quant_suffix` rule emits `[]` (greedy), `"lazy"`, or `"possessive"` accordingly.

## Json carrier

The `ParseContent::Json(serde_json::Value)` variant. Carries the typed shape produced by a return annotation. See [The Json Carrier](json-carrier.md).

## Lookaround

`(?=...)`, `(?!...)`, `(?<=...)`, `(?<!...)`, plus PCRE2's non-atomic and alpha-form variants. Zero-width assertions matched without consuming input. See [Groups and Alternations](examples-groups-alt.md).

## Or rule / alternation rule

A grammar rule whose right-hand side is `A | B | C ...`. The codegen wraps the matched alternative in `ParseContent::Alternative(boxed)`. Annotated Or rules with per-branch `-> ...` produce the branch-specific output (e.g. `quant_suffix` 2-branch Or both annotated to strings).

## ParseContent

The Rust enum carrying the inner content of a `ParseNode`. Six variants: `Terminal`, `TransformedTerminal`, `Json`, `Sequence`, `Alternative`, `Quantified`. See [ParseContent Variants](parse-content-variants.md).

## ParseNode

The Rust struct envelope: `{ rule_name, content, span }`. The unit of recursion in the AST. Top-level node is always `rule_name = "regex"` for regex parses.

## Parser release

A versioned release of the PGEN parser library. Tagged in git. Different from "schema version" — see [Schema Versioning](schema-versioning.md).

## PCRE2

Perl Compatible Regular Expressions, version 2. The reference implementation whose syntax the regex grammar targets. Authoritative documentation at `pcre2pattern(3)`.

## PEG

Parsing Expression Grammar. The parser semantic that PGEN implements. Key property: alternation is **ordered** — `A | B` tries A first; if A succeeds, B is never tried. This is unlike CFG/Yacc which may try all alternatives.

## Piece

A grammar rule and a regex concept. A `piece` is one regex unit: an atom optionally followed by a quantifier, OR (under PGEN-RGX-0074's new branch) a quoted-run member. Pieces are the elements of a `concatenation`.

## Probe

The `parseability_probe` binary. A debugging/dumping tool that exposes the parser surface from the command line. The book and contract use probes to capture canonical AST shapes.

## Quantifier

`*`, `+`, `?`, `{n}`, `{n,}`, `{,m}`, `{n,m}` plus suffix `?` (lazy) or `+` (possessive). Parsed by the `quantifier` rule.

## Quantified

A `ParseContent` variant: `Quantified(Vec<ParseNode>, &'static str)`. Holds the children of a `?`/`*`/`+` grammar quantifier and the marker `"?"`, `"*"`, `"+"`. Distinct from regex-level quantifier — Quantified is the codegen's output shape for grammar repetition.

## Quoted run

The PGEN-internal name for the inner contents of `\Q...\E`. The `piece_quoted_run_quantified` branch handles `\Q...\E quantifier`, splitting per-char so the quantifier binds to the last char only (per PCRE2 semantics).

## Recursive envelope

The pre-1.1.30 AST shape, where every rule produced a `Sequence` / `Alternative` / `Quantified` / `Terminal` envelope and consumers walked the structure recursively. Still present for unannotated rules; replaced by `Json` carrier for annotated rules. See [Migration](migration-from-recursive-envelope.md).

## Return annotation

The `-> ...` clause attached to an EBNF rule's right-hand side. Determines what the rule emits at runtime. Distinct from semantic annotations (`@xxx:`).

## RGX

A downstream consumer of the PGEN regex parser. RGX uses the regex parser's AST as its source-of-truth representation for compiling/evaluating regex patterns. This book's primary audience.

## Schema version

A version number tracking the AST output shape, distinct from the parser release version. Bumped on shape-affecting changes. See [Schema Versioning](schema-versioning.md).

## Semantic annotation

The `@xxx: ...` clause attached to an EBNF rule. Specifies semantic transforms (e.g. `@transform: str::parse::<usize>().unwrap_or(0)`) or runtime-validation predicates. Distinct from return annotations.

## Slice

A small, focused unit of work — typically one rule's annotation or one bug fix. The campaign for task #40 ("Annotate regex.ebnf for full AST usability") proceeds slice-by-slice, one rule per slice, with contract bumps per slice.

## Span

A `start..end` byte range in the original input. Every `ParseNode` carries one. Useful for source-location reporting; see [AST Envelope](ast-envelope.md) for caveats on synthetic-node spans.

## Spread operator

The `*` operator in the return annotation language: `[$1, $2*]` means "include $1, then spread $2's contents into the array." Distinct from `**` (flatten-spread) which unwraps an extra layer.

## Tier 1 / 2 / 3 stability

The three-tier classification of stability guarantees. Tier 1 = stable surface API; Tier 2 = annotated rule shapes; Tier 3 = unannotated rule shapes. See [Schema Versioning](schema-versioning.md).

## Typed parser entry

`crate::parse_regex_typed(input) -> Result<serde_json::Value, ParseError>`. Returns the typed JSON value directly, skipping the `ParseNode` envelope. Equivalent to `parse_full_regex(input).content.to_json_value()` but more direct.

## Un-annotated chain

A descent through multiple rules that all use implicit `-> $1` (single-positional pass-through). Produces nested Alternative-wrapped layers in the AST, e.g. `escape_unit → simple_escape → any_char → /char/` chains four wrappers around one terminal. Consumer descent recipe: keep unwrapping one-element arrays until reaching a non-array.

## UnifiedReturnAST

The Rust enum representing parsed return-annotation expressions, defined in `rust/src/ast_pipeline/unified_return_ast.rs`. Variants: PositionalRef, StringLiteral, NumberLiteral, BooleanLiteral, NullLiteral, Object, Array, Spread, FlattenSpread, PropertyAccess, ArrayAccess, QuantifiedExtraction, Identifier, Passthrough.
