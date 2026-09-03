// SV-CORPUS-GRAD.14b - a CONTROL under the profile it names - must keep ACCEPTING there.
// function void crs(const ref static int i); - a `ref static` task/function port, judged as IEEE 1800-2023
// IEEE 1800-2023 A.2.7 tf_port_direction ::= port_direction | [ const ] ref [ static ] docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1198-1202. the edition contrast: byte-identical text to the sv_2017 invalid pin, legal in 2023.
module t;
  function void crs(const ref static int i);
  endfunction
endmodule
