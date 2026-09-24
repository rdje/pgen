// SV-CORPUS-GRAD.32 - an IDENTITY CONTROL - ACCEPT before and after release 1.0.245, the typed AST byte-identical (both SV profiles).
// A class-internal scoped type as a function-body local (`C::T x;` inside class C) - the shape the 2026-05-30 comment named as the reason to
// keep the permissive `provisional_unscoped_block_class_type` arm at the block site. Measured on 1.0.244: `block_data_type` selects branch 9/18
// (`scoped_block_type_identifier`, through `class_scope_plain`), not the provisional arm - the permissive arm never served this shape.
// IEEE 1800-2017 A.2.2.1 data_type ::= ... | [ class_scope | package_scope ] type_identifier { packed_dimension }; A.2.8 block_item_declaration
class C;
  typedef int T;
  function void f();
    C::T x;
    x = 1;
  endfunction
endclass
