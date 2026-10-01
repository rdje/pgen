class P #(type T=int); endclass
typedef P #(int) T1;
class C;
  const local T1 events = new("events");
endclass
