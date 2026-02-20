// AST-Based Parser Generator using syn and quote
// This module replaces string concatenation with proper AST manipulation
// GUARANTEES: No unbalanced braces, no syntax errors, type-safe code generation

use super::Logger;
use crate::ast_pipeline::{
    ASTNode, ASTValue, Annotations, BranchAnnotation, SemanticAnnotation, SemanticAssociativity,
    SemanticBranchPolicy, SemanticValueConstraints, TokenValue, UnifiedSemanticAST,
    ast_return_transform::AstReturnTransformer, extract_semantic_directive,
    normalize_semantic_scalar, parse_canonical_transform_expression, parse_semantic_bool,
    parse_semantic_branch_priorities, parse_semantic_len_bounds, parse_semantic_numeric_bounds,
    parse_semantic_string_list,
};
use anyhow::Result;
use prettyplease;
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use std::collections::{HashMap, HashSet};
use syn::Ident;

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
        eprintln!(
            "📊  Grammar: '{}' with {} rules",
            self.grammar_name,
            rule_order.len()
        );
        eprintln!("🎯  Target: {}", filename);
        eprintln!("📂  File: {}:{}", file!(), line!());
        eprintln!();

        let parser_tokens = self.generate_parser_tokens(grammar_tree, rule_order, filename)?;

        eprintln!();
        eprintln!(
            "✅  TokenStream generation complete ({} tokens)",
            parser_tokens.to_string().len()
        );
        eprintln!("📂  File: {}:{}", file!(), line!());

        eprintln!();
        // Convert TokenStream to formatted string using prettyplease
        eprintln!("🎨  Converting TokenStream to formatted Rust code...");
        let formatted_code = prettyplease::unparse(&syn::parse2(parser_tokens)?);
        eprintln!(
            "✨  Code formatting complete ({} characters)",
            formatted_code.len()
        );
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
        eprintln!(
            "   🔧  Starting parser code generation for {} rules using AST-based approach",
            rule_order.len()
        );
        eprintln!("        File: {}:{}", file!(), line!());

        // Determine entry rule
        let entry_rule = self
            .entry_rule
            .as_ref()
            .map(|s| s.clone())
            .or_else(|| rule_order.first().cloned())
            .ok_or_else(|| anyhow::anyhow!("No entry rule found"))?;

        eprintln!("        Entry rule determined: '{}'", entry_rule);
        eprintln!("        File: {}:{}", file!(), line!());

        let parser_name = format_ident!(
            "{}Parser",
            self.grammar_name
                .chars()
                .next()
                .unwrap()
                .to_uppercase()
                .collect::<String>()
                + &self.grammar_name[1..]
        );

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
        let parser_impl = self.generate_parser_impl(
            &parser_name,
            grammar_tree,
            rule_order,
            &entry_rule,
            filename,
        )?;
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

        eprintln!(
            "        Combined all components into final TokenStream ({} chars)",
            result.to_string().len()
        );
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
                let method =
                    self.generate_rule_method(rule_name, ast_node, rule_order, filename)?;
                rule_methods.push(method);
                eprintln!("        ✓   Completed - File: {}:{}", file!(), line!());
                eprintln!();
                eprintln!();
            }
        }
        let unresolved_reference_methods =
            self.generate_unresolved_reference_methods(grammar_tree, rule_order);
        if !unresolved_reference_methods.is_empty() {
            eprintln!(
                "Generated {} unresolved reference fallback method(s) - File: {}:{}",
                unresolved_reference_methods.len(),
                file!(),
                line!()
            );
        }
        eprintln!(
            "All rule methods generated ({}) - File: {}:{}",
            rule_methods.len(),
            file!(),
            line!()
        );

        // Generate helper methods
        let helpers = self.generate_helper_methods(filename);

        Ok(quote! {
            impl<'input> #parser_name<'input> {
                #rule_constants
                #constructor
                #parse_method
                #(#rule_methods)*
                #(#unresolved_reference_methods)*
                #helpers
            }
        })
    }

    fn generate_rule_constants(&self, rule_order: &[String]) -> TokenStream {
        let constants: Vec<TokenStream> = rule_order
            .iter()
            .enumerate()
            .map(|(i, name)| {
                let const_name = format_ident!("RULE_{}", name.to_uppercase());
                let id = i as u16;
                quote! {
                    const #const_name: RuleId = #id;
                }
            })
            .collect();

        quote! {
            #(#constants)*
        }
    }

    fn generate_unresolved_reference_methods(
        &self,
        grammar_tree: &HashMap<String, ASTNode>,
        rule_order: &[String],
    ) -> Vec<TokenStream> {
        let known_rules: HashSet<&str> = rule_order.iter().map(|rule| rule.as_str()).collect();
        let mut referenced_rules: HashSet<String> = HashSet::new();
        for rule_name in rule_order {
            if let Some(ast_node) = grammar_tree.get(rule_name) {
                Self::collect_rule_references(ast_node, &mut referenced_rules);
            }
        }

        let mut unresolved: Vec<String> = referenced_rules
            .into_iter()
            .filter(|rule| !known_rules.contains(rule.as_str()))
            .collect();
        unresolved.sort();
        unresolved.dedup();

        unresolved
            .iter()
            .map(|rule| self.generate_unresolved_reference_method(rule))
            .collect()
    }

    fn collect_rule_references(node: &ASTNode, out: &mut HashSet<String>) {
        match node {
            ASTNode::Or { alternatives } => {
                for alt in alternatives {
                    Self::collect_rule_references(alt, out);
                }
            }
            ASTNode::Sequence { elements } => {
                for element in elements {
                    Self::collect_rule_references(element, out);
                }
            }
            ASTNode::Quantified { element, .. } => {
                Self::collect_rule_references(element, out);
            }
            ASTNode::Atom { value } => {
                if let ASTValue::Token(parts) = value {
                    if parts.len() >= 2 {
                        if let (
                            TokenValue::String(token_type),
                            TokenValue::String(token_value),
                        ) = (&parts[0], &parts[1])
                        {
                            if token_type == "rule_reference" {
                                out.insert(token_value.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    fn generate_unresolved_reference_method(&self, rule_name: &str) -> TokenStream {
        let method_name = format_ident!("parse_{}", rule_name);

        match rule_name {
            "true" => quote! {
                pub fn #method_name(&mut self) -> ParseResult<ParseNode<'input>> {
                    let start_pos = self.position;
                    Ok(ParseNode {
                        rule_name: #rule_name,
                        content: ParseContent::Terminal("true"),
                        span: start_pos..start_pos,
                    })
                }
            },
            "false" => quote! {
                pub fn #method_name(&mut self) -> ParseResult<ParseNode<'input>> {
                    let start_pos = self.position;
                    Ok(ParseNode {
                        rule_name: #rule_name,
                        content: ParseContent::Terminal("false"),
                        span: start_pos..start_pos,
                    })
                }
            },
            "semantic_annotation" => quote! {
                pub fn #method_name(&mut self) -> ParseResult<ParseNode<'input>> {
                    let checkpoint = self.position;
                    self.consume_optional_whitespace();
                    let start_pos = self.position;
                    if start_pos >= self.input.len() || self.input.as_bytes()[start_pos] != b'@' {
                        self.position = checkpoint;
                        return Err(ParseError::Backtrack {
                            position: checkpoint,
                        });
                    }

                    while self.position < self.input.len() {
                        let b = self.input.as_bytes()[self.position];
                        if b == b'\n' || b == b'\r' {
                            break;
                        }
                        self.position += 1;
                    }

                    let end_pos = self.position;
                    let matched = &self.input[start_pos..end_pos];
                    Ok(ParseNode {
                        rule_name: #rule_name,
                        content: ParseContent::Terminal(matched),
                        span: start_pos..end_pos,
                    })
                }
            },
            _ => quote! {
                pub fn #method_name(&mut self) -> ParseResult<ParseNode<'input>> {
                    Err(ParseError::Backtrack {
                        position: self.position,
                    })
                }
            },
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
        let parse_full_method = format_ident!("parse_full_{}", entry_rule);

        quote! {
            pub fn parse(&mut self) -> ParseResult<ParseNode<'input>> {
                self.#parse_method()
            }

            pub fn parse_full(&mut self) -> ParseResult<ParseNode<'input>> {
                let parsed = self.#parse_method()?;
                // Allow trailing layout/comments so parse_full reports structural completeness.
                self.consume_layout_for_terminal("<EOF>");
                if self.position == self.input.len() {
                    Ok(parsed)
                } else {
                    Err(ParseError::InvalidSyntax {
                        message: "Parser did not consume full input",
                        position: self.position,
                    })
                }
            }

            pub fn #parse_full_method(&mut self) -> ParseResult<ParseNode<'input>> {
                self.parse_full()
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

        eprintln!(
            "        ↳   Entering rule processing block - File: {}:{}",
            file!(),
            line!()
        );

        eprintln!();
        // Generate the parsing logic based on AST node type
        let parse_logic = self.generate_node_parsing_logic(ast_node, rule_name, filename)?;

        eprintln!();
        eprintln!(
            "            File: {}:{}: Exiting rule processing block",
            file!(),
            line!()
        );

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
                                let consumed_preview = self.byte_window_lossy(start_pos, node.span.end);
                                self.logger.log_success(#filename, 0, &format!("✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')", #rule_name, start_pos, node.span.end, consumed, consumed_preview));
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

    fn generate_node_parsing_logic(
        &self,
        ast_node: &ASTNode,
        rule_name: &str,
        filename: &str,
    ) -> Result<TokenStream> {
        eprintln!(
            "   🔍  Generating parsing logic for rule '{}' with AST node type: {:?} - File: {}:{}",
            rule_name,
            ast_node,
            file!(),
            line!()
        );

        match ast_node {
            ASTNode::Or { alternatives } => {
                eprintln!(
                    "        Processing OR node with {} alternatives - File: {}:{}",
                    alternatives.len(),
                    file!(),
                    line!()
                );
                self.generate_or_logic(alternatives, rule_name, filename)
            }
            ASTNode::Sequence { elements } => {
                eprintln!(
                    "        Processing sequence node with {} elements - File: {}:{}",
                    elements.len(),
                    file!(),
                    line!()
                );
                self.generate_sequence_logic(elements, rule_name, filename)
            }
            ASTNode::Atom { value } => {
                eprintln!(
                    "        Processing atom node - File: {}:{}",
                    file!(),
                    line!()
                );
                self.generate_atom_logic(value, rule_name, filename)
            }
            ASTNode::Quantified {
                element,
                quantifier,
            } => {
                eprintln!(
                    "        Processing quantified node with '{}' quantifier - File: {}:{}",
                    quantifier,
                    file!(),
                    line!()
                );
                self.generate_quantified_logic(element, quantifier, rule_name, filename)
            }
        }
    }

    fn generate_or_logic(
        &self,
        alternatives: &[ASTNode],
        rule_name: &str,
        filename: &str,
    ) -> Result<TokenStream> {
        let branch_count = alternatives.len();

        // Check if this is a single-branch or multi-branch rule
        if branch_count == 1 {
            // Single branch - simpler logic without try_parse
            let branch = &alternatives[0];
            eprintln!();
            let branch_logic = self.generate_node_parsing_logic(branch, rule_name, filename)?;

            // Check for return annotations for this rule
            let has_transform =
                if let Some(branches) = self.branch_return_annotations.get(rule_name) {
                    branches.get(0).map(|ann| ann.is_some()).unwrap_or(false)
                } else {
                    false
                };

            if has_transform {
                // Has return annotation - need to transform
                let transform = self
                    .branch_return_annotations
                    .get(rule_name)
                    .and_then(|branches| branches.get(0))
                    .and_then(|ann| ann.as_ref())
                    .map(|annotation| {
                        self.generate_return_transform(
                            annotation,
                            rule_name,
                            &["result".to_string()],
                        )
                    })
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
            // Multi-branch - evaluate all branches and keep the longest successful match
            let mut branch_attempts = Vec::new();
            let branch_priorities = self.rule_branch_priorities(rule_name, branch_count);
            let associativity = self.rule_associativity(rule_name);
            let associativity_mode = associativity.as_str();
            let branch_policy = self.rule_branch_policy(rule_name);
            let branch_policy_mode = branch_policy.as_str();
            let (recover_enabled, sync_tokens, panic_until_tokens) =
                self.rule_recovery_hints(rule_name);
            let sync_tokens_label = sync_tokens.join(", ");
            let panic_until_tokens_label = panic_until_tokens.join(", ");
            let sync_tokens_for_code = sync_tokens.clone();
            let panic_until_tokens_for_code = panic_until_tokens.clone();

            let recovery_failure_path = if recover_enabled {
                quote! {
                    if parser.recover_with_hints(
                        #rule_name,
                        parse_start,
                        &[#(#sync_tokens_for_code),*],
                        &[#(#panic_until_tokens_for_code),*],
                    ) {
                        if parser.logger.is_enabled() {
                            parser.logger.log_warning(#filename, 0, &format!(
                                "🛟 Rule '{}' recovered from branch failure using sync=[{}] panic_until=[{}]",
                                #rule_name,
                                #sync_tokens_label,
                                #panic_until_tokens_label
                            ));
                        }
                        result = ParseContent::Sequence(Vec::new());
                    } else {
                        return Err(ParseError::Backtrack {
                            position: parse_start,
                        });
                    }
                }
            } else {
                quote! {
                    return Err(ParseError::Backtrack {
                        position: parse_start,
                    });
                }
            };

            for (idx, alternative) in alternatives.iter().enumerate() {
                eprintln!();
                let branch_logic =
                    self.generate_node_parsing_logic(alternative, rule_name, filename)?;

                // Check for branch-specific return annotation
                let transform =
                    if let Some(branches) = self.branch_return_annotations.get(rule_name) {
                        if let Some(Some(annotation)) = branches.get(idx) {
                            self.generate_return_transform(
                                annotation,
                                rule_name,
                                &["content".to_string()],
                            )?
                        } else {
                            quote! { content }
                        }
                    } else {
                        quote! { content }
                    };

                let branch_num = idx + 1;
                let branch_priority = branch_priorities.get(idx).copied().unwrap_or(0);
                let branch_index = idx;
                branch_attempts.push(quote! {
                    if #branch_policy_mode == "ordered" && best_content.is_some() {
                        // Ordered branch policy keeps first successful branch.
                    } else {
                        parser.position = parse_start;
                        if let Some(content) = parser.try_parse(|p| {
                            let parser = p;
                            if parser.logger.is_enabled() {
                                parser.logger.log_info(#filename, 0, &format!("🚪 Entering branch {}/{} for rule '{}' at position {}", #branch_num, #branch_count, #rule_name, parser.position));
                            }
                            #branch_logic;
                            if parser.logger.is_enabled() {
                                parser.logger.log_info(#filename, 0, &format!("✅ Leaving branch {}/{} for rule '{}' at position {} (success)", #branch_num, #branch_count, #rule_name, parser.position));
                            }
                            Ok(result)
                        }) {
                            let candidate_end = parser.position;
                            parser.position = parse_start;
                            let candidate_priority: i64 = #branch_priority;
                            let transformed = {
                                let content = content;
                                #transform
                            };
                            let should_take = if #branch_policy_mode == "ordered" {
                                best_content.is_none()
                            } else if #branch_policy_mode == "priority_first" {
                                if best_content.is_none() {
                                    true
                                } else if candidate_priority > best_priority {
                                    true
                                } else if candidate_priority < best_priority {
                                    false
                                } else if candidate_end > best_end {
                                    true
                                } else if candidate_end < best_end {
                                    false
                                } else {
                                    match #associativity_mode {
                                        "right" => #branch_index > best_branch_index,
                                        "nonassoc" => {
                                            if #branch_index != best_branch_index {
                                                nonassoc_tie = true;
                                            }
                                            false
                                        }
                                        _ => false,
                                    }
                                }
                            } else if best_content.is_none() {
                                true
                            } else if candidate_end > best_end {
                                true
                            } else if candidate_end < best_end {
                                false
                            } else if candidate_priority > best_priority {
                                true
                            } else if candidate_priority < best_priority {
                                false
                            } else {
                                match #associativity_mode {
                                    "right" => #branch_index > best_branch_index,
                                    "nonassoc" => {
                                        if #branch_index != best_branch_index {
                                            nonassoc_tie = true;
                                        }
                                        false
                                    }
                                    _ => false,
                                }
                            };

                            if should_take {
                                best_end = candidate_end;
                                best_priority = candidate_priority;
                                best_branch_index = #branch_index;
                                best_branch = #branch_num;
                                best_content = Some(transformed);
                            }
                        } else if parser.logger.is_enabled() {
                            parser.logger.log_info(#filename, 0, &format!("❌ Branch {}/{} for rule '{}' failed at position {}", #branch_num, #branch_count, #rule_name, parser.position));
                        }
                    }
                });
            }

            Ok(quote! {
                // Multi-branch parsing logic (branch-policy guided)
                let parse_start = parser.position;
                let mut best_content: Option<ParseContent<'input>> = None;
                let mut best_end = parse_start;
                let mut best_priority: i64 = i64::MIN;
                let mut best_branch_index: usize = 0usize;
                let mut best_branch = 0usize;
                let mut nonassoc_tie = false;
                let mut result = ParseContent::Sequence(Vec::new());
                #(#branch_attempts)*

                if nonassoc_tie {
                    return Err(ParseError::Backtrack {
                        position: parse_start,
                    });
                } else if let Some(content) = best_content {
                    parser.position = best_end;
                    if parser.logger.is_enabled() {
                        parser.logger.log_info(#filename, 0, &format!(
                            "🏁 Rule '{}' selected branch {}/{} consuming {} chars (priority={}, associativity={}, branch_policy={})",
                            #rule_name,
                            best_branch,
                            #branch_count,
                            best_end.saturating_sub(parse_start),
                            best_priority,
                            #associativity_mode,
                            #branch_policy_mode
                        ));
                    }
                    result = content;
                } else {
                    #recovery_failure_path
                }
            })
        }
    }

    fn generate_sequence_logic(
        &self,
        elements: &[ASTNode],
        rule_name: &str,
        filename: &str,
    ) -> Result<TokenStream> {
        let element_count = elements.len();
        let mut element_parsers = Vec::new();

        for (idx, element) in elements.iter().enumerate() {
            let element_parser =
                self.generate_sequence_element(element, idx, element_count, rule_name, filename)?;
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
            ASTNode::Quantified {
                element,
                quantifier,
            } if quantifier == "?" => {
                // Optional element
                eprintln!();
                let inner_logic = self.generate_node_parsing_logic(element, rule_name, filename)?;
                quote! {
                    if let Some(content) = parser.try_parse(|p| {
                        let parser = p;
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

    fn generate_atom_logic(
        &self,
        value: &ASTValue,
        rule_name: &str,
        filename: &str,
    ) -> Result<TokenStream> {
        eprintln!(
            "        Processing atom value: {:?} - File: {}:{}",
            value,
            file!(),
            line!()
        );

        let value_constraints = self.rule_value_constraints(rule_name);

        match value {
            ASTValue::Token(parts) if parts.len() >= 2 => {
                let token_type_str = if let TokenValue::String(ref s) = parts[0] {
                    s.as_str()
                } else {
                    ""
                };
                let token_value_str = if let TokenValue::String(ref s) = parts[1] {
                    s.as_str()
                } else {
                    ""
                };

                eprintln!(
                    "        Token type: '{}', value: '{}' - File: {}:{}",
                    token_type_str,
                    token_value_str,
                    file!(),
                    line!()
                );

                match token_type_str {
                    "quoted_string" => {
                        eprintln!(
                            "        Generating string terminal matcher for '{}' - File: {}:{}",
                            token_value_str,
                            file!(),
                            line!()
                        );
                        let constraint_guards =
                            self.semantic_value_constraint_tokens(rule_name, &value_constraints);
                        Ok(quote! {
                            let matched_str = parser.match_string(#token_value_str)?;
                            #constraint_guards
                            let result = ParseContent::Terminal(matched_str)
                        })
                    }
                    "rule_reference" => {
                        eprintln!(
                            "        Generating rule reference call to '{}' - File: {}:{}",
                            token_value_str,
                            file!(),
                            line!()
                        );
                        let method = format_ident!("parse_{}", token_value_str);
                        Ok(quote! {
                            let result = ParseContent::Alternative(Box::new(parser.#method()?))
                        })
                    }
                    "regex" => {
                        eprintln!(
                            "        Generating regex matcher for pattern '{}' - File: {}:{}",
                            token_value_str,
                            file!(),
                            line!()
                        );
                        let skip_leading_whitespace =
                            !matches!(rule_name, "string_content_double" | "string_content_single");
                        let effective_regex_pattern = if self.grammar_name == "semantic_annotation"
                            && rule_name == "identifier_literal"
                            && token_value_str == "([a-zA-Z_][a-zA-Z0-9_]*)"
                        {
                            "([a-zA-Z_][a-zA-Z0-9_]*(?:\\.[a-zA-Z_][a-zA-Z0-9_]*)*)".to_string()
                        } else {
                            token_value_str.to_string()
                        };
                        // Check for semantic annotations that should transform the matched string
                        if let Some(annotations) = &self.annotations {
                            if let Some(semantic_annotations) =
                                annotations.semantic_annotations.get(rule_name)
                            {
                                for semantic_annotation in semantic_annotations {
                                    if Self::semantic_directive_name(semantic_annotation).as_deref()
                                        != Some("transform")
                                    {
                                        continue;
                                    }

                                    if let UnifiedSemanticAST::TransformExpr { expression } =
                                        semantic_annotation.ast()
                                    {
                                        if let Some(transform) =
                                            parse_canonical_transform_expression(expression)
                                        {
                                            if let Ok(target_type) =
                                                syn::parse_str::<syn::Type>(&transform.target_type)
                                            {
                                                let default_expr: syn::Expr =
                                                    syn::parse_str(&transform.default_expr)
                                                        .unwrap_or_else(|_| {
                                                            syn::parse_str("0").expect(
                                                                "fallback default expression",
                                                            )
                                                        });

                                                // Canonical transform path: parse matched regex token into target type.
                                                let constraint_guards = self
                                                    .semantic_value_constraint_tokens(
                                                        rule_name,
                                                        &value_constraints,
                                                    );
                                                if self.enable_debug {
                                                    return Ok(quote! {
                                                        let matched_str = parser.match_regex(#effective_regex_pattern, #skip_leading_whitespace)?;
                                                        #constraint_guards
                                                        let transformed = matched_str.parse::<#target_type>().unwrap_or(#default_expr);
                                                        parser.debug_output.push(format!("🎯 Applied semantic transform: parsed '{}' to {}={}", matched_str, stringify!(#target_type), transformed));
                                                        let result = ParseContent::TransformedTerminal(transformed.to_string())
                                                    });
                                                } else {
                                                    return Ok(quote! {
                                                        let matched_str = parser.match_regex(#effective_regex_pattern, #skip_leading_whitespace)?;
                                                        #constraint_guards
                                                        let transformed = matched_str.parse::<#target_type>().unwrap_or(#default_expr);
                                                        let result = ParseContent::TransformedTerminal(transformed.to_string())
                                                    });
                                                }
                                            }
                                        }

                                        // Fallback: treat as raw expression
                                        let constraint_guards = self
                                            .semantic_value_constraint_tokens(
                                                rule_name,
                                                &value_constraints,
                                            );
                                        if self.enable_debug {
                                            return Ok(quote! {
                                                let matched_str = parser.match_regex(#effective_regex_pattern, #skip_leading_whitespace)?;
                                                #constraint_guards
                                                parser.debug_output.push(format!("🎯 Applied semantic transform: raw expression '{}' to rule '{}': matched '{}'", #expression, #rule_name, matched_str));
                                                let result = ParseContent::TransformedTerminal(#expression.to_string())
                                            });
                                        } else {
                                            return Ok(quote! {
                                                let matched_str = parser.match_regex(#effective_regex_pattern, #skip_leading_whitespace)?;
                                                #constraint_guards
                                                let result = ParseContent::TransformedTerminal(#expression.to_string())
                                            });
                                        }
                                    }
                                }
                            }
                        }

                        // Default behavior: return matched string as terminal
                        let constraint_guards =
                            self.semantic_value_constraint_tokens(rule_name, &value_constraints);
                        Ok(quote! {
                            let matched_str = parser.match_regex(#effective_regex_pattern, #skip_leading_whitespace)?;
                            #constraint_guards
                            let result = ParseContent::Terminal(matched_str)
                        })
                    }
                    "number" | "probability" | "include_dir" | "include_file" | "rule" => {
                        eprintln!(
                            "        Generating literal matcher for token type '{}' value '{}' - File: {}:{}",
                            token_type_str,
                            token_value_str,
                            file!(),
                            line!()
                        );
                        let constraint_guards =
                            self.semantic_value_constraint_tokens(rule_name, &value_constraints);
                        Ok(quote! {
                            let matched_str = parser.match_string(#token_value_str)?;
                            #constraint_guards
                            let result = ParseContent::Terminal(matched_str)
                        })
                    }
                    _ => Ok(quote! {
                        let result = ParseContent::Terminal("")
                    }),
                }
            }
            _ => Ok(quote! {
                let result = ParseContent::Terminal("")
            }),
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
        // Optional semantic guard for line-delimited declaration grammars.
        // Example usage in a grammar:
        //   @stop_at_rule_boundary: true
        //   sequence := sequence_element+
        let stop_at_rule_boundary = self.rule_has_semantic_bool_directive(
            rule_name,
            &["stop_at_rule_boundary", "stop_on_rule_boundary", "line_delimited_sequence"],
        );

        match quantifier {
            "*" => Ok(quote! {
                let mut results = Vec::new();
                let mut last_position = parser.position;
                let mut iteration_count = 0;
                const MAX_ITERATIONS: usize = 10000; // Safety limit

                while iteration_count < MAX_ITERATIONS {
                    if #stop_at_rule_boundary && parser.looks_like_rule_definition_boundary() {
                        break;
                    }
                    if let Some(node) = parser.try_parse(|p| {
                        let parser = p;
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

                if #stop_at_rule_boundary && parser.looks_like_rule_definition_boundary() {
                    return Err(ParseError::Backtrack {
                        position: parser.position,
                    });
                }

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
                    if #stop_at_rule_boundary && parser.looks_like_rule_definition_boundary() {
                        break;
                    }
                    if let Some(node) = parser.try_parse(|p| {
                        let parser = p;
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
                    let parser = p;
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
            _ => Err(anyhow::anyhow!("Unknown quantifier: {}", quantifier)),
        }
    }

    fn generate_return_transform(
        &self,
        annotation: &BranchAnnotation,
        rule_name: &str,
        captured_vars: &[String],
    ) -> Result<TokenStream> {
        eprintln!(
            "DEBUG: generate_return_transform called for rule '{}', parsed_ast is {}",
            rule_name,
            if annotation.parsed_ast.is_some() {
                "Some"
            } else {
                "None"
            }
        );

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
            eprintln!(
                "DEBUG: Adding warning comment for rule '{}' with annotation '{}'",
                rule_name, annotation.annotation_content
            );
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
            fn byte_window_lossy(&self, start: usize, end: usize) -> String {
                if start >= end || start >= self.input.len() {
                    return String::new();
                }
                let clamped_end = end.min(self.input.len());
                String::from_utf8_lossy(&self.input.as_bytes()[start..clamped_end]).to_string()
            }
            fn bytes_match_at(&self, start: usize, expected: &[u8]) -> bool {
                let Some(end) = start.checked_add(expected.len()) else {
                    return false;
                };
                if end > self.input.len() {
                    return false;
                }
                &self.input.as_bytes()[start..end] == expected
            }
            fn find_token_from(&self, start: usize, token: &str) -> Option<usize> {
                if token.is_empty() || start >= self.input.len() {
                    return None;
                }
                let token_bytes = token.as_bytes();
                if token_bytes.len() > self.input.len() {
                    return None;
                }
                let max_start = self.input.len().saturating_sub(token_bytes.len());
                for idx in start..=max_start {
                    if self.bytes_match_at(idx, token_bytes) {
                        return Some(idx);
                    }
                }
                None
            }
            fn recover_with_hints(
                &mut self,
                rule_name: &str,
                parse_start: usize,
                sync_tokens: &[&str],
                panic_until_tokens: &[&str],
            ) -> bool {
                let recovery_start = parse_start.min(self.input.len());
                let mut best: Option<(usize, usize, u8)> = None;

                for token in panic_until_tokens {
                    if token.is_empty() {
                        continue;
                    }
                    if let Some(pos) = self.find_token_from(recovery_start, token) {
                        let candidate = (pos, token.len(), 0u8);
                        let take_candidate = match best {
                            None => true,
                            Some((best_pos, _best_len, best_priority)) => {
                                pos < best_pos || (pos == best_pos && candidate.2 < best_priority)
                            }
                        };
                        if take_candidate {
                            best = Some(candidate);
                        }
                    }
                }

                for token in sync_tokens {
                    if token.is_empty() {
                        continue;
                    }
                    if let Some(pos) = self.find_token_from(recovery_start, token) {
                        let candidate = (pos, token.len(), 1u8);
                        let take_candidate = match best {
                            None => true,
                            Some((best_pos, _best_len, best_priority)) => {
                                pos < best_pos || (pos == best_pos && candidate.2 < best_priority)
                            }
                        };
                        if take_candidate {
                            best = Some(candidate);
                        }
                    }
                }

                if let Some((token_pos, token_len, token_priority)) = best {
                    let previous = self.position;
                    let token_end = token_pos.saturating_add(token_len).min(self.input.len());
                    let mut new_position = token_end;
                    if new_position <= previous && previous < self.input.len() {
                        new_position = previous + 1;
                    }
                    self.position = new_position.min(self.input.len());

                    if self.logger.is_enabled() {
                        let marker = if token_priority == 0 {
                            "panic_until"
                        } else {
                            "sync"
                        };
                        self.logger.log_warning(#filename, 0, &format!(
                            "🛟 Recovery for rule '{}': moved parser from {} to {} using {} token at {}",
                            rule_name,
                            previous,
                            self.position,
                            marker,
                            token_pos
                        ));
                    }
                    return self.position > parse_start;
                }

                if self.position < self.input.len() {
                    let previous = self.position;
                    self.position = self.input.len();
                    if self.logger.is_enabled() {
                        self.logger.log_warning(#filename, 0, &format!(
                            "🛟 Recovery for rule '{}': no sync/panic token found, skipped to EOF ({} -> {})",
                            rule_name,
                            previous,
                            self.position
                        ));
                    }
                    return true;
                }

                false
            }
            fn consume_optional_whitespace(&mut self) {
                while self.position < self.input.len() {
                    let b = self.input.as_bytes()[self.position];
                    if matches!(b, b' ' | b'\t' | b'\n' | b'\r') {
                        self.position += 1;
                    } else {
                        break;
                    }
                }
            }
            fn consume_horizontal_whitespace(&mut self) {
                while self.position < self.input.len() {
                    let b = self.input.as_bytes()[self.position];
                    if matches!(b, b' ' | b'\t') {
                        self.position += 1;
                    } else {
                        break;
                    }
                }
            }
            fn consume_layout_for_terminal(&mut self, expected: &str) {
                // Skip comments as layout for structural terminals, but avoid swallowing
                // comment-introducer tokens themselves.
                let allow_comment_skip = expected != "#"
                    && expected != "//"
                    && expected != "/*"
                    && expected != "/**"
                    && expected != "///"
                    && expected != "/";

                loop {
                    let before = self.position;
                    self.consume_optional_whitespace();

                    if !allow_comment_skip || self.position >= self.input.len() {
                        break;
                    }

                    let bytes = self.input.as_bytes();
                    let len = bytes.len();

                    if bytes[self.position] == b'#' {
                        while self.position < self.input.len() {
                            let b = bytes[self.position];
                            if b == b'\n' || b == b'\r' {
                                break;
                            }
                            self.position += 1;
                        }
                        continue;
                    }

                    if self.position + 1 < len
                        && bytes[self.position] == b'/'
                        && bytes[self.position + 1] == b'/'
                    {
                        self.position += 2;
                        while self.position < self.input.len() {
                            let b = bytes[self.position];
                            if b == b'\n' || b == b'\r' {
                                break;
                            }
                            self.position += 1;
                        }
                        continue;
                    }

                    if self.position + 1 < len
                        && bytes[self.position] == b'/'
                        && bytes[self.position + 1] == b'*'
                    {
                        self.position += 2;
                        while self.position + 1 < len
                            && !(bytes[self.position] == b'*' && bytes[self.position + 1] == b'/')
                        {
                            self.position += 1;
                        }
                        if self.position + 1 < len {
                            self.position += 2;
                        }
                        continue;
                    }

                    if self.position == before {
                        break;
                    }
                }
            }
            fn consume_layout_for_regex(&mut self, can_match_empty: bool) {
                if can_match_empty {
                    // Empty-matching regexes must not cross line boundaries implicitly.
                    self.consume_horizontal_whitespace();
                    return;
                }

                loop {
                    let before = self.position;
                    self.consume_optional_whitespace();

                    if self.position >= self.input.len() {
                        break;
                    }

                    let bytes = self.input.as_bytes();
                    let len = bytes.len();

                    if bytes[self.position] == b'#' {
                        while self.position < self.input.len() {
                            let b = bytes[self.position];
                            if b == b'\n' || b == b'\r' {
                                break;
                            }
                            self.position += 1;
                        }
                        continue;
                    }

                    if self.position + 1 < len
                        && bytes[self.position] == b'/'
                        && bytes[self.position + 1] == b'/'
                    {
                        self.position += 2;
                        while self.position < self.input.len() {
                            let b = bytes[self.position];
                            if b == b'\n' || b == b'\r' {
                                break;
                            }
                            self.position += 1;
                        }
                        continue;
                    }

                    if self.position + 1 < len
                        && bytes[self.position] == b'/'
                        && bytes[self.position + 1] == b'*'
                    {
                        self.position += 2;
                        while self.position + 1 < len
                            && !(bytes[self.position] == b'*' && bytes[self.position + 1] == b'/')
                        {
                            self.position += 1;
                        }
                        if self.position + 1 < len {
                            self.position += 2;
                        }
                        continue;
                    }

                    if self.position == before {
                        break;
                    }
                }
            }
            fn looks_like_rule_definition_boundary(&self) -> bool {
                let bytes = self.input.as_bytes();
                let len = bytes.len();
                let mut i = self.position;
                let mut saw_newline = false;

                while i < len {
                    match bytes[i] {
                        b' ' | b'\t' => {
                            i += 1;
                            continue;
                        }
                        b'\n' | b'\r' => {
                            saw_newline = true;
                            i += 1;
                            continue;
                        }
                        b'#' => {
                            while i < len && bytes[i] != b'\n' && bytes[i] != b'\r' {
                                i += 1;
                            }
                            continue;
                        }
                        b'/' if i + 1 < len && bytes[i + 1] == b'/' => {
                            i += 2;
                            while i < len && bytes[i] != b'\n' && bytes[i] != b'\r' {
                                i += 1;
                            }
                            continue;
                        }
                        b'/' if i + 1 < len && bytes[i + 1] == b'*' => {
                            i += 2;
                            while i + 1 < len && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                                i += 1;
                            }
                            if i + 1 < len {
                                i += 2;
                            }
                            continue;
                        }
                        _ => {
                            break;
                        }
                    }
                }

                if !saw_newline || i >= len {
                    return false;
                }

                let is_ident_start = |b: u8| b == b'_' || (b as char).is_ascii_alphabetic();
                let is_ident_continue = |b: u8| b == b'_' || (b as char).is_ascii_alphanumeric();

                if !is_ident_start(bytes[i]) {
                    return false;
                }
                i += 1;
                while i < len && is_ident_continue(bytes[i]) {
                    i += 1;
                }
                while i < len && matches!(bytes[i], b' ' | b'\t') {
                    i += 1;
                }

                (i + 2 <= len && &bytes[i..i + 2] == b":=")
                    || (i + 3 <= len && &bytes[i..i + 3] == b"::=")
                    || (i + 2 <= len && &bytes[i..i + 2] == b":-")
                    || (i + 1 <= len && bytes[i] == b'=')
            }
            fn match_string(&mut self, expected: &str) -> ParseResult<&'input str> {
                self.consume_layout_for_terminal(expected);
                let start = self.position;
                let expected_bytes = expected.as_bytes();
                let end = start + expected_bytes.len();

                if self.logger.is_enabled() {
                    self.logger.log_debug(#filename, 0, &format!("🔤 Attempting to match terminal '{}' at position {} (end: {})", expected, start, end));
                }

                if self.bytes_match_at(start, expected_bytes) {
                    if !self.input.is_char_boundary(start) || !self.input.is_char_boundary(end) {
                        return Err(self.create_contextual_error(&format!(
                            "Internal UTF-8 boundary mismatch while matching '{}'",
                            expected
                        )));
                    }
                    self.position = end;

                    if self.logger.is_enabled() {
                        self.logger.log_success(#filename, 0, &format!("✅ Terminal '{}' matched, advanced to position {}", expected, end));
                    }

                    return Ok(&self.input[start..end]);
                }

                // Enhanced error with context
                let found_str = if self.position < self.input.len() {
                    let end = (self.position + expected_bytes.len()).min(self.input.len());
                    self.byte_window_lossy(self.position, end)
                } else {
                    "<EOF>".to_string()
                };

                if self.logger.is_enabled() {
                    self.logger.log_error(#filename, 0, &format!("❌ Terminal '{}' failed at position {} - found '{}'", expected, start, found_str));
                }

                Err(self.create_contextual_error(&format!(
                    "Expected '{}' but found '{}'",
                    expected, found_str
                )))
            }

            fn match_regex(&mut self, pattern: &str, skip_leading_whitespace: bool) -> ParseResult<&'input str> {
                let re = regex::Regex::new(pattern)
                    .map_err(|e| self.create_contextual_error(&format!(
                        "Invalid regex pattern '{}': {}",
                        pattern, e
                    )))?;

                if skip_leading_whitespace {
                    // Regexes that can match empty should not consume newlines first.
                    // Otherwise optional branches can silently jump into the next rule/line.
                    let can_match_empty = re
                        .find("")
                        .map(|m| m.start() == 0 && m.end() == 0)
                        .unwrap_or(false);
                    self.consume_layout_for_regex(can_match_empty);
                }

                let Some(haystack) = self.input.get(self.position..) else {
                    return Err(self.create_contextual_error("Parser position is not on a UTF-8 boundary"));
                };

                if let Some(mat) = re.find(haystack) {
                    if mat.start() == 0 {
                        let matched = mat.as_str();
                        let start = self.position;

                        if self.logger.is_enabled() {
                            self.logger.log_success(#filename, 0, &format!("✅ Regex '{}' matched '{}' at position {}", pattern, matched, start));
                        }

                        self.position += matched.len();
                        if let Some(slice) = self.input.get(start..self.position) {
                            return Ok(slice);
                        }
                        return Err(self.create_contextual_error("Regex matched invalid UTF-8 span"));
                    }
                }

                if self.logger.is_enabled() {
                    let preview = if self.position < self.input.len() {
                        let end = (self.position + 10).min(self.input.len());
                        self.byte_window_lossy(self.position, end)
                    } else {
                        "<EOF>".to_string()
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
                    if let Some(node) = cached {
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
                let input_context = self.byte_window_lossy(start, end);

                ParseError::ContextualError {
                    message: message.to_string(),
                    position,
                    rule_stack,
                    input_context,
                }
            }
        }
    }

    fn semantic_directive_name(annotation: &SemanticAnnotation) -> Option<String> {
        Self::semantic_directive_parts(annotation).map(|(name, _)| name)
    }

    fn rule_has_semantic_bool_directive(&self, rule_name: &str, names: &[&str]) -> bool {
        let Some(annotations) = &self.annotations else {
            return false;
        };
        let Some(semantic_annotations) = annotations.semantic_annotations.get(rule_name) else {
            return false;
        };

        semantic_annotations.iter().any(|annotation| {
            let Some((name, payload)) = Self::semantic_directive_parts(annotation) else {
                return false;
            };
            let name_matches = names.iter().any(|candidate| name.eq_ignore_ascii_case(candidate));
            if !name_matches {
                return false;
            }

            // Presence implies true; explicit falsy payload disables the gate.
            let normalized = payload
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_ascii_lowercase();
            !matches!(normalized.as_str(), "false" | "0" | "no" | "off")
        })
    }

    fn semantic_directive_parts(annotation: &SemanticAnnotation) -> Option<(String, String)> {
        if let Some(name) = annotation.name() {
            let normalized = name.trim().to_ascii_lowercase();
            if !normalized.is_empty() {
                let payload = match annotation.ast() {
                    UnifiedSemanticAST::TransformExpr { expression } => expression.clone(),
                    UnifiedSemanticAST::Raw { content } => content.clone(),
                };
                return Some((normalized, payload.trim().to_string()));
            }
        }

        match annotation.ast() {
            UnifiedSemanticAST::TransformExpr { expression } => {
                if let Some(parts) = extract_semantic_directive(expression) {
                    return Some(parts);
                }
                Some(("transform".to_string(), expression.clone()))
            }
            UnifiedSemanticAST::Raw { content } => extract_semantic_directive(content),
        }
    }

    fn rule_branch_policy(&self, rule_name: &str) -> SemanticBranchPolicy {
        let Some(annotations) = &self.annotations else {
            return SemanticBranchPolicy::LongestMatch;
        };
        let Some(entries) = annotations.semantic_annotations.get(rule_name) else {
            return SemanticBranchPolicy::LongestMatch;
        };

        let mut policy = SemanticBranchPolicy::LongestMatch;
        for annotation in entries {
            let Some((name, payload)) = Self::semantic_directive_parts(annotation) else {
                continue;
            };
            if name == "branch_policy" {
                if let Some(parsed) = SemanticBranchPolicy::parse(&payload) {
                    policy = parsed;
                }
            }
        }

        policy
    }

    fn rule_recovery_hints(&self, rule_name: &str) -> (bool, Vec<String>, Vec<String>) {
        let Some(annotations) = &self.annotations else {
            return (false, Vec::new(), Vec::new());
        };
        let Some(entries) = annotations.semantic_annotations.get(rule_name) else {
            return (false, Vec::new(), Vec::new());
        };

        let mut recover_enabled = false;
        let mut sync_tokens = Vec::new();
        let mut panic_until_tokens = Vec::new();
        for annotation in entries {
            let Some((name, payload)) = Self::semantic_directive_parts(annotation) else {
                continue;
            };
            match name.as_str() {
                "recover" => {
                    if let Some(parsed) = parse_semantic_bool(&payload) {
                        recover_enabled = parsed;
                    }
                }
                "sync" => {
                    if let Some(parsed) = parse_semantic_string_list(&payload) {
                        sync_tokens = parsed;
                    }
                }
                "panic_until" => {
                    if let Some(parsed) = parse_semantic_string_list(&payload) {
                        panic_until_tokens = parsed;
                    }
                }
                _ => {}
            }
        }
        (recover_enabled, sync_tokens, panic_until_tokens)
    }

    fn rule_associativity(&self, rule_name: &str) -> SemanticAssociativity {
        let Some(annotations) = &self.annotations else {
            return SemanticAssociativity::Left;
        };
        let Some(entries) = annotations.semantic_annotations.get(rule_name) else {
            return SemanticAssociativity::Left;
        };

        let mut associativity = SemanticAssociativity::Left;
        for annotation in entries {
            let Some((name, payload)) = Self::semantic_directive_parts(annotation) else {
                continue;
            };
            if name == "associativity" {
                if let Some(parsed) = SemanticAssociativity::parse(&payload) {
                    associativity = parsed;
                }
            }
        }

        associativity
    }

    fn rule_branch_priorities(&self, rule_name: &str, branch_count: usize) -> Vec<i64> {
        let default_priorities = vec![0i64; branch_count];
        let Some(annotations) = &self.annotations else {
            return default_priorities;
        };
        let Some(entries) = annotations.semantic_annotations.get(rule_name) else {
            return default_priorities;
        };

        let mut precedence_priorities: Option<Vec<i64>> = None;
        let mut explicit_priorities: Option<Vec<i64>> = None;

        for annotation in entries {
            let Some((name, payload)) = Self::semantic_directive_parts(annotation) else {
                continue;
            };
            let Some(parsed) = parse_semantic_branch_priorities(&payload, branch_count) else {
                continue;
            };
            match name.as_str() {
                "precedence" => {
                    precedence_priorities = Some(parsed);
                }
                "priority" => {
                    explicit_priorities = Some(parsed);
                }
                _ => {}
            }
        }

        explicit_priorities
            .or(precedence_priorities)
            .unwrap_or(default_priorities)
    }

    fn rule_value_constraints(&self, rule_name: &str) -> SemanticValueConstraints {
        let mut constraints = SemanticValueConstraints::default();
        let Some(annotations) = &self.annotations else {
            return constraints;
        };
        let Some(entries) = annotations.semantic_annotations.get(rule_name) else {
            return constraints;
        };

        for annotation in entries {
            let Some((name, payload)) = Self::semantic_directive_parts(annotation) else {
                continue;
            };

            match name.as_str() {
                "enum" => {
                    if let Some(values) = parse_semantic_string_list(&payload) {
                        constraints.enum_values = values;
                    }
                }
                "regex" => {
                    let pattern = normalize_semantic_scalar(&payload);
                    if !pattern.is_empty() {
                        constraints.regex_pattern = Some(pattern);
                    }
                }
                "range" => {
                    if let Some((min, max)) = parse_semantic_numeric_bounds(&payload) {
                        constraints.min_numeric = Some(min);
                        constraints.max_numeric = Some(max);
                    }
                }
                "len" => {
                    if let Some((min_len, max_len)) = parse_semantic_len_bounds(&payload) {
                        constraints.min_len = Some(min_len);
                        constraints.max_len = Some(max_len);
                    }
                }
                _ => {}
            }
        }

        constraints
    }

    fn semantic_value_constraint_tokens(
        &self,
        rule_name: &str,
        constraints: &SemanticValueConstraints,
    ) -> TokenStream {
        if constraints.is_empty() {
            return quote! {};
        }

        let mut checks = Vec::new();

        if !constraints.enum_values.is_empty() {
            let enum_values = constraints.enum_values.clone();
            checks.push(quote! {
                if ![#(#enum_values),*].iter().any(|allowed| *allowed == matched_str) {
                    return Err(parser.create_contextual_error(&format!(
                        "Semantic enum constraint failed for rule '{}': value '{}' not in allowed set",
                        #rule_name,
                        matched_str
                    )));
                }
            });
        }

        if let Some(pattern) = &constraints.regex_pattern {
            let pattern = pattern.clone();
            checks.push(quote! {
                let semantic_re = regex::Regex::new(#pattern).map_err(|e| {
                    parser.create_contextual_error(&format!(
                        "Invalid semantic regex constraint '{}' for rule '{}': {}",
                        #pattern,
                        #rule_name,
                        e
                    ))
                })?;
                let semantic_regex_full_match = semantic_re
                    .find(matched_str)
                    .map(|m| m.start() == 0 && m.end() == matched_str.len())
                    .unwrap_or(false);
                if !semantic_regex_full_match {
                    return Err(parser.create_contextual_error(&format!(
                        "Semantic regex constraint '{}' failed for rule '{}': value '{}'",
                        #pattern,
                        #rule_name,
                        matched_str
                    )));
                }
            });
        }

        match (constraints.min_len, constraints.max_len) {
            (Some(min_len), Some(max_len)) => checks.push(quote! {
                let semantic_len = matched_str.chars().count();
                if semantic_len < #min_len || semantic_len > #max_len {
                    return Err(parser.create_contextual_error(&format!(
                        "Semantic len constraint [{}, {}] failed for rule '{}': value '{}' has length {}",
                        #min_len,
                        #max_len,
                        #rule_name,
                        matched_str,
                        semantic_len
                    )));
                }
            }),
            (Some(min_len), None) => checks.push(quote! {
                let semantic_len = matched_str.chars().count();
                if semantic_len < #min_len {
                    return Err(parser.create_contextual_error(&format!(
                        "Semantic len minimum {} failed for rule '{}': value '{}' has length {}",
                        #min_len,
                        #rule_name,
                        matched_str,
                        semantic_len
                    )));
                }
            }),
            (None, Some(max_len)) => checks.push(quote! {
                let semantic_len = matched_str.chars().count();
                if semantic_len > #max_len {
                    return Err(parser.create_contextual_error(&format!(
                        "Semantic len maximum {} failed for rule '{}': value '{}' has length {}",
                        #max_len,
                        #rule_name,
                        matched_str,
                        semantic_len
                    )));
                }
            }),
            (None, None) => {}
        }

        match (constraints.min_numeric, constraints.max_numeric) {
            (Some(min), Some(max)) => checks.push(quote! {
                let semantic_numeric = matched_str.parse::<f64>().map_err(|_| {
                    parser.create_contextual_error(&format!(
                        "Semantic numeric constraint failed for rule '{}': value '{}' is not numeric",
                        #rule_name,
                        matched_str
                    ))
                })?;
                if semantic_numeric < #min || semantic_numeric > #max {
                    return Err(parser.create_contextual_error(&format!(
                        "Semantic numeric range [{}, {}] failed for rule '{}': value {}",
                        #min,
                        #max,
                        #rule_name,
                        semantic_numeric
                    )));
                }
            }),
            (Some(min), None) => checks.push(quote! {
                let semantic_numeric = matched_str.parse::<f64>().map_err(|_| {
                    parser.create_contextual_error(&format!(
                        "Semantic numeric constraint failed for rule '{}': value '{}' is not numeric",
                        #rule_name,
                        matched_str
                    ))
                })?;
                if semantic_numeric < #min {
                    return Err(parser.create_contextual_error(&format!(
                        "Semantic numeric min {} failed for rule '{}': value {}",
                        #min,
                        #rule_name,
                        semantic_numeric
                    )));
                }
            }),
            (None, Some(max)) => checks.push(quote! {
                let semantic_numeric = matched_str.parse::<f64>().map_err(|_| {
                    parser.create_contextual_error(&format!(
                        "Semantic numeric constraint failed for rule '{}': value '{}' is not numeric",
                        #rule_name,
                        matched_str
                    ))
                })?;
                if semantic_numeric > #max {
                    return Err(parser.create_contextual_error(&format!(
                        "Semantic numeric max {} failed for rule '{}': value {}",
                        #max,
                        #rule_name,
                        semantic_numeric
                    )));
                }
            }),
            (None, None) => {}
        }

        quote! {
            #(#checks)*
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

#[cfg(test)]
mod semantic_usage_tests {
    use super::*;
    use crate::ast_pipeline::{
        ASTNode, ASTValue, Annotations, SemanticAnnotation, TokenValue, UnifiedSemanticAST,
    };
    use std::collections::HashMap;

    fn regex_atom(pattern: &str) -> ASTNode {
        ASTNode::Atom {
            value: ASTValue::Token(vec![
                TokenValue::String("regex".to_string()),
                TokenValue::String(pattern.to_string()),
            ]),
        }
    }

    fn token(token_type: &str, token_value: &str) -> ASTNode {
        ASTNode::Atom {
            value: ASTValue::Token(vec![
                TokenValue::String(token_type.to_string()),
                TokenValue::String(token_value.to_string()),
            ]),
        }
    }

    fn generator_with_semantic(
        rule_name: &str,
        semantic_asts: Vec<UnifiedSemanticAST>,
    ) -> AstBasedGenerator {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            rule_name.to_string(),
            semantic_asts
                .into_iter()
                .map(SemanticAnnotation::from)
                .collect(),
        );

        AstBasedGenerator {
            grammar_name: "usage_test".to_string(),
            entry_rule: None,
            logger: None,
            annotations: Some(annotations),
            branch_return_annotations: HashMap::new(),
            enable_debug: false,
        }
    }

    fn or_rule() -> ASTNode {
        ASTNode::Or {
            alternatives: vec![token("quoted_string", "L"), token("quoted_string", "R")],
        }
    }

    #[test]
    fn semantic_usage_codegen_applies_canonical_transform_on_regex_atom() {
        let generator = generator_with_semantic(
            "number",
            vec![UnifiedSemanticAST::TransformExpr {
                expression: "str::parse::<i64>().unwrap_or(0)".to_string(),
            }],
        );

        let logic = generator
            .generate_node_parsing_logic(&regex_atom("[0-9]+"), "number", "semantic_usage.rs")
            .expect("regex atom logic generation should succeed");
        let rendered = logic.to_string();

        assert!(
            rendered.contains("ParseContent :: TransformedTerminal"),
            "expected transformed terminal output, got: {}",
            rendered
        );
        assert!(
            rendered.contains("parse :: < i64 >"),
            "expected canonical parse target in generated code, got: {}",
            rendered
        );
        assert!(
            rendered.contains("unwrap_or (0)"),
            "expected unwrap_or default in generated code, got: {}",
            rendered
        );
    }

    #[test]
    fn semantic_usage_codegen_accepts_path_target_type() {
        let generator = generator_with_semantic(
            "number",
            vec![UnifiedSemanticAST::TransformExpr {
                expression: "str::parse::<std::primitive::i64>().unwrap_or(0)".to_string(),
            }],
        );

        let logic = generator
            .generate_node_parsing_logic(&regex_atom("[0-9]+"), "number", "semantic_usage.rs")
            .expect("regex atom logic generation should succeed");
        let rendered = logic.to_string();

        assert!(
            rendered.contains("parse :: < std :: primitive :: i64 >"),
            "expected path target type in generated code, got: {}",
            rendered
        );
        assert!(
            rendered.contains("ParseContent :: TransformedTerminal"),
            "path target type should still produce transformed terminal output, got: {}",
            rendered
        );
    }

    #[test]
    fn semantic_usage_codegen_ignores_raw_annotations_for_regex_atom() {
        let generator = generator_with_semantic(
            "number",
            vec![UnifiedSemanticAST::Raw {
                content: "\"Number\"".to_string(),
            }],
        );

        let logic = generator
            .generate_node_parsing_logic(&regex_atom("[0-9]+"), "number", "semantic_usage.rs")
            .expect("regex atom logic generation should succeed");
        let rendered = logic.to_string();

        assert!(
            rendered.contains("ParseContent :: Terminal"),
            "raw semantic annotations should keep default terminal behavior, got: {}",
            rendered
        );
        assert!(
            !rendered.contains("ParseContent :: TransformedTerminal"),
            "raw semantic annotations should not force transformed terminal output, got: {}",
            rendered
        );
    }

    #[test]
    fn semantic_usage_codegen_ignores_transformexpr_when_named_non_transform_directive() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "number".to_string(),
            vec![SemanticAnnotation::Named {
                name: "type".to_string(),
                ast: UnifiedSemanticAST::TransformExpr {
                    expression: "str::parse::<i64>().unwrap_or(0)".to_string(),
                },
            }],
        );

        let generator = AstBasedGenerator {
            grammar_name: "usage_test".to_string(),
            entry_rule: None,
            logger: None,
            annotations: Some(annotations),
            branch_return_annotations: HashMap::new(),
            enable_debug: false,
        };

        let logic = generator
            .generate_node_parsing_logic(&regex_atom("[0-9]+"), "number", "semantic_usage.rs")
            .expect("regex atom logic generation should succeed");
        let rendered = logic.to_string();

        assert!(
            rendered.contains("ParseContent :: Terminal"),
            "non-transform directive should not trigger transform steering, got: {}",
            rendered
        );
        assert!(
            !rendered.contains("ParseContent :: TransformedTerminal"),
            "non-transform directive should not force transformed terminal output, got: {}",
            rendered
        );
    }

    #[test]
    fn semantic_usage_codegen_parses_associativity_directive() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "expr".to_string(),
            vec![SemanticAnnotation::Named {
                name: "associativity".to_string(),
                ast: UnifiedSemanticAST::Raw {
                    content: "right".to_string(),
                },
            }],
        );

        let generator = AstBasedGenerator {
            grammar_name: "usage_test".to_string(),
            entry_rule: None,
            logger: None,
            annotations: Some(annotations),
            branch_return_annotations: HashMap::new(),
            enable_debug: false,
        };

        assert_eq!(
            generator.rule_associativity("expr"),
            SemanticAssociativity::Right
        );
    }

    #[test]
    fn semantic_usage_codegen_parses_branch_priorities() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "expr".to_string(),
            vec![SemanticAnnotation::Named {
                name: "priority".to_string(),
                ast: UnifiedSemanticAST::Raw {
                    content: "[1, 9]".to_string(),
                },
            }],
        );

        let generator = AstBasedGenerator {
            grammar_name: "usage_test".to_string(),
            entry_rule: None,
            logger: None,
            annotations: Some(annotations),
            branch_return_annotations: HashMap::new(),
            enable_debug: false,
        };

        assert_eq!(generator.rule_branch_priorities("expr", 2), vec![1, 9]);
    }

    #[test]
    fn semantic_usage_codegen_parses_branch_policy_directive() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "expr".to_string(),
            vec![SemanticAnnotation::Named {
                name: "branch_policy".to_string(),
                ast: UnifiedSemanticAST::Raw {
                    content: "priority_first".to_string(),
                },
            }],
        );

        let generator = AstBasedGenerator {
            grammar_name: "usage_test".to_string(),
            entry_rule: None,
            logger: None,
            annotations: Some(annotations),
            branch_return_annotations: HashMap::new(),
            enable_debug: false,
        };

        assert_eq!(
            generator.rule_branch_policy("expr"),
            SemanticBranchPolicy::PriorityFirst
        );
    }

    #[test]
    fn semantic_usage_codegen_extracts_recovery_hints() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "stmt".to_string(),
            vec![
                SemanticAnnotation::Named {
                    name: "recover".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "true".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "sync".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "[\";\", \"end\"]".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "panic_until".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "[\"}\"]".to_string(),
                    },
                },
            ],
        );

        let generator = AstBasedGenerator {
            grammar_name: "usage_test".to_string(),
            entry_rule: None,
            logger: None,
            annotations: Some(annotations),
            branch_return_annotations: HashMap::new(),
            enable_debug: false,
        };

        let (recover_enabled, sync_tokens, panic_tokens) = generator.rule_recovery_hints("stmt");
        assert!(recover_enabled);
        assert_eq!(sync_tokens, vec![";".to_string(), "end".to_string()]);
        assert_eq!(panic_tokens, vec!["}".to_string()]);
    }

    #[test]
    fn semantic_usage_codegen_emits_runtime_recovery_hook_when_recover_enabled() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "stmt".to_string(),
            vec![
                SemanticAnnotation::Named {
                    name: "recover".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "true".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "sync".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "[\";\", \"end\"]".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "panic_until".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "[\"}\"]".to_string(),
                    },
                },
            ],
        );

        let generator = AstBasedGenerator {
            grammar_name: "usage_test".to_string(),
            entry_rule: None,
            logger: None,
            annotations: Some(annotations),
            branch_return_annotations: HashMap::new(),
            enable_debug: false,
        };

        let logic = generator
            .generate_node_parsing_logic(&or_rule(), "stmt", "semantic_usage.rs")
            .expect("or-node logic generation should succeed");
        let rendered = logic.to_string();

        assert!(
            rendered.contains("recover_with_hints"),
            "recover-enabled rule should emit runtime recovery hook, got: {}",
            rendered
        );
        assert!(
            rendered.contains("\";\"") && rendered.contains("\"}\""),
            "recovery hook should carry sync/panic tokens, got: {}",
            rendered
        );
    }

    #[test]
    fn semantic_usage_codegen_skips_runtime_recovery_hook_when_recover_not_enabled() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "stmt".to_string(),
            vec![SemanticAnnotation::Named {
                name: "sync".to_string(),
                ast: UnifiedSemanticAST::Raw {
                    content: "[\";\"]".to_string(),
                },
            }],
        );

        let generator = AstBasedGenerator {
            grammar_name: "usage_test".to_string(),
            entry_rule: None,
            logger: None,
            annotations: Some(annotations),
            branch_return_annotations: HashMap::new(),
            enable_debug: false,
        };

        let logic = generator
            .generate_node_parsing_logic(&or_rule(), "stmt", "semantic_usage.rs")
            .expect("or-node logic generation should succeed");
        let rendered = logic.to_string();

        assert!(
            !rendered.contains("recover_with_hints"),
            "recover-disabled rule should not emit runtime recovery hook, got: {}",
            rendered
        );
    }

    #[test]
    fn semantic_usage_codegen_priority_overrides_precedence_regardless_of_order() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "expr".to_string(),
            vec![
                SemanticAnnotation::Named {
                    name: "priority".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "[1, 9]".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "precedence".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "[9, 1]".to_string(),
                    },
                },
            ],
        );

        let generator = AstBasedGenerator {
            grammar_name: "usage_test".to_string(),
            entry_rule: None,
            logger: None,
            annotations: Some(annotations),
            branch_return_annotations: HashMap::new(),
            enable_debug: false,
        };

        assert_eq!(
            generator.rule_branch_priorities("expr", 2),
            vec![1, 9],
            "priority payload should override precedence payload independent of annotation order"
        );
    }

    #[test]
    fn semantic_usage_codegen_last_associativity_directive_wins() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "expr".to_string(),
            vec![
                SemanticAnnotation::Named {
                    name: "associativity".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "left".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "associativity".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "right".to_string(),
                    },
                },
            ],
        );

        let generator = AstBasedGenerator {
            grammar_name: "usage_test".to_string(),
            entry_rule: None,
            logger: None,
            annotations: Some(annotations),
            branch_return_annotations: HashMap::new(),
            enable_debug: false,
        };

        assert_eq!(
            generator.rule_associativity("expr"),
            SemanticAssociativity::Right,
            "duplicate associativity directives should resolve with last occurrence wins"
        );
    }

    #[test]
    fn semantic_usage_codegen_emits_priority_and_associativity_tiebreak_logic() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "expr".to_string(),
            vec![
                SemanticAnnotation::Named {
                    name: "priority".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "[1, 9]".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "associativity".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "right".to_string(),
                    },
                },
            ],
        );

        let generator = AstBasedGenerator {
            grammar_name: "usage_test".to_string(),
            entry_rule: None,
            logger: None,
            annotations: Some(annotations),
            branch_return_annotations: HashMap::new(),
            enable_debug: false,
        };

        let logic = generator
            .generate_node_parsing_logic(&or_rule(), "expr", "semantic_usage.rs")
            .expect("or-node logic generation should succeed");
        let rendered = logic.to_string();

        assert!(
            rendered.contains("candidate_priority"),
            "expected candidate priority tie-break support, got: {}",
            rendered
        );
        assert!(
            rendered.contains("match \"right\""),
            "expected associativity-aware logging content, got: {}",
            rendered
        );
    }

    #[test]
    fn semantic_usage_codegen_emits_value_constraint_guards_for_regex_atoms() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "ident".to_string(),
            vec![
                SemanticAnnotation::Named {
                    name: "enum".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "[\"AA\", \"BB\"]".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "len".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "[2, 2]".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "regex".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "^[A-Z]{2}$".to_string(),
                    },
                },
            ],
        );

        let generator = AstBasedGenerator {
            grammar_name: "usage_test".to_string(),
            entry_rule: None,
            logger: None,
            annotations: Some(annotations),
            branch_return_annotations: HashMap::new(),
            enable_debug: false,
        };

        let logic = generator
            .generate_node_parsing_logic(&regex_atom("[A-Z]+"), "ident", "semantic_usage.rs")
            .expect("regex atom logic generation should succeed");
        let rendered = logic.to_string();

        assert!(
            rendered.contains("Semantic enum constraint failed"),
            "expected enum constraint guard in generated code, got: {}",
            rendered
        );
        assert!(
            rendered.contains("Semantic len constraint"),
            "expected len constraint guard in generated code, got: {}",
            rendered
        );
        assert!(
            rendered.contains("Semantic regex constraint"),
            "expected regex constraint guard in generated code, got: {}",
            rendered
        );
    }

    #[test]
    fn semantic_usage_codegen_emits_numeric_range_constraint_guards() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "number".to_string(),
            vec![SemanticAnnotation::Named {
                name: "range".to_string(),
                ast: UnifiedSemanticAST::Raw {
                    content: "10..20".to_string(),
                },
            }],
        );

        let generator = AstBasedGenerator {
            grammar_name: "usage_test".to_string(),
            entry_rule: None,
            logger: None,
            annotations: Some(annotations),
            branch_return_annotations: HashMap::new(),
            enable_debug: false,
        };

        let logic = generator
            .generate_node_parsing_logic(&regex_atom("[0-9]+"), "number", "semantic_usage.rs")
            .expect("regex atom logic generation should succeed");
        let rendered = logic.to_string();

        assert!(
            rendered.contains("Semantic numeric range"),
            "expected numeric range guard in generated code, got: {}",
            rendered
        );
    }

    #[test]
    fn unresolved_reference_codegen_emits_semantic_and_boolean_fallbacks() {
        let generator = AstBasedGenerator::new("usage_test".to_string());

        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Sequence {
                elements: vec![
                    token("rule_reference", "semantic_annotation"),
                    token("rule_reference", "true"),
                ],
            },
        );
        let rule_order = vec!["start".to_string()];

        let methods = generator.generate_unresolved_reference_methods(&grammar_tree, &rule_order);
        let rendered = methods
            .into_iter()
            .map(|m| m.to_string())
            .collect::<Vec<_>>()
            .join("\n");

        assert!(
            rendered.contains("pub fn parse_semantic_annotation"),
            "expected semantic_annotation fallback method in unresolved reference emission"
        );
        assert!(
            rendered.contains("starts_with") || rendered.contains("b'@'"),
            "expected semantic_annotation fallback to detect '@' directives"
        );
        assert!(
            rendered.contains("pub fn parse_true"),
            "expected boolean fallback method for malformed rule_reference true"
        );
        assert!(
            rendered.contains("\"true\""),
            "expected parse_true fallback to materialize boolean content, got: {}",
            rendered
        );
    }

    #[test]
    fn unresolved_reference_codegen_skips_known_rules() {
        let generator = AstBasedGenerator::new("usage_test".to_string());

        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Sequence {
                elements: vec![token("rule_reference", "known")],
            },
        );
        grammar_tree.insert(
            "known".to_string(),
            ASTNode::Atom {
                value: ASTValue::Token(vec![
                    TokenValue::String("quoted_string".to_string()),
                    TokenValue::String("k".to_string()),
                ]),
            },
        );
        let rule_order = vec!["start".to_string(), "known".to_string()];

        let methods = generator.generate_unresolved_reference_methods(&grammar_tree, &rule_order);
        assert!(
            methods.is_empty(),
            "known in-grammar rule references should not emit fallback methods"
        );
    }
}
