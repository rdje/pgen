// SV-CORPUS-GRAD.41 - an INVALID pin - text neither IEEE 1800-2017 nor 1800-2023 derives; must keep REJECTING on every profile.
// extern function automatic int f(); - a LIFETIME in a method prototype
// IEEE 1800-2017 A.2.6 function_prototype ::= function data_type_or_void function_identifier [ ( [ tf_port_list ] ) ] docs/systemverilog/2017/txt/section-41-data-read-api.txt:714; 8.24 docs/systemverilog/2017/txt/section-8-classes.txt:1011-1013; IEEE 1800-2023 docs/systemverilog/2023/txt/section-Annex_A-normative-formal-syntax.txt:741-743, 8.24 docs/systemverilog/2023/txt/section-8-classes.txt:1198
class c;
  extern function automatic int f();
endclass
function automatic int c::f();
  return 1;
endfunction
