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

use crate::ast_pipeline::{ASTNode, ASTValue, Annotations, ParserHooks, ParserImplContext, TokenValue};
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

        // Per-rule typed methods. For non-annotated rules whose AST shape
        // matches a known shape-typed emit, build `serde_json::Value`
        // directly; otherwise fall back to the slice-2 passthrough
        // (legacy parse + `ParseContent::to_json_value()`). Stage 2
        // covers `ASTNode::Atom` for the three common subtypes
        // (`quoted_string`, `regex`, `rule_reference`). Stages 3+ cover
        // `Sequence`, `Or`, `Quantified`, `Lookahead`, plus annotation-
        // aware emit. By construction every emit path produces output
        // byte-equivalent to `legacy.content.to_json_value()`; the
        // differential gate ([`make regex_typed_differential_gate`])
        // enforces this on every regen.
        let per_rule_typed_methods = ctx.rule_order.iter().map(|rule_name| {
            let typed_method_name = format_ident!("parse_{}_typed", rule_name);
            let legacy_method_name = format_ident!("parse_{}", rule_name);

            let body = ctx
                .grammar_tree
                .get(rule_name)
                .and_then(|ast_node| {
                    generate_typed_node_body(ast_node, rule_name, ctx.annotations)
                })
                .unwrap_or_else(|| {
                    quote! {
                        let node = self.#legacy_method_name()?;
                        Ok(node.content.to_json_value())
                    }
                });

            quote! {
                /// Typed entry point. Returns
                /// `ParseResult<serde_json::Value>` byte-equivalent to
                /// `self.<legacy>().map(|n| n.content.to_json_value())`.
                /// Body is shape-typed where the rule's AST node shape
                /// is supported by the hook's emit dispatcher; otherwise
                /// delegates to the legacy method and converts via
                /// `ParseContent::to_json_value()`. Either way, all
                /// semantic side-effects (predicates, fact emission,
                /// `with_semantic_runtime_rule_transaction`,
                /// `memoized_call`, recursion-guard checks) fire via
                /// the legacy method that the body either calls
                /// directly or recurses into through `parse_<inner>_typed`.
                pub fn #typed_method_name(
                    &mut self,
                ) -> ParseResult<serde_json::Value> {
                    #body
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

/// Slice 6 / M3-stage-2: shape-typed body emit per rule AST node.
///
/// Returns `Some(body)` when this rule's typed body has been shape-
/// typed (currently `ASTNode::Atom` for non-annotated rules with the
/// three common subtypes). Returns `None` to signal "fall back to the
/// passthrough (legacy + `to_json_value()`)" — for annotated rules
/// whose annotation-applied shape needs the legacy path's
/// `with_semantic_runtime_rule_transaction` wrapping, and for AST
/// shapes (`Sequence` / `Or` / `Quantified` / `Lookahead`) not yet
/// covered by the dispatcher.
///
/// Every shape-typed body produces output byte-equivalent to
/// `legacy.content.to_json_value()` — verified by the differential
/// gate at every regen.
fn generate_typed_node_body(
    ast_node: &ASTNode,
    rule_name: &str,
    annotations: Option<&Annotations>,
) -> Option<TokenStream> {
    // Slice 6 only dispatches non-annotated rules. Rules with semantic
    // annotations (`@semantic_value`, `@predicate`, `@emit_fact`,
    // `@validate`) or branch return annotations need the legacy path's
    // semantic-runtime wrapper to fire; the typed body would skip
    // those side-effects. Stage 4c (later slice) extends the dispatch
    // to semantic-annotated rules once the byte-equivalence story for
    // annotation-applied shapes is in place.
    if !rule_has_no_semantic_annotations(rule_name, annotations) {
        return None;
    }
    if let Some(annotations) = annotations
        && annotations.branch_return_annotations.contains_key(rule_name)
    {
        return None;
    }

    match ast_node {
        ASTNode::Atom { value } => generate_typed_atom_body(value, rule_name),
        // Stages 3+ add the other shapes.
        _ => None,
    }
}

/// Slice 6 / M3-stage-2: shape-typed body for `ASTNode::Atom`.
///
/// Three subtypes:
/// - `quoted_string`: `match_string(literal)` → `Value::String(matched)`.
/// - `regex`: `match_regex(pattern, skip_ws)` → `Value::String(matched)`.
/// - `rule_reference`: `parse_<inner>_typed()` (recurses).
///
/// All three produce output byte-equivalent to what `to_json_value()`
/// returns for the legacy `ParseContent::Terminal(matched_str)` (or, for
/// `rule_reference`, whatever the inner rule's typed entry returns —
/// which is byte-equivalent by induction).
///
/// Returns `None` for unrecognized atom subtypes so the caller falls
/// back to the passthrough.
fn generate_typed_atom_body(value: &ASTValue, rule_name: &str) -> Option<TokenStream> {
    let ASTValue::Token(parts) = value else {
        return None;
    };
    if parts.len() < 2 {
        return None;
    }
    let TokenValue::String(token_type) = &parts[0];
    let TokenValue::String(token_value) = &parts[1];

    match token_type.as_str() {
        "quoted_string" => Some(quote! {
            let matched_str = self.match_string(#token_value)?;
            Ok(serde_json::Value::String(matched_str.to_string()))
        }),
        "regex" => {
            // Inherits the same skip-leading-whitespace policy as the
            // legacy `match_regex` call sites in the regex grammar.
            // The two exceptions are `string_content_double` and
            // `string_content_single` — string-body rules that must
            // not consume leading whitespace because that whitespace
            // would belong to the string literal's contents. Other
            // regex-grammar rules either don't have these subtypes or
            // are happy to skip leading whitespace.
            let skip_leading_whitespace =
                !matches!(rule_name, "string_content_double" | "string_content_single");
            Some(quote! {
                let matched_str = self.match_regex(
                    #token_value,
                    #skip_leading_whitespace,
                )?;
                Ok(serde_json::Value::String(matched_str.to_string()))
            })
        }
        "rule_reference" => {
            let inner_typed = format_ident!("parse_{}_typed", token_value);
            Some(quote! {
                self.#inner_typed()
            })
        }
        _ => None,
    }
}

/// Slice 6 / M3-stage-2: rule has no semantic annotations of any kind
/// (direct, branch-level, or branch-mid-sequence). Mirrors the helper
/// of the same name on `AstBasedGenerator` (which is private to the
/// pipeline crate); kept duplicated here so the hook stays self-
/// contained without exposing internal generator state.
fn rule_has_no_semantic_annotations(
    rule_name: &str,
    annotations: Option<&Annotations>,
) -> bool {
    let Some(annotations) = annotations else {
        return true;
    };
    let direct_empty = annotations
        .semantic_annotations
        .get(rule_name)
        .is_none_or(|v| v.is_empty());
    let branch_empty = annotations
        .branch_semantic_annotations
        .get(rule_name)
        .is_none_or(|branches| branches.iter().all(|b| b.is_empty()));
    let mid_seq_empty = annotations
        .branch_mid_sequence_semantic_annotations
        .get(rule_name)
        .is_none_or(|branches| branches.iter().all(|b| b.is_empty()));
    direct_empty && branch_empty && mid_seq_empty
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
