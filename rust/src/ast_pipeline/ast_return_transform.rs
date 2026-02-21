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
            UnifiedReturnAST::Identifier { name } => {
                Ok(quote! { ParseContent::Terminal(#name) })
            }
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

    fn parse_content_to_string(content_expr: TokenStream) -> TokenStream {
        quote! {
            {
                let __pgen_content = #content_expr;
                match __pgen_content {
                    ParseContent::Terminal(s) => s.to_string(),
                    ParseContent::TransformedTerminal(s) => s,
                    ParseContent::Alternative(node) => {
                        match node.content {
                            ParseContent::Terminal(s) => s.to_string(),
                            ParseContent::TransformedTerminal(s) => s,
                            other => format!("{:?}", other),
                        }
                    }
                    other => format!("{:?}", other),
                }
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
                        ParseContent::Alternative(node) if #element_index == 0usize => node.content.clone(),
                        other if #element_index == 0usize => other,
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

    /// Generate object transformation using JSON
    fn generate_object_transform(
        properties: &std::collections::HashMap<String, Box<UnifiedReturnAST>>,
        captured_vars: &[String],
    ) -> Result<TokenStream> {
        let mut field_assignments = Vec::new();

        for (key, value_ast) in properties {
            let value_code = Self::generate_value_extraction(value_ast, captured_vars)?;
            field_assignments.push(quote! {
                json_obj[#key] = serde_json::json!((#value_code));
            });
        }

        Ok(quote! {
            {
                let mut json_obj = serde_json::json!({});
                #(#field_assignments)*
                let json_str = serde_json::to_string(&json_obj)
                    .unwrap_or_else(|_| "{}".to_string());
                ParseContent::TransformedTerminal(json_str)
            }
        })
    }

    /// Generate code to extract value for object property
    fn generate_value_extraction(
        ast: &UnifiedReturnAST,
        captured_vars: &[String],
    ) -> Result<TokenStream> {
        match ast {
            UnifiedReturnAST::PositionalRef { index } => {
                if *index == 0 {
                    return Ok(quote! { "<invalid_ref_0>".to_string() });
                }

                if captured_vars.len() == 1 {
                    let base_expr = Self::parse_capture_expr(&captured_vars[0]);
                    let element_index = index - 1;
                    return Ok(Self::parse_content_to_string(quote! {
                        {
                            let __pgen_base = (#base_expr).clone();
                            match __pgen_base {
                                ParseContent::Sequence(elements) if elements.len() > #element_index => {
                                    elements[#element_index].content.clone()
                                }
                                ParseContent::Quantified(elements, _) if elements.len() > #element_index => {
                                    elements[#element_index].content.clone()
                                }
                                ParseContent::Alternative(node) if #element_index == 0usize => node.content.clone(),
                                other if #element_index == 0usize => other,
                                _ => ParseContent::Terminal("<invalid_sequence_access>"),
                            }
                        }
                    }));
                }

                if *index <= captured_vars.len() {
                    let expr = Self::parse_capture_expr(&captured_vars[index - 1]);
                    return Ok(Self::parse_content_to_string(quote! { (#expr).clone() }));
                }

                Ok(quote! { format!("<invalid_ref_{}>", #index) })
            }
            UnifiedReturnAST::StringLiteral { value } => Ok(quote! { #value }),
            UnifiedReturnAST::NumberLiteral { value } => Ok(quote! { #value }),
            UnifiedReturnAST::BooleanLiteral { value } => Ok(quote! { #value }),
            UnifiedReturnAST::Identifier { name } => Ok(quote! { #name }),
            _ => {
                let nested = Self::generate_transform(ast, captured_vars, "")?;
                Ok(Self::parse_content_to_string(quote! { #nested }))
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

    /// Generate property access transformation
    fn generate_property_access(
        base: &UnifiedReturnAST,
        property: &str,
        captured_vars: &[String],
    ) -> Result<TokenStream> {
        let base_code = Self::generate_transform(base, captured_vars, "")?;
        let json_source = Self::parse_content_to_string(quote! { #base_code });

        Ok(quote! {
            {
                let __pgen_json_source = #json_source;
                if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&__pgen_json_source) {
                    if let Some(prop_val) = json_val.get(#property) {
                        ParseContent::TransformedTerminal(prop_val.to_string())
                    } else {
                        ParseContent::Terminal("<missing_property>")
                    }
                } else {
                    ParseContent::Terminal("<invalid_json>")
                }
            }
        })
    }

    /// Generate array access transformation
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
            match #base_code {
                ParseContent::Sequence(ref elements) if elements.len() > #index_code => {
                    elements[#index_code].content.clone()
                }
                ParseContent::Quantified(ref elements, _) if elements.len() > #index_code => {
                    elements[#index_code].content.clone()
                }
                _ => ParseContent::Terminal("<invalid_array_access>"),
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
