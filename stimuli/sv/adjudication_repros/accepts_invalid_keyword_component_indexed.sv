// SV-CORPUS-GRAD.13c.2k — a KNOWN OVER-ACCEPTANCE (class `accepts_invalid`). `module` is a RESERVED
// keyword (IEEE 1800-2017 Annex B), and §5.6.2 forbids a reserved word as a simple identifier, so
// A.9.3's `hierarchical_identifier ::= { identifier [ [ constant_expression ] ] . } identifier` has
// no derivation for `ral.module[0]`. PGEN accepts it because the component LOOPS spell the raw
// `identifier` rule while only the FINAL component is `non_keyword_identifier`.
// ⭐ The position matrix that pins it: this row and its two `plain`/`first` siblings ACCEPT, while
// `invalid_keyword_method_name.sv`, `invalid_keyword_final_component.sv` and
// `invalid_keyword_declaration.sv` all REJECT — every GUARDED position refuses the same keyword.
// `control_keyword_shaped_member.sv` is the same text with `module` spelled `modxx`, and it parses.
// Expected today: ACCEPT. When .13c.2k lands the runner FAILS with "flip it to invalid".
class inner; function int g(); return 0; endfunction endclass
class rb; inner arr[2]; endclass
class w; function int f(rb ral); return ral.module[0].g(); endfunction endclass
