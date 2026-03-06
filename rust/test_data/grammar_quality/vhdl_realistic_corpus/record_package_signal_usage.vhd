package math_pkg is
  type pair_t is record
    a : integer;
    b : integer;
  end record;
  constant C0 : integer := 1;
end package math_pkg;

entity math_top is
  port(i0 : in integer; o0 : out integer);
end entity math_top;

architecture rtl of math_top is
  signal p : pair_t;
begin
  p.a <= i0;
  o0 <= p.a + C0;
end architecture rtl;
