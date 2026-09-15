// SVPP-EXPANSION.5(j) - an OBJECT macro's `"…`" region is rendered as a string literal, its embedded
// usage expanded (IEEE 1800-2017 §22.5.1: "the expansion shall include the quotation mark, substitution
// of actual arguments, and expansions of embedded macros"). Corpus row: OpenTitan
// hw/ip/otbn/dv/uvm/env/otbn_ssctrl_if.sv:15 - refused at the `{` holding the unrendered region until now.
`define SSCTRL_HPATH tb.dut.u_otbn_core.u_otbn_start_stop_control
`define SSCTRL_PATH  `"`SSCTRL_HPATH`"
interface otbn_ssctrl_if;
  function automatic string resolve_path(string signal);
    return {`SSCTRL_PATH, ".", signal};
  endfunction
endinterface
