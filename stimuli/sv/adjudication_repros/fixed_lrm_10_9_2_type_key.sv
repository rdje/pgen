// SV-CORPUS-GRAD.13c.2t — IEEE 1800-2017 §10.9.2's OWN worked example
// (docs/systemverilog/2017/md/section-10-assignment-statements.md).
// Annex A gives `assignment_pattern_key ::= simple_type | default`, and `simple_type` cannot derive
// `string`. The clause text gives `'{data_type: default_value}`. PGEN now derives the UNION, so the
// standard's example parses through the TYPE-key route rather than riding `member_identifier`'s
// acceptance of a reserved word.
module top;
  typedef struct {
    logic [7:0] a;
    bit b;
    bit signed [31:0] c;
    string s;
  } sa;
  sa s2;
  initial s2 = '{int:1, default:0, string:""};
endmodule
