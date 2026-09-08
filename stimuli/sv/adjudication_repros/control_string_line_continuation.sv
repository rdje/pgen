// SV-CORPUS-GRAD.3.47 - a CONTROL: the standard's own §5.9 example - must keep ACCEPTING under the SV profiles.
// IEEE 1800-2017 §5.9 (docs/systemverilog/2017/md/section-32-h-12ab-f001.md:101-102), 1800-2023 §5.9 (docs/systemverilog/2023/md/section-5-lexical-conventions.md:463-464): a string literal shall be contained in a single line unless the newline character is immediately preceded by a backslash; the backslash and the newline are ignored.
// Through release 1.0.220 the terminal's `\\.` could not match the newline, so this REJECTED at the backslash on every profile (sv-tests 5.9-string-broken-line.sv, verilator t_display.v, Surelog AssumeProp, ispras 05.09_01, verible forbid_line_continuations.sv).
module m;
  initial begin
    $display("Humpty Dumpty sat on a wall. \
Humpty Dumpty had a great fall.");
    $display("broken \
line");
  end
endmodule
