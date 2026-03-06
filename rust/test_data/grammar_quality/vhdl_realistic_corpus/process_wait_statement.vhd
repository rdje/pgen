entity timer is
  port(clk : in bit; q : out bit);
end entity timer;

architecture rtl of timer is
begin
  process
  begin
    wait until clk = '1';
    q <= '1';
  end process;
end architecture rtl;
