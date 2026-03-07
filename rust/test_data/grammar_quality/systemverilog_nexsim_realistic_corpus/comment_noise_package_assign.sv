/* package fake_pkg; localparam int A = 0; endpackage */
package real_pkg;
  localparam int A = 1;
endpackage

module comment_noise_package_assign;
  int x;
  // import fake_pkg::*;
  assign x = real_pkg::A;
endmodule
