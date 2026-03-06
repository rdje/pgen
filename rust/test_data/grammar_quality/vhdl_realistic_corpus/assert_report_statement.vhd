entity a0 is
  port(a : in bit);
end entity a0;

architecture rtl of a0 is
begin
  process(a)
  begin
    assert a = '1' report "bad";
  end process;
end architecture rtl;
