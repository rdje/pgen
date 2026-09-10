// SV-CORPUS-GRAD.3.58 control — the forms every release accepted: `@*`, `@ *`, `@(*)`, `@ (*)`, `@ (* )`;
// byte-identical before and after the slice.
module t;
  reg x, y;
  always @* y = x;
  always @ * y = x;
  always @(*) y = x;
  always @ (*) y = x;
  always @ (* ) y = x;
endmodule
