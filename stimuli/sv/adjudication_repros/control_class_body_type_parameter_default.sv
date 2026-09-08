// SV-CORPUS-GRAD.3.52 - the twins of invalid_class_body_type_parameter_without_default.sv: a body type parameter with a default and a port-list one without: ACCEPT.
class C;
  parameter type AT = logic;
endclass
class D #(parameter type BT);
endclass
