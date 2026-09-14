// SV-CORPUS-GRAD.3.60 - a FUNCTION named after a $unit typedef (iverilog sv_type_identifier_function_name.v:21).
// A.2.6 function_body_declaration ::= function_data_type_or_implicit [ ... ] function_identifier ; with
// function_data_type_or_implicit ::= data_type_or_void | implicit_data_type - the return type is followed by the function
// identifier, so `function U;` is an implicit-typed function named U (3.13: the module name space, not the
// compilation-unit one the typedef lives in).
typedef int U;
module m;
  function U;
    U = 1;
  endfunction
endmodule
