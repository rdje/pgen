// SV-CORPUS-GRAD.32 - an INVALID pin - REJECT from release 1.0.245 (ACCEPTED through 1.0.244 on both SV profiles: `block_data_type` selected
// branch 13/18, `provisional_unscoped_block_class_type`, gated on has_fact(type_name) alone). `T::P` with T a typedef'd `int` is not a type:
// a scope resolution applies to a class or a package (IEEE 1800-2017 8.23 / 26.3); the data_type site refused it since SV-EXH-PROOF.5.2.4.1
// (the `parameter_typedef_fail` case) while the block site kept the permissive arm.
// IEEE 1800-2017 A.2.2.1 data_type; A.1.9 class_scope ::= class_type ::; 8.25 parameterized classes; 6.18 typedef
typedef int T;
module m;
  initial begin
    T::P x;
  end
endmodule
