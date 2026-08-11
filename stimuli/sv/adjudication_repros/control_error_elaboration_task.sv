// Control: `$error` legitimately takes no finish_number, and parses in the same position.
module m;
  if (1) begin
    $error("boom");
  end
endmodule
