// SV-CORPUS-GRAD.13c.2m - a CONTROL - ACCEPT from release 1.0.244 (REJECTED through 1.0.243, both SV profiles).
// local::T'{1, 2} inside randomize() with { ... } - the LRM's `local ::` qualifier before a TYPE name (an assignment pattern's type).
// IEEE 1800-2017 A.9.3 ps_type_identifier ::= [ local :: | package_scope | class_scope ] type_identifier (footnote 43; 18.7.1); A.8.7 assignment_pattern_expression_type ::= ps_type_identifier | ...
class c;
  typedef struct { int a; int b; } T;
  rand int x;
  function void f();
    int r;
    r = randomize() with { x == local::T'{1, 2}.a; };
  endfunction
endclass
