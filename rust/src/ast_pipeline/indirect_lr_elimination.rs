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
    ChainRoute, IndirectChainCandidate, survey_indirect_left_recursion,
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

/// What [`eliminate_indirect_left_recursion`] did, as opposed to a belief about it — the same
/// posture `GRAMMAR-WELLFORMED.A2.6` forced on the linter.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IndirectEliminationOutcome {
    /// Base rules rewritten to `X := X_lr_base ( X_lr_suffix )*`, in application order.
    pub eliminated_base_rules: Vec<String>,
    /// Clone rules synthesized, in creation order — the blow-up term, measured.
    pub synthesized_clone_rules: Vec<String>,
    /// Starvation-safe candidates that could NOT be planned, with the reason.
    pub refusals: Vec<PlanRefusal>,
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
    mut annotations: Option<&mut Annotations>,
) -> IndirectEliminationOutcome {
    let mut outcome = IndirectEliminationOutcome::default();
    // A refused candidate stays refused for the whole pass — otherwise every re-survey would
    // re-report it and the refusal list would grow with the loop counter rather than with the
    // number of distinct problems.
    let mut refused: BTreeSet<String> = BTreeSet::new();

    loop {
        let survey = survey_indirect_left_recursion(grammar_tree, rule_order);
        let mut ordered: Vec<&IndirectChainCandidate> = survey
            .safe_candidates()
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
                eprintln!(
                    "[indirect_lr_elimination] ✅ Absorbing indirect left-recursive chain at rule '{}' \
                     ({} route(s), {} clone(s) via helper '{}')",
                    plan.base_rule,
                    plan.suffix_branches.len(),
                    plan.clones.len(),
                    plan.helper_base_rule
                );
                outcome.eliminated_base_rules.push(plan.base_rule.clone());
                outcome
                    .synthesized_clone_rules
                    .extend(plan.clones.iter().map(|clone| clone.name.clone()));
                apply_plan(&plan, grammar_tree, rule_order, annotations.as_deref_mut());
            }
            Err(reason) => {
                eprintln!(
                    "[indirect_lr_elimination] ⏭️  Declining rule '{}': {reason}",
                    candidate.base_rule
                );
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
                    element: Box::new(suffix_element),
                    quantifier: "*".to_string(),
                },
            ],
        },
    );

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
            .insert(plan.helper_suffix_rule.clone(), base_profiles);
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
    annotations.branch_return_annotations.insert(
        plan.base_rule.clone(),
        vec![Some(BranchAnnotation {
            annotation_type: "_pgen_lr_chain_synthetic".to_string(),
            annotation_content: String::new(),
            parsed_ast: Some(UnifiedReturnAST::LrChainFold {
                initial: Box::new(UnifiedReturnAST::PositionalRef { index: 1 }),
                suffixes: Box::new(UnifiedReturnAST::PositionalRef { index: 2 }),
                specs,
            }),
        })],
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
    annotations.branch_return_annotations.insert(
        plan.helper_suffix_rule.clone(),
        plan.suffix_branches
            .iter()
            .map(|_| {
                Some(BranchAnnotation {
                    annotation_type: "_pgen_lr_chain_synthetic".to_string(),
                    annotation_content: String::new(),
                    parsed_ast: Some(UnifiedReturnAST::PositionalRef { index: 1 }),
                })
            })
            .collect(),
    );
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
