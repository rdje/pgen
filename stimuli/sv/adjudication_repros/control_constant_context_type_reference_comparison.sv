// SV-CORPUS-GRAD.3.32 - a CONTROL: the nearest LEGAL twins (each one token or one context different) - must keep ACCEPTING.
// The same type-reference comparisons where a constant_expression is required - a module-level if / case (A.4.2 conditional_generate_construct / case_generate_construct on a constant_expression, section-41-data-read-api.md:1887-1896), a localparam initializer (A.2.4 param_assignment :1002 / constant_param_expression :2996) - and the one form an ordinary expression admits, the static cast (fn 40; A.8.4 cast :3186, casting_type ::= ... | constant_primary :841), in procedural code.
// IEEE 1800-2017 A.8.4 constant_primary ::= ... | type_reference section-41-data-read-api.md:3109; 6.23 section-65536-216-bits.md:1610-1631.
module t;
  int a; real b; logic [7:0] y; int x;
  typedef logic [12:0] logic12_t;
  localparam bit EQ = (type(a) === type(b));
  localparam int W = (type(a) != type(b)) ? 8 : 16;
  logic [W-1:0] v;
  if (type(a) == type(b)) initial $stop;
  if (type(logic[12:0]) !== type(logic12_t)) initial $stop;
  generate if (type(a) !== type(b)) begin : g end endgenerate
  case (type(real))
    type(int): initial $stop;
    type(real): ;
    default: initial $stop;
  endcase
  initial begin
    x = type(a)'(y);
    if (type(a)'(y) == 3) x = 1;
  end
endmodule
