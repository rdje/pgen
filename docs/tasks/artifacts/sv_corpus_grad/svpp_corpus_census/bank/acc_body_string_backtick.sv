`define CHECK(val) \
  $display("FAILED(%0d): Expected `%b`, got `%b`.", `__LINE__, \
           val)
