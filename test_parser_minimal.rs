//! regex High-Performance Parser
//! Generated for rgx regex engine - SOTA performance
//! Features: Zero-copy, memoization, SIMD-optimized, minimal allocations

use std::{
    collections::HashMap,
    fmt,
    ops::Range,
};

/// Parse result with zero-copy string slices
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    UnexpectedEof { position: usize },
    UnexpectedToken { expected: &'static str, found: char, position: usize },
    InvalidSyntax { message: &'static str, position: usize },
    Backtrack { position: usize },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::UnexpectedEof { position } => 
                write!(f, "Unexpected EOF at position {}", position),
            ParseError::UnexpectedToken { expected, found, position } => 
                write!(f, "Expected '{}', found '{}' at position {}", expected, found, position),
            ParseError::InvalidSyntax { message, position } => 
                write!(f, "{} at position {}", message, position),
            ParseError::Backtrack { position } => 
                write!(f, "Backtrack at position {}", position),
        }
    }
}

impl std::error::Error for ParseError {}

pub type ParseResult<T> = Result<T, ParseError>;

/// Zero-copy AST node with string slice references
#[derive(Debug, Clone, PartialEq)]
pub struct ParseNode<'input> {
    pub rule_name: &'static str,
    pub content: ParseContent<'input>,
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParseContent<'input> {
    Terminal(&'input str),
    Sequence(Vec<ParseNode<'input>>),
    Alternative(Box<ParseNode<'input>>),
    Quantified(Vec<ParseNode<'input>>, &'static str),
}

/// Memoization entry for packrat parsing
#[derive(Debug, Clone)]
struct MemoEntry<'input> {
    result: Option<ParseNode<'input>>,
    end_pos: usize,
}

/// Compact rule ID for fast memoization lookups
type RuleId = u16;

/// High-Performance parser with memoization and zero-copy parsing
pub struct regexParser<'input> {
    input: &'input str,
    position: usize,
    memo: HashMap<(RuleId, usize), MemoEntry<'input>>,
    bytes: &'input [u8], // For SIMD optimizations
    debug_mode: bool,
    debug_depth: usize,
    debug_output: Vec<String>,
}

impl<'input> regexParser<'input> {
    /// Create new parser with zero-copy input
    #[inline]
    pub fn new(input: &'input str) -> Self {
        Self {
            input,
            position: 0,
            memo: HashMap::with_capacity(1024), // Pre-allocate memo table
            bytes: input.as_bytes(),
            debug_mode: false,
            debug_depth: 0,
            debug_output: Vec::new(),
        }
    }
    
    /// Create new parser with debug mode enabled
    #[inline]
    pub fn with_debug(input: &'input str) -> Self {
        Self {
            input,
            position: 0,
            memo: HashMap::with_capacity(1024),
            bytes: input.as_bytes(),
            debug_mode: true,
            debug_depth: 0,
            debug_output: Vec::new(),
        }
    }
    
    /// Get debug output for analysis
    pub fn debug_output(&self) -> &[String] {
        &self.debug_output
    }
    
    /// Clear debug output
    pub fn clear_debug(&mut self) {
        self.debug_output.clear();
        self.debug_depth = 0;
    }

    /// Parse entry point - returns AST or error
    pub fn parse(&mut self) -> ParseResult<ParseNode<'input>> {
        self.position = 0;
        self.memo.clear();
        self.parse_regex() // Entry rule
    }

    /// Fast character access with bounds checking
    #[inline(always)]
    fn current_char(&self) -> Option<char> {
        self.input.chars().nth(self.position)
    }

    /// SIMD-optimized byte access for ASCII fast path
    #[inline(always)]
    fn current_byte(&self) -> Option<u8> {
        self.bytes.get(self.position).copied()
    }

    /// Advance position with UTF-8 awareness
    #[inline(always)]
    fn advance(&mut self) -> Option<char> {
        if let Some(ch) = self.current_char() {
            self.position += ch.len_utf8();
            Some(ch)
        } else {
            None
        }
    }

    /// Fast advance for ASCII characters
    #[inline(always)]
    fn advance_ascii(&mut self) -> Option<u8> {
        if let Some(byte) = self.current_byte() {
            if byte < 128 {
                self.position += 1;
                Some(byte)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get zero-copy slice from input
    #[inline(always)]
    fn slice(&self, range: Range<usize>) -> &'input str {
        &self.input[range]
    }

    /// Memoized rule call with packrat parsing
    #[inline]
    fn memoized_call<F>(&mut self, rule_id: RuleId, f: F) -> ParseResult<ParseNode<'input>>
    where
        F: FnOnce(&mut Self) -> ParseResult<ParseNode<'input>>,
    {
        let key = (rule_id, self.position);
        
        // Check memo table
        if let Some(entry) = self.memo.get(&key) {
            self.position = entry.end_pos;
            match &entry.result {
                Some(node) => Ok(node.clone()),
                None => Err(ParseError::Backtrack { position: self.position }),
            }
        } else {
            // Not memoized - compute result
            let start_pos = self.position;
            match f(self) {
                Ok(node) => {
                    let end_pos = self.position;
                    self.memo.insert(key, MemoEntry {
                        result: Some(node.clone()),
                        end_pos,
                    });
                    Ok(node)
                }
                Err(err) => {
                    self.memo.insert(key, MemoEntry {
                        result: None,
                        end_pos: start_pos,
                    });
                    Err(err)
                }
            }
        }
    }

    /// Fast string matching with SIMD potential
    #[inline]
    fn match_string(&mut self, expected: &'static str) -> ParseResult<&'input str> {
        let start_pos = self.position;
        
        // ASCII fast path for single characters
        if expected.len() == 1 {
            let expected_byte = expected.as_bytes()[0];
            if expected_byte < 128 {
                if let Some(byte) = self.current_byte() {
                    if byte == expected_byte {
                        self.position += 1;
                        return Ok(&self.input[start_pos..self.position]);
                    }
                }
                return Err(ParseError::UnexpectedToken {
                    expected,
                    found: self.current_char().unwrap_or('\0'),
                    position: start_pos,
                });
            }
        }
        
        // General UTF-8 path
        for (i, expected_char) in expected.chars().enumerate() {
            match self.current_char() {
                Some(ch) if ch == expected_char => {
                    self.advance();
                }
                Some(found) => {
                    self.position = start_pos;
                    return Err(ParseError::UnexpectedToken {
                        expected,
                        found,
                        position: start_pos + i,
                    });
                }
                None => {
                    self.position = start_pos;
                    return Err(ParseError::UnexpectedEof {
                        position: start_pos + i,
                    });
                }
            }
        }
        
        Ok(&self.input[start_pos..self.position])
    }

    /// Try parsing with automatic backtracking
    #[inline]
    fn try_parse<T, F>(&mut self, f: F) -> Option<T>
    where
        F: FnOnce(&mut Self) -> ParseResult<T>,
    {
        let saved_pos = self.position;
        match f(self) {
            Ok(result) => Some(result),
            Err(_) => {
                self.position = saved_pos;
                None
            }
        }
    }
    
    /// Debug: Log entry into a parsing rule
    #[inline]
    fn debug_enter_rule(&mut self, rule_name: &str) {
        if self.debug_mode {
            let indent = "  ".repeat(self.debug_depth);
            let context = if self.position < self.input.len() {
                let end_pos = (self.position + 10).min(self.input.len());
                let context_str = self.format_debug_string(&self.input[self.position..end_pos]);
                format!(" at '{}'", context_str)
            } else {
                " at EOF".to_string()
            };
            let msg = format!("{}→ ENTER {}: pos={}{}", indent, rule_name, self.position, context);
            self.debug_output.push(msg);
            self.debug_depth += 1;
        }
    }
    
    /// Debug: Log successful exit from a parsing rule
    #[inline]
    fn debug_exit_success(&mut self, rule_name: &str, start_pos: usize) {
        if self.debug_mode {
            self.debug_depth = self.debug_depth.saturating_sub(1);
            let indent = "  ".repeat(self.debug_depth);
            let consumed = if self.position > start_pos {
                let consumed_str = self.format_debug_string(&self.input[start_pos..self.position]);
                format!(" consumed '{}'", consumed_str)
            } else {
                " (no input consumed)".to_string()
            };
            let msg = format!("{}← SUCCESS {}: {}->{}{}", 
                indent, rule_name, start_pos, self.position, consumed);
            self.debug_output.push(msg);
        }
    }
    
    /// Debug: Log failed exit from a parsing rule
    #[inline]
    fn debug_exit_fail(&mut self, rule_name: &str, error: &ParseError) {
        if self.debug_mode {
            self.debug_depth = self.debug_depth.saturating_sub(1);
            let indent = "  ".repeat(self.debug_depth);
            let msg = format!("{}← FAIL {}: {}", indent, rule_name, error);
            self.debug_output.push(msg);
        }
    }
    
    /// Debug: Log backtracking
    #[inline]
    fn debug_backtrack(&mut self, from_pos: usize, to_pos: usize, reason: &str) {
        if self.debug_mode {
            let indent = "  ".repeat(self.debug_depth);
            let msg = format!("{}⟲ BACKTRACK: {}->{} ({})", indent, from_pos, to_pos, reason);
            self.debug_output.push(msg);
        }
    }
    
    /// Debug: Log alternative attempt
    #[inline]
    fn debug_try_alternative(&mut self, alt_index: usize, total: usize) {
        if self.debug_mode {
            let indent = "  ".repeat(self.debug_depth);
            let msg = format!("{}? TRY ALT {}/{}: pos={}", indent, alt_index + 1, total, self.position);
            self.debug_output.push(msg);
        }
    }
    
    /// Debug: Log sequence element attempt
    #[inline]
    fn debug_sequence_element(&mut self, elem_index: usize, total: usize, elem_name: &str) {
        if self.debug_mode {
            let indent = "  ".repeat(self.debug_depth);
            let msg = format!("{}▶ SEQ {}/{}: {} at pos={}", 
                indent, elem_index + 1, total, elem_name, self.position);
            self.debug_output.push(msg);
        }
    }
    
    /// Debug: Log quantifier iteration
    #[inline]
    fn debug_quantifier_iteration(&mut self, iteration: usize, quantifier: &str) {
        if self.debug_mode {
            let indent = "  ".repeat(self.debug_depth);
            let msg = format!("{}* QUANT '{}' iteration {}: pos={}", 
                indent, quantifier, iteration, self.position);
            self.debug_output.push(msg);
        }
    }
    
    /// Helper function to format strings safely for debug output
    #[inline]
    fn format_debug_string(&self, s: &str) -> String {
        s.chars()
            .map(|c| match c {
                '\n' => "\\n".to_string(),
                '\r' => "\\r".to_string(),
                '\t' => "\\t".to_string(),
                '\\' => "\\\\".to_string(),
                '"' => "\\\"".to_string(),
                c if c.is_control() => format!("\\u{:04x}", c as u32),
                c => c.to_string(),
            })
            .collect()
    }

    // Rule IDs for memoization
    const RULE_INLINE_MODIFIERS: RuleId = 0;
    const RULE_WS: RuleId = 1;
    const RULE_CLASS_RANGE: RuleId = 2;
    const RULE_CODE_BLOCK: RuleId = 3;
    const RULE_CODE_STRING_SINGLE_CHAR: RuleId = 4;
    const RULE_CHAR_CLASS: RuleId = 5;
    const RULE_CODE_STRING_DOUBLE_CHAR: RuleId = 6;
    const RULE_PROPERTY_ESCAPE: RuleId = 7;
    const RULE_ALTERNATION: RuleId = 8;
    const RULE_UNICODE_ESCAPE: RuleId = 9;
    const RULE_ATOM: RuleId = 10;
    const RULE_NAME: RuleId = 11;
    const RULE_PATTERN: RuleId = 12;
    const RULE_SCOPED_INLINE_MODIFIERS: RuleId = 13;
    const RULE_CODE_ELEMENT: RuleId = 14;
    const RULE_LOOKAHEAD_NEG: RuleId = 15;
    const RULE_QUANT_SUFFIX: RuleId = 16;
    const RULE_NONCAPTURING_GROUP: RuleId = 17;
    const RULE_CODE_STRING_DOUBLE: RuleId = 18;
    const RULE_POSIX_CLASS: RuleId = 19;
    const RULE_LOOKBEHIND_NEG: RuleId = 20;
    const RULE_BACKREFERENCE: RuleId = 21;
    const RULE_ESCAPE: RuleId = 22;
    const RULE_LITERAL: RuleId = 23;
    const RULE_NAME_CONTINUE: RuleId = 24;
    const RULE_GROUP: RuleId = 25;
    const RULE_NAME_REF: RuleId = 26;
    const RULE_CODE_CONTENT: RuleId = 27;
    const RULE_MODIFIER_CHAR: RuleId = 28;
    const RULE_CODE_BLOCK_LANG: RuleId = 29;
    const RULE_COUNTED_QUANTIFIER: RuleId = 30;
    const RULE_CLASS_ESCAPE: RuleId = 31;
    const RULE_CLASS_BODY: RuleId = 32;
    const RULE_CONTROL_ESCAPE: RuleId = 33;
    const RULE_DIGITS: RuleId = 34;
    const RULE_MODIFIER_GROUP: RuleId = 35;
    const RULE_CODE_REGULAR_CHAR: RuleId = 36;
    const RULE_NAMED_GROUP: RuleId = 37;
    const RULE_YES_BRANCH: RuleId = 38;
    const RULE_CLASS_LITERAL: RuleId = 39;
    const RULE_POSIX_NAME: RuleId = 40;
    const RULE_HEX_ESCAPE: RuleId = 41;
    const RULE_CONDITIONAL: RuleId = 42;
    const RULE_PROP_NAME: RuleId = 43;
    const RULE_CONDITION: RuleId = 44;
    const RULE_CODE_LANG: RuleId = 45;
    const RULE_REGEX: RuleId = 46;
    const RULE_CODE_STRING_SINGLE: RuleId = 47;
    const RULE_CODE_BLOCK_PLAIN: RuleId = 48;
    const RULE_DOT: RuleId = 49;
    const RULE_QUANTIFIER: RuleId = 50;
    const RULE_NO_BRANCH: RuleId = 51;
    const RULE_LOOKAROUND: RuleId = 52;
    const RULE_CLASS_ITEM: RuleId = 53;
    const RULE_LOOKBEHIND_POS: RuleId = 54;
    const RULE_LOOKAHEAD_POS: RuleId = 55;
    const RULE_CODE_ESCAPED_CHAR: RuleId = 56;
    const RULE_CONCATENATION: RuleId = 57;
    const RULE_PIECE: RuleId = 58;
    const RULE_LITERAL_CHAR: RuleId = 59;
    const RULE_OCTAL_ESCAPE: RuleId = 60;
    const RULE_ATOMIC_GROUP: RuleId = 61;
    const RULE_QUANT_BASE: RuleId = 62;
    const RULE_ANCHOR: RuleId = 63;
    const RULE_CAPTURING_GROUP: RuleId = 64;
    const RULE_SIMPLE_ESCAPE: RuleId = 65;
    const RULE_NAME_START: RuleId = 66;
    const RULE_MODIFIER_SEQ: RuleId = 67;
    const RULE_CLASS_START_END_BRACKET: RuleId = 68;
    const RULE_ESCAPE_UNIT: RuleId = 69;
    const RULE_CLASS_ATOM: RuleId = 70;
    const RULE_CODE_STRING_ESCAPE: RuleId = 71;
    const RULE_CODE_BALANCED_BRACES: RuleId = 72;

    /// Parse inline_modifiers with memoization
    #[inline]
    fn parse_inline_modifiers(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_INLINE_MODIFIERS, |parser| {
            parser.debug_enter_rule("inline_modifiers");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let mut sequence_elements = Vec::with_capacity(3);
        {
            parser.debug_sequence_element(0, 3, "element_0");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string(r#"(?("#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_0",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(1, 3, "element_1");
            let element_start = parser.position;
    let element_content = ParseContent::Alternative(Box::new(parser.parse_modifier_seq()?));
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_1",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(2, 3, "element_2");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string(r#")"#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_2",
                content: element_content,
                span: element_start..element_end,
            });
        }
        let result = ParseContent::Sequence(sequence_elements);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "inline_modifiers",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("inline_modifiers", start_pos),
                Err(err) => parser.debug_exit_fail("inline_modifiers", err),
            };
            
            parse_result
        })
    }

    /// Parse ws with memoization
    #[inline]
    fn parse_ws(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_WS, |parser| {
            parser.debug_enter_rule("ws");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let result = ParseContent::Terminal(parser.match_regex_optimized(r#"\\s+"#)?);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "ws",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("ws", start_pos),
                Err(err) => parser.debug_exit_fail("ws", err),
            };
            
            parse_result
        })
    }

    /// Parse class_range with memoization
    #[inline]
    fn parse_class_range(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_CLASS_RANGE, |parser| {
            parser.debug_enter_rule("class_range");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let mut sequence_elements = Vec::with_capacity(3);
        {
            parser.debug_sequence_element(0, 3, "element_0");
            let element_start = parser.position;
    let element_content = ParseContent::Alternative(Box::new(parser.parse_class_atom()?));
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_0",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(1, 3, "element_1");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string(r#"-"#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_1",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(2, 3, "element_2");
            let element_start = parser.position;
    let element_content = ParseContent::Alternative(Box::new(parser.parse_class_atom()?));
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_2",
                content: element_content,
                span: element_start..element_end,
            });
        }
        let result = ParseContent::Sequence(sequence_elements);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "class_range",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("class_range", start_pos),
                Err(err) => parser.debug_exit_fail("class_range", err),
            };
            
            parse_result
        })
    }

    /// Parse code_block with memoization
    #[inline]
    fn parse_code_block(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_CODE_BLOCK, |parser| {
            parser.debug_enter_rule("code_block");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let result = if false {
            ParseContent::Terminal("unreachable")
        } else {
            parser.debug_try_alternative(0, 2);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_code_block_plain()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_0",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(1, 2);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_code_block_lang()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_1",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
            } else {
                ParseContent::Terminal("no_alternative_matched")
            }
        };

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "code_block",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("code_block", start_pos),
                Err(err) => parser.debug_exit_fail("code_block", err),
            };
            
            parse_result
        })
    }

    /// Parse code_string_single_char with memoization
    #[inline]
    fn parse_code_string_single_char(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_CODE_STRING_SINGLE_CHAR, |parser| {
            parser.debug_enter_rule("code_string_single_char");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let result = if false {
            ParseContent::Terminal("unreachable")
        } else {
            parser.debug_try_alternative(0, 2);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_code_string_escape()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_0",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(1, 2);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_regex_optimized(r#"[^'\\\\]"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_1",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
            } else {
                ParseContent::Terminal("no_alternative_matched")
            }
        };

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "code_string_single_char",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("code_string_single_char", start_pos),
                Err(err) => parser.debug_exit_fail("code_string_single_char", err),
            };
            
            parse_result
        })
    }

    /// Parse char_class with memoization
    #[inline]
    fn parse_char_class(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_CHAR_CLASS, |parser| {
            parser.debug_enter_rule("char_class");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let mut sequence_elements = Vec::with_capacity(3);
        {
            parser.debug_sequence_element(0, 3, "element_0");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string(r#"["#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_0",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(1, 3, "element_1");
            let element_start = parser.position;
    let element_content = ParseContent::Alternative(Box::new(parser.parse_class_body()?));
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_1",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(2, 3, "element_2");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string(r#"]"#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_2",
                content: element_content,
                span: element_start..element_end,
            });
        }
        let result = ParseContent::Sequence(sequence_elements);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "char_class",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("char_class", start_pos),
                Err(err) => parser.debug_exit_fail("char_class", err),
            };
            
            parse_result
        })
    }

    /// Parse code_string_double_char with memoization
    #[inline]
    fn parse_code_string_double_char(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_CODE_STRING_DOUBLE_CHAR, |parser| {
            parser.debug_enter_rule("code_string_double_char");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let result = if false {
            ParseContent::Terminal("unreachable")
        } else {
            parser.debug_try_alternative(0, 2);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_code_string_escape()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_0",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(1, 2);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_regex_optimized(r#"[^\"\\\\]"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_1",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
            } else {
                ParseContent::Terminal("no_alternative_matched")
            }
        };

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "code_string_double_char",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("code_string_double_char", start_pos),
                Err(err) => parser.debug_exit_fail("code_string_double_char", err),
            };
            
            parse_result
        })
    }

    /// Parse property_escape with memoization
    #[inline]
    fn parse_property_escape(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_PROPERTY_ESCAPE, |parser| {
            parser.debug_enter_rule("property_escape");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let result = if false {
            ParseContent::Terminal("unreachable")
        } else {
            parser.debug_try_alternative(0, 2);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
let mut sequence_elements = Vec::with_capacity(3);
{
    p.debug_sequence_element(0, 3, "element_0");
    let element_start = p.position;
    let element_content = ParseContent::Terminal(p.match_string(r#"p{"#)?);
    let element_end = p.position;
    sequence_elements.push(ParseNode {
        rule_name: "element_0",
        content: element_content,
        span: element_start..element_end,
    });
}
{
    p.debug_sequence_element(1, 3, "element_1");
    let element_start = p.position;
    let element_content = ParseContent::Alternative(Box::new(p.parse_prop_name()?));
    let element_end = p.position;
    sequence_elements.push(ParseNode {
        rule_name: "element_1",
        content: element_content,
        span: element_start..element_end,
    });
}
{
    p.debug_sequence_element(2, 3, "element_2");
    let element_start = p.position;
    let element_content = ParseContent::Terminal(p.match_string(r#"}"#)?);
    let element_end = p.position;
    sequence_elements.push(ParseNode {
        rule_name: "element_2",
        content: element_content,
        span: element_start..element_end,
    });
}
        let content = ParseContent::Sequence(sequence_elements);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_0",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(1, 2);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
let mut sequence_elements = Vec::with_capacity(3);
{
    p.debug_sequence_element(0, 3, "element_0");
    let element_start = p.position;
    let element_content = ParseContent::Terminal(p.match_string(r#"P{"#)?);
    let element_end = p.position;
    sequence_elements.push(ParseNode {
        rule_name: "element_0",
        content: element_content,
        span: element_start..element_end,
    });
}
{
    p.debug_sequence_element(1, 3, "element_1");
    let element_start = p.position;
    let element_content = ParseContent::Alternative(Box::new(p.parse_prop_name()?));
    let element_end = p.position;
    sequence_elements.push(ParseNode {
        rule_name: "element_1",
        content: element_content,
        span: element_start..element_end,
    });
}
{
    p.debug_sequence_element(2, 3, "element_2");
    let element_start = p.position;
    let element_content = ParseContent::Terminal(p.match_string(r#"}"#)?);
    let element_end = p.position;
    sequence_elements.push(ParseNode {
        rule_name: "element_2",
        content: element_content,
        span: element_start..element_end,
    });
}
        let content = ParseContent::Sequence(sequence_elements);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_1",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
            } else {
                ParseContent::Terminal("no_alternative_matched")
            }
        };

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "property_escape",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("property_escape", start_pos),
                Err(err) => parser.debug_exit_fail("property_escape", err),
            };
            
            parse_result
        })
    }

    /// Parse alternation with memoization
    #[inline]
    fn parse_alternation(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_ALTERNATION, |parser| {
            parser.debug_enter_rule("alternation");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let mut sequence_elements = Vec::with_capacity(2);
        {
            parser.debug_sequence_element(0, 2, "element_0");
            let element_start = parser.position;
    let element_content = ParseContent::Alternative(Box::new(parser.parse_concatenation()?));
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_0",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(1, 2, "element_1");
            let element_start = parser.position;
    let element_content = parser.parse_quantified_optimized("*", |p| {
let mut sequence_elements = Vec::with_capacity(2);
{
    p.debug_sequence_element(0, 2, "element_0");
    let element_start = p.position;
    let element_content = ParseContent::Terminal(p.match_string(r#"|"#)?);
    let element_end = p.position;
    sequence_elements.push(ParseNode {
        rule_name: "element_0",
        content: element_content,
        span: element_start..element_end,
    });
}
{
    p.debug_sequence_element(1, 2, "element_1");
    let element_start = p.position;
    let element_content = ParseContent::Alternative(Box::new(p.parse_concatenation()?));
    let element_end = p.position;
    sequence_elements.push(ParseNode {
        rule_name: "element_1",
        content: element_content,
        span: element_start..element_end,
    });
}
    let content = ParseContent::Sequence(sequence_elements);
    Ok(content)
})?;
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_1",
                content: element_content,
                span: element_start..element_end,
            });
        }
        let result = ParseContent::Sequence(sequence_elements);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "alternation",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("alternation", start_pos),
                Err(err) => parser.debug_exit_fail("alternation", err),
            };
            
            parse_result
        })
    }

    /// Parse unicode_escape with memoization
    #[inline]
    fn parse_unicode_escape(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_UNICODE_ESCAPE, |parser| {
            parser.debug_enter_rule("unicode_escape");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let mut sequence_elements = Vec::with_capacity(3);
        {
            parser.debug_sequence_element(0, 3, "element_0");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string(r#"u{"#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_0",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(1, 3, "element_1");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_regex_optimized(r#"[0-9A-Fa-f]+"#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_1",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(2, 3, "element_2");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string(r#"}"#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_2",
                content: element_content,
                span: element_start..element_end,
            });
        }
        let result = ParseContent::Sequence(sequence_elements);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "unicode_escape",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("unicode_escape", start_pos),
                Err(err) => parser.debug_exit_fail("unicode_escape", err),
            };
            
            parse_result
        })
    }

    /// Parse atom with memoization
    #[inline]
    fn parse_atom(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_ATOM, |parser| {
            parser.debug_enter_rule("atom");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let result = if false {
            ParseContent::Terminal("unreachable")
        } else {
            parser.debug_try_alternative(0, 13);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_literal()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_0",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(1, 13);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_dot()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_1",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(2, 13);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_anchor()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_2",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(3, 13);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_escape()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_3",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(4, 13);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_backreference()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_4",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(5, 13);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_char_class()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_5",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(6, 13);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_group()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_6",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(7, 13);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_atomic_group()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_7",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(8, 13);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_lookaround()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_8",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(9, 13);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_inline_modifiers()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_9",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(10, 13);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_scoped_inline_modifiers()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_10",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(11, 13);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_conditional()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_11",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(12, 13);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_code_block()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_12",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
            } else {
                ParseContent::Terminal("no_alternative_matched")
            }
        };

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "atom",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("atom", start_pos),
                Err(err) => parser.debug_exit_fail("atom", err),
            };
            
            parse_result
        })
    }

    /// Parse name with memoization
    #[inline]
    fn parse_name(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_NAME, |parser| {
            parser.debug_enter_rule("name");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let mut sequence_elements = Vec::with_capacity(2);
        {
            parser.debug_sequence_element(0, 2, "element_0");
            let element_start = parser.position;
    let element_content = ParseContent::Alternative(Box::new(parser.parse_name_start()?));
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_0",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(1, 2, "element_1");
            let element_start = parser.position;
    let element_content = parser.parse_quantified_optimized("*", |p| {
    let content = ParseContent::Alternative(Box::new(p.parse_name_continue()?));
    Ok(content)
})?;
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_1",
                content: element_content,
                span: element_start..element_end,
            });
        }
        let result = ParseContent::Sequence(sequence_elements);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "name",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("name", start_pos),
                Err(err) => parser.debug_exit_fail("name", err),
            };
            
            parse_result
        })
    }

    /// Parse pattern with memoization
    #[inline]
    fn parse_pattern(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_PATTERN, |parser| {
            parser.debug_enter_rule("pattern");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let result = ParseContent::Alternative(Box::new(parser.parse_alternation()?));

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "pattern",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("pattern", start_pos),
                Err(err) => parser.debug_exit_fail("pattern", err),
            };
            
            parse_result
        })
    }

    /// Parse scoped_inline_modifiers with memoization
    #[inline]
    fn parse_scoped_inline_modifiers(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_SCOPED_INLINE_MODIFIERS, |parser| {
            parser.debug_enter_rule("scoped_inline_modifiers");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let mut sequence_elements = Vec::with_capacity(5);
        {
            parser.debug_sequence_element(0, 5, "element_0");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string(r#"(?("#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_0",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(1, 5, "element_1");
            let element_start = parser.position;
    let element_content = ParseContent::Alternative(Box::new(parser.parse_modifier_seq()?));
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_1",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(2, 5, "element_2");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string(r#":"#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_2",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(3, 5, "element_3");
            let element_start = parser.position;
    let element_content = ParseContent::Alternative(Box::new(parser.parse_pattern()?));
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_3",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(4, 5, "element_4");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string(r#")"#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_4",
                content: element_content,
                span: element_start..element_end,
            });
        }
        let result = ParseContent::Sequence(sequence_elements);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "scoped_inline_modifiers",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("scoped_inline_modifiers", start_pos),
                Err(err) => parser.debug_exit_fail("scoped_inline_modifiers", err),
            };
            
            parse_result
        })
    }

    /// Parse code_element with memoization
    #[inline]
    fn parse_code_element(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_CODE_ELEMENT, |parser| {
            parser.debug_enter_rule("code_element");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let result = if false {
            ParseContent::Terminal("unreachable")
        } else {
            parser.debug_try_alternative(0, 5);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_code_string_double()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_0",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(1, 5);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_code_string_single()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_1",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(2, 5);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_code_balanced_braces()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_2",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(3, 5);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_code_escaped_char()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_3",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(4, 5);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_code_regular_char()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_4",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
            } else {
                ParseContent::Terminal("no_alternative_matched")
            }
        };

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "code_element",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("code_element", start_pos),
                Err(err) => parser.debug_exit_fail("code_element", err),
            };
            
            parse_result
        })
    }

    /// Parse lookahead_neg with memoization
    #[inline]
    fn parse_lookahead_neg(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_LOOKAHEAD_NEG, |parser| {
            parser.debug_enter_rule("lookahead_neg");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let mut sequence_elements = Vec::with_capacity(3);
        {
            parser.debug_sequence_element(0, 3, "element_0");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string(r#"(?!"#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_0",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(1, 3, "element_1");
            let element_start = parser.position;
    let element_content = ParseContent::Alternative(Box::new(parser.parse_pattern()?));
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_1",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(2, 3, "element_2");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string(r#")"#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_2",
                content: element_content,
                span: element_start..element_end,
            });
        }
        let result = ParseContent::Sequence(sequence_elements);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "lookahead_neg",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("lookahead_neg", start_pos),
                Err(err) => parser.debug_exit_fail("lookahead_neg", err),
            };
            
            parse_result
        })
    }

    /// Parse quant_suffix with memoization
    #[inline]
    fn parse_quant_suffix(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_QUANT_SUFFIX, |parser| {
            parser.debug_enter_rule("quant_suffix");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let result = if false {
            ParseContent::Terminal("unreachable")
        } else {
            parser.debug_try_alternative(0, 2);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_string(r#"?"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_0",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(1, 2);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_string(r#"+"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_1",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
            } else {
                ParseContent::Terminal("no_alternative_matched")
            }
        };

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "quant_suffix",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("quant_suffix", start_pos),
                Err(err) => parser.debug_exit_fail("quant_suffix", err),
            };
            
            parse_result
        })
    }

    /// Parse noncapturing_group with memoization
    #[inline]
    fn parse_noncapturing_group(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_NONCAPTURING_GROUP, |parser| {
            parser.debug_enter_rule("noncapturing_group");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let mut sequence_elements = Vec::with_capacity(3);
        {
            parser.debug_sequence_element(0, 3, "element_0");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string(r#"(?:"#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_0",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(1, 3, "element_1");
            let element_start = parser.position;
    let element_content = ParseContent::Alternative(Box::new(parser.parse_pattern()?));
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_1",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(2, 3, "element_2");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string(r#")"#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_2",
                content: element_content,
                span: element_start..element_end,
            });
        }
        let result = ParseContent::Sequence(sequence_elements);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "noncapturing_group",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("noncapturing_group", start_pos),
                Err(err) => parser.debug_exit_fail("noncapturing_group", err),
            };
            
            parse_result
        })
    }

    /// Parse code_string_double with memoization
    #[inline]
    fn parse_code_string_double(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_CODE_STRING_DOUBLE, |parser| {
            parser.debug_enter_rule("code_string_double");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let mut sequence_elements = Vec::with_capacity(3);
        {
            parser.debug_sequence_element(0, 3, "element_0");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string("")?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_0",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(1, 3, "element_1");
            let element_start = parser.position;
    let element_content = parser.parse_quantified_optimized("*", |p| {
    let content = ParseContent::Alternative(Box::new(p.parse_code_string_double_char()?));
    Ok(content)
})?;
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_1",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(2, 3, "element_2");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string("")?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_2",
                content: element_content,
                span: element_start..element_end,
            });
        }
        let result = ParseContent::Sequence(sequence_elements);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "code_string_double",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("code_string_double", start_pos),
                Err(err) => parser.debug_exit_fail("code_string_double", err),
            };
            
            parse_result
        })
    }

    /// Parse posix_class with memoization
    #[inline]
    fn parse_posix_class(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_POSIX_CLASS, |parser| {
            parser.debug_enter_rule("posix_class");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let mut sequence_elements = Vec::with_capacity(3);
        {
            parser.debug_sequence_element(0, 3, "element_0");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string(r#"[:"#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_0",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(1, 3, "element_1");
            let element_start = parser.position;
    let element_content = ParseContent::Alternative(Box::new(parser.parse_posix_name()?));
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_1",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(2, 3, "element_2");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string(r#":]"#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_2",
                content: element_content,
                span: element_start..element_end,
            });
        }
        let result = ParseContent::Sequence(sequence_elements);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "posix_class",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("posix_class", start_pos),
                Err(err) => parser.debug_exit_fail("posix_class", err),
            };
            
            parse_result
        })
    }

    /// Parse lookbehind_neg with memoization
    #[inline]
    fn parse_lookbehind_neg(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_LOOKBEHIND_NEG, |parser| {
            parser.debug_enter_rule("lookbehind_neg");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let mut sequence_elements = Vec::with_capacity(3);
        {
            parser.debug_sequence_element(0, 3, "element_0");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string(r#"(?<!"#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_0",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(1, 3, "element_1");
            let element_start = parser.position;
    let element_content = ParseContent::Alternative(Box::new(parser.parse_pattern()?));
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_1",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(2, 3, "element_2");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string(r#")"#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_2",
                content: element_content,
                span: element_start..element_end,
            });
        }
        let result = ParseContent::Sequence(sequence_elements);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "lookbehind_neg",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("lookbehind_neg", start_pos),
                Err(err) => parser.debug_exit_fail("lookbehind_neg", err),
            };
            
            parse_result
        })
    }

    /// Parse backreference with memoization
    #[inline]
    fn parse_backreference(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_BACKREFERENCE, |parser| {
            parser.debug_enter_rule("backreference");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let result = if false {
            ParseContent::Terminal("unreachable")
        } else {
            parser.debug_try_alternative(0, 2);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
let mut sequence_elements = Vec::with_capacity(2);
{
    p.debug_sequence_element(0, 2, "element_0");
    let element_start = p.position;
    let element_content = ParseContent::Terminal(p.match_string(r#"\\\\"#)?);
    let element_end = p.position;
    sequence_elements.push(ParseNode {
        rule_name: "element_0",
        content: element_content,
        span: element_start..element_end,
    });
}
{
    p.debug_sequence_element(1, 2, "element_1");
    let element_start = p.position;
    let element_content = ParseContent::Alternative(Box::new(p.parse_digits()?));
    let element_end = p.position;
    sequence_elements.push(ParseNode {
        rule_name: "element_1",
        content: element_content,
        span: element_start..element_end,
    });
}
        let content = ParseContent::Sequence(sequence_elements);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_0",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(1, 2);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
let mut sequence_elements = Vec::with_capacity(2);
{
    p.debug_sequence_element(0, 2, "element_0");
    let element_start = p.position;
    let element_content = ParseContent::Terminal(p.match_string(r#"\\\\k"#)?);
    let element_end = p.position;
    sequence_elements.push(ParseNode {
        rule_name: "element_0",
        content: element_content,
        span: element_start..element_end,
    });
}
{
    p.debug_sequence_element(1, 2, "element_1");
    let element_start = p.position;
    let element_content = ParseContent::Alternative(Box::new(p.parse_name_ref()?));
    let element_end = p.position;
    sequence_elements.push(ParseNode {
        rule_name: "element_1",
        content: element_content,
        span: element_start..element_end,
    });
}
        let content = ParseContent::Sequence(sequence_elements);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_1",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
            } else {
                ParseContent::Terminal("no_alternative_matched")
            }
        };

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "backreference",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("backreference", start_pos),
                Err(err) => parser.debug_exit_fail("backreference", err),
            };
            
            parse_result
        })
    }

    /// Parse escape with memoization
    #[inline]
    fn parse_escape(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_ESCAPE, |parser| {
            parser.debug_enter_rule("escape");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let mut sequence_elements = Vec::with_capacity(2);
        {
            parser.debug_sequence_element(0, 2, "element_0");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string(r#"\\\\"#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_0",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(1, 2, "element_1");
            let element_start = parser.position;
    let element_content = ParseContent::Alternative(Box::new(parser.parse_escape_unit()?));
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_1",
                content: element_content,
                span: element_start..element_end,
            });
        }
        let result = ParseContent::Sequence(sequence_elements);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "escape",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("escape", start_pos),
                Err(err) => parser.debug_exit_fail("escape", err),
            };
            
            parse_result
        })
    }

    /// Parse literal with memoization
    #[inline]
    fn parse_literal(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_LITERAL, |parser| {
            parser.debug_enter_rule("literal");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let mut sequence_elements = Vec::with_capacity(1);
        {
            parser.debug_sequence_element(0, 1, "element_0");
            let element_start = parser.position;
    let element_content = parser.parse_quantified_optimized("+", |p| {
    let content = ParseContent::Alternative(Box::new(p.parse_literal_char()?));
    Ok(content)
})?;
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_0",
                content: element_content,
                span: element_start..element_end,
            });
        }
        let result = ParseContent::Sequence(sequence_elements);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "literal",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("literal", start_pos),
                Err(err) => parser.debug_exit_fail("literal", err),
            };
            
            parse_result
        })
    }

    /// Parse name_continue with memoization
    #[inline]
    fn parse_name_continue(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_NAME_CONTINUE, |parser| {
            parser.debug_enter_rule("name_continue");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let result = ParseContent::Terminal(parser.match_regex_optimized(r#"[A-Za-z0-9_]"#)?);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "name_continue",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("name_continue", start_pos),
                Err(err) => parser.debug_exit_fail("name_continue", err),
            };
            
            parse_result
        })
    }

    /// Parse group with memoization
    #[inline]
    fn parse_group(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_GROUP, |parser| {
            parser.debug_enter_rule("group");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let result = if false {
            ParseContent::Terminal("unreachable")
        } else {
            parser.debug_try_alternative(0, 3);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_capturing_group()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_0",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(1, 3);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_noncapturing_group()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_1",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(2, 3);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_named_group()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_2",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
            } else {
                ParseContent::Terminal("no_alternative_matched")
            }
        };

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "group",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("group", start_pos),
                Err(err) => parser.debug_exit_fail("group", err),
            };
            
            parse_result
        })
    }

    /// Parse name_ref with memoization
    #[inline]
    fn parse_name_ref(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_NAME_REF, |parser| {
            parser.debug_enter_rule("name_ref");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let result = if false {
            ParseContent::Terminal("unreachable")
        } else {
            parser.debug_try_alternative(0, 2);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
let mut sequence_elements = Vec::with_capacity(3);
{
    p.debug_sequence_element(0, 3, "element_0");
    let element_start = p.position;
    let element_content = ParseContent::Terminal(p.match_string(r#"<"#)?);
    let element_end = p.position;
    sequence_elements.push(ParseNode {
        rule_name: "element_0",
        content: element_content,
        span: element_start..element_end,
    });
}
{
    p.debug_sequence_element(1, 3, "element_1");
    let element_start = p.position;
    let element_content = ParseContent::Alternative(Box::new(p.parse_name()?));
    let element_end = p.position;
    sequence_elements.push(ParseNode {
        rule_name: "element_1",
        content: element_content,
        span: element_start..element_end,
    });
}
{
    p.debug_sequence_element(2, 3, "element_2");
    let element_start = p.position;
    let element_content = ParseContent::Terminal(p.match_string(r#">"#)?);
    let element_end = p.position;
    sequence_elements.push(ParseNode {
        rule_name: "element_2",
        content: element_content,
        span: element_start..element_end,
    });
}
        let content = ParseContent::Sequence(sequence_elements);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_0",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(1, 2);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
let mut sequence_elements = Vec::with_capacity(3);
{
    p.debug_sequence_element(0, 3, "element_0");
    let element_start = p.position;
    let element_content = ParseContent::Terminal(p.match_string("")?);
    let element_end = p.position;
    sequence_elements.push(ParseNode {
        rule_name: "element_0",
        content: element_content,
        span: element_start..element_end,
    });
}
{
    p.debug_sequence_element(1, 3, "element_1");
    let element_start = p.position;
    let element_content = ParseContent::Alternative(Box::new(p.parse_name()?));
    let element_end = p.position;
    sequence_elements.push(ParseNode {
        rule_name: "element_1",
        content: element_content,
        span: element_start..element_end,
    });
}
{
    p.debug_sequence_element(2, 3, "element_2");
    let element_start = p.position;
    let element_content = ParseContent::Terminal(p.match_string("")?);
    let element_end = p.position;
    sequence_elements.push(ParseNode {
        rule_name: "element_2",
        content: element_content,
        span: element_start..element_end,
    });
}
        let content = ParseContent::Sequence(sequence_elements);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_1",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
            } else {
                ParseContent::Terminal("no_alternative_matched")
            }
        };

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "name_ref",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("name_ref", start_pos),
                Err(err) => parser.debug_exit_fail("name_ref", err),
            };
            
            parse_result
        })
    }

    /// Parse code_content with memoization
    #[inline]
    fn parse_code_content(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_CODE_CONTENT, |parser| {
            parser.debug_enter_rule("code_content");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let mut sequence_elements = Vec::with_capacity(1);
        {
            parser.debug_sequence_element(0, 1, "element_0");
            let element_start = parser.position;
    let element_content = parser.parse_quantified_optimized("*", |p| {
    let content = ParseContent::Alternative(Box::new(p.parse_code_element()?));
    Ok(content)
})?;
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_0",
                content: element_content,
                span: element_start..element_end,
            });
        }
        let result = ParseContent::Sequence(sequence_elements);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "code_content",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("code_content", start_pos),
                Err(err) => parser.debug_exit_fail("code_content", err),
            };
            
            parse_result
        })
    }

    /// Parse modifier_char with memoization
    #[inline]
    fn parse_modifier_char(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_MODIFIER_CHAR, |parser| {
            parser.debug_enter_rule("modifier_char");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let result = if false {
            ParseContent::Terminal("unreachable")
        } else {
            parser.debug_try_alternative(0, 14);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_string(r#"i"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_0",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(1, 14);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_string(r#"m"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_1",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(2, 14);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_string(r#"s"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_2",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(3, 14);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_string(r#"x"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_3",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(4, 14);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_string(r#"U"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_4",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(5, 14);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_string(r#"J"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_5",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(6, 14);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_string(r#"u"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_6",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(7, 14);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_string(r#"a"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_7",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(8, 14);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_string(r#"d"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_8",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(9, 14);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_string(r#"S"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_9",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(10, 14);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_string(r#"A"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_10",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(11, 14);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_string(r#"X"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_11",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(12, 14);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_string(r#"R"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_12",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(13, 14);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_string(r#"n"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_13",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
            } else {
                ParseContent::Terminal("no_alternative_matched")
            }
        };

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "modifier_char",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("modifier_char", start_pos),
                Err(err) => parser.debug_exit_fail("modifier_char", err),
            };
            
            parse_result
        })
    }

    /// Parse code_block_lang with memoization
    #[inline]
    fn parse_code_block_lang(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_CODE_BLOCK_LANG, |parser| {
            parser.debug_enter_rule("code_block_lang");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let mut sequence_elements = Vec::with_capacity(6);
        {
            parser.debug_sequence_element(0, 6, "element_0");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string(r#"(?{"#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_0",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(1, 6, "element_1");
            let element_start = parser.position;
    let element_content = ParseContent::Alternative(Box::new(parser.parse_code_lang()?));
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_1",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(2, 6, "element_2");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string(r#":"#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_2",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(3, 6, "element_3");
            let element_start = parser.position;
    let element_content = parser.parse_quantified_optimized("?", |p| {
    let content = ParseContent::Alternative(Box::new(p.parse_ws()?));
    Ok(content)
})?;
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_3",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(4, 6, "element_4");
            let element_start = parser.position;
    let element_content = ParseContent::Alternative(Box::new(parser.parse_code_content()?));
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_4",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(5, 6, "element_5");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string(r#"})"#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_5",
                content: element_content,
                span: element_start..element_end,
            });
        }
        let result = ParseContent::Sequence(sequence_elements);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "code_block_lang",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("code_block_lang", start_pos),
                Err(err) => parser.debug_exit_fail("code_block_lang", err),
            };
            
            parse_result
        })
    }

    /// Parse counted_quantifier with memoization
    #[inline]
    fn parse_counted_quantifier(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_COUNTED_QUANTIFIER, |parser| {
            parser.debug_enter_rule("counted_quantifier");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let mut sequence_elements = Vec::with_capacity(7);
        {
            parser.debug_sequence_element(0, 7, "element_0");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string(r#"{"#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_0",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(1, 7, "element_1");
            let element_start = parser.position;
    let element_content = parser.parse_quantified_optimized("?", |p| {
    let content = ParseContent::Alternative(Box::new(p.parse_ws()?));
    Ok(content)
})?;
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_1",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(2, 7, "element_2");
            let element_start = parser.position;
    let element_content = ParseContent::Alternative(Box::new(parser.parse_digits()?));
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_2",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(3, 7, "element_3");
            let element_start = parser.position;
    let element_content = parser.parse_quantified_optimized("?", |p| {
    let content = ParseContent::Alternative(Box::new(p.parse_ws()?));
    Ok(content)
})?;
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_3",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(4, 7, "element_4");
            let element_start = parser.position;
    let element_content = parser.parse_quantified_optimized("?", |p| {
let mut sequence_elements = Vec::with_capacity(5);
{
    p.debug_sequence_element(0, 5, "element_0");
    let element_start = p.position;
    let element_content = ParseContent::Terminal(p.match_string(r#","#)?);
    let element_end = p.position;
    sequence_elements.push(ParseNode {
        rule_name: "element_0",
        content: element_content,
        span: element_start..element_end,
    });
}
{
    p.debug_sequence_element(1, 5, "element_1");
    let element_start = p.position;
    let element_content = ParseContent::Alternative(Box::new(p.parse_ws()?));
    let element_end = p.position;
    sequence_elements.push(ParseNode {
        rule_name: "element_1",
        content: element_content,
        span: element_start..element_end,
    });
}
{
    p.debug_sequence_element(2, 5, "element_2");
    let element_start = p.position;
    let element_content = ParseContent::Terminal(r#"?"#);
    let element_end = p.position;
    sequence_elements.push(ParseNode {
        rule_name: "element_2",
        content: element_content,
        span: element_start..element_end,
    });
}
{
    p.debug_sequence_element(3, 5, "element_3");
    let element_start = p.position;
    let element_content = ParseContent::Alternative(Box::new(p.parse_digits()?));
    let element_end = p.position;
    sequence_elements.push(ParseNode {
        rule_name: "element_3",
        content: element_content,
        span: element_start..element_end,
    });
}
{
    p.debug_sequence_element(4, 5, "element_4");
    let element_start = p.position;
    let element_content = ParseContent::Terminal(r#"?"#);
    let element_end = p.position;
    sequence_elements.push(ParseNode {
        rule_name: "element_4",
        content: element_content,
        span: element_start..element_end,
    });
}
    let content = ParseContent::Sequence(sequence_elements);
    Ok(content)
})?;
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_4",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(5, 7, "element_5");
            let element_start = parser.position;
    let element_content = parser.parse_quantified_optimized("?", |p| {
    let content = ParseContent::Alternative(Box::new(p.parse_ws()?));
    Ok(content)
})?;
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_5",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(6, 7, "element_6");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string(r#"}"#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_6",
                content: element_content,
                span: element_start..element_end,
            });
        }
        let result = ParseContent::Sequence(sequence_elements);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "counted_quantifier",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("counted_quantifier", start_pos),
                Err(err) => parser.debug_exit_fail("counted_quantifier", err),
            };
            
            parse_result
        })
    }

    /// Parse class_escape with memoization
    #[inline]
    fn parse_class_escape(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_CLASS_ESCAPE, |parser| {
            parser.debug_enter_rule("class_escape");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let result = ParseContent::Alternative(Box::new(parser.parse_escape()?));

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "class_escape",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("class_escape", start_pos),
                Err(err) => parser.debug_exit_fail("class_escape", err),
            };
            
            parse_result
        })
    }

    /// Parse class_body with memoization
    #[inline]
    fn parse_class_body(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_CLASS_BODY, |parser| {
            parser.debug_enter_rule("class_body");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let mut sequence_elements = Vec::with_capacity(1);
        {
            parser.debug_sequence_element(0, 1, "element_0");
            let element_start = parser.position;
    let element_content = parser.parse_quantified_optimized("*", |p| {
    let content = ParseContent::Alternative(Box::new(p.parse_class_item()?));
    Ok(content)
})?;
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_0",
                content: element_content,
                span: element_start..element_end,
            });
        }
        let result = ParseContent::Sequence(sequence_elements);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "class_body",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("class_body", start_pos),
                Err(err) => parser.debug_exit_fail("class_body", err),
            };
            
            parse_result
        })
    }

    /// Parse control_escape with memoization
    #[inline]
    fn parse_control_escape(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_CONTROL_ESCAPE, |parser| {
            parser.debug_enter_rule("control_escape");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let mut sequence_elements = Vec::with_capacity(2);
        {
            parser.debug_sequence_element(0, 2, "element_0");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string(r#"c"#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_0",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(1, 2, "element_1");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_regex_optimized(r#"."#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_1",
                content: element_content,
                span: element_start..element_end,
            });
        }
        let result = ParseContent::Sequence(sequence_elements);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "control_escape",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("control_escape", start_pos),
                Err(err) => parser.debug_exit_fail("control_escape", err),
            };
            
            parse_result
        })
    }

    /// Parse digits with memoization
    #[inline]
    fn parse_digits(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_DIGITS, |parser| {
            parser.debug_enter_rule("digits");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let result = ParseContent::Terminal(parser.match_regex_optimized(r#"[0-9]+"#)?);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "digits",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("digits", start_pos),
                Err(err) => parser.debug_exit_fail("digits", err),
            };
            
            parse_result
        })
    }

    /// Parse modifier_group with memoization
    #[inline]
    fn parse_modifier_group(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_MODIFIER_GROUP, |parser| {
            parser.debug_enter_rule("modifier_group");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let mut sequence_elements = Vec::with_capacity(1);
        {
            parser.debug_sequence_element(0, 1, "element_0");
            let element_start = parser.position;
    let element_content = parser.parse_quantified_optimized("+", |p| {
    let content = ParseContent::Alternative(Box::new(p.parse_modifier_char()?));
    Ok(content)
})?;
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_0",
                content: element_content,
                span: element_start..element_end,
            });
        }
        let result = ParseContent::Sequence(sequence_elements);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "modifier_group",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("modifier_group", start_pos),
                Err(err) => parser.debug_exit_fail("modifier_group", err),
            };
            
            parse_result
        })
    }

    /// Parse code_regular_char with memoization
    #[inline]
    fn parse_code_regular_char(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_CODE_REGULAR_CHAR, |parser| {
            parser.debug_enter_rule("code_regular_char");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let result = ParseContent::Terminal(parser.match_regex_optimized(r#"[^{}'\\\"\\\\]"#)?);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "code_regular_char",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("code_regular_char", start_pos),
                Err(err) => parser.debug_exit_fail("code_regular_char", err),
            };
            
            parse_result
        })
    }

    /// Parse named_group with memoization
    #[inline]
    fn parse_named_group(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_NAMED_GROUP, |parser| {
            parser.debug_enter_rule("named_group");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let result = if false {
            ParseContent::Terminal("unreachable")
        } else {
            parser.debug_try_alternative(0, 2);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
let mut sequence_elements = Vec::with_capacity(5);
{
    p.debug_sequence_element(0, 5, "element_0");
    let element_start = p.position;
    let element_content = ParseContent::Terminal(p.match_string(r#"(?<"#)?);
    let element_end = p.position;
    sequence_elements.push(ParseNode {
        rule_name: "element_0",
        content: element_content,
        span: element_start..element_end,
    });
}
{
    p.debug_sequence_element(1, 5, "element_1");
    let element_start = p.position;
    let element_content = ParseContent::Alternative(Box::new(p.parse_name()?));
    let element_end = p.position;
    sequence_elements.push(ParseNode {
        rule_name: "element_1",
        content: element_content,
        span: element_start..element_end,
    });
}
{
    p.debug_sequence_element(2, 5, "element_2");
    let element_start = p.position;
    let element_content = ParseContent::Terminal(p.match_string(r#">"#)?);
    let element_end = p.position;
    sequence_elements.push(ParseNode {
        rule_name: "element_2",
        content: element_content,
        span: element_start..element_end,
    });
}
{
    p.debug_sequence_element(3, 5, "element_3");
    let element_start = p.position;
    let element_content = ParseContent::Alternative(Box::new(p.parse_pattern()?));
    let element_end = p.position;
    sequence_elements.push(ParseNode {
        rule_name: "element_3",
        content: element_content,
        span: element_start..element_end,
    });
}
{
    p.debug_sequence_element(4, 5, "element_4");
    let element_start = p.position;
    let element_content = ParseContent::Terminal(p.match_string(r#")"#)?);
    let element_end = p.position;
    sequence_elements.push(ParseNode {
        rule_name: "element_4",
        content: element_content,
        span: element_start..element_end,
    });
}
        let content = ParseContent::Sequence(sequence_elements);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_0",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(1, 2);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
let mut sequence_elements = Vec::with_capacity(5);
{
    p.debug_sequence_element(0, 5, "element_0");
    let element_start = p.position;
    let element_content = ParseContent::Terminal(p.match_string(r#"(?"#)?);
    let element_end = p.position;
    sequence_elements.push(ParseNode {
        rule_name: "element_0",
        content: element_content,
        span: element_start..element_end,
    });
}
{
    p.debug_sequence_element(1, 5, "element_1");
    let element_start = p.position;
    let element_content = ParseContent::Alternative(Box::new(p.parse_name()?));
    let element_end = p.position;
    sequence_elements.push(ParseNode {
        rule_name: "element_1",
        content: element_content,
        span: element_start..element_end,
    });
}
{
    p.debug_sequence_element(2, 5, "element_2");
    let element_start = p.position;
    let element_content = ParseContent::Terminal(p.match_string("")?);
    let element_end = p.position;
    sequence_elements.push(ParseNode {
        rule_name: "element_2",
        content: element_content,
        span: element_start..element_end,
    });
}
{
    p.debug_sequence_element(3, 5, "element_3");
    let element_start = p.position;
    let element_content = ParseContent::Alternative(Box::new(p.parse_pattern()?));
    let element_end = p.position;
    sequence_elements.push(ParseNode {
        rule_name: "element_3",
        content: element_content,
        span: element_start..element_end,
    });
}
{
    p.debug_sequence_element(4, 5, "element_4");
    let element_start = p.position;
    let element_content = ParseContent::Terminal(p.match_string(r#")"#)?);
    let element_end = p.position;
    sequence_elements.push(ParseNode {
        rule_name: "element_4",
        content: element_content,
        span: element_start..element_end,
    });
}
        let content = ParseContent::Sequence(sequence_elements);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_1",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
            } else {
                ParseContent::Terminal("no_alternative_matched")
            }
        };

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "named_group",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("named_group", start_pos),
                Err(err) => parser.debug_exit_fail("named_group", err),
            };
            
            parse_result
        })
    }

    /// Parse yes_branch with memoization
    #[inline]
    fn parse_yes_branch(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_YES_BRANCH, |parser| {
            parser.debug_enter_rule("yes_branch");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let result = ParseContent::Alternative(Box::new(parser.parse_pattern()?));

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "yes_branch",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("yes_branch", start_pos),
                Err(err) => parser.debug_exit_fail("yes_branch", err),
            };
            
            parse_result
        })
    }

    /// Parse class_literal with memoization
    #[inline]
    fn parse_class_literal(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_CLASS_LITERAL, |parser| {
            parser.debug_enter_rule("class_literal");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let result = ParseContent::Terminal(parser.match_regex_optimized(r#"[^\\\\\\]]"#)?);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "class_literal",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("class_literal", start_pos),
                Err(err) => parser.debug_exit_fail("class_literal", err),
            };
            
            parse_result
        })
    }

    /// Parse posix_name with memoization
    #[inline]
    fn parse_posix_name(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_POSIX_NAME, |parser| {
            parser.debug_enter_rule("posix_name");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let result = if false {
            ParseContent::Terminal("unreachable")
        } else {
            parser.debug_try_alternative(0, 14);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_string(r#"alnum"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_0",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(1, 14);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_string(r#"alpha"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_1",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(2, 14);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_string(r#"ascii"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_2",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(3, 14);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_string(r#"blank"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_3",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(4, 14);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_string(r#"cntrl"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_4",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(5, 14);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_string(r#"digit"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_5",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(6, 14);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_string(r#"graph"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_6",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(7, 14);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_string(r#"lower"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_7",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(8, 14);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_string(r#"print"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_8",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(9, 14);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_string(r#"punct"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_9",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(10, 14);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_string(r#"space"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_10",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(11, 14);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_string(r#"upper"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_11",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(12, 14);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_string(r#"word"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_12",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(13, 14);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_string(r#"xdigit"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_13",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
            } else {
                ParseContent::Terminal("no_alternative_matched")
            }
        };

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "posix_name",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("posix_name", start_pos),
                Err(err) => parser.debug_exit_fail("posix_name", err),
            };
            
            parse_result
        })
    }

    /// Parse hex_escape with memoization
    #[inline]
    fn parse_hex_escape(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_HEX_ESCAPE, |parser| {
            parser.debug_enter_rule("hex_escape");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let result = if false {
            ParseContent::Terminal("unreachable")
        } else {
            parser.debug_try_alternative(0, 2);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
let mut sequence_elements = Vec::with_capacity(3);
{
    p.debug_sequence_element(0, 3, "element_0");
    let element_start = p.position;
    let element_content = ParseContent::Terminal(p.match_string(r#"x"#)?);
    let element_end = p.position;
    sequence_elements.push(ParseNode {
        rule_name: "element_0",
        content: element_content,
        span: element_start..element_end,
    });
}
{
    p.debug_sequence_element(1, 3, "element_1");
    let element_start = p.position;
    let element_content = ParseContent::Terminal(p.match_regex_optimized(r#"[0-9A-Fa-f]"#)?);
    let element_end = p.position;
    sequence_elements.push(ParseNode {
        rule_name: "element_1",
        content: element_content,
        span: element_start..element_end,
    });
}
{
    p.debug_sequence_element(2, 3, "element_2");
    let element_start = p.position;
    let element_content = ParseContent::Terminal(p.match_regex_optimized(r#"[0-9A-Fa-f]"#)?);
    let element_end = p.position;
    sequence_elements.push(ParseNode {
        rule_name: "element_2",
        content: element_content,
        span: element_start..element_end,
    });
}
        let content = ParseContent::Sequence(sequence_elements);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_0",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(1, 2);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
let mut sequence_elements = Vec::with_capacity(3);
{
    p.debug_sequence_element(0, 3, "element_0");
    let element_start = p.position;
    let element_content = ParseContent::Terminal(p.match_string(r#"x{"#)?);
    let element_end = p.position;
    sequence_elements.push(ParseNode {
        rule_name: "element_0",
        content: element_content,
        span: element_start..element_end,
    });
}
{
    p.debug_sequence_element(1, 3, "element_1");
    let element_start = p.position;
    let element_content = ParseContent::Terminal(p.match_regex_optimized(r#"[0-9A-Fa-f]+"#)?);
    let element_end = p.position;
    sequence_elements.push(ParseNode {
        rule_name: "element_1",
        content: element_content,
        span: element_start..element_end,
    });
}
{
    p.debug_sequence_element(2, 3, "element_2");
    let element_start = p.position;
    let element_content = ParseContent::Terminal(p.match_string(r#"}"#)?);
    let element_end = p.position;
    sequence_elements.push(ParseNode {
        rule_name: "element_2",
        content: element_content,
        span: element_start..element_end,
    });
}
        let content = ParseContent::Sequence(sequence_elements);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_1",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
            } else {
                ParseContent::Terminal("no_alternative_matched")
            }
        };

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "hex_escape",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("hex_escape", start_pos),
                Err(err) => parser.debug_exit_fail("hex_escape", err),
            };
            
            parse_result
        })
    }

    /// Parse conditional with memoization
    #[inline]
    fn parse_conditional(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_CONDITIONAL, |parser| {
            parser.debug_enter_rule("conditional");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let mut sequence_elements = Vec::with_capacity(7);
        {
            parser.debug_sequence_element(0, 7, "element_0");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string(r#"("#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_0",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(1, 7, "element_1");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string(r#"?("#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_1",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(2, 7, "element_2");
            let element_start = parser.position;
    let element_content = ParseContent::Alternative(Box::new(parser.parse_condition()?));
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_2",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(3, 7, "element_3");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string(r#")"#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_3",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(4, 7, "element_4");
            let element_start = parser.position;
    let element_content = ParseContent::Alternative(Box::new(parser.parse_yes_branch()?));
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_4",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(5, 7, "element_5");
            let element_start = parser.position;
    let element_content = parser.parse_quantified_optimized("?", |p| {
let mut sequence_elements = Vec::with_capacity(2);
{
    p.debug_sequence_element(0, 2, "element_0");
    let element_start = p.position;
    let element_content = ParseContent::Terminal(p.match_string(r#"|"#)?);
    let element_end = p.position;
    sequence_elements.push(ParseNode {
        rule_name: "element_0",
        content: element_content,
        span: element_start..element_end,
    });
}
{
    p.debug_sequence_element(1, 2, "element_1");
    let element_start = p.position;
    let element_content = ParseContent::Alternative(Box::new(p.parse_no_branch()?));
    let element_end = p.position;
    sequence_elements.push(ParseNode {
        rule_name: "element_1",
        content: element_content,
        span: element_start..element_end,
    });
}
    let content = ParseContent::Sequence(sequence_elements);
    Ok(content)
})?;
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_5",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(6, 7, "element_6");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string(r#")"#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_6",
                content: element_content,
                span: element_start..element_end,
            });
        }
        let result = ParseContent::Sequence(sequence_elements);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "conditional",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("conditional", start_pos),
                Err(err) => parser.debug_exit_fail("conditional", err),
            };
            
            parse_result
        })
    }

    /// Parse prop_name with memoization
    #[inline]
    fn parse_prop_name(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_PROP_NAME, |parser| {
            parser.debug_enter_rule("prop_name");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let result = ParseContent::Terminal(parser.match_regex_optimized(r#"[A-Za-z0-9_:-]+"#)?);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "prop_name",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("prop_name", start_pos),
                Err(err) => parser.debug_exit_fail("prop_name", err),
            };
            
            parse_result
        })
    }

    /// Parse condition with memoization
    #[inline]
    fn parse_condition(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_CONDITION, |parser| {
            parser.debug_enter_rule("condition");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let result = if false {
            ParseContent::Terminal("unreachable")
        } else {
            parser.debug_try_alternative(0, 3);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_lookaround()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_0",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(1, 3);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_name_ref()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_1",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(2, 3);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_digits()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_2",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
            } else {
                ParseContent::Terminal("no_alternative_matched")
            }
        };

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "condition",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("condition", start_pos),
                Err(err) => parser.debug_exit_fail("condition", err),
            };
            
            parse_result
        })
    }

    /// Parse code_lang with memoization
    #[inline]
    fn parse_code_lang(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_CODE_LANG, |parser| {
            parser.debug_enter_rule("code_lang");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let result = if false {
            ParseContent::Terminal("unreachable")
        } else {
            parser.debug_try_alternative(0, 2);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_string(r#"lua"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_0",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(1, 2);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_string(r#"js"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_1",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
            } else {
                ParseContent::Terminal("no_alternative_matched")
            }
        };

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "code_lang",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("code_lang", start_pos),
                Err(err) => parser.debug_exit_fail("code_lang", err),
            };
            
            parse_result
        })
    }

    /// Parse regex with memoization
    #[inline]
    fn parse_regex(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_REGEX, |parser| {
            parser.debug_enter_rule("regex");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let result = ParseContent::Alternative(Box::new(parser.parse_pattern()?));

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "regex",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("regex", start_pos),
                Err(err) => parser.debug_exit_fail("regex", err),
            };
            
            parse_result
        })
    }

    /// Parse code_string_single with memoization
    #[inline]
    fn parse_code_string_single(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_CODE_STRING_SINGLE, |parser| {
            parser.debug_enter_rule("code_string_single");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let mut sequence_elements = Vec::with_capacity(3);
        {
            parser.debug_sequence_element(0, 3, "element_0");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string("")?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_0",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(1, 3, "element_1");
            let element_start = parser.position;
    let element_content = parser.parse_quantified_optimized("*", |p| {
    let content = ParseContent::Alternative(Box::new(p.parse_code_string_single_char()?));
    Ok(content)
})?;
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_1",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(2, 3, "element_2");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string("")?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_2",
                content: element_content,
                span: element_start..element_end,
            });
        }
        let result = ParseContent::Sequence(sequence_elements);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "code_string_single",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("code_string_single", start_pos),
                Err(err) => parser.debug_exit_fail("code_string_single", err),
            };
            
            parse_result
        })
    }

    /// Parse code_block_plain with memoization
    #[inline]
    fn parse_code_block_plain(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_CODE_BLOCK_PLAIN, |parser| {
            parser.debug_enter_rule("code_block_plain");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let mut sequence_elements = Vec::with_capacity(3);
        {
            parser.debug_sequence_element(0, 3, "element_0");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string(r#"(?{"#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_0",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(1, 3, "element_1");
            let element_start = parser.position;
    let element_content = ParseContent::Alternative(Box::new(parser.parse_code_content()?));
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_1",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(2, 3, "element_2");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string(r#"})"#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_2",
                content: element_content,
                span: element_start..element_end,
            });
        }
        let result = ParseContent::Sequence(sequence_elements);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "code_block_plain",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("code_block_plain", start_pos),
                Err(err) => parser.debug_exit_fail("code_block_plain", err),
            };
            
            parse_result
        })
    }

    /// Parse dot with memoization
    #[inline]
    fn parse_dot(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_DOT, |parser| {
            parser.debug_enter_rule("dot");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let result = ParseContent::Terminal(parser.match_string(r#"."#)?);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "dot",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("dot", start_pos),
                Err(err) => parser.debug_exit_fail("dot", err),
            };
            
            parse_result
        })
    }

    /// Parse quantifier with memoization
    #[inline]
    fn parse_quantifier(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_QUANTIFIER, |parser| {
            parser.debug_enter_rule("quantifier");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let mut sequence_elements = Vec::with_capacity(2);
        {
            parser.debug_sequence_element(0, 2, "element_0");
            let element_start = parser.position;
    let element_content = ParseContent::Alternative(Box::new(parser.parse_quant_base()?));
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_0",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(1, 2, "element_1");
            let element_start = parser.position;
    let element_content = parser.parse_quantified_optimized("?", |p| {
    let content = ParseContent::Alternative(Box::new(p.parse_quant_suffix()?));
    Ok(content)
})?;
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_1",
                content: element_content,
                span: element_start..element_end,
            });
        }
        let result = ParseContent::Sequence(sequence_elements);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "quantifier",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("quantifier", start_pos),
                Err(err) => parser.debug_exit_fail("quantifier", err),
            };
            
            parse_result
        })
    }

    /// Parse no_branch with memoization
    #[inline]
    fn parse_no_branch(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_NO_BRANCH, |parser| {
            parser.debug_enter_rule("no_branch");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let result = ParseContent::Alternative(Box::new(parser.parse_pattern()?));

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "no_branch",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("no_branch", start_pos),
                Err(err) => parser.debug_exit_fail("no_branch", err),
            };
            
            parse_result
        })
    }

    /// Parse lookaround with memoization
    #[inline]
    fn parse_lookaround(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_LOOKAROUND, |parser| {
            parser.debug_enter_rule("lookaround");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let result = if false {
            ParseContent::Terminal("unreachable")
        } else {
            parser.debug_try_alternative(0, 4);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_lookahead_pos()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_0",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(1, 4);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_lookahead_neg()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_1",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(2, 4);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_lookbehind_pos()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_2",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(3, 4);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_lookbehind_neg()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_3",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
            } else {
                ParseContent::Terminal("no_alternative_matched")
            }
        };

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "lookaround",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("lookaround", start_pos),
                Err(err) => parser.debug_exit_fail("lookaround", err),
            };
            
            parse_result
        })
    }

    /// Parse class_item with memoization
    #[inline]
    fn parse_class_item(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_CLASS_ITEM, |parser| {
            parser.debug_enter_rule("class_item");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let result = if false {
            ParseContent::Terminal("unreachable")
        } else {
            parser.debug_try_alternative(0, 5);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_class_range()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_0",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(1, 5);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_class_escape()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_1",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(2, 5);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_posix_class()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_2",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(3, 5);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_class_literal()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_3",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(4, 5);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_class_start_end_bracket()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_4",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
            } else {
                ParseContent::Terminal("no_alternative_matched")
            }
        };

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "class_item",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("class_item", start_pos),
                Err(err) => parser.debug_exit_fail("class_item", err),
            };
            
            parse_result
        })
    }

    /// Parse lookbehind_pos with memoization
    #[inline]
    fn parse_lookbehind_pos(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_LOOKBEHIND_POS, |parser| {
            parser.debug_enter_rule("lookbehind_pos");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let mut sequence_elements = Vec::with_capacity(3);
        {
            parser.debug_sequence_element(0, 3, "element_0");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string(r#"(?<="#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_0",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(1, 3, "element_1");
            let element_start = parser.position;
    let element_content = ParseContent::Alternative(Box::new(parser.parse_pattern()?));
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_1",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(2, 3, "element_2");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string(r#")"#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_2",
                content: element_content,
                span: element_start..element_end,
            });
        }
        let result = ParseContent::Sequence(sequence_elements);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "lookbehind_pos",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("lookbehind_pos", start_pos),
                Err(err) => parser.debug_exit_fail("lookbehind_pos", err),
            };
            
            parse_result
        })
    }

    /// Parse lookahead_pos with memoization
    #[inline]
    fn parse_lookahead_pos(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_LOOKAHEAD_POS, |parser| {
            parser.debug_enter_rule("lookahead_pos");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let mut sequence_elements = Vec::with_capacity(3);
        {
            parser.debug_sequence_element(0, 3, "element_0");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string(r#"(?="#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_0",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(1, 3, "element_1");
            let element_start = parser.position;
    let element_content = ParseContent::Alternative(Box::new(parser.parse_pattern()?));
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_1",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(2, 3, "element_2");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string(r#")"#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_2",
                content: element_content,
                span: element_start..element_end,
            });
        }
        let result = ParseContent::Sequence(sequence_elements);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "lookahead_pos",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("lookahead_pos", start_pos),
                Err(err) => parser.debug_exit_fail("lookahead_pos", err),
            };
            
            parse_result
        })
    }

    /// Parse code_escaped_char with memoization
    #[inline]
    fn parse_code_escaped_char(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_CODE_ESCAPED_CHAR, |parser| {
            parser.debug_enter_rule("code_escaped_char");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let mut sequence_elements = Vec::with_capacity(2);
        {
            parser.debug_sequence_element(0, 2, "element_0");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string(r#"\\\\"#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_0",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(1, 2, "element_1");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_regex_optimized(r#"."#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_1",
                content: element_content,
                span: element_start..element_end,
            });
        }
        let result = ParseContent::Sequence(sequence_elements);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "code_escaped_char",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("code_escaped_char", start_pos),
                Err(err) => parser.debug_exit_fail("code_escaped_char", err),
            };
            
            parse_result
        })
    }

    /// Parse concatenation with memoization
    #[inline]
    fn parse_concatenation(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_CONCATENATION, |parser| {
            parser.debug_enter_rule("concatenation");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let mut sequence_elements = Vec::with_capacity(1);
        {
            parser.debug_sequence_element(0, 1, "element_0");
            let element_start = parser.position;
    let element_content = parser.parse_quantified_optimized("+", |p| {
    let content = ParseContent::Alternative(Box::new(p.parse_piece()?));
    Ok(content)
})?;
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_0",
                content: element_content,
                span: element_start..element_end,
            });
        }
        let result = ParseContent::Sequence(sequence_elements);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "concatenation",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("concatenation", start_pos),
                Err(err) => parser.debug_exit_fail("concatenation", err),
            };
            
            parse_result
        })
    }

    /// Parse piece with memoization
    #[inline]
    fn parse_piece(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_PIECE, |parser| {
            parser.debug_enter_rule("piece");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let mut sequence_elements = Vec::with_capacity(2);
        {
            parser.debug_sequence_element(0, 2, "element_0");
            let element_start = parser.position;
    let element_content = ParseContent::Alternative(Box::new(parser.parse_atom()?));
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_0",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(1, 2, "element_1");
            let element_start = parser.position;
    let element_content = parser.parse_quantified_optimized("?", |p| {
    let content = ParseContent::Alternative(Box::new(p.parse_quantifier()?));
    Ok(content)
})?;
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_1",
                content: element_content,
                span: element_start..element_end,
            });
        }
        let result = ParseContent::Sequence(sequence_elements);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "piece",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("piece", start_pos),
                Err(err) => parser.debug_exit_fail("piece", err),
            };
            
            parse_result
        })
    }

    /// Parse literal_char with memoization
    #[inline]
    fn parse_literal_char(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_LITERAL_CHAR, |parser| {
            parser.debug_enter_rule("literal_char");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let result = ParseContent::Terminal(parser.match_regex_optimized(r#"[^|()*+?.{}^$\\\\\\[\\]]"#)?);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "literal_char",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("literal_char", start_pos),
                Err(err) => parser.debug_exit_fail("literal_char", err),
            };
            
            parse_result
        })
    }

    /// Parse octal_escape with memoization
    #[inline]
    fn parse_octal_escape(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_OCTAL_ESCAPE, |parser| {
            parser.debug_enter_rule("octal_escape");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let mut sequence_elements = Vec::with_capacity(3);
        {
            parser.debug_sequence_element(0, 3, "element_0");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_regex_optimized(r#"[0-7]"#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_0",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(1, 3, "element_1");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_regex_optimized(r#"[0-7]?"#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_1",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(2, 3, "element_2");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_regex_optimized(r#"[0-7]?"#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_2",
                content: element_content,
                span: element_start..element_end,
            });
        }
        let result = ParseContent::Sequence(sequence_elements);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "octal_escape",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("octal_escape", start_pos),
                Err(err) => parser.debug_exit_fail("octal_escape", err),
            };
            
            parse_result
        })
    }

    /// Parse atomic_group with memoization
    #[inline]
    fn parse_atomic_group(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_ATOMIC_GROUP, |parser| {
            parser.debug_enter_rule("atomic_group");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let mut sequence_elements = Vec::with_capacity(3);
        {
            parser.debug_sequence_element(0, 3, "element_0");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string(r#"(?>"#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_0",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(1, 3, "element_1");
            let element_start = parser.position;
    let element_content = ParseContent::Alternative(Box::new(parser.parse_pattern()?));
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_1",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(2, 3, "element_2");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string(r#")"#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_2",
                content: element_content,
                span: element_start..element_end,
            });
        }
        let result = ParseContent::Sequence(sequence_elements);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "atomic_group",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("atomic_group", start_pos),
                Err(err) => parser.debug_exit_fail("atomic_group", err),
            };
            
            parse_result
        })
    }

    /// Parse quant_base with memoization
    #[inline]
    fn parse_quant_base(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_QUANT_BASE, |parser| {
            parser.debug_enter_rule("quant_base");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let result = if false {
            ParseContent::Terminal("unreachable")
        } else {
            parser.debug_try_alternative(0, 4);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_string(r#"*"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_0",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(1, 4);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_string(r#"+"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_1",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(2, 4);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_string(r#"?"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_2",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(3, 4);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_counted_quantifier()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_3",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
            } else {
                ParseContent::Terminal("no_alternative_matched")
            }
        };

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "quant_base",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("quant_base", start_pos),
                Err(err) => parser.debug_exit_fail("quant_base", err),
            };
            
            parse_result
        })
    }

    /// Parse anchor with memoization
    #[inline]
    fn parse_anchor(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_ANCHOR, |parser| {
            parser.debug_enter_rule("anchor");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let result = if false {
            ParseContent::Terminal("unreachable")
        } else {
            parser.debug_try_alternative(0, 8);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_string(r#"^"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_0",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(1, 8);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_string(r#"$"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_1",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(2, 8);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_string(r#"\\\\A"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_2",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(3, 8);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_string(r#"\\\\Z"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_3",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(4, 8);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_string(r#"\\\\z"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_4",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(5, 8);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_string(r#"\\\\b"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_5",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(6, 8);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_string(r#"\\\\B"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_6",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(7, 8);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Terminal(p.match_string(r#"\\\\G"#)?);
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_7",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
            } else {
                ParseContent::Terminal("no_alternative_matched")
            }
        };

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "anchor",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("anchor", start_pos),
                Err(err) => parser.debug_exit_fail("anchor", err),
            };
            
            parse_result
        })
    }

    /// Parse capturing_group with memoization
    #[inline]
    fn parse_capturing_group(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_CAPTURING_GROUP, |parser| {
            parser.debug_enter_rule("capturing_group");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let mut sequence_elements = Vec::with_capacity(3);
        {
            parser.debug_sequence_element(0, 3, "element_0");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string(r#"("#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_0",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(1, 3, "element_1");
            let element_start = parser.position;
    let element_content = ParseContent::Alternative(Box::new(parser.parse_pattern()?));
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_1",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(2, 3, "element_2");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string(r#")"#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_2",
                content: element_content,
                span: element_start..element_end,
            });
        }
        let result = ParseContent::Sequence(sequence_elements);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "capturing_group",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("capturing_group", start_pos),
                Err(err) => parser.debug_exit_fail("capturing_group", err),
            };
            
            parse_result
        })
    }

    /// Parse simple_escape with memoization
    #[inline]
    fn parse_simple_escape(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_SIMPLE_ESCAPE, |parser| {
            parser.debug_enter_rule("simple_escape");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let result = ParseContent::Terminal(parser.match_regex_optimized(r#"."#)?);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "simple_escape",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("simple_escape", start_pos),
                Err(err) => parser.debug_exit_fail("simple_escape", err),
            };
            
            parse_result
        })
    }

    /// Parse name_start with memoization
    #[inline]
    fn parse_name_start(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_NAME_START, |parser| {
            parser.debug_enter_rule("name_start");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let result = ParseContent::Terminal(parser.match_regex_optimized(r#"[A-Za-z_]"#)?);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "name_start",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("name_start", start_pos),
                Err(err) => parser.debug_exit_fail("name_start", err),
            };
            
            parse_result
        })
    }

    /// Parse modifier_seq with memoization
    #[inline]
    fn parse_modifier_seq(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_MODIFIER_SEQ, |parser| {
            parser.debug_enter_rule("modifier_seq");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let mut sequence_elements = Vec::with_capacity(2);
        {
            parser.debug_sequence_element(0, 2, "element_0");
            let element_start = parser.position;
    let element_content = ParseContent::Alternative(Box::new(parser.parse_modifier_group()?));
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_0",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(1, 2, "element_1");
            let element_start = parser.position;
    let element_content = parser.parse_quantified_optimized("?", |p| {
let mut sequence_elements = Vec::with_capacity(2);
{
    p.debug_sequence_element(0, 2, "element_0");
    let element_start = p.position;
    let element_content = ParseContent::Terminal(p.match_string(r#"-"#)?);
    let element_end = p.position;
    sequence_elements.push(ParseNode {
        rule_name: "element_0",
        content: element_content,
        span: element_start..element_end,
    });
}
{
    p.debug_sequence_element(1, 2, "element_1");
    let element_start = p.position;
    let element_content = ParseContent::Alternative(Box::new(p.parse_modifier_group()?));
    let element_end = p.position;
    sequence_elements.push(ParseNode {
        rule_name: "element_1",
        content: element_content,
        span: element_start..element_end,
    });
}
    let content = ParseContent::Sequence(sequence_elements);
    Ok(content)
})?;
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_1",
                content: element_content,
                span: element_start..element_end,
            });
        }
        let result = ParseContent::Sequence(sequence_elements);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "modifier_seq",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("modifier_seq", start_pos),
                Err(err) => parser.debug_exit_fail("modifier_seq", err),
            };
            
            parse_result
        })
    }

    /// Parse class_start_end_bracket with memoization
    #[inline]
    fn parse_class_start_end_bracket(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_CLASS_START_END_BRACKET, |parser| {
            parser.debug_enter_rule("class_start_end_bracket");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let result = ParseContent::Terminal(parser.match_string(r#"]"#)?);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "class_start_end_bracket",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("class_start_end_bracket", start_pos),
                Err(err) => parser.debug_exit_fail("class_start_end_bracket", err),
            };
            
            parse_result
        })
    }

    /// Parse escape_unit with memoization
    #[inline]
    fn parse_escape_unit(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_ESCAPE_UNIT, |parser| {
            parser.debug_enter_rule("escape_unit");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let result = if false {
            ParseContent::Terminal("unreachable")
        } else {
            parser.debug_try_alternative(0, 6);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_simple_escape()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_0",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(1, 6);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_hex_escape()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_1",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(2, 6);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_unicode_escape()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_2",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(3, 6);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_octal_escape()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_3",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(4, 6);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_control_escape()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_4",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(5, 6);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_property_escape()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_5",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
            } else {
                ParseContent::Terminal("no_alternative_matched")
            }
        };

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "escape_unit",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("escape_unit", start_pos),
                Err(err) => parser.debug_exit_fail("escape_unit", err),
            };
            
            parse_result
        })
    }

    /// Parse class_atom with memoization
    #[inline]
    fn parse_class_atom(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_CLASS_ATOM, |parser| {
            parser.debug_enter_rule("class_atom");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let result = if false {
            ParseContent::Terminal("unreachable")
        } else {
            parser.debug_try_alternative(0, 2);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_class_escape()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_0",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
        } else {
            parser.debug_try_alternative(1, 2);
            if let Some(node) = parser.try_parse(|p| {
                let start = p.position;
        let content = ParseContent::Alternative(Box::new(p.parse_class_literal()?));
                let end = p.position;
                Ok(ParseNode {
                    rule_name: "alt_1",
                    content,
                    span: start..end,
                })
            }) {
                ParseContent::Alternative(Box::new(node))
            } else {
                ParseContent::Terminal("no_alternative_matched")
            }
        };

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "class_atom",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("class_atom", start_pos),
                Err(err) => parser.debug_exit_fail("class_atom", err),
            };
            
            parse_result
        })
    }

    /// Parse code_string_escape with memoization
    #[inline]
    fn parse_code_string_escape(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_CODE_STRING_ESCAPE, |parser| {
            parser.debug_enter_rule("code_string_escape");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let mut sequence_elements = Vec::with_capacity(2);
        {
            parser.debug_sequence_element(0, 2, "element_0");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string(r#"\\\\"#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_0",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(1, 2, "element_1");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_regex_optimized(r#"."#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_1",
                content: element_content,
                span: element_start..element_end,
            });
        }
        let result = ParseContent::Sequence(sequence_elements);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "code_string_escape",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("code_string_escape", start_pos),
                Err(err) => parser.debug_exit_fail("code_string_escape", err),
            };
            
            parse_result
        })
    }

    /// Parse code_balanced_braces with memoization
    #[inline]
    fn parse_code_balanced_braces(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_CODE_BALANCED_BRACES, |parser| {
            parser.debug_enter_rule("code_balanced_braces");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {
        let mut sequence_elements = Vec::with_capacity(3);
        {
            parser.debug_sequence_element(0, 3, "element_0");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string(r#"{"#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_0",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(1, 3, "element_1");
            let element_start = parser.position;
    let element_content = ParseContent::Alternative(Box::new(parser.parse_code_content()?));
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_1",
                content: element_content,
                span: element_start..element_end,
            });
        }
        {
            parser.debug_sequence_element(2, 3, "element_2");
            let element_start = parser.position;
    let element_content = ParseContent::Terminal(parser.match_string(r#"}"#)?);
            let element_end = parser.position;
            sequence_elements.push(ParseNode {
                rule_name: "element_2",
                content: element_content,
                span: element_start..element_end,
            });
        }
        let result = ParseContent::Sequence(sequence_elements);

                let end_pos = parser.position;
                
                Ok(ParseNode {
                    rule_name: "code_balanced_braces",
                    content: result,
                    span: start_pos..end_pos,
                })
            })();
            
            match &parse_result {
                Ok(_) => parser.debug_exit_success("code_balanced_braces", start_pos),
                Err(err) => parser.debug_exit_fail("code_balanced_braces", err),
            };
            
            parse_result
        })
    }

    /// Optimized regex matching with character classes
    #[inline]
    fn match_regex_optimized(&mut self, pattern: &str) -> ParseResult<&'input str> {
        let start_pos = self.position;
        
        match pattern {
            "." => {
                // Any character except newline
                if let Some(ch) = self.current_char() {
                    if ch != '\n' {
                        self.advance();
                        return Ok(&self.input[start_pos..self.position]);
                    }
                }
                Err(ParseError::UnexpectedEof { position: start_pos })
            }
            r"[0-9]" | r"\d" => {
                // ASCII digit fast path
                if let Some(byte) = self.current_byte() {
                    if byte >= b'0' && byte <= b'9' {
                        self.position += 1;
                        return Ok(&self.input[start_pos..self.position]);
                    }
                }
                Err(ParseError::InvalidSyntax { 
                    message: "Expected digit", 
                    position: start_pos 
                })
            }
            r"[A-Za-z]" => {
                // ASCII letter fast path
                if let Some(byte) = self.current_byte() {
                    if (byte >= b'A' && byte <= b'Z') || (byte >= b'a' && byte <= b'z') {
                        self.position += 1;
                        return Ok(&self.input[start_pos..self.position]);
                    }
                }
                Err(ParseError::InvalidSyntax { 
                    message: "Expected letter", 
                    position: start_pos 
                })
            }
            r"[A-Za-z_]" => {
                // Name start character
                if let Some(byte) = self.current_byte() {
                    if (byte >= b'A' && byte <= b'Z') || 
                       (byte >= b'a' && byte <= b'z') || 
                       byte == b'_' {
                        self.position += 1;
                        return Ok(&self.input[start_pos..self.position]);
                    }
                }
                Err(ParseError::InvalidSyntax { 
                    message: "Expected name start", 
                    position: start_pos 
                })
            }
            r"[A-Za-z0-9_]" => {
                // Name continue character  
                if let Some(byte) = self.current_byte() {
                    if (byte >= b'A' && byte <= b'Z') || 
                       (byte >= b'a' && byte <= b'z') || 
                       (byte >= b'0' && byte <= b'9') ||
                       byte == b'_' {
                        self.position += 1;
                        return Ok(&self.input[start_pos..self.position]);
                    }
                }
                Err(ParseError::InvalidSyntax { 
                    message: "Expected name continue", 
                    position: start_pos 
                })
            }
            r"\s+" => {
                // Whitespace - consume greedily
                let mut consumed = false;
                while let Some(ch) = self.current_char() {
                    if ch.is_whitespace() {
                        self.advance();
                        consumed = true;
                    } else {
                        break;
                    }
                }
                if consumed {
                    Ok(&self.input[start_pos..self.position])
                } else {
                    Err(ParseError::InvalidSyntax { 
                        message: "Expected whitespace", 
                        position: start_pos 
                    })
                }
            }
            _ => {
                // Fallback for complex patterns including character classes
                if let Some(ch) = self.current_char() {
                    let matches = if pattern.starts_with("[^") && pattern.ends_with("]") {
                        // Negated character class - match chars NOT in the class
                        let class_chars = &pattern[2..pattern.len()-1];
                        !self.char_in_class(ch, class_chars)
                    } else if pattern.starts_with("[") && pattern.ends_with("]") {
                        // Positive character class - match chars IN the class
                        let class_chars = &pattern[1..pattern.len()-1];
                        self.char_in_class(ch, class_chars)
                    } else {
                        // Simple pattern matching
                        pattern.contains(ch)
                    };
                    
                    if matches {
                        self.advance();
                        Ok(&self.input[start_pos..self.position])
                    } else {
                        Err(ParseError::InvalidSyntax {
                            message: "Pattern mismatch",
                            position: start_pos,
                        })
                    }
                } else {
                    Err(ParseError::UnexpectedEof { position: start_pos })
                }
            }
        }
    }

    /// Check if a character matches a character class string
    #[inline]
    fn char_in_class(&self, ch: char, class_chars: &str) -> bool {
        // Handle escaped characters in character classes
        let mut chars = class_chars.chars().peekable();
        
        while let Some(c) = chars.next() {
            match c {
                '\\' => {
                    // Handle escaped character - consume pairs of backslashes
                    if let Some(next_char) = chars.next() {
                        if next_char == '\\' {
                            // Double backslash becomes single literal backslash
                            if ch == '\\' {
                                return true;
                            }
                        } else {
                            // Backslash followed by other character
                            if ch == next_char {
                                return true;
                            }
                        }
                    }
                }
                _ => {
                    // Regular character
                    if ch == c {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// High-performance quantifier parsing
    #[inline]
    fn parse_quantified_optimized<F>(&mut self, quantifier: &str, mut f: F) -> ParseResult<ParseContent<'input>>
    where
        F: FnMut(&mut Self) -> ParseResult<ParseContent<'input>>,
    {
        let mut results = Vec::new();
        let mut iteration = 0;
        
        match quantifier {
            "*" => {
                // Zero or more - optimized loop
                while let Some(content) = self.try_parse(&mut f) {
                    iteration += 1;
                    self.debug_quantifier_iteration(iteration, quantifier);
                    results.push(ParseNode {
                        rule_name: "quantified",
                        content,
                        span: 0..0, // Will be filled by caller
                    });
                }
                Ok(ParseContent::Quantified(results, "*"))
            }
            "+" => {
                // One or more - require at least one
                iteration += 1;
                self.debug_quantifier_iteration(iteration, quantifier);
                match f(self) {
                    Ok(content) => {
                        results.push(ParseNode {
                            rule_name: "quantified",
                            content,
                            span: 0..0,
                        });
                        
                        while let Some(content) = self.try_parse(&mut f) {
                            iteration += 1;
                            self.debug_quantifier_iteration(iteration, quantifier);
                            results.push(ParseNode {
                                rule_name: "quantified", 
                                content,
                                span: 0..0,
                            });
                        }
                        Ok(ParseContent::Quantified(results, "+"))
                    }
                    Err(err) => Err(err),
                }
            }
            "?" => {
                // Zero or one
                if let Some(content) = self.try_parse(&mut f) {
                    iteration += 1;
                    self.debug_quantifier_iteration(iteration, quantifier);
                    results.push(ParseNode {
                        rule_name: "quantified",
                        content,
                        span: 0..0,
                    });
                }
                Ok(ParseContent::Quantified(results, "?"))
            }
            _ => Err(ParseError::InvalidSyntax {
                message: "Unknown quantifier",
                position: self.position,
            }),
        }
    }

}


#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_parser_creation_speed() {
        let input = "test input for performance";
        let start = Instant::now();
        
        for _ in 0..10_000 {
            let _parser = regexParser::new(input);
        }
        
        let duration = start.elapsed();
        println!("Parser creation: {:?} for 10k iterations", duration);
        assert!(duration.as_millis() < 100); // Should be very fast
    }

    #[test] 
    fn test_simple_parsing_speed() {
        let mut parser = regexParser::new("simple test input");
        let start = Instant::now();
        
        for _ in 0..1_000 {
            parser.position = 0;
            parser.memo.clear();
            let _result = parser.try_parse(|p| p.match_string("simple"));
        }
        
        let duration = start.elapsed();
        println!("Simple parsing: {:?} for 1k iterations", duration);
        assert!(duration.as_millis() < 50);
    }

    #[test]
    fn test_memoization_effectiveness() {
        let mut parser = regexParser::new("test test test");
        
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
        
        println!("First parse: {:?}, Second parse: {:?}", duration1, duration2);
        // Second should be significantly faster due to memoization
        assert!(duration2 < duration1);
    }

    #[bench]
    fn bench_regex_parsing(b: &mut test::Bencher) {
        let input = "(?{lua: return 'test'})|[a-z]+(?={word})";
        b.iter(|| {
            let mut parser = regexParser::new(input);
            parser.parse().unwrap()
        });
    }
}
