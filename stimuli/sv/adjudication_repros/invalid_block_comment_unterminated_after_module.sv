// ENGINE-UNIVERSAL-SERVICES.107 / SV-CORPUS-GRAD.3.62 - IEEE 1800-2017 §5.4: "A block comment shall
// start with /* and end with */." A file that ends INSIDE a block comment has no such comment, so
// the text is illegal however much legal source precedes it. Through 1.0.230 the engine's built-in
// layout skipper consumed the opener and kept going, so this parsed.
module m;
endmodule
/*
