class C;
  rand int x;
  constraint c1 { x dist { [100:102] := 1, 200 := 2, 300 := 5}; }
endclass
