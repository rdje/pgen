// The one-identifier CONTROL for `accepts_invalid_null_class_qualifier_assign.sv`. REJECTS.
module top;
  int y;
  initial y = null foo_qualifier:=;
endmodule
