// SV-CORPUS-GRAD.3.42 - CONTROL: a block-event METHOD identifier with a CLASS SCOPE prefix, IEEE 1800-2017 A.9.3
// `hierarchical_btf_identifier ::= ... | [ hierarchical_identifier . | class_scope ] method_identifier`. The class is
// DECLARED, so `class_scope_type_head` takes `C` as a known class and the scoped branch wins (undeclared, the head
// reads `C::m` as package::class and the text REJECTS at the `)` - the store gate, not an over-rejection).
// ACCEPT before and after. Until release 1.0.214 the `scope` field read `{kind: "block", body: ...}` - the host's
// alternative-2 template wrapped around the class_scope node (ENGINE-UNIVERSAL-SERVICES.96, the inline
// `( hierarchical_identifier dot | class_scope )?` group typed by branch index); since 1.0.215 the alternative is
// spelled `class_scope method_identifier` (its only live form: `a.m` and a bare `m` tie with alternatives 1 and 2 and
// lose) and `scope` is the class_scope node itself. Arm `block_event>begin>method` / `block_event>end>method`: the btf node is the METHOD alternative;
// `!block_event>block`: no `block`-kind node anywhere under a block event (the corrupted scope carried one; the bare
// `@@(begin m)` form, which legitimately parses as the `block` alternative, is deliberately absent from this file).
// docs/systemverilog/2017/md/section-41-data-read-api.md (A.9.3 hierarchical_btf_identifier; A.2.11 coverage_event)
class C;
  task m; endtask
  function void f(); endfunction
endclass
module t;
  int x;
  covergroup cg1 @@(begin C::m);
    coverpoint x;
  endgroup
  covergroup cg2 @@(end C::f);
    coverpoint x;
  endgroup
  covergroup cg3 @@(begin C::m or end C::f);
    coverpoint x;
  endgroup
endmodule
