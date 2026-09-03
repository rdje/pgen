// SV-CORPUS-GRAD.14b - a CONTROL: the 2017 spelling of the same construct, legal in both editions - must keep ACCEPTING.
// get_str()[1] - a select on a function call's result - the 2017 spelling
// IEEE 1800-2017 A.8.4 primary ::= ... | function_subroutine_call | ... docs/systemverilog/2017/md/section-41-data-read-api.md:3122-3128 - no range after a call.
module t;
  function string get_str(); return "ab"; endfunction
  initial begin
    string s;
    s = get_str();
    if (s[1] != "a") $stop;
  end
endmodule
