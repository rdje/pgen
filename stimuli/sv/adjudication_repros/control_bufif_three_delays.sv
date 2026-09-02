// SV-CORPUS-GRAD.13p — an ACCEPTING CONTROL: three delays on an ENABLE gate are legal on every profile.
// IEEE 1800-2017 A.3.1 enable_gatetype [ drive_strength ] [ delay3 ]; §28.6:631 "zero, one, two, or three delays".
// WHY: pins the narrowing to the delay2 SITES — if the split were applied to the enable gates too, this control goes RED.
module test;
  wire a, e, o;
  bufif0 #(1,2,3) g (o, a, e);
endmodule
