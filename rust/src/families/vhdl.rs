//! The VHDL family's module (`ENGINE-UNIVERSAL-SERVICES.112` slice 3): the family declares no hook
//! (its parser runs on the engine's neutral defaults) and keeps here the convenience entry points it
//! has always published through the embedding API (`parse_vhdl_1076_2019*`), each one line over the
//! generic, registry-derived API. This module may name its own family and nothing else.

use super::FamilyHooks;

/// No family-specific behaviour: the neutral defaults.
pub const HOOKS: FamilyHooks = FamilyHooks::NONE;

pub mod embed {
    use crate::embedding_api::{self as api, AstDumpOptions, GrammarAstDumpOutcome, GrammarParseOutcome, ParseDiagnostic, ParseLimits};
    const FAMILY: &str = "vhdl";
    const PROFILE: &str = "vhdl_1076_2019";
    pub fn parse_vhdl_1076_2019(input: &str) -> GrammarParseOutcome { api::parse_grammar_profile_named_typed(FAMILY, PROFILE, input, &ParseLimits::default()) }
    pub fn parse_vhdl_1076_2019_with_limits(input: &str, limits: &ParseLimits) -> GrammarParseOutcome { api::parse_grammar_profile_named_typed(FAMILY, PROFILE, input, limits) }
    pub fn parse_vhdl_1076_2019_result(input: &str) -> Result<(), ParseDiagnostic> { api::parse_grammar_profile_named_with_limits_result(FAMILY, PROFILE, input, &ParseLimits::default()) }
    pub fn parse_vhdl_1076_2019_with_limits_result(input: &str, limits: &ParseLimits) -> Result<(), ParseDiagnostic> { api::parse_grammar_profile_named_with_limits_result(FAMILY, PROFILE, input, limits) }
    pub fn parse_vhdl_1076_2019_ast_dump(input: &str, options: &AstDumpOptions) -> GrammarAstDumpOutcome { api::parse_grammar_profile_ast_dump_named_typed(FAMILY, PROFILE, input, &ParseLimits::default(), options) }
    pub fn parse_vhdl_1076_2019_ast_dump_with_limits(input: &str, limits: &ParseLimits, options: &AstDumpOptions) -> GrammarAstDumpOutcome { api::parse_grammar_profile_ast_dump_named_typed(FAMILY, PROFILE, input, limits, options) }
}
