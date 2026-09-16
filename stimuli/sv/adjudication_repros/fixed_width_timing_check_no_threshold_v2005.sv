// SV-CORPUS-GRAD.3.63 — IEEE 1364-2005 A.7.5.1: `$width ( controlled_reference_event , timing_check_limit
// [ , threshold [ , notifier ] ] ) ;` — the threshold is OPTIONAL under 1364-2005 (§15.4's own example,
// ispras ieee-1364-2005/test_15_04_00_1.v). IEEE 1800-2017/2023 A.7.5.1 require it, so this file ACCEPTS
// under verilog_2005 only; the same bytes are invalid_width_timing_check_no_threshold_sv.sv on the SV profiles.
module m(input clr);
  specify
    specparam tWidth = 10;
    $width(posedge clr, tWidth);
  endspecify
endmodule
