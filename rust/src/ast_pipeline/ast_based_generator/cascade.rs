//! RGX-0078.5.i.7 (D2-A + D2-B) — the FUSED CASCADE emitter: the full cascade
//! fold (acyclic sub-regions + the cyclic spine).
//!
//! For every rule in the cascade emission plan (the SHARED
//! `fusibility_census::compute_cascade_emission_plan_for_increment` at
//! `CascadeIncrement::CyclicSpine` — sub-roots + internal rules, every
//! cascade-eligible rule), this module emits one compact `cascade_<rule>`
//! function: the rule's parse logic and return-annotation value construction
//! VERBATIM in semantics, with the per-rule protocol frame elided — no per-rule
//! entry counter, no coverage push, no trace scope/lines, no rule transaction —
//! and plain position-restore speculation wherever the speculation scope
//! provably cannot reach a semantic effect.
//!
//! ACYCLIC fused rules additionally elide the recursion-guard/parse-stack frame
//! (`check_cycle` is load-bearing only on a cycle — the Optim #16 argument) and
//! every memo lane (same-position re-probe multiplicity is bounded by the
//! grammar's static caller constant). CYCLE-PARTICIPATING internal fused rules
//! (the plan's `thin_memo`, D2-B) keep both, in lean form: the protocol-mirror
//! `check_cycle` + `enter`/`exit` guard frame, and the epoch-stamped THIN memo
//! (⛔ the session-#49 bound — a cyclic fused rule never loses memo
//! protection; see `ThinMemoEntry` for the value-only-replay soundness
//! argument). A cycle-participating SUB-ROOT needs neither in its cascade fn:
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

use super::super::{ASTNode, ASTValue, TokenValue, parse_quantifier_bounds};
use super::{AstBasedGenerator, BranchAnnotation};
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashSet;
use syn::Ident;

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
    pub(crate) fused_order: Vec<String>,
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
    /// (gates the `thin_memo` parser-struct field + its constructor init, so a
    /// fully-acyclic grammar's artifact stays byte-identical to the D2-A
    /// emission)?
    pub(super) fn cascade_thin_memo_active(&self) -> bool {
        self.cascade_plan()
            .is_some_and(|plan| !plan.thin_memo_internal.is_empty())
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
        }
        for rule_name in &fused_order {
            let Some(ast_node) = grammar_tree.get(rule_name) else {
                anyhow::bail!(
                    "cascade emission plan names rule '{rule_name}' absent from the gen-AST tree"
                );
            };
            cascade_fns.push(self.generate_cascade_rule_fn(rule_name, ast_node, filename)?);
        }
        Ok(quote! {
            /// RGX-0078.5.i.7 (D2-A + D2-B) — the FUSED cascade graph: compact
            /// per-rule functions for the bare-parse path (no coverage / trace /
            /// counters / memo-stats consumer). Entered exclusively through the
            /// observability-twin dispatch at plan sub-root memoized bodies;
            /// every diagnostic consumer runs the untouched protocol methods.
            /// Cycle-participating internal rules carry the recursion-guard
            /// frame and the epoch-stamped thin memo (the ⛔ #49 bound).
            impl<'input> #parser_name<'input> {
                #(#cascade_fns)*
            }
        })
    }

    /// One fused rule function. Mirrors `generate_rule_body_inner`'s semantics
    /// with the protocol frame elided: furthest max-update at the head (the
    /// method's rule-entry update — the ONLY furthest write site, so parity is
    /// exact), parse logic, the rule-level return transform for non-`Or` roots,
    /// and the `ParseNode` result. Every protocol tail the plan gate statically
    /// excludes (relational guards, span `@transform`, coverage-target /
    /// partition events, raw capture for post/final predicates) is provably
    /// absent for a plan rule and not emitted.
    fn generate_cascade_rule_fn(
        &self,
        rule_name: &str,
        ast_node: &ASTNode,
        filename: &str,
    ) -> Result<TokenStream> {
        let cascade_fn = format_ident!("cascade_{}", rule_name);

        let parse_logic = match ast_node {
            ASTNode::Or { alternatives } => {
                self.cascade_or_logic(alternatives, rule_name, filename, true)?
            }
            _ => self.cascade_node_logic(ast_node, rule_name, filename)?,
        };

        // Rule-level return annotation for non-`Or` roots (the `Or` path applies
        // per-branch transforms inline) — the `generate_rule_body_inner` mirror,
        // with the assignment expressed as a shadowing rebind (value-identical;
        // no `mut` requirement on the body's `result` binding).
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
                    quote! {
                        let result = { #transform };
                    }
                } else {
                    quote! {}
                }
            }
        };

        // The rule's core body — shared verbatim between the plain (acyclic)
        // form and the D2-B thin-memo (cyclic) form, so the acyclic emission is
        // token-identical to the landed D2-A shape.
        let core_body = quote! {
            let start_pos = parser.position;
            #parse_logic;
            #post_parse_transform_tokens
            let end_pos = parser.position;
            Ok(ParseNode {
                rule_name: #rule_name,
                content: result,
                span: start_pos..end_pos,
            })
        };

        if self.cascade_thin_memo_internal(rule_name) {
            // RGX-0078.5.i.7 (D2-B) — a CYCLE-PARTICIPATING internal fused rule:
            // the protocol frame parts that are load-bearing exactly on a cycle
            // are carried over, everything else stays elided.
            //
            // 1. RECURSION GUARD — `check_cycle`'s `Infinite`/`LeftRecursive`
            //    verdicts scan the parse stack for THIS rule's in-flight frames,
            //    so they are exact iff every cyclic rule pushes in both graphs:
            //    each thin-memo fn mirrors the protocol method's
            //    check/enter/exit (reject arms identical minus the trace lines,
            //    which a bare parse can never enable). The whole-stack depth
            //    ceiling keeps the landed D2-A margin semantics (the bare-path
            //    stack omits acyclic fused frames in both increments).
            // 2. THIN MEMO — the ⛔ #49 bound: probe/insert around the body,
            //    with the protocol memo's own per-entry taint classes (PURE /
            //    STORE-READ / STORE-MUTATING — see `ThinMemoEntry`), measured
            //    across the body by the write epoch, the deferred-obligation
            //    count, and the predicate-evaluation counter. Cached failures
            //    replay as `Backtrack` at the probe position, exactly as the
            //    protocol memo replays every cached failure.
            //
            // The body runs inside a closure so every `?`/early-return path
            // still passes `recursion_guard.exit()` and the thin-memo insert
            // (the [[feedback_question_bypasses_manual_cleanup]] IIFE pattern).
            let rule_const = format_ident!("RULE_{}", rule_name.to_uppercase());
            let recursion_guard_max_depth = super::GENERATED_RECURSION_GUARD_MAX_DEPTH;
            return Ok(quote! {
                fn #cascade_fn(&mut self) -> ParseResult<ParseNode<'input>> {
                    let parser = self;
                    // The protocol method's rule-entry furthest update — the only
                    // furthest write site — mirrored one-for-one for exact parity.
                    if parser.position > parser.furthest_position {
                        parser.furthest_position = parser.position;
                    }
                    let position = parser.position;
                    match parser.recursion_guard.check_cycle(#rule_name, position) {
                        CycleType::Infinite => {
                            return Err(ParseError::InvalidSyntax {
                                message: "Infinite recursion detected",
                                position,
                            });
                        }
                        CycleType::LeftRecursive => {
                            return Err(ParseError::InvalidSyntax {
                                message: "Left recursion detected",
                                position,
                            });
                        }
                        CycleType::MutualRecursive { depth, .. } if depth >= #recursion_guard_max_depth => {
                            return Err(ParseError::RecursionDepthExceeded {
                                position,
                                depth,
                            });
                        }
                        _ => {}
                    }
                    let __pgen_thin_key = (Self::#rule_const, position);
                    let __pgen_thin_epoch = parser.semantic_runtime_state.write_epoch();
                    let __pgen_thin_deferred = parser.semantic_runtime_state.deferred_obligation_count();
                    let mut __pgen_thin_stale = false;
                    if let Some(__pgen_thin_entry) = parser.thin_memo.get(&__pgen_thin_key) {
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
                                Some((__pgen_thin_end, __pgen_thin_node)) => {
                                    let __pgen_thin_end = *__pgen_thin_end;
                                    let __pgen_thin_node = __pgen_thin_node.clone();
                                    parser.position = __pgen_thin_end;
                                    return Ok(__pgen_thin_node);
                                }
                                None => {
                                    return Err(ParseError::Backtrack { position });
                                }
                            }
                        }
                        __pgen_thin_stale = true;
                    }
                    if __pgen_thin_stale {
                        parser.thin_memo.remove(&__pgen_thin_key);
                    }
                    let __pgen_thin_preds = parser.semantic_runtime_state.predicate_evaluations();
                    parser.recursion_guard.enter(#rule_name, position);
                    let __pgen_thin_result: ParseResult<ParseNode<'input>> =
                        (|parser: &mut Self| -> ParseResult<ParseNode<'input>> {
                            #core_body
                        })(parser);
                    parser.recursion_guard.exit();
                    // Classify the body per the ThinMemoEntry taint classes: a
                    // store-MUTATING body is never cached (value-only replay
                    // would skip its effects); a store-READ body is cached with
                    // the unchanged-epoch stamp; a PURE body is cached
                    // unconditionally (the protocol's untainted license).
                    let __pgen_thin_mutated =
                        parser.semantic_runtime_state.write_epoch() != __pgen_thin_epoch
                            || parser.semantic_runtime_state.deferred_obligation_count()
                                != __pgen_thin_deferred;
                    if !__pgen_thin_mutated {
                        let __pgen_thin_stamp =
                            if parser.semantic_runtime_state.predicate_evaluations()
                                == __pgen_thin_preds
                            {
                                None
                            } else {
                                Some((__pgen_thin_epoch, __pgen_thin_deferred))
                            };
                        match &__pgen_thin_result {
                            Ok(__pgen_thin_node) => {
                                parser.thin_memo.insert(
                                    __pgen_thin_key,
                                    crate::ast_pipeline::ThinMemoEntry {
                                        stamp: __pgen_thin_stamp,
                                        outcome: Some((
                                            __pgen_thin_node.span.end,
                                            __pgen_thin_node.clone(),
                                        )),
                                    },
                                );
                            }
                            Err(_) => {
                                parser.thin_memo.insert(
                                    __pgen_thin_key,
                                    crate::ast_pipeline::ThinMemoEntry {
                                        stamp: __pgen_thin_stamp,
                                        outcome: None,
                                    },
                                );
                            }
                        }
                    }
                    __pgen_thin_result
                }
            });
        }

        Ok(quote! {
            fn #cascade_fn(&mut self) -> ParseResult<ParseNode<'input>> {
                let parser = self;
                // The protocol method's rule-entry furthest update — the only
                // furthest write site — mirrored one-for-one for exact parity.
                if parser.position > parser.furthest_position {
                    parser.furthest_position = parser.position;
                }
                #core_body
            }
        })
    }

    /// Construct dispatch — the `generate_node_parsing_logic` mirror.
    fn cascade_node_logic(
        &self,
        ast_node: &ASTNode,
        rule_name: &str,
        filename: &str,
    ) -> Result<TokenStream> {
        match ast_node {
            ASTNode::Or { alternatives } => {
                // Nested `Or`: no FIRST prune guard (same license restriction as
                // the protocol emission — `parse_start` may exceed the recorded
                // rule-entry position).
                self.cascade_or_logic(alternatives, rule_name, filename, false)
            }
            ASTNode::Sequence { elements } => {
                self.cascade_sequence_logic(elements, rule_name, filename)
            }
            ASTNode::Atom { value } => self.cascade_atom_logic(value, rule_name),
            ASTNode::Quantified {
                element,
                quantifier,
            } => self.cascade_quantified_logic(element, quantifier, rule_name, filename),
            ASTNode::Lookahead { element, positive } => {
                self.cascade_lookahead_logic(element, *positive, rule_name, filename)
            }
        }
    }

    /// One speculation scope: binds `__pgen_attempt: Option<T>` from a `body`
    /// producing `ParseResult<T>`. Effect-reaching subtrees run under
    /// `try_parse` (⛔ C3-B rule 1 — semantic snapshot/rollback exactly as the
    /// protocol's speculation wrapper); effect-free subtrees use plain position
    /// save/restore (nothing else can have changed: no store write, no coverage
    /// push, and boundary methods balance their own parse-stack frames on every
    /// path).
    fn cascade_speculation_tokens(&self, subtree: &ASTNode, body: TokenStream) -> TokenStream {
        if self.cascade_subtree_reaches_effects(subtree) {
            quote! {
                let __pgen_attempt = parser.try_parse(|p| {
                    let parser = p;
                    #body
                });
            }
        } else {
            quote! {
                let __pgen_attempt = {
                    let __pgen_spec_start = parser.position;
                    match (|parser: &mut Self| -> ParseResult<_> { #body })(parser) {
                        Ok(__pgen_speculated) => Some(__pgen_speculated),
                        Err(_) => {
                            parser.position = __pgen_spec_start;
                            None
                        }
                    }
                };
            }
        }
    }

    /// The `Or` mirror: single-branch pass-through, the P2 degenerate
    /// byte-switch, or the tournament (plain for effect-free sites; the protocol
    /// ISLAND — checkpoint / per-branch delta extraction + rollback /
    /// winner-delta replay — when ≥ 1 branch reaches effects, ⛔ C3-B rule 2).
    /// Branch predicates, branch-start effects, partition rotation, recovery
    /// hints, and `nonassoc` ties are all statically excluded by the shared
    /// cascade gate, so their machinery is not emitted.
    fn cascade_or_logic(
        &self,
        alternatives: &[ASTNode],
        rule_name: &str,
        filename: &str,
        top_level: bool,
    ) -> Result<TokenStream> {
        let branch_count = alternatives.len();

        if branch_count == 1 {
            let branch = &alternatives[0];
            let branch_logic = self.cascade_node_logic(branch, rule_name, filename)?;
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
                    return Ok(quote! {
                        #branch_logic;
                    });
                }
                return Ok(quote! {
                    #branch_logic;
                    let result = { #transform };
                });
            }
            return Ok(quote! {
                #branch_logic;
            });
        }

        let branch_priorities = self.rule_branch_priorities(rule_name, branch_count);
        let associativity = self.rule_associativity(rule_name);
        let associativity_mode = associativity.as_str();
        let branch_policy = self.rule_branch_policy(rule_name);
        let branch_policy_mode = branch_policy.as_str();

        let emit_first_set_guard = top_level && self.layout_sensitivity().terminals;
        let mut first_set_cache: std::collections::HashMap<
            String,
            super::super::first_set::FirstSetSummary,
        > = std::collections::HashMap::new();
        let mut second_byte_cache: std::collections::HashMap<
            String,
            super::super::first_set::SecondByteSummary,
        > = std::collections::HashMap::new();

        // The P2 DEGENERATE byte-switch (pairwise-disjoint FIRST bytes ⇒ at most
        // one candidate per next byte) — the SAME shared gate as the protocol
        // emission, so both graphs dispatch identically.
        if let Some(branch_byte_sets) = self.degenerate_dispatch_byte_sets(
            alternatives,
            rule_name,
            emit_first_set_guard,
            &mut first_set_cache,
        ) {
            let mut dispatch_arms = Vec::new();
            for (idx, alternative) in alternatives.iter().enumerate() {
                let branch_logic = self.cascade_node_logic(alternative, rule_name, filename)?;
                let transform = self.cascade_branch_transform(rule_name, idx, alternative)?;
                let byte_patterns = &branch_byte_sets[idx];
                // The sole candidate runs speculatively (failure arm restores);
                // on success its effects stay in place — no losers ran (the
                // protocol's P2 semantics, mirrored per-branch by the
                // effect-aware speculation helper).
                let speculation = self.cascade_speculation_tokens(
                    alternative,
                    quote! {
                        #branch_logic;
                        Ok(result)
                    },
                );
                dispatch_arms.push(quote! {
                    #(#byte_patterns)|* => {
                        #speculation
                        if let Some(content) = __pgen_attempt {
                            let transformed = {
                                let content = content;
                                #transform
                            };
                            result = transformed;
                        } else {
                            return Err(ParseError::Backtrack {
                                position: parse_start,
                            });
                        }
                    }
                });
            }
            return Ok(quote! {
                let parse_start = parser.position;
                let mut result = ParseContent::Sequence(Vec::new());
                if parse_start < parser.input.len() {
                    match parser.input.as_bytes()[parse_start] {
                        #(#dispatch_arms,)*
                        _ => {
                            return Err(ParseError::Backtrack {
                                position: parse_start,
                            });
                        }
                    }
                } else {
                    return Err(ParseError::Backtrack {
                        position: parse_start,
                    });
                }
            });
        }

        // ⛔ C3-B rule 2 — the tournament island verdict for this site.
        let island = alternatives
            .iter()
            .any(|alt| self.cascade_subtree_reaches_effects(alt));

        let mut branch_attempt_blocks: Vec<TokenStream> = Vec::new();
        for (idx, alternative) in alternatives.iter().enumerate() {
            let branch_logic = self.cascade_node_logic(alternative, rule_name, filename)?;
            let transform = self.cascade_branch_transform(rule_name, idx, alternative)?;
            let branch_num = idx + 1;
            let branch_priority = branch_priorities.get(idx).copied().unwrap_or(0);
            let branch_index = idx;
            let first_set_prune_guard = self.first_set_prune_guard_for_branch(
                alternative,
                emit_first_set_guard,
                &mut first_set_cache,
                &mut second_byte_cache,
            );

            // The winner-selection cascade — the protocol's exact chain (the
            // policy/associativity string compares are compile-time constants
            // that fold at opt time; `nonassoc` is statically excluded by the
            // shared gate, so its tie arm is provably dead and not emitted).
            let should_take_chain = quote! {
                let should_take = if #branch_policy_mode == "ordered" {
                    best_content.is_none()
                } else if #branch_policy_mode == "priority_first" {
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
                        match #associativity_mode {
                            "right" => current_branch_index > best_branch_index,
                            _ => false,
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
                    match #associativity_mode {
                        "right" => current_branch_index > best_branch_index,
                        _ => false,
                    }
                };
            };

            let arm_inner = if island {
                // The protocol tournament island: per-branch `try_parse`
                // (semantic rollback on failure), then delta extraction +
                // rollback to the tournament checkpoint; the winner's delta is
                // replayed once the tournament concludes.
                quote! {
                    if #branch_policy_mode == "ordered" && best_content.is_some() {
                        // Ordered branch policy keeps the first successful branch.
                    } else {
                        parser.position = parse_start;
                        if let Some(content) = parser.try_parse(|p| {
                            let parser = p;
                            #branch_logic;
                            Ok(result)
                        }) {
                            let candidate_end = parser.position;
                            let candidate_priority: i64 = #branch_priority;
                            let current_branch_index: usize = #branch_index;
                            let transformed = {
                                let content = content;
                                #transform
                            };
                            parser.position = parse_start;
                            #should_take_chain
                            let candidate_delta = parser
                                .semantic_runtime_state
                                .extract_delta_since(&tournament_semantic_checkpoint);
                            parser.semantic_runtime_state.rollback_to_labeled(
                                tournament_semantic_checkpoint.clone(),
                                crate::ast_pipeline::RollbackLabel::C3bBranchCleanup {
                                    rule: #rule_name,
                                    branch: #branch_num,
                                    total: #branch_count,
                                },
                            );
                            if should_take {
                                best_end = candidate_end;
                                best_priority = candidate_priority;
                                best_branch_index = current_branch_index;
                                best_content = Some(transformed);
                                best_semantic_delta = Some(candidate_delta);
                            }
                        }
                    }
                }
            } else {
                // Effect-free site: plain position-restore tournament — the
                // semantic checkpoint/delta machinery is provably a no-op here
                // (no branch subtree can reach a store write).
                let speculation = self.cascade_speculation_tokens(
                    alternative,
                    quote! {
                        #branch_logic;
                        Ok(result)
                    },
                );
                quote! {
                    if #branch_policy_mode == "ordered" && best_content.is_some() {
                        // Ordered branch policy keeps the first successful branch.
                    } else {
                        parser.position = parse_start;
                        #speculation
                        if let Some(content) = __pgen_attempt {
                            let candidate_end = parser.position;
                            let candidate_priority: i64 = #branch_priority;
                            let current_branch_index: usize = #branch_index;
                            let transformed = {
                                let content = content;
                                #transform
                            };
                            parser.position = parse_start;
                            #should_take_chain
                            if should_take {
                                best_end = candidate_end;
                                best_priority = candidate_priority;
                                best_branch_index = current_branch_index;
                                best_content = Some(transformed);
                            }
                        }
                    }
                }
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
            }
        } else {
            quote! {}
        };
        let island_winner_replay = if island {
            quote! {
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
            let mut best_content: Option<ParseContent<'input>> = None;
            let mut best_end = parse_start;
            let mut best_priority: i64 = i64::MIN;
            let mut best_branch_index: usize = 0usize;
            let mut result = ParseContent::Sequence(Vec::new());
            #island_prologue
            // Branches evaluate in declaration order (partition rotation is
            // statically excluded by the shared cascade gate).
            #(#branch_attempt_blocks)*
            if let Some(content) = best_content {
                parser.position = best_end;
                #island_winner_replay
                result = content;
            } else {
                return Err(ParseError::Backtrack {
                    position: parse_start,
                });
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

    /// The `Sequence` mirror.
    fn cascade_sequence_logic(
        &self,
        elements: &[ASTNode],
        rule_name: &str,
        filename: &str,
    ) -> Result<TokenStream> {
        let element_count = elements.len();
        let mut element_parsers = Vec::new();
        for (idx, element) in elements.iter().enumerate() {
            element_parsers.push(self.cascade_sequence_element(element, idx, rule_name, filename)?);
        }
        Ok(quote! {
            let mut sequence_elements: Vec<&'input ParseNode<'input>> = Vec::with_capacity(#element_count);
            #(#element_parsers)*
            let result = ParseContent::Sequence(sequence_elements)
        })
    }

    /// One sequence element — the `generate_sequence_element` mirror, including
    /// the optional-element fast path with the Q-guard attempt elision (the same
    /// shared license helpers as the protocol emission).
    fn cascade_sequence_element(
        &self,
        element: &ASTNode,
        index: usize,
        rule_name: &str,
        filename: &str,
    ) -> Result<TokenStream> {
        let element_logic = match element {
            ASTNode::Quantified {
                element: inner,
                quantifier,
            } if quantifier == "?" => {
                let inner_logic = self.cascade_node_logic(inner, rule_name, filename)?;
                let speculation = self.cascade_speculation_tokens(
                    inner,
                    quote! {
                        #inner_logic;
                        Ok(result)
                    },
                );
                let attempt = quote! {
                    {
                        #speculation
                        if let Some(content) = __pgen_attempt {
                            content
                        } else {
                            ParseContent::Sequence(Vec::new())
                        }
                    }
                };
                match self.quantified_prune_guard_for_element(inner) {
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
                                ParseContent::Sequence(Vec::new())
                            }
                        }
                    }
                    None => attempt,
                }
            }
            _ => {
                let inner_logic = self.cascade_node_logic(element, rule_name, filename)?;
                quote! {
                    {
                        #inner_logic;
                        result
                    }
                }
            }
        };

        let element_name = format!("element_{}", index);
        Ok(quote! {
            {
                let element_start = parser.position;
                let element_content = #element_logic;
                let element_end = parser.position;
                sequence_elements.push(parser.arena.alloc(ParseNode {
                    rule_name: #element_name,
                    content: element_content,
                    span: element_start..element_end,
                }));
            }
        })
    }

    /// The `Atom` mirror: terminals through the SAME `match_string` /
    /// `match_regex` helpers (layout policy inherited by construction);
    /// plan-internal rule references (including the D2-B cyclic spine) become
    /// direct `cascade_<rule>` calls (furthest updated at the callee's head —
    /// the method-entry mirror); every other reference (sub-roots, ineligible
    /// rules, engine builtins) stays a protocol method call-out — a sub-root
    /// call-out is what gives a cyclic sub-root its guard + real-memo
    /// protection on the bare path.
    fn cascade_atom_logic(&self, value: &ASTValue, rule_name: &str) -> Result<TokenStream> {
        match value {
            ASTValue::Token(parts) if parts.len() >= 2 => {
                let TokenValue::String(token_type) = &parts[0];
                let TokenValue::String(token_value) = &parts[1];
                match token_type.as_str() {
                    "quoted_string" | "number" | "probability" | "include_dir" | "include_file"
                    | "rule" => Ok(quote! {
                        let matched_str = parser.match_string(#token_value)?;
                        let result = ParseContent::Terminal(matched_str)
                    }),
                    "rule_reference" => {
                        if self.cascade_internal(token_value) {
                            let cascade_target = format_ident!("cascade_{}", token_value);
                            Ok(quote! {
                                let __pgen_alt_child = parser.#cascade_target()?;
                                let result = ParseContent::Alternative(parser.arena.alloc(__pgen_alt_child))
                            })
                        } else {
                            let method = format_ident!("parse_{}", token_value);
                            Ok(quote! {
                                let __pgen_alt_child = parser.#method()?;
                                let result = ParseContent::Alternative(parser.arena.alloc(__pgen_alt_child))
                            })
                        }
                    }
                    "regex" => {
                        // A rule-level matched-text `@transform` is ineligible at
                        // the shared cascade gate; if one is ever seen here the
                        // gate and the emission disagree — a loud hard error,
                        // never a silent semantics drop (the P1a re-entry-guard
                        // precedent).
                        if self.cascade_rule_has_matched_text_transform(rule_name) {
                            anyhow::bail!(
                                "cascade emission reached regex atom of rule '{rule_name}' which carries a matched-text @transform — the shared cascade gate must have excluded it"
                            );
                        }
                        let skip_leading_whitespace = !matches!(
                            rule_name,
                            "string_content_double" | "string_content_single"
                        );
                        let effective_regex_pattern =
                            self.effective_regex_pattern(rule_name, token_value);
                        Ok(quote! {
                            let matched_str = parser.match_regex(#effective_regex_pattern, #skip_leading_whitespace)?;
                            let result = ParseContent::Terminal(matched_str)
                        })
                    }
                    _ => Ok(quote! {
                        let result = ParseContent::Terminal("")
                    }),
                }
            }
            _ => Ok(quote! {
                let result = ParseContent::Terminal("")
            }),
        }
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

    /// The `Quantified` mirror: the unified (min, max) loop with the Q-guard
    /// attempt elision, the zero-length-match guard, and the safety limit —
    /// `@stop_at_rule_boundary` is statically excluded by the shared cascade
    /// gate, so its break/error checks are not emitted.
    fn cascade_quantified_logic(
        &self,
        element: &ASTNode,
        quantifier: &str,
        rule_name: &str,
        filename: &str,
    ) -> Result<TokenStream> {
        let element_logic = self.cascade_node_logic(element, rule_name, filename)?;
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
                        return Err(ParseError::Backtrack {
                            position: quantifier_start_position,
                        });
                    }
                },
            )
        } else {
            (quote! {}, quote! {})
        };
        let quantifier_label = quantifier;

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

        let speculation = self.cascade_speculation_tokens(
            element,
            quote! {
                #element_logic;
                Ok(ParseNode {
                    rule_name: "quantified",
                    content: result,
                    span: 0..0,
                })
            },
        );

        Ok(quote! {
            #quantifier_start_position_bind
            let mut results: Vec<&'input ParseNode<'input>> = Vec::new();
            let mut last_position = parser.position;
            let mut iteration_count: usize = 0;
            const SAFETY_LIMIT: usize = 10_000;

            loop {
                if iteration_count >= SAFETY_LIMIT {
                    break;
                }

                #max_check_tokens
                #quant_guard_tokens

                #speculation
                if let Some(node) = __pgen_attempt {
                    let current_position = parser.position;
                    // Zero-length match guard — prevent infinite loops on rules
                    // that can match the empty string.
                    if current_position == last_position {
                        break;
                    }
                    results.push(parser.arena.alloc(node));
                    last_position = current_position;
                    iteration_count += 1;
                } else {
                    break;
                }
            }

            #min_check_tokens

            let result = ParseContent::Quantified(results, #quantifier_label);
        })
    }

    /// The `Lookahead` mirror. On an effect-reaching subtree the attempt runs
    /// under `try_parse` — semantic effects roll back on inner FAILURE and
    /// persist on inner SUCCESS, exactly the protocol's observable behavior
    /// (only the position is restored after a successful probe).
    fn cascade_lookahead_logic(
        &self,
        element: &ASTNode,
        positive: bool,
        rule_name: &str,
        filename: &str,
    ) -> Result<TokenStream> {
        let inner_logic = self.cascade_node_logic(element, rule_name, filename)?;
        let speculation = self.cascade_speculation_tokens(
            element,
            quote! {
                #inner_logic;
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
            #speculation
            parser.position = lookahead_start;
            if #failure_condition {
                return Err(ParseError::Backtrack {
                    position: lookahead_start,
                });
            }
            let result = ParseContent::Sequence(Vec::new())
        })
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
            entry_rule: Some("entry".to_string()),
            logger: None,
            annotations,
            branch_return_annotations: HashMap::new(),
            emit_typed_entry_skeleton: false,
            enable_debug: false,
            parser_hook_registry: None,
            ebnf_grammar_name: None,
            uses_match_regex: std::cell::Cell::new(false),
            first_set_grammar_tree: std::cell::RefCell::new(HashMap::new()),
            analysis_runtime_annotations: std::cell::OnceCell::new(),
            inline_decided_rules: std::cell::OnceCell::new(),
            inline_emission_stack: std::cell::RefCell::new(Vec::new()),
            cascade_emission_plan: std::cell::OnceCell::new(),
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

        assert!(
            rendered.contains("fn cascade_entry"),
            "sub-root gets a cascade fn too, got: {rendered}"
        );
        assert!(
            rendered.contains("fn cascade_wrapper") && rendered.contains("fn cascade_leaf"),
            "internal rules get cascade fns, got: {rendered}"
        );
        assert!(
            rendered.contains("cascade_wrapper ()") && rendered.contains("cascade_leaf ()"),
            "internal references are direct cascade calls, got: {rendered}"
        );
        assert!(
            !rendered.contains("parse_wrapper ()"),
            "an internal reference must not fall back to the protocol method, got: {rendered}"
        );
        assert!(
            !rendered.contains("memoized_call") && !rendered.contains("recursion_guard"),
            "the fused graph carries no protocol frame, got: {rendered}"
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

        let clean = generator
            .cascade_or_logic(
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
            .cascade_or_logic(
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

    /// RGX-0078.5.i.7 (D2-B) — a CYCLE-PARTICIPATING internal rule fuses under
    /// the CyclicSpine increment: its cascade fn recurses through direct
    /// cascade calls and carries the recursion-guard frame + the epoch-stamped
    /// thin memo; an acyclic rule in the same plan keeps the frame-free D2-A
    /// shape.
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

        // The cyclic fn: guard frame + thin memo + recursive direct cascade call.
        let cyc_fn_start = rendered
            .find("fn cascade_cyc")
            .expect("cyclic rule gets a cascade fn");
        let leaf_fn_start = rendered
            .find("fn cascade_leaf")
            .expect("acyclic rule gets a cascade fn");
        let cyc_body = &rendered[cyc_fn_start
            ..rendered[cyc_fn_start + 1..]
                .find("fn cascade_")
                .map(|off| cyc_fn_start + 1 + off)
                .unwrap_or(rendered.len())];
        assert!(
            cyc_body.contains("check_cycle")
                && cyc_body.contains("recursion_guard . enter")
                && cyc_body.contains("recursion_guard . exit"),
            "the cyclic fn carries the protocol-mirror guard frame, got: {cyc_body}"
        );
        assert!(
            cyc_body.contains("thin_memo")
                && cyc_body.contains("write_epoch")
                && cyc_body.contains("deferred_obligation_count")
                && cyc_body.contains("ThinMemoEntry"),
            "the cyclic fn carries the epoch-stamped thin memo, got: {cyc_body}"
        );
        assert!(
            cyc_body.contains("cascade_cyc ()"),
            "the cyclic self-reference is a direct recursive cascade call, got: {cyc_body}"
        );
        // The acyclic fn keeps the frame-free D2-A shape.
        let leaf_body = &rendered[leaf_fn_start
            ..rendered[leaf_fn_start + 1..]
                .find("fn cascade_")
                .map(|off| leaf_fn_start + 1 + off)
                .unwrap_or(rendered.len())];
        assert!(
            !leaf_body.contains("recursion_guard") && !leaf_body.contains("thin_memo"),
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

        let parser_name = quote::format_ident!("CascadeTestParser");
        let rendered = generator
            .generate_cascade_impl(&parser_name, "cascade_test.rs")
            .expect("cascade impl generation should succeed")
            .to_string();
        assert!(
            !rendered.contains("recursion_guard") && !rendered.contains("thin_memo"),
            "the cyclic sub-root's cascade fn carries neither guard nor thin memo, got: {rendered}"
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
            rendered.contains("cascade_leaf ()") && !rendered.contains("parse_leaf ()"),
            "fused references to the demoted rule are direct cascade calls, got: {rendered}"
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
    }
}
