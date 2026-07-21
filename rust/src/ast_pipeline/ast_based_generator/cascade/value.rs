//! RGX-0078.5.j.2 STEP-2a — the DIRECT-VALUE BUILD emitter (session #150,
//! consuming the CORRECTED `DirectValueBuildPlan` — `PGEN-RGX-0078-0126`).
//!
//! Three emission classes over the fused build pass (the plan partition is the
//! census's, single implementation — `compute_direct_value_build_plan` and the
//! shared classification fns in `fusibility_census`):
//!
//! - **value_licensed** rules get `cascade_build_value_<rule>(&mut self) ->
//!   PgenValue<'input>` — the rule's committed VALUE computed directly from
//!   the derivation tape: no element-wrapper `ParseNode`s, no
//!   `sequence_elements` Vec, no `Sequence` content, no transform-input
//!   clones. Sound because every consumption path folds the rule's content to
//!   a value before it can be observed (`to_shaped_value` compositionality:
//!   convert-early ≡ convert-late).
//! - **barrier** rules (VALUE-PURE fold on every branch) keep the node
//!   signature (`ParseNode` — consumers may need the node, and the content is
//!   byte-identical either way because a VALUE-PURE fold rebuilds it from
//!   scratch) but compute the fold's content IN PLACE from bound child values.
//! - **node_locked** rules keep `generate_mtb_build_rule_fn`'s node emission,
//!   with PER-BRANCH modes decided here ([`DvBranchMode`]): a VALUE-PURE
//!   branch builds its `Shaped`/`Terminal` content in place; a transparent
//!   `-> $N` branch with a statically-resolved element target builds ONLY that
//!   element's content (siblings advance the cursor through discard walkers);
//!   everything else (passthrough, content-carrying, `WholeBody` targets,
//!   vocabulary-demoted rules) stays VERBATIM — exactly the branches whose
//!   references the census demand walk node-locked, so partition and emission
//!   cannot disagree (and a drift bails loudly at the verbatim reference
//!   site — see `mtb_build_atom_logic`).
//!
//! Byte-exactness parity edges honoured throughout (each mirrors the runtime
//! the fold would have executed):
//! - `Sequence`/`Quantified` → `Array` element order; an absent `?` element
//!   and a `Lookahead` fold to an EMPTY array.
//! - Terminal spans replay the exact match-pass event protocol (`TokStart`
//!   only where a layout skip precedes a regex token; `TokEnd` where the
//!   length is dynamic; static-layout literals derive both ends).
//! - Sentinel spellings per context: content-level `$0` →
//!   `<invalid_positional_ref>` vs object-property `$0` → `<invalid_ref_0>`;
//!   both out-of-range forms → `<invalid_sequence_access>` (the
//!   `generate_positional_ref` / `generate_value_extraction` split).
//! - `$text` = span math over `start_pos..deriv_pos` (the `-0101` fix's
//!   invariant: at a build-side transform site `deriv_pos` IS what
//!   `parser.position` was at the eager transform point).
//! - `Spread` keeps its runtime variant dispatch collapsed STATICALLY (the
//!   vocabulary audit guarantees the base is a non-`?` `Quantified` target or
//!   a static sentinel); `FlattenSpread` unwraps exactly one level per item
//!   (item Array-ness ⇔ the content shapes the audit admitted).

use super::super::super::{ASTNode, ASTValue, TokenValue};
use super::super::AstBasedGenerator;
use crate::ast_pipeline::fusibility_census::{
    self as census, PositionalTarget, TransformFoldClass,
};
use crate::ast_pipeline::unified_return_ast::UnifiedReturnAST;
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

/// The per-branch build mode of a NODE-form (barrier or node_locked) fused
/// rule — decided from the SAME census fns the plan partition used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in super::super) enum DvBranchMode {
    /// Today's emission: full body scaffolding + the branch transform.
    Verbatim,
    /// VALUE-PURE fold: bind referenced element VALUES (discard-walk the
    /// rest), emit the fold's `Shaped`/`Terminal` content directly.
    InPlaceValuePure,
    /// Transparent `-> $N` with a static `Element` target: build ONLY that
    /// element's content in place; discard-walk the siblings.
    InPlaceTransparentElement(usize),
    /// Transparent `-> $N` statically out of range: discard-walk everything,
    /// content = the sentinel `Terminal`.
    InPlaceTransparentSentinel(&'static str),
}

/// Sentinel spelling context — `generate_positional_ref` (content level) vs
/// `generate_value_extraction` (object-property level) differ on `$0`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DvSentinelCtx {
    Content,
    Extraction,
}

impl DvSentinelCtx {
    fn index_zero(self) -> &'static str {
        match self {
            DvSentinelCtx::Content => "<invalid_positional_ref>",
            DvSentinelCtx::Extraction => "<invalid_ref_0>",
        }
    }
}

/// The branch body viewed as the element list the tape walk traverses: a
/// `Sequence`'s elements, or the whole body as one pseudo-element.
struct DvShape<'tree> {
    elements: Vec<&'tree ASTNode>,
    is_sequence: bool,
}

impl<'tree> DvShape<'tree> {
    fn of(branch_body: &'tree ASTNode) -> Self {
        match branch_body {
            ASTNode::Sequence { elements } => DvShape {
                elements: elements.iter().collect(),
                is_sequence: true,
            },
            other => DvShape {
                elements: vec![other],
                is_sequence: false,
            },
        }
    }
}

impl AstBasedGenerator {
    /// The per-branch mode of a NODE-form fused rule (never called for
    /// value_licensed rules — those get the value fn). Demoted rules are
    /// verbatim on every branch by definition.
    pub(in super::super) fn dv_branch_build_mode(
        &self,
        rule_name: &str,
        branch_index: usize,
        branch_body: &ASTNode,
    ) -> DvBranchMode {
        if self.dv_demoted_rule(rule_name) {
            return DvBranchMode::Verbatim;
        }
        if !self.dv_barrier_rule(rule_name) && !self.dv_node_locked_rule(rule_name) {
            // Plan-inactive generation or a rule outside the direct-value
            // partition: today's emission.
            return DvBranchMode::Verbatim;
        }
        if census::rule_has_matched_text_transform(rule_name, self.annotations.as_ref()) {
            // Defensive: such a rule is vocabulary-demoted by the census; a
            // reachable disagreement would mean partition drift, and verbatim
            // is the sound answer either way.
            return DvBranchMode::Verbatim;
        }
        let resolved = census::resolved_branch_return_ast(
            rule_name,
            branch_index,
            branch_body,
            self.annotations.as_ref(),
        );
        match resolved.as_ref().map(census::return_ast_fold_class) {
            Some(TransformFoldClass::ValuePure) => DvBranchMode::InPlaceValuePure,
            Some(TransformFoldClass::Transparent) => match resolved.as_ref() {
                Some(UnifiedReturnAST::PositionalRef { index }) => {
                    if *index == 0 {
                        return DvBranchMode::InPlaceTransparentSentinel(
                            DvSentinelCtx::Content.index_zero(),
                        );
                    }
                    match census::resolve_positional_target(branch_body, *index) {
                        PositionalTarget::Element(_) => {
                            DvBranchMode::InPlaceTransparentElement(*index - 1)
                        }
                        PositionalTarget::WholeBody => DvBranchMode::Verbatim,
                        PositionalTarget::StaticSentinel => {
                            DvBranchMode::InPlaceTransparentSentinel("<invalid_sequence_access>")
                        }
                    }
                }
                _ => DvBranchMode::Verbatim,
            },
            Some(TransformFoldClass::ContentCarrying) | None => DvBranchMode::Verbatim,
        }
    }

    // ------------------------------------------------------------------
    // The VALUE fn — value_licensed rules.
    // ------------------------------------------------------------------

    /// `cascade_build_value_<rule>(&mut self) -> PgenValue<'input>` — the
    /// whole rule as a direct value computation over the committed tape.
    pub(in super::super) fn generate_mtb_build_value_fn(
        &self,
        rule_name: &str,
        ast_node: &ASTNode,
    ) -> Result<TokenStream> {
        let value_fn = format_ident!("cascade_build_value_{}", rule_name);
        let dispatch = match ast_node {
            ASTNode::Or { alternatives } => self.dv_or_value_expr(alternatives, rule_name, true)?,
            _ => self.dv_branch_value_expr(rule_name, 0, ast_node)?,
        };
        let start_pos_bind = if self.dv_rule_uses_matched_text(rule_name, ast_node) {
            quote! { let start_pos = parser.deriv_pos; }
        } else {
            quote! {}
        };
        Ok(quote! {
            fn #value_fn(&mut self) -> PgenValue<'input> {
                let parser = self;
                #start_pos_bind
                #dispatch
            }
        })
    }

    /// Does any branch's resolved transform contain `$text` (MatchedText)?
    /// Gates the `start_pos` binding so value fns without it carry no unused
    /// local.
    fn dv_rule_uses_matched_text(&self, rule_name: &str, ast_node: &ASTNode) -> bool {
        census::rule_branch_bodies(ast_node)
            .iter()
            .enumerate()
            .any(|(idx, branch_body)| {
                census::resolved_branch_return_ast(
                    rule_name,
                    idx,
                    branch_body,
                    self.annotations.as_ref(),
                )
                .as_ref()
                .is_some_and(dv_ast_contains_matched_text)
            })
    }

    /// The `Or` value dispatch — the `mtb_build_or_logic` mirror in value
    /// space: single-branch pass-through, the SHARED P2 byte-switch
    /// re-dispatch on the committed input byte, or the `OrWinner`-driven
    /// winner arm. Every arm is one branch's value expression.
    fn dv_or_value_expr(
        &self,
        alternatives: &[ASTNode],
        rule_name: &str,
        top_level: bool,
    ) -> Result<TokenStream> {
        if alternatives.len() == 1 {
            return self.dv_branch_value_expr(rule_name, 0, &alternatives[0]);
        }
        let emit_first_set_guard = top_level && self.layout_sensitivity().terminals;
        let mut first_set_cache: std::collections::HashMap<
            String,
            super::super::super::first_set::FirstSetSummary,
        > = std::collections::HashMap::new();
        if let Some(branch_byte_sets) = self.degenerate_dispatch_byte_sets(
            alternatives,
            rule_name,
            emit_first_set_guard,
            &mut first_set_cache,
        ) {
            let mut dispatch_arms = Vec::new();
            for (idx, alternative) in alternatives.iter().enumerate() {
                let branch_value = self.dv_branch_value_expr(rule_name, idx, alternative)?;
                let byte_patterns = &branch_byte_sets[idx];
                dispatch_arms.push(quote! {
                    #(#byte_patterns)|* => { #branch_value }
                });
            }
            return Ok(quote! {
                match parser.input.as_bytes()[parser.deriv_pos] {
                    #(#dispatch_arms,)*
                    __pgen_byte => unreachable!(
                        "derivation-tape drift in rule '{}': no byte-switch arm admits committed byte {}",
                        #rule_name,
                        __pgen_byte,
                    ),
                }
            });
        }
        let mut winner_arms: Vec<TokenStream> = Vec::new();
        for (idx, alternative) in alternatives.iter().enumerate() {
            let branch_value = self.dv_branch_value_expr(rule_name, idx, alternative)?;
            winner_arms.push(quote! {
                #idx => { #branch_value }
            });
        }
        Ok(quote! {
            {
                let __pgen_or_winner = match parser.deriv_next_event() {
                    crate::ast_pipeline::DerivEvent::OrWinner(__pgen_idx) => __pgen_idx,
                    __pgen_other => unreachable!(
                        "derivation-tape drift in rule '{}': expected OrWinner, found {:?}",
                        #rule_name,
                        __pgen_other,
                    ),
                };
                match __pgen_or_winner {
                    #(#winner_arms,)*
                    __pgen_idx => unreachable!(
                        "derivation-tape drift in rule '{}': winner index {} out of range",
                        #rule_name,
                        __pgen_idx,
                    ),
                }
            }
        })
    }

    /// One branch's VALUE: walk the branch body's elements in tape order —
    /// binding the transform-referenced ones as `PgenValue` locals, advancing
    /// the cursor through discard walkers for the rest — then evaluate the
    /// resolved transform in value space.
    fn dv_branch_value_expr(
        &self,
        rule_name: &str,
        branch_index: usize,
        branch_body: &ASTNode,
    ) -> Result<TokenStream> {
        let resolved = census::resolved_branch_return_ast(
            rule_name,
            branch_index,
            branch_body,
            self.annotations.as_ref(),
        );
        let shape = DvShape::of(branch_body);
        let (referenced, whole) = self.dv_referenced_elements(resolved.as_ref(), branch_body, &shape);

        let mut stmts: Vec<TokenStream> = Vec::new();
        let mut element_idents: Vec<Option<proc_macro2::Ident>> = Vec::new();
        for (idx, element) in shape.elements.iter().enumerate() {
            if whole || referenced.contains(&idx) {
                let ident = format_ident!("__pgen_el_{}", idx);
                // ⚠️ The `?` OptPresent fast path exists ONLY at true
                // SEQUENCE positions (the match pass's
                // `mtb_match_sequence_element`); a bare `?` BODY records
                // QuantCount — walk the pseudo-element through the NODE
                // builder.
                let value_expr = if shape.is_sequence {
                    self.dv_element_value_expr(element, rule_name)?
                } else {
                    self.dv_node_value_expr(element, rule_name)?
                };
                stmts.push(quote! { let #ident: PgenValue<'input> = #value_expr; });
                element_idents.push(Some(ident));
            } else {
                let discard = if shape.is_sequence {
                    self.dv_element_discard_stmts(element, rule_name)?
                } else {
                    self.dv_node_discard_stmts(element, rule_name)?
                };
                stmts.push(discard);
                element_idents.push(None);
            }
        }

        let bindings = DvBindings {
            shape: &shape,
            element_idents: &element_idents,
            whole_bound: whole,
        };
        let tail = match resolved.as_ref() {
            None | Some(UnifiedReturnAST::Passthrough) => bindings.whole_value_expr(),
            Some(ast) => self.dv_value_transform_expr(
                rule_name,
                ast,
                branch_body,
                &bindings,
                DvSentinelCtx::Content,
            )?,
        };
        Ok(quote! {
            {
                #(#stmts)*
                #tail
            }
        })
    }

    /// Which element indices does the resolved transform reference (and does
    /// it consume the whole body)? Mirrors the census demand analysis' static
    /// `$N` resolution — over-binding is safe (an extra bound value is just a
    /// computed local), under-binding is a compile error, so this stays
    /// conservative the cheap way.
    fn dv_referenced_elements(
        &self,
        resolved: Option<&UnifiedReturnAST>,
        branch_body: &ASTNode,
        shape: &DvShape<'_>,
    ) -> (std::collections::BTreeSet<usize>, bool) {
        let mut referenced = std::collections::BTreeSet::new();
        let mut whole = false;
        match resolved {
            None | Some(UnifiedReturnAST::Passthrough) => whole = true,
            Some(ast) => {
                if dv_ast_contains_passthrough(ast) {
                    whole = true;
                }
                let mut indices = std::collections::BTreeSet::new();
                census::collect_positional_indices(ast, &mut indices);
                for index in indices {
                    if index == 0 {
                        continue;
                    }
                    match census::resolve_positional_target(branch_body, index) {
                        PositionalTarget::Element(_) => {
                            referenced.insert(index - 1);
                        }
                        PositionalTarget::WholeBody => whole = true,
                        PositionalTarget::StaticSentinel => {}
                    }
                }
            }
        }
        if whole {
            referenced.extend(0..shape.elements.len());
        }
        (referenced, whole)
    }

    // ------------------------------------------------------------------
    // Value expressions per transform AST — the AstReturnTransformer twin in
    // value space (fold-of-today's-output, computed directly).
    // ------------------------------------------------------------------

    fn dv_value_transform_expr(
        &self,
        rule_name: &str,
        ast: &UnifiedReturnAST,
        branch_body: &ASTNode,
        bindings: &DvBindings<'_, '_>,
        ctx: DvSentinelCtx,
    ) -> Result<TokenStream> {
        use UnifiedReturnAST as U;
        Ok(match ast {
            U::PositionalRef { index } => {
                self.dv_positional_value_expr(*index, branch_body, bindings, ctx)
            }
            U::Passthrough => bindings.whole_value_expr(),
            U::StringLiteral { value } => quote! { PgenValue::Str(#value) },
            U::Identifier { name } => quote! { PgenValue::Str(#name) },
            U::NumberLiteral { value } => dv_number_value_expr(*value),
            U::BooleanLiteral { value } => quote! { PgenValue::Bool(#value) },
            U::NullLiteral => quote! { PgenValue::Null },
            U::MatchedText => quote! {
                PgenValue::Str(&parser.input[start_pos..parser.deriv_pos])
            },
            U::Object { properties } => {
                // Sorted at codegen time — the emitted pair slice is born
                // satisfying the `PgenValue::Object` invariant, exactly as
                // `generate_object_transform` emits it.
                let mut sorted: Vec<_> = properties.iter().collect();
                sorted.sort_by(|(left, _), (right, _)| left.cmp(right));
                let mut field_entries = Vec::new();
                for (key, value_ast) in sorted {
                    // The `generate_value_extraction` split: positional refs
                    // and scalar literals resolve at EXTRACTION level; any
                    // nested transform evaluates at CONTENT level and is
                    // folded.
                    let value_expr = match value_ast.as_ref() {
                        U::PositionalRef { index } => self.dv_positional_value_expr(
                            *index,
                            branch_body,
                            bindings,
                            DvSentinelCtx::Extraction,
                        ),
                        U::StringLiteral { value } => quote! { PgenValue::Str(#value) },
                        U::Identifier { name } => quote! { PgenValue::Str(#name) },
                        U::NumberLiteral { value } => dv_number_value_expr(*value),
                        U::BooleanLiteral { value } => quote! { PgenValue::Bool(#value) },
                        U::NullLiteral => quote! { PgenValue::Null },
                        nested => self.dv_value_transform_expr(
                            rule_name,
                            nested,
                            branch_body,
                            bindings,
                            DvSentinelCtx::Content,
                        )?,
                    };
                    field_entries.push(quote! { (#key, #value_expr) });
                }
                if field_entries.is_empty() {
                    quote! { PgenValue::Object(&[]) }
                } else {
                    quote! {
                        PgenValue::Object(parser.arena.alloc_shaped_pairs([
                            #(#field_entries),*
                        ]))
                    }
                }
            }
            U::Array { elements } => {
                let mut element_stmts = Vec::new();
                for element in elements {
                    match element {
                        U::Spread { base } => element_stmts.push(self.dv_spread_into_array(
                            rule_name,
                            base,
                            branch_body,
                            bindings,
                            false,
                        )?),
                        U::FlattenSpread { base } => element_stmts.push(
                            self.dv_spread_into_array(rule_name, base, branch_body, bindings, true)?,
                        ),
                        other => {
                            let value_expr = self.dv_value_transform_expr(
                                rule_name,
                                other,
                                branch_body,
                                bindings,
                                DvSentinelCtx::Content,
                            )?;
                            element_stmts.push(quote! {
                                __pgen_arr.push(#value_expr);
                            });
                        }
                    }
                }
                quote! {
                    {
                        let mut __pgen_arr: Vec<PgenValue<'input>> = Vec::new();
                        #(#element_stmts)*
                        PgenValue::Array(parser.arena.alloc_shaped_values(__pgen_arr))
                    }
                }
            }
            U::PropertyAccess { base, property } => {
                let base_expr = self.dv_value_transform_expr(
                    rule_name,
                    base,
                    branch_body,
                    bindings,
                    DvSentinelCtx::Content,
                )?;
                quote! {
                    {
                        let __pgen_base_value: PgenValue<'input> = #base_expr;
                        match __pgen_base_value {
                            PgenValue::Object(__pgen_pairs) => match __pgen_pairs
                                .binary_search_by(|(__pgen_key, _)| {
                                    __pgen_key.as_bytes().cmp(#property.as_bytes())
                                }) {
                                Ok(__pgen_found) => __pgen_pairs[__pgen_found].1,
                                Err(_) => PgenValue::Null,
                            },
                            _ => PgenValue::Null,
                        }
                    }
                }
            }
            U::Spread { base } | U::FlattenSpread { base } => {
                // Outside an array literal a flatten-spread degenerates to
                // Spread (the `generate_spread_transform` route). On a
                // statically-SPLICE base the spread is the IDENTITY under the
                // fold (`Sequence`→`Sequence` / `Quantified`→`Quantified`
                // both fold to the same Array); a statically-WRAP base folds
                // to a ONE-element array around the base value; a
                // static-sentinel base folds to a one-element sentinel array.
                match base.as_ref() {
                    U::PositionalRef { index } => {
                        if *index == 0 {
                            let sentinel = DvSentinelCtx::Content.index_zero();
                            quote! {
                                PgenValue::Array(parser.arena.alloc_shaped_values([
                                    PgenValue::Str(#sentinel),
                                ]))
                            }
                        } else {
                            let (target_node, at_seq): (&ASTNode, bool) =
                                match census::resolve_positional_target(branch_body, *index) {
                                    PositionalTarget::Element(element) => (element, true),
                                    PositionalTarget::WholeBody => (branch_body, false),
                                    PositionalTarget::StaticSentinel => {
                                        return Ok(quote! {
                                            PgenValue::Array(parser.arena.alloc_shaped_values([
                                                PgenValue::Str("<invalid_sequence_access>"),
                                            ]))
                                        });
                                    }
                                };
                            let base_value = self.dv_positional_value_expr(
                                *index,
                                branch_body,
                                bindings,
                                DvSentinelCtx::Content,
                            );
                            match census::spread_base_static_variant(target_node, at_seq) {
                                census::SpreadBaseStaticVariant::Splice => base_value,
                                census::SpreadBaseStaticVariant::Wrap => quote! {
                                    PgenValue::Array(parser.arena.alloc_shaped_values([
                                        #base_value,
                                    ]))
                                },
                                census::SpreadBaseStaticVariant::Undecidable => anyhow::bail!(
                                    "direct-value emission reached an undecidable spread base in a value branch — the vocabulary audit must have demoted it (partition drift)"
                                ),
                            }
                        }
                    }
                    _ => anyhow::bail!(
                        "direct-value emission reached a spread with a non-positional base — the vocabulary audit must have demoted it (partition drift)"
                    ),
                }
            }
            U::ArrayAccess { .. } | U::QuantifiedExtraction { .. } => anyhow::bail!(
                "direct-value emission reached an out-of-vocabulary transform (array access / quantified extraction) in rule '{rule_name}' — the vocabulary audit must have demoted it (partition drift)"
            ),
        })
    }

    /// A `$index` positional reference as a value, statically collapsed
    /// against the branch body (the `resolve_positional_target` single
    /// source).
    fn dv_positional_value_expr(
        &self,
        index: usize,
        branch_body: &ASTNode,
        bindings: &DvBindings<'_, '_>,
        ctx: DvSentinelCtx,
    ) -> TokenStream {
        if index == 0 {
            let sentinel = ctx.index_zero();
            return quote! { PgenValue::Str(#sentinel) };
        }
        match census::resolve_positional_target(branch_body, index) {
            PositionalTarget::Element(_) => bindings.element_value_expr(index - 1),
            PositionalTarget::WholeBody => bindings.whole_value_expr(),
            PositionalTarget::StaticSentinel => {
                quote! { PgenValue::Str("<invalid_sequence_access>") }
            }
        }
    }

    /// A spread / flatten-spread ARRAY element: extend the accumulator per
    /// the base's STATIC variant — SPLICE from an Array value, WRAP-push a
    /// single value (the runtime's `other` arm), or push the static sentinel.
    fn dv_spread_into_array(
        &self,
        rule_name: &str,
        base: &UnifiedReturnAST,
        branch_body: &ASTNode,
        bindings: &DvBindings<'_, '_>,
        flatten: bool,
    ) -> Result<TokenStream> {
        let UnifiedReturnAST::PositionalRef { index } = base else {
            anyhow::bail!(
                "direct-value emission reached an array spread with a non-positional base in rule '{rule_name}' — the vocabulary audit must have demoted it (partition drift)"
            );
        };
        if *index == 0 {
            let sentinel = DvSentinelCtx::Content.index_zero();
            return Ok(quote! { __pgen_arr.push(PgenValue::Str(#sentinel)); });
        }
        let (target_node, at_seq): (&ASTNode, bool) =
            match census::resolve_positional_target(branch_body, *index) {
                PositionalTarget::Element(element) => (element, true),
                PositionalTarget::WholeBody => (branch_body, false),
                PositionalTarget::StaticSentinel => {
                    return Ok(quote! {
                        __pgen_arr.push(PgenValue::Str("<invalid_sequence_access>"));
                    });
                }
            };
        let base_value = match census::resolve_positional_target(branch_body, *index) {
            PositionalTarget::Element(_) => bindings.element_value_expr(*index - 1),
            PositionalTarget::WholeBody => bindings.whole_value_expr(),
            PositionalTarget::StaticSentinel => unreachable!("handled above"),
        };
        match census::spread_base_static_variant(target_node, at_seq) {
            census::SpreadBaseStaticVariant::Wrap => {
                // The runtime's `other` arm: ONE pushed element whose fold is
                // the base value (spread wraps a `spread_element` node;
                // flatten wraps a `flatten_spread_element` node — either way
                // the fold is the single value).
                return Ok(quote! { __pgen_arr.push(#base_value); });
            }
            census::SpreadBaseStaticVariant::Undecidable => anyhow::bail!(
                "direct-value emission reached an undecidable array-spread base in rule '{rule_name}' — the vocabulary audit must have demoted it (partition drift)"
            ),
            census::SpreadBaseStaticVariant::Splice => {}
        }
        let rule_label = rule_name.to_string();
        if flatten {
            Ok(quote! {
                match #base_value {
                    PgenValue::Array(__pgen_items) => {
                        for __pgen_item in __pgen_items {
                            // `PgenValue` is `Copy` — match by value so the
                            // inner slice binds as `&[PgenValue]`, not `&&[_]`.
                            match *__pgen_item {
                                PgenValue::Array(__pgen_inner) => {
                                    for __pgen_value in __pgen_inner {
                                        __pgen_arr.push(*__pgen_value);
                                    }
                                }
                                __pgen_other => {
                                    __pgen_arr.push(__pgen_other);
                                }
                            }
                        }
                    }
                    __pgen_other => unreachable!(
                        "derivation-tape drift in rule '{}': flatten-spread base is not an Array value ({:?})",
                        #rule_label,
                        __pgen_other,
                    ),
                }
            })
        } else {
            Ok(quote! {
                match #base_value {
                    PgenValue::Array(__pgen_items) => {
                        for __pgen_item in __pgen_items {
                            __pgen_arr.push(*__pgen_item);
                        }
                    }
                    __pgen_other => unreachable!(
                        "derivation-tape drift in rule '{}': spread base is not an Array value ({:?})",
                        #rule_label,
                        __pgen_other,
                    ),
                }
            })
        }
    }

    // ------------------------------------------------------------------
    // Element / node VALUE builders — the `mtb_build_*` cursor semantics
    // mirrored in value space.
    // ------------------------------------------------------------------

    /// One SEQUENCE-POSITION element's value. The `?` fast path consumes its
    /// mandatory `OptPresent` event (absent folds to the EMPTY array — the
    /// fold of `Sequence(Vec::new())`); everything else is the node builder.
    fn dv_element_value_expr(&self, element: &ASTNode, rule_name: &str) -> Result<TokenStream> {
        match element {
            ASTNode::Quantified {
                element: inner,
                quantifier,
            } if quantifier == "?" => {
                let inner_value = self.dv_node_value_expr(inner, rule_name)?;
                Ok(quote! {
                    {
                        let __pgen_opt_present = match parser.deriv_next_event() {
                            crate::ast_pipeline::DerivEvent::OptPresent(__pgen_p) => __pgen_p,
                            __pgen_other => unreachable!(
                                "derivation-tape drift in rule '{}': expected OptPresent, found {:?}",
                                #rule_name,
                                __pgen_other,
                            ),
                        };
                        if __pgen_opt_present {
                            #inner_value
                        } else {
                            PgenValue::Array(&[])
                        }
                    }
                })
            }
            _ => self.dv_node_value_expr(element, rule_name),
        }
    }

    /// Any body construct's value at the replayed cursor.
    fn dv_node_value_expr(&self, node: &ASTNode, rule_name: &str) -> Result<TokenStream> {
        match node {
            ASTNode::Or { alternatives } => self.dv_or_value_expr(alternatives, rule_name, false),
            ASTNode::Sequence { elements } => {
                // A nested Sequence folds to the Array of ALL its element
                // values, in order.
                let mut stmts = Vec::new();
                let mut idents = Vec::new();
                for (idx, element) in elements.iter().enumerate() {
                    let ident = format_ident!("__pgen_seq_{}", idx);
                    let value_expr = self.dv_element_value_expr(element, rule_name)?;
                    stmts.push(quote! { let #ident: PgenValue<'input> = #value_expr; });
                    idents.push(ident);
                }
                if idents.is_empty() {
                    return Ok(quote! { PgenValue::Array(&[]) });
                }
                Ok(quote! {
                    {
                        #(#stmts)*
                        PgenValue::Array(parser.arena.alloc_shaped_values([#(#idents),*]))
                    }
                })
            }
            ASTNode::Atom { value } => self.dv_atom_value_expr(value, rule_name),
            ASTNode::Quantified { element, .. } => {
                // Every non-`?`-sequence-position quantified site is the
                // QuantCount loop (matching the match pass); the fold is the
                // Array of iteration values.
                let inner_value = self.dv_node_value_expr(element, rule_name)?;
                Ok(quote! {
                    {
                        let __pgen_quant_n = match parser.deriv_next_event() {
                            crate::ast_pipeline::DerivEvent::QuantCount(__pgen_n) => __pgen_n,
                            __pgen_other => unreachable!(
                                "derivation-tape drift in rule '{}': expected QuantCount, found {:?}",
                                #rule_name,
                                __pgen_other,
                            ),
                        };
                        let mut __pgen_vals: Vec<PgenValue<'input>> = Vec::new();
                        for _ in 0..__pgen_quant_n {
                            __pgen_vals.push(#inner_value);
                        }
                        PgenValue::Array(parser.arena.alloc_shaped_values(__pgen_vals))
                    }
                })
            }
            ASTNode::Lookahead { .. } => Ok(quote! { PgenValue::Array(&[]) }),
        }
    }

    /// A terminal / reference atom's value — the `mtb_build_atom_logic`
    /// mirror: identical tape-event protocol, `Str` instead of `Terminal`,
    /// and reference call-outs adapted to the callee's emission class (a
    /// node-form callee is built on the STACK and folded — the `Alternative`
    /// wrapper's arena alloc is elided).
    fn dv_atom_value_expr(&self, value: &ASTValue, rule_name: &str) -> Result<TokenStream> {
        match value {
            ASTValue::Token(parts) if parts.len() >= 2 => {
                let TokenValue::String(token_type) = &parts[0];
                let TokenValue::String(token_value) = &parts[1];
                match token_type.as_str() {
                    "quoted_string" | "number" | "probability" | "include_dir" | "include_file"
                    | "rule" => {
                        let lit_len = token_value.len();
                        if self.layout_sensitivity().terminals {
                            Ok(quote! {
                                {
                                    let __pgen_tok_start = parser.deriv_pos;
                                    parser.deriv_pos = __pgen_tok_start + #lit_len;
                                    PgenValue::Str(&parser.input[__pgen_tok_start..parser.deriv_pos])
                                }
                            })
                        } else {
                            Ok(quote! {
                                {
                                    let __pgen_tok_end = match parser.deriv_next_event() {
                                        crate::ast_pipeline::DerivEvent::TokEnd(__pgen_e) => __pgen_e,
                                        __pgen_other => unreachable!(
                                            "derivation-tape drift in rule '{}': expected TokEnd, found {:?}",
                                            #rule_name,
                                            __pgen_other,
                                        ),
                                    };
                                    parser.deriv_pos = __pgen_tok_end;
                                    PgenValue::Str(&parser.input[__pgen_tok_end - #lit_len..__pgen_tok_end])
                                }
                            })
                        }
                    }
                    "rule_reference" => {
                        if self.mtb_internal(token_value) {
                            if self.dv_value_licensed(token_value) {
                                let value_target =
                                    format_ident!("cascade_build_value_{}", token_value);
                                Ok(quote! { parser.#value_target() })
                            } else {
                                // Node-form fused callee: build on the stack,
                                // fold the content — no arena node, no
                                // `Alternative` wrapper.
                                let build_target = format_ident!("cascade_build_{}", token_value);
                                Ok(quote! {
                                    {
                                        let __pgen_child = parser.#build_target();
                                        __pgen_child.content.to_shaped_value(parser.arena)
                                    }
                                })
                            }
                        } else {
                            // Boundary / scan call-out: the side-vec node is
                            // an arena ref; fold its content (the fold of
                            // `Alternative(node)` IS the fold of the node's
                            // content).
                            Ok(quote! {
                                {
                                    let __pgen_alt_node = parser.deriv_next_boundary();
                                    parser.deriv_pos = __pgen_alt_node.span.end as usize;
                                    __pgen_alt_node.content.to_shaped_value(parser.arena)
                                }
                            })
                        }
                    }
                    "regex" => {
                        if self.cascade_rule_has_matched_text_transform(rule_name) {
                            anyhow::bail!(
                                "direct-value emission reached regex atom of rule '{rule_name}' which carries a matched-text @transform — the shared cascade gate must have excluded it"
                            );
                        }
                        let skip_leading_whitespace = !matches!(
                            rule_name,
                            "string_content_double" | "string_content_single"
                        );
                        let start_dynamic =
                            skip_leading_whitespace && !self.layout_sensitivity().regex_tokens;
                        let start_tokens = if start_dynamic {
                            quote! {
                                let __pgen_tok_start = match parser.deriv_next_event() {
                                    crate::ast_pipeline::DerivEvent::TokStart(__pgen_s) => __pgen_s,
                                    __pgen_other => unreachable!(
                                        "derivation-tape drift in rule '{}': expected TokStart, found {:?}",
                                        #rule_name,
                                        __pgen_other,
                                    ),
                                };
                            }
                        } else {
                            quote! {
                                let __pgen_tok_start = parser.deriv_pos;
                            }
                        };
                        Ok(quote! {
                            {
                                #start_tokens
                                let __pgen_tok_end = match parser.deriv_next_event() {
                                    crate::ast_pipeline::DerivEvent::TokEnd(__pgen_e) => __pgen_e,
                                    __pgen_other => unreachable!(
                                        "derivation-tape drift in rule '{}': expected TokEnd, found {:?}",
                                        #rule_name,
                                        __pgen_other,
                                    ),
                                };
                                parser.deriv_pos = __pgen_tok_end;
                                PgenValue::Str(&parser.input[__pgen_tok_start..__pgen_tok_end])
                            }
                        })
                    }
                    _ => Ok(quote! { PgenValue::Str("") }),
                }
            }
            _ => Ok(quote! { PgenValue::Str("") }),
        }
    }

    // ------------------------------------------------------------------
    // Discard walkers — advance `deriv_pos` (and the event/boundary cursors)
    // past an element without constructing its value.
    // ------------------------------------------------------------------

    /// A SEQUENCE-POSITION element in discard mode (`?` fast path handled).
    fn dv_element_discard_stmts(&self, element: &ASTNode, rule_name: &str) -> Result<TokenStream> {
        match element {
            ASTNode::Quantified {
                element: inner,
                quantifier,
            } if quantifier == "?" => {
                let inner_discard = self.dv_node_discard_stmts(inner, rule_name)?;
                Ok(quote! {
                    {
                        let __pgen_opt_present = match parser.deriv_next_event() {
                            crate::ast_pipeline::DerivEvent::OptPresent(__pgen_p) => __pgen_p,
                            __pgen_other => unreachable!(
                                "derivation-tape drift in rule '{}': expected OptPresent, found {:?}",
                                #rule_name,
                                __pgen_other,
                            ),
                        };
                        if __pgen_opt_present {
                            #inner_discard
                        }
                    }
                })
            }
            _ => self.dv_node_discard_stmts(element, rule_name),
        }
    }

    /// Any body construct in discard mode. Reference call-outs still run the
    /// callee's build (the tape must be consumed identically); terminals are
    /// pure cursor arithmetic — the common unreferenced-punctuation case pays
    /// nothing.
    fn dv_node_discard_stmts(&self, node: &ASTNode, rule_name: &str) -> Result<TokenStream> {
        match node {
            ASTNode::Or { alternatives } => {
                if alternatives.len() == 1 {
                    return self.dv_node_discard_stmts(&alternatives[0], rule_name);
                }
                let emit_first_set_guard = false;
                let mut first_set_cache: std::collections::HashMap<
                    String,
                    super::super::super::first_set::FirstSetSummary,
                > = std::collections::HashMap::new();
                if let Some(branch_byte_sets) = self.degenerate_dispatch_byte_sets(
                    alternatives,
                    rule_name,
                    emit_first_set_guard,
                    &mut first_set_cache,
                ) {
                    let mut dispatch_arms = Vec::new();
                    for (idx, alternative) in alternatives.iter().enumerate() {
                        let branch_discard = self.dv_node_discard_stmts(alternative, rule_name)?;
                        let byte_patterns = &branch_byte_sets[idx];
                        dispatch_arms.push(quote! {
                            #(#byte_patterns)|* => { #branch_discard }
                        });
                    }
                    return Ok(quote! {
                        match parser.input.as_bytes()[parser.deriv_pos] {
                            #(#dispatch_arms,)*
                            __pgen_byte => unreachable!(
                                "derivation-tape drift in rule '{}': no byte-switch arm admits committed byte {}",
                                #rule_name,
                                __pgen_byte,
                            ),
                        }
                    });
                }
                let mut winner_arms: Vec<TokenStream> = Vec::new();
                for (idx, alternative) in alternatives.iter().enumerate() {
                    let branch_discard = self.dv_node_discard_stmts(alternative, rule_name)?;
                    winner_arms.push(quote! {
                        #idx => { #branch_discard }
                    });
                }
                Ok(quote! {
                    {
                        let __pgen_or_winner = match parser.deriv_next_event() {
                            crate::ast_pipeline::DerivEvent::OrWinner(__pgen_idx) => __pgen_idx,
                            __pgen_other => unreachable!(
                                "derivation-tape drift in rule '{}': expected OrWinner, found {:?}",
                                #rule_name,
                                __pgen_other,
                            ),
                        };
                        match __pgen_or_winner {
                            #(#winner_arms,)*
                            __pgen_idx => unreachable!(
                                "derivation-tape drift in rule '{}': winner index {} out of range",
                                #rule_name,
                                __pgen_idx,
                            ),
                        }
                    }
                })
            }
            ASTNode::Sequence { elements } => {
                let mut stmts = Vec::new();
                for element in elements {
                    stmts.push(self.dv_element_discard_stmts(element, rule_name)?);
                }
                Ok(quote! { #(#stmts)* })
            }
            ASTNode::Atom { value } => self.dv_atom_discard_stmts(value, rule_name),
            ASTNode::Quantified { element, .. } => {
                let inner_discard = self.dv_node_discard_stmts(element, rule_name)?;
                Ok(quote! {
                    {
                        let __pgen_quant_n = match parser.deriv_next_event() {
                            crate::ast_pipeline::DerivEvent::QuantCount(__pgen_n) => __pgen_n,
                            __pgen_other => unreachable!(
                                "derivation-tape drift in rule '{}': expected QuantCount, found {:?}",
                                #rule_name,
                                __pgen_other,
                            ),
                        };
                        for _ in 0..__pgen_quant_n {
                            #inner_discard
                        }
                    }
                })
            }
            ASTNode::Lookahead { .. } => Ok(quote! {}),
        }
    }

    fn dv_atom_discard_stmts(&self, value: &ASTValue, rule_name: &str) -> Result<TokenStream> {
        match value {
            ASTValue::Token(parts) if parts.len() >= 2 => {
                let TokenValue::String(token_type) = &parts[0];
                let TokenValue::String(token_value) = &parts[1];
                match token_type.as_str() {
                    "quoted_string" | "number" | "probability" | "include_dir" | "include_file"
                    | "rule" => {
                        let lit_len = token_value.len();
                        if self.layout_sensitivity().terminals {
                            Ok(quote! {
                                parser.deriv_pos += #lit_len;
                            })
                        } else {
                            Ok(quote! {
                                {
                                    let __pgen_tok_end = match parser.deriv_next_event() {
                                        crate::ast_pipeline::DerivEvent::TokEnd(__pgen_e) => __pgen_e,
                                        __pgen_other => unreachable!(
                                            "derivation-tape drift in rule '{}': expected TokEnd, found {:?}",
                                            #rule_name,
                                            __pgen_other,
                                        ),
                                    };
                                    parser.deriv_pos = __pgen_tok_end;
                                }
                            })
                        }
                    }
                    "rule_reference" => {
                        if self.mtb_internal(token_value) {
                            if self.dv_value_licensed(token_value) {
                                let value_target =
                                    format_ident!("cascade_build_value_{}", token_value);
                                Ok(quote! {
                                    let _ = parser.#value_target();
                                })
                            } else {
                                let build_target = format_ident!("cascade_build_{}", token_value);
                                Ok(quote! {
                                    let _ = parser.#build_target();
                                })
                            }
                        } else {
                            Ok(quote! {
                                {
                                    let __pgen_alt_node = parser.deriv_next_boundary();
                                    parser.deriv_pos = __pgen_alt_node.span.end as usize;
                                }
                            })
                        }
                    }
                    "regex" => {
                        let skip_leading_whitespace = !matches!(
                            rule_name,
                            "string_content_double" | "string_content_single"
                        );
                        let start_dynamic =
                            skip_leading_whitespace && !self.layout_sensitivity().regex_tokens;
                        let start_consume = if start_dynamic {
                            quote! {
                                match parser.deriv_next_event() {
                                    crate::ast_pipeline::DerivEvent::TokStart(_) => {}
                                    __pgen_other => unreachable!(
                                        "derivation-tape drift in rule '{}': expected TokStart, found {:?}",
                                        #rule_name,
                                        __pgen_other,
                                    ),
                                }
                            }
                        } else {
                            quote! {}
                        };
                        Ok(quote! {
                            {
                                #start_consume
                                let __pgen_tok_end = match parser.deriv_next_event() {
                                    crate::ast_pipeline::DerivEvent::TokEnd(__pgen_e) => __pgen_e,
                                    __pgen_other => unreachable!(
                                        "derivation-tape drift in rule '{}': expected TokEnd, found {:?}",
                                        #rule_name,
                                        __pgen_other,
                                    ),
                                };
                                parser.deriv_pos = __pgen_tok_end;
                            }
                        })
                    }
                    _ => Ok(quote! {}),
                }
            }
            _ => Ok(quote! {}),
        }
    }

    // ------------------------------------------------------------------
    // In-place branch CONTENT for NODE-form rules (barrier + node_locked
    // in-place branches).
    // ------------------------------------------------------------------

    /// The branch's `ParseContent` computed in place per [`DvBranchMode`]
    /// (never called with `Verbatim`). Yields an expression block.
    pub(in super::super) fn dv_inplace_branch_content(
        &self,
        rule_name: &str,
        branch_index: usize,
        branch_body: &ASTNode,
        mode: &DvBranchMode,
    ) -> Result<TokenStream> {
        match mode {
            DvBranchMode::Verbatim => anyhow::bail!(
                "dv_inplace_branch_content called with Verbatim mode for rule '{rule_name}' — emitter wiring bug"
            ),
            DvBranchMode::InPlaceValuePure => {
                let resolved = census::resolved_branch_return_ast(
                    rule_name,
                    branch_index,
                    branch_body,
                    self.annotations.as_ref(),
                );
                let Some(ast) = resolved else {
                    anyhow::bail!(
                        "InPlaceValuePure branch {branch_index} of rule '{rule_name}' has no resolved transform — partition drift"
                    );
                };
                let shape = DvShape::of(branch_body);
                let (referenced, whole) =
                    self.dv_referenced_elements(Some(&ast), branch_body, &shape);
                let mut stmts: Vec<TokenStream> = Vec::new();
                let mut element_idents: Vec<Option<proc_macro2::Ident>> = Vec::new();
                for (idx, element) in shape.elements.iter().enumerate() {
                    if whole || referenced.contains(&idx) {
                        let ident = format_ident!("__pgen_el_{}", idx);
                        // The `?` fast path only at true sequence positions
                        // (see `dv_branch_value_expr`).
                        let value_expr = if shape.is_sequence {
                            self.dv_element_value_expr(element, rule_name)?
                        } else {
                            self.dv_node_value_expr(element, rule_name)?
                        };
                        stmts.push(quote! { let #ident: PgenValue<'input> = #value_expr; });
                        element_idents.push(Some(ident));
                    } else {
                        let discard = if shape.is_sequence {
                            self.dv_element_discard_stmts(element, rule_name)?
                        } else {
                            self.dv_node_discard_stmts(element, rule_name)?
                        };
                        stmts.push(discard);
                        element_idents.push(None);
                    }
                }
                let bindings = DvBindings {
                    shape: &shape,
                    element_idents: &element_idents,
                    whole_bound: whole,
                };
                // The fold's CONTENT constructor — `Shaped` for structured
                // folds, `Terminal` for string-ish folds — exactly as the
                // eager transform emission constructs it.
                let content_expr = match &ast {
                    UnifiedReturnAST::StringLiteral { value } => {
                        quote! { ParseContent::Terminal(#value) }
                    }
                    UnifiedReturnAST::Identifier { name } => {
                        quote! { ParseContent::Terminal(#name) }
                    }
                    UnifiedReturnAST::MatchedText => quote! {
                        ParseContent::Terminal(&parser.input[start_pos..parser.deriv_pos])
                    },
                    structured => {
                        let value_expr = self.dv_value_transform_expr(
                            rule_name,
                            structured,
                            branch_body,
                            &bindings,
                            DvSentinelCtx::Content,
                        )?;
                        quote! { ParseContent::Shaped(#value_expr) }
                    }
                };
                Ok(quote! {
                    {
                        #(#stmts)*
                        #content_expr
                    }
                })
            }
            DvBranchMode::InPlaceTransparentElement(target_idx) => {
                let shape = DvShape::of(branch_body);
                if !shape.is_sequence || *target_idx >= shape.elements.len() {
                    anyhow::bail!(
                        "InPlaceTransparentElement({target_idx}) of rule '{rule_name}' does not address a sequence element — partition drift"
                    );
                }
                let mut stmts: Vec<TokenStream> = Vec::new();
                for (idx, element) in shape.elements.iter().enumerate() {
                    if idx == *target_idx {
                        let content_stmts =
                            self.dv_element_content_binding(element, rule_name)?;
                        stmts.push(content_stmts);
                    } else {
                        stmts.push(self.dv_element_discard_stmts(element, rule_name)?);
                    }
                }
                Ok(quote! {
                    {
                        #(#stmts)*
                        __pgen_target_content
                    }
                })
            }
            DvBranchMode::InPlaceTransparentSentinel(sentinel) => {
                let shape = DvShape::of(branch_body);
                let mut stmts: Vec<TokenStream> = Vec::new();
                for element in &shape.elements {
                    let discard = if shape.is_sequence {
                        self.dv_element_discard_stmts(element, rule_name)?
                    } else {
                        self.dv_node_discard_stmts(element, rule_name)?
                    };
                    stmts.push(discard);
                }
                Ok(quote! {
                    {
                        #(#stmts)*
                        ParseContent::Terminal(#sentinel)
                    }
                })
            }
        }
    }

    /// Bind `__pgen_target_content` = the `$N` target element's CONTENT,
    /// built exactly as the verbatim element interior builds it (the same
    /// `mtb_build_*` machinery — node-form children and all) minus the
    /// element WRAPPER node the transform would have peeled and cloned
    /// through.
    fn dv_element_content_binding(
        &self,
        element: &ASTNode,
        rule_name: &str,
    ) -> Result<TokenStream> {
        match element {
            ASTNode::Quantified {
                element: inner,
                quantifier,
            } if quantifier == "?" => {
                let inner_logic = self.mtb_build_node_logic(inner, rule_name)?;
                Ok(quote! {
                    let __pgen_target_content = {
                        let __pgen_opt_present = match parser.deriv_next_event() {
                            crate::ast_pipeline::DerivEvent::OptPresent(__pgen_p) => __pgen_p,
                            __pgen_other => unreachable!(
                                "derivation-tape drift in rule '{}': expected OptPresent, found {:?}",
                                #rule_name,
                                __pgen_other,
                            ),
                        };
                        if __pgen_opt_present {
                            #inner_logic
                            result
                        } else {
                            ParseContent::Sequence(Vec::new())
                        }
                    };
                })
            }
            _ => {
                let inner_logic = self.mtb_build_node_logic(element, rule_name)?;
                Ok(quote! {
                    let __pgen_target_content = {
                        #inner_logic
                        result
                    };
                })
            }
        }
    }
}

/// Bindings available to a branch's value-transform expression.
struct DvBindings<'a, 'tree> {
    shape: &'a DvShape<'tree>,
    element_idents: &'a [Option<proc_macro2::Ident>],
    whole_bound: bool,
}

impl DvBindings<'_, '_> {
    /// The bound value of element `idx` (0-based). A missing binding is an
    /// emitter bug (the referenced-set computation is what created the
    /// bindings), so panic at CODEGEN time, not runtime.
    fn element_value_expr(&self, idx: usize) -> TokenStream {
        match self.element_idents.get(idx) {
            Some(Some(ident)) => quote! { #ident },
            _ => unreachable!(
                "direct-value emitter: element {idx} referenced but not bound — referenced-set computation drifted"
            ),
        }
    }

    /// The whole body's value: the single pseudo-element for non-sequence
    /// bodies, the Array of all elements for sequences (empty ⇒ the empty
    /// array).
    fn whole_value_expr(&self) -> TokenStream {
        if !self.shape.is_sequence {
            return self.element_value_expr(0);
        }
        if self.shape.elements.is_empty() {
            return quote! { PgenValue::Array(&[]) };
        }
        assert!(
            self.whole_bound,
            "direct-value emitter: whole-body value requested but elements were not all bound"
        );
        let idents: Vec<_> = self
            .element_idents
            .iter()
            .map(|ident| {
                ident
                    .as_ref()
                    .expect("whole_bound implies every element is bound")
            })
            .collect();
        quote! {
            PgenValue::Array(parser.arena.alloc_shaped_values([#(#idents),*]))
        }
    }
}

/// Does this transform AST contain `$text` anywhere?
fn dv_ast_contains_matched_text(ast: &UnifiedReturnAST) -> bool {
    use UnifiedReturnAST as U;
    match ast {
        U::MatchedText => true,
        U::Array { elements } => elements.iter().any(dv_ast_contains_matched_text),
        U::Object { properties } => properties.values().any(|v| dv_ast_contains_matched_text(v)),
        U::Spread { base } | U::FlattenSpread { base } => dv_ast_contains_matched_text(base),
        U::PropertyAccess { base, .. } | U::QuantifiedExtraction { base, .. } => {
            dv_ast_contains_matched_text(base)
        }
        U::ArrayAccess { base, index } => {
            dv_ast_contains_matched_text(base) || dv_ast_contains_matched_text(index)
        }
        _ => false,
    }
}

/// Does this transform AST contain a bare `Passthrough` anywhere (⇒ the whole
/// body value must be bound)?
fn dv_ast_contains_passthrough(ast: &UnifiedReturnAST) -> bool {
    use UnifiedReturnAST as U;
    match ast {
        U::Passthrough => true,
        U::Array { elements } => elements.iter().any(dv_ast_contains_passthrough),
        U::Object { properties } => properties.values().any(|v| dv_ast_contains_passthrough(v)),
        U::Spread { base } | U::FlattenSpread { base } => dv_ast_contains_passthrough(base),
        U::PropertyAccess { base, .. } | U::QuantifiedExtraction { base, .. } => {
            dv_ast_contains_passthrough(base)
        }
        U::ArrayAccess { base, index } => {
            dv_ast_contains_passthrough(base) || dv_ast_contains_passthrough(index)
        }
        _ => false,
    }
}

/// The value form of a number literal — integer-preserving exactly as the
/// eager emission (`ParseContent::Shaped(PgenValue::Int/from_f64)`) and the
/// extraction path emit it.
fn dv_number_value_expr(value: f64) -> TokenStream {
    if value.is_finite() && value.fract() == 0.0 && value >= i64::MIN as f64 && value <= i64::MAX as f64
    {
        let int_value = value as i64;
        quote! { PgenValue::Int(#int_value) }
    } else {
        quote! { PgenValue::from_f64(#value) }
    }
}
