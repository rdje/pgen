// SV-CORPUS-GRAD.32 - FIXED in release 1.0.245 (was an accepts_invalid pin: ACCEPTED through 1.0.244) - now class invalid / REJECT on both SV profiles.
// SV-CORPUS-GRAD.30 - an accepts_invalid pin (ACCEPT today, both SV profiles; NO import): a typedef'd non-class name with a
// parameter list as a begin-end block local is admitted by provisional_unscoped_block_class_type (gated on has_fact(type_name)
// alone - the permissive arm SV-EXH-PROOF.5.2.4.1 kept at the block site while data_type took the typedef-excluded twin).
// `T #(int)` is illegal for a typedef'd int (IEEE 1800-2017 8.25: a parameter value assignment applies to a class). ROUTED to
// SV-CORPUS-GRAD.32; the pin flips to `invalid` / REJECT the day the block site takes the typedef-excluded arm.
typedef int T;
module m;
  initial begin
    T #(int) x;
  end
endmodule
