// SV-CORPUS-GRAD.3.63 — the same two-argument `$width` as fixed_width_timing_check_no_threshold_v2005.sv,
// judged under the SystemVerilog profiles: IEEE 1800-2017 A.7.5.1 (section-41-data-read-api.md:2783-2786)
// and 1800-2023 A.7.5.1 read `$width ( controlled_reference_event , timing_check_limit , threshold [ , [ notifier ] ] ) ;`
// — the threshold is REQUIRED there, so sv_2017 / sv_2023 must keep refusing it. The 1364-2005 admission
// is profile-gated and must not leak.
module m(input clr);
  specify
    specparam tWidth = 10;
    $width(posedge clr, tWidth);
  endspecify
endmodule
