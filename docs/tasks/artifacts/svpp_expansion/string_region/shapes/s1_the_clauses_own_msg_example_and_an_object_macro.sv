`define msg(x,y) `"x: `\`"y`\`"`"
`define plain(x) `"x`"
`define obj `"obj `\`"q`\`" end`"
module t;
initial begin
  $display(`msg(left side,right side));
  $display(`plain(hello));
  $display(`obj);
end
endmodule
