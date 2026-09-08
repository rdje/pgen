interface I;
  bit [1:0] req;
  modport client_mp (output .client_req(req[1]));
endinterface
