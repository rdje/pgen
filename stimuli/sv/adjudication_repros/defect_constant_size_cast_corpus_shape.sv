// SV-CORPUS-GRAD.13c.2b — the CORPUS shape of `defect_constant_size_cast.sv`, verbatim from
// OpenTitan `top_darjeeling_rnd_cnst_pkg.sv:240-243` (and the earlgrey twin): a numeric size cast
// wrapping a concatenation, inside a package parameter's constant expression.
// The minimal `8'(1)` reproducer and this one fail for the identical reason — casting_type's
// `constant_primary` alternative is unreachable at the seed position (indirect left recursion,
// ENGINE-UNIVERSAL-SERVICES.13).
// Expected today: REJECT.  Expected once fixed: ACCEPT.
package p;
  parameter logic [511:0] K = 512'({
    64'h45515694E825B33,
    448'h0
  });
endpackage
