// SV-CORPUS-GRAD.60 — CORRECT REJECTION under IEEE 1800-2017 — the SV-CORPUS-GRAD.12b.1 ruling. event_trigger ::= -> hierarchical_event_identifier ;
// (A.6.5, docs/systemverilog/2017/md/section-41-data-read-api.md:2191; 15.5 section-15-interprocess-synchronization-and-communication.md:287)
// and hierarchical_identifier ::= [ $root . ] { identifier constant_bit_select . } identifier (A.9.3, :3361): the FINAL identifier
// carries no select. IEEE 1800-2023 adds `nonrange_select` to the trigger (A.6.5, docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2189-2192;
// 15.5 :293): an EDITION CONTRAST; the twin control_event_trigger_select_2023.sv holds that side. The corpus row: OpenTitan
// hw/ip/adc_ctrl/dv/env/adc_ctrl_env_cfg.sv:128 `->m_adc_push_pull_ev[channel];`. Expected on sv_2017 FOREVER: REJECT.
module m;
  event ev[2];
  initial begin
    -> ev[0];
  end
endmodule
