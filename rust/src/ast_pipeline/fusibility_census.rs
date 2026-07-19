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
    /// RGX-0078.5.i.7 (D2-B plan seam) — the CYCLIC-SPINE increment's plan
    /// ([`compute_cascade_emission_plan_for_increment`] at
    /// [`CascadeIncrement::CyclicSpine`]): every cascade-eligible rule fused,
    /// sub-roots = the census's own full-fold roots, `thin_memo` = the cycle
    /// participants (the ⛔ #49 carriers). Reported (`CASCADE-PLAN-B`) ahead of
    /// the D2-B emitter consuming it — the same no-drift seam as `cascade_plan`.
    pub cascade_plan_b: CascadeEmissionPlan,
    /// RGX-0078.5.j.2 (STEP-1, the plan seam) — the DIRECT-VALUE BUILD
    /// partition of the fused rule set ([`compute_direct_value_build_plan`] —
    /// the same map the `.5.j.2` emitter consumes). Reported
    /// (`DIRECT-VALUE-PLAN`) ahead of emission — the same no-drift seam.
    /// Additive census-JSON field (the `-0089` additive-only precedent).
    pub direct_value_plan: DirectValueBuildPlan,
    /// RGX-0078.5.i.9 (D3) — the boundary-scanner emission plan
    /// ([`compute_boundary_scanner_plan`] — the same map the scan emitter
    /// consumes). Reported (`BOUNDARY-SCANNER-PLAN`) — the same no-drift seam.
    pub boundary_scanner_plan: BoundaryScannerPlan,
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
    /// RGX-0078.5.j.4 K4b C1 — the per-branch FIRSTₖ (bounded prefix-trie)
    /// guard verdicts (top-level sites only; empty for nested sites, which are
    /// R1-blocked at every level). Driven by the SAME shared license the
    /// emitter consumes.
    pub firstk_branches: Vec<FirstkBranchCensus>,
}

/// RGX-0078.5.j.4 K4b C1 — one branch's FIRSTₖ prefix-trie guard census row.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FirstkBranchCensus {
    /// 1-based branch index at the site.
    pub index: usize,
    /// `"guarded"` or the NAMED level-1 refusal (the branch keeps today's
    /// unguarded emission).
    pub verdict: String,
    /// Usable walk depth (1 = level-1-degenerate; 0 when refused).
    pub max_depth: usize,
    /// The guard refines nothing beyond level 1 (today's exact emission).
    pub level1_degenerate: bool,
    /// A cap (depth 4 / fanout 24 / 16 nodes) truncated the per-path analysis
    /// somewhere along this branch's trie.
    pub truncated: bool,
    /// The D1 global FIRST₂ layer refined at least one depth-1 leaf.
    pub d1_fallback_used: bool,
    /// Distinct nonzero furthest-emulation offsets over the refutation arms.
    pub emulation_offsets: Vec<u8>,
    /// Finalized trie size (nodes incl. the root).
    pub nodes: usize,
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
    /// RGX-0078.5.j.4 K4b C1 — shared per-rule prefix-trie cache (the FIRSTₖ lane).
    trie_cache: HashMap<String, super::first_set::PrefixTrieNode>,
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
            trie_cache: HashMap::new(),
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

    /// RGX-0078.5.j.4 K4b C1 — the branch's FIRSTₖ prefix-trie guard through the
    /// SHARED licensing function codegen's emission consumes (`first_set::
    /// branch_prefix_trie_guard`) — the census verdict and the emitted guard
    /// cannot drift. The regex-token trust flag mirrors codegen's
    /// `layout_sensitivity().regex_tokens` exactly as the level-1/FIRST₂ lanes do.
    fn branch_prefix_trie_guard(
        &mut self,
        branch: &ASTNode,
    ) -> Result<super::first_set::PrefixTrieGuard, String> {
        super::first_set::branch_prefix_trie_guard(
            branch,
            self.tree,
            &mut self.first_set_cache,
            &mut self.second_byte_cache,
            &mut self.trie_cache,
            self.layout.regex_tokens,
        )
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
        // RGX-0078.5.i.7 (D2-A) — four outcome-relevant rule-policy knobs the compiled
        // runtime-directive table does NOT carry (they are codegen-time policies, not
        // store directives), found by the emission slice's soundness audit of every
        // behavior knob `generate_rule_body_inner`/`generate_or_logic` consult. Each is
        // read through the SAME shared registry resolution codegen emits from, so gate
        // and emission cannot drift:
        // - `@stop_at_rule_boundary` family: quantifier-loop break/error policy;
        // - `@recover`: tournament failure-path recovery;
        // - nonzero `@coverage_target`: an unconditional `record_coverage_target_event`
        //   at the rule tail (observable parser state on every parse);
        // - `@invalid_case`: `record_negative_case_failure` on the method failure path.
        if super::semantic_directive_registry::effective_rule_bool_directive(
            self.annotations,
            rule,
            &[
                "stop_at_rule_boundary",
                "stop_on_rule_boundary",
                "line_delimited_sequence",
            ],
        ) {
            reasons.push("@stop_at_rule_boundary quantifier break policy".to_string());
        }
        if super::semantic_directive_registry::effective_rule_recovery_enabled(
            self.annotations,
            rule,
        ) {
            reasons.push("@recover failure-path recovery".to_string());
        }
        if super::semantic_directive_registry::effective_rule_coverage_target_weight(
            self.annotations,
            rule,
        ) != 0
        {
            reasons.push("@coverage_target event recording".to_string());
        }
        if super::semantic_directive_registry::effective_rule_negative_case_enabled(
            self.annotations,
            rule,
        ) {
            reasons.push("@invalid_case negative-case recording".to_string());
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
                // RGX-0078.5.j.4 K4b C1 — the FIRSTₖ lane: per-branch prefix-trie
                // guard verdicts for top-level sites, under the SAME gate as
                // codegen's `emit_first_set_guard` (rule-top-level Or +
                // terminal-whitespace-sensitive layout).
                let firstk_branches: Vec<FirstkBranchCensus> = if is_rule_body {
                    alternatives
                        .iter()
                        .enumerate()
                        .map(|(i, branch)| {
                            if !classifier.layout.terminals {
                                return FirstkBranchCensus {
                                    index: i + 1,
                                    verdict:
                                        "terminals skip leading layout (R2 raw-byte peek unsound)"
                                            .to_string(),
                                    max_depth: 0,
                                    level1_degenerate: false,
                                    truncated: false,
                                    d1_fallback_used: false,
                                    emulation_offsets: Vec::new(),
                                    nodes: 0,
                                };
                            }
                            match classifier.branch_prefix_trie_guard(branch) {
                                Ok(guard) => FirstkBranchCensus {
                                    index: i + 1,
                                    verdict: "guarded".to_string(),
                                    max_depth: guard.max_depth,
                                    level1_degenerate: guard.is_level1_degenerate(),
                                    truncated: guard.truncated,
                                    d1_fallback_used: guard.d1_fallback_used,
                                    emulation_offsets: guard.emulation_offsets(),
                                    nodes: guard.root.count_nodes(),
                                },
                                Err(reason) => FirstkBranchCensus {
                                    index: i + 1,
                                    verdict: reason,
                                    max_depth: 0,
                                    level1_degenerate: false,
                                    truncated: false,
                                    d1_fallback_used: false,
                                    emulation_offsets: Vec::new(),
                                    nodes: 0,
                                },
                            }
                        })
                        .collect()
                } else {
                    Vec::new()
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
                    firstk_branches,
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
    // RGX-0078.5.i.7 (D2-B plan seam) — the cyclic-spine increment's plan.
    let cascade_plan_b = compute_cascade_emission_plan_for_increment(
        grammar_tree,
        annotations,
        entry_rule.as_deref(),
        CascadeIncrement::CyclicSpine,
    )?;

    // RGX-0078.5.j.2 (STEP-1) — the direct-value build plan (the same SHARED
    // function the value-twin emitter consumes), reported ahead of emission.
    let direct_value_plan =
        compute_direct_value_build_plan(grammar_tree, annotations, entry_rule.as_deref())?;

    // RGX-0078.5.i.9 (D3) — the boundary-scanner plan (the same SHARED function the
    // scan emitter consumes), reported ahead of emission — the no-drift seam.
    let boundary_scanner_plan =
        compute_boundary_scanner_plan(grammar_tree, annotations, entry_rule.as_deref())?;

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
        cascade_plan_b,
        direct_value_plan,
        boundary_scanner_plan,
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
    /// RGX-0078.5.i.7 (D2-A emitter) — the per-SITE form of the same obligation: a
    /// speculation scope (branch attempt / quantifier iteration / optional attempt /
    /// lookahead) inside a fused body reaches semantic effects iff its subtree
    /// references ANY rule in this set. It is exactly `{ r ∈ tree : ¬eligible(r) } ∪
    /// effect_reaching` — an ineligible target may itself carry a directive, and an
    /// eligible target may reach a fact-writing descendant (already closed under the
    /// monotone fixpoint above), so membership is a sound per-reference test with no
    /// further closure walk needed at the emission site.
    pub effect_targets: std::collections::BTreeSet<String>,
    /// RGX-0078.5.i.7 (D2-B plan seam) — the CYCLE-PARTICIPATING fused rules: every
    /// fused rule that can reach itself through tree references. EMPTY under
    /// increment A by construction (cyclic eligible rules are protocol boundaries
    /// there, never fused). Under increment B these are the rules whose fused
    /// `cascade_*` functions recurse, and the ⛔ session-#49 bound applies: they
    /// NEVER lose memo protection — the emitter gives each an epoch-stamped thin
    /// memo (`(rule, pos) → {end, value, write_epoch}` validated at replay, the
    /// MEMO-STORE-SOUNDNESS.2 semantics) so packrat asymptotics are preserved.
    pub thin_memo: std::collections::BTreeSet<String>,
}

/// RGX-0078.5.i.7 (D2-B plan seam) — which cascade-fold increment a plan is
/// computed for. The partition logic is ONE implementation; the increment only
/// widens the fused-candidate gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CascadeIncrement {
    /// D2-A: acyclic sub-regions only — a cyclic eligible rule stays a protocol
    /// boundary (no recursion, no memo lane; re-probe multiplicity bounded by
    /// the grammar's static caller constant).
    AcyclicSubRegions,
    /// D2-B: the cyclic spine folds too — every cascade-eligible rule is fused;
    /// cycle participants carry the epoch-stamped thin memo (the #49 bound).
    CyclicSpine,
}

/// Compute the D2-A cascade-emission plan (the increment consumed by the LANDED
/// `-0087` emitter). Verdicts come from the census's OWN `cascade_rule_verdict`
/// (one implementation of the D2 gate for report and emission); cyclicity from
/// `rule_reaches_itself` over the census's own reference collector.
/// Deterministic (`BTreeSet` output, monotone fixpoint).
pub fn compute_cascade_emission_plan(
    tree: &HashMap<String, ASTNode>,
    annotations: Option<&Annotations>,
    entry_rule: Option<&str>,
) -> Result<CascadeEmissionPlan, String> {
    compute_cascade_emission_plan_for_increment(
        tree,
        annotations,
        entry_rule,
        CascadeIncrement::AcyclicSubRegions,
    )
}

/// RGX-0078.5.i.7 (D2-B plan seam) — the increment-parameterized plan: ONE
/// implementation of the fused-candidate gate + sub-root/internal partition +
/// effect fixpoint for both increments, so the census report, the D2-A emitter,
/// and the D2-B emitter cannot drift. Under `CyclicSpine` the fused-candidate
/// gate drops the acyclicity requirement (fused = cascade-eligible), the
/// sub-root partition therefore reproduces the census's OWN full-fold
/// root/internal split, and `thin_memo` names the cycle participants (the ⛔
/// #49 carriers).
pub fn compute_cascade_emission_plan_for_increment(
    tree: &HashMap<String, ASTNode>,
    annotations: Option<&Annotations>,
    entry_rule: Option<&str>,
    increment: CascadeIncrement,
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
                eligible[rule.as_str()]
                    && (increment == CascadeIncrement::CyclicSpine
                        || !rule_reaches_itself(rule, &forward)),
            )
        })
        .collect();
    let thin_memo: std::collections::BTreeSet<String> = tree
        .keys()
        .filter(|rule| fused_candidate[rule.as_str()] && rule_reaches_itself(rule, &forward))
        .cloned()
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

    let effect_targets: std::collections::BTreeSet<String> = tree
        .keys()
        .filter(|rule| !eligible[rule.as_str()] || effect_reaching.contains(rule.as_str()))
        .cloned()
        .collect();

    Ok(CascadeEmissionPlan {
        sub_roots,
        internal,
        effect_reaching: effect_reaching.into_iter().map(String::from).collect(),
        effect_targets,
        thin_memo,
    })
}

/// RGX-0078.5.j.2 (STEP-1, the plan seam) — the DIRECT-VALUE BUILD partition of
/// the fused (CyclicSpine) rule set: which `cascade_build_*` functions may
/// construct the rule's committed VALUE directly (no element-wrapper
/// `ParseNode`s, no `sequence_elements` Vec, no `Sequence` content — the
/// `.5.j.1` census measured that scaffolding at 100% dead on the bench corpus),
/// and which must keep today's node-building form because their content VARIANT
/// is observable.
///
/// The soundness model (the `.5.j.1` design, tool-verified; CORRECTED session
/// #150 before any emission — the 2-way barrier test mis-classified
/// content-RE-EMITTING folds as demand-stopping):
/// - `ParseContent::to_shaped_value` is COMPOSITIONAL — converting children
///   early and assembling the composite value directly produces the same
///   `PgenValue` as materializing the node tree and converting late
///   (`Terminal→Str`, `Shaped→copy`, `Alternative→recurse`,
///   `Sequence`/`Quantified→Array` element-wise, `TransformedTerminal` parses
///   the SAME text either way). So a rule's scaffolding may be elided wherever
///   every consumer folds its content to a value.
/// - A **VALUE-PURE** branch transform (object/scalar literals, identifier,
///   property access, `$text`, rule-level matched-text `@transform`) consumes
///   every child reference as a FOLDED VALUE and rebuilds its output content
///   (`Shaped(...)`/`Terminal(...)`) from scratch: its content variant is
///   independent of how children were built, and it demands NO node-form
///   children.
/// - A **CONTENT-CARRYING** branch transform (array literal, spread,
///   flatten-spread, array access, quantified extraction) RE-EMITS child
///   content or child NODES inside its output (`generate_array_transform`
///   pushes child `ParseNode`s / clones child content), so it is NOT a
///   demand stop: where the rule's own content is observable, the referenced
///   children's content is observable through it.
/// - A **TRANSPARENT** branch (bare `$N`/passthrough/no annotation) re-emits
///   child content verbatim. Node-form demand propagates PER-REFERENCE: a
///   `$N` branch demands exactly element N−1's subtree references; a
///   passthrough branch demands the whole branch body's references; a
///   content-carrying branch of a demanded rule demands the WHOLE branch body
///   (v1: such branches are emitted verbatim; STEP-2b's in-place input feeding
///   narrows this to the `$N`-targeted elements).
/// - Escape roots = the plan's fused SUB-ROOTS: their orchestrators return the
///   `ParseNode` to the protocol zone (memo entries, semantic flattening,
///   entry-relative parses, the committed root), where the content variant is
///   serialized/observable — they must keep today-form content.
/// - A **VOCABULARY AUDIT** (the D3 `ScanAudit` precedent) demotes any rule
///   whose value-form emission would need a static decision the emitter cannot
///   make (a `Spread` distinguishes `Sequence`/`Quantified` from
///   `Shaped(Array)` at runtime; a `FlattenSpread` item carrying
///   `TransformedTerminal` JSON-array text would splice where today it nests;
///   `ArrayAccess`/`QuantifiedExtraction` are out of the v1 vocabulary) to
///   verbatim node emission, with NAMED reasons — never a silent drop.
///
/// Deterministic (`BTreeSet` output, monotone worklist over sorted sets).
/// Consumed by BOTH the census report (`DIRECT-VALUE-PLAN`) and the `.5.j.2`
/// emitter — the `compute_cascade_emission_plan` no-drift precedent.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DirectValueBuildPlan {
    /// Fused rules whose effective transform is VALUE-PURE on EVERY branch and
    /// inside the value vocabulary: build internals value-izable
    /// unconditionally (content = the fold's own `Shaped`/`Terminal` result,
    /// byte-identical however children were built); the node wrapper survives
    /// only where a consumer needs the node.
    pub barrier: std::collections::BTreeSet<String>,
    /// Fused TRANSPARENT/CONTENT-CARRYING rules NOT demanded through any
    /// content-position reference chain from an escape root, and inside the
    /// value vocabulary: every consumption path folds their content to a value
    /// before it can be observed, so the whole build may emit the converted
    /// value directly (convert-early ≡ convert-late).
    pub value_licensed: std::collections::BTreeSet<String>,
    /// Fused rules whose content variant IS observable (a fused sub-root, or
    /// reachable from one through content-position references of
    /// transparent/content-carrying/demoted rules), plus every vocabulary
    /// demotion: node-building `cascade_build_*` emission (verbatim, except
    /// that in-vocabulary VALUE-PURE branches and `$N`-targeted elements of
    /// non-demoted rules may still build their fold inputs in place — the
    /// per-branch modes the emitter derives from the same shared fns).
    pub node_locked: std::collections::BTreeSet<String>,
    /// The vocabulary-audit demotions (⊆ `node_locked`), each with its NAMED
    /// reasons — the report's honest residue (the `BoundaryScannerPlan::dropped`
    /// precedent). Additive JSON field (the `-0089` precedent).
    pub demoted: BTreeMap<String, Vec<String>>,
}

/// RGX-0078.5.j.2 (session #150 correction) — the fold class of one parsed
/// return-annotation root. Shared verbatim by the census partition and the
/// value-twin emitter (single implementation, no drift).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TransformFoldClass {
    /// Every child reference is consumed as a folded VALUE; output content is
    /// rebuilt (`Shaped`/`Terminal`) — variant-independent, demands nothing.
    ValuePure,
    /// Output content RE-EMITS child content or nodes (array literal, spread,
    /// flatten-spread, array access, quantified extraction).
    ContentCarrying,
    /// Bare `$N` / passthrough — re-emits child content verbatim.
    Transparent,
}

/// Classify one parsed return-annotation root. `Object` is VALUE-PURE even
/// with nested arrays/spreads inside property values: the whole property value
/// is folded to a `PgenValue` (compositionality covers the transient), so no
/// child content escapes.
pub fn return_ast_fold_class(
    ast: &crate::ast_pipeline::unified_return_ast::UnifiedReturnAST,
) -> TransformFoldClass {
    use crate::ast_pipeline::unified_return_ast::UnifiedReturnAST as U;
    match ast {
        U::StringLiteral { .. }
        | U::NumberLiteral { .. }
        | U::BooleanLiteral { .. }
        | U::NullLiteral
        | U::Identifier { .. }
        | U::Object { .. }
        | U::PropertyAccess { .. }
        | U::MatchedText => TransformFoldClass::ValuePure,
        U::Array { .. }
        | U::ArrayAccess { .. }
        | U::QuantifiedExtraction { .. }
        | U::Spread { .. }
        | U::FlattenSpread { .. } => TransformFoldClass::ContentCarrying,
        U::PositionalRef { .. } | U::Passthrough => TransformFoldClass::Transparent,
    }
}

/// Does `rule` carry a rule-level matched-text `@transform`? The
/// `cascade_rule_has_matched_text_transform` mirror (such a rule pins its
/// content to `TransformedTerminal` at the rule tail — VALUE-PURE, but demoted
/// from value emission defensively; the cascade gate excludes it from fusion
/// anyway).
pub fn rule_has_matched_text_transform(rule: &str, annotations: Option<&Annotations>) -> bool {
    annotations
        .and_then(|a| a.semantic_annotations.get(rule))
        .is_some_and(|entries| {
            entries.iter().any(|annotation| {
                crate::ast_pipeline::semantic_directive_registry::semantic_directive_name_payload(
                    annotation,
                )
                .is_some_and(|(name, _)| name == "transform")
            })
        })
}

/// The effective (explicit or synthesized) return-transform AST of one branch,
/// resolved EXACTLY as the build emitter resolves it
/// (`generate_mtb_build_rule_fn` / `cascade_branch_transform`): the explicit
/// parsed annotation when present; the synthesized single-element `-> $1`
/// (`AstBasedGenerator::body_has_single_element` — the SAME shared predicate)
/// ONLY when NO annotation slot exists at all; `None` = bare passthrough.
///
/// ⚠️ A PRESENT-but-UNPARSED annotation (`parsed_ast: None` — the bootstrap
/// parse-failure warning path) is NOT synthesized to `$1`: the emitter's
/// warning path re-emits the WHOLE content (`result.clone()`), which is
/// passthrough — on a single-element `Sequence` body the two differ (`$1`
/// peels the element; passthrough keeps the one-element `Sequence` → the
/// folded `Array`-of-one), so the distinction is byte-visible.
pub fn resolved_branch_return_ast(
    rule: &str,
    branch_index: usize,
    branch_body: &ASTNode,
    annotations: Option<&Annotations>,
) -> Option<crate::ast_pipeline::unified_return_ast::UnifiedReturnAST> {
    let slot = annotations
        .and_then(|a| a.branch_return_annotations.get(rule))
        .and_then(|branches| branches.get(branch_index))
        .and_then(|opt| opt.as_ref());
    match slot {
        // An annotation exists: its parsed AST, or (unparsed — the warning
        // path) the whole-content passthrough.
        Some(annotation) => annotation.parsed_ast.clone(),
        // No annotation at all: the emitter synthesizes `-> $1` for
        // single-element bodies.
        None => {
            if AstBasedGenerator::body_has_single_element(branch_body) {
                Some(
                    crate::ast_pipeline::unified_return_ast::UnifiedReturnAST::PositionalRef {
                        index: 1,
                    },
                )
            } else {
                None
            }
        }
    }
}

/// The branch bodies of a fused rule exactly as the emitter walks them: an
/// `Or` rule's alternatives, else the whole body as branch 0.
pub fn rule_branch_bodies(body: &ASTNode) -> Vec<&ASTNode> {
    match body {
        ASTNode::Or { alternatives } => alternatives.iter().collect(),
        _ => vec![body],
    }
}

/// EVERY context a rule's branch annotations apply in. The emitter applies
/// `cascade_branch_transform(rule, idx, alt)` at EVERY `Or` site — nested ones
/// included (the branch-broadcast semantics: a nested `Or`'s branch INDEX
/// indexes the rule's branch-annotation list) — so a static audit or demand
/// walk that only looks at the top-level branch bodies mis-resolves `$N`
/// against the wrong shape (the ebnf `grammar_file` spread-context bug,
/// session #150). Returns `(annotation_index, context_body)` pairs: the
/// top-level body as branch 0 when the body is not an `Or`, plus every `Or`
/// alternative anywhere in the tree.
pub fn collect_annotation_contexts(body: &ASTNode) -> Vec<(usize, &ASTNode)> {
    fn walk<'t>(node: &'t ASTNode, out: &mut Vec<(usize, &'t ASTNode)>) {
        match node {
            ASTNode::Or { alternatives } => {
                for (idx, alternative) in alternatives.iter().enumerate() {
                    out.push((idx, alternative));
                    walk(alternative, out);
                }
            }
            ASTNode::Sequence { elements } => {
                for element in elements {
                    walk(element, out);
                }
            }
            ASTNode::Quantified { element, .. } | ASTNode::Lookahead { element, .. } => {
                walk(element, out);
            }
            ASTNode::Atom { value } => {
                if let ASTValue::Node(inner) = value {
                    walk(inner, out);
                }
            }
        }
    }
    let mut out = Vec::new();
    if !matches!(body, ASTNode::Or { .. }) {
        out.push((0, body));
    }
    walk(body, &mut out);
    out
}

/// The STATIC runtime variant of a spread base target — the codegen-time
/// collapse of `generate_spread_transform`'s dispatch (`Sequence`/`Quantified`
/// SPLICE; everything else WRAPS as one element). `at_sequence_position`
/// distinguishes the `?` forms: a `?` SEQUENCE element takes the OptPresent
/// fast path (absent = empty `Sequence` → splice; present = the INNER's
/// variant — decidable only when the inner is itself a non-`?` quantifier),
/// while a `?` BODY runs the QuantCount loop (content `Quantified` → splice
/// unconditionally).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpreadBaseStaticVariant {
    /// Content statically `Sequence`/`Quantified` — the runtime splices.
    Splice,
    /// Content statically NEITHER — the runtime wraps the value as ONE
    /// element.
    Wrap,
    /// The variant depends on runtime state or cross-rule content classes —
    /// out of the v1 vocabulary (demote).
    Undecidable,
}

pub fn spread_base_static_variant(
    node: &ASTNode,
    at_sequence_position: bool,
) -> SpreadBaseStaticVariant {
    match node {
        ASTNode::Quantified {
            element, quantifier
        } => {
            if at_sequence_position && quantifier == "?" {
                match element.as_ref() {
                    ASTNode::Quantified { quantifier: inner_q, .. } if inner_q != "?" => {
                        SpreadBaseStaticVariant::Splice
                    }
                    _ => SpreadBaseStaticVariant::Undecidable,
                }
            } else {
                SpreadBaseStaticVariant::Splice
            }
        }
        ASTNode::Sequence { .. } | ASTNode::Lookahead { .. } => SpreadBaseStaticVariant::Splice,
        ASTNode::Atom { value } => match value {
            ASTValue::Token(parts) if parts.len() >= 2 => {
                let TokenValue::String(token_type) = &parts[0];
                if token_type == "rule_reference" {
                    // The `$1`-peel reaches the CHILD rule's content — its
                    // variant is a cross-rule content class (the ebnf
                    // `grammar_file` case folded an include OBJECT here).
                    SpreadBaseStaticVariant::Undecidable
                } else {
                    // Literal / regex terminals → `Terminal` content → wrap.
                    SpreadBaseStaticVariant::Wrap
                }
            }
            _ => SpreadBaseStaticVariant::Wrap,
        },
        ASTNode::Or { .. } => SpreadBaseStaticVariant::Undecidable,
    }
}

/// What a `$N` positional reference statically resolves to against a branch
/// body — the codegen-time collapse of `generate_positional_ref`'s runtime
/// match (the body's own content variant is statically known per shape).
#[derive(Debug, Clone, Copy)]
pub enum PositionalTarget<'tree> {
    /// A multi-element `Sequence` body's element N−1 (also the single-element
    /// case: the runtime `Sequence non-empty ⇒ elements[0]` arm).
    Element(&'tree ASTNode),
    /// The whole branch body re-emitted: `$1` on an atom body (the
    /// `Alternative`-peel arm reaches the child's content — the body's own
    /// reference), `$1` on a `Quantified` body (the whole capture group), or
    /// the empty-`Sequence` fall-through.
    WholeBody,
    /// Statically out of range: the `<invalid_sequence_access>` sentinel — no
    /// child content is re-emitted.
    StaticSentinel,
}

/// Resolve `$index` against `branch_body` (1-based, the annotation spelling).
pub fn resolve_positional_target(branch_body: &ASTNode, index: usize) -> PositionalTarget<'_> {
    if index == 0 {
        // `$0` → the `<invalid_positional_ref>` sentinel.
        return PositionalTarget::StaticSentinel;
    }
    match branch_body {
        ASTNode::Sequence { elements } => {
            if elements.is_empty() {
                // Runtime guard `!elements.is_empty()` fails ⇒ the `other`
                // arm re-emits the (empty) whole content.
                PositionalTarget::WholeBody
            } else if index <= elements.len() {
                PositionalTarget::Element(&elements[index - 1])
            } else {
                PositionalTarget::StaticSentinel
            }
        }
        ASTNode::Quantified { .. } => {
            if index == 1 {
                // PGEN-RGX-0075: `$1` on a Quantified body is the WHOLE
                // capture group.
                PositionalTarget::WholeBody
            } else {
                PositionalTarget::StaticSentinel
            }
        }
        _ => {
            if index == 1 {
                PositionalTarget::WholeBody
            } else {
                PositionalTarget::StaticSentinel
            }
        }
    }
}

/// Collect every `PositionalRef` index occurring anywhere in a transform AST.
/// Consumed by the `.5.j.2` value-twin emitter to decide which body elements a
/// branch BINDS (referenced) vs discard-walks (unreferenced) — conservative:
/// value-position `$N`s inside object properties are included, which can only
/// over-bind.
pub fn collect_positional_indices(
    ast: &crate::ast_pipeline::unified_return_ast::UnifiedReturnAST,
    out: &mut std::collections::BTreeSet<usize>,
) {
    use crate::ast_pipeline::unified_return_ast::UnifiedReturnAST as U;
    match ast {
        U::PositionalRef { index } => {
            out.insert(*index);
        }
        U::Array { elements } => {
            for element in elements {
                collect_positional_indices(element, out);
            }
        }
        U::Object { properties } => {
            for value in properties.values() {
                collect_positional_indices(value, out);
            }
        }
        U::Spread { base }
        | U::FlattenSpread { base }
        | U::PropertyAccess { base, .. }
        | U::QuantifiedExtraction { base, .. } => collect_positional_indices(base, out),
        U::ArrayAccess { base, index } => {
            collect_positional_indices(base, out);
            collect_positional_indices(index, out);
        }
        U::StringLiteral { .. }
        | U::NumberLiteral { .. }
        | U::BooleanLiteral { .. }
        | U::NullLiteral
        | U::Identifier { .. }
        | U::Passthrough
        | U::MatchedText => {}
    }
}

/// RGX-0078.5.j.2 (session #150) — the v1 VALUE-EMISSION VOCABULARY audit for
/// one branch. Appends a NAMED reason per static decision the value-twin
/// emitter cannot make; an empty result means the branch is value-emittable.
/// Shared by the census (rule demotion) and the emitter (which never emits a
/// value form outside it — bailing loudly instead, so census and emission
/// cannot drift).
pub fn branch_value_vocabulary_reasons(
    ast: Option<&crate::ast_pipeline::unified_return_ast::UnifiedReturnAST>,
    branch_body: &ASTNode,
    fused: &std::collections::BTreeSet<&str>,
    reasons: &mut Vec<String>,
) {
    let Some(ast) = ast else {
        // Bare passthrough: the value form is the whole-body fold — always
        // emittable.
        return;
    };
    audit_value_ast(ast, branch_body, fused, false, reasons);
}

/// The flatten-spread item audit: every pushed item's content class must be
/// statically `TransformedTerminal`-free (a `TransformedTerminal` carrying
/// JSON-array text folds to an `Array` and would SPLICE where today's
/// node-dispatch NESTS it). v1: the quantified inner must be a literal/regex
/// terminal (items fold to `Str`) or a FUSED rule reference (fused rules never
/// emit `TransformedTerminal` content — the cascade gate excludes `@transform`
/// rules — and their transparent chains re-emit only fused/terminal content
/// checked the same way at their own audit).
fn flatten_items_statically_tt_free(
    inner: &ASTNode,
    fused: &std::collections::BTreeSet<&str>,
) -> bool {
    match inner {
        ASTNode::Atom { value } => match value {
            ASTValue::Token(parts) if parts.len() >= 2 => {
                let TokenValue::String(token_type) = &parts[0];
                let TokenValue::String(token_value) = &parts[1];
                match token_type.as_str() {
                    "rule_reference" => fused.contains(token_value.as_str()),
                    // Literal / regex terminals fold to `Str` items.
                    _ => true,
                }
            }
            _ => false,
        },
        _ => false,
    }
}

/// A FLATTEN base in Splice form needs the per-item TT-freedom guarantee; v1
/// admits only the non-`?` Quantified shape (inner audited), demoting the
/// other splice-able shapes conservatively.
fn flatten_splice_base_auditable(
    node: &ASTNode,
    fused: &std::collections::BTreeSet<&str>,
) -> bool {
    match node {
        ASTNode::Quantified {
            element, quantifier
        } if quantifier != "?" => flatten_items_statically_tt_free(element, fused),
        _ => false,
    }
}

fn audit_value_ast(
    ast: &crate::ast_pipeline::unified_return_ast::UnifiedReturnAST,
    branch_body: &ASTNode,
    fused: &std::collections::BTreeSet<&str>,
    inside_array: bool,
    reasons: &mut Vec<String>,
) {
    use crate::ast_pipeline::unified_return_ast::UnifiedReturnAST as U;
    match ast {
        U::PositionalRef { .. }
        | U::Passthrough
        | U::StringLiteral { .. }
        | U::NumberLiteral { .. }
        | U::BooleanLiteral { .. }
        | U::NullLiteral
        | U::Identifier { .. }
        | U::MatchedText => {}
        U::Object { properties } => {
            // Deterministic reason order (HashMap source).
            let mut sorted: Vec<_> = properties.iter().collect();
            sorted.sort_by(|(a, _), (b, _)| a.cmp(b));
            for (_, value) in sorted {
                audit_value_ast(value, branch_body, fused, false, reasons);
            }
        }
        U::Array { elements } => {
            for element in elements {
                audit_value_ast(element, branch_body, fused, true, reasons);
            }
        }
        U::PropertyAccess { base, .. } => {
            audit_value_ast(base, branch_body, fused, false, reasons);
        }
        U::Spread { base } | U::FlattenSpread { base } => {
            let _ = inside_array; // spread semantics are audited identically at
            // array-element and top-level positions.
            let flatten = matches!(ast, U::FlattenSpread { .. });
            let label = if flatten { "flatten-spread" } else { "spread" };
            let U::PositionalRef { index } = base.as_ref() else {
                reasons.push(format!("{label} base is not a positional reference"));
                return;
            };
            let (target_node, at_sequence_position): (&ASTNode, bool) =
                match resolve_positional_target(branch_body, *index) {
                    PositionalTarget::Element(element) => (element, true),
                    PositionalTarget::WholeBody => (branch_body, false),
                    PositionalTarget::StaticSentinel => {
                        // Static sentinel base: the spread wraps a static
                        // `Terminal` — emittable.
                        return;
                    }
                };
            match spread_base_static_variant(target_node, at_sequence_position) {
                SpreadBaseStaticVariant::Splice => {
                    if flatten && !flatten_splice_base_auditable(target_node, fused) {
                        reasons.push(format!(
                            "flatten-spread ${index} splice items not statically TransformedTerminal-free (non-fused, structured, or non-Quantified base)"
                        ));
                    }
                }
                SpreadBaseStaticVariant::Wrap => {
                    // The runtime wraps ONE value — no per-item dispatch, no
                    // TT concern; emittable for spread AND flatten (outside an
                    // array a flatten degenerates to spread; inside one, the
                    // wrap arm pushes a single node whose fold is the value).
                }
                SpreadBaseStaticVariant::Undecidable => {
                    reasons.push(format!(
                        "{label} base ${index} runtime content variant undecidable (rule-reference or dynamic `?` shape)"
                    ));
                }
            }
        }
        U::ArrayAccess { .. } => {
            reasons.push("array access is outside the v1 value vocabulary".to_string());
        }
        U::QuantifiedExtraction { .. } => {
            reasons.push("quantified extraction is outside the v1 value vocabulary".to_string());
        }
    }
}

/// Compute the `.5.j.2` direct-value build plan on top of the CyclicSpine
/// cascade plan (ONE implementation for report and emission; session #150
/// CORRECTED partition — 3-way fold classes, per-reference demand, vocabulary
/// demotions).
pub fn compute_direct_value_build_plan(
    tree: &HashMap<String, ASTNode>,
    annotations: Option<&Annotations>,
    entry_rule: Option<&str>,
) -> Result<DirectValueBuildPlan, String> {
    let plan_b = compute_cascade_emission_plan_for_increment(
        tree,
        annotations,
        entry_rule,
        CascadeIncrement::CyclicSpine,
    )?;
    let fused: std::collections::BTreeSet<&str> = plan_b
        .sub_roots
        .iter()
        .chain(plan_b.internal.iter())
        .map(String::as_str)
        .collect();

    // 1. Per-rule classification: VALUE-PURE on every branch ⇒ barrier
    //    candidate; anything else propagates/receives node-form demand.
    let mut barrier_pure: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    for &rule in &fused {
        let Some(body) = tree.get(rule) else { continue };
        let all_pure = if rule_has_matched_text_transform(rule, annotations) {
            true
        } else {
            rule_branch_bodies(body)
                .iter()
                .enumerate()
                .all(|(idx, branch_body)| {
                    resolved_branch_return_ast(rule, idx, branch_body, annotations)
                        .as_ref()
                        .map(return_ast_fold_class)
                        == Some(TransformFoldClass::ValuePure)
                })
        };
        if all_pure {
            barrier_pure.insert(rule);
        }
    }

    // 2. The vocabulary audit (rule-level demotion; demoted rules emit
    //    VERBATIM, so they consume — and therefore demand — every fused
    //    reference of every branch).
    let mut demoted: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for &rule in &fused {
        let Some(body) = tree.get(rule) else { continue };
        let mut reasons: Vec<String> = Vec::new();
        if rule_has_matched_text_transform(rule, annotations) {
            reasons.push("rule-level matched-text @transform (span-transform tail)".to_string());
        }
        // Audit EVERY context the annotations apply in — the top-level
        // branches AND every nested `Or` site (the branch-broadcast
        // semantics; session-#150 ebnf `grammar_file` fix).
        for (idx, context_body) in collect_annotation_contexts(body) {
            let resolved = resolved_branch_return_ast(rule, idx, context_body, annotations);
            let mut branch_reasons: Vec<String> = Vec::new();
            branch_value_vocabulary_reasons(
                resolved.as_ref(),
                context_body,
                &fused,
                &mut branch_reasons,
            );
            for reason in branch_reasons {
                reasons.push(format!("branch {}: {}", idx + 1, reason));
            }
        }
        reasons.dedup();
        if !reasons.is_empty() {
            demoted.insert(rule.to_string(), reasons);
        }
    }

    // 3. Per-reference node-form demand. Seeds: the fused sub-roots (escape
    //    roots). Sources: demoted rules (verbatim — always, demanded or not,
    //    since their bodies still build node-form children when called) and
    //    demanded non-barrier rules (per-branch content-position targets).
    let mut regex_pattern_sink: Vec<String> = Vec::new();
    let mut demanded: std::collections::BTreeSet<&str> = plan_b
        .sub_roots
        .iter()
        .map(String::as_str)
        .filter(|rule| fused.contains(rule))
        .collect();
    loop {
        let mut changed = false;
        for &rule in &fused {
            let is_demoted = demoted.contains_key(rule);
            let is_source =
                is_demoted || (demanded.contains(rule) && !barrier_pure.contains(rule));
            if !is_source {
                continue;
            }
            let Some(body) = tree.get(rule) else { continue };
            let mut target_nodes: Vec<&ASTNode> = Vec::new();
            if is_demoted {
                // A demoted rule is verbatim on every branch — its whole body
                // is consumed in node form.
                target_nodes.push(body);
            }
            // Per-context (top-level + nested `Or` sites — the same contexts
            // the emitter applies annotations in).
            for (idx, branch_body) in collect_annotation_contexts(body) {
                if is_demoted {
                    break;
                }
                let resolved = resolved_branch_return_ast(rule, idx, branch_body, annotations);
                match resolved.as_ref().map(return_ast_fold_class) {
                    Some(TransformFoldClass::ValuePure) => {}
                    Some(TransformFoldClass::Transparent) => {
                        use crate::ast_pipeline::unified_return_ast::UnifiedReturnAST as U;
                        match resolved.as_ref() {
                            Some(U::PositionalRef { index }) => {
                                match resolve_positional_target(branch_body, *index) {
                                    PositionalTarget::Element(element) => {
                                        target_nodes.push(element)
                                    }
                                    PositionalTarget::WholeBody => target_nodes.push(branch_body),
                                    PositionalTarget::StaticSentinel => {}
                                }
                            }
                            _ => target_nodes.push(branch_body),
                        }
                    }
                    Some(TransformFoldClass::ContentCarrying) => {
                        // v1: a carrying branch of a DEMANDED rule is emitted
                        // VERBATIM (whole-body node build feeding today's
                        // transform code), so the demand must cover the WHOLE
                        // branch body — not only the `$N`-targeted elements.
                        // (Per-element in-place input feeding for carrying
                        // folds is the STEP-2b refinement; when it lands, this
                        // arm narrows to the targeted elements again.)
                        target_nodes.push(branch_body);
                    }
                    None => target_nodes.push(branch_body),
                }
            }
            for target in target_nodes {
                let mut refs = HashSet::new();
                collect_refs(target, &mut refs, &mut regex_pattern_sink);
                for referenced in refs {
                    if let Some(&fused_target) = fused.get(referenced.as_str()) {
                        if demanded.insert(fused_target) {
                            changed = true;
                        }
                    }
                }
            }
        }
        if !changed {
            break;
        }
    }

    // 4. The partition. Vocabulary demotions land in node_locked whatever
    //    their class; a demanded barrier stays barrier (its VALUE-PURE content
    //    is byte-identical either way, so demand costs it nothing).
    let mut barrier = std::collections::BTreeSet::new();
    let mut value_licensed = std::collections::BTreeSet::new();
    let mut node_locked = std::collections::BTreeSet::new();
    for &rule in &fused {
        if demoted.contains_key(rule) {
            node_locked.insert(rule.to_string());
        } else if barrier_pure.contains(rule) {
            barrier.insert(rule.to_string());
        } else if demanded.contains(rule) {
            node_locked.insert(rule.to_string());
        } else {
            value_licensed.insert(rule.to_string());
        }
    }

    Ok(DirectValueBuildPlan {
        barrier,
        value_licensed,
        node_locked,
        demoted,
    })
}

/// RGX-0078.5.i.9 (D3) — the VALUE class of one boundary-scanner plan rule: how the
/// emitted `scan_<rule>` reproduces the committed value the protocol body folds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScannerValueClass {
    /// Census text-folding (`FusibleToken`/`FusibleLookahead`): the value is exactly
    /// the matched text — `ParseContent::Terminal(&input[start..end])`, zero-copy.
    Text,
    /// A rule-level matched-text `@transform` over a NON-single-terminal body (the
    /// span-fallback path): `ParseContent::TransformedTerminal(span.trim().parse::<T>()
    /// .unwrap_or(D).to_string())` — the generator's own span-transform emission reused.
    SpanTransform,
    /// A single-branch static-key `-> {…}` object template over value-reproducible
    /// elements (quoted terminals / plan-rule references): the raw element `Sequence`
    /// is built exactly as the protocol does (minus frames) and the generator's own
    /// template emission is reused over it.
    ShapedObject,
}

/// One boundary-scanner plan rule (see [`compute_boundary_scanner_plan`]).
#[derive(Debug, Clone, serde::Serialize)]
pub struct BoundaryScannerRule {
    pub class: ScannerValueClass,
    /// PLAN-B sub-root (`true`) vs residual ineligible rule (`false`) — report flavor.
    pub sub_root: bool,
    /// The rule carries read-only post-phase `@predicate`s: the emitted scan evaluates
    /// them via the same content-aware machinery after the value fold (fresh on every
    /// entry — ≥ the epoch-validated memo-replay guarantee).
    pub post_predicates: bool,
}

/// RGX-0078.5.i.9 (D3) — the boundary-scanner emission plan: the SHARED census gate
/// (`docs/tasks/RGX-0078.md` `.5.i.8` §5.d.1) consumed by BOTH the census report and
/// the codegen scan emitter, so the two cannot drift (the cascade-plan precedent).
#[derive(Debug, Clone, serde::Serialize)]
pub struct BoundaryScannerPlan {
    /// Qualified rules, deterministic order.
    pub rules: BTreeMap<String, BoundaryScannerRule>,
    /// CANDIDATE rules (PLAN-B boundaries) that failed a gate, with the NAMED
    /// reasons — the report's honest residue (never silently dropped).
    pub dropped: BTreeMap<String, Vec<String>>,
}

/// RGX-0078.5.i.9 (D3) — the audit walker behind the boundary-scanner plan: is this
/// rule's body (transitively) inside the scan-emitter vocabulary, with every interior
/// reference either a char builtin (`builtin_any_char`/`builtin_ascii_char` — the
/// frameless engine methods, called directly) or an inlinable effect-free rule
/// (emitted as a per-rule recognizer helper carrying the rule-entry furthest update
/// the bare-path graph performs today)? Every failure is a NAMED reason
/// (conservative under-approximation — the census doctrine). The codegen scan
/// emitter (`ast_based_generator/scan.rs`) consumes the SAME plan and mirrors this
/// vocabulary one-for-one, bailing loudly on anything outside it.
pub(crate) struct ScanAudit<'a> {
    tree: &'a HashMap<String, ASTNode>,
    annotations: Option<&'a Annotations>,
    compiled: Option<&'a CompiledSemanticRuntimeAnnotations>,
    /// Memoized per-rule interior verdicts (`Ok` / named reason).
    verdicts: HashMap<String, Result<(), String>>,
    visiting: HashSet<String>,
}

impl<'a> ScanAudit<'a> {
    /// The literal-token atom types the mtb/protocol emitters compile to
    /// `match_string` — the scan vocabulary mirrors the same set.
    const LITERAL_TOKEN_TYPES: &'static [&'static str] = &[
        "quoted_string",
        "number",
        "probability",
        "include_dir",
        "include_file",
        "rule",
    ];

    fn interior_ref_ok(&mut self, target: &str) -> Result<(), String> {
        if !self.tree.contains_key(target) {
            return match target {
                "builtin_any_char" | "builtin_ascii_char" => Ok(()),
                other => Err(format!("unresolved reference '{other}'")),
            };
        }
        self.audit_interior_rule(target)
    }

    /// An INTERIOR rule (referenced from inside a scan body in recognizer position):
    /// its acceptance semantics must be pure control flow — value-only directives
    /// (a matched-text `@transform`, whose result is discarded in recognizer
    /// position) are the only residue allowed.
    fn audit_interior_rule(&mut self, rule: &str) -> Result<(), String> {
        if let Some(v) = self.verdicts.get(rule) {
            return v.clone();
        }
        if self.visiting.contains(rule) {
            // Witness frame only (the classifier precedent): the outer frame owns
            // and memoizes the rule's real verdict.
            return Err(format!("reference cycle through '{rule}'"));
        }
        self.visiting.insert(rule.to_string());
        let verdict = self.audit_interior_rule_uncached(rule);
        self.visiting.remove(rule);
        self.verdicts.insert(rule.to_string(), verdict.clone());
        verdict
    }

    fn audit_interior_rule_uncached(&mut self, rule: &str) -> Result<(), String> {
        // Acceptance-relevant directives: NONE allowed on interior rules (a store
        // gate would change accept/reject; an effect would need C3-B machinery).
        if let Some(compiled) = self.compiled {
            if !compiled.directives_for_rule(rule).is_empty()
                || compiled
                    .branch_directives_for_rule(rule)
                    .iter()
                    .any(|b| !b.is_empty())
            {
                return Err(format!("interior rule '{rule}' carries runtime directives"));
            }
        }
        self.audit_rule_policies(rule)?;
        if let Some(ann) = self.annotations {
            for name in rule_level_directive_names(ann, rule) {
                match name.as_str() {
                    // Value-only: discarded in recognizer position.
                    "transform" => {}
                    "profiles" => {
                        return Err(format!("interior rule '{rule}' has a @profiles gate"))
                    }
                    _ => {}
                }
            }
        }
        let body = self
            .tree
            .get(rule)
            .ok_or_else(|| format!("rule '{rule}' not defined"))?;
        self.audit_node(body, rule)
    }

    /// The codegen-time rule policies that alter control flow or observable parser
    /// state — the same knob list the cascade gate audits (`cascade_rule_verdict`).
    fn audit_rule_policies(&self, rule: &str) -> Result<(), String> {
        if let Some(ann) = self.annotations {
            if ann
                .branch_mid_sequence_semantic_annotations
                .get(rule)
                .is_some_and(|branches| branches.iter().any(|b| !b.is_empty()))
            {
                return Err(format!("rule '{rule}' has mid-sequence inline directives"));
            }
            if ann.lexical_follow_restrictions.contains_key(rule) {
                return Err(format!("rule '{rule}' has a lexical follow restriction"));
            }
        }
        if !effective_rule_value_constraints(self.annotations, rule).is_empty() {
            return Err(format!("rule '{rule}' has value constraints"));
        }
        if effective_rule_associativity(self.annotations, rule) == SemanticAssociativity::NonAssoc {
            return Err(format!("rule '{rule}' is @associativity nonassoc"));
        }
        if effective_rule_deterministic_partition_policy(self.annotations, rule).enabled {
            return Err(format!("rule '{rule}' has @deterministic_group rotation"));
        }
        if super::semantic_directive_registry::effective_rule_bool_directive(
            self.annotations,
            rule,
            &[
                "stop_at_rule_boundary",
                "stop_on_rule_boundary",
                "line_delimited_sequence",
            ],
        ) {
            return Err(format!("rule '{rule}' has a quantifier break policy"));
        }
        if super::semantic_directive_registry::effective_rule_recovery_enabled(
            self.annotations,
            rule,
        ) {
            return Err(format!("rule '{rule}' has @recover"));
        }
        if super::semantic_directive_registry::effective_rule_coverage_target_weight(
            self.annotations,
            rule,
        ) != 0
        {
            return Err(format!("rule '{rule}' records @coverage_target events"));
        }
        if super::semantic_directive_registry::effective_rule_negative_case_enabled(
            self.annotations,
            rule,
        ) {
            return Err(format!("rule '{rule}' records @invalid_case failures"));
        }
        Ok(())
    }

    fn audit_node(&mut self, node: &ASTNode, rule: &str) -> Result<(), String> {
        match node {
            ASTNode::Or { alternatives } => {
                for alt in alternatives {
                    self.audit_node(alt, rule)?;
                }
                Ok(())
            }
            ASTNode::Sequence { elements } => {
                for el in elements {
                    self.audit_node(el, rule)?;
                }
                Ok(())
            }
            ASTNode::Quantified {
                element,
                quantifier,
            } => {
                if super::parse_quantifier_bounds(quantifier).is_none() {
                    return Err(format!("rule '{rule}': unknown quantifier '{quantifier}'"));
                }
                self.audit_node(element, rule)
            }
            ASTNode::Lookahead { element, .. } => self.audit_node(element, rule),
            ASTNode::Atom { value } => match value {
                ASTValue::Token(parts) if parts.len() >= 2 => {
                    let TokenValue::String(token_type) = &parts[0];
                    let TokenValue::String(token_value) = &parts[1];
                    if Self::LITERAL_TOKEN_TYPES.contains(&token_type.as_str())
                        || token_type == "regex"
                    {
                        return Ok(());
                    }
                    if token_type == "rule_reference" {
                        self.interior_ref_ok(token_value)?;
                        return Ok(());
                    }
                    Err(format!(
                        "rule '{rule}': atom token type '{token_type}' outside the scan vocabulary"
                    ))
                }
                other => Err(format!(
                    "rule '{rule}': atom shape {other:?} outside the scan vocabulary"
                )),
            },
        }
    }
}

/// RGX-0078.5.i.9 (D3) — compute the boundary-scanner emission plan (the `.5.i.8`
/// §5.d.1 SHARED census gate). A rule qualifies iff:
///
/// 1. it is a PLAN-B protocol boundary (a CyclicSpine sub-root, or an ineligible
///    residual rule) and not the grammar entry;
/// 2. its OWN directive residue is within {rule-level matched-text `@transform`,
///    read-only post-phase `@predicate`} — every scope/fact effect, library
///    directive, pre/final/branch-phase predicate, value constraint, follow
///    restriction, and observability policy (`@coverage_target`/`@invalid_case`/
///    `@recover`/quantifier-break) disqualifies;
/// 3. its body (transitively) stays inside the scan-emitter vocabulary with every
///    interior reference a char builtin or an inlinable effect-free rule
///    ([`ScanAudit`]) — in particular the closure is acyclic (memo-loss soundness:
///    a lost hit re-runs an O(k) token-bounded scan);
/// 4. its committed VALUE is reproducible from spans + nested plan values
///    ([`ScannerValueClass`]).
///
/// Every dropped CANDIDATE carries its named reasons (`dropped`) — conservative
/// under-approximation, never a silent narrowing.
pub fn compute_boundary_scanner_plan(
    tree: &HashMap<String, ASTNode>,
    annotations: Option<&Annotations>,
    entry_rule: Option<&str>,
) -> Result<BoundaryScannerPlan, String> {
    let cascade_plan = compute_cascade_emission_plan_for_increment(
        tree,
        annotations,
        entry_rule,
        CascadeIncrement::CyclicSpine,
    )?;
    let mut classifier = Classifier::new(tree, annotations)?;
    // A second compile of the SAME table the classifier holds (the classifier's copy
    // stays private to its own verdicts; both come from the one shared resolution
    // codegen burns in, so they cannot disagree).
    let compiled = match annotations {
        Some(ann) => Some(
            compile_semantic_runtime_annotations(ann)
                .map_err(|e| format!("annotations failed to compile: {e}"))?,
        ),
        None => None,
    };
    let mut audit = ScanAudit {
        tree,
        annotations,
        compiled: compiled.as_ref(),
        verdicts: HashMap::new(),
        visiting: HashSet::new(),
    };

    // Candidates = the PLAN-B protocol boundaries: sub-roots (fused rules entered
    // through their full-frame methods) + residual ineligible rules.
    let mut candidates: Vec<(String, bool)> = Vec::new();
    for rule in tree.keys() {
        if Some(rule.as_str()) == entry_rule {
            continue;
        }
        if cascade_plan.sub_roots.contains(rule) {
            candidates.push((rule.clone(), true));
        } else if !cascade_plan.internal.contains(rule) {
            candidates.push((rule.clone(), false));
        }
    }
    candidates.sort();

    let mut rules: BTreeMap<String, BoundaryScannerRule> = BTreeMap::new();
    let mut dropped: BTreeMap<String, Vec<String>> = BTreeMap::new();

    // PASS 1 — Text and SpanTransform classes (no cross-plan dependency).
    for (rule, sub_root) in &candidates {
        let (branch_count, or_rooted) = match tree.get(rule.as_str()) {
            Some(ASTNode::Or { alternatives }) => (alternatives.len(), true),
            _ => (1, false),
        };
        let mut reasons: Vec<String> = Vec::new();
        let directive_audit = scanner_rule_directive_audit(
            annotations,
            compiled.as_ref(),
            rule,
            branch_count,
            or_rooted,
            &mut reasons,
        );
        let Some(directive_audit) = directive_audit else {
            dropped.insert(rule.clone(), reasons);
            continue;
        };
        if let Err(reason) = audit.audit_rule_policies(rule) {
            dropped.insert(rule.clone(), vec![reason]);
            continue;
        }
        let body = &tree[rule.as_str()];
        if let Err(reason) = audit.audit_node(body, rule) {
            dropped.insert(rule.clone(), vec![reason]);
            continue;
        }

        if directive_audit.matched_text_transform {
            // SpanTransform: the span-fallback path only — the generator splices
            // `generate_post_body_span_transform` for NON-`Or` roots (Or roots
            // apply transforms per-branch), and a single REGEX-terminal body takes
            // the atom-path transform instead (a different emission over the token
            // text, not the trimmed span) — both dropped, mirroring the emitter.
            if rule_has_return_annotations(annotations, rule) {
                dropped.insert(
                    rule.clone(),
                    vec!["matched-text @transform combined with return annotations".to_string()],
                );
                continue;
            }
            if matches!(body, ASTNode::Or { .. }) {
                dropped.insert(
                    rule.clone(),
                    vec!["Or-rooted @transform body (per-branch transform emission)".to_string()],
                );
                continue;
            }
            let is_regex_terminal_atom = matches!(
                body,
                ASTNode::Atom {
                    value: ASTValue::Token(parts)
                } if parts.len() >= 2 && {
                    let TokenValue::String(token_type) = &parts[0];
                    token_type == "regex"
                }
            );
            if is_regex_terminal_atom {
                dropped.insert(
                    rule.clone(),
                    vec!["single regex-terminal @transform body (atom-path emission)".to_string()],
                );
                continue;
            }
            if !scanner_transform_is_emittable(annotations, rule) {
                dropped.insert(
                    rule.clone(),
                    vec!["@transform expression not canonical/emittable".to_string()],
                );
                continue;
            }
            rules.insert(
                rule.clone(),
                BoundaryScannerRule {
                    class: ScannerValueClass::SpanTransform,
                    sub_root: *sub_root,
                    post_predicates: directive_audit.post_predicates,
                },
            );
            continue;
        }

        // Text: the census's own text-folding verdict (tier != NotFusible implies a
        // directive-free closure whose value is exactly the matched text) — WITH the
        // layout-skipping span/token distinction the SV equivalence divergence
        // exposed: `Terminal(&input[start..end])` equals the protocol fold either
        // when EVERY branch is an explicit `-> $text` (the protocol folds the whole
        // span, trivia included) or when NO atom in the closure skips leading layout
        // (span == token text by construction). A token-fold rule over a skipping
        // closure would embed skipped trivia the protocol's `Terminal(matched_str)`
        // excludes — dropped, named.
        let outcome = classifier.classify_rule(rule);
        if outcome.fusible {
            if directive_audit.post_predicates {
                dropped.insert(
                    rule.clone(),
                    vec!["text-fold rule with predicates not classed (v1)".to_string()],
                );
                continue;
            }
            let span_fold = annotations
                .and_then(|a| a.branch_return_annotations.get(rule))
                .is_some_and(|branches| {
                    !branches.is_empty()
                        && branches.iter().all(|b| {
                            matches!(
                                b.as_ref().and_then(|a| a.parsed_ast.as_ref()),
                                Some(UnifiedReturnAST::MatchedText)
                            )
                        })
                });
            if !span_fold && outcome.skipping_atom {
                dropped.insert(
                    rule.clone(),
                    vec!["layout-skipping token fold (span ≠ token text)".to_string()],
                );
                continue;
            }
            rules.insert(
                rule.clone(),
                BoundaryScannerRule {
                    class: ScannerValueClass::Text,
                    sub_root: *sub_root,
                    post_predicates: false,
                },
            );
            continue;
        }
        // Neither class matched in pass 1: defer to pass 2 (ShapedObject) — record
        // the text-fold reasons only if pass 2 also fails.
    }

    // PASS 2 — ShapedObject (element references must resolve to PASS-1 plan rules).
    for (rule, sub_root) in &candidates {
        if rules.contains_key(rule) || dropped.contains_key(rule) {
            continue;
        }
        let (branch_count, or_rooted) = match tree.get(rule.as_str()) {
            Some(ASTNode::Or { alternatives }) => (alternatives.len(), true),
            _ => (1, false),
        };
        let mut reasons: Vec<String> = Vec::new();
        let directive_audit = scanner_rule_directive_audit(
            annotations,
            compiled.as_ref(),
            rule,
            branch_count,
            or_rooted,
            &mut reasons,
        );
        let Some(directive_audit) = directive_audit else {
            dropped.insert(rule.clone(), reasons);
            continue;
        };
        match shaped_object_class_verdict(tree, annotations, rule, &rules) {
            Ok(()) => {
                rules.insert(
                    rule.clone(),
                    BoundaryScannerRule {
                        class: ScannerValueClass::ShapedObject,
                        sub_root: *sub_root,
                        post_predicates: directive_audit.post_predicates,
                    },
                );
            }
            Err(reason) => {
                dropped.insert(rule.clone(), vec![reason]);
            }
        }
    }

    Ok(BoundaryScannerPlan { rules, dropped })
}

/// The per-rule directive audit behind the plan (gate 2 above). Returns `None` and
/// pushes reasons on any disqualifying directive; otherwise reports the allowed
/// residue found.
struct ScannerDirectiveAudit {
    matched_text_transform: bool,
    post_predicates: bool,
}

fn scanner_rule_directive_audit(
    annotations: Option<&Annotations>,
    compiled: Option<&CompiledSemanticRuntimeAnnotations>,
    rule: &str,
    branch_count: usize,
    or_rooted: bool,
    reasons: &mut Vec<String>,
) -> Option<ScannerDirectiveAudit> {
    let mut post_predicates = false;
    if let Some(compiled) = compiled {
        for directive in compiled.directives_for_rule(rule) {
            match directive {
                SemanticRuntimeDirective::Predicate(_) => {}
                other => reasons.push(format!(
                    "runtime directive @{} (not scan-eligible)",
                    directive_kind_name(other)
                )),
            }
        }
        if compiled.pre_predicates_for_rule(rule).next().is_some() {
            reasons.push("pre-phase @predicate (scan tails are post-match)".to_string());
        }
        if compiled.final_predicates_for_rule(rule).next().is_some() {
            reasons.push("final-phase @predicate (enqueues deferred obligations)".to_string());
        }
        post_predicates = compiled.post_predicates_for_rule(rule).next().is_some();
        let has_branch_predicates = compiled.branch_predicates_for_rule(rule).next().is_some()
            || (0..branch_count).any(|i| {
                compiled
                    .branch_predicates_for_rule_branch(rule, i)
                    .next()
                    .is_some()
            });
        if has_branch_predicates {
            reasons.push("branch-phase @predicate".to_string());
        }
        if (0..branch_count).any(|i| {
            compiled
                .branch_effect_directives_for_rule_branch(rule, i)
                .next()
                .is_some()
        }) {
            reasons.push("branch-start effect directive".to_string());
        }
        // The RAW-content capture mirror: on the non-`Or` path the EMITTER captures
        // raw only for a POSITIONAL raw-view post predicate (the RAWCAP-TRANSFORM-
        // PATH.2 narrow gate) — named refs resolve against the shaped content, which
        // is exactly what a scan tail passes for both views. A positional raw-view
        // predicate (no shipped grammar has one) or an `Or`-rooted predicated rule
        // (that path captures raw per-branch regardless) cannot be mirrored — drop.
        if post_predicates && compiled.needs_positional_raw_post_capture_for_rule(rule) {
            reasons.push("positional raw-view post @predicate (raw capture)".to_string());
        }
        if post_predicates && or_rooted {
            reasons.push("Or-rooted rule with post @predicate (per-branch raw capture)".to_string());
        }
    }
    let mut matched_text_transform = false;
    if let Some(ann) = annotations {
        for name in rule_level_directive_names(ann, rule) {
            match name.as_str() {
                "transform" => matched_text_transform = true,
                "profiles" => reasons.push("@profiles dialect gate".to_string()),
                _ => {}
            }
        }
    }
    if reasons.is_empty() {
        Some(ScannerDirectiveAudit {
            matched_text_transform,
            post_predicates,
        })
    } else {
        None
    }
}

/// Does the rule carry any (rule- or branch-level) return annotation?
fn rule_has_return_annotations(annotations: Option<&Annotations>, rule: &str) -> bool {
    annotations
        .and_then(|a| a.branch_return_annotations.get(rule))
        .is_some_and(|branches| branches.iter().any(|b| b.is_some()))
}

/// Mirror of the generator's span-fallback emission condition
/// (`generate_post_body_span_transform`): the `@transform` expression must be
/// canonical AND its target type / default expression must be syn-parseable —
/// otherwise the protocol body would NOT rebind to a `TransformedTerminal` and the
/// scan value would diverge.
fn scanner_transform_is_emittable(annotations: Option<&Annotations>, rule: &str) -> bool {
    let Some(ann) = annotations else {
        return false;
    };
    let Some(entries) = ann.semantic_annotations.get(rule) else {
        return false;
    };
    for annotation in entries {
        let Some((name, _)) =
            super::semantic_directive_registry::semantic_directive_name_payload(annotation)
        else {
            continue;
        };
        if name != "transform" {
            continue;
        }
        if let super::UnifiedSemanticAST::TransformExpr { expression } = annotation.ast() {
            if let Some(transform) =
                super::semantic_transform::parse_canonical_transform_expression(expression)
            {
                return syn::parse_str::<syn::Type>(&transform.target_type).is_ok()
                    && syn::parse_str::<syn::Expr>(&transform.default_expr).is_ok();
            }
        }
        return false;
    }
    false
}

/// The ShapedObject class gate (pass 2): a single-branch rule whose return
/// annotation is a static-key object template over literal values and positional
/// refs, and whose body is a `Sequence` of quoted terminals and references to
/// PASS-1 plan rules (each element buildable in value mode exactly as the protocol
/// builds it, minus frames).
fn shaped_object_class_verdict(
    tree: &HashMap<String, ASTNode>,
    annotations: Option<&Annotations>,
    rule: &str,
    pass1_rules: &BTreeMap<String, BoundaryScannerRule>,
) -> Result<(), String> {
    let Some(ann) = annotations else {
        return Err("no annotations (no object template)".to_string());
    };
    let branches = ann
        .branch_return_annotations
        .get(rule)
        .ok_or_else(|| "no return annotation (and not text-folding)".to_string())?;
    let body = tree
        .get(rule)
        .ok_or_else(|| format!("rule '{rule}' not defined"))?;
    if matches!(body, ASTNode::Or { .. }) {
        return Err("Or-rooted body (single-branch templates only, v1)".to_string());
    }
    if branches.len() != 1 {
        return Err(format!(
            "expected exactly 1 branch annotation, found {}",
            branches.len()
        ));
    }
    let Some(annotation) = branches[0].as_ref() else {
        return Err("no return annotation (and not text-folding)".to_string());
    };
    let Some(UnifiedReturnAST::Object { properties }) = annotation.parsed_ast.as_ref() else {
        return Err("return annotation is not an object template".to_string());
    };
    let ASTNode::Sequence { elements } = body else {
        return Err("body is not a Sequence (object templates index elements)".to_string());
    };
    for (key, value) in properties {
        match value.as_ref() {
            UnifiedReturnAST::StringLiteral { .. }
            | UnifiedReturnAST::NumberLiteral { .. }
            | UnifiedReturnAST::BooleanLiteral { .. }
            | UnifiedReturnAST::NullLiteral => {}
            UnifiedReturnAST::PositionalRef { index } => {
                if *index == 0 || *index > elements.len() {
                    return Err(format!(
                        "template key '{key}' positional ref {index} outside the {}-element body",
                        elements.len()
                    ));
                }
            }
            other => {
                return Err(format!(
                    "template key '{key}' value {other:?} outside the v1 template vocabulary"
                ))
            }
        }
    }
    for (idx, element) in elements.iter().enumerate() {
        match element {
            ASTNode::Atom { value: ASTValue::Token(parts) } if parts.len() >= 2 => {
                let TokenValue::String(token_type) = &parts[0];
                let TokenValue::String(token_value) = &parts[1];
                if ScanAudit::LITERAL_TOKEN_TYPES.contains(&token_type.as_str()) {
                    continue;
                }
                if token_type == "rule_reference" {
                    if pass1_rules.contains_key(token_value) {
                        continue;
                    }
                    return Err(format!(
                        "element {} references '{token_value}', not a pass-1 plan rule",
                        idx + 1
                    ));
                }
                return Err(format!(
                    "element {} token type '{token_type}' outside the value-mode vocabulary",
                    idx + 1
                ));
            }
            other => {
                return Err(format!(
                    "element {} shape {other:?} outside the value-mode vocabulary",
                    idx + 1
                ))
            }
        }
    }
    Ok(())
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
    // RGX-0078.5.j.4 (K4b C1) — the FIRSTₖ bounded prefix-trie guard surface.
    let firstk_rows: Vec<(&ChoiceSiteCensus, &FirstkBranchCensus)> = top_level_sites
        .iter()
        .flat_map(|s| s.firstk_branches.iter().map(move |b| (*s, b)))
        .collect();
    let guarded: Vec<&(&ChoiceSiteCensus, &FirstkBranchCensus)> = firstk_rows
        .iter()
        .filter(|(_, b)| b.verdict == "guarded")
        .collect();
    let mut depth_histogram: BTreeMap<usize, usize> = BTreeMap::new();
    for (_, b) in &guarded {
        *depth_histogram.entry(b.max_depth).or_default() += 1;
    }
    let deep: Vec<&(&ChoiceSiteCensus, &FirstkBranchCensus)> = guarded
        .iter()
        .copied()
        .filter(|(_, b)| !b.level1_degenerate)
        .collect();
    let with_emulation = deep
        .iter()
        .filter(|(_, b)| !b.emulation_offsets.is_empty())
        .count();
    let truncated = guarded.iter().filter(|(_, b)| b.truncated).count();
    let d1_fallback = guarded.iter().filter(|(_, b)| b.d1_fallback_used).count();
    println!(
        "FIRSTK-CENSUS: grammar={} top_level_branches={} guarded={} deeper_than_level1={} with_emulation={} truncated={} d1_fallback={}",
        census.grammar_name,
        firstk_rows.len(),
        guarded.len(),
        deep.len(),
        with_emulation,
        truncated,
        d1_fallback,
    );
    println!(
        "  gate: level-1 admission (branch_dispatch_first_bytes) + per-path bounded trie (depth≤4, fanout≤24, ≤16 nodes) + exact furthest emulation on refutation arms — RGX-0078.5.j.4 (C1)"
    );
    if !depth_histogram.is_empty() {
        let ranked: Vec<String> = depth_histogram
            .iter()
            .map(|(d, n)| format!("d{d}={n}"))
            .collect();
        println!("  guarded-branch walk-depth histogram: {}", ranked.join(" "));
    }
    if !deep.is_empty() {
        let mut names: Vec<String> = deep
            .iter()
            .map(|(s, b)| {
                let mut tags = String::new();
                if !b.emulation_offsets.is_empty() {
                    tags.push_str(&format!(
                        ",w{}",
                        b.emulation_offsets
                            .iter()
                            .map(|w| w.to_string())
                            .collect::<Vec<_>>()
                            .join("/")
                    ));
                }
                if b.truncated {
                    tags.push_str(",trunc");
                }
                if b.d1_fallback_used {
                    tags.push_str(",d1");
                }
                format!("{}#b{}(d{}{})", s.rule, b.index, b.max_depth, tags)
            })
            .collect();
        names.sort();
        let shown = names.len().min(60);
        println!(
            "  deep branches (rule#branch(depth[,w-offsets][,trunc][,d1])): {}{}",
            names[..shown].join(" "),
            if names.len() > shown {
                format!(" … +{} more", names.len() - shown)
            } else {
                String::new()
            }
        );
    }
    let mut firstk_refusal_histogram: HashMap<String, usize> = HashMap::new();
    for (_, b) in &firstk_rows {
        if b.verdict == "guarded" {
            continue;
        }
        let key = b
            .verdict
            .split_once(':')
            .map(|(head, _)| head)
            .unwrap_or(b.verdict.as_str());
        *firstk_refusal_histogram.entry(key.to_string()).or_default() += 1;
    }
    if !firstk_refusal_histogram.is_empty() {
        let mut ranked: Vec<(String, usize)> = firstk_refusal_histogram.into_iter().collect();
        ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        println!("  firstk level-1 refusal histogram (unguarded branches):");
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
    // RGX-0078.5.i.7 (D2-B plan seam) — the cyclic-spine increment's plan
    // (compute_cascade_emission_plan_for_increment at CyclicSpine — the map the
    // D2-B emitter will consume).
    {
        let plan = &census.cascade_plan_b;
        let fused_effect_reaching = plan
            .sub_roots
            .iter()
            .chain(plan.internal.iter())
            .filter(|r| plan.effect_reaching.contains(*r))
            .count();
        println!(
            "CASCADE-PLAN-B: grammar={} increment=B(cyclic-spine) sub_roots={} internal={} thin_memo={} effect_reaching_fused={} (of {} fused)",
            census.grammar_name,
            plan.sub_roots.len(),
            plan.internal.len(),
            plan.thin_memo.len(),
            fused_effect_reaching,
            plan.sub_roots.len() + plan.internal.len(),
        );
        println!(
            "  model: EVERY cascade-eligible rule fused (sub-roots = the census's full-fold roots, keeping the protocol frame + twin dispatch); cycle-participating fused rules carry the epoch-stamped thin memo (⛔ the #49 bound — they never lose memo protection); effect obligations as in increment A."
        );
        if dump_all {
            for rule in plan.sub_roots.iter().chain(plan.internal.iter()) {
                println!(
                    "  [cascade-plan-b] {rule}: {}{}{}",
                    if plan.sub_roots.contains(rule) {
                        "SUB-ROOT"
                    } else {
                        "internal"
                    },
                    if plan.thin_memo.contains(rule) {
                        " thin_memo"
                    } else {
                        ""
                    },
                    if plan.effect_reaching.contains(rule) {
                        " effect_reaching"
                    } else {
                        ""
                    }
                );
            }
        }
    }
    // RGX-0078.5.j.2 (STEP-1) — the direct-value build plan (the same SHARED
    // function the value-twin emitter consumes), reported ahead of emission.
    {
        let plan = &census.direct_value_plan;
        println!(
            "DIRECT-VALUE-PLAN: grammar={} barrier={} value_licensed={} node_locked={} (of {} fused; demoted={})",
            census.grammar_name,
            plan.barrier.len(),
            plan.value_licensed.len(),
            plan.node_locked.len(),
            plan.barrier.len() + plan.value_licensed.len() + plan.node_locked.len(),
            plan.demoted.len(),
        );
        println!(
            "  model (session #150 corrected): barrier rules (VALUE-PURE fold on every branch) build value-internally unconditionally; value-licensed rules (undemanded through content-position references) convert early (compositional to_shaped_value); node-locked rules keep node builds (escape root, content-position demand, or a NAMED vocabulary demotion)."
        );
        if dump_all {
            for rule in &plan.barrier {
                println!("  [direct-value-plan] {rule}: barrier");
            }
            for rule in &plan.value_licensed {
                println!("  [direct-value-plan] {rule}: value_licensed");
            }
            for rule in &plan.node_locked {
                println!("  [direct-value-plan] {rule}: node_locked");
            }
            for (rule, reasons) in &plan.demoted {
                println!("  [direct-value-plan-demotion] {rule}: {}", reasons.join("; "));
            }
        }
    }
    // RGX-0078.5.i.9 (D3) — the boundary-scanner plan (the same SHARED function the
    // scan emitter consumes).
    {
        let plan = &census.boundary_scanner_plan;
        let count_class = |class: ScannerValueClass| {
            plan.rules.values().filter(|r| r.class == class).count()
        };
        let sub_roots = plan.rules.values().filter(|r| r.sub_root).count();
        println!(
            "BOUNDARY-SCANNER-PLAN: grammar={} rules={} (text={} span_transform={} shaped_object={}; sub_roots={} residual={}; post_predicates={}) dropped_candidates={}",
            census.grammar_name,
            plan.rules.len(),
            count_class(ScannerValueClass::Text),
            count_class(ScannerValueClass::SpanTransform),
            count_class(ScannerValueClass::ShapedObject),
            sub_roots,
            plan.rules.len() - sub_roots,
            plan.rules.values().filter(|r| r.post_predicates).count(),
            plan.dropped.len(),
        );
        println!(
            "  model: per plan rule a direct-coded frameless scan_<rule> serves BARE-path call sites (protocol twin verbatim; exact furthest emulation; acyclic O(k) memo-loss) — RGX-0078.5.i.9 (D3)"
        );
        if dump_all {
            for (rule, entry) in &plan.rules {
                println!(
                    "  [scanner-plan] {rule}: {:?} {}{}",
                    entry.class,
                    if entry.sub_root { "SUB-ROOT" } else { "residual" },
                    if entry.post_predicates {
                        " post_predicates"
                    } else {
                        ""
                    },
                );
            }
            for (rule, reasons) in &plan.dropped {
                println!("  [scanner-plan] {rule}: DROPPED — {}", reasons.join("; "));
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

    /// RGX-0078.5.j.2 (STEP-1) — a parsed BARRIER branch annotation (an empty
    /// object literal: content variant `Shaped(Object)` regardless of children).
    fn barrier_branch() -> Option<super::super::BranchAnnotation> {
        Some(super::super::BranchAnnotation {
            annotation_type: "return".to_string(),
            annotation_content: "{}".to_string(),
            parsed_ast: Some(
                crate::ast_pipeline::unified_return_ast::UnifiedReturnAST::Object {
                    properties: std::collections::HashMap::new(),
                },
            ),
        })
    }

    /// RGX-0078.5.j.2 (STEP-1) — a fold BARRIER stops node-form demand: under a
    /// barrier entry, the whole transparent chain below is value-licensed; the
    /// barrier rule itself never node-locks (its wrapper carries its own fold
    /// result). Deterministic across recomputation.
    #[test]
    fn direct_value_plan_barrier_stops_demand_and_licenses_the_chain() {
        let mut tree = HashMap::new();
        // entry(-> {…}) := mid ; mid := leaf ; leaf := 'x'
        tree.insert("entry".to_string(), or(vec![rule_ref("mid")]));
        tree.insert("mid".to_string(), or(vec![rule_ref("leaf")]));
        tree.insert("leaf".to_string(), or(vec![atom("quoted_string", "x")]));
        let mut annotations = Annotations::default();
        annotations
            .branch_return_annotations
            .insert("entry".to_string(), vec![barrier_branch()]);
        let plan = compute_direct_value_build_plan(&tree, Some(&annotations), Some("entry"))
            .expect("plan computes");
        assert_eq!(
            plan.barrier,
            ["entry"].iter().map(|s| s.to_string()).collect(),
            "the object-annotated entry is a barrier: {plan:?}"
        );
        assert_eq!(
            plan.value_licensed,
            ["leaf", "mid"].iter().map(|s| s.to_string()).collect(),
            "the transparent chain under a barrier converts early: {plan:?}"
        );
        assert!(plan.node_locked.is_empty(), "{plan:?}");
        let again = compute_direct_value_build_plan(&tree, Some(&annotations), Some("entry"))
            .expect("plan recomputes");
        assert_eq!(format!("{plan:?}"), format!("{again:?}"), "deterministic");
    }

    /// RGX-0078.5.j.2 (STEP-1) — a TRANSPARENT escape root (fused sub-root with
    /// no fold) locks itself and everything reachable through transparent rules:
    /// its content variant is protocol-observable, so today's node build stays.
    #[test]
    fn direct_value_plan_transparent_sub_root_locks_the_transparent_chain() {
        let mut tree = HashMap::new();
        // entry := mid ; mid := leaf ; leaf := 'x' — no annotations anywhere.
        tree.insert("entry".to_string(), or(vec![rule_ref("mid")]));
        tree.insert("mid".to_string(), or(vec![rule_ref("leaf")]));
        tree.insert("leaf".to_string(), or(vec![atom("quoted_string", "x")]));
        let plan = compute_direct_value_build_plan(&tree, None, Some("entry"))
            .expect("plan computes");
        assert!(plan.barrier.is_empty(), "{plan:?}");
        assert!(plan.value_licensed.is_empty(), "{plan:?}");
        assert_eq!(
            plan.node_locked,
            ["entry", "leaf", "mid"].iter().map(|s| s.to_string()).collect(),
            "the transparent sub-root demands node form all the way down: {plan:?}"
        );
    }

    /// RGX-0078.5.j.2 (STEP-1) — an INELIGIBLE caller promotes its fused target
    /// to sub-root (the cascade plan's outside-entered partition), which locks a
    /// transparent target — while a barrier rule referenced the same way stays
    /// value-internal (barrier membership is annotation-decided, not position-
    /// decided). Or-rules: ALL branches must fold for barrier class; one bare
    /// branch makes the rule transparent.
    #[test]
    fn direct_value_plan_sub_root_promotion_locks_transparent_but_not_barrier() {
        let mut tree = HashMap::new();
        // entry := gate ; gate(@transform, ineligible) := mid ; mid := leaf | 'z'
        // leaf(-> {…} | -> {…}) := 'x' | 'y'   (all-branch fold ⇒ barrier)
        tree.insert("entry".to_string(), or(vec![rule_ref("gate")]));
        tree.insert("gate".to_string(), or(vec![rule_ref("mid")]));
        tree.insert(
            "mid".to_string(),
            or(vec![rule_ref("leaf"), atom("quoted_string", "z")]),
        );
        tree.insert(
            "leaf".to_string(),
            or(vec![atom("quoted_string", "x"), atom("quoted_string", "y")]),
        );
        let mut annotations = transform_annotations("gate");
        annotations
            .branch_return_annotations
            .insert("leaf".to_string(), vec![barrier_branch(), barrier_branch()]);
        let plan = compute_direct_value_build_plan(&tree, Some(&annotations), Some("entry"))
            .expect("plan computes");
        // gate is ineligible (not fused) ⇒ absent from every partition set.
        assert!(
            !plan.barrier.contains("gate")
                && !plan.value_licensed.contains("gate")
                && !plan.node_locked.contains("gate"),
            "{plan:?}"
        );
        // mid is outside-entered by the ineligible gate ⇒ fused sub-root; its
        // second branch is bare ⇒ transparent ⇒ node-locked.
        assert!(plan.node_locked.contains("mid"), "{plan:?}");
        // entry: transparent sub-root ⇒ node-locked.
        assert!(plan.node_locked.contains("entry"), "{plan:?}");
        // leaf: every branch folds ⇒ barrier stays value-internal even though
        // its parent mid is node-locked (the wrapper carries the fold result).
        assert_eq!(
            plan.barrier,
            ["leaf"].iter().map(|s| s.to_string()).collect(),
            "{plan:?}"
        );
        assert!(plan.value_licensed.is_empty(), "{plan:?}");
    }

    /// A parsed branch annotation around an arbitrary return AST.
    fn branch_of(
        ast: crate::ast_pipeline::unified_return_ast::UnifiedReturnAST,
    ) -> Option<super::super::BranchAnnotation> {
        Some(super::super::BranchAnnotation {
            annotation_type: "return".to_string(),
            annotation_content: String::new(),
            parsed_ast: Some(ast),
        })
    }

    /// RGX-0078.5.j.2 (session #150 CORRECTION) — a CONTENT-CARRYING fold
    /// (`-> [$1**]`, the regex `concatenation` shape) is NOT a demand stop:
    /// when the carrying rule is demanded, its `$N`-targeted children are
    /// demanded too (their nodes are re-emitted inside its output content).
    /// Under the retired 2-way classifier `list` was a "barrier" and `leaf`
    /// was value-licensed while `list`'s verbatim spread needed `leaf` NODES —
    /// the inconsistency this correction closes.
    #[test]
    fn direct_value_plan_content_carrying_fold_propagates_demand() {
        use crate::ast_pipeline::unified_return_ast::UnifiedReturnAST as U;
        let mut tree = HashMap::new();
        // entry := list ; list := leaf+ -> [$1**] ; leaf := 'x'
        tree.insert("entry".to_string(), or(vec![rule_ref("list")]));
        tree.insert(
            "list".to_string(),
            ASTNode::Quantified {
                element: Box::new(rule_ref("leaf")),
                quantifier: "+".to_string(),
            },
        );
        tree.insert("leaf".to_string(), or(vec![atom("quoted_string", "x")]));
        let mut annotations = Annotations::default();
        annotations.branch_return_annotations.insert(
            "list".to_string(),
            vec![branch_of(U::Array {
                elements: vec![U::FlattenSpread {
                    base: Box::new(U::PositionalRef { index: 1 }),
                }],
            })],
        );
        let plan = compute_direct_value_build_plan(&tree, Some(&annotations), Some("entry"))
            .expect("plan computes");
        assert!(
            plan.node_locked.contains("list") && plan.node_locked.contains("leaf"),
            "a demanded content-carrying fold demands its targeted children: {plan:?}"
        );
        assert!(
            !plan.barrier.contains("list"),
            "a content-carrying fold is never a barrier: {plan:?}"
        );
        assert!(plan.demoted.is_empty(), "in-vocabulary flatten-spread: {plan:?}");
    }

    /// RGX-0078.5.j.2 (session #150 CORRECTION) — demand propagates
    /// PER-REFERENCE: a transparent `-> $2` branch demands exactly element 2's
    /// subtree; the sibling elements' references stay value-licensed.
    #[test]
    fn direct_value_plan_transparent_positional_targets_only_its_element() {
        use crate::ast_pipeline::unified_return_ast::UnifiedReturnAST as U;
        let mut tree = HashMap::new();
        // entry := a b c -> $2 ; a := 'x' ; b := 'y' ; c := 'z'
        tree.insert(
            "entry".to_string(),
            ASTNode::Sequence {
                elements: vec![rule_ref("a"), rule_ref("b"), rule_ref("c")],
            },
        );
        tree.insert("a".to_string(), or(vec![atom("quoted_string", "x")]));
        tree.insert("b".to_string(), or(vec![atom("quoted_string", "y")]));
        tree.insert("c".to_string(), or(vec![atom("quoted_string", "z")]));
        let mut annotations = Annotations::default();
        annotations.branch_return_annotations.insert(
            "entry".to_string(),
            vec![branch_of(U::PositionalRef { index: 2 })],
        );
        let plan = compute_direct_value_build_plan(&tree, Some(&annotations), Some("entry"))
            .expect("plan computes");
        assert!(plan.node_locked.contains("b"), "$2 demands element 2: {plan:?}");
        assert!(
            plan.value_licensed.contains("a") && plan.value_licensed.contains("c"),
            "sibling elements are not demanded by $2: {plan:?}"
        );
    }

    /// RGX-0078.5.j.2 (session #150 CORRECTION) — per-BRANCH precision on a
    /// mixed Or (the regex `piece` shape): the transparent `-> $1` branch
    /// demands its target, the VALUE-PURE object branch demands nothing.
    #[test]
    fn direct_value_plan_mixed_or_propagates_per_branch() {
        use crate::ast_pipeline::unified_return_ast::UnifiedReturnAST as U;
        let mut tree = HashMap::new();
        // entry := piece_like ; piece_like := special -> $1 | other 'q' -> {k:$1}
        tree.insert("entry".to_string(), or(vec![rule_ref("piece_like")]));
        tree.insert(
            "piece_like".to_string(),
            or(vec![
                rule_ref("special"),
                ASTNode::Sequence {
                    elements: vec![rule_ref("other"), atom("quoted_string", "q")],
                },
            ]),
        );
        tree.insert("special".to_string(), or(vec![atom("quoted_string", "x")]));
        tree.insert("other".to_string(), or(vec![atom("quoted_string", "y")]));
        let mut properties = std::collections::HashMap::new();
        properties.insert(
            "k".to_string(),
            Box::new(U::PositionalRef { index: 1 }),
        );
        let mut annotations = Annotations::default();
        annotations.branch_return_annotations.insert(
            "piece_like".to_string(),
            vec![
                branch_of(U::PositionalRef { index: 1 }),
                branch_of(U::Object { properties }),
            ],
        );
        let plan = compute_direct_value_build_plan(&tree, Some(&annotations), Some("entry"))
            .expect("plan computes");
        assert!(
            plan.node_locked.contains("piece_like") && plan.node_locked.contains("special"),
            "the transparent branch demands its target: {plan:?}"
        );
        assert!(
            plan.value_licensed.contains("other"),
            "the VALUE-PURE object branch demands nothing: {plan:?}"
        );
    }

    /// RGX-0078.5.j.2 (session #150 CORRECTION) — an out-of-vocabulary
    /// transform (v1: array access) is DEMOTED to node_locked with a NAMED
    /// reason, and — because its emission is verbatim — it demands its fused
    /// references even when the rule itself is undemanded (its VALUE-PURE
    /// caller folds it).
    #[test]
    fn direct_value_plan_vocabulary_demotion_is_named_and_propagates() {
        use crate::ast_pipeline::unified_return_ast::UnifiedReturnAST as U;
        let mut tree = HashMap::new();
        // entry(-> {v:$1}) := acc ; acc := inner 'z' -> $1[0] ; inner := 'x'
        tree.insert("entry".to_string(), or(vec![rule_ref("acc")]));
        tree.insert(
            "acc".to_string(),
            ASTNode::Sequence {
                elements: vec![rule_ref("inner"), atom("quoted_string", "z")],
            },
        );
        tree.insert("inner".to_string(), or(vec![atom("quoted_string", "x")]));
        let mut properties = std::collections::HashMap::new();
        properties.insert(
            "v".to_string(),
            Box::new(U::PositionalRef { index: 1 }),
        );
        let mut annotations = Annotations::default();
        annotations
            .branch_return_annotations
            .insert("entry".to_string(), vec![branch_of(U::Object { properties })]);
        annotations.branch_return_annotations.insert(
            "acc".to_string(),
            vec![branch_of(U::ArrayAccess {
                base: Box::new(U::PositionalRef { index: 1 }),
                index: Box::new(U::NumberLiteral { value: 0.0 }),
            })],
        );
        let plan = compute_direct_value_build_plan(&tree, Some(&annotations), Some("entry"))
            .expect("plan computes");
        let reasons = plan.demoted.get("acc").expect("acc is demoted");
        assert!(
            reasons.iter().any(|r| r.contains("array access")),
            "the demotion reason is named: {reasons:?}"
        );
        assert!(plan.node_locked.contains("acc"), "{plan:?}");
        assert!(
            plan.node_locked.contains("inner"),
            "a demoted (verbatim) rule demands its fused references even undemanded: {plan:?}"
        );
        assert!(plan.barrier.contains("entry"), "{plan:?}");
    }

    /// RGX-0078.5.j.2 (session #150, the ebnf `grammar_file` fix) — branch
    /// annotations apply at EVERY `Or` site (branch-broadcast), so the
    /// vocabulary audit must check the NESTED contexts too: a spread that is
    /// statically-Splice against the top-level Quantified body is UNDECIDABLE
    /// against a nested rule-reference branch (the `$1`-peel reaches the
    /// child's content, whose variant is a cross-rule class) — the rule is
    /// demoted with a named reason.
    #[test]
    fn direct_value_plan_nested_or_context_demotes_undecidable_spread() {
        use crate::ast_pipeline::unified_return_ast::UnifiedReturnAST as U;
        let mut tree = HashMap::new();
        // entry(-> {v:$1}) := list ; list := (a | b)+ -> [$1*] ; a := 'x' ; b := 'y'
        tree.insert("entry".to_string(), or(vec![rule_ref("list")]));
        tree.insert(
            "list".to_string(),
            ASTNode::Quantified {
                element: Box::new(or(vec![rule_ref("a"), rule_ref("b")])),
                quantifier: "+".to_string(),
            },
        );
        tree.insert("a".to_string(), or(vec![atom("quoted_string", "x")]));
        tree.insert("b".to_string(), or(vec![atom("quoted_string", "y")]));
        let mut properties = std::collections::HashMap::new();
        properties.insert("v".to_string(), Box::new(U::PositionalRef { index: 1 }));
        let mut annotations = Annotations::default();
        annotations
            .branch_return_annotations
            .insert("entry".to_string(), vec![branch_of(U::Object { properties })]);
        annotations.branch_return_annotations.insert(
            "list".to_string(),
            vec![branch_of(U::Array {
                elements: vec![U::Spread {
                    base: Box::new(U::PositionalRef { index: 1 }),
                }],
            })],
        );
        let plan = compute_direct_value_build_plan(&tree, Some(&annotations), Some("entry"))
            .expect("plan computes");
        let reasons = plan.demoted.get("list").expect("list is demoted");
        assert!(
            reasons.iter().any(|r| r.contains("undecidable")),
            "the nested-context demotion reason is named: {reasons:?}"
        );
        assert!(plan.node_locked.contains("list"), "{plan:?}");
        assert!(
            plan.node_locked.contains("a") && plan.node_locked.contains("b"),
            "the demoted rule's references are node-locked: {plan:?}"
        );
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

    /// RGX-0078.5.i.7 (D2-B plan seam) — under the CYCLIC-SPINE increment the
    /// fused-candidate gate drops acyclicity: the cyclic pair fuses as internal
    /// rules carrying the thin-memo obligation (⛔ the #49 bound), the acyclic
    /// chain off the cycle is PROMOTED to internal (its cyclic caller now folds),
    /// and increment A's plan on the same grammar carries an EMPTY thin_memo.
    #[test]
    fn cascade_plan_b_fuses_the_cyclic_spine_with_thin_memo() {
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
        let plan_a =
            compute_cascade_emission_plan(&tree, None, Some("entry")).expect("plan computes");
        assert!(
            plan_a.thin_memo.is_empty(),
            "increment A never fuses a cycle participant: {:?}",
            plan_a.thin_memo
        );
        let plan_b = compute_cascade_emission_plan_for_increment(
            &tree,
            None,
            Some("entry"),
            CascadeIncrement::CyclicSpine,
        )
        .expect("plan computes");
        assert!(plan_b.sub_roots.contains("entry"), "{:?}", plan_b.sub_roots);
        for fused_internal in ["a", "b", "c", "d"] {
            assert!(
                plan_b.internal.contains(fused_internal),
                "{fused_internal} fuses as internal under B (c/d promoted — the cyclic caller folds): {:?}",
                plan_b.internal
            );
        }
        assert_eq!(
            plan_b.thin_memo,
            ["a", "b"].iter().map(|s| s.to_string()).collect(),
            "thin memo = exactly the cycle participants"
        );
        // A's fused set is a strict subset of B's (the increment only widens).
        for rule in plan_a.sub_roots.iter().chain(plan_a.internal.iter()) {
            assert!(
                plan_b.sub_roots.contains(rule) || plan_b.internal.contains(rule),
                "increment B fuses everything A fused: {rule}"
            );
        }
    }

    /// RGX-0078.5.i.7 (D2-B plan seam) — plan B's sub-roots reproduce the census's
    /// OWN full-fold root partition (fused = eligible ⇒ outside-entered = the
    /// census `root` flag), and the census carries the SAME B plan the emitter
    /// will consume (no drift, the `-0086` precedent).
    #[test]
    fn cascade_plan_b_sub_roots_equal_the_census_full_fold_roots() {
        let mut tree = HashMap::new();
        // gate(@transform, ineligible) := x ; x := y | 'z' x (cyclic) ; y := 'w'
        // entry := gate  — so x is entered from an INELIGIBLE caller (a full-fold root).
        tree.insert("entry".to_string(), or(vec![rule_ref("gate")]));
        tree.insert("gate".to_string(), or(vec![rule_ref("x")]));
        tree.insert(
            "x".to_string(),
            or(vec![
                rule_ref("y"),
                ASTNode::Sequence {
                    elements: vec![atom("quoted_string", "z"), rule_ref("x")],
                },
            ]),
        );
        tree.insert("y".to_string(), or(vec![atom("quoted_string", "w")]));
        let ann = transform_annotations("gate");
        let plan_b = compute_cascade_emission_plan_for_increment(
            &tree,
            None,
            Some("entry"),
            CascadeIncrement::CyclicSpine,
        )
        .expect("plan computes");
        // Without annotations everything is eligible; with the @transform gate the
        // census census_of run below carries the SAME partition — assert both layers.
        let census = census_of(
            tree,
            vec![
                "entry".to_string(),
                "gate".to_string(),
                "x".to_string(),
                "y".to_string(),
            ],
            Some(ann),
        );
        let census_roots: std::collections::BTreeSet<String> = census
            .cascade_rules
            .iter()
            .filter(|(_, c)| c.eligible && c.root)
            .map(|(r, _)| r.clone())
            .collect();
        assert_eq!(
            census.cascade_plan_b.sub_roots, census_roots,
            "plan B sub-roots == the census full-fold roots"
        );
        let census_internal: std::collections::BTreeSet<String> = census
            .cascade_rules
            .iter()
            .filter(|(_, c)| c.eligible && !c.root)
            .map(|(r, _)| r.clone())
            .collect();
        assert_eq!(
            census.cascade_plan_b.internal, census_internal,
            "plan B internal == the census full-fold internal rules"
        );
        let census_cyclic_fused: std::collections::BTreeSet<String> = census
            .cascade_rules
            .iter()
            .filter(|(_, c)| c.eligible && c.on_cycle)
            .map(|(r, _)| r.clone())
            .collect();
        assert_eq!(
            census.cascade_plan_b.thin_memo, census_cyclic_fused,
            "plan B thin memo == the census's eligible cycle participants"
        );
        // The annotation-free standalone plan (everything eligible) is a sanity
        // shape: x cyclic-fused with thin memo, entered only by fused callers.
        assert!(plan_b.thin_memo.contains("x"));
        assert!(plan_b.internal.contains("x"));
    }

    /// RGX-0078.5.i.7 (D2-B plan seam) — the effect fixpoint is INCREMENT-
    /// INDEPENDENT (it ranges over all tree rules), so plan A and plan B agree on
    /// `effect_reaching`/`effect_targets`; under B a fused CYCLIC rule that
    /// reaches an ineligible descendant is exactly the design's snapshot/island
    /// carrier (the `atom → python_named_backreference` class): fused + thin-memo
    /// + effect-reaching simultaneously.
    #[test]
    fn cascade_plan_b_effect_fixpoint_matches_increment_a() {
        let mut tree = HashMap::new();
        // p := q ; q := r | 'z' q  (q cyclic eligible) ; r(@transform, ineligible).
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
        let plan_a =
            compute_cascade_emission_plan(&tree, Some(&ann), Some("p")).expect("plan computes");
        let plan_b = compute_cascade_emission_plan_for_increment(
            &tree,
            Some(&ann),
            Some("p"),
            CascadeIncrement::CyclicSpine,
        )
        .expect("plan computes");
        assert_eq!(plan_a.effect_reaching, plan_b.effect_reaching);
        assert_eq!(plan_a.effect_targets, plan_b.effect_targets);
        assert!(
            plan_b.internal.contains("q") && plan_b.thin_memo.contains("q"),
            "q fuses under B with the thin-memo obligation"
        );
        assert!(
            plan_b.effect_reaching.contains("q"),
            "q is the fused-cyclic-and-effect-reaching carrier the design names"
        );
    }
}
