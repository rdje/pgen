// SV-CORPUS-GRAD.3.51 - `vectored` on a VARIABLE declaration: `logic vectored [15:0] a = 0;` (sv-tests chapter-6/6.9.2--vector_vectored_inv.sv). IEEE 1800-2017 A.2.1.3 net_declaration ::= net_type [ drive_strength | charge_strength ] [ vectored | scalared ] data_type_or_implicit [ delay3 ] list_of_net_decl_assignments ; (docs/systemverilog/2017/md/section-41-data-read-api.md:805-812) - the keyword follows a net_type, and no data_declaration carries it; 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:785-789); IEEE 1364-2005 A.2.1.3 the same net shape. The suite's own intended failure (a bit-select on a vectored net, 6.9.2) is semantic, but the declaration line is not derivable, which is what the parser stops on.
// REJECTS at `vectored` on every profile; the control `wire vectored [15:0] a;` ACCEPTS on every profile.
module m;
  logic vectored [15:0] a = 0;
endmodule
