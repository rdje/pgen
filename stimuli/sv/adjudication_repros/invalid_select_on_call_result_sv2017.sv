// SV-CORPUS-GRAD.14b - an INVALID pin under the profile it names: text the 1800-2017 edition derives nothing for - must keep REJECTING there.
// get_str()[1] - a select on a function call's result, judged as IEEE 1800-2017
// IEEE 1800-2023 A.8.4 primary ::= ... | function_subroutine_call [ [ range_expression ] ] | ... docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3079-3085 (2023 only); IEEE 1800-2017 A.8.4 primary ::= ... | function_subroutine_call | ... docs/systemverilog/2017/md/section-41-data-read-api.md:3122-3128 - no range after a call. verilator t_string_sel.v l.35; the same bytes are the sv_2023 control beside this row.
module t;
  function string get_str(); return "ab"; endfunction
  initial begin
    if (get_str()[1] != "a") $stop;
  end
endmodule
