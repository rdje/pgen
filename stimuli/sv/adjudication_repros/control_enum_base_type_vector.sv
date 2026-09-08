// SV-CORPUS-GRAD.3.51 - the twin of invalid_enum_base_type_package_scoped.sv and invalid_enum_base_dimension_without_type.sv with an integer_vector_type base: ACCEPT.
package pkg2;
  typedef enum logic [7:0] { a = 1 } e_t;
endpackage
