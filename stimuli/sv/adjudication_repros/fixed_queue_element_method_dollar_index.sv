// SV-CORPUS-GRAD.3.59 — a queue ELEMENT selected with `$` as the receiver of a method call.
// IEEE 1800-2017 A.8.4 `primary ::= … | hierarchical_identifier select`, `bit_select ::= { [ expression ] }`,
// and `$` is a primary (7.10.4: `q[$]` is the last element, `q[$-1]` the one before it);
// A.8.2 `method_call ::= method_call_root . method_call_body`, `method_call_root ::= primary | …`.
// Through 1.0.228 the chains' index operand carried only constant_primary, so `q[$-1].delete` rejected at the `$`
// (verilator t_dynarray_multid.v:201 `qa1[$-1].delete;`) while `q[0].delete` and `q[x-1].delete` parsed.
module t;
  int q[$];
  int x;
  initial begin
    q[$-1].delete;
    q[$].delete();
    q[$-1].push_back(1);
    x = q[$ - 1];
  end
endmodule
