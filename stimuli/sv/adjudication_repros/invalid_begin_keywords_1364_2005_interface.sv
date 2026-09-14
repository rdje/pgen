// SVPP-EXPANSION.8 - §22.14.1's own NEGATIVE example: "This example shall result in errors because
// the identifiers interface and endinterface are not reserved keywords in IEEE Std 1364-2005."
// Unreserved, they are ordinary identifiers, and no IEEE 1800 production derives `\interface if2 ;`
// at module-item position - so the region turns an accepts-invalid row into a rejection.
// Corpus row: ispras-sv-tests 22/22.14.01_04.sv (`// ! TYPE: NEGATIVE`).
`begin_keywords "1364-2005"
module top;
  interface if2;
  endinterface
endmodule
`end_keywords
