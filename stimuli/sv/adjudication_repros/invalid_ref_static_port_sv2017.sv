// SV-CORPUS-GRAD.14b - an INVALID pin under the profile it names: text the 1800-2017 edition derives nothing for - must keep REJECTING there.
// function void crs(const ref static int i); - a `ref static` task/function port, judged as IEEE 1800-2017
// IEEE 1800-2023 A.2.7 tf_port_direction ::= port_direction | [ const ] ref [ static ] docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1198-1202 (2023 only); IEEE 1800-2017 A.2.7 tf_port_direction ::= port_direction | const ref docs/systemverilog/2017/md/section-41-data-read-api.md:1159 - no `static`. verilator t_var_ref_static.v l.12; the same bytes are the sv_2023 control beside this row.
module t;
  function void crs(const ref static int i);
  endfunction
endmodule
