// SV-CORPUS-GRAD.13e.2 — REJECT FOREVER under verilog_2005.
// A NET may not carry a data type in IEEE 1364-2005. A.2.1.3 net_declaration
// (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:153) admits only
// `net_type [signed] [delay3] list_of_net_identifiers ;` or a form whose `range` follows the
// net_type directly — so in `wire bool [7:0] b;` the name `bool` can only fill net_identifier and
// `[7:0]` its dimension (A.2.3:221), leaving `b` with no derivation.
// `bool` is a CADENCE EXTENDED TYPE, and the upstream row ivtest/ivltests/br_gh1087b.v enables it
// by hand with `iverilog-args: ["-gxtypes"]` — its golden is an ELABORATION error, so even
// upstream's parser accepted this only with a non-standard extension switched on.
// Expected FOREVER: REJECT on verilog_2005.
module test();
  wire bool [7:0] b;
endmodule
