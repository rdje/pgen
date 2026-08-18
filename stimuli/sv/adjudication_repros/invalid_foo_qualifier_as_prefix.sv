// The one-identifier CONTROL for `accepts_invalid_class_qualifier_as_prefix.sv`: the same shape
// with `class_qualifier` spelled `foo_qualifier`. It REJECTS, which is why the sibling's ACCEPT
// is attributable to the literal keyword and to nothing else. Filed `invalid` rather than
// `control` because the runner's `control` class means ACCEPT — this text is illegal SV and the
// parser is right to refuse it.
module top;
  int y, m;
  initial y = foo_qualifier m;
endmodule
