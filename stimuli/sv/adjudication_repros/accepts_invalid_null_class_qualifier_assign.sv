// SV-CORPUS-GRAD.13c.2m — the same extraction defect, one layer deeper. The LRM prints
//     | null
//   class_qualifier := [ local ::43 ] [ implicit_class_handle . | class_scope ]
// and the text dump renders `::=` as `:=`, so the extractor saw no production boundary and welded
// the whole `class_qualifier` DEFINITION LINE onto `primary`'s final `| null` alternative.
// This is the minimal firing shape of that welded branch. Isolated by
// `invalid_foo_qualifier_assign.sv`. Expected today: ACCEPT.
module top;
  int y;
  initial y = null class_qualifier:=;
endmodule
