// SV-CORPUS-GRAD.3.60 - a TYPE-ASSIGNMENT LIST followed by a package-scoped data type declaration: after the comma,
// `p::PT x = ...` is the next parameter_port_declaration (A.1.3), never a default-less type assignment named `p`. The
// .3.35 guard refused only `<identifier> <identifier>` and `type`; the list now continues only when the identifier after
// the comma is COMPLETE as a type assignment (followed by `,` `)` `;` or `=`).
package p;
  typedef logic [5:0] PT;
endpackage
module m #(parameter type T = int, p::PT x = 6'h2a) ();
endmodule
