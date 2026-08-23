//! PARSE-HARNESS.5 — the differential-equivalence gate driver.
//!
//! # What this is
//!
//! The certifying oracle that makes the grammar-AST interpreter (`PARSE-HARNESS.4`,
//! [`crate::parse_harness_interpreter`]) *authoritative by verification*: for a registered grammar it
//! runs **both** the interpreter and the shipped **generated parser** over the **same** deterministic
//! corpus and asserts they agree **byte for byte** — same accept/reject verdict and, on accept, the
//! **byte-identical typed AST**. Any divergence names the grammar, the exact input, and the diff.
//!
//! The corpus is the grammar's own **stimuli generator** output at fixed seeds (the repo's strongest
//! in-tree oracle of grammar-valid inputs) plus curated reject probes. Crucially, the interpreter and
//! the stimuli generator consume the **identical** normalized gen-AST triple
//! (`grammar_tree` / `rule_order` / `annotations`) that codegen consumed to build the generated parser,
//! so all three implementations are driven from one source of truth.
//!
//! # Report-first, never-panic
//!
//! [`evaluate_grammar_equivalence`] *collects* divergences into a [`GrammarEquivalenceReport`] rather
//! than asserting — so a caller can measure the honest state of every grammar in one pass (the
//! scoping/measurement use) *and* a gate test can assert `report.is_clean()` (the enforcement use).
//! This is the same "measure, then lock" discipline the rest of the platform uses.
//!
//! # Honest scope (`PARSE-HARNESS.md` §3.4 / §13.4)
//!
//! The interpreter core (`.4`) is byte-identical on the **structural + return-annotation** surface but
//! *defers* the semantic-directive orchestration that gates parse *outcomes* on the store
//! (`@predicate`/`@emit_fact`), packrat memoization (SV performance/termination), profile filtering
//! (SV dialect profiles), and non-default `branch_policy`/`@priority`. So the gate classifies each
//! registered grammar as **certified** (byte-identity required — a regression fails the gate) or
//! **deferred** (an explicit, reasoned allowlist tracked to `.6`), and — per the no-silent-caps
//! discipline — a deferred grammar that turns out to be byte-identical is itself a failure (it must be
//! *promoted* to certified). Nothing is silently skipped.

#![cfg(all(feature = "ebnf_dual_run", feature = "generated_parsers"))]

use std::collections::HashMap;
use std::path::Path;

use crate::ast_pipeline::stimuli_generator::{StimuliConfig, StimuliGenerator};
use crate::ast_pipeline::{ASTNode, Annotations, PipelineConfig, RustASTPipeline};
use crate::parse_harness_interpreter::interpret_parse_gen_ast;

/// The normalized gen-AST triple codegen (and the stimuli generator, and the interpreter) consume.
type GenAst = (HashMap<String, ASTNode>, Vec<String>, Option<Annotations>);

/// One input on which the interpreter and the generated parser disagreed.
#[derive(Debug, Clone)]
pub struct SampleDivergence {
    /// The input that diverged (truncated for reporting via [`truncate`]).
    pub sample: String,
    /// What kind of disagreement it was.
    pub kind: DivergenceKind,
    /// A human-readable detail (the two verdicts, or a compact AST-diff note).
    pub detail: String,
}

/// The category of a [`SampleDivergence`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DivergenceKind {
    /// The interpreter's *setup* plumbing failed (missing rule, serialization) — not a parse rejection.
    InterpreterPlumbing,
    /// accept/reject verdict differed.
    Verdict,
    /// Both accepted but the typed AST differed byte-for-byte.
    Ast,
    /// Both accepted but the oracle could not produce an AST (an oracle-side inconsistency).
    OracleAst,
}

/// The result of running the differential over one grammar's corpus.
#[derive(Debug, Clone)]
pub struct GrammarEquivalenceReport {
    pub grammar_name: String,
    /// A setup failure (grammar load / normalization / unregistered) that stopped the run entirely.
    pub load_error: Option<String>,
    /// Total corpus samples actually compared.
    pub samples_total: usize,
    /// Samples on which interpreter and generated parser agreed exactly.
    pub agreements: usize,
    /// Every recorded divergence (capped by [`EquivalenceConfig::max_recorded_divergences`]).
    pub divergences: Vec<SampleDivergence>,
    /// How many divergences were observed beyond the recording cap.
    pub divergences_suppressed: usize,
}

impl GrammarEquivalenceReport {
    /// A grammar is byte-identical iff it loaded, compared at least one sample, and had no divergence.
    pub fn is_clean(&self) -> bool {
        self.load_error.is_none()
            && self.samples_total > 0
            && self.divergences.is_empty()
            && self.divergences_suppressed == 0
    }

    /// A one-line human summary for the measurement report.
    pub fn summary_line(&self) -> String {
        if let Some(err) = &self.load_error {
            return format!("{:<32} LOAD-ERROR  {}", self.grammar_name, truncate(err, 160));
        }
        let verdict = if self.is_clean() { "CLEAN " } else { "DIVERGE" };
        let first = self
            .divergences
            .first()
            .map(|d| format!("  first={:?}: {}", d.kind, truncate(&d.detail, 120)))
            .unwrap_or_default();
        format!(
            "{:<32} {} samples={} agree={} diverge={}{}{}",
            self.grammar_name,
            verdict,
            self.samples_total,
            self.agreements,
            self.divergences.len(),
            if self.divergences_suppressed > 0 {
                format!(" (+{} suppressed)", self.divergences_suppressed)
            } else {
                String::new()
            },
            first,
        )
    }
}

/// Knobs for the differential run.
#[derive(Debug, Clone)]
pub struct EquivalenceConfig {
    /// Deterministic corpus seeds (the standard 0/7/42 trio for the gate).
    pub seeds: Vec<u64>,
    /// Valid samples generated per (seed × depth-rung).
    pub count_per_seed: usize,
    /// The stimuli-generation depth ladder. A *ladder* (not one cap) so a single config covers both
    /// shallow grammars (bottom out at a small depth) and deep-precedence grammars (regex/rtl_const_expr
    /// need a much larger minimal derivation) — and yields a construct-diverse corpus. Each rung is
    /// generated under [`generation_timeout_ms`](Self::generation_timeout_ms).
    pub depth_ladder: Vec<usize>,
    /// Per-sample deterministic generation step budget (ms→steps, machine-independent), passed to
    /// `generate_many_bounded`. Bounds the super-linear generators (rtl_const_expr/regex spin
    /// unboundedly at high depth without it — the same pathology the cert-coverage pass bounds).
    pub generation_timeout_ms: u64,
    /// Also probe reject-path parity by truncating each generated sample to its first half.
    pub include_truncation_probes: bool,
    /// Dialect profile passed to the generated-parser oracle (`Some("sv_2017")` for SV; `None` else).
    pub profile: Option<String>,
    /// Cap on recorded divergences per grammar (the rest are counted, not stored).
    pub max_recorded_divergences: usize,
}

impl Default for EquivalenceConfig {
    fn default() -> Self {
        Self {
            seeds: vec![0, 7, 42],
            count_per_seed: 8,
            depth_ladder: vec![6, 12, 18],
            generation_timeout_ms: 2000,
            include_truncation_probes: true,
            profile: None,
            max_recorded_divergences: 8,
        }
    }
}

/// Load + normalize a `.ebnf` into the gen-AST triple, exactly as [`crate::parse_harness_interpreter`]
/// does internally (so the interpreter, the stimuli generator, and — transitively — the generated
/// parser are all driven from the identical IR).
fn load_gen_ast(grammar_ebnf: &Path) -> Result<GenAst, String> {
    if !grammar_ebnf.is_file() {
        return Err(format!("grammar file not found: {}", grammar_ebnf.display()));
    }
    let path_str = grammar_ebnf.to_string_lossy().into_owned();
    let envelope = crate::ebnf_frontend::parse_ebnf_file_to_raw_ast_envelope(&path_str)
        .map_err(|e| format!("EBNF frontend: {e}"))?;
    let raw_ast = envelope
        .get("raw_ast")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "EBNF envelope has no `raw_ast` array".to_string())?;
    let pipeline = RustASTPipeline::new(PipelineConfig::default());
    pipeline
        .transform_from_raw_ast(raw_ast)
        .map_err(|e| format!("normalization: {e}"))
}

/// Add `s` to the corpus (deduped, order-stable) and — when `include_truncation_probes` — also its
/// first-half char-boundary truncation, so the corpus probes reject-path parity as well as accept paths.
fn push_sample_with_probes(
    samples: &mut Vec<String>,
    seen: &mut std::collections::HashSet<String>,
    s: &str,
    include_truncation_probes: bool,
) {
    if seen.insert(s.to_string()) {
        samples.push(s.to_string());
    }
    if include_truncation_probes {
        let half = s.len() / 2;
        if half > 0 {
            // Truncate on a char boundary.
            let mut cut = half;
            while cut > 0 && !s.is_char_boundary(cut) {
                cut -= 1;
            }
            if cut > 0 {
                let t = s[..cut].to_string();
                if seen.insert(t.clone()) {
                    samples.push(t);
                }
            }
        }
    }
}

/// Build the deterministic corpus for a grammar: `count_per_seed` stimuli-generated valid samples per
/// seed, plus (optionally) a first-half truncation of each to probe reject-path parity, plus any
/// hand-authored [`CURATED_CORPUS`] inputs for grammars the stimuli generator cannot cover within the
/// bounded depth ladder (PARSE-HARNESS.5.5). Deduplicated, order-stable.
fn build_corpus(grammar_name: &str, gen_ast: &GenAst, cfg: &EquivalenceConfig) -> Vec<String> {
    let (tree, order, annotations) = gen_ast;
    let mut samples: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    for &seed in &cfg.seeds {
        for &depth in &cfg.depth_ladder {
        let stimuli_config = StimuliConfig {
            seed: Some(seed),
            max_depth: depth,
            ..StimuliConfig::default()
        };
        let mut generator = StimuliGenerator::new(
            grammar_name.to_string(),
            tree,
            order,
            annotations.as_ref(),
            stimuli_config,
        );
        // Bounded generation: a depth rung a grammar cannot satisfy (too shallow) or that would spin
        // (too deep for a super-linear grammar) fails deterministically; other rungs still contribute.
        let batch = match generator.generate_many_bounded(
            cfg.count_per_seed,
            None,
            cfg.generation_timeout_ms,
        ) {
            Ok(v) => v,
            Err(_) => continue, // a generation failure for one rung is not a differential divergence
        };
        for s in batch {
            push_sample_with_probes(&mut samples, &mut seen, &s, cfg.include_truncation_probes);
        }
        }
    }
    // Append the grammar's curated inputs (if any). For a grammar like `rtl_const_expr` whose ~16-level
    // precedence chain the stimuli generator cannot bottom out within the bounded depth ladder (and whose
    // only generating depth window yields pathologically huge, hang-adjacent expressions — tool-established,
    // PARSE-HARNESS.5.5), these ARE the corpus; for a well-generated grammar this is a no-op. The
    // differential compares the interpreter against the AUTHORITATIVE generated parser, so curated INPUTS
    // carry no expected-output mirror risk (the oracle supplies the verdict + AST).
    if let Some(curated) = curated_corpus_for(grammar_name) {
        for &s in curated {
            push_sample_with_probes(&mut samples, &mut seen, s, cfg.include_truncation_probes);
        }
    }
    samples
}

/// A generous worker-thread stack for the differential. Recursive-descent parsing (both the
/// interpreter and the generated parser) can nest deeply on a pathologically-nested generated sample;
/// the default test-thread stack (~2 MiB) overflows and *aborts the process* well before the
/// interpreter's `max_depth` logical guard fires. Per the `recursion_ceiling_must_bound_the_real_stack`
/// discipline, we give the worker a large (virtual, not resident) stack so the logical guard — not an
/// OS stack fault — is what bounds recursion. 512 MiB is far above any realistic need and costs nothing
/// until touched.
pub const LARGE_STACK_BYTES: usize = 512 * 1024 * 1024;

/// Run `f` on a worker thread with [`LARGE_STACK_BYTES`] of stack and return its result.
pub fn run_on_large_stack<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::Builder::new()
        .stack_size(LARGE_STACK_BYTES)
        .spawn(f)
        .expect("spawn large-stack differential worker")
        .join()
        .expect("large-stack differential worker panicked")
}

/// [`evaluate_grammar_equivalence`] run on a [`LARGE_STACK_BYTES`] worker thread (owns its inputs). This
/// is the entry the measurement and the enforcing gate use, so a deeply-nested sample can never overflow
/// the caller's (small) stack and abort the run.
pub fn evaluate_grammar_equivalence_on_large_stack(
    grammar_name: &str,
    grammar_ebnf: &Path,
    cfg: &EquivalenceConfig,
) -> GrammarEquivalenceReport {
    let name = grammar_name.to_string();
    let ebnf = grammar_ebnf.to_path_buf();
    let cfg = cfg.clone();
    run_on_large_stack(move || evaluate_grammar_equivalence(&name, &ebnf, &cfg))
}

/// Run the interpreter-vs-generated-parser differential over one grammar's corpus and return the
/// [`GrammarEquivalenceReport`]. Never panics: a grammar that fails to load, or that is not registered,
/// yields a report with `load_error` set.
pub fn evaluate_grammar_equivalence(
    grammar_name: &str,
    grammar_ebnf: &Path,
    cfg: &EquivalenceConfig,
) -> GrammarEquivalenceReport {
    let mut report = GrammarEquivalenceReport {
        grammar_name: grammar_name.to_string(),
        load_error: None,
        samples_total: 0,
        agreements: 0,
        divergences: Vec::new(),
        divergences_suppressed: 0,
    };

    if !crate::parser_registry::supports_grammar(grammar_name) {
        report.load_error = Some("grammar is not present in the generated-parser registry".to_string());
        return report;
    }

    let gen_ast = match load_gen_ast(grammar_ebnf) {
        Ok(t) => t,
        Err(e) => {
            report.load_error = Some(e);
            return report;
        }
    };
    let (tree, order, annotations) = &gen_ast;

    let corpus = build_corpus(grammar_name, &gen_ast, cfg);
    let profile = cfg.profile.as_deref();
    // The ACTIVE (normalized) profile the oracle parses under — the interpreter must gate `@profiles`
    // rules against the SAME profile (an unspecified regex profile resolves to the grammar-declared
    // `@default_profile`, strict `pcre2` — DEFAULT-PROFILE.2) so it matches `parse_sample`
    // byte-for-byte (PARSE-HARNESS.5.1).
    let active_profile = crate::parser_registry::active_grammar_profile(grammar_name, profile);

    for sample in &corpus {
        report.samples_total += 1;

        // Interpreter side (approach 1). Pass the grammar name + active profile so the interpreter uses
        // the same layout/whitespace policy and `@profiles` gating the generated parser was built with
        // (PARSE-HARNESS.5.1).
        let interp = match interpret_parse_gen_ast(
            grammar_name,
            active_profile.as_deref(),
            tree,
            order,
            annotations.as_ref(),
            None,
            sample,
        ) {
            Ok(o) => o,
            Err(e) => {
                record(
                    &mut report,
                    cfg,
                    sample,
                    DivergenceKind::InterpreterPlumbing,
                    format!("interpreter setup error: {e}"),
                );
                continue;
            }
        };

        // Generated-parser oracle side (the authoritative registry path).
        let oracle_accepted =
            match crate::parser_registry::parse_sample_with_profile(grammar_name, sample, profile) {
                Some(b) => b,
                None => {
                    report.load_error =
                        Some("grammar became unregistered mid-run (unexpected)".to_string());
                    return report;
                }
            };

        // The interpreter reproduces the generated *grammar parse*; `parse_sample` additionally applies
        // the grammar's post-parse semantic contract (regex's PCRE2-fidelity check — e.g. it rejects a
        // quantifier on an anchor like `$+`, which the grammar accepts but PCRE2 rejects). Apply that
        // same contract to the interpreter's verdict so both sides are compared at the identical
        // "grammar parse + registry contract" layer a downstream consumer sees (PARSE-HARNESS.5.1). For
        // grammars with no contract this is a no-op (`Ok`), so `interp_accepted == interp.accepted`.
        let interp_accepted = interp.accepted
            && crate::parser_registry::post_parse_semantic_contract(grammar_name, sample).is_ok();

        if interp_accepted != oracle_accepted {
            record(
                &mut report,
                cfg,
                sample,
                DivergenceKind::Verdict,
                format!(
                    "interp.accepted={} (grammar-parse={}) oracle.accepted={} (interp furthest={})",
                    interp_accepted, interp.accepted, oracle_accepted, interp.furthest_position
                ),
            );
            continue;
        }

        if !oracle_accepted {
            // Both reject — agreement on the reject path. (furthest_position parity is a `.6` refinement;
            // the certified claim here is verdict + accept-path AST byte-identity.)
            report.agreements += 1;
            continue;
        }

        // Both accept → compare the typed AST byte-for-byte.
        let oracle_ast = crate::parser_registry::parse_sample_ast_json_with_profile(
            grammar_name,
            sample,
            profile,
        );
        match oracle_ast {
            Some(Ok(oracle_value)) => {
                if interp.ast_json.as_ref() == Some(&oracle_value) {
                    report.agreements += 1;
                } else {
                    record(
                        &mut report,
                        cfg,
                        sample,
                        DivergenceKind::Ast,
                        ast_diff_note(interp.ast_json.as_ref(), &oracle_value),
                    );
                }
            }
            Some(Err(e)) => {
                record(
                    &mut report,
                    cfg,
                    sample,
                    DivergenceKind::OracleAst,
                    format!("oracle accepted but produced no AST: {e}"),
                );
            }
            None => {
                report.load_error =
                    Some("grammar has no AST-JSON oracle variant (unexpected)".to_string());
                return report;
            }
        }
    }

    report
}

fn record(
    report: &mut GrammarEquivalenceReport,
    cfg: &EquivalenceConfig,
    sample: &str,
    kind: DivergenceKind,
    detail: String,
) {
    if report.divergences.len() < cfg.max_recorded_divergences {
        report.divergences.push(SampleDivergence {
            sample: truncate(sample, 160),
            kind,
            detail,
        });
    } else {
        report.divergences_suppressed += 1;
    }
}

/// A compact note describing how two accepted ASTs differ, without dumping megabytes.
fn ast_diff_note(interp: Option<&serde_json::Value>, oracle: &serde_json::Value) -> String {
    let interp = match interp {
        Some(v) => v,
        None => return "interpreter accepted but produced no AST".to_string(),
    };
    let i = serde_json::to_string(interp).unwrap_or_default();
    let o = serde_json::to_string(oracle).unwrap_or_default();
    // Find the first byte where the two serializations differ.
    let first_diff = i
        .as_bytes()
        .iter()
        .zip(o.as_bytes().iter())
        .position(|(a, b)| a != b)
        .unwrap_or_else(|| i.len().min(o.len()));
    format!(
        "AST differs at byte {first_diff} (interp_len={} oracle_len={}); interp…{}… vs oracle…{}…",
        i.len(),
        o.len(),
        truncate(context_from(&i, first_diff), 60),
        truncate(context_from(&o, first_diff), 60),
    )
}

/// A char-boundary-safe suffix of `s` starting ~30 bytes before `at` (serde_json emits raw UTF-8, so a
/// naive byte slice could split a multi-byte char and panic).
fn context_from(s: &str, at: usize) -> &str {
    let mut lo = at.saturating_sub(30).min(s.len());
    while lo < s.len() && !s.is_char_boundary(lo) {
        lo += 1;
    }
    &s[lo..]
}

/// Truncate a string for reporting, appending an ellipsis marker if cut.
pub fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let mut cut = max;
    while cut > 0 && !s.is_char_boundary(cut) {
        cut -= 1;
    }
    format!("{}…[+{}B]", &s[..cut], s.len() - cut)
}

/// The registered grammar → (`.ebnf` path relative to `grammars/`, dialect profile) table the
/// differential runs over. One source of truth for both the measurement and the enforcing gate.
pub const EQUIVALENCE_TARGETS: &[(&str, &str, Option<&str>)] = &[
    ("json", "json.ebnf", None),
    ("regex", "regex.ebnf", None),
    ("ebnf", "ebnf.ebnf", None),
    ("rtl_const_expr", "rtl_const_expr.ebnf", None),
    ("return_annotation", "return_annotation.ebnf", None),
    ("semantic_annotation", "semantic_annotation.ebnf", None),
    ("builtin_return_annotation", "builtin_return_annotation.ebnf", None),
    ("builtin_semantic_annotation", "builtin_semantic_annotation.ebnf", None),
    ("rtl_frontend", "rtl_frontend.ebnf", None),
    ("systemverilog_preprocessor", "systemverilog_preprocessor.ebnf", None),
    ("vhdl", "vhdl.ebnf", None),
    ("systemverilog", "systemverilog.ebnf", Some("sv_2017")),
    ("scratch", "scratch/scratch.ebnf", None),
];

/// **CERTIFIED** — grammars proven byte-identical (interpreter == generated parser: verdict + typed
/// AST) over the gate's deterministic stimuli corpus (seeds 0/7/42, the [`EquivalenceConfig::default`]
/// depth ladder). The gate FAILS if any of these regresses. Established by the PARSE-HARNESS.5
/// measurement (session #41); notably includes the store-using SV/VHDL/rtl_frontend, whose semantic
/// gates do not change the verdict/AST on this corpus (the deeper store-gated-outcome constructs are
/// the `.6` combinator+semantic suite's job — this is a corpus-scoped certification, not an all-inputs
/// proof; `PARSE-HARNESS.md` §3.4).
pub const CERTIFIED: &[&str] = &[
    "json",
    "semantic_annotation",
    "rtl_frontend",
    "vhdl",
    "systemverilog",
    "scratch",
    // Promoted from DEFERRED by PARSE-HARNESS.5.1 (the four regex-fidelity root causes: whitespace-sensitive
    // layout policy, unresolved-reference built-ins `builtin_any_char`/`builtin_ascii_char`, the `@transform`
    // numeric span coercion + the PCRE2 post-parse contract, and `@profiles` dialect gating). Byte-identical
    // over the gate corpus AND a deeper stress ladder (5 seeds, depths 6-30 — `probe_layout_deep_stress`).
    "regex",
    // Also promoted by PARSE-HARNESS.5.1: the same general fixes (the `systemverilog_preprocessor` regex-token
    // whitespace-sensitivity via the shared layout policy, + the built-ins) incidentally closed the `.5.4`
    // AST span/shape divergence. Byte-identical over the gate corpus AND the deep stress ladder (459 samples).
    "systemverilog_preprocessor",
    // Promoted from DEFERRED by PARSE-HARNESS.5.2: the interpreter's two layout skippers unconditionally
    // skipped all three comment introducers (`#`/`//`/`/*`), but codegen SUPPRESSES a comment arm per-grammar
    // when the grammar claims that introducer as a non-comment token (GRAMMAR-WELLFORMED.H.11.5). ebnf's
    // `block_comment := "/*" …` makes `"/*"` a real token, so the generated ebnf parser has NO `/*` layout
    // arm and rejects the comment-only `/**/`. The interpreter now gates each arm via codegen's OWN
    // predicate (`comment_arm_suppression_for_grammar`), so it matches the generated parser for every
    // grammar. Byte-identical over the gate corpus AND the deep stress ladder (`probe_layout_deep_stress`).
    "ebnf",
    // Promoted from DEFERRED by PARSE-HARNESS.5.3: the `_pgen_lr_chain` `wrapper_specs` blob (the
    // serialized per-alt `annotation_template`s of an LR-eliminated rule such as
    // `property_access_expression`) was serialized from a std `HashMap` (`UnifiedReturnAST::Object`),
    // whose per-instance iteration order is non-deterministic. Codegen froze ONE arbitrary order into the
    // generated parser; the interpreter re-serialized a fresh (and itself non-deterministic) order each
    // load — so the two typed ASTs diverged byte-for-byte on the dotted/array LR-eliminated shapes
    // (`$1.S15`, `$1[$1*]`). The `Object.properties` serializer now emits keys SORTED, canonicalizing
    // every serialization site at once (gen-AST, codegen-frozen literal, interpreter) AND removing a
    // latent codegen non-determinism (regens could otherwise freeze different orders). Byte-identical over
    // the gate corpus (68 samples).
    "return_annotation",
    // Promoted from DEFERRED by PARSE-HARNESS.5.5: NOT an interpreter fidelity fix — a corpus fix. The
    // stimuli generator produces ZERO usable samples for rtl_const_expr within the bounded depth ladder
    // (its ~16-level precedence chain needs `max_depth ≳ 30` to reach a leaf; the only generating depth
    // window emits pathologically huge, hang-adjacent expressions — tool-established via the CLI generation
    // sweep). A construct-complete hand-authored [`CURATED_CORPUS`] entry gives the differential inputs,
    // and the interpreter is byte-identical to the generated parser over it (verdict + typed AST) —
    // confirming the leaf's thesis that this was a corpus gap, not an interpreter divergence.
    "rtl_const_expr",
];

/// **DEFERRED** — registered grammars whose interpreter differential is NOT yet byte-identical, each
/// with the tool-established reason + owning follow-up leaf. The gate runs them and asserts they are
/// *still* divergent (the no-silent-caps discipline): if one becomes byte-identical it must be
/// *promoted* to [`CERTIFIED`], so the gate fails until it is — an honest ratchet, never a silent skip.
///
/// Now **empty**: every previously-deferred grammar has been promoted to [`CERTIFIED`] through the
/// per-grammar closures — `regex`+`systemverilog_preprocessor` (PARSE-HARNESS.5.1), `ebnf` (5.2),
/// `return_annotation` (5.3), and `rtl_const_expr` (5.5, via the curated corpus). The ratchet stays wired
/// (an empty list is a vacuous-but-honest guard) so a future newly-registered divergent grammar has a
/// place to land, and [`every_registered_grammar_is_classified_exactly_once`] keeps the classification
/// exhaustive.
pub const DEFERRED: &[(&str, &str)] = &[];

/// **EXCLUDED** — registered grammar names the differential does NOT apply to, because their registry
/// oracle is not a codegen parser *of their own `.ebnf`*: `builtin_return_annotation` aliases the
/// `return_annotation` parser (a different grammar), and `builtin_semantic_annotation` uses the
/// hand-rolled `UnifiedSemanticAST::parse_bootstrap`. Interpreting their `.ebnf` and comparing to those
/// oracles compares two different parsers — the differential's premise fails — so they are out of scope
/// by construction, not deferred.
pub const EXCLUDED: &[(&str, &str)] = &[
    (
        "builtin_return_annotation",
        "registry oracle is the return_annotation parser (a different grammar), not codegen of \
         builtin_return_annotation.ebnf",
    ),
    (
        "builtin_semantic_annotation",
        "registry oracle is the hand-rolled parse_bootstrap, not a codegen parser of \
         builtin_semantic_annotation.ebnf",
    ),
];

/// **CURATED_CORPUS** — hand-authored input corpora for grammars whose stimuli generator cannot produce
/// a usable corpus within the gate's bounded depth ladder. Parser-agnostic: keyed by grammar name, one
/// row per such grammar, kept next to the CERTIFIED/DEFERRED/EXCLUDED classification as one source of
/// truth. These are INPUTS only — the differential still compares the interpreter against the
/// authoritative generated parser (which supplies the verdict + typed AST), so a curated input carries no
/// "expected-output mirror" risk; it merely gives the differential something to compare (PARSE-HARNESS.5.5).
pub const CURATED_CORPUS: &[(&str, &[&str])] = &[
    ("rtl_const_expr", RTL_CONST_EXPR_CURATED),
    ("return_annotation", RETURN_ANNOTATION_CURATED),
];

/// GRAMMAR-WELLFORMED.H.22 — the DISCRIMINATING rows for the two codegen decisions the interpreter
/// does not mirror on `match_regex`.
///
/// ⛔ These exist because `return_annotation` was CERTIFIED byte-identical while a real divergence
/// sat inside it: codegen emits `match_regex("[^']*", false)` for `string_content_single` (the
/// `string_content_double`/`string_content_single` layout allowlist), the interpreter passes `true`,
/// and the generated stimuli corpus never produces the input that separates them. The gate was green
/// because the corpus did not reach the hole, not because the hole was closed — so the row comes
/// FIRST and the fix second. A fix whose gate cannot fail is not verified.
///
/// The discriminator is a quoted string whose CONTENT begins with layout: codegen hands the content
/// regex the bytes as written, the unmirrored interpreter skipped the leading horizontal whitespace
/// first (`[^']*` can match empty, so it takes `consume_layout_for_regex`'s early return) and
/// captured `abc` where the parser captured ` abc`. Measured before the fix, first diff at byte 317.
///
/// Each leading-layout row is paired with its no-layout CONTROL, so the pair proves the corpus can
/// return BOTH readings rather than being uniformly insensitive.
const RETURN_ANNOTATION_CURATED: &[&str] = &[
    // ── single-quoted content: the measured divergence, each with its control ──
    "-> {k: 'abc'}",
    "-> {k: ' abc'}",
    "-> {k: '\tabc'}",
    "-> {k: 'abc '}",
    "-> {k: '  '}",
    "-> {k: ''}",
    // ── double-quoted content: the sibling allowlist entry, same shape ──
    "-> {k: \"abc\"}",
    "-> {k: \" abc\"}",
    "-> {k: \"\tabc\"}",
    "-> {k: \"abc \"}",
    // ── the same content in the other positions a string_literal reaches ──
    "-> ' abc'",
    "-> \" abc\"",
    "-> [' abc', 'abc']",
    "-> {' k': 'v'}",
];

/// Construct-complete curated inputs for `rtl_const_expr` (PARSE-HARNESS.5.5). The ~16-level precedence
/// chain (`rtl_const_expr → conditional_expr → logical_or_expr → … → multiplicative_expr → unary_expr →
/// primary_expr → literal → decimal_integer`) needs `max_depth ≳ 30` just to reach a leaf, and the only
/// generating depth window (~32) emits pathologically huge, hang-adjacent expressions (tool-established),
/// so the stimuli generator yields ZERO usable samples on the bounded ladder. This list instead exercises
/// every construct directly, kept small so the no-memo interpreter stays fast: both literal kinds (incl.
/// underscores), plain/dotted/package-qualified identifiers, all four unary ops (incl. nesting), every
/// binary op at each of the 10 precedence levels + multi-term chains + mixed precedence, ternary (incl.
/// nesting), parentheses, whitespace/trivia variety, and near-miss rejects (both sides must agree on the
/// reject too).
const RTL_CONST_EXPR_CURATED: &[&str] = &[
    // ── literals — decimal (incl. underscores) and every based radix (b/o/d/h, signed, underscores) ──
    "0",
    "5",
    "42",
    "255",
    "1_000",
    "8'hFF",
    "4'b1010",
    "16'd255",
    "3'o7",
    "1'sb0",
    "8'hDE_AD",
    "2'b1_0",
    "12'HABC",
    // ── identifiers — plain, keyword-ish, `$`-bearing, dotted, package-qualified, and mixed ──
    "foo",
    "x",
    "_bar",
    "a$",
    "WIDTH",
    "a.b",
    "pkg.member.sub",
    "pkg::NAME",
    "a::b::c",
    "pkg::a.b",
    "a.b::c",
    // ── unary ops — each, plus nesting/stacking ──
    "+5",
    "-5",
    "!x",
    "~x",
    "- -5",
    "~~x",
    "!~x",
    "+-x",
    // ── binary ops — one per precedence level (multiplicative … logical_or) ──
    "a*b",
    "a/b",
    "a%b",
    "a+b",
    "a-b",
    "a<<b",
    "a>>b",
    "a<b",
    "a<=b",
    "a>b",
    "a>=b",
    "a==b",
    "a!=b",
    "a&b",
    "a^b",
    "a|b",
    "a&&b",
    "a||b",
    // ── multi-term chains (the `(op X)*` quantifier with >1 repetition) ──
    "a+b+c",
    "a*b*c*d",
    "1+2+3+4+5",
    "a|b|c",
    "a&&b&&c",
    "a-b-c-d",
    // ── mixed precedence (associativity + level interleaving) ──
    "a+b*c",
    "a*b+c",
    "a<<b+c",
    "a||b&&c",
    "a==b&&c==d",
    "a|b^c&d",
    "a+b<c",
    // ── ternary — flat and nested (right-associative via conditional_expr recursion) ──
    "a?b:c",
    "1?2:3",
    "a?b:c?d:e",
    "a?b?c:d:e",
    "N>0?N:1",
    // ── parentheses — grouping, nesting, override precedence ──
    "(a)",
    "((a))",
    "(a+b)",
    "(a+b)*c",
    "a*(b+c)",
    "(a?b:c)",
    // ── whitespace / trivia variety (spaces, tabs, newlines around tokens) ──
    "a + b",
    " a ",
    "a  *  b",
    "a\t*\tb",
    "1 +\n2",
    // ── realistic constant expressions ──
    "WIDTH-1",
    "8'hFF & MASK",
    "(A+B)*2",
    "pkg::WIDTH*2+1",
    "DEPTH<<1",
    "~MASK & DATA",
    // ── near-miss rejects (both interpreter and generated parser must REJECT) ──
    "a+",
    "(a",
    "a?b",
    "?:",
    "*a",
    "a b",
    "1 2",
];

/// Look up a grammar's curated input corpus, if any ([`CURATED_CORPUS`]).
pub fn curated_corpus_for(grammar_name: &str) -> Option<&'static [&'static str]> {
    CURATED_CORPUS
        .iter()
        .find(|(n, _)| *n == grammar_name)
        .map(|(_, s)| *s)
}

/// Look up a grammar's `.ebnf` path (relative to `grammars/`) + dialect profile from
/// [`EQUIVALENCE_TARGETS`].
pub fn target_for(grammar_name: &str) -> Option<(&'static str, Option<&'static str>)> {
    EQUIVALENCE_TARGETS
        .iter()
        .find(|(n, _, _)| *n == grammar_name)
        .map(|(_, file, profile)| (*file, *profile))
}

/// Evaluate one grammar's differential using the gate's standard corpus config (its `.ebnf` +
/// profile resolved from [`EQUIVALENCE_TARGETS`]), on a large-stack worker. `grammars_dir` is the repo
/// `grammars/` directory. Returns `None` if the grammar is not in the target table.
pub fn gate_evaluate(grammar_name: &str, grammars_dir: &Path) -> Option<GrammarEquivalenceReport> {
    let (file, profile) = target_for(grammar_name)?;
    let cfg = EquivalenceConfig {
        profile: profile.map(|s| s.to_string()),
        ..Default::default()
    };
    let ebnf = grammars_dir.join(file);
    Some(evaluate_grammar_equivalence_on_large_stack(grammar_name, &ebnf, &cfg))
}

#[cfg(test)]
mod measurement {
    use super::*;
    use std::path::PathBuf;

    /// PARSE-HARNESS.5 SCOUTING (not an assertion): print the interpreter-vs-generated-parser
    /// differential for every registered grammar so the certified/deferred split is measured, not
    /// guessed. Run with:
    ///   cargo test --features "generated_parsers ebnf_dual_run" --lib \
    ///     parse_harness_equivalence::measurement -- --nocapture --ignored
    /// Ignored by default (slow: it loads + stimuli-generates + differentially parses every grammar).
    #[test]
    #[ignore = "measurement/scouting probe — run explicitly with --ignored --nocapture"]
    fn measure_equivalence_across_registered_grammars() {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let grammars_dir = manifest.join("../grammars");

        // Optional single-grammar filter so a shell `timeout` can isolate one grammar (a no-memo
        // interpreter can be very slow on a deep sample for a recursion/store-heavy grammar).
        let only = std::env::var("PGEN_PHEQ_ONLY").ok();

        println!("\n=== PARSE-HARNESS.5 differential measurement (interpreter vs generated parser) ===");
        for (name, file, profile) in EQUIVALENCE_TARGETS {
            if let Some(f) = &only {
                if f != name {
                    continue;
                }
            }
            let cfg = EquivalenceConfig {
                profile: profile.map(|s| s.to_string()),
                ..Default::default()
            };
            let ebnf = grammars_dir.join(file);
            println!("[start {name}]");
            let report = evaluate_grammar_equivalence_on_large_stack(name, &ebnf, &cfg);
            println!("{}", report.summary_line());
            for d in report.divergences.iter().take(3) {
                println!("    · [{:?}] sample={:?}", d.kind, d.sample);
                println!("        {}", d.detail);
            }
            println!("[done {name}]");
        }
        println!("=== end measurement ===\n");
    }

    /// PARSE-HARNESS.5 SCOUTING: minimize the regex interpreter-vs-generated divergence. Prints, for
    /// each candidate, the interpreter verdict vs the generated regex parser verdict, so the exact
    /// diverging construct is isolated. Run with `--ignored --nocapture`.
    #[test]
    #[ignore = "measurement/scouting probe — run explicitly with --ignored --nocapture"]
    fn probe_regex_divergence_minimizer() {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let grammar = manifest.join("../grammars/regex.ebnf");
        let candidates = [
            r"a",
            r"a?",
            r"a??",
            r"\Q]\E",
            r"\Qa\E",
            r"\Qa\E?",
            r"\Qa\E??",
            r"\Qa\E+",
            r"\Q \E+",
            r"\Q]]\E",
            r"\Q]]\E+",
            r"[^]]",
            r"[^]]?",
            r"\A",
            r"\A?",
            r"\A??",
            r"a||b",
            r"a|",
            r"|",
            r"||",
            // PARSE-HARNESS.5.1 regression cases (the two root causes fixed this leaf):
            // (1) whitespace-sensitivity — the literal space between `*` and `?` must NOT be skipped,
            //     so `quant_suffix` stays empty (`greediness:[]`), not lazy.
            r"\Q]\E* ?",
            r"\Q]\E*?",
            r"a* ?",
            r"a b",
            // (2) the PCRE2 post-parse contract — a quantifier on an anchor is grammar-accepted but
            //     PCRE2-rejected, so `parse_sample` (and the interpreter+contract) must REJECT.
            r"$+",
            r"$*",
            r"^+",
            r"$+(?C)",
            r"\Q]\E?\Q]\E*\Q]\E*|$+(?C)",
        ];
        println!("\n=== regex divergence minimizer (interp | generated) ===");
        for input in candidates {
            let interp = run_on_large_stack({
                let g = grammar.clone();
                let inp = input.to_string();
                move || {
                    interpret_parse_gen_ast_from_ebnf(&g, &inp)
                }
            });
            // Reconcile at the same layer the gate uses: grammar-parse + the registry post-parse
            // contract (PARSE-HARNESS.5.1), so the anchor-quantifier cases show the effective verdict.
            let interp_verdict = match interp {
                Ok(o) => {
                    let effective = o.accepted
                        && crate::parser_registry::post_parse_semantic_contract("regex", input).is_ok();
                    format!("{effective}")
                }
                Err(e) => format!("ERR({e})"),
            };
            let gen_verdict = crate::parser_registry::parse_sample("regex", input);
            let flag = if format!("{:?}", gen_verdict.map(|b| b.to_string())).contains(&interp_verdict) {
                "  "
            } else {
                "≠≠"
            };
            println!("{flag} input={input:<12?} interp={interp_verdict:<10} generated={gen_verdict:?}");
        }
        println!("=== end regex minimizer ===\n");
    }

    /// PARSE-HARNESS.5.3 SCOUTING (not an assertion): dump the FULL interpreter vs generated-parser
    /// (`parse_sample_ast_json`) AST for the diverging `return_annotation` samples, and isolate the exact
    /// `wrapper_specs` string on each side. The `.5.3` divergence is an Object KEY-ORDER difference inside
    /// the `_pgen_lr_chain` `wrapper_specs` serialized blob for LR-eliminated rules (`$1.S15`, `$1[$1*]`).
    /// Run with `--ignored --nocapture`.
    #[test]
    #[ignore = "measurement/scouting probe — run explicitly with --ignored --nocapture"]
    fn probe_return_annotation_divergence() {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let grammar = manifest.join("../grammars/return_annotation.ebnf");
        let samples = ["$1.S15", "$1[$1*]**", "$1.S15K.vbn**"];
        println!("\n=== return_annotation .5.3 divergence dump (interp vs oracle) ===");
        for input in samples {
            let interp = run_on_large_stack({
                let g = grammar.clone();
                let inp = input.to_string();
                move || interpret_parse_gen_ast_from_ebnf(&g, &inp)
            });
            let interp_json = match &interp {
                Ok(o) => o.ast_json.clone(),
                Err(e) => {
                    println!("input={input:?} interp ERR({e})");
                    continue;
                }
            };
            let oracle_json =
                match crate::parser_registry::parse_sample_ast_json("return_annotation", input) {
                    Some(Ok(v)) => Some(v),
                    other => {
                        println!("input={input:?} oracle no-AST: {other:?}");
                        None
                    }
                };
            let ws = |v: &Option<serde_json::Value>| -> String {
                v.as_ref()
                    .and_then(|j| serde_json::to_string(j).ok())
                    .map(|s| {
                        // Isolate every `wrapper_specs` string occurrence for a clean side-by-side.
                        s.match_indices("wrapper_specs")
                            .map(|(i, _)| {
                                let tail = &s[i..];
                                truncate(tail, 180).to_string()
                            })
                            .collect::<Vec<_>>()
                            .join("\n        ")
                    })
                    .unwrap_or_else(|| "<none>".to_string())
            };
            let identical = interp_json == oracle_json;
            println!("\ninput={input:?}  identical={identical}");
            println!("    interp wrapper_specs: {}", ws(&interp_json));
            println!("    oracle wrapper_specs: {}", ws(&oracle_json));
        }
        println!("=== end return_annotation .5.3 dump ===\n");
    }

    /// DEEP STRESS (scouting, not an assertion): run the differential over a DEEPER + WIDER corpus than
    /// the gate (ladder up to 30, 16/rung, 5 seeds) to build confidence that a certification is not an
    /// artifact of the shallow gate ladder before promoting. Covers the grammars the layout/fidelity
    /// closures certified — `regex` + `systemverilog_preprocessor` (PARSE-HARNESS.5.1) and `ebnf`
    /// (PARSE-HARNESS.5.2, the per-introducer comment-arm suppression). Add a new target here when a
    /// grammar is promoted through a layout/fidelity fix; filter with `PGEN_PHEQ_ONLY`. Run with
    /// `--ignored --nocapture`.
    #[test]
    #[ignore = "measurement/scouting probe — run explicitly with --ignored --nocapture"]
    fn probe_layout_deep_stress() {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let grammars_dir = manifest.join("../grammars");
        let only = std::env::var("PGEN_PHEQ_ONLY").ok();
        // Per-target ladder. The whitespace-flat grammars (`regex` / `systemverilog_preprocessor`) stress
        // cheaply to depth 30 × 5 seeds. `ebnf` is deeply RECURSIVE (`grammar_rule` → `expression` → …), so
        // the no-memo interpreter is super-linear PAST depth ~18 (the §`.6` memoization is the fix) — a
        // deeper depth ladder does not complete in practical time. So `ebnf`'s deep-stress instead widens
        // the SEED coverage at the gate depths (5 seeds vs the gate's 3), which completes; its certification
        // proper is the gate (`[6,12,18]` × seeds 0/7/42, 83 samples).
        let targets: &[(&str, &str, &[usize])] = &[
            ("regex", "regex.ebnf", &[6, 12, 18, 24, 30]),
            (
                "systemverilog_preprocessor",
                "systemverilog_preprocessor.ebnf",
                &[6, 12, 18, 24, 30],
            ),
            ("ebnf", "ebnf.ebnf", &[6, 12, 18]),
        ];
        println!("\n=== deep stress (per-target ladder, 5 seeds, 16/rung) ===");
        for (name, file, ladder) in targets {
            if let Some(f) = &only {
                if f != name {
                    continue;
                }
            }
            let cfg = EquivalenceConfig {
                seeds: vec![0, 7, 42, 101, 2024],
                count_per_seed: 16,
                depth_ladder: ladder.to_vec(),
                max_recorded_divergences: 12,
                ..Default::default()
            };
            let report =
                evaluate_grammar_equivalence_on_large_stack(name, &grammars_dir.join(file), &cfg);
            println!("{}", report.summary_line());
            for d in report.divergences.iter().take(12) {
                println!("    · [{:?}] sample={:?}\n        {}", d.kind, d.sample, d.detail);
            }
        }
        println!("=== end deep stress ===\n");
    }

    /// Small helper: load + interpret in one call (mirrors `interpret_parse` but reusable in probes).
    fn interpret_parse_gen_ast_from_ebnf(
        grammar_ebnf: &std::path::Path,
        input: &str,
    ) -> Result<crate::parse_harness::ParseOutcome, String> {
        let gen_ast = load_gen_ast(grammar_ebnf)?;
        let (tree, order, annotations) = &gen_ast;
        // Grammar name = `.ebnf` file stem, so the interpreter picks the same layout policy + `@profiles`
        // gating the generated parser was built with (PARSE-HARNESS.5.1).
        let grammar_name = grammar_ebnf
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        let active_profile = crate::parser_registry::active_grammar_profile(grammar_name, None);
        interpret_parse_gen_ast(
            grammar_name,
            active_profile.as_deref(),
            tree,
            order,
            annotations.as_ref(),
            None,
            input,
        )
        .map_err(|e| e.to_string())
    }
}

// ── The enforcing gate (PARSE-HARNESS.5 `parse_harness_equivalence_gate`) ────────────────────────────
//
// Runs only with BOTH `generated_parsers` (the registry oracle) and `ebnf_dual_run` (the `.ebnf`
// loader), so a default `cargo test` is unaffected; the `make parse_harness_equivalence_gate` target
// builds with both. Deterministic: fixed seeds 0/7/42, bounded generation, large-stack workers.
#[cfg(test)]
mod gate {
    use super::*;
    use std::path::PathBuf;

    fn grammars_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../grammars")
    }

    /// COMPLETENESS: every grammar in the generated-parser registry must be classified exactly once
    /// (CERTIFIED xor DEFERRED xor EXCLUDED). A newly-registered grammar left unclassified fails here,
    /// so the differential can never silently omit a grammar.
    #[test]
    fn every_registered_grammar_is_classified_exactly_once() {
        let deferred: Vec<&str> = DEFERRED.iter().map(|(n, _)| *n).collect();
        let excluded: Vec<&str> = EXCLUDED.iter().map(|(n, _)| *n).collect();
        for g in crate::parser_registry::registered_grammars() {
            let in_c = CERTIFIED.contains(&g);
            let in_d = deferred.contains(&g);
            let in_e = excluded.contains(&g);
            let count = [in_c, in_d, in_e].iter().filter(|b| **b).count();
            assert_eq!(
                count, 1,
                "registered grammar {g:?} must be classified exactly once \
                 (CERTIFIED={in_c} DEFERRED={in_d} EXCLUDED={in_e}) — add it to one of the \
                 PARSE-HARNESS.5 classification lists"
            );
        }
        // And every classified name must actually be registered + have a target entry (no dangling).
        for g in CERTIFIED.iter().chain(deferred.iter()).chain(excluded.iter()) {
            assert!(
                crate::parser_registry::supports_grammar(g),
                "classified grammar {g:?} is not in the registry"
            );
        }
        for g in CERTIFIED.iter().chain(deferred.iter()) {
            assert!(target_for(g).is_some(), "grammar {g:?} has no EQUIVALENCE_TARGETS entry");
        }
    }

    /// PARSE-HARNESS.5.2: the interpreter skips comment layout via codegen's own per-introducer
    /// suppression predicate (`comment_arm_suppression_for_grammar`). Pin its result for every registered
    /// grammar against the ground truth read directly from the shipped `generated/*.rs`
    /// `consume_layout_for_terminal` arms (the `(#, //, /*)` columns), so a codegen predicate change that
    /// would desync the interpreter from the generated parsers fails HERE (in addition to the differential
    /// gate). `true` = the grammar claims that introducer as a non-comment token ⇒ its layout arm is
    /// suppressed.
    #[test]
    fn comment_arm_suppression_matrix_is_pinned() {
        use crate::ast_pipeline::ast_based_generator::comment_arm_suppression_for_grammar;
        // (grammar, file, (claims_hash, claims_line_comment, claims_block_comment)) — read from the
        // generated parsers' `consume_layout_for_terminal`: an ABSENT arm ⇒ claimed ⇒ `true`.
        let expected: &[(&str, &str, (bool, bool, bool))] = &[
            ("json", "json.ebnf", (false, false, false)),
            ("regex", "regex.ebnf", (true, false, false)),
            ("vhdl", "vhdl.ebnf", (true, false, false)),
            ("systemverilog", "systemverilog.ebnf", (true, false, false)),
            ("rtl_frontend", "rtl_frontend.ebnf", (true, false, false)),
            (
                "systemverilog_preprocessor",
                "systemverilog_preprocessor.ebnf",
                (false, false, false),
            ),
            ("rtl_const_expr", "rtl_const_expr.ebnf", (false, false, false)),
            // GRAMMAR-WELLFORMED.H.16.2: `#` flipped false -> true. `set_value := "#{" /\s*/ …`
            // always WAS a non-comment claim on `#`; the claim was dropped because the analysis
            // scored its `/\s*/` SEPARATOR as a comment content TAIL, so the engine's `#`-to-EOL
            // arm stayed and ate `#{` on every input. This grammar defines no `#` comment rule.
            ("semantic_annotation", "semantic_annotation.ebnf", (true, true, true)),
            ("return_annotation", "return_annotation.ebnf", (false, false, false)),
            ("ebnf", "ebnf.ebnf", (false, false, true)),
        ];
        let dir = grammars_dir();
        let mut failures: Vec<String> = Vec::new();
        for (name, file, (eh, el, eb)) in expected {
            let (tree, _order, annotations) =
                load_gen_ast(&dir.join(file)).expect("grammar loads");
            let sup = comment_arm_suppression_for_grammar(name, &tree, annotations.as_ref());
            let got = (sup.claims_hash, sup.claims_line_comment, sup.claims_block_comment);
            if got != (*eh, *el, *eb) {
                failures.push(format!(
                    "  {name}: expected (#,//,/*)=({eh},{el},{eb}) but got ({},{},{})",
                    sup.claims_hash, sup.claims_line_comment, sup.claims_block_comment
                ));
            }
        }
        assert!(
            failures.is_empty(),
            "PARSE-HARNESS.5.2: comment-arm suppression desynced from the generated parsers — the \
             interpreter would skip comment layout differently than codegen emits:\n{}",
            failures.join("\n")
        );
    }

    /// CERTIFIED: each of these must be byte-identical (verdict + typed AST) between the interpreter and
    /// the generated parser over the deterministic corpus. A regression (any divergence, or an empty
    /// corpus) fails the gate.
    #[test]
    fn certified_grammars_are_byte_identical() {
        let dir = grammars_dir();
        let mut failures: Vec<String> = Vec::new();
        for &g in CERTIFIED {
            let report = gate_evaluate(g, &dir).expect("certified grammar has a target");
            if !report.is_clean() {
                failures.push(format!("  {}", report.summary_line()));
                for d in report.divergences.iter().take(3) {
                    failures.push(format!("      · [{:?}] {} :: {}", d.kind, truncate(&d.sample, 100), d.detail));
                }
            }
        }
        assert!(
            failures.is_empty(),
            "PARSE-HARNESS.5: CERTIFIED grammar(s) regressed — interpreter no longer byte-identical to \
             the generated parser:\n{}",
            failures.join("\n")
        );
    }

    /// DEFERRED ratchet: each deferred grammar must be OBSERVABLY not-yet-certifiable over this corpus.
    /// If one is now byte-identical, the gate fails demanding it be *promoted* to CERTIFIED — so
    /// progress is never lost silently. (`rtl_const_expr`'s zero corpus counts as not-clean, honestly.)
    #[test]
    fn deferred_grammars_are_still_divergent_or_promote_them() {
        let dir = grammars_dir();
        let mut promotable: Vec<String> = Vec::new();
        for (g, reason) in DEFERRED {
            let report = gate_evaluate(g, &dir).expect("deferred grammar has a target");
            if report.is_clean() {
                promotable.push(format!("  {g}  (was deferred: {reason})"));
            }
        }
        assert!(
            promotable.is_empty(),
            "PARSE-HARNESS.5: deferred grammar(s) are now byte-identical — PROMOTE them to CERTIFIED \
             (move from DEFERRED to CERTIFIED and update the tree):\n{}",
            promotable.join("\n")
        );
    }
}
