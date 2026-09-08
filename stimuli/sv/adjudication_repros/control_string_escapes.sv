// SV-CORPUS-GRAD.3.47 - a CONTROL: every escape form on one line, and the empty string - must keep ACCEPTING on every profile, byte-identical across 1.0.221.
// IEEE 1800-2017 §5.9.1 / IEEE 1364-2005 §3.6.3: \n \t \\ \" \ddd (octal) and the format escapes inside $display; the empty string "".
module m;
  initial begin
    $display("tab \t nl \n bs \\ q \" oct \101 pct %d", 1);
    $display("");
    if ("" == "") $display("empty");
  end
endmodule
