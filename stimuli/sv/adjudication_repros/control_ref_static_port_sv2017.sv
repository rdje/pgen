// SV-CORPUS-GRAD.14b - a CONTROL: the 2017 spelling of the same construct, legal in both editions - must keep ACCEPTING.
// function void crs(const ref static int i); - a `ref static` task/function port - the 2017 spelling
// IEEE 1800-2017 A.2.7 tf_port_direction ::= port_direction | const ref docs/systemverilog/2017/md/section-41-data-read-api.md:1159 - no `static`.
module t;
  function void crs(const ref int i);
  endfunction
endmodule
