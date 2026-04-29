//! Parser-agnostic extensibility surface for the Rust AST pipeline.
//!
//! # Standing rule
//!
//! The Rust AST pipeline must remain 100% parser-agnostic. Parser-specific
//! code does not live inside `rust/src/ast_pipeline/`. When a particular
//! grammar needs codegen behavior that other grammars do not, the
//! parser-specific code lives in a separate module (e.g.
//! `rust/src/parser_hooks/<grammar>.rs`) and registers itself with the
//! pipeline through the abstraction defined in this file.
//!
//! # Design
//!
//! - [`ParserHooks`] is a trait that parser-specific modules implement to
//!   participate in particular phases of pipeline codegen. Every method has
//!   a default no-op implementation, so adding a new hook phase is
//!   non-breaking for existing handlers.
//!
//! - [`ParserHookRegistry`] owns the set of registered handlers. The
//!   pipeline is given a registry and looks up handlers BY EBNF GRAMMAR
//!   NAME (e.g. `foolang.ebnf` → `foolang`). The pipeline NEVER names a
//!   specific grammar in code; it only asks the registry whether a handler
//!   exists for the grammar currently being processed.
//!
//! - When no handler is registered for a grammar (the default state for
//!   every tracked grammar today), the pipeline emits exactly what it
//!   emitted before this abstraction landed. This is the load-bearing
//!   property: SV / VHDL / regex / annotation grammars / RTL grammars all
//!   continue to compile and behave byte-identically until a handler is
//!   explicitly registered for them.
//!
//! # Where handlers live
//!
//! `rust/src/parser_hooks/<grammar>.rs` (sibling of `ast_pipeline/`).
//! Registration happens at the binary boundary (e.g. inside the binary's
//! `main` or wherever the pipeline is constructed). Never inside the
//! pipeline.
//!
//! # Why this design
//!
//! 1. **Parser-agnostic by construction.** The trait and registry don't
//!    name any grammar. Adding a new parser-specific behavior never
//!    requires touching `ast_pipeline/`.
//!
//! 2. **Latent risk to other parsers is zero.** A handler registered for
//!    grammar A cannot influence the codegen path for grammar B. The
//!    registry's `get` is keyed on grammar name; mismatches return `None`
//!    and the pipeline falls through to the default emit.
//!
//! 3. **Default no-op trait methods.** New pipeline phases that gain
//!    extension points don't break existing handlers — they simply opt
//!    out by inheriting the default. New handlers can opt into whichever
//!    phases they care about.
//!
//! 4. **Independent verification.** Regenerating a tracked parser without
//!    a registered handler must produce byte-identical output to before
//!    this abstraction landed. The byte-identity check is the contract
//!    that the abstraction has not introduced parser-specific behavior.

use crate::ast_pipeline::{ASTNode, Annotations};
use proc_macro2::TokenStream;
use std::collections::HashMap;
use syn::Ident;

/// Context passed to [`ParserHooks::extend_parser_impl`] when the pipeline
/// has finished generating the legacy parser impl block and is asking
/// registered handlers for any additional impl items to append.
///
/// The context fields are immutable references; handlers cannot mutate
/// pipeline state. They produce a `TokenStream` to be inserted alongside
/// the existing impl.
pub struct ParserImplContext<'a> {
    /// EBNF grammar name (e.g. `regex` for `regex.ebnf`). The pipeline
    /// uses this to look up handlers; handlers can also confirm the
    /// dispatch matches their advertised `grammar_name`.
    pub grammar_name: &'a str,
    /// Generated parser type name (e.g. `RegexParser`). Used to emit
    /// `impl<'input> #parser_name<'input> { … }` blocks.
    pub parser_name: &'a Ident,
    /// Full grammar tree, keyed by rule name. Handlers that need to
    /// inspect rule bodies (e.g. to emit per-rule typed methods) read
    /// from this map.
    pub grammar_tree: &'a HashMap<String, ASTNode>,
    /// Deterministic rule iteration order. Handlers should iterate via
    /// this slice instead of `grammar_tree.keys()` because HashMap
    /// iteration order is non-deterministic; using `rule_order`
    /// guarantees the emitted output is reproducible across builds.
    pub rule_order: &'a [String],
    /// Top-level entry rule (e.g. `regex` for the regex grammar).
    pub entry_rule: &'a str,
    /// Compiled annotations (return / semantic / branch / mid-sequence).
    /// `None` when no annotations were attached to the grammar tree.
    pub annotations: Option<&'a Annotations>,
    /// Source filename for error messages and tracing.
    pub filename: &'a str,
}

/// Trait implemented by parser-specific extension modules.
///
/// All methods have default no-op implementations; a handler only
/// overrides the phases it cares about.
///
/// **Implementations live OUTSIDE the AST pipeline** — typically under
/// `rust/src/parser_hooks/<grammar>.rs`. The pipeline only knows about
/// the trait, never about specific implementations.
pub trait ParserHooks: Send + Sync {
    /// EBNF grammar name this handler claims responsibility for. The
    /// registry uses this string as the key; the pipeline looks up
    /// handlers by the grammar name it derived from the EBNF source
    /// path.
    ///
    /// Convention: the bare grammar stem, lowercased — `regex.ebnf` →
    /// `"regex"`, `systemverilog.ebnf` → `"systemverilog"`.
    fn grammar_name(&self) -> &'static str;

    /// Called after the pipeline has produced the legacy parser impl
    /// block and is collecting any additional impl items to append.
    /// Returning `Some(tokens)` causes those tokens to be emitted
    /// inside an additional `impl<'input> #parser_name<'input> { … }`
    /// block immediately after the legacy one. Returning `None` (the
    /// default) leaves the parser unchanged.
    ///
    /// Use this hook to add parallel public methods (e.g. an alternate
    /// parse entry point that returns a different result type), helper
    /// methods used only by handler-emitted code, or anything else
    /// that fits as additional impl items on the same parser type.
    fn extend_parser_impl(&self, _ctx: &ParserImplContext<'_>) -> Option<TokenStream> {
        None
    }
}

/// Owns the set of registered parser hook handlers.
///
/// The pipeline is given a `&ParserHookRegistry` (or an `Option<&>`) and
/// queries it at extension points. Handlers are looked up by grammar
/// name; missing entries fall through to the pipeline's default behavior.
///
/// This type is parser-agnostic: it stores `Box<dyn ParserHooks>` values
/// and never inspects their content.
#[derive(Default)]
pub struct ParserHookRegistry {
    hooks: HashMap<String, Box<dyn ParserHooks>>,
}

impl ParserHookRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a handler. The handler's `grammar_name()` is used as the
    /// lookup key; registering twice for the same grammar replaces the
    /// previous handler.
    pub fn register(&mut self, hooks: Box<dyn ParserHooks>) {
        let key = hooks.grammar_name().to_string();
        self.hooks.insert(key, hooks);
    }

    /// Retrieve the handler registered for `grammar_name`, or `None` if
    /// no handler is registered.
    pub fn get(&self, grammar_name: &str) -> Option<&dyn ParserHooks> {
        self.hooks.get(grammar_name).map(|b| b.as_ref())
    }

    /// True when at least one handler is registered. The pipeline can
    /// use this to skip the registry-aware code path entirely when no
    /// handlers exist (the common case for binaries that don't opt
    /// into any extension).
    pub fn is_empty(&self) -> bool {
        self.hooks.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubHooks;
    impl ParserHooks for StubHooks {
        fn grammar_name(&self) -> &'static str {
            "stub"
        }
    }

    #[test]
    fn registry_empty_by_default() {
        let registry = ParserHookRegistry::new();
        assert!(registry.is_empty());
        assert!(registry.get("stub").is_none());
        assert!(registry.get("regex").is_none());
    }

    #[test]
    fn registry_returns_registered_handler_only_for_matching_grammar_name() {
        let mut registry = ParserHookRegistry::new();
        registry.register(Box::new(StubHooks));
        assert!(!registry.is_empty());

        let stub = registry.get("stub");
        assert!(stub.is_some());
        assert_eq!(stub.unwrap().grammar_name(), "stub");

        // The point of the registry: a handler registered for grammar
        // A cannot influence the lookup result for grammar B.
        assert!(registry.get("regex").is_none());
        assert!(registry.get("systemverilog").is_none());
        assert!(registry.get("vhdl").is_none());
    }

    #[test]
    fn default_extend_parser_impl_returns_none() {
        // Critical contract: a handler that does NOT override
        // extend_parser_impl must yield `None`, so the pipeline falls
        // through to its default emit. This is the safety net that
        // keeps unrelated parsers byte-identical when the abstraction
        // is added.
        let stub: Box<dyn ParserHooks> = Box::new(StubHooks);
        let parser_name = syn::parse_str::<Ident>("StubParser").unwrap();
        let grammar_tree: HashMap<String, ASTNode> = HashMap::new();
        let rule_order: Vec<String> = vec![];
        let ctx = ParserImplContext {
            grammar_name: "stub",
            parser_name: &parser_name,
            grammar_tree: &grammar_tree,
            rule_order: &rule_order,
            entry_rule: "root",
            annotations: None,
            filename: "stub.ebnf",
        };
        assert!(stub.extend_parser_impl(&ctx).is_none());
    }
}
