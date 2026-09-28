//! The regex family's hooks (`ENGINE-UNIVERSAL-SERVICES.112` slice 2). This module may name its
//! own family and nothing else; the engine reaches it only through `HOOKS`.
//!
//! Two facts of this family the registry's template used to carry as a hand-copied helper set:
//! (1) every regex parse runs on a dedicated-stack worker thread of [`STACK_BYTES`] — the regex
//! grammar's recursion depth on nested groups exceeds a default thread stack; (2) an ACCEPTED parse
//! is then held to the PCRE2 compile contract (`regex_compile_validation`,
//! `REGEX-PCRE2-FIDELITY`): a contract failure is an oracle disagreement reported with the
//! family's own message and no invented byte offset (`CORPUS-GRAD-ALL.2.0`).
//!
//! The embedding API's regex path keeps its persistent worker, its nesting ceiling and its inline
//! threshold until slice 3 derives that surface from this module too.

use super::FamilyHooks;
use crate::regex_compile_validation::validate_regex_compile_contract;

/// The dedicated worker stack every regex parse runs on (the value the registry and the embedding
/// API have always used).
pub const STACK_BYTES: usize = 64 * 1024 * 1024;

fn dedicated_stack(_sample: &str) -> Option<usize> {
    Some(STACK_BYTES)
}

fn post_parse(sample: &str) -> Result<(), String> {
    validate_regex_compile_contract(sample).map_err(|err| err.message)
}

/// This family's hooks: the dedicated stack for every parse, the PCRE2 compile contract after an
/// accepted one.
pub const HOOKS: FamilyHooks = FamilyHooks {
    dedicated_stack,
    post_parse,
    ..FamilyHooks::NONE
};
