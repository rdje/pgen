// SV-CORPUS-GRAD.3.38 - CONTROL: the unsized PACKED dimension where footnote 20 permits it - a DPI IMPORT declaration.
// The formal-argument types of 35.5.6.1's own list (`bit[]`, `logic [] arrayNx3 [3:1]`, `bit [] arrayNxN []`,
// `logic [31:0] array32xN []`), a task import, an unnamed formal (footnote 23; slang tests/regression/all.sv:819
// `output logic[]`), a context import with a C alias, and the export forms that carry no prototype at all.
// Accepted before and after; typed AST byte-identical (the scope markers bind the elements the annotations already
// bind or discard). Arm import_function>integer_vector>unsized / import_task>integer_vector>unsized: the `[]` sits
// in the PACKED dims of the data type, not in a variable_dimension after the identifier.
// docs/systemverilog/2017/md/section-35-direct-programming-interface.md:550-582 (35.5.6.1);
// docs/systemverilog/2017/md/section-41-data-read-api.md:3522-3523 (footnote 20)
module t;
  import "DPI-C" function void f1(input bit[]);
  import "DPI-C" function void f2(input logic [] arrayNx3 [3:1], output bit [] arrayNxN []);
  import "DPI-C" function void f3(input logic [31:0] array32xN [], input logic [] scalar_open);
  import "DPI-C" context c_f4 = function int f4(input, output logic[]);
  import "DPI-C" task t1(inout bit unsigned [] d);
  import "DPI" function void f5(input reg [] r);
  export "DPI-C" function g;
  export "DPI-C" c_t2 = task t2;
  function void g; endfunction
  task t2; endtask
endmodule
