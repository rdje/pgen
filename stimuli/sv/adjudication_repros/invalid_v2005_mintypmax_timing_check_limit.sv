// SV-CORPUS-GRAD.13e.2 — REJECT FOREVER under verilog_2005.
// A timing_check_limit is an `expression`, NOT a mintypmax_expression, so the min:typ:max triple
// `0:0:0` has no derivation there. IEEE 1364-2005 says so twice, in both of its surfaces:
// A.7.5.2 (…/section-Annex_A-normative-formal-syntax-definition.txt:800) and clause 15.5.2
// (docs/verilog/2005/txt/section-15-timing-checks.txt:92). The SAME argument box spends
// mintypmax_expression on end_edge_offset (:84), stamptime_condition (:89) and start_edge_offset
// (:90), so the distinction is deliberate rather than an omission.
// iverilog is a superset here BY ITS OWN GRAMMAR: stimuli/sv/subs/iverilog/parse.y:3746 is
// `delay_value : expression | expression ':' expression ':' expression`.
// Expected FOREVER: REJECT on verilog_2005.
module test;
  wire sig1, sig2;
  specify
    $setup(posedge sig1 , negedge sig2 , 0:0:0);
  endspecify
endmodule
