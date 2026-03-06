entity child is
  port(a : in bit; y : out bit);
end entity child;

entity top is
  port(d : in bit; q : out bit);
end entity top;

architecture rtl of top is
begin
  u0: child port map(a => d, y => q);
end architecture rtl;
