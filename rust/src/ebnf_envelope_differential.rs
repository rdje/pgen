//! LANG-CAPABILITY-AUDIT.10.6 part 2 — the frontend⟷meta-parser **raw-AST envelope
//! differential**.
//!
//! # What this measures, and why the existing instruments cannot
//!
//! PGEN's stated endgame is that `grammars/ebnf.ebnf` REPLACES the hand-written EBNF frontend
//! (`README.md`: *"handwritten parsers exist only as bootstrap scaffolding"*). Three instruments
//! already touch the pair, and none of them can size that replacement:
//!
//! | instrument | what it compares |
//! |---|---|
//! | [`crate::ebnf_frontend`]'s inline cross-check | **verdict only** (`Ok`/`Err`), and soft by default |
//! | `ebnf_frontend_dual_run_diff_gate.sh` | the generated meta-parser's `parse_full` verdict |
//! | `parse_harness_equivalence` | interpreter ⟷ generated `ebnf.rs` — the *same* arm twice |
//!
//! A verdict says the meta-parser **reads** every grammar we ship. It cannot say it reads them
//! the **same way**. This module answers that second question: it projects the meta-parser's
//! typed AST into the hand-written frontend's `raw_ast` token-envelope vocabulary and diffs the
//! two, token by token.
//!
//! # The two arms
//!
//! - **Arm 1** — the hand-written Rust frontend, [`crate::ebnf_frontend`], which emits a flat
//!   per-rule token envelope: `[["rule","json"], ["rule_reference","value"], …]`.
//! - **Arm 2** — the parser GENERATED from `grammars/ebnf.ebnf`, whose return annotations shape
//!   a tree: `{type: "grammar_file", elements: [{type: "grammar_rule", …}]}`.
//!
//! The shapes differ by construction, so the differential needs a **projection**: an explicit,
//! auditable mapping from arm 2's 30-odd node types onto arm 1's 14 envelope kinds
//! ([`project_meta_parser_ast`]). Everything the projection cannot map is COUNTED AND NAMED
//! ([`Projection::unmapped`]) rather than dropped — an unmapped construct is a measurement gap,
//! and a differential that silently ignored it would report false parity.
//!
//! # What is comparable, and what is honestly not
//!
//! Return annotations are the one asymmetry that no projection can close. Arm 1 carries the
//! **raw source text** of the return expression (`"{type: \"json\", value: $1}"`); arm 2 carries
//! it **already parsed** into a tree. Re-serializing that tree back to source would require a
//! pretty-printer — a new trusted surface whose own bugs would show up as false divergences. So
//! this differential compares return annotations at **kind** level (`return_object` /
//! `return_array` / `return_scalar`) and reports their payloads as
//! [`TokenComparison::PayloadNotComparable`]. That bound is stated in the report itself
//! ([`DifferentialReport::payload_not_comparable`]), never implied.
//!
//! # Ground truth — this instrument refuses rather than guesses
//!
//! Per the standing decision *"an instrument with no ground truth is a confident guess"*, the
//! differential carries **both** a positive and a negative control and REFUSES on a miss:
//!
//! - [`positive_control_grammar`] — a synthetic grammar exercising every projected construct,
//!   which MUST project to an exactly-identical envelope. A divergence there means the
//!   PROJECTION is broken, so every number it produces is meaningless.
//! - [`negative_control_mutation`] — one token of the positive control's arm-1 envelope is
//!   mutated, and the differ MUST report exactly that one divergence at exactly that index. A
//!   clean verdict there means the DIFFER is blind, so its parity claims are meaningless.
//!
//! [`run_ground_truth_controls`] executes both against the live arms and returns `Err` on either
//! miss; the `ebnf_dual_run_diff` binary runs it before it will emit a report.

use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::{Value, json};

/// A single token of the hand-written frontend's `raw_ast` envelope, in the flat
/// `[kind, payload]` form arm 1 emits.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EnvelopeToken {
    /// One of the 14 envelope kinds arm 1 produces (`rule`, `rule_reference`, `operator`, …).
    pub kind: String,
    /// The token's payload. `None` marks a payload arm 2 structurally cannot supply — today
    /// only the return-annotation source text; see the module docs.
    pub payload: Option<Value>,
}

impl EnvelopeToken {
    fn new(kind: &str, payload: Value) -> Self {
        Self {
            kind: kind.to_string(),
            payload: Some(payload),
        }
    }

    /// A token whose KIND is comparable but whose payload arm 2 cannot supply.
    fn kind_only(kind: &str) -> Self {
        Self {
            kind: kind.to_string(),
            payload: None,
        }
    }

    /// Read arm 1's `[kind, payload]` pair. Returns `None` for anything that is not a
    /// two-element array with a string head — arm 1 emits nothing else, so a `None` here is a
    /// malformed envelope and the caller reports it rather than skipping it.
    fn from_arm1(value: &Value) -> Option<Self> {
        let items = value.as_array()?;
        if items.len() != 2 {
            return None;
        }
        Some(Self {
            kind: items[0].as_str()?.to_string(),
            payload: Some(items[1].clone()),
        })
    }
}

/// One rule's worth of envelope tokens. The leading `["rule", <name>]` token is kept in
/// [`tokens`](Self::tokens) so both arms are compared over identical sequences; [`name`](Self::name)
/// is lifted out purely to pair the arms up and to label divergences.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RuleEnvelope {
    pub name: Option<String>,
    pub tokens: Vec<EnvelopeToken>,
}

impl RuleEnvelope {
    fn from_arm1(value: &Value) -> Self {
        let tokens: Vec<EnvelopeToken> = value
            .as_array()
            .map(|items| items.iter().filter_map(EnvelopeToken::from_arm1).collect())
            .unwrap_or_default();
        let name = tokens
            .first()
            .filter(|token| token.kind == "rule")
            .and_then(|token| token.payload.as_ref())
            .and_then(Value::as_str)
            .map(str::to_string);
        Self { name, tokens }
    }
}

/// The result of projecting arm 2's typed AST into arm 1's envelope vocabulary.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Projection {
    pub rules: Vec<RuleEnvelope>,
    /// Every arm-2 construct the projection has no mapping for, by `<site>:<type>` label and
    /// count. A non-empty map means the differential's coverage is incomplete BY THAT MUCH, and
    /// the report says so instead of counting the construct as agreement.
    pub unmapped: BTreeMap<String, usize>,
    /// Top-level `grammar_file` elements that are not `grammar_rule` — comments, which arm 1
    /// drops entirely. Recorded so "arm 2 saw more elements" is never mistaken for a divergence.
    pub skipped_non_rule_elements: usize,
    /// Top-level `include(…)` directives arm 2 parsed but did NOT resolve.
    ///
    /// ⚠️ This is a REAL asymmetry, not a projection shortcut. Arm 1 resolves includes and
    /// splices the included rules into its envelope; arm 2 is a parser and stops at the
    /// directive. So the two arms describe different rule sets, and any token-level parity
    /// number over them would be meaningless. A non-zero count therefore defeats
    /// [`DifferentialReport::is_envelope_equivalent`] and is named in the report rather than
    /// quietly folded into the counts.
    pub unresolved_include_directives: usize,
}

impl Projection {
    fn note_unmapped(&mut self, label: String) {
        *self.unmapped.entry(label).or_insert(0) += 1;
    }
}

/// How one token position compared across the two arms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenComparison {
    /// Same kind, same payload.
    Match,
    /// Same kind; payload is one arm 2 structurally cannot supply (return-annotation source
    /// text). Counted separately so it is never silently folded into either column.
    PayloadNotComparable,
    /// Same kind, different payload.
    PayloadDivergence,
    /// Different kind, or one arm ran out of tokens.
    KindDivergence,
}

/// A single located disagreement between the arms.
#[derive(Debug, Clone, Serialize)]
pub struct Divergence {
    pub rule_index: usize,
    pub rule_name: Option<String>,
    pub token_index: usize,
    pub comparison: TokenComparison,
    pub arm1: Option<EnvelopeToken>,
    pub arm2: Option<EnvelopeToken>,
}

/// The differential's verdict for one grammar.
#[derive(Debug, Clone, Serialize)]
pub struct DifferentialReport {
    pub grammar: String,
    pub arm1_rules: usize,
    pub arm2_rules: usize,
    /// `true` iff both arms produced the same rule names in the same source order. This is the
    /// pairing precondition: when it is `false`, per-token counts below compare rules that may
    /// not correspond, and the flag says so.
    pub rule_name_sequence_identical: bool,
    pub tokens_compared: usize,
    pub token_matches: usize,
    pub payload_not_comparable: usize,
    pub payload_divergences: usize,
    pub kind_divergences: usize,
    /// Tokens after a rule's first divergence. One inserted or dropped token desynchronizes
    /// everything after it, so those positions are NOT counted as agreement or disagreement —
    /// they are unverified, and reported as such.
    pub tokens_unverified: usize,
    /// Divergences in source order, capped at [`MAX_REPORTED_DIVERGENCES`]. `divergence_total`
    /// is the uncapped count, so a truncated list can never read as a complete one.
    pub divergences: Vec<Divergence>,
    pub divergence_total: usize,
    pub unmapped_arm2_constructs: BTreeMap<String, usize>,
    pub skipped_non_rule_elements: usize,
    /// See [`Projection::unresolved_include_directives`] — a non-zero count means the arms are
    /// describing different rule sets and the token counts below cannot be read as parity.
    pub unresolved_include_directives: usize,
    /// [`Self::is_envelope_equivalent`], carried in the serialized report so a consuming gate
    /// reads THIS verdict rather than re-deriving it from the counts — two implementations of
    /// one predicate is how a gate starts disagreeing with the instrument it runs.
    pub is_envelope_equivalent: bool,
}

impl DifferentialReport {
    /// The headline number: the share of compared token positions where the arms agree, with
    /// not-comparable payloads counted as agreement on the KIND (which is all that was checked).
    /// `None` when nothing was comparable at all.
    pub fn agreement_ratio(&self) -> Option<f64> {
        if self.tokens_compared == 0 {
            return None;
        }
        let agreed = self.token_matches + self.payload_not_comparable;
        Some(agreed as f64 / self.tokens_compared as f64)
    }

    /// Derive [`Self::is_envelope_equivalent`] from the final counts. Private, and called once
    /// by [`diff_envelopes`] the moment they are final: the FIELD is the single public answer,
    /// so a consumer cannot read one verdict while the serialized report carries another.
    fn compute_envelope_equivalence(&self) -> bool {
        self.rule_name_sequence_identical
            && self.payload_divergences == 0
            && self.kind_divergences == 0
            && self.tokens_unverified == 0
            && self.unmapped_arm2_constructs.is_empty()
            && self.unresolved_include_directives == 0
    }
}

/// Cap on the divergence list carried in a report. The uncapped count travels alongside it
/// (`divergence_total`), so truncation is always visible.
pub const MAX_REPORTED_DIVERGENCES: usize = 40;

/// Project arm 2's typed AST — the `serde_json` serialization of the generated meta-parser's
/// `parse_full_grammar_file()` node — into arm 1's envelope vocabulary.
pub fn project_meta_parser_ast(ast: &Value) -> Projection {
    let mut projection = Projection::default();

    let Some(elements) = ast
        .pointer("/content/Json/elements")
        .and_then(Value::as_array)
    else {
        projection.note_unmapped("root:missing_grammar_file_elements".to_string());
        return projection;
    };

    for element in elements {
        match element.get("type").and_then(Value::as_str) {
            Some("grammar_rule") => {
                let rule = project_grammar_rule(element, &mut projection);
                projection.rules.push(rule);
            }
            // Arm 1 drops comments entirely, so an arm-2 comment element is an expected
            // asymmetry, not a divergence.
            Some("line_comment") | Some("block_comment") | Some("whitespace") => {
                projection.skipped_non_rule_elements += 1;
            }
            // Arm 1 RESOLVES includes; arm 2 only parses the directive. Counted and reported —
            // see `Projection::unresolved_include_directives`.
            Some("include_directive") => projection.unresolved_include_directives += 1,
            Some(other) => projection.note_unmapped(format!("grammar_file_element:{}", other)),
            None => projection.note_unmapped("grammar_file_element:<untyped>".to_string()),
        }
    }

    projection
}

fn project_grammar_rule(element: &Value, projection: &mut Projection) -> RuleEnvelope {
    let mut tokens = Vec::new();
    let Some(rule) = element.get("rule") else {
        projection.note_unmapped("grammar_rule:missing_rule".to_string());
        return RuleEnvelope { name: None, tokens };
    };

    let name = rule
        .pointer("/name/name")
        .and_then(Value::as_str)
        .map(str::to_string);
    match name.as_deref() {
        Some(rule_name) => tokens.push(EnvelopeToken::new("rule", json!(rule_name))),
        None => projection.note_unmapped("rule_definition:missing_name".to_string()),
    }

    // `annotation_list? rule_definition` — annotations that PRECEDE the rule attach to it.
    for annotation in optional_list(element.get("annotations")) {
        project_semantic_annotation(annotation, "semantic_annotation", &mut tokens, projection);
    }

    project_expression(rule.get("expression"), &mut tokens, projection);

    // `return_annotation?` — an absent optional serializes as an empty array, a present one as
    // the node itself.
    if let Some(return_annotation) = optional_node(rule.get("return_annotation")) {
        project_return_annotation(return_annotation, &mut tokens, projection);
    }

    RuleEnvelope { name, tokens }
}

fn project_semantic_annotation(
    annotation: &Value,
    kind: &str,
    tokens: &mut Vec<EnvelopeToken>,
    projection: &mut Projection,
) {
    if annotation.get("type").and_then(Value::as_str) != Some("semantic_annotation") {
        projection.note_unmapped(format!(
            "annotation:{}",
            annotation
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or("<untyped>")
        ));
        return;
    }

    let annotation_name = annotation
        .pointer("/name/name")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let value = annotation.get("value");
    let payload = match value
        .and_then(|value| value.get("type"))
        .and_then(Value::as_str)
    {
        // `@name: value` — the payload runs to end of line.
        Some("line_payload") => value
            .and_then(|value| value.get("text"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        // `@name: { … }` — LANG-CAPABILITY-AUDIT.10.2's opaque-payload delimiter form. The body
        // is carried as a list of source fragments; joining them reconstructs the payload arm 1
        // slurps in one piece.
        Some("braced_payload") => value
            .and_then(|value| value.get("body"))
            .map(join_source_fragments)
            .unwrap_or_default(),
        other => {
            projection.note_unmapped(format!(
                "annotation_payload:{}",
                other.unwrap_or("<untyped>")
            ));
            return;
        }
    };

    // Arm 1 emits the annotation as a NESTED pair: `["semantic_annotation", [name, payload]]`.
    tokens.push(EnvelopeToken::new(
        kind,
        json!([annotation_name, payload.trim_end()]),
    ));
}

fn project_expression(
    expression: Option<&Value>,
    tokens: &mut Vec<EnvelopeToken>,
    projection: &mut Projection,
) {
    let Some(expression) = expression else {
        return;
    };

    match expression {
        // Bare punctuation the shaped AST carries through verbatim. The alternation separator is
        // a REAL token in arm 1's envelope and must be projected AT THE POSITION arm 2 carries
        // it, not synthesized between alternatives: with per-branch return annotations
        // (`a := x -> $1 | y -> $2`) arm 2 stores branch x's `->` at the head of the NEXT
        // alternative, ahead of the `|`. Synthesizing the separator first put every such rule
        // one token out of step and reported ~100 false divergences across the tracked
        // grammars. Everything else here (`"]"`, `"}"`, `","`) is return-expression interior,
        // which this walk never enters.
        Value::String(text) => {
            if text == "|" {
                tokens.push(EnvelopeToken::new("operator", json!("|")));
            }
        }
        // The shaped carrier wraps repeated groups in anonymous arrays; walk through them.
        Value::Array(items) => {
            for item in items {
                project_expression(Some(item), tokens, projection);
            }
        }
        Value::Object(_) => project_expression_node(expression, tokens, projection),
        other => projection.note_unmapped(format!("expression:<{}>", json_type_name(other))),
    }
}

fn project_expression_node(
    node: &Value,
    tokens: &mut Vec<EnvelopeToken>,
    projection: &mut Projection,
) {
    let node_type = node.get("type").and_then(Value::as_str);
    match node_type {
        // The `|` separators are NOT synthesized here — each alternative after the first carries
        // its own literal `"|"`, and the walk emits it where arm 2 actually stores it. See the
        // `Value::String` arm of `project_expression`.
        Some("alternation") => {
            for alternative in optional_list(node.get("alternatives")) {
                project_expression(Some(alternative), tokens, projection);
            }
        }
        Some("sequence") => {
            for element in optional_list(node.get("elements")) {
                project_expression(Some(element), tokens, projection);
            }
        }
        Some("non_terminal") => match node.pointer("/name/name").and_then(Value::as_str) {
            Some(name) => tokens.push(EnvelopeToken::new("rule_reference", json!(name))),
            None => projection.note_unmapped("non_terminal:missing_name".to_string()),
        },
        Some("quantified") => {
            project_expression(node.get("element"), tokens, projection);
            project_quantifier(node.get("quantifier"), tokens, projection);
        }
        Some("grouped") => {
            tokens.push(EnvelopeToken::new("group_open", json!("(")));
            project_expression(node.get("expression"), tokens, projection);
            tokens.push(EnvelopeToken::new("group_close", json!(")")));
        }
        Some("lookahead") => {
            // Arm 1 emits the operator BEFORE its operand (`anchor !quantifier` becomes
            // `["rule_reference","anchor"], ["operator","!"], ["rule_reference","quantifier"]`).
            match node.get("operator").and_then(Value::as_str) {
                Some(operator) => tokens.push(EnvelopeToken::new("operator", json!(operator))),
                None => projection.note_unmapped("lookahead:missing_operator".to_string()),
            }
            project_expression(node.get("expression"), tokens, projection);
        }
        Some("quoted_string") => {
            // Arm 2 keeps the source delimiters and the source escapes; arm 1 strips the former
            // and decodes the latter. Normalizing here — through arm 1's OWN decoder — is what
            // keeps a representational difference from reading as a divergence.
            match node.get("value").and_then(Value::as_str) {
                Some(literal) => tokens.push(EnvelopeToken::new(
                    "quoted_string",
                    json!(decode_quoted_source_literal(literal)),
                )),
                None => projection.note_unmapped("quoted_string:missing_value".to_string()),
            }
        }
        Some("raw_string") => match node.get("value").and_then(Value::as_str) {
            // `r"…"` — no escape processing in either arm.
            Some(literal) => tokens.push(EnvelopeToken::new("quoted_string", json!(literal))),
            None => projection.note_unmapped("raw_string:missing_value".to_string()),
        },
        Some("regex") => match node.get("pattern").and_then(Value::as_str) {
            Some(pattern) => tokens.push(EnvelopeToken::new("regex", json!(pattern))),
            None => projection.note_unmapped("regex:missing_pattern".to_string()),
        },
        // An annotation reached through the EXPRESSION rather than through `annotation_list` is
        // arm 2 binding it to a different rule than arm 1 does. Project it at its arm-2 position
        // so the differential LOCATES the rebinding instead of hiding it.
        Some("semantic_annotation") => {
            project_semantic_annotation(node, "semantic_annotation", tokens, projection);
        }
        // A PER-BRANCH return annotation: `a := x -> $1 | y -> $2` puts one `->` inside each
        // alternative, so it reaches the walk through the expression rather than through
        // `rule_definition.return_annotation`. Arm 1 emits it inline at exactly this position.
        Some("return_annotation") => project_return_annotation(node, tokens, projection),
        Some("line_comment") | Some("block_comment") | Some("whitespace") => {}
        Some(other) => projection.note_unmapped(format!("expression:{}", other)),
        None => projection.note_unmapped("expression:<untyped>".to_string()),
    }
}

fn project_quantifier(
    quantifier: Option<&Value>,
    tokens: &mut Vec<EnvelopeToken>,
    projection: &mut Projection,
) {
    let Some(quantifier) = optional_node(quantifier) else {
        return;
    };
    match quantifier.get("type").and_then(Value::as_str) {
        Some("simple_quantifier") => match quantifier.get("symbol").and_then(Value::as_str) {
            Some(symbol) => tokens.push(EnvelopeToken::new("operator", json!(symbol))),
            None => projection.note_unmapped("simple_quantifier:missing_symbol".to_string()),
        },
        // `{m,n}` — arm 1 emits it as a single `["quantifier", "m,n"]` token.
        Some("bounded_quantifier") => {
            let min = quantifier.pointer("/bounds/min").and_then(Value::as_str);
            let max = quantifier.pointer("/bounds/max").and_then(Value::as_str);
            match (min, max) {
                (Some(min), Some(max)) => tokens.push(EnvelopeToken::new(
                    "quantifier",
                    json!(format!("{},{}", min, max)),
                )),
                _ => projection.note_unmapped("bounded_quantifier:missing_bounds".to_string()),
            }
        }
        other => projection.note_unmapped(format!("quantifier:{}", other.unwrap_or("<untyped>"))),
    }
}

fn project_return_annotation(
    return_annotation: &Value,
    tokens: &mut Vec<EnvelopeToken>,
    projection: &mut Projection,
) {
    let expression_type = return_annotation
        .pointer("/expression/type")
        .and_then(Value::as_str);

    // The mapping is EXHAUSTIVE by design rather than defaulted: a return form nobody has taught
    // this table about must surface as `unmapped`, not be silently bucketed as a scalar and
    // counted as agreement.
    let kind = match expression_type {
        Some("object_return") => "return_object",
        Some("array_return") => "return_array",
        Some(
            "positional_reference"
            | "named_reference"
            | "flatten_reference"
            | "quantified_reference"
            | "property_access"
            | "extraction"
            | "literal_return"
            | "quoted_string"
            | "boolean"
            | "integer"
            | "float"
            | "null",
        ) => "return_scalar",
        other => {
            projection.note_unmapped(format!(
                "return_expression:{}",
                other.unwrap_or("<untyped>")
            ));
            return;
        }
    };

    // Payload deliberately absent — arm 1 carries the raw source text of the return expression
    // and arm 2 carries it already parsed. See the module docs.
    tokens.push(EnvelopeToken::kind_only(kind));
}

/// Strip arm 2's retained source delimiters and decode the escapes, using the hand-written
/// frontend's OWN decoder so the two arms cannot disagree through two implementations of the
/// same rule.
fn decode_quoted_source_literal(literal: &str) -> String {
    let mut chars = literal.chars();
    let opening = chars.next();
    let body = match opening {
        Some(quote @ ('"' | '\'')) => literal
            .strip_prefix(quote)
            .and_then(|rest| rest.strip_suffix(quote))
            .unwrap_or(literal),
        _ => literal,
    };
    crate::ebnf_frontend::decode_quoted_literal_body(body)
}

/// Read an optional node that the shaped AST may serialize either as the node itself or — when
/// the `?` optional did not match — as an empty array.
fn optional_node(value: Option<&Value>) -> Option<&Value> {
    match value {
        Some(Value::Array(items)) => items.first(),
        Some(Value::Null) | None => None,
        other => other,
    }
}

fn optional_list(value: Option<&Value>) -> Vec<&Value> {
    match value {
        Some(Value::Array(items)) => items.iter().collect(),
        Some(Value::Null) | None => Vec::new(),
        Some(single) => vec![single],
    }
}

/// Join a shaped `body` carrier — a nested array of source fragments — back into one string.
fn join_source_fragments(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Array(items) => items.iter().map(join_source_fragments).collect(),
        _ => String::new(),
    }
}

fn json_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Read the hand-written frontend's `raw_ast` envelope into the comparison form.
pub fn read_arm1_envelope(raw_ast_envelope: &Value) -> Vec<RuleEnvelope> {
    raw_ast_envelope
        .get("raw_ast")
        .and_then(Value::as_array)
        .map(|rules| rules.iter().map(RuleEnvelope::from_arm1).collect())
        .unwrap_or_default()
}

/// Diff the two arms and produce the located, counted verdict.
pub fn diff_envelopes(
    grammar: &str,
    arm1: &[RuleEnvelope],
    projection: &Projection,
) -> DifferentialReport {
    let arm2 = &projection.rules;
    let arm1_names: Vec<Option<&String>> = arm1.iter().map(|rule| rule.name.as_ref()).collect();
    let arm2_names: Vec<Option<&String>> = arm2.iter().map(|rule| rule.name.as_ref()).collect();

    let mut report = DifferentialReport {
        grammar: grammar.to_string(),
        arm1_rules: arm1.len(),
        arm2_rules: arm2.len(),
        rule_name_sequence_identical: arm1_names == arm2_names,
        tokens_compared: 0,
        token_matches: 0,
        payload_not_comparable: 0,
        payload_divergences: 0,
        kind_divergences: 0,
        tokens_unverified: 0,
        divergences: Vec::new(),
        divergence_total: 0,
        unmapped_arm2_constructs: projection.unmapped.clone(),
        skipped_non_rule_elements: projection.skipped_non_rule_elements,
        unresolved_include_directives: projection.unresolved_include_directives,
        // Recomputed once every count is final, immediately before the report is returned.
        is_envelope_equivalent: false,
    };

    // Both arms emit rules in source order, so index pairing is correct exactly when the name
    // sequences agree — which the report states either way.
    for rule_index in 0..arm1.len().max(arm2.len()) {
        let left = arm1.get(rule_index);
        let right = arm2.get(rule_index);
        let rule_name = left
            .and_then(|rule| rule.name.clone())
            .or_else(|| right.and_then(|rule| rule.name.clone()));

        let empty: Vec<EnvelopeToken> = Vec::new();
        let left_tokens = left.map(|rule| &rule.tokens).unwrap_or(&empty);
        let right_tokens = right.map(|rule| &rule.tokens).unwrap_or(&empty);

        let mut diverged = false;
        for token_index in 0..left_tokens.len().max(right_tokens.len()) {
            let arm1_token = left_tokens.get(token_index);
            let arm2_token = right_tokens.get(token_index);

            // One inserted or dropped token desynchronizes every position after it. Counting
            // those as disagreement would inflate the gap; counting them as agreement would
            // hide it. They are UNVERIFIED, and reported under their own name.
            if diverged {
                report.tokens_unverified += 1;
                continue;
            }

            let comparison = compare_tokens(arm1_token, arm2_token);
            report.tokens_compared += 1;
            match comparison {
                TokenComparison::Match => report.token_matches += 1,
                TokenComparison::PayloadNotComparable => report.payload_not_comparable += 1,
                TokenComparison::PayloadDivergence => report.payload_divergences += 1,
                TokenComparison::KindDivergence => {
                    report.kind_divergences += 1;
                    // A payload disagreement is local; a KIND disagreement means the two token
                    // streams have parted company, so nothing after it can be trusted.
                    diverged = true;
                }
            }

            if matches!(
                comparison,
                TokenComparison::PayloadDivergence | TokenComparison::KindDivergence
            ) {
                report.divergence_total += 1;
                if report.divergences.len() < MAX_REPORTED_DIVERGENCES {
                    report.divergences.push(Divergence {
                        rule_index,
                        rule_name: rule_name.clone(),
                        token_index,
                        comparison,
                        arm1: arm1_token.cloned(),
                        arm2: arm2_token.cloned(),
                    });
                }
            }
        }
    }

    report.is_envelope_equivalent = report.compute_envelope_equivalence();
    report
}

fn compare_tokens(arm1: Option<&EnvelopeToken>, arm2: Option<&EnvelopeToken>) -> TokenComparison {
    let (Some(arm1), Some(arm2)) = (arm1, arm2) else {
        return TokenComparison::KindDivergence;
    };
    if arm1.kind != arm2.kind {
        return TokenComparison::KindDivergence;
    }
    match (&arm1.payload, &arm2.payload) {
        (_, None) => TokenComparison::PayloadNotComparable,
        (Some(left), Some(right))
            if canonical_payload(&arm1.kind, left) == canonical_payload(&arm2.kind, right) =>
        {
            TokenComparison::Match
        }
        (None, Some(_)) => TokenComparison::PayloadDivergence,
        _ => TokenComparison::PayloadDivergence,
    }
}

/// Normalize a payload to the form both arms can be held to.
///
/// Today this touches exactly one kind: a **semantic annotation's payload is opaque text**, and
/// the two arms delimit it differently — arm 1 keeps the source `{ … }` braces around a braced
/// payload, arm 2 hands back the interior. Comparing them raw reports a representational
/// difference as a divergence. Normalizing strips the brace delimiter and the whitespace
/// immediately inside it, so what is compared is the payload's CONTENT.
///
/// ⚠️ The bound is deliberate and narrow: only the outermost delimiter and the whitespace at the
/// two ends are ignored. Interior whitespace, ordering and every character of the payload body
/// still have to match exactly.
fn canonical_payload(kind: &str, payload: &Value) -> Value {
    if !matches!(
        kind,
        "semantic_annotation" | "semantic_annotation_inline" | "semantic_annotation_mid_sequence"
    ) {
        return payload.clone();
    }
    // The payload is arm 1's nested `[name, text]` pair.
    let Some([name, text]) = payload.as_array().map(Vec::as_slice) else {
        return payload.clone();
    };
    let Some(text) = text.as_str() else {
        return payload.clone();
    };
    let trimmed = text.trim();
    let body = trimmed
        .strip_prefix('{')
        .and_then(|rest| rest.strip_suffix('}'))
        .unwrap_or(trimmed)
        .trim();
    json!([name, body])
}

// ---------------------------------------------------------------------------------------------
// Ground truth
// ---------------------------------------------------------------------------------------------

/// The POSITIVE control: a synthetic grammar exercising every construct the projection maps —
/// rule names, a preceding semantic annotation, alternation, grouping, both quantifier forms, a
/// negative lookahead, quoted strings (both delimiters, and an escape), a regex terminal, and
/// all three return-annotation kinds.
///
/// Both arms MUST project this to an exactly-identical envelope. If they do not, the PROJECTION
/// is wrong and every number this module produces is meaningless — so the instrument refuses.
pub fn positive_control_grammar() -> &'static str {
    // ⚠️ Deliberately free of a regex terminal followed by an identifier starting with
    // `[gimsuyx]`: that combination is a LIVE, tracked defect in `grammars/ebnf.ebnf`
    // (LANG-CAPABILITY-AUDIT.10.13), and a control's job is to prove the projection faithful,
    // not to re-assert a defect that is already measured and owned elsewhere.
    concat!(
        "@entry: true\n",
        "control_root := control_alternatives control_terminals\n",
        "    -> {type: \"control_root\", head: $1, tail: $2}\n",
        "\n",
        "control_alternatives := (control_atom | \"literal\" | 'single')* control_atom{0,4}\n",
        "    -> [$1*, $2]\n",
        "\n",
        "control_terminals := /[a-z]+/ !control_atom \"esc\\\\aped\"?\n",
        "    -> $1\n",
        "\n",
        "control_atom := \"atom\"\n",
        "    -> $1\n",
    )
}

/// The NEGATIVE control: the index of the arm-1 token the differ must be shown to catch when it
/// is mutated. Token 2 of rule 0 is `["rule_reference", "control_alternatives"]`.
pub const NEGATIVE_CONTROL_RULE_INDEX: usize = 0;
/// The token index mutated by [`negative_control_mutation`].
pub const NEGATIVE_CONTROL_TOKEN_INDEX: usize = 2;

/// Plant one payload divergence in a copy of arm 1's envelope. The differ MUST report exactly
/// one divergence, at exactly [`NEGATIVE_CONTROL_RULE_INDEX`]/[`NEGATIVE_CONTROL_TOKEN_INDEX`].
/// A clean verdict here means the differ is blind and its parity claims are worthless.
pub fn negative_control_mutation(arm1: &[RuleEnvelope]) -> Vec<RuleEnvelope> {
    let mut mutated = arm1.to_vec();
    if let Some(token) = mutated
        .get_mut(NEGATIVE_CONTROL_RULE_INDEX)
        .and_then(|rule| rule.tokens.get_mut(NEGATIVE_CONTROL_TOKEN_INDEX))
    {
        token.payload = Some(json!("a_name_no_arm_produces"));
    }
    mutated
}

/// The outcome of the two ground-truth controls.
#[derive(Debug, Clone, Serialize)]
pub struct GroundTruthOutcome {
    pub positive_control_report: DifferentialReport,
    pub negative_control_divergences: usize,
}

/// Run both controls against the LIVE arms and refuse on either miss.
///
/// `arm1_envelope` and `arm2_ast` must be the two arms' output for
/// [`positive_control_grammar`]. Returns `Err` with the reason when a control fails — the caller
/// is expected to abort rather than publish numbers from an unvalidated instrument.
pub fn run_ground_truth_controls(
    arm1_envelope: &Value,
    arm2_ast: &Value,
) -> Result<GroundTruthOutcome, String> {
    let arm1 = read_arm1_envelope(arm1_envelope);
    if arm1.is_empty() {
        return Err("positive control: arm 1 produced no rules".to_string());
    }

    let projection = project_meta_parser_ast(arm2_ast);
    let positive = diff_envelopes("<positive-control>", &arm1, &projection);
    if !positive.is_envelope_equivalent {
        return Err(format!(
            "POSITIVE CONTROL FAILED — the projection does not reproduce arm 1 on a grammar it \
             must: {} kind divergence(s), {} payload divergence(s), {} unverified token(s), \
             unmapped {:?}, rule-name sequences identical: {}. First divergence: {}",
            positive.kind_divergences,
            positive.payload_divergences,
            positive.tokens_unverified,
            positive.unmapped_arm2_constructs,
            positive.rule_name_sequence_identical,
            positive
                .divergences
                .first()
                .map(|divergence| format!("{:?}", divergence))
                .unwrap_or_else(|| "<none recorded>".to_string()),
        ));
    }

    let mutated = negative_control_mutation(&arm1);
    let negative = diff_envelopes("<negative-control>", &mutated, &projection);
    if negative.divergence_total != 1 {
        return Err(format!(
            "NEGATIVE CONTROL FAILED — a planted divergence must be caught exactly once, but the \
             differ reported {}. A differ that cannot see a known defect cannot certify parity.",
            negative.divergence_total
        ));
    }
    let located = negative
        .divergences
        .first()
        .map(|divergence| {
            divergence.rule_index == NEGATIVE_CONTROL_RULE_INDEX
                && divergence.token_index == NEGATIVE_CONTROL_TOKEN_INDEX
        })
        .unwrap_or(false);
    if !located {
        return Err(format!(
            "NEGATIVE CONTROL FAILED — the planted divergence was reported at the wrong place: \
             expected rule {} token {}, got {:?}.",
            NEGATIVE_CONTROL_RULE_INDEX, NEGATIVE_CONTROL_TOKEN_INDEX, negative.divergences,
        ));
    }

    Ok(GroundTruthOutcome {
        positive_control_report: positive,
        negative_control_divergences: negative.divergence_total,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal arm-2 AST: one rule `a := b -> $1`.
    fn arm2_single_rule() -> Value {
        json!({
            "rule_name": "grammar_file",
            "content": {"Json": {"type": "grammar_file", "elements": [
                {"type": "grammar_rule", "annotations": [], "rule": {
                    "type": "rule_definition",
                    "name": {"type": "rule_name", "name": "a"},
                    "operator": {"type": "rule_operator", "symbol": ":="},
                    "expression": {"type": "alternation", "alternatives": [
                        {"type": "sequence", "elements": [
                            {"type": "non_terminal", "name": {"type": "rule_name", "name": "b"}}
                        ]}
                    ]},
                    "return_annotation": {"type": "return_annotation", "expression": {
                        "type": "positional_reference", "index": "1"
                    }}
                }}
            ]}}
        })
    }

    fn arm1_single_rule() -> Value {
        json!({"raw_ast": [[["rule", "a"], ["rule_reference", "b"], ["return_scalar", "$1"]]]})
    }

    #[test]
    fn projects_a_single_rule_into_the_arm1_envelope() {
        let projection = project_meta_parser_ast(&arm2_single_rule());
        assert!(projection.unmapped.is_empty(), "{:?}", projection.unmapped);
        assert_eq!(projection.rules.len(), 1);
        let tokens = &projection.rules[0].tokens;
        assert_eq!(tokens[0], EnvelopeToken::new("rule", json!("a")));
        assert_eq!(tokens[1], EnvelopeToken::new("rule_reference", json!("b")));
        // The return annotation compares by KIND only — arm 2 has no source text to offer.
        assert_eq!(tokens[2], EnvelopeToken::kind_only("return_scalar"));
    }

    #[test]
    fn a_matching_pair_reports_envelope_equivalence() {
        let arm1 = read_arm1_envelope(&arm1_single_rule());
        let projection = project_meta_parser_ast(&arm2_single_rule());
        let report = diff_envelopes("fixture", &arm1, &projection);
        assert!(report.is_envelope_equivalent, "{:?}", report.divergences);
        assert_eq!(report.tokens_compared, 3);
        assert_eq!(report.token_matches, 2);
        assert_eq!(report.payload_not_comparable, 1);
        assert_eq!(report.agreement_ratio(), Some(1.0));
    }

    #[test]
    fn a_renamed_reference_is_located_as_a_payload_divergence() {
        let arm1 = read_arm1_envelope(&json!({
            "raw_ast": [[["rule", "a"], ["rule_reference", "DIFFERENT"], ["return_scalar", "$1"]]]
        }));
        let projection = project_meta_parser_ast(&arm2_single_rule());
        let report = diff_envelopes("fixture", &arm1, &projection);
        assert!(!report.is_envelope_equivalent);
        assert_eq!(report.divergence_total, 1);
        assert_eq!(report.payload_divergences, 1);
        let divergence = &report.divergences[0];
        assert_eq!(divergence.token_index, 1);
        assert_eq!(divergence.comparison, TokenComparison::PayloadDivergence);
    }

    #[test]
    fn a_kind_divergence_stops_the_rule_and_marks_the_rest_unverified() {
        // Arm 1 has an extra leading reference, so every later position is shifted.
        let arm1 = read_arm1_envelope(&json!({
            "raw_ast": [[
                ["rule", "a"], ["quoted_string", "x"], ["rule_reference", "b"],
                ["return_scalar", "$1"]
            ]]
        }));
        let projection = project_meta_parser_ast(&arm2_single_rule());
        let report = diff_envelopes("fixture", &arm1, &projection);
        assert_eq!(report.kind_divergences, 1);
        assert_eq!(report.tokens_unverified, 2);
        // The shifted tail is counted as unverified, NOT as agreement.
        assert_eq!(report.tokens_compared, 2);
    }

    #[test]
    fn an_unmapped_construct_defeats_equivalence_instead_of_being_dropped() {
        let mut ast = arm2_single_rule();
        ast["content"]["Json"]["elements"][0]["rule"]["expression"]["alternatives"][0]["elements"]
            [0] = json!({"type": "a_construct_nobody_mapped"});
        let projection = project_meta_parser_ast(&ast);
        assert_eq!(
            projection
                .unmapped
                .get("expression:a_construct_nobody_mapped"),
            Some(&1)
        );
        let arm1 = read_arm1_envelope(&arm1_single_rule());
        let report = diff_envelopes("fixture", &arm1, &projection);
        assert!(!report.is_envelope_equivalent);
    }

    #[test]
    fn quoted_literals_are_normalized_through_arm1s_own_decoder() {
        assert_eq!(decode_quoted_source_literal("\"|\""), "|");
        assert_eq!(decode_quoted_source_literal("'->'"), "->");
        // `"\\Q"` in source is backslash + Q after decoding — the exact shape arm 1 emits.
        assert_eq!(decode_quoted_source_literal("\"\\\\Q\""), "\\Q");
        assert_eq!(decode_quoted_source_literal("\"a\\nb\""), "a\nb");
    }

    #[test]
    fn the_negative_control_mutation_is_caught_exactly_once() {
        let arm1 = read_arm1_envelope(&json!({
            "raw_ast": [[["rule", "a"], ["rule_reference", "b"], ["rule_reference", "c"]]]
        }));
        let projection = Projection {
            rules: arm1.clone(),
            ..Projection::default()
        };
        let mutated = negative_control_mutation(&arm1);
        let report = diff_envelopes("fixture", &mutated, &projection);
        assert_eq!(report.divergence_total, 1);
        assert_eq!(
            report.divergences[0].token_index,
            NEGATIVE_CONTROL_TOKEN_INDEX
        );
    }

    /// Regression pin. `a := x -> $1 | y -> $2` stores branch x's `->` at the HEAD of the next
    /// alternative, ahead of the `|`. Synthesizing the separator between alternatives instead of
    /// projecting the one arm 2 carries put every such rule one token out of step — measured at
    /// ~100 false divergences across the tracked grammars, including all 37 in `vhdl.ebnf`.
    #[test]
    fn a_per_branch_return_annotation_is_projected_before_its_separator() {
        let ast = json!({
            "content": {"Json": {"type": "grammar_file", "elements": [
                {"type": "grammar_rule", "annotations": [], "rule": {
                    "type": "rule_definition",
                    "name": {"type": "rule_name", "name": "a"},
                    "expression": {"type": "alternation", "alternatives": [
                        {"type": "sequence", "elements": [
                            {"type": "non_terminal", "name": {"type": "rule_name", "name": "x"}}
                        ]},
                        [
                            {"type": "return_annotation", "expression": {
                                "type": "positional_reference", "index": "1"
                            }},
                            "|",
                            {"type": "sequence", "elements": [
                                {"type": "non_terminal", "name": {"type": "rule_name", "name": "y"}}
                            ]}
                        ]
                    ]},
                    "return_annotation": {"type": "return_annotation", "expression": {
                        "type": "positional_reference", "index": "2"
                    }}
                }}
            ]}}
        });
        let projection = project_meta_parser_ast(&ast);
        let kinds: Vec<&str> = projection.rules[0]
            .tokens
            .iter()
            .map(|token| token.kind.as_str())
            .collect();
        assert_eq!(
            kinds,
            vec![
                "rule",
                "rule_reference",
                "return_scalar",
                "operator",
                "rule_reference",
                "return_scalar",
            ]
        );

        // The same order arm 1 emits — so the pair must compare clean.
        let arm1 = read_arm1_envelope(&json!({"raw_ast": [[
            ["rule", "a"], ["rule_reference", "x"], ["return_scalar", "$1"],
            ["operator", "|"], ["rule_reference", "y"], ["return_scalar", "$2"]
        ]]}));
        assert!(diff_envelopes("fixture", &arm1, &projection).is_envelope_equivalent);
    }

    /// Regression pin. Arm 1 keeps a braced annotation payload's `{ … }`; arm 2 hands back the
    /// interior. Comparing them raw reported a representational difference as a divergence.
    #[test]
    fn a_braced_annotation_payload_compares_modulo_its_delimiter() {
        let kind = "semantic_annotation";
        let arm1 = json!(["profile_alias", "{ \"2017\": sv_2017 }"]);
        let arm2 = json!(["profile_alias", "\"2017\": sv_2017 "]);
        assert_eq!(
            canonical_payload(kind, &arm1),
            canonical_payload(kind, &arm2)
        );

        // The normalization is narrow: a genuinely different body still diverges.
        let other = json!(["profile_alias", "\"2023\": sv_2023"]);
        assert_ne!(
            canonical_payload(kind, &arm1),
            canonical_payload(kind, &other)
        );

        // And it applies to annotation payloads ONLY — a rule reference that happens to be
        // brace-wrapped is never normalized.
        let reference = json!("{x}");
        assert_eq!(canonical_payload("rule_reference", &reference), reference);
    }

    #[test]
    fn an_absent_optional_serialized_as_an_empty_array_reads_as_absent() {
        assert!(optional_node(Some(&json!([]))).is_none());
        assert!(optional_node(Some(&json!(null))).is_none());
        assert_eq!(
            optional_node(Some(&json!({"type": "x"}))),
            Some(&json!({"type": "x"}))
        );
    }
}
