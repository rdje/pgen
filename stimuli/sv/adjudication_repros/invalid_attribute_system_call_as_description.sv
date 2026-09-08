// SV-CORPUS-GRAD.3.51 - iverilog's `$attribute(...)` as a top-level item: `$attribute(ATTR_TYPE, "test", "true")` after a typedef (iverilog ivtest/ivltests/sv_type_identifier_attribute_target.v). IEEE 1800-2017 A.1.2 description ::= module_declaration | udp_declaration | interface_declaration | program_declaration | package_declaration | { attribute_instance } package_item | { attribute_instance } bind_directive | config_declaration (docs/systemverilog/2017/md/section-41-data-read-api.md:79-90) - no system task call; 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:64-75); IEEE 1364-2005 A.1.2 description ::= module_declaration | udp_declaration | config_declaration.
// REJECTS at the `$` on every profile; the control without the line ACCEPTS.
typedef int T;
$attribute(T, "test", "true")
module m;
endmodule
