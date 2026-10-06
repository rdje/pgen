// SVPP-EXPANSION.5(q) - a FIXED pin (preprocess=1) - a file that includes ITSELF under a guard nests once and is finite; must keep ACCEPTING on every profile.
// ifndef INCLUDED / define INCLUDED / include "fixed_guarded_self_include.sv" / else ... endif - the shape of sv2v test/lex/file_line.sv; before, any file already on the include stack was refused as an "include cycle"
// IEEE 1800-2017 22.4 "The number of nesting levels for include files shall be finite. Implementations may limit the maximum number of levels to which include files can be nested, but the limit shall be at least 15" (docs/systemverilog/2017/txt/section-22-compiler-directives.txt:96-98); IEEE 1800-2023 22.4 (docs/systemverilog/2023/txt/section-22-compiler-directives.txt:107-109)
`ifndef INCLUDED
`define INCLUDED
`include "fixed_guarded_self_include.sv"
`else
module m;
  wire from_second_pass;
endmodule
`endif
