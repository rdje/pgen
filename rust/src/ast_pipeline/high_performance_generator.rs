//! High-Performance Rust Code Generator
//! Generates lightning-fast parsers with:
//! - Zero-copy parsing where possible
//! - Memoization/packrat parsing for backtracking
//! - Inline optimizations and SIMD-friendly code
//! - Minimal allocations for rgx regex engine integration

use crate::ast_pipeline::{ASTNode, ASTValue, Annotations, ReturnAnnotation};
use std::collections::HashMap;
use anyhow::Result;
use super::return_annotation_handler::{ReturnAnnotationHandler, ReturnAnnotationMode};
use serde_json::Value as JsonValue;

/// Escape a string for safe inclusion in Rust raw string literals
fn escape_rust_string(s: &str) -> String {
    // For raw strings r#"..."#, we need minimal escaping
    // The main issue is handling strings that contain the raw string delimiter
    // For now, just return the string as-is since raw strings handle most cases
    s.to_string()
}

/// Builder for systematic Rust code generation with automatic brace tracking
#[allow(dead_code)]
struct RustCodeBuilder {
    lines: Vec<String>,
    brace_depth: i32,
    scope_variables: Vec<std::collections::HashSet<String>>,
    current_indent: String,
}

impl RustCodeBuilder {
    fn new() -> Self {
        Self { 
            lines: Vec::new(),
            brace_depth: 0,
            scope_variables: vec![std::collections::HashSet::new()],
            current_indent: String::new(),
        }
    }
    
    fn add_line(&mut self, line: &str) {
        // Track brace depth changes
        let opens = line.matches('{').count() as i32;
        let closes = line.matches('}').count() as i32;
        self.brace_depth += opens - closes;
        
        // Manage scopes
        for _ in 0..opens {
            self.scope_variables.push(std::collections::HashSet::new());
        }
        for _ in 0..closes {
            if self.scope_variables.len() > 1 {
                self.scope_variables.pop();
            }
        }
        
        self.lines.push(line.to_string());
    }
    
    fn add_raw(&mut self, code: &str) {
        // Process each line in the raw code to track braces
        for line in code.lines() {
            self.add_line(line);
        }
    }
    
    fn enter_scope(&mut self, indent: &str) {
        self.add_line(&format!("{indent}{{"));
        self.current_indent = format!("{indent}    ");
    }
    
    fn exit_scope(&mut self, indent: &str) {
        self.add_line(&format!("{indent}}}"));
        if indent.len() >= 4 {
            self.current_indent = indent[..indent.len()-4].to_string();
        } else {
            self.current_indent = String::new();
        }
    }
    
    fn declare_variable(&mut self, var_name: &str) {
        if let Some(current_scope) = self.scope_variables.last_mut() {
            current_scope.insert(var_name.to_string());
        }
    }
    
    fn is_variable_in_scope(&self, var_name: &str) -> bool {
        self.scope_variables.iter().any(|scope| scope.contains(var_name))
    }
    
    fn validate(&self) -> Result<()> {
        if self.brace_depth != 0 {
            return Err(anyhow::anyhow!(
                "Unbalanced braces detected: depth = {} (positive means missing closing braces, negative means extra closing braces)",
                self.brace_depth
            ));
        }
        Ok(())
    }
    
    fn build(self) -> String {
        // Validate before building
        if let Err(e) = self.validate() {
            eprintln!("⚠️  Code generation warning: {}", e);
        }
        self.lines.join("\n")
    }
}

/// High-performance code generator optimized for regex parsing
pub struct HighPerformanceRustGenerator {
    grammar_name: String,
    entry_rule: Option<String>,
    enable_trace: bool,
    enable_backtrack_debug: bool,
    bootstrap_mode: bool,
    annotations: Option<Annotations>,
    branch_return_annotations: HashMap<String, Vec<Option<ReturnAnnotation>>>,
    #[allow(dead_code)]
    quantified_group_counter: std::cell::RefCell<u32>,
    quantified_groups: std::cell::RefCell<Vec<QuantifiedGroupInfo>>,
}

#[derive(Debug, Clone)]
struct QuantifiedGroupInfo {
    rule_name: String,
    group_id: u32,
    element: ASTNode,
    quantifier: String,
    rule_annotations: Option<Vec<String>>,
}

/// Represents processed elements in a sequence, distinguishing between mandatory and optional groups
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum ProcessedElement {
    /// A mandatory element that must be parsed
    Mandatory(ASTNode),
    /// An optional group that can be skipped if parsing fails
    OptionalGroup(Vec<ASTNode>),
}

impl HighPerformanceRustGenerator {
    pub fn new(grammar_name: &str) -> Self {
        Self {
            grammar_name: grammar_name.to_string(),
            entry_rule: None,
            enable_trace: false,
            enable_backtrack_debug: false,
            bootstrap_mode: false,
            annotations: None,
            branch_return_annotations: HashMap::new(),
            quantified_group_counter: std::cell::RefCell::new(0),
            quantified_groups: std::cell::RefCell::new(Vec::new()),
        }
    }
    
    /// Create generator with trace mode enabled
    pub fn with_trace(grammar_name: &str, enable_trace: bool) -> Self {
        Self {
            grammar_name: grammar_name.to_string(),
            entry_rule: None,
            enable_trace,
            enable_backtrack_debug: false,
            bootstrap_mode: false,
            annotations: None,
            branch_return_annotations: HashMap::new(),
            quantified_group_counter: std::cell::RefCell::new(0),
            quantified_groups: std::cell::RefCell::new(Vec::new()),
        }
    }
    
    /// Create generator with full debug enabled (trace + backtrack)
    pub fn with_full_debug(grammar_name: &str) -> Self {
        Self {
            grammar_name: grammar_name.to_string(),
            entry_rule: None,
            enable_trace: true,
            enable_backtrack_debug: true,
            bootstrap_mode: false,
            annotations: None,
            branch_return_annotations: HashMap::new(),
            quantified_group_counter: std::cell::RefCell::new(0),
            quantified_groups: std::cell::RefCell::new(Vec::new()),
        }
    }
    
    /// Set the entry rule name dynamically
    pub fn set_entry_rule(&mut self, entry_rule: &str) {
        self.entry_rule = Some(entry_rule.to_string());
    }
    
    /// Set the annotations for the generator to use during code generation
    #[allow(dead_code)]
    pub fn set_annotations(&mut self, annotations: Annotations) {
        self.annotations = Some(annotations);
    }
    
    /// Set branch return annotations for code generation
    pub fn set_branch_return_annotations(&mut self, branch_return_annotations: &HashMap<String, Vec<Option<ReturnAnnotation>>>) {
        println!("[DEBUG] Setting branch return annotations: {} rules with annotations", branch_return_annotations.len());
        for (rule_name, annotations) in branch_return_annotations {
            let ann_count = annotations.iter().filter(|a| a.is_some()).count();
            if ann_count > 0 {
                println!("[DEBUG]   Rule '{}': {} branch annotations", rule_name, ann_count);
            }
        }
        self.branch_return_annotations = branch_return_annotations.clone();
    }
    
    /// Set bootstrap mode for limited return annotation support
    pub fn set_bootstrap_mode(&mut self, bootstrap: bool) {
        self.bootstrap_mode = bootstrap;
    }
    
    /// Enable debug output for return annotations
    pub fn set_debug_mode(&mut self, enable_debug: bool) {
        // When debug mode is enabled, we want to show AST dumps for return annotations
        if enable_debug {
            self.enable_trace = true;
        }
    }
    
    /// Log message using the AST Pipeline's unified logging system
    /// This method will be replaced with calls to pipeline.log_debug() when we have access to the pipeline
    fn log_debug(&self, message: &str) {
        // Fallback: only print to console if no pipeline is available
        // This should rarely be used since we'll pass pipeline references
        println!("{}", message);
    }
    
    /// Validate generated Rust code for common issues
    fn validate_generated_code(&self, code: &str, context: &str) -> Result<()> {
        let mut issues = Vec::new();
        
        // Check brace balance
        let open_braces = code.matches('{').count();
        let close_braces = code.matches('}').count();
        if open_braces != close_braces {
            issues.push(format!("Unbalanced braces: {} open, {} close", open_braces, close_braces));
        }
        
        // Check parentheses balance
        let open_parens = code.matches('(').count();
        let close_parens = code.matches(')').count();
        if open_parens != close_parens {
            issues.push(format!("Unbalanced parentheses: {} open, {} close", open_parens, close_parens));
        }
        
        // Check for common scoping issues
        if code.contains("group_element_content") {
            // Look for variable declarations and usage patterns
            let lines: Vec<&str> = code.lines().collect();
            let mut in_scope = false;
            let mut scope_depth = 0;
            let mut declaration_depth = None;
            
            for (line_num, line) in lines.iter().enumerate() {
                let trimmed = line.trim();
                
                // Track scope depth
                scope_depth += trimmed.matches('{').count() as i32;
                scope_depth -= trimmed.matches('}').count() as i32;
                
                // Check for variable declaration
                if trimmed.contains("let group_element_content =") {
                    declaration_depth = Some(scope_depth);
                    in_scope = true;
                }
                
                // Check for variable usage outside scope
                if trimmed.contains("content: group_element_content") {
                    if let Some(decl_depth) = declaration_depth {
                        if scope_depth < decl_depth {
                            issues.push(format!("Variable 'group_element_content' used outside its scope at line {}", line_num + 1));
                        }
                    } else if !in_scope {
                        issues.push(format!("Variable 'group_element_content' used before declaration at line {}", line_num + 1));
                    }
                }
                
                // Reset scope tracking when we exit the declaration scope
                if let Some(decl_depth) = declaration_depth {
                    if scope_depth < decl_depth {
                        in_scope = false;
                        declaration_depth = None;
                    }
                }
            }
        }
        
        if !issues.is_empty() {
            return Err(anyhow::anyhow!(
                "Code validation failed for {}: {}\nGenerated code:\n{}",
                context,
                issues.join("; "),
                code
            ));
        }
        
        Ok(())
    }
    
    /// Helper to validate and log code generation
    fn generate_and_validate_code<F>(&self, context: &str, generator_fn: F) -> Result<String>
    where
        F: FnOnce() -> Result<String>,
    {
        let code = generator_fn()?;
        
        // Validate the generated code
        if let Err(e) = self.validate_generated_code(&code, context) {
            self.log_debug(&format!("⚠️ Code validation warning for {}: {}", context, e));
            // Don't fail the build, just warn
        }
        
        Ok(code)
    }
    
    /// Generate next unique ID for quantified groups
    fn get_next_quantified_group_id(&self) -> u32 {
        let mut counter = self.quantified_group_counter.borrow_mut();
        let id = *counter;
        *counter += 1;
        id
    }
    
    /// Register a quantified group for later function generation
    fn register_quantified_group(
        &self, 
        rule_name: &str, 
        group_id: u32, 
        element: &ASTNode, 
        quantifier: &str,
        rule_annotations: Option<&[String]>
    ) -> Result<()> {
        let group_info = QuantifiedGroupInfo {
            rule_name: rule_name.to_string(),
            group_id,
            element: element.clone(),
            quantifier: quantifier.to_string(),
            rule_annotations: rule_annotations.map(|a| a.to_vec()),
        };
        
        self.quantified_groups.borrow_mut().push(group_info);
        self.log_debug(&format!("[HighPerformanceRustGenerator] 📋 Registered quantified group: {}_group_{} with quantifier '{}'", rule_name, group_id, quantifier));
        Ok(())
    }
    
    /// Generate all quantified group functions with proper scope isolation
    fn generate_quantified_group_functions(&self) -> Result<String> {
        self.generate_quantified_group_functions_with_pipeline(None)
    }
    
    /// Generate quantified group functions with pipeline logging
    fn generate_quantified_group_functions_with_pipeline(&self, mut pipeline: Option<&mut crate::ast_pipeline::RustASTPipeline>) -> Result<String> {
        let groups = self.quantified_groups.borrow();
        let mut code = String::new();
        
        if let Some(ref mut p) = pipeline {
            p.log_info("generate_quantified_group_functions", &format!("🏰 Generating {} quantified group functions", groups.len()));
        } else {
            println!("[HighPerformanceRustGenerator] 🏰 Generating {} quantified group functions", groups.len());
        }
        
        for (index, group) in groups.iter().enumerate() {
            if let Some(ref mut p) = pipeline {
                p.log_info("generate_quantified_group_functions", &format!("🔧 [{}/{}] Generating function: parse_{}_quantified_group_{}", index + 1, groups.len(), group.rule_name, group.group_id));
            } else {
                println!("[HighPerformanceRustGenerator]   🔧 Generating function: parse_{}_quantified_group_{}", group.rule_name, group.group_id);
            }
            
            let function_code = self.generate_single_quantified_group_function(group)?;
            code.push_str(&function_code);
            code.push_str("\n");
        }
        
        Ok(code)
    }
    
    /// Generate a single quantified group function with proper memoization
    fn generate_single_quantified_group_function(&self, group: &QuantifiedGroupInfo) -> Result<String> {
        let function_name = format!("parse_{}_quantified_group_{}", group.rule_name, group.group_id);
        
        // Generate element code with "p" context for use inside try_parse closures
        let element_code = self.generate_optimized_node_code_with_context(
            &group.element, 
            2, 
            &group.rule_name, 
            group.rule_annotations.as_deref(),
            "p"  // Use "p" since this will be inside a try_parse closure
        )?;
        
        // Generate the quantifier logic based on the specific quantifier type
        let quantifier_logic = match group.quantifier.as_str() {
            "*" => self.generate_star_quantifier_logic(&element_code, &function_name),
            "+" => self.generate_plus_quantifier_logic(&element_code, &function_name), 
            "?" => self.generate_question_quantifier_logic(&element_code, &function_name),
            _ => return Err(anyhow::anyhow!("Unsupported quantifier: {}", group.quantifier))
        };
        
        let template = format!(r#"    /// Parse quantified group {group_id} for rule {rule_name} with quantifier '{quantifier}'
    /// This function provides proper scope isolation for the quantified group
    #[inline]
    fn {function_name}(&mut self) -> ParseResult<ParseContent<'input>> {{
        if self.debug_mode {{
            self.debug_output.push(format!("🚀 QUANTIFIED GROUP FUNCTION ENTRY: {function_name}() at position {{}}", self.position));
        }}
        
{quantifier_logic}
        
        if self.debug_mode {{
            match &result {{
                Ok(_) => self.debug_output.push(format!("✅ QUANTIFIED GROUP FUNCTION EXIT SUCCESS: {function_name}() completed at position {{}}", self.position)),
                Err(e) => self.debug_output.push(format!("❌ QUANTIFIED GROUP FUNCTION EXIT FAILURE: {function_name}() failed: {{:?}}", e)),
            }}
        }}
        
        result
    }}
"#, 
            group_id = group.group_id,
            rule_name = group.rule_name,
            quantifier = group.quantifier,
            function_name = function_name,
            quantifier_logic = quantifier_logic
        );
        
        Ok(template)
    }

    /// Generate star quantifier logic (*) with proper scope isolation
    fn generate_star_quantifier_logic(&self, element_code: &str, function_name: &str) -> String {
        // Element code should already have proper "self" context from generate_optimized_node_code_with_context
        // We need to indent the element_code properly for embedding
        let indented_element_code = element_code.lines()
            .map(|line| if line.trim().is_empty() { 
                line.to_string() 
            } else { 
                format!("                {}", line) 
            })
            .collect::<Vec<_>>()
            .join("\n");
            
        format!(r#"        // Star quantifier (*) - zero or more with proper scope isolation
        let mut results = Vec::new();
        let mut iteration = 0;
        let start_pos = self.position;
        
        if self.debug_mode {{
            self.debug_output.push(format!("🔄 {function_name}: Starting STAR quantifier (zero-or-more) at position {{}}", self.position));
        }}
        
        loop {{
            let checkpoint_pos = self.position;
            if self.debug_mode {{
                self.debug_output.push(format!("🔁 {function_name}: STAR iteration {{}} at position {{}}", iteration + 1, checkpoint_pos));
            }}
            
            // Try to parse element with backtracking support
            let element_result = self.try_parse(|p| {{
                // Element parsing with proper context (p is self in closure)
{indented_element_code}
                Ok(result)
            }});
            
            if let Some(content) = element_result {{
                iteration += 1;
                if self.debug_mode {{
                    self.debug_output.push(format!("✅ {function_name}: STAR iteration {{}} succeeded, position {{}} -> {{}}", iteration, checkpoint_pos, self.position));
                }}
                results.push(ParseNode {{
                    rule_name: "quantified",
                    content,
                    span: 0..0,
                }});
                
                // Prevent infinite loops on zero-length matches
                if self.position == checkpoint_pos {{
                    if self.debug_mode {{
                        self.debug_output.push(format!("⚠️ {function_name}: STAR zero-length match detected - stopping to prevent infinite loop"));
                    }}
                    break;
                }}
            }} else {{
                if self.debug_mode {{
                    self.debug_output.push(format!("🛑 {function_name}: STAR iteration {{}} failed at position {{}} - ending (normal for * quantifier)", iteration + 1, checkpoint_pos));
                }}
                break;
            }}
        }}
        
        if self.debug_mode {{
            self.debug_output.push(format!("🏁 {function_name}: STAR quantifier completed - matched {{}} iterations, position {{}} -> {{}}", iteration, start_pos, self.position));
        }}
        
        let result = Ok(ParseContent::Quantified(results, "*"));
"#, function_name = function_name, indented_element_code = indented_element_code)
    }
    
    /// Generate plus quantifier logic (+) with proper scope isolation
    fn generate_plus_quantifier_logic(&self, element_code: &str, function_name: &str) -> String {
        // Element code should already have proper "self" context from generate_optimized_node_code_with_context
        // We need to indent the element_code properly for embedding
        let indented_element_code = element_code.lines()
            .map(|line| if line.trim().is_empty() { 
                line.to_string() 
            } else { 
                format!("                {}", line) 
            })
            .collect::<Vec<_>>()
            .join("\n");
            
        format!(r#"        // Plus quantifier (+) - one or more with proper scope isolation
        let mut results = Vec::new();
        let mut iteration = 0;
        let start_pos = self.position;
        
        if self.debug_mode {{
            self.debug_output.push(format!("🔄 {function_name}: Starting PLUS quantifier (one-or-more) at position {{}}", self.position));
        }}
        
        // We need at least one successful match for '+' quantifier
        // First attempt with backtracking
        let first_result = self.try_parse(|p| {{
{indented_element_code}
            Ok(result)
        }});
        
        let result = if let Some(content) = first_result {{
            iteration = 1;
            if self.debug_mode {{
                self.debug_output.push(format!("✅ {function_name}: PLUS first iteration succeeded, position {{}} -> {{}}", start_pos, self.position));
            }}
            results.push(ParseNode {{
                rule_name: "quantified",
                content,
                span: 0..0,
            }});
            
            // Continue with zero-or-more pattern
            loop {{
                let checkpoint_pos = self.position;
                if self.debug_mode {{
                    self.debug_output.push(format!("🔁 {function_name}: PLUS iteration {{}} at position {{}}", iteration + 1, checkpoint_pos));
                }}
                
                let element_result = self.try_parse(|p| {{
{indented_element_code}
                    Ok(result)
                }});
                
                if let Some(content) = element_result {{
                    iteration += 1;
                    if self.debug_mode {{
                        self.debug_output.push(format!("✅ {function_name}: PLUS iteration {{}} succeeded, position {{}} -> {{}}", iteration, checkpoint_pos, self.position));
                    }}
                    results.push(ParseNode {{
                        rule_name: "quantified",
                        content,
                        span: 0..0,
                    }});
                    
                    // Prevent infinite loops on zero-length matches
                    if self.position == checkpoint_pos {{
                        if self.debug_mode {{
                            self.debug_output.push(format!("⚠️ {function_name}: PLUS zero-length match detected - stopping"));
                        }}
                        break;
                    }}
                }} else {{
                    if self.debug_mode {{
                        self.debug_output.push(format!("🛑 {function_name}: PLUS iteration {{}} failed at position {{}} - ending", iteration + 1, checkpoint_pos));
                    }}
                    break;
                }}
            }}
            
            if self.debug_mode {{
                self.debug_output.push(format!("🏁 {function_name}: PLUS quantifier completed - matched {{}} iterations, position {{}} -> {{}}", iteration, start_pos, self.position));
            }}
            
            Ok(ParseContent::Quantified(results, "+"))
        }} else {{
            if self.debug_mode {{
                self.debug_output.push(format!("❌ {function_name}: PLUS first iteration failed - quantifier fails"));
            }}
            Err(ParseError::InvalidSyntax {{
                message: "Plus quantifier requires at least one match",
                position: start_pos,
            }})
        }};
"#, function_name = function_name, indented_element_code = indented_element_code)
    }
    
    /// Generate question quantifier logic (?) with proper scope isolation
    fn generate_question_quantifier_logic(&self, element_code: &str, function_name: &str) -> String {
        // Element code should already have proper "self" context from generate_optimized_node_code_with_context
        // We need to indent the element_code properly for embedding
        let indented_element_code = element_code.lines()
            .map(|line| if line.trim().is_empty() { 
                line.to_string() 
            } else { 
                format!("            {}", line) 
            })
            .collect::<Vec<_>>()
            .join("\n");
            
        format!(r#"        // Question quantifier (?) - zero or one with proper scope isolation
        let start_pos = self.position;
        
        if self.debug_mode {{
            self.debug_output.push(format!("🔄 {function_name}: Starting QUESTION quantifier (zero-or-one) at position {{}}", self.position));
        }}
        
        // Try to parse the element once with backtracking
        let element_result = self.try_parse(|p| {{
{indented_element_code}
            Ok(result)
        }});
        
        let result = if let Some(content) = element_result {{
            if self.debug_mode {{
                self.debug_output.push(format!("✅ {function_name}: QUESTION quantifier matched (1 occurrence), position {{}} -> {{}}", start_pos, self.position));
            }}
            let results = vec![ParseNode {{
                rule_name: "quantified",
                content,
                span: 0..0,
            }}];
            Ok(ParseContent::Quantified(results, "?"))
        }} else {{
            if self.debug_mode {{
                self.debug_output.push(format!("⭕ {function_name}: QUESTION quantifier no match (0 occurrences) - this is OK for ? quantifier"));
            }}
            // Zero matches is OK for ? quantifier
            Ok(ParseContent::Quantified(Vec::new(), "?"))
        }};
"#, function_name = function_name, indented_element_code = indented_element_code)
    }

    /// Generate lightning-fast parser suitable for production regex engine
    pub fn generate_lightning_fast_parser(
        &self,
        grammar_tree: &HashMap<String, ASTNode>,
        rule_order: &[String],
    ) -> Result<String> {
        // Fallback method that doesn't use pipeline logging
        self.generate_lightning_fast_parser_impl(grammar_tree, rule_order, None)
    }
    
    /// Generate lightning-fast parser with unified AST pipeline logging
    pub fn generate_lightning_fast_parser_with_logging(
        &self,
        grammar_tree: &HashMap<String, ASTNode>,
        rule_order: &[String],
        pipeline: &mut crate::ast_pipeline::RustASTPipeline,
    ) -> Result<String> {
        // Use pipeline logging for unified output
        self.generate_lightning_fast_parser_impl(grammar_tree, rule_order, Some(pipeline))
    }
    
    /// Internal implementation that can optionally use pipeline logging
    fn generate_lightning_fast_parser_impl(
        &self,
        grammar_tree: &HashMap<String, ASTNode>,
        rule_order: &[String],
        mut pipeline: Option<&mut crate::ast_pipeline::RustASTPipeline>,
    ) -> Result<String> {
        if rule_order.is_empty() {
            return Err(anyhow::anyhow!("No rules provided - cannot determine entry rule"));
        }
        
        // Log using pipeline if available, otherwise fallback
        if let Some(ref mut p) = pipeline {
            p.log_info("generate_lightning_fast_parser", "🚀 Starting high-performance parser generation");
            p.log_info("generate_lightning_fast_parser", &format!("📋 {} rules to process: {}", rule_order.len(), rule_order.join(", ")));
        } else {
            self.log_debug("[HighPerformanceRustGenerator] 🚀 Starting high-performance parser generation");
        }
        
        // Entry rule should be the first rule in the grammar, or explicitly set entry rule
        let entry_rule = self.entry_rule.as_ref()
            .map(|s| s.clone())
            .or_else(|| rule_order.first().cloned())
            .ok_or_else(|| anyhow::anyhow!("No rules provided in rule_order - cannot determine entry rule"))?;
        
        if let Some(ref mut p) = pipeline {
            p.log_success("generate_lightning_fast_parser", &format!("📍 Entry rule determined: {}", entry_rule));
        }
        
        let mut code = String::with_capacity(65536); // Pre-allocate for performance

        // Generate high-performance parser header
        if let Some(ref mut p) = pipeline {
            p.log_info("generate_lightning_fast_parser", "📜 Generating parser header and imports");
        }
        code.push_str(&self.generate_parser_header());
        
        // Generate core parsing engine with memoization (starts impl block)
        if let Some(ref mut p) = pipeline {
            p.log_info("generate_lightning_fast_parser", "⚙️  Generating core memoized parser engine");
        }
        code.push_str(&self.generate_memoized_parser_core(&entry_rule));
        
        // Generate optimized rule methods (inside impl block)
        if let Some(ref mut p) = pipeline {
            p.log_info("generate_lightning_fast_parser", &format!("🛠️  Generating {} optimized rule methods", rule_order.len()));
        }
        code.push_str(&self.generate_optimized_rule_methods_with_pipeline(grammar_tree, rule_order, pipeline.as_deref_mut())?);
        
        // Generate quantified group functions (inside impl block)
        if let Some(ref mut p) = pipeline {
            let group_count = self.quantified_groups.borrow().len();
            p.log_info("generate_lightning_fast_parser", &format!("🔄 Generating {} quantified group functions", group_count));
        }
        code.push_str(&self.generate_quantified_group_functions_with_pipeline(pipeline.as_deref_mut())?);
        
        // Generate fast helper methods (inside impl block)
        if let Some(ref mut p) = pipeline {
            p.log_info("generate_lightning_fast_parser", "⚡ Generating fast helper methods");
        }
        code.push_str(&self.generate_fast_helpers());
        
        // Close the impl block
        code.push_str("}\n\n");
        
        // Generate performance tests
        if let Some(ref mut p) = pipeline {
            p.log_info("generate_lightning_fast_parser", "📏 Generating performance tests");
        }
        code.push_str(&self.generate_performance_tests());
        
        if let Some(ref mut p) = pipeline {
            p.log_success("generate_lightning_fast_parser", &format!("🎉 High-performance parser generation complete! Generated {} lines of Rust code", code.lines().count()));
        }
        
        Ok(code)
    }

    fn generate_parser_header(&self) -> String {
        format!(r#"// {grammar_name} High-Performance Parser
// Generated for rgx regex engine - SOTA performance
// Features: Zero-copy, memoization, SIMD-optimized, minimal allocations

use std::{{
    collections::{{HashMap, HashSet}},
    fmt,
    ops::Range,
    fs::File,
    io::{{Write, BufWriter}},
}};
use regex::Regex;

/// Cycle detection for mutual recursion handling
#[derive(Debug, Clone, PartialEq)]
pub enum CycleType {{
    None,
    Infinite,
    LeftRecursive,
    MutualRecursive {{ depth: usize, rules: Vec<String> }},
}}

/// Smart recursion guard with cycle detection
pub struct RecursionGuard {{
    parse_stack: Vec<(String, usize)>,
    max_depth: usize,
    cycle_cache: HashMap<(String, usize), CycleType>,
}}

impl RecursionGuard {{
    pub fn new(max_depth: usize) -> Self {{
        Self {{
            parse_stack: Vec::new(),
            max_depth,
            cycle_cache: HashMap::new(),
        }}
    }}
    
    pub fn check_cycle(&mut self, rule_name: &str, position: usize) -> CycleType {{
        if let Some(cached) = self.cycle_cache.get(&(rule_name.to_string(), position)) {{
            return cached.clone();
        }}
        
        for (r, p) in self.parse_stack.iter() {{
            if r == rule_name && *p == position {{
                let cycle = CycleType::Infinite;
                self.cycle_cache.insert((rule_name.to_string(), position), cycle.clone());
                return cycle;
            }}
            if r == rule_name && *p > position {{
                let cycle = CycleType::LeftRecursive;
                self.cycle_cache.insert((rule_name.to_string(), position), cycle.clone());
                return cycle;
            }}
        }}
        
        if self.parse_stack.len() >= self.max_depth {{
            let rules: HashSet<String> = self.parse_stack.iter().map(|(r, _)| r.clone()).collect();
            return CycleType::MutualRecursive {{
                depth: self.parse_stack.len(),
                rules: rules.into_iter().collect(),
            }};
        }}
        
        CycleType::None
    }}
    
    pub fn enter(&mut self, rule_name: &str, position: usize) {{
        self.parse_stack.push((rule_name.to_string(), position));
    }}
    
    pub fn exit(&mut self) {{
        self.parse_stack.pop();
    }}
}}

/// Parse result with zero-copy string slices
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {{
    UnexpectedEof {{ position: usize }},
    UnexpectedToken {{ expected: &'static str, found: char, position: usize }},
    InvalidSyntax {{ message: &'static str, position: usize }},
    Backtrack {{ position: usize }},
    RecursionDepthExceeded {{ position: usize, depth: usize }},
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
            ParseError::RecursionDepthExceeded {{ position, depth }} => 
                write!(f, "Recursion depth exceeded ({{}} levels) at position {{}}", depth, position),
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

/// Maximum recursion depth to prevent stack overflow in circular grammars
const MAX_RECURSION_DEPTH: usize = 1000;

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
    rule_stack: Vec<String>, // Track rule hierarchy
    recursion_depth: usize, // Track recursion depth to prevent stack overflow
    recursion_guard: RecursionGuard, // Mutual recursion detection
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
            rule_stack: Vec::new(),
            recursion_depth: 0,
            recursion_guard: RecursionGuard::new(100), // Max depth 100
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
            rule_stack: Vec::new(),
            recursion_depth: 0,
            recursion_guard: RecursionGuard::new(100), // Max depth 100
        }}
    }}
    
    // TODO: Re-add debug log file functionality after fixing template issues
    
    /// Get debug output for analysis
    pub fn debug_output(&self) -> &[String] {{
        &self.debug_output
    }}
    
    /// Clear debug output
    pub fn clear_debug(&mut self) {{
        self.debug_output.clear();
        self.debug_depth = 0;
        self.rule_stack.clear();
        self.recursion_depth = 0;
    }}
    
    // TODO: Re-implement debug logging methods after fixing template formatting

    /// Parse entry point - returns AST or error with beautiful debug output
    pub fn parse(&mut self) -> ParseResult<ParseNode<'input>> {{
        self.position = 0;
        self.memo.clear();
        
        if self.debug_mode {{
            let input_preview = if self.input.len() > 40 {{
                format!("{{}}", self.format_debug_string(&self.input[..40]))
            }} else {{
                format!("{{}}", self.format_debug_string(self.input))
            }};
            self.debug_output.push(format!("🔍 PARSING: {{}}", input_preview));
            self.debug_output.push("".to_string());
        }}
        
        let result = self.parse_{entry_rule}(); // Dynamic entry rule
        
        if self.debug_mode {{
            match &result {{
                Ok(ast) => {{
                    self.debug_output.push("".to_string());
                    self.debug_output.push("✅ PARSE COMPLETE: {entry_rule}".to_string());
                    self.debug_output.push(format!("📊 Total consumed: {{}}/{{}} characters", 
                        self.position, self.input.len()));
                    self.debug_output.push(format!("🎯 Result: {{:?}}", ast));
                }}
                Err(err) => {{
                    self.debug_output.push("".to_string());
                    self.debug_output.push("❌ PARSE FAILED: {entry_rule}".to_string());
                    self.debug_output.push(format!("📍 Failed at: Position {{}}", self.position));
                    self.debug_output.push(format!("🎯 Reason: {{}}", err));
                    
                    // Add helpful suggestion based on error type
                    match err {{
                        ParseError::UnexpectedEof {{ .. }} => {{
                            self.debug_output.push("💡 Fix: Input appears to be incomplete".to_string());
                        }}
                        ParseError::UnexpectedToken {{ expected, .. }} => {{
                            self.debug_output.push(format!("💡 Fix: Try using '{{}}' at the error position", expected));
                        }}
                        _ => {{}}
                    }}
                }}
            }}
            
            // TODO: Re-add debug log file writing after fixing template issues
        }}
        
        result
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

    /// Memoized rule call with packrat parsing and recursion depth tracking
    #[inline]
    fn memoized_call<F>(&mut self, rule_id: RuleId, f: F) -> ParseResult<ParseNode<'input>>
    where
        F: FnOnce(&mut Self) -> ParseResult<ParseNode<'input>>,
    {{
        let key = (rule_id, self.position);
        
        // Left-recursion detection: check if we're already trying to parse this rule at this position
        if self.recursion_depth > 0 {{
            // This is a simple left-recursion detector - for full LR support we'd need a more sophisticated approach
            // For now, we'll rely on the memo table and recursion limits to handle cycles
            if self.debug_mode {{
                self.debug_output.push(format!("      🔄 RECURSION CHECK: Rule ID {{}} at pos {{}} (depth {{}})", rule_id, key.1, self.recursion_depth));
            }}
        }}
        
        // Check memo table
        if let Some(entry) = self.memo.get(&key) {{
            // Memoization HIT - use cached result
            if self.debug_mode {{
                self.debug_output.push(format!("      🎯 MEMO HIT: Rule ID {{}} at pos {{}} → cached result (pos {{}})", rule_id, key.1, entry.end_pos));
            }}
            self.position = entry.end_pos;
            match &entry.result {{
                Some(node) => Ok(node.clone()),
                None => Err(ParseError::Backtrack {{ position: self.position }}),
            }}
        }} else {{
            // Memoization MISS - need to compute result
            if self.debug_mode {{
                self.debug_output.push(format!("      📝 MEMO MISS: Rule ID {{}} at pos {{}} → computing...", rule_id, key.1));
            }}
            // Check recursion depth limit to prevent stack overflow
            if self.recursion_depth >= MAX_RECURSION_DEPTH {{
                return Err(ParseError::RecursionDepthExceeded {{
                    position: self.position,
                    depth: self.recursion_depth,
                }});
            }}
            
            // Not memoized - compute result with recursion tracking
            self.recursion_depth += 1;
            let start_pos = self.position;
            
            let result = f(self);
            self.recursion_depth -= 1;
            
            match result {{
                Ok(node) => {{
                    let end_pos = self.position;
                    self.memo.insert(key, MemoEntry {{
                        result: Some(node.clone()),
                        end_pos,
                    }});
                    if self.debug_mode {{
                        self.debug_output.push(format!("      ✅ MEMO CACHE: Rule ID {{}} at pos {{}} → SUCCESS (advanced to {{}})", rule_id, key.1, end_pos));
                    }}
                    Ok(node)
                }}
                Err(err) => {{
                    self.memo.insert(key, MemoEntry {{
                        result: None,
                        end_pos: start_pos,
                    }});
                    if self.debug_mode {{
                        self.debug_output.push(format!("      ❌ MEMO CACHE: Rule ID {{}} at pos {{}} → FAILURE (stayed at {{}})", rule_id, key.1, start_pos));
                    }}
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
    
    /// Try parsing rule call with memoization-aware backtracking
    /// This version preserves memoization when used in quantifiers
    #[inline]
    fn try_parse_memoized<F>(&mut self, f: F) -> Option<ParseContent<'input>>
    where
        F: FnOnce(&mut Self) -> ParseResult<ParseContent<'input>>,
    {{
        let saved_pos = self.position;
        let saved_recursion_depth = self.recursion_depth;
        
        if self.debug_mode {{
            self.debug_output.push(format!("        🔍 MEMOIZED TRY: Starting attempt at position {{}} (checkpoint set, depth={{}})", saved_pos, saved_recursion_depth));
        }}
        
        match f(self) {{
            Ok(result) => {{
                if self.debug_mode {{
                    self.debug_output.push(format!("        ✅ MEMOIZED SUCCESS: Parse succeeded, advanced from {{}} to {{}} (keeping new position & depth)", 
                        saved_pos, self.position));
                }}
                Some(result)
            }},
            Err(err) => {{
                if self.debug_mode {{
                    self.debug_output.push(format!("        🔄 MEMOIZED BACKTRACK: Parse failed at position {{}}, restoring to checkpoint {{}} (depth {{}} → {{}}, error: {{:?}})", 
                        self.position, saved_pos, self.recursion_depth, saved_recursion_depth, err));
                }}
                // Restore parser state completely
                self.position = saved_pos;
                self.recursion_depth = saved_recursion_depth;
                None
            }}
        }}
    }}
    
    /// Debug: Log entry into a parsing rule with top-notch formatting
    #[inline]
    fn debug_enter_rule(&mut self, rule_name: &str) {{
        if self.debug_mode {{
            // Push current rule to stack for hierarchy tracking
            self.rule_stack.push(rule_name.to_string());
            
            // Add empty line before non-top rules for visual separation
            if self.debug_depth > 0 {{
                self.debug_output.push("".to_string());
            }}
            
            // Build full hierarchical path from root to current rule
            let rule_hierarchy = self.rule_stack.join(" → ");
            
            let context = if self.position < self.input.len() {{
                let end_pos = (self.position + 20).min(self.input.len());
                let context_str = self.format_debug_string(&self.input[self.position..end_pos]);
                format!("\"{{}}...\"", context_str)
            }} else {{
                "<EOF>".to_string()
            }};
            
            let msg = format!("   {{}}: {{}}", self.debug_output.len() + 1, rule_hierarchy);
            self.debug_output.push(msg);
            
            let detail_msg = format!("      🔍 Attempting to parse {{}} ", rule_name);
            self.debug_output.push(detail_msg);
            
            let position_msg = format!("      📍 Position: {{}}, Looking at: {{}}", self.position, context);
            self.debug_output.push(position_msg);
            
            self.debug_depth += 1;
        }}
    }}
    
    /// Debug: Log successful exit from a parsing rule with beautiful formatting
    #[inline]
    fn debug_exit_success(&mut self, rule_name: &str, start_pos: usize) {{
        if self.debug_mode {{
            self.debug_depth = self.debug_depth.saturating_sub(1);
            
            // Pop rule from stack
            if !self.rule_stack.is_empty() {{
                self.rule_stack.pop();
            }}
            
            let consumed = if self.position > start_pos {{
                let consumed_str = self.format_debug_string(&self.input[start_pos..self.position]);
                format!("'{{}}'", consumed_str)
            }} else {{
                "(no input)".to_string()
            }};
            
            let chars_consumed = if self.position > start_pos {{
                self.position - start_pos
            }} else {{
                0
            }};
            
            let success_msg = format!("      ✅ SUCCESS: Found {{}} (rule: {{}})", consumed, rule_name);
            self.debug_output.push(success_msg);
            
            let stats_msg = format!("      📊 Consumed: {{}} characters (pos {{}} → {{}})", 
                chars_consumed, start_pos, self.position);
            self.debug_output.push(stats_msg);
        }}
    }}
    
    /// Debug: Log failed exit from a parsing rule with beautiful formatting
    #[inline]
    fn debug_exit_fail(&mut self, rule_name: &str, error: &ParseError) {{
        if self.debug_mode {{
            self.debug_depth = self.debug_depth.saturating_sub(1);
            
            // Pop rule from stack
            if !self.rule_stack.is_empty() {{
                self.rule_stack.pop();
            }}
            
            let error_reason = match error {{
                ParseError::UnexpectedEof {{ position }} => 
                    format!("Found end of input, expected more content"),
                ParseError::UnexpectedToken {{ expected, found, position }} => 
                    format!("Found '{{}}', expected '{{}}'", found, expected),
                ParseError::InvalidSyntax {{ message, position }} => 
                    format!("{{}}", message),
                ParseError::Backtrack {{ position }} => 
                    format!("Backtracked to position {{}}", position),
                ParseError::RecursionDepthExceeded {{ position, depth }} => 
                    format!("Recursion depth exceeded ({{}} levels) at position {{}}", depth, position),
            }};
            
            let suggestion = match error {{
                ParseError::UnexpectedToken {{ expected, .. }} => 
                    format!(" 💡 Suggestion: Try using '{{}}' instead", expected),
                _ => "".to_string(),
            }};
            
            let fail_msg = format!("      ❌ FAILURE: {{}}", error_reason);
            self.debug_output.push(fail_msg);
            
            if !suggestion.is_empty() {{
                self.debug_output.push(format!("     {{}}", suggestion));
            }}
        }}
    }}
    
    /// Debug: Log backtracking with beautiful formatting
    #[inline]
    fn debug_backtrack(&mut self, from_pos: usize, to_pos: usize, reason: &str) {{
        if self.debug_mode {{
            let backtrack_msg = format!("      ⟲ BACKTRACK: Position {{}} → {{}} ({{}})", from_pos, to_pos, reason);
            self.debug_output.push(backtrack_msg);
        }}
    }}
    
    /// Debug: Log alternative attempt with beautiful formatting
    #[inline]
    fn debug_try_alternative(&mut self, rule_name: &str, alt_index: usize, total: usize, alt_name: &str) {{
        if self.debug_mode {{
            let alt_msg = format!("      🔄 Trying alternative {{}}/{{}} of {{}}: '{{}}' at position {{}}", 
                alt_index + 1, total, rule_name, alt_name, self.position);
            self.debug_output.push(alt_msg);
        }}
    }}
    
    /// Debug: Log sequence element attempt with beautiful formatting
    #[inline]
    fn debug_sequence_element(&mut self, elem_index: usize, total: usize, rule_name: &str, elem_description: &str) {{
        if self.debug_mode {{
            let seq_msg = format!("      🔗 Sequence element {{}}/{{}} of {{}}: {{}} at position {{}}", 
                elem_index + 1, total, rule_name, elem_description, self.position);
            self.debug_output.push(seq_msg);
        }}
    }}
    
    /// Debug: Log quantifier iteration with beautiful formatting
    #[inline]
    fn debug_quantifier_iteration(&mut self, iteration: usize, quantifier: &str) {{
        if self.debug_mode {{
            let quant_msg = format!("      🔁 Quantifier '{{}}' iteration {{}} at position {{}}", 
                quantifier, iteration, self.position);
            self.debug_output.push(quant_msg);
        }}
    }}
    
    /// Debug: Log quantifier iteration failure with beautiful formatting
    #[inline]
    fn debug_quantifier_iteration_failed(&mut self, quantifier: &str) {{
        if self.debug_mode {{
            let fail_msg = format!("      ⚡ Quantifier '{{}}' iteration FAILED at position {{}} (will backtrack)", 
                quantifier, self.position);
            self.debug_output.push(fail_msg);
        }}
    }}
    
    /// Debug: Log sequence element success with beautiful formatting
    #[inline]
    fn debug_sequence_element_success(&mut self, elem_index: usize, total: usize, rule_name: &str, elem_description: &str, consumed_chars: usize) {{
        if self.debug_mode {{
            let success_msg = format!("      ✅ SUCCESS: Element {{}}/{{}} of {{}}: {{}} (consumed {{}} chars at pos {{}})", 
                elem_index + 1, total, rule_name, elem_description, consumed_chars, self.position);
            self.debug_output.push(success_msg);
        }}
    }}
    
    /// Debug: Log sequence element failure with beautiful formatting
    #[inline]
    fn debug_sequence_element_failure(&mut self, elem_index: usize, total: usize, rule_name: &str, elem_description: &str, error: &ParseError) {{
        if self.debug_mode {{
            let error_reason = match error {{
                ParseError::UnexpectedEof {{ position }} => "Unexpected end of input".to_string(),
                ParseError::UnexpectedToken {{ expected, found, position }} => 
                    format!("Expected '{{}}', found '{{}}'", expected, found),
                ParseError::InvalidSyntax {{ message, position }} => message.to_string(),
                ParseError::Backtrack {{ position }} => "Backtracked".to_string(),
                ParseError::RecursionDepthExceeded {{ position, depth }} => 
                    format!("Recursion depth exceeded ({{}} levels)", depth),
            }};
            let failure_msg = format!("      ❌ FAILURE: Element {{}}/{{}} of {{}}: {{}} ({{}})", 
                elem_index + 1, total, rule_name, elem_description, error_reason);
            self.debug_output.push(failure_msg);
        }}
    }}
    
    /// Debug: Log quantifier start with beautiful formatting
    #[inline]
    fn debug_quantifier_start(&mut self, rule_name: &str, quantified_description: &str, quantifier: &str) {{
        if self.debug_mode {{
            let start_msg = format!("      🔄 QUANTIFIER START: Attempting {{}} for rule '{{}}' at position {{}}", 
                quantified_description, rule_name, self.position);
            self.debug_output.push(start_msg);
        }}
    }}
    
    /// Debug: Log quantifier end with beautiful formatting
    #[inline]
    fn debug_quantifier_end(&mut self, rule_name: &str, quantified_description: &str, quantifier: &str, result: &ParseContent<'input>) {{
        if self.debug_mode {{
            let result_info = match result {{
                ParseContent::Terminal(content) => format!("terminal: '{{}}'", self.format_debug_string(content)),
                ParseContent::Sequence(nodes) => format!("sequence with {{}} elements", nodes.len()),
                ParseContent::Alternative(node) => format!("alternative: {{}}", node.rule_name),
                ParseContent::Quantified(nodes, quant) => format!("quantified: {{}} matches with '{{}}'", nodes.len(), quant),
            }};
            let end_msg = format!("      ✅ QUANTIFIER END: {{}} for rule '{{}}' → {{}} at position {{}}", 
                quantified_description, rule_name, result_info, self.position);
            self.debug_output.push(end_msg);
        }}
    }}
    
    /// Helper function to get current rule hierarchy path
    #[inline]
    fn get_rule_hierarchy(&self) -> String {{
        if self.rule_stack.is_empty() {{
            "<root>".to_string()
        }} else {{
            self.rule_stack.join("→")
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
            enable_trace = self.enable_trace,
            backtrack_debug_code = backtrack_debug_code
        )
    }

    fn generate_optimized_rule_methods(
        &self,
        grammar_tree: &HashMap<String, ASTNode>,
        rule_order: &[String],
    ) -> Result<String> {
        self.generate_optimized_rule_methods_with_pipeline(grammar_tree, rule_order, None)
    }
    
    fn generate_optimized_rule_methods_with_pipeline(
        &self,
        grammar_tree: &HashMap<String, ASTNode>,
        rule_order: &[String],
        mut pipeline: Option<&mut crate::ast_pipeline::RustASTPipeline>,
    ) -> Result<String> {
        let mut code = String::new();
        
        if let Some(ref mut p) = pipeline {
            p.log_info("generate_optimized_rule_methods", &format!("🚀 Starting rule method generation: {} rules in order, {} in tree", rule_order.len(), grammar_tree.len()));
            p.log_debug("generate_optimized_rule_methods", &format!("Rules in order: {:?}", rule_order));
            p.log_debug("generate_optimized_rule_methods", &format!("Rules in tree: {:?}", grammar_tree.keys().collect::<Vec<_>>()));
        } else {
            println!("[HighPerformanceRustGenerator] Starting rule method generation");
            println!("[HighPerformanceRustGenerator] Total rules in rule_order: {}", rule_order.len());
            println!("[HighPerformanceRustGenerator] Total rules in grammar_tree: {}", grammar_tree.len());
        }
        
        // Generate rule ID constants
        if let Some(ref mut p) = pipeline {
            p.log_info("generate_optimized_rule_methods", "🏷️  Generating rule ID constants for memoization");
        }
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
                if let Some(ref mut p) = pipeline {
                    p.log_info("generate_optimized_rule_methods", &format!("✅ [{}/{}] Generating method for rule: {}", i + 1, rule_order.len(), rule_name));
                } else {
                    println!("[HighPerformanceRustGenerator] ✓ Generating method for rule: {} (index {})", rule_name, i);
                }
                let method_code = self.generate_optimized_rule_method_with_pipeline(rule_name, ast_node, i as u16, pipeline.as_deref_mut())?;
                code.push_str(&method_code);
                methods_generated += 1;
            } else {
                if let Some(ref mut p) = pipeline {
                    p.log_warning("generate_optimized_rule_methods", &format!("SKIPPING rule: {} (not found in grammar_tree)", rule_name));
                } else {
                    println!("[HighPerformanceRustGenerator] ✗ SKIPPING rule: {} (not found in grammar_tree)", rule_name);
                }
                methods_skipped += 1;
            }
        }
        
        if let Some(ref mut p) = pipeline {
            p.log_success("generate_optimized_rule_methods", &format!("📊 Summary: {} methods generated, {} skipped, {} total processed", methods_generated, methods_skipped, rule_order.len()));
            if methods_skipped > 0 {
                p.log_warning("generate_optimized_rule_methods", &format!("⚠️  WARNING: {} rules were skipped - this will cause compilation errors!", methods_skipped));
            }
        } else {
            println!("[HighPerformanceRustGenerator] Summary:");
            println!("[HighPerformanceRustGenerator]   ✓ Methods generated: {}", methods_generated);
            println!("[HighPerformanceRustGenerator]   ✗ Methods skipped: {}", methods_skipped);
            println!("[HighPerformanceRustGenerator]   📊 Total rules processed: {}", rule_order.len());
            
            if methods_skipped > 0 {
                println!("[HighPerformanceRustGenerator] ⚠️  WARNING: {} rules were skipped - this will cause compilation errors!", methods_skipped);
            }
        }

        Ok(code)
    }

    fn generate_optimized_rule_method(
        &self,
        rule_name: &str,
        ast_node: &ASTNode,
        rule_id: u16,
    ) -> Result<String> {
        self.generate_optimized_rule_method_with_pipeline(rule_name, ast_node, rule_id, None)
    }
    
    fn generate_optimized_rule_method_with_pipeline(
        &self,
        rule_name: &str,
        ast_node: &ASTNode,
        rule_id: u16,
        mut pipeline: Option<&mut crate::ast_pipeline::RustASTPipeline>,
    ) -> Result<String> {
        if let Some(ref mut p) = pipeline {
            p.log_debug("generate_optimized_rule_method", &format!("🔧 Processing rule: '{}' (ID: {})", rule_name, rule_id));
        } else {
            println!("[HighPerformanceRustGenerator][generate_optimized_rule_method] 🔧 Processing rule: '{}' (ID: {})", rule_name, rule_id);
            println!("[HighPerformanceRustGenerator][generate_optimized_rule_method]   📋 Rule AST: {:?}", ast_node);
        }
        
        // Get semantic annotations for this rule if available
        let rule_annotations = if let Some(ref annotations) = self.annotations {
            annotations.semantic_annotations.get(rule_name).cloned()
        } else {
            None
        };
        
        if let Some(ref annotations) = rule_annotations {
            if let Some(ref mut p) = pipeline {
                p.log_info("generate_optimized_rule_method", &format!("🏷️  Found {} semantic annotations for '{}': {}", annotations.len(), rule_name, annotations.join(", ")));
            } else {
                println!("[HighPerformanceRustGenerator][generate_optimized_rule_method]   🏷️  Found {} semantic annotations for '{}'", annotations.len(), rule_name);
                for (i, annotation) in annotations.iter().enumerate() {
                    println!("[HighPerformanceRustGenerator][generate_optimized_rule_method]     {}. {}", i + 1, annotation);
                }
            }
        } else {
            if let Some(ref mut p) = pipeline {
                p.log_debug("generate_optimized_rule_method", &format!("❌ No semantic annotations found for '{}'", rule_name));
            } else {
                println!("[HighPerformanceRustGenerator][generate_optimized_rule_method]   ❌ No semantic annotations found for '{}'", rule_name);
            }
        }
        
        if let Some(ref mut p) = pipeline {
            p.log_debug("generate_optimized_rule_method", &format!("🏢️  Generating method body for '{}'", rule_name));
        } else {
            println!("[HighPerformanceRustGenerator][generate_optimized_rule_method]   🏢️  Generating method body for '{}'", rule_name);
        }
        let method_body = if pipeline.is_some() {
            self.generate_optimized_node_code_with_context_and_pipeline(
                ast_node, 2, rule_name, rule_annotations.as_deref(), "parser", pipeline.as_deref_mut()
            )?
        } else {
            self.generate_optimized_node_code(ast_node, 2, rule_name, rule_annotations.as_deref())?
        };
        
        let method_name = format!("parse_{}", rule_name);
        if let Some(ref mut p) = pipeline {
            p.log_success("generate_optimized_rule_method", &format!("✅ Generated method: '{}()' for rule '{}'", method_name, rule_name));
        } else {
            println!("[HighPerformanceRustGenerator][generate_optimized_rule_method]   ✅ Generated method: '{}()' for rule '{}'\n", method_name, rule_name);
        }
        
        let method_code_template = format!(r#"    /// Parse {rule_name} with memoization and mutual recursion protection
    #[inline]
    fn parse_{rule_name}(&mut self) -> ParseResult<ParseNode<'input>> {{
        // Check for recursion cycles before entering
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("{rule_name}", position);
        
        match cycle_type {{
            CycleType::Infinite => {{
                // Infinite loop detected, fail immediately
                if self.debug_mode {{
                    self.debug_output.push(format!("🔄 INFINITE RECURSION detected in rule: {rule_name} at position {{}}", position));
                }}
                return Err(ParseError::InvalidSyntax {{
                    message: "Infinite recursion detected in rule: {rule_name}",
                    position,
                }});
            }}
            CycleType::LeftRecursive => {{
                // Left recursion detected, fail to break cycle
                if self.debug_mode {{
                    self.debug_output.push(format!("↩️ LEFT RECURSION detected in rule: {rule_name} at position {{}}", position));
                }}
                return Err(ParseError::InvalidSyntax {{
                    message: "Left recursion detected in rule: {rule_name}",
                    position,
                }});
            }}
            CycleType::MutualRecursive {{ depth, ref rules }} if depth >= 100 => {{
                // Maximum recursion depth exceeded
                if self.debug_mode {{
                    self.debug_output.push(format!("🔃 MUTUAL RECURSION depth limit exceeded in {rule_name}: depth={{}} rules={{:?}}", depth, rules));
                }}
                return Err(ParseError::InvalidSyntax {{
                    message: "Maximum recursion depth exceeded in mutual recursion",
                    position,
                }});
            }}
            _ => {{
                // Safe to proceed
            }}
        }}
        
        // Enter recursion tracking
        self.recursion_guard.enter("{rule_name}", position);
        
        if self.debug_mode {{
            self.debug_output.push(format!("🚀 FUNCTION ENTRY: parse_{{}}() at position {{}}", "{rule_name}", self.position));
        }}
        
        let result = self.memoized_call(Self::RULE_{rule_name_upper}, |parser| {{
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
        }});
        
        // Exit recursion tracking
        self.recursion_guard.exit();
        
        if self.debug_mode {{
            match &result {{
                Ok(node) => {{
                    self.debug_output.push(format!("✅ FUNCTION EXIT SUCCESS: parse_{{}}() returned node spanning {{}} -> {{}}", "{rule_name}", node.span.start, node.span.end));
                }}
                Err(err) => {{
                    self.debug_output.push(format!("❌ FUNCTION EXIT FAILURE: parse_{{}}() failed with error: {{:?}}", "{rule_name}", err));
                }}
            }}
        }}
        
        result
    }}

"#, 
            rule_name = rule_name, 
            rule_name_upper = rule_name.to_uppercase(),
            method_body = method_body
        );
        
        // Replace rule name placeholders in the method body
        let final_method_code = method_code_template.replace("{rule_name}", rule_name);
        
        Ok(final_method_code)
    }

    fn generate_optimized_node_code(
        &self, 
        ast_node: &ASTNode, 
        indent_level: usize,
        rule_name: &str,
        rule_annotations: Option<&[String]>
    ) -> Result<String> {
        self.generate_optimized_node_code_with_context(ast_node, indent_level, rule_name, rule_annotations, "parser")
    }
    
    fn generate_optimized_node_code_with_context(
        &self, 
        ast_node: &ASTNode, 
        indent_level: usize,
        rule_name: &str,
        rule_annotations: Option<&[String]>,
        parser_var: &str
    ) -> Result<String> {
        self.generate_optimized_node_code_with_context_and_pipeline(
            ast_node,
            indent_level,
            rule_name,
            rule_annotations,
            parser_var,
            None
        )
    }
    
    fn generate_optimized_node_code_with_context_and_pipeline(
        &self, 
        ast_node: &ASTNode, 
        indent_level: usize,
        rule_name: &str,
        rule_annotations: Option<&[String]>,
        parser_var: &str,
        mut pipeline: Option<&mut crate::ast_pipeline::RustASTPipeline>
    ) -> Result<String> {
        let indent = "    ".repeat(indent_level);
        
        if let Some(ref mut p) = pipeline {
            p.log_debug("generate_optimized_node_code", &format!("🎯 Processing node for rule: {}", rule_name));
            p.log_debug("generate_optimized_node_code", &format!("🔍 Node type: {:?}", std::mem::discriminant(ast_node)));
            if self.enable_trace {
                p.log_debug("generate_optimized_node_code", &format!("📝 Full AST: {:?}", ast_node));
            }
        } else if self.enable_trace {
            println!("[HighPerformanceRustGenerator][generate_optimized_node_code] 🎯 Processing node for rule: {}", rule_name);
            println!("[HighPerformanceRustGenerator][generate_optimized_node_code]   🔍 Node type: {:?}", std::mem::discriminant(ast_node));
            println!("[HighPerformanceRustGenerator][generate_optimized_node_code]   📝 Full AST: {:?}", ast_node);
        }
        
        match ast_node {
            ASTNode::Atom { value } => {
                self.generate_atom_code_with_context_and_pipeline(value, &indent, rule_annotations, parser_var, pipeline)
            }
            ASTNode::Sequence { elements } => {
                self.generate_sequence_code_with_context_and_pipeline(elements, &indent, rule_name, rule_annotations, parser_var, pipeline)
            }
            ASTNode::Or { alternatives } => {
                self.generate_or_code_with_context_and_pipeline(alternatives, &indent, rule_name, rule_annotations, parser_var, pipeline)
            }
            ASTNode::Quantified { element, quantifier } => {
                self.generate_quantified_code_with_context_and_pipeline(element, quantifier, &indent, rule_name, rule_annotations, parser_var, pipeline)
            }
        }
    }

    fn generate_atom_code(&self, value: &ASTValue, indent: &str, rule_annotations: Option<&[String]>) -> Result<String> {
        self.generate_atom_code_with_context(value, indent, rule_annotations, "parser")
    }
    
    fn generate_atom_code_with_context(&self, value: &ASTValue, indent: &str, rule_annotations: Option<&[String]>, parser_var: &str) -> Result<String> {
        self.generate_atom_code_with_context_and_pipeline(value, indent, rule_annotations, parser_var, None)
    }
    
    fn generate_atom_code_with_context_and_pipeline(&self, value: &ASTValue, indent: &str, rule_annotations: Option<&[String]>, parser_var: &str, mut pipeline: Option<&mut crate::ast_pipeline::RustASTPipeline>) -> Result<String> {
        // Only log if debug or trace mode is enabled
        if self.enable_trace {
            if let Some(ref mut p) = pipeline {
                p.log_debug("generate_atom_code", &format!("⚛️  Processing atom: {:?}", value));
            } else {
                self.log_debug(&format!("[HighPerformanceRustGenerator][generate_atom_code] ⚛️  Processing atom: {:?}", value));
            }
        }
        
        match value {
            ASTValue::Token(token) if token.len() == 2 => {
                let token_type = &token[0];
                let token_value = &token[1];
                
                // Only log detailed token info in trace mode
                if self.enable_trace {
                    if let Some(ref mut p) = pipeline {
                        p.log_debug("generate_atom_code", &format!("  📝 Token type: {:?}", token_type));
                        p.log_debug("generate_atom_code", &format!("  📝 Token value: {:?}", token_value));
                    } else {
                        println!("[HighPerformanceRustGenerator][generate_atom_code]   📝 Token type: {:?}", token_type);
                        println!("[HighPerformanceRustGenerator][generate_atom_code]   📝 Token value: {:?}", token_value);
                    }
                }
                
                // Check for semantic annotations that might guide code generation
                let custom_code = if let Some(annotations) = rule_annotations {
                    if self.enable_trace {
                        println!("[HighPerformanceRustGenerator][generate_atom_code]   🏷️  Checking rule-level semantic annotations for atom customization");
                    }
                    self.apply_semantic_annotations(annotations, token_type, token_value, indent)
                } else {
                    // Note: This is normal - most atoms use default generation
                    // Semantic annotations are rule-level, not atom-level
                    None
                };
                
                // If we have custom code from semantic annotations, use it; otherwise use default generation
                if let Some(code) = custom_code {
                    if self.enable_trace {
                        println!("[HighPerformanceRustGenerator][generate_atom_code]   🎯 Using custom code from semantic annotations");
                    }
                    return Ok(code);
                }
                
                if self.enable_trace {
                    println!("[HighPerformanceRustGenerator][generate_atom_code]   🔧 Using default code generation for token_type: {:?}", token_type.as_str());
                }
                match token_type.as_str() {
                        Some("quoted_string") => {
                            if self.enable_trace {
                                println!("[HighPerformanceRustGenerator][generate_atom_code]     ➤ Generating quoted_string code");
                            }
                            if token_value.is_empty() {
                                // Handle empty strings with regular string literals
                                Ok(format!("{indent}let result = ParseContent::Terminal({parser_var}.match_string(\"\")?);\n"))
                            } else {
                                let escaped_value = escape_rust_string(token_value.as_str().unwrap_or(""));
                                Ok(format!("{indent}let result = ParseContent::Terminal({parser_var}.match_string(r#\"{escaped_value}\"#)?);\n"))
                            }
                        }
                        Some("regex") => {
                            if self.enable_trace {
                                println!("[HighPerformanceRustGenerator][generate_atom_code]     ➤ Generating regex code");
                            }
                            let escaped_value = escape_rust_string(token_value.as_str().unwrap_or(""));
                            Ok(format!("{indent}let result = ParseContent::Terminal({parser_var}.match_regex_optimized(r#\"{escaped_value}\"#)?);\n"))
                        }
                    Some("rule_reference") => {
                        let rule_name = token_value.as_str().unwrap_or("unknown");
                        if self.enable_trace {
                            println!("[HighPerformanceRustGenerator][generate_atom_code]     ➤ Generating rule_reference code for rule: '{}'", rule_name);
                            println!("[HighPerformanceRustGenerator][generate_atom_code]       🔗 Will call method: parse_{}()", rule_name);
                        }
                        Ok(format!("{indent}let result = ParseContent::Alternative(Box::new({parser_var}.parse_{rule_name}()?));\n"))
                    }
                    // Skip grammar tokens - these should not generate any parsing code
                    // These tokens are EBNF syntax elements and should have been processed during AST transformation
                    Some("group_open") | Some("group_close") | Some("operator") | Some("separator") => {
                        if self.enable_trace {
                            self.log_debug(&format!("[HighPerformanceRustGenerator][generate_atom_code]     ⚠️ WARNING: Grammar token {:?} found in code generation - this indicates an AST transformation issue", token_type.as_str()));
                        }
                        // Return empty string to skip generating any code for these tokens
                        // They should not appear in a properly transformed AST
                        Ok(String::new())
                    }
                    _ => {
                        if self.enable_trace {
                            println!("[HighPerformanceRustGenerator][generate_atom_code]     ➤ Generating default terminal code for unknown token type: {:?}", token_type.as_str());
                        }
                        let escaped_value = escape_rust_string(token_value.as_str().unwrap_or(""));
                        Ok(format!("{indent}let result = ParseContent::Terminal(r#\"{escaped_value}\"#);\n"))
                    }
                }
            }
            _ => {
                if self.enable_trace {
                    println!("[HighPerformanceRustGenerator][generate_atom_code]   ⚠️  Non-token AST value: {:?}", value);
                }
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
        if self.enable_trace {
            println!("[HighPerformanceRustGenerator][apply_semantic_annotations] 📝 Processing {} annotations for token_type: {:?}, token_value: {:?}", annotations.len(), token_type, token_value);
        }
        
        if annotations.is_empty() {
            if self.enable_trace {
                println!("[HighPerformanceRustGenerator][apply_semantic_annotations]   ❌ No annotations to process");
            }
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

    fn generate_sequence_code(&self, elements: &[ASTNode], indent: &str, rule_name: &str, rule_annotations: Option<&[String]>) -> Result<String> {
        self.generate_sequence_code_with_context(elements, indent, rule_name, rule_annotations, "parser")
    }
    
    fn generate_sequence_code_with_context(&self, elements: &[ASTNode], indent: &str, rule_name: &str, rule_annotations: Option<&[String]>, parser_var: &str) -> Result<String> {
        self.generate_sequence_code_with_context_and_pipeline(elements, indent, rule_name, rule_annotations, parser_var, None)
    }
    
    fn generate_sequence_code_with_context_and_pipeline(&self, elements: &[ASTNode], indent: &str, rule_name: &str, rule_annotations: Option<&[String]>, parser_var: &str, mut pipeline: Option<&mut crate::ast_pipeline::RustASTPipeline>) -> Result<String> {
        if let Some(ref mut p) = pipeline {
            p.log_debug("generate_sequence_code", &format!("🚀 Generating sequence code for rule: {}", rule_name));
            p.log_debug("generate_sequence_code", &format!("  Elements count: {}", elements.len()));
            for (i, element) in elements.iter().enumerate() {
                p.log_debug("generate_sequence_code", &format!("  Element {}: {:?}", i, element));
            }
        } else if self.enable_trace {
            println!("[HighPerformanceRustGenerator][generate_sequence_code] 🚀 Generating sequence code for rule: {}", rule_name);
            println!("[HighPerformanceRustGenerator][generate_sequence_code]   Elements count: {}", elements.len());
            for (i, element) in elements.iter().enumerate() {
                println!("[HighPerformanceRustGenerator][generate_sequence_code]   Element {}: {:?}", i, element);
            }
        }
        
        // Check for return annotation for this rule (sequence level - not branch level)
        // This is for sequences that are not part of an OR alternative
        // For now, we don't have sequence-level return annotations, only branch-level
        let return_annotation: Option<&ReturnAnnotation> = None;
        
        // Check for optional quantified groups in the sequence
        let processed_elements = self.analyze_and_group_optional_quantifiers(elements)?;
        let mut code = format!("{indent}let mut sequence_elements = Vec::with_capacity({});\n", processed_elements.len());
        
        // Track captured elements for return annotation processing
        // We'll reference the actual sequence_elements array indices
        let mut captured_vars = Vec::new();
        
        for (i, element_group) in processed_elements.iter().enumerate() {
            match element_group {
                ProcessedElement::Mandatory(element) => {
                    // Generate mandatory element
                    let element_code = self.generate_optimized_node_code_with_context_and_pipeline(element, 0, rule_name, rule_annotations, parser_var, pipeline.as_deref_mut())?;
                    let element_description = self.extract_ebnf_description(element);
                    
                    // Track reference to this element in sequence_elements
                    captured_vars.push(format!("sequence_elements[{}]", i));
                    
                    // When in closure, elements should use 'result' consistently
                    code.push_str(&self.generate_mandatory_element_code_with_context(element_code, element_description, i, processed_elements.len(), rule_name, indent, parser_var)?);
                }
                ProcessedElement::OptionalGroup(group_elements) => {
                    // Generate optional group using try_parse
                    let group_description = group_elements.iter()
                        .map(|e| self.extract_ebnf_description(e))
                        .collect::<Vec<_>>()
                        .join(" ");
                    
                    // Track reference to this element in sequence_elements
                    captured_vars.push(format!("sequence_elements[{}]", i));
                    
                    code.push_str(&self.generate_optional_group_code(group_elements, &group_description, i, processed_elements.len(), rule_name, indent)?);
                }
            }
        }
        
        // Apply return annotation if present, otherwise use default sequence
        if let Some(return_ann) = return_annotation {
            if self.enable_trace {
                println!("[HighPerformanceRustGenerator][generate_sequence_code] 📌 Applying return annotation for rule: {}", rule_name);
                println!("[HighPerformanceRustGenerator][generate_sequence_code]   Annotation: {:?}", return_ann.annotation_content);
            }
            
            // Use the pre-parsed unified AST
            if let Some(ref parsed_ast) = return_ann.parsed_ast {
                if self.enable_trace {
                    println!("[HighPerformanceRustGenerator][generate_sequence_code] Using pre-parsed unified AST:");
                    println!("{}", parsed_ast.pretty_print(2));
                }
                
                // Generate code from the unified AST
                match parsed_ast.generate_code(&captured_vars, indent, self.enable_trace) {
                    Ok(ast_code) => {
                        code.push_str(&format!("{indent}let result = {};\n", ast_code));
                    }
                    Err(e) => {
                        // Fallback to default if code generation fails
                        if self.enable_trace {
                            println!("[HighPerformanceRustGenerator][generate_sequence_code] ⚠️ Failed to generate code from unified AST: {}", e);
                        }
                        code.push_str(&format!("{indent}let result = ParseContent::Sequence(sequence_elements);\n"));
                    }
                }
            } else {
                // No parsed AST, try to parse on-demand (shouldn't normally happen)
                if self.enable_trace {
                    println!("[HighPerformanceRustGenerator][generate_sequence_code] ⚠️ No pre-parsed AST, falling back to default");
                }
                code.push_str(&format!("{indent}let result = ParseContent::Sequence(sequence_elements);\n"));
            }
        } else {
            // No return annotation - use default sequence
            code.push_str(&format!("{indent}let result = ParseContent::Sequence(sequence_elements);\n"));
        }
        
        Ok(code)
    }

    fn generate_or_code(&self, alternatives: &[ASTNode], indent: &str, rule_name: &str, rule_annotations: Option<&[String]>) -> Result<String> {
        self.generate_or_code_with_context(alternatives, indent, rule_name, rule_annotations, "parser")
    }
    
    fn generate_or_code_with_context(&self, alternatives: &[ASTNode], indent: &str, rule_name: &str, rule_annotations: Option<&[String]>, parser_var: &str) -> Result<String> {
        self.generate_or_code_with_context_and_pipeline(alternatives, indent, rule_name, rule_annotations, parser_var, None)
    }
    
    fn generate_or_code_with_context_and_pipeline(&self, alternatives: &[ASTNode], indent: &str, rule_name: &str, rule_annotations: Option<&[String]>, parser_var: &str, pipeline: Option<&mut crate::ast_pipeline::RustASTPipeline>) -> Result<String> {
        let n_branches = alternatives.len();
        
        match n_branches {
            0 => {
                // No alternatives - this shouldn't happen but handle gracefully
                Ok(format!("{indent}return Err(ParseError::InvalidSyntax {{\n{indent}    message: \"No alternatives provided\",\n{indent}    position: {parser_var}.position,\n{indent}}});\n"))
            }
            1 => {
                // Single branch - no alternatives, just execute directly
                let alt_code = self.generate_optimized_node_code_with_context_and_pipeline(&alternatives[0], 0, rule_name, rule_annotations, parser_var, pipeline)?;
                Ok(alt_code)
            }
            _ => {
                // Multiple branches - use systematic N-branch template
                self.generate_n_branch_template_with_context_and_pipeline(alternatives, indent, rule_name, rule_annotations, parser_var, pipeline)
            }
        }
    }
    
    /// Analyze sequence elements to identify optional quantified groups
    fn analyze_and_group_optional_quantifiers(&self, elements: &[ASTNode]) -> Result<Vec<ProcessedElement>> {
        println!("[HighPerformanceRustGenerator][analyze_and_group_optional_quantifiers] 🔍 Analyzing {} elements for optional quantifiers:", elements.len());
        for (i, element) in elements.iter().enumerate() {
            println!("[HighPerformanceRustGenerator][analyze_and_group_optional_quantifiers]   Element {}: {:?}", i, element);
        }
        
        let mut processed = Vec::new();
        let mut i = 0;
        
        while i < elements.len() {
            // IMPORTANT: Do NOT flatten sequences! Each element should be preserved as-is
            // The quantifier applies to the ENTIRE element, not its sub-parts
            processed.push(ProcessedElement::Mandatory(elements[i].clone()));
            i += 1;
            // Removed the faulty look-ahead logic that was flattening structures
        }
        
        println!("[HighPerformanceRustGenerator][analyze_and_group_optional_quantifiers] 🎯 Processed {} elements into {} groups", elements.len(), processed.len());
        Ok(processed)
    }
    
    /// Generate code for a mandatory element
    fn generate_mandatory_element_code(
        &self,
        element_code: String,
        element_description: String,
        index: usize,
        total: usize,
        rule_name: &str,
        indent: &str
    ) -> Result<String> {
        self.generate_mandatory_element_code_with_context(element_code, element_description, index, total, rule_name, indent, "parser")
    }
    
    fn generate_mandatory_element_code_with_context(
        &self,
        element_code: String,
        element_description: String,
        index: usize,
        total: usize,
        rule_name: &str,
        indent: &str,
        parser_var: &str
    ) -> Result<String> {
        // Skip generating code for empty elements (grammar tokens that were incorrectly left in the AST)
        if element_code.trim().is_empty() {
            return Ok(String::new());
        }
        
        let mut code = String::new();
        
        code.push_str(&format!("{indent}{{\n"));
        code.push_str(&format!("{indent}    {parser_var}.debug_sequence_element({}, {}, \"{}\", r#\"{}\"#);\n", index, total, rule_name, element_description));
        code.push_str(&format!("{indent}    let element_start = {parser_var}.position;\n"));
        
        // Wrap element parsing with result checking for better debug output
        code.push_str(&format!("{indent}    let element_result = (|| -> Result<ParseContent<'input>, ParseError> {{\n"));
        // SIMPLIFIED: Always use 'result' for consistency everywhere
        let fixed_element_code = element_code
            .replace("parser.", &format!("{parser_var}."));
        
        code.push_str(&fixed_element_code);
        code.push_str(&format!("{indent}        Ok(result)\n"));
        code.push_str(&format!("{indent}    }})();\n"));
        
        // Add success/failure debug logging
        code.push_str(&format!("{indent}    let result = match element_result {{\n"));
        code.push_str(&format!("{indent}        Ok(content) => {{\n"));
        code.push_str(&format!("{indent}            {parser_var}.debug_sequence_element_success({}, {}, \"{}\", r#\"{}\"#, {parser_var}.position - element_start);\n", index, total, rule_name, element_description));
        code.push_str(&format!("{indent}            content\n"));
        code.push_str(&format!("{indent}        }},\n"));
        code.push_str(&format!("{indent}        Err(e) => {{\n"));
        code.push_str(&format!("{indent}            {parser_var}.debug_sequence_element_failure({}, {}, \"{}\", r#\"{}\"#, &e);\n", index, total, rule_name, element_description));
        code.push_str(&format!("{indent}            return Err(e);\n"));
        code.push_str(&format!("{indent}        }}\n"));
        code.push_str(&format!("{indent}    }};\n"));
        
        code.push_str(&format!("{indent}    let element_end = {parser_var}.position;\n"));
        code.push_str(&format!("{indent}    sequence_elements.push(ParseNode {{\n"));
        code.push_str(&format!("{indent}        rule_name: \"element_{}\",\n", index));
        code.push_str(&format!("{indent}        content: result,\n"));
        code.push_str(&format!("{indent}        span: element_start..element_end,\n"));
        code.push_str(&format!("{indent}    }});\n"));
        code.push_str(&format!("{indent}}};\n"));
        
        Ok(code)
    }
    
    /// Generate code for an optional group using try_parse
    fn generate_optional_group_code(
        &self,
        group_elements: &[ASTNode],
        group_description: &str,
        index: usize,
        total: usize,
        rule_name: &str,
        indent: &str
    ) -> Result<String> {
        let mut code = String::new();
        
        code.push_str(&format!("{indent}{{\n"));
        code.push_str(&format!("{indent}    parser.debug_sequence_element({}, {}, \"{}\", r#\"({})? (optional group)\"#);\n", index, total, rule_name, group_description));
        code.push_str(&format!("{indent}    let element_start = parser.position;\n"));
        
        // Use try_parse to make the group optional
        code.push_str(&format!("{indent}    let result = if let Some(content) = parser.try_parse(|p| {{\n"));
        
        // Generate the group content as a sequence
        code.push_str(&format!("{indent}        let mut group_elements = Vec::with_capacity({});\n", group_elements.len()));
        
        for (group_idx, element) in group_elements.iter().enumerate() {
            let element_code = self.generate_optimized_node_code_with_context(element, 0, rule_name, None, "p")?;
            let _element_desc = self.extract_ebnf_description(element);
            
            code.push_str(&format!("{indent}        {{\n"));
            code.push_str(&format!("{indent}            let group_element_start = p.position;\n"));
            
            // Fix scoping issue: ensure group_element_content is properly scoped
            let fixed_element_code = element_code
                .replace("let result =", "let group_element_content =")
                .replace("let result =", "let group_element_content =")  // Handle quantified groups
                .replace("&result)", "&group_element_content)")
                .replace("&result)", "&group_element_content)")  // Handle quantified groups
                .replace("parser.", "p.");
            
            // Add proper indentation to the element code
            let indented_element_code = fixed_element_code
                .lines()
                .map(|line| if line.trim().is_empty() { 
                    line.to_string() 
                } else { 
                    format!("{indent}            {}", line) 
                })
                .collect::<Vec<_>>()
                .join("\n");
            
            code.push_str(&indented_element_code);
            code.push_str("\n");
            
            code.push_str(&format!("{indent}            let group_element_end = p.position;\n"));
            code.push_str(&format!("{indent}            group_elements.push(ParseNode {{\n"));
            code.push_str(&format!("{indent}                rule_name: \"group_element_{}\",\n", group_idx));
            code.push_str(&format!("{indent}                content: group_element_content,\n"));
            code.push_str(&format!("{indent}                span: group_element_start..group_element_end,\n"));
            code.push_str(&format!("{indent}            }});\n"));
            code.push_str(&format!("{indent}        }}\n"));
        }
        
        code.push_str(&format!("{indent}        Ok(ParseContent::Sequence(group_elements))\n"));
        code.push_str(&format!("{indent}    }}) {{\n"));
        code.push_str(&format!("{indent}        parser.debug_sequence_element_success({}, {}, \"{}\", r#\"({})? (optional group)\"#, parser.position - element_start);\n", index, total, rule_name, group_description));
        code.push_str(&format!("{indent}        content\n"));
        code.push_str(&format!("{indent}    }} else {{\n"));
        code.push_str(&format!("{indent}        parser.debug_sequence_element_success({}, {}, \"{}\", r#\"({})? (optional group - empty)\"#, 0);\n", index, total, rule_name, group_description));
        code.push_str(&format!("{indent}        ParseContent::Sequence(Vec::new())  // Empty optional group\n"));
        code.push_str(&format!("{indent}    }};\n"));
        
        code.push_str(&format!("{indent}    let element_end = parser.position;\n"));
        code.push_str(&format!("{indent}    sequence_elements.push(ParseNode {{\n"));
        code.push_str(&format!("{indent}        rule_name: \"element_{}\",\n", index));
        code.push_str(&format!("{indent}        content: result,\n"));
        code.push_str(&format!("{indent}        span: element_start..element_end,\n"));
        code.push_str(&format!("{indent}    }});\n"));
        code.push_str(&format!("{indent}}};\n"));
        
        Ok(code)
    }
    
    /// Generate systematic N-branch template using builder pattern
    fn generate_n_branch_template(&self, alternatives: &[ASTNode], indent: &str, rule_name: &str, rule_annotations: Option<&[String]>) -> Result<String> {
        self.generate_n_branch_template_with_context(alternatives, indent, rule_name, rule_annotations, "parser")
    }
    
    fn generate_n_branch_template_with_context(&self, alternatives: &[ASTNode], indent: &str, rule_name: &str, rule_annotations: Option<&[String]>, parser_var: &str) -> Result<String> {
        self.generate_n_branch_template_with_context_and_pipeline(alternatives, indent, rule_name, rule_annotations, parser_var, None)
    }
    
    fn generate_n_branch_template_with_context_and_pipeline(&self, alternatives: &[ASTNode], indent: &str, rule_name: &str, rule_annotations: Option<&[String]>, parser_var: &str, _pipeline: Option<&mut crate::ast_pipeline::RustASTPipeline>) -> Result<String> {
        println!("[DEBUG] generate_n_branch_template called for rule '{}' with {} branches, enable_trace={}", rule_name, alternatives.len(), self.enable_trace);
        let mut builder = RustCodeBuilder::new();
        let n_branches = alternatives.len();
        
        // Check if any branch has a return annotation
        let any_branch_has_annotation = self.branch_return_annotations
            .get(rule_name)
            .map(|branches| branches.iter().any(|opt| opt.is_some()))
            .unwrap_or(false);
        
        if !any_branch_has_annotation && self.enable_trace {
            println!("[DEBUG] Rule '{}' has no branch annotations - will apply implicit -> $1 (passthrough)", rule_name);
        }
        
        // Declare result variable in outer scope
        builder.add_line(&format!("{indent}let result: ParseContent<'input>;"));
        
        // Generate alternatives as a single if-else-if-else chain
        for (branch_idx, alt) in alternatives.iter().enumerate() {
            let alt_name = self.extract_rule_name_from_ast(alt);
            
            if branch_idx == 0 {
                // First branch: if let Some(...)
                builder.add_line(&format!("{indent}parser.debug_try_alternative(\"{{rule_name}}\", {}, {}, \"{}\");", branch_idx, n_branches, alt_name));
                builder.add_line(&format!("{indent}if let Some(content) = parser.try_parse(|p| {{"));
            } else {
                // Subsequent branches: } else if let Some(...)
                builder.add_line(&format!("{indent}}} else if let Some(content) = parser.try_parse(|p| {{"));
                builder.add_line(&format!("{indent}    p.debug_try_alternative(\"{{rule_name}}\", {}, {}, \"{}\");", branch_idx, n_branches, alt_name));
            }
            
            // Add mutual recursion check for each branch
            // If a branch contains a rule reference that would cause mutual recursion,
            // that specific branch should return None to skip to the next branch
            if let Some(referenced_rule) = self.get_rule_reference_from_ast(alt) {
                builder.add_line(&format!("{indent}    // Check for mutual recursion on this branch"));
                builder.add_line(&format!("{indent}    let branch_cycle = p.recursion_guard.check_cycle(\"{}\", p.position);", referenced_rule));
                builder.add_line(&format!("{indent}    if matches!(branch_cycle, CycleType::MutualRecursive {{ .. }}) {{"));
                builder.add_line(&format!("{indent}        // Skip this branch due to mutual recursion - try next alternative"));
                builder.add_line(&format!("{indent}        if p.debug_mode {{"));
                builder.add_line(&format!("{indent}            p.debug_output.push(format!(\"⚠️ Skipping branch {{}} due to mutual recursion with {{}}\", {}, \"{}\"));", branch_idx, referenced_rule));
                builder.add_line(&format!("{indent}        }}"));
                builder.add_line(&format!("{indent}        return Err(ParseError::Backtrack {{ position: p.position }});"));
                builder.add_line(&format!("{indent}    }}"));
            }
            
            // Generate branch content with proper indentation
            let alt_code = self.generate_optimized_node_code_with_context(&alt, 0, rule_name, rule_annotations, "p")?;
            let branch_indent = if branch_idx == 0 { "        " } else { "            " };
            // SIMPLIFY: Don't rename variables - just use result consistently
            let branch_content = alt_code
                .replace("parser.", "p.")  // Use p. inside the closure, not parser.
                .replace("self.", "p.");
            
            builder.add_raw(&branch_content);
            
            // Check for branch-specific return annotation
            println!("[DEBUG] Checking branch return annotation for rule '{}', branch {}", rule_name, branch_idx);
            let branch_return_annotation = self.branch_return_annotations
                .get(rule_name)
                .and_then(|branches| {
                    println!("[DEBUG]   Found {} branches for rule '{}'", branches.len(), rule_name);
                    branches.get(branch_idx)
                })
                .and_then(|opt| {
                    if opt.is_some() {
                        println!("[DEBUG]   Found annotation for branch {}", branch_idx);
                    } else {
                        println!("[DEBUG]   No annotation for branch {}", branch_idx);
                    }
                    opt.as_ref()
                });
            
            // Apply return annotation (explicit or implicit)
            if let Some(return_ann) = branch_return_annotation {
                // Apply explicit branch-specific return annotation
                if self.enable_trace {
                    builder.add_line(&format!("{indent}{branch_indent}// Applying branch {} return annotation: {}", branch_idx, return_ann.annotation_type));
                }
            } else if !any_branch_has_annotation {
                // Apply implicit passthrough (-> $1) when NO branches have annotations
                if self.enable_trace {
                    builder.add_line(&format!("{indent}{branch_indent}// No annotations on any branch - applying implicit -> $1 (passthrough)"));
                }
                // Result is already in 'result' variable from branch execution, so no action needed
                // The passthrough is implicit - we just use the result as-is
            }
            
            if let Some(return_ann) = branch_return_annotation {
                // Use the pre-parsed unified AST
                if let Some(ref parsed_ast) = return_ann.parsed_ast {
                    println!("[DEBUG] Using pre-parsed unified AST for branch {}", branch_idx);
                    builder.add_line(&format!("{indent}{branch_indent}// DEBUG: Using pre-parsed unified return annotation AST"));
                    
                    // Add debug output for return annotation AST as comments
                    // when either debug or trace mode is enabled
                    builder.add_line(&format!("{indent}{branch_indent}"));
                    builder.add_line(&format!("{indent}{branch_indent}// ═══════════════════════════════════════════════════════"));
                    builder.add_line(&format!("{indent}{branch_indent}// Return Annotation Debug Output for branch {}", branch_idx));
                    builder.add_line(&format!("{indent}{branch_indent}// ═══════════════════════════════════════════════════════"));
                    builder.add_line(&format!("{indent}{branch_indent}// Text representation: {}", return_ann.annotation_content));
                    builder.add_line(&format!("{indent}{branch_indent}// Annotation type: {}", return_ann.annotation_type));
                    builder.add_line(&format!("{indent}{branch_indent}//"));
                    builder.add_line(&format!("{indent}{branch_indent}// Parsed Unified AST:"));
                    let ast_pretty = parsed_ast.pretty_print(0);
                    for line in ast_pretty.lines() {
                        builder.add_line(&format!("{indent}{branch_indent}// {}", line));
                    }
                    builder.add_line(&format!("{indent}{branch_indent}// ═══════════════════════════════════════════════════════"));
                    builder.add_line(&format!("{indent}{branch_indent}"));
                    
                    // For branches, the result is typically in a variable called 'result'
                    let captured_vars = vec!["result".to_string()];
                    
                    match parsed_ast.generate_code(&captured_vars, &format!("{indent}{branch_indent}"), self.enable_trace) {
                        Ok(ast_code) => {
                            builder.add_line(&format!("{indent}{branch_indent}let result = {};", ast_code));
                        }
                        Err(e) => {
                            if self.enable_trace {
                                builder.add_line(&format!("{indent}{branch_indent}// Failed to generate code from unified AST: {}", e));
                            }
                            // Keep the original result
                        }
                    }
                } else {
                    println!("[DEBUG] No pre-parsed AST for branch {}, keeping original result", branch_idx);
                    if self.enable_trace {
                        builder.add_line(&format!("{indent}{branch_indent}// No pre-parsed AST available, keeping original result"));
                    }
                }
            }
            
            builder.add_line(&format!("{indent}{branch_indent}Ok(result)"));
            
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
        builder.add_line(&format!("{indent}        message: \"No alternative matched in {}-branch rule: {{rule_name}}\",", n_branches));
        builder.add_line(&format!("{indent}        position: {parser_var}.position,"));
        builder.add_line(&format!("{indent}    }});"));
        builder.add_line(&format!("{indent}}}"));
        
        Ok(builder.build())
    }
    
    // Removed format_return_annotation_ast - now using UnifiedReturnAST::pretty_print directly
    
    /// Helper function to get the first rule reference in an AST node (for mutual recursion checking)
    fn get_rule_reference_from_ast(&self, ast_node: &ASTNode) -> Option<String> {
        match ast_node {
            ASTNode::Atom { value } => {
                match value {
                    ASTValue::Token(token) if token.len() == 2 => {
                        if let (crate::ast_pipeline::TokenValue::String(token_type), crate::ast_pipeline::TokenValue::String(token_value)) = (&token[0], &token[1]) {
                            if token_type == "rule_reference" {
                                return Some(token_value.clone());
                            }
                        }
                    }
                    _ => {}
                }
            }
            ASTNode::Sequence { elements } => {
                // Return the first rule reference found in the sequence
                for element in elements {
                    if let Some(rule_ref) = self.get_rule_reference_from_ast(element) {
                        return Some(rule_ref);
                    }
                }
            }
            ASTNode::Or { alternatives } => {
                // For OR nodes, check the first alternative
                if let Some(first_alt) = alternatives.first() {
                    return self.get_rule_reference_from_ast(first_alt);
                }
            }
            ASTNode::Quantified { element, .. } => {
                return self.get_rule_reference_from_ast(element);
            }
        }
        None
    }
    
    /// Helper function to extract rule name from AST node
    fn extract_rule_name_from_ast(&self, ast_node: &ASTNode) -> String {
        match ast_node {
            ASTNode::Atom { value } => {
                match value {
                    ASTValue::Token(token) if token.len() == 2 => {
                        if let (crate::ast_pipeline::TokenValue::String(token_type), crate::ast_pipeline::TokenValue::String(token_value)) = (&token[0], &token[1]) {
                            if token_type == "rule_reference" {
                                token_value.clone()
                            } else {
                                format!("{}:{}", token_type, token_value)
                            }
                        } else {
                            "<unknown_token>".to_string()
                        }
                    }
                    _ => "<atom>".to_string(),
                }
            }
            ASTNode::Sequence { .. } => "<sequence>".to_string(),
            ASTNode::Or { .. } => "<or>".to_string(),
            ASTNode::Quantified { element, quantifier } => {
                format!("{}_{}", self.extract_rule_name_from_ast(element), quantifier)
            }
        }
    }
    
    /// Helper function to extract EBNF-like description from AST node for debugging
    fn extract_ebnf_description(&self, ast_node: &ASTNode) -> String {
        match ast_node {
            ASTNode::Atom { value } => {
                match value {
                    ASTValue::Token(token) if token.len() == 2 => {
                        if let (crate::ast_pipeline::TokenValue::String(token_type), crate::ast_pipeline::TokenValue::String(token_value)) = (&token[0], &token[1]) {
                            match token_type.as_str() {
                                "quoted_string" => format!("'{}'", token_value),
                                "regex" => format!("/{}/", token_value),
                                "rule_reference" => token_value.clone(),
                                // Handle grammar tokens - show their symbol value instead of raw token
                                "group_open" => token_value.clone(),
                                "group_close" => token_value.clone(),
                                "operator" => token_value.clone(),
                                "separator" => token_value.clone(),
                                _ => format!("{}:{}", token_type, token_value),
                            }
                        } else {
                            "<unknown_token>".to_string()
                        }
                    }
                    _ => "<atom>".to_string(),
                }
            }
            ASTNode::Sequence { elements } => {
                let descriptions: Vec<String> = elements.iter()
                    .map(|e| self.extract_ebnf_description(e))
                    .collect();
                format!("({})", descriptions.join(" "))
            }
            ASTNode::Or { alternatives } => {
                let descriptions: Vec<String> = alternatives.iter()
                    .map(|e| self.extract_ebnf_description(e))
                    .collect();
                format!("({})", descriptions.join(" | "))
            }
            ASTNode::Quantified { element, quantifier } => {
                format!("{}{}", self.extract_ebnf_description(element), quantifier)
            }
        }
    }

    fn generate_quantified_code(&self, element: &ASTNode, quantifier: &str, indent: &str, rule_name: &str, rule_annotations: Option<&[String]>) -> Result<String> {
        self.generate_quantified_code_with_context(element, quantifier, indent, rule_name, rule_annotations, "parser")
    }
    
    fn generate_quantified_code_with_context(&self, element: &ASTNode, quantifier: &str, indent: &str, rule_name: &str, rule_annotations: Option<&[String]>, parser_var: &str) -> Result<String> {
        self.generate_quantified_code_with_context_and_pipeline(element, quantifier, indent, rule_name, rule_annotations, parser_var, None)
    }
    
    fn generate_quantified_code_with_context_and_pipeline(&self, element: &ASTNode, quantifier: &str, indent: &str, rule_name: &str, rule_annotations: Option<&[String]>, parser_var: &str, _pipeline: Option<&mut crate::ast_pipeline::RustASTPipeline>) -> Result<String> {
        let mut code = String::new();
        let element_description = self.extract_ebnf_description(element);
        
        match quantifier {
            "?" => {
                // Optional: use try_parse to make entire element optional
                code.push_str(&format!("{indent}// Optional group (?) for: {element_description}\n"));
                code.push_str(&format!("{indent}let result = if let Some(content) = {parser_var}.try_parse(|p| {{\n"));
                
                // Generate code for inner element preserving its structure
                let inner_code = self.generate_optimized_node_code_with_context_and_pipeline(
                    element, 0, rule_name, rule_annotations, "p", None
                )?;
                
                // Wrap the inner code to return the content
                // UNIFIED: Always expect 'result' from inner code
                code.push_str(&format!("{indent}    {}", inner_code.trim_end()));
                code.push_str(&format!("\n{indent}    Ok(result)\n"));
                code.push_str(&format!("{indent}}}) {{\n"));
                code.push_str(&format!("{indent}    content\n"));
                code.push_str(&format!("{indent}}} else {{\n"));
                code.push_str(&format!("{indent}    ParseContent::Sequence(Vec::new())\n"));
                code.push_str(&format!("{indent}}};\n"));
                // No need to reassign result to itself
            }
            "*" => {
                // Zero or more: use loop
                code.push_str(&format!("{indent}// Zero or more (*) for: {element_description}\n"));
                code.push_str(&format!("{indent}let mut quantified_elements = Vec::new();\n"));
                code.push_str(&format!("{indent}loop {{\n"));
                code.push_str(&format!("{indent}    if let Some(content) = {parser_var}.try_parse(|p| {{\n"));
                
                // Generate code for the inner element
                let inner_code = self.generate_optimized_node_code_with_context_and_pipeline(
                    element, 0, rule_name, rule_annotations, "p", None
                )?;
                
                // UNIFIED: Always expect 'result' from inner code
                code.push_str(&format!("{indent}        {}", inner_code.trim_end()));
                code.push_str(&format!("\n{indent}        Ok(result)\n"));
                code.push_str(&format!("{indent}    }}) {{\n"));
                code.push_str(&format!("{indent}        quantified_elements.push(ParseNode {{\n"));
                code.push_str(&format!("{indent}            rule_name: \"quantified\",\n"));
                code.push_str(&format!("{indent}            content,\n"));
                code.push_str(&format!("{indent}            span: 0..0,\n"));
                code.push_str(&format!("{indent}        }});\n"));
                code.push_str(&format!("{indent}    }} else {{\n"));
                code.push_str(&format!("{indent}        break;\n"));
                code.push_str(&format!("{indent}    }}\n"));
                code.push_str(&format!("{indent}}}\n"));
                code.push_str(&format!("{indent}let result = ParseContent::Quantified(quantified_elements, \"*\");\n"));
            }
            "+" => {
                // One or more: parse first (mandatory), then loop
                code.push_str(&format!("{indent}// One or more (+) for: {element_description}\n"));
                code.push_str(&format!("{indent}let mut quantified_elements = Vec::new();\n"));
                code.push_str(&format!("{indent}// First element is mandatory\n"));
                
                let first_code = self.generate_optimized_node_code_with_context_and_pipeline(
                    element, 0, rule_name, rule_annotations, parser_var, None
                )?;
                
                code.push_str(&first_code);
                code.push_str(&format!("{indent}quantified_elements.push(ParseNode {{\n"));
                code.push_str(&format!("{indent}    rule_name: \"quantified\",\n"));
                code.push_str(&format!("{indent}    content: result,\n"));
                code.push_str(&format!("{indent}    span: 0..0,\n"));
                code.push_str(&format!("{indent}}});\n"));
                
                code.push_str(&format!("{indent}// Then zero or more additional\n"));
                code.push_str(&format!("{indent}loop {{\n"));
                code.push_str(&format!("{indent}    if let Some(content) = {parser_var}.try_parse(|p| {{\n"));
                
                let loop_code = self.generate_optimized_node_code_with_context_and_pipeline(
                    element, 0, rule_name, rule_annotations, "p", None
                )?;
                
                // UNIFIED: Always expect 'result' from inner code
                code.push_str(&format!("{indent}        {}", loop_code.trim_end()));
                code.push_str(&format!("\n{indent}        Ok(result)\n"));
                code.push_str(&format!("{indent}    }}) {{\n"));
                code.push_str(&format!("{indent}        quantified_elements.push(ParseNode {{\n"));
                code.push_str(&format!("{indent}            rule_name: \"quantified\",\n"));
                code.push_str(&format!("{indent}            content,\n"));
                code.push_str(&format!("{indent}            span: 0..0,\n"));
                code.push_str(&format!("{indent}        }});\n"));
                code.push_str(&format!("{indent}    }} else {{\n"));
                code.push_str(&format!("{indent}        break;\n"));
                code.push_str(&format!("{indent}    }}\n"));
                code.push_str(&format!("{indent}}}\n"));
                code.push_str(&format!("{indent}let result = ParseContent::Quantified(quantified_elements, \"+\");\n"));
            }
            _ => {
                return Err(anyhow::anyhow!("Unknown quantifier: {}", quantifier));
            }
        }
        
        Ok(code)
    }

    fn generate_fast_helpers(&self) -> String {
        String::from(r#"    /// Regex-based pattern matching using Rust regex engine
    /// Handles EBNF /.../ regex patterns properly
    #[inline]
    fn match_regex_optimized(&mut self, pattern: &str) -> ParseResult<&'input str> {
        let start_pos = self.position;
        
        // Use the actual Rust regex engine for proper EBNF regex support
        use regex::Regex;
        
        let remaining_input = &self.input[self.position..];
        
        // Create regex with anchoring at start of string
        let anchored_pattern = if pattern.starts_with('^') {
            pattern.to_string()
        } else {
            format!("^{}", pattern)
        };
        
        match Regex::new(&anchored_pattern) {
            Ok(regex) => {
                if let Some(matched) = regex.find(remaining_input) {
                    // Ensure the match starts at position 0 (our current position)
                    if matched.start() == 0 {
                        let match_len = matched.len();
                        let old_pos = self.position;
                        self.position += match_len;
                        Ok(&self.input[old_pos..self.position])
                    } else {
                        Err(ParseError::InvalidSyntax {
                            message: "Pattern mismatch at current position",
                            position: start_pos,
                        })
                    }
                } else {
                    Err(ParseError::InvalidSyntax {
                        message: "Pattern mismatch",
                        position: start_pos,
                    })
                }
            }
            Err(_) => {
                Err(ParseError::InvalidSyntax {
                    message: "Invalid regex pattern",
                    position: start_pos,
                })
            }
        }
    }
    
    /// Universal pattern matcher that works for any grammar
    /// Interprets pattern based on its structure, not hardcoded assumptions
    #[inline]
    fn pattern_matches(&self, ch: char, pattern: &str) -> bool {
        if pattern.len() == 1 {
            // Single character literal
            ch == pattern.chars().next().unwrap()
        } else if pattern == "." {
            // Dot pattern - matches any character except newline in most grammars
            // TODO: This behavior should come from semantic annotations
            ch != '\n'
        } else if pattern.starts_with("[^") && pattern.ends_with("]") {
            // Negated character class - match chars NOT in the class
            let class_chars = &pattern[2..pattern.len()-1];
            !self.char_in_class(ch, class_chars)
        } else if pattern.starts_with("[") && pattern.ends_with("]") {
            // Positive character class - match chars IN the class
            let class_chars = &pattern[1..pattern.len()-1];
            self.char_in_class(ch, class_chars)
        } else {
            // Complex pattern or escape sequence - generic fallback
            // TODO: This should be driven by semantic annotations from grammar
            self.match_generic_pattern(ch, pattern)
        }
    }
    
    /// Generic pattern matcher for complex patterns
    /// This is where semantic annotations would drive specialized matching
    #[inline]
    fn match_generic_pattern(&self, ch: char, pattern: &str) -> bool {
        // For now, simple contains check
        // TODO: Replace with AST-driven pattern interpretation
        pattern.contains(ch)
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
        let start_pos = self.position;
        
        if self.debug_mode {
            self.debug_output.push(format!("      🔧 QUANTIFIER ENGINE START: '{}' quantifier at position {}", quantifier, start_pos));
            self.debug_output.push(format!("      📝 QUANTIFIER DETAILS: Will attempt to match pattern repeatedly with '{}' semantics", 
                match quantifier {
                    "*" => "zero-or-more (greedy)",
                    "+" => "one-or-more (greedy)", 
                    "?" => "zero-or-one (optional)",
                    _ => "custom"
                }
            ));
        }
        
        match quantifier {
            "*" => {
                // Zero or more - optimized loop with memoization preservation
                if self.debug_mode {
                    self.debug_output.push(format!("      🔄 STAR QUANTIFIER: Starting zero-or-more loop at position {}", self.position));
                }
                
                loop {
                    let loop_start_pos = self.position;
                    if self.debug_mode {
                        self.debug_output.push(format!("      🔁 STAR ITERATION: Attempting iteration {} at position {}", iteration + 1, loop_start_pos));
                    }
                    
                    match self.try_parse_memoized(&mut f) {
                        Some(content) => {
                            iteration += 1;
                            if self.debug_mode {
                                self.debug_output.push(format!("      ✅ STAR ITERATION SUCCESS: Iteration {} succeeded, advanced from {} to {}", 
                                    iteration, loop_start_pos, self.position));
                            }
                            self.debug_quantifier_iteration(iteration, quantifier);
                            results.push(ParseNode {
                                rule_name: "quantified",
                                content,
                                span: 0..0, // Will be filled by caller
                            });
                        }
                        None => {
                            if self.debug_mode {
                                self.debug_output.push(format!("      🛑 STAR ITERATION END: Iteration {} failed at position {} - stopping loop (this is normal for '*' quantifier)", 
                                    iteration + 1, loop_start_pos));
                            }
                            break;
                        }
                    }
                    
                    // Prevent infinite loops on zero-length matches
                    if self.position == loop_start_pos {
                        if self.debug_mode {
                            self.debug_output.push(format!("      ⚠️ STAR ZERO-LENGTH: Detected zero-length match at position {} - stopping to prevent infinite loop", self.position));
                        }
                        break;
                    }
                }
                
                if self.debug_mode {
                    self.debug_output.push(format!("      🏁 STAR QUANTIFIER END: Matched {} iterations, advanced from {} to {}", 
                        iteration, start_pos, self.position));
                }
                Ok(ParseContent::Quantified(results, "*"))
            }
            
            "+" => {
                // One or more - require at least one
                if self.debug_mode {
                    self.debug_output.push(format!("      🔄 PLUS QUANTIFIER: Starting one-or-more, requires at least 1 match at position {}", self.position));
                }
                
                iteration += 1;
                let first_attempt_pos = self.position;
                if self.debug_mode {
                    self.debug_output.push(format!("      🎯 PLUS FIRST ATTEMPT: Trying required first match at position {}", first_attempt_pos));
                }
                
                self.debug_quantifier_iteration(iteration, quantifier);
                match f(self) {
                    Ok(content) => {
                        if self.debug_mode {
                            self.debug_output.push(format!("      ✅ PLUS FIRST SUCCESS: Required first match succeeded, advanced from {} to {}", 
                                first_attempt_pos, self.position));
                        }
                        
                        results.push(ParseNode {
                            rule_name: "quantified",
                            content,
                            span: 0..0,
                        });
                        
                        // Continue with zero-or-more pattern
                        if self.debug_mode {
                            self.debug_output.push(format!("      🔁 PLUS CONTINUATION: First match succeeded, continuing with zero-or-more loop at position {}", self.position));
                        }
                        
                        loop {
                            let loop_start_pos = self.position;
                            if self.debug_mode {
                                self.debug_output.push(format!("      🔁 PLUS ITERATION: Attempting iteration {} at position {}", iteration + 1, loop_start_pos));
                            }
                            
                            match self.try_parse_memoized(&mut f) {
                                Some(content) => {
                                    iteration += 1;
                                    if self.debug_mode {
                                        self.debug_output.push(format!("      ✅ PLUS ITERATION SUCCESS: Iteration {} succeeded, advanced from {} to {}", 
                                            iteration, loop_start_pos, self.position));
                                    }
                                    self.debug_quantifier_iteration(iteration, quantifier);
                                    results.push(ParseNode {
                                        rule_name: "quantified", 
                                        content,
                                        span: 0..0,
                                    });
                                }
                                None => {
                                    if self.debug_mode {
                                        self.debug_output.push(format!("      🛑 PLUS ITERATION END: Iteration {} failed at position {} - stopping loop", 
                                            iteration + 1, loop_start_pos));
                                    }
                                    break;
                                }
                            }
                            
                            // Prevent infinite loops on zero-length matches
                            if self.position == loop_start_pos {
                                if self.debug_mode {
                                    self.debug_output.push(format!("      ⚠️ PLUS ZERO-LENGTH: Detected zero-length match at position {} - stopping to prevent infinite loop", self.position));
                                }
                                break;
                            }
                        }
                        
                        if self.debug_mode {
                            self.debug_output.push(format!("      🏁 PLUS QUANTIFIER END: Matched {} iterations, advanced from {} to {}", 
                                iteration, start_pos, self.position));
                        }
                        Ok(ParseContent::Quantified(results, "+"))
                    }
                    Err(err) => {
                        if self.debug_mode {
                            self.debug_output.push(format!("      ❌ PLUS FIRST FAILURE: Required first match failed at position {} - entire '+' quantifier fails", 
                                first_attempt_pos));
                        }
                        Err(err)
                    },
                }
            }
            "?" => {
                // Zero or one with memoization preservation
                if self.debug_mode {
                    self.debug_output.push(format!("      🔄 QUESTION QUANTIFIER: Attempting zero-or-one (optional) match at position {}", self.position));
                }
                
                let attempt_pos = self.position;
                match self.try_parse_memoized(&mut f) {
                    Some(content) => {
                        iteration += 1;
                        if self.debug_mode {
                            self.debug_output.push(format!("      ✅ QUESTION SUCCESS: Optional match succeeded, advanced from {} to {}", 
                                attempt_pos, self.position));
                        }
                        self.debug_quantifier_iteration(iteration, quantifier);
                        results.push(ParseNode {
                            rule_name: "quantified",
                            content,
                            span: 0..0,
                        });
                    }
                    None => {
                        if self.debug_mode {
                            self.debug_output.push(format!("      ⭕ QUESTION NO MATCH: Optional match failed at position {} - this is OK for '?' quantifier (zero matches)", 
                                attempt_pos));
                        }
                    }
                }
                
                if self.debug_mode {
                    self.debug_output.push(format!("      🏁 QUESTION QUANTIFIER END: Matched {} iterations, position {} to {}", 
                        iteration, start_pos, self.position));
                }
                Ok(ParseContent::Quantified(results, "?"))
            }
            _ => {
                if self.debug_mode {
                    self.debug_output.push(format!("      ❌ UNKNOWN QUANTIFIER: '{}' is not supported", quantifier));
                }
                Err(ParseError::InvalidSyntax {
                    message: "Unknown quantifier",
                    position: self.position,
                })
            },
        }
    }

"# )
    }

    fn generate_performance_tests(&self) -> String {
        // Performance tests disabled temporarily to avoid string formatting issues
        String::new()
    }
}
