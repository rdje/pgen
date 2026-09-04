// SV-CORPUS-GRAD.3.37 - an INVALID pin - must keep REJECTING on every profile.
// var final; - the .13z finding: a keyword declared as a variable behind var
// IEEE 1800-2017 Annex B Table B.1 docs/systemverilog/2017/md/section-41-data-read-api.md:3599-3858 (248 reserved words; 1800-2023 Annex B lists the same 248, docs/systemverilog/2023/md/section-Annex_B-normative-keywords.md); 5.6.2 docs/systemverilog/2017/md/section-1024-characters-if-an-identifier-exceeds-the-implementation-specific-length-limit-an-error-shall-be-reported.md:52
module t;
  var final;
endmodule
