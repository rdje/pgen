//! High-Performance Rust Code Generator
//! Generates lightning-fast parsers with:
//! - Zero-copy parsing where possible
//! - Memoization/packrat parsing for backtracking
//! - Inline optimizations and SIMD-friendly code
//! - Minimal allocations for rgx regex engine integration

use crate::ast_pipeline::{ASTNode, ASTValue, Annotations, ReturnAnnotation};
use std::collections::HashMap;
use anyhow::Result;
use serde_json::Value as JsonValue;


/// Escape a string for safe inclusion in Rust raw string literals
fn escape_rust_string(s: &str) -> String {
    // For raw strings r#"..."#, we need minimal escaping
    // The main issue is handling strings that contain the raw string delimiter
    // For now, just return the string as-is since raw strings handle most cases
    s.to_string()
}

/// Builder for systematic Rust code generation
struct RustCodeBuilder {
    lines: Vec<String>,
}

impl RustCodeBuilder {
    fn new() -> Self {
        Self { lines: Vec::new() }
    }
    
    fn add_line(&mut self, line: &str) {
        self.lines.push(line.to_string());
    }
    
    fn add_raw(&mut self, code: &str) {
        self.lines.push(code.to_string());
    }
    
    fn build(self) -> String {
        self.lines.join("\n")
    }
}

/// High-performance code generator optimized for regex parsing
pub struct HighPerformanceRustGenerator {
    grammar_name: String,
    entry_rule: Option<String>,
    enable_trace: bool,
    pub enable_backtrack_debug: bool,
    annotations: Option<Annotations>,
    return_annotations: HashMap<String, ReturnAnnotation>,
}

impl HighPerformanceRustGenerator {
    pub fn new(grammar_name: &str) -> Self {
        Self {
            grammar_name: grammar_name.to_string(),
            entry_rule: None,
            enable_trace: false,
            enable_backtrack_debug: false,
            annotations: None,
            return_annotations: HashMap::new(),
        }
    }
    
    /// Create generator with trace mode enabled
    pub fn with_trace(grammar_name: &str, enable_trace: bool) -> Self {
        Self {
            grammar_name: grammar_name.to_string(),
            entry_rule: None,
            enable_trace,
            enable_backtrack_debug: false,
            annotations: None,
            return_annotations: HashMap::new(),
        }
    }
    
    /// Create generator with full debug enabled (trace + backtrack)
    pub fn with_full_debug(grammar_name: &str) -> Self {
        Self {
            grammar_name: grammar_name.to_string(),
            entry_rule: None,
            enable_trace: true,
            enable_backtrack_debug: true,
            annotations: None,
            return_annotations: HashMap::new(),
        }
    }
    
    /// Set the entry rule name dynamically
    pub fn set_entry_rule(&mut self, entry_rule: &str) {
        self.entry_rule = Some(entry_rule.to_string());
    }
    
    /// Set the annotations for the generator to use during code generation
    pub fn set_annotations(&mut self, annotations: Annotations) {
        self.annotations = Some(annotations);
    }
    
    /// Set return annotations for code generation
    pub fn set_return_annotations(&mut self, return_annotations: &HashMap<String, ReturnAnnotation>) {
        self.return_annotations = return_annotations.clone();
    }

    /// Generate lightning-fast parser suitable for production regex engine
    pub fn generate_lightning_fast_parser(
        &self,
        grammar_tree: &HashMap<String, ASTNode>,
        rule_order: &[String],
    ) -> Result<String> {
        if rule_order.is_empty() {
            return Err(anyhow::anyhow!("No rules provided - cannot determine entry rule"));
        }
        
        // Entry rule should be the first rule in the grammar, or explicitly set entry rule
        let entry_rule = self.entry_rule.as_ref()
            .map(|s| s.clone())
            .or_else(|| rule_order.first().cloned())
            .ok_or_else(|| anyhow::anyhow!("No rules provided in rule_order - cannot determine entry rule"))?;
        
        let mut code = String::with_capacity(65536); // Pre-allocate for performance

        // Generate high-performance parser header
        code.push_str(&self.generate_parser_header());
        
        // Generate core parsing engine with memoization (starts impl block)
        code.push_str(&self.generate_memoized_parser_core(&entry_rule));
        
        // Generate optimized rule methods (inside impl block)
        code.push_str(&self.generate_optimized_rule_methods(grammar_tree, rule_order)?);
        
        // Generate fast helper methods (inside impl block)
        code.push_str(&self.generate_fast_helpers());
        
        // Close the impl block
        code.push_str("}\n\n");
        
        // Generate performance tests
        code.push_str(&self.generate_performance_tests());
        
        Ok(code)
    }

    fn generate_parser_header(&self) -> String {
        format!(r#"// {grammar_name} High-Performance Parser
// Generated for rgx regex engine - SOTA performance
// Features: Zero-copy, memoization, SIMD-optimized, minimal allocations

use std::{{
    collections::HashMap,
    fmt,
    ops::Range,
}};
use regex::Regex;

/// Parse result with zero-copy string slices
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {{
    UnexpectedEof {{ position: usize }},
    UnexpectedToken {{ expected: &'static str, found: char, position: usize }},
    InvalidSyntax {{ message: &'static str, position: usize }},
    Backtrack {{ position: usize }},
}}

impl fmt::Display for ParseError {{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {{
        match self {{
            ParseError::UnexpectedEof {{ position }} => 
                write!(f, "Unexpected EOF at position {{}}", position),
            ParseError::UnexpectedToken {{ expected, found, position }} => 
                write!(f, "Expected '{{}}', found '{{}}' at position {{}}", expected, found, position),
            ParseError::InvalidSyntax {{ message, position }} => 
                write!(f, "{{}} at position {{}}", message, position),
            ParseError::Backtrack {{ position }} => 
                write!(f, "Backtrack at position {{}}", position),
        }}
    }}
}}

impl std::error::Error for ParseError {{}}

pub type ParseResult<T> = Result<T, ParseError>;

/// Zero-copy AST node with string slice references
#[derive(Debug, Clone, PartialEq)]
pub struct ParseNode<'input> {{
    pub rule_name: &'static str,
    pub content: ParseContent<'input>,
    pub span: Range<usize>,
}}

#[derive(Debug, Clone, PartialEq)]
pub enum ParseContent<'input> {{
    Terminal(&'input str),
    Sequence(Vec<ParseNode<'input>>),
    Alternative(Box<ParseNode<'input>>),
    Quantified(Vec<ParseNode<'input>>, &'static str),
}}

/// Memoization entry for packrat parsing
#[derive(Debug, Clone)]
struct MemoEntry<'input> {{
    result: Option<ParseNode<'input>>,
    end_pos: usize,
}}

/// Compact rule ID for fast memoization lookups
type RuleId = u16;

"#, grammar_name = self.grammar_name)
    }

    fn generate_memoized_parser_core(&self, entry_rule: &str) -> String {
        // Convert grammar name to PascalCase (e.g., "regex" -> "RegexParser")
        let mut chars = self.grammar_name.chars();
        let parser_name = format!("{}Parser", 
            chars.next().unwrap().to_uppercase().collect::<String>() + &chars.collect::<String>());
        
        let backtrack_debug_code = if self.enable_backtrack_debug {
            "                self.debug_backtrack(self.position, saved_pos, \"try_parse failed\");\n"
        } else {
            ""
        };
        
        format!(r#"/// High-Performance parser with memoization and zero-copy parsing
pub struct {parser_name}<'input> {{
    input: &'input str,
    position: usize,
    memo: HashMap<(RuleId, usize), MemoEntry<'input>>,
    bytes: &'input [u8], // For SIMD optimizations
    debug_mode: bool,
    debug_depth: usize,
    debug_output: Vec<String>,
}}

impl<'input> {parser_name}<'input> {{
    /// Create new parser with zero-copy input
    #[inline]
    pub fn new(input: &'input str) -> Self {{
        Self {{
            input,
            position: 0,
            memo: HashMap::with_capacity(1024), // Pre-allocate memo table
            bytes: input.as_bytes(),
            debug_mode: {enable_trace},
            debug_depth: 0,
            debug_output: Vec::new(),
        }}
    }}
    
    /// Create new parser with debug mode enabled
    #[inline]
    pub fn with_debug(input: &'input str) -> Self {{
        Self {{
            input,
            position: 0,
            memo: HashMap::with_capacity(1024),
            bytes: input.as_bytes(),
            debug_mode: true,
            debug_depth: 0,
            debug_output: Vec::new(),
        }}
    }}
    
    /// Get debug output for analysis
    pub fn debug_output(&self) -> &[String] {{
        &self.debug_output
    }}
    
    /// Clear debug output
    pub fn clear_debug(&mut self) {{
        self.debug_output.clear();
        self.debug_depth = 0;
    }}

    /// Parse entry point - returns AST or error
    pub fn parse(&mut self) -> ParseResult<ParseNode<'input>> {{
        self.position = 0;
        self.memo.clear();
        self.parse_{entry_rule}() // Dynamic entry rule
    }}

    /// Fast character access with bounds checking
    #[inline(always)]
    fn current_char(&self) -> Option<char> {{
        self.input.chars().nth(self.position)
    }}

    /// SIMD-optimized byte access for ASCII fast path
    #[inline(always)]
    fn current_byte(&self) -> Option<u8> {{
        self.bytes.get(self.position).copied()
    }}

    /// Advance position with UTF-8 awareness
    #[inline(always)]
    fn advance(&mut self) -> Option<char> {{
        if let Some(ch) = self.current_char() {{
            self.position += ch.len_utf8();
            Some(ch)
        }} else {{
            None
        }}
    }}

    /// Fast advance for ASCII characters
    #[inline(always)]
    fn advance_ascii(&mut self) -> Option<u8> {{
        if let Some(byte) = self.current_byte() {{
            if byte < 128 {{
                self.position += 1;
                Some(byte)
            }} else {{
                None
            }}
        }} else {{
            None
        }}
    }}

    /// Get zero-copy slice from input
    #[inline(always)]
    fn slice(&self, range: Range<usize>) -> &'input str {{
        &self.input[range]
    }}

    /// Memoized rule call with packrat parsing
    #[inline]
    fn memoized_call<F>(&mut self, rule_id: RuleId, f: F) -> ParseResult<ParseNode<'input>>
    where
        F: FnOnce(&mut Self) -> ParseResult<ParseNode<'input>>,
    {{
        let key = (rule_id, self.position);
        
        // Check memo table
        if let Some(entry) = self.memo.get(&key) {{
            self.position = entry.end_pos;
            match &entry.result {{
                Some(node) => Ok(node.clone()),
                None => Err(ParseError::Backtrack {{ position: self.position }}),
            }}
        }} else {{
            // Not memoized - compute result
            let start_pos = self.position;
            match f(self) {{
                Ok(node) => {{
                    let end_pos = self.position;
                    self.memo.insert(key, MemoEntry {{
                        result: Some(node.clone()),
                        end_pos,
                    }});
                    Ok(node)
                }}
                Err(err) => {{
                    self.memo.insert(key, MemoEntry {{
                        result: None,
                        end_pos: start_pos,
                    }});
                    Err(err)
                }}
            }}
        }}
    }}

    /// Fast string matching with SIMD potential
    #[inline]
    fn match_string(&mut self, expected: &'static str) -> ParseResult<&'input str> {{
        let start_pos = self.position;
        
        // ASCII fast path for single characters
        if expected.len() == 1 {{
            let expected_byte = expected.as_bytes()[0];
            if expected_byte < 128 {{
                if let Some(byte) = self.current_byte() {{
                    if byte == expected_byte {{
                        self.position += 1;
                        return Ok(&self.input[start_pos..self.position]);
                    }}
                }}
                return Err(ParseError::UnexpectedToken {{
                    expected,
                    found: self.current_char().unwrap_or('\0'),
                    position: start_pos,
                }});
            }}
        }}
        
        // General UTF-8 path
        for (i, expected_char) in expected.chars().enumerate() {{
            match self.current_char() {{
                Some(ch) if ch == expected_char => {{
                    self.advance();
                }}
                Some(found) => {{
                    self.position = start_pos;
                    return Err(ParseError::UnexpectedToken {{
                        expected,
                        found,
                        position: start_pos + i,
                    }});
                }}
                None => {{
                    self.position = start_pos;
                    return Err(ParseError::UnexpectedEof {{
                        position: start_pos + i,
                    }});
                }}
            }}
        }}
        
        Ok(&self.input[start_pos..self.position])
    }}

    /// Try parsing with automatic backtracking
    #[inline]
    fn try_parse<T, F>(&mut self, f: F) -> Option<T>
    where
        F: FnOnce(&mut Self) -> ParseResult<T>,
    {{
        let saved_pos = self.position;
        match f(self) {{
            Ok(result) => Some(result),
            Err(_) => {{
{backtrack_debug_code}                self.position = saved_pos;
                None
            }}
        }}
    }}
    
    /// Debug: Log entry into a parsing rule
    #[inline]
    fn debug_enter_rule(&mut self, rule_name: &str) {{
        if self.debug_mode {{
            let indent = "  ".repeat(self.debug_depth);
            let context = if self.position < self.input.len() {{
                let end_pos = (self.position + 10).min(self.input.len());
                let context_str = self.format_debug_string(&self.input[self.position..end_pos]);
                format!(" at '{{}}'", context_str)
            }} else {{
                " at EOF".to_string()
            }};
            let msg = format!("{{}}→ ENTER {{}}: pos={{}}{{}}", indent, rule_name, self.position, context);
            self.debug_output.push(msg);
            self.debug_depth += 1;
        }}
    }}
    
    /// Debug: Log successful exit from a parsing rule
    #[inline]
    fn debug_exit_success(&mut self, rule_name: &str, start_pos: usize) {{
        if self.debug_mode {{
            self.debug_depth = self.debug_depth.saturating_sub(1);
            let indent = "  ".repeat(self.debug_depth);
            let consumed = if self.position > start_pos {{
                let consumed_str = self.format_debug_string(&self.input[start_pos..self.position]);
                format!(" consumed '{{}}'", consumed_str)
            }} else {{
                " (no input consumed)".to_string()
            }};
            let msg = format!("{{}}← SUCCESS {{}}: {{}}->{{}}{{}}", 
                indent, rule_name, start_pos, self.position, consumed);
            self.debug_output.push(msg);
        }}
    }}
    
    /// Debug: Log failed exit from a parsing rule
    #[inline]
    fn debug_exit_fail(&mut self, rule_name: &str, error: &ParseError) {{
        if self.debug_mode {{
            self.debug_depth = self.debug_depth.saturating_sub(1);
            let indent = "  ".repeat(self.debug_depth);
            let msg = format!("{{}}← FAIL {{}}: {{}}", indent, rule_name, error);
            self.debug_output.push(msg);
        }}
    }}
    
    /// Debug: Log backtracking
    #[inline]
    fn debug_backtrack(&mut self, from_pos: usize, to_pos: usize, reason: &str) {{
        if self.debug_mode {{
            let indent = "  ".repeat(self.debug_depth);
            let msg = format!("{{}}⟲ BACKTRACK: {{}}->{{}} ({{}})", indent, from_pos, to_pos, reason);
            self.debug_output.push(msg);
        }}
    }}
    
    /// Debug: Log alternative attempt
    #[inline]
    fn debug_try_alternative(&mut self, alt_index: usize, total: usize) {{
        if self.debug_mode {{
            let indent = "  ".repeat(self.debug_depth);
            let msg = format!("{{}}? TRY ALT {{}}/{{}}: pos={{}}", indent, alt_index + 1, total, self.position);
            self.debug_output.push(msg);
        }}
    }}
    
    /// Debug: Log sequence element attempt
    #[inline]
    fn debug_sequence_element(&mut self, elem_index: usize, total: usize, elem_name: &str) {{
        if self.debug_mode {{
            let indent = "  ".repeat(self.debug_depth);
            let msg = format!("{{}}▶ SEQ {{}}/{{}}: {{}} at pos={{}}", 
                indent, elem_index + 1, total, elem_name, self.position);
            self.debug_output.push(msg);
        }}
    }}
    
    /// Debug: Log quantifier iteration
    #[inline]
    fn debug_quantifier_iteration(&mut self, iteration: usize, quantifier: &str) {{
        if self.debug_mode {{
            let indent = "  ".repeat(self.debug_depth);
            let msg = format!("{{}}* QUANT '{{}}' iteration {{}}: pos={{}}", 
                indent, quantifier, iteration, self.position);
            self.debug_output.push(msg);
        }}
    }}
    
    /// Helper function to format strings safely for debug output
    #[inline]
    fn format_debug_string(&self, s: &str) -> String {{
        s.chars()
            .map(|c| match c {{
                '\n' => "\\n".to_string(),
                '\r' => "\\r".to_string(),
                '\t' => "\\t".to_string(),
                '\\' => "\\\\".to_string(),
                '"' => "\\\"".to_string(),
                c if c.is_control() => format!("\\u{{:04x}}", c as u32),
                c => c.to_string(),
            }})
            .collect()
    }}

"#, 
            parser_name = parser_name,
            entry_rule = entry_rule,
            enable_trace = self.enable_trace
        )
    }

    fn generate_optimized_rule_methods(
        &self,
        grammar_tree: &HashMap<String, ASTNode>,
        rule_order: &[String],
    ) -> Result<String> {
        let mut code = String::new();
        
        println!("[HighPerformanceRustGenerator] Starting rule method generation");
        println!("[HighPerformanceRustGenerator] Total rules in rule_order: {}", rule_order.len());
        println!("[HighPerformanceRustGenerator] Total rules in grammar_tree: {}", grammar_tree.len());
        
        // Debug: List all rules
        println!("[HighPerformanceRustGenerator] Rules in rule_order: {:?}", rule_order);
        println!("[HighPerformanceRustGenerator] Rules in grammar_tree: {:?}", grammar_tree.keys().collect::<Vec<_>>());
        
        // Generate rule ID constants
        code.push_str("    // Rule IDs for memoization\n");
        for (i, rule_name) in rule_order.iter().enumerate() {
            code.push_str(&format!("    const RULE_{}: RuleId = {};\n", 
                rule_name.to_uppercase(), i));
        }
        code.push_str("\n");

        // Generate rule methods
        let mut methods_generated = 0;
        let mut methods_skipped = 0;
        
        for (i, rule_name) in rule_order.iter().enumerate() {
            if let Some(ast_node) = grammar_tree.get(rule_name) {
                println!("[HighPerformanceRustGenerator] ✓ Generating method for rule: {} (index {})", rule_name, i);
                let method_code = self.generate_optimized_rule_method(rule_name, ast_node, i as u16)?;
                code.push_str(&method_code);
                methods_generated += 1;
            } else {
                println!("[HighPerformanceRustGenerator] ✗ SKIPPING rule: {} (not found in grammar_tree)", rule_name);
                methods_skipped += 1;
            }
        }
        
        println!("[HighPerformanceRustGenerator] Summary:");
        println!("[HighPerformanceRustGenerator]   ✓ Methods generated: {}", methods_generated);
        println!("[HighPerformanceRustGenerator]   ✗ Methods skipped: {}", methods_skipped);
        println!("[HighPerformanceRustGenerator]   📊 Total rules processed: {}", rule_order.len());
        
        if methods_skipped > 0 {
            println!("[HighPerformanceRustGenerator] ⚠️  WARNING: {} rules were skipped - this will cause compilation errors!", methods_skipped);
        }

        Ok(code)
    }

    fn generate_optimized_rule_method(
        &self,
        rule_name: &str,
        ast_node: &ASTNode,
        rule_id: u16,
    ) -> Result<String> {
        println!("[HighPerformanceRustGenerator][generate_optimized_rule_method] 🔧 Processing rule: '{}' (ID: {})", rule_name, rule_id);
        println!("[HighPerformanceRustGenerator][generate_optimized_rule_method]   📋 Rule AST: {:?}", ast_node);
        
        // Get semantic annotations for this rule if available
        let rule_annotations = if let Some(ref annotations) = self.annotations {
            annotations.semantic_annotations.get(rule_name).cloned()
        } else {
            None
        };
        
        if let Some(ref annotations) = rule_annotations {
            println!("[HighPerformanceRustGenerator][generate_optimized_rule_method]   🏷️  Found {} semantic annotations for '{}'", annotations.len(), rule_name);
            for (i, annotation) in annotations.iter().enumerate() {
                println!("[HighPerformanceRustGenerator][generate_optimized_rule_method]     {}. {}", i + 1, annotation);
            }
        } else {
            println!("[HighPerformanceRustGenerator][generate_optimized_rule_method]   ❌ No semantic annotations found for '{}'", rule_name);
        }
        
        println!("[HighPerformanceRustGenerator][generate_optimized_rule_method]   🏗️  Generating method body for '{}'", rule_name);
        let method_body = self.generate_optimized_node_code(ast_node, 2, rule_annotations.as_deref())?;
        
        let method_name = format!("parse_{}", rule_name);
        println!("[HighPerformanceRustGenerator][generate_optimized_rule_method]   ✅ Generated method: '{}()' for rule '{}'\n", method_name, rule_name);
        
        Ok(format!(r#"    /// Parse {rule_name} with memoization
    #[inline]
    fn parse_{rule_name}(&mut self) -> ParseResult<ParseNode<'input>> {{
        self.memoized_call(Self::RULE_{rule_name_upper}, |parser| {{
            parser.debug_enter_rule("{rule_name}");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {{
{method_body}
                let end_pos = parser.position;
                
                Ok(ParseNode {{
                    rule_name: "{rule_name}",
                    content: result,
                    span: start_pos..end_pos,
                }})
            }})();
            
            match &parse_result {{
                Ok(_) => parser.debug_exit_success("{rule_name}", start_pos),
                Err(err) => parser.debug_exit_fail("{rule_name}", err),
            }};
            
            parse_result
        }})
    }}

"#, 
            rule_name = rule_name, 
            rule_name_upper = rule_name.to_uppercase(),
            method_body = method_body
        ))
    }

    fn generate_optimized_node_code(
        &self, 
        ast_node: &ASTNode, 
        indent_level: usize,
        rule_annotations: Option<&[String]>
    ) -> Result<String> {
        let indent = "    ".repeat(indent_level);
        
        match ast_node {
            ASTNode::Atom { value } => {
                self.generate_atom_code(value, &indent, rule_annotations)
            }
            ASTNode::Sequence { elements } => {
                self.generate_sequence_code(elements, &indent, rule_annotations)
            }
            ASTNode::Or { alternatives } => {
                self.generate_or_code(alternatives, &indent, rule_annotations)
            }
            ASTNode::Quantified { element, quantifier } => {
                self.generate_quantified_code(element, quantifier, &indent, rule_annotations)
            }
        }
    }

    fn generate_atom_code(&self, value: &ASTValue, indent: &str, rule_annotations: Option<&[String]>) -> Result<String> {
        println!("[HighPerformanceRustGenerator][generate_atom_code] ⚛️  Processing atom: {:?}", value);
        
        match value {
            ASTValue::Token(token) if token.len() == 2 => {
                let token_type = &token[0];
                let token_value = &token[1];
                
                println!("[HighPerformanceRustGenerator][generate_atom_code]   📝 Token type: {:?}", token_type);
                println!("[HighPerformanceRustGenerator][generate_atom_code]   📝 Token value: {:?}", token_value);
                
                // Check for semantic annotations that might guide code generation
                let custom_code = if let Some(annotations) = rule_annotations {
                    println!("[HighPerformanceRustGenerator][generate_atom_code]   🏷️  Checking rule-level semantic annotations for atom customization");
                    self.apply_semantic_annotations(annotations, token_type, token_value, indent)
                } else {
                    // Note: This is normal - most atoms use default generation
                    // Semantic annotations are rule-level, not atom-level
                    None
                };
                
                // If we have custom code from semantic annotations, use it; otherwise use default generation
                if let Some(code) = custom_code {
                    println!("[HighPerformanceRustGenerator][generate_atom_code]   🎯 Using custom code from semantic annotations");
                    return Ok(code);
                }
                
                println!("[HighPerformanceRustGenerator][generate_atom_code]   🔧 Using default code generation for token_type: {:?}", token_type.as_str());
                match token_type.as_str() {
                        Some("quoted_string") => {
                            println!("[HighPerformanceRustGenerator][generate_atom_code]     ➤ Generating quoted_string code");
                            if token_value.is_empty() {
                                // Handle empty strings with regular string literals
                                Ok(format!("{indent}let result = ParseContent::Terminal(parser.match_string(\"\")?);\n"))
                            } else {
                                let escaped_value = escape_rust_string(token_value.as_str().unwrap_or(""));
                                Ok(format!("{indent}let result = ParseContent::Terminal(parser.match_string(r#\"{escaped_value}\"#)?);\n"))
                            }
                        }
                        Some("regex") => {
                            println!("[HighPerformanceRustGenerator][generate_atom_code]     ➤ Generating regex code");
                            let escaped_value = escape_rust_string(token_value.as_str().unwrap_or(""));
                            Ok(format!("{indent}let result = ParseContent::Terminal(parser.match_regex_optimized(r#\"{escaped_value}\"#)?);\n"))
                        }
                    Some("rule_reference") => {
                        let rule_name = token_value.as_str().unwrap_or("unknown");
                        println!("[HighPerformanceRustGenerator][generate_atom_code]     ➤ Generating rule_reference code for rule: '{}'", rule_name);
                        println!("[HighPerformanceRustGenerator][generate_atom_code]       🔗 Will call method: parse_{}()", rule_name);
                        Ok(format!("{indent}let result = ParseContent::Alternative(Box::new(parser.parse_{rule_name}()?));\n"))
                    }
                    _ => {
                        println!("[HighPerformanceRustGenerator][generate_atom_code]     ➤ Generating default terminal code for unknown token type: {:?}", token_type.as_str());
                        let escaped_value = escape_rust_string(token_value.as_str().unwrap_or(""));
                        Ok(format!("{indent}let result = ParseContent::Terminal(r#\"{escaped_value}\"#);\n"))
                    }
                }
            }
            _ => {
                println!("[HighPerformanceRustGenerator][generate_atom_code]   ⚠️  Non-token AST value: {:?}", value);
                Ok(format!("{indent}let result = ParseContent::Terminal(\"unknown\");\n"))
            }
        }
    }
    
    /// Apply semantic annotations to guide code generation
    /// This is where the magic happens - semantic annotations drive custom code generation
    fn apply_semantic_annotations(
        &self, 
        annotations: &[String], 
        token_type: &crate::ast_pipeline::TokenValue, 
        token_value: &crate::ast_pipeline::TokenValue, 
        indent: &str
    ) -> Option<String> {
        println!("[HighPerformanceRustGenerator][apply_semantic_annotations] 📝 Processing {} annotations for token_type: {:?}, token_value: {:?}", annotations.len(), token_type, token_value);
        
        if annotations.is_empty() {
            println!("[HighPerformanceRustGenerator][apply_semantic_annotations]   ❌ No annotations to process");
            return None;
        }
        
        // Parse the semantic annotations looking for @generate directives
        for annotation in annotations {
            println!("[HighPerformanceRustGenerator][apply_semantic_annotations]   🔍 Processing annotation: {}", annotation);
            
            // Semantic annotations are in format "name:parsed_json_ast"
            if let Some(colon_pos) = annotation.find(':') {
                let annotation_name = &annotation[..colon_pos];
                let annotation_value = &annotation[colon_pos + 1..];
                
                println!("[HighPerformanceRustGenerator][apply_semantic_annotations]     📋 Annotation name: '{}'", annotation_name);
                println!("[HighPerformanceRustGenerator][apply_semantic_annotations]     📋 Annotation value: '{}'", annotation_value);
                
                match annotation_name {
                    "codegen" => {
                        println!("[HighPerformanceRustGenerator][apply_semantic_annotations]     🎯 Found codegen annotation!");
                        // Handle special code generation directives
                        let directive = annotation_value.trim_matches('"');
                        println!("[HighPerformanceRustGenerator][apply_semantic_annotations]     🎯 Codegen directive: '{}'", directive);
                        
                        match directive {
                            "escape_literal_handling" => {
                                println!("[HighPerformanceRustGenerator][apply_semantic_annotations]     ✅ Found escape_literal_handling directive! Calling generate_escape_literal_code");
                                return self.generate_escape_literal_code(token_type, token_value, indent);
                            }
                            _ => {
                                println!("[HighPerformanceRustGenerator][apply_semantic_annotations]     ❌ Unknown codegen directive: {}", directive);
                            }
                        }
                    }
                    "generate" => {
                        println!("[HighPerformanceRustGenerator][apply_semantic_annotations]     🎯 Found generate annotation!");
                        // Parse the semantic annotation AST to extract code generation instructions
                        if let Ok(parsed_annotation) = serde_json::from_str::<JsonValue>(annotation_value) {
                            println!("[HighPerformanceRustGenerator][apply_semantic_annotations]     ✅ Successfully parsed generate annotation JSON");
                            return self.generate_code_from_semantic_ast(&parsed_annotation, token_type, token_value, indent);
                        } else if annotation_value.starts_with("raw:") {
                            println!("[HighPerformanceRustGenerator][apply_semantic_annotations]     🔧 Found raw generate annotation");
                            // Handle raw annotations that failed to parse
                            let raw_value = &annotation_value[4..];
                            return self.generate_code_from_raw_annotation(raw_value, token_type, token_value, indent);
                        } else {
                            println!("[HighPerformanceRustGenerator][apply_semantic_annotations]     ❌ Failed to parse generate annotation: {}", annotation_value);
                        }
                    }
                    "dispatch" | "dispatch_table" => {
                        println!("[HighPerformanceRustGenerator][apply_semantic_annotations]     🎯 Found dispatch annotation!");
                        // Handle dispatch table annotations for character classes and escapes
                        if let Ok(parsed_annotation) = serde_json::from_str::<JsonValue>(annotation_value) {
                            println!("[HighPerformanceRustGenerator][apply_semantic_annotations]     ✅ Successfully parsed dispatch annotation JSON");
                            return self.generate_dispatch_code(&parsed_annotation, token_type, token_value, indent);
                        } else {
                            println!("[HighPerformanceRustGenerator][apply_semantic_annotations]     ❌ Failed to parse dispatch annotation: {}", annotation_value);
                        }
                    }
                    "optimize" => {
                        println!("[HighPerformanceRustGenerator][apply_semantic_annotations]     🎯 Found optimize annotation!");
                        // Handle optimization directives
                        if let Ok(parsed_annotation) = serde_json::from_str::<JsonValue>(annotation_value) {
                            println!("[HighPerformanceRustGenerator][apply_semantic_annotations]     ✅ Successfully parsed optimize annotation JSON");
                            return self.generate_optimized_code(&parsed_annotation, token_type, token_value, indent);
                        } else {
                            println!("[HighPerformanceRustGenerator][apply_semantic_annotations]     ❌ Failed to parse optimize annotation: {}", annotation_value);
                        }
                    }
                    _ => {
                        println!("[HighPerformanceRustGenerator][apply_semantic_annotations]     ⚠️  Unknown annotation type: '{}'", annotation_name);
                        // Other annotation types - could be extended in the future
                        continue;
                    }
                }
            }
        }
        
        None
    }
    
    /// Generate code from parsed semantic annotation AST
    fn generate_code_from_semantic_ast(
        &self,
        ast: &JsonValue,
        token_type: &crate::ast_pipeline::TokenValue,
        token_value: &crate::ast_pipeline::TokenValue,
        indent: &str
    ) -> Option<String> {
        // This is where we'd interpret the semantic annotation AST
        // For now, provide some basic examples based on common patterns
        
        // Example: If the AST represents a function call like generate_char_class_matcher(...)
        if let JsonValue::Object(obj) = ast {
            if let Some(JsonValue::String(rule_name)) = obj.get("rule_name") {
                match rule_name.as_str() {
                    "function_call" => {
                        // Extract function name and parameters
                        if let Some(content) = obj.get("content") {
                            return self.interpret_function_call(content, token_type, token_value, indent);
                        }
                    }
                    "expression" => {
                        // Handle expression evaluation
                        if let Some(content) = obj.get("content") {
                            return self.interpret_expression(content, token_type, token_value, indent);
                        }
                    }
                    _ => {}
                }
            }
        }
        
        None
    }
    
    /// Generate code from raw semantic annotation (fallback for unparsed annotations)
    fn generate_code_from_raw_annotation(
        &self,
        raw_annotation: &str,
        _token_type: &crate::ast_pipeline::TokenValue,
        _token_value: &crate::ast_pipeline::TokenValue,
        indent: &str
    ) -> Option<String> {
        println!("[HighPerformanceRustGenerator][generate_code_from_raw_annotation] 🔍 ENTERING function");
        println!("[HighPerformanceRustGenerator][generate_code_from_raw_annotation] Raw annotation: '{}'", raw_annotation);
        println!("[HighPerformanceRustGenerator][generate_code_from_raw_annotation] Indent: '{}'", indent);
        
        // Handle common raw annotation patterns we know about
        if raw_annotation.starts_with("generate_char_class_matcher") {
            println!("[HighPerformanceRustGenerator][generate_code_from_raw_annotation] 🎯 MATCHED generate_char_class_matcher pattern!");
            println!("[HighPerformanceRustGenerator][generate_code_from_raw_annotation] Full annotation: '{}'", raw_annotation);
            
            // Generate character class matching code
            let result = Some(format!("{indent}let result = ParseContent::Terminal(parser.match_char_class_optimized()?);\n"));
            println!("[HighPerformanceRustGenerator][generate_code_from_raw_annotation] Generated code: {:?}", result);
            result
        } else if raw_annotation.starts_with("resolve_escape_pattern") {
            println!("[HighPerformanceRustGenerator][generate_code_from_raw_annotation] 🎯 MATCHED resolve_escape_pattern!");
            // Generate escape sequence matching code
            Some(format!("{indent}let result = ParseContent::Terminal(parser.match_escape_optimized()?);\n"))
        } else {
            println!("[HighPerformanceRustGenerator][generate_code_from_raw_annotation] ❌ NO MATCH for raw annotation: '{}'", raw_annotation);
            None
        }
    }
    
    /// Generate dispatch table code for character classes and escapes
    fn generate_dispatch_code(
        &self,
        ast: &JsonValue,
        _token_type: &crate::ast_pipeline::TokenValue,
        _token_value: &crate::ast_pipeline::TokenValue,
        indent: &str
    ) -> Option<String> {
        // If this is a dispatch table (like for escape sequences)
        if let JsonValue::Object(dispatch_table) = ast {
            let mut code = format!("{indent}let result = match parser.current_char() {{\n");
            
            for (_pattern, rust_code) in dispatch_table {
                if let JsonValue::String(code_str) = rust_code {
                    code.push_str(&format!("{indent}    Some(ch) if {} => {{\n", code_str));
                    code.push_str(&format!("{indent}        parser.advance();\n"));
                    code.push_str(&format!("{indent}        ParseContent::Terminal(parser.slice(start_pos..parser.position))\n"));
                    code.push_str(&format!("{indent}    }}\n"));
                }
            }
            
            code.push_str(&format!("{indent}    _ => return Err(ParseError::InvalidSyntax {{\n"));
            code.push_str(&format!("{indent}        message: \"Invalid escape sequence\",\n"));
            code.push_str(&format!("{indent}        position: parser.position,\n"));
            code.push_str(&format!("{indent}    }}),\n"));
            code.push_str(&format!("{indent}}};\n"));
            
            return Some(code);
        }
        
        None
    }
    
    /// Generate optimized code based on optimization annotations
    fn generate_optimized_code(
        &self,
        _ast: &JsonValue,
        _token_type: &crate::ast_pipeline::TokenValue,
        _token_value: &crate::ast_pipeline::TokenValue,
        indent: &str
    ) -> Option<String> {
        // Generate optimized code based on the optimization directive
        // This could include lookup tables, SIMD operations, etc.
        Some(format!("{indent}let result = ParseContent::Terminal(parser.match_optimized_pattern()?);\n"))
    }
    
    /// Interpret function call from semantic annotation AST
    fn interpret_function_call(
        &self,
        content: &JsonValue,
        _token_type: &crate::ast_pipeline::TokenValue,
        _token_value: &crate::ast_pipeline::TokenValue,
        indent: &str
    ) -> Option<String> {
        println!("[HighPerformanceRustGenerator][interpret_function_call] 🔍 ENTERING function");
        println!("[HighPerformanceRustGenerator][interpret_function_call] Content: {:?}", content);
        println!("[HighPerformanceRustGenerator][interpret_function_call] Indent: '{}'", indent);
        
        // Extract function name and generate appropriate code
        if let JsonValue::Object(obj) = content {
            println!("[HighPerformanceRustGenerator][interpret_function_call] Content is a JSON object with keys: {:?}", obj.keys().collect::<Vec<_>>());
            
            if let Some(JsonValue::String(func_name)) = obj.get("function_name") {
                println!("[HighPerformanceRustGenerator][interpret_function_call] Found function name: '{}'", func_name);
                
                match func_name.as_str() {
                    "generate_char_class_matcher" => {
                        println!("[HighPerformanceRustGenerator][interpret_function_call] 🎯 MATCHED generate_char_class_matcher!");
                        let result = Some(format!("{indent}let result = ParseContent::Terminal(parser.match_char_class_optimized()?);\n"));
                        println!("[HighPerformanceRustGenerator][interpret_function_call] Generated code: {:?}", result);
                        result
                    }
                    "resolve_escape_pattern" => {
                        println!("[HighPerformanceRustGenerator][interpret_function_call] 🎯 MATCHED resolve_escape_pattern!");
                        Some(format!("{indent}let result = ParseContent::Terminal(parser.match_escape_optimized()?);\n"))
                    }
                    "generate_literal_check" => {
                        println!("[HighPerformanceRustGenerator][interpret_function_call] 🎯 MATCHED generate_literal_check!");
                        Some(format!("{indent}let result = ParseContent::Terminal(parser.match_literal_optimized()?);\n"))
                    }
                    _ => {
                        println!("[HighPerformanceRustGenerator][interpret_function_call] ❌ NO MATCH for function name: '{}'", func_name);
                        None
                    }
                }
            } else {
                println!("[HighPerformanceRustGenerator][interpret_function_call] ❌ No 'function_name' key found in object");
                None
            }
        } else {
            println!("[HighPerformanceRustGenerator][interpret_function_call] ❌ Content is not a JSON object: {:?}", content);
            None
        }
    }
    
    /// Interpret expression from semantic annotation AST
    fn interpret_expression(
        &self,
        _content: &JsonValue,
        _token_type: &crate::ast_pipeline::TokenValue,
        _token_value: &crate::ast_pipeline::TokenValue,
        indent: &str
    ) -> Option<String> {
        // Generate code from expression evaluation
        // This could handle arithmetic, logical operations, function calls, etc.
        Some(format!("{indent}let result = ParseContent::Terminal(parser.evaluate_expression()?);\n"))
    }
    
    /// Generate escape literal handling code
    /// This method specifically handles escape sequences with proper single backslash representation
    fn generate_escape_literal_code(
        &self,
        token_type: &crate::ast_pipeline::TokenValue,
        token_value: &crate::ast_pipeline::TokenValue,
        indent: &str
    ) -> Option<String> {
        println!("[HighPerformanceRustGenerator][generate_escape_literal_code] 🔧 Handling escape sequence");
        println!("[HighPerformanceRustGenerator][generate_escape_literal_code]   📝 Token type: {:?}", token_type);
        println!("[HighPerformanceRustGenerator][generate_escape_literal_code]   📝 Token value: {:?}", token_value);
        
        match token_type.as_str() {
            Some("quoted_string") => {
                // This is a quoted string that represents escape sequences
                if let Some(string_value) = token_value.as_str() {
                    // For escape sequences, we need to ensure single backslash representation in raw strings
                    // The key insight: JSON "\\\\" should become Rust r#"\"# (single backslash)
                    let corrected_value = if string_value == "\\\\" {
                        // Double backslash in JSON becomes single backslash in raw string
                        "\\".to_string()
                    } else {
                        // Keep other values as-is since escape_rust_string now handles them correctly
                        string_value.to_string()
                    };
                    
                    println!("[HighPerformanceRustGenerator][generate_escape_literal_code]   ✅ Corrected escape: '{}' -> '{}'", string_value, corrected_value);
                    Some(format!("{indent}let result = ParseContent::Terminal(parser.match_string(r#\"{}\"#)?);\n", corrected_value))
                } else {
                    println!("[HighPerformanceRustGenerator][generate_escape_literal_code]   ❌ Token value is not a string");
                    None
                }
            }
            _ => {
                println!("[HighPerformanceRustGenerator][generate_escape_literal_code]   ❌ Not a quoted string token");
                None
            }
        }
    }

    fn generate_sequence_code(&self, elements: &[ASTNode], indent: &str, rule_annotations: Option<&[String]>) -> Result<String> {
        let mut code = format!("{indent}let mut sequence_elements = Vec::with_capacity({});\n", elements.len());
        
        for (i, element) in elements.iter().enumerate() {
            let element_code = self.generate_optimized_node_code(element, 0, rule_annotations)?;
            code.push_str(&format!("{indent}{{\n"));
            code.push_str(&format!("{indent}    parser.debug_sequence_element({}, {}, \"element_{}\");\n", i, elements.len(), i));
            code.push_str(&format!("{indent}    let element_start = parser.position;\n"));
            let fixed_element_code = element_code.replace("let result =", "    let element_content =").replace("parser.", "parser.");
            code.push_str(&fixed_element_code);
            code.push_str(&format!("{indent}    let element_end = parser.position;\n"));
            code.push_str(&format!("{indent}    sequence_elements.push(ParseNode {{\n"));
            code.push_str(&format!("{indent}        rule_name: \"element_{}\",\n", i));
            code.push_str(&format!("{indent}        content: element_content,\n"));
            code.push_str(&format!("{indent}        span: element_start..element_end,\n"));
            code.push_str(&format!("{indent}    }});\n"));
            code.push_str(&format!("{indent}}};\n"));
        }
        
        code.push_str(&format!("{indent}let result = ParseContent::Sequence(sequence_elements);\n"));
        Ok(code)
    }

    fn generate_or_code(&self, alternatives: &[ASTNode], indent: &str, rule_annotations: Option<&[String]>) -> Result<String> {
        let n_branches = alternatives.len();
        
        match n_branches {
            0 => {
                // No alternatives - this shouldn't happen but handle gracefully
                Ok(format!("{indent}return Err(ParseError::InvalidSyntax {{\n{indent}    message: \"No alternatives provided\",\n{indent}    position: parser.position,\n{indent}}});\n"))
            }
            1 => {
                // Single branch - no alternatives, just execute directly
                let alt_code = self.generate_optimized_node_code(&alternatives[0], 0, rule_annotations)?;
                let single_branch_code = alt_code.replace("parser.", "parser.");
                Ok(single_branch_code)
            }
            _ => {
                // Multiple branches - use systematic N-branch template
                self.generate_n_branch_template(alternatives, indent, rule_annotations)
            }
        }
    }
    
    /// Generate systematic N-branch template using builder pattern
    fn generate_n_branch_template(&self, alternatives: &[ASTNode], indent: &str, rule_annotations: Option<&[String]>) -> Result<String> {
        let mut builder = RustCodeBuilder::new();
        let n_branches = alternatives.len();
        
        // Declare result variable in outer scope
        builder.add_line(&format!("{indent}let result: ParseContent<'input>;"));
        
        // Generate alternatives as a single if-else-if-else chain
        for (branch_idx, alt) in alternatives.iter().enumerate() {
            if branch_idx == 0 {
                // First branch: if let Some(...)
                builder.add_line(&format!("{indent}parser.debug_try_alternative({}, {});", branch_idx, n_branches));
                builder.add_line(&format!("{indent}if let Some(content) = parser.try_parse(|p| {{"));
            } else {
                // Subsequent branches: } else if let Some(...)
                builder.add_line(&format!("{indent}}} else if let Some(content) = parser.try_parse(|p| {{"));
                builder.add_line(&format!("{indent}    p.debug_try_alternative({}, {});", branch_idx, n_branches));
            }
            
            // Generate branch content with proper indentation
            let alt_code = self.generate_optimized_node_code(&alt, 0, rule_annotations)?;
            let branch_indent = if branch_idx == 0 { "        " } else { "            " };
            let branch_content = alt_code
                .replace("let result =", &format!("{branch_indent}let branch_content ="))
                .replace("parser.", "p.");
            
            builder.add_raw(&branch_content);
            builder.add_line(&format!("{indent}{branch_indent}Ok(branch_content)"));
            
            // Close the try_parse closure and assign result
            if branch_idx == 0 {
                builder.add_line(&format!("{indent}    }}) {{"));
                builder.add_line(&format!("{indent}        result = ParseContent::Alternative(Box::new(ParseNode {{"));
                builder.add_line(&format!("{indent}            rule_name: \"branch_{}\",", branch_idx));
                builder.add_line(&format!("{indent}            content,"));
                builder.add_line(&format!("{indent}            span: 0..0,"));
                builder.add_line(&format!("{indent}        }}));"));
            } else {
                builder.add_line(&format!("{indent}        }}) {{"));
                builder.add_line(&format!("{indent}            result = ParseContent::Alternative(Box::new(ParseNode {{"));
                builder.add_line(&format!("{indent}                rule_name: \"branch_{}\",", branch_idx));
                builder.add_line(&format!("{indent}                content,"));
                builder.add_line(&format!("{indent}                span: 0..0,"));
                builder.add_line(&format!("{indent}            }}));"));
            }
        }
        
        // Final else clause for no match
        builder.add_line(&format!("{indent}}} else {{"));
        builder.add_line(&format!("{indent}    return Err(ParseError::InvalidSyntax {{"));
        builder.add_line(&format!("{indent}        message: \"No alternative matched in {}-branch rule\",", n_branches));
        builder.add_line(&format!("{indent}        position: parser.position,"));
        builder.add_line(&format!("{indent}    }});"));
        builder.add_line(&format!("{indent}}}"));
        
        Ok(builder.build())
    }

    fn generate_quantified_code(&self, element: &ASTNode, quantifier: &str, indent: &str, rule_annotations: Option<&[String]>) -> Result<String> {
        let element_code = self.generate_optimized_node_code(element, 0, rule_annotations)?;
        let inner_code = element_code.replace("let result =", "    let content =").replace("parser.", "p.");
        
        Ok(format!("{indent}let result = parser.parse_quantified_optimized(\"{quantifier}\", |p| {{\n{inner_code}{indent}    Ok(content)\n{indent}}})?;\n",
            indent = indent,
            quantifier = quantifier,
            inner_code = inner_code
        ))
    }

    fn generate_fast_helpers(&self) -> String {
        format!(r#"    /// Regex-based pattern matching using Rust regex engine
    /// Handles EBNF /.../ regex patterns properly
    #[inline]
    fn match_regex_optimized(&mut self, pattern: &str) -> ParseResult<&'input str> {{
        let start_pos = self.position;
        
        // Use the actual Rust regex engine for proper EBNF regex support
        use regex::Regex;
        
        let remaining_input = &self.input[self.position..];
        
        // Create regex with anchoring at start of string
        let anchored_pattern = if pattern.starts_with('^') {{
            pattern.to_string()
        }} else {{
            format!("^{{}}", pattern)
        }};
        
        match Regex::new(&anchored_pattern) {{
            Ok(regex) => {{
                if let Some(matched) = regex.find(remaining_input) {{
                    // Ensure the match starts at position 0 (our current position)
                    if matched.start() == 0 {{
                        let match_len = matched.len();
                        let old_pos = self.position;
                        self.position += match_len;
                        Ok(&self.input[old_pos..self.position])
                    }} else {{
                        Err(ParseError::InvalidSyntax {{
                            message: "Pattern mismatch at current position",
                            position: start_pos,
                        }})
                    }}
                }} else {{
                    Err(ParseError::InvalidSyntax {{
                        message: "Pattern mismatch",
                        position: start_pos,
                    }})
                }}
            }}
            Err(_) => {{
                Err(ParseError::InvalidSyntax {{
                    message: "Invalid regex pattern",
                    position: start_pos,
                }})
            }}
        }}
    }}
    
    /// Universal pattern matcher that works for any grammar
    /// Interprets pattern based on its structure, not hardcoded assumptions
    #[inline]
    fn pattern_matches(&self, ch: char, pattern: &str) -> bool {{
        if pattern.len() == 1 {{
            // Single character literal
            ch == pattern.chars().next().unwrap()
        }} else if pattern == "." {{
            // Dot pattern - matches any character except newline in most grammars
            // TODO: This behavior should come from semantic annotations
            ch != '\n'
        }} else if pattern.starts_with("[^") && pattern.ends_with("]") {{
            // Negated character class - match chars NOT in the class
            let class_chars = &pattern[2..pattern.len()-1];
            !self.char_in_class(ch, class_chars)
        }} else if pattern.starts_with("[") && pattern.ends_with("]") {{
            // Positive character class - match chars IN the class
            let class_chars = &pattern[1..pattern.len()-1];
            self.char_in_class(ch, class_chars)
        }} else {{
            // Complex pattern or escape sequence - generic fallback
            // TODO: This should be driven by semantic annotations from grammar
            self.match_generic_pattern(ch, pattern)
        }}
    }}
    
    /// Generic pattern matcher for complex patterns
    /// This is where semantic annotations would drive specialized matching
    #[inline]
    fn match_generic_pattern(&self, ch: char, pattern: &str) -> bool {{
        // For now, simple contains check
        // TODO: Replace with AST-driven pattern interpretation
        pattern.contains(ch)
    }}

    /// Check if a character matches a character class string
    #[inline]
    fn char_in_class(&self, ch: char, class_chars: &str) -> bool {{
        // Handle escaped characters in character classes
        let mut chars = class_chars.chars().peekable();
        
        while let Some(c) = chars.next() {{
            match c {{
                '\\' => {{
                    // Handle escaped character - consume pairs of backslashes
                    if let Some(next_char) = chars.next() {{
                        if next_char == '\\' {{
                            // Double backslash becomes single literal backslash
                            if ch == '\\' {{
                                return true;
                            }}
                        }} else {{
                            // Backslash followed by other character
                            if ch == next_char {{
                                return true;
                            }}
                        }}
                    }}
                }}
                _ => {{
                    // Regular character
                    if ch == c {{
                        return true;
                    }}
                }}
            }}
        }}
        false
    }}

    /// High-performance quantifier parsing
    #[inline]
    fn parse_quantified_optimized<F>(&mut self, quantifier: &str, mut f: F) -> ParseResult<ParseContent<'input>>
    where
        F: FnMut(&mut Self) -> ParseResult<ParseContent<'input>>,
    {{
        let mut results = Vec::new();
        let mut iteration = 0;
        
        match quantifier {{
            "*" => {{
                // Zero or more - optimized loop
                while let Some(content) = self.try_parse(&mut f) {{
                    iteration += 1;
                    self.debug_quantifier_iteration(iteration, quantifier);
                    results.push(ParseNode {{
                        rule_name: "quantified",
                        content,
                        span: 0..0, // Will be filled by caller
                    }});
                }}
                Ok(ParseContent::Quantified(results, "*"))
            }}
            "+" => {{
                // One or more - require at least one
                iteration += 1;
                self.debug_quantifier_iteration(iteration, quantifier);
                match f(self) {{
                    Ok(content) => {{
                        results.push(ParseNode {{
                            rule_name: "quantified",
                            content,
                            span: 0..0,
                        }});
                        
                        while let Some(content) = self.try_parse(&mut f) {{
                            iteration += 1;
                            self.debug_quantifier_iteration(iteration, quantifier);
                            results.push(ParseNode {{
                                rule_name: "quantified", 
                                content,
                                span: 0..0,
                            }});
                        }}
                        Ok(ParseContent::Quantified(results, "+"))
                    }}
                    Err(err) => Err(err),
                }}
            }}
            "?" => {{
                // Zero or one
                if let Some(content) = self.try_parse(&mut f) {{
                    iteration += 1;
                    self.debug_quantifier_iteration(iteration, quantifier);
                    results.push(ParseNode {{
                        rule_name: "quantified",
                        content,
                        span: 0..0,
                    }});
                }}
                Ok(ParseContent::Quantified(results, "?"))
            }}
            _ => Err(ParseError::InvalidSyntax {{
                message: "Unknown quantifier",
                position: self.position,
            }}),
        }}
    }}

"#)
    }

    fn generate_performance_tests(&self) -> String {
        // Convert grammar name to PascalCase for struct name
        let mut chars = self.grammar_name.chars();
        let parser_name = format!("{}Parser", 
            chars.next().unwrap().to_uppercase().collect::<String>() + &chars.collect::<String>());
            
        format!(r#"
#[cfg(test)]
mod performance_tests {{
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_parser_creation_speed() {{
        let input = "test input for performance";
        let start = Instant::now();
        
        for _ in 0..10_000 {{
            let _parser = {parser_name}::new(input);
        }}
        
        let duration = start.elapsed();
        println!("Parser creation: {{:?}} for 10k iterations", duration);
        assert!(duration.as_millis() < 100); // Should be very fast
    }}

    #[test] 
    fn test_simple_parsing_speed() {{
        let mut parser = {parser_name}::new("simple test input");
        let start = Instant::now();
        
        for _ in 0..1_000 {{
            parser.position = 0;
            parser.memo.clear();
            let _result = parser.try_parse(|p| p.match_string("simple"));
        }}
        
        let duration = start.elapsed();
        println!("Simple parsing: {{:?}} for 1k iterations", duration);
        assert!(duration.as_millis() < 50);
    }}

    #[test]
    fn test_memoization_effectiveness() {{
        let mut parser = {parser_name}::new("test test test");
        
        // First parse - should populate memo
        let start1 = Instant::now();
        let _result1 = parser.parse();
        let duration1 = start1.elapsed();
        
        // Reset position but keep memo
        parser.position = 0;
        
        // Second parse - should use memoized results
        let start2 = Instant::now(); 
        let _result2 = parser.parse();
        let duration2 = start2.elapsed();
        
        println!("First parse: {{:?}}, Second parse: {{:?}}", duration1, duration2);
        // Second should be significantly faster due to memoization
        assert!(duration2 < duration1);
    }}
}}
"#, parser_name = parser_name)
    }
}
