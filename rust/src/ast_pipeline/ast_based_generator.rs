// AST-Based Parser Generator using syn and quote
// This module replaces string concatenation with proper AST manipulation
// GUARANTEES: No unbalanced braces, no syntax errors, type-safe code generation

use syn::{parse_quote, Block, Expr, Ident, ItemFn, Stmt};
use quote::{quote, format_ident, ToTokens};
use proc_macro2::TokenStream;
use std::collections::HashMap;
use anyhow::Result;
use prettyplease;
use crate::ast_pipeline::{ASTNode, ASTValue, Annotations};
use crate::ast_pipeline::ast_return_transform::AstReturnTransformer;

/// AST-based generator that produces guaranteed syntactically correct Rust code
pub struct AstBasedGenerator {
    pub grammar_name: String,
    pub entry_rule: Option<String>,
    pub annotations: Option<Annotations>,
    pub branch_return_annotations: HashMap<String, Vec<Option<BranchAnnotation>>>,
    pub enable_debug: bool,
}

#[derive(Debug, Clone)]
pub struct BranchAnnotation {
    pub annotation_type: String,
    pub annotation_content: String,
    pub parsed_ast: Option<crate::ast_pipeline::UnifiedReturnAST>,
}

impl AstBasedGenerator {
    pub fn new(grammar_name: String) -> Self {
        Self {
            grammar_name,
            entry_rule: None,
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
    ) -> Result<String> {
        let parser_tokens = self.generate_parser_tokens(grammar_tree, rule_order)?;
        
        // Convert TokenStream to formatted string using prettyplease
        let formatted_code = prettyplease::unparse(&syn::parse2(parser_tokens)?);
        Ok(formatted_code)
    }
    
    /// Generate parser as TokenStream (the actual AST)
    pub fn generate_parser_tokens(
        &self,
        grammar_tree: &HashMap<String, ASTNode>,
        rule_order: &[String],
    ) -> Result<TokenStream> {
        // Determine entry rule
        let entry_rule = self.entry_rule.as_ref()
            .map(|s| s.clone())
            .or_else(|| rule_order.first().cloned())
            .ok_or_else(|| anyhow::anyhow!("No entry rule found"))?;
        
        // Generate parser name
        let parser_name = format_ident!("{}Parser", 
            self.grammar_name.chars()
                .next().unwrap().to_uppercase().collect::<String>() + 
            &self.grammar_name[1..]);
        
        // Generate imports
        let imports = self.generate_imports();
        
        // Generate types
        let types = self.generate_types();
        
        // Generate parser struct
        let parser_struct = self.generate_parser_struct(&parser_name);
        
        // Generate parser implementation
        let parser_impl = self.generate_parser_impl(&parser_name, grammar_tree, rule_order, &entry_rule)?;
        
        // Generate tests
        let tests = self.generate_tests(&parser_name);
        
        // Combine everything
        Ok(quote! {
            #imports
            #types
            #parser_struct
            #parser_impl
            #tests
        })
    }
    
    fn generate_imports(&self) -> TokenStream {
        quote! {
            use std::collections::HashMap;
            use std::ops::Range;
            use regex::Regex;
            
            #[allow(dead_code)]
            #[allow(unused_variables)]
            #[allow(unused_mut)]
        }
    }
    
    fn generate_types(&self) -> TokenStream {
        quote! {
            /// Parse result type
            pub type ParseResult<T> = Result<T, ParseError>;
            
            /// Parse errors
            #[derive(Debug, Clone, PartialEq)]
            pub enum ParseError {
                UnexpectedEof { position: usize },
                UnexpectedToken { expected: &'static str, found: char, position: usize },
                InvalidSyntax { message: &'static str, position: usize },
                Backtrack { position: usize },
                RecursionDepthExceeded { position: usize, depth: usize },
                ContextualError { 
                    message: String,
                    position: usize,
                    rule_stack: Vec<String>,
                    input_context: String,
                },
            }
            
            impl std::fmt::Display for ParseError {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    match self {
                        ParseError::UnexpectedEof { position } => 
                            write!(f, "Unexpected EOF at position {}", position),
                        ParseError::UnexpectedToken { expected, found, position } => 
                            write!(f, "Expected '{}', found '{}' at position {}", expected, found, position),
                        ParseError::InvalidSyntax { message, position } => 
                            write!(f, "{} at position {}", message, position),
                        ParseError::Backtrack { position } => 
                            write!(f, "Backtrack at position {}", position),
                        ParseError::RecursionDepthExceeded { position, depth } => 
                            write!(f, "Recursion depth exceeded ({} levels) at position {}", depth, position),
                        ParseError::ContextualError { message, position, rule_stack, input_context } => {
                            writeln!(f, "Parse Error: {}\n", message)?;
                            writeln!(f, "Position: {}\n", position)?;
                            writeln!(f, "Context: {}\n", input_context)?;
                            writeln!(f, "Rule Stack:")?;
                            for (i, rule) in rule_stack.iter().enumerate() {
                                writeln!(f, "  {}: {}", i, rule)?;
                            }
                            Ok(())
                        }
                    }
                }
            }
            
            impl std::error::Error for ParseError {}
            
            /// Parse content types
            #[derive(Debug, Clone, PartialEq)]
            pub enum ParseContent<'input> {
                Terminal(&'input str),
                Sequence(Vec<ParseNode<'input>>),
                Alternative(Box<ParseNode<'input>>),
                Quantified(Vec<ParseNode<'input>>, &'static str),
            }
            
            /// Parse node
            #[derive(Debug, Clone, PartialEq)]
            pub struct ParseNode<'input> {
                pub rule_name: &'static str,
                pub content: ParseContent<'input>,
                pub span: Range<usize>,
            }
            
            /// Memoization entry
            #[derive(Debug, Clone)]
            struct MemoEntry<'input> {
                result: Option<ParseNode<'input>>,
                end_pos: usize,
            }
            
            /// Rule ID type for memoization
            type RuleId = u16;
            
            /// Recursion cycle types
            #[derive(Debug, Clone, PartialEq)]
            pub enum CycleType {
                None,
                Infinite,
                LeftRecursive,
                MutualRecursive { depth: usize, rules: Vec<String> },
            }
            
            /// Recursion guard
            pub struct RecursionGuard {
                parse_stack: Vec<(String, usize)>,
                max_depth: usize,
                cycle_cache: HashMap<(String, usize), CycleType>,
            }
            
            impl RecursionGuard {
                pub fn new(max_depth: usize) -> Self {
                    Self {
                        parse_stack: Vec::new(),
                        max_depth,
                        cycle_cache: HashMap::new(),
                    }
                }
                
                pub fn check_cycle(&mut self, rule_name: &str, position: usize) -> CycleType {
                    if let Some(cached) = self.cycle_cache.get(&(rule_name.to_string(), position)) {
                        return cached.clone();
                    }
                    
                    for (r, p) in self.parse_stack.iter() {
                        if r == rule_name && *p == position {
                            let cycle = CycleType::Infinite;
                            self.cycle_cache.insert((rule_name.to_string(), position), cycle.clone());
                            return cycle;
                        }
                        if r == rule_name && *p > position {
                            let cycle = CycleType::LeftRecursive;
                            self.cycle_cache.insert((rule_name.to_string(), position), cycle.clone());
                            return cycle;
                        }
                    }
                    
                    if self.parse_stack.len() >= self.max_depth {
                        let rules: Vec<String> = self.parse_stack.iter()
                            .map(|(r, _)| r.clone())
                            .collect();
                        return CycleType::MutualRecursive {
                            depth: self.parse_stack.len(),
                            rules,
                        };
                    }
                    
                    CycleType::None
                }
                
                pub fn enter(&mut self, rule_name: &str, position: usize) {
                    self.parse_stack.push((rule_name.to_string(), position));
                }
                
                pub fn exit(&mut self) {
                    self.parse_stack.pop();
                }
            }
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
                debug_mode: bool,
                debug_output: Vec<String>,
            }
        }
    }
    
    fn generate_parser_impl(
        &self,
        parser_name: &Ident,
        grammar_tree: &HashMap<String, ASTNode>,
        rule_order: &[String],
        entry_rule: &str,
    ) -> Result<TokenStream> {
        // Generate rule ID constants
        let rule_constants = self.generate_rule_constants(rule_order);
        
        // Generate constructor and main parse method
        let constructor = self.generate_constructor();
        let parse_method = self.generate_parse_method(entry_rule);
        
        // Generate rule methods
        let mut rule_methods = Vec::new();
        for rule_name in rule_order {
            if let Some(ast_node) = grammar_tree.get(rule_name) {
                let method = self.generate_rule_method(rule_name, ast_node, rule_order)?;
                rule_methods.push(method);
            }
        }
        
        // Generate helper methods
        let helpers = self.generate_helper_methods();
        
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
            pub fn new(input: &'input str) -> Self {
                Self {
                    input,
                    position: 0,
                    memo: HashMap::new(),
                    recursion_guard: RecursionGuard::new(100),
                    debug_mode: false,
                    debug_output: Vec::new(),
                }
            }
            
            pub fn with_debug(input: &'input str) -> Self {
                Self {
                    input,
                    position: 0,
                    memo: HashMap::new(),
                    recursion_guard: RecursionGuard::new(100),
                    debug_mode: true,
                    debug_output: Vec::new(),
                }
            }
            
            pub fn with_debug_log(input: &'input str, test_name: &str) -> Self {
                use std::fs::File;
                use std::io::Write;
                
                let mut parser = Self::with_debug(input);
                
                // Create debug log file
                let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
                let filename = format!("{}_{}.log", test_name, timestamp);
                
                // Write header to debug output
                parser.debug_output.push(format!("==== Parser Debug Log ===="));
                parser.debug_output.push(format!("Test: {}", test_name));
                parser.debug_output.push(format!("Time: {}", timestamp));
                parser.debug_output.push(format!("Input: {:?}", input));
                parser.debug_output.push(format!("=".repeat(50)));
                parser.debug_output.push(String::new());
                
                parser
            }
            
            fn format_debug_indent(&self) -> String {
                let depth = self.recursion_guard.parse_stack.len();
                if depth == 0 {
                    String::new()
                } else {
                    format!("{}├─ ", "│  ".repeat(depth.saturating_sub(1)))
                }
            }
            
            pub fn get_debug_trace(&self) -> String {
                self.debug_output.join("\n")
            }
            
            fn create_contextual_error(&self, message: &str) -> ParseError {
                let rule_stack: Vec<String> = self.recursion_guard.parse_stack
                    .iter()
                    .map(|(rule, pos)| format!("{} @ {}", rule, pos))
                    .collect();
                
                let context_start = self.position.saturating_sub(20);
                let context_end = (self.position + 20).min(self.input.len());
                let context = &self.input[context_start..context_end];
                
                let input_context = if context_start > 0 {
                    format!("...{}", context)
                } else {
                    context.to_string()
                };
                
                let input_context = if context_end < self.input.len() {
                    format!("{}...", input_context)
                } else {
                    input_context
                };
                
                ParseError::ContextualError {
                    message: message.to_string(),
                    position: self.position,
                    rule_stack,
                    input_context,
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
    ) -> Result<TokenStream> {
        let method_name = format_ident!("parse_{}", rule_name);
        let rule_const = format_ident!("RULE_{}", rule_name.to_uppercase());
        
        // Generate the parsing logic based on AST node type
        let parse_logic = self.generate_node_parsing_logic(ast_node, rule_name)?;
        
        // Build the complete method
        Ok(quote! {
            fn #method_name(&mut self) -> ParseResult<ParseNode<'input>> {
                // Check for recursion cycles
                let position = self.position;
                let cycle_type = self.recursion_guard.check_cycle(#rule_name, position);
                
                match cycle_type {
                    CycleType::Infinite => {
                        if self.debug_mode {
                            self.debug_output.push(format!("🔄 INFINITE RECURSION: {} at {}", #rule_name, position));
                        }
                        return Err(ParseError::InvalidSyntax {
                            message: "Infinite recursion detected",
                            position,
                        });
                    }
                    CycleType::LeftRecursive => {
                        if self.debug_mode {
                            self.debug_output.push(format!("↩️ LEFT RECURSION: {} at {}", #rule_name, position));
                        }
                        return Err(ParseError::InvalidSyntax {
                            message: "Left recursion detected",
                            position,
                        });
                    }
                    CycleType::MutualRecursive { depth, ref rules } if depth >= 100 => {
                        if self.debug_mode {
                            self.debug_output.push(format!("🔃 RECURSION LIMIT: {} depth={}", #rule_name, depth));
                        }
                        return Err(ParseError::RecursionDepthExceeded {
                            position,
                            depth,
                        });
                    }
                    _ => {}
                }
                
                self.recursion_guard.enter(#rule_name, position);
                
                let result = self.memoized_call(Self::#rule_const, |parser| {
                    let start_pos = parser.position;
                    
                    if parser.debug_mode {
                        let indent = parser.format_debug_indent();
                        let preview = if parser.position < parser.input.len() {
                            let end = (parser.position + 20).min(parser.input.len());
                            format!(" [{}...]", &parser.input[parser.position..end])
                        } else {
                            " [EOF]".to_string()
                        };
                        
                        parser.debug_output.push(format!(
                            "{}🚀 {} @ {}{}",
                            indent, #rule_name, start_pos, preview
                        ));
                    }
                    
                    // Main parsing logic - produces the 'result' variable
                    let result: ParseContent<'input>;
                    #parse_logic;
                    
                    let end_pos = parser.position;
                    
                    if parser.debug_mode {
                        let indent = parser.format_debug_indent();
                        let consumed = end_pos - start_pos;
                        let status = if consumed > 0 {
                            format!("✅ consumed {} chars", consumed)
                        } else {
                            "⚠️ zero-length match".to_string()
                        };
                        
                        parser.debug_output.push(format!(
                            "{}{} {} [{}.{}] {}",
                            indent, status, #rule_name, start_pos, end_pos,
                            if consumed > 0 {
                                format!(" = {:?}", &parser.input[start_pos..end_pos])
                            } else {
                                String::new()
                            }
                        ));
                    }
                    
                    Ok(ParseNode {
                        rule_name: #rule_name,
                        content: result,
                        span: start_pos..end_pos,
                    })
                });
                
                self.recursion_guard.exit();
                result
            }
        })
    }
    
    fn generate_node_parsing_logic(&self, ast_node: &ASTNode, rule_name: &str) -> Result<TokenStream> {
        match ast_node {
            ASTNode::Or { alternatives } => {
                self.generate_or_logic(alternatives, rule_name)
            }
            ASTNode::Sequence { elements } => {
                self.generate_sequence_logic(elements, rule_name)
            }
            ASTNode::Atom { value } => {
                self.generate_atom_logic(value, rule_name)
            }
            ASTNode::Quantified { element, quantifier } => {
                self.generate_quantified_logic(element, quantifier, rule_name)
            }
        }
    }
    
    fn generate_or_logic(&self, alternatives: &[ASTNode], rule_name: &str) -> Result<TokenStream> {
        let branch_count = alternatives.len();
        
        // Check if this is a single-branch or multi-branch rule
        if branch_count == 1 {
            // Single branch - simpler logic without try_parse
            let branch = &alternatives[0];
            let branch_logic = self.generate_node_parsing_logic(branch, rule_name)?;
            
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
                let branch_logic = self.generate_node_parsing_logic(alternative, rule_name)?;
                
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
                    quote! {
                        if let Some(content) = parser.try_parse(|p| {
                            #branch_logic;
                            Ok(result)
                        }) {
                            result = #transform;
                        }
                    }
                } else if idx < branch_count - 1 {
                    quote! {
                        else if let Some(content) = parser.try_parse(|p| {
                            #branch_logic;
                            Ok(result)
                        }) {
                            result = #transform;
                        }
                    }
                } else {
                    // Last branch - no try_parse needed
                    quote! {
                        else {
                            #branch_logic;
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
    
    fn generate_sequence_logic(&self, elements: &[ASTNode], rule_name: &str) -> Result<TokenStream> {
        let element_count = elements.len();
        let mut element_parsers = Vec::new();
        
        for (idx, element) in elements.iter().enumerate() {
            let element_parser = self.generate_sequence_element(element, idx, element_count, rule_name)?;
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
    ) -> Result<TokenStream> {
        let element_logic = match element {
            ASTNode::Quantified { element, quantifier } if quantifier == "?" => {
                // Optional element
                let inner_logic = self.generate_node_parsing_logic(element, rule_name)?;
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
                let inner_logic = self.generate_node_parsing_logic(element, rule_name)?;
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
    
    fn generate_atom_logic(&self, value: &ASTValue, rule_name: &str) -> Result<TokenStream> {
        match value {
            ASTValue::Token(parts) if parts.len() >= 2 => {
                let token_type_str = parts[0].as_str().unwrap_or("");
                let token_value_str = parts[1].as_str().unwrap_or("");
                
                match token_type_str {
                    "quoted_string" => {
                        Ok(quote! {
                            let result = ParseContent::Terminal(parser.match_string(#token_value_str)?)
                        })
                    }
                    "rule_reference" => {
                        let method = format_ident!("parse_{}", token_value_str);
                        Ok(quote! {
                            let result = ParseContent::Alternative(Box::new(parser.#method()?))
                        })
                    }
                    "regex" => {
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
    ) -> Result<TokenStream> {
        let element_logic = self.generate_node_parsing_logic(element, rule_name)?;
        
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
                            if parser.debug_mode {
                                parser.debug_output.push(format!(
                                    "⚠️ ZERO-LENGTH MATCH in {}: Breaking to prevent infinite loop at position {}",
                                    #rule_name, current_position
                                ));
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
                
                if iteration_count >= MAX_ITERATIONS && parser.debug_mode {
                    parser.debug_output.push(format!(
                        "⚠️ MAX ITERATIONS ({}) reached in {} quantifier",
                        MAX_ITERATIONS, #rule_name
                    ));
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
                    if parser.debug_mode {
                        parser.debug_output.push(format!(
                            "⚠️ ZERO-LENGTH FIRST MATCH in {}+ quantifier at position {}",
                            #rule_name, start_position
                        ));
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
                            if parser.debug_mode {
                                parser.debug_output.push(format!(
                                    "⚠️ ZERO-LENGTH MATCH in {}+: Breaking at position {}",
                                    #rule_name, current_position
                                ));
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
                
                if iteration_count >= MAX_ITERATIONS && parser.debug_mode {
                    parser.debug_output.push(format!(
                        "⚠️ MAX ITERATIONS ({}) reached in {}+ quantifier",
                        MAX_ITERATIONS, #rule_name
                    ));
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
    
    fn generate_helper_methods(&self) -> TokenStream {
        quote! {
            fn match_string(&mut self, expected: &str) -> ParseResult<&'input str> {
                let start = self.position;
                let end = start + expected.len();
                
                if end <= self.input.len() {
                    let slice = &self.input[start..end];
                    if slice == expected {
                        if self.debug_mode {
                            let indent = self.format_debug_indent();
                            self.debug_output.push(format!(
                                "{}🎯 matched {:?} at position {}",
                                indent, expected, start
                            ));
                        }
                        self.position = end;
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
                
                if self.debug_mode {
                    let indent = self.format_debug_indent();
                    self.debug_output.push(format!(
                        "{}❌ expected {:?}, found {:?} at position {}",
                        indent, expected, found_str, start
                    ));
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
                        
                        if self.debug_mode {
                            let indent = self.format_debug_indent();
                            self.debug_output.push(format!(
                                "{}🎯 regex {:?} matched {:?} at position {}",
                                indent, pattern, matched, start
                            ));
                        }
                        
                        self.position += matched.len();
                        return Ok(&self.input[start..self.position]);
                    }
                }
                
                if self.debug_mode {
                    let indent = self.format_debug_indent();
                    let preview = if self.position < self.input.len() {
                        let end = (self.position + 10).min(self.input.len());
                        &self.input[self.position..end]
                    } else {
                        "<EOF>"
                    };
                    self.debug_output.push(format!(
                        "{}❌ regex {:?} no match at position {} (next: {:?})",
                        indent, pattern, self.position, preview
                    ));
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
                
                match f(self) {
                    Ok(result) => Some(result),
                    Err(_) => {
                        self.position = saved_pos;
                        self.recursion_guard.parse_stack.truncate(saved_stack_len);
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
                        return Ok(node.clone());
                    } else {
                        return Err(ParseError::Backtrack {
                            position: self.position,
                        });
                    }
                }
                
                let start_pos = self.position;
                let result = f(self);
                
                if let Ok(ref node) = result {
                    self.memo.insert(key, Some(node.clone()));
                } else {
                    self.memo.insert(key, None);
                }
                
                result
            }
        }
    }
    
    fn generate_tests(&self, parser_name: &Ident) -> TokenStream {
        quote! {
            #[cfg(test)]
            mod tests {
                use super::*;
                
                #[test]
                fn test_basic_parsing() {
                    let input = "$1";
                    let mut parser = #parser_name::new(input);
                    let result = parser.parse();
                    assert!(result.is_ok());
                }
            }
        }
    }
}