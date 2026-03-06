library work;
use work.defs.all;

entity top is
  port(a : in bit_vector(WIDTH-1 downto 0); y : out bit_vector(WIDTH-1 downto 0));
end entity top;

architecture rtl of top is
begin
  y <= a;
end architecture rtl;
