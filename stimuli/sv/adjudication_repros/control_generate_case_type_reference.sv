// SV-CORPUS-GRAD.13x.b - a CONTROL: the nearest LEGAL twin (one token different) - must keep ACCEPTING.
// generate case (type(bus_t)) type(bit[12:0]): ... endcase endgenerate - the standard's own 6.23 example shape, a constant_expression
// IEEE 1800-2017 A.4.2 case_generate_construct ::= case ( constant_expression ) ... section-41-data-read-api.md:1890; A.8.4 constant_primary ::= ... | type_reference section-41-data-read-api.md:3109. isolates the .3.32 question to the procedural context.
module addfixed #(parameter type T = int)(input T a, output T b); endmodule
module t; bit [12:0] A_bus, B_bus; parameter type bus_t = type(A_bus);
  generate
    case (type(bus_t))
      type(bit[12:0]): addfixed #(bus_t) u(A_bus, B_bus);
      default: addfixed #(bus_t) v(A_bus, B_bus);
    endcase
  endgenerate
endmodule
