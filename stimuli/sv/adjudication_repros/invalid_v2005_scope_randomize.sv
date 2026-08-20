// SV-CORPUS-GRAD.13c.2v — REJECT FOREVER under verilog_2005, and the standing PROFILE guard for
// the SV-0065 fix when it lands.
// `randomize` is IEEE 1800 only; IEEE 1364-2005 has no such construct and no `std` package.
// `primary_sv_2017` is verilog_2005-ADMITTED, so the fix had to be lifted into a profile-gated
// `scope_randomize_sv_only` rule (the `primary_dollar_sv_only` idiom) rather than added inline.
// This row is what will prove the lift held: the same text must stay REJECTED under verilog_2005
// both before the fix (trivially — nothing derives it) and after (because the lift is gated).
// Expected FOREVER: REJECT on verilog_2005.
module m;
  integer a, b;
  initial begin
    if (randomize(a, b) with { a < b; }) $display("ok");
  end
endmodule
