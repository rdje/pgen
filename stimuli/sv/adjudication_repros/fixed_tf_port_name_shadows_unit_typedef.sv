// SV-CORPUS-GRAD.3.60 - a TASK/FUNCTION PORT named after a $unit typedef (iverilog
// sv_type_identifier_task_function_argument_name.v:33). A.2.7 tf_port_declaration ::= { attribute_instance }
// tf_port_direction [ var ] data_type_or_implicit list_of_tf_variable_identifiers ; - a data type is followed by a variable
// identifier, so `input T;` is direction + implicit type + the variable T.
typedef int T;
module m;
  function int f;
    input T;
    return 1;
  endfunction
endmodule
