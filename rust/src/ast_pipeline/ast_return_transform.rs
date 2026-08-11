// AST-based Return Annotation Transformer
// Uses syn/quote to generate proper return value transformations

use crate::ast_pipeline::unified_return_ast::{ExtractionTarget, UnifiedReturnAST};
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::quote;

/// Generate AST-based return transformation code
pub struct AstReturnTransformer;

impl AstReturnTransformer {
    /// Generate transformation code from UnifiedReturnAST
    pub fn generate_transform(
        ast: &UnifiedReturnAST,
        captured_vars: &[String],
        _rule_name: &str,
    ) -> Result<TokenStream> {
        match ast {
            UnifiedReturnAST::PositionalRef { index } => {
                Self::generate_positional_ref(*index, captured_vars)
            }
            UnifiedReturnAST::StringLiteral { value } => {
                Ok(quote! { ParseContent::Terminal(#value) })
            }
            UnifiedReturnAST::NumberLiteral { value } => {
                // Emit the typed number on the arena carrier, integer-
                // preserving when the literal has no fractional part (matches
                // the value-extraction path's behaviour). `PgenValue::Int`
                // serializes via the same `serialize_i64` path as
                // `serde_json::Value::from(i64)`, and `from_f64` mirrors
                // `Value::from(f64)`'s non-finite → Null rule, so the emitted
                // bytes are unchanged.
                if value.is_finite() && value.fract() == 0.0
                    && *value >= i64::MIN as f64
                    && *value <= i64::MAX as f64
                {
                    let int_value = *value as i64;
                    Ok(quote! { ParseContent::Shaped(PgenValue::Int(#int_value)) })
                } else {
                    let v = *value;
                    Ok(quote! { ParseContent::Shaped(PgenValue::from_f64(#v)) })
                }
            }
            UnifiedReturnAST::BooleanLiteral { value } => {
                // Emit the typed boolean directly (on the arena carrier). The
                // string-Terminal `"true"`/`"false"` mis-typing this replaced
                // was PGEN-RGX-0076.
                let bool_lit = *value;
                Ok(quote! { ParseContent::Shaped(PgenValue::Bool(#bool_lit)) })
            }
            UnifiedReturnAST::NullLiteral => Ok(quote! {
                ParseContent::Shaped(PgenValue::Null)
            }),
            UnifiedReturnAST::Identifier { name } => Ok(quote! { ParseContent::Terminal(#name) }),
            UnifiedReturnAST::Array { elements } => {
                Self::generate_array_transform(elements, captured_vars)
            }
            UnifiedReturnAST::Object { properties } => {
                Self::generate_object_transform(properties, captured_vars)
            }
            UnifiedReturnAST::Spread { base } => {
                Self::generate_spread_transform(base, captured_vars)
            }
            UnifiedReturnAST::FlattenSpread { base } => {
                // Outside an array literal a flatten-spread degenerates to
                // the same shape-preserving identity as Spread — the
                // recursive-unwrap step is meaningful only when each pushed
                // child is appended into a parent accumulator (which happens
                // in `generate_array_transform`'s Spread/FlattenSpread arm).
                Self::generate_spread_transform(base, captured_vars)
            }
            UnifiedReturnAST::PropertyAccess { base, property } => {
                Self::generate_property_access(base, property, captured_vars)
            }
            UnifiedReturnAST::ArrayAccess { base, index } => {
                Self::generate_array_access(base, index, captured_vars)
            }
            UnifiedReturnAST::QuantifiedExtraction { base, target } => {
                Self::generate_quantified_extraction(base, target, captured_vars)
            }
            UnifiedReturnAST::Passthrough => Self::generate_passthrough(captured_vars),
            // REGEX-SELF-HOSTING.3: `$text` / `$0` — the rule's full matched source text as one
            // string Terminal. At the splice point the rule's `start_pos` (entry, bound by the parse
            // logic) and `parser.position` (body end) are in scope, and the final node is
            // `span: start_pos..end_pos`, so this slice is exactly the matched substring. The native
            // equivalent of a `/.../`-with-capture; lets a quantified char-payload (`octal_digit+`)
            // emit `"777"` instead of the structured Quantified shape.
            UnifiedReturnAST::MatchedText => Ok(quote! {
                ParseContent::Terminal(&parser.input[start_pos..parser.position])
            }),
            // ENGINE-UNIVERSAL-SERVICES.8 — an LR-eliminated rule returns the AST
            // its author declared, not the eliminator's chain record. The fold
            // itself lives in ONE place (`lr_chain_fold`) that this emitter, the
            // cascade emitter and the interpreter all call.
            UnifiedReturnAST::LrChainFold {
                initial,
                suffixes,
                specs,
            } => Self::generate_lr_chain_fold(initial, suffixes, specs, captured_vars),
        }
    }

    /// Emit the left-recursion chain fold (`ENGINE-UNIVERSAL-SERVICES.8`).
    ///
    /// The per-alternative templates are embedded as their canonical (key-sorted,
    /// therefore byte-deterministic) JSON and parsed **once per process** behind a
    /// `OnceLock`, so a parse pays one relaxed atomic load rather than a JSON
    /// decode — and the giant `wrapper_specs` string literal that used to be a
    /// FIELD OF EVERY EMITTED CHAIN VALUE disappears from the AST entirely.
    fn generate_lr_chain_fold(
        initial: &UnifiedReturnAST,
        suffixes: &UnifiedReturnAST,
        specs: &[crate::ast_pipeline::unified_return_ast::LrChainWrapperSpec],
        captured_vars: &[String],
    ) -> Result<TokenStream> {
        let initial_code = Self::generate_transform(initial, captured_vars, "")?;
        let suffixes_code = Self::generate_transform(suffixes, captured_vars, "")?;
        let specs_json = serde_json::to_string(specs)?;
        Ok(quote! {
            {
                static __PGEN_LR_CHAIN_SPECS: std::sync::OnceLock<
                    Vec<crate::ast_pipeline::unified_return_ast::LrChainWrapperSpec>,
                > = std::sync::OnceLock::new();
                let __pgen_lr_specs = __PGEN_LR_CHAIN_SPECS.get_or_init(|| {
                    crate::ast_pipeline::lr_chain_fold::parse_specs(#specs_json)
                });
                let __pgen_lr_initial = {
                    let __pgen_lr_content = #initial_code;
                    __pgen_lr_content.to_shaped_value(parser.arena)
                };
                let __pgen_lr_suffixes = {
                    let __pgen_lr_content = #suffixes_code;
                    __pgen_lr_content.to_shaped_value(parser.arena)
                };
                ParseContent::Shaped(crate::ast_pipeline::lr_chain_fold::fold_lr_chain(
                    parser.arena,
                    __pgen_lr_initial,
                    __pgen_lr_suffixes,
                    __pgen_lr_specs.as_slice(),
                ))
            }
        })
    }

    fn parse_capture_expr(var_ref: &str) -> TokenStream {
        match syn::parse_str::<syn::Expr>(var_ref) {
            Ok(expr) => quote! { #expr },
            Err(_) => quote! { ParseContent::Terminal("<invalid_capture_ref>") },
        }
    }

    /// Convert a runtime `ParseContent` into a [`PgenValue`] without going
    /// through string-encoded or owned-`serde_json::Value` intermediates. Used
    /// by object-literal field extraction and property access to keep typed
    /// shapes typed end-to-end on the arena carrier (RGX-0078.5.i.7
    /// REPRESENTATION, `PGEN-RGX-0078-0105` — replaced the
    /// `parse_content_to_json_value` emission that fed the profile-dominant
    /// `to_json_value` deep-clone).
    fn parse_content_to_shaped_value(content_expr: TokenStream) -> TokenStream {
        quote! {
            {
                let __pgen_content = #content_expr;
                __pgen_content.to_shaped_value(parser.arena)
            }
        }
    }

    /// Generate code for positional reference ($1, $2, etc.)
    fn generate_positional_ref(index: usize, captured_vars: &[String]) -> Result<TokenStream> {
        if index == 0 {
            return Ok(quote! {
                ParseContent::Terminal("<invalid_positional_ref>")
            });
        }

        if captured_vars.len() == 1 {
            let base_expr = Self::parse_capture_expr(&captured_vars[0]);
            let element_index = index - 1;
            // PGEN-RGX-0073 Optim #12: match by reference instead of
            // cloning the whole content first. The previous shape did
            // `let __pgen_base = (base).clone(); match __pgen_base { ... }`
            // — a deep clone of the entire ParseContent (Sequence's
            // Vec<ParseNode>, etc.) only to drop most of it after picking
            // out one child via `elements[N].content.clone()`. Matching
            // on `&base` skips the outer clone entirely; only the picked
            // child's content gets the inner clone. For Sequence/
            // Quantified-shaped results in deep grammars, this halves
            // the clone work per `$N` extraction. Samply post-Optim-#11
            // showed Vec::clone + ParseNode::clone summing to ~4.4% of
            // self-time on the regex parser.
            if element_index == 0 {
                // PGEN-RGX-0075: do NOT peel `elements[0]` from a Quantified
                // base when the rule body has a single capture position. For a
                // rule like `RULE = X+ -> [$1**]`, $1 must resolve to the
                // whole Quantified (every iteration), not just the first
                // match. This mirrors the implicit `-> $1` default policy
                // (annotation-system.md) which excludes Quantified bodies for
                // exactly the same reason: $1 on a Quantified is "the whole
                // capture group", not its first element. Multi-element
                // Sequence wrapping (the codegen's synthetic packaging) is
                // still peeled here because that wrap is artificial.
                return Ok(quote! {
                    {
                        match &#base_expr {
                            ParseContent::Sequence(elements) if !elements.is_empty() => {
                                elements[0usize].content.clone()
                            }
                            ParseContent::Alternative(node) => node.content.clone(),
                            other => other.clone(),
                        }
                    }
                });
            }
            // Multi-positional access ($2, $3, ...): the rule body is a
            // multi-element Sequence; index into the matched slot. A slot may
            // itself be a Quantified, in which case `elements[N].content`
            // gives the whole Quantified content (passing through correctly
            // — same Quantified-is-the-whole-capture rule).
            return Ok(quote! {
                {
                    match &#base_expr {
                        ParseContent::Sequence(elements) if elements.len() > #element_index => {
                            elements[#element_index].content.clone()
                        }
                        _ => ParseContent::Terminal("<invalid_sequence_access>"),
                    }
                }
            });
        }

        if index > captured_vars.len() {
            return Ok(quote! {
                ParseContent::Terminal("<invalid_positional_ref>")
            });
        }

        let expr = Self::parse_capture_expr(&captured_vars[index - 1]);
        Ok(quote! { (#expr).clone() })
    }

    /// Generate array transformation
    fn generate_array_transform(
        elements: &[UnifiedReturnAST],
        captured_vars: &[String],
    ) -> Result<TokenStream> {
        let mut element_codes = Vec::new();

        for (idx, element) in elements.iter().enumerate() {
            match element {
                UnifiedReturnAST::Spread { base } => {
                    let base_code = Self::generate_transform(base, captured_vars, "")?;
                    element_codes.push(quote! {
                        match #base_code {
                            ParseContent::Sequence(nodes) => {
                                for node in nodes {
                                    array_elements.push(node);
                                }
                            }
                            ParseContent::Quantified(nodes, _) => {
                                for node in nodes {
                                    array_elements.push(node);
                                }
                            }
                            other => {
                                // RGX-0078.5.d.4.i — arena-alloc the wrapper node.
                                array_elements.push(parser.arena.alloc(ParseNode {
                                    rule_name: &"spread_element",
                                    content: other,
                                    span: Span::new(0, 0),
                                }));
                            }
                        }
                    });
                }
                UnifiedReturnAST::FlattenSpread { base } => {
                    // `[$N**]` — like Spread, but for each pushed child node,
                    // if its `content` is itself an array-shaped value,
                    // unwrap one level and push that wrapper's children
                    // inline. Used when a child rule may produce either a
                    // single value OR an array of values that should appear
                    // flat under the parent's accumulator.
                    //
                    // The "array-shaped" check covers three runtime variants:
                    //   - ParseContent::Sequence(nodes)      — codegen-produced
                    //     packaging of a multi-element body.
                    //   - ParseContent::Quantified(nodes, _) — codegen-produced
                    //     packaging of a `?`/`*`/`+` Quantified body.
                    //   - ParseContent::Shaped(PgenValue::Array(_)) — typed
                    //     shaped array produced by an upstream annotation like
                    //     `child = ... -> [$2**, ...]`. PGEN-RGX-0077 was the
                    //     missing-arm regression on this route's `Json(Array)`
                    //     predecessor: pre-fix, a typed-array pushed-child fell
                    //     into `other_content` and the whole array got wrapped
                    //     as ONE element instead of each member spreading
                    //     inline. Surfaced by `\Qab*\E{2,}` family —
                    //     `piece_quoted_run_quantified` emits a typed array of
                    //     pieces, and the parent `concatenation = piece+ ->
                    //     [$1**]` failed to spread. (`-0105`: the arm now
                    //     matches `Shaped`; emitted constructions reference
                    //     the retired `Json` variant nowhere, so its `-0106`
                    //     lib-side retirement never touched artifacts.)
                    let base_code = Self::generate_transform(base, captured_vars, "")?;
                    element_codes.push(quote! {
                        // Inner helper: peel `Alternative` one level recursively
                        // until the underlying content is reached. The codegen
                        // wraps Or-rule and rule-reference branch results in
                        // `Alternative(boxed_inner)`, so a piece node from
                        // `concatenation = piece+ -> [$1**]` typically arrives as
                        // `Alternative(piece_inner_node)` — we want to inspect
                        // the inner content for the Sequence/Quantified/Json-Array
                        // unwrap decision below. Without this peel, every
                        // Alternative-wrapped child silently falls into the
                        // `other_content` "push as-is" arm and the spread is
                        // lost — that's the PGEN-RGX-0077 regression on the
                        // `piece_quoted_run_quantified` route.
                        fn __pgen_peel_alternative<'__pgen_input>(
                            content: ParseContent<'__pgen_input>,
                        ) -> ParseContent<'__pgen_input> {
                            let mut current = content;
                            // RGX-0078.5.d.4.i — `node` is an arena `&` borrow, so
                            // clone its content out (shallow: children are `Copy`
                            // refs) rather than moving it.
                            while let ParseContent::Alternative(node) = current {
                                current = node.content.clone();
                            }
                            current
                        }
                        match #base_code {
                            ParseContent::Sequence(nodes) | ParseContent::Quantified(nodes, _) => {
                                for node in nodes {
                                    let span_for_inherit = node.span;
                                    let rule_name_for_inherit = node.rule_name;
                                    // RGX-0078.5.d.4.i — clone content out of the arena ref.
                                    let peeled = __pgen_peel_alternative(node.content.clone());
                                    match peeled {
                                        ParseContent::Sequence(inner_nodes)
                                        | ParseContent::Quantified(inner_nodes, _) => {
                                            for inner_node in inner_nodes {
                                                array_elements.push(inner_node);
                                            }
                                        }
                                        ParseContent::Shaped(PgenValue::Array(values)) => {
                                            // Typed shaped array (post-collapse from a
                                            // child annotation that built [$N**, ...]
                                            // or similar). Spread each value as its
                                            // own ParseNode so consumers see N flat
                                            // entries, not [<N entries>]. The values
                                            // are `Copy` arena refs — no clone.
                                            for value in values {
                                                // RGX-0078.5.d.4.i — arena-alloc.
                                                array_elements.push(parser.arena.alloc(ParseNode {
                                                    rule_name: rule_name_for_inherit,
                                                    content: ParseContent::Shaped(*value),
                                                    span: span_for_inherit,
                                                }));
                                            }
                                        }
                                        other_content => {
                                            // RGX-0078.5.d.4.i — arena-alloc.
                                            array_elements.push(parser.arena.alloc(ParseNode {
                                                rule_name: rule_name_for_inherit,
                                                content: other_content,
                                                span: span_for_inherit,
                                            }));
                                        }
                                    }
                                }
                            }
                            other => {
                                // RGX-0078.5.d.4.i — arena-alloc.
                                array_elements.push(parser.arena.alloc(ParseNode {
                                    rule_name: &"flatten_spread_element",
                                    content: other,
                                    span: Span::new(0, 0),
                                }));
                            }
                        }
                    });
                }
                _ => {
                    let elem_code = Self::generate_transform(element, captured_vars, "")?;
                    let elem_name = format!("element_{}", idx);
                    element_codes.push(quote! {
                        // RGX-0078.5.d.4.i — arena-alloc the array element.
                        array_elements.push(parser.arena.alloc(ParseNode {
                            rule_name: &#elem_name,
                            content: #elem_code,
                            span: Span::new(0, 0),
                        }));
                    });
                }
            }
        }

        Ok(quote! {
            {
                // RGX-0078.5.d.4.i — explicit element type so the `&mut` from
                // `arena.alloc` coerces to the shared `&'input` the Vec holds.
                let mut array_elements: Vec<&'input ParseNode<'input>> = Vec::new();
                #(#element_codes)*
                ParseContent::Sequence(array_elements)
            }
        })
    }

    /// Generate object transformation. Builds a typed `PgenValue::Object` on
    /// the node arena and wraps it as `ParseContent::Shaped(value)`
    /// (RGX-0078.5.i.7 REPRESENTATION, `PGEN-RGX-0078-0105`). This replaces
    /// the `serde_json::Map::new()` + per-key `String` + `BTreeMap`-insert
    /// emission that RE-PROFILE #13 pinned as the dominant committed-value
    /// compute (416 object-build sites in the regex artifact alone).
    ///
    /// The emitted shape is a stack ARRAY of `(key, value)` tuples handed to
    /// one `alloc_shaped_pairs` bump: template keys are static string
    /// literals, unique by construction (`HashMap` source) and sorted HERE at
    /// codegen time by the same byte order `BTreeMap` iterates in — so the
    /// pair slice is born satisfying the `PgenValue::Object` sorted/deduped
    /// invariant with zero runtime map machinery. Evaluating every element
    /// expression inside the array literal BEFORE the arena call also
    /// satisfies `alloc_extend`'s no-reentrant-allocation constraint (nested
    /// object/array values allocate while being evaluated, not during the
    /// outer bump).
    fn generate_object_transform(
        properties: &std::collections::HashMap<String, Box<UnifiedReturnAST>>,
        captured_vars: &[String],
    ) -> Result<TokenStream> {
        // Sorting also stabilizes field emission order so generated parser
        // code is deterministic across process runs (HashMap iteration order
        // is randomized). `String::cmp` = byte order = `BTreeMap`'s.
        let mut sorted_properties: Vec<_> = properties.iter().collect();
        sorted_properties.sort_by(|(left_key, _), (right_key, _)| left_key.cmp(right_key));

        let mut field_entries = Vec::new();
        for (key, value_ast) in sorted_properties {
            let value_code = Self::generate_value_extraction(value_ast, captured_vars)?;
            field_entries.push(quote! {
                (#key, #value_code)
            });
        }

        if field_entries.is_empty() {
            return Ok(quote! { ParseContent::Shaped(PgenValue::Object(&[])) });
        }

        Ok(quote! {
            ParseContent::Shaped(PgenValue::Object(parser.arena.alloc_shaped_pairs([
                #(#field_entries),*
            ])))
        })
    }

    /// Generate code to extract value for object property
    /// Build a TokenStream that evaluates to a [`PgenValue`] at runtime
    /// (RGX-0078.5.i.7 REPRESENTATION — previously a `serde_json::Value`).
    /// Used to populate object-literal field values without going through a
    /// stringified intermediate. For positional refs the captured
    /// `ParseContent` is converted via `to_shaped_value()` (a `Copy` for
    /// already-shaped content, a zero-copy `Str` borrow for terminals — where
    /// `to_json_value()` deep-cloned); for primitive literals the matching
    /// `PgenValue` constructor is emitted directly; for nested transforms the
    /// inner `ParseContent` is again converted via `to_shaped_value()`.
    /// No `serde_json::to_string`, no `from_str`, no owned tree.
    fn generate_value_extraction(
        ast: &UnifiedReturnAST,
        captured_vars: &[String],
    ) -> Result<TokenStream> {
        match ast {
            UnifiedReturnAST::PositionalRef { index } => {
                if *index == 0 {
                    return Ok(quote! {
                        PgenValue::Str("<invalid_ref_0>")
                    });
                }

                if captured_vars.len() == 1 {
                    let base_expr = Self::parse_capture_expr(&captured_vars[0]);
                    let element_index = index - 1;
                    // PGEN-RGX-0075: same fix as in `generate_positional_ref` —
                    // never peel `elements[0]` from a Quantified base. $1 on a
                    // Quantified body is "the whole capture group". Multi-
                    // element Sequence wrapping (artificial codegen packaging)
                    // is still peeled.
                    //
                    // PGEN-RGX-0078-0154 (V1 M1): the extraction result is a
                    // VALUE, and `to_shaped_value` takes `&self` — so each arm
                    // converts THROUGH THE BORROW. The previous shape cloned
                    // the picked content into an owned temporary first; for
                    // Sequence/Quantified receivers that clone was a real Vec
                    // malloc+memcpy per `$N` (1,889/parse on the corpus-MAX
                    // cell), existing only because the emission routed through
                    // `parse_content_to_shaped_value`'s owned binding.
                    if element_index == 0 {
                        return Ok(quote! {
                            {
                                match &#base_expr {
                                    ParseContent::Sequence(elements) if !elements.is_empty() => {
                                        elements[0usize].content.to_shaped_value(parser.arena)
                                    }
                                    ParseContent::Alternative(node) => {
                                        node.content.to_shaped_value(parser.arena)
                                    }
                                    other => other.to_shaped_value(parser.arena),
                                }
                            }
                        });
                    }
                    return Ok(quote! {
                        {
                            match &#base_expr {
                                ParseContent::Sequence(elements) if elements.len() > #element_index => {
                                    elements[#element_index].content.to_shaped_value(parser.arena)
                                }
                                _ => PgenValue::Str("<invalid_sequence_access>"),
                            }
                        }
                    });
                }

                if *index <= captured_vars.len() {
                    let expr = Self::parse_capture_expr(&captured_vars[index - 1]);
                    return Ok(quote! { (#expr).to_shaped_value(parser.arena) });
                }

                // The index is a codegen-time constant, so the sentinel label
                // is rendered HERE — the artifact carries a static literal
                // where it used to carry a runtime `format!` (byte-identical
                // output, one less allocation).
                let invalid_label = format!("<invalid_ref_{}>", index);
                Ok(quote! {
                    PgenValue::Str(#invalid_label)
                })
            }
            UnifiedReturnAST::StringLiteral { value } => {
                Ok(quote! { PgenValue::Str(#value) })
            }
            UnifiedReturnAST::NumberLiteral { value } => {
                // Preserve integer typing when the literal has no fractional
                // part. `PgenValue::Float(0.0)` serialises as `0.0`;
                // `PgenValue::Int(0)` serialises as `0` (the serde parity the
                // step-1 oracle tests pin). Most typed-AST shapes (e.g.
                // min/max counts in counted_quantifier_body) want integers and
                // would otherwise mix `0.0` (literal) with `2` (from a
                // `digits` capture) in the same field.
                if value.is_finite() && value.fract() == 0.0
                    && *value >= i64::MIN as f64
                    && *value <= i64::MAX as f64
                {
                    let int_value = *value as i64;
                    Ok(quote! { PgenValue::Int(#int_value) })
                } else {
                    Ok(quote! { PgenValue::from_f64(#value) })
                }
            }
            UnifiedReturnAST::BooleanLiteral { value } => {
                Ok(quote! { PgenValue::Bool(#value) })
            }
            UnifiedReturnAST::NullLiteral => {
                Ok(quote! { PgenValue::Null })
            }
            UnifiedReturnAST::Identifier { name } => {
                Ok(quote! { PgenValue::Str(#name) })
            }
            _ => {
                let nested = Self::generate_transform(ast, captured_vars, "")?;
                Ok(Self::parse_content_to_shaped_value(quote! { #nested }))
            }
        }
    }

    /// Generate spread operator transformation
    fn generate_spread_transform(
        base: &UnifiedReturnAST,
        captured_vars: &[String],
    ) -> Result<TokenStream> {
        let base_code = Self::generate_transform(base, captured_vars, "")?;

        Ok(quote! {
            match #base_code {
                ParseContent::Sequence(elements) => ParseContent::Sequence(elements),
                ParseContent::Quantified(elements, q) => ParseContent::Quantified(elements, q),
                // RGX-0078.5.d.4.i — arena-alloc the degenerate single wrapper.
                other => ParseContent::Sequence(vec![parser.arena.alloc(ParseNode {
                    rule_name: &"spread_base",
                    content: other,
                    span: Span::new(0, 0),
                })]),
            }
        })
    }

    /// Generate property access transformation. Operates on the typed
    /// [`PgenValue`] carrier directly (RGX-0078.5.i.7 REPRESENTATION) — no
    /// `to_json_value()` owned-tree conversion, no clone: the pair slice is
    /// key-sorted (the `PgenValue::Object` invariant), so a binary search is
    /// the byte-exact equivalent of `serde_json::Map::get`, and the looked-up
    /// value is a `Copy`. A non-object base or a missing key yields `Null`,
    /// exactly like `Value::get(str)`.
    fn generate_property_access(
        base: &UnifiedReturnAST,
        property: &str,
        captured_vars: &[String],
    ) -> Result<TokenStream> {
        let base_code = Self::generate_transform(base, captured_vars, "")?;

        Ok(quote! {
            {
                let __pgen_base: ParseContent = #base_code;
                let __pgen_value = __pgen_base.to_shaped_value(parser.arena);
                let __pgen_prop = match __pgen_value {
                    PgenValue::Object(pairs) => match pairs
                        .binary_search_by(|(key, _)| key.as_bytes().cmp(#property.as_bytes()))
                    {
                        Ok(found) => pairs[found].1,
                        Err(_) => PgenValue::Null,
                    },
                    _ => PgenValue::Null,
                };
                ParseContent::Shaped(__pgen_prop)
            }
        })
    }

    /// Generate array access transformation. Handles both the legacy
    /// `Sequence`/`Quantified` carrier (used by raw grammar captures) and the
    /// typed `Shaped(PgenValue::Array)` carrier produced by chained property
    /// access or array-literal transforms.
    fn generate_array_access(
        base: &UnifiedReturnAST,
        index: &UnifiedReturnAST,
        captured_vars: &[String],
    ) -> Result<TokenStream> {
        let base_code = Self::generate_transform(base, captured_vars, "")?;

        let index_code = match index {
            UnifiedReturnAST::NumberLiteral { value } => {
                let idx = *value as usize;
                quote! { #idx }
            }
            _ => quote! { 0usize },
        };

        Ok(quote! {
            {
                let __pgen_base: ParseContent = #base_code;
                match __pgen_base {
                    ParseContent::Sequence(elements) if elements.len() > #index_code => {
                        elements[#index_code].content.clone()
                    }
                    ParseContent::Quantified(elements, _) if elements.len() > #index_code => {
                        elements[#index_code].content.clone()
                    }
                    ParseContent::Shaped(value) => {
                        let __pgen_elem = match value {
                            PgenValue::Array(items) if items.len() > #index_code => {
                                items[#index_code]
                            }
                            _ => PgenValue::Null,
                        };
                        ParseContent::Shaped(__pgen_elem)
                    }
                    _ => ParseContent::Terminal("<invalid_array_access>"),
                }
            }
        })
    }

    /// Generate quantified extraction ($1*, $2+, etc.)
    fn generate_quantified_extraction(
        base: &UnifiedReturnAST,
        target: &ExtractionTarget,
        captured_vars: &[String],
    ) -> Result<TokenStream> {
        // The base expression must resolve to a `ParseContent` whose Quantified
        // / Sequence elements we extract from. Two captured-vars conventions
        // are in play:
        //   * Multi-capture: `captured_vars` is the rule's per-element list
        //     (`["sequence_elements[0]", "sequence_elements[1]", ...]`).
        //     `$N` resolves to `captured_vars[N-1]` directly.
        //   * Single-capture: `captured_vars == ["result"]` is the shadow-
        //     rebind convention used by `generate_rule_method`'s post_parse
        //     transform path. `$N` must index into the single capture's
        //     Sequence/Quantified content the same way `generate_value_extraction`
        //     does.
        // Without the single-capture branch, any non-Or rule whose annotation
        // contains a `$N::M*` form for `N >= 2` falls through to the
        // `<invalid_extraction_base>` runtime fallback because
        // `captured_vars.len() == 1 < N`.
        let base_expr = match base {
            UnifiedReturnAST::PositionalRef { index } if *index > 0 => {
                if captured_vars.len() == 1 {
                    let single = Self::parse_capture_expr(&captured_vars[0]);
                    let element_index = *index - 1;
                    // PGEN-RGX-0075: same fix as elsewhere — never peel
                    // `elements[0]` from a Quantified base. $1 on a Quantified
                    // body is the whole capture group; for `$1*`/`$1+` etc.
                    // the embedded extraction operator iterates the whole
                    // Quantified directly.
                    if element_index == 0 {
                        quote! {
                            {
                                match &#single {
                                    ParseContent::Sequence(elements) if !elements.is_empty() => {
                                        elements[0usize].content.clone()
                                    }
                                    ParseContent::Alternative(node) => node.content.clone(),
                                    other => other.clone(),
                                }
                            }
                        }
                    } else {
                        quote! {
                            {
                                match &#single {
                                    ParseContent::Sequence(elements) if elements.len() > #element_index => {
                                        elements[#element_index].content.clone()
                                    }
                                    _ => ParseContent::Terminal("<invalid_extraction_base>"),
                                }
                            }
                        }
                    }
                } else if *index <= captured_vars.len() {
                    Self::parse_capture_expr(&captured_vars[*index - 1])
                } else {
                    return Ok(quote! { ParseContent::Terminal("<invalid_extraction_base>") });
                }
            }
            _ => return Ok(quote! { ParseContent::Terminal("<invalid_extraction_base>") }),
        };

        let extraction_idx = match target {
            ExtractionTarget::Index(idx) => *idx,
            ExtractionTarget::First => 0,
            ExtractionTarget::Last => {
                return Ok(quote! {
                    {
                        // Optim #12: match by reference; only clone the last element's content.
                        match &#base_expr {
                            ParseContent::Quantified(elements, _) if !elements.is_empty() => {
                                elements.last().unwrap().content.clone()
                            }
                            _ => ParseContent::Terminal("<no_last_element>"),
                        }
                    }
                });
            }
        };

        Ok(quote! {
            {
                // Optim #12: match by reference; the inner clone of each
                // extracted ParseNode is unavoidable but the outer
                // ParseContent::clone() is not.
                match &#base_expr {
                    ParseContent::Quantified(elements, _) => {
                        // RGX-0078.5.d.4.i — children are `&'input` refs; the
                        // extracted vector holds the `Copy` refs directly (no clone).
                        let extracted: Vec<&ParseNode<'input>> = elements
                            .iter()
                            .filter_map(|node| {
                                match &node.content {
                                    ParseContent::Sequence(subelems) if subelems.len() > #extraction_idx => {
                                        Some(subelems[#extraction_idx])
                                    }
                                    _ => None,
                                }
                            })
                            .collect();
                        ParseContent::Sequence(extracted)
                    }
                    _ => ParseContent::Terminal("<not_quantified>"),
                }
            }
        })
    }

    /// Generate passthrough transformation (default behavior)
    fn generate_passthrough(captured_vars: &[String]) -> Result<TokenStream> {
        if captured_vars.is_empty() {
            return Ok(quote! { ParseContent::Terminal("") });
        }

        let var_ref = if captured_vars.len() == 1 {
            &captured_vars[0]
        } else {
            captured_vars.last().unwrap()
        };
        let expr = Self::parse_capture_expr(var_ref);
        Ok(quote! { (#expr).clone() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast_pipeline::unified_return_ast::UnifiedReturnAST;

    fn render(stream: TokenStream) -> String {
        stream.to_string()
    }

    /// REGEX-SELF-HOSTING.3: `$text` parses to `MatchedText` and codegen to a matched-span string
    /// Terminal — no Rust `regex`, no structural shape. (`$0` alias deferred — see the task tree.)
    #[test]
    fn matched_text_dollar_text_emits_span_terminal() {
        let logger = crate::ast_pipeline::NoOpLogger;
        for src in ["$text"] {
            let ast = UnifiedReturnAST::parse_bootstrap(src, &logger)
                .unwrap_or_else(|e| panic!("{src:?} should parse: {e}"));
            assert!(
                matches!(ast, UnifiedReturnAST::MatchedText),
                "{src:?} should parse to MatchedText, got {ast:?}"
            );
            let rendered = render(
                AstReturnTransformer::generate_transform(&ast, &["result".to_string()], "r")
                    .expect("transform should generate"),
            );
            assert!(
                rendered.contains("ParseContent :: Terminal")
                    && rendered.contains("parser . input")
                    && rendered.contains("start_pos")
                    && rendered.contains("parser . position"),
                "{src:?} should emit the matched-span Terminal, got: {rendered}"
            );
            assert!(
                !rendered.contains("Regex") && !rendered.contains("match_regex"),
                "{src:?} matched-text codegen must not use Rust regex, got: {rendered}"
            );
        }
    }

    /// Typed-carrier contract (`-0105` REPRESENTATION vintage): object literal
    /// annotations must emit a `ParseContent::Shaped(PgenValue::Object(...))`
    /// constructor over ONE `alloc_shaped_pairs` arena bump, with the static
    /// keys pre-sorted at codegen time — and none of the retired machinery:
    /// no `serde_json::Map`, no key `.to_string()`, no `ParseContent::Json`
    /// reference at all (the `Json` variant was retired lib-side at `-0106`),
    /// no `serde_json::to_string`, no `TransformedTerminal` wrapping.
    #[test]
    fn object_literal_transform_emits_shaped_carrier_with_presorted_static_keys() {
        let mut props: std::collections::HashMap<String, Box<UnifiedReturnAST>> =
            std::collections::HashMap::new();
        props.insert(
            "type".to_string(),
            Box::new(UnifiedReturnAST::StringLiteral {
                value: "regex".to_string(),
            }),
        );
        props.insert(
            "pattern".to_string(),
            Box::new(UnifiedReturnAST::PositionalRef { index: 1 }),
        );

        let captured_vars = vec!["sequence_elements[0]".to_string()];
        let stream =
            AstReturnTransformer::generate_object_transform(&props, &captured_vars).unwrap();
        let rendered = render(stream);

        assert!(
            rendered.contains("ParseContent :: Shaped (PgenValue :: Object"),
            "object literal must emit the Shaped/Object carrier; rendered = {}",
            rendered
        );
        assert!(
            rendered.contains("alloc_shaped_pairs"),
            "object literal must build through one alloc_shaped_pairs bump; rendered = {}",
            rendered
        );
        // Keys sorted at codegen time: "pattern" precedes "type" in the array.
        let pattern_at = rendered.find("\"pattern\"").expect("pattern key emitted");
        let type_at = rendered.find("\"type\"").expect("type key emitted");
        assert!(
            pattern_at < type_at,
            "static keys must be emitted pre-sorted (BTreeMap byte order); rendered = {}",
            rendered
        );
        for forbidden in [
            "serde_json :: Map",
            "ParseContent :: Json",
            "serde_json :: to_string",
            "ParseContent :: TransformedTerminal",
        ] {
            assert!(
                !rendered.contains(forbidden),
                "object literal must not emit {forbidden}; rendered = {rendered}"
            );
        }
    }

    /// Typed-carrier contract (`-0105` REPRESENTATION vintage): property
    /// access on a base value must operate on the typed [`PgenValue`] carrier
    /// directly — `to_shaped_value` + sorted-slice binary search — with no
    /// `to_json_value` owned-tree conversion, no string deserialisation, and
    /// no `ParseContent::Json` reference.
    #[test]
    fn property_access_transform_operates_on_the_shaped_carrier() {
        let base = UnifiedReturnAST::PositionalRef { index: 1 };
        let captured_vars = vec!["sequence_elements[0]".to_string()];
        let stream =
            AstReturnTransformer::generate_property_access(&base, "field", &captured_vars)
                .unwrap();
        let rendered = render(stream);

        assert!(
            rendered.contains("to_shaped_value"),
            "property access must call ParseContent::to_shaped_value; rendered = {}",
            rendered
        );
        assert!(
            rendered.contains("binary_search_by"),
            "property access must look up via the sorted-pair binary search; rendered = {}",
            rendered
        );
        assert!(
            rendered.contains("ParseContent :: Shaped"),
            "property access must wrap the result as ParseContent::Shaped; rendered = {}",
            rendered
        );
        for forbidden in [
            "to_json_value",
            "from_str",
            "ParseContent :: TransformedTerminal",
            "ParseContent :: Json",
        ] {
            assert!(
                !rendered.contains(forbidden),
                "property access must not emit {forbidden}; rendered = {rendered}"
            );
        }
    }

    /// Phase 2 typed-carrier contract: `to_json_value` is the carrier-agnostic
    /// helper that lets transform paths cross between the legacy
    /// `Sequence`/`Quantified`/`Terminal` carriers and the typed `Shaped`
    /// carrier without re-stringifying. This pins the helper's translation rules.
    #[test]
    fn parse_content_to_json_value_translates_each_variant() {
        use crate::ast_pipeline::{ParseContent, ParseNode};

        // Terminal -> Value::String
        let t = ParseContent::Terminal("abc");
        assert_eq!(t.to_json_value(), serde_json::Value::String("abc".into()));

        // Shaped -> the owned Value the Serialize mirror guarantees
        let s = ParseContent::Shaped(crate::ast_pipeline::PgenValue::Int(1));
        assert_eq!(s.to_json_value(), serde_json::json!(1));

        // TransformedTerminal carrying valid JSON -> parsed value
        let parsed = ParseContent::TransformedTerminal("{\"k\":1}".to_string());
        assert_eq!(parsed.to_json_value(), serde_json::json!({"k": 1}));

        // TransformedTerminal carrying non-JSON text -> Value::String
        let raw = ParseContent::TransformedTerminal("plain".to_string());
        assert_eq!(raw.to_json_value(), serde_json::Value::String("plain".into()));

        // Sequence -> Value::Array
        // RGX-0078.5.d.4.i — children are arena `&'input` refs.
        let arena = crate::ast_pipeline::NodeArena::new();
        let seq = ParseContent::Sequence(vec![arena.alloc(ParseNode {
            rule_name: &"x",
            content: ParseContent::Terminal("a"),
            span: crate::ast_pipeline::Span::new(0, 1),
        })]);
        assert_eq!(
            seq.to_json_value(),
            serde_json::json!([serde_json::Value::String("a".into())])
        );
    }

    /// `-0105` REPRESENTATION: `to_shaped_value` is the arena twin of
    /// `to_json_value` — for every carrier variant the two conversions must
    /// yield the same logical JSON value (hence identical serialized bytes).
    #[test]
    fn parse_content_to_shaped_value_mirrors_to_json_value_per_variant() {
        use crate::ast_pipeline::{NodeArena, ParseContent, ParseNode, PgenValue};

        let arena = NodeArena::new();
        let inner = arena.alloc(ParseNode {
            rule_name: &"x",
            content: ParseContent::Terminal("a"),
            span: crate::ast_pipeline::Span::new(0, 1),
        });
        let shaped_items = arena.alloc_shaped_values([PgenValue::Int(1), PgenValue::Str("s")]);
        let contents: Vec<ParseContent> = vec![
            ParseContent::Terminal("abc"),
            ParseContent::TransformedTerminal("{\"k\":1}".to_string()),
            ParseContent::TransformedTerminal("18446744073709551615".to_string()),
            ParseContent::TransformedTerminal("plain".to_string()),
            ParseContent::Shaped(PgenValue::Array(shaped_items)),
            ParseContent::Sequence(vec![&*inner]),
            ParseContent::Alternative(&*inner),
        ];
        for content in &contents {
            assert_eq!(
                content.to_shaped_value(&arena).to_serde_value(),
                content.to_json_value(),
                "to_shaped_value diverged from to_json_value for {content:?}"
            );
        }
        // The zero-copy terminal contract: no rendered-string interning, a
        // plain input borrow.
        assert_eq!(
            ParseContent::Terminal("abc").to_shaped_value(&arena),
            PgenValue::Str("abc")
        );
    }
}
