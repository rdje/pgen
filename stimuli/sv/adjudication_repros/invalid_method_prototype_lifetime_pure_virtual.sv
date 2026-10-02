// SV-CORPUS-GRAD.41 - an INVALID pin - text neither IEEE 1800-2017 nor 1800-2023 derives; must keep REJECTING on every profile.
// pure virtual task automatic t(); - a LIFETIME in a method prototype
// IEEE 1800-2017 A.2.7 task_prototype ::= task task_identifier [ ( [ tf_port_list ] ) ] docs/systemverilog/2017/txt/section-41-data-read-api.txt:753 (no lifetime, where task_declaration writes task [ lifetime ]); 8.24 docs/systemverilog/2017/txt/section-8-classes.txt:1011-1013 (a prototype declares whether it is a function or task, any qualifiers (local, protected, or virtual), and the full argument specification); IEEE 1800-2023 the same: docs/systemverilog/2023/txt/section-Annex_A-normative-formal-syntax.txt:782, 8.24 docs/systemverilog/2023/txt/section-8-classes.txt:1198 (the enumeration gains the specifiers initial / extends / final and still no lifetime)
virtual class b;
  pure virtual task automatic t();
endclass
