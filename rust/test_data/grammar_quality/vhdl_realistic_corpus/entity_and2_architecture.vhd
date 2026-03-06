entity and2 is
  port(a, b : in bit; y : out bit);
end entity and2;

architecture rtl of and2 is
begin
  y <= a and b;
end architecture rtl;
