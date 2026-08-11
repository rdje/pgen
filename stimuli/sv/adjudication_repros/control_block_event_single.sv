// The one-difference control for fixed_block_event_or.sv (SV-CORPUS-GRAD.13c.2a.4).
// A single block event, no `or` continuation. It parsed BEFORE GRAMMAR-WELLFORMED.A2.5 landed and it
// parses after, so the sibling's REJECT→ACCEPT flip is attributable to the revived `or` alternative
// alone rather than to anything else about `@@( … )`, `covergroup`, or the hierarchical identifier.
// ⛔ It carries no `arm`: the seed alternatives of `block_event_expression` are un-annotated, so
// there is no `kind` to pin. A control that would need an arm it cannot have says so rather than
// implying its ACCEPT proves more than a verdict.
module m;
  logic t;
  covergroup cg @@(begin m.t);
  endgroup
endmodule
