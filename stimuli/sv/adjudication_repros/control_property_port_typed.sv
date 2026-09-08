// SV-CORPUS-GRAD.3.51 - the twin of invalid_property_port_var_qualifier.sv with a data_type formal: ACCEPT.
module m;
  logic clk;
  property check_if_1(int cyc_mod_2);
    @(posedge clk) cyc_mod_2;
  endproperty
endmodule
