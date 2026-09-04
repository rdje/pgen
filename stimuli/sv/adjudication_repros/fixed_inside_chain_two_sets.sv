// SV-CORPUS-GRAD.3.41 - FIXED (release 1.0.213): two inside expressions joined by a logical operator ACCEPT.
// z = a inside {..} || b inside {..}; - the verilator t_randomize_method_constraints.v:14 shape `x inside {1,2} || x inside {4,5};`
// (a bar row until 1.0.212, stop `|| ID inside`). Both sets are chain elements: [a, inside{..}, ||, b, inside{..}].
// Also the `if (a inside {..} && b inside {..})` shape that cond_predicate reaches through expression_base.
// IEEE 1800-2017 A.8.3 docs/systemverilog/2017/md/section-41-data-read-api.md:3026-3043; 11.3.2 Table 11-2 line 337
module t;
  logic [1:0] a, b;
  logic z, w;
  always_comb z = a inside {2'b01, 2'b10} || b inside {[2'b00:2'b01]};
  always_comb if (a inside {2'b01} && b inside {2'b10}) w = 1'b1; else w = 1'b0;
endmodule
