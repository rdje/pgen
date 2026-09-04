// SV-CORPUS-GRAD.13y - a CONTROL - must keep ACCEPTING.
// localparam type p0_t = p::rq_t; - the qualified type Annex A does admit (a package scope)
// IEEE 1800-2017 A.2.2.1 data_type docs/systemverilog/2017/md/section-41-data-read-api.md:842
package p;
  typedef logic [3:0] rq_t;
endpackage
module t;
  localparam type p0_t = p::rq_t;
endmodule
