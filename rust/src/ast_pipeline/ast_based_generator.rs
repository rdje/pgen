// AST-Based Parser Generator using syn and quote
// This module replaces string concatenation with proper AST manipulation
// GUARANTEES: No unbalanced braces, no syntax errors, type-safe code generation

use super::Logger;
use crate::ast_pipeline::{
    ASTNode, ASTValue, Annotations, BranchAnnotation, LayoutSensitivity,
    SemanticAnnotation, SemanticAssociativity, SemanticBranchPolicy, SemanticRuntimeDirective,
    SemanticRuntimeValue, SemanticScopeKind, SemanticTokenClass, SemanticValueConstraints,
    TokenValue, UnifiedSemanticAST, UnifiedSemanticProperty, UnifiedSemanticValue,
    ast_return_transform::AstReturnTransformer, compile_default_profile,
    compile_layout_sensitivity, compile_profile_aliases,
    compile_semantic_runtime_annotations,
    parse_canonical_transform_expression,
    parse_semantic_bool, parse_semantic_charset,
    parse_semantic_constraint_expression,
    parse_semantic_implication,
    parse_semantic_nonnegative_usize,
    parse_quantifier_bounds, parse_semantic_pattern, parse_semantic_reference_list,
    parse_semantic_string_list, parse_semantic_token_class,
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

/// The per-introducer comment-arm suppression decision for the two layout skippers
/// (`consume_layout_for_terminal` / `consume_layout_for_regex`).
///
/// Codegen emits a comment-skip arm for an introducer (`#` / `//` / `/*`) **only** when the grammar
/// does NOT claim that introducer as a non-comment token (GRAMMAR-WELLFORMED.H.11.5 — see
/// [`AstBasedGenerator::grammar_claims_introducer_as_non_comment`]). A `true` field means the grammar
/// DOES claim the introducer, so its layout arm is **suppressed** and those bytes must be matched
/// structurally instead of skipped as trivia.
///
/// This is the single source of truth shared by codegen (which emits the arms) and the parse-harness
/// interpreter (`PARSE-HARNESS.5.2`, which must skip layout byte-identically to the generated parser).
#[derive(Debug, Clone, Copy)]
pub(crate) struct CommentArmSuppression {
    /// The grammar claims `#` as a non-comment token ⇒ the `#`-to-EOL layout arm is suppressed.
    pub claims_hash: bool,
    /// The grammar claims `//` as a non-comment token ⇒ the `//`-to-EOL layout arm is suppressed.
    pub claims_line_comment: bool,
    /// The grammar claims `/*` as a non-comment token ⇒ the `/* */` block-comment layout arm is suppressed.
    pub claims_block_comment: bool,
}

/// Compute a grammar's [`CommentArmSuppression`] — the exact per-introducer arm-emission decision the
/// shipped codegen makes for the layout skippers — **without** running codegen, so the parse-harness
/// interpreter (`PARSE-HARNESS.5.2`) can skip layout byte-identically to the generated parser.
///
/// This reconstructs codegen's *minimal relevant* generator config so it calls codegen's own predicate
/// with the same inputs: `AstBasedGenerator::new(snake_to_pascal(grammar_name))` sets `self.grammar_name`
/// exactly as the real generation path does (`ast_generator_direct.rs:109-110`), and `annotations` is
/// set exactly as `generator.annotations = Some(annotations.clone())` there (`:177`). The predicate
/// `grammar_claims_introducer_as_non_comment` reads only those two fields (via `effective_regex_pattern`
/// → `rule_token_steering_policy`), so the result is provably identical to what codegen emits — reusing
/// codegen's kernel means the interpreter can never drift from the generated parser.
pub(crate) fn comment_arm_suppression_for_grammar(
    grammar_name: &str,
    grammar_tree: &HashMap<String, ASTNode>,
    annotations: Option<&Annotations>,
) -> CommentArmSuppression {
    let mut generator =
        AstBasedGenerator::new(crate::ast_pipeline::ast_generator_direct::snake_to_pascal(
            grammar_name,
        ));
    generator.annotations = annotations.cloned();
    generator.comment_arm_suppression(grammar_tree)
}

/// AST-based generator that produces guaranteed syntactically correct Rust code
pub struct AstBasedGenerator {
    pub grammar_name: String,
    pub entry_rule: Option<String>,
    pub logger: Option<Box<dyn Logger>>,
    pub annotations: Option<Annotations>,
    pub branch_return_annotations: HashMap<String, Vec<Option<BranchAnnotation>>>,
    pub enable_debug: bool,
    /// Phase 2 M1 toggle. When true, the generator emits a typed entry-point
    /// skeleton — `parse_full_<entry>_typed` returning
    /// `ParseResult<serde_json::Value>` alongside the existing
    /// `parse_full_<entry>` returning `ParseResult<ParseNode>`. The skeleton
    /// body is a passthrough wrapper around the legacy method + `serde_json::to_value`;
    /// it does NOT inline anything per-rule and does NOT enable annotation support
    /// (which is always-on regardless of this flag — `@predicate`, `@emit_fact`,
    /// `@semantic_value`, and `-> {...}` return annotations all fire whether this
    /// flag is set or not). Default false: generator emit is unchanged from
    /// prior behavior. (PARSER-NEUTRALITY.1: the per-grammar "parser hook"
    /// mechanism that once accompanied this flag is REMOVED by director
    /// ruling — the pipeline has no per-parser extension points.)
    pub emit_typed_entry_skeleton: bool,
    /// REGEX-SELF-HOSTING.6a: set true during codegen iff any generated rule method emits a
    /// `match_regex` call (i.e. the grammar has ≥1 `/.../` regex literal). When false, the
    /// `match_regex` helper + `use regex::Regex` import are ELIDED so a fully-literal grammar (regex)
    /// does not link Rust's `regex` crate. Interior-mutable so it can be set behind `&self` during
    /// codegen; computed once after the rule methods are generated, before imports/helpers are emitted.
    pub uses_match_regex: std::cell::Cell<bool>,
    /// RGX-0078.5.c.2 — snapshot of the normalized (post-LR-elimination) gen-AST
    /// `grammar_tree`, taken at the `&self` generation entry
    /// (`generate_parser` / `generate_parser_tokens`) so `generate_or_logic` can
    /// compute per-branch FIRST-set first-byte prune guards WITHOUT threading
    /// `grammar_tree` through the whole `generate_*` chain. Interior-mutable to be
    /// settable behind `&self` (mirrors `uses_match_regex`). Empty until an entry
    /// populates it; the guard is emitted ONLY when it is populated, so a direct
    /// `generate_node_parsing_logic` / `generate_or_logic` unit-test call (which
    /// never populates it) resolves every rule reference as `unresolved` and emits
    /// no guard — byte-identical to the pre-feature codegen.
    pub first_set_grammar_tree: std::cell::RefCell<HashMap<String, ASTNode>>,
    /// RGX-0078.5.i.3 (P2) — the compiled runtime-annotation table for codegen-time
    /// ANALYSIS queries (the degenerate-dispatch gate (e): branch-phase predicates),
    /// compiled lazily ONCE per generation from `self.annotations`. `None` inside the
    /// cell = compilation failed — treated as "has predicates" (blocks dispatch), the
    /// sound default; a failing compile aborts generation anyway in
    /// `generate_compiled_semantic_runtime_annotations_tokens`. Distinct from the
    /// EMITTED table (which that method produces) — this one is never emitted.
    pub analysis_runtime_annotations:
        std::cell::OnceCell<Option<crate::ast_pipeline::CompiledSemanticRuntimeAnnotations>>,
    /// RGX-0078.5.i.4 (P1a) — the inline-EMISSION decision set: rules whose call
    /// sites receive the rule body inline under `inlined_frame_call` (memo
    /// preserved, per-frame observability verbatim). Computed ONCE per generation
    /// from the SHARED census decision function
    /// (`fusibility_census::compute_inline_decisions` — gates (a)–(d) + the
    /// measured code-size budget), so the census's `INLINE-DECISIONS` report and
    /// the emission cannot drift. `None` inside the cell = no inlining (analysis
    /// annotation-table compile failure, or zero decided rules) — the sound
    /// default, and the state for direct `generate_*` unit-test calls that never
    /// populate it (those stay byte-identical to the pre-P1a emission).
    pub inline_decided_rules: std::cell::OnceCell<Option<HashSet<String>>>,
    /// RGX-0078.5.i.4 (P1a) — codegen-time re-entry guard for the inline-emission
    /// walk. The decided subgraph is provably acyclic (gate (a)); re-entry here
    /// means the shared verdict and the emission disagree — a loud hard error,
    /// never an infinite emission recursion.
    pub inline_emission_stack: std::cell::RefCell<Vec<String>>,
    /// RGX-0078.5.i.7 (D2-A) — the CASCADE emission plan: the acyclic-sub-region
    /// partition (sub-roots with the observability-twin dispatch, internal rules as
    /// fused `cascade_<rule>` functions) plus the per-site C3-B effect-target set.
    /// Computed ONCE per generation from the SHARED census plan function
    /// (`fusibility_census::compute_cascade_emission_plan`), so the census's
    /// `CASCADE-PLAN` report and the emission cannot drift. `None` inside the cell =
    /// no fused graph (analysis annotation-table compile failure, or an empty fused
    /// set) — the sound default, and the state for direct `generate_*` unit-test
    /// calls that never populate it (those stay byte-identical to the pre-D2-A
    /// emission).
    pub(crate) cascade_emission_plan: std::cell::OnceCell<Option<cascade::CascadeCodegenPlan>>,
    /// RGX-0078.5.i.9 (D3) — the BOUNDARY-SCANNER emission plan: per plan rule a
    /// direct-coded frameless `scan_<rule>` serves BARE-path call sites in place of
    /// the full-frame protocol method (the protocol twin stays VERBATIM). Computed
    /// ONCE per generation from the SHARED census gate
    /// (`fusibility_census::compute_boundary_scanner_plan`), so the census's
    /// `BOUNDARY-SCANNER-PLAN` report and the emission cannot drift. `None` inside
    /// the cell = no scan graph (cascade plan inactive / annotation-table compile
    /// failure / zero qualifying rules) — the sound default, and the state for
    /// direct `generate_*` unit-test calls that never populate it (those stay
    /// byte-identical to the pre-D3 emission).
    pub(crate) scan_emission_plan: std::cell::OnceCell<Option<scan::ScanCodegenPlan>>,
}

pub(crate) mod cascade;
pub(crate) mod scan;

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

// GRAMMAR-WELLFORMED.A2.4: `SemanticDeterminismPartitionPolicy` moved to the shared
// `semantic_directive_registry` (the linter conditions evaluation-order-based deadness verdicts
// on it, so codegen and linter must read the SAME resolution).
use crate::ast_pipeline::semantic_directive_registry::SemanticDeterminismPartitionPolicy;

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
        // `WS-DIRECTIVE.2` / `DEFAULT-PROFILE.2` / `PROFILE-ALIAS.2` /
        // `STIMULI-SIGNOFF.12` / `STIMULI-SIGNOFF.13.4`: the grammar-level
        // `@whitespace_sensitive:`, `@default_profile:`, and `@profile_alias:`
        // directives are compile-time only, and the rule-level
        // `@quantified_separator:` / `@gen_emit_fact:` / `@gen_predicate:`
        // directives are generation-side only (each compiles to ZERO runtime
        // directives — see `semantic_runtime::compile_layout_sensitivity` /
        // `semantic_runtime::compile_default_profile` /
        // `semantic_runtime::compile_profile_aliases` /
        // `semantic_runtime::compile_quantified_separators` /
        // `stimuli_generator::compute_store_aware_gen_directives`), so the
        // rule any of them binds to must NOT be pushed onto the full
        // `with_semantic_runtime_rule_transaction` path by its mere
        // presence: declaring a parser-inert policy stays emit-neutral for
        // the rule body.
        let is_runtime_relevant = |annotation: &SemanticAnnotation| {
            annotation.name().is_none_or(|name| {
                let normalized = name.trim().to_ascii_lowercase();
                normalized
                    != crate::ast_pipeline::semantic_runtime::WHITESPACE_SENSITIVE_DIRECTIVE_NAME
                    && normalized
                        != crate::ast_pipeline::semantic_runtime::DEFAULT_PROFILE_DIRECTIVE_NAME
                    && normalized
                        != crate::ast_pipeline::semantic_runtime::PROFILE_ALIAS_DIRECTIVE_NAME
                    && normalized
                        != crate::ast_pipeline::semantic_runtime::QUANTIFIED_SEPARATOR_DIRECTIVE_NAME
                    && normalized
                        != crate::ast_pipeline::semantic_runtime::GEN_EMIT_FACT_DIRECTIVE_NAME
                    && normalized
                        != crate::ast_pipeline::semantic_runtime::GEN_PREDICATE_DIRECTIVE_NAME
            })
        };
        let Some(annotations) = &self.annotations else {
            return true;
        };
        let direct_empty = annotations
            .semantic_annotations
            .get(rule_name)
            .is_none_or(|v| !v.iter().any(&is_runtime_relevant));
        let branch_empty = annotations
            .branch_semantic_annotations
            .get(rule_name)
            .is_none_or(|branches| {
                branches
                    .iter()
                    .all(|b| !b.iter().any(&is_runtime_relevant))
            });
        let mid_seq_empty = annotations
            .branch_mid_sequence_semantic_annotations
            .get(rule_name)
            .is_none_or(|branches| branches.iter().all(|b| b.is_empty()));
        direct_empty && branch_empty && mid_seq_empty
    }

    /// INLINE-ACTIONS.2: does any branch of `rule_name` carry a branch-START
    /// inline ACTION annotation (`@emit_fact` / `@open_scope` / `@close_scope`
    /// — the `is_effect()` set)? Used to gate the per-rule winning-branch
    /// effect-application loop so rules without branch-start actions keep
    /// byte-identical parse logic (zero blast radius when the feature is
    /// unused). The annotation-name set mirrors `is_effect()` in
    /// `semantic_runtime.rs`.
    fn rule_has_branch_start_effects(&self, rule_name: &str) -> bool {
        let Some(annotations) = &self.annotations else {
            return false;
        };
        annotations
            .branch_semantic_annotations
            .get(rule_name)
            .is_some_and(|branches| {
                branches.iter().any(|branch| {
                    branch.iter().any(|annotation| {
                        matches!(
                            annotation.name(),
                            Some("emit_fact") | Some("open_scope") | Some("close_scope")
                        )
                    })
                })
            })
    }

    /// RGX-0078.5.i.3 (P2) — the lazily-compiled ANALYSIS runtime-annotation table
    /// (see the field doc). `None` = no annotations or a failing compile — callers
    /// must treat `None` conservatively.
    fn analysis_runtime_annotations(
        &self,
    ) -> Option<&crate::ast_pipeline::CompiledSemanticRuntimeAnnotations> {
        self.analysis_runtime_annotations
            .get_or_init(|| {
                self.annotations
                    .as_ref()
                    .map(|ann| compile_semantic_runtime_annotations(ann).ok())
                    .unwrap_or_else(|| {
                        Some(crate::ast_pipeline::CompiledSemanticRuntimeAnnotations::default())
                    })
            })
            .as_ref()
    }

    /// RGX-0078.5.i.3 (P2) — degenerate-dispatch gate (e), predicate half: does the
    /// rule carry any Branch-phase predicate (rule-level or branch-local)? Queried
    /// against the SAME compiled resolution the generated parser consults at runtime
    /// (`branch_predicates_for_rule` / `branch_predicates_for_rule_branch`), never
    /// re-derived from raw annotations. Returns `true` (blocks dispatch — the sound
    /// default) when the analysis table is unavailable.
    fn rule_has_branch_phase_predicates(&self, rule_name: &str, branch_count: usize) -> bool {
        let Some(compiled) = self.analysis_runtime_annotations() else {
            return true;
        };
        compiled
            .branch_predicates_for_rule(rule_name)
            .next()
            .is_some()
            || (0..branch_count).any(|i| {
                compiled
                    .branch_predicates_for_rule_branch(rule_name, i)
                    .next()
                    .is_some()
            })
    }

    /// RGX-0078.5.i.4 (P1a) — build the inline-emission decision set from the
    /// SHARED census decision function (`fusibility_census::compute_inline_decisions`
    /// — gates (a)–(d) + the measured code-size budget). Called once per generation
    /// from `generate_parser_tokens`. Analysis-table compile failure ⇒ no inlining
    /// (the sound default). No silent caps: the eligible-but-over-budget set is
    /// logged by name.
    fn build_inline_emission_plan(
        &self,
        grammar_tree: &HashMap<String, ASTNode>,
        entry_rule: &str,
    ) {
        let plan: Option<HashSet<String>> = (|| {
            let compiled = self.analysis_runtime_annotations()?;
            let decisions = crate::ast_pipeline::fusibility_census::compute_inline_decisions(
                grammar_tree,
                self.annotations.as_ref(),
                Some(compiled),
                Some(entry_rule),
            );
            let decided: HashSet<String> = decisions
                .iter()
                .filter(|(_, d)| d.decided)
                .map(|(rule, _)| rule.clone())
                .collect();
            let over_budget: Vec<&String> = decisions
                .iter()
                .filter(|(_, d)| d.eligible && !d.decided)
                .map(|(rule, _)| rule)
                .collect();
            eprintln!(
                "        P1a inline-emission plan: {} rule(s) inlined at call sites, {} eligible over budget",
                decided.len(),
                over_budget.len(),
            );
            if !over_budget.is_empty() {
                eprintln!("        over-budget (stay method calls): {:?}", over_budget);
            }
            if decided.is_empty() {
                None
            } else {
                Some(decided)
            }
        })();
        // Second `generate_parser_tokens` call on the same generator instance keeps
        // the first plan (OnceCell) — the tree snapshot choke point above guarantees
        // both calls saw the same tree, so the plan is identical anyway.
        let _ = self.inline_decided_rules.set(plan);
    }

    /// RGX-0078.5.i.4 (P1a) — is `rule` in the inline-emission decision set?
    /// False when no plan was built (unit-test direct calls; analysis-table
    /// failure) — the pre-P1a emission shape.
    fn inline_decided(&self, rule: &str) -> bool {
        self.inline_decided_rules
            .get()
            .and_then(|plan| plan.as_ref())
            .is_some_and(|decided| decided.contains(rule))
    }

    /// RGX-0078.5.i.4 (P1a) — does the plan have ≥1 decided rule (gates emission
    /// of the `inlined_frame_call` engine helper)?
    fn inline_plan_active(&self) -> bool {
        self.inline_decided_rules
            .get()
            .and_then(|plan| plan.as_ref())
            .is_some_and(|decided| !decided.is_empty())
    }

    /// RGX-0078.5.i.4 (P1a) — the needs_raw ride-along fold gate: true iff the
    /// ANALYSIS annotation table (the SAME compiled resolution the parser burns
    /// in) proves BOTH `needs_raw_post_capture_for_rule` and
    /// `needs_raw_final_capture_for_rule` are false for this rule, so the body
    /// head may bind `semantic_capture_raw_for_post = false` instead of paying
    /// the two per-body-execution table probes. Analysis-table failure keeps the
    /// probes (sound default).
    fn rule_needs_raw_capture_statically_false(&self, rule_name: &str) -> bool {
        self.analysis_runtime_annotations().is_some_and(|compiled| {
            !compiled.needs_raw_post_capture_for_rule(rule_name)
                && !compiled.needs_raw_final_capture_for_rule(rule_name)
        })
    }

    /// INLINE-ACTIONS.2: does ANY rule in the grammar carry a branch-start
    /// inline ACTION annotation? Used to gate emission of the
    /// `apply_branch_start_effect_directive` helper method so grammars that do
    /// not use the feature regenerate byte-identical (no dead helper).
    fn grammar_has_branch_start_effects(&self) -> bool {
        let Some(annotations) = &self.annotations else {
            return false;
        };
        annotations
            .branch_semantic_annotations
            .values()
            .any(|branches| {
                branches.iter().any(|branch| {
                    branch.iter().any(|annotation| {
                        matches!(
                            annotation.name(),
                            Some("emit_fact") | Some("open_scope") | Some("close_scope")
                        )
                    })
                })
            })
    }

    pub fn new(grammar_name: String) -> Self {
        Self {
            grammar_name,
            entry_rule: None,
            logger: None,
            annotations: None,
            branch_return_annotations: HashMap::new(),
            enable_debug: true,
            emit_typed_entry_skeleton: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
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
        // RGX-0078.5.c.2 — snapshot the normalized gen-AST so `generate_or_logic`
        // can compute per-branch FIRST-set first-byte prune guards. This is the
        // single choke point: `generate_parser` reaches rule-method generation only
        // through here, and a direct `generate_parser_tokens` caller lands here too.
        // The tree is the SAME one every rule method is emitted from, so transitive
        // rule-reference resolution runs against exactly the codegen's own view.
        //
        // RGX-0078.5.i.7 D0 — every regex atom is rewritten through
        // `effective_regex_pattern` first, so the FIRST analysis derives prefix
        // bytes from EXACTLY the pattern the emitted `match_regex` will compile
        // (`@token_class`/`@charset`/explicit-pattern steering + the
        // semantic_annotation `identifier_literal` special case). Idempotent for
        // the P1a inline-emission path, which re-applies `effective_regex_pattern`
        // at the atom site: steering returns its replacement regardless of input,
        // and the special case keys on the RAW spelling.
        *self.first_set_grammar_tree.borrow_mut() = grammar_tree
            .iter()
            .map(|(rule, node)| {
                (
                    rule.clone(),
                    self.rewrite_regex_atoms_to_effective_patterns(rule, node),
                )
            })
            .collect();

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

        // RGX-0078.5.i.4 (P1a) — compute the inline-emission decision set ONCE
        // per generation from the SHARED census decision function, so the
        // census's `INLINE-DECISIONS` report and this emission cannot drift.
        self.build_inline_emission_plan(grammar_tree, &entry_rule);

        // RGX-0078.5.i.7 (D2-A) — compute the cascade-emission plan ONCE per
        // generation from the SHARED census plan function, so the census's
        // `CASCADE-PLAN` report and this emission cannot drift.
        self.build_cascade_emission_plan_for_codegen(grammar_tree, &entry_rule);

        // RGX-0078.5.i.9 (D3) — compute the boundary-scanner plan ONCE per
        // generation from the SHARED census gate, so the census's
        // `BOUNDARY-SCANNER-PLAN` report and this emission cannot drift.
        self.build_scan_emission_plan_for_codegen(grammar_tree, &entry_rule);

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

        // RGX-0078.5.i.7 (D2-A) — the FUSED cascade graph: one compact
        // `cascade_<rule>` fn per plan rule (sub-roots + internal), entered on the
        // bare-parse path via the observability-twin dispatch inside each
        // sub-root's memoized body. Empty tokens when the plan is inactive
        // (byte-identical pre-D2-A emission).
        let cascade_impl = self.generate_cascade_impl(&parser_name, filename)?;

        // RGX-0078.5.i.9 (D3) — the BOUNDARY-SCANNER graph: one direct-coded
        // frameless `scan_<rule>` fn per plan rule (+ interior recognizer
        // helpers), serving BARE-path call sites while the protocol twin stays
        // verbatim. Empty tokens when the plan is inactive (byte-identical
        // pre-D3 emission).
        let scan_impl = self.generate_scan_impl(&parser_name)?;

        // Generate tests
        let tests = generate_tests(&parser_name);
        eprintln!("        Generated test module");
        eprintln!("        File: {}:{}", file!(), line!());
        eprintln!();

        // Phase 2 M1: parallel typed parser impl, only emitted when --emit-typed-entry-skeleton is set.
        let typed_parser_impl = if self.emit_typed_entry_skeleton {
            self.generate_typed_parser_impl_skeleton(&parser_name, &entry_rule)
        } else {
            TokenStream::new()
        };


        // Combine everything
        let result = quote! {
            #imports
            #types
            #parser_struct
            #parser_impl
            #cascade_impl
            #scan_impl
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
        // REGEX-SELF-HOSTING.6a: only import `regex::Regex` when the grammar actually uses a `/.../`
        // regex literal (and thus the `match_regex` helper). A fully-literal grammar (regex) emits
        // neither, so it does not link Rust's `regex` crate. (`uses_match_regex` is set during codegen
        // before this runs.)
        let regex_import = if self.uses_match_regex.get() {
            quote! { use regex::Regex; }
        } else {
            quote! {}
        };
        quote! {
            use std::collections::HashMap;
            use std::ops::Range;
            #regex_import
            use crate::ast_pipeline::{
                Logger, ParseResult, ParseError, ParseContent, ParseNode, MemoEntry, NodeArena, PgenValue, RuleId, CycleType, RecursionGuard
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

        // RGX-0078.5.i.7 (D2-B) — the fused graph's THIN memo for
        // cycle-participating internal rules (⛔ the #49 bound), emitted only
        // when the plan carries ≥ 1 such rule so a fully-acyclic grammar's
        // artifact stays byte-identical to the D2-A emission. Same lifecycle as
        // the protocol memo maps: constructor-fresh, never cleared per parse.
        let thin_memo_struct_field: TokenStream = if self.cascade_thin_memo_active() {
            // RGX-0078.5.i.7 (MTB-B) — the thin memo carries derivation
            // SEGMENTS, not constructed values: hits splice the cached segment
            // onto the live tape inside `cascade_match_<rule>`.
            // RGX-0078.5.i.14 (C3) — the segment vectors are inline-small
            // (`ThinDerivSegMemoEntry`) so the common short segment is stored
            // without a per-success heap allocation.
            quote! {
                thin_memo: rustc_hash::FxHashMap<(RuleId, usize), crate::ast_pipeline::ThinDerivSegMemoEntry<'input>>,
            }
        } else {
            quote! {}
        };
        // RGX-0078.5.i.7 (MTB-A) — the DERIVATION TAPE of the match-then-build
        // split, emitted only when the plan carries ≥ 1 acyclic-increment rule
        // (a fully-cyclic grammar's artifact stays byte-identical to the D2-B
        // emission). `deriv_events`/`deriv_boundary` are position-like state:
        // match fns append, every speculation-failure restore point truncates
        // to its marks, and each sub-root orchestrator nets its segment to
        // zero — the vecs retain capacity across parses (cleared defensively
        // at `parse()` start). The three cursors are build-walk scratch, live
        // only inside one `cascade_build_*` walk at a time.
        let mtb_struct_fields: TokenStream = if self.cascade_mtb_active() {
            quote! {
                deriv_events: Vec<crate::ast_pipeline::DerivEvent>,
                deriv_boundary: Vec<&'input ParseNode<'input>>,
                deriv_ev_cursor: usize,
                deriv_b_cursor: usize,
                deriv_pos: usize,
            }
        } else {
            quote! {}
        };
        // RGX-0078.5.i.7 (D2-A) — the observability-twin routing state, emitted only
        // when the cascade plan is active so plan-inactive grammars stay
        // byte-identical to the pre-D2-A emission.
        let cascade_struct_fields: TokenStream = if self.cascade_plan_active() {
            quote! {
                // RGX-0078.5.i.7 (D2-A) — BARE-PARSE ROUTING. Cached once at
                // `parse()` start: true iff NO diagnostic consumer is active
                // (coverage, trace/logger, a rule-call-counter reader, memo
                // stats). A bare parse routes each plan sub-root's memoized
                // body to its fused `cascade_<rule>` fn; any diagnostic
                // consumer keeps the full protocol graph so every counter,
                // witness record, and trace line stays byte-exact.
                // Entry-relative parses (`parse_from`) always run the
                // protocol graph.
                bare_parse: bool,
                // RGX-0078.5.i.7 (D2-A) — set by the `rule_call_counts()`
                // accessor (interior-mutable behind `&self`): any consumer
                // that takes the counter Arc — the probe dashboard, the
                // entry/outcome count dumps (both grab their baseline BEFORE
                // the parse) — thereby requests truthful per-rule counters
                // and routes the parse to the protocol graph automatically.
                counters_observed: std::cell::Cell<bool>,
                // RGX-0078.5.j.4 (`-0202`) — the park slot behind
                // `CascadeControlError::Parked`: a rich/legacy `ParseError`
                // crossing INTO the fused graph waits here; the boundary
                // rehydration takes it. Written only on the rare rich
                // crossing — the hot Copy error channel never touches it.
                cascade_parked_error: Option<ParseError>,
                #thin_memo_struct_field
                #mtb_struct_fields
            }
        } else {
            quote! {}
        };

        quote! {
            /// High-performance parser with memoization and zero-copy parsing
            pub struct #parser_name<'input> {
                input: &'input str,
                // RGX-0078.5.d.4.i (candidate B) — the per-parse node arena.
                // Every child `ParseNode` combined into a `ParseContent::{Sequence,
                // Alternative,Quantified}` is `arena.alloc`'d and held as an
                // `&'input` borrow instead of an owned `Box`/`Vec` element. The
                // arena is created at the boundary, borrowed in here, and dropped
                // once the boundary has serialized the tree to owned JSON — one
                // mass free (with each leaf's `String`/`Value` destructor run,
                // since `typed_arena::Arena` is Drop-correct). A single `'input`
                // lifetime spans "borrows input" and "borrows arena" (input is
                // reborrowed down to the arena's scope), so no viral 2nd lifetime.
                arena: &'input NodeArena<'input>,
                position: usize,
                // Optim #6: FxHashMap (rustc-hash) for the memo. Hit on every rule
                // entry, with internal (RuleId, usize) integer keys — no DoS exposure,
                // siphash is overkill. FxHash is the same fast hasher rustc uses
                // internally; ~3-4× faster on small integer keys.
                memo: rustc_hash::FxHashMap<(RuleId, usize), MemoEntry<'input>>,
                // PARSE-TERMINATION.6 — failed `(rule, position)` probes (the
                // majority) live here with no per-entry value/allocation; the
                // backtrack position is the key itself.
                memo_fail: rustc_hash::FxHashSet<(RuleId, usize)>,
                // MEMO-STORE-SOUNDNESS.2 — STORE-TAINTED failures live apart,
                // stamped with the store write epoch at insert; they are
                // replayable only while that epoch is unchanged and are
                // evicted (then re-parsed) once the store has moved. The lean
                // pure-structural set above stays payload-free.
                memo_fail_tainted: rustc_hash::FxHashMap<(RuleId, usize), u64>,
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
                // `RGX-0078.5.g` (construction cache): the compiled annotation
                // table is grammar-CONSTANT, so every parser instance shares
                // one process-wide table (built once in
                // `shared_semantic_runtime_annotations`) instead of rebuilding
                // and dropping it per parse. RE-PROFILE #4 pinned the per-parse
                // rebuild at 11.9% of total parse time (std-HashMap SipHash
                // inserts + a std→Fx re-hash of every key + ~92 short-string
                // allocs + the full table drop).
                semantic_runtime_annotations: &'static crate::ast_pipeline::CompiledSemanticRuntimeAnnotations,
                semantic_runtime_state: crate::ast_pipeline::SemanticRuntimeState,
                // `SV-EXH-PROOF.3.3.4.a` MVP-0: parser-agnostic library plumbing.
                // `library_in_dir`  — root for `@import_from_library`
                //                     (artifacts read at `<dir>/<kind>/<name>.facts.json`).
                // `library_out_dir` — root for `@export_to_library`
                //                     (artifacts written same path layout).
                // Both default to `None` so single-file parses (no library
                // flags, no library annotations) are byte-identical to today.
                library_in_dir: Option<std::path::PathBuf>,
                library_out_dir: Option<std::path::PathBuf>,
                logger: Box<dyn Logger>,
                // Optim #5: cache logger.is_enabled() at construction so the parser hot
                // path skips the per-call vtable dispatch through Box<dyn Logger>. Logger
                // is set once at new() and not swapped at runtime, so the cache is sound.
                logger_enabled: bool,
                // SV-EXH-PROOF.3.3.4.b.6.2.17 — RULE-LEVEL TARGETED TRACE.
                // `trace_rules` (None = parser-level full trace, the prior `--trace`
                // behavior). Some(set) = trace ONLY inside the call-tree of rules
                // whose names are in the set. The per-call gate is then
                // `logger_enabled && (trace_rules.is_none() || trace_active_depth > 0)`.
                // `trace_active_depth` is incremented on entry to a rule listed in
                // `trace_rules` and decremented on exit. Both inits to None/0 so the
                // default behavior (no flag) is identical to today (full trace when
                // `--trace` is passed, no trace otherwise).
                trace_rules: Option<std::collections::HashSet<String>>,
                trace_active_depth: u32,
                // SV-EXH-PROOF.3.3.4.b.6.2.22 — PER-RULE CALL COUNTER.
                // Always-on Vec<AtomicU64> indexed by rule_id; incremented at
                // every rule entry with Ordering::Relaxed (lock-free, ~1ns).
                // Shared via Arc with the optional dashboard thread spawned by
                // the probe when --dump-rule-call-counts is set. Parser-agnostic
                // diagnostic primitive; benefits every grammar pgen builds.
                // Identifies the rules dominating a stuck/slow parse by direct
                // count, replacing samply for this use case.
                rule_call_counts: std::sync::Arc<Vec<std::sync::atomic::AtomicU64>>,
                // SV-EXH-PROOF.3.3.4.b.6.2.25 — FURTHEST-POSITION TRACKING.
                // Monotone max of `self.position` across the entire parse,
                // updated once per rule entry. NEVER restored on speculation
                // backtrack (the whole point: capture the deepest byte any
                // branch reached even if it later failed and the parser
                // rewound). Used on parse failure to report the actual
                // deepest-touched byte alongside the surface `position`
                // (the outermost rule's start) — pinpoints the real
                // defect locus instead of the misleading outermost-start.
                // Cost: one `max` per rule entry (~1-2ns, no atomics needed,
                // no save/restore). Parser-agnostic primitive; benefits
                // every grammar pgen builds.
                furthest_position: usize,
                // GRAMMAR-WELLFORMED.G.4.6 — TRANSACTIONAL PARSE-COVERAGE.
                // The certifying linter's WITNESS side needs the rules a
                // SUCCESSFUL parse genuinely exercises — i.e. the rules of the
                // ACCEPTED parse tree, NOT (a) the output AST's `rule_name`s
                // (return annotations fold whole subtrees into the shaped
                // `ParseContent::Shaped` carrier, erasing the children's rule
                // identities)
                // and NOT (b) `rule_call_counts` (monotone entry counts that
                // include speculative attempts later backtracked). Both are
                // wrong: (a) under-counts, (b) over-counts.
                //
                // This stack is the rigorous source: rule-ids are pushed at
                // rule entry (below), and — crucially — it is TRANSACTIONAL.
                // `try_parse` (the universal speculation wrapper) snapshots its
                // length on entry and truncates back to it on failure, exactly
                // as it already does for `position`, the recursion stack, and
                // the semantic checkpoint. So a rule entered inside a
                // speculation that is later rolled back has its push removed
                // too. After a successful top-level parse every failure
                // necessarily occurred inside some rolled-back `try_parse`
                // (else the parse would have failed), so the surviving entries
                // are EXACTLY the rules of the accepted parse — sound (no
                // backtracked attempts) and complete (annotation folding can't
                // hide them: this records entries, not output nodes).
                //
                // OPT-IN: pushes happen only when `coverage_enabled` is set
                // (via `enable_coverage`), so ordinary parsing pays nothing —
                // the field stays an empty Vec and `try_parse`'s
                // snapshot/truncate is O(1) on it. Cost is borne only by the
                // certification gate, which parses small clean witness samples.
                // Parser-AGNOSTIC: every generated parser gains it identically.
                coverage_stack: Vec<u32>,
                coverage_enabled: bool,
                #cascade_struct_fields
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
        let parse_method = self.generate_parse_method(entry_rule, grammar_tree, rule_order);

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

        // REGEX-SELF-HOSTING.6a: a grammar uses Rust's regex engine iff a generated rule method emits a
        // `match_regex` call (the only call sites are `parser.match_regex(...)` from `/.../` regex-literal
        // terminals). Computed here — after rule methods, before imports/helpers — so `generate_imports`
        // (the `use regex::Regex` line) and `generate_helper_methods` (the `match_regex` helper) can ELIDE
        // both for a fully-literal grammar (regex), dropping its `regex`-crate link entirely.
        self.uses_match_regex.set(
            rule_methods
                .iter()
                .any(|m| m.to_string().contains("match_regex")),
        );

        // Generate helper methods
        let helpers = self.generate_helper_methods(filename, grammar_tree);

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

        // SV-EXH-PROOF.3.3.4.b.6.2.22 — rule-count + name table for the
        // per-rule call counter dashboard. RULE_COUNT sizes the AtomicU64
        // vector; RULE_NAMES maps rule_id -> &'static str for the live
        // top-N display. Both are emitted into the same impl block as the
        // RULE_<NAME> constants so they share the same scoping; pub
        // accessor methods further down expose them to the dashboard
        // (which lives in parseability_probe, not the parser).
        let rule_count = rule_order.len();
        let rule_name_literals: Vec<TokenStream> = rule_order
            .iter()
            .map(|name| {
                let lit = proc_macro2::Literal::string(name);
                quote! { #lit }
            })
            .collect();

        quote! {
            #(#constants)*
            const RULE_COUNT: usize = #rule_count;
            // SV-EXH-PROOF.3.3.4.b.6.2.22 — explicit `'static` lifetime
            // required for constants holding references inside an
            // `impl` block (E0491 otherwise).
            const RULE_NAMES: &'static [&'static str] = &[ #(#rule_name_literals),* ];
        }
    }

    /// UNDEFINED-REF-DIAGNOSTICS.2: the SINGLE SOURCE OF TRUTH for which
    /// referenced-but-undefined rule names codegen synthesizes a NATIVE
    /// matcher for (instead of the never-matching bare `Err(Backtrack)`
    /// stub). The linter's `detect_undefined_references`
    /// (`grammar_wellformedness.rs`) consumes this const as its allowlist, so
    /// the linter can never drift from what codegen actually synthesizes; the
    /// `native_unresolved_builtins_const_matches_dispatch` oracle test locks
    /// the const to the `generate_unresolved_reference_method` dispatch arms
    /// in both directions. The `builtin_` prefix names are the deliberately
    /// grammar-consumable primitives (director 2026-06-07); `true`/`false`
    /// are the boolean-literal fallbacks; `semantic_annotation` is the native
    /// `@…`-line matcher used by the annotation grammars.
    pub const NATIVE_UNRESOLVED_REFERENCE_BUILTINS: &'static [&'static str] = &[
        "builtin_any_char",
        "builtin_ascii_char",
        "false",
        "semantic_annotation",
        "true",
    ];

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

        // UNDEFINED-REF-DIAGNOSTICS.2: belt-and-braces warning at the moment of
        // stub emission — a NON-native unresolved reference compiles into a
        // never-matching `Err(Backtrack)` stub, which silently kills every
        // referencing path. The linter (`--lint-grammar`,
        // `detect_undefined_references`) is the gating diagnostic; this warning
        // covers the direct `--generate-parser` path. Unconditional per the
        // severity doctrine (never gated by verbosity).
        for rule in &unresolved {
            if !Self::NATIVE_UNRESOLVED_REFERENCE_BUILTINS.contains(&rule.as_str()) {
                crate::pgen_warn!(
                    "grammar '{}': reference to UNDEFINED rule '{}' — codegen emits a \
                     never-matching stub (every path through it always fails). Define the rule \
                     or fix the reference (run --lint-grammar for the gating diagnostic).",
                    self.grammar_name,
                    rule
                );
            }
        }

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

        // NOTE: the native arms below MUST stay in lockstep with
        // `NATIVE_UNRESOLVED_REFERENCE_BUILTINS` (the single source of truth
        // the linter consumes). The
        // `native_unresolved_builtins_const_matches_dispatch` oracle test
        // locks the two together: every const name must emit NON-stub tokens
        // here, and any non-const name must emit exactly the bare stub.
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
            // REGEX-SELF-HOSTING.2/.5b: built-in native any-single-character matcher. A grammar that
            // REFERENCES `builtin_any_char` without DEFINING it gets this native matcher instead of the
            // Backtrack stub — consume exactly one Unicode scalar value (advance by its UTF-8 byte
            // length), with NO `regex::Regex`. This lets a `/.../`-free grammar express negated/any-char
            // classes as the `!<X> builtin_any_char` idiom (parser-agnostic; usable by any grammar). No
            // leading-whitespace skipping (it matches the literal next character); Backtracks at end of
            // input. The `builtin_` prefix (director 2026-06-07) namespaces these primitives so they can
            // NEVER be shadowed by a same-named grammar rule (e.g. regex.ebnf's own `any_char` rule).
            "builtin_any_char" => quote! {
                pub fn #method_name(&mut self) -> ParseResult<ParseNode<'input>> {
                    let start_pos = self.position;
                    let matched_char = match self.input[start_pos..].chars().next() {
                        Some(ch) => ch,
                        None => {
                            return Err(ParseError::Backtrack {
                                position: start_pos,
                            });
                        }
                    };
                    let end_pos = start_pos + matched_char.len_utf8();
                    self.position = end_pos;
                    Ok(ParseNode {
                        rule_name: #rule_name,
                        content: ParseContent::Terminal(&self.input[start_pos..end_pos]),
                        span: start_pos..end_pos,
                    })
                }
            },
            // REGEX-SELF-HOSTING.5: built-in native single-ASCII-character matcher. Like
            // `builtin_any_char`, but matches one char ONLY when `ch.is_ascii()` (code point 0x00-0x7F);
            // Backtracks on a non-ASCII char or end of input. Composes with the negation idiom to express
            // the range-negation `[^\x00-\x7F]` as `unicode_char = !builtin_ascii_char builtin_any_char`
            // (any char that is NOT ASCII). Parser-agnostic; `builtin_` prefix avoids any rule-name shadow.
            "builtin_ascii_char" => quote! {
                pub fn #method_name(&mut self) -> ParseResult<ParseNode<'input>> {
                    let start_pos = self.position;
                    let matched_char = match self.input[start_pos..].chars().next() {
                        Some(ch) if ch.is_ascii() => ch,
                        _ => {
                            return Err(ParseError::Backtrack {
                                position: start_pos,
                            });
                        }
                    };
                    let end_pos = start_pos + matched_char.len_utf8();
                    self.position = end_pos;
                    Ok(ParseNode {
                        rule_name: #rule_name,
                        content: ParseContent::Terminal(&self.input[start_pos..end_pos]),
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
        // RGX-0078.5.i.7 (D2-A) — twin-routing state init (plan-active only; see
        // `generate_parser_struct`). `bare_parse` starts false: the real verdict is
        // computed at `parse()` start, and `parse_from` (entry-relative) keeps it
        // false so those parses always run the protocol graph.
        let cascade_field_init: TokenStream = if self.cascade_plan_active() {
            // RGX-0078.5.i.7 (D2-B) — the thin memo mirrors the protocol memo
            // maps' lifecycle: constructor-fresh, never cleared per parse.
            let thin_memo_init: TokenStream = if self.cascade_thin_memo_active() {
                // RGX-0078.5.i.14 (C2) — adaptive pre-size; the tiny-input
                // hazard is still governed by the input-proportional term.
                // RGX-0078.5.j.4 (K3a) — the C2 cap of 256 elements is blown
                // by large inputs (measured: a 3,511 B pattern fills 8,382
                // success entries ⇒ ~7 growth doublings, each a full rehash —
                // 13.4% of that cell's profile). Pre-size to the measured
                // per-byte bound (success ≤ 4.51 entries/B ⇒ K=6 covers every
                // censused cell) with a 32K-element cap bounding construction
                // memory on giant inputs. Correctness-neutral capacity hint;
                // the thin memo's contents are unchanged.
                quote! {
                    thin_memo: rustc_hash::FxHashMap::with_capacity_and_hasher(
                        ((input.len() + 1) * 6).min(32768),
                        Default::default(),
                    ),
                }
            } else {
                quote! {}
            };
            // RGX-0078.5.i.7 (MTB-A) — derivation-tape init (see
            // `generate_parser_struct`): constructor-fresh empty vecs +
            // zeroed build cursors.
            let mtb_init: TokenStream = if self.cascade_mtb_active() {
                // RGX-0078.5.i.14 (C2) — small-constant pre-size for the
                // derivation tape so a cold parse skips the first handful of
                // 0→4→8→… reallocations. Correctness-neutral capacity hints;
                // the tape's contents/lifecycle are unchanged.
                // RGX-0078.5.j.4 (K3a) — the tape grows with input size; scale
                // the hint input-proportionally (clamped: the old 64/16 floors
                // keep tiny inputs unchanged, the caps bound giant inputs) so a
                // large pattern skips the doubling-memcpy chain.
                quote! {
                    deriv_events: Vec::with_capacity(((input.len() + 1) * 4).clamp(64, 32768)),
                    deriv_boundary: Vec::with_capacity((input.len() + 1).clamp(16, 8192)),
                    deriv_ev_cursor: 0,
                    deriv_b_cursor: 0,
                    deriv_pos: 0,
                }
            } else {
                quote! {}
            };
            quote! {
                bare_parse: false,
                counters_observed: std::cell::Cell::new(false),
                cascade_parked_error: None,
                #thin_memo_init
                #mtb_init
            }
        } else {
            quote! {}
        };
        // RGX-0078.5.i.7 (D2-A) — taking the counter Arc IS the counters-consumer
        // signal: every reader (dashboard, entry/outcome dumps) grabs it BEFORE the
        // parse it wants counted, so the routing to the protocol graph is automatic
        // and cannot be forgotten by a future consumer.
        let counters_observed_mark: TokenStream = if self.cascade_plan_active() {
            quote! {
                self.counters_observed.set(true);
            }
        } else {
            quote! {}
        };
        // `DEFAULT-PROFILE.2`: a directive-bearing grammar's parser starts on
        // its declared default profile — the artifact carries its own default,
        // so no caller has to remember to set it. Grammars without the
        // directive emit today's exact `None` (byte-identical regeneration).
        let grammar_profile_init = if self.default_grammar_profile().is_some() {
            quote! { grammar_profile: Some(Self::DEFAULT_GRAMMAR_PROFILE.to_string()), }
        } else {
            quote! { grammar_profile: None, }
        };

        Ok(quote! {
            /// `RGX-0078.5.g` (construction cache): the grammar-constant
            /// compiled annotation table, built ONCE per process on first
            /// use and shared by every parser instance. The table is
            /// immutable after construction (`set_fact_kinds` is part of
            /// the build) and all parse-time access is `&self` point
            /// lookups, so sharing is output-neutral by construction.
            fn shared_semantic_runtime_annotations() -> &'static crate::ast_pipeline::CompiledSemanticRuntimeAnnotations {
                static COMPILED_SEMANTIC_RUNTIME_ANNOTATIONS: std::sync::OnceLock<
                    crate::ast_pipeline::CompiledSemanticRuntimeAnnotations,
                > = std::sync::OnceLock::new();
                COMPILED_SEMANTIC_RUNTIME_ANNOTATIONS
                    .get_or_init(|| {
                        let mut compiled = #compiled_semantic_runtime_annotations;
                        // `RGX-0078.5.j.4` (K3c): install the dense per-rule-id
                        // annotation mirror (rule id = RULE_NAMES index) so the
                        // transaction wrapper's probes are O(1) indexed loads.
                        compiled.install_rule_id_index(Self::RULE_NAMES);
                        compiled
                    })
            }

            pub fn new(input: &'input str, arena: &'input NodeArena<'input>, logger: Box<dyn Logger>) -> Self {
                let logger_enabled = logger.is_enabled();
                // `SV-EXH-PROOF.3.3.4.b.5.1.5.c`: take the compiled
                // annotations first, then seed the semantic-runtime state
                // with the composed-predicate registry so a runtime
                // `@predicate <user-defined-name>` call can dispatch to its
                // `@predicate_def:` body (built-in predicate names are
                // handled directly by `evaluate_predicate`; user-defined
                // names fall through to the registry).
                // `RGX-0078.5.g`: the table is the process-wide shared one.
                let semantic_runtime_annotations: &'static crate::ast_pipeline::CompiledSemanticRuntimeAnnotations =
                    Self::shared_semantic_runtime_annotations();
                let mut semantic_runtime_state = crate::ast_pipeline::SemanticRuntimeState::new();
                semantic_runtime_state
                    .set_predicate_defs(semantic_runtime_annotations.clone_predicate_defs());
                Self {
                    input,
                    // RGX-0078.5.d.4.i — the per-parse node arena (candidate B).
                    arena,
                    position: 0,
                    // Optim #7: pre-size memo to skip the 4-→8-→16-→...-→256 rehash
                    // chain during parse. Even small regex patterns produce ~50-200
                    // memo entries; large ones produce more. 256 covers the common
                    // case without growth; FxHashMap doubles past that. The cost is
                    // a one-time allocation at parser construction (cheap).
                    memo: rustc_hash::FxHashMap::with_capacity_and_hasher(256, Default::default()),
                    // RGX-0078.5.i.14 (C2) — adaptive pre-size for the failure
                    // sets (the #15 census attributed `reserve_rehash` cost to
                    // their default-sized growth on the bench).
                    // RGX-0078.5.j.4 (K3a) — `memo_fail` scales input-
                    // proportionally past the old 256-element cap (measured:
                    // failures ≤ 3.15 entries/B, 11,045 on the 3,511 B corpus
                    // MAX ⇒ K=6 covers every censused cell; 32K-element cap
                    // bounds memory). `memo_fail_tainted` deliberately KEEPS
                    // the 256 cap: tainted populations measured ≤ 54 entries
                    // across the worst-cell census — scaling it would buy
                    // nothing and waste the pre-allocation.
                    memo_fail: rustc_hash::FxHashSet::with_capacity_and_hasher(
                        ((input.len() + 1) * 6).min(32768),
                        Default::default(),
                    ),
                    memo_fail_tainted: rustc_hash::FxHashMap::with_capacity_and_hasher(
                        (input.len() + 1).min(256),
                        Default::default(),
                    ),
                    recursion_guard: RecursionGuard::new(#recursion_guard_max_depth),
                    #grammar_profile_init
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
                    semantic_runtime_annotations,
                    semantic_runtime_state,
                    // `SV-EXH-PROOF.3.3.4.a` MVP-0: opt-in via setter.
                    library_in_dir: None,
                    library_out_dir: None,
                    logger,
                    logger_enabled,
                    trace_rules: None,
                    trace_active_depth: 0,
                    // SV-EXH-PROOF.3.3.4.b.6.2.22 — per-rule call counter init.
                    // Sized to RULE_COUNT (codegen-emitted constant); each
                    // cell starts at 0 and gets fetch_add(1, Relaxed)'d on
                    // every entry to its rule. Wrapped in Arc so the
                    // dashboard thread spawned by the probe can read
                    // concurrently without coordination.
                    rule_call_counts: std::sync::Arc::new(
                        (0..Self::RULE_COUNT).map(|_| std::sync::atomic::AtomicU64::new(0)).collect()
                    ),
                    // SV-EXH-PROOF.3.3.4.b.6.2.25 — furthest-position init.
                    furthest_position: 0,
                    // GRAMMAR-WELLFORMED.G.4.6 — coverage off by default
                    // (opt-in via enable_coverage); empty stack = zero cost.
                    coverage_stack: Vec::new(),
                    coverage_enabled: false,
                    #cascade_field_init
                }
            }

            /// SV-EXH-PROOF.3.3.4.b.6.2.25 — public accessor for the deepest
            /// byte position any branch reached during the parse, regardless
            /// of subsequent backtracking. On a successful parse, this equals
            /// `self.position` at completion. On a failed parse, this is the
            /// byte where the actual defective construct lives — generally
            /// FAR deeper than the surface `position` (which reports the
            /// outermost failing rule's start). Used by the caller (e.g.
            /// parseability_probe) to report a meaningful error location.
            pub fn furthest_position(&self) -> usize {
                self.furthest_position
            }

            /// SV-EXH-PROOF.3.3.4.b.6.2.22 — accessor for the per-rule call
            /// counter array. Returns an Arc clone so the dashboard thread
            /// (spawned by the probe when --dump-rule-call-counts is set)
            /// can poll the live counters without locking. Each index is the
            /// rule's RuleId; RULE_NAMES gives the corresponding name.
            pub fn rule_call_counts(&self) -> std::sync::Arc<Vec<std::sync::atomic::AtomicU64>> {
                #counters_observed_mark
                self.rule_call_counts.clone()
            }

            /// SV-EXH-PROOF.3.3.4.b.6.2.22 — accessor for the rule-name table.
            /// Same length as `rule_call_counts()`; same indexing.
            pub fn rule_names() -> &'static [&'static str] {
                Self::RULE_NAMES
            }

            /// GRAMMAR-WELLFORMED.G.4.6 — opt in to TRANSACTIONAL PARSE-COVERAGE.
            /// Clears any prior coverage and enables per-rule-entry recording on
            /// the transactional `coverage_stack`. Call immediately before the
            /// parse whose accepted-tree rule set you want; read it back with
            /// `exercised_rule_names()` after a SUCCESSFUL parse. Off by default
            /// so ordinary parsing carries zero coverage overhead.
            pub fn enable_coverage(&mut self) {
                self.coverage_enabled = true;
                self.coverage_stack.clear();
            }

            /// GRAMMAR-WELLFORMED.G.4.6 — the rules EXERCISED by the accepted
            /// parse, as names. Sound + complete for the witness side: it is the
            /// de-duplicated set of rule entries that survived all speculative
            /// rollbacks (so: committed successes only — no backtracked attempts,
            /// and immune to return-annotation `ParseContent::Shaped` folding,
            /// since it records entries rather than output nodes). Meaningful
            /// only after a successful parse made with coverage enabled; returns
            /// an empty set otherwise. Indices map through `RULE_NAMES`.
            pub fn exercised_rule_names(&self) -> std::collections::HashSet<String> {
                self.coverage_stack
                    .iter()
                    .filter_map(|&id| Self::RULE_NAMES.get(id as usize).map(|s| s.to_string()))
                    .collect()
            }

            /// RGX-0078.5.h.1b — the COMMITTED (surviving) rule-entry COUNTS of the
            /// accepted parse: a fold of the transactional `coverage_stack` into a
            /// per-RuleId histogram. Same soundness as `exercised_rule_names` (failed
            /// speculations are truncated by `try_parse`, so only committed successes
            /// remain — C3-B semantics: winners AND successful-but-losing tournament
            /// branches both count), but keeps multiplicity instead of deduplicating,
            /// so `rule_call_counts()[id] − committed[id]` is the rule's
            /// FAILED-speculation entry count. Read-only; meaningful only after a
            /// successful parse made with coverage enabled. Indexed like RULE_NAMES.
            pub fn exercised_rule_entry_counts(&self) -> Vec<u64> {
                let mut counts = vec![0u64; Self::RULE_COUNT];
                for &id in self.coverage_stack.iter() {
                    if let Some(slot) = counts.get_mut(id as usize) {
                        *slot += 1;
                    }
                }
                counts
            }

            /// SV-EXH-PROOF.3.3.4.b.6.2.17 — opt in to RULE-LEVEL TARGETED TRACE.
            /// Pass `Some(set_of_rule_names)` to restrict trace output to the
            /// call-tree of the listed rules (any rule whose name matches is the
            /// scope root; trace is active inside that rule and any rule it calls,
            /// recursively, until it returns). Pass `None` (default) for the
            /// existing parser-level full trace behavior. Has no effect unless
            /// `logger_enabled` (i.e. `--trace` was also passed on the CLI).
            pub fn set_trace_rules(&mut self, rules: Option<std::collections::HashSet<String>>) {
                self.trace_rules = rules;
                self.trace_active_depth = 0;
            }

            /// SV-EXH-PROOF.3.3.4.b.6.2.17 — the per-call trace gate. Replaces the
            /// bare `if self.logger_enabled` checks throughout codegen. When
            /// `trace_rules` is None this collapses to the prior behavior (any
            /// log emits if logger_enabled); when Some(...), emits only inside
            /// the listed rules' call-tree. `#[inline(always)]` because this is
            /// called on every potential log site in the hot parse path.
            #[inline(always)]
            fn trace_enabled(&self) -> bool {
                self.logger_enabled
                    && (self.trace_rules.is_none() || self.trace_active_depth > 0)
            }
        })
    }

    /// Phase 2 M1 emit. Produces a parallel `impl` block carrying
    /// `parse_full_<entry>_typed` returning `ParseResult<serde_json::Value>`. The M1 body
    /// is a skeleton wrapper around the legacy `parse_full_<entry>` plus
    /// `serde_json::to_value(&node)`, which is functionally equivalent to "parse + AST
    /// dump as JSON". M2 will replace the body with truly inline shape-emit logic that
    /// honors the rule's return annotation. M1 establishes the architectural pattern and
    /// the public API surface; it does NOT yet produce shaped output per annotations.
    fn generate_typed_parser_impl_skeleton(
        &self,
        parser_name: &Ident,
        entry_rule: &str,
    ) -> TokenStream {
        let parse_full_method = format_ident!("parse_full_{}", entry_rule);
        let parse_full_typed_method = format_ident!("parse_full_{}_typed", entry_rule);

        quote! {
            impl<'input> #parser_name<'input> {
                /// Phase 2 M1 skeleton: parses via the legacy ParseNode path, then
                /// converts the tree to `serde_json::Value` via Serialize. Functionally
                /// equivalent to "parse + AST-dump-as-JSON" for now. M2 will replace
                /// this body with truly inline shape-emit logic that honors the rule's
                /// return annotation, eliminating the post-parse transform.
                pub fn #parse_full_typed_method(
                    &mut self,
                ) -> ParseResult<serde_json::Value> {
                    let node = self.#parse_full_method()?;
                    serde_json::to_value(&node).map_err(|err| {
                        self.create_contextual_error(&format!(
                            "Phase 2 M1 typed serialization failed: {}",
                            err
                        ))
                    })
                }
            }
        }
    }

    /// `WS-DIRECTIVE.2`: the grammar's declared layout policy — the
    /// grammar-level `@whitespace_sensitive:` directive compiled from
    /// `self.annotations` (all-`false` = whitespace-insensitive when absent).
    /// This replaced the retired grammar-NAME gate (`grammar_name != "regex"`
    /// / `"systemverilogpreprocessor"`), so the policy is declared IN the
    /// `.ebnf`. A malformed/conflicting payload is NOT swallowed by the
    /// `.ok()` here: the same compile runs — and aborts generation with the
    /// precise error — in `generate_compiled_semantic_runtime_annotations_tokens`
    /// (via `compile_semantic_runtime_annotations`), which every generation
    /// emits.
    fn layout_sensitivity(&self) -> LayoutSensitivity {
        self.annotations
            .as_ref()
            .and_then(|annotations| compile_layout_sensitivity(annotations).ok().flatten())
            .unwrap_or_default()
    }

    /// `DEFAULT-PROFILE.2`: the grammar's declared default dialect profile —
    /// the grammar-level `@default_profile:` directive compiled from
    /// `self.annotations` (`None` when absent: an unspecified requested
    /// profile stays unset, the permissive guard default). This replaced the
    /// retired grammar-NAME gates (`parser_registry.rs` / `main.rs` /
    /// `embedding_api.rs` `== "regex"` → `pcre2`), so the default is declared
    /// IN the `.ebnf` and burned into the emitted constructor/setter. A
    /// malformed/conflicting payload is NOT swallowed by the `.ok()` here:
    /// the same compile runs — and aborts generation with the precise error —
    /// in `generate_compiled_semantic_runtime_annotations_tokens` (via
    /// `compile_semantic_runtime_annotations`), which every generation emits.
    fn default_grammar_profile(&self) -> Option<String> {
        self.annotations
            .as_ref()
            .and_then(|annotations| compile_default_profile(annotations).ok().flatten())
    }

    /// `PROFILE-ALIAS.2`: the grammar's declared request-spelling alias map —
    /// the grammar-level `@profile_alias:` directive(s) compiled from
    /// `self.annotations` (`None` when absent: requested profiles pass
    /// through unresolved). This replaces the retired grammar-NAME alias
    /// tables (`parser_registry.rs` `"systemverilog"` arm / the global
    /// `main.rs` spelling table / the hand-copied `embedding_api.rs` arms),
    /// so the spellings are declared IN the `.ebnf` and burned into the
    /// emitted `GRAMMAR_PROFILE_ALIASES` const + alias-resolving setter. A
    /// malformed/conflicting payload is NOT swallowed by the `.ok()` here:
    /// the same compile runs — and aborts generation with the precise error —
    /// in `generate_compiled_semantic_runtime_annotations_tokens` (via
    /// `compile_semantic_runtime_annotations`), which every generation emits.
    fn grammar_profile_aliases(&self) -> Option<std::collections::BTreeMap<String, String>> {
        self.annotations
            .as_ref()
            .and_then(|annotations| compile_profile_aliases(annotations).ok().flatten())
    }

    /// `PROFILE-ALIAS.2`: the emitted alias carrier for an alias-declaring
    /// grammar — the sorted `GRAMMAR_PROFILE_ALIASES` const (deterministic:
    /// `BTreeMap` iteration order) plus the case-insensitive resolver
    /// `set_grammar_profile` routes through. `pub` on both so the parser
    /// registry's profile oracle can source the SAME artifact-owned data
    /// instead of keeping its own copy.
    fn grammar_profile_alias_surface(
        aliases: &std::collections::BTreeMap<String, String>,
    ) -> TokenStream {
        let alias_pairs = aliases
            .iter()
            .map(|(alias, canonical)| quote! { (#alias, #canonical) });
        quote! {
            /// `PROFILE-ALIAS.2`: the grammar-declared `@profile_alias` map —
            /// request spellings → canonical profile names (sorted by
            /// spelling; spellings stored lowercase). `set_grammar_profile`
            /// resolves through it case-insensitively, so every entry point
            /// accepts the declared spellings with no caller cooperation.
            pub const GRAMMAR_PROFILE_ALIASES: &'static [(&'static str, &'static str)] =
                &[#(#alias_pairs),*];

            /// Resolve one requested profile spelling through
            /// [`Self::GRAMMAR_PROFILE_ALIASES`] (case-insensitive);
            /// unmatched spellings pass through unchanged.
            pub fn resolve_grammar_profile_alias(profile: &str) -> &str {
                Self::GRAMMAR_PROFILE_ALIASES
                    .iter()
                    .find(|(alias, _)| alias.eq_ignore_ascii_case(profile))
                    .map(|(_, canonical)| *canonical)
                    .unwrap_or(profile)
            }
        }
    }

    fn generate_parse_method(
        &self,
        entry_rule: &str,
        grammar_tree: &HashMap<String, ASTNode>,
        rule_order: &[String],
    ) -> TokenStream {
        let parse_method = format_ident!("parse_{}", entry_rule);
        let parse_full_method = format_ident!("parse_full_{}", entry_rule);
        let allow_trailing_layout = !self.layout_sensitivity().trailing;

        // RGX-0078.5.i.7 (D2-A) — the BARE-PARSE verdict, cached once per parse:
        // true iff NO diagnostic consumer is active. Each clause routes one
        // observability surface to the protocol graph: `coverage_enabled` =
        // cert-coverage + the outcome dump; `logger_enabled` = every `--trace`
        // form; `counters_observed` = any taker of the `rule_call_counts()` Arc
        // (dashboard + entry/outcome dumps, marked by the accessor itself);
        // the env probe = `PGEN_REPORT_MEMO_STATS` (fused internal rules skip
        // their memo lane, so memo stats are truthful only on the protocol
        // graph). `parse_from` (entry-relative) always clears the flag.
        let bare_parse_compute: TokenStream = if self.cascade_plan_active() {
            // RGX-0078.5.i.7 (MTB-A) — defensive per-parse tape reset (each
            // orchestrator already nets its segment to zero on both arms; the
            // clear keeps capacity and guards against any leaked prefix).
            let mtb_tape_reset: TokenStream = if self.cascade_mtb_active() {
                quote! {
                    self.deriv_events.clear();
                    self.deriv_boundary.clear();
                }
            } else {
                quote! {}
            };
            quote! {
                self.bare_parse = !self.coverage_enabled
                    && !self.logger_enabled
                    && !self.counters_observed.get()
                    && !crate::ast_pipeline::report_memo_stats_enabled();
                #mtb_tape_reset
            }
        } else {
            quote! {}
        };
        let bare_parse_clear: TokenStream = if self.cascade_plan_active() {
            quote! {
                self.bare_parse = false;
            }
        } else {
            quote! {}
        };

        // `DEFAULT-PROFILE.2`: a directive-bearing grammar's parser CARRIES its
        // declared default profile (`@default_profile`) — an associated const
        // plus a `set_grammar_profile` where `None` RESTORES the declared
        // default instead of falling back to the permissive unset state, so
        // "unspecified means the declared default" holds at every entry point
        // with no caller cooperation.
        // `PROFILE-ALIAS.2`: an alias-declaring grammar's parser likewise
        // CARRIES its request-spelling map (`@profile_alias`) — a sorted
        // `GRAMMAR_PROFILE_ALIASES` const plus alias resolution INSIDE the
        // setter, so every entry point accepts the declared spellings with no
        // caller cooperation. Grammars without either directive emit today's
        // exact tokens (byte-identical regeneration).
        let grammar_profile_surface = match (
            self.default_grammar_profile(),
            self.grammar_profile_aliases(),
        ) {
            (Some(default_profile), Some(aliases)) => {
                let alias_surface = Self::grammar_profile_alias_surface(&aliases);
                quote! {
                    #alias_surface

                    /// `DEFAULT-PROFILE.2`: the grammar-declared `@default_profile` —
                    /// the dialect profile an UNSPECIFIED requested profile resolves
                    /// to. The constructor starts on it; `set_grammar_profile(None)`
                    /// restores it.
                    pub const DEFAULT_GRAMMAR_PROFILE: &'static str = #default_profile;

                    pub fn set_grammar_profile(&mut self, profile: Option<&str>) {
                        // `None` restores the grammar-declared default profile,
                        // never a permissive unset state; an explicit spelling
                        // resolves through the grammar-declared alias map.
                        self.grammar_profile = match profile {
                            Some(value) => {
                                Some(Self::resolve_grammar_profile_alias(value).to_string())
                            }
                            None => Some(Self::DEFAULT_GRAMMAR_PROFILE.to_string()),
                        };
                    }
                }
            }
            (Some(default_profile), None) => {
                quote! {
                    /// `DEFAULT-PROFILE.2`: the grammar-declared `@default_profile` —
                    /// the dialect profile an UNSPECIFIED requested profile resolves
                    /// to. The constructor starts on it; `set_grammar_profile(None)`
                    /// restores it.
                    pub const DEFAULT_GRAMMAR_PROFILE: &'static str = #default_profile;

                    pub fn set_grammar_profile(&mut self, profile: Option<&str>) {
                        // `None` restores the grammar-declared default profile,
                        // never a permissive unset state.
                        self.grammar_profile = match profile {
                            Some(value) => Some(value.to_string()),
                            None => Some(Self::DEFAULT_GRAMMAR_PROFILE.to_string()),
                        };
                    }
                }
            }
            (None, Some(aliases)) => {
                let alias_surface = Self::grammar_profile_alias_surface(&aliases);
                quote! {
                    #alias_surface

                    pub fn set_grammar_profile(&mut self, profile: Option<&str>) {
                        // An explicit spelling resolves through the
                        // grammar-declared alias map; unmatched spellings pass
                        // through unchanged.
                        self.grammar_profile = profile
                            .map(|value| Self::resolve_grammar_profile_alias(value).to_string());
                    }
                }
            }
            (None, None) => quote! {
                pub fn set_grammar_profile(&mut self, profile: Option<&str>) {
                    self.grammar_profile = profile.map(|value| value.to_string());
                }
            },
        };

        // GRAMMAR-WELLFORMED.H.12.8.4.3: entry-aware full parse. Compute one dispatch
        // arm per rule so `parse_from` / `parse_full_from` can begin a full-input parse
        // from ANY rule, not only the canonical entry. This is the enabler for
        // certificate witness verification of entry-relative rules (rules rooted under
        // an alternate LRM start symbol such as `library_text`): the cert's
        // `--entry-rule` / `--cert-union-config` entry now drives BOTH generation and
        // verification, and `parseability_probe --entry-rule` can reproduce them. The
        // arm set is exactly the rules that get a `parse_<rule>` method emitted in
        // `generate_parser_tokens` (present in `grammar_tree`); the canonical entry is
        // the default arm; the set is deduped so a duplicate `rule_order` entry cannot
        // produce an unreachable-pattern arm. Parser-agnostic and inert for single-entry
        // grammars (the default arm reproduces today's `parse()` exactly).
        let mut seen_entry_arms: std::collections::HashSet<&str> =
            std::collections::HashSet::new();
        let mut entry_arm_names: Vec<&str> = Vec::new();
        for rule_name in rule_order {
            if rule_name == entry_rule || !grammar_tree.contains_key(rule_name) {
                continue;
            }
            if seen_entry_arms.insert(rule_name.as_str()) {
                entry_arm_names.push(rule_name.as_str());
            }
        }
        let entry_arm_methods: Vec<Ident> = entry_arm_names
            .iter()
            .map(|rule_name| format_ident!("parse_{}", rule_name))
            .collect();

        // INLINE-ACTIONS.2: emit the branch-start effect-application helper only
        // when some rule actually uses a branch-start inline action directive, so
        // grammars that do not use the feature regenerate byte-identical (no dead
        // helper). The companion per-rule call loop in `generate_or_logic` is
        // gated the same way, so the parse logic of non-feature rules is also
        // byte-identical.
        let branch_start_effect_helper: TokenStream = if self.grammar_has_branch_start_effects() {
            quote! {
                /// INLINE-ACTIONS.2: apply a single branch-start inline ACTION
                /// directive (`@emit_fact` / `@open_scope` / `@close_scope`) for
                /// the WINNING branch, resolving its `$ref`s against the selected
                /// branch's content, DIRECTLY onto the live semantic state. There
                /// is intentionally no transaction wrapper here: the enclosing
                /// rule transaction (and, for multi-branch rules, the tournament
                /// checkpoint) already own rollback, so a later rule failure
                /// undoes these emissions exactly like the branch body's own
                /// emissions. Mirrors the resolution in
                /// `apply_semantic_runtime_effect_directive` but targets the
                /// state rather than a transaction. Returns `Ok(false)` for
                /// non-effect directives (predicate / library / declaration),
                /// which are handled elsewhere.
                fn apply_branch_start_effect_directive(
                    &mut self,
                    directive: &crate::ast_pipeline::SemanticRuntimeDirective,
                    root_content: &ParseContent<'input>,
                ) -> ParseResult<bool> {
                    let resolved: Option<crate::ast_pipeline::SemanticRuntimeDirective> = match directive {
                        crate::ast_pipeline::SemanticRuntimeDirective::EmitFact(spec) => {
                            let resolved_name = self
                                .resolve_semantic_runtime_value_against_content(&spec.name, root_content)
                                .ok_or_else(|| {
                                    self.create_contextual_error(
                                        "Branch-start @emit_fact could not resolve the fact name against the selected branch content",
                                    )
                                })?;
                            let resolved_attributes = self
                                .resolve_unified_semantic_properties_against_content(
                                    &spec.attributes,
                                    root_content,
                                    // FINAL-PHASE-PREDICATE.3: `@emit_fact` is an effect,
                                    // not a predicate — single-content resolution, so its
                                    // own content is the inert fallback (byte-identical).
                                    root_content,
                                )?;
                            Some(crate::ast_pipeline::SemanticRuntimeDirective::EmitFact(
                                crate::ast_pipeline::SemanticFactSpec {
                                    kind: spec.kind.clone(),
                                    name: resolved_name,
                                    attributes: resolved_attributes,
                                },
                            ))
                        }
                        crate::ast_pipeline::SemanticRuntimeDirective::OpenScope(spec) => {
                            let resolved_name = spec
                                .name
                                .as_ref()
                                .map(|value| {
                                    self.resolve_semantic_runtime_value_against_content(value, root_content)
                                        .ok_or_else(|| {
                                            self.create_contextual_error(
                                                "Branch-start @open_scope could not resolve the scope name against the selected branch content",
                                            )
                                        })
                                })
                                .transpose()?;
                            Some(crate::ast_pipeline::SemanticRuntimeDirective::OpenScope(
                                crate::ast_pipeline::SemanticScopeSpec {
                                    kind: spec.kind.clone(),
                                    name: resolved_name,
                                },
                            ))
                        }
                        crate::ast_pipeline::SemanticRuntimeDirective::CloseScope(spec) => {
                            let resolved_name = spec
                                .name
                                .as_ref()
                                .map(|value| {
                                    self.resolve_semantic_runtime_value_against_content(value, root_content)
                                        .ok_or_else(|| {
                                            self.create_contextual_error(
                                                "Branch-start @close_scope could not resolve the scope name against the selected branch content",
                                            )
                                        })
                                })
                                .transpose()?;
                            Some(crate::ast_pipeline::SemanticRuntimeDirective::CloseScope(
                                crate::ast_pipeline::SemanticCloseScopeSpec {
                                    kind: spec.kind.clone(),
                                    name: resolved_name,
                                },
                            ))
                        }
                        _ => None,
                    };
                    match resolved {
                        Some(resolved) => Ok(self.semantic_runtime_state.apply_directive(&resolved)),
                        None => Ok(false),
                    }
                }
            }
        } else {
            quote! {}
        };

        quote! {
            /// GRAMMAR-WELLFORMED.H.12.8.4.3: the shared per-parse reset ceremony, run
            /// before any entry rule. Extracted from `parse()` so `parse_from` reuses it
            /// verbatim (entry-aware verification must reset identically). `parse()` is
            /// behavior-identical to before this extraction.
            fn prepare_parse_state(&mut self) {
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
                // `SV-EXH-PROOF.3.3.4.b.6.2.37.2`: facts pushed onto the
                // parser BEFORE the parse are intentional preloads (e.g. the
                // SV stdlib's `process`/`semaphore`/`mailbox` `type_name`
                // facts loaded by `preload_systemverilog_stdlib` in
                // parser_registry) and survive the per-parse reset (re-based
                // to the root scope); everything else — leftovers from a
                // prior parse or from PEG speculation that wasn't properly
                // rolled back — returns to `new()` semantics, and the
                // composed-predicate registry is re-seeded from the compiled
                // annotations (`.3.3.4.b.5.1.5.c`).
                // `PGEN-RGX-0078-0198`: the reset is IN PLACE — behavior-
                // identical to the former facts().to_vec() → new() →
                // push_fact_record replay → set_predicate_defs(clone)
                // ceremony, without rebuilding and dropping the whole state
                // on every parse.
                self.semantic_runtime_state.reset_for_new_parse(
                    self.semantic_runtime_annotations.predicate_defs_map(),
                );
            }

            pub fn parse(&mut self) -> ParseResult<ParseNode<'input>> {
                self.prepare_parse_state();
                #bare_parse_compute
                let parse_outcome = self.#parse_method();
                // PARSE-TERMINATION.6 (WHY+WHERE): opt-in memo footprint report.
                // RGX-0078.5.i.7 P-env: process-once cached (was a per-parse getenv).
                if crate::ast_pipeline::report_memo_stats_enabled() {
                    self.report_memo_stats();
                }
                parse_outcome
            }

            /// GRAMMAR-WELLFORMED.H.12.8.4.3: entry-aware `parse`. Same reset ceremony as
            /// `parse()`, but the root rule is selected by `entry` so a parse can start
            /// from an alternate LRM start symbol (e.g. `library_text`). An
            /// unknown/unsupported `entry` falls back to the canonical entry, so this is
            /// behavior-identical to `parse()` for single-entry grammars.
            pub fn parse_from(&mut self, entry: &str) -> ParseResult<ParseNode<'input>> {
                self.prepare_parse_state();
                #bare_parse_clear
                let parse_outcome = match entry {
                    #( #entry_arm_names => self.#entry_arm_methods(), )*
                    _ => self.#parse_method(),
                };
                if crate::ast_pipeline::report_memo_stats_enabled() {
                    self.report_memo_stats();
                }
                parse_outcome
            }

            /// FINAL-PHASE-PREDICATE.2: discharge whole-input `phase: final`
            /// obligations against the now-complete semantic store at top-level
            /// parse completion. A `phase: final` predicate resolved+enqueued a
            /// deferred obligation at its rule's commit; here — once the whole
            /// input has been consumed and the store holds the complete
            /// inventory — each obligation is evaluated, and the first that
            /// fails (e.g. a named reference whose definition never appeared
            /// anywhere in the input) fails the whole parse at the obligation's
            /// source position (validator first-error-by-position semantics). A
            /// no-op — and byte-identical to before — for any grammar with no
            /// `phase: final` predicate (the worklist is empty).
            fn discharge_final_phase_obligations(&self) -> ParseResult<()> {
                match self.semantic_runtime_state.discharge_deferred_obligations() {
                    Ok(()) => Ok(()),
                    Err((position, message)) => Err(ParseError::ContextualError {
                        message,
                        position,
                        rule_stack: Vec::new(),
                        input_context: String::new(),
                    }),
                }
            }

            pub fn parse_full(&mut self) -> ParseResult<ParseNode<'input>> {
                let parsed = self.parse()?;
                if #allow_trailing_layout {
                    // Allow trailing layout/comments so parse_full reports structural completeness.
                    self.consume_layout_for_terminal("<EOF>");
                }
                if self.position == self.input.len() {
                    // FINAL-PHASE-PREDICATE.2: whole-input obligations discharge
                    // here, after full consumption is confirmed and the store is
                    // complete.
                    self.discharge_final_phase_obligations()?;
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

            /// GRAMMAR-WELLFORMED.H.12.8.4.3: entry-aware `parse_full` — full-input parse
            /// from an arbitrary start symbol. Drives certificate witness verification
            /// (and `parseability_probe --entry-rule`) from the configured entry. The
            /// trailing-layout / full-consumption check is byte-identical to `parse_full`.
            pub fn parse_full_from(&mut self, entry: &str) -> ParseResult<ParseNode<'input>> {
                let parsed = self.parse_from(entry)?;
                if #allow_trailing_layout {
                    // Allow trailing layout/comments so the parse reports structural completeness.
                    self.consume_layout_for_terminal("<EOF>");
                }
                if self.position == self.input.len() {
                    // FINAL-PHASE-PREDICATE.2: whole-input obligations discharge
                    // on the entry-aware path too (drives cert witnesses +
                    // `--entry-rule`), byte-identical to `parse_full`.
                    self.discharge_final_phase_obligations()?;
                    Ok(parsed)
                } else {
                    Err(ParseError::InvalidSyntax {
                        message: "Parser did not consume full input",
                        position: self.position,
                    })
                }
            }

            #grammar_profile_surface

            pub fn grammar_profile(&self) -> Option<&str> {
                self.grammar_profile.as_deref()
            }

            pub fn semantic_runtime_annotations(
                &self,
            ) -> &crate::ast_pipeline::CompiledSemanticRuntimeAnnotations {
                self.semantic_runtime_annotations
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
                    .transaction_for_rule(self.semantic_runtime_annotations, rule_name)
            }

            fn semantic_predicate_debug_label(
                &self,
                spec: &crate::ast_pipeline::SemanticPredicateSpec,
            ) -> String {
                format!("{} {:?}", spec.name, spec.args)
            }

            // ================================================================
            // `SV-EXH-PROOF.3.3.4.a` MVP-0: parser-agnostic library plumbing.
            // Two setters wire the CLI-supplied `--lib-in` / `--lib-out`
            // directories into the parser. Setters return `&mut Self` so they
            // chain naturally on a builder-style construction.
            // ================================================================
            pub fn set_library_in_dir(
                &mut self,
                dir: Option<std::path::PathBuf>,
            ) -> &mut Self {
                self.library_in_dir = dir;
                self
            }

            pub fn set_library_out_dir(
                &mut self,
                dir: Option<std::path::PathBuf>,
            ) -> &mut Self {
                self.library_out_dir = dir;
                self
            }

            pub fn library_in_dir(&self) -> Option<&std::path::Path> {
                self.library_in_dir.as_deref()
            }

            pub fn library_out_dir(&self) -> Option<&std::path::Path> {
                self.library_out_dir.as_deref()
            }

            /// `SV-EXH-PROOF.3.3.4.a` MVP-0: apply `@import_from_library` for the
            /// current rule. Resolves `name_from` against the rule's parse
            /// content, reads the artifact, merges its facts into the
            /// transaction-scoped semantic state. Missing artifact = warning
            /// + continue (matches "single-file parse" fallback behaviour);
            /// other I/O / parse errors are propagated as
            /// `ParseError::ContextualError` so the IIFE's `restore` path
            /// fires cleanly.
            fn apply_semantic_runtime_library_import_directive(
                &self,
                transaction: &mut crate::ast_pipeline::SemanticRuntimeTransaction<'_>,
                spec: &crate::ast_pipeline::SemanticLibraryImportSpec,
                root_content: &ParseContent<'input>,
            ) -> ParseResult<()> {
                // No lib-in configured -> import is a no-op (single-file mode).
                let Some(lib_in) = self.library_in_dir.as_deref() else {
                    return Ok(());
                };
                let resolved = self
                    .resolve_semantic_runtime_value_against_content(
                        &spec.name,
                        root_content,
                    )
                    .ok_or_else(|| ParseError::ContextualError {
                        message: format!(
                            "@import_from_library.name_from could not be resolved against rule content (kind={})",
                            spec.kind
                        ),
                        position: self.position,
                        rule_stack: Vec::new(),
                        input_context: String::new(),
                    })?;
                let name = resolved.as_text().ok_or_else(|| {
                    ParseError::ContextualError {
                        message: format!(
                            "@import_from_library.name_from resolved to a non-textual value (kind={}, value={:?})",
                            spec.kind, resolved
                        ),
                        position: self.position,
                        rule_stack: Vec::new(),
                        input_context: String::new(),
                    }
                })?;
                match crate::ast_pipeline::library::read_artifact(lib_in, &spec.kind, name) {
                    Ok(records) => {
                        for record in records {
                            transaction.state_mut().push_fact_record(record);
                        }
                        Ok(())
                    }
                    Err(crate::ast_pipeline::library::LibraryError::NotFound(_)) => {
                        // Missing artifact: log + continue. This makes the
                        // importer's downstream `has_fact` checks evaluate
                        // false — same outcome as today's no-library run.
                        if self.trace_enabled() {
                            self.logger.log_info(
                                file!(),
                                line!(),
                                &format!(
                                    "📚 @import_from_library: artifact not found (kind={}, name={}); continuing",
                                    spec.kind, name
                                ),
                            );
                        }
                        Ok(())
                    }
                    Err(other) => Err(ParseError::ContextualError {
                        message: format!(
                            "@import_from_library failed: kind={}, name={}, error={:?}",
                            spec.kind, name, other
                        ),
                        position: self.position,
                        rule_stack: Vec::new(),
                        input_context: String::new(),
                    }),
                }
            }

            /// `SV-EXH-PROOF.3.3.4.a` MVP-0: apply `@export_to_library` for the
            /// current rule. Resolves `name_from`, computes the rule's
            /// emitted-fact delta against `entry_checkpoint`, writes the
            /// artifact. Atomic write; failure -> `ParseError::ContextualError`.
            ///
            /// `entry_checkpoint` is the checkpoint captured at the start of
            /// the rule's body transaction; everything in `state.facts()` from
            /// `entry_checkpoint.fact_len()` onward is the rule's contribution.
            fn apply_semantic_runtime_library_export_directive(
                &self,
                transaction: &crate::ast_pipeline::SemanticRuntimeTransaction<'_>,
                spec: &crate::ast_pipeline::SemanticLibraryExportSpec,
                root_content: &ParseContent<'input>,
                entry_fact_len: usize,
            ) -> ParseResult<()> {
                // No lib-out configured -> export is a no-op.
                let Some(lib_out) = self.library_out_dir.as_deref() else {
                    return Ok(());
                };
                let resolved = self
                    .resolve_semantic_runtime_value_against_content(
                        &spec.name,
                        root_content,
                    )
                    .ok_or_else(|| ParseError::ContextualError {
                        message: format!(
                            "@export_to_library.name_from could not be resolved against rule content (kind={})",
                            spec.kind
                        ),
                        position: self.position,
                        rule_stack: Vec::new(),
                        input_context: String::new(),
                    })?;
                let name = resolved.as_text().ok_or_else(|| {
                    ParseError::ContextualError {
                        message: format!(
                            "@export_to_library.name_from resolved to a non-textual value (kind={}, value={:?})",
                            spec.kind, resolved
                        ),
                        position: self.position,
                        rule_stack: Vec::new(),
                        input_context: String::new(),
                    }
                })?;
                let all_facts = transaction.state().facts();
                let start = entry_fact_len.min(all_facts.len());
                let delta = &all_facts[start..];
                // `SV-EXH-PROOF.3.3.4.b.5.3`: export eligibility is schema-driven.
                // The set of exportable fact kinds is derived from the grammar's
                // `@fact_kind:` declarations (or the MVP-0 default when no schema
                // is declared) — any declared-exportable kind round-trips, not
                // just `type_name`.
                let exportable_kinds =
                    self.semantic_runtime_annotations.exportable_fact_kinds();
                match crate::ast_pipeline::library::write_artifact(
                    lib_out,
                    &spec.kind,
                    name,
                    delta,
                    &exportable_kinds,
                ) {
                    Ok(path) => {
                        if self.trace_enabled() {
                            self.logger.log_info(
                                file!(),
                                line!(),
                                &format!(
                                    "📚 @export_to_library: wrote artifact kind={}, name={}, facts={}, path={}",
                                    spec.kind,
                                    name,
                                    delta.len(),
                                    path.display(),
                                ),
                            );
                        }
                        Ok(())
                    }
                    Err(err) => Err(ParseError::ContextualError {
                        message: format!(
                            "@export_to_library failed: kind={}, name={}, error={:?}",
                            spec.kind, name, err
                        ),
                        position: self.position,
                        rule_stack: Vec::new(),
                        input_context: String::new(),
                    }),
                }
            }

            // RGX-0078.5.i.2 (P0): `rule_name` is `&'static str` — every call
            // site passes the rule-name literal, and the static lifetime lets
            // `push_rule_context_static` store it as `Cow::Borrowed` with NO
            // per-entry allocation (the measured V1 census surface, ≈−4.3% of
            // the bench).
            // `RGX-0078.5.j.4` (K3c): the wrapper takes the emitted numeric
            // rule id alongside the name (the C1 `rule_id_stack` precedent —
            // every call site knows its rule at codegen time), and every
            // annotation probe below is an O(1) id-indexed load instead of a
            // String hash (`has_rule` + up to six phase filters per annotated
            // entry — measured 5.9% cum of the RGX corpus-MAX cell). The name
            // stays for the rule-context stack + transaction naming + traces.
            pub fn with_semantic_runtime_rule_transaction<F>(
                &mut self,
                rule_id: RuleId,
                rule_name: &'static str,
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
                    || !self.semantic_runtime_annotations.has_rule_id(rule_id)
                {
                    // SV-EXH-PROOF.3.3.4.b.6.2.36.2 — even the fast-path
                    // pushes/pops the rule context so any nested rule's
                    // emit_fact / predicate eval / rollback / apply_delta
                    // trace events include the full call chain. The unannotated
                    // rule itself emits nothing, but it CAN call into
                    // annotated child rules.
                    self.semantic_runtime_state.push_rule_context_static(rule_name);
                    let result = f(self);
                    self.semantic_runtime_state.pop_rule_context();
                    let (node, _raw) = result?;
                    return Ok(node);
                }

                // SV-EXH-PROOF.3.3.4.b.6.2.36.2 — push the rule context here; PARSE-TERMINATION.3.1
                // takes the entry checkpoint just below (self.semantic_runtime_state stays in
                // place, so the pushed context is on the same state the checkpoint snapshots and
                // the rollback restores). The matching pop runs after the err-restore regardless
                // of result.is_err().
                self.semantic_runtime_state.push_rule_context_static(rule_name);

                // PARSE-TERMINATION.3.1: O(1) checkpoint REPLACES the former O(N) full-state
                // clone (`take` + `.clone()`), which was the O(N^2) per-rule cost behind the
                // ~N^1.66 stateful-parse super-linearity and the uvm ~26 GB blow-up
                // (PARSE-TERMINATION.1/.3/.3.2). `self.semantic_runtime_state` now STAYS in place
                // (holds the pre-body facts); the rule body mutates it directly; any non-commit
                // exit `rollback_to_named`s to this checkpoint below (O(changes), Laurent & Mens
                // SLE 2016). The lost-facts subtlety (the mid-fn `take` for the txn machinery) is
                // handled by an inner closure that ALWAYS restores the taken state before any
                // error propagates — see below.
                let semantic_runtime_checkpoint = self.semantic_runtime_state.checkpoint();
                // ============================================================
                // SV-EXH-PROOF.3.3.3 FIX — exception-safe semantic-state restore.
                //
                // ROOT CAUSE: the fallible body below `std::mem::take`s
                // self.semantic_runtime_state and then performs `?` operations
                // (f(self)?, apply_..effect..?, resolve_..predicate..?). When
                // this body was a plain `{ }` block, a `?` early-return
                // propagated out of the ENTIRE function, JUMPING OVER the
                // `if result.is_err() { self.semantic_runtime_state = original }`
                // restore below. self.semantic_runtime_state was therefore left
                // as `take`'s leftover (== `Default::default()` == valid-but-
                // EMPTY: `SemanticRuntimeState::new()` — one Global scope, no
                // facts), silently destroying every fact emitted by prior
                // COMMITTED sibling rules (e.g. a `typedef`'s `type_name` fact),
                // so later sibling `@predicate has_fact(...)` checks saw nothing.
                //
                // FIX (Option B — idiomatic "try block" emulation): wrap the
                // body in an immediately-invoked closure so every `?` returns
                // into `result` instead of out of the function; the restore
                // below now runs on EVERY non-commit exit (`?`, explicit Err,
                // early return). Parser-agnostic; zero behaviour change on the
                // success path; no `unsafe`.
                //
                // Note: `<SemanticRuntimeState as Default>::default()` delegates
                // to `Self::new()` (Global scope, no facts) — a VALID state — so
                // `std::mem::take` never leaves a *corrupt* state; a panic
                // between the take and the restore leaves a valid (empty) state,
                // and the parser never `catch_unwind`s mid-parse (a panic aborts
                // the parse). The IIFE is thus also panic-ROBUST. No
                // `mem::replace`-with-placeholder is needed (it would be a
                // behavioural no-op since `take` already yields `new()`).
                //
                // RAII alternative (CONSIDERED, NOT USED): a Drop guard that
                // restores self.semantic_runtime_state would be panic-SAFE by
                // strict definition (Drop also runs while unwinding). It was
                // rejected because a *safe-Rust* guard is not cleanly
                // expressible here: the post-take body calls `&self` methods
                // (apply_semantic_runtime_effect_directive,
                // resolve_semantic_predicate_spec_against_content,
                // semantic_predicate_debug_label), and a guard owning a mutable
                // handle to self.semantic_runtime_state for its lifetime
                // conflicts with those whole-struct `&self` borrows (Rust has no
                // partial borrows across method calls). The only true Drop-guard
                // form needs a contained `*mut` + ~3 lines of `unsafe` in Drop;
                // given `Default == new()` makes the IIFE already panic-robust
                // (no corruption; panic aborts the parse anyway), the extra
                // `unsafe` was judged not worth it. If a mid-parse
                // `catch_unwind` is ever introduced, revisit the RAII guard.
                // ============================================================
                // Note: rule_context push/pop straddles the IIFE — pushed
                // BEFORE the `mem::take` above so original AND clone both
                // hold it, popped AFTER the err-restore below.
                let result: ParseResult<ParseNode<'input>> = (|| -> ParseResult<ParseNode<'input>> {
                    let mut predicate_blocked = false;
                    for directive in self.semantic_runtime_annotations.pre_predicates_for_rule_id(rule_id)
                    {
                        match self
                            .semantic_runtime_state
                            .evaluate_directive_predicate(directive)
                        {
                            Some(true) => {}
                            Some(false) => {
                                if self.trace_enabled() {
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
                        // `SV-EXH-PROOF.3.3.4.a` MVP-0: capture the rule's
                        // TRUE entry checkpoint so a later
                        // `@export_to_library` can snapshot the rule's
                        // emitted-fact delta. The TRUE entry point is BEFORE
                        // `f(self)?` (where the rule's body parses and its
                        // child rules' `@emit_fact` directives fire into
                        // self.semantic_runtime_state); `original_*` holds
                        // the pre-body state. After body parse the state's
                        // `fact_len` will be `original.facts().len() + body
                        // emitted`; the delta is exactly the facts the
                        // rule's body contributed.
                        let semantic_runtime_entry_fact_len: usize =
                            semantic_runtime_checkpoint.fact_len();
                        let (node, semantic_raw_content) = f(self)?;
                        let semantic_raw_content =
                            semantic_raw_content.as_ref().unwrap_or(&node.content);
                        // PARSE-TERMINATION.3.1: O(1) mid-`take` (a borrow-checker workaround so
                        // the `&self` apply_* methods can run while the txn mutates the taken
                        // state). The txn section runs in an INNER closure returning
                        // `ParseResult<()>` so any `?` returns LOCALLY; we then ALWAYS move the
                        // taken state back into `self.semantic_runtime_state` BEFORE propagating
                        // an error — so the outer `rollback_to_named` operates on a POPULATED
                        // state (the lost-facts hole). `node` stays owned by THIS scope (not moved
                        // into the closure) so `semantic_raw_content` (which borrows `node`) stays
                        // valid; the closure borrows `node`/`semantic_raw_content` read-only.
                        let mut semantic_runtime_state =
                            std::mem::take(&mut self.semantic_runtime_state);
                        let semantic_txn_result: ParseResult<()> = (|| -> ParseResult<()> {
                        // SV-EXH-PROOF.3.3.4.b.6.2.36.2 — name the
                        // transaction with the owning rule so its Drop
                        // auto-rollback (the once-anonymous code path) now
                        // identifies itself in the trace. Per
                        // [[feedback_why_and_where_before_solution]].
                        let mut semantic_runtime_transaction =
                            semantic_runtime_state.transaction_named(rule_name);
                        for directive in self
                            .semantic_runtime_annotations
                            .effect_directives_for_rule_id(rule_id)
                        {
                            let _ = self.apply_semantic_runtime_effect_directive(
                                &mut semantic_runtime_transaction,
                                directive,
                                &node.content,
                            )?;
                        }
                        // `SV-EXH-PROOF.3.3.4.a` MVP-0: process
                        // `@import_from_library` AFTER effect directives so
                        // any rule-local `@emit_fact` is already applied,
                        // BEFORE post-predicates so they observe the merged
                        // imported facts. (Design memo originally said
                        // "rule entry"; corrected: `name_from` is resolved
                        // against the rule's parse content, which only exists
                        // post-body. Imports still merge into the importer's
                        // scope, so later sibling `has_fact` checks see them.)
                        for directive in self
                            .semantic_runtime_annotations
                            .library_imports_for_rule_id(rule_id)
                        {
                            if let crate::ast_pipeline::SemanticRuntimeDirective::ImportFromLibrary(spec) =
                                directive
                            {
                                self.apply_semantic_runtime_library_import_directive(
                                    &mut semantic_runtime_transaction,
                                    spec,
                                    &node.content,
                                )?;
                            }
                        }
                        let mut post_predicate_blocked = false;
                        let mut blocked_post_predicate: Option<String> = None;
                        for directive in self
                            .semantic_runtime_annotations
                            .post_predicates_for_rule_id(rule_id)
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
                                | crate::ast_pipeline::SemanticRuntimeDirective::Predicate(_)
                                | crate::ast_pipeline::SemanticRuntimeDirective::ExportToLibrary(_)
                                | crate::ast_pipeline::SemanticRuntimeDirective::ImportFromLibrary(_)
                                | crate::ast_pipeline::SemanticRuntimeDirective::DeclareFactKind(_)
                                | crate::ast_pipeline::SemanticRuntimeDirective::DefinePredicate(_) => {}
                            }
                        }
                        if post_predicate_blocked {
                            if self.trace_enabled() {
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
                                // SV-EXH-PROOF.3.3.4.b.6.2.36.2 — DBG-level
                                // rule stack on every failure event per
                                // [[feedback_why_and_where_before_solution]]:
                                // every failure must answer WHO + WHY. The
                                // rejected-by-predicate log gives WHY (the
                                // failing predicate); the DBG stack adds
                                // the full chain of rules that led to this
                                // point so a diagnoser can see the call
                                // context immediately.
                                if crate::ast_pipeline::trace_enabled(crate::ast_pipeline::TraceLevel::Debug) {
                                    let stack_path: Vec<&str> = self
                                        .recursion_guard
                                        .parse_stack
                                        .iter()
                                        .map(|entry| entry.0)
                                        .collect();
                                    self.logger.log_debug(
                                        file!(),
                                        line!(),
                                        &format!(
                                            "   ↪ rule_stack={} ({} frames), position={}",
                                            stack_path.join(" > "),
                                            stack_path.len(),
                                            node.span.start,
                                        ),
                                    );
                                }
                            }
                            return Err(ParseError::Backtrack {
                                position: node.span.start,
                            });
                        }
                            // FINAL-PHASE-PREDICATE.2: enqueue this rule's
                            // `phase: final` obligations. Each is resolved
                            // against the rule's captured content EXACTLY like a
                            // post predicate (so `$name` becomes the concrete
                            // captured string) — but instead of gating inline it
                            // registers a DEFERRED OBLIGATION on the
                            // transaction's state (transactional: a rollback
                            // discards it, and the tournament delta path replays
                            // only the winner's). It is discharged ONCE at
                            // `parse_full` success against the completed store,
                            // so it can validate a LEGAL FORWARD REFERENCE a
                            // `post` gate cannot (the definition may appear
                            // LATER than this rule). Reached only when no post
                            // predicate blocked (the early `return Err` skips it).
                            for directive in self
                                .semantic_runtime_annotations
                                .final_predicates_for_rule_id(rule_id)
                            {
                                if let crate::ast_pipeline::SemanticRuntimeDirective::Predicate(spec) =
                                    directive
                                {
                                    let resolved_spec = self
                                        .resolve_semantic_predicate_spec_against_content(
                                            spec,
                                            semantic_raw_content,
                                            &node.content,
                                        )?;
                                    semantic_runtime_transaction
                                        .state_mut()
                                        .enqueue_deferred_obligation(
                                            resolved_spec,
                                            node.span.start,
                                        );
                                }
                            }
                            // `SV-EXH-PROOF.3.3.4.a` MVP-0: process
                            // `@export_to_library` here — AFTER the rule's
                            // body has emitted all its facts (so the artifact
                            // captures the complete delta), AFTER post
                            // predicates have validated the rule's content,
                            // BEFORE commit so a write failure rolls back the
                            // transaction (atomic semantics matches the
                            // `.3.3.3` IIFE's exception-safety invariant).
                            for directive in self
                                .semantic_runtime_annotations
                                .library_exports_for_rule_id(rule_id)
                            {
                                if let crate::ast_pipeline::SemanticRuntimeDirective::ExportToLibrary(spec) =
                                    directive
                                {
                                    self.apply_semantic_runtime_library_export_directive(
                                        &semantic_runtime_transaction,
                                        spec,
                                        &node.content,
                                        semantic_runtime_entry_fact_len,
                                    )?;
                                }
                            }
                            let _ = semantic_runtime_transaction.commit();
                            Ok(())
                        })();
                        // PARSE-TERMINATION.3.1: ALWAYS move the taken state back (commit kept its
                        // changes; a non-commit drop of the txn auto-rolled-back its own effects),
                        // BEFORE propagating, so the outer rollback_to_named below operates on a
                        // POPULATED state — no lost-facts hole.
                        self.semantic_runtime_state = semantic_runtime_state;
                        semantic_txn_result?;
                        Ok(node)
                    }
                })();
                if result.is_err() {
                    // PARSE-TERMINATION.3.1: O(changes) rollback to the entry checkpoint instead
                    // of restoring a full O(N) clone. `rule_name` names the rollback in the trace.
                    self.semantic_runtime_state
                        .rollback_to_named(semantic_runtime_checkpoint, Some(rule_name));
                }
                // SV-EXH-PROOF.3.3.4.b.6.2.36.2 — pop the rule context.
                // Pairs with the `push_rule_context` above. Stack discipline:
                // exactly one pop per push, regardless of result.is_ok().
                self.semantic_runtime_state.pop_rule_context();
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
                    // `SV-EXH-PROOF.3.3.4.a` MVP-0: library directives are
                    // their own phase (handled by
                    // `apply_semantic_runtime_library_{import,export}_directive`
                    // from `with_semantic_runtime_rule_transaction`); they are
                    // not effects in the emit_fact/open_scope/close_scope sense.
                    // Returning `Ok(false)` mirrors `Predicate`.
                    crate::ast_pipeline::SemanticRuntimeDirective::ExportToLibrary(_)
                    | crate::ast_pipeline::SemanticRuntimeDirective::ImportFromLibrary(_) => {
                        Ok(false)
                    }
                    // `SV-EXH-PROOF.3.3.4.b.5.1.2`: declarations are compile-time
                    // only; at runtime they are no-ops (the registry lives in
                    // CompiledSemanticRuntimeAnnotations.fact_kinds).
                    crate::ast_pipeline::SemanticRuntimeDirective::DeclareFactKind(_)
                    | crate::ast_pipeline::SemanticRuntimeDirective::DefinePredicate(_) => {
                        Ok(false)
                    }
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
                                // FINAL-PHASE-PREDICATE.3: `@emit_fact` is an effect, not a
                                // predicate — single-content resolution, so its own content
                                // is the inert fallback (byte-identical).
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

            #branch_start_effect_helper

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
                fallback_content: &ParseContent<'input>,
            ) -> ParseResult<Vec<crate::ast_pipeline::UnifiedSemanticProperty>> {
                let mut resolved = Vec::with_capacity(properties.len());
                for property in properties {
                    resolved.push(crate::ast_pipeline::UnifiedSemanticProperty {
                        key: property.key.clone(),
                        value: self.resolve_unified_semantic_value_against_content(
                            &property.value,
                            root_content,
                            fallback_content,
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
                // FINAL-PHASE-PREDICATE.3: the `view` selects the PRIMARY content; the
                // OTHER view is the fallback. A NAMED / object-key reference absent from
                // the primary view resolves against the fallback before the hard
                // unresolved-attribute error, so a MULTI-BRANCH rule's shaped-key `$name`
                // under the default `view: raw` resolves exactly as a single-branch rule
                // already does (whose raw capture is `None`, so both views coincide).
                let (selected_content, fallback_content) = match spec.view {
                    crate::ast_pipeline::SemanticPredicateContentView::Raw => {
                        (raw_content, shaped_content)
                    }
                    crate::ast_pipeline::SemanticPredicateContentView::Shaped => {
                        (shaped_content, raw_content)
                    }
                };

                let mut resolved_args = Vec::with_capacity(spec.args.len());
                for arg in &spec.args {
                    resolved_args.push(
                        self.resolve_unified_semantic_value_against_content(
                            arg,
                            selected_content,
                            fallback_content,
                        )?,
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
                // FINAL-PHASE-PREDICATE.3: same primary/fallback view split as the
                // hard resolver above (see its note).
                let (selected_content, fallback_content) = match spec.view {
                    crate::ast_pipeline::SemanticPredicateContentView::Raw => {
                        (raw_content, shaped_content)
                    }
                    crate::ast_pipeline::SemanticPredicateContentView::Shaped => {
                        (shaped_content, raw_content)
                    }
                };

                let mut resolved_args = Vec::with_capacity(spec.args.len());
                for arg in &spec.args {
                    let Some(resolved_arg) =
                        self.try_resolve_unified_semantic_value_against_content(
                            arg,
                            selected_content,
                            fallback_content,
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
                fallback_content: &ParseContent<'input>,
            ) -> ParseResult<crate::ast_pipeline::UnifiedSemanticValue> {
                match value {
                    crate::ast_pipeline::UnifiedSemanticValue::RuleReference(reference) => self
                        .resolve_semantic_reference(root_content, reference)
                        .or_else(|| {
                            // FINAL-PHASE-PREDICATE.3: a NAMED / object-key reference
                            // absent from the primary view falls back to the OTHER
                            // content view. Positional `$N` references walk the raw tree
                            // structurally and are NEVER retried against the other view
                            // (their index means something different there), so `$text` /
                            // `$N` semantics are unchanged.
                            if Self::semantic_reference_is_named(reference) {
                                self.resolve_semantic_reference(fallback_content, reference)
                            } else {
                                None
                            }
                        })
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
                                    fallback_content,
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
                                fallback_content,
                            )?,
                        ),
                    ),
                }
            }

            fn try_resolve_unified_semantic_value_against_content(
                &self,
                value: &crate::ast_pipeline::UnifiedSemanticValue,
                root_content: &ParseContent<'input>,
                fallback_content: &ParseContent<'input>,
            ) -> ParseResult<Option<crate::ast_pipeline::UnifiedSemanticValue>> {
                match value {
                    crate::ast_pipeline::UnifiedSemanticValue::RuleReference(reference) => Ok(
                        self.resolve_semantic_reference(root_content, reference)
                            .or_else(|| {
                                // FINAL-PHASE-PREDICATE.3: named-reference fallback to the
                                // OTHER content view (see the hard resolver's note);
                                // positional `$N` is never retried.
                                if Self::semantic_reference_is_named(reference) {
                                    self.resolve_semantic_reference(fallback_content, reference)
                                } else {
                                    None
                                }
                            })
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
                                    fallback_content,
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
                                    fallback_content,
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

    /// RGX-0078.5.i.4 (P1a) — the rule BODY (the closure body handed to
    /// `memoized_call`): parse logic + return-annotation transforms + relational
    /// guards + coverage/partition observability + the `(ParseNode, raw)` result.
    /// Factored out of `generate_rule_method_with_recursion` so an inlined call
    /// site (`generate_inlined_frame`) emits the IDENTICAL body the rule method
    /// carries — identical-by-construction, no drift. Context contract: the
    /// emitted tokens reference `parser` (a `&mut Self`) and a `start_pos`
    /// binding visible at the point of insertion (the method binds it before the
    /// memoized closure; the inlined frame binds it at the closure head — the
    /// same value, since `memoized_call`'s miss path does not move `position`
    /// before running the body).
    fn generate_rule_body_inner(
        &self,
        rule_name: &str,
        ast_node: &ASTNode,
        filename: &str,
    ) -> Result<TokenStream> {
        // Generate the parsing logic based on AST node type.
        // RGX-0078.5.c.2 — dispatch a TOP-LEVEL `Or` body directly to
        // `generate_or_logic` with `top_level = true` so it may emit the FIRST-set
        // prune guard (furthest_position-neutral only when `parse_start` is the
        // rule-entry position — which holds exactly for the rule body). Every other
        // node shape (incl. a nested `Or`) goes through `generate_node_parsing_logic`,
        // which passes `top_level = false`.
        let parse_logic = match ast_node {
            ASTNode::Or { alternatives } => {
                self.generate_or_logic(alternatives, rule_name, filename, true)?
            }
            _ => self.generate_node_parsing_logic(ast_node, rule_name, filename)?,
        };

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
        let deterministic_partition_policy = self.rule_deterministic_partition_policy(rule_name);
        let deterministic_partition_enabled = deterministic_partition_policy.enabled;
        let deterministic_partition_group = deterministic_partition_policy
            .group_label
            .unwrap_or_else(|| format!("rule.{}", rule_name));

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

        // REGEX-SELF-HOSTING.4c: a rule's `@transform` applied to the matched SPAN text, for the
        // case its body is NOT a single terminal (so the terminal-path `@transform` at the atom
        // codegen did not fire — e.g. `digits = digit+` after self-hosting). The emitted block is a
        // no-op when `result` is already a `TransformedTerminal` (terminal-body `@transform` rules),
        // so it is additive and cannot double-apply. Or roots apply transforms per-branch.
        let semantic_span_transform_tokens: TokenStream = match ast_node {
            ASTNode::Or { .. } => quote! {},
            _ => self.generate_post_body_span_transform(rule_name),
        };
        // RAWCAP-TRANSFORM-PATH.2: on the non-`Or` transform path, capture the raw
        // body `result` BEFORE `#post_parse_transform_tokens` shadows it with the
        // shaped `->` Json, so a POSITIONAL (`$N`) raw-view post-predicate resolves
        // against the ordered raw captures. The `Or` path already captures inline
        // per-branch in `generate_or_logic` (empty here for `Or` roots — symmetric
        // with `semantic_span_transform_tokens`). CODEGEN-time-gated to the NARROW
        // positional case: emitted ONLY for a rule that actually has a positional
        // raw-view post-predicate, so a rule without one is byte-identical to before
        // (no inert guard, no `unused_assignments`) — every shipped grammar today,
        // since none has such a predicate. Named refs keep resolving against the
        // shaped Json (`SEMREF-SHAPED`), so this cannot regress the SV `declared_*` /
        // regex `numeric_backreference` family. The interpreter applies the same
        // narrow gate at runtime (`parse_harness_interpreter.rs`), so the
        // differential-equivalence contract holds.
        let semantic_nonor_positional_raw_capture_tokens: TokenStream = match ast_node {
            ASTNode::Or { .. } => quote! {},
            _ if self.rule_needs_positional_raw_post_capture(rule_name) => quote! {
                semantic_raw_content = Some(result.clone());
            },
            _ => quote! {},
        };
        // RGX-0078.5.i.4 (P1a) — the needs_raw ride-along fold: when the ANALYSIS
        // table proves both probes false for this rule (every directive-free
        // rule), bind the constant instead of paying two FxHash probes per body
        // execution. Provably behavior-identical: the fold only replaces a lookup
        // whose runtime result is statically known.
        let semantic_capture_raw_head: TokenStream =
            if self.rule_needs_raw_capture_statically_false(rule_name) {
                quote! {
                    let semantic_capture_raw_for_post = false;
                }
            } else {
                quote! {
                    let semantic_capture_raw_for_post =
                        parser.semantic_runtime_annotations
                            .needs_raw_post_capture_for_rule(#rule_name)
                        // FINAL-PHASE-PREDICATE.2: a raw-view `phase: final` predicate
                        // resolves `$name`/`$N` against the rule's raw body exactly like
                        // a raw-view `post` predicate, so it needs the same raw capture.
                        // Byte-identical for a rule without a raw-view final predicate
                        // (`needs_raw_final_capture_for_rule` returns false).
                        || parser.semantic_runtime_annotations
                            .needs_raw_final_capture_for_rule(#rule_name);
                }
            };
        // RGX-0078.5.i.7 (D2-A) — the OBSERVABILITY-TWIN dispatch: a plan SUB-ROOT's
        // memoized body routes a bare parse (no coverage / trace / counters /
        // memo-stats consumer — see the `bare_parse` computation in `parse()`) to its
        // compact fused `cascade_<rule>` fn; every diagnostic consumer keeps the
        // protocol body VERBATIM below, so the outcome pins and cert witnesses stay
        // byte-exact by construction. The raw half of the tuple is `None` exactly as
        // the protocol path computes for a plan rule (directive-free ⇒ the analysis
        // table proves `needs_raw_post/final` false ⇒ `semantic_raw_content` is never
        // set). Also spliced into P1a inlined frames of a decided sub-root — dead
        // there by construction (inlined frames execute only inside protocol bodies,
        // which run only when `bare_parse` is false).
        let cascade_twin_dispatch: TokenStream = if self.cascade_sub_root(rule_name) {
            let cascade_fn = format_ident!("cascade_{}", rule_name);
            quote! {
                if parser.bare_parse {
                    return parser.#cascade_fn().map(|node| (node, None));
                }
            }
        } else {
            quote! {}
        };

        Ok(quote! {
            #cascade_twin_dispatch
            #semantic_capture_raw_head
            let mut semantic_selected_branch_index: Option<usize> = None;
            let mut semantic_raw_content: Option<ParseContent<'input>> = None;
            // Main parsing logic - produces the 'result' variable
            #parse_logic;

            // RAWCAP-TRANSFORM-PATH.2: capture raw body content for non-`Or`
            // positional raw-view post-predicates before the transform shadows it.
            #semantic_nonor_positional_raw_capture_tokens

            // Apply rule-level return annotation for non-Or roots
            // (Or roots apply per-branch transforms inline)
            #post_parse_transform_tokens

            // REGEX-SELF-HOSTING.4c: rule-level `@transform` over the matched span (no-op if the
            // terminal-path @transform already produced a TransformedTerminal).
            #semantic_span_transform_tokens

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
        })
    }

    /// RGX-0078.5.i.4 (P1a + P1b) — an INLINED wrapper frame for a decided rule
    /// at one of its call sites: the rule's body (identical-by-construction via
    /// `generate_rule_body_inner`) run under the emitted `inlined_frame_call`
    /// engine helper, which preserves the per-frame observability verbatim
    /// (entry `fetch_add`, coverage push, furthest-position, the
    /// method-identical exit trace lines and negative-case recording). Elided
    /// vs the method call: recursion-guard enter/exit (gate (a): provably
    /// non-load-bearing), rule-context push/pop and the `--trace-rules` scope
    /// probe (both proven trace-only), the method call frame itself, and —
    /// since P1b — the packrat memo at the inlined frame (the body runs
    /// directly; the rule METHOD keeps its memoized_call).
    fn generate_inlined_frame(&self, rule_name: &str, filename: &str) -> Result<TokenStream> {
        {
            let mut stack = self.inline_emission_stack.borrow_mut();
            if stack.iter().any(|entry| entry == rule_name) {
                return Err(anyhow::anyhow!(
                    "P1a inline emission re-entered rule '{}' (emission stack: {:?}) — \
                     the decided subgraph must be acyclic by gate (a); this is a \
                     census/emission drift bug",
                    rule_name,
                    stack
                ));
            }
            stack.push(rule_name.to_string());
        }
        let body_ast = {
            let tree = self.first_set_grammar_tree.borrow();
            tree.get(rule_name).cloned().ok_or_else(|| {
                anyhow::anyhow!(
                    "P1a inline emission: decided rule '{}' missing from the gen-AST snapshot",
                    rule_name
                )
            })?
        };
        let body = self.generate_rule_body_inner(rule_name, &body_ast, filename)?;
        self.inline_emission_stack.borrow_mut().pop();
        let rule_const = format_ident!("RULE_{}", rule_name.to_uppercase());
        let negative_case_policy = self.rule_negative_case_policy(rule_name);
        let negative_case_enabled = negative_case_policy.invalid_case;
        let negative_case_strict = negative_case_policy.negative;
        Ok(quote! {
            parser.inlined_frame_call(
                Self::#rule_const,
                #rule_name,
                #negative_case_enabled,
                #negative_case_strict,
                |parser| {
                    let start_pos = parser.position;
                    #body
                },
            )
        })
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
        // RGX-0078.5.i.4 (P1a) — the body is factored into
        // `generate_rule_body_inner` so an inlined call site emits the IDENTICAL
        // body this method carries (identical-by-construction, no drift).
        let rule_body_inner = self.generate_rule_body_inner(rule_name, ast_node, filename)?;

        eprintln!();
        eprintln!(
            "            File: {}:{}: Exiting rule processing block",
            file!(),
            line!()
        );

        let negative_case_policy = self.rule_negative_case_policy(rule_name);
        let negative_case_enabled = negative_case_policy.invalid_case;
        let negative_case_strict = negative_case_policy.negative;
        let recursion_guard_max_depth = GENERATED_RECURSION_GUARD_MAX_DEPTH;

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
        // SV-EXH-PROOF.3.3.4.b.6.2.15 — UNIVERSAL PACKRAT MEMOIZATION.
        // Pre-fix: memoization was only applied to RECURSIVE rules (rules
        // that call themselves transitively). Non-recursive leaf rules
        // (identifier, simple_identifier, number, string_literal, etc.)
        // were re-computed every visit. When a deep PEG alternation tries
        // many branches that all descend through these leaves at the same
        // position, the leaves are evaluated 50-100× redundantly. Trace
        // analysis on uvm_pkg line 11755 hang showed `identifier` visited
        // 96× at the same byte position (117064) — pure redundant work.
        // FIX: memoize ALL rules unconditionally. This gives true Packrat
        // O(N) parsing semantics. Memory cost: a HashMap entry per
        // (rule, position) pair actually queried — typically O(N × log N)
        // for real programs, not N × R worst-case. Parser-AGNOSTIC level 5
        // engine enhancement. The `is_recursive` flag is now ignored for
        // this purpose; left in place for the `cycle_check_emit` (which
        // has different semantics).
        let memoized_inner: TokenStream = quote! {
            parser.memoized_call(Self::#rule_const, |parser| {
                #rule_body_inner
            })
        };
        let _is_recursive_unused = is_recursive; // suppress unused warning if any
        let wrapped_rule_call: TokenStream = if self.rule_has_no_semantic_annotations(rule_name) {
            // SV-EXH-PROOF.3.3.4.b.6.2.36.2 — even unannotated rules push
            // their name onto the rule_context_stack so the WHO trace
            // events for any annotated child rule include the full nested
            // call chain (e.g. `class_declaration_sv_2017 >
            // parameter_port_list > mixed_string_parameter_port_list >
            // type_assignment_sv_2017 > declared_type_parameter_identifier`).
            // Without this push/pop here, intermediate unannotated rules
            // were INVISIBLE in the chain trace — breaking the WHY+WHERE
            // visibility guarantee per
            // [[feedback_why_and_where_before_solution]]. Cost: two Vec
            // pushes per rule entry; negligible vs the rule's parse cost.
            quote! {
                let result: ParseResult<ParseNode<'input>> = (|parser: &mut Self| {
                    parser.semantic_runtime_state.push_rule_context_static(#rule_name);
                    let inner_result = #memoized_inner;
                    parser.semantic_runtime_state.pop_rule_context();
                    inner_result.map(|(node, _raw)| node)
                })(self);
            }
        } else {
            quote! {
                let result = self.with_semantic_runtime_rule_transaction(Self::#rule_const, #rule_name, |parser| {
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
                let cycle_type = self.recursion_guard.check_cycle_id(Self::#rule_const, position);

                match cycle_type {
                    CycleType::Infinite => {
                        if self.trace_enabled() {
                            self.logger.log_error(#filename, self.position as u32, &format!("💥 Infinite recursion detected in rule '{}' at position {}", #rule_name, position));
                        }
                        return Err(ParseError::InvalidSyntax {
                            message: "Infinite recursion detected",
                            position,
                        });
                    }
                    CycleType::LeftRecursive => {
                        if self.trace_enabled() {
                            self.logger.log_error(#filename, self.position as u32, &format!("🔄 Left recursion detected in rule '{}' at position {}", #rule_name, position));
                        }
                        return Err(ParseError::InvalidSyntax {
                            message: "Left recursion detected",
                            position,
                        });
                    }
                    CycleType::MutualRecursive { depth, ref rules } if depth >= #recursion_guard_max_depth => {
                        if self.trace_enabled() {
                            self.logger.log_error(#filename, self.position as u32, &format!("🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})", #rule_name, position, depth));
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

                self.recursion_guard.enter_id(Self::#rule_const, #rule_name, position);

                // SV-EXH-PROOF.3.3.4.b.6.2.22 — PER-RULE CALL COUNTER.
                // Single Relaxed atomic add on rule entry (~1ns; lock-free).
                // Always-on so the dashboard can be enabled at any point
                // without rebuild/reconfigure; the cost is negligible vs the
                // rest of the rule body. Uses Self::#rule_const (the
                // codegen-emitted RuleId for THIS rule) as the array index.
                self.rule_call_counts[Self::#rule_const as usize]
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                // GRAMMAR-WELLFORMED.G.4.6 — transactional parse-coverage push.
                // Record this rule entry on the coverage stack (opt-in). Unlike
                // the monotone counter above, this push is rolled back by
                // `try_parse` if the enclosing speculation fails, so on a
                // successful parse only ACCEPTED-tree entries survive. O(1) push
                // of a u32; gated so disabled parsing pays nothing.
                if self.coverage_enabled {
                    self.coverage_stack.push(Self::#rule_const as u32);
                }

                // SV-EXH-PROOF.3.3.4.b.6.2.25 — FURTHEST-POSITION TRACKING.
                // Update the monotone max of position. Single max + assignment
                // on rule entry (~1-2ns). Captures the deepest byte any
                // branch reaches, even if that branch later fails and is
                // backtracked — the whole point is to remember "we got this
                // far" so a failure report can pinpoint the real defect locus
                // instead of just the outermost-rule-start. NEVER restored on
                // speculation; the field is conceptually monotone.
                if self.position > self.furthest_position {
                    self.furthest_position = self.position;
                }

                // SV-EXH-PROOF.3.3.4.b.6.2.17 — rule-level targeted trace push.
                // If `--trace-rules <list>` includes this rule name, enter its
                // call-tree's trace scope (trace_active_depth > 0 enables the
                // per-call log gate via trace_enabled()). Otherwise no-op.
                // Zero overhead when trace_rules is None (the common case).
                let __pgen_trace_scope_active = self
                    .trace_rules
                    .as_ref()
                    .map(|rs| rs.contains(#rule_name))
                    .unwrap_or(false);
                if __pgen_trace_scope_active {
                    self.trace_active_depth += 1;
                }

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

                // SV-EXH-PROOF.3.3.4.b.6.2.17 — rule-level targeted trace pop.
                // Symmetric to the entry push above; exits the call-tree's
                // trace scope. saturating_sub guards against underflow if
                // some unforeseen path skipped the push (defense-in-depth).
                if __pgen_trace_scope_active {
                    self.trace_active_depth = self.trace_active_depth.saturating_sub(1);
                }

                self.recursion_guard.exit();

                match &result {
                    Ok(node) => {
                        if self.trace_enabled() {
                            let consumed = node.span.end - start_pos;
                            if consumed > 0 {
                                let consumed_preview = self.byte_window_lossy(start_pos, node.span.end);
                                self.logger.log_success(#filename, self.position as u32, &format!("✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')", #rule_name, start_pos, node.span.end, consumed, consumed_preview));
                            } else {
                                self.logger.log_warning(#filename, self.position as u32, &format!("⚠️ Rule '{}' matched with zero length at position {}", #rule_name, start_pos));
                            }
                            self.logger.log_success(#filename, self.position as u32, &format!("✅ Exiting rule '{}' successfully - advanced from {} to {}", #rule_name, start_pos, self.position));
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
                        if self.trace_enabled() {
                            self.logger.log_error(#filename, self.position as u32, &format!("❌ Exiting rule '{}' with error: {:?} - backtracked to {}", #rule_name, e, self.position));
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
                // Nested `Or` (inside a sequence / quantifier / lookahead): NOT a
                // rule's top-level body, so no FIRST-set prune guard (its
                // `parse_start` may exceed the rule-entry position that recorded
                // `furthest_position`). RGX-0078.5.c.2.
                self.generate_or_logic(alternatives, rule_name, filename, false)
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
                if parser.trace_enabled() {
                    parser.logger.log_debug(#filename, parser.position as u32, &format!(
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
                if parser.trace_enabled() {
                    parser.logger.log_debug(#filename, parser.position as u32, &format!(
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
        // RGX-0078.5.c.2 — true iff this `Or` is a rule's TOP-LEVEL body (not a
        // nested alternation). Only then is the FIRST-set prune guard emitted:
        // `parse_start` equals the rule-entry position that already recorded
        // `furthest_position`, so pruning a char-1-failing branch is
        // furthest_position-neutral by construction. See `generate_rule_method_with_recursion`.
        top_level: bool,
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
                        if parser.trace_enabled() {
                            parser.logger.log_warning(#filename, parser.position as u32, &format!(
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
            // RGX-0078.5.c.2 — FIRST-set predictive dispatch. Emit a per-branch
            // prune guard ONLY for a top-level `Or` (so `parse_start` is the
            // rule-entry position already folded into `furthest_position` ⇒ pruning
            // is furthest-neutral) in a grammar whose TERMINALS are
            // whitespace-sensitive (no leading-layout skip before a terminal match ⇒
            // peeking the raw next byte is sound). Otherwise no guard is emitted:
            // byte-identical codegen, no regression, no win there.
            let emit_first_set_guard = top_level && self.layout_sensitivity().terminals;
            let mut first_set_cache: std::collections::HashMap<
                String,
                super::first_set::FirstSetSummary,
            > = std::collections::HashMap::new();
            // RGX-0078.5.i.7 D1 — the FIRST₂ sibling cache (per-Or, like the level-1
            // cache; both are persistent-taint-gated inside the analysis).
            let mut second_byte_cache: std::collections::HashMap<
                String,
                super::first_set::SecondByteSummary,
            > = std::collections::HashMap::new();
            // RGX-0078.5.j.4 K4b C1 — the FIRSTₖ per-rule prefix-trie cache
            // (per-Or like its siblings; persistent-taint-gated inside).
            let mut prefix_trie_cache: std::collections::HashMap<
                String,
                super::first_set::PrefixTrieNode,
            > = std::collections::HashMap::new();

            // RGX-0078.5.i.3 (P2) — DEGENERATE-TOURNAMENT BYTE-SWITCH DISPATCH.
            // When FIRST-set pairwise-disjointness proves at most ONE branch can
            // begin a match at any next byte (gates a–e; see
            // `degenerate_dispatch_byte_sets`), the longest-match tournament is
            // protocol-only: emit ONE byte-switch to the sole candidate and elide
            // the per-branch guard-scan loop, the tournament semantic checkpoint,
            // the winner's extract_delta/C3-B-rollback/apply_delta round-trip, the
            // `should_take` cascade, and the partition-offset computation (rotation
            // only permutes evaluation order — irrelevant with ≤1 candidate). The
            // sole candidate still runs under `try_parse` (speculation snapshot/
            // restore for the failure arm); on success its semantic effects stay in
            // place (no losers ran; gate (e) excludes branch predicates/effects).
            // Parse output is byte-identical: same winner, same position, same
            // furthest_position (R1: top-level `Or` ⇒ `parse_start` is the already-
            // recorded rule-entry position), same failure path.
            if let Some(branch_byte_sets) = self.degenerate_dispatch_byte_sets(
                alternatives,
                rule_name,
                emit_first_set_guard,
                &mut first_set_cache,
            ) {
                let mut dispatch_arms = Vec::new();
                for idx in 0..branch_count {
                    let alternative = &alternatives[idx];
                    eprintln!();
                    let branch_logic =
                        self.generate_node_parsing_logic(alternative, rule_name, filename)?;
                    let explicit_annotation: Option<BranchAnnotation> = self
                        .branch_return_annotations
                        .get(rule_name)
                        .and_then(|branches| branches.get(idx).cloned())
                        .flatten();
                    let resolved_annotation: Option<BranchAnnotation> = explicit_annotation
                        .or_else(|| {
                            Self::synthesize_default_passthrough_for_single_element_branch(
                                alternative,
                            )
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
                    let byte_patterns = &branch_byte_sets[idx];
                    dispatch_arms.push(quote! {
                        #(#byte_patterns)|* => {
                            if let Some(content) = parser.try_parse(|p| {
                                let parser = p;
                                if parser.trace_enabled() {
                                    parser.logger.log_info(#filename, parser.position as u32, &format!("🚪 Entering branch {}/{} for rule '{}' at position {}", #branch_num, #branch_count, #rule_name, parser.position));
                                }
                                #branch_logic;
                                if parser.trace_enabled() {
                                    parser.logger.log_info(#filename, parser.position as u32, &format!("✅ Leaving branch {}/{} for rule '{}' at position {} (success)", #branch_num, #branch_count, #rule_name, parser.position));
                                }
                                Ok(result)
                            }) {
                                let candidate_end = parser.position;
                                let raw_content = content;
                                // BRANCH-BROADCAST-FIX.3 order preserved trivially:
                                // the transform runs at candidate_end and nothing
                                // rolls the position back, so `$text` sees the same
                                // span as the general tournament emission.
                                let transformed = {
                                    let content = raw_content.clone();
                                    #transform
                                };
                                semantic_selected_branch_index = Some(#branch_num);
                                if semantic_capture_raw_for_post {
                                    semantic_raw_content = Some(raw_content.clone());
                                }
                                if parser.trace_enabled() {
                                    parser.logger.log_info(#filename, parser.position as u32, &format!(
                                        "🏁 Rule '{}' selected branch {}/{} consuming {} chars (priority={}, associativity={}, branch_policy={})",
                                        #rule_name,
                                        #branch_num,
                                        #branch_count,
                                        candidate_end.saturating_sub(parse_start),
                                        #branch_priority,
                                        #associativity_mode,
                                        #branch_policy_mode
                                    ));
                                }
                                result = transformed;
                            } else {
                                if parser.trace_enabled() {
                                    parser.logger.log_info(#filename, parser.position as u32, &format!("❌ Branch {}/{} for rule '{}' failed at position {}", #branch_num, #branch_count, #rule_name, parser.position));
                                }
                                #recovery_failure_path
                            }
                        }
                    });
                }
                return Ok(quote! {
                    // Multi-branch DEGENERATE dispatch (RGX-0078.5.i.3 P2): the
                    // branches' FIRST-byte sets are pairwise disjoint, so the byte
                    // at parse_start selects the ONLY branch that could match —
                    // the tournament protocol is provably unnecessary here.
                    let parse_start = parser.position;
                    let mut result = ParseContent::Sequence(Vec::new());
                    if parse_start < parser.input.len() {
                        match parser.input.as_bytes()[parse_start] {
                            #(#dispatch_arms,)*
                            _ => {
                                // No branch can begin at this byte (exactly the
                                // all-guards-false outcome of the general loop).
                                #recovery_failure_path
                            }
                        }
                    } else {
                        // EOF: every branch is non-nullable ⇒ nothing can match.
                        #recovery_failure_path
                    }
                });
            }

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
                // RGX-0078.5.c.2 — the FIRST-set prune guard for this branch
                // (`None` ⇒ the branch is always tried: nullable / unresolved /
                // guarding disabled).
                let first_set_prune_guard = self.first_set_prune_guard_for_branch(
                    alternative,
                    emit_first_set_guard,
                    &mut first_set_cache,
                    &mut second_byte_cache,
                    &mut prefix_trie_cache,
                );
                let arm_inner = quote! {
                        if #branch_policy_mode == "ordered" && best_content.is_some() {
                            // Ordered branch policy keeps first successful branch.
                        } else {
                            // RGX-0078.5.j.4 K1 — deferred cleanup of the LIVE
                            // best branch: this branch's body is about to run,
                            // so the previous winner's effects must leave the
                            // store (branch isolation). Its delta is extracted
                            // NOW (it may still win the tournament) and the
                            // store returns to the checkpoint state — exactly
                            // what the eager per-branch cleanup used to do,
                            // paid only when a later branch actually runs.
                            if live_semantic_branch {
                                best_semantic_delta = Some(
                                    parser
                                        .semantic_runtime_state
                                        .extract_delta_since(&tournament_semantic_checkpoint),
                                );
                                parser
                                    .semantic_runtime_state
                                    .rollback_to_labeled(
                                        tournament_semantic_checkpoint.clone(),
                                        crate::ast_pipeline::RollbackLabel::C3bBranchCleanup {
                                            rule: #rule_name,
                                            branch: #branch_num,
                                            total: #branch_count,
                                        },
                                    );
                                live_semantic_branch = false;
                            }
                            parser.position = parse_start;
                            if let Some(content) = parser.try_parse(|p| {
                                let parser = p;
                                if parser.trace_enabled() {
                                    parser.logger.log_info(#filename, parser.position as u32, &format!("🚪 Entering branch {}/{} for rule '{}' at position {}", #branch_num, #branch_count, #rule_name, parser.position));
                                }
                                #branch_logic;
                                if parser.trace_enabled() {
                                    parser.logger.log_info(#filename, parser.position as u32, &format!("✅ Leaving branch {}/{} for rule '{}' at position {} (success)", #branch_num, #branch_count, #rule_name, parser.position));
                                }
                                Ok(result)
                            }) {
                                let candidate_end = parser.position;
                                let candidate_priority: i64 = #branch_priority;
                                let current_branch_index: usize = #branch_index;
                                let raw_content = content;
                                // BRANCH-BROADCAST-FIX.3 — evaluate the branch
                                // transform BEFORE rolling the position back to
                                // parse_start: `$text`/MatchedText slices
                                // `parser.input[start_pos..parser.position]`, so
                                // the rollback-first order made every branch-level
                                // `$text` in a tournament return the EMPTY span.
                                // All other transform forms read the captured
                                // `content` only, so the order is observable to
                                // MatchedText alone.
                                let transformed = {
                                    let content = raw_content.clone();
                                    #transform
                                };
                                parser.position = parse_start;
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
                                            // SV-EXH-PROOF.3.3.4.b.6.2.28 —
                                            // self-explaining predicate trace.
                                            // HIGH = the verdict (one line per
                                            // call site); DBG = the inputs
                                            // (resolved arg values, the spec
                                            // location). Each predicate that
                                            // can affect parse outcome
                                            // explains itself clearly per
                                            // user-set principle (2026-05-24).
                                            let Some(resolved_spec) = parser
                                                .try_resolve_semantic_predicate_spec_against_content(
                                                    spec,
                                                    &raw_content,
                                                    &transformed,
                                                )?
                                            else {
                                                if parser.trace_enabled() {
                                                    parser.logger.log_info(
                                                        #filename,
                                                        parser.position as u32,
                                                        &format!(
                                                            "🛡️ predicate '{}' REJECTED branch {}/{} of rule '{}' — reason: at least one $reference could not be resolved (unresolved args: {:?})",
                                                            spec.name,
                                                            current_branch_index + 1,
                                                            #branch_count,
                                                            #rule_name,
                                                            spec.args,
                                                        ),
                                                    );
                                                    parser.logger.log_debug(
                                                        #filename,
                                                        parser.position as u32,
                                                        &format!(
                                                            "   ↪ spec: {} | phase: Branch | view: {:?} | raw_content_kind: {:?} | transformed_kind: {:?}",
                                                            parser.semantic_predicate_debug_label(spec),
                                                            spec.view,
                                                            std::mem::discriminant(&raw_content),
                                                            std::mem::discriminant(&transformed),
                                                        ),
                                                    );
                                                }
                                                blocked_branch_predicate = Some(
                                                    parser.semantic_predicate_debug_label(spec),
                                                );
                                                branch_predicate_blocked = true;
                                                break;
                                            };
                                            let verdict = parser
                                                .semantic_runtime_state
                                                .evaluate_content_aware_predicate(
                                                    &resolved_spec,
                                                    &raw_content,
                                                    &transformed,
                                                );
                                            if parser.trace_enabled() {
                                                let verdict_label = match verdict {
                                                    Some(true) => "PASSED",
                                                    Some(false) => "REJECTED",
                                                    None => "INAPPLICABLE (treated as no-op)",
                                                };
                                                parser.logger.log_info(
                                                    #filename,
                                                    parser.position as u32,
                                                    &format!(
                                                        "🛡️ predicate '{}' {} branch {}/{} of rule '{}'",
                                                        resolved_spec.name,
                                                        verdict_label,
                                                        current_branch_index + 1,
                                                        #branch_count,
                                                        #rule_name,
                                                    ),
                                                );
                                                parser.logger.log_debug(
                                                    #filename,
                                                    parser.position as u32,
                                                    &format!(
                                                        "   ↪ resolved spec: {} | phase: Branch | view: {:?}",
                                                        parser.semantic_predicate_debug_label(&resolved_spec),
                                                        resolved_spec.view,
                                                    ),
                                                );
                                            }
                                            match verdict {
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
                                        | crate::ast_pipeline::SemanticRuntimeDirective::Predicate(_)
                                        | crate::ast_pipeline::SemanticRuntimeDirective::ExportToLibrary(_)
                                        | crate::ast_pipeline::SemanticRuntimeDirective::ImportFromLibrary(_)
                                        | crate::ast_pipeline::SemanticRuntimeDirective::DeclareFactKind(_)
                                | crate::ast_pipeline::SemanticRuntimeDirective::DefinePredicate(_) => {}
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

                                // SV-EXH-PROOF.3.3.4.b.6.2.33 (C3-B FIX) +
                                // RGX-0078.5.j.4 K1 (lazy refinement) — the
                                // winner's effects STAY LIVE (its extract+
                                // rollback is deferred to the next attempted
                                // branch's preamble; with no later attempt it
                                // commits in place). A loser is rolled back
                                // immediately, WITHOUT the extract its delta
                                // would waste (it is never consumed).
                                // Predicates have already fired above against
                                // this branch's state (correct: predicates
                                // need to see the branch's emissions).
                                if should_take {
                                    live_semantic_branch = true;
                                    // The dethroned previous best's stored
                                    // delta (if any) is dead — this branch's
                                    // effects are the live winner state.
                                    best_semantic_delta = None;
                                    best_end = candidate_end;
                                    best_priority = candidate_priority;
                                    best_branch_index = current_branch_index;
                                    best_branch = #branch_num;
                                    if semantic_capture_raw_for_post {
                                        best_raw_content = Some(raw_content.clone());
                                    }
                                    best_content = Some(transformed);
                                } else {
                                    // SV-EXH-PROOF.3.3.4.b.6.2.36.2 — the
                                    // cleanup rollback stays tagged with the
                                    // owning rule + branch index (deferred
                                    // `RollbackLabel`, the P0 discipline).
                                    parser
                                        .semantic_runtime_state
                                        .rollback_to_labeled(
                                            tournament_semantic_checkpoint.clone(),
                                            crate::ast_pipeline::RollbackLabel::C3bBranchCleanup {
                                                rule: #rule_name,
                                                branch: #branch_num,
                                                total: #branch_count,
                                            },
                                        );
                                }
                                if !should_take && branch_predicate_blocked && parser.logger_enabled {
                                    parser.logger.log_info(#filename, parser.position as u32, &format!(
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
                            } else if parser.trace_enabled() {
                                parser.logger.log_info(#filename, parser.position as u32, &format!("❌ Branch {}/{} for rule '{}' failed at position {}", #branch_num, #branch_count, #rule_name, parser.position));
                            }
                        }
                };
                // RGX-0078.5.c.2 — wrap the branch body in the FIRST-set prune
                // guard when one applies; otherwise emit the arm unchanged
                // (byte-identical to the pre-feature codegen).
                let arm = match first_set_prune_guard {
                    Some(guard_condition) => quote! {
                        #branch_index => {
                            // RGX-0078.5.c.2 — this non-nullable branch's leading
                            // terminals cannot begin at the next input byte, so it
                            // would fail at char 1. Skip its entire body: it
                            // contributes nothing to the longest_match tournament,
                            // and pruning is furthest_position-neutral (the rule
                            // entry already recorded parse_start).
                            if #guard_condition {
                                #arm_inner
                            }
                        }
                    },
                    None => quote! {
                        #branch_index => {
                            #arm_inner
                        }
                    },
                };
                branch_attempt_arms.push(arm);
            }

            // INLINE-ACTIONS.2: apply the WINNING branch's branch-start inline
            // action directives (@emit_fact / @open_scope / @close_scope) for
            // multi-branch rules. Gated per-rule so rules without branch-start
            // actions keep byte-identical parse logic. The directives are cloned
            // out of the registry first so the immutable borrow on `parser` is
            // released before the `&mut self` apply call; `best_branch_index` is
            // the 0-based winner index.
            // `RGX-0078.5.j.4` (K3c): the same `RULE_<NAME>` const the rule's
            // own emission derives (`generate_rule_constants` invariant:
            // rule id = RULE_NAMES index) — lets the winning-branch action
            // lookup skip the rule-name String hash.
            let branch_effect_rule_const = format_ident!("RULE_{}", rule_name.to_uppercase());
            let branch_start_effect_application: TokenStream =
                if self.rule_has_branch_start_effects(rule_name) {
                    quote! {
                        let branch_start_effects: Vec<crate::ast_pipeline::SemanticRuntimeDirective> = parser
                            .semantic_runtime_annotations
                            .branch_effect_directives_for_rule_branch_id(Self::#branch_effect_rule_const, best_branch_index)
                            .cloned()
                            .collect();
                        for branch_start_effect in &branch_start_effects {
                            parser.apply_branch_start_effect_directive(branch_start_effect, &content)?;
                        }
                    }
                } else {
                    quote! {}
                };

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
                // SV-EXH-PROOF.3.3.4.b.6.2.33 (C3-B FIX) — tournament-scope
                // semantic checkpoint + the winner's captured delta.
                // Each branch attempt extracts its delta then rolls back; the
                // winner's delta is replayed at end of tournament. This makes
                // "loser-successful branch emissions accumulate" structurally
                // impossible — only the chosen branch's effects survive.
                let tournament_semantic_checkpoint =
                    parser.semantic_runtime_state.checkpoint();
                let mut best_semantic_delta:
                    Option<crate::ast_pipeline::SemanticRuntimeDelta> = None;
                // RGX-0078.5.j.4 K1 — WINNER-IN-PLACE TOURNAMENT COMMIT.
                // TRUE ⇔ the current best branch's semantic effects are still
                // applied (its extract+rollback was DEFERRED). At most one
                // branch's effects are ever live, and only the current best's:
                // a later ATTEMPTED branch's preamble extracts the live delta
                // into `best_semantic_delta` and rolls back before its body
                // runs (branch isolation exactly as before); a branch that is
                // never attempted (byte-pruned / ordered-skip) triggers no
                // cleanup, so a winner with no later attempts commits IN PLACE
                // — the post-loop `best_semantic_delta` is `None` and the
                // extract/rollback/apply round-trip never happens. Losers are
                // rolled back WITHOUT the extract (their delta is never
                // consumed). End state is identical on every path: checkpoint
                // + exactly the winner's effects.
                let mut live_semantic_branch = false;
                let deterministic_partition_effective_enabled = parser
                    .effective_deterministic_partition_enabled(#deterministic_partition_annotation_enabled);
                // RGX-0078.5.i.2 (P0): the partition-group String is computed
                // ONLY when partitioning is effectively enabled — the previous
                // emit built it unconditionally per Or-body execution (part of
                // the measured −6.5% V3 census surface), even though the
                // disabled path never consumed it.
                let deterministic_partition_offset = if deterministic_partition_effective_enabled {
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group(#rule_name, #deterministic_partition_annotation_group);
                    parser.deterministic_partition_offset_runtime(
                        &deterministic_partition_effective_group,
                        #branch_count,
                    )
                } else {
                    0usize
                };
                // RGX-0078.5.i.2 (P0): iterate the rotated branch order
                // directly as `(step + offset) % n` — provably the same
                // sequence `rotate_left(offset)` produced (offset==0 ⇒ the
                // identity order) — instead of collecting a `Vec<usize>` per
                // Or-body execution (the other half of the V3 census surface).
                for evaluation_step in 0..#branch_count {
                    let branch_index =
                        (evaluation_step + deterministic_partition_offset) % #branch_count;
                    match branch_index {
                        #(#branch_attempt_arms,)*
                        _ => {}
                    }
                }

                if nonassoc_tie {
                    // RGX-0078.5.j.4 K1 — a nonassoc tie fails the WHOLE
                    // choice: if the (dethroned-by-tie) best branch's effects
                    // are still live, discard them before backtracking.
                    if live_semantic_branch {
                        parser
                            .semantic_runtime_state
                            .rollback_to_labeled(
                                tournament_semantic_checkpoint.clone(),
                                crate::ast_pipeline::RollbackLabel::C3bBranchCleanup {
                                    rule: #rule_name,
                                    branch: best_branch,
                                    total: #branch_count,
                                },
                            );
                    }
                    return Err(ParseError::Backtrack {
                        position: parse_start,
                    });
                } else if let Some(content) = best_content {
                    parser.position = best_end;
                    semantic_selected_branch_index = Some(best_branch);
                    if parser.trace_enabled() {
                        parser.logger.log_info(#filename, parser.position as u32, &format!(
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
                    // SV-EXH-PROOF.3.3.4.b.6.2.33 (C3-B FIX) + RGX-0078.5.j.4
                    // K1 — commit ONLY the winning branch's semantic effects.
                    // Two mutually-exclusive paths to the same end state:
                    // `live_semantic_branch` ⇒ the winner's effects never left
                    // the store (`best_semantic_delta` is `None` — nothing to
                    // do, the round-trip is elided); otherwise the state is at
                    // the tournament checkpoint and the winner's extracted
                    // delta is replayed. Either way: checkpoint + exactly the
                    // winner's effects — no loser-branch pollution, no
                    // accumulation across branches.
                    if let Some(delta) = best_semantic_delta {
                        if !delta.is_empty() {
                            parser.semantic_runtime_state.apply_delta(delta);
                        }
                    }
                    // INLINE-ACTIONS.2: winning-branch branch-start inline action
                    // directives fire here — after the winner's body delta is
                    // replayed (so its own emissions are visible) and before the
                    // rule-level effect / post-predicate phase. No-op token block
                    // for rules without branch-start actions (byte-identical).
                    #branch_start_effect_application
                    result = content;
                    semantic_raw_content = best_raw_content;
                } else {
                    #recovery_failure_path
                }
            })
        }
    }

    /// RGX-0078.5.i.3 (P2) — the DEGENERATE-DISPATCH gate for one rule-top-level `Or`:
    /// `Some(per-branch sorted first-byte sets)` iff the site may be emitted as a
    /// degenerate-tournament byte-switch, `None` ⇒ the general tournament emission.
    ///
    /// Gates (the leaf spec's (a)–(e); (a)+(b) arrive pre-computed as `emit` — the
    /// same `top_level && layout_sensitivity().terminals` condition the `.5.c.2`
    /// prune guard uses):
    /// - (c) EVERY branch is first-byte-decided (the SHARED
    ///   `first_set::branch_dispatch_first_bytes` eligibility — also the degeneracy
    ///   census's predicate, so census verdict and emission cannot drift);
    /// - (d) the byte sets are PAIRWISE DISJOINT ⇒ at most one branch can begin a
    ///   match at any next byte ⇒ every branch policy / priority / associativity
    ///   resolves to the same sole candidate and `nonassoc_tie` is impossible;
    /// - (e) the rule has NO Branch-phase predicates and NO branch-start effect
    ///   directives (both would require the tournament's rollback-and-continue
    ///   machinery around a rejected/winning candidate).
    fn degenerate_dispatch_byte_sets(
        &self,
        alternatives: &[ASTNode],
        rule_name: &str,
        emit: bool,
        cache: &mut std::collections::HashMap<String, super::first_set::FirstSetSummary>,
    ) -> Option<Vec<Vec<u8>>> {
        if !emit {
            return None;
        }
        if self.rule_has_branch_start_effects(rule_name)
            || self.rule_has_branch_phase_predicates(rule_name, alternatives.len())
        {
            return None;
        }
        let grammar_tree = self.first_set_grammar_tree.borrow();
        let trust_regex_token_bytes = self.layout_sensitivity().regex_tokens;
        let mut sets: Vec<Vec<u8>> = Vec::with_capacity(alternatives.len());
        let mut seen: std::collections::HashSet<u8> = std::collections::HashSet::new();
        for branch in alternatives {
            let bytes = super::first_set::branch_dispatch_first_bytes(
                branch,
                &grammar_tree,
                cache,
                trust_regex_token_bytes,
            )
            .ok()?;
            for byte in &bytes {
                if !seen.insert(*byte) {
                    // Overlap: a real tournament remains on this byte.
                    return None;
                }
            }
            sets.push(bytes);
        }
        Some(sets)
    }

    /// RGX-0078.5.c.2 — compute the FIRST-set predictive-dispatch prune guard for one
    /// tournament branch, or `None` when the branch must always be tried.
    ///
    /// Returns `Some(cond)` — a boolean token expression that is TRUE iff the branch
    /// could begin a match at the next input byte (`parse_start`) — ONLY for a branch
    /// that is provably prunable: guarding enabled (`emit`), the branch is NOT nullable
    /// and NOT `unresolved` (so its FIRST terminal set is exhaustive), and every FIRST
    /// terminal yields an extractable first byte. In every other case it returns `None`
    /// (always try the branch), which is the SOUND default: the guard can only ever
    /// remove a branch that would have failed at char 1, so soundness rests entirely on
    /// never pruning a branch that could match. The generated condition
    /// (`parse_start < input.len() && matches!(next_byte, …)`) skips the whole branch
    /// body when false. Uses the interior-mutable `first_set_grammar_tree` snapshot so
    /// transitive rule-reference FIRST sets resolve against codegen's own normalized
    /// tree; `cache` is reused across a rule's branches.
    ///
    /// RGX-0078.5.j.4 K4b C1 — FIRSTₖ generalization (k ≤ 4): the guard is now
    /// computed by the ONE shared license `first_set::branch_prefix_trie_guard`
    /// (level-1 admission + bounded per-path prefix trie + the D1 global FIRST₂
    /// fallback layer), and emitted in three forms:
    /// - level-1-degenerate trie ⇒ today's exact level-1 expression
    ///   (byte-identical codegen for every branch C1 does not deepen);
    /// - the uniform w=0 depth-2 rectangle ⇒ today's exact D1 conjunct
    ///   (`parse_start + 1 < len && matches!(b[ps], F₁) && matches!(b[ps+1], F₂)`);
    /// - anything deeper ⇒ a nested byte-walk `match` whose refutation arms
    ///   carry the EXACT furthest emulation (`w = deepest_entry_offset`; w = 0
    ///   emits nothing — provably neutral). Soundness: pure CANNOT-match
    ///   pruning + the C1 exactness license (see the `first_set` module docs);
    ///   the D1 licenses (rolled-back-effect epoch/memo-taint divergence,
    ///   diagnostic-only counter drift) are inherited unchanged.
    fn first_set_prune_guard_for_branch(
        &self,
        branch: &ASTNode,
        emit: bool,
        cache: &mut std::collections::HashMap<String, super::first_set::FirstSetSummary>,
        second_byte_cache: &mut std::collections::HashMap<
            String,
            super::first_set::SecondByteSummary,
        >,
        prefix_trie_cache: &mut std::collections::HashMap<
            String,
            super::first_set::PrefixTrieNode,
        >,
    ) -> Option<TokenStream> {
        if !emit {
            return None;
        }

        let grammar_tree = self.first_set_grammar_tree.borrow();
        // RGX-0078.5.j.4 K4b C1 — ONE shared license computes the whole guard
        // (level-1 admission + per-path FIRSTₖ trie + the D1 fallback layer);
        // the FIRSTₖ census lane consumes the same function, so census verdict
        // and emission cannot drift. `Err` = the branch is always tried
        // (nullable / unresolved / trust-refused / unextractable — exactly the
        // pre-C1 level-1 gates).
        let guard = super::first_set::branch_prefix_trie_guard(
            branch,
            &grammar_tree,
            cache,
            second_byte_cache,
            prefix_trie_cache,
            self.layout_sensitivity().regex_tokens,
        )
        .ok()?;
        let first_byte_patterns: Vec<u8> = guard.root.children.keys().copied().collect();
        if first_byte_patterns.is_empty() {
            return None;
        }

        // Degenerate form 1 — level-1-only trie: today's exact expression
        // (byte-identical codegen for every branch C1 does not deepen).
        if guard.is_level1_degenerate() {
            return Some(quote! {
                parse_start < parser.input.len()
                    && matches!(parser.input.as_bytes()[parse_start], #(#first_byte_patterns)|*)
            });
        }

        // Degenerate form 2 — the uniform w=0 depth-2 rectangle (the D1 shape:
        // identical second-byte set under every first byte, nothing deeper, no
        // emulation): today's exact FIRST₂ conjunct.
        if let Some(second_byte_patterns) = Self::prefix_trie_rectangle_bytes(&guard.root) {
            return Some(quote! {
                parse_start + 1 < parser.input.len()
                    && matches!(parser.input.as_bytes()[parse_start], #(#first_byte_patterns)|*)
                    && matches!(parser.input.as_bytes()[parse_start + 1], #(#second_byte_patterns)|*)
            });
        }

        // General form — the nested byte-walk. Attempt arms yield `true`;
        // falling off the walk refutes with the node's EXACT furthest emulation
        // (`w = deepest_entry_offset`, 0 ⇒ provably neutral, no tokens). `None`
        // (end of input) refutes only non-accepting, non-unresolved nodes —
        // encoded structurally: attempt-terminal children are `true` arms, so
        // the `_` arm of a live node covers exactly EOF + excluded bytes, and
        // both refute with the same w (the real attempt executes the same
        // entries before failing on the byte or on end-of-input).
        Some(Self::prefix_trie_walk_tokens(&guard.root, 0))
    }

    /// C1 — detect the uniform w=0 depth-2 rectangle (today's D1 emission
    /// shape): every root child is the SAME non-terminal node whose children
    /// are all attempt-terminal leaves and whose refutation is emulation-free.
    /// Returns the shared second-byte set.
    fn prefix_trie_rectangle_bytes(root: &super::first_set::PrefixTrieNode) -> Option<Vec<u8>> {
        let mut children = root.children.values();
        let first = children.next()?;
        if first.attempt_terminal()
            || first.deepest_entry_offset != 0
            || first.children.is_empty()
            || !first
                .children
                .values()
                .all(|leaf| leaf.attempt_terminal() && leaf.children.is_empty())
        {
            return None;
        }
        for other in children {
            if !Self::prefix_trie_emission_equal(first, other) {
                return None;
            }
        }
        Some(first.children.keys().copied().collect())
    }

    /// C1 — EMISSION equivalence of two trie nodes (used to group match arms):
    /// attempt-terminal nodes all emit `true` regardless of which flag made
    /// them terminal; live nodes must agree on w and on their (recursively
    /// emission-equal) children.
    fn prefix_trie_emission_equal(
        a: &super::first_set::PrefixTrieNode,
        b: &super::first_set::PrefixTrieNode,
    ) -> bool {
        if a.attempt_terminal() || b.attempt_terminal() {
            return a.attempt_terminal() && b.attempt_terminal();
        }
        a.deepest_entry_offset == b.deepest_entry_offset
            && a.children.len() == b.children.len()
            && a.children.iter().zip(b.children.iter()).all(
                |((byte_a, child_a), (byte_b, child_b))| {
                    byte_a == byte_b && Self::prefix_trie_emission_equal(child_a, child_b)
                },
            )
    }

    /// C1 — the recursive nested-match walk for one live trie node at `depth`.
    /// The whole expression is the guard condition (parenthesized match).
    fn prefix_trie_walk_tokens(
        node: &super::first_set::PrefixTrieNode,
        depth: usize,
    ) -> TokenStream {
        // Group children by emission equivalence so identical subtrees share
        // one arm (deterministic: BTreeMap byte order, first-seen group order).
        let mut groups: Vec<(Vec<u8>, &super::first_set::PrefixTrieNode)> = Vec::new();
        for (byte, child) in &node.children {
            if let Some((bytes, _)) = groups
                .iter_mut()
                .find(|(_, member)| Self::prefix_trie_emission_equal(member, child))
            {
                bytes.push(*byte);
            } else {
                groups.push((vec![*byte], child));
            }
        }
        let arms = groups.iter().map(|(bytes, child)| {
            let byte_patterns = bytes.iter();
            if child.attempt_terminal() {
                quote! { Some(#(#byte_patterns)|*) => true }
            } else {
                let inner = Self::prefix_trie_walk_tokens(child, depth + 1);
                quote! { Some(#(#byte_patterns)|*) => #inner }
            }
        });
        let index = if depth == 0 {
            quote! { parse_start }
        } else {
            quote! { parse_start + #depth }
        };
        let w = node.deepest_entry_offset as usize;
        let refute = if w == 0 {
            quote! { false }
        } else {
            quote! {
                {
                    if parse_start + #w > parser.furthest_position {
                        parser.furthest_position = parse_start + #w;
                    }
                    false
                }
            }
        };
        quote! {
            (match parser.input.as_bytes().get(#index).copied() {
                #(#arms,)*
                _ => #refute,
            })
        }
    }

    /// RGX-0078.5.i.7 Q-GUARD — the min-0 quantified-site ATTEMPT-ELISION license:
    /// the element's admissible FIRST bytes plus whether the guard must emit the
    /// EXACT furthest emulation. `None` = always attempt (the sound default).
    /// The gates mirror the census lane (`QuantSiteCensus`) one-for-one:
    /// - terminal-layout trust (`layout_sensitivity().terminals` — the R2 raw-byte
    ///   peek, same gate as `emit_first_set_guard`);
    /// - the SHARED dispatch predicate `first_set::branch_dispatch_first_bytes`
    ///   (non-nullable + resolved + regex-token trust + extractable bytes — census
    ///   gate 3, so the census verdict and this emission cannot drift);
    /// - the SHARED frontier classification
    ///   (`first_set::quantified_element_frontier`): `BareRef` ⇒ guard + emulation
    ///   (a refuted attempt always executes the referenced entry's furthest
    ///   preamble at the attempt position; the memo-hit case is exact by
    ///   monotonicity), `NoRefs` ⇒ guard without emulation (no furthest writer
    ///   exists in a pure-terminal attempt), `Mixed` ⇒ unguarded (exact emulation
    ///   undecidable at this granularity);
    /// - the census gate-4 mirror: NO rule reachable from the element subtree
    ///   carries Branch-phase predicates or branch-start effect directives
    ///   (rolled back on failure anyway; excluded so counters/diagnostics stay
    ///   honest by exclusion — the `-0075` P2-(e) mirror), queried against the
    ///   SAME compiled table the census consults.
    fn quantified_prune_guard_for_element(&self, element: &ASTNode) -> Option<(Vec<u8>, bool)> {
        if !self.layout_sensitivity().terminals {
            return None;
        }
        let emulate = match super::first_set::quantified_element_frontier(element) {
            super::first_set::QuantFrontier::BareRef => true,
            super::first_set::QuantFrontier::NoRefs => false,
            super::first_set::QuantFrontier::Mixed => return None,
        };
        let grammar_tree = self.first_set_grammar_tree.borrow();
        let mut cache: std::collections::HashMap<String, super::first_set::FirstSetSummary> =
            std::collections::HashMap::new();
        let bytes = super::first_set::branch_dispatch_first_bytes(
            element,
            &grammar_tree,
            &mut cache,
            self.layout_sensitivity().regex_tokens,
        )
        .ok()?;
        let mut element_refs: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        super::fusibility_census::collect_ref_occurrences(element, &mut element_refs);
        let reachable = super::fusibility_census::reachable_rules(
            &grammar_tree,
            element_refs.keys().cloned(),
        );
        for rule in &reachable {
            let branch_count = match grammar_tree.get(rule.as_str()) {
                Some(ASTNode::Or { alternatives }) => alternatives.len(),
                _ => 1,
            };
            if self.rule_has_branch_phase_predicates(rule, branch_count)
                || self.rule_has_branch_start_effects(rule)
            {
                return None;
            }
        }
        Some((bytes, emulate))
    }

    /// RGX-0078.5.i.7 Q-GUARD — the emitted EXACT furthest emulation (`BareRef`
    /// sites), or nothing (`NoRefs` sites). Shared by both emission sites (the
    /// quantifier loop and the optional-element fast path).
    fn quantified_guard_emulation_tokens(emulate: bool) -> TokenStream {
        if emulate {
            quote! {
                if parser.position > parser.furthest_position {
                    parser.furthest_position = parser.position;
                }
            }
        } else {
            quote! {}
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
            // RGX-0078.5.d.4.i — explicit element type so the `&mut` from
            // `arena.alloc` coerces to the shared `&'input` the Vec holds.
            let mut sequence_elements: Vec<&'input ParseNode<'input>> = Vec::with_capacity(#element_count);
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
                let attempt = quote! {
                    if let Some(content) = parser.try_parse(|p| {
                        let parser = p;
                        #inner_logic;
                        Ok(result)
                    }) {
                        content
                    } else {
                        ParseContent::Sequence(Vec::new())
                    }
                };
                // RGX-0078.5.i.7 Q-GUARD — the optional-element fast path is the
                // SECOND emission site of the min-0 attempt elision (this arm
                // bypasses `generate_quantified_logic`): when the next byte
                // cannot start the element, take the empty arm directly with
                // exact furthest parity (see
                // `quantified_prune_guard_for_element`). Unguarded sites emit
                // the `attempt` tokens verbatim (byte-identical codegen).
                match self.quantified_prune_guard_for_element(element) {
                    Some((bytes, emulate)) => {
                        let emulation = Self::quantified_guard_emulation_tokens(emulate);
                        quote! {
                            if parser.position < parser.input.len()
                                && matches!(parser.input.as_bytes()[parser.position], #(#bytes)|*)
                            {
                                #attempt
                            } else {
                                // RGX-0078.5.i.7 Q-GUARD: FIRST-refuted optional
                                // — the attempt would fail at byte 1; elide it.
                                #emulation
                                ParseContent::Sequence(Vec::new())
                            }
                        }
                    }
                    None => attempt,
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

                // RGX-0078.5.d.4.i — arena-alloc the child element and store the
                // `&'input` borrow (the `ParseContent::Sequence` Vec holds refs).
                sequence_elements.push(parser.arena.alloc(ParseNode {
                    rule_name: #element_name,
                    content: element_content,
                    span: element_start..element_end,
                }));
            }
        })
    }

    /// RGX-0078.5.i.12 — terminal-literal specialization: the ONE shared
    /// decision point for every constant-literal emission site (the
    /// census↔emission single-implementation doctrine). A literal takes the
    /// emitted `match_lit_ascii` fast path iff it is non-empty, all-ASCII,
    /// and at most 8 bytes — the population for which a successful byte-match
    /// proves both UTF-8 slice boundaries (an ASCII byte is never a
    /// continuation byte) and a const-width compare replaces the libc
    /// `memcmp` call. Everything else keeps `match_string` verbatim,
    /// including the 8-bit-unclean and long-literal populations.
    fn terminal_literal_match_call(literal: &str) -> TokenStream {
        if Self::terminal_literal_takes_ascii_fast_path(literal) {
            let byte_lit = proc_macro2::Literal::byte_string(literal.as_bytes());
            quote! { match_lit_ascii(#literal, #byte_lit) }
        } else {
            quote! { match_string(#literal) }
        }
    }

    /// The ONE fast-path predicate behind both terminal-literal selectors
    /// (protocol and bare), so the two emissions cannot drift.
    fn terminal_literal_takes_ascii_fast_path(literal: &str) -> bool {
        !literal.is_empty() && literal.len() <= 8 && literal.is_ascii()
    }

    /// RGX-0078.5.j.4 (-0202) — the BARE twin selector for fused-body terminal
    /// sites: the same decision, dispatched to the `_bare` twins that
    /// construct the Copy `CascadeControlError` at the source.
    fn terminal_literal_match_call_bare(literal: &str) -> TokenStream {
        if Self::terminal_literal_takes_ascii_fast_path(literal) {
            let byte_lit = proc_macro2::Literal::byte_string(literal.as_bytes());
            quote! { match_lit_ascii_bare(#literal, #byte_lit) }
        } else {
            quote! { match_string_bare(#literal) }
        }
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
                        let match_call = Self::terminal_literal_match_call(token_value_str);
                        Ok(quote! {
                            let matched_str = parser.#match_call?;
                            #constraint_guards
                            let result = ParseContent::Terminal(matched_str)
                        })
                    }
                    "rule_reference" => {
                        // RGX-0078.5.i.9 (D3) — a call site of a boundary-scanner
                        // plan rule dispatches to the frameless `scan_<rule>` on
                        // the BARE path (the D2-A twin-dispatch precedent at call
                        // granularity); the protocol arm below stays byte-VERBATIM
                        // for every diagnostic consumer. Plan-inactive generations
                        // emit the unchanged protocol call only.
                        let scan_dispatch: Option<TokenStream> = if self.scan_rule(token_value_str)
                        {
                            let scan_target = Self::scan_fn_ident(token_value_str);
                            Some(quote! { parser.#scan_target() })
                        } else {
                            None
                        };
                        // RGX-0078.5.i.4 (P1a) — a call site of a DECIDED rule
                        // receives the rule body inline under the emitted
                        // `inlined_frame_call` engine helper (memo preserved,
                        // per-frame observability verbatim) instead of the
                        // method call. Decision from the SHARED census function
                        // (gates (a)–(d) + budget) via the once-per-generation
                        // plan; no plan (unit tests / analysis failure) ⇒ the
                        // method call below, byte-identical to pre-P1a.
                        if self.inline_decided(token_value_str) {
                            eprintln!(
                                "        Inlining rule reference '{}' (P1a) - File: {}:{}",
                                token_value_str,
                                file!(),
                                line!()
                            );
                            let inlined_frame =
                                self.generate_inlined_frame(token_value_str, filename)?;
                            if let Some(scan_call) = scan_dispatch {
                                return Ok(quote! {
                                    let __pgen_alt_child = if parser.bare_parse {
                                        #scan_call
                                    } else {
                                        #inlined_frame
                                    }?;
                                    let result = ParseContent::Alternative(parser.arena.alloc(__pgen_alt_child))
                                });
                            }
                            return Ok(quote! {
                                let __pgen_alt_child = #inlined_frame?;
                                let result = ParseContent::Alternative(parser.arena.alloc(__pgen_alt_child))
                            });
                        }
                        eprintln!(
                            "        Generating rule reference call to '{}' - File: {}:{}",
                            token_value_str,
                            file!(),
                            line!()
                        );
                        let method = format_ident!("parse_{}", token_value_str);
                        if let Some(scan_call) = scan_dispatch {
                            return Ok(quote! {
                                let __pgen_alt_child = if parser.bare_parse {
                                    #scan_call
                                } else {
                                    parser.#method()
                                }?;
                                let result = ParseContent::Alternative(parser.arena.alloc(__pgen_alt_child))
                            });
                        }
                        Ok(quote! {
                            // RGX-0078.5.d.4.i — the child rule returns an owned
                            // `ParseNode`; arena-alloc it and wrap the `&'input`
                            // borrow (hoisted so the `&mut self` call finishes
                            // before the arena read, mirroring the interpreter).
                            let __pgen_alt_child = parser.#method()?;
                            let result = ParseContent::Alternative(parser.arena.alloc(__pgen_alt_child))
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
                                                        if parser.trace_enabled() {
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
                                                if parser.trace_enabled() {
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
                        let match_call = Self::terminal_literal_match_call(token_value_str);
                        Ok(quote! {
                            let matched_str = parser.#match_call?;
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

        // SV-EXH-PROOF.3.3.4.b.3 (Layer 0, PGEN-SV-EXH-PROOF-0029, 2026-05-21):
        // Unified quantifier codegen — replaces the prior three special-cased
        // arms for `*` / `+` / `?` with a single (min, max)-parameterised
        // loop. Per-iteration atomicity is now UNIFORM across all repetition
        // operators (every iteration — INCLUDING the first iteration of `+`,
        // which previously was emitted inline without `try_parse` wrapping —
        // is wrapped in `try_parse`, so if iteration (N+1) fails the cursor
        // rolls back to where iteration N left it). The `min` count is
        // enforced at the end of the loop: if the loop didn't reach `min`
        // iterations, the rule fails (Err(Backtrack)) — the surrounding
        // caller's `try_parse` then rolls the cursor back fully. Bounded
        // quantifiers (`{N}` / `{N,M}` / `{N,}` / `{,M}`) are now also
        // well-formed at codegen time (the prior `_ => Err("Unknown
        // quantifier")` fallthrough is replaced by `parse_quantifier_bounds`);
        // no current grammar exercises bounded operators end-to-end so the
        // codegen is parser-agnostic infrastructure available for future use.
        let (min, max) = match parse_quantifier_bounds(quantifier) {
            Some(bounds) => bounds,
            None => return Err(anyhow::anyhow!("Unknown quantifier: {}", quantifier)),
        };

        let max_check_tokens = match max {
            Some(m) => {
                let m_lit = proc_macro2::Literal::usize_unsuffixed(m);
                quote! {
                    if iteration_count >= #m_lit {
                        break;
                    }
                }
            }
            None => quote! {},
        };
        // Only emit a min-count enforcement clause + the
        // `quantifier_start_position` bind when min > 0; emitting
        // `iteration_count < 0` (for `*`/`?`) is always false and triggers an
        // unused-comparison warning at every codegen site (thousands of them
        // in a parser the size of systemverilog), and the `_position` bind
        // would be unused.
        let (quantifier_start_position_bind, min_check_tokens) = if min > 0 {
            let min_lit = proc_macro2::Literal::usize_unsuffixed(min);
            (
                quote! { let quantifier_start_position = parser.position; },
                quote! {
                    if iteration_count < #min_lit {
                        parser.position = quantifier_start_position;
                        return Err(ParseError::Backtrack {
                            position: quantifier_start_position,
                        });
                    }
                },
            )
        } else {
            (quote! {}, quote! {})
        };
        let quantifier_label = quantifier;

        // RGX-0078.5.i.7 Q-GUARD — FIRST-guarded ATTEMPT ELISION at min-0 sites:
        // when the next byte cannot start the element, the iteration attempt
        // would fail at byte 1 (the `.5.c.2` argument at the quantifier
        // boundary), so skip it and take the loop's exit path directly. Fires at
        // EVERY iteration boundary (iteration 0 = the elided sole attempt;
        // iteration k>0 = the elided loop-EXIT attempt). min>0 sites are out of
        // scope (a refuted first attempt fails the whole quantifier — a
        // different emission). Furthest-position parity is EXACT per the
        // license's frontier class (see `quantified_prune_guard_for_element`).
        let quant_prune_guard = if min == 0 {
            self.quantified_prune_guard_for_element(element)
        } else {
            None
        };
        let quant_guard_tokens = match &quant_prune_guard {
            Some((bytes, emulate)) => {
                let emulation = Self::quantified_guard_emulation_tokens(*emulate);
                quote! {
                    // RGX-0078.5.i.7 Q-GUARD: FIRST-refuted next byte ⇒ the
                    // attempt would fail at byte 1 — elide it (exit the loop
                    // with the iterations committed so far).
                    if parser.position >= parser.input.len()
                        || !matches!(parser.input.as_bytes()[parser.position], #(#bytes)|*)
                    {
                        #emulation
                        break;
                    }
                }
            }
            None => quote! {},
        };

        Ok(quote! {
            // SV-EXH-PROOF.3.3.4.b.3 (Layer 0): the quantifier is ATOMIC at
            // its own boundary. When min > 0 we save the cursor at the start;
            // on min-failure we restore the cursor to that position BEFORE
            // signalling Err, so the caller observes a cursor position
            // equivalent to "the quantifier was never tried." Per-iteration
            // atomicity is delegated to `try_parse` inside the loop;
            // quantifier-level atomicity is delegated to this explicit
            // save/restore (elided for min == 0 since `*`/`?` always succeed).
            #quantifier_start_position_bind
            // RGX-0078.5.d.4.i — explicit element type so the `&mut` from
            // `arena.alloc` coerces to the shared `&'input` the Vec holds.
            let mut results: Vec<&'input ParseNode<'input>> = Vec::new();
            let mut last_position = parser.position;
            let mut iteration_count: usize = 0;
            const SAFETY_LIMIT: usize = 10_000;

            loop {
                if iteration_count >= SAFETY_LIMIT {
                    if parser.trace_enabled() {
                        parser.logger.log_warning(#filename, parser.position as u32, &format!(
                            "⚠️ SAFETY_LIMIT ({}) reached in quantifier {} at position {}",
                            SAFETY_LIMIT, #quantifier_label, parser.position
                        ));
                    }
                    break;
                }

                #stop_at_rule_boundary_on_break
                #max_check_tokens
                #quant_guard_tokens

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

                    // Zero-length match guard — prevent infinite loops on rules
                    // that can match the empty string.
                    if current_position == last_position {
                        if parser.trace_enabled() {
                            parser.logger.log_warning(#filename, parser.position as u32, &format!(
                                "⚠️ ZERO-LENGTH MATCH in quantifier {} at position {}: breaking to prevent infinite loop",
                                #quantifier_label, current_position
                            ));
                        }
                        break;
                    }

                    // RGX-0078.5.d.4.i — arena-alloc the per-iteration node and
                    // store the `&'input` borrow (`ParseContent::Quantified` Vec).
                    results.push(parser.arena.alloc(node));
                    last_position = current_position;
                    iteration_count += 1;
                } else {
                    // Iteration (N+1) failed: try_parse has already rolled the
                    // cursor back to where iteration N left it (= where
                    // iteration N+1 started). Exit the loop with iteration_count
                    // iterations committed.
                    break;
                }
            }

            // Min-count enforcement: if the loop didn't reach `min` iterations,
            // the whole quantifier fails. We RESTORE the cursor to
            // `quantifier_start_position` (atomic at the quantifier boundary —
            // so the partial successes that did happen are undone) before
            // signalling Err. Elided entirely for `min == 0` (i.e. `*` / `?`)
            // to avoid the always-false `iteration_count < 0` comparison.
            #min_check_tokens

            let result = ParseContent::Quantified(results, #quantifier_label);
        })
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
    pub(crate) fn body_has_single_element(node: &ASTNode) -> bool {
        match node {
            ASTNode::Sequence { elements } => elements.len() <= 1,
            // A `+`/`*`/`?` body emits a `Quantified(...)` that holds ALL
            // matches. Synthesizing `-> $1` here would extract `elements[0]`
            // and silently drop every iteration past the first (e.g.
            // `concatenation = piece+` would only ever surface the first
            // piece). The natural reading of `$1` on a Quantified body is
            // "the whole capture group" — i.e. raw passthrough — and that's
            // already what no-transform produces, so an implicit default
            // here would either be wrong (current `elements[0]` shape) or
            // redundant. Authors who want a non-trivial transform on a
            // Quantified body must declare it explicitly (e.g. `-> [$1*]`).
            ASTNode::Quantified { .. } => false,
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

    /// REGEX-SELF-HOSTING.4c: emit a rule-level `@transform` applied to the matched SPAN text, for
    /// the case where the rule body is NOT a single terminal (so the terminal-path `@transform` at
    /// the atom codegen — which inlines `match_regex(...).parse::<T>()` — did not fire). Lets a
    /// self-hosted literal body (e.g. `digits = digit+`) keep its typed result (`usize`) without a
    /// `/.../` body. Returns empty tokens when the rule has no `@transform`.
    ///
    /// The emitted block REBINDS `result` only when it is not already a `TransformedTerminal` — so
    /// it is a no-op for terminal-body `@transform` rules (which already produced one), making this
    /// purely additive (no double-apply, no regression on existing `@transform` rules in any grammar).
    /// `start_pos` / `parser.position` are in scope at the splice point (the rule's matched span).
    fn generate_post_body_span_transform(&self, rule_name: &str) -> TokenStream {
        let Some(annotations) = &self.annotations else {
            return quote! {};
        };
        let Some(semantic_annotations) = annotations.semantic_annotations.get(rule_name) else {
            return quote! {};
        };
        for semantic_annotation in semantic_annotations {
            if Self::semantic_directive_name(semantic_annotation).as_deref() != Some("transform") {
                continue;
            }
            if let UnifiedSemanticAST::TransformExpr { expression } = semantic_annotation.ast() {
                if let Some(transform) = parse_canonical_transform_expression(expression) {
                    if let Ok(target_type) = syn::parse_str::<syn::Type>(&transform.target_type) {
                        let default_expr: syn::Expr = syn::parse_str(&transform.default_expr)
                            .unwrap_or_else(|_| {
                                syn::parse_str("0").expect("fallback default expression")
                            });
                        return quote! {
                            let result = if matches!(result, ParseContent::TransformedTerminal(_)) {
                                result
                            } else {
                                let __pgen_span_text = parser.input[start_pos..parser.position].trim();
                                let __pgen_transformed =
                                    __pgen_span_text.parse::<#target_type>().unwrap_or(#default_expr);
                                ParseContent::TransformedTerminal(__pgen_transformed.to_string())
                            };
                        };
                    }
                }
            }
        }
        quote! {}
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

    /// GRAMMAR-WELLFORMED.H.11.5: does any terminal token of `grammar_tree`
    /// assign `introducer` a NON-COMMENT meaning?
    ///
    /// This is the static emit-time input for per-introducer comment-arm
    /// suppression in the generated layout skippers. The engine's hard-coded
    /// comment convention (`#`-to-EOL, `//`-to-EOL, `/* */`) is an EBNF
    /// meta-grammar convenience; when a grammar defines a real token that can
    /// begin with one of those introducers (SV `#` delays/param lists, VHDL
    /// `#` based-literal delimiters), the corresponding arm can steal that
    /// token while the parser is speculatively attempting a DIFFERENT token —
    /// the dynamic `regex_token_matches_at_cursor` guard (H.11.3) only
    /// protects the active token, not the dual case — and the stolen span can
    /// be memoized as bogus trivia, poisoning the whole parse (released SV
    /// parser bug, `PGEN-GRAMMAR-WELLFORMED-0074`). Such an arm is wrong for
    /// that grammar on EVERY input, so it is not emitted at all.
    ///
    /// The discriminator is non-comment MEANING, not mere prefix overlap: a
    /// claiming terminal that is itself COMMENT-DEFINING — a regex whose
    /// every match starts with the introducer and runs an unbounded content
    /// tail (SV `line_comment`/`block_comment`), or an introducer literal
    /// followed by an unbounded content terminal (the ebnf meta-grammar's
    /// `("#" | "//") comment_content`) — agrees with the arm about what those
    /// bytes mean, so the arm stays (shipped status quo; measured by the
    /// decisive stash A/B, suppressing the ebnf grammar's arms regressed the
    /// generated ebnf parser on real grammar files whose mid-rule comments
    /// the meta-grammar does not yet structurally own).
    fn grammar_claims_introducer_as_non_comment(
        &self,
        grammar_tree: &HashMap<String, ASTNode>,
        introducer: &str,
    ) -> bool {
        grammar_tree.iter().any(|(rule_name, node)| {
            self.node_has_non_comment_claim(rule_name, node, introducer, grammar_tree, None)
        })
    }

    /// The three per-introducer comment-arm suppression decisions for the layout skippers — codegen's
    /// own kernel, exposed so the parse-harness interpreter (`PARSE-HARNESS.5.2`) mirrors the generated
    /// parser's layout skipping byte-for-byte. See [`CommentArmSuppression`] +
    /// [`comment_arm_suppression_for_grammar`]. Used by codegen's layout-skipper emission (H.11.5) is
    /// left inline for a byte-identical emit path; this method is the shared read-only query.
    pub(crate) fn comment_arm_suppression(
        &self,
        grammar_tree: &HashMap<String, ASTNode>,
    ) -> CommentArmSuppression {
        CommentArmSuppression {
            claims_hash: self.grammar_claims_introducer_as_non_comment(grammar_tree, "#"),
            claims_line_comment: self.grammar_claims_introducer_as_non_comment(grammar_tree, "//"),
            claims_block_comment: self.grammar_claims_introducer_as_non_comment(grammar_tree, "/*"),
        }
    }

    /// Walk `node` looking for a terminal that can START a match with
    /// `introducer` but is not comment-defining. `follower` is the element
    /// that comes immediately after `node` in its enclosing sequence (used to
    /// recognize the two-token comment shape `INTRODUCER content_tail`).
    fn node_has_non_comment_claim(
        &self,
        rule_name: &str,
        node: &ASTNode,
        introducer: &str,
        grammar_tree: &HashMap<String, ASTNode>,
        follower: Option<&ASTNode>,
    ) -> bool {
        match node {
            ASTNode::Or { alternatives } => alternatives.iter().any(|alt| {
                self.node_has_non_comment_claim(rule_name, alt, introducer, grammar_tree, follower)
            }),
            ASTNode::Sequence { elements } => elements.iter().enumerate().any(|(i, el)| {
                let next = elements.get(i + 1).or(follower);
                self.node_has_non_comment_claim(rule_name, el, introducer, grammar_tree, next)
            }),
            ASTNode::Quantified { element, .. } => self.node_has_non_comment_claim(
                rule_name,
                element,
                introducer,
                grammar_tree,
                follower,
            ),
            // A lookahead consumes nothing — the claim must come from the
            // consuming terminal, which appears elsewhere in the tree.
            ASTNode::Lookahead { .. } => false,
            ASTNode::Atom { value } => match value {
                ASTValue::Node(inner) => self.node_has_non_comment_claim(
                    rule_name,
                    inner,
                    introducer,
                    grammar_tree,
                    follower,
                ),
                ASTValue::Token(parts) if parts.len() >= 2 => {
                    let TokenValue::String(token_type) = &parts[0];
                    let TokenValue::String(token_value) = &parts[1];
                    match token_type.as_str() {
                        "regex" => {
                            // The parser matches the steered/effective
                            // pattern, so the analysis must look at the same.
                            // A regex CLAIMS the introducer only when every
                            // match MANDATORILY starts with it (a content
                            // class like `[^\r\n]*` merely CAN start with it
                            // and is no claim); the claim is non-comment when
                            // the pattern has no unbounded comment tail.
                            let pattern = self.effective_regex_pattern(rule_name, token_value);
                            Self::regex_pattern_mandatorily_starts_with(&pattern, introducer)
                                && !Self::regex_pattern_is_comment_defining(&pattern, introducer)
                        }
                        "rule_reference" => false,
                        // quoted_string + the literal matcher token types all
                        // match their value verbatim. An introducer-prefixed
                        // literal is comment-defining only when an unbounded
                        // content terminal immediately follows it.
                        _ => {
                            token_value.starts_with(introducer)
                                && !follower.is_some_and(|next| {
                                    Self::node_is_unbounded_content(next, grammar_tree, 4)
                                })
                        }
                    }
                }
                ASTValue::Token(_) => false,
            },
        }
    }

    /// A regex terminal is comment-defining for `introducer` when EVERY match
    /// starts with the introducer (its mandatory leading bytes equal the
    /// introducer) and the pattern carries an unbounded content tail. SV's
    /// `line_comment` / `block_comment` qualify; VHDL's `/#/` based-literal
    /// delimiter does not (no unbounded tail), and neither does any pattern
    /// that only OPTIONALLY starts with the introducer.
    fn regex_pattern_is_comment_defining(pattern: &str, introducer: &str) -> bool {
        let Ok(hir) = regex_syntax::parse(pattern.trim()) else {
            return false;
        };
        let mut mandatory: Vec<u8> = Vec::new();
        Self::hir_mandatory_prefix(&hir, introducer.len(), &mut mandatory);
        mandatory.starts_with(introducer.as_bytes()) && Self::hir_has_unbounded_repetition(&hir)
    }

    /// Accumulate into `out` the bytes that EVERY match of `hir` must start
    /// with, stopping at the first point of divergence or once `k` bytes are
    /// known. Returns `true` while the walk is still on a fixed prefix (a
    /// concat caller may continue with its next part).
    fn hir_mandatory_prefix(hir: &regex_syntax::hir::Hir, k: usize, out: &mut Vec<u8>) -> bool {
        use regex_syntax::hir::{Class, HirKind};
        if out.len() >= k {
            return false;
        }
        match hir.kind() {
            HirKind::Empty | HirKind::Look(_) => true,
            HirKind::Literal(lit) => {
                let bytes: &[u8] = &lit.0;
                let take = bytes.len().min(k - out.len());
                out.extend_from_slice(&bytes[..take]);
                // A literal is wholly fixed; the caller may continue.
                true
            }
            HirKind::Class(class) => {
                // Only a single-codepoint class contributes a fixed byte.
                let single = match class {
                    Class::Unicode(c) => {
                        let mut iter = c.iter();
                        match (iter.next(), iter.next()) {
                            (Some(r), None)
                                if r.start() == r.end() && (r.start() as u32) < 128 =>
                            {
                                Some(r.start() as u8)
                            }
                            _ => None,
                        }
                    }
                    Class::Bytes(c) => {
                        let mut iter = c.iter();
                        match (iter.next(), iter.next()) {
                            (Some(r), None) if r.start() == r.end() => Some(r.start()),
                            _ => None,
                        }
                    }
                };
                match single {
                    Some(b) => {
                        out.push(b);
                        true
                    }
                    None => false,
                }
            }
            HirKind::Capture(cap) => Self::hir_mandatory_prefix(&cap.sub, k, out),
            HirKind::Concat(parts) => {
                for part in parts {
                    if out.len() >= k {
                        break;
                    }
                    if !Self::hir_mandatory_prefix(part, k, out) {
                        return false;
                    }
                }
                true
            }
            HirKind::Alternation(alts) => {
                // The mandatory contribution is the longest common fixed
                // prefix of every branch; branches diverge after it.
                let mut common: Option<Vec<u8>> = None;
                for alt in alts {
                    let mut branch = Vec::new();
                    Self::hir_mandatory_prefix(alt, k - out.len(), &mut branch);
                    common = Some(match common {
                        None => branch,
                        Some(prev) => {
                            let shared = prev
                                .iter()
                                .zip(branch.iter())
                                .take_while(|(a, b)| a == b)
                                .count();
                            prev[..shared].to_vec()
                        }
                    });
                }
                if let Some(common) = common {
                    out.extend_from_slice(&common);
                }
                false
            }
            HirKind::Repetition(rep) => {
                if rep.min == 0 {
                    // Nullable repetition: nothing past here is mandatory.
                    return false;
                }
                // The first `min` iterations are mandatory; iteration counts
                // diverge beyond that unless the repetition is exact.
                for _ in 0..rep.min {
                    if out.len() >= k {
                        return false;
                    }
                    if !Self::hir_mandatory_prefix(&rep.sub, k, out) {
                        return false;
                    }
                }
                rep.max == Some(rep.min)
            }
        }
    }

    fn hir_has_unbounded_repetition(hir: &regex_syntax::hir::Hir) -> bool {
        use regex_syntax::hir::HirKind;
        match hir.kind() {
            HirKind::Repetition(rep) => {
                rep.max.is_none() || Self::hir_has_unbounded_repetition(&rep.sub)
            }
            HirKind::Concat(parts) | HirKind::Alternation(parts) => {
                parts.iter().any(Self::hir_has_unbounded_repetition)
            }
            HirKind::Capture(cap) => Self::hir_has_unbounded_repetition(&cap.sub),
            _ => false,
        }
    }

    /// Is `node` an unbounded content terminal — a regex with an unbounded
    /// repetition — possibly behind a quantifier, group, sequence head, or a
    /// rule reference (`depth` bounds the deref hops)? This recognizes the
    /// content tail of the two-token comment shape, e.g. the ebnf
    /// meta-grammar's `comment_content := /([^\r\n]*)/`.
    fn node_is_unbounded_content(
        node: &ASTNode,
        grammar_tree: &HashMap<String, ASTNode>,
        depth: usize,
    ) -> bool {
        if depth == 0 {
            return false;
        }
        match node {
            ASTNode::Or { alternatives } => alternatives
                .iter()
                .any(|alt| Self::node_is_unbounded_content(alt, grammar_tree, depth - 1)),
            ASTNode::Sequence { elements } => elements
                .first()
                .is_some_and(|el| Self::node_is_unbounded_content(el, grammar_tree, depth - 1)),
            ASTNode::Quantified { element, .. } => {
                Self::node_is_unbounded_content(element, grammar_tree, depth - 1)
            }
            ASTNode::Lookahead { .. } => false,
            ASTNode::Atom { value } => match value {
                ASTValue::Node(inner) => {
                    Self::node_is_unbounded_content(inner, grammar_tree, depth - 1)
                }
                ASTValue::Token(parts) if parts.len() >= 2 => {
                    let TokenValue::String(token_type) = &parts[0];
                    let TokenValue::String(token_value) = &parts[1];
                    match token_type.as_str() {
                        "regex" => regex_syntax::parse(token_value.trim())
                            .map(|hir| Self::hir_has_unbounded_repetition(&hir))
                            .unwrap_or(false),
                        "rule_reference" => grammar_tree.get(token_value).is_some_and(|body| {
                            Self::node_is_unbounded_content(body, grammar_tree, depth - 1)
                        }),
                        _ => false,
                    }
                }
                ASTValue::Token(_) => false,
            },
        }
    }

    /// Does EVERY match of `pattern` start with `prefix`? (The introducer is
    /// the token's mandatory head — SV's `"#"`, vhdl's `/#/`, SV's
    /// `line_comment` — as opposed to a content class like `[^\r\n]*` that
    /// merely CAN start with it.) A token whose introducer prefix is only
    /// optional is deliberately not treated as a claim: the dynamic
    /// active-token guard still protects it when it is the active token, and
    /// treating possible-prefix as a claim would mis-flag every open content
    /// class (measured: it suppressed the ebnf meta-grammar's arms via
    /// `comment_content := /([^\r\n]*)/` and regressed the generated ebnf
    /// parser flow of the dual-run gate).
    fn regex_pattern_mandatorily_starts_with(pattern: &str, prefix: &str) -> bool {
        let Ok(hir) = regex_syntax::parse(pattern.trim()) else {
            return false;
        };
        let mut mandatory: Vec<u8> = Vec::new();
        Self::hir_mandatory_prefix(&hir, prefix.len(), &mut mandatory);
        mandatory.starts_with(prefix.as_bytes())
    }


    fn generate_helper_methods(
        &self,
        filename: &str,
        grammar_tree: &HashMap<String, ASTNode>,
    ) -> TokenStream {
        // RGX-0078.5.i.4 (P1a + P1b) — the inlined-wrapper-frame engine helper,
        // emitted ONLY when ≥1 rule is decided for inlining (grammars with no
        // decided rule regenerate without a dead helper). ONE definition carries
        // the preserved per-frame observability for EVERY inlined site — entry
        // `fetch_add`, transactional coverage push, furthest-position, and the
        // method-identical exit trace lines + negative-case recording — so an
        // inlined site duplicates only the rule body and the observability
        // protocol cannot drift from the rule-method emission. Elided vs a
        // method call: recursion-guard enter/exit (callers are gated on gate (a)
        // acyclicity), rule-context push/pop and the `--trace-rules` scope probe
        // (both proven trace-only), the method call frame, and — since P1b — the
        // packrat memo (probes + inserts; the body runs directly, result-neutral
        // by the memo's soundness contract; counters change truthfully where
        // former hits re-execute). The rule METHOD keeps its memoized_call.
        let inlined_frame_call_helper: TokenStream = if self.inline_plan_active() {
            quote! {
                fn inlined_frame_call<F>(
                    &mut self,
                    rule_id: RuleId,
                    rule_name: &'static str,
                    negative_case_enabled: bool,
                    negative_case_strict: bool,
                    f: F,
                ) -> ParseResult<ParseNode<'input>>
                where
                    F: FnOnce(&mut Self) -> ParseResult<(ParseNode<'input>, Option<ParseContent<'input>>)>,
                {
                    self.rule_call_counts[rule_id as usize]
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    if self.coverage_enabled {
                        self.coverage_stack.push(rule_id as u32);
                    }
                    if self.position > self.furthest_position {
                        self.furthest_position = self.position;
                    }
                    let start_pos = self.position;
                    // RGX-0078.5.i.4 (P1b) — the memo is ELIDED at inlined
                    // frames: the body runs directly, paying neither the
                    // fail-set/tainted-map/success-map probes nor the
                    // success-insert (node.clone() + delta/coverage
                    // extraction). Result-neutral by the memo's own soundness
                    // contract (a pure, taint-gated cache — replay ≡
                    // re-execution wherever a replay was legal); a former hit
                    // re-executes the acyclic, budget-capped body, whose
                    // non-decided children keep their own memoized methods.
                    // Rule-entry counters change TRUTHFULLY where former hits
                    // re-execute (they count real executions); the 💾 memo
                    // trace lines vanish at inlined frames (trace-only). The
                    // rule METHOD keeps its memoized_call untouched.
                    let result: ParseResult<ParseNode<'input>> =
                        f(self).map(|(node, _raw)| node);
                    match &result {
                        Ok(node) => {
                            if self.trace_enabled() {
                                let consumed = node.span.end - start_pos;
                                if consumed > 0 {
                                    let consumed_preview = self.byte_window_lossy(start_pos, node.span.end);
                                    self.logger.log_success(#filename, self.position as u32, &format!("✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')", rule_name, start_pos, node.span.end, consumed, consumed_preview));
                                } else {
                                    self.logger.log_warning(#filename, self.position as u32, &format!("⚠️ Rule '{}' matched with zero length at position {}", rule_name, start_pos));
                                }
                                self.logger.log_success(#filename, self.position as u32, &format!("✅ Exiting rule '{}' successfully - advanced from {} to {}", rule_name, start_pos, self.position));
                            }
                        }
                        Err(e) => {
                            if negative_case_enabled {
                                self.record_negative_case_failure(
                                    rule_name,
                                    start_pos,
                                    self.position,
                                    negative_case_strict,
                                    &format!("{:?}", e),
                                );
                            }
                            if self.trace_enabled() {
                                self.logger.log_error(#filename, self.position as u32, &format!("❌ Exiting rule '{}' with error: {:?} - backtracked to {}", rule_name, e, self.position));
                            }
                        }
                    }
                    result
                }
            }
        } else {
            quote! {}
        };
        // `WS-DIRECTIVE.2`: the layout policy comes from the grammar-level
        // `@whitespace_sensitive:` directive (the grammar-NAME gate is
        // retired) — see `Self::layout_sensitivity`.
        let layout_sensitivity = self.layout_sensitivity();
        let allow_layout_skip_for_terminals = !layout_sensitivity.terminals;
        let allow_layout_skip_for_regexes = !layout_sensitivity.regex_tokens;
        // GRAMMAR-WELLFORMED.H.11.5: per-introducer static comment-arm
        // suppression — see `grammar_claims_introducer_as_non_comment`. An
        // introducer the grammar assigns a non-comment meaning loses its arm
        // in BOTH layout skippers (`consume_layout_for_regex` /
        // `consume_layout_for_terminal`).
        let claims_hash = self.grammar_claims_introducer_as_non_comment(grammar_tree, "#");
        let claims_line_comment =
            self.grammar_claims_introducer_as_non_comment(grammar_tree, "//");
        let claims_block_comment =
            self.grammar_claims_introducer_as_non_comment(grammar_tree, "/*");
        let any_comment_arm = !(claims_hash && claims_line_comment && claims_block_comment);
        // GRAMMAR-WELLFORMED.H.11.5: the regex-side layout skipper's comment
        // arms, each emitted ONLY when the grammar does not claim the arm's
        // introducer. Each remaining arm keeps the dynamic H.11.3 guard
        // (active token wins over the comment convention) as defense in
        // depth.
        let regex_hash_arm = if claims_hash {
            quote! {}
        } else {
            quote! {
                if bytes[self.position] == b'#' {
                    if self.regex_token_matches_at_cursor(pattern) {
                        break;
                    }
                    while self.position < self.input.len() {
                        let b = bytes[self.position];
                        if b == b'\n' || b == b'\r' {
                            break;
                        }
                        self.position += 1;
                    }
                    continue;
                }
            }
        };
        let regex_line_comment_arm = if claims_line_comment {
            quote! {}
        } else {
            quote! {
                if self.position + 1 < bytes.len()
                    && bytes[self.position] == b'/'
                    && bytes[self.position + 1] == b'/'
                {
                    if self.regex_token_matches_at_cursor(pattern) {
                        break;
                    }
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
            }
        };
        let regex_block_comment_arm = if claims_block_comment {
            quote! {}
        } else {
            quote! {
                if self.position + 1 < bytes.len()
                    && bytes[self.position] == b'/'
                    && bytes[self.position + 1] == b'*'
                {
                    if self.regex_token_matches_at_cursor(pattern) {
                        break;
                    }
                    self.position += 2;
                    while self.position + 1 < bytes.len()
                        && !(bytes[self.position] == b'*' && bytes[self.position + 1] == b'/')
                    {
                        self.position += 1;
                    }
                    if self.position + 1 < bytes.len() {
                        self.position += 2;
                    }
                    continue;
                }
            }
        };
        let layout_skip_regex_fns = if any_comment_arm {
            quote! {
            // GRAMMAR-WELLFORMED.H.11.3: does the active token's own (anchored)
            // pattern match at the current cursor? Used by the layout skipper's
            // comment arms so a token that IS a comment introducer (e.g. the VHDL
            // based-literal `#`) wins over the engine's comment convention —
            // the regex-side mirror of the introducer guard
            // `consume_layout_for_terminal` already applies to string terminals.
            // Cold path: only reached when a comment introducer sits at the
            // cursor, so it keeps a small sibling cache instead of widening the
            // function-scoped cache inside `match_regex`.
            fn regex_token_matches_at_cursor(&self, pattern: &str) -> bool {
                use std::cell::RefCell;
                use std::collections::HashMap;
                thread_local! {
                    static GUARD_REGEX_CACHE: RefCell<HashMap<String, regex::Regex>> =
                        RefCell::new(HashMap::new());
                }
                GUARD_REGEX_CACHE.with(|cache| {
                    let mut cache = cache.borrow_mut();
                    if !cache.contains_key(pattern) {
                        let Ok(compiled) = regex::Regex::new(&format!(r"\A(?:{})", pattern)) else {
                            return false;
                        };
                        cache.insert(pattern.to_string(), compiled);
                    }
                    let re = cache.get(pattern).expect("just inserted");
                    let Some(haystack) = self.input.get(self.position..) else {
                        return false;
                    };
                    re.find(haystack).map(|m| m.start() == 0).unwrap_or(false)
                })
            }
            fn consume_layout_for_regex(&mut self, can_match_empty: bool, pattern: &str) {
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

                    #regex_hash_arm
                    #regex_line_comment_arm
                    #regex_block_comment_arm

                    if self.position == before {
                        break;
                    }
                }
            }
            }
        } else {
            // GRAMMAR-WELLFORMED.H.11.5: every comment introducer is claimed
            // by a grammar token, so NO engine comment arm is emitted — the
            // grammar owns its comment/`#` surface explicitly, and the
            // skipper reduces to whitespace skipping. The dynamic-guard
            // helper would be dead code here, so it is elided too.
            quote! {
            fn consume_layout_for_regex(&mut self, can_match_empty: bool, _pattern: &str) {
                if can_match_empty {
                    // Empty-matching regexes must not cross line boundaries implicitly.
                    self.consume_horizontal_whitespace();
                    return;
                }
                self.consume_optional_whitespace();
            }
            }
        };
        // GRAMMAR-WELLFORMED.H.11.5: the string-terminal-side layout skipper,
        // with the same per-introducer static arm suppression as the
        // regex side. Remaining arms keep the original expected-token guard
        // (`allow_comment_skip`) — the terminal-side dynamic defense.
        let terminal_hash_arm = if claims_hash {
            quote! {}
        } else {
            quote! {
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
            }
        };
        let terminal_line_comment_arm = if claims_line_comment {
            quote! {}
        } else {
            quote! {
                if self.position + 1 < bytes.len()
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
            }
        };
        let terminal_block_comment_arm = if claims_block_comment {
            quote! {}
        } else {
            quote! {
                if self.position + 1 < bytes.len()
                    && bytes[self.position] == b'/'
                    && bytes[self.position + 1] == b'*'
                {
                    self.position += 2;
                    while self.position + 1 < bytes.len()
                        && !(bytes[self.position] == b'*' && bytes[self.position + 1] == b'/')
                    {
                        self.position += 1;
                    }
                    if self.position + 1 < bytes.len() {
                        self.position += 2;
                    }
                    continue;
                }
            }
        };
        let consume_layout_for_terminal_fn = if any_comment_arm {
            quote! {
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

                    #terminal_hash_arm
                    #terminal_line_comment_arm
                    #terminal_block_comment_arm

                    if self.position == before {
                        break;
                    }
                }
            }
            }
        } else {
            // GRAMMAR-WELLFORMED.H.11.5: every comment introducer is claimed
            // by a grammar token — no engine comment arm; layout is
            // whitespace only.
            quote! {
            fn consume_layout_for_terminal(&mut self, _expected: &str) {
                self.consume_optional_whitespace();
            }
            }
        };
        // REGEX-SELF-HOSTING.6a: emit the `match_regex` helper (Rust `regex` engine) ONLY when the grammar
        // uses a `/.../` regex literal. A fully-literal grammar (regex) gets an empty fragment, so the
        // generated parser neither defines nor links the regex engine. `uses_match_regex` is computed
        // during codegen (after the rule methods) before this runs.
        // RGX-0078.5.j.4 (`-0200`) — the BARE speculation wrapper, emitted only
        // when the cascade plan is active (its only call sites are the two
        // cascade body emissions). Differences from the protocol `try_parse`,
        // each licensed by an invariant of the bare path:
        //   1. recursion-guard snapshot/restore is ID-only — bare fused frames
        //      maintain only `rule_id_stack` (`enter_id_bare`/`exit_bare`).
        //      `parse_stack` needs no snapshot because it is BALANCED across
        //      `f`: a cascade body's only paired pushes happen inside
        //      out-of-plan protocol METHOD calls, and a protocol method always
        //      restores the name stack before returning (its `exit()` runs
        //      unconditionally on the captured result, and its own inner
        //      speculations restore per-stack via `truncate_stacks`);
        //   2. the rollback label maps the innermost retained `RuleId` through
        //      `RULE_NAMES` — the same frame (hence the same label value) that
        //      `parse_stack.last()` names in a paired parse, and the label
        //      only materializes under trace anyway;
        //   3. no trace branches: `bare_parse ⇒ !logger_enabled ⇒
        //      !trace_enabled` (the documented invariant), so the protocol
        //      wrapper's gated log calls are statically dead here.
        // Position, coverage, and semantic checkpoint/rollback semantics are
        // IDENTICAL to the protocol wrapper.
        let try_parse_bare_helper = if self.cascade_plan_active() {
            quote! {
                // RGX-0078.5.j.4 (`-0201`) — the bare wrapper carries NO
                // coverage snapshot/truncate: it runs only under `bare_parse`
                // (⇒ `!coverage_enabled`), every `coverage_stack.push` is
                // `coverage_enabled`-gated, and the fused graph emits no
                // pushes, so the stack length is invariant across the bare
                // region. The rollback uses the bare twin, which skips only
                // the diagnostic-only `rollbacks_nonempty_chain`
                // classification per the documented observed-parse boundary.
                // RGX-0078.5.j.4 (`-0202`) — the closure's error channel is
                // the Copy `CascadeResult` (every caller is a fused-graph
                // speculation), so the discarded `Err(_)` compiles to no drop
                // code — the executed `drop_in_place::<Result<(),
                // ParseError>>` this lever removes.
                fn try_parse_bare<F, T>(&mut self, f: F) -> Option<T>
                where
                    F: FnOnce(&mut Self) -> CascadeResult<T>,
                {
                    let saved_pos = self.position;
                    let saved_id_stack_len = self.recursion_guard.rule_id_stack.len();
                    let saved_semantic_checkpoint =
                        self.semantic_runtime_state.checkpoint();
                    match f(self) {
                        Ok(result) => Some(result),
                        Err(_) => {
                            self.position = saved_pos;
                            let try_parse_rule: Option<&'static str> = self
                                .recursion_guard
                                .rule_id_stack
                                .last()
                                .and_then(|(rid, _)| {
                                    Self::RULE_NAMES.get(*rid as usize).copied()
                                });
                            self.recursion_guard
                                .rule_id_stack
                                .truncate(saved_id_stack_len);
                            self.semantic_runtime_state.rollback_to_labeled_bare(
                                saved_semantic_checkpoint,
                                crate::ast_pipeline::RollbackLabel::TryParseErr(try_parse_rule),
                            );
                            None
                        }
                    }
                }
                /// RGX-0078.5.j.4 (`-0202`) — TOTAL inbound conversion at
                /// fused boundary call-outs (protocol methods, scanners,
                /// `match_regex`): the three Copy-shaped variants map 1:1
                /// with no slot write; a rich/legacy error is PARKED in
                /// `cascade_parked_error` and carried as `Parked`, so the
                /// public payload survives the Copy channel byte-identically.
                fn cascade_error_from_parse(&mut self, e: ParseError) -> CascadeControlError {
                    match e {
                        ParseError::InvalidSyntax { message, position } => {
                            CascadeControlError::InvalidSyntax { message, position }
                        }
                        ParseError::Backtrack { position } => {
                            CascadeControlError::Backtrack { position }
                        }
                        ParseError::RecursionDepthExceeded { position, depth } => {
                            CascadeControlError::RecursionDepthExceeded { position, depth }
                        }
                        rich => {
                            self.cascade_parked_error = Some(rich);
                            CascadeControlError::Parked
                        }
                    }
                }
                /// RGX-0078.5.j.4 (`-0202`) — outbound rehydration at the
                /// region's only escape edge (the sub-root orchestrators'
                /// failure arms): bijective 1:1 for the mirrors; `Parked`
                /// takes the parked rich error. A live `Parked` without a
                /// parked value is codegen drift, not an input error (the
                /// MTB-A tape stance).
                fn rehydrate_cascade_error(&mut self, e: CascadeControlError) -> ParseError {
                    match e {
                        CascadeControlError::InvalidSyntax { message, position } => {
                            ParseError::InvalidSyntax { message, position }
                        }
                        CascadeControlError::Backtrack { position } => {
                            ParseError::Backtrack { position }
                        }
                        CascadeControlError::RecursionDepthExceeded { position, depth } => {
                            ParseError::RecursionDepthExceeded { position, depth }
                        }
                        CascadeControlError::Parked => self
                            .cascade_parked_error
                            .take()
                            .expect(
                                "cascade parked-error invariant: a live Parked marker always has a parked value",
                            ),
                    }
                }
            }
        } else {
            quote! {}
        };

        // RGX-0078.5.j.4 (`-0202`) — the BARE terminal twins for fused-body
        // sites: identical match semantics to `match_lit_ascii`/`match_string`
        // with every diagnostic branch dropped (statically dead on the bare
        // path — the emitted routing invariant `bare_parse ⇒ !logger_enabled ⇒
        // !trace_enabled`), constructing the Copy `CascadeControlError` at the
        // source. `match_string_bare`'s defensive UTF-8 boundary rich error is
        // PARKED so its public payload is byte-identical when it escapes.
        // `#[allow(dead_code)]`: a cascade plan whose fused bodies carry no
        // terminal atoms emits the twins unused.
        let bare_terminal_twins = if self.cascade_plan_active() {
            quote! {
                #[allow(dead_code)]
                #[inline(always)]
                fn match_lit_ascii_bare<const N: usize>(
                    &mut self,
                    expected: &'static str,
                    expected_bytes: &[u8; N],
                ) -> CascadeResult<&'input str> {
                    if #allow_layout_skip_for_terminals {
                        self.consume_layout_for_terminal(expected);
                    }
                    let start = self.position;
                    let end = start + N;
                    if end <= self.input.len()
                        && self.input.as_bytes()[start..end] == *expected_bytes
                    {
                        self.position = end;
                        return Ok(expected);
                    }
                    Err(CascadeControlError::Backtrack { position: start })
                }
                #[allow(dead_code)]
                fn match_string_bare(&mut self, expected: &str) -> CascadeResult<&'input str> {
                    if #allow_layout_skip_for_terminals {
                        self.consume_layout_for_terminal(expected);
                    }
                    let start = self.position;
                    let expected_bytes = expected.as_bytes();
                    let end = start + expected_bytes.len();
                    if self.bytes_match_at(start, expected_bytes) {
                        if !self.input.is_char_boundary(start) || !self.input.is_char_boundary(end) {
                            let __pgen_rich = self.create_contextual_error(&format!(
                                "Internal UTF-8 boundary mismatch while matching '{}'",
                                expected
                            ));
                            self.cascade_parked_error = Some(__pgen_rich);
                            return Err(CascadeControlError::Parked);
                        }
                        self.position = end;
                        return Ok(&self.input[start..end]);
                    }
                    Err(CascadeControlError::Backtrack { position: start })
                }
            }
        } else {
            quote! {}
        };

        let match_regex_helper = if self.uses_match_regex.get() {
            quote! {
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
                // PARSE-TERMINATION.7: cache (Regex, can_match_empty) together. `can_match_empty`
                // is a STATIC property of the pattern (does it match ""), so it is computed ONCE
                // at compile/insert — NOT recomputed per call (the former `re.find("")` on every
                // match_regex call was millions of redundant regex executions on large inputs).
                thread_local! {
                    static REGEX_CACHE: RefCell<HashMap<String, (regex::Regex, bool)>> =
                        RefCell::new(HashMap::new());
                }

                // Phase 1: ensure pattern is compiled + cached (with its precomputed
                // can_match_empty) and read the cached bool — no per-call regex execution.
                let can_match_empty: bool = REGEX_CACHE.with(|cache| -> Result<bool, regex::Error> {
                    let mut cache = cache.borrow_mut();
                    if !cache.contains_key(pattern) {
                        // PARSE-TERMINATION.7.1: ANCHOR the terminal match at the parse position.
                        // match_regex only ever accepts a match at offset 0 (it filters
                        // m.start()==0), but an UNANCHORED `find` scans the ENTIRE remaining
                        // haystack (up to MBs) on every FAILING terminal attempt (the common PEG
                        // ordered-choice case) looking for the pattern elsewhere, then discards it
                        // — O(remaining input) per call = the dominant parse-time root (uvm). A
                        // leading `\A` (with `(?:..)` to preserve the pattern's precedence) makes
                        // `find` anchored => O(match length), no haystack scan. Semantically
                        // identical (same start-0 match or None). The cache key stays the ORIGINAL
                        // pattern so call sites still share the compiled instance.
                        let compiled = regex::Regex::new(&format!(r"\A(?:{})", pattern))?;
                        let empties = compiled
                            .find("")
                            .map(|m| m.start() == 0 && m.end() == 0)
                            .unwrap_or(false);
                        cache.insert(pattern.to_string(), (compiled, empties));
                    }
                    let (_re, empties) = cache.get(pattern).expect("just inserted");
                    if #allow_layout_skip_for_regexes {
                        Ok(*empties)
                    } else {
                        Ok(false)
                    }
                }).map_err(|e| self.create_contextual_error(&format!(
                    "Invalid regex pattern '{}': {}",
                    pattern, e
                )))?;

                if skip_leading_whitespace && #allow_layout_skip_for_regexes {
                    self.consume_layout_for_regex(can_match_empty, pattern);
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
                    let (re, _empties) = cache.get(pattern).expect("compiled in phase 1");
                    re.find(haystack).filter(|m| m.start() == 0).map(|m| m.end())
                });

                if let Some(end_offset) = match_end {
                    let start = self.position;
                    self.position += end_offset;
                    if self.trace_enabled() {
                        self.logger.log_success(#filename, self.position as u32, &format!(
                            "✅ Regex '{}' matched at position {} (len {})",
                            pattern, start, end_offset
                        ));
                    }
                    if let Some(slice) = self.input.get(start..self.position) {
                        return Ok(slice);
                    }
                    return Err(self.create_contextual_error("Regex matched invalid UTF-8 span"));
                }

                if self.trace_enabled() {
                    let preview = if self.position < self.input.len() {
                        let end = (self.position + 10).min(self.input.len());
                        self.byte_window_lossy(self.position, end)
                    } else {
                        "<EOF>".to_string()
                    };
                    self.logger.log_error(#filename, self.position as u32, &format!("❌ Regex '{}' no match at position {} (next: '{}')", pattern, self.position, preview));
                }

                Err(self.create_contextual_error(&format!(
                    "No match for regex pattern '{}'",
                    pattern
                )))
            }
            #layout_skip_regex_fns
            }
        } else {
            quote! {}
        };
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

                if self.trace_enabled() {
                    let marker = if critical_path {
                        "critical"
                    } else {
                        "target"
                    };
                    self.logger.log_info(#filename, self.position as u32, &format!(
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

                if self.trace_enabled() {
                    self.logger.log_info(#filename, self.position as u32, &format!(
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

                if self.trace_enabled() {
                    let mode = if negative {
                        "near-invalid"
                    } else {
                        "invalid-case"
                    };
                    self.logger.log_info(#filename, self.position as u32, &format!(
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
                        if self.trace_enabled() {
                            self.logger.log_warning(#filename, self.position as u32, &format!(
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
                        if self.trace_enabled() {
                            self.logger.log_warning(#filename, self.position as u32, &format!(
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
                        if self.trace_enabled() {
                            self.logger.log_warning(#filename, self.position as u32, &format!(
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

                    if self.trace_enabled() {
                        let marker = if token_priority == 0 {
                            "panic_until"
                        } else {
                            "sync"
                        };
                        self.logger.log_warning(#filename, self.position as u32, &format!(
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
                    if self.trace_enabled() {
                        self.logger.log_warning(#filename, self.position as u32, &format!(
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
            /// FINAL-PHASE-PREDICATE.3: is this `$reference` a NAMED / object-key
            /// reference (eligible for the other-view fallback) rather than a
            /// POSITIONAL `$N` one? Mirrors the named-vs-positional split in
            /// `resolve_semantic_reference`: `$` followed by a digit is positional
            /// (walks the raw tree by index — never retried against the other view);
            /// everything else (`$name`, dotted `$a.b`, a bare `name`) is named.
            fn semantic_reference_is_named(reference: &str) -> bool {
                let normalized = reference.trim();
                let core = normalized.strip_suffix(".len").unwrap_or(normalized);
                match core.strip_prefix('$') {
                    Some(body) => !body
                        .trim()
                        .as_bytes()
                        .first()
                        .map(|byte| byte.is_ascii_digit())
                        .unwrap_or(false),
                    None => !core.trim().is_empty(),
                }
            }
            fn resolve_positional_semantic_reference(
                &self,
                root_content: &ParseContent<'input>,
                reference: &str,
            ) -> Option<String> {
                let (index, path_segments) = self.parse_semantic_reference_segments(reference)?;
                let mut current_node = match root_content {
                    // RGX-0078.5.d.4.i — children are `&'input` refs; deref the
                    // `&&ParseNode` from `.get()` / the matched ref to `&ParseNode`.
                    ParseContent::Sequence(elements) => *elements.get(index.saturating_sub(1))?,
                    ParseContent::Alternative(node) => {
                        if index == 1 {
                            *node
                        } else {
                            return None;
                        }
                    }
                    ParseContent::Quantified(elements, _) => *elements.get(index.saturating_sub(1))?,
                    _ => return None,
                };

                // SV-EXH-PROOF.3.3.4.a.2 (PGEN-SV-EXH-PROOF-0027): segments
                // may be either property names (`name`) or bracketed
                // indexed-access forms (`[N]`). Dispatch per segment:
                // bracketed → pick the N'th child of a Sequence/Quantified
                // (raw-tree indexed walk); otherwise → named-descendant
                // search. Strict bracket policy: a malformed `[N]` was
                // already rejected at the lexer surface, so we only need
                // to dispatch here.
                for segment in path_segments {
                    if let Some(index) = Self::parse_bracketed_index(segment) {
                        current_node =
                            self.find_semantic_indexed_child(&current_node.content, index)?;
                    } else {
                        current_node =
                            self.find_semantic_named_descendant(&current_node.content, segment)?;
                    }
                }

                self.semantic_node_scalar(current_node)
            }
            fn resolve_named_semantic_reference(
                &self,
                root_content: &ParseContent<'input>,
                reference: &str,
            ) -> Option<String> {
                // SEMREF-SHAPED: when the rule's content is a shaped
                // `->` return-annotation structure (`ParseContent::Shaped`),
                // a semantic-annotation `$name` / `$a.b` reference
                // resolves **against that produced structure** — a
                // JSON object-key path lookup down to a scalar leaf —
                // rather than the raw parse tree. This is the
                // "shaped-only for `->` rules" contract: the rule's
                // declared output is its semantic surface. The branch
                // is keyed purely on the content variant, so rules
                // without a `->` (raw `Sequence`/`Quantified`/
                // `Alternative` content) fall straight through to the
                // unchanged raw sub-rule-name resolution below and are
                // byte-identical. A missing key, a non-object
                // intermediate, or a non-scalar / null leaf yields
                // `None` — surfaced upstream as the same hard
                // `ContextualError` an unresolved ref already raises
                // (referencing a field the `->` does not produce, or a
                // non-scalar, is an author/grammar bug — fail loud,
                // never silently mis-parse, never panic).
                // SV-EXH-PROOF.3.3.4.a.2 (PGEN-SV-EXH-PROOF-0027): lex
                // the reference into segments where each segment is
                // either a property name (`name`) or a bracketed
                // indexed-access form (`[N]`). The lexer is the same
                // helper used by `parse_semantic_reference_segments` for
                // the positional path; both walks dispatch identically.
                // Strict trailing-dot / strict-bracket policy enforced
                // at the lexer surface (malformed forms → `None`).
                let lexed_segments = Self::lex_semantic_reference_segments_named(reference)?;
                if lexed_segments.is_empty() {
                    return None;
                }

                // RGX-0078.5.i.7 (`-0105`): the shaped `->` carrier is now
                // `Shaped` (arena `PgenValue`); the walk mirrors the retired
                // `Json` walk byte-exactly — `[N]` indexes only arrays,
                // `.name` looks up only objects (the sorted-pair binary
                // search = `serde_json::Map::get`), any mismatch or missing
                // key is a resolution failure, and scalar leaves render
                // through serde's OWN formatters (`serde_json::Number`), so
                // the produced text is identical by construction.
                if let ParseContent::Shaped(shaped_root) = root_content {
                    let mut current = *shaped_root;
                    for segment in &lexed_segments {
                        if let Some(index) = Self::parse_bracketed_index(segment) {
                            current = match current {
                                PgenValue::Array(items) => *items.get(index)?,
                                _ => return None,
                            };
                        } else {
                            if !self.semantic_identifier(segment) {
                                return None;
                            }
                            current = match current {
                                PgenValue::Object(pairs) => match pairs
                                    .binary_search_by(|(key, _)| key.as_bytes().cmp(segment.as_bytes()))
                                {
                                    Ok(found) => pairs[found].1,
                                    Err(_) => return None,
                                },
                                _ => return None,
                            };
                        }
                    }
                    return match current {
                        PgenValue::Str(text) => Some(text.to_string()),
                        PgenValue::Int(number) => Some(serde_json::Number::from(number).to_string()),
                        PgenValue::UInt(number) => Some(serde_json::Number::from(number).to_string()),
                        PgenValue::Float(number) => {
                            serde_json::Number::from_f64(number).map(|rendered| rendered.to_string())
                        }
                        PgenValue::Bool(boolean) => Some(boolean.to_string()),
                        // Null / Array / Object: not a scalar leaf →
                        // resolution failure (loud upstream).
                        _ => None,
                    };
                }

                let mut iter = lexed_segments.iter();
                let first = iter.next()?;
                if !self.semantic_identifier(first) {
                    return None;
                }

                let mut current_node = self.find_semantic_named_descendant(root_content, first)?;
                for segment in iter {
                    if let Some(index) = Self::parse_bracketed_index(segment) {
                        current_node =
                            self.find_semantic_indexed_child(&current_node.content, index)?;
                    } else {
                        if !self.semantic_identifier(segment) {
                            return None;
                        }
                        current_node =
                            self.find_semantic_named_descendant(&current_node.content, segment)?;
                    }
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

                // SV-EXH-PROOF.3.3.4.a.2 (PGEN-SV-EXH-PROOF-0027): the
                // suffix now accepts a mix of `.<ident>` and `[<digits>]`
                // segments. Lexer returns `Vec<&str>` where each
                // `[<digits>]` segment is preserved with its brackets
                // (caller dispatches via `parse_bracketed_index`); strictly
                // additive over the prior dotted-only form.
                let suffix = normalized[index_end..].trim();
                let segments = Self::lex_semantic_reference_segments_suffix(suffix)?;

                // Backwards-compat validation: property-segments must
                // pass `semantic_identifier` (the lexer enforces character
                // class but `semantic_identifier` may add further checks
                // such as reserved-keyword rejection in some embedders).
                for segment in &segments {
                    if Self::parse_bracketed_index(segment).is_none()
                        && !self.semantic_identifier(segment)
                    {
                        return None;
                    }
                }

                Some((index, segments))
            }

            fn find_semantic_indexed_child<'a>(
                &self,
                content: &'a ParseContent<'input>,
                index: usize,
            ) -> Option<&'a ParseNode<'input>> {
                // SV-EXH-PROOF.3.3.4.a.2 (PGEN-SV-EXH-PROOF-0027): raw-tree
                // companion to `find_semantic_named_descendant` for
                // `[N]` indexed walks. For `Sequence`/`Quantified` content,
                // returns the `index`'th element directly (no recursion;
                // index access has explicit numeric semantics, not a
                // "find by name" search). For `Alternative` content,
                // index 0 returns the wrapped node. For non-array-like
                // content, returns `None`.
                match content {
                    // RGX-0078.5.d.4.i — `.copied()` turns `Option<&&ParseNode>`
                    // into `Option<&'input ParseNode>`; `*node` derefs the ref.
                    ParseContent::Sequence(elements)
                    | ParseContent::Quantified(elements, _) => elements.get(index).copied(),
                    ParseContent::Alternative(node) if index == 0 => Some(*node),
                    _ => None,
                }
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
                                // RGX-0078.5.d.4.i — deref `&&ParseNode` to `&ParseNode`.
                                return Some(*node);
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
                            // RGX-0078.5.d.4.i — deref `&&ParseNode` to `&ParseNode`.
                            Some(*node)
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
                    // RGX-0078.5.i.7 (`-0105`): the shaped `->` carrier is now
                    // `Shaped` (arena `PgenValue`); the arm mirrors the retired
                    // `Json` arm byte-exactly (`Str` → the raw text, `Null` →
                    // no scalar, everything else → its compact-JSON rendering
                    // — `to_serde_value().to_string()` = the owned `Value`'s
                    // `Display` bytes). The retired `Json` variant was
                    // referenced nowhere, so its `-0106` lib-side retirement
                    // never touched this artifact.
                    ParseContent::Shaped(value) => match value {
                        PgenValue::Str(text) => Some((*text).to_string()),
                        PgenValue::Null => None,
                        other => Some(other.to_serde_value().to_string()),
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
                    // Compile-affordance wildcard (RGX-0078.5.i.6): lets THIS
                    // generation keep compiling if the engine's ParseContent
                    // gains a variant before this parser is regenerated
                    // (unreachable today).
                    #[allow(unreachable_patterns)]
                    _ => None,
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

            // SV-EXH-PROOF.3.3.4.a.2 (PGEN-SV-EXH-PROOF-0027): shared
            // segment lexer used by both `parse_semantic_reference_segments`
            // (positional path) and `resolve_named_semantic_reference`
            // (named path).
            //
            // Accepts the bare suffix of a reference (no leading `$`, no
            // leading head identifier — the caller has already consumed
            // those). Returns `Vec<&str>` where each element is either a
            // property name like `body` or a bracketed indexed-access
            // form like `[0]` (brackets and digits preserved verbatim so
            // the walker can dispatch via `parse_bracketed_index`).
            //
            // Empty suffix → empty Vec. Malformed forms (`.`, `.123`,
            // `[`, `[abc]`, `[0`) return `None`. Depth is structurally
            // unbounded — the parse loop has no max-iteration cap and
            // the resulting Vec is bounded only by available memory.
            fn lex_semantic_reference_segments_suffix<'a>(
                suffix: &'a str,
            ) -> Option<Vec<&'a str>> {
                let mut segments = Vec::new();
                let mut remaining = suffix.trim();
                while !remaining.is_empty() {
                    if let Some(rest) = remaining.strip_prefix('.') {
                        let bytes = rest.as_bytes();
                        let mut end = 0usize;
                        if bytes.is_empty()
                            || !(bytes[0] == b'_' || (bytes[0] as char).is_ascii_alphabetic())
                        {
                            return None;
                        }
                        end += 1;
                        while end < bytes.len()
                            && (bytes[end] == b'_' || (bytes[end] as char).is_ascii_alphanumeric())
                        {
                            end += 1;
                        }
                        segments.push(&rest[..end]);
                        remaining = &rest[end..];
                    } else if let Some(rest) = remaining.strip_prefix('[') {
                        let bytes = rest.as_bytes();
                        let mut end = 0usize;
                        while end < bytes.len() && (bytes[end] as char).is_ascii_digit() {
                            end += 1;
                        }
                        if end == 0 || end >= bytes.len() || bytes[end] != b']' {
                            return None;
                        }
                        // Capture the FULL `[<digits>]` form, brackets included,
                        // so the walker can dispatch via `parse_bracketed_index`.
                        let bracketed_end = 1 + end + 1; // '[' + digits + ']'
                        segments.push(&remaining[..bracketed_end]);
                        remaining = &remaining[bracketed_end..];
                    } else {
                        return None;
                    }
                }
                Some(segments)
            }

            // SV-EXH-PROOF.3.3.4.a.2 (PGEN-SV-EXH-PROOF-0027): named-path
            // counterpart to `lex_semantic_reference_segments_suffix`.
            // The named-resolver receives a reference WITHOUT the leading
            // `$` (e.g. `name.body[0].sub`), so the head identifier is
            // also part of the input. We lex the head as the first
            // segment, then delegate to the suffix lexer for the rest.
            fn lex_semantic_reference_segments_named<'a>(reference: &'a str) -> Option<Vec<&'a str>> {
                let normalized = reference.trim();
                let bytes = normalized.as_bytes();
                if bytes.is_empty() {
                    return None;
                }
                let mut head_end = 0usize;
                if !(bytes[0] == b'_' || (bytes[0] as char).is_ascii_alphabetic()) {
                    return None;
                }
                head_end += 1;
                while head_end < bytes.len()
                    && (bytes[head_end] == b'_'
                        || (bytes[head_end] as char).is_ascii_alphanumeric())
                {
                    head_end += 1;
                }
                let head = &normalized[..head_end];
                let suffix = &normalized[head_end..];
                let mut segments = Vec::with_capacity(1);
                segments.push(head);
                segments.extend(Self::lex_semantic_reference_segments_suffix(suffix)?);
                Some(segments)
            }

            // SV-EXH-PROOF.3.3.4.a.2 (PGEN-SV-EXH-PROOF-0027): if `segment`
            // is a bracketed-digits form like `[0]` / `[42]`, return the
            // inner integer. Otherwise return `None` (the segment is a
            // property name). Strict policy: requires both brackets AND
            // a non-empty digit run with no overflow.
            fn parse_bracketed_index(segment: &str) -> Option<usize> {
                let inner = segment.strip_prefix('[')?.strip_suffix(']')?;
                if inner.is_empty() || !inner.bytes().all(|b| (b as char).is_ascii_digit()) {
                    return None;
                }
                inner.parse::<usize>().ok()
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
            #consume_layout_for_terminal_fn
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
            /// RGX-0078.5.i.12 — the terminal-literal fast path for constant
            /// literals codegen proved non-empty, all-ASCII, and ≤8 bytes at
            /// emission. `bare_parse ⇒ !logger_enabled ⇒ !trace_enabled`, so
            /// the cold fallback keeps every trace byte-exact through
            /// `match_string` while bench parses never take the branch. For an
            /// all-ASCII literal a successful byte-match proves both `start`
            /// and `end` are char boundaries (an ASCII byte is never a UTF-8
            /// continuation byte), so the success slice cannot panic;
            /// `position ≤ input.len()` and N ≤ 8 make `start + N` overflow-
            /// free, matching `bytes_match_at`'s reachable semantics exactly.
            ///
            /// RGX-0078.5.j.4 G1-A (`PGEN-RGX-0078-0165`) — the success arm
            /// returns the `expected` literal itself, NOT `&self.input[..]`.
            /// The two are byte-identical BY CONSTRUCTION: the single shared
            /// emission point (`terminal_literal_match_call`) derives
            /// `expected_bytes` as `expected.as_bytes()`, so reaching this line
            /// has already proven `input[start..end] == expected.as_bytes()`,
            /// and `&'static str` coerces to `&'input str`.
            ///
            /// ⚠️ This replaced a DOCUMENTED-BUT-FALSE assumption. The prior
            /// form asserted the slice's "internal boundary checks fold under
            /// const propagation" — disassembling the fat-LTO release probe
            /// (`1d3fa0ee`) refuted it: the per-atom fast path really did call
            /// `core::str::…SliceIndex::get` with two `is_char_boundary` checks
            /// and `slice_error_fail` panic scaffolding. The optimizer could
            /// not fold them because a POTENTIAL PANIC is an observable effect
            /// it must preserve — not dead code. Returning the literal removes
            /// the panic path at the source, which is why it is removable at
            /// all. Evidence: `docs/tasks/artifacts/g1_atom_cost/`.
            #[inline(always)]
            fn match_lit_ascii<const N: usize>(
                &mut self,
                expected: &'static str,
                expected_bytes: &[u8; N],
            ) -> ParseResult<&'input str> {
                if self.logger_enabled {
                    return self.match_string(expected);
                }
                if #allow_layout_skip_for_terminals {
                    self.consume_layout_for_terminal(expected);
                }
                let start = self.position;
                let end = start + N;
                if end <= self.input.len()
                    && self.input.as_bytes()[start..end] == *expected_bytes
                {
                    self.position = end;
                    return Ok(expected);
                }
                Err(ParseError::Backtrack { position: start })
            }
            fn match_string(&mut self, expected: &str) -> ParseResult<&'input str> {
                if #allow_layout_skip_for_terminals {
                    self.consume_layout_for_terminal(expected);
                }
                let start = self.position;
                let expected_bytes = expected.as_bytes();
                let end = start + expected_bytes.len();

                if self.trace_enabled() {
                    self.logger.log_debug(#filename, self.position as u32, &format!("🔤 Attempting to match terminal '{}' at position {} (end: {})", expected, start, end));
                }

                if self.bytes_match_at(start, expected_bytes) {
                    if !self.input.is_char_boundary(start) || !self.input.is_char_boundary(end) {
                        return Err(self.create_contextual_error(&format!(
                            "Internal UTF-8 boundary mismatch while matching '{}'",
                            expected
                        )));
                    }
                    self.position = end;

                    if self.trace_enabled() {
                        self.logger.log_success(#filename, self.position as u32, &format!("✅ Terminal '{}' matched, advanced to position {}", expected, end));
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
                if self.trace_enabled() {
                    let found_str = if self.position < self.input.len() {
                        let end = (self.position + expected_bytes.len()).min(self.input.len());
                        self.byte_window_lossy(self.position, end)
                    } else {
                        "<EOF>".to_string()
                    };
                    self.logger.log_error(#filename, self.position as u32, &format!("❌ Terminal '{}' failed at position {} - found '{}'", expected, start, found_str));
                }

                Err(ParseError::Backtrack { position: start })
            }

            #bare_terminal_twins

            #match_regex_helper

            fn try_parse<F, T>(&mut self, f: F) -> Option<T>
            where
                F: FnOnce(&mut Self) -> ParseResult<T>,
            {
                let saved_pos = self.position;
                // RGX-0078.5.j.4 (`-0200`) — TWO independent stack snapshots:
                // a speculation that spans generated BARE id-only frames can
                // leave `rule_id_stack` deeper than `parse_stack`, so each
                // stack must be restored to its OWN saved length. In an
                // all-paired parse the two lengths are equal and this is
                // byte-identical to the previous single-length snapshot.
                let saved_name_stack_len = self.recursion_guard.parse_stack.len();
                let saved_id_stack_len = self.recursion_guard.rule_id_stack.len();
                // GRAMMAR-WELLFORMED.G.4.6 — snapshot the transactional
                // parse-coverage length so a failed speculation's rule-entry
                // pushes are discarded on backtrack (mirrors position/stack/
                // semantic rollback). O(1); when coverage is disabled the stack
                // is empty so this is a trivial 0.
                let saved_coverage_len = self.coverage_stack.len();
                // ============================================================
                // SV-EXH-PROOF.3.3.4.b.6.2.7 — speculative-parse semantic
                // rollback (Bug A from the C3 diagnosis).
                //
                // ROOT CAUSE: `try_parse` is the universal wrapper for every
                // PEG speculation in the generated code — branch alternatives
                // (`generate_or_logic`), optional groups `( X )?`, quantifier
                // iterations `*`/`+`/`{N,M}`, and lookahead `&X`/`!X`. Before
                // this fix, on failure it rolled back ONLY the position and
                // the recursion-guard stack — NOT the semantic runtime
                // state. So when a speculative branch entered a fact-emitting
                // child rule (whose own `with_semantic_runtime_rule_transaction`
                // *committed* the fact on the child's local success), then the
                // outer speculation failed later (e.g. branch 6 of the SV
                // `type_declaration_sv_2017` alternation matches
                // `kw_typedef ( enum|struct|union )? declared_type_identifier`
                // on `typedef TYPE T;`, emits `TYPE=typedef`, then fails on
                // `semi` because the next token is ` T` not `;`), the leaked
                // fact persisted in the parent's committed state. Later
                // `lacks_fact_attribute_equals(declaration_family, typedef)`
                // checks then incorrectly rejected `TYPE` (which actually has
                // family `type_parameter`), corrupting downstream parse paths
                // — the precise blocker behind `.3.3.4.b.6.2.6`'s minimal
                // 7-line repro.
                //
                // FIX (Level 5 per the no-workarounds hierarchy — none of
                // levels 1–4 reach this; the universal semantic store IS
                // transactional, but the engine wasn't *using* a transaction
                // at the speculative-attempt boundary): snapshot the
                // `semantic_runtime_state` at try_parse entry, and on failure
                // `rollback_to` that checkpoint so any emit_fact /
                // open_scope / close_scope side effects of the failed
                // speculation are undone alongside the position. Parser-
                // AGNOSTIC: every parser benefits (the regex / json grammars
                // have no semantic predicates so their snapshot is trivially
                // O(1) — checkpoint captures three usize lengths plus an
                // empty `active_chain` Vec — and rollback is a no-op).
                //
                // Cost on grammars with active annotations:
                //   - On entry: one `SemanticRuntimeCheckpoint` (3 usizes +
                //     `active_chain.clone()` — typically <10 ScopeIds for SV).
                //   - On Ok: nothing (successful speculations keep their
                //     emissions; the per-rule outer transaction handles
                //     ultimate commit/rollback).
                //   - On Err: `rollback_to` truncates the three vecs +
                //     removes the per-kind index entries for the discarded
                //     facts (perf contract §3.7 — O(rolled-back facts)).
                //
                // SCOPE LIMITATION (Bug B, intentionally NOT addressed
                // here): under `longest_match` / `priority_first` branch
                // policy the multi-branch codegen tries ALL branches and
                // selects the best; loser-but-SUCCESSFUL branches' emissions
                // still accumulate in the committed parent state. That's a
                // strictly broader fix (multi-branch commit-only-winner with
                // either delta-replay or winner-double-execute) and is a
                // tracked taxonomy entry (C3-B). It's a fact-count
                // efficiency issue and does NOT corrupt boolean `has_fact`
                // consumers (the SV consumer model), so it does not block
                // the SV-EXH-PROOF campaign.
                // ============================================================
                let saved_semantic_checkpoint =
                    self.semantic_runtime_state.checkpoint();

                if self.trace_enabled() {
                    self.logger.log_debug(#filename, self.position as u32, &format!("🔄 Starting speculative parse at position {}", saved_pos));
                }

                match f(self) {
                    Ok(result) => {
                        if self.trace_enabled() {
                            self.logger.log_success(#filename, self.position as u32, &format!("🔄 Speculative parse succeeded, advanced to position {}", self.position));
                        }
                        Some(result)
                    }
                    Err(e) => {
                        // Backtrack
                        self.position = saved_pos;
                        // GRAMMAR-WELLFORMED.G.4.6 — discard rule-entry pushes
                        // recorded inside this failed speculation, so coverage
                        // reflects only the accepted parse. No-op when disabled.
                        self.coverage_stack.truncate(saved_coverage_len);
                        // SV-EXH-PROOF.3.3.4.b.6.2.36.2 — capture the rule
                        // at parse_stack top BEFORE truncating, so the
                        // rollback trace event identifies the failing
                        // speculation's owning rule. Per
                        // [[feedback_why_and_where_before_solution]].
                        // RGX-0078.5.i.2 (P0): the capture is a free
                        // `&'static str` copy and the label travels as a
                        // deferred `RollbackLabel` — the previous eager
                        // 2×`String` here ran on EVERY failed speculation
                        // and was only consumed under trace (the dominant
                        // part of the measured −14.7% V2 census surface,
                        // 2676 rollbacks on the 8-pattern bench).
                        let try_parse_rule: Option<&'static str> = self
                            .recursion_guard
                            .parse_stack
                            .last()
                            .map(|entry| entry.0);
                        self.recursion_guard
                            .truncate_stacks(saved_name_stack_len, saved_id_stack_len);
                        // .b.6.2.7: also undo semantic side-effects of the
                        // failed speculation (see the block-comment above).
                        self.semantic_runtime_state.rollback_to_labeled(
                            saved_semantic_checkpoint,
                            crate::ast_pipeline::RollbackLabel::TryParseErr(try_parse_rule),
                        );

                        if self.trace_enabled() {
                            self.logger.log_warning(#filename, self.position as u32, &format!("🔙 Speculative parse failed with error '{:?}', backtracked to position {} (rule={})", e, saved_pos, try_parse_rule.unwrap_or("<top-level>")));
                        }

                        None
                    }
                }
            }

            #try_parse_bare_helper

            // PARSE-TERMINATION.6 (WHY+WHERE): opt-in memo footprint report.
            // Gated by env PGEN_REPORT_MEMO_STATS; zero cost otherwise.
            // Subtree-node count is a memory proxy (each MemoEntry clones a
            // full ParseNode subtree). Identifies which rules dominate the
            // packrat memo so a SOUND selective-memoization bound can target
            // them. Parser-agnostic; emitted into every generated parser.
            fn parse_node_size_proxy(node: &ParseNode<'input>) -> usize {
                1 + match &node.content {
                    ParseContent::Terminal(_) => 0,
                    ParseContent::TransformedTerminal(_) => 0,
                    ParseContent::Shaped(_) => 0,
                    // RGX-0078.5.d.4.i — `.copied()` turns the `&&ParseNode` items
                    // into `&ParseNode` for the fn-pointer map.
                    ParseContent::Sequence(items) => items.iter().copied().map(Self::parse_node_size_proxy).sum(),
                    ParseContent::Alternative(inner) => Self::parse_node_size_proxy(inner),
                    ParseContent::Quantified(items, _) => items.iter().copied().map(Self::parse_node_size_proxy).sum(),
                    // Compile-affordance wildcard (RGX-0078.5.i.6): lets THIS
                    // generation keep compiling if the engine's ParseContent
                    // gains a variant before this parser is regenerated
                    // (unreachable today). Without it, ANY additive
                    // ParseContent change forces a two-stage regen bootstrap.
                    #[allow(unreachable_patterns)]
                    _ => 0,
                }
            }

            fn report_memo_stats(&self) {
                // per rule: (success_entries, subtree_node_sum, failure_entries)
                let mut per_rule: std::collections::HashMap<RuleId, (usize, usize, usize)> =
                    std::collections::HashMap::new();
                let mut total_nodes = 0usize;
                for ((rule_id, _pos), entry) in self.memo.iter() {
                    let sz = match &entry.result {
                        Some(node) => Self::parse_node_size_proxy(node),
                        None => 0,
                    };
                    total_nodes += sz;
                    let e = per_rule.entry(*rule_id).or_insert((0, 0, 0));
                    e.0 += 1;
                    e.1 += sz;
                }
                for (rule_id, _pos) in self.memo_fail.iter() {
                    let e = per_rule.entry(*rule_id).or_insert((0, 0, 0));
                    e.2 += 1;
                }
                // MEMO-STORE-SOUNDNESS.2 — epoch-stamped tainted failures are
                // failures too; fold them into the per-rule counts and report
                // the taint split in the header.
                for ((rule_id, _pos), _epoch) in self.memo_fail_tainted.iter() {
                    let e = per_rule.entry(*rule_id).or_insert((0, 0, 0));
                    e.2 += 1;
                }
                let tainted_successes = self
                    .memo
                    .values()
                    .filter(|entry| entry.tainted_at_epoch.is_some())
                    .count();
                let mut rows: Vec<(RuleId, (usize, usize, usize))> = per_rule.into_iter().collect();
                eprintln!(
                    "=== MEMO STATS: {} success entries ({} tainted) + {} cached failures ({} tainted) = {} total, {} subtree-nodes, {} distinct rules ===",
                    self.memo.len(),
                    tainted_successes,
                    self.memo_fail.len() + self.memo_fail_tainted.len(),
                    self.memo_fail_tainted.len(),
                    self.memo.len() + self.memo_fail.len() + self.memo_fail_tainted.len(),
                    total_nodes,
                    rows.len()
                );
                rows.sort_by(|a, b| b.1 .1.cmp(&a.1 .1));
                eprintln!("--- top 30 rules by subtree-node-sum (memory proxy) ---");
                for (rid, (ok, szsum, fail)) in rows.iter().take(30) {
                    let name = Self::RULE_NAMES.get(*rid as usize).copied().unwrap_or("?");
                    eprintln!("  nodes={:>11} ok={:>9} fail={:>9}  {}", szsum, ok, fail, name);
                }
                rows.sort_by(|a, b| (b.1 .0 + b.1 .2).cmp(&(a.1 .0 + a.1 .2)));
                eprintln!("--- top 15 rules by total entry-count (ok+fail) ---");
                for (rid, (ok, szsum, fail)) in rows.iter().take(15) {
                    let name = Self::RULE_NAMES.get(*rid as usize).copied().unwrap_or("?");
                    eprintln!("  total={:>9} ok={:>9} fail={:>9} nodes={:>11}  {}", ok + fail, ok, fail, szsum, name);
                }
            }

            #inlined_frame_call_helper

            fn memoized_call<F>(
                &mut self,
                rule_id: RuleId,
                f: F,
            ) -> ParseResult<(ParseNode<'input>, Option<ParseContent<'input>>)>
            where
                F: FnOnce(&mut Self) -> ParseResult<(ParseNode<'input>, Option<ParseContent<'input>>)>,
            {
                let key = (rule_id, self.position);

                // PARSE-TERMINATION.6 — SPLIT MEMO. Failures (the ~81% majority on
                // real grammars) carry no payload beyond "this rule failed here →
                // backtrack to this position", and that position IS the key, so they
                // live in a lean set with no per-entry value/allocation.
                if self.memo_fail.contains(&key) {
                    if self.trace_enabled() {
                        self.logger.log_warning(#filename, self.position as u32, &format!("💾 Memo hit for rule {} at position {} - cached failure", rule_id, self.position));
                    }
                    // RGX-0078.5.i.4 (P1 STEP-0) — per-rule memo-HIT counter,
                    // recorded only under the transactional-coverage opt-in
                    // (the outcome dump's flag): ordinary parsing pays one
                    // predictable bool check on the hit path, nothing more.
                    if self.coverage_enabled {
                        self.semantic_runtime_state.record_memo_hit(rule_id as usize);
                    }
                    return Err(ParseError::Backtrack {
                        position: key.1,
                    });
                }

                // MEMO-STORE-SOUNDNESS.2 — a STORE-TAINTED cached failure is
                // replayable only while the store write epoch is unchanged
                // since it was recorded (predicates are pure functions of
                // position-determined args + store, so an unchanged epoch
                // reproduces every verdict). Once the store has moved, the
                // entry is evicted and the retry honestly re-parses — the
                // `sem_memo_wrapper` sound anchor.
                if let Some(&epoch) = self.memo_fail_tainted.get(&key) {
                    if epoch == self.semantic_runtime_state.write_epoch() {
                        if self.trace_enabled() {
                            self.logger.log_warning(#filename, self.position as u32, &format!("💾 Memo hit for rule {} at position {} - cached tainted failure (store epoch unchanged)", rule_id, self.position));
                        }
                        // RGX-0078.5.i.4 (P1 STEP-0) — see the fail-set hit above.
                        if self.coverage_enabled {
                            self.semantic_runtime_state.record_memo_hit(rule_id as usize);
                        }
                        return Err(ParseError::Backtrack {
                            position: key.1,
                        });
                    }
                    if self.trace_enabled() {
                        self.logger.log_debug(#filename, self.position as u32, &format!("💾 Evicting STALE tainted failure for rule {} at position {} - store epoch moved", rule_id, self.position));
                    }
                    self.memo_fail_tainted.remove(&key);
                }

                // The success map holds only successful parses (`result` is always
                // `Some`); a cache hit replays the cached node + semantic delta.
                // MEMO-STORE-SOUNDNESS.2 — a STORE-TAINTED success (the body
                // transitively evaluated ≥1 predicate) is replayable only
                // while the store write epoch is unchanged since insert; a
                // stale one is evicted and re-parsed fresh (the
                // `sem_memo_success_*` sound anchors — a stale replay can
                // flip both the verdict and the tree).
                let stale_tainted_success = matches!(
                    self.memo.get(&key),
                    Some(entry) if entry.tainted_at_epoch.is_some_and(|epoch| epoch != self.semantic_runtime_state.write_epoch())
                );
                if stale_tainted_success {
                    if self.trace_enabled() {
                        self.logger.log_debug(#filename, self.position as u32, &format!("💾 Evicting STALE tainted success for rule {} at position {} - store epoch moved", rule_id, self.position));
                    }
                    self.memo.remove(&key);
                }
                if let Some(entry) = self.memo.get(&key) {
                    if let Some(node) = &entry.result {
                        self.position = entry.end_pos;

                        if self.trace_enabled() {
                            self.logger.log_info(#filename, self.position as u32, &format!("💾 Memo hit for rule {} at position {} - reusing cached result", rule_id, self.position));
                        }

                        // SV-EXH-PROOF.3.3.4.b.6.2.36.4 — replay the cached
                        // semantic delta so the rule's `@emit_fact` /
                        // `@open_scope` / `@close_scope` side effects appear
                        // in the current state. Closes the memoization ×
                        // semantic-store composition gap pinned by
                        // `.b.6.2.36.3`.
                        if let Some(delta) = entry.semantic_delta.clone() {
                            if !delta.is_empty() {
                                self.semantic_runtime_state.apply_delta(delta);
                            }
                        }

                        // GRAMMAR-WELLFORMED.H.10.2.2 — replay the cached
                        // coverage delta. The per-rule-entry coverage push only
                        // fires when a rule BODY executes; a memo hit bypasses
                        // the body, so without this replay a subtree first
                        // parsed inside a rolled-back speculation (coverage
                        // truncated by try_parse) and then memo-hit on the
                        // committed path stays absent from the witness record.
                        // The replay happens inside the CURRENT speculation, so
                        // a later rollback still truncates it — the record
                        // stays transactional (sound) while regaining
                        // completeness on cache hits.
                        if self.coverage_enabled {
                            if let Some(coverage) = &entry.coverage_delta {
                                self.coverage_stack.extend_from_slice(coverage);
                            }
                            // RGX-0078.5.i.4 (P1 STEP-0) — see the fail-set hit above.
                            self.semantic_runtime_state.record_memo_hit(rule_id as usize);
                        }

                        return Ok((node.clone(), entry.raw_semantic_content.clone()));
                    }
                }

                if self.trace_enabled() {
                    self.logger.log_debug(#filename, self.position as u32, &format!("💾 Memo miss for rule {} at position {} - computing fresh result", rule_id, self.position));
                }

                // SV-EXH-PROOF.3.3.4.b.6.2.36.4 — capture entry checkpoint
                // BEFORE f(self) so we can extract the body's delta after.
                let memo_entry_checkpoint = self.semantic_runtime_state.checkpoint();
                // GRAMMAR-WELLFORMED.H.10.2.2 — snapshot the coverage-stack
                // length so the body's pushed entries can be stored alongside
                // the result and replayed on every future cache hit.
                let memo_coverage_checkpoint = self.coverage_stack.len();
                // MEMO-STORE-SOUNDNESS.2 — taint-gated memo participation.
                // The memo key is (rule, position) — store-BLIND — so an
                // outcome that depended on the mutable semantic store must not
                // be replayed after the store changes: a stale FAILURE replays
                // a REJECT (`sem_memo_wrapper`), a stale SUCCESS replays body
                // content whose nested tournament choices were made under the
                // OLD store (verdict-flipping AND tree-corrupting —
                // `sem_memo_success_verdict` / `sem_memo_success_ast`). The
                // predicate-evaluation counter is a COMPLETE store-dependence
                // signal (predicates are the store's only read path into
                // parsing) and is monotonic across speculation (truncation
                // rollback never resets counters). Tainted outcomes ARE cached
                // — stamped with the store write epoch and validated on every
                // hit — because outright exclusion collapses packrat
                // protection on predicate-heavy grammars (MEASURED: SV
                // scr1_core_top 1.48 s → 173.6 s, 117×, session #49). A rule's
                // OWN pre/post gates evaluate OUTSIDE its memoized_call, so
                // they never taint its own entry (the transaction re-evaluates
                // them fresh on every hit); inline branch predicates evaluate
                // INSIDE the body and taint it — correct, they steer which
                // branch's content wins.
                let memo_taint_snapshot = self.semantic_runtime_state.predicate_evaluations();
                let result = f(self);
                let memo_store_tainted =
                    self.semantic_runtime_state.predicate_evaluations() != memo_taint_snapshot;

                if let Ok((node, raw_semantic_content)) = &result {
                    let semantic_delta = self
                        .semantic_runtime_state
                        .extract_delta_since(&memo_entry_checkpoint);
                    // GRAMMAR-WELLFORMED.H.10.2.2 — `None` (no allocation) when
                    // coverage recording is off: ordinary parsing pays nothing.
                    let coverage_delta = if self.coverage_enabled {
                        Some(self.coverage_stack[memo_coverage_checkpoint..].to_vec())
                    } else {
                        None
                    };
                    self.memo.insert(
                        key,
                        MemoEntry {
                            result: Some(node.clone()),
                            raw_semantic_content: raw_semantic_content.clone(),
                            end_pos: node.span.end,
                            semantic_delta: Some(semantic_delta),
                            coverage_delta,
                            tainted_at_epoch: if memo_store_tainted {
                                Some(self.semantic_runtime_state.write_epoch())
                            } else {
                                None
                            },
                        },
                    );
                    if self.trace_enabled() {
                        self.logger.log_info(#filename, self.position as u32, &format!("💾 Memoized successful result for rule {} at position {}", rule_id, self.position));
                    }
                } else if memo_store_tainted {
                    // MEMO-STORE-SOUNDNESS.2 — a store-tainted failure is
                    // cached with its epoch stamp; valid only while the store
                    // is unchanged.
                    self.memo_fail_tainted
                        .insert(key, self.semantic_runtime_state.write_epoch());
                    if self.trace_enabled() {
                        self.logger.log_warning(#filename, self.position as u32, &format!("💾 Memoized TAINTED failed result for rule {} at position {} (epoch-stamped)", rule_id, self.position));
                    }
                } else {
                    // PARSE-TERMINATION.6 — record the failure in the lean set
                    // (no value: the backtrack position is the key itself).
                    self.memo_fail.insert(key);
                    if self.trace_enabled() {
                        self.logger.log_warning(#filename, self.position as u32, &format!("💾 Memoized failed result for rule {} at position {}", rule_id, self.position));
                    }
                }

                result
            }

            fn create_contextual_error(&self, message: &str) -> ParseError {
                let position = self.position;

                // Gather rule stack (rule names are &'static str — pointer copy,
                // no allocation per entry). RGX-0078.5.j.4 (`-0200`): read the
                // COMPLETE `rule_id_stack` mapped through RULE_NAMES rather than
                // `parse_stack` — during a bare parse only the ID stack carries
                // every live frame (bare fused frames are id-only), and in a
                // paired parse the bijection makes the two reads identical.
                let rule_stack: Vec<&'static str> = self.recursion_guard.rule_id_stack.iter()
                    .filter_map(|(rid, _)| Self::RULE_NAMES.get(*rid as usize).copied())
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

        // CODEGEN-DETERMINISM.1: emit the semantic-directive / fact-kind inserts in a
        // deterministic key-sorted order. The backing collections (`directives_by_rule`,
        // `branch_directives_by_rule`, `fact_kinds`) are `HashMap`s with process-random
        // iteration order, so an unsorted emission makes the generated parser SOURCE
        // byte-non-reproducible (same input → different bytes across regens). The
        // generated parser re-inserts these into its own order-insensitive runtime map,
        // so sorting the emission is behavior-preserving — it only canonicalizes the
        // emitted source order.
        let mut rule_pairs: Vec<_> = compiled.iter().collect();
        rule_pairs.sort_by(|a, b| a.0.cmp(b.0));
        let rule_entries = rule_pairs.into_iter().map(|(rule_name, directives)| {
            let directive_tokens = directives
                .iter()
                .map(Self::generate_semantic_runtime_directive_tokens);
            quote! {
                directives_by_rule.insert(#rule_name.to_string(), vec![#(#directive_tokens),*]);
            }
        });
        let mut branch_pairs: Vec<_> = compiled.branch_iter().collect();
        branch_pairs.sort_by(|a, b| a.0.cmp(b.0));
        let branch_rule_entries = branch_pairs
            .into_iter()
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

        // `SV-EXH-PROOF.3.3.4.b.6.1.1`: emit the `@fact_kind:` registry into the
        // generated parser. `from_parts` seeds an empty registry; without these
        // entries + the `set_fact_kinds` call the generated parser's
        // `fact_kinds` is always empty (the registry was only ever populated on
        // the compile-time `compile()` path before `.b.6.1`).
        let mut fact_kind_pairs: Vec<_> = compiled.fact_kinds().collect();
        fact_kind_pairs.sort_by(|a, b| a.0.cmp(b.0));
        let fact_kind_entries = fact_kind_pairs.into_iter().map(|(name, decl)| {
            let decl_tokens = Self::generate_fact_kind_decl_tokens(decl);
            quote! {
                fact_kinds.insert(#name.to_string(), #decl_tokens);
            }
        });

        Ok(quote! {
            {
                let mut directives_by_rule = std::collections::HashMap::new();
                let mut branch_directives_by_rule = std::collections::HashMap::new();
                let mut fact_kinds = std::collections::HashMap::new();
                #(#rule_entries)*
                #(#branch_rule_entries)*
                #(#fact_kind_entries)*
                let mut compiled =
                    crate::ast_pipeline::CompiledSemanticRuntimeAnnotations::from_parts(
                        directives_by_rule,
                        branch_directives_by_rule,
                    );
                compiled.set_fact_kinds(fact_kinds);
                compiled
            }
        })
    }

    /// `SV-EXH-PROOF.3.3.4.b.6.1.1`: reconstruct a `FactKindDecl` struct literal
    /// for the generated parser's `@fact_kind:` registry. All eight fields are
    /// serialised verbatim from the compile-time-validated declaration.
    fn generate_fact_kind_decl_tokens(decl: &crate::ast_pipeline::FactKindDecl) -> TokenStream {
        let name = decl.name.as_str();
        let attributes = decl.attributes.iter().map(|a| a.as_str());
        let required = decl.required.iter().map(|r| r.as_str());
        let indexes = decl.indexes.iter().map(|tuple| {
            let items = tuple.iter().map(|s| s.as_str());
            quote! { vec![#(#items.to_string()),*] }
        });
        let optional_string = |value: &Option<String>| match value {
            Some(text) => {
                let text = text.as_str();
                quote! { Some(#text.to_string()) }
            }
            None => quote! { None },
        };
        let scope_kind = optional_string(&decl.scope_kind);
        let exportable = decl.exportable;
        let artefact_kind = optional_string(&decl.artefact_kind);
        let description = optional_string(&decl.description);
        quote! {
            crate::ast_pipeline::FactKindDecl {
                name: #name.to_string(),
                attributes: vec![#(#attributes.to_string()),*],
                required: vec![#(#required.to_string()),*],
                indexes: vec![#(#indexes),*],
                scope_kind: #scope_kind,
                exportable: #exportable,
                artefact_kind: #artefact_kind,
                description: #description,
            }
        }
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
            // `SV-EXH-PROOF.3.3.4.a` MVP-0: emit parser-side construction tokens
            // for the new parser-agnostic library directives. The compiled
            // parser will register these in `directives_by_rule` and the
            // generator-emitted `with_semantic_runtime_rule_transaction` will
            // dispatch on them (export at successful commit; import at entry).
            SemanticRuntimeDirective::ExportToLibrary(spec) => {
                let kind = spec.kind.as_str();
                let name = Self::generate_semantic_runtime_value_tokens(&spec.name);
                quote! {
                    crate::ast_pipeline::SemanticRuntimeDirective::ExportToLibrary(
                        crate::ast_pipeline::SemanticLibraryExportSpec {
                            kind: #kind.to_string(),
                            name: #name,
                        }
                    )
                }
            }
            SemanticRuntimeDirective::ImportFromLibrary(spec) => {
                let kind = spec.kind.as_str();
                let name = Self::generate_semantic_runtime_value_tokens(&spec.name);
                quote! {
                    crate::ast_pipeline::SemanticRuntimeDirective::ImportFromLibrary(
                        crate::ast_pipeline::SemanticLibraryImportSpec {
                            kind: #kind.to_string(),
                            name: #name,
                        }
                    )
                }
            }
            // `SV-EXH-PROOF.3.3.4.b.5.1.2` + `.b.6.1.1`: `@fact_kind:` is a
            // declaration, not a per-rule runtime action. The real declaration
            // data IS emitted into the generated parser's
            // `CompiledSemanticRuntimeAnnotations.fact_kinds` registry by
            // `generate_compiled_semantic_runtime_annotations_tokens` (the
            // `fact_kind_entries` + `set_fact_kinds` call). This per-rule
            // directive stays a trivial no-op marker so the per-rule directives
            // Vec preserves its element count and the runtime `apply_directive`
            // short-circuits cleanly.
            SemanticRuntimeDirective::DeclareFactKind(_) => {
                quote! {
                    crate::ast_pipeline::SemanticRuntimeDirective::DeclareFactKind(
                        crate::ast_pipeline::FactKindDecl::default()
                    )
                }
            }
            // `SV-EXH-PROOF.3.3.4.b.5.1.5`: `@predicate_def:` is a declaration,
            // not a per-rule runtime action; this per-rule directive is a no-op
            // marker (element-count preservation).
            //
            // KNOWN GAP (owned by `.b.6.2`): unlike `fact_kinds` above, the
            // `predicate_defs` registry is NOT yet emitted into the generated
            // parser — `generate_compiled_semantic_runtime_annotations_tokens`
            // does not serialise it (the `PredicateDef.body` is a recursive
            // `PredicateExpr` AST that needs its own token serialiser). So a
            // generated parser's `predicate_defs` is currently always empty;
            // `.b.5.1.5.c`'s composed-predicate dispatch only works on the
            // direct-construction path exercised by unit tests. `.b.6.2` (which
            // introduces `@predicate_def:` in a real grammar) must close this.
            SemanticRuntimeDirective::DefinePredicate(_) => {
                quote! {
                    crate::ast_pipeline::SemanticRuntimeDirective::DefinePredicate(
                        crate::ast_pipeline::PredicateDef::default()
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
            // FINAL-PHASE-PREDICATE.2: round-trip the whole-input phase into the
            // generated parser so it reconstructs a `phase: final` predicate.
            crate::ast_pipeline::SemanticPredicatePhase::Final => {
                quote! { crate::ast_pipeline::SemanticPredicatePhase::Final }
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
        // RGX-0078.5.i.7 (D2-A): the derivation moved to the shared registry function so the
        // fusibility census's cascade gate reads the exact resolution codegen emits from
        // (the `effective_rule_branch_policy` shared-delegate precedent).
        crate::ast_pipeline::semantic_directive_registry::effective_rule_bool_directive(
            self.annotations.as_ref(),
            rule_name,
            names,
        )
    }

    fn semantic_directive_parts(annotation: &SemanticAnnotation) -> Option<(String, String)> {
        // GRAMMAR-WELLFORMED.A2.3: the derivation moved to the shared registry function so the
        // linter's policy-conditioned verdicts read the exact resolution codegen uses.
        crate::ast_pipeline::semantic_directive_registry::semantic_directive_name_payload(
            annotation,
        )
    }

    /// `RAWCAP-TRANSFORM-PATH.2`: codegen-time gate — true iff `rule_name` carries a
    /// POSITIONAL (`$N`) raw-view POST predicate. Reuses the exact compiled
    /// classification (`compile_semantic_runtime_annotations` +
    /// `needs_positional_raw_post_capture_for_rule`) rather than re-parsing the raw
    /// payload, so it can never drift from the runtime resolver's positional
    /// dispatch. Decides whether the non-`Or` transform path must capture the raw
    /// body content before the `->` transform shadows it — emitted for such a rule
    /// ONLY, so a rule without one (every shipped rule today) is byte-identical to
    /// the pre-fix codegen. The fast path skips the compile for a rule that carries
    /// no directives at all (the overwhelming majority), so this is O(rules with
    /// annotations), not O(all rules).
    fn rule_needs_positional_raw_post_capture(&self, rule_name: &str) -> bool {
        let Some(annotations) = &self.annotations else {
            return false;
        };
        if annotations
            .semantic_annotations
            .get(rule_name)
            .is_none_or(|entries| entries.is_empty())
        {
            return false;
        }
        match compile_semantic_runtime_annotations(annotations) {
            Ok(compiled) => compiled.needs_positional_raw_post_capture_for_rule(rule_name),
            Err(_) => false,
        }
    }

    fn rule_branch_policy(&self, rule_name: &str) -> SemanticBranchPolicy {
        // GRAMMAR-WELLFORMED.A2.3: shared with the linter (see `effective_rule_branch_policy`).
        crate::ast_pipeline::semantic_directive_registry::effective_rule_branch_policy(
            self.annotations.as_ref(),
            rule_name,
        )
    }

    fn rule_coverage_target_policy(&self, rule_name: &str) -> SemanticCoverageTargetPolicy {
        let Some(annotations) = &self.annotations else {
            return SemanticCoverageTargetPolicy::default();
        };
        let Some(entries) = annotations.semantic_annotations.get(rule_name) else {
            return SemanticCoverageTargetPolicy::default();
        };

        let mut policy = SemanticCoverageTargetPolicy::default();
        // RGX-0078.5.i.7 (D2-A): the weight resolution moved to the shared registry function
        // so the fusibility census's cascade gate reads the exact resolution codegen emits
        // `record_coverage_target_event` from.
        policy.coverage_target_weight =
            crate::ast_pipeline::semantic_directive_registry::effective_rule_coverage_target_weight(
                self.annotations.as_ref(),
                rule_name,
            );
        for annotation in entries {
            let Some((name, payload)) = Self::semantic_directive_parts(annotation) else {
                continue;
            };
            if name == "critical_path" {
                if let Some(enabled) = parse_semantic_bool(&payload) {
                    policy.critical_path = enabled;
                }
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
        // RGX-0078.5.i.7 (D2-A): the enable resolution moved to the shared registry function
        // so the fusibility census's cascade gate reads the exact resolution codegen emits
        // `record_negative_case_failure` from.
        policy.invalid_case =
            crate::ast_pipeline::semantic_directive_registry::effective_rule_negative_case_enabled(
                self.annotations.as_ref(),
                rule_name,
            );
        for annotation in entries {
            let Some((name, payload)) = Self::semantic_directive_parts(annotation) else {
                continue;
            };
            if name == "negative" {
                if let Some(enabled) = parse_semantic_bool(&payload) {
                    policy.negative = enabled;
                }
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
        // GRAMMAR-WELLFORMED.A2.4: shared with the linter (see
        // `effective_rule_deterministic_partition_policy` — partition rotation reorders the
        // branch tournament, so deadness verdicts are conditioned on the same resolution).
        crate::ast_pipeline::semantic_directive_registry::effective_rule_deterministic_partition_policy(
            self.annotations.as_ref(),
            rule_name,
        )
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

        // RGX-0078.5.i.7 (D2-A): the enable resolution moved to the shared registry function
        // so the fusibility census's cascade gate reads the exact resolution codegen routes
        // the tournament failure path through `recover_with_hints` from.
        let recover_enabled =
            crate::ast_pipeline::semantic_directive_registry::effective_rule_recovery_enabled(
                self.annotations.as_ref(),
                rule_name,
            );
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
        // GRAMMAR-WELLFORMED.A2.4: shared with the linter (see `effective_rule_associativity` —
        // the tie-break selects the LATER branch under `right`, so deadness verdicts must read
        // the same resolution).
        crate::ast_pipeline::semantic_directive_registry::effective_rule_associativity(
            self.annotations.as_ref(),
            rule_name,
        )
    }

    fn rule_branch_priorities(&self, rule_name: &str, branch_count: usize) -> Vec<i64> {
        // GRAMMAR-WELLFORMED.A2.4: shared with the linter (see
        // `effective_rule_branch_priorities` — priority is compared before the associativity
        // tie-break, so deadness verdicts must read the same resolution).
        crate::ast_pipeline::semantic_directive_registry::effective_rule_branch_priorities(
            self.annotations.as_ref(),
            rule_name,
            branch_count,
        )
    }

    fn rule_value_constraints(&self, rule_name: &str) -> SemanticValueConstraints {
        // STIMULI-SIGNOFF.13.2: the extraction moved to the shared registry function so the
        // parse-harness interpreter's value-constraint guard mirror reads the exact constraint
        // resolution these emitted guards are compiled from.
        crate::ast_pipeline::semantic_directive_registry::effective_rule_value_constraints(
            self.annotations.as_ref(),
            rule_name,
        )
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

    /// RGX-0078.5.i.7 D0 — clone `node` with every regex atom's pattern replaced by
    /// its `effective_regex_pattern` under `rule_name`'s steering policy. Feeds the
    /// `first_set_grammar_tree` snapshot so the FIRST analysis and the emitted
    /// `match_regex` calls can never diverge on the pattern text (see the snapshot
    /// choke point in `generate_parser_tokens`).
    fn rewrite_regex_atoms_to_effective_patterns(
        &self,
        rule_name: &str,
        node: &ASTNode,
    ) -> ASTNode {
        match node {
            ASTNode::Or { alternatives } => ASTNode::Or {
                alternatives: alternatives
                    .iter()
                    .map(|alt| self.rewrite_regex_atoms_to_effective_patterns(rule_name, alt))
                    .collect(),
            },
            ASTNode::Sequence { elements } => ASTNode::Sequence {
                elements: elements
                    .iter()
                    .map(|el| self.rewrite_regex_atoms_to_effective_patterns(rule_name, el))
                    .collect(),
            },
            ASTNode::Quantified {
                element,
                quantifier,
            } => ASTNode::Quantified {
                element: Box::new(
                    self.rewrite_regex_atoms_to_effective_patterns(rule_name, element),
                ),
                quantifier: quantifier.clone(),
            },
            ASTNode::Lookahead { element, positive } => ASTNode::Lookahead {
                element: Box::new(
                    self.rewrite_regex_atoms_to_effective_patterns(rule_name, element),
                ),
                positive: *positive,
            },
            ASTNode::Atom { value } => match value {
                ASTValue::Node(inner) => ASTNode::Atom {
                    value: ASTValue::Node(Box::new(
                        self.rewrite_regex_atoms_to_effective_patterns(rule_name, inner),
                    )),
                },
                ASTValue::Token(parts) => {
                    if parts.len() >= 2 {
                        let (TokenValue::String(token_type), TokenValue::String(token_value)) =
                            (&parts[0], &parts[1]);
                        if token_type == "regex" {
                            let effective =
                                self.effective_regex_pattern(rule_name, token_value);
                            if effective != *token_value {
                                let mut rewritten = parts.clone();
                                rewritten[1] = TokenValue::String(effective);
                                return ASTNode::Atom {
                                    value: ASTValue::Token(rewritten),
                                };
                            }
                        }
                    }
                    node.clone()
                }
            },
        }
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

        // P4-i (RGX-0078.5.i.6): a `@constraint` whose expression is provably
        // constant-true — and whose policy consults nothing else (no
        // `@requires` references, no `@implies`) — compiles to a rule-exit
        // check that can never fail and has no observable surface (no trace
        // line, no store effect, an error branch that is dead by
        // construction; `enforce_relational_requires` with an empty slice is
        // a pure no-op). Fold it at generation time instead of re-parsing
        // the constant string on every rule exit at runtime.
        if policy.requires_references.is_empty()
            && policy.implication.is_none()
            && relational_constraint_is_provably_truthy(&constraint_expression)
        {
            return quote! {};
        }

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
                let node_arena = crate::ast_pipeline::NodeArena::new();
                let mut parser = #parser_name::new(input, &node_arena, logger);
                let _ = parser.parse();
            }
        }
    }
}

/// P4-i (RGX-0078.5.i.6): decide at GENERATION time whether a `@constraint`
/// expression is provably constant-true under the emitted relational
/// evaluator's own decision procedure, so the whole rule-exit guard can be
/// elided as dead code.
///
/// The gate is deliberately STRICTLY NARROWER than the emitted evaluator.
/// Each banned character maps to one evaluator feature the expression must
/// not be able to reach: `|`/`&` (top-level `||`/`&&` splits), `!`
/// (negation), `<`/`>`/`=` (the six comparison operators), `$`
/// (capture-reference syntax), `"`/`'` (quote handling in both the
/// splitter's quote-state tracking and `semantic_unquote`), `(`/`)`
/// (full-parenthesis stripping and split depth tracking). What remains must
/// also NOT be a bare dotted-identifier chain — `semantic_reference_syntax`
/// accepts `name` / `a.b.c` WITHOUT a `$` sigil, and such a reference
/// resolves against parse content (it can even reject the rule when
/// unresolved), so it is never a constant. It must NOT parse as `f64`
/// (numeric truthiness: `0`/`inf`/`nan`-class values decide differently)
/// and must not be one of the boolean/falsy words the evaluator special-
/// cases (`true`/`false` and `semantic_truthy`'s falsy set) — a deliberate
/// boolean or a (defective) constant-false constraint keeps its exact
/// runtime behavior. Every surviving expression provably runs the emitted
/// evaluator into `semantic_truthy(non-empty prose) == true` with zero side
/// effects.
fn relational_constraint_is_provably_truthy(expression: &str) -> bool {
    let normalized = expression.trim();
    if normalized.is_empty() {
        return false;
    }
    if normalized.chars().any(|c| {
        matches!(
            c,
            '|' | '&' | '!' | '<' | '>' | '=' | '$' | '"' | '\'' | '(' | ')'
        )
    }) {
        return false;
    }
    let bare_identifier_chain = normalized.split('.').all(|segment| {
        let bytes = segment.as_bytes();
        match bytes.first() {
            Some(&first) if first == b'_' || first.is_ascii_alphabetic() => bytes[1..]
                .iter()
                .all(|byte| *byte == b'_' || byte.is_ascii_alphanumeric()),
            _ => false,
        }
    });
    if bare_identifier_chain {
        return false;
    }
    if normalized.parse::<f64>().is_ok() {
        return false;
    }
    let lowered = normalized.to_ascii_lowercase();
    !matches!(
        lowered.as_str(),
        "true" | "false" | "0" | "no" | "off" | "none" | "null"
    )
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
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
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
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
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
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
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

    // `STIMULI-SIGNOFF.12`: the generation-side-only `@quantified_separator`
    // directive must NOT push its rule onto the full
    // `with_semantic_runtime_rule_transaction` path — it compiles to ZERO
    // runtime directives, so the annotated rule keeps the fast-path emission
    // (the same emit-neutrality contract as `@whitespace_sensitive` /
    // `@default_profile` / `@profile_alias`). Pinned here because the svpp
    // regeneration surfaced exactly this leak: pp_item's body flipped onto
    // the transaction wrapper before the `rule_has_no_semantic_annotations`
    // exclusion was added.
    #[test]
    fn quantified_separator_directive_keeps_the_annotated_rule_on_the_fast_path() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "item".to_string(),
            vec![structured_named_annotation(
                "quantified_separator",
                r#""\n""#,
                UnifiedSemanticValue::String("\n".to_string()),
            )],
        );
        let generator = AstBasedGenerator {
            grammar_name: "separator_fast_path_test".to_string(),
            entry_rule: None,
            logger: None,
            annotations: Some(annotations),
            branch_return_annotations: HashMap::new(),
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
        };
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "file".to_string(),
            ASTNode::Quantified {
                element: Box::new(token("rule_reference", "item")),
                quantifier: "*".to_string(),
            },
        );
        grammar_tree.insert("item".to_string(), token("quoted_string", "x"));
        let rule_order = vec!["file".to_string(), "item".to_string()];
        let rendered = generator
            .generate_parser(&grammar_tree, &rule_order, "separator_fast_path.rs")
            .expect("parser generation should succeed");
        assert!(
            !rendered.contains(r#"with_semantic_runtime_rule_transaction(Self::RULE_ITEM"#),
            "a rule annotated ONLY with @quantified_separator must keep the fast-path emission"
        );
        assert!(
            rendered.contains(r#"push_rule_context_static("item")"#),
            "the fast-path body still pushes the rule context for trace parity"
        );
    }

    // `STIMULI-SIGNOFF.13.4`: the generation-side-only `@gen_emit_fact` /
    // `@gen_predicate` directives must NOT push their rule onto the full
    // `with_semantic_runtime_rule_transaction` path — each compiles to ZERO
    // runtime directives (StimuliSteering; consumed only by the stimuli
    // generator's `compute_store_aware_gen_directives`), so the annotated
    // rule keeps the fast-path emission (the same emit-neutrality contract
    // as `@quantified_separator`).
    #[test]
    fn gen_store_directives_keep_the_annotated_rule_on_the_fast_path() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "item".to_string(),
            vec![structured_named_annotation(
                "gen_emit_fact",
                "{ kind: test_name, name: $item }",
                UnifiedSemanticValue::Object(vec![
                    UnifiedSemanticProperty {
                        key: "kind".to_string(),
                        value: UnifiedSemanticValue::Identifier("test_name".to_string()),
                    },
                    UnifiedSemanticProperty {
                        key: "name".to_string(),
                        value: UnifiedSemanticValue::RuleReference("item".to_string()),
                    },
                ]),
            )],
        );
        annotations.semantic_annotations.insert(
            "use_site".to_string(),
            vec![structured_named_annotation(
                "gen_predicate",
                "{ name: has_fact, args: [test_name, $text], phase: post }",
                UnifiedSemanticValue::Object(vec![
                    UnifiedSemanticProperty {
                        key: "name".to_string(),
                        value: UnifiedSemanticValue::Identifier("has_fact".to_string()),
                    },
                    UnifiedSemanticProperty {
                        key: "args".to_string(),
                        value: UnifiedSemanticValue::Array(vec![
                            UnifiedSemanticValue::Identifier("test_name".to_string()),
                            UnifiedSemanticValue::RuleReference("text".to_string()),
                        ]),
                    },
                    UnifiedSemanticProperty {
                        key: "phase".to_string(),
                        value: UnifiedSemanticValue::Identifier("post".to_string()),
                    },
                ]),
            )],
        );
        let generator = AstBasedGenerator {
            grammar_name: "gen_store_fast_path_test".to_string(),
            entry_rule: None,
            logger: None,
            annotations: Some(annotations),
            branch_return_annotations: HashMap::new(),
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
        };
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "file".to_string(),
            ASTNode::Sequence {
                elements: vec![
                    token("rule_reference", "item"),
                    token("rule_reference", "use_site"),
                ],
            },
        );
        grammar_tree.insert("item".to_string(), token("quoted_string", "x"));
        grammar_tree.insert("use_site".to_string(), token("quoted_string", "y"));
        let rule_order = vec![
            "file".to_string(),
            "item".to_string(),
            "use_site".to_string(),
        ];
        let rendered = generator
            .generate_parser(&grammar_tree, &rule_order, "gen_store_fast_path.rs")
            .expect("parser generation should succeed");
        for rule in ["item", "use_site"] {
            assert!(
                !rendered.contains(&format!(r#"with_semantic_runtime_rule_transaction(Self::RULE_{}"#, rule.to_uppercase())),
                "a rule annotated ONLY with @gen_emit_fact/@gen_predicate must keep the fast-path emission ({rule})"
            );
        }
        assert!(
            !rendered.contains("gen_emit_fact") && !rendered.contains("gen_predicate"),
            "the generation-side directives must never serialize into the parser artifact"
        );
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
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
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
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
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

    // INLINE-ACTIONS.2: a generator whose 2-branch rule carries a branch-START
    // inline `@emit_fact` on its SECOND branch (mirrors the SV
    // `package_import_item` wildcard branch). Used to prove the codegen wires the
    // winning-branch effect application, and that the gate fires for an *effect*
    // (not a *predicate*) branch annotation.
    fn branch_emit_generator() -> AstBasedGenerator {
        let mut annotations = Annotations::default();
        annotations.branch_semantic_annotations.insert(
            "import_item".to_string(),
            vec![
                Vec::new(),
                vec![structured_named_annotation(
                    "emit_fact",
                    "{ kind: wildcard_open, name: $1 }",
                    UnifiedSemanticValue::Object(vec![
                        UnifiedSemanticProperty {
                            key: "kind".to_string(),
                            value: UnifiedSemanticValue::Identifier("wildcard_open".to_string()),
                        },
                        UnifiedSemanticProperty {
                            key: "name".to_string(),
                            value: UnifiedSemanticValue::RuleReference("$1".to_string()),
                        },
                    ]),
                )],
            ],
        );

        AstBasedGenerator {
            grammar_name: "branch_emit_test".to_string(),
            entry_rule: None,
            logger: None,
            annotations: Some(annotations),
            branch_return_annotations: HashMap::new(),
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
        }
    }

    fn branch_emit_rendered_parser() -> &'static str {
        static RENDERED: OnceLock<String> = OnceLock::new();
        RENDERED
            .get_or_init(|| {
                let generator = branch_emit_generator();
                let mut grammar_tree = HashMap::new();
                grammar_tree.insert(
                    "import_item".to_string(),
                    ASTNode::Or {
                        alternatives: vec![
                            ASTNode::Sequence {
                                elements: vec![
                                    token("quoted_string", "import"),
                                    token("quoted_string", "pkg"),
                                ],
                            },
                            token("quoted_string", "*"),
                        ],
                    },
                );
                let rule_order = vec!["import_item".to_string()];
                generator
                    .generate_parser(
                        &grammar_tree,
                        &rule_order,
                        "semantic_branch_emit_usage.rs",
                    )
                    .expect("parser generation should succeed")
            })
            .as_str()
    }

    #[test]
    fn generated_parser_wires_branch_start_effect_application_for_winning_branch() {
        // INLINE-ACTIONS.2: a branch-start `@emit_fact` must produce (a) the
        // conditionally-emitted apply helper and (b) the winning-branch call loop
        // that resolves + applies it against the selected branch's content.
        let rendered = branch_emit_rendered_parser();
        assert!(
            rendered.contains("fn apply_branch_start_effect_directive"),
            "a grammar with a branch-start @emit_fact should emit the apply helper, got: {}",
            rendered
        );
        assert!(
            rendered.contains("branch_effect_directives_for_rule_branch_id"),
            "the winning-branch block should look up branch-start effect directives (id-indexed, K3c), got: {}",
            rendered
        );
        assert!(
            rendered.contains("for branch_start_effect in &branch_start_effects"),
            "the winning-branch block should iterate + apply each branch-start effect directive, got: {}",
            rendered
        );
    }

    #[test]
    fn generated_parser_omits_branch_start_effect_wiring_without_branch_effects() {
        // INLINE-ACTIONS.2 (gating / zero-blast-radius): a grammar whose branch
        // annotation is a PREDICATE (steering), not an effect, must NOT get the
        // branch-start effect helper or call loop — so non-feature grammars stay
        // byte-identical.
        let rendered = branch_predicate_rendered_parser();
        assert!(
            !rendered.contains("apply_branch_start_effect_directive"),
            "a branch-predicate-only grammar must not emit branch-start effect wiring, got: {}",
            rendered
        );
        assert!(
            !rendered.contains("branch_effect_directives_for_rule_branch"),
            "a branch-predicate-only grammar must not look up branch-start effects, got: {}",
            rendered
        );
    }

    /// RGX-0078.5.i.7 Q-GUARD test scaffolding: a minimal generator with an
    /// optional grammar-level `@whitespace_sensitive: true` (the R2 gate).
    fn quant_guard_generator(annotations: Option<Annotations>) -> AstBasedGenerator {
        AstBasedGenerator {
            grammar_name: "quant_guard_test".to_string(),
            entry_rule: None,
            logger: None,
            annotations,
            branch_return_annotations: HashMap::new(),
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
        }
    }

    fn quant_guard_ws_annotations() -> Annotations {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "top".to_string(),
            vec![crate::ast_pipeline::SemanticAnnotation::Named {
                name: "whitespace_sensitive".to_string(),
                ast: crate::ast_pipeline::UnifiedSemanticAST::Structured {
                    canonical: String::new(),
                    value: crate::ast_pipeline::UnifiedSemanticValue::Boolean(true),
                },
            }],
        );
        annotations
    }

    fn nospace(rendered: &str) -> String {
        rendered.chars().filter(|c| !c.is_whitespace()).collect()
    }

    /// RGX-0078.5.i.7 Q-GUARD — a min-0 `*` site over a BARE rule reference in a
    /// terminal-ws-sensitive grammar gets the loop guard (peek at
    /// `parser.position`, NOT the Or-guards' `parse_start`) plus the EXACT
    /// furthest emulation.
    #[test]
    fn quantified_loop_emits_first_guard_with_furthest_emulation_for_bare_ref() {
        let generator = quant_guard_generator(Some(quant_guard_ws_annotations()));
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "top".to_string(),
            ASTNode::Sequence {
                elements: vec![
                    ASTNode::Quantified {
                        element: Box::new(token("rule_reference", "zref")),
                        quantifier: "*".to_string(),
                    },
                    token("quoted_string", "z"),
                ],
            },
        );
        grammar_tree.insert("zref".to_string(), token("quoted_string", "k"));
        let rule_order = vec!["top".to_string(), "zref".to_string()];
        let rendered = generator
            .generate_parser(&grammar_tree, &rule_order, "quant_guard_test.rs")
            .expect("parser generation should succeed");
        let flat = nospace(&rendered);
        assert!(
            flat.contains("!matches!(parser.input.as_bytes()[parser.position],107u8)"),
            "the min-0 loop should peek the next byte against FIRST(zref) = {{'k'}}, got: {rendered}"
        );
        assert!(
            flat.contains("parser.furthest_position=parser.position"),
            "a bare-ref site must emit the exact furthest emulation, got: {rendered}"
        );
    }

    /// RGX-0078.5.i.7 Q-GUARD — an optional (`?`) element mid-sequence takes the
    /// fast-path emission site; a NoRefs element (pure terminal) gets the guard
    /// WITHOUT the furthest emulation (no furthest writer exists to emulate).
    #[test]
    fn optional_fast_path_emits_guard_without_emulation_for_no_refs_element() {
        let generator = quant_guard_generator(Some(quant_guard_ws_annotations()));
        // RGX-0078.5.i.7 (D2-A): pin the PROTOCOL-graph Q-guard surface in
        // isolation — every fused `cascade_<rule>` fn legitimately carries the
        // rule-entry furthest max-update at its head, which this test's
        // whole-parser negative assertion would otherwise trip on. Disabling the
        // cascade plan reproduces the plan-inactive emission this test pins.
        let _ = generator.cascade_emission_plan.set(None);
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "top".to_string(),
            ASTNode::Sequence {
                elements: vec![
                    ASTNode::Quantified {
                        element: Box::new(token("quoted_string", "q")),
                        quantifier: "?".to_string(),
                    },
                    token("quoted_string", "z"),
                ],
            },
        );
        let rule_order = vec!["top".to_string()];
        let rendered = generator
            .generate_parser(&grammar_tree, &rule_order, "quant_guard_test.rs")
            .expect("parser generation should succeed");
        let flat = nospace(&rendered);
        assert!(
            flat.contains("matches!(parser.input.as_bytes()[parser.position],113u8)"),
            "the optional fast path should peek the next byte against FIRST('q'), got: {rendered}"
        );
        assert!(
            !flat.contains("parser.furthest_position=parser.position"),
            "a NoRefs site must NOT emit the furthest emulation, got: {rendered}"
        );
    }

    /// RGX-0078.5.i.7 Q-GUARD — without `@whitespace_sensitive` terminals (the R2
    /// raw-byte-peek gate) NO quantified-site guard is emitted: byte-identical
    /// codegen for layout-skipping grammars.
    #[test]
    fn quantified_guard_not_emitted_under_layout_skipping_terminals() {
        let generator = quant_guard_generator(None);
        // RGX-0078.5.i.7 (D2-A): see the fast-path test above — this test's
        // whole-parser negative assertions pin the PROTOCOL Q-guard surface;
        // the fused fns' rule-entry furthest heads are out of its scope.
        let _ = generator.cascade_emission_plan.set(None);
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "top".to_string(),
            ASTNode::Sequence {
                elements: vec![
                    ASTNode::Quantified {
                        element: Box::new(token("rule_reference", "zref")),
                        quantifier: "*".to_string(),
                    },
                    ASTNode::Quantified {
                        element: Box::new(token("quoted_string", "q")),
                        quantifier: "?".to_string(),
                    },
                    token("quoted_string", "z"),
                ],
            },
        );
        grammar_tree.insert("zref".to_string(), token("quoted_string", "k"));
        let rule_order = vec!["top".to_string(), "zref".to_string()];
        let rendered = generator
            .generate_parser(&grammar_tree, &rule_order, "quant_guard_test.rs")
            .expect("parser generation should succeed");
        let flat = nospace(&rendered);
        assert!(
            !flat.contains("parser.input.as_bytes()[parser.position]"),
            "layout-skipping grammars must not peek the raw next byte at quantified sites, got: {rendered}"
        );
        assert!(
            !flat.contains("parser.furthest_position=parser.position"),
            "layout-skipping grammars must not emit the furthest emulation, got: {rendered}"
        );
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
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
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
        // `ParseContent::Shaped(PgenValue::Object(...))` (`-0105`
        // REPRESENTATION vintage — previously `Json(Value::Object)`).
        // prettyplease line-breaks long constructor chains, so match on the
        // whitespace-stripped source.
        let rendered_nows: String = rendered.chars().filter(|c| !c.is_whitespace()).collect();
        assert!(
            rendered_nows.contains("ParseContent::Shaped(PgenValue::Object("),
            "non-Or rule with object-literal return annotation must emit the typed Shaped/Object carrier; rendered did not contain it. snippet around fn parse_r: {}",
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

    /// BRANCH-BROADCAST-FIX.3 — pins the tournament branch-arm ORDER:
    /// the branch transform must be evaluated BEFORE the arm rolls
    /// `parser.position` back to `parse_start`. `$text`/MatchedText is the
    /// one transform form that reads `parser.position`
    /// (`&parser.input[start_pos..parser.position]`), so the rollback-first
    /// order made every branch-level `$text` in a multi-branch tournament
    /// slice the EMPTY span (runtime-proven: `restrict:""`/`name:""` for
    /// matched single chars in the `.1` investigation). All other transform
    /// forms read the captured `content` only, which is why this order is
    /// the complete fix.
    #[test]
    fn tournament_branch_transform_evaluates_before_position_rollback() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "r".to_string(),
            ASTNode::Or {
                alternatives: vec![token("quoted_string", "D"), token("quoted_string", "S")],
            },
        );
        let rule_order = vec!["r".to_string()];

        // Both branches carry `-> $text` (the post-BRANCH-BROADCAST-FIX.2
        // broadcast shape for `r = ( "D" | "S" ) -> $text`).
        let matched_text_ann = || {
            Some(BranchAnnotation {
                annotation_type: "return_scalar".to_string(),
                annotation_content: "$text".to_string(),
                parsed_ast: Some(crate::ast_pipeline::UnifiedReturnAST::MatchedText),
            })
        };
        let mut annotations = Annotations::default();
        annotations
            .branch_return_annotations
            .insert("r".to_string(), vec![matched_text_ann(), matched_text_ann()]);
        let mut converted_branches: HashMap<String, Vec<Option<BranchAnnotation>>> =
            HashMap::new();
        for (rule, branches) in annotations.branch_return_annotations.iter() {
            converted_branches.insert(rule.clone(), branches.clone());
        }

        let mut generator = AstBasedGenerator::new("tournament_text_span_test".to_string());
        generator.enable_debug = false;
        generator.annotations = Some(annotations);
        generator.branch_return_annotations = converted_branches;

        let rendered = generator
            .generate_parser(&grammar_tree, &rule_order, "tournament_text_span_test.rs")
            .expect("parser generation should succeed");

        // Compare positions whitespace-insensitively (the rendered source may
        // be token-stream spaced or pretty-printed).
        let compact: String = rendered.chars().filter(|c| !c.is_whitespace()).collect();
        let arm_anchor = "letcandidate_end=parser.position;";
        let transform_anchor = "lettransformed=";
        let rollback_anchor = "parser.position=parse_start;";

        // RGX-0078.5.i.7 (MTB-B) — the rendered parser now carries TWO
        // tournament families: the protocol arms (which evaluate transforms
        // inline — the ordering obligation applies) and the fused
        // `cascade_match_*` arms (which evaluate NO transforms by design —
        // values are built once over the derivation tape, where the `$text`
        // end is the build cursor). Each arm's window is bounded by the next
        // arm anchor so a later arm's transform can never satisfy an earlier
        // arm's check.
        let mut arm_count = 0usize;
        let mut transform_arm_count = 0usize;
        let mut search_from = 0usize;
        while let Some(rel) = compact[search_from..].find(arm_anchor) {
            arm_count += 1;
            let arm_start = search_from + rel + arm_anchor.len();
            let window_end = compact[arm_start..]
                .find(arm_anchor)
                .map(|off| arm_start + off)
                .unwrap_or(compact.len());
            let window = &compact[arm_start..window_end];
            let rollback_idx = window.find(rollback_anchor).unwrap_or_else(|| {
                panic!("branch arm {} has no position rollback", arm_count)
            });
            if let Some(transform_idx) = window.find(transform_anchor) {
                transform_arm_count += 1;
                assert!(
                    transform_idx < rollback_idx,
                    "branch arm {}: the transform must be evaluated BEFORE the \
                     position rollback (transform at {}, rollback at {}) — \
                     rollback-first re-introduces the empty-span `$text` defect",
                    arm_count,
                    transform_idx,
                    rollback_idx
                );
            }
            search_from = arm_start;
        }
        assert!(
            transform_arm_count >= 2,
            "expected a 2-branch value-evaluating tournament (found {} transform arms of {} total)",
            transform_arm_count,
            arm_count
        );

        // And the MatchedText emission itself is present for the branches.
        assert!(
            compact.contains("&parser.input[start_pos..parser.position]"),
            "MatchedText transform should slice the input span"
        );
        // The MTB dual of the ordering obligation (the `-0101` $text fix): a
        // build-side `$text` end must be the BUILD CURSOR — never the frozen
        // whole-match end. Two byte-equal forms satisfy it: the verbatim
        // path's sync-then-read (`position = deriv_pos;` before the
        // `..parser.position` slice) and — since RGX-0078.5.j.2 STEP-2a — the
        // in-place VALUE-PURE branch's direct slice to `deriv_pos` (a
        // MatchedText branch is value-pure, so the fused build computes the
        // Terminal in place with no `position` involvement at all).
        assert!(
            compact.contains("parser.position=parser.deriv_pos;")
                || compact.contains("&parser.input[start_pos..parser.deriv_pos]"),
            "a build-side $text transform must end at the build cursor (sync form or direct in-place slice)"
        );
    }

    /// GRAMMAR-WELLFORMED.G.4.6 — pins the TRANSACTIONAL PARSE-COVERAGE wiring
    /// at the codegen level so a future refactor of the rule-method or
    /// `try_parse` template cannot silently drop it (which would regress the
    /// certifying-linter witness side back to the broken AST-walk numbers).
    /// All four pieces must be present and consistent: (1) the opt-in state
    /// (`coverage_stack` + `coverage_enabled`), (2) the per-rule-entry push,
    /// (3) the `try_parse` snapshot/truncate that makes it transactional, and
    /// (4) the public `enable_coverage` / `exercised_rule_names` accessors.
    #[test]
    fn transactional_parse_coverage_wiring_is_emitted_at_codegen() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert("r".to_string(), token("quoted_string", "v"));
        let rule_order = vec!["r".to_string()];

        let mut generator = AstBasedGenerator::new("coverage_wiring_test".to_string());
        generator.enable_debug = false;
        let rendered = generator
            .generate_parser(&grammar_tree, &rule_order, "coverage_wiring_test.rs")
            .expect("parser generation should succeed");

        for needle in [
            "coverage_stack",   // the transactional stack field
            "coverage_enabled", // the opt-in gate
            "saved_coverage_len", // try_parse snapshot of the stack length
            "enable_coverage",  // public opt-in accessor
            "exercised_rule_names", // public read-back accessor
            "exercised_rule_entry_counts", // RGX-0078.5.h.1b — committed-count histogram accessor
        ] {
            assert!(
                rendered.contains(needle),
                "generated parser must carry the parse-coverage wiring token `{}`; rendered did not contain it",
                needle
            );
        }
        // The push must be gated by the opt-in flag (zero cost when disabled),
        // and the rollback must truncate to the snapshot (transactionality).
        // Whitespace-agnostic: `generate_parser` may return either the raw
        // spaced token stream or rustfmt-formatted source.
        let nospace: String = rendered.chars().filter(|c| !c.is_whitespace()).collect();
        assert!(
            nospace.contains("ifself.coverage_enabled"),
            "the per-rule-entry coverage push must be gated by coverage_enabled"
        );
        assert!(
            nospace.contains("coverage_stack.truncate(saved_coverage_len)"),
            "try_parse must truncate the coverage stack back to its pre-speculation length on failure"
        );
        // GRAMMAR-WELLFORMED.H.10.2.2 — the memo path must (a) capture the
        // body's coverage delta into the MemoEntry on success and (b) replay
        // it on every cache hit; without the replay, a subtree first parsed
        // inside a rolled-back speculation and then memo-hit on the committed
        // path is silently absent from the witness record (the completeness
        // half of the coverage-record guarantee).
        assert!(
            nospace.contains("letmemo_coverage_checkpoint=self.coverage_stack.len()"),
            "memoized_call must snapshot the coverage-stack length before executing the rule body"
        );
        assert!(
            nospace
                .contains("self.coverage_stack[memo_coverage_checkpoint..].to_vec()"),
            "memoized_call must store the body's coverage delta in the MemoEntry on success"
        );
        assert!(
            nospace.contains("self.coverage_stack.extend_from_slice(coverage)"),
            "a memo hit must replay the cached coverage delta onto the live coverage stack"
        );
        // MEMO-STORE-SOUNDNESS.2 — the memo must be TAINT-GATED with
        // write-epoch VALIDATION: the body's predicate-evaluation delta
        // decides taint, tainted outcomes (failure AND success) are stamped
        // with the store write epoch at insert, validated on every hit, and
        // evicted once the store has moved (the memo key is store-blind, so
        // an unvalidated store-dependent entry would replay stale
        // verdicts/trees on same-position retries after a store change;
        // outright taint-EXCLUSION is NOT acceptable — it collapses packrat
        // protection on predicate-heavy grammars, measured 117× on SV
        // scr1_core_top, session #49).
        assert!(
            nospace.contains(
                "letmemo_taint_snapshot=self.semantic_runtime_state.predicate_evaluations()"
            ),
            "memoized_call must snapshot the predicate-evaluation counter before the rule body"
        );
        assert!(
            nospace.contains(
                "self.semantic_runtime_state.predicate_evaluations()!=memo_taint_snapshot"
            ),
            "memoized_call must compare the predicate-evaluation counter after the rule body"
        );
        assert!(
            nospace.contains("memo_fail_tainted:rustc_hash::FxHashMap<(RuleId,usize),u64>"),
            "the parser must carry the epoch-stamped tainted-failure map beside the lean pure set"
        );
        assert!(
            nospace.contains("tainted_at_epoch:ifmemo_store_tainted{"),
            "a memoized success must be epoch-stamped when the body was store-tainted"
        );
        assert!(
            nospace.contains("self.memo_fail_tainted.remove(&key)"),
            "a stale tainted failure must be EVICTED (then re-parsed), not replayed"
        );
        assert!(
            nospace.contains("self.memo.remove(&key)"),
            "a stale tainted success must be EVICTED (then re-parsed), not replayed"
        );
    }

    /// Regression test for the implicit `-> $1` over-reach.
    ///
    /// The original commit (`6ea521e`) listed "single quantified element"
    /// among the bodies that should get a synthetic `-> $1` default. That
    /// was wrong: for `RULE := X+` the body parses to `Quantified(matches,
    /// "+")`, and the synthetic `-> $1` then extracts `elements[0].content`
    /// — silently dropping every match past the first. The visible symptom
    /// was `concatenation = piece+` in `regex.ebnf` only ever surfacing the
    /// FIRST piece in the typed JSON output for any multi-piece input.
    ///
    /// The fix in `body_has_single_element` excludes `ASTNode::Quantified`,
    /// matching how multi-element Sequences are already excluded. An
    /// unannotated Quantified-bodied rule now emits raw `Quantified(...)`
    /// content (which serialises as the array of all matches), and authors
    /// who want a different transform must declare it explicitly.
    #[test]
    fn quantified_bodied_rule_with_no_annotation_does_not_get_synthetic_dollar_one() {
        // Build `r = a+` (no return annotation). `a` is a TWO-element sequence
        // deliberately: since RGX-0078.5.i.4 (P1a) the tiny leaf `a` is
        // inline-DECIDED, so its body is emitted INSIDE `parse_r` — and a
        // single-element `a` would carry its own legitimate synthetic `-> $1`
        // passthrough, tripping this test's textual carve of `fn parse_r`
        // (the assertion targets R's transform, not the inlined child's).
        // A multi-element Sequence gets no synthetic transform, keeping the
        // carve unambiguous while still exercising the production emission
        // path (plan built, `a` inlined).
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "r".to_string(),
            ASTNode::Quantified {
                element: Box::new(token("rule_reference", "a")),
                quantifier: "+".to_string(),
            },
        );
        grammar_tree.insert(
            "a".to_string(),
            ASTNode::Sequence {
                elements: vec![token("quoted_string", "a"), token("quoted_string", "b")],
            },
        );
        let rule_order = vec!["r".to_string(), "a".to_string()];

        let mut generator = AstBasedGenerator::new("quant_no_synthetic_test".to_string());
        generator.enable_debug = false;

        let rendered = generator
            .generate_parser(&grammar_tree, &rule_order, "quant_no_synthetic_test.rs")
            .expect("parser generation should succeed");

        // Carve out just `fn parse_r` so the assertion can't be confused by
        // any other rule's body.
        let parse_r_body: String = rendered
            .lines()
            .skip_while(|l| !l.contains("fn parse_r"))
            .take_while(|l| !l.contains("fn parse_a") && !l.contains("fn parse_full"))
            .collect::<Vec<_>>()
            .join("\n");

        // The synthetic `-> $1` would emit a `let result = { match &result {
        // ... ParseContent::Quantified(elements, _) if !elements.is_empty()
        // => elements[0usize].content.clone() ... } };` shadow rebind right
        // after the parse logic. With the fix that rebind must NOT appear
        // for an unannotated Quantified-bodied rule.
        assert!(
            !parse_r_body.contains("elements [0usize] . content . clone")
                && !parse_r_body.contains("elements[0usize].content.clone"),
            "unannotated Quantified-bodied rule must not emit a synthetic `-> $1` \
             extraction; parse_r body was:\n{}",
            parse_r_body
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
    fn phase_2_m1_typed_entry_emits_only_when_emit_typed_entry_skeleton_flag_is_set() {
        // Phase 2 M1 contract: when AstBasedGenerator.emit_typed_entry_skeleton is
        // true, the emitted parser carries `parse_full_<entry>_typed` returning
        // `ParseResult<serde_json::Value>` alongside the existing
        // `parse_full_<entry>` returning `ParseResult<ParseNode>`. Default-off behavior
        // is byte-unchanged: no typed method appears.
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

        // Flag on: legacy emit + parallel typed impl block.
        let mut generator_on = AstBasedGenerator::new("usage_test".to_string());
        generator_on.enable_debug = false;
        generator_on.emit_typed_entry_skeleton = true;
        let rendered_on = generator_on
            .generate_parser(&grammar_tree, &rule_order, "phase_2_m1.rs")
            .expect("flag-on parser generation should succeed");
        assert!(
            rendered_on.contains("pub fn parse_full_package_declaration_typed"),
            "flag-on emit must include typed entry method, got: {}",
            rendered_on
        );
        assert!(
            rendered_on.contains("ParseResult<serde_json::Value>")
                || rendered_on.contains("ParseResult < serde_json :: Value >"),
            "typed method must return ParseResult<serde_json::Value>, got: {}",
            rendered_on
        );
        // The legacy method is preserved unchanged.
        assert!(
            rendered_on.contains("pub fn parse_full_package_declaration"),
            "flag-on emit must preserve legacy parse_full method, got: {}",
            rendered_on
        );
        // The typed method body delegates to the legacy method (M1 skeleton).
        assert!(
            rendered_on.contains("self.parse_full_package_declaration()"),
            "M1 typed body must wrap the legacy parse_full call, got: {}",
            rendered_on
        );
    }

    #[test]
    fn generated_parser_runtime_contract_owns_semantic_runtime_fields() {
        let rendered = pre_runtime_rendered_parser();

        // RGX-0078.5.i.2.t1 — re-pinned to the post-`.5.g` contract: the
        // compiled annotation table is the process-shared `&'static` one
        // (construction cache), not a per-instance owned copy. The `.5.g`
        // commit changed the emission without updating this pin.
        assert!(
            rendered.contains(
                "semantic_runtime_annotations: &'static crate::ast_pipeline::CompiledSemanticRuntimeAnnotations"
            ) || rendered.contains(
                "semantic_runtime_annotations : & 'static crate :: ast_pipeline :: CompiledSemanticRuntimeAnnotations"
            ),
            "generated parser should reference the shared compiled semantic runtime annotations table (RGX-0078.5.g), got: {}",
            rendered
        );
        assert!(
            rendered.contains("semantic_runtime_state: crate::ast_pipeline::SemanticRuntimeState"),
            "generated parser should own semantic runtime state, got: {}",
            rendered
        );
        // `PGEN-RGX-0078-0198` — re-pinned: the per-parse reset is IN PLACE
        // (preserving facts/fact indices and the re-seeded predicate defs)
        // instead of the historical whole-state `SemanticRuntimeState::new()`
        // replacement ceremony.
        assert!(
            (rendered.contains("reset_for_new_parse(") || rendered.contains("reset_for_new_parse ("))
                && (rendered.contains("predicate_defs_map()")
                    || rendered.contains("predicate_defs_map ()")),
            "parse() should reset semantic runtime state in place from the compiled predicate defs, got: {}",
            rendered
        );
        assert!(
            rendered.contains("memo: rustc_hash :: FxHashMap < (RuleId, usize), MemoEntry < 'input > >")
                || rendered.contains("memo: rustc_hash::FxHashMap<(RuleId, usize), MemoEntry<'input>>"),
            "generated parser should memoize rich entries instead of bare shaped nodes, got: {}",
            rendered
        );
        // PARSE-TERMINATION.6 — the SPLIT memo: failures live in a lean set.
        assert!(
            rendered.contains("memo_fail: rustc_hash :: FxHashSet < (RuleId, usize) >")
                || rendered.contains("memo_fail: rustc_hash::FxHashSet<(RuleId, usize)>"),
            "generated parser should carry the lean memo_fail set for cached failures, got: {}",
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
            rendered.contains("pre_predicates_for_rule_id"),
            "generated parser should use the explicit pre-predicate rule view (id-indexed, K3c), got: {}",
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
        // SEMREF-SHAPED.2: a `$name`/`$a.b` ref on a rule whose
        // content is a shaped `->` structure resolves against that
        // produced structure (object-key path → scalar), keyed purely
        // on the content variant so the raw (no-`->`) path is
        // untouched. `-0105` REPRESENTATION vintage: the shaped
        // carrier is `Shaped(PgenValue)`; the emitted resolver must
        // reference the retired `Json` variant NOWHERE (checked as a
        // code ref, `Json(`) — the permanent tripwire that kept the
        // `-0106` lib-side retirement regen-free. prettyplease
        // line-breaks long patterns, so match on the
        // whitespace-stripped source.
        let rendered_nows: String = rendered.chars().filter(|c| !c.is_whitespace()).collect();
        assert!(
            rendered_nows.contains("ParseContent::Shaped(shaped_root)=root_content"),
            "generated resolver should resolve named refs against a shaped -> structure, got: {}",
            rendered
        );
        assert!(
            !rendered_nows.contains("ParseContent::Json("),
            "generated parser must not reference the retired ParseContent::Json variant"
        );
        // SV-EXH-PROOF.3.3.4.a.2 (PGEN-SV-EXH-PROOF-0027) via `-0105`:
        // the segment dispatch handles BOTH `.name` property access and
        // `[N]` indexed access on the shaped carrier — the property
        // branch binary-searches the key-sorted pair slice (the
        // byte-exact `serde_json::Map::get` equivalent) and the indexed
        // branch reads `items.get(index)`. Assert one needle per branch.
        assert!(
            rendered_nows.contains("pairs.binary_search_by("),
            "generated resolver should walk `.name` segments via the sorted-pair binary search, got: {}",
            rendered
        );
        assert!(
            rendered_nows.contains("items.get(index)"),
            "generated resolver should walk `[N]` segments via the shaped array index, got: {}",
            rendered
        );
        // Scalar-leaf coercion renders through serde's own formatter
        // (`serde_json::Number`), never a hand-rolled renderer. Two
        // needles (the arm pattern + the coercion call) rather than one,
        // so the assertion is independent of the formatter's brace style.
        assert!(
            rendered_nows.contains("PgenValue::Int(number)")
                && rendered_nows.contains("serde_json::Number::from(number).to_string()"),
            "generated resolver should coerce scalar shaped fields (Int) through serde_json::Number, got: {}",
            rendered
        );
    }

    #[test]
    fn generated_parser_runtime_contract_orders_effects_before_post_predicates() {
        let rendered = post_runtime_rendered_parser();

        assert!(
            rendered.contains("effect_directives_for_rule_id"),
            "generated parser should use the explicit effect-directive rule view (id-indexed, K3c), got: {}",
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
            "generated parser helper should detach semantic runtime state for the txn machinery, got: {}",
            rendered
        );
        // PARSE-TERMINATION.3.1: the O(N) full-state clone is replaced by an O(1) entry
        // checkpoint; the parent-failure restore is an O(changes) rollback to it.
        assert!(
            rendered.contains("let semantic_runtime_checkpoint")
                && rendered.contains(".checkpoint()"),
            "generated parser helper should snapshot an O(1) entry checkpoint (not a full clone), got: {}",
            rendered
        );
        assert!(
            !rendered.contains("original_semantic_runtime_state.clone()"),
            "generated parser helper must NOT clone the full semantic runtime state per rule (the O(N^2) regression), got: {}",
            rendered
        );
        assert!(
            rendered.contains("rollback_to_named(semantic_runtime_checkpoint"),
            "generated parser should rollback to the entry checkpoint when the parent rule fails, got: {}",
            rendered
        );
        assert!(
            rendered.contains("if result.is_err()"),
            "generated parser should restore semantic runtime state when the parent rule fails, got: {}",
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

    // `DEFAULT-PROFILE.2`: a directive-bearing grammar's parser CARRIES its
    // declared default profile. The payload must be the STRUCTURED form the
    // real `.ebnf` pipeline produces (`from_named_payload` parses `pcre2` to
    // an Identifier); `compile_default_profile` requires a structured scalar.
    fn default_profile_rendered_parser() -> &'static str {
        static RENDERED: OnceLock<String> = OnceLock::new();
        RENDERED
            .get_or_init(|| {
                let mut generator = generator_with_named_semantic("entry", vec![]);
                let mut annotations = Annotations::default();
                annotations.semantic_annotations.insert(
                    "entry".to_string(),
                    vec![SemanticAnnotation::Named {
                        name: "default_profile".to_string(),
                        ast: UnifiedSemanticAST::from_named_payload("default_profile", "pcre2"),
                    }],
                );
                generator.annotations = Some(annotations);
                let mut grammar_tree = HashMap::new();
                grammar_tree.insert("entry".to_string(), token("quoted_string", "x"));
                let rule_order = vec!["entry".to_string()];
                generator
                    .generate_parser(
                        &grammar_tree,
                        &rule_order,
                        "semantic_default_profile_usage.rs",
                    )
                    .expect("parser generation should succeed")
            })
            .as_str()
    }

    #[test]
    fn generated_parser_default_profile_contract_carries_the_declared_default() {
        let rendered = default_profile_rendered_parser();
        assert!(
            rendered.contains("DEFAULT_GRAMMAR_PROFILE: &'static str = \"pcre2\""),
            "a @default_profile grammar should emit the declared-default constant, got: {}",
            rendered
        );
        // The constructor starts on the declared default AND `set_grammar_profile(None)`
        // restores it — the two `Self::DEFAULT_GRAMMAR_PROFILE.to_string()` consumers.
        assert!(
            rendered
                .matches("Self::DEFAULT_GRAMMAR_PROFILE.to_string()")
                .count()
                >= 2,
            "constructor init AND setter restore should both read the constant, got: {}",
            rendered
        );
        assert!(
            !rendered.contains("grammar_profile: None"),
            "a @default_profile grammar must not start on the permissive unset state, got: {}",
            rendered
        );
    }

    #[test]
    fn generated_parser_without_default_profile_directive_keeps_the_permissive_unset_state() {
        // Byte-identity leg for non-bearing grammars: no constant, today's
        // exact constructor init and permissive setter.
        let rendered = profile_guard_rendered_parser();
        assert!(
            !rendered.contains("DEFAULT_GRAMMAR_PROFILE"),
            "a grammar without @default_profile must not emit the constant, got: {}",
            rendered
        );
        assert!(
            rendered.contains("grammar_profile: None"),
            "a grammar without @default_profile keeps the unset-profile constructor init, got: {}",
            rendered
        );
        assert!(
            rendered.contains("self.grammar_profile = profile.map(|value| value.to_string())"),
            "a grammar without @default_profile keeps the plain permissive setter, got: {}",
            rendered
        );
    }

    // `PROFILE-ALIAS.2`: an alias-declaring grammar's parser CARRIES its
    // request-spelling map. The payload is the STRUCTURED object form the real
    // `.ebnf` pipeline produces (`from_named_payload` parses the map literal);
    // `@profiles` on the same rule supplies the declared universe the alias
    // targets validate against.
    fn profile_alias_rendered_parser() -> &'static str {
        static RENDERED: OnceLock<String> = OnceLock::new();
        RENDERED
            .get_or_init(|| {
                let mut generator = generator_with_named_semantic("entry", vec![]);
                let mut annotations = Annotations::default();
                annotations.semantic_annotations.insert(
                    "entry".to_string(),
                    vec![
                        SemanticAnnotation::Named {
                            name: "profile_alias".to_string(),
                            ast: UnifiedSemanticAST::from_named_payload(
                                "profile_alias",
                                "{ \"ieee1800-2017\": sv_2017, \"2017\": sv_2017 }",
                            ),
                        },
                        SemanticAnnotation::Named {
                            name: "profiles".to_string(),
                            ast: UnifiedSemanticAST::from_named_payload(
                                "profiles",
                                "[\"sv_2017\"]",
                            ),
                        },
                    ],
                );
                generator.annotations = Some(annotations);
                let mut grammar_tree = HashMap::new();
                grammar_tree.insert("entry".to_string(), token("quoted_string", "x"));
                let rule_order = vec!["entry".to_string()];
                generator
                    .generate_parser(
                        &grammar_tree,
                        &rule_order,
                        "semantic_profile_alias_usage.rs",
                    )
                    .expect("parser generation should succeed")
            })
            .as_str()
    }

    #[test]
    fn generated_parser_profile_alias_contract_carries_the_declared_map() {
        let rendered = profile_alias_rendered_parser();
        assert!(
            rendered.contains("GRAMMAR_PROFILE_ALIASES"),
            "a @profile_alias grammar should emit the alias constant, got: {}",
            rendered
        );
        // Sorted by spelling (BTreeMap order): "2017" precedes "ieee1800-2017"
        // even though the payload declared them in the other order.
        let first = rendered
            .find("(\"2017\", \"sv_2017\")")
            .expect("first alias pair should be embedded");
        let second = rendered
            .find("(\"ieee1800-2017\", \"sv_2017\")")
            .expect("second alias pair should be embedded");
        assert!(
            first < second,
            "alias pairs should be emitted sorted by spelling, got: {}",
            rendered
        );
        // The setter resolves explicit spellings through the map…
        assert!(
            rendered.contains("Self::resolve_grammar_profile_alias(value).to_string()"),
            "the setter should resolve requested spellings through the alias map, got: {}",
            rendered
        );
        // …while (no `@default_profile` here) the constructor stays unset.
        assert!(
            rendered.contains("grammar_profile: None"),
            "an alias-only grammar keeps the unset-profile constructor init, got: {}",
            rendered
        );
    }

    #[test]
    fn generated_parser_without_profile_alias_directive_emits_no_alias_surface() {
        // Byte-identity leg for non-bearing grammars: no constant, no resolver.
        let rendered = profile_guard_rendered_parser();
        assert!(
            !rendered.contains("GRAMMAR_PROFILE_ALIASES"),
            "a grammar without @profile_alias must not emit the alias constant, got: {}",
            rendered
        );
        assert!(
            !rendered.contains("resolve_grammar_profile_alias"),
            "a grammar without @profile_alias must not emit the resolver, got: {}",
            rendered
        );
        let default_rendered = default_profile_rendered_parser();
        assert!(
            !default_rendered.contains("GRAMMAR_PROFILE_ALIASES"),
            "a default-profile-only grammar must not emit the alias surface, got: {}",
            default_rendered
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
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
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
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
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
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
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
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
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
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
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
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
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
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
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
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
        };

        let rendered = generator.generate_parse_method("start", &std::collections::HashMap::new(), &[]).to_string();
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

    // GRAMMAR-WELLFORMED.H.11.5: the static introducer-claim analysis that
    // drives per-introducer comment-arm suppression in the layout skippers.
    fn h115_atom(token_type: &str, value: &str) -> ASTNode {
        ASTNode::Atom {
            value: ASTValue::Token(vec![
                TokenValue::String(token_type.to_string()),
                TokenValue::String(value.to_string()),
            ]),
        }
    }

    fn h115_generator() -> AstBasedGenerator {
        AstBasedGenerator {
            grammar_name: "claims_test".to_string(),
            entry_rule: None,
            logger: None,
            annotations: None,
            branch_return_annotations: HashMap::new(),
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
        }
    }

    #[test]
    fn regex_mandatory_prefix_analysis_is_sound_on_the_shipped_shapes() {
        // Tokens whose every match starts with the introducer — claims.
        // vhdl `hash := trivia /#/`
        assert!(AstBasedGenerator::regex_pattern_mandatorily_starts_with(
            "#", "#"
        ));
        // SV `line_comment` / `block_comment` (mandatory introducer head —
        // they are then filtered out by the comment-defining classifier).
        assert!(AstBasedGenerator::regex_pattern_mandatorily_starts_with(
            r"\/\/[^\n]*(\n|$)",
            "//"
        ));
        assert!(AstBasedGenerator::regex_pattern_mandatorily_starts_with(
            r"\/\*([^*]|\*+[^*\/])*\*+\/",
            "/*"
        ));
        assert!(AstBasedGenerator::regex_pattern_mandatorily_starts_with(
            "#abc", "#"
        ));
        // Repetition head: `/{2}` mandatorily starts with `//`.
        assert!(AstBasedGenerator::regex_pattern_mandatorily_starts_with(
            "/{2}", "//"
        ));
        // NON-claims: a shorter token, an optional introducer, an open
        // content class, alternation divergence.
        assert!(!AstBasedGenerator::regex_pattern_mandatorily_starts_with(
            "/", "//"
        ));
        assert!(!AstBasedGenerator::regex_pattern_mandatorily_starts_with(
            "a?#b", "#"
        ));
        assert!(!AstBasedGenerator::regex_pattern_mandatorily_starts_with(
            r"([^\r\n]*)",
            "#"
        ));
        assert!(!AstBasedGenerator::regex_pattern_mandatorily_starts_with(
            "(?:x|#)y", "#"
        ));
        assert!(!AstBasedGenerator::regex_pattern_mandatorily_starts_with(
            "[a-zA-Z_][a-zA-Z0-9_]*",
            "#"
        ));
    }

    #[test]
    fn non_comment_claim_analysis_matches_the_shipped_grammar_shapes() {
        let generator = h115_generator();
        // SV-shaped inventory: a standalone `#` string terminal (delays /
        // param lists — a NON-comment meaning) + comment-DEFINING tokens.
        let mut tree: HashMap<String, ASTNode> = HashMap::new();
        tree.insert("hash".to_string(), h115_atom("quoted_string", "#"));
        tree.insert(
            "line_comment".to_string(),
            h115_atom("regex", r"\/\/[^\n]*(\n|$)"),
        );
        tree.insert(
            "block_comment".to_string(),
            h115_atom("regex", r"\/\*([^*]|\*+[^*\/])*\*+\/"),
        );
        tree.insert(
            "ident".to_string(),
            h115_atom("regex", "[a-zA-Z_][a-zA-Z0-9_]*"),
        );
        assert!(generator.grammar_claims_introducer_as_non_comment(&tree, "#"));
        // line/block comment tokens AGREE with the engine's comment arms —
        // they must not suppress them.
        assert!(!generator.grammar_claims_introducer_as_non_comment(&tree, "//"));
        assert!(!generator.grammar_claims_introducer_as_non_comment(&tree, "/*"));

        // vhdl-shaped: `hash := /#/` (based-literal delimiter, no unbounded
        // tail) is a non-comment claim.
        let mut vhdl_tree: HashMap<String, ASTNode> = HashMap::new();
        vhdl_tree.insert("hash".to_string(), h115_atom("regex", "#"));
        assert!(generator.grammar_claims_introducer_as_non_comment(&vhdl_tree, "#"));

        // ebnf-meta-grammar-shaped: the two-token comment shape
        // `("#" | "//") comment_content` is comment-DEFINING, so the arms
        // stay (the decisive stash A/B showed suppressing them regresses the
        // generated ebnf parser on real grammar files).
        let mut ebnf_tree: HashMap<String, ASTNode> = HashMap::new();
        ebnf_tree.insert(
            "line_comment".to_string(),
            ASTNode::Sequence {
                elements: vec![
                    ASTNode::Or {
                        alternatives: vec![
                            h115_atom("quoted_string", "#"),
                            h115_atom("quoted_string", "//"),
                        ],
                    },
                    h115_atom("rule_reference", "comment_content"),
                ],
            },
        );
        ebnf_tree.insert(
            "comment_content".to_string(),
            h115_atom("regex", r"([^\r\n]*)"),
        );
        assert!(!generator.grammar_claims_introducer_as_non_comment(&ebnf_tree, "#"));
        assert!(!generator.grammar_claims_introducer_as_non_comment(&ebnf_tree, "//"));

        // The same literal WITHOUT a content tail is a real token — claim.
        let mut bare_tree: HashMap<String, ASTNode> = HashMap::new();
        bare_tree.insert("hash".to_string(), h115_atom("quoted_string", "#"));
        assert!(generator.grammar_claims_introducer_as_non_comment(&bare_tree, "#"));

        // A lookahead-only occurrence must not claim (nothing is consumed).
        let mut la_tree: HashMap<String, ASTNode> = HashMap::new();
        la_tree.insert(
            "guard".to_string(),
            ASTNode::Lookahead {
                element: Box::new(h115_atom("quoted_string", "#")),
                positive: true,
            },
        );
        assert!(!generator.grammar_claims_introducer_as_non_comment(&la_tree, "#"));
    }

    #[test]
    fn regex_comment_defining_classifier_is_sound_on_the_shipped_shapes() {
        // SV comment tokens: every match starts with the introducer and runs
        // an unbounded tail — comment-defining.
        assert!(AstBasedGenerator::regex_pattern_is_comment_defining(
            r"\/\/[^\n]*(\n|$)",
            "//"
        ));
        assert!(AstBasedGenerator::regex_pattern_is_comment_defining(
            r"\/\*([^*]|\*+[^*\/])*\*+\/",
            "/*"
        ));
        // vhdl `/#/`: no unbounded tail — NOT comment-defining.
        assert!(!AstBasedGenerator::regex_pattern_is_comment_defining(
            "#", "#"
        ));
        // Only optionally introducer-prefixed — NOT comment-defining.
        assert!(!AstBasedGenerator::regex_pattern_is_comment_defining(
            r"(?:#|x)[^\n]*",
            "#"
        ));
    }

    #[test]
    fn claimed_introducers_suppress_their_comment_arms_in_both_skippers() {
        let generator = h115_generator();
        generator.uses_match_regex.set(true);

        // No claims: both skippers carry all three comment arms.
        let rendered_all = generator
            .generate_helper_methods("claims_test.rs", &HashMap::new())
            .to_string();
        // (`== b'#'` is the comparison form only the skipper arms use —
        // `looks_like_rule_definition_boundary` pattern-matches `b'#' =>`.)
        assert!(rendered_all.contains("== b'#'"));
        assert!(rendered_all.contains("regex_token_matches_at_cursor"));
        assert!(rendered_all.contains("allow_comment_skip"));

        // Three standalone (non-comment) claims: NO comment arm and no dead
        // dynamic-guard helper; both skippers reduce to whitespace skipping.
        let mut tree: HashMap<String, ASTNode> = HashMap::new();
        tree.insert("hash".to_string(), h115_atom("quoted_string", "#"));
        tree.insert("dslash".to_string(), h115_atom("quoted_string", "//"));
        tree.insert("copen".to_string(), h115_atom("quoted_string", "/*"));
        let rendered_none = generator
            .generate_helper_methods("claims_test.rs", &tree)
            .to_string();
        assert!(!rendered_none.contains("== b'#'"));
        assert!(!rendered_none.contains("regex_token_matches_at_cursor"));
        assert!(!rendered_none.contains("allow_comment_skip"));
        assert!(rendered_none.contains("fn consume_layout_for_regex"));
        assert!(rendered_none.contains("fn consume_layout_for_terminal"));

        // SV-shaped claims: only the `#` arm disappears — the grammar's
        // line/block comment tokens agree with the `//` / `/*` arms.
        let mut sv_tree: HashMap<String, ASTNode> = HashMap::new();
        sv_tree.insert("hash".to_string(), h115_atom("quoted_string", "#"));
        sv_tree.insert(
            "line_comment".to_string(),
            h115_atom("regex", r"\/\/[^\n]*(\n|$)"),
        );
        sv_tree.insert(
            "block_comment".to_string(),
            h115_atom("regex", r"\/\*([^*]|\*+[^*\/])*\*+\/"),
        );
        let rendered_sv = generator
            .generate_helper_methods("claims_test.rs", &sv_tree)
            .to_string();
        assert!(!rendered_sv.contains("== b'#'"));
        assert!(rendered_sv.contains("== b'*'"));
        assert!(rendered_sv.contains("regex_token_matches_at_cursor"));
    }

    #[test]
    fn semantic_usage_codegen_records_recovery_events_in_helper_methods() {
        let generator = AstBasedGenerator {
            grammar_name: "usage_test".to_string(),
            entry_rule: None,
            logger: None,
            annotations: None,
            branch_return_annotations: HashMap::new(),
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
        };

        let rendered = generator
            .generate_helper_methods("semantic_usage.rs", &HashMap::new())
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
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
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
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
        };

        let types_rendered = generator.generate_types().to_string();
        assert!(
            types_rendered.contains("pub struct CoverageTargetEvent"),
            "generated types should include CoverageTargetEvent, got: {}",
            types_rendered
        );

        let parse_rendered = generator.generate_parse_method("start", &std::collections::HashMap::new(), &[]).to_string();
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
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
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
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
        };

        let rendered = generator
            .generate_helper_methods("semantic_usage.rs", &HashMap::new())
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
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
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
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
        };

        let types_rendered = generator.generate_types().to_string();
        assert!(
            types_rendered.contains("pub struct NegativeCaseEvent"),
            "generated types should include NegativeCaseEvent, got: {}",
            types_rendered
        );

        let parse_rendered = generator.generate_parse_method("start", &std::collections::HashMap::new(), &[]).to_string();
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
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
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
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
        };

        let rendered = generator
            .generate_helper_methods("semantic_usage.rs", &HashMap::new())
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
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
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
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
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

        let parse_rendered = generator.generate_parse_method("start", &std::collections::HashMap::new(), &[]).to_string();
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
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
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
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
        };

        let rendered = generator
            .generate_helper_methods("semantic_usage.rs", &HashMap::new())
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
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
        };

        let logic = generator
            .generate_node_parsing_logic(&or_rule_three(), "expr", "semantic_usage.rs")
            .expect("or-node logic generation should succeed");
        let rendered = logic.to_string();
        // RGX-0078.5.i.2 (P0): the rotated evaluation order is emitted as
        // direct `(step + offset) % n` iteration (no per-execution
        // `Vec<usize>` + `rotate_left`), and the partition-group String is
        // computed only inside the effectively-enabled branch.
        let has_modulo_order = rendered
            .contains("(evaluation_step + deterministic_partition_offset) % 3usize")
            || rendered
                .contains("( evaluation_step + deterministic_partition_offset ) % 3usize");

        assert!(
            rendered.contains("effective_deterministic_partition_enabled")
                && rendered.contains("effective_deterministic_partition_group")
                && rendered.contains("deterministic_partition_offset_runtime")
                && has_modulo_order
                && !rendered.contains("rotate_left")
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
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
        };

        let rendered = generator
            .generate_helper_methods("semantic_usage.rs", &HashMap::new())
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
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
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
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
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
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
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
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
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
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
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
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
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
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
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
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
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
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
        };

        let rendered = generator
            .generate_helper_methods("semantic_usage.rs", &HashMap::new())
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
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
        };

        let rendered = generator
            .generate_helper_methods("semantic_usage.rs", &HashMap::new())
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
    fn native_unresolved_builtins_const_matches_dispatch() {
        // UNDEFINED-REF-DIAGNOSTICS.2 oracle lock: the const the linter
        // consumes must equal the dispatch's native arms IN BOTH DIRECTIONS.
        // (1) every const name emits NON-stub tokens (a native matcher);
        // (2) a non-const name emits EXACTLY the bare Backtrack stub — so a
        // new native arm added without updating the const, or a const entry
        // without an arm, fails here.
        let generator = AstBasedGenerator::new("usage_test".to_string());
        let bare_stub_probe = generator
            .generate_unresolved_reference_method("definitely_not_a_native_builtin_probe")
            .to_string()
            .replace("definitely_not_a_native_builtin_probe", "NAME");
        assert!(
            bare_stub_probe.contains("Backtrack"),
            "the fallback arm must be the bare Backtrack stub, got: {}",
            bare_stub_probe
        );
        for native in AstBasedGenerator::NATIVE_UNRESOLVED_REFERENCE_BUILTINS {
            let rendered = generator
                .generate_unresolved_reference_method(native)
                .to_string()
                .replace(native, "NAME");
            assert_ne!(
                rendered, bare_stub_probe,
                "'{}' is in NATIVE_UNRESOLVED_REFERENCE_BUILTINS but the dispatch emits the bare \
                 stub for it — const and dispatch have drifted",
                native
            );
        }
        // Direction (2) is the probe assertion above: any name NOT in the
        // const must take the `_ =>` arm. Guard the const against silent
        // growth/shrink so a dispatch edit forces a conscious update here.
        assert_eq!(
            AstBasedGenerator::NATIVE_UNRESOLVED_REFERENCE_BUILTINS.len(),
            5,
            "the native-builtin allowlist changed — update the linter docs/book and this count"
        );
    }

    #[test]
    fn unresolved_reference_codegen_emits_native_any_char_matcher() {
        // REGEX-SELF-HOSTING.2/.5b: a `rule_reference` to `builtin_any_char` with no grammar definition
        // emits a native one-char matcher (no Rust `regex`), not the Backtrack stub.
        let generator = AstBasedGenerator::new("usage_test".to_string());

        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Sequence {
                elements: vec![token("rule_reference", "builtin_any_char")],
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
            rendered.contains("pub fn parse_builtin_any_char"),
            "expected a parse_builtin_any_char method, got: {}",
            rendered
        );
        assert!(
            rendered.contains("len_utf8"),
            "expected the native any_char matcher to advance by one UTF-8 char (len_utf8), got: {}",
            rendered
        );
        assert!(
            rendered.contains("chars"),
            "expected the native any_char matcher to read one char via chars().next(), got: {}",
            rendered
        );
        assert!(
            !rendered.contains("match_regex") && !rendered.contains("Regex"),
            "the native any_char matcher must NOT use Rust regex, got: {}",
            rendered
        );
    }

    #[test]
    fn unresolved_reference_codegen_emits_native_ascii_char_matcher() {
        // REGEX-SELF-HOSTING.5: a `rule_reference` to `builtin_ascii_char` with no grammar definition
        // emits a native one-ASCII-char matcher (`ch.is_ascii()` guard, no Rust `regex`) — the building
        // block for the range-negation idiom `unicode_char = !builtin_ascii_char builtin_any_char`.
        let generator = AstBasedGenerator::new("usage_test".to_string());

        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Sequence {
                elements: vec![token("rule_reference", "builtin_ascii_char")],
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
            rendered.contains("pub fn parse_builtin_ascii_char"),
            "expected a parse_builtin_ascii_char method, got: {}",
            rendered
        );
        assert!(
            rendered.contains("is_ascii"),
            "expected the native ascii_char matcher to guard on ch.is_ascii(), got: {}",
            rendered
        );
        assert!(
            !rendered.contains("match_regex") && !rendered.contains("Regex"),
            "the native ascii_char matcher must NOT use Rust regex, got: {}",
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

    /// P4-i (RGX-0078.5.i.6): the generation-time constant-truth gate for
    /// `@constraint` folding — the truth table pins every mirror obligation
    /// against the emitted evaluator's decision procedure.
    #[test]
    fn relational_constraint_constant_truth_gate() {
        // Every live regex-grammar constraint string folds (all prose).
        for prose in [
            "produces control character",
            "interpretation depends on escaped character",
            "may be disabled by compile options or UTF/lookbehind validation",
            "property name must be valid Unicode property",
            "must produce valid character value",
            "must be valid Unicode code point",
            "must be valid octal digits 0-7",
            "must be valid hexadecimal digits",
        ] {
            assert!(
                relational_constraint_is_provably_truthy(prose),
                "live prose constraint must fold: {prose:?}"
            );
        }
        // Prose that merely starts with a digit still folds (not an
        // identifier chain, not an f64).
        assert!(relational_constraint_is_provably_truthy("0-7 range prose"));

        // Structural / content-dependent / boolean shapes must NOT fold —
        // each line names the evaluator feature it could reach.
        for expression in [
            "",                    // empty -> runtime ERROR path, not truthy
            "   ",                 // trim-empty
            "a || b",              // top-level disjunction
            "a && b",              // top-level conjunction
            "!negated",            // negation
            "$1 <= $2",            // comparison + references
            "min <= max",          // comparison operators
            "x == y",              // equality
            "value<other>",        // bare '<'
            "a|b",                 // any '|' (split machinery)
            "a&b",                 // any '&'
            "$name",               // dollar reference
            "\"quoted\"",          // quote handling
            "'quoted'",            // quote handling
            "(grouped prose)",     // parenthesis stripping
            "validated",           // bare identifier -> reference resolution
            "_private",            // bare identifier (underscore lead)
            "a.b.c",               // bare dotted reference
            "config.enabled",      // bare dotted reference
            "1.5",                 // numeric truthiness
            "-2",                  // numeric truthiness
            "0",                   // falsy numeric
            "inf",                 // parses as f64
            "nan",                 // parses as f64
            "true",                // boolean special case
            "false",               // boolean special case
            "no",                  // semantic_truthy falsy word
            "off",                 // semantic_truthy falsy word
            "none",                // semantic_truthy falsy word
            "null",                // semantic_truthy falsy word
        ] {
            assert!(
                !relational_constraint_is_provably_truthy(expression),
                "must NOT fold: {expression:?}"
            );
        }
    }

    /// P4-i: a provably constant-true `@constraint` (with no `@requires` and
    /// no `@implies`) emits NO rule-exit guard; a real relational expression
    /// keeps the runtime path.
    #[test]
    fn constant_true_constraint_folds_to_no_emission() {
        let folded = generator_with_named_semantic(
            "control_escape",
            vec![("constraint", "\"produces control character\"")],
        );
        assert!(
            folded
                .semantic_relational_constraint_tokens("control_escape")
                .is_empty(),
            "constant-true prose constraint must emit nothing"
        );

        let kept = generator_with_named_semantic(
            "counted_rule",
            vec![("constraint", "\"$min <= $max\"")],
        );
        let tokens = kept.semantic_relational_constraint_tokens("counted_rule");
        assert!(
            tokens.to_string().contains("evaluate_relational_expression"),
            "a real relational expression must keep the runtime guard"
        );
    }
}
