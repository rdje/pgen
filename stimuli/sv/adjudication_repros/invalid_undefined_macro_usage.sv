// SVPP-EXPANSION.6f - an INVALID pin (preprocess=1) - a text-macro usage with no definition in the file or its context; must keep REJECTING on every profile.
// IEEE 1800-2017 22.5.1 gives a usage its meaning by substitution: "Once a text macro name has been defined, it can be used anywhere in the compilation unit where it is defined" (docs/systemverilog/2017/md/section-22-compiler-directives.md:272-273) - no definition, no derivation.
// Before this the usage was kept as written and the SV grammar's raw-lane residue (compiler_directive -> opaque) ACCEPTED the top-level line: 38 pipeline rows read pass over unresolved text (verilator t_preproc_nodef_bad.v, sv2v test/error/undefined_macro.sv, 25 riscv-dv instruction files).
`not_defined
module top;
endmodule
