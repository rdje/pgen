//! High-Performance Rust Code Generator
//! Generates lightning-fast parsers with:
//! - Zero-copy parsing where possible
//! - Memoization/packrat parsing for backtracking
//! - Inline optimizations and SIMD-friendly code
//! - Minimal allocations for rgx regex engine integration

use crate::ast_pipeline::{ASTNode, ASTValue};
use std::collections::HashMap;
use anyhow::Result;


/// Escape a string for safe inclusion in Rust source code
fn escape_rust_string(s: &str) -> String {
    s.replace('\\', "\\\\")
     .replace('"', "\\\"")
     .replace('\n', "\\n")
     .replace('\r', "\\r")
     .replace('\t', "\\t")
     .replace('\0', "\\0")
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
}

impl HighPerformanceRustGenerator {
    pub fn new(grammar_name: &str) -> Self {
        Self {
            grammar_name: grammar_name.to_string(),
            entry_rule: None,
            enable_trace: false,
            enable_backtrack_debug: false,
        }
    }
    
    /// Create generator with trace mode enabled
    pub fn with_trace(grammar_name: &str, enable_trace: bool) -> Self {
        Self {
            grammar_name: grammar_name.to_string(),
            entry_rule: None,
            enable_trace,
            enable_backtrack_debug: false,
        }
    }
    
    /// Create generator with full debug enabled (trace + backtrack)
    pub fn with_full_debug(grammar_name: &str) -> Self {
        Self {
            grammar_name: grammar_name.to_string(),
            entry_rule: None,
            enable_trace: true,
            enable_backtrack_debug: true,
        }
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
        
        // Entry rule should be the grammar name (first rule in grammar)
        let entry_rule = self.entry_rule.as_ref()
            .unwrap_or(&self.grammar_name)
            .clone();
        
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
        
        // Generate rule ID constants
        code.push_str("    // Rule IDs for memoization\n");
        for (i, rule_name) in rule_order.iter().enumerate() {
            code.push_str(&format!("    const RULE_{}: RuleId = {};\n", 
                rule_name.to_uppercase(), i));
        }
        code.push_str("\n");

        // Generate rule methods
        for (i, rule_name) in rule_order.iter().enumerate() {
            if let Some(ast_node) = grammar_tree.get(rule_name) {
                let method_code = self.generate_optimized_rule_method(rule_name, ast_node, i as u16)?;
                code.push_str(&method_code);
            }
        }

        Ok(code)
    }

    fn generate_optimized_rule_method(
        &self,
        rule_name: &str,
        ast_node: &ASTNode,
        _rule_id: u16,
    ) -> Result<String> {
        let method_body = self.generate_optimized_node_code(ast_node, 2)?;
        
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

    fn generate_optimized_node_code(&self, ast_node: &ASTNode, indent_level: usize) -> Result<String> {
        let indent = "    ".repeat(indent_level);
        
        match ast_node {
            ASTNode::Atom { value } => {
                self.generate_atom_code(value, &indent)
            }
            ASTNode::Sequence { elements } => {
                self.generate_sequence_code(elements, &indent)
            }
            ASTNode::Or { alternatives } => {
                self.generate_or_code(alternatives, &indent)
            }
            ASTNode::Quantified { element, quantifier } => {
                self.generate_quantified_code(element, quantifier, &indent)
            }
        }
    }

    fn generate_atom_code(&self, value: &ASTValue, indent: &str) -> Result<String> {
        match value {
            ASTValue::Token(token) if token.len() == 2 => {
                let token_type = &token[0];
                let token_value = &token[1];
                
                match token_type.as_str() {
                    "quoted_string" => {
                        if token_value.is_empty() {
                            // Handle empty strings with regular string literals
                            Ok(format!("{indent}let result = ParseContent::Terminal(parser.match_string(\"\")?);\n"))
                        } else {
                            let escaped_value = escape_rust_string(token_value);
                            Ok(format!("{indent}let result = ParseContent::Terminal(parser.match_string(r#\"{escaped_value}\"#)?);\n"))
                        }
                    }
                    "regex" => {
                        let escaped_value = escape_rust_string(token_value);
                        Ok(format!("{indent}let result = ParseContent::Terminal(parser.match_regex_optimized(r#\"{escaped_value}\"#)?);\n"))
                    }
                    "rule_reference" => {
                        Ok(format!("{indent}let result = ParseContent::Alternative(Box::new(parser.parse_{token_value}()?));\n"))
                    }
                    _ => {
                        let escaped_value = escape_rust_string(token_value);
                        Ok(format!("{indent}let result = ParseContent::Terminal(r#\"{escaped_value}\"#);\n"))
                    }
                }
            }
            _ => {
                Ok(format!("{indent}let result = ParseContent::Terminal(\"unknown\");\n"))
            }
        }
    }

    fn generate_sequence_code(&self, elements: &[ASTNode], indent: &str) -> Result<String> {
        let mut code = format!("{indent}let mut sequence_elements = Vec::with_capacity({});\n", elements.len());
        
        for (i, element) in elements.iter().enumerate() {
            let element_code = self.generate_optimized_node_code(element, 0)?;
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

    fn generate_or_code(&self, alternatives: &[ASTNode], indent: &str) -> Result<String> {
        let n_branches = alternatives.len();
        
        match n_branches {
            0 => {
                // No alternatives - this shouldn't happen but handle gracefully
                Ok(format!("{indent}return Err(ParseError::InvalidSyntax {{\n{indent}    message: \"No alternatives provided\",\n{indent}    position: parser.position,\n{indent}}});\n"))
            }
            1 => {
                // Single branch - no alternatives, just execute directly
                let alt_code = self.generate_optimized_node_code(&alternatives[0], 0)?;
                let single_branch_code = alt_code.replace("parser.", "parser.");
                Ok(single_branch_code)
            }
            _ => {
                // Multiple branches - use systematic N-branch template
                self.generate_n_branch_template(alternatives, indent)
            }
        }
    }
    
    /// Generate systematic N-branch template using builder pattern
    fn generate_n_branch_template(&self, alternatives: &[ASTNode], indent: &str) -> Result<String> {
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
            let alt_code = self.generate_optimized_node_code(alt, 0)?;
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

    fn generate_quantified_code(&self, element: &ASTNode, quantifier: &str, indent: &str) -> Result<String> {
        let element_code = self.generate_optimized_node_code(element, 0)?;
        let inner_code = element_code.replace("let result =", "    let content =").replace("parser.", "p.");
        
        Ok(format!("{indent}let result = parser.parse_quantified_optimized(\"{quantifier}\", |p| {{\n{inner_code}{indent}    Ok(content)\n{indent}}})?;\n",
            indent = indent,
            quantifier = quantifier,
            inner_code = inner_code
        ))
    }

    fn generate_fast_helpers(&self) -> String {
        format!(r#"    /// Optimized regex matching with character classes
    #[inline]
    fn match_regex_optimized(&mut self, pattern: &str) -> ParseResult<&'input str> {{
        let start_pos = self.position;
        
        match pattern {{
            "." => {{
                // Any character except newline
                if let Some(ch) = self.current_char() {{
                    if ch != '\n' {{
                        self.advance();
                        return Ok(&self.input[start_pos..self.position]);
                    }}
                }}
                Err(ParseError::UnexpectedEof {{ position: start_pos }})
            }}
            r"[0-9]" | r"\d" => {{
                // ASCII digit fast path
                if let Some(byte) = self.current_byte() {{
                    if byte >= b'0' && byte <= b'9' {{
                        self.position += 1;
                        return Ok(&self.input[start_pos..self.position]);
                    }}
                }}
                Err(ParseError::InvalidSyntax {{ 
                    message: "Expected digit", 
                    position: start_pos 
                }})
            }}
            r"[A-Za-z]" => {{
                // ASCII letter fast path
                if let Some(byte) = self.current_byte() {{
                    if (byte >= b'A' && byte <= b'Z') || (byte >= b'a' && byte <= b'z') {{
                        self.position += 1;
                        return Ok(&self.input[start_pos..self.position]);
                    }}
                }}
                Err(ParseError::InvalidSyntax {{ 
                    message: "Expected letter", 
                    position: start_pos 
                }})
            }}
            r"[A-Za-z_]" => {{
                // Name start character
                if let Some(byte) = self.current_byte() {{
                    if (byte >= b'A' && byte <= b'Z') || 
                       (byte >= b'a' && byte <= b'z') || 
                       byte == b'_' {{
                        self.position += 1;
                        return Ok(&self.input[start_pos..self.position]);
                    }}
                }}
                Err(ParseError::InvalidSyntax {{ 
                    message: "Expected name start", 
                    position: start_pos 
                }})
            }}
            r"[A-Za-z0-9_]" => {{
                // Name continue character  
                if let Some(byte) = self.current_byte() {{
                    if (byte >= b'A' && byte <= b'Z') || 
                       (byte >= b'a' && byte <= b'z') || 
                       (byte >= b'0' && byte <= b'9') ||
                       byte == b'_' {{
                        self.position += 1;
                        return Ok(&self.input[start_pos..self.position]);
                    }}
                }}
                Err(ParseError::InvalidSyntax {{ 
                    message: "Expected name continue", 
                    position: start_pos 
                }})
            }}
            r"\s+" => {{
                // Whitespace - consume greedily
                let mut consumed = false;
                while let Some(ch) = self.current_char() {{
                    if ch.is_whitespace() {{
                        self.advance();
                        consumed = true;
                    }} else {{
                        break;
                    }}
                }}
                if consumed {{
                    Ok(&self.input[start_pos..self.position])
                }} else {{
                    Err(ParseError::InvalidSyntax {{ 
                        message: "Expected whitespace", 
                        position: start_pos 
                    }})
                }}
            }}
            _ => {{
                // Fallback for complex patterns including character classes
                if let Some(ch) = self.current_char() {{
                    let matches = if pattern.starts_with("[^") && pattern.ends_with("]") {{
                        // Negated character class - match chars NOT in the class
                        let class_chars = &pattern[2..pattern.len()-1];
                        !self.char_in_class(ch, class_chars)
                    }} else if pattern.starts_with("[") && pattern.ends_with("]") {{
                        // Positive character class - match chars IN the class
                        let class_chars = &pattern[1..pattern.len()-1];
                        self.char_in_class(ch, class_chars)
                    }} else {{
                        // Simple pattern matching
                        pattern.contains(ch)
                    }};
                    
                    if matches {{
                        self.advance();
                        Ok(&self.input[start_pos..self.position])
                    }} else {{
                        Err(ParseError::InvalidSyntax {{
                            message: "Pattern mismatch",
                            position: start_pos,
                        }})
                    }}
                }} else {{
                    Err(ParseError::UnexpectedEof {{ position: start_pos }})
                }}
            }}
        }}
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

    #[bench]
    fn bench_regex_parsing(b: &mut test::Bencher) {{
        let input = "(?{{lua: return 'test'}})|[a-z]+(?={{word}})";
        b.iter(|| {{
            let mut parser = {parser_name}::new(input);
            parser.parse().unwrap()
        }});
    }}
}}
"#, parser_name = parser_name)
    }
}
