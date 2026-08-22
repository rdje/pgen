package p; property myprop; 1; endproperty endpackage
checker chk(untyped a); endchecker
module top; endmodule
bind top chk c1 (myprop);
