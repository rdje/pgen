// SV-CORPUS-GRAD.13c.2t control — the same construct keyed by a genuine MEMBER name.
// It routes through `member_identifier`, not `assignment_pattern_key`, and parsed before the fix
// and after, so the type-key row above is attributable to the `data_type` alternative alone.
module top;
  typedef struct { int x; string s; } sa;
  sa s2;
  initial s2 = '{x:1, s:""};
endmodule
