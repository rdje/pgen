// SV-CORPUS-GRAD.3.38 - CONTROL: the UNPACKED `[]` (7.5 dynamic arrays) is legal everywhere and untouched by footnote 20.
// reg legal []; (Surelog tests/UnsizedPacked/dut.sv line 3), a dynamic array of a dynamic array, a queue, a class
// property, a function argument with an unpacked open dimension in an ordinary (non-DPI) function.
// Arm !integer_vector>unsized: no unsized node under a data type's packed dims - which is also the proof that the
// control pin's arm can fail (the same chain must be PRESENT in control_dpi_open_packed_dimension.sv).
// docs/systemverilog/2017/md/section-41-data-read-api.md:1070-1079 (A.2.5 variable_dimension ::= unsized_dimension | ...)
module t;
  reg legal [];
  int a [] [];
  logic [7:0] b [];
  int q [$];
  class C;
    bit [3:0] payload [];
    function void h(input int x []); endfunction
  endclass
  initial begin
    a = new[3];
    legal = new[4];
  end
endmodule
