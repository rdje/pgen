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
                let num_str = value.to_string();
                Ok(quote! { ParseContent::Terminal(#num_str) })
            }
            UnifiedReturnAST::BooleanLiteral { value } => {
                let bool_str = value.to_string();
                Ok(quote! { ParseContent::Terminal(#bool_str) })
            }
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
        }
    }

    fn parse_capture_expr(var_ref: &str) -> TokenStream {
        match syn::parse_str::<syn::Expr>(var_ref) {
            Ok(expr) => quote! { #expr },
            Err(_) => quote! { ParseContent::Terminal("<invalid_capture_ref>") },
        }
    }

    /// Convert a runtime `ParseContent` into a `String` for object-key/value
    /// extraction. The non-typed fallback uses `to_json_value().to_string()` so
    /// structured shapes serialise as JSON rather than as Rust Debug output.
    fn parse_content_to_string(content_expr: TokenStream) -> TokenStream {
        quote! {
            {
                let __pgen_content = #content_expr;
                match __pgen_content {
                    ParseContent::Terminal(s) => s.to_string(),
                    ParseContent::TransformedTerminal(s) => s,
                    ParseContent::Json(value) => match value {
                        serde_json::Value::String(s) => s,
                        other => other.to_string(),
                    },
                    other => other.to_json_value().to_string(),
                }
            }
        }
    }

    /// Convert a runtime `ParseContent` into a `serde_json::Value` without
    /// going through string-encoded intermediates. Used by object-literal
    /// field extraction to keep typed shapes typed end-to-end.
    fn parse_content_to_json_value(content_expr: TokenStream) -> TokenStream {
        quote! {
            {
                let __pgen_content = #content_expr;
                __pgen_content.to_json_value()
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
            if element_index == 0 {
                return Ok(quote! {
                    {
                        let __pgen_base = (#base_expr).clone();
                        match __pgen_base {
                            ParseContent::Sequence(elements) if !elements.is_empty() => {
                                elements[0usize].content.clone()
                            }
                            ParseContent::Quantified(elements, _) if !elements.is_empty() => {
                                elements[0usize].content.clone()
                            }
                            ParseContent::Alternative(node) => node.content.clone(),
                            other => other,
                        }
                    }
                });
            }
            return Ok(quote! {
                {
                    let __pgen_base = (#base_expr).clone();
                    match __pgen_base {
                        ParseContent::Sequence(elements) if elements.len() > #element_index => {
                            elements[#element_index].content.clone()
                        }
                        ParseContent::Quantified(elements, _) if elements.len() > #element_index => {
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
                                array_elements.push(ParseNode {
                                    rule_name: "spread_element",
                                    content: other,
                                    span: 0..0,
                                });
                            }
                        }
                    });
                }
                _ => {
                    let elem_code = Self::generate_transform(element, captured_vars, "")?;
                    let elem_name = format!("element_{}", idx);
                    element_codes.push(quote! {
                        array_elements.push(ParseNode {
                            rule_name: #elem_name,
                            content: #elem_code,
                            span: 0..0,
                        });
                    });
                }
            }
        }

        Ok(quote! {
            {
                let mut array_elements = Vec::new();
                #(#element_codes)*
                ParseContent::Sequence(array_elements)
            }
        })
    }

    /// Generate object transformation. Builds a typed `serde_json::Value::Object`
    /// at runtime and wraps it as `ParseContent::Json(value)`. This replaces an
    /// older path that built a `serde_json::Value`, serialised it to a `String`,
    /// and stuffed the string into `ParseContent::TransformedTerminal(String)` —
    /// which forced any subsequent property/array access to deserialise the
    /// string back, look up the field, and re-stringify per access.
    fn generate_object_transform(
        properties: &std::collections::HashMap<String, Box<UnifiedReturnAST>>,
        captured_vars: &[String],
    ) -> Result<TokenStream> {
        let mut field_assignments = Vec::new();

        // Stabilize field emission order so generated parser code is deterministic
        // across process runs (HashMap iteration order is randomized).
        let mut sorted_properties: Vec<_> = properties.iter().collect();
        sorted_properties.sort_by(|(left_key, _), (right_key, _)| left_key.cmp(right_key));

        for (key, value_ast) in sorted_properties {
            let value_code = Self::generate_value_extraction(value_ast, captured_vars)?;
            field_assignments.push(quote! {
                __pgen_obj.insert(#key.to_string(), #value_code);
            });
        }

        Ok(quote! {
            {
                let mut __pgen_obj = serde_json::Map::new();
                #(#field_assignments)*
                ParseContent::Json(serde_json::Value::Object(__pgen_obj))
            }
        })
    }

    /// Generate code to extract value for object property
    /// Build a TokenStream that evaluates to a `serde_json::Value` at runtime.
    /// Used to populate object-literal field values without going through a
    /// stringified intermediate. For positional refs the captured `ParseContent`
    /// is converted via `to_json_value()`; for primitive literals the matching
    /// `serde_json::Value::*` constructor is emitted directly; for nested
    /// transforms the inner `ParseContent` is again converted via
    /// `to_json_value()`. No `serde_json::to_string`, no `from_str`.
    fn generate_value_extraction(
        ast: &UnifiedReturnAST,
        captured_vars: &[String],
    ) -> Result<TokenStream> {
        match ast {
            UnifiedReturnAST::PositionalRef { index } => {
                if *index == 0 {
                    return Ok(quote! {
                        serde_json::Value::String("<invalid_ref_0>".to_string())
                    });
                }

                if captured_vars.len() == 1 {
                    let base_expr = Self::parse_capture_expr(&captured_vars[0]);
                    let element_index = index - 1;
                    if element_index == 0 {
                        return Ok(Self::parse_content_to_json_value(quote! {
                            {
                                let __pgen_base = (#base_expr).clone();
                                match __pgen_base {
                                    ParseContent::Sequence(elements) if !elements.is_empty() => {
                                        elements[0usize].content.clone()
                                    }
                                    ParseContent::Quantified(elements, _) if !elements.is_empty() => {
                                        elements[0usize].content.clone()
                                    }
                                    ParseContent::Alternative(node) => node.content.clone(),
                                    other => other,
                                }
                            }
                        }));
                    }
                    return Ok(Self::parse_content_to_json_value(quote! {
                        {
                            let __pgen_base = (#base_expr).clone();
                            match __pgen_base {
                                ParseContent::Sequence(elements) if elements.len() > #element_index => {
                                    elements[#element_index].content.clone()
                                }
                                ParseContent::Quantified(elements, _) if elements.len() > #element_index => {
                                    elements[#element_index].content.clone()
                                }
                                _ => ParseContent::Terminal("<invalid_sequence_access>"),
                            }
                        }
                    }));
                }

                if *index <= captured_vars.len() {
                    let expr = Self::parse_capture_expr(&captured_vars[index - 1]);
                    return Ok(Self::parse_content_to_json_value(quote! { (#expr).clone() }));
                }

                let invalid_index = *index;
                Ok(quote! {
                    serde_json::Value::String(format!("<invalid_ref_{}>", #invalid_index))
                })
            }
            UnifiedReturnAST::StringLiteral { value } => {
                Ok(quote! { serde_json::Value::String(#value.to_string()) })
            }
            UnifiedReturnAST::NumberLiteral { value } => {
                Ok(quote! { serde_json::Value::from(#value) })
            }
            UnifiedReturnAST::BooleanLiteral { value } => {
                Ok(quote! { serde_json::Value::Bool(#value) })
            }
            UnifiedReturnAST::Identifier { name } => {
                Ok(quote! { serde_json::Value::String(#name.to_string()) })
            }
            _ => {
                let nested = Self::generate_transform(ast, captured_vars, "")?;
                Ok(Self::parse_content_to_json_value(quote! { #nested }))
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
                other => ParseContent::Sequence(vec![ParseNode {
                    rule_name: "spread_base",
                    content: other,
                    span: 0..0,
                }]),
            }
        })
    }

    /// Generate property access transformation. Operates on the typed
    /// `serde_json::Value` carrier directly — no `from_str`/`to_string`
    /// roundtrip. The previous path stringified the base, deserialised the
    /// string back into a `serde_json::Value`, looked up the property, and
    /// re-stringified before wrapping again as `TransformedTerminal(String)`.
    fn generate_property_access(
        base: &UnifiedReturnAST,
        property: &str,
        captured_vars: &[String],
    ) -> Result<TokenStream> {
        let base_code = Self::generate_transform(base, captured_vars, "")?;

        Ok(quote! {
            {
                let __pgen_base: ParseContent = #base_code;
                let __pgen_value = __pgen_base.to_json_value();
                let __pgen_prop = __pgen_value
                    .get(#property)
                    .cloned()
                    .unwrap_or(serde_json::Value::Null);
                ParseContent::Json(__pgen_prop)
            }
        })
    }

    /// Generate array access transformation. Handles both the legacy
    /// `Sequence`/`Quantified` carrier (used by raw grammar captures) and the
    /// typed `Json(Value::Array)` carrier produced by chained property access
    /// or array-literal transforms.
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
                    ParseContent::Json(value) => {
                        let __pgen_elem = match value {
                            serde_json::Value::Array(ref arr) if arr.len() > #index_code => {
                                arr[#index_code].clone()
                            }
                            _ => serde_json::Value::Null,
                        };
                        ParseContent::Json(__pgen_elem)
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
        let base_expr = match base {
            UnifiedReturnAST::PositionalRef { index }
                if *index > 0 && *index <= captured_vars.len() =>
            {
                Self::parse_capture_expr(&captured_vars[index - 1])
            }
            _ => return Ok(quote! { ParseContent::Terminal("<invalid_extraction_base>") }),
        };

        let extraction_idx = match target {
            ExtractionTarget::Index(idx) => *idx,
            ExtractionTarget::First => 0,
            ExtractionTarget::Last => {
                return Ok(quote! {
                    {
                        let __pgen_base = (#base_expr).clone();
                        match __pgen_base {
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
                let __pgen_base = (#base_expr).clone();
                match __pgen_base {
                    ParseContent::Quantified(elements, _) => {
                        let extracted: Vec<ParseNode> = elements
                            .iter()
                            .filter_map(|node| {
                                match &node.content {
                                    ParseContent::Sequence(subelems) if subelems.len() > #extraction_idx => {
                                        Some(subelems[#extraction_idx].clone())
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

    /// Phase 2 typed-carrier contract: object literal annotations must emit a
    /// `ParseContent::Json(serde_json::Value::Object(...))` constructor with no
    /// `serde_json::to_string` and no `ParseContent::TransformedTerminal`
    /// wrapping. The earlier path serialised the assembled `Value` to a `String`
    /// before wrapping; that stringification is what produced the per-property
    /// serialise/parse/serialise roundtrip the user flagged.
    #[test]
    fn object_literal_transform_emits_typed_json_carrier_without_stringify() {
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
            rendered.contains("ParseContent :: Json (serde_json :: Value :: Object")
                || rendered.contains("ParseContent::Json(serde_json::Value::Object"),
            "object literal must emit typed Json/Object carrier; rendered = {}",
            rendered
        );
        assert!(
            !rendered.contains("serde_json :: to_string")
                && !rendered.contains("serde_json::to_string"),
            "object literal must NOT serialise to a String; rendered = {}",
            rendered
        );
        assert!(
            !rendered.contains("ParseContent :: TransformedTerminal")
                && !rendered.contains("ParseContent::TransformedTerminal"),
            "object literal must NOT wrap as TransformedTerminal(String); rendered = {}",
            rendered
        );
    }

    /// Phase 2 typed-carrier contract: property access on a base value must
    /// operate on the typed `serde_json::Value` directly. The earlier path
    /// stringified the base, then `serde_json::from_str`'d the string back to a
    /// `serde_json::Value`, then re-stringified the looked-up property.
    #[test]
    fn property_access_transform_avoids_serialise_parse_serialise_roundtrip() {
        let base = UnifiedReturnAST::PositionalRef { index: 1 };
        let captured_vars = vec!["sequence_elements[0]".to_string()];
        let stream =
            AstReturnTransformer::generate_property_access(&base, "field", &captured_vars)
                .unwrap();
        let rendered = render(stream);

        assert!(
            rendered.contains("to_json_value"),
            "property access must call ParseContent::to_json_value; rendered = {}",
            rendered
        );
        assert!(
            !rendered.contains("from_str") && !rendered.contains("from_str ::"),
            "property access must NOT deserialise a string; rendered = {}",
            rendered
        );
        assert!(
            !rendered.contains("ParseContent :: TransformedTerminal")
                && !rendered.contains("ParseContent::TransformedTerminal"),
            "property access must NOT wrap as TransformedTerminal(String); rendered = {}",
            rendered
        );
        assert!(
            rendered.contains("ParseContent :: Json")
                || rendered.contains("ParseContent::Json"),
            "property access must wrap the result as ParseContent::Json; rendered = {}",
            rendered
        );
    }

    /// Phase 2 typed-carrier contract: `to_json_value` is the carrier-agnostic
    /// helper that lets transform paths cross between the legacy
    /// `Sequence`/`Quantified`/`Terminal` carriers and the typed `Json` carrier
    /// without re-stringifying. This pins the helper's translation rules.
    #[test]
    fn parse_content_to_json_value_translates_each_variant() {
        use crate::ast_pipeline::{ParseContent, ParseNode};

        // Terminal -> Value::String
        let t = ParseContent::Terminal("abc");
        assert_eq!(t.to_json_value(), serde_json::Value::String("abc".into()));

        // Json -> identity
        let v = serde_json::json!({"k": 1});
        let j = ParseContent::Json(v.clone());
        assert_eq!(j.to_json_value(), v);

        // TransformedTerminal carrying valid JSON -> parsed value
        let parsed = ParseContent::TransformedTerminal("{\"k\":1}".to_string());
        assert_eq!(parsed.to_json_value(), serde_json::json!({"k": 1}));

        // TransformedTerminal carrying non-JSON text -> Value::String
        let raw = ParseContent::TransformedTerminal("plain".to_string());
        assert_eq!(raw.to_json_value(), serde_json::Value::String("plain".into()));

        // Sequence -> Value::Array
        let seq = ParseContent::Sequence(vec![ParseNode {
            rule_name: "x",
            content: ParseContent::Terminal("a"),
            span: 0..1,
        }]);
        assert_eq!(
            seq.to_json_value(),
            serde_json::json!([serde_json::Value::String("a".into())])
        );
    }
}
