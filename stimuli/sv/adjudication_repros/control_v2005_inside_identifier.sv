// SV-CORPUS-GRAD.3.41 - CONTROL (verilog_2005): `inside` is an ordinary identifier in IEEE 1364-2005 (not in its Annex B),
// so `wire inside; assign y = a & inside;` ACCEPTS under verilog_2005 - the chain element inside_operator is @profiles-gated to
// the two SystemVerilog profiles and must never be tried there (arm !inside). Under sv_2017 the same text REJECTS: `inside` is
// reserved by IEEE 1800-2017 Annex B (pinned by the SV-RESERVED-WORDS doctrine, SV-CORPUS-GRAD.3.37).
// IEEE 1364-2005 Annex B docs/verilog/2005; IEEE 1800-2017 Annex B Table B.1
module t(input a, output y);
  wire inside;
  assign inside = ~a;
  assign y = a & inside;
endmodule
