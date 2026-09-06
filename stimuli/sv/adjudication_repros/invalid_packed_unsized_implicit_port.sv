// SV-CORPUS-GRAD.3.38 - INVALID: an unsized PACKED dimension on an implicit-type port outside a DPI import - REJECTS forever.
// input [] port_a (iverilog ivltests/packed_dims_invalid_module.v) - A.2.2.1 implicit_data_type ::= [ signing ] { packed_dimension }
// reaches the same packed_dimension footnote 20 gates; the ansi-port site is guarded here because the corpus file that
// spells it is rejected on an earlier `[$]` and could never report this site on its own.
// docs/systemverilog/2017/md/section-41-data-read-api.md:1058-1062 (A.2.5), :3522-3523 (footnote 20)
module t (input [] port_a);
endmodule
