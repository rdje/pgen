// Control for `defect_constant_size_cast.sv`: the SAME constant-expression position, but the
// casting_type is a SIMPLE TYPE (`int`), so casting_type matches on branch 1/5 (`simple_type`)
// and never needs its `constant_primary` alternative. Isolates the defect to the ONE casting_type
// alternative the indirect left-recursive cycle makes unreachable.
// IEEE 1800-2017 A.8.4: casting_type ::= simple_type | constant_primary | signing | string | const
// Expected: ACCEPT — today and after the fix.
package p;
  parameter int K = int'(1);
endpackage
