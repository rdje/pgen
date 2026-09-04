// SV-CORPUS-GRAD.3.37 - an INVALID pin - must keep REJECTING on every profile.
// i.super.i = 1; - super in the MIDDLE of a hierarchical name
// IEEE 1800-2017 Annex B Table B.1 docs/systemverilog/2017/md/section-41-data-read-api.md:3599-3858 (248 reserved words; 1800-2023 Annex B lists the same 248, docs/systemverilog/2023/md/section-Annex_B-normative-keywords.md); 5.6.2 docs/systemverilog/2017/md/section-1024-characters-if-an-identifier-exceeds-the-implementation-specific-length-limit-an-error-shall-be-reported.md:52; A.8.5 variable_lvalue ::= [ implicit_class_handle . | package_scope ] hierarchical_variable_identifier select docs/systemverilog/2017/md/section-41-data-read-api.md:3199-3202 - the handle may only LEAD; implicit_class_handle ::= this | super | this . super docs/systemverilog/2017/md/section-41-data-read-api.md:3156
class Cls;
  int i;
  task t;
    i.super.i = 1;
  endtask
endclass
