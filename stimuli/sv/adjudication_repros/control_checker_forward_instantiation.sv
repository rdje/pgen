// SV-CORPUS-GRAD.3.49 - a CHECKER instantiated BEFORE its declaration (the declaration follows the module, as in Surelog tests/CheckerInst/dut.sv and ispras 14.14_05.sv): legal - IEEE 1800-2017 section 3.2 makes a checker a design element of the compilation unit and section 23.3.2 says an instantiated name may be "previously declared or one declared later" (section-23-modules-and-hierarchy.md:869-870); A.4.1.4 checker_instantiation ::= ps_checker_identifier name_of_instance ( [ list_of_checker_port_connections ] ) ; with A.9.3 checker_identifier ::= identifier (section-41-data-read-api.md:1835).
// Through 1.0.222 a has_fact(checker_name) post gate on the unscoped checker name rejected this at the second actual's `@` (the byte after `p2(a, b, c),`); the actuals are property / sequence syntax no module port admits, so module_instantiation cannot take it.
module top;
  logic a, b, c, clk;
  property p2(req, ack, interrupt);
    @(posedge clk) accept_on(interrupt) req |=> ack;
  endproperty
  my_checker check(p2(a, b, c), @(posedge clk) a[*1:$] ##1 b);
endmodule

checker my_checker(property p, sequence s);
  assert property (p);
  cover property (s);
endchecker
