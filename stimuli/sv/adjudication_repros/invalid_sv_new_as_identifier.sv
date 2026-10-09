// SV-CORPUS-GRAD.52 — an EDITION CONTRAST: `reg [7:0] new;` is legal IEEE 1364-2005 text (`new` is not a
// Verilog-2005 keyword, Annex B of 1364-2005) and ILLEGAL under IEEE 1800-2017 / 1800-2023, where `new` is a
// reserved keyword (1800-2017 Annex B Table B.1, docs/systemverilog/2017/md/section-41-data-read-api.md:3734)
// and cannot be bound as a variable name. iverilog 13.0 agrees: -g2005 rc 0, -g2012 `syntax error` at this line.
// From ivtest/ivltests/udp_lfsr.v:61. The same text ACCEPTS under verilog_2005: control_v2005_new_as_identifier.sv.
module t;
  reg [7:0] new;
endmodule
