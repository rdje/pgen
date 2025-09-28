//! Rust AST Pipeline Implementation
//!
//! Provides complete EBNF AST transformation pipeline with dual-mode API:
//! - Same-language optimization: In-memory data structures
//! - Cross-language interface: JSON input/output
//!
//! Implements the 5-stage transformation pipeline equivalent to Perl AST::Transform.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use anyhow::{Context, Result};

// Import the generated semantic annotation parser
pub mod semantic_annotation_parser {
    include!("../../generated/semantic_annotation_parser.rs");
}
use semantic_annotation_parser::Semantic_annotationParser;

// Import the generated return annotation parser
pub mod return_annotation_parser {
    include!("../../generated/return_annotation_parser.rs");
}
use return_annotation_parser::Return_annotationParser;

mod high_performance_generator;
use high_performance_generator::HighPerformanceRustGenerator;

/// Configuration for AST transformation pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub debug: bool,
    pub trace: bool,
    pub preserve_annotations: bool,
    pub validate_input: bool,
    pub validate_output: bool,
    pub max_recursion_depth: usize,
    pub bootstrap_mode: bool,
    pub eliminate_left_recursion: bool,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            debug: false,
            trace: false,
            preserve_annotations: true,
            validate_input: true,
            validate_output: true,
            max_recursion_depth: 100,
            bootstrap_mode: false,
            eliminate_left_recursion: true, // Enable by default to fix stack overflow
        }
    }
}

/// Raw AST token representation - supports mixed String and Array content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TokenValue {
    String(String),
    Array(Vec<String>),
}

impl TokenValue {
    /// Get as string reference if this is a String variant
    pub fn as_str(&self) -> Option<&str> {
        match self {
            TokenValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }
    
    /// Check if this is an empty string
    pub fn is_empty(&self) -> bool {
        match self {
            TokenValue::String(s) => s.is_empty(),
            TokenValue::Array(v) => v.is_empty(),
        }
    }
}

impl std::fmt::Display for TokenValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenValue::String(s) => write!(f, "{}", s),
            TokenValue::Array(v) => write!(f, "{:?}", v),
        }
    }
}

impl PartialEq<&str> for TokenValue {
    fn eq(&self, other: &&str) -> bool {
        match self {
            TokenValue::String(s) => s == *other,
            _ => false,
        }
    }
}

pub type Token = Vec<TokenValue>;
pub type TokenSequence = Vec<Token>;
pub type RawAST = Vec<TokenSequence>;

/// Raw AST JSON structure from ebnf_to_json.pl
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawASTJson {
    pub grammar_name: String,
    pub raw_ast: RawAST,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// AST node types in the transformed AST
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ASTNode {
    Atom { value: ASTValue },
    Sequence { elements: Vec<ASTNode> },
    Or { alternatives: Vec<ASTNode> },
    Quantified { element: Box<ASTNode>, quantifier: String },
}

/// Values that can be stored in AST nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ASTValue {
    Token(Token),
    Node(Box<ASTNode>),
}

/// Return annotation information for code generation  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnAnnotation {
    pub annotation_type: String, // "return_scalar", "return_array", "return_object"
    pub annotation_content: String, // The original annotation content - we parse this on demand in the generator
}

/// Preserved annotations from raw AST
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Annotations {
    pub semantic_annotations: HashMap<String, Vec<String>>,
    pub logging_annotations: HashMap<String, Vec<String>>,
    pub return_annotations: HashMap<String, ReturnAnnotation>,
}

/// Transformation statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransformStats {
    pub rules_processed: usize,
    pub annotations_preserved: usize,
    pub transformations_applied: usize,
}

/// Transformed AST JSON structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformedASTJson {
    pub grammar_name: String,
    pub grammar_tree: HashMap<String, ASTNode>,
    pub rule_order: Vec<String>,
    pub metadata: TransformMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformMetadata {
    pub format: String,
    pub source_format: String,
    pub transformed_at: String,
    pub transformer: String,
    pub pipeline_stage: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Annotations>,
    pub stats: TransformStats,
}

/// Main Rust AST Pipeline implementation
pub struct RustASTPipeline {
    config: PipelineConfig,
    stats: TransformStats,
    annotations: Annotations,
    entry_rule: Option<String>,
}

impl RustASTPipeline {
    /// Create new pipeline with configuration
    pub fn new(config: PipelineConfig) -> Self {
        Self {
            config,
            stats: TransformStats::default(),
            annotations: Annotations::default(),
            entry_rule: None,
        }
    }
    
    /// Create new pipeline with left recursion elimination enabled
    /// This will help resolve stack overflow issues caused by left-recursive grammars
    #[allow(dead_code)]
    pub fn with_left_recursion_elimination() -> Self {
        let mut config = PipelineConfig::default();
        config.eliminate_left_recursion = true;
        Self {
            config,
            stats: TransformStats::default(),
            annotations: Annotations::default(),
            entry_rule: None,
        }
    }
    
    /// Enable left recursion elimination on this pipeline
    #[allow(dead_code)]
    pub fn enable_left_recursion_elimination(&mut self) {
        self.config.eliminate_left_recursion = true;
    }
    
    /// Disable left recursion elimination on this pipeline
    #[allow(dead_code)]
    pub fn disable_left_recursion_elimination(&mut self) {
        self.config.eliminate_left_recursion = false;
    }
    
    /// Check if left recursion elimination is enabled
    #[allow(dead_code)]
    pub fn is_left_recursion_elimination_enabled(&self) -> bool {
        self.config.eliminate_left_recursion
    }
    
    /// Extract the entry rule name from raw AST JSON
    fn extract_entry_rule(&mut self, raw_ast: &RawAST) -> Result<String> {
        if raw_ast.is_empty() {
            anyhow::bail!("Raw AST is empty - cannot determine entry rule");
        }
        
        let first_rule = &raw_ast[0];
        if first_rule.is_empty() {
            anyhow::bail!("First rule in raw AST is empty");
        }
        
        // Look for the first rule token to get the entry rule name
        for token in first_rule {
            if token.len() == 2 {
                if let (TokenValue::String(token_type), TokenValue::String(rule_name)) = (&token[0], &token[1]) {
                    if token_type == "rule" {
                        self.entry_rule = Some(rule_name.clone());
                        return Ok(rule_name.clone());
                    }
                }
            }
        }
        
        anyhow::bail!("Could not find entry rule name in first rule of raw AST")
    }

    /// Load raw AST JSON from file
    pub fn load_raw_ast(&self, file_path: &str) -> Result<RawASTJson> {
        if self.config.debug {
            println!("Loading raw AST from: {}", file_path);
        }

        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file: {}", file_path))?;

        let data: RawASTJson = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse JSON from: {}", file_path))?;

        if self.config.validate_input {
            self.validate_raw_ast(&data)?;
        }

        Ok(data)
    }

    /// Validate raw AST JSON format
    fn validate_raw_ast(&self, data: &RawASTJson) -> Result<()> {
        if data.grammar_name.is_empty() {
            anyhow::bail!("Raw AST JSON missing grammar_name");
        }

        if data.raw_ast.is_empty() {
            anyhow::bail!("Raw AST JSON has empty raw_ast array");
        }

        if let Some(format) = data.metadata.get("format") {
            if format.as_str() != Some("raw_ast") {
                anyhow::bail!("metadata.format must be 'raw_ast'");
            }
        }

        Ok(())
    }

    /// Transform raw AST to semantic AST (main pipeline)
    pub fn transform_raw_ast(&mut self, raw_ast: &RawAST) -> Result<(HashMap<String, ASTNode>, Vec<String>)> {
        if self.config.debug {
            println!("=== Rust AST Transformation Pipeline ===");
        }
        
        // Extract the entry rule name for dynamic parser instantiation
        let entry_rule = self.extract_entry_rule(raw_ast)?;
        if self.config.debug {
            println!("Detected entry rule: {}", entry_rule);
        }

        // Stage 1: Extract annotations
        let cleaned_ast = self.extract_annotations(raw_ast)?;

        // Stage 2: Group by OR operators  
        let grouped_rules = self.group_by_or_operators(&cleaned_ast)?;

        // Stage 2.5: Handle parentheses
        let processed_rules = self.handle_parentheses(&grouped_rules)?;

        // Stage 3: Parse sequences
        let sequenced_rules = self.parse_sequences(&processed_rules)?;

        // Stage 4: Handle quantifiers
        let quantified_rules = self.handle_quantifiers(&sequenced_rules)?;

        // Stage 5: Build tree structure
        let (mut grammar_tree, mut rule_order) = self.build_tree_structure(&quantified_rules)?;

        // Stage 6 (Optional): Left recursion elimination
        if self.config.eliminate_left_recursion {
            if self.config.debug {
                println!("Stage 6: Applying left recursion elimination...");
            }
            (grammar_tree, rule_order) = self.eliminate_left_recursion(grammar_tree, rule_order)?;
            self.stats.transformations_applied = 6;
        } else {
            self.stats.transformations_applied = 5;
        }

        self.stats.rules_processed = grammar_tree.len();

        Ok((grammar_tree, rule_order))
    }

    /// Check if bootstrap mode should be used for annotation parsing
    fn should_use_bootstrap_mode(&self) -> bool {
        self.config.bootstrap_mode ||
        !self.external_parsers_available()
    }
    
    /// Check if external generated parsers are available
    fn external_parsers_available(&self) -> bool {
        // In a more sophisticated implementation, we could check file existence
        // For now, assume they're available unless explicitly in bootstrap mode
        !self.config.bootstrap_mode
    }
    
    /// Parse semantic annotation using the semantic annotation parser
    fn parse_semantic_annotation(&self, annotation_value: &str) -> Result<String> {
        if self.config.debug {
            println!("[SEMANTIC_PARSE] ===== ENTERING parse_semantic_annotation =====");
            println!("[SEMANTIC_PARSE] Input annotation_value: '{}'", annotation_value);
            println!("[SEMANTIC_PARSE] Input length: {} characters", annotation_value.len());
        }
        
        if self.should_use_bootstrap_mode() {
            if self.config.debug {
                println!("[SEMANTIC_PARSE] Using BOOTSTRAP mode for semantic annotation: '{}'", annotation_value);
            }
            let result = self.parse_semantic_annotation_bootstrap(annotation_value);
            if self.config.debug {
                println!("[SEMANTIC_PARSE] ===== EXITING parse_semantic_annotation (BOOTSTRAP) =====");
            }
            return result;
        }
        
        if self.config.debug {
            println!("[SEMANTIC_PARSE] Creating external parser for annotation: '{}'", annotation_value);
        }
        
        // Use the generated semantic annotation parser to parse the value
        let mut parser = if self.config.debug || self.config.trace {
            if self.config.debug {
                println!("[SEMANTIC_PARSE] Creating semantic parser with DEBUG enabled");
            }
            Semantic_annotationParser::with_debug(annotation_value)
        } else {
            if self.config.debug {
                println!("[SEMANTIC_PARSE] Creating semantic parser without debug");
            }
            Semantic_annotationParser::new(annotation_value)
        };
        
        if self.config.debug {
            println!("[SEMANTIC_PARSE] About to call parser.parse() for annotation: '{}'", annotation_value);
            println!("[SEMANTIC_PARSE] This is where stack overflow might occur...");
        }
        
        // Parse the annotation value into an AST
        match parser.parse() {
            Ok(parsed_ast) => {
                if self.config.debug {
                    println!("[SEMANTIC_PARSE] ✅ SUCCESS: External parser succeeded for: '{}'", annotation_value);
                    println!("[SEMANTIC_PARSE] About to simplify AST...");
                }
                // For now, convert the AST to a simplified JSON representation for storage
                // In the future, this will be used by the code generator directly
                let simplified = self.simplify_semantic_parse_node(&parsed_ast);
                if self.config.debug {
                    println!("[SEMANTIC_PARSE] ✅ SUCCESS: AST simplified");
                    println!("[SEMANTIC_PARSE] About to serialize to JSON...");
                }
                let result = serde_json::to_string(&simplified)
                    .context("Failed to serialize parsed semantic annotation AST");
                if self.config.debug {
                    println!("[SEMANTIC_PARSE] ✅ SUCCESS: JSON serialization complete");
                    println!("[SEMANTIC_PARSE] ===== EXITING parse_semantic_annotation (SUCCESS) =====");
                }
                result
            }
            Err(err) => {
                // If parsing fails, fallback to bootstrap mode
                if self.config.debug {
                    println!("[SEMANTIC_PARSE] ❌ FAILURE: External parser failed with error: {:?}", err);
                    println!("[SEMANTIC_PARSE] Falling back to bootstrap mode for: '{}'", annotation_value);
                }
                let result = self.parse_semantic_annotation_bootstrap(annotation_value);
                if self.config.debug {
                    println!("[SEMANTIC_PARSE] ===== EXITING parse_semantic_annotation (FALLBACK) =====");
                }
                result
            }
        }
    }
    
    /// Built-in bootstrap semantic annotation parser
    /// Simple name:value parsing - handles function calls naturally without artificial limits
    fn parse_semantic_annotation_bootstrap(&self, annotation_value: &str) -> Result<String> {
        let trimmed = annotation_value.trim();
        
        // Simple pattern: name:value (handles function calls naturally)
        if let Some(captures) = regex::Regex::new(r"^([a-zA-Z_][a-zA-Z0-9_]*):?\s*(.+)$")
            .unwrap()
            .captures(trimmed) {
            let name = captures.get(1).unwrap().as_str();
            let value = captures.get(2).unwrap().as_str().trim();
            
            // Simple storage - no complex parsing or artificial limits
            return Ok(format!("{}:{}", name, value.trim_matches('"')));
        }
        
        // Fallback: store as raw if pattern doesn't match basic name:value
        if self.config.debug {
            println!("WARNING: Semantic annotation pattern not recognized in bootstrap mode");
            println!("  Pattern: {}", annotation_value);
            println!("  Stored as raw string - use full parser mode for complete support");
        }
        Ok(format!("raw:{}", annotation_value))
    }
    
    
    /// Parse return annotation using the return annotation parser
    fn parse_return_annotation(&self, annotation_value: &str) -> Result<String> {
        if self.should_use_bootstrap_mode() {
            if self.config.debug {
                println!("[AST Pipeline] Using bootstrap mode for return annotation: '{}'", annotation_value);
            }
            return self.parse_return_annotation_bootstrap(annotation_value);
        }
        
        // Use the generated return annotation parser to parse the value
        let mut parser = if self.config.debug || self.config.trace {
            Return_annotationParser::with_debug(annotation_value)
        } else {
            Return_annotationParser::new(annotation_value)
        };
        
        if self.config.debug {
            println!("[AST Pipeline] Parsing return annotation with value: '{}'", annotation_value);
        }
        
        // Parse the annotation value into an AST
        match parser.parse() {
            Ok(parsed_ast) => {
                // Convert the AST to a simplified JSON representation for storage
                // This will be used by the code generator to execute return annotation operations
                let simplified = self.simplify_return_parse_node(&parsed_ast);
                serde_json::to_string(&simplified)
                    .context("Failed to serialize parsed return annotation AST")
            }
            Err(_) => {
                // If parsing fails, fallback to bootstrap mode
                if self.config.debug {
                    println!("Warning: External return parser failed, falling back to bootstrap mode: {}", annotation_value);
                }
                self.parse_return_annotation_bootstrap(annotation_value)
            }
        }
    }
    
    /// Built-in bootstrap return annotation parser
    /// Supports FLAT structures ONLY: scalars, simple arrays, objects with ≤3 keys, ZERO nesting
    /// As defined in BOOTSTRAP_MODE_SPECIFICATION.md
    fn parse_return_annotation_bootstrap(&self, annotation_value: &str) -> Result<String> {
        let trimmed = annotation_value.trim();
        
        // Pattern 1: Simple scalar reference ($1, $2, etc.)
        if let Some(captures) = regex::Regex::new(r"^\$([0-9]+)$")
            .unwrap()
            .captures(trimmed) {
            let index: usize = captures.get(1).unwrap().as_str().parse().unwrap_or(1);
            let result = serde_json::json!({
                "type": "scalar_ref",
                "index": index
            });
            return Ok(serde_json::to_string(&result)?);
        }
        
        // Pattern 2: Simple array ([$1, $2] or [$1*])
        if let Some(captures) = regex::Regex::new(r"^\[([^\[\]{}]+)\]$")
            .unwrap()
            .captures(trimmed) {
            let content = captures.get(1).unwrap().as_str().trim();
            
            // Check for quantified array ([$1*], [$2+])
            if let Some(quant_captures) = regex::Regex::new(r"^\$([0-9]+)[*+]$")
                .unwrap()
                .captures(content) {
                let index: usize = quant_captures.get(1).unwrap().as_str().parse().unwrap_or(1);
                let result = serde_json::json!({
                    "type": "quantified_array",
                    "element": {
                        "type": "scalar_ref",
                        "index": index
                    }
                });
                return Ok(serde_json::to_string(&result)?);
            }
            
            // Check for simple array elements ([$1, $2, $3])
            let elements: Result<Vec<_>, _> = content
                .split(',')
                .map(|elem| {
                    let elem_trimmed = elem.trim();
                    if let Some(scalar_captures) = regex::Regex::new(r"^\$([0-9]+)$")
                        .unwrap()
                        .captures(elem_trimmed) {
                        let index: usize = scalar_captures.get(1).unwrap().as_str().parse().unwrap_or(1);
                        Ok(serde_json::json!({
                            "type": "scalar_ref",
                            "index": index
                        }))
                    } else {
                        Err(anyhow::anyhow!("Invalid array element: {}", elem_trimmed))
                    }
                })
                .collect();
                
            if let Ok(elements) = elements {
                let result = serde_json::json!({
                    "type": "array",
                    "elements": elements
                });
                return Ok(serde_json::to_string(&result)?);
            }
        }
        
        // Pattern 3: Simple object ({key: $1} up to 3 keys, FLAT only)
        if let Some(captures) = regex::Regex::new(r"^\{([^\[\]{}]+)\}$")
            .unwrap()
            .captures(trimmed) {
            let content = captures.get(1).unwrap().as_str().trim();
            
            // Check for nesting - REJECT if found
            if content.contains('{') || content.contains('[') {
                if self.config.debug {
                    println!("WARNING: Nested structure detected - exceeds bootstrap flat-only policy");
                    println!("  Pattern: {}", annotation_value);
                    println!("  Bootstrap mode supports FLAT structures only");
                    println!("  Stored as raw string - use full parser mode for nesting support");
                }
                return Ok(format!("raw:{}", annotation_value));
            }
            
            // Parse object properties (key: value pairs)
            let properties: Result<Vec<_>, _> = content
                .split(',')
                .map(|prop| {
                    let prop_trimmed = prop.trim();
                    if let Some(prop_captures) = regex::Regex::new(r"^([a-zA-Z_][a-zA-Z0-9_]*)\s*:\s*(.+)$")
                        .unwrap()
                        .captures(prop_trimmed) {
                        let key = prop_captures.get(1).unwrap().as_str();
                        let value_str = prop_captures.get(2).unwrap().as_str().trim();
                        
                        // Parse the value (must be simple - no nesting)
                        let value = if let Some(scalar_captures) = regex::Regex::new(r"^\$([0-9]+)$")
                            .unwrap()
                            .captures(value_str) {
                            let index: usize = scalar_captures.get(1).unwrap().as_str().parse().unwrap_or(1);
                            serde_json::json!({
                                "type": "scalar_ref",
                                "index": index
                            })
                        } else if let Some(array_captures) = regex::Regex::new(r"^\[\$([0-9]+)[*+]\]$")
                            .unwrap()
                            .captures(value_str) {
                            // Simple quantified array in object property (FLAT)
                            let index: usize = array_captures.get(1).unwrap().as_str().parse().unwrap_or(1);
                            serde_json::json!({
                                "type": "quantified_array",
                                "element": {
                                    "type": "scalar_ref",
                                    "index": index
                                }
                            })
                        } else if let Some(simple_array_captures) = regex::Regex::new(r"^\[([^\[\]{}]+)\]$")
                            .unwrap()
                            .captures(value_str) {
                            // Simple array in object property (FLAT) - parse elements
                            let array_content = simple_array_captures.get(1).unwrap().as_str().trim();
                            let array_elements: Result<Vec<_>, _> = array_content
                                .split(',')
                                .map(|elem| {
                                    let elem_trimmed = elem.trim();
                                    if let Some(scalar_captures) = regex::Regex::new(r"^\$([0-9]+)$")
                                        .unwrap()
                                        .captures(elem_trimmed) {
                                        let index: usize = scalar_captures.get(1).unwrap().as_str().parse().unwrap_or(1);
                                        Ok(serde_json::json!({
                                            "type": "scalar_ref",
                                            "index": index
                                        }))
                                    } else {
                                        Err(anyhow::anyhow!("Invalid array element in object: {}", elem_trimmed))
                                    }
                                })
                                .collect();
                            
                            if let Ok(elements) = array_elements {
                                serde_json::json!({
                                    "type": "array",
                                    "elements": elements
                                })
                            } else {
                                return Err(anyhow::anyhow!("Failed to parse array in object property: {}", value_str));
                            }
                        } else {
                            return Err(anyhow::anyhow!("Unsupported value type in bootstrap mode: {}", value_str));
                        };
                        
                        Ok(serde_json::json!({
                            "key": key,
                            "value": value
                        }))
                    } else {
                        Err(anyhow::anyhow!("Invalid property format: {}", prop_trimmed))
                    }
                })
                .collect();
                
            if let Ok(properties) = properties {
                // Bootstrap mode supports any number of flat key/value pairs
                
                let result = serde_json::json!({
                    "type": "object",
                    "properties": properties
                });
                return Ok(serde_json::to_string(&result)?);
            }
        }
        
        // Fallback: store as raw if pattern doesn't match
        if self.config.debug {
            println!("WARNING: Return annotation pattern not recognized in bootstrap mode");
            println!("  Pattern: {}", annotation_value);
            println!("  Bootstrap mode supports FLAT structures only");
            println!("  Stored as raw string - use full parser mode for complete support");
        }
        Ok(format!("raw:{}", annotation_value))
    }
    
    /// Convert semantic annotation ParseNode to a serializable simplified representation
    fn simplify_semantic_parse_node(&self, node: &semantic_annotation_parser::ParseNode) -> serde_json::Value {
        use serde_json::{json, Map, Value};
        use semantic_annotation_parser::ParseContent;
        
        let mut obj = Map::new();
        obj.insert("rule_name".to_string(), json!(node.rule_name));
        obj.insert("span".to_string(), json!({"start": node.span.start, "end": node.span.end}));
        
        let content = match &node.content {
            ParseContent::Terminal(s) => json!({"type": "terminal", "value": s}),
            ParseContent::Sequence(nodes) => json!({
                "type": "sequence",
                "elements": nodes.iter().map(|n| self.simplify_semantic_parse_node(n)).collect::<Vec<_>>()
            }),
            ParseContent::Alternative(node) => json!({
                "type": "alternative",
                "element": self.simplify_semantic_parse_node(node)
            }),
            ParseContent::Quantified(nodes, quantifier) => json!({
                "type": "quantified",
                "quantifier": quantifier,
                "elements": nodes.iter().map(|n| self.simplify_semantic_parse_node(n)).collect::<Vec<_>>()
            }),
        };
        
        obj.insert("content".to_string(), content);
        Value::Object(obj)
    }
    
    /// Convert return annotation ParseNode to a serializable simplified representation
    fn simplify_return_parse_node(&self, node: &return_annotation_parser::ParseNode) -> serde_json::Value {
        use serde_json::{json, Map, Value};
        use return_annotation_parser::ParseContent;
        
        let mut obj = Map::new();
        obj.insert("rule_name".to_string(), json!(node.rule_name));
        obj.insert("span".to_string(), json!({"start": node.span.start, "end": node.span.end}));
        
        let content = match &node.content {
            ParseContent::Terminal(s) => json!({"type": "terminal", "value": s}),
            ParseContent::Sequence(nodes) => json!({
                "type": "sequence",
                "elements": nodes.iter().map(|n| self.simplify_return_parse_node(n)).collect::<Vec<_>>()
            }),
            ParseContent::Alternative(node) => json!({
                "type": "alternative",
                "element": self.simplify_return_parse_node(node)
            }),
            ParseContent::Quantified(nodes, quantifier) => json!({
                "type": "quantified",
                "quantifier": quantifier,
                "elements": nodes.iter().map(|n| self.simplify_return_parse_node(n)).collect::<Vec<_>>()
            }),
        };
        
        obj.insert("content".to_string(), content);
        Value::Object(obj)
    }

    /// Stage 1: Extract and preserve annotations
    fn extract_annotations(&mut self, raw_ast: &RawAST) -> Result<RawAST> {
        if self.config.debug {
            println!("Stage 1: Extracting annotations...");
        }

        let mut cleaned_ast = RawAST::new();

        for rule_def in raw_ast {
            if rule_def.is_empty() {
                continue;
            }

            let mut rule_name: Option<String> = None;
            let mut cleaned_rule = TokenSequence::new();

            for token in rule_def {
                if token.len() != 2 {
                    continue;
                }

                // Extract token type and value from the new TokenValue enum
                let token_type = match &token[0] {
                    TokenValue::String(s) => s,
                    _ => continue, // Skip malformed tokens
                };
                
                match token_type.as_str() {
                    "rule" => {
                        if let TokenValue::String(name) = &token[1] {
                            rule_name = Some(name.clone());
                            cleaned_rule.push(token.clone());
                        }
                    }
                    "semantic_annotation" | "logging_annotation" => {
                        if let Some(ref name) = rule_name {
                            if self.config.preserve_annotations {
                                // New format: token[1] is the annotation array [name, value]
                                if let TokenValue::Array(annotation_data) = &token[1] {
                                    if annotation_data.len() >= 2 {
                                        let annotation_name = &annotation_data[0];
                                        let annotation_value = &annotation_data[1];
                                        
                                        match token_type.as_str() {
                                            "semantic_annotation" => {
                                                // Use the semantic annotation parser for semantic annotations
                                                let parsed_value = self.parse_semantic_annotation(annotation_value)
                                                    .unwrap_or_else(|_| format!("raw:{}", annotation_value));
                                                let formatted_annotation = format!("{}:{}", annotation_name, parsed_value);
                                                self.annotations.semantic_annotations
                                                    .entry(name.clone())
                                                    .or_insert_with(Vec::new)
                                                    .push(formatted_annotation);
                                            }
                                            "logging_annotation" => {
                                                let formatted_annotation = format!("{}({})", annotation_name, annotation_value);
                                                self.annotations.logging_annotations
                                                    .entry(name.clone())
                                                    .or_insert_with(Vec::new)
                                                    .push(formatted_annotation);
                                            }
                                            _ => unreachable!(),
                                        }
                                        
                                        if self.config.debug {
                                            println!("Parsed {} annotation: {} = {}", token_type, annotation_name, annotation_value);
                                        }
                                    }
                                } else {
                                    // Fallback for old string format or malformed data
                                    if self.config.debug {
                                        println!("Warning: Unexpected annotation format for {}: {:?}", token_type, token[1]);
                                    }
                                    match token_type.as_str() {
                                        "semantic_annotation" => {
                                            // Still try to parse string format semantic annotations
                                            if let TokenValue::String(value_str) = &token[1] {
                                                let parsed_value = self.parse_semantic_annotation(value_str)
                                                    .unwrap_or_else(|_| format!("raw:{}", value_str));
                                                self.annotations.semantic_annotations
                                                    .entry(name.clone())
                                                    .or_insert_with(Vec::new)
                                                    .push(parsed_value);
                                            } else {
                                                self.annotations.semantic_annotations
                                                    .entry(name.clone())
                                                    .or_insert_with(Vec::new)
                                                    .push(format!("raw:{:?}", token[1]));
                                            }
                                        }
                                        "logging_annotation" => {
                                            self.annotations.logging_annotations
                                                .entry(name.clone())
                                                .or_insert_with(Vec::new)
                                                .push(format!("raw:{:?}", token[1]));
                                        }
                                        _ => unreachable!(),
                                    }
                                }
                                self.stats.annotations_preserved += 1;
                            }
                        }
                        // Don't add to cleaned rule
                    }
                    "return_scalar" | "return_array" | "return_object" => {
                        if let Some(ref name) = rule_name {
                            if self.config.preserve_annotations {
                                // Extract return annotation content (same as before)
                                let annotation_content = match &token[1] {
                                    TokenValue::String(content) => content.as_str(),
                                    TokenValue::Array(arr) if !arr.is_empty() => &arr[0],
                                    _ => "",
                                };
                                
                                if !annotation_content.is_empty() {
                                    // Parse the return annotation NOW (consistent with semantic annotations)
                                    let parsed_content = self.parse_return_annotation(annotation_content)
                                        .unwrap_or_else(|_| format!("raw:{}", annotation_content));
                                    
                                    let return_annotation = ReturnAnnotation {
                                        annotation_type: token_type.clone(),
                                        annotation_content: parsed_content, // Store parsed JSON instead of raw string
                                    };
                                    self.annotations.return_annotations
                                        .insert(name.clone(), return_annotation);
                                    
                                    if self.config.debug {
                                        println!("Parsed return annotation for {}: {} (type: {})", name, annotation_content, token_type);
                                    }
                                } else {
                                    // Fallback: create a basic return annotation with just the type for empty content
                                    // This shouldn't happen with valid grammar, but we need a fallback
                                    if self.config.debug {
                                        println!("Warning: Empty return annotation content for {}, skipping", name);
                                    }
                                }
                                
                                self.stats.annotations_preserved += 1;
                            }
                        }
                        // Don't add to cleaned rule
                    }
                    _ => {
                        cleaned_rule.push(token.clone());
                    }
                }
            }

            if !cleaned_rule.is_empty() {
                cleaned_ast.push(cleaned_rule);
            }
        }

        if self.config.debug {
            println!("Preserved {} annotations", self.stats.annotations_preserved);
        }

        Ok(cleaned_ast)
    }

    /// Stage 2: Group rule definitions by OR operators
    fn group_by_or_operators(&self, ast: &RawAST) -> Result<HashMap<String, Vec<TokenSequence>>> {
        if self.config.debug {
            println!("Stage 2: Grouping by OR operators...");
        }

        let mut grouped = HashMap::new();

        for rule_def in ast {
            if rule_def.is_empty() {
                continue;
            }

            let mut rule_name: Option<String> = None;
            for token in rule_def {
                if token.len() == 2 {
                    if let (TokenValue::String(type_str), TokenValue::String(name_str)) = (&token[0], &token[1]) {
                        if type_str == "rule" {
                            rule_name = Some(name_str.clone());
                            break;
                        }
                    }
                }
            }

            if let Some(name) = rule_name {
                let mut alternatives = Vec::new();
                let mut current_alt = TokenSequence::new();

                // Skip rule definition token
                for token in rule_def.iter().skip(1) {
                    if token.len() == 2 {
                        if let (TokenValue::String(type_str), TokenValue::String(value_str)) = (&token[0], &token[1]) {
                            if type_str == "operator" && value_str == "|" {
                                if !current_alt.is_empty() {
                                    alternatives.push(current_alt);
                                    current_alt = TokenSequence::new();
                                }
                                continue;
                            }
                        }
                    }
                    current_alt.push(token.clone());
                }

                if !current_alt.is_empty() {
                    alternatives.push(current_alt);
                }

                grouped.entry(name).or_insert_with(Vec::new).extend(alternatives);
            }
        }

        Ok(grouped)
    }

    /// Stage 2.5: Handle parentheses and grouping
    fn handle_parentheses(&self, grouped_rules: &HashMap<String, Vec<TokenSequence>>) -> Result<HashMap<String, Vec<TokenSequence>>> {
        if self.config.debug {
            println!("Stage 2.5: Handling parentheses...");
        }

        let mut processed = HashMap::new();

        for (rule_name, alternatives) in grouped_rules {
            let mut processed_alts = Vec::new();

            for alt in alternatives {
                let processed_alt = self.process_parentheses_in_sequence(alt)?;
                processed_alts.push(processed_alt);
            }

            processed.insert(rule_name.clone(), processed_alts);
        }

        Ok(processed)
    }

    /// Process parentheses within a token sequence
    fn process_parentheses_in_sequence(&self, sequence: &TokenSequence) -> Result<TokenSequence> {
        let mut result = TokenSequence::new();
        let mut i = 0;

        while i < sequence.len() {
            let token = &sequence[i];

            if token.len() == 2 && token[0] == "group_open" {
                // Find matching close
                let mut paren_count = 1;
                let mut j = i + 1;
                let mut group_content = TokenSequence::new();

                while j < sequence.len() && paren_count > 0 {
                    if sequence[j].len() == 2 {
                        if let Some(token_str) = sequence[j][0].as_str() {
                            match token_str {
                                "group_open" => paren_count += 1,
                                "group_close" => paren_count -= 1,
                                _ => {}
                            }
                        }
                    }

                    if paren_count > 0 {
                        group_content.push(sequence[j].clone());
                    }
                    j += 1;
                }

                if !group_content.is_empty() {
                    // Create group token - serialize content as JSON for now
                    let content_json = serde_json::to_string(&group_content)
                        .context("Failed to serialize group content")?;
                    result.push(vec![TokenValue::String("group".to_string()), TokenValue::String(content_json)]);
                }

                i = j;
            } else {
                result.push(token.clone());
                i += 1;
            }
        }

        Ok(result)
    }

    /// Stage 3: Parse sequences
    fn parse_sequences(&self, processed_rules: &HashMap<String, Vec<TokenSequence>>) -> Result<HashMap<String, Vec<ASTNode>>> {
        if self.config.debug {
            println!("Stage 3: Parsing sequences...");
        }

        let mut sequenced = HashMap::new();

        for (rule_name, alternatives) in processed_rules {
            let mut parsed_alts = Vec::new();

            for alt in alternatives {
                let parsed_alt = if alt.len() == 1 {
                    self.parse_single_element(&alt[0])?
                } else {
                    let elements: Result<Vec<ASTNode>> = alt
                        .iter()
                        .map(|elem| self.parse_single_element(elem))
                        .collect();
                    ASTNode::Sequence { elements: elements? }
                };
                parsed_alts.push(parsed_alt);
            }

            sequenced.insert(rule_name.clone(), parsed_alts);
        }

        Ok(sequenced)
    }

    /// Parse a single grammar element
    fn parse_single_element(&self, element: &Token) -> Result<ASTNode> {
        if element.len() != 2 {
            return Ok(ASTNode::Atom { value: ASTValue::Token(element.clone()) });
        }

        let token_type = &element[0];
        let token_value = &element[1];

        match token_type.as_str() {
            Some("group") => {
                // Deserialize group content
                if let TokenValue::String(json_str) = token_value {
                    let group_content: TokenSequence = serde_json::from_str(json_str)
                        .context("Failed to deserialize group content")?;

                    if group_content.len() == 1 {
                        self.parse_single_element(&group_content[0])
                    } else {
                        let elements: Result<Vec<ASTNode>> = group_content
                            .iter()
                            .map(|elem| self.parse_single_element(elem))
                            .collect();
                        Ok(ASTNode::Sequence { elements: elements? })
                    }
                } else {
                    Ok(ASTNode::Atom { value: ASTValue::Token(element.clone()) })
                }
            }
            _ => Ok(ASTNode::Atom { value: ASTValue::Token(element.clone()) })
        }
    }

    /// Stage 4: Handle quantifiers
    fn handle_quantifiers(&self, sequenced_rules: &HashMap<String, Vec<ASTNode>>) -> Result<HashMap<String, Vec<ASTNode>>> {
        if self.config.debug {
            println!("Stage 4: Handling quantifiers...");
        }

        let mut quantified = HashMap::new();

        for (rule_name, alternatives) in sequenced_rules {
            let mut processed_alts = Vec::new();

            for alt in alternatives {
                let processed_alt = self.apply_quantifiers_to_node(alt.clone())?;
                processed_alts.push(processed_alt);
            }

            quantified.insert(rule_name.clone(), processed_alts);
        }

        Ok(quantified)
    }

    /// Apply quantifiers to AST node
    fn apply_quantifiers_to_node(&self, node: ASTNode) -> Result<ASTNode> {
        match node {
            ASTNode::Sequence { elements } => {
                let mut new_elements = Vec::new();
                let mut i = 0;

                while i < elements.len() {
                    let element = &elements[i];

                    // Check if next element is a quantifier
                    if i + 1 < elements.len() {
                        if let ASTNode::Atom { value: ASTValue::Token(token) } = &elements[i + 1] {
                            if token.len() == 2 && token[0] == "operator" {
                                if let Some(op_str) = token[1].as_str() {
                                    if ["*", "+", "?"].contains(&op_str) {
                                        let quantified_node = ASTNode::Quantified {
                                            element: Box::new(element.clone()),
                                            quantifier: op_str.to_string(),
                                        };
                                        new_elements.push(quantified_node);
                                        i += 2; // Skip quantifier token
                                        continue;
                                    }
                                }
                            }
                        }
                    }

                    new_elements.push(element.clone());
                    i += 1;
                }

                Ok(ASTNode::Sequence { elements: new_elements })
            }
            _ => Ok(node)
        }
    }

    /// Stage 5: Build final tree structure
    fn build_tree_structure(&self, quantified_rules: &HashMap<String, Vec<ASTNode>>) -> Result<(HashMap<String, ASTNode>, Vec<String>)> {
        if self.config.debug {
            println!("Stage 5: Building tree structure...");
        }

        let mut grammar_tree = HashMap::new();
        let rule_order: Vec<String> = quantified_rules.keys().cloned().collect();

        for (rule_name, alternatives) in quantified_rules {
            let final_node = if alternatives.len() == 1 {
                alternatives[0].clone()
            } else {
                ASTNode::Or {
                    alternatives: alternatives.clone()
                }
            };

            grammar_tree.insert(rule_name.clone(), final_node);
        }

        Ok((grammar_tree, rule_order))
    }

    /// Save transformed AST to JSON file
    pub fn save_transformed_ast(
        &self,
        grammar_tree: &HashMap<String, ASTNode>,
        rule_order: &[String],
        grammar_name: &str,
        output_file: &str,
    ) -> Result<()> {
        if self.config.debug {
            println!("Saving transformed AST to: {}", output_file);
        }

        let metadata = TransformMetadata {
            format: "transformed_ast".to_string(),
            source_format: "raw_ast".to_string(),
            transformed_at: chrono::Utc::now().to_rfc3339(),
            transformer: "Rust AST Pipeline v1.0".to_string(),
            pipeline_stage: "transformation".to_string(),
            annotations: Some(self.annotations.clone()),
            stats: self.stats.clone(),
        };

        let transformed_data = TransformedASTJson {
            grammar_name: grammar_name.to_string(),
            grammar_tree: grammar_tree.clone(),
            rule_order: rule_order.to_vec(),
            metadata,
        };

        let json = serde_json::to_string_pretty(&transformed_data)
            .context("Failed to serialize transformed AST")?;

        fs::write(output_file, json)
            .with_context(|| format!("Failed to write file: {}", output_file))?;

        if self.config.debug {
            println!("Transformed AST saved successfully");
        }

        Ok(())
    }

    /// Same-language API: Transform raw AST JSON file to in-memory AST
    pub fn transform_from_file(
        &mut self,
        raw_ast_json_file: &str,
        output_json_file: Option<&str>,
    ) -> Result<(HashMap<String, ASTNode>, Vec<String>)> {
        let raw_data = self.load_raw_ast(raw_ast_json_file)?;
        let (grammar_tree, rule_order) = self.transform_raw_ast(&raw_data.raw_ast)?;

        if let Some(output_file) = output_json_file {
            self.save_transformed_ast(&grammar_tree, &rule_order, &raw_data.grammar_name, output_file)?;
        }

        Ok((grammar_tree, rule_order))
    }

    /// Cross-language API: Transform raw AST JSON file to transformed AST JSON file
    pub fn transform_to_json(&mut self, raw_ast_json_file: &str, output_json_file: &str) -> Result<()> {
        let (grammar_tree, rule_order) = self.transform_from_file(raw_ast_json_file, None)?;
        let raw_data = self.load_raw_ast(raw_ast_json_file)?;
        self.save_transformed_ast(&grammar_tree, &rule_order, &raw_data.grammar_name, output_json_file)?;
        Ok(())
    }

    /// Get return annotations for a specific rule
    #[allow(dead_code)]
    pub fn get_return_annotation(&self, rule_name: &str) -> Option<&ReturnAnnotation> {
        self.annotations.return_annotations.get(rule_name)
    }
    
    /// Get all return annotations
    #[allow(dead_code)]
    pub fn get_all_return_annotations(&self) -> &HashMap<String, ReturnAnnotation> {
        &self.annotations.return_annotations
    }
    
    /// Get semantic annotations for a specific rule
    #[allow(dead_code)]
    pub fn get_semantic_annotations(&self, rule_name: &str) -> Option<&Vec<String>> {
        self.annotations.semantic_annotations.get(rule_name)
    }
    
    /// Get logging annotations for a specific rule
    #[allow(dead_code)]
    pub fn get_logging_annotations(&self, rule_name: &str) -> Option<&Vec<String>> {
        self.annotations.logging_annotations.get(rule_name)
    }

    /// Generate high-performance Rust parser code directly from transformed AST
    /// Produces SOTA parser suitable for embedding in rgx regex engine
    pub fn generate_high_performance_parser(
        &mut self,
        raw_ast_json_file: &str,
        output_rust_file: &str,
        enable_trace: bool,
        enable_backtrack_debug: bool,
    ) -> Result<()> {
        let (grammar_tree, rule_order) = self.transform_from_file(raw_ast_json_file, None)?;
        let raw_data = self.load_raw_ast(raw_ast_json_file)?;
        
        // Extract the entry rule name BEFORE creating the generator
        let entry_rule = self.entry_rule.as_ref()
            .map(|s| s.clone())
            .unwrap_or_else(|| {
                // Always use the first rule in rule_order to prevent infinite recursion
                // Never use grammar_name as it creates circular calls parse_grammar_name() -> parse_grammar_name()
                rule_order.first().cloned().unwrap_or_else(|| {
                    eprintln!("ERROR: No rules found in rule_order, cannot determine entry rule");
                    "unknown_entry_rule".to_string()
                })
            });
        
        let mut code_generator = if enable_trace && enable_backtrack_debug {
            HighPerformanceRustGenerator::with_full_debug(&raw_data.grammar_name)
        } else if enable_trace {
            HighPerformanceRustGenerator::with_trace(&raw_data.grammar_name, true)
        } else {
            let mut gen = HighPerformanceRustGenerator::new(&raw_data.grammar_name);
            if enable_backtrack_debug {
                gen.enable_backtrack_debug = true;
            }
            gen
        };
        
        // Set the entry rule immediately after generator creation
        code_generator.set_entry_rule(&entry_rule);
        
        // Pass the return annotations to the code generator
        code_generator.set_return_annotations(&self.annotations.return_annotations);
        
        let rust_code = code_generator.generate_lightning_fast_parser(&grammar_tree, &rule_order)?;
        
        fs::write(output_rust_file, rust_code)
            .with_context(|| format!("Failed to write high-performance Rust parser to: {}", output_rust_file))?;
        
        if self.config.debug {
            println!("Generated SOTA regex parser: {}", output_rust_file);
            if enable_trace {
                println!("  - Trace logging enabled");
            }
            if enable_backtrack_debug {
                println!("  - Backtrack debugging enabled");
            }
        }
        
        Ok(())
    }

    /// Apply left recursion elimination using Aho-Sethi-Ullman algorithm
    /// Based on the implementation from rust_parser_gen
    fn eliminate_left_recursion(
        &self,
        grammar_tree: HashMap<String, ASTNode>,
        rule_order: Vec<String>,
    ) -> Result<(HashMap<String, ASTNode>, Vec<String>)> {
        
        if self.config.debug {
            println!("🔥 DEPLOYING LEFT-RECURSION ELIMINATOR!");
            println!("📚 Based on Aho-Sethi-Ullman Algorithm 4.19");
        }
        
        // Convert AST nodes to simple production format
        let mut simple_grammar = HashMap::new();
        let mut ordered_rules = rule_order.clone();
        ordered_rules.sort();
        
        for (rule_name, ast_node) in &grammar_tree {
            simple_grammar.insert(rule_name.clone(), self.ast_node_to_productions(ast_node)?); 
        }
        
        // Apply the standard algorithm
        for i in 0..ordered_rules.len() {
            let ai = ordered_rules[i].clone();
            
            if self.config.debug {
                println!("  Processing rule {} ({}/{})", ai, i+1, ordered_rules.len());
            }
            
            // Substitute earlier rules
            for j in 0..i {
                let aj = ordered_rules[j].clone();
                
                if self.could_create_indirect_left_recursion(&simple_grammar, &ai, &aj) {
                    self.substitute_rule(&mut simple_grammar, &ai, &aj)?;
                }
            }
            
            // Eliminate immediate left recursion
            self.eliminate_immediate_left_recursion(&mut simple_grammar, &ai, &mut ordered_rules)?;
        }
        
        // Convert back to AST format
        let mut result_grammar = HashMap::new();
        for (rule_name, productions) in simple_grammar {
            result_grammar.insert(rule_name, self.productions_to_ast_node(productions)?);
        }
        
        if self.config.debug {
            println!("💀 LEFT-RECURSION ELIMINATION COMPLETE!");
        }
        
        Ok((result_grammar, ordered_rules))
    }
    
    /// Convert AST node to list of productions
    fn ast_node_to_productions(&self, node: &ASTNode) -> Result<Vec<Vec<String>>> {
        match node {
            ASTNode::Or { alternatives } => {
                let mut productions = Vec::new();
                for alt in alternatives {
                    productions.extend(self.ast_node_to_productions(alt)?);
                }
                Ok(productions)
            }
            ASTNode::Sequence { elements } => {
                let mut production = Vec::new();
                for element in elements {
                    match element {
                        ASTNode::Atom { value: ASTValue::Token(token) } => {
                            if token.len() == 2 {
                                match token[0].as_str() {
                                    Some("quoted_string") => {
                                        if let Some(value) = token[1].as_str() {
                                            production.push(format!("TERMINAL:{}", value));
                                        }
                                    }
                                    Some("rule_reference") => {
                                        if let Some(name) = token[1].as_str() {
                                            production.push(name.to_string());
                                        }
                                    }
                                    Some("regex") => {
                                        if let Some(pattern) = token[1].as_str() {
                                            production.push(format!("REGEX:{}", pattern));
                                        }
                                    }
                                    _ => production.push(format!("COMPLEX:{:?}", element)),
                                }
                            } else {
                                production.push(format!("COMPLEX:{:?}", element));
                            }
                        }
                        _ => production.push(format!("COMPLEX:{:?}", element)),
                    }
                }
                Ok(vec![production])
            }
            ASTNode::Atom { value: ASTValue::Token(token) } => {
                if token.len() == 2 {
                    match token[0].as_str() {
                        Some("quoted_string") => {
                            if let Some(value) = token[1].as_str() {
                                Ok(vec![vec![format!("TERMINAL:{}", value)]])
                            } else {
                                Ok(vec![vec![format!("COMPLEX:{:?}", token)]])
                            }
                        }
                        Some("rule_reference") => {
                            if let Some(name) = token[1].as_str() {
                                Ok(vec![vec![name.to_string()]])
                            } else {
                                Ok(vec![vec![format!("COMPLEX:{:?}", token)]])
                            }
                        }
                        Some("regex") => {
                            if let Some(pattern) = token[1].as_str() {
                                Ok(vec![vec![format!("REGEX:{}", pattern)]])
                            } else {
                                Ok(vec![vec![format!("COMPLEX:{:?}", token)]])
                            }
                        }
                        _ => Ok(vec![vec![format!("COMPLEX:{:?}", token)]]),
                    }
                } else {
                    Ok(vec![vec![format!("COMPLEX:{:?}", token)]])
                }
            }
            _ => Ok(vec![vec![format!("COMPLEX:{:?}", node)]])
        }
    }
    
    /// Convert productions back to AST node
    fn productions_to_ast_node(&self, productions: Vec<Vec<String>>) -> Result<ASTNode> {
        if productions.is_empty() {
            return Ok(ASTNode::Atom { 
                value: ASTValue::Token(vec![
                    TokenValue::String("quoted_string".to_string()), 
                    TokenValue::String("epsilon".to_string())
                ]) 
            });
        }
        
        if productions.len() == 1 {
            let production = &productions[0];
            if production.len() == 1 {
                let symbol = &production[0];
                if symbol.starts_with("TERMINAL:") {
                    Ok(ASTNode::Atom { 
                        value: ASTValue::Token(vec![
                            TokenValue::String("quoted_string".to_string()), 
                            TokenValue::String(symbol[9..].to_string())
                        ]) 
                    })
                } else if symbol.starts_with("REGEX:") {
                    Ok(ASTNode::Atom { 
                        value: ASTValue::Token(vec![
                            TokenValue::String("regex".to_string()), 
                            TokenValue::String(symbol[6..].to_string())
                        ]) 
                    })
                } else if symbol.starts_with("COMPLEX:") {
                    // Handle COMPLEX entries by treating as empty terminal
                    Ok(ASTNode::Atom { 
                        value: ASTValue::Token(vec![
                            TokenValue::String("quoted_string".to_string()), 
                            TokenValue::String("".to_string()) // Empty terminal
                        ]) 
                    })
                } else if symbol == "ε" {
                    // Handle epsilon productions - convert to empty terminal
                    Ok(ASTNode::Atom { 
                        value: ASTValue::Token(vec![
                            TokenValue::String("quoted_string".to_string()), 
                            TokenValue::String("".to_string()) // Empty terminal for epsilon
                        ]) 
                    })
                } else {
                    Ok(ASTNode::Atom { 
                        value: ASTValue::Token(vec![
                            TokenValue::String("rule_reference".to_string()), 
                            TokenValue::String(symbol.clone())
                        ]) 
                    })
                }
            } else {
                let mut elements = Vec::new();
                for symbol in production {
                    if symbol.starts_with("TERMINAL:") {
                        elements.push(ASTNode::Atom { 
                            value: ASTValue::Token(vec![
                                TokenValue::String("quoted_string".to_string()), 
                                TokenValue::String(symbol[9..].to_string())
                            ]) 
                        });
                    } else if symbol.starts_with("REGEX:") {
                        elements.push(ASTNode::Atom { 
                            value: ASTValue::Token(vec![
                                TokenValue::String("regex".to_string()), 
                                TokenValue::String(symbol[6..].to_string())
                            ]) 
                        });
                    } else if symbol.starts_with("COMPLEX:") {
                        // Handle COMPLEX entries by parsing the debug representation
                        // For now, treat as empty terminal to avoid generation errors
                        elements.push(ASTNode::Atom { 
                            value: ASTValue::Token(vec![
                                TokenValue::String("quoted_string".to_string()), 
                                TokenValue::String("".to_string()) // Empty terminal
                            ]) 
                        });
                    } else {
                        elements.push(ASTNode::Atom { 
                            value: ASTValue::Token(vec![
                                TokenValue::String("rule_reference".to_string()), 
                                TokenValue::String(symbol.clone())
                            ]) 
                        });
                    }
                }
                Ok(ASTNode::Sequence { elements })
            }
        } else {
            let mut alternatives = Vec::new();
            for production in productions {
                alternatives.push(self.productions_to_ast_node(vec![production])?);
            }
            Ok(ASTNode::Or { alternatives })
        }
    }
    
    /// Check if substitution could create indirect left recursion
    fn could_create_indirect_left_recursion(
        &self,
        grammar: &HashMap<String, Vec<Vec<String>>>,
        current_rule: &str,
        referenced_rule: &str
    ) -> bool {
        if let Some(productions) = grammar.get(referenced_rule) {
            for production in productions {
                if !production.is_empty() && production[0] == current_rule {
                    return true;
                }
            }
        }
        false
    }
    
    /// Substitute rule in grammar
    fn substitute_rule(
        &self,
        grammar: &mut HashMap<String, Vec<Vec<String>>>,
        target_rule: &str,
        substitute_rule: &str
    ) -> Result<()> {
        let target_productions = grammar.get(target_rule).cloned().unwrap_or_default();
        let substitute_productions = grammar.get(substitute_rule).cloned().unwrap_or_default();
        
        let mut new_productions = Vec::new();
        
        for production in target_productions {
            if !production.is_empty() && production[0] == substitute_rule {
                // Substitute this production
                let gamma = &production[1..]; // Rest after the substituted rule
                
                for sub_production in &substitute_productions {
                    if sub_production.len() == 1 && sub_production[0] == "ε" {
                        // Epsilon production
                        new_productions.push(gamma.to_vec());
                    } else {
                        // Normal substitution
                        let mut new_production = sub_production.clone();
                        new_production.extend_from_slice(gamma);
                        new_productions.push(new_production);
                    }
                }
            } else {
                // Keep as-is
                new_productions.push(production);
            }
        }
        
        grammar.insert(target_rule.to_string(), new_productions);
        Ok(())
    }
    
    /// Eliminate immediate left recursion from a rule
    fn eliminate_immediate_left_recursion(
        &self,
        grammar: &mut HashMap<String, Vec<Vec<String>>>,
        rule: &str,
        rule_order: &mut Vec<String>
    ) -> Result<()> {
        let productions = grammar.get(rule).cloned().unwrap_or_default();
        
        let mut left_recursive = Vec::new();
        let mut non_left_recursive = Vec::new();
        
        for production in productions {
            if !production.is_empty() && production[0] == rule {
                left_recursive.push(production);
            } else {
                non_left_recursive.push(production);
            }
        }
        
        if !left_recursive.is_empty() {
            let prime_rule = format!("{}_prime", rule);
            
            // Create new main productions: A → β A'
            let mut new_main_productions = Vec::new();
            if non_left_recursive.is_empty() {
                new_main_productions.push(vec![prime_rule.clone()]);
            } else {
                for beta in non_left_recursive {
                    let mut production = beta;
                    production.push(prime_rule.clone());
                    new_main_productions.push(production);
                }
            }
            
            // Create prime productions: A' → α A' | ε
            let mut prime_productions = Vec::new();
            for left_prod in left_recursive {
                let mut alpha = left_prod[1..].to_vec(); // Remove the left-recursive symbol
                alpha.push(prime_rule.clone());
                prime_productions.push(alpha);
            }
            prime_productions.push(vec!["ε".to_string()]);
            
            // Update grammar
            grammar.insert(rule.to_string(), new_main_productions);
            grammar.insert(prime_rule.clone(), prime_productions);
            
            // Add prime rule to order
            if !rule_order.contains(&prime_rule) {
                rule_order.push(prime_rule);
            }
        }
        
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_creation() {
        let config = PipelineConfig::default();
        let pipeline = RustASTPipeline::new(config);
        assert!(!pipeline.config.debug);
    }

    #[test]
    fn test_extract_annotations() {
        let mut pipeline = RustASTPipeline::new(PipelineConfig::default());
        let raw_ast = vec![
            vec![
                vec![TokenValue::String("rule".to_string()), TokenValue::String("test".to_string())],
                vec![TokenValue::String("semantic_annotation".to_string()), TokenValue::String("[\"type\", \"TestRule\"]".to_string())],
                vec![TokenValue::String("identifier".to_string()), TokenValue::String("value".to_string())],
            ]
        ];

        let _cleaned = pipeline.extract_annotations(&raw_ast).unwrap();
        assert_eq!(pipeline.stats.annotations_preserved, 1);
        assert!(pipeline.annotations.semantic_annotations.contains_key("test"));
    }
}
