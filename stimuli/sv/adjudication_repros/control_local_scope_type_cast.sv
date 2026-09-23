// SV-CORPUS-GRAD.13c.2m - a CONTROL - ACCEPT from release 1.0.244 (REJECTED through 1.0.243, both SV profiles).
// local::T'(1) inside randomize() with { ... } - the LRM's `local ::` qualifier before a TYPE name (a cast's casting_type).
// IEEE 1800-2017 A.9.3 ps_type_identifier ::= [ local :: | package_scope | class_scope ] type_identifier (footnote 43: local:: only within randomize with constraints, 18.7.1) docs/systemverilog/2017/md/section-41-data-read-api.md; A.8.4 casting_type
// Through 1.0.243 the grammar spelled the footnote SUPERSCRIPT as a literal token (`local :: 43` / `local :: 48`), so this qualifier was dead on legal text.
class c;
  typedef int T;
  rand int x;
  function void f();
    int r;
    r = randomize() with { x == local::T'(1); };
  endfunction
endclass
