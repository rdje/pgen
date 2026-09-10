// SV-CORPUS-GRAD.3.58 narrowness lock — a `*` inside an event control's parentheses is the wildcard alone;
// `@( * , a )` is neither `@ ( * )` (IEEE 1800-2023 A.6.5) nor an event_expression (A.6.5: an expression,
// a sequence instance, `or` / `,` lists of those — `*` is no expression).
module t;
  reg x, y, a;
  always @( * , a ) y = x;
endmodule
