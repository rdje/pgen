// SV-CORPUS-GRAD.13c.2q — IEEE 1364-2005 §7.1.6's OWN worked example, the corpus file that
// exposed the defect (ispras-sv-tests/ieee-1364-2005/test_07_01_06_2.v): an ARRAY of three-state
// buffers. This is the row that pins the finding to the standard rather than to a synthetic probe.
module driver(in, out, en);
  input [3:0] in;
  output [3:0] out;
  input en;
  bufif0 ar[3:0](out, in, en);
endmodule
