// SV-CORPUS-GRAD.3.44 - CONTROL: IEEE 1800-2017 A.2.6 function_body_declaration alternative 2 - the PORTED form
// `function_data_type_or_implicit function_identifier ( [ tf_port_list ] ) ; { block_item_declaration }
// { function_statement_or_null } endfunction [ : function_identifier ]` - the most common function in any corpus.
// ACCEPT before and after on every profile (legal IEEE 1364-2005 A.2.6 too: function_port_list); through release 1.0.217
// its body arrived as the RAW ten-element Sequence envelope [return_type, name, (, ports, ), ;, items, statements,
// endfunction, label]; since 1.0.218 it is {return_type, name, ports, items, statements, end_label} - `ports` present
// exactly because the form is ported (arm !tf_port: the ports are NOT tf_item_declaration items; `items` holds bare
// block_item_declaration nodes, as A.2.6's second production reads).
// docs/systemverilog/2017/md/section-41-data-read-api.md:1096-1100
module m;
  function integer f(input integer a, input integer b);
    integer x;
    begin
      x = a + b;
      f = x;
    end
  endfunction : f
  initial $display(f(1, 2));
endmodule
