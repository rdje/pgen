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

/// Like `collect_node_rule_refs` but counts a reference as POSITIVE only — it does NOT descend into
/// `!`/`&` lookahead sub-expressions. A lookahead is a parser ASSERTION that consumes no input and
/// emits nothing, so a rule referenced ONLY inside a lookahead is never POSITIVELY entered during a
/// parse. The transactional witness primitive records positive rule ENTRY (not assertion), so such a
/// rule can never be witnessed — this collector is the building block for that distinction at the
/// proof layer (the same reach-honesty principle the stimuli reach search uses, lifted here to the
/// linter's reachability analysis). Used by `positively_reachable_rules` / `detect_lookahead_only_rules`.
fn collect_node_positive_rule_refs(node: &ASTNode, out: &mut HashSet<String>) {
    match node {
        ASTNode::Or { alternatives } => {
            for a in alternatives {
                collect_node_positive_rule_refs(a, out);
            }
        }
        ASTNode::Sequence { elements } => {
            for e in elements {
                collect_node_positive_rule_refs(e, out);
            }
        }
        ASTNode::Quantified { element, .. } => collect_node_positive_rule_refs(element, out),
        // The crux: a lookahead emits nothing positively — do NOT follow references inside it.
        ASTNode::Lookahead { .. } => {}
        ASTNode::Atom { value } => match value {
            ASTValue::Node(inner) => collect_node_positive_rule_refs(inner, out),
            ASTValue::Token(parts) => {
                if let Some(r) = referenced_rule(parts) {
                    out.insert(r.to_string());
                }
            }
        },
    }
}

/// GRAMMAR-WELLFORMED (lookahead-honesty at the proof layer): the set of rules POSITIVELY reachable
/// from the roots — reachable by a chain of POSITIVELY-EMITTED references, never descending into
/// `!`/`&` lookahead sub-expressions. Mirrors `reachable_rules` EXACTLY except the transitive
/// closure follows only positive edges (`collect_node_positive_rule_refs`); the ROOT set is computed
/// from references-ANYWHERE (`collect_node_rule_refs`) so a rule referenced only inside a lookahead
/// is NOT mistaken for an unreferenced top-level entry (which would make it a spurious positive
/// root). A rule that is in `reachable_rules` but NOT here is POSITIVELY-UNREACHABLE: structurally
/// referenced, but reachable only through a lookahead edge, so the parser never positively enters it
/// and the transactional witness primitive (which records positive entry) correctly never witnesses
/// it. Deterministic; references to undefined (external/include) rules are ignored.
pub fn positively_reachable_rules(
    grammar: &HashMap<String, ASTNode>,
    rule_order: &[String],
) -> HashSet<String> {
    let mut pos_refs_of: HashMap<&str, HashSet<String>> = HashMap::new();
    let mut referenced: HashSet<String> = HashSet::new();
    for (rule, body) in grammar {
        // ROOTS use references-ANYWHERE so a lookahead-only rule is NOT a spurious top-level entry.
        let mut all_refs = HashSet::new();
        collect_node_rule_refs(body, &mut all_refs);
        for r in &all_refs {
            if grammar.contains_key(r) {
                referenced.insert(r.clone());
            }
        }
        // The CLOSURE follows only POSITIVELY-emitted edges.
        let mut pos_refs = HashSet::new();
        collect_node_positive_rule_refs(body, &mut pos_refs);
        pos_refs.retain(|r| grammar.contains_key(r));
        pos_refs_of.insert(rule.as_str(), pos_refs);
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
        if let Some(refs) = pos_refs_of.get(rule.as_str()) {
            for r in refs {
                if reachable.insert(r.clone()) {
                    stack.push(r.clone());
                }
            }
        }
    }
    reachable
}

/// GRAMMAR-WELLFORMED (the lookahead-only PROOF detector): rules that are structurally REACHABLE
/// (`reachable_rules`) but POSITIVELY-UNREACHABLE (`positively_reachable_rules`) — i.e. reachable
/// only through `!`/`&` lookahead edges. Such a rule can NEVER be positively entered, so the
/// transactional coverage primitive can never witness it — and that non-witnessing is SOUND, a
/// verified PROOF (the certifying dual of the constructive WITNESS), NOT an attribution-rule UNKNOWN
/// ticket. The canonical case is a guard token used only in a negative lookahead (rtl_frontend's
/// `port_direction_token`, referenced only inside `( comma !port_direction_token port_item )*`).
/// Returns the rule names in `rule_order` order (deterministic). Parser-agnostic (keyed purely on
/// the `Lookahead` node shape, never a rule name).
pub fn detect_lookahead_only_rules(
    grammar: &HashMap<String, ASTNode>,
    rule_order: &[String],
) -> Vec<String> {
    let reachable = reachable_rules(grammar, rule_order);
    let positively = positively_reachable_rules(grammar, rule_order);
    rule_order
        .iter()
        .filter(|r| {
            grammar.contains_key(r.as_str())
                && reachable.contains(r.as_str())
                && !positively.contains(r.as_str())
        })
        .cloned()
        .collect()
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
    /// `rule` is structurally reachable but POSITIVELY-UNREACHABLE — reachable only through `!`/`&`
    /// lookahead edges, so the parser never positively enters it and the transactional witness
    /// primitive (which records positive entry) correctly never records it. The non-witnessing is a
    /// SOUND, decidable PROOF (re-derived as `reachable ∖ positively_reachable`), not a coverage gap.
    LookaheadOnlyRule { rule: String },
    /// VERILOG-2005-PROFILE.6.7 (P1): `rule` SURVIVES the `profile` filter but is NOT positively
    /// reachable from any entry in the DECLARED ENTRY UNIVERSE `entries` over the active
    /// (profile-filtered) tree with satisfiability-honest edges — no accepted parse from any declared
    /// entry under `profile` can positively enter it, so the transactional witness primitive can never
    /// record it. A SOUND, decidable PER-PROFILE PROOF (the profile-scoped generalization of
    /// `LookaheadOnlyRule`), re-derived from scratch by `verify_profile_certificate`. Verified only
    /// through the dedicated verifier (it needs the pre-filter rule set + the entry universe, which the
    /// generic `verify_wellformedness_certificate` signature does not carry).
    ProfileEntryUnreachable { rule: String, profile: String, entries: Vec<String> },
    /// VERILOG-2005-PROFILE.6.7 (P2): `rule` is P1-live under `profile` but dead under the
    /// profile-unproducible-store-gate FIXPOINT (`classify_profile_residual`) — its mandatory descent
    /// forces a positive store-gate (or a rule that forces one) on a fact-kind no P1-live rule can
    /// emit, with NO live `@import_from_library` degrading the analysis. `reason` is the machine
    /// attribution (`gate kind 'K' unproducible` / `mandatory descent forces 'R'` / `stranded by the
    /// store fixpoint`). A SOUND per-profile PROOF re-derived from scratch by `verify_profile_certificate`.
    ProfileUnproducibleGate { rule: String, profile: String, reason: String },
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
        WellformednessCertificate::LookaheadOnlyRule { rule } => {
            if !grammar.contains_key(rule) {
                return Err(format!("certificate cites unknown rule '{rule}'"));
            }
            // Re-derive BOTH reachability sets independently; the claim holds iff the rule is
            // structurally reachable but NOT positively reachable (reachable only via a `!`/`&` edge).
            let reachable = reachable_rules(grammar, rule_order).contains(rule);
            let positively = positively_reachable_rules(grammar, rule_order).contains(rule);
            if reachable && !positively {
                Ok(())
            } else {
                Err(format!(
                    "certificate claims rule '{rule}' is LOOKAHEAD-ONLY (positively-unreachable), but reachable={reachable} positively_reachable={positively} (not a lookahead-only rule)"
                ))
            }
        }
        WellformednessCertificate::ProfileEntryUnreachable { rule, .. }
        | WellformednessCertificate::ProfileUnproducibleGate { rule, .. } => Err(format!(
            "profile certificate for rule '{rule}' must be re-verified through \
             `verify_profile_certificate` (it needs the pre-filter rule set + the declared entry \
             universe, which this generic verifier does not carry)"
        )),
    }
}

/// VERILOG-2005-PROFILE.6.7 — THE CHECKER for the per-profile proof certificates
/// (`ProfileEntryUnreachable` / `ProfileUnproducibleGate`). Independently re-derives the claim from
/// the active (profile-filtered) tree + the pre-filter rule set (`full_defined`, so profile-PRUNED
/// vs external/include references are distinguished) + the declared entry universe (`entries`), via
/// the SAME pure `.6.6` analyses the detector uses — recomputing, never trusting a passed-in flag
/// (the `LookaheadOnlyRule` re-derivation pattern). Non-profile variants delegate to the generic
/// `verify_wellformedness_certificate`. `Ok(())` iff the certificate genuinely holds.
pub fn verify_profile_certificate(
    active: &HashMap<String, ASTNode>,
    rule_order: &[String],
    full_defined: &HashSet<String>,
    entries: &[String],
    annotations: Option<&Annotations>,
    cert: &WellformednessCertificate,
) -> Result<(), String> {
    match cert {
        WellformednessCertificate::ProfileEntryUnreachable {
            rule,
            profile: _,
            entries: cert_entries,
        } => {
            if !active.contains_key(rule) {
                return Err(format!(
                    "profile-entry-unreachable certificate cites rule '{rule}' absent from the active (profile-filtered) tree"
                ));
            }
            if cert_entries != entries {
                return Err(format!(
                    "profile-entry-unreachable certificate for '{rule}' carries entry universe {cert_entries:?} but the run's universe is {entries:?}"
                ));
            }
            // Re-derive P1 (positive reachability from the declared entry universe, no store-dead
            // set) independently; the claim holds iff `rule` is NOT positively reachable.
            let live0 =
                profile_entry_positively_live(active, rule_order, full_defined, entries, &HashSet::new());
            if live0.contains(rule) {
                Err(format!(
                    "certificate claims '{rule}' is PROFILE-ENTRY-UNREACHABLE, but it IS positively reachable from the declared entry universe"
                ))
            } else {
                Ok(())
            }
        }
        WellformednessCertificate::ProfileUnproducibleGate { rule, profile: _, reason: _ } => {
            if !active.contains_key(rule) {
                return Err(format!(
                    "profile-unproducible-gate certificate cites rule '{rule}' absent from the active (profile-filtered) tree"
                ));
            }
            // Re-derive the P1+P2 store fixpoint from scratch (grammar-global; the `unknown` argument
            // only selects which rules are bucketed, so classifying just `[rule]` yields the same
            // verdict) and confirm `rule` lands in `store_unproducible`.
            let classification = classify_profile_residual(
                active,
                rule_order,
                full_defined,
                entries,
                annotations,
                std::slice::from_ref(rule),
            );
            if classification.store_unproducible.iter().any(|(r, _)| r == rule) {
                Ok(())
            } else {
                Err(format!(
                    "certificate claims '{rule}' is STORE-UNPRODUCIBLE under the profile, but the re-derived classification does not place it there (degraded_inert={})",
                    classification.degraded_inert
                ))
            }
        }
        other => verify_wellformedness_certificate(active, rule_order, annotations, other),
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

/// GRAMMAR-WELLFORMED.G.3.3: the rule names PRESENT in a parse tree — a sound AST walk over the
/// STRUCTURAL `ParseNode` content.
///
/// ⚠️ NOT a valid parse-coverage source for ANNOTATED grammars (G.4.6 finding). A rule carrying a
/// `-> {…}` return annotation folds its whole subtree into `ParseContent::Json`, which has no child
/// `ParseNode`s — so this walk stops at the first annotated rule and never sees the rules folded
/// beneath it. On heavily-annotated grammars (e.g. SystemVerilog) it collapses to ~one rule. The
/// production witness-coverage path is the PARSER'S OWN transactional record
/// (`enable_coverage` + `exercised_rule_names` on the generated parser), exposed grammar-agnostically
/// via `parser_registry::parse_and_cover`. This walk is retained only for structural (annotation-free)
/// trees in unit tests, where it is exact.
pub fn parse_node_covered_rules(node: &super::ParseNode<'_>) -> HashSet<String> {
    let mut out = HashSet::new();
    collect_covered_rules(node, &mut out);
    out
}

fn collect_covered_rules(node: &super::ParseNode<'_>, out: &mut HashSet<String>) {
    out.insert(node.rule_name.to_string());
    match &node.content {
        super::ParseContent::Sequence(children) | super::ParseContent::Quantified(children, _) => {
            for c in children {
                collect_covered_rules(c, out);
            }
        }
        super::ParseContent::Alternative(child) => collect_covered_rules(child, out),
        super::ParseContent::Terminal(_)
        | super::ParseContent::TransformedTerminal(_)
        | super::ParseContent::Json(_) => {}
    }
}

/// GRAMMAR-WELLFORMED.G.4: the certificate-COVERAGE report — the capstone that unifies the two
/// duality sides. For every grammar fragment, is it covered by a verified unreachability PROOF (the
/// linter proved it dead) or a verified reachability WITNESS (the generator demonstrated it
/// reachable)? `unknown` = fragments with NEITHER — the attribution-rule TICKETS, never silently
/// accepted. The caller supplies the ALREADY-VERIFIED proof/witness fragment sets (each fragment in
/// them passed its independent checker — `verify_wellformedness_certificate` /
/// `verify_reachability_witness`). PURE + deterministic (iterates `all_fragments`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CertificateCoverageReport {
    pub total: usize,
    pub covered_by_proof: Vec<String>,
    pub covered_by_witness: Vec<String>,
    /// Fragments with no verified proof AND no verified witness — the UNKNOWN tickets.
    pub unknown: Vec<String>,
}

impl CertificateCoverageReport {
    /// FULLY CERTIFIED ⟺ no UNKNOWN fragment — every fragment carries a verified PROOF or WITNESS.
    /// This is the objective "the linter is trustworthy on this grammar" number (`unknown.len()==0`).
    pub fn is_fully_certified(&self) -> bool {
        self.unknown.is_empty()
    }
}

pub fn certificate_coverage(
    all_fragments: &[String],
    proof_covered: &HashSet<String>,
    witness_covered: &HashSet<String>,
) -> CertificateCoverageReport {
    let mut covered_by_proof = Vec::new();
    let mut covered_by_witness = Vec::new();
    let mut unknown = Vec::new();
    for f in all_fragments {
        if proof_covered.contains(f) {
            covered_by_proof.push(f.clone());
        } else if witness_covered.contains(f) {
            covered_by_witness.push(f.clone());
        } else {
            unknown.push(f.clone());
        }
    }
    CertificateCoverageReport {
        total: all_fragments.len(),
        covered_by_proof,
        covered_by_witness,
        unknown,
    }
}

/// GRAMMAR-WELLFORMED.G.4.2 (proof gathering): the set of rules covered by a VERIFIED whole-rule
/// PROOF — each flagged by a detector AND independently re-confirmed by
/// `verify_wellformedness_certificate`. Two sound, disjoint whole-rule proof classes:
///   1. **Structurally unreachable** (`detect_unreachable_rules`) — not reachable from any root at
///      all (a genuinely dead rule, Hopcroft–Ullman "useless symbol").
///   2. **Lookahead-only / positively-unreachable** (`detect_lookahead_only_rules`) — structurally
///      reachable but reachable only through `!`/`&` lookahead edges, so it is never positively
///      entered and the transactional witness primitive can never record it. The non-witnessing is
///      SOUND (a proof), not an attribution-rule UNKNOWN ticket; the reach pass independently flags
///      such rules "dead-rule candidate — adjudicate via the linter", and this IS that adjudication.
/// (Rule-level: shadowing/orphan/unbound proofs concern branches/predicates WITHIN a reachable rule,
/// not whole-rule deadness, so they do not make a RULE proof-covered.) A detector finding whose
/// certificate fails to re-verify is a LINTER BUG — returned in `failures`, never silently covered.
pub fn gather_verified_proof_covered_rules(
    grammar: &HashMap<String, ASTNode>,
    rule_order: &[String],
) -> (HashSet<String>, Vec<String>) {
    let mut covered = HashSet::new();
    let mut failures = Vec::new();
    for issue in detect_unreachable_rules(grammar, rule_order) {
        if let WellformednessIssue::UnreachableRule { rule } = issue {
            let cert = WellformednessCertificate::UnreachableRule { rule: rule.clone() };
            match verify_wellformedness_certificate(grammar, rule_order, None, &cert) {
                Ok(()) => {
                    covered.insert(rule);
                }
                Err(e) => failures.push(format!("unreachable-rule proof for '{rule}' failed re-verify: {e}")),
            }
        }
    }
    // Lookahead-only / positively-unreachable rules (disjoint from the structurally-unreachable set
    // above — those are not reachable at all; these are reachable, only via a lookahead edge).
    for rule in detect_lookahead_only_rules(grammar, rule_order) {
        let cert = WellformednessCertificate::LookaheadOnlyRule { rule: rule.clone() };
        match verify_wellformedness_certificate(grammar, rule_order, None, &cert) {
            Ok(()) => {
                covered.insert(rule);
            }
            Err(e) => {
                failures.push(format!("lookahead-only proof for '{rule}' failed re-verify: {e}"))
            }
        }
    }
    (covered, failures)
}

/// GRAMMAR-WELLFORMED.G.4.2 (witness gathering): the set of fragments covered by a VERIFIED
/// reachability witness — each witness replayed through the real parser (`parse_and_cover`) and
/// confirmed to parse AND exercise its fragment. A witness that fails to re-verify (doesn't parse, or
/// parses but misses its fragment) is a GENERATOR/witness bug — returned in `failures`, NOT counted.
/// Parser-AGNOSTIC (the closure carries the grammar's parser) → works for every grammar (Phase H).
pub fn gather_verified_witness_covered<F>(
    witnesses: &[ReachabilityWitness],
    parse_and_cover: F,
) -> (HashSet<String>, Vec<String>)
where
    F: Fn(&str) -> (bool, HashSet<String>),
{
    let mut covered = HashSet::new();
    let mut failures = Vec::new();
    for w in witnesses {
        match verify_reachability_witness(&parse_and_cover, w) {
            Ok(()) => {
                covered.insert(w.fragment.clone());
            }
            Err(e) => failures.push(e),
        }
    }
    (covered, failures)
}

// ============================================================================================
// VERILOG-2005-PROFILE.6.6 — per-profile residual classification (READ-ONLY; staged toward the
// `.6.7` per-profile `proof` promotion).
//
// Under a dialect profile the cert-coverage residual contains rules that are UNKNOWN not because
// anything is missing, but BECAUSE the profile excludes them — and today's whole-rule proofs
// cannot say so: `apply_grammar_profile_filter` prunes only rules whose OWN `@profiles` tag
// excludes the profile, so an untagged rule stranded by the pruning of every referencing rule
// SURVIVES, and `reachable_rules`' unreferenced→secondary-root promotion then makes it its own
// root (never "unreachable" → no proof → UNKNOWN). Two pure analyses close that gap:
//
//   P1 — profile-entry-universe unreachability. Over the ACTIVE (profile-filtered) tree, the
//   positively-reachable set from the DECLARED ENTRY UNIVERSE (the cert entry + every
//   `--cert-union-config` entry present in the active tree — NEVER the unreferenced-root
//   heuristic, which is exactly what a stranded rule defeats), with SATISFIABILITY-HONEST edges:
//   a reference site contributes an edge only if its enclosing derivation within the referencing
//   rule is satisfiable over the active tree (Or = any alternative, Sequence = all elements,
//   min-0 quantifier = skippable, lookahead = no positive edge, and — the `mandatory_node_gated`
//   missing-rule half lifted to the proof layer — a reference to a rule PRUNED from the active
//   tree is unsatisfiable, while a reference to a rule defined in NO tree stays external/⊤).
//   A rule outside that set can never be positively entered by a parse from any declared entry.
//
//   P2 — profile-unproducible mandatory store-gate, a fixpoint composed with P1. Fact-kind K is
//   producible iff SOME P1-live rule carries an `@emit_fact` of kind K; a live rule whose
//   RULE-LEVEL `@predicate` REQUIRES a positive fact-query (`has_fact` / `fact_attribute_equals`
//   / `fact_count_at_least` with a literal count ≥ 1 — NEVER `lacks_fact`/negations: an
//   unsatisfiable NEGATIVE gate makes a rule always-live, not dead) on an unproducible K is dead.
//   Deadness then CASCADES through the satisfiability composition itself (a mandatory reference
//   to a dead rule makes the referencer unsatisfiable — the `mandatory_node_gated` algebra is the
//   satisfiability algebra's dual), and a dead rule's emissions vanish, so the analysis iterates
//   P1+P2 to a fixpoint; the mandatory-descent walk is used at CLASSIFICATION time to attribute
//   each cascade casualty to its forcing rule. Sound-core boundaries: KIND-level only
//   (attribute-level refinement deliberately out); branch-level predicates are ignored (a branch
//   gate kills only its branch — flattening it rule-wide would falsely brand escape-carrying
//   rules dead); and if ANY P1-live rule carries `@import_from_library` the store analysis is
//   DEGRADED TO INERT (external artifacts can inject facts without an in-parse emitter, and the
//   artifact contents are not statically known — so no `store_unproducible` claim is safe).
//
// Both analyses are PURE, deterministic, parser-agnostic, and independently re-derivable — the
// `.6.7` certificate variants re-derive them from scratch. Design record: the `.6.5` Findings in
// `docs/tasks/VERILOG-2005-PROFILE.md`.
// ============================================================================================

/// The read-only classification of a profile run's residual `UNKNOWN` set (printed by the cert
/// report only under `PGEN_CERT_RESIDUAL_CLASSIFICATION=1`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileResidualClassification {
    /// The declared entry universe the analysis quantified over.
    pub entries: Vec<String>,
    /// `true` iff some P1-live rule carries `@import_from_library` — P2 then makes NO
    /// `store_unproducible` claims (P1 entry-unreachability is unaffected: reachability is
    /// structural, facts cannot re-wire references).
    pub degraded_inert: bool,
    /// Residual rules NOT positively reachable from any declared entry (P1).
    pub profile_entry_unreachable: Vec<String>,
    /// Residual rules that are P1-live but dead under the store fixpoint (P2): `(rule, reason)`.
    pub store_unproducible: Vec<(String, String)>,
    /// Residual rules the analysis cannot prove dead — the honest remainder.
    pub genuine: Vec<String>,
}

/// Is `node` satisfiable over the ACTIVE (profile-filtered) tree, given the current per-rule
/// estimates (fixpoint) and the store-dead set? Same composition algebra as `node_satisfiable`,
/// with the profile question already resolved by the filter: a reference to a rule missing from
/// the active tree is PRUNED = ⊥ when the full (pre-filter) tree defines it, and external/include
/// = ⊤ when nothing defines it (never false-accuse).
fn node_satisfiable_active(
    node: &ASTNode,
    sat: &HashMap<String, bool>,
    active: &HashMap<String, ASTNode>,
    full_defined: &HashSet<String>,
    dead: &HashSet<String>,
) -> bool {
    match node {
        ASTNode::Or { alternatives } => alternatives
            .iter()
            .any(|a| node_satisfiable_active(a, sat, active, full_defined, dead)),
        ASTNode::Sequence { elements } => elements
            .iter()
            .all(|e| node_satisfiable_active(e, sat, active, full_defined, dead)),
        ASTNode::Quantified { element, quantifier } => {
            let (min, _) = parse_quantifier_bounds(quantifier).unwrap_or((0, None));
            min == 0 || node_satisfiable_active(element, sat, active, full_defined, dead)
        }
        ASTNode::Lookahead { .. } => true,
        ASTNode::Atom { value } => match value {
            ASTValue::Node(inner) => node_satisfiable_active(inner, sat, active, full_defined, dead),
            ASTValue::Token(parts) => match referenced_rule(parts) {
                Some(r) if !active.contains_key(r) => !full_defined.contains(r),
                Some(r) => !dead.contains(r) && sat.get(r).copied().unwrap_or(false),
                None => true,
            },
        },
    }
}

/// The satisfiability fixpoint over the active tree (the `compute_sat_by_profile` shape, with the
/// profile dimension already resolved by the filter and the store-dead set treated as ⊥).
fn active_tree_satisfiability(
    active: &HashMap<String, ASTNode>,
    rule_order: &[String],
    full_defined: &HashSet<String>,
    dead: &HashSet<String>,
) -> HashMap<String, bool> {
    let mut sat: HashMap<String, bool> = HashMap::new();
    loop {
        let mut changed = false;
        for rule in rule_order {
            if dead.contains(rule) {
                continue;
            }
            let Some(body) = active.get(rule) else { continue };
            let value = node_satisfiable_active(body, &sat, active, full_defined, dead);
            if sat.get(rule).copied().unwrap_or(false) != value {
                sat.insert(rule.clone(), value);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    sat
}

/// Collect the POSITIVE, SATISFIABLE-context rule-reference edges of `node`: a reference
/// contributes only when some derivation of the enclosing structure can positively render it —
/// an unsatisfiable Or-alternative contributes nothing, a Sequence with ANY unsatisfiable element
/// contributes nothing (the whole sequence can never derive), a quantifier body contributes only
/// if the body itself is satisfiable (rendering it at least once is the only way it emits), and a
/// lookahead contributes nothing (an assertion renders no input). The edge target must itself be
/// active, satisfiable, and not store-dead — an accepted parse can never contain a rule that
/// cannot complete.
fn collect_satisfiable_positive_edges(
    node: &ASTNode,
    sat: &HashMap<String, bool>,
    active: &HashMap<String, ASTNode>,
    full_defined: &HashSet<String>,
    dead: &HashSet<String>,
    out: &mut HashSet<String>,
) {
    match node {
        ASTNode::Or { alternatives } => {
            for a in alternatives {
                if node_satisfiable_active(a, sat, active, full_defined, dead) {
                    collect_satisfiable_positive_edges(a, sat, active, full_defined, dead, out);
                }
            }
        }
        ASTNode::Sequence { elements } => {
            if elements
                .iter()
                .all(|e| node_satisfiable_active(e, sat, active, full_defined, dead))
            {
                for e in elements {
                    collect_satisfiable_positive_edges(e, sat, active, full_defined, dead, out);
                }
            }
        }
        ASTNode::Quantified { element, .. } => {
            if node_satisfiable_active(element, sat, active, full_defined, dead) {
                collect_satisfiable_positive_edges(element, sat, active, full_defined, dead, out);
            }
        }
        ASTNode::Lookahead { .. } => {}
        ASTNode::Atom { value } => match value {
            ASTValue::Node(inner) => {
                collect_satisfiable_positive_edges(inner, sat, active, full_defined, dead, out)
            }
            ASTValue::Token(parts) => {
                if let Some(r) = referenced_rule(parts) {
                    if active.contains_key(r)
                        && !dead.contains(r)
                        && sat.get(r).copied().unwrap_or(false)
                    {
                        out.insert(r.to_string());
                    }
                }
            }
        },
    }
}

/// P1: the set of rules POSITIVELY reachable from the DECLARED ENTRY UNIVERSE over the active
/// (profile-filtered) tree with satisfiability-honest edges, with `dead` rules treated as removed
/// (⊥ satisfiability, no edges in or out). Entries not present in the active tree (or themselves
/// unsatisfiable/dead) seed nothing. PURE; deterministic (set membership only — callers order
/// output by `rule_order`/input order).
pub fn profile_entry_positively_live(
    active: &HashMap<String, ASTNode>,
    rule_order: &[String],
    full_defined: &HashSet<String>,
    entries: &[String],
    dead: &HashSet<String>,
) -> HashSet<String> {
    let sat = active_tree_satisfiability(active, rule_order, full_defined, dead);
    let mut live: HashSet<String> = HashSet::new();
    let mut stack: Vec<String> = Vec::new();
    for entry in entries {
        if active.contains_key(entry)
            && !dead.contains(entry)
            && sat.get(entry).copied().unwrap_or(false)
            && live.insert(entry.clone())
        {
            stack.push(entry.clone());
        }
    }
    while let Some(rule) = stack.pop() {
        let Some(body) = active.get(&rule) else { continue };
        let mut edges = HashSet::new();
        collect_satisfiable_positive_edges(body, &sat, active, full_defined, dead, &mut edges);
        for r in edges {
            if live.insert(r.clone()) {
                stack.push(r);
            }
        }
    }
    live
}

/// Per-rule `@emit_fact` kinds across ALL annotation surfaces (rule-level, per-branch,
/// mid-sequence) — the per-rule refinement of `collect_emitted_fact_kinds`, same complete
/// enumeration. Counting a BRANCH emission toward its whole rule is deliberate and sound in P2's
/// direction: it can only make MORE kinds producible (fewer deadness claims), never fewer.
fn emitted_fact_kinds_by_rule(annotations: &Annotations) -> HashMap<String, HashSet<String>> {
    let mut emitted: HashMap<String, HashSet<String>> = HashMap::new();
    let visit = |rule: &str, ann: &SemanticAnnotation, out: &mut HashMap<String, HashSet<String>>| {
        if let Ok(Some(SemanticRuntimeDirective::EmitFact(spec))) =
            parse_semantic_runtime_directive(ann)
        {
            out.entry(rule.to_string()).or_default().insert(spec.kind);
        }
    };
    for (rule, anns) in &annotations.semantic_annotations {
        for ann in anns {
            visit(rule, ann, &mut emitted);
        }
    }
    for (rule, branches) in &annotations.branch_semantic_annotations {
        for branch in branches {
            for ann in branch {
                visit(rule, ann, &mut emitted);
            }
        }
    }
    for (rule, branches) in &annotations.branch_mid_sequence_semantic_annotations {
        for branch in branches {
            for mid in branch {
                visit(rule, &mid.annotation, &mut emitted);
            }
        }
    }
    emitted
}

/// The rules carrying an `@import_from_library` directive on ANY annotation surface — the P2
/// degradation trigger (a live import can inject facts with no in-parse emitter).
fn library_import_rules(annotations: &Annotations) -> HashSet<String> {
    let mut rules: HashSet<String> = HashSet::new();
    let visit = |rule: &str, ann: &SemanticAnnotation, out: &mut HashSet<String>| {
        if let Ok(Some(SemanticRuntimeDirective::ImportFromLibrary(_))) =
            parse_semantic_runtime_directive(ann)
        {
            out.insert(rule.to_string());
        }
    };
    for (rule, anns) in &annotations.semantic_annotations {
        for ann in anns {
            visit(rule, ann, &mut rules);
        }
    }
    for (rule, branches) in &annotations.branch_semantic_annotations {
        for branch in branches {
            for ann in branch {
                visit(rule, ann, &mut rules);
            }
        }
    }
    for (rule, branches) in &annotations.branch_mid_sequence_semantic_annotations {
        for branch in branches {
            for mid in branch {
                visit(rule, &mid.annotation, &mut rules);
            }
        }
    }
    rules
}

/// The fact-kind a primitive call REQUIRES to be positively present for the call to be true —
/// `has_fact`/`fact_attribute_equals` always; `fact_count_at_least` only with a LITERAL count ≥ 1
/// (a dynamic or zero count requires nothing). Everything else (incl. `lacks_fact` and the
/// negative duals) requires nothing.
fn positive_call_required_kind(call: &PrimitiveCall) -> Option<String> {
    match call.name.as_str() {
        "has_fact" | "fact_attribute_equals" => call.args.first().and_then(literal_fact_kind),
        "fact_count_at_least" => match call.args.get(1) {
            Some(PredicateValue::IntLit(n)) if *n >= 1 => {
                call.args.first().and_then(literal_fact_kind)
            }
            _ => None,
        },
        _ => None,
    }
}

/// The fact-kinds a predicate expression REQUIRES positively present for the WHOLE expression to
/// be true — the sound propositional core: And = union, Or = INTERSECTION (a kind is required
/// only if every disjunct requires it), Not/Compare/In = nothing (negation flips truth, and a
/// comparison's semantics are not modelled — never false-accuse).
fn required_positive_kinds_in_expr(expr: &PredicateExpr) -> std::collections::BTreeSet<String> {
    match expr {
        PredicateExpr::Call(call) => positive_call_required_kind(call).into_iter().collect(),
        PredicateExpr::Not(_) => Default::default(),
        PredicateExpr::And(a, b) => {
            let mut out = required_positive_kinds_in_expr(a);
            out.extend(required_positive_kinds_in_expr(b));
            out
        }
        PredicateExpr::Or(a, b) => {
            let a = required_positive_kinds_in_expr(a);
            let b = required_positive_kinds_in_expr(b);
            a.intersection(&b).cloned().collect()
        }
        PredicateExpr::Compare { .. } | PredicateExpr::In { .. } => Default::default(),
    }
}

/// Per-rule REQUIRED positive gate kinds from RULE-LEVEL `@predicate`s only (every derivation of
/// the rule must pass a rule-level predicate; a branch-level predicate kills only its branch, so
/// counting it rule-wide would falsely brand an escape-carrying rule dead — deliberately
/// excluded). Handles both the inline-expression form and the structured
/// `{name: has_fact, args: [...]}` form, mirroring `consulted_kinds_in_predicate`.
fn rule_level_required_positive_gate_kinds(
    annotations: &Annotations,
) -> HashMap<String, std::collections::BTreeSet<String>> {
    let mut gates: HashMap<String, std::collections::BTreeSet<String>> = HashMap::new();
    for (rule, anns) in &annotations.semantic_annotations {
        for ann in anns {
            let Ok(Some(SemanticRuntimeDirective::Predicate(spec))) =
                parse_semantic_runtime_directive(ann)
            else {
                continue;
            };
            let mut kinds: std::collections::BTreeSet<String> = Default::default();
            if let Ok(expr) = parse_predicate_expression(&spec.name) {
                kinds.extend(required_positive_kinds_in_expr(&expr));
            }
            let scalar_arg = |v: &super::UnifiedSemanticValue| match v {
                super::UnifiedSemanticValue::String(s)
                | super::UnifiedSemanticValue::Identifier(s) => Some(s.clone()),
                _ => None,
            };
            match spec.name.trim() {
                "has_fact" | "fact_attribute_equals" => {
                    kinds.extend(spec.args.first().and_then(scalar_arg));
                }
                "fact_count_at_least" => {
                    let literal_min_one = matches!(
                        spec.args.get(1),
                        Some(super::UnifiedSemanticValue::Number(n))
                            if n.trim().parse::<i64>().is_ok_and(|v| v >= 1)
                    );
                    if literal_min_one {
                        kinds.extend(spec.args.first().and_then(scalar_arg));
                    }
                }
                _ => {}
            }
            if !kinds.is_empty() {
                gates.entry(rule.clone()).or_default().extend(kinds);
            }
        }
    }
    gates
}

/// Does a MANDATORY descent of `node` force a rule outside `live_now` (dead, pruned, or
/// entry-unreachable — no derivation can render it)? The `mandatory_node_gated` algebra: Or is
/// forced iff EVERY alternative is (any clean alternative is an escape), Sequence iff ANY element
/// is, a min-0 quantifier never (skippable), a lookahead never (renders nothing). References to
/// rules defined in NO tree are external/include — never forcing. Returns the first forcing
/// referenced rule (deterministic: body walk order) for the reason string.
fn mandatory_forces_non_live(
    node: &ASTNode,
    live_now: &HashSet<String>,
    full_defined: &HashSet<String>,
) -> Option<String> {
    match node {
        ASTNode::Or { alternatives } => {
            let mut first: Option<String> = None;
            for alt in alternatives {
                match mandatory_forces_non_live(alt, live_now, full_defined) {
                    Some(r) => {
                        if first.is_none() {
                            first = Some(r);
                        }
                    }
                    None => return None,
                }
            }
            first
        }
        ASTNode::Sequence { elements } => elements
            .iter()
            .find_map(|e| mandatory_forces_non_live(e, live_now, full_defined)),
        ASTNode::Quantified { element, quantifier } => {
            if parse_quantifier_bounds(quantifier).is_some_and(|(min, _)| min >= 1) {
                mandatory_forces_non_live(element, live_now, full_defined)
            } else {
                None
            }
        }
        ASTNode::Lookahead { .. } => None,
        ASTNode::Atom { value } => match value {
            ASTValue::Node(inner) => mandatory_forces_non_live(inner, live_now, full_defined),
            ASTValue::Token(parts) => match referenced_rule(parts) {
                Some(r) if full_defined.contains(r) && !live_now.contains(r) => Some(r.to_string()),
                _ => None,
            },
        },
    }
}

/// P2 composed with P1, then the classification of a residual `UNKNOWN` set. See the section
/// comment above for the design + soundness boundaries. `active`/`rule_order` = the
/// profile-filtered tree; `full_defined` = the PRE-filter rule names (so pruned vs external
/// references are distinguished); `entries` = the declared entry universe; `unknown` = the cert
/// report's residual (classified in its given order). PURE; deterministic.
pub fn classify_profile_residual(
    active: &HashMap<String, ASTNode>,
    rule_order: &[String],
    full_defined: &HashSet<String>,
    entries: &[String],
    annotations: Option<&Annotations>,
    unknown: &[String],
) -> ProfileResidualClassification {
    let no_dead: HashSet<String> = HashSet::new();
    let live0 = profile_entry_positively_live(active, rule_order, full_defined, entries, &no_dead);

    let degraded_inert = annotations
        .map(|ann| library_import_rules(ann).iter().any(|r| live0.contains(r)))
        .unwrap_or(false);

    // The composed P1+P2 fixpoint (skipped entirely when degraded or annotation-free — P2 then
    // claims nothing and `live_final == live0`).
    let mut dead_reason: std::collections::BTreeMap<String, String> = Default::default();
    let mut live_final = live0.clone();
    if !degraded_inert {
        if let Some(annotations) = annotations {
            let gate_kinds = rule_level_required_positive_gate_kinds(annotations);
            let emitted_by_rule = emitted_fact_kinds_by_rule(annotations);
            if !gate_kinds.is_empty() {
                loop {
                    let dead_set: HashSet<String> = dead_reason.keys().cloned().collect();
                    let live_now = profile_entry_positively_live(
                        active,
                        rule_order,
                        full_defined,
                        entries,
                        &dead_set,
                    );
                    let producible: HashSet<&String> = live_now
                        .iter()
                        .filter_map(|r| emitted_by_rule.get(r))
                        .flatten()
                        .collect();
                    // Only the DIRECT gate check is needed here: a rule whose mandatory descent
                    // forces a dead rule is unsatisfiable once the dead rule is ⊥, so the next
                    // live recomputation drops it (and everything reachable only through it)
                    // automatically — the satisfiability composition IS the mandatory algebra's
                    // dual. The mandatory-descent walk is used for ATTRIBUTION at classification
                    // time (naming the forcing rule), never as a second deadness mechanism.
                    let mut changed = false;
                    for rule in rule_order {
                        if !live_now.contains(rule) || dead_reason.contains_key(rule) {
                            continue;
                        }
                        if let Some(kind) = gate_kinds
                            .get(rule)
                            .and_then(|ks| ks.iter().find(|k| !producible.contains(k)))
                        {
                            dead_reason
                                .insert(rule.clone(), format!("gate kind '{kind}' unproducible"));
                            changed = true;
                        }
                    }
                    if !changed {
                        live_final = live_now;
                        break;
                    }
                }
            }
        }
    }

    let mut profile_entry_unreachable = Vec::new();
    let mut store_unproducible = Vec::new();
    let mut genuine = Vec::new();
    for rule in unknown {
        if !live0.contains(rule) {
            profile_entry_unreachable.push(rule.clone());
        } else if let Some(reason) = dead_reason.get(rule) {
            store_unproducible.push((rule.clone(), reason.clone()));
        } else if !live_final.contains(rule) {
            // Dead by the store fixpoint's cascade — attribute the mechanism: name the mandatory
            // reference that forces a non-live rule when there is one (the
            // `known_unscoped_block_type_identifier := checked_type_identifier …` shape), else the
            // rule lost its only reach path through a dead carrier (stranded).
            let reason = active
                .get(rule)
                .and_then(|body| mandatory_forces_non_live(body, &live_final, full_defined))
                .map(|via| format!("mandatory descent forces '{via}'"))
                .unwrap_or_else(|| "stranded by the store fixpoint".to_string());
            store_unproducible.push((rule.clone(), reason));
        } else {
            genuine.push(rule.clone());
        }
    }
    ProfileResidualClassification {
        entries: entries.to_vec(),
        degraded_inert,
        profile_entry_unreachable,
        store_unproducible,
        genuine,
    }
}

/// VERILOG-2005-PROFILE.6.7 (proof gathering, per-profile): the set of active rules covered by a
/// VERIFIED PER-PROFILE proof — each proposed by `classify_profile_residual` (P1 profile-entry
/// unreachability + P2 unproducible-store-gate fixpoint) AND independently re-confirmed by
/// `verify_profile_certificate`. This is the profile-scoped analogue of
/// `gather_verified_proof_covered_rules`: under a dialect profile, a rule stranded by the pruning of
/// every context that could reach it (or dead under the store fixpoint) is provably never-witnessed,
/// but the profile-agnostic whole-rule proofs cannot say so (`reachable_rules` promotes such a
/// stranded rule to its own secondary root). SOUND because a rule NOT positively reachable from any
/// declared entry — or dead under the store fixpoint — can never be exercised by an accepted parse,
/// so the transactional witness primitive correctly never records it. The classification quantifies
/// over the DECLARED ENTRY UNIVERSE `entries`, so the entry-relative cohort (`library_text`, …) is
/// NEVER falsely branded dead. PURE + deterministic (set membership; the caller need not order). A
/// proposed rule whose certificate fails re-verify is a LINTER BUG — returned in `failures`, never
/// silently covered.
pub fn gather_verified_profile_proof_covered_rules(
    active: &HashMap<String, ASTNode>,
    rule_order: &[String],
    full_defined: &HashSet<String>,
    entries: &[String],
    annotations: Option<&Annotations>,
    profile: &str,
) -> (HashSet<String>, Vec<String>) {
    let mut covered = HashSet::new();
    let mut failures = Vec::new();
    // Classify the WHOLE active rule set (pass `rule_order` as the candidate universe): P1-dead rules
    // land in `profile_entry_unreachable`, store-fixpoint-dead in `store_unproducible`, the rest in
    // `genuine` (never covered here). The store fixpoint itself is grammar-global — independent of
    // this candidate list.
    let classification =
        classify_profile_residual(active, rule_order, full_defined, entries, annotations, rule_order);
    for rule in &classification.profile_entry_unreachable {
        let cert = WellformednessCertificate::ProfileEntryUnreachable {
            rule: rule.clone(),
            profile: profile.to_string(),
            entries: entries.to_vec(),
        };
        match verify_profile_certificate(active, rule_order, full_defined, entries, annotations, &cert)
        {
            Ok(()) => {
                covered.insert(rule.clone());
            }
            Err(e) => failures
                .push(format!("profile-entry-unreachable proof for '{rule}' failed re-verify: {e}")),
        }
    }
    for (rule, reason) in &classification.store_unproducible {
        let cert = WellformednessCertificate::ProfileUnproducibleGate {
            rule: rule.clone(),
            profile: profile.to_string(),
            reason: reason.clone(),
        };
        match verify_profile_certificate(active, rule_order, full_defined, entries, annotations, &cert)
        {
            Ok(()) => {
                covered.insert(rule.clone());
            }
            Err(e) => failures
                .push(format!("profile-unproducible-gate proof for '{rule}' failed re-verify: {e}")),
        }
    }
    (covered, failures)
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
    fn lookahead_only_rule_is_proof_covered_and_checker_rejects_bogus() {
        // RTL-FE-CLOSURE.5.1.1: a rule referenced ONLY inside a negative lookahead is structurally
        // reachable but POSITIVELY-UNREACHABLE — it can never be positively entered, so the witness
        // primitive can never record it; its non-witnessing is a SOUND proof (covered_by_proof),
        // mirroring rtl_frontend's `port_direction_token`. Shape:
        //   list := item ( sep !stop item )*    (stop appears ONLY under `!`)
        let mut g = HashMap::new();
        g.insert(
            "list".into(),
            seq(vec![
                rule_ref("item"),
                quant(
                    seq(vec![rule_ref("sep"), look(rule_ref("stop"), false), rule_ref("item")]),
                    "*",
                ),
            ]),
        );
        g.insert("item".into(), token("string", "id"));
        g.insert("sep".into(), token("string", ","));
        g.insert("stop".into(), token("string", "X"));
        let order: Vec<String> =
            vec!["list".into(), "item".into(), "sep".into(), "stop".into()];

        // `stop` is structurally reachable (via the lookahead edge) but NOT positively reachable.
        assert!(reachable_rules(&g, &order).contains("stop"));
        assert!(!positively_reachable_rules(&g, &order).contains("stop"));
        // ...while every positively-used rule IS positively reachable.
        for r in ["list", "item", "sep"] {
            assert!(
                positively_reachable_rules(&g, &order).contains(r),
                "{r} is used positively and must be positively reachable"
            );
        }

        // The detector surfaces exactly `stop`; the proof-gather puts it (and only it) in the proof set.
        assert_eq!(detect_lookahead_only_rules(&g, &order), vec!["stop".to_string()]);
        let (proof, fails) = gather_verified_proof_covered_rules(&g, &order);
        assert!(fails.is_empty(), "no proof should fail re-verify: {fails:?}");
        assert!(proof.contains("stop"), "lookahead-only `stop` must be proof-covered");
        assert!(!proof.contains("item"), "a positively-used rule is NOT proof-covered");

        // The checker re-verifies a valid certificate and REJECTS a bogus one (a positively-reachable
        // rule claimed lookahead-only) — soundness: the proof bucket can never be gamed.
        let cert = WellformednessCertificate::LookaheadOnlyRule { rule: "stop".into() };
        assert!(verify_wellformedness_certificate(&g, &order, None, &cert).is_ok());
        let bogus = WellformednessCertificate::LookaheadOnlyRule { rule: "item".into() };
        assert!(
            verify_wellformedness_certificate(&g, &order, None, &bogus).is_err(),
            "claiming a positively-reachable rule is lookahead-only must be rejected"
        );
    }

    #[test]
    fn rule_used_both_positively_and_in_lookahead_is_not_lookahead_only() {
        // SOUNDNESS guard: a rule used BOTH positively AND inside a lookahead is positively
        // reachable, so it must NOT be proof-classified (it can and must still be witnessed). This
        // is what stops the proof bucket from ever masking a real witness. Shape:
        //   r := dir ( sep !dir item )*   (`dir` is the first positive element AND the guard)
        let mut g = HashMap::new();
        g.insert(
            "r".into(),
            seq(vec![
                rule_ref("dir"),
                quant(
                    seq(vec![rule_ref("sep"), look(rule_ref("dir"), false), rule_ref("item")]),
                    "*",
                ),
            ]),
        );
        g.insert("dir".into(), token("string", "in"));
        g.insert("sep".into(), token("string", ","));
        g.insert("item".into(), token("string", "id"));
        let order: Vec<String> = vec!["r".into(), "dir".into(), "sep".into(), "item".into()];
        assert!(positively_reachable_rules(&g, &order).contains("dir"));
        assert!(detect_lookahead_only_rules(&g, &order).is_empty());
        let (proof, _) = gather_verified_proof_covered_rules(&g, &order);
        assert!(
            !proof.contains("dir"),
            "a rule used both positively and in a lookahead must NOT be proof-covered"
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
    fn g4_2_gatherers_compose_into_full_certification() {
        // GRAMMAR-WELLFORMED.G.4.2: the proof gatherer (verified unreachable rules) + the witness
        // gatherer (verified witnesses) compose into certificate_coverage → full certification.
        // entry->keep reachable; island<->other a dead island.
        let mut g = HashMap::new();
        g.insert("entry".into(), rule_ref("keep"));
        g.insert("keep".into(), token("string", "k"));
        g.insert("island".into(), rule_ref("other"));
        g.insert("other".into(), rule_ref("island"));
        let order: Vec<String> =
            vec!["entry".into(), "keep".into(), "island".into(), "other".into()];
        let (proof_covered, pf) = gather_verified_proof_covered_rules(&g, &order);
        assert!(pf.is_empty(), "no proof should fail re-verify: {pf:?}");
        assert!(proof_covered.contains("island") && proof_covered.contains("other"));
        // witnesses for the reachable rules; a deliberately-bad one must be rejected (not counted).
        let witnesses = vec![
            ReachabilityWitness { fragment: "entry".into(), input: "k".into() },
            ReachabilityWitness { fragment: "keep".into(), input: "k".into() },
            ReachabilityWitness { fragment: "keep".into(), input: "BAD".into() },
        ];
        let parse_and_cover = |input: &str| -> (bool, std::collections::HashSet<String>) {
            if input == "k" {
                (true, ["entry".to_string(), "keep".to_string()].into_iter().collect())
            } else {
                (false, std::collections::HashSet::new())
            }
        };
        let (witness_covered, wf) = gather_verified_witness_covered(&witnesses, parse_and_cover);
        assert!(witness_covered.contains("entry") && witness_covered.contains("keep"));
        assert_eq!(wf.len(), 1, "the non-parsing witness must be a recorded failure: {wf:?}");
        // capstone: entry/keep witnessed + island/other proven ⇒ every rule covered ⇒ fully certified.
        let report = certificate_coverage(&order, &proof_covered, &witness_covered);
        assert!(report.is_fully_certified(), "all 4 rules proof- or witness-covered: {report:?}");
    }

    #[test]
    fn certificate_coverage_classifies_proof_witness_unknown() {
        // GRAMMAR-WELLFORMED.G.4: every fragment is proof-covered, witness-covered, or UNKNOWN;
        // fully-certified iff UNKNOWN is empty.
        let fragments: Vec<String> =
            ["a", "b", "c", "d"].iter().map(|s| s.to_string()).collect();
        let proof: HashSet<String> = ["a".to_string()].into_iter().collect(); // a proven dead
        let witness: HashSet<String> = ["b".to_string(), "c".to_string()].into_iter().collect(); // b,c witnessed
        let report = certificate_coverage(&fragments, &proof, &witness);
        assert_eq!(report.total, 4);
        assert_eq!(report.covered_by_proof, vec!["a".to_string()]);
        assert_eq!(report.covered_by_witness, vec!["b".to_string(), "c".to_string()]);
        assert_eq!(report.unknown, vec!["d".to_string()]); // d = neither -> ticket
        assert!(!report.is_fully_certified(), "an UNKNOWN fragment means NOT fully certified");
        // Once d is witnessed, the grammar is fully certified.
        let witness2: HashSet<String> =
            ["b", "c", "d"].iter().map(|s| s.to_string()).collect();
        let report2 = certificate_coverage(&fragments, &proof, &witness2);
        assert!(report2.unknown.is_empty() && report2.is_fully_certified());
    }

    #[test]
    fn certificate_coverage_union_is_over_positively_covered_sets() {
        // GRAMMAR-WELLFORMED.H.12.8.1.1 — the load-bearing SOUNDNESS rule for the opt-in multi-config
        // cert-coverage union (`--cert-union-config`): a rule is CERTIFIED iff it is POSITIVELY
        // covered (proof OR witness) in SOME config; UNKNOWN iff covered in NONE. The union must be
        // over the COVERED sets, NEVER over "not-UNKNOWN-in-some-config" — otherwise a rule that is
        // covered by NO config (e.g. a profile FILTERS it out of its `rule_order`, so it is neither
        // covered nor UNKNOWN there) would be falsely certified. `profile_rule` models an
        // IEEE-1800-2023 rule that witnesses under `sv_2023` (sound to certify); `entry_only_rule`
        // models a library/parseable-fragment rule that NO supported config covers (it must stay
        // UNKNOWN — the union does not invent a certificate for it).
        let canonical_fragments: Vec<String> = ["base_rule", "profile_rule", "entry_only_rule"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        // Canonical (sv_2017-like) config: only `base_rule` witnesses.
        let canonical_witness: HashSet<String> = ["base_rule".to_string()].into_iter().collect();
        let canonical_proof: HashSet<String> = HashSet::new();
        let canonical =
            certificate_coverage(&canonical_fragments, &canonical_proof, &canonical_witness);
        assert_eq!(
            canonical.unknown,
            vec!["profile_rule".to_string(), "entry_only_rule".to_string()],
            "canonically, both the profile rule and the uncovered entry-only rule are UNKNOWN"
        );

        // Alt config (sv_2023-like): witnesses {base_rule, profile_rule}; it does NOT cover
        // `entry_only_rule` (whether the profile filters it out of its universe or simply never
        // witnesses it, the effect is the same — it is absent from this config's covered set).
        let alt_witness: HashSet<String> =
            ["base_rule", "profile_rule"].iter().map(|s| s.to_string()).collect();

        // The SOUND union: union the POSITIVELY-covered sets, classify the CANONICAL fragment set.
        let mut witness_union = canonical_witness.clone();
        witness_union.extend(alt_witness.iter().cloned());
        let proof_union = canonical_proof.clone();
        let union = certificate_coverage(&canonical_fragments, &proof_union, &witness_union);
        assert_eq!(
            union.covered_by_witness,
            vec!["base_rule".to_string(), "profile_rule".to_string()],
            "the union credits the profile rule that witnesses under the alt config"
        );
        assert_eq!(
            union.unknown,
            vec!["entry_only_rule".to_string()],
            "a rule covered by NO config is NOT leaked into the certified set"
        );
        assert!(!union.is_fully_certified());

        // Guard the TRAP this rule exists to prevent: a naïve "not-UNKNOWN-in-some-config" union would
        // wrongly certify `entry_only_rule`. The alt config's UNKNOWN set is EMPTY (everything in its
        // own universe witnesses), so `canonical.unknown − alt.unknown` would still contain
        // `entry_only_rule` and a "subtract per-config UNKNOWN" scheme would falsely drop it. The
        // covered-set union never adds it, which is exactly the soundness invariant.
        let alt_fragments: Vec<String> =
            ["base_rule", "profile_rule"].iter().map(|s| s.to_string()).collect();
        let alt = certificate_coverage(&alt_fragments, &HashSet::new(), &alt_witness);
        assert!(
            alt.unknown.is_empty(),
            "the alt config has no UNKNOWN in its own universe — the trap source"
        );
        assert!(
            !witness_union.contains("entry_only_rule") && !proof_union.contains("entry_only_rule"),
            "soundness invariant: a rule covered by no config never enters the covered union"
        );
    }

    #[test]
    fn parse_node_covered_rules_walks_the_ast() {
        // GRAMMAR-WELLFORMED.G.3.3: the covered set = rule names PRESENT in the parse tree.
        use crate::ast_pipeline::{ParseContent, ParseNode};
        let leaf = ParseNode { rule_name: "leaf", content: ParseContent::Terminal("x"), span: 0..1 };
        let inner =
            ParseNode { rule_name: "inner", content: ParseContent::Sequence(vec![leaf]), span: 0..1 };
        let root = ParseNode {
            rule_name: "root",
            content: ParseContent::Alternative(Box::new(inner)),
            span: 0..1,
        };
        let covered = parse_node_covered_rules(&root);
        assert!(covered.contains("root") && covered.contains("inner") && covered.contains("leaf"));
        // And it composes with the witness checker: a witness for "inner" verifies via a real walk.
        let parse_and_cover = |_input: &str| (true, parse_node_covered_rules(&root));
        let w = ReachabilityWitness { fragment: "inner".into(), input: "x".into() };
        assert!(verify_reachability_witness(parse_and_cover, &w).is_ok());
        let miss = ReachabilityWitness { fragment: "absent".into(), input: "x".into() };
        assert!(verify_reachability_witness(parse_and_cover, &miss).is_err());
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

    // ---- VERILOG-2005-PROFILE.6.6: per-profile residual classification (P1 + P2) ----

    /// The pre-filter rule universe for the P1/P2 tests: the ACTIVE tree's rules plus the rules
    /// the "profile" pruned (so pruned references read as unsatisfiable, not external).
    fn full_with(active: &HashMap<String, ASTNode>, pruned: &[&str]) -> HashSet<String> {
        let mut full: HashSet<String> = active.keys().cloned().collect();
        full.extend(pruned.iter().map(|s| s.to_string()));
        full
    }

    #[test]
    fn p1_stranded_rule_is_not_live_and_second_entry_rescues_its_cohort() {
        // ACTIVE tree (post profile filter): the filter pruned `host` (the only referencer of
        // `stranded`), so `stranded` survives UNREFERENCED — the exact shape the
        // unreferenced→secondary-root heuristic mis-promotes. P1 must NOT treat it as a root.
        // `lib_root` is a second DECLARED entry whose cohort must stay live.
        let mut g = HashMap::new();
        g.insert("entry".into(), rule_ref("a"));
        g.insert("a".into(), token("string", "x"));
        g.insert("stranded".into(), token("string", "y"));
        g.insert("lib_root".into(), rule_ref("lib_item"));
        g.insert("lib_item".into(), token("string", "z"));
        let order: Vec<String> =
            vec!["entry".into(), "a".into(), "stranded".into(), "lib_root".into(), "lib_item".into()];
        let full = full_with(&g, &["host"]);
        let no_dead = HashSet::new();

        // Single-entry universe: the stranded rule AND the other entry's cohort are not live.
        let live = profile_entry_positively_live(&g, &order, &full, &["entry".into()], &no_dead);
        assert!(live.contains("entry") && live.contains("a"));
        assert!(!live.contains("stranded"), "an unreferenced stranded rule must NOT self-root");
        assert!(!live.contains("lib_item"), "another entry's cohort is not live under one entry");

        // Declaring the second entry rescues exactly its cohort — never the stranded rule.
        let live2 = profile_entry_positively_live(
            &g,
            &order,
            &full,
            &["entry".into(), "lib_root".into()],
            &no_dead,
        );
        assert!(live2.contains("lib_root") && live2.contains("lib_item"));
        assert!(!live2.contains("stranded"));
    }

    #[test]
    fn p1_edges_are_satisfiability_honest_and_lookaheads_emit_none() {
        // entry := ( pruned_ref target ) | b        — alternative 1 mandatorily crosses a PRUNED
        // rule, so `target` gets NO edge from it (class B: a spurious reach path through a gated
        // mandatory sibling). guard := !target b     — a lookahead reference is no positive edge.
        let mut g = HashMap::new();
        g.insert(
            "entry".into(),
            or(vec![seq(vec![rule_ref("pruned"), rule_ref("target")]), rule_ref("b")]),
        );
        g.insert("target".into(), token("string", "t"));
        g.insert("b".into(), seq(vec![look(rule_ref("target"), false), token("string", "b")]));
        let order: Vec<String> = vec!["entry".into(), "target".into(), "b".into()];
        let full = full_with(&g, &["pruned"]);
        let live =
            profile_entry_positively_live(&g, &order, &full, &["entry".into()], &HashSet::new());
        assert!(live.contains("b"), "the satisfiable alternative stays live");
        assert!(
            !live.contains("target"),
            "no positive edge through an unsatisfiable alternative or a lookahead"
        );
    }

    #[test]
    fn p2_unproducible_gate_and_cascade_are_classified_and_escapes_stay_genuine() {
        // The v2005 class-A shape in miniature:
        //   entry     := use_site | cascade | escape | negative | plain
        //   producer  := "decl"        @emit_fact(type_name)   — but its ONLY referencer is
        //                                                        PRUNED, so it is NOT live
        //   use_site  := "id"          @predicate has_fact(type_name, x)   → gate-dead
        //   cascade   := use_site "!"                                       → dead via use_site
        //   escape    := use_site | "ok"                                    → has an escape: live
        //   negative  := "id2"         @predicate lacks_fact(type_name, x)  → negative gate: live
        let mut g = HashMap::new();
        g.insert(
            "entry".into(),
            or(vec![
                rule_ref("use_site"),
                rule_ref("cascade"),
                rule_ref("escape"),
                rule_ref("negative"),
                rule_ref("plain"),
            ]),
        );
        g.insert("producer".into(), token("string", "decl"));
        g.insert("use_site".into(), token("string", "id"));
        g.insert("cascade".into(), seq(vec![rule_ref("use_site"), token("string", "!")]));
        g.insert("escape".into(), or(vec![rule_ref("use_site"), token("string", "ok")]));
        g.insert("negative".into(), token("string", "id2"));
        g.insert("plain".into(), token("string", "p"));
        let order: Vec<String> = vec![
            "entry".into(),
            "producer".into(),
            "use_site".into(),
            "cascade".into(),
            "escape".into(),
            "negative".into(),
            "plain".into(),
        ];
        let full = full_with(&g, &["pruned_host"]);
        let mut ann = Annotations::default();
        ann.semantic_annotations.insert("producer".into(), vec![emit_fact_ann("type_name")]);
        ann.semantic_annotations
            .insert("use_site".into(), vec![predicate_ann("has_fact(type_name, x)")]);
        ann.semantic_annotations
            .insert("negative".into(), vec![predicate_ann("lacks_fact(type_name, x)")]);
        let unknown: Vec<String> = vec![
            "producer".into(),
            "use_site".into(),
            "cascade".into(),
            "escape".into(),
            "negative".into(),
            "plain".into(),
        ];
        let c = classify_profile_residual(
            &g,
            &order,
            &full,
            &["entry".into()],
            Some(&ann),
            &unknown,
        );
        assert!(!c.degraded_inert);
        // The producer is entry-unreachable (its referencer was pruned) — class (i).
        assert_eq!(c.profile_entry_unreachable, vec!["producer".to_string()]);
        // The use-site's kind has no LIVE emitter → gate-dead; the cascade follows — class (ii).
        assert_eq!(
            c.store_unproducible,
            vec![
                ("use_site".to_string(), "gate kind 'type_name' unproducible".to_string()),
                ("cascade".to_string(), "mandatory descent forces 'use_site'".to_string()),
            ]
        );
        // The escape-carrying rule, the negative-gated rule, and the plain rule stay honest.
        assert_eq!(
            c.genuine,
            vec!["escape".to_string(), "negative".to_string(), "plain".to_string()]
        );
    }

    #[test]
    fn profile_proof_gathering_covers_p1_and_p2_and_checker_rejects_live_and_genuine_rules() {
        // VERILOG-2005-PROFILE.6.7: the class-A miniature — `producer` is P1 (its ONLY referencer is
        // pruned, so it survives unreferenced but is entry-unreachable), `use_site` is P2 (its
        // store-gate kind has no LIVE emitter), `escape`/`plain`/`entry` are genuine/live.
        let mut g = HashMap::new();
        g.insert(
            "entry".into(),
            or(vec![rule_ref("use_site"), rule_ref("escape"), rule_ref("plain")]),
        );
        g.insert("producer".into(), token("string", "decl"));
        g.insert("use_site".into(), token("string", "id"));
        g.insert("escape".into(), or(vec![rule_ref("use_site"), token("string", "ok")]));
        g.insert("plain".into(), token("string", "p"));
        let order: Vec<String> = vec![
            "entry".into(),
            "producer".into(),
            "use_site".into(),
            "escape".into(),
            "plain".into(),
        ];
        let full = full_with(&g, &["pruned_host"]);
        let mut ann = Annotations::default();
        ann.semantic_annotations.insert("producer".into(), vec![emit_fact_ann("type_name")]);
        ann.semantic_annotations
            .insert("use_site".into(), vec![predicate_ann("has_fact(type_name, x)")]);
        let entries = vec!["entry".to_string()];

        // The gatherer covers exactly the P1∪P2 set, and EVERY proposed proof re-verifies.
        let (covered, failures) = gather_verified_profile_proof_covered_rules(
            &g,
            &order,
            &full,
            &entries,
            Some(&ann),
            "test_profile",
        );
        assert!(failures.is_empty(), "every proposed per-profile proof must re-verify: {failures:?}");
        assert!(covered.contains("producer"), "P1 entry-unreachable rule is proof-covered");
        assert!(covered.contains("use_site"), "P2 store-unproducible rule is proof-covered");
        assert!(
            !covered.contains("escape") && !covered.contains("plain") && !covered.contains("entry"),
            "a genuine/live rule is NEVER covered by a per-profile proof"
        );

        // The checker independently ACCEPTS a valid P1 cert, REJECTS one for a LIVE rule, and
        // REJECTS a cert whose carried entry universe differs from the run's.
        let good_p1 = WellformednessCertificate::ProfileEntryUnreachable {
            rule: "producer".into(),
            profile: "test_profile".into(),
            entries: entries.clone(),
        };
        assert!(verify_profile_certificate(&g, &order, &full, &entries, Some(&ann), &good_p1).is_ok());
        let bad_p1 = WellformednessCertificate::ProfileEntryUnreachable {
            rule: "plain".into(),
            profile: "test_profile".into(),
            entries: entries.clone(),
        };
        assert!(
            verify_profile_certificate(&g, &order, &full, &entries, Some(&ann), &bad_p1).is_err(),
            "a live rule is NOT profile-entry-unreachable"
        );
        let mismatched = WellformednessCertificate::ProfileEntryUnreachable {
            rule: "producer".into(),
            profile: "test_profile".into(),
            entries: vec!["entry".into(), "other".into()],
        };
        assert!(
            verify_profile_certificate(&g, &order, &full, &entries, Some(&ann), &mismatched).is_err(),
            "a cert whose entry universe differs from the run's must be rejected"
        );

        // The checker ACCEPTS a valid P2 cert and REJECTS one for a genuine (live, ungated) rule.
        let good_p2 = WellformednessCertificate::ProfileUnproducibleGate {
            rule: "use_site".into(),
            profile: "test_profile".into(),
            reason: "gate kind 'type_name' unproducible".into(),
        };
        assert!(verify_profile_certificate(&g, &order, &full, &entries, Some(&ann), &good_p2).is_ok());
        let bad_p2 = WellformednessCertificate::ProfileUnproducibleGate {
            rule: "plain".into(),
            profile: "test_profile".into(),
            reason: "irrelevant".into(),
        };
        assert!(
            verify_profile_certificate(&g, &order, &full, &entries, Some(&ann), &bad_p2).is_err(),
            "a genuine rule is NOT store-unproducible"
        );

        // The GENERIC verifier refuses the profile variants (it lacks the entry universe / pre-filter
        // set) — routing them here is mandatory, not optional.
        assert!(verify_wellformedness_certificate(&g, &order, Some(&ann), &good_p1).is_err());
    }

    #[test]
    fn p2_live_emitter_keeps_gate_satisfiable_and_import_degrades_inert() {
        // (a) With the producer LIVE (referenced from the entry), the same gate is satisfiable —
        // the gated rule is genuine. (b) With `@import_from_library` on a live rule, P2 makes NO
        // store claims at all (degraded inert), while P1 still classifies entry-unreachability.
        let mut g = HashMap::new();
        g.insert("entry".into(), or(vec![rule_ref("producer"), rule_ref("use_site")]));
        g.insert("producer".into(), token("string", "decl"));
        g.insert("use_site".into(), token("string", "id"));
        g.insert("stranded".into(), token("string", "s"));
        let order: Vec<String> =
            vec!["entry".into(), "producer".into(), "use_site".into(), "stranded".into()];
        let full = full_with(&g, &["pruned_host"]);
        let mut ann = Annotations::default();
        ann.semantic_annotations.insert("producer".into(), vec![emit_fact_ann("type_name")]);
        ann.semantic_annotations
            .insert("use_site".into(), vec![predicate_ann("has_fact(type_name, x)")]);
        let unknown: Vec<String> = vec!["use_site".into(), "stranded".into()];
        let c = classify_profile_residual(
            &g,
            &order,
            &full,
            &["entry".into()],
            Some(&ann),
            &unknown,
        );
        assert!(!c.degraded_inert);
        assert!(c.store_unproducible.is_empty(), "a live emitter satisfies the gate: {c:?}");
        assert_eq!(c.genuine, vec!["use_site".to_string()]);
        assert_eq!(c.profile_entry_unreachable, vec!["stranded".to_string()]);

        // (b) the degradation trigger: an @import_from_library on a LIVE rule. Rebuild with the
        // producer NOT live (so the gate WOULD be dead) — the import must suppress the claim.
        let mut g2 = g.clone();
        g2.insert("entry".into(), rule_ref("use_site"));
        let mut ann2 = Annotations::default();
        ann2.semantic_annotations.insert("producer".into(), vec![emit_fact_ann("type_name")]);
        ann2.semantic_annotations.insert(
            "use_site".into(),
            vec![
                predicate_ann("has_fact(type_name, x)"),
                sem_named(
                    "import_from_library",
                    crate::ast_pipeline::UnifiedSemanticValue::Object(vec![
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "kind".into(),
                            value: crate::ast_pipeline::UnifiedSemanticValue::Identifier(
                                "package".into(),
                            ),
                        },
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "name_from".into(),
                            value: crate::ast_pipeline::UnifiedSemanticValue::RuleReference(
                                "$1".into(),
                            ),
                        },
                    ]),
                ),
            ],
        );
        let unknown2: Vec<String> = vec!["use_site".into()];
        let c2 = classify_profile_residual(
            &g2,
            &order,
            &full,
            &["entry".into()],
            Some(&ann2),
            &unknown2,
        );
        assert!(c2.degraded_inert, "a live @import_from_library must degrade P2 to inert");
        assert!(c2.store_unproducible.is_empty(), "no store claim under degradation: {c2:?}");
        assert_eq!(c2.genuine, vec!["use_site".to_string()]);
    }

    #[test]
    fn p2_or_required_kinds_intersect_and_count_gate_needs_literal_min_one() {
        // (a) `has_fact(a, x) || has_fact(b, x)` REQUIRES neither kind alone (intersection) —
        // with only `a` unproducible the rule stays live. (b) `fact_count_at_least` gates only
        // with a literal count ≥ 1: a dynamic count claims nothing.
        let mut g = HashMap::new();
        g.insert("entry".into(), or(vec![rule_ref("either"), rule_ref("counted"), rule_ref("emit_b")]));
        g.insert("either".into(), token("string", "e"));
        g.insert("counted".into(), token("string", "c"));
        g.insert("emit_b".into(), token("string", "b"));
        let order: Vec<String> =
            vec!["entry".into(), "either".into(), "counted".into(), "emit_b".into()];
        let full = full_with(&g, &[]);
        let mut ann = Annotations::default();
        ann.semantic_annotations.insert("emit_b".into(), vec![emit_fact_ann("b")]);
        ann.semantic_annotations
            .insert("either".into(), vec![predicate_ann("has_fact(a, x) || has_fact(b, x)")]);
        ann.semantic_annotations
            .insert("counted".into(), vec![predicate_ann("fact_count_at_least(a, $n)")]);
        let unknown: Vec<String> = vec!["either".into(), "counted".into()];
        let c = classify_profile_residual(
            &g,
            &order,
            &full,
            &["entry".into()],
            Some(&ann),
            &unknown,
        );
        assert!(
            c.store_unproducible.is_empty(),
            "an Or-escape kind and a dynamic count must claim nothing: {c:?}"
        );
        assert_eq!(c.genuine, vec!["either".to_string(), "counted".to_string()]);

        // The AND form DOES require the unproducible kind — the claim fires.
        let mut ann2 = Annotations::default();
        ann2.semantic_annotations.insert("emit_b".into(), vec![emit_fact_ann("b")]);
        ann2.semantic_annotations
            .insert("either".into(), vec![predicate_ann("has_fact(a, x) && has_fact(b, x)")]);
        let unknown2: Vec<String> = vec!["either".into()];
        let c2 = classify_profile_residual(
            &g,
            &order,
            &full,
            &["entry".into()],
            Some(&ann2),
            &unknown2,
        );
        assert_eq!(
            c2.store_unproducible,
            vec![("either".to_string(), "gate kind 'a' unproducible".to_string())]
        );
    }

    #[test]
    fn p2_stranding_through_a_store_dead_rule_is_reported() {
        // `inner` is reachable ONLY through the gate-dead `use_site` — the fixpoint removes the
        // dead rule's edges, so `inner` leaves the live set and is reported as stranded.
        let mut g = HashMap::new();
        g.insert("entry".into(), rule_ref("use_site"));
        g.insert("use_site".into(), rule_ref("inner"));
        g.insert("inner".into(), token("string", "i"));
        let order: Vec<String> = vec!["entry".into(), "use_site".into(), "inner".into()];
        let full = full_with(&g, &[]);
        let mut ann = Annotations::default();
        ann.semantic_annotations
            .insert("use_site".into(), vec![predicate_ann("has_fact(type_name, x)")]);
        let unknown: Vec<String> = vec!["use_site".into(), "inner".into()];
        let c = classify_profile_residual(
            &g,
            &order,
            &full,
            &["entry".into()],
            Some(&ann),
            &unknown,
        );
        assert_eq!(
            c.store_unproducible,
            vec![
                ("use_site".to_string(), "gate kind 'type_name' unproducible".to_string()),
                ("inner".to_string(), "stranded by the store fixpoint".to_string()),
            ]
        );
        assert!(c.genuine.is_empty());
    }
}
