//! Rust AST Pipeline Implementation
//!
//! Provides complete EBNF AST transformation pipeline with dual-mode API:
//! - Same-language optimization: In-memory data structures
//! - Cross-language interface: JSON input/output
//!
//! Implements the 5-stage transformation pipeline equivalent to Perl AST::Transform.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use anyhow::{Result, Context, anyhow};
use prettyplease::unparse;

pub mod grouped_quantifier_parser;
// Visualization functionality implemented inline to avoid import issues

// TEMPORARILY DISABLED: Generated parsers need regeneration with AST-based generator
// Import the generated semantic annotation parser
// pub mod semantic_annotation_parser {
//     include!("../../generated/semantic_annotation_parser.rs");
// }
// use semantic_annotation_parser::Semantic_annotationParser;

// Import the generated return annotation parser (only when not in bootstrap mode)
pub mod return_annotation_parser {
    include!("../../generated/return_annotation_parser.rs");
}
use return_annotation_parser::Return_annotationParser;

// ⚠️ FORBIDDEN: String-based generator - DO NOT UNCOMMENT!
// This module uses string concatenation for code generation which is FORBIDDEN.
// All code generation MUST use AST-based approach with syn/quote.
// See docs/AST_GENERATOR_MIGRATION.md for features migration status.
// mod high_performance_generator;  // DELETED - used string-based code generation (FORBIDDEN)
// use high_performance_generator::HighPerformanceRustGenerator; // DELETED - never use string-based generation
mod mutual_recursion_handler;
mod return_annotation_handler;
use return_annotation_handler::{ReturnAnnotationHandler, ReturnAnnotationMode};
pub mod unified_return_ast;
use unified_return_ast::UnifiedReturnAST;

// AST-based generator - Using syn/quote for guaranteed syntax correctness
pub mod ast_based_generator;
use ast_based_generator::AstBasedGenerator;
mod ast_code_generator;
pub mod ast_return_transform;
pub mod ast_generator_direct;

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
    pub annotation_content: String, // The original annotation content (for logging/display)
    pub parsed_ast: Option<UnifiedReturnAST>, // The parsed AST ready for code generation
}

/// Preserved annotations from raw AST
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Annotations {
    pub semantic_annotations: HashMap<String, Vec<String>>,
    pub logging_annotations: HashMap<String, Vec<String>>,
    /// Branch-level return annotations: rule_name -> Vec[branch_index] -> Option<annotation>
    /// Each alternative/branch can have its own return annotation
    pub branch_return_annotations: HashMap<String, Vec<Option<ReturnAnnotation>>>,
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
    log_file: Option<BufWriter<std::fs::File>>,
    log_filename: Option<String>,
    /// Track which contexts have already been logged to add empty lines before first occurrence
    logged_contexts: HashSet<String>,
    /// Track the last context that was logged to detect method boundary changes
    last_logged_context: Option<String>,
}

impl RustASTPipeline {
    /// Create new pipeline with configuration
    pub fn new(config: PipelineConfig) -> Self {
        let (log_file, log_filename) = if config.debug {
            Self::create_log_file().unwrap_or((None, None))
        } else {
            (None, None)
        };
        
        let mut pipeline = Self {
            config,
            stats: TransformStats::default(),
            annotations: Annotations::default(),
            entry_rule: None,
            log_file,
            log_filename,
            logged_contexts: HashSet::new(),
            last_logged_context: None,
        };
        
        if pipeline.config.debug {
            pipeline.write_log_header();
        }
        
        pipeline
    }
    
    /// Create new pipeline with left recursion elimination enabled
    /// This will help resolve stack overflow issues caused by left-recursive grammars
    #[allow(dead_code)]
    pub fn with_left_recursion_elimination() -> Self {
        let mut config = PipelineConfig::default();
        config.eliminate_left_recursion = true;
        config.debug = true; // Enable debug for logging
        Self::new(config)
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
    
    /// Create a timestamped log file for debug output
    fn create_log_file() -> Result<(Option<BufWriter<std::fs::File>>, Option<String>)> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let filename = format!("ast_pipeline_{}.log", timestamp);
        
        match OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&filename) {
            Ok(file) => {
                let writer = BufWriter::new(file);
                Ok((Some(writer), Some(filename)))
            }
            Err(e) => {
                eprintln!("[ast_pipeline.rs][create_log_file] Warning: Failed to create log file {}: {}", filename, e);
                Ok((None, None))
            }
        }
    }
    
    /// Write a comprehensive header to the log file
    fn write_log_header(&mut self) {
        let header = format!(
            "# AST Pipeline Debug Log\n# Generated: {}\n# Pipeline: Rust AST Pipeline v1.0\n# Debug Mode: {}\n# Trace Mode: {}\n# Bootstrap Mode: {}\n# Left Recursion Elimination: {}\n# Validate Input: {}\n# Validate Output: {}\n# Max Recursion Depth: {}\n\n",
            chrono::Utc::now().to_rfc3339(),
            self.config.debug,
            self.config.trace,
            self.config.bootstrap_mode,
            self.config.eliminate_left_recursion,
            self.config.validate_input,
            self.config.validate_output,
            self.config.max_recursion_depth
        );
        
        self.write_to_log(&header);
    }
    
    /// Unified logging method that writes to both console (if debug) and log file
    fn log_debug(&mut self, context: &str, message: &str) {
        // Check if this is a method context that hasn't been logged before
        // Add empty line before first occurrence of specific method contexts for better readability
        let ast_pipeline_contexts = [
            "extract_annotations", "group_by_or_operators", "handle_parentheses", 
            "parse_sequences", "handle_quantifiers", "apply_quantifiers_to_node",
            "build_tree_structure", "eliminate_left_recursion"
        ];
        
        let generator_contexts = [
            "generate_quantified_group_functions", "generate_lightning_fast_parser",
            "generate_optimized_rule_methods", "generate_optimized_rule_method"
        ];
        
        let all_method_contexts: Vec<&str> = ast_pipeline_contexts.iter().chain(generator_contexts.iter()).copied().collect();
        
        if all_method_contexts.contains(&context) && !self.logged_contexts.contains(context) {
            // Add empty line before first occurrence of this method context
            if self.config.debug {
                println!(""); // Empty line to console
            }
            self.write_to_log("\n"); // Empty line to log file
            self.logged_contexts.insert(context.to_string());
        }
        
        // Determine the correct source file based on the context
        let source_file = if generator_contexts.contains(&context) {
            "ast_based_generator.rs"
        } else {
            "ast_pipeline.rs"
        };
        
        let formatted_message = format!("[{}][{}] {}", source_file, context, message);
        
        if self.config.debug {
            println!("{}", formatted_message);
        }
        
        self.write_to_log(&format!("{} {}\n", 
            chrono::Utc::now().format("%H:%M:%S%.3f"), 
            formatted_message
        ));
    }
    
    /// Write formatted progress indicator to log
    fn log_progress(&mut self, context: &str, step: usize, total: usize, description: &str) {
        let progress_msg = format!(
            "🔄 PROGRESS [{}/{}] {}: {}", 
            step, total, context, description
        );
        
        // Add empty line before PROGRESS messages for better readability
        if self.config.debug {
            println!(""); // Empty line to console
        }
        self.write_to_log("\n"); // Empty line to log file
        
        self.log_debug("PROGRESS", &progress_msg);
    }
    
    /// Write success indicator to log
    fn log_success(&mut self, context: &str, message: &str) {
        self.log_debug(context, &format!("✅ SUCCESS: {}", message));
    }
    
    /// Write failure indicator to log
    fn log_failure(&mut self, context: &str, message: &str) {
        self.log_debug(context, &format!("❌ FAILURE: {}", message));
    }
    
    /// Write informational message to log
    fn log_info(&mut self, context: &str, message: &str) {
        self.log_debug(context, &format!("ℹ️  INFO: {}", message));
    }
    
    /// Write error message to log
    fn log_error(&mut self, context: &str, message: &str) {
        self.log_debug(context, &format!("❌ ERROR: {}", message));
    }
    
    /// Write warning message to log
    fn log_warning(&mut self, context: &str, message: &str) {
        self.log_debug(context, &format!("⚠️  WARNING: {}", message));
    }
    
    /// Internal method to write to log file
    fn write_to_log(&mut self, message: &str) {
        if let Some(ref mut log_file) = self.log_file {
            if let Err(e) = log_file.write_all(message.as_bytes()) {
                if self.config.debug {
                    eprintln!("[ast_pipeline.rs][write_to_log] Warning: Failed to write to log file: {}", e);
                }
            }
            if let Err(e) = log_file.flush() {
                if self.config.debug {
                    eprintln!("[ast_pipeline.rs][write_to_log] Warning: Failed to flush log file: {}", e);
                }
            }
        }
    }
    
    /// Write a summary to the log file at the end of processing
    fn write_log_summary(&mut self) {
        if !self.config.debug {
            return;
        }
        
        let summary = format!(
            "\n\n# AST Pipeline Summary\n# Processing completed: {}\n# Rules processed: {}\n# Annotations preserved: {}\n# Transformations applied: {}\n# Entry rule: {}\n# Log file: {}\n\n",
            chrono::Utc::now().to_rfc3339(),
            self.stats.rules_processed,
            self.stats.annotations_preserved,
            self.stats.transformations_applied,
            self.entry_rule.as_deref().unwrap_or("None"),
            self.log_filename.as_deref().unwrap_or("None")
        );
        
        self.write_to_log(&summary);
        
        if let Some(ref filename) = self.log_filename {
            if self.config.debug {
                println!("[ast_pipeline.rs][write_log_summary] ✅ Complete debug log written to: {}", filename);
            }
        }
    }
    
    /// Flush the log file to ensure all data is written
    fn flush_log(&mut self) {
        if let Some(ref mut log_file) = self.log_file {
            if let Err(e) = log_file.flush() {
                if self.config.debug {
                    eprintln!("[ast_pipeline.rs][flush_log] Warning: Failed to flush log file: {}", e);
                }
            }
        }
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

        let content = std::fs::read_to_string(file_path)
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
        self.log_info("transform_raw_ast", "🚀 STARTING Rust AST Transformation Pipeline");
        self.log_info("transform_raw_ast", &format!("📊 Input contains {} rule definitions", raw_ast.len()));
        
        // Extract the entry rule name for dynamic parser instantiation
        let entry_rule = self.extract_entry_rule(raw_ast)?;
        self.log_success("transform_raw_ast", &format!("📍 Detected entry rule: {}", entry_rule));

        let total_steps = if self.config.eliminate_left_recursion { 6 } else { 5 };
        let mut current_step = 1;
        
        // Stage 1: Extract annotations
        self.log_progress("transform_raw_ast", current_step, total_steps, "Extract annotations");
        let cleaned_ast = self.extract_annotations(raw_ast)?;
        self.log_success("transform_raw_ast", &format!("Stage 1 completed: {} rules cleaned", cleaned_ast.len()));
        current_step += 1;

        // Stage 2: Group by OR operators  
        self.log_progress("transform_raw_ast", current_step, total_steps, "Group by OR operators");
        let grouped_rules = self.group_by_or_operators(&cleaned_ast)?;
        self.log_success("transform_raw_ast", &format!("Stage 2 completed: {} rules grouped", grouped_rules.len()));
        current_step += 1;

        // Stage 2a: Extract branch return annotations
        self.log_progress("transform_raw_ast", current_step, total_steps, "Extract branch return annotations");
        let grouped_rules_cleaned = self.extract_branch_return_annotations(&grouped_rules)?;
        self.log_success("transform_raw_ast", &format!("Stage 2a completed: {} branch annotations extracted", 
            self.annotations.branch_return_annotations.len()));
        // Don't increment step counter - this is a sub-stage

        // Stage 2.5: Handle parentheses
        self.log_progress("transform_raw_ast", current_step, total_steps, "Handle parentheses");
        let processed_rules = self.handle_parentheses(&grouped_rules_cleaned)?;
        self.log_success("transform_raw_ast", &format!("Stage 2.5 completed: {} rules processed", processed_rules.len()));
        current_step += 1;

        // Stage 3: Parse sequences
        self.log_progress("transform_raw_ast", current_step, total_steps, "Parse sequences");
        let sequenced_rules = self.parse_sequences(&processed_rules)?;
        self.log_success("transform_raw_ast", &format!("Stage 3 completed: {} rules sequenced", sequenced_rules.len()));
        
        // AST visualization removed
        
        current_step += 1;

        // Stage 4: Handle quantifiers
        self.log_progress("transform_raw_ast", current_step, total_steps, "Handle quantifiers");
        let quantified_rules = self.handle_quantifiers(&sequenced_rules)?;
        self.log_success("transform_raw_ast", &format!("Stage 4 completed: {} rules quantified", quantified_rules.len()));
        
        // AST visualization removed
        
        current_step += 1;

        // Stage 5: Build tree structure
        self.log_progress("transform_raw_ast", current_step, total_steps, "Build tree structure");
        let (mut grammar_tree, mut rule_order) = self.build_tree_structure(&quantified_rules)?;
        self.log_success("transform_raw_ast", &format!("Stage 5 completed: {} final rules", grammar_tree.len()));
        
        // AST visualization removed
        
        current_step += 1;

        // Stage 6 (Optional): Left recursion elimination
        if self.config.eliminate_left_recursion {
            self.log_progress("transform_raw_ast", current_step, total_steps, "Apply left recursion elimination");
            (grammar_tree, rule_order) = self.eliminate_left_recursion(grammar_tree, rule_order)?;
            self.log_success("transform_raw_ast", "Stage 6 completed: Left recursion eliminated");
            
            // AST visualization removed
            
            self.stats.transformations_applied = 6;
        } else {
            self.log_info("transform_raw_ast", "Stage 6 skipped: Left recursion elimination disabled");
            self.stats.transformations_applied = 5;
        }

        self.stats.rules_processed = grammar_tree.len();
        
        self.log_success("transform_raw_ast", &format!(
            "🎉 PIPELINE COMPLETE! {} rules processed, {} annotations preserved, {} stages applied",
            self.stats.rules_processed,
            self.stats.annotations_preserved, 
            self.stats.transformations_applied
        ));
        
        self.write_log_summary();

        Ok((grammar_tree, rule_order))
    }

    /// Check if bootstrap mode should be used for annotation parsing
    fn should_use_bootstrap_mode(&self) -> bool {
        self.config.bootstrap_mode ||
        !self.external_parsers_available()
    }
    
    /// Check if external generated parsers are available
    fn external_parsers_available(&self) -> bool {
        // TEMPORARY: Force bootstrap mode until parsers are regenerated with AST-based generator
        false // Always use bootstrap mode for now
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
        
        // TEMPORARY: External parser disabled until regenerated with AST-based generator
        // Bootstrap mode will be used instead
        return self.parse_semantic_annotation_bootstrap(annotation_value);
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
    
    
    /// Parse return annotation into the unified AST
    fn parse_return_annotation(&mut self, annotation_value: &str) -> Result<UnifiedReturnAST> {
        if self.config.debug {
            println!("[AST Pipeline] Parsing return annotation: '{}'", annotation_value);
        }

        // First try bootstrap mode
        match UnifiedReturnAST::parse_bootstrap(annotation_value, self.config.debug) {
            Ok(ast) => {
                if self.config.debug {
                    println!("[AST Pipeline] Successfully parsed with bootstrap mode");
                }
                return Ok(ast);
            }
            Err(e) => {
                // Bootstrap parsing failed - log this for debugging
                self.log_warning("parse_return_annotation",
                    &format!("Bootstrap parser failed for '{}': {}. Will try external parser.", annotation_value, e));

                if self.config.debug {
                    println!("[AST Pipeline] Bootstrap failed: {}", e);
                    println!("[AST Pipeline] Attempting external parser fallback");
                }
            }
        }

        // Bootstrap failed, try external parser
        if self.config.debug {
            println!("[AST Pipeline] Trying external parser for: '{}'", annotation_value);
        }

        let parse_result = {
            let mut parser = return_annotation_parser::Return_annotationParser::new(annotation_value);
            match parser.parse() {
                Ok(result) => result,
                Err(e) => {
                    // Both parsers failed - this is a serious issue
                    self.log_error("parse_return_annotation",
                        &format!("Both bootstrap and external parsers failed for '{}'. Bootstrap error: {}. External error: {:?}",
                                annotation_value, "previous error", e));
                    return Err(anyhow!("Both bootstrap and external parsers failed for return annotation: {}", annotation_value));
                }
            }
        };

        // Convert ParseNode to UnifiedReturnAST
        match self.convert_parse_node_to_unified_ast(&parse_result) {
            Ok(ast) => Ok(ast),
            Err(e) => {
                self.log_error("parse_return_annotation",
                    &format!("ParseNode conversion failed for '{}': {}", annotation_value, e));
                Err(e)
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
    /* TEMPORARILY DISABLED: Depends on generated parsers
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
    */
    
    /// Parse and pretty-print return annotation for display
    fn parse_return_annotation_for_display(&self, annotation: &str) -> Result<String> {
        // Parse the return annotation to understand its structure
        let mut output = String::new();
        let indent = "     ";
        
        // Simple pattern matching for common return annotation patterns
        if annotation.starts_with("$") {
            // Scalar reference like $1, $2, etc.
            output.push_str(&format!("{}📌 AST Type: Scalar Reference\n", indent));
            output.push_str(&format!("{}└─ Index: {}\n", indent, &annotation[1..]));
        } else if annotation.starts_with("[") && annotation.ends_with("]") {
            // Array pattern like [$1, $2*]
            output.push_str(&format!("{}📌 AST Type: Array\n", indent));
            let content = &annotation[1..annotation.len()-1];
            output.push_str(&format!("{}└─ Elements: {}\n", indent, content));
            
            // Parse array elements
            if content.contains("$") {
                output.push_str(&format!("{}   ├─ Contains scalar references\n", indent));
            }
            if content.contains("*") {
                output.push_str(&format!("{}   └─ Has spread operator (*)\n", indent));
            }
        } else if annotation.starts_with("{") && annotation.ends_with("}") {
            // Object pattern like {type: "array", contents: $3}
            output.push_str(&format!("{}📌 AST Type: Object\n", indent));
            let content = &annotation[1..annotation.len()-1];
            
            // Parse object properties
            let parts: Vec<&str> = content.split(',').collect();
            for (i, part) in parts.iter().enumerate() {
                let is_last = i == parts.len() - 1;
                let prefix = if is_last { "└─" } else { "├─" };
                
                if let Some(colon_pos) = part.find(':') {
                    let key = part[..colon_pos].trim();
                    let value = part[colon_pos+1..].trim();
                    output.push_str(&format!("{}   {} Property: {}\n", indent, prefix, key));
                    output.push_str(&format!("{}      Value: {}\n", indent, value));
                }
            }
        } else if annotation == "\"*\"" || annotation == "\"+\"" || annotation == "\"?\"" {
            // Quantifier
            output.push_str(&format!("{}📌 AST Type: Quantifier\n", indent));
            output.push_str(&format!("{}└─ Value: {}\n", indent, annotation));
        } else if annotation.starts_with('"') && annotation.ends_with('"') {
            // String literal
            output.push_str(&format!("{}📌 AST Type: String Literal\n", indent));
            output.push_str(&format!("{}└─ Value: {}\n", indent, annotation));
        } else {
            // Complex or unknown pattern
            output.push_str(&format!("{}📌 AST Type: Complex Expression\n", indent));
            output.push_str(&format!("{}└─ Raw: {}\n", indent, annotation));
        }
        
        Ok(output)
    }
    
    /// Convert external parser's ParseNode to UnifiedReturnAST
    /// This interprets the syntactic parse tree to extract semantic meaning
    fn convert_parse_node_to_unified_ast(&self, node: &return_annotation_parser::ParseNode) -> Result<UnifiedReturnAST> {
        use return_annotation_parser::ParseContent;
        
        if self.config.debug {
            println!("[AST Pipeline] Converting ParseNode to UnifiedReturnAST");
            println!("  Rule: {}", node.rule_name);
        }
        
        // The external parser produces a syntactic tree based on the return_annotation.ebnf grammar
        // We need to interpret this tree to build the semantic UnifiedReturnAST
        
        match node.rule_name {
            "return_annotation" => {
                // Top level rule - delegate to the content
                match &node.content {
                    ParseContent::Sequence(nodes) if !nodes.is_empty() => {
                        // Usually contains the actual annotation content
                        self.convert_parse_node_to_unified_ast(&nodes[0])
                    }
                    ParseContent::Alternative(inner) => {
                        self.convert_parse_node_to_unified_ast(inner)
                    }
                    _ => Err(anyhow!("Unexpected return_annotation structure"))
                }
            }
            "scalar_ref" | "positional_ref" => {
                // Parse $N references
                match &node.content {
                    ParseContent::Terminal(text) => {
                        // Extract the number from $N
                        let num_str = text.trim_start_matches('$');
                        let index = num_str.parse::<usize>()
                            .map_err(|_| anyhow!("Invalid positional reference: {}", text))?;
                        Ok(UnifiedReturnAST::PositionalRef { index })
                    }
                    _ => Err(anyhow!("Invalid scalar_ref structure"))
                }
            }
            "array" | "array_spec" => {
                // Parse array [...]
                match &node.content {
                    ParseContent::Sequence(nodes) => {
                        // Extract array elements, handling spread operators
                        let mut elements = Vec::new();
                        for node in nodes {
                            if node.rule_name == "spread" || node.rule_name.contains("spread") {
                                // Handle spread operator
                                let base = self.convert_parse_node_to_unified_ast(node)?;
                                elements.push(UnifiedReturnAST::Spread { base: Box::new(base) });
                            } else if node.rule_name != "[" && node.rule_name != "]" && node.rule_name != "," {
                                // Regular element
                                elements.push(self.convert_parse_node_to_unified_ast(node)?);
                            }
                        }
                        Ok(UnifiedReturnAST::Array { elements })
                    }
                    _ => Err(anyhow!("Invalid array structure"))
                }
            }
            "object" | "object_spec" => {
                // Parse object {...}
                match &node.content {
                    ParseContent::Sequence(nodes) => {
                        // Extract key-value pairs
                        let mut properties = std::collections::HashMap::new();
                        let mut i = 0;
                        while i < nodes.len() {
                            // Look for key:value patterns
                            if i + 2 < nodes.len() {
                                let key_node = &nodes[i];
                                let colon_node = &nodes[i + 1];
                                let value_node = &nodes[i + 2];
                                
                                if let ParseContent::Terminal(colon) = &colon_node.content {
                                    if *colon == ":" {
                                        let key = self.extract_string_from_node(key_node)?;
                                        let value = self.convert_parse_node_to_unified_ast(value_node)?;
                                        properties.insert(key, Box::new(value));
                                        i += 3;
                                        continue;
                                    }
                                }
                            }
                            i += 1;
                        }
                        Ok(UnifiedReturnAST::Object { properties })
                    }
                    _ => Err(anyhow!("Invalid object structure"))
                }
            }
            "string_literal" | "string" => {
                // Parse string literals
                match &node.content {
                    ParseContent::Terminal(text) => {
                        // Remove quotes if present
                        let cleaned = text.trim_matches('"');
                        Ok(UnifiedReturnAST::StringLiteral { value: cleaned.to_string() })
                    }
                    _ => Err(anyhow!("Invalid string literal structure"))
                }
            }
            _ => {
                // Fallback: try to interpret based on content
                match &node.content {
                    ParseContent::Terminal(text) => {
                        // Check what kind of terminal this is
                        if text.starts_with('$') {
                            let index = text[1..].parse::<usize>()
                                .map_err(|_| anyhow!("Invalid positional reference: {}", text))?;
                            Ok(UnifiedReturnAST::PositionalRef { index })
                        } else if text.starts_with('"') && text.ends_with('"') {
                            Ok(UnifiedReturnAST::StringLiteral { 
                                value: text[1..text.len()-1].to_string() 
                            })
                        } else if let Ok(num) = text.parse::<f64>() {
                            Ok(UnifiedReturnAST::NumberLiteral { value: num })
                        } else {
                            Err(anyhow!("Unknown terminal content: {}", text))
                        }
                    }
                    ParseContent::Alternative(inner) => {
                        // Unwrap alternatives
                        self.convert_parse_node_to_unified_ast(inner)
                    }
                    ParseContent::Sequence(nodes) if nodes.len() == 1 => {
                        // Unwrap single-element sequences
                        self.convert_parse_node_to_unified_ast(&nodes[0])
                    }
                    _ => Err(anyhow!("Cannot convert ParseNode with rule '{}' to UnifiedReturnAST", node.rule_name))
                }
            }
        }
    }
    
    /// Helper to extract a string from a parse node (for object keys, etc.)
    fn extract_string_from_node(&self, node: &return_annotation_parser::ParseNode) -> Result<String> {
        use return_annotation_parser::ParseContent;
        
        match &node.content {
            ParseContent::Terminal(text) => {
                // Remove quotes if present
                Ok(text.trim_matches('"').to_string())
            }
            _ => Err(anyhow!("Expected terminal for string extraction"))
        }
    }
    
    /* TEMPORARILY DISABLED: Depends on generated parsers
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
    */

    /// Stage 1: Extract and preserve annotations
    fn extract_annotations(&mut self, raw_ast: &RawAST) -> Result<RawAST> {
        self.log_info("extract_annotations", &format!("🔍 Starting annotation extraction for {} rules", raw_ast.len()));

        let mut cleaned_ast = RawAST::new();
        let mut _total_annotations_found = 0;

        for (rule_index, rule_def) in raw_ast.iter().enumerate() {
            if rule_def.is_empty() {
                self.log_warning("extract_annotations", &format!("Skipping empty rule at index {}", rule_index));
                continue;
            }

            let mut rule_name: Option<String> = None;
            let mut cleaned_rule = TokenSequence::new();
            let mut rule_annotations_found = 0;

            for (token_index, token) in rule_def.iter().enumerate() {
                if token.len() != 2 {
                    self.log_warning("extract_annotations", &format!("Skipping malformed token at rule[{}][{}]: {:?}", rule_index, token_index, token));
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
                            self.log_info("extract_annotations", &format!("🔖 Processing rule: '{}' (index: {})", name, rule_index));
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
                                                self.log_info("extract_annotations", &format!("🏷️  Parsing semantic annotation: '{}' = '{}' for rule '{}'", annotation_name, annotation_value, name));
                                                // Use the semantic annotation parser for semantic annotations
                                                let parsed_value = self.parse_semantic_annotation(annotation_value)
                                                    .unwrap_or_else(|_| format!("raw:{}", annotation_value));
                                                let formatted_annotation = format!("{}:{}", annotation_name, parsed_value);
                                                self.annotations.semantic_annotations
                                                    .entry(name.clone())
                                                    .or_insert_with(Vec::new)
                                                    .push(formatted_annotation);
                                                self.log_success("extract_annotations", &format!("Semantic annotation processed: '{}'", annotation_name));
                                            }
                                            "logging_annotation" => {
                                                self.log_info("extract_annotations", &format!("📝 Parsing logging annotation: '{}' = '{}' for rule '{}'", annotation_name, annotation_value, name));
                                                let formatted_annotation = format!("{}({})", annotation_name, annotation_value);
                                                self.annotations.logging_annotations
                                                    .entry(name.clone())
                                                    .or_insert_with(Vec::new)
                                                    .push(formatted_annotation);
                                                self.log_success("extract_annotations", &format!("Logging annotation processed: '{}'", annotation_name));
                                            }
                                            _ => unreachable!(),
                                        }
                                        
                                        rule_annotations_found += 1;
                                    }
                                } else {
                                    // Fallback for old string format or malformed data
                                    self.log_warning("extract_annotations", &format!("Unexpected annotation format for {}: {:?}", token_type, token[1]));
                                    match token_type.as_str() {
                                        "semantic_annotation" => {
                                            // Still try to parse string format semantic annotations
                                            if let TokenValue::String(value_str) = &token[1] {
                                                self.log_info("extract_annotations", &format!("🔄 Fallback: Parsing string format semantic annotation for rule '{}'", name));
                                                let parsed_value = self.parse_semantic_annotation(value_str)
                                                    .unwrap_or_else(|_| format!("raw:{}", value_str));
                                                self.annotations.semantic_annotations
                                                    .entry(name.clone())
                                                    .or_insert_with(Vec::new)
                                                    .push(parsed_value);
                                            } else {
                                                self.log_warning("extract_annotations", &format!("Storing raw semantic annotation for rule '{}'", name));
                                                self.annotations.semantic_annotations
                                                    .entry(name.clone())
                                                    .or_insert_with(Vec::new)
                                                    .push(format!("raw:{:?}", token[1]));
                                            }
                                        }
                                        "logging_annotation" => {
                                            self.log_warning("extract_annotations", &format!("Storing raw logging annotation for rule '{}'", name));
                                            self.annotations.logging_annotations
                                                .entry(name.clone())
                                                .or_insert_with(Vec::new)
                                                .push(format!("raw:{:?}", token[1]));
                                        }
                                        _ => unreachable!(),
                                    }
                                    rule_annotations_found += 1;
                                }
                                self.stats.annotations_preserved += 1;
                            } else {
                                self.log_warning("extract_annotations", "Annotation preservation is disabled");
                            }
                        } else {
                            self.log_warning("extract_annotations", "Found annotation token but no rule name available");
                        }
                        // Don't add to cleaned rule
                    }
                    "return_scalar" | "return_array" | "return_object" => {
                        // IMPORTANT: Keep return annotations in the token stream!
                        // They will be extracted later after alternatives are split
                        if let Some(ref name) = rule_name {
                            self.log_info("extract_annotations", &format!("↩️  Keeping return annotation in stream: type={} for rule '{}'", token_type, name));
                        }
                        cleaned_rule.push(token.clone());  // ADD to cleaned rule, don't remove!
                    }
                    _ => {
                        cleaned_rule.push(token.clone());
                    }
                }
            }

            if !cleaned_rule.is_empty() {
                cleaned_ast.push(cleaned_rule);
                if rule_annotations_found > 0 {
                    self.log_success("extract_annotations", &format!("Rule '{}' completed: {} annotations found", rule_name.as_deref().unwrap_or("unknown"), rule_annotations_found));
                    _total_annotations_found += rule_annotations_found;
                }
            } else {
                self.log_warning("extract_annotations", &format!("Rule at index {} resulted in empty cleaned rule", rule_index));
            }
        }

        self.log_success("extract_annotations", &format!(
            "📊 Annotation extraction complete! {} total annotations preserved from {} rules", 
            self.stats.annotations_preserved, 
            cleaned_ast.len()
        ));
        
        // Log breakdown by annotation type
        let semantic_count = self.annotations.semantic_annotations.values().map(|v| v.len()).sum::<usize>();
        let logging_count = self.annotations.logging_annotations.values().map(|v| v.len()).sum::<usize>();
        // Return annotations are now kept in stream for later extraction
        
        self.log_info("extract_annotations", &format!(
            "📊 Breakdown: {} semantic, {} logging annotations (return annotations kept in stream)",
            semantic_count, logging_count
        ));

        Ok(cleaned_ast)
    }

    /// Stage 2: Group rule definitions by OR operators
    fn group_by_or_operators(&mut self, ast: &RawAST) -> Result<HashMap<String, Vec<TokenSequence>>> {
        self.log_info("group_by_or_operators", &format!("🔀 Starting OR operator grouping for {} rules", ast.len()));

        let mut grouped = HashMap::new();
        let mut total_alternatives = 0;

        for (rule_index, rule_def) in ast.iter().enumerate() {
            if rule_def.is_empty() {
                self.log_warning("group_by_or_operators", &format!("Skipping empty rule at index {}", rule_index));
                continue;
            }

            let mut rule_name: Option<String> = None;
            for token in rule_def {
                if token.len() == 2 {
                    if let (TokenValue::String(type_str), TokenValue::String(name_str)) = (&token[0], &token[1]) {
                        if type_str == "rule" {
                            rule_name = Some(name_str.clone());
                            self.log_info("group_by_or_operators", &format!("🔖 Processing rule: '{}' (index: {})", name_str, rule_index));
                            break;
                        }
                    }
                }
            }

            if let Some(name) = rule_name {
                let mut alternatives = Vec::new();
                let mut current_alt = TokenSequence::new();
                let mut or_operators_found = 0;

                // Skip rule definition token
                let mut paren_depth: i32 = 0;
                for token in rule_def.iter().skip(1) {
                    if token.len() == 2 {
                        if let (TokenValue::String(type_str), TokenValue::String(value_str)) = (&token[0], &token[1]) {
                            // Track parentheses depth
                            match type_str.as_str() {
                                "group_open" => paren_depth += 1,
                                "group_close" => paren_depth = paren_depth.saturating_sub(1),
                                _ => {}
                            }
                            
                            // Only split on | operators at top level (outside parentheses)
                            if type_str == "operator" && value_str == "|" && paren_depth == 0 {
                                if !current_alt.is_empty() {
                                    alternatives.push(current_alt);
                                    current_alt = TokenSequence::new();
                                    or_operators_found += 1;
                                    self.log_info("group_by_or_operators", &format!("✂️  OR operator #{} found in rule '{}', creating alternative", or_operators_found, name));
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

                self.log_success("group_by_or_operators", &format!("Rule '{}' processed: {} alternatives created", name, alternatives.len()));
                total_alternatives += alternatives.len();
                grouped.entry(name).or_insert_with(Vec::new).extend(alternatives);
            } else {
                self.log_warning("group_by_or_operators", &format!("No rule name found for rule at index {}", rule_index));
            }
        }

        self.log_success("group_by_or_operators", &format!(
            "📊 OR operator grouping complete! {} rules processed, {} total alternatives created",
            grouped.len(), total_alternatives
        ));

        Ok(grouped)
    }

    /// Stage 2a: Extract branch return annotations from grouped alternatives
    fn extract_branch_return_annotations(&mut self, grouped_rules: &HashMap<String, Vec<TokenSequence>>) -> Result<HashMap<String, Vec<TokenSequence>>> {
        self.log_info("extract_branch_return_annotations", &format!("🎯 Extracting branch-level return annotations for {} rules", grouped_rules.len()));
        
        let mut processed = HashMap::new();
        let mut total_branch_annotations = 0;
        
        for (rule_name, alternatives) in grouped_rules {
            eprintln!("DEBUG: Processing rule '{}'", rule_name);
            let mut branch_annotations = Vec::new();
            let mut cleaned_alts = Vec::new();
            
            self.log_info("extract_branch_return_annotations", &format!("📌 Processing rule '{}' with {} alternatives", rule_name, alternatives.len()));
            
            for (branch_idx, alt) in alternatives.iter().enumerate() {
                eprintln!("DEBUG: Processing branch {} of rule '{}'", branch_idx, rule_name);
                let mut cleaned_alt = TokenSequence::new();
                let mut branch_annotation: Option<ReturnAnnotation> = None;
                
                // Scan the alternative for return annotations
                for token in alt {
                    if token.len() == 2 {
                        if let (TokenValue::String(type_str), TokenValue::String(content)) = (&token[0], &token[1]) {
                            match type_str.as_str() {
                                "return_scalar" | "return_array" | "return_object" => {
                                    // Found a return annotation for this branch!
                                    self.log_success("extract_branch_return_annotations", 
                                        &format!("  Branch {}: Found {} annotation: '{}'", branch_idx, type_str, content));
                                    
                                    // Always log the return annotation details with pretty-printed AST
                                    self.log_info("extract_branch_return_annotations", 
                                        &format!("\n📝 RETURN ANNOTATION DETAILS:"));
                                    self.log_info("extract_branch_return_annotations", 
                                        &format!("   Rule: {}", rule_name));
                                    self.log_info("extract_branch_return_annotations", 
                                        &format!("   Branch: {}", branch_idx));
                                    self.log_info("extract_branch_return_annotations", 
                                        &format!("   Type: {}", type_str));
                                    self.log_info("extract_branch_return_annotations", 
                                        &format!("   Raw Value: '{}'", content));
                                    
                                    // Try to parse and pretty-print the return annotation
                                    if let Ok(parsed) = self.parse_return_annotation_for_display(content) {
                                        self.log_info("extract_branch_return_annotations", 
                                            &format!("   Parsed AST (pretty-printed):\n{}", parsed));
                                    }
                                    self.log_info("extract_branch_return_annotations", &format!(""));
                                    
                                    // Ensure the annotation has the -> prefix for the new grammar
                                    let prefixed_annotation = if content.starts_with("->") {
                                        content.clone()
                                    } else {
                                        format!("-> {}", content)
                                    };
                                    
                                    // Parse the annotation into the unified AST
                                    let parsed_ast = self.parse_return_annotation(&prefixed_annotation)
                                        .map_err(|e| {
                                            self.log_warning("extract_branch_return_annotations",
                                                &format!("Failed to parse return annotation: {}", e));
                                            e
                                        })
                                        .ok();
                                    
                                    // Log the parsed AST for debugging
                                    if let Some(ref ast) = parsed_ast {
                                        self.log_info("extract_branch_return_annotations",
                                            &format!("   Parsed Unified AST:\n{}", ast.pretty_print(5)));
                                    }
                                    
                                    branch_annotation = Some(ReturnAnnotation {
                                        annotation_type: type_str.clone(),
                                        annotation_content: prefixed_annotation,
                                        parsed_ast,
                                    });
                                    total_branch_annotations += 1;
                                    // Don't add return annotation tokens to cleaned alternative
                                    continue;
                                }
                                _ => {}
                            }
                        }
                    }
                    cleaned_alt.push(token.clone());
                }
                
                branch_annotations.push(branch_annotation);
                cleaned_alts.push(cleaned_alt);
            }
            
            // Store the branch annotations for this rule
            if branch_annotations.iter().any(|a| a.is_some()) {
                self.annotations.branch_return_annotations.insert(rule_name.clone(), branch_annotations);
                self.log_success("extract_branch_return_annotations", 
                    &format!("Rule '{}': Stored {} branch annotations", rule_name, 
                        self.annotations.branch_return_annotations[rule_name].iter().filter(|a| a.is_some()).count()));
            }
            
            processed.insert(rule_name.clone(), cleaned_alts);
        }
        
        self.log_success("extract_branch_return_annotations", 
            &format!("✅ Branch return annotation extraction complete! {} total annotations from {} rules", 
                total_branch_annotations, processed.len()));
        
        Ok(processed)
    }

    /// Stage 2.5: Handle parentheses and grouping
    fn handle_parentheses(&mut self, grouped_rules: &HashMap<String, Vec<TokenSequence>>) -> Result<HashMap<String, Vec<TokenSequence>>> {
        self.log_info("handle_parentheses", &format!("🔗 Starting parentheses handling for {} rules", grouped_rules.len()));

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
        // For now, just return the sequence as-is
        // We'll handle groups properly in the quantifier stage
        // This preserves the group_open and group_close tokens
        Ok(sequence.clone())
    }


    /// Stage 3: Parse sequences
    fn parse_sequences(&mut self, processed_rules: &HashMap<String, Vec<TokenSequence>>) -> Result<HashMap<String, Vec<ASTNode>>> {
        self.log_info("parse_sequences", &format!("🔗 Starting sequence parsing for {} rules", processed_rules.len()));

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
        // All tokens are now treated as atoms since we preserve group boundaries
        // Groups will be handled properly in the quantifier stage
        Ok(ASTNode::Atom { value: ASTValue::Token(element.clone()) })
    }

    /// Stage 4: Handle quantifiers
    fn handle_quantifiers(&mut self, sequenced_rules: &HashMap<String, Vec<ASTNode>>) -> Result<HashMap<String, Vec<ASTNode>>> {
        self.log_info("handle_quantifiers", &format!("✨ Starting quantifier handling for {} rules", sequenced_rules.len()));

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

    /// Apply quantifiers to AST node using the new SOTA grouped quantifier parser
    fn apply_quantifiers_to_node(&mut self, node: ASTNode) -> Result<ASTNode> {
        self.log_info("apply_quantifiers_to_node", "📥 ENTER: Processing AST node for quantifiers");
        
        match node {
            ASTNode::Sequence { elements } => {
                self.log_info("apply_quantifiers_to_node", &format!("🔄 Processing sequence with {} elements", elements.len()));
                if self.config.debug {
                    for (idx, elem) in elements.iter().enumerate() {
                        self.log_debug("apply_quantifiers_to_node", &format!("  Element[{}]: {:?}", idx, elem));
                    }
                }
                
                self.log_debug("apply_quantifiers_to_node", "🔄 Converting AST elements to tokens (no flattening)");
                // Convert AST elements DIRECTLY to tokens - NO FLATTENING
                let mut tokens = Vec::new();
                for element in &elements {
                    self.ast_node_to_tokens_simple(element, &mut tokens)?;
                }
                self.log_debug("apply_quantifiers_to_node", &format!("  Converted to {} raw tokens", tokens.len()));
                
                // Use the new grouped quantifier parser
                self.log_debug("apply_quantifiers_to_node", "🎭 Creating GroupedQuantifierParser");
                let parser = grouped_quantifier_parser::GroupedQuantifierParser::new(self.config.debug);
                
                self.log_debug("apply_quantifiers_to_node", "🔄 Tokenizing raw tokens");
                let parsed_tokens = parser.tokenize_from_raw_tokens(&tokens)?;
                self.log_debug("apply_quantifiers_to_node", &format!("  Tokenized to {} tokens", parsed_tokens.len()));
                
                self.log_debug("apply_quantifiers_to_node", "🔄 Parsing token sequence for quantified groups");
                let parsed_elements = parser.parse_sequence(&parsed_tokens)
                    .with_context(|| format!("Failed to parse quantified groups"))?;
                self.log_debug("apply_quantifiers_to_node", &format!("  Parsed {} elements", parsed_elements.len()));
                
                // Convert back to AST nodes
                self.log_debug("apply_quantifiers_to_node", "🔄 Converting parsed elements back to AST nodes");
                let mut new_elements = Vec::new();
                for (idx, parsed) in parsed_elements.into_iter().enumerate() {
                    self.log_debug("apply_quantifiers_to_node", &format!("  Converting element[{}]", idx));
                    new_elements.push(parser.to_ast_node(parsed));
                }
                
                if self.config.debug {
                    self.log_success("apply_quantifiers_to_node", &format!("✅ Finished processing sequence. Original: {} elements, New: {} elements", elements.len(), new_elements.len()));
                    println!("[AST][apply_quantifiers_to_node] New sequence elements:");
                    for (idx, elem) in new_elements.iter().enumerate() {
                        println!("[AST][apply_quantifiers_to_node]   NewElement[{}]: {:?}", idx, elem);
                    }
                }
                self.log_info("apply_quantifiers_to_node", "📤 EXIT: Returning processed sequence");
                Ok(ASTNode::Sequence { elements: new_elements })
            }
            _ => {
                self.log_debug("apply_quantifiers_to_node", &format!("Non-sequence node, returning as-is: {:?}", node));
                self.log_info("apply_quantifiers_to_node", "📤 EXIT: Non-sequence node unchanged");
                Ok(node)
            }
        }
    }
    
    /// Convert an ASTNode to tokens
    fn ast_node_to_tokens_simple(&self, node: &ASTNode, tokens: &mut Vec<Token>) -> Result<()> {
        match node {
            ASTNode::Atom { value } => {
                if let ASTValue::Token(token) = value {
                    if self.config.debug && self.config.trace {
                        println!("[AST][ast_node_to_tokens_simple]   Adding Atom token: {:?}", token);
                    }
                    tokens.push(token.clone());
                }
            }
            ASTNode::Sequence { elements } => {
                // Since we're not collapsing groups anymore, sequences should just be
                // a flat list of atoms at this point
                if self.config.debug && self.config.trace {
                    println!("[AST][ast_node_to_tokens_simple]   Processing Sequence with {} elements", elements.len());
                }
                for elem in elements {
                    self.ast_node_to_tokens_simple(elem, tokens)?;
                }
            }
            ASTNode::Or { alternatives } => {
                if self.config.debug && self.config.trace {
                    println!("[AST][ast_node_to_tokens_simple]   Processing Or with {} alternatives", alternatives.len());
                }
                // This shouldn't happen at this stage but handle it
                for (i, alt) in alternatives.iter().enumerate() {
                    if i > 0 {
                        tokens.push(vec![TokenValue::String("operator".to_string()), TokenValue::String("|".to_string())]);
                    }
                    self.ast_node_to_tokens_simple(alt, tokens)?;
                }
            }
            ASTNode::Quantified { .. } => {
                // This shouldn't happen at this stage - quantifiers haven't been applied yet
                if self.config.debug {
                    println!("[AST][ast_node_to_tokens_simple] ⚠️ WARNING: Unexpected quantified node before quantifier processing");
                }
                return Err(anyhow!("Unexpected quantified node before quantifier processing"));
            }
        }
        Ok(())
    }
    /// Stage 5: Build final tree structure
    fn build_tree_structure(&mut self, quantified_rules: &HashMap<String, Vec<ASTNode>>) -> Result<(HashMap<String, ASTNode>, Vec<String>)> {
        self.log_info("build_tree_structure", &format!("🌳 Starting tree structure building for {} rules", quantified_rules.len()));

        let mut grammar_tree = HashMap::new();
        let rule_order: Vec<String> = quantified_rules.keys().cloned().collect();

        for (rule_name, alternatives) in quantified_rules {
            // Debug output for simple_object
            if rule_name == "simple_object" {
                println!("[DEBUG] build_tree_structure: Processing simple_object with {} alternatives", alternatives.len());
                for (i, alt) in alternatives.iter().enumerate() {
                    println!("[DEBUG]   Alternative {}: {:?}", i, std::mem::discriminant(alt));
                }
            }
            
            // ALWAYS wrap in Or node - even single-branch rules
            // This ensures uniform handling of return annotations
            let final_node = ASTNode::Or {
                alternatives: alternatives.clone()
            };
            
            if rule_name == "simple_object" {
                println!("[DEBUG] build_tree_structure: simple_object wrapped in Or node");
            }

            grammar_tree.insert(rule_name.clone(), final_node);
        }
        
        // Verify simple_object is wrapped
        if let Some(node) = grammar_tree.get("simple_object") {
            match node {
                ASTNode::Or { .. } => println!("[DEBUG] build_tree_structure: VERIFIED simple_object is Or node in final grammar_tree"),
                _ => println!("[DEBUG] build_tree_structure: ERROR! simple_object is NOT Or node in grammar_tree!")
            }
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

        std::fs::write(output_file, json)
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
        self.log_info("transform_from_file", &format!("📂 Loading raw AST from: {}", raw_ast_json_file));
        let raw_data = self.load_raw_ast(raw_ast_json_file)?;
        let (grammar_tree, rule_order) = self.transform_raw_ast(&raw_data.raw_ast)?;

        if let Some(output_file) = output_json_file {
            self.log_info("transform_from_file", &format!("💾 Saving transformed AST to: {}", output_file));
            self.save_transformed_ast(&grammar_tree, &rule_order, &raw_data.grammar_name, output_file)?;
        }

        // Ensure log file is flushed before returning
        self.flush_log();
        Ok((grammar_tree, rule_order))
    }

    /// Cross-language API: Transform raw AST JSON file to transformed AST JSON file
    pub fn transform_to_json(&mut self, raw_ast_json_file: &str, output_json_file: &str) -> Result<()> {
        self.log_info("transform_to_json", &format!("🔄 Cross-language transformation: {} → {}", raw_ast_json_file, output_json_file));
        let (grammar_tree, rule_order) = self.transform_from_file(raw_ast_json_file, None)?;
        let raw_data = self.load_raw_ast(raw_ast_json_file)?;
        self.save_transformed_ast(&grammar_tree, &rule_order, &raw_data.grammar_name, output_json_file)?;
        self.log_success("transform_to_json", &format!("✅ Cross-language transformation complete: {}", output_json_file));
        self.flush_log();
        Ok(())
    }

    /// Get branch return annotations for a specific rule
    #[allow(dead_code)]
    pub fn get_branch_return_annotations(&self, rule_name: &str) -> Option<&Vec<Option<ReturnAnnotation>>> {
        self.annotations.branch_return_annotations.get(rule_name)
    }
    
    /// Get all branch return annotations
    #[allow(dead_code)]
    pub fn get_all_branch_return_annotations(&self) -> &HashMap<String, Vec<Option<ReturnAnnotation>>> {
        &self.annotations.branch_return_annotations
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

    /// Generate AST-based Rust parser code using syn/quote for guaranteed correctness
    /// Produces syntactically correct parsers with no string concatenation issues
    pub fn generate_rust_parser(
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
                rule_order.first().cloned().unwrap_or_else(|| {
                    eprintln!("ERROR: No rules found in rule_order, cannot determine entry rule");
                    "unknown_entry_rule".to_string()
                })
            });
        
        // Use AST-based generator for guaranteed syntax correctness
        let mut generator = AstBasedGenerator::new(raw_data.grammar_name.clone());
        generator.entry_rule = Some(entry_rule);
        generator.annotations = Some(self.annotations.clone());
        generator.enable_debug = enable_trace || enable_backtrack_debug;
        
        // Convert ReturnAnnotation to BranchAnnotation format
        generator.branch_return_annotations = self.annotations.branch_return_annotations
            .iter()
            .map(|(rule, branches)| {
                let converted_branches = branches
                    .iter()
                    .map(|opt_annotation| {
                        opt_annotation.as_ref().map(|ann| ast_based_generator::BranchAnnotation {
                            annotation_type: ann.annotation_type.clone(),
                            annotation_content: ann.annotation_content.clone(),
                            parsed_ast: ann.parsed_ast.clone(),
                        })
                    })
                    .collect();
                (rule.clone(), converted_branches)
            })
            .collect();
        
        // Generate parser using AST-based approach
        let rust_code = generator.generate_parser(&grammar_tree, &rule_order)?;
        
        std::fs::write(output_rust_file, rust_code)
            .with_context(|| format!("Failed to write AST-based Rust parser to: {}", output_rust_file))?;
        
        if self.config.debug {
            println!("Generated AST-based parser: {}", output_rust_file);
            println!("  - Guaranteed syntax correctness via syn/quote");
            if enable_trace {
                println!("  - Trace logging enabled");
            }
            if enable_backtrack_debug {
                println!("  - Debug mode enabled");
            }
        }
        
        Ok(())
    }
    
    /// Compatibility alias for old method name
    pub fn generate_high_performance_parser(
        &mut self,
        raw_ast_json_file: &str,
        output_rust_file: &str,
        enable_trace: bool,
        enable_backtrack_debug: bool,
    ) -> Result<()> {
        self.generate_rust_parser(raw_ast_json_file, output_rust_file, enable_trace, enable_backtrack_debug)
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
            let ast_node = self.productions_to_ast_node(productions)?;
            
            // CRITICAL: Ensure all rules are wrapped in Or nodes for uniform return annotation handling
            // This preserves the structure established by build_tree_structure
            let final_node = match ast_node {
                ASTNode::Or { .. } => ast_node,  // Already an Or node
                other => ASTNode::Or { alternatives: vec![other] }  // Wrap in Or node
            };
            
            result_grammar.insert(rule_name, final_node);
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
                                    _ => {
                                        // Handle complex AST nodes by preserving their structure
                                        let serialized = serde_json::to_string(element)
                                            .unwrap_or_else(|_| format!("{:?}", element));
                                        production.push(format!("COMPLEX_GROUP:{}", serialized));
                                    },
                                }
                            } else {
                                // Handle complex AST nodes by preserving their structure
                                let serialized = serde_json::to_string(element)
                                    .unwrap_or_else(|_| format!("{:?}", element));
                                production.push(format!("COMPLEX_GROUP:{}", serialized));
                            }
                        }
                        ASTNode::Quantified { element, quantifier } => {
                            // Preserve quantified groups with their structure
                            let serialized = serde_json::to_string(element)
                                .unwrap_or_else(|_| format!("{:?}", element));
                            production.push(format!("QUANTIFIED:{}:{}", quantifier, serialized));
                        }
                        ASTNode::Or { alternatives } => {
                            // Preserve OR groups with their structure  
                            let serialized = serde_json::to_string(alternatives)
                                .unwrap_or_else(|_| format!("{:?}", alternatives));
                            production.push(format!("OR_GROUP:{}", serialized));
                        }
                        ASTNode::Sequence { elements } => {
                            // Preserve nested sequences with their structure
                            let serialized = serde_json::to_string(elements)
                                .unwrap_or_else(|_| format!("{:?}", elements));
                            production.push(format!("NESTED_SEQUENCE:{}", serialized));
                        }
                        ASTNode::Atom { value: ASTValue::Node(node) } => {
                            // Handle nested nodes by recursively processing them
                            let inner_productions = self.ast_node_to_productions(node)?;
                            if !inner_productions.is_empty() && !inner_productions[0].is_empty() {
                                production.extend(inner_productions[0].clone());
                            } else {
                                production.push("ε".to_string());
                            }
                        }
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
                                let serialized = serde_json::to_string(token)
                                    .unwrap_or_else(|_| format!("{:?}", token));
                                Ok(vec![vec![format!("COMPLEX_GROUP:{}", serialized)]])
                            }
                        }
                        Some("rule_reference") => {
                            if let Some(name) = token[1].as_str() {
                                Ok(vec![vec![name.to_string()]])
                            } else {
                                let serialized = serde_json::to_string(token)
                                    .unwrap_or_else(|_| format!("{:?}", token));
                                Ok(vec![vec![format!("COMPLEX_GROUP:{}", serialized)]])
                            }
                        }
                        Some("regex") => {
                            if let Some(pattern) = token[1].as_str() {
                                Ok(vec![vec![format!("REGEX:{}", pattern)]])
                            } else {
                                let serialized = serde_json::to_string(token)
                                    .unwrap_or_else(|_| format!("{:?}", token));
                                Ok(vec![vec![format!("COMPLEX_GROUP:{}", serialized)]])
                            }
                        }
                        _ => {
                            let serialized = serde_json::to_string(token)
                                .unwrap_or_else(|_| format!("{:?}", token));
                            Ok(vec![vec![format!("COMPLEX_GROUP:{}", serialized)]])
                        },
                    }
                } else {
                    let serialized = serde_json::to_string(token)
                        .unwrap_or_else(|_| format!("{:?}", token));
                    Ok(vec![vec![format!("COMPLEX_GROUP:{}", serialized)]])
                }
            }
            ASTNode::Quantified { element, quantifier } => {
                // Handle top-level quantified nodes
                let serialized = serde_json::to_string(element)
                    .unwrap_or_else(|_| format!("{:?}", element));
                Ok(vec![vec![format!("QUANTIFIED:{}:{}", quantifier, serialized)]])
            }
            ASTNode::Atom { value: ASTValue::Node(node) } => {
                // Handle nested nodes by recursively processing them
                self.ast_node_to_productions(node)
            }
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
                } else if symbol.starts_with("QUANTIFIED:") {
                    // Handle quantified groups: format is "QUANTIFIED:quantifier:serialized_element"
                    let parts: Vec<&str> = symbol[11..].splitn(2, ':').collect();
                    if parts.len() == 2 {
                        let quantifier = parts[0].to_string();
                        let serialized_element = parts[1];
                        
                        // Try to deserialize the element back to ASTNode
                        let element: ASTNode = serde_json::from_str(serialized_element)
                            .unwrap_or_else(|_| ASTNode::Atom { 
                                value: ASTValue::Token(vec![
                                    TokenValue::String("quoted_string".to_string()),
                                    TokenValue::String("".to_string()) // Fallback to empty terminal
                                ]) 
                            });
                            
                        Ok(ASTNode::Quantified { 
                            element: Box::new(element), 
                            quantifier 
                        })
                    } else {
                        // Malformed quantified entry, treat as empty terminal
                        Ok(ASTNode::Atom { 
                            value: ASTValue::Token(vec![
                                TokenValue::String("quoted_string".to_string()), 
                                TokenValue::String("".to_string())
                            ]) 
                        })
                    }
                } else if symbol.starts_with("OR_GROUP:") {
                    // Handle OR groups: format is "OR_GROUP:serialized_alternatives"
                    let serialized_alternatives = &symbol[9..];
                    
                    // Try to deserialize the alternatives back to Vec<ASTNode>
                    let alternatives: Vec<ASTNode> = serde_json::from_str(serialized_alternatives)
                        .unwrap_or_else(|_| vec![ASTNode::Atom { 
                            value: ASTValue::Token(vec![
                                TokenValue::String("quoted_string".to_string()),
                                TokenValue::String("".to_string()) // Fallback to empty terminal
                            ]) 
                        }]);
                        
                    Ok(ASTNode::Or { alternatives })
                } else if symbol.starts_with("NESTED_SEQUENCE:") {
                    // Handle nested sequences: format is "NESTED_SEQUENCE:serialized_elements"
                    let serialized_elements = &symbol[16..];
                    
                    // Try to deserialize the elements back to Vec<ASTNode>
                    let elements: Vec<ASTNode> = serde_json::from_str(serialized_elements)
                        .unwrap_or_else(|_| vec![ASTNode::Atom { 
                            value: ASTValue::Token(vec![
                                TokenValue::String("quoted_string".to_string()),
                                TokenValue::String("".to_string()) // Fallback to empty terminal
                            ]) 
                        }]);
                        
                    Ok(ASTNode::Sequence { elements })
                } else if symbol.starts_with("COMPLEX_GROUP:") {
                    // Handle complex groups by deserializing back to original structure
                    let serialized_node = &symbol[14..];
                    
                    // Try to deserialize back to original ASTNode structure
                    serde_json::from_str(serialized_node)
                        .or_else(|_| {
                            // If JSON deserialization fails, try parsing as token
                            if let Ok(token) = serde_json::from_str::<Vec<TokenValue>>(serialized_node) {
                                Ok(ASTNode::Atom { value: ASTValue::Token(token) })
                            } else {
                                // Final fallback to empty terminal
                                Ok(ASTNode::Atom { 
                                    value: ASTValue::Token(vec![
                                        TokenValue::String("quoted_string".to_string()), 
                                        TokenValue::String("".to_string())
                                    ]) 
                                })
                            }
                        })
                } else if symbol.starts_with("COMPLEX:") {
                    // Legacy complex handling - convert to empty terminal
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
                    } else if symbol.starts_with("QUANTIFIED:") {
                        // Handle quantified groups in sequences
                        let parts: Vec<&str> = symbol[11..].splitn(2, ':').collect();
                        if parts.len() == 2 {
                            let quantifier = parts[0].to_string();
                            let serialized_element = parts[1];
                            
                            let element: ASTNode = serde_json::from_str(serialized_element)
                                .unwrap_or_else(|_| ASTNode::Atom { 
                                    value: ASTValue::Token(vec![
                                        TokenValue::String("quoted_string".to_string()),
                                        TokenValue::String("".to_string())
                                    ]) 
                                });
                                
                            elements.push(ASTNode::Quantified { 
                                element: Box::new(element), 
                                quantifier 
                            });
                        } else {
                            // Fallback for malformed quantified entry
                            elements.push(ASTNode::Atom { 
                                value: ASTValue::Token(vec![
                                    TokenValue::String("quoted_string".to_string()), 
                                    TokenValue::String("".to_string())
                                ]) 
                            });
                        }
                    } else if symbol.starts_with("OR_GROUP:") {
                        // Handle OR groups in sequences
                        let serialized_alternatives = &symbol[9..];
                        let alternatives: Vec<ASTNode> = serde_json::from_str(serialized_alternatives)
                            .unwrap_or_else(|_| vec![ASTNode::Atom { 
                                value: ASTValue::Token(vec![
                                    TokenValue::String("quoted_string".to_string()),
                                    TokenValue::String("".to_string())
                                ]) 
                            }]);
                        elements.push(ASTNode::Or { alternatives });
                    } else if symbol.starts_with("NESTED_SEQUENCE:") {
                        // Handle nested sequences in sequences
                        let serialized_elements = &symbol[16..];
                        let nested_elements: Vec<ASTNode> = serde_json::from_str(serialized_elements)
                            .unwrap_or_else(|_| vec![ASTNode::Atom { 
                                value: ASTValue::Token(vec![
                                    TokenValue::String("quoted_string".to_string()),
                                    TokenValue::String("".to_string())
                                ]) 
                            }]);
                        elements.push(ASTNode::Sequence { elements: nested_elements });
                    } else if symbol.starts_with("COMPLEX_GROUP:") {
                        // Handle complex groups in sequences
                        let serialized_node = &symbol[14..];
                        let node = serde_json::from_str(serialized_node)
                            .or_else(|_| {
                                if let Ok(token) = serde_json::from_str::<Vec<TokenValue>>(serialized_node) {
                                    Ok(ASTNode::Atom { value: ASTValue::Token(token) })
                                } else {
                                    Ok(ASTNode::Atom { 
                                        value: ASTValue::Token(vec![
                                            TokenValue::String("quoted_string".to_string()), 
                                            TokenValue::String("".to_string())
                                        ]) 
                                    })
                                }
                            })
                            .unwrap_or_else(|_: serde_json::Error| ASTNode::Atom { 
                                value: ASTValue::Token(vec![
                                    TokenValue::String("quoted_string".to_string()), 
                                    TokenValue::String("".to_string())
                                ]) 
                            });
                        elements.push(node);
                    } else if symbol.starts_with("COMPLEX:") {
                        // Legacy complex handling
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

    // HTML visualization functions removed

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
