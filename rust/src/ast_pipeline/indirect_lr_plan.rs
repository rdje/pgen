//! `ENGINE-UNIVERSAL-SERVICES.13` slice 4 — the INDIRECT left-recursion SURVEY.
//!
//! PURE analysis over the post-elimination gen-AST: no I/O, no codegen, mutates nothing. It
//! answers, per surviving left-recursive cycle, the three questions acceptance (d)'s
//! transformation has to be designed against — **where** the chain can be absorbed, **what**
//! the absorbed suffix is, and **what it costs** in new rule names.
//!
//! ## Why this exists as its own pass
//!
//! [`super::RustASTPipeline::detect_left_recursive_chain_plan`] matches exactly one shape: a base
//! rule alternative that is a **bare reference to a wrapper rule whose own body begins with the
//! base rule** (`X := seed | W`, `W := X suffix`). `GRAMMAR-WELLFORMED.A2.5` taught the pass to
//! normalize the *inline direct* shape (`X := X op Y | seed`) into that same wrapper form. Neither
//! covers an **indirect** cycle — `X := … A …`, `A := … B …`, `B := … X …` — and slice 1 measured
//! what that costs on the shipped grammars: **7 distinct cycles in SystemVerilog, 3 in `ebnf`, and
//! 8 of those 10 reject text the standard licenses**
//! (`docs/tasks/artifacts/engine_universal_services/lr_cycle_adjudication/`).
//!
//! ## Why a SURVEY and not the rewrite
//!
//! Slice 3 measured the naive rewrite and it is a **regression**, not a fix
//! (`docs/tasks/artifacts/engine_universal_services/indirect_lr/`): eliminating at the rule the
//! lint names first (`casting_type`) turns the accepted `int'(3)` into a rejection, because PGEN's
//! `*` is greedy and does not backtrack its iteration count — the eliminated rule swallows the whole
//! chain and the *consumer* that needed the trailing residual can never match it. Eliminating at the
//! **consumer** rule (`constant_primary`) accepts every probe input on both oracles.
//!
//! That makes base-rule selection the load-bearing decision of the whole fix, and "pick the
//! consumer" is a description of two hand-written synthetics, not a rule an engine can apply. This
//! module turns it into a **mechanical, measured** criterion: for each candidate base rule it
//! reports every site that would be exposed to exactly the P2 starvation — a rule holding the
//! candidate at its left corner with a NON-EMPTY residual, which is precisely the shape
//! `cast_expr := ct "'" "(" lit ")"` has and `ct := kw | prim` has not.
//!
//! ## The transformation this survey prices (specified by P3, not implemented here)
//!
//! ```text
//! X          := X_lr_base ( X_lr_suffix )*
//! X_lr_base  := <every alternative of X that reaches no cycle>  |  <a CLONE of each that does>
//! X_lr_suffix:= <route residual, innermost step first>
//! ```
//!
//! The clone is the blow-up term: one new rule per intermediate on the cycle path, each a copy of
//! the intermediate with the cycle-closing edge sheared off. [`IndirectChainCandidate::clone_cost`]
//! counts them, so ANTLR4's "wholly unworkable in practice" objection is priced against this
//! repository's real grammars rather than against an arbitrary one.
//!
//! ## Soundness posture — DECLINE LOUDLY
//!
//! A route is only reported when every step's left corner is a **bare leading rule reference**. A
//! cycle that closes through a nullable prefix, a quantified left corner or a group is reported in
//! [`IndirectChainSurvey::declined`] with its reason, never silently dropped. The survey's own
//! coverage is therefore a measured number rather than an assumption — the same posture
//! `GRAMMAR-WELLFORMED.A2.6` forced on the lint, which used to assert that cycles it had declined
//! were "handled".

use super::grammar_wellformedness::{WellformednessIssue, detect_left_recursion};
use super::{ASTNode, ASTValue, TokenValue};
use std::collections::{BTreeMap, BTreeSet, HashMap};

/// Upper bound on simple left-corner routes enumerated per candidate base rule.
///
/// ⛔ A bound that is silently hit is a lie about coverage, so hitting it sets
/// [`IndirectChainCandidate::routes_truncated`] and the report prints it. The value is deliberately
/// far above what the shipped grammars need (SystemVerilog's widest knot is single digits) — it is
/// a runaway guard for a pathological grammar, not a sampling policy.
pub const MAX_ROUTES_PER_BASE: usize = 128;

/// One step on a left-corner route: "inside `rule`, alternative `alternative_index` begins with a
/// reference to `next_rule`, and everything after that reference is `residual`".
#[derive(Debug, Clone)]
pub struct ChainStep {
    /// The rule this step is inside.
    pub rule: String,
    /// Index of the alternative taken within `rule`'s `Or` (`0` when the body is not an `Or`).
    pub alternative_index: usize,
    /// The rule reference sitting at that alternative's left corner — the edge this step follows.
    pub next_rule: String,
    /// The alternative's elements AFTER the leading rule reference. This is the step's contribution
    /// to the chain suffix, and an empty residual is the common case (a bare rule reference).
    pub residual: Vec<ASTNode>,
}

/// One simple left-corner route from a base rule back to itself.
///
/// `steps[0].rule` is the base rule and the final step's `next_rule` is the base rule again, so a
/// route of length `n` visits `n` distinct rules.
#[derive(Debug, Clone)]
pub struct ChainRoute {
    /// Which alternative of the base rule this route leaves through. The rewrite replaces exactly
    /// this alternative in `X_lr_base` with a sheared clone.
    pub base_alternative_index: usize,
    /// Steps, outermost first.
    pub steps: Vec<ChainStep>,
}

impl ChainRoute {
    /// The suffix the eliminated base rule iterates: the steps' residuals concatenated
    /// **innermost first**.
    ///
    /// ⭐ The order is the load-bearing detail and it is the reverse of the route. A derivation
    /// descends the route to the cycle-closing reference and then *returns* through the steps,
    /// consuming each one's residual on the way out, so the deepest step's residual is the text
    /// that follows the recursive occurrence most closely. On SystemVerilog knot A the steps are
    /// `constant_primary` (residual empty) → `constant_primary_sv_2017` (empty) → `constant_cast`
    /// (`tick lparen constant_expression rparen`) → `casting_type` (empty), and the suffix is
    /// `tick lparen constant_expression rparen` — which is exactly the text `'(3)` that
    /// `int'(2)'(3)` needs and today's parser cannot reach.
    pub fn suffix_elements(&self) -> Vec<ASTNode> {
        self.steps
            .iter()
            .rev()
            .flat_map(|step| step.residual.iter().cloned())
            .collect()
    }

    /// The rules on the route, outermost first, starting with the base rule.
    pub fn path(&self) -> Vec<String> {
        self.steps.iter().map(|step| step.rule.clone()).collect()
    }

    /// The rules between the base rule and the cycle-closing rule, inclusive of the closer and
    /// exclusive of the base — the rules a rewrite must CLONE with the cycle edge sheared off.
    pub fn intermediate_rules(&self) -> Vec<String> {
        self.steps
            .iter()
            .skip(1)
            .map(|step| step.rule.clone())
            .collect()
    }

    /// A route whose suffix is empty derives nothing new per iteration: `X := X_base ( )*` would
    /// loop without consuming. Such a route must never be turned into a `*` — it is a
    /// **non-terminating** cycle and belongs to the linter's `NonTerminating` error.
    pub fn is_degenerate(&self) -> bool {
        self.suffix_elements().is_empty()
    }
}

/// Why a surviving cycle produced no route — the survey's own coverage gap, itemised.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeclineReason {
    /// No alternative of some rule on the cycle exposes a **bare leading rule reference** toward the
    /// cycle: the left corner is reached through a nullable prefix, a quantifier, a group or a
    /// lookahead. The residual is then not `elements[1..]` and this survey will not guess one.
    NoBareLeftCornerRoute,
    /// A route exists but every one of them iterates an empty suffix, so the rewrite would emit a
    /// non-consuming `*`. Reported rather than emitted.
    OnlyDegenerateRoutes,
}

impl DeclineReason {
    /// A short, stable token for the report and for tests.
    pub fn token(&self) -> &'static str {
        match self {
            DeclineReason::NoBareLeftCornerRoute => "no_bare_left_corner_route",
            DeclineReason::OnlyDegenerateRoutes => "only_degenerate_routes",
        }
    }
}

/// A surviving cycle the survey could not turn into a plan, with the reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclinedCycle {
    /// The rule the lint reported the cycle from.
    pub rule: String,
    /// The cycle path the lint printed, for cross-referencing the `--lint-grammar` output.
    pub cycle: Vec<String>,
    pub reason: DeclineReason,
}

/// A site that would be exposed to the P2 starvation if `base_rule` absorbed the chain.
///
/// ⛔ This is the measurement that decides base-rule selection. After `X := X_base ( suffix )*`,
/// any rule holding `X` — **or any rule TRANSPARENT to `X`** ([`rules_transparent_to`]) — at its
/// left corner **followed by more elements** is at risk: PGEN's `*` is greedy and never retries at
/// a lower iteration count, so if the greedy `X` consumes text the holder's own residual needed,
/// the holder can no longer match. Slice 3 measured this directly (`cast_expr := ct "'" "(" lit ")"`
/// went accept → reject when `ct` was the base); slice 5 measured the TRANSITIVE form, where the
/// holder never names `X` at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StarvationSite {
    /// The rule holding the candidate — or a rule transparent to it — at a left corner with a
    /// non-empty residual.
    pub rule: String,
    /// Which alternative of that rule.
    pub alternative_index: usize,
    /// The residual that a greedy suffix could steal, rendered for the report.
    pub residual: String,
    /// Is this holder itself an intermediate on one of the candidate's own routes?
    ///
    /// ⛔ **CONTEXT, NEVER AN EXEMPTION — and the first draft of this survey made it one, which the
    /// measured ground truth immediately refuted.** The reasoning that failed: "an on-route holder
    /// is fine, the rewrite shears the cycle edge out of the clone it makes of this rule". It does
    /// — but it shears the CLONE and leaves the ORIGINAL holder standing, still pointing at a base
    /// rule that is now greedy. On the P1 synthetic that exemption made `ct` report `MAY-ABSORB`,
    /// and `ct` is exactly the rule slice 3 measured turning `t'(n)` from accept into reject
    /// (`p2_eliminated_at_inner_rule.ebnf`). What decides the hazard is [`Self::survives_rewrite`],
    /// not this flag.
    pub on_route: bool,
    /// Is this holder still REACHABLE once the rewrite has run — i.e. can it still be entered and
    /// therefore still be starved?
    ///
    /// ⭐ This is the distinction the raw site count cannot make, and `grammars/ebnf.ebnf` is the
    /// case that forced it. `arithmetic_return := return_expression arithmetic_operator
    /// return_expression` holds the candidate `return_expression` with a residual, so it reads as a
    /// hazard — but it is referenced from nowhere except `expression_return`, which is referenced
    /// from nowhere except the very alternative of `return_expression` the rewrite replaces with a
    /// clone. The whole holder chain goes dead with the rewrite and can starve nothing. The P1
    /// counter-case is exactly as sharp: with `ct` as the candidate, the holder `cast_expr` stays
    /// live because `prim` still names it and `scratch := prim` still names `prim` — reachable from
    /// outside the plan, and measurably starved.
    pub survives_rewrite: bool,
}

/// Everything the survey knows about one candidate base rule.
#[derive(Debug, Clone)]
pub struct IndirectChainCandidate {
    pub base_rule: String,
    /// Simple left-corner routes from `base_rule` back to itself, in deterministic order.
    pub routes: Vec<ChainRoute>,
    /// True when route enumeration hit [`MAX_ROUTES_PER_BASE`]; the candidate's numbers are then
    /// lower bounds and must not be reported as totals.
    pub routes_truncated: bool,
    /// Alternatives of the base rule that reach no cycle — the verbatim seeds of `X_lr_base`.
    ///
    /// ⛔ **An empty vector is NOT a disqualification, and treating it as one is what hid
    /// SystemVerilog's real base rule for two slices.** The direct/wrapper elimination DROPS a
    /// left-recursive alternative, so a rule with no acyclic alternative has nothing left to seed
    /// from; the indirect transformation CLONES it with the cycle edge sheared, so a cyclic
    /// alternative still contributes every non-cyclic derivation underneath it. `constant_primary`
    /// has 0 acyclic alternatives and its two clones carry 13 and 14 seeds between them.
    pub acyclic_alternative_indices: Vec<usize>,
    /// Sites exposed to the greedy-`*` starvation, in deterministic order.
    pub starvation_sites: Vec<StarvationSite>,
    /// How many routes were found and then DROPPED for iterating an empty suffix.
    ///
    /// ⛔ Reported rather than silently filtered. A degenerate route would become a non-consuming
    /// `( )*` branch — an infinite loop, not a chain — so it can never enter the plan; but a
    /// candidate that quietly lost routes is a candidate whose numbers mean something different
    /// from what they say.
    pub degenerate_routes_dropped: usize,
}

impl IndirectChainCandidate {
    /// Distinct intermediates across all routes — one clone each, and the whole of ANTLR4's
    /// blow-up objection as it actually applies here.
    pub fn clone_cost(&self) -> BTreeSet<String> {
        self.routes
            .iter()
            .flat_map(|route| route.intermediate_rules())
            .collect()
    }

    /// Every rule this candidate's routes pass through, itself included — the knot it would close.
    ///
    /// ⭐ Used to prefer the rule that DOMINATES a mutually-recursive set over one of its members.
    /// SystemVerilog's cast/call knot holds two dialect twins (`constant_primary_sv_2017` and
    /// `_sv_2023`) that reach each other through a shared `constant_primary` spine: eliminating at
    /// either twin leaves the other's arm live, because the path back is not a SIMPLE route and so
    /// no route shears it. Eliminating at `constant_primary` — whose coverage is a strict superset
    /// — puts both twins inside one plan, and its routes are simple.
    pub fn covered_rules(&self) -> BTreeSet<String> {
        let mut covered: BTreeSet<String> = BTreeSet::new();
        covered.insert(self.base_rule.clone());
        for route in &self.routes {
            for step in &route.steps {
                covered.insert(step.rule.clone());
                covered.insert(step.next_rule.clone());
            }
        }
        covered
    }

    /// Starvation sites that would still be REACHABLE after the rewrite — the real hazard.
    pub fn surviving_starvation_sites(&self) -> Vec<&StarvationSite> {
        self.starvation_sites
            .iter()
            .filter(|site| site.survives_rewrite)
            .collect()
    }

    /// The mechanical verdict slice 3 asked for: may this rule absorb the chain?
    ///
    /// Safe iff no rule that OUTLIVES the rewrite holds this candidate at a left corner with a
    /// non-empty residual. Verified against both hand-written outcomes: `prim` (P3, accepts all
    /// five probe inputs on both oracles) is safe; `ct` (P2, turns `t'(n)` from accept into
    /// reject) is not.
    pub fn is_starvation_safe(&self) -> bool {
        self.surviving_starvation_sites().is_empty()
    }
}

/// The survey's whole result.
#[derive(Debug, Clone, Default)]
pub struct IndirectChainSurvey {
    /// One entry per rule that has at least one non-degenerate left-corner route back to itself,
    /// in `rule_order`.
    pub candidates: Vec<IndirectChainCandidate>,
    /// Surviving cycles the survey declined, with reasons.
    pub declined: Vec<DeclinedCycle>,
    /// Rules the lint reported as left-recursive, in `rule_order` — the survey's denominator.
    pub surviving_cycle_rules: Vec<String>,
}

impl IndirectChainSurvey {
    /// Candidates that are starvation-safe, i.e. the rules a rewrite may legally target.
    pub fn safe_candidates(&self) -> Vec<&IndirectChainCandidate> {
        self.candidates
            .iter()
            .filter(|candidate| candidate.is_starvation_safe())
            .collect()
    }

    /// Cycle rules covered by at least one candidate — the survey's coverage numerator.
    pub fn covered_cycle_rules(&self) -> BTreeSet<String> {
        let mut covered: BTreeSet<String> = BTreeSet::new();
        for candidate in &self.candidates {
            covered.insert(candidate.base_rule.clone());
            for route in &candidate.routes {
                for step in &route.steps {
                    covered.insert(step.rule.clone());
                }
            }
        }
        covered.retain(|rule| self.surviving_cycle_rules.contains(rule));
        covered
    }
}

/// Survey the grammar's surviving left recursion.
///
/// `grammar` / `rule_order` are the POST-elimination gen-AST, the same view `--lint-grammar` reads,
/// so a cycle reaching this function is by construction one the existing pass declined.
pub fn survey_indirect_left_recursion(
    grammar: &HashMap<String, ASTNode>,
    rule_order: &[String],
) -> IndirectChainSurvey {
    let mut surviving_cycle_rules: Vec<String> = Vec::new();
    let mut cycles_by_rule: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for issue in detect_left_recursion(grammar, rule_order) {
        if let WellformednessIssue::LeftRecursive { rule, cycle } = issue {
            surviving_cycle_rules.push(rule.clone());
            cycles_by_rule.insert(rule, cycle);
        }
    }

    // ⭐ Build the bare-left-corner step table ONCE. Recomputing it inside the walk would clone
    // every alternative's AST at every visit — on a 1 481-rule grammar that is the difference
    // between a report and a hang.
    let steps_by_rule: HashMap<String, Vec<ChainStep>> = rule_order
        .iter()
        .filter_map(|rule_name| {
            let body = grammar.get(rule_name)?;
            let steps: Vec<ChainStep> = alternatives_of(body)
                .iter()
                .enumerate()
                .filter_map(|(index, alternative)| left_corner_step(rule_name, index, alternative))
                .collect();
            Some((rule_name.clone(), steps))
        })
        .collect();
    let alternative_counts: HashMap<&String, usize> = rule_order
        .iter()
        .filter_map(|rule_name| Some((rule_name, alternatives_of(grammar.get(rule_name)?).len())))
        .collect();

    let mut candidates: Vec<IndirectChainCandidate> = Vec::new();
    let mut declined: Vec<DeclinedCycle> = Vec::new();

    // Iterate `rule_order`, never the HashMap — every report this repository emits has to be
    // byte-deterministic for the same reason codegen does.
    for rule_name in rule_order {
        let Some(cycle) = cycles_by_rule.get(rule_name) else {
            continue;
        };
        let Some(own_steps) = steps_by_rule.get(rule_name) else {
            continue;
        };
        // ⛔ Prune to the rules that can actually close back on this base. Without it the walk is
        // a blind DFS over the whole left-corner graph and explores overwhelmingly many paths that
        // never reach the base at all; the route BUDGET does not bound that, because it only counts
        // routes FOUND.
        let closers = rules_reaching(rule_name, &steps_by_rule);

        let mut routes: Vec<ChainRoute> = Vec::new();
        let mut routes_truncated = false;
        for step in own_steps {
            if &step.next_rule != rule_name && !closers.contains(&step.next_rule) {
                continue;
            }
            let mut visited: Vec<String> = vec![rule_name.to_string()];
            let mut found: Vec<Vec<ChainStep>> = Vec::new();
            let budget = MAX_ROUTES_PER_BASE.saturating_sub(routes.len());
            let complete = collect_routes(
                rule_name,
                step,
                &steps_by_rule,
                &closers,
                &mut visited,
                &mut found,
                budget,
            );
            routes_truncated |= !complete;
            for steps in found {
                routes.push(ChainRoute {
                    base_alternative_index: step.alternative_index,
                    steps,
                });
            }
            if routes.len() >= MAX_ROUTES_PER_BASE {
                routes_truncated = true;
                break;
            }
        }

        if routes.is_empty() {
            declined.push(DeclinedCycle {
                rule: rule_name.clone(),
                cycle: cycle.clone(),
                reason: DeclineReason::NoBareLeftCornerRoute,
            });
            continue;
        }
        // ⛔ A route iterating an empty suffix cannot enter a plan — `X := X_base ( )*` never
        // consumes. Drop it, and COUNT the drop so the candidate's route total stays honest.
        let degenerate_routes_dropped = routes.iter().filter(|r| r.is_degenerate()).count();
        routes.retain(|route| !route.is_degenerate());
        if routes.is_empty() {
            declined.push(DeclinedCycle {
                rule: rule_name.clone(),
                cycle: cycle.clone(),
                reason: DeclineReason::OnlyDegenerateRoutes,
            });
            continue;
        }
        // Re-derive from the SURVIVING routes: an alternative reached only by a dropped route is
        // still an acyclic seed for the plan that is actually built.
        let cyclic_alternatives: BTreeSet<usize> = routes
            .iter()
            .map(|route| route.base_alternative_index)
            .collect();
        let alternative_count = alternative_counts.get(rule_name).copied().unwrap_or(0);
        let acyclic_alternative_indices: Vec<usize> = (0..alternative_count)
            .filter(|index| !cyclic_alternatives.contains(index))
            .collect();

        let on_route_rules: BTreeSet<String> = routes
            .iter()
            .flat_map(|route| route.intermediate_rules())
            .collect();
        let survivors = rules_surviving_rewrite(
            rule_name,
            &routes,
            &on_route_rules,
            &acyclic_alternative_indices,
            grammar,
            rule_order,
        );
        let starvation_sites = collect_starvation_sites(
            rule_name,
            &steps_by_rule,
            rule_order,
            &on_route_rules,
            &survivors,
        );

        candidates.push(IndirectChainCandidate {
            base_rule: rule_name.clone(),
            routes,
            routes_truncated,
            acyclic_alternative_indices,
            starvation_sites,
            degenerate_routes_dropped,
        });
    }

    IndirectChainSurvey {
        candidates,
        declined,
        surviving_cycle_rules,
    }
}

/// Every rule from which `target` is reachable through bare left-corner edges — the only rules a
/// route out of `target` can usefully pass through.
///
/// Computed by BFS over the REVERSED edge relation, so it costs one pass over the step table per
/// candidate rather than one per DFS branch.
fn rules_reaching(
    target: &str,
    steps_by_rule: &HashMap<String, Vec<ChainStep>>,
) -> BTreeSet<String> {
    let mut predecessors: HashMap<&str, Vec<&str>> = HashMap::new();
    for (rule, steps) in steps_by_rule {
        for step in steps {
            predecessors
                .entry(step.next_rule.as_str())
                .or_default()
                .push(rule.as_str());
        }
    }
    let mut reaching: BTreeSet<String> = BTreeSet::new();
    let mut frontier: Vec<&str> = vec![target];
    while let Some(rule) = frontier.pop() {
        for predecessor in predecessors.get(rule).into_iter().flatten() {
            if reaching.insert((*predecessor).to_string()) {
                frontier.push(predecessor);
            }
        }
    }
    reaching
}

/// Depth-first enumeration of simple left-corner routes from `head.next_rule` back to `base_rule`.
///
/// `head` is the step already taken out of the current rule; `closers` prunes the search to rules
/// from which `base_rule` is reachable at all. Returns `false` when the budget was exhausted before
/// enumeration finished, so the caller can mark its numbers as lower bounds.
fn collect_routes(
    base_rule: &str,
    head: &ChainStep,
    steps_by_rule: &HashMap<String, Vec<ChainStep>>,
    closers: &BTreeSet<String>,
    visited: &mut Vec<String>,
    out: &mut Vec<Vec<ChainStep>>,
    budget: usize,
) -> bool {
    if out.len() >= budget {
        return false;
    }
    if head.next_rule == base_rule {
        out.push(vec![head.clone()]);
        return true;
    }
    if visited.iter().any(|rule| rule == &head.next_rule) {
        // A simple route only: re-entering a rule already on the path is a DIFFERENT cycle, and
        // folding two cycles into one route would produce a suffix neither of them has.
        return true;
    }
    let Some(steps) = steps_by_rule.get(&head.next_rule) else {
        return true;
    };

    visited.push(head.next_rule.clone());
    let mut complete = true;
    for next in steps {
        if next.next_rule != base_rule && !closers.contains(&next.next_rule) {
            continue;
        }
        let mut tails: Vec<Vec<ChainStep>> = Vec::new();
        let remaining = budget.saturating_sub(out.len());
        if remaining == 0 {
            complete = false;
            break;
        }
        complete &= collect_routes(
            base_rule,
            next,
            steps_by_rule,
            closers,
            visited,
            &mut tails,
            remaining,
        );
        for tail in tails {
            let mut route = vec![head.clone()];
            route.extend(tail);
            out.push(route);
        }
    }
    visited.pop();
    complete
}

/// The left-corner step an alternative exposes, if it exposes a **bare leading rule reference**.
///
/// ⛔ Deliberately narrower than the lint's `leftmost_refs`, which follows through nullable
/// prefixes to answer "is there a cycle at all". A rewrite needs the residual, and past a nullable
/// prefix the residual is not `elements[1..]` — the prefix may or may not have consumed. Declining
/// that shape here and counting it in [`IndirectChainSurvey::declined`] keeps the survey's coverage
/// a measured number instead of a silent approximation.
fn left_corner_step(rule: &str, alternative_index: usize, alternative: &ASTNode) -> Option<ChainStep> {
    let elements: Vec<ASTNode> = match alternative {
        ASTNode::Sequence { elements } => elements.clone(),
        other => vec![other.clone()],
    };
    let first = elements.first()?;
    let next_rule = rule_reference_name(first)?;
    Some(ChainStep {
        rule: rule.to_string(),
        alternative_index,
        next_rule,
        residual: elements[1..].to_vec(),
    })
}

/// Which of the plan's own rules are still REACHABLE after the base rule absorbs the chain.
///
/// The rewrite replaces each cyclic alternative of `base_rule` with a CLONE, and a clone references
/// other clones, never the originals. An on-route intermediate therefore survives only if something
/// outside the plan still names it — directly, or through another surviving intermediate. That is a
/// least-fixed-point, computed here over the ORIGINAL bodies because a rule that survives keeps its
/// body unchanged.
///
/// ⭐ Deliberately NOT entry-reachability. Whether the grammar's entry can reach a rule is a
/// different question (the lint's `unreachable_rules` owns it); what matters for starvation is
/// whether the REWRITE severs the rule's last incoming edge.
fn rules_surviving_rewrite(
    base_rule: &str,
    routes: &[ChainRoute],
    on_route_rules: &BTreeSet<String>,
    acyclic_alternative_indices: &[usize],
    grammar: &HashMap<String, ASTNode>,
    rule_order: &[String],
) -> BTreeSet<String> {
    use super::grammar_wellformedness::collect_node_rule_refs;

    let mut plan_set: BTreeSet<String> = on_route_rules.clone();
    plan_set.insert(base_rule.to_string());

    // Seed 1 — anything named from OUTSIDE the plan keeps its target alive.
    let mut live: BTreeSet<String> = BTreeSet::new();
    let mut seed_from = |node: &ASTNode, live: &mut BTreeSet<String>| {
        let mut refs = std::collections::HashSet::new();
        collect_node_rule_refs(node, &mut refs);
        for name in refs {
            if plan_set.contains(&name) {
                live.insert(name);
            }
        }
    };
    for rule_name in rule_order {
        if plan_set.contains(rule_name) {
            continue;
        }
        if let Some(body) = grammar.get(rule_name) {
            seed_from(body, &mut live);
        }
    }
    // Seed 2 — the base rule's surviving parts: its acyclic alternatives (which move into
    // `X_lr_base` verbatim) and the suffix each route contributes (which moves into `X_lr_suffix`).
    if let Some(body) = grammar.get(base_rule) {
        let alternatives = alternatives_of(body);
        for index in acyclic_alternative_indices {
            if let Some(alternative) = alternatives.get(*index) {
                seed_from(alternative, &mut live);
            }
        }
    }
    for route in routes {
        for element in route.suffix_elements() {
            seed_from(&element, &mut live);
        }
    }

    // Propagate — a surviving rule keeps everything its unchanged body names.
    loop {
        let mut grown = false;
        for rule_name in live.clone() {
            if rule_name == base_rule {
                continue;
            }
            if let Some(body) = grammar.get(&rule_name) {
                let before = live.len();
                seed_from(body, &mut live);
                grown |= live.len() != before;
            }
        }
        if !grown {
            break;
        }
    }

    // The base rule itself is rewritten, never removed.
    live.insert(base_rule.to_string());
    live
}

/// Rules that are **transparent** to `base_rule`: the base itself, plus every rule with an
/// alternative that is a BARE reference (empty residual) to another transparent rule.
///
/// ⛔⛔ **WHY TRANSPARENCY, AND NOT JUST DIRECT HOLDERS — this is a REGRESSION the first version of
/// the transformation actually shipped, and the two-sided reproducer ratchet caught it.**
/// A transparent rule adds nothing between its caller and the base, so once the base becomes
/// `X_base ( X_suffix )*` the transparent rule is **just as greedy as the base**. Measured on
/// SystemVerilog:
///
/// ```text
/// casting_type := … | constant_primary          ← bare reference: TRANSPARENT to constant_primary
/// cast         := casting_type tick lparen expression rparen   ← holds it WITH a residual
/// ```
///
/// With `constant_primary` as the base, `casting_type` inherits the greed, `constant_primary`
/// swallows the whole `8'(1)` as a cast of its own, and `cast` can never match its trailing
/// `tick lparen expression rparen`. `initial k = 8'(1);` went **ACCEPT → REJECT** — the same P2
/// starvation slice 3 measured, one hop further out, where a direct-holder scan cannot see it.
///
/// ⭐ The synthetic could not have found this and its own README said so: P3 deleted `ct` and
/// `cast_expr` because nothing else reached them, while *"SystemVerilog reaches `casting_type` from
/// `cast` too, so a real transformation must ADD the clone and KEEP the originals"*. Keeping the
/// original is exactly what leaves the starved holder standing.
fn rules_transparent_to(
    base_rule: &str,
    steps_by_rule: &HashMap<String, Vec<ChainStep>>,
) -> BTreeSet<String> {
    let mut transparent: BTreeSet<String> = BTreeSet::new();
    transparent.insert(base_rule.to_string());
    loop {
        let mut grown = false;
        for (rule, steps) in steps_by_rule {
            if transparent.contains(rule) {
                continue;
            }
            if steps
                .iter()
                .any(|step| step.residual.is_empty() && transparent.contains(&step.next_rule))
            {
                transparent.insert(rule.clone());
                grown = true;
            }
        }
        if !grown {
            break;
        }
    }
    transparent
}

/// Every site holding a rule TRANSPARENT to `base_rule` at a left corner with a non-empty residual.
fn collect_starvation_sites(
    base_rule: &str,
    steps_by_rule: &HashMap<String, Vec<ChainStep>>,
    rule_order: &[String],
    on_route_rules: &BTreeSet<String>,
    survivors: &BTreeSet<String>,
) -> Vec<StarvationSite> {
    let transparent = rules_transparent_to(base_rule, steps_by_rule);
    let mut sites: Vec<StarvationSite> = Vec::new();
    for rule_name in rule_order {
        if rule_name == base_rule {
            continue;
        }
        let Some(steps) = steps_by_rule.get(rule_name) else {
            continue;
        };
        for step in steps {
            if !transparent.contains(&step.next_rule) || step.residual.is_empty() {
                continue;
            }
            sites.push(StarvationSite {
                rule: rule_name.clone(),
                alternative_index: step.alternative_index,
                residual: render_elements(&step.residual),
                on_route: on_route_rules.contains(rule_name),
                // A holder outside the plan is untouched by the rewrite and therefore always
                // survives; an on-route holder survives only if the fixpoint says so.
                survives_rewrite: !on_route_rules.contains(rule_name)
                    || survivors.contains(rule_name),
            });
        }
    }
    sites
}

fn alternatives_of(node: &ASTNode) -> Vec<ASTNode> {
    match node {
        ASTNode::Or { alternatives } => alternatives.clone(),
        other => vec![other.clone()],
    }
}

/// The rule name a node references, when the node IS a bare rule reference.
fn rule_reference_name(node: &ASTNode) -> Option<String> {
    match node {
        ASTNode::Atom {
            value: ASTValue::Token(parts),
        } => {
            if parts.len() < 2 {
                return None;
            }
            let TokenValue::String(token_type) = &parts[0];
            let TokenValue::String(token_value) = &parts[1];
            (token_type == "rule_reference").then(|| token_value.clone())
        }
        ASTNode::Sequence { elements } if elements.len() == 1 => rule_reference_name(&elements[0]),
        _ => None,
    }
}

/// Render nodes as compact EBNF-ish text. Report-only — nothing parses this back.
pub fn render_elements(elements: &[ASTNode]) -> String {
    elements
        .iter()
        .map(render_node)
        .collect::<Vec<_>>()
        .join(" ")
}

fn render_node(node: &ASTNode) -> String {
    match node {
        ASTNode::Or { alternatives } => format!(
            "( {} )",
            alternatives
                .iter()
                .map(render_node)
                .collect::<Vec<_>>()
                .join(" | ")
        ),
        ASTNode::Sequence { elements } => render_elements(elements),
        ASTNode::Quantified { element, quantifier } => {
            format!("{}{}", render_node(element), quantifier)
        }
        ASTNode::Lookahead { element, positive } => {
            format!("{}{}", if *positive { "&" } else { "!" }, render_node(element))
        }
        ASTNode::Atom { value } => match value {
            ASTValue::Node(inner) => render_node(inner),
            ASTValue::Token(parts) if parts.len() >= 2 => {
                let TokenValue::String(token_type) = &parts[0];
                let TokenValue::String(token_value) = &parts[1];
                match token_type.as_str() {
                    "rule_reference" => token_value.clone(),
                    "quoted_string" => format!("\"{}\"", token_value),
                    _ => token_value.clone(),
                }
            }
            ASTValue::Token(_) => "<atom>".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build the P1 synthetic — SystemVerilog knot A, edge for edge
    /// (`docs/tasks/artifacts/engine_universal_services/indirect_lr/p1_knot_a_defect.ebnf`):
    ///
    /// ```text
    /// scratch   := prim
    /// prim      := lit | cast_expr
    /// cast_expr := ct "'" "(" lit ")"
    /// ct        := kw | prim
    /// ```
    fn knot_a() -> (HashMap<String, ASTNode>, Vec<String>) {
        let rule = |name: &str| ASTNode::Atom {
            value: ASTValue::Token(vec![
                TokenValue::String("rule_reference".to_string()),
                TokenValue::String(name.to_string()),
            ]),
        };
        let text = |literal: &str| ASTNode::Atom {
            value: ASTValue::Token(vec![
                TokenValue::String("quoted_string".to_string()),
                TokenValue::String(literal.to_string()),
            ]),
        };
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
        let order = ["scratch", "prim", "cast_expr", "ct", "lit", "kw"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        (grammar, order)
    }

    #[test]
    fn knot_a_route_reproduces_the_p3_suffix() {
        let (grammar, order) = knot_a();
        let survey = survey_indirect_left_recursion(&grammar, &order);

        let prim = survey
            .candidates
            .iter()
            .find(|c| c.base_rule == "prim")
            .expect("prim is a candidate base rule");
        assert_eq!(prim.routes.len(), 1);
        let route = &prim.routes[0];
        assert_eq!(route.base_alternative_index, 1, "the cyclic alternative is cast_expr");
        assert_eq!(route.path(), vec!["prim", "cast_expr", "ct"]);
        assert_eq!(route.intermediate_rules(), vec!["cast_expr", "ct"]);
        // The whole point of the survey: this string is `p3_eliminated_at_consumer_rule.ebnf`'s
        // hand-written `prim_suffix := "'" "(" lit ")"`, derived instead of guessed.
        assert_eq!(render_elements(&route.suffix_elements()), "\"'\" \"(\" lit \")\"");
        assert_eq!(prim.acyclic_alternative_indices, vec![0]);
    }

    #[test]
    fn the_consumer_rule_is_starvation_safe_and_the_lint_named_rule_is_not() {
        let (grammar, order) = knot_a();
        let survey = survey_indirect_left_recursion(&grammar, &order);

        let prim = survey
            .candidates
            .iter()
            .find(|c| c.base_rule == "prim")
            .expect("prim is a candidate");
        let ct = survey
            .candidates
            .iter()
            .find(|c| c.base_rule == "ct")
            .expect("ct is a candidate too — every rule on a cycle is");

        // P3 accepted all five probe inputs on both oracles; P2 turned `t'(n)` accept → reject.
        // The survey has to separate them WITHOUT being told which is which.
        assert!(prim.is_starvation_safe(), "prim holds no external starvation site");
        assert!(
            !ct.is_starvation_safe(),
            "ct is starved by cast_expr := ct \"'\" \"(\" lit \")\" — the measured P2 regression"
        );
        let site = &ct.starvation_sites[0];
        assert_eq!(site.rule, "cast_expr");
        assert_eq!(site.residual, "\"'\" \"(\" lit \")\"");
        // ⛔ And the site is ON `ct`'s own route, which is exactly why an on-route exemption cannot
        // exist: the transformation shears the clone, never this original holder — and this holder
        // stays reachable, because `prim` still names it and `scratch := prim` still names `prim`.
        assert!(site.on_route);
        assert!(site.survives_rewrite);
    }

    /// ⛔⛔ **THE TRANSITIVE HALF OF THE STARVATION CRITERION — the one slice 5 shipped a regression
    /// without.** `p5_transparent_holder.ebnf`, rule for rule.
    ///
    /// `knot_a()` cannot pin it: there, the only holder of the transparent rule (`cast_expr`) DIES
    /// with the rewrite, so the transitive path is never load-bearing and a scan over DIRECT
    /// holders returns the same verdict. SystemVerilog reaches `casting_type` from `cast` as well,
    /// so its holder OUTLIVES the plan — and `initial k = 8'(1);` went ACCEPT → REJECT.
    ///
    /// Here `outer_cast` plays `cast`: it is named from `scratch`, so it survives, and it holds
    /// `ct` — a bare reference to `prim`, hence transparent — with a non-empty residual. `prim` must
    /// therefore be STARVED. ⭐ Under the pre-5b criterion it reported MAY-ABSORB, so this test is
    /// RED-provable against the exact defect it guards ([[a-check-whose-inputs-all-pass-has-not-been-tested]]).
    #[test]
    fn a_transparent_holder_that_outlives_the_rewrite_starves_the_base_rule() {
        let rule = |name: &str| ASTNode::Atom {
            value: ASTValue::Token(vec![
                TokenValue::String("rule_reference".to_string()),
                TokenValue::String(name.to_string()),
            ]),
        };
        let text = |literal: &str| ASTNode::Atom {
            value: ASTValue::Token(vec![
                TokenValue::String("quoted_string".to_string()),
                TokenValue::String(literal.to_string()),
            ]),
        };
        let cast_body = || ASTNode::Sequence {
            elements: vec![rule("ct"), text("'"), text("("), rule("lit"), text(")")],
        };

        let (mut grammar, mut order) = knot_a();
        // `scratch := prim` becomes `scratch := outer_cast | prim`, and `outer_cast` is `cast`.
        grammar.insert(
            "scratch".to_string(),
            ASTNode::Or {
                alternatives: vec![rule("outer_cast"), rule("prim")],
            },
        );
        grammar.insert("outer_cast".to_string(), cast_body());
        order.insert(1, "outer_cast".to_string());

        let survey = survey_indirect_left_recursion(&grammar, &order);
        let prim = survey
            .candidates
            .iter()
            .find(|candidate| candidate.base_rule == "prim")
            .expect("prim is still a candidate — the cycle is unchanged");

        assert!(
            !prim.is_starvation_safe(),
            "prim is starved THROUGH ct, which is transparent to it; a direct-holder scan sees \
             nothing here and that is precisely the regression"
        );
        let site = prim
            .surviving_starvation_sites()
            .into_iter()
            .find(|site| site.rule == "outer_cast")
            .expect("the surviving holder is outer_cast");
        assert_eq!(site.residual, "\"'\" \"(\" lit \")\"");
        assert!(
            !site.on_route,
            "outer_cast is not on prim's route at all — it holds the TRANSPARENT rule, not the base"
        );
        assert!(site.survives_rewrite);

        // ⭐ The one-difference control: drop `outer_cast` and this is P4 again, where `prim` IS
        // safe because the only holder of the transparent rule dies with the rewrite. If both
        // halves agreed, the criterion would be measuring nothing.
        let (control_grammar, control_order) = knot_a();
        let control = survey_indirect_left_recursion(&control_grammar, &control_order);
        assert!(
            control
                .candidates
                .iter()
                .find(|candidate| candidate.base_rule == "prim")
                .expect("prim is a candidate")
                .is_starvation_safe(),
            "without a surviving holder the same cycle must stay absorbable"
        );
    }

    #[test]
    fn clone_cost_is_one_rule_per_intermediate() {
        let (grammar, order) = knot_a();
        let survey = survey_indirect_left_recursion(&grammar, &order);
        let prim = survey
            .candidates
            .iter()
            .find(|c| c.base_rule == "prim")
            .expect("prim is a candidate");
        // ANTLR4's blow-up objection, priced: two clones (`cast_expr`, `ct`), not an exponential
        // closure. P3 hand-wrote exactly one because it also inlined `ct_seed := kw`.
        let cost = prim.clone_cost();
        assert_eq!(cost.len(), 2);
        assert!(cost.contains("cast_expr") && cost.contains("ct"));
    }

    #[test]
    fn a_nullable_left_corner_is_declined_not_guessed() {
        // `a := ( "x" )? b` reaches `b` at its left corner only THROUGH a nullable prefix, so the
        // residual is not `elements[1..]` and the survey must decline rather than invent one.
        let rule = |name: &str| ASTNode::Atom {
            value: ASTValue::Token(vec![
                TokenValue::String("rule_reference".to_string()),
                TokenValue::String(name.to_string()),
            ]),
        };
        let text = |literal: &str| ASTNode::Atom {
            value: ASTValue::Token(vec![
                TokenValue::String("quoted_string".to_string()),
                TokenValue::String(literal.to_string()),
            ]),
        };
        let mut grammar = HashMap::new();
        grammar.insert(
            "a".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    text("seed"),
                    ASTNode::Sequence {
                        elements: vec![
                            ASTNode::Quantified {
                                element: Box::new(text("x")),
                                quantifier: "?".to_string(),
                            },
                            rule("b"),
                        ],
                    },
                ],
            },
        );
        grammar.insert(
            "b".to_string(),
            ASTNode::Sequence {
                elements: vec![rule("a"), text("!")],
            },
        );
        let order: Vec<String> = ["a", "b"].iter().map(|s| s.to_string()).collect();
        let survey = survey_indirect_left_recursion(&grammar, &order);

        assert!(
            survey.candidates.iter().all(|c| c.base_rule != "a"),
            "the nullable-prefixed left corner must not produce a plan"
        );
        let declined = survey
            .declined
            .iter()
            .find(|d| d.rule == "a")
            .expect("the declined cycle is REPORTED, never silently dropped");
        assert_eq!(declined.reason.token(), "no_bare_left_corner_route");
    }

    #[test]
    fn an_empty_suffix_route_is_degenerate_and_refused() {
        // `a := "s" | b`, `b := a` — a real cycle whose every route iterates nothing. Turning it
        // into `a := a_base ( )*` would emit a non-consuming `*`.
        let rule = |name: &str| ASTNode::Atom {
            value: ASTValue::Token(vec![
                TokenValue::String("rule_reference".to_string()),
                TokenValue::String(name.to_string()),
            ]),
        };
        let text = |literal: &str| ASTNode::Atom {
            value: ASTValue::Token(vec![
                TokenValue::String("quoted_string".to_string()),
                TokenValue::String(literal.to_string()),
            ]),
        };
        let mut grammar = HashMap::new();
        grammar.insert(
            "a".to_string(),
            ASTNode::Or {
                alternatives: vec![text("s"), rule("b")],
            },
        );
        grammar.insert("b".to_string(), rule("a"));
        let order: Vec<String> = ["a", "b"].iter().map(|s| s.to_string()).collect();
        let survey = survey_indirect_left_recursion(&grammar, &order);

        assert!(survey.candidates.is_empty());
        assert_eq!(
            survey
                .declined
                .iter()
                .map(|d| d.reason.token())
                .collect::<Vec<_>>(),
            vec!["only_degenerate_routes", "only_degenerate_routes"]
        );
    }
}
