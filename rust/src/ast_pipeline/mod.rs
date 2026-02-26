// Shared Logger trait that both binaries can access
pub trait Logger: std::fmt::Debug {
    fn is_enabled(&self) -> bool;
    fn log_info(&self, file: &str, line: u32, message: &str);
    fn log_debug(&self, file: &str, line: u32, message: &str);
    fn log_success(&self, file: &str, line: u32, message: &str);
    fn log_warning(&self, file: &str, line: u32, message: &str);
    fn log_error(&self, file: &str, line: u32, message: &str);

    // Clone method for logger instances
    fn clone_box(&self) -> Box<dyn Logger>;
}

// No-op logger implementation
#[derive(Debug, Clone)]
pub struct NoOpLogger;

impl Logger for NoOpLogger {
    fn is_enabled(&self) -> bool {
        false
    }
    fn log_info(&self, _file: &str, _line: u32, _message: &str) {}
    fn log_debug(&self, _file: &str, _line: u32, _message: &str) {}
    fn log_success(&self, _file: &str, _line: u32, _message: &str) {}
    fn log_warning(&self, _file: &str, _line: u32, _message: &str) {}
    fn log_error(&self, _file: &str, _line: u32, _message: &str) {}

    fn clone_box(&self) -> Box<dyn Logger> {
        Box::new(self.clone())
    }
}

use anyhow::Result;
use serde;
use std::collections::{HashMap, HashSet};

#[cfg(feature = "generated_parsers")]
use crate::generated_parsers::return_annotation::Return_annotationParser;
#[cfg(feature = "generated_parsers")]
use crate::generated_parsers::semantic_annotation::Semantic_annotationParser;

// Shared parser types used by generated parsers
/// Parse result type
pub type ParseResult<T> = Result<T, ParseError>;

/// Parse errors
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    UnexpectedEof {
        position: usize,
    },
    UnexpectedToken {
        expected: &'static str,
        found: char,
        position: usize,
    },
    InvalidSyntax {
        message: &'static str,
        position: usize,
    },
    Backtrack {
        position: usize,
    },
    RecursionDepthExceeded {
        position: usize,
        depth: usize,
    },
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
            ParseError::UnexpectedEof { position } => {
                write!(f, "Unexpected EOF at position {}", position)
            }
            ParseError::UnexpectedToken {
                expected,
                found,
                position,
            } => {
                write!(
                    f,
                    "Expected '{}', found '{}' at position {}",
                    expected, found, position
                )
            }
            ParseError::InvalidSyntax { message, position } => {
                write!(f, "{} at position {}", message, position)
            }
            ParseError::Backtrack { position } => {
                write!(f, "Backtrack at position {}", position)
            }
            ParseError::RecursionDepthExceeded { position, depth } => {
                write!(
                    f,
                    "Recursion depth exceeded ({} levels) at position {}",
                    depth, position
                )
            }
            ParseError::ContextualError {
                message,
                position,
                rule_stack,
                input_context,
            } => {
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
    TransformedTerminal(String),
    Sequence(Vec<ParseNode<'input>>),
    Alternative(Box<ParseNode<'input>>),
    Quantified(Vec<ParseNode<'input>>, &'static str),
}

/// Parse node
#[derive(Debug, Clone, PartialEq)]
pub struct ParseNode<'input> {
    pub rule_name: &'static str,
    pub content: ParseContent<'input>,
    pub span: std::ops::Range<usize>,
}

/// Memoization entry
#[derive(Debug, Clone)]
pub struct MemoEntry<'input> {
    pub result: Option<ParseNode<'input>>,
    pub end_pos: usize,
}

/// Rule ID type for memoization
pub type RuleId = u16;

/// Recursion cycle types
#[derive(Debug, Clone, PartialEq)]
pub enum CycleType {
    None,
    Infinite,
    LeftRecursive,
    MutualRecursive { depth: usize, rules: Vec<String> },
}

/// Recursion guard
#[derive(Debug, Clone)]
pub struct RecursionGuard {
    pub parse_stack: Vec<(String, usize)>,
    pub max_depth: usize,
    pub cycle_cache: HashMap<(String, usize), CycleType>,
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
                self.cycle_cache
                    .insert((rule_name.to_string(), position), cycle.clone());
                return cycle;
            }
            if r == rule_name && *p > position {
                let cycle = CycleType::LeftRecursive;
                self.cycle_cache
                    .insert((rule_name.to_string(), position), cycle.clone());
                return cycle;
            }
        }
        if self.parse_stack.len() >= self.max_depth {
            let rules: Vec<String> = self.parse_stack.iter().map(|(r, _)| r.clone()).collect();
            let cycle = CycleType::MutualRecursive {
                depth: self.parse_stack.len(),
                rules,
            };
            self.cycle_cache
                .insert((rule_name.to_string(), position), cycle.clone());
            return cycle;
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

#[derive(Debug, Clone, serde::Deserialize)]
pub enum ASTValue {
    Token(Vec<TokenValue>),
    Node(Box<ASTNode>),
}

#[derive(Debug, Clone, serde::Deserialize)]
pub enum TokenValue {
    String(String),
}

#[derive(Debug, Clone, serde::Deserialize)]
pub enum ASTNode {
    Or {
        alternatives: Vec<ASTNode>,
    },
    Sequence {
        elements: Vec<ASTNode>,
    },
    Atom {
        value: ASTValue,
    },
    Quantified {
        element: Box<ASTNode>,
        quantifier: String,
    },
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct BranchAnnotation {
    pub annotation_type: String,
    pub annotation_content: String,
    pub parsed_ast: Option<UnifiedReturnAST>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq)]
#[serde(untagged)]
pub enum SemanticAnnotation {
    Legacy(UnifiedSemanticAST),
    Named {
        name: String,
        ast: UnifiedSemanticAST,
    },
}

impl SemanticAnnotation {
    pub fn ast(&self) -> &UnifiedSemanticAST {
        match self {
            SemanticAnnotation::Legacy(ast) => ast,
            SemanticAnnotation::Named { ast, .. } => ast,
        }
    }

    pub fn name(&self) -> Option<&str> {
        match self {
            SemanticAnnotation::Legacy(_) => None,
            SemanticAnnotation::Named { name, .. } => Some(name.as_str()),
        }
    }
}

impl From<UnifiedSemanticAST> for SemanticAnnotation {
    fn from(value: UnifiedSemanticAST) -> Self {
        SemanticAnnotation::Legacy(value)
    }
}

#[derive(Debug, Clone, serde::Deserialize, Default)]
pub struct Annotations {
    #[serde(default)]
    pub branch_return_annotations: std::collections::HashMap<String, Vec<Option<BranchAnnotation>>>,
    #[serde(default)]
    pub semantic_annotations: std::collections::HashMap<String, Vec<SemanticAnnotation>>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TransformMetadata {
    pub format: String,
    pub source_format: String,
    pub transformed_at: String,
    pub transformer: String,
    pub pipeline_stage: String,
    pub annotations: Option<Annotations>,
    #[serde(default)]
    pub stats: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TransformedASTJson {
    pub grammar_name: String,
    pub grammar_tree: std::collections::HashMap<String, ASTNode>,
    pub rule_order: Vec<String>,
    pub metadata: TransformMetadata,
}

// Type aliases for compatibility
// pub type ParseNode<'input> = ASTNode;  // Removed - now using full ParseNode struct

pub struct PipelineConfig {
    pub debug: bool,
    pub trace: bool,
    pub bootstrap_mode: bool,
    pub preserve_annotations: bool,
    pub validate_input: bool,
    pub validate_output: bool,
    pub max_recursion_depth: usize,
    pub eliminate_left_recursion: bool,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        PipelineConfig {
            debug: false,
            trace: false,
            bootstrap_mode: false,
            preserve_annotations: true,
            validate_input: true,
            validate_output: true,
            max_recursion_depth: 100,
            eliminate_left_recursion: true,
        }
    }
}

pub struct RustASTPipeline {
    config: PipelineConfig,
}

#[derive(Debug, Clone)]
struct LeftRecursiveChainPlan {
    base_rule: String,
    helper_base_rule: String,
    base_alternatives: Vec<ASTNode>,
    wrapper_rules: Vec<(String, ASTNode)>,
    suffix_alternative: ASTNode,
}

#[derive(Debug, Clone)]
enum RawRuleElement {
    Atom(ASTNode),
    OrOperator,
    GroupOpen,
    GroupClose,
    Quantifier(String),
}

#[derive(Debug, Clone)]
struct ParsedRuleContent {
    ast_node: ASTNode,
    branch_return_annotations: Vec<Option<BranchAnnotation>>,
    semantic_annotations: Vec<SemanticAnnotation>,
}

#[derive(Debug, Clone)]
struct ExtractedRuleAnnotations {
    syntax_elements: Vec<serde_json::Value>,
    branch_return_annotations: Vec<Option<BranchAnnotation>>,
    semantic_annotations: Vec<SemanticAnnotation>,
}

impl RustASTPipeline {
    pub fn new(config: PipelineConfig) -> Self {
        RustASTPipeline { config }
    }

    /// Transform raw AST JSON into processed AST format
    pub fn transform_from_raw_ast(
        &self,
        raw_ast_data: &[serde_json::Value],
    ) -> Result<(HashMap<String, ASTNode>, Vec<String>, Option<Annotations>)> {
        eprintln!("\n{}", "=".repeat(80));
        eprintln!("🔄  AST PIPELINE TRANSFORMATION STARTED");
        eprintln!("{}", "=".repeat(80));
        eprintln!(
            "📊  Processing {} raw AST elements into structured grammar",
            raw_ast_data.len()
        );
        eprintln!("📂  File: {}:{}", file!(), line!());
        eprintln!();

        let mut grammar_tree = HashMap::new();
        let mut rule_order = Vec::new();
        let mut annotations = Annotations::default();

        for (rule_idx, rule_data) in raw_ast_data.iter().enumerate() {
            eprintln!("   📋  Rule {}/{}", rule_idx + 1, raw_ast_data.len());
            eprintln!(
                "       Raw JSON: {}",
                rule_data.to_string().chars().take(80).collect::<String>()
                    + if rule_data.to_string().len() > 80 {
                        "..."
                    } else {
                        ""
                    }
            );
            eprintln!("       File: {}:{}", file!(), line!());

            if let Some(rule_array) = rule_data.as_array() {
                if rule_array.is_empty() {
                    eprintln!("       ⚠️   WARNING: Skipping empty rule array");
                    eprintln!("       File: {}:{}", file!(), line!());
                    eprintln!();
                    continue;
                }

                // First element should be ["rule", "rule_name"]
                if let Some(first_elem) = rule_array.first() {
                    if let Some(rule_name) = self.extract_rule_name(first_elem) {
                        eprintln!("       ✅  Rule declaration found: '{}' ", rule_name);
                        eprintln!("       File: {}:{}", file!(), line!());
                        rule_order.push(rule_name.clone());

                        // Parse the rule content (everything after the rule declaration)
                        let rule_content = &rule_array[1..];
                        eprintln!(
                            "       🔍  Parsing {} content elements for rule '{}'",
                            rule_content.len(),
                            rule_name
                        );
                        eprintln!("       File: {}:{}", file!(), line!());

                        let parsed_rule = self.parse_rule_content(rule_content)?;

                        eprintln!(
                            "       🎯  Rule '{}' successfully transformed to AST",
                            rule_name
                        );
                        eprintln!("       Result: {:?}", parsed_rule.ast_node);
                        eprintln!("       File: {}:{}", file!(), line!());
                        if self.config.preserve_annotations {
                            if parsed_rule
                                .branch_return_annotations
                                .iter()
                                .any(|entry| entry.is_some())
                            {
                                annotations.branch_return_annotations.insert(
                                    rule_name.clone(),
                                    parsed_rule.branch_return_annotations.clone(),
                                );
                            }
                            if !parsed_rule.semantic_annotations.is_empty() {
                                annotations.semantic_annotations.insert(
                                    rule_name.clone(),
                                    parsed_rule.semantic_annotations.clone(),
                                );
                            }
                        }
                        grammar_tree.insert(rule_name, parsed_rule.ast_node);
                        eprintln!();
                    } else {
                        eprintln!("       ❌  ERROR: Failed to extract rule name from element");
                        eprintln!("       Element: {:?}", first_elem);
                        eprintln!("       File: {}:{}", file!(), line!());
                        eprintln!();
                    }
                } else {
                    eprintln!("       ❌  ERROR: Rule array has no first element");
                    eprintln!("       File: {}:{}", file!(), line!());
                    eprintln!();
                }
            } else {
                eprintln!("       ❌  ERROR: Rule data is not an array");
                eprintln!(
                    "       Data type: {}",
                    std::any::type_name::<serde_json::Value>()
                );
                eprintln!("       File: {}:{}", file!(), line!());
                eprintln!();
            }
        }

        if self.config.eliminate_left_recursion {
            self.eliminate_left_recursive_patterns(&mut grammar_tree, &mut rule_order);
        } else {
            eprintln!(
                "[mod.rs][transform_from_raw_ast()] ⏭️  Left-recursion elimination disabled by configuration"
            );
        }

        eprintln!("🎉  TRANSFORMATION COMPLETE");
        eprintln!("📊  Generated grammar with {} rules", grammar_tree.len());
        eprintln!("📋  Rule execution order: {:?}", rule_order);
        eprintln!("📂  File: {}:{}", file!(), line!());
        eprintln!("{}", "=".repeat(80));
        eprintln!();

        let annotations = if self.config.preserve_annotations
            && (!annotations.branch_return_annotations.is_empty()
                || !annotations.semantic_annotations.is_empty())
        {
            Some(annotations)
        } else {
            None
        };

        Ok((grammar_tree, rule_order, annotations))
    }

    fn eliminate_left_recursive_patterns(
        &self,
        grammar_tree: &mut HashMap<String, ASTNode>,
        rule_order: &mut Vec<String>,
    ) {
        eprintln!(
            "[mod.rs][eliminate_left_recursive_patterns()] 🔧 Starting left-recursion elimination pass"
        );
        let original_order = rule_order.clone();
        let mut transformed_rules = HashSet::new();
        let mut transformation_count = 0usize;

        for rule_name in original_order {
            if transformed_rules.contains(&rule_name) {
                continue;
            }

            let Some(plan) = self.detect_left_recursive_chain_plan(&rule_name, grammar_tree) else {
                continue;
            };

            eprintln!(
                "[mod.rs][eliminate_left_recursive_patterns()] ✅ Rewriting left-recursive chain for rule '{}' via helper '{}' ({} wrapper rules)",
                plan.base_rule,
                plan.helper_base_rule,
                plan.wrapper_rules.len()
            );

            self.apply_left_recursive_chain_plan(&plan, grammar_tree, rule_order);
            transformation_count += 1;
            transformed_rules.insert(plan.base_rule.clone());
            for (wrapper_rule, _) in &plan.wrapper_rules {
                transformed_rules.insert(wrapper_rule.clone());
            }
            transformed_rules.insert(plan.helper_base_rule.clone());
        }

        eprintln!(
            "[mod.rs][eliminate_left_recursive_patterns()] 🏁 Completed left-recursion elimination pass ({} transformations)",
            transformation_count
        );
    }

    fn detect_left_recursive_chain_plan(
        &self,
        rule_name: &str,
        grammar_tree: &HashMap<String, ASTNode>,
    ) -> Option<LeftRecursiveChainPlan> {
        let rule_node = grammar_tree.get(rule_name)?;
        let rule_alternatives = Self::as_alternatives(rule_node);
        if rule_alternatives.is_empty() {
            return None;
        }

        let mut base_alternatives = Vec::new();
        let mut wrapper_rules: Vec<(String, ASTNode)> = Vec::new();

        for alternative in &rule_alternatives {
            if let Some(wrapper_rule) = Self::extract_rule_reference_name(alternative) {
                if let Some(wrapper_suffix) =
                    Self::extract_wrapper_suffix(rule_name, &wrapper_rule, grammar_tree)
                {
                    wrapper_rules.push((wrapper_rule, wrapper_suffix));
                    continue;
                }
            }
            base_alternatives.push(alternative.clone());
        }

        if wrapper_rules.is_empty() || base_alternatives.is_empty() {
            return None;
        }

        let suffix_alternative = Self::build_or_node(
            wrapper_rules
                .iter()
                .map(|(_, suffix)| suffix.clone())
                .collect(),
        );

        let helper_base_rule =
            Self::allocate_synthetic_rule_name(format!("{}_lr_base", rule_name), grammar_tree);

        Some(LeftRecursiveChainPlan {
            base_rule: rule_name.to_string(),
            helper_base_rule,
            base_alternatives,
            wrapper_rules,
            suffix_alternative,
        })
    }

    fn apply_left_recursive_chain_plan(
        &self,
        plan: &LeftRecursiveChainPlan,
        grammar_tree: &mut HashMap<String, ASTNode>,
        rule_order: &mut Vec<String>,
    ) {
        let helper_base_ref = Self::make_rule_reference_node(&plan.helper_base_rule);
        let suffix_repetition = ASTNode::Quantified {
            element: Box::new(plan.suffix_alternative.clone()),
            quantifier: "*".to_string(),
        };

        let rewritten_base_rule =
            Self::build_sequence_node(vec![helper_base_ref.clone(), suffix_repetition.clone()]);
        grammar_tree.insert(plan.base_rule.clone(), rewritten_base_rule);

        for (wrapper_rule, wrapper_suffix) in &plan.wrapper_rules {
            let rewritten_wrapper = Self::build_sequence_node(vec![
                helper_base_ref.clone(),
                wrapper_suffix.clone(),
                suffix_repetition.clone(),
            ]);
            grammar_tree.insert(wrapper_rule.clone(), rewritten_wrapper);
        }

        let helper_base_ast = Self::build_or_node(plan.base_alternatives.clone());
        grammar_tree.insert(plan.helper_base_rule.clone(), helper_base_ast);

        if !rule_order.contains(&plan.helper_base_rule) {
            if let Some(base_pos) = rule_order.iter().position(|name| name == &plan.base_rule) {
                rule_order.insert(base_pos, plan.helper_base_rule.clone());
            } else {
                rule_order.push(plan.helper_base_rule.clone());
            }
        }
    }

    fn as_alternatives(node: &ASTNode) -> Vec<ASTNode> {
        match node {
            ASTNode::Or { alternatives } => alternatives.clone(),
            _ => vec![node.clone()],
        }
    }

    fn build_or_node(mut alternatives: Vec<ASTNode>) -> ASTNode {
        if alternatives.len() == 1 {
            alternatives.remove(0)
        } else {
            ASTNode::Or { alternatives }
        }
    }

    fn build_sequence_node(mut elements: Vec<ASTNode>) -> ASTNode {
        if elements.len() == 1 {
            elements.remove(0)
        } else {
            ASTNode::Sequence { elements }
        }
    }

    fn make_rule_reference_node(rule_name: &str) -> ASTNode {
        ASTNode::Atom {
            value: ASTValue::Token(vec![
                TokenValue::String("rule_reference".to_string()),
                TokenValue::String(rule_name.to_string()),
            ]),
        }
    }

    fn allocate_synthetic_rule_name(
        base_name: String,
        grammar_tree: &HashMap<String, ASTNode>,
    ) -> String {
        if !grammar_tree.contains_key(&base_name) {
            return base_name;
        }

        let mut index = 1usize;
        loop {
            let candidate = format!("{}_{}", base_name, index);
            if !grammar_tree.contains_key(&candidate) {
                return candidate;
            }
            index += 1;
        }
    }

    fn extract_rule_reference_name(node: &ASTNode) -> Option<String> {
        match node {
            ASTNode::Atom {
                value: ASTValue::Token(parts),
            } => {
                if parts.len() < 2 {
                    return None;
                }
                let TokenValue::String(token_type) = &parts[0] else {
                    return None;
                };
                let TokenValue::String(token_value) = &parts[1] else {
                    return None;
                };
                if token_type == "rule_reference" {
                    Some(token_value.clone())
                } else {
                    None
                }
            }
            ASTNode::Sequence { elements } if elements.len() == 1 => {
                Self::extract_rule_reference_name(&elements[0])
            }
            _ => None,
        }
    }

    fn sequence_suffix_if_prefixed_with_rule(
        elements: &[ASTNode],
        base_rule: &str,
    ) -> Option<ASTNode> {
        if elements.is_empty() {
            return None;
        }
        if Self::extract_rule_reference_name(&elements[0]).as_deref() != Some(base_rule) {
            return None;
        }
        if elements.len() < 2 {
            return None;
        }
        Some(Self::build_sequence_node(elements[1..].to_vec()))
    }

    fn extract_wrapper_suffix(
        base_rule: &str,
        wrapper_rule: &str,
        grammar_tree: &HashMap<String, ASTNode>,
    ) -> Option<ASTNode> {
        let wrapper_node = grammar_tree.get(wrapper_rule)?;
        match wrapper_node {
            ASTNode::Sequence { elements } => {
                Self::sequence_suffix_if_prefixed_with_rule(elements, base_rule)
            }
            ASTNode::Or { alternatives } => {
                let mut suffixes = Vec::new();
                for alternative in alternatives {
                    let ASTNode::Sequence { elements } = alternative else {
                        return None;
                    };
                    let Some(suffix) =
                        Self::sequence_suffix_if_prefixed_with_rule(elements, base_rule)
                    else {
                        return None;
                    };
                    suffixes.push(suffix);
                }
                if suffixes.is_empty() {
                    None
                } else {
                    Some(Self::build_or_node(suffixes))
                }
            }
            _ => None,
        }
    }

    fn extract_rule_name(&self, rule_decl: &serde_json::Value) -> Option<String> {
        if let Some(arr) = rule_decl.as_array() {
            if arr.len() >= 2 {
                if let (Some(type_str), Some(name_str)) = (arr[0].as_str(), arr[1].as_str()) {
                    if type_str == "rule" {
                        return Some(name_str.to_string());
                    }
                }
            }
        }
        None
    }

    fn parse_rule_content(&self, content: &[serde_json::Value]) -> Result<ParsedRuleContent> {
        if content.is_empty() {
            eprintln!(
                "[mod.rs][parse_rule_content()] 📝 Rule content is empty - creating empty sequence node"
            );
            eprintln!("   File: {}:{}", file!(), line!());
            return Ok(ParsedRuleContent {
                ast_node: ASTNode::Sequence { elements: vec![] },
                branch_return_annotations: vec![None],
                semantic_annotations: Vec::new(),
            });
        }

        eprintln!("   🏗️   RULE CONTENT PARSING (STAGED PIPELINE)");
        eprintln!("        Elements to process: {}", content.len());
        eprintln!("        File: {}:{}", file!(), line!());

        let extracted = self.extract_rule_annotations(content)?;
        eprintln!(
            "        Annotation extraction: {} branch return slot(s), {} semantic annotation(s)",
            extracted.branch_return_annotations.len(),
            extracted.semantic_annotations.len()
        );

        if extracted.syntax_elements.is_empty() {
            let mut branch_return_annotations = extracted.branch_return_annotations;
            if branch_return_annotations.is_empty() {
                branch_return_annotations.push(None);
            }
            return Ok(ParsedRuleContent {
                ast_node: ASTNode::Sequence { elements: vec![] },
                branch_return_annotations,
                semantic_annotations: extracted.semantic_annotations,
            });
        }

        eprintln!("        Stage-1: normalize raw elements");
        let normalized = self.step1_normalize_raw_elements(&extracted.syntax_elements)?;
        eprintln!(
            "        Stage-1 result: {} normalized elements",
            normalized.len()
        );
        eprintln!("        Stage-2: group top-level alternatives (|)");
        let branches = self.step2_group_by_or(&normalized);
        eprintln!(
            "        Stage-2 result: {} top-level branches",
            branches.len()
        );
        eprintln!("        Stage-2.5: handle parentheses/groups per branch");
        let mut branch_asts = Vec::with_capacity(branches.len());
        for (branch_idx, branch) in branches.iter().enumerate() {
            eprintln!(
                "          🔀 Branch {}/{} has {} elements",
                branch_idx + 1,
                branches.len(),
                branch.len()
            );
            let branch_elements = self.step2_5_handle_parentheses(branch)?;
            eprintln!(
                "          ✅ Branch {} grouped into {} sequence elements",
                branch_idx + 1,
                branch_elements.len()
            );
            eprintln!("          Stage-3: build sequence nodes");
            let branch_ast = self.step3_parse_sequences(branch_elements);
            branch_asts.push(branch_ast);
        }
        eprintln!("        Stage-5: build final tree structure");
        let result = self.step5_build_tree_structure(branch_asts);

        eprintln!("   🏆  Rule content parsing complete (staged pipeline)");
        eprintln!("       Final AST: {:?}", result);
        eprintln!("       File: {}:{}", file!(), line!());

        let mut branch_return_annotations = extracted.branch_return_annotations;
        let branch_count = match &result {
            ASTNode::Or { alternatives } => alternatives.len(),
            _ => 1,
        };
        if branch_return_annotations.len() < branch_count {
            branch_return_annotations.resize(branch_count, None);
        } else if branch_return_annotations.len() > branch_count {
            branch_return_annotations.truncate(branch_count);
        }

        Ok(ParsedRuleContent {
            ast_node: result,
            branch_return_annotations,
            semantic_annotations: extracted.semantic_annotations,
        })
    }

    fn extract_rule_annotations(
        &self,
        content: &[serde_json::Value],
    ) -> Result<ExtractedRuleAnnotations> {
        let mut syntax_elements = Vec::with_capacity(content.len());
        let mut branch_return_annotations: Vec<Option<BranchAnnotation>> = vec![None];
        let mut semantic_annotations = Vec::new();

        let mut group_depth = 0usize;
        let mut branch_idx = 0usize;

        for item in content {
            let Some(arr) = item.as_array() else {
                syntax_elements.push(item.clone());
                continue;
            };
            let Some(elem_type) = arr.first().and_then(|v| v.as_str()) else {
                syntax_elements.push(item.clone());
                continue;
            };

            match elem_type {
                "group_open" => {
                    group_depth = group_depth.saturating_add(1);
                    syntax_elements.push(item.clone());
                }
                "group_close" => {
                    group_depth = group_depth.saturating_sub(1);
                    syntax_elements.push(item.clone());
                }
                "operator" => {
                    if arr.get(1).and_then(|v| v.as_str()) == Some("|") && group_depth == 0 {
                        branch_idx = branch_idx.saturating_add(1);
                        if branch_return_annotations.len() <= branch_idx {
                            branch_return_annotations.push(None);
                        }
                    }
                    syntax_elements.push(item.clone());
                }
                "return_scalar" | "return_array" | "return_object" => {
                    let Some(annotation_content) = arr.get(1).and_then(|v| v.as_str()) else {
                        eprintln!(
                            "[mod.rs][extract_rule_annotations()] ⚠️ malformed return annotation payload: {:?}",
                            item
                        );
                        continue;
                    };
                    if branch_return_annotations.len() <= branch_idx {
                        branch_return_annotations.resize(branch_idx + 1, None);
                    }
                    if branch_return_annotations[branch_idx].is_some() {
                        eprintln!(
                            "[mod.rs][extract_rule_annotations()] ⚠️ multiple return annotations in branch {} - keeping last",
                            branch_idx + 1
                        );
                    }
                    let parsed_ast = self.parse_return_annotation_ast(annotation_content);
                    branch_return_annotations[branch_idx] = Some(BranchAnnotation {
                        annotation_type: elem_type.to_string(),
                        annotation_content: annotation_content.to_string(),
                        parsed_ast,
                    });
                }
                "semantic_annotation" => {
                    if let Some(payload) = arr.get(1) {
                        if let Some(annotation) =
                            self.parse_semantic_annotation_entry(payload, item)?
                        {
                            semantic_annotations.push(annotation);
                        }
                    } else {
                        eprintln!(
                            "[mod.rs][extract_rule_annotations()] ⚠️ semantic annotation missing payload: {:?}",
                            item
                        );
                    }
                }
                _ => syntax_elements.push(item.clone()),
            }
        }

        Ok(ExtractedRuleAnnotations {
            syntax_elements,
            branch_return_annotations,
            semantic_annotations,
        })
    }

    fn parse_return_annotation_ast(&self, annotation_content: &str) -> Option<UnifiedReturnAST> {
        let content = annotation_content.trim();
        if content.is_empty() {
            return None;
        }

        let logger = NoOpLogger;
        if !self.config.bootstrap_mode {
            if !self.validate_return_annotation_backend(content) {
                eprintln!(
                    "[mod.rs][parse_return_annotation_ast()] ⚠️ selected backend could not validate return annotation '{}'",
                    content
                );
                return None;
            }

            #[cfg(feature = "generated_parsers")]
            {
                let mut parser = Return_annotationParser::new(content, Box::new(NoOpLogger));
                match parser.parse_full_return_annotation() {
                    Ok(parse_tree) => {
                        return match UnifiedReturnAST::parse_generated_return_annotation(
                            content,
                            &parse_tree,
                            &logger,
                        ) {
                            Ok(ast) => Some(ast),
                            Err(err) => {
                                eprintln!(
                                    "[mod.rs][parse_return_annotation_ast()] ⚠️ generated return tree -> typed AST failed for '{}' ({})",
                                    content, err
                                );
                                None
                            }
                        };
                    }
                    Err(err) => {
                        eprintln!(
                            "[mod.rs][parse_return_annotation_ast()] ⚠️ generated parser failed for '{}' ({})",
                            content, err
                        );
                        return None;
                    }
                }
            }
        }

        match UnifiedReturnAST::parse_bootstrap(content, &logger) {
            Ok(ast) => Some(ast),
            Err(err) => {
                eprintln!(
                    "[mod.rs][parse_return_annotation_ast()] ⚠️ failed to build typed return AST for '{}' ({})",
                    content, err
                );
                None
            }
        }
    }

    fn parse_semantic_annotation_entry(
        &self,
        payload: &serde_json::Value,
        original_element: &serde_json::Value,
    ) -> Result<Option<SemanticAnnotation>> {
        match payload {
            serde_json::Value::Array(parts) if parts.len() >= 2 => {
                let name = self.semantic_value_to_string(&parts[0]);
                let annotation_name = name.trim().to_ascii_lowercase();
                if annotation_name.is_empty() {
                    eprintln!(
                        "[mod.rs][parse_semantic_annotation_entry()] ⚠️ empty semantic annotation name in {:?}",
                        original_element
                    );
                    return Ok(None);
                }

                let payload_text = self.semantic_value_to_string(&parts[1]);
                let canonical = format!("@{}: {}", annotation_name, payload_text);
                let backend_valid = self.validate_semantic_annotation_backend(&canonical);
                if !backend_valid {
                    eprintln!(
                        "[mod.rs][parse_semantic_annotation_entry()] ⚠️ selected backend could not validate semantic annotation '{}'",
                        canonical
                    );
                }

                Ok(Some(SemanticAnnotation::Named {
                    name: annotation_name.clone(),
                    ast: self.parse_semantic_annotation_ast(
                        &annotation_name,
                        &payload_text,
                        backend_valid,
                    )?,
                }))
            }
            serde_json::Value::String(text) => {
                let trimmed = text.trim();
                if trimmed.is_empty() {
                    return Ok(None);
                }

                if let Some((name, payload)) =
                    self::semantic_directive_registry::extract_semantic_directive(trimmed)
                {
                    let backend_valid = self.validate_semantic_annotation_backend(trimmed);
                    if !backend_valid {
                        eprintln!(
                            "[mod.rs][parse_semantic_annotation_entry()] ⚠️ selected backend could not validate semantic annotation '{}'",
                            trimmed
                        );
                    }
                    return Ok(Some(SemanticAnnotation::Named {
                        name: name.clone(),
                        ast: self.parse_semantic_annotation_ast(&name, &payload, backend_valid)?,
                    }));
                }

                let ast = if self.config.bootstrap_mode {
                    let logger = NoOpLogger;
                    UnifiedSemanticAST::parse_bootstrap(trimmed, &logger).unwrap_or_else(|_| {
                        UnifiedSemanticAST::Raw {
                            content: trimmed.to_string(),
                        }
                    })
                } else {
                    // Only directive-shaped strings should go through generated semantic
                    // full-parse conversion in non-bootstrap mode.
                    if trimmed.starts_with('@') {
                        if let Some((name, ast)) =
                            self.parse_semantic_annotation_with_generated_parser(trimmed)?
                        {
                            return Ok(Some(SemanticAnnotation::Named { name, ast }));
                        }
                    }
                    // In non-bootstrap mode, do not apply bootstrap marker heuristics.
                    // Non-directive payload is intentionally preserved as raw content.
                    UnifiedSemanticAST::Raw {
                        content: trimmed.to_string(),
                    }
                };
                Ok(Some(SemanticAnnotation::Legacy(ast)))
            }
            _ => {
                let raw = self.semantic_value_to_string(payload);
                if raw.trim().is_empty() {
                    return Ok(None);
                }
                Ok(Some(SemanticAnnotation::Legacy(UnifiedSemanticAST::Raw {
                    content: raw,
                })))
            }
        }
    }

    fn parse_semantic_annotation_ast(
        &self,
        annotation_name: &str,
        payload: &str,
        backend_valid: bool,
    ) -> Result<UnifiedSemanticAST> {
        let normalized_name = annotation_name.trim().to_ascii_lowercase();
        let canonical = format!("@{}: {}", normalized_name, payload.trim());

        if backend_valid {
            if let Some((parsed_name, ast)) =
                self.parse_semantic_annotation_with_generated_parser(&canonical)?
            {
                if parsed_name != normalized_name {
                    return Err(anyhow::anyhow!(
                        "named semantic annotation parse produced mismatched name '{}' (expected '{}') for canonical '{}'",
                        parsed_name,
                        normalized_name,
                        canonical
                    ));
                }
                return Ok(ast);
            }
        }

        Ok(self.semantic_named_ast(&normalized_name, payload))
    }

    fn parse_semantic_annotation_with_generated_parser(
        &self,
        annotation_text: &str,
    ) -> Result<Option<(String, UnifiedSemanticAST)>> {
        if self.config.bootstrap_mode {
            return Ok(None);
        }

        #[cfg(feature = "generated_parsers")]
        {
            let logger = NoOpLogger;
            let mut parser = Semantic_annotationParser::new(annotation_text, Box::new(NoOpLogger));
            let parse_tree = parser.parse_full_semantic_annotation().map_err(|err| {
                anyhow::anyhow!(
                    "generated semantic parser failed for '{}': {}",
                    annotation_text,
                    err
                )
            })?;
            let entry = UnifiedSemanticAST::parse_generated_semantic_annotation_entry(
                annotation_text,
                &parse_tree,
                &logger,
            )
            .map_err(|err| {
                anyhow::anyhow!(
                    "generated semantic tree -> typed AST failed for '{}': {}",
                    annotation_text,
                    err
                )
            })?;
            return Ok(Some(entry));
        }

        #[cfg(not(feature = "generated_parsers"))]
        {
            let _ = annotation_text;
            Ok(None)
        }
    }

    fn semantic_named_ast(&self, name: &str, payload: &str) -> UnifiedSemanticAST {
        if name.eq_ignore_ascii_case("transform") {
            UnifiedSemanticAST::TransformExpr {
                expression: payload.trim().to_string(),
            }
        } else {
            UnifiedSemanticAST::Raw {
                content: payload.trim().to_string(),
            }
        }
    }

    fn semantic_value_to_string(&self, value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::String(text) => text.clone(),
            _ => value.to_string(),
        }
    }

    fn validate_return_annotation_backend(&self, annotation_content: &str) -> bool {
        if self.config.bootstrap_mode {
            return true;
        }

        #[cfg(feature = "generated_parsers")]
        {
            let mut parser = Return_annotationParser::new(annotation_content, Box::new(NoOpLogger));
            return parser.parse_full_return_annotation().is_ok();
        }

        #[cfg(not(feature = "generated_parsers"))]
        {
            let _ = annotation_content;
            eprintln!(
                "[mod.rs][validate_return_annotation_backend()] ⚠️ generated parser backend unavailable (build with --features generated_parsers)"
            );
            true
        }
    }

    fn validate_semantic_annotation_backend(&self, annotation_text: &str) -> bool {
        if self.config.bootstrap_mode {
            return true;
        }

        #[cfg(feature = "generated_parsers")]
        {
            let mut parser = Semantic_annotationParser::new(annotation_text, Box::new(NoOpLogger));
            return parser.parse_full_semantic_annotation().is_ok();
        }

        #[cfg(not(feature = "generated_parsers"))]
        {
            let _ = annotation_text;
            eprintln!(
                "[mod.rs][validate_semantic_annotation_backend()] ⚠️ generated parser backend unavailable (build with --features generated_parsers)"
            );
            true
        }
    }

    fn step1_normalize_raw_elements(
        &self,
        content: &[serde_json::Value],
    ) -> Result<Vec<RawRuleElement>> {
        eprintln!("[mod.rs][step1_normalize_raw_elements()] 🔎 Start normalization");
        let mut normalized = Vec::new();

        for (elem_idx, item) in content.iter().enumerate() {
            eprintln!("        🔧  Element {}/{}", elem_idx + 1, content.len());
            eprintln!("            Raw data: {:?}", item);
            eprintln!("            File: {}:{}", file!(), line!());
            if let Some(parsed) = self.parse_raw_element(item)? {
                eprintln!(
                    "            ✅  Normalized element kind: {}",
                    self.raw_element_kind(&parsed)
                );
                normalized.push(parsed);
            } else {
                eprintln!("            ⚠️   Element skipped (return annotation or unknown type)");
            }
            eprintln!();
        }

        Ok(normalized)
    }

    fn parse_raw_element(&self, element: &serde_json::Value) -> Result<Option<RawRuleElement>> {
        let Some(arr) = element.as_array() else {
            eprintln!("            ❌  [mod.rs][parse_raw_element()] Element is not array");
            return Ok(None);
        };

        if arr.len() < 2 {
            eprintln!(
                "            ❌  [mod.rs][parse_raw_element()] Element array too short: {}",
                arr.len()
            );
            return Ok(None);
        }

        let (Some(elem_type), Some(elem_value)) = (arr[0].as_str(), arr[1].as_str()) else {
            eprintln!(
                "            ❌  [mod.rs][parse_raw_element()] Invalid element structure: {:?}",
                arr
            );
            return Ok(None);
        };

        eprintln!("            🔍  \x1b[34mELEMENT ANALYSIS\x1b[0m");
        eprintln!(
            "                Type: '{}' | Value: '{}'",
            elem_type, elem_value
        );
        eprintln!("                File: {}:{}", file!(), line!());

        let atom_from = |token_type: &str, token_value: &str| -> RawRuleElement {
            RawRuleElement::Atom(ASTNode::Atom {
                value: ASTValue::Token(vec![
                    TokenValue::String(token_type.to_string()),
                    TokenValue::String(token_value.to_string()),
                ]),
            })
        };

        let parsed = match elem_type {
            "rule_reference" => {
                eprintln!(
                    "                📋  RULE REFERENCE - Creating call to rule '{}'",
                    elem_value
                );
                Some(atom_from("rule_reference", elem_value))
            }
            "quoted_string" => {
                eprintln!(
                    "                💬  \x1b[32mSTRING TERMINAL\x1b[0m - Creating matcher for '{}'",
                    elem_value
                );
                Some(atom_from("quoted_string", elem_value))
            }
            "regex" => {
                eprintln!(
                    "                🔤  \x1b[32mREGEX PATTERN\x1b[0m - Creating regex matcher for '{}'",
                    elem_value
                );
                Some(atom_from("regex", elem_value))
            }
            "group_open" => {
                eprintln!(
                    "                🔓  \x1b[32mGROUP OPEN\x1b[0m - Start grouped expression"
                );
                Some(RawRuleElement::GroupOpen)
            }
            "group_close" => {
                eprintln!(
                    "                🔒  \x1b[32mGROUP CLOSE\x1b[0m - End grouped expression"
                );
                Some(RawRuleElement::GroupClose)
            }
            "quantifier" => {
                eprintln!(
                    "                🔢  \x1b[32mQUANTIFIER\x1b[0m - Binding quantifier '{}'",
                    elem_value
                );
                Some(RawRuleElement::Quantifier(elem_value.to_string()))
            }
            "operator" => match elem_value {
                "|" => {
                    eprintln!(
                        "                🔀  \x1b[32mALTERNATIVE OPERATOR\x1b[0m (|) - Split branches"
                    );
                    Some(RawRuleElement::OrOperator)
                }
                "?" | "*" | "+" => {
                    eprintln!(
                        "                🔁  \x1b[32mQUANTIFIER OPERATOR\x1b[0m '{}' - Bind to previous primary",
                        elem_value
                    );
                    Some(RawRuleElement::Quantifier(elem_value.to_string()))
                }
                _ => {
                    eprintln!(
                        "                ⚙️   \x1b[33mNON-STRUCTURAL OPERATOR\x1b[0m '{}' - treat as terminal",
                        elem_value
                    );
                    Some(atom_from("quoted_string", elem_value))
                }
            },
            "number" => {
                eprintln!(
                    "                🔢  \x1b[32mNUMBER\x1b[0m - treat as terminal '{}'",
                    elem_value
                );
                Some(atom_from("number", elem_value))
            }
            "probability" => {
                eprintln!(
                    "                🎲  \x1b[32mPROBABILITY\x1b[0m - treat as terminal '{}'",
                    elem_value
                );
                Some(atom_from("probability", elem_value))
            }
            "include_dir" => {
                eprintln!(
                    "                📁  \x1b[32mINCLUDE DIR\x1b[0m - preserve '{}' token",
                    elem_value
                );
                Some(atom_from("include_dir", elem_value))
            }
            "include_file" => {
                eprintln!(
                    "                📄  \x1b[32mINCLUDE FILE\x1b[0m - preserve '{}' token",
                    elem_value
                );
                Some(atom_from("include_file", elem_value))
            }
            "rule" => {
                eprintln!(
                    "                📝  \x1b[33mRULE TOKEN\x1b[0m - preserve '{}' token",
                    elem_value
                );
                Some(atom_from("rule", elem_value))
            }
            "return_scalar" | "return_array" | "return_object" => {
                eprintln!(
                    "                🔙  \x1b[33mRETURN ANNOTATION\x1b[0m '{}' - skipped in syntax tree stage",
                    elem_type
                );
                None
            }
            _ => {
                eprintln!(
                    "                ❓  \x1b[33mUNKNOWN ELEMENT TYPE\x1b[0m '{}' - skipping",
                    elem_type
                );
                None
            }
        };

        Ok(parsed)
    }

    fn step2_group_by_or(&self, elements: &[RawRuleElement]) -> Vec<Vec<RawRuleElement>> {
        eprintln!("[mod.rs][step2_group_by_or()] 🔀 Splitting top-level alternatives");
        let mut branches: Vec<Vec<RawRuleElement>> = Vec::new();
        let mut current: Vec<RawRuleElement> = Vec::new();
        let mut group_depth = 0usize;

        for elem in elements {
            match elem {
                RawRuleElement::GroupOpen => {
                    group_depth += 1;
                    current.push(elem.clone());
                }
                RawRuleElement::GroupClose => {
                    if group_depth > 0 {
                        group_depth -= 1;
                    } else {
                        eprintln!(
                            "  ⚠️ [mod.rs][step2_group_by_or()] unmatched group_close at top-level"
                        );
                    }
                    current.push(elem.clone());
                }
                RawRuleElement::OrOperator if group_depth == 0 => {
                    branches.push(current);
                    current = Vec::new();
                }
                _ => current.push(elem.clone()),
            }
        }

        branches.push(current);

        if group_depth != 0 {
            eprintln!(
                "  ⚠️ [mod.rs][step2_group_by_or()] unbalanced parentheses depth={}",
                group_depth
            );
        }

        branches
    }

    fn step2_5_handle_parentheses(&self, branch: &[RawRuleElement]) -> Result<Vec<ASTNode>> {
        eprintln!(
            "[mod.rs][step2_5_handle_parentheses()] 🧩 Parsing grouped branch of {} elements",
            branch.len()
        );
        let mut result = Vec::new();
        let mut idx = 0usize;

        while idx < branch.len() {
            let mut primary = match &branch[idx] {
                RawRuleElement::Atom(node) => {
                    idx += 1;
                    node.clone()
                }
                RawRuleElement::GroupOpen => {
                    let (inner, next_idx) = self.extract_group_contents(branch, idx)?;
                    idx = next_idx;
                    self.build_ast_from_elements(&inner)?
                }
                RawRuleElement::GroupClose => {
                    eprintln!(
                        "  ⚠️ [mod.rs][step2_5_handle_parentheses()] unexpected group_close at idx={}",
                        idx
                    );
                    idx += 1;
                    continue;
                }
                RawRuleElement::OrOperator => {
                    eprintln!(
                        "  ⚠️ [mod.rs][step2_5_handle_parentheses()] unexpected top-level OR token inside branch at idx={}",
                        idx
                    );
                    idx += 1;
                    continue;
                }
                RawRuleElement::Quantifier(q) => {
                    eprintln!(
                        "  ⚠️ [mod.rs][step2_5_handle_parentheses()] dangling quantifier '{}' at idx={} (ignored)",
                        q, idx
                    );
                    idx += 1;
                    continue;
                }
            };

            primary = self.step4_handle_quantifiers(primary, branch, &mut idx);
            result.push(primary);
        }

        Ok(result)
    }

    fn extract_group_contents(
        &self,
        branch: &[RawRuleElement],
        open_idx: usize,
    ) -> Result<(Vec<RawRuleElement>, usize)> {
        let mut depth = 1usize;
        let mut idx = open_idx + 1;
        let mut inner = Vec::new();

        while idx < branch.len() {
            match &branch[idx] {
                RawRuleElement::GroupOpen => {
                    depth += 1;
                    inner.push(branch[idx].clone());
                }
                RawRuleElement::GroupClose => {
                    depth -= 1;
                    if depth == 0 {
                        return Ok((inner, idx + 1));
                    }
                    inner.push(branch[idx].clone());
                }
                _ => inner.push(branch[idx].clone()),
            }
            idx += 1;
        }

        Err(anyhow::anyhow!(
            "[mod.rs][extract_group_contents()] Unclosed group starting at index {}",
            open_idx
        ))
    }

    fn step3_parse_sequences(&self, elements: Vec<ASTNode>) -> ASTNode {
        eprintln!(
            "[mod.rs][step3_parse_sequences()] 🧱 Building sequence from {} elements",
            elements.len()
        );
        match elements.len() {
            0 => ASTNode::Sequence { elements: vec![] },
            1 => elements.into_iter().next().unwrap(),
            _ => ASTNode::Sequence { elements },
        }
    }

    fn step4_handle_quantifiers(
        &self,
        mut node: ASTNode,
        branch: &[RawRuleElement],
        idx: &mut usize,
    ) -> ASTNode {
        while *idx < branch.len() {
            match &branch[*idx] {
                RawRuleElement::Quantifier(q) => {
                    eprintln!(
                        "[mod.rs][step4_handle_quantifiers()] 🔁 Apply quantifier '{}' at idx={}",
                        q, *idx
                    );
                    node = ASTNode::Quantified {
                        element: Box::new(node),
                        quantifier: q.clone(),
                    };
                    *idx += 1;
                }
                _ => break,
            }
        }

        node
    }

    fn step5_build_tree_structure(&self, branches: Vec<ASTNode>) -> ASTNode {
        eprintln!(
            "[mod.rs][step5_build_tree_structure()] 🌳 Final tree from {} branches",
            branches.len()
        );
        if branches.len() <= 1 {
            branches
                .into_iter()
                .next()
                .unwrap_or(ASTNode::Sequence { elements: vec![] })
        } else {
            ASTNode::Or {
                alternatives: branches,
            }
        }
    }

    fn build_ast_from_elements(&self, elements: &[RawRuleElement]) -> Result<ASTNode> {
        let branches = self.step2_group_by_or(elements);
        let mut branch_asts = Vec::with_capacity(branches.len());
        for branch in branches {
            let seq_elements = self.step2_5_handle_parentheses(&branch)?;
            branch_asts.push(self.step3_parse_sequences(seq_elements));
        }
        Ok(self.step5_build_tree_structure(branch_asts))
    }

    fn raw_element_kind(&self, elem: &RawRuleElement) -> &'static str {
        match elem {
            RawRuleElement::Atom(_) => "atom",
            RawRuleElement::OrOperator => "or_operator",
            RawRuleElement::GroupOpen => "group_open",
            RawRuleElement::GroupClose => "group_close",
            RawRuleElement::Quantifier(_) => "quantifier",
        }
    }

    fn parse_single_element(&self, element: &serde_json::Value) -> Result<Option<ASTNode>> {
        if let Some(arr) = element.as_array() {
            if arr.len() >= 2 {
                if let (Some(elem_type), Some(elem_value)) = (arr[0].as_str(), arr[1].as_str()) {
                    eprintln!("            🔍  \x1b[34mELEMENT ANALYSIS\x1b[0m");
                    eprintln!(
                        "                Type: '{}' | Value: '{}'",
                        elem_type, elem_value
                    );
                    eprintln!("                File: {}:{}", file!(), line!());

                    match elem_type {
                        "rule" => {
                            eprintln!(
                                "                📝  \x1b[32mRULE DECLARATION\x1b[0m - Defining rule '{}'",
                                elem_value
                            );
                            eprintln!("                File: {}:{}", file!(), line!());
                            Ok(Some(ASTNode::Atom {
                                value: ASTValue::Token(vec![
                                    TokenValue::String("rule".to_string()),
                                    TokenValue::String(elem_value.to_string()),
                                ]),
                            }))
                        }
                        "rule_reference" => {
                            eprintln!(
                                "                📋  RULE REFERENCE - Creating call to rule '{}'",
                                elem_value
                            );
                            eprintln!("                File: {}:{}", file!(), line!());
                            Ok(Some(ASTNode::Atom {
                                value: ASTValue::Token(vec![
                                    TokenValue::String("rule_reference".to_string()),
                                    TokenValue::String(elem_value.to_string()),
                                ]),
                            }))
                        }
                        "quoted_string" => {
                            eprintln!(
                                "                💬  \x1b[32mSTRING TERMINAL\x1b[0m - Creating matcher for '{}'",
                                elem_value
                            );
                            eprintln!("                File: {}:{}", file!(), line!());
                            Ok(Some(ASTNode::Atom {
                                value: ASTValue::Token(vec![
                                    TokenValue::String("quoted_string".to_string()),
                                    TokenValue::String(elem_value.to_string()),
                                ]),
                            }))
                        }
                        "regex" => {
                            eprintln!(
                                "                🔤  \x1b[32mREGEX PATTERN\x1b[0m - Creating regex matcher for '{}'",
                                elem_value
                            );
                            eprintln!("                File: {}:{}", file!(), line!());
                            Ok(Some(ASTNode::Atom {
                                value: ASTValue::Token(vec![
                                    TokenValue::String("regex".to_string()),
                                    TokenValue::String(elem_value.to_string()),
                                ]),
                            }))
                        }
                        "group_open" => {
                            eprintln!(
                                "                🔓  \x1b[32mGROUP OPEN\x1b[0m - Starting group '{}'",
                                elem_value
                            );
                            eprintln!("                File: {}:{}", file!(), line!());
                            Ok(Some(ASTNode::Atom {
                                value: ASTValue::Token(vec![
                                    TokenValue::String("group_open".to_string()),
                                    TokenValue::String(elem_value.to_string()),
                                ]),
                            }))
                        }
                        "group_close" => {
                            eprintln!(
                                "                🔒  \x1b[32mGROUP CLOSE\x1b[0m - Ending group '{}'",
                                elem_value
                            );
                            eprintln!("                File: {}:{}", file!(), line!());
                            Ok(Some(ASTNode::Atom {
                                value: ASTValue::Token(vec![
                                    TokenValue::String("group_close".to_string()),
                                    TokenValue::String(elem_value.to_string()),
                                ]),
                            }))
                        }
                        "quantifier" => {
                            eprintln!(
                                "                🔢  \x1b[32mEXPLICIT QUANTIFIER\x1b[0m - Processing '{}'",
                                elem_value
                            );
                            eprintln!("                File: {}:{}", file!(), line!());
                            Ok(Some(ASTNode::Quantified {
                                element: Box::new(ASTNode::Sequence { elements: vec![] }), // Placeholder
                                quantifier: elem_value.to_string(),
                            }))
                        }
                        "number" => {
                            eprintln!(
                                "                🔢  \x1b[32mNUMERIC LITERAL\x1b[0m - Processing '{}'",
                                elem_value
                            );
                            eprintln!("                File: {}:{}", file!(), line!());
                            Ok(Some(ASTNode::Atom {
                                value: ASTValue::Token(vec![
                                    TokenValue::String("number".to_string()),
                                    TokenValue::String(elem_value.to_string()),
                                ]),
                            }))
                        }
                        "probability" => {
                            eprintln!(
                                "                🎲  \x1b[32mPROBABILITY\x1b[0m - Processing '{}'",
                                elem_value
                            );
                            eprintln!("                File: {}:{}", file!(), line!());
                            Ok(Some(ASTNode::Atom {
                                value: ASTValue::Token(vec![
                                    TokenValue::String("probability".to_string()),
                                    TokenValue::String(elem_value.to_string()),
                                ]),
                            }))
                        }
                        "include_dir" => {
                            eprintln!(
                                "                📁  \x1b[32mINCLUDE DIRECTORY\x1b[0m - Processing '{}'",
                                elem_value
                            );
                            eprintln!("                File: {}:{}", file!(), line!());
                            Ok(Some(ASTNode::Atom {
                                value: ASTValue::Token(vec![
                                    TokenValue::String("include_dir".to_string()),
                                    TokenValue::String(elem_value.to_string()),
                                ]),
                            }))
                        }
                        "include_file" => {
                            eprintln!(
                                "                📄  \x1b[32mINCLUDE FILE\x1b[0m - Processing '{}'",
                                elem_value
                            );
                            eprintln!("                File: {}:{}", file!(), line!());
                            Ok(Some(ASTNode::Atom {
                                value: ASTValue::Token(vec![
                                    TokenValue::String("include_file".to_string()),
                                    TokenValue::String(elem_value.to_string()),
                                ]),
                            }))
                        }
                        "operator" => {
                            eprintln!(
                                "                🔄  \x1b[33mQUANTIFIER OPERATOR\x1b[0m - Processing '{}'",
                                elem_value
                            );
                            eprintln!("                File: {}:{}", file!(), line!());
                            // Handle quantifiers
                            match elem_value {
                                "?" => {
                                    eprintln!(
                                        "                    ❓  \x1b[32mOPTIONAL QUANTIFIER\x1b[0m (?) - Zero or one occurrence"
                                    );
                                    eprintln!("                    File: {}:{}", file!(), line!());
                                    Ok(Some(ASTNode::Quantified {
                                        element: Box::new(ASTNode::Sequence { elements: vec![] }), // Placeholder
                                        quantifier: "?".to_string(),
                                    }))
                                }
                                "*" => {
                                    eprintln!(
                                        "                    🔁  \x1b[32mZERO-OR-MORE QUANTIFIER\x1b[0m (*) - Zero or more occurrences"
                                    );
                                    eprintln!("                    File: {}:{}", file!(), line!());
                                    Ok(Some(ASTNode::Quantified {
                                        element: Box::new(ASTNode::Sequence { elements: vec![] }), // Placeholder
                                        quantifier: "*".to_string(),
                                    }))
                                }
                                "+" => {
                                    eprintln!(
                                        "                    ➕  \x1b[32mONE-OR-MORE QUANTIFIER\x1b[0m (+) - One or more occurrences"
                                    );
                                    eprintln!("                    File: {}:{}", file!(), line!());
                                    Ok(Some(ASTNode::Quantified {
                                        element: Box::new(ASTNode::Sequence { elements: vec![] }), // Placeholder
                                        quantifier: "+".to_string(),
                                    }))
                                }
                                "|" => {
                                    eprintln!(
                                        "                    🔀  \x1b[32mALTERNATIVE OPERATOR\x1b[0m (|) - Creating choice between alternatives"
                                    );
                                    eprintln!("                    File: {}:{}", file!(), line!());
                                    Ok(Some(ASTNode::Atom {
                                        value: ASTValue::Token(vec![
                                            TokenValue::String("operator".to_string()),
                                            TokenValue::String("|".to_string()),
                                        ]),
                                    }))
                                }
                                _ => {
                                    eprintln!(
                                        "                    ⚠️   \x1b[33mUNKNOWN OPERATOR\x1b[0m '{}' - Skipping",
                                        elem_value
                                    );
                                    eprintln!("                    File: {}:{}", file!(), line!());
                                    Ok(None) // Skip unknown operators
                                }
                            }
                        }
                        "return_scalar" | "return_array" | "return_object" => {
                            eprintln!(
                                "                🔙  \x1b[33mRETURN ANNOTATION\x1b[0m '{}' - Skipping (semantic annotation)",
                                elem_type
                            );
                            eprintln!("                File: {}:{}", file!(), line!());
                            // Skip return annotations for now
                            Ok(None)
                        }
                        _ => {
                            eprintln!(
                                "                ❓  \x1b[33mUNKNOWN ELEMENT TYPE\x1b[0m '{}' - Skipping",
                                elem_type
                            );
                            eprintln!("                File: {}:{}", file!(), line!());
                            Ok(None) // Skip unknown element types
                        }
                    }
                } else {
                    eprintln!("            ❌  \x1b[31mERROR: Invalid element structure\x1b[0m");
                    eprintln!(
                        "                Expected [string, string] but got: [{:?}, {:?}]",
                        arr[0], arr[1]
                    );
                    eprintln!("                File: {}:{}", file!(), line!());
                    Ok(None)
                }
            } else {
                eprintln!("            ❌  \x1b[31mERROR: Element array too short\x1b[0m");
                eprintln!(
                    "                Need at least 2 elements, got {}",
                    arr.len()
                );
                eprintln!("                File: {}:{}", file!(), line!());
                Ok(None)
            }
        } else {
            eprintln!("            ❌  \x1b[31mERROR: Element is not an array\x1b[0m");
            eprintln!(
                "                Type: {} | Value: {:?}",
                std::any::type_name::<serde_json::Value>(),
                element
            );
            eprintln!("                File: {}:{}", file!(), line!());
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn transform_from_raw_ast_preserves_return_and_semantic_annotations() {
        let pipeline = RustASTPipeline::new(PipelineConfig::default());
        let raw_ast_data = vec![json!([
            ["rule", "expr"],
            ["semantic_annotation", ["priority", "[9, 1]"]],
            ["rule_reference", "lhs"],
            ["operator", "|"],
            ["rule_reference", "rhs"],
            ["return_scalar", "$1"]
        ])];

        let (_grammar_tree, _rule_order, annotations) = pipeline
            .transform_from_raw_ast(&raw_ast_data)
            .expect("raw_ast transformation should succeed");
        let annotations = annotations.expect("annotations should be preserved");

        let branch_annotations = annotations
            .branch_return_annotations
            .get("expr")
            .expect("rule return annotations should exist");
        assert_eq!(branch_annotations.len(), 2);
        assert!(branch_annotations[0].is_none());
        let return_annotation = branch_annotations[1]
            .as_ref()
            .expect("second branch should carry return annotation");
        assert_eq!(return_annotation.annotation_type, "return_scalar");
        assert_eq!(return_annotation.annotation_content, "$1");
        assert!(return_annotation.parsed_ast.is_some());

        let semantic_annotations = annotations
            .semantic_annotations
            .get("expr")
            .expect("rule semantic annotations should exist");
        assert_eq!(semantic_annotations.len(), 1);
        match &semantic_annotations[0] {
            SemanticAnnotation::Named { name, ast } => {
                assert_eq!(name, "priority");
                assert!(matches!(
                    ast,
                    UnifiedSemanticAST::Raw { content } if content == "[9, 1]"
                ));
            }
            _ => panic!("semantic annotation should be captured as named directive"),
        }
    }

    #[test]
    fn transform_from_raw_ast_promotes_transform_semantic_payload() {
        let pipeline = RustASTPipeline::new(PipelineConfig::default());
        let raw_ast_data = vec![json!([
            ["rule", "int_rule"],
            [
                "semantic_annotation",
                ["transform", "str::parse::<i64>().unwrap_or(0)"]
            ],
            ["regex", "[-+]?[0-9]+"]
        ])];

        let (_grammar_tree, _rule_order, annotations) = pipeline
            .transform_from_raw_ast(&raw_ast_data)
            .expect("raw_ast transformation should succeed");
        let annotations = annotations.expect("annotations should be preserved");

        let semantic_annotations = annotations
            .semantic_annotations
            .get("int_rule")
            .expect("semantic annotation should be present for int_rule");
        assert_eq!(semantic_annotations.len(), 1);
        match &semantic_annotations[0] {
            SemanticAnnotation::Named { name, ast } => {
                assert_eq!(name, "transform");
                assert!(matches!(
                    ast,
                    UnifiedSemanticAST::TransformExpr { expression }
                        if expression == "str::parse::<i64>().unwrap_or(0)"
                ));
            }
            _ => panic!("transform semantic annotation should be named"),
        }
    }

    #[test]
    fn transform_from_raw_ast_nonbootstrap_legacy_semantic_does_not_use_marker_transform_fallback()
    {
        let pipeline = RustASTPipeline::new(PipelineConfig::default());
        let raw_ast_data = vec![json!([
            ["rule", "legacy_sem_rule"],
            ["semantic_annotation", "str::parse::<i64>().unwrap_or(0)"],
            ["regex", "[-+]?[0-9]+"]
        ])];

        let (_grammar_tree, _rule_order, annotations) = pipeline
            .transform_from_raw_ast(&raw_ast_data)
            .expect("raw_ast transformation should succeed");
        let annotations = annotations.expect("annotations should be preserved");

        let semantic_annotations = annotations
            .semantic_annotations
            .get("legacy_sem_rule")
            .expect("legacy semantic annotation should be present");
        assert_eq!(semantic_annotations.len(), 1);
        match &semantic_annotations[0] {
            SemanticAnnotation::Legacy(UnifiedSemanticAST::Raw { content }) => {
                assert_eq!(content, "str::parse::<i64>().unwrap_or(0)")
            }
            other => panic!(
                "non-bootstrap legacy semantic should stay raw and not transform fallback: {:?}",
                other
            ),
        }
    }

    #[cfg(feature = "generated_parsers")]
    #[test]
    fn transform_from_raw_ast_nonbootstrap_named_semantic_preserves_payload_when_backend_rejects() {
        let pipeline = RustASTPipeline::new(PipelineConfig::default());
        let raw_ast_data = vec![json!([
            ["rule", "bad_sem_rule"],
            ["semantic_annotation", ["priority", "\"unterminated"]],
            ["regex", "[0-9]+"]
        ])];

        let (_grammar_tree, _rule_order, annotations) = pipeline
            .transform_from_raw_ast(&raw_ast_data)
            .expect("invalid named semantic payload should remain preserved when backend rejects");
        let annotations = annotations.expect("annotations should be preserved");
        let semantic_annotations = annotations
            .semantic_annotations
            .get("bad_sem_rule")
            .expect("semantic annotation should be present");
        assert_eq!(semantic_annotations.len(), 1);
        match &semantic_annotations[0] {
            SemanticAnnotation::Named { name, ast } => {
                assert_eq!(name, "priority");
                assert!(matches!(
                    ast,
                    UnifiedSemanticAST::Raw { content } if content == "\"unterminated"
                ));
            }
            other => panic!(
                "expected named semantic annotation with preserved raw payload, got {:?}",
                other
            ),
        }
    }
}

pub mod annotation_validator;
pub mod ast_based_generator;
pub mod ast_code_generator;
pub mod ast_generator_direct;
pub mod ast_return_transform;
pub mod grouped_quantifier_parser;
pub mod mutual_recursion_handler;
pub mod return_annotation_handler;
pub mod semantic_directive_registry;
pub mod semantic_transform;
pub mod stimuli_generator;
pub mod unified_return_ast;
pub mod unified_semantic_ast;

// Re-export key types
pub use annotation_validator::{
    AnnotationDiagnostic, AnnotationKind, AnnotationSeverity, AnnotationValidationReport,
    AnnotationValidator, AnnotationValidatorConfig,
};
pub use semantic_directive_registry::{
    SemanticAssociativity, SemanticBranchPolicy, SemanticDeterministicGroupHint,
    SemanticDirectiveCapability, SemanticDirectiveSpec, SemanticTokenClass,
    SemanticValueConstraints, UnknownSemanticDirectivePolicy, extract_semantic_directive,
    extract_semantic_directive_name, normalize_semantic_scalar, parse_semantic_bool,
    parse_semantic_branch_priorities, parse_semantic_charset, parse_semantic_constraint_expression,
    parse_semantic_coverage_target_weight, parse_semantic_deterministic_group,
    parse_semantic_float_list, parse_semantic_group_label, parse_semantic_implication,
    parse_semantic_len_bounds, parse_semantic_nonnegative_usize, parse_semantic_numeric_bounds,
    parse_semantic_numeric_list, parse_semantic_pattern, parse_semantic_reference_list,
    parse_semantic_string_list, parse_semantic_token_class, semantic_directive_spec,
};
pub use semantic_transform::{
    CanonicalSemanticTransform, parse_canonical_transform_expression, stimuli_hint_for_target_type,
};
pub use unified_return_ast::{ExtractionTarget, UnifiedReturnAST};
pub use unified_semantic_ast::UnifiedSemanticAST;
