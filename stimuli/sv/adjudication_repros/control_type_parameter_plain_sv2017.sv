// SV-CORPUS-GRAD.13x.d - a CONTROL under the profile it names - must keep ACCEPTING there.
// parameter type E_t = int; - the 2017 spelling, legal in both editions
// IEEE 1800-2017 A.2.1.1 parameter_declaration docs/systemverilog/2017/md/section-41-data-read-api.md:739-741. isolates the sv_2017 rejection to the forward type.
module sub;
  parameter type E_t = int;
endmodule
