//! PARSE-HARNESS.4 — the grammar-AST interpreter (approach 1), authoritative BY VERIFICATION.
//!
//! # What this is
//!
//! [`interpret_parse`] takes an **arbitrary** grammar (`grammars/foo.ebnf`) and an input string and
//! parses the input against that grammar **in-process, with no codegen and no compile** — by
//! *dynamically dispatching* over the normalized generation-input AST (the same `ASTNode` IR that
//! `--dump-gen-ast` emits and that codegen consumes) instead of running generated match-arms. It
//! returns the same [`ParseOutcome`](crate::parse_harness::ParseOutcome) the compile-and-run harness
//! ([`crate::parse_harness::compile_and_parse`], approach 2) returns — the accept/reject verdict, the
//! `furthest_position` on reject, and the **typed AST** on accept — so the two are directly diffable
//! (that diff is the `PARSE-HARNESS.5` differential-equivalence gate).
//!
//! # Why it is trustworthy — "authoritative by VERIFICATION" (not by construction)
//!
//! Unlike the scratch slot (approach 3) and the compile-and-run harness (approach 2) — both of which
//! *are* the shipped pipeline and so inherit its correctness — this interpreter is a genuine **second
//! parsing implementation**. Its trust is therefore *earned*, not assumed, by two design legs:
//!
//! 1. **A minimized trusted surface (the shared core).** Two code-mapping passes over the tree
//!    established exactly where the "thin dynamic-dispatch layer" boundary is (`PARSE-HARNESS.md` §13):
//!    a generated parser's *combinator control-flow is fully inlined per-rule as codegen `quote!`
//!    templates* — there is **no shared `impl`** for it — while the *semantic + type layer* survives as
//!    callable runtime. So this interpreter:
//!    - **reuses VERBATIM (Section A)** the shipped [`ParseNode`](crate::ast_pipeline::ParseNode) /
//!      [`ParseContent`](crate::ast_pipeline::ParseContent) types (so the typed AST it serializes is
//!      byte-identical), [`parse_quantifier_bounds`](crate::ast_pipeline::parse_quantifier_bounds),
//!      the [`SemanticRuntimeState`](crate::ast_pipeline::SemanticRuntimeState) checkpoint/rollback
//!      used by speculation, and the [`ASTNode`](crate::ast_pipeline::ASTNode) IR + the in-process
//!      gen-AST loader;
//!    - **re-expresses (Sections B/C)** only the small, parser-AGNOSTIC lexical/speculation primitives
//!      (`match_string`, `match_regex`, the layout consumers, `try_parse`) — mirrored byte-for-byte
//!      from the emitted code in a generated parser, the authoritative behavior — plus the combinator
//!      dispatch (the ordered-choice tournament + `branch_policy`, sequence assembly, the quantifier
//!      loop, lookahead) and the return-annotation fold.
//! 2. **Differential equivalence (the certifying oracle).** The residual — that thin dispatch layer —
//!    is checked against the authoritative generated parser: this module's tests assert the
//!    interpreter is **byte-identical** to the registered `json` parser (via
//!    [`crate::parser_registry::parse_sample_ast_json`]) and to the compile-and-run harness on
//!    synthetic per-combinator grammars. The full-corpus, all-registered-grammars gate is
//!    `PARSE-HARNESS.5`; this module is its subject and `compile_and_parse` is its oracle.
//!
//! # Honest bounds (`PARSE-HARNESS.md` §3.4 / §13.4)
//!
//! This is the interpreter **core**. It is byte-identical to the generated parser on the **structural
//! + return-annotation** surface (ordered choice under the default `longest_match` policy, sequence,
//! the quantifier forms, lookahead, terminals/regex-tokens/layout, rule references, and the full
//! return-annotation fold) plus the accept/reject **verdict** and `furthest_position` — proven on the
//! smoke set. Deferred to `PARSE-HARNESS.5`/`.6` (threaded here so they extend without restructuring,
//! but **not** in the `.4` smoke set): the semantic-directive orchestration that *gates parse outcomes*
//! on the store (`@predicate` branch/pre/post gates, `@emit_fact`/scope effects, the
//! `$reference`-against-content resolution, library import/export), the non-default `branch_policy` /
//! per-branch `@priority` / associativity paths beyond the default, and packrat memoization (a
//! transparent cache — AST-invariant — added in `.5` where the full corpus needs it). No over-claim of
//! a formal all-inputs proof.

use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use crate::ast_pipeline::ast_based_generator::{
    CommentArmSuppression, comment_arm_suppression_for_grammar,
};
use crate::ast_pipeline::semantic_directive_registry::{
    SemanticAssociativity, SemanticBranchPolicy, parse_semantic_branch_priorities,
};
use crate::ast_pipeline::unified_return_ast::{ExtractionTarget, UnifiedReturnAST};
use crate::ast_pipeline::{
    ASTNode, ASTValue, Annotations, BranchAnnotation, CompiledSemanticRuntimeAnnotations,
    ParseContent, ParseError, ParseNode, ParseResult, SemanticAnnotation, SemanticCloseScopeSpec,
    SemanticFactSpec, SemanticPredicateContentView, SemanticPredicatePhase, SemanticPredicateSpec,
    SemanticRuntimeDelta, SemanticRuntimeDirective, SemanticRuntimeState, SemanticRuntimeTransaction,
    SemanticRuntimeValue, SemanticScopeSpec, TokenValue, UnifiedSemanticAST, UnifiedSemanticProperty,
    UnifiedSemanticValue, compile_semantic_runtime_annotations, parse_canonical_transform_expression,
    parse_quantifier_bounds, parse_semantic_string_list,
};

pub use crate::parse_harness::ParseOutcome;

/// A structured failure of the interpreter's *setup* plumbing (loading / normalizing the grammar, or
/// an unknown entry rule). A *parse rejection* is NOT an error — it is a successful [`ParseOutcome`]
/// with `accepted == false`, exactly as for [`crate::parse_harness::compile_and_parse`].
#[derive(Debug)]
pub enum InterpretError {
    /// The grammar file does not exist / is not readable.
    GrammarNotFound(PathBuf),
    /// The EBNF frontend / normalization pipeline failed to produce a gen-AST for the grammar.
    Load(String),
    /// The requested entry rule is not a rule in the grammar.
    UnknownEntryRule(String),
    /// The grammar has no rules at all (empty `rule_order`).
    EmptyGrammar,
}

impl fmt::Display for InterpretError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InterpretError::GrammarNotFound(p) => {
                write!(f, "parse-harness interpreter: grammar file not found: {}", p.display())
            }
            InterpretError::Load(msg) => {
                write!(f, "parse-harness interpreter: failed to load/normalize grammar: {msg}")
            }
            InterpretError::UnknownEntryRule(r) => {
                write!(f, "parse-harness interpreter: entry rule '{r}' is not defined in the grammar")
            }
            InterpretError::EmptyGrammar => {
                write!(f, "parse-harness interpreter: grammar defines no rules")
            }
        }
    }
}

impl std::error::Error for InterpretError {}

/// Knobs for [`interpret_parse`]. All optional; [`Default`] mirrors the canonical entry + standard tree.
#[derive(Debug, Clone, Default)]
pub struct InterpretOptions {
    /// Parse from an alternate start symbol; `None` = the grammar's canonical entry (`rule_order[0]`,
    /// unless the loader recorded an explicit entry). Entry-rule-agnostic, exactly like the scratch
    /// slot and the compile-and-run harness.
    pub entry_rule: Option<String>,
    /// The ALREADY-NORMALIZED active dialect profile to gate `@profiles` rules against (PARSE-HARNESS.5.1)
    /// — e.g. `Some("pcre2")` for strict regex, `Some("sv_2017")` for SV. `None` (default) = all rules
    /// active. Normalize a requested profile with [`crate::parser_registry::active_grammar_profile`].
    pub profile: Option<String>,
}

/// Parse `input` against the arbitrary grammar at `grammar_ebnf`, returning the [`ParseOutcome`].
///
/// This is approach 1 of the PARSE-HARNESS tree — an in-process, no-codegen interpreter over the
/// normalized gen-AST, authoritative **by verification** (see the module docs). It reuses the exact
/// shipped `ParseNode`/`ParseContent` types, so `outcome.ast_json` is byte-for-byte what the shipped
/// parser produces.
///
/// # Errors
/// Returns [`InterpretError`] only for *setup* failures (missing grammar file, a normalization
/// failure, an unknown entry rule). A grammar that simply *rejects* the input is a successful
/// `Ok(ParseOutcome { accepted: false, .. })`.
///
/// Requires the `ebnf_dual_run` feature (the EBNF frontend that reads a `.ebnf` directly). To parse
/// from an already-normalized gen-AST without that feature, use [`interpret_parse_gen_ast`].
#[cfg(feature = "ebnf_dual_run")]
pub fn interpret_parse(
    grammar_ebnf: &Path,
    input: &str,
    opts: &InterpretOptions,
) -> Result<ParseOutcome, InterpretError> {
    crate::pgen_trace_low!(
        "parse-harness interpreter: interpret_parse grammar={} entry={:?} input_len={}",
        grammar_ebnf.display(),
        opts.entry_rule,
        input.len()
    );

    if !grammar_ebnf.is_file() {
        return Err(InterpretError::GrammarNotFound(grammar_ebnf.to_path_buf()));
    }
    let path_str = grammar_ebnf.to_string_lossy().into_owned();
    let envelope = crate::ebnf_frontend::parse_ebnf_file_to_raw_ast_envelope(&path_str)
        .map_err(|e| InterpretError::Load(format!("EBNF frontend: {e}")))?;
    let raw_ast = envelope
        .get("raw_ast")
        .and_then(|v| v.as_array())
        .ok_or_else(|| InterpretError::Load("EBNF envelope has no `raw_ast` array".to_string()))?;

    // Normalize exactly as codegen does: LR-elimination on (PipelineConfig default), annotations
    // preserved. The returned triple IS the in-memory gen-AST codegen consumes.
    let pipeline =
        crate::ast_pipeline::RustASTPipeline::new(crate::ast_pipeline::PipelineConfig::default());
    let (grammar_tree, rule_order, annotations) = pipeline
        .transform_from_raw_ast(raw_ast)
        .map_err(|e| InterpretError::Load(format!("normalization: {e}")))?;

    // The grammar name = the `.ebnf` file stem (what codegen normalizes for its layout-policy
    // decision, PARSE-HARNESS.5.1). Falls back to the empty string (⇒ the whitespace-insensitive
    // default policy) only for a nameless path, which never matches a whitespace-sensitive grammar.
    let grammar_name = grammar_ebnf
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    interpret_parse_gen_ast(
        grammar_name,
        opts.profile.as_deref(),
        &grammar_tree,
        &rule_order,
        annotations.as_ref(),
        opts.entry_rule.as_deref(),
        input,
    )
}

/// Parse `input` against an already-normalized gen-AST (the triple codegen consumes:
/// `grammar_tree` rule-name → body, `rule_order`, and `annotations`). This is the feature-independent
/// core [`interpret_parse`] calls after loading; it is exposed for gates/tests that already hold a
/// gen-AST (e.g. the `PARSE-HARNESS.5` differential gate).
///
/// `entry` selects the start symbol (`None` = `rule_order[0]`).
///
/// `grammar_name` is the grammar's `.ebnf` file stem (e.g. `regex`); it selects the [`LayoutPolicy`]
/// so the interpreter's terminal / regex-token / trailing-layout skipping is byte-identical to the
/// generated parser for whitespace-sensitive grammars (PARSE-HARNESS.5.1). A whitespace-insensitive
/// grammar (the common case) yields the all-`true` policy = the interpreter's prior behavior.
///
/// `active_profile` is the ALREADY-NORMALIZED dialect profile to gate `@profiles` rules against (e.g.
/// `Some("pcre2")` for default regex, `Some("sv_2017")` for SV, `None` = unprofiled → all rules active).
/// Normalize a requested profile via [`crate::parser_registry::active_grammar_profile`] before calling.
pub fn interpret_parse_gen_ast(
    grammar_name: &str,
    active_profile: Option<&str>,
    grammar_tree: &HashMap<String, ASTNode>,
    rule_order: &[String],
    annotations: Option<&Annotations>,
    entry: Option<&str>,
    input: &str,
) -> Result<ParseOutcome, InterpretError> {
    let entry_rule = match entry {
        Some(e) => {
            if !grammar_tree.contains_key(e) {
                return Err(InterpretError::UnknownEntryRule(e.to_string()));
            }
            e.to_string()
        }
        None => rule_order
            .first()
            .filter(|r| grammar_tree.contains_key(*r))
            .cloned()
            .or_else(|| grammar_tree.keys().next().cloned())
            .ok_or(InterpretError::EmptyGrammar)?,
    };

    // Compile the semantic-runtime registry EXACTLY as codegen does at generation time
    // (`compile_semantic_runtime_annotations` is the same function codegen calls at
    // `ast_based_generator.rs:6586` before freezing the literal into the generated parser). A grammar
    // whose annotations fail to compile cannot have a generated-parser oracle either, so surfacing the
    // compile error as a setup failure mirrors the codegen abort.
    let default_annotations = Annotations::default();
    let compiled_sem =
        compile_semantic_runtime_annotations(annotations.unwrap_or(&default_annotations))
            .map_err(|e| InterpretError::Load(format!("semantic-runtime compile: {e}")))?;

    // Mirror the generated constructor: the state starts empty with the compiled `@predicate_def`
    // registry installed (the generated parser does `set_predicate_defs(clone_predicate_defs())`).
    let mut semantic_state = SemanticRuntimeState::new();
    semantic_state.set_predicate_defs(compiled_sem.clone_predicate_defs());

    let mut interp = Interp {
        grammar: grammar_tree,
        annotations,
        layout: grammar_layout_policy(grammar_name),
        comment_arms: comment_arm_suppression_for_grammar(grammar_name, grammar_tree, annotations),
        active_profile: active_profile.map(|s| s.to_string()),
        input,
        position: 0,
        furthest_position: 0,
        depth: 0,
        max_depth: 2000,
        semantic_state,
        compiled_sem,
        memo: rustc_hash::FxHashMap::default(),
        memo_fail: rustc_hash::FxHashSet::default(),
        memo_fail_tainted: rustc_hash::FxHashMap::default(),
    };

    // Mirror `parse_full`: parse the entry rule, consume trailing layout, then require the whole input
    // was consumed. A prefix-only parse is a REJECT (accepted == false), not an error. The trailing
    // consume is gated by the layout policy — a whitespace-sensitive grammar (regex) treats trailing
    // whitespace as significant (`allow_trailing_layout`, ast_based_generator.rs:1144).
    let outcome = match interp.parse_rule(&entry_rule) {
        Ok(node) => {
            if interp.layout.allow_trailing_layout {
                interp.consume_layout_for_terminal("<EOF>");
            }
            if interp.position == interp.input.len() {
                let ast_json = serde_json::to_value(&node).map_err(|e| {
                    InterpretError::Load(format!("failed to serialize typed AST: {e}"))
                })?;
                ParseOutcome {
                    accepted: true,
                    furthest_position: interp.furthest_position,
                    error: None,
                    ast_json: Some(ast_json),
                }
            } else {
                ParseOutcome {
                    accepted: false,
                    furthest_position: interp.furthest_position,
                    error: Some(format!(
                        "Parser did not consume full input at position {}",
                        interp.position
                    )),
                    ast_json: None,
                }
            }
        }
        Err(e) => ParseOutcome {
            accepted: false,
            furthest_position: interp.furthest_position,
            error: Some(format!("{e:?}")),
            ast_json: None,
        },
    };
    Ok(outcome)
}

/// Intern a grammar-derived string to `&'static str`.
///
/// The shipped `ParseNode.rule_name` and `ParseContent::Quantified(_, label)` are `&'static str` —
/// compile-time literals in a generated parser. An interpreter over an *arbitrary* grammar has runtime
/// `String` rule-names / quantifier-labels / annotation-literals, so to reuse the exact `ParseNode`
/// type (and thus produce a byte-identical serialized AST) it must hand out `&'static str`. The set of
/// such strings is *finite per grammar* (rule names, the handful of quantifier labels, the annotation
/// literals), so a process-global dedup+leak is bounded: each distinct string is leaked at most once,
/// ever. This is the standard "intern to `'static`" pattern; for a probe/gate tool the bounded,
/// one-time-per-string leak is deliberate and acceptable.
fn intern(s: &str) -> &'static str {
    static INTERN: OnceLock<Mutex<HashMap<String, &'static str>>> = OnceLock::new();
    let map = INTERN.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = map.lock().expect("intern mutex poisoned");
    if let Some(&existing) = guard.get(s) {
        return existing;
    }
    let leaked: &'static str = Box::leak(s.to_string().into_boxed_str());
    guard.insert(s.to_string(), leaked);
    leaked
}

/// A grammar's layout/whitespace-skipping policy — whether the parser skips leading layout
/// (whitespace + comments) before matching terminals / regex-tokens, and trailing layout at the end
/// of a full parse. Most grammars are whitespace-INSENSITIVE (all three `true`, layout is skipped),
/// but a few are whitespace-SENSITIVE where layout is literal input.
///
/// # PARSE-HARNESS.5.1 — why this exists (the regex fidelity fix)
///
/// The interpreter must reproduce the generated parser **byte-for-byte**, and the shipped codegen makes
/// this a per-grammar, grammar-NAME-keyed decision (`ast_based_generator.rs`): `regex` is
/// whitespace-sensitive so its terminals, regex-tokens, and trailing `<EOF>` do NOT skip layout;
/// `systemverilog_preprocessor` disables the regex-token layout skip. Without this, on a regex input
/// like `\Q]\E* ?` the interpreter would skip the literal space and bind the `?` as a lazy
/// `quant_suffix` (`greediness:"lazy"`) where the generated regex parser leaves the space in place and
/// the suffix empty (`greediness:[]`) — the exact divergence PARSE-HARNESS.5's measurement recorded.
///
/// [`grammar_layout_policy`] mirrors, expression-for-expression, the three codegen decisions:
/// - `skip_layout_for_terminals` ⟷ `allow_layout_skip_for_terminals` (`ast_based_generator.rs:4509`),
/// - `skip_layout_for_regexes`   ⟷ `allow_layout_skip_for_regexes`   (`ast_based_generator.rs:4510`),
/// - `allow_trailing_layout`     ⟷ `allow_trailing_layout`           (`ast_based_generator.rs:1144`).
///
/// The differential-equivalence gate (PARSE-HARNESS.5) is the runtime drift guard: once `regex` is
/// CERTIFIED byte-identical, any future codegen change to its whitespace handling that is not mirrored
/// here fails the gate (the un-fakeable oracle leg per `DOCTRINE_ENFORCEMENT.md` §6.1).
#[derive(Debug, Clone, Copy)]
struct LayoutPolicy {
    /// Skip leading layout before a string terminal (`match_string`). `false` ⇒ terminals are
    /// whitespace-sensitive (regex).
    skip_layout_for_terminals: bool,
    /// Skip leading layout before a regex-token (`match_regex`). `false` ⇒ regex-tokens are
    /// whitespace-sensitive (regex, systemverilog_preprocessor).
    skip_layout_for_regexes: bool,
    /// Consume trailing layout after the entry rule before the end-of-input check. `false` ⇒ trailing
    /// whitespace is significant (regex).
    allow_trailing_layout: bool,
}

/// Compute a grammar's [`LayoutPolicy`] from its name, mirroring the shipped codegen's grammar-name
/// keyed decisions **verbatim** (see [`LayoutPolicy`] for the crux + the exact `ast_based_generator.rs`
/// line references). `grammar_name` is the grammar's `.ebnf` file stem (e.g. `regex`), exactly the
/// value codegen normalizes.
fn grammar_layout_policy(grammar_name: &str) -> LayoutPolicy {
    // Codegen's `normalized_grammar_name`: keep only ASCII alphanumerics, lowercase
    // (ast_based_generator.rs:4503-4508).
    let normalized: String = grammar_name
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase();
    LayoutPolicy {
        skip_layout_for_terminals: normalized != "regex",
        skip_layout_for_regexes: !matches!(normalized.as_str(), "regex" | "systemverilogpreprocessor"),
        // Codegen uses the RAW name here (`eq_ignore_ascii_case`, ast_based_generator.rs:1144).
        allow_trailing_layout: !grammar_name.eq_ignore_ascii_case("regex"),
    }
}

/// Is `annotation` the rule's `@transform` directive? Mirrors codegen's `semantic_directive_name`
/// (`ast_based_generator.rs:6514/6955`): a named directive matches on its normalized name; an unnamed
/// `TransformExpr` defaults to the `transform` directive. The canonical-parse guard at the call site is
/// the real filter, so this only needs to exclude a differently-named directive.
fn annotation_is_transform_directive(annotation: &SemanticAnnotation) -> bool {
    if let Some(name) = annotation.name() {
        let normalized = name.trim().to_ascii_lowercase();
        if !normalized.is_empty() {
            return normalized == "transform";
        }
    }
    matches!(annotation.ast(), UnifiedSemanticAST::TransformExpr { .. })
}

/// Mirror the generated `span_text.parse::<T>().unwrap_or(D).to_string()` for the canonical `@transform`
/// target types (`ast_based_generator.rs:4116-4119`). On parse failure the default expression is used,
/// parsed as the same type (matching `unwrap_or(D)`); a non-numeric target (`String`/`str`/unknown) is
/// the identity, since codegen's `parse::<String>()` is infallible and yields the text itself.
fn numeric_span_transform(target_type: &str, default_expr: &str, text: &str) -> String {
    fn parse_or_default<T>(text: &str, default_expr: &str) -> String
    where
        T: std::str::FromStr + ToString,
    {
        text.parse::<T>()
            .or_else(|_| default_expr.parse::<T>())
            .map(|v| v.to_string())
            .unwrap_or_else(|_| default_expr.trim().to_string())
    }
    // Codegen interpolates the type verbatim into `parse::<#target_type>()`; take the leaf so a
    // path-qualified type (`std::primitive::usize`) still dispatches.
    let leaf = target_type.rsplit("::").next().unwrap_or(target_type).trim();
    match leaf {
        "usize" => parse_or_default::<usize>(text, default_expr),
        "u8" => parse_or_default::<u8>(text, default_expr),
        "u16" => parse_or_default::<u16>(text, default_expr),
        "u32" => parse_or_default::<u32>(text, default_expr),
        "u64" => parse_or_default::<u64>(text, default_expr),
        "u128" => parse_or_default::<u128>(text, default_expr),
        "isize" => parse_or_default::<isize>(text, default_expr),
        "i8" => parse_or_default::<i8>(text, default_expr),
        "i16" => parse_or_default::<i16>(text, default_expr),
        "i32" => parse_or_default::<i32>(text, default_expr),
        "i64" => parse_or_default::<i64>(text, default_expr),
        "i128" => parse_or_default::<i128>(text, default_expr),
        "f32" => parse_or_default::<f32>(text, default_expr),
        "f64" => parse_or_default::<f64>(text, default_expr),
        "bool" => parse_or_default::<bool>(text, default_expr),
        _ => text.to_string(),
    }
}

/// The dynamic-dispatch interpreter state. `'g` = the borrowed gen-AST; `'i` = the input string.
struct Interp<'g, 'i> {
    grammar: &'g HashMap<String, ASTNode>,
    annotations: Option<&'g Annotations>,
    /// The grammar's layout/whitespace-skipping policy (PARSE-HARNESS.5.1) — consulted by
    /// `match_string`, `match_regex`, and the trailing-layout consume so the interpreter is
    /// byte-identical to the generated parser for whitespace-sensitive grammars (regex).
    layout: LayoutPolicy,
    /// The grammar's per-introducer comment-arm suppression (PARSE-HARNESS.5.2) — which of the
    /// `#` / `//` / `/*` comment introducers the two layout skippers (`consume_layout_for_terminal` /
    /// `consume_layout_for_regex`) may skip as trivia. Computed via codegen's OWN predicate
    /// (`comment_arm_suppression_for_grammar`), so the interpreter skips comment layout byte-identically
    /// to the generated parser: a grammar that CLAIMS an introducer as a real token (e.g. ebnf's
    /// `block_comment := "/*" …`) has that arm SUPPRESSED, so those bytes must be matched structurally.
    comment_arms: CommentArmSuppression,
    /// The active (already-normalized) dialect profile (`Some("pcre2")` for default regex,
    /// `Some("sv_2017")` for SV, `None` = unprofiled). A rule annotated `@profiles` is excluded when the
    /// active profile is not among its allowed profiles (PARSE-HARNESS.5.1) — mirrors codegen's
    /// `rule_profile_is_enabled`. `None` ⇒ all rules active (matches codegen's `None` case).
    active_profile: Option<String>,
    input: &'i str,
    position: usize,
    /// The deepest input byte any branch reached (even after backtracking) — updated monotonically at
    /// each rule entry, exactly as the generated parser does. This is the A2.2/A2.3-grade reject locus.
    furthest_position: usize,
    depth: usize,
    max_depth: usize,
    /// Reused VERBATIM (Section A): speculation snapshots/restores it, faithfully. Populated by the
    /// `.6.2` semantic-directive orchestration mirror (`with_rule_transaction` + the branch machinery);
    /// for a grammar with no semantic directives it stays empty, so it never perturbs the typed AST.
    semantic_state: SemanticRuntimeState,
    /// The COMPILED semantic-runtime registry (Section A, reused verbatim): the same
    /// `compile_semantic_runtime_annotations` output codegen freezes into a generated parser
    /// (`ast_based_generator.rs:6586`), compiled here in-process from the identical gen-AST
    /// `Annotations`. Drives the `.6.2` orchestration mirror: pre/branch/post predicates, effect
    /// directives, branch-start inline actions, library phases, and `needs_raw_post_capture_for_rule`.
    compiled_sem: CompiledSemanticRuntimeAnnotations,
    /// The split packrat memo, mirrored from the generated `memoized_call`
    /// (`ast_based_generator.rs:6440`, PARSE-TERMINATION.6): successes carry the BODY's node + the
    /// captured raw content + the semantic delta (replayed on hit); failures live in the lean
    /// `memo_fail` set keyed `(rule, position)` ONLY. The rule transaction WRAPS the memo, so a rule's
    /// own gates/effects are never cached; store-TAINTED bodies (transitively evaluated ≥1 predicate)
    /// are EPOCH-STAMPED and replayable only while the store write epoch is unchanged
    /// (MEMO-STORE-SOUNDNESS.2 taint gate — the `sem_memo_wrapper` / `sem_memo_success_*` sound pins),
    /// so a stale store-dependent entry can never replay.
    memo: rustc_hash::FxHashMap<(&'static str, usize), InterpMemoEntry<'i>>,
    /// The failure half of the split memo (see `memo`).
    memo_fail: rustc_hash::FxHashSet<(&'static str, usize)>,
    /// MEMO-STORE-SOUNDNESS.2 — store-tainted failures, epoch-stamped (mirror of the generated
    /// `memo_fail_tainted`).
    memo_fail_tainted: rustc_hash::FxHashMap<(&'static str, usize), u64>,
}

/// One success entry of the interpreter's split memo — the mirror of the generated `MemoEntry` minus
/// the coverage lane (the interpreter has no coverage recording; that is a registry/cert surface).
#[derive(Clone)]
struct InterpMemoEntry<'i> {
    node: ParseNode<'i>,
    raw_semantic_content: Option<ParseContent<'i>>,
    end_pos: usize,
    semantic_delta: Option<SemanticRuntimeDelta>,
    /// MEMO-STORE-SOUNDNESS.2 — `Some(write_epoch_at_insert)` for a store-tainted body (mirror of
    /// the generated `MemoEntry::tainted_at_epoch`); validated on every hit, evicted when stale.
    tainted_at_epoch: Option<u64>,
}

impl<'g, 'i> Interp<'g, 'i> {
    // ── Rule dispatch ────────────────────────────────────────────────────────────────────────────

    /// Parse a named rule, producing its `ParseNode` (`rule_name`, the shaped `content`, and the
    /// `span`). Mirrors a generated `parse_<rule>` method: bump `furthest_position` at entry, run the
    /// body (with its return-annotation transform), wrap in a `ParseNode`.
    fn parse_rule(&mut self, rule_name: &str) -> ParseResult<ParseNode<'i>> {
        self.depth += 1;
        if self.depth > self.max_depth {
            self.depth -= 1;
            return Err(ParseError::RecursionDepthExceeded {
                position: self.position,
                depth: self.depth,
            });
        }
        let result = self.parse_rule_inner(rule_name);
        self.depth -= 1;
        result
    }

    fn parse_rule_inner(&mut self, rule_name: &str) -> ParseResult<ParseNode<'i>> {
        let start_pos = self.position;

        // A rule REFERENCED but not DEFINED in the grammar is not an error: codegen synthesizes a
        // native method for a handful of built-in names (`generate_unresolved_reference_method`), so the
        // interpreter mirrors that dispatch instead of hard-erroring (PARSE-HARNESS.5.1). The regex
        // grammar reaches this via `unicode_char = !builtin_ascii_char builtin_any_char`.
        // Dispatched BEFORE the furthest bump / profile gate / transaction: the generated
        // unresolved-reference methods are bare stubs with NO rule preamble at all (no
        // `furthest_position` bump, no memo, no rule context — tool-verified from the emitted
        // `parse_word` stub and `parse_builtin_any_char`, PARSE-HARNESS.6.2).
        let Some(body) = self.grammar.get(rule_name) else {
            return self.parse_unresolved_reference(rule_name, start_pos);
        };

        // furthest_position is updated at rule entry, monotonically (never decremented on backtrack).
        if self.position > self.furthest_position {
            self.furthest_position = self.position;
        }

        // Profile gate: a rule annotated `@profiles` is EXCLUDED (Backtracks) when the active dialect
        // profile is not among its allowed profiles — mirrors codegen's rule-entry `profile_guard`
        // (ast_based_generator.rs:2692-2704) + `rule_profile_is_enabled` (:4903). Empty (ungated) rule
        // or `None` active profile ⇒ always enabled. This is what makes the interpreter reject a
        // relaxed-only construct (e.g. `directive_name_relaxed`) under strict `pcre2` (PARSE-HARNESS.5.1).
        if self.active_profile.is_some() {
            let profiles = self.rule_profiles(rule_name);
            if !profiles.is_empty() && !self.profile_enabled(&profiles) {
                return Err(ParseError::Backtrack { position: start_pos });
            }
        }

        // The generated rule skeleton (tool-verified from the emitted parsers, PARSE-HARNESS.6.2):
        // `with_semantic_runtime_rule_transaction(rule, |p| p.memoized_call(rule_id, body))` — the
        // TRANSACTION (pre gates → body → effects → imports → post gates → exports → commit) WRAPS the
        // memo, so a rule's own gates/effects are re-evaluated fresh on every memo hit; the memo caches
        // only the BODY's `(node, raw, semantic_delta)`.
        let interned_rule = intern(rule_name);
        self.with_rule_transaction(interned_rule, |s| {
            s.memoized_call(interned_rule, |s| {
                let capture_raw = s.compiled_sem.needs_raw_post_capture_for_rule(interned_rule);
                let mut semantic_raw_content: Option<ParseContent<'i>> = None;
                let content = s.parse_rule_body(
                    body,
                    rule_name,
                    start_pos,
                    capture_raw,
                    &mut semantic_raw_content,
                )?;
                // A rule-level `@transform` applied to the matched SPAN text, for a body that is NOT a
                // single terminal (e.g. `digits = digit+` → `usize`). Codegen splices this AFTER the
                // return-annotation transform and ONLY for a non-`Or` body
                // (`generate_post_body_span_transform`, ast_based_generator.rs:2606-2609/4094).
                let content = if matches!(body, ASTNode::Or { .. }) {
                    content
                } else {
                    s.apply_post_body_span_transform(rule_name, content, start_pos)
                };
                let end_pos = s.position;
                Ok((
                    ParseNode {
                        rule_name: interned_rule,
                        content,
                        span: start_pos..end_pos,
                    },
                    semantic_raw_content,
                ))
            })
        })
    }

    // ── The semantic-directive orchestration mirror (PARSE-HARNESS.6.2) ────────────────────────────
    //
    // Everything below `── ` here mirrors the generated orchestration templates byte-for-byte from the
    // EMITTED code (the authoritative behavior): `with_semantic_runtime_rule_transaction`
    // (`ast_based_generator.rs:1661`), `memoized_call` (`:6440`), the effect/branch-start appliers
    // (`:2010` / `:1225`), the library phases, and the `$reference` resolver family (`:2116` / `:5534`).
    // The queries themselves (predicate evaluation, transactions, checkpoints, deltas) are the SHARED
    // runtime (`semantic_runtime.rs`), reused verbatim — the mirror is only the orchestration glue that
    // exists solely as codegen `quote!` templates.

    /// Mirror of the generated `with_semantic_runtime_rule_transaction`: the rule-level semantic
    /// orchestration skeleton. Fast path for a rule with no compiled directives (push/pop rule context
    /// only); else: checkpoint → PRE predicates → body → effect directives → library imports → POST
    /// predicates → library exports → commit, with a named rollback on any failure and exactly one
    /// rule-context pop per push.
    fn with_rule_transaction(
        &mut self,
        rule_name: &'static str,
        f: impl FnOnce(&mut Self) -> ParseResult<(ParseNode<'i>, Option<ParseContent<'i>>)>,
    ) -> ParseResult<ParseNode<'i>> {
        if self.compiled_sem.is_empty() || !self.compiled_sem.has_rule(rule_name) {
            self.semantic_state.push_rule_context(rule_name);
            let result = f(self);
            self.semantic_state.pop_rule_context();
            let (node, _raw) = result?;
            return Ok(node);
        }
        self.semantic_state.push_rule_context(rule_name);
        let semantic_checkpoint = self.semantic_state.checkpoint();
        let result: ParseResult<ParseNode<'i>> = (|s: &mut Self| -> ParseResult<ParseNode<'i>> {
            // PRE predicates — evaluated content-free (`evaluate_directive_predicate`), before the body.
            let mut predicate_blocked = false;
            for directive in s.compiled_sem.pre_predicates_for_rule(rule_name) {
                match s.semantic_state.evaluate_directive_predicate(directive) {
                    Some(true) => {}
                    Some(false) => {
                        predicate_blocked = true;
                        break;
                    }
                    None => {}
                }
            }
            if predicate_blocked {
                return Err(ParseError::Backtrack {
                    position: s.position,
                });
            }
            let entry_fact_len: usize = semantic_checkpoint.fact_len();
            let (node, semantic_raw_content) = f(s)?;
            let semantic_raw_content = semantic_raw_content.as_ref().unwrap_or(&node.content);
            // The generated code `mem::take`s the state so the transaction can borrow it mutably while
            // the `&self` resolvers stay callable; restored unconditionally below.
            let mut semantic_state = std::mem::take(&mut s.semantic_state);
            let semantic_txn_result: ParseResult<()> = (|| -> ParseResult<()> {
                let mut txn = semantic_state.transaction_named(rule_name);
                for directive in s.compiled_sem.effect_directives_for_rule(rule_name) {
                    let _ = s.apply_effect_directive(&mut txn, directive, &node.content)?;
                }
                for directive in s.compiled_sem.library_imports_for_rule(rule_name) {
                    if let SemanticRuntimeDirective::ImportFromLibrary(spec) = directive {
                        s.apply_library_import_directive(&mut txn, spec, &node.content)?;
                    }
                }
                // POST predicates — resolved against the (raw, shaped) content pair, evaluated on the
                // transaction's state (so this rule's own effects above are visible to its gates).
                let mut post_predicate_blocked = false;
                for directive in s.compiled_sem.post_predicates_for_rule(rule_name) {
                    match directive {
                        SemanticRuntimeDirective::Predicate(spec)
                            if spec.phase == SemanticPredicatePhase::Post =>
                        {
                            let resolved_spec = s.resolve_predicate_spec_against_content(
                                spec,
                                semantic_raw_content,
                                &node.content,
                            )?;
                            match txn.state().evaluate_content_aware_predicate(
                                &resolved_spec,
                                semantic_raw_content,
                                &node.content,
                            ) {
                                Some(true) => {}
                                Some(false) => {
                                    post_predicate_blocked = true;
                                    break;
                                }
                                None => {}
                            }
                        }
                        _ => {}
                    }
                }
                if post_predicate_blocked {
                    return Err(ParseError::Backtrack {
                        position: node.span.start,
                    });
                }
                for directive in s.compiled_sem.library_exports_for_rule(rule_name) {
                    if let SemanticRuntimeDirective::ExportToLibrary(_spec) = directive {
                        s.apply_library_export_directive(&txn, &node.content, entry_fact_len)?;
                    }
                }
                let _ = txn.commit();
                Ok(())
            })();
            s.semantic_state = semantic_state;
            semantic_txn_result?;
            Ok(node)
        })(self);
        if result.is_err() {
            self.semantic_state
                .rollback_to_named(semantic_checkpoint, Some(rule_name));
        }
        self.semantic_state.pop_rule_context();
        result
    }

    /// Mirror of the generated split-memo `memoized_call` (PARSE-TERMINATION.6 +
    /// `SV-EXH-PROOF.3.3.4.b.6.2.36.4` + MEMO-STORE-SOUNDNESS.2): failures are cached in a lean
    /// position-keyed set; successes replay the cached node + raw content + semantic delta. Keyed on
    /// the CALL position (the rule's entry position — pre predicates are zero-width). The memo key is
    /// store-BLIND, so a STORE-TAINTED outcome (the body transitively evaluated ≥1 predicate) is
    /// EPOCH-STAMPED at insert and replayable only while the store write epoch is unchanged —
    /// mirroring the generated template's taint gate exactly (the `sem_memo_wrapper` /
    /// `sem_memo_success_*` sound re-anchors; outright taint-EXCLUSION measured 117× slower on SV).
    fn memoized_call(
        &mut self,
        rule_name: &'static str,
        f: impl FnOnce(&mut Self) -> ParseResult<(ParseNode<'i>, Option<ParseContent<'i>>)>,
    ) -> ParseResult<(ParseNode<'i>, Option<ParseContent<'i>>)> {
        let key = (rule_name, self.position);
        if self.memo_fail.contains(&key) {
            return Err(ParseError::Backtrack { position: key.1 });
        }
        // MEMO-STORE-SOUNDNESS.2 — tainted-failure validation: replay only while the store write
        // epoch is unchanged since insert; evict + re-parse once the store has moved.
        if let Some(&epoch) = self.memo_fail_tainted.get(&key) {
            if epoch == self.semantic_state.write_epoch() {
                return Err(ParseError::Backtrack { position: key.1 });
            }
            self.memo_fail_tainted.remove(&key);
        }
        // MEMO-STORE-SOUNDNESS.2 — tainted-success validation (mirror of the generated template).
        let stale_tainted_success = matches!(
            self.memo.get(&key),
            Some(entry) if entry.tainted_at_epoch.is_some_and(|epoch| epoch != self.semantic_state.write_epoch())
        );
        if stale_tainted_success {
            self.memo.remove(&key);
        }
        if let Some(entry) = self.memo.get(&key) {
            self.position = entry.end_pos;
            let node = entry.node.clone();
            let raw = entry.raw_semantic_content.clone();
            if let Some(delta) = entry.semantic_delta.clone() {
                if !delta.is_empty() {
                    self.semantic_state.apply_delta(delta);
                }
            }
            return Ok((node, raw));
        }
        let memo_entry_checkpoint = self.semantic_state.checkpoint();
        let memo_taint_snapshot = self.semantic_state.predicate_evaluations();
        let result = f(self);
        let memo_store_tainted =
            self.semantic_state.predicate_evaluations() != memo_taint_snapshot;
        match &result {
            Ok((node, raw_semantic_content)) => {
                let semantic_delta = self
                    .semantic_state
                    .extract_delta_since(&memo_entry_checkpoint);
                self.memo.insert(
                    key,
                    InterpMemoEntry {
                        node: node.clone(),
                        raw_semantic_content: raw_semantic_content.clone(),
                        end_pos: node.span.end,
                        semantic_delta: Some(semantic_delta),
                        tainted_at_epoch: if memo_store_tainted {
                            Some(self.semantic_state.write_epoch())
                        } else {
                            None
                        },
                    },
                );
            }
            Err(_) if memo_store_tainted => {
                self.memo_fail_tainted
                    .insert(key, self.semantic_state.write_epoch());
            }
            Err(_) => {
                self.memo_fail.insert(key);
            }
        }
        result
    }

    /// Mirror of the generated `apply_semantic_runtime_effect_directive`: apply one
    /// `@open_scope`/`@close_scope`/`@emit_fact` onto the rule transaction, resolving `$ref`s against
    /// the rule's (shaped) content. Predicates / library / declaration directives are handled by their
    /// own phases and return `Ok(false)` here.
    fn apply_effect_directive(
        &self,
        transaction: &mut SemanticRuntimeTransaction<'_>,
        directive: &SemanticRuntimeDirective,
        root_content: &ParseContent<'i>,
    ) -> ParseResult<bool> {
        match directive {
            SemanticRuntimeDirective::Predicate(_) => Ok(false),
            SemanticRuntimeDirective::ExportToLibrary(_)
            | SemanticRuntimeDirective::ImportFromLibrary(_) => Ok(false),
            SemanticRuntimeDirective::DeclareFactKind(_)
            | SemanticRuntimeDirective::DefinePredicate(_) => Ok(false),
            SemanticRuntimeDirective::OpenScope(spec) => {
                let resolved_name = spec
                    .name
                    .as_ref()
                    .map(|value| {
                        self.resolve_semantic_runtime_value_against_content(value, root_content)
                            .ok_or_else(|| {
                                self.create_contextual_error(
                                    "Semantic runtime could not resolve scope name for directive in current parse result",
                                )
                            })
                    })
                    .transpose()?;
                Ok(transaction.apply_directive(&SemanticRuntimeDirective::OpenScope(
                    SemanticScopeSpec {
                        kind: spec.kind.clone(),
                        name: resolved_name,
                    },
                )))
            }
            SemanticRuntimeDirective::CloseScope(spec) => {
                let resolved_name = spec
                    .name
                    .as_ref()
                    .map(|value| {
                        self.resolve_semantic_runtime_value_against_content(value, root_content)
                            .ok_or_else(|| {
                                self.create_contextual_error(
                                    "Semantic runtime could not resolve close-scope name for directive in current parse result",
                                )
                            })
                    })
                    .transpose()?;
                Ok(transaction.apply_directive(&SemanticRuntimeDirective::CloseScope(
                    SemanticCloseScopeSpec {
                        kind: spec.kind.clone(),
                        name: resolved_name,
                    },
                )))
            }
            SemanticRuntimeDirective::EmitFact(spec) => {
                let resolved_name = self
                    .resolve_semantic_runtime_value_against_content(&spec.name, root_content)
                    .ok_or_else(|| {
                        self.create_contextual_error(
                            "Semantic runtime could not resolve fact name for directive in current parse result",
                        )
                    })?;
                let resolved_attributes = self
                    .resolve_unified_semantic_properties_against_content(
                        &spec.attributes,
                        root_content,
                    )?;
                Ok(transaction.apply_directive(&SemanticRuntimeDirective::EmitFact(
                    SemanticFactSpec {
                        kind: spec.kind.clone(),
                        name: resolved_name,
                        attributes: resolved_attributes,
                    },
                )))
            }
        }
    }

    /// Mirror of the generated `apply_branch_start_effect_directive` (INLINE-ACTIONS.2): apply a
    /// WINNING branch's branch-start inline ACTION directly onto the live semantic state (no
    /// transaction wrapper — the enclosing rule transaction and the tournament checkpoint own
    /// rollback), resolving `$ref`s against the selected branch's content.
    fn apply_branch_start_effect_directive(
        &mut self,
        directive: &SemanticRuntimeDirective,
        root_content: &ParseContent<'i>,
    ) -> ParseResult<bool> {
        let resolved: Option<SemanticRuntimeDirective> = match directive {
            SemanticRuntimeDirective::EmitFact(spec) => {
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
                    )?;
                Some(SemanticRuntimeDirective::EmitFact(SemanticFactSpec {
                    kind: spec.kind.clone(),
                    name: resolved_name,
                    attributes: resolved_attributes,
                }))
            }
            SemanticRuntimeDirective::OpenScope(spec) => {
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
                Some(SemanticRuntimeDirective::OpenScope(SemanticScopeSpec {
                    kind: spec.kind.clone(),
                    name: resolved_name,
                }))
            }
            SemanticRuntimeDirective::CloseScope(spec) => {
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
                Some(SemanticRuntimeDirective::CloseScope(SemanticCloseScopeSpec {
                    kind: spec.kind.clone(),
                    name: resolved_name,
                }))
            }
            _ => None,
        };
        match resolved {
            Some(resolved) => Ok(self.semantic_state.apply_directive(&resolved)),
            None => Ok(false),
        }
    }

    /// Mirror of the generated `apply_semantic_runtime_library_import_directive`. The interpreter's
    /// harness API has NO library configuration (`library_in_dir` is always unset — exactly like the
    /// compile-and-run oracle's throwaway main), so the generated early-return-on-`None` path is the
    /// whole behavior: a no-op BEFORE any `name_from` resolution (PARSE-HARNESS.md §21.3 honest bound;
    /// the real-I/O lane is registry-owned and proven by the SV gates).
    fn apply_library_import_directive(
        &self,
        _transaction: &mut SemanticRuntimeTransaction<'_>,
        _spec: &crate::ast_pipeline::SemanticLibraryImportSpec,
        _root_content: &ParseContent<'i>,
    ) -> ParseResult<()> {
        // Mirrors: `let Some(lib_in) = self.library_in_dir.as_deref() else { return Ok(()); };`
        Ok(())
    }

    /// Mirror of the generated `apply_semantic_runtime_library_export_directive` — the same
    /// no-library-configured no-op as the import side (early return before resolution).
    fn apply_library_export_directive(
        &self,
        _transaction: &SemanticRuntimeTransaction<'_>,
        _root_content: &ParseContent<'i>,
        _entry_fact_len: usize,
    ) -> ParseResult<()> {
        // Mirrors: `let Some(lib_out) = self.library_out_dir.as_deref() else { return Ok(()); };`
        Ok(())
    }

    /// Mirror of the generated `create_contextual_error`. Only the failure CLASS matters to the
    /// harness differential (verdict + `furthest_position` + typed AST are compared; the error string
    /// is not), so the rule stack / input context are left empty.
    fn create_contextual_error(&self, message: &str) -> ParseError {
        ParseError::ContextualError {
            message: message.to_string(),
            position: self.position,
            rule_stack: Vec::new(),
            input_context: String::new(),
        }
    }

    // ── The `$reference` resolver family (mirrored from the emitted templates, `:2116`/`:5534`) ─────

    /// Mirror of the generated `resolve_semantic_runtime_value_against_content`: resolve one
    /// `SemanticRuntimeValue` (a directive's `name`/`name_from` payload) against the rule content —
    /// `$ref`s go through `resolve_semantic_reference` + the scalar coercion; literals pass through.
    fn resolve_semantic_runtime_value_against_content(
        &self,
        value: &SemanticRuntimeValue,
        root_content: &ParseContent<'i>,
    ) -> Option<SemanticRuntimeValue> {
        match value {
            SemanticRuntimeValue::RuleReference(reference) => self
                .resolve_semantic_reference(root_content, reference)
                .map(|resolved| self.coerce_semantic_runtime_scalar(&resolved)),
            SemanticRuntimeValue::String(text) => Some(SemanticRuntimeValue::String(text.clone())),
            SemanticRuntimeValue::Identifier(text) => {
                Some(SemanticRuntimeValue::Identifier(text.clone()))
            }
            SemanticRuntimeValue::Number(text) => Some(SemanticRuntimeValue::Number(text.clone())),
            SemanticRuntimeValue::Boolean(value) => Some(SemanticRuntimeValue::Boolean(*value)),
            SemanticRuntimeValue::Null => Some(SemanticRuntimeValue::Null),
        }
    }

    /// Mirror of the generated `resolve_unified_semantic_value_against_content` (the HARD-resolving
    /// variant: an unresolvable `$ref` is a `ContextualError`).
    fn resolve_unified_semantic_value_against_content(
        &self,
        value: &UnifiedSemanticValue,
        root_content: &ParseContent<'i>,
    ) -> ParseResult<UnifiedSemanticValue> {
        match value {
            UnifiedSemanticValue::RuleReference(reference) => self
                .resolve_semantic_reference(root_content, reference)
                .map(|resolved| self.coerce_unified_semantic_scalar(&resolved))
                .ok_or_else(|| {
                    self.create_contextual_error(&format!(
                        "Semantic runtime could not resolve attribute reference '{reference}'"
                    ))
                }),
            UnifiedSemanticValue::String(text) => Ok(UnifiedSemanticValue::String(text.clone())),
            UnifiedSemanticValue::Identifier(text) => {
                Ok(UnifiedSemanticValue::Identifier(text.clone()))
            }
            UnifiedSemanticValue::Number(text) => Ok(UnifiedSemanticValue::Number(text.clone())),
            UnifiedSemanticValue::Boolean(value) => Ok(UnifiedSemanticValue::Boolean(*value)),
            UnifiedSemanticValue::Null => Ok(UnifiedSemanticValue::Null),
            UnifiedSemanticValue::Array(elements) => {
                let mut resolved = Vec::with_capacity(elements.len());
                for element in elements {
                    resolved
                        .push(self.resolve_unified_semantic_value_against_content(element, root_content)?);
                }
                Ok(UnifiedSemanticValue::Array(resolved))
            }
            UnifiedSemanticValue::Object(properties) => Ok(UnifiedSemanticValue::Object(
                self.resolve_unified_semantic_properties_against_content(properties, root_content)?,
            )),
        }
    }

    /// Mirror of the generated `try_resolve_unified_semantic_value_against_content` (the SOFT variant:
    /// an unresolvable `$ref` yields `Ok(None)` — a BRANCH predicate with an unresolvable arg blocks
    /// that branch instead of erroring).
    fn try_resolve_unified_semantic_value_against_content(
        &self,
        value: &UnifiedSemanticValue,
        root_content: &ParseContent<'i>,
    ) -> ParseResult<Option<UnifiedSemanticValue>> {
        match value {
            UnifiedSemanticValue::RuleReference(reference) => Ok(self
                .resolve_semantic_reference(root_content, reference)
                .map(|resolved| self.coerce_unified_semantic_scalar(&resolved))),
            UnifiedSemanticValue::String(text) => {
                Ok(Some(UnifiedSemanticValue::String(text.clone())))
            }
            UnifiedSemanticValue::Identifier(text) => {
                Ok(Some(UnifiedSemanticValue::Identifier(text.clone())))
            }
            UnifiedSemanticValue::Number(text) => {
                Ok(Some(UnifiedSemanticValue::Number(text.clone())))
            }
            UnifiedSemanticValue::Boolean(value) => Ok(Some(UnifiedSemanticValue::Boolean(*value))),
            UnifiedSemanticValue::Null => Ok(Some(UnifiedSemanticValue::Null)),
            UnifiedSemanticValue::Array(elements) => {
                let mut resolved = Vec::with_capacity(elements.len());
                for element in elements {
                    let Some(resolved_element) =
                        self.try_resolve_unified_semantic_value_against_content(element, root_content)?
                    else {
                        return Ok(None);
                    };
                    resolved.push(resolved_element);
                }
                Ok(Some(UnifiedSemanticValue::Array(resolved)))
            }
            UnifiedSemanticValue::Object(properties) => {
                let mut resolved = Vec::with_capacity(properties.len());
                for property in properties {
                    let Some(resolved_value) = self
                        .try_resolve_unified_semantic_value_against_content(&property.value, root_content)?
                    else {
                        return Ok(None);
                    };
                    resolved.push(UnifiedSemanticProperty {
                        key: property.key.clone(),
                        value: resolved_value,
                    });
                }
                Ok(Some(UnifiedSemanticValue::Object(resolved)))
            }
        }
    }

    /// Mirror of the generated `resolve_unified_semantic_properties_against_content`.
    fn resolve_unified_semantic_properties_against_content(
        &self,
        properties: &[UnifiedSemanticProperty],
        root_content: &ParseContent<'i>,
    ) -> ParseResult<Vec<UnifiedSemanticProperty>> {
        let mut resolved = Vec::with_capacity(properties.len());
        for property in properties {
            resolved.push(UnifiedSemanticProperty {
                key: property.key.clone(),
                value: self
                    .resolve_unified_semantic_value_against_content(&property.value, root_content)?,
            });
        }
        Ok(resolved)
    }

    /// Mirror of the generated `resolve_semantic_predicate_spec_against_content`: resolve a POST
    /// predicate's args against the view-selected content (default `Raw`; `view: shaped` selects the
    /// `->`-shaped content). HARD-resolving (unresolvable `$ref` → `ContextualError`).
    fn resolve_predicate_spec_against_content(
        &self,
        spec: &SemanticPredicateSpec,
        raw_content: &ParseContent<'i>,
        shaped_content: &ParseContent<'i>,
    ) -> ParseResult<SemanticPredicateSpec> {
        let selected_content = match spec.view {
            SemanticPredicateContentView::Raw => raw_content,
            SemanticPredicateContentView::Shaped => shaped_content,
        };
        let mut resolved_args = Vec::with_capacity(spec.args.len());
        for arg in &spec.args {
            resolved_args
                .push(self.resolve_unified_semantic_value_against_content(arg, selected_content)?);
        }
        Ok(SemanticPredicateSpec {
            name: spec.name.clone(),
            args: resolved_args,
            phase: spec.phase,
            view: spec.view,
        })
    }

    /// Mirror of the generated `try_resolve_semantic_predicate_spec_against_content` (the BRANCH
    /// predicate variant — an unresolvable arg yields `Ok(None)`, blocking the branch).
    fn try_resolve_predicate_spec_against_content(
        &self,
        spec: &SemanticPredicateSpec,
        raw_content: &ParseContent<'i>,
        shaped_content: &ParseContent<'i>,
    ) -> ParseResult<Option<SemanticPredicateSpec>> {
        let selected_content = match spec.view {
            SemanticPredicateContentView::Raw => raw_content,
            SemanticPredicateContentView::Shaped => shaped_content,
        };
        let mut resolved_args = Vec::with_capacity(spec.args.len());
        for arg in &spec.args {
            let Some(resolved_arg) =
                self.try_resolve_unified_semantic_value_against_content(arg, selected_content)?
            else {
                return Ok(None);
            };
            resolved_args.push(resolved_arg);
        }
        Ok(Some(SemanticPredicateSpec {
            name: spec.name.clone(),
            args: resolved_args,
            phase: spec.phase,
            view: spec.view,
        }))
    }

    /// Mirror of the generated `coerce_semantic_runtime_scalar`: a resolved reference's text is coerced
    /// bool → number → identifier → string (variant-SENSITIVE downstream — the fact-index name match
    /// distinguishes `Identifier("x")` from `String("x")`, which is why grammar predicate args use
    /// unquoted identifiers).
    fn coerce_semantic_runtime_scalar(&self, value: &str) -> SemanticRuntimeValue {
        let normalized = value.trim();
        if normalized.eq_ignore_ascii_case("true") {
            return SemanticRuntimeValue::Boolean(true);
        }
        if normalized.eq_ignore_ascii_case("false") {
            return SemanticRuntimeValue::Boolean(false);
        }
        if normalized.parse::<f64>().is_ok() {
            return SemanticRuntimeValue::Number(normalized.to_string());
        }
        if Self::semantic_identifier(normalized) {
            return SemanticRuntimeValue::Identifier(normalized.to_string());
        }
        SemanticRuntimeValue::String(normalized.to_string())
    }

    /// Mirror of the generated `coerce_unified_semantic_scalar` (same ladder, unified-value flavor).
    fn coerce_unified_semantic_scalar(&self, value: &str) -> UnifiedSemanticValue {
        let normalized = value.trim();
        if normalized.eq_ignore_ascii_case("true") {
            return UnifiedSemanticValue::Boolean(true);
        }
        if normalized.eq_ignore_ascii_case("false") {
            return UnifiedSemanticValue::Boolean(false);
        }
        if normalized.parse::<f64>().is_ok() {
            return UnifiedSemanticValue::Number(normalized.to_string());
        }
        if Self::semantic_identifier(normalized) {
            return UnifiedSemanticValue::Identifier(normalized.to_string());
        }
        UnifiedSemanticValue::String(normalized.to_string())
    }

    /// Mirror of the generated `resolve_semantic_reference`: `$N…` positional / named-dotted /
    /// `[N]`-indexed / `.len` reference text → the resolved scalar string.
    fn resolve_semantic_reference(
        &self,
        root_content: &ParseContent<'i>,
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

    /// Mirror of the generated `resolve_positional_semantic_reference` (`$N[.path]` over the RAW tree).
    fn resolve_positional_semantic_reference(
        &self,
        root_content: &ParseContent<'i>,
        reference: &str,
    ) -> Option<String> {
        let (index, path_segments) = Self::parse_semantic_reference_segments(reference)?;
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
            if let Some(index) = Self::parse_bracketed_index(segment) {
                current_node = Self::find_semantic_indexed_child(&current_node.content, index)?;
            } else {
                current_node =
                    Self::find_semantic_named_descendant(&current_node.content, segment)?;
            }
        }
        self.semantic_node_scalar(current_node)
    }

    /// Mirror of the generated `resolve_named_semantic_reference` (SEMREF-SHAPED: against a
    /// `ParseContent::Json` the dotted path walks the shaped object; otherwise the raw named-descendant
    /// walk).
    fn resolve_named_semantic_reference(
        &self,
        root_content: &ParseContent<'i>,
        reference: &str,
    ) -> Option<String> {
        let lexed_segments = Self::lex_semantic_reference_segments_named(reference)?;
        if lexed_segments.is_empty() {
            return None;
        }
        if let ParseContent::Json(value) = root_content {
            let mut current = value;
            for segment in &lexed_segments {
                if let Some(index) = Self::parse_bracketed_index(segment) {
                    current = current.get(index)?;
                } else {
                    if !Self::semantic_identifier(segment) {
                        return None;
                    }
                    current = current.get(*segment)?;
                }
            }
            return match current {
                serde_json::Value::String(text) => Some(text.clone()),
                serde_json::Value::Number(number) => Some(number.to_string()),
                serde_json::Value::Bool(boolean) => Some(boolean.to_string()),
                _ => None,
            };
        }
        let mut iter = lexed_segments.iter();
        let first = iter.next()?;
        if !Self::semantic_identifier(first) {
            return None;
        }
        let mut current_node = Self::find_semantic_named_descendant(root_content, first)?;
        for segment in iter {
            if let Some(index) = Self::parse_bracketed_index(segment) {
                current_node = Self::find_semantic_indexed_child(&current_node.content, index)?;
            } else {
                if !Self::semantic_identifier(segment) {
                    return None;
                }
                current_node =
                    Self::find_semantic_named_descendant(&current_node.content, segment)?;
            }
        }
        self.semantic_node_scalar(current_node)
    }

    /// Mirror of the generated `parse_semantic_reference_segments` (`$N` + lexed `.name`/`[N]` suffix).
    fn parse_semantic_reference_segments(reference: &str) -> Option<(usize, Vec<&str>)> {
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
        let suffix = normalized[index_end..].trim();
        let segments = Self::lex_semantic_reference_segments_suffix(suffix)?;
        for segment in &segments {
            if Self::parse_bracketed_index(segment).is_none() && !Self::semantic_identifier(segment)
            {
                return None;
            }
        }
        Some((index, segments))
    }

    /// Mirror of the generated `find_semantic_indexed_child`.
    fn find_semantic_indexed_child<'a>(
        content: &'a ParseContent<'i>,
        index: usize,
    ) -> Option<&'a ParseNode<'i>> {
        match content {
            ParseContent::Sequence(elements) | ParseContent::Quantified(elements, _) => {
                elements.get(index)
            }
            ParseContent::Alternative(node) if index == 0 => Some(node.as_ref()),
            _ => None,
        }
    }

    /// Mirror of the generated `find_semantic_named_descendant` (a raw-tree walk — it does NOT descend
    /// into a shaped `Json` leaf).
    fn find_semantic_named_descendant<'a>(
        content: &'a ParseContent<'i>,
        target_name: &str,
    ) -> Option<&'a ParseNode<'i>> {
        match content {
            ParseContent::Sequence(elements) | ParseContent::Quantified(elements, _) => {
                for node in elements {
                    if node.rule_name == target_name {
                        return Some(node);
                    }
                    if let Some(found) =
                        Self::find_semantic_named_descendant(&node.content, target_name)
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
                    Self::find_semantic_named_descendant(&node.content, target_name)
                }
            }
            _ => None,
        }
    }

    /// Mirror of the generated `semantic_node_scalar` / `semantic_content_scalar`.
    fn semantic_node_scalar(&self, node: &ParseNode<'i>) -> Option<String> {
        self.semantic_content_scalar(&node.content)
    }

    fn semantic_content_scalar(&self, content: &ParseContent<'i>) -> Option<String> {
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
                if merged.trim().is_empty() { None } else { Some(merged) }
            }
        }
    }

    /// Mirror of the generated `semantic_identifier`.
    fn semantic_identifier(segment: &str) -> bool {
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

    /// Mirror of the generated `lex_semantic_reference_segments_suffix`.
    fn lex_semantic_reference_segments_suffix(suffix: &str) -> Option<Vec<&str>> {
        let mut segments = Vec::new();
        let mut remaining = suffix.trim();
        while !remaining.is_empty() {
            if let Some(rest) = remaining.strip_prefix('.') {
                let bytes = rest.as_bytes();
                let mut end = 0usize;
                if bytes.is_empty() || !(bytes[0] == b'_' || (bytes[0] as char).is_ascii_alphabetic())
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
                let bracketed_end = 1 + end + 1;
                segments.push(&remaining[..bracketed_end]);
                remaining = &remaining[bracketed_end..];
            } else {
                return None;
            }
        }
        Some(segments)
    }

    /// Mirror of the generated `lex_semantic_reference_segments_named`.
    fn lex_semantic_reference_segments_named(reference: &str) -> Option<Vec<&str>> {
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
            && (bytes[head_end] == b'_' || (bytes[head_end] as char).is_ascii_alphanumeric())
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

    /// Mirror of the generated `parse_bracketed_index`.
    fn parse_bracketed_index(segment: &str) -> Option<usize> {
        let inner = segment.strip_prefix('[')?.strip_suffix(']')?;
        if inner.is_empty() || !inner.bytes().all(|b| (b as char).is_ascii_digit()) {
            return None;
        }
        inner.parse::<usize>().ok()
    }

    // ── Tournament annotation helpers (mirror codegen's generation-time readers) ────────────────────

    /// Mirror of codegen's `rule_branch_priorities` (`ast_based_generator.rs:7305`): the per-branch
    /// `@priority` (or `@precedence`) list, defaulting to all-zero. Explicit `@priority` wins over
    /// `@precedence`.
    fn rule_branch_priorities(&self, rule_name: &str, branch_count: usize) -> Vec<i64> {
        let default_priorities = vec![0i64; branch_count];
        let Some(entries) = self
            .annotations
            .and_then(|a| a.semantic_annotations.get(rule_name))
        else {
            return default_priorities;
        };
        let mut precedence_priorities: Option<Vec<i64>> = None;
        let mut explicit_priorities: Option<Vec<i64>> = None;
        for annotation in entries {
            let Some(name) = annotation.name() else {
                continue;
            };
            let payload = annotation.ast().payload_text().trim().to_string();
            let Some(parsed) = parse_semantic_branch_priorities(&payload, branch_count) else {
                continue;
            };
            let normalized = name.trim().to_ascii_lowercase();
            match normalized.as_str() {
                "precedence" => precedence_priorities = Some(parsed),
                "priority" => explicit_priorities = Some(parsed),
                _ => {}
            }
        }
        explicit_priorities
            .or(precedence_priorities)
            .unwrap_or(default_priorities)
    }

    /// Mirror of codegen's `rule_associativity` (`ast_based_generator.rs:7282`): the rule's
    /// `@associativity` (default `left`).
    fn rule_associativity(&self, rule_name: &str) -> SemanticAssociativity {
        let Some(entries) = self
            .annotations
            .and_then(|a| a.semantic_annotations.get(rule_name))
        else {
            return SemanticAssociativity::Left;
        };
        let mut associativity = SemanticAssociativity::Left;
        for annotation in entries {
            let Some(name) = annotation.name() else {
                continue;
            };
            if !name.trim().eq_ignore_ascii_case("associativity") {
                continue;
            }
            let payload = annotation.ast().payload_text().trim().to_string();
            if let Some(parsed) = SemanticAssociativity::parse(&payload) {
                associativity = parsed;
            }
        }
        associativity
    }

    /// Mirror codegen's `generate_post_body_span_transform` (`ast_based_generator.rs:4094`): if the
    /// rule carries a canonical `@transform` (`str::parse::<T>().unwrap_or(D)`) and the body result is
    /// NOT already a `TransformedTerminal`, replace it with `TransformedTerminal(matched_span.trim()
    /// .parse::<T>().unwrap_or(D).to_string())`. This lets a self-hosted `digit+` rule carry a typed
    /// numeric result with no `/.../` body; `to_json_value()` then renders the `TransformedTerminal` as
    /// a JSON number (e.g. `{min: 12}` instead of `["1","2"]`).
    fn apply_post_body_span_transform(
        &self,
        rule_name: &str,
        content: ParseContent<'i>,
        start_pos: usize,
    ) -> ParseContent<'i> {
        if matches!(content, ParseContent::TransformedTerminal(_)) {
            return content;
        }
        let Some((target_type, default_expr)) = self.rule_span_transform(rule_name) else {
            return content;
        };
        let span_text = self.input[start_pos..self.position].trim();
        ParseContent::TransformedTerminal(numeric_span_transform(&target_type, &default_expr, span_text))
    }

    /// The rule's canonical `@transform` `(target_type, default_expr)`, if any — mirrors the detection
    /// in `generate_post_body_span_transform`: the first `transform`-named semantic annotation whose AST
    /// is a `TransformExpr` parsing as a canonical `str::parse::<T>().unwrap_or(D)`.
    fn rule_span_transform(&self, rule_name: &str) -> Option<(String, String)> {
        let annotations = self.annotations?;
        let semantic_annotations = annotations.semantic_annotations.get(rule_name)?;
        for annotation in semantic_annotations {
            if !annotation_is_transform_directive(annotation) {
                continue;
            }
            if let UnifiedSemanticAST::TransformExpr { expression } = annotation.ast()
                && let Some(transform) = parse_canonical_transform_expression(expression)
            {
                return Some((transform.target_type, transform.default_expr));
            }
        }
        None
    }

    /// The rule's allowed dialect profiles from its `@profiles` annotation (lowercased, empty = ungated)
    /// — mirrors codegen's `rule_profiles` (`ast_based_generator.rs:7108`).
    fn rule_profiles(&self, rule_name: &str) -> Vec<String> {
        let Some(annotations) = self.annotations else {
            return Vec::new();
        };
        let Some(entries) = annotations.semantic_annotations.get(rule_name) else {
            return Vec::new();
        };
        let mut profiles = Vec::new();
        for annotation in entries {
            // Mirror `semantic_directive_parts` for the NAMED `@profiles` directive: name + list payload.
            let Some(name) = annotation.name() else {
                continue;
            };
            if !name.trim().eq_ignore_ascii_case("profiles") {
                continue;
            }
            let payload = annotation.ast().payload_text().trim().to_string();
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

    /// Is a rule with the given `@profiles` allow-list active under `self.active_profile`? Mirrors
    /// codegen's `rule_profile_is_enabled` (`ast_based_generator.rs:4903`): empty allow-list or a `None`
    /// active profile ⇒ enabled; otherwise the active profile must match one allowed (case-insensitive).
    fn profile_enabled(&self, allowed: &[String]) -> bool {
        if allowed.is_empty() {
            return true;
        }
        match self.active_profile.as_deref() {
            Some(active) => allowed.iter().any(|candidate| active.eq_ignore_ascii_case(candidate)),
            None => true,
        }
    }

    /// Mirror codegen's `generate_unresolved_reference_method` (`ast_based_generator.rs:833-946`) for a
    /// rule that is referenced but not defined in the grammar. Byte-identical to the emitted method:
    /// same `rule_name`, `content`, and `span`. These built-ins let a `/.../`-free grammar express
    /// any-char / negated-class idioms (the regex grammar's `unicode_char` uses `builtin_ascii_char`
    /// and `builtin_any_char`); everything else is codegen's Backtrack stub.
    fn parse_unresolved_reference(
        &mut self,
        rule_name: &str,
        start_pos: usize,
    ) -> ParseResult<ParseNode<'i>> {
        let interned = intern(rule_name);
        match rule_name {
            // Zero-width literal markers (span start..start) — ast_based_generator.rs:837-856.
            "true" => Ok(ParseNode {
                rule_name: interned,
                content: ParseContent::Terminal("true"),
                span: start_pos..start_pos,
            }),
            "false" => Ok(ParseNode {
                rule_name: interned,
                content: ParseContent::Terminal("false"),
                span: start_pos..start_pos,
            }),
            // `@…`-to-end-of-line matcher. Codegen skips leading layout UNCONDITIONALLY here
            // (`consume_optional_whitespace`, not layout-policy-gated) — ast_based_generator.rs:857-885.
            "semantic_annotation" => {
                let checkpoint = self.position;
                self.consume_optional_whitespace();
                let at_pos = self.position;
                if at_pos >= self.input.len() || self.input.as_bytes()[at_pos] != b'@' {
                    self.position = checkpoint;
                    return Err(ParseError::Backtrack { position: checkpoint });
                }
                while self.position < self.input.len() {
                    let b = self.input.as_bytes()[self.position];
                    if b == b'\n' || b == b'\r' {
                        break;
                    }
                    self.position += 1;
                }
                let end_pos = self.position;
                Ok(ParseNode {
                    rule_name: interned,
                    content: ParseContent::Terminal(&self.input[at_pos..end_pos]),
                    span: at_pos..end_pos,
                })
            }
            // Native any-single-Unicode-scalar matcher (no layout skip) — ast_based_generator.rs:894-912.
            "builtin_any_char" => {
                let matched_char = match self.input[start_pos..].chars().next() {
                    Some(ch) => ch,
                    None => return Err(ParseError::Backtrack { position: start_pos }),
                };
                let end_pos = start_pos + matched_char.len_utf8();
                self.position = end_pos;
                Ok(ParseNode {
                    rule_name: interned,
                    content: ParseContent::Terminal(&self.input[start_pos..end_pos]),
                    span: start_pos..end_pos,
                })
            }
            // Native single-ASCII-scalar matcher; Backtracks on a non-ASCII char or EOF — the negation
            // half of `!builtin_ascii_char builtin_any_char` — ast_based_generator.rs:919-937.
            "builtin_ascii_char" => {
                let matched_char = match self.input[start_pos..].chars().next() {
                    Some(ch) if ch.is_ascii() => ch,
                    _ => return Err(ParseError::Backtrack { position: start_pos }),
                };
                let end_pos = start_pos + matched_char.len_utf8();
                self.position = end_pos;
                Ok(ParseNode {
                    rule_name: interned,
                    content: ParseContent::Terminal(&self.input[start_pos..end_pos]),
                    span: start_pos..end_pos,
                })
            }
            // Default: the Backtrack stub for a genuinely unknown reference — ast_based_generator.rs:939-944.
            _ => Err(ParseError::Backtrack {
                position: self.position,
            }),
        }
    }

    /// Produce the rule's final (post-transform) `ParseContent`. An `Or` body applies its
    /// return-annotation transform per-branch inside the tournament; a non-`Or` body gets the
    /// rule-level branch-0 transform (explicit annotation, else the synthetic `-> $1` passthrough for a
    /// single-element body, else raw).
    ///
    /// `capture_raw` / `raw_out` mirror the generated rule body's `semantic_capture_raw_for_post` /
    /// `semantic_raw_content` locals (PARSE-HARNESS.6.2): codegen INLINES every nested construct into
    /// the rule method, so ANY `Or` (body-level or nested) writes the same rule-scoped local. The
    /// out-param is threaded through the whole dispatch to reproduce exactly that
    /// mutable-local-with-no-rollback behavior.
    fn parse_rule_body(
        &mut self,
        body: &'g ASTNode,
        rule_name: &str,
        start_pos: usize,
        capture_raw: bool,
        raw_out: &mut Option<ParseContent<'i>>,
    ) -> ParseResult<ParseContent<'i>> {
        match body {
            ASTNode::Or { alternatives } => {
                self.parse_or(alternatives, rule_name, capture_raw, raw_out)
            }
            _ => {
                let raw = self.parse_node(body, rule_name, capture_raw, raw_out)?;
                let ann = self.resolve_branch_annotation(rule_name, 0, body);
                Ok(match ann {
                    Some(a) => self.apply_return_annotation(&a, &raw, start_pos),
                    None => raw,
                })
            }
        }
    }

    /// Dispatch a single (sub-)node to its raw structural `ParseContent`. This is the codegen's
    /// `generate_node_parsing_logic`: it does NOT apply a rule-level transform (that is the caller's
    /// job); an `Or` sub-node applies its own per-branch transforms inside `parse_or`.
    fn parse_node(
        &mut self,
        node: &'g ASTNode,
        rule_name: &str,
        capture_raw: bool,
        raw_out: &mut Option<ParseContent<'i>>,
    ) -> ParseResult<ParseContent<'i>> {
        match node {
            ASTNode::Or { alternatives } => {
                self.parse_or(alternatives, rule_name, capture_raw, raw_out)
            }
            ASTNode::Sequence { elements } => {
                self.parse_sequence(elements, rule_name, capture_raw, raw_out)
            }
            ASTNode::Atom { value } => self.parse_atom(value, rule_name),
            ASTNode::Quantified { element, quantifier } => {
                self.parse_quantified(element, quantifier, rule_name, capture_raw, raw_out)
            }
            ASTNode::Lookahead { element, positive } => {
                self.parse_lookahead(element, *positive, rule_name, capture_raw, raw_out)
            }
        }
    }

    // ── Ordered choice (the tournament) ────────────────────────────────────────────────────────────

    /// Ordered choice honoring `branch_policy` — the full generated tournament, mirrored from the
    /// emitted multi-branch template (PARSE-HARNESS.6.2, tool-verified from the emitted parsers):
    ///
    /// - single-branch: parse + raw capture (before the transform) + branch-0 transform;
    /// - multi-branch: per branch — the `ordered` early-skip, `try_parse` attempt, the branch
    ///   transform computed BEFORE the position rollback (BRANCH-BROADCAST-FIX.3), the BRANCH-phase
    ///   predicates (rule-level ∪ branch-local: `branch_predicates_for_rule` returns RULE-level
    ///   branch predicates only, and `_for_rule_branch` supplies the candidate branch's own — an
    ///   inline branch predicate is branch-LOCAL since BRANCH-PREDICATE-LOCALITY.2),
    ///   the full `should_take` ladder (policy × priority × length × associativity, incl. the
    ///   `nonassoc` tie), then the C3-B semantic-delta extract + rollback (no loser leakage);
    /// - winner: position, the winner's delta replay, the winning branch's branch-start inline
    ///   actions (INLINE-ACTIONS.2), and the raw-capture assignment.
    fn parse_or(
        &mut self,
        alternatives: &'g [ASTNode],
        rule_name: &str,
        capture_raw: bool,
        raw_out: &mut Option<ParseContent<'i>>,
    ) -> ParseResult<ParseContent<'i>> {
        if alternatives.len() == 1 {
            let branch_start = self.position;
            let raw = self.parse_node(&alternatives[0], rule_name, capture_raw, raw_out)?;
            // The generated single-branch template captures the RAW result before the transform
            // (`if semantic_capture_raw_for_post { semantic_raw_content = Some(result.clone()); }`).
            if capture_raw {
                *raw_out = Some(raw.clone());
            }
            let ann = self.resolve_branch_annotation(rule_name, 0, &alternatives[0]);
            return Ok(match ann {
                Some(a) => self.apply_return_annotation(&a, &raw, branch_start),
                None => raw,
            });
        }

        let policy = self.rule_branch_policy(rule_name);
        let priorities = self.rule_branch_priorities(rule_name, alternatives.len());
        let associativity = self.rule_associativity(rule_name);
        let parse_start = self.position;

        let mut best_content: Option<ParseContent<'i>> = None;
        let mut best_raw_content: Option<ParseContent<'i>> = None;
        let mut best_end = parse_start;
        let mut best_priority: i64 = i64::MIN;
        let mut best_branch_index: usize = 0;
        let mut nonassoc_tie = false;
        // C3-B (SV-EXH-PROOF.3.3.4.b.6.2.33): tournament-scope semantic checkpoint; each branch
        // extracts its delta then rolls back; ONLY the winner's delta is replayed at the end.
        let tournament_checkpoint = self.semantic_state.checkpoint();
        let mut best_semantic_delta: Option<SemanticRuntimeDelta> = None;

        for (idx, alternative) in alternatives.iter().enumerate() {
            // The emitted arm guard: `if policy == ordered && best_content.is_some() {} else {…}` —
            // under `ordered` later branches are not even attempted once one succeeded.
            if policy == SemanticBranchPolicy::Ordered && best_content.is_some() {
                continue;
            }
            self.position = parse_start;
            let branch_start = self.position;
            let attempt =
                self.try_parse(|p| p.parse_node(alternative, rule_name, capture_raw, raw_out));
            if let Some(raw) = attempt {
                let candidate_end = self.position;
                let candidate_priority: i64 = priorities.get(idx).copied().unwrap_or(0);
                // BRANCH-BROADCAST-FIX.3 — the branch transform runs BEFORE the position rollback
                // ($text/MatchedText slices `input[start..position]`).
                let ann = self.resolve_branch_annotation(rule_name, idx, alternative);
                let transformed = match ann {
                    Some(a) => self.apply_return_annotation(&a, &raw, branch_start),
                    None => raw.clone(),
                };
                self.position = parse_start;

                // BRANCH-phase predicates: rule-level ∪ branch-local, `try_resolve` semantics (an
                // unresolvable `$ref` BLOCKS the branch), evaluated on the LIVE state (this branch's
                // own emissions are visible), against the (raw, transformed) content pair.
                let mut branch_predicate_blocked = false;
                for directive in self
                    .compiled_sem
                    .branch_predicates_for_rule(rule_name)
                    .chain(self.compiled_sem.branch_predicates_for_rule_branch(rule_name, idx))
                {
                    if let SemanticRuntimeDirective::Predicate(spec) = directive {
                        if spec.phase == SemanticPredicatePhase::Branch {
                            let Some(resolved_spec) = self
                                .try_resolve_predicate_spec_against_content(
                                    spec,
                                    &raw,
                                    &transformed,
                                )?
                            else {
                                branch_predicate_blocked = true;
                                break;
                            };
                            match self.semantic_state.evaluate_content_aware_predicate(
                                &resolved_spec,
                                &raw,
                                &transformed,
                            ) {
                                Some(true) => {}
                                Some(false) => {
                                    branch_predicate_blocked = true;
                                    break;
                                }
                                None => {}
                            }
                        }
                    }
                }

                // The FULL generated `should_take` ladder (policy × priority × length × associativity).
                let should_take = if branch_predicate_blocked {
                    false
                } else if policy == SemanticBranchPolicy::Ordered {
                    best_content.is_none()
                } else if policy == SemanticBranchPolicy::PriorityFirst {
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
                        match associativity {
                            SemanticAssociativity::Right => idx > best_branch_index,
                            SemanticAssociativity::NonAssoc => {
                                if idx != best_branch_index {
                                    nonassoc_tie = true;
                                }
                                false
                            }
                            SemanticAssociativity::Left => false,
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
                    match associativity {
                        SemanticAssociativity::Right => idx > best_branch_index,
                        SemanticAssociativity::NonAssoc => {
                            if idx != best_branch_index {
                                nonassoc_tie = true;
                            }
                            false
                        }
                        SemanticAssociativity::Left => false,
                    }
                };

                // C3-B: extract this branch's delta, then roll back to the tournament checkpoint so no
                // branch's effects leak into the next; the winner's delta is replayed after the loop.
                let candidate_delta = self
                    .semantic_state
                    .extract_delta_since(&tournament_checkpoint);
                self.semantic_state
                    .rollback_to_named(tournament_checkpoint.clone(), Some(rule_name));

                if should_take {
                    best_end = candidate_end;
                    best_priority = candidate_priority;
                    best_branch_index = idx;
                    if capture_raw {
                        best_raw_content = Some(raw.clone());
                    }
                    best_content = Some(transformed);
                    best_semantic_delta = Some(candidate_delta);
                }
            }
        }

        if nonassoc_tie {
            return Err(ParseError::Backtrack {
                position: parse_start,
            });
        }
        match best_content {
            Some(content) => {
                self.position = best_end;
                // Replay ONLY the winning branch's semantic effects (C3-B).
                if let Some(delta) = best_semantic_delta {
                    if !delta.is_empty() {
                        self.semantic_state.apply_delta(delta);
                    }
                }
                // INLINE-ACTIONS.2: the winning branch's branch-start inline actions fire here — after
                // the winner's body delta is replayed and before the rule-level effect/post phases.
                // Cloned out of the registry first (the generated code does the same, to release the
                // immutable borrow before the `&mut self` apply).
                let branch_start_effects: Vec<SemanticRuntimeDirective> = self
                    .compiled_sem
                    .branch_effect_directives_for_rule_branch(rule_name, best_branch_index)
                    .cloned()
                    .collect();
                for effect in &branch_start_effects {
                    self.apply_branch_start_effect_directive(effect, &content)?;
                }
                // The emitted tournament ends with the UNCONDITIONAL rule-local assignment
                // `semantic_raw_content = best_raw_content` (None unless capture_raw).
                *raw_out = best_raw_content;
                Ok(content)
            }
            None => Err(ParseError::Backtrack {
                position: parse_start,
            }),
        }
    }

    // ── Sequence ───────────────────────────────────────────────────────────────────────────────────

    /// Concatenation. Each element is wrapped as `ParseNode { rule_name: "element_{i}", content, span }`;
    /// an optional (`?`) element yields the inner content on success or an empty `Sequence` on absence
    /// (the codegen's `generate_sequence_element` special case). The whole is `Sequence(elements)`.
    fn parse_sequence(
        &mut self,
        elements: &'g [ASTNode],
        rule_name: &str,
        capture_raw: bool,
        raw_out: &mut Option<ParseContent<'i>>,
    ) -> ParseResult<ParseContent<'i>> {
        let mut sequence_elements: Vec<ParseNode<'i>> = Vec::with_capacity(elements.len());
        for (idx, element) in elements.iter().enumerate() {
            let element_start = self.position;
            let element_content = match element {
                ASTNode::Quantified { element: inner, quantifier } if quantifier == "?" => {
                    // Optional sequence element: inner content on match, empty Sequence on absence.
                    match self.try_parse(|p| p.parse_node(inner, rule_name, capture_raw, raw_out)) {
                        Some(content) => content,
                        None => ParseContent::Sequence(Vec::new()),
                    }
                }
                _ => self.parse_node(element, rule_name, capture_raw, raw_out)?,
            };
            let element_end = self.position;
            sequence_elements.push(ParseNode {
                rule_name: intern(&format!("element_{idx}")),
                content: element_content,
                span: element_start..element_end,
            });
        }
        Ok(ParseContent::Sequence(sequence_elements))
    }

    // ── Quantifier ───────────────────────────────────────────────────────────────────────────────────

    /// The unified quantifier engine: `parse_quantifier_bounds` decodes the surface label to `(min,
    /// max)`; each iteration is atomic (via `try_parse`); a zero-length iteration breaks (the
    /// zero-length guard); `max` caps the count; an unmet `min` backtracks. Yields
    /// `Quantified(iterations, label)`.
    fn parse_quantified(
        &mut self,
        element: &'g ASTNode,
        quantifier: &str,
        rule_name: &str,
        capture_raw: bool,
        raw_out: &mut Option<ParseContent<'i>>,
    ) -> ParseResult<ParseContent<'i>> {
        const SAFETY_LIMIT: usize = 10_000;
        let (min, max) = parse_quantifier_bounds(quantifier).unwrap_or((0, None));
        let quantifier_start = self.position;

        let mut results: Vec<ParseNode<'i>> = Vec::new();
        let mut last_position = self.position;
        let mut iteration_count = 0usize;
        loop {
            if iteration_count >= SAFETY_LIMIT {
                break;
            }
            if let Some(m) = max {
                if iteration_count >= m {
                    break;
                }
            }
            match self.try_parse(|p| p.parse_node(element, rule_name, capture_raw, raw_out)) {
                Some(content) => {
                    let current_position = self.position;
                    // Zero-length guard: a matching but non-advancing iteration would loop forever;
                    // it is discarded (not pushed, not counted), exactly as the codegen does.
                    if current_position == last_position {
                        break;
                    }
                    // The generated per-iteration node is rule_name "quantified", span 0..0 (a fixed
                    // synthetic wrapper — the real spans live on the element's own sub-nodes).
                    results.push(ParseNode {
                        rule_name: intern("quantified"),
                        content,
                        span: 0..0,
                    });
                    last_position = current_position;
                    iteration_count += 1;
                }
                None => break,
            }
        }

        if iteration_count < min {
            self.position = quantifier_start;
            return Err(ParseError::Backtrack { position: quantifier_start });
        }
        Ok(ParseContent::Quantified(results, intern(quantifier)))
    }

    // ── Lookahead ────────────────────────────────────────────────────────────────────────────────────

    /// Zero-width assertion `&e` (positive) / `!e` (negative). Speculatively parses the inner element,
    /// then unconditionally restores the cursor (consumes nothing); positive fails if the inner failed,
    /// negative fails if it succeeded.
    fn parse_lookahead(
        &mut self,
        element: &'g ASTNode,
        positive: bool,
        rule_name: &str,
        capture_raw: bool,
        raw_out: &mut Option<ParseContent<'i>>,
    ) -> ParseResult<ParseContent<'i>> {
        let lookahead_start = self.position;
        let matched = self.try_parse(|p| p.parse_node(element, rule_name, capture_raw, raw_out));
        self.position = lookahead_start;
        let ok = if positive { matched.is_some() } else { matched.is_none() };
        if ok {
            Ok(ParseContent::Sequence(Vec::new()))
        } else {
            Err(ParseError::Backtrack { position: lookahead_start })
        }
    }

    // ── Atom (terminal / regex / rule reference) ─────────────────────────────────────────────────────

    /// A leaf `Atom`. The token type-tag (`parts[0]`) selects: `rule_reference` → recurse into the named
    /// rule, wrapped `Alternative(Box<node>)`; `regex` → `match_regex` → `Terminal`; `quoted_string` and
    /// the other literal tags → `match_string` → `Terminal`; anything else → an empty terminal
    /// (consumes 0), exactly as the codegen's `generate_atom_logic`.
    fn parse_atom(&mut self, value: &'g ASTValue, _rule_name: &str) -> ParseResult<ParseContent<'i>> {
        let parts = match value {
            ASTValue::Token(parts) => parts,
            // The normalizer never emits `ASTValue::Node`; codegen treats it as an empty terminal.
            ASTValue::Node(_) => return Ok(ParseContent::Terminal("")),
        };
        if parts.len() < 2 {
            return Ok(ParseContent::Terminal(""));
        }
        let TokenValue::String(tag) = &parts[0];
        let TokenValue::String(val) = &parts[1];
        match tag.as_str() {
            "rule_reference" => {
                let node = self.parse_rule(val)?;
                Ok(ParseContent::Alternative(Box::new(node)))
            }
            "regex" => {
                let matched = self.match_regex(val, true)?;
                Ok(ParseContent::Terminal(matched))
            }
            "quoted_string" | "number" | "probability" | "include_dir" | "include_file" | "rule" => {
                let matched = self.match_string(val)?;
                Ok(ParseContent::Terminal(matched))
            }
            _ => Ok(ParseContent::Terminal("")),
        }
    }

    // ── Speculation ──────────────────────────────────────────────────────────────────────────────────

    /// Speculative parse with full backtrack: save cursor + depth + a semantic checkpoint; on `Err`,
    /// restore the cursor and depth and roll the semantic state back (mirroring the generated
    /// `try_parse`, which snapshots semantic state too). `furthest_position` is deliberately NOT
    /// restored — it is monotonic, so a failed deeper probe still records how far it reached.
    fn try_parse<T>(&mut self, f: impl FnOnce(&mut Self) -> ParseResult<T>) -> Option<T> {
        let saved_pos = self.position;
        let saved_depth = self.depth;
        let checkpoint = self.semantic_state.checkpoint();
        match f(self) {
            Ok(result) => Some(result),
            Err(_) => {
                self.position = saved_pos;
                self.depth = saved_depth;
                self.semantic_state
                    .rollback_to_named(checkpoint, Some("interpreter try_parse Err"));
                None
            }
        }
    }

    // ── Lexical primitives (mirrored byte-for-byte from a generated parser) ───────────────────────────

    fn match_string(&mut self, expected: &str) -> ParseResult<&'i str> {
        // Gated by the layout policy: a whitespace-sensitive grammar (regex) does NOT skip layout
        // before a terminal (`allow_layout_skip_for_terminals`, ast_based_generator.rs:4509/6146).
        if self.layout.skip_layout_for_terminals {
            self.consume_layout_for_terminal(expected);
        }
        let start = self.position;
        let expected_bytes = expected.as_bytes();
        let end = start + expected_bytes.len();
        if self.bytes_match_at(start, expected_bytes) {
            if !self.input.is_char_boundary(start) || !self.input.is_char_boundary(end) {
                return Err(ParseError::InvalidSyntax {
                    message: "interpreter: UTF-8 boundary mismatch matching a terminal",
                    position: start,
                });
            }
            self.position = end;
            return Ok(&self.input[start..end]);
        }
        Err(ParseError::Backtrack { position: start })
    }

    fn match_regex(&mut self, pattern: &str, skip_leading_whitespace: bool) -> ParseResult<&'i str> {
        let can_match_empty = regex_can_match_empty(pattern).map_err(|e| ParseError::ContextualError {
            message: format!("interpreter: invalid regex pattern '{pattern}': {e}"),
            position: self.position,
            rule_stack: Vec::new(),
            input_context: String::new(),
        })?;
        // Gated by the layout policy: a regex-layout-sensitive grammar (regex,
        // systemverilog_preprocessor) forces `can_match_empty=false` AND does not skip layout before a
        // regex-token (`allow_layout_skip_for_regexes`, ast_based_generator.rs:4510/4832-4844). The
        // compile above still runs at the same point, so an invalid pattern still errors identically.
        let can_match_empty = can_match_empty && self.layout.skip_layout_for_regexes;
        if skip_leading_whitespace && self.layout.skip_layout_for_regexes {
            self.consume_layout_for_regex(can_match_empty, pattern);
        }
        let Some(haystack) = self.input.get(self.position..) else {
            return Err(ParseError::ContextualError {
                message: "interpreter: parser position is not on a UTF-8 boundary".to_string(),
                position: self.position,
                rule_stack: Vec::new(),
                input_context: String::new(),
            });
        };
        let match_end = with_anchored_regex(pattern, |re| {
            re.find(haystack).filter(|m| m.start() == 0).map(|m| m.end())
        })
        .map_err(|e| ParseError::ContextualError {
            message: format!("interpreter: invalid regex pattern '{pattern}': {e}"),
            position: self.position,
            rule_stack: Vec::new(),
            input_context: String::new(),
        })?;
        if let Some(end_offset) = match_end {
            let start = self.position;
            self.position += end_offset;
            if let Some(slice) = self.input.get(start..self.position) {
                return Ok(slice);
            }
            return Err(ParseError::ContextualError {
                message: "interpreter: regex matched an invalid UTF-8 span".to_string(),
                position: start,
                rule_stack: Vec::new(),
                input_context: String::new(),
            });
        }
        Err(ParseError::Backtrack { position: self.position })
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
            // Per-introducer comment-arm suppression (PARSE-HARNESS.5.2): each arm is entered only when
            // the grammar does NOT claim its introducer as a non-comment token, mirroring codegen's
            // `terminal_hash_arm` / `terminal_line_comment_arm` / `terminal_block_comment_arm`
            // (`ast_based_generator.rs:4677-4733`). A claimed introducer (`claims_* == true`) is matched
            // structurally, not skipped as trivia.
            if !self.comment_arms.claims_hash && bytes[self.position] == b'#' {
                while self.position < self.input.len() {
                    let b = bytes[self.position];
                    if b == b'\n' || b == b'\r' {
                        break;
                    }
                    self.position += 1;
                }
                continue;
            }
            if !self.comment_arms.claims_line_comment
                && self.position + 1 < bytes.len()
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
            if !self.comment_arms.claims_block_comment
                && self.position + 1 < bytes.len()
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
            if self.position == before {
                break;
            }
        }
    }

    fn consume_layout_for_regex(&mut self, can_match_empty: bool, pattern: &str) {
        if can_match_empty {
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
            // Per-introducer comment-arm suppression (PARSE-HARNESS.5.2), mirroring codegen's
            // `regex_hash_arm` / `regex_line_comment_arm` / `regex_block_comment_arm`
            // (`ast_based_generator.rs:4530-4595`): the STATIC arm is present only when the grammar does
            // not claim the introducer (`!claims_*`), and each present arm keeps the DYNAMIC H.11.3 guard
            // (`regex_token_matches_at_cursor`) so an active token that IS a comment introducer wins.
            if !self.comment_arms.claims_hash && bytes[self.position] == b'#' {
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
            if !self.comment_arms.claims_line_comment
                && self.position + 1 < bytes.len()
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
            if !self.comment_arms.claims_block_comment
                && self.position + 1 < bytes.len()
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
            if self.position == before {
                break;
            }
        }
    }

    fn regex_token_matches_at_cursor(&self, pattern: &str) -> bool {
        let Some(haystack) = self.input.get(self.position..) else {
            return false;
        };
        with_anchored_regex(pattern, |re| re.find(haystack).map(|m| m.start() == 0).unwrap_or(false))
            .unwrap_or(false)
    }

    // ── Return-annotation fold (a faithful runtime mirror of `AstReturnTransformer`) ─────────────────

    /// Resolve the return annotation that applies to branch `branch_idx` of rule `rule_name`: the
    /// explicit `branch_return_annotations[rule][branch_idx]` if present, else the synthetic `-> $1`
    /// passthrough for a single-element body (`body_has_single_element`), else `None`.
    fn resolve_branch_annotation(
        &self,
        rule_name: &str,
        branch_idx: usize,
        branch_body: &ASTNode,
    ) -> Option<BranchAnnotation> {
        let explicit = self
            .annotations
            .and_then(|a| a.branch_return_annotations.get(rule_name))
            .and_then(|branches| branches.get(branch_idx).cloned())
            .flatten();
        explicit.or_else(|| synthesize_default_passthrough(branch_body))
    }

    /// Apply a resolved return annotation's `parsed_ast` fold to `base` (the branch's raw structural
    /// content), producing the shaped content. If the annotation failed to parse (`parsed_ast == None`)
    /// the base passes through unchanged.
    fn apply_return_annotation(
        &self,
        annotation: &BranchAnnotation,
        base: &ParseContent<'i>,
        start_pos: usize,
    ) -> ParseContent<'i> {
        match &annotation.parsed_ast {
            Some(ast) => self.fold_return(ast, base, start_pos),
            None => base.clone(),
        }
    }

    /// The runtime equivalent of `AstReturnTransformer::generate_transform` for the single-capture
    /// convention (`captured_vars == ["result"|"content"]`, verified the only convention at every
    /// codegen call site): `$N` resolves against the single `base` content.
    fn fold_return(
        &self,
        ast: &UnifiedReturnAST,
        base: &ParseContent<'i>,
        start_pos: usize,
    ) -> ParseContent<'i> {
        match ast {
            UnifiedReturnAST::PositionalRef { index } => self.resolve_positional(*index, base),
            UnifiedReturnAST::StringLiteral { value } => ParseContent::Terminal(intern(value)),
            UnifiedReturnAST::NumberLiteral { value } => {
                ParseContent::Json(number_to_json(*value))
            }
            UnifiedReturnAST::BooleanLiteral { value } => {
                ParseContent::Json(serde_json::Value::Bool(*value))
            }
            UnifiedReturnAST::NullLiteral => ParseContent::Json(serde_json::Value::Null),
            UnifiedReturnAST::Identifier { name } => ParseContent::Terminal(intern(name)),
            UnifiedReturnAST::Array { elements } => self.fold_array(elements, base, start_pos),
            UnifiedReturnAST::Object { properties } => {
                let mut obj = serde_json::Map::new();
                let mut sorted: Vec<_> = properties.iter().collect();
                sorted.sort_by(|(l, _), (r, _)| l.cmp(r));
                for (key, value_ast) in sorted {
                    obj.insert(key.clone(), self.fold_value(value_ast, base, start_pos));
                }
                ParseContent::Json(serde_json::Value::Object(obj))
            }
            UnifiedReturnAST::Spread { base: inner } | UnifiedReturnAST::FlattenSpread { base: inner } => {
                // Outside an array a (flatten-)spread degenerates to the shape-preserving identity.
                match self.fold_return(inner, base, start_pos) {
                    ParseContent::Sequence(elems) => ParseContent::Sequence(elems),
                    ParseContent::Quantified(elems, q) => ParseContent::Quantified(elems, q),
                    other => ParseContent::Sequence(vec![ParseNode {
                        rule_name: intern("spread_base"),
                        content: other,
                        span: 0..0,
                    }]),
                }
            }
            UnifiedReturnAST::PropertyAccess { base: inner, property } => {
                let value = self.fold_return(inner, base, start_pos).to_json_value();
                let prop = value.get(property).cloned().unwrap_or(serde_json::Value::Null);
                ParseContent::Json(prop)
            }
            UnifiedReturnAST::ArrayAccess { base: inner, index } => {
                let idx = match index.as_ref() {
                    UnifiedReturnAST::NumberLiteral { value } => *value as usize,
                    _ => 0usize,
                };
                match self.fold_return(inner, base, start_pos) {
                    ParseContent::Sequence(elems) if elems.len() > idx => elems[idx].content.clone(),
                    ParseContent::Quantified(elems, _) if elems.len() > idx => {
                        elems[idx].content.clone()
                    }
                    ParseContent::Json(value) => {
                        let elem = match value {
                            serde_json::Value::Array(ref arr) if arr.len() > idx => arr[idx].clone(),
                            _ => serde_json::Value::Null,
                        };
                        ParseContent::Json(elem)
                    }
                    _ => ParseContent::Terminal(intern("<invalid_array_access>")),
                }
            }
            UnifiedReturnAST::QuantifiedExtraction { base: inner, target } => {
                self.fold_quantified_extraction(inner, target, base, start_pos)
            }
            UnifiedReturnAST::Passthrough => base.clone(),
            UnifiedReturnAST::MatchedText => {
                ParseContent::Terminal(&self.input[start_pos..self.position])
            }
        }
    }

    /// The object-field / array-index value extraction: fold, then convert to a typed `serde_json`
    /// value (`generate_value_extraction` reduces to exactly this — the scalar arms build the same
    /// `Value` the fold's `to_json_value()` yields).
    fn fold_value(
        &self,
        ast: &UnifiedReturnAST,
        base: &ParseContent<'i>,
        start_pos: usize,
    ) -> serde_json::Value {
        self.fold_return(ast, base, start_pos).to_json_value()
    }

    /// `$N` against the single base (the len==1 `generate_positional_ref` rule).
    fn resolve_positional(&self, index: usize, base: &ParseContent<'i>) -> ParseContent<'i> {
        if index == 0 {
            return ParseContent::Terminal(intern("<invalid_positional_ref>"));
        }
        let element_index = index - 1;
        if element_index == 0 {
            // $1: never peel a Quantified (it is "the whole capture group"); peel the artificial
            // Sequence/Alternative packaging.
            match base {
                ParseContent::Sequence(elements) if !elements.is_empty() => {
                    elements[0].content.clone()
                }
                ParseContent::Alternative(node) => node.content.clone(),
                other => other.clone(),
            }
        } else {
            match base {
                ParseContent::Sequence(elements) if elements.len() > element_index => {
                    elements[element_index].content.clone()
                }
                _ => ParseContent::Terminal(intern("<invalid_sequence_access>")),
            }
        }
    }

    /// `[ … ]` array literal → `Sequence(elements)` (matching `generate_array_transform`, which builds
    /// a `ParseContent::Sequence`, not a `Json(Array)`), flattening `Spread`/`FlattenSpread` bases.
    fn fold_array(
        &self,
        elements: &[UnifiedReturnAST],
        base: &ParseContent<'i>,
        start_pos: usize,
    ) -> ParseContent<'i> {
        let mut array_elements: Vec<ParseNode<'i>> = Vec::new();
        for (idx, element) in elements.iter().enumerate() {
            match element {
                UnifiedReturnAST::Spread { base: inner } => {
                    match self.fold_return(inner, base, start_pos) {
                        ParseContent::Sequence(nodes) | ParseContent::Quantified(nodes, _) => {
                            for node in nodes {
                                array_elements.push(node);
                            }
                        }
                        other => array_elements.push(ParseNode {
                            rule_name: intern("spread_element"),
                            content: other,
                            span: 0..0,
                        }),
                    }
                }
                UnifiedReturnAST::FlattenSpread { base: inner } => {
                    match self.fold_return(inner, base, start_pos) {
                        ParseContent::Sequence(nodes) | ParseContent::Quantified(nodes, _) => {
                            for node in nodes {
                                let span_for_inherit = node.span.clone();
                                let rule_name_for_inherit = node.rule_name;
                                let peeled = peel_alternative(node.content);
                                match peeled {
                                    ParseContent::Sequence(inner_nodes)
                                    | ParseContent::Quantified(inner_nodes, _) => {
                                        for inner_node in inner_nodes {
                                            array_elements.push(inner_node);
                                        }
                                    }
                                    ParseContent::Json(serde_json::Value::Array(values)) => {
                                        for value in values {
                                            array_elements.push(ParseNode {
                                                rule_name: rule_name_for_inherit,
                                                content: ParseContent::Json(value),
                                                span: span_for_inherit.clone(),
                                            });
                                        }
                                    }
                                    other_content => array_elements.push(ParseNode {
                                        rule_name: rule_name_for_inherit,
                                        content: other_content,
                                        span: span_for_inherit.clone(),
                                    }),
                                }
                            }
                        }
                        other => array_elements.push(ParseNode {
                            rule_name: intern("flatten_spread_element"),
                            content: other,
                            span: 0..0,
                        }),
                    }
                }
                _ => array_elements.push(ParseNode {
                    rule_name: intern(&format!("element_{idx}")),
                    content: self.fold_return(element, base, start_pos),
                    span: 0..0,
                }),
            }
        }
        ParseContent::Sequence(array_elements)
    }

    /// `$N::target` quantified extraction (`generate_quantified_extraction`). Only the single-capture
    /// `PositionalRef` base is meaningful here; extract per-iteration `subelems[idx]` (or `first`/`last`).
    fn fold_quantified_extraction(
        &self,
        base_ast: &UnifiedReturnAST,
        target: &ExtractionTarget,
        base: &ParseContent<'i>,
        start_pos: usize,
    ) -> ParseContent<'i> {
        let base_expr = match base_ast {
            UnifiedReturnAST::PositionalRef { index } if *index > 0 => {
                self.resolve_positional(*index, base)
            }
            _ => return ParseContent::Terminal(intern("<invalid_extraction_base>")),
        };
        let extraction_idx = match target {
            ExtractionTarget::Index(idx) => *idx,
            ExtractionTarget::First => 0,
            ExtractionTarget::Last => {
                return match &base_expr {
                    ParseContent::Quantified(elements, _) if !elements.is_empty() => {
                        elements.last().unwrap().content.clone()
                    }
                    _ => ParseContent::Terminal(intern("<no_last_element>")),
                };
            }
        };
        match &base_expr {
            ParseContent::Quantified(elements, _) => {
                let extracted: Vec<ParseNode<'i>> = elements
                    .iter()
                    .filter_map(|node| match &node.content {
                        ParseContent::Sequence(subelems) if subelems.len() > extraction_idx => {
                            Some(subelems[extraction_idx].clone())
                        }
                        _ => None,
                    })
                    .collect();
                ParseContent::Sequence(extracted)
            }
            _ => ParseContent::Terminal(intern("<not_quantified>")),
        }
    }

    // ── Per-rule policy resolution ───────────────────────────────────────────────────────────────────

    /// The rule's ordered-choice policy (`rule_branch_policy`): scan `semantic_annotations[rule]` for a
    /// `branch_policy` directive; default `LongestMatch`. (Per-branch `@priority` / associativity beyond
    /// the defaults are the `.6` extension.)
    fn rule_branch_policy(&self, rule_name: &str) -> SemanticBranchPolicy {
        let Some(entries) = self
            .annotations
            .and_then(|a| a.semantic_annotations.get(rule_name))
        else {
            return SemanticBranchPolicy::LongestMatch;
        };
        let mut policy = SemanticBranchPolicy::LongestMatch;
        for annotation in entries {
            if let Some(name) = annotation.name() {
                if name.trim().eq_ignore_ascii_case("branch_policy") {
                    if let Some(parsed) =
                        SemanticBranchPolicy::parse(annotation.ast().payload_text().trim())
                    {
                        policy = parsed;
                    }
                }
            }
        }
        policy
    }
}

// ── Free helpers ─────────────────────────────────────────────────────────────────────────────────────

/// `body_has_single_element` — a body earns the synthetic `-> $1` passthrough default iff it is a
/// single-element `Sequence` (len ≤ 1) or a non-`Quantified` leaf/choice/lookahead. A `Quantified`
/// body never does (`$1` there means the whole capture group, which raw passthrough already yields).
fn synthesize_default_passthrough(body: &ASTNode) -> Option<BranchAnnotation> {
    let single = match body {
        ASTNode::Sequence { elements } => elements.len() <= 1,
        ASTNode::Quantified { .. } => false,
        _ => true,
    };
    if !single {
        return None;
    }
    Some(BranchAnnotation {
        annotation_type: "_pgen_default_passthrough_synthetic".to_string(),
        annotation_content: String::new(),
        parsed_ast: Some(UnifiedReturnAST::PositionalRef { index: 1 }),
    })
}

/// Peel `Alternative` wrappers one level at a time (the codegen's `__pgen_peel_alternative`).
fn peel_alternative(content: ParseContent<'_>) -> ParseContent<'_> {
    let mut current = content;
    while let ParseContent::Alternative(node) = current {
        current = node.content;
    }
    current
}

/// Integer-preserving number lowering (matches `AstReturnTransformer`): an integral `f64` becomes a
/// JSON integer, otherwise a JSON float.
fn number_to_json(value: f64) -> serde_json::Value {
    if value.is_finite()
        && value.fract() == 0.0
        && value >= i64::MIN as f64
        && value <= i64::MAX as f64
    {
        serde_json::Value::from(value as i64)
    } else {
        serde_json::Value::from(value)
    }
}

/// A process-global cache of anchored `\A(?:pattern)` regexes — the interpreter analogue of the
/// generated parser's per-parser `thread_local! REGEX_CACHE`, so a pattern is compiled once.
fn regex_cache() -> &'static Mutex<HashMap<String, regex::Regex>> {
    static CACHE: OnceLock<Mutex<HashMap<String, regex::Regex>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Run `f` with the compiled anchored regex for `pattern` (compiling + caching on first use).
fn with_anchored_regex<T>(
    pattern: &str,
    f: impl FnOnce(&regex::Regex) -> T,
) -> Result<T, regex::Error> {
    let mut cache = regex_cache().lock().expect("regex cache mutex poisoned");
    if !cache.contains_key(pattern) {
        let compiled = regex::Regex::new(&format!(r"\A(?:{pattern})"))?;
        cache.insert(pattern.to_string(), compiled);
    }
    let re = cache.get(pattern).expect("just inserted");
    Ok(f(re))
}

/// Whether the anchored `\A(?:pattern)` matches the empty string (the codegen's `can_match_empty`
/// probe that decides whether regex-layout consumes only horizontal whitespace).
fn regex_can_match_empty(pattern: &str) -> Result<bool, regex::Error> {
    with_anchored_regex(pattern, |re| {
        re.find("").map(|m| m.start() == 0 && m.end() == 0).unwrap_or(false)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intern_dedups_identical_strings_to_one_static() {
        let a = intern("interp_test_rule_alpha");
        let b = intern("interp_test_rule_alpha");
        assert!(std::ptr::eq(a, b), "identical interned strings must share one &'static str");
        assert_eq!(a, "interp_test_rule_alpha");
    }

    #[test]
    fn number_to_json_preserves_integer_typing() {
        assert_eq!(number_to_json(0.0), serde_json::json!(0));
        assert_eq!(number_to_json(42.0), serde_json::json!(42));
        assert_eq!(number_to_json(3.5), serde_json::json!(3.5));
    }

    #[test]
    fn synthesize_default_passthrough_matches_body_has_single_element() {
        // Single atom → synthesized $1.
        let atom = ASTNode::Atom {
            value: ASTValue::Token(vec![
                TokenValue::String("quoted_string".into()),
                TokenValue::String("x".into()),
            ]),
        };
        assert!(synthesize_default_passthrough(&atom).is_some());
        // Multi-element sequence → none.
        let seq = ASTNode::Sequence { elements: vec![atom.clone(), atom.clone()] };
        assert!(synthesize_default_passthrough(&seq).is_none());
        // Quantified → none.
        let quant = ASTNode::Quantified { element: Box::new(atom), quantifier: "+".into() };
        assert!(synthesize_default_passthrough(&quant).is_none());
    }

    #[test]
    fn peel_alternative_unwraps_to_the_core_content() {
        let inner = ParseContent::Terminal("core");
        let node = ParseNode { rule_name: "r", content: inner, span: 0..4 };
        let wrapped = ParseContent::Alternative(Box::new(node));
        assert_eq!(peel_alternative(wrapped), ParseContent::Terminal("core"));
    }

    #[test]
    fn regex_can_match_empty_detects_optional_vs_required() {
        assert!(regex_can_match_empty(r"\s*").unwrap(), r"\s* can match empty");
        assert!(!regex_can_match_empty(r"[0-9]+").unwrap(), "[0-9]+ cannot match empty");
    }

    // ── Differential integration tests (the `.4` acceptance oracle) ──────────────────────────────────

    /// The interpreter is byte-identical to the REGISTERED `json` parser (`parse_sample_ast_json`) —
    /// verdict + typed AST — across accept and reject inputs. This exercises the full `.4` core surface:
    /// multi-branch `Or` (longest_match), single-branch rules, `Sequence`, regex tokens, `rule_reference`
    /// recursion, the object/array/spread return-annotation fold, and layout. Fast (no compile).
    #[cfg(all(feature = "generated_parsers", feature = "ebnf_dual_run", has_generated_json_parser))]
    #[test]
    fn interpreter_is_byte_identical_to_the_json_registry_parser() {
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let grammar = manifest_dir.join("../grammars/json.ebnf");
        assert!(grammar.is_file(), "json grammar must exist at {}", grammar.display());

        let accepts = [
            r#"{"a": 1, "b": [true, null, "x"]}"#,
            r#"[1, 2, 3]"#,
            r#""hello""#,
            r#"true"#,
            r#"{"nested": {"deep": [42]}}"#,
            r#"[]"#,
            r#"{}"#,
        ];
        for input in accepts {
            let outcome = interpret_parse(&grammar, input, &InterpretOptions::default())
                .expect("interpret_parse plumbing must succeed");
            assert!(outcome.accepted, "interpreter must accept {input:?}: {outcome:?}");
            assert_eq!(
                crate::parser_registry::parse_sample("json", input),
                Some(true),
                "registry must agree {input:?} is accepted"
            );
            let registry_ast = crate::parser_registry::parse_sample_ast_json("json", input)
                .expect("json registered")
                .expect("registry AST on accept");
            assert_eq!(
                outcome.ast_json.as_ref(),
                Some(&registry_ast),
                "interpreter typed AST must be byte-identical to the registry's for {input:?}"
            );
        }

        let rejects = [r#"{"a": }"#, r#"[1,,2]"#, r#"{"a" 1}"#, r#"nul"#];
        for input in rejects {
            let outcome = interpret_parse(&grammar, input, &InterpretOptions::default())
                .expect("interpret_parse plumbing must succeed");
            assert!(!outcome.accepted, "interpreter must reject {input:?}: {outcome:?}");
            assert_eq!(outcome.ast_json, None);
            assert_eq!(
                crate::parser_registry::parse_sample("json", input),
                Some(false),
                "registry must agree {input:?} is rejected"
            );
        }
    }

    /// The interpreter agrees with the compile-and-run harness (`compile_and_parse`, approach 2 — the
    /// by-construction oracle) on synthetic per-combinator grammars: the ordered-choice fixed-prefix
    /// shape (the A2.3 case), quantifiers, and lookahead. This is a mini-`PARSE-HARNESS.5`. Slow (the
    /// first probe compiles `pgen`); uses a shared workdir so that cost is paid once.
    #[cfg(feature = "ebnf_dual_run")]
    #[test]
    fn interpreter_agrees_with_compile_and_run_on_synthetic_combinators() {
        use crate::parse_harness::{CompileAndParseOptions, compile_and_parse};

        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let ast_pipeline_bin = manifest_dir.join("target/debug/ast_pipeline");
        if !ast_pipeline_bin.is_file() {
            eprintln!(
                "skipping synthetic-combinator differential test: {} not built (needs `cargo build \
                 --features \"generated_parsers ebnf_dual_run\" --bin ast_pipeline`)",
                ast_pipeline_bin.display()
            );
            return;
        }

        let workdir = manifest_dir.join("target/parse_harness_it/interp");
        std::fs::create_dir_all(&workdir).expect("create shared workdir");
        let grammar_dir = workdir.join("grammars");
        std::fs::create_dir_all(&grammar_dir).expect("create grammar dir");

        // (grammar body, [(input, expected-accept-as-a-sanity-anchor)]). The real assertion is
        // interpreter == compile_and_parse, byte-for-byte; the anchors just prove the cases are live.
        let cases: &[(&str, &str, &[(&str, bool)])] = &[
            (
                "fixed_prefix",
                // The A2.3 fixed-terminal-prefix shape: longest_match must pick the LONGER alt on "ab".
                "start := \"a\" | \"a\" \"b\"\n",
                &[("ab", true), ("a", true), ("b", false), ("abc", false)],
            ),
            (
                "star_quant",
                "start := item*\nitem := \"x\"\n",
                &[("", true), ("x", true), ("xxx", true), ("xy", false)],
            ),
            (
                "plus_opt",
                "start := \"a\" item+ tail?\nitem := \"b\"\ntail := \"c\"\n",
                &[("ab", true), ("abbb", true), ("abbc", true), ("a", false)],
            ),
            (
                "neg_lookahead",
                "start := !\"x\" any\nany := \"y\" | \"z\"\n",
                &[("y", true), ("z", true), ("x", false)],
            ),
            (
                "pos_lookahead",
                // Positive lookahead `&`: the digit must be followed by `!`, then consumed by `rest`.
                "start := &digit rest\ndigit := \"1\" | \"2\"\nrest := digit \"!\"\n",
                &[("1!", true), ("2!", true), ("1", false), ("x!", false)],
            ),
            // NOTE: bounded `{N,M}` quantifiers are intentionally NOT a smoke case here — the EBNF
            // *input* surface does not accept `item{2,3}` in a grammar body end-to-end yet (the codegen
            // treats bounded operators as parser-agnostic infrastructure "available for future use"), so
            // the compile-and-run oracle cannot generate a parser for it. `parse_quantified` still honors
            // `{N,M}` bounds via `parse_quantifier_bounds`; it is simply unreachable from `.ebnf` syntax.
        ];

        let opts = CompileAndParseOptions {
            workdir: Some(workdir.clone()),
            keep_workdir: true,
            ..Default::default()
        };

        for (name, body, inputs) in cases {
            let grammar_path = grammar_dir.join(format!("{name}.ebnf"));
            std::fs::write(&grammar_path, body).expect("write synthetic grammar");
            for (input, expected_accept) in *inputs {
                let interp = interpret_parse(&grammar_path, input, &InterpretOptions::default())
                    .unwrap_or_else(|e| panic!("interpreter plumbing failed for {name}/{input:?}: {e}"));
                let oracle = compile_and_parse(&grammar_path, input, &opts)
                    .unwrap_or_else(|e| panic!("compile_and_parse oracle failed for {name}/{input:?}: {e}"));
                assert_eq!(
                    interp.accepted, oracle.accepted,
                    "verdict divergence {name}/{input:?}: interp={} oracle={}",
                    interp.accepted, oracle.accepted
                );
                assert_eq!(
                    interp.accepted, *expected_accept,
                    "sanity anchor wrong for {name}/{input:?} (interp={})",
                    interp.accepted
                );
                assert_eq!(
                    interp.furthest_position, oracle.furthest_position,
                    "furthest_position divergence {name}/{input:?}: interp={} oracle={}",
                    interp.furthest_position, oracle.furthest_position
                );
                assert_eq!(
                    interp.ast_json, oracle.ast_json,
                    "typed AST divergence {name}/{input:?}"
                );
            }
        }
    }
}
