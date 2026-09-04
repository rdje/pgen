// SV-CORPUS-GRAD.3.37 - a CONTROL - must keep ACCEPTING.
// this.j, super.f(), super.new(1), C c = new, d = new(), d = new c, null - every legal use of the newly reserved this / super / new / null
// IEEE 1800-2017 Annex B Table B.1 docs/systemverilog/2017/md/section-41-data-read-api.md:3599-3858 (248 reserved words; 1800-2023 Annex B lists the same 248, docs/systemverilog/2023/md/section-Annex_B-normative-keywords.md); 5.6.2 docs/systemverilog/2017/md/section-1024-characters-if-an-identifier-exceeds-the-implementation-specific-length-limit-an-error-shall-be-reported.md:52; A.2.7 docs/systemverilog/2017/md/section-41-data-read-api.md:622-626; A.8.2 method_call_root ::= primary | implicit_class_handle docs/systemverilog/2017/md/section-41-data-read-api.md:2968; A.2.4 class_new docs/systemverilog/2017/md/section-41-data-read-api.md:1028-1034
class B; int i; function new(int a = 0); i = a; endfunction task f(); endtask endclass
class C extends B; int j; function new(); super.new(1); this.j = 2; endfunction task g(); super.f(); this.f(); j = this.i; endtask endclass
module t; C c = new; C d; initial begin d = new(); d = new c; if (d == null) d = null; end endmodule
