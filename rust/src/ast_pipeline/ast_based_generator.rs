// AST-Based Parser Generator using syn and quote
// This module replaces string concatenation with proper AST manipulation
// GUARANTEES: No unbalanced braces, no syntax errors, type-safe code generation

use quote::{quote, format_ident, ToTokens};
use proc_macro2::TokenStream;
use syn::Ident;
use std::collections::HashMap;
use anyhow::Result;
use prettyplease;
use super::Logger;
use crate::ast_pipeline::{ASTNode, ASTValue, TokenValue, UnifiedSemanticAST, BranchAnnotation, ast_return_transform::AstReturnTransformer, Annotations};

/// AST-based generator that produces guaranteed syntactically correct Rust code
pub struct AstBasedGenerator {
    pub grammar_name: String,
    pub entry_rule: Option<String>,
    pub logger: Option<Box<dyn Logger>>,
    pub annotations: Option<Annotations>,
    pub branch_return_annotations: HashMap<String, Vec<Option<BranchAnnotation>>>,
    pub enable_debug: bool,
}

impl AstBasedGenerator {
    pub fn new(grammar_name: String) -> Self {
        Self {
            grammar_name,
            entry_rule: None,
            logger: None,
            annotations: None,
            branch_return_annotations: HashMap::new(),
            enable_debug: true,
        }
    }
    
    /// Main entry point: Generate complete parser from grammar tree
    pub fn generate_parser(
        &self,
        grammar_tree: &HashMap<String, ASTNode>,
        rule_order: &[String],
        filename: &str,
    ) -> Result<String> {
        eprintln!("\n{}", "=".repeat(80));
        eprintln!("🚀  AST-BASED PARSER GENERATION STARTED");
        eprintln!("{}", "=".repeat(80));
        eprintln!("📊  Grammar: '{}' with {} rules", self.grammar_name, rule_order.len());
        eprintln!("🎯  Target: {}", filename);
        eprintln!("📂  File: {}:{}", file!(), line!());
        eprintln!();

        let parser_tokens = self.generate_parser_tokens(grammar_tree, rule_order, filename)?;
        
        eprintln!();
        eprintln!("✅  TokenStream generation complete ({} tokens)", parser_tokens.to_string().len());
        eprintln!("📂  File: {}:{}", file!(), line!());
        
        eprintln!();
        // Convert TokenStream to formatted string using prettyplease
        eprintln!("🎨  Converting TokenStream to formatted Rust code...");
        let formatted_code = prettyplease::unparse(&syn::parse2(parser_tokens)?);
        eprintln!("✨  Code formatting complete ({} characters)", formatted_code.len());
        eprintln!("📂  File: {}:{}", file!(), line!());
        eprintln!();
        Ok(formatted_code)
    }
    
    /// Generate parser as TokenStream (the actual AST)
    pub fn generate_parser_tokens(
        &self,
        grammar_tree: &HashMap<String, ASTNode>,
        rule_order: &[String],
        filename: &str,
    ) -> Result<TokenStream> {
        eprintln!("   🔧  Starting parser code generation for {} rules using AST-based approach", rule_order.len());
        eprintln!("        File: {}:{}", file!(), line!());

        // Determine entry rule
        let entry_rule = self.entry_rule.as_ref()
            .map(|s| s.clone())
            .or_else(|| rule_order.first().cloned())
            .ok_or_else(|| anyhow::anyhow!("No entry rule found"))?;
        
        eprintln!("        Entry rule determined: '{}'", entry_rule);
        eprintln!("        File: {}:{}", file!(), line!());
        
        let parser_name = format_ident!("{}Parser", 
            self.grammar_name.chars()
                .next().unwrap().to_uppercase().collect::<String>() + 
            &self.grammar_name[1..]);
        
        eprintln!("        Generated parser struct name: '{}'", parser_name);
        eprintln!("        File: {}:{}", file!(), line!());
        eprintln!();

        // Generate imports
        let imports = self.generate_imports();
        eprintln!("        Generated import statements");
        eprintln!("        File: {}:{}", file!(), line!());
        
        // Generate types
        let types = self.generate_types();
        eprintln!("        Generated type definitions");
        eprintln!("        File: {}:{}", file!(), line!());
        
        // Generate parser struct
        let parser_struct = self.generate_parser_struct(&parser_name);
        eprintln!("        Generated parser struct definition");
        eprintln!("        File: {}:{}", file!(), line!());
        
        // Generate parser implementation
        let parser_impl = self.generate_parser_impl(&parser_name, grammar_tree, rule_order, &entry_rule, filename)?;
        eprintln!("        Generated parser implementation with all rule methods");
        eprintln!("        File: {}:{}", file!(), line!());
        
        // Generate tests
        let tests = generate_tests(&parser_name);
        eprintln!("        Generated test module");
        eprintln!("        File: {}:{}", file!(), line!());
        eprintln!();
        
        // Combine everything
        let result = quote! {
            #imports
            #types
            #parser_struct
            #parser_impl
            #tests
        };
        
        eprintln!("        Combined all components into final TokenStream ({} chars)", result.to_string().len());
        eprintln!("        File: {}:{}", file!(), line!());
        Ok(result)
    }
    
    fn generate_imports(&self) -> TokenStream {
        quote! {
            use std::collections::HashMap;
            use std::ops::Range;
            use regex::Regex;
            use crate::ast_pipeline::{
                Logger, ParseResult, ParseError, ParseContent, ParseNode, MemoEntry, RuleId, CycleType, RecursionGuard
            };
        }
    }
    
    fn generate_types(&self) -> TokenStream {
        quote! {
            // Types are now shared in crate::ast_pipeline
            // ParseResult, ParseError, ParseContent, ParseNode, MemoEntry, RuleId, CycleType, RecursionGuard
            // are all defined in the parent module
        }
    }
    
    fn generate_parser_struct(&self, parser_name: &Ident) -> TokenStream {
        let grammar_name_upper = self.grammar_name.to_uppercase();
        
        quote! {
            /// High-performance parser with memoization and zero-copy parsing
            pub struct #parser_name<'input> {
                input: &'input str,
                position: usize,
                memo: HashMap<(RuleId, usize), Option<ParseNode<'input>>>,
                recursion_guard: RecursionGuard,
                logger: Box<dyn Logger>,
            }
        }
    }
    
    fn generate_parser_impl(
        &self,
        parser_name: &Ident,
        grammar_tree: &HashMap<String, ASTNode>,
        rule_order: &[String],
        entry_rule: &str,
        filename: &str,
    ) -> Result<TokenStream> {
        // Generate rule ID constants
        let rule_constants = self.generate_rule_constants(rule_order);
        
        // Generate constructor and main parse method
        let constructor = self.generate_constructor();
        let parse_method = self.generate_parse_method(entry_rule);
        
        // Generate rule methods
        eprintln!("\n{}", "-".repeat(60));
        eprintln!("RULE METHOD GENERATION");
        eprintln!("{}", "-".repeat(60));
        let mut rule_methods = Vec::new();
        for rule_name in rule_order {
            eprintln!("   📋  Rule: {} - File: {}:{}", rule_name, file!(), line!());
            if let Some(ast_node) = grammar_tree.get(rule_name) {
                let method = self.generate_rule_method(rule_name, ast_node, rule_order, filename)?;
                rule_methods.push(method);
                eprintln!("        ✓   Completed - File: {}:{}", file!(), line!());
                eprintln!();
                eprintln!();
            }
        }
        eprintln!("All rule methods generated ({}) - File: {}:{}", rule_methods.len(), file!(), line!());
        
        // Generate helper methods
        let helpers = self.generate_helper_methods(filename);
        
        Ok(quote! {
            impl<'input> #parser_name<'input> {
                #rule_constants
                #constructor
                #parse_method
                #(#rule_methods)*
                #helpers
            }
        })
    }
    
    fn generate_rule_constants(&self, rule_order: &[String]) -> TokenStream {
        let constants: Vec<TokenStream> = rule_order.iter().enumerate().map(|(i, name)| {
            let const_name = format_ident!("RULE_{}", name.to_uppercase());
            let id = i as u16;
            quote! {
                const #const_name: RuleId = #id;
            }
        }).collect();
        
        quote! {
            #(#constants)*
        }
    }
    
    fn generate_constructor(&self) -> TokenStream {
        quote! {
            pub fn new(input: &'input str, logger: Box<dyn Logger>) -> Self {
                Self {
                    input,
                    position: 0,
                    memo: HashMap::new(),
                    recursion_guard: RecursionGuard::new(100),
                    logger,
                }
            }
        }
    }
    
    fn generate_parse_method(&self, entry_rule: &str) -> TokenStream {
        let parse_method = format_ident!("parse_{}", entry_rule);
        
        quote! {
            pub fn parse(&mut self) -> ParseResult<ParseNode<'input>> {
                self.#parse_method()
            }
        }
    }
    
    fn generate_rule_method(
        &self,
        rule_name: &str,
        ast_node: &ASTNode,
        rule_order: &[String],
        filename: &str,
    ) -> Result<TokenStream> {
        let method_name = format_ident!("parse_{}", rule_name);
        let rule_const = format_ident!("RULE_{}", rule_name.to_uppercase());
        
        eprintln!("        ↳   Entering rule processing block - File: {}:{}", file!(), line!());
        
        eprintln!();
        // Generate the parsing logic based on AST node type
        let parse_logic = self.generate_node_parsing_logic(ast_node, rule_name, filename)?;
        
        eprintln!();
        eprintln!("            File: {}:{}: Exiting rule processing block", file!(), line!());
        
        // Build the complete method
        Ok(quote! {
            pub fn #method_name(&mut self) -> ParseResult<ParseNode<'input>> {
                let filename_str = #filename;
                // Check for recursion cycles
                let position = self.position;
                let cycle_type = self.recursion_guard.check_cycle(#rule_name, position);
                
                match cycle_type {
                    CycleType::Infinite => {
                        if self.logger.is_enabled() {
                            self.logger.log_error(#filename, 0, &format!("💥 Infinite recursion detected in rule '{}' at position {}", #rule_name, position));
                        }
                        return Err(ParseError::InvalidSyntax {
                            message: "Infinite recursion detected",
                            position,
                        });
                    }
                    CycleType::LeftRecursive => {
                        if self.logger.is_enabled() {
                            self.logger.log_error(#filename, 0, &format!("🔄 Left recursion detected in rule '{}' at position {}", #rule_name, position));
                        }
                        return Err(ParseError::InvalidSyntax {
                            message: "Left recursion detected",
                            position,
                        });
                    }
                    CycleType::MutualRecursive { depth, ref rules } if depth >= 100 => {
                        if self.logger.is_enabled() {
                            self.logger.log_error(#filename, 0, &format!("🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})", #rule_name, position, depth));
                        }
                        return Err(ParseError::RecursionDepthExceeded {
                            position,
                            depth,
                        });
                    }
                    _ => {}
                }
                
                self.recursion_guard.enter(#rule_name, position);
                
                // Declare start_pos outside the closure so it can be used outside
                let start_pos = self.position;
                
                let result = self.memoized_call(Self::#rule_const, |parser| {
                    // Main parsing logic - produces the 'result' variable
                    #parse_logic;
                    
                    let end_pos = parser.position;
                    
                    Ok(ParseNode {
                        rule_name: #rule_name,
                        content: result,
                        span: start_pos..end_pos,
                    })
                });
                
                self.recursion_guard.exit();
                
                match &result {
                    Ok(node) => {
                        if self.logger.is_enabled() {
                            let consumed = node.span.end - start_pos;
                            if consumed > 0 {
                                self.logger.log_success(#filename, 0, &format!("✅ Rule '{}' successfully parsed from {} to {} (consumed {} chars: '{}')", #rule_name, start_pos, node.span.end, consumed, &self.input[start_pos..node.span.end]));
                            } else {
                                self.logger.log_warning(#filename, 0, &format!("⚠️ Rule '{}' matched with zero length at position {}", #rule_name, start_pos));
                            }
                            self.logger.log_success(#filename, 0, &format!("✅ Exiting rule '{}' successfully - advanced from {} to {}", #rule_name, start_pos, self.position));
                        }
                    }
                    Err(e) => {
                        if self.logger.is_enabled() {
                            self.logger.log_error(#filename, 0, &format!("❌ Exiting rule '{}' with error: {:?} - backtracked to {}", #rule_name, e, self.position));
                        }
                    }
                }
                
                result
            }
        })
    }
    
    fn generate_node_parsing_logic(&self, ast_node: &ASTNode, rule_name: &str, filename: &str) -> Result<TokenStream> {
        eprintln!("   🔍  Generating parsing logic for rule '{}' with AST node type: {:?} - File: {}:{}", rule_name, ast_node, file!(), line!());
        
        match ast_node {
            ASTNode::Or { alternatives } => {
                eprintln!("        Processing OR node with {} alternatives - File: {}:{}", alternatives.len(), file!(), line!());
                self.generate_or_logic(alternatives, rule_name, filename)
            }
            ASTNode::Sequence { elements } => {
                eprintln!("        Processing sequence node with {} elements - File: {}:{}", elements.len(), file!(), line!());
                self.generate_sequence_logic(elements, rule_name, filename)
            }
            ASTNode::Atom { value } => {
                eprintln!("        Processing atom node - File: {}:{}", file!(), line!());
                self.generate_atom_logic(value, rule_name, filename)
            }
            ASTNode::Quantified { element, quantifier } => {
                eprintln!("        Processing quantified node with '{}' quantifier - File: {}:{}", quantifier, file!(), line!());
                self.generate_quantified_logic(element, quantifier, rule_name, filename)
            }
        }
    }
    
    fn generate_or_logic(&self, alternatives: &[ASTNode], rule_name: &str, filename: &str) -> Result<TokenStream> {
        let branch_count = alternatives.len();
        
        // Check if this is a single-branch or multi-branch rule
        if branch_count == 1 {
            // Single branch - simpler logic without try_parse
            let branch = &alternatives[0];
            eprintln!();
            let branch_logic = self.generate_node_parsing_logic(branch, rule_name, filename)?;
            
            // Check for return annotations for this rule
            let has_transform = if let Some(branches) = self.branch_return_annotations.get(rule_name) {
                branches.get(0).map(|ann| ann.is_some()).unwrap_or(false)
            } else {
                false
            };
            
            if has_transform {
                // Has return annotation - need to transform
                let transform = self.branch_return_annotations.get(rule_name)
                    .and_then(|branches| branches.get(0))
                    .and_then(|ann| ann.as_ref())
                    .map(|annotation| self.generate_return_transform(annotation, rule_name, &["result".to_string()]))
                    .transpose()?
                    .unwrap_or(quote! { result });
                    
                // Check if transform is just "result" - avoid redundant assignment
                if transform.to_string() == "result" {
                    Ok(quote! {
                        // Single-branch rule, no transformation needed
                        #branch_logic  // Sets result
                    })
                } else {
                    Ok(quote! {
                        // Single-branch rule with transformation
                        #branch_logic;
                        
                        // Apply transformation (reassign result)
                        result = #transform
                    })
                }
            } else {
                // No transformation needed - simpler code
                Ok(quote! {
                    // Single-branch rule
                    #branch_logic  // Sets result directly
                })
            }
        } else {
            // Multi-branch - use try_parse for each alternative
            let mut branch_attempts = Vec::new();
            
        for (idx, alternative) in alternatives.iter().enumerate() {
            eprintln!();
            let branch_logic = self.generate_node_parsing_logic(alternative, rule_name, filename)?;
                
                // Check for branch-specific return annotation
                let transform = if let Some(branches) = self.branch_return_annotations.get(rule_name) {
                    if let Some(Some(annotation)) = branches.get(idx) {
                        self.generate_return_transform(annotation, rule_name, &["content".to_string()])?
                    } else {
                        quote! { content }
                    }
                } else {
                    quote! { content }
                };
                
                let branch_attempt = if idx == 0 {
                    let branch_num = idx + 1;
                    quote! {
                        if let Some(content) = parser.try_parse(|p| {
                            if p.logger.is_enabled() {
                                p.logger.log_info(#filename, 0, &format!("🚪 Entering branch {}/{} for rule '{}' at position {}", #branch_num, #branch_count, #rule_name, p.position));
                            }
                            #branch_logic;
                            if p.logger.is_enabled() {
                                p.logger.log_info(#filename, 0, &format!("✅ Leaving branch {}/{} for rule '{}' at position {} (success)", #branch_num, #branch_count, #rule_name, p.position));
                            }
                            Ok(result)
                        }) {
                            result = #transform;
                        } else {
                            if parser.logger.is_enabled() {
                                parser.logger.log_info(#filename, 0, &format!("❌ Branch {}/{} for rule '{}' failed at position {}", #branch_num, #branch_count, #rule_name, parser.position));
                            }
                        }
                    }
                } else if idx < branch_count - 1 {
                    let branch_num = idx + 1;
                    quote! {
                        else if let Some(content) = parser.try_parse(|p| {
                            if p.logger.is_enabled() {
                                p.logger.log_info(#filename, 0, &format!("🚪 Entering branch {}/{} for rule '{}' at position {}", #branch_num, #branch_count, #rule_name, p.position));
                            }
                            #branch_logic;
                            if p.logger.is_enabled() {
                                p.logger.log_info(#filename, 0, &format!("✅ Leaving branch {}/{} for rule '{}' at position {} (success)", #branch_num, #branch_count, #rule_name, p.position));
                            }
                            Ok(result)
                        }) {
                            result = #transform;
                        } else {
                            if parser.logger.is_enabled() {
                                parser.logger.log_info(#filename, 0, &format!("❌ Branch {}/{} for rule '{}' failed at position {}", #branch_num, #branch_count, #rule_name, parser.position));
                            }
                        }
                    }
                } else {
                    // Last branch - no try_parse needed
                    let branch_num = idx + 1;
                    quote! {
                        else {
                            if parser.logger.is_enabled() {
                                parser.logger.log_info(#filename, 0, &format!("🚪 Entering branch {}/{} for rule '{}' at position {}", #branch_num, #branch_count, #rule_name, parser.position));
                            }
                            #branch_logic;
                            if parser.logger.is_enabled() {
                                parser.logger.log_info(#filename, 0, &format!("✅ Leaving branch {}/{} for rule '{}' at position {} (success)", #branch_num, #branch_count, #rule_name, parser.position));
                            }
                            let content = result;
                            result = #transform;
                        }
                    }
                };
                
                branch_attempts.push(branch_attempt);
            }
            
            Ok(quote! {
                // Multi-branch parsing logic
                #(#branch_attempts)*
            })
        }
    }
    
    fn generate_sequence_logic(&self, elements: &[ASTNode], rule_name: &str, filename: &str) -> Result<TokenStream> {
        let element_count = elements.len();
        let mut element_parsers = Vec::new();
        
        for (idx, element) in elements.iter().enumerate() {
            let element_parser = self.generate_sequence_element(element, idx, element_count, rule_name, filename)?;
            element_parsers.push(element_parser);
        }
        
        Ok(quote! {
            let mut sequence_elements = Vec::with_capacity(#element_count);
            #(#element_parsers)*
            let result = ParseContent::Sequence(sequence_elements)
        })
    }
    
    fn generate_sequence_element(
        &self,
        element: &ASTNode,
        index: usize,
        total: usize,
        rule_name: &str,
        filename: &str,
    ) -> Result<TokenStream> {
        let element_logic = match element {
            ASTNode::Quantified { element, quantifier } if quantifier == "?" => {
                // Optional element
                eprintln!();
                let inner_logic = self.generate_node_parsing_logic(element, rule_name, filename)?;
                quote! {
                    if let Some(content) = parser.try_parse(|p| {
                        #inner_logic;
                        Ok(result)
                    }) {
                        content
                    } else {
                        ParseContent::Sequence(Vec::new())
                    }
                }
            }
            _ => {
                eprintln!();
                let inner_logic = self.generate_node_parsing_logic(element, rule_name, filename)?;
                quote! {
                    {
                        #inner_logic;
                        result
                    }
                }
            }
        };
        
        let element_name = format!("element_{}", index);
        
        Ok(quote! {
            {
                let element_start = parser.position;
                let element_content = #element_logic;
                let element_end = parser.position;
                
                sequence_elements.push(ParseNode {
                    rule_name: #element_name,
                    content: element_content,
                    span: element_start..element_end,
                });
            }
        })
    }
    
    fn generate_atom_logic(&self, value: &ASTValue, rule_name: &str, filename: &str) -> Result<TokenStream> {
        eprintln!("        Processing atom value: {:?} - File: {}:{}", value, file!(), line!());
        
        match value {
            ASTValue::Token(parts) if parts.len() >= 2 => {
                let token_type_str = if let TokenValue::String(ref s) = parts[0] { s.as_str() } else { "" };
                let token_value_str = if let TokenValue::String(ref s) = parts[1] { s.as_str() } else { "" };
                
                eprintln!("        Token type: '{}', value: '{}' - File: {}:{}", token_type_str, token_value_str, file!(), line!());
                
                match token_type_str {
                    "quoted_string" => {
                        eprintln!("        Generating string terminal matcher for '{}' - File: {}:{}", token_value_str, file!(), line!());
                        Ok(quote! {
                            let result = ParseContent::Terminal(parser.match_string(#token_value_str)?)
                        })
                    }
                    "rule_reference" => {
                        eprintln!("        Generating rule reference call to '{}' - File: {}:{}", token_value_str, file!(), line!());
                        let method = format_ident!("parse_{}", token_value_str);
                        Ok(quote! {
                            let result = ParseContent::Alternative(Box::new(parser.#method()?))
                        })
                    }
                    "regex" => {
                        eprintln!("        Generating regex matcher for pattern '{}' - File: {}:{}", token_value_str, file!(), line!());
                        // Check for semantic annotations that should transform the matched string
                        if let Some(annotations) = &self.annotations {
                            if let Some(semantic_asts) = annotations.semantic_annotations.get(rule_name) {
                                for ast in semantic_asts {
                                    if let UnifiedSemanticAST::TransformExpr { expression } = ast {
                                        // Parse transform expressions like "str::parse::<f64>().unwrap_or(0.0)"
                                        if expression.starts_with("str::parse::<") && expression.contains(">().unwrap_or(") {
                                            if let Some(type_end) = expression.find(">().unwrap_or(") {
                                                let type_str = &expression["str::parse::<".len()..type_end];
                                                let default_start = type_end + ">().unwrap_or(".len();
                                                let default_end = expression.len() - 1; // remove closing )
                                                let default_str = &expression[default_start..default_end];
                                                
                                                let type_ident = format_ident!("{}", type_str);
                                                let default_expr: syn::Expr = syn::parse_str(default_str).unwrap_or(syn::parse_str("0").unwrap());
                                                
                                                // Generate transformation code
                                                if self.enable_debug {
                                                    return Ok(quote! {
                                                        let matched_str = parser.match_regex(#token_value_str)?;
                                                        let transformed = matched_str.parse::<#type_ident>().unwrap_or(#default_expr);
                                                        parser.debug_output.push(format!("🎯 Applied semantic transform: parsed '{}' to {}={}", matched_str, stringify!(#type_ident), transformed));
                                                        let result = ParseContent::TransformedTerminal(transformed.to_string())
                                                    });
                                                } else {
                                                    return Ok(quote! {
                                                        let matched_str = parser.match_regex(#token_value_str)?;
                                                        let transformed = matched_str.parse::<#type_ident>().unwrap_or(#default_expr);
                                                        let result = ParseContent::TransformedTerminal(transformed.to_string())
                                                    });
                                                }
                                            }
                                        }
                                        
                                        // Fallback: treat as raw expression
                                        if self.enable_debug {
                                            return Ok(quote! {
                                                let matched_str = parser.match_regex(#token_value_str)?;
                                                parser.debug_output.push(format!("🎯 Applied semantic transform: raw expression '{}' to rule '{}': matched '{}'", #expression, #rule_name, matched_str));
                                                let result = ParseContent::TransformedTerminal(#expression.to_string())
                                            });
                                        } else {
                                            return Ok(quote! {
                                                let matched_str = parser.match_regex(#token_value_str)?;
                                                let result = ParseContent::TransformedTerminal(#expression.to_string())
                                            });
                                        }
                                    }
                                }
                            }
                        }

                        // Default behavior: return matched string as terminal
                        Ok(quote! {
                            let result = ParseContent::Terminal(parser.match_regex(#token_value_str)?)
                        })
                    }
                    _ => Ok(quote! {
                        let result = ParseContent::Terminal("")
                    })
                }
            }
            _ => Ok(quote! {
                let result = ParseContent::Terminal("")
            })
        }
    }
    
    fn generate_quantified_logic(
        &self,
        element: &ASTNode,
        quantifier: &str,
        rule_name: &str,
        filename: &str,
    ) -> Result<TokenStream> {
        eprintln!();
        let element_logic = self.generate_node_parsing_logic(element, rule_name, filename)?;
        
        match quantifier {
            "*" => Ok(quote! {
                let mut results = Vec::new();
                let mut last_position = parser.position;
                let mut iteration_count = 0;
                const MAX_ITERATIONS: usize = 10000; // Safety limit
                
                while iteration_count < MAX_ITERATIONS {
                    if let Some(node) = parser.try_parse(|p| {
                        #element_logic;
                        Ok(ParseNode {
                            rule_name: "quantified",
                            content: result,
                            span: 0..0,
                        })
                    }) {
                        let current_position = parser.position;
                        
                        // Critical: Check for zero-length match
                        if current_position == last_position {
                            if parser.logger.is_enabled() {
                                parser.logger.log_warning(#filename, 0, &format!("⚠️ ZERO-LENGTH MATCH in quantifier: Breaking to prevent infinite loop at position {}", current_position));
                            }
                            break;
                        }
                        
                        results.push(node);
                        last_position = current_position;
                        iteration_count += 1;
                    } else {
                        break;
                    }
                }
                
                if iteration_count >= MAX_ITERATIONS && parser.logger.is_enabled() {
                    parser.logger.log_warning(#filename, 0, &format!("⚠️ MAX ITERATIONS ({}) reached in quantifier", MAX_ITERATIONS));
                }
                
                let result = ParseContent::Quantified(results, "*")
            }),
            "+" => Ok(quote! {
                let mut results = Vec::new();
                let start_position = parser.position;
                
                // First match is mandatory
                {
                    #element_logic;
                    results.push(ParseNode {
                        rule_name: "quantified",
                        content: result,
                        span: 0..0,
                    });
                }
                
                // Check if first match consumed any input
                if parser.position == start_position {
                    if parser.logger.is_enabled() {
                        parser.logger.log_warning(#filename, 0, &format!("⚠️ ZERO-LENGTH FIRST MATCH in + quantifier at position {}", start_position));
                    }
                }
                
                // Additional matches are optional
                let mut last_position = parser.position;
                let mut iteration_count = 1;
                const MAX_ITERATIONS: usize = 10000;
                
                while iteration_count < MAX_ITERATIONS {
                    if let Some(node) = parser.try_parse(|p| {
                        #element_logic;
                        Ok(ParseNode {
                            rule_name: "quantified",
                            content: result,
                            span: 0..0,
                        })
                    }) {
                        let current_position = parser.position;
                        
                        // Check for zero-length match
                        if current_position == last_position {
                            if parser.logger.is_enabled() {
                                parser.logger.log_warning(#filename, 0, &format!("⚠️ ZERO-LENGTH MATCH in + quantifier: Breaking at position {}", current_position));
                            }
                            break;
                        }
                        
                        results.push(node);
                        last_position = current_position;
                        iteration_count += 1;
                    } else {
                        break;
                    }
                }
                
                if iteration_count >= MAX_ITERATIONS && parser.logger.is_enabled() {
                    parser.logger.log_warning(#filename, 0, &format!("⚠️ MAX ITERATIONS ({}) reached in + quantifier", MAX_ITERATIONS));
                }
                
                let result = ParseContent::Quantified(results, "+")
            }),
            "?" => Ok(quote! {
                let result = if let Some(node) = parser.try_parse(|p| {
                    #element_logic;
                    Ok(ParseNode {
                        rule_name: "quantified",
                        content: result,
                        span: 0..0,
                    })
                }) {
                    ParseContent::Quantified(vec![node], "?")
                } else {
                    ParseContent::Quantified(Vec::new(), "?")
                }
            }),
            _ => Err(anyhow::anyhow!("Unknown quantifier: {}", quantifier))
        }
    }
    
    fn generate_return_transform(
        &self,
        annotation: &BranchAnnotation,
        rule_name: &str,
        captured_vars: &[String],
    ) -> Result<TokenStream> {
        eprintln!("DEBUG: generate_return_transform called for rule '{}', parsed_ast is {}", 
                  rule_name, if annotation.parsed_ast.is_some() { "Some" } else { "None" });
        
        if let Some(ref ast) = annotation.parsed_ast {
            AstReturnTransformer::generate_transform(ast, captured_vars, rule_name)
        } else {
            // Return annotation parsing failed - add comment explaining why
            let comment = format!(
                "/* WARNING: Return annotation '{}' for rule '{}' failed to parse.\n   \
                 This may be due to complex syntax not supported by bootstrap parser.\n   \
                 Enable bootstrap=false to use full external parser.\n   \
                 Raw annotation: {} */",
                annotation.annotation_content, rule_name, annotation.annotation_content
            );
            eprintln!("DEBUG: Adding warning comment for rule '{}' with annotation '{}'", rule_name, annotation.annotation_content);
            Ok(quote! {
                #comment
                result
            })
        }
    }
    /// Utility function to unparse a ParseNode back to text for round-trip testing
    // pub fn unparse_node(&self, node: &ParseNode<'input>) -> String {
    //     format!("{:?}", node.content)
    // }

    
    fn generate_helper_methods(&self, filename: &str) -> TokenStream {
        quote! {
            fn match_string(&mut self, expected: &str) -> ParseResult<&'input str> {
                let start = self.position;
                let end = start + expected.len();
                
                if self.logger.is_enabled() {
                    self.logger.log_debug(#filename, 0, &format!("🔤 Attempting to match terminal '{}' at position {} (end: {})", expected, start, end));
                }
                
                if end <= self.input.len() {
                    let slice = &self.input[start..end];
                    if slice == expected {
                        self.position = end;
                        
                        if self.logger.is_enabled() {
                            self.logger.log_success(#filename, 0, &format!("✅ Terminal '{}' matched, advanced to position {}", expected, end));
                        }
                        
                        return Ok(slice);
                    }
                }
                
                // Enhanced error with context
                let found_str = if self.position < self.input.len() {
                    let end = (self.position + expected.len()).min(self.input.len());
                    &self.input[self.position..end]
                } else {
                    "<EOF>"
                };
                
                if self.logger.is_enabled() {
                    self.logger.log_error(#filename, 0, &format!("❌ Terminal '{}' failed at position {} - found '{}'", expected, start, found_str));
                }
                
                Err(self.create_contextual_error(&format!(
                    "Expected '{}' but found '{}'",
                    expected, found_str
                )))
            }
            
            fn match_regex(&mut self, pattern: &str) -> ParseResult<&'input str> {
                let re = regex::Regex::new(pattern)
                    .map_err(|e| self.create_contextual_error(&format!(
                        "Invalid regex pattern '{}': {}",
                        pattern, e
                    )))?;
                
                if let Some(mat) = re.find(&self.input[self.position..]) {
                    if mat.start() == 0 {
                        let matched = mat.as_str();
                        let start = self.position;
                        
                        if self.logger.is_enabled() {
                            self.logger.log_success(#filename, 0, &format!("✅ Regex '{}' matched '{}' at position {}", pattern, matched, start));
                        }
                        
                        self.position += matched.len();
                        return Ok(&self.input[start..self.position]);
                    }
                }
                
                if self.logger.is_enabled() {
                    let preview = if self.position < self.input.len() {
                        let end = (self.position + 10).min(self.input.len());
                        &self.input[self.position..end]
                    } else {
                        "<EOF>"
                    };
                    self.logger.log_error(#filename, 0, &format!("❌ Regex '{}' no match at position {} (next: '{}')", pattern, self.position, preview));
                }
                
                Err(self.create_contextual_error(&format!(
                    "No match for regex pattern '{}'",
                    pattern
                )))
            }
            
            fn try_parse<F, T>(&mut self, f: F) -> Option<T>
            where
                F: FnOnce(&mut Self) -> ParseResult<T>,
            {
                let saved_pos = self.position;
                let saved_stack_len = self.recursion_guard.parse_stack.len();
                
                if self.logger.is_enabled() {
                    self.logger.log_debug(#filename, 0, &format!("🔄 Starting speculative parse at position {}", saved_pos));
                }
                
                match f(self) {
                    Ok(result) => {
                        if self.logger.is_enabled() {
                            self.logger.log_success(#filename, 0, &format!("🔄 Speculative parse succeeded, advanced to position {}", self.position));
                        }
                        Some(result)
                    }
                    Err(e) => {
                        // Backtrack
                        self.position = saved_pos;
                        self.recursion_guard.parse_stack.truncate(saved_stack_len);
                        
                        if self.logger.is_enabled() {
                            self.logger.log_warning(#filename, 0, &format!("🔙 Speculative parse failed with error '{:?}', backtracked to position {}", e, saved_pos));
                        }
                        
                        None
                    }
                }
            }
            
            fn memoized_call<F>(&mut self, rule_id: RuleId, f: F) -> ParseResult<ParseNode<'input>>
            where
                F: FnOnce(&mut Self) -> ParseResult<ParseNode<'input>>,
            {
                let key = (rule_id, self.position);
                
                if let Some(cached) = self.memo.get(&key) {
                    if let Some(ref node) = cached {
                        self.position = node.span.end;
                        
                        if self.logger.is_enabled() {
                            self.logger.log_info(#filename, 0, &format!("💾 Memo hit for rule {} at position {} - reusing cached result", rule_id, self.position));
                        }
                        
                        return Ok(node.clone());
                    } else {
                        if self.logger.is_enabled() {
                            self.logger.log_warning(#filename, 0, &format!("💾 Memo miss for rule {} at position {} - cached failure", rule_id, self.position));
                        }
                        return Err(ParseError::Backtrack {
                            position: self.position,
                        });
                    }
                }
                
                if self.logger.is_enabled() {
                    self.logger.log_debug(#filename, 0, &format!("💾 Memo miss for rule {} at position {} - computing fresh result", rule_id, self.position));
                }
                
                let result = f(self);
                
                if let Ok(ref node) = result {
                    self.memo.insert(key, Some(node.clone()));
                    if self.logger.is_enabled() {
                        self.logger.log_info(#filename, 0, &format!("💾 Memoized successful result for rule {} at position {}", rule_id, self.position));
                    }
                } else {
                    self.memo.insert(key, None);
                    if self.logger.is_enabled() {
                        self.logger.log_warning(#filename, 0, &format!("💾 Memoized failed result for rule {} at position {}", rule_id, self.position));
                    }
                }
                
                result
            }
            
            fn create_contextual_error(&self, message: &str) -> ParseError {
                let position = self.position;
                
                // Gather rule stack
                let rule_stack: Vec<String> = self.recursion_guard.parse_stack.iter()
                    .map(|(rule, _)| rule.clone())
                    .collect();
                
                // Get input context around the error position
                let start = position.saturating_sub(20);
                let end = (position + 20).min(self.input.len());
                let input_context = if start < end {
                    self.input[start..end].to_string()
                } else {
                    String::new()
                };
                
                ParseError::ContextualError {
                    message: message.to_string(),
                    position,
                    rule_stack,
                    input_context,
                }
            }
        }
    }
}

fn generate_tests(parser_name: &Ident) -> TokenStream {
        quote! {
            #[cfg(test)]
            mod tests {
                use super::*;
                use super::Logger;
                
                #[test]
                fn test_basic_parsing() {
                    let input = "$1";
                    let logger = Box::new(crate::NoOpLogger);
                    let mut parser = #parser_name::new(input, logger);
                    let result = parser.parse();
                    assert!(result.is_ok());
                }
            }
        }
}
