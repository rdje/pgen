//! PARSE-SOTA.8 / adoption A1 (PGEN-PARSE-SOTA-0005): static grammar well-formedness.
//!
//! Ford's PEG well-formedness condition (Bryan Ford, "Parsing Expression Grammars", POPL
//! 2004, §3.5–3.6; see docs/book/src/academic-foundations.md): a **left-recursive** rule —
//! one reachable from itself through only nullable left-edge positions, so it can recurse
//! WITHOUT consuming input — loops forever / stack-overflows at runtime. PGEN grammars
//! deliberately avoid left recursion via the iterative `next (OP next)*` idiom, so this is
//! a guard: detect a left-recursive rule from the compiled grammar IR up front and let the
//! generator REJECT it (via the DIAG-SEVERITY always-on `pgen_error!` channel) instead of
//! hanging. PARSER-AGNOSTIC; PURE analysis (no I/O, no generation, mutates nothing).
//!
//! Soundness for a *rejection* check = NO false positives: we are conservative about
//! nullability (terminals are treated as never-nullable) so we never over-report a
//! well-formed grammar as left-recursive.

use super::predicate_expr::{
    PredicateExpr, PredicateValue, PrimitiveCall, parse_predicate_expression,
};
use super::semantic_directive_registry::parse_semantic_string_list;
use super::semantic_runtime::{SemanticRuntimeDirective, parse_semantic_runtime_directive};
use super::{ASTNode, ASTValue, Annotations, SemanticAnnotation, TokenValue, parse_quantifier_bounds};
use std::collections::{HashMap, HashSet};

/// A detected grammar well-formedness issue.
///
/// IMPORTANT SCOPE NOTE (verified on the real SystemVerilog grammar, PARSE-SOTA.8.1):
/// PGEN **handles left recursion** — it runs a compile-time left-recursion-elimination
/// step (the `pre_lr_elim` annotations) AND the runtime `mutual_recursion_handler` detects
/// and breaks left-recursive cycles. So `LeftRecursive` is **INFORMATIONAL only** here (28
/// rules in the shipped SV grammar are left-recursive *by design* and parse fine); it must
/// NOT be used to reject a grammar. The genuine, reject-worthy well-formedness defect is
/// `NonTerminating` — a rule with NO finite terminal derivation, which neither LR
/// elimination nor cycle-breaking can rescue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WellformednessIssue {
    /// INFORMATIONAL: `rule` is directly/indirectly left-recursive (PGEN handles this via
    /// LR elimination + the runtime mutual-recursion handler; NOT an error). `cycle` is one
    /// recursion path for the diagnostic.
    LeftRecursive { rule: String, cycle: Vec<String> },
    /// ERROR: `rule` has NO finite terminal derivation (every path recurses without ever
    /// bottoming out at terminals) — it can never produce/parse a complete string. This is
    /// genuinely ill-formed and IS reject-worthy.
    NonTerminating { rule: String },
    /// WARNING (PARSE-TERMINATION.2): an UNBOUNDED quantifier (`*`/`+`/`{N,}`) at `node_path`
    /// in `rule` has a NULLABLE body — it can iterate without consuming input ("loop without
    /// consuming", Ford PEG well-formedness, POPL 2004 §3.6). PGEN's runtime zero-length guard
    /// prevents an actual hang, but the grammar is ill-formed (the repetition is meaningless)
    /// and it is almost always a grammar bug.
    NullableRepetition { rule: String, node_path: String },
    /// WARNING (ANNOTATION-COMPOSITION.2): `rule` is PRESENT under grammar profile `profile`
    /// (its `@profiles` set is universal/empty OR contains `profile`) but is NOT SATISFIABLE
    /// under it — every production has a required element that references a rule absent under
    /// `profile` (a "profile orphan": the profile filter removed what this rule needs, leaving
    /// a dangling reference). This is the canary class found via `binary_module_path_operator`.
    /// See [[project_semantic_annotation_composition_doctrine]]. Composition algebra: a
    /// sequence needs ALL required parts satisfiable (∩), an alternation needs ANY (∪), and an
    /// optional/star element is absorbing (⊤, never makes its container an orphan).
    /// `suggested_profiles` (ANNOTATION-COMPOSITION.4 — derive-by-default RESOLUTION) = the DERIVED
    /// set of profiles under which `rule` IS satisfiable (the minimal `@profiles` tag that makes the
    /// orphan disappear). Empty = satisfiable under none of the declared profiles (a deeper defect).
    ProfileOrphan {
        rule: String,
        profile: String,
        suggested_profiles: Vec<String>,
    },
    /// ERROR (GRAMMAR-WELLFORMED.A1b): `rule` is DEFINED but UNREACHABLE — not reachable, by
    /// transitive reference, from any ROOT (the canonical entry `rule_order[0]` OR any rule that
    /// nothing references, i.e. a secondary entry such as a `*_multi_entry_root`). A dead rule;
    /// a well-formed grammar has none (Hopcroft–Ullman "no useless symbols" — the reachable half).
    UnreachableRule { rule: String },
    /// ERROR (GRAMMAR-WELLFORMED.F1, data-dependent binding-before-use — Jim/Mandelbaum/Walker,
    /// POPL 2010): `rule`'s `@predicate` consults a fact `kind` (via `primitive`, one of
    /// `has_fact`/`lacks_fact`/`fact_attribute_equals`/`fact_count_at_least`) that NO `@emit_fact`
    /// anywhere in the grammar ever emits. `has_fact` & co. query exactly the store `@emit_fact`
    /// populates, so the consulted fact can never be established — the predicate is degenerate
    /// (`has_fact` always false → the rule is dead; `lacks_fact` always true → the gate is a no-op).
    /// The SOUND, decidable core of binding-before-use: a fact-kind used but with no producing
    /// source. (Per-name / parse-ORDER reachability is undecidable and deliberately not attempted.)
    UnboundFactKind {
        rule: String,
        kind: String,
        primitive: String,
    },
}

impl WellformednessIssue {
    pub fn message(&self) -> String {
        match self {
            WellformednessIssue::LeftRecursive { rule, cycle } => format!(
                "grammar info: rule '{}' is left-recursive (cycle: {}) — handled by PGEN's LR elimination + runtime cycle-breaking (informational, not an error)",
                rule,
                cycle.join(" -> ")
            ),
            WellformednessIssue::NonTerminating { rule } => format!(
                "grammar well-formedness ERROR: rule '{}' has no finite terminal derivation (it can never produce a complete string) — it is ill-formed; add a terminating alternative",
                rule
            ),
            WellformednessIssue::NullableRepetition { rule, node_path } => format!(
                "grammar well-formedness WARNING: rule '{}' has an unbounded repetition at '{}' over a NULLABLE body (can loop without consuming input — Ford PEG well-formedness POPL 2004 §3.6); the runtime is zero-length-guarded but the grammar is ill-formed (likely a bug — the body should consume, or use a bounded quantifier)",
                rule, node_path
            ),
            WellformednessIssue::ProfileOrphan { rule, profile, suggested_profiles } => {
                let fix = if suggested_profiles.is_empty() {
                    " — but it is satisfiable under NO declared profile, so the real fix is a production valid under some profile (or remove the rule; it may be globally dead)".to_string()
                } else {
                    format!(
                        " — DERIVED minimal fix (ANNOTATION-COMPOSITION.4): tag '{}' with @profiles: [{}] (the profiles where it IS satisfiable)",
                        rule,
                        suggested_profiles.join(", ")
                    )
                };
                format!(
                    "grammar well-formedness ERROR: rule '{}' is present under profile '{}' but is NOT satisfiable there — every production references a rule absent under '{}' (a @profiles ORPHAN){}",
                    rule, profile, profile, fix
                )
            }
            WellformednessIssue::UnreachableRule { rule } => format!(
                "grammar well-formedness ERROR: rule '{}' is DEFINED but UNREACHABLE from any entry/root by transitive reference — a dead rule (a well-formed grammar has no useless symbols). Remove it, or reference it from a reachable rule, or make it a top-level entry.",
                rule
            ),
            WellformednessIssue::UnboundFactKind { rule, kind, primitive } => format!(
                "grammar well-formedness ERROR: rule '{}' consults fact-kind '{}' via {}(...), but NO @emit_fact in the grammar emits kind '{}' — the fact can never be established (binding-before-use, Jim et al. POPL 2010). The predicate is degenerate (has_fact always-false / lacks_fact always-true). Fix: emit '{}' somewhere with @emit_fact, or correct the consulted kind (likely a typo).",
                rule, kind, primitive, kind, kind
            ),
        }
    }
}

/// Minimum terminal-atom count of a node given the current per-rule estimates (Purdom
/// phase-1 / `.7.4.2` logic, standalone). `None` while undeterminable. A rule that stays
/// `None` after the fixpoint has no finite derivation = non-terminating.
///
/// `defined` = the set of rules actually defined in THIS grammar. A reference to a rule
/// NOT in `defined` is an EXTERNAL / include reference (PGEN's `include(...)` system
/// resolves it elsewhere) — it is treated as a terminating atom (length 1), NOT a dead
/// end, so a rule that merely references an included rule is never mis-flagged as
/// non-terminating. Without this, standalone analysis of a grammar with includes
/// false-positives (e.g. ebnf.ebnf's `annotation_list := semantic_annotation+`).
fn node_min_terminal_length(
    node: &ASTNode,
    min_len: &HashMap<String, usize>,
    defined: &HashSet<&str>,
) -> Option<usize> {
    match node {
        ASTNode::Or { alternatives } => alternatives
            .iter()
            .filter_map(|a| node_min_terminal_length(a, min_len, defined))
            .min(),
        ASTNode::Sequence { elements } => {
            let mut sum = 0usize;
            for e in elements {
                sum = sum.saturating_add(node_min_terminal_length(e, min_len, defined)?);
            }
            Some(sum)
        }
        ASTNode::Quantified { element, quantifier } => {
            let (min, _) = parse_quantifier_bounds(quantifier).unwrap_or((0, None));
            if min == 0 {
                Some(0)
            } else {
                Some(min.saturating_mul(node_min_terminal_length(element, min_len, defined)?))
            }
        }
        ASTNode::Lookahead { .. } => Some(0),
        ASTNode::Atom { value } => match value {
            ASTValue::Node(inner) => node_min_terminal_length(inner, min_len, defined),
            ASTValue::Token(parts) => match referenced_rule(parts) {
                // A reference to a rule defined HERE: its current estimate (None until
                // resolved). A reference to an UNDEFINED rule (external / include): treat
                // as a terminating atom so it never causes a false non-terminating flag.
                Some(rule) if defined.contains(rule) => min_len.get(rule).copied(),
                Some(_) => Some(1),
                None => Some(1),
            },
        },
    }
}

/// Detect rules with NO finite terminal derivation (non-terminating / ill-formed). PURE
/// analysis: a min-terminal-length fixpoint; any rule absent from the converged table can
/// never bottom out at terminals. NOTE: a left-recursive rule WITH a terminating
/// alternative (e.g. `expr := expr "+" t | t`) has a finite min via the base alternative,
/// so it is correctly NOT reported here — only genuinely-stuck rules are.
pub fn detect_nonterminating_rules(
    grammar: &HashMap<String, ASTNode>,
    rule_order: &[String],
) -> Vec<WellformednessIssue> {
    let defined: HashSet<&str> = grammar.keys().map(|s| s.as_str()).collect();
    let mut min_len: HashMap<String, usize> = HashMap::new();
    loop {
        let mut changed = false;
        for rule in rule_order {
            let Some(body) = grammar.get(rule) else { continue };
            if let Some(candidate) = node_min_terminal_length(body, &min_len, &defined) {
                match min_len.get(rule.as_str()) {
                    Some(&existing) if existing <= candidate => {}
                    _ => {
                        min_len.insert(rule.clone(), candidate);
                        changed = true;
                    }
                }
            }
        }
        if !changed {
            break;
        }
    }
    rule_order
        .iter()
        .filter(|r| grammar.contains_key(r.as_str()) && !min_len.contains_key(r.as_str()))
        .map(|r| WellformednessIssue::NonTerminating { rule: r.clone() })
        .collect()
}

/// Extract the referenced rule name from a token atom that is a `rule_reference`, else None.
fn referenced_rule(parts: &[TokenValue]) -> Option<&str> {
    if parts.len() < 2 {
        return None;
    }
    let TokenValue::String(token_type) = &parts[0] else {
        return None;
    };
    let TokenValue::String(token_value) = &parts[1] else {
        return None;
    };
    if token_type == "rule_reference" {
        Some(token_value.as_str())
    } else {
        None
    }
}

/// Collect every rule referenced ANYWHERE in `node` (all positions, including inside
/// lookaheads) into `out`. Building block for structural reachability (GRAMMAR-WELLFORMED.A1b).
fn collect_node_rule_refs(node: &ASTNode, out: &mut HashSet<String>) {
    match node {
        ASTNode::Or { alternatives } => {
            for a in alternatives {
                collect_node_rule_refs(a, out);
            }
        }
        ASTNode::Sequence { elements } => {
            for e in elements {
                collect_node_rule_refs(e, out);
            }
        }
        ASTNode::Quantified { element, .. } => collect_node_rule_refs(element, out),
        ASTNode::Lookahead { element, .. } => collect_node_rule_refs(element, out),
        ASTNode::Atom { value } => match value {
            ASTValue::Node(inner) => collect_node_rule_refs(inner, out),
            ASTValue::Token(parts) => {
                if let Some(r) = referenced_rule(parts) {
                    out.insert(r.to_string());
                }
            }
        },
    }
}

/// GRAMMAR-WELLFORMED.A1b: rules DEFINED but UNREACHABLE from any root. Roots = the canonical
/// entry (`rule_order[0]`) PLUS every rule that NOTHING references (a secondary entry, e.g. a
/// `*_multi_entry_root` that unions in the alternative start symbols). Reachability = transitive
/// closure of rule references from the roots. Multi-entry-SAFE (an unreferenced top is a root,
/// never a false "unreachable") and conservative (an unreferenced dead orphan is treated as a
/// root → not flagged; only referenced-but-unreachable dead ISLANDS are caught — false negatives
/// are safe, false positives would wrongly reject a good grammar). References to undefined
/// (external/include) rules are ignored. Deterministic (iterates `rule_order`).
pub fn detect_unreachable_rules(
    grammar: &HashMap<String, ASTNode>,
    rule_order: &[String],
) -> Vec<WellformednessIssue> {
    let reachable = reachable_rules(grammar, rule_order);
    rule_order
        .iter()
        .filter(|r| grammar.contains_key(r.as_str()) && !reachable.contains(r.as_str()))
        .map(|r| WellformednessIssue::UnreachableRule { rule: r.clone() })
        .collect()
}

/// The set of rules REACHABLE from the roots (the canonical entry `rule_order[0]` PLUS every rule
/// nothing references — a secondary entry), by transitive reference. Shared core of A1b
/// (`detect_unreachable_rules`) and the certifying CHECKER (`verify_wellformedness_certificate`).
/// Deterministic; references to undefined (external/include) rules are ignored.
pub fn reachable_rules(
    grammar: &HashMap<String, ASTNode>,
    rule_order: &[String],
) -> HashSet<String> {
    let mut refs_of: HashMap<&str, HashSet<String>> = HashMap::new();
    let mut referenced: HashSet<String> = HashSet::new();
    for (rule, body) in grammar {
        let mut refs = HashSet::new();
        collect_node_rule_refs(body, &mut refs);
        refs.retain(|r| grammar.contains_key(r));
        for r in &refs {
            referenced.insert(r.clone());
        }
        refs_of.insert(rule.as_str(), refs);
    }
    let mut reachable: HashSet<String> = HashSet::new();
    let mut stack: Vec<String> = Vec::new();
    if let Some(entry) = rule_order.first() {
        if grammar.contains_key(entry) && reachable.insert(entry.clone()) {
            stack.push(entry.clone());
        }
    }
    for rule in rule_order {
        if grammar.contains_key(rule)
            && !referenced.contains(rule)
            && reachable.insert(rule.clone())
        {
            stack.push(rule.clone());
        }
    }
    while let Some(rule) = stack.pop() {
        if let Some(refs) = refs_of.get(rule.as_str()) {
            for r in refs {
                if reachable.insert(r.clone()) {
                    stack.push(r.clone());
                }
            }
        }
    }
    reachable
}

/// Can `node` match the empty string (succeed without consuming input)? Uses the current
/// per-rule `nullable` estimates (fixpoint). Conservative: a terminal token is treated as
/// never-nullable (consumes input), so we never over-report nullability.
fn node_nullable(node: &ASTNode, nullable: &HashMap<String, bool>) -> bool {
    match node {
        ASTNode::Or { alternatives } => {
            alternatives.iter().any(|a| node_nullable(a, nullable))
        }
        ASTNode::Sequence { elements } => {
            elements.iter().all(|e| node_nullable(e, nullable))
        }
        ASTNode::Quantified { element, quantifier } => {
            let (min, _) = parse_quantifier_bounds(quantifier).unwrap_or((0, None));
            min == 0 || node_nullable(element, nullable)
        }
        // A syntactic predicate consumes no input.
        ASTNode::Lookahead { .. } => true,
        ASTNode::Atom { value } => match value {
            ASTValue::Node(inner) => node_nullable(inner, nullable),
            ASTValue::Token(parts) => match referenced_rule(parts) {
                Some(rule) => nullable.get(rule).copied().unwrap_or(false),
                None => false, // a terminal lexeme consumes input
            },
        },
    }
}

/// Fixpoint over the grammar: which rules are nullable.
fn compute_nullable(grammar: &HashMap<String, ASTNode>, rule_order: &[String]) -> HashMap<String, bool> {
    let mut nullable: HashMap<String, bool> = HashMap::new();
    loop {
        let mut changed = false;
        for rule in rule_order {
            let Some(body) = grammar.get(rule) else { continue };
            let value = node_nullable(body, &nullable);
            if nullable.get(rule).copied().unwrap_or(false) != value {
                nullable.insert(rule.clone(), value);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    nullable
}

/// Does `node` ALWAYS SUCCEED — i.e. match (possibly empty) on EVERY input, never failing?
/// This is the dual of nullability for ordered-choice shadowing: in `a | b`, if `a` always
/// succeeds, PEG commits to `a` and `b` is unreachable (GRAMMAR-WELLFORMED.A2). It DIFFERS from
/// `node_nullable` on syntactic predicates: a lookahead `&e`/`!e` consumes no input (so it is
/// *nullable*) but it can FAIL, so it does NOT always succeed. CONSERVATIVE: anything we cannot
/// PROVE always-succeeds is `false` — so we only ever UNDER-report (miss a shadow), never
/// false-accuse a live branch of being dead.
fn node_always_succeeds(node: &ASTNode, always: &HashMap<String, bool>) -> bool {
    match node {
        // ordered choice succeeds if ANY alternative always succeeds.
        ASTNode::Or { alternatives } => alternatives.iter().any(|a| node_always_succeeds(a, always)),
        // sequence succeeds only if EVERY element always succeeds.
        ASTNode::Sequence { elements } => elements.iter().all(|e| node_always_succeeds(e, always)),
        ASTNode::Quantified { element, quantifier } => {
            let (min, _) = parse_quantifier_bounds(quantifier).unwrap_or((0, None));
            // `?` / `*` / `{0,M}` always succeed (zero matches is a match); `+` / `{N,}` need
            // the body to always succeed.
            min == 0 || node_always_succeeds(element, always)
        }
        // A syntactic predicate is consume-free but CAN FAIL — never "always succeeds".
        ASTNode::Lookahead { .. } => false,
        ASTNode::Atom { value } => match value {
            ASTValue::Node(inner) => node_always_succeeds(inner, always),
            ASTValue::Token(parts) => match referenced_rule(parts) {
                Some(rule) => always.get(rule).copied().unwrap_or(false),
                None => false, // a terminal lexeme can fail (input may differ)
            },
        },
    }
}

/// Fixpoint over the grammar: which rules ALWAYS SUCCEED. Same monotone-upward shape as
/// `compute_nullable`; an undefined/unknown rule stays `false` (conservative).
fn compute_always_succeeds(
    grammar: &HashMap<String, ASTNode>,
    rule_order: &[String],
) -> HashMap<String, bool> {
    let mut always: HashMap<String, bool> = HashMap::new();
    loop {
        let mut changed = false;
        for rule in rule_order {
            let Some(body) = grammar.get(rule) else { continue };
            let value = node_always_succeeds(body, &always);
            if always.get(rule).copied().unwrap_or(false) != value {
                always.insert(rule.clone(), value);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    always
}

/// PARSE-TERMINATION.2 (static no-hang surface): detect UNBOUNDED repetitions over a
/// NULLABLE body — `e*` / `e+` / `e{N,}` where `e` can match the empty string. Such a
/// repetition can iterate without consuming input (Ford PEG well-formedness, POPL 2004 §3.6,
/// "loop without consuming") — the classic PEG infinite-loop hazard. PGEN's runtime is
/// zero-length-guarded so this does not actually hang, but the grammar is ill-formed; this
/// surfaces it as a lint WARNING. Reuses the `compute_nullable` fixpoint. Deterministic,
/// parser-agnostic (keyed only on ASTNode structure + nullability). Returns
/// `WellformednessIssue::NullableRepetition` for each site.
pub fn detect_nullable_repetition(
    grammar: &HashMap<String, ASTNode>,
    rule_order: &[String],
) -> Vec<WellformednessIssue> {
    let nullable = compute_nullable(grammar, rule_order);
    let mut out = Vec::new();
    for rule in rule_order {
        let Some(body) = grammar.get(rule) else { continue };
        collect_nullable_repetition(rule, body, "root", &nullable, &mut out);
    }
    out
}

fn collect_nullable_repetition(
    rule: &str,
    node: &ASTNode,
    path: &str,
    nullable: &HashMap<String, bool>,
    out: &mut Vec<WellformednessIssue>,
) {
    match node {
        ASTNode::Or { alternatives } => {
            for (i, a) in alternatives.iter().enumerate() {
                collect_nullable_repetition(rule, a, &format!("{path}/o{i}"), nullable, out);
            }
        }
        ASTNode::Sequence { elements } => {
            for (i, e) in elements.iter().enumerate() {
                collect_nullable_repetition(rule, e, &format!("{path}/s{i}"), nullable, out);
            }
        }
        ASTNode::Quantified { element, quantifier } => {
            let (_, max) = parse_quantifier_bounds(quantifier).unwrap_or((0, None));
            // Unbounded (max == None: `*`, `+`, `{N,}`) over a nullable body = the hazard.
            if max.is_none() && node_nullable(element, nullable) {
                out.push(WellformednessIssue::NullableRepetition {
                    rule: rule.to_string(),
                    node_path: path.to_string(),
                });
            }
            collect_nullable_repetition(rule, element, &format!("{path}/q"), nullable, out);
        }
        ASTNode::Lookahead { element, .. } => {
            collect_nullable_repetition(rule, element, &format!("{path}/l"), nullable, out);
        }
        ASTNode::Atom { value } => {
            if let ASTValue::Node(inner) = value {
                collect_nullable_repetition(rule, inner, &format!("{path}/a"), nullable, out);
            }
        }
    }
}

// ── ANNOTATION-COMPOSITION.2: @profiles consistency (profile-orphan detection) ───────────
// Doctrine: [[project_semantic_annotation_composition_doctrine]]. A semantic-annotation tag
// (here @profiles) composes across rule references; a rule PRESENT under a profile must be
// SATISFIABLE under it. Computed as a per-profile SATISFIABILITY fixpoint mirroring
// compute_nullable. Pure + parser-agnostic: keyed only on ASTNode structure + a plain
// rule->profiles map + the profile universe (the caller extracts the map from @profiles).

/// PRESENT under `profile` = the rule's @profiles set is universal (untagged / empty) OR
/// explicitly contains `profile`. This mirrors the generator's profile filter.
fn rule_present_under_profile(
    rule: &str,
    rule_profiles: &HashMap<String, Vec<String>>,
    profile: &str,
) -> bool {
    match rule_profiles.get(rule) {
        None => true,
        Some(profiles) => profiles.is_empty() || profiles.iter().any(|p| p == profile),
    }
}

/// Is `node` SATISFIABLE under `profile` given the current per-rule estimates (fixpoint)?
/// Composition algebra: terminal/lexeme = ⊤; rule ref = present-under-profile AND satisfiable;
/// sequence = AND (every required part, ∩); alternation = OR (∪); optional/`*` = ⊤ (skippable,
/// so a star-guarded reference NEVER makes its container an orphan); `+`/`{N,}` (min≥1) = inner;
/// lookahead = ⊤ (a syntactic predicate produces nothing). Conservative like `node_nullable`.
fn node_satisfiable(
    node: &ASTNode,
    sat: &HashMap<String, bool>,
    rule_profiles: &HashMap<String, Vec<String>>,
    defined: &HashSet<String>,
    profile: &str,
) -> bool {
    match node {
        ASTNode::Or { alternatives } => alternatives
            .iter()
            .any(|a| node_satisfiable(a, sat, rule_profiles, defined, profile)),
        ASTNode::Sequence { elements } => elements
            .iter()
            .all(|e| node_satisfiable(e, sat, rule_profiles, defined, profile)),
        ASTNode::Quantified { element, quantifier } => {
            let (min, _) = parse_quantifier_bounds(quantifier).unwrap_or((0, None));
            min == 0 || node_satisfiable(element, sat, rule_profiles, defined, profile)
        }
        ASTNode::Lookahead { .. } => true,
        ASTNode::Atom { value } => match value {
            ASTValue::Node(inner) => {
                node_satisfiable(inner, sat, rule_profiles, defined, profile)
            }
            ASTValue::Token(parts) => match referenced_rule(parts) {
                // A reference to a rule NOT defined in THIS grammar is an external/include
                // reference (PGEN's include(...) system) — treat as ⊤ (no false positives),
                // exactly as detect_nonterminating_rules treats undefined references.
                Some(r) if !defined.contains(r) => true,
                Some(r) => {
                    rule_present_under_profile(r, rule_profiles, profile)
                        && sat.get(r).copied().unwrap_or(false)
                }
                None => true, // a terminal lexeme is profile-neutral
            },
        },
    }
}

/// The per-profile satisfiability fixpoint (shared by `detect_profile_orphans` +
/// `derive_rule_profiles`): `result[profile][rule]` = is `rule` satisfiable under `profile`.
fn compute_sat_by_profile(
    grammar: &HashMap<String, ASTNode>,
    rule_order: &[String],
    rule_profiles: &HashMap<String, Vec<String>>,
    all_profiles: &[String],
) -> HashMap<String, HashMap<String, bool>> {
    let defined: HashSet<String> = grammar.keys().cloned().collect();
    let mut sat_by_profile: HashMap<String, HashMap<String, bool>> = HashMap::new();
    for profile in all_profiles {
        let mut sat: HashMap<String, bool> = HashMap::new();
        loop {
            let mut changed = false;
            for rule in rule_order {
                if !rule_present_under_profile(rule, rule_profiles, profile) {
                    continue;
                }
                let Some(body) = grammar.get(rule) else { continue };
                let value = node_satisfiable(body, &sat, rule_profiles, &defined, profile);
                if sat.get(rule).copied().unwrap_or(false) != value {
                    sat.insert(rule.clone(), value);
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        sat_by_profile.insert(profile.clone(), sat);
    }
    sat_by_profile
}

/// ANNOTATION-COMPOSITION.4 (derive-by-default — the doctrine's R1): the DERIVED `@profiles` set
/// per rule = the profiles under which the rule is SATISFIABLE (the composition-algebra fixpoint).
/// A rule's effective profiles can thus be COMPUTED from its productions rather than hand-declared;
/// an explicit `@profiles` then reads as a *verified assertion* (declared ⊆ derived — a
/// declared-but-unsatisfiable profile is exactly a `ProfileOrphan`). Consumed today as the orphan
/// fix-suggestion (`suggested_profiles`); a future option is the generator consuming it to
/// auto-eliminate manual tags (deferred — the lint already makes inconsistency a hard-gate, so that
/// is a no-op-behaviour convenience, not a correctness need). PURE; deterministic; parser-agnostic.
pub fn derive_rule_profiles(
    grammar: &HashMap<String, ASTNode>,
    rule_order: &[String],
    rule_profiles: &HashMap<String, Vec<String>>,
    all_profiles: &[String],
) -> HashMap<String, Vec<String>> {
    let sat_by_profile = compute_sat_by_profile(grammar, rule_order, rule_profiles, all_profiles);
    let mut derived: HashMap<String, Vec<String>> = HashMap::new();
    for rule in rule_order {
        if !grammar.contains_key(rule) {
            continue;
        }
        let profiles: Vec<String> = all_profiles
            .iter()
            .filter(|p| {
                sat_by_profile
                    .get(*p)
                    .and_then(|s| s.get(rule))
                    .copied()
                    .unwrap_or(false)
            })
            .cloned()
            .collect();
        derived.insert(rule.clone(), profiles);
    }
    derived
}

/// Extract each rule's `@profiles` set + the profile universe from the annotations (the same shape
/// the generator filters by, and `run_grammar_lint` computes inline). Shared by the profile-orphan
/// certificate checker (G.2.1b). A rule absent from the map is universal (present under all profiles).
pub fn extract_profile_context(
    annotations: &Annotations,
) -> (HashMap<String, Vec<String>>, Vec<String>) {
    let mut rule_profiles: HashMap<String, Vec<String>> = HashMap::new();
    let mut universe: std::collections::BTreeSet<String> = Default::default();
    for (rule, entries) in &annotations.semantic_annotations {
        for annotation in entries {
            if annotation.name().map(|n| n.trim().to_ascii_lowercase()).as_deref() != Some("profiles")
            {
                continue;
            }
            if let Some(list) = parse_semantic_string_list(annotation.ast().payload_text()) {
                let profs: Vec<String> = list
                    .into_iter()
                    .map(|v| v.trim().to_ascii_lowercase())
                    .filter(|v| !v.is_empty())
                    .collect();
                if !profs.is_empty() {
                    for p in &profs {
                        universe.insert(p.clone());
                    }
                    rule_profiles.insert(rule.clone(), profs);
                }
            }
        }
    }
    (rule_profiles, universe.into_iter().collect())
}

/// ANNOTATION-COMPOSITION.2: detect @profiles ORPHANS — a rule PRESENT under a profile but NOT
/// satisfiable there (every production references a rule the profile filter removed → dangling).
/// A rule is reported under `profile` only if it IS satisfiable under some OTHER profile, so a
/// genuinely non-terminating rule (unsatisfiable everywhere) is NOT mis-reported here (that is
/// `detect_nonterminating_rules`' job) — this isolates the *profile-specific* breakage.
/// `rule_profiles`: rule -> its @profiles list (absent/empty = universal). `all_profiles`: the
/// profile universe (union of declared profiles). PURE; parser-agnostic.
pub fn detect_profile_orphans(
    grammar: &HashMap<String, ASTNode>,
    rule_order: &[String],
    rule_profiles: &HashMap<String, Vec<String>>,
    all_profiles: &[String],
) -> Vec<WellformednessIssue> {
    // Rules actually defined in THIS grammar (a reference outside this set is external/include).
    let sat_by_profile = compute_sat_by_profile(grammar, rule_order, rule_profiles, all_profiles);

    let mut out = Vec::new();
    for profile in all_profiles {
        let sat = &sat_by_profile[profile];
        for rule in rule_order {
            if !grammar.contains_key(rule)
                || !rule_present_under_profile(rule, rule_profiles, profile)
                || sat.get(rule).copied().unwrap_or(false)
            {
                continue;
            }
            // Profile-specific orphan ONLY: satisfiable under some other profile (else it is
            // globally non-terminating, reported elsewhere).
            let satisfiable_elsewhere = all_profiles.iter().any(|q| {
                q != profile
                    && sat_by_profile
                        .get(q)
                        .and_then(|s| s.get(rule))
                        .copied()
                        .unwrap_or(false)
            });
            if satisfiable_elsewhere {
                // ANNOTATION-COMPOSITION.4: the DERIVED minimal fix = the profiles where the rule
                // IS satisfiable; tagging it with these makes the orphan disappear.
                let suggested_profiles: Vec<String> = all_profiles
                    .iter()
                    .filter(|q| {
                        sat_by_profile
                            .get(*q)
                            .and_then(|s| s.get(rule))
                            .copied()
                            .unwrap_or(false)
                    })
                    .cloned()
                    .collect();
                out.push(WellformednessIssue::ProfileOrphan {
                    rule: rule.clone(),
                    profile: profile.clone(),
                    suggested_profiles,
                });
            }
        }
    }
    out
}

/// The fact-query primitives whose FIRST argument is a fact-KIND and which read the store that
/// `@emit_fact` populates (`semantic_runtime.rs`: each does `self.fact_index...(args[0])`). A kind
/// consulted via any of these but emitted by nothing can never be established.
const FACT_QUERY_PRIMITIVES: [&str; 4] = [
    "has_fact",
    "lacks_fact",
    "fact_attribute_equals",
    "fact_count_at_least",
];

/// A literal (statically-known) fact-kind value, or `None` for a dynamic/arg-ref value (which we
/// conservatively skip — under-report, never false-accuse).
fn literal_fact_kind(value: &PredicateValue) -> Option<String> {
    match value {
        PredicateValue::StringLit(s) | PredicateValue::IdentLit(s) => Some(s.clone()),
        _ => None,
    }
}

fn collect_consulted_kinds_in_call(call: &PrimitiveCall, out: &mut Vec<(String, String)>) {
    if FACT_QUERY_PRIMITIVES.contains(&call.name.as_str()) {
        if let Some(kind) = call.args.first().and_then(literal_fact_kind) {
            out.push((kind, call.name.clone()));
        }
    }
}

fn collect_consulted_kinds_in_value(value: &PredicateValue, out: &mut Vec<(String, String)>) {
    if let PredicateValue::AttributeOf { call, .. } = value {
        collect_consulted_kinds_in_call(call, out);
    }
}

/// Walk a parsed predicate expression collecting `(kind, primitive)` for every literal fact-kind
/// consulted by a fact-query primitive anywhere in the boolean/comparison tree.
fn collect_consulted_kinds(expr: &PredicateExpr, out: &mut Vec<(String, String)>) {
    match expr {
        PredicateExpr::Call(call) => collect_consulted_kinds_in_call(call, out),
        PredicateExpr::Not(inner) => collect_consulted_kinds(inner, out),
        PredicateExpr::And(a, b) | PredicateExpr::Or(a, b) => {
            collect_consulted_kinds(a, out);
            collect_consulted_kinds(b, out);
        }
        PredicateExpr::Compare { lhs, op: _, rhs } => {
            collect_consulted_kinds_in_value(lhs, out);
            collect_consulted_kinds_in_value(rhs, out);
        }
        PredicateExpr::In { lhs, set } => {
            collect_consulted_kinds_in_value(lhs, out);
            for v in set {
                collect_consulted_kinds_in_value(v, out);
            }
        }
    }
}

/// Extract `(kind, primitive)` consulted by a single `@predicate` spec — handling BOTH the inline
/// expression form (`@predicate: has_fact(type_name, $1)`, where the whole expression text is in
/// `spec.name`) AND the structured form (`@predicate: {name: has_fact, args: [type_name, $1]}`).
fn consulted_kinds_in_predicate(spec_name: &str, args: &[super::UnifiedSemanticValue]) -> Vec<(String, String)> {
    let mut out = Vec::new();
    // Inline expression form: the text parses into a PredicateExpr tree.
    if let Ok(expr) = parse_predicate_expression(spec_name) {
        collect_consulted_kinds(&expr, &mut out);
    }
    // Structured form: the directive IS a bare fact-query primitive named in `spec_name` with the
    // kind in args[0]. (Covers payloads where the expression isn't a parseable inline string.)
    if FACT_QUERY_PRIMITIVES.contains(&spec_name) {
        if let Some(kind) = args.first().and_then(|v| match v {
            super::UnifiedSemanticValue::String(s) | super::UnifiedSemanticValue::Identifier(s) => {
                Some(s.clone())
            }
            _ => None,
        }) {
            out.push((kind, spec_name.to_string()));
        }
    }
    out
}

/// GRAMMAR-WELLFORMED.F1 — data-dependent BINDING-BEFORE-USE (Jim/Mandelbaum/Walker, POPL 2010),
/// sound decidable core. A `@predicate` may only consult a fact-KIND that some `@emit_fact` can
/// establish. `has_fact`/`lacks_fact`/`fact_attribute_equals`/`fact_count_at_least` all query the
/// exact store `@emit_fact` populates (`fact_index`), so a consulted kind that NOTHING emits can
/// never be true — the predicate is degenerate and the rule effectively dead.
///
/// SOUNDNESS: the emitted-kind set is COMPLETE — every `@emit_fact` carries a literal `kind`
/// (`parse_emit_fact` requires a non-empty scalar) and `fact_index` is populated only from emitted
/// facts. We enumerate emit_facts across ALL annotation surfaces (rule-level, per-branch, and
/// mid-sequence) so no emitter is missed. On the CONSULTED side we only flag LITERAL kinds; a
/// dynamic/arg-ref kind is skipped (under-report, never false-accuse). PURE; parser-agnostic.
/// Collect every fact-KIND emitted by an `@emit_fact` across ALL annotation surfaces (rule-level,
/// per-branch, mid-sequence). The COMPLETE emitter enumeration — the soundness basis for F1
/// (`detect_unbound_fact_kinds`) and the G.2.1 unbound-fact certificate checker. Every `@emit_fact`
/// carries a literal `kind` (`parse_emit_fact` requires a non-empty scalar).
pub fn collect_emitted_fact_kinds(annotations: &Annotations) -> HashSet<String> {
    let mut emitted: HashSet<String> = HashSet::new();
    let mut visit = |ann: &SemanticAnnotation| {
        if let Ok(Some(SemanticRuntimeDirective::EmitFact(spec))) =
            parse_semantic_runtime_directive(ann)
        {
            emitted.insert(spec.kind);
        }
    };
    for anns in annotations.semantic_annotations.values() {
        for ann in anns {
            visit(ann);
        }
    }
    for branches in annotations.branch_semantic_annotations.values() {
        for branch in branches {
            for ann in branch {
                visit(ann);
            }
        }
    }
    for branches in annotations.branch_mid_sequence_semantic_annotations.values() {
        for branch in branches {
            for mid in branch {
                visit(&mid.annotation);
            }
        }
    }
    emitted
}

pub fn detect_unbound_fact_kinds(annotations: &Annotations) -> Vec<WellformednessIssue> {
    let emitted = collect_emitted_fact_kinds(annotations);
    // (rule, kind, primitive) consulted via a fact-query primitive.
    let mut consulted: Vec<(String, String, String)> = Vec::new();
    let mut visit = |rule: &str, ann: &SemanticAnnotation| {
        if let Ok(Some(SemanticRuntimeDirective::Predicate(spec))) =
            parse_semantic_runtime_directive(ann)
        {
            for (kind, primitive) in consulted_kinds_in_predicate(&spec.name, &spec.args) {
                consulted.push((rule.to_string(), kind, primitive));
            }
        }
    };
    for (rule, anns) in &annotations.semantic_annotations {
        for ann in anns {
            visit(rule, ann);
        }
    }
    for (rule, branches) in &annotations.branch_semantic_annotations {
        for branch in branches {
            for ann in branch {
                visit(rule, ann);
            }
        }
    }
    for (rule, branches) in &annotations.branch_mid_sequence_semantic_annotations {
        for branch in branches {
            for mid in branch {
                visit(rule, &mid.annotation);
            }
        }
    }

    // Deterministic order: sort the findings (HashMap iteration order is non-deterministic).
    let mut out: Vec<WellformednessIssue> = consulted
        .into_iter()
        .filter(|(_, kind, _)| !emitted.contains(kind))
        .map(|(rule, kind, primitive)| WellformednessIssue::UnboundFactKind { rule, kind, primitive })
        .collect();
    out.sort_by(|a, b| match (a, b) {
        (
            WellformednessIssue::UnboundFactKind { rule: r1, kind: k1, primitive: p1 },
            WellformednessIssue::UnboundFactKind { rule: r2, kind: k2, primitive: p2 },
        ) => (r1, k1, p1).cmp(&(r2, k2, p2)),
        _ => std::cmp::Ordering::Equal,
    });
    out.dedup();
    out
}

/// Rules referenced at the LEFT EDGE of `node` — i.e. reachable before any input is
/// necessarily consumed. A `Sequence` extends past a leading element only while that
/// element is nullable; a quantifier/lookahead exposes its element at the left edge.
fn leftmost_refs(node: &ASTNode, nullable: &HashMap<String, bool>, out: &mut HashSet<String>) {
    match node {
        ASTNode::Or { alternatives } => {
            for a in alternatives {
                leftmost_refs(a, nullable, out);
            }
        }
        ASTNode::Sequence { elements } => {
            for e in elements {
                leftmost_refs(e, nullable, out);
                if !node_nullable(e, nullable) {
                    break; // this element must consume → later elements are not left-edge
                }
            }
        }
        ASTNode::Quantified { element, .. } => leftmost_refs(element, nullable, out),
        ASTNode::Lookahead { element, .. } => leftmost_refs(element, nullable, out),
        ASTNode::Atom { value } => match value {
            ASTValue::Node(inner) => leftmost_refs(inner, nullable, out),
            ASTValue::Token(parts) => {
                if let Some(rule) = referenced_rule(parts) {
                    out.insert(rule.to_string());
                }
            }
        },
    }
}

/// Detect every left-recursive rule in the grammar (direct or indirect). PURE analysis.
/// Returns one `LeftRecursive` issue per offending rule (deterministic order via
/// `rule_order`), each with a concrete recursion cycle for the diagnostic.
pub fn detect_left_recursion(
    grammar: &HashMap<String, ASTNode>,
    rule_order: &[String],
) -> Vec<WellformednessIssue> {
    let nullable = compute_nullable(grammar, rule_order);

    // Left-edge call graph: rule -> rules it can reach at the left edge.
    let mut left_edge: HashMap<String, HashSet<String>> = HashMap::new();
    for rule in rule_order {
        let Some(body) = grammar.get(rule) else { continue };
        let mut refs = HashSet::new();
        leftmost_refs(body, &nullable, &mut refs);
        // Keep only references to defined rules.
        refs.retain(|r| grammar.contains_key(r));
        left_edge.insert(rule.clone(), refs);
    }

    let mut issues = Vec::new();
    for start in rule_order {
        if !grammar.contains_key(start.as_str()) {
            continue;
        }
        // DFS from `start`; report if we return to `start` along left edges.
        if let Some(cycle) = find_left_cycle(start, &left_edge) {
            issues.push(WellformednessIssue::LeftRecursive {
                rule: start.clone(),
                cycle,
            });
        }
    }
    issues
}

/// DFS that returns a cycle path `start -> … -> start` if `start` is left-recursive.
fn find_left_cycle(start: &str, left_edge: &HashMap<String, HashSet<String>>) -> Option<Vec<String>> {
    let mut stack: Vec<String> = vec![start.to_string()];
    let mut visited: HashSet<String> = HashSet::new();
    if dfs(start, start, left_edge, &mut stack, &mut visited) {
        Some(stack)
    } else {
        None
    }
}

fn dfs(
    start: &str,
    current: &str,
    left_edge: &HashMap<String, HashSet<String>>,
    stack: &mut Vec<String>,
    visited: &mut HashSet<String>,
) -> bool {
    let Some(next_set) = left_edge.get(current) else {
        return false;
    };
    // Deterministic neighbour order.
    let mut neighbours: Vec<&String> = next_set.iter().collect();
    neighbours.sort();
    for next in neighbours {
        if next == start {
            stack.push(start.to_string());
            return true; // closed the cycle back to start
        }
        if visited.insert(next.clone()) {
            stack.push(next.clone());
            if dfs(start, next, left_edge, stack, visited) {
                return true;
            }
            stack.pop();
        }
    }
    false
}

// ---------------------------------------------------------------------------
// PARSE-SOTA.9 / adoption A2 (⭐): static ordered-choice SHADOWING lint.
//
// In a PEG ordered choice `a / b`, an alternative is UNREACHABLE if an earlier
// alternative always matches whenever it could (the `A := a | ab` quirk that the ALL(*)
// authors call out and that the SystemVerilog grammar work repeatedly hits as
// catch-all-shadows-specific). A general subsumption check risks false positives, so this
// lint is deliberately SOUND-ONLY — it flags just the two unambiguous structural cases:
//   (1) a DUPLICATE alternative (an exact structural copy of an earlier one), and
//   (2) an earlier alternative that is a FIXED-TERMINAL prefix of a later one
//       (`a` before `a b` → `a b` is dead, because PEG commits to `a`).
// It is a WARNING (emitted via the DIAG-SEVERITY pgen_warn! channel when wired), not a
// hard error. Pure analysis.
// ---------------------------------------------------------------------------

/// An ordered-choice alternative shadowed (made unreachable) by an earlier one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShadowingIssue {
    pub rule: String,
    pub node_path: String,
    /// Index of the unreachable alternative.
    pub shadowed_index: usize,
    /// Index of the earlier alternative that shadows it.
    pub by_index: usize,
    pub reason: ShadowingReason,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShadowingReason {
    /// Exact structural duplicate of the earlier alternative.
    DuplicateAlternative,
    /// The earlier alternative is a fixed-terminal prefix of this one (PEG commits first).
    FixedTerminalPrefix,
    /// The earlier alternative ALWAYS SUCCEEDS (e.g. `e?`, `e*`, an all-optional sequence) — so
    /// PEG commits to it on every input and this one can never be tried.
    EarlierAlwaysMatches,
}

impl ShadowingReason {
    /// Is this reason part of the HARD `--lint-grammar` gate yet? The exact-duplicate and
    /// fixed-terminal-prefix reasons are gated (every authored grammar was cleaned to 0 — A1a/.1/.2);
    /// `EarlierAlwaysMatches` is the newly-added A2 detector and currently has an UNFIXED backlog in
    /// the SV grammar (the `( X )?`-as-an-alternative anti-pattern), so it is surfaced as a WARNING
    /// until that backlog is cleaned LRM-grounded, exactly as exact-dup shadowing was staged before
    /// A1a promoted it. Promote here once the warnings reach 0 (GRAMMAR-WELLFORMED.A2.1).
    pub fn is_hard_gate(&self) -> bool {
        match self {
            ShadowingReason::DuplicateAlternative | ShadowingReason::FixedTerminalPrefix => true,
            ShadowingReason::EarlierAlwaysMatches => false,
        }
    }
}

impl ShadowingIssue {
    pub fn message(&self) -> String {
        let why = match self.reason {
            ShadowingReason::DuplicateAlternative => "is an exact duplicate of",
            ShadowingReason::FixedTerminalPrefix => "is a fixed-terminal prefix of",
            ShadowingReason::EarlierAlwaysMatches => "always matches (never fails) earlier than",
        };
        format!(
            "grammar shadowing: in rule '{}' (ordered choice at {}), alternative #{} is unreachable — alternative #{} {} it (PEG commits to the earlier alternative); reorder (specific before general) or merge",
            self.rule, self.node_path, self.shadowed_index, self.by_index, why
        )
    }
}

/// If `node` is composed ENTIRELY of fixed terminals (a terminal atom, or a sequence of
/// fixed-terminal pieces), return that ordered list of lexemes; else None (a rule
/// reference, alternation, quantifier, lookahead, or any non-fixed structure → not a
/// guaranteed fixed match, so unsound to treat as a prefix).
fn fixed_terminal_seq(node: &ASTNode) -> Option<Vec<String>> {
    match node {
        ASTNode::Atom { value } => match value {
            ASTValue::Token(parts) => {
                if referenced_rule(parts).is_some() {
                    None // a rule reference can fail → not a guaranteed fixed match
                } else if let (Some(TokenValue::String(_)), Some(TokenValue::String(v))) =
                    (parts.first(), parts.get(1))
                {
                    Some(vec![v.clone()])
                } else {
                    None
                }
            }
            ASTValue::Node(inner) => fixed_terminal_seq(inner),
        },
        ASTNode::Sequence { elements } => {
            let mut seq = Vec::new();
            for e in elements {
                seq.extend(fixed_terminal_seq(e)?); // any non-fixed element → not fixed
            }
            Some(seq)
        }
        _ => None,
    }
}

/// The leading fixed-terminal lexemes of `node` (a possibly-empty prefix). Used to test
/// whether an earlier fixed sequence prefixes a later alternative.
fn leading_fixed_terminals(node: &ASTNode) -> Vec<String> {
    match node {
        ASTNode::Sequence { elements } => {
            let mut seq = Vec::new();
            for e in elements {
                match fixed_terminal_seq(e) {
                    Some(part) => seq.extend(part),
                    None => break, // stop at the first non-fixed element
                }
            }
            seq
        }
        other => fixed_terminal_seq(other).unwrap_or_default(),
    }
}

fn ast_eq(a: &ASTNode, b: &ASTNode) -> bool {
    // ASTNode derives Serialize; compare structurally via the serialized value.
    match (serde_json::to_value(a), serde_json::to_value(b)) {
        (Ok(va), Ok(vb)) => va == vb,
        _ => false,
    }
}

/// Detect shadowed (unreachable) alternatives in every ordered choice of every rule.
/// PURE analysis; deterministic order via `rule_order` + source order of Or nodes.
pub fn detect_ordered_choice_shadowing(
    grammar: &HashMap<String, ASTNode>,
    rule_order: &[String],
) -> Vec<ShadowingIssue> {
    let always = compute_always_succeeds(grammar, rule_order);
    let mut issues = Vec::new();
    for rule in rule_order {
        let Some(body) = grammar.get(rule) else { continue };
        collect_shadowing(rule, body, "root", &always, &mut issues);
    }
    issues
}

fn collect_shadowing(
    rule: &str,
    node: &ASTNode,
    path: &str,
    always: &HashMap<String, bool>,
    out: &mut Vec<ShadowingIssue>,
) {
    match node {
        ASTNode::Or { alternatives } => {
            for (j, alt_j) in alternatives.iter().enumerate() {
                for (i, alt_i) in alternatives.iter().enumerate().take(j) {
                    let reason = if ast_eq(alt_i, alt_j) {
                        Some(ShadowingReason::DuplicateAlternative)
                    } else if node_always_succeeds(alt_i, always) {
                        // An earlier alternative that always succeeds makes this one (and every
                        // later one) unreachable — PEG commits to the first success.
                        Some(ShadowingReason::EarlierAlwaysMatches)
                    } else if let Some(prefix) = fixed_terminal_seq(alt_i) {
                        // alt_i is a guaranteed fixed match; if it prefixes alt_j's leading
                        // fixed terminals, PEG commits to alt_i and alt_j is unreachable.
                        let later = leading_fixed_terminals(alt_j);
                        if !prefix.is_empty()
                            && later.len() >= prefix.len()
                            && later[..prefix.len()] == prefix[..]
                        {
                            Some(ShadowingReason::FixedTerminalPrefix)
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    if let Some(reason) = reason {
                        out.push(ShadowingIssue {
                            rule: rule.to_string(),
                            node_path: path.to_string(),
                            shadowed_index: j,
                            by_index: i,
                            reason,
                        });
                        break; // one shadower per alternative is enough
                    }
                }
            }
            for (idx, alt) in alternatives.iter().enumerate() {
                collect_shadowing(rule, alt, &format!("{}/o{}", path, idx), always, out);
            }
        }
        ASTNode::Sequence { elements } => {
            for (idx, e) in elements.iter().enumerate() {
                collect_shadowing(rule, e, &format!("{}/s{}", path, idx), always, out);
            }
        }
        ASTNode::Quantified { element, .. } => {
            collect_shadowing(rule, element, &format!("{}/q", path), always, out);
        }
        ASTNode::Lookahead { element, .. } => {
            collect_shadowing(rule, element, &format!("{}/l", path), always, out);
        }
        ASTNode::Atom { value } => {
            if let ASTValue::Node(inner) = value {
                collect_shadowing(rule, inner, &format!("{}/a", path), always, out);
            }
        }
    }
}

// =============================================================================
// GRAMMAR-WELLFORMED.G — the CERTIFYING LINTER (G.1: certificate model + the
// independent re-checker for UNREACHABILITY verdicts).
//
// Every `dead` (unreachable) verdict ships a structured, self-contained CERTIFICATE.
// `verify_unreachability_certificate` is the INDEPENDENT CHECKER: it re-navigates to
// the cited node from the grammar and re-derives the claim directly — it does NOT
// trust the detector's output. A certificate that fails to verify is a linter bug (or
// a tampered certificate). The exact-duplicate and fixed-terminal-prefix re-checks are
// trivial + fully independent; the always-succeeds re-check re-derives via
// `node_always_succeeds` (a structural-witness form that needs no fixpoint is a planned
// G.1.1 refinement). Reachability certificates (WITNESSES, generator-produced) are G.3.
// =============================================================================

/// The structured, checkable reason an ordered-choice alternative is unreachable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnreachabilityReason {
    /// Alternative `by` is an exact structural duplicate of the dead one.
    DuplicateOf { by: usize },
    /// Alternative `by` is a fixed-terminal prefix of the dead one (PEG commits to `by`).
    FixedTerminalPrefixBy { by: usize },
    /// Alternative `by` ALWAYS SUCCEEDS, so PEG commits before the dead one is ever tried.
    EarlierArmAlwaysSucceeds { by: usize },
}

/// A certificate that one ordered-choice alternative is UNREACHABLE — the proof half of the
/// certifying linter. Self-contained: `rule` + `node_path` locate the `Or` node, `dead_index`
/// names the dead alternative, `reason` names the earlier alternative that kills it. An
/// independent checker (`verify_unreachability_certificate`) re-derives this from the grammar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnreachabilityCertificate {
    pub rule: String,
    pub node_path: String,
    pub dead_index: usize,
    pub reason: UnreachabilityReason,
}

impl ShadowingIssue {
    /// Emit the structured unreachability certificate for this shadowing finding (G.1).
    pub fn certificate(&self) -> UnreachabilityCertificate {
        let reason = match self.reason {
            ShadowingReason::DuplicateAlternative => {
                UnreachabilityReason::DuplicateOf { by: self.by_index }
            }
            ShadowingReason::FixedTerminalPrefix => {
                UnreachabilityReason::FixedTerminalPrefixBy { by: self.by_index }
            }
            ShadowingReason::EarlierAlwaysMatches => {
                UnreachabilityReason::EarlierArmAlwaysSucceeds { by: self.by_index }
            }
        };
        UnreachabilityCertificate {
            rule: self.rule.clone(),
            node_path: self.node_path.clone(),
            dead_index: self.shadowed_index,
            reason,
        }
    }
}

/// Navigate the `node_path` (as produced by `collect_shadowing`: `root` then `/o{i}` ordered-choice
/// alternative, `/s{i}` sequence element, `/q` quantifier element, `/l` lookahead element, `/a` atom
/// inner) from a rule body to the cited node. Returns `None` if the path does not resolve (a
/// structurally invalid certificate). Independent of the detector — pure navigation.
fn navigate_node_path<'a>(body: &'a ASTNode, node_path: &str) -> Option<&'a ASTNode> {
    let mut cur = body;
    let mut segs = node_path.split('/');
    if segs.next() != Some("root") {
        return None;
    }
    for seg in segs {
        if seg.is_empty() {
            continue;
        }
        let (tag, idx) = seg.split_at(1);
        match (tag, cur) {
            ("o", ASTNode::Or { alternatives }) => {
                cur = alternatives.get(idx.parse::<usize>().ok()?)?;
            }
            ("s", ASTNode::Sequence { elements }) => {
                cur = elements.get(idx.parse::<usize>().ok()?)?;
            }
            ("q", ASTNode::Quantified { element, .. }) => cur = element,
            ("l", ASTNode::Lookahead { element, .. }) => cur = element,
            ("a", ASTNode::Atom { value: ASTValue::Node(inner) }) => cur = inner,
            _ => return None,
        }
    }
    Some(cur)
}

/// THE CHECKER (G.1/G.2 seed): independently re-validate an unreachability certificate against the
/// grammar. Re-navigates to the cited `Or` node and re-derives the deadness claim DIRECTLY from the
/// AST — it never trusts the detector. Returns `Ok(())` iff the certificate genuinely holds; `Err`
/// (with the reason) means the certificate is invalid — a linter bug or a tampered/stale certificate.
pub fn verify_unreachability_certificate(
    grammar: &HashMap<String, ASTNode>,
    rule_order: &[String],
    cert: &UnreachabilityCertificate,
) -> Result<(), String> {
    let body = grammar
        .get(&cert.rule)
        .ok_or_else(|| format!("certificate cites unknown rule '{}'", cert.rule))?;
    let node = navigate_node_path(body, &cert.node_path)
        .ok_or_else(|| format!("certificate path '{}' does not resolve in rule '{}'", cert.node_path, cert.rule))?;
    let ASTNode::Or { alternatives } = node else {
        return Err(format!("certificate path '{}' is not an ordered choice", cert.node_path));
    };
    let by = match &cert.reason {
        UnreachabilityReason::DuplicateOf { by }
        | UnreachabilityReason::FixedTerminalPrefixBy { by }
        | UnreachabilityReason::EarlierArmAlwaysSucceeds { by } => *by,
    };
    if by >= cert.dead_index {
        return Err(format!(
            "certificate shadower #{by} is not earlier than the dead alternative #{}",
            cert.dead_index
        ));
    }
    let by_alt = alternatives
        .get(by)
        .ok_or_else(|| format!("certificate shadower index #{by} out of range"))?;
    let dead_alt = alternatives
        .get(cert.dead_index)
        .ok_or_else(|| format!("certificate dead index #{} out of range", cert.dead_index))?;
    let holds = match &cert.reason {
        UnreachabilityReason::DuplicateOf { .. } => ast_eq(by_alt, dead_alt),
        UnreachabilityReason::FixedTerminalPrefixBy { .. } => match fixed_terminal_seq(by_alt) {
            Some(prefix) => {
                let later = leading_fixed_terminals(dead_alt);
                !prefix.is_empty() && later.len() >= prefix.len() && later[..prefix.len()] == prefix[..]
            }
            None => false,
        },
        UnreachabilityReason::EarlierArmAlwaysSucceeds { .. } => {
            let always = compute_always_succeeds(grammar, rule_order);
            node_always_succeeds(by_alt, &always)
        }
    };
    if holds {
        Ok(())
    } else {
        Err(format!(
            "certificate for rule '{}' at '{}' does NOT hold: alternative #{by} does not shadow #{} ({:?})",
            cert.rule, cert.node_path, cert.dead_index, cert.reason
        ))
    }
}

/// GRAMMAR-WELLFORMED.G.2: a certificate for ANY decidable unreachability verdict (generalizes the
/// shadowing `UnreachabilityCertificate`). Each variant carries enough to be re-derived from the
/// grammar by the independent checker. (Profile-orphan + unbound-fact certificates — which need the
/// annotations/profiles context — are the planned G.2.1 extension; witnesses for REACHABLE fragments
/// are G.3.)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WellformednessCertificate {
    /// An ordered-choice alternative is dead (shadowing) — see `UnreachabilityCertificate`.
    DeadAlternative(UnreachabilityCertificate),
    /// A rule is unreachable from every root (A1b).
    UnreachableRule { rule: String },
    /// A `@predicate` in `rule` consults a fact-`kind` no `@emit_fact` establishes (F1).
    UnboundFactKind { rule: String, kind: String },
    /// `rule` is present under `profile` but not satisfiable there (a @profiles orphan).
    ProfileOrphan { rule: String, profile: String },
}

/// THE CHECKER (G.2): independently re-validate ANY wellformedness certificate against the grammar.
/// Dispatches per variant; each re-derives the claim directly (never trusts the detector). `Ok(())`
/// iff the certificate genuinely holds. `annotations` is needed only for the annotation-derived
/// variants (`UnboundFactKind`); pass `None` for the pure-structural ones.
pub fn verify_wellformedness_certificate(
    grammar: &HashMap<String, ASTNode>,
    rule_order: &[String],
    annotations: Option<&Annotations>,
    cert: &WellformednessCertificate,
) -> Result<(), String> {
    match cert {
        WellformednessCertificate::DeadAlternative(c) => {
            verify_unreachability_certificate(grammar, rule_order, c)
        }
        WellformednessCertificate::UnreachableRule { rule } => {
            if !grammar.contains_key(rule) {
                return Err(format!("certificate cites unknown rule '{rule}'"));
            }
            // Re-derive reachability from the roots, independently.
            if reachable_rules(grammar, rule_order).contains(rule) {
                Err(format!(
                    "certificate claims rule '{rule}' UNREACHABLE, but it IS reachable from a root"
                ))
            } else {
                Ok(())
            }
        }
        WellformednessCertificate::UnboundFactKind { rule, kind } => {
            let Some(ann) = annotations else {
                return Err(format!(
                    "unbound-fact certificate for rule '{rule}' kind '{kind}' needs the annotations to re-check"
                ));
            };
            // Re-collect every emitted kind independently; the claim holds iff none emits `kind`.
            if collect_emitted_fact_kinds(ann).contains(kind) {
                Err(format!(
                    "certificate claims fact-kind '{kind}' (consulted in '{rule}') is unemitted, but an @emit_fact DOES emit it"
                ))
            } else {
                Ok(())
            }
        }
        WellformednessCertificate::ProfileOrphan { rule, profile } => {
            let Some(ann) = annotations else {
                return Err(format!(
                    "profile-orphan certificate for rule '{rule}' under '{profile}' needs the annotations to re-check"
                ));
            };
            if !grammar.contains_key(rule) {
                return Err(format!("certificate cites unknown rule '{rule}'"));
            }
            // Re-derive per-profile satisfiability independently; the claim holds iff `rule` is
            // PRESENT under `profile` yet NOT satisfiable there.
            let (rule_profiles, all_profiles) = extract_profile_context(ann);
            let sat_by_profile =
                compute_sat_by_profile(grammar, rule_order, &rule_profiles, &all_profiles);
            let present = rule_present_under_profile(rule, &rule_profiles, profile);
            let sat_here = sat_by_profile
                .get(profile)
                .and_then(|s| s.get(rule))
                .copied()
                .unwrap_or(false);
            // Mirror detect_profile_orphans: a profile-SPECIFIC orphan is satisfiable under some
            // OTHER profile (else it is globally non-terminating, not a profile orphan).
            let sat_elsewhere = all_profiles.iter().any(|q| {
                q != profile
                    && sat_by_profile.get(q).and_then(|s| s.get(rule)).copied().unwrap_or(false)
            });
            if present && !sat_here && sat_elsewhere {
                Ok(())
            } else {
                Err(format!(
                    "certificate claims '{rule}' is a @profiles orphan under '{profile}', but present={present} sat_here={sat_here} sat_elsewhere={sat_elsewhere} (not a profile orphan)"
                ))
            }
        }
    }
}

/// GRAMMAR-WELLFORMED.G.3 — a reachability WITNESS certificate: proof-BY-CONSTRUCTION that
/// `fragment` is reachable. `input` is a string that, when parsed, EXERCISES the fragment. This is
/// the constructive dual of the unreachability PROOF (`WellformednessCertificate`): the linter says
/// "reachable", the stimuli generator DEMONSTRATES it. Verified by replaying `input` through the real
/// parser — so trust rests on the (tiny) replay, not on the generator's internals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReachabilityWitness {
    /// The rule or branch the witness exercises (e.g. `"my_rule"` or `"my_rule#2"`).
    pub fragment: String,
    /// The witness input string (a generator-produced sample that reaches `fragment`).
    pub input: String,
}

/// THE WITNESS CHECKER (G.3): independently re-validate a reachability witness. The caller supplies
/// `parse_and_cover` — it replays `input` through the REAL grammar parser and returns
/// `(parsed_ok, fragments_exercised)`. The witness holds iff the input parses AND the parse genuinely
/// exercises the claimed fragment. Parser-AGNOSTIC: the closure carries the grammar's parser, so this
/// works for every PGEN grammar (Phase H). A witness that does not parse, or parses but misses its
/// fragment, is REJECTED — exactly as the proof checker rejects a bogus proof.
pub fn verify_reachability_witness<F>(
    parse_and_cover: F,
    witness: &ReachabilityWitness,
) -> Result<(), String>
where
    F: Fn(&str) -> (bool, HashSet<String>),
{
    let (parsed, covered) = parse_and_cover(&witness.input);
    if !parsed {
        return Err(format!(
            "reachability witness for '{}' does NOT parse — not a valid witness",
            witness.fragment
        ));
    }
    if !covered.contains(&witness.fragment) {
        return Err(format!(
            "reachability witness for '{}' parses but does NOT exercise that fragment",
            witness.fragment
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule_ref(name: &str) -> ASTNode {
        ASTNode::Atom {
            value: ASTValue::Token(vec![
                TokenValue::String("rule_reference".into()),
                TokenValue::String(name.into()),
            ]),
        }
    }
    fn token(kind: &str, value: &str) -> ASTNode {
        ASTNode::Atom {
            value: ASTValue::Token(vec![
                TokenValue::String(kind.into()),
                TokenValue::String(value.into()),
            ]),
        }
    }
    fn seq(elems: Vec<ASTNode>) -> ASTNode {
        ASTNode::Sequence { elements: elems }
    }
    fn or(alts: Vec<ASTNode>) -> ASTNode {
        ASTNode::Or { alternatives: alts }
    }
    fn quant(element: ASTNode, q: &str) -> ASTNode {
        ASTNode::Quantified { element: Box::new(element), quantifier: q.to_string() }
    }
    fn look(element: ASTNode, positive: bool) -> ASTNode {
        ASTNode::Lookahead { element: Box::new(element), positive }
    }
    fn sem_named(name: &str, value: crate::ast_pipeline::UnifiedSemanticValue) -> SemanticAnnotation {
        SemanticAnnotation::Named {
            name: name.to_string(),
            ast: crate::ast_pipeline::UnifiedSemanticAST::Structured {
                canonical: String::new(),
                value,
            },
        }
    }
    fn emit_fact_ann(kind: &str) -> SemanticAnnotation {
        use crate::ast_pipeline::{UnifiedSemanticProperty as P, UnifiedSemanticValue as V};
        sem_named(
            "emit_fact",
            V::Object(vec![
                P { key: "kind".into(), value: V::Identifier(kind.into()) },
                P { key: "name".into(), value: V::RuleReference("$1".into()) },
            ]),
        )
    }
    fn predicate_ann(expr: &str) -> SemanticAnnotation {
        sem_named("predicate", crate::ast_pipeline::UnifiedSemanticValue::String(expr.into()))
    }
    fn profiles_ann(list: &str) -> SemanticAnnotation {
        // payload_text() returns the `canonical` string for a Structured AST; parse_semantic_string_list
        // parses `[a, b]`-style lists from it.
        SemanticAnnotation::Named {
            name: "profiles".to_string(),
            ast: crate::ast_pipeline::UnifiedSemanticAST::Structured {
                canonical: list.to_string(),
                value: crate::ast_pipeline::UnifiedSemanticValue::String(list.to_string()),
            },
        }
    }

    #[test]
    fn profile_orphan_certificate_verifies_and_rejects() {
        // GRAMMAR-WELLFORMED.G.2.1b: a profile-orphan certificate re-verifies; a bogus one is REJECTED.
        // base := variant (untagged → present under all profiles); variant @profiles [sv_2023];
        // dummy @profiles [sv_2017] (puts sv_2017 in the universe). Under sv_2017 base references the
        // absent variant → base is a profile orphan under sv_2017; under sv_2023 it is satisfiable.
        let mut g = HashMap::new();
        g.insert("base".into(), rule_ref("variant"));
        g.insert("variant".into(), token("string", "=="));
        g.insert("dummy".into(), token("string", "d"));
        let order: Vec<String> = vec!["base".into(), "variant".into(), "dummy".into()];
        let mut ann = Annotations::default();
        ann.semantic_annotations.insert("variant".into(), vec![profiles_ann("[sv_2023]")]);
        ann.semantic_annotations.insert("dummy".into(), vec![profiles_ann("[sv_2017]")]);
        let cert =
            WellformednessCertificate::ProfileOrphan { rule: "base".into(), profile: "sv_2017".into() };
        assert!(
            verify_wellformedness_certificate(&g, &order, Some(&ann), &cert).is_ok(),
            "valid profile-orphan certificate must verify: {:?}",
            verify_wellformedness_certificate(&g, &order, Some(&ann), &cert)
        );
        // bogus: base is satisfiable under sv_2023 (variant present) → not an orphan there → rejected.
        let bogus = WellformednessCertificate::ProfileOrphan {
            rule: "base".into(),
            profile: "sv_2023".into(),
        };
        assert!(
            verify_wellformedness_certificate(&g, &order, Some(&ann), &bogus).is_err(),
            "claiming an orphan under a profile where the rule IS satisfiable must be rejected"
        );
    }

    #[test]
    fn detects_unbound_fact_kind_but_not_bound_one() {
        // GRAMMAR-WELLFORMED.F1: a @predicate consulting a fact-kind no @emit_fact emits is a
        // binding-before-use defect; one whose kind IS emitted is clean.
        let mut ann = Annotations::default();
        ann.semantic_annotations.insert("producer".into(), vec![emit_fact_ann("type_name")]);
        ann.semantic_annotations
            .insert("good".into(), vec![predicate_ann("has_fact(type_name, head)")]);
        ann.semantic_annotations
            .insert("bad".into(), vec![predicate_ann("has_fact(nonexistent_kind, head)")]);
        let issues = detect_unbound_fact_kinds(&ann);
        assert!(
            issues.iter().any(|i| matches!(i,
                WellformednessIssue::UnboundFactKind { rule, kind, .. }
                if rule == "bad" && kind == "nonexistent_kind")),
            "must flag the consulted-but-never-emitted fact-kind: {issues:?}"
        );
        assert!(
            !issues.iter().any(|i| matches!(i,
                WellformednessIssue::UnboundFactKind { kind, .. } if kind == "type_name")),
            "must NOT flag a kind that IS emitted by some @emit_fact: {issues:?}"
        );
    }

    #[test]
    fn unbound_fact_kind_skips_dynamic_kinds_and_finds_branch_emitters() {
        // (a) a dynamic (arg-ref) kind is conservatively skipped — never false-accused.
        // (b) an emitter living in a BRANCH-level annotation still counts (complete enumeration).
        let mut ann = Annotations::default();
        ann.branch_semantic_annotations
            .insert("producer".into(), vec![vec![emit_fact_ann("class_name")]]);
        ann.semantic_annotations.insert(
            "consumer".into(),
            vec![
                predicate_ann("has_fact(class_name, head)"), // bound by the branch emitter -> clean
                predicate_ann("has_fact($k, head)"),         // dynamic kind -> skipped
            ],
        );
        let issues = detect_unbound_fact_kinds(&ann);
        assert!(
            issues.is_empty(),
            "branch-level emitter must satisfy the consult; dynamic kind must be skipped: {issues:?}"
        );
    }

    #[test]
    fn detects_profile_orphan_dangling_under_profile() {
        // base := variant ; `variant` is @profiles ["sv_2023"] only. Under sv_2017 `base`
        // (untagged → present) references the removed `variant` → present-but-unsatisfiable =
        // a @profiles orphan (the binary_module_path_operator canary, reduced).
        let mut g = HashMap::new();
        g.insert("base".into(), rule_ref("variant"));
        g.insert("variant".into(), token("string", "=="));
        let order: Vec<String> = vec!["base".into(), "variant".into()];
        let mut profiles: HashMap<String, Vec<String>> = HashMap::new();
        profiles.insert("variant".into(), vec!["sv_2023".into()]);
        let all = vec!["sv_2017".into(), "sv_2023".into()];
        let issues = detect_profile_orphans(&g, &order, &profiles, &all);
        assert!(
            issues.iter().any(|i| matches!(i, WellformednessIssue::ProfileOrphan { rule, profile, suggested_profiles }
                if rule == "base" && profile == "sv_2017" && suggested_profiles == &vec!["sv_2023".to_string()])),
            "base must be flagged a profile orphan under sv_2017 WITH the derived fix-suggestion @profiles:[sv_2023] (ANNOTATION-COMPOSITION.4): {issues:?}"
        );
        assert!(
            !issues.iter().any(|i| matches!(i, WellformednessIssue::ProfileOrphan { profile, .. } if profile == "sv_2023")),
            "no orphan under sv_2023 (variant is present there): {issues:?}"
        );
    }

    #[test]
    fn derive_rule_profiles_computes_satisfiable_set_per_rule() {
        // base := variant ; variant @profiles [sv_2023] ; universal := <terminal> (untagged).
        let mut g = HashMap::new();
        g.insert("base".into(), rule_ref("variant"));
        g.insert("variant".into(), token("string", "=="));
        g.insert("universal".into(), token("string", "x"));
        let order: Vec<String> = vec!["base".into(), "variant".into(), "universal".into()];
        let mut profiles: HashMap<String, Vec<String>> = HashMap::new();
        profiles.insert("variant".into(), vec!["sv_2023".into()]);
        let all = vec!["sv_2017".into(), "sv_2023".into()];
        let derived = derive_rule_profiles(&g, &order, &profiles, &all);
        // variant is tagged sv_2023 → satisfiable only there; base references it → likewise only sv_2023.
        assert_eq!(derived.get("variant"), Some(&vec!["sv_2023".to_string()]));
        assert_eq!(derived.get("base"), Some(&vec!["sv_2023".to_string()]));
        // a plain-terminal untagged rule is satisfiable under both profiles.
        assert_eq!(
            derived.get("universal"),
            Some(&vec!["sv_2017".to_string(), "sv_2023".to_string()])
        );
    }

    #[test]
    fn detects_unreachable_rule_but_respects_multi_entry_roots() {
        // entry (rule_order[0]) -> main_body ; multi_entry (unreferenced root) -> alt_entry ;
        // dead_a <-> dead_b (mutual cycle, referenced only by each other → no root reaches them).
        let mut g = HashMap::new();
        g.insert("entry".into(), rule_ref("main_body"));
        g.insert("main_body".into(), token("string", "x"));
        g.insert("multi_entry".into(), rule_ref("alt_entry")); // unreferenced → a root
        g.insert("alt_entry".into(), token("string", "y")); // reachable via the multi_entry root
        g.insert("dead_a".into(), rule_ref("dead_b")); // dead island
        g.insert("dead_b".into(), rule_ref("dead_a")); // dead island
        let order: Vec<String> = ["entry", "main_body", "multi_entry", "alt_entry", "dead_a", "dead_b"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let issues = detect_unreachable_rules(&g, &order);
        for dead in ["dead_a", "dead_b"] {
            assert!(
                issues.iter().any(|i| matches!(i, WellformednessIssue::UnreachableRule { rule } if rule == dead)),
                "{dead} must be flagged unreachable (dead island): {issues:?}"
            );
        }
        // entry/main_body reachable from the canonical entry; multi_entry is an unreferenced ROOT
        // and alt_entry is reachable via it — none may be false-flagged (multi-entry safety).
        for keep in ["entry", "main_body", "multi_entry", "alt_entry"] {
            assert!(
                !issues.iter().any(|i| matches!(i, WellformednessIssue::UnreachableRule { rule } if rule == keep)),
                "{keep} must NOT be flagged (reachable or a root): {issues:?}"
            );
        }
    }

    #[test]
    fn star_guarded_profile_reference_is_not_an_orphan() {
        // base := "x" (variant)* ; `variant` is sv_2023-only. Under sv_2017 the (variant)* is
        // skippable (zero reps) so `base` stays satisfiable → NOT an orphan (the algebra's ⊤ for
        // optional/star — this is why module_path_expression is not flagged, only the standalone rule).
        let mut g = HashMap::new();
        g.insert(
            "base".into(),
            seq(vec![
                token("string", "x"),
                ASTNode::Quantified { element: Box::new(rule_ref("variant")), quantifier: "*".into() },
            ]),
        );
        g.insert("variant".into(), token("string", "=="));
        let order: Vec<String> = vec!["base".into(), "variant".into()];
        let mut profiles: HashMap<String, Vec<String>> = HashMap::new();
        profiles.insert("variant".into(), vec!["sv_2023".into()]);
        let all = vec!["sv_2017".into(), "sv_2023".into()];
        let issues = detect_profile_orphans(&g, &order, &profiles, &all);
        assert!(
            !issues.iter().any(|i| matches!(i, WellformednessIssue::ProfileOrphan { rule, .. } if rule == "base")),
            "a star-guarded profile reference must NOT make base an orphan: {issues:?}"
        );
    }

    #[test]
    fn detects_direct_left_recursion() {
        // expr := expr "+" term | term ; term := "n"
        let mut g = HashMap::new();
        g.insert("expr".into(), or(vec![seq(vec![rule_ref("expr"), token("op", "+"), rule_ref("term")]), rule_ref("term")]));
        g.insert("term".into(), token("string", "n"));
        let order: Vec<String> = vec!["expr".into(), "term".into()];
        let issues = detect_left_recursion(&g, &order);
        assert!(
            issues.iter().any(|i| matches!(i, WellformednessIssue::LeftRecursive { rule, .. } if rule == "expr")),
            "direct left recursion in expr must be detected: {issues:?}"
        );
        assert!(
            !issues.iter().any(|i| matches!(i, WellformednessIssue::LeftRecursive { rule, .. } if rule == "term")),
            "term is not left-recursive"
        );
    }

    #[test]
    fn detects_indirect_left_recursion() {
        // a := b ; b := a "z" | "w"
        let mut g = HashMap::new();
        g.insert("a".into(), rule_ref("b"));
        g.insert("b".into(), or(vec![seq(vec![rule_ref("a"), token("string", "z")]), token("string", "w")]));
        let order: Vec<String> = vec!["a".into(), "b".into()];
        let issues = detect_left_recursion(&g, &order);
        assert!(!issues.is_empty(), "indirect a->b->a left recursion must be detected: {issues:?}");
    }

    #[test]
    fn detects_left_recursion_through_nullable_prefix() {
        // a := opt a | "x" ; opt := "p"?   (opt is nullable, so `a` sits at the left edge)
        let mut g = HashMap::new();
        g.insert("a".into(), or(vec![seq(vec![rule_ref("opt"), rule_ref("a")]), token("string", "x")]));
        g.insert("opt".into(), ASTNode::Quantified { element: Box::new(token("string", "p")), quantifier: "?".into() });
        let order: Vec<String> = vec!["a".into(), "opt".into()];
        let issues = detect_left_recursion(&g, &order);
        assert!(
            issues.iter().any(|i| matches!(i, WellformednessIssue::LeftRecursive { rule, .. } if rule == "a")),
            "left recursion through a nullable prefix must be detected: {issues:?}"
        );
    }

    #[test]
    fn no_false_positive_on_well_formed_grammar() {
        // The iterative idiom PGEN uses: expr := term (op term)* ; term := "n" ; op := "+"
        let mut g = HashMap::new();
        g.insert(
            "expr".into(),
            seq(vec![
                rule_ref("term"),
                ASTNode::Quantified {
                    element: Box::new(seq(vec![rule_ref("op"), rule_ref("term")])),
                    quantifier: "*".into(),
                },
            ]),
        );
        g.insert("term".into(), token("string", "n"));
        g.insert("op".into(), token("op", "+"));
        let order: Vec<String> = vec!["expr".into(), "term".into(), "op".into()];
        let issues = detect_left_recursion(&g, &order);
        assert!(issues.is_empty(), "the iterative idiom must NOT be flagged: {issues:?}");
    }

    #[test]
    fn non_left_recursive_chain_is_clean() {
        // a := "k" b ; b := "m" a   (each consumes before recursing → not left-recursive)
        let mut g = HashMap::new();
        g.insert("a".into(), seq(vec![token("string", "k"), rule_ref("b")]));
        g.insert("b".into(), seq(vec![token("string", "m"), rule_ref("a")]));
        let order: Vec<String> = vec!["a".into(), "b".into()];
        let issues = detect_left_recursion(&g, &order);
        assert!(issues.is_empty(), "a consuming prefix breaks left recursion: {issues:?}");
    }

    #[test]
    fn detects_nonterminating_rule_but_not_left_recursion_with_base() {
        // bad := bad "z"            → no terminating alternative → NON-TERMINATING.
        // lr  := lr "+" t | t ; t := "n"   → left-recursive BUT has a base alt → finite
        //                                     min via `t` → NOT reported as non-terminating.
        let mut g = HashMap::new();
        g.insert("bad".into(), seq(vec![rule_ref("bad"), token("string", "z")]));
        g.insert(
            "lr".into(),
            or(vec![seq(vec![rule_ref("lr"), token("op", "+"), rule_ref("t")]), rule_ref("t")]),
        );
        g.insert("t".into(), token("string", "n"));
        let order: Vec<String> = vec!["bad".into(), "lr".into(), "t".into()];
        let nonterm = detect_nonterminating_rules(&g, &order);
        assert!(
            nonterm.iter().any(|i| matches!(i, WellformednessIssue::NonTerminating { rule } if rule == "bad")),
            "a rule with no terminating alternative must be flagged: {nonterm:?}"
        );
        assert!(
            !nonterm.iter().any(|i| matches!(i, WellformednessIssue::NonTerminating { rule } if rule == "lr" || rule == "t")),
            "a left-recursive rule WITH a base alternative is NOT non-terminating: {nonterm:?}"
        );
    }

    #[test]
    fn reference_to_undefined_include_rule_is_not_nonterminating() {
        // Mirrors ebnf.ebnf's `annotation_list := semantic_annotation+` where
        // semantic_annotation is defined via include(...) — i.e. NOT in this grammar.
        // It must NOT be flagged non-terminating (the include resolves it elsewhere).
        let mut g = HashMap::new();
        g.insert(
            "annotation_list".into(),
            ASTNode::Quantified {
                element: Box::new(rule_ref("semantic_annotation")), // undefined here
                quantifier: "+".into(),
            },
        );
        let order: Vec<String> = vec!["annotation_list".into()];
        let nonterm = detect_nonterminating_rules(&g, &order);
        assert!(
            nonterm.is_empty(),
            "a rule referencing an undefined (included) rule must NOT be flagged non-terminating: {nonterm:?}"
        );
    }

    // ---- PARSE-SOTA.9 / A2: ordered-choice shadowing lint ----

    #[test]
    fn detects_duplicate_alternative() {
        // r := "a" | "a"   → the 2nd alternative is a dead duplicate.
        let mut g = HashMap::new();
        g.insert("r".into(), or(vec![token("string", "a"), token("string", "a")]));
        let order: Vec<String> = vec!["r".into()];
        let issues = detect_ordered_choice_shadowing(&g, &order);
        assert_eq!(issues.len(), 1, "exactly one shadowed alt: {issues:?}");
        assert_eq!(issues[0].shadowed_index, 1);
        assert_eq!(issues[0].by_index, 0);
        assert_eq!(issues[0].reason, ShadowingReason::DuplicateAlternative);
    }

    #[test]
    fn detects_fixed_terminal_prefix_shadowing() {
        // r := "a" | "a" "b"   → "a" commits first, so `a b` is unreachable (the a|ab quirk).
        let mut g = HashMap::new();
        g.insert(
            "r".into(),
            or(vec![token("string", "a"), seq(vec![token("string", "a"), token("string", "b")])]),
        );
        let order: Vec<String> = vec!["r".into()];
        let issues = detect_ordered_choice_shadowing(&g, &order);
        assert_eq!(issues.len(), 1, "the `a | ab` quirk must flag `ab`: {issues:?}");
        assert_eq!(issues[0].shadowed_index, 1);
        assert_eq!(issues[0].reason, ShadowingReason::FixedTerminalPrefix);
    }

    #[test]
    fn reachability_witness_verifies_parses_and_covers_else_rejects() {
        // GRAMMAR-WELLFORMED.G.3: a witness holds iff its input parses AND exercises the fragment.
        // Mock parse_and_cover: input "ab" parses and covers {entry, branch_a}; anything else fails.
        let parse_and_cover = |input: &str| -> (bool, std::collections::HashSet<String>) {
            if input == "ab" {
                (true, ["entry".to_string(), "branch_a".to_string()].into_iter().collect())
            } else {
                (false, std::collections::HashSet::new())
            }
        };
        // valid witness: "ab" exercises branch_a.
        let good = ReachabilityWitness { fragment: "branch_a".into(), input: "ab".into() };
        assert!(verify_reachability_witness(parse_and_cover, &good).is_ok());
        // rejected: input does not parse.
        let no_parse = ReachabilityWitness { fragment: "branch_a".into(), input: "zz".into() };
        assert!(verify_reachability_witness(parse_and_cover, &no_parse).is_err());
        // rejected: parses but does not exercise the claimed fragment.
        let wrong_fragment = ReachabilityWitness { fragment: "branch_b".into(), input: "ab".into() };
        assert!(verify_reachability_witness(parse_and_cover, &wrong_fragment).is_err());
    }

    #[test]
    fn unreachable_rule_certificate_verifies_and_rejects() {
        // GRAMMAR-WELLFORMED.G.2: a rule-level unreachability certificate re-verifies; a bogus one
        // (a reachable rule claimed unreachable) is REJECTED.
        // entry -> keep (reachable). island <-> other reference each other but nothing reachable
        // references them and neither is unreferenced (so neither is a root) -> dead island.
        let mut g = HashMap::new();
        g.insert("entry".into(), rule_ref("keep"));
        g.insert("keep".into(), token("string", "k"));
        g.insert("island".into(), rule_ref("other"));
        g.insert("other".into(), rule_ref("island"));
        let order: Vec<String> =
            vec!["entry".into(), "keep".into(), "island".into(), "other".into()];
        let issues = detect_unreachable_rules(&g, &order);
        assert!(
            issues.iter().any(|i| matches!(i, WellformednessIssue::UnreachableRule { rule } if rule == "island")),
            "island must be flagged unreachable: {issues:?}"
        );
        let cert = WellformednessCertificate::UnreachableRule { rule: "island".into() };
        assert!(
            verify_wellformedness_certificate(&g, &order, None, &cert).is_ok(),
            "valid unreachable-rule certificate must verify"
        );
        let bogus = WellformednessCertificate::UnreachableRule { rule: "keep".into() };
        assert!(
            verify_wellformedness_certificate(&g, &order, None, &bogus).is_err(),
            "claiming the reachable rule 'keep' unreachable must be rejected"
        );
        // the generalized checker also dispatches DeadAlternative correctly.
        let mut g2 = HashMap::new();
        g2.insert("o".into(), or(vec![quant(token("string", "x"), "?"), token("string", "y")]));
        let order2: Vec<String> = vec!["o".into()];
        let sh = detect_ordered_choice_shadowing(&g2, &order2);
        let wrapped = WellformednessCertificate::DeadAlternative(sh[0].certificate());
        assert!(verify_wellformedness_certificate(&g2, &order2, None, &wrapped).is_ok());
    }

    #[test]
    fn unbound_fact_kind_certificate_verifies_and_rejects() {
        // GRAMMAR-WELLFORMED.G.2.1: an unbound-fact certificate re-verifies; a bogus one (a kind that
        // IS emitted, claimed unbound) is REJECTED.
        let mut ann = Annotations::default();
        ann.semantic_annotations.insert("producer".into(), vec![emit_fact_ann("type_name")]);
        ann.semantic_annotations
            .insert("bad".into(), vec![predicate_ann("has_fact(nonexistent_kind, head)")]);
        let order: Vec<String> = vec![];
        let g: HashMap<String, ASTNode> = HashMap::new();
        let cert = WellformednessCertificate::UnboundFactKind {
            rule: "bad".into(),
            kind: "nonexistent_kind".into(),
        };
        assert!(
            verify_wellformedness_certificate(&g, &order, Some(&ann), &cert).is_ok(),
            "valid unbound-fact certificate must verify"
        );
        // bogus: 'type_name' IS emitted -> claiming it unbound must be rejected.
        let bogus = WellformednessCertificate::UnboundFactKind {
            rule: "x".into(),
            kind: "type_name".into(),
        };
        assert!(
            verify_wellformedness_certificate(&g, &order, Some(&ann), &bogus).is_err(),
            "claiming an emitted kind unbound must be rejected"
        );
        // without annotations the checker honestly refuses.
        assert!(verify_wellformedness_certificate(&g, &order, None, &cert).is_err());
    }

    #[test]
    fn unreachability_certificates_verify_and_reject_tampering() {
        // GRAMMAR-WELLFORMED.G.1: every dead-verdict certificate must independently re-verify, and a
        // tampered/bogus certificate must be REJECTED by the checker (the trust comes from the checker).
        //   r := "a" | "a"           exact duplicate
        //   p := "a" | "a" "b"       fixed-terminal prefix
        //   o := "x"? | "y"          earlier-always-succeeds
        let mut g = HashMap::new();
        g.insert("r".into(), or(vec![token("string", "a"), token("string", "a")]));
        g.insert(
            "p".into(),
            or(vec![token("string", "a"), seq(vec![token("string", "a"), token("string", "b")])]),
        );
        g.insert("o".into(), or(vec![quant(token("string", "x"), "?"), token("string", "y")]));
        let order: Vec<String> = vec!["r".into(), "p".into(), "o".into()];
        let issues = detect_ordered_choice_shadowing(&g, &order);
        assert!(!issues.is_empty(), "expected shadowing findings");
        // (1) every real certificate independently re-verifies.
        for iss in &issues {
            let cert = iss.certificate();
            assert!(
                verify_unreachability_certificate(&g, &order, &cert).is_ok(),
                "valid certificate must verify: {cert:?} -> {:?}",
                verify_unreachability_certificate(&g, &order, &cert)
            );
        }
        // (2) tamper: point the dead alternative at the shadower itself (by not < dead) -> rejected.
        let mut tampered = issues.iter().find(|i| i.rule == "o").unwrap().certificate();
        tampered.dead_index = 0;
        assert!(
            verify_unreachability_certificate(&g, &order, &tampered).is_err(),
            "tampered certificate (dead_index == shadower) must be rejected"
        );
        // (3) bogus: claim an exact-duplicate relation where the alternatives are NOT identical.
        let bogus = UnreachabilityCertificate {
            rule: "p".into(),
            node_path: "root".into(),
            dead_index: 1,
            reason: UnreachabilityReason::DuplicateOf { by: 0 },
        };
        assert!(
            verify_unreachability_certificate(&g, &order, &bogus).is_err(),
            "bogus duplicate claim must be rejected (the alternatives are not identical)"
        );
        // (4) a path that does not resolve is rejected.
        let bad_path = UnreachabilityCertificate {
            rule: "o".into(),
            node_path: "root/s9".into(),
            dead_index: 1,
            reason: UnreachabilityReason::EarlierArmAlwaysSucceeds { by: 0 },
        };
        assert!(
            verify_unreachability_certificate(&g, &order, &bad_path).is_err(),
            "unresolvable path must be rejected"
        );
    }

    #[test]
    fn detects_always_succeeds_branch_shadowing() {
        // GRAMMAR-WELLFORMED.A2 (sound subset): an earlier alternative that ALWAYS SUCCEEDS
        // makes every later one unreachable (PEG commits to the first success).
        //   opt    := "x"? | "y"           → `"x"?` always succeeds → `"y"` dead
        //   star   := "z"* | "w"           → `"z"*` always succeeds → `"w"` dead
        //   allopt := ("a"? "b"?) | "c"    → an all-optional sequence always succeeds → `"c"` dead
        //   nref   := nullable_rule | "d"  → ref to an always-succeeding rule → `"d"` dead
        let mut g = HashMap::new();
        g.insert("opt".into(), or(vec![quant(token("string", "x"), "?"), token("string", "y")]));
        g.insert("star".into(), or(vec![quant(token("string", "z"), "*"), token("string", "w")]));
        g.insert(
            "allopt".into(),
            or(vec![
                seq(vec![quant(token("string", "a"), "?"), quant(token("string", "b"), "?")]),
                token("string", "c"),
            ]),
        );
        g.insert("nref".into(), or(vec![rule_ref("nullable_rule"), token("string", "d")]));
        g.insert("nullable_rule".into(), quant(token("string", "n"), "?"));
        let order: Vec<String> = vec![
            "opt".into(),
            "star".into(),
            "allopt".into(),
            "nref".into(),
            "nullable_rule".into(),
        ];
        let issues = detect_ordered_choice_shadowing(&g, &order);
        for rule in ["opt", "star", "allopt", "nref"] {
            assert!(
                issues.iter().any(|i| i.rule == rule
                    && i.shadowed_index == 1
                    && i.reason == ShadowingReason::EarlierAlwaysMatches),
                "rule '{rule}' second branch must be flagged EarlierAlwaysMatches: {issues:?}"
            );
        }
    }

    #[test]
    fn lookahead_earlier_branch_does_not_shadow() {
        // A syntactic predicate is nullable (consumes nothing) but CAN FAIL — so it does NOT
        // always succeed and must NOT shadow a later branch (the soundness line that separates
        // `always_succeeds` from `nullable`).
        //   pos := &"a" | "b"     (&"a" fails on input not starting "a" → "b" reachable)
        //   neg := !"a" | "a"     (!"a" fails on input starting "a" → "a" reachable)
        let mut g = HashMap::new();
        g.insert("pos".into(), or(vec![look(token("string", "a"), true), token("string", "b")]));
        g.insert("neg".into(), or(vec![look(token("string", "a"), false), token("string", "a")]));
        let order: Vec<String> = vec!["pos".into(), "neg".into()];
        let issues = detect_ordered_choice_shadowing(&g, &order);
        assert!(
            issues.is_empty(),
            "a lookahead earlier branch can fail, so it must not shadow a later branch: {issues:?}"
        );
    }

    #[test]
    fn no_false_positive_distinct_or_longer_first() {
        // (a) distinct terminals; (b) longer-before-shorter (`ab | a`: `a` IS reachable on
        // input "a" alone, so NOT shadowed); (c) rule-reference alternatives (a rule can
        // fail, so it is unsound to treat as a guaranteed prefix).
        let mut g = HashMap::new();
        g.insert("distinct".into(), or(vec![token("string", "a"), token("string", "b")]));
        g.insert(
            "longer_first".into(),
            or(vec![seq(vec![token("string", "a"), token("string", "b")]), token("string", "a")]),
        );
        g.insert("rule_alts".into(), or(vec![rule_ref("x"), rule_ref("y")]));
        g.insert("x".into(), token("string", "x"));
        g.insert("y".into(), token("string", "y"));
        let order: Vec<String> = vec![
            "distinct".into(),
            "longer_first".into(),
            "rule_alts".into(),
            "x".into(),
            "y".into(),
        ];
        let issues = detect_ordered_choice_shadowing(&g, &order);
        assert!(
            issues.is_empty(),
            "no false positives on distinct/longer-first/rule-ref alternatives: {issues:?}"
        );
    }

    /// PARSE-SOTA.8.1 prerequisite — verify the analyses on a REAL shipped grammar:
    /// NO left-recursion (a hard error if found — would block wiring the reject) and
    /// REPORT shadowing findings for review. Env-gated (skips when the compiled-grammar
    /// artifact is absent, e.g. fresh CI). Run with:
    ///   PGEN_WELLFORMEDNESS_GEN_AST=<…/systemverilog_gen_ast.json> cargo test --lib \
    ///     shipped_grammar_is_well_formed -- --nocapture
    #[test]
    fn detects_nullable_repetition() {
        // r := nullable_body*     (unbounded `*` over a nullable body -> WARN)
        // nullable_body := "x"?   (nullable: min 0)
        let mut g = HashMap::new();
        g.insert(
            "r".to_string(),
            ASTNode::Quantified {
                element: Box::new(rule_ref("nullable_body")),
                quantifier: "*".to_string(),
            },
        );
        g.insert(
            "nullable_body".to_string(),
            ASTNode::Quantified {
                element: Box::new(token("string", "x")),
                quantifier: "?".to_string(),
            },
        );
        let order = vec!["r".to_string(), "nullable_body".to_string()];
        let issues = detect_nullable_repetition(&g, &order);
        assert_eq!(
            issues.len(),
            1,
            "exactly one nullable-repetition site (`r := nullable_body*`); got {:?}",
            issues
        );
        assert!(
            matches!(&issues[0], WellformednessIssue::NullableRepetition { rule, .. } if rule == "r"),
            "the flagged rule must be `r`; got {:?}",
            issues[0]
        );
    }

    #[test]
    fn no_false_positive_nonnullable_or_bounded_repetition() {
        // r := a*   (unbounded but `a` is NON-nullable -> fine)
        // s := a?   (bounded -> never flagged regardless of nullability)
        // a := "x"  (consumes input -> non-nullable)
        let mut g = HashMap::new();
        g.insert(
            "r".to_string(),
            ASTNode::Quantified {
                element: Box::new(rule_ref("a")),
                quantifier: "*".to_string(),
            },
        );
        g.insert(
            "s".to_string(),
            ASTNode::Quantified {
                element: Box::new(rule_ref("a")),
                quantifier: "?".to_string(),
            },
        );
        g.insert("a".to_string(), token("string", "x"));
        let order = vec!["r".to_string(), "s".to_string(), "a".to_string()];
        let issues = detect_nullable_repetition(&g, &order);
        assert!(
            issues.is_empty(),
            "non-nullable `a*` and bounded `a?` must NOT be flagged; got {:?}",
            issues
        );
    }

    #[test]
    fn shipped_grammar_is_well_formed() {
        let Ok(path) = std::env::var("PGEN_WELLFORMEDNESS_GEN_AST") else {
            eprintln!("skip: set PGEN_WELLFORMEDNESS_GEN_AST to a gen_ast.json to run");
            return;
        };
        let data = std::fs::read_to_string(&path).expect("read gen_ast.json");
        let v: serde_json::Value = serde_json::from_str(&data).expect("parse gen_ast.json");
        let grammar: HashMap<String, ASTNode> =
            serde_json::from_value(v["grammar_tree"].clone()).expect("grammar_tree");
        let rule_order: Vec<String> =
            serde_json::from_value(v["rule_order"].clone()).expect("rule_order");
        eprintln!("loaded grammar: {} rules", grammar.len());

        // Left recursion is INFORMATIONAL for PGEN (handled by LR elimination + the
        // runtime mutual-recursion handler) — report, do NOT fail. Verified on the SV
        // grammar: 28 left-recursive rules, all parse fine.
        let lr = detect_left_recursion(&grammar, &rule_order);
        eprintln!("left-recursive rules (informational — PGEN handles these): {}", lr.len());
        for i in lr.iter().take(5) {
            eprintln!("  {}", i.message());
        }

        // NON-TERMINATING rules are the genuine, reject-worthy defect — assert NONE.
        let nonterm = detect_nonterminating_rules(&grammar, &rule_order);
        for i in &nonterm {
            eprintln!("NON-TERMINATING: {}", i.message());
        }
        assert!(
            nonterm.is_empty(),
            "shipped grammar has {} non-terminating rule(s) — genuinely ill-formed",
            nonterm.len()
        );

        let shadow = detect_ordered_choice_shadowing(&grammar, &rule_order);
        eprintln!("shadowing findings: {} (soft — review for false positives)", shadow.len());
        for s in shadow.iter().take(15) {
            eprintln!("  {}", s.message());
        }
    }

    #[test]
    fn shadowing_reports_nested_or_node_path() {
        // r := "p" ("a" | "a")   → the dup is in a NESTED Or; path must point at it.
        let mut g = HashMap::new();
        g.insert(
            "r".into(),
            seq(vec![
                token("string", "p"),
                or(vec![token("string", "a"), token("string", "a")]),
            ]),
        );
        let order: Vec<String> = vec!["r".into()];
        let issues = detect_ordered_choice_shadowing(&g, &order);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].node_path, "root/s1", "path must locate the nested Or");
    }
}
