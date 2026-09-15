// SV-CORPUS-GRAD.20 — a LIST of path inputs joined to an output by `=>`.
// IEEE 1364-2005 A.7.2: `parallel_path_description ::= ( specify_input_terminal_descriptor
// [ polarity_operator ] => specify_output_terminal_descriptor )` — ONE input; a list takes the FULL
// connection `full_path_description ::= ( list_of_path_inputs [ polarity_operator ] *> list_of_path_outputs )`
// (§14.2.2: "*> establishes a full connection", "=> establishes a parallel connection"). IEEE
// 1800-2017 A.7.2 is the same. Shape from iverilog/ivtest/ivltests/pr1877743.v (`(a, b, ci => co) = 1.5;`)
// and specify_01.v (`(clk,d => q) = (tR_clk_q,tF_clk_q);`), which Icarus accepts under -gspecify.
module m(input a, b, ci, output co);
  specify
    (a, b, ci => co) = 1.5;
  endspecify
endmodule
