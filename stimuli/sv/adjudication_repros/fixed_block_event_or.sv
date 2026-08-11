// SV-CORPUS-GRAD.13c.2a.4 — FIXED by GRAMMAR-WELLFORMED.A2.5. Measured REJECT at PGEN-SV-CORPUS-GRAD-0213,
// ACCEPT here, with ZERO grammar bytes changed between the two.
// IEEE 1800-2017 A.6.11 writes `block_event_expression ::= block_event_expression or
// block_event_expression | begin hierarchical_btf_identifier | end hierarchical_btf_identifier`.
// Alternative 1 is DIRECTLY left-recursive — the self-reference written inline in the choice — so
// before A2.5 it matched no elimination pattern, reached codegen intact, and the runtime cycle guard
// rejected it at the seed position. The rule still parsed a single block event, which is why the
// construct looked supported until something had to follow the first operand.
// ⭐ WHY THIS FILE IS NOT ITS OWN LEAF'S WORK: it is the SECOND rule with the shape, and the engine
// pre-pass revived it for free. That is the entire argument for fixing the engine rather than the
// grammar — a grammar-tier flatten would have fixed `select_expression` and left this one dead.
// Its one-difference control is control_block_event_single.sv.
module m;
  logic t;
  covergroup cg @@(begin m.t or end m.t);
  endgroup
endmodule
