//! `ENGINE-UNIVERSAL-SERVICES.13` slice 5 — the INDIRECT left-recursion **transformation**.
//!
//! Slice 4's survey ([`super::indirect_lr_plan`]) answers *where* a chain can be absorbed, *what*
//! suffix it iterates and *what it costs*. This module turns one of its starvation-safe candidates
//! into an **applicable plan** and applies it, so an indirect cycle is eliminated at GENERATION
//! time — the mechanism acceptance (b) decided
//! ([[project_indirect_left_recursion_is_eliminated_at_generation_not_grown_at_runtime]]).
//!
//! ## The rewrite
//!
//! ```text
//! X            := X_lr_base ( X_lr_suffix )*
//! X_lr_base    := <every alternative of X, in the author's order, with each CYCLIC one
//!                  replaced by a sheared CLONE of the rule it referenced>
//! X_lr_suffix  := <one branch per route: the route's residuals, innermost first>
//! ```
//!
//! plus one clone rule per intermediate on a route. A clone is the intermediate with the
//! cycle-closing edge sheared off, so the seed position can still derive everything the
//! intermediate derived *except* the recursion — which the `( X_lr_suffix )*` now carries.
//!
//! This is exactly the shape slice 3 measured (`p3_eliminated_at_consumer_rule.ebnf`), derived
//! rather than hand-written.
//!
//! ## The AST, and why [`super::lr_chain_fold`] needs no change
//!
//! `.8` made an LR-eliminated rule return the AST its grammar declared, by handing the runtime
//! fold a per-suffix-alternative *template* whose `$1` is the running value. That machinery is
//! one hop deep because the direct/wrapper shape is one hop deep.
//!
//! An indirect route is `k` hops, and each hop has its own declared annotation. The generalization
//! is therefore **not** a new fold — it is a **composed template**: the route's annotations nested
//! outermost-first, with each level's `$1` filled by the level below it and every other `$N`
//! remapped into the flattened suffix's capture space. On SystemVerilog knot A the four hops
//!
//! ```text
//! constant_primary_sv_2017 alt#11  -> {kind: "cast", body: $1}
//! constant_cast            alt#0   -> {type: $1, body: $4}
//! casting_type             alt#1   -> {kind: "constant_primary", body: $1}
//! constant_primary         alt#0   -> {kind: "sv_2017", body: $1}
//! ```
//!
//! compose into the single template
//!
//! ```text
//! {kind: "cast", body: {type: {kind: "constant_primary",
//!                              body: {kind: "sv_2017", body: $1}},
//!                       body: $4}}
//! ```
//!
//! which [`super::lr_chain_fold::fold_lr_chain`] consumes unmodified. ⭐ That is the whole of
//! "generalize the fold from ONE rule to a mutually-recursive SET": the fold was always general;
//! only the spec that feeds it was not.
//!
//! ## Soundness posture — REFUSE LOUDLY, never approximate
//!
//! Every condition under which the composed AST could differ from the author's declaration is a
//! [`PlanRefusal`], reported by name and skipped, never guessed at:
//!
//! * a route the survey **truncated** or **dropped as degenerate** — an unsheared cycle would
//!   survive inside the clone, which is worse than not eliminating at all;
//! * a hop with **no declared return annotation** while the grammar declares annotations at all —
//!   the hop's unannotated value is not `$1` in general, and inventing one is precisely the silent
//!   wrong-AST defect `.8` exists to close;
//! * a hop whose template uses a construct the fold cannot express (`$text`, `$N::target`, `$0`).
//!
//! ⛔ The refusals are *data*, not logs: [`IndirectEliminationOutcome::refusals`] carries them to
//! the caller so a grammar that keeps a cycle can say WHY.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use super::grammar_wellformedness::{WellformednessIssue, detect_left_recursion};
use super::indirect_lr_plan::{
    ChainRoute, GuardVerdict, IndirectChainCandidate, StarvationSite, left_corner_step,
    render_elements, survey_indirect_left_recursion,
};
use super::lr_chain_fold;
use super::unified_return_ast::{LrChainWrapperSpec, UnifiedReturnAST};
use super::{
    ASTNode, Annotations, BranchAnnotation, FollowRestriction, MidSequenceSemanticAnnotation,
    RustASTPipeline, SemanticAnnotation,
};

/// A candidate the driver considered and declined, with the reason. Reported, never silent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanRefusal {
    pub base_rule: String,
    pub reason: String,
}

/// `ENGINE-UNIVERSAL-SERVICES.17` slice 3 — WHICH candidates the driver is allowed to consider.
///
/// ⛔ **`StarvationSafe` is the shipped policy and the only one any generated parser ever sees.**
/// The second variant exists so the question *"what would option (iii) actually unlock?"* can be
/// answered by running the REAL planner rather than by a second implementation of it that would
/// drift from the one it predicts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateAdmission {
    /// The shipped policy: a candidate is considered iff
    /// [`IndirectChainCandidate::is_starvation_safe`] — no rule outliving the rewrite holds it at a
    /// left corner with a non-empty residual a greedy `*` could steal.
    StarvationSafe,
    /// `.17` slice 3's DRY RUN: additionally consider candidates that are merely
    /// [`IndirectChainCandidate::is_guard_feasible`] — the population a call-site
    /// follow-restriction guard would make safe.
    ///
    /// ⛔ **This admits them WITHOUT emitting any guard**, so the grammar it produces is the
    /// measured-regressing one (`.13` slice 5: rewriting `casting_type` unguarded turns the
    /// accepted `int'(3)` into a rejection). It is therefore an instrument for PLAN-STAGE refusals
    /// only — annotation composability, the trial re-lint, the ambiguity check — and never a claim
    /// that the rewritten grammar parses. It is reachable only from
    /// [`dry_run_guard_feasible_elimination`], which works on a clone.
    GuardFeasibleDryRun,
}

impl CandidateAdmission {
    /// Does this admission mode narrate to stderr? Only the shipped pass does — a dry run's
    /// `✅ Absorbing` line would read as something the parser actually did.
    fn narrates(self) -> bool {
        matches!(self, CandidateAdmission::StarvationSafe)
    }
}

/// What [`eliminate_indirect_left_recursion`] did, as opposed to a belief about it — the same
/// posture `GRAMMAR-WELLFORMED.A2.6` forced on the linter.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IndirectEliminationOutcome {
    /// Base rules rewritten to `X := X_lr_base ( X_lr_suffix )*`, in application order.
    pub eliminated_base_rules: Vec<String>,
    /// Clone rules synthesized, in creation order — the blow-up term, measured.
    pub synthesized_clone_rules: Vec<String>,
    /// `.17` slice 7 — the CALL-SITE GUARD chains synthesized, in creation order.
    ///
    /// ⛔ Reported SEPARATELY from `synthesized_clone_rules`, not folded into it, because they price
    /// two different things: a sheared clone is what absorbing the chain costs, a guarded clone is
    /// what closing the starvation it exposes costs. Empty on every shipped plan, and that is a
    /// consequence of the admission criterion rather than a switch (see `EliminationPlan`).
    pub synthesized_guards: Vec<SynthesizedGuard>,
    /// Starvation-safe candidates that could NOT be planned, with the reason.
    pub refusals: Vec<PlanRefusal>,
}

impl IndirectEliminationOutcome {
    /// Every guard rule name, in emission order — the flat count for a report headline.
    pub fn guard_rule_names(&self) -> Vec<String> {
        self.synthesized_guards
            .iter()
            .flat_map(|guard| guard.rules.iter().cloned())
            .collect()
    }
}

/// `.17` slice 7 — one emitted guarded clone chain, as the caller can read it.
///
/// The report's job is to make the emission FALSIFIABLE without a second tool: the chain says which
/// rules were cloned, `positions` says which of the two lookaheads the sites actually owed, and
/// `call_sites` says which holders were repointed. A guard whose call-site list is empty synthesized
/// rules nothing reaches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SynthesizedGuard {
    /// The base rule whose chain this guards.
    pub base_rule: String,
    /// `X_lr_guard{v}` — the guarded stand-in for `base_rule`.
    pub guarded_base_rule: String,
    /// The transparent rules cloned, deepest first, `base_rule` last.
    pub chain: Vec<String>,
    /// `loop+trailing` · `loop` · `trailing` — which positions carry `&( residual )`.
    pub positions: &'static str,
    /// The follow restriction both lookaheads test, rendered.
    pub residual: String,
    /// The holder call sites repointed, as `rule alt#N`.
    pub call_sites: Vec<String>,
    /// Every rule name this chain synthesized, in emission order.
    pub rules: Vec<String>,
}

/// One clone rule: an intermediate with the cycle-closing edge sheared off.
#[derive(Debug, Clone)]
struct CloneRule {
    name: String,
    body: ASTNode,
    branch_return: Vec<Option<BranchAnnotation>>,
    branch_semantic: Vec<Vec<SemanticAnnotation>>,
    branch_mid_sequence: Vec<Vec<MidSequenceSemanticAnnotation>>,
    /// ⛔ The clone's RULE-level annotations, copied verbatim from the rule it clones — and the
    /// reason this field exists is a defect this slice shipped and then measured.
    ///
    /// `@profiles:` is a rule-level semantic annotation. Without it, the clone of
    /// `constant_primary_sv_2017` — a rule gated to `["sv_2017", "verilog_2005"]` — is **universal**,
    /// so the sv_2017 primary becomes reachable under an `sv_2023` parse. That is an
    /// OVER-ACCEPTANCE, the exact failure mode the strict-LRM default exists to prevent, introduced
    /// silently by a rewrite that looks purely structural. Measured on the emitted parser:
    /// `parse_constant_primary_sv_2017` carries
    /// `if !self.rule_profile_is_enabled(&["sv_2017", "verilog_2005"])` and its first clone carried
    /// nothing.
    rule_semantic: Vec<SemanticAnnotation>,
    lexical_follow_restriction: Option<FollowRestriction>,
}

/// One branch of `X_lr_suffix`: the flattened residual, the composed template that folds it back
/// into the AST the grammar declared, and the profile gate its route inherits.
#[derive(Debug, Clone)]
struct SuffixBranch {
    /// The synthetic rule the branch's elements live in. ⛔ A branch is a RULE and not an inline
    /// alternative for one reason: a rule can carry `@profiles:` and an alternative cannot. Two
    /// routes through a dialect-split spine produce **syntactically identical** suffixes with
    /// different composed templates (`{kind: "sv_2017", …}` vs `{kind: "sv_2023", …}`), so inline
    /// branches would let the first one win under BOTH profiles and stamp `sv_2017` on an
    /// `sv_2023` parse. Gating the branch rule is what keeps the discriminator honest.
    rule_name: String,
    elements: Vec<ASTNode>,
    template: UnifiedReturnAST,
    /// The `@profiles:` annotation the route is constrained by, copied verbatim from the
    /// most-restrictive rule on the route. `None` when no rule on the route declares one.
    profiles: Option<SemanticAnnotation>,
    /// Rendered profile list, for the ambiguity check and the refusal message.
    profile_key: Option<Vec<String>>,
}

/// One holder call site a guard chain serves: the alternative whose LEFT CORNER is repointed.
///
/// ⛔ Only the left corner moves. Every other element of the alternative — and therefore every `$N`
/// position in its declared annotation — is preserved, which is the same guarantee
/// [`replace_left_corner`] gives the sheared clones.
#[derive(Debug, Clone, PartialEq, Eq)]
struct GuardRedirect {
    holder_rule: String,
    alternative_index: usize,
    /// The guarded rule the holder's left corner now names — the TOP of the clone chain.
    target: String,
}

/// `ENGINE-UNIVERSAL-SERVICES.17` slice 7 — one CALL-SITE-SCOPED guarded clone chain: the shape
/// slice 6 measured as `guard_effectiveness/g7_guarded_clone_chain.ebnf`, as a plan.
///
/// ```text
/// holder     := X_lr_guard0 <residual>          ← only the LEFT CORNER is repointed
/// hop        := … | X                          ← the ORIGINAL is untouched, for residual-free holders
/// X          := X_lr_base ( X_lr_suffix )*      ← likewise untouched
/// X_lr_guard0_<hop> := … | X_lr_guard0          ← one guarded clone per TRANSPARENT hop
/// X_lr_guard0       := X_lr_base ( X_lr_guard0_suffix )* &( residual )
/// X_lr_guard0_suffix := X_lr_suffix &( residual )
/// ```
///
/// ⭐⭐ **Why the guard cannot live on the shared rule, measured rather than argued.** Slice 4's
/// bank row `g4` puts both lookaheads on the rule every caller shares and `k = n;` — a holder that
/// wants the whole run with NO residual after it — **REJECTs**; `g5` (the same minus the trailing
/// guard) accepts it, so the reject is the guard. `g7` moves the identical lookaheads onto a chain
/// reached only from the residual-bearing holder and accepts all seven inputs on both oracles. The
/// follow restriction is a property of the CALL SITE, so it belongs on a rule only that call site
/// can reach.
///
/// ⭐ **And the fallback IS the mechanism, which is why the chain must reach *into* a choice.** When
/// the trailing guard refuses an over-long seed, `X_lr_guard0` FAILS — and the guarded hop clone's
/// OTHER alternatives, copied verbatim, are still in the tournament, so a shorter one wins and the
/// holder gets its residual back. A guard placed at the holder's own call would have nothing to fall
/// back to ([[project_pgen_gives_back_at_neither_combinator]]).
#[derive(Debug, Clone)]
struct GuardChain {
    /// The follow restriction both lookaheads test, as ELEMENTS — a structural sub-parse, not a byte
    /// set. Slice 4 measured the byte form dead (`exact = 0 of 129` sites, and one comment at the
    /// iteration boundary defeats it).
    residual: Vec<ASTNode>,
    /// Emit `&( residual )` INSIDE the `*`: stops the loop at the right count. Owed exactly where
    /// [`GuardVerdict::Guardable`] — a competing, contained suffix.
    loop_guard: bool,
    /// Emit `&( residual )` at rule exit: refuses an over-long SEED, which the loop guard provably
    /// cannot touch (there the loop runs zero times). Owed where the seed verdict is `Required`.
    trailing_guard: bool,
    /// `X_lr_guard{v}` — the guarded stand-in for the base rule.
    guarded_base_rule: String,
    /// `X_lr_guard{v}_suffix` — `X_lr_suffix &( residual )`. `None` when no loop guard is owed.
    guarded_suffix_rule: Option<String>,
    /// The guarded clones of the TRANSPARENT hops, in construction order (shallowest first, so each
    /// one's repointed arm already has a target).
    clones: Vec<CloneRule>,
    /// The transparent chain this variant guards, deepest first — [`StarvationSite::guard_chain`].
    chain: Vec<String>,
    /// Original rule → the guarded rule that stands in for it inside this chain, base rule included.
    ///
    /// ⛔ Kept because a chain member can ALSO be a starvation holder in its own right — a rule with
    /// one bare arm into the chain and a second alternative holding the candidate with a residual.
    /// Its guarded clone is built from the ORIGINAL body, so without this map that second
    /// alternative would keep naming the unguarded rule while the original it was cloned from gets
    /// repointed: one guarded path and one unguarded path to the same starvation, differing only in
    /// which call site you arrived through. [`apply_plan`] closes it with a second pass.
    guarded_by_source: BTreeMap<String, String>,
    /// The holder call sites repointed at this chain.
    redirects: Vec<GuardRedirect>,
}

impl GuardChain {
    /// Every rule name this chain synthesizes, in emission order — the guard half of the blow-up
    /// term, reported next to `synthesized_clone_rules` rather than folded into it.
    fn synthesized_rules(&self) -> Vec<String> {
        let mut names: Vec<String> = self.clones.iter().map(|clone| clone.name.clone()).collect();
        if let Some(suffix) = &self.guarded_suffix_rule {
            names.push(suffix.clone());
        }
        names.push(self.guarded_base_rule.clone());
        names
    }

    /// What was WRITTEN, read back off the grammar — never what was planned.
    ///
    /// ⛔⛔ **The first version of this function reported `self.loop_guard` / `self.trailing_guard`
    /// directly, and a falsifiability plant caught it in the flattering direction** (`.17` slice 7).
    /// Deleting the trailing-lookahead emission in `apply_plan` left the report — and therefore the
    /// bank row that pins it — printing `[loop+trailing]` for a rule that no longer carried one. A
    /// summary derived from the PLAN cannot detect a defect in the EMISSION, which is the only thing
    /// it exists to describe. Same posture as [`IndirectEliminationOutcome`]'s own contract: what the
    /// pass DID, as opposed to a belief about it.
    fn summary(&self, base_rule: &str, grammar_tree: &HashMap<String, ASTNode>) -> SynthesizedGuard {
        // A guard position is present iff the rule that should carry it ENDS in a lookahead.
        let ends_in_lookahead = |rule: &str| -> bool {
            matches!(
                grammar_tree.get(rule),
                Some(ASTNode::Sequence { elements })
                    if matches!(elements.last(), Some(ASTNode::Lookahead { positive: true, .. }))
            )
        };
        let trailing = ends_in_lookahead(&self.guarded_base_rule);
        let loop_guard = self
            .guarded_suffix_rule
            .as_deref()
            .is_some_and(ends_in_lookahead);
        SynthesizedGuard {
            base_rule: base_rule.to_string(),
            guarded_base_rule: self.guarded_base_rule.clone(),
            chain: self.chain.clone(),
            positions: match (loop_guard, trailing) {
                (true, true) => "loop+trailing",
                (true, false) => "loop",
                (false, true) => "trailing",
                // ⛔ Reachable only when the emission dropped a lookahead the plan owed — the exact
                // defect above. Named rather than `unreachable!()`, because a report that panics is
                // a worse instrument than one that says what it saw.
                (false, false) => "NONE-EMITTED",
            },
            residual: super::indirect_lr_plan::render_elements_display(&self.residual),
            call_sites: self
                .redirects
                .iter()
                .map(|redirect| {
                    format!("{} alt#{}", redirect.holder_rule, redirect.alternative_index)
                })
                .collect(),
            // Likewise read back: a name the plan allocated but nothing wrote is not a synthesized
            // rule.
            rules: self
                .synthesized_rules()
                .into_iter()
                .filter(|name| grammar_tree.contains_key(name))
                .collect(),
        }
    }
}

/// An applicable plan — everything [`apply_plan`] needs, computed without mutating anything.
#[derive(Debug, Clone)]
struct EliminationPlan {
    base_rule: String,
    helper_base_rule: String,
    helper_suffix_rule: String,
    clones: Vec<CloneRule>,
    /// `X_lr_base`'s alternatives, in the author's original order.
    base_alternatives: Vec<ASTNode>,
    /// `X_lr_base`'s per-alternative annotations, index-aligned with `base_alternatives`.
    base_branch_return: Vec<Option<BranchAnnotation>>,
    base_branch_semantic: Vec<Vec<SemanticAnnotation>>,
    base_branch_mid_sequence: Vec<Vec<MidSequenceSemanticAnnotation>>,
    suffix_branches: Vec<SuffixBranch>,
    /// `.17` slice 7 — the call-site guards this plan owes its surviving starvation sites.
    ///
    /// ⛔⛔ **EMPTY on every plan a shipped parser has ever seen, and that is a CONSEQUENCE, not a
    /// switch.** The shipped admission is [`CandidateAdmission::StarvationSafe`], whose whole
    /// definition is `surviving_starvation_sites().is_empty()` — so a shipped candidate has no site
    /// to guard and this vector cannot be non-empty. It fills only under
    /// [`CandidateAdmission::GuardFeasibleDryRun`]. There is deliberately no flag to read: a flag
    /// would be a second thing that has to agree with the criterion.
    guard_chains: Vec<GuardChain>,
}

/// What the rewrite must do to one alternative of one rule on a route.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Shear {
    /// The alternative's leading reference IS the base rule: the clone drops the alternative
    /// entirely, because the `( X_lr_suffix )*` now carries that derivation.
    Drop,
    /// The alternative's leading reference continues the route: the clone redirects it to the
    /// clone of `next_rule`.
    Redirect { next_rule: String },
}

/// Eliminate every indirect left-recursive cycle the survey reports as starvation-safe, to a
/// fixpoint.
///
/// ⭐ **Why a fixpoint and not one pass.** Two candidates can sit on the SAME knot — `ebnf`'s
/// `return_expression` and `expression_return` are each other's intermediates — and rewriting one
/// breaks the other's cycle. Re-surveying after every application is what keeps the pass from
/// eliminating the same knot twice at two different rules, and it costs one survey per rewrite on
/// grammars whose rewrite count is single-digit.
///
/// ⭐ **Why the order is "widest knot first, then most seeds".** Two criteria, each forced by a
/// measurement rather than by argument:
///
/// 1. **Widest coverage first** — a mutually-recursive SET must be eliminated at a rule that
///    DOMINATES it, never at one of its members. SystemVerilog's cast/call knot holds two dialect
///    twins reaching each other through a shared `constant_primary` spine; a plan built at either
///    twin leaves the other's arm live, because the path back re-enters the spine and so is not a
///    SIMPLE route for the walk to shear. [`IndirectChainCandidate::covered_rules`] separates the
///    dominator from its members, and the trial below proves the separation rather than trusting it.
/// 2. **Most seeds next** — slice 3 measured that the base rule must be the CONSUMER, the rule the
///    outside world asks for, and a consumer is the rule with the richest non-cyclic alternatives.
///    On `ebnf` the two candidates cover the identical knot and this picks `return_expression`
///    (4 seeds) over `expression_return` (1) — the knot slice 1 named.
///
/// Ties break on fewer clones, then on `rule_order` position, so the choice is byte-deterministic
/// like the codegen it feeds.
pub(super) fn eliminate_indirect_left_recursion(
    grammar_tree: &mut HashMap<String, ASTNode>,
    rule_order: &mut Vec<String>,
    annotations: Option<&mut Annotations>,
) -> IndirectEliminationOutcome {
    eliminate_indirect_left_recursion_with_admission(
        grammar_tree,
        rule_order,
        annotations,
        CandidateAdmission::StarvationSafe,
    )
}

/// What a [`dry_run_guard_feasible_elimination`] found — the outcome, plus the two numbers that
/// price option (iii): how many left-recursive rule rows it started from and how many it left.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuardDryRun {
    /// What the driver did with the widened admission set.
    pub outcome: IndirectEliminationOutcome,
    /// Left-recursive rule rows before the dry run — the report's own `surviving_cycle_rules`.
    pub cycle_rows_before: usize,
    /// Left-recursive rule rows the dry run LEFT STANDING. ⛔ This is the payoff figure, and it is
    /// re-derived from the rewritten clone by the same [`detect_left_recursion`] the lint uses —
    /// never inferred from "N absorbed, so N fewer".
    pub cycle_rows_after: usize,
}

/// `.17` slice 3 — answer *"which of the guard-feasible candidates actually reach a plan?"* by
/// running the REAL driver on a CLONE with the guard census admitted.
///
/// ⛔ **What this measures and what it does not.** The three refusal sources downstream of the
/// starvation check — a hop with no declared return annotation, the trial re-lint, the ambiguity
/// comparison — are all independent of whether a guard is emitted, so a guardless dry run is a
/// SOUND predictor for them. It models nothing the guard emission would itself add, and it makes no
/// claim whatsoever about the resulting grammar's *parses* (see [`CandidateAdmission`]).
pub fn dry_run_guard_feasible_elimination(
    grammar_tree: &HashMap<String, ASTNode>,
    rule_order: &[String],
    annotations: Option<&Annotations>,
) -> GuardDryRun {
    let cycle_rows = |tree: &HashMap<String, ASTNode>, order: &[String]| -> usize {
        detect_left_recursion(tree, order)
            .into_iter()
            .filter(|issue| matches!(issue, WellformednessIssue::LeftRecursive { .. }))
            .count()
    };

    let mut trial_tree = grammar_tree.clone();
    let mut trial_order = rule_order.to_vec();
    let mut trial_annotations = annotations.cloned();
    let cycle_rows_before = cycle_rows(&trial_tree, &trial_order);
    let outcome = eliminate_indirect_left_recursion_with_admission(
        &mut trial_tree,
        &mut trial_order,
        trial_annotations.as_mut(),
        CandidateAdmission::GuardFeasibleDryRun,
    );
    let cycle_rows_after = cycle_rows(&trial_tree, &trial_order);
    GuardDryRun {
        outcome,
        cycle_rows_before,
        cycle_rows_after,
    }
}

/// The driver itself. Behaviour is documented on [`eliminate_indirect_left_recursion`], which is
/// the only caller a generated parser ever goes through; `admission` is the sole difference between
/// that path and `.17` slice 3's dry run, and it is read in exactly two places — the candidate
/// filter below, and [`CandidateAdmission::narrates`].
fn eliminate_indirect_left_recursion_with_admission(
    grammar_tree: &mut HashMap<String, ASTNode>,
    rule_order: &mut Vec<String>,
    mut annotations: Option<&mut Annotations>,
    admission: CandidateAdmission,
) -> IndirectEliminationOutcome {
    let mut outcome = IndirectEliminationOutcome::default();
    // A refused candidate stays refused for the whole pass — otherwise every re-survey would
    // re-report it and the refusal list would grow with the loop counter rather than with the
    // number of distinct problems.
    let mut refused: BTreeSet<String> = BTreeSet::new();

    loop {
        let survey = survey_indirect_left_recursion(grammar_tree, rule_order);
        let admitted = match admission {
            CandidateAdmission::StarvationSafe => survey.safe_candidates(),
            CandidateAdmission::GuardFeasibleDryRun => survey.guard_admissible_candidates(),
        };
        let mut ordered: Vec<&IndirectChainCandidate> = admitted
            .into_iter()
            .filter(|candidate| !refused.contains(&candidate.base_rule))
            .collect();
        if ordered.is_empty() {
            break;
        }
        let position = |rule: &str| {
            rule_order
                .iter()
                .position(|name| name == rule)
                .unwrap_or(usize::MAX)
        };
        ordered.sort_by(|left, right| {
            right
                .covered_rules()
                .len()
                .cmp(&left.covered_rules().len())
                .then_with(|| {
                    right
                        .acyclic_alternative_indices
                        .len()
                        .cmp(&left.acyclic_alternative_indices.len())
                })
                .then_with(|| left.clone_cost().len().cmp(&right.clone_cost().len()))
                .then_with(|| position(&left.base_rule).cmp(&position(&right.base_rule)))
        });

        let candidate = ordered[0];
        let before_rows = survey.surviving_cycle_rules.len();
        let attempt = plan_elimination(candidate, grammar_tree, annotations.as_deref()).and_then(
            |plan| {
                // ⭐ TRIAL FIRST, COMMIT SECOND — the pass verifies its own postcondition instead
                // of asserting it. A route set only covers cycles whose every hop exposes a BARE
                // leading rule reference (`indirect_lr_plan`'s deliberate narrowness), so a cycle
                // closing through a nullable prefix is invisible to the plan and would survive
                // unsheared inside a clone. Applying to copies and re-running the LINT turns that
                // from an argument into a measurement, and costs one grammar clone per rewrite on
                // grammars whose rewrite count is single-digit.
                let mut trial_grammar = grammar_tree.clone();
                let mut trial_order = rule_order.clone();
                let mut trial_annotations = annotations.as_deref().cloned();
                apply_plan(
                    &plan,
                    &mut trial_grammar,
                    &mut trial_order,
                    trial_annotations.as_mut(),
                );
                let after: Vec<(String, Vec<String>)> =
                    detect_left_recursion(&trial_grammar, &trial_order)
                        .into_iter()
                        .filter_map(|issue| match issue {
                            WellformednessIssue::LeftRecursive { rule, cycle } => {
                                Some((rule, cycle))
                            }
                            _ => None,
                        })
                        .collect();
                if let Some((_, cycle)) = after.iter().find(|(rule, _)| rule == &plan.base_rule) {
                    return Err(format!(
                        "the rewrite left '{}' left-recursive — the plan sheared only part of the \
                         cycle, which still closes through {}",
                        plan.base_rule,
                        cycle.join(" -> ")
                    ));
                }
                if after.len() >= before_rows {
                    return Err(format!(
                        "the rewrite did not reduce the left-recursive rule rows ({before_rows} \
                         before, {} after), so it would trade one cycle for another",
                        after.len()
                    ));
                }
                Ok(plan)
            },
        );

        match attempt {
            Ok(plan) => {
                if admission.narrates() {
                    eprintln!(
                        "[indirect_lr_elimination] ✅ Absorbing indirect left-recursive chain at rule '{}' \
                         ({} route(s), {} clone(s) via helper '{}')",
                        plan.base_rule,
                        plan.suffix_branches.len(),
                        plan.clones.len(),
                        plan.helper_base_rule
                    );
                }
                outcome.eliminated_base_rules.push(plan.base_rule.clone());
                outcome
                    .synthesized_clone_rules
                    .extend(plan.clones.iter().map(|clone| clone.name.clone()));
                apply_plan(&plan, grammar_tree, rule_order, annotations.as_deref_mut());
                // ⛔ AFTER `apply_plan`, deliberately: every field below is read back off the tree
                // the pass just wrote, so a defect in the EMISSION shows up in the report instead of
                // being narrated over by the plan that intended it.
                outcome.synthesized_guards.extend(
                    plan.guard_chains
                        .iter()
                        .map(|chain| chain.summary(&plan.base_rule, grammar_tree)),
                );
            }
            Err(reason) => {
                if admission.narrates() {
                    eprintln!(
                        "[indirect_lr_elimination] ⏭️  Declining rule '{}': {reason}",
                        candidate.base_rule
                    );
                }
                refused.insert(candidate.base_rule.clone());
                outcome.refusals.push(PlanRefusal {
                    base_rule: candidate.base_rule.clone(),
                    reason,
                });
            }
        }
    }

    outcome
}

/// Turn a starvation-safe survey candidate into an applicable plan, or say why not.
fn plan_elimination(
    candidate: &IndirectChainCandidate,
    grammar_tree: &HashMap<String, ASTNode>,
    annotations: Option<&Annotations>,
) -> Result<EliminationPlan, String> {
    // ⛔ Both of these mean the survey KNOWS about cycle edges it did not report. Shearing only the
    // reported ones would leave a live cycle inside the clone — a rewrite that changes the grammar
    // and does not fix it, which is strictly worse than declining.
    if candidate.routes_truncated {
        return Err(format!(
            "route enumeration hit the {}-route budget, so the plan would shear only a prefix of \
             the cycle edges",
            super::indirect_lr_plan::MAX_ROUTES_PER_BASE
        ));
    }
    if candidate.degenerate_routes_dropped > 0 {
        return Err(format!(
            "{} route(s) iterate an empty suffix; their cycle edges would survive unsheared inside \
             the clones (the linter's non-terminating class owns them)",
            candidate.degenerate_routes_dropped
        ));
    }

    let base_rule = candidate.base_rule.as_str();
    let base_body = grammar_tree
        .get(base_rule)
        .ok_or_else(|| format!("rule '{base_rule}' is not in the grammar"))?;
    let base_alternatives = RustASTPipeline::as_alternatives(base_body);

    // ---- 1. The shear map: for every (rule, alternative) on any route, what the clone must do.
    let mut shear: BTreeMap<String, BTreeMap<usize, Shear>> = BTreeMap::new();
    for route in &candidate.routes {
        for step in &route.steps {
            let action = if step.next_rule == base_rule {
                Shear::Drop
            } else {
                Shear::Redirect {
                    next_rule: step.next_rule.clone(),
                }
            };
            shear
                .entry(step.rule.clone())
                .or_default()
                .insert(step.alternative_index, action);
        }
    }

    // ---- 2. The clones, built depth-first from each of the base rule's cyclic alternatives.
    // `taken` guards the synthetic names against both the grammar and this plan's own allocations.
    let mut taken: BTreeSet<String> = grammar_tree.keys().cloned().collect();
    let mut clones: Vec<CloneRule> = Vec::new();
    let mut clone_names: BTreeMap<String, Option<String>> = BTreeMap::new();

    let base_shear = shear
        .get(base_rule)
        .cloned()
        .ok_or_else(|| format!("rule '{base_rule}' has no route step of its own"))?;

    // ---- 3. `X_lr_base`: the author's alternatives in the author's order, each cyclic one
    // replaced by a reference to the sheared clone of the rule it named.
    let mut rewritten_alternatives: Vec<Option<ASTNode>> =
        base_alternatives.iter().cloned().map(Some).collect();
    for (index, action) in &base_shear {
        let alternative = rewritten_alternatives
            .get(*index)
            .and_then(|slot| slot.clone())
            .ok_or_else(|| {
                format!("route names alternative {index} of '{base_rule}', which does not exist")
            })?;
        match action {
            // A `Drop` at the base rule is DIRECT left recursion, which
            // `normalize_direct_left_recursive_alternatives` has already turned into the wrapper
            // shape by the time this pass runs. Reaching it here means the two passes disagree.
            Shear::Drop => {
                return Err(format!(
                    "alternative {index} of '{base_rule}' is directly left-recursive; the direct \
                     normalization pass should have consumed it before this one ran"
                ));
            }
            Shear::Redirect { next_rule } => {
                let clone = clone_rule(
                    next_rule,
                    base_rule,
                    &shear,
                    grammar_tree,
                    annotations,
                    &mut taken,
                    &mut clones,
                    &mut clone_names,
                    &mut Vec::new(),
                )?;
                rewritten_alternatives[*index] = clone.map(|name| {
                    replace_left_corner(&alternative, &name)
                });
            }
        }
    }

    let (base_branch_return, base_branch_semantic, base_branch_mid_sequence) =
        branch_annotations_of(base_rule, base_alternatives.len(), annotations);
    let mut kept_alternatives: Vec<ASTNode> = Vec::new();
    let mut kept_return: Vec<Option<BranchAnnotation>> = Vec::new();
    let mut kept_semantic: Vec<Vec<SemanticAnnotation>> = Vec::new();
    let mut kept_mid: Vec<Vec<MidSequenceSemanticAnnotation>> = Vec::new();
    for (index, slot) in rewritten_alternatives.into_iter().enumerate() {
        let Some(alternative) = slot else {
            continue;
        };
        kept_alternatives.push(alternative);
        kept_return.push(base_branch_return.get(index).cloned().flatten());
        kept_semantic.push(base_branch_semantic.get(index).cloned().unwrap_or_default());
        kept_mid.push(base_branch_mid_sequence.get(index).cloned().unwrap_or_default());
    }
    if kept_alternatives.is_empty() {
        return Err(format!(
            "every alternative of '{base_rule}' vanished under shearing, leaving no seed"
        ));
    }

    // ---- 4. The suffix branches, one per route, each with its composed template and the profile
    // gate its route inherits.
    let mut suffix_branches: Vec<SuffixBranch> = Vec::new();
    let mut underivable_routes = 0usize;
    for (index, route) in candidate.routes.iter().enumerate() {
        let (profiles, profile_key) = match route_profile_gate(route, annotations)? {
            RouteGate::Unconstrained => (None, None),
            RouteGate::Gated {
                annotation,
                profiles,
            } => (Some(annotation), Some(profiles)),
            RouteGate::Underivable => {
                underivable_routes += 1;
                continue;
            }
        };
        suffix_branches.push(SuffixBranch {
            rule_name: allocate(format!("{base_rule}_lr_suffix_r{index}"), &mut taken),
            elements: route.suffix_elements(),
            template: compose_route_template(route, annotations)?,
            profiles,
            profile_key,
        });
    }
    if underivable_routes > 0 {
        eprintln!(
            "[indirect_lr_elimination] ℹ️  '{base_rule}': {underivable_routes} route(s) cross \
             disjoint @profiles and derive nothing under any profile — sheared, but contributing \
             no suffix branch"
        );
    }
    if suffix_branches.is_empty() {
        return Err(format!(
            "every route out of '{base_rule}' crosses disjoint @profiles, so the chain would \
             iterate nothing"
        ));
    }
    // ⛔ Two routes that are INDISTINGUISHABLE at parse time — same suffix syntax, same profile
    // gate — but carry different ASTs are an ambiguity the ordered choice resolves arbitrarily.
    // Refuse rather than let the first branch stamp its template on the other's derivations.
    for (left_index, left) in suffix_branches.iter().enumerate() {
        for right in suffix_branches.iter().skip(left_index + 1) {
            if left.profile_key == right.profile_key
                && super::indirect_lr_plan::render_elements(&left.elements)
                    == super::indirect_lr_plan::render_elements(&right.elements)
                && left.template != right.template
            {
                return Err(format!(
                    "two routes iterate the identical suffix '{}' under the identical profile gate \
                     {:?} but declare different ASTs, so the ordered choice would pick one \
                     arbitrarily",
                    super::indirect_lr_plan::render_elements(&left.elements),
                    left.profile_key
                ));
            }
        }
    }

    // ---- 5. Refuse now, at generation time, anything the runtime fold cannot express.
    let specs: Vec<LrChainWrapperSpec> = suffix_branches
        .iter()
        .enumerate()
        .map(|(alt_index, branch)| LrChainWrapperSpec {
            alt_index,
            original_body_length: 1 + branch.elements.len(),
            annotation_template: branch.template.clone(),
        })
        .collect();
    lr_chain_fold::validate_chain_templates(base_rule, &specs)?;

    let helper_base_rule = allocate(format!("{base_rule}_lr_base"), &mut taken);
    let helper_suffix_rule = allocate(format!("{base_rule}_lr_suffix"), &mut taken);

    // ---- 6. `.17` slice 7 — the call-site guards the surviving starvation sites owe.
    //
    // ⛔ Unconditional, and NOT behind an admission check, deliberately. `safe_candidates()` is
    // defined as "no surviving starvation site", so on the shipped path this returns an empty vector
    // by construction — a second switch here would be a second thing that has to agree with the
    // criterion, and the two could drift. The byte-identity of every generated parser is therefore
    // a MEASUREMENT of that argument, not a policy.
    let guard_chains = plan_guard_chains(
        candidate,
        grammar_tree,
        annotations,
        &mut taken,
    )?;

    Ok(EliminationPlan {
        base_rule: base_rule.to_string(),
        helper_base_rule,
        helper_suffix_rule,
        clones,
        base_alternatives: kept_alternatives,
        base_branch_return: kept_return,
        base_branch_semantic: kept_semantic,
        base_branch_mid_sequence: kept_mid,
        suffix_branches,
        guard_chains,
    })
}

/// Build (once) the sheared clone of `rule`, returning its name — or `None` when shearing leaves
/// the rule with no alternatives at all, in which case the referring alternative vanishes too.
///
/// The clone graph is a DAG by construction: every `Redirect` edge follows a route step, and a
/// route is a *simple* path that terminates at the base rule with a `Drop`. `in_progress` is kept
/// anyway so a future non-simple route source fails loudly instead of recursing forever.
#[allow(clippy::too_many_arguments)]
fn clone_rule(
    rule: &str,
    base_rule: &str,
    shear: &BTreeMap<String, BTreeMap<usize, Shear>>,
    grammar_tree: &HashMap<String, ASTNode>,
    annotations: Option<&Annotations>,
    taken: &mut BTreeSet<String>,
    clones: &mut Vec<CloneRule>,
    clone_names: &mut BTreeMap<String, Option<String>>,
    in_progress: &mut Vec<String>,
) -> Result<Option<String>, String> {
    if let Some(existing) = clone_names.get(rule) {
        return Ok(existing.clone());
    }
    if in_progress.iter().any(|name| name == rule) {
        return Err(format!(
            "clone construction re-entered '{rule}'; the route set is not a simple-path set"
        ));
    }

    let body = grammar_tree
        .get(rule)
        .ok_or_else(|| format!("route names rule '{rule}', which is not in the grammar"))?;
    let alternatives = RustASTPipeline::as_alternatives(body);
    let rule_shear = shear
        .get(rule)
        .ok_or_else(|| format!("rule '{rule}' is on a route but carries no shear action"))?;

    in_progress.push(rule.to_string());
    let mut kept: Vec<ASTNode> = Vec::new();
    let mut kept_indices: Vec<usize> = Vec::new();
    for (index, alternative) in alternatives.iter().enumerate() {
        match rule_shear.get(&index) {
            None => {
                kept.push(alternative.clone());
                kept_indices.push(index);
            }
            Some(Shear::Drop) => {}
            Some(Shear::Redirect { next_rule }) => {
                let nested = clone_rule(
                    next_rule,
                    base_rule,
                    shear,
                    grammar_tree,
                    annotations,
                    taken,
                    clones,
                    clone_names,
                    in_progress,
                )?;
                if let Some(nested_name) = nested {
                    kept.push(replace_left_corner(alternative, &nested_name));
                    kept_indices.push(index);
                }
            }
        }
    }
    in_progress.pop();

    if kept.is_empty() {
        clone_names.insert(rule.to_string(), None);
        return Ok(None);
    }

    let name = allocate(format!("{base_rule}_lr_seed_{rule}"), taken);
    let (branch_return, branch_semantic, branch_mid) =
        branch_annotations_of(rule, alternatives.len(), annotations);
    clones.push(CloneRule {
        name: name.clone(),
        body: RustASTPipeline::build_or_node(kept),
        branch_return: kept_indices
            .iter()
            .map(|index| branch_return.get(*index).cloned().flatten())
            .collect(),
        branch_semantic: kept_indices
            .iter()
            .map(|index| branch_semantic.get(*index).cloned().unwrap_or_default())
            .collect(),
        branch_mid_sequence: kept_indices
            .iter()
            .map(|index| branch_mid.get(*index).cloned().unwrap_or_default())
            .collect(),
        // The clone stands in for the rule at the seed position, so every rule-level directive that
        // constrains the original — `@profiles:` above all — must constrain the clone identically.
        rule_semantic: annotations
            .and_then(|annotations| annotations.semantic_annotations.get(rule).cloned())
            .unwrap_or_default(),
        lexical_follow_restriction: annotations
            .and_then(|annotations| annotations.lexical_follow_restrictions.get(rule).cloned()),
    });
    clone_names.insert(rule.to_string(), Some(name.clone()));
    Ok(Some(name))
}

/// `.17` slice 7 — the elements a site's holder still needs after the candidate: the LOOKAHEAD BODY.
///
/// Read back through [`left_corner_step`] rather than re-derived, so the emitter's notion of
/// "everything after the leading rule reference" cannot drift from the survey's.
fn site_residual(
    site: &StarvationSite,
    grammar_tree: &HashMap<String, ASTNode>,
) -> Result<Vec<ASTNode>, String> {
    let body = grammar_tree.get(&site.rule).ok_or_else(|| {
        format!(
            "starvation site names rule '{}', which is not in the grammar",
            site.rule
        )
    })?;
    let alternatives = RustASTPipeline::as_alternatives(body);
    let alternative = alternatives.get(site.alternative_index).ok_or_else(|| {
        format!(
            "starvation site names alternative {} of '{}', which does not exist",
            site.alternative_index, site.rule
        )
    })?;
    let step = left_corner_step(&site.rule, site.alternative_index, alternative).ok_or_else(|| {
        format!(
            "alternative {} of '{}' no longer exposes a bare leading rule reference, so its follow \
             restriction cannot be emitted",
            site.alternative_index, site.rule
        )
    })?;
    if step.residual.is_empty() {
        return Err(format!(
            "alternative {} of '{}' has an empty residual, so it is not a starvation site at all",
            site.alternative_index, site.rule
        ));
    }
    Ok(step.residual)
}

/// `.17` slice 7 — turn a candidate's SURVIVING starvation sites into the guarded clone chains that
/// close them, or say why not.
///
/// One chain per distinct **(positions owed, residual, transparent chain)**. Two holders share a
/// chain only when all three agree, and each term is load-bearing:
///
/// * **positions** — a chain carrying the trailing guard REJECTS a holder that wants no residual
///   (slice 4's `g4` on `e7`), so the position set cannot be unioned across sites;
/// * **residual** — the lookahead body IS the residual, so a different one is a different rule;
/// * **chain** — two holders can reach the base through different transparent rules, and their
///   clone sets then differ anyway.
///
/// ⛔ **This is a strictly finer key than [`IndirectChainCandidate::guard_variants`], which slice 2
/// used to PRICE the design and which counts distinct residual BYTE SETS.** A byte set is what the
/// dead byte-test form would have tested; the shipped form tests structure, and two residuals with
/// the same FIRST set are two different sub-parses. The census number is therefore a lower bound on
/// the chain count, and the report prints both rather than letting one stand for the other.
fn plan_guard_chains(
    candidate: &IndirectChainCandidate,
    grammar_tree: &HashMap<String, ASTNode>,
    annotations: Option<&Annotations>,
    taken: &mut BTreeSet<String>,
) -> Result<Vec<GuardChain>, String> {
    let base_rule = candidate.base_rule.as_str();
    // key -> (positions, residual, chain, sites) in a deterministic order.
    let mut grouped: BTreeMap<String, (bool, bool, Vec<ASTNode>, Vec<String>, Vec<GuardRedirect>)> =
        BTreeMap::new();

    for site in candidate.surviving_starvation_sites() {
        // ⛔ A blocked verdict must stop the WHOLE plan, not just skip its site. Emitting guards for
        // the closable sites and leaving one starved is a rewrite that changes the grammar and does
        // not fix it — strictly worse than declining, which is this module's standing posture.
        if site.guard.verdict.blocks_guard() {
            return Err(format!(
                "starvation site '{}' alt#{} is {} on the loop guard, so a guarded chain cannot \
                 close it",
                site.rule,
                site.alternative_index,
                site.guard.verdict.token()
            ));
        }
        if site.guard.seed_verdict.blocks_guard() {
            return Err(format!(
                "starvation site '{}' alt#{} is {} on the seed guard, so a guarded chain cannot \
                 close it",
                site.rule,
                site.alternative_index,
                site.guard.seed_verdict.token()
            ));
        }
        let loop_guard = matches!(site.guard.verdict, GuardVerdict::Guardable);
        let trailing_guard = site.guard.seed_verdict.needs_trailing_guard();
        // `no_competition` / `residual_nullable` on BOTH positions: this holder provably cannot be
        // starved, so it is left naming the original rule and pays nothing.
        if !loop_guard && !trailing_guard {
            continue;
        }
        if site.guard_chain.is_empty() || site.guard_chain.last().map(String::as_str) != Some(base_rule)
        {
            return Err(format!(
                "starvation site '{}' alt#{} has no transparent chain ending at '{base_rule}', so \
                 there is nowhere to hang its guard",
                site.rule, site.alternative_index
            ));
        }
        let residual = site_residual(site, grammar_tree)?;
        // ⛔ The key's last field is a SERIALIZATION, not a rendering, and the reason is soundness
        // rather than tidiness. `render_elements` is deliberately paren-free — it is frozen for the
        // ambiguity refusal that compares it — so `a b*` is the rendering of BOTH
        // `Sequence[a, Quantified{b}]` and `Quantified{Sequence[a, b]}`. Two sites whose residuals
        // collide there would share one chain, and the chain carries ONE lookahead: the second
        // site would be guarded against a follow restriction that is not its own, silently. The
        // readable fields come first so the variant numbering still sorts by something a human can
        // read in the report.
        let key = format!(
            "{}|{}|{}|{}|{}",
            u8::from(loop_guard),
            u8::from(trailing_guard),
            site.guard_chain.join(">"),
            render_elements(&residual),
            serde_json::to_string(&residual).unwrap_or_default()
        );
        let entry = grouped.entry(key).or_insert_with(|| {
            (
                loop_guard,
                trailing_guard,
                residual.clone(),
                site.guard_chain.clone(),
                Vec::new(),
            )
        });
        entry.4.push(GuardRedirect {
            holder_rule: site.rule.clone(),
            alternative_index: site.alternative_index,
            // Filled in below, once the chain's top clone has a name.
            target: String::new(),
        });
    }

    let mut chains: Vec<GuardChain> = Vec::new();
    for (variant, (_, (loop_guard, trailing_guard, residual, chain, mut redirects))) in
        grouped.into_iter().enumerate()
    {
        let guarded_base_rule = allocate(format!("{base_rule}_lr_guard{variant}"), taken);
        let guarded_suffix_rule = loop_guard
            .then(|| allocate(format!("{base_rule}_lr_guard{variant}_suffix"), taken));

        // The clones, built shallowest-first so each repointed arm already has its target. The
        // chain arrives deepest-first (transparency depth descending, base rule last), so the
        // construction order is that list reversed.
        let mut guarded: BTreeMap<String, String> = BTreeMap::new();
        guarded.insert(base_rule.to_string(), guarded_base_rule.clone());
        let chain_set: BTreeSet<&String> = chain.iter().collect();
        let mut clones: Vec<CloneRule> = Vec::new();
        for hop in chain.iter().rev() {
            if hop == base_rule {
                continue;
            }
            let body = grammar_tree
                .get(hop)
                .ok_or_else(|| format!("guard chain names rule '{hop}', which is not in the grammar"))?;
            let alternatives = RustASTPipeline::as_alternatives(body);
            let mut repointed = 0usize;
            let mut kept: Vec<ASTNode> = Vec::with_capacity(alternatives.len());
            for (index, alternative) in alternatives.iter().enumerate() {
                let step = left_corner_step(hop, index, alternative);
                let transparent_into_chain = step
                    .as_ref()
                    .is_some_and(|step| step.residual.is_empty() && chain_set.contains(&step.next_rule));
                match step.filter(|_| transparent_into_chain) {
                    Some(step) => {
                        // ⛔ DECLINE LOUDLY rather than emit a hole. A bare arm into the chain whose
                        // target this construction order has not reached yet means the transparency
                        // relation is not a DAG under "strictly shallower" — the base would stay
                        // reachable from this holder through an UNGUARDED path, and the guard would
                        // silently fail to fire on exactly the derivations it was emitted for.
                        let target = guarded.get(&step.next_rule).ok_or_else(|| {
                            format!(
                                "guarded clone of '{hop}' alt#{index} reaches '{}' , which is on \
                                 the chain but not yet guarded — the transparent chain is cyclic, \
                                 so a guarded path cannot be built",
                                step.next_rule
                            )
                        })?;
                        kept.push(replace_left_corner(alternative, target));
                        repointed += 1;
                    }
                    None => kept.push(alternative.clone()),
                }
            }
            if repointed == 0 {
                return Err(format!(
                    "rule '{hop}' is on the transparent chain to '{base_rule}' but exposes no bare \
                     arm into it, so its guarded clone would guard nothing"
                ));
            }
            let name = allocate(format!("{base_rule}_lr_guard{variant}_{hop}"), taken);
            // Every alternative is kept, so the annotations stay index-aligned with the original —
            // no `kept_indices` remap, unlike the sheared clones above.
            let (branch_return, branch_semantic, branch_mid) =
                branch_annotations_of(hop, alternatives.len(), annotations);
            clones.push(CloneRule {
                name: name.clone(),
                body: RustASTPipeline::build_or_node(kept),
                branch_return,
                branch_semantic,
                branch_mid_sequence: branch_mid,
                // The clone STANDS IN for the hop at this call site, so every rule-level directive
                // that constrains the original must constrain it identically — `@profiles:` above
                // all, whose omission on the sheared clones was a measured over-acceptance.
                rule_semantic: annotations
                    .and_then(|annotations| annotations.semantic_annotations.get(hop).cloned())
                    .unwrap_or_default(),
                lexical_follow_restriction: annotations
                    .and_then(|annotations| annotations.lexical_follow_restrictions.get(hop).cloned()),
            });
            guarded.insert(hop.clone(), name);
        }

        // ⛔ CONSTRUCTION order is bottom-up because each repointed arm needs its target to exist;
        // EMISSION order is name-ascending because a reader compares `rules:` against `chain:` and a
        // clone set that came out `…_pb, …_pa` for a chain printed `pa > pb` reads as a defect.
        // Reordering is safe and is not a matter of taste: rule order carries no semantics — it is
        // the emission sequence only — while it IS part of the codegen's byte-determinism contract,
        // so it has to be a stated rule rather than a by-product of the traversal.
        clones.sort_by(|left, right| left.name.cmp(&right.name));

        // The chain TOP is the guarded clone of the rule the holder actually names, which is the
        // deepest-first list's head — the base rule itself when the holder names it directly.
        let top = chain
            .first()
            .and_then(|rule| guarded.get(rule))
            .cloned()
            .ok_or_else(|| format!("guard chain for '{base_rule}' has no top rule"))?;
        for redirect in &mut redirects {
            redirect.target = top.clone();
        }
        chains.push(GuardChain {
            residual,
            loop_guard,
            trailing_guard,
            guarded_base_rule,
            guarded_suffix_rule,
            clones,
            chain,
            guarded_by_source: guarded,
            redirects,
        });
    }
    Ok(chains)
}

/// Replace an alternative's leading rule reference with a reference to `replacement`, preserving
/// every other element and therefore every `$N` position in the alternative's own annotation.
fn replace_left_corner(alternative: &ASTNode, replacement: &str) -> ASTNode {
    let reference = RustASTPipeline::make_rule_reference_node(replacement);
    match alternative {
        ASTNode::Sequence { elements } if !elements.is_empty() => {
            let mut rewritten = elements.clone();
            rewritten[0] = reference;
            ASTNode::Sequence {
                elements: rewritten,
            }
        }
        // A bare reference alternative IS its own left corner.
        _ => reference,
    }
}

/// Compose one route's per-hop annotations into the single template
/// [`super::lr_chain_fold::fold_lr_chain`] applies per suffix iteration.
///
/// `$1` at hop `i` is the value of hop `i + 1`, so the hops nest outermost-first and the innermost
/// hop's `$1` is the running fold value. Every other `$m` is remapped into the flattened suffix's
/// capture space by [`suffix_offsets`].
fn compose_route_template(
    route: &ChainRoute,
    annotations: Option<&Annotations>,
) -> Result<UnifiedReturnAST, String> {
    // Without an `Annotations` the grammar declares no ASTs at all, so the eliminated rule's value
    // is the engine default and a template is neither needed nor meaningful.
    let Some(annotations) = annotations else {
        return Ok(UnifiedReturnAST::PositionalRef { index: 1 });
    };
    let offsets = suffix_offsets(route);
    // Innermost hop first: its `$1` is the running value.
    let mut composed = UnifiedReturnAST::PositionalRef { index: 1 };
    for (index, step) in route.steps.iter().enumerate().rev() {
        let declared = annotations
            .branch_return_annotations
            .get(&step.rule)
            .and_then(|branches| branches.get(step.alternative_index).cloned().flatten())
            .and_then(|annotation| annotation.parsed_ast);
        let template = match declared {
            Some(template) => template,
            // ⭐ An UNANNOTATED hop is composable in exactly one case, and it is a structural fact
            // rather than a convention: an alternative that is a BARE rule reference adds no
            // wrapper node, so its value IS the referenced rule's value and `$1` reproduces it
            // exactly. That is the same AST-transparency `hoist_branch_annotations` relies on.
            None if step.residual.is_empty() => UnifiedReturnAST::PositionalRef { index: 1 },
            // ⛔ With a residual there is no such fact to lean on: the hop's undeclared value is the
            // engine's default shaping of the WHOLE alternative, and `$1` would silently drop
            // everything after the leading reference. Refuse — a wrong AST is the defect `.8`
            // exists to close, and "the grammars we ship annotate this hop" is not a proof.
            None => {
                return Err(format!(
                    "hop '{}' alternative {} declares no return annotation and its residual is \
                     '{}', so the chain's AST cannot be composed faithfully — annotate it",
                    step.rule,
                    step.alternative_index,
                    super::indirect_lr_plan::render_elements(&step.residual)
                ));
            }
        };
        composed = substitute_hop(&template, &composed, offsets[index], &step.rule)?;
    }
    Ok(composed)
}

/// The `@profiles:` gate a route inherits, and its rendered list.
///
/// ⛔ **WHY A ROUTE HAS A PROFILE AT ALL.** A route's derivation exists only under the profiles
/// where EVERY rule on it exists, so the route's constraint is the intersection of the declared
/// `@profiles` lists along it. SystemVerilog's cast/call knot runs through `constant_primary_sv_2017`
/// (`["sv_2017", "verilog_2005"]`) or `constant_primary_sv_2023` (`["sv_2023"]`), and the two routes
/// iterate byte-identical suffixes — so without the gate the sv_2017 branch would serve an sv_2023
/// parse and stamp `kind: "sv_2017"` on its AST.
///
/// ⭐ The gate is **copied verbatim** from the rule on the route whose declared list IS the
/// intersection, never synthesized: a constructed annotation would be a second spelling of a
/// directive whose only authoritative spelling is the grammar's. When no rule's list equals the
/// intersection the route is refused, which is rare by construction — dialect gates nest.
fn route_profile_gate(
    route: &ChainRoute,
    annotations: Option<&Annotations>,
) -> Result<RouteGate, String> {
    let Some(annotations) = annotations else {
        return Ok(RouteGate::Unconstrained);
    };
    // (rule, declared list, the annotation itself) for every hop that declares one.
    let mut declared: Vec<(&str, Vec<String>, SemanticAnnotation)> = Vec::new();
    for step in &route.steps {
        for entry in annotations
            .semantic_annotations
            .get(&step.rule)
            .into_iter()
            .flatten()
        {
            if !is_profiles_annotation(entry) {
                continue;
            }
            let Some(list) =
                super::semantic_directive_registry::parse_semantic_string_list(
                    entry.ast().payload_text(),
                )
            else {
                continue;
            };
            let profiles: Vec<String> = list
                .into_iter()
                .map(|value| value.trim().to_ascii_lowercase())
                .filter(|value| !value.is_empty())
                .collect();
            if !profiles.is_empty() {
                declared.push((step.rule.as_str(), profiles, entry.clone()));
            }
        }
    }
    if declared.is_empty() {
        return Ok(RouteGate::Unconstrained);
    }

    let mut intersection: BTreeSet<String> = declared[0].1.iter().cloned().collect();
    for (_, profiles, _) in declared.iter().skip(1) {
        let other: BTreeSet<String> = profiles.iter().cloned().collect();
        intersection = intersection.intersection(&other).cloned().collect();
    }
    // ⭐⭐ AN EMPTY INTERSECTION IS A ROUTE NO PROFILE CAN DERIVE, AND SUCH ROUTES EXIST — the route
    // walk reads the grammar structurally and knows nothing about `@profiles`. SystemVerilog has
    // four of them, each splicing an `sv_2017` constant primary onto an `sv_2023` method-call
    // receiver (or the mirror). ⛔ The right handling is NOT to refuse the candidate: the route's
    // edges must still be SHEARED (the structural cycle is real, and the lint that verifies this
    // pass is profile-blind too), but it must contribute NO suffix branch, because a branch for a
    // derivation no profile admits is an over-acceptance wearing a chain's clothes.
    if intersection.is_empty() {
        return Ok(RouteGate::Underivable);
    }
    // Copy the annotation whose declared list IS the intersection.
    let chosen = declared.iter().find(|(_, profiles, _)| {
        profiles.iter().cloned().collect::<BTreeSet<String>>() == intersection
    });
    match chosen {
        Some((_, profiles, annotation)) => Ok(RouteGate::Gated {
            annotation: annotation.clone(),
            profiles: profiles.clone(),
        }),
        None => Err(format!(
            "the route's @profiles intersection is {:?}, which no single rule on the route \
             declares, so the gate cannot be copied verbatim",
            intersection.iter().collect::<Vec<_>>()
        )),
    }
}

/// Is this the rule-level `@profiles:` directive?
fn is_profiles_annotation(entry: &SemanticAnnotation) -> bool {
    entry
        .name()
        .map(|name| name.trim().eq_ignore_ascii_case("profiles"))
        .unwrap_or(false)
}

/// What the profiles on a route's rules say about the route.
#[derive(Debug, Clone)]
enum RouteGate {
    /// No rule on the route declares `@profiles:` — the route is universal.
    Unconstrained,
    /// The route derives only under `profiles`, gated by `annotation` (copied verbatim).
    Gated {
        annotation: SemanticAnnotation,
        profiles: Vec<String>,
    },
    /// The route's rules have DISJOINT profile lists, so no profile can derive it. Its edges are
    /// still sheared; it contributes no suffix branch.
    Underivable,
}

/// Per-hop offset into the flattened suffix: the total residual length of every hop DEEPER than
/// this one, because [`ChainRoute::suffix_elements`] concatenates innermost-first.
fn suffix_offsets(route: &ChainRoute) -> Vec<usize> {
    let mut offsets = vec![0usize; route.steps.len()];
    let mut running = 0usize;
    for index in (0..route.steps.len()).rev() {
        offsets[index] = running;
        running += route.steps[index].residual.len();
    }
    offsets
}

/// One hop's template with `$1` bound to `inner` and `$m` (`m >= 2`) remapped to the flattened
/// suffix position the hop's residual element landed on.
///
/// A hop's body is `[<leading rule reference>, ...residual]`, so `$m` names `residual[m - 2]`,
/// which sits at flattened position `offset + m - 1`. The fold reads flattened position `p` as
/// `$(p + 1)` (`$1` being the running value), so the remapped index is `offset + m`.
fn substitute_hop(
    template: &UnifiedReturnAST,
    inner: &UnifiedReturnAST,
    offset: usize,
    rule: &str,
) -> Result<UnifiedReturnAST, String> {
    let recurse = |node: &UnifiedReturnAST| substitute_hop(node, inner, offset, rule);
    Ok(match template {
        UnifiedReturnAST::PositionalRef { index } => match index {
            0 => {
                return Err(format!(
                    "hop '{rule}' references $0 / $text, which the chain fold cannot express"
                ));
            }
            1 => inner.clone(),
            other => UnifiedReturnAST::PositionalRef {
                index: offset + other,
            },
        },
        UnifiedReturnAST::Passthrough => inner.clone(),
        literal @ (UnifiedReturnAST::StringLiteral { .. }
        | UnifiedReturnAST::NumberLiteral { .. }
        | UnifiedReturnAST::BooleanLiteral { .. }
        | UnifiedReturnAST::NullLiteral
        | UnifiedReturnAST::Identifier { .. }) => literal.clone(),
        UnifiedReturnAST::Object { properties } => {
            let mut rewritten = HashMap::with_capacity(properties.len());
            for (key, value) in properties {
                rewritten.insert(key.clone(), Box::new(recurse(value)?));
            }
            UnifiedReturnAST::Object {
                properties: rewritten,
            }
        }
        UnifiedReturnAST::Array { elements } => UnifiedReturnAST::Array {
            elements: elements
                .iter()
                .map(recurse)
                .collect::<Result<Vec<_>, _>>()?,
        },
        UnifiedReturnAST::Spread { base } => UnifiedReturnAST::Spread {
            base: Box::new(recurse(base)?),
        },
        UnifiedReturnAST::FlattenSpread { base } => UnifiedReturnAST::FlattenSpread {
            base: Box::new(recurse(base)?),
        },
        UnifiedReturnAST::PropertyAccess { base, property } => UnifiedReturnAST::PropertyAccess {
            base: Box::new(recurse(base)?),
            property: property.clone(),
        },
        UnifiedReturnAST::ArrayAccess { base, index } => UnifiedReturnAST::ArrayAccess {
            base: Box::new(recurse(base)?),
            index: Box::new(recurse(index)?),
        },
        // Refused here rather than at `validate_chain_templates`, so the message names the HOP the
        // construct came from — the composed template no longer knows.
        UnifiedReturnAST::MatchedText => {
            return Err(format!(
                "hop '{rule}' uses $text, which the chain fold cannot express"
            ));
        }
        UnifiedReturnAST::QuantifiedExtraction { .. } => {
            return Err(format!(
                "hop '{rule}' uses $N::target (quantified extraction), which the chain fold cannot \
                 express"
            ));
        }
        UnifiedReturnAST::LrChainFold { .. } => {
            return Err(format!(
                "hop '{rule}' already carries a left-recursion chain; nesting two chains is refused"
            ));
        }
    })
}

/// A rule's per-branch annotations, in the three kinds the pipeline keys separately: return,
/// semantic, and mid-sequence semantic. Index-aligned with the rule's alternatives.
type BranchAnnotationTriple = (
    Vec<Option<BranchAnnotation>>,
    Vec<Vec<SemanticAnnotation>>,
    Vec<Vec<MidSequenceSemanticAnnotation>>,
);

/// A rule's three per-branch annotation vectors, padded to `alternative_count` so index arithmetic
/// over them never depends on whether the author annotated the last branch.
fn branch_annotations_of(
    rule: &str,
    alternative_count: usize,
    annotations: Option<&Annotations>,
) -> BranchAnnotationTriple {
    let mut branch_return = vec![None; alternative_count];
    let mut branch_semantic = vec![Vec::new(); alternative_count];
    let mut branch_mid = vec![Vec::new(); alternative_count];
    if let Some(annotations) = annotations {
        if let Some(existing) = annotations.branch_return_annotations.get(rule) {
            for (index, entry) in existing.iter().take(alternative_count).enumerate() {
                branch_return[index] = entry.clone();
            }
        }
        if let Some(existing) = annotations.branch_semantic_annotations.get(rule) {
            for (index, entry) in existing.iter().take(alternative_count).enumerate() {
                branch_semantic[index] = entry.clone();
            }
        }
        if let Some(existing) = annotations
            .branch_mid_sequence_semantic_annotations
            .get(rule)
        {
            for (index, entry) in existing.iter().take(alternative_count).enumerate() {
                branch_mid[index] = entry.clone();
            }
        }
    }
    (branch_return, branch_semantic, branch_mid)
}

/// Claim a synthetic rule name, appending `_1`, `_2`, … on collision, and record the claim so this
/// plan cannot hand the same name out twice.
fn allocate(preferred: String, taken: &mut BTreeSet<String>) -> String {
    if taken.insert(preferred.clone()) {
        return preferred;
    }
    let mut index = 1usize;
    loop {
        let candidate = format!("{preferred}_{index}");
        if taken.insert(candidate.clone()) {
            return candidate;
        }
        index += 1;
    }
}

/// Write the plan into the grammar.
fn apply_plan(
    plan: &EliminationPlan,
    grammar_tree: &mut HashMap<String, ASTNode>,
    rule_order: &mut Vec<String>,
    annotations: Option<&mut Annotations>,
) {
    let want_chain_annotations = annotations.is_some();

    // ---- Clones first: nothing else references them until the base rule below does.
    for clone in &plan.clones {
        grammar_tree.insert(clone.name.clone(), clone.body.clone());
        insert_before(rule_order, &plan.base_rule, &clone.name);
    }

    // ---- `X_lr_base`.
    grammar_tree.insert(
        plan.helper_base_rule.clone(),
        RustASTPipeline::build_or_node(plan.base_alternatives.clone()),
    );
    insert_before(rule_order, &plan.base_rule, &plan.helper_base_rule);

    // ---- `X_lr_suffix`. With annotations in scope each route gets its OWN rule so it can carry a
    // `@profiles:` gate (an alternative cannot); `X_lr_suffix` is then the ordered choice over
    // those references. Without annotations there is nothing to gate or fold, so the legacy inline
    // Or the direct pass uses is kept exactly.
    let suffix_element = if want_chain_annotations {
        for branch in &plan.suffix_branches {
            grammar_tree.insert(
                branch.rule_name.clone(),
                RustASTPipeline::build_sequence_node(branch.elements.clone()),
            );
            insert_before(rule_order, &plan.base_rule, &branch.rule_name);
        }
        grammar_tree.insert(
            plan.helper_suffix_rule.clone(),
            RustASTPipeline::build_or_node(
                plan.suffix_branches
                    .iter()
                    .map(|branch| RustASTPipeline::make_rule_reference_node(&branch.rule_name))
                    .collect(),
            ),
        );
        insert_before(rule_order, &plan.base_rule, &plan.helper_suffix_rule);
        RustASTPipeline::make_rule_reference_node(&plan.helper_suffix_rule)
    } else {
        RustASTPipeline::build_or_node(
            plan.suffix_branches
                .iter()
                .map(|branch| RustASTPipeline::build_sequence_node(branch.elements.clone()))
                .collect(),
        )
    };

    // ---- `X := X_lr_base ( X_lr_suffix )*`.
    grammar_tree.insert(
        plan.base_rule.clone(),
        ASTNode::Sequence {
            elements: vec![
                RustASTPipeline::make_rule_reference_node(&plan.helper_base_rule),
                ASTNode::Quantified {
                    element: Box::new(suffix_element.clone()),
                    quantifier: "*".to_string(),
                },
            ],
        },
    );

    // ---- `.17` slice 7 — the guarded clone chains, and the holder left corners they claim.
    //
    // ⭐ The guarded base rule is `X`'s body with the SAME two positions and one extra element, so
    // the chain fold sees the `$N` it always saw: `$1` is still `X_lr_base` and `$2` is still the
    // quantified suffix. That is why the loop guard is hoisted into `X_lr_guard{v}_suffix` instead
    // of being written inline as `( X_lr_suffix &( R ) )*` — an inline group would make the
    // quantifier iterate a SEQUENCE rather than the suffix record, and `$2` would stop being the
    // list `fold_lr_chain` consumes. The trailing guard needs no such care: it APPENDS, and a
    // lookahead contributes `ParseContent::Sequence(Vec::new())` at a position no template names.
    for chain in &plan.guard_chains {
        for clone in &chain.clones {
            grammar_tree.insert(clone.name.clone(), clone.body.clone());
            insert_before(rule_order, &plan.base_rule, &clone.name);
        }
        let lookahead = || ASTNode::Lookahead {
            element: Box::new(RustASTPipeline::build_sequence_node(chain.residual.clone())),
            positive: true,
        };
        let guarded_suffix_element = match &chain.guarded_suffix_rule {
            Some(rule) => {
                grammar_tree.insert(
                    rule.clone(),
                    ASTNode::Sequence {
                        elements: vec![suffix_element.clone(), lookahead()],
                    },
                );
                insert_before(rule_order, &plan.base_rule, rule);
                RustASTPipeline::make_rule_reference_node(rule)
            }
            None => suffix_element.clone(),
        };
        let mut guarded_elements = vec![
            RustASTPipeline::make_rule_reference_node(&plan.helper_base_rule),
            ASTNode::Quantified {
                element: Box::new(guarded_suffix_element),
                quantifier: "*".to_string(),
            },
        ];
        if chain.trailing_guard {
            guarded_elements.push(lookahead());
        }
        grammar_tree.insert(
            chain.guarded_base_rule.clone(),
            ASTNode::Sequence {
                elements: guarded_elements,
            },
        );
        insert_before(rule_order, &plan.base_rule, &chain.guarded_base_rule);
    }

    // The holder keeps every element but its left corner, so its own `$N` are untouched. Applied in
    // a SECOND pass over all chains, and to every guarded clone of the holder as well as to the
    // holder itself: a guarded clone is built from the original body, so a chain member that is also
    // a starvation holder would otherwise keep an unguarded arm to the same starvation. The
    // alternative index is shared because a guarded clone keeps every alternative, in order.
    let guarded_copies_of = |rule: &str| -> Vec<String> {
        plan.guard_chains
            .iter()
            .filter_map(|chain| chain.guarded_by_source.get(rule).cloned())
            .collect()
    };
    for chain in &plan.guard_chains {
        for redirect in &chain.redirects {
            let mut targets = vec![redirect.holder_rule.clone()];
            targets.extend(guarded_copies_of(&redirect.holder_rule));
            for rule in targets {
                let Some(body) = grammar_tree.get(&rule) else {
                    continue;
                };
                let mut alternatives = RustASTPipeline::as_alternatives(body);
                let Some(slot) = alternatives.get_mut(redirect.alternative_index) else {
                    continue;
                };
                *slot = replace_left_corner(slot, &redirect.target);
                grammar_tree.insert(rule, RustASTPipeline::build_or_node(alternatives));
            }
        }
    }

    let Some(annotations) = annotations else {
        return;
    };

    for clone in &plan.clones {
        install_branch_annotations(
            annotations,
            &clone.name,
            &clone.branch_return,
            &clone.branch_semantic,
            &clone.branch_mid_sequence,
        );
        if !clone.rule_semantic.is_empty() {
            annotations
                .semantic_annotations
                .insert(clone.name.clone(), clone.rule_semantic.clone());
        }
        if let Some(restriction) = &clone.lexical_follow_restriction {
            annotations
                .lexical_follow_restrictions
                .insert(clone.name.clone(), restriction.clone());
        }
    }
    install_branch_annotations(
        annotations,
        &plan.helper_base_rule,
        &plan.base_branch_return,
        &plan.base_branch_semantic,
        &plan.base_branch_mid_sequence,
    );
    // ⛔ The helpers inherit the BASE rule's own rule-level directives. A `@profiles:`-gated base
    // whose helpers are universal makes both helpers **profile ORPHANS** — present under a profile
    // where nothing they reference is satisfiable. Measured: gating
    // `incomplete_class_scoped_type_sv_2023` (`["sv_2023"]`) and leaving its two helpers universal
    // took `profile_orphans` 0 → 4, an ERROR-class lint counter.
    //
    // ⭐ `@profiles:` ONLY, deliberately. It is the one rule-level directive that says *whether the
    // rule exists*, so a helper that carries the base's productions must carry it too. Every other
    // rule-level directive (`@predicate:`, `@emit_fact:`, …) says what happens WHEN the rule runs,
    // and the base rule still runs — copying those would apply them twice per parse.
    let base_profiles: Vec<SemanticAnnotation> = annotations
        .semantic_annotations
        .get(&plan.base_rule)
        .into_iter()
        .flatten()
        .filter(|entry| is_profiles_annotation(entry))
        .cloned()
        .collect();
    if !base_profiles.is_empty() {
        annotations
            .semantic_annotations
            .insert(plan.helper_base_rule.clone(), base_profiles.clone());
        annotations
            .semantic_annotations
            .insert(plan.helper_suffix_rule.clone(), base_profiles.clone());
    }
    // The base rule's own per-branch annotations have moved to `X_lr_base`; its body is now a
    // Sequence with exactly one branch.
    annotations
        .branch_return_annotations
        .remove(&plan.base_rule);
    annotations
        .branch_semantic_annotations
        .remove(&plan.base_rule);
    annotations
        .branch_mid_sequence_semantic_annotations
        .remove(&plan.base_rule);

    let specs: Vec<LrChainWrapperSpec> = plan
        .suffix_branches
        .iter()
        .enumerate()
        .map(|(alt_index, branch)| LrChainWrapperSpec {
            alt_index,
            original_body_length: 1 + branch.elements.len(),
            annotation_template: branch.template.clone(),
        })
        .collect();
    let chain_fold_annotation = |specs: Vec<LrChainWrapperSpec>| BranchAnnotation {
        annotation_type: "_pgen_lr_chain_synthetic".to_string(),
        annotation_content: String::new(),
        parsed_ast: Some(UnifiedReturnAST::LrChainFold {
            initial: Box::new(UnifiedReturnAST::PositionalRef { index: 1 }),
            suffixes: Box::new(UnifiedReturnAST::PositionalRef { index: 2 }),
            specs,
        }),
    };
    annotations.branch_return_annotations.insert(
        plan.base_rule.clone(),
        vec![Some(chain_fold_annotation(specs.clone()))],
    );

    // Each route's own rule emits the `{alt_index, captures}` record the fold consumes — its `$N`
    // are that rule's own elements, so the capture indices stay the branch's own.
    for (alt_index, branch) in plan.suffix_branches.iter().enumerate() {
        let mut properties: HashMap<String, Box<UnifiedReturnAST>> = HashMap::new();
        properties.insert(
            "type".to_string(),
            Box::new(UnifiedReturnAST::StringLiteral {
                value: lr_chain_fold::CHAIN_ALT_TYPE_MARKER.to_string(),
            }),
        );
        properties.insert(
            "alt_index".to_string(),
            Box::new(UnifiedReturnAST::NumberLiteral {
                value: alt_index as f64,
            }),
        );
        properties.insert(
            "captures".to_string(),
            Box::new(UnifiedReturnAST::Array {
                elements: (1..=branch.elements.len())
                    .map(|index| UnifiedReturnAST::PositionalRef { index })
                    .collect(),
            }),
        );
        annotations.branch_return_annotations.insert(
            branch.rule_name.clone(),
            vec![Some(BranchAnnotation {
                annotation_type: "_pgen_lr_chain_synthetic".to_string(),
                annotation_content: String::new(),
                parsed_ast: Some(UnifiedReturnAST::Object { properties }),
            })],
        );
        if let Some(profiles) = &branch.profiles {
            annotations
                .semantic_annotations
                .insert(branch.rule_name.clone(), vec![profiles.clone()]);
        }
    }
    // `X_lr_suffix` is the ordered choice over those rules; each alternative is a bare reference,
    // so `-> $1` passes the record through unchanged.
    let pass_through = || BranchAnnotation {
        annotation_type: "_pgen_lr_chain_synthetic".to_string(),
        annotation_content: String::new(),
        parsed_ast: Some(UnifiedReturnAST::PositionalRef { index: 1 }),
    };
    annotations.branch_return_annotations.insert(
        plan.helper_suffix_rule.clone(),
        plan.suffix_branches
            .iter()
            .map(|_| Some(pass_through()))
            .collect(),
    );

    // ---- `.17` slice 7 — the guarded chain's annotations.
    //
    // ⭐ The rule-level split is the one `apply_plan` already draws for the helpers, applied to the
    // two KINDS of guarded rule this chain emits, and the two kinds sit on opposite sides of it:
    //
    // * a guarded CLONE (`X_lr_guard{v}_<hop>`, and the guarded base itself) STANDS IN for a rule at
    //   one call site, so it must carry that rule's directives IDENTICALLY — a `@predicate:` still
    //   runs exactly once, because the original is not also entered on this path;
    // * a guarded HELPER (`X_lr_guard{v}_suffix`) is a new sub-rule underneath the base, so it takes
    //   `@profiles:` ONLY, exactly as `X_lr_suffix` does — copying the rest would apply it twice.
    for chain in &plan.guard_chains {
        for clone in &chain.clones {
            install_branch_annotations(
                annotations,
                &clone.name,
                &clone.branch_return,
                &clone.branch_semantic,
                &clone.branch_mid_sequence,
            );
            if !clone.rule_semantic.is_empty() {
                annotations
                    .semantic_annotations
                    .insert(clone.name.clone(), clone.rule_semantic.clone());
            }
            if let Some(restriction) = &clone.lexical_follow_restriction {
                annotations
                    .lexical_follow_restrictions
                    .insert(clone.name.clone(), restriction.clone());
            }
        }
        // The guarded base rule IS the base rule at this call site: same body shape, same fold, same
        // rule-level directives.
        annotations.branch_return_annotations.insert(
            chain.guarded_base_rule.clone(),
            vec![Some(chain_fold_annotation(specs.clone()))],
        );
        if let Some(base_semantic) = annotations.semantic_annotations.get(&plan.base_rule).cloned() {
            annotations
                .semantic_annotations
                .insert(chain.guarded_base_rule.clone(), base_semantic);
        }
        if let Some(restriction) = annotations
            .lexical_follow_restrictions
            .get(&plan.base_rule)
            .cloned()
        {
            annotations
                .lexical_follow_restrictions
                .insert(chain.guarded_base_rule.clone(), restriction);
        }
        if let Some(guarded_suffix) = &chain.guarded_suffix_rule {
            // `X_lr_guard{v}_suffix := X_lr_suffix &( residual )` — `$1` is the suffix record and
            // the lookahead appends, so the same pass-through carries it to the fold unchanged.
            annotations
                .branch_return_annotations
                .insert(guarded_suffix.clone(), vec![Some(pass_through())]);
            if !base_profiles.is_empty() {
                annotations
                    .semantic_annotations
                    .insert(guarded_suffix.clone(), base_profiles.clone());
            }
        }
    }
}

fn install_branch_annotations(
    annotations: &mut Annotations,
    rule: &str,
    branch_return: &[Option<BranchAnnotation>],
    branch_semantic: &[Vec<SemanticAnnotation>],
    branch_mid_sequence: &[Vec<MidSequenceSemanticAnnotation>],
) {
    if branch_return.iter().any(|entry| entry.is_some()) {
        annotations
            .branch_return_annotations
            .insert(rule.to_string(), branch_return.to_vec());
    }
    if branch_semantic.iter().any(|entry| !entry.is_empty()) {
        annotations
            .branch_semantic_annotations
            .insert(rule.to_string(), branch_semantic.to_vec());
    }
    if branch_mid_sequence.iter().any(|entry| !entry.is_empty()) {
        annotations
            .branch_mid_sequence_semantic_annotations
            .insert(rule.to_string(), branch_mid_sequence.to_vec());
    }
}

/// Insert `name` immediately before `anchor` in `rule_order`, or append when the anchor is absent.
/// Order is part of the codegen's byte-determinism contract, so this never touches a HashMap.
fn insert_before(rule_order: &mut Vec<String>, anchor: &str, name: &str) {
    if rule_order.iter().any(|existing| existing == name) {
        return;
    }
    match rule_order.iter().position(|existing| existing == anchor) {
        Some(position) => rule_order.insert(position, name.to_string()),
        None => rule_order.push(name.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast_pipeline::indirect_lr_plan::render_elements;
    use crate::ast_pipeline::{ASTValue, TokenValue};

    fn rule(name: &str) -> ASTNode {
        ASTNode::Atom {
            value: ASTValue::Token(vec![
                TokenValue::String("rule_reference".to_string()),
                TokenValue::String(name.to_string()),
            ]),
        }
    }

    fn text(literal: &str) -> ASTNode {
        ASTNode::Atom {
            value: ASTValue::Token(vec![
                TokenValue::String("quoted_string".to_string()),
                TokenValue::String(literal.to_string()),
            ]),
        }
    }

    fn object(pairs: Vec<(&str, UnifiedReturnAST)>) -> UnifiedReturnAST {
        UnifiedReturnAST::Object {
            properties: pairs
                .into_iter()
                .map(|(key, value)| (key.to_string(), Box::new(value)))
                .collect(),
        }
    }

    fn positional(index: usize) -> UnifiedReturnAST {
        UnifiedReturnAST::PositionalRef { index }
    }

    fn literal(value: &str) -> UnifiedReturnAST {
        UnifiedReturnAST::StringLiteral {
            value: value.to_string(),
        }
    }

    fn annotation(ast: UnifiedReturnAST) -> Option<BranchAnnotation> {
        Some(BranchAnnotation {
            annotation_type: "return_annotation".to_string(),
            annotation_content: String::new(),
            parsed_ast: Some(ast),
        })
    }

    /// `p4_knot_a_annotated.ebnf`, rule for rule — SystemVerilog knot A's shape with the
    /// annotations SystemVerilog declares on the same four hops.
    fn knot_a_annotated() -> (HashMap<String, ASTNode>, Vec<String>, Annotations) {
        let mut grammar = HashMap::new();
        grammar.insert("scratch".to_string(), rule("prim"));
        grammar.insert(
            "prim".to_string(),
            ASTNode::Or {
                alternatives: vec![rule("lit"), rule("cast_expr")],
            },
        );
        grammar.insert(
            "cast_expr".to_string(),
            ASTNode::Sequence {
                elements: vec![rule("ct"), text("'"), text("("), rule("lit"), text(")")],
            },
        );
        grammar.insert(
            "ct".to_string(),
            ASTNode::Or {
                alternatives: vec![rule("kw"), rule("prim")],
            },
        );
        grammar.insert("lit".to_string(), text("n"));
        grammar.insert("kw".to_string(), text("t"));
        let order: Vec<String> = ["scratch", "prim", "cast_expr", "ct", "lit", "kw"]
            .iter()
            .map(|name| name.to_string())
            .collect();

        let mut annotations = Annotations::default();
        annotations.branch_return_annotations.insert(
            "prim".to_string(),
            vec![
                annotation(object(vec![("kind", literal("lit")), ("body", positional(1))])),
                annotation(object(vec![("kind", literal("cast")), ("body", positional(1))])),
            ],
        );
        annotations.branch_return_annotations.insert(
            "cast_expr".to_string(),
            vec![annotation(object(vec![
                ("type", positional(1)),
                ("body", positional(4)),
            ]))],
        );
        annotations.branch_return_annotations.insert(
            "ct".to_string(),
            vec![
                annotation(object(vec![("kind", literal("kw")), ("body", positional(1))])),
                annotation(object(vec![("kind", literal("prim")), ("body", positional(1))])),
            ],
        );
        (grammar, order, annotations)
    }

    /// `p5_transparent_holder.ebnf` with knot A's annotations: [`knot_a_annotated`] plus an OUTSIDE
    /// holder of the transparent rule, so the holder OUTLIVES any rewrite and every candidate on the
    /// cycle is starved. ⇒ the shipped driver has nothing to admit here at all, which is what makes
    /// it the discriminating fixture for the dry run.
    fn knot_a_annotated_with_surviving_holder()
    -> (HashMap<String, ASTNode>, Vec<String>, Annotations) {
        let (mut grammar, mut order, annotations) = knot_a_annotated();
        grammar.insert(
            "scratch".to_string(),
            ASTNode::Or {
                alternatives: vec![rule("outer_cast"), rule("prim")],
            },
        );
        grammar.insert(
            "outer_cast".to_string(),
            ASTNode::Sequence {
                elements: vec![rule("ct"), text("'"), text("("), rule("lit"), text(")")],
            },
        );
        order.insert(1, "outer_cast".to_string());
        (grammar, order, annotations)
    }

    /// Serialize-compare, because neither `ASTNode` nor `Annotations` derives `PartialEq` — and a
    /// dry run that quietly mutated its caller's grammar is precisely the failure this must catch.
    fn frozen<T: serde::Serialize>(value: &T) -> String {
        serde_json::to_string(value).expect("serializable")
    }

    /// ⭐⭐ `.17` slice 3 — the dry run reaches a candidate the shipped pass never considers, and
    /// leaves the caller's grammar untouched while doing it.
    ///
    /// ⛔ RED-provable on the property that matters: point the dry run at
    /// `CandidateAdmission::StarvationSafe` and the first assertion fails by name — it would report
    /// nothing, exactly like the shipped pass.
    ///
    /// ⛔ **The three `frozen` assertions are NOT falsifiable today, and saying so is the point.**
    /// [`dry_run_guard_feasible_elimination`] takes `&HashMap` / `&[String]` / `&Annotations`, so
    /// the compiler already forbids what they check; they cannot fail while that signature holds.
    /// They are a tripwire against a future refactor that widens those borrows for convenience —
    /// the moment one becomes `&mut`, the safety argument moves from the type system into these
    /// three lines, and they start doing real work. A test that can only pass is documentation
    /// ([[a-check-whose-inputs-all-pass-has-not-been-tested]]); this one is labelled as such rather
    /// than counted as evidence.
    #[test]
    fn the_guard_dry_run_reaches_what_the_shipped_pass_refuses_without_touching_the_grammar() {
        let (mut grammar, mut order, mut annotations) = knot_a_annotated_with_surviving_holder();
        let frozen_grammar = frozen(&grammar);
        let frozen_order = frozen(&order);
        let frozen_annotations = frozen(&annotations);

        let dry = dry_run_guard_feasible_elimination(&grammar, &order, Some(&annotations));
        assert!(
            !dry.outcome.eliminated_base_rules.is_empty() || !dry.outcome.refusals.is_empty(),
            "the dry run must reach the PLAN stage on a guard-feasible candidate — an empty \
             outcome here means it saw the same empty admission set the shipped pass does, and \
             the instrument would be measuring nothing"
        );
        // ⛔ The payoff figure is MEASURED on the rewritten clone, not inferred from the absorb
        // count: this knot has two candidates and one rewrite at the dominator clears both rows.
        assert!(
            dry.cycle_rows_after < dry.cycle_rows_before,
            "a dry run that absorbed something must leave fewer left-recursive rows ({} -> {})",
            dry.cycle_rows_before,
            dry.cycle_rows_after
        );

        // ⛔ The whole safety argument of the instrument: it works on a clone.
        assert_eq!(frozen(&grammar), frozen_grammar, "dry run mutated the grammar tree");
        assert_eq!(frozen(&order), frozen_order, "dry run mutated the rule order");
        assert_eq!(
            frozen(&annotations),
            frozen_annotations,
            "dry run mutated the annotations"
        );

        // ⭐ The one-difference control, on the SAME grammar: the shipped policy is a strict no-op
        // here. Without this half, the assertion above could be satisfied by a dry run that merely
        // repeats what the pass already does.
        let shipped =
            eliminate_indirect_left_recursion(&mut grammar, &mut order, Some(&mut annotations));
        assert!(
            shipped.eliminated_base_rules.is_empty() && shipped.refusals.is_empty(),
            "every candidate on this knot is STARVED, so the shipped driver admits none of them — \
             it absorbed {:?} and refused {:?}",
            shipped.eliminated_base_rules,
            shipped.refusals
        );
        assert_eq!(
            frozen(&grammar),
            frozen_grammar,
            "a pass that admits nothing must also change nothing"
        );
    }

    /// ⭐⭐ `.17` slice 7 — the emitted guard is `guard_effectiveness/g7_guarded_clone_chain.ebnf`,
    /// rule for rule.
    ///
    /// ⛔⛔ **This fixture is not a convenience — it is the ONLY thing in the repository that
    /// exercises the hop-clone half of the emitter.** Measured on the shipped grammar this session:
    /// every chain the SystemVerilog dry run synthesizes has `max_hops=0` (the holders `cast`,
    /// `constant_cast` and `prop_primary_*` name their base rule directly), so the corpus-scale run
    /// walks past `X_lr_guard{v}_<hop>` entirely. P5's `outer_cast → ct → prim` is one hop, and one
    /// hop is what separates "the guard is on a clone" from "the guard is on the shared rule" —
    /// slice 4's `g4`, which REJECTS `k = n;`.
    ///
    /// The six assertions below are the bank's own mapping table, in the same order:
    ///
    /// ```text
    /// outer_cast := ct_guard tick lparen lit rparen   ← only the LEFT CORNER moved
    /// ct         := kw | prim                         ← the ORIGINAL is untouched
    /// prim       := prim_base ( prim_suffix )*        ← likewise
    /// ct_guard   := kw | prim_guard                   ← the non-chain arm copied VERBATIM
    /// prim_guard := prim_base ( prim_suffix &( R ) )* &( R )
    /// ```
    #[test]
    fn the_guarded_clone_chain_reproduces_the_hand_written_g7_shape() {
        let (mut grammar, mut order, mut annotations) = knot_a_annotated_with_surviving_holder();
        let outcome = eliminate_indirect_left_recursion_with_admission(
            &mut grammar,
            &mut order,
            Some(&mut annotations),
            CandidateAdmission::GuardFeasibleDryRun,
        );

        assert_eq!(
            outcome.synthesized_guards.len(),
            1,
            "one residual, one position set, one chain — got {:?}",
            outcome.synthesized_guards
        );
        let guard = &outcome.synthesized_guards[0];
        assert_eq!(guard.base_rule, "prim");
        assert_eq!(guard.chain, vec!["ct".to_string(), "prim".to_string()]);
        // ⭐ BOTH positions, and the pair is the whole of slice 4's decision (c): the loop guard
        // alone leaves `e5` starved, the trailing guard alone leaves `e1` starved.
        assert_eq!(guard.positions, "loop+trailing");
        assert_eq!(guard.residual, "\"'\" \"(\" lit \")\"");
        // ⛔ `cast_expr` holds the identical residual and is deliberately NOT here: it goes dead
        // with the rewrite (nothing outside the plan names it), so guarding it would synthesize a
        // chain nothing can reach.
        assert_eq!(guard.call_sites, vec!["outer_cast alt#0".to_string()]);

        // ---- the holder: left corner repointed, every other element preserved.
        assert_eq!(
            alternatives(&grammar, "outer_cast"),
            vec!["prim_lr_guard0_ct \"'\" \"(\" lit \")\"".to_string()]
        );
        // ---- the ORIGINALS, untouched. This is what a residual-FREE holder still reaches, and it
        // is the one difference between `g7` and slice 4's `g4`.
        assert_eq!(
            alternatives(&grammar, "ct"),
            vec!["kw".to_string(), "prim".to_string()]
        );
        assert_eq!(
            alternatives(&grammar, "prim"),
            vec!["prim_lr_base prim_lr_suffix*".to_string()]
        );
        // ---- the guarded chain. The hop clone keeps `kw` VERBATIM, and that copy is what the
        // tournament falls back to when the trailing guard refuses an over-long seed.
        assert_eq!(
            alternatives(&grammar, "prim_lr_guard0_ct"),
            vec!["kw".to_string(), "prim_lr_guard0".to_string()]
        );
        assert_eq!(
            alternatives(&grammar, "prim_lr_guard0"),
            vec!["prim_lr_base prim_lr_guard0_suffix* &\"'\" \"(\" lit \")\"".to_string()]
        );
        assert_eq!(
            alternatives(&grammar, "prim_lr_guard0_suffix"),
            vec!["prim_lr_suffix &\"'\" \"(\" lit \")\"".to_string()]
        );

        // ---- the AST contract: the guarded base rule folds exactly as the base rule does, so the
        // holder's `$1` is the same value whichever of the two it reached.
        let base_fold = annotations
            .branch_return_annotations
            .get("prim")
            .and_then(|branches| branches.first().cloned())
            .flatten()
            .and_then(|branch| branch.parsed_ast);
        let guarded_fold = annotations
            .branch_return_annotations
            .get("prim_lr_guard0")
            .and_then(|branches| branches.first().cloned())
            .flatten()
            .and_then(|branch| branch.parsed_ast);
        assert!(base_fold.is_some(), "the base rule must carry the chain fold");
        assert_eq!(
            frozen(&guarded_fold),
            frozen(&base_fold),
            "the guarded base rule must fold identically to the base rule — a different AST at one \
             call site is the silent-wrong-AST defect `.8` exists to close"
        );
        // The guarded suffix wrapper passes the suffix record through unchanged; the lookahead
        // appends and names no `$N`.
        assert_eq!(
            frozen(
                &annotations
                    .branch_return_annotations
                    .get("prim_lr_guard0_suffix")
                    .and_then(|branches| branches.first().cloned())
                    .flatten()
                    .and_then(|branch| branch.parsed_ast)
            ),
            frozen(&Some(UnifiedReturnAST::PositionalRef { index: 1 }))
        );
        // The hop clone keeps the hop's OWN per-branch annotations, index-aligned — every
        // alternative survives into a guarded clone, unlike a sheared one.
        assert_eq!(
            frozen(&annotations.branch_return_annotations.get("prim_lr_guard0_ct")),
            frozen(&annotations.branch_return_annotations.get("ct")),
            "a guarded clone stands in for the hop, so it must return what the hop returns"
        );
    }

    /// [`knot_a_annotated_with_surviving_holder`] with the transparent hop SPLIT IN TWO, which is
    /// SystemVerilog's dialect-twin shape (`primary` reaches `cast` through `primary_sv_2017` **and**
    /// `primary_sv_2023`) reduced to its skeleton.
    ///
    /// `ct := kw | pa | pb` with `pa := prim` and `pb := prim`: both arms are bare references, so
    /// both are transparent at the same depth and the guarded chain has to clone `ct`, `pa`, `pb`
    /// AND the base — four rules where the SHORTEST transparency distance is two hops.
    fn knot_a_annotated_with_branching_transparency()
    -> (HashMap<String, ASTNode>, Vec<String>, Annotations) {
        let (mut grammar, mut order, mut annotations) = knot_a_annotated_with_surviving_holder();
        grammar.insert(
            "ct".to_string(),
            ASTNode::Or {
                alternatives: vec![rule("kw"), rule("pa"), rule("pb")],
            },
        );
        grammar.insert("pa".to_string(), rule("prim"));
        grammar.insert("pb".to_string(), rule("prim"));
        let ct_position = order.iter().position(|name| name == "ct").expect("ct is ordered");
        order.insert(ct_position + 1, "pa".to_string());
        order.insert(ct_position + 2, "pb".to_string());
        // `ct` grew a third alternative, so its per-branch annotations must stay index-aligned or
        // the clone would carry the wrong one — the same trap `branch_annotations_of` exists for.
        //
        // ⛔ Both transparent arms declare the SAME AST, and that is required rather than tidy: the
        // two routes through them iterate a syntactically IDENTICAL suffix under no profile gate, so
        // arms with different templates hit `plan_elimination`'s ambiguity refusal (*"two routes
        // iterate the identical suffix … but declare different ASTs"*) and no plan — hence no guard
        // — is ever built. Found by pointing `--report-indirect-lr-plan` at this shape rather than
        // by reading the planner. SystemVerilog's real twins avoid it with `@profiles:`, which is
        // exactly what `SuffixBranch::profile_key` exists to carry.
        annotations.branch_return_annotations.insert(
            "ct".to_string(),
            vec![
                annotation(object(vec![("kind", literal("kw")), ("body", positional(1))])),
                annotation(object(vec![("kind", literal("prim")), ("body", positional(1))])),
                annotation(object(vec![("kind", literal("prim")), ("body", positional(1))])),
            ],
        );
        for name in ["pa", "pb"] {
            annotations.branch_return_annotations.insert(
                name.to_string(),
                vec![annotation(positional(1))],
            );
        }
        (grammar, order, annotations)
    }

    /// ⭐⭐ `.17` slice 7 — a BRANCHING transparent chain clones every arm, and `guard_hops` does not
    /// count them.
    ///
    /// ⛔⛔ **This test exists because the invariant its own slice first documented was FALSE.** The
    /// JSON comment claimed `guard_hops == guard_chain.len() - 1` "by construction"; sweeping the
    /// shipped grammars for it found **6 of 129** SystemVerilog sites and **5 of 77** wrapper sites
    /// where it does not hold, every one of them a dialect-twin split. `guard_hops` is the SHORTEST
    /// transparency distance and the chain is the SET of rules on any transparent path — so a guard
    /// priced from `max_hops` under-counts the rules it will clone, exactly as `guard_variants`
    /// under-counts the chains (RESULT 2). Neither the SystemVerilog dry run nor the P5 fixture
    /// reaches a branching chain, so without this fixture the code path that repoints BOTH arms was
    /// unexecuted.
    #[test]
    fn a_branching_transparent_chain_clones_every_arm_and_guard_hops_undercounts_them() {
        let (mut grammar, mut order, mut annotations) =
            knot_a_annotated_with_branching_transparency();

        // The survey first: the chain is a SET, and it is strictly larger than hops + 1.
        let survey = survey_indirect_left_recursion(&grammar, &order);
        let candidate = survey
            .candidates
            .iter()
            .find(|candidate| candidate.base_rule == "prim")
            .expect("prim is a candidate on this knot");
        let site = candidate
            .surviving_starvation_sites()
            .into_iter()
            .find(|site| site.rule == "outer_cast")
            .expect("the outside holder survives the rewrite");
        assert_eq!(
            site.guard_chain,
            vec![
                "ct".to_string(),
                "pa".to_string(),
                "pb".to_string(),
                "prim".to_string()
            ],
            "both transparent arms belong to the chain — cloning one leaves an unguarded path"
        );
        assert!(
            site.guard.guard_hops < site.guard_chain.len() - 1,
            "the branching case is the one where hops UNDER-counts the clones: hops={} chain={:?}",
            site.guard.guard_hops,
            site.guard_chain
        );

        // Then the emission: every arm is cloned, and the branching hop repoints BOTH.
        let outcome = eliminate_indirect_left_recursion_with_admission(
            &mut grammar,
            &mut order,
            Some(&mut annotations),
            CandidateAdmission::GuardFeasibleDryRun,
        );
        let guard = outcome
            .synthesized_guards
            .iter()
            .find(|guard| guard.base_rule == "prim")
            .expect("the starved holder gets a guard chain");
        assert_eq!(
            guard.rules,
            vec![
                "prim_lr_guard0_ct".to_string(),
                "prim_lr_guard0_pa".to_string(),
                "prim_lr_guard0_pb".to_string(),
                "prim_lr_guard0_suffix".to_string(),
                "prim_lr_guard0".to_string(),
            ],
            "every rule on the branching chain needs its own guarded clone"
        );
        assert_eq!(
            alternatives(&grammar, "prim_lr_guard0_ct"),
            vec![
                "kw".to_string(),
                "prim_lr_guard0_pa".to_string(),
                "prim_lr_guard0_pb".to_string()
            ],
            "the branching hop must repoint BOTH arms — one repointed arm leaves an unguarded route \
             to the same starvation"
        );
        assert_eq!(
            alternatives(&grammar, "prim_lr_guard0_pa"),
            vec!["prim_lr_guard0".to_string()]
        );
        assert_eq!(
            alternatives(&grammar, "prim_lr_guard0_pb"),
            vec!["prim_lr_guard0".to_string()]
        );
        // The originals stay untouched, which is the whole call-site-scoping property.
        assert_eq!(
            alternatives(&grammar, "ct"),
            vec!["kw".to_string(), "pa".to_string(), "pb".to_string()]
        );
    }

    /// ⛔ `.17` slice 7 — the SHIPPED path synthesizes no guard, and it is a CONSEQUENCE of the
    /// admission criterion rather than a flag.
    ///
    /// The one-difference pair is the admission mode: the same fixture, the same planner, the same
    /// call — `GuardFeasibleDryRun` emits one chain (the test above), `StarvationSafe` emits none.
    /// A control that could only pass would be one that ran the planner on a grammar with no
    /// starvation at all; this one runs it on the knot where every candidate IS starved.
    #[test]
    fn the_shipped_admission_synthesizes_no_guard_on_the_same_starved_knot() {
        let (mut grammar, mut order, mut annotations) = knot_a_annotated_with_surviving_holder();
        let shipped =
            eliminate_indirect_left_recursion(&mut grammar, &mut order, Some(&mut annotations));
        assert!(
            shipped.synthesized_guards.is_empty(),
            "the shipped admission admits only starvation-SAFE candidates, which by definition have \
             no site to guard — it emitted {:?}",
            shipped.synthesized_guards
        );
        assert!(
            !grammar.keys().any(|name| name.contains("_lr_guard")),
            "no guard rule may reach a shipped grammar"
        );
    }

    /// ⛔ `.17` slice 7 — a starvation site the guard cannot close REFUSES the whole plan.
    ///
    /// Emitting guards for the closable sites and leaving one starved is a rewrite that changes the
    /// grammar and does not fix it, which this module's standing posture rates strictly worse than
    /// declining. Forced here by poisoning the loop verdict to `guard_incomplete` — the outcome
    /// `assess_guard` returns for a competing suffix that is not contained in the residual.
    #[test]
    fn a_site_the_guard_cannot_close_refuses_the_plan_rather_than_half_guarding_it() {
        let (grammar, order, annotations) = knot_a_annotated_with_surviving_holder();
        let survey = survey_indirect_left_recursion(&grammar, &order);
        let mut candidate = survey
            .candidates
            .iter()
            .find(|candidate| candidate.base_rule == "prim")
            .expect("prim is a candidate on this knot")
            .clone();
        let poisoned = candidate
            .starvation_sites
            .iter_mut()
            .find(|site| site.survives_rewrite)
            .expect("the surviving holder is a site");
        poisoned.guard.verdict = GuardVerdict::Incomplete;

        let error = plan_guard_chains(
            &candidate,
            &grammar,
            Some(&annotations),
            &mut BTreeSet::new(),
        )
        .expect_err("a guard-blocking site must refuse the plan");
        assert!(
            error.contains("guard_incomplete"),
            "the refusal must name the verdict that caused it — got '{error}'"
        );
    }

    fn alternatives(grammar: &HashMap<String, ASTNode>, name: &str) -> Vec<String> {
        RustASTPipeline::as_alternatives(grammar.get(name).expect("rule exists"))
            .iter()
            .map(|alternative| render_elements(std::slice::from_ref(alternative)))
            .collect()
    }

    /// The whole rewrite, asserted against `p3_eliminated_at_consumer_rule.ebnf` — the shape slice 3
    /// measured accepting all five probe inputs on both oracles, hand-written. The engine now
    /// derives it.
    #[test]
    fn the_rewrite_reproduces_the_hand_written_p3_shape() {
        let (mut grammar, mut order, mut annotations) = knot_a_annotated();
        let outcome =
            eliminate_indirect_left_recursion(&mut grammar, &mut order, Some(&mut annotations));

        assert_eq!(outcome.eliminated_base_rules, vec!["prim"]);
        assert_eq!(outcome.refusals, vec![]);
        // ⭐ ANTLR4's blow-up objection, priced on this knot: one clone per intermediate.
        assert_eq!(
            outcome.synthesized_clone_rules,
            vec!["prim_lr_seed_ct", "prim_lr_seed_cast_expr"]
        );

        // `prim := prim_lr_base ( prim_lr_suffix )*`
        assert_eq!(
            render_elements(std::slice::from_ref(
                grammar.get("prim").expect("prim survives")
            )),
            "prim_lr_base prim_lr_suffix*"
        );
        // `prim_lr_base := lit | <clone of cast_expr>` — the author's order, the cyclic alternative
        // replaced IN PLACE so its annotation index still lines up.
        assert_eq!(
            alternatives(&grammar, "prim_lr_base"),
            vec!["lit", "prim_lr_seed_cast_expr"]
        );
        // `prim_suffix := "'" "(" lit ")"` — P3's hand-written suffix, derived. It lives in a
        // per-route RULE so it can carry a `@profiles:` gate, with `prim_lr_suffix` the ordered
        // choice over those rules.
        assert_eq!(
            alternatives(&grammar, "prim_lr_suffix"),
            vec!["prim_lr_suffix_r0"]
        );
        assert_eq!(
            alternatives(&grammar, "prim_lr_suffix_r0"),
            vec!["\"'\" \"(\" lit \")\""]
        );
        // `cast_expr_seed := kw "'" "(" lit ")"` — P3 inlined `ct_seed := kw`; the engine keeps the
        // clone as its own rule because a real grammar reaches the intermediate from elsewhere too.
        assert_eq!(
            alternatives(&grammar, "prim_lr_seed_cast_expr"),
            vec!["prim_lr_seed_ct \"'\" \"(\" lit \")\""]
        );
        assert_eq!(alternatives(&grammar, "prim_lr_seed_ct"), vec!["kw"]);

        // ⛔ The ORIGINALS stay standing — P3 could drop `ct`/`cast_expr` only because its synthetic
        // reached them from nowhere else, and SystemVerilog reaches `casting_type` from `cast` too.
        assert!(grammar.contains_key("ct") && grammar.contains_key("cast_expr"));
        // Helpers and clones are ordered before the rule that names them.
        let index = |name: &str| order.iter().position(|rule| rule == name).expect(name);
        assert!(index("prim_lr_seed_ct") < index("prim"));
        assert!(index("prim_lr_base") < index("prim"));
        assert!(index("prim_lr_suffix") < index("prim"));
    }

    /// The load-bearing AST claim: the composed template is the route's hops nested outermost-first,
    /// with every non-`$1` positional remapped into the flattened suffix's capture space.
    #[test]
    fn the_composed_template_is_the_hand_derived_nesting() {
        let (mut grammar, mut order, mut annotations) = knot_a_annotated();
        eliminate_indirect_left_recursion(&mut grammar, &mut order, Some(&mut annotations));

        let chain = annotations
            .branch_return_annotations
            .get("prim")
            .and_then(|branches| branches.first().cloned().flatten())
            .and_then(|branch| branch.parsed_ast)
            .expect("the base rule carries the synthetic chain annotation");
        let UnifiedReturnAST::LrChainFold {
            initial,
            suffixes,
            specs,
        } = chain
        else {
            panic!("the base rule's annotation must be the chain fold, got {chain:?}");
        };
        assert_eq!(*initial, positional(1));
        assert_eq!(*suffixes, positional(2));
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].alt_index, 0);
        // `$1` (the running value) + the 4 flattened suffix elements.
        assert_eq!(specs[0].original_body_length, 5);

        // `{kind: "cast", body: {type: {kind: "prim", body: $1}, body: $4}}`
        //
        //  * `prim` alt#1's `{kind: "cast", body: $1}` is the OUTERMOST hop;
        //  * `cast_expr`'s `{type: $1, body: $4}` nests inside it, and its `$4` — the `lit` at
        //    flattened suffix position 3 — stays `$4` because the fold reads position `p` as
        //    `$(p + 1)`;
        //  * `ct` alt#1's `{kind: "prim", body: $1}` is innermost, and ITS `$1` is the running
        //    fold value, which is what makes the chain left-nested.
        let expected = object(vec![
            ("kind", literal("cast")),
            (
                "body",
                object(vec![
                    (
                        "type",
                        object(vec![("kind", literal("prim")), ("body", positional(1))]),
                    ),
                    ("body", positional(4)),
                ]),
            ),
        ]);
        assert_eq!(specs[0].annotation_template, expected);

        // And the per-route rule emits the `{alt_index, captures}` record the fold consumes, with
        // `prim_lr_suffix`'s own branch a `$1` passthrough over the bare reference.
        let route_record = annotations
            .branch_return_annotations
            .get("prim_lr_suffix_r0")
            .and_then(|branches| branches.first().cloned().flatten())
            .and_then(|branch| branch.parsed_ast)
            .expect("the route rule carries the chain-alt record");
        assert_eq!(
            route_record,
            object(vec![
                ("type", literal(lr_chain_fold::CHAIN_ALT_TYPE_MARKER)),
                ("alt_index", UnifiedReturnAST::NumberLiteral { value: 0.0 }),
                (
                    "captures",
                    UnifiedReturnAST::Array {
                        elements: (1..=4).map(positional).collect()
                    }
                ),
            ])
        );
        assert_eq!(
            annotations
                .branch_return_annotations
                .get("prim_lr_suffix")
                .and_then(|branches| branches.first().cloned().flatten())
                .and_then(|branch| branch.parsed_ast),
            Some(positional(1))
        );
    }

    /// ⛔ A hop with a residual and no declared AST is REFUSED, not approximated. `p1_knot_a_defect`
    /// is exactly this grammar and this is the refusal it earns.
    #[test]
    fn an_unannotated_hop_with_a_residual_is_refused_not_guessed() {
        let (mut grammar, mut order, _) = knot_a_annotated();
        let mut annotations = Annotations::default();
        let outcome =
            eliminate_indirect_left_recursion(&mut grammar, &mut order, Some(&mut annotations));

        assert_eq!(outcome.eliminated_base_rules, Vec::<String>::new());
        let refusal = outcome
            .refusals
            .iter()
            .find(|refusal| refusal.base_rule == "prim")
            .expect("the refusal is DATA, reported to the caller");
        assert!(
            refusal.reason.contains("cast_expr") && refusal.reason.contains("no return annotation"),
            "the refusal must name the hop and the cause: {}",
            refusal.reason
        );
        // Nothing was written: a refused plan leaves the grammar exactly as it was.
        assert!(!grammar.contains_key("prim_lr_base"));
        assert_eq!(
            alternatives(&grammar, "prim"),
            vec!["lit", "cast_expr"],
            "a refusal must not half-apply"
        );
    }

    /// ⭐⭐ The mutually-recursive SET, and the two things it decides.
    ///
    /// `prim_a` and `prim_b` are dialect twins reaching each other through a shared `prim` spine —
    /// SystemVerilog's `constant_primary_sv_2017` / `_sv_2023` under `constant_primary`, minimised.
    /// Eliminating at either twin leaves the other's arm live, because the path back re-enters the
    /// spine and so is not a SIMPLE route for the walk to shear. The pass must therefore
    ///
    ///  1. pick the DOMINATOR `prim`, even though every one of its alternatives is on the cycle
    ///     (`seeds=0` is not a disqualification once cyclic alternatives are CLONED, not dropped),
    ///     and
    ///  2. leave nothing left-recursive behind — which the trial-before-commit check proves rather
    ///     than assumes.
    #[test]
    fn a_mutually_recursive_set_is_eliminated_at_its_dominator() {
        let mut grammar = HashMap::new();
        grammar.insert("scratch".to_string(), rule("prim"));
        grammar.insert(
            "prim".to_string(),
            ASTNode::Or {
                alternatives: vec![rule("prim_a"), rule("prim_b")],
            },
        );
        for (twin, seed) in [("prim_a", "lit_a"), ("prim_b", "lit_b")] {
            grammar.insert(
                twin.to_string(),
                ASTNode::Or {
                    alternatives: vec![rule(seed), rule("cast_expr")],
                },
            );
        }
        grammar.insert(
            "cast_expr".to_string(),
            ASTNode::Sequence {
                elements: vec![rule("ct"), text("'"), text("("), rule("lit_a"), text(")")],
            },
        );
        grammar.insert("ct".to_string(), rule("prim"));
        grammar.insert("lit_a".to_string(), text("n"));
        grammar.insert("lit_b".to_string(), text("m"));
        let mut order: Vec<String> = [
            "scratch", "prim", "prim_a", "prim_b", "cast_expr", "ct", "lit_a", "lit_b",
        ]
        .iter()
        .map(|name| name.to_string())
        .collect();

        let mut annotations = Annotations::default();
        annotations.branch_return_annotations.insert(
            "cast_expr".to_string(),
            vec![annotation(object(vec![
                ("type", positional(1)),
                ("body", positional(4)),
            ]))],
        );

        let outcome =
            eliminate_indirect_left_recursion(&mut grammar, &mut order, Some(&mut annotations));

        assert_eq!(
            outcome.eliminated_base_rules,
            vec!["prim"],
            "the DOMINATOR absorbs the chain, never one of the twins"
        );
        // Both twins are inside the plan, so both arms are sheared.
        assert_eq!(
            alternatives(&grammar, "prim_lr_base"),
            vec!["prim_lr_seed_prim_a", "prim_lr_seed_prim_b"]
        );
        assert!(
            detect_left_recursion(&grammar, &order).is_empty(),
            "the rewritten grammar must carry no left recursion at all"
        );
    }

    /// ⛔⛔ **The profile gate must survive the rewrite, on BOTH the clone and the suffix route.**
    ///
    /// This is a defect slice 5 shipped and then measured on the emitted parser:
    /// `parse_constant_primary_sv_2017` carries
    /// `if !self.rule_profile_is_enabled(&["sv_2017", "verilog_2005"])` and its first clone carried
    /// nothing, so the sv_2017 primary became reachable under an `sv_2023` parse — an
    /// OVER-ACCEPTANCE introduced by a rewrite that looks purely structural.
    ///
    /// The suffix half is subtler and is the reason a route gets its own RULE: the two dialect
    /// routes iterate BYTE-IDENTICAL suffixes and differ only in the AST they declare, so an
    /// ungated ordered choice would stamp `kind: "a"` on a `b`-profile parse.
    #[test]
    fn a_profile_gate_survives_on_both_the_clone_and_the_suffix_route() {
        use crate::ast_pipeline::unified_semantic_ast::UnifiedSemanticAST;

        let profiles = |list: &str| {
            SemanticAnnotation::Named {
                name: "profiles".to_string(),
                ast: UnifiedSemanticAST::Raw {
                    content: format!("[{list}]"),
                },
            }
        };

        // `prim := prim_a | prim_b` with the twins gated to disjoint profiles — SystemVerilog's
        // `constant_primary := constant_primary_sv_2017 | constant_primary_sv_2023`, minimised.
        let build = |gated: bool| {
            let mut grammar = HashMap::new();
            grammar.insert("scratch".to_string(), rule("prim"));
            grammar.insert(
                "prim".to_string(),
                ASTNode::Or {
                    alternatives: vec![rule("prim_a"), rule("prim_b")],
                },
            );
            for (twin, seed) in [("prim_a", "lit_a"), ("prim_b", "lit_b")] {
                grammar.insert(
                    twin.to_string(),
                    ASTNode::Or {
                        alternatives: vec![rule(seed), rule("cast_expr")],
                    },
                );
            }
            grammar.insert(
                "cast_expr".to_string(),
                ASTNode::Sequence {
                    elements: vec![rule("ct"), text("'"), text("("), rule("lit_a"), text(")")],
                },
            );
            grammar.insert("ct".to_string(), rule("prim"));
            grammar.insert("lit_a".to_string(), text("n"));
            grammar.insert("lit_b".to_string(), text("m"));
            let order: Vec<String> = [
                "scratch", "prim", "prim_a", "prim_b", "cast_expr", "ct", "lit_a", "lit_b",
            ]
            .iter()
            .map(|name| name.to_string())
            .collect();

            let mut annotations = Annotations::default();
            if gated {
                annotations
                    .semantic_annotations
                    .insert("prim_a".to_string(), vec![profiles("\"a\"")]);
                annotations
                    .semantic_annotations
                    .insert("prim_b".to_string(), vec![profiles("\"b\"")]);
            }
            annotations.branch_return_annotations.insert(
                "prim".to_string(),
                vec![
                    annotation(object(vec![("kind", literal("a")), ("body", positional(1))])),
                    annotation(object(vec![("kind", literal("b")), ("body", positional(1))])),
                ],
            );
            annotations.branch_return_annotations.insert(
                "cast_expr".to_string(),
                vec![annotation(object(vec![
                    ("type", positional(1)),
                    ("body", positional(4)),
                ]))],
            );
            (grammar, order, annotations)
        };

        let (mut grammar, mut order, mut annotations) = build(true);
        let outcome =
            eliminate_indirect_left_recursion(&mut grammar, &mut order, Some(&mut annotations));
        assert_eq!(outcome.eliminated_base_rules, vec!["prim"]);

        let gate_of = |rule: &str| -> Option<String> {
            annotations
                .semantic_annotations
                .get(rule)
                .and_then(|entries| entries.first())
                .map(|entry| entry.ast().payload_text().to_string())
        };
        // The clones keep the twins' gates verbatim.
        assert_eq!(gate_of("prim_lr_seed_prim_a").as_deref(), Some("[\"a\"]"));
        assert_eq!(gate_of("prim_lr_seed_prim_b").as_deref(), Some("[\"b\"]"));
        // ⭐ And so do the two routes, whose suffixes are byte-identical.
        assert_eq!(gate_of("prim_lr_suffix_r0").as_deref(), Some("[\"a\"]"));
        assert_eq!(gate_of("prim_lr_suffix_r1").as_deref(), Some("[\"b\"]"));
        assert_eq!(
            alternatives(&grammar, "prim_lr_suffix_r0"),
            alternatives(&grammar, "prim_lr_suffix_r1"),
            "the two routes must be syntactically indistinguishable — that is what makes the gate \
             load-bearing rather than decorative"
        );

        // ⛔ Drop ONLY the gates and the SAME grammar becomes an ambiguity the pass must refuse
        // rather than resolve by alternative order — the two routes are then indistinguishable at
        // parse time and declare different ASTs.
        let (mut ungated_grammar, mut ungated_order, mut ungated_annotations) = build(false);
        let refused = eliminate_indirect_left_recursion(
            &mut ungated_grammar,
            &mut ungated_order,
            Some(&mut ungated_annotations),
        );
        assert_eq!(refused.eliminated_base_rules, Vec::<String>::new());
        assert!(
            refused
                .refusals
                .iter()
                .any(|refusal| refusal.reason.contains("identical suffix")),
            "an ungated ambiguity must be refused, not ordered: {:?}",
            refused.refusals
        );
    }

    /// A route the survey truncated or dropped knows about cycle edges it did not report; shearing
    /// only the reported ones would leave a live cycle inside the clone.
    #[test]
    fn a_degenerate_route_blocks_the_plan_rather_than_half_shearing_it() {
        // `a := "s" | b | c x`, `b := a`, `c := a` — the `b` route iterates an EMPTY suffix.
        let mut grammar = HashMap::new();
        grammar.insert(
            "a".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    text("s"),
                    rule("b"),
                    ASTNode::Sequence {
                        elements: vec![rule("c"), text("x")],
                    },
                ],
            },
        );
        grammar.insert("b".to_string(), rule("a"));
        grammar.insert("c".to_string(), rule("a"));
        let mut order: Vec<String> = ["a", "b", "c"].iter().map(|n| n.to_string()).collect();
        let mut annotations = Annotations::default();

        let outcome =
            eliminate_indirect_left_recursion(&mut grammar, &mut order, Some(&mut annotations));
        assert_eq!(outcome.eliminated_base_rules, Vec::<String>::new());
        assert!(
            outcome
                .refusals
                .iter()
                .any(|refusal| refusal.reason.contains("empty suffix")),
            "the degenerate route must be named as the reason: {:?}",
            outcome.refusals
        );
    }
}
