// SV-CORPUS-GRAD.14b - a CONTROL under the profile it names - must keep ACCEPTING there.
// get_str()[1] - a select on a function call's result, judged as IEEE 1800-2023
// IEEE 1800-2023 A.8.4 primary ::= ... | function_subroutine_call [ [ range_expression ] ] | ... docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3079-3085. the edition contrast: byte-identical text to the sv_2017 invalid pin, legal in 2023.
module t;
  function string get_str(); return "ab"; endfunction
  initial begin
    if (get_str()[1] != "a") $stop;
  end
endmodule
