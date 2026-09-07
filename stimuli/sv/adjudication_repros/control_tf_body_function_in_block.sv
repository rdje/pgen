// SV-CORPUS-GRAD.3.44 - CONTROL (the alternative that CANNOT move): IEEE 1800-2017 A.2.6 function_body_declaration
// alternative 1 - `function_data_type_or_implicit function_identifier ; { tf_item_declaration } { function_statement_or_null }
// endfunction [ : function_identifier ]` - the NON-ported form, whose ports are tf_item_declaration items INSIDE the body
// (arm tf_port PRESENT). Typed since SV-Slice-25 (release 1.0.25) as {return_type, name, items, statements, end_label};
// release 1.0.218 types alternatives 2-4 and leaves this one byte-identical (no `ports`, no `scope` key). Legal IEEE
// 1364-2005 too (A.2.6 function_declaration with function_item_declaration items).
// docs/systemverilog/2017/md/section-41-data-read-api.md:1090-1100
module m;
  function integer f;
    input integer a;
    integer x;
    begin
      x = a;
      f = x + 1;
    end
  endfunction : f
  initial $display(f(1));
endmodule
