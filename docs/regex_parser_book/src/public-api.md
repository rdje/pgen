# Public API Surface

The regex parser exposes a small, stable Rust API. This chapter is the canonical reference for what downstream consumers should call.

## Library identity

| Item | Value |
|---|---|
| Crate | `pgen` (path: `rust/`) |
| Required Cargo feature | `generated_parsers` |
| Module path | `pgen::generated_parsers::regex` |
| Parser type | `RegexParser<'input>` |
| Entry method (default emit) | `parse_full_regex()` |
| Entry method (typed fast path, opt-in) | `parse_regex_typed()` |
| Schema version | `2` (typed-Json carrier era — see [Schema Versioning](schema-versioning.md)) |

## Top-level types

```rust
// The parser itself.
pub struct RegexParser<'input> { /* opaque */ }

impl<'input> RegexParser<'input> {
    /// Construct a parser over an input string with a logger.
    /// Pass `runtime_logger_box("generated.regex")` for the standard
    /// runtime logger, or any other type that implements `Logger`.
    pub fn new(input: &'input str, logger: Box<dyn Logger>) -> Self;

    /// Optional grammar profile selector. Default = `None` (uses
    /// `regex_default`).
    pub fn set_grammar_profile(&mut self, profile: Option<&str>);

    /// Standard parse entry. Returns the full `regex` rule's AST envelope.
    pub fn parse_full_regex(&mut self) -> ParseResult<ParseNode<'input>>;

    /// Optional typed fast-path entry. Bypasses `ParseNode` allocation
    /// and returns `serde_json::Value` directly. Available only when
    /// the parser was generated with `--enable-parser-hooks` (see Build
    /// Recipe). Output is byte-equivalent to
    /// `parse_full_regex()?.content.to_json_value()`.
    pub fn parse_regex_typed(&mut self) -> ParseResult<serde_json::Value>;
}
```

## ParseNode envelope

Every node the parser produces is wrapped in this envelope:

```rust
pub struct ParseNode<'input> {
    /// The grammar rule that produced this node. Static string,
    /// matches a rule name from grammars/regex.ebnf.
    pub rule_name: &'static str,

    /// The node's content. See ParseContent variants below.
    pub content: ParseContent<'input>,

    /// Source-byte span: [start, end) inclusive of start, exclusive
    /// of end. Indexes into the original input string.
    pub span: std::ops::Range<usize>,
}
```

The span is always populated for the OUTERMOST node. Inner nodes synthesised by the codegen (e.g. `element_0`, `element_1` synthetic-element wrappers, `quantified` synthetic wrappers) sometimes carry `0..0` spans because they were not materialised from a specific source range.

## ParseContent variants

```rust
pub enum ParseContent<'input> {
    /// Raw matched text. Borrowed from the input string.
    /// Used by terminal-matching rules (e.g. `"["`, `digit`).
    Terminal(&'input str),

    /// Owned string — typically the result of `@transform` semantic
    /// annotations that coerce a regex match to a different scalar
    /// representation. Less common in current output.
    TransformedTerminal(String),

    /// Typed structured carrier — `serde_json::Value`. Used by
    /// rules whose return annotation produces a typed shape
    /// (object literal, array literal, integer-coerced number,
    /// etc.). This is the most-common variant for annotated rules
    /// like `regex`, `piece`, `concatenation`, `quant_suffix`,
    /// `digits`. See [The Json Carrier](json-carrier.md).
    Json(serde_json::Value),

    /// A flat sequence of child nodes. Produced by:
    /// (a) unannotated multi-element rule bodies in regex.ebnf, OR
    /// (b) the `[$1**]` flatten-spread operator on annotated rules
    ///     like `concatenation = piece+ -> [$1**]`.
    Sequence(Vec<ParseNode<'input>>),

    /// A boxed inner node, used as a synthetic carrier for branch
    /// alternatives. Mostly internal to the codegen path; consumers
    /// rarely need to pattern-match on this directly because typed-
    /// shape rules unwrap it.
    Alternative(Box<ParseNode<'input>>),

    /// A repetition group — produced by `*` / `+` / `?` quantifiers
    /// in the grammar body. Carries the iteration count's marker
    /// string ("*", "+", "?") and the matched repetitions.
    Quantified(Vec<ParseNode<'input>>, &'static str),
}

impl<'input> ParseContent<'input> {
    /// Convert any ParseContent shape to serde_json::Value. This is
    /// what the typed-entry-point's byte-equivalence is verified
    /// against. Useful when you want to JSON-serialize a sub-tree.
    pub fn to_json_value(&self) -> serde_json::Value;
}
```

For the full mapping of which rules emit which variant **today**, see [Per-Rule Shape Reference](rules-top-level.md) onwards.

## ParseResult and error types

```rust
pub type ParseResult<T> = Result<T, ParseError>;

pub enum ParseError {
    InvalidSyntax {
        message: &'static str,
        position: usize,
    },
    UnexpectedToken {
        expected: &'static str,
        found: String,
        position: usize,
    },
    Backtrack {
        position: usize,
    },
    RecursionDepthExceeded {
        position: usize,
        depth: usize,
    },
    ContextualError {
        message: String,
        position: usize,
        rule_stack: Vec<&'static str>,
        input_context: String,
    },
}
```

Most consumer error-handling reduces to "did `parse_full_regex` return `Ok` or not." For diagnostic purposes, the `ContextualError` variant carries a `rule_stack` showing the chain of rules that were active when the error fired — useful for surfacing localized error messages.

## Embedding-API surface

For higher-level integration, PGEN exposes a stable embedding API at `pgen::embedding_api`. Downstream consumers should generally prefer this over calling `RegexParser::new` directly because it carries:

- Backend-availability discovery (`parser_embedding_api_contract().supports_regex_generated_backend`).
- Dedicated worker stack for deep-recursion patterns (see `REGEX-0055`/`0056`).
- Stable error-localization (`location.byte_offset`, `location.line`, `location.column`).
- Profile-selection (`regex_default` is the published profile).

```rust
use pgen::embedding_api::{parser_embedding_api_contract, ParserBackend};

let contract = parser_embedding_api_contract();
assert!(contract.supports_regex_generated_backend);
```

The full embedding-API surface is documented separately in `rust/docs/EMBEDDING_API_CONTRACT.md`.

## Legacy vs typed entry points

| Feature | `parse_full_regex()` | `parse_regex_typed()` |
|---|---|---|
| Return type | `ParseResult<ParseNode<'input>>` | `ParseResult<serde_json::Value>` |
| Allocation | Allocates ParseNode tree | Bypasses ParseNode allocation for annotated rules |
| Output equivalence | `node.content.to_json_value() == typed_value` (byte-equivalent) | same |
| Default emit availability | YES — default `make regex_parser` builds it | NO — requires `--enable-parser-hooks` regen |
| Use case | Standard consumers; need source spans / `rule_name` access | JSON-direct fast path; serialization-only consumers |

For consumers like RGX that walk the AST to build a richer downstream representation, `parse_full_regex()` is usually the right call because the ParseNode tree carries source spans (useful for error reporting back to the regex source author). The typed entry point is most useful when the consumer just wants the JSON.

## Stability guarantees

| Item | Guarantee |
|---|---|
| `RegexParser::new` signature | Stable across PGEN 1.1.x. Logger trait may evolve. |
| `parse_full_regex()` signature | Stable across PGEN 1.1.x. |
| `ParseNode` struct shape | Stable across PGEN 1.1.x — `rule_name`, `content`, `span`. |
| `ParseContent` variants | **NOT** stable in the strictest sense — new variants may be added (e.g. `Json` was added in 1.1.x). Consumers should pattern-match exhaustively or with a `_` arm to remain forward-compatible. |
| `rule_name` values | Tied to rule names in `grammars/regex.ebnf`. Renames or rule additions may happen across releases — see the per-release changelog. |
| AST shape per rule | Documented per release in the integration contract; this book is the cumulative-state view. |
| Schema version (1 → 2) | Published in the contract. Consumers should check the version when bumping PGEN. |
