// SV-CORPUS-GRAD.3.38 - INVALID: an unsized PACKED dimension outside a DPI import declaration - REJECTS forever.
// reg [] illegal; (iverilog ivltests/br_ml20181012b.v; Surelog tests/UnsizedPacked/dut.sv) - IEEE 1800-2017 A.2.5
// packed_dimension ::= [ constant_range ] | unsized_dimension carries footnote 20: "In packed_dimension, unsized_dimension
// is permitted only as the sole packed dimension in a DPI import declaration". ACCEPTED through 1.0.213 (the unsized arm
// was taken at every site); the UNPACKED `[]` after the identifier (7.5 dynamic arrays) is untouched.
// docs/systemverilog/2017/md/section-41-data-read-api.md:1058-1062 (A.2.5), :3522-3523 (footnote 20)
module t;
  reg [] illegal;
endmodule
