// SV-CORPUS-GRAD.3.43 - CONTROL: every DPI import function RESULT type §35.5.5 admits - the nine keyword types, scalar
// bit / logic (signing admitted: the subclause names types and the word "scalar" and is silent on signing), a typedef
// name, a package-scoped and a class-scoped type identifier and a type reference (what a NAME resolves to is semantic;
// the grammar does not see through a typedef), a context import with a C alias whose FORMAL keeps footnote 20's open
// array, a pure import, a task import and both export forms. Accepted before and after with the typed AST BYTE-IDENTICAL
// (interpreter AST JSON, HEAD grammar vs 1.0.216, both SV profiles). Arms: one `kind` chain per admitted alternative that
// no formal in this file can satisfy (every formal is `int` or the open `bit []`, so `integer_vector>signed`,
// `known_unscoped_data_type`, `scoped_data_type`, `type_reference`, `chandle`, `string`, `non_integer`, `void` can
// come only from a RESULT slot).
// IEEE 1800-2017 §35.5.5 docs/systemverilog/2017/md/section-35-direct-programming-interface.md:485-493 ("Function result
// types are restricted to small values ... void, byte, shortint, int, longint, real, shortreal, chandle, and string;
// scalar values of type bit and logic"); Annex A footnote 21 docs/systemverilog/2017/md/section-41-data-read-api.md:3524
// ("dpi_function_proto return types are restricted to small values, per 35.5.5"); 1800-2023 §35.5.5 :493-501 and
// footnote 26 docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3490.
package p; typedef int int_t; endpackage
class C; typedef int T; endclass
typedef int my_t;
module t;
  import "DPI-C" function void f_void(input int a);
  import "DPI-C" function byte f_byte(input int a);
  import "DPI-C" function byte unsigned f_byte_u(input int a);
  import "DPI-C" function shortint f_shortint(input int a);
  import "DPI-C" function shortint unsigned f_shortint_u(input int a);
  import "DPI-C" function int f_int(input int a);
  import "DPI-C" function int unsigned f_int_u(input int a);
  import "DPI-C" function longint f_longint(input int a);
  import "DPI-C" function longint unsigned f_longint_u(input int a);
  import "DPI-C" function real f_real(input int a);
  import "DPI-C" function shortreal f_shortreal(input int a);
  import "DPI-C" function chandle f_chandle(input int a);
  import "DPI-C" function string f_string(input int a);
  import "DPI-C" function bit f_bit(input int a);
  import "DPI-C" function bit signed f_bit_s(input int a);
  import "DPI-C" function logic f_logic(input int a);
  import "DPI-C" function logic unsigned f_logic_u(input int a);
  import "DPI-C" function my_t f_typedef(input int a);
  import "DPI-C" function p::int_t f_pkg(input int a);
  import "DPI-C" function C::T f_class_scoped(input int a);
  import "DPI-C" function type(1) f_typeref(input int a);
  import "DPI-C" context c_alias = function int f_ctx(input bit [] open, output int b);
  import "DPI-C" pure function int f_pure(input int a);
  import "DPI-C" task t_task(input int a);
  export "DPI-C" function g;
  export "DPI-C" c_h = task h;
  function int g(); return 1; endfunction
  task h; endtask
endmodule
