//! RGX-0078.5.i.7 (D2-A + D2-B + MTB-A + MTB-B) — the FUSED CASCADE emitter:
//! the full cascade fold (acyclic sub-regions + the cyclic spine), emitted
//! ENTIRELY in MATCH-THEN-BUILD form (the `-0093` derivation-tape design; the
//! `-0101` B increment extended the split across the cyclic spine — the
//! population carrying 95.6% of the doomed alloc BYTES per the `-0092`/`-0100`
//! census — so no eager fused value construction remains).
//!
//! For every rule in the cascade emission plan (the SHARED
//! `fusibility_census::compute_cascade_emission_plan_for_increment` at
//! `CascadeIncrement::CyclicSpine` — sub-roots + internal rules, every
//! cascade-eligible rule), this module emits the rule's fused form: the rule's
//! parse logic VERBATIM in semantics, with the per-rule protocol frame elided —
//! no per-rule entry counter, no coverage push, no trace scope/lines, no rule
//! transaction — and plain position-restore speculation wherever the
//! speculation scope provably cannot reach a semantic effect.
//!
//! EVERY fused rule splits into TWO static fn families (MTB-A
//! `PGEN-RGX-0078-0094` for the acyclic population; MTB-B `PGEN-RGX-0078-0101`
//! for the cyclic spine):
//!
//! - `cascade_match_<rule>` — the control flow VERBATIM minus ALL value
//!   construction, appending the committed-derivation TAPE
//!   (`crate::ast_pipeline::DerivEvent`: `OrWinner`/`QuantCount`/`OptPresent`
//!   placeholder-push-then-patch; `TokStart`/`TokEnd` only where a terminal
//!   span is dynamic) plus the boundary side vec (eager call-out values in
//!   append order). Every speculation-failure restore point truncates both to
//!   its marks IN THE CALLER'S FAILURE ARM (the
//!   [[feedback_question_bypasses_manual_cleanup]] rule applied to the tape);
//!   tournament winner segments compact IN-TAPE (`copy_within` + `truncate`).
//!   C3-B islands keep checkpoint / per-branch delta extraction + rollback /
//!   winner-delta replay VERBATIM, but island LOSERS build no values —
//!   `should_take` consumes only (end, priority, index). LICENSE: a fused rule
//!   carries no runtime directive (the cascade gate), so no fused value is
//!   ever consulted mid-parse; ineligible rules reference only SUB-ROOTS,
//!   whose orchestrators return full values.
//! - `cascade_build_<rule>` — the value half only, walked ONCE over the
//!   committed tape with the replayed `deriv_pos` cursor: the exact structural
//!   `ParseContent` + the rule/branch transforms (pure over content + spans).
//!   No matching, no guards; an out-of-shape tape read is a loud
//!   `unreachable!` (codegen drift), never an input error.
//!
//! SUB-ROOT `cascade_<rule>` fns become mark→match→build→truncate
//! ORCHESTRATORS at an UNCHANGED signature, so the observability-twin dispatch
//! and every fused/method call seam are untouched; INTERNAL rules get no
//! `cascade_<rule>` fn at all (only match/build fns reference them — the plan
//! partition). CYCLIC internal rules keep the D2-B protocol-mirror frame
//! inside their match fn — the recursion guard verbatim plus the thin memo
//! with a derivation-SEGMENT payload (`ThinTapeMemoEntry`): a valid hit
//! splices the cached `(end, event-segment, boundary-segment)` onto the live
//! tape and jumps the position (⛔ #49 — cycle participants never lose memo
//! protection); cyclic SUB-ROOTS keep their guard + real-memo protection at
//! the protocol frame that wraps the twin dispatch, exactly as under D2-B.
//!
//! ACYCLIC fused rules additionally elide the recursion-guard/parse-stack frame
//! (`check_cycle` is load-bearing only on a cycle — the Optim #16 argument) and
//! every memo lane (same-position re-probe multiplicity is bounded by the
//! grammar's static caller constant). CYCLE-PARTICIPATING internal fused rules
//! (the plan's `thin_memo`, D2-B) keep both, in lean form: the protocol-mirror
//! `check_cycle` + `enter`/`exit` guard frame, and the epoch-stamped THIN memo
//! (⛔ the session-#49 bound — a cyclic fused rule never loses memo
//! protection; see `ThinTapeMemoEntry` for the segment splice-replay
//! soundness argument). A cycle-participating SUB-ROOT needs neither in its cascade fn:
//! fused bodies call sub-roots as protocol METHODS, whose full frame already
//! carries the guard and the real memo.
//!
//! The fused graph is reached ONLY on the bare-parse path via the
//! observability-twin dispatch inside each sub-root's memoized body (see
//! `generate_rule_body_inner`); every diagnostic consumer (coverage, trace,
//! counters, memo stats) keeps the untouched protocol graph, so counters,
//! witnesses, and trace lines stay byte-exact by construction.
//!
//! The two ⛔ C3-B/store soundness rules from the D2 EMISSION DESIGN
//! (`docs/tasks/RGX-0078.md`, the `-0085` section) are encoded per SITE from the
//! plan's `effect_targets` set:
//! 1. a speculation scope whose subtree can reach a semantic effect runs under
//!    `try_parse` (position + coverage + parse-stack + SEMANTIC-CHECKPOINT
//!    snapshot/rollback — [[feedback_try_parse_must_snapshot_semantic_state]] at
//!    fused granularity);
//! 2. an `Or` site with ≥ 1 effect-reaching branch keeps the protocol tournament
//!    as a site ISLAND (checkpoint / per-branch delta extraction + rollback /
//!    winner-delta replay), because C3-B rolls every branch back and replays only
//!    the winner — plain position-restore would leak losing-successful branches'
//!    store effects.
//!
//! `furthest_position` parity is EXACT by construction: the protocol updates it
//! at rule entry only (one site per method), and every `cascade_<rule>` fn
//! performs the same max-update at its head, so both graphs write the same
//! positions at the same call structure; the FIRST/FIRST₂/Q-guard prune licenses
//! carry over verbatim because the emission reuses the SAME guard helpers
//! (`first_set_prune_guard_for_branch`, `degenerate_dispatch_byte_sets`,
//! `quantified_prune_guard_for_element`) the protocol graph is emitted with.

use super::super::{
    ASTNode, ASTValue, SemanticAssociativity, SemanticBranchPolicy, TokenValue,
    parse_quantifier_bounds,
};
use super::{AstBasedGenerator, BranchAnnotation};
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashSet;
use syn::Ident;

/// RGX-0078.5.j.2 STEP-2a — the direct-value build emitter (value fns,
/// in-place fold internals, discard walkers) consuming the corrected
/// `DirectValueBuildPlan`.
mod value;
use value::DvBranchMode;

/// The codegen-side cascade emission plan (the census plan's sets re-keyed for
/// O(1) per-rule queries plus a deterministic emission order).
pub(crate) struct CascadeCodegenPlan {
    /// Fused rules entered from OUTSIDE the fused graph: their methods keep the
    /// full protocol frame and gain the observability-twin dispatch.
    pub(crate) sub_roots: HashSet<String>,
    /// Fused rules referenced ONLY by fused rules: reached through
    /// `cascade_<rule>` fns exclusively on the bare-parse path.
    pub(crate) internal: HashSet<String>,
    /// The per-SITE C3-B test set: a speculation scope reaches semantic effects
    /// iff its subtree references any rule in here (ineligible rules ∪ the
    /// effect-reaching fixpoint — see `CascadeEmissionPlan::effect_targets`).
    pub(crate) effect_targets: HashSet<String>,
    /// RGX-0078.5.i.7 (D2-B) — the cycle-participating INTERNAL fused rules
    /// (`CascadeEmissionPlan::thin_memo ∩ internal`): each one's `cascade_<rule>`
    /// fn carries the recursion-guard frame (`check_cycle` + `enter`/`exit` —
    /// load-bearing exactly on a cycle, the Optim #16 argument inverted) and the
    /// epoch-stamped THIN memo (⛔ the session-#49 bound: a cyclic fused rule
    /// never loses memo protection). A cycle-participating SUB-ROOT is
    /// deliberately NOT in here: every bare-path entry to it goes through its
    /// protocol method (fused bodies call sub-roots as methods), whose full
    /// frame already provides `check_cycle`, the parse-stack frame, and the
    /// REAL memo — a second guard frame inside its cascade fn would see the
    /// method's own in-flight `(rule, position)` frame and falsely report
    /// `Infinite`.
    pub(crate) thin_memo_internal: HashSet<String>,
    /// All fused rules (sub-roots + internal), sorted — the deterministic
    /// emission order for the `cascade_<rule>` fns.
    ///
    /// RGX-0078.5.i.7 (MTB-B) — the ENTIRE fused partition is the
    /// match-then-build population: internal rules are
    /// `cascade_match_<rule>`/`cascade_build_<rule>` pairs (cyclic ones with
    /// the guard frame + the derivation-SEGMENT thin memo), sub-roots are
    /// mark→match→build→truncate orchestrators at the unchanged twin seam.
    pub(crate) fused_order: Vec<String>,
    /// RGX-0078.5.j.2 STEP-2a — the corrected direct-value partition
    /// (`compute_direct_value_build_plan`): rules whose fold is VALUE-PURE on
    /// every branch (node-signature build fns with in-place value internals).
    pub(crate) dv_barrier: HashSet<String>,
    /// Fused rules whose committed value is computed directly
    /// (`cascade_build_value_<rule>` — no node scaffolding at all).
    pub(crate) dv_value: HashSet<String>,
    /// Vocabulary-demoted rules (⊆ node_locked): verbatim on every branch.
    pub(crate) dv_demoted: HashSet<String>,
}

impl AstBasedGenerator {
    /// Build the cascade emission plan ONCE per generation from the SHARED census
    /// plan function (the P1a `build_inline_emission_plan` twin). `None` — the
    /// sound "no fused graph" default — when the analysis annotation table failed
    /// to compile (mirrors P1a: no annotations ⇒ default table ⇒ plan still
    /// builds; a compile FAILURE disables emission) or when the grammar has no
    /// sub-roots (nothing to dispatch ⇒ byte-identical pre-D2-A emission).
    pub(super) fn build_cascade_emission_plan_for_codegen(
        &self,
        grammar_tree: &std::collections::HashMap<String, ASTNode>,
        entry_rule: &str,
    ) {
        let plan: Option<CascadeCodegenPlan> = (|| {
            let _compiled = self.analysis_runtime_annotations()?;
            // RGX-0078.5.i.7 (D2-B) — the emitter now consumes the CYCLIC-SPINE
            // increment of the SHARED census plan: every cascade-eligible rule
            // is fused, sub-roots are the census's own full-fold region roots,
            // and `thin_memo` names the cycle participants (the ⛔ #49
            // carriers). The A increment remains computed by the census for
            // its report lanes; the partition logic is ONE implementation.
            let census_plan =
                crate::ast_pipeline::fusibility_census::compute_cascade_emission_plan_for_increment(
                    grammar_tree,
                    self.annotations.as_ref(),
                    Some(entry_rule),
                    crate::ast_pipeline::fusibility_census::CascadeIncrement::CyclicSpine,
                )
                .ok()?;
            if census_plan.sub_roots.is_empty() {
                return None;
            }
            // RGX-0078.5.i.7 (MTB-B) — the match-then-build population is the
            // WHOLE fused partition: every internal rule (acyclic AND cyclic)
            // is a `cascade_match_<rule>`/`cascade_build_<rule>` pair — cyclic
            // internals additionally carry the recursion-guard frame and the
            // derivation-SEGMENT thin memo (⛔ #49 held) — and every sub-root's
            // `cascade_<rule>` fn is a mark→match→build→truncate orchestrator
            // at the unchanged twin-dispatch signature. No eager fused value
            // construction remains; doomed derivations are truncated as tape
            // segments, never built.
            let mut fused_order: Vec<String> = census_plan
                .sub_roots
                .iter()
                .chain(census_plan.internal.iter())
                .cloned()
                .collect();
            fused_order.sort();
            let thin_memo_internal: HashSet<String> = census_plan
                .thin_memo
                .iter()
                .filter(|rule| census_plan.internal.contains(*rule))
                .cloned()
                .collect();
            // RGX-0078.5.j.2 STEP-2a — the direct-value partition from the
            // SAME census (single implementation). It can only fail where the
            // cascade plan itself fails (identical inputs), so `.ok()?` is the
            // same sound "no fused graph" default.
            let direct_value_plan =
                crate::ast_pipeline::fusibility_census::compute_direct_value_build_plan(
                    grammar_tree,
                    self.annotations.as_ref(),
                    Some(entry_rule),
                )
                .ok()?;
            crate::pgen_trace_debug!(
                "        D2-B cascade-emission plan: {} sub-root(s) with twin dispatch, {} internal rule(s) fused ({} thin-memo cyclic), {} effect target(s)",
                census_plan.sub_roots.len(),
                census_plan.internal.len(),
                thin_memo_internal.len(),
                census_plan.effect_targets.len(),
            );
            Some(CascadeCodegenPlan {
                sub_roots: census_plan.sub_roots.into_iter().collect(),
                internal: census_plan.internal.into_iter().collect(),
                effect_targets: census_plan.effect_targets.into_iter().collect(),
                thin_memo_internal,
                fused_order,
                dv_barrier: direct_value_plan.barrier.into_iter().collect(),
                dv_value: direct_value_plan.value_licensed.into_iter().collect(),
                dv_demoted: direct_value_plan.demoted.into_keys().collect(),
            })
        })();
        // Second `generate_parser_tokens` call on the same generator instance
        // keeps the first plan (OnceCell) — both calls saw the same tree snapshot,
        // so the plan is identical anyway (the P1a precedent).
        let _ = self.cascade_emission_plan.set(plan);
    }

    fn cascade_plan(&self) -> Option<&CascadeCodegenPlan> {
        self.cascade_emission_plan
            .get()
            .and_then(|plan| plan.as_ref())
    }

    /// Is the cascade plan active for this generation (≥ 1 sub-root)? Gates every
    /// emitted surface of the twin (struct fields, `parse()` routing, the
    /// accessor mark, the fused impl block) so plan-inactive generations are
    /// byte-identical to the pre-D2-A emission.
    pub(super) fn cascade_plan_active(&self) -> bool {
        self.cascade_plan().is_some()
    }

    /// Is `rule` a plan SUB-ROOT (its memoized body gains the twin dispatch)?
    pub(super) fn cascade_sub_root(&self, rule: &str) -> bool {
        self.cascade_plan()
            .is_some_and(|plan| plan.sub_roots.contains(rule))
    }

    /// Is `rule` plan-INTERNAL (a fused-body reference to it is a direct
    /// `cascade_<rule>` call rather than a method call)?
    fn cascade_internal(&self, rule: &str) -> bool {
        self.cascade_plan()
            .is_some_and(|plan| plan.internal.contains(rule))
    }

    /// RGX-0078.5.i.7 (D2-B) — is `rule` a cycle-participating INTERNAL fused
    /// rule (its `cascade_<rule>` fn carries the recursion-guard frame + the
    /// epoch-stamped thin memo)?
    fn cascade_thin_memo_internal(&self, rule: &str) -> bool {
        self.cascade_plan()
            .is_some_and(|plan| plan.thin_memo_internal.contains(rule))
    }

    /// RGX-0078.5.i.7 (D2-B) — does this generation carry ≥ 1 thin-memo rule
    /// (gates the thin-memo parser-struct fields — `-0205`: `thin_scratch` /
    /// `thin_entries` / `thin_stride` — + their constructor init, so a
    /// fully-acyclic grammar's artifact stays byte-identical to the D2-A
    /// emission)?
    pub(super) fn cascade_thin_memo_active(&self) -> bool {
        self.cascade_plan()
            .is_some_and(|plan| !plan.thin_memo_internal.is_empty())
    }

    /// RGX-0078.5.j.4 (`-0205`) — the thin-memo rules in SORTED order: the
    /// direct-index carrier assigns each rule a `THIN_ROW_<RULE>` row number,
    /// and the plan's set is a `HashSet`, so sorting here is what keeps the
    /// emitted constants (and therefore the whole artifact)
    /// regen-deterministic.
    pub(super) fn cascade_thin_memo_rows(&self) -> Vec<String> {
        let mut rows: Vec<String> = self
            .cascade_plan()
            .map(|plan| plan.thin_memo_internal.iter().cloned().collect())
            .unwrap_or_default();
        rows.sort();
        rows
    }

    /// RGX-0078.5.i.7 (MTB-B) — is the match-then-build split active? Under
    /// the B increment the MTB population IS the fused partition, so this is
    /// exactly plan-active. Gates the derivation-tape parser-struct fields +
    /// their init and the tape clear at `parse()` start.
    pub(super) fn cascade_mtb_active(&self) -> bool {
        self.cascade_plan_active()
    }

    /// RGX-0078.5.i.7 (MTB-B) — is `rule` an MTB SUB-ROOT (its
    /// `cascade_<rule>` fn is a mark→match→build→truncate orchestrator)?
    /// Under the B increment: every plan sub-root.
    fn mtb_sub_root(&self, rule: &str) -> bool {
        self.cascade_sub_root(rule)
    }

    /// RGX-0078.5.i.7 (MTB-B) — is `rule` MTB-INTERNAL (reached as a direct
    /// `cascade_match_<rule>`/`cascade_build_<rule>` pair from match/build fns
    /// exclusively)? Under the B increment: every plan-internal rule.
    fn mtb_internal(&self, rule: &str) -> bool {
        self.cascade_internal(rule)
    }

    /// RGX-0078.5.j.2 STEP-2a — is `rule` VALUE-LICENSED (its build fn is
    /// `cascade_build_value_<rule>() -> PgenValue`, no node form exists)?
    fn dv_value_licensed(&self, rule: &str) -> bool {
        self.cascade_plan()
            .is_some_and(|plan| plan.dv_value.contains(rule))
    }

    /// RGX-0078.5.j.2 STEP-2a — is `rule` a direct-value BARRIER (VALUE-PURE
    /// fold on every branch: node signature, in-place value internals)?
    fn dv_barrier_rule(&self, rule: &str) -> bool {
        self.cascade_plan()
            .is_some_and(|plan| plan.dv_barrier.contains(rule))
    }

    /// RGX-0078.5.j.2 STEP-2a — is `rule` NODE-LOCKED under the direct-value
    /// partition (fused, neither barrier nor value-licensed)?
    fn dv_node_locked_rule(&self, rule: &str) -> bool {
        self.cascade_plan().is_some_and(|plan| {
            (plan.sub_roots.contains(rule) || plan.internal.contains(rule))
                && !plan.dv_barrier.contains(rule)
                && !plan.dv_value.contains(rule)
        })
    }

    /// RGX-0078.5.j.2 STEP-2a — was `rule` vocabulary-DEMOTED (verbatim on
    /// every branch)?
    fn dv_demoted_rule(&self, rule: &str) -> bool {
        self.cascade_plan()
            .is_some_and(|plan| plan.dv_demoted.contains(rule))
    }

    /// ⛔ C3-B rule 1's per-SITE test: does `node`'s subtree reference any rule
    /// from whose body a semantic effect is reachable (the plan's
    /// `effect_targets` — ineligible rules ∪ the effect-reaching fixpoint)? A
    /// `true` verdict makes every speculation scope over this subtree run under
    /// `try_parse` (semantic snapshot/rollback) and makes an `Or` site keep the
    /// protocol tournament island.
    fn cascade_subtree_reaches_effects(&self, node: &ASTNode) -> bool {
        let Some(plan) = self.cascade_plan() else {
            // Unreachable from emission (fns below are only called with an active
            // plan); the conservative answer is the sound one.
            return true;
        };
        let mut refs: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        crate::ast_pipeline::fusibility_census::collect_ref_occurrences(node, &mut refs);
        refs.keys().any(|target| plan.effect_targets.contains(target))
    }

    /// The fused-graph impl block: one `cascade_<rule>` fn per plan rule, in the
    /// plan's deterministic order. Empty tokens when the plan is inactive.
    pub(super) fn generate_cascade_impl(
        &self,
        parser_name: &Ident,
        filename: &str,
    ) -> Result<TokenStream> {
        let Some(plan) = self.cascade_plan() else {
            return Ok(TokenStream::new());
        };
        let fused_order = plan.fused_order.clone();
        let mut cascade_fns: Vec<TokenStream> = Vec::with_capacity(fused_order.len());
        // Bodies come from the generation-entry gen-AST snapshot (the same tree
        // every protocol method is emitted from; regex atoms already rewritten
        // through `effective_regex_pattern`, which is idempotent at the atom
        // site — the P1a inlined-frame precedent).
        let grammar_tree = self.first_set_grammar_tree.borrow().clone();
        // RGX-0078.5.i.7 (D2-B) — the guard-frame license is a CYCLICITY claim
        // resolved twice (the census's `rule_reaches_itself` behind the plan's
        // `thin_memo`, and the generator's own `compute_recursive_rules` behind
        // the protocol methods' `check_cycle` emission). The two walk the same
        // rule-reference edges, so they must agree on every fused rule; a
        // disagreement means the fused graph and the protocol graph would guard
        // DIFFERENT rule sets — a loud codegen error, never a silent divergence
        // (the P1a re-entry-guard precedent).
        let recursive_rules = Self::compute_recursive_rules(&grammar_tree);
        for rule_name in &fused_order {
            let generator_cyclic = recursive_rules.contains(rule_name);
            let plan_thin = plan.thin_memo_internal.contains(rule_name);
            if plan_thin && !generator_cyclic {
                anyhow::bail!(
                    "cascade emission plan marks rule '{rule_name}' thin-memo (census-cyclic) but the generator's recursion analysis calls it non-recursive — the two cyclicity analyses drifted"
                );
            }
            if plan.internal.contains(rule_name) && !plan_thin && generator_cyclic {
                anyhow::bail!(
                    "generator recursion analysis calls fused internal rule '{rule_name}' recursive but the census plan carries no thin memo for it — the two cyclicity analyses drifted"
                );
            }
            // RGX-0078.5.i.7 (MTB-B) — `cascade_match_<a>`/`cascade_build_<a>`
            // share the `cascade_` namespace with `cascade_<b>`: a fused rule
            // literally named `match_<a>`/`build_<a>` for a fused rule `<a>`
            // would collide. Loud error, never a silent shadow.
            for prefix in ["match_", "build_", "build_value_"] {
                let colliding = format!("{prefix}{rule_name}");
                if plan.sub_roots.contains(&colliding) || plan.internal.contains(&colliding) {
                    anyhow::bail!(
                        "fused rule '{colliding}' collides with the emitted cascade_{prefix}{rule_name} function of fused rule '{rule_name}' — rename one of the grammar rules"
                    );
                }
            }
        }
        // RGX-0078.5.i.7 (MTB-A) — the derivation-tape read helpers, emitted
        // once per artifact when the match-then-build split is active.
        if self.cascade_mtb_active() {
            cascade_fns.push(quote! {
                /// RGX-0078.5.i.7 (MTB-A) — consume the next committed
                /// derivation-tape event (build pass only; the tape segment a
                /// build walks is committed by construction, so an
                /// out-of-shape read is a codegen drift, not an input error).
                /// RGX-0078.5.j.4 (`-0203`) — the tape is ONE packed-word
                /// lane: decode the word(s) at the unified cursor.
                #[inline]
                fn deriv_next_event(&mut self) -> crate::ast_pipeline::DerivEvent {
                    let (event, consumed) =
                        TapeWord::decode_event(&self.deriv_tape, self.deriv_cursor);
                    self.deriv_cursor += consumed;
                    event
                }
                /// RGX-0078.5.i.7 (MTB-A) — consume the next boundary call-out
                /// value (tape order is the implicit Boundary record).
                /// RGX-0078.5.j.4 (`-0203`) — the tag-0 word restores the
                /// arena reference through the carrier's sole hard-checked
                /// dereference.
                #[inline]
                fn deriv_next_boundary(&mut self) -> &'input ParseNode<'input> {
                    let node = self.deriv_tape[self.deriv_cursor].boundary_node();
                    self.deriv_cursor += 1;
                    node
                }
            });
        }
        for rule_name in &fused_order {
            let Some(ast_node) = grammar_tree.get(rule_name) else {
                anyhow::bail!(
                    "cascade emission plan names rule '{rule_name}' absent from the gen-AST tree"
                );
            };
            // RGX-0078.5.i.7 (MTB-B) — EVERY fused rule is match-then-build:
            // the match/build pair (cyclic internals get the guard frame +
            // the derivation-SEGMENT thin memo inside the match fn), plus the
            // orchestrator at sub-roots (the unchanged `cascade_<rule>` twin
            // seam). Internal rules get NO `cascade_<rule>` fn — by the plan
            // partition only match/build fns reference them.
            //
            // RGX-0078.5.j.2 STEP-2a — the build half is emitted per the
            // direct-value partition: value-licensed rules get the VALUE fn
            // (`cascade_build_value_*` — no node scaffolding); barrier and
            // node-locked rules keep the node signature (with per-branch
            // in-place modes inside `generate_mtb_build_rule_fn`).
            cascade_fns.push(self.generate_mtb_match_rule_fn(rule_name, ast_node, filename)?);
            if self.dv_value_licensed(rule_name) {
                if self.mtb_sub_root(rule_name) {
                    anyhow::bail!(
                        "direct-value partition drift: fused sub-root '{rule_name}' is value-licensed — a sub-root is always demanded (its orchestrator returns the node to the protocol zone)"
                    );
                }
                cascade_fns.push(self.generate_mtb_build_value_fn(rule_name, ast_node)?);
            } else {
                cascade_fns.push(self.generate_mtb_build_rule_fn(rule_name, ast_node)?);
            }
            if self.mtb_sub_root(rule_name) {
                cascade_fns.push(self.generate_mtb_orchestrator_fn(rule_name));
            }
        }
        Ok(quote! {
            // RGX-0078.5.j.4 (-0202) — the drop-free internal error carrier
            // for the fused graph (also consumed by the emitted bare terminal
            // twins and boundary conversion helpers). (-0203) — the unified
            // packed derivation-tape word. Emitted only when a cascade plan
            // is active, so no artifact carries an unused import.
            use crate::ast_pipeline::{CascadeControlError, CascadeResult, TapeWord};
            /// RGX-0078.5.i.7 (D2-A + D2-B + MTB-A) — the FUSED cascade graph:
            /// compact per-rule functions for the bare-parse path (no coverage /
            /// trace / counters / memo-stats consumer). Entered exclusively
            /// through the observability-twin dispatch at plan sub-root memoized
            /// bodies; every diagnostic consumer runs the untouched protocol
            /// methods. Cycle-participating internal rules carry the
            /// recursion-guard frame and the epoch-stamped thin memo (the ⛔ #49
            /// bound). ACYCLIC rules run match-then-build: `cascade_match_*`
            /// records the committed derivation on the tape, `cascade_build_*`
            /// constructs values ONCE over it, and sub-root `cascade_*` fns are
            /// mark→match→build→truncate orchestrators at an unchanged
            /// signature.
            impl<'input> #parser_name<'input> {
                #(#cascade_fns)*
            }
        })
    }

    /// A tournament branch's value transform over the captured `content` —
    /// resolution identical to the protocol emission (explicit annotation, else
    /// the synthetic `-> $1` single-element default, else pass-through). The
    /// fused form binds `content` by MOVE (the protocol's `raw_content.clone()`
    /// exists only to also feed the post/final raw capture, which is statically
    /// absent for a plan rule).
    fn cascade_branch_transform(
        &self,
        rule_name: &str,
        branch_index: usize,
        alternative: &ASTNode,
    ) -> Result<TokenStream> {
        let explicit_annotation: Option<BranchAnnotation> = self
            .branch_return_annotations
            .get(rule_name)
            .and_then(|branches| branches.get(branch_index).cloned())
            .flatten();
        let resolved_annotation: Option<BranchAnnotation> = explicit_annotation.or_else(|| {
            Self::synthesize_default_passthrough_for_single_element_branch(alternative)
        });
        Ok(match resolved_annotation {
            Some(annotation) => {
                self.generate_return_transform(&annotation, rule_name, &["content".to_string()])?
            }
            None => quote! { content },
        })
    }

    /// Does `rule_name` carry a rule-level matched-text `@transform` (the atom
    /// path the fused emission must never silently drop)? Mirrors the protocol
    /// atom path's detection: any semantic annotation named `transform`.
    fn cascade_rule_has_matched_text_transform(&self, rule_name: &str) -> bool {
        let Some(annotations) = &self.annotations else {
            return false;
        };
        let Some(entries) = annotations.semantic_annotations.get(rule_name) else {
            return false;
        };
        entries.iter().any(|annotation| {
            crate::ast_pipeline::semantic_directive_registry::semantic_directive_name_payload(
                annotation,
            )
            .is_some_and(|(name, _)| name == "transform")
        })
    }

    // ------------------------------------------------------------------
    // RGX-0078.5.i.7 (MTB-A + MTB-B) — the MATCH pass: the fused control
    // flow VERBATIM (guards, prune licenses, speculation classes, furthest
    // updates, C3-B island machinery) minus ALL value construction,
    // appending the committed-derivation tape (`DerivEvent`) + the boundary
    // side vec. Every fragment is a fully-terminated statement sequence.
    // Under MTB-B this is the ONLY fused emission: every fused rule is a
    // match/build pair; cyclic internals wrap the match fn in the guard
    // frame + the derivation-SEGMENT thin memo (⛔ #49).
    // ------------------------------------------------------------------

    /// The match half of a fused rule: `cascade_match_<rule>`. Control-flow
    /// mirror of the retired eager `cascade_<rule>` form with the value half
    /// (bindings, transforms, `ParseNode` creation) removed; cyclic internal
    /// rules additionally carry the D2-B protocol-mirror frame (recursion
    /// guard + thin memo, segment payload).
    fn generate_mtb_match_rule_fn(
        &self,
        rule_name: &str,
        ast_node: &ASTNode,
        filename: &str,
    ) -> Result<TokenStream> {
        let match_fn = format_ident!("cascade_match_{}", rule_name);
        let parse_logic = match ast_node {
            ASTNode::Or { alternatives } => {
                self.mtb_match_or_logic(alternatives, rule_name, filename, true)?
            }
            _ => self.mtb_match_node_logic(ast_node, rule_name, filename)?,
        };

        if self.cascade_thin_memo_internal(rule_name) {
            // RGX-0078.5.i.7 (MTB-B) — a CYCLE-PARTICIPATING internal rule in
            // match form: the D2-B protocol-mirror frame carries over verbatim
            // (⛔ #49 — cycle participants never lose memo protection), with
            // the memo PAYLOAD now a derivation SEGMENT instead of a value:
            //
            // 1. RECURSION GUARD — identical check/enter/exit to the eager
            //    D2-B form (the two-graph parity argument is unchanged).
            // 2. SEGMENT THIN MEMO — same key, same taint classes (PURE /
            //    STORE-READ via the stamp; a store-MUTATING body is never
            //    cached), same staleness eviction; a valid HIT splices the
            //    cached `(end, tape-segment)` onto the live tape and jumps
            //    the cursor — the build pass later constructs the value ONCE
            //    from the spliced words, so a memoized sub-derivation on a
            //    DOOMED path is truncated un-built. Tape records are
            //    tape-index-free (input positions / counts / branch indices /
            //    arena pointers only — the MTB-A compaction license), so a
            //    segment is position-independent within the tape and splicing
            //    is sound; input positions are absolute and the memo key pins
            //    the input position, so a hit replays at the exact recorded
            //    position. Boundary words carry arena refs (`Copy`), alive
            //    for the whole parse — the same replay economics as the eager
            //    entry's arena-borrow clone.
            //
            // Failure-arm tape hygiene stays with the CALLER's speculation
            // scope (the [[feedback_question_bypasses_manual_cleanup]] rule):
            // a failed body's pushes are truncated by the enclosing scope's
            // marks, and a cached FAILURE carries no segment.
            let rule_const = format_ident!("RULE_{}", rule_name.to_uppercase());
            let thin_row_const = format_ident!("THIN_ROW_{}", rule_name.to_uppercase());
            let recursion_guard_max_depth = super::GENERATED_RECURSION_GUARD_MAX_DEPTH;
            return Ok(quote! {
                fn #match_fn(&mut self) -> CascadeResult<()> {
                    let parser = self;
                    // The protocol method's rule-entry furthest update — the only
                    // furthest write site — mirrored one-for-one for exact parity.
                    if parser.position > parser.furthest_position {
                        parser.furthest_position = parser.position;
                    }
                    let position = parser.position;
                    match parser.recursion_guard.check_cycle_id(Self::#rule_const, position) {
                        CycleType::Infinite => {
                            // SV-CORPUS-GRAD.3.12 — recursion taint, mirrored
                            // one-for-one from the protocol guard so the two
                            // graphs cache exactly the same set of bodies.
                            parser.note_recursion_block(parser.recursion_guard.last_block_frame);
                            return Err(CascadeControlError::InvalidSyntax {
                                message: "Infinite recursion detected",
                                position,
                            });
                        }
                        CycleType::LeftRecursive => {
                            // SV-CORPUS-GRAD.3.12 — see above.
                            parser.note_recursion_block(parser.recursion_guard.last_block_frame);
                            return Err(CascadeControlError::InvalidSyntax {
                                message: "Left recursion detected",
                                position,
                            });
                        }
                        CycleType::MutualRecursive { depth, .. } if depth >= #recursion_guard_max_depth => {
                            // ENGINE-UNIVERSAL-SERVICES.43 — the depth ceiling
                            // names no blocking frame, so it is counted on its
                            // own channel rather than tainting the frame floor
                            // to `0`. Mirrored one-for-one from the protocol
                            // guard; see `ast_based_generator.rs`.
                            parser.recursion_depth_block_events += 1;
                            return Err(CascadeControlError::RecursionDepthExceeded {
                                position,
                                depth,
                            });
                        }
                        _ => {}
                    }
                    // RGX-0078.5.j.4 (`-0205`) — direct-index probe: one
                    // mul/add + one slot load + one generation compare
                    // replaces the map's hash + group probe; the stamp/taint
                    // classes and the splice replay are UNCHANGED.
                    let __pgen_thin_slot =
                        Self::#thin_row_const * parser.thin_stride + position;
                    let __pgen_thin_epoch = parser.semantic_runtime_state.write_epoch();
                    let __pgen_thin_deferred = parser.semantic_runtime_state.deferred_obligation_count();
                    let mut __pgen_thin_stale = false;
                    if let Some(__pgen_thin_idx) = parser.thin_scratch.lookup(__pgen_thin_slot) {
                        let __pgen_thin_entry = &parser.thin_entries[__pgen_thin_idx as usize];
                        let __pgen_thin_valid = match __pgen_thin_entry.stamp {
                            // PURE — neither read nor mutated: valid at any store state.
                            None => true,
                            // STORE-READ — valid while the store is unchanged since.
                            Some((__pgen_thin_e, __pgen_thin_d)) => {
                                __pgen_thin_e == __pgen_thin_epoch
                                    && __pgen_thin_d == __pgen_thin_deferred
                            }
                        };
                        if __pgen_thin_valid {
                            match &__pgen_thin_entry.outcome {
                                Some((__pgen_thin_end, __pgen_thin_seg)) => {
                                    // Disjoint-field borrows: `thin_entries`
                                    // is shared-borrowed while the tape vec is
                                    // mutably borrowed — distinct places.
                                    parser.deriv_tape.extend_from_slice(__pgen_thin_seg);
                                    parser.position = *__pgen_thin_end;
                                    return Ok(());
                                }
                                None => {
                                    return Err(CascadeControlError::Backtrack { position });
                                }
                            }
                        }
                        __pgen_thin_stale = true;
                    }
                    if __pgen_thin_stale {
                        // Stale eviction = slot clear; the superseded dense
                        // entry stays as bounded garbage (bounded by the same
                        // store-epoch churn that bounded the map's remove).
                        parser.thin_scratch.clear(__pgen_thin_slot);
                    }
                    // ⭐ ENGINE-UNIVERSAL-SERVICES.43 — DEPTH-GATED failure replay,
                    // the fused graph's mirror of the protocol memo's probe. Runs
                    // only once the whole-stack depth ceiling has fired at least
                    // once in this parse, so every ordinary parse pays a single
                    // predictable integer compare and never touches the map. The
                    // bare frame is pushed BELOW this point, so the depth this
                    // attempt would run at is `len() + 1` — the stamp was taken
                    // with the frame already pushed.
                    if parser.recursion_depth_block_events != 0 {
                        if let Some(&__pgen_thin_min_depth) = parser
                            .thin_fail_depth_gated
                            .get(&(Self::#rule_const, position))
                        {
                            if parser.recursion_guard.rule_id_stack.len() + 1
                                >= __pgen_thin_min_depth as usize
                            {
                                return Err(CascadeControlError::Backtrack { position });
                            }
                        }
                    }
                    let __pgen_thin_preds = parser.semantic_runtime_state.predicate_evaluations();
                    let __pgen_thin_mark = parser.deriv_tape.len();
                    // RGX-0078.5.j.4 (`-0200`) — BARE id-only frame: the ID
                    // stack is the complete cycle/depth representation
                    // (`check_cycle_id` scans it and counts its depth), and on
                    // the bare path every name reader is dead or reconstructs
                    // through RULE_NAMES, so the 24-byte name frame is not
                    // maintained here.
                    parser.recursion_guard.enter_id_bare(Self::#rule_const, position);
                    // SV-CORPUS-GRAD.3.12 — RECURSION-TAINT scope, the thin
                    // memo's mirror of the protocol memo's gate. Opened AFTER
                    // the bare frame is pushed, so `entry_depth - 1` is this
                    // rule's own index exactly as on the protocol path.
                    let __pgen_thin_saved_floor = parser.recursion_block_floor;
                    parser.recursion_block_floor = usize::MAX;
                    let __pgen_thin_entry_depth = parser.recursion_guard.rule_id_stack.len();
                    // ENGINE-UNIVERSAL-SERVICES.43 — the DEPTH channel is a
                    // monotone counter, so it needs no save/restore scope: "did
                    // the body trip the ceiling" is a compare against entry.
                    let __pgen_thin_depth_events = parser.recursion_depth_block_events;
                    let __pgen_thin_result: CascadeResult<()> =
                        (|parser: &mut Self| -> CascadeResult<()> {
                            #parse_logic
                            Ok(())
                        })(parser);
                    parser.recursion_guard.exit_bare();
                    // SV-CORPUS-GRAD.3.12 — close the taint scope and hand the
                    // floor up min'd with the caller's (see the protocol memo).
                    let __pgen_thin_recursion_floor = parser.recursion_block_floor;
                    parser.recursion_block_floor =
                        __pgen_thin_saved_floor.min(__pgen_thin_recursion_floor);
                    // Classify the body per the taint classes: a store-MUTATING
                    // body is never cached (segment replay would skip its
                    // effects); a store-READ body is cached with the
                    // unchanged-epoch stamp; a PURE body is cached
                    // unconditionally (the protocol's untainted license).
                    // SV-CORPUS-GRAD.3.12 adds a fourth, ORTHOGONAL class:
                    // RECURSION-TAINTED FAILURE — a cycle-guard rejection
                    // caused by a frame OUTSIDE this rule's own subtree — is
                    // never cached, for the same reason the protocol memo
                    // refuses it: the outcome is keyed by the live parse
                    // stack, which the `(row, position)` slot does not carry.
                    // A block owned by this rule or one of its descendants is
                    // re-created by every replay and is NOT taint, which is
                    // what the `floor < entry_depth - 1` test says; and a
                    // tainted SUCCESS stays cached, because a guard limits the
                    // SEARCH rather than the LANGUAGE (measured: refusing it
                    // regressed 4 corpus files pass→fail). Kept as a separate
                    // condition rather than folded into the stamp: the stamp
                    // is a *validatable* store epoch pair, and there is no
                    // recursion equivalent to validate against.
                    let __pgen_thin_mutated =
                        parser.semantic_runtime_state.write_epoch() != __pgen_thin_epoch
                            || parser.semantic_runtime_state.deferred_obligation_count()
                                != __pgen_thin_deferred;
                    // FAILURES ONLY — see the protocol memo's gate for the
                    // measured reason the success side must keep replaying.
                    let __pgen_thin_recursion_tainted = __pgen_thin_result.is_err()
                        && __pgen_thin_recursion_floor
                            < __pgen_thin_entry_depth.saturating_sub(1);
                    // ⭐ ENGINE-UNIVERSAL-SERVICES.43 — the DEPTH-scoped half of the
                    // recursion taint, mirroring the protocol memo. It keeps the
                    // failure OUT of the plain thin memo (which carries no depth
                    // condition, so replaying it from a shallower stack could
                    // refuse a parse the ceiling would have allowed) and files it
                    // in the depth-stamped side map instead. Discarding it is what
                    // turned the ceiling from a BOUND into an exponential search.
                    let __pgen_thin_depth_tainted = __pgen_thin_result.is_err()
                        && parser.recursion_depth_block_events != __pgen_thin_depth_events;
                    // ⛔ PURE bodies only, mirroring the protocol memo's
                    // `!memo_store_tainted`. A store-MUTATING body is never cached
                    // at all; a store-READ one is cached in the thin memo under a
                    // validatable epoch STAMP, and this side map carries a depth
                    // condition and no stamp — so composing the two here would
                    // drop the store condition on the floor and could replay a
                    // stale store-dependent refusal. The store axis keeps its own
                    // validation; this one keeps its own.
                    if __pgen_thin_depth_tainted
                        && !__pgen_thin_mutated
                        && parser.semantic_runtime_state.predicate_evaluations()
                            == __pgen_thin_preds
                    {
                        parser.thin_fail_depth_gated.insert(
                            (Self::#rule_const, position),
                            __pgen_thin_entry_depth as u32,
                        );
                    }
                    if !__pgen_thin_mutated
                        && !__pgen_thin_recursion_tainted
                        && !__pgen_thin_depth_tainted
                    {
                        let __pgen_thin_stamp =
                            if parser.semantic_runtime_state.predicate_evaluations()
                                == __pgen_thin_preds
                            {
                                None
                            } else {
                                Some((__pgen_thin_epoch, __pgen_thin_deferred))
                            };
                        // RGX-0078.5.j.4 (`-0205`) — direct-index insert:
                        // `Vec::push` + one slot store replace the map's
                        // hash + probe + ctrl/bucket write + growth
                        // amortization. The u32-index guard SKIPS caching
                        // past 2³²−1 entries (a memo skip is always sound;
                        // the +1 slot encoding needs idx+1 to fit u32).
                        if parser.thin_entries.len() < u32::MAX as usize {
                            let __pgen_thin_idx = parser.thin_entries.len() as u32;
                            match &__pgen_thin_result {
                                Ok(()) => {
                                    // RGX-0078.5.i.14 (C3) — the committed segment
                                    // is copied inline-small (POD `Copy` memcpy),
                                    // eliding the `Vec` malloc for the common short
                                    // segment. RGX-0078.5.j.4 (`-0203`) — the
                                    // unified tape makes it ONE copy; the inline
                                    // capacity is inferred from
                                    // `ThinTapeMemoEntry::outcome`.
                                    let __pgen_thin_seg = smallvec::SmallVec::from_slice(
                                        &parser.deriv_tape[__pgen_thin_mark..],
                                    );
                                    parser.thin_entries.push(
                                        crate::ast_pipeline::ThinTapeMemoEntry {
                                            stamp: __pgen_thin_stamp,
                                            outcome: Some((
                                                parser.position,
                                                __pgen_thin_seg,
                                            )),
                                        },
                                    );
                                }
                                Err(_) => {
                                    parser.thin_entries.push(
                                        crate::ast_pipeline::ThinTapeMemoEntry {
                                            stamp: __pgen_thin_stamp,
                                            outcome: None,
                                        },
                                    );
                                }
                            }
                            parser.thin_scratch.store(__pgen_thin_slot, __pgen_thin_idx);
                        }
                    }
                    __pgen_thin_result
                }
            });
        }

        Ok(quote! {
            fn #match_fn(&mut self) -> CascadeResult<()> {
                let parser = self;
                // The protocol method's rule-entry furthest update — the only
                // furthest write site — mirrored one-for-one for exact parity.
                if parser.position > parser.furthest_position {
                    parser.furthest_position = parser.position;
                }
                #parse_logic
                Ok(())
            }
        })
    }

    /// Construct dispatch for the match pass — the `cascade_node_logic` mirror.
    fn mtb_match_node_logic(
        &self,
        ast_node: &ASTNode,
        rule_name: &str,
        filename: &str,
    ) -> Result<TokenStream> {
        match ast_node {
            ASTNode::Or { alternatives } => {
                self.mtb_match_or_logic(alternatives, rule_name, filename, false)
            }
            ASTNode::Sequence { elements } => {
                self.mtb_match_sequence_logic(elements, rule_name, filename)
            }
            ASTNode::Atom { value } => self.mtb_match_atom_logic(value, rule_name),
            ASTNode::Quantified {
                element,
                quantifier,
            } => self.mtb_match_quantified_logic(element, quantifier, rule_name, filename),
            ASTNode::Lookahead { element, positive } => {
                self.mtb_match_lookahead_logic(element, *positive, rule_name, filename)
            }
        }
    }

    /// One match-pass speculation scope over a `ParseResult<()>` body — the
    /// `cascade_speculation_tokens` mirror plus TAPE hygiene: the derivation
    /// tape is position-like state `try_parse` does not manage, so the
    /// truncation lives in the CALLER's failure arm, never inside the closure
    /// (the [[feedback_question_bypasses_manual_cleanup]] rule applied to the
    /// tape).
    fn mtb_match_speculation_tokens(&self, subtree: &ASTNode, body: TokenStream) -> TokenStream {
        if self.cascade_subtree_reaches_effects(subtree) {
            quote! {
                let __pgen_spec_mark = parser.deriv_tape.len();
                // RGX-0078.5.j.4 (`-0200`) — the BARE wrapper: id-only
                // recursion-guard snapshot/restore (bare frames never touch
                // the name stack), otherwise the protocol mirror verbatim.
                let __pgen_attempt = parser.try_parse_bare(|p| {
                    let parser = p;
                    #body
                });
                if __pgen_attempt.is_none() {
                    parser.deriv_tape.truncate(__pgen_spec_mark);
                }
            }
        } else {
            quote! {
                let __pgen_attempt = {
                    let __pgen_spec_start = parser.position;
                    let __pgen_spec_mark = parser.deriv_tape.len();
                    match (|parser: &mut Self| -> CascadeResult<_> { #body })(parser) {
                        Ok(__pgen_speculated) => Some(__pgen_speculated),
                        Err(_) => {
                            parser.position = __pgen_spec_start;
                            parser.deriv_tape.truncate(__pgen_spec_mark);
                            None
                        }
                    }
                };
            }
        }
    }

    /// The `Or` match mirror: single-branch pass-through (no event), the P2
    /// byte-switch (no event — the build pass re-dispatches on the committed
    /// input byte), or the tournament with the `OrWinner`
    /// placeholder-push-then-patch and IN-TAPE winner-segment compaction
    /// (`copy_within` + `truncate` — POD moves, zero allocation). `should_take`
    /// consumes only (end, priority, index) — island LOSERS build no values
    /// (the `-0093` island refinement; the checkpoint / per-branch delta
    /// extraction + rollback / winner-delta replay machinery stays VERBATIM).
    fn mtb_match_or_logic(
        &self,
        alternatives: &[ASTNode],
        rule_name: &str,
        filename: &str,
        top_level: bool,
    ) -> Result<TokenStream> {
        let branch_count = alternatives.len();

        if branch_count == 1 {
            let branch_logic = self.mtb_match_node_logic(&alternatives[0], rule_name, filename)?;
            return Ok(branch_logic);
        }

        let branch_priorities = self.rule_branch_priorities(rule_name, branch_count);
        // GENERATED-LINT-CORRECTNESS.2 — with the tie-break folded below, the
        // `as_str()` spelling has no consumer in this emitter (it emits no trace
        // strings), exactly as `branch_policy_mode` lost its consumer in `.1`.
        let associativity = self.rule_associativity(rule_name);
        // GENERATED-LINT-CORRECTNESS.1 — the cascade graph emits no trace
        // strings, so once the policy is folded at codegen time (below) the
        // `as_str()` spelling has no remaining consumer here.
        let branch_policy = self.rule_branch_policy(rule_name);

        let emit_first_set_guard = top_level && self.layout_sensitivity().terminals;
        let mut first_set_cache: std::collections::HashMap<
            String,
            super::super::first_set::FirstSetSummary,
        > = std::collections::HashMap::new();
        let mut second_byte_cache: std::collections::HashMap<
            String,
            super::super::first_set::SecondByteSummary,
        > = std::collections::HashMap::new();
        // RGX-0078.5.j.4 K4b C1 — the FIRSTₖ per-rule prefix-trie cache.
        let mut prefix_trie_cache: std::collections::HashMap<
            String,
            super::super::first_set::PrefixTrieNode,
        > = std::collections::HashMap::new();

        // The P2 DEGENERATE byte-switch — the SAME shared gate as the eager
        // emission (and as `mtb_build_or_logic`, which re-runs it on identical
        // inputs), so match, build, and protocol graphs dispatch identically.
        if let Some(branch_byte_sets) = self.degenerate_dispatch_byte_sets(
            alternatives,
            rule_name,
            emit_first_set_guard,
            &mut first_set_cache,
        ) {
            let mut dispatch_arms = Vec::new();
            for (idx, alternative) in alternatives.iter().enumerate() {
                let branch_logic = self.mtb_match_node_logic(alternative, rule_name, filename)?;
                let byte_patterns = &branch_byte_sets[idx];
                let speculation = self.mtb_match_speculation_tokens(
                    alternative,
                    quote! {
                        #branch_logic
                        Ok(())
                    },
                );
                dispatch_arms.push(quote! {
                    #(#byte_patterns)|* => {
                        #speculation
                        if __pgen_attempt.is_none() {
                            return Err(CascadeControlError::Backtrack {
                                position: parse_start,
                            });
                        }
                    }
                });
            }
            return Ok(quote! {
                let parse_start = parser.position;
                if parse_start < parser.input.len() {
                    match parser.input.as_bytes()[parse_start] {
                        #(#dispatch_arms,)*
                        _ => {
                            return Err(CascadeControlError::Backtrack {
                                position: parse_start,
                            });
                        }
                    }
                } else {
                    return Err(CascadeControlError::Backtrack {
                        position: parse_start,
                    });
                }
            });
        }

        // ⛔ C3-B rule 2 — the tournament island verdict for this site.
        let island = alternatives
            .iter()
            .any(|alt| self.cascade_subtree_reaches_effects(alt));

        // GENERATED-LINT-CORRECTNESS.1 — resolve the branch policy HERE rather
        // than interpolating it as a string literal and re-asking in the emitted
        // parser. See the same fold in `ast_based_generator.rs`: the cascade
        // (bare) graph is the SECOND of the three emitters that carried this
        // shape, and together with `scan.rs` it accounted for HALF of every
        // affected parser's degenerate comparisons — the tree charter had named
        // only the protocol emitter.
        //
        // The winner-selection cascade — the protocol's exact chain over
        // (end, priority, index); `best_content.is_none()` becomes
        // `!__pgen_best_found` (no values exist on the match pass).
        // GENERATED-LINT-CORRECTNESS.2 — the associativity is a codegen-time
        // constant too, so the tie-break is resolved here instead of emitting
        // `match "left" { "right" => …, _ => false }`. No knock-on in this
        // emitter: `best_branch_index` is read unconditionally by the tape's
        // `DerivEvent::OrWinner(best_branch_index)`, so it stays live under every
        // associativity (unlike the protocol emitter, where it can go write-only).
        let associativity_tie_break = match associativity {
            SemanticAssociativity::Right => quote! { current_branch_index > best_branch_index },
            // Left keeps the incumbent on a tie. The bare graph has no
            // `nonassoc_tie` channel, so `nonassoc` degrades to the same
            // no-dethrone answer here exactly as it did through the `_` arm.
            SemanticAssociativity::Left | SemanticAssociativity::NonAssoc => quote! { false },
        };
        // ⚠️ When the tie-break folds to a literal `false`, the cascade's last `<`
        // arm and the final `else` would both read `false` — `clippy::needless_bool`,
        // and a fresh instance of the very defect class this tree removes. They are
        // merged instead: `<` and `==` both answer "do not take".
        let tie_break_is_constant_false =
            !matches!(associativity, SemanticAssociativity::Right);
        let should_take_chain = match branch_policy {
            SemanticBranchPolicy::Ordered => quote! {
                let should_take = !__pgen_best_found;
            },
            SemanticBranchPolicy::PriorityFirst if tie_break_is_constant_false => quote! {
                let should_take = if !__pgen_best_found {
                    true
                } else if candidate_priority > best_priority {
                    true
                } else if candidate_priority < best_priority {
                    false
                } else {
                    // Priorities tie: longer wins, and an end-tie does not dethrone,
                    // so the comparison IS the answer — emitted collapsed, or the
                    // tail would be a `clippy::needless_bool`.
                    candidate_end > best_end
                };
            },
            SemanticBranchPolicy::PriorityFirst => quote! {
                let should_take = if !__pgen_best_found {
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
                    #associativity_tie_break
                };
            },
            SemanticBranchPolicy::LongestMatch if tie_break_is_constant_false => quote! {
                let should_take = if !__pgen_best_found {
                    true
                } else if candidate_end > best_end {
                    true
                } else if candidate_end < best_end {
                    false
                } else {
                    // Ends tie: higher priority wins, and a priority-tie does not
                    // dethrone, so the comparison IS the answer — emitted collapsed,
                    // or the tail would be a `clippy::needless_bool`.
                    candidate_priority > best_priority
                };
            },
            SemanticBranchPolicy::LongestMatch => quote! {
                let should_take = if !__pgen_best_found {
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
                    #associativity_tie_break
                };
            },
        };
        let branch_policy_is_ordered = matches!(branch_policy, SemanticBranchPolicy::Ordered);

        let mut branch_attempt_blocks: Vec<TokenStream> = Vec::new();
        for (idx, alternative) in alternatives.iter().enumerate() {
            let branch_logic = self.mtb_match_node_logic(alternative, rule_name, filename)?;
            let branch_num = idx + 1;
            let branch_priority = branch_priorities.get(idx).copied().unwrap_or(0);
            let branch_index = idx;
            let first_set_prune_guard = self.first_set_prune_guard_for_branch(
                alternative,
                emit_first_set_guard,
                &mut first_set_cache,
                &mut second_byte_cache,
                &mut prefix_trie_cache,
            );

            // IN-TAPE winner-segment compaction: the candidate segment
            // (appended after the current best segment) memmoves down over
            // the best; when the best is empty the copy is a no-op onto
            // itself. Records carry no absolute tape indices, so the move is
            // safe by construction. RGX-0078.5.j.4 (`-0203`) — the unified
            // lane makes the winner's interleaved segment ONE contiguous
            // word range: one copy + one truncate.
            let take_compaction = quote! {
                let __pgen_cand_len = parser.deriv_tape.len() - __pgen_cand_start;
                parser.deriv_tape.copy_within(__pgen_cand_start.., __pgen_or_mark + 1);
                parser.deriv_tape.truncate(__pgen_or_mark + 1 + __pgen_cand_len);
            };

            let arm_body = if island {
                quote! {
                        // RGX-0078.5.j.4 K1 — deferred cleanup of the LIVE
                        // best branch before this body runs (branch
                        // isolation); its delta may still win, so extract it
                        // into `best_semantic_delta` first.
                        if __pgen_live_semantic_branch {
                            best_semantic_delta = Some(
                                parser
                                    .semantic_runtime_state
                                    .extract_delta_since(&tournament_semantic_checkpoint),
                            );
                            // RGX-0078.5.j.4 (`-0201`): bare-graph-only site —
                            // the bare rollback twin (observed-parse boundary).
                            parser.semantic_runtime_state.rollback_to_labeled_bare(
                                tournament_semantic_checkpoint.clone(),
                                crate::ast_pipeline::RollbackLabel::C3bBranchCleanup {
                                    rule: #rule_name,
                                    branch: #branch_num,
                                    total: #branch_count,
                                },
                            );
                            __pgen_live_semantic_branch = false;
                        }
                        parser.position = parse_start;
                        let __pgen_cand_start = parser.deriv_tape.len();
                        // RGX-0078.5.j.4 (`-0200`) — the BARE wrapper (id-only
                        // guard snapshot/restore), the protocol mirror otherwise.
                        if let Some(()) = parser.try_parse_bare(|p| {
                            let parser = p;
                            #branch_logic
                            Ok(())
                        }) {
                            let candidate_end = parser.position;
                            let candidate_priority: i64 = #branch_priority;
                            let current_branch_index: usize = #branch_index;
                            parser.position = parse_start;
                            #should_take_chain
                            // RGX-0078.5.j.4 K1 — the winner's effects stay
                            // LIVE (no eager extract/rollback round-trip); a
                            // loser is rolled back WITHOUT the extract its
                            // never-consumed delta would waste.
                            if should_take {
                                __pgen_live_semantic_branch = true;
                                best_semantic_delta = None;
                                #take_compaction
                                best_end = candidate_end;
                                best_priority = candidate_priority;
                                best_branch_index = current_branch_index;
                                __pgen_best_found = true;
                            } else {
                                // RGX-0078.5.j.4 (`-0201`): bare-graph-only
                                // site — the bare rollback twin.
                                parser.semantic_runtime_state.rollback_to_labeled_bare(
                                    tournament_semantic_checkpoint.clone(),
                                    crate::ast_pipeline::RollbackLabel::C3bBranchCleanup {
                                        rule: #rule_name,
                                        branch: #branch_num,
                                        total: #branch_count,
                                    },
                                );
                                parser.deriv_tape.truncate(__pgen_cand_start);
                            }
                        } else {
                            // Tape hygiene in the CALLER's failure arm —
                            // `try_parse` restores position/semantics only.
                            parser.deriv_tape.truncate(__pgen_cand_start);
                        }
                }
            } else {
                let speculation = self.mtb_match_speculation_tokens(
                    alternative,
                    quote! {
                        #branch_logic
                        Ok(())
                    },
                );
                quote! {
                        parser.position = parse_start;
                        let __pgen_cand_start = parser.deriv_tape.len();
                        #speculation
                        if let Some(()) = __pgen_attempt {
                            let candidate_end = parser.position;
                            let candidate_priority: i64 = #branch_priority;
                            let current_branch_index: usize = #branch_index;
                            parser.position = parse_start;
                            #should_take_chain
                            if should_take {
                                #take_compaction
                                best_end = candidate_end;
                                best_priority = candidate_priority;
                                best_branch_index = current_branch_index;
                                __pgen_best_found = true;
                            } else {
                                parser.deriv_tape.truncate(__pgen_cand_start);
                            }
                        }
                }
            };
            // GENERATED-LINT-CORRECTNESS.1 — only the `ordered` policy needs the
            // keep-first-winner short circuit; every other policy used to emit
            // `"longest_match" == "ordered" && __pgen_best_found` around the
            // whole branch body.
            let arm_inner = if branch_policy_is_ordered {
                quote! {
                    if __pgen_best_found {
                        // Ordered branch policy keeps the first successful branch.
                    } else {
                        #arm_body
                    }
                }
            } else {
                arm_body
            };

            let block = match first_set_prune_guard {
                Some(guard_condition) => quote! {
                    {
                        // FIRST/FIRST₂ prune guard — the same shared license as
                        // the protocol emission (top-level `Or` only).
                        if #guard_condition {
                            #arm_inner
                        }
                    }
                },
                None => quote! {
                    {
                        #arm_inner
                    }
                },
            };
            branch_attempt_blocks.push(block);
        }

        let island_prologue = if island {
            quote! {
                let tournament_semantic_checkpoint =
                    parser.semantic_runtime_state.checkpoint();
                let mut best_semantic_delta:
                    Option<crate::ast_pipeline::SemanticRuntimeDelta> = None;
                // RGX-0078.5.j.4 K1 — WINNER-IN-PLACE TOURNAMENT COMMIT
                // (the protocol emitter's lazy-cleanup twin, kept in
                // lockstep). TRUE ⇔ the current best branch's effects are
                // still applied; a later ATTEMPTED branch's preamble
                // extracts+rolls back; a winner with no later attempt
                // commits in place (`best_semantic_delta` stays `None`).
                let mut __pgen_live_semantic_branch = false;
            }
        } else {
            quote! {}
        };
        let island_winner_replay = if island {
            quote! {
                // RGX-0078.5.j.4 K1 — `__pgen_live_semantic_branch` ⇒ the
                // winner's effects never left the store and
                // `best_semantic_delta` is `None` (in-place commit, round-trip
                // elided); otherwise replay the extracted winner delta. The
                // no-branch-succeeded arm can never see a live branch
                // (live ⇒ `__pgen_best_found`).
                if let Some(delta) = best_semantic_delta {
                    if !delta.is_empty() {
                        parser.semantic_runtime_state.apply_delta(delta);
                    }
                }
            }
        } else {
            quote! {}
        };

        Ok(quote! {
            let parse_start = parser.position;
            let mut __pgen_best_found = false;
            let mut best_end = parse_start;
            let mut best_priority: i64 = i64::MIN;
            let mut best_branch_index: usize = 0usize;
            let __pgen_or_mark = parser.deriv_tape.len();
            parser.deriv_tape.push(TapeWord::narrow_event(
                crate::ast_pipeline::DerivEvent::OrWinner(0),
            ));
            #island_prologue
            // Branches evaluate in declaration order (partition rotation is
            // statically excluded by the shared cascade gate).
            #(#branch_attempt_blocks)*
            if __pgen_best_found {
                parser.position = best_end;
                // The patch is total: a branch index is narrow by construction.
                parser.deriv_tape[__pgen_or_mark] = TapeWord::narrow_event(
                    crate::ast_pipeline::DerivEvent::OrWinner(best_branch_index),
                );
                #island_winner_replay
            } else {
                parser.deriv_tape.truncate(__pgen_or_mark);
                return Err(CascadeControlError::Backtrack {
                    position: parse_start,
                });
            }
        })
    }

    /// The `Sequence` match mirror.
    fn mtb_match_sequence_logic(
        &self,
        elements: &[ASTNode],
        rule_name: &str,
        filename: &str,
    ) -> Result<TokenStream> {
        let mut element_parsers = Vec::new();
        for element in elements.iter() {
            element_parsers.push(self.mtb_match_sequence_element(element, rule_name, filename)?);
        }
        Ok(quote! {
            #(#element_parsers)*
        })
    }

    /// One sequence element on the match pass — the `cascade_sequence_element`
    /// mirror. The `?` fast path pushes the MANDATORY `OptPresent` placeholder
    /// (a static-literal inner produces zero events yet advances the build
    /// cursor) before the attempt and patches it on success; the Q-guard
    /// attempt elision keeps the placeholder at `false`.
    fn mtb_match_sequence_element(
        &self,
        element: &ASTNode,
        rule_name: &str,
        filename: &str,
    ) -> Result<TokenStream> {
        match element {
            ASTNode::Quantified {
                element: inner,
                quantifier,
            } if quantifier == "?" => {
                let inner_logic = self.mtb_match_node_logic(inner, rule_name, filename)?;
                let speculation = self.mtb_match_speculation_tokens(
                    inner,
                    quote! {
                        #inner_logic
                        Ok(())
                    },
                );
                let attempt = quote! {
                    #speculation
                    if __pgen_attempt.is_some() {
                        parser.deriv_tape[__pgen_opt_mark] = TapeWord::narrow_event(
                            crate::ast_pipeline::DerivEvent::OptPresent(true),
                        );
                    }
                };
                let attempt_with_guard = match self.quantified_prune_guard_for_element(inner) {
                    Some((bytes, emulate)) => {
                        let emulation = Self::quantified_guard_emulation_tokens(emulate);
                        quote! {
                            if parser.position < parser.input.len()
                                && matches!(parser.input.as_bytes()[parser.position], #(#bytes)|*)
                            {
                                #attempt
                            } else {
                                // Q-GUARD: FIRST-refuted optional — the attempt
                                // would fail at byte 1; elide it (exact furthest
                                // parity per the shared license).
                                #emulation
                            }
                        }
                    }
                    None => attempt,
                };
                Ok(quote! {
                    {
                        let __pgen_opt_mark = parser.deriv_tape.len();
                        parser.deriv_tape.push(TapeWord::narrow_event(
                            crate::ast_pipeline::DerivEvent::OptPresent(false),
                        ));
                        #attempt_with_guard
                    }
                })
            }
            _ => {
                let inner_logic = self.mtb_match_node_logic(element, rule_name, filename)?;
                Ok(quote! {
                    {
                        #inner_logic
                    }
                })
            }
        }
    }

    /// The `Atom` match mirror: terminals through the SAME `match_string` /
    /// `match_regex` helpers (layout policy inherited by construction), with
    /// terminal tape events STATICALLY ELIDED wherever the build cursor can
    /// re-derive the span (a layout-sensitive grammar's static literals emit
    /// none — the regex artifact carries ZERO terminal events). References:
    /// A-internal targets become `cascade_match_<rule>` calls; every other
    /// target (A-sub-root orchestrators, cyclic fused fns, sub-root/ineligible
    /// methods) stays an EAGER value call-out whose node is arena-allocated
    /// once and pushed to the boundary side vec (order is the implicit
    /// Boundary record).
    fn mtb_match_atom_logic(&self, value: &ASTValue, rule_name: &str) -> Result<TokenStream> {
        match value {
            ASTValue::Token(parts) if parts.len() >= 2 => {
                let TokenValue::String(token_type) = &parts[0];
                let TokenValue::String(token_value) = &parts[1];
                match token_type.as_str() {
                    "quoted_string" | "number" | "probability" | "include_dir" | "include_file"
                    | "rule" => {
                        // RGX-0078.5.j.4 (-0202) — fused bodies call the BARE
                        // terminal twins, which construct the Copy
                        // `CascadeControlError` at the source (the hottest
                        // producer: 1,076 sites in the regex artifact).
                        let match_call = Self::terminal_literal_match_call_bare(token_value);
                        if self.layout_sensitivity().terminals {
                            // No layout skip can precede this literal: the
                            // build cursor derives start AND end statically.
                            Ok(quote! {
                                parser.#match_call?;
                            })
                        } else {
                            // A layout skip may precede the literal: record
                            // the dynamic end (start = end − literal length).
                            Ok(quote! {
                                parser.#match_call?;
                                TapeWord::push_event(
                                    &mut parser.deriv_tape,
                                    crate::ast_pipeline::DerivEvent::TokEnd(parser.position),
                                );
                            })
                        }
                    }
                    "rule_reference" => {
                        if self.mtb_internal(token_value) {
                            // RGX-0078.5.i.7 (MTB-B) — EVERY fused internal
                            // rule (acyclic and cyclic alike) is a match-fn
                            // call; cyclic targets carry their own guard +
                            // segment thin memo inside `cascade_match_<r>`.
                            let match_target = format_ident!("cascade_match_{}", token_value);
                            Ok(quote! {
                                parser.#match_target()?;
                            })
                        } else if self.scan_rule(token_value) {
                            // RGX-0078.5.i.9 (D3) — a boundary reference to a
                            // scan-plan rule: fused bodies run ONLY on the bare
                            // path, so the frameless `scan_<rule>` replaces the
                            // full-frame protocol method unconditionally; its
                            // value rides the side vec exactly as before (the
                            // build pass is untouched).
                            let scan_target = format_ident!("scan_{}", token_value);
                            // RGX-0078.5.j.4 (-0202) — the scanner keeps its
                            // shared `ParseResult` signature (273 protocol-side
                            // call sites); the fused site converts its error
                            // into the Copy internal carrier (total: rich
                            // variants park).
                            Ok(quote! {
                                let __pgen_alt_child = match parser.#scan_target() {
                                    Ok(__pgen_v) => __pgen_v,
                                    Err(__pgen_e) => {
                                        return Err(parser.cascade_error_from_parse(__pgen_e));
                                    }
                                };
                                parser.deriv_tape.push(TapeWord::boundary(
                                    parser.arena.alloc(__pgen_alt_child),
                                ));
                            })
                        } else {
                            // A sub-root or an ineligible rule: a BOUNDARY
                            // call-out — the protocol method returns a full
                            // value, recorded on the side vec in append order.
                            // RGX-0078.5.j.4 (-0202) — the inbound conversion
                            // is total: the three Copy variants map 1:1, a
                            // rich/legacy error parks in the slot.
                            let method = format_ident!("parse_{}", token_value);
                            Ok(quote! {
                                let __pgen_alt_child = match parser.#method() {
                                    Ok(__pgen_v) => __pgen_v,
                                    Err(__pgen_e) => {
                                        return Err(parser.cascade_error_from_parse(__pgen_e));
                                    }
                                };
                                parser.deriv_tape.push(TapeWord::boundary(
                                    parser.arena.alloc(__pgen_alt_child),
                                ));
                            })
                        }
                    }
                    "regex" => {
                        if self.cascade_rule_has_matched_text_transform(rule_name) {
                            anyhow::bail!(
                                "cascade match emission reached regex atom of rule '{rule_name}' which carries a matched-text @transform — the shared cascade gate must have excluded it"
                            );
                        }
                        // GRAMMAR-WELLFORMED.H.16.4a — the ONE shared layout decision;
                        // `cascade_build_*` below calls the same predicate on the same
                        // atom, and the two must agree or the derivation tape drifts.
                        let skip_leading_whitespace =
                            self.regex_atom_skips_leading_layout(rule_name, token_value)?;
                        let effective_regex_pattern =
                            self.effective_regex_pattern(rule_name, token_value);
                        // The emitted `match_regex` skips layout iff the site
                        // requests it AND the grammar's regex tokens are
                        // layout-insensitive — mirror that static product for
                        // the TokStart elision.
                        let start_dynamic =
                            skip_leading_whitespace && !self.layout_sensitivity().regex_tokens;
                        // RGX-0078.5.j.4 (-0202) — `match_regex` keeps its
                        // shared `ParseResult` signature (protocol callers);
                        // the fused site converts its error into the Copy
                        // internal carrier (total: rich variants park).
                        if start_dynamic {
                            Ok(quote! {
                                let __pgen_matched = match parser
                                    .match_regex(#effective_regex_pattern, #skip_leading_whitespace)
                                {
                                    Ok(__pgen_v) => __pgen_v,
                                    Err(__pgen_e) => {
                                        return Err(parser.cascade_error_from_parse(__pgen_e));
                                    }
                                };
                                TapeWord::push_event(
                                    &mut parser.deriv_tape,
                                    crate::ast_pipeline::DerivEvent::TokStart(
                                        parser.position - __pgen_matched.len(),
                                    ),
                                );
                                TapeWord::push_event(
                                    &mut parser.deriv_tape,
                                    crate::ast_pipeline::DerivEvent::TokEnd(parser.position),
                                );
                            })
                        } else {
                            Ok(quote! {
                                if let Err(__pgen_e) = parser
                                    .match_regex(#effective_regex_pattern, #skip_leading_whitespace)
                                {
                                    return Err(parser.cascade_error_from_parse(__pgen_e));
                                }
                                TapeWord::push_event(
                                    &mut parser.deriv_tape,
                                    crate::ast_pipeline::DerivEvent::TokEnd(parser.position),
                                );
                            })
                        }
                    }
                    _ => Ok(quote! {}),
                }
            }
            _ => Ok(quote! {}),
        }
    }

    /// The `Quantified` match mirror: the unified (min, max) loop with the
    /// Q-guard attempt elision, the zero-length-match guard (the discarded
    /// zero-length success also discards its tape segment — the eager loop
    /// never pushes that value), the safety limit, and the `QuantCount`
    /// placeholder-push-then-patch.
    fn mtb_match_quantified_logic(
        &self,
        element: &ASTNode,
        quantifier: &str,
        rule_name: &str,
        filename: &str,
    ) -> Result<TokenStream> {
        let element_logic = self.mtb_match_node_logic(element, rule_name, filename)?;
        let (min, max) = match parse_quantifier_bounds(quantifier) {
            Some(bounds) => bounds,
            None => anyhow::bail!("Unknown quantifier: {}", quantifier),
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
        let (quantifier_start_position_bind, min_check_tokens) = if min > 0 {
            let min_lit = proc_macro2::Literal::usize_unsuffixed(min);
            (
                quote! { let quantifier_start_position = parser.position; },
                quote! {
                    if iteration_count < #min_lit {
                        parser.position = quantifier_start_position;
                        return Err(CascadeControlError::Backtrack {
                            position: quantifier_start_position,
                        });
                    }
                },
            )
        } else {
            (quote! {}, quote! {})
        };

        let quant_prune_guard = if min == 0 {
            self.quantified_prune_guard_for_element(element)
        } else {
            None
        };
        let quant_guard_tokens = match &quant_prune_guard {
            Some((bytes, emulate)) => {
                let emulation = Self::quantified_guard_emulation_tokens(*emulate);
                quote! {
                    // Q-GUARD: FIRST-refuted next byte ⇒ the attempt would fail
                    // at byte 1 — elide it (exit with the committed iterations).
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

        let speculation = self.mtb_match_speculation_tokens(
            element,
            quote! {
                #element_logic
                Ok(())
            },
        );

        Ok(quote! {
            #quantifier_start_position_bind
            let __pgen_quant_mark = parser.deriv_tape.len();
            parser.deriv_tape.push(TapeWord::narrow_event(
                crate::ast_pipeline::DerivEvent::QuantCount(0),
            ));
            let mut last_position = parser.position;
            let mut iteration_count: usize = 0;
            const SAFETY_LIMIT: usize = 10_000;

            loop {
                if iteration_count >= SAFETY_LIMIT {
                    break;
                }

                #max_check_tokens
                #quant_guard_tokens

                let __pgen_iter_mark = parser.deriv_tape.len();
                #speculation
                if let Some(()) = __pgen_attempt {
                    let current_position = parser.position;
                    // Zero-length match guard — prevent infinite loops on rules
                    // that can match the empty string. The eager loop discards
                    // the zero-length value (break without push): discard its
                    // tape segment identically.
                    if current_position == last_position {
                        parser.deriv_tape.truncate(__pgen_iter_mark);
                        break;
                    }
                    last_position = current_position;
                    iteration_count += 1;
                } else {
                    break;
                }
            }

            #min_check_tokens

            // The patch is total: the count is bounded by SAFETY_LIMIT.
            parser.deriv_tape[__pgen_quant_mark] = TapeWord::narrow_event(
                crate::ast_pipeline::DerivEvent::QuantCount(iteration_count),
            );
        })
    }

    /// The `Lookahead` match mirror. The probe's VALUE is discarded in the
    /// eager form (content is always the empty sequence), so its tape segment
    /// is truncated on success AND failure; semantic effects keep the
    /// protocol's observable behavior via the shared speculation helper (roll
    /// back on inner failure, persist on inner success).
    fn mtb_match_lookahead_logic(
        &self,
        element: &ASTNode,
        positive: bool,
        rule_name: &str,
        filename: &str,
    ) -> Result<TokenStream> {
        let inner_logic = self.mtb_match_node_logic(element, rule_name, filename)?;
        let speculation = self.mtb_match_speculation_tokens(
            element,
            quote! {
                #inner_logic
                Ok(())
            },
        );
        let failure_condition = if positive {
            quote! { __pgen_attempt.is_none() }
        } else {
            quote! { __pgen_attempt.is_some() }
        };
        Ok(quote! {
            let lookahead_start = parser.position;
            let __pgen_la_mark = parser.deriv_tape.len();
            #speculation
            parser.position = lookahead_start;
            parser.deriv_tape.truncate(__pgen_la_mark);
            if #failure_condition {
                return Err(CascadeControlError::Backtrack {
                    position: lookahead_start,
                });
            }
        })
    }

    // ------------------------------------------------------------------
    // RGX-0078.5.i.7 (MTB-A) — the BUILD pass: the value half only, walked
    // ONCE over the committed tape with the replayed `deriv_pos` cursor. No
    // matching, no guards, no speculation — an out-of-shape tape read is a
    // codegen drift (loud `unreachable!`), never an input error.
    // ------------------------------------------------------------------

    /// RGX-0078.5.i.7 (MTB-B, the `-0101` $text fix) — the shared transform
    /// emission slices `$text`/MatchedText as
    /// `input[start_pos..parser.position]`, written for the protocol/eager
    /// context where `parser.position` IS the rule end at transform time. In
    /// a build fn the cursor is `deriv_pos` and `parser.position` is frozen
    /// at the WHOLE match phase's end (measured: `(*LIMIT_HEAP=00700)a`
    /// carried value `"00700)a"` — the slice ran to parse end; the regex/svpp
    /// equivalence divergences were the same class). At a build-side
    /// transform site `deriv_pos` equals exactly what `position` was at the
    /// eager transform point, so a sync immediately before the transform is
    /// semantics-exact; the orchestrator restores `position` to the recorded
    /// match end after every build walk. Emitted ONLY when the rendered
    /// transform actually references `parser.position` (checked on the real
    /// emission tokens, so the guard cannot drift from the transform
    /// generator).
    fn build_transform_position_sync(transform: &TokenStream) -> TokenStream {
        if transform.to_string().contains("parser . position") {
            quote! {
                parser.position = parser.deriv_pos;
            }
        } else {
            quote! {}
        }
    }

    /// The build half of an A-population rule: `cascade_build_<rule>`.
    /// Value mirror of `generate_cascade_rule_fn` (structural `ParseContent`
    /// + the rule/branch transforms + the `ParseNode` with the replayed span).
    ///
    /// RGX-0078.5.j.2 STEP-2a — for barrier and node-locked rules the
    /// per-branch modes apply: an in-place branch computes its fold's content
    /// directly (no body scaffolding); a verbatim branch keeps today's
    /// emission byte-for-byte. Non-`Or` bodies resolve their single branch's
    /// mode here; `Or` bodies resolve per-arm inside `mtb_build_or_logic`.
    fn generate_mtb_build_rule_fn(&self, rule_name: &str, ast_node: &ASTNode) -> Result<TokenStream> {
        let build_fn = format_ident!("cascade_build_{}", rule_name);
        if !matches!(ast_node, ASTNode::Or { .. }) {
            let mode = self.dv_branch_build_mode(rule_name, 0, ast_node);
            if mode != DvBranchMode::Verbatim {
                let inplace = self.dv_inplace_branch_content(rule_name, 0, ast_node, &mode)?;
                return Ok(quote! {
                    fn #build_fn(&mut self) -> ParseNode<'input> {
                        let parser = self;
                        let start_pos = parser.deriv_pos;
                        let result = #inplace;
                        let end_pos = parser.deriv_pos;
                        ParseNode {
                            rule_name: &#rule_name,
                            content: result,
                            span: Span::new(start_pos, end_pos),
                        }
                    }
                });
            }
        }
        let build_logic = match ast_node {
            ASTNode::Or { alternatives } => {
                self.mtb_build_or_logic(alternatives, rule_name, true)?
            }
            _ => self.mtb_build_node_logic(ast_node, rule_name)?,
        };

        // Rule-level return annotation for non-`Or` roots — the eager tail
        // verbatim (the `Or` path applies per-branch transforms inline).
        let post_parse_transform_tokens: TokenStream = match ast_node {
            ASTNode::Or { .. } => quote! {},
            _ => {
                let annotation_opt = self
                    .branch_return_annotations
                    .get(rule_name)
                    .and_then(|branches| branches.first().cloned())
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
                    // The `-0101` $text fix — see `build_transform_position_sync`.
                    let position_sync = Self::build_transform_position_sync(&transform);
                    quote! {
                        #position_sync
                        let result = { #transform };
                    }
                } else {
                    quote! {}
                }
            }
        };

        Ok(quote! {
            fn #build_fn(&mut self) -> ParseNode<'input> {
                let parser = self;
                let start_pos = parser.deriv_pos;
                #build_logic
                #post_parse_transform_tokens
                let end_pos = parser.deriv_pos;
                ParseNode {
                    rule_name: &#rule_name,
                    content: result,
                    span: Span::new(start_pos, end_pos),
                }
            }
        })
    }

    /// Construct dispatch for the build pass — the `cascade_node_logic` mirror.
    fn mtb_build_node_logic(&self, ast_node: &ASTNode, rule_name: &str) -> Result<TokenStream> {
        match ast_node {
            ASTNode::Or { alternatives } => self.mtb_build_or_logic(alternatives, rule_name, false),
            ASTNode::Sequence { elements } => self.mtb_build_sequence_logic(elements, rule_name),
            ASTNode::Atom { value } => self.mtb_build_atom_logic(value, rule_name),
            ASTNode::Quantified {
                element,
                quantifier,
            } => self.mtb_build_quantified_logic(element, quantifier, rule_name),
            ASTNode::Lookahead { .. } => Ok(quote! {
                let result = ParseContent::Sequence(Vec::new());
            }),
        }
    }

    /// The `Or` build mirror: single-branch pass-through, the P2 byte-switch
    /// re-dispatch on the committed input byte at the replayed cursor
    /// (deterministic by construction — no event), or the `OrWinner`-driven
    /// winner build + branch transform. The committed tape holds ONLY the
    /// winner's segment (losers were compacted away on the match pass).
    fn mtb_build_or_logic(
        &self,
        alternatives: &[ASTNode],
        rule_name: &str,
        top_level: bool,
    ) -> Result<TokenStream> {
        let branch_count = alternatives.len();

        if branch_count == 1 {
            let branch = &alternatives[0];
            // RGX-0078.5.j.2 STEP-2a — an in-place single branch at the rule
            // top level replaces scaffolding + transform with the direct
            // content computation.
            if top_level {
                let mode = self.dv_branch_build_mode(rule_name, 0, branch);
                if mode != DvBranchMode::Verbatim {
                    let inplace = self.dv_inplace_branch_content(rule_name, 0, branch, &mode)?;
                    return Ok(quote! {
                        let result = #inplace;
                    });
                }
            }
            let branch_logic = self.mtb_build_node_logic(branch, rule_name)?;
            let resolved_annotation: Option<BranchAnnotation> = self
                .branch_return_annotations
                .get(rule_name)
                .and_then(|branches| branches.first().cloned())
                .flatten()
                .or_else(|| {
                    Self::synthesize_default_passthrough_for_single_element_branch(branch)
                });
            if let Some(annotation) = resolved_annotation {
                let transform = self.generate_return_transform(
                    &annotation,
                    rule_name,
                    &["result".to_string()],
                )?;
                if transform.to_string() == "result" {
                    return Ok(branch_logic);
                }
                // The `-0101` $text fix — see `build_transform_position_sync`.
                let position_sync = Self::build_transform_position_sync(&transform);
                return Ok(quote! {
                    #branch_logic
                    #position_sync
                    let result = { #transform };
                });
            }
            return Ok(branch_logic);
        }

        let emit_first_set_guard = top_level && self.layout_sensitivity().terminals;
        let mut first_set_cache: std::collections::HashMap<
            String,
            super::super::first_set::FirstSetSummary,
        > = std::collections::HashMap::new();

        // The SAME shared P2 gate on IDENTICAL inputs as the match pass — the
        // two passes cannot disagree about a site's dispatch form.
        if let Some(branch_byte_sets) = self.degenerate_dispatch_byte_sets(
            alternatives,
            rule_name,
            emit_first_set_guard,
            &mut first_set_cache,
        ) {
            let mut dispatch_arms = Vec::new();
            for (idx, alternative) in alternatives.iter().enumerate() {
                let byte_patterns = &branch_byte_sets[idx];
                // RGX-0078.5.j.2 STEP-2a — per-branch in-place modes at the
                // rule top level; nested dispatches stay verbatim.
                if top_level {
                    let mode = self.dv_branch_build_mode(rule_name, idx, alternative);
                    if mode != DvBranchMode::Verbatim {
                        let inplace =
                            self.dv_inplace_branch_content(rule_name, idx, alternative, &mode)?;
                        dispatch_arms.push(quote! {
                            #(#byte_patterns)|* => { #inplace }
                        });
                        continue;
                    }
                }
                let branch_logic = self.mtb_build_node_logic(alternative, rule_name)?;
                let transform = self.cascade_branch_transform(rule_name, idx, alternative)?;
                // The `-0101` $text fix — see `build_transform_position_sync`.
                let position_sync = Self::build_transform_position_sync(&transform);
                dispatch_arms.push(quote! {
                    #(#byte_patterns)|* => {
                        #branch_logic
                        {
                            #position_sync
                            let content = result;
                            #transform
                        }
                    }
                });
            }
            return Ok(quote! {
                let result = match parser.input.as_bytes()[parser.deriv_pos] {
                    #(#dispatch_arms,)*
                    __pgen_byte => unreachable!(
                        "derivation-tape drift in rule '{}': no byte-switch arm admits committed byte {}",
                        #rule_name,
                        __pgen_byte,
                    ),
                };
            });
        }

        let mut winner_arms: Vec<TokenStream> = Vec::new();
        for (idx, alternative) in alternatives.iter().enumerate() {
            // RGX-0078.5.j.2 STEP-2a — per-branch in-place modes at the rule
            // top level; nested dispatches stay verbatim.
            if top_level {
                let mode = self.dv_branch_build_mode(rule_name, idx, alternative);
                if mode != DvBranchMode::Verbatim {
                    let inplace =
                        self.dv_inplace_branch_content(rule_name, idx, alternative, &mode)?;
                    winner_arms.push(quote! {
                        #idx => { #inplace }
                    });
                    continue;
                }
            }
            let branch_logic = self.mtb_build_node_logic(alternative, rule_name)?;
            let transform = self.cascade_branch_transform(rule_name, idx, alternative)?;
            // The `-0101` $text fix — see `build_transform_position_sync`.
            let position_sync = Self::build_transform_position_sync(&transform);
            winner_arms.push(quote! {
                #idx => {
                    #branch_logic
                    {
                        #position_sync
                        let content = result;
                        #transform
                    }
                }
            });
        }
        Ok(quote! {
            let __pgen_or_winner = match parser.deriv_next_event() {
                crate::ast_pipeline::DerivEvent::OrWinner(__pgen_idx) => __pgen_idx,
                __pgen_other => unreachable!(
                    "derivation-tape drift in rule '{}': expected OrWinner, found {:?}",
                    #rule_name,
                    __pgen_other,
                ),
            };
            let result = match __pgen_or_winner {
                #(#winner_arms,)*
                __pgen_idx => unreachable!(
                    "derivation-tape drift in rule '{}': winner index {} out of range",
                    #rule_name,
                    __pgen_idx,
                ),
            };
        })
    }

    /// The `Sequence` build mirror — the eager element wrappers verbatim
    /// (`element_<i>` names, replayed spans, arena allocation).
    fn mtb_build_sequence_logic(&self, elements: &[ASTNode], rule_name: &str) -> Result<TokenStream> {
        let element_count = elements.len();
        let mut element_parsers = Vec::new();
        for (idx, element) in elements.iter().enumerate() {
            element_parsers.push(self.mtb_build_sequence_element(element, idx, rule_name)?);
        }
        Ok(quote! {
            let mut sequence_elements: Vec<&'input ParseNode<'input>> = Vec::with_capacity(#element_count);
            #(#element_parsers)*
            let result = ParseContent::Sequence(sequence_elements);
        })
    }

    /// One sequence element on the build pass. The `?` fast path consumes its
    /// mandatory `OptPresent` event; everything else derives from cursor
    /// replay.
    fn mtb_build_sequence_element(
        &self,
        element: &ASTNode,
        index: usize,
        rule_name: &str,
    ) -> Result<TokenStream> {
        let element_logic = match element {
            ASTNode::Quantified {
                element: inner,
                quantifier,
            } if quantifier == "?" => {
                let inner_logic = self.mtb_build_node_logic(inner, rule_name)?;
                quote! {
                    {
                        let __pgen_opt_present = match parser.deriv_next_event() {
                            crate::ast_pipeline::DerivEvent::OptPresent(__pgen_p) => __pgen_p,
                            __pgen_other => unreachable!(
                                "derivation-tape drift in rule '{}': expected OptPresent, found {:?}",
                                #rule_name,
                                __pgen_other,
                            ),
                        };
                        if __pgen_opt_present {
                            #inner_logic
                            result
                        } else {
                            ParseContent::Sequence(Vec::new())
                        }
                    }
                }
            }
            _ => {
                let inner_logic = self.mtb_build_node_logic(element, rule_name)?;
                quote! {
                    {
                        #inner_logic
                        result
                    }
                }
            }
        };

        let element_name = format!("element_{}", index);
        Ok(quote! {
            {
                let element_start = parser.deriv_pos;
                let element_content = #element_logic;
                let element_end = parser.deriv_pos;
                sequence_elements.push(parser.arena.alloc(ParseNode {
                    rule_name: &#element_name,
                    content: element_content,
                    span: Span::new(element_start, element_end),
                }));
            }
        })
    }

    /// The `Atom` build mirror: terminal spans from cursor replay (+ the
    /// `TokStart`/`TokEnd` events exactly where the match pass emitted them);
    /// A-internal references build recursively; every other reference splices
    /// the next boundary side-vec node (the eagerly-built call-out value) and
    /// advances the cursor to its span end.
    fn mtb_build_atom_logic(&self, value: &ASTValue, rule_name: &str) -> Result<TokenStream> {
        match value {
            ASTValue::Token(parts) if parts.len() >= 2 => {
                let TokenValue::String(token_type) = &parts[0];
                let TokenValue::String(token_value) = &parts[1];
                match token_type.as_str() {
                    "quoted_string" | "number" | "probability" | "include_dir" | "include_file"
                    | "rule" => {
                        let lit_len = token_value.len();
                        if self.layout_sensitivity().terminals {
                            Ok(quote! {
                                let __pgen_input: &'input str = parser.input;
                                let __pgen_tok_start = parser.deriv_pos;
                                parser.deriv_pos = __pgen_tok_start + #lit_len;
                                let result = ParseContent::Terminal(
                                    &__pgen_input[__pgen_tok_start..parser.deriv_pos],
                                );
                            })
                        } else {
                            Ok(quote! {
                                let __pgen_tok_end = match parser.deriv_next_event() {
                                    crate::ast_pipeline::DerivEvent::TokEnd(__pgen_e) => __pgen_e,
                                    __pgen_other => unreachable!(
                                        "derivation-tape drift in rule '{}': expected TokEnd, found {:?}",
                                        #rule_name,
                                        __pgen_other,
                                    ),
                                };
                                let __pgen_input: &'input str = parser.input;
                                let result = ParseContent::Terminal(
                                    &__pgen_input[__pgen_tok_end - #lit_len..__pgen_tok_end],
                                );
                                parser.deriv_pos = __pgen_tok_end;
                            })
                        }
                    }
                    "rule_reference" => {
                        if self.mtb_internal(token_value) {
                            // RGX-0078.5.j.2 STEP-2a — the partition-drift
                            // tripwire: a VERBATIM (node-form) build site may
                            // never reference a value-licensed rule (its node
                            // form does not exist). The census demand walk
                            // node-locks exactly the rules verbatim sites can
                            // reach; reaching one here means census and
                            // emission disagreed — fail codegen loudly.
                            if self.dv_value_licensed(token_value) {
                                anyhow::bail!(
                                    "direct-value partition drift: verbatim build site in rule '{rule_name}' references value-licensed rule '{token_value}' (no node form exists) — census demand and emission disagree"
                                );
                            }
                            let build_target = format_ident!("cascade_build_{}", token_value);
                            Ok(quote! {
                                let __pgen_alt_child = parser.#build_target();
                                let result = ParseContent::Alternative(parser.arena.alloc(__pgen_alt_child));
                            })
                        } else {
                            Ok(quote! {
                                let __pgen_alt_node = parser.deriv_next_boundary();
                                parser.deriv_pos = __pgen_alt_node.span.end as usize;
                                let result = ParseContent::Alternative(__pgen_alt_node);
                            })
                        }
                    }
                    "regex" => {
                        if self.cascade_rule_has_matched_text_transform(rule_name) {
                            anyhow::bail!(
                                "cascade build emission reached regex atom of rule '{rule_name}' which carries a matched-text @transform — the shared cascade gate must have excluded it"
                            );
                        }
                        // GRAMMAR-WELLFORMED.H.16.4a — the ONE shared layout decision, the
                        // same call `cascade_match_*` made on this atom (see there).
                        let skip_leading_whitespace =
                            self.regex_atom_skips_leading_layout(rule_name, token_value)?;
                        let start_dynamic =
                            skip_leading_whitespace && !self.layout_sensitivity().regex_tokens;
                        let start_tokens = if start_dynamic {
                            quote! {
                                let __pgen_tok_start = match parser.deriv_next_event() {
                                    crate::ast_pipeline::DerivEvent::TokStart(__pgen_s) => __pgen_s,
                                    __pgen_other => unreachable!(
                                        "derivation-tape drift in rule '{}': expected TokStart, found {:?}",
                                        #rule_name,
                                        __pgen_other,
                                    ),
                                };
                            }
                        } else {
                            quote! {
                                let __pgen_tok_start = parser.deriv_pos;
                            }
                        };
                        Ok(quote! {
                            #start_tokens
                            let __pgen_tok_end = match parser.deriv_next_event() {
                                crate::ast_pipeline::DerivEvent::TokEnd(__pgen_e) => __pgen_e,
                                __pgen_other => unreachable!(
                                    "derivation-tape drift in rule '{}': expected TokEnd, found {:?}",
                                    #rule_name,
                                    __pgen_other,
                                ),
                            };
                            let __pgen_input: &'input str = parser.input;
                            let result = ParseContent::Terminal(
                                &__pgen_input[__pgen_tok_start..__pgen_tok_end],
                            );
                            parser.deriv_pos = __pgen_tok_end;
                        })
                    }
                    _ => Ok(quote! {
                        let result = ParseContent::Terminal("");
                    }),
                }
            }
            _ => Ok(quote! {
                let result = ParseContent::Terminal("");
            }),
        }
    }

    /// The `Quantified` build mirror: `QuantCount` names the committed
    /// iteration count; each iteration rebuilds its element at the replayed
    /// cursor (the eager iteration nodes' `span: 0..0` is reproduced
    /// verbatim).
    fn mtb_build_quantified_logic(
        &self,
        element: &ASTNode,
        quantifier: &str,
        rule_name: &str,
    ) -> Result<TokenStream> {
        let element_logic = self.mtb_build_node_logic(element, rule_name)?;
        let quantifier_label = quantifier;
        Ok(quote! {
            let __pgen_quant_n = match parser.deriv_next_event() {
                crate::ast_pipeline::DerivEvent::QuantCount(__pgen_n) => __pgen_n,
                __pgen_other => unreachable!(
                    "derivation-tape drift in rule '{}': expected QuantCount, found {:?}",
                    #rule_name,
                    __pgen_other,
                ),
            };
            let mut results: Vec<&'input ParseNode<'input>> = Vec::new();
            for _ in 0..__pgen_quant_n {
                #element_logic
                results.push(parser.arena.alloc(ParseNode {
                    rule_name: &"quantified",
                    content: result,
                    span: Span::new(0, 0),
                }));
            }
            let result = ParseContent::Quantified(results, &#quantifier_label);
        })
    }

    /// RGX-0078.5.i.7 (MTB-A) — an A-SUB-ROOT's `cascade_<rule>` fn as a
    /// mark→match→build→truncate ORCHESTRATOR at the UNCHANGED signature: the
    /// twin dispatch, every fused reference, and every cyclic-spine call seam
    /// are untouched. Nesting (a sub-root called mid-match as a boundary
    /// call-out) is a clean stack discipline on the single tape: the inner
    /// orchestrator truncates back to its own mark before returning, and
    /// build never suspends into match, so the two build cursors are only
    /// live within one build walk at a time.
    fn generate_mtb_orchestrator_fn(&self, rule_name: &str) -> TokenStream {
        let cascade_fn = format_ident!("cascade_{}", rule_name);
        let match_fn = format_ident!("cascade_match_{}", rule_name);
        let build_fn = format_ident!("cascade_build_{}", rule_name);
        quote! {
            fn #cascade_fn(&mut self) -> ParseResult<ParseNode<'input>> {
                let parser = self;
                let __pgen_orch_mark = parser.deriv_tape.len();
                let __pgen_orch_start = parser.position;
                match parser.#match_fn() {
                    Ok(()) => {
                        let __pgen_match_end = parser.position;
                        parser.deriv_cursor = __pgen_orch_mark;
                        parser.deriv_pos = __pgen_orch_start;
                        let __pgen_node = parser.#build_fn();
                        // The end-parity drift tripwires: the built winner must
                        // land exactly on the recorded match end with the tape
                        // segment fully consumed.
                        debug_assert_eq!(
                            __pgen_node.span.end as usize,
                            __pgen_match_end,
                            "derivation-tape drift in rule '{}': build end != match end",
                            #rule_name,
                        );
                        debug_assert_eq!(
                            parser.deriv_cursor,
                            parser.deriv_tape.len(),
                            "derivation-tape drift in rule '{}': unconsumed tape words",
                            #rule_name,
                        );
                        parser.deriv_tape.truncate(__pgen_orch_mark);
                        // The `-0101` $text fix — a build-side transform may
                        // have synced `position` to its own site's cursor;
                        // restore the recorded match end so every enclosing
                        // seam (protocol memo insert, boundary caller) sees
                        // exactly the position the match pass established.
                        parser.position = __pgen_match_end;
                        Ok(__pgen_node)
                    }
                    Err(__pgen_err) => {
                        // Tape hygiene in the failure arm — the tape is
                        // position-like state the error path must unwind.
                        parser.deriv_tape.truncate(__pgen_orch_mark);
                        // RGX-0078.5.j.4 (-0202) — the region's ONLY outbound
                        // error edge: rehydrate the Copy internal carrier into
                        // the rich public `ParseError` (bijective; `Parked`
                        // takes the slot).
                        Err(parser.rehydrate_cascade_error(__pgen_err))
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::{
        ASTNode, ASTValue, Annotations, SemanticAnnotation, TokenValue, UnifiedSemanticAST,
        UnifiedSemanticValue,
    };
    use super::super::AstBasedGenerator;
    use std::collections::HashMap;

    /// A rule-level matched-text `@transform` — cascade-INELIGIBLE at the shared
    /// gate without exercising the runtime-directive compiler (the census test
    /// fixture's shape).
    fn transform_annotation() -> SemanticAnnotation {
        SemanticAnnotation::Named {
            name: "transform".to_string(),
            ast: UnifiedSemanticAST::Structured {
                canonical: String::new(),
                value: UnifiedSemanticValue::Boolean(true),
            },
        }
    }

    fn atom_ref(target: &str) -> ASTNode {
        ASTNode::Atom {
            value: ASTValue::Token(vec![
                TokenValue::String("rule_reference".to_string()),
                TokenValue::String(target.to_string()),
            ]),
        }
    }

    fn atom_lit(text: &str) -> ASTNode {
        ASTNode::Atom {
            value: ASTValue::Token(vec![
                TokenValue::String("quoted_string".to_string()),
                TokenValue::String(text.to_string()),
            ]),
        }
    }

    fn generator_for(annotations: Option<Annotations>) -> AstBasedGenerator {
        AstBasedGenerator {
            grammar_name: "cascade_test".to_string(),
            source_grammar_name: None,
            entry_rule: Some("entry".to_string()),
            logger: None,
            annotations,
            branch_return_annotations: HashMap::new(),
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            profile_gate_bypass_needed: std::cell::RefCell::new(None),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
            scan_emission_plan: std::cell::OnceCell::new(),
        }
    }

    /// entry → wrapper → leaf, all effect-free: wrapper/leaf are plan-internal,
    /// entry is the sole sub-root; the fused impl carries one fn per plan rule;
    /// internal references are direct cascade calls; the twin dispatch appears
    /// in the sub-root's body and NOT in internal rules' bodies.
    #[test]
    fn cascade_emits_fused_fns_with_internal_calls_and_twin_dispatch_at_sub_roots() {
        let mut tree: HashMap<String, ASTNode> = HashMap::new();
        tree.insert(
            "entry".to_string(),
            ASTNode::Sequence {
                elements: vec![atom_ref("wrapper")],
            },
        );
        tree.insert(
            "wrapper".to_string(),
            ASTNode::Sequence {
                elements: vec![atom_ref("leaf")],
            },
        );
        tree.insert(
            "leaf".to_string(),
            ASTNode::Sequence {
                elements: vec![atom_lit("x")],
            },
        );

        let generator = generator_for(Some(Annotations::default()));
        *generator.first_set_grammar_tree.borrow_mut() = tree.clone();
        generator.build_cascade_emission_plan_for_codegen(&tree, "entry");

        assert!(generator.cascade_plan_active(), "plan must be active");
        assert!(generator.cascade_sub_root("entry"), "entry is the sub-root");
        assert!(
            generator.cascade_internal("wrapper") && generator.cascade_internal("leaf"),
            "wrapper/leaf are internal"
        );

        let parser_name = quote::format_ident!("CascadeTestParser");
        let rendered = generator
            .generate_cascade_impl(&parser_name, "cascade_test.rs")
            .expect("cascade impl generation should succeed")
            .to_string();

        // RGX-0078.5.i.7 (MTB-A) — a fully-acyclic plan is entirely in the
        // match-then-build population: the sub-root's `cascade_<rule>` fn is
        // the mark→match→build→truncate ORCHESTRATOR, internal rules get
        // match/build pairs and NO `cascade_<rule>` fn at all.
        assert!(
            generator.cascade_mtb_active()
                && generator.mtb_sub_root("entry")
                && generator.mtb_internal("wrapper")
                && generator.mtb_internal("leaf"),
            "the acyclic plan is the MTB population"
        );
        assert!(
            rendered.contains("fn cascade_entry")
                && rendered.contains("fn cascade_match_entry")
                && rendered.contains("fn cascade_build_entry"),
            "the sub-root gets the orchestrator + its match/build pair, got: {rendered}"
        );
        assert!(
            rendered.contains("fn cascade_match_wrapper")
                && rendered.contains("fn cascade_build_wrapper")
                && rendered.contains("fn cascade_match_leaf")
                && rendered.contains("fn cascade_build_leaf"),
            "internal rules get match/build pairs, got: {rendered}"
        );
        assert!(
            !rendered.contains("fn cascade_wrapper") && !rendered.contains("fn cascade_leaf"),
            "A-internal rules get no eager cascade fn, got: {rendered}"
        );
        assert!(
            rendered.contains("cascade_match_wrapper ()")
                && rendered.contains("cascade_match_leaf ()")
                && rendered.contains("cascade_build_wrapper ()")
                && rendered.contains("cascade_build_leaf ()"),
            "internal references are direct match/build calls, got: {rendered}"
        );
        assert!(
            !rendered.contains("parse_wrapper ()"),
            "an internal reference must not fall back to the protocol method, got: {rendered}"
        );
        assert!(
            !rendered.contains("memoized_call") && !rendered.contains("recursion_guard"),
            "the fused graph carries no protocol frame, got: {rendered}"
        );
        // The orchestrator shape: marks, match→build, the end-parity drift
        // tripwires, truncation on both arms.
        assert!(
            rendered.contains("__pgen_orch_mark")
                && rendered.contains("deriv_cursor")
                && rendered.contains("debug_assert_eq !")
                && rendered.contains("deriv_tape . truncate"),
            "the orchestrator marks, builds over the segment, asserts parity, and truncates, got: {rendered}"
        );

        // The twin dispatch: present in the SUB-ROOT's body, absent from an
        // internal rule's body.
        let entry_body = generator
            .generate_rule_body_inner("entry", tree.get("entry").unwrap(), "cascade_test.rs")
            .expect("entry body generation should succeed")
            .to_string();
        assert!(
            entry_body.contains("bare_parse") && entry_body.contains("cascade_entry"),
            "sub-root body carries the twin dispatch, got: {entry_body}"
        );
        let wrapper_body = generator
            .generate_rule_body_inner("wrapper", tree.get("wrapper").unwrap(), "cascade_test.rs")
            .expect("wrapper body generation should succeed")
            .to_string();
        assert!(
            !wrapper_body.contains("bare_parse"),
            "internal rule bodies carry no dispatch, got: {wrapper_body}"
        );
    }

    /// An `Or` with a branch that reaches a directive-carrying rule keeps the
    /// protocol tournament ISLAND (checkpoint + C3-B cleanup + winner replay),
    /// and the directive carrier itself stays a method call-out; an all-clean
    /// `Or` gets the plain tournament with no semantic machinery.
    #[test]
    fn cascade_or_islands_on_effect_reaching_branches_only() {
        let mut annotations = Annotations::default();
        annotations
            .semantic_annotations
            .insert("fact_writer".to_string(), vec![transform_annotation()]);

        let mut tree: HashMap<String, ASTNode> = HashMap::new();
        tree.insert(
            "entry".to_string(),
            ASTNode::Or {
                alternatives: vec![atom_ref("clean_pair"), atom_ref("dirty_pair")],
            },
        );
        tree.insert(
            "clean_pair".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    ASTNode::Sequence {
                        elements: vec![atom_lit("a"), atom_lit("b")],
                    },
                    ASTNode::Sequence {
                        elements: vec![atom_lit("c"), atom_lit("d")],
                    },
                ],
            },
        );
        tree.insert(
            "dirty_pair".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    ASTNode::Sequence {
                        elements: vec![atom_lit("e"), atom_lit("f")],
                    },
                    ASTNode::Sequence {
                        elements: vec![atom_lit("g"), atom_ref("fact_writer")],
                    },
                ],
            },
        );
        tree.insert(
            "fact_writer".to_string(),
            ASTNode::Sequence {
                elements: vec![atom_lit("z")],
            },
        );

        let generator = generator_for(Some(annotations));
        *generator.first_set_grammar_tree.borrow_mut() = tree.clone();
        generator.build_cascade_emission_plan_for_codegen(&tree, "entry");

        assert!(generator.cascade_plan_active(), "plan must be active");
        assert!(
            !generator.cascade_internal("fact_writer") && !generator.cascade_sub_root("fact_writer"),
            "the directive carrier is not fused"
        );

        // RGX-0078.5.i.7 (MTB-B) — the match mirror is the ONLY fused Or
        // emission now; the island/plain verdicts are unchanged.
        let clean = generator
            .mtb_match_or_logic(
                match tree.get("clean_pair").unwrap() {
                    ASTNode::Or { alternatives } => alternatives,
                    _ => unreachable!(),
                },
                "clean_pair",
                "cascade_test.rs",
                true,
            )
            .expect("clean Or generation should succeed")
            .to_string();
        assert!(
            !clean.contains("tournament_semantic_checkpoint") && !clean.contains("try_parse"),
            "an effect-free Or site carries no semantic machinery, got: {clean}"
        );

        let dirty = generator
            .mtb_match_or_logic(
                match tree.get("dirty_pair").unwrap() {
                    ASTNode::Or { alternatives } => alternatives,
                    _ => unreachable!(),
                },
                "dirty_pair",
                "cascade_test.rs",
                true,
            )
            .expect("dirty Or generation should succeed")
            .to_string();
        assert!(
            dirty.contains("tournament_semantic_checkpoint")
                && dirty.contains("C3bBranchCleanup")
                && dirty.contains("apply_delta"),
            "an effect-reaching Or site keeps the protocol tournament island, got: {dirty}"
        );
        assert!(
            dirty.contains("parse_fact_writer"),
            "the boundary call-out stays a protocol method call, got: {dirty}"
        );
    }

    /// A grammar whose entry carries a runtime directive fuses nothing at the
    /// entry (it is ineligible), but its clean subtree still folds: the eligible
    /// rules referenced by the ineligible entry become sub-roots.
    #[test]
    fn cascade_plan_roots_at_the_ineligible_boundary() {
        let mut annotations = Annotations::default();
        annotations
            .semantic_annotations
            .insert("entry".to_string(), vec![transform_annotation()]);

        let mut tree: HashMap<String, ASTNode> = HashMap::new();
        tree.insert(
            "entry".to_string(),
            ASTNode::Sequence {
                elements: vec![atom_ref("clean")],
            },
        );
        tree.insert(
            "clean".to_string(),
            ASTNode::Sequence {
                elements: vec![atom_lit("x")],
            },
        );

        let generator = generator_for(Some(annotations));
        *generator.first_set_grammar_tree.borrow_mut() = tree.clone();
        generator.build_cascade_emission_plan_for_codegen(&tree, "entry");

        assert!(generator.cascade_plan_active(), "plan must be active");
        assert!(
            !generator.cascade_sub_root("entry") && !generator.cascade_internal("entry"),
            "the ineligible entry is not fused"
        );
        assert!(
            generator.cascade_sub_root("clean"),
            "the eligible rule referenced by the ineligible entry is a sub-root"
        );
    }

    /// RGX-0078.5.i.7 (D2-B + MTB-B) — a CYCLE-PARTICIPATING internal rule
    /// fuses under the CyclicSpine increment as a match/build pair: its MATCH
    /// fn recurses through direct match calls and carries the recursion-guard
    /// frame + the epoch-stamped thin memo with a derivation-SEGMENT payload
    /// (`ThinTapeMemoEntry` — hits splice, inserts capture the segment); an
    /// acyclic rule in the same plan keeps the frame-free match/build shape.
    #[test]
    fn cascade_cyclic_internal_rule_gets_guard_frame_and_thin_memo() {
        let mut tree: HashMap<String, ASTNode> = HashMap::new();
        tree.insert(
            "entry".to_string(),
            ASTNode::Sequence {
                elements: vec![atom_ref("cyc"), atom_ref("leaf")],
            },
        );
        // cyc → "(" cyc? — reaches itself, so it is census-cyclic AND
        // generator-recursive (the drift assert's agreeing case).
        tree.insert(
            "cyc".to_string(),
            ASTNode::Sequence {
                elements: vec![
                    atom_lit("("),
                    ASTNode::Quantified {
                        element: Box::new(atom_ref("cyc")),
                        quantifier: "?".to_string(),
                    },
                ],
            },
        );
        tree.insert(
            "leaf".to_string(),
            ASTNode::Sequence {
                elements: vec![atom_lit("x")],
            },
        );

        let generator = generator_for(Some(Annotations::default()));
        *generator.first_set_grammar_tree.borrow_mut() = tree.clone();
        generator.build_cascade_emission_plan_for_codegen(&tree, "entry");

        assert!(generator.cascade_plan_active(), "plan must be active");
        assert!(
            generator.cascade_internal("cyc"),
            "the cyclic rule fuses as internal under the CyclicSpine increment"
        );
        assert!(
            generator.cascade_thin_memo_active(),
            "a cyclic internal rule activates the thin memo"
        );

        let parser_name = quote::format_ident!("CascadeTestParser");
        let rendered = generator
            .generate_cascade_impl(&parser_name, "cascade_test.rs")
            .expect("cascade impl generation should succeed")
            .to_string();

        // RGX-0078.5.i.7 (MTB-B) — EVERY fused rule is in the MTB population:
        // internal rules (acyclic `leaf` AND cyclic `cyc`) get match/build
        // pairs with no eager `cascade_<rule>` fn; the sub-root `entry` gets
        // the orchestrator plus its own pair. The cyclic internal rule's
        // MATCH fn carries the protocol-mirror frame (guard + thin memo) with
        // the memo payload now a derivation SEGMENT (`ThinTapeMemoEntry`).
        assert!(
            generator.cascade_mtb_active()
                && generator.mtb_sub_root("entry")
                && generator.mtb_internal("leaf")
                && generator.mtb_internal("cyc"),
            "the whole fused partition is the MTB population under B"
        );
        assert!(
            !rendered.contains("fn cascade_leaf") && !rendered.contains("fn cascade_cyc "),
            "internal rules get no eager cascade fn, got: {rendered}"
        );
        let entry_match_start = rendered
            .find("fn cascade_match_entry")
            .expect("the sub-root gets a match fn");
        let entry_match_body = &rendered[entry_match_start
            ..rendered[entry_match_start + 1..]
                .find("fn cascade_")
                .map(|off| entry_match_start + 1 + off)
                .unwrap_or(rendered.len())];
        assert!(
            entry_match_body.contains("cascade_match_cyc ()")
                && !entry_match_body.contains("TapeWord :: boundary"),
            "a match-fn reference to a cyclic INTERNAL rule is a direct match call (no boundary call-out), got: {entry_match_body}"
        );
        let cyc_fn_start = rendered
            .find("fn cascade_match_cyc")
            .expect("the cyclic internal rule gets a match fn");
        let leaf_fn_start = rendered
            .find("fn cascade_match_leaf")
            .expect("acyclic internal rule gets a match fn");
        let cyc_body = &rendered[cyc_fn_start
            ..rendered[cyc_fn_start + 1..]
                .find("fn cascade_")
                .map(|off| cyc_fn_start + 1 + off)
                .unwrap_or(rendered.len())];
        assert!(
            cyc_body.contains("check_cycle")
                && cyc_body.contains("recursion_guard . enter_id_bare")
                && cyc_body.contains("recursion_guard . exit_bare"),
            "the cyclic match fn carries the id-only bare guard frame (`-0200`), got: {cyc_body}"
        );
        assert!(
            cyc_body.contains("thin_scratch")
                && cyc_body.contains("thin_entries")
                && cyc_body.contains("THIN_ROW_CYC")
                && cyc_body.contains("write_epoch")
                && cyc_body.contains("deferred_obligation_count")
                && cyc_body.contains("ThinTapeMemoEntry"),
            "the cyclic match fn carries the epoch-stamped direct-index SEGMENT thin memo (`-0205`), got: {cyc_body}"
        );
        assert!(
            cyc_body.contains("extend_from_slice"),
            "a thin-memo hit splices the cached segment onto the live tape, got: {cyc_body}"
        );
        assert!(
            cyc_body.contains("__pgen_thin_mark") && cyc_body.contains("from_slice"),
            "a thin-memo insert captures the body's tape segment (RGX-0078.5.i.14/C3: inline-small SmallVec), got: {cyc_body}"
        );
        assert!(
            cyc_body.contains("cascade_match_cyc ()"),
            "the cyclic self-reference is a direct recursive match call, got: {cyc_body}"
        );
        // The cyclic rule's BUILD fn exists and walks the tape without guards.
        let cyc_build_start = rendered
            .find("fn cascade_build_cyc")
            .expect("the cyclic internal rule gets a build fn");
        let cyc_build_body = &rendered[cyc_build_start
            ..rendered[cyc_build_start + 1..]
                .find("fn cascade_")
                .map(|off| cyc_build_start + 1 + off)
                .unwrap_or(rendered.len())];
        assert!(
            !cyc_build_body.contains("recursion_guard") && !cyc_build_body.contains("thin_scratch"),
            "the build fn walks the committed tape with no guard/memo, got: {cyc_build_body}"
        );
        // The acyclic match fn keeps the frame-free shape.
        let leaf_body = &rendered[leaf_fn_start
            ..rendered[leaf_fn_start + 1..]
                .find("fn cascade_")
                .map(|off| leaf_fn_start + 1 + off)
                .unwrap_or(rendered.len())];
        assert!(
            !leaf_body.contains("recursion_guard") && !leaf_body.contains("thin_scratch"),
            "an acyclic fused rule carries neither guard nor memo, got: {leaf_body}"
        );
    }

    /// RGX-0078.5.i.7 (D2-B) — a CYCLE-PARTICIPATING SUB-ROOT keeps the twin
    /// dispatch in its protocol method but gets NO guard frame and NO thin memo
    /// in its cascade fn: every bare-path entry to it goes through the method
    /// (fused bodies call sub-roots as methods), whose full frame already
    /// provides `check_cycle` + the real memo — a duplicated guard frame would
    /// see the method's own in-flight frame and falsely report `Infinite`.
    #[test]
    fn cascade_cyclic_sub_root_keeps_protocol_protection_not_thin_memo() {
        let mut tree: HashMap<String, ASTNode> = HashMap::new();
        // The entry itself is the cycle: entry → "(" entry? ")".
        tree.insert(
            "entry".to_string(),
            ASTNode::Sequence {
                elements: vec![
                    atom_lit("("),
                    ASTNode::Quantified {
                        element: Box::new(atom_ref("entry")),
                        quantifier: "?".to_string(),
                    },
                    atom_lit(")"),
                ],
            },
        );

        let generator = generator_for(Some(Annotations::default()));
        *generator.first_set_grammar_tree.borrow_mut() = tree.clone();
        generator.build_cascade_emission_plan_for_codegen(&tree, "entry");

        assert!(generator.cascade_plan_active(), "plan must be active");
        assert!(
            generator.cascade_sub_root("entry"),
            "the cyclic entry is a sub-root"
        );
        assert!(
            !generator.cascade_thin_memo_active(),
            "a cyclic SUB-ROOT activates no thin memo (its method's real memo protects it)"
        );
        // RGX-0078.5.i.7 (MTB-B) — the match-then-build split covers the
        // cyclic spine too: even a fully-cyclic plan is MTB-active, and the
        // cyclic sub-root becomes an orchestrator with its match/build pair.
        assert!(
            generator.cascade_mtb_active(),
            "a fully-cyclic plan is MTB-active under the B increment"
        );

        let parser_name = quote::format_ident!("CascadeTestParser");
        let rendered = generator
            .generate_cascade_impl(&parser_name, "cascade_test.rs")
            .expect("cascade impl generation should succeed")
            .to_string();
        assert!(
            rendered.contains("fn cascade_entry")
                && rendered.contains("fn cascade_match_entry")
                && rendered.contains("fn cascade_build_entry"),
            "the cyclic sub-root gets the orchestrator + its match/build pair, got: {rendered}"
        );
        assert!(
            !rendered.contains("recursion_guard") && !rendered.contains("thin_scratch"),
            "the cyclic sub-root's fused fns carry neither guard nor thin memo (its protocol frame protects every bare-path entry), got: {rendered}"
        );
        assert!(
            rendered.contains("parse_entry ()"),
            "the cyclic self-reference routes through the protocol METHOD (guard + real memo), got: {rendered}"
        );
        // The twin dispatch stays at the sub-root's protocol method.
        let entry_body = generator
            .generate_rule_body_inner("entry", tree.get("entry").unwrap(), "cascade_test.rs")
            .expect("entry body generation should succeed")
            .to_string();
        assert!(
            entry_body.contains("bare_parse") && entry_body.contains("cascade_entry"),
            "the cyclic sub-root keeps the twin dispatch, got: {entry_body}"
        );
    }

    /// RGX-0078.5.i.7 (D2-B) — twin-dispatch RELOCATION: an acyclic rule whose
    /// only caller is a cyclic rule was a SUB-ROOT under increment A (its
    /// caller was a protocol boundary there) and demotes to a plain fused
    /// INTERNAL under the CyclicSpine increment — its references become direct
    /// cascade calls and its protocol method loses the twin dispatch.
    #[test]
    fn cascade_former_a_sub_root_demotes_to_internal_under_cyclic_spine() {
        let mut tree: HashMap<String, ASTNode> = HashMap::new();
        tree.insert(
            "entry".to_string(),
            ASTNode::Sequence {
                elements: vec![atom_ref("cyc")],
            },
        );
        tree.insert(
            "cyc".to_string(),
            ASTNode::Sequence {
                elements: vec![
                    atom_lit("("),
                    atom_ref("leaf"),
                    ASTNode::Quantified {
                        element: Box::new(atom_ref("cyc")),
                        quantifier: "?".to_string(),
                    },
                ],
            },
        );
        tree.insert(
            "leaf".to_string(),
            ASTNode::Sequence {
                elements: vec![atom_lit("x")],
            },
        );

        let generator = generator_for(Some(Annotations::default()));
        *generator.first_set_grammar_tree.borrow_mut() = tree.clone();
        generator.build_cascade_emission_plan_for_codegen(&tree, "entry");

        // Under increment A `leaf` was a sub-root (referenced by the cyclic —
        // then non-fused — `cyc`); under B its every caller is fused, so it is
        // plain internal (the -0088 "188 promoted sub-root entries" class).
        assert!(
            generator.cascade_internal("leaf") && !generator.cascade_sub_root("leaf"),
            "the former A sub-root demotes to internal under the CyclicSpine increment"
        );
        assert!(
            generator.cascade_internal("cyc"),
            "the cyclic caller fuses as internal"
        );

        let parser_name = quote::format_ident!("CascadeTestParser");
        let rendered = generator
            .generate_cascade_impl(&parser_name, "cascade_test.rs")
            .expect("cascade impl generation should succeed")
            .to_string();
        assert!(
            rendered.contains("cascade_match_leaf ()") && !rendered.contains("parse_leaf ()"),
            "fused references to the demoted rule are direct match calls, got: {rendered}"
        );
        // The demoted rule's protocol method carries no twin dispatch anymore.
        let leaf_body = generator
            .generate_rule_body_inner("leaf", tree.get("leaf").unwrap(), "cascade_test.rs")
            .expect("leaf body generation should succeed")
            .to_string();
        assert!(
            !leaf_body.contains("bare_parse"),
            "a demoted internal rule's method loses the twin dispatch, got: {leaf_body}"
        );
        // RGX-0078.5.i.7 (MTB-B) — the MTB population is the fused partition
        // itself: the demoted `leaf` is MTB-INTERNAL (match/build pair, no
        // eager fn, no orchestrator), and the cyclic `cyc` is too — its match
        // fn carries the guard + segment thin memo.
        assert!(
            generator.mtb_internal("leaf")
                && !generator.mtb_sub_root("leaf")
                && generator.mtb_internal("cyc")
                && generator.mtb_sub_root("entry"),
            "the demoted rule and the cyclic rule are MTB-internal; the entry is the sub-root"
        );
        assert!(
            !rendered.contains("fn cascade_leaf ")
                && rendered.contains("fn cascade_match_leaf")
                && rendered.contains("fn cascade_build_leaf"),
            "the demoted internal rule gets a match/build pair and no eager fn, got: {rendered}"
        );
    }

    /// RGX-0078.5.i.7 (MTB-A) — the match pass strips ALL value construction
    /// (no `ParseContent`, no arena writes, no transforms) while keeping the
    /// tournament control flow, and records the committed derivation
    /// (`OrWinner` placeholder-push-then-patch, in-tape compaction,
    /// `QuantCount`, `OptPresent`); the build pass constructs values ONCE over
    /// the tape (no `match_string`, no speculation) with the replayed cursor.
    #[test]
    fn mtb_match_strips_values_and_build_constructs_over_the_tape() {
        let mut tree: HashMap<String, ASTNode> = HashMap::new();
        // entry := ("a" "b" | "c") inner? inner*
        tree.insert(
            "entry".to_string(),
            ASTNode::Sequence {
                elements: vec![
                    ASTNode::Or {
                        alternatives: vec![
                            ASTNode::Sequence {
                                elements: vec![atom_lit("a"), atom_lit("b")],
                            },
                            atom_lit("c"),
                        ],
                    },
                    ASTNode::Quantified {
                        element: Box::new(atom_ref("inner")),
                        quantifier: "?".to_string(),
                    },
                    ASTNode::Quantified {
                        element: Box::new(atom_ref("inner")),
                        quantifier: "*".to_string(),
                    },
                ],
            },
        );
        tree.insert(
            "inner".to_string(),
            ASTNode::Sequence {
                elements: vec![atom_lit("x")],
            },
        );

        let generator = generator_for(Some(Annotations::default()));
        *generator.first_set_grammar_tree.borrow_mut() = tree.clone();
        generator.build_cascade_emission_plan_for_codegen(&tree, "entry");
        assert!(generator.cascade_mtb_active(), "the acyclic plan is MTB-active");

        let parser_name = quote::format_ident!("CascadeTestParser");
        let rendered = generator
            .generate_cascade_impl(&parser_name, "cascade_test.rs")
            .expect("cascade impl generation should succeed")
            .to_string();

        let match_start = rendered
            .find("fn cascade_match_entry")
            .expect("entry gets a match fn");
        let match_body = &rendered[match_start
            ..rendered[match_start + 1..]
                .find("fn cascade_")
                .map(|off| match_start + 1 + off)
                .unwrap_or(rendered.len())];
        assert!(
            !match_body.contains("ParseContent")
                && !match_body.contains("sequence_elements")
                && !match_body.contains("arena . alloc"),
            "the match fn builds no values (no boundary call-outs in this fixture), got: {match_body}"
        );
        assert!(
            match_body.contains("OrWinner (0)")
                && match_body.contains("copy_within")
                && match_body.contains("QuantCount")
                && match_body.contains("OptPresent"),
            "the match fn records the committed derivation with in-tape compaction, got: {match_body}"
        );
        assert!(
            match_body.contains("match_string") || match_body.contains("match_lit_ascii"),
            "the match fn keeps the terminal matching control flow, got: {match_body}"
        );
        // The fixture generator carries no `@whitespace_sensitive` directive,
        // so terminals are layout-SKIPPING (the default posture) and each
        // literal records its dynamic end. (The regex artifact's ZERO-event
        // claim is verified against the regenerated artifact itself — its
        // grammar declares layout sensitivity, which elides these statically.)
        assert!(
            match_body.contains("TokEnd"),
            "a skipping literal records its dynamic end, got: {match_body}"
        );

        let build_start = rendered
            .find("fn cascade_build_entry")
            .expect("entry gets a build fn");
        let build_body = &rendered[build_start
            ..rendered[build_start + 1..]
                .find("fn cascade_")
                .map(|off| build_start + 1 + off)
                .unwrap_or(rendered.len())];
        assert!(
            !build_body.contains("match_string")
                && !build_body.contains("match_lit_ascii")
                && !build_body.contains("try_parse")
                && !build_body.contains("furthest_position"),
            "the build fn matches nothing, got: {build_body}"
        );
        assert!(
            build_body.contains("deriv_next_event")
                && build_body.contains("ParseContent :: Sequence")
                && build_body.contains("deriv_pos"),
            "the build fn constructs values over the tape with the replayed cursor, got: {build_body}"
        );
    }

    /// RGX-0078.5.i.7 (MTB-A) — an effect-reaching `Or` inside an acyclic rule
    /// keeps the C3-B island machinery VERBATIM on the match pass (checkpoint,
    /// per-branch delta extraction + rollback, winner-delta replay) while
    /// building NO values: island losers stop building values too (the `-0093`
    /// island refinement — `should_take` consumes only end/priority/index).
    #[test]
    fn mtb_match_island_keeps_delta_machinery_without_values() {
        let mut annotations = Annotations::default();
        annotations
            .semantic_annotations
            .insert("fact_writer".to_string(), vec![transform_annotation()]);

        let mut tree: HashMap<String, ASTNode> = HashMap::new();
        tree.insert(
            "entry".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    ASTNode::Sequence {
                        elements: vec![atom_lit("e"), atom_lit("f")],
                    },
                    ASTNode::Sequence {
                        elements: vec![atom_lit("g"), atom_ref("fact_writer")],
                    },
                ],
            },
        );
        tree.insert(
            "fact_writer".to_string(),
            ASTNode::Sequence {
                elements: vec![atom_lit("z")],
            },
        );

        let generator = generator_for(Some(annotations));
        *generator.first_set_grammar_tree.borrow_mut() = tree.clone();
        generator.build_cascade_emission_plan_for_codegen(&tree, "entry");
        assert!(
            generator.cascade_mtb_active() && generator.mtb_sub_root("entry"),
            "the effect-reaching acyclic entry still fuses (MTB sub-root)"
        );

        let match_or = generator
            .mtb_match_or_logic(
                match tree.get("entry").unwrap() {
                    ASTNode::Or { alternatives } => alternatives,
                    _ => unreachable!(),
                },
                "entry",
                "cascade_test.rs",
                true,
            )
            .expect("match Or generation should succeed")
            .to_string();
        assert!(
            match_or.contains("tournament_semantic_checkpoint")
                && match_or.contains("C3bBranchCleanup")
                && match_or.contains("apply_delta")
                && match_or.contains("try_parse_bare"),
            "the island machinery survives on the match pass (bare wrapper since `-0200`), got: {match_or}"
        );
        assert!(
            !match_or.contains("ParseContent") && !match_or.contains("transformed"),
            "island branches build no values on the match pass, got: {match_or}"
        );
        assert!(
            match_or.contains("parse_fact_writer")
                && match_or.contains("TapeWord :: boundary"),
            "the boundary call-out stays a protocol method whose value joins the unified tape, got: {match_or}"
        );
    }

    /// RGX-0078.5.i.7 (MTB-A) — a layout-INSENSITIVE grammar records the
    /// dynamic terminal facts the build cursor cannot re-derive: `TokEnd` for
    /// literals behind a possible layout skip (start = end − literal length),
    /// and the build pass consumes them.
    #[test]
    fn mtb_layout_skipping_literals_record_tok_end_events() {
        let mut tree: HashMap<String, ASTNode> = HashMap::new();
        tree.insert(
            "entry".to_string(),
            ASTNode::Sequence {
                elements: vec![atom_lit("kw")],
            },
        );

        let generator = generator_for(Some(Annotations::default()));
        // No `@whitespace_sensitive` directive ⇒ terminals are
        // layout-INSENSITIVE (skipping) — the default-grammar posture.
        assert!(
            !generator.layout_sensitivity().terminals,
            "fixture precondition: terminals skip layout"
        );
        *generator.first_set_grammar_tree.borrow_mut() = tree.clone();
        generator.build_cascade_emission_plan_for_codegen(&tree, "entry");
        assert!(generator.cascade_mtb_active(), "the acyclic plan is MTB-active");

        let parser_name = quote::format_ident!("CascadeTestParser");
        let rendered = generator
            .generate_cascade_impl(&parser_name, "cascade_test.rs")
            .expect("cascade impl generation should succeed")
            .to_string();
        let match_start = rendered
            .find("fn cascade_match_entry")
            .expect("entry gets a match fn");
        let match_body = &rendered[match_start
            ..rendered[match_start + 1..]
                .find("fn cascade_")
                .map(|off| match_start + 1 + off)
                .unwrap_or(rendered.len())];
        assert!(
            match_body.contains("TokEnd (parser . position)"),
            "a skipping literal records its dynamic end, got: {match_body}"
        );
        let build_start = rendered
            .find("fn cascade_build_entry")
            .expect("entry gets a build fn");
        let build_body = &rendered[build_start
            ..rendered[build_start + 1..]
                .find("fn cascade_")
                .map(|off| build_start + 1 + off)
                .unwrap_or(rendered.len())];
        assert!(
            build_body.contains("TokEnd (__pgen_e)"),
            "the build pass consumes the recorded end, got: {build_body}"
        );
    }

    /// A parsed return-annotation branch (both the census's `Annotations`
    /// source AND the emitter's generator-field source must carry it, exactly
    /// as real generation populates both).
    fn dv_branch(
        ast: crate::ast_pipeline::unified_return_ast::UnifiedReturnAST,
    ) -> Option<super::BranchAnnotation> {
        Some(super::BranchAnnotation {
            annotation_type: "return".to_string(),
            annotation_content: String::new(),
            parsed_ast: Some(ast),
        })
    }

    /// RGX-0078.5.j.2 STEP-2a — a VALUE-PURE (empty-object) entry over a
    /// transparent chain: the chain gets `cascade_build_value_*` fns (zero
    /// node scaffolding), the barrier entry keeps the node signature with
    /// IN-PLACE `Shaped` content and discard-walks its unreferenced child.
    #[test]
    fn direct_value_emits_value_fns_and_inplace_barrier_content() {
        use crate::ast_pipeline::unified_return_ast::UnifiedReturnAST as U;
        let mut tree: HashMap<String, ASTNode> = HashMap::new();
        tree.insert(
            "entry".to_string(),
            ASTNode::Or {
                alternatives: vec![atom_ref("mid")],
            },
        );
        tree.insert(
            "mid".to_string(),
            ASTNode::Or {
                alternatives: vec![atom_ref("leaf")],
            },
        );
        tree.insert(
            "leaf".to_string(),
            ASTNode::Or {
                alternatives: vec![atom_lit("x")],
            },
        );
        let mut annotations = Annotations::default();
        annotations.branch_return_annotations.insert(
            "entry".to_string(),
            vec![dv_branch(U::Object {
                properties: std::collections::HashMap::new(),
            })],
        );
        let mut generator = generator_for(Some(annotations.clone()));
        generator.branch_return_annotations = annotations.branch_return_annotations.clone();
        *generator.first_set_grammar_tree.borrow_mut() = tree.clone();
        generator.build_cascade_emission_plan_for_codegen(&tree, "entry");
        assert!(generator.cascade_plan_active(), "plan must be active");
        assert!(
            generator.dv_barrier_rule("entry")
                && generator.dv_value_licensed("mid")
                && generator.dv_value_licensed("leaf"),
            "the census partition drives the emission classes"
        );

        let parser_name = quote::format_ident!("CascadeTestParser");
        let rendered = generator
            .generate_cascade_impl(&parser_name, "cascade_test.rs")
            .expect("cascade impl generation should succeed")
            .to_string();

        assert!(
            rendered.contains("fn cascade_build_value_mid")
                && rendered.contains("fn cascade_build_value_leaf"),
            "value-licensed rules get value fns, got: {rendered}"
        );
        assert!(
            !rendered.contains("fn cascade_build_mid (")
                && !rendered.contains("fn cascade_build_leaf ("),
            "value-licensed rules have NO node-form build fn, got: {rendered}"
        );
        assert!(
            rendered.contains("fn cascade_build_entry (")
                && rendered.contains("ParseContent :: Shaped (PgenValue :: Object"),
            "the barrier entry keeps the node signature with in-place Shaped content, got: {rendered}"
        );
        assert!(
            rendered.contains("let _ = parser . cascade_build_value_mid ()"),
            "the barrier's unreferenced child is discard-walked through its value fn, got: {rendered}"
        );
        assert!(
            !rendered.contains("sequence_elements"),
            "no body scaffolding anywhere in this fixture, got: {rendered}"
        );
    }

    /// RGX-0078.5.j.2 STEP-2a — a transparent `-> $2` sub-root builds ONLY
    /// element 2's content in place; the sibling elements are value-licensed
    /// and discard-walked; no `sequence_elements` Vec exists anywhere.
    #[test]
    fn direct_value_transparent_positional_builds_only_its_target() {
        use crate::ast_pipeline::unified_return_ast::UnifiedReturnAST as U;
        let mut tree: HashMap<String, ASTNode> = HashMap::new();
        tree.insert(
            "entry".to_string(),
            ASTNode::Sequence {
                elements: vec![atom_ref("a"), atom_ref("b"), atom_ref("c")],
            },
        );
        for (name, lit) in [("a", "x"), ("b", "y"), ("c", "z")] {
            tree.insert(
                name.to_string(),
                ASTNode::Or {
                    alternatives: vec![atom_lit(lit)],
                },
            );
        }
        let mut annotations = Annotations::default();
        annotations.branch_return_annotations.insert(
            "entry".to_string(),
            vec![dv_branch(U::PositionalRef { index: 2 })],
        );
        let mut generator = generator_for(Some(annotations.clone()));
        generator.branch_return_annotations = annotations.branch_return_annotations.clone();
        *generator.first_set_grammar_tree.borrow_mut() = tree.clone();
        generator.build_cascade_emission_plan_for_codegen(&tree, "entry");
        assert!(
            generator.dv_value_licensed("a")
                && generator.dv_value_licensed("c")
                && !generator.dv_value_licensed("b"),
            "$2 demands exactly element 2 (per-reference propagation)"
        );

        let parser_name = quote::format_ident!("CascadeTestParser");
        let rendered = generator
            .generate_cascade_impl(&parser_name, "cascade_test.rs")
            .expect("cascade impl generation should succeed")
            .to_string();

        assert!(
            rendered.contains("fn cascade_build_value_a")
                && rendered.contains("fn cascade_build_value_c")
                && rendered.contains("fn cascade_build_b ("),
            "siblings are value fns, the target keeps its node fn, got: {rendered}"
        );
        assert!(
            rendered.contains("__pgen_target_content")
                && rendered.contains("let _ = parser . cascade_build_value_a ()")
                && rendered.contains("let _ = parser . cascade_build_value_c ()"),
            "the $2 branch builds only its target element in place, got: {rendered}"
        );
        assert!(
            !rendered.contains("sequence_elements"),
            "no body scaffolding anywhere in this fixture, got: {rendered}"
        );
    }

    /// RGX-0078.5.j.2 STEP-2a — a BARE `?`-Quantified BODY records QuantCount
    /// on the tape (the OptPresent fast path exists only at true sequence
    /// positions), so its value fn must walk the QuantCount protocol. The
    /// `entry_alternative = entry_concatenation?` panic class from the first
    /// equivalence run, pinned.
    #[test]
    fn direct_value_bare_optional_body_walks_quant_count_not_opt_present() {
        use crate::ast_pipeline::unified_return_ast::UnifiedReturnAST as U;
        let mut tree: HashMap<String, ASTNode> = HashMap::new();
        tree.insert(
            "entry".to_string(),
            ASTNode::Or {
                alternatives: vec![atom_ref("mid")],
            },
        );
        tree.insert(
            "mid".to_string(),
            ASTNode::Quantified {
                element: Box::new(atom_ref("leaf")),
                quantifier: "?".to_string(),
            },
        );
        tree.insert(
            "leaf".to_string(),
            ASTNode::Or {
                alternatives: vec![atom_lit("x")],
            },
        );
        let mut annotations = Annotations::default();
        annotations.branch_return_annotations.insert(
            "entry".to_string(),
            vec![dv_branch(U::Object {
                properties: std::collections::HashMap::new(),
            })],
        );
        let mut generator = generator_for(Some(annotations.clone()));
        generator.branch_return_annotations = annotations.branch_return_annotations.clone();
        *generator.first_set_grammar_tree.borrow_mut() = tree.clone();
        generator.build_cascade_emission_plan_for_codegen(&tree, "entry");
        assert!(generator.dv_value_licensed("mid"), "mid is value-licensed");

        let parser_name = quote::format_ident!("CascadeTestParser");
        let rendered = generator
            .generate_cascade_impl(&parser_name, "cascade_test.rs")
            .expect("cascade impl generation should succeed")
            .to_string();
        let mid_start = rendered
            .find("fn cascade_build_value_mid")
            .expect("mid value fn emitted");
        let after_marker = mid_start + "fn cascade_build_value_mid".len();
        let mid_end = rendered[after_marker..]
            .find("fn cascade_build")
            .map(|off| after_marker + off)
            .unwrap_or(rendered.len());
        let mid_body = &rendered[mid_start..mid_end];
        assert!(
            mid_body.contains("QuantCount"),
            "a bare `?` body walks QuantCount, got: {mid_body}"
        );
        assert!(
            !mid_body.contains("OptPresent"),
            "a bare `?` body must NOT expect OptPresent, got: {mid_body}"
        );
    }
}
