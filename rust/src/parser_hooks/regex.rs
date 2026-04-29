//! Regex grammar parser hook.
//!
//! Implements [`crate::ast_pipeline::ParserHooks`] for the `regex`
//! grammar. When registered with the pipeline's
//! [`crate::ast_pipeline::ParserHookRegistry`], this hook causes the
//! generated regex parser to gain per-rule `parse_<rule>_typed`
//! entry-point methods alongside the existing legacy `parse_<rule>`
//! methods.
//!
//! # Architectural contract
//!
//! - **The hook lives outside `rust/src/ast_pipeline/`.** Nothing in
//!   the pipeline knows the regex grammar exists; the pipeline only
//!   asks the registry whether a handler is registered for the
//!   grammar currently being processed.
//!
//! - **The hook does not break SV / VHDL / any other parser.** The
//!   registry is keyed on EBNF grammar name; this hook returns
//!   `"regex"` from `grammar_name()` and is never returned for
//!   lookups against other grammars. Any binary that registers this
//!   hook can still regenerate every other parser byte-identically.
//!
//! - **The hook preserves semantic side-effects.** The typed methods
//!   it emits delegate to the legacy `parse_<rule>` methods, which
//!   means `with_semantic_runtime_rule_transaction`, `memoized_call`,
//!   recursion-guard checks, predicate evaluation, fact emission, and
//!   all other runtime-state interactions fire exactly as they would
//!   on the legacy entry path. The typed method then converts the
//!   resulting `ParseNode`'s content via
//!   `ParseContent::to_json_value()`.
//!
//! - **The hook is byte-equivalent to the legacy + `to_json_value()`
//!   reference path by construction.** Because the typed body is
//!   `let node = self.parse_<rule>()?; Ok(node.content.to_json_value())`,
//!   the differential gate that compares hook output to the
//!   reference path passes trivially. Future optimization passes can
//!   replace specific rules' typed bodies with shape-typed emit that
//!   bypasses `ParseNode` allocation, but each replacement must:
//!   1. Continue to invoke `with_semantic_runtime_rule_transaction`
//!      and `memoized_call` so semantic side-effects fire.
//!   2. Produce byte-equivalent JSON to the reference path
//!      (verified by the differential gate before the optim lands).
//!
//! # Registration
//!
//! Construct and register at the binary boundary:
//!
//! ```ignore
//! use pgen::ast_pipeline::ParserHookRegistry;
//! use pgen::parser_hooks::regex::RegexParserHooks;
//!
//! let mut registry = ParserHookRegistry::new();
//! registry.register(Box::new(RegexParserHooks));
//! generator.parser_hook_registry = Some(registry);
//! ```
//!
//! # Why a passthrough body in this slice
//!
//! Slice 2 establishes the parser-hook architecture. Optimizing the
//! typed bodies further (replacing the delegation with shape-typed
//! emit) is a separate, follow-up slice that lands together with the
//! differential gate proving each optimization preserves byte-
//! equivalent JSON output. Architecture first; perf optimization
//! second.

use crate::ast_pipeline::{ParserHooks, ParserImplContext};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

/// Parser-hook handler for the `regex` grammar.
///
/// See the module-level docs for the architectural contract this
/// handler honors.
pub struct RegexParserHooks;

impl ParserHooks for RegexParserHooks {
    fn grammar_name(&self) -> &'static str {
        "regex"
    }

    fn extend_parser_impl(&self, ctx: &ParserImplContext<'_>) -> Option<TokenStream> {
        let parser_name = ctx.parser_name;

        // Per-rule typed methods. Each delegates to the corresponding
        // legacy parse method and converts the result to
        // `serde_json::Value` via the parsed content's
        // `to_json_value()`. By construction the output is byte-
        // equivalent to `legacy.content.to_json_value()`.
        let per_rule_typed_methods = ctx.rule_order.iter().map(|rule_name| {
            let typed_method_name = format_ident!("parse_{}_typed", rule_name);
            let legacy_method_name = format_ident!("parse_{}", rule_name);
            quote! {
                /// Typed entry point. Delegates to the legacy
                /// `parse_<rule>` method (preserving all semantic
                /// side-effects: predicates, fact emission,
                /// `with_semantic_runtime_rule_transaction`,
                /// `memoized_call`, recursion-guard checks) and
                /// converts the parsed content to
                /// `serde_json::Value`. By construction byte-
                /// equivalent to
                /// `self.<legacy>().map(|n| n.content.to_json_value())`.
                pub fn #typed_method_name(
                    &mut self,
                ) -> ParseResult<serde_json::Value> {
                    let node = self.#legacy_method_name()?;
                    Ok(node.content.to_json_value())
                }
            }
        });

        // No `parse_full_<entry>_typed` from this hook. The pipeline's
        // existing `--emit-typed-entry-skeleton` flag already emits one
        // (parser-agnostic, in `ast_pipeline::ast_based_generator`'s M1
        // skeleton). Avoiding the duplicate name here keeps both
        // mechanisms compatible: when a downstream binary opts into
        // both this hook and `--emit-typed-entry-skeleton`, the parser
        // ends up with M1's `parse_full_<entry>_typed` plus the per-rule
        // typed methods this hook emits. They don't collide.
        //
        // Consumers that want a full-input entry that returns
        // `legacy.content.to_json_value()` (rather than
        // `serde_json::to_value(&node)` which M1 produces) call
        // `self.parse_<entry>_typed()` from this hook.

        Some(quote! {
            impl<'input> #parser_name<'input> {
                #(#per_rule_typed_methods)*
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast_pipeline::ParserHooks;

    #[test]
    fn regex_hook_advertises_regex_grammar_name() {
        let hook = RegexParserHooks;
        assert_eq!(
            hook.grammar_name(),
            "regex",
            "regex hook must claim the `regex` grammar so the registry's lookup matches `regex.ebnf`"
        );
    }

    // The actual emit shape is exercised end-to-end by the pgen_ast
    // build path: when the regex hook is registered and `make
    // regex_parser` is run, the resulting parser must compile, the
    // typed methods must call the corresponding legacy methods, and
    // the differential gate must observe byte-equivalent JSON. Those
    // contracts live in integration tests and the M2 differential
    // gate, not here.
}
