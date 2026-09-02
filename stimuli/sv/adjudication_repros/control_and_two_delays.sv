// SV-CORPUS-GRAD.13p — an ACCEPTING CONTROL: two delays on an n-input gate are legal on every profile.
// IEEE 1800-2017 A.3.1 n_input_gatetype [ drive_strength ] [ delay2 ]; §28.16:396 "zero, one, or two delays".
// WHY: pins the narrowing to the THIRD value alone — if delay2 lost its pair alternative this control goes RED.
module test;
  wire a, b, o;
  and #(1,2) g (o, a, b);
endmodule
