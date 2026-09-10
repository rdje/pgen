// SV-CORPUS-GRAD.3.59 control — slice sizes the standard derives: a simple_type and a constant_expression.
module t;
  bit [5:0] b6;
  bit [1:0] arr2[3];
  initial begin
    { >> bit {arr2}} = b6;
    { >> byte {arr2}} = b6;
    { >> 2 {arr2}} = b6;
    {>> {arr2}} = b6;
    b6 = { >> byte {arr2}};
  end
endmodule
