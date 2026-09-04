// SV-CORPUS-GRAD.13x.d - a CONTROL under the profile it names - must keep ACCEPTING there.
// parameter type enum E_t = e_t; judged as IEEE 1800-2023 - a forward-type restriction WITH its default
// IEEE 1800-2023 A.2.1.1 type_parameter_declaration ::= type [ forward_type ] list_of_type_assignments docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:721, forward_type :803; footnote 22 :3483-3484 - the data_type may be omitted from a type_assignment ONLY within a parameter_port_list, so the body declaration carries its default.
// CORRECTED by SV-CORPUS-GRAD.3.37 (PGEN-SV-CORPUS-GRAD-0366): the .13x.d text `parameter type enum E_t;` had no default and was ruled legal from A.2.1.1 alone; footnote 22 (the 2023 twin of 2017's footnote 18) makes it illegal in a module body, and the footnote-18 parameter tightening of .3.37 exposed it. The two corpus rows the contrast was made for (verilator t_param_type_fwd.v / t_param_type_fwd_bad.v) omit the default in a module body too and are must_reject in both editions; the .14b.2 iverilog sv_type_param_restrict_* rows carry defaults and stand.
module sub;
  typedef enum {A0, A1} e_t;
  parameter type enum E_t = e_t;
endmodule
