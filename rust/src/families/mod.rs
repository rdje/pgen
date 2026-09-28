//! `ENGINE-UNIVERSAL-SERVICES.112` slice 2 — the per-family HOOK modules.
//!
//! The engine serves every generated parser through ONE template (`parser_registry`), and the
//! template never names a language. What is genuinely a fact about one family — the SystemVerilog
//! standard-library bundle a parse is preloaded with, the regex parser's nesting ceiling and its
//! PCRE2 compile contract — lives here, one module per family, each naming ONLY its own family
//! (the `ENGINE-NEUTRALITY` register classes every file under this directory `per-family`), and is
//! reached by the template through [`FamilyHooks`] keyed on the artifact's own `GRAMMAR_NAME`.
//!
//! A family with no module gets [`FamilyHooks::NONE`]: no pre-check, no dedicated stack, no
//! pre-parse, no post-parse — which is exactly what every family other than the two above had in
//! the hand-copied helpers this template replaced.

/// What a family hook may touch on a parser before the parse begins — the surface every
/// generated parser exposes, behind a trait so a hook needs neither the parser's type nor a
/// downcast (a parser borrows its input and its arena, so it can never be type-erased as `Any`).
pub trait ParseTarget {
    fn semantic_runtime_state_mut(&mut self) -> &mut crate::ast_pipeline::SemanticRuntimeState;
}

/// The hook points a family may fill. Every field defaults to a no-op; the template calls them in
/// this order around a parse: `pre_check(sample)` → (`dedicated_stack(sample)` decides where the
/// parse runs) → `pre_parse(&mut parser, resolved_profile)` → the parse → `post_parse(sample)`.
#[derive(Clone, Copy)]
pub struct FamilyHooks {
    /// Refuse an input BEFORE any parser is built (the regex nesting ceiling): `Err(message)` is
    /// the parse's error, positioned by the family's own text.
    pub pre_check: fn(&str) -> Result<(), String>,
    /// `Some(stack_bytes)` when this input must run on a dedicated-stack thread of that size
    /// rather than inline on the caller's stack.
    pub dedicated_stack: fn(&str) -> Option<usize>,
    /// Touch the parser before the parse begins (the SV stdlib preload), through [`ParseTarget`];
    /// the template calls a family's hook only on that family's own rows.
    pub pre_parse: fn(&mut dyn ParseTarget, Option<&str>) -> Result<(), String>,
    /// Judge the input after a SUCCESSFUL parse (the PCRE2 compile contract): `Err(message)` turns
    /// the accepted parse into a failure with the family's own message.
    pub post_parse: fn(&str) -> Result<(), String>,
}

fn no_pre_check(_: &str) -> Result<(), String> {
    Ok(())
}
fn never_dedicated(_: &str) -> Option<usize> {
    None
}
fn no_pre_parse(_: &mut dyn ParseTarget, _: Option<&str>) -> Result<(), String> {
    Ok(())
}
fn no_post_parse(_: &str) -> Result<(), String> {
    Ok(())
}

impl FamilyHooks {
    /// The hooks of a family that declares none.
    pub const NONE: FamilyHooks = FamilyHooks {
        pre_check: no_pre_check,
        dedicated_stack: never_dedicated,
        pre_parse: no_pre_parse,
        post_parse: no_post_parse,
    };
}

// The family modules. Each is compiled only when its own artifact is present (its own cfg), and
// the build-derived table (`$OUT_DIR/generated_parser_table.rs`) names a module's `HOOKS` only for
// the families that have one; every other row carries `FamilyHooks::NONE`.
include!(concat!(env!("OUT_DIR"), "/family_hook_modules.rs"));
