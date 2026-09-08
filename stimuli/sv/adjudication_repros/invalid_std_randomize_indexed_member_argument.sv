class C; int member[5]; endclass
module m;
  C example;
  initial begin example = new;
    if (std::randomize(example.member[1]) with { example.member[1] inside {[0:3]}; } == 0) $stop;
  end
endmodule
