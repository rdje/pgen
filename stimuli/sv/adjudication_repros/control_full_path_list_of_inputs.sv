// SV-CORPUS-GRAD.20 — the legal spelling of invalid_parallel_path_list_of_inputs.sv:
// a list of inputs uses the full connection `*>` (1364-2005 A.7.2 / 1800-2017 A.7.2).
module m(input a, b, ci, output co);
  specify
    (a, b, ci *> co) = 1.5;
  endspecify
endmodule
