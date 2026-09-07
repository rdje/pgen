// SV-CORPUS-GRAD.3.32 - an INVALID pin: text no Annex A production derives - must keep REJECTING.
// initial begin if (type(a) == type(b)) ...; end - a type-reference comparison as a PROCEDURAL if condition (all four equality operators, then a procedural case on a type reference).
// IEEE 1800-2017 A.6.6 conditional_statement section-41-data-read-api.md:2205-2212 / cond_predicate :2213 / expression_or_cond_pattern :2218-2219 and A.6.7 case_expression :2239 take an expression; A.8.4 primary section-41-data-read-api.md:3122-3140 has no type_reference (constant_primary does, :3109, fn 40 :3560-3562); 6.23 section-65536-216-bits.md:1610-1614 makes the comparison 'a constant expression (see 11.2.1)'; 11.2.1 section-11-operators-and-expressions.md:87-90. Stands for verilator t_type_compare.v:32, t_type_expression_compare.v:14 and sv-tests 6.23--type_op_compare.sv:18. The parser stops on the byte after the first `type(a)`: the static-cast derivation (cast :3186, casting_type :841) wants its ' and meets `==`.
module t;
  int a; real b;
  initial begin
    if (type(a) == type(b)) $stop;
    if (type(a) != type(b)) $stop;
    if (type(a) === type(b)) $stop;
    if (type(a) !== type(b)) $stop;
    case (type(a))
      type(int): ;
      default: $stop;
    endcase
  end
endmodule
