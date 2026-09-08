// SV-CORPUS-GRAD.3.51 - an event trigger on a PACKAGE-SCOPED event: `#1 ->my_package::e;` (iverilog ivtest/ivltests/sv_wildcard_import2.v, sv_wildcard_import3.v). IEEE 1800-2017 A.6.5 event_trigger ::= -> hierarchical_event_identifier ; | ->> [ delay_or_event_control ] hierarchical_event_identifier ; (docs/systemverilog/2017/md/section-41-data-read-api.md:2191-2194) with A.9.3 hierarchical_identifier ::= [ $root . ] { identifier constant_bit_select . } identifier (docs/systemverilog/2017/md/section-41-data-read-api.md:3361) - no package scope; 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2189-2192, :3310).
// REJECTS at the `::` on both SV profiles; the control `->e;` ACCEPTS.
module m;
  initial #1 ->pkg::e;
endmodule
