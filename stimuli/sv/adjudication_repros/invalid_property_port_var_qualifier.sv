// SV-CORPUS-GRAD.3.51 - `var` on a property formal: `property check_if_1(var int cyc_mod_2);` (verilator t_property_named.v). IEEE 1800-2017 A.2.10 property_port_item ::= { attribute_instance } [ local [ property_lvar_port_direction ] ] property_formal_type formal_port_identifier { variable_dimension } [ = property_actual_arg ] (docs/systemverilog/2017/md/section-41-data-read-api.md:1301-1304) with sequence_formal_type ::= data_type_or_implicit | sequence | untyped (docs/systemverilog/2017/md/section-41-data-read-api.md:1397-1401) - `var` is not a data_type; 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1340-1343, :1420-1424).
// REJECTS at `var` on both SV profiles; the control `(int cyc_mod_2)` ACCEPTS.
module m;
  logic clk;
  property check_if_1(var int cyc_mod_2);
    @(posedge clk) cyc_mod_2;
  endproperty
endmodule
