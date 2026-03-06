entity e is
  port(a : in bit; y : out bit);
end entity e;

architecture rtl of e is
begin
  y <= a;
end architecture rtl;

configuration cfg of e is
  for rtl
  end for;
end configuration cfg;
