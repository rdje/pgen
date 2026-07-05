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

use crate::ast_pipeline::semantic_directive_registry::SemanticBranchPolicy;
use crate::ast_pipeline::unified_return_ast::{ExtractionTarget, UnifiedReturnAST};
use crate::ast_pipeline::{
    ASTNode, ASTValue, Annotations, BranchAnnotation, ParseContent, ParseError, ParseNode,
    ParseResult, SemanticRuntimeState, TokenValue, parse_quantifier_bounds,
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

    interpret_parse_gen_ast(
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
pub fn interpret_parse_gen_ast(
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

    let mut interp = Interp {
        grammar: grammar_tree,
        annotations,
        input,
        position: 0,
        furthest_position: 0,
        depth: 0,
        max_depth: 2000,
        semantic_state: SemanticRuntimeState::new(),
    };

    // Mirror `parse_full`: parse the entry rule, consume trailing layout, then require the whole input
    // was consumed. A prefix-only parse is a REJECT (accepted == false), not an error.
    let outcome = match interp.parse_rule(&entry_rule) {
        Ok(node) => {
            interp.consume_layout_for_terminal("<EOF>");
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

/// The dynamic-dispatch interpreter state. `'g` = the borrowed gen-AST; `'i` = the input string.
struct Interp<'g, 'i> {
    grammar: &'g HashMap<String, ASTNode>,
    annotations: Option<&'g Annotations>,
    input: &'i str,
    position: usize,
    /// The deepest input byte any branch reached (even after backtracking) — updated monotonically at
    /// each rule entry, exactly as the generated parser does. This is the A2.2/A2.3-grade reject locus.
    furthest_position: usize,
    depth: usize,
    max_depth: usize,
    /// Reused VERBATIM (Section A): speculation snapshots/restores it, faithfully, so the semantic
    /// orchestration extends here in `.6` without restructuring. For the `.4` structural smoke set
    /// (no `@emit_fact`) it stays empty, so it never perturbs the typed AST.
    semantic_state: SemanticRuntimeState,
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
        // furthest_position is updated at rule entry, monotonically (never decremented on backtrack).
        if self.position > self.furthest_position {
            self.furthest_position = self.position;
        }

        let body = self.grammar.get(rule_name).ok_or(ParseError::InvalidSyntax {
            message: "interpreter: reference to undefined rule",
            position: start_pos,
        })?;

        let content = self.parse_rule_body(body, rule_name, start_pos)?;
        let end_pos = self.position;
        Ok(ParseNode {
            rule_name: intern(rule_name),
            content,
            span: start_pos..end_pos,
        })
    }

    /// Produce the rule's final (post-transform) `ParseContent`. An `Or` body applies its
    /// return-annotation transform per-branch inside the tournament; a non-`Or` body gets the
    /// rule-level branch-0 transform (explicit annotation, else the synthetic `-> $1` passthrough for a
    /// single-element body, else raw).
    fn parse_rule_body(
        &mut self,
        body: &'g ASTNode,
        rule_name: &str,
        start_pos: usize,
    ) -> ParseResult<ParseContent<'i>> {
        match body {
            ASTNode::Or { alternatives } => self.parse_or(alternatives, rule_name),
            _ => {
                let raw = self.parse_node(body, rule_name)?;
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
    fn parse_node(&mut self, node: &'g ASTNode, rule_name: &str) -> ParseResult<ParseContent<'i>> {
        match node {
            ASTNode::Or { alternatives } => self.parse_or(alternatives, rule_name),
            ASTNode::Sequence { elements } => self.parse_sequence(elements, rule_name),
            ASTNode::Atom { value } => self.parse_atom(value, rule_name),
            ASTNode::Quantified { element, quantifier } => {
                self.parse_quantified(element, quantifier, rule_name)
            }
            ASTNode::Lookahead { element, positive } => {
                self.parse_lookahead(element, *positive, rule_name)
            }
        }
    }

    // ── Ordered choice (the tournament) ────────────────────────────────────────────────────────────

    /// Ordered choice honoring `branch_policy`. Single-branch: parse + branch-0 transform. Multi-branch:
    /// the try_parse tournament — under the default `longest_match`, strictly-greater `candidate_end`
    /// replaces the incumbent and an equal end keeps the earliest branch (the exact `should_take`
    /// ladder the generated parser emits, for the default policy/priority/associativity).
    fn parse_or(
        &mut self,
        alternatives: &'g [ASTNode],
        rule_name: &str,
    ) -> ParseResult<ParseContent<'i>> {
        if alternatives.len() == 1 {
            let branch_start = self.position;
            let raw = self.parse_node(&alternatives[0], rule_name)?;
            let ann = self.resolve_branch_annotation(rule_name, 0, &alternatives[0]);
            return Ok(match ann {
                Some(a) => self.apply_return_annotation(&a, &raw, branch_start),
                None => raw,
            });
        }

        let policy = self.rule_branch_policy(rule_name);
        let parse_start = self.position;

        let mut best_content: Option<ParseContent<'i>> = None;
        let mut best_end = parse_start;

        for (idx, alternative) in alternatives.iter().enumerate() {
            // `ordered` keeps the first successful branch.
            if policy == SemanticBranchPolicy::Ordered && best_content.is_some() {
                continue;
            }
            self.position = parse_start;
            let branch_start = self.position;
            let attempt = self.try_parse(|p| p.parse_node(alternative, rule_name));
            if let Some(raw) = attempt {
                let candidate_end = self.position;
                let ann = self.resolve_branch_annotation(rule_name, idx, alternative);
                let transformed = match ann {
                    Some(a) => self.apply_return_annotation(&a, &raw, branch_start),
                    None => raw,
                };
                // Default longest_match ladder (priority/associativity are default here — `.6` extends):
                //   first candidate wins; strictly-longer replaces; equal end keeps the earlier branch.
                let take = match policy {
                    SemanticBranchPolicy::Ordered => best_content.is_none(),
                    // priority_first with all-default priorities degenerates to first-wins-then-longest;
                    // the full priority/associativity ladder is the `.6` extension (honest bound §13.4).
                    _ => best_content.is_none() || candidate_end > best_end,
                };
                if take {
                    best_end = candidate_end;
                    best_content = Some(transformed);
                }
            }
        }

        match best_content {
            Some(content) => {
                self.position = best_end;
                Ok(content)
            }
            None => Err(ParseError::Backtrack { position: parse_start }),
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
    ) -> ParseResult<ParseContent<'i>> {
        let mut sequence_elements: Vec<ParseNode<'i>> = Vec::with_capacity(elements.len());
        for (idx, element) in elements.iter().enumerate() {
            let element_start = self.position;
            let element_content = match element {
                ASTNode::Quantified { element: inner, quantifier } if quantifier == "?" => {
                    // Optional sequence element: inner content on match, empty Sequence on absence.
                    match self.try_parse(|p| p.parse_node(inner, rule_name)) {
                        Some(content) => content,
                        None => ParseContent::Sequence(Vec::new()),
                    }
                }
                _ => self.parse_node(element, rule_name)?,
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
            match self.try_parse(|p| p.parse_node(element, rule_name)) {
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
    ) -> ParseResult<ParseContent<'i>> {
        let lookahead_start = self.position;
        let matched = self.try_parse(|p| p.parse_node(element, rule_name));
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
        self.consume_layout_for_terminal(expected);
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
        if skip_leading_whitespace {
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
