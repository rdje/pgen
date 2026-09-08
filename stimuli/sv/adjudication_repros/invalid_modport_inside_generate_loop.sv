interface I #(N = 2);
  bit [N-1:0] req;
  for (genvar i = 0; i < N; ++i) begin : mps
    modport client_mp (output .client_req(req[i]));
  end
endinterface
