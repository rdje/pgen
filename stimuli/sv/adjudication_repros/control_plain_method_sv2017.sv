// SV-CORPUS-GRAD.13x.d - a CONTROL under the profile it names - must keep ACCEPTING there.
// function int get_e; ... - the 2017 spelling, legal in both editions
// IEEE 1800-2017 A.1.9 class_method docs/systemverilog/2017/md/section-41-data-read-api.md:210-215. isolates the sv_2017 rejection to the :extends specifier.
class C;
  function int get_e; return 1; endfunction
endclass
