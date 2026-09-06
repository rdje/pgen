// SV-CORPUS-GRAD.3.43 - INVALID: `import "DPI-C" function virtual intf f(input int a);` - a virtual interface is not among the eleven admitted types.
// IEEE 1800-2017 §35.5.5 docs/systemverilog/2017/md/section-35-direct-programming-interface.md:485-493 ("Function result
// types are restricted to small values ... void, byte, shortint, int, longint, real, shortreal, chandle, and string;
// scalar values of type bit and logic"); Annex A footnote 21 docs/systemverilog/2017/md/section-41-data-read-api.md:3524
// ("dpi_function_proto return types are restricted to small values, per 35.5.5"); 1800-2023 §35.5.5 :493-501 and
// footnote 26 docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3490. ACCEPTED through 1.0.215.
interface intf; endinterface
module t;
  import "DPI-C" function virtual intf f(input int a);
endmodule
