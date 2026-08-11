//! The left-recursion chain FOLD — the engine's single implementation of
//! "an LR-eliminated rule returns the AST its author declared"
//! (`ENGINE-UNIVERSAL-SERVICES.8`).
//!
//! # Why this module exists
//!
//! PGEN eliminates left recursion for the grammar author (`README.md`'s
//! *EBNF-backed* + *annotation-shaped* doctrines: *"every generated parser
//! returns an AST; return annotations shape that AST"*). The elimination
//! rewrites
//!
//! ```text
//! accessor_base := property_access_expression | array_access_expression | primary
//! property_access_expression := accessor_base "." identifier
//!         -> {type: "property_access", base: $1, property: $3}
//! ```
//!
//! into `accessor_base := accessor_base_lr_base accessor_base_lr_suffix*`, and
//! `rewrite_lr_chain_annotations` (`super::mod`) replaces the rule's declared
//! annotation with a synthetic chain carrying three things: the *seed* value,
//! the per-iteration *suffix* values, and the per-alternative *templates* the
//! author declared.
//!
//! Folding those three back into the author's shape is what this module does.
//! Before `ENGINE-UNIVERSAL-SERVICES.8` **nothing did it**, and the emitted AST
//! for `$1.a.b` was the eliminator's own internal record —
//! `{initial, suffixes, type: "_pgen_lr_chain", wrapper_specs}` — instead of the
//! declared left-nested `{type: "property_access", base: {…}, property: "b"}`.
//!
//! # Why it is ONE function
//!
//! The chain value is built by **three** independent emitters (the protocol
//! graph's [`AstReturnTransformer`](super::ast_return_transform), the fused
//! `cascade_*` graph's direct-value emitter, and the grammar-AST interpreter)
//! and the differential gates only ever prove those three agree *with each
//! other*. Two implementations agreeing is not evidence either one is right —
//! that is precisely how `.8` survived. So the fold is written once, here, and
//! all three call it: they cannot diverge on it by construction, and the thing
//! the gates then check is the property that actually matters (the AST equals
//! the declaration), not mutual agreement.
//!
//! # Shape contract
//!
//! * `initial` — the seed rule's shaped value (already the author's shape).
//! * `suffixes` — a [`PgenValue::Array`] of one object per iteration, each
//!   `{alt_index: Int, captures: Array, type: "_pgen_lr_chain_alt"}`, or
//!   [`PgenValue::Null`] when the quantifier matched zero times.
//! * `specs` — the per-alternative templates, `alt_index`-keyed. `$1` in a
//!   template is the running fold value (the left operand the standard's binary
//!   production names); `$K` for `K >= 2` is `captures[K - 2]`.
//!
//! Everything the fold can encounter is decided at GENERATION time by
//! [`validate_chain_templates`] — a template construct the fold cannot express
//! fails codegen loudly rather than silently emitting a wrong AST at runtime.

use std::sync::{Mutex, OnceLock};

use super::pgen_value::{PgenValue, insert_object_pair};
use super::unified_return_ast::{LrChainWrapperSpec, UnifiedReturnAST};
use super::NodeArena;

/// The `type:` discriminator the eliminator stamps on a chain record.
pub const CHAIN_TYPE_MARKER: &str = "_pgen_lr_chain";

/// The `type:` discriminator the eliminator stamps on one suffix iteration.
pub const CHAIN_ALT_TYPE_MARKER: &str = "_pgen_lr_chain_alt";

/// The prefix every engine-internal `type:` discriminator carries. No grammar
/// author may declare one; the annotation-conformance oracle
/// (`auto_return_annotation_shape_gate`) fails any emitted AST that contains a
/// value carrying it, which is the standing regression guard for this module.
pub const ENGINE_INTERNAL_TYPE_PREFIX: &str = "_pgen_";

/// Parse the codegen-embedded `wrapper_specs` JSON blob.
///
/// The generated parser calls this exactly once per LR-eliminated rule per
/// process (behind its own `OnceLock`), so the per-parse cost is one relaxed
/// atomic load.
pub fn parse_specs(specs_json: &str) -> Vec<LrChainWrapperSpec> {
    serde_json::from_str(specs_json).unwrap_or_else(|err| {
        panic!(
            "ENGINE-UNIVERSAL-SERVICES.8: codegen-embedded LR chain wrapper_specs failed to \
             deserialize ({err}) — the emitting and consuming sides are out of sync: {specs_json}"
        )
    })
}

/// Intern a spec table to `'static` (dedup + one-time leak, bounded by the
/// number of LR-eliminated rules in the grammars a process touches — measured
/// across the 11 shipped grammars: **2**).
///
/// The generated parser does not need this — its table is a `'static` `OnceLock`
/// already — but the grammar-AST interpreter holds its templates behind the
/// gen-AST's lifetime, and [`fold_lr_chain`] borrows template strings directly
/// into the produced [`PgenValue`]s so a fold costs **no** string allocation.
/// This mirrors the interpreter's existing `intern` discipline exactly.
///
/// ⚠️ Dedup is by VALUE over a small registry, deliberately: keying by a
/// serialized form would put a `serde_json::to_string` of the whole table on
/// **every fold**, and keying by the slice's address would be unsound — a freed
/// gen-AST's allocation can be reused by a different table at the same address.
/// The registry holds one entry per distinct table, so the scan is over a
/// single-digit list.
pub fn intern_specs(specs: &[LrChainWrapperSpec]) -> &'static [LrChainWrapperSpec] {
    static INTERN: OnceLock<Mutex<Vec<&'static [LrChainWrapperSpec]>>> = OnceLock::new();
    let registry = INTERN.get_or_init(|| Mutex::new(Vec::new()));
    let mut guard = registry.lock().expect("lr-chain spec intern mutex poisoned");
    if let Some(&existing) = guard.iter().find(|existing| ***existing == *specs) {
        return existing;
    }
    let leaked: &'static [LrChainWrapperSpec] = Box::leak(specs.to_vec().into_boxed_slice());
    guard.push(leaked);
    leaked
}

/// Fold an LR-eliminated rule's chain into the AST its grammar declared.
///
/// Returns `initial` unchanged when there are no suffix iterations — the
/// zero-iteration case is exactly "the seed alternative matched", whose value is
/// already the declared shape.
pub fn fold_lr_chain<'input>(
    arena: &'input NodeArena<'input>,
    initial: PgenValue<'input>,
    suffixes: PgenValue<'input>,
    specs: &'input [LrChainWrapperSpec],
) -> PgenValue<'input> {
    let iterations: &[PgenValue<'input>] = match suffixes {
        PgenValue::Array(items) => items,
        // `Null` is the zero-iteration quantifier result.
        PgenValue::Null => &[],
        other => {
            // Anything else means an emitter handed us something that is not a
            // suffix list — a codegen contract break, not a parse outcome.
            debug_assert!(
                false,
                "ENGINE-UNIVERSAL-SERVICES.8: LR chain `suffixes` must be an Array (or Null for \
                 zero iterations); got {other:?}. An emitter and this fold are out of sync."
            );
            return initial;
        }
    };

    let mut running = initial;
    for iteration in iterations {
        // ⛔ The two `else` arms below cannot happen: the suffix record is built
        // by `rewrite_lr_chain_annotations`' own synthetic annotation and its
        // `alt_index` is allocated from the same table that lands in `specs`.
        // They are `debug_assert!`ed rather than silently skipped because a
        // silent skip would DROP a continuation — producing a truncated AST,
        // which is the very defect class this module exists to close.
        let Some((alt_index, captures)) = read_iteration(iteration) else {
            debug_assert!(
                false,
                "ENGINE-UNIVERSAL-SERVICES.8: LR chain suffix is not the \
                 `{{alt_index, captures}}` record this fold consumes: {iteration:?}"
            );
            continue;
        };
        let Some(spec) = specs.iter().find(|spec| spec.alt_index == alt_index) else {
            debug_assert!(
                false,
                "ENGINE-UNIVERSAL-SERVICES.8: LR chain suffix names alt_index {alt_index}, which \
                 is absent from the {} wrapper template(s) codegen embedded.",
                specs.len()
            );
            continue;
        };
        running = substitute(arena, &spec.annotation_template, running, captures);
    }
    running
}

/// Read one suffix iteration's `(alt_index, captures)` out of its record.
fn read_iteration<'input>(iteration: &PgenValue<'input>) -> Option<(usize, &'input [PgenValue<'input>])> {
    let PgenValue::Object(pairs) = iteration else {
        return None;
    };
    let alt_index = match lookup(pairs, "alt_index")? {
        PgenValue::Int(value) if value >= 0 => value as usize,
        PgenValue::UInt(value) => value as usize,
        // The synthetic annotation routes `alt_index` through
        // `UnifiedReturnAST::NumberLiteral`, so a float-typed integral Number is
        // a legal spelling of the same index.
        PgenValue::Float(value) if value >= 0.0 && value.fract() == 0.0 => value as usize,
        _ => return None,
    };
    let captures = match lookup(pairs, "captures")? {
        PgenValue::Array(items) => items,
        _ => return None,
    };
    Some((alt_index, captures))
}

/// `serde_json::Map::get` over the key-sorted pair slice invariant.
fn lookup<'input>(
    pairs: &'input [(&'input str, PgenValue<'input>)],
    key: &str,
) -> Option<PgenValue<'input>> {
    pairs
        .binary_search_by(|(existing, _)| existing.as_bytes().cmp(key.as_bytes()))
        .ok()
        .map(|index| pairs[index].1)
}

/// Substitute one wrapper alternative's declared template, binding `$1` to the
/// running fold value and `$K` (`K >= 2`) to `captures[K - 2]`.
///
/// Every arm mirrors the value the three emitters build for the same
/// `UnifiedReturnAST` node, so an LR-eliminated rule's AST is indistinguishable
/// from a hand-written rule's.
fn substitute<'input>(
    arena: &'input NodeArena<'input>,
    template: &'input UnifiedReturnAST,
    running: PgenValue<'input>,
    captures: &'input [PgenValue<'input>],
) -> PgenValue<'input> {
    match template {
        UnifiedReturnAST::PositionalRef { index } => match index {
            0 => PgenValue::Null,
            1 => running,
            _ => captures.get(index - 2).copied().unwrap_or(PgenValue::Null),
        },
        UnifiedReturnAST::Passthrough => running,
        UnifiedReturnAST::StringLiteral { value } => PgenValue::Str(value.as_str()),
        UnifiedReturnAST::Identifier { name } => PgenValue::Str(name.as_str()),
        UnifiedReturnAST::NumberLiteral { value } => number_value(*value),
        UnifiedReturnAST::BooleanLiteral { value } => PgenValue::Bool(*value),
        UnifiedReturnAST::NullLiteral => PgenValue::Null,
        UnifiedReturnAST::Object { properties } => {
            let mut sorted: Vec<_> = properties.iter().collect();
            sorted.sort_by(|(left, _), (right, _)| left.cmp(right));
            let mut pairs: Vec<(&'input str, PgenValue<'input>)> = Vec::with_capacity(sorted.len());
            for (key, value) in sorted {
                insert_object_pair(
                    &mut pairs,
                    key.as_str(),
                    substitute(arena, value, running, captures),
                );
            }
            PgenValue::Object(arena.alloc_shaped_pairs(pairs))
        }
        UnifiedReturnAST::Array { elements } => {
            let mut items: Vec<PgenValue<'input>> = Vec::with_capacity(elements.len());
            for element in elements {
                match element {
                    UnifiedReturnAST::Spread { base } | UnifiedReturnAST::FlattenSpread { base } => {
                        match substitute(arena, base, running, captures) {
                            PgenValue::Array(inner) => items.extend_from_slice(inner),
                            other => items.push(other),
                        }
                    }
                    other => items.push(substitute(arena, other, running, captures)),
                }
            }
            PgenValue::Array(arena.alloc_shaped_values(items))
        }
        // Outside an array literal a (flatten-)spread is the shape-preserving
        // identity — the same degeneration the emitters apply.
        UnifiedReturnAST::Spread { base } | UnifiedReturnAST::FlattenSpread { base } => {
            substitute(arena, base, running, captures)
        }
        UnifiedReturnAST::PropertyAccess { base, property } => {
            match substitute(arena, base, running, captures) {
                PgenValue::Object(pairs) => lookup(pairs, property).unwrap_or(PgenValue::Null),
                _ => PgenValue::Null,
            }
        }
        UnifiedReturnAST::ArrayAccess { base, index } => {
            let position = match index.as_ref() {
                UnifiedReturnAST::NumberLiteral { value } => *value as usize,
                _ => 0usize,
            };
            match substitute(arena, base, running, captures) {
                PgenValue::Array(items) => items.get(position).copied().unwrap_or(PgenValue::Null),
                _ => PgenValue::Null,
            }
        }
        // Refused at generation time by `validate_chain_templates`, so these are
        // unreachable. `debug_assert!` makes a future template shape that slips
        // past validation fail LOUDLY in every gate build; the `Null` keeps a
        // release parse total rather than panicking inside a library.
        other @ (UnifiedReturnAST::MatchedText
        | UnifiedReturnAST::QuantifiedExtraction { .. }
        | UnifiedReturnAST::LrChainFold { .. }) => {
            debug_assert!(
                false,
                "ENGINE-UNIVERSAL-SERVICES.8: chain template reached the fold with a construct \
                 `validate_chain_templates` should have refused at generation time: {other:?}"
            );
            PgenValue::Null
        }
    }
}

/// The emitters' number encoding: an integral literal in `i64` range is `Int`
/// (mirroring `dv_number_value_expr` / `generate_transform`'s `NumberLiteral`
/// arm), anything else takes `Value::from(f64)`'s non-finite → `Null` rule.
fn number_value<'input>(value: f64) -> PgenValue<'input> {
    if value.is_finite()
        && value.fract() == 0.0
        && value >= i64::MIN as f64
        && value <= i64::MAX as f64
    {
        PgenValue::Int(value as i64)
    } else {
        PgenValue::from_f64(value)
    }
}

/// GENERATION-TIME validation: every wrapper template must be expressible by
/// [`substitute`], and every positional it names must exist.
///
/// ⛔ This is the honest-bounds leg. A chain template PGEN cannot fold is a
/// missing engine capability, and the right failure is a loud codegen error
/// naming the rule and the construct — never a silently wrong AST, which is the
/// exact defect `.8` exists to close.
pub fn validate_chain_templates(rule: &str, specs: &[LrChainWrapperSpec]) -> Result<(), String> {
    for spec in specs {
        // `$1` is the running fold value; `$2..=original_body_length` are the
        // wrapper body's remaining positions, which become the suffix captures.
        validate_template(rule, spec.alt_index, &spec.annotation_template, spec.original_body_length)?;
    }
    Ok(())
}

fn validate_template(
    rule: &str,
    alt_index: usize,
    template: &UnifiedReturnAST,
    body_length: usize,
) -> Result<(), String> {
    let refuse = |construct: &str| {
        Err(format!(
            "ENGINE-UNIVERSAL-SERVICES.8: rule '{rule}' alternative {alt_index} is left-recursive and \
             its return annotation uses `{construct}`, which the left-recursion chain fold cannot \
             express. Rewrite the annotation, or extend `ast_pipeline::lr_chain_fold::substitute` \
             (and say so in its honest bounds)."
        ))
    };
    match template {
        UnifiedReturnAST::PositionalRef { index } => {
            if *index == 0 {
                return refuse("$0 / $text");
            }
            if *index > body_length {
                return Err(format!(
                    "ENGINE-UNIVERSAL-SERVICES.8: rule '{rule}' alternative {alt_index} references \
                     ${index}, but the left-recursive alternative has only {body_length} element(s)."
                ));
            }
            Ok(())
        }
        UnifiedReturnAST::MatchedText => refuse("$text / $0"),
        UnifiedReturnAST::QuantifiedExtraction { .. } => refuse("$N::target (quantified extraction)"),
        UnifiedReturnAST::LrChainFold { .. } => refuse("a nested left-recursion chain"),
        UnifiedReturnAST::Passthrough
        | UnifiedReturnAST::StringLiteral { .. }
        | UnifiedReturnAST::Identifier { .. }
        | UnifiedReturnAST::NumberLiteral { .. }
        | UnifiedReturnAST::BooleanLiteral { .. }
        | UnifiedReturnAST::NullLiteral => Ok(()),
        UnifiedReturnAST::Object { properties } => properties
            .values()
            .try_for_each(|value| validate_template(rule, alt_index, value, body_length)),
        UnifiedReturnAST::Array { elements } => elements
            .iter()
            .try_for_each(|element| validate_template(rule, alt_index, element, body_length)),
        UnifiedReturnAST::Spread { base }
        | UnifiedReturnAST::FlattenSpread { base }
        | UnifiedReturnAST::PropertyAccess { base, .. } => {
            validate_template(rule, alt_index, base, body_length)
        }
        UnifiedReturnAST::ArrayAccess { base, index } => {
            validate_template(rule, alt_index, base, body_length)?;
            validate_template(rule, alt_index, index, body_length)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(alt_index: usize, body_length: usize, template: UnifiedReturnAST) -> LrChainWrapperSpec {
        LrChainWrapperSpec {
            alt_index,
            original_body_length: body_length,
            annotation_template: template,
        }
    }

    /// `{type: "property_access", base: $1, property: $3}` — the shape
    /// `return_annotation.ebnf`'s `property_access_expression` declares.
    fn property_access_template() -> UnifiedReturnAST {
        let mut properties: std::collections::HashMap<String, Box<UnifiedReturnAST>> =
            std::collections::HashMap::new();
        properties.insert(
            "type".to_string(),
            Box::new(UnifiedReturnAST::StringLiteral {
                value: "property_access".to_string(),
            }),
        );
        properties.insert(
            "base".to_string(),
            Box::new(UnifiedReturnAST::PositionalRef { index: 1 }),
        );
        properties.insert(
            "property".to_string(),
            Box::new(UnifiedReturnAST::PositionalRef { index: 3 }),
        );
        UnifiedReturnAST::Object { properties }
    }

    fn iteration<'input>(
        arena: &'input NodeArena<'input>,
        alt_index: i64,
        captures: Vec<PgenValue<'input>>,
    ) -> PgenValue<'input> {
        PgenValue::Object(arena.alloc_shaped_pairs([
            ("alt_index", PgenValue::Int(alt_index)),
            ("captures", PgenValue::Array(arena.alloc_shaped_values(captures))),
            ("type", PgenValue::Str(CHAIN_ALT_TYPE_MARKER)),
        ]))
    }

    #[test]
    fn zero_iterations_returns_the_seed_unchanged() {
        let arena = NodeArena::new();
        let specs = vec![spec(0, 3, property_access_template())];
        let seed = PgenValue::Str("seed");
        assert_eq!(
            fold_lr_chain(&arena, seed, PgenValue::Array(&[]), &specs),
            seed
        );
        assert_eq!(fold_lr_chain(&arena, seed, PgenValue::Null, &specs), seed);
    }

    #[test]
    fn iterations_fold_left_nested_into_the_declared_shape() {
        let arena = NodeArena::new();
        let specs = intern_specs(&[spec(0, 3, property_access_template())]);
        let suffixes = PgenValue::Array(arena.alloc_shaped_values([
            iteration(&arena, 0, vec![PgenValue::Str("."), PgenValue::Str("a")]),
            iteration(&arena, 0, vec![PgenValue::Str("."), PgenValue::Str("b")]),
        ]));
        let folded = fold_lr_chain(&arena, PgenValue::Str("seed"), suffixes, specs);
        let rendered = serde_json::to_string(&folded).expect("serialize folded value");
        assert_eq!(
            rendered,
            r#"{"base":{"base":"seed","property":"a","type":"property_access"},"property":"b","type":"property_access"}"#,
            "the fold must produce the left-nested binary shape the standard's production describes"
        );
    }

    #[test]
    fn an_unfoldable_template_is_refused_at_generation_time() {
        let refused = validate_chain_templates(
            "expr",
            &[spec(0, 3, UnifiedReturnAST::MatchedText)],
        )
        .expect_err("$text inside a chain template must be refused");
        assert!(refused.contains("$text"), "refusal must name the construct: {refused}");

        let out_of_range =
            validate_chain_templates("expr", &[spec(0, 2, UnifiedReturnAST::PositionalRef { index: 4 })])
                .expect_err("an out-of-range positional must be refused");
        assert!(
            out_of_range.contains("$4"),
            "refusal must name the positional: {out_of_range}"
        );

        validate_chain_templates("expr", &[spec(0, 3, property_access_template())])
            .expect("a declarable object template must validate");
    }

    #[test]
    fn interned_specs_dedup_to_one_leak() {
        let first = intern_specs(&[spec(0, 3, property_access_template())]);
        let second = intern_specs(&[spec(0, 3, property_access_template())]);
        assert!(
            std::ptr::eq(first, second),
            "identical spec tables must intern to the same leaked slice"
        );
    }
}
