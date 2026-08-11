// Control for `invalid_fatal_without_finish_number.sv`: with the finish_number it parses, so
// the elaboration system task itself — and the conditional generate around it — are supported.
module m;
  if (1) begin
    $fatal(1, "boom");
  end
endmodule
