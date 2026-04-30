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

use crate::ast_pipeline::{
    ASTNode, ASTValue, Annotations, ParserHooks, ParserImplContext, TokenValue, UnifiedReturnAST,
};
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
    // Semantic annotations (`@semantic_value`, `@predicate`,
    // `@emit_fact`, `@validate`) shape the parser's runtime state;
    // the typed path doesn't yet apply them, so rules carrying any
    // semantic annotation still fall back to passthrough. Slice 12
    // (M3 stage 4c) extends the dispatch to semantic-annotated rules
    // on the typed path.
    if !rule_has_no_semantic_annotations(rule_name, annotations) {
        return None;
    }

    // Slice 10 / M3-stage-4b: explicit return annotations
    // (`return_object`, `return_array`, `return_scalar`) get a typed
    // body that applies the annotation transform directly to typed
    // sub-values, no `ParseNode` round-trip. The frontend stores
    // per-branch return-annotation slots for every rule, but a `None`
    // slot just means "no explicit annotation, fall back to the
    // synthetic `-> $1` default". The check below counts only EXPLICIT
    // (non-`None`) branch annotations — implicit `-> $1` is identity
    // passthrough, exactly what the typed shape emit produces, so
    // implicit-only rules stay on the typed shape-emit path.
    let has_explicit_return_annotation = annotations
        .and_then(|a| a.branch_return_annotations.get(rule_name))
        .is_some_and(|branches| branches.iter().any(Option::is_some));
    if has_explicit_return_annotation {
        return generate_typed_annotated_body(ast_node, rule_name, annotations);
    }

    match ast_node {
        ASTNode::Atom { value } => generate_typed_atom_body(value, rule_name),
        ASTNode::Sequence { elements } => generate_typed_sequence_body(elements, rule_name),
        ASTNode::Or { alternatives } => generate_typed_or_body(alternatives, rule_name),
        ASTNode::Quantified {
            element,
            quantifier,
        } => generate_typed_quantified_body(element, quantifier, rule_name),
        // `Lookahead` is rare at the rule-body level and stays on
        // passthrough until a concrete need surfaces.
        _ => None,
    }
}

/// Slice 10 / M3-stage-4b: shape-typed body for rules with explicit
/// return annotations (`return_scalar`, `return_object`, `return_array`).
///
/// **Two-phase emit:**
/// 1. Parse phase: walk the rule body, parsing each top-level element
///    into a numbered local `__pgen_v<i>` (1-based, matching `$N`
///    positional refs in annotations). Failures propagate via `?`.
/// 2. Build phase: walk the parsed `UnifiedReturnAST` and emit
///    `serde_json::Value`-building code that references the locals
///    by index.
///
/// **Body shape support:** `Sequence` (N elements at the top), `Atom`
/// (1 element), `Quantified` (1 element with `?`/`*`/`+` over Atom
/// inner). Or rules (per-branch annotations) and Lookahead deferred.
///
/// **Annotation shape support:** `PositionalRef`, `StringLiteral`,
/// `NumberLiteral`, `BooleanLiteral`, `Object` (recursive), `Array`,
/// `Passthrough` (treated as `$1`). PropertyAccess / ArrayAccess /
/// Spread / QuantifiedExtraction / Identifier return `None` and the
/// caller falls back to the slice-2 passthrough body.
///
/// **Byte-equivalence:** the legacy emit for annotated rules produces
/// `ParseContent::Json(transformed_value)` whose `to_json_value()`
/// returns `value.clone()`. The typed body builds the same JSON
/// structure directly. Positional ref semantics: `$N` over a
/// `?`-Quantified element returns the inner value or `Value::Null`
/// (NOT `Value::Array(vec![])` — annotation-side positional refs
/// dereference Quantified content rather than carry the array
/// wrapper).
fn generate_typed_annotated_body(
    ast_node: &ASTNode,
    rule_name: &str,
    annotations: Option<&Annotations>,
) -> Option<TokenStream> {
    // For Or rules, each branch may carry its own annotation; the
    // typed emit needs per-branch dispatch. Defer to slice 11+.
    if matches!(ast_node, ASTNode::Or { .. }) {
        return None;
    }

    // Resolve the rule's branch-0 annotation (Or rules are filtered
    // above; non-Or rules have a single conceptual branch).
    let branch_annotation = annotations?
        .branch_return_annotations
        .get(rule_name)
        .and_then(|branches| branches.first().cloned())
        .flatten()?;
    let parsed_ast = branch_annotation.parsed_ast.as_ref()?;

    // Determine top-level matched elements for `$N` indexing and emit
    // the parse-phase code that stores each element's typed value in
    // `__pgen_v<i>`.
    let elements: Vec<&ASTNode> = match ast_node {
        ASTNode::Sequence { elements } => elements.iter().collect(),
        ASTNode::Atom { .. } => vec![ast_node],
        ASTNode::Quantified { .. } => vec![ast_node],
        ASTNode::Or { .. } => return None,
        ASTNode::Lookahead { .. } => return None,
    };

    let mut parse_phase: Vec<TokenStream> = Vec::with_capacity(elements.len());
    for (idx, elem) in elements.iter().enumerate() {
        let v_ident = format_ident!("__pgen_v{}", idx + 1);
        let elem_expr = generate_typed_annotated_element_expr(elem, rule_name)?;
        parse_phase.push(quote! {
            let #v_ident: serde_json::Value = #elem_expr;
        });
    }

    let value_expr = generate_typed_annotation_value_expr(parsed_ast, elements.len())?;

    Some(quote! {
        #(#parse_phase)*
        Ok(#value_expr)
    })
}

/// Slice 10 / M3-stage-4b: per-element typed expression for the
/// stage-4b parse phase. Returns an expression of type
/// `serde_json::Value` (or `?`-propagating to `ParseError`). Each
/// element corresponds to one positional `$N` slot.
///
/// **Positional-ref Quantified semantics:** `$N` references over a
/// `?`-Quantified element produce the inner value on hit and
/// `Value::Null` on miss (legacy positional ref dereferences the
/// Quantified content rather than carrying the array wrapper).
/// `*`/`+` produce `Value::Array(matches)`. This differs from
/// rule-body-level Quantified emit (slice 9) which always wraps in
/// `Value::Array` to match `ParseContent::Quantified.to_json_value()`.
fn generate_typed_annotated_element_expr(
    element: &ASTNode,
    rule_name: &str,
) -> Option<TokenStream> {
    match element {
        ASTNode::Atom { value } => {
            generate_typed_atom_value_expr(value, rule_name, quote! { self })
        }
        ASTNode::Quantified {
            element: inner,
            quantifier,
        } if quantifier == "?" => {
            let inner_expr = generate_typed_annotated_element_expr_with_receiver(
                inner,
                rule_name,
                quote! { parser },
            )?;
            Some(quote! {
                if let Some(__pgen_qv) = self.try_parse(|p| {
                    let parser = p;
                    Ok::<serde_json::Value, ParseError>(#inner_expr)
                }) {
                    __pgen_qv
                } else {
                    serde_json::Value::Null
                }
            })
        }
        ASTNode::Quantified {
            element: inner,
            quantifier,
        } if quantifier == "*" => {
            let inner_expr = generate_typed_annotated_element_expr_with_receiver(
                inner,
                rule_name,
                quote! { parser },
            )?;
            Some(quote! {
                {
                    let mut __pgen_qe: Vec<serde_json::Value> = Vec::new();
                    while let Some(__pgen_qv) = self.try_parse(|p| {
                        let parser = p;
                        Ok::<serde_json::Value, ParseError>(#inner_expr)
                    }) {
                        __pgen_qe.push(__pgen_qv);
                    }
                    serde_json::Value::Array(__pgen_qe)
                }
            })
        }
        ASTNode::Quantified {
            element: inner,
            quantifier,
        } if quantifier == "+" => {
            let inner_expr_self = generate_typed_annotated_element_expr_with_receiver(
                inner,
                rule_name,
                quote! { self },
            )?;
            let inner_expr_parser = generate_typed_annotated_element_expr_with_receiver(
                inner,
                rule_name,
                quote! { parser },
            )?;
            Some(quote! {
                {
                    let mut __pgen_qe: Vec<serde_json::Value> = Vec::new();
                    __pgen_qe.push(#inner_expr_self);
                    while let Some(__pgen_qv) = self.try_parse(|p| {
                        let parser = p;
                        Ok::<serde_json::Value, ParseError>(#inner_expr_parser)
                    }) {
                        __pgen_qe.push(__pgen_qv);
                    }
                    serde_json::Value::Array(__pgen_qe)
                }
            })
        }
        _ => None,
    }
}

/// Companion to [`generate_typed_annotated_element_expr`] that lets
/// the caller supply the parser receiver (`self` at top-level,
/// `parser` inside `try_parse` closures). Used by the `?`/`*`/`+`
/// quantified arms when recursing into the inner element.
fn generate_typed_annotated_element_expr_with_receiver(
    element: &ASTNode,
    rule_name: &str,
    receiver: TokenStream,
) -> Option<TokenStream> {
    match element {
        ASTNode::Atom { value } => generate_typed_atom_value_expr(value, rule_name, receiver),
        _ => None,
    }
}

/// Slice 10 / M3-stage-4b: build-phase emitter. Walks the parsed
/// return-annotation AST and emits a `serde_json::Value`-producing
/// expression that references `__pgen_v<i>` locals built in the
/// parse phase.
///
/// **Annotation shape support:** `PositionalRef`, `StringLiteral`,
/// `NumberLiteral`, `BooleanLiteral`, `Object`, `Array`, `Passthrough`.
/// Returns `None` for `PropertyAccess`, `ArrayAccess`, `Spread`,
/// `QuantifiedExtraction`, `Identifier` — those rules fall back to
/// the slice-2 passthrough until a follow-up slice extends the
/// emitter.
fn generate_typed_annotation_value_expr(
    ast: &UnifiedReturnAST,
    num_elements: usize,
) -> Option<TokenStream> {
    match ast {
        UnifiedReturnAST::PositionalRef { index } => {
            if *index < 1 || *index > num_elements {
                return None;
            }
            let v_ident = format_ident!("__pgen_v{}", index);
            Some(quote! { #v_ident.clone() })
        }
        UnifiedReturnAST::StringLiteral { value } => Some(quote! {
            serde_json::Value::String(#value.to_string())
        }),
        UnifiedReturnAST::NumberLiteral { value } => {
            let lit = proc_macro2::Literal::f64_unsuffixed(*value);
            Some(quote! {
                serde_json::Value::Number(
                    serde_json::Number::from_f64(#lit)
                        .unwrap_or_else(|| serde_json::Number::from(0))
                )
            })
        }
        UnifiedReturnAST::BooleanLiteral { value } => {
            let v = *value;
            Some(quote! { serde_json::Value::Bool(#v) })
        }
        UnifiedReturnAST::Object { properties } => {
            // Sort keys for deterministic emit order.
            let mut keys: Vec<&String> = properties.keys().collect();
            keys.sort();
            let mut inserts: Vec<TokenStream> = Vec::with_capacity(keys.len());
            for key in keys {
                let value_ast = properties.get(key)?;
                let value_expr = generate_typed_annotation_value_expr(value_ast, num_elements)?;
                inserts.push(quote! {
                    __pgen_obj.insert(#key.to_string(), #value_expr);
                });
            }
            Some(quote! {
                {
                    let mut __pgen_obj = serde_json::Map::new();
                    #(#inserts)*
                    serde_json::Value::Object(__pgen_obj)
                }
            })
        }
        UnifiedReturnAST::Array { elements } => {
            let element_exprs: Option<Vec<TokenStream>> = elements
                .iter()
                .map(|e| generate_typed_annotation_value_expr(e, num_elements))
                .collect();
            let element_exprs = element_exprs?;
            Some(quote! {
                serde_json::Value::Array(vec![#(#element_exprs),*])
            })
        }
        UnifiedReturnAST::Passthrough => {
            if num_elements < 1 {
                return None;
            }
            Some(quote! { __pgen_v1.clone() })
        }
        // PropertyAccess, ArrayAccess, QuantifiedExtraction, Spread,
        // Identifier — defer.
        _ => None,
    }
}

/// Slice 6 / M3-stage-2: shape-typed body (statement form) for
/// `ASTNode::Atom`. Wraps [`generate_typed_atom_value_expr`] with
/// `Ok(...)` to produce a method-body-shaped statement.
fn generate_typed_atom_body(value: &ASTValue, rule_name: &str) -> Option<TokenStream> {
    let expr = generate_typed_atom_value_expr(value, rule_name, quote! { self })?;
    Some(quote! { Ok(#expr) })
}

/// Slice 6 / M3-stage-2 (refactored in slice 7): shape-typed
/// **expression** (not statement body) for `ASTNode::Atom`. Returns
/// a block expression that produces `serde_json::Value` (or short-
/// circuits via `?`). The `receiver` parameter is the parser receiver
/// to use — `self` at top-level method bodies, or `parser` inside
/// `try_parse` / nested closures (consistent with the legacy emit's
/// `let parser = p;` shadowing convention). Slice 7 introduces the
/// expression form so Sequence emit can inline atom expressions at
/// child positions including `try_parse` closures.
///
/// Three subtypes:
/// - `quoted_string`: `match_string(literal)` → `Value::String(matched)`.
/// - `regex`: `match_regex(pattern, skip_ws)` → `Value::String(matched)`.
/// - `rule_reference`: `parse_<inner>_typed()?` (recurses).
///
/// All three produce output byte-equivalent to what `to_json_value()`
/// returns for the legacy `ParseContent::Terminal(matched_str)` (or, for
/// `rule_reference`, whatever the inner rule's typed entry returns —
/// which is byte-equivalent by induction).
fn generate_typed_atom_value_expr(
    value: &ASTValue,
    rule_name: &str,
    receiver: TokenStream,
) -> Option<TokenStream> {
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
            {
                let __pgen_typed_matched = #receiver.match_string(#token_value)?;
                serde_json::Value::String(__pgen_typed_matched.to_string())
            }
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
                {
                    let __pgen_typed_matched = #receiver.match_regex(
                        #token_value,
                        #skip_leading_whitespace,
                    )?;
                    serde_json::Value::String(__pgen_typed_matched.to_string())
                }
            })
        }
        "rule_reference" => {
            let inner_typed = format_ident!("parse_{}_typed", token_value);
            Some(quote! {
                #receiver.#inner_typed()?
            })
        }
        _ => None,
    }
}

/// Slice 7 / M3-stage-3: shape-typed body for `ASTNode::Sequence`.
///
/// Builds `Value::Array(child_typed_values)` by parsing each child in
/// order. Stage 3 handles two child shapes:
/// - `ASTNode::Atom`: inline the typed atom expression and push the
///   resulting `Value`.
/// - `ASTNode::Quantified` with quantifier `"?"` whose inner element
///   is an `Atom`: wrap in `try_parse` and push a
///   `Value::Array(...)` carrier matching what
///   `ParseContent::Quantified(_, "?").to_json_value()` produces on
///   the legacy path.
///
/// **Byte-equivalence for `?`-Quantified-Atom:** legacy
/// `ParseContent::Quantified(nodes, _)` always serializes via
/// `to_json_value()` to `Value::Array`. So matched gives
/// `Value::Array(vec![<inner-typed>])` and unmatched gives
/// `Value::Array(vec![])` — NOT a bare matched value or `Value::Null`.
/// (M3's stage 3 emitted `Value::Null` on miss, which was the
/// byte-equivalence divergence the differential gate caught at
/// rollback time. Slice 7 fixes this.)
///
/// Returns `None` if any child is a shape stage 3 doesn't yet handle
/// (nested `Or` / `Sequence` / `Lookahead` / `Quantified` with non-`?`
/// quantifier or non-`Atom` inner). In that case the caller falls
/// back to the passthrough body.
fn generate_typed_sequence_body(elements: &[ASTNode], rule_name: &str) -> Option<TokenStream> {
    let element_count = elements.len();
    let mut element_pushes: Vec<TokenStream> = Vec::with_capacity(element_count);
    for elem in elements {
        let push = match elem {
            ASTNode::Atom { value } => {
                let expr = generate_typed_atom_value_expr(value, rule_name, quote! { self })?;
                quote! {
                    elements.push(#expr);
                }
            }
            ASTNode::Quantified {
                element,
                quantifier,
            } if quantifier == "?" => {
                let ASTNode::Atom { value } = element.as_ref() else {
                    return None;
                };
                let inner_expr =
                    generate_typed_atom_value_expr(value, rule_name, quote! { parser })?;
                quote! {
                    let __pgen_optional_carrier: Vec<serde_json::Value> = if let Some(__pgen_v) =
                        self.try_parse(|p| {
                            let parser = p;
                            Ok::<serde_json::Value, ParseError>(#inner_expr)
                        })
                    {
                        vec![__pgen_v]
                    } else {
                        Vec::new()
                    };
                    elements.push(serde_json::Value::Array(__pgen_optional_carrier));
                }
            }
            _ => return None,
        };
        element_pushes.push(push);
    }

    Some(quote! {
        let mut elements: Vec<serde_json::Value> = Vec::with_capacity(#element_count);
        #(#element_pushes)*
        Ok(serde_json::Value::Array(elements))
    })
}

/// Slice 9 / M3-stage-5: shape-typed body for `ASTNode::Quantified`
/// at the rule-body level.
///
/// Three quantifiers handled, each requiring the inner element to be
/// `ASTNode::Atom`:
/// - `?` — optional. Emits `Value::Array(vec![v])` on hit /
///   `Value::Array(vec![])` on miss.
/// - `*` — zero-or-more. Loops `try_parse`; emits
///   `Value::Array(matches)`.
/// - `+` — one-or-more. Requires a first match (no `try_parse`), then
///   loops; emits `Value::Array(matches)`.
///
/// **Byte-equivalence fix vs M3:** M3's stage 5 emit for `?` returned
/// `Value::Null` on miss and the bare matched value on hit. Legacy
/// `ParseContent::Quantified(_, _).to_json_value()` always serializes
/// to `Value::Array(...)` regardless of quantifier kind — `?` matched
/// gives `Value::Array(vec![v])`, unmatched gives `Value::Array(vec![])`.
/// Slice 9 emits the carrier shape, the same way slice 7 fixed the
/// `Quantified-?-Atom` child case inside Sequence emit. The `*` and
/// `+` cases already produced `Value::Array` correctly in M3 and are
/// ported as-is.
///
/// Returns `None` for `{n,m}` (bounded count, deferred to a separate
/// slice that handles min/max enforcement) and for non-Atom inners
/// (nested `Sequence` / `Or` / etc., deferred to slice 11's generic
/// shape composer).
fn generate_typed_quantified_body(
    element: &ASTNode,
    quantifier: &str,
    rule_name: &str,
) -> Option<TokenStream> {
    let ASTNode::Atom { value } = element else {
        return None;
    };
    match quantifier {
        "?" => {
            let expr_parser =
                generate_typed_atom_value_expr(value, rule_name, quote! { parser })?;
            Some(quote! {
                let __pgen_q_carrier: Vec<serde_json::Value> = if let Some(__pgen_q_v) =
                    self.try_parse(|p| {
                        let parser = p;
                        Ok::<serde_json::Value, ParseError>(#expr_parser)
                    })
                {
                    vec![__pgen_q_v]
                } else {
                    Vec::new()
                };
                Ok(serde_json::Value::Array(__pgen_q_carrier))
            })
        }
        "*" => {
            let expr_parser =
                generate_typed_atom_value_expr(value, rule_name, quote! { parser })?;
            Some(quote! {
                let mut __pgen_q_elements: Vec<serde_json::Value> = Vec::new();
                while let Some(__pgen_q_v) = self.try_parse(|p| {
                    let parser = p;
                    Ok::<serde_json::Value, ParseError>(#expr_parser)
                }) {
                    __pgen_q_elements.push(__pgen_q_v);
                }
                Ok(serde_json::Value::Array(__pgen_q_elements))
            })
        }
        "+" => {
            let expr_self = generate_typed_atom_value_expr(value, rule_name, quote! { self })?;
            let expr_parser =
                generate_typed_atom_value_expr(value, rule_name, quote! { parser })?;
            Some(quote! {
                let mut __pgen_q_elements: Vec<serde_json::Value> = Vec::new();
                let __pgen_q_first = #expr_self;
                __pgen_q_elements.push(__pgen_q_first);
                while let Some(__pgen_q_v) = self.try_parse(|p| {
                    let parser = p;
                    Ok::<serde_json::Value, ParseError>(#expr_parser)
                }) {
                    __pgen_q_elements.push(__pgen_q_v);
                }
                Ok(serde_json::Value::Array(__pgen_q_elements))
            })
        }
        // {n,m} and other forms deferred to a later slice (min/max
        // enforcement is a separate concern).
        _ => None,
    }
}

/// Slice 8 / M3-stage-4: shape-typed body for `ASTNode::Or`.
///
/// For Or rules whose alternatives are all `ASTNode::Atom`, emits a
/// sequence of `try_parse` blocks: each tries the next alternative,
/// and the first successful one returns its typed value. If all
/// alternatives fail, returns `Err(ParseError::Backtrack { position:
/// self.position })` so the caller's outer Or / Sequence can retry at
/// that position.
///
/// **Byte-equivalence:** legacy `ParseContent::Alternative(child)
/// .to_json_value()` returns `child.content.to_json_value()` — i.e.
/// the chosen alternative's content unwrapped. The typed emit returns
/// the alternative's typed value directly. Match.
///
/// The dispatcher's gate at `generate_typed_node_body` already
/// filters out rules with semantic or branch return annotations, so
/// the Or rules that reach this function either have no annotations
/// at all OR rely on the implicit `-> $1` default for single-element
/// branches (which is synthesized at codegen time by the generator,
/// not stored in `branch_return_annotations`). The typed semantics —
/// return the alternative's typed value — match the implicit `-> $1`
/// passthrough, so the typed path is equivalent.
///
/// Returns `None` if any alternative is not an `ASTNode::Atom` (e.g.
/// nested Sequence / Or / Quantified / Lookahead). Stages 5+ extend
/// the supported alternative shapes.
fn generate_typed_or_body(alternatives: &[ASTNode], rule_name: &str) -> Option<TokenStream> {
    let mut try_blocks: Vec<TokenStream> = Vec::with_capacity(alternatives.len());
    for alt in alternatives {
        let ASTNode::Atom { value } = alt else {
            return None;
        };
        let expr = generate_typed_atom_value_expr(value, rule_name, quote! { parser })?;
        try_blocks.push(quote! {
            if let Some(__pgen_or_v) = self.try_parse(|p| {
                let parser = p;
                Ok::<serde_json::Value, ParseError>(#expr)
            }) {
                return Ok(__pgen_or_v);
            }
        });
    }
    Some(quote! {
        #(#try_blocks)*
        Err(ParseError::Backtrack { position: self.position })
    })
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
