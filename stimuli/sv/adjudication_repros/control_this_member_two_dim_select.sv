// SV-CORPUS-GRAD.29 - a CONTROL (ACCEPT from 1.0.241 under BOTH SV profiles) - must keep ACCEPTING.
// A `this.member` with TWO selects: IEEE 1800-2017 A.8.4 primary ::= [ class_qualifier ] hierarchical_identifier select, where A.8.4
// select ::= [ { . member_identifier bit_select } . member_identifier ] bit_select [ [ part_select_range ] ] and bit_select ::=
// { [ expression ] } takes every dimension. Through 1.0.240 BOTH profiles refused it: the sv_2023 twin's LRM-faithful trailing select
// after a call (1800-2023 A.8.4 function_subroutine_call [ [ range_expression ] ]) takes exactly ONE range, and no arm read `this.` as
// the hierarchical primary's class_qualifier.
class c;
  int m2[2][2];
  int id;
  function void f();
    id = this.m2[0][1];
  endfunction
endclass
