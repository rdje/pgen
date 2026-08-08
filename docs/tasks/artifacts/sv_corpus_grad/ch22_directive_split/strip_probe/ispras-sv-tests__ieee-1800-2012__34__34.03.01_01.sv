// IEEE Std 1800-2012
//   34. Protected envelopes
//    34.3 Processing protected envelopes
//     34.3.1 Encryption

// ! TYPE: POSITIVE

module top(a, b);
  input a;
  output b;




  logic b;

  initial
    begin
      b = 0;
    end

  always
    begin
      #5 b = a;
    end


endmodule
