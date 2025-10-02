// AST-based Return Annotation Transformer
// Uses syn/quote to generate proper return value transformations

use quote::{quote, ToTokens};
use proc_macro2::TokenStream;
use syn::parse_quote;
use anyhow::Result;
use crate::ast_pipeline::unified_return_ast::{UnifiedReturnAST, ExtractionTarget};

/// Generate AST-based return transformation code
pub struct AstReturnTransformer;

impl AstReturnTransformer {
    /// Generate transformation code from UnifiedReturnAST
    pub fn generate_transform(
        ast: &UnifiedReturnAST,
        captured_vars: &[String],
        rule_name: &str,
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
            UnifiedReturnAST::Passthrough => {
                Self::generate_passthrough(captured_vars)
            }
        }
    }
    
    /// Generate code for positional reference ($1, $2, etc.)
    fn generate_positional_ref(index: usize, captured_vars: &[String]) -> Result<TokenStream> {
        // Special case: if we have a single "result" variable, assume it might be a sequence
        // and try to extract the element at the requested index
        if captured_vars.len() == 1 && captured_vars[0] == "result" {
            let element_index = index - 1; // Convert from 1-based to 0-based
            return Ok(quote! {
                match &result {
                    ParseContent::Sequence(elements) if elements.len() > #element_index => {
                        elements[#element_index].content.clone()
                    }
                    _ => ParseContent::Terminal("<invalid_sequence_access>")
                }
            });
        }
        
        if index == 0 || index > captured_vars.len() {
            return Ok(quote! { 
                ParseContent::Terminal("<invalid_positional_ref>") 
            });
        }
        
        let var_ref = &captured_vars[index - 1];
        
        // Check if this is a sequence element reference
        if var_ref.starts_with("sequence_elements[") {
            // Extract from sequence element
            Ok(quote! {
                match &#var_ref.content {
                    ParseContent::Terminal(s) => ParseContent::Terminal(s),
                    ParseContent::Alternative(node) => node.content.clone(),
                    other => other.clone()
                }
            })
        } else if var_ref.starts_with("match &branch_result") {
            // This is a complex extraction expression for sequences
            // Parse it to generate proper code
            Ok(quote! {
                #var_ref.content.clone()
            })
        } else {
            // Direct reference
            Ok(quote! {
                match &#var_ref {
                    ParseContent::Sequence(elements) if elements.len() > 0 => {
                        elements[0].content.clone()
                    }
                    other => other.clone()
                }
            })
        }
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
                    // Handle spread operator in array
                    let base_code = Self::generate_transform(base, captured_vars, "")?;
                    element_codes.push(quote! {
                        // Spread elements
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
                    // Regular element
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
                json_obj[#key] = serde_json::json!(#value_code);
            });
        }
        
        Ok(quote! {
            {
                let mut json_obj = serde_json::json!({});
                #(#field_assignments)*
                let json_str = serde_json::to_string(&json_obj)
                    .unwrap_or_else(|_| "{}".to_string());
                ParseContent::Terminal(&json_str)
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
                // Special case: if we have a single "result" variable, assume it might be a sequence
                if captured_vars.len() == 1 && captured_vars[0] == "result" {
                    let element_index = index - 1; // Convert from 1-based to 0-based
                    return Ok(quote! {
                        match &result {
                            ParseContent::Sequence(elements) if elements.len() > #element_index => {
                                match &elements[#element_index].content {
                                    ParseContent::Terminal(s) => s.to_string(),
                                    ParseContent::Alternative(node) => {
                                        match &node.content {
                                            ParseContent::Terminal(s) => s.to_string(),
                                            _ => format!("{:?}", node.content)
                                        }
                                    }
                                    _ => format!("{:?}", elements[#element_index].content)
                                }
                            }
                            _ => "<invalid_sequence_access>".to_string()
                        }
                    });
                }
                
                if *index > 0 && *index <= captured_vars.len() {
                    let var_ref = &captured_vars[index - 1];
                    
                    // Check for complex match expression
                    if var_ref.starts_with("match &branch_result") {
                        // This is a complex extraction - use it as-is
                        let var_tokens: TokenStream = var_ref.parse()
                            .map_err(|e| anyhow::anyhow!("Failed to parse var ref: {}", e))?;
                        Ok(quote! {
                            match #var_tokens {
                                ParseContent::Terminal(s) => s.to_string(),
                                ParseContent::Alternative(node) => {
                                    match &node.content {
                                        ParseContent::Terminal(s) => s.to_string(),
                                        _ => format!("{:?}", node.content)
                                    }
                                }
                                other => format!("{:?}", other)
                            }
                        })
                    } else if var_ref.starts_with("sequence_elements[") {
                        // Extract from sequence element
                        Ok(quote! {
                            match &#var_ref.content {
                                ParseContent::Terminal(s) => s.to_string(),
                                ParseContent::Alternative(node) => {
                                    match &node.content {
                                        ParseContent::Terminal(s) => s.to_string(),
                                        _ => format!("{:?}", node.content)
                                    }
                                }
                                _ => format!("{:?}", #var_ref.content)
                            }
                        })
                    } else {
                        // Simple variable reference
                        Ok(quote! {
                            match &#var_ref {
                                ParseContent::Terminal(s) => s.to_string(),
                                _ => format!("{:?}", #var_ref)
                            }
                        })
                    }
                } else {
                    Ok(quote! { format!("<invalid_ref_{}>", #index) })
                }
            }
            UnifiedReturnAST::StringLiteral { value } => {
                Ok(quote! { #value })
            }
            UnifiedReturnAST::NumberLiteral { value } => {
                Ok(quote! { #value })
            }
            UnifiedReturnAST::BooleanLiteral { value } => {
                Ok(quote! { #value })
            }
            _ => {
                // For complex nested values
                let nested = Self::generate_transform(ast, captured_vars, "")?;
                Ok(quote! {
                    match #nested {
                        ParseContent::Terminal(s) => s.to_string(),
                        _ => format!("{:?}", #nested)
                    }
                })
            }
        }
    }
    
    /// Generate spread operator transformation
    fn generate_spread_transform(
        base: &Box<UnifiedReturnAST>,
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
                }])
            }
        })
    }
    
    /// Generate property access transformation
    fn generate_property_access(
        base: &Box<UnifiedReturnAST>,
        property: &str,
        captured_vars: &[String],
    ) -> Result<TokenStream> {
        let base_code = Self::generate_transform(base, captured_vars, "")?;
        
        // For now, return a placeholder - would need runtime reflection
        Ok(quote! {
            {
                // Property access: .#property
                // This would require runtime JSON parsing
                match #base_code {
                    ParseContent::Terminal(json_str) => {
                        if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(json_str) {
                            if let Some(prop_val) = json_val.get(#property) {
                                ParseContent::Terminal(&prop_val.to_string())
                            } else {
                                ParseContent::Terminal("<missing_property>")
                            }
                        } else {
                            ParseContent::Terminal("<invalid_json>")
                        }
                    }
                    _ => ParseContent::Terminal("<not_an_object>")
                }
            }
        })
    }
    
    /// Generate array access transformation
    fn generate_array_access(
        base: &Box<UnifiedReturnAST>,
        index: &Box<UnifiedReturnAST>,
        captured_vars: &[String],
    ) -> Result<TokenStream> {
        let base_code = Self::generate_transform(base, captured_vars, "")?;
        
        // Get index value
        let index_code = match index.as_ref() {
            UnifiedReturnAST::NumberLiteral { value } => {
                let idx = *value as usize;
                quote! { #idx }
            }
            _ => {
                // Dynamic index - would need runtime evaluation
                quote! { 0usize }
            }
        };
        
        Ok(quote! {
            match #base_code {
                ParseContent::Sequence(ref elements) if elements.len() > #index_code => {
                    elements[#index_code].content.clone()
                }
                ParseContent::Quantified(ref elements, _) if elements.len() > #index_code => {
                    elements[#index_code].content.clone()
                }
                _ => ParseContent::Terminal("<invalid_array_access>")
            }
        })
    }
    
    /// Generate quantified extraction ($1*, $2+, etc.)
    fn generate_quantified_extraction(
        base: &Box<UnifiedReturnAST>,
        target: &ExtractionTarget,
        captured_vars: &[String],
    ) -> Result<TokenStream> {
        // Get the base reference
        let base_ref = match base.as_ref() {
            UnifiedReturnAST::PositionalRef { index } if *index > 0 && *index <= captured_vars.len() => {
                &captured_vars[index - 1]
            }
            _ => return Ok(quote! { ParseContent::Terminal("<invalid_extraction_base>") })
        };
        
        // Determine extraction index
        let extraction_idx = match target {
            ExtractionTarget::Index(idx) => *idx,
            ExtractionTarget::First => 0,
            ExtractionTarget::Last => {
                // Would need runtime determination
                return Ok(quote! {
                    match &#base_ref {
                        ParseContent::Quantified(elements, _) if !elements.is_empty() => {
                            elements.last().unwrap().content.clone()
                        }
                        _ => ParseContent::Terminal("<no_last_element>")
                    }
                });
            }
        };
        
        Ok(quote! {
            match &#base_ref {
                ParseContent::Quantified(elements, _) => {
                    let extracted: Vec<ParseNode> = elements.iter()
                        .filter_map(|node| {
                            match &node.content {
                                ParseContent::Sequence(subelems) if subelems.len() > #extraction_idx => {
                                    Some(subelems[#extraction_idx].clone())
                                }
                                _ => None
                            }
                        })
                        .collect();
                    ParseContent::Sequence(extracted)
                }
                _ => ParseContent::Terminal("<not_quantified>")
            }
        })
    }
    
    /// Generate passthrough transformation (default behavior)
    fn generate_passthrough(captured_vars: &[String]) -> Result<TokenStream> {
        if captured_vars.is_empty() {
            return Ok(quote! { ParseContent::Terminal("") });
        }
        
        // Return the last captured element or first if only one
        let var_ref = if captured_vars.len() == 1 {
            &captured_vars[0]
        } else {
            captured_vars.last().unwrap()
        };
        
        if var_ref.starts_with("sequence_elements[") {
            Ok(quote! { #var_ref.content.clone() })
        } else {
            Ok(quote! { #var_ref.clone() })
        }
    }
}