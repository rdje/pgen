// SV-CORPUS-GRAD.3.52 - an event control on an INDEXED identifier: `@x[y];` (verible kythe testdata/event_control.sv:8). IEEE 1800-2017 A.6.5 event_control ::= @ hierarchical_event_identifier | @ ( event_expression ) | @* | @ (*) | @ ps_or_hierarchical_sequence_identifier (docs/systemverilog/2017/md/section-41-data-read-api.md:2147-2154) with A.9.3 hierarchical_identifier ::= [ $root . ] { identifier constant_bit_select . } identifier (docs/systemverilog/2017/md/section-41-data-read-api.md:3361) - a select sits only on a non-final path element; 1800-2023 event_control ::= clocking_event | @ * | @ ( * ) and clocking_event ::= @ ps_identifier | @ hierarchical_identifier | @ ( event_expression ) (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2140-2151) - the same. `@(x[y])` is the derivable spelling.
// REJECTS at the `[` on both SV profiles; the control `@(x[y]);` ACCEPTS.
module m;
  event x[2];
  int y;
  initial @x[y];
endmodule
