// SV-CORPUS-GRAD.3.52 - the twin of invalid_unary_on_unary_expression.sv with the inner unary parenthesised (a primary): ACCEPT on every profile.
module m;
  reg [7:0] c;
  wire [7:0] d;
  assign d = -(~c);
endmodule
