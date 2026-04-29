// AST-Based Parser Generator using syn and quote
// This module replaces string concatenation with proper AST manipulation
// GUARANTEES: No unbalanced braces, no syntax errors, type-safe code generation

use super::Logger;
use crate::ast_pipeline::{
    ASTNode, ASTValue, Annotations, BranchAnnotation, SemanticAnnotation, SemanticAssociativity,
    SemanticBranchPolicy, SemanticRuntimeDirective, SemanticRuntimeValue, SemanticScopeKind,
    SemanticTokenClass, SemanticValueConstraints, TokenValue, UnifiedReturnAST,
    UnifiedSemanticAST, UnifiedSemanticProperty, UnifiedSemanticValue,
    ast_return_transform::AstReturnTransformer, compile_semantic_runtime_annotations,
    extract_semantic_directive, normalize_semantic_scalar, parse_canonical_transform_expression,
    parse_semantic_bool, parse_semantic_branch_priorities, parse_semantic_charset,
    parse_semantic_constraint_expression, parse_semantic_coverage_target_weight,
    parse_semantic_deterministic_group, parse_semantic_group_label, parse_semantic_implication,
    parse_semantic_len_bounds, parse_semantic_nonnegative_usize, parse_semantic_numeric_bounds,
    parse_semantic_pattern, parse_semantic_reference_list, parse_semantic_string_list,
    parse_semantic_token_class,
};
use anyhow::Result;
use prettyplease;
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use std::collections::{HashMap, HashSet};
use syn::Ident;

// PCRE2 conformance includes legal regexes whose syntax is shallow in bytes but
// deep in generated parser calls. Keep this bounded, but above real corpus depth.
const GENERATED_RECURSION_GUARD_MAX_DEPTH: usize = 4096;

macro_rules! eprintln {
    ($($arg:tt)*) => {
        crate::pgen_trace_debug!($($arg)*)
    };
}

/// AST-based generator that produces guaranteed syntactically correct Rust code
pub struct AstBasedGenerator {
    pub grammar_name: String,
    pub entry_rule: Option<String>,
    pub logger: Option<Box<dyn Logger>>,
    pub annotations: Option<Annotations>,
    pub branch_return_annotations: HashMap<String, Vec<Option<BranchAnnotation>>>,
    pub enable_debug: bool,
    /// Phase 2 M1 toggle. When true, the generator emits `parse_full_<entry>_typed`
    /// returning `ParseResult<serde_json::Value>` alongside the existing
    /// `parse_full_<entry>` returning `ParseResult<ParseNode>`. The M1 typed method is
    /// a skeleton wrapper around the legacy method + `serde_json::to_value`; M2
    /// replaces the body with truly inline shape-emit logic per the rule's return
    /// annotation. Default false: generator emit is unchanged from prior behavior.
    pub inline_annotations: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct SemanticRelationalConstraintPolicy {
    constraint_expression: Option<String>,
    requires_references: Vec<String>,
    implication: Option<(String, String)>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct SemanticCoverageTargetPolicy {
    coverage_target_weight: u64,
    critical_path: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct SemanticNegativeCasePolicy {
    invalid_case: bool,
    negative: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct SemanticDeterminismPartitionPolicy {
    enabled: bool,
    group_label: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct SemanticTokenSteeringPolicy {
    token_class: Option<SemanticTokenClass>,
    charset_pattern: Option<String>,
    explicit_pattern: Option<String>,
}

impl AstBasedGenerator {
    /// PGEN-RGX-0073 Optim #14: true when this specific rule carries
    /// no semantic annotations of any kind (direct, branch-local, or
    /// mid-sequence). Such a rule's `with_semantic_runtime_rule_transaction`
    /// wrapper is dead weight — the runtime fast-path landed in Optim #11
    /// always short-circuits in this case, and Optim #13 already proved
    /// the elision is safe at the grammar level. This is the per-rule
    /// generalization: a partially-annotated grammar (e.g. `regex`,
    /// where `@semantic_value` lives on a small fraction of rules) can
    /// have its non-annotated rules elided too.
    ///
    /// Safety note: the per-rule elision is exactly equivalent to the
    /// runtime fast-path always firing for this rule. Children that
    /// carry annotations still run their own wrappers; the parent's
    /// wrapper is not load-bearing for child state because each child
    /// transaction commits/rolls back independently at its own wrapper.
    /// PGEN-RGX-0073 Optim #16: returns the set of rules that are
    /// transitively reachable from themselves through rule references.
    /// Rules NOT in this set never form a cycle, so the per-rule entry
    /// `recursion_guard.check_cycle` + `enter`/`exit` is dead weight,
    /// and `memoized_call` (which exists to give Packrat its
    /// linear-time guarantee under recursive sub-parses) cannot
    /// observe a cache hit either: a non-recursive rule never re-enters
    /// itself at the same position, and shared-sub-parse hits across
    /// distinct call paths are rare in PEG-style grammars and bounded
    /// by the rule's own complexity. Eliding both for the non-recursive
    /// set is parser-agnostic: any generated parser whose grammar has
    /// non-recursive rules picks up the same wins.
    fn compute_recursive_rules(grammar_tree: &HashMap<String, ASTNode>) -> HashSet<String> {
        let mut direct_calls: HashMap<&str, HashSet<String>> = HashMap::new();
        for (rule_name, body) in grammar_tree {
            let mut refs: HashSet<String> = HashSet::new();
            Self::collect_rule_references(body, &mut refs);
            direct_calls.insert(rule_name.as_str(), refs);
        }
        let mut recursive: HashSet<String> = HashSet::new();
        for rule_name in grammar_tree.keys() {
            let mut visited: HashSet<String> = HashSet::new();
            let mut stack: Vec<String> = direct_calls
                .get(rule_name.as_str())
                .into_iter()
                .flatten()
                .cloned()
                .collect();
            while let Some(child) = stack.pop() {
                if child.as_str() == rule_name.as_str() {
                    recursive.insert(rule_name.clone());
                    break;
                }
                if visited.insert(child.clone()) {
                    if let Some(grandchildren) = direct_calls.get(child.as_str()) {
                        stack.extend(grandchildren.iter().cloned());
                    }
                }
            }
        }
        recursive
    }

    fn rule_has_no_semantic_annotations(&self, rule_name: &str) -> bool {
        let Some(annotations) = &self.annotations else {
            return true;
        };
        let direct_empty = annotations
            .semantic_annotations
            .get(rule_name)
            .is_none_or(|v| v.is_empty());
        let branch_empty = annotations
            .branch_semantic_annotations
            .get(rule_name)
            .is_none_or(|branches| branches.iter().all(|b| b.is_empty()));
        let mid_seq_empty = annotations
            .branch_mid_sequence_semantic_annotations
            .get(rule_name)
            .is_none_or(|branches| branches.iter().all(|b| b.is_empty()));
        direct_empty && branch_empty && mid_seq_empty
    }

    pub fn new(grammar_name: String) -> Self {
        Self {
            grammar_name,
            entry_rule: None,
            logger: None,
            annotations: None,
            branch_return_annotations: HashMap::new(),
            enable_debug: true,
            inline_annotations: false,
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
        let syntax_tree: syn::File = match syn::parse2(parser_tokens.clone()) {
            Ok(file) => file,
            Err(err) => {
                let dump_path = format!("{filename}.tokens_dump.rs");
                let rendered = parser_tokens.to_string();
                let _ = std::fs::write(&dump_path, &rendered);
                // Try to localize the failure by binary-searching token chunks
                // for syn-parse acceptance. The largest accepted prefix's
                // boundary is a strong hint at where the broken token sits.
                let approx_byte = locate_syn_parse_boundary(&rendered);
                let context_window =
                    render_token_context(&rendered, approx_byte, 200);
                return Err(anyhow::anyhow!(
                    "Failed to parse generated TokenStream: {} (dumped raw tokens to {})\n  approximate failure byte: {}\n  context: {}",
                    err,
                    dump_path,
                    approx_byte,
                    context_window
                ));
            }
        };
        let formatted_code = prettyplease::unparse(&syntax_tree);
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

        // Phase 2 M1+M3-stage-1: parallel typed parser impl, only emitted when
        // --inline-annotations is set. Emits a per-rule `parse_<rule>_typed`
        // method for every rule in `rule_order` plus the `parse_full_<entry>_typed`
        // entry-point that dispatches through the per-rule typed entry. Stage 1
        // bodies wrap the legacy parse + serde_json::to_value (no perf gain yet,
        // but the API surface is stable). Future stages replace per-shape bodies
        // with shape-typed emit and `ParseNode` is bypassed entirely.
        let typed_parser_impl = if self.inline_annotations {
            self.generate_typed_parser_impl_skeleton(
                &parser_name,
                grammar_tree,
                rule_order,
                &entry_rule,
            )
        } else {
            TokenStream::new()
        };

        // Combine everything
        let result = quote! {
            #imports
            #types
            #parser_struct
            #parser_impl
            #typed_parser_impl
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
            #[derive(Debug, Clone, PartialEq, Eq)]
            pub enum RecoveryMarkerKind {
                PanicUntil,
                Sync,
                EofFallback,
            }

            #[derive(Debug, Clone, PartialEq, Eq)]
            pub struct RecoveryEvent {
                pub rule_name: String,
                pub parse_start: usize,
                pub previous_position: usize,
                pub new_position: usize,
                pub marker_kind: RecoveryMarkerKind,
                pub marker_position: Option<usize>,
                pub marker_value: Option<String>,
            }

            #[derive(Debug, Clone, PartialEq, Eq)]
            pub struct CoverageTargetEvent {
                pub rule_name: String,
                pub parse_start: usize,
                pub parse_end: usize,
                pub branch_index: Option<usize>,
                pub coverage_target_weight: u64,
                pub critical_path: bool,
            }

            #[derive(Debug, Clone, PartialEq, Eq)]
            pub struct NegativeCaseEvent {
                pub rule_name: String,
                pub parse_start: usize,
                pub failure_position: usize,
                pub negative: bool,
                pub error_kind: String,
            }

            #[derive(Debug, Clone, PartialEq, Eq)]
            pub struct DeterministicPartitionEvent {
                pub rule_name: String,
                pub parse_start: usize,
                pub parse_end: usize,
                pub group_key: String,
            }

            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            pub enum DeterministicPartitionRuntimeMode {
                AnnotationDriven,
                ForceEnabled,
                ForceDisabled,
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
                // Optim #6: FxHashMap (rustc-hash) for the memo. Hit on every rule
                // entry, with internal (RuleId, usize) integer keys — no DoS exposure,
                // siphash is overkill. FxHash is the same fast hasher rustc uses
                // internally; ~3-4× faster on small integer keys.
                memo: rustc_hash::FxHashMap<(RuleId, usize), MemoEntry<'input>>,
                recursion_guard: RecursionGuard,
                grammar_profile: Option<String>,
                recovery_events: Vec<RecoveryEvent>,
                recovery_counts: HashMap<String, usize>,
                recovery_parse_count: usize,
                recovery_global_count: usize,
                coverage_target_events: Vec<CoverageTargetEvent>,
                coverage_target_rule_hits: HashMap<String, usize>,
                coverage_target_branch_hits: HashMap<String, usize>,
                negative_case_events: Vec<NegativeCaseEvent>,
                negative_case_rule_hits: HashMap<String, usize>,
                deterministic_partition_events: Vec<DeterministicPartitionEvent>,
                deterministic_partition_rule_hits: HashMap<String, usize>,
                deterministic_partition_runtime_mode: DeterministicPartitionRuntimeMode,
                semantic_runtime_annotations: crate::ast_pipeline::CompiledSemanticRuntimeAnnotations,
                semantic_runtime_state: crate::ast_pipeline::SemanticRuntimeState,
                logger: Box<dyn Logger>,
                // Optim #5: cache logger.is_enabled() at construction so the parser hot
                // path skips the per-call vtable dispatch through Box<dyn Logger>. Logger
                // is set once at new() and not swapped at runtime, so the cache is sound.
                logger_enabled: bool,
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
        let constructor = self.generate_constructor()?;
        let parse_method = self.generate_parse_method(entry_rule);

        // Generate rule methods
        eprintln!("\n{}", "-".repeat(60));
        eprintln!("RULE METHOD GENERATION");
        eprintln!("{}", "-".repeat(60));
        let recursive_rules = Self::compute_recursive_rules(grammar_tree);
        let mut rule_methods = Vec::new();
        for rule_name in rule_order {
            eprintln!("   📋  Rule: {} - File: {}:{}", rule_name, file!(), line!());
            if let Some(ast_node) = grammar_tree.get(rule_name) {
                let is_recursive = recursive_rules.contains(rule_name.as_str());
                let method = self.generate_rule_method_with_recursion(
                    rule_name,
                    ast_node,
                    rule_order,
                    filename,
                    is_recursive,
                )?;
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
            ASTNode::Lookahead { element, .. } => {
                Self::collect_rule_references(element, out);
            }
            ASTNode::Atom { value } => {
                if let ASTValue::Token(parts) = value {
                    if parts.len() >= 2 {
                        if let (TokenValue::String(token_type), TokenValue::String(token_value)) =
                            (&parts[0], &parts[1])
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

    fn generate_constructor(&self) -> Result<TokenStream> {
        let compiled_semantic_runtime_annotations =
            self.generate_compiled_semantic_runtime_annotations_tokens()?;
        let recursion_guard_max_depth = GENERATED_RECURSION_GUARD_MAX_DEPTH;

        Ok(quote! {
            pub fn new(input: &'input str, logger: Box<dyn Logger>) -> Self {
                let logger_enabled = logger.is_enabled();
                Self {
                    input,
                    position: 0,
                    // Optim #7: pre-size memo to skip the 4-→8-→16-→...-→256 rehash
                    // chain during parse. Even small regex patterns produce ~50-200
                    // memo entries; large ones produce more. 256 covers the common
                    // case without growth; FxHashMap doubles past that. The cost is
                    // a one-time allocation at parser construction (cheap).
                    memo: rustc_hash::FxHashMap::with_capacity_and_hasher(256, Default::default()),
                    recursion_guard: RecursionGuard::new(#recursion_guard_max_depth),
                    grammar_profile: None,
                    recovery_events: Vec::new(),
                    recovery_counts: HashMap::new(),
                    recovery_parse_count: 0,
                    recovery_global_count: 0,
                    coverage_target_events: Vec::new(),
                    coverage_target_rule_hits: HashMap::new(),
                    coverage_target_branch_hits: HashMap::new(),
                    negative_case_events: Vec::new(),
                    negative_case_rule_hits: HashMap::new(),
                    deterministic_partition_events: Vec::new(),
                    deterministic_partition_rule_hits: HashMap::new(),
                    deterministic_partition_runtime_mode: DeterministicPartitionRuntimeMode::AnnotationDriven,
                    semantic_runtime_annotations: #compiled_semantic_runtime_annotations,
                    semantic_runtime_state: crate::ast_pipeline::SemanticRuntimeState::new(),
                    logger,
                    logger_enabled,
                }
            }
        })
    }

    /// Phase 2 M3 stage 2: shape-typed body emit per rule AST node.
    ///
    /// Returns `Some(body)` when the typed body for this rule has been
    /// implemented at codegen — currently `ASTNode::Atom` for the three
    /// common subtypes (`quoted_string`, `regex`, `rule_reference`) when
    /// the rule has no semantic / return annotations that change the
    /// emitted shape. Returns `None` when no shape-typed emit is
    /// available yet (annotated rules; `Sequence` / `Or` / `Quantified`
    /// / `Lookahead` shapes pending stages 3-5); the caller falls back
    /// to the stage-1 wrapper.
    ///
    /// Stage 2 contract: shape-typed body produces the matched text as
    /// `Value::String` for leaves, or dispatches to the inner rule's
    /// typed entry for rule references. `ParseNode` is bypassed
    /// entirely on this path.
    fn generate_typed_node_body(
        &self,
        ast_node: &ASTNode,
        rule_name: &str,
    ) -> Option<TokenStream> {
        // Phase 2 M3 stage 4c: semantic annotations (`@semantic_value`,
        // `@predicate`, `@emit_fact`, etc.) are runtime side-effects.
        // `@semantic_value` computes a value used for predicates / fact
        // emission elsewhere; `@predicate` and `@emit_fact` interact
        // with the parser's runtime state via the
        // `with_semantic_runtime_rule_transaction` wrapper. None of
        // them affect the rule's AST output shape — the parsed content
        // is the same regardless of whether semantic annotations are
        // present. The typed body produces the AST shape (matching
        // `serde_json::to_value(legacy)`); semantic side-effects are
        // owned by the legacy path and remain there.
        //
        // This means the typed path is *unsafe* for grammars whose
        // parsing correctness depends on semantic annotations
        // (predicates that gate parsing, facts referenced by sibling
        // rules). For the regex grammar, the `@semantic_value`
        // annotations are pure metadata for downstream tooling, not
        // gating predicates, so the typed AST shape is correct without
        // them. Grammars that DO depend on semantic predicates (SV,
        // VHDL) keep using the legacy path by default; the typed path
        // is opt-in via `--inline-annotations`, and adopters know what
        // they're trading.
        // Stage 4b: rules with explicit return annotations
        // (`return_object`, `return_array`, `return_scalar`) get a
        // typed body that applies the annotation transform directly to
        // typed sub-values, no `ParseNode` round-trip. The
        // implementation supports a bounded subset of annotation shapes
        // (PositionalRef + literals + Object/Array nesting) over a
        // bounded subset of rule body shapes (Sequence / single Atom /
        // single ?-Quantified Atom). Cases outside that subset return
        // `None` and fall back to the stage-1 wrapper.
        let has_explicit_return_annotation = self
            .annotations
            .as_ref()
            .and_then(|a| a.branch_return_annotations.get(rule_name))
            .is_some_and(|branches| branches.iter().any(Option::is_some));
        if has_explicit_return_annotation {
            return self.generate_typed_annotated_body(ast_node, rule_name);
        }

        // Phase 2 M3 stage 5b: route non-annotated rules through the
        // generic `generate_typed_value_expr` composer. The composer
        // handles all five `ASTNode` shapes and recursively composes
        // for nested non-Atom inners — the per-shape body emitters
        // from stages 2-5 are now subsumed. `Lookahead` returns `None`
        // from the composer and falls back to the stage-1 wrapper.
        let expr = self.generate_typed_value_expr(ast_node, rule_name, quote! { self })?;
        Some(quote! { Ok(#expr) })
    }

    /// Phase 2 M3 stage 5b: generic shape composer. Returns a
    /// `serde_json::Value`-producing expression for any non-annotated
    /// `ASTNode` shape, recursing into nested shapes. The `receiver`
    /// parameter is the parser receiver used inside the expression
    /// (`self` at top-level method bodies; `parser` inside `try_parse`
    /// closures, matching the legacy `let parser = p;` shadowing
    /// convention).
    ///
    /// This unifies the per-shape stages 2-5 emitters and unlocks
    /// composite shapes (Quantified-* over Sequence, Or with Sequence
    /// alternatives, Sequence with non-Atom Quantified children) by
    /// delegating to itself for inner shapes. `Lookahead` returns
    /// `None` for now — it's rare at the rule-body level and needs
    /// separate semantics.
    fn generate_typed_value_expr(
        &self,
        ast_node: &ASTNode,
        rule_name: &str,
        receiver: TokenStream,
    ) -> Option<TokenStream> {
        match ast_node {
            ASTNode::Atom { value } => {
                self.generate_typed_atom_value_expr(value, rule_name, receiver)
            }
            ASTNode::Sequence { elements } => {
                self.generate_typed_sequence_value_expr_v2(elements, rule_name, receiver)
            }
            ASTNode::Or { alternatives } => {
                self.generate_typed_or_value_expr(alternatives, rule_name, receiver)
            }
            ASTNode::Quantified { element, quantifier } => self
                .generate_typed_quantified_value_expr(
                    element,
                    quantifier,
                    rule_name,
                    receiver,
                ),
            ASTNode::Lookahead { .. } => None,
        }
    }

    /// Stage 5b helper. Sequence value-expression emitter that
    /// recurses into the composer for non-Atom children. Produces a
    /// block expression yielding `Value::Array(child_typed_values)`.
    /// Optional `?-Quantified` children are wrapped in `try_parse` and
    /// produce `Value::Null` on miss; required children short-circuit
    /// via `?` if their typed expression fails.
    fn generate_typed_sequence_value_expr_v2(
        &self,
        elements: &[ASTNode],
        rule_name: &str,
        receiver: TokenStream,
    ) -> Option<TokenStream> {
        let count = elements.len();
        let mut pushes: Vec<TokenStream> = Vec::with_capacity(count);
        for elem in elements {
            let push = match elem {
                ASTNode::Quantified { element: inner, quantifier } if quantifier == "?" => {
                    let inner_expr =
                        self.generate_typed_value_expr(inner, rule_name, quote! { parser })?;
                    quote! {
                        let __pgen_oe = if let Some(v) = #receiver.try_parse(|p| {
                            let parser = p;
                            Ok::<serde_json::Value, ParseError>(#inner_expr)
                        }) {
                            v
                        } else {
                            serde_json::Value::Null
                        };
                        __pgen_seq_elements.push(__pgen_oe);
                    }
                }
                _ => {
                    let elem_expr =
                        self.generate_typed_value_expr(elem, rule_name, receiver.clone())?;
                    quote! {
                        __pgen_seq_elements.push(#elem_expr);
                    }
                }
            };
            pushes.push(push);
        }
        Some(quote! {
            {
                let mut __pgen_seq_elements: Vec<serde_json::Value> =
                    Vec::with_capacity(#count);
                #(#pushes)*
                serde_json::Value::Array(__pgen_seq_elements)
            }
        })
    }

    /// Stage 5b helper. Or value-expression emitter. Tries each
    /// alternative inside `try_parse(|p| { let parser = p; … })`; the
    /// first alternative whose typed expression succeeds yields its
    /// value via `break`. If every alternative fails, the expression
    /// returns `ParseError::Backtrack { position }` from the enclosing
    /// function/closure — that's the same semantics PEG alternation
    /// has when no branch matches.
    fn generate_typed_or_value_expr(
        &self,
        alternatives: &[ASTNode],
        rule_name: &str,
        receiver: TokenStream,
    ) -> Option<TokenStream> {
        let mut tries: Vec<TokenStream> = Vec::with_capacity(alternatives.len());
        for alt in alternatives {
            let alt_expr =
                self.generate_typed_value_expr(alt, rule_name, quote! { parser })?;
            tries.push(quote! {
                if let Some(__pgen_av) = #receiver.try_parse(|p| {
                    let parser = p;
                    Ok::<serde_json::Value, ParseError>(#alt_expr)
                }) {
                    break 'pgen_or_block __pgen_av;
                }
            });
        }
        Some(quote! {
            'pgen_or_block: {
                #(#tries)*
                return Err(ParseError::Backtrack {
                    position: #receiver.position,
                });
            }
        })
    }

    /// Stage 5b helper. Quantified value-expression emitter for `?`,
    /// `*`, `+` (over any inner shape via the composer). `{n,m}` and
    /// other quantifier forms return `None`.
    fn generate_typed_quantified_value_expr(
        &self,
        element: &ASTNode,
        quantifier: &str,
        rule_name: &str,
        receiver: TokenStream,
    ) -> Option<TokenStream> {
        match quantifier {
            "?" => {
                let inner =
                    self.generate_typed_value_expr(element, rule_name, quote! { parser })?;
                Some(quote! {
                    if let Some(__pgen_qv) = #receiver.try_parse(|p| {
                        let parser = p;
                        Ok::<serde_json::Value, ParseError>(#inner)
                    }) {
                        __pgen_qv
                    } else {
                        serde_json::Value::Null
                    }
                })
            }
            "*" => {
                let inner =
                    self.generate_typed_value_expr(element, rule_name, quote! { parser })?;
                Some(quote! {
                    {
                        let mut __pgen_qe: Vec<serde_json::Value> = Vec::new();
                        while let Some(v) = #receiver.try_parse(|p| {
                            let parser = p;
                            Ok::<serde_json::Value, ParseError>(#inner)
                        }) {
                            __pgen_qe.push(v);
                        }
                        serde_json::Value::Array(__pgen_qe)
                    }
                })
            }
            "+" => {
                let inner_first =
                    self.generate_typed_value_expr(element, rule_name, receiver.clone())?;
                let inner_loop =
                    self.generate_typed_value_expr(element, rule_name, quote! { parser })?;
                Some(quote! {
                    {
                        let mut __pgen_qe: Vec<serde_json::Value> = Vec::new();
                        __pgen_qe.push(#inner_first);
                        while let Some(v) = #receiver.try_parse(|p| {
                            let parser = p;
                            Ok::<serde_json::Value, ParseError>(#inner_loop)
                        }) {
                            __pgen_qe.push(v);
                        }
                        serde_json::Value::Array(__pgen_qe)
                    }
                })
            }
            _ => None,
        }
    }

    /// Phase 2 M3 stage 4b: shape-typed body for rules with explicit
    /// return annotations (`return_scalar`, `return_object`,
    /// `return_array`).
    ///
    /// Two-phase emit:
    /// 1. Parse phase. Walk the rule body, parsing each top-level
    ///    element and storing its typed value in a numbered local
    ///    `__pgen_v<i>` (1-based, matching the `$N` positional refs in
    ///    annotations). Failures during parse propagate via `?`.
    /// 2. Build phase. Walk the parsed annotation AST and emit
    ///    `serde_json::Value`-building code that references the
    ///    locals by index.
    ///
    /// Body shape support: `ASTNode::Sequence` (N elements at the
    /// top), `ASTNode::Atom` (1 element), `ASTNode::Quantified` (1
    /// element with the inner shape in $1 — `?` produces value or
    /// `Value::Null`, `*`/`+` produce `Value::Array`). Or rules
    /// (per-branch annotations) and Lookahead deferred.
    ///
    /// Annotation shape support: `PositionalRef`, `StringLiteral`,
    /// `NumberLiteral`, `BooleanLiteral`, `Object` (with values that
    /// recursively support these shapes), `Array`, and `Passthrough`
    /// (treated as `$1`). PropertyAccess / ArrayAccess / Spread /
    /// QuantifiedExtraction return `None` — those cases fall back to
    /// the stage-1 wrapper until per-shape typed emit is added.
    fn generate_typed_annotated_body(
        &self,
        ast_node: &ASTNode,
        rule_name: &str,
    ) -> Option<TokenStream> {
        // For Or rules, every branch may carry its own annotation; the
        // typed emit needs per-branch dispatch. Defer.
        if matches!(ast_node, ASTNode::Or { .. }) {
            return None;
        }

        // Resolve the rule's branch-0 annotation (Or rules are filtered
        // above; non-Or rules have a single conceptual branch).
        let branch_annotation = self
            .annotations
            .as_ref()?
            .branch_return_annotations
            .get(rule_name)
            .and_then(|branches| branches.first().cloned())
            .flatten()?;
        let parsed_ast = branch_annotation.parsed_ast.as_ref()?;

        // Determine top-level matched elements for $N indexing and
        // emit the parse-phase code that stores each element's typed
        // value in `__pgen_v<i>`.
        let elements: Vec<&ASTNode> = match ast_node {
            ASTNode::Sequence { elements } => elements.iter().collect(),
            ASTNode::Atom { .. } => vec![ast_node],
            ASTNode::Quantified { .. } => vec![ast_node],
            ASTNode::Or { .. } => return None,
            ASTNode::Lookahead { .. } => return None,
        };

        let mut parse_phase: Vec<TokenStream> = Vec::with_capacity(elements.len());
        for (idx, elem) in elements.iter().enumerate() {
            let v_ident = format_ident!("__pgen_v{}", idx + 1);
            let elem_expr = self.generate_typed_annotated_element_expr(elem, rule_name)?;
            parse_phase.push(quote! {
                let #v_ident: serde_json::Value = #elem_expr;
            });
        }

        let value_expr =
            self.generate_typed_annotation_value_expr(parsed_ast, elements.len())?;

        Some(quote! {
            #(#parse_phase)*
            Ok(#value_expr)
        })
    }

    /// Per-element typed expression for stage 4b parse phase. Returns
    /// an expression of type `serde_json::Value` (or `?`-propagating
    /// to `ParseError`). Each element corresponds to one positional
    /// `$N` slot. Stage 5b: dispatches through the generic
    /// `generate_typed_value_expr` composer, so any inner shape (Atom,
    /// Sequence, Or, Quantified) is handled.
    fn generate_typed_annotated_element_expr(
        &self,
        element: &ASTNode,
        rule_name: &str,
    ) -> Option<TokenStream> {
        self.generate_typed_value_expr(element, rule_name, quote! { self })
    }

    /// Stage 4b build-phase emitter. Walks the parsed return-annotation
    /// AST and emits a `serde_json::Value`-producing expression that
    /// references `__pgen_v<i>` locals built in the parse phase.
    fn generate_typed_annotation_value_expr(
        &self,
        ast: &UnifiedReturnAST,
        num_elements: usize,
    ) -> Option<TokenStream> {
        match ast {
            UnifiedReturnAST::PositionalRef { index } => {
                if *index < 1 || *index > num_elements {
                    return None;
                }
                let v_ident = format_ident!("__pgen_v{}", index);
                Some(quote! { #v_ident.clone() })
            }
            UnifiedReturnAST::StringLiteral { value } => Some(quote! {
                serde_json::Value::String(#value.to_string())
            }),
            UnifiedReturnAST::NumberLiteral { value } => {
                let lit = proc_macro2::Literal::f64_unsuffixed(*value);
                Some(quote! {
                    serde_json::Value::Number(
                        serde_json::Number::from_f64(#lit)
                            .unwrap_or_else(|| serde_json::Number::from(0))
                    )
                })
            }
            UnifiedReturnAST::BooleanLiteral { value } => {
                let v = *value;
                Some(quote! { serde_json::Value::Bool(#v) })
            }
            UnifiedReturnAST::Object { properties } => {
                // Sort keys for deterministic emit order.
                let mut keys: Vec<&String> = properties.keys().collect();
                keys.sort();
                let mut inserts: Vec<TokenStream> = Vec::with_capacity(keys.len());
                for key in keys {
                    let value_ast = properties.get(key)?;
                    let value_expr =
                        self.generate_typed_annotation_value_expr(value_ast, num_elements)?;
                    inserts.push(quote! {
                        __pgen_obj.insert(#key.to_string(), #value_expr);
                    });
                }
                Some(quote! {
                    {
                        let mut __pgen_obj = serde_json::Map::new();
                        #(#inserts)*
                        serde_json::Value::Object(__pgen_obj)
                    }
                })
            }
            UnifiedReturnAST::Array { elements } => {
                let element_exprs: Option<Vec<TokenStream>> = elements
                    .iter()
                    .map(|e| self.generate_typed_annotation_value_expr(e, num_elements))
                    .collect();
                let element_exprs = element_exprs?;
                Some(quote! {
                    serde_json::Value::Array(vec![#(#element_exprs),*])
                })
            }
            UnifiedReturnAST::Passthrough => {
                if num_elements < 1 {
                    return None;
                }
                Some(quote! { __pgen_v1.clone() })
            }
            // PropertyAccess, ArrayAccess, QuantifiedExtraction, Spread,
            // Identifier — defer.
            _ => None,
        }
    }

    /// Phase 2 M3 stage 5: shape-typed body for `ASTNode::Quantified`.
    ///
    /// Three quantifiers handled at this stage, each requiring the
    /// inner element to be `ASTNode::Atom`:
    /// - `?` — optional. Returns the typed value on success, `Value::Null` on miss.
    /// - `*` — zero-or-more. Loops `try_parse` until it fails, returns
    ///   `Value::Array(values)`.
    /// - `+` — one-or-more. Requires a first match (no `try_parse`), then
    ///   loops `try_parse` for further matches; returns `Value::Array(values)`.
    ///
    /// Returns `None` for `{n,m}` (bounded count, deferred) and for non-Atom
    /// inner elements (nested `Sequence` / `Or` / etc., deferred to a later
    /// slice).
    fn generate_typed_quantified_body(
        &self,
        element: &ASTNode,
        quantifier: &str,
        rule_name: &str,
    ) -> Option<TokenStream> {
        let ASTNode::Atom { value } = element else {
            return None;
        };
        match quantifier {
            "?" => {
                let expr_parser = self.generate_typed_atom_value_expr(
                    value,
                    rule_name,
                    quote! { parser },
                )?;
                Some(quote! {
                    if let Some(__pgen_q_v) = self.try_parse(|p| {
                        let parser = p;
                        Ok::<serde_json::Value, ParseError>(#expr_parser)
                    }) {
                        Ok(__pgen_q_v)
                    } else {
                        Ok(serde_json::Value::Null)
                    }
                })
            }
            "*" => {
                let expr_parser = self.generate_typed_atom_value_expr(
                    value,
                    rule_name,
                    quote! { parser },
                )?;
                Some(quote! {
                    let mut __pgen_q_elements: Vec<serde_json::Value> = Vec::new();
                    while let Some(__pgen_q_v) = self.try_parse(|p| {
                        let parser = p;
                        Ok::<serde_json::Value, ParseError>(#expr_parser)
                    }) {
                        __pgen_q_elements.push(__pgen_q_v);
                    }
                    Ok(serde_json::Value::Array(__pgen_q_elements))
                })
            }
            "+" => {
                let expr_self = self.generate_typed_atom_value_expr(
                    value,
                    rule_name,
                    quote! { self },
                )?;
                let expr_parser = self.generate_typed_atom_value_expr(
                    value,
                    rule_name,
                    quote! { parser },
                )?;
                Some(quote! {
                    let mut __pgen_q_elements: Vec<serde_json::Value> = Vec::new();
                    let __pgen_q_first = #expr_self;
                    __pgen_q_elements.push(__pgen_q_first);
                    while let Some(__pgen_q_v) = self.try_parse(|p| {
                        let parser = p;
                        Ok::<serde_json::Value, ParseError>(#expr_parser)
                    }) {
                        __pgen_q_elements.push(__pgen_q_v);
                    }
                    Ok(serde_json::Value::Array(__pgen_q_elements))
                })
            }
            _ => None, // {n,m} and other forms deferred
        }
    }

    /// Phase 2 M3 stage 4: shape-typed body for `ASTNode::Or`.
    ///
    /// Tries each alternative in order via `try_parse`; the first
    /// alternative that succeeds returns its typed value. If every
    /// alternative fails, the typed body returns
    /// `ParseError::Backtrack { position }` so the caller's outer Or
    /// or sequence can retry as needed.
    ///
    /// Stage 4 handles alternatives that are themselves `ASTNode::Atom`
    /// (any subtype the stage-2 atom emit handles). Returns `None`
    /// (stage-1 fallback) if any alternative is a more complex shape;
    /// nested `Sequence` / `Or` / `Quantified` / `Lookahead`
    /// alternatives are deferred to a later slice. Annotated rules
    /// were already filtered out at `generate_typed_node_body` because
    /// per-branch return-annotation transforms need their own
    /// implementation pass before they land on the typed path.
    fn generate_typed_or_body(
        &self,
        alternatives: &[ASTNode],
        rule_name: &str,
    ) -> Option<TokenStream> {
        let mut try_blocks: Vec<TokenStream> = Vec::with_capacity(alternatives.len());
        for alt in alternatives {
            let ASTNode::Atom { value } = alt else {
                return None;
            };
            let expr = self.generate_typed_atom_value_expr(
                value,
                rule_name,
                quote! { parser },
            )?;
            try_blocks.push(quote! {
                if let Some(__pgen_or_v) = self.try_parse(|p| {
                    let parser = p;
                    Ok::<serde_json::Value, ParseError>(#expr)
                }) {
                    return Ok(__pgen_or_v);
                }
            });
        }
        Some(quote! {
            #(#try_blocks)*
            Err(ParseError::Backtrack { position: self.position })
        })
    }

    /// Phase 2 M3 stage 2: shape-typed body for `ASTNode::Atom`.
    ///
    /// Three subtypes:
    /// - `quoted_string`: emit `match_string` and return the literal
    ///   text as `Value::String`.
    /// - `regex`: emit `match_regex` and return the matched text as
    ///   `Value::String`.
    /// - `rule_reference`: dispatch to the inner rule's typed entry.
    ///
    /// Returns `None` for unrecognized atom subtypes (literal_keyword,
    /// dpi_spec_string, etc.) so the stage-1 wrapper handles them.
    fn generate_typed_atom_body(
        &self,
        value: &ASTValue,
        rule_name: &str,
    ) -> Option<TokenStream> {
        let expr = self.generate_typed_atom_value_expr(value, rule_name, quote! { self })?;
        Some(quote! { Ok(#expr) })
    }

    /// Phase 2 M3 stage 2/3: shape-typed *expression* (not statement
    /// body) for `ASTNode::Atom`. Returns a block expression that
    /// produces `serde_json::Value` (or short-circuits via `?`). The
    /// `receiver` parameter is the parser receiver to use — `self` at
    /// top-level method bodies, or `parser` inside `try_parse` /
    /// nested closures (consistent with the legacy emit's `let parser
    /// = p;` shadowing convention).
    fn generate_typed_atom_value_expr(
        &self,
        value: &ASTValue,
        rule_name: &str,
        receiver: TokenStream,
    ) -> Option<TokenStream> {
        let ASTValue::Token(parts) = value else {
            return None;
        };
        if parts.len() < 2 {
            return None;
        }
        let TokenValue::String(token_type) = &parts[0] else {
            return None;
        };
        let TokenValue::String(token_value) = &parts[1] else {
            return None;
        };

        match token_type.as_str() {
            "quoted_string" => Some(quote! {
                {
                    let __pgen_typed_matched = #receiver.match_string(#token_value)?;
                    serde_json::Value::String(__pgen_typed_matched.to_string())
                }
            }),
            "regex" => {
                let skip_leading_whitespace = !matches!(
                    rule_name,
                    "string_content_double" | "string_content_single"
                );
                let effective_pattern = self.effective_regex_pattern(rule_name, token_value);
                Some(quote! {
                    {
                        let __pgen_typed_matched = #receiver.match_regex(
                            #effective_pattern,
                            #skip_leading_whitespace,
                        )?;
                        serde_json::Value::String(__pgen_typed_matched.to_string())
                    }
                })
            }
            "rule_reference" => {
                let inner_typed = format_ident!("parse_{}_typed", token_value);
                Some(quote! {
                    #receiver.#inner_typed()?
                })
            }
            _ => None,
        }
    }

    /// Phase 2 M3 stage 3: shape-typed body for `ASTNode::Sequence`.
    ///
    /// Builds `Value::Array(child_typed_values)` by parsing each child
    /// in order. Stage 3 handles the two most common child shapes:
    /// - `ASTNode::Atom` (any subtype handled by
    ///   `generate_typed_atom_value_expr`): inline the typed atom
    ///   expression and push the resulting `Value`.
    /// - `ASTNode::Quantified` with quantifier `?` whose inner element
    ///   is an `Atom`: wrap in `try_parse` and push the typed value or
    ///   `Value::Null` on miss.
    ///
    /// Returns `None` if any child is a shape stage 3 doesn't yet
    /// handle (nested `Or` / `Sequence` / `Lookahead` / `Quantified`
    /// with non-`?` quantifier or non-`Atom` inner). In that case the
    /// stage-1 fallback handles the whole rule.
    fn generate_typed_sequence_body(
        &self,
        elements: &[ASTNode],
        rule_name: &str,
    ) -> Option<TokenStream> {
        let element_count = elements.len();
        let mut element_pushes: Vec<TokenStream> = Vec::with_capacity(element_count);
        for elem in elements {
            let push = match elem {
                ASTNode::Atom { value } => {
                    let expr =
                        self.generate_typed_atom_value_expr(value, rule_name, quote! { self })?;
                    quote! {
                        elements.push(#expr);
                    }
                }
                ASTNode::Quantified { element, quantifier } if quantifier == "?" => {
                    let ASTNode::Atom { value } = element.as_ref() else {
                        return None;
                    };
                    let inner_expr = self.generate_typed_atom_value_expr(
                        value,
                        rule_name,
                        quote! { parser },
                    )?;
                    quote! {
                        let __pgen_optional_value = if let Some(__pgen_v) =
                            self.try_parse(|p| {
                                let parser = p;
                                Ok::<serde_json::Value, ParseError>(#inner_expr)
                            })
                        {
                            __pgen_v
                        } else {
                            serde_json::Value::Null
                        };
                        elements.push(__pgen_optional_value);
                    }
                }
                _ => return None,
            };
            element_pushes.push(push);
        }

        Some(quote! {
            let mut elements: Vec<serde_json::Value> =
                Vec::with_capacity(#element_count);
            #(#element_pushes)*
            Ok(serde_json::Value::Array(elements))
        })
    }

    /// Phase 2 M1+M3 emit. Produces a parallel `impl` block carrying
    /// per-rule `parse_<rule>_typed` methods plus the entry-point
    /// `parse_full_<entry>_typed` — all returning `ParseResult<serde_json::Value>`.
    ///
    /// Stage 1 (this commit) keeps the bodies as thin wrappers around the legacy
    /// `parse_<rule>` plus `serde_json::to_value(&node)`. Functionally equivalent to
    /// "parse + AST-dump-as-JSON" today. The architectural pattern and the public
    /// API surface are now stable across rules — every rule has a typed entry, and
    /// the entry-point typed method dispatches to its per-rule typed entry.
    ///
    /// Subsequent M3 stages replace the per-rule bodies with shape-typed emit per
    /// `ASTNode::Or` / `Sequence` / `Atom` / `Quantified` / `Lookahead` that builds
    /// `serde_json::Value` directly without going through `ParseNode`. That is the
    /// architectural slice that closes the remaining PGEN-RGX-0073 gap by
    /// eliminating the generic `ParseNode` tree allocation per rule entry. The
    /// signature `fn parse_<rule>_typed(&mut self) -> ParseResult<serde_json::Value>`
    /// is stable; only the body changes per stage.
    fn generate_typed_parser_impl_skeleton(
        &self,
        parser_name: &Ident,
        grammar_tree: &HashMap<String, ASTNode>,
        rule_order: &[String],
        entry_rule: &str,
    ) -> TokenStream {
        let parse_full_method = format_ident!("parse_full_{}", entry_rule);
        let parse_full_typed_method = format_ident!("parse_full_{}_typed", entry_rule);
        let parse_entry_typed_method = format_ident!("parse_{}_typed", entry_rule);

        let per_rule_typed_methods: Vec<TokenStream> = rule_order
            .iter()
            .map(|rule_name| {
                let typed_method_name = format_ident!("parse_{}_typed", rule_name);
                let rule_label = rule_name.as_str();
                let body = grammar_tree
                    .get(rule_name)
                    .and_then(|ast_node| self.generate_typed_node_body(ast_node, rule_name))
                    .unwrap_or_else(|| {
                        // Stage 1 fallback: legacy parse + serde_json::to_value.
                        // Used for shapes / annotations not yet covered by the
                        // typed-emit dispatcher (everything except non-annotated
                        // ASTNode::Atom in stage 2). Future M3 stages replace
                        // these fallbacks with shape-typed bodies per shape.
                        let legacy_method_name = format_ident!("parse_{}", rule_name);
                        quote! {
                            let node = self.#legacy_method_name()?;
                            serde_json::to_value(&node).map_err(|err| {
                                self.create_contextual_error(&format!(
                                    "Phase 2 M3 stage 1 typed serialization failed for rule '{}': {}",
                                    #rule_label,
                                    err
                                ))
                            })
                        }
                    });
                quote! {
                    /// Phase 2 M3 per-rule typed entry. Returns a
                    /// `serde_json::Value` representing the parsed subtree.
                    /// Body shape depends on the rule's AST node:
                    /// - `ASTNode::Atom` (non-annotated): shape-typed emit
                    ///   that returns `Value::String(matched_text)` for
                    ///   leaves or dispatches to the inner rule's typed
                    ///   method for rule references — `ParseNode` is
                    ///   bypassed.
                    /// - everything else: stage-1 fallback (`legacy + to_value`)
                    ///   pending dedicated typed emit for that shape.
                    pub fn #typed_method_name(
                        &mut self,
                    ) -> ParseResult<serde_json::Value> {
                        #body
                    }
                }
            })
            .collect();

        quote! {
            impl<'input> #parser_name<'input> {
                #(#per_rule_typed_methods)*

                /// Phase 2 M1 + M3 stage 1: full-input typed entry point.
                /// Stage 1 body dispatches through the per-rule typed method
                /// then enforces end-of-input. Stage 1's per-rule typed
                /// method internally calls the legacy `parse_<rule>` plus
                /// `serde_json::to_value`, so the same parse work happens
                /// once. Stage 2+ replaces the per-rule body with shape-
                /// typed emit and the legacy parse goes away on this path.
                pub fn #parse_full_typed_method(
                    &mut self,
                ) -> ParseResult<serde_json::Value> {
                    let value = self.#parse_entry_typed_method()?;
                    if self.position != self.input.len() {
                        return Err(self.create_contextual_error(
                            "Phase 2 typed entry: trailing input not consumed",
                        ));
                    }
                    Ok(value)
                }
            }
        }
    }

    fn generate_parse_method(&self, entry_rule: &str) -> TokenStream {
        let parse_method = format_ident!("parse_{}", entry_rule);
        let parse_full_method = format_ident!("parse_full_{}", entry_rule);
        let allow_trailing_layout = !self.grammar_name.eq_ignore_ascii_case("regex");

        quote! {
            pub fn parse(&mut self) -> ParseResult<ParseNode<'input>> {
                self.recovery_events.clear();
                self.recovery_counts.clear();
                self.recovery_parse_count = 0;
                self.coverage_target_events.clear();
                self.coverage_target_rule_hits.clear();
                self.coverage_target_branch_hits.clear();
                self.negative_case_events.clear();
                self.negative_case_rule_hits.clear();
                self.deterministic_partition_events.clear();
                self.deterministic_partition_rule_hits.clear();
                self.semantic_runtime_state = crate::ast_pipeline::SemanticRuntimeState::new();
                self.#parse_method()
            }

            pub fn parse_full(&mut self) -> ParseResult<ParseNode<'input>> {
                let parsed = self.parse()?;
                if #allow_trailing_layout {
                    // Allow trailing layout/comments so parse_full reports structural completeness.
                    self.consume_layout_for_terminal("<EOF>");
                }
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

            pub fn set_grammar_profile(&mut self, profile: Option<&str>) {
                self.grammar_profile = profile.map(|value| value.to_string());
            }

            pub fn grammar_profile(&self) -> Option<&str> {
                self.grammar_profile.as_deref()
            }

            pub fn semantic_runtime_annotations(
                &self,
            ) -> &crate::ast_pipeline::CompiledSemanticRuntimeAnnotations {
                &self.semantic_runtime_annotations
            }

            pub fn semantic_runtime_state(&self) -> &crate::ast_pipeline::SemanticRuntimeState {
                &self.semantic_runtime_state
            }

            pub fn semantic_runtime_state_mut(
                &mut self,
            ) -> &mut crate::ast_pipeline::SemanticRuntimeState {
                &mut self.semantic_runtime_state
            }

            pub fn semantic_runtime_transaction_for_rule(
                &mut self,
                rule_name: &str,
            ) -> (crate::ast_pipeline::SemanticRuntimeTransaction<'_>, usize) {
                self.semantic_runtime_state
                    .transaction_for_rule(&self.semantic_runtime_annotations, rule_name)
            }

            fn semantic_predicate_debug_label(
                &self,
                spec: &crate::ast_pipeline::SemanticPredicateSpec,
            ) -> String {
                format!("{} {:?}", spec.name, spec.args)
            }

            pub fn with_semantic_runtime_rule_transaction<F>(
                &mut self,
                rule_name: &str,
                f: F,
            ) -> ParseResult<ParseNode<'input>>
            where
                F: FnOnce(&mut Self) -> ParseResult<(ParseNode<'input>, Option<ParseContent<'input>>)>,
            {
                // PGEN-RGX-0073 Optim #11: fast-path when no semantic
                // directives are configured for this rule. Skips the
                // state-take + clone + transaction-create + commit
                // sequence below, which costs 4–6% of self-time on the
                // regex parser hot path (samply, post-Optim-#10) even
                // though the regex grammar has zero semantic predicates.
                // Two-level check: whole-grammar emptiness
                // (`is_empty`, O(1) HashMap len check) catches grammars
                // like regex / json that have no predicates anywhere;
                // per-rule lookup (`has_rule`, O(1) HashMap probe)
                // catches the unannotated rules in grammars that have
                // some predicates elsewhere (e.g. systemverilog).
                if self.semantic_runtime_annotations.is_empty()
                    || !self.semantic_runtime_annotations.has_rule(rule_name)
                {
                    let (node, _raw) = f(self)?;
                    return Ok(node);
                }

                let original_semantic_runtime_state =
                    std::mem::take(&mut self.semantic_runtime_state);
                self.semantic_runtime_state = original_semantic_runtime_state.clone();
                let result = {
                    let mut predicate_blocked = false;
                    for directive in self.semantic_runtime_annotations.pre_predicates_for_rule(rule_name)
                    {
                        match self
                            .semantic_runtime_state
                            .evaluate_directive_predicate(directive)
                        {
                            Some(true) => {}
                            Some(false) => {
                                if self.logger_enabled {
                                    if let crate::ast_pipeline::SemanticRuntimeDirective::Predicate(spec) =
                                        directive
                                    {
                                        self.logger.log_info(
                                            file!(),
                                            line!(),
                                            &format!(
                                                "🚫 Rule '{}' rejected by pre predicate '{}'",
                                                rule_name,
                                                self.semantic_predicate_debug_label(spec),
                                            ),
                                        );
                                    }
                                }
                                predicate_blocked = true;
                                break;
                            }
                            None => {}
                        }
                    }

                    if predicate_blocked {
                        Err(ParseError::Backtrack {
                            position: self.position,
                        })
                    } else {
                        let (node, semantic_raw_content) = f(self)?;
                        let semantic_raw_content =
                            semantic_raw_content.as_ref().unwrap_or(&node.content);
                        let mut semantic_runtime_state =
                            std::mem::take(&mut self.semantic_runtime_state);
                        let mut semantic_runtime_transaction = semantic_runtime_state.transaction();
                        for directive in self
                            .semantic_runtime_annotations
                            .effect_directives_for_rule(rule_name)
                        {
                            let _ = self.apply_semantic_runtime_effect_directive(
                                &mut semantic_runtime_transaction,
                                directive,
                                &node.content,
                            )?;
                        }
                        let mut post_predicate_blocked = false;
                        let mut blocked_post_predicate: Option<String> = None;
                        for directive in self
                            .semantic_runtime_annotations
                            .post_predicates_for_rule(rule_name)
                        {
                            match directive {
                                crate::ast_pipeline::SemanticRuntimeDirective::Predicate(spec)
                                    if spec.phase
                                        == crate::ast_pipeline::SemanticPredicatePhase::Post =>
                                {
                                    let resolved_spec = self
                                        .resolve_semantic_predicate_spec_against_content(
                                            spec,
                                            semantic_raw_content,
                                            &node.content,
                                        )?;
                                    match semantic_runtime_transaction
                                        .state()
                                        .evaluate_content_aware_predicate(
                                            &resolved_spec,
                                            semantic_raw_content,
                                            &node.content,
                                        )
                                    {
                                        Some(true) => {}
                                        Some(false) => {
                                            blocked_post_predicate = Some(
                                                self.semantic_predicate_debug_label(&resolved_spec),
                                            );
                                            post_predicate_blocked = true;
                                            break;
                                        }
                                        None => {}
                                    }
                                }
                                crate::ast_pipeline::SemanticRuntimeDirective::OpenScope(_)
                                | crate::ast_pipeline::SemanticRuntimeDirective::CloseScope(_)
                                | crate::ast_pipeline::SemanticRuntimeDirective::EmitFact(_)
                                | crate::ast_pipeline::SemanticRuntimeDirective::Predicate(_) => {}
                            }
                        }
                        if post_predicate_blocked {
                            if self.logger_enabled {
                                self.logger.log_info(
                                    file!(),
                                    line!(),
                                    &format!(
                                        "🚫 Rule '{}' rejected by post predicate '{}'",
                                        rule_name,
                                        blocked_post_predicate
                                            .as_deref()
                                            .unwrap_or("<unknown>"),
                                    ),
                                );
                            }
                            Err(ParseError::Backtrack {
                                position: node.span.start,
                            })
                        } else {
                            let _ = semantic_runtime_transaction.commit();
                            self.semantic_runtime_state = semantic_runtime_state;
                            Ok(node)
                        }
                    }
                };
                if result.is_err() {
                    self.semantic_runtime_state = original_semantic_runtime_state;
                }
                result
            }

            fn apply_semantic_runtime_effect_directive(
                &self,
                transaction: &mut crate::ast_pipeline::SemanticRuntimeTransaction<'_>,
                directive: &crate::ast_pipeline::SemanticRuntimeDirective,
                root_content: &ParseContent<'input>,
            ) -> ParseResult<bool> {
                match directive {
                    crate::ast_pipeline::SemanticRuntimeDirective::Predicate(_) => Ok(false),
                    crate::ast_pipeline::SemanticRuntimeDirective::OpenScope(spec) => {
                        let resolved_name = spec
                            .name
                            .as_ref()
                            .map(|value| {
                                self.resolve_semantic_runtime_value_against_content(
                                    value,
                                    root_content,
                                )
                                .ok_or_else(|| {
                                    self.create_contextual_error(&format!(
                                        "Semantic runtime could not resolve scope name for directive in current parse result"
                                    ))
                                })
                            })
                            .transpose()?;
                        Ok(transaction.apply_directive(
                            &crate::ast_pipeline::SemanticRuntimeDirective::OpenScope(
                                crate::ast_pipeline::SemanticScopeSpec {
                                    kind: spec.kind.clone(),
                                    name: resolved_name,
                                },
                            ),
                        ))
                    }
                    crate::ast_pipeline::SemanticRuntimeDirective::CloseScope(spec) => {
                        let resolved_name = spec
                            .name
                            .as_ref()
                            .map(|value| {
                                self.resolve_semantic_runtime_value_against_content(
                                    value,
                                    root_content,
                                )
                                .ok_or_else(|| {
                                    self.create_contextual_error(&format!(
                                        "Semantic runtime could not resolve close-scope name for directive in current parse result"
                                    ))
                                })
                            })
                            .transpose()?;
                        Ok(transaction.apply_directive(
                            &crate::ast_pipeline::SemanticRuntimeDirective::CloseScope(
                                crate::ast_pipeline::SemanticCloseScopeSpec {
                                    kind: spec.kind.clone(),
                                    name: resolved_name,
                                },
                            ),
                        ))
                    }
                    crate::ast_pipeline::SemanticRuntimeDirective::EmitFact(spec) => {
                        let resolved_name = self
                            .resolve_semantic_runtime_value_against_content(
                                &spec.name,
                                root_content,
                            )
                            .ok_or_else(|| {
                                self.create_contextual_error(&format!(
                                    "Semantic runtime could not resolve fact name for directive in current parse result"
                                ))
                            })?;
                        let resolved_attributes = self
                            .resolve_unified_semantic_properties_against_content(
                                &spec.attributes,
                                root_content,
                            )?;
                        Ok(transaction.apply_directive(
                            &crate::ast_pipeline::SemanticRuntimeDirective::EmitFact(
                                crate::ast_pipeline::SemanticFactSpec {
                                    kind: spec.kind.clone(),
                                    name: resolved_name,
                                    attributes: resolved_attributes,
                                },
                            ),
                        ))
                    }
                }
            }

            fn resolve_semantic_runtime_value_against_content(
                &self,
                value: &crate::ast_pipeline::SemanticRuntimeValue,
                root_content: &ParseContent<'input>,
            ) -> Option<crate::ast_pipeline::SemanticRuntimeValue> {
                match value {
                    crate::ast_pipeline::SemanticRuntimeValue::RuleReference(reference) => self
                        .resolve_semantic_reference(root_content, reference)
                        .map(|resolved| self.coerce_semantic_runtime_scalar(&resolved)),
                    crate::ast_pipeline::SemanticRuntimeValue::String(text) => Some(
                        crate::ast_pipeline::SemanticRuntimeValue::String(text.clone()),
                    ),
                    crate::ast_pipeline::SemanticRuntimeValue::Identifier(text) => Some(
                        crate::ast_pipeline::SemanticRuntimeValue::Identifier(text.clone()),
                    ),
                    crate::ast_pipeline::SemanticRuntimeValue::Number(text) => Some(
                        crate::ast_pipeline::SemanticRuntimeValue::Number(text.clone()),
                    ),
                    crate::ast_pipeline::SemanticRuntimeValue::Boolean(value) => Some(
                        crate::ast_pipeline::SemanticRuntimeValue::Boolean(*value),
                    ),
                    crate::ast_pipeline::SemanticRuntimeValue::Null => Some(
                        crate::ast_pipeline::SemanticRuntimeValue::Null,
                    ),
                }
            }

            fn resolve_unified_semantic_properties_against_content(
                &self,
                properties: &[crate::ast_pipeline::UnifiedSemanticProperty],
                root_content: &ParseContent<'input>,
            ) -> ParseResult<Vec<crate::ast_pipeline::UnifiedSemanticProperty>> {
                let mut resolved = Vec::with_capacity(properties.len());
                for property in properties {
                    resolved.push(crate::ast_pipeline::UnifiedSemanticProperty {
                        key: property.key.clone(),
                        value: self.resolve_unified_semantic_value_against_content(
                            &property.value,
                            root_content,
                        )?,
                    });
                }
                Ok(resolved)
            }

            fn resolve_semantic_predicate_spec_against_content(
                &self,
                spec: &crate::ast_pipeline::SemanticPredicateSpec,
                raw_content: &ParseContent<'input>,
                shaped_content: &ParseContent<'input>,
            ) -> ParseResult<crate::ast_pipeline::SemanticPredicateSpec> {
                let selected_content = match spec.view {
                    crate::ast_pipeline::SemanticPredicateContentView::Raw => raw_content,
                    crate::ast_pipeline::SemanticPredicateContentView::Shaped => shaped_content,
                };

                let mut resolved_args = Vec::with_capacity(spec.args.len());
                for arg in &spec.args {
                    resolved_args.push(
                        self.resolve_unified_semantic_value_against_content(arg, selected_content)?,
                    );
                }

                Ok(crate::ast_pipeline::SemanticPredicateSpec {
                    name: spec.name.clone(),
                    args: resolved_args,
                    phase: spec.phase,
                    view: spec.view,
                })
            }

            fn try_resolve_semantic_predicate_spec_against_content(
                &self,
                spec: &crate::ast_pipeline::SemanticPredicateSpec,
                raw_content: &ParseContent<'input>,
                shaped_content: &ParseContent<'input>,
            ) -> ParseResult<Option<crate::ast_pipeline::SemanticPredicateSpec>> {
                let selected_content = match spec.view {
                    crate::ast_pipeline::SemanticPredicateContentView::Raw => raw_content,
                    crate::ast_pipeline::SemanticPredicateContentView::Shaped => shaped_content,
                };

                let mut resolved_args = Vec::with_capacity(spec.args.len());
                for arg in &spec.args {
                    let Some(resolved_arg) =
                        self.try_resolve_unified_semantic_value_against_content(
                            arg,
                            selected_content,
                        )?
                    else {
                        return Ok(None);
                    };
                    resolved_args.push(resolved_arg);
                }

                Ok(Some(crate::ast_pipeline::SemanticPredicateSpec {
                    name: spec.name.clone(),
                    args: resolved_args,
                    phase: spec.phase,
                    view: spec.view,
                }))
            }

            fn resolve_unified_semantic_value_against_content(
                &self,
                value: &crate::ast_pipeline::UnifiedSemanticValue,
                root_content: &ParseContent<'input>,
            ) -> ParseResult<crate::ast_pipeline::UnifiedSemanticValue> {
                match value {
                    crate::ast_pipeline::UnifiedSemanticValue::RuleReference(reference) => self
                        .resolve_semantic_reference(root_content, reference)
                        .map(|resolved| self.coerce_unified_semantic_scalar(&resolved))
                        .ok_or_else(|| {
                            self.create_contextual_error(&format!(
                                "Semantic runtime could not resolve attribute reference '{}'",
                                reference
                            ))
                        }),
                    crate::ast_pipeline::UnifiedSemanticValue::String(text) => {
                        Ok(crate::ast_pipeline::UnifiedSemanticValue::String(text.clone()))
                    }
                    crate::ast_pipeline::UnifiedSemanticValue::Identifier(text) => Ok(
                        crate::ast_pipeline::UnifiedSemanticValue::Identifier(text.clone()),
                    ),
                    crate::ast_pipeline::UnifiedSemanticValue::Number(text) => {
                        Ok(crate::ast_pipeline::UnifiedSemanticValue::Number(text.clone()))
                    }
                    crate::ast_pipeline::UnifiedSemanticValue::Boolean(value) => Ok(
                        crate::ast_pipeline::UnifiedSemanticValue::Boolean(*value),
                    ),
                    crate::ast_pipeline::UnifiedSemanticValue::Null => {
                        Ok(crate::ast_pipeline::UnifiedSemanticValue::Null)
                    }
                    crate::ast_pipeline::UnifiedSemanticValue::Array(elements) => {
                        let mut resolved = Vec::with_capacity(elements.len());
                        for element in elements {
                            resolved.push(
                                self.resolve_unified_semantic_value_against_content(
                                    element,
                                    root_content,
                                )?,
                            );
                        }
                        Ok(crate::ast_pipeline::UnifiedSemanticValue::Array(resolved))
                    }
                    crate::ast_pipeline::UnifiedSemanticValue::Object(properties) => Ok(
                        crate::ast_pipeline::UnifiedSemanticValue::Object(
                            self.resolve_unified_semantic_properties_against_content(
                                properties,
                                root_content,
                            )?,
                        ),
                    ),
                }
            }

            fn try_resolve_unified_semantic_value_against_content(
                &self,
                value: &crate::ast_pipeline::UnifiedSemanticValue,
                root_content: &ParseContent<'input>,
            ) -> ParseResult<Option<crate::ast_pipeline::UnifiedSemanticValue>> {
                match value {
                    crate::ast_pipeline::UnifiedSemanticValue::RuleReference(reference) => Ok(
                        self.resolve_semantic_reference(root_content, reference)
                            .map(|resolved| self.coerce_unified_semantic_scalar(&resolved)),
                    ),
                    crate::ast_pipeline::UnifiedSemanticValue::String(text) => Ok(Some(
                        crate::ast_pipeline::UnifiedSemanticValue::String(text.clone()),
                    )),
                    crate::ast_pipeline::UnifiedSemanticValue::Identifier(text) => Ok(Some(
                        crate::ast_pipeline::UnifiedSemanticValue::Identifier(text.clone()),
                    )),
                    crate::ast_pipeline::UnifiedSemanticValue::Number(text) => Ok(Some(
                        crate::ast_pipeline::UnifiedSemanticValue::Number(text.clone()),
                    )),
                    crate::ast_pipeline::UnifiedSemanticValue::Boolean(value) => Ok(Some(
                        crate::ast_pipeline::UnifiedSemanticValue::Boolean(*value),
                    )),
                    crate::ast_pipeline::UnifiedSemanticValue::Null => {
                        Ok(Some(crate::ast_pipeline::UnifiedSemanticValue::Null))
                    }
                    crate::ast_pipeline::UnifiedSemanticValue::Array(elements) => {
                        let mut resolved = Vec::with_capacity(elements.len());
                        for element in elements {
                            let Some(resolved_element) =
                                self.try_resolve_unified_semantic_value_against_content(
                                    element,
                                    root_content,
                                )?
                            else {
                                return Ok(None);
                            };
                            resolved.push(resolved_element);
                        }
                        Ok(Some(crate::ast_pipeline::UnifiedSemanticValue::Array(resolved)))
                    }
                    crate::ast_pipeline::UnifiedSemanticValue::Object(properties) => {
                        let mut resolved = Vec::with_capacity(properties.len());
                        for property in properties {
                            let Some(resolved_value) =
                                self.try_resolve_unified_semantic_value_against_content(
                                    &property.value,
                                    root_content,
                                )?
                            else {
                                return Ok(None);
                            };
                            resolved.push(crate::ast_pipeline::UnifiedSemanticProperty {
                                key: property.key.clone(),
                                value: resolved_value,
                            });
                        }
                        Ok(Some(crate::ast_pipeline::UnifiedSemanticValue::Object(resolved)))
                    }
                }
            }

            fn coerce_semantic_runtime_scalar(
                &self,
                value: &str,
            ) -> crate::ast_pipeline::SemanticRuntimeValue {
                let normalized = value.trim();
                if normalized.eq_ignore_ascii_case("true") {
                    return crate::ast_pipeline::SemanticRuntimeValue::Boolean(true);
                }
                if normalized.eq_ignore_ascii_case("false") {
                    return crate::ast_pipeline::SemanticRuntimeValue::Boolean(false);
                }
                if normalized.parse::<f64>().is_ok() {
                    return crate::ast_pipeline::SemanticRuntimeValue::Number(
                        normalized.to_string(),
                    );
                }
                if self.semantic_identifier(normalized) {
                    return crate::ast_pipeline::SemanticRuntimeValue::Identifier(
                        normalized.to_string(),
                    );
                }
                crate::ast_pipeline::SemanticRuntimeValue::String(normalized.to_string())
            }

            fn coerce_unified_semantic_scalar(
                &self,
                value: &str,
            ) -> crate::ast_pipeline::UnifiedSemanticValue {
                let normalized = value.trim();
                if normalized.eq_ignore_ascii_case("true") {
                    return crate::ast_pipeline::UnifiedSemanticValue::Boolean(true);
                }
                if normalized.eq_ignore_ascii_case("false") {
                    return crate::ast_pipeline::UnifiedSemanticValue::Boolean(false);
                }
                if normalized.parse::<f64>().is_ok() {
                    return crate::ast_pipeline::UnifiedSemanticValue::Number(
                        normalized.to_string(),
                    );
                }
                if self.semantic_identifier(normalized) {
                    return crate::ast_pipeline::UnifiedSemanticValue::Identifier(
                        normalized.to_string(),
                    );
                }
                crate::ast_pipeline::UnifiedSemanticValue::String(normalized.to_string())
            }

            pub fn recovery_events(&self) -> &[RecoveryEvent] {
                &self.recovery_events
            }

            pub fn take_recovery_events(&mut self) -> Vec<RecoveryEvent> {
                std::mem::take(&mut self.recovery_events)
            }

            pub fn recovery_event_count(&self) -> usize {
                self.recovery_events.len()
            }

            pub fn recovery_parse_count(&self) -> usize {
                self.recovery_parse_count
            }

            pub fn recovery_global_count(&self) -> usize {
                self.recovery_global_count
            }

            pub fn coverage_target_events(&self) -> &[CoverageTargetEvent] {
                &self.coverage_target_events
            }

            pub fn take_coverage_target_events(&mut self) -> Vec<CoverageTargetEvent> {
                std::mem::take(&mut self.coverage_target_events)
            }

            pub fn coverage_target_event_count(&self) -> usize {
                self.coverage_target_events.len()
            }

            pub fn coverage_target_rule_hits(&self) -> &HashMap<String, usize> {
                &self.coverage_target_rule_hits
            }

            pub fn coverage_target_branch_hits(&self) -> &HashMap<String, usize> {
                &self.coverage_target_branch_hits
            }

            pub fn negative_case_events(&self) -> &[NegativeCaseEvent] {
                &self.negative_case_events
            }

            pub fn take_negative_case_events(&mut self) -> Vec<NegativeCaseEvent> {
                std::mem::take(&mut self.negative_case_events)
            }

            pub fn negative_case_event_count(&self) -> usize {
                self.negative_case_events.len()
            }

            pub fn negative_case_rule_hits(&self) -> &HashMap<String, usize> {
                &self.negative_case_rule_hits
            }

            pub fn deterministic_partition_events(&self) -> &[DeterministicPartitionEvent] {
                &self.deterministic_partition_events
            }

            pub fn take_deterministic_partition_events(&mut self) -> Vec<DeterministicPartitionEvent> {
                std::mem::take(&mut self.deterministic_partition_events)
            }

            pub fn deterministic_partition_event_count(&self) -> usize {
                self.deterministic_partition_events.len()
            }

            pub fn deterministic_partition_rule_hits(&self) -> &HashMap<String, usize> {
                &self.deterministic_partition_rule_hits
            }

            pub fn deterministic_partition_runtime_mode(&self) -> DeterministicPartitionRuntimeMode {
                self.deterministic_partition_runtime_mode
            }

            pub fn set_deterministic_partition_runtime_mode(
                &mut self,
                mode: DeterministicPartitionRuntimeMode,
            ) {
                self.deterministic_partition_runtime_mode = mode;
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
        // Default to "recursive" — preserves prior emit shape for
        // callers that don't supply a recursive-rules set (notably the
        // unit tests that exercise `generate_rule_method` directly).
        // Production codegen goes through `generate_rule_method_with_recursion`.
        self.generate_rule_method_with_recursion(rule_name, ast_node, rule_order, filename, true)
    }

    fn generate_rule_method_with_recursion(
        &self,
        rule_name: &str,
        ast_node: &ASTNode,
        rule_order: &[String],
        filename: &str,
        is_recursive: bool,
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

        // Apply rule-level return annotation for non-Or roots. The Or path
        // (`generate_or_logic`) already applies per-branch transforms inline,
        // so applying again would double-transform. For Sequence / Atom /
        // Quantified / Lookahead roots, the annotation otherwise is silently
        // dropped at codegen — that was the regex-grammar drop bug. The
        // shadow-rebind pattern is safe because `parse_logic` introduces a
        // `let result = ...` binding that the transform expression reads, and
        // downstream code (relational guards, coverage events, etc.) sees the
        // shadowed `result`.
        //
        // The transform expression is wrapped in `{ ... }` so callers can
        // emit either a single expression OR a sequence of statements ending
        // in an expression. `generate_return_transform`'s parsed-ast-failed
        // fallback returns the latter shape (warning let-bindings followed
        // by `result.clone()`), which is only valid in an expression-block
        // context. Without the block, `let result = let _warning = ...;` is
        // a syn-parse error ("expected `=`").
        let post_parse_transform_tokens: TokenStream = match ast_node {
            ASTNode::Or { .. } => quote! {},
            _ => {
                // Look up the explicit branch annotation; if absent and the
                // body is a single-element shape, fall back to a synthetic
                // `-> $1` default. Multi-element Sequences keep no-transform
                // semantics until an author declares one.
                let annotation_opt = self
                    .branch_return_annotations
                    .get(rule_name)
                    .and_then(|branches| branches.get(0).cloned())
                    .flatten()
                    .or_else(|| {
                        Self::synthesize_default_passthrough_for_single_element_branch(ast_node)
                    });
                if let Some(annotation) = annotation_opt {
                    let transform = self.generate_return_transform(
                        &annotation,
                        rule_name,
                        &["result".to_string()],
                    )?;
                    quote! {
                        let result = { #transform };
                    }
                } else {
                    quote! {}
                }
            }
        };

        let relational_guards = self.semantic_relational_constraint_tokens(rule_name);
        let coverage_target_policy = self.rule_coverage_target_policy(rule_name);
        let coverage_target_weight = coverage_target_policy.coverage_target_weight;
        let coverage_critical_path = coverage_target_policy.critical_path;
        let negative_case_policy = self.rule_negative_case_policy(rule_name);
        let negative_case_enabled = negative_case_policy.invalid_case;
        let negative_case_strict = negative_case_policy.negative;
        let deterministic_partition_policy = self.rule_deterministic_partition_policy(rule_name);
        let deterministic_partition_enabled = deterministic_partition_policy.enabled;
        let deterministic_partition_group = deterministic_partition_policy
            .group_label
            .unwrap_or_else(|| format!("rule.{}", rule_name));
        let recursion_guard_max_depth = GENERATED_RECURSION_GUARD_MAX_DEPTH;

        // Optim #15: elide / lazify per-rule observability hooks that
        // the previous emit ran unconditionally on every rule entry.
        //
        // 1. `record_coverage_target_event` early-returns when
        //    `coverage_target_weight == 0`. The weight is a compile-time
        //    constant per rule (no runtime override path), so when zero
        //    we elide the call entirely instead of paying the arg setup
        //    plus call frame for an immediate return.
        //
        // 2. `record_deterministic_partition_event` early-returns when
        //    `enabled == false`. The previous emit STILL paid for the
        //    `effective_deterministic_partition_group` call before that
        //    early-return — and that helper unconditionally allocates a
        //    `String` (`.to_string()` or `format!`). For grammars that
        //    declare no `@deterministic_partition` (the regex grammar
        //    is the canonical case), this was a String allocation per
        //    rule entry that was immediately thrown away. When the rule
        //    has no annotation, we elide the entire partition logic at
        //    codegen — under the default `AnnotationDriven` runtime
        //    mode no event would fire anyway, and `ForceEnabled` is a
        //    diagnostic mode rather than a steady-state production
        //    setting. When the rule does have the annotation, the
        //    group computation is moved inside the runtime
        //    `effective_enabled` check so the allocation only happens
        //    when an event will actually be recorded.
        let coverage_event_emit = if coverage_target_weight == 0 {
            quote! {}
        } else {
            quote! {
                parser.record_coverage_target_event(
                    #rule_name,
                    start_pos,
                    end_pos,
                    semantic_selected_branch_index,
                    #coverage_target_weight,
                    #coverage_critical_path,
                );
            }
        };
        let partition_event_emit = if !deterministic_partition_enabled {
            quote! {}
        } else {
            quote! {
                let deterministic_partition_effective_enabled =
                    parser.effective_deterministic_partition_enabled(true);
                if deterministic_partition_effective_enabled {
                    let deterministic_partition_effective_group =
                        parser.effective_deterministic_partition_group(
                            #rule_name,
                            #deterministic_partition_group,
                        );
                    parser.record_deterministic_partition_event(
                        #rule_name,
                        start_pos,
                        end_pos,
                        true,
                        &deterministic_partition_effective_group,
                    );
                }
            }
        };

        // Optim #14: conditionally emit the semantic-runtime transaction
        // wrapper per rule. The inner `memoized_call` body is identical
        // in both arms; only the surrounding wrapper differs. The Optim
        // #13 grammar-level gate is generalized here to a per-rule check
        // so a partially-annotated grammar (e.g. `regex`, where most
        // rules have no semantic annotations) can elide the wrapper on
        // its non-annotated rules too.
        //
        // Optim #16: when the rule is statically non-recursive, the
        // `memoized_call` wrapper is also dead weight. Memoization gives
        // Packrat its linear-time guarantee in the presence of recursive
        // / shared-sub-parse calls; for a rule that never re-enters
        // itself directly or transitively, the cache cannot observe a
        // hit it could not avoid by simply running the body once. The
        // emitted body is the same; only the wrapping differs.
        let rule_body_inner = quote! {
            let semantic_capture_raw_for_post =
                parser.semantic_runtime_annotations
                    .needs_raw_post_capture_for_rule(#rule_name);
            let mut semantic_selected_branch_index: Option<usize> = None;
            let mut semantic_raw_content: Option<ParseContent<'input>> = None;
            // Main parsing logic - produces the 'result' variable
            #parse_logic;

            // Apply rule-level return annotation for non-Or roots
            // (Or roots apply per-branch transforms inline)
            #post_parse_transform_tokens

            #relational_guards

            let end_pos = parser.position;
            #coverage_event_emit
            #partition_event_emit

            Ok((
                ParseNode {
                    rule_name: #rule_name,
                    content: result,
                    span: start_pos..end_pos,
                },
                semantic_raw_content,
            ))
        };
        let memoized_inner: TokenStream = if is_recursive {
            quote! {
                parser.memoized_call(Self::#rule_const, |parser| {
                    #rule_body_inner
                })
            }
        } else {
            quote! {
                (|parser: &mut Self| -> ParseResult<(ParseNode<'input>, Option<ParseContent<'input>>)> {
                    #rule_body_inner
                })(parser)
            }
        };
        let wrapped_rule_call: TokenStream = if self.rule_has_no_semantic_annotations(rule_name) {
            quote! {
                let result: ParseResult<ParseNode<'input>> = (|parser: &mut Self| {
                    let inner_result = #memoized_inner;
                    inner_result.map(|(node, _raw)| node)
                })(self);
            }
        } else {
            quote! {
                let result = self.with_semantic_runtime_rule_transaction(#rule_name, |parser| {
                    #memoized_inner
                });
            }
        };

        let rule_profiles = self.rule_profiles(rule_name);
        let profile_guard = if rule_profiles.is_empty() {
            quote! {}
        } else {
            let profile_literals = rule_profiles.iter().map(|profile| profile.as_str());
            quote! {
                if !self.rule_profile_is_enabled(&[#(#profile_literals),*]) {
                    return Err(ParseError::Backtrack {
                        position,
                    });
                }
            }
        };

        // Optim #16: emit `recursion_guard.check_cycle` only for rules
        // statically known to be recursive. A non-recursive rule cannot
        // directly or transitively re-enter itself, so `check_cycle`'s
        // linear scan of `parse_stack` always returns
        // `CycleType::None` for it; the call is dead weight per rule
        // entry. Push/pop on `parse_stack` is still emitted so error
        // messages keep an accurate rule frame.
        let cycle_check_emit = if is_recursive {
            quote! {
                let cycle_type = self.recursion_guard.check_cycle(#rule_name, position);

                match cycle_type {
                    CycleType::Infinite => {
                        if self.logger_enabled {
                            self.logger.log_error(#filename, 0, &format!("💥 Infinite recursion detected in rule '{}' at position {}", #rule_name, position));
                        }
                        return Err(ParseError::InvalidSyntax {
                            message: "Infinite recursion detected",
                            position,
                        });
                    }
                    CycleType::LeftRecursive => {
                        if self.logger_enabled {
                            self.logger.log_error(#filename, 0, &format!("🔄 Left recursion detected in rule '{}' at position {}", #rule_name, position));
                        }
                        return Err(ParseError::InvalidSyntax {
                            message: "Left recursion detected",
                            position,
                        });
                    }
                    CycleType::MutualRecursive { depth, ref rules } if depth >= #recursion_guard_max_depth => {
                        if self.logger_enabled {
                            self.logger.log_error(#filename, 0, &format!("🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})", #rule_name, position, depth));
                        }
                        return Err(ParseError::RecursionDepthExceeded {
                            position,
                            depth,
                        });
                    }
                    _ => {}
                }
            }
        } else {
            quote! {}
        };

        // Build the complete method
        Ok(quote! {
            pub fn #method_name(&mut self) -> ParseResult<ParseNode<'input>> {
                let filename_str = #filename;
                // Check for recursion cycles (recursive rules only after Optim #16)
                let position = self.position;
                #cycle_check_emit

                #profile_guard

                self.recursion_guard.enter(#rule_name, position);

                // Declare start_pos outside the closure so it can be used outside
                let start_pos = self.position;

                // PGEN-RGX-0073 Optim #13: when the grammar has no
                // semantic annotations, skip the semantic-runtime
                // wrapper at codegen time. The runtime fast-path landed
                // in Optim #11 already short-circuits in this case, but
                // the wrapper call + per-call HashMap probe (~2700 sites
                // in the regex parser) still costs a few %; eliding at
                // codegen drops it entirely.
                #wrapped_rule_call

                self.recursion_guard.exit();

                match &result {
                    Ok(node) => {
                        if self.logger_enabled {
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
                        if #negative_case_enabled {
                            self.record_negative_case_failure(
                                #rule_name,
                                start_pos,
                                self.position,
                                #negative_case_strict,
                                &format!("{:?}", e),
                            );
                        }
                        if self.logger_enabled {
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
            ASTNode::Lookahead { element, positive } => {
                eprintln!(
                    "        Processing {} lookahead node - File: {}:{}",
                    if *positive { "positive" } else { "negative" },
                    file!(),
                    line!()
                );
                self.generate_lookahead_logic(element, *positive, rule_name, filename)
            }
        }
    }

    fn generate_lookahead_logic(
        &self,
        element: &ASTNode,
        positive: bool,
        rule_name: &str,
        filename: &str,
    ) -> Result<TokenStream> {
        let inner_logic = self.generate_node_parsing_logic(element, rule_name, filename)?;
        let lookahead_kind = if positive {
            "positive lookahead"
        } else {
            "negative lookahead"
        };

        if positive {
            Ok(quote! {
                let lookahead_start = parser.position;
                let matched = parser.try_parse(|p| {
                    let parser = p;
                    #inner_logic;
                    Ok(())
                });
                parser.position = lookahead_start;
                if matched.is_none() {
                    return Err(ParseError::Backtrack {
                        position: lookahead_start,
                    });
                }
                if parser.logger_enabled {
                    parser.logger.log_debug(#filename, 0, &format!(
                        "👀 Rule '{}' satisfied {} at position {}",
                        #rule_name,
                        #lookahead_kind,
                        lookahead_start
                    ));
                }
                let result = ParseContent::Sequence(Vec::new());
            })
        } else {
            Ok(quote! {
                let lookahead_start = parser.position;
                let matched = parser.try_parse(|p| {
                    let parser = p;
                    #inner_logic;
                    Ok(())
                });
                parser.position = lookahead_start;
                if matched.is_some() {
                    return Err(ParseError::Backtrack {
                        position: lookahead_start,
                    });
                }
                if parser.logger_enabled {
                    parser.logger.log_debug(#filename, 0, &format!(
                        "🚫 Rule '{}' satisfied {} at position {}",
                        #rule_name,
                        #lookahead_kind,
                        lookahead_start
                    ));
                }
                let result = ParseContent::Sequence(Vec::new());
            })
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

            // Resolve the branch's annotation: explicit declaration if
            // present; otherwise the synthetic `-> $1` default for single-
            // element branches; otherwise None (no transform).
            let resolved_annotation: Option<BranchAnnotation> = self
                .branch_return_annotations
                .get(rule_name)
                .and_then(|branches| branches.get(0).cloned())
                .flatten()
                .or_else(|| {
                    Self::synthesize_default_passthrough_for_single_element_branch(branch)
                });
            let has_transform = resolved_annotation.is_some();

            if has_transform {
                // Has return annotation (explicit or default) - need to transform
                let transform = resolved_annotation
                    .as_ref()
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
                        #branch_logic;  // Sets result
                        if semantic_capture_raw_for_post {
                            semantic_raw_content = Some(result.clone());
                        }
                        semantic_selected_branch_index = Some(1usize);
                    })
                } else {
                    Ok(quote! {
                        // Single-branch rule with transformation
                        #branch_logic;
                        if semantic_capture_raw_for_post {
                            semantic_raw_content = Some(result.clone());
                        }

                        // Apply transformation (reassign result). The block
                        // wrapper lets `#transform` be either a single
                        // expression OR a statement-list ending in an
                        // expression — `generate_return_transform`'s
                        // parsed-ast-failed fallback emits the latter shape
                        // and would be a syn-parse error otherwise.
                        result = { #transform };
                        semantic_selected_branch_index = Some(1usize);
                    })
                }
            } else {
                // No transformation needed - simpler code
                Ok(quote! {
                    // Single-branch rule
                    #branch_logic;  // Sets result directly
                    if semantic_capture_raw_for_post {
                        semantic_raw_content = Some(result.clone());
                    }
                    semantic_selected_branch_index = Some(1usize);
                })
            }
        } else {
            // Multi-branch - evaluate all branches and keep the longest successful match
            let branch_priorities = self.rule_branch_priorities(rule_name, branch_count);
            let associativity = self.rule_associativity(rule_name);
            let associativity_mode = associativity.as_str();
            let branch_policy = self.rule_branch_policy(rule_name);
            let branch_policy_mode = branch_policy.as_str();
            let deterministic_partition_policy =
                self.rule_deterministic_partition_policy(rule_name);
            let deterministic_partition_annotation_enabled = deterministic_partition_policy.enabled;
            let deterministic_partition_annotation_group = deterministic_partition_policy
                .group_label
                .unwrap_or_else(|| format!("rule.{}", rule_name));
            let (
                recover_enabled,
                sync_tokens,
                panic_until_tokens,
                recover_budget,
                recover_parse_budget,
                recover_global_budget,
            ) = self.rule_recovery_hints(rule_name);
            let sync_tokens_label = sync_tokens.join(", ");
            let panic_until_tokens_label = panic_until_tokens.join(", ");
            let recover_budget_label = recover_budget
                .map(|limit| limit.to_string())
                .unwrap_or_else(|| "unbounded".to_string());
            let recover_parse_budget_label = recover_parse_budget
                .map(|limit| limit.to_string())
                .unwrap_or_else(|| "unbounded".to_string());
            let recover_global_budget_label = recover_global_budget
                .map(|limit| limit.to_string())
                .unwrap_or_else(|| "unbounded".to_string());
            let sync_tokens_for_code = sync_tokens.clone();
            let panic_until_tokens_for_code = panic_until_tokens.clone();

            let recovery_failure_path = if recover_enabled {
                quote! {
                    if parser.recover_with_hints(
                        #rule_name,
                        parse_start,
                        &[#(#sync_tokens_for_code),*],
                        &[#(#panic_until_tokens_for_code),*],
                        #recover_budget,
                        #recover_parse_budget,
                        #recover_global_budget,
                    ) {
                        if parser.logger_enabled {
                            parser.logger.log_warning(#filename, 0, &format!(
                                "🛟 Rule '{}' recovered from branch failure using sync=[{}] panic_until=[{}] budget(rule={}, parse={}, global={})",
                                #rule_name,
                                #sync_tokens_label,
                                #panic_until_tokens_label,
                                #recover_budget_label,
                                #recover_parse_budget_label,
                                #recover_global_budget_label
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

            let mut branch_attempt_arms = Vec::new();
            for idx in 0..branch_count {
                let alternative = &alternatives[idx];
                eprintln!();
                let branch_logic =
                    self.generate_node_parsing_logic(alternative, rule_name, filename)?;

                // Resolve the branch's annotation: explicit declaration if
                // present; otherwise the synthetic `-> $1` default for
                // single-element branches; otherwise pass `content` through
                // as-is (no transform).
                let explicit_annotation: Option<BranchAnnotation> = self
                    .branch_return_annotations
                    .get(rule_name)
                    .and_then(|branches| branches.get(idx).cloned())
                    .flatten();
                let resolved_annotation: Option<BranchAnnotation> = explicit_annotation
                    .or_else(|| {
                        Self::synthesize_default_passthrough_for_single_element_branch(alternative)
                    });
                let transform = match resolved_annotation {
                    Some(annotation) => self.generate_return_transform(
                        &annotation,
                        rule_name,
                        &["content".to_string()],
                    )?,
                    None => quote! { content },
                };

                let branch_num = idx + 1;
                let branch_priority = branch_priorities.get(idx).copied().unwrap_or(0);
                let branch_index = idx;
                branch_attempt_arms.push(quote! {
                    #branch_index => {
                        if #branch_policy_mode == "ordered" && best_content.is_some() {
                            // Ordered branch policy keeps first successful branch.
                        } else {
                            parser.position = parse_start;
                            if let Some(content) = parser.try_parse(|p| {
                                let parser = p;
                                if parser.logger_enabled {
                                    parser.logger.log_info(#filename, 0, &format!("🚪 Entering branch {}/{} for rule '{}' at position {}", #branch_num, #branch_count, #rule_name, parser.position));
                                }
                                #branch_logic;
                                if parser.logger_enabled {
                                    parser.logger.log_info(#filename, 0, &format!("✅ Leaving branch {}/{} for rule '{}' at position {} (success)", #branch_num, #branch_count, #rule_name, parser.position));
                                }
                                Ok(result)
                            }) {
                                let candidate_end = parser.position;
                                parser.position = parse_start;
                                let candidate_priority: i64 = #branch_priority;
                                let current_branch_index: usize = #branch_index;
                                let raw_content = content;
                                let transformed = {
                                    let content = raw_content.clone();
                                    #transform
                                };
                                let mut branch_predicate_blocked = false;
                                let mut blocked_branch_predicate: Option<String> = None;
                                for directive in parser
                                    .semantic_runtime_annotations
                                    .branch_predicates_for_rule(#rule_name)
                                    .chain(
                                        parser
                                            .semantic_runtime_annotations
                                            .branch_predicates_for_rule_branch(
                                                #rule_name,
                                                current_branch_index,
                                            ),
                                    )
                                {
                                    match directive {
                                        crate::ast_pipeline::SemanticRuntimeDirective::Predicate(spec)
                                            if spec.phase
                                                == crate::ast_pipeline::SemanticPredicatePhase::Branch =>
                                        {
                                            let Some(resolved_spec) = parser
                                                .try_resolve_semantic_predicate_spec_against_content(
                                                    spec,
                                                    &raw_content,
                                                    &transformed,
                                                )?
                                            else {
                                                blocked_branch_predicate = Some(
                                                    parser.semantic_predicate_debug_label(spec),
                                                );
                                                branch_predicate_blocked = true;
                                                break;
                                            };
                                            match parser
                                                .semantic_runtime_state
                                                .evaluate_content_aware_predicate(
                                                    &resolved_spec,
                                                    &raw_content,
                                                    &transformed,
                                                )
                                            {
                                                Some(true) => {}
                                                Some(false) => {
                                                    blocked_branch_predicate = Some(
                                                        parser.semantic_predicate_debug_label(
                                                            &resolved_spec,
                                                        ),
                                                    );
                                                    branch_predicate_blocked = true;
                                                    break;
                                                }
                                                None => {}
                                            }
                                        }
                                        crate::ast_pipeline::SemanticRuntimeDirective::OpenScope(_)
                                        | crate::ast_pipeline::SemanticRuntimeDirective::CloseScope(_)
                                        | crate::ast_pipeline::SemanticRuntimeDirective::EmitFact(_)
                                        | crate::ast_pipeline::SemanticRuntimeDirective::Predicate(_) => {}
                                    }
                                }
                                let should_take = if branch_predicate_blocked {
                                    false
                                } else if #branch_policy_mode == "ordered" {
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
                                            "right" => current_branch_index > best_branch_index,
                                            "nonassoc" => {
                                                if current_branch_index != best_branch_index {
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
                                        "right" => current_branch_index > best_branch_index,
                                        "nonassoc" => {
                                            if current_branch_index != best_branch_index {
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
                                    best_branch_index = current_branch_index;
                                    best_branch = #branch_num;
                                    if semantic_capture_raw_for_post {
                                        best_raw_content = Some(raw_content.clone());
                                    }
                                    best_content = Some(transformed);
                                } else if branch_predicate_blocked && parser.logger_enabled {
                                    parser.logger.log_info(#filename, 0, &format!(
                                        "🚫 Branch {}/{} for rule '{}' rejected by branch predicate '{}' at position {}",
                                        #branch_num,
                                        #branch_count,
                                        #rule_name,
                                        blocked_branch_predicate
                                            .as_deref()
                                            .unwrap_or("<unknown>"),
                                        candidate_end
                                    ));
                                }
                            } else if parser.logger_enabled {
                                parser.logger.log_info(#filename, 0, &format!("❌ Branch {}/{} for rule '{}' failed at position {}", #branch_num, #branch_count, #rule_name, parser.position));
                            }
                        }
                    }
                });
            }

            Ok(quote! {
                // Multi-branch parsing logic (branch-policy guided)
                let parse_start = parser.position;
                let mut best_content: Option<ParseContent<'input>> = None;
                let mut best_raw_content: Option<ParseContent<'input>> = None;
                let mut best_end = parse_start;
                let mut best_priority: i64 = i64::MIN;
                let mut best_branch_index: usize = 0usize;
                let mut best_branch = 0usize;
                let mut nonassoc_tie = false;
                let mut result = ParseContent::Sequence(Vec::new());
                let deterministic_partition_effective_enabled = parser
                    .effective_deterministic_partition_enabled(#deterministic_partition_annotation_enabled);
                let deterministic_partition_effective_group = parser
                    .effective_deterministic_partition_group(#rule_name, #deterministic_partition_annotation_group);
                let deterministic_partition_offset = if deterministic_partition_effective_enabled {
                    parser.deterministic_partition_offset_runtime(
                        &deterministic_partition_effective_group,
                        #branch_count,
                    )
                } else {
                    0usize
                };
                let mut evaluation_order: Vec<usize> = (0..#branch_count).collect();
                if deterministic_partition_effective_enabled
                    && #branch_count > 1
                    && deterministic_partition_offset > 0
                {
                    evaluation_order.rotate_left(deterministic_partition_offset);
                }
                for branch_index in evaluation_order {
                    match branch_index {
                        #(#branch_attempt_arms,)*
                        _ => {}
                    }
                }

                if nonassoc_tie {
                    return Err(ParseError::Backtrack {
                        position: parse_start,
                    });
                } else if let Some(content) = best_content {
                    parser.position = best_end;
                    semantic_selected_branch_index = Some(best_branch);
                    if parser.logger_enabled {
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
                    semantic_raw_content = best_raw_content;
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
                        let effective_regex_pattern =
                            self.effective_regex_pattern(rule_name, token_value_str);
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
                                                        if parser.logger_enabled {
                                                            parser.logger.log_debug(
                                                                file!(),
                                                                line!(),
                                                                &format!(
                                                                    "🎯 Applied semantic transform: parsed '{}' to {}={}",
                                                                    matched_str,
                                                                    stringify!(#target_type),
                                                                    transformed
                                                                ),
                                                            );
                                                        }
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
                                                if parser.logger_enabled {
                                                    parser.logger.log_debug(
                                                        file!(),
                                                        line!(),
                                                        &format!(
                                                            "🎯 Applied semantic transform: raw expression '{}' to rule '{}': matched '{}'",
                                                            #expression,
                                                            #rule_name,
                                                            matched_str
                                                        ),
                                                    );
                                                }
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
            &[
                "stop_at_rule_boundary",
                "stop_on_rule_boundary",
                "line_delimited_sequence",
            ],
        );
        let stop_at_rule_boundary_on_break = if stop_at_rule_boundary {
            quote! {
                if parser.looks_like_rule_definition_boundary() {
                    break;
                }
            }
        } else {
            quote! {}
        };
        let stop_at_rule_boundary_on_error = if stop_at_rule_boundary {
            quote! {
                if parser.looks_like_rule_definition_boundary() {
                    return Err(ParseError::Backtrack {
                        position: parser.position,
                    });
                }
            }
        } else {
            quote! {}
        };

        match quantifier {
            "*" => Ok(quote! {
                let mut results = Vec::new();
                let mut last_position = parser.position;
                let mut iteration_count = 0;
                const MAX_ITERATIONS: usize = 10000; // Safety limit

                while iteration_count < MAX_ITERATIONS {
                    #stop_at_rule_boundary_on_break
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
                            if parser.logger_enabled {
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

                if iteration_count >= MAX_ITERATIONS && parser.logger_enabled {
                    parser.logger.log_warning(#filename, 0, &format!("⚠️ MAX ITERATIONS ({}) reached in quantifier", MAX_ITERATIONS));
                }

                let result = ParseContent::Quantified(results, "*")
            }),
            "+" => Ok(quote! {
                let mut results = Vec::new();
                let start_position = parser.position;

                #stop_at_rule_boundary_on_error

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
                    if parser.logger_enabled {
                        parser.logger.log_warning(#filename, 0, &format!("⚠️ ZERO-LENGTH FIRST MATCH in + quantifier at position {}", start_position));
                    }
                }

                // Additional matches are optional
                let mut last_position = parser.position;
                let mut iteration_count = 1;
                const MAX_ITERATIONS: usize = 10000;

                while iteration_count < MAX_ITERATIONS {
                    #stop_at_rule_boundary_on_break
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
                            if parser.logger_enabled {
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

                if iteration_count >= MAX_ITERATIONS && parser.logger_enabled {
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

    /// True when a branch's body parses to exactly one element — a single
    /// terminal, single rule reference, single quantified element, single
    /// lookahead, single Or, or a Sequence whose `elements` length is 1.
    /// False for Sequences with two or more elements.
    ///
    /// Used by the implicit `-> $1` default policy: branches with a
    /// single-element body get a synthetic Passthrough annotation when
    /// the grammar author didn't write one explicitly. Multi-element
    /// Sequences keep current behaviour (no transform) until the author
    /// declares one — defaulting to `$1` there would silently drop every
    /// element past the first (e.g. for `'(' expression ')'`, $1 = `'('`,
    /// not the expression payload the author meant).
    fn body_has_single_element(node: &ASTNode) -> bool {
        match node {
            ASTNode::Sequence { elements } => elements.len() <= 1,
            _ => true,
        }
    }

    /// Synthesize a `-> $1` BranchAnnotation when the branch body is a
    /// single element AND no explicit annotation was declared. The
    /// synthetic annotation is **codegen-only**: it's never written back
    /// to `branch_return_annotations` and never appears in the inventory
    /// artifact. The inventory contract continues to surface only
    /// grammar-author-written annotations.
    fn synthesize_default_passthrough_for_single_element_branch(
        body: &ASTNode,
    ) -> Option<BranchAnnotation> {
        if !Self::body_has_single_element(body) {
            return None;
        }
        Some(BranchAnnotation {
            annotation_type: "_pgen_default_passthrough_synthetic".to_string(),
            annotation_content: String::new(),
            parsed_ast: Some(crate::ast_pipeline::unified_return_ast::UnifiedReturnAST::PositionalRef {
                index: 1,
            }),
        })
    }

    // FUTURE: bolder default — for a multi-element Sequence with EXACTLY
    // ONE non-terminal (rule reference / quantified rule ref / grouped
    // non-terminal), default to `-> $N` where N is that non-terminal's
    // position. Example: `'(' expression ')'` → `-> $2`.
    //
    // Attempted on 2026-04-27 and reverted: the heuristic fires
    // PER-BRANCH and ignores whether sibling branches in the same Or
    // already carry an explicit annotation. For
    // `string_literal := ('"' string_content_double '"' | "'" string_content_single "'") -> {type: "string", value: $2}`
    // the codegen stores the explicit `{type: "string", ...}` on
    // branch_index 0 only. The bolder default then fires `-> $2` on
    // branch_index 1, producing a bare-string output while branch 0
    // produces a typed object — inconsistent shape per branch.
    //
    // Before re-introducing: add a "don't override sibling-branch intent"
    // rule (skip the default if any sibling Or-branch has an explicit
    // annotation OR if the rule itself carries a body-level annotation
    // that's stored at a single branch index).

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
                let _pgen_unparsed_return_annotation_warning: &str = #comment;
                let _ = _pgen_unparsed_return_annotation_warning;
                result.clone()
            })
        }
    }
    /// Utility function to unparse a ParseNode back to text for round-trip testing
    // pub fn unparse_node(&self, node: &ParseNode<'input>) -> String {
    //     format!("{:?}", node.content)
    // }

    fn generate_helper_methods(&self, filename: &str) -> TokenStream {
        let normalized_grammar_name = self
            .grammar_name
            .chars()
            .filter(|ch| ch.is_ascii_alphanumeric())
            .collect::<String>()
            .to_ascii_lowercase();
        let allow_layout_skip_for_terminals = normalized_grammar_name != "regex";
        let allow_layout_skip_for_regexes = !matches!(
            normalized_grammar_name.as_str(),
            "regex" | "systemverilogpreprocessor"
        );
        quote! {
            fn byte_window_lossy(&self, start: usize, end: usize) -> String {
                if start >= end || start >= self.input.len() {
                    return String::new();
                }
                let clamped_end = end.min(self.input.len());
                String::from_utf8_lossy(&self.input.as_bytes()[start..clamped_end]).to_string()
            }
            fn rule_profile_is_enabled(&self, allowed_profiles: &[&str]) -> bool {
                if allowed_profiles.is_empty() {
                    return true;
                }

                match self.grammar_profile.as_deref() {
                    Some(active) => allowed_profiles
                        .iter()
                        .any(|candidate| active.eq_ignore_ascii_case(candidate)),
                    None => true,
                }
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
            fn effective_deterministic_partition_enabled(
                &self,
                annotation_enabled: bool,
            ) -> bool {
                match self.deterministic_partition_runtime_mode {
                    DeterministicPartitionRuntimeMode::AnnotationDriven => annotation_enabled,
                    DeterministicPartitionRuntimeMode::ForceEnabled => true,
                    DeterministicPartitionRuntimeMode::ForceDisabled => false,
                }
            }
            fn effective_deterministic_partition_group(
                &self,
                rule_name: &str,
                annotation_group: &str,
            ) -> String {
                let trimmed = annotation_group.trim();
                if !trimmed.is_empty() {
                    trimmed.to_string()
                } else {
                    format!("rule.{}", rule_name)
                }
            }
            fn deterministic_partition_offset_runtime(
                &self,
                group_key: &str,
                branch_count: usize,
            ) -> usize {
                if branch_count <= 1 {
                    return 0;
                }

                let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
                for byte in group_key.as_bytes() {
                    hash ^= *byte as u64;
                    hash = hash.wrapping_mul(0x0000_0100_0000_01B3);
                }
                (hash as usize) % branch_count
            }
            fn record_coverage_target_event(
                &mut self,
                rule_name: &str,
                parse_start: usize,
                parse_end: usize,
                branch_index: Option<usize>,
                coverage_target_weight: u64,
                critical_path: bool,
            ) {
                if coverage_target_weight == 0 {
                    return;
                }

                self.coverage_target_events.push(CoverageTargetEvent {
                    rule_name: rule_name.to_string(),
                    parse_start,
                    parse_end,
                    branch_index,
                    coverage_target_weight,
                    critical_path,
                });
                *self
                    .coverage_target_rule_hits
                    .entry(rule_name.to_string())
                    .or_insert(0) += 1;
                if let Some(branch) = branch_index {
                    let branch_key = format!("{}::{}", rule_name, branch);
                    *self.coverage_target_branch_hits.entry(branch_key).or_insert(0) += 1;
                }

                if self.logger_enabled {
                    let marker = if critical_path {
                        "critical"
                    } else {
                        "target"
                    };
                    self.logger.log_info(#filename, 0, &format!(
                        "🎯 SC-10 parser instrumentation: rule='{}' branch={:?} weight={} kind={} span={}..{}",
                        rule_name,
                        branch_index,
                        coverage_target_weight,
                        marker,
                        parse_start,
                        parse_end
                    ));
                }
            }
            fn record_deterministic_partition_event(
                &mut self,
                rule_name: &str,
                parse_start: usize,
                parse_end: usize,
                enabled: bool,
                group_key: &str,
            ) {
                if !enabled {
                    return;
                }

                self.deterministic_partition_events.push(DeterministicPartitionEvent {
                    rule_name: rule_name.to_string(),
                    parse_start,
                    parse_end,
                    group_key: group_key.to_string(),
                });
                *self
                    .deterministic_partition_rule_hits
                    .entry(rule_name.to_string())
                    .or_insert(0) += 1;

                if self.logger_enabled {
                    self.logger.log_info(#filename, 0, &format!(
                        "🧭 SC-12 parser partition: rule='{}' group='{}' span={}..{}",
                        rule_name,
                        group_key,
                        parse_start,
                        parse_end
                    ));
                }
            }
            fn record_negative_case_failure(
                &mut self,
                rule_name: &str,
                parse_start: usize,
                failure_position: usize,
                negative: bool,
                error_kind: &str,
            ) {
                self.negative_case_events.push(NegativeCaseEvent {
                    rule_name: rule_name.to_string(),
                    parse_start,
                    failure_position,
                    negative,
                    error_kind: error_kind.to_string(),
                });
                *self
                    .negative_case_rule_hits
                    .entry(rule_name.to_string())
                    .or_insert(0) += 1;

                if self.logger_enabled {
                    let mode = if negative {
                        "near-invalid"
                    } else {
                        "invalid-case"
                    };
                    self.logger.log_info(#filename, 0, &format!(
                        "⚠️ SC-11 expected-failure path: rule='{}' mode={} start={} failure={} kind={}",
                        rule_name,
                        mode,
                        parse_start,
                        failure_position,
                        error_kind
                    ));
                }
            }
            fn recover_with_hints(
                &mut self,
                rule_name: &str,
                parse_start: usize,
                sync_tokens: &[&str],
                panic_until_tokens: &[&str],
                recover_budget: Option<usize>,
                recover_parse_budget: Option<usize>,
                recover_global_budget: Option<usize>,
            ) -> bool {
                if let Some(limit) = recover_budget {
                    let used = self.recovery_counts.get(rule_name).copied().unwrap_or(0);
                    if used >= limit {
                        if self.logger_enabled {
                            self.logger.log_warning(#filename, 0, &format!(
                                "🛟 Recovery budget exhausted for rule '{}': used={} limit={}",
                                rule_name,
                                used,
                                limit
                            ));
                        }
                        return false;
                    }
                }
                if let Some(limit) = recover_parse_budget {
                    if self.recovery_parse_count >= limit {
                        if self.logger_enabled {
                            self.logger.log_warning(#filename, 0, &format!(
                                "🛟 Parse-scope recovery budget exhausted for rule '{}': used={} limit={}",
                                rule_name,
                                self.recovery_parse_count,
                                limit
                            ));
                        }
                        return false;
                    }
                }
                if let Some(limit) = recover_global_budget {
                    if self.recovery_global_count >= limit {
                        if self.logger_enabled {
                            self.logger.log_warning(#filename, 0, &format!(
                                "🛟 Global recovery budget exhausted for rule '{}': used={} limit={}",
                                rule_name,
                                self.recovery_global_count,
                                limit
                            ));
                        }
                        return false;
                    }
                }

                let recovery_start = parse_start.min(self.input.len());
                let mut best: Option<(usize, usize, u8, String)> = None;

                for token in panic_until_tokens {
                    if token.is_empty() {
                        continue;
                    }
                    if let Some(pos) = self.find_token_from(recovery_start, token) {
                        let candidate = (pos, token.len(), 0u8, token.to_string());
                        let take_candidate = match &best {
                            None => true,
                            Some((best_pos, _best_len, best_priority, _best_token)) => {
                                pos < *best_pos
                                    || (pos == *best_pos && candidate.2 < *best_priority)
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
                        let candidate = (pos, token.len(), 1u8, token.to_string());
                        let take_candidate = match &best {
                            None => true,
                            Some((best_pos, _best_len, best_priority, _best_token)) => {
                                pos < *best_pos
                                    || (pos == *best_pos && candidate.2 < *best_priority)
                            }
                        };
                        if take_candidate {
                            best = Some(candidate);
                        }
                    }
                }

                if let Some((token_pos, token_len, token_priority, token_value)) = best {
                    let previous = self.position;
                    let token_end = token_pos.saturating_add(token_len).min(self.input.len());
                    let mut new_position = token_end;
                    if new_position <= previous && previous < self.input.len() {
                        new_position = previous + 1;
                    }
                    self.position = new_position.min(self.input.len());
                    let marker_kind = if token_priority == 0 {
                        RecoveryMarkerKind::PanicUntil
                    } else {
                        RecoveryMarkerKind::Sync
                    };
                    self.recovery_events.push(RecoveryEvent {
                        rule_name: rule_name.to_string(),
                        parse_start,
                        previous_position: previous,
                        new_position: self.position,
                        marker_kind,
                        marker_position: Some(token_pos),
                        marker_value: Some(token_value.clone()),
                    });
                    *self.recovery_counts.entry(rule_name.to_string()).or_insert(0) += 1;
                    self.recovery_parse_count += 1;
                    self.recovery_global_count += 1;

                    if self.logger_enabled {
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
                    self.recovery_events.push(RecoveryEvent {
                        rule_name: rule_name.to_string(),
                        parse_start,
                        previous_position: previous,
                        new_position: self.position,
                        marker_kind: RecoveryMarkerKind::EofFallback,
                        marker_position: None,
                        marker_value: None,
                    });
                    *self.recovery_counts.entry(rule_name.to_string()).or_insert(0) += 1;
                    self.recovery_parse_count += 1;
                    self.recovery_global_count += 1;
                    if self.logger_enabled {
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
            fn enforce_relational_requires(
                &self,
                rule_name: &str,
                root_content: &ParseContent<'input>,
                required_references: &[&str],
            ) -> ParseResult<()> {
                for reference in required_references {
                    let normalized = reference.trim();
                    if normalized.is_empty() {
                        continue;
                    }
                    let Some(value) = self.resolve_semantic_reference(root_content, normalized) else {
                        return Err(self.create_contextual_error(&format!(
                            "Semantic @requires contract failed for rule '{}': unresolved reference '{}'",
                            rule_name,
                            normalized
                        )));
                    };
                    if value.trim().is_empty() {
                        return Err(self.create_contextual_error(&format!(
                            "Semantic @requires contract failed for rule '{}': empty reference '{}'",
                            rule_name,
                            normalized
                        )));
                    }
                }

                Ok(())
            }
            fn evaluate_relational_expression(
                &self,
                root_content: &ParseContent<'input>,
                expression: &str,
            ) -> ParseResult<bool> {
                let normalized = expression.trim();
                if normalized.is_empty() {
                    return Err(self.create_contextual_error(
                        "Semantic relational expression cannot be empty",
                    ));
                }
                self.evaluate_relational_expression_inner(root_content, normalized)
            }
            fn evaluate_relational_expression_inner(
                &self,
                root_content: &ParseContent<'input>,
                expression: &str,
            ) -> ParseResult<bool> {
                let mut normalized = expression.trim();
                while self.semantic_encloses_full_parens(normalized) {
                    normalized = normalized[1..normalized.len() - 1].trim();
                }

                let disjuncts = self.split_semantic_top_level(normalized, "||");
                if disjuncts.len() > 1 {
                    for term in disjuncts {
                        if term.is_empty() {
                            continue;
                        }
                        if self.evaluate_relational_expression_inner(root_content, term)? {
                            return Ok(true);
                        }
                    }
                    return Ok(false);
                }

                let conjuncts = self.split_semantic_top_level(normalized, "&&");
                if conjuncts.len() > 1 {
                    for term in conjuncts {
                        if term.is_empty() {
                            continue;
                        }
                        if !self.evaluate_relational_expression_inner(root_content, term)? {
                            return Ok(false);
                        }
                    }
                    return Ok(true);
                }

                if let Some(rest) = normalized.strip_prefix('!') {
                    return Ok(!self.evaluate_relational_expression_inner(root_content, rest)?);
                }

                for operator in ["==", "!=", ">=", "<=", ">", "<"] {
                    if let Some((left, right)) =
                        self.split_semantic_top_level_once(normalized, operator)
                    {
                        return self.evaluate_relational_comparison(
                            root_content,
                            left,
                            operator,
                            right,
                        );
                    }
                }

                if self.semantic_reference_syntax(normalized) {
                    let value = self
                        .resolve_semantic_reference(root_content, normalized)
                        .ok_or_else(|| {
                            self.create_contextual_error(&format!(
                                "Semantic relational expression references unresolved capture '{}'",
                                normalized
                            ))
                        })?;
                    return Ok(Self::semantic_truthy(&value));
                }

                if let Some(unquoted) = Self::semantic_unquote(normalized) {
                    return Ok(Self::semantic_truthy(unquoted));
                }

                if let Ok(number) = normalized.parse::<f64>() {
                    return Ok(number != 0.0);
                }

                let lowered = normalized.to_ascii_lowercase();
                if lowered == "true" {
                    return Ok(true);
                }
                if lowered == "false" {
                    return Ok(false);
                }

                Ok(Self::semantic_truthy(normalized))
            }
            fn evaluate_relational_comparison(
                &self,
                root_content: &ParseContent<'input>,
                left: &str,
                operator: &str,
                right: &str,
            ) -> ParseResult<bool> {
                let lhs = self.resolve_relational_operand(root_content, left)?;
                let rhs = self.resolve_relational_operand(root_content, right)?;
                let lhs_numeric = lhs.parse::<f64>().ok();
                let rhs_numeric = rhs.parse::<f64>().ok();

                match operator {
                    "==" => {
                        if let (Some(a), Some(b)) = (lhs_numeric, rhs_numeric) {
                            Ok((a - b).abs() <= f64::EPSILON)
                        } else {
                            Ok(lhs == rhs)
                        }
                    }
                    "!=" => {
                        if let (Some(a), Some(b)) = (lhs_numeric, rhs_numeric) {
                            Ok((a - b).abs() > f64::EPSILON)
                        } else {
                            Ok(lhs != rhs)
                        }
                    }
                    ">" => {
                        if let (Some(a), Some(b)) = (lhs_numeric, rhs_numeric) {
                            Ok(a > b)
                        } else {
                            Ok(lhs > rhs)
                        }
                    }
                    ">=" => {
                        if let (Some(a), Some(b)) = (lhs_numeric, rhs_numeric) {
                            Ok(a >= b)
                        } else {
                            Ok(lhs >= rhs)
                        }
                    }
                    "<" => {
                        if let (Some(a), Some(b)) = (lhs_numeric, rhs_numeric) {
                            Ok(a < b)
                        } else {
                            Ok(lhs < rhs)
                        }
                    }
                    "<=" => {
                        if let (Some(a), Some(b)) = (lhs_numeric, rhs_numeric) {
                            Ok(a <= b)
                        } else {
                            Ok(lhs <= rhs)
                        }
                    }
                    _ => Err(self.create_contextual_error(&format!(
                        "Unsupported semantic comparison operator '{}'",
                        operator
                    ))),
                }
            }
            fn resolve_relational_operand(
                &self,
                root_content: &ParseContent<'input>,
                operand: &str,
            ) -> ParseResult<String> {
                let normalized = operand.trim();
                if normalized.is_empty() {
                    return Err(
                        self.create_contextual_error("Semantic relational operand cannot be empty")
                    );
                }

                if let Some(unquoted) = Self::semantic_unquote(normalized) {
                    return Ok(unquoted.to_string());
                }

                if self.semantic_reference_syntax(normalized) {
                    return self
                        .resolve_semantic_reference(root_content, normalized)
                        .ok_or_else(|| {
                            self.create_contextual_error(&format!(
                                "Semantic relational operand references unresolved capture '{}'",
                                normalized
                            ))
                        });
                }

                Ok(normalized.to_string())
            }
            fn resolve_semantic_reference(
                &self,
                root_content: &ParseContent<'input>,
                reference: &str,
            ) -> Option<String> {
                let normalized = reference.trim();
                if normalized.is_empty() {
                    return None;
                }

                let (core_reference, wants_len) = if let Some(stripped) = normalized.strip_suffix(".len") {
                    (stripped, true)
                } else {
                    (normalized, false)
                };

                let resolved = if core_reference.starts_with('$') {
                    let dollar_reference_body = core_reference[1..].trim();
                    let dollar_reference_is_positional = dollar_reference_body
                        .as_bytes()
                        .first()
                        .map(|byte| byte.is_ascii_digit())
                        .unwrap_or(false);
                    if dollar_reference_is_positional {
                        self.resolve_positional_semantic_reference(root_content, core_reference)
                    } else {
                        self.resolve_named_semantic_reference(root_content, dollar_reference_body)
                    }
                } else {
                    self.resolve_named_semantic_reference(root_content, core_reference)
                }?;

                if wants_len {
                    Some(resolved.chars().count().to_string())
                } else {
                    Some(resolved)
                }
            }
            fn resolve_positional_semantic_reference(
                &self,
                root_content: &ParseContent<'input>,
                reference: &str,
            ) -> Option<String> {
                let (index, path_segments) = self.parse_semantic_reference_segments(reference)?;
                let mut current_node = match root_content {
                    ParseContent::Sequence(elements) => elements.get(index.saturating_sub(1))?,
                    ParseContent::Alternative(node) => {
                        if index == 1 {
                            node.as_ref()
                        } else {
                            return None;
                        }
                    }
                    ParseContent::Quantified(elements, _) => elements.get(index.saturating_sub(1))?,
                    _ => return None,
                };

                for segment in path_segments {
                    current_node =
                        self.find_semantic_named_descendant(&current_node.content, segment)?;
                }

                self.semantic_node_scalar(current_node)
            }
            fn resolve_named_semantic_reference(
                &self,
                root_content: &ParseContent<'input>,
                reference: &str,
            ) -> Option<String> {
                let mut path_segments = reference
                    .split('.')
                    .map(str::trim)
                    .filter(|segment| !segment.is_empty());
                let first = path_segments.next()?;
                if !self.semantic_identifier(first) {
                    return None;
                }

                let mut current_node = self.find_semantic_named_descendant(root_content, first)?;
                for segment in path_segments {
                    if !self.semantic_identifier(segment) {
                        return None;
                    }
                    current_node =
                        self.find_semantic_named_descendant(&current_node.content, segment)?;
                }

                self.semantic_node_scalar(current_node)
            }
            fn parse_semantic_reference_segments<'a>(
                &self,
                reference: &'a str,
            ) -> Option<(usize, Vec<&'a str>)> {
                let normalized = reference.trim();
                if !normalized.starts_with('$') {
                    return None;
                }

                let bytes = normalized.as_bytes();
                let mut index_end = 1usize;
                while index_end < bytes.len() && bytes[index_end].is_ascii_digit() {
                    index_end += 1;
                }
                if index_end == 1 {
                    return None;
                }

                let index = normalized[1..index_end].parse::<usize>().ok()?;
                if index == 0 {
                    return None;
                }

                let mut segments = Vec::new();
                let suffix = normalized[index_end..].trim();
                if suffix.is_empty() {
                    return Some((index, segments));
                }
                if !suffix.starts_with('.') {
                    return None;
                }

                for segment in suffix[1..].split('.') {
                    let normalized_segment = segment.trim();
                    if normalized_segment.is_empty() || !self.semantic_identifier(normalized_segment)
                    {
                        return None;
                    }
                    segments.push(normalized_segment);
                }

                Some((index, segments))
            }
            fn find_semantic_named_descendant<'a>(
                &self,
                content: &'a ParseContent<'input>,
                target_name: &str,
            ) -> Option<&'a ParseNode<'input>> {
                match content {
                    ParseContent::Sequence(elements) | ParseContent::Quantified(elements, _) => {
                        for node in elements {
                            if node.rule_name == target_name {
                                return Some(node);
                            }
                            if let Some(found) =
                                self.find_semantic_named_descendant(&node.content, target_name)
                            {
                                return Some(found);
                            }
                        }
                        None
                    }
                    ParseContent::Alternative(node) => {
                        if node.rule_name == target_name {
                            Some(node)
                        } else {
                            self.find_semantic_named_descendant(&node.content, target_name)
                        }
                    }
                    _ => None,
                }
            }
            fn semantic_node_scalar(&self, node: &ParseNode<'input>) -> Option<String> {
                self.semantic_content_scalar(&node.content)
            }
            fn semantic_content_scalar(&self, content: &ParseContent<'input>) -> Option<String> {
                match content {
                    ParseContent::Terminal(value) => Some((*value).to_string()),
                    ParseContent::TransformedTerminal(value) => Some(value.clone()),
                    ParseContent::Json(value) => match value {
                        serde_json::Value::String(s) => Some(s.clone()),
                        serde_json::Value::Null => None,
                        other => Some(other.to_string()),
                    },
                    ParseContent::Alternative(node) => self.semantic_node_scalar(node),
                    ParseContent::Sequence(elements) | ParseContent::Quantified(elements, _) => {
                        let mut merged = String::new();
                        for node in elements {
                            if let Some(value) = self.semantic_node_scalar(node) {
                                merged.push_str(&value);
                            }
                        }
                        if merged.trim().is_empty() {
                            None
                        } else {
                            Some(merged)
                        }
                    }
                }
            }
            fn semantic_reference_syntax(&self, reference: &str) -> bool {
                let normalized = reference.trim();
                if normalized.is_empty() {
                    return false;
                }
                if normalized.starts_with('$') {
                    let dollar_reference_body = normalized[1..].trim();
                    if dollar_reference_body.is_empty() {
                        return false;
                    }
                    let dollar_reference_is_positional = dollar_reference_body
                        .as_bytes()
                        .first()
                        .map(|byte| byte.is_ascii_digit())
                        .unwrap_or(false);
                    if dollar_reference_is_positional {
                        return self.parse_semantic_reference_segments(normalized).is_some();
                    }

                    let mut segments = dollar_reference_body.split('.');
                    let Some(first) = segments.next() else {
                        return false;
                    };
                    if !self.semantic_identifier(first) {
                        return false;
                    }
                    return segments.all(|segment| self.semantic_identifier(segment));
                }

                let mut segments = normalized.split('.');
                let Some(first) = segments.next() else {
                    return false;
                };
                if !self.semantic_identifier(first) {
                    return false;
                }
                segments.all(|segment| self.semantic_identifier(segment))
            }
            fn semantic_identifier(&self, segment: &str) -> bool {
                let bytes = segment.as_bytes();
                let Some(first) = bytes.first() else {
                    return false;
                };
                if !(*first == b'_' || (*first as char).is_ascii_alphabetic()) {
                    return false;
                }
                bytes[1..]
                    .iter()
                    .all(|b| *b == b'_' || (*b as char).is_ascii_alphanumeric())
            }
            fn split_semantic_top_level<'a>(
                &self,
                expression: &'a str,
                separator: &str,
            ) -> Vec<&'a str> {
                if separator.is_empty() {
                    return vec![expression.trim()];
                }

                let bytes = expression.as_bytes();
                let separator_bytes = separator.as_bytes();
                if separator_bytes.is_empty() || bytes.len() < separator_bytes.len() {
                    return vec![expression.trim()];
                }

                let mut parts = Vec::new();
                let mut start = 0usize;
                let mut idx = 0usize;
                let mut depth = 0usize;
                let mut quote: Option<u8> = None;

                while idx < bytes.len() {
                    let current = bytes[idx];

                    if let Some(active_quote) = quote {
                        if current == active_quote && (idx == 0 || bytes[idx - 1] != b'\\') {
                            quote = None;
                        }
                        idx += 1;
                        continue;
                    }

                    match current {
                        b'"' | b'\'' => {
                            quote = Some(current);
                            idx += 1;
                            continue;
                        }
                        b'(' => {
                            depth += 1;
                            idx += 1;
                            continue;
                        }
                        b')' => {
                            depth = depth.saturating_sub(1);
                            idx += 1;
                            continue;
                        }
                        _ => {}
                    }

                    if depth == 0
                        && idx + separator_bytes.len() <= bytes.len()
                        && &bytes[idx..idx + separator_bytes.len()] == separator_bytes
                    {
                        parts.push(expression[start..idx].trim());
                        idx += separator_bytes.len();
                        start = idx;
                        continue;
                    }

                    idx += 1;
                }

                parts.push(expression[start..].trim());
                parts
            }
            fn split_semantic_top_level_once<'a>(
                &self,
                expression: &'a str,
                separator: &str,
            ) -> Option<(&'a str, &'a str)> {
                let pieces = self.split_semantic_top_level(expression, separator);
                if pieces.len() != 2 {
                    return None;
                }
                Some((pieces[0], pieces[1]))
            }
            fn semantic_encloses_full_parens(&self, expression: &str) -> bool {
                let normalized = expression.trim();
                if normalized.len() < 2
                    || !normalized.starts_with('(')
                    || !normalized.ends_with(')')
                {
                    return false;
                }

                let bytes = normalized.as_bytes();
                let mut depth = 0usize;
                let mut quote: Option<u8> = None;

                for (idx, current) in bytes.iter().enumerate() {
                    let current = *current;
                    if let Some(active_quote) = quote {
                        if current == active_quote && (idx == 0 || bytes[idx - 1] != b'\\') {
                            quote = None;
                        }
                        continue;
                    }

                    match current {
                        b'"' | b'\'' => {
                            quote = Some(current);
                        }
                        b'(' => depth += 1,
                        b')' => {
                            if depth == 0 {
                                return false;
                            }
                            depth -= 1;
                            if depth == 0 && idx + 1 < bytes.len() {
                                return false;
                            }
                        }
                        _ => {}
                    }
                }

                depth == 0 && quote.is_none()
            }
            fn semantic_unquote(value: &str) -> Option<&str> {
                let normalized = value.trim();
                if normalized.len() >= 2
                    && ((normalized.starts_with('"') && normalized.ends_with('"'))
                        || (normalized.starts_with('\'') && normalized.ends_with('\'')))
                {
                    return Some(&normalized[1..normalized.len() - 1]);
                }
                None
            }
            fn semantic_truthy(value: &str) -> bool {
                let normalized = value.trim();
                if normalized.is_empty() {
                    return false;
                }
                let lowered = normalized
                    .trim_matches('"')
                    .trim_matches('\'')
                    .trim()
                    .to_ascii_lowercase();
                !matches!(
                    lowered.as_str(),
                    "" | "false" | "0" | "no" | "off" | "none" | "null"
                )
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
                if #allow_layout_skip_for_terminals {
                    self.consume_layout_for_terminal(expected);
                }
                let start = self.position;
                let expected_bytes = expected.as_bytes();
                let end = start + expected_bytes.len();

                if self.logger_enabled {
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

                    if self.logger_enabled {
                        self.logger.log_success(#filename, 0, &format!("✅ Terminal '{}' matched, advanced to position {}", expected, end));
                    }

                    return Ok(&self.input[start..end]);
                }

                // Optim #4: cheap Backtrack on failure. Token-mismatch is the most common
                // failure mode (every alternation backtrack), and the rich ContextualError
                // (with byte_window_lossy + format!() + rule_stack collection) is almost
                // always discarded by the next OR retry. Construct that only when the
                // logger is enabled (debug builds, gates) — production parses skip the
                // allocation entirely. Caller's OR retry loop handles ParseError::Backtrack
                // identically to ContextualError.
                if self.logger_enabled {
                    let found_str = if self.position < self.input.len() {
                        let end = (self.position + expected_bytes.len()).min(self.input.len());
                        self.byte_window_lossy(self.position, end)
                    } else {
                        "<EOF>".to_string()
                    };
                    self.logger.log_error(#filename, 0, &format!("❌ Terminal '{}' failed at position {} - found '{}'", expected, start, found_str));
                }

                Err(ParseError::Backtrack { position: start })
            }

            fn match_regex(&mut self, pattern: &str, skip_leading_whitespace: bool) -> ParseResult<&'input str> {
                use std::cell::RefCell;
                use std::collections::HashMap;
                // Thread-local cache: each pattern is compiled once per thread,
                // and the resulting `regex::Regex` instance is **borrowed in
                // place** for every subsequent match. The instance carries its
                // own internal `Cache` pool (the lazy-DFA scratch space the
                // regex crate uses across searches); reusing the same instance
                // means that pool warms up once and is kept hot. The previous
                // shape cloned the cached `Regex` out of the closure on every
                // call; even though `Regex::clone()` is an O(1) Arc bump,
                // Cargo profiles of PGEN-RGX-0073 (samply, post-Optim-#8)
                // show 5.55% of self-time inside
                // `regex_automata::hybrid::dfa::Lazy::init_cache` —
                // i.e. the lazy DFA cache being re-initialized on first use
                // of each fresh borrow of the cloned instance, defeating the
                // regex crate's internal cache pool. Doing the search in
                // place fixes it.
                thread_local! {
                    static REGEX_CACHE: RefCell<HashMap<String, regex::Regex>> =
                        RefCell::new(HashMap::new());
                }

                // Phase 1: ensure pattern is compiled and cached, and (if
                // needed) compute `can_match_empty` for layout-skip. Both
                // reads can be done without exposing the Regex past the
                // closure boundary.
                let can_match_empty: bool = REGEX_CACHE.with(|cache| -> Result<bool, regex::Error> {
                    let mut cache = cache.borrow_mut();
                    if !cache.contains_key(pattern) {
                        let compiled = regex::Regex::new(pattern)?;
                        cache.insert(pattern.to_string(), compiled);
                    }
                    let re = cache.get(pattern).expect("just inserted");
                    if #allow_layout_skip_for_regexes {
                        Ok(re.find("").map(|m| m.start() == 0 && m.end() == 0).unwrap_or(false))
                    } else {
                        Ok(false)
                    }
                }).map_err(|e| self.create_contextual_error(&format!(
                    "Invalid regex pattern '{}': {}",
                    pattern, e
                )))?;

                if skip_leading_whitespace && #allow_layout_skip_for_regexes {
                    self.consume_layout_for_regex(can_match_empty);
                }

                let Some(haystack) = self.input.get(self.position..) else {
                    return Err(self.create_contextual_error("Parser position is not on a UTF-8 boundary"));
                };

                // Phase 2: do the actual `find` inside the cache closure
                // — no Regex clone, internal Cache pool stays hot. Returns
                // just the byte length of the matched prefix; we re-borrow
                // self.input outside the closure for the typed return.
                let match_end: Option<usize> = REGEX_CACHE.with(|cache| {
                    let cache = cache.borrow();
                    let re = cache.get(pattern).expect("compiled in phase 1");
                    re.find(haystack).filter(|m| m.start() == 0).map(|m| m.end())
                });

                if let Some(end_offset) = match_end {
                    let start = self.position;
                    self.position += end_offset;
                    if self.logger_enabled {
                        self.logger.log_success(#filename, 0, &format!(
                            "✅ Regex '{}' matched at position {} (len {})",
                            pattern, start, end_offset
                        ));
                    }
                    if let Some(slice) = self.input.get(start..self.position) {
                        return Ok(slice);
                    }
                    return Err(self.create_contextual_error("Regex matched invalid UTF-8 span"));
                }

                if self.logger_enabled {
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

                if self.logger_enabled {
                    self.logger.log_debug(#filename, 0, &format!("🔄 Starting speculative parse at position {}", saved_pos));
                }

                match f(self) {
                    Ok(result) => {
                        if self.logger_enabled {
                            self.logger.log_success(#filename, 0, &format!("🔄 Speculative parse succeeded, advanced to position {}", self.position));
                        }
                        Some(result)
                    }
                    Err(e) => {
                        // Backtrack
                        self.position = saved_pos;
                        self.recursion_guard.parse_stack.truncate(saved_stack_len);

                        if self.logger_enabled {
                            self.logger.log_warning(#filename, 0, &format!("🔙 Speculative parse failed with error '{:?}', backtracked to position {}", e, saved_pos));
                        }

                        None
                    }
                }
            }

            fn memoized_call<F>(
                &mut self,
                rule_id: RuleId,
                f: F,
            ) -> ParseResult<(ParseNode<'input>, Option<ParseContent<'input>>)>
            where
                F: FnOnce(&mut Self) -> ParseResult<(ParseNode<'input>, Option<ParseContent<'input>>)>,
            {
                let key = (rule_id, self.position);

                if let Some(entry) = self.memo.get(&key) {
                    if let Some(node) = &entry.result {
                        self.position = entry.end_pos;

                        if self.logger_enabled {
                            self.logger.log_info(#filename, 0, &format!("💾 Memo hit for rule {} at position {} - reusing cached result", rule_id, self.position));
                        }

                        return Ok((node.clone(), entry.raw_semantic_content.clone()));
                    } else {
                        if self.logger_enabled {
                            self.logger.log_warning(#filename, 0, &format!("💾 Memo miss for rule {} at position {} - cached failure", rule_id, self.position));
                        }
                        self.position = entry.end_pos;
                        return Err(ParseError::Backtrack {
                            position: entry.end_pos,
                        });
                    }
                }

                if self.logger_enabled {
                    self.logger.log_debug(#filename, 0, &format!("💾 Memo miss for rule {} at position {} - computing fresh result", rule_id, self.position));
                }

                let start_pos = key.1;
                let result = f(self);

                if let Ok((node, raw_semantic_content)) = &result {
                    self.memo.insert(
                        key,
                        MemoEntry {
                            result: Some(node.clone()),
                            raw_semantic_content: raw_semantic_content.clone(),
                            end_pos: node.span.end,
                        },
                    );
                    if self.logger_enabled {
                        self.logger.log_info(#filename, 0, &format!("💾 Memoized successful result for rule {} at position {}", rule_id, self.position));
                    }
                } else {
                    self.memo.insert(
                        key,
                        MemoEntry {
                            result: None,
                            raw_semantic_content: None,
                            end_pos: start_pos,
                        },
                    );
                    if self.logger_enabled {
                        self.logger.log_warning(#filename, 0, &format!("💾 Memoized failed result for rule {} at position {}", rule_id, self.position));
                    }
                }

                result
            }

            fn create_contextual_error(&self, message: &str) -> ParseError {
                let position = self.position;

                // Gather rule stack (rule names are &'static str — pointer copy, no allocation per entry)
                let rule_stack: Vec<&'static str> = self.recursion_guard.parse_stack.iter()
                    .map(|(rule, _)| *rule)
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

    fn generate_compiled_semantic_runtime_annotations_tokens(&self) -> Result<TokenStream> {
        let Some(annotations) = &self.annotations else {
            return Ok(quote! {
                crate::ast_pipeline::CompiledSemanticRuntimeAnnotations::default()
            });
        };

        let compiled = compile_semantic_runtime_annotations(annotations).map_err(|err| {
            anyhow::anyhow!(
                "Failed to compile semantic runtime annotations for generated parser '{}': {}",
                self.grammar_name,
                err
            )
        })?;

        if compiled.is_empty() {
            return Ok(quote! {
                crate::ast_pipeline::CompiledSemanticRuntimeAnnotations::default()
            });
        }

        let rule_entries = compiled.iter().map(|(rule_name, directives)| {
            let directive_tokens = directives
                .iter()
                .map(Self::generate_semantic_runtime_directive_tokens);
            quote! {
                directives_by_rule.insert(#rule_name.to_string(), vec![#(#directive_tokens),*]);
            }
        });
        let branch_rule_entries = compiled
            .branch_iter()
            .map(|(rule_name, branch_directives)| {
                let branch_directive_tokens = branch_directives.iter().map(|directives| {
                    let directive_tokens = directives
                        .iter()
                        .map(Self::generate_semantic_runtime_directive_tokens);
                    quote! {
                        vec![#(#directive_tokens),*]
                    }
                });
                quote! {
                    branch_directives_by_rule.insert(
                        #rule_name.to_string(),
                        vec![#(#branch_directive_tokens),*],
                    );
                }
            });

        Ok(quote! {
            {
                let mut directives_by_rule = std::collections::HashMap::new();
                let mut branch_directives_by_rule = std::collections::HashMap::new();
                #(#rule_entries)*
                #(#branch_rule_entries)*
                crate::ast_pipeline::CompiledSemanticRuntimeAnnotations::from_parts(
                    directives_by_rule,
                    branch_directives_by_rule,
                )
            }
        })
    }

    fn generate_semantic_runtime_directive_tokens(
        directive: &SemanticRuntimeDirective,
    ) -> TokenStream {
        match directive {
            SemanticRuntimeDirective::OpenScope(spec) => {
                let kind = Self::generate_semantic_scope_kind_tokens(&spec.kind);
                let name =
                    Self::generate_optional_semantic_runtime_value_tokens(spec.name.as_ref());
                quote! {
                    crate::ast_pipeline::SemanticRuntimeDirective::OpenScope(
                        crate::ast_pipeline::SemanticScopeSpec {
                            kind: #kind,
                            name: #name,
                        }
                    )
                }
            }
            SemanticRuntimeDirective::CloseScope(spec) => {
                let kind = match &spec.kind {
                    Some(kind) => {
                        let kind_tokens = Self::generate_semantic_scope_kind_tokens(kind);
                        quote! { Some(#kind_tokens) }
                    }
                    None => quote! { None },
                };
                let name =
                    Self::generate_optional_semantic_runtime_value_tokens(spec.name.as_ref());
                quote! {
                    crate::ast_pipeline::SemanticRuntimeDirective::CloseScope(
                        crate::ast_pipeline::SemanticCloseScopeSpec {
                            kind: #kind,
                            name: #name,
                        }
                    )
                }
            }
            SemanticRuntimeDirective::EmitFact(spec) => {
                let name = Self::generate_semantic_runtime_value_tokens(&spec.name);
                let attributes = spec
                    .attributes
                    .iter()
                    .map(Self::generate_unified_semantic_property_tokens);
                let kind = spec.kind.as_str();
                quote! {
                    crate::ast_pipeline::SemanticRuntimeDirective::EmitFact(
                        crate::ast_pipeline::SemanticFactSpec {
                            kind: #kind.to_string(),
                            name: #name,
                            attributes: vec![#(#attributes),*],
                        }
                    )
                }
            }
            SemanticRuntimeDirective::Predicate(spec) => {
                let name = spec.name.as_str();
                let args = spec
                    .args
                    .iter()
                    .map(Self::generate_unified_semantic_value_tokens);
                let phase = Self::generate_semantic_predicate_phase_tokens(spec.phase);
                let view = Self::generate_semantic_predicate_content_view_tokens(spec.view);
                quote! {
                    crate::ast_pipeline::SemanticRuntimeDirective::Predicate(
                        crate::ast_pipeline::SemanticPredicateSpec {
                            name: #name.to_string(),
                            args: vec![#(#args),*],
                            phase: #phase,
                            view: #view,
                        }
                    )
                }
            }
        }
    }

    fn generate_optional_semantic_runtime_value_tokens(
        value: Option<&SemanticRuntimeValue>,
    ) -> TokenStream {
        match value {
            Some(value) => {
                let value_tokens = Self::generate_semantic_runtime_value_tokens(value);
                quote! { Some(#value_tokens) }
            }
            None => quote! { None },
        }
    }

    fn generate_semantic_predicate_phase_tokens(
        phase: crate::ast_pipeline::SemanticPredicatePhase,
    ) -> TokenStream {
        match phase {
            crate::ast_pipeline::SemanticPredicatePhase::Pre => {
                quote! { crate::ast_pipeline::SemanticPredicatePhase::Pre }
            }
            crate::ast_pipeline::SemanticPredicatePhase::Branch => {
                quote! { crate::ast_pipeline::SemanticPredicatePhase::Branch }
            }
            crate::ast_pipeline::SemanticPredicatePhase::Post => {
                quote! { crate::ast_pipeline::SemanticPredicatePhase::Post }
            }
        }
    }

    fn generate_semantic_predicate_content_view_tokens(
        view: crate::ast_pipeline::SemanticPredicateContentView,
    ) -> TokenStream {
        match view {
            crate::ast_pipeline::SemanticPredicateContentView::Raw => {
                quote! { crate::ast_pipeline::SemanticPredicateContentView::Raw }
            }
            crate::ast_pipeline::SemanticPredicateContentView::Shaped => {
                quote! { crate::ast_pipeline::SemanticPredicateContentView::Shaped }
            }
        }
    }

    fn generate_semantic_runtime_value_tokens(value: &SemanticRuntimeValue) -> TokenStream {
        match value {
            SemanticRuntimeValue::String(text) => quote! {
                crate::ast_pipeline::SemanticRuntimeValue::String(#text.to_string())
            },
            SemanticRuntimeValue::Identifier(text) => quote! {
                crate::ast_pipeline::SemanticRuntimeValue::Identifier(#text.to_string())
            },
            SemanticRuntimeValue::RuleReference(text) => quote! {
                crate::ast_pipeline::SemanticRuntimeValue::RuleReference(#text.to_string())
            },
            SemanticRuntimeValue::Number(text) => quote! {
                crate::ast_pipeline::SemanticRuntimeValue::Number(#text.to_string())
            },
            SemanticRuntimeValue::Boolean(value) => quote! {
                crate::ast_pipeline::SemanticRuntimeValue::Boolean(#value)
            },
            SemanticRuntimeValue::Null => quote! {
                crate::ast_pipeline::SemanticRuntimeValue::Null
            },
        }
    }

    fn generate_semantic_scope_kind_tokens(kind: &SemanticScopeKind) -> TokenStream {
        match kind {
            SemanticScopeKind::Global => quote! { crate::ast_pipeline::SemanticScopeKind::Global },
            SemanticScopeKind::File => quote! { crate::ast_pipeline::SemanticScopeKind::File },
            SemanticScopeKind::Package => {
                quote! { crate::ast_pipeline::SemanticScopeKind::Package }
            }
            SemanticScopeKind::Class => quote! { crate::ast_pipeline::SemanticScopeKind::Class },
            SemanticScopeKind::Interface => {
                quote! { crate::ast_pipeline::SemanticScopeKind::Interface }
            }
            SemanticScopeKind::Type => quote! { crate::ast_pipeline::SemanticScopeKind::Type },
            SemanticScopeKind::Function => {
                quote! { crate::ast_pipeline::SemanticScopeKind::Function }
            }
            SemanticScopeKind::Task => quote! { crate::ast_pipeline::SemanticScopeKind::Task },
            SemanticScopeKind::Block => quote! { crate::ast_pipeline::SemanticScopeKind::Block },
            SemanticScopeKind::Custom(text) => quote! {
                crate::ast_pipeline::SemanticScopeKind::Custom(#text.to_string())
            },
        }
    }

    fn generate_unified_semantic_property_tokens(
        property: &UnifiedSemanticProperty,
    ) -> TokenStream {
        let key = property.key.as_str();
        let value = Self::generate_unified_semantic_value_tokens(&property.value);
        quote! {
            crate::ast_pipeline::UnifiedSemanticProperty {
                key: #key.to_string(),
                value: #value,
            }
        }
    }

    fn generate_unified_semantic_value_tokens(value: &UnifiedSemanticValue) -> TokenStream {
        match value {
            UnifiedSemanticValue::String(text) => quote! {
                crate::ast_pipeline::UnifiedSemanticValue::String(#text.to_string())
            },
            UnifiedSemanticValue::Number(text) => quote! {
                crate::ast_pipeline::UnifiedSemanticValue::Number(#text.to_string())
            },
            UnifiedSemanticValue::Boolean(value) => quote! {
                crate::ast_pipeline::UnifiedSemanticValue::Boolean(#value)
            },
            UnifiedSemanticValue::Null => quote! {
                crate::ast_pipeline::UnifiedSemanticValue::Null
            },
            UnifiedSemanticValue::Identifier(text) => quote! {
                crate::ast_pipeline::UnifiedSemanticValue::Identifier(#text.to_string())
            },
            UnifiedSemanticValue::RuleReference(text) => quote! {
                crate::ast_pipeline::UnifiedSemanticValue::RuleReference(#text.to_string())
            },
            UnifiedSemanticValue::Array(values) => {
                let values = values
                    .iter()
                    .map(Self::generate_unified_semantic_value_tokens);
                quote! {
                    crate::ast_pipeline::UnifiedSemanticValue::Array(vec![#(#values),*])
                }
            }
            UnifiedSemanticValue::Object(properties) => {
                let properties = properties
                    .iter()
                    .map(Self::generate_unified_semantic_property_tokens);
                quote! {
                    crate::ast_pipeline::UnifiedSemanticValue::Object(vec![#(#properties),*])
                }
            }
        }
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
            let name_matches = names
                .iter()
                .any(|candidate| name.eq_ignore_ascii_case(candidate));
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
                let payload = annotation.ast().payload_text().to_string();
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
            _ => extract_semantic_directive(annotation.ast().payload_text()),
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

    fn rule_coverage_target_policy(&self, rule_name: &str) -> SemanticCoverageTargetPolicy {
        let Some(annotations) = &self.annotations else {
            return SemanticCoverageTargetPolicy::default();
        };
        let Some(entries) = annotations.semantic_annotations.get(rule_name) else {
            return SemanticCoverageTargetPolicy::default();
        };

        let mut policy = SemanticCoverageTargetPolicy::default();
        for annotation in entries {
            let Some((name, payload)) = Self::semantic_directive_parts(annotation) else {
                continue;
            };
            match name.as_str() {
                "coverage_target" => {
                    if let Some(weight) = parse_semantic_coverage_target_weight(&payload) {
                        policy.coverage_target_weight = weight;
                    }
                }
                "critical_path" => {
                    if let Some(enabled) = parse_semantic_bool(&payload) {
                        policy.critical_path = enabled;
                    }
                }
                _ => {}
            }
        }

        policy
    }

    fn rule_negative_case_policy(&self, rule_name: &str) -> SemanticNegativeCasePolicy {
        let Some(annotations) = &self.annotations else {
            return SemanticNegativeCasePolicy::default();
        };
        let Some(entries) = annotations.semantic_annotations.get(rule_name) else {
            return SemanticNegativeCasePolicy::default();
        };

        let mut policy = SemanticNegativeCasePolicy::default();
        for annotation in entries {
            let Some((name, payload)) = Self::semantic_directive_parts(annotation) else {
                continue;
            };
            match name.as_str() {
                "invalid_case" => {
                    if let Some(enabled) = parse_semantic_bool(&payload) {
                        policy.invalid_case = enabled;
                    }
                }
                "negative" => {
                    if let Some(enabled) = parse_semantic_bool(&payload) {
                        policy.negative = enabled;
                    }
                }
                _ => {}
            }
        }

        if !policy.invalid_case {
            policy.negative = false;
        }
        policy
    }

    fn rule_deterministic_partition_policy(
        &self,
        rule_name: &str,
    ) -> SemanticDeterminismPartitionPolicy {
        let Some(annotations) = &self.annotations else {
            return SemanticDeterminismPartitionPolicy::default();
        };
        let Some(entries) = annotations.semantic_annotations.get(rule_name) else {
            return SemanticDeterminismPartitionPolicy::default();
        };

        let mut policy = SemanticDeterminismPartitionPolicy::default();
        for annotation in entries {
            let Some((name, payload)) = Self::semantic_directive_parts(annotation) else {
                continue;
            };
            match name.as_str() {
                "seed_group" => {
                    if let Some(label) = parse_semantic_group_label(&payload) {
                        policy.group_label = Some(label);
                    }
                }
                "deterministic_group" => {
                    if let Some(parsed) = parse_semantic_deterministic_group(&payload) {
                        policy.enabled = parsed.enabled;
                        if let Some(label) = parsed.group {
                            policy.group_label = Some(label);
                        }
                    }
                }
                _ => {}
            }
        }

        if !policy.enabled {
            policy.group_label = None;
            return policy;
        }
        if policy.group_label.is_none() {
            policy.group_label = Some(format!("rule.{}", rule_name));
        }

        policy
    }

    fn rule_profiles(&self, rule_name: &str) -> Vec<String> {
        let Some(annotations) = &self.annotations else {
            return Vec::new();
        };
        let Some(entries) = annotations.semantic_annotations.get(rule_name) else {
            return Vec::new();
        };

        let mut profiles = Vec::new();
        for annotation in entries {
            let Some((name, payload)) = Self::semantic_directive_parts(annotation) else {
                continue;
            };
            if name != "profiles" {
                continue;
            }
            if let Some(parsed) = parse_semantic_string_list(&payload) {
                profiles = parsed
                    .into_iter()
                    .map(|value| value.trim().to_ascii_lowercase())
                    .filter(|value| !value.is_empty())
                    .collect();
            }
        }

        profiles
    }

    fn deterministic_partition_offset(group_key: &str, branch_count: usize) -> usize {
        if branch_count <= 1 {
            return 0;
        }

        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for byte in group_key.as_bytes() {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(0x0000_0100_0000_01B3);
        }
        (hash as usize) % branch_count
    }

    fn rule_recovery_hints(
        &self,
        rule_name: &str,
    ) -> (
        bool,
        Vec<String>,
        Vec<String>,
        Option<usize>,
        Option<usize>,
        Option<usize>,
    ) {
        let Some(annotations) = &self.annotations else {
            return (false, Vec::new(), Vec::new(), None, None, None);
        };
        let Some(entries) = annotations.semantic_annotations.get(rule_name) else {
            return (false, Vec::new(), Vec::new(), None, None, None);
        };

        let mut recover_enabled = false;
        let mut sync_tokens = Vec::new();
        let mut panic_until_tokens = Vec::new();
        let mut recover_budget = None;
        let mut recover_parse_budget = None;
        let mut recover_global_budget = None;
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
                "recover_budget" => {
                    if let Some(parsed) = parse_semantic_nonnegative_usize(&payload) {
                        recover_budget = Some(parsed);
                    }
                }
                "recover_parse_budget" => {
                    if let Some(parsed) = parse_semantic_nonnegative_usize(&payload) {
                        recover_parse_budget = Some(parsed);
                    }
                }
                "recover_global_budget" => {
                    if let Some(parsed) = parse_semantic_nonnegative_usize(&payload) {
                        recover_global_budget = Some(parsed);
                    }
                }
                _ => {}
            }
        }
        (
            recover_enabled,
            sync_tokens,
            panic_until_tokens,
            recover_budget,
            recover_parse_budget,
            recover_global_budget,
        )
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

    fn rule_token_steering_policy(&self, rule_name: &str) -> SemanticTokenSteeringPolicy {
        let Some(annotations) = &self.annotations else {
            return SemanticTokenSteeringPolicy::default();
        };
        let Some(entries) = annotations.semantic_annotations.get(rule_name) else {
            return SemanticTokenSteeringPolicy::default();
        };

        let mut policy = SemanticTokenSteeringPolicy::default();
        for annotation in entries {
            let Some((name, payload)) = Self::semantic_directive_parts(annotation) else {
                continue;
            };
            match name.as_str() {
                "token_class" => {
                    if let Some(parsed) = parse_semantic_token_class(&payload) {
                        policy.token_class = Some(parsed);
                    }
                }
                "charset" => {
                    if let Some(pattern) = parse_semantic_charset(&payload) {
                        policy.charset_pattern = Some(pattern);
                    }
                }
                "pattern" => {
                    if let Some(pattern) = parse_semantic_pattern(&payload) {
                        policy.explicit_pattern = Some(pattern);
                    }
                }
                _ => {}
            }
        }

        policy
    }

    fn effective_regex_pattern(&self, rule_name: &str, grammar_pattern: &str) -> String {
        let mut effective = if self.grammar_name == "semantic_annotation"
            && rule_name == "identifier_literal"
            && grammar_pattern == "([a-zA-Z_][a-zA-Z0-9_]*)"
        {
            "([a-zA-Z_][a-zA-Z0-9_]*(?:\\.[a-zA-Z_][a-zA-Z0-9_]*)*)".to_string()
        } else {
            grammar_pattern.to_string()
        };

        let policy = self.rule_token_steering_policy(rule_name);
        if let Some(pattern) = policy.explicit_pattern {
            return pattern;
        }
        if let Some(pattern) = policy.charset_pattern {
            return pattern;
        }
        if let Some(token_class) = policy.token_class {
            return token_class.regex_pattern().to_string();
        }

        effective.shrink_to_fit();
        effective
    }

    fn rule_relational_constraints(&self, rule_name: &str) -> SemanticRelationalConstraintPolicy {
        let mut policy = SemanticRelationalConstraintPolicy::default();
        let Some(annotations) = &self.annotations else {
            return policy;
        };
        let Some(entries) = annotations.semantic_annotations.get(rule_name) else {
            return policy;
        };

        for annotation in entries {
            let Some((name, payload)) = Self::semantic_directive_parts(annotation) else {
                continue;
            };

            match name.as_str() {
                "constraint" => {
                    if let Some(parsed) = parse_semantic_constraint_expression(&payload) {
                        policy.constraint_expression = Some(parsed);
                    }
                }
                "requires" => {
                    if let Some(parsed) = parse_semantic_reference_list(&payload) {
                        policy.requires_references = parsed;
                    }
                }
                "implies" => {
                    if let Some(parsed) = parse_semantic_implication(&payload) {
                        policy.implication = Some(parsed);
                    }
                }
                _ => {}
            }
        }

        // Keep validator/runtime contract aligned: relational hints are inactive
        // unless @constraint is present.
        if policy.constraint_expression.is_none() {
            policy.requires_references.clear();
            policy.implication = None;
        }

        policy
    }

    fn semantic_relational_constraint_tokens(&self, rule_name: &str) -> TokenStream {
        let policy = self.rule_relational_constraints(rule_name);
        let Some(constraint_expression) = policy.constraint_expression else {
            return quote! {};
        };

        let requires_references = policy.requires_references;
        let implication_guard = if let Some((antecedent, consequent)) = policy.implication {
            quote! {
                let implication_antecedent = #antecedent;
                let implication_consequent = #consequent;
                if parser.evaluate_relational_expression(&result, implication_antecedent)?
                    && !parser.evaluate_relational_expression(&result, implication_consequent)?
                {
                    return Err(parser.create_contextual_error(&format!(
                        "Semantic implication failed for rule '{}': {} => {}",
                        #rule_name,
                        implication_antecedent,
                        implication_consequent
                    )));
                }
            }
        } else {
            quote! {}
        };

        quote! {
            parser.enforce_relational_requires(#rule_name, &result, &[#(#requires_references),*])?;

            let relational_constraint = #constraint_expression;
            if !parser.evaluate_relational_expression(&result, relational_constraint)? {
                return Err(parser.create_contextual_error(&format!(
                    "Semantic relational constraint failed for rule '{}': {}",
                    #rule_name,
                    relational_constraint
                )));
            }

            #implication_guard
        }
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

/// Try to localize a syn-parse failure by binary-search: find the largest
/// prefix of `rendered` that still parses as a `syn::File`. The byte at the
/// boundary points near where the broken token sits.
fn locate_syn_parse_boundary(rendered: &str) -> usize {
    use std::str::FromStr;
    let total = rendered.len();
    if total == 0 {
        return 0;
    }
    let mut lo = 0usize;
    let mut hi = total;
    let mut last_good = 0usize;
    // Coarse 20-step bisect to keep the diagnostic cheap; precision is
    // approximate by design.
    for _ in 0..20 {
        if hi - lo < 64 {
            break;
        }
        let mid = lo + (hi - lo) / 2;
        let safe_mid = (mid..=mid + 8.min(total - mid))
            .find(|i| rendered.is_char_boundary(*i))
            .unwrap_or(mid.min(total));
        let prefix = &rendered[..safe_mid];
        let parsed = proc_macro2::TokenStream::from_str(prefix)
            .ok()
            .and_then(|ts| syn::parse2::<syn::File>(ts).ok());
        if parsed.is_some() {
            last_good = safe_mid;
            lo = safe_mid;
        } else {
            hi = safe_mid;
        }
    }
    last_good
}

/// Render a window of the rendered TokenStream around the given byte offset
/// for inclusion in error messages. Surfaces the broken token shape inline
/// so the operator does not need to grep the dumped file by hand.
fn render_token_context(rendered: &str, byte: usize, window: usize) -> String {
    if rendered.is_empty() {
        return String::from("<empty>");
    }
    let lo = byte.saturating_sub(window);
    let hi = (byte + window).min(rendered.len());
    let lo = (lo..=lo + 8.min(window))
        .find(|i| rendered.is_char_boundary(*i))
        .unwrap_or(0);
    let hi = (hi..=hi.saturating_add(8))
        .find(|i| *i <= rendered.len() && rendered.is_char_boundary(*i))
        .unwrap_or(rendered.len());
    let snippet: String = rendered[lo..hi]
        .chars()
        .map(|ch| if ch == '\n' { ' ' } else { ch })
        .collect();
    format!(
        "...{}<<HERE@{}>>{}...",
        &snippet[..byte.saturating_sub(lo).min(snippet.len())],
        byte,
        &snippet[byte.saturating_sub(lo).min(snippet.len())..]
    )
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
                let logger = Box::new(crate::ast_pipeline::NoOpLogger);
                let mut parser = #parser_name::new(input, logger);
                let _ = parser.parse();
            }
        }
    }
}

#[cfg(test)]
mod semantic_usage_tests {
    use super::*;
    use crate::ast_pipeline::{
        ASTNode, ASTValue, Annotations, SemanticAnnotation, TokenValue, UnifiedSemanticAST,
        UnifiedSemanticProperty, UnifiedSemanticValue,
    };
    use std::collections::HashMap;
    use std::sync::OnceLock;

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
            inline_annotations: false,
            enable_debug: false,
        }
    }

    fn generator_with_named_semantic(
        rule_name: &str,
        directives: Vec<(&str, &str)>,
    ) -> AstBasedGenerator {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            rule_name.to_string(),
            directives
                .into_iter()
                .map(|(name, payload)| SemanticAnnotation::Named {
                    name: name.to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: payload.to_string(),
                    },
                })
                .collect(),
        );

        AstBasedGenerator {
            grammar_name: "usage_test".to_string(),
            entry_rule: None,
            logger: None,
            annotations: Some(annotations),
            branch_return_annotations: HashMap::new(),
            inline_annotations: false,
            enable_debug: false,
        }
    }

    fn structured_named_annotation(
        name: &str,
        canonical: &str,
        value: UnifiedSemanticValue,
    ) -> SemanticAnnotation {
        SemanticAnnotation::Named {
            name: name.to_string(),
            ast: UnifiedSemanticAST::Structured {
                canonical: canonical.to_string(),
                value,
            },
        }
    }

    fn pre_runtime_generator() -> AstBasedGenerator {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "package_declaration".to_string(),
            vec![structured_named_annotation(
                "predicate",
                "{ name: current_scope_is, args: [global] }",
                UnifiedSemanticValue::Object(vec![
                    UnifiedSemanticProperty {
                        key: "name".to_string(),
                        value: UnifiedSemanticValue::Identifier("current_scope_is".to_string()),
                    },
                    UnifiedSemanticProperty {
                        key: "args".to_string(),
                        value: UnifiedSemanticValue::Array(vec![UnifiedSemanticValue::Identifier(
                            "global".to_string(),
                        )]),
                    },
                ]),
            )],
        );

        AstBasedGenerator {
            grammar_name: "usage_test".to_string(),
            entry_rule: None,
            logger: None,
            annotations: Some(annotations),
            branch_return_annotations: HashMap::new(),
            inline_annotations: false,
            enable_debug: false,
        }
    }

    fn pre_runtime_rendered_parser() -> &'static str {
        static RENDERED: OnceLock<String> = OnceLock::new();
        RENDERED
            .get_or_init(|| {
                let generator = pre_runtime_generator();
                let mut grammar_tree = HashMap::new();
                grammar_tree.insert(
                    "package_declaration".to_string(),
                    token("quoted_string", "pkg"),
                );
                let rule_order = vec!["package_declaration".to_string()];
                generator
                    .generate_parser(&grammar_tree, &rule_order, "semantic_runtime_usage.rs")
                    .expect("parser generation should succeed")
            })
            .as_str()
    }

    fn post_runtime_generator() -> AstBasedGenerator {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "package_declaration".to_string(),
            vec![
                structured_named_annotation(
                    "emit_fact",
                    "{ kind: package_name, name: $1 }",
                    UnifiedSemanticValue::Object(vec![
                        UnifiedSemanticProperty {
                            key: "kind".to_string(),
                            value: UnifiedSemanticValue::Identifier("package_name".to_string()),
                        },
                        UnifiedSemanticProperty {
                            key: "name".to_string(),
                            value: UnifiedSemanticValue::RuleReference("$1".to_string()),
                        },
                    ]),
                ),
                structured_named_annotation(
                    "predicate",
                    "{ name: has_fact, args: [package_name, $1], phase: post }",
                    UnifiedSemanticValue::Object(vec![
                        UnifiedSemanticProperty {
                            key: "name".to_string(),
                            value: UnifiedSemanticValue::Identifier("has_fact".to_string()),
                        },
                        UnifiedSemanticProperty {
                            key: "args".to_string(),
                            value: UnifiedSemanticValue::Array(vec![
                                UnifiedSemanticValue::Identifier("package_name".to_string()),
                                UnifiedSemanticValue::RuleReference("$1".to_string()),
                            ]),
                        },
                        UnifiedSemanticProperty {
                            key: "phase".to_string(),
                            value: UnifiedSemanticValue::Identifier("post".to_string()),
                        },
                    ]),
                ),
                structured_named_annotation(
                    "predicate",
                    "{ name: content_kind_is, args: [terminal], phase: post, view: raw }",
                    UnifiedSemanticValue::Object(vec![
                        UnifiedSemanticProperty {
                            key: "name".to_string(),
                            value: UnifiedSemanticValue::Identifier("content_kind_is".to_string()),
                        },
                        UnifiedSemanticProperty {
                            key: "args".to_string(),
                            value: UnifiedSemanticValue::Array(vec![
                                UnifiedSemanticValue::Identifier("terminal".to_string()),
                            ]),
                        },
                        UnifiedSemanticProperty {
                            key: "phase".to_string(),
                            value: UnifiedSemanticValue::Identifier("post".to_string()),
                        },
                        UnifiedSemanticProperty {
                            key: "view".to_string(),
                            value: UnifiedSemanticValue::Identifier("raw".to_string()),
                        },
                    ]),
                ),
            ],
        );

        AstBasedGenerator {
            grammar_name: "usage_test".to_string(),
            entry_rule: None,
            logger: None,
            annotations: Some(annotations),
            branch_return_annotations: HashMap::new(),
            inline_annotations: false,
            enable_debug: false,
        }
    }

    fn post_runtime_rendered_parser() -> &'static str {
        static RENDERED: OnceLock<String> = OnceLock::new();
        RENDERED
            .get_or_init(|| {
                let generator = post_runtime_generator();
                let mut grammar_tree = HashMap::new();
                grammar_tree.insert(
                    "package_declaration".to_string(),
                    token("quoted_string", "pkg"),
                );
                let rule_order = vec!["package_declaration".to_string()];
                generator
                    .generate_parser(&grammar_tree, &rule_order, "semantic_runtime_post_usage.rs")
                    .expect("parser generation should succeed")
            })
            .as_str()
    }

    fn branch_predicate_generator() -> AstBasedGenerator {
        let mut annotations = Annotations::default();
        annotations.branch_semantic_annotations.insert(
            "statement_or_decl".to_string(),
            vec![
                Vec::new(),
                vec![structured_named_annotation(
                    "predicate",
                    "{ name: content_kind_is, args: [terminal], phase: branch, view: raw }",
                    UnifiedSemanticValue::Object(vec![
                        UnifiedSemanticProperty {
                            key: "name".to_string(),
                            value: UnifiedSemanticValue::Identifier("content_kind_is".to_string()),
                        },
                        UnifiedSemanticProperty {
                            key: "args".to_string(),
                            value: UnifiedSemanticValue::Array(vec![
                                UnifiedSemanticValue::Identifier("terminal".to_string()),
                            ]),
                        },
                        UnifiedSemanticProperty {
                            key: "phase".to_string(),
                            value: UnifiedSemanticValue::Identifier("branch".to_string()),
                        },
                        UnifiedSemanticProperty {
                            key: "view".to_string(),
                            value: UnifiedSemanticValue::Identifier("raw".to_string()),
                        },
                    ]),
                )],
            ],
        );

        AstBasedGenerator {
            grammar_name: "usage_test".to_string(),
            entry_rule: None,
            logger: None,
            annotations: Some(annotations),
            branch_return_annotations: HashMap::new(),
            inline_annotations: false,
            enable_debug: false,
        }
    }

    fn branch_predicate_rendered_parser() -> &'static str {
        static RENDERED: OnceLock<String> = OnceLock::new();
        RENDERED
            .get_or_init(|| {
                let generator = branch_predicate_generator();
                let mut grammar_tree = HashMap::new();
                grammar_tree.insert(
                    "statement_or_decl".to_string(),
                    ASTNode::Or {
                        alternatives: vec![
                            ASTNode::Sequence {
                                elements: vec![
                                    token("quoted_string", "typedef"),
                                    token("quoted_string", "pkg"),
                                ],
                            },
                            token("quoted_string", "pkg"),
                        ],
                    },
                );
                let rule_order = vec!["statement_or_decl".to_string()];
                generator
                    .generate_parser(&grammar_tree, &rule_order, "semantic_branch_usage.rs")
                    .expect("parser generation should succeed")
            })
            .as_str()
    }

    fn profile_guard_generator() -> AstBasedGenerator {
        generator_with_named_semantic(
            "package_declaration",
            vec![("profiles", "[\"sv_2017\", \"sv_2023\"]")],
        )
    }

    fn profile_guard_rendered_parser() -> &'static str {
        static RENDERED: OnceLock<String> = OnceLock::new();
        RENDERED
            .get_or_init(|| {
                let generator = profile_guard_generator();
                let mut grammar_tree = HashMap::new();
                grammar_tree.insert(
                    "package_declaration".to_string(),
                    token("quoted_string", "pkg"),
                );
                let rule_order = vec!["package_declaration".to_string()];
                generator
                    .generate_parser(&grammar_tree, &rule_order, "semantic_profile_usage.rs")
                    .expect("parser generation should succeed")
            })
            .as_str()
    }

    fn or_rule() -> ASTNode {
        ASTNode::Or {
            alternatives: vec![token("quoted_string", "L"), token("quoted_string", "R")],
        }
    }

    fn or_rule_three() -> ASTNode {
        ASTNode::Or {
            alternatives: vec![
                token("quoted_string", "L"),
                token("quoted_string", "M"),
                token("quoted_string", "R"),
            ],
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
            inline_annotations: false,
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
    fn semantic_usage_codegen_token_class_overrides_regex_atom_pattern() {
        let generator = generator_with_named_semantic("ident", vec![("token_class", "identifier")]);

        let logic = generator
            .generate_node_parsing_logic(&regex_atom("[0-9]+"), "ident", "semantic_usage.rs")
            .expect("regex atom logic generation should succeed");
        let rendered = logic.to_string();

        assert!(
            rendered.contains("[A-Za-z_][A-Za-z0-9_]*"),
            "token_class steering should replace grammar regex with token-class matcher, got: {}",
            rendered
        );
    }

    /// Regression test for the regex-grammar codegen drop. Before this fix,
    /// `generate_rule_method` only applied return-annotation transforms via
    /// `generate_or_logic`, so any rule whose top-level AST node was Sequence
    /// / Atom / Quantified / Lookahead silently dropped its return annotation.
    /// `grammars/regex.ebnf` declares `regex = pattern? -> {type: "regex", pattern: $1}`
    /// (Quantified root) and `piece = atom quantifier? -> {type: "piece", ...}`
    /// (Sequence root); neither was emitting a transform in the generated
    /// parser. This test pins the fix at the codegen level: a synthetic
    /// non-Or rule with a return annotation must produce a transform-emit
    /// step in the rendered parser source.
    #[test]
    fn return_annotation_on_non_or_root_rule_emits_transform_at_codegen() {
        use crate::ast_pipeline::unified_return_ast::UnifiedReturnAST;

        // Build a synthetic grammar: `r = atom -> {type: "x"}` (Atom root).
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert("r".to_string(), token("quoted_string", "v"));
        let rule_order = vec!["r".to_string()];

        // Build a return annotation matching the regex-grammar shape:
        // an object literal with a string field.
        let mut props = std::collections::HashMap::new();
        props.insert(
            "type".to_string(),
            Box::new(UnifiedReturnAST::StringLiteral {
                value: "x".to_string(),
            }),
        );
        let parsed_ast = UnifiedReturnAST::Object { properties: props };

        let mut annotations = Annotations::default();
        annotations.branch_return_annotations.insert(
            "r".to_string(),
            vec![Some(BranchAnnotation {
                annotation_type: "return_object".to_string(),
                annotation_content: "{type: \"x\"}".to_string(),
                parsed_ast: Some(parsed_ast),
            })],
        );

        let mut converted_branches: HashMap<String, Vec<Option<BranchAnnotation>>> = HashMap::new();
        for (rule, branches) in annotations.branch_return_annotations.iter() {
            converted_branches.insert(rule.clone(), branches.clone());
        }

        let mut generator = AstBasedGenerator::new("non_or_anno_test".to_string());
        generator.enable_debug = false;
        generator.annotations = Some(annotations);
        generator.branch_return_annotations = converted_branches;

        let rendered = generator
            .generate_parser(&grammar_tree, &rule_order, "non_or_anno_test.rs")
            .expect("parser generation should succeed");

        // The fix: rule `r` (Atom root) must now apply its object-literal
        // transform inline. The typed-carrier work emits
        // `ParseContent::Json(serde_json::Value::Object(...))`.
        assert!(
            rendered.contains("ParseContent :: Json (serde_json :: Value :: Object")
                || rendered.contains("ParseContent::Json(serde_json::Value::Object"),
            "non-Or rule with object-literal return annotation must emit typed Json/Object carrier; rendered did not contain it. snippet around fn parse_r: {}",
            rendered
                .lines()
                .skip_while(|l| !l.contains("fn parse_r"))
                .take(60)
                .collect::<Vec<_>>()
                .join("\n")
        );
        // The fix introduces a `let result = <transform>;` shadow rebind
        // after the parse logic. Check that the literal "x" key/value pair
        // ended up in the generated source for rule `r`.
        assert!(
            rendered.contains("\"type\""),
            "rendered source must contain the annotation's literal field name, got: {}",
            rendered
                .lines()
                .filter(|l| l.contains("type") || l.contains("parse_r"))
                .take(30)
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    /// Regression test for the semantic_annotation regen syn-parse error.
    /// `generate_return_transform` returns multi-statement tokens (warning
    /// let-bindings + final `result.clone()` expression) when annotation
    /// parsing fails (`parsed_ast: None`). The codegen-fix from commit
    /// `6ad4ffd` (apply rule-level transform for non-Or roots) and the
    /// single-branch path in `generate_or_logic` both wrap `#transform` with
    /// `let result = #transform;` / `result = #transform;`. Without an
    /// explicit block wrapper around `#transform`, the multi-statement form
    /// produces invalid syntax (`let result = let _warning = ...;` is a syn
    /// parse error: "expected `=`"). The fix is to emit `let result = {
    /// #transform };` so multi-statement transforms remain valid in either
    /// expression context.
    #[test]
    fn unparseable_annotation_falls_back_to_block_wrapped_warning_emit() {
        // Build a synthetic non-Or rule with an annotation whose text WILL
        // fail bootstrap parsing (random unparseable garbage), so
        // `generate_return_transform`'s parsed-ast-failed fallback fires
        // and emits its multi-statement warning shape.
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert("r".to_string(), token("quoted_string", "v"));
        let rule_order = vec!["r".to_string()];

        let mut annotations = Annotations::default();
        annotations.branch_return_annotations.insert(
            "r".to_string(),
            vec![Some(BranchAnnotation {
                annotation_type: "return_object".to_string(),
                // Intentionally unparseable: bare pipe + bare identifier
                // mimicking the EBNF-frontend-over-grab failure mode.
                annotation_content: "this is | not parseable !".to_string(),
                parsed_ast: None,
            })],
        );

        let mut converted_branches: HashMap<String, Vec<Option<BranchAnnotation>>> = HashMap::new();
        for (rule, branches) in annotations.branch_return_annotations.iter() {
            converted_branches.insert(rule.clone(), branches.clone());
        }

        let mut generator = AstBasedGenerator::new("unparseable_anno_test".to_string());
        generator.enable_debug = false;
        generator.annotations = Some(annotations);
        generator.branch_return_annotations = converted_branches;

        // The codegen must produce syntactically valid Rust even when the
        // annotation fell through to the warning fallback. If the rule-method
        // emit forgets the block wrapper, syn will reject the output and
        // generate_parser will return an error.
        let rendered = generator
            .generate_parser(&grammar_tree, &rule_order, "unparseable_anno_test.rs")
            .expect("parser generation must succeed even with unparseable annotation");

        // The rendered output must contain the warning text in a block form
        // (the multi-statement fallback wrapped by the defensive block).
        assert!(
            rendered.contains("WARNING")
                || rendered.contains("warning")
                || rendered.contains("_pgen_unparsed_return_annotation_warning"),
            "rendered must carry the failed-parse warning marker; got first 800 chars: {}",
            &rendered.chars().take(800).collect::<String>()
        );
        // Block wrapping presence: after `let result =` for the rule-method
        // post_parse_transform_tokens, the next non-whitespace token must be
        // `{` (open brace) — that's the wrapper that makes the multi-statement
        // transform a valid expression. Search anywhere in the rendered file.
        assert!(
            rendered.contains("let result = {") || rendered.contains("result = {"),
            "rendered must wrap #transform in a block (let result = {{ ... }})"
        );
    }

    #[test]
    fn phase_2_m1_typed_entry_emits_only_when_inline_annotations_flag_is_set() {
        // Phase 2 M1+M3-stage-1 contract: when AstBasedGenerator.inline_annotations
        // is true, the emitted parser carries:
        //   1. a per-rule `parse_<rule>_typed` method for every rule in `rule_order`,
        //      each returning `ParseResult<serde_json::Value>` and wrapping the
        //      legacy `parse_<rule>` plus `serde_json::to_value(&node)` (stage 1
        //      body; stage 2+ replaces with shape-typed emit per ASTNode shape);
        //   2. a `parse_full_<entry>_typed` entry-point that dispatches through
        //      the per-rule typed method and enforces end-of-input.
        // Default-off behavior is byte-unchanged: no typed method appears.
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "package_declaration".to_string(),
            token("quoted_string", "pkg"),
        );
        let rule_order = vec!["package_declaration".to_string()];

        // Flag off: legacy emit only.
        let mut generator_off = AstBasedGenerator::new("usage_test".to_string());
        generator_off.enable_debug = false;
        let rendered_off = generator_off
            .generate_parser(&grammar_tree, &rule_order, "phase_2_m1.rs")
            .expect("flag-off parser generation should succeed");
        assert!(
            !rendered_off.contains("parse_full_package_declaration_typed"),
            "flag-off emit must not include typed method, got: {}",
            rendered_off
        );
        assert!(
            !rendered_off.contains("parse_package_declaration_typed"),
            "flag-off emit must not include per-rule typed method, got: {}",
            rendered_off
        );

        // Flag on: legacy emit + parallel typed impl block with per-rule typed methods.
        let mut generator_on = AstBasedGenerator::new("usage_test".to_string());
        generator_on.enable_debug = false;
        generator_on.inline_annotations = true;
        let rendered_on = generator_on
            .generate_parser(&grammar_tree, &rule_order, "phase_2_m1.rs")
            .expect("flag-on parser generation should succeed");
        // The full-input typed entry is present.
        assert!(
            rendered_on.contains("pub fn parse_full_package_declaration_typed"),
            "flag-on emit must include typed entry method, got: {}",
            rendered_on
        );
        // The per-rule typed method is present for every rule.
        assert!(
            rendered_on.contains("pub fn parse_package_declaration_typed"),
            "flag-on emit must include per-rule typed method for every rule, got: {}",
            rendered_on
        );
        assert!(
            rendered_on.contains("ParseResult<serde_json::Value>")
                || rendered_on.contains("ParseResult < serde_json :: Value >"),
            "typed methods must return ParseResult<serde_json::Value>, got: {}",
            rendered_on
        );
        // The legacy method is preserved unchanged.
        assert!(
            rendered_on.contains("pub fn parse_full_package_declaration"),
            "flag-on emit must preserve legacy parse_full method, got: {}",
            rendered_on
        );
        // Stage 2 per-rule typed body for `ASTNode::Atom`(`quoted_string`)
        // calls `self.match_string` directly and returns the matched
        // literal as `Value::String` — the legacy `parse_<rule>` is NOT
        // invoked on this path, so `ParseNode` allocation is bypassed.
        assert!(
            rendered_on.contains("self.match_string(\"pkg\")")
                || rendered_on.contains("self.match_string (\"pkg\")"),
            "stage-2 typed body for quoted_string Atom must call self.match_string with the literal, got: {}",
            rendered_on
        );
        assert!(
            rendered_on.contains("serde_json::Value::String")
                || rendered_on.contains("serde_json :: Value :: String"),
            "stage-2 typed body for quoted_string Atom must return Value::String, got: {}",
            rendered_on
        );
        // Stage 2 typed body must NOT use the stage-1 fallback pattern
        // (`legacy parse + serde_json::to_value`). The legacy
        // `parse_<rule>` still appears in the rendered output because
        // `parse_full_<entry>` calls it on the legacy path; the assertion
        // must not match that. Check absence of the stage-1 fallback
        // marker instead, which is unique to the fallback body.
        assert!(
            !rendered_on.contains("Phase 2 M3 stage 1 typed serialization failed"),
            "stage-2 typed body for non-annotated quoted_string Atom must NOT fall back to stage-1 wrapper, got: {}",
            rendered_on
        );
        // The full-input typed entry dispatches through the per-rule typed entry.
        assert!(
            rendered_on.contains("self.parse_package_declaration_typed()"),
            "full-input typed entry must dispatch through per-rule typed method, got: {}",
            rendered_on
        );
    }

    #[test]
    fn generated_parser_runtime_contract_owns_semantic_runtime_fields() {
        let rendered = pre_runtime_rendered_parser();

        assert!(
            rendered.contains(
                "semantic_runtime_annotations: crate::ast_pipeline::CompiledSemanticRuntimeAnnotations"
            ),
            "generated parser should own compiled semantic runtime annotations, got: {}",
            rendered
        );
        assert!(
            rendered.contains("semantic_runtime_state: crate::ast_pipeline::SemanticRuntimeState"),
            "generated parser should own semantic runtime state, got: {}",
            rendered
        );
        assert!(
            rendered.contains(
                "self.semantic_runtime_state = crate::ast_pipeline::SemanticRuntimeState::new();"
            ),
            "parse() should reset semantic runtime state, got: {}",
            rendered
        );
        assert!(
            rendered.contains("memo: rustc_hash :: FxHashMap < (RuleId, usize), MemoEntry < 'input > >")
                || rendered.contains("memo: rustc_hash::FxHashMap<(RuleId, usize), MemoEntry<'input>>"),
            "generated parser should memoize rich entries instead of bare shaped nodes, got: {}",
            rendered
        );
        assert!(
            rendered.contains("CompiledSemanticRuntimeAnnotations::from_parts"),
            "generated parser constructor should embed compiled runtime annotations, got: {}",
            rendered
        );
    }

    #[test]
    fn generated_parser_runtime_contract_exposes_transaction_helpers() {
        let rendered = pre_runtime_rendered_parser();

        assert!(
            rendered.contains("pub fn semantic_runtime_transaction_for_rule"),
            "generated parser should expose a rule-transaction helper, got: {}",
            rendered
        );
        assert!(
            rendered.contains("pub fn with_semantic_runtime_rule_transaction"),
            "generated parser should expose the detached state transaction wrapper, got: {}",
            rendered
        );
        assert!(
            rendered
                .matches("with_semantic_runtime_rule_transaction")
                .count()
                >= 2,
            "generated parser should both define and use the semantic runtime transaction wrapper, got: {}",
            rendered
        );
    }

    #[test]
    fn generated_parser_runtime_contract_emits_pre_predicate_guard_flow() {
        let rendered = pre_runtime_rendered_parser();

        assert!(
            rendered.contains("evaluate_directive_predicate"),
            "generated parser should consult semantic runtime predicates before parsing, got: {}",
            rendered
        );
        assert!(
            rendered.contains("predicate_blocked"),
            "generated parser should track predicate failures inside the transaction wrapper, got: {}",
            rendered
        );
        assert!(
            rendered.contains("SemanticPredicatePhase::Pre"),
            "generated parser should embed typed predicate phase defaults, got: {}",
            rendered
        );
        assert!(
            rendered.contains("pre_predicates_for_rule"),
            "generated parser should use the explicit pre-predicate rule view, got: {}",
            rendered
        );
        assert!(
            rendered.contains("semantic_predicate_debug_label"),
            "generated parser should expose a helper for readable semantic predicate diagnostics, got: {}",
            rendered
        );
        assert!(
            rendered.contains("rejected by pre predicate"),
            "generated parser should log which pre predicate blocked a rule, got: {}",
            rendered
        );
    }

    #[test]
    fn generated_parser_runtime_contract_emits_post_predicate_content_flow() {
        let rendered = post_runtime_rendered_parser();

        assert!(
            rendered.contains("SemanticPredicatePhase::Post"),
            "generated parser should embed typed post-predicate directives when present, got: {}",
            rendered
        );
        assert!(
            rendered.contains("SemanticPredicateContentView::Raw"),
            "generated parser should embed typed predicate content-view defaults, got: {}",
            rendered
        );
        assert!(
            rendered.contains("post_predicates_for_rule"),
            "generated parser should use the explicit post-predicate rule view, got: {}",
            rendered
        );
        assert!(
            rendered.contains("needs_raw_post_capture_for_rule"),
            "generated parser should query raw post-predicate capture needs, got: {}",
            rendered
        );
        assert!(
            rendered.contains("semantic_capture_raw_for_post"),
            "generated parser should track whether raw post-predicate capture is required, got: {}",
            rendered
        );
        assert!(
            rendered.contains("semantic_raw_content"),
            "generated parser should preserve raw content when post/raw predicates require it, got: {}",
            rendered
        );
        assert!(
            rendered.contains("entry.raw_semantic_content.clone()"),
            "generated parser should restore memoized raw semantic content on cache hits, got: {}",
            rendered
        );
        assert!(
            rendered.contains("resolve_semantic_predicate_spec_against_content"),
            "generated parser should resolve post-predicate args against parse content, got: {}",
            rendered
        );
        assert!(
            rendered.contains("evaluate_content_aware_predicate"),
            "generated parser should evaluate resolved content-aware predicates after parse success, got: {}",
            rendered
        );
        assert!(
            rendered.contains("rejected by post predicate"),
            "generated parser should log which post predicate blocked a rule, got: {}",
            rendered
        );
    }

    #[test]
    fn generated_parser_runtime_contract_orders_effects_before_post_predicates() {
        let rendered = post_runtime_rendered_parser();

        assert!(
            rendered.contains("effect_directives_for_rule"),
            "generated parser should use the explicit effect-directive rule view, got: {}",
            rendered
        );
        assert!(
            rendered.contains("apply_semantic_runtime_effect_directive"),
            "generated parser should apply semantic runtime effects after parse success, got: {}",
            rendered
        );
        assert!(
            rendered.contains("resolve_semantic_runtime_value_against_content"),
            "generated parser should resolve runtime values against parse content, got: {}",
            rendered
        );
        assert!(
            rendered.contains("resolve_unified_semantic_value_against_content"),
            "generated parser should resolve structured semantic attribute values against parse content, got: {}",
            rendered
        );
        assert!(
            rendered.contains("coerce_semantic_runtime_scalar"),
            "generated parser should coerce resolved capture text into semantic runtime scalar values, got: {}",
            rendered
        );
        let effect_pos = rendered
            .find("apply_semantic_runtime_effect_directive(")
            .expect("generated parser should apply semantic runtime effects");
        let post_pos = rendered
            .find("resolve_semantic_predicate_spec_against_content(")
            .expect("generated parser should evaluate post predicates");
        assert!(
            effect_pos < post_pos,
            "generated parser should apply semantic effects before evaluating post predicates so post predicates can see same-rule facts/scopes, got: {}",
            rendered
        );
    }

    #[test]
    fn generated_parser_runtime_contract_refreshes_child_state_and_rolls_back_parent_failure() {
        let rendered = pre_runtime_rendered_parser();

        assert!(
            rendered.contains("std::mem::take("),
            "generated parser helper should detach semantic runtime state before parsing, got: {}",
            rendered
        );
        assert!(
            rendered.contains("let original_semantic_runtime_state"),
            "generated parser helper should preserve the original semantic runtime snapshot for rollback, got: {}",
            rendered
        );
        assert!(
            rendered.matches("std::mem::take(").count() >= 2,
            "generated parser should refresh semantic runtime state from child-rule commits before applying parent effects, got: {}",
            rendered
        );
        assert!(
            rendered.contains("if result.is_err()"),
            "generated parser should restore the original semantic runtime snapshot when the parent rule fails, got: {}",
            rendered
        );
    }

    #[test]
    fn generated_parser_branch_contract_embeds_branch_phase_view_and_rule_lookup() {
        let rendered = branch_predicate_rendered_parser();

        assert!(
            rendered.contains("SemanticPredicatePhase::Branch"),
            "generated parser should embed typed branch predicates when present, got: {}",
            rendered
        );
        assert!(
            rendered.contains("let mut branch_directives_by_rule"),
            "generated parser constructor should preserve compiled branch-local directives, got: {}",
            rendered
        );
        assert!(
            rendered.contains("branch_predicates_for_rule_branch"),
            "generated parser should consult the explicit branch-local branch-predicate view, got: {}",
            rendered
        );
    }

    #[test]
    fn generated_parser_branch_contract_uses_nullable_candidate_capture_resolution() {
        let rendered = branch_predicate_rendered_parser();

        assert!(
            rendered.contains("try_resolve_semantic_predicate_spec_against_content"),
            "generated parser should resolve branch-predicate args with nullable candidate-content support, got: {}",
            rendered
        );
        assert!(
            rendered.contains("branch_predicate_blocked"),
            "generated parser should treat unresolved branch captures as branch rejection rather than fatal parse error, got: {}",
            rendered
        );
        assert!(
            rendered.contains("blocked_branch_predicate"),
            "generated parser should retain the specific branch predicate that blocked a candidate, got: {}",
            rendered
        );
        assert!(
            rendered.contains("rejected by branch predicate '"),
            "generated parser should log which branch predicate rejected a candidate, got: {}",
            rendered
        );
    }

    #[test]
    fn generated_parser_branch_contract_reads_live_semantic_state_for_candidate_checks() {
        let rendered = branch_predicate_rendered_parser();

        assert!(
            rendered.contains(".semantic_runtime_state"),
            "generated parser should consult semantic runtime state during branch predicate evaluation, got: {}",
            rendered
        );
        assert!(
            rendered.contains("evaluate_content_aware_predicate"),
            "generated parser should evaluate branch predicates against semantic state without routing through effect application, got: {}",
            rendered
        );
    }

    #[test]
    fn semantic_usage_codegen_extracts_rule_profiles_from_named_directive() {
        let generator = profile_guard_generator();
        let profiles = generator.rule_profiles("package_declaration");
        assert_eq!(profiles, vec!["sv_2017".to_string(), "sv_2023".to_string()]);
    }

    #[test]
    fn generated_parser_profile_contract_emits_rule_profile_guard() {
        let rendered = profile_guard_rendered_parser();

        assert!(
            rendered.contains("if !self.rule_profile_is_enabled(&[\"sv_2017\", \"sv_2023\"])"),
            "generated parser should emit a rule-profile guard when @profiles is present, got: {}",
            rendered
        );
        assert!(
            rendered.contains("sv_2017"),
            "generated parser should embed the first allowed profile literal, got: {}",
            rendered
        );
        assert!(
            rendered.contains("sv_2023"),
            "generated parser should embed the second allowed profile literal, got: {}",
            rendered
        );
    }

    #[test]
    fn semantic_usage_codegen_charset_overrides_token_class_pattern() {
        let generator = generator_with_named_semantic(
            "ident",
            vec![("token_class", "identifier"), ("charset", "[A-F0-9]")],
        );

        let logic = generator
            .generate_node_parsing_logic(&regex_atom("[a-z]+"), "ident", "semantic_usage.rs")
            .expect("regex atom logic generation should succeed");
        let rendered = logic.to_string();

        assert!(
            rendered.contains("[A-F0-9]+"),
            "charset steering should override token_class matcher, got: {}",
            rendered
        );
    }

    #[test]
    fn semantic_usage_codegen_pattern_overrides_charset_and_token_class() {
        let generator = generator_with_named_semantic(
            "ident",
            vec![
                ("token_class", "identifier"),
                ("charset", "[A-F0-9]"),
                ("pattern", "^[A-Z]{2}$"),
            ],
        );

        let logic = generator
            .generate_node_parsing_logic(&regex_atom("[a-z]+"), "ident", "semantic_usage.rs")
            .expect("regex atom logic generation should succeed");
        let rendered = logic.to_string();

        assert!(
            rendered.contains("^[A-Z]{2}$"),
            "pattern steering should take precedence over charset/token_class, got: {}",
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
            inline_annotations: false,
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
            inline_annotations: false,
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
            inline_annotations: false,
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
                SemanticAnnotation::Named {
                    name: "recover_budget".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "3".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "recover_parse_budget".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "5".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "recover_global_budget".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "7".to_string(),
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
            inline_annotations: false,
            enable_debug: false,
        };

        let (
            recover_enabled,
            sync_tokens,
            panic_tokens,
            recover_budget,
            recover_parse_budget,
            recover_global_budget,
        ) = generator.rule_recovery_hints("stmt");
        assert!(recover_enabled);
        assert_eq!(sync_tokens, vec![";".to_string(), "end".to_string()]);
        assert_eq!(panic_tokens, vec!["}".to_string()]);
        assert_eq!(recover_budget, Some(3));
        assert_eq!(recover_parse_budget, Some(5));
        assert_eq!(recover_global_budget, Some(7));
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
                SemanticAnnotation::Named {
                    name: "recover_budget".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "2".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "recover_parse_budget".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "4".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "recover_global_budget".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "6".to_string(),
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
            inline_annotations: false,
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
        assert!(
            rendered.contains("2usize"),
            "recovery hook should carry typed recover_budget value, got: {}",
            rendered
        );
        assert!(
            rendered.contains("4usize"),
            "recovery hook should carry typed recover_parse_budget value, got: {}",
            rendered
        );
        assert!(
            rendered.contains("6usize"),
            "recovery hook should carry typed recover_global_budget value, got: {}",
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
            inline_annotations: false,
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
    fn semantic_usage_codegen_declares_structured_recovery_types() {
        let generator = AstBasedGenerator {
            grammar_name: "usage_test".to_string(),
            entry_rule: None,
            logger: None,
            annotations: None,
            branch_return_annotations: HashMap::new(),
            inline_annotations: false,
            enable_debug: false,
        };

        let rendered = generator.generate_types().to_string();
        assert!(
            rendered.contains("pub enum RecoveryMarkerKind"),
            "generated types should include RecoveryMarkerKind, got: {}",
            rendered
        );
        assert!(
            rendered.contains("pub struct RecoveryEvent"),
            "generated types should include RecoveryEvent, got: {}",
            rendered
        );
    }

    #[test]
    fn semantic_usage_codegen_emits_recovery_event_accessors() {
        let generator = AstBasedGenerator {
            grammar_name: "usage_test".to_string(),
            entry_rule: None,
            logger: None,
            annotations: None,
            branch_return_annotations: HashMap::new(),
            inline_annotations: false,
            enable_debug: false,
        };

        let rendered = generator.generate_parse_method("start").to_string();
        assert!(
            rendered.contains("pub fn recovery_events"),
            "parse method generation should expose recovery_events accessor, got: {}",
            rendered
        );
        assert!(
            rendered.contains("pub fn take_recovery_events"),
            "parse method generation should expose take_recovery_events accessor, got: {}",
            rendered
        );
        assert!(
            rendered.contains("pub fn recovery_event_count"),
            "parse method generation should expose recovery_event_count accessor, got: {}",
            rendered
        );
        assert!(
            rendered.contains("pub fn recovery_parse_count"),
            "parse method generation should expose recovery_parse_count accessor, got: {}",
            rendered
        );
        assert!(
            rendered.contains("pub fn recovery_global_count"),
            "parse method generation should expose recovery_global_count accessor, got: {}",
            rendered
        );
    }

    #[test]
    fn semantic_usage_codegen_records_recovery_events_in_helper_methods() {
        let generator = AstBasedGenerator {
            grammar_name: "usage_test".to_string(),
            entry_rule: None,
            logger: None,
            annotations: None,
            branch_return_annotations: HashMap::new(),
            inline_annotations: false,
            enable_debug: false,
        };

        let rendered = generator
            .generate_helper_methods("semantic_usage.rs")
            .to_string();
        assert!(
            rendered.contains("self . recovery_events . push"),
            "helper methods should record recovery events, got: {}",
            rendered
        );
        assert!(
            rendered.contains("RecoveryMarkerKind :: PanicUntil")
                && rendered.contains("RecoveryMarkerKind :: Sync")
                && rendered.contains("RecoveryMarkerKind :: EofFallback"),
            "helper methods should classify recovery markers, got: {}",
            rendered
        );
        assert!(
            rendered.contains("self . recovery_parse_count += 1")
                && rendered.contains("self . recovery_global_count += 1"),
            "helper methods should update parse/global recovery counters, got: {}",
            rendered
        );
    }

    #[test]
    fn semantic_usage_codegen_extracts_coverage_target_policy() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "stmt".to_string(),
            vec![
                SemanticAnnotation::Named {
                    name: "coverage_target".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "3".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "critical_path".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "true".to_string(),
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
            inline_annotations: false,
            enable_debug: false,
        };

        let policy = generator.rule_coverage_target_policy("stmt");
        assert_eq!(policy.coverage_target_weight, 3);
        assert!(policy.critical_path);
    }

    #[test]
    fn semantic_usage_codegen_emits_coverage_target_types_and_accessors() {
        let generator = AstBasedGenerator {
            grammar_name: "usage_test".to_string(),
            entry_rule: None,
            logger: None,
            annotations: None,
            branch_return_annotations: HashMap::new(),
            inline_annotations: false,
            enable_debug: false,
        };

        let types_rendered = generator.generate_types().to_string();
        assert!(
            types_rendered.contains("pub struct CoverageTargetEvent"),
            "generated types should include CoverageTargetEvent, got: {}",
            types_rendered
        );

        let parse_rendered = generator.generate_parse_method("start").to_string();
        assert!(
            parse_rendered.contains("pub fn coverage_target_events"),
            "parse method generation should expose coverage_target_events accessor, got: {}",
            parse_rendered
        );
        assert!(
            parse_rendered.contains("pub fn take_coverage_target_events"),
            "parse method generation should expose take_coverage_target_events accessor, got: {}",
            parse_rendered
        );
        assert!(
            parse_rendered.contains("pub fn coverage_target_rule_hits"),
            "parse method generation should expose coverage_target_rule_hits accessor, got: {}",
            parse_rendered
        );
        assert!(
            parse_rendered.contains("pub fn coverage_target_branch_hits"),
            "parse method generation should expose coverage_target_branch_hits accessor, got: {}",
            parse_rendered
        );
    }

    #[test]
    fn semantic_usage_codegen_emits_coverage_target_runtime_hooks_for_rules() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "stmt".to_string(),
            vec![
                SemanticAnnotation::Named {
                    name: "coverage_target".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "5".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "critical_path".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "true".to_string(),
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
            inline_annotations: false,
            enable_debug: false,
        };

        let method = generator
            .generate_rule_method(
                "stmt",
                &or_rule(),
                &["stmt".to_string()],
                "semantic_usage.rs",
            )
            .expect("rule method generation should succeed");
        let rendered = method.to_string();

        assert!(
            rendered.contains("record_coverage_target_event"),
            "rule method should emit SC-10 instrumentation recording hook, got: {}",
            rendered
        );
        assert!(
            rendered.contains("5u64") && rendered.contains("true"),
            "SC-10 hook should carry typed coverage_target/critical_path payloads, got: {}",
            rendered
        );
        assert!(
            rendered.contains("semantic_selected_branch_index = Some (best_branch)"),
            "OR rule should pass selected branch index into SC-10 instrumentation hook, got: {}",
            rendered
        );
    }

    #[test]
    fn semantic_usage_codegen_records_coverage_target_events_in_helper_methods() {
        let generator = AstBasedGenerator {
            grammar_name: "usage_test".to_string(),
            entry_rule: None,
            logger: None,
            annotations: None,
            branch_return_annotations: HashMap::new(),
            inline_annotations: false,
            enable_debug: false,
        };

        let rendered = generator
            .generate_helper_methods("semantic_usage.rs")
            .to_string();
        assert!(
            rendered.contains("fn record_coverage_target_event"),
            "helper methods should define SC-10 recording helper, got: {}",
            rendered
        );
        assert!(
            rendered.contains("if coverage_target_weight == 0"),
            "SC-10 helper should remain inactive when effective coverage_target weight is zero, got: {}",
            rendered
        );
        assert!(
            rendered.contains("self . coverage_target_events . push")
                && rendered.contains("self . coverage_target_rule_hits")
                && rendered.contains("self . coverage_target_branch_hits"),
            "SC-10 helper should record events and counters, got: {}",
            rendered
        );
    }

    #[test]
    fn semantic_usage_codegen_extracts_negative_case_policy() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "stmt".to_string(),
            vec![
                SemanticAnnotation::Named {
                    name: "invalid_case".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "true".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "negative".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "true".to_string(),
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
            inline_annotations: false,
            enable_debug: false,
        };

        let policy = generator.rule_negative_case_policy("stmt");
        assert!(policy.invalid_case);
        assert!(policy.negative);
    }

    #[test]
    fn semantic_usage_codegen_emits_negative_case_types_and_accessors() {
        let generator = AstBasedGenerator {
            grammar_name: "usage_test".to_string(),
            entry_rule: None,
            logger: None,
            annotations: None,
            branch_return_annotations: HashMap::new(),
            inline_annotations: false,
            enable_debug: false,
        };

        let types_rendered = generator.generate_types().to_string();
        assert!(
            types_rendered.contains("pub struct NegativeCaseEvent"),
            "generated types should include NegativeCaseEvent, got: {}",
            types_rendered
        );

        let parse_rendered = generator.generate_parse_method("start").to_string();
        assert!(
            parse_rendered.contains("pub fn negative_case_events"),
            "parse method generation should expose negative_case_events accessor, got: {}",
            parse_rendered
        );
        assert!(
            parse_rendered.contains("pub fn take_negative_case_events"),
            "parse method generation should expose take_negative_case_events accessor, got: {}",
            parse_rendered
        );
        assert!(
            parse_rendered.contains("pub fn negative_case_rule_hits"),
            "parse method generation should expose negative_case_rule_hits accessor, got: {}",
            parse_rendered
        );
    }

    #[test]
    fn semantic_usage_codegen_emits_negative_case_runtime_hooks_for_rules() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "stmt".to_string(),
            vec![
                SemanticAnnotation::Named {
                    name: "invalid_case".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "true".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "negative".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "true".to_string(),
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
            inline_annotations: false,
            enable_debug: false,
        };

        let method = generator
            .generate_rule_method(
                "stmt",
                &or_rule(),
                &["stmt".to_string()],
                "semantic_usage.rs",
            )
            .expect("rule method generation should succeed");
        let rendered = method.to_string();

        assert!(
            rendered.contains("record_negative_case_failure"),
            "rule method should emit SC-11 expected-failure runtime hook, got: {}",
            rendered
        );
        assert!(
            rendered.contains(
                "record_negative_case_failure (\"stmt\" , start_pos , self . position , true ,"
            ),
            "SC-11 hook should carry typed invalid_case/negative payload state, got: {}",
            rendered
        );
    }

    #[test]
    fn semantic_usage_codegen_records_negative_case_events_in_helper_methods() {
        let generator = AstBasedGenerator {
            grammar_name: "usage_test".to_string(),
            entry_rule: None,
            logger: None,
            annotations: None,
            branch_return_annotations: HashMap::new(),
            inline_annotations: false,
            enable_debug: false,
        };

        let rendered = generator
            .generate_helper_methods("semantic_usage.rs")
            .to_string();
        assert!(
            rendered.contains("fn record_negative_case_failure"),
            "helper methods should define SC-11 expected-failure recording helper, got: {}",
            rendered
        );
        assert!(
            rendered.contains("self . negative_case_events . push")
                && rendered.contains("self . negative_case_rule_hits"),
            "SC-11 helper should record events and per-rule hit counters, got: {}",
            rendered
        );
    }

    #[test]
    fn semantic_usage_codegen_extracts_deterministic_partition_policy() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "stmt".to_string(),
            vec![
                SemanticAnnotation::Named {
                    name: "seed_group".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "\"stable.alpha\"".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "deterministic_group".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "true".to_string(),
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
            inline_annotations: false,
            enable_debug: false,
        };

        let policy = generator.rule_deterministic_partition_policy("stmt");
        assert!(policy.enabled);
        assert_eq!(policy.group_label.as_deref(), Some("stable.alpha"));
    }

    #[test]
    fn semantic_usage_codegen_emits_deterministic_partition_types_and_accessors() {
        let generator = AstBasedGenerator {
            grammar_name: "usage_test".to_string(),
            entry_rule: None,
            logger: None,
            annotations: None,
            branch_return_annotations: HashMap::new(),
            inline_annotations: false,
            enable_debug: false,
        };

        let types_rendered = generator.generate_types().to_string();
        assert!(
            types_rendered.contains("pub struct DeterministicPartitionEvent"),
            "generated types should include DeterministicPartitionEvent, got: {}",
            types_rendered
        );
        assert!(
            types_rendered.contains("pub enum DeterministicPartitionRuntimeMode"),
            "generated types should expose DeterministicPartitionRuntimeMode, got: {}",
            types_rendered
        );

        let parse_rendered = generator.generate_parse_method("start").to_string();
        assert!(
            parse_rendered.contains("pub fn deterministic_partition_events"),
            "parse method generation should expose deterministic_partition_events accessor, got: {}",
            parse_rendered
        );
        assert!(
            parse_rendered.contains("pub fn take_deterministic_partition_events"),
            "parse method generation should expose take_deterministic_partition_events accessor, got: {}",
            parse_rendered
        );
        assert!(
            parse_rendered.contains("pub fn deterministic_partition_rule_hits"),
            "parse method generation should expose deterministic_partition_rule_hits accessor, got: {}",
            parse_rendered
        );
        assert!(
            parse_rendered.contains("pub fn deterministic_partition_runtime_mode"),
            "parse method generation should expose deterministic_partition_runtime_mode accessor, got: {}",
            parse_rendered
        );
        assert!(
            parse_rendered.contains("pub fn set_deterministic_partition_runtime_mode"),
            "parse method generation should expose runtime mode setter, got: {}",
            parse_rendered
        );
    }

    #[test]
    fn semantic_usage_codegen_emits_deterministic_partition_runtime_hooks_for_rules() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "stmt".to_string(),
            vec![
                SemanticAnnotation::Named {
                    name: "seed_group".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "\"stable.alpha\"".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "deterministic_group".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "true".to_string(),
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
            inline_annotations: false,
            enable_debug: false,
        };

        let method = generator
            .generate_rule_method(
                "stmt",
                &or_rule(),
                &["stmt".to_string()],
                "semantic_usage.rs",
            )
            .expect("rule method generation should succeed");
        let rendered = method.to_string();

        assert!(
            rendered.contains("record_deterministic_partition_event"),
            "rule method should emit SC-12 deterministic partition runtime hook, got: {}",
            rendered
        );
        assert!(
            rendered.contains("effective_deterministic_partition_enabled")
                && rendered.contains("effective_deterministic_partition_group"),
            "SC-12 hook should use runtime-effective partition policy helpers, got: {}",
            rendered
        );
    }

    #[test]
    fn semantic_usage_codegen_records_deterministic_partition_events_in_helper_methods() {
        let generator = AstBasedGenerator {
            grammar_name: "usage_test".to_string(),
            entry_rule: None,
            logger: None,
            annotations: None,
            branch_return_annotations: HashMap::new(),
            inline_annotations: false,
            enable_debug: false,
        };

        let rendered = generator
            .generate_helper_methods("semantic_usage.rs")
            .to_string();
        assert!(
            rendered.contains("fn record_deterministic_partition_event"),
            "helper methods should define SC-12 deterministic partition recorder, got: {}",
            rendered
        );
        assert!(
            rendered.contains("self . deterministic_partition_events . push")
                && rendered.contains("self . deterministic_partition_rule_hits"),
            "SC-12 helper should record partition events and per-rule hit counters, got: {}",
            rendered
        );
        assert!(
            rendered.contains("fn effective_deterministic_partition_enabled")
                && rendered.contains("fn effective_deterministic_partition_group")
                && rendered.contains("fn deterministic_partition_offset_runtime"),
            "SC-12 helper methods should expose runtime-effective enable/group/offset helpers, got: {}",
            rendered
        );
    }

    #[test]
    fn semantic_usage_codegen_uses_runtime_partition_order_for_ordered_or() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "expr".to_string(),
            vec![
                SemanticAnnotation::Named {
                    name: "branch_policy".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "ordered".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "seed_group".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "\"stable.beta\"".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "deterministic_group".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "true".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "priority".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "[11, 22, 33]".to_string(),
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
            inline_annotations: false,
            enable_debug: false,
        };

        let logic = generator
            .generate_node_parsing_logic(&or_rule_three(), "expr", "semantic_usage.rs")
            .expect("or-node logic generation should succeed");
        let rendered = logic.to_string();
        let has_rotate = rendered
            .contains("evaluation_order . rotate_left (deterministic_partition_offset)")
            || rendered
                .contains("evaluation_order . rotate_left ( deterministic_partition_offset )");

        assert!(
            rendered.contains("effective_deterministic_partition_enabled")
                && rendered.contains("effective_deterministic_partition_group")
                && rendered.contains("deterministic_partition_offset_runtime")
                && has_rotate
                && rendered.contains("for branch_index in evaluation_order")
                && rendered.contains("match branch_index"),
            "ordered OR logic should compute deterministic partition order at parser runtime, got rendered: {}",
            rendered
        );
    }

    #[test]
    fn semantic_usage_codegen_compares_recovery_candidates_without_moving_best_marker() {
        let generator = AstBasedGenerator {
            grammar_name: "usage_test".to_string(),
            entry_rule: None,
            logger: None,
            annotations: None,
            branch_return_annotations: HashMap::new(),
            inline_annotations: false,
            enable_debug: false,
        };

        let rendered = generator
            .generate_helper_methods("semantic_usage.rs")
            .to_string();
        assert!(
            rendered.contains("match & best"),
            "recovery candidate tie-break should borrow best marker (to avoid move errors), got: {}",
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
            inline_annotations: false,
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
            inline_annotations: false,
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
            inline_annotations: false,
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
            inline_annotations: false,
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
            inline_annotations: false,
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
    fn semantic_usage_codegen_parses_relational_constraint_policy() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "pair".to_string(),
            vec![
                SemanticAnnotation::Named {
                    name: "constraint".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "\"$1 != $2\"".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "requires".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "[\"$1\", \"$2\"]".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "implies".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "\"$1 => $2\"".to_string(),
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
            inline_annotations: false,
            enable_debug: false,
        };

        let policy = generator.rule_relational_constraints("pair");
        assert_eq!(policy.constraint_expression.as_deref(), Some("$1 != $2"));
        assert_eq!(
            policy.requires_references,
            vec!["$1".to_string(), "$2".to_string()]
        );
        assert_eq!(
            policy.implication,
            Some(("$1".to_string(), "$2".to_string()))
        );
    }

    #[test]
    fn semantic_usage_codegen_disables_relational_hints_without_constraint() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "pair".to_string(),
            vec![
                SemanticAnnotation::Named {
                    name: "requires".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "[\"$1\", \"$2\"]".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "implies".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "\"$1 => $2\"".to_string(),
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
            inline_annotations: false,
            enable_debug: false,
        };

        let policy = generator.rule_relational_constraints("pair");
        assert_eq!(policy.constraint_expression, None);
        assert!(
            policy.requires_references.is_empty(),
            "requires hints must remain inactive without @constraint"
        );
        assert_eq!(
            policy.implication, None,
            "implies hints must remain inactive without @constraint"
        );
    }

    #[test]
    fn semantic_usage_codegen_emits_runtime_relational_guards_for_rule_methods() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "pair".to_string(),
            vec![
                SemanticAnnotation::Named {
                    name: "constraint".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "\"$1 != $2\"".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "requires".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "[\"$1\", \"$2\"]".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "implies".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "\"$1 => $2\"".to_string(),
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
            inline_annotations: false,
            enable_debug: false,
        };

        let method = generator
            .generate_rule_method(
                "pair",
                &ASTNode::Sequence {
                    elements: vec![regex_atom("[A-Z]+"), regex_atom("[A-Z]+")],
                },
                &["pair".to_string()],
                "semantic_usage.rs",
            )
            .expect("rule method generation should succeed");
        let rendered = method.to_string();

        assert!(
            rendered.contains("enforce_relational_requires"),
            "expected runtime @requires enforcement hook, got: {}",
            rendered
        );
        assert!(
            rendered.contains("evaluate_relational_expression"),
            "expected runtime relational expression evaluation hook, got: {}",
            rendered
        );
        assert!(
            rendered.contains("Semantic implication failed"),
            "expected runtime implication failure diagnostic, got: {}",
            rendered
        );
    }

    #[test]
    fn semantic_usage_codegen_declares_relational_runtime_helper_methods() {
        let generator = AstBasedGenerator {
            grammar_name: "usage_test".to_string(),
            entry_rule: None,
            logger: None,
            annotations: None,
            branch_return_annotations: HashMap::new(),
            inline_annotations: false,
            enable_debug: false,
        };

        let rendered = generator
            .generate_helper_methods("semantic_usage.rs")
            .to_string();
        assert!(
            rendered.contains("fn evaluate_relational_expression"),
            "helper methods should include runtime relational evaluation support, got: {}",
            rendered
        );
        assert!(
            rendered.contains("fn resolve_semantic_reference"),
            "helper methods should include semantic reference resolution support, got: {}",
            rendered
        );
        assert!(
            rendered.contains("fn enforce_relational_requires"),
            "helper methods should include @requires contract enforcement support, got: {}",
            rendered
        );
    }

    #[test]
    fn semantic_usage_codegen_supports_named_dollar_semantic_references() {
        let generator = AstBasedGenerator {
            grammar_name: "usage_test".to_string(),
            entry_rule: None,
            logger: None,
            annotations: None,
            branch_return_annotations: HashMap::new(),
            inline_annotations: false,
            enable_debug: false,
        };

        let rendered = generator
            .generate_helper_methods("semantic_usage.rs")
            .to_string();

        assert!(
            rendered.contains("dollar_reference_body")
                && rendered.contains("dollar_reference_is_positional"),
            "helper methods should split named $rule_name references from positional $1 references explicitly, got: {}",
            rendered
        );
        assert!(
            rendered.contains("resolve_named_semantic_reference")
                && rendered.contains("core_reference [1 ..] . trim ()"),
            "helper methods should route named $rule_name references through named descendant resolution, got: {}",
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
