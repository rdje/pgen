// SV-CORPUS-GRAD.13c.2q — `bufif0` is an `enable_gatetype`, not a UDP type name.
// IEEE 1364-2005 A.3.4 / IEEE 1800-2017 A.3.4: enable_gatetype ::= bufif0 | bufif1 | notif0 | notif1
// Before the fix this ACCEPTED through `udp_instantiation` because the grammar held only the
// digit-stripped stem `/bufif\b/`, which no source can match. The verdict never moved; the ARM did.
module m;
  wire o, i, e;
  bufif0 g1(o, i, e);
endmodule
