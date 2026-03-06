entity g is
  port(a : in bit; y : out bit);
end entity g;

architecture rtl of g is
begin
  gen0: for i in 0 to 0 generate
    y <= a;
  end generate;
end architecture rtl;
