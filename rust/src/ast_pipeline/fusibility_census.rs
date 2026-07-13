//! RGX-0078.5.h.1 — STEP-0 FUSIBILITY CENSUS (read-only analysis; no codegen change).
//!
//! The capability-gate classifier for the DERIVED-SCANNER rung (`RGX-0078.5.h`, tree
//! section F): for every rule of a grammar, decide whether its subtree is
//! **scanner-compilable under the increment-1 strict gate** — i.e. whether a
//! codegen-emitted direct-coded DFA `scan_R(pos) -> (accept_tag, len)` could replace the
//! rule-cascade descent at `parse_R` call sites while rebuilding the SAME AST value.
//!
//! The gate (section D/F of `docs/tasks/RGX-0078.md`), applied CONSERVATIVELY — every
//! verdict here is a sound UNDER-approximation of fusibility, so the measured ceiling
//! never over-promises:
//!
//! 1. **Regular**: the rule's reference closure is acyclic (iteration only via
//!    quantifiers); every atom is a terminal, a regex-literal token, a char builtin
//!    (`builtin_any_char` / `builtin_ascii_char`), or a reference to a fusible rule.
//! 2. **Effect-free**: no runtime semantic directive (`@predicate` any phase,
//!    `@emit_fact`, `@open_scope`/`@close_scope`, library import/export, …) anywhere in
//!    the closure; no value constraints (`@enum`/`@regex`/`@range`/`@len`), no lexical
//!    follow restrictions, no mid-sequence inline actions, no `@transform`, and no
//!    `@profiles` dialect gate (a fused subtree would inline a gated rule's language
//!    without its runtime profile guard — an increment-2 concern, flagged distinctly).
//! 3. **Text-folding** (increment-1 "strictest"): the rule's AST contribution is exactly
//!    the matched text — an unannotated body whose branches fold to terminal text, or a
//!    positional `-> $N` passthrough whose sibling elements are all zero-width lookaheads
//!    (the `unicode_char := !builtin_ascii_char builtin_any_char -> $2` shape).
//! 4. **Choice semantics DFA-encodable**: `longest_match` (maximal munch), `ordered` /
//!    `priority_first` (accept-priority tie-break), `@associativity` `left`/`right`
//!    (tie direction); `nonassoc` (equal-tie fails the whole choice) and
//!    `@deterministic_group` evaluation-order rotation are excluded conservatively.
//! 5. **Layout respected at the boundary**: under a layout-SKIPPING policy facet
//!    (`match_string` skips iff `!layout.terminals`; `match_regex` skips iff
//!    `!layout.regex_tokens` — mirroring `ast_based_generator.rs`), a fused subtree must
//!    consume at most ONE lexeme (the boundary skip is hoistable to the scan call site);
//!    multi-lexeme bodies would embed skipped trivia inside the scan and are excluded.
//!    Fully whitespace-sensitive facets (regex, svpp) make any arity contiguous.
//!
//! Output: per-rule verdicts (tier + disqualifying reasons), the MAXIMAL fusible roots
//! (fusible rules referenced by at least one non-fusible rule — the future `scan_*` call
//! sites), and, when joined with per-parse rule-entry counts (written by
//! `parseability_probe --dump-rule-entry-counts-json`), the measured share of bench rule
//! ENTRIES eliminated by fusion — the census number that gates `.5.h.2+` DFA emission.

use super::ast_based_generator::AstBasedGenerator;
use super::semantic_directive_registry::{
    effective_rule_associativity, effective_rule_branch_policy,
    effective_rule_deterministic_partition_policy, effective_rule_value_constraints,
    SemanticAssociativity,
};
use super::semantic_runtime::{
    compile_semantic_runtime_annotations, CompiledSemanticRuntimeAnnotations, LayoutSensitivity,
    SemanticRuntimeDirective,
};
use super::{
    parse_quantifier_bounds, ASTNode, ASTValue, Annotations, TokenValue, UnifiedReturnAST,
};
use std::collections::{BTreeMap, HashMap, HashSet};

/// The per-atom layout mirror of `ast_based_generator.rs`: these two rule names never
/// skip leading layout before a regex token regardless of policy (the generator's
/// `skip_leading_whitespace = !matches!(rule_name, …)` special case).
const REGEX_ATOM_NO_SKIP_RULES: &[&str] = &["string_content_double", "string_content_single"];

/// Classification tier of one rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FusibilityTier {
    /// Increment-1 strict: regular + effect-free + text-folding + encodable + contiguous,
    /// with NO lookahead anywhere in the closure.
    FusibleToken,
    /// Same gate but the closure uses `&`/`!` lookahead (DFA-encodable via product /
    /// complement construction — e.g. the `unicode_char` char-complement shape); tracked
    /// as its own tier so DFA-emission increments can sequence it separately.
    FusibleLookahead,
    /// Fails at least one gate criterion; see `reasons`.
    NotFusible,
}

/// One rule's census verdict.
#[derive(Debug, Clone, serde::Serialize)]
pub struct RuleCensus {
    pub tier: FusibilityTier,
    /// Disqualifying reasons (empty for fusible rules). Deduped, insertion-ordered,
    /// shallow: local causes plus the names of directly-referenced non-fusible rules.
    pub reasons: Vec<String>,
    /// Fusible AND (the entry rule, referenced by ≥1 non-fusible rule, or unreferenced):
    /// a maximal fused subtree root — the site a `scan_*` call would replace.
    pub maximal_root: bool,
    pub uses_lookahead: bool,
    /// The value-agnostic half of the gate passed (regular + effect-free +
    /// policy-encodable + layout-contiguous). A `shape_encodable` rule that is NOT
    /// fusible fails only the increment-1 text-folding value gate — the candidate pool
    /// for a later value-building fusion increment.
    pub shape_encodable: bool,
}

/// The measured entry-share join (census × per-parse rule-entry counts).
#[derive(Debug, Clone, serde::Serialize)]
pub struct EntryShare {
    pub count_files: usize,
    pub total_entries: u64,
    /// Entries on fusible NON-root rules — the cascade a fused scan eliminates outright.
    pub eliminated_below_roots: u64,
    /// Entries on maximal fusible roots — each becomes one `scan_*` call.
    pub at_roots: u64,
    /// Entries on non-fusible rules — untouched by increment-1 fusion.
    pub untouched: u64,
    /// Of `untouched`: entries on SHAPE-ENCODABLE rules (only the text-folding value
    /// gate fails) — the measured upside of a later value-building fusion increment.
    pub shape_only_entries: u64,
    /// Rule names present in the counts files but absent from the census (e.g.
    /// LR-elimination-synthesized helpers). Never silently dropped.
    pub unmatched_rules: Vec<String>,
    pub unmatched_entries: u64,
    /// `total / (total - eliminated_below_roots)` under the uniform per-entry cost model
    /// (~262ns/rule-entry, RGX-0078.5.e §A), first-order scan cost ≈ a terminal match.
    pub ceiling_estimate: f64,
}

/// The whole-grammar census.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FusibilityCensus {
    pub grammar_name: String,
    pub total_rules: usize,
    pub fusible_token: usize,
    pub fusible_lookahead: usize,
    pub not_fusible: usize,
    /// Not fusible under increment 1, but the value-agnostic shape gate passes — the
    /// rule's LANGUAGE is scanner-encodable and only the text-folding value gate fails.
    pub shape_only: usize,
    /// Regex-literal terminal atom SITES (`/.../` atoms compiled to `match_regex` calls
    /// through the external Rust regex engine). The derived scanner's OTHER surface:
    /// each site is a de-facto leaf-scanner call whose engine a PGEN-derived DFA would
    /// replace (platform-wide self-hosting, RGX-0078 tree §E item 6) — independent of
    /// rule-level fusion.
    pub regex_atom_sites: usize,
    /// Distinct regex-literal patterns behind `regex_atom_sites`.
    pub distinct_regex_patterns: usize,
    pub maximal_roots: Vec<String>,
    /// (reason, rule count) sorted by count desc, then reason asc. A rule with several
    /// reasons contributes to each — the histogram answers "what blocks fusion most".
    pub reason_histogram: Vec<(String, usize)>,
    pub rules: BTreeMap<String, RuleCensus>,
    pub entry_share: Option<EntryShare>,
    /// RGX-0078.5.h.1b — every choice (Or) site with ≥2 branches, classified for the
    /// increment-(ii) merged-choice gate. Deterministic order: rule-universe order,
    /// then pre-order within the rule.
    pub choice_sites: Vec<ChoiceSiteCensus>,
    /// RGX-0078.5.h.1b — the measured raw/committed/discarded decomposition (present
    /// iff `--fusibility-outcome-counts` files were joined).
    pub outcome_share: Option<OutcomeShare>,
}

/// The JSON shape `parseability_probe --dump-rule-entry-counts-json` writes; consumed by
/// `--fusibility-entry-counts`. Kept minimal and forward-stable.
#[derive(Debug, serde::Deserialize)]
struct RuleEntryCountsFile {
    grammar: String,
    #[allow(dead_code)]
    accepted: bool,
    #[serde(default)]
    rule_entry_counts: BTreeMap<String, u64>,
}

/// RGX-0078.5.h.1b — the JSON shape `parseability_probe --dump-rule-outcome-counts-json`
/// writes (raw + COMMITTED per-rule entry counts); consumed by
/// `--fusibility-outcome-counts`. `raw − committed` = the rule's FAILED-speculation
/// entries (committed keeps C3-B semantics: winners + successful-but-losing branches).
#[derive(Debug, serde::Deserialize)]
struct RuleOutcomeCountsFile {
    grammar: String,
    #[allow(dead_code)]
    accepted: bool,
    #[serde(default)]
    rule_entry_counts: BTreeMap<String, u64>,
    #[serde(default)]
    rule_committed_counts: BTreeMap<String, u64>,
}

/// RGX-0078.5.h.1b — one branch of a choice (Or) site, classified for the
/// increment-(ii) MERGED-CHOICE gate.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ChoiceBranchVerdict {
    /// 1-based branch index (matches the trace's `branch N/M` numbering).
    pub index: usize,
    /// The branch LANGUAGE is DFA-decidable (value-agnostic shape gate + layout
    /// contiguity) — a multi-accept DFA at the site can answer this branch's
    /// match/no-match + length in one scan.
    pub encodable: bool,
    /// The branch value is also a matched-text fold (increment-1 tier): the winning
    /// descent itself could be replaced, not just the failing probes.
    pub text_folding: bool,
    /// Rule references appearing anywhere in the branch subtree (deduped, sorted).
    pub direct_refs: Vec<String>,
    /// Of `direct_refs`: rules whose EVERY grammar-wide reference occurrence lives in
    /// this branch subtree — their measured entries are attributable to THIS site
    /// (the sound per-site attribution basis; a rule referenced from several sites
    /// cannot be split with per-rule aggregate counters).
    pub sole_refs: Vec<String>,
    /// RGX-0078.5.i.3 (P2) — the branch's admissible FIRST bytes (sorted) when the
    /// branch is first-byte-DECIDED: its FIRST-set summary is resolved + non-nullable
    /// and every FIRST terminal's first byte is extractable (exactly the codegen
    /// prune-guard eligibility, `first_set_prune_guard_for_branch`). `None` = the
    /// branch must always be tried, so its site can never dispatch degenerately.
    pub first_bytes: Option<Vec<u8>>,
}

/// RGX-0078.5.h.1b — one choice (Or) site of the grammar: where a merged-choice
/// multi-accept DFA could pre-discriminate the token-shaped branch subset.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ChoiceSiteCensus {
    pub rule: String,
    /// Site id within the rule: `or#N` in pre-order walk order (`or#0` = the rule's
    /// top-level alternation when the body is an Or).
    pub site: String,
    /// True for the rule's top-level Or (branch return annotations align by index
    /// there, so the per-branch text judgment uses them; nested sites judge the
    /// default fold).
    pub top_level: bool,
    pub branches: usize,
    pub encodable_branches: usize,
    /// Every branch is encodable — the full section-F increment-(ii) shape (ONE
    /// multi-accept DFA replaces the whole tournament).
    pub all_encodable: bool,
    pub branch_verdicts: Vec<ChoiceBranchVerdict>,
    /// Measured DISCARDED entries on this site's sole-attributable encodable-branch
    /// rules (only meaningful when outcome counts were joined; 0 otherwise). Sound
    /// attribution to this site's encodable branch SUBTREES (every occurrence of a
    /// sole ref lives here, and an encodable rule's closure is encodable, so all its
    /// failing work is DFA-killable within this subtree — at this site or an inner
    /// one). A lower bound: shared-reference rules and in-branch terminal probing
    /// are invisible to per-rule aggregates.
    pub attributable_discarded: u64,
    /// RGX-0078.5.i.3 (P2) — the site qualifies for DEGENERATE-TOURNAMENT byte-switch
    /// dispatch: rule-top-level + terminal-whitespace-sensitive layout + EVERY branch
    /// first-byte-decided + pairwise-DISJOINT first-byte sets + no branch-phase
    /// predicates / branch-start effect directives on the rule. At such a site at most
    /// ONE branch can begin a match at any next byte, so the longest-match tournament
    /// (checkpoint / delta-extract / rollback / replay / `should_take`) is provably
    /// protocol-only — the P2 emission surface.
    pub degenerate_dispatch: bool,
    /// The NAMED failing P2 gates (empty iff `degenerate_dispatch`). Deterministic
    /// order: R1 nesting, R2 layout, per-branch undecided (by index), first-byte
    /// overlaps (by byte), branch predicates, branch-start effects.
    pub degeneracy_blockers: Vec<String>,
}

/// RGX-0078.5.h.1b — the measured outcome-share join (census × raw+committed counts):
/// the increment-(ii) decomposition of real parse work.
#[derive(Debug, Clone, serde::Serialize)]
pub struct OutcomeShare {
    pub count_files: usize,
    pub total_entries: u64,
    /// Entries surviving all speculative rollbacks (C3-B: winners + successful-but-
    /// losing tournament branches). This work builds/validates the accepted parse —
    /// no site discriminator can remove the non-encodable part of it.
    pub total_committed: u64,
    /// `total_entries − total_committed`: entries inside failed speculations — the
    /// probing waste. The increment-(ii) target surface.
    pub total_discarded: u64,
    /// Of `total_discarded`: entries on SHAPE-ENCODABLE rules — every such failing
    /// attempt is answerable by a derived-DFA test at its call site (choice branch,
    /// optional group, or iteration attempt alike), so this is the sound
    /// increment-(ii) kill surface under the uniform per-entry cost model.
    pub discarded_on_encodable: u64,
    /// Of `total_discarded`: entries on NON-encodable (structural) rules — killable
    /// only by prefix-approximation discrimination (out of increment-(ii) scope).
    pub discarded_on_non_encodable: u64,
    /// Of `total_committed`: entries on shape-encodable rules (context — the
    /// increment-(i)-adjacent surface already measured by `EntryShare`).
    pub committed_on_encodable: u64,
    /// Rule names present in the counts files but absent from the census.
    pub unmatched_rules: Vec<String>,
    pub unmatched_entries: u64,
    /// `total / (total − discarded_on_encodable)` — the measured increment-(ii)
    /// ceiling under the uniform per-entry cost model (scan calls ≈ a terminal
    /// match, first-order 0; memo-hit re-entries counted at full weight — both
    /// caveats carried from `.5.h.1`).
    pub ceiling_estimate: f64,
    /// Sum of per-rule `committed − raw` excess (a committed count exceeding the raw
    /// entry delta — possible when a positive-lookahead success's coverage pushes
    /// survive AND the later real parse memo-hits, replaying the cached delta without
    /// re-entering descendants). Per-rule discards use `saturating_sub`, so overshoot
    /// is never silently negative; a nonzero value is reported loudly.
    pub committed_overshoot: u64,
    /// RGX-0078.5.i.3 (P2) — raw entries on rules whose TOP-LEVEL choice site is
    /// `degenerate_dispatch`: the Or-body executions the P2 emission strips of
    /// tournament protocol. Memo-hit re-entries (which never execute the body) are
    /// counted at full weight — the standing census caveat, carried.
    pub degenerate_site_entries: u64,
    /// Of `degenerate_site_entries`: the committed (C3-B surviving) part.
    pub degenerate_site_committed: u64,
    /// Of `degenerate_site_entries`: the discarded (failed-speculation) part.
    pub degenerate_site_discarded: u64,
}

/// How many lexemes a node consumes, for the layout-contiguity gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Arity {
    Zero,
    Single,
    Multi,
}

impl Arity {
    fn seq(self, other: Arity) -> Arity {
        match (self, other) {
            (Arity::Zero, x) | (x, Arity::Zero) => x,
            _ => Arity::Multi,
        }
    }
}

/// Facts about one grammar node, composed bottom-up.
#[derive(Debug, Clone)]
struct NodeFacts {
    /// Scanner-compilable shape (regular, effect-free closure, valid atoms/quantifiers).
    ok: bool,
    reasons: Vec<String>,
    uses_lookahead: bool,
    arity: Arity,
    /// Any consuming atom in the (sub)closure skips leading layout at runtime.
    skipping_atom: bool,
    /// The node's default (unannotated) AST contribution is exactly its matched text.
    is_text: bool,
    /// The node is a `Lookahead` (zero-width, contributes no value).
    is_lookahead_node: bool,
}

impl NodeFacts {
    fn fail(reason: String) -> Self {
        NodeFacts {
            ok: false,
            reasons: vec![reason],
            uses_lookahead: false,
            arity: Arity::Zero,
            skipping_atom: false,
            is_text: false,
            is_lookahead_node: false,
        }
    }
}

/// Per-rule memoized outcome. `shape_ok` is the VALUE-AGNOSTIC half of the gate
/// (regular + effect-free + policy-encodable + layout-contiguous — i.e. the rule's
/// LANGUAGE is scanner-encodable); `fusible` additionally requires the value half
/// (gate 3, text-folding). The split matters because a subtree whose value is
/// DISCARDED — a lookahead operand, or anything under a `-> $text` branch — needs
/// only `shape_ok`.
#[derive(Debug, Clone)]
struct RuleOutcome {
    shape_ok: bool,
    fusible: bool,
    reasons: Vec<String>,
    uses_lookahead: bool,
    arity: Arity,
    skipping_atom: bool,
    text_folding: bool,
}

struct Classifier<'a> {
    tree: &'a HashMap<String, ASTNode>,
    annotations: Option<&'a Annotations>,
    directives_by_rule: HashMap<String, Vec<String>>,
    layout: LayoutSensitivity,
    /// RGX-0078.5.i.3 (P2) — the FULL compiled runtime-annotation table (the same
    /// resolution the generated parser burns in), kept for the per-rule branch-phase
    /// predicate / branch-start effect queries behind the degeneracy gate (e).
    compiled: Option<CompiledSemanticRuntimeAnnotations>,
    /// RGX-0078.5.i.3 (P2) — shared FIRST-set cache for the per-branch dispatch
    /// first-byte analysis (the same `first_set` module codegen's prune guard uses).
    first_set_cache: HashMap<String, super::first_set::FirstSetSummary>,
    memo: HashMap<String, RuleOutcome>,
    visiting: HashSet<String>,
}

impl<'a> Classifier<'a> {
    fn new(
        tree: &'a HashMap<String, ASTNode>,
        annotations: Option<&'a Annotations>,
    ) -> Result<Self, String> {
        // Compile once: the SAME per-rule runtime-directive resolution codegen burns into
        // the generated parser (the shared-delegate precedent — the census must read the
        // resolution codegen actually emits, never re-derive its own).
        let (directives_by_rule, layout, compiled) = match annotations {
            Some(ann) => {
                let compiled = compile_semantic_runtime_annotations(ann)
                    .map_err(|e| format!("annotations failed to compile: {e}"))?;
                let mut map: HashMap<String, Vec<String>> = HashMap::new();
                for rule in ann
                    .semantic_annotations
                    .keys()
                    .chain(ann.branch_semantic_annotations.keys())
                {
                    let mut kinds: Vec<String> = compiled
                        .directives_for_rule(rule)
                        .iter()
                        .map(directive_kind_name)
                        .collect();
                    for branch in compiled.branch_directives_for_rule(rule) {
                        kinds.extend(branch.iter().map(directive_kind_name));
                    }
                    kinds.sort();
                    kinds.dedup();
                    if !kinds.is_empty() {
                        map.insert(rule.clone(), kinds);
                    }
                }
                (map, compiled.layout_sensitivity(), Some(compiled))
            }
            None => (HashMap::new(), LayoutSensitivity::default(), None),
        };
        Ok(Classifier {
            tree,
            annotations,
            directives_by_rule,
            layout,
            compiled,
            first_set_cache: HashMap::new(),
            memo: HashMap::new(),
            visiting: HashSet::new(),
        })
    }

    /// RGX-0078.5.i.3 (P2) — the branch's admissible dispatch FIRST bytes, or the
    /// NAMED reason the branch is not first-byte-decided. Mirrors the codegen prune
    /// guard's eligibility EXACTLY (`first_set_prune_guard_for_branch`): a resolved,
    /// non-nullable summary whose every FIRST terminal yields an extractable first
    /// byte; anything uncertain means "always try" — the site cannot dispatch.
    fn branch_dispatch_first_bytes(&mut self, branch: &ASTNode) -> Result<Vec<u8>, String> {
        let mut visiting = HashSet::new();
        let summary = super::first_set::branch_first_set(
            branch,
            self.tree,
            &mut self.first_set_cache,
            &mut visiting,
            0,
        );
        if summary.nullable {
            return Err("nullable (can match empty)".to_string());
        }
        if summary.unresolved {
            return Err("unresolved FIRST set (regex token / cycle / depth cutoff)".to_string());
        }
        if summary.terminals.is_empty() {
            return Err("empty FIRST terminal set".to_string());
        }
        let mut bytes: std::collections::BTreeSet<u8> = std::collections::BTreeSet::new();
        for terminal in &summary.terminals {
            match super::first_set::terminal_first_byte(terminal) {
                Some(byte) => {
                    bytes.insert(byte);
                }
                None => {
                    return Err(format!("unextractable first byte for terminal {terminal}"));
                }
            }
        }
        Ok(bytes.into_iter().collect())
    }

    /// RGX-0078.5.i.3 (P2) — gate (e): does the rule carry any Branch-phase
    /// predicate (rule-level or branch-local) or branch-start effect directive?
    /// Queried against the SAME compiled table the generated parser consults at
    /// runtime, never re-derived from raw annotations.
    fn rule_branch_predicate_effect_facts(&self, rule: &str, branch_count: usize) -> (bool, bool) {
        let Some(compiled) = &self.compiled else {
            return (false, false);
        };
        let has_branch_predicates = compiled.branch_predicates_for_rule(rule).next().is_some()
            || (0..branch_count).any(|i| {
                compiled
                    .branch_predicates_for_rule_branch(rule, i)
                    .next()
                    .is_some()
            });
        let has_branch_start_effects = (0..branch_count).any(|i| {
            compiled
                .branch_effect_directives_for_rule_branch(rule, i)
                .next()
                .is_some()
        });
        (has_branch_predicates, has_branch_start_effects)
    }

    /// Classify one rule (memoized; cycle-guarded — re-entry means the reference closure
    /// is cyclic, which fails gate criterion 1).
    fn classify_rule(&mut self, rule: &str) -> RuleOutcome {
        if let Some(hit) = self.memo.get(rule) {
            return hit.clone();
        }
        if self.visiting.contains(rule) {
            // Do NOT memoize the re-entry verdict: this frame only witnesses the cycle;
            // the rule's own outer frame computes (and memoizes) its full outcome.
            return RuleOutcome {
                shape_ok: false,
                fusible: false,
                reasons: vec![format!("reference cycle through '{rule}' (not regular)")],
                uses_lookahead: false,
                arity: Arity::Zero,
                skipping_atom: false,
                text_folding: false,
            };
        }
        self.visiting.insert(rule.to_string());
        let outcome = self.classify_rule_uncached(rule);
        self.visiting.remove(rule);
        self.memo.insert(rule.to_string(), outcome.clone());
        outcome
    }

    fn classify_rule_uncached(&mut self, rule: &str) -> RuleOutcome {
        let mut reasons: Vec<String> = Vec::new();

        // Gate 2 — effect-free: runtime directives anywhere on the rule (rule- or
        // branch-attached; the compiled map holds only outcome/store-relevant kinds —
        // steering directives like @branch_policy never enter it).
        if let Some(kinds) = self.directives_by_rule.get(rule) {
            for kind in kinds {
                reasons.push(format!("runtime directive @{kind}"));
            }
        }
        if let Some(ann) = self.annotations {
            if ann
                .branch_mid_sequence_semantic_annotations
                .get(rule)
                .is_some_and(|branches| branches.iter().any(|b| !b.is_empty()))
            {
                reasons.push("mid-sequence inline directive".to_string());
            }
            if ann.lexical_follow_restrictions.contains_key(rule) {
                reasons.push("lexical follow restriction [> …]".to_string());
            }
            for name in rule_level_directive_names(ann, rule) {
                match name.as_str() {
                    "transform" => reasons
                        .push("matched-text @transform (increment-2 candidate)".to_string()),
                    "profiles" => reasons.push(
                        "@profiles dialect gate (guard not scanner-encoded in increment 1)"
                            .to_string(),
                    ),
                    _ => {}
                }
            }
        }
        if !effective_rule_value_constraints(self.annotations, rule).is_empty() {
            reasons.push("value constraint (@enum/@regex/@range/@len)".to_string());
        }

        // Gate 4 — choice semantics: all three branch policies are DFA-encodable
        // (longest_match = maximal munch; ordered/priority_first = accept-priority
        // tie-break), so the policy itself never disqualifies. The two exclusions:
        let _ = effective_rule_branch_policy(self.annotations, rule);
        if effective_rule_associativity(self.annotations, rule) == SemanticAssociativity::NonAssoc {
            reasons.push("@associativity nonassoc (equal-tie failure semantics)".to_string());
        }
        if effective_rule_deterministic_partition_policy(self.annotations, rule).enabled {
            reasons.push("@deterministic_group evaluation-order rotation".to_string());
        }

        // Gates 1 + 3 + 5 — shape walk per branch (top-level Or alternatives are the
        // branches; return annotations align with them by index).
        let Some(body) = self.tree.get(rule) else {
            return RuleOutcome {
                shape_ok: false,
                fusible: false,
                reasons: vec![format!("rule '{rule}' not defined in the grammar tree")],
                uses_lookahead: false,
                arity: Arity::Zero,
                skipping_atom: false,
                text_folding: false,
            };
        };
        let branches: Vec<&ASTNode> = match body {
            ASTNode::Or { alternatives } => alternatives.iter().collect(),
            other => vec![other],
        };
        let empty: Vec<Option<super::BranchAnnotation>> = Vec::new();
        let branch_annotations = self
            .annotations
            .and_then(|a| a.branch_return_annotations.get(rule))
            .unwrap_or(&empty);
        if branch_annotations.len() > branches.len() {
            reasons.push(format!(
                "return-annotation arity mismatch ({} annotations, {} branches)",
                branch_annotations.len(),
                branches.len()
            ));
        }

        let mut shape_reasons_ok = reasons.is_empty();
        let mut shape_walk_ok = true;
        let mut uses_lookahead = false;
        let mut arity = Arity::Zero;
        let mut skipping_atom = false;
        let mut text_folding = true;
        let mut value_reasons: Vec<String> = Vec::new();

        for (i, branch) in branches.iter().enumerate() {
            let facts = self.node_facts(branch, rule);
            if !facts.ok {
                shape_walk_ok = false;
                reasons.extend(facts.reasons.iter().cloned());
            }
            uses_lookahead |= facts.uses_lookahead;
            arity = arity.max(facts.arity);
            skipping_atom |= facts.skipping_atom;

            let annotation = branch_annotations.get(i).and_then(|a| a.as_ref());
            if !self.branch_value_is_text(branch, annotation, rule) {
                text_folding = false;
                value_reasons.push(match annotation {
                    Some(_) => format!("branch {} return annotation is not a matched-text fold", i + 1),
                    None => format!("branch {} default fold is not matched text", i + 1),
                });
            }
        }

        // Gate 5 — layout contiguity: a multi-lexeme subtree whose atoms skip leading
        // layout at runtime would embed trivia inside the fused span.
        if skipping_atom && arity == Arity::Multi {
            shape_reasons_ok = false;
            reasons.push(
                "internal layout boundary (multi-lexeme body under a layout-skipping policy)"
                    .to_string(),
            );
        }

        let shape_ok = shape_reasons_ok && shape_walk_ok;
        let fusible = shape_ok && text_folding;
        reasons.extend(value_reasons);
        let mut deduped = Vec::new();
        for r in reasons {
            if !deduped.contains(&r) {
                deduped.push(r);
            }
        }
        RuleOutcome {
            shape_ok,
            fusible,
            reasons: deduped,
            uses_lookahead,
            arity,
            skipping_atom,
            text_folding,
        }
    }

    /// Bottom-up structural facts for one node (gate 1 shape + gate 5 layout + the
    /// default-fold text judgment used by gate 3).
    fn node_facts(&mut self, node: &ASTNode, rule: &str) -> NodeFacts {
        match node {
            ASTNode::Atom { value } => self.atom_facts(value, rule),
            ASTNode::Or { alternatives } => {
                let mut out = NodeFacts {
                    ok: true,
                    reasons: Vec::new(),
                    uses_lookahead: false,
                    arity: Arity::Zero,
                    skipping_atom: false,
                    is_text: true,
                    is_lookahead_node: false,
                };
                for alt in alternatives {
                    let f = self.node_facts(alt, rule);
                    out.ok &= f.ok;
                    out.reasons.extend(f.reasons);
                    out.uses_lookahead |= f.uses_lookahead;
                    out.arity = out.arity.max(f.arity);
                    out.skipping_atom |= f.skipping_atom;
                    out.is_text &= f.is_text;
                }
                out
            }
            ASTNode::Sequence { elements } => {
                let mut out = NodeFacts {
                    ok: true,
                    reasons: Vec::new(),
                    uses_lookahead: false,
                    arity: Arity::Zero,
                    skipping_atom: false,
                    is_text: true,
                    is_lookahead_node: false,
                };
                let mut value_elements = 0usize;
                let mut value_is_text = true;
                for element in elements {
                    let f = self.node_facts(element, rule);
                    out.ok &= f.ok;
                    out.reasons.extend(f.reasons.clone());
                    out.uses_lookahead |= f.uses_lookahead;
                    out.arity = out.arity.seq(f.arity);
                    out.skipping_atom |= f.skipping_atom;
                    if !f.is_lookahead_node {
                        value_elements += 1;
                        value_is_text &= f.is_text;
                    }
                }
                // The default fold of a multi-element sequence is a LIST of the element
                // values, not their concatenated text — only a single value-bearing
                // element (its lookahead siblings contribute nothing) folds to text.
                out.is_text = value_elements == 1 && value_is_text;
                out
            }
            ASTNode::Quantified {
                element,
                quantifier,
            } => {
                let mut f = self.node_facts(element, rule);
                match parse_quantifier_bounds(quantifier) {
                    Some((_, Some(max))) if max <= 1 => { /* at most one lexeme */ }
                    Some(_) => {
                        if f.arity != Arity::Zero {
                            f.arity = Arity::Multi;
                        }
                    }
                    None => {
                        f.ok = false;
                        f.reasons.push(format!("invalid quantifier '{quantifier}'"));
                    }
                }
                // The default fold of a quantified group is a list of iteration values,
                // never the concatenated matched text (increment-1 strictest).
                f.is_text = false;
                f.is_lookahead_node = false;
                f
            }
            ASTNode::Lookahead { element, .. } => {
                let f = self.node_facts(element, rule);
                NodeFacts {
                    // A lookahead's operand LANGUAGE must be scanner-encodable (product /
                    // complement DFA); its value never surfaces.
                    ok: f.ok && !(f.skipping_atom && f.arity == Arity::Multi),
                    reasons: f.reasons,
                    uses_lookahead: true,
                    arity: Arity::Zero,
                    skipping_atom: f.skipping_atom,
                    is_text: false,
                    is_lookahead_node: true,
                }
            }
        }
    }

    fn atom_facts(&mut self, value: &ASTValue, rule: &str) -> NodeFacts {
        match value {
            ASTValue::Node(inner) => self.node_facts(inner, rule),
            ASTValue::Token(parts) => {
                if parts.len() < 2 {
                    return NodeFacts::fail("malformed token atom".to_string());
                }
                let TokenValue::String(token_type) = &parts[0];
                let TokenValue::String(token_value) = &parts[1];
                match token_type.as_str() {
                    // Literal terminals — matched via `match_string`, which skips leading
                    // layout iff the grammar's terminals facet is insensitive.
                    "quoted_string" | "number" | "probability" | "include_dir"
                    | "include_file" | "rule" => NodeFacts {
                        ok: true,
                        reasons: Vec::new(),
                        uses_lookahead: false,
                        arity: if token_type == "quoted_string" && token_value.is_empty() {
                            Arity::Zero
                        } else {
                            Arity::Single
                        },
                        skipping_atom: !self.layout.terminals,
                        is_text: true,
                        is_lookahead_node: false,
                    },
                    // Regex-literal terminals — `match_regex` skips leading layout iff the
                    // regex_tokens facet is insensitive AND the rule is not one of the two
                    // generator-special-cased string-content rules (mirrored exactly).
                    "regex" => NodeFacts {
                        ok: true,
                        reasons: Vec::new(),
                        uses_lookahead: false,
                        arity: Arity::Single,
                        skipping_atom: !self.layout.regex_tokens
                            && !REGEX_ATOM_NO_SKIP_RULES.contains(&rule),
                        is_text: true,
                        is_lookahead_node: false,
                    },
                    "rule_reference" => self.reference_facts(token_value),
                    other => NodeFacts::fail(format!("unknown token atom type '{other}'")),
                }
            }
        }
    }

    fn reference_facts(&mut self, target: &str) -> NodeFacts {
        match target {
            // The char builtins are single-char recognizers synthesized by codegen —
            // regular, contiguous, text-valued by construction.
            "builtin_any_char" | "builtin_ascii_char" => NodeFacts {
                ok: true,
                reasons: Vec::new(),
                uses_lookahead: false,
                arity: Arity::Single,
                skipping_atom: false,
                is_text: true,
                is_lookahead_node: false,
            },
            _ if AstBasedGenerator::NATIVE_UNRESOLVED_REFERENCE_BUILTINS.contains(&target)
                && !self.tree.contains_key(target) =>
            {
                NodeFacts::fail(format!(
                    "native builtin '{target}' outside the increment-1 gate"
                ))
            }
            _ if !self.tree.contains_key(target) => {
                NodeFacts::fail(format!("undefined reference '{target}'"))
            }
            _ => {
                // The reference's SHAPE gate is the target's shape gate (value-agnostic
                // — a lookahead operand or a `$text` subtree may reference a rule whose
                // own value is not text); the VALUE judgment (`is_text`) additionally
                // requires the target to be text-folding.
                let sub = self.classify_rule(target);
                NodeFacts {
                    ok: sub.shape_ok,
                    reasons: if sub.shape_ok {
                        Vec::new()
                    } else {
                        vec![format!("references non-encodable rule '{target}'")]
                    },
                    uses_lookahead: sub.uses_lookahead,
                    arity: sub.arity,
                    skipping_atom: sub.skipping_atom,
                    is_text: sub.text_folding,
                    is_lookahead_node: false,
                }
            }
        }
    }

    /// Gate 3 — is this branch's AST contribution exactly its matched text?
    fn branch_value_is_text(
        &mut self,
        branch: &ASTNode,
        annotation: Option<&super::BranchAnnotation>,
        rule: &str,
    ) -> bool {
        match annotation {
            None => self.node_facts(branch, rule).is_text,
            Some(ann) => match ann.parsed_ast.as_ref() {
                // `-> $text`: the branch's value IS its matched span text, by
                // construction — the canonical text fold. Child values are all
                // discarded, so only the (already-walked) SHAPE gate applies.
                Some(UnifiedReturnAST::MatchedText) => true,
                // `-> $N` passthrough: element N must fold to text and every sibling must
                // be a zero-width lookahead (so $N's text IS the whole matched span).
                Some(UnifiedReturnAST::PositionalRef { index }) => {
                    let elements: Vec<&ASTNode> = match branch {
                        ASTNode::Sequence { elements } => elements.iter().collect(),
                        other => vec![other],
                    };
                    if *index == 0 || *index > elements.len() {
                        return false;
                    }
                    elements.iter().enumerate().all(|(i, element)| {
                        if i + 1 == *index {
                            self.node_facts(element, rule).is_text
                        } else {
                            matches!(element, ASTNode::Lookahead { .. })
                        }
                    })
                }
                // Any structural template (objects, arrays, spreads, literals, …) or an
                // unparsed annotation is not a matched-text fold.
                _ => false,
            },
        }
    }
}

/// Rule-level directive NAMES (rule- and branch-attached), for the compile-time-only
/// directives that never enter the runtime map (`@transform`, `@profiles`).
fn rule_level_directive_names(annotations: &Annotations, rule: &str) -> Vec<String> {
    let mut names = Vec::new();
    if let Some(entries) = annotations.semantic_annotations.get(rule) {
        for annotation in entries {
            if let Some(name) = annotation.name() {
                names.push(name.trim().to_ascii_lowercase());
            }
        }
    }
    if let Some(branches) = annotations.branch_semantic_annotations.get(rule) {
        for branch in branches {
            for annotation in branch {
                if let Some(name) = annotation.name() {
                    names.push(name.trim().to_ascii_lowercase());
                }
            }
        }
    }
    names.sort();
    names.dedup();
    names
}

fn directive_kind_name(directive: &SemanticRuntimeDirective) -> String {
    match directive {
        SemanticRuntimeDirective::OpenScope(_) => "open_scope".to_string(),
        SemanticRuntimeDirective::CloseScope(_) => "close_scope".to_string(),
        SemanticRuntimeDirective::EmitFact(_) => "emit_fact".to_string(),
        SemanticRuntimeDirective::Predicate(_) => "predicate".to_string(),
        SemanticRuntimeDirective::ExportToLibrary(_) => "export_to_library".to_string(),
        SemanticRuntimeDirective::ImportFromLibrary(_) => "import_from_library".to_string(),
        SemanticRuntimeDirective::DeclareFactKind(_) => "fact_kind".to_string(),
        SemanticRuntimeDirective::DefinePredicate(_) => "predicate_def".to_string(),
    }
}

/// Collect every rule referenced anywhere in `node` (all positions, including inside
/// lookaheads). Local twin of `grammar_wellformedness::collect_node_rule_refs` (private
/// there); used for the referenced-by map behind maximal-root detection. Also counts
/// regex-literal atom sites into `regex_patterns` (one push per site).
fn collect_refs(node: &ASTNode, out: &mut HashSet<String>, regex_patterns: &mut Vec<String>) {
    match node {
        ASTNode::Or { alternatives } => alternatives
            .iter()
            .for_each(|a| collect_refs(a, out, regex_patterns)),
        ASTNode::Sequence { elements } => elements
            .iter()
            .for_each(|e| collect_refs(e, out, regex_patterns)),
        ASTNode::Quantified { element, .. } | ASTNode::Lookahead { element, .. } => {
            collect_refs(element, out, regex_patterns)
        }
        ASTNode::Atom { value } => match value {
            ASTValue::Node(inner) => collect_refs(inner, out, regex_patterns),
            ASTValue::Token(parts) => {
                if parts.len() >= 2 {
                    let TokenValue::String(token_type) = &parts[0];
                    let TokenValue::String(token_value) = &parts[1];
                    if token_type == "rule_reference" {
                        out.insert(token_value.clone());
                    } else if token_type == "regex" {
                        regex_patterns.push(token_value.clone());
                    }
                }
            }
        },
    }
}

/// RGX-0078.5.h.1b — count every `rule_reference` occurrence (with multiplicity) in a
/// subtree. The grammar-wide totals feed the sole-reference attribution test: a rule
/// whose every occurrence lives inside one choice branch has all its measured entries
/// attributable to that site.
fn collect_ref_occurrences(node: &ASTNode, out: &mut HashMap<String, usize>) {
    match node {
        ASTNode::Or { alternatives } => alternatives
            .iter()
            .for_each(|a| collect_ref_occurrences(a, out)),
        ASTNode::Sequence { elements } => elements
            .iter()
            .for_each(|e| collect_ref_occurrences(e, out)),
        ASTNode::Quantified { element, .. } | ASTNode::Lookahead { element, .. } => {
            collect_ref_occurrences(element, out)
        }
        ASTNode::Atom { value } => match value {
            ASTValue::Node(inner) => collect_ref_occurrences(inner, out),
            ASTValue::Token(parts) => {
                if parts.len() >= 2 {
                    let TokenValue::String(token_type) = &parts[0];
                    let TokenValue::String(token_value) = &parts[1];
                    if token_type == "rule_reference" {
                        *out.entry(token_value.clone()).or_default() += 1;
                    }
                }
            }
        },
    }
}

/// RGX-0078.5.i.3 (P2) — the per-site degeneracy verdict: can this choice site
/// dispatch as a degenerate tournament (ONE byte-switch, no speculation protocol)?
/// Pure over its inputs so the gate logic is unit-testable in isolation. Blockers
/// are named per the leaf spec's gates (a)–(e), in deterministic order; the site
/// qualifies iff NO blocker fires.
fn site_degeneracy_verdict(
    top_level: bool,
    layout_terminals: bool,
    has_branch_phase_predicates: bool,
    has_branch_start_effects: bool,
    branch_first_bytes: &[Result<Vec<u8>, String>],
) -> (bool, Vec<String>) {
    let mut blockers: Vec<String> = Vec::new();
    if !top_level {
        blockers.push("nested Or site (rule-top-level only — R1 furthest-position neutrality)".to_string());
    }
    if !layout_terminals {
        blockers.push("terminals skip leading layout (R2 raw-byte peek unsound)".to_string());
    }
    for (i, bytes) in branch_first_bytes.iter().enumerate() {
        if let Err(reason) = bytes {
            blockers.push(format!("branch {} not first-byte-decided: {reason}", i + 1));
        }
    }
    // Pairwise disjointness over the DECIDED branches: any byte admissible for two or
    // more branches keeps the tournament real (several candidates on that byte).
    let mut byte_owners: BTreeMap<u8, Vec<usize>> = BTreeMap::new();
    for (i, bytes) in branch_first_bytes.iter().enumerate() {
        if let Ok(bytes) = bytes {
            for byte in bytes {
                byte_owners.entry(*byte).or_default().push(i + 1);
            }
        }
    }
    for (byte, owners) in &byte_owners {
        if owners.len() >= 2 {
            let printable = if byte.is_ascii_graphic() {
                format!(" ('{}')", *byte as char)
            } else {
                String::new()
            };
            blockers.push(format!(
                "first byte 0x{byte:02X}{printable} shared by branches {}",
                owners
                    .iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            ));
        }
    }
    if has_branch_phase_predicates {
        blockers.push(
            "rule has branch-phase predicates (a rejected sole candidate must roll back and continue the tournament)"
                .to_string(),
        );
    }
    if has_branch_start_effects {
        blockers.push("rule has branch-start effect directives".to_string());
    }
    (blockers.is_empty(), blockers)
}

/// RGX-0078.5.h.1b — enumerate and classify every choice (Or) site with ≥2 branches.
/// Pre-order per rule; `or#0` is the first Or encountered (the top-level alternation
/// when the rule body is an Or). Branch encodability mirrors the rule-level shape gate
/// (`node_facts` + the layout-contiguity exclusion); the per-branch text judgment uses
/// the rule's branch return annotations at the TOP-LEVEL site (they align by index
/// there) and the default fold at nested sites.
fn enumerate_choice_sites(
    classifier: &mut Classifier,
    universe: &[String],
    grammar_wide_refs: &HashMap<String, usize>,
) -> Vec<ChoiceSiteCensus> {
    let mut sites = Vec::new();
    for rule in universe {
        let Some(body) = classifier.tree.get(rule.as_str()) else {
            continue;
        };
        let mut or_counter = 0usize;
        walk_for_choice_sites(
            classifier,
            rule,
            body,
            true,
            &mut or_counter,
            grammar_wide_refs,
            &mut sites,
        );
    }
    sites
}

#[allow(clippy::too_many_arguments)]
fn walk_for_choice_sites(
    classifier: &mut Classifier,
    rule: &str,
    node: &ASTNode,
    is_rule_body: bool,
    or_counter: &mut usize,
    grammar_wide_refs: &HashMap<String, usize>,
    sites: &mut Vec<ChoiceSiteCensus>,
) {
    match node {
        ASTNode::Or { alternatives } => {
            let site_index = *or_counter;
            *or_counter += 1;
            if alternatives.len() >= 2 {
                let empty: Vec<Option<super::BranchAnnotation>> = Vec::new();
                let branch_annotations = if is_rule_body {
                    classifier
                        .annotations
                        .and_then(|a| a.branch_return_annotations.get(rule))
                        .unwrap_or(&empty)
                        .clone()
                } else {
                    empty
                };
                // RGX-0078.5.i.3 (P2) — per-branch dispatch first-byte analysis
                // (computed for every site so nested/blocked sites still report
                // their byte sets for steering).
                let branch_dispatch_bytes: Vec<Result<Vec<u8>, String>> = alternatives
                    .iter()
                    .map(|branch| classifier.branch_dispatch_first_bytes(branch))
                    .collect();
                let mut branch_verdicts = Vec::with_capacity(alternatives.len());
                for (i, branch) in alternatives.iter().enumerate() {
                    let facts = classifier.node_facts(branch, rule);
                    let encodable =
                        facts.ok && !(facts.skipping_atom && facts.arity == Arity::Multi);
                    let text_folding = if is_rule_body {
                        let annotation = branch_annotations.get(i).and_then(|a| a.as_ref());
                        classifier.branch_value_is_text(branch, annotation, rule)
                    } else {
                        facts.is_text
                    };
                    let mut branch_refs: HashMap<String, usize> = HashMap::new();
                    collect_ref_occurrences(branch, &mut branch_refs);
                    let mut direct_refs: Vec<String> = branch_refs.keys().cloned().collect();
                    direct_refs.sort();
                    let mut sole_refs: Vec<String> = branch_refs
                        .iter()
                        .filter(|(name, n)| {
                            grammar_wide_refs.get(name.as_str()).copied() == Some(**n)
                        })
                        .map(|(name, _)| name.clone())
                        .collect();
                    sole_refs.sort();
                    branch_verdicts.push(ChoiceBranchVerdict {
                        index: i + 1,
                        encodable,
                        text_folding,
                        direct_refs,
                        sole_refs,
                        first_bytes: branch_dispatch_bytes[i].as_ref().ok().cloned(),
                    });
                }
                let encodable_branches =
                    branch_verdicts.iter().filter(|b| b.encodable).count();
                // RGX-0078.5.i.3 (P2) — the site degeneracy verdict (gates a–e).
                let (has_branch_predicates, has_branch_start_effects) = classifier
                    .rule_branch_predicate_effect_facts(rule, alternatives.len());
                let (degenerate_dispatch, degeneracy_blockers) = site_degeneracy_verdict(
                    is_rule_body,
                    classifier.layout.terminals,
                    has_branch_predicates,
                    has_branch_start_effects,
                    &branch_dispatch_bytes,
                );
                sites.push(ChoiceSiteCensus {
                    rule: rule.to_string(),
                    site: format!("or#{site_index}"),
                    top_level: is_rule_body,
                    branches: alternatives.len(),
                    encodable_branches,
                    all_encodable: encodable_branches == alternatives.len(),
                    branch_verdicts,
                    attributable_discarded: 0,
                    degenerate_dispatch,
                    degeneracy_blockers,
                });
            }
            for branch in alternatives {
                walk_for_choice_sites(
                    classifier,
                    rule,
                    branch,
                    false,
                    or_counter,
                    grammar_wide_refs,
                    sites,
                );
            }
        }
        ASTNode::Sequence { elements } => {
            for element in elements {
                walk_for_choice_sites(
                    classifier,
                    rule,
                    element,
                    false,
                    or_counter,
                    grammar_wide_refs,
                    sites,
                );
            }
        }
        ASTNode::Quantified { element, .. } | ASTNode::Lookahead { element, .. } => {
            walk_for_choice_sites(
                classifier,
                rule,
                element,
                false,
                or_counter,
                grammar_wide_refs,
                sites,
            );
        }
        ASTNode::Atom { value } => {
            if let ASTValue::Node(inner) = value {
                walk_for_choice_sites(
                    classifier,
                    rule,
                    inner,
                    false,
                    or_counter,
                    grammar_wide_refs,
                    sites,
                );
            }
        }
    }
}

/// RGX-0078.5.h.1b — join outcome-count files (raw + committed) into the measured
/// discarded-work decomposition, and fill each choice site's sole-attributable
/// discarded total.
fn join_outcome_counts(
    grammar_name: &str,
    rules: &BTreeMap<String, RuleCensus>,
    choice_sites: &mut [ChoiceSiteCensus],
    files: &[std::path::PathBuf],
) -> Result<OutcomeShare, String> {
    let mut entries_sum: BTreeMap<String, u64> = BTreeMap::new();
    let mut committed_sum: BTreeMap<String, u64> = BTreeMap::new();
    for path in files {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read outcome-counts file '{}': {e}", path.display()))?;
        let parsed: RuleOutcomeCountsFile = serde_json::from_str(&text)
            .map_err(|e| format!("cannot parse outcome-counts file '{}': {e}", path.display()))?;
        if parsed.grammar != grammar_name {
            return Err(format!(
                "outcome-counts file '{}' is for grammar '{}', census is for '{}'",
                path.display(),
                parsed.grammar,
                grammar_name
            ));
        }
        for (rule, count) in parsed.rule_entry_counts {
            *entries_sum.entry(rule).or_default() += count;
        }
        for (rule, count) in parsed.rule_committed_counts {
            *committed_sum.entry(rule).or_default() += count;
        }
    }

    let mut share = OutcomeShare {
        count_files: files.len(),
        total_entries: 0,
        total_committed: 0,
        total_discarded: 0,
        discarded_on_encodable: 0,
        discarded_on_non_encodable: 0,
        committed_on_encodable: 0,
        unmatched_rules: Vec::new(),
        unmatched_entries: 0,
        ceiling_estimate: 1.0,
        committed_overshoot: 0,
        degenerate_site_entries: 0,
        degenerate_site_committed: 0,
        degenerate_site_discarded: 0,
    };
    let mut discarded_by_rule: BTreeMap<String, u64> = BTreeMap::new();
    let rule_universe: std::collections::BTreeSet<&String> =
        entries_sum.keys().chain(committed_sum.keys()).collect();
    for rule in rule_universe {
        let entries = entries_sum.get(rule).copied().unwrap_or(0);
        let committed = committed_sum.get(rule).copied().unwrap_or(0);
        share.total_entries += entries;
        share.total_committed += committed;
        share.committed_overshoot += committed.saturating_sub(entries);
        let discarded = entries.saturating_sub(committed);
        share.total_discarded += discarded;
        discarded_by_rule.insert(rule.clone(), discarded);
        match rules.get(rule) {
            Some(census) if census.shape_encodable => {
                share.discarded_on_encodable += discarded;
                share.committed_on_encodable += committed.min(entries);
            }
            Some(_) => {
                share.discarded_on_non_encodable += discarded;
            }
            None => {
                share.unmatched_rules.push(rule.clone());
                share.unmatched_entries += entries;
                // Conservative: unmatched rules never count toward the kill surface.
                share.discarded_on_non_encodable += discarded;
            }
        }
    }
    let remaining = share.total_entries - share.discarded_on_encodable.min(share.total_entries);
    share.ceiling_estimate = if remaining == 0 {
        f64::INFINITY
    } else {
        share.total_entries as f64 / remaining as f64
    };

    // RGX-0078.5.i.3 (P2) — the degenerate-dispatch exposure: raw/committed entries
    // on rules whose TOP-LEVEL site qualifies (one top-level site per rule, so each
    // rule is counted at most once).
    let mut degenerate_rules_counted: HashSet<&str> = HashSet::new();
    for site in choice_sites.iter() {
        if site.top_level
            && site.degenerate_dispatch
            && degenerate_rules_counted.insert(site.rule.as_str())
        {
            let entries = entries_sum.get(&site.rule).copied().unwrap_or(0);
            let committed = committed_sum.get(&site.rule).copied().unwrap_or(0);
            share.degenerate_site_entries += entries;
            share.degenerate_site_committed += committed.min(entries);
            share.degenerate_site_discarded += entries.saturating_sub(committed);
        }
    }

    // Per-site sole-attribution (a sound lower bound; see `attributable_discarded`).
    for site in choice_sites.iter_mut() {
        let mut counted: HashSet<&String> = HashSet::new();
        let mut total = 0u64;
        for branch in &site.branch_verdicts {
            if !branch.encodable {
                continue;
            }
            for sole in &branch.sole_refs {
                if counted.insert(sole) {
                    total += discarded_by_rule.get(sole).copied().unwrap_or(0);
                }
            }
        }
        site.attributable_discarded = total;
    }
    Ok(share)
}

/// Run the census over a grammar (the UNFILTERED tree — the view codegen compiles), and
/// optionally join per-parse rule-entry-count files into the measured entry share.
pub fn run_fusibility_census(
    grammar_name: &str,
    grammar_tree: &HashMap<String, ASTNode>,
    rule_order: &[String],
    annotations: Option<&Annotations>,
    entry_counts_files: &[std::path::PathBuf],
    outcome_counts_files: &[std::path::PathBuf],
) -> Result<FusibilityCensus, String> {
    let mut classifier = Classifier::new(grammar_tree, annotations)?;

    // Deterministic rule universe: rule_order first, then any tree-only stragglers.
    let mut universe: Vec<String> = rule_order
        .iter()
        .filter(|r| grammar_tree.contains_key(r.as_str()))
        .cloned()
        .collect();
    let mut extra: Vec<String> = grammar_tree
        .keys()
        .filter(|k| !universe.contains(k))
        .cloned()
        .collect();
    extra.sort();
    universe.extend(extra);

    let outcomes: BTreeMap<String, RuleOutcome> = universe
        .iter()
        .map(|rule| (rule.clone(), classifier.classify_rule(rule)))
        .collect();

    // Referenced-by map for maximal-root detection + the regex-atom-site count.
    let mut referenced_by: HashMap<String, Vec<String>> = HashMap::new();
    let mut regex_patterns: Vec<String> = Vec::new();
    for rule in &universe {
        let mut refs = HashSet::new();
        collect_refs(&grammar_tree[rule], &mut refs, &mut regex_patterns);
        for target in refs {
            referenced_by.entry(target).or_default().push(rule.clone());
        }
    }
    let regex_atom_sites = regex_patterns.len();
    let distinct_regex_patterns = regex_patterns
        .iter()
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let entry_rule = rule_order.first().cloned();

    let mut rules: BTreeMap<String, RuleCensus> = BTreeMap::new();
    let mut maximal_roots: Vec<String> = Vec::new();
    let mut histogram: HashMap<String, usize> = HashMap::new();
    let (mut n_token, mut n_lookahead, mut n_not, mut n_shape_only) =
        (0usize, 0usize, 0usize, 0usize);

    for rule in &universe {
        let outcome = &outcomes[rule];
        let tier = if outcome.fusible {
            if outcome.uses_lookahead {
                n_lookahead += 1;
                FusibilityTier::FusibleLookahead
            } else {
                n_token += 1;
                FusibilityTier::FusibleToken
            }
        } else {
            n_not += 1;
            if outcome.shape_ok {
                n_shape_only += 1;
            }
            FusibilityTier::NotFusible
        };
        for reason in &outcome.reasons {
            *histogram.entry(reason.clone()).or_default() += 1;
        }
        let maximal_root = outcome.fusible
            && (Some(rule) == entry_rule.as_ref()
                || referenced_by
                    .get(rule)
                    .map(|parents| parents.iter().any(|p| !outcomes[p].fusible))
                    .unwrap_or(true));
        if maximal_root {
            maximal_roots.push(rule.clone());
        }
        rules.insert(
            rule.clone(),
            RuleCensus {
                tier,
                reasons: outcome.reasons.clone(),
                maximal_root,
                uses_lookahead: outcome.uses_lookahead,
                shape_encodable: outcome.shape_ok,
            },
        );
    }

    let mut reason_histogram: Vec<(String, usize)> = histogram.into_iter().collect();
    reason_histogram.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

    let entry_share = if entry_counts_files.is_empty() {
        None
    } else {
        Some(join_entry_counts(
            grammar_name,
            &rules,
            entry_counts_files,
        )?)
    };

    // RGX-0078.5.h.1b — the choice-site census (static), then the outcome join
    // (raw + committed decomposition + per-site attribution) when files were given.
    let mut grammar_wide_refs: HashMap<String, usize> = HashMap::new();
    for rule in &universe {
        collect_ref_occurrences(&grammar_tree[rule], &mut grammar_wide_refs);
    }
    let mut choice_sites = enumerate_choice_sites(&mut classifier, &universe, &grammar_wide_refs);
    let outcome_share = if outcome_counts_files.is_empty() {
        None
    } else {
        Some(join_outcome_counts(
            grammar_name,
            &rules,
            &mut choice_sites,
            outcome_counts_files,
        )?)
    };

    Ok(FusibilityCensus {
        grammar_name: grammar_name.to_string(),
        total_rules: universe.len(),
        fusible_token: n_token,
        fusible_lookahead: n_lookahead,
        not_fusible: n_not,
        shape_only: n_shape_only,
        regex_atom_sites,
        distinct_regex_patterns,
        maximal_roots,
        reason_histogram,
        rules,
        entry_share,
        choice_sites,
        outcome_share,
    })
}

fn join_entry_counts(
    grammar_name: &str,
    rules: &BTreeMap<String, RuleCensus>,
    files: &[std::path::PathBuf],
) -> Result<EntryShare, String> {
    let mut summed: BTreeMap<String, u64> = BTreeMap::new();
    for path in files {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read entry-counts file '{}': {e}", path.display()))?;
        let parsed: RuleEntryCountsFile = serde_json::from_str(&text)
            .map_err(|e| format!("cannot parse entry-counts file '{}': {e}", path.display()))?;
        if parsed.grammar != grammar_name {
            return Err(format!(
                "entry-counts file '{}' is for grammar '{}', census is for '{}'",
                path.display(),
                parsed.grammar,
                grammar_name
            ));
        }
        for (rule, count) in parsed.rule_entry_counts {
            *summed.entry(rule).or_default() += count;
        }
    }

    let mut share = EntryShare {
        count_files: files.len(),
        total_entries: 0,
        eliminated_below_roots: 0,
        at_roots: 0,
        untouched: 0,
        shape_only_entries: 0,
        unmatched_rules: Vec::new(),
        unmatched_entries: 0,
        ceiling_estimate: 1.0,
    };
    for (rule, count) in &summed {
        share.total_entries += count;
        match rules.get(rule) {
            Some(census) if census.tier != FusibilityTier::NotFusible => {
                if census.maximal_root {
                    share.at_roots += count;
                } else {
                    share.eliminated_below_roots += count;
                }
            }
            Some(census) => {
                share.untouched += count;
                if census.shape_encodable {
                    share.shape_only_entries += count;
                }
            }
            None => {
                share.unmatched_rules.push(rule.clone());
                share.unmatched_entries += count;
            }
        }
    }
    let remaining = share.total_entries - share.eliminated_below_roots;
    share.ceiling_estimate = if remaining == 0 {
        f64::INFINITY
    } else {
        share.total_entries as f64 / remaining as f64
    };
    Ok(share)
}

/// Print the census report. The headline is grep-stable (`FUSIBILITY-CENSUS:`); the full
/// per-rule listing is gated behind `dump_all` (`PGEN_FUSIBILITY_DUMP_ALL=1`, mirroring
/// the certificate-coverage DUMP_ALL convention) so large grammars stay readable.
pub fn print_fusibility_census(census: &FusibilityCensus, dump_all: bool) {
    let fusible = census.fusible_token + census.fusible_lookahead;
    let static_share = if census.total_rules == 0 {
        0.0
    } else {
        100.0 * fusible as f64 / census.total_rules as f64
    };
    println!(
        "FUSIBILITY-CENSUS: grammar={} rules={} fusible={} (token={} lookahead={}) not_fusible={} (shape_only={}) maximal_roots={} static_share={:.1}%",
        census.grammar_name,
        census.total_rules,
        fusible,
        census.fusible_token,
        census.fusible_lookahead,
        census.not_fusible,
        census.shape_only,
        census.maximal_roots.len(),
        static_share,
    );
    println!(
        "  gate: increment-1 strict (regular + effect-free + text-folding + policy-encodable + layout-contiguous) — RGX-0078.5.h section F"
    );
    println!(
        "  match_regex surface: {} regex-literal atom sites ({} distinct patterns) — the derived-DFA engine-replacement surface (self-hosting), independent of rule-level fusion",
        census.regex_atom_sites, census.distinct_regex_patterns,
    );
    if !census.maximal_roots.is_empty() {
        println!("  maximal fusible roots (future scan_* sites):");
        for root in &census.maximal_roots {
            let tier = match census.rules[root].tier {
                FusibilityTier::FusibleToken => "token",
                FusibilityTier::FusibleLookahead => "lookahead",
                FusibilityTier::NotFusible => unreachable!("roots are fusible by construction"),
            };
            println!("    [root:{tier}] {root}");
        }
    }
    if !census.reason_histogram.is_empty() {
        println!("  disqualification histogram (rules per reason):");
        for (reason, count) in &census.reason_histogram {
            println!("    {count:>5}  {reason}");
        }
    }
    if dump_all {
        println!("  per-rule verdicts (PGEN_FUSIBILITY_DUMP_ALL):");
        for (rule, verdict) in &census.rules {
            let tier = match verdict.tier {
                FusibilityTier::FusibleToken => "fusible_token",
                FusibilityTier::FusibleLookahead => "fusible_lookahead",
                FusibilityTier::NotFusible => "not_fusible",
            };
            if verdict.reasons.is_empty() {
                println!("    {rule}: {tier}{}", if verdict.maximal_root { " [root]" } else { "" });
            } else {
                println!("    {rule}: {tier} — {}", verdict.reasons.join("; "));
            }
        }
    }
    if let Some(share) = &census.entry_share {
        println!(
            "FUSIBILITY-ENTRY-SHARE: grammar={} files={} total_entries={} eliminated_below_roots={} at_roots={} untouched={} (shape_only={}) unmatched={} ceiling≈{:.2}x",
            census.grammar_name,
            share.count_files,
            share.total_entries,
            share.eliminated_below_roots,
            share.at_roots,
            share.untouched,
            share.shape_only_entries,
            share.unmatched_entries,
            share.ceiling_estimate,
        );
        println!(
            "  model: uniform per-entry cost (~262ns/rule-entry, RGX-0078.5.e §A); a fused root entry becomes one scan_* call; entries below roots are eliminated"
        );
        if !share.unmatched_rules.is_empty() {
            println!(
                "  unmatched count-file rules (absent from census): {}",
                share.unmatched_rules.join(", ")
            );
        }
    }

    // RGX-0078.5.h.1b — the choice-site census + the measured outcome decomposition.
    let with_subset = census
        .choice_sites
        .iter()
        .filter(|s| s.encodable_branches >= 1)
        .count();
    let all_encodable = census
        .choice_sites
        .iter()
        .filter(|s| s.all_encodable)
        .count();
    println!(
        "CHOICE-SITE-CENSUS: grammar={} sites={} (top_level={}) with_encodable_subset={} all_encodable={}",
        census.grammar_name,
        census.choice_sites.len(),
        census.choice_sites.iter().filter(|s| s.top_level).count(),
        with_subset,
        all_encodable,
    );
    // RGX-0078.5.i.3 (P2) — the degenerate-dispatch surface + what blocks it.
    let top_level_sites: Vec<&ChoiceSiteCensus> =
        census.choice_sites.iter().filter(|s| s.top_level).collect();
    let degenerate_sites: Vec<&ChoiceSiteCensus> = top_level_sites
        .iter()
        .copied()
        .filter(|s| s.degenerate_dispatch)
        .collect();
    println!(
        "DEGENERACY-CENSUS: grammar={} top_level_sites={} degenerate_dispatch={}",
        census.grammar_name,
        top_level_sites.len(),
        degenerate_sites.len(),
    );
    println!(
        "  gate: top-level + terminal-ws-sensitive + all branches first-byte-decided + pairwise-disjoint + no branch predicates/effects — RGX-0078.5.i.3 (P2)"
    );
    if !degenerate_sites.is_empty() {
        let names: Vec<String> = degenerate_sites
            .iter()
            .map(|s| format!("{}({})", s.rule, s.branches))
            .collect();
        println!("  degenerate sites (rule(branches)): {}", names.join(" "));
    }
    let mut blocker_histogram: HashMap<String, usize> = HashMap::new();
    for site in &top_level_sites {
        for blocker in &site.degeneracy_blockers {
            // Fold per-branch/per-byte detail out of the histogram key so the
            // histogram answers "what blocks degeneracy most", not "where".
            let key = blocker
                .split_once(':')
                .map(|(head, _)| head)
                .unwrap_or(blocker.as_str());
            let key = if key.starts_with("first byte ") {
                "first-byte overlap between branches"
            } else if key.starts_with("branch ") {
                "branch not first-byte-decided"
            } else {
                key
            };
            *blocker_histogram.entry(key.to_string()).or_default() += 1;
        }
    }
    if !blocker_histogram.is_empty() {
        let mut ranked: Vec<(String, usize)> = blocker_histogram.into_iter().collect();
        ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        println!("  degeneracy blocker histogram (top-level site occurrences):");
        for (reason, count) in &ranked {
            println!("    {count:>5}  {reason}");
        }
    }
    if let Some(share) = &census.outcome_share {
        println!(
            "OUTCOME-SHARE: grammar={} files={} total_entries={} committed={} discarded={} (on_encodable={} on_structural={}) committed_on_encodable={} unmatched={} ceiling≈{:.2}x",
            census.grammar_name,
            share.count_files,
            share.total_entries,
            share.total_committed,
            share.total_discarded,
            share.discarded_on_encodable,
            share.discarded_on_non_encodable,
            share.committed_on_encodable,
            share.unmatched_entries,
            share.ceiling_estimate,
        );
        println!(
            "  model: discarded = raw − committed (failed-speculation work; committed keeps C3-B successful losers); kill surface = discarded entries on shape-encodable rules (choice/optional/iteration attempts alike); uniform per-entry cost, scan ≈ terminal match"
        );
        // RGX-0078.5.i.3 (P2) — the measured exposure of the degenerate-dispatch
        // emission: Or-body executions at qualified sites (protocol-elision surface,
        // NOT an entry-kill surface — dispatch skips exactly the branches the
        // `.5.c.2` guards already skip).
        let entry_share_pct = if share.total_entries == 0 {
            0.0
        } else {
            100.0 * share.degenerate_site_entries as f64 / share.total_entries as f64
        };
        println!(
            "DEGENERACY-EXPOSURE: grammar={} degenerate_site_entries={} ({:.1}% of total) committed={} discarded={}",
            census.grammar_name,
            share.degenerate_site_entries,
            entry_share_pct,
            share.degenerate_site_committed,
            share.degenerate_site_discarded,
        );
        println!(
            "  model: entries on rules whose top-level site dispatches degenerately — each such Or-body execution sheds the tournament checkpoint/delta/rollback/replay + guard-scan protocol (memo-hit re-entries counted at full weight)"
        );
        if share.committed_overshoot > 0 {
            println!(
                "  WARNING: committed_overshoot={} (committed > raw on some rules — lookahead-success coverage + memo replay; discards use saturating_sub)",
                share.committed_overshoot
            );
        }
        if !share.unmatched_rules.is_empty() {
            println!(
                "  unmatched outcome-file rules (absent from census): {}",
                share.unmatched_rules.join(", ")
            );
        }
        let mut ranked: Vec<&ChoiceSiteCensus> = census
            .choice_sites
            .iter()
            .filter(|s| s.attributable_discarded > 0)
            .collect();
        ranked.sort_by(|a, b| {
            b.attributable_discarded
                .cmp(&a.attributable_discarded)
                .then(a.rule.cmp(&b.rule))
                .then(a.site.cmp(&b.site))
        });
        if !ranked.is_empty() {
            println!("  top choice sites by sole-attributable discarded entries (lower bounds):");
            for site in ranked.iter().take(15) {
                println!(
                    "    {:>6}  {}@{} ({} branches, {} encodable{})",
                    site.attributable_discarded,
                    site.rule,
                    site.site,
                    site.branches,
                    site.encodable_branches,
                    if site.all_encodable { ", ALL" } else { "" },
                );
            }
        }
    }
    if dump_all && !census.choice_sites.is_empty() {
        println!("  per-site choice verdicts (PGEN_FUSIBILITY_DUMP_ALL):");
        for site in &census.choice_sites {
            let subset: Vec<String> = site
                .branch_verdicts
                .iter()
                .map(|b| {
                    format!(
                        "{}{}{}",
                        b.index,
                        if b.encodable { ":enc" } else { ":-" },
                        if b.text_folding { "+text" } else { "" }
                    )
                })
                .collect();
            println!(
                "    {}@{}{}: {}/{} encodable [{}]{}{}",
                site.rule,
                site.site,
                if site.top_level { " (top)" } else { "" },
                site.encodable_branches,
                site.branches,
                subset.join(" "),
                if site.attributable_discarded > 0 {
                    format!(" attributable_discarded={}", site.attributable_discarded)
                } else {
                    String::new()
                },
                if site.degenerate_dispatch {
                    " [DEGENERATE]".to_string()
                } else if site.top_level {
                    format!(
                        " blocked: {}",
                        site.degeneracy_blockers
                            .first()
                            .map(String::as_str)
                            .unwrap_or("<none>")
                    )
                } else {
                    String::new()
                },
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn atom(token_type: &str, value: &str) -> ASTNode {
        ASTNode::Atom {
            value: ASTValue::Token(vec![
                TokenValue::String(token_type.to_string()),
                TokenValue::String(value.to_string()),
            ]),
        }
    }

    fn rule_ref(name: &str) -> ASTNode {
        atom("rule_reference", name)
    }

    fn or(alternatives: Vec<ASTNode>) -> ASTNode {
        ASTNode::Or { alternatives }
    }

    fn census_of(
        tree: HashMap<String, ASTNode>,
        order: Vec<String>,
        annotations: Option<Annotations>,
    ) -> FusibilityCensus {
        run_fusibility_census("t", &tree, &order, annotations.as_ref(), &[], &[])
            .expect("census runs")
    }

    /// The regex leaf-cascade shape: an unannotated Or over quoted terminals folds to
    /// matched text — increment-1 fusible; a wrapper chain over it stays fusible and the
    /// TOP of the chain is the maximal root when referenced from a non-fusible parent.
    #[test]
    fn terminal_alternation_chain_is_fusible_and_rooted_at_the_top() {
        let mut tree = HashMap::new();
        tree.insert(
            "letter".to_string(),
            or(vec![atom("quoted_string", "a"), atom("quoted_string", "b")]),
        );
        tree.insert("literal_char".to_string(), or(vec![rule_ref("letter")]));
        // A structural (annotated-object-shaped) parent — simulated by a directive-free
        // rule whose default fold is a two-element sequence (a list, not text).
        tree.insert(
            "top".to_string(),
            ASTNode::Sequence {
                elements: vec![rule_ref("literal_char"), rule_ref("literal_char")],
            },
        );
        let census = census_of(
            tree,
            vec!["top".to_string(), "literal_char".to_string(), "letter".to_string()],
            None,
        );
        assert_eq!(census.rules["letter"].tier, FusibilityTier::FusibleToken);
        assert_eq!(census.rules["literal_char"].tier, FusibilityTier::FusibleToken);
        assert_eq!(census.rules["top"].tier, FusibilityTier::NotFusible);
        assert!(census.rules["literal_char"].maximal_root);
        assert!(!census.rules["letter"].maximal_root);
    }

    /// Recursion (a reference cycle) fails gate 1 for every rule on and above the cycle.
    #[test]
    fn reference_cycles_are_not_fusible() {
        let mut tree = HashMap::new();
        tree.insert(
            "a".to_string(),
            or(vec![atom("quoted_string", "x"), rule_ref("b")]),
        );
        tree.insert("b".to_string(), or(vec![rule_ref("a")]));
        let census = census_of(tree, vec!["a".to_string(), "b".to_string()], None);
        assert_eq!(census.rules["a"].tier, FusibilityTier::NotFusible);
        assert_eq!(census.rules["b"].tier, FusibilityTier::NotFusible);
    }

    /// The `unicode_char := !builtin_ascii_char builtin_any_char -> $2` shape: a `$N`
    /// passthrough whose sibling is a zero-width lookahead is text-folding and lands in
    /// the LOOKAHEAD tier (DFA-encodable, tracked separately).
    #[test]
    fn lookahead_guarded_positional_passthrough_is_fusible_lookahead() {
        let mut tree = HashMap::new();
        tree.insert(
            "unicode_char".to_string(),
            ASTNode::Sequence {
                elements: vec![
                    ASTNode::Lookahead {
                        element: Box::new(rule_ref("builtin_ascii_char")),
                        positive: false,
                    },
                    rule_ref("builtin_any_char"),
                ],
            },
        );
        let mut annotations = Annotations::default();
        annotations.branch_return_annotations.insert(
            "unicode_char".to_string(),
            vec![Some(super::super::BranchAnnotation {
                annotation_type: "standard".to_string(),
                annotation_content: "$2".to_string(),
                parsed_ast: Some(UnifiedReturnAST::PositionalRef { index: 2 }),
            })],
        );
        let census = census_of(
            tree,
            vec!["unicode_char".to_string()],
            Some(annotations),
        );
        assert_eq!(
            census.rules["unicode_char"].tier,
            FusibilityTier::FusibleLookahead
        );
    }

    /// A quantified body's default fold is a list — not increment-1 text-folding.
    #[test]
    fn quantified_default_fold_is_not_text() {
        let mut tree = HashMap::new();
        tree.insert(
            "digits".to_string(),
            ASTNode::Quantified {
                element: Box::new(atom("quoted_string", "7")),
                quantifier: "+".to_string(),
            },
        );
        let census = census_of(tree, vec!["digits".to_string()], None);
        assert_eq!(census.rules["digits"].tier, FusibilityTier::NotFusible);
    }

    /// RGX-0078.5.h.1b — the regex `piece`-cascade shape: a structural rule whose
    /// top-level Or mixes an encodable (token-shaped) branch and a recursive
    /// (non-encodable) branch. The site census must find the site, classify exactly
    /// the token branch as encodable, and mark the token rule as a SOLE ref of that
    /// branch (its only grammar-wide reference).
    #[test]
    fn choice_site_census_classifies_mixed_sites_and_sole_refs() {
        let mut tree = HashMap::new();
        tree.insert(
            "tok".to_string(),
            or(vec![atom("quoted_string", "*"), atom("quoted_string", "+")]),
        );
        // top := tok | "(" top ")"   — branch 1 encodable, branch 2 recursive.
        tree.insert(
            "top".to_string(),
            or(vec![
                rule_ref("tok"),
                ASTNode::Sequence {
                    elements: vec![
                        atom("quoted_string", "("),
                        rule_ref("top"),
                        atom("quoted_string", ")"),
                    ],
                },
            ]),
        );
        let census = census_of(tree, vec!["top".to_string(), "tok".to_string()], None);
        // Two sites: top@or#0 (mixed) and tok@or#0 (all-encodable).
        assert_eq!(census.choice_sites.len(), 2);
        let top_site = census
            .choice_sites
            .iter()
            .find(|s| s.rule == "top")
            .expect("top site present");
        assert!(top_site.top_level);
        assert_eq!(top_site.branches, 2);
        assert_eq!(top_site.encodable_branches, 1);
        assert!(!top_site.all_encodable);
        assert!(top_site.branch_verdicts[0].encodable);
        assert!(!top_site.branch_verdicts[1].encodable);
        // `tok` is referenced exactly once grammar-wide — sole to branch 1.
        assert_eq!(top_site.branch_verdicts[0].sole_refs, vec!["tok".to_string()]);
        // `top` recursion in branch 2: NOT sole there (also the entry / self-ref
        // counts as one occurrence — it IS the only occurrence, so sole applies;
        // but the branch is non-encodable so it never feeds attribution).
        let tok_site = census
            .choice_sites
            .iter()
            .find(|s| s.rule == "tok")
            .expect("tok site present");
        assert!(tok_site.all_encodable);
    }

    /// RGX-0078.5.i.3 (P2) — a `@whitespace_sensitive: true` grammar whose top-level
    /// site has pairwise-disjoint single-byte terminal branches qualifies for
    /// degenerate dispatch, with the per-branch byte sets reported.
    #[test]
    fn degenerate_dispatch_qualifies_disjoint_terminal_site() {
        let mut tree = HashMap::new();
        tree.insert(
            "top".to_string(),
            or(vec![atom("quoted_string", "+"), atom("quoted_string", "*")]),
        );
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "top".to_string(),
            vec![super::super::SemanticAnnotation::Named {
                name: "whitespace_sensitive".to_string(),
                ast: super::super::UnifiedSemanticAST::Structured {
                    canonical: String::new(),
                    value: super::super::UnifiedSemanticValue::Boolean(true),
                },
            }],
        );
        let census = census_of(tree, vec!["top".to_string()], Some(annotations));
        let site = census
            .choice_sites
            .iter()
            .find(|s| s.rule == "top")
            .expect("top site present");
        assert!(site.degenerate_dispatch, "blockers: {:?}", site.degeneracy_blockers);
        assert!(site.degeneracy_blockers.is_empty());
        assert_eq!(site.branch_verdicts[0].first_bytes, Some(vec![b'+']));
        assert_eq!(site.branch_verdicts[1].first_bytes, Some(vec![b'*']));
    }

    /// RGX-0078.5.i.3 (P2) — blockers are NAMED: the layout-insensitive default fails
    /// R2, an overlapping first byte fails disjointness, and a nullable branch is not
    /// first-byte-decided. Nested sites fail R1.
    #[test]
    fn degeneracy_blockers_name_layout_overlap_and_undecided_branches() {
        let mut tree = HashMap::new();
        // top := '+' | '+' 'x' | 'a'?   — branches 1/2 share first byte '+',
        // branch 3 is nullable.
        tree.insert(
            "top".to_string(),
            or(vec![
                atom("quoted_string", "+"),
                ASTNode::Sequence {
                    elements: vec![atom("quoted_string", "+"), atom("quoted_string", "x")],
                },
                ASTNode::Quantified {
                    element: Box::new(atom("quoted_string", "a")),
                    quantifier: "?".to_string(),
                },
            ]),
        );
        // No annotations ⇒ whitespace-INSENSITIVE default ⇒ the R2 blocker fires too.
        let census = census_of(tree, vec!["top".to_string()], None);
        let site = census
            .choice_sites
            .iter()
            .find(|s| s.rule == "top")
            .expect("top site present");
        assert!(!site.degenerate_dispatch);
        assert!(site
            .degeneracy_blockers
            .iter()
            .any(|b| b.contains("R2 raw-byte peek unsound")));
        assert!(site
            .degeneracy_blockers
            .iter()
            .any(|b| b.contains("shared by branches 1,2")));
        assert!(site
            .degeneracy_blockers
            .iter()
            .any(|b| b.starts_with("branch 3 not first-byte-decided: nullable")));
        assert_eq!(site.branch_verdicts[2].first_bytes, None);
    }

    /// RGX-0078.5.i.3 (P2) — the pure verdict names the gate-(e) blockers and the
    /// R1 nesting blocker (exercised directly: compiling a branch predicate through
    /// the full annotation path is out of scope for this unit).
    #[test]
    fn site_degeneracy_verdict_names_predicate_effect_and_nesting_blockers() {
        let decided = vec![Ok(vec![b'a']), Ok(vec![b'b'])];
        let (degenerate, blockers) = site_degeneracy_verdict(false, true, true, true, &decided);
        assert!(!degenerate);
        assert!(blockers.iter().any(|b| b.contains("nested Or site")));
        assert!(blockers.iter().any(|b| b.contains("branch-phase predicates")));
        assert!(blockers
            .iter()
            .any(|b| b.contains("branch-start effect directives")));
        // And the all-gates-pass dual.
        let (degenerate, blockers) = site_degeneracy_verdict(true, true, false, false, &decided);
        assert!(degenerate);
        assert!(blockers.is_empty());
    }

    /// RGX-0078.5.h.1b — the outcome join decomposes raw/committed into the
    /// discarded kill surface (encodable vs structural) and fills per-site
    /// sole-attribution. Uses a temp file to exercise the real file path.
    #[test]
    fn outcome_join_decomposes_discarded_work_and_attributes_sites() {
        let mut tree = HashMap::new();
        tree.insert(
            "tok".to_string(),
            or(vec![atom("quoted_string", "*"), atom("quoted_string", "+")]),
        );
        tree.insert(
            "top".to_string(),
            or(vec![
                rule_ref("tok"),
                ASTNode::Sequence {
                    elements: vec![
                        atom("quoted_string", "("),
                        rule_ref("top"),
                        atom("quoted_string", ")"),
                    ],
                },
            ]),
        );
        let payload = serde_json::json!({
            "grammar": "t",
            "accepted": true,
            "total_entries": 30,
            "total_committed": 12,
            // tok: 20 raw, 2 committed → 18 discarded on an ENCODABLE rule.
            // top: 10 raw, 10 committed → 0 discarded (structural spine).
            "rule_entry_counts": {"tok": 20, "top": 10},
            "rule_committed_counts": {"tok": 2, "top": 10},
        });
        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "pgen_fusibility_outcome_test_{}.json",
            std::process::id()
        ));
        std::fs::write(&path, serde_json::to_string(&payload).unwrap()).unwrap();
        let census = run_fusibility_census(
            "t",
            &tree,
            &["top".to_string(), "tok".to_string()],
            None,
            &[],
            std::slice::from_ref(&path),
        )
        .expect("census runs");
        std::fs::remove_file(&path).ok();
        let share = census.outcome_share.as_ref().expect("outcome share joined");
        assert_eq!(share.total_entries, 30);
        assert_eq!(share.total_committed, 12);
        assert_eq!(share.total_discarded, 18);
        assert_eq!(share.discarded_on_encodable, 18); // tok is shape-encodable
        assert_eq!(share.discarded_on_non_encodable, 0);
        assert_eq!(share.committed_overshoot, 0);
        // ceiling = 30 / (30 − 18) = 2.5×
        assert!((share.ceiling_estimate - 2.5).abs() < 1e-9);
        // The mixed top site attributes tok's 18 discarded entries (sole ref of its
        // encodable branch).
        let top_site = census
            .choice_sites
            .iter()
            .find(|s| s.rule == "top")
            .expect("top site present");
        assert_eq!(top_site.attributable_discarded, 18);
    }
}
