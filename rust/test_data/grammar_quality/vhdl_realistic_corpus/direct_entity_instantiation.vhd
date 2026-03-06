entity child is
  port(a : in bit; y : out bit);
end entity child;

architecture rtl of child is
begin
  y <= a;
end architecture rtl;

entity top is
  port(d : in bit; q : out bit);
end entity top;

architecture rtl of top is
begin
  u0: entity work.child port map(a => d, y => q);
end architecture rtl;
