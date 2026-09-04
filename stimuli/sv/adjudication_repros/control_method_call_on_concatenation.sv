// SV-CORPUS-GRAD.3.37 - a CONTROL - must keep ACCEPTING.
// {foo_s, foo_s}.f1 - a method_call whose root is a concatenation primary and whose body omits the argument list
// IEEE 1800-2017 A.8.2 method_call_root ::= primary | implicit_class_handle docs/systemverilog/2017/md/section-41-data-read-api.md:2968; method_call_body ::= method_identifier { attribute_instance } [ ( list_of_arguments ) ] docs/systemverilog/2017/md/section-41-data-read-api.md:2942-2944
module t;
  typedef logic [3:0] foo_t;
  foo_t foo_s;
  assign bar_s = {foo_s, foo_s}.f1;
endmodule
