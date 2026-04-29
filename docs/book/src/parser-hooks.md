# Parser Hooks

This chapter explains how PGEN extends the Rust AST pipeline's codegen with **parser-specific behavior** while keeping the pipeline itself **100% parser-agnostic**.

## The non-negotiable rule

> The Rust AST pipeline (`rust/src/ast_pipeline/`) is parser-agnostic.
> Code that reasons about a specific grammar — "for grammar X, do Y"
> — does not live there. PGEN-RGX-0073 must not be fixed in a way
> that risks breaking SV / VHDL or any other parser.

The parser-hook mechanism is how PGEN honors that rule. When a particular grammar needs codegen behavior other grammars do not, the parser-specific code lives **outside** the pipeline (in `rust/src/parser_hooks/<grammar>.rs`) and registers itself with the pipeline through a generic dispatch surface.

## Architecture at a glance

```text
                                       binary boundary
                                              │
                                              ▼
                            ┌──────────────────────────────┐
                            │ build a ParserHookRegistry   │
                            │ register parser-specific     │
                            │ handlers (e.g. RegexParser-  │
                            │ Hooks) and hand the registry │
                            │ to the pipeline              │
                            └──────────────┬───────────────┘
                                           │
                                           ▼
┌─────────────────────────────────────────────────────────────────────┐
│ rust/src/ast_pipeline/  (PARSER-AGNOSTIC — no grammar names allowed)│
│                                                                     │
│   ParserHooks trait        ParserHookRegistry                       │
│   ParserImplContext        AstBasedGenerator                        │
│        ▲                                                            │
│        │ at codegen extension points, the pipeline asks the         │
│        │ registry: "is there a handler for the grammar named        │
│        │ <ebnf_grammar_name>?"  If yes, call it; if no, default.    │
│        └──────────────────────────────────────────┐                 │
└──────────────────────────────────────────────────│─────────────────┘
                                                   │
                                                   ▼
                            ┌──────────────────────────────┐
                            │ rust/src/parser_hooks/       │
                            │  ├── mod.rs                  │
                            │  └── regex.rs   ← regex-     │
                            │                   specific   │
                            │                   handler    │
                            │                              │
                            │ parser-specific code lives   │
                            │ HERE; never inside           │
                            │ ast_pipeline/                │
                            └──────────────────────────────┘
```

The pipeline never names a specific grammar in its code. It forwards the runtime grammar name (read from the EBNF source) to the registry's lookup. When no handler is registered, the pipeline emits exactly what it has always emitted — every tracked parser stays byte-identical to its baseline.

## The contract

The pipeline-side abstraction lives in [`rust/src/ast_pipeline/parser_hooks.rs`][parser-hooks-rs]:

```rust
pub trait ParserHooks: Send + Sync {
    fn grammar_name(&self) -> &'static str;

    fn extend_parser_impl(
        &self,
        _ctx: &ParserImplContext<'_>,
    ) -> Option<TokenStream> {
        None
    }
}

pub struct ParserHookRegistry { /* HashMap<String, Box<dyn ParserHooks>> */ }

pub struct ParserImplContext<'a> {
    pub grammar_name: &'a str,
    pub parser_name: &'a Ident,
    pub grammar_tree: &'a HashMap<String, ASTNode>,
    pub rule_order: &'a [String],
    pub entry_rule: &'a str,
    pub annotations: Option<&'a Annotations>,
    pub filename: &'a str,
}
```

Three properties of the trait that make this contract robust:

1. **Default no-op implementations.** Adding a new hook phase in a future version of the trait does not break existing handlers; they inherit the default `None`. New handlers opt into whichever phases they care about.
2. **Immutable context.** Handlers receive immutable references to the data they need. They cannot mutate pipeline state, only return additional `TokenStream` for the pipeline to splice in.
3. **Lookup keyed on EBNF grammar name.** A handler claims `"regex"` (or whatever grammar stem matches its `<grammar>.ebnf` file) via `grammar_name()`. Mismatches return `None` from `ParserHookRegistry::get`, and the pipeline's default behavior runs unchanged for that grammar.

## Where the pipeline calls hooks

Today the pipeline has one extension point — the parser impl block — and the trait has one corresponding method. More extension points will be added as concrete needs surface (each must be parser-agnostic in design and have a default no-op so existing handlers don't break).

### `extend_parser_impl` — append items to the parser's impl block

Called inside [`AstBasedGenerator::generate_parser`][generator-impl] AFTER the legacy parser impl block has been generated. The pipeline checks:

```rust
let extension_impl = match (
    &self.parser_hook_registry,
    self.ebnf_grammar_name.as_deref(),
) {
    (Some(registry), Some(ebnf_name)) => {
        if let Some(hooks) = registry.get(ebnf_name) {
            let ctx = ParserImplContext { /* ... */ };
            hooks
                .extend_parser_impl(&ctx)
                .unwrap_or_else(TokenStream::new)
        } else {
            TokenStream::new()
        }
    }
    _ => TokenStream::new(),
};
```

The returned `TokenStream` is appended to the parser source as additional impl items. Typical use: emit parallel public methods (e.g. an alternate parse entry-point that returns `serde_json::Value` instead of `ParseNode`), helper methods used only by handler-emitted code, or anything else that fits as additional items on the same parser type.

The pipeline never names a grammar. It asks `registry.get(ebnf_name)` where `ebnf_name` is the snake_case stem the binary set on `AstBasedGenerator::ebnf_grammar_name`.

## How to write a hook

Suppose you have a grammar `foolang.ebnf` and want to extend its generated parser with extra methods.

### Step 1 — add a sibling submodule

Create `rust/src/parser_hooks/foolang.rs`. Do **not** put parser-specific code anywhere inside `rust/src/ast_pipeline/`.

Add the module declaration to [`rust/src/parser_hooks/mod.rs`][parser-hooks-mod]:

```rust
pub mod foolang;
```

### Step 2 — implement `ParserHooks` for your handler

```rust
use crate::ast_pipeline::{ParserHooks, ParserImplContext};
use proc_macro2::TokenStream;
use quote::quote;

pub struct FoolangParserHooks;

impl ParserHooks for FoolangParserHooks {
    fn grammar_name(&self) -> &'static str {
        // Snake_case stem of the EBNF source. The registry looks up
        // handlers by this exact string against `ebnf_grammar_name` set
        // on `AstBasedGenerator`. Convention: bare grammar stem,
        // lowercased — `foolang.ebnf` → `"foolang"`.
        "foolang"
    }

    fn extend_parser_impl(
        &self,
        ctx: &ParserImplContext<'_>,
    ) -> Option<TokenStream> {
        let parser_name = ctx.parser_name;
        Some(quote! {
            impl<'input> #parser_name<'input> {
                pub fn parser_hello(&self) -> &'static str {
                    "hello from the foolang hook"
                }
            }
        })
    }
}
```

Constraints on what your hook emits:

- **Preserve semantic side-effects.** Any typed parse entry-point you emit must invoke the parser methods that fire `with_semantic_runtime_rule_transaction`, `memoized_call`, recursion-guard checks, predicate evaluation, fact emission, and any other runtime-state interactions. The simplest safe pattern is to delegate to the existing `parse_<rule>` method and post-process its result.
- **Don't collide with the pipeline's emit.** The pipeline already emits `parse_<rule>`, `parse_full_<entry>`, helper methods, and (when `--inline-annotations` is set) the M1 skeleton's `parse_full_<entry>_typed`. Pick names that don't conflict.
- **Be deterministic.** Iterate over `ctx.rule_order` (a slice with deterministic order) rather than `ctx.grammar_tree.keys()` (HashMap iteration order is non-deterministic). Determinism makes the generated parser file reproducible across builds.

### Step 3 — register at the binary boundary

`rust/src/main.rs` (or any binary that drives the pipeline) constructs the registry and hands it to the pipeline:

```rust
use pgen::ast_pipeline::ParserHookRegistry;

let mut registry = ParserHookRegistry::new();
registry.register(Box::new(pgen::parser_hooks::foolang::FoolangParserHooks));

let parser_code = pgen::ast_pipeline::ast_generator_direct::generate_parser_ast_based_with_hooks(
    grammar_name,
    grammar_tree,
    rule_order,
    annotations,
    filename,
    inline_annotations,
    Some(registry),
)?;
```

When you don't want the hook to fire (e.g. default builds), just pass `None` as the registry. The pipeline's default emit path runs unchanged.

The default `pgen_ast` binary (`rust/src/main.rs`) registers the `regex` hook only when `--inline-annotations` is set, so default `make regex_parser` continues to produce a byte-identical legacy parser. Other downstream binaries are free to wire hook registration however suits their workflow.

### Step 4 — add a differential gate (mandatory for typed-emit hooks)

If your hook emits a parallel typed entry-point that returns `serde_json::Value`, add a maintained gate that proves byte-equivalent JSON between your typed entry and the legacy reference path. The regex hook's gate at [`regex_typed_differential_gate`][regex-gate] is the template; copy its shape:

1. New binary `rust/src/bin/<grammar>_typed_differential_gate.rs` that runs both paths on a corpus and compares.
2. New Cargo feature gating the binary (so default builds don't try to reference methods that only exist when the hook was registered during regen).
3. New maintained `make` target wrapping regen + build + run + restore-tracked-parser-to-baseline. The maintained target is the public surface; users don't manually wire the steps.

The differential gate is the regression-lock that lets future optimization slices replace the hook's typed-body delegation with shape-typed emit while staying correctness-honest.

## Verification properties

Three properties any pipeline change must preserve:

1. **Default regen is byte-identical.** Regenerating a tracked parser without registering any hook for it must produce the same bytes as before the pipeline change. The `make regex_parser` baseline (SHA256 `88d3e04fe1ffde36b3056debcd25ca450167d203a4b071aaeb2f87dffcfc7d07` at the time of writing) is a quick load-bearing check.
2. **Every parser-family parser unaffected unless its grammar has a registered handler.** SV / VHDL / annotation / RTL parsers stay byte-unchanged because no handler is registered for those grammars. The registry's lookup-by-name is what guarantees this; it is not a property of the pipeline being "well-behaved" in some informal sense.
3. **Hook output passes the differential gate.** When a hook emits a parallel typed entry-point, its output must be byte-equivalent to the legacy reference path until the gate is explicitly updated alongside an annotation transform that changes the documented shape.

## Why this design

PGEN's perf and integration work routinely produces parser-specific changes: a grammar's typed AST shape, a particular rule's optimization opportunity, a corpus that's specific to one parser's bug history. Without an extensibility surface, that work either pollutes the pipeline (every grammar gets the change whether it makes sense or not, and the pipeline grows brittle) or sits as a fork outside the pipeline (the grammar drifts away from the rest of PGEN's machinery).

The hook surface gives PGEN a third option: parser-specific code lives outside the pipeline, registers itself when invoked, and runs through a strictly bounded interface. Adding a new parser-specific behavior never requires touching `rust/src/ast_pipeline/`. Removing it is equally easy: unregister the handler. Composing handlers across grammars is just registering more of them.

## Reference

- Pipeline-side trait, registry, and context: [`rust/src/ast_pipeline/parser_hooks.rs`][parser-hooks-rs].
- The example regex hook: [`rust/src/parser_hooks/regex.rs`][regex-hook].
- The parser-hooks module entry point: [`rust/src/parser_hooks/mod.rs`][parser-hooks-mod].
- Differential gate template: [`make -C rust regex_typed_differential_gate`][regex-gate].
- Generator integration site: [`AstBasedGenerator::generate_parser`][generator-impl] in `rust/src/ast_pipeline/ast_based_generator.rs`.

[parser-hooks-rs]: https://github.com/anthropics/pgen/blob/main/rust/src/ast_pipeline/parser_hooks.rs
[parser-hooks-mod]: https://github.com/anthropics/pgen/blob/main/rust/src/parser_hooks/mod.rs
[regex-hook]: https://github.com/anthropics/pgen/blob/main/rust/src/parser_hooks/regex.rs
[regex-gate]: https://github.com/anthropics/pgen/blob/main/rust/Makefile
[generator-impl]: https://github.com/anthropics/pgen/blob/main/rust/src/ast_pipeline/ast_based_generator.rs
