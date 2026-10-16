// SV-CORPUS-GRAD.50 (release 1.0.253) — the IEEE 1800 control for the five invalid_v2005_*_two_packed_dims.sv pins and the
// flipped accepts_invalid_v2005_reg_two_packed_dims.sv: the same six shapes (a reg variable, a named-block reg, a function range,
// an ANSI input port, an output reg port, a task port), each with TWO packed dimensions — legal under IEEE 1800-2017 A.2.2.1
// data_type ::= integer_vector_type [ signing ] { packed_dimension } (docs/systemverilog/2017/md/section-41-data-read-api.md:842) and implicit_data_type ::= [ signing ]
// { packed_dimension } (:867); IEEE 1800-2023 A.2.2.1 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:825, :850).
// ACCEPT under sv_2017 / sv_2023; never listed on verilog_2005. packed_dimension_list_sv_only keeps the repetition.
module m(input [7:0][3:0] a, output reg [1:0][1:0] q);
  reg [7:0][3:0] r;
  initial begin : b
    reg [1:0][1:0] s;
  end
  function [1:0][1:0] f;
    input x;
    f = x;
  endfunction
  task t(input [1:0][1:0] v);
    ;
  endtask
endmodule
