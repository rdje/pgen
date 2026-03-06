entity p0 is
  port(a : in bit; b : in bit; y : out bit);
end entity p0;

architecture rtl of p0 is
begin
  process(a,b)
  begin
    if a = b then
      y <= a;
    else
      y <= b;
    end if;
  end process;
end architecture rtl;
