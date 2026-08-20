class C;
  rand int a, b;
  constraint c1 { solve a before b; }
endclass
