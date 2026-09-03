// SV-CORPUS-GRAD.13x.b - a CONTROL: the nearest LEGAL twin (one token different) - must keep ACCEPTING.
// property p1; case (sel) default: 1'b1; default: 1'b0; endcase endproperty - the golden's duplicate default, 12.5 prose
// IEEE 1800-2017 A.2.10 section-41-data-read-api.md:1337, section-41-data-read-api.md:1363-1365. isolates the defect to the empty item list.
module t; logic [1:0] sel;
  property p1; case (sel) default: 1'b1; default: 1'b0; endcase endproperty
endmodule
