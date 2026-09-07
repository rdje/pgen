`resetall
`timescale 1 ns / 1 ps
`timescale 10ns/100ps
`timescale 100 s / 1 fs
`timescale 1ps/1ns
`default_nettype none
`default_nettype uwire
`default_nettype tri0
`unconnected_drive pull1
`unconnected_drive pull0
`nounconnected_drive
`celldefine
module m; endmodule
`endcelldefine
`undef FOO
`undef \esc*ape 
`pragma protect begin
`pragma protect encoding = (enctype = "BASE64", line_length = 76, bytes = 128)
`pragma protect key_keyowner="BIG3#1"
`pragma once
`pragma foo something, somethingelse = 7, "abcdef", ( hello, ( "world", pr = 4, "gm" ) ), a
`pragma pragma_number 123
`pragma diagnostic warn="-Wfoobaz" // a trailing comment is layout
`line 3 "orig.v" 2
`line 8273 "somefile" 0
`begin_keywords "1364-2005"
module n; endmodule
`end_keywords
`include "x.svh"
`define X 1
`FOO
`default_trireg_strength this_is_optional
