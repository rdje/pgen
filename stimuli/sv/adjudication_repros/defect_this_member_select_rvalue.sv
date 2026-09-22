// SV-CORPUS-GRAD.28 - a DEFECT pin (REJECT today) - routed to SV-CORPUS-GRAD.29.
// IEEE 1800-2017 A.8.4 primary ::= [ implicit_class_handle . | class_scope | package_scope ] hierarchical_identifier select — a `this.member` with a
// trailing select is a primary; the parser refuses it as an RVALUE (accepts it as an lvalue and with a further `.member`).
// Surelog tests/SplitFile/amiq_eth_ve_top.v line 8925 `m_id = this.x_port_names[m_port_name];` (the RESIDUAL dark row after .28).
class c;
  int arr[4];
  int q[$];
  string names[string];
  string n;
  int i, id;
  function void f();
    id = this.arr[0];
    id = this.q[i];
    id = this.names[n];
  endfunction
endclass
