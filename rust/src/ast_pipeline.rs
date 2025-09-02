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
use std::io::{self, Write};
use std::path::Path;
use anyhow::{Context, Result};

/// Configuration for AST transformation pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub debug: bool,
    pub preserve_annotations: bool,
    pub validate_input: bool,
    pub validate_output: bool,
    pub max_recursion_depth: usize,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            debug: false,
            preserve_annotations: true,
            validate_input: true,
            validate_output: true,
            max_recursion_depth: 100,
        }
    }
}

/// Raw AST token representation
pub type Token = Vec<String>;
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

/// Preserved annotations from raw AST
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Annotations {
    pub semantic_annotations: HashMap<String, Vec<String>>,
    pub logging_annotations: HashMap<String, Vec<String>>,
    pub return_annotations: HashMap<String, String>,
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
    pub annotations: Annotations,
    pub stats: TransformStats,
}

/// Main Rust AST Pipeline implementation
pub struct RustASTPipeline {
    config: PipelineConfig,
    stats: TransformStats,
    annotations: Annotations,
}

impl RustASTPipeline {
    /// Create new pipeline with configuration
    pub fn new(config: PipelineConfig) -> Self {
        Self {
            config,
            stats: TransformStats::default(),
            annotations: Annotations::default(),
        }
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
        let (grammar_tree, rule_order) = self.build_tree_structure(&quantified_rules)?;

        self.stats.rules_processed = grammar_tree.len();
        self.stats.transformations_applied = 5;

        Ok((grammar_tree, rule_order))
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

                let token_type = &token[0];
                let token_value = &token[1];

                match token_type.as_str() {
                    "rule" => {
                        rule_name = Some(token_value.clone());
                        cleaned_rule.push(token.clone());
                    }
                    "semantic_annotation" | "logging_annotation" => {
                        if let Some(ref name) = rule_name {
                            if self.config.preserve_annotations {
                                // Parse annotation format: ["annotation_type", [name, value]] for semantic
                                // or ["annotation_type", [name, [args...]]] for logging
                                if let Ok(parsed_value) = serde_json::from_str::<serde_json::Value>(token_value) {
                                    if let Some(annotation_array) = parsed_value.as_array() {
                                        if annotation_array.len() >= 2 {
                                            let annotation_name = annotation_array[0].as_str().unwrap_or("unknown");
                                            
                                            match token_type.as_str() {
                                                "semantic_annotation" => {
                                                    let annotation_value = annotation_array[1].as_str().unwrap_or("");
                                                    let formatted_annotation = format!("{}:{}", annotation_name, annotation_value);
                                                    self.annotations.semantic_annotations
                                                        .entry(name.clone())
                                                        .or_insert_with(Vec::new)
                                                        .push(formatted_annotation);
                                                }
                                                "logging_annotation" => {
                                                    let args = if let Some(args_array) = annotation_array[1].as_array() {
                                                        args_array.iter()
                                                            .filter_map(|v| v.as_str())
                                                            .collect::<Vec<_>>()
                                                            .join(",")
                                                    } else {
                                                        annotation_array[1].as_str().unwrap_or("").to_string()
                                                    };
                                                    let formatted_annotation = format!("{}({})", annotation_name, args);
                                                    self.annotations.logging_annotations
                                                        .entry(name.clone())
                                                        .or_insert_with(Vec::new)
                                                        .push(formatted_annotation);
                                                }
                                                _ => unreachable!(),
                                            }
                                        }
                                    }
                                } else {
                                    // Fallback for malformed annotation data
                                    match token_type.as_str() {
                                        "semantic_annotation" => {
                                            self.annotations.semantic_annotations
                                                .entry(name.clone())
                                                .or_insert_with(Vec::new)
                                                .push(format!("raw:{}", token_value));
                                        }
                                        "logging_annotation" => {
                                            self.annotations.logging_annotations
                                                .entry(name.clone())
                                                .or_insert_with(Vec::new)
                                                .push(format!("raw:{}", token_value));
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
                                self.annotations.return_annotations
                                    .insert(name.clone(), token_type.clone());
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
                if token.len() == 2 && token[0] == "rule" {
                    rule_name = Some(token[1].clone());
                    break;
                }
            }

            if let Some(name) = rule_name {
                let mut alternatives = Vec::new();
                let mut current_alt = TokenSequence::new();

                // Skip rule definition token
                for token in rule_def.iter().skip(1) {
                    if token.len() == 2 && token[0] == "operator" && token[1] == "|" {
                        if !current_alt.is_empty() {
                            alternatives.push(current_alt);
                            current_alt = TokenSequence::new();
                        }
                    } else {
                        current_alt.push(token.clone());
                    }
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
                        match sequence[j][0].as_str() {
                            "group_open" => paren_count += 1,
                            "group_close" => paren_count -= 1,
                            _ => {}
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
                    result.push(vec!["group".to_string(), content_json]);
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
            "group" => {
                // Deserialize group content
                let group_content: TokenSequence = serde_json::from_str(token_value)
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
                            if token.len() == 2 && token[0] == "operator" && 
                               ["*", "+", "?"].contains(&token[1].as_str()) {
                                let quantifier = token[1].clone();
                                let quantified_node = ASTNode::Quantified {
                                    element: Box::new(element.clone()),
                                    quantifier,
                                };
                                new_elements.push(quantified_node);
                                i += 2; // Skip quantifier token
                                continue;
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
            annotations: self.annotations.clone(),
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
                vec!["rule".to_string(), "test".to_string()],
                vec!["semantic_annotation".to_string(), "@type:test".to_string()],
                vec!["quoted_string".to_string(), "literal".to_string()],
            ]
        ];

        let cleaned = pipeline.extract_annotations(&raw_ast).unwrap();
        assert_eq!(pipeline.stats.annotations_preserved, 1);
        assert!(pipeline.annotations.semantic_annotations.contains_key("test"));
    }
}
