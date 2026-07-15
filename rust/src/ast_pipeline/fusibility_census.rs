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
    /// RGX-0078.5.i.4 (P1 STEP-0) — the per-rule inline-eligibility census.
    pub inline_rules: BTreeMap<String, InlineRuleCensus>,
    /// RGX-0078.5.i.4 (P1 STEP-0) — the measured inline-exposure join (present iff
    /// `--fusibility-outcome-counts` files were joined).
    pub inline_exposure: Option<InlineExposure>,
    /// RGX-0078.5.i.7 (Q-GUARD STEP-0) — every quantified site, classified for
    /// the min-0 attempt-elision gate. Deterministic order: rule-universe order,
    /// then pre-order within the rule.
    pub quant_sites: Vec<QuantSiteCensus>,
    /// RGX-0078.5.i.7 (Q-GUARD STEP-0) — the measured quantified-site exposure
    /// join (present iff `--fusibility-outcome-counts` files were joined).
    pub quant_exposure: Option<QuantExposure>,
    /// RGX-0078.5.i.7 (D2 STEP-0) — the per-rule CASCADE-FOLD census.
    pub cascade_rules: BTreeMap<String, CascadeRuleCensus>,
    /// RGX-0078.5.i.7 (D2 STEP-0) — the measured cascade-fold exposure join
    /// (present iff `--fusibility-outcome-counts` files were joined).
    pub cascade_exposure: Option<CascadeExposure>,
    /// RGX-0078.5.i.7 (D2-A) — the SHARED acyclic-sub-region emission plan
    /// ([`compute_cascade_emission_plan`] — the same map codegen consumes).
    pub cascade_plan: CascadeEmissionPlan,
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
    /// RGX-0078.5.i.4 (P1 STEP-0) — per-rule memo HITS (fail-set + valid
    /// tainted-failure + success replays). `#[serde(default)]` keeps pre-`.5.i.4`
    /// dump files loadable (they report zero hits).
    #[serde(default)]
    rule_memo_hit_counts: BTreeMap<String, u64>,
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
    /// RGX-0078.5.i.7 (D1 STEP-0) — the site qualifies for TWO-LEVEL (FIRST₂) prefix
    /// dispatch: the P2 gates minus first-byte disjointness, plus for every first
    /// byte shared by ≥2 branches the admitting subset has RESOLVED second-byte
    /// facts and its non-WILDCARD members are pairwise-disjoint on second bytes. A
    /// byte-2 WILDCARD (`len1_possible` — some one-byte match leaves byte 2
    /// unconstrained) is legal (it joins every second-byte arm) but caps the kill;
    /// see `prefix2_wildcard_branches`. Judged independently of
    /// `degenerate_dispatch` (a degenerate site needs no second level — consumers
    /// filter on `!degenerate_dispatch`).
    pub prefix2_dispatchable: bool,
    /// The NAMED failing FIRST₂ gates (empty iff `prefix2_dispatchable`).
    pub prefix2_blockers: Vec<String>,
    /// Branch indices (1-based, sorted, deduped) that are byte-2 WILDCARDS inside
    /// some shared-first-byte subset.
    pub prefix2_wildcard_branches: Vec<usize>,
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
    /// RGX-0078.5.i.7 (D1 STEP-0) — raw entries on rules whose TOP-LEVEL site is
    /// `prefix2_dispatchable` but NOT `degenerate_dispatch` (the D1 emission
    /// surface — a P2 site needs no second level).
    pub prefix2_site_entries: u64,
    /// Of `prefix2_site_entries`: the committed part.
    pub prefix2_site_committed: u64,
    /// Of `prefix2_site_entries`: the discarded part.
    pub prefix2_site_discarded: u64,
}

/// RGX-0078.5.i.4 (P1 STEP-0) — the body shape of an inline-ELIGIBLE rule, for
/// sizing the emission increments (which shapes dominate the collapsible frames).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InlineWrapperClass {
    /// The body is (after unwrapping single-element shells) exactly one rule
    /// reference — the `entry_alternation`-style delegation frame.
    PassThrough,
    /// A top-level alternation (≥2 branches) whose subtree contains NO rule
    /// references — the `letter`/`digit` terminal-leaf shape (post-P2 these are
    /// the byte-switch bodies).
    AlternationLeaf,
    /// Any other eligible body (mixed terminals + refs, sequences, quantifiers).
    Shaped,
}

/// RGX-0078.5.i.4 (P1 STEP-0) — one rule's INLINE-eligibility verdict under the
/// P1 gates (leaf spec a–d): (a) the rule participates in no reference cycle (its
/// recursion guard is provably non-load-bearing), (b) it carries no semantic
/// directives in any phase / no value constraints / no follow restrictions (its
/// frame's transactional role is trace-naming only), (c) it is not the entry rule,
/// (d) it is not `@profiles`-gated. Verdicts are conservative under-approximations
/// (a blocked rule is never falsely eligible).
#[derive(Debug, Clone, serde::Serialize)]
pub struct InlineRuleCensus {
    pub eligible: bool,
    /// NAMED failing gates (empty iff `eligible`). Deterministic order: cycle,
    /// runtime directives (sorted kinds), mid-sequence, follow restriction, value
    /// constraint, branch predicate/effect, rule-level @transform, @profiles, entry.
    pub blockers: Vec<String>,
    /// Present iff `eligible`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wrapper_class: Option<InlineWrapperClass>,
    /// Grammar-wide reference OCCURRENCES of this rule — the code-duplication
    /// factor an inlining emission pays (every call site receives a body copy).
    pub reference_sites: usize,
    /// Gen-AST node count of the rule body — the per-site duplication size.
    pub body_nodes: usize,
    /// RGX-0078.5.i.4 (P1a) — the EMISSION decision under the shared code-size
    /// budget ([`compute_inline_decisions`]): true iff codegen inlines this
    /// rule's body at its call sites. `eligible && !decided` = over-budget.
    pub decided: bool,
}

/// RGX-0078.5.i.4 (P1 STEP-0) — the measured inline-exposure join (inline census ×
/// raw/committed/memo-hit outcome counts): how much real parse work sits on
/// collapsible wrapper frames, and how much of it is memo-hit replay (the P1b
/// lost-hit surface — each such hit becomes a body re-execution if the memo is
/// elided at inlined sites).
#[derive(Debug, Clone, serde::Serialize)]
pub struct InlineExposure {
    /// Raw entries on inline-eligible rules — the collapsible frame count.
    pub eligible_entries: u64,
    /// Of `eligible_entries`: the committed (C3-B surviving) part.
    pub eligible_committed: u64,
    /// Of `eligible_entries`: the discarded (failed-speculation) part.
    pub eligible_discarded: u64,
    /// Memo HITS on eligible rules (from `rule_memo_hit_counts`; requires dumps
    /// written by a parser generation that records them — older dump files
    /// deserialize with zero hits, loudly visible as `total_memo_hits=0`).
    pub eligible_memo_hits: u64,
    /// Memo hits across ALL rules (context for the eligible share).
    pub total_memo_hits: u64,
    /// RGX-0078.5.i.4 (P1b pricing) — the DECIDED subset of the eligible sums:
    /// entries on rules the shared budget actually inlines
    /// ([`compute_inline_decisions`]). This is the exposure the landed P1a
    /// emission collapses, and the surface P1b's memo elision prices —
    /// `decided_memo_hits` is the lost-hit population that would re-execute.
    pub decided_entries: u64,
    /// Of `decided_entries`: the committed (C3-B surviving) part.
    pub decided_committed: u64,
    /// Of `decided_entries`: the discarded (failed-speculation) part.
    pub decided_discarded: u64,
    /// Memo hits on decided rules — the P1b lost-hit surface at the budget.
    pub decided_memo_hits: u64,
    /// Top eligible rules by raw entries: (rule, entries, committed, memo_hits).
    pub top_eligible_rules: Vec<(String, u64, u64, u64)>,
}

/// RGX-0078.5.i.7 (Q-GUARD STEP-0) — one QUANTIFIED site of the grammar: where a
/// FIRST-guarded attempt elision (skip the element's `try_parse` when
/// `input[p] ∉ FIRST(element)`, with the EXACT furthest emulation
/// `if p > furthest { furthest = p }`) could replace the refuted attempt.
/// Unlike the Or-branch guards there is NO rule-top-level restriction: a min-0
/// quantifier ALWAYS attempts its element exactly once at the current position,
/// and a byte-1-refuted attempt performs every rule entry AT that position (the
/// `-0075` exactness lemma), so the emulation is exact wherever the site sits.
#[derive(Debug, Clone, serde::Serialize)]
pub struct QuantSiteCensus {
    pub rule: String,
    /// Site id within the rule: `q#N` in pre-order walk order.
    pub site: String,
    /// The literal quantifier text (`*`, `?`, `+`, `{,M}`, …).
    pub quantifier: String,
    /// The quantifier's minimum repeat count is 0 (`*` / `?` / `{,M}` / `{0,M}`)
    /// — the attempt-elision lane (min>0 sites are censused for steering only).
    pub min_zero: bool,
    /// ALL Q-guard gates pass: min-0 + terminal-layout trust + element
    /// first-byte-decided + predicate/effect-free reachable closure.
    pub guardable: bool,
    /// The NAMED failing gates (empty iff `guardable`). Deterministic order:
    /// min>0, layout, element FIRST verdict, reachable predicate/effect rules.
    pub blockers: Vec<String>,
    /// The guard byte set (sorted) when the element is first-byte-decided —
    /// present even on sites blocked by OTHER gates, for steering.
    pub first_bytes: Option<Vec<u8>>,
    /// RGX-0078.5.i.7 Q-GUARD EMISSION — the furthest-emulation frontier class of
    /// the element (`bare_ref` / `no_refs` / `mixed`, the SHARED
    /// `first_set::quantified_element_frontier` verdict). The emission guards ONLY
    /// `bare_ref` (guard + exact `furthest` emulation) and `no_refs` (guard, no
    /// emulation) sites; `mixed` sites stay unguarded (exact emulation undecidable
    /// at this granularity) even when `guardable` is true.
    pub frontier: String,
    /// Rule references anywhere in the element subtree (deduped, sorted).
    pub element_refs: Vec<String>,
    /// Of `element_refs`: rules whose EVERY grammar-wide reference occurrence
    /// lives in THIS site's element subtree (the per-site sound attribution
    /// basis, mirroring `ChoiceBranchVerdict::sole_refs`). A rule referenced
    /// from several quantified sites (the `class_zero_width` shape) is NOT
    /// site-sole — the population lanes in [`QuantExposure`] cover it.
    pub sole_refs: Vec<String>,
    /// Measured DISCARDED entries on this site's `sole_refs` (only meaningful
    /// when outcome counts were joined; 0 otherwise).
    pub sole_attributable_discarded: u64,
}

/// RGX-0078.5.i.7 (Q-GUARD STEP-0) — the measured quantified-site exposure join.
/// POPULATION-level attribution: per-rule aggregate counters cannot split a rule
/// referenced from several sites, so a rule counts as ATTRIBUTABLE only when every
/// grammar-wide reference occurrence sits inside a GUARDABLE site's element
/// subtree (occurrences are counted at OUTERMOST guardable sites, so a guardable
/// site nested inside another guardable site's element never double-counts).
/// HONEST BOUNDS: `attributable_discarded` OVER-approximates the guard's kill —
/// a byte-1-ADMITTED, byte-2+-refuted attempt survives the guard (the D1 residual
/// lesson) — while guardable sites whose element is pure terminal probing are
/// invisible to per-rule counters entirely (an uncounted upside).
#[derive(Debug, Clone, serde::Serialize)]
pub struct QuantExposure {
    /// Quantified sites censused / the min-0 subset / the guardable subset.
    pub total_sites: usize,
    pub min_zero_sites: usize,
    pub guardable_sites: usize,
    /// Rules attributable to guardable sites (every occurrence under one),
    /// sorted, with their raw/committed/discarded sums.
    pub attributable_rules: Vec<String>,
    pub attributable_entries: u64,
    pub attributable_committed: u64,
    pub attributable_discarded: u64,
    /// Rules under ≥1 guardable site but ALSO referenced elsewhere — exposure
    /// context (an upper bound on top of the attributable lane), never priced.
    pub shared_rules: Vec<String>,
    pub shared_entries: u64,
    pub shared_discarded: u64,
    /// Top attributable rules by discarded entries:
    /// (rule, entries, committed, discarded).
    pub top_attributable_rules: Vec<(String, u64, u64, u64)>,
}

/// RGX-0078.5.i.7 (D2 STEP-0) — one rule's CASCADE-FOLD verdict: can the rule live
/// INSIDE a fused direct-coded region ("emit what a hand-written parser would be"
/// for the committed descent)? This gate is deliberately DIFFERENT from the
/// scanner-rung tier gate above, because a fused *matcher* (specialized straight-line
/// / recursive Rust) is strictly more expressive than a DFA:
/// - reference CYCLES are ALLOWED (a region emits specialized recursive functions;
///   `on_cycle` is a named fact, not a blocker — the emission design owns the memo
///   soundness obligations it raises),
/// - multi-lexeme / layout-skipping bodies are ALLOWED (a fused matcher skips layout
///   inline where the protocol descent does today),
/// - lookahead is ALLOWED (position-reset probes are cheap in an effect-free region),
/// - EVERY `UnifiedReturnAST` value shape is emittable (all variants are static
///   constructors/selectors over child results — there is no dynamic value form),
///   so the value side blocks only on `@transform` and value constraints.
///
/// What DOES block: runtime semantic directives in any phase (store effects and
/// predicates need the transactional protocol), mid-sequence inline directives,
/// lexical follow restrictions, `@profiles` dialect gates, `@transform`,
/// `@associativity nonassoc`, and `@deterministic_group` rotation.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CascadeRuleCensus {
    pub eligible: bool,
    /// Named blockers (empty for eligible rules).
    pub reasons: Vec<String>,
    /// Eligible AND (the entry rule, referenced by ≥1 INELIGIBLE rule, or
    /// unreferenced): a fused-region ROOT — the protocol↔fused boundary where a
    /// specialized region function would be called from ordinary generated code.
    /// Entries at roots are NOT counted as eliminated (conservative: a rule
    /// referenced from both inside and outside regions is classed a root, so its
    /// inside-region entries are under-counted as kills).
    pub root: bool,
    /// The rule sits on a reference cycle (named fact — allowed inside a region).
    pub on_cycle: bool,
    /// Tree-defined rules this rule references that are NOT cascade-eligible: the
    /// region's protocol call-outs (a fused function calls the ordinary generated
    /// method at these edges). Sorted, deduped; empty for ineligible rules.
    pub boundary_refs: Vec<String>,
}

/// RGX-0078.5.i.7 (D2 STEP-0) — the measured cascade-fold exposure join (census ×
/// per-parse outcome counts): how much of the bench's COMMITTED work sits inside
/// fused regions. `internal_*` = entries on eligible non-root rules (the per-entry
/// protocol a fold eliminates outright); `root_*` = entries on region roots (each
/// becomes one specialized-function call — kept, first-order); `residual_*` =
/// entries on ineligible rules (untouched). `committed_floor` = root_committed +
/// residual_committed: the first-order post-fold count of protocol-paying committed
/// entries — the honest denominator for any D2 pricing.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CascadeExposure {
    pub count_files: usize,
    pub total_entries: u64,
    pub internal_entries: u64,
    pub internal_committed: u64,
    pub internal_discarded: u64,
    /// Memo hits on internal rules: replays a memo-free fused region re-executes
    /// (named fact; the emission design owns the re-probe boundedness proof).
    pub internal_memo_hits: u64,
    pub root_entries: u64,
    pub root_committed: u64,
    pub root_discarded: u64,
    pub root_memo_hits: u64,
    pub residual_entries: u64,
    pub residual_committed: u64,
    pub residual_discarded: u64,
    /// Rule names present in the counts files but absent from the census (e.g.
    /// LR-elimination-synthesized helpers). Never silently dropped.
    pub unmatched_rules: Vec<String>,
    pub unmatched_entries: u64,
    pub committed_floor: u64,
    /// Top internal rules by committed entries: (rule, entries, committed, discarded).
    pub top_internal_rules: Vec<(String, u64, u64, u64)>,
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
    /// RGX-0078.5.i.7 (D1 STEP-0) — shared SECOND-byte cache (the FIRST₂ analysis).
    second_byte_cache: HashMap<String, super::first_set::SecondByteSummary>,
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
            second_byte_cache: HashMap::new(),
            memo: HashMap::new(),
            visiting: HashSet::new(),
        })
    }

    /// RGX-0078.5.i.3 (P2) — the branch's admissible dispatch FIRST bytes, or the
    /// NAMED reason the branch is not first-byte-decided. Thin wrapper over the
    /// SHARED eligibility predicate (`first_set::branch_dispatch_first_bytes`) so
    /// this census verdict and codegen's degenerate-dispatch gate cannot drift.
    /// The D0 trust flag mirrors codegen's `layout_sensitivity().regex_tokens`.
    /// HONEST BOUND (D0): the census analyzes the RAW gen-AST, while codegen's
    /// snapshot rewrites regex atoms through `effective_regex_pattern` (token
    /// steering) — drift is currently ∅ (no tracked grammar steers tokens; the sole
    /// live special case is first-byte-equivalent) and the census is diagnostic-only.
    fn branch_dispatch_first_bytes(&mut self, branch: &ASTNode) -> Result<Vec<u8>, String> {
        super::first_set::branch_dispatch_first_bytes(
            branch,
            self.tree,
            &mut self.first_set_cache,
            self.layout.regex_tokens,
        )
    }

    /// RGX-0078.5.i.7 (D1 STEP-0) — the branch's SECOND-byte facts under the same
    /// layout-trust discipline as the level-1 predicate: regex-token-derived facts
    /// are honored only when regex tokens are whitespace-sensitive; otherwise the
    /// whole summary degrades to unresolved (never a partial byte set).
    fn branch_second_byte_summary(
        &mut self,
        branch: &ASTNode,
    ) -> super::first_set::SecondByteSummary {
        let mut visiting = super::first_set::RuleVisit::default();
        let summary = super::first_set::branch_second_byte_summary(
            branch,
            self.tree,
            &mut self.first_set_cache,
            &mut self.second_byte_cache,
            &mut visiting,
            0,
        );
        if summary.regex_token_derived && !self.layout.regex_tokens {
            return super::first_set::SecondByteSummary {
                unresolved: true,
                ..Default::default()
            };
        }
        summary
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

    /// RGX-0078.5.i.4 (P1 STEP-0) — one rule's INLINE-eligibility verdict under the
    /// P1 gates (leaf spec a–d). Thin wrapper over the SHARED free function
    /// [`rule_inline_verdict`] so this census verdict and codegen's P1a inline
    /// emission gate cannot drift (the P2 `first_set::branch_dispatch_first_bytes`
    /// precedent).
    fn inline_rule_verdict(
        &self,
        rule: &str,
        on_cycle: bool,
        is_entry: bool,
    ) -> (bool, Vec<String>) {
        rule_inline_verdict(
            self.tree,
            self.annotations,
            self.compiled.as_ref(),
            rule,
            on_cycle,
            is_entry,
        )
    }

    /// RGX-0078.5.i.7 (D2 STEP-0) — one rule's CASCADE-FOLD eligibility (see
    /// [`CascadeRuleCensus`]): the EFFECT-freedom + policy-encodability subset of the
    /// scanner gate, with the cycle / layout-contiguity / text-folding / DFA-shape
    /// criteria deliberately absent (a fused matcher handles all four). Local (no
    /// closure walk): a reference to an ineligible rule is a region BOUNDARY, never
    /// a blocker.
    fn cascade_rule_verdict(&self, rule: &str) -> (bool, Vec<String>) {
        let mut reasons: Vec<String> = Vec::new();
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
                    "transform" => reasons.push("matched-text @transform".to_string()),
                    "profiles" => reasons.push("@profiles dialect gate".to_string()),
                    _ => {}
                }
            }
        }
        if !effective_rule_value_constraints(self.annotations, rule).is_empty() {
            reasons.push("value constraint (@enum/@regex/@range/@len)".to_string());
        }
        if effective_rule_associativity(self.annotations, rule) == SemanticAssociativity::NonAssoc {
            reasons.push("@associativity nonassoc (equal-tie failure semantics)".to_string());
        }
        if effective_rule_deterministic_partition_policy(self.annotations, rule).enabled {
            reasons.push("@deterministic_group evaluation-order rotation".to_string());
        }
        let mut deduped: Vec<String> = Vec::new();
        for r in reasons {
            if !deduped.contains(&r) {
                deduped.push(r);
            }
        }
        (deduped.is_empty(), deduped)
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
pub(crate) fn collect_ref_occurrences(node: &ASTNode, out: &mut HashMap<String, usize>) {
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

/// RGX-0078.5.i.4 (P1 STEP-0) — gen-AST node count of a subtree (the per-site
/// duplication size an inlining emission pays).
fn count_nodes(node: &ASTNode) -> usize {
    1 + match node {
        ASTNode::Or { alternatives } => alternatives.iter().map(count_nodes).sum(),
        ASTNode::Sequence { elements } => elements.iter().map(count_nodes).sum(),
        ASTNode::Quantified { element, .. } | ASTNode::Lookahead { element, .. } => {
            count_nodes(element)
        }
        ASTNode::Atom { value } => match value {
            ASTValue::Node(inner) => count_nodes(inner),
            ASTValue::Token(_) => 0,
        },
    }
}

/// RGX-0078.5.i.4 (P1 STEP-0) — does `rule` reach ITSELF through ≥1 reference edge?
/// Exactly the fact that makes its recursion guard load-bearing (the guard detects
/// (rule, position) re-entry; a rule off every cycle can never re-enter itself).
/// Unresolved references (codegen builtins / undefined) expand to nothing.
fn rule_reaches_itself(rule: &str, forward: &HashMap<String, HashSet<String>>) -> bool {
    let mut seen: HashSet<&str> = HashSet::new();
    let mut stack: Vec<&str> = forward
        .get(rule)
        .map(|s| s.iter().map(String::as_str).collect())
        .unwrap_or_default();
    while let Some(current) = stack.pop() {
        if current == rule {
            return true;
        }
        if seen.insert(current) {
            if let Some(next) = forward.get(current) {
                stack.extend(next.iter().map(String::as_str));
            }
        }
    }
    false
}

/// RGX-0078.5.i.4 (P1) — one rule's INLINE-eligibility verdict under the P1 gates
/// (leaf spec a–d), with every failing gate NAMED. THE shared predicate: both the
/// census (`INLINE-CENSUS` reporting) and codegen's P1a inline emission consume this
/// one function, so the two cannot drift (the P2 `first_set` precedent). Facts come
/// from the SAME compiled runtime-annotation table the generated parser consults.
/// Deterministic blocker order: cycle, runtime directives (sorted kinds),
/// mid-sequence, follow restriction, value constraint, branch predicate/effect,
/// rule-level @transform, @profiles, entry rule.
pub fn rule_inline_verdict(
    tree: &HashMap<String, ASTNode>,
    annotations: Option<&Annotations>,
    compiled: Option<&CompiledSemanticRuntimeAnnotations>,
    rule: &str,
    on_cycle: bool,
    is_entry: bool,
) -> (bool, Vec<String>) {
    let mut blockers: Vec<String> = Vec::new();
    // Gate (a) — acyclicity: the recursion guard is load-bearing on a cycle.
    if on_cycle {
        blockers.push("reference cycle (recursion guard is load-bearing)".to_string());
    }
    // Gate (b) — directive-free frame: any runtime semantic directive (any
    // phase, rule- or branch-attached) makes the frame's transactional /
    // context role real, not trace-naming-only.
    if let Some(compiled) = compiled {
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
        for kind in kinds {
            blockers.push(format!("runtime directive @{kind}"));
        }
    }
    if let Some(ann) = annotations {
        if ann
            .branch_mid_sequence_semantic_annotations
            .get(rule)
            .is_some_and(|branches| branches.iter().any(|b| !b.is_empty()))
        {
            blockers.push("mid-sequence inline directive".to_string());
        }
        if ann.lexical_follow_restrictions.contains_key(rule) {
            blockers.push("lexical follow restriction [> …]".to_string());
        }
    }
    if !effective_rule_value_constraints(annotations, rule).is_empty() {
        blockers.push("value constraint (@enum/@regex/@range/@len)".to_string());
    }
    let branch_count = match tree.get(rule) {
        Some(ASTNode::Or { alternatives }) => alternatives.len(),
        _ => 1,
    };
    if let Some(compiled) = compiled {
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
        if has_branch_predicates {
            blockers.push("branch-phase predicate".to_string());
        }
        if has_branch_start_effects {
            blockers.push("branch-start effect directive".to_string());
        }
    }
    if let Some(ann) = annotations {
        for name in rule_level_directive_names(ann, rule) {
            match name.as_str() {
                // A rule-level matched-text transform is reproducible in
                // principle but out of the P1 emission increment —
                // conservatively blocked (a blocked rule is never falsely
                // eligible).
                "transform" => blockers.push("rule-level @transform".to_string()),
                // Gate (d) — a profile-gated rule must resolve identically at
                // every call site under every declared profile; blocked.
                "profiles" => blockers.push("@profiles dialect gate".to_string()),
                _ => {}
            }
        }
    }
    // Gate (c) — the entry rule keeps its method as the parse entry point
    // (vacuous for exposure: it has no call sites; named for honesty).
    if is_entry {
        blockers.push("entry rule".to_string());
    }
    (blockers.is_empty(), blockers)
}

/// RGX-0078.5.i.4 (P1a) — the emission EXPANSION cap: a rule whose capped-transitive
/// inlined-body weight (gen-AST nodes, decided children expanded) exceeds this stays
/// a method call. Chosen from the measured weight model over the 8-pattern bench
/// outcome dumps (leaf `.5.i.4` P1a design record): T=12 keeps ~49% of the eligible
/// bench-entry exposure at ×1.75 emitted-body growth on regex.
pub const INLINE_EXPANSION_CAP: usize = 12;

/// RGX-0078.5.i.4 (P1a) — the emission DUPLICATION cap: a rule whose
/// `weight × grammar-wide reference sites` exceeds this stays a method call (its
/// total body duplication would dominate the emitted-code growth; `digit` at 11
/// nodes × 124 sites is the canonical exclusion). Same measured basis as
/// [`INLINE_EXPANSION_CAP`].
pub const INLINE_DUPLICATION_CAP: usize = 192;

/// RGX-0078.5.i.4 (P1a) — one rule's inline-emission decision under the budget.
#[derive(Debug, Clone, serde::Serialize)]
pub struct InlineDecision {
    /// The gates (a)–(d) verdict from [`rule_inline_verdict`].
    pub eligible: bool,
    /// Capped-transitive inlined-body weight (gen-AST nodes; decided children
    /// expanded, undecided references stay method calls contributing nothing).
    pub weight: usize,
    /// Grammar-wide reference OCCURRENCES of this rule (the duplication factor).
    pub sites: usize,
    /// `eligible && weight ≤ INLINE_EXPANSION_CAP && weight×sites ≤
    /// INLINE_DUPLICATION_CAP` — the rule's call sites receive its body inline.
    pub decided: bool,
}

/// RGX-0078.5.i.4 (P1a) — the SHARED inline-emission decision map: gates (a)–(d)
/// plus the measured code-size budget, computed bottom-up over the (provably
/// acyclic) eligible reference subgraph. Codegen consumes this for the P1a
/// emission; the census reports it (`INLINE-DECISIONS`), so the two cannot drift.
/// Deterministic: memoized structural recursion; `BTreeMap` output.
pub fn compute_inline_decisions(
    tree: &HashMap<String, ASTNode>,
    annotations: Option<&Annotations>,
    compiled: Option<&CompiledSemanticRuntimeAnnotations>,
    entry_rule: Option<&str>,
) -> BTreeMap<String, InlineDecision> {
    // Reference closure + occurrence counts via the census's OWN collectors
    // (one implementation of "what is a reference" for verdicts and budget).
    let mut forward: HashMap<String, HashSet<String>> = HashMap::new();
    let mut regex_pattern_sink: Vec<String> = Vec::new();
    let mut body_occurrences: HashMap<String, HashMap<String, usize>> = HashMap::new();
    let mut sites: HashMap<String, usize> = HashMap::new();
    let mut local_nodes: HashMap<String, usize> = HashMap::new();
    for (rule, body) in tree {
        let mut refs = HashSet::new();
        collect_refs(body, &mut refs, &mut regex_pattern_sink);
        forward.insert(rule.clone(), refs);
        let mut occ = HashMap::new();
        collect_ref_occurrences(body, &mut occ);
        for (target, n) in &occ {
            *sites.entry(target.clone()).or_default() += n;
        }
        body_occurrences.insert(rule.clone(), occ);
        local_nodes.insert(rule.clone(), count_nodes(body));
    }
    let eligible: HashMap<&str, bool> = tree
        .keys()
        .map(|rule| {
            let on_cycle = rule_reaches_itself(rule, &forward);
            let is_entry = Some(rule.as_str()) == entry_rule;
            (
                rule.as_str(),
                rule_inline_verdict(tree, annotations, compiled, rule, on_cycle, is_entry).0,
            )
        })
        .collect();

    struct Ctx<'a> {
        eligible: &'a HashMap<&'a str, bool>,
        body_occurrences: &'a HashMap<String, HashMap<String, usize>>,
        local_nodes: &'a HashMap<String, usize>,
        sites: &'a HashMap<String, usize>,
        weight_memo: HashMap<String, usize>,
        decided_memo: HashMap<String, bool>,
        visiting: HashSet<String>,
    }
    impl Ctx<'_> {
        fn weight(&mut self, rule: &str) -> usize {
            if let Some(&w) = self.weight_memo.get(rule) {
                return w;
            }
            if !self.visiting.insert(rule.to_string()) {
                // A cycle among decided rules is impossible by gate (a);
                // defensively price re-entry as over-budget so the emitter
                // can never recurse into it.
                return usize::MAX / 4;
            }
            let mut w = self.local_nodes.get(rule).copied().unwrap_or(0);
            if let Some(occ) = self.body_occurrences.get(rule) {
                let targets: Vec<(String, usize)> =
                    occ.iter().map(|(t, n)| (t.clone(), *n)).collect();
                for (target, n) in targets {
                    if self.decided(&target) {
                        w = w.saturating_add(self.weight(&target).saturating_mul(n));
                    }
                }
            }
            self.visiting.remove(rule);
            self.weight_memo.insert(rule.to_string(), w);
            w
        }
        fn decided(&mut self, rule: &str) -> bool {
            if let Some(&d) = self.decided_memo.get(rule) {
                return d;
            }
            let d = self.eligible.get(rule).copied().unwrap_or(false) && {
                let w = self.weight(rule);
                w <= INLINE_EXPANSION_CAP
                    && w.saturating_mul(self.sites.get(rule).copied().unwrap_or(0))
                        <= INLINE_DUPLICATION_CAP
            };
            self.decided_memo.insert(rule.to_string(), d);
            d
        }
    }
    let mut ctx = Ctx {
        eligible: &eligible,
        body_occurrences: &body_occurrences,
        local_nodes: &local_nodes,
        sites: &sites,
        weight_memo: HashMap::new(),
        decided_memo: HashMap::new(),
        visiting: HashSet::new(),
    };
    tree.keys()
        .map(|rule| {
            let decided = ctx.decided(rule);
            let weight = ctx.weight(rule);
            (
                rule.clone(),
                InlineDecision {
                    eligible: eligible.get(rule.as_str()).copied().unwrap_or(false),
                    weight,
                    sites: sites.get(rule).copied().unwrap_or(0),
                    decided,
                },
            )
        })
        .collect()
}

/// RGX-0078.5.i.4 (P1 STEP-0) — the body-shape class of an inline-eligible rule.
fn inline_wrapper_class(body: &ASTNode) -> InlineWrapperClass {
    fn effective(node: &ASTNode) -> &ASTNode {
        match node {
            ASTNode::Sequence { elements } if elements.len() == 1 => effective(&elements[0]),
            ASTNode::Or { alternatives } if alternatives.len() == 1 => {
                effective(&alternatives[0])
            }
            ASTNode::Atom {
                value: ASTValue::Node(inner),
            } => effective(inner),
            other => other,
        }
    }
    let core = effective(body);
    if let ASTNode::Atom {
        value: ASTValue::Token(parts),
    } = core
    {
        if parts.len() >= 2 {
            let TokenValue::String(token_type) = &parts[0];
            if token_type == "rule_reference" {
                return InlineWrapperClass::PassThrough;
            }
        }
    }
    if let ASTNode::Or { alternatives } = core {
        if alternatives.len() >= 2 {
            let mut refs: HashMap<String, usize> = HashMap::new();
            collect_ref_occurrences(body, &mut refs);
            if refs.is_empty() {
                return InlineWrapperClass::AlternationLeaf;
            }
        }
    }
    InlineWrapperClass::Shaped
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

/// RGX-0078.5.i.7 (D1 STEP-0) — the per-site FIRST₂ (two-level prefix dispatch)
/// verdict: can the site dispatch on byte 1 to an admitting subset and, within an
/// overlapping subset, on byte 2 to (at most) one non-wildcard candidate? Pure over
/// its inputs (unit-testable in isolation). Gates = the P2 gates MINUS first-byte
/// disjointness, PLUS per shared first byte: every admitting branch's second-byte
/// facts RESOLVED, and the non-WILDCARD members pairwise-disjoint on second bytes.
/// A `len1_possible` member is a byte-2 wildcard — legal (it joins every byte-2 arm,
/// including end-of-input) but it caps the kill; reported, never a blocker. An
/// `offset1_rule_entry` member (the D1 emission's furthest-position-parity license)
/// is classified the same way: the guard emission cannot byte-2-prune it, so it
/// joins every second-byte arm exactly like a wildcard.
fn site_prefix2_verdict(
    top_level: bool,
    layout_terminals: bool,
    has_branch_phase_predicates: bool,
    has_branch_start_effects: bool,
    branch_first_bytes: &[Result<Vec<u8>, String>],
    branch_seconds: &[super::first_set::SecondByteSummary],
) -> (bool, Vec<String>, Vec<usize>) {
    let mut blockers: Vec<String> = Vec::new();
    let mut wildcards: std::collections::BTreeSet<usize> = std::collections::BTreeSet::new();
    if !top_level {
        blockers.push(
            "nested Or site (rule-top-level only — R1 furthest-position neutrality)".to_string(),
        );
        return (false, blockers, Vec::new());
    }
    if !layout_terminals {
        blockers.push("terminals skip leading layout (R2 raw-byte peek unsound)".to_string());
    }
    for (i, bytes) in branch_first_bytes.iter().enumerate() {
        if let Err(reason) = bytes {
            blockers.push(format!("branch {} not first-byte-decided: {reason}", i + 1));
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
    let mut byte_owners: BTreeMap<u8, Vec<usize>> = BTreeMap::new();
    for (i, bytes) in branch_first_bytes.iter().enumerate() {
        if let Ok(bytes) = bytes {
            for byte in bytes {
                byte_owners.entry(*byte).or_default().push(i + 1);
            }
        }
    }
    for (byte, owners) in &byte_owners {
        if owners.len() < 2 {
            continue;
        }
        let printable = if byte.is_ascii_graphic() {
            format!(" ('{}')", *byte as char)
        } else {
            String::new()
        };
        let mut second_owners: BTreeMap<u8, Vec<usize>> = BTreeMap::new();
        for &owner in owners {
            let second = &branch_seconds[owner - 1];
            if second.unresolved {
                blockers.push(format!(
                    "first byte 0x{byte:02X}{printable}: branch {owner} second bytes UNRESOLVED"
                ));
                continue;
            }
            if second.len1_possible || second.nullable || second.offset1_rule_entry {
                // `offset1_rule_entry` (D1 emission license): a member the guard
                // emission cannot byte-2-prune (furthest-position parity) behaves
                // exactly like a wildcard — it joins every second-byte arm.
                wildcards.insert(owner);
                continue;
            }
            for second_byte in &second.second_bytes {
                second_owners.entry(*second_byte).or_default().push(owner);
            }
        }
        for (second_byte, second_shared) in &second_owners {
            if second_shared.len() >= 2 {
                let printable2 = if second_byte.is_ascii_graphic() {
                    format!(" ('{}')", *second_byte as char)
                } else {
                    String::new()
                };
                blockers.push(format!(
                    "first byte 0x{byte:02X}{printable}: SECOND byte 0x{second_byte:02X}{printable2} shared by branches {}",
                    second_shared
                        .iter()
                        .map(|n| n.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                ));
            }
        }
    }
    (
        blockers.is_empty(),
        blockers,
        wildcards.into_iter().collect(),
    )
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
                // RGX-0078.5.i.7 (D1 STEP-0) — the FIRST₂ verdict. Second-byte
                // summaries are computed only for top-level sites (nested sites are
                // R1-blocked at level 2 exactly as at level 1).
                let (prefix2_dispatchable, prefix2_blockers, prefix2_wildcard_branches) =
                    if is_rule_body {
                        let branch_seconds: Vec<super::first_set::SecondByteSummary> =
                            alternatives
                                .iter()
                                .map(|branch| classifier.branch_second_byte_summary(branch))
                                .collect();
                        site_prefix2_verdict(
                            is_rule_body,
                            classifier.layout.terminals,
                            has_branch_predicates,
                            has_branch_start_effects,
                            &branch_dispatch_bytes,
                            &branch_seconds,
                        )
                    } else {
                        (
                            false,
                            vec![
                                "nested Or site (rule-top-level only — R1 furthest-position neutrality)"
                                    .to_string(),
                            ],
                            Vec::new(),
                        )
                    };
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
                    prefix2_dispatchable,
                    prefix2_blockers,
                    prefix2_wildcard_branches,
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

/// RGX-0078.5.i.7 (Q-GUARD STEP-0) — the transitive rule-reference closure from a
/// seed set: an over-approximation of the rules a refuted element attempt could
/// enter before its first terminal match fails (references behind consuming units
/// are unreachable in a byte-1-refuted attempt but still count — the sound,
/// no-elision direction, mirroring `contains_rule_reference_shallow`'s stance).
pub(crate) fn reachable_rules(
    tree: &HashMap<String, ASTNode>,
    seeds: impl Iterator<Item = String>,
) -> Vec<String> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut stack: Vec<String> = seeds.collect();
    while let Some(rule) = stack.pop() {
        if !seen.insert(rule.clone()) {
            continue;
        }
        if let Some(body) = tree.get(rule.as_str()) {
            let mut refs: HashMap<String, usize> = HashMap::new();
            collect_ref_occurrences(body, &mut refs);
            for name in refs.keys() {
                if !seen.contains(name) {
                    stack.push(name.clone());
                }
            }
        }
    }
    seen.into_iter().collect()
}

/// RGX-0078.5.i.7 (Q-GUARD STEP-0) — enumerate and classify every QUANTIFIED site.
/// Pre-order per rule (`q#N`). Also returns the per-rule reference-occurrence
/// counts inside OUTERMOST guardable element subtrees — the [`QuantExposure`]
/// population-attribution basis (outermost-only, so a guardable site nested inside
/// another guardable site's element never double-counts its references).
fn enumerate_quant_sites(
    classifier: &mut Classifier,
    universe: &[String],
    grammar_wide_refs: &HashMap<String, usize>,
) -> (Vec<QuantSiteCensus>, HashMap<String, usize>) {
    let mut sites = Vec::new();
    let mut guardable_occurrences: HashMap<String, usize> = HashMap::new();
    for rule in universe {
        let Some(body) = classifier.tree.get(rule.as_str()) else {
            continue;
        };
        let mut q_counter = 0usize;
        walk_for_quant_sites(
            classifier,
            rule,
            body,
            false,
            &mut q_counter,
            grammar_wide_refs,
            &mut sites,
            &mut guardable_occurrences,
        );
    }
    (sites, guardable_occurrences)
}

#[allow(clippy::too_many_arguments)]
fn walk_for_quant_sites(
    classifier: &mut Classifier,
    rule: &str,
    node: &ASTNode,
    inside_counted_element: bool,
    q_counter: &mut usize,
    grammar_wide_refs: &HashMap<String, usize>,
    sites: &mut Vec<QuantSiteCensus>,
    guardable_occurrences: &mut HashMap<String, usize>,
) {
    match node {
        ASTNode::Quantified {
            element,
            quantifier,
        } => {
            let site_index = *q_counter;
            *q_counter += 1;
            let min_zero = parse_quantifier_bounds(quantifier)
                .map(|(min, _)| min == 0)
                .unwrap_or(false);
            let mut blockers: Vec<String> = Vec::new();
            // Gate 1 — lane scope: attempt elision replaces the ONE guaranteed
            // attempt of a min-0 site with its zero-iteration path; a min>0 site's
            // refuted first attempt fails the whole quantifier instead (a different
            // emission, out of this lane).
            if !min_zero {
                blockers.push(format!(
                    "min>0 quantifier '{quantifier}' (attempt elision targets min-0 sites)"
                ));
            }
            // Gate 2 — the P2 R2 mirror: the guard peeks `input[p]` directly;
            // unsound under an implicit leading terminal-layout skip.
            if !classifier.layout.terminals {
                blockers
                    .push("terminal layout skipping (R2 raw-byte peek unsound)".to_string());
            }
            // Gate 3 — the SHARED first-byte-decided predicate (non-nullable +
            // resolved + regex-token layout-trust + extractable bytes): the same
            // no-census-vs-emission-drift discipline as P2/D1. Nullable elision is
            // structurally meaningless anyway — a nullable element's zero-length
            // success is a COMMITTED attempt the skip would drop.
            let first_bytes = match classifier.branch_dispatch_first_bytes(element) {
                Ok(bytes) => Some(bytes),
                Err(reason) => {
                    blockers.push(format!("element not first-byte-decided: {reason}"));
                    None
                }
            };
            // Gate 4 — the `-0075` P2-(e) mirror: no rule REACHABLE from the
            // element subtree carries Branch-phase predicates or branch-start
            // effect directives (rolled back on failure anyway; excluded so
            // counters/diagnostics stay honest by exclusion).
            let mut element_ref_occurrences: HashMap<String, usize> = HashMap::new();
            collect_ref_occurrences(element, &mut element_ref_occurrences);
            let mut effectful: Vec<String> =
                reachable_rules(classifier.tree, element_ref_occurrences.keys().cloned())
                    .into_iter()
                    .filter(|reached| {
                        let branch_count = match classifier.tree.get(reached.as_str()) {
                            Some(ASTNode::Or { alternatives }) => alternatives.len(),
                            _ => 1,
                        };
                        let (predicates, effects) = classifier
                            .rule_branch_predicate_effect_facts(reached, branch_count);
                        predicates || effects
                    })
                    .collect();
            effectful.sort();
            for reached in &effectful {
                blockers.push(format!(
                    "reachable rule '{reached}' carries branch predicates/effects"
                ));
            }
            let guardable = blockers.is_empty();
            let mut element_refs: Vec<String> =
                element_ref_occurrences.keys().cloned().collect();
            element_refs.sort();
            let mut sole_refs: Vec<String> = element_ref_occurrences
                .iter()
                .filter(|(name, n)| grammar_wide_refs.get(name.as_str()).copied() == Some(**n))
                .map(|(name, _)| name.clone())
                .collect();
            sole_refs.sort();
            if guardable && !inside_counted_element {
                for (name, n) in &element_ref_occurrences {
                    *guardable_occurrences.entry(name.clone()).or_default() += n;
                }
            }
            sites.push(QuantSiteCensus {
                rule: rule.to_string(),
                site: format!("q#{site_index}"),
                quantifier: quantifier.clone(),
                min_zero,
                guardable,
                blockers,
                first_bytes,
                frontier: super::first_set::quantified_element_frontier(element)
                    .name()
                    .to_string(),
                element_refs,
                sole_refs,
                sole_attributable_discarded: 0,
            });
            walk_for_quant_sites(
                classifier,
                rule,
                element,
                inside_counted_element || guardable,
                q_counter,
                grammar_wide_refs,
                sites,
                guardable_occurrences,
            );
        }
        ASTNode::Sequence { elements } => {
            for element in elements {
                walk_for_quant_sites(
                    classifier,
                    rule,
                    element,
                    inside_counted_element,
                    q_counter,
                    grammar_wide_refs,
                    sites,
                    guardable_occurrences,
                );
            }
        }
        ASTNode::Or { alternatives } => {
            for branch in alternatives {
                walk_for_quant_sites(
                    classifier,
                    rule,
                    branch,
                    inside_counted_element,
                    q_counter,
                    grammar_wide_refs,
                    sites,
                    guardable_occurrences,
                );
            }
        }
        ASTNode::Lookahead { element, .. } => {
            walk_for_quant_sites(
                classifier,
                rule,
                element,
                inside_counted_element,
                q_counter,
                grammar_wide_refs,
                sites,
                guardable_occurrences,
            );
        }
        ASTNode::Atom { value } => {
            if let ASTValue::Node(inner) = value {
                walk_for_quant_sites(
                    classifier,
                    rule,
                    inner,
                    inside_counted_element,
                    q_counter,
                    grammar_wide_refs,
                    sites,
                    guardable_occurrences,
                );
            }
        }
    }
}

/// RGX-0078.5.h.1b — join outcome-count files (raw + committed) into the measured
/// discarded-work decomposition, and fill each choice site's sole-attributable
/// discarded total.
#[allow(clippy::too_many_arguments)]
fn join_outcome_counts(
    grammar_name: &str,
    rules: &BTreeMap<String, RuleCensus>,
    inline_rules: &BTreeMap<String, InlineRuleCensus>,
    choice_sites: &mut [ChoiceSiteCensus],
    quant_sites: &mut [QuantSiteCensus],
    q_guardable_occurrences: &HashMap<String, usize>,
    grammar_wide_refs: &HashMap<String, usize>,
    files: &[std::path::PathBuf],
) -> Result<(OutcomeShare, InlineExposure, QuantExposure), String> {
    let mut entries_sum: BTreeMap<String, u64> = BTreeMap::new();
    let mut committed_sum: BTreeMap<String, u64> = BTreeMap::new();
    let mut memo_hits_sum: BTreeMap<String, u64> = BTreeMap::new();
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
        for (rule, count) in parsed.rule_memo_hit_counts {
            *memo_hits_sum.entry(rule).or_default() += count;
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
        prefix2_site_entries: 0,
        prefix2_site_committed: 0,
        prefix2_site_discarded: 0,
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

    // RGX-0078.5.i.7 (D1 STEP-0) — the FIRST₂-dispatch exposure: entries on rules
    // whose top-level site is prefix2-dispatchable and NOT already degenerate.
    let mut prefix2_rules_counted: HashSet<&str> = HashSet::new();
    for site in choice_sites.iter() {
        if site.top_level
            && site.prefix2_dispatchable
            && !site.degenerate_dispatch
            && prefix2_rules_counted.insert(site.rule.as_str())
        {
            let entries = entries_sum.get(&site.rule).copied().unwrap_or(0);
            let committed = committed_sum.get(&site.rule).copied().unwrap_or(0);
            share.prefix2_site_entries += entries;
            share.prefix2_site_committed += committed.min(entries);
            share.prefix2_site_discarded += entries.saturating_sub(committed);
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

    // RGX-0078.5.i.4 (P1 STEP-0) — the inline-exposure join: real parse work on
    // inline-eligible rules, split raw/committed/discarded, plus the memo-hit
    // share (the P1b lost-hit surface). Each rule counted once (rule-keyed sums).
    let mut exposure = InlineExposure {
        eligible_entries: 0,
        eligible_committed: 0,
        eligible_discarded: 0,
        eligible_memo_hits: 0,
        total_memo_hits: memo_hits_sum.values().sum(),
        decided_entries: 0,
        decided_committed: 0,
        decided_discarded: 0,
        decided_memo_hits: 0,
        top_eligible_rules: Vec::new(),
    };
    let exposure_universe: std::collections::BTreeSet<&String> = entries_sum
        .keys()
        .chain(committed_sum.keys())
        .chain(memo_hits_sum.keys())
        .collect();
    for rule in exposure_universe {
        let Some(census) = inline_rules.get(rule.as_str()).filter(|c| c.eligible) else {
            continue;
        };
        let entries = entries_sum.get(rule).copied().unwrap_or(0);
        let committed = committed_sum.get(rule).copied().unwrap_or(0);
        let hits = memo_hits_sum.get(rule).copied().unwrap_or(0);
        exposure.eligible_entries += entries;
        exposure.eligible_committed += committed.min(entries);
        exposure.eligible_discarded += entries.saturating_sub(committed);
        exposure.eligible_memo_hits += hits;
        // RGX-0078.5.i.4 (P1b pricing) — the budget-DECIDED subset: the frames
        // the landed emission actually collapses, and the memo hits P1b loses.
        if census.decided {
            exposure.decided_entries += entries;
            exposure.decided_committed += committed.min(entries);
            exposure.decided_discarded += entries.saturating_sub(committed);
            exposure.decided_memo_hits += hits;
        }
        exposure
            .top_eligible_rules
            .push((rule.clone(), entries, committed, hits));
    }
    exposure
        .top_eligible_rules
        .sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

    // RGX-0078.5.i.7 (Q-GUARD STEP-0) — per-site sole attribution + the population
    // exposure lanes (attribution + honest-bound rules on [`QuantExposure`]).
    for site in quant_sites.iter_mut() {
        site.sole_attributable_discarded = site
            .sole_refs
            .iter()
            .map(|sole| discarded_by_rule.get(sole).copied().unwrap_or(0))
            .sum();
    }
    let mut q_exposure = QuantExposure {
        total_sites: quant_sites.len(),
        min_zero_sites: quant_sites.iter().filter(|s| s.min_zero).count(),
        guardable_sites: quant_sites.iter().filter(|s| s.guardable).count(),
        attributable_rules: Vec::new(),
        attributable_entries: 0,
        attributable_committed: 0,
        attributable_discarded: 0,
        shared_rules: Vec::new(),
        shared_entries: 0,
        shared_discarded: 0,
        top_attributable_rules: Vec::new(),
    };
    let mut q_touched: Vec<&String> = q_guardable_occurrences.keys().collect();
    q_touched.sort();
    for rule in q_touched {
        let occurrences = q_guardable_occurrences[rule];
        let entries = entries_sum.get(rule).copied().unwrap_or(0);
        let committed = committed_sum.get(rule).copied().unwrap_or(0);
        let discarded = entries.saturating_sub(committed);
        if grammar_wide_refs.get(rule.as_str()).copied() == Some(occurrences) {
            q_exposure.attributable_rules.push(rule.clone());
            q_exposure.attributable_entries += entries;
            q_exposure.attributable_committed += committed.min(entries);
            q_exposure.attributable_discarded += discarded;
            q_exposure.top_attributable_rules.push((
                rule.clone(),
                entries,
                committed.min(entries),
                discarded,
            ));
        } else {
            q_exposure.shared_rules.push(rule.clone());
            q_exposure.shared_entries += entries;
            q_exposure.shared_discarded += discarded;
        }
    }
    q_exposure
        .top_attributable_rules
        .sort_by(|a, b| b.3.cmp(&a.3).then(a.0.cmp(&b.0)));

    Ok((share, exposure, q_exposure))
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

    // RGX-0078.5.i.7 (Q-GUARD STEP-0) — the quantified-site census + the
    // population-attribution occurrence basis.
    let (mut quant_sites, q_guardable_occurrences) =
        enumerate_quant_sites(&mut classifier, &universe, &grammar_wide_refs);

    // RGX-0078.5.i.4 (P1 STEP-0) — the per-rule INLINE-eligibility census.
    let mut forward_refs: HashMap<String, HashSet<String>> = HashMap::new();
    let mut regex_pattern_sink: Vec<String> = Vec::new();
    for rule in &universe {
        let mut refs = HashSet::new();
        collect_refs(&grammar_tree[rule], &mut refs, &mut regex_pattern_sink);
        forward_refs.insert(rule.clone(), refs);
    }
    // RGX-0078.5.i.4 (P1a) — the SHARED emission decisions (gates + budget), the
    // same map codegen consumes for the inline emission.
    let inline_decisions = compute_inline_decisions(
        grammar_tree,
        annotations,
        classifier.compiled.as_ref(),
        entry_rule.as_deref(),
    );
    let inline_rules: BTreeMap<String, InlineRuleCensus> = universe
        .iter()
        .map(|rule| {
            let on_cycle = rule_reaches_itself(rule, &forward_refs);
            let is_entry = Some(rule) == entry_rule.as_ref();
            let (eligible, blockers) = classifier.inline_rule_verdict(rule, on_cycle, is_entry);
            let body = &grammar_tree[rule];
            (
                rule.clone(),
                InlineRuleCensus {
                    eligible,
                    blockers,
                    wrapper_class: eligible.then(|| inline_wrapper_class(body)),
                    reference_sites: grammar_wide_refs.get(rule).copied().unwrap_or(0),
                    body_nodes: count_nodes(body),
                    decided: inline_decisions
                        .get(rule)
                        .is_some_and(|decision| decision.decided),
                },
            )
        })
        .collect();

    // RGX-0078.5.i.7 (D2 STEP-0) — the CASCADE-FOLD census: per-rule eligibility
    // (effect-freedom + policy-encodability; cycles/layout/lookahead/value shapes
    // are named facts, not blockers), the root/internal partition (the EntryShare
    // maximal-root logic under the D2 gate), and the region boundary edges.
    let cascade_verdicts: BTreeMap<String, (bool, Vec<String>)> = universe
        .iter()
        .map(|rule| (rule.clone(), classifier.cascade_rule_verdict(rule)))
        .collect();
    let cascade_rules: BTreeMap<String, CascadeRuleCensus> = universe
        .iter()
        .map(|rule| {
            let (eligible, reasons) = cascade_verdicts[rule].clone();
            let root = eligible
                && (Some(rule) == entry_rule.as_ref()
                    || referenced_by
                        .get(rule)
                        .map(|parents| parents.iter().any(|p| !cascade_verdicts[p].0))
                        .unwrap_or(true));
            let boundary_refs: Vec<String> = if eligible {
                let mut v: Vec<String> = forward_refs[rule]
                    .iter()
                    .filter(|target| {
                        grammar_tree.contains_key(target.as_str())
                            && !cascade_verdicts
                                .get(target.as_str())
                                .map(|(ok, _)| *ok)
                                .unwrap_or(false)
                    })
                    .cloned()
                    .collect();
                v.sort();
                v
            } else {
                Vec::new()
            };
            (
                rule.clone(),
                CascadeRuleCensus {
                    eligible,
                    reasons,
                    root,
                    on_cycle: rule_reaches_itself(rule, &forward_refs),
                    boundary_refs,
                },
            )
        })
        .collect();

    let (outcome_share, inline_exposure, quant_exposure) = if outcome_counts_files.is_empty() {
        (None, None, None)
    } else {
        let (share, exposure, q_exposure) = join_outcome_counts(
            grammar_name,
            &rules,
            &inline_rules,
            &mut choice_sites,
            &mut quant_sites,
            &q_guardable_occurrences,
            &grammar_wide_refs,
            outcome_counts_files,
        )?;
        (Some(share), Some(exposure), Some(q_exposure))
    };
    let cascade_exposure = if outcome_counts_files.is_empty() {
        None
    } else {
        Some(join_cascade_outcome_counts(
            grammar_name,
            &cascade_rules,
            outcome_counts_files,
        )?)
    };
    // RGX-0078.5.i.7 (D2-A) — the SAME plan function codegen consumes.
    let cascade_plan =
        compute_cascade_emission_plan(grammar_tree, annotations, entry_rule.as_deref())?;

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
        inline_rules,
        inline_exposure,
        quant_sites,
        quant_exposure,
        cascade_rules,
        cascade_exposure,
        cascade_plan,
    })
}

/// RGX-0078.5.i.7 (D2 STEP-0) — join the cascade census with per-parse outcome
/// counts. Per-rule discarded = `raw.saturating_sub(committed)` (mirroring
/// `join_outcome_counts`' handling of the documented ±1 committed-overshoot class).
fn join_cascade_outcome_counts(
    grammar_name: &str,
    cascade_rules: &BTreeMap<String, CascadeRuleCensus>,
    files: &[std::path::PathBuf],
) -> Result<CascadeExposure, String> {
    let mut exposure = CascadeExposure {
        count_files: 0,
        total_entries: 0,
        internal_entries: 0,
        internal_committed: 0,
        internal_discarded: 0,
        internal_memo_hits: 0,
        root_entries: 0,
        root_committed: 0,
        root_discarded: 0,
        root_memo_hits: 0,
        residual_entries: 0,
        residual_committed: 0,
        residual_discarded: 0,
        unmatched_rules: Vec::new(),
        unmatched_entries: 0,
        committed_floor: 0,
        top_internal_rules: Vec::new(),
    };
    let mut unmatched: BTreeMap<String, u64> = BTreeMap::new();
    let mut internal_by_rule: BTreeMap<String, (u64, u64, u64)> = BTreeMap::new();

    for path in files {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read outcome-counts file {}: {e}", path.display()))?;
        let parsed: RuleOutcomeCountsFile = serde_json::from_str(&text)
            .map_err(|e| format!("cannot parse outcome-counts file {}: {e}", path.display()))?;
        if parsed.grammar != grammar_name {
            return Err(format!(
                "outcome-counts file {} is for grammar '{}', census is for '{}'",
                path.display(),
                parsed.grammar,
                grammar_name
            ));
        }
        exposure.count_files += 1;
        for (rule, raw) in &parsed.rule_entry_counts {
            let committed = parsed.rule_committed_counts.get(rule).copied().unwrap_or(0);
            let hits = parsed.rule_memo_hit_counts.get(rule).copied().unwrap_or(0);
            let discarded = raw.saturating_sub(committed);
            exposure.total_entries += raw;
            match cascade_rules.get(rule) {
                Some(census) if census.eligible && !census.root => {
                    exposure.internal_entries += raw;
                    exposure.internal_committed += committed;
                    exposure.internal_discarded += discarded;
                    exposure.internal_memo_hits += hits;
                    let slot = internal_by_rule.entry(rule.clone()).or_insert((0, 0, 0));
                    slot.0 += raw;
                    slot.1 += committed;
                    slot.2 += discarded;
                }
                Some(census) if census.eligible => {
                    exposure.root_entries += raw;
                    exposure.root_committed += committed;
                    exposure.root_discarded += discarded;
                    exposure.root_memo_hits += hits;
                }
                Some(_) => {
                    exposure.residual_entries += raw;
                    exposure.residual_committed += committed;
                    exposure.residual_discarded += discarded;
                }
                None => {
                    *unmatched.entry(rule.clone()).or_default() += raw;
                }
            }
        }
    }
    exposure.unmatched_entries = unmatched.values().sum();
    exposure.unmatched_rules = unmatched.into_keys().collect();
    exposure.committed_floor = exposure.root_committed + exposure.residual_committed;
    let mut top: Vec<(String, u64, u64, u64)> = internal_by_rule
        .into_iter()
        .map(|(rule, (raw, committed, discarded))| (rule, raw, committed, discarded))
        .collect();
    top.sort_by(|a, b| b.2.cmp(&a.2).then(a.0.cmp(&b.0)));
    top.truncate(12);
    exposure.top_internal_rules = top;
    Ok(exposure)
}

/// RGX-0078.5.i.7 (D2-A) — the SHARED cascade-emission plan for the ACYCLIC
/// SUB-REGION increment (the D2 EMISSION DESIGN's D2-A slice): CYCLIC eligible
/// rules are treated as protocol boundaries, so every fused function in this
/// increment is non-recursive and needs no memo lane (re-probe multiplicity is
/// bounded by the grammar's static caller constant). Codegen consumes this plan
/// for the D2-A emission; the census reports it (`CASCADE-PLAN`), so the two
/// cannot drift — the P1a `compute_inline_decisions` precedent.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CascadeEmissionPlan {
    /// Acyclic cascade-eligible rules entered from OUTSIDE the increment's fused
    /// graph (the entry rule, a rule referenced by an ineligible or cyclic rule,
    /// or an unreferenced rule): their methods keep the FULL protocol frame and
    /// gain the observability-twin dispatch to `cascade_<rule>` inside the
    /// memoized body.
    pub sub_roots: std::collections::BTreeSet<String>,
    /// Acyclic cascade-eligible rules referenced ONLY by other acyclic
    /// cascade-eligible rules: on the bare-parse path they are reached through
    /// fused `cascade_<rule>` functions exclusively (their methods remain for
    /// the protocol graph / entry-relative parses).
    pub internal: std::collections::BTreeSet<String>,
    /// Rules (ALL tree rules, not only fused ones) from whose body an INELIGIBLE
    /// tree-defined rule is reachable through tree references — the conservative
    /// "can this call change semantic state / consult a gate?" set. The emitter
    /// uses it for the two ⛔ C3-B/store rules: a fused speculation scope whose
    /// calls can reach this set carries the semantic-checkpoint snapshot, and an
    /// Or site with an effect-reaching branch keeps the protocol tournament as a
    /// site island.
    pub effect_reaching: std::collections::BTreeSet<String>,
}

/// Compute the D2-A cascade-emission plan. Verdicts come from the census's OWN
/// `cascade_rule_verdict` (one implementation of the D2 gate for report and
/// emission); cyclicity from `rule_reaches_itself` over the census's own
/// reference collector. Deterministic (`BTreeSet` output, monotone fixpoint).
pub fn compute_cascade_emission_plan(
    tree: &HashMap<String, ASTNode>,
    annotations: Option<&Annotations>,
    entry_rule: Option<&str>,
) -> Result<CascadeEmissionPlan, String> {
    let classifier = Classifier::new(tree, annotations)?;
    let mut forward: HashMap<String, HashSet<String>> = HashMap::new();
    let mut regex_pattern_sink: Vec<String> = Vec::new();
    for (rule, body) in tree {
        let mut refs = HashSet::new();
        collect_refs(body, &mut refs, &mut regex_pattern_sink);
        forward.insert(rule.clone(), refs);
    }
    let mut referenced_by: HashMap<&str, Vec<&str>> = HashMap::new();
    for (rule, refs) in &forward {
        for target in refs {
            if tree.contains_key(target.as_str()) {
                referenced_by.entry(target).or_default().push(rule);
            }
        }
    }
    let eligible: HashMap<&str, bool> = tree
        .keys()
        .map(|rule| (rule.as_str(), classifier.cascade_rule_verdict(rule).0))
        .collect();
    let fused_candidate: HashMap<&str, bool> = tree
        .keys()
        .map(|rule| {
            (
                rule.as_str(),
                eligible[rule.as_str()] && !rule_reaches_itself(rule, &forward),
            )
        })
        .collect();

    let mut sub_roots = std::collections::BTreeSet::new();
    let mut internal = std::collections::BTreeSet::new();
    for rule in tree.keys() {
        if !fused_candidate[rule.as_str()] {
            continue;
        }
        let outside_entered = Some(rule.as_str()) == entry_rule
            || referenced_by
                .get(rule.as_str())
                .map(|callers| callers.iter().any(|c| !fused_candidate[c]))
                .unwrap_or(true);
        if outside_entered {
            sub_roots.insert(rule.clone());
        } else {
            internal.insert(rule.clone());
        }
    }

    // Monotone fixpoint: a rule reaches an effect boundary iff any tree-defined
    // reference target is ineligible OR itself effect-reaching (propagates
    // through eligible non-fused rules too — a method call to an effect-free
    // cyclic rule can still reach a fact-writing descendant).
    let mut effect_reaching: HashSet<&str> = HashSet::new();
    loop {
        let mut changed = false;
        for (rule, refs) in &forward {
            if effect_reaching.contains(rule.as_str()) {
                continue;
            }
            let reaches = refs.iter().any(|t| {
                tree.contains_key(t.as_str())
                    && (!eligible[t.as_str()] || effect_reaching.contains(t.as_str()))
            });
            if reaches {
                effect_reaching.insert(rule.as_str());
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    Ok(CascadeEmissionPlan {
        sub_roots,
        internal,
        effect_reaching: effect_reaching.into_iter().map(String::from).collect(),
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
    // RGX-0078.5.i.7 (D1 STEP-0) — the FIRST₂ two-level-dispatch surface.
    let blocked_sites: Vec<&ChoiceSiteCensus> = top_level_sites
        .iter()
        .copied()
        .filter(|s| !s.degenerate_dispatch)
        .collect();
    let prefix2_sites: Vec<&ChoiceSiteCensus> = blocked_sites
        .iter()
        .copied()
        .filter(|s| s.prefix2_dispatchable)
        .collect();
    let wildcard_limited = prefix2_sites
        .iter()
        .filter(|s| !s.prefix2_wildcard_branches.is_empty())
        .count();
    println!(
        "PREFIX2-CENSUS: grammar={} blocked_top_level_sites={} prefix2_dispatchable={} (wildcard_limited={})",
        census.grammar_name,
        blocked_sites.len(),
        prefix2_sites.len(),
        wildcard_limited,
    );
    println!(
        "  gate: the P2 gates minus first-byte disjointness + per shared first byte: resolved second-byte facts + non-wildcard members pairwise-disjoint on byte 2 — RGX-0078.5.i.7 (D1)"
    );
    if !prefix2_sites.is_empty() {
        let names: Vec<String> = prefix2_sites
            .iter()
            .map(|s| {
                if s.prefix2_wildcard_branches.is_empty() {
                    format!("{}({})", s.rule, s.branches)
                } else {
                    format!("{}({},w{})", s.rule, s.branches, s.prefix2_wildcard_branches.len())
                }
            })
            .collect();
        println!("  prefix2 sites (rule(branches[,wildcards])): {}", names.join(" "));
    }
    let mut prefix2_blocker_histogram: HashMap<String, usize> = HashMap::new();
    for site in &blocked_sites {
        for blocker in &site.prefix2_blockers {
            let key = if blocker.contains("SECOND byte") {
                "second-byte overlap between branches"
            } else if blocker.contains("second bytes UNRESOLVED") {
                "branch second bytes unresolved"
            } else if blocker.starts_with("branch ") {
                "branch not first-byte-decided"
            } else {
                blocker
                    .split_once(':')
                    .map(|(head, _)| head)
                    .unwrap_or(blocker.as_str())
            };
            *prefix2_blocker_histogram.entry(key.to_string()).or_default() += 1;
        }
    }
    if !prefix2_blocker_histogram.is_empty() {
        let mut ranked: Vec<(String, usize)> = prefix2_blocker_histogram.into_iter().collect();
        ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        println!("  prefix2 blocker histogram (blocked-site occurrences):");
        for (reason, count) in &ranked {
            println!("    {count:>5}  {reason}");
        }
    }
    // RGX-0078.5.i.7 (Q-GUARD STEP-0) — the quantified-site attempt-elision surface.
    let min_zero_sites = census.quant_sites.iter().filter(|s| s.min_zero).count();
    let guardable_sites: Vec<&QuantSiteCensus> = census
        .quant_sites
        .iter()
        .filter(|s| s.guardable)
        .collect();
    let frontier_count = |class: &str| {
        guardable_sites
            .iter()
            .filter(|s| s.frontier == class)
            .count()
    };
    println!(
        "QUANT-SITE-CENSUS: grammar={} quantified_sites={} min_zero={} guardable={} (bare_ref={} no_refs={} mixed={})",
        census.grammar_name,
        census.quant_sites.len(),
        min_zero_sites,
        guardable_sites.len(),
        frontier_count("bare_ref"),
        frontier_count("no_refs"),
        frontier_count("mixed"),
    );
    println!(
        "  gate: min-0 quantifier + terminal-ws-sensitive + element first-byte-decided + predicate/effect-free reachable closure — RGX-0078.5.i.7 (Q-GUARD); the EMISSION guards bare_ref (+ exact furthest emulation) and no_refs sites only"
    );
    if !guardable_sites.is_empty() {
        let names: Vec<String> = guardable_sites
            .iter()
            .map(|s| format!("{}@{}({})", s.rule, s.site, s.quantifier))
            .collect();
        println!("  guardable sites (rule@site(quantifier)): {}", names.join(" "));
    }
    let mut quant_blocker_histogram: HashMap<String, usize> = HashMap::new();
    for site in census.quant_sites.iter().filter(|s| s.min_zero && !s.guardable) {
        for blocker in &site.blockers {
            // Fold per-site detail out of the key (the choice-census convention).
            let key = if blocker.starts_with("element not first-byte-decided") {
                "element not first-byte-decided"
            } else if blocker.starts_with("reachable rule") {
                "reachable rule carries branch predicates/effects"
            } else {
                blocker
                    .split_once(':')
                    .map(|(head, _)| head)
                    .unwrap_or(blocker.as_str())
            };
            *quant_blocker_histogram.entry(key.to_string()).or_default() += 1;
        }
    }
    if !quant_blocker_histogram.is_empty() {
        let mut ranked: Vec<(String, usize)> = quant_blocker_histogram.into_iter().collect();
        ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        println!("  quant blocker histogram (blocked min-0 site occurrences):");
        for (reason, count) in &ranked {
            println!("    {count:>5}  {reason}");
        }
    }
    if let Some(exposure) = &census.quant_exposure {
        println!(
            "QUANT-EXPOSURE: grammar={} attributable_rules={} entries={} committed={} discarded={} | shared_rules={} shared_entries={} shared_discarded={}",
            census.grammar_name,
            exposure.attributable_rules.len(),
            exposure.attributable_entries,
            exposure.attributable_committed,
            exposure.attributable_discarded,
            exposure.shared_rules.len(),
            exposure.shared_entries,
            exposure.shared_discarded,
        );
        println!(
            "  attribution: a rule counts only when EVERY grammar-wide occurrence sits under a guardable min-0 site's element; discarded is an UPPER bound on the guard's kill (byte-1-admitted refutations survive); terminal-only guardable sites are invisible to per-rule counters"
        );
        if !exposure.top_attributable_rules.is_empty() {
            println!("  top attributable rules (rule: entries/committed/discarded):");
            for (rule, entries, committed, discarded) in
                exposure.top_attributable_rules.iter().take(16)
            {
                println!("    {rule}: {entries}/{committed}/{discarded}");
            }
        }
        if !exposure.shared_rules.is_empty() {
            println!("  shared-exposure rules: {}", exposure.shared_rules.join(", "));
        }
    }
    // RGX-0078.5.i.7 (D2 STEP-0) — the CASCADE-FOLD census + exposure.
    {
        let eligible_rules: Vec<(&String, &CascadeRuleCensus)> = census
            .cascade_rules
            .iter()
            .filter(|(_, c)| c.eligible)
            .collect();
        let roots = eligible_rules.iter().filter(|(_, c)| c.root).count();
        let internal = eligible_rules.len() - roots;
        let internal_cyclic = eligible_rules
            .iter()
            .filter(|(_, c)| !c.root && c.on_cycle)
            .count();
        let boundary_edges: usize = eligible_rules
            .iter()
            .map(|(_, c)| c.boundary_refs.len())
            .sum();
        println!(
            "CASCADE-CENSUS: grammar={} rules={} cascade_eligible={} (roots={} internal={} internal_cyclic={}) residual={} boundary_edges={}",
            census.grammar_name,
            census.cascade_rules.len(),
            eligible_rules.len(),
            roots,
            internal,
            internal_cyclic,
            census.cascade_rules.len() - eligible_rules.len(),
            boundary_edges,
        );
        println!(
            "  gate: effect-free + policy-encodable ONLY — cycles/layout/lookahead/value shapes are emittable by a fused matcher (named facts, not blockers) — RGX-0078.5.i.7 (D2)"
        );
        let mut cascade_blocker_histogram: HashMap<String, usize> = HashMap::new();
        for census_entry in census.cascade_rules.values() {
            for reason in &census_entry.reasons {
                *cascade_blocker_histogram.entry(reason.clone()).or_default() += 1;
            }
        }
        if !cascade_blocker_histogram.is_empty() {
            let mut ranked: Vec<(String, usize)> = cascade_blocker_histogram.into_iter().collect();
            ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
            println!("  cascade blocker histogram (rule occurrences):");
            for (reason, count) in &ranked {
                println!("    {count:>5}  {reason}");
            }
        }
        if dump_all {
            for (rule, c) in &census.cascade_rules {
                if c.eligible {
                    println!(
                        "  [cascade] {rule}: {} on_cycle={} boundary_refs=[{}]",
                        if c.root { "ROOT" } else { "internal" },
                        c.on_cycle,
                        c.boundary_refs.join(", "),
                    );
                } else {
                    println!("  [cascade] {rule}: BLOCKED — {}", c.reasons.join("; "));
                }
            }
        }
    }
    // RGX-0078.5.i.7 (D2-A) — the shared acyclic-sub-region emission plan
    // (compute_cascade_emission_plan, the same map codegen consumes).
    {
        let plan = &census.cascade_plan;
        let fused_effect_reaching = plan
            .sub_roots
            .iter()
            .chain(plan.internal.iter())
            .filter(|r| plan.effect_reaching.contains(*r))
            .count();
        println!(
            "CASCADE-PLAN: grammar={} increment=A(acyclic-subregions) sub_roots={} internal={} effect_reaching_fused={} (of {} fused)",
            census.grammar_name,
            plan.sub_roots.len(),
            plan.internal.len(),
            fused_effect_reaching,
            plan.sub_roots.len() + plan.internal.len(),
        );
        println!(
            "  model: cyclic eligible rules stay protocol boundaries in this increment (no memo lane, no recursion); sub-roots keep the full protocol frame + the observability-twin dispatch; internal rules run as fused cascade_* functions on the bare-parse path; effect-reaching fused rules carry the C3-B snapshot/island obligations."
        );
        if dump_all {
            for rule in &plan.sub_roots {
                println!(
                    "  [cascade-plan] {rule}: SUB-ROOT{}",
                    if plan.effect_reaching.contains(rule) {
                        " effect_reaching"
                    } else {
                        ""
                    }
                );
            }
            for rule in &plan.internal {
                println!(
                    "  [cascade-plan] {rule}: internal{}",
                    if plan.effect_reaching.contains(rule) {
                        " effect_reaching"
                    } else {
                        ""
                    }
                );
            }
        }
    }
    if let Some(exposure) = &census.cascade_exposure {
        let pct = |part: u64| {
            if exposure.total_entries == 0 {
                0.0
            } else {
                part as f64 * 100.0 / exposure.total_entries as f64
            }
        };
        println!(
            "CASCADE-EXPOSURE: grammar={} files={} total_entries={} | internal={} ({:.1}%) committed={} discarded={} memo_hits={} | roots={} committed={} | residual={} committed={} | committed_floor={} unmatched={}",
            census.grammar_name,
            exposure.count_files,
            exposure.total_entries,
            exposure.internal_entries,
            pct(exposure.internal_entries),
            exposure.internal_committed,
            exposure.internal_discarded,
            exposure.internal_memo_hits,
            exposure.root_entries,
            exposure.root_committed,
            exposure.residual_entries,
            exposure.residual_committed,
            exposure.committed_floor,
            exposure.unmatched_entries,
        );
        println!(
            "  first-order model: internal entries (and their per-entry protocol) are eliminated by a fold; root entries become one specialized-function call each; committed_floor = the post-fold protocol-paying committed entries. Memo hits on internal rules re-execute (the emission design owns the re-probe boundedness proof)."
        );
        if !exposure.top_internal_rules.is_empty() {
            println!("  top internal rules by committed (rule: entries/committed/discarded):");
            for (rule, entries, committed, discarded) in exposure.top_internal_rules.iter() {
                println!("    {rule}: {entries}/{committed}/{discarded}");
            }
        }
        if !exposure.unmatched_rules.is_empty() {
            println!("  unmatched rules: {}", exposure.unmatched_rules.join(", "));
        }
    }
    // RGX-0078.5.i.4 (P1 STEP-0) — the inline-eligibility census + what blocks it.
    let eligible: Vec<(&String, &InlineRuleCensus)> = census
        .inline_rules
        .iter()
        .filter(|(_, c)| c.eligible)
        .collect();
    let class_count = |class: InlineWrapperClass| {
        eligible
            .iter()
            .filter(|(_, c)| c.wrapper_class == Some(class))
            .count()
    };
    println!(
        "INLINE-CENSUS: grammar={} rules={} inline_eligible={} (pass_through={} alternation_leaf={} shaped={})",
        census.grammar_name,
        census.inline_rules.len(),
        eligible.len(),
        class_count(InlineWrapperClass::PassThrough),
        class_count(InlineWrapperClass::AlternationLeaf),
        class_count(InlineWrapperClass::Shaped),
    );
    println!(
        "  gate: acyclic (guard non-load-bearing) + directive-free frame + non-entry + no @profiles/@transform — RGX-0078.5.i.4 (P1)"
    );
    let mut inline_blocker_histogram: HashMap<String, usize> = HashMap::new();
    for census_entry in census.inline_rules.values() {
        for blocker in &census_entry.blockers {
            *inline_blocker_histogram.entry(blocker.clone()).or_default() += 1;
        }
    }
    if !inline_blocker_histogram.is_empty() {
        let mut ranked: Vec<(String, usize)> = inline_blocker_histogram.into_iter().collect();
        ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        println!("  inline blocker histogram (rule occurrences):");
        for (reason, count) in &ranked {
            println!("    {count:>5}  {reason}");
        }
    }
    // RGX-0078.5.i.4 (P1a) — the EMISSION decisions under the shared budget
    // (`compute_inline_decisions`, the same map codegen consumes). No silent
    // caps: the over-budget count is always printed.
    let decided_count = eligible.iter().filter(|(_, c)| c.decided).count();
    println!(
        "INLINE-DECISIONS: grammar={} decided_under_budget={} over_budget={} (expansion_cap={} duplication_cap={})",
        census.grammar_name,
        decided_count,
        eligible.len() - decided_count,
        INLINE_EXPANSION_CAP,
        INLINE_DUPLICATION_CAP,
    );
    if dump_all && !eligible.is_empty() {
        println!("  inline-eligible rules (PGEN_FUSIBILITY_DUMP_ALL; class, reference sites, body nodes, emission decision):");
        for (rule, c) in &eligible {
            let class = match c.wrapper_class {
                Some(InlineWrapperClass::PassThrough) => "pass_through",
                Some(InlineWrapperClass::AlternationLeaf) => "alternation_leaf",
                Some(InlineWrapperClass::Shaped) => "shaped",
                None => unreachable!("eligible rules carry a wrapper class by construction"),
            };
            println!(
                "    {rule}: {class} refs={} body_nodes={} {}",
                c.reference_sites,
                c.body_nodes,
                if c.decided { "INLINED" } else { "over-budget" },
            );
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
        // RGX-0078.5.i.7 (D1 STEP-0) — the FIRST₂-dispatch exposure.
        let prefix2_share_pct = if share.total_entries == 0 {
            0.0
        } else {
            100.0 * share.prefix2_site_entries as f64 / share.total_entries as f64
        };
        println!(
            "PREFIX2-EXPOSURE: grammar={} prefix2_site_entries={} ({:.1}% of total) committed={} discarded={}",
            census.grammar_name,
            share.prefix2_site_entries,
            prefix2_share_pct,
            share.prefix2_site_committed,
            share.prefix2_site_discarded,
        );
        println!(
            "  model: entries on rules whose top-level site is prefix2-dispatchable (and not already P2-degenerate) — a two-level byte dispatch admits at most one non-wildcard candidate, so byte-1-admitted/byte-2-refuted branch attempts are never entered"
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
        // RGX-0078.5.i.4 (P1 STEP-0) — the measured inline exposure.
        if let Some(exposure) = &census.inline_exposure {
            let entry_pct = if share.total_entries == 0 {
                0.0
            } else {
                100.0 * exposure.eligible_entries as f64 / share.total_entries as f64
            };
            println!(
                "INLINE-EXPOSURE: grammar={} eligible_entries={} ({:.1}% of total) committed={} discarded={} memo_hits_on_eligible={} (total_memo_hits={})",
                census.grammar_name,
                exposure.eligible_entries,
                entry_pct,
                exposure.eligible_committed,
                exposure.eligible_discarded,
                exposure.eligible_memo_hits,
                exposure.total_memo_hits,
            );
            println!(
                "  model: each eligible-rule entry is a collapsible wrapper frame — P1a elides guard/context/annotation-probe/call protocol with the memo preserved (counters byte-identical); P1b additionally elides the memo probes/inserts, so each memo_hits_on_eligible replay becomes a body re-execution (counters change truthfully). Hits need dumps from a `.5.i.4`+ parser generation — total_memo_hits=0 on older dumps."
            );
            // RGX-0078.5.i.4 (P1b pricing) — the budget-DECIDED subset of the
            // exposure: what the landed P1a emission actually collapses, and
            // the memo-hit population P1b's elision re-executes.
            let decided_pct = if share.total_entries == 0 {
                0.0
            } else {
                100.0 * exposure.decided_entries as f64 / share.total_entries as f64
            };
            println!(
                "INLINE-EXPOSURE-DECIDED: grammar={} decided_entries={} ({:.1}% of total) committed={} discarded={} memo_hits_on_decided={}",
                census.grammar_name,
                exposure.decided_entries,
                decided_pct,
                exposure.decided_committed,
                exposure.decided_discarded,
                exposure.decided_memo_hits,
            );
            println!(
                "  model: the same sums restricted to rules the shared budget DECIDES for inlining (compute_inline_decisions — the emission plan): decided_entries = frames P1a collapses at call sites; memo_hits_on_decided = the P1b lost-hit surface at the budget (each becomes a body re-execution)."
            );
            if !exposure.top_eligible_rules.is_empty() {
                println!("  top inline-eligible rules by raw entries (entries/committed/memo_hits):");
                for (rule, entries, committed, hits) in
                    exposure.top_eligible_rules.iter().take(15)
                {
                    println!("    {entries:>6} {committed:>6} {hits:>6}  {rule}");
                }
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
                } else if site.top_level && site.prefix2_dispatchable {
                    format!(
                        " [PREFIX2{}]",
                        if site.prefix2_wildcard_branches.is_empty() {
                            String::new()
                        } else {
                            format!(
                                " wildcards={}",
                                site.prefix2_wildcard_branches
                                    .iter()
                                    .map(|n| n.to_string())
                                    .collect::<Vec<_>>()
                                    .join(",")
                            )
                        }
                    )
                } else if site.top_level {
                    format!(
                        " blocked: {} | prefix2: {}",
                        site.degeneracy_blockers
                            .first()
                            .map(String::as_str)
                            .unwrap_or("<none>"),
                        site.prefix2_blockers
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

    /// RGX-0078.5.i.7 (D1 STEP-0) — the FIRST₂ verdict: a same-first-byte site with
    /// DISJOINT second bytes qualifies; a shared second byte or an unresolved second
    /// summary blocks with a named reason; a len1 member is a counted WILDCARD, not
    /// a blocker.
    #[test]
    fn site_prefix2_verdict_discriminates_on_second_bytes() {
        use crate::ast_pipeline::first_set::SecondByteSummary;
        let second = |bytes: &[u8]| SecondByteSummary {
            second_bytes: bytes.iter().copied().collect(),
            ..SecondByteSummary::default()
        };
        // The zero_width shape: both branches admit '\'; second bytes 'b' vs 'B'.
        let firsts = vec![Ok(vec![b'\\']), Ok(vec![b'\\'])];
        let seconds = vec![second(&[b'b']), second(&[b'B'])];
        let (ok, blockers, wildcards) =
            site_prefix2_verdict(true, true, false, false, &firsts, &seconds);
        assert!(ok, "blockers: {blockers:?}");
        assert!(wildcards.is_empty());

        // A shared SECOND byte blocks with a named reason.
        let seconds = vec![second(&[b'b']), second(&[b'b', b'B'])];
        let (ok, blockers, _) =
            site_prefix2_verdict(true, true, false, false, &firsts, &seconds);
        assert!(!ok);
        assert!(
            blockers.iter().any(|b| b.contains("SECOND byte 0x62")),
            "blockers: {blockers:?}"
        );

        // An unresolved second summary blocks with a named reason.
        let seconds = vec![
            second(&[b'b']),
            SecondByteSummary {
                unresolved: true,
                ..SecondByteSummary::default()
            },
        ];
        let (ok, blockers, _) =
            site_prefix2_verdict(true, true, false, false, &firsts, &seconds);
        assert!(!ok);
        assert!(
            blockers.iter().any(|b| b.contains("second bytes UNRESOLVED")),
            "blockers: {blockers:?}"
        );

        // A len1 member is a byte-2 WILDCARD: legal, counted, never a blocker.
        let seconds = vec![
            second(&[b'b']),
            SecondByteSummary {
                len1_possible: true,
                ..SecondByteSummary::default()
            },
        ];
        let (ok, blockers, wildcards) =
            site_prefix2_verdict(true, true, false, false, &firsts, &seconds);
        assert!(ok, "blockers: {blockers:?}");
        assert_eq!(wildcards, vec![2]);

        // Disjoint FIRST bytes need no second level: trivially dispatchable, and
        // the wildcard set stays empty (no shared byte-1 subset exists).
        let firsts = vec![Ok(vec![b'a']), Ok(vec![b'b'])];
        let seconds = vec![second(&[b'x']), second(&[b'x'])];
        let (ok, blockers, wildcards) =
            site_prefix2_verdict(true, true, false, false, &firsts, &seconds);
        assert!(ok, "blockers: {blockers:?}");
        assert!(wildcards.is_empty());

        // D1 emission license — an `offset1_rule_entry` member (byte-2-refuted
        // attempts may enter a rule at offset ≥1 ⇒ un-prunable for
        // furthest-position parity) is classified a WILDCARD, exactly like len1:
        // it joins every second-byte arm, capping the kill but never blocking.
        let firsts = vec![Ok(vec![b'\\']), Ok(vec![b'\\'])];
        let seconds = vec![
            second(&[b'b']),
            SecondByteSummary {
                offset1_rule_entry: true,
                ..second(&[b'B'])
            },
        ];
        let (ok, blockers, wildcards) =
            site_prefix2_verdict(true, true, false, false, &firsts, &seconds);
        assert!(ok, "blockers: {blockers:?}");
        assert_eq!(wildcards, vec![2]);
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

    fn whitespace_sensitive_annotations(rule: &str) -> Annotations {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            rule.to_string(),
            vec![super::super::SemanticAnnotation::Named {
                name: "whitespace_sensitive".to_string(),
                ast: super::super::UnifiedSemanticAST::Structured {
                    canonical: String::new(),
                    value: super::super::UnifiedSemanticValue::Boolean(true),
                },
            }],
        );
        annotations
    }

    /// RGX-0078.5.i.7 (Q-GUARD STEP-0) — a min-0 quantified site with a
    /// first-byte-decided element under terminal-sensitive layout is GUARDABLE
    /// with the element's byte set; a nullable element and a min>0 quantifier are
    /// blocked with NAMED reasons (the min>0 site still reports its byte set for
    /// steering). Sites are censused at ANY nesting (no rule-top-level gate).
    #[test]
    fn quant_census_classifies_min0_sites_and_names_blockers() {
        let mut tree = HashMap::new();
        // top := 'a'* nullable_opt? 't'+   (three quantified sites)
        tree.insert(
            "top".to_string(),
            ASTNode::Sequence {
                elements: vec![
                    ASTNode::Quantified {
                        element: Box::new(atom("quoted_string", "a")),
                        quantifier: "*".to_string(),
                    },
                    ASTNode::Quantified {
                        element: Box::new(rule_ref("nullable_opt")),
                        quantifier: "?".to_string(),
                    },
                    ASTNode::Quantified {
                        element: Box::new(atom("quoted_string", "t")),
                        quantifier: "+".to_string(),
                    },
                ],
            },
        );
        // nullable_opt := 'x'?   — nullable ⇒ not first-byte-decided; its own
        // inner site ('x'?) is guardable in its own right.
        tree.insert(
            "nullable_opt".to_string(),
            ASTNode::Quantified {
                element: Box::new(atom("quoted_string", "x")),
                quantifier: "?".to_string(),
            },
        );
        let census = census_of(
            tree,
            vec!["top".to_string(), "nullable_opt".to_string()],
            Some(whitespace_sensitive_annotations("top")),
        );
        let top_sites: Vec<&QuantSiteCensus> = census
            .quant_sites
            .iter()
            .filter(|s| s.rule == "top")
            .collect();
        assert_eq!(top_sites.len(), 3);
        let star = top_sites.iter().find(|s| s.site == "q#0").unwrap();
        assert!(star.min_zero && star.guardable, "blockers: {:?}", star.blockers);
        assert_eq!(star.first_bytes, Some(vec![b'a']));
        assert_eq!(star.frontier, "no_refs");
        let opt = top_sites.iter().find(|s| s.site == "q#1").unwrap();
        assert!(opt.min_zero && !opt.guardable);
        assert_eq!(opt.frontier, "bare_ref");
        assert!(opt
            .blockers
            .iter()
            .any(|b| b.starts_with("element not first-byte-decided: nullable")));
        assert_eq!(opt.element_refs, vec!["nullable_opt".to_string()]);
        let plus = top_sites.iter().find(|s| s.site == "q#2").unwrap();
        assert!(!plus.min_zero && !plus.guardable);
        assert!(plus.blockers.iter().any(|b| b.starts_with("min>0 quantifier")));
        assert_eq!(plus.first_bytes, Some(vec![b't']));
        let inner = census
            .quant_sites
            .iter()
            .find(|s| s.rule == "nullable_opt")
            .expect("inner site present");
        assert!(inner.guardable, "blockers: {:?}", inner.blockers);
        assert_eq!(inner.first_bytes, Some(vec![b'x']));
    }

    /// RGX-0078.5.i.7 (Q-GUARD STEP-0) — without terminal whitespace-sensitivity
    /// the R2 layout blocker fires on every site (the next-byte peek is unsound
    /// under an implicit leading skip).
    #[test]
    fn quant_census_names_the_layout_blocker() {
        let mut tree = HashMap::new();
        tree.insert(
            "top".to_string(),
            ASTNode::Quantified {
                element: Box::new(atom("quoted_string", "a")),
                quantifier: "*".to_string(),
            },
        );
        let census = census_of(tree, vec!["top".to_string()], None);
        let site = &census.quant_sites[0];
        assert!(!site.guardable);
        assert!(site
            .blockers
            .iter()
            .any(|b| b.contains("R2 raw-byte peek unsound")));
    }

    /// RGX-0078.5.i.7 (Q-GUARD STEP-0) — the exposure join attributes a rule ONLY
    /// when every grammar-wide occurrence sits under a guardable min-0 site's
    /// element (two sites in the same rule — the `class_zero_width` shape — still
    /// attribute), routes rules with non-quantified references to the SHARED lane,
    /// and never double-counts references under NESTED guardable sites.
    #[test]
    fn quant_exposure_attributes_population_and_shared_lanes() {
        let mut tree = HashMap::new();
        // range := z* '-' z* s*   (z under TWO guardable sites; s also bare in `other`)
        tree.insert(
            "range".to_string(),
            ASTNode::Sequence {
                elements: vec![
                    ASTNode::Quantified {
                        element: Box::new(rule_ref("z")),
                        quantifier: "*".to_string(),
                    },
                    atom("quoted_string", "-"),
                    ASTNode::Quantified {
                        element: Box::new(rule_ref("z")),
                        quantifier: "*".to_string(),
                    },
                    ASTNode::Quantified {
                        element: Box::new(rule_ref("s")),
                        quantifier: "*".to_string(),
                    },
                ],
            },
        );
        // other := s 'k'   (the bare reference that keeps s in the shared lane)
        tree.insert(
            "other".to_string(),
            ASTNode::Sequence {
                elements: vec![rule_ref("s"), atom("quoted_string", "k")],
            },
        );
        // nested := ('p' n*)*   — outer guardable site whose element hosts an inner
        // guardable site; n's single occurrence must count ONCE (outermost-only).
        tree.insert(
            "nested".to_string(),
            ASTNode::Quantified {
                element: Box::new(ASTNode::Sequence {
                    elements: vec![
                        atom("quoted_string", "p"),
                        ASTNode::Quantified {
                            element: Box::new(rule_ref("n")),
                            quantifier: "*".to_string(),
                        },
                    ],
                }),
                quantifier: "*".to_string(),
            },
        );
        tree.insert("z".to_string(), atom("quoted_string", "z"));
        tree.insert("s".to_string(), atom("quoted_string", "s"));
        tree.insert("n".to_string(), atom("quoted_string", "n"));
        let payload = serde_json::json!({
            "grammar": "t",
            "accepted": true,
            "rule_entry_counts": {"z": 10u64, "s": 8u64, "n": 5u64, "range": 1u64},
            "rule_committed_counts": {"z": 4u64, "s": 8u64, "n": 2u64, "range": 1u64},
        });
        let path = std::env::temp_dir().join(format!(
            "pgen_quant_exposure_test_{}.json",
            std::process::id()
        ));
        std::fs::write(&path, serde_json::to_string(&payload).unwrap()).unwrap();
        let order = vec![
            "range".to_string(),
            "other".to_string(),
            "nested".to_string(),
            "z".to_string(),
            "s".to_string(),
            "n".to_string(),
        ];
        let census = run_fusibility_census(
            "t",
            &tree,
            &order,
            Some(whitespace_sensitive_annotations("range")).as_ref(),
            &[],
            std::slice::from_ref(&path),
        )
        .expect("census runs");
        std::fs::remove_file(&path).ok();
        let exposure = census.quant_exposure.as_ref().expect("quant exposure joined");
        assert_eq!(exposure.total_sites, 5);
        assert_eq!(exposure.min_zero_sites, 5);
        assert_eq!(exposure.guardable_sites, 5);
        // z (10/4) + n (5/2) attributable; s shared (bare ref in `other`).
        assert_eq!(
            exposure.attributable_rules,
            vec!["n".to_string(), "z".to_string()]
        );
        assert_eq!(exposure.attributable_entries, 15);
        assert_eq!(exposure.attributable_committed, 6);
        assert_eq!(exposure.attributable_discarded, 9);
        assert_eq!(exposure.shared_rules, vec!["s".to_string()]);
        assert_eq!(exposure.shared_entries, 8);
        assert_eq!(exposure.shared_discarded, 0);
        assert_eq!(
            exposure.top_attributable_rules,
            vec![
                ("z".to_string(), 10, 4, 6),
                ("n".to_string(), 5, 2, 3),
            ]
        );
        // Per-site sole attribution: z (2 grammar-wide occurrences) is sole at
        // NEITHER range site; n (1 occurrence) is sole at the nested inner site.
        let range_q0 = census
            .quant_sites
            .iter()
            .find(|s| s.rule == "range" && s.site == "q#0")
            .unwrap();
        assert!(range_q0.sole_refs.is_empty());
        assert_eq!(range_q0.sole_attributable_discarded, 0);
        let inner_n = census
            .quant_sites
            .iter()
            .find(|s| s.rule == "nested" && s.site == "q#1")
            .unwrap();
        assert_eq!(inner_n.sole_refs, vec!["n".to_string()]);
        assert_eq!(inner_n.sole_attributable_discarded, 3);
    }

    /// RGX-0078.5.i.4 (P1 STEP-0) — the inline census classifies the wrapper shapes
    /// (pass-through / alternation-leaf / shaped) and NAMES the cycle and entry
    /// blockers.
    #[test]
    fn inline_census_classifies_wrapper_shapes_and_names_blockers() {
        let mut tree = HashMap::new();
        // top (entry) := wrapper | '(' top ')'   — entry AND on a cycle.
        tree.insert(
            "top".to_string(),
            or(vec![
                rule_ref("wrapper"),
                ASTNode::Sequence {
                    elements: vec![
                        atom("quoted_string", "("),
                        rule_ref("top"),
                        atom("quoted_string", ")"),
                    ],
                },
            ]),
        );
        // wrapper := leaf   — a pure delegation frame.
        tree.insert("wrapper".to_string(), rule_ref("leaf"));
        // leaf := 'a' | 'b'   — a terminal alternation leaf.
        tree.insert(
            "leaf".to_string(),
            or(vec![atom("quoted_string", "a"), atom("quoted_string", "b")]),
        );
        // mixed := leaf 'x'   — eligible but neither pass-through nor leaf.
        tree.insert(
            "mixed".to_string(),
            ASTNode::Sequence {
                elements: vec![rule_ref("leaf"), atom("quoted_string", "x")],
            },
        );
        let census = census_of(
            tree,
            vec![
                "top".to_string(),
                "wrapper".to_string(),
                "leaf".to_string(),
                "mixed".to_string(),
            ],
            None,
        );
        let top = &census.inline_rules["top"];
        assert!(!top.eligible);
        assert!(top
            .blockers
            .iter()
            .any(|b| b.contains("reference cycle")));
        assert!(top.blockers.iter().any(|b| b == "entry rule"));
        let wrapper = &census.inline_rules["wrapper"];
        assert!(wrapper.eligible, "blockers: {:?}", wrapper.blockers);
        assert_eq!(wrapper.wrapper_class, Some(InlineWrapperClass::PassThrough));
        assert_eq!(wrapper.reference_sites, 1);
        let leaf = &census.inline_rules["leaf"];
        assert!(leaf.eligible, "blockers: {:?}", leaf.blockers);
        assert_eq!(leaf.wrapper_class, Some(InlineWrapperClass::AlternationLeaf));
        assert_eq!(leaf.reference_sites, 2); // wrapper + mixed
        let mixed = &census.inline_rules["mixed"];
        assert!(mixed.eligible, "blockers: {:?}", mixed.blockers);
        assert_eq!(mixed.wrapper_class, Some(InlineWrapperClass::Shaped));
        assert!(leaf.body_nodes >= 1 && wrapper.body_nodes >= 1);
    }

    /// RGX-0078.5.i.4 (P1 STEP-0, re-anchored by the P1a refactor) — the verdict
    /// names runtime-directive blockers from the same COMPILED table the
    /// generated parser consults: the shared free function
    /// (`rule_inline_verdict`) reads `compiled` directly, so the test builds
    /// REAL annotations and lets `Classifier::new` compile them (no map
    /// injection — stronger than the original, which faked the derived map).
    #[test]
    fn inline_rule_verdict_names_directive_blockers() {
        let mut tree = HashMap::new();
        tree.insert("gated".to_string(), rule_ref("leaf"));
        tree.insert(
            "leaf".to_string(),
            or(vec![atom("quoted_string", "a"), atom("quoted_string", "b")]),
        );
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "gated".to_string(),
            vec![super::super::SemanticAnnotation::Named {
                name: "emit_fact".to_string(),
                ast: super::super::UnifiedSemanticAST::Structured {
                    canonical: "{ kind: typedef, name: $1 }".to_string(),
                    value: super::super::UnifiedSemanticValue::Object(vec![
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "kind".to_string(),
                            value: super::super::UnifiedSemanticValue::Identifier(
                                "typedef".to_string(),
                            ),
                        },
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "name".to_string(),
                            value: super::super::UnifiedSemanticValue::RuleReference(
                                "$1".to_string(),
                            ),
                        },
                    ]),
                },
            }],
        );
        let classifier =
            Classifier::new(&tree, Some(&annotations)).expect("classifier builds");
        let (eligible, blockers) = classifier.inline_rule_verdict("gated", false, false);
        assert!(!eligible);
        assert!(blockers.iter().any(|b| b == "runtime directive @emit_fact"));
        // The unblocked dual on the same classifier.
        let (eligible, blockers) = classifier.inline_rule_verdict("leaf", false, false);
        assert!(eligible, "blockers: {blockers:?}");
        assert!(blockers.is_empty());
    }

    /// RGX-0078.5.i.4 (P1a) — the SHARED emission-decision function: the budget
    /// admits a small wrapper, prices a decided child into its parent's
    /// capped-transitive weight, and excludes a rule whose `weight × sites`
    /// exceeds the duplication cap — with cycle/entry rules never decided.
    #[test]
    fn compute_inline_decisions_applies_budget_and_gates() {
        let mut tree = HashMap::new();
        // entry -> wrapper -> leaf; `popular` is tiny but referenced from more
        // sites than the duplication cap admits; `cyclic` references itself.
        tree.insert("entry".to_string(), rule_ref("wrapper"));
        tree.insert("wrapper".to_string(), rule_ref("leaf"));
        tree.insert(
            "leaf".to_string(),
            or(vec![atom("quoted_string", "a"), atom("quoted_string", "b")]),
        );
        let popular_sites: Vec<ASTNode> = (0..INLINE_DUPLICATION_CAP + 1)
            .map(|_| rule_ref("popular"))
            .collect();
        tree.insert(
            "hub".to_string(),
            ASTNode::Sequence {
                elements: popular_sites,
            },
        );
        tree.insert("popular".to_string(), atom("quoted_string", "p"));
        tree.insert(
            "cyclic".to_string(),
            ASTNode::Sequence {
                elements: vec![atom("quoted_string", "c"), rule_ref("cyclic")],
            },
        );
        let decisions = compute_inline_decisions(&tree, None, None, Some("entry"));
        assert!(!decisions["entry"].decided, "entry rule is never decided");
        assert!(!decisions["cyclic"].eligible && !decisions["cyclic"].decided);
        assert!(decisions["leaf"].decided, "{:?}", decisions["leaf"]);
        // wrapper's weight prices leaf's inlined body in and stays under the
        // caps (wrapper has ONE reference site, so the duplication cap holds).
        assert!(decisions["wrapper"].decided, "{:?}", decisions["wrapper"]);
        assert!(
            decisions["wrapper"].weight > decisions["leaf"].weight,
            "decided child expands into the parent weight: {:?} vs {:?}",
            decisions["wrapper"],
            decisions["leaf"]
        );
        // popular: weight ≥1 × sites (> cap) ⇒ over budget, never decided.
        assert!(decisions["popular"].eligible);
        assert!(
            !decisions["popular"].decided,
            "duplication cap must exclude it: {:?}",
            decisions["popular"]
        );
    }

    /// RGX-0078.5.i.4 (P1 STEP-0) — the exposure join decomposes eligible-rule
    /// entries (raw/committed/discarded) and surfaces the memo-hit share from the
    /// extended outcome dump (pre-`.5.i.4` files deserialize with zero hits).
    #[test]
    fn inline_exposure_joins_outcome_and_memo_hit_counts() {
        let mut tree = HashMap::new();
        tree.insert(
            "top".to_string(),
            or(vec![
                rule_ref("tok_wrapper"),
                ASTNode::Sequence {
                    elements: vec![
                        atom("quoted_string", "("),
                        rule_ref("top"),
                        atom("quoted_string", ")"),
                    ],
                },
            ]),
        );
        tree.insert("tok_wrapper".to_string(), rule_ref("tok"));
        tree.insert(
            "tok".to_string(),
            or(vec![atom("quoted_string", "*"), atom("quoted_string", "+")]),
        );
        // RGX-0078.5.i.4 (P1b pricing) — an ELIGIBLE-but-OVER-BUDGET rule: a
        // 14-branch alternation whose capped-transitive weight exceeds
        // INLINE_EXPANSION_CAP (12), so it stays a method call. Its counts must
        // land in the eligible sums but NOT the decided sums.
        tree.insert(
            "fat".to_string(),
            or((0..14)
                .map(|i| atom("quoted_string", &format!("k{i}")))
                .collect()),
        );
        let payload = serde_json::json!({
            "grammar": "t",
            "accepted": true,
            "total_entries": 62,
            "total_committed": 17,
            "total_memo_hits": 10,
            "rule_entry_counts": {"tok_wrapper": 20, "tok": 20, "top": 10, "fat": 12},
            "rule_committed_counts": {"tok_wrapper": 2, "tok": 2, "top": 10, "fat": 3},
            "rule_memo_hit_counts": {"tok": 5, "top": 3, "fat": 2},
        });
        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "pgen_inline_exposure_test_{}.json",
            std::process::id()
        ));
        std::fs::write(&path, serde_json::to_string(&payload).unwrap()).unwrap();
        let census = run_fusibility_census(
            "t",
            &tree,
            &[
                "top".to_string(),
                "tok_wrapper".to_string(),
                "tok".to_string(),
                "fat".to_string(),
            ],
            None,
            &[],
            std::slice::from_ref(&path),
        )
        .expect("census runs");
        std::fs::remove_file(&path).ok();
        // The budget split the test relies on: fat is eligible but over the
        // expansion cap (weight > 12), so it must never be decided.
        let fat = &census.inline_rules["fat"];
        assert!(fat.eligible, "fat must be eligible: {fat:?}");
        assert!(!fat.decided, "fat must be over-budget: {fat:?}");
        assert!(census.inline_rules["tok"].decided);
        assert!(census.inline_rules["tok_wrapper"].decided);
        let exposure = census.inline_exposure.as_ref().expect("exposure joined");
        // top is entry + cyclic ⇒ ineligible; tok_wrapper + tok + fat are eligible.
        assert_eq!(exposure.eligible_entries, 52);
        assert_eq!(exposure.eligible_committed, 7);
        assert_eq!(exposure.eligible_discarded, 45);
        assert_eq!(exposure.eligible_memo_hits, 7); // tok 5 + fat 2 — top's 3 are blocked
        assert_eq!(exposure.total_memo_hits, 10);
        // RGX-0078.5.i.4 (P1b pricing) — the DECIDED sums exclude the
        // over-budget fat rule (and, as before, the ineligible top).
        assert_eq!(exposure.decided_entries, 40);
        assert_eq!(exposure.decided_committed, 4);
        assert_eq!(exposure.decided_discarded, 36);
        assert_eq!(exposure.decided_memo_hits, 5);
        // Sorted by raw entries desc, then name: tok (20) before tok_wrapper (20).
        assert_eq!(exposure.top_eligible_rules[0].0, "tok");
        assert_eq!(exposure.top_eligible_rules[0], ("tok".to_string(), 20, 2, 5));
        assert_eq!(
            exposure.top_eligible_rules[1],
            ("tok_wrapper".to_string(), 20, 2, 0)
        );
    }

    fn transform_annotations(rule: &str) -> Annotations {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            rule.to_string(),
            vec![super::super::SemanticAnnotation::Named {
                name: "transform".to_string(),
                ast: super::super::UnifiedSemanticAST::Structured {
                    canonical: String::new(),
                    value: super::super::UnifiedSemanticValue::Boolean(true),
                },
            }],
        );
        annotations
    }

    /// RGX-0078.5.i.7 (D2 STEP-0) — the cascade gate ALLOWS cycles (unlike the
    /// scanner tier gate): an effect-free mutually-recursive pair under an
    /// ineligible (@transform-carrying) parent partitions as one region rooted at
    /// the protocol boundary, with the cycle recorded as a named fact.
    #[test]
    fn cascade_gate_allows_cycles_and_roots_at_the_protocol_boundary() {
        let mut tree = HashMap::new();
        // top(@transform) := a ; a := 'x' | b ; b := a   (a↔b cycle, effect-free)
        tree.insert("top".to_string(), or(vec![rule_ref("a")]));
        tree.insert(
            "a".to_string(),
            or(vec![atom("quoted_string", "x"), rule_ref("b")]),
        );
        tree.insert("b".to_string(), or(vec![rule_ref("a")]));
        let census = census_of(
            tree,
            vec!["top".to_string(), "a".to_string(), "b".to_string()],
            Some(transform_annotations("top")),
        );
        let top = &census.cascade_rules["top"];
        assert!(!top.eligible);
        assert!(top.reasons.iter().any(|r| r.contains("@transform")));
        let a = &census.cascade_rules["a"];
        assert!(a.eligible, "cycle must not block: {:?}", a.reasons);
        assert!(a.root, "a is referenced by the ineligible top");
        assert!(a.on_cycle);
        let b = &census.cascade_rules["b"];
        assert!(b.eligible);
        assert!(!b.root, "b is referenced only from inside the region");
        assert!(b.on_cycle);
        // The scanner tier gate REJECTS the same pair (cycle) — the two lanes must
        // keep disagreeing here by design.
        assert_eq!(census.rules["a"].tier, FusibilityTier::NotFusible);
    }

    /// RGX-0078.5.i.7 (D2 STEP-0) — a reference from an eligible rule to an
    /// ineligible one is a BOUNDARY call-out (named per rule), never a blocker.
    #[test]
    fn cascade_boundary_refs_name_ineligible_callouts() {
        let mut tree = HashMap::new();
        tree.insert(
            "x".to_string(),
            ASTNode::Sequence {
                elements: vec![atom("quoted_string", "("), rule_ref("y")],
            },
        );
        tree.insert("y".to_string(), or(vec![atom("quoted_string", "z")]));
        let census = census_of(
            tree,
            vec!["x".to_string(), "y".to_string()],
            Some(transform_annotations("y")),
        );
        let x = &census.cascade_rules["x"];
        assert!(x.eligible);
        assert_eq!(x.boundary_refs, vec!["y".to_string()]);
        assert!(!census.cascade_rules["y"].eligible);
    }

    /// RGX-0078.5.i.7 (D2 STEP-0) — the exposure join partitions entries into
    /// internal (eliminated) / roots (kept as fused calls) / residual (untouched),
    /// with unmatched rules surfaced and the committed floor = roots + residual.
    #[test]
    fn cascade_exposure_join_partitions_entries() {
        let mut tree = HashMap::new();
        // top(@transform, ineligible) := a ; a := 'x' | b ; b := 'y'
        // ⇒ a = ROOT (referenced by ineligible top), b = internal.
        tree.insert("top".to_string(), or(vec![rule_ref("a")]));
        tree.insert(
            "a".to_string(),
            or(vec![atom("quoted_string", "x"), rule_ref("b")]),
        );
        tree.insert("b".to_string(), or(vec![atom("quoted_string", "y")]));
        let payload = serde_json::json!({
            "grammar": "t",
            "accepted": true,
            "rule_entry_counts": {"top": 10, "a": 8, "b": 20, "zz_unknown": 2},
            "rule_committed_counts": {"top": 6, "a": 5, "b": 12},
            "rule_memo_hit_counts": {"a": 1, "b": 3},
        });
        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "pgen_cascade_outcome_test_{}.json",
            std::process::id()
        ));
        std::fs::write(&path, serde_json::to_string(&payload).unwrap()).unwrap();
        let census = run_fusibility_census(
            "t",
            &tree,
            &["top".to_string(), "a".to_string(), "b".to_string()],
            Some(transform_annotations("top")).as_ref(),
            &[],
            std::slice::from_ref(&path),
        )
        .expect("census runs");
        std::fs::remove_file(&path).ok();
        let exposure = census.cascade_exposure.as_ref().expect("exposure joined");
        assert_eq!(exposure.total_entries, 40);
        assert_eq!(exposure.internal_entries, 20);
        assert_eq!(exposure.internal_committed, 12);
        assert_eq!(exposure.internal_discarded, 8);
        assert_eq!(exposure.internal_memo_hits, 3);
        assert_eq!(exposure.root_entries, 8);
        assert_eq!(exposure.root_committed, 5);
        assert_eq!(exposure.root_memo_hits, 1);
        assert_eq!(exposure.residual_entries, 10);
        assert_eq!(exposure.residual_committed, 6);
        assert_eq!(exposure.unmatched_rules, vec!["zz_unknown".to_string()]);
        assert_eq!(exposure.unmatched_entries, 2);
        assert_eq!(exposure.committed_floor, 11);
        assert_eq!(
            exposure.top_internal_rules,
            vec![("b".to_string(), 20, 12, 8)]
        );
    }

    /// RGX-0078.5.i.7 (D2-A) — the acyclic-sub-region plan treats CYCLIC eligible
    /// rules as protocol boundaries: an acyclic chain hanging off a cycle
    /// partitions as sub-root (entered from the cyclic caller) + internal
    /// (referenced only from inside the fused acyclic graph); the cyclic pair
    /// itself joins NEITHER set in this increment.
    #[test]
    fn cascade_plan_partitions_acyclic_subregions_under_cyclic_callers() {
        let mut tree = HashMap::new();
        // entry := a ; a := 'x' | b | c ; b := a   (a↔b cyclic, effect-free)
        // c := d ; d := 'y'                        (the acyclic chain off the cycle)
        tree.insert("entry".to_string(), or(vec![rule_ref("a")]));
        tree.insert(
            "a".to_string(),
            or(vec![atom("quoted_string", "x"), rule_ref("b"), rule_ref("c")]),
        );
        tree.insert("b".to_string(), or(vec![rule_ref("a")]));
        tree.insert("c".to_string(), or(vec![rule_ref("d")]));
        tree.insert("d".to_string(), or(vec![atom("quoted_string", "y")]));
        let plan =
            compute_cascade_emission_plan(&tree, None, Some("entry")).expect("plan computes");
        assert!(
            plan.sub_roots.contains("entry"),
            "the entry rule is a sub-root: {:?}",
            plan.sub_roots
        );
        assert!(
            plan.sub_roots.contains("c"),
            "c is entered from the cyclic caller a: {:?}",
            plan.sub_roots
        );
        assert!(
            plan.internal.contains("d"),
            "d is referenced only from the fused acyclic graph: {:?}",
            plan.internal
        );
        for cyclic in ["a", "b"] {
            assert!(
                !plan.sub_roots.contains(cyclic) && !plan.internal.contains(cyclic),
                "cyclic rules stay protocol boundaries in increment A: {cyclic}"
            );
        }
        // Purely structural grammar: nothing reaches an ineligible rule.
        assert!(plan.effect_reaching.is_empty());
    }

    /// RGX-0078.5.i.7 (D2-A) — effect-reachability is transitive and propagates
    /// through NON-fused eligible rules too (a method call to an effect-free
    /// cyclic rule can still reach a fact-writing descendant); an ineligible rule
    /// that reaches no OTHER ineligible rule is not itself in the set (the
    /// emitter checks call-target eligibility separately).
    #[test]
    fn cascade_plan_effect_reaching_is_transitive_through_methods() {
        let mut tree = HashMap::new();
        // p := q ; q := r | 'z' q  (q cyclic eligible — NOT fused) ; r(@transform).
        tree.insert("p".to_string(), or(vec![rule_ref("q")]));
        tree.insert(
            "q".to_string(),
            or(vec![
                rule_ref("r"),
                ASTNode::Sequence {
                    elements: vec![atom("quoted_string", "z"), rule_ref("q")],
                },
            ]),
        );
        tree.insert("r".to_string(), or(vec![atom("quoted_string", "w")]));
        let ann = transform_annotations("r");
        let plan =
            compute_cascade_emission_plan(&tree, Some(&ann), Some("p")).expect("plan computes");
        assert!(
            plan.effect_reaching.contains("q"),
            "q references the ineligible r directly: {:?}",
            plan.effect_reaching
        );
        assert!(
            plan.effect_reaching.contains("p"),
            "p reaches r THROUGH the non-fused cyclic q: {:?}",
            plan.effect_reaching
        );
        assert!(
            !plan.effect_reaching.contains("r"),
            "r is ineligible but reaches no ineligible rule itself"
        );
        assert!(plan.sub_roots.contains("p"), "p is the entry sub-root");
        assert!(
            !plan.sub_roots.contains("q") && !plan.internal.contains("q"),
            "cyclic q stays a protocol boundary"
        );
    }

    /// RGX-0078.5.i.7 (D2-A) — the entry rule and unreferenced eligible acyclic
    /// rules are sub-roots (entered from outside the fused graph by definition),
    /// and the census carries the SAME plan codegen consumes (no drift).
    #[test]
    fn cascade_plan_entry_and_unreferenced_rules_are_sub_roots() {
        let mut tree = HashMap::new();
        tree.insert("entry".to_string(), or(vec![rule_ref("leaf")]));
        tree.insert("leaf".to_string(), or(vec![atom("quoted_string", "x")]));
        tree.insert("orphan".to_string(), or(vec![atom("quoted_string", "o")]));
        let plan =
            compute_cascade_emission_plan(&tree, None, Some("entry")).expect("plan computes");
        assert!(plan.sub_roots.contains("entry"));
        assert!(
            plan.sub_roots.contains("orphan"),
            "unreferenced rules are entered from outside the fused graph: {:?}",
            plan.sub_roots
        );
        assert!(plan.internal.contains("leaf"));
        let census = census_of(
            tree,
            vec![
                "entry".to_string(),
                "leaf".to_string(),
                "orphan".to_string(),
            ],
            None,
        );
        assert_eq!(census.cascade_plan.sub_roots, plan.sub_roots);
        assert_eq!(census.cascade_plan.internal, plan.internal);
        assert_eq!(census.cascade_plan.effect_reaching, plan.effect_reaching);
    }
}
