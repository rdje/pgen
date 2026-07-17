//! RGX-0078.5.i.9 (D3) — the BOUNDARY-SCANNER emitter: per plan rule a direct-coded
//! frameless `scan_<rule>(&mut self) -> ParseResult<ParseNode<'input>>` that serves
//! BARE-path call sites in place of the full-frame protocol method (no recursion
//! guard, no entry counter, no coverage push, no trace scope, no rule transaction,
//! no memo lane — the ≈4× per-entry boundary cost the `.5.i.8` STEP-0 cost model
//! priced), while the protocol twin keeps the current full-frame emission VERBATIM
//! (every diagnostic consumer routes there exactly as since D2-A).
//!
//! The plan is the SHARED census gate
//! ([`crate::ast_pipeline::fusibility_census::compute_boundary_scanner_plan`] —
//! `docs/tasks/RGX-0078.md` `.5.i.8` §5.d.1), so the census report and this
//! emission cannot drift. Per [`ScannerValueClass`]:
//!
//! - **Text** — the committed value is exactly the matched text:
//!   `ParseContent::Terminal(&input[start..end])`, zero-copy.
//! - **SpanTransform** — the rule-level matched-text `@transform` span-fallback
//!   emission (`generate_post_body_span_transform`) is reused VERBATIM over the
//!   scanned span.
//! - **ShapedObject** — the raw element `Sequence` is built exactly as the protocol
//!   body builds it (minus frames) and the generator's own return-template emission
//!   (`generate_return_transform`) is reused VERBATIM over it; read-only post-phase
//!   `@predicate`s evaluate through the same content-aware machinery afterwards
//!   (fresh on every entry — ≥ the epoch-validated memo-replay guarantee).
//!
//! `furthest_position` parity is EXACT by construction (the D1/Q-GUARD obligation):
//! the protocol graph writes it at rule-frame entries only (verified: `match_string`
//! / `match_regex` / the char builtins never touch it), so every `scan_<rule>` and
//! every per-rule recognizer helper `scan_rec_<rule>` performs the same max-update
//! at its head — the same positions at the same call structure as the bare-path
//! graph today (fused `cascade_match_*` heads + boundary method entries). The
//! FIRST/FIRST₂/Q-guard prune licenses carry over verbatim because this emission
//! reuses the SAME guard helpers (`first_set_prune_guard_for_branch`,
//! `degenerate_dispatch_byte_sets`, `quantified_prune_guard_for_element`) the
//! protocol and cascade graphs are emitted with.
//!
//! Memo-loss soundness: every plan rule is acyclic token-shaped (census-verified),
//! so a lost memo hit re-runs an O(k) scan bounded by token length — no ⛔-#49
//! obligation. Speculation inside scan bodies is plain position-restore: the plan
//! audit guarantees a scan closure reaches no store effect (read-only predicate
//! evaluation needs no rollback).

use super::super::fusibility_census::{
    compute_boundary_scanner_plan, BoundaryScannerRule, ScannerValueClass,
};
use super::super::{parse_quantifier_bounds, ASTNode, ASTValue, TokenValue};
use super::AstBasedGenerator;
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::BTreeMap;
use syn::Ident;

/// The codegen-side scan plan: the census plan plus deterministic emission orders.
pub(crate) struct ScanCodegenPlan {
    /// Qualified rules (the SHARED census gate's output).
    pub(crate) rules: BTreeMap<String, BoundaryScannerRule>,
    /// Interior (non-plan) tree rules referenced in recognizer position from plan
    /// closures — each becomes one `scan_rec_<rule>` helper (sorted).
    pub(crate) rec_rules: Vec<String>,
}

impl AstBasedGenerator {
    /// Build the scan emission plan ONCE per generation from the SHARED census gate
    /// (the cascade-plan precedent). `None` — the sound "no scan graph" default —
    /// when the cascade plan is inactive (the `bare_parse` twin seam does not
    /// exist), the analysis annotation table failed to compile, or no rule
    /// qualifies (byte-identical pre-D3 emission).
    pub(super) fn build_scan_emission_plan_for_codegen(
        &self,
        grammar_tree: &std::collections::HashMap<String, ASTNode>,
        entry_rule: &str,
    ) {
        let plan: Option<ScanCodegenPlan> = (|| {
            if !self.cascade_plan_active() {
                return None;
            }
            let _compiled = self.analysis_runtime_annotations()?;
            let census_plan = compute_boundary_scanner_plan(
                grammar_tree,
                self.annotations.as_ref(),
                Some(entry_rule),
            )
            .ok()?;
            if census_plan.rules.is_empty() {
                return None;
            }
            let rec_rules =
                Self::collect_scan_rec_rules(grammar_tree, &census_plan.rules);
            crate::pgen_trace_debug!(
                "        D3 boundary-scanner plan: {} scan rule(s), {} recognizer helper(s), {} candidate(s) dropped",
                census_plan.rules.len(),
                rec_rules.len(),
                census_plan.dropped.len(),
            );
            Some(ScanCodegenPlan {
                rules: census_plan.rules,
                rec_rules,
            })
        })();
        // Second `generate_parser_tokens` call on the same generator instance keeps
        // the first plan (OnceCell) — same tree snapshot, identical plan (the
        // P1a/D2-A precedent).
        let _ = self.scan_emission_plan.set(plan);
    }

    fn scan_plan(&self) -> Option<&ScanCodegenPlan> {
        self.scan_emission_plan.get().and_then(|plan| plan.as_ref())
    }

    /// Is `rule` a scan-plan rule (its BARE-path call sites dispatch to
    /// `scan_<rule>`)? `false` whenever the plan is inactive, so plan-inactive
    /// generations are byte-identical to the pre-D3 emission at every seam.
    pub(crate) fn scan_rule(&self, rule: &str) -> bool {
        self.scan_plan().is_some_and(|plan| plan.rules.contains_key(rule))
    }

    pub(crate) fn scan_fn_ident(rule: &str) -> Ident {
        format_ident!("scan_{}", rule)
    }

    fn scan_rec_fn_ident(rule: &str) -> Ident {
        format_ident!("scan_rec_{}", rule)
    }

    /// Collect the interior (non-plan, non-builtin) tree rules referenced in
    /// recognizer position from plan-rule closures — deterministic (sorted), one
    /// `scan_rec_<rule>` helper each. The plan audit already proved every one of
    /// them effect-free, in-vocabulary, and acyclic.
    fn collect_scan_rec_rules(
        tree: &std::collections::HashMap<String, ASTNode>,
        plan_rules: &BTreeMap<String, BoundaryScannerRule>,
    ) -> Vec<String> {
        let mut rec: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        let mut work: Vec<&str> = plan_rules.keys().map(String::as_str).collect();
        while let Some(rule) = work.pop() {
            let Some(body) = tree.get(rule) else { continue };
            let mut refs: std::collections::HashMap<String, usize> =
                std::collections::HashMap::new();
            super::super::fusibility_census::collect_ref_occurrences(body, &mut refs);
            for target in refs.keys() {
                if plan_rules.contains_key(target)
                    || !tree.contains_key(target)
                    || rec.contains(target)
                {
                    continue;
                }
                rec.insert(target.clone());
                work.push(tree.get_key_value(target).map(|(k, _)| k.as_str()).unwrap());
            }
        }
        rec.into_iter().collect()
    }

    /// The scan-graph impl block: one `scan_<rule>` fn per plan rule plus one
    /// `scan_rec_<rule>` recognizer helper per interior rule, in deterministic
    /// order. Empty tokens when the plan is inactive.
    pub(super) fn generate_scan_impl(&self, parser_name: &Ident) -> Result<TokenStream> {
        let Some(plan) = self.scan_plan() else {
            return Ok(TokenStream::new());
        };
        let rules: Vec<(String, BoundaryScannerRule)> = plan
            .rules
            .iter()
            .map(|(rule, entry)| (rule.clone(), entry.clone()))
            .collect();
        let rec_rules = plan.rec_rules.clone();
        // Fn-namespace collision audit (the cascade `match_`/`build_` precedent):
        // `scan_rec_<a>` collides with `scan_<b>` iff `b == "rec_<a>"`.
        for rule in &rec_rules {
            let colliding = format!("rec_{rule}");
            if plan.rules.contains_key(&colliding) {
                anyhow::bail!(
                    "scan-plan rule '{colliding}' collides with the emitted scan_rec_{rule} recognizer of interior rule '{rule}' — rename one of the grammar rules"
                );
            }
        }
        let grammar_tree = self.first_set_grammar_tree.borrow().clone();
        let mut scan_fns: Vec<TokenStream> = Vec::new();
        if rules.iter().any(|(_, entry)| entry.post_predicates) {
            scan_fns.push(Self::scan_post_predicate_gate_helper());
        }
        for rule in &rec_rules {
            let Some(body) = grammar_tree.get(rule) else {
                anyhow::bail!("scan plan references interior rule '{rule}' absent from the gen-AST tree");
            };
            scan_fns.push(self.generate_scan_rec_fn(rule, body)?);
        }
        for (rule, entry) in &rules {
            let Some(body) = grammar_tree.get(rule) else {
                anyhow::bail!("scan plan names rule '{rule}' absent from the gen-AST tree");
            };
            scan_fns.push(self.generate_scan_rule_fn(rule, entry, body)?);
        }
        Ok(quote! {
            /// RGX-0078.5.i.9 (D3) — the BOUNDARY-SCANNER graph: direct-coded
            /// frameless scanners for the bare-parse path (no coverage / trace /
            /// counters / memo-stats consumer). Reached from fused `cascade_*`
            /// bodies and from `bare_parse`-dispatched protocol call sites; every
            /// diagnostic consumer runs the untouched protocol methods. Exact
            /// furthest-position emulation at every rule-frame entry point the
            /// bare-path graph performs today; read-only post-`@predicate` tails
            /// evaluate fresh on every entry.
            impl<'input> #parser_name<'input> {
                #(#scan_fns)*
            }
        })
    }

    /// The read-only post-`@predicate` tail helper, emitted once per artifact when
    /// any plan rule carries post predicates. Mirrors the post-predicate block of
    /// `with_semantic_runtime_rule_transaction` (resolution + content-aware
    /// evaluation) with both content views the shaped content — exactly the
    /// wrapper's `unwrap_or(&node.content)` semantics for a rule with no raw
    /// capture (a plan-gate requirement). Read-only: plan rules carry no effect
    /// directives, so there is no transaction to apply.
    fn scan_post_predicate_gate_helper() -> TokenStream {
        quote! {
            /// RGX-0078.5.i.9 (D3) — read-only post-`@predicate` gate for
            /// `scan_<rule>` tails. `Err(Backtrack { start })` = a predicate
            /// rejected (the protocol's `node.span.start` error, byte-exact);
            /// resolution failures propagate as the protocol body would.
            fn scan_post_predicate_gate(
                &self,
                rule_name: &'static str,
                start_pos: usize,
                content: &ParseContent<'input>,
            ) -> ParseResult<()> {
                for directive in self
                    .semantic_runtime_annotations
                    .post_predicates_for_rule(rule_name)
                {
                    if let crate::ast_pipeline::SemanticRuntimeDirective::Predicate(spec) =
                        directive
                    {
                        if spec.phase == crate::ast_pipeline::SemanticPredicatePhase::Post {
                            let resolved_spec = self
                                .resolve_semantic_predicate_spec_against_content(
                                    spec, content, content,
                                )?;
                            if let Some(false) = self
                                .semantic_runtime_state
                                .evaluate_content_aware_predicate(
                                    &resolved_spec,
                                    content,
                                    content,
                                )
                            {
                                return Err(ParseError::Backtrack {
                                    position: start_pos,
                                });
                            }
                        }
                    }
                }
                Ok(())
            }
        }
    }

    /// One interior recognizer helper: the rule-entry furthest update (the
    /// bare-path graph's per-frame update, mirrored exactly) + the frameless
    /// recognizer body. Failure propagates without restoring position (the
    /// protocol-method contract — every consumer restores through its own
    /// speculation scope).
    fn generate_scan_rec_fn(&self, rule_name: &str, ast_node: &ASTNode) -> Result<TokenStream> {
        let rec_fn = Self::scan_rec_fn_ident(rule_name);
        let body = self.scan_match_node_logic(ast_node, rule_name, true)?;
        Ok(quote! {
            fn #rec_fn(&mut self) -> ParseResult<()> {
                let parser = self;
                if parser.position > parser.furthest_position {
                    parser.furthest_position = parser.position;
                }
                #body
                Ok(())
            }
        })
    }

    /// One plan rule's scanner: entry furthest update, frameless matching, the
    /// class value fold, the optional post-predicate tail, and the `ParseNode`
    /// twin of the protocol method's return (same `rule_name`/`span`/content).
    fn generate_scan_rule_fn(
        &self,
        rule_name: &str,
        entry: &BoundaryScannerRule,
        ast_node: &ASTNode,
    ) -> Result<TokenStream> {
        let scan_fn = Self::scan_fn_ident(rule_name);
        let (matcher, value): (TokenStream, TokenStream) = match entry.class {
            ScannerValueClass::Text => {
                let matcher = self.scan_match_node_logic(ast_node, rule_name, true)?;
                let value = quote! {
                    let result = ParseContent::Terminal(
                        &parser.input[start_pos..parser.position],
                    );
                };
                (matcher, value)
            }
            ScannerValueClass::SpanTransform => {
                let matcher = self.scan_match_node_logic(ast_node, rule_name, true)?;
                let transform = self.generate_post_body_span_transform(rule_name);
                if transform.is_empty() {
                    anyhow::bail!(
                        "scan plan classed rule '{rule_name}' SpanTransform but the span-fallback transform emission is empty — the shared census gate must have excluded it"
                    );
                }
                // The placeholder is never a `TransformedTerminal`, so the reused
                // span-fallback emission always rebinds — the same net result the
                // protocol body reaches through its structural fold.
                let value = quote! {
                    let result = ParseContent::Sequence(Vec::new());
                    #transform
                };
                (matcher, value)
            }
            ScannerValueClass::ShapedObject => {
                let matcher = self.scan_value_sequence_logic(ast_node, rule_name)?;
                let annotation = self
                    .annotations
                    .as_ref()
                    .and_then(|a| a.branch_return_annotations.get(rule_name))
                    .and_then(|branches| branches.first())
                    .and_then(|b| b.as_ref())
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "scan plan classed rule '{rule_name}' ShapedObject but no branch annotation exists — the shared census gate must have excluded it"
                        )
                    })?;
                // `captured_vars = ["result"]` — the protocol body's own call shape
                // (the template pattern-matches `&result`, the raw `Sequence` bound
                // by the value-mode matcher), so the emitted fold is byte-equal.
                let template =
                    self.generate_return_transform(annotation, rule_name, &["result".to_string()])?;
                let value = quote! {
                    let result = { #template };
                };
                (matcher, value)
            }
        };
        let predicate_tail = if entry.post_predicates {
            quote! {
                parser.scan_post_predicate_gate(#rule_name, start_pos, &result)?;
            }
        } else {
            quote! {}
        };
        Ok(quote! {
            fn #scan_fn(&mut self) -> ParseResult<ParseNode<'input>> {
                let parser = self;
                if parser.position > parser.furthest_position {
                    parser.furthest_position = parser.position;
                }
                let start_pos = parser.position;
                #matcher
                #value
                #predicate_tail
                let end_pos = parser.position;
                Ok(ParseNode {
                    rule_name: #rule_name,
                    content: result,
                    span: start_pos..end_pos,
                })
            }
        })
    }

    // ------------------------------------------------------------------
    // The frameless MATCH walk — the `mtb_match_*` control-flow mirror with
    // the derivation tape, boundary vec, and C3-B island machinery removed
    // (the plan audit proves scan closures reach no store effect). Every
    // fragment is a fully-terminated statement sequence; failure paths
    // `return Err(...)` from the enclosing scan fn.
    // ------------------------------------------------------------------

    fn scan_match_node_logic(
        &self,
        ast_node: &ASTNode,
        rule_name: &str,
        rule_top: bool,
    ) -> Result<TokenStream> {
        match ast_node {
            ASTNode::Or { alternatives } => {
                self.scan_match_or_logic(alternatives, rule_name, rule_top)
            }
            ASTNode::Sequence { elements } => {
                let mut parts: Vec<TokenStream> = Vec::with_capacity(elements.len());
                for element in elements {
                    let logic = self.scan_match_node_logic(element, rule_name, false)?;
                    parts.push(quote! { { #logic } });
                }
                Ok(quote! { #(#parts)* })
            }
            ASTNode::Atom { value } => self.scan_match_atom_logic(value, rule_name),
            ASTNode::Quantified {
                element,
                quantifier,
            } => self.scan_match_quantified_logic(element, quantifier, rule_name),
            ASTNode::Lookahead { element, positive } => {
                self.scan_match_lookahead_logic(element, *positive, rule_name)
            }
        }
    }

    /// One scan speculation scope over a `ParseResult<()>` body — plain
    /// position-restore (the plan audit licenses the elision of the semantic
    /// snapshot: no store effect is reachable from a scan closure).
    fn scan_speculation_tokens(body: TokenStream) -> TokenStream {
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

    fn scan_match_atom_logic(&self, value: &ASTValue, rule_name: &str) -> Result<TokenStream> {
        match value {
            ASTValue::Token(parts) if parts.len() >= 2 => {
                let TokenValue::String(token_type) = &parts[0];
                let TokenValue::String(token_value) = &parts[1];
                match token_type.as_str() {
                    "quoted_string" | "number" | "probability" | "include_dir" | "include_file"
                    | "rule" => {
                        let match_call = Self::terminal_literal_match_call(token_value);
                        Ok(quote! {
                            parser.#match_call?;
                        })
                    }
                    "rule_reference" => {
                        if self.scan_rule(token_value) {
                            let scan_target = Self::scan_fn_ident(token_value);
                            Ok(quote! {
                                let _ = parser.#scan_target()?;
                            })
                        } else if !self
                            .first_set_grammar_tree
                            .borrow()
                            .contains_key(token_value.as_str())
                        {
                            match token_value.as_str() {
                                "builtin_any_char" => Ok(quote! {
                                    let _ = parser.parse_builtin_any_char()?;
                                }),
                                "builtin_ascii_char" => Ok(quote! {
                                    let _ = parser.parse_builtin_ascii_char()?;
                                }),
                                other => anyhow::bail!(
                                    "scan emission reached unresolved reference '{other}' in rule '{rule_name}' — the shared census gate must have excluded it"
                                ),
                            }
                        } else {
                            let rec_target = Self::scan_rec_fn_ident(token_value);
                            Ok(quote! {
                                parser.#rec_target()?;
                            })
                        }
                    }
                    "regex" => {
                        let skip_leading_whitespace = !matches!(
                            rule_name,
                            "string_content_double" | "string_content_single"
                        );
                        let effective_regex_pattern =
                            self.effective_regex_pattern(rule_name, token_value);
                        Ok(quote! {
                            parser.match_regex(#effective_regex_pattern, #skip_leading_whitespace)?;
                        })
                    }
                    other => anyhow::bail!(
                        "scan emission reached atom token type '{other}' in rule '{rule_name}' — the shared census gate must have excluded it"
                    ),
                }
            }
            other => anyhow::bail!(
                "scan emission reached atom shape {other:?} in rule '{rule_name}' — the shared census gate must have excluded it"
            ),
        }
    }

    /// The `Or` scan mirror: single-branch pass-through, the P2 byte-switch (the
    /// SAME shared gate as the protocol/cascade emissions), or the tournament over
    /// (end, priority, index) — values are span-derived downstream, so only the
    /// winning END position must be reproduced, but the winner-selection chain is
    /// kept verbatim so priority/associativity policies stay byte-exact.
    fn scan_match_or_logic(
        &self,
        alternatives: &[ASTNode],
        rule_name: &str,
        top_level: bool,
    ) -> Result<TokenStream> {
        let branch_count = alternatives.len();
        if branch_count == 1 {
            return self.scan_match_node_logic(&alternatives[0], rule_name, false);
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

        if let Some(branch_byte_sets) = self.degenerate_dispatch_byte_sets(
            alternatives,
            rule_name,
            emit_first_set_guard,
            &mut first_set_cache,
        ) {
            let mut dispatch_arms = Vec::new();
            for (idx, alternative) in alternatives.iter().enumerate() {
                let branch_logic = self.scan_match_node_logic(alternative, rule_name, false)?;
                let byte_patterns = &branch_byte_sets[idx];
                let speculation = Self::scan_speculation_tokens(quote! {
                    #branch_logic
                    Ok(())
                });
                dispatch_arms.push(quote! {
                    #(#byte_patterns)|* => {
                        #speculation
                        if __pgen_attempt.is_none() {
                            return Err(ParseError::Backtrack {
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

        let mut branch_attempt_blocks: Vec<TokenStream> = Vec::new();
        for (idx, alternative) in alternatives.iter().enumerate() {
            let branch_logic = self.scan_match_node_logic(alternative, rule_name, false)?;
            let branch_priority = branch_priorities.get(idx).copied().unwrap_or(0);
            let branch_index = idx;
            let first_set_prune_guard = self.first_set_prune_guard_for_branch(
                alternative,
                emit_first_set_guard,
                &mut first_set_cache,
                &mut second_byte_cache,
            );

            // The winner-selection cascade — the protocol's exact chain over
            // (end, priority, index).
            let should_take_chain = quote! {
                let should_take = if #branch_policy_mode == "ordered" {
                    !__pgen_best_found
                } else if #branch_policy_mode == "priority_first" {
                    if !__pgen_best_found {
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
                } else if !__pgen_best_found {
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

            let speculation = Self::scan_speculation_tokens(quote! {
                #branch_logic
                Ok(())
            });
            let arm_inner = quote! {
                if #branch_policy_mode == "ordered" && __pgen_best_found {
                    // Ordered branch policy keeps the first successful branch.
                } else {
                    parser.position = parse_start;
                    #speculation
                    if let Some(()) = __pgen_attempt {
                        let candidate_end = parser.position;
                        let candidate_priority: i64 = #branch_priority;
                        let current_branch_index: usize = #branch_index;
                        parser.position = parse_start;
                        #should_take_chain
                        if should_take {
                            best_end = candidate_end;
                            best_priority = candidate_priority;
                            best_branch_index = current_branch_index;
                            __pgen_best_found = true;
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

        Ok(quote! {
            let parse_start = parser.position;
            let mut __pgen_best_found = false;
            let mut best_end = parse_start;
            let mut best_priority: i64 = i64::MIN;
            let mut best_branch_index: usize = 0usize;
            #(#branch_attempt_blocks)*
            if __pgen_best_found {
                parser.position = best_end;
            } else {
                return Err(ParseError::Backtrack {
                    position: parse_start,
                });
            }
        })
    }

    /// The `Quantified` scan mirror: the unified (min, max) loop with the Q-guard
    /// attempt elision (the SAME shared license + EXACT furthest emulation), the
    /// zero-length-match guard, and the safety limit.
    fn scan_match_quantified_logic(
        &self,
        element: &ASTNode,
        quantifier: &str,
        rule_name: &str,
    ) -> Result<TokenStream> {
        let element_logic = self.scan_match_node_logic(element, rule_name, false)?;
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

        let speculation = Self::scan_speculation_tokens(quote! {
            #element_logic
            Ok(())
        });

        Ok(quote! {
            #quantifier_start_position_bind
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
                if let Some(()) = __pgen_attempt {
                    let current_position = parser.position;
                    // Zero-length match guard — prevent infinite loops on rules
                    // that can match the empty string.
                    if current_position == last_position {
                        break;
                    }
                    last_position = current_position;
                    iteration_count += 1;
                } else {
                    break;
                }
            }

            #min_check_tokens
        })
    }

    /// The `Lookahead` scan mirror: probe, restore, verdict.
    fn scan_match_lookahead_logic(
        &self,
        element: &ASTNode,
        positive: bool,
        rule_name: &str,
    ) -> Result<TokenStream> {
        let element_logic = self.scan_match_node_logic(element, rule_name, false)?;
        let speculation = Self::scan_speculation_tokens(quote! {
            #element_logic
            Ok(())
        });
        let verdict = if positive {
            quote! {
                if __pgen_attempt.is_none() {
                    return Err(ParseError::Backtrack {
                        position: lookahead_start,
                    });
                }
            }
        } else {
            quote! {
                if __pgen_attempt.is_some() {
                    return Err(ParseError::Backtrack {
                        position: lookahead_start,
                    });
                }
            }
        };
        Ok(quote! {
            let lookahead_start = parser.position;
            #speculation
            parser.position = lookahead_start;
            #verdict
        })
    }

    /// The ShapedObject VALUE-mode body: the raw element `Sequence` built exactly
    /// as the protocol body builds it (element nodes with `element_<idx>` names and
    /// true spans; references wrapped `Alternative(arena(child))`), minus frames.
    /// The plan audit restricts elements to quoted terminals and plan-rule
    /// references. Binds `sequence_elements`; the caller folds it through the
    /// rule's own return template.
    fn scan_value_sequence_logic(
        &self,
        ast_node: &ASTNode,
        rule_name: &str,
    ) -> Result<TokenStream> {
        let ASTNode::Sequence { elements } = ast_node else {
            anyhow::bail!(
                "scan emission classed rule '{rule_name}' ShapedObject over a non-Sequence body — the shared census gate must have excluded it"
            );
        };
        let element_count = elements.len();
        let mut element_parts: Vec<TokenStream> = Vec::with_capacity(element_count);
        for (idx, element) in elements.iter().enumerate() {
            let element_name = format!("element_{idx}");
            let content_expr: TokenStream = match element {
                ASTNode::Atom {
                    value: ASTValue::Token(parts),
                } if parts.len() >= 2 => {
                    let TokenValue::String(token_type) = &parts[0];
                    let TokenValue::String(token_value) = &parts[1];
                    if matches!(
                        token_type.as_str(),
                        "quoted_string" | "number" | "probability" | "include_dir"
                            | "include_file" | "rule"
                    ) {
                        let match_call = Self::terminal_literal_match_call(token_value);
                        quote! {
                            {
                                let matched_str = parser.#match_call?;
                                ParseContent::Terminal(matched_str)
                            }
                        }
                    } else if token_type == "rule_reference" && self.scan_rule(token_value) {
                        let scan_target = Self::scan_fn_ident(token_value);
                        quote! {
                            {
                                let __pgen_alt_child = parser.#scan_target()?;
                                ParseContent::Alternative(parser.arena.alloc(__pgen_alt_child))
                            }
                        }
                    } else {
                        anyhow::bail!(
                            "scan value-mode element {idx} of rule '{rule_name}' is outside the vocabulary — the shared census gate must have excluded it"
                        );
                    }
                }
                _ => anyhow::bail!(
                    "scan value-mode element {idx} of rule '{rule_name}' is outside the vocabulary — the shared census gate must have excluded it"
                ),
            };
            element_parts.push(quote! {
                {
                    let element_start = parser.position;
                    let element_content = #content_expr;
                    let element_end = parser.position;
                    sequence_elements.push(
                        parser.arena.alloc(ParseNode {
                            rule_name: #element_name,
                            content: element_content,
                            span: element_start..element_end,
                        }),
                    );
                }
            });
        }
        Ok(quote! {
            let mut sequence_elements: Vec<&'input ParseNode<'input>> = Vec::with_capacity(#element_count);
            #(#element_parts)*
            let result = ParseContent::Sequence(sequence_elements);
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::fusibility_census::{
        compute_boundary_scanner_plan, ScannerValueClass,
    };
    use super::super::super::unified_return_ast::UnifiedReturnAST;
    use super::super::super::{
        ASTNode, ASTValue, Annotations, BranchAnnotation, SemanticAnnotation, TokenValue,
        UnifiedSemanticAST, UnifiedSemanticValue,
    };
    use super::super::AstBasedGenerator;
    use std::collections::HashMap;

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

    /// An opaque rule-level `@transform` marker — enough to make a rule
    /// cascade-INELIGIBLE (the census keys on the directive NAME), used on the
    /// entry so the fixture's leaf rules become protocol boundaries.
    fn opaque_transform_annotation() -> SemanticAnnotation {
        SemanticAnnotation::Named {
            name: "transform".to_string(),
            ast: UnifiedSemanticAST::Structured {
                canonical: String::new(),
                value: UnifiedSemanticValue::Boolean(true),
            },
        }
    }

    /// A CANONICAL matched-text `@transform` (the regex.ebnf spelling) — parses
    /// through `parse_canonical_transform_expression`, so the span-fallback
    /// emission fires and the SpanTransform class qualifies.
    fn canonical_transform_annotation() -> SemanticAnnotation {
        SemanticAnnotation::Named {
            name: "transform".to_string(),
            ast: UnifiedSemanticAST::TransformExpr {
                expression: "str::parse::<usize>().unwrap_or(0)".to_string(),
            },
        }
    }

    /// The shared fixture: an INELIGIBLE entry (opaque `@transform`) referencing
    /// - `digits` — a single-char `Or` with `-> $text` on every branch (Text class
    ///   sub-root; the explicit span fold licenses the class under the fixture's
    ///   default layout-SKIPPING policy — a token fold there would be dropped),
    /// - `num` — `digits digits` under a canonical matched-text `@transform`
    ///   (SpanTransform class residual),
    /// - `obj` — `"k" num -> {kind: "n", value: $2}` (ShapedObject class),
    /// - `cyc` — a self-referencing rule (dropped: reference cycle).
    fn fixture() -> (HashMap<String, ASTNode>, Annotations) {
        let mut tree: HashMap<String, ASTNode> = HashMap::new();
        tree.insert(
            "entry".to_string(),
            ASTNode::Sequence {
                elements: vec![
                    atom_ref("digits"),
                    atom_ref("num"),
                    atom_ref("obj"),
                    atom_ref("cyc"),
                ],
            },
        );
        tree.insert(
            "digits".to_string(),
            ASTNode::Or {
                alternatives: vec![atom_lit("1"), atom_lit("2")],
            },
        );
        tree.insert(
            "num".to_string(),
            ASTNode::Sequence {
                elements: vec![atom_ref("digits"), atom_ref("digits")],
            },
        );
        tree.insert(
            "obj".to_string(),
            ASTNode::Sequence {
                elements: vec![atom_lit("k"), atom_ref("num")],
            },
        );
        tree.insert(
            "cyc".to_string(),
            ASTNode::Sequence {
                elements: vec![
                    atom_lit("c"),
                    ASTNode::Quantified {
                        element: Box::new(atom_ref("cyc")),
                        quantifier: "?".to_string(),
                    },
                ],
            },
        );

        let mut annotations = Annotations::default();
        annotations
            .semantic_annotations
            .insert("entry".to_string(), vec![opaque_transform_annotation()]);
        annotations
            .semantic_annotations
            .insert("num".to_string(), vec![canonical_transform_annotation()]);
        // `-> $text` on both digits branches: the explicit span fold (the Text
        // class's layout-independent license).
        annotations.branch_return_annotations.insert(
            "digits".to_string(),
            vec![
                Some(BranchAnnotation {
                    annotation_type: "matched_text".to_string(),
                    annotation_content: "$text".to_string(),
                    parsed_ast: Some(UnifiedReturnAST::MatchedText),
                }),
                Some(BranchAnnotation {
                    annotation_type: "matched_text".to_string(),
                    annotation_content: "$text".to_string(),
                    parsed_ast: Some(UnifiedReturnAST::MatchedText),
                }),
            ],
        );
        let mut properties: HashMap<String, Box<UnifiedReturnAST>> = HashMap::new();
        properties.insert(
            "kind".to_string(),
            Box::new(UnifiedReturnAST::StringLiteral {
                value: "n".to_string(),
            }),
        );
        properties.insert(
            "value".to_string(),
            Box::new(UnifiedReturnAST::PositionalRef { index: 2 }),
        );
        annotations.branch_return_annotations.insert(
            "obj".to_string(),
            vec![Some(BranchAnnotation {
                annotation_type: "object".to_string(),
                annotation_content: "{kind: \"n\", value: $2}".to_string(),
                parsed_ast: Some(UnifiedReturnAST::Object { properties }),
            })],
        );
        (tree, annotations)
    }

    fn generator_for(annotations: Option<Annotations>) -> AstBasedGenerator {
        AstBasedGenerator {
            grammar_name: "scan_test".to_string(),
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
            scan_emission_plan: std::cell::OnceCell::new(),
        }
    }

    fn strip_ws(s: &str) -> String {
        s.chars().filter(|c| !c.is_whitespace()).collect()
    }

    /// The SHARED census gate classifies the fixture's boundary rules into the
    /// three value classes and drops the cyclic candidate with a NAMED reason.
    #[test]
    fn boundary_scanner_plan_classifies_the_three_value_classes_and_names_drops() {
        let (tree, annotations) = fixture();
        let plan = compute_boundary_scanner_plan(&tree, Some(&annotations), Some("entry"))
            .expect("plan computation should succeed");

        let digits = plan.rules.get("digits").expect("digits qualifies");
        assert_eq!(digits.class, ScannerValueClass::Text);
        assert!(digits.sub_root, "digits is a PLAN-B sub-root");
        assert!(!digits.post_predicates);

        let num = plan.rules.get("num").expect("num qualifies");
        assert_eq!(num.class, ScannerValueClass::SpanTransform);
        assert!(!num.sub_root, "num is a residual (matched-text @transform)");

        let obj = plan.rules.get("obj").expect("obj qualifies");
        assert_eq!(obj.class, ScannerValueClass::ShapedObject);

        assert!(
            !plan.rules.contains_key("entry"),
            "the entry rule never qualifies"
        );
        let cyc_reasons = plan.dropped.get("cyc").expect("cyc is dropped, with reasons");
        assert!(
            cyc_reasons.iter().any(|r| r.contains("cycle")),
            "cyc's drop names the reference cycle, got: {cyc_reasons:?}"
        );
    }

    /// The emitter renders one frameless `scan_<rule>` per plan rule with the
    /// class value folds (Terminal span / reused span-transform / reused object
    /// template over value-mode elements), each headed by the exact furthest
    /// update, and the cascade boundary seam dispatches to `scan_num`.
    #[test]
    fn scan_impl_emits_class_value_folds_and_cascade_seam_dispatches() {
        let (tree, annotations) = fixture();
        let generator = generator_for(Some(annotations));
        *generator.first_set_grammar_tree.borrow_mut() = tree.clone();
        generator.build_cascade_emission_plan_for_codegen(&tree, "entry");
        assert!(generator.cascade_plan_active(), "cascade plan must be active");
        generator.build_scan_emission_plan_for_codegen(&tree, "entry");
        assert!(
            generator.scan_rule("digits") && generator.scan_rule("num") && generator.scan_rule("obj"),
            "the three fixture rules are scan-plan rules"
        );

        let parser_name = quote::format_ident!("ScanTestParser");
        let rendered = strip_ws(
            &generator
                .generate_scan_impl(&parser_name)
                .expect("scan impl generation should succeed")
                .to_string(),
        );

        // Text: the zero-copy span Terminal.
        assert!(
            rendered.contains("fnscan_digits")
                && rendered.contains("ParseContent::Terminal(&parser.input[start_pos..parser.position]"),
            "scan_digits folds the matched span, got: {rendered}"
        );
        // SpanTransform: the generator's own span-fallback emission, reused.
        assert!(
            rendered.contains("fnscan_num")
                && rendered.contains("parse::<usize>()")
                && rendered.contains("TransformedTerminal"),
            "scan_num reuses the span-transform fold, got: {rendered}"
        );
        // ShapedObject: value-mode elements + the reused object template.
        assert!(
            rendered.contains("fnscan_obj")
                && rendered.contains("alloc_shaped_pairs")
                && rendered.contains("parser.scan_num()?"),
            "scan_obj builds elements through scan_num and folds the template, got: {rendered}"
        );
        // The exact furthest emulation at every scan head.
        assert!(
            rendered.contains("ifparser.position>parser.furthest_position"),
            "scan fns carry the rule-entry furthest update, got: {rendered}"
        );

        // The cascade boundary seam: a fused body referencing the residual plan
        // rule `num` calls the frameless scan, value on the side vec unchanged.
        let cascade_rendered = strip_ws(
            &generator
                .generate_cascade_impl(&parser_name, "scan_test.rs")
                .expect("cascade impl generation should succeed")
                .to_string(),
        );
        assert!(
            cascade_rendered.contains("parser.scan_num()?")
                && cascade_rendered.contains("deriv_boundary.push"),
            "fused bodies dispatch boundary references to scan fns, got: {cascade_rendered}"
        );
    }

    /// The protocol-side seam: a `rule_reference` atom to a plan rule wraps the
    /// verbatim protocol call in the `bare_parse` twin dispatch; a plan-inactive
    /// generator emits the unchanged protocol call only.
    #[test]
    fn protocol_atom_seam_gates_the_scan_dispatch_on_the_plan() {
        let (tree, annotations) = fixture();
        let generator = generator_for(Some(annotations));
        *generator.first_set_grammar_tree.borrow_mut() = tree.clone();
        generator.build_cascade_emission_plan_for_codegen(&tree, "entry");
        generator.build_scan_emission_plan_for_codegen(&tree, "entry");

        let reference = ASTValue::Token(vec![
            TokenValue::String("rule_reference".to_string()),
            TokenValue::String("digits".to_string()),
        ]);
        let rendered = strip_ws(
            &generator
                .generate_atom_logic(&reference, "entry", "scan_test.rs")
                .expect("atom logic generation should succeed")
                .to_string(),
        );
        assert!(
            rendered.contains("ifparser.bare_parse")
                && rendered.contains("parser.scan_digits()")
                && rendered.contains("parser.parse_digits()"),
            "the plan rule's call site carries the twin dispatch with the protocol arm verbatim, got: {rendered}"
        );

        // Plan-inactive: byte-identical pre-D3 emission (no dispatch).
        let (tree2, annotations2) = fixture();
        let bare_generator = generator_for(Some(annotations2));
        *bare_generator.first_set_grammar_tree.borrow_mut() = tree2;
        let bare_rendered = strip_ws(
            &bare_generator
                .generate_atom_logic(&reference, "entry", "scan_test.rs")
                .expect("atom logic generation should succeed")
                .to_string(),
        );
        assert!(
            !bare_rendered.contains("bare_parse") && bare_rendered.contains("parser.parse_digits()"),
            "a plan-inactive generation emits the unchanged protocol call, got: {bare_rendered}"
        );
    }
}
