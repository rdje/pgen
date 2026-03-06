---
title: "Section 41: IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language"
document: "SystemVerilog Language Reference Manual"
standard: "IEEE 1800-2017"
domain: "SystemVerilog"
section: "41"
source_txt: "section-41-data-read-api.txt"
source_pdf: "/Users/richarddje/Documents/github/SystemVerilog-LRM-IEEE-1800-2017.pdf"
---

# Section 41: IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language

IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1134
Copyright © 2018 IEEE. All rights reserved.
41. Data read API
This clause has been deprecated. See IEEE Std 1800-2005 for the contents of this clause.
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1135
Copyright © 2018 IEEE. All rights reserved.
Part Four:
Annexes
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1136
Copyright © 2018 IEEE. All rights reserved.
Annex A
(normative)
Formal syntax
The formal syntax of SystemVerilog is described using Backus-Naur Form (BNF). The syntax of
SystemVerilog source is derived from the starting symbol source_text. The syntax of a library map file is
derived from the starting symbol library_text. The conventions used are as follows:
—
Keywords and punctuation are in bold-red text.
—
Syntactic categories are named in nonbold text.
—
A vertical bar ( | ) separates alternatives.
—
Square brackets ( [ ] ) enclose optional items.
—
Braces ( { } ) enclose items that can be repeated zero or more times.
The full syntax and semantics of SystemVerilog are not described solely using BNF. The normative text
description contained within the clauses and annexes of this standard provide additional details on the syntax
and semantics described in this BNF.
A qualified term in the syntax is a term such as array_identifier for which the “array” portion represents
some semantic intent and the “identifier” term indicates that the qualified term reduces to the “identifier”
term in the syntax. The syntax does not completely define the semantics of such qualified terms; for exam-
ple, while an identifier that would qualify semantically as an array_identifier is created by a declaration,
such declaration forms are not explicitly described using array_identifier in the syntax.
A.1 Source text
A.1.1 Library source text
```ebnf
library_text ::= { library_description }
library_description ::=
```

library_declaration
| include_statement
| config_declaration
| ;
```ebnf
library_declaration ::=
```

library library_identifier file_path_spec { , file_path_spec }
[ -incdir file_path_spec { , file_path_spec } ] ;
```ebnf
include_statement ::= include file_path_spec ;
```

A.1.2 SystemVerilog source text
```ebnf
source_text ::= [ timeunits_declaration ] { description }
description ::=
```

module_declaration
| udp_declaration
| interface_declaration
| program_declaration
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1137
Copyright © 2018 IEEE. All rights reserved.
| package_declaration
| { attribute_instance } package_item
| { attribute_instance } bind_directive
| config_declaration
```ebnf
module_nonansi_header ::=
```

{ attribute_instance } module_keyword [ lifetime ] module_identifier
{ package_import_declaration } [ parameter_port_list ] list_of_ports ;
```ebnf
module_ansi_header ::=
```

{ attribute_instance } module_keyword [ lifetime ] module_identifier
{ package_import_declaration }1 [ parameter_port_list ] [ list_of_port_declarations ] ;
```ebnf
module_declaration ::=
```

module_nonansi_header [ timeunits_declaration ] { module_item }
endmodule [ : module_identifier ]
| module_ansi_header [ timeunits_declaration ] { non_port_module_item }
endmodule [ : module_identifier ]
| { attribute_instance } module_keyword [ lifetime ] module_identifier ( .* ) ;
[ timeunits_declaration ] { module_item } endmodule [ : module_identifier ]
| extern module_nonansi_header
| extern module_ansi_header
```ebnf
module_keyword ::= module | macromodule
interface_declaration ::=
```

interface_nonansi_header [ timeunits_declaration ] { interface_item }
endinterface [ : interface_identifier ]
| interface_ansi_header [ timeunits_declaration ] { non_port_interface_item }
endinterface [ : interface_identifier ]
| { attribute_instance } interface interface_identifier ( .* ) ;
[ timeunits_declaration ] { interface_item }
endinterface [ : interface_identifier ]
| extern interface_nonansi_header
| extern interface_ansi_header
```ebnf
interface_nonansi_header ::=
```

{ attribute_instance } interface [ lifetime ] interface_identifier
{ package_import_declaration } [ parameter_port_list ] list_of_ports ;
```ebnf
interface_ansi_header ::=
```

{attribute_instance } interface [ lifetime ] interface_identifier
{ package_import_declaration }1 [ parameter_port_list ] [ list_of_port_declarations ] ;
```ebnf
program_declaration ::=
```

program_nonansi_header [ timeunits_declaration ] { program_item }
endprogram [ : program_identifier ]
| program_ansi_header [ timeunits_declaration ] { non_port_program_item }
endprogram [ : program_identifier ]
| { attribute_instance } program program_identifier ( .* ) ;
[ timeunits_declaration ] { program_item }
endprogram [ : program_identifier ]
| extern program_nonansi_header
| extern program_ansi_header
```ebnf
program_nonansi_header ::=
```

{ attribute_instance } program [ lifetime ] program_identifier
{ package_import_declaration } [ parameter_port_list ] list_of_ports ;
```ebnf
program_ansi_header ::=
```

{attribute_instance } program [ lifetime ] program_identifier
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1138
Copyright © 2018 IEEE. All rights reserved.
{ package_import_declaration }1 [ parameter_port_list ] [ list_of_port_declarations ] ;
```ebnf
checker_declaration ::=
```

checker checker_identifier [ ( [ checker_port_list ] ) ] ;
{ { attribute_instance } checker_or_generate_item }
endchecker [ : checker_identifier ]
```ebnf
class_declaration ::=
```

[ virtual ] class [ lifetime ] class_identifier [ parameter_port_list ]
[ extends class_type [ ( list_of_arguments ) ] ]
[ implements interface_class_type { , interface_class_type } ] ;
{ class_item }
endclass [ : class_identifier]
```ebnf
interface_class_type ::= ps_class_identifier [ parameter_value_assignment ]
interface_class_declaration ::=
```

interface class class_identifier [ parameter_port_list ]
[ extends interface_class_type { , interface_class_type } ] ;
{ interface_class_item }
endclass [ : class_identifier]
```ebnf
interface_class_item ::=
```

type_declaration
| { attribute_instance } interface_class_method
| local_parameter_declaration ;
| parameter_declaration7 ;
| ;
```ebnf
interface_class_method ::=
```

pure virtual method_prototype ;
```ebnf
package_declaration ::=
```

{ attribute_instance } package [ lifetime ] package_identifier ;
[ timeunits_declaration ] { { attribute_instance } package_item }
endpackage [ : package_identifier ]
```ebnf
timeunits_declaration ::=
```

timeunit time_literal [ / time_literal ] ;
| timeprecision time_literal ;
| timeunit time_literal ; timeprecision time_literal ;
| timeprecision time_literal ; timeunit time_literal ;
A.1.3 Module parameters and ports
```ebnf
parameter_port_list ::=
```

# ( list_of_param_assignments { , parameter_port_declaration } )
| # ( parameter_port_declaration { , parameter_port_declaration } )
| #( )
```ebnf
parameter_port_declaration ::=
```

parameter_declaration
| local_parameter_declaration
| data_type list_of_param_assignments
| type list_of_type_assignments
```ebnf
list_of_ports ::= ( port { , port } )
list_of_port_declarations2 ::=
```

( [ { attribute_instance} ansi_port_declaration { , { attribute_instance} ansi_port_declaration } ] )
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1139
Copyright © 2018 IEEE. All rights reserved.
```ebnf
port_declaration ::=
```

{ attribute_instance } inout_declaration
| { attribute_instance } input_declaration
| { attribute_instance } output_declaration
| { attribute_instance } ref_declaration
| { attribute_instance } interface_port_declaration
```ebnf
port ::=
```

[ port_expression ]
| . port_identifier ( [ port_expression ] )
```ebnf
port_expression ::=
```

port_reference
| { port_reference { , port_reference } }
```ebnf
port_reference ::=
```

port_identifier constant_select
```ebnf
port_direction ::= input | output | inout | ref
net_port_header ::= [ port_direction ] net_port_type
variable_port_header ::= [ port_direction ] variable_port_type
interface_port_header ::=
```

interface_identifier [ . modport_identifier ]
| interface [ . modport_identifier ]
```ebnf
ansi_port_declaration ::=
```

[ net_port_header | interface_port_header ] port_identifier { unpacked_dimension }
[ = constant_expression ]
| [ variable_port_header ] port_identifier { variable_dimension } [ = constant_expression ]
| [ port_direction ] . port_identifier ( [ expression ] )
A.1.4 Module items
```ebnf
elaboration_system_task ::=
```

$fatal [ ( finish_number [, list_of_arguments ] ) ] ;
| $error [ ( [ list_of_arguments ] ) ] ;
| $warning [ ( [ list_of_arguments ] ) ] ;
| $info [ ( [ list_of_arguments ] ) ] ;
```ebnf
finish_number ::= 0 | 1 | 2
module_common_item ::=
```

module_or_generate_item_declaration
| interface_instantiation
| program_instantiation
| assertion_item
| bind_directive
| continuous_assign
| net_alias
| initial_construct
| final_construct
| always_construct
| loop_generate_construct
| conditional_generate_construct
| elaboration_system_task
```ebnf
module_item ::=
```

port_declaration ;
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1140
Copyright © 2018 IEEE. All rights reserved.
| non_port_module_item
```ebnf
module_or_generate_item ::=
```

{ attribute_instance } parameter_override
| { attribute_instance } gate_instantiation
| { attribute_instance } udp_instantiation
| { attribute_instance } module_instantiation
| { attribute_instance } module_common_item
```ebnf
module_or_generate_item_declaration ::=
```

package_or_generate_item_declaration
| genvar_declaration
| clocking_declaration
| default clocking clocking_identifier ;
| default disable iff expression_or_dist ;
```ebnf
non_port_module_item ::=
```

generate_region
| module_or_generate_item
| specify_block
| { attribute_instance } specparam_declaration
| program_declaration
| module_declaration
| interface_declaration
| timeunits_declaration3
```ebnf
parameter_override ::= defparam list_of_defparam_assignments ;
bind_directive4 ::=
```

bind bind_target_scope [: bind_target_instance_list] bind_instantiation ;
| bind bind_target_instance bind_instantiation ;
```ebnf
bind_target_scope ::=
```

module_identifier
| interface_identifier
```ebnf
bind_target_instance ::=
```

hierarchical_identifier constant_bit_select
```ebnf
bind_target_instance_list ::=
```

bind_target_instance { , bind_target_instance }
```ebnf
bind_instantiation ::=
```

program_instantiation
| module_instantiation
| interface_instantiation
| checker_instantiation
A.1.5 Configuration source text
```ebnf
config_declaration ::=
```

config config_identifier ;
{ local_parameter_declaration ; }
design_statement
{ config_rule_statement }
endconfig [ : config_identifier ]
```ebnf
design_statement ::= design { [ library_identifier . ] cell_identifier } ;
config_rule_statement ::=
```

default_clause liblist_clause ;
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1141
Copyright © 2018 IEEE. All rights reserved.
| inst_clause liblist_clause ;
| inst_clause use_clause ;
| cell_clause liblist_clause ;
| cell_clause use_clause ;
```ebnf
default_clause ::= default
inst_clause ::= instance inst_name
inst_name ::= topmodule_identifier { . instance_identifier }
cell_clause ::= cell [ library_identifier . ] cell_identifier
liblist_clause ::= liblist {library_identifier}
use_clause ::= use [ library_identifier . ] cell_identifier [ : config ]
| use named_parameter_assignment { , named_parameter_assignment } [ : config ]
| use [ library_identifier . ] cell_identifier named_parameter_assignment
```

{ , named_parameter_assignment } [ : config ]
A.1.6 Interface items
```ebnf
interface_or_generate_item ::=
```

{ attribute_instance } module_common_item
| { attribute_instance } extern_tf_declaration
```ebnf
extern_tf_declaration ::=
```

extern method_prototype ;
| extern forkjoin task_prototype ;
```ebnf
interface_item ::=
```

port_declaration ;
| non_port_interface_item
```ebnf
non_port_interface_item ::=
```

generate_region
| interface_or_generate_item
| program_declaration
| modport_declaration
| interface_declaration
| timeunits_declaration3
A.1.7 Program items
```ebnf
program_item ::=
```

port_declaration ;
| non_port_program_item
```ebnf
non_port_program_item ::=
```

{ attribute_instance } continuous_assign
| { attribute_instance } module_or_generate_item_declaration
| { attribute_instance } initial_construct
| { attribute_instance } final_construct
| { attribute_instance } concurrent_assertion_item
| timeunits_declaration3
| program_generate_item
```ebnf
program_generate_item5 ::=
```

loop_generate_construct
| conditional_generate_construct
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1142
Copyright © 2018 IEEE. All rights reserved.
| generate_region
| elaboration_system_task
A.1.8 Checker items
```ebnf
checker_port_list ::=
```

checker_port_item {, checker_port_item}
```ebnf
checker_port_item ::=
```

{ attribute_instance } [ checker_port_direction ] property_formal_type formal_port_identifier
{variable_dimension} [ = property_actual_arg ]
```ebnf
checker_port_direction ::=
```

input | output
```ebnf
checker_or_generate_item ::=
```

checker_or_generate_item_declaration
| initial_construct
| always_construct
| final_construct
| assertion_item
| continuous_assign
| checker_generate_item
```ebnf
checker_or_generate_item_declaration ::=
```

[ rand ] data_declaration
| function_declaration
| checker_declaration
| assertion_item_declaration
| covergroup_declaration
| genvar_declaration
| clocking_declaration
| default clocking clocking_identifier ;
| default disable iff expression_or_dist ;
| ;
```ebnf
checker_generate_item6 ::=
```

loop_generate_construct
| conditional_generate_construct
| generate_region
| elaboration_system_task
A.1.9 Class items
```ebnf
class_item ::=
```

{ attribute_instance } class_property
| { attribute_instance } class_method
| { attribute_instance } class_constraint
| { attribute_instance } class_declaration
| { attribute_instance } covergroup_declaration
| local_parameter_declaration ;
| parameter_declaration7 ;
| ;
```ebnf
class_property ::=
```

{ property_qualifier } data_declaration
| const { class_item_qualifier } data_type const_identifier [ = constant_expression ] ;
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1143
Copyright © 2018 IEEE. All rights reserved.
```ebnf
class_method ::=
```

{ method_qualifier } task_declaration
| { method_qualifier } function_declaration
| pure virtual { class_item_qualifier } method_prototype ;
| extern { method_qualifier } method_prototype ;
| { method_qualifier } class_constructor_declaration
| extern { method_qualifier } class_constructor_prototype
```ebnf
class_constructor_prototype ::=
```

function new [ ( [ tf_port_list ] ) ] ;
```ebnf
class_constraint ::=
```

constraint_prototype
| constraint_declaration
```ebnf
class_item_qualifier8 ::=
```

static
| protected
| local
```ebnf
property_qualifier8 ::=
```

random_qualifier
| class_item_qualifier
```ebnf
random_qualifier8 ::=
```

rand
| randc
```ebnf
method_qualifier8 ::=
```

[ pure ] virtual
| class_item_qualifier
```ebnf
method_prototype ::=
```

task_prototype
| function_prototype
```ebnf
class_constructor_declaration ::=
```

function [ class_scope ] new [ ( [ tf_port_list ] ) ] ;
{ block_item_declaration }
[ super . new [ ( list_of_arguments ) ] ; ]
{ function_statement_or_null }
endfunction [ : new ]
A.1.10 Constraints
```ebnf
constraint_declaration ::= [ static ] constraint constraint_identifier constraint_block
constraint_block ::= { { constraint_block_item } }
constraint_block_item ::=
```

solve solve_before_list before solve_before_list ;
| constraint_expression
```ebnf
solve_before_list ::= constraint_primary { , constraint_primary }
constraint_primary ::= [ implicit_class_handle . | class_scope ] hierarchical_identifier select
constraint_expression ::=
```

[ soft ] expression_or_dist ;
| uniqueness_constraint ;
| expression –> constraint_set
| if ( expression ) constraint_set [ else constraint_set ]
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1144
Copyright © 2018 IEEE. All rights reserved.
| foreach ( ps_or_hierarchical_array_identifier [ loop_variables ] ) constraint_set
| disable soft constraint_primary ;
```ebnf
uniqueness_constraint ::=
```

unique { open_range_list9 }
```ebnf
constraint_set ::=
```

constraint_expression
| { { constraint_expression } }
```ebnf
dist_list ::= dist_item { , dist_item }
dist_item ::= value_range [ dist_weight ]
dist_weight ::=
```

:= expression
| :/ expression
```ebnf
constraint_prototype ::= [constraint_prototype_qualifier] [ static ] constraint constraint_identifier ;
constraint_prototype_qualifier ::= extern | pure
extern_constraint_declaration ::=
```

[ static ] constraint class_scope constraint_identifier constraint_block
```ebnf
identifier_list ::= identifier { , identifier }
```

A.1.11 Package items
```ebnf
package_item ::=
```

package_or_generate_item_declaration
| anonymous_program
| package_export_declaration
| timeunits_declaration3
```ebnf
package_or_generate_item_declaration ::=
```

net_declaration
| data_declaration
| task_declaration
| function_declaration
| checker_declaration
| dpi_import_export
| extern_constraint_declaration
| class_declaration
| class_constructor_declaration
| local_parameter_declaration ;
| parameter_declaration ;
| covergroup_declaration
| assertion_item_declaration
| ;
```ebnf
anonymous_program ::= program ; { anonymous_program_item } endprogram
anonymous_program_item ::=
```

task_declaration
| function_declaration
| class_declaration
| covergroup_declaration
| class_constructor_declaration
| ;
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1145
Copyright © 2018 IEEE. All rights reserved.
A.2 Declarations
A.2.1 Declaration types
A.2.1.1 Module parameter declarations
```ebnf
local_parameter_declaration ::=
```

localparam data_type_or_implicit list_of_param_assignments
| localparam type list_of_type_assignments
```ebnf
parameter_declaration ::=
```

parameter data_type_or_implicit list_of_param_assignments
| parameter type list_of_type_assignments
```ebnf
specparam_declaration ::=
```

specparam [ packed_dimension ] list_of_specparam_assignments ;
A.2.1.2 Port declarations
```ebnf
inout_declaration ::=
```

inout net_port_type list_of_port_identifiers
```ebnf
input_declaration ::=
```

input net_port_type list_of_port_identifiers
| input variable_port_type list_of_variable_identifiers
```ebnf
output_declaration ::=
```

output net_port_type list_of_port_identifiers
| output variable_port_type list_of_variable_port_identifiers
```ebnf
interface_port_declaration ::=
```

interface_identifier list_of_interface_identifiers
| interface_identifier . modport_identifier list_of_interface_identifiers
```ebnf
ref_declaration ::= ref variable_port_type  list_of_variable_identifiers
```

A.2.1.3 Type declarations
```ebnf
data_declaration ::=
```

[ const ] [ var ] [ lifetime ] data_type_or_implicit list_of_variable_decl_assignments ;10
| type_declaration
| package_import_declaration11
| net_type_declaration
```ebnf
package_import_declaration ::=
```

import package_import_item { , package_import_item } ;
```ebnf
package_import_item ::=
```

package_identifier :: identifier
| package_identifier :: *
```ebnf
package_export_declaration ::=
```

export *::* ;
| export package_import_item { , package_import_item } ;
```ebnf
genvar_declaration ::= genvar list_of_genvar_identifiers ;
net_declaration12 ::=
```

net_type [ drive_strength | charge_strength ] [ vectored | scalared ]
data_type_or_implicit [ delay3 ] list_of_net_decl_assignments ;
| net_type_identifier [ delay_control ]
list_of_net_decl_assignments ;
| interconnect implicit_data_type [ # delay_value ]
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1146
Copyright © 2018 IEEE. All rights reserved.
net_identifier { unpacked_dimension }
[ , net_identifier { unpacked_dimension }] ;
```ebnf
type_declaration ::=
```

typedef data_type type_identifier { variable_dimension } ;
| typedef interface_instance_identifier constant_bit_select . type_identifier type_identifier ;
| typedef [ enum | struct | union | class | interface class ] type_identifier ;
```ebnf
net_type_declaration ::=
```

nettype data_type net_type_identifier
[ with [ package_scope | class_scope ] tf_identifier ] ;
| nettype [ package_scope | class_scope ] net_type_identifier net_type_identifier ;
```ebnf
lifetime ::= static | automatic
```

A.2.2 Declaration data types
A.2.2.1 Net and variable types
```ebnf
casting_type ::= simple_type | constant_primary | signing | string | const
data_type ::=
```

integer_vector_type [ signing ] { packed_dimension }
| integer_atom_type [ signing ]
| non_integer_type
| struct_union [ packed [ signing ] ] { struct_union_member { struct_union_member } }
{ packed_dimension }13
| enum [ enum_base_type ] { enum_name_declaration { , enum_name_declaration } }
{ packed_dimension }
| string
| chandle
| virtual [ interface ] interface_identifier [ parameter_value_assignment ] [ . modport_identifier ]
| [ class_scope | package_scope ] type_identifier { packed_dimension }
| class_type
| event
| ps_covergroup_identifier
| type_reference14
```ebnf
data_type_or_implicit ::=
```

data_type
| implicit_data_type
```ebnf
implicit_data_type ::= [ signing ] { packed_dimension }
enum_base_type ::=
```

integer_atom_type [ signing ]
| integer_vector_type [ signing ] [ packed_dimension ]
| type_identifier [ packed_dimension ]15
```ebnf
enum_name_declaration ::=
```

enum_identifier [ [ integral_number [ : integral_number ] ] ] [ = constant_expression ]
```ebnf
class_scope ::= class_type ::
class_type ::=
```

ps_class_identifier [ parameter_value_assignment ]
{ :: class_identifier [ parameter_value_assignment ] }
```ebnf
integer_type ::= integer_vector_type | integer_atom_type
integer_atom_type ::= byte | shortint | int | longint | integer | time
integer_vector_type ::= bit | logic | reg
```

Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1147
Copyright © 2018 IEEE. All rights reserved.
```ebnf
non_integer_type ::= shortreal | real | realtime
net_type ::= supply0 | supply1 | tri | triand | trior | trireg| tri0 | tri1 | uwire| wire | wand | wor
net_port_type ::=
```

[ net_type ] data_type_or_implicit
| net_type_identifier
| interconnect implicit_data_type
```ebnf
variable_port_type ::= var_data_type
var_data_type ::= data_type | var data_type_or_implicit
signing ::= signed | unsigned
simple_type ::= integer_type | non_integer_type | ps_type_identifier | ps_parameter_identifier
struct_union_member16 ::=
```

{ attribute_instance } [random_qualifier] data_type_or_void list_of_variable_decl_assignments ;
```ebnf
data_type_or_void ::= data_type | void
struct_union ::= struct | union [ tagged ]
type_reference ::=
```

type ( expression17 )
| type ( data_type )
A.2.2.2 Strengths
```ebnf
drive_strength ::=
```

( strength0 , strength1 )
| ( strength1 , strength0 )
| ( strength0 , highz1 )
| ( strength1 , highz0 )
| ( highz0 , strength1 )
| ( highz1 , strength0 )
```ebnf
strength0 ::= supply0 | strong0 | pull0 | weak0
strength1 ::= supply1 | strong1 | pull1 | weak1
charge_strength ::= ( small ) | ( medium ) | ( large )
```

A.2.2.3 Delays
```ebnf
delay3 ::= # delay_value | # ( mintypmax_expression [ , mintypmax_expression [ ,
```

mintypmax_expression ] ] )
```ebnf
delay2 ::= # delay_value | # ( mintypmax_expression [ , mintypmax_expression ] )
delay_value ::=
```

unsigned_number
| real_number
| ps_identifier
| time_literal
| 1step
A.2.3 Declaration lists
```ebnf
list_of_defparam_assignments ::= defparam_assignment { , defparam_assignment }
list_of_genvar_identifiers ::= genvar_identifier { , genvar_identifier }
list_of_interface_identifiers ::= interface_identifier { unpacked_dimension }
```

{ , interface_identifier { unpacked_dimension } }
```ebnf
list_of_net_decl_assignments ::= net_decl_assignment { , net_decl_assignment }
```

Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1148
Copyright © 2018 IEEE. All rights reserved.
```ebnf
list_of_param_assignments ::= param_assignment { , param_assignment }
list_of_port_identifiers ::= port_identifier { unpacked_dimension }
```

{ , port_identifier { unpacked_dimension } }
```ebnf
list_of_udp_port_identifiers ::= port_identifier { , port_identifier }
list_of_specparam_assignments ::= specparam_assignment { , specparam_assignment }
list_of_tf_variable_identifiers ::= port_identifier { variable_dimension } [ = expression ]
```

{ , port_identifier { variable_dimension } [ = expression ] }
```ebnf
list_of_type_assignments ::= type_assignment { , type_assignment }
list_of_variable_decl_assignments ::= variable_decl_assignment { , variable_decl_assignment }
list_of_variable_identifiers ::= variable_identifier { variable_dimension }
```

{ , variable_identifier { variable_dimension } }
```ebnf
list_of_variable_port_identifiers ::= port_identifier { variable_dimension } [ = constant_expression ]
```

{ , port_identifier { variable_dimension } [ = constant_expression ] }
A.2.4 Declaration assignments
```ebnf
defparam_assignment ::= hierarchical_parameter_identifier = constant_mintypmax_expression
net_decl_assignment ::= net_identifier { unpacked_dimension } [ = expression ]
param_assignment ::=
```

parameter_identifier { unpacked_dimension } [ = constant_param_expression ]18
```ebnf
specparam_assignment ::=
```

specparam_identifier = constant_mintypmax_expression
| pulse_control_specparam
```ebnf
type_assignment ::=
```

type_identifier [ = data_type ]18
```ebnf
pulse_control_specparam ::=
```

PATHPULSE$ = ( reject_limit_value [ , error_limit_value ] )
| PATHPULSE$specify_input_terminal_descriptor$specify_output_terminal_descriptor
= ( reject_limit_value [ , error_limit_value ] )
```ebnf
error_limit_value ::= limit_value
reject_limit_value ::= limit_value
limit_value ::= constant_mintypmax_expression
variable_decl_assignment ::=
```

variable_identifier { variable_dimension } [ = expression ]
| dynamic_array_variable_identifier unsized_dimension { variable_dimension }
[ = dynamic_array_new ]
| class_variable_identifier [ = class_new ]
```ebnf
class_new19 ::=
```

[ class_scope ] new [ ( list_of_arguments ) ]
| new expression
```ebnf
dynamic_array_new ::= new [ expression ] [ ( expression ) ]
```

A.2.5 Declaration ranges
```ebnf
unpacked_dimension ::=
```

[ constant_range ]
| [ constant_expression ]
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1149
Copyright © 2018 IEEE. All rights reserved.
```ebnf
packed_dimension20 ::=
```

[ constant_range ]
| unsized_dimension
```ebnf
associative_dimension ::=
```

[ data_type ]
| [ * ]
```ebnf
variable_dimension ::=
```

unsized_dimension
| unpacked_dimension
| associative_dimension
| queue_dimension
```ebnf
queue_dimension ::= [ $ [ : constant_expression ] ]
unsized_dimension ::= [ ]
```

A.2.6 Function declarations
```ebnf
function_data_type_or_implicit ::=
```

data_type_or_void
| implicit_data_type
```ebnf
function_declaration ::= function [ lifetime ] function_body_declaration
function_body_declaration ::=
```

function_data_type_or_implicit
[ interface_identifier . | class_scope ] function_identifier ;
{ tf_item_declaration }
{ function_statement_or_null }
endfunction [ : function_identifier ]
| function_data_type_or_implicit
[ interface_identifier . | class_scope ] function_identifier ( [ tf_port_list ] ) ;
{ block_item_declaration }
{ function_statement_or_null }
endfunction [ : function_identifier ]
```ebnf
function_prototype ::= function data_type_or_void function_identifier [ ( [ tf_port_list ] ) ]
dpi_import_export ::=
```

import dpi_spec_string [ dpi_function_import_property ] [ c_identifier = ] dpi_function_proto ;
| import dpi_spec_string [ dpi_task_import_property ] [ c_identifier = ] dpi_task_proto ;
| export dpi_spec_string [ c_identifier = ] function function_identifier ;
| export dpi_spec_string [ c_identifier = ] task task_identifier ;
```ebnf
dpi_spec_string ::= "DPI-C" | "DPI"
dpi_function_import_property ::= context | pure
dpi_task_import_property ::= context
dpi_function_proto21,22 ::= function_prototype
dpi_task_proto22 ::= task_prototype
```

A.2.7 Task declarations
```ebnf
task_declaration ::= task [ lifetime ] task_body_declaration
task_body_declaration ::=
```

[ interface_identifier . | class_scope ] task_identifier ;
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1150
Copyright © 2018 IEEE. All rights reserved.
{ tf_item_declaration }
{ statement_or_null }
endtask [ : task_identifier ]
| [ interface_identifier . | class_scope ] task_identifier ( [ tf_port_list ] ) ;
{ block_item_declaration }
{ statement_or_null }
endtask [ : task_identifier ]
```ebnf
tf_item_declaration ::=
```

block_item_declaration
| tf_port_declaration
```ebnf
tf_port_list ::=
```

tf_port_item { , tf_port_item }
```ebnf
tf_port_item23 ::=
```

{ attribute_instance }
[ tf_port_direction ] [ var ] data_type_or_implicit
[ port_identifier { variable_dimension } [ = expression ] ]
```ebnf
tf_port_direction ::= port_direction | const ref
tf_port_declaration ::=
```

{ attribute_instance } tf_port_direction [ var ] data_type_or_implicit list_of_tf_variable_identifiers ;
```ebnf
task_prototype ::= task task_identifier [ ( [ tf_port_list ] ) ]
```

A.2.8 Block item declarations
```ebnf
block_item_declaration ::=
```

{ attribute_instance } data_declaration
| { attribute_instance } local_parameter_declaration ;
| { attribute_instance } parameter_declaration ;
| { attribute_instance } let_declaration
A.2.9 Interface declarations
```ebnf
modport_declaration ::= modport modport_item { , modport_item } ;
modport_item ::= modport_identifier ( modport_ports_declaration { , modport_ports_declaration } )
modport_ports_declaration ::=
```

{ attribute_instance } modport_simple_ports_declaration
| { attribute_instance } modport_tf_ports_declaration
| { attribute_instance } modport_clocking_declaration
```ebnf
modport_clocking_declaration ::= clocking clocking_identifier
modport_simple_ports_declaration ::=
```

port_direction modport_simple_port { , modport_simple_port }
```ebnf
modport_simple_port ::=
```

port_identifier
| . port_identifier ( [ expression ] )
```ebnf
modport_tf_ports_declaration ::=
```

import_export modport_tf_port { , modport_tf_port }
```ebnf
modport_tf_port ::=
```

method_prototype
| tf_identifier
```ebnf
import_export ::= import | export
```

Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1151
Copyright © 2018 IEEE. All rights reserved.
A.2.10 Assertion declarations
```ebnf
concurrent_assertion_item ::=
```

[ block_identifier : ] concurrent_assertion_statement
| checker_instantiation
```ebnf
concurrent_assertion_statement ::=
```

assert_property_statement
| assume_property_statement
| cover_property_statement
| cover_sequence_statement
| restrict_property_statement
```ebnf
assert_property_statement::=
```

assert property ( property_spec ) action_block
```ebnf
assume_property_statement::=
```

assume property ( property_spec ) action_block
```ebnf
cover_property_statement::=
```

cover property ( property_spec ) statement_or_null
```ebnf
expect_property_statement ::=
```

expect ( property_spec ) action_block
```ebnf
cover_sequence_statement::=
```

cover sequence ( [clocking_event ] [ disable iff ( expression_or_dist ) ]
sequence_expr ) statement_or_null
```ebnf
restrict_property_statement::=
```

restrict property ( property_spec ) ;
```ebnf
property_instance ::=
 ps_or_hierarchical_property_identifier [ ( [ property_list_of_arguments ] ) ]
property_list_of_arguments ::=
```

[property_actual_arg] { , [property_actual_arg] } { , . identifier ( [property_actual_arg] ) }
| . identifier ( [property_actual_arg] ) { , . identifier ( [property_actual_arg] ) }
```ebnf
property_actual_arg ::=
```

property_expr
| sequence_actual_arg
```ebnf
assertion_item_declaration ::=
```

property_declaration
| sequence_declaration
| let_declaration
```ebnf
property_declaration ::=
```

property property_identifier [ ( [ property_port_list ] ) ] ;
{ assertion_variable_declaration }
 property_spec [ ; ]
endproperty [ : property_identifier ]
```ebnf
property_port_list ::=
```

property_port_item {, property_port_item}
```ebnf
property_port_item ::=
```

{ attribute_instance } [ local [ property_lvar_port_direction ] ] property_formal_type
formal_port_identifier {variable_dimension} [ = property_actual_arg ]
```ebnf
property_lvar_port_direction ::= input
property_formal_type ::=
```

sequence_formal_type
| property
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1152
Copyright © 2018 IEEE. All rights reserved.
```ebnf
property_spec ::=
```

[clocking_event ] [ disable iff ( expression_or_dist ) ] property_expr
```ebnf
property_expr ::=
```

sequence_expr
| strong ( sequence_expr )
| weak ( sequence_expr )
| ( property_expr )
| not property_expr
| property_expr or property_expr
| property_expr and property_expr
| sequence_expr |-> property_expr
| sequence_expr |=> property_expr
| if ( expression_or_dist ) property_expr [ else property_expr ]
| case ( expression_or_dist ) property_case_item { property_case_item } endcase
| sequence_expr #-# property_expr
| sequence_expr #=# property_expr
| nexttime property_expr
| nexttime [ constant _expression ] property_expr
| s_nexttime property_expr
| s_nexttime [ constant_expression ] property_expr
| always property_expr
| always [ cycle_delay_const_range_expression ] property_expr
| s_always [ constant_range] property_expr
| s_eventually property_expr
| eventually [ constant_range ] property_expr
| s_eventually [ cycle_delay_const_range_expression ] property_expr
| property_expr until property_expr
| property_expr s_until property_expr
| property_expr until_with property_expr
| property_expr s_until_with property_expr
| property_expr implies property_expr
| property_expr iff property_expr
| accept_on ( expression_or_dist ) property_expr
| reject_on ( expression_or_dist ) property_expr
| sync_accept_on ( expression_or_dist ) property_expr
| sync_reject_on ( expression_or_dist ) property_expr
| property_instance
| clocking_event property_expr
```ebnf
property_case_item ::=
```

expression_or_dist { , expression_or_dist } : property_expr ;
| default [ : ] property_expr ;
```ebnf
sequence_declaration ::=
```

sequence sequence_identifier [ ( [ sequence_port_list ] ) ] ;
{ assertion_variable_declaration }
sequence_expr [ ; ]
endsequence [ : sequence_identifier ]
```ebnf
sequence_port_list ::=
```

sequence_port_item {, sequence_port_item}
```ebnf
sequence_port_item ::=
```

{ attribute_instance } [ local [ sequence_lvar_port_direction ] ] sequence_formal_type
formal_port_identifier {variable_dimension} [ = sequence_actual_arg ]
```ebnf
sequence_lvar_port_direction ::= input | inout | output
```

Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1153
Copyright © 2018 IEEE. All rights reserved.
```ebnf
sequence_formal_type ::=
```

data_type_or_implicit
| sequence
| untyped
```ebnf
sequence_expr ::=
```

cycle_delay_range sequence_expr { cycle_delay_range sequence_expr }
| sequence_expr cycle_delay_range sequence_expr { cycle_delay_range sequence_expr }
| expression_or_dist [ boolean_abbrev ]
| sequence_instance [ sequence_abbrev ]
| ( sequence_expr {, sequence_match_item } ) [ sequence_abbrev ]
| sequence_expr and sequence_expr
| sequence_expr intersect sequence_expr
| sequence_expr or sequence_expr
| first_match ( sequence_expr {, sequence_match_item} )
| expression_or_dist throughout sequence_expr
| sequence_expr within sequence_expr
| clocking_event sequence_expr
```ebnf
cycle_delay_range ::=
```

## constant_primary
| ## [ cycle_delay_const_range_expression ]
| ##[*]
| ##[+]
```ebnf
sequence_method_call ::=
```

sequence_instance . method_identifier
```ebnf
sequence_match_item ::=
```

operator_assignment
| inc_or_dec_expression
| subroutine_call
```ebnf
sequence_instance ::=
 ps_or_hierarchical_sequence_identifier [ ( [ sequence_list_of_arguments ] ) ]
sequence_list_of_arguments ::=
```

[sequence_actual_arg] { , [sequence_actual_arg] } { , . identifier ( [sequence_actual_arg] ) }
| . identifier ( [sequence_actual_arg] ) { , . identifier ( [sequence_actual_arg] ) }
```ebnf
sequence_actual_arg ::=
```

event_expression
| sequence_expr
```ebnf
boolean_abbrev ::=
```

consecutive_repetition
| non_consecutive_repetition
| goto_repetition
```ebnf
sequence_abbrev ::= consecutive_repetition
consecutive_repetition ::=
```

[* const_or_range_expression ]
| [*]
| [+]
```ebnf
non_consecutive_repetition ::= [= const_or_range_expression ]
goto_repetition ::= [-> const_or_range_expression ]
const_or_range_expression ::=
```

constant_expression
| cycle_delay_const_range_expression
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1154
Copyright © 2018 IEEE. All rights reserved.
```ebnf
cycle_delay_const_range_expression ::=
```

constant_expression : constant_expression
| constant_expression : $
```ebnf
expression_or_dist ::= expression [ dist { dist_list } ]
assertion_variable_declaration ::=
```

var_data_type list_of_variable_decl_assignments ;
A.2.11 Covergroup declarations
```ebnf
covergroup_declaration ::=
```

covergroup covergroup_identifier [ ( [ tf_port_list ] ) ] [ coverage_event ] ;
{ coverage_spec_or_option }
endgroup [ : covergroup_identifier ]
```ebnf
coverage_spec_or_option ::=
```

{attribute_instance} coverage_spec
| {attribute_instance} coverage_option ;
```ebnf
coverage_option ::=
```

option.member_identifier = expression
| type_option.member_identifier = constant_expression
```ebnf
coverage_spec ::=
```

cover_point
| cover_cross
```ebnf
coverage_event ::=
```

clocking_event
| with function sample ( [ tf_port_list ] )
| @@( block_event_expression )
```ebnf
block_event_expression ::=
```

block_event_expression or block_event_expression
| begin hierarchical_btf_identifier
| end hierarchical_btf_identifier
```ebnf
hierarchical_btf_identifier ::=
```

hierarchical_tf_identifier
| hierarchical_block_identifier
| [ hierarchical_identifier. | class_scope ] method_identifier
```ebnf
cover_point ::=
```

[ [ data_type_or_implicit ] cover_point_identifier : ] coverpoint expression [ iff ( expression ) ]
bins_or_empty
```ebnf
bins_or_empty ::=
```

{ {attribute_instance} { bins_or_options ; } }
|  ;
```ebnf
bins_or_options ::=
```

coverage_option
| [ wildcard ] bins_keyword bin_identifier [ [ [ covergroup_expression ] ] ] =
{ covergroup_range_list } [ with ( with_covergroup_expression ) ]
[ iff ( expression ) ]
| [ wildcard ] bins_keyword bin_identifier [ [ [ covergroup_expression ] ] ] =
cover_point_identifier with ( with_covergroup_expression ) [ iff ( expression ) ]
| [ wildcard ] bins_keyword bin_identifier [ [ [ covergroup_expression ] ] ] =
set_covergroup_expression [ iff ( expression ) ]
| [ wildcard] bins_keyword bin_identifier [ [ ] ] = trans_list [ iff ( expression ) ]
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1155
Copyright © 2018 IEEE. All rights reserved.
| bins_keyword bin_identifier [ [ [ covergroup_expression ] ] ] = default [ iff ( expression ) ]
| bins_keyword bin_identifier = default sequence [ iff ( expression ) ]
```ebnf
bins_keyword::= bins | illegal_bins | ignore_bins
trans_list ::= ( trans_set ) { , ( trans_set ) }
trans_set ::= trans_range_list { => trans_range_list }
trans_range_list ::=
```

trans_item
| trans_item [* repeat_range ]
| trans_item [–> repeat_range ]
| trans_item [= repeat_range ]
```ebnf
trans_item ::= covergroup_range_list
repeat_range ::=
```

covergroup_expression
| covergroup_expression : covergroup_expression
```ebnf
cover_cross ::=
```

[ cross_identifier : ] cross list_of_cross_items [ iff ( expression ) ] cross_body
```ebnf
list_of_cross_items ::= cross_item , cross_item { , cross_item }
cross_item ::=
```

cover_point_identifier
| variable_identifier
```ebnf
cross_body ::=
```

{ { cross_body_item ; } }
| ;
```ebnf
cross_body_item ::=
```

function_declaraton
| bins_selection_or_option ;
```ebnf
bins_selection_or_option ::=
```

{ attribute_instance } coverage_option
| { attribute_instance } bins_selection
```ebnf
bins_selection ::= bins_keyword bin_identifier = select_expression [ iff ( expression ) ]
select_expression24 ::=
```

select_condition
| ! select_condition
| select_expression && select_expression
| select_expression || select_expression
| ( select_expression )
| select_expression with ( with_covergroup_expression ) [ matches integer_covergroup_expression ]
| cross_identifier
| cross_set_expression [ matches integer_covergroup_expression ]
```ebnf
select_condition ::= binsof ( bins_expression ) [ intersect { covergroup_range_list } ]
bins_expression ::=
```

variable_identifier
| cover_point_identifier [ . bin_identifier ]
```ebnf
covergroup_range_list ::= covergroup_value_range { , covergroup_value_range }
covergroup_value_range ::=
```

covergroup_expression
| [ covergroup_expression : covergroup_expression ]25
```ebnf
with_covergroup_expression ::= covergroup_expression26
```

Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1156
Copyright © 2018 IEEE. All rights reserved.
```ebnf
set_covergroup_expression ::= covergroup_expression27
integer_covergroup_expression ::= covergroup_expression
cross_set_expression ::= covergroup_expression
covergroup_expression ::= expression28
```

A.2.12 Let declarations
```ebnf
let_declaration ::=
```

let let_identifier [ ( [ let_port_list ] ) ] = expression ;
```ebnf
let_identifier ::=
```

identifier
```ebnf
let_port_list ::=
```

let_port_item {, let_port_item}
```ebnf
let_port_item ::=
```

{ attribute_instance } let_formal_type formal_port_identifier { variable_dimension } [ = expression ]
```ebnf
let_formal_type ::=
```

data_type_or_implicit
| untyped
```ebnf
let_expression ::=
```

[ package_scope ] let_identifier [ ( [ let_list_of_arguments ] ) ]
```ebnf
let_list_of_arguments ::=
```

[ let_actual_arg ] {, [ let_actual_arg ] } {, . identifier ( [ let_actual_arg ] ) }
| . identifier ( [ let_actual_arg ] ) { , . identifier ( [ let_actual_arg ] ) }
```ebnf
let_actual_arg ::=
```

expression
A.3 Primitive instances
A.3.1 Primitive instantiation and instances
```ebnf
gate_instantiation ::=
```

cmos_switchtype [delay3] cmos_switch_instance { , cmos_switch_instance } ;
| enable_gatetype [drive_strength] [delay3] enable_gate_instance { , enable_gate_instance } ;
| mos_switchtype [delay3] mos_switch_instance { , mos_switch_instance } ;
| n_input_gatetype [drive_strength] [delay2] n_input_gate_instance { , n_input_gate_instance } ;
| n_output_gatetype [drive_strength] [delay2] n_output_gate_instance
{ , n_output_gate_instance } ;
| pass_en_switchtype [delay2] pass_enable_switch_instance { , pass_enable_switch_instance } ;
| pass_switchtype pass_switch_instance { , pass_switch_instance } ;
| pulldown [pulldown_strength] pull_gate_instance { , pull_gate_instance } ;
| pullup [pullup_strength] pull_gate_instance { , pull_gate_instance } ;
```ebnf
cmos_switch_instance ::= [ name_of_instance ] ( output_terminal , input_terminal ,
```

ncontrol_terminal , pcontrol_terminal )
```ebnf
enable_gate_instance ::= [ name_of_instance ] ( output_terminal , input_terminal , enable_terminal )
mos_switch_instance ::= [ name_of_instance ] ( output_terminal , input_terminal , enable_terminal )
n_input_gate_instance ::= [ name_of_instance ] ( output_terminal , input_terminal { , input_terminal } )
n_output_gate_instance ::= [ name_of_instance ] ( output_terminal { , output_terminal } ,
```

Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1157
Copyright © 2018 IEEE. All rights reserved.
input_terminal )
```ebnf
pass_switch_instance ::= [ name_of_instance ] ( inout_terminal , inout_terminal )
pass_enable_switch_instance ::= [ name_of_instance ] ( inout_terminal , inout_terminal ,
```

enable_terminal )
```ebnf
pull_gate_instance ::= [ name_of_instance ] ( output_terminal )
```

A.3.2 Primitive strengths
```ebnf
pulldown_strength ::=
```

( strength0 , strength1 )
| ( strength1 , strength0 )
| ( strength0 )
```ebnf
pullup_strength ::=
```

( strength0 , strength1 )
| ( strength1 , strength0 )
| ( strength1 )
A.3.3 Primitive terminals
```ebnf
enable_terminal ::= expression
inout_terminal ::= net_lvalue
input_terminal ::= expression
ncontrol_terminal ::= expression
output_terminal ::= net_lvalue
pcontrol_terminal ::= expression
```

A.3.4 Primitive gate and switch types
```ebnf
cmos_switchtype ::= cmos | rcmos
enable_gatetype ::= bufif0 | bufif1 | notif0 | notif1
mos_switchtype ::= nmos | pmos | rnmos | rpmos
n_input_gatetype ::= and | nand | or | nor | xor | xnor
n_output_gatetype ::= buf | not
pass_en_switchtype ::= tranif0 | tranif1 | rtranif1 | rtranif0
pass_switchtype ::= tran | rtran
```

A.4 Instantiations
A.4.1 Instantiation
A.4.1.1 Module instantiation
```ebnf
module_instantiation ::=
```

module_identifier [ parameter_value_assignment ] hierarchical_instance { , hierarchical_instance } ;
```ebnf
parameter_value_assignment ::= # ( [ list_of_parameter_assignments ] )
list_of_parameter_assignments ::=
```

ordered_parameter_assignment { , ordered_parameter_assignment }
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1158
Copyright © 2018 IEEE. All rights reserved.
| named_parameter_assignment { , named_parameter_assignment }
```ebnf
ordered_parameter_assignment ::= param_expression
named_parameter_assignment ::= . parameter_identifier ( [ param_expression ] )
hierarchical_instance ::= name_of_instance ( [ list_of_port_connections ] )
name_of_instance ::= instance_identifier { unpacked_dimension }
list_of_port_connections29 ::=
```

ordered_port_connection { , ordered_port_connection }
| named_port_connection { , named_port_connection }
```ebnf
ordered_port_connection ::= { attribute_instance } [ expression ]
named_port_connection ::=
```

{ attribute_instance } . port_identifier [ ( [ expression ] ) ]
| { attribute_instance } .*
A.4.1.2 Interface instantiation
```ebnf
interface_instantiation ::=
```

interface_identifier [ parameter_value_assignment ] hierarchical_instance { , hierarchical_instance } ;
A.4.1.3 Program instantiation
```ebnf
program_instantiation ::=
```

program_identifier [ parameter_value_assignment ] hierarchical_instance { , hierarchical_instance } ;
A.4.1.4 Checker instantiation
```ebnf
checker_instantiation ::=
 ps_checker_identifier name_of_instance ( [list_of_checker_port_connections] ) ;
list_of_checker_port_connections29 ::=
```

ordered_checker_port_connection { , ordered_checker_port_connection }
| named_checker_port_connection { , named_checker_port_connection }
```ebnf
ordered_checker_port_connection ::= { attribute_instance } [ property_actual_arg ]
named_checker_port_connection ::=
```

{ attribute_instance } . formal_port_identifier [ ( [ property_actual_arg ] ) ]
| { attribute_instance } .*
A.4.2 Generated instantiation
```ebnf
generate_region ::=
```

generate { generate_item } endgenerate
```ebnf
loop_generate_construct ::=
```

for ( genvar_initialization ; genvar_expression ; genvar_iteration )
generate_block
```ebnf
genvar_initialization ::=
```

[ genvar ] genvar_identifier = constant_expression
```ebnf
genvar_iteration ::=
```

genvar_identifier assignment_operator genvar_expression
| inc_or_dec_operator genvar_identifier
| genvar_identifier inc_or_dec_operator
```ebnf
conditional_generate_construct ::=
```

if_generate_construct
| case_generate_construct
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1159
Copyright © 2018 IEEE. All rights reserved.
```ebnf
if_generate_construct ::=
```

if ( constant_expression ) generate_block [ else generate_block ]
```ebnf
case_generate_construct ::=
```

case ( constant_expression ) case_generate_item { case_generate_item } endcase
```ebnf
case_generate_item ::=
```

constant_expression { , constant_expression } : generate_block
| default [ : ] generate_block
```ebnf
generate_block ::=
```

generate_item
| [ generate_block_identifier : ] begin [ : generate_block_identifier ]
{ generate_item }
end [ : generate_block_identifier ]
```ebnf
generate_item30 ::=
```

module_or_generate_item
| interface_or_generate_item
| checker_or_generate_item
A.5 UDP declaration and instantiation
A.5.1 UDP declaration
```ebnf
udp_nonansi_declaration ::=
```

{ attribute_instance } primitive udp_identifier ( udp_port_list ) ;
```ebnf
udp_ansi_declaration ::=
```

{ attribute_instance } primitive udp_identifier ( udp_declaration_port_list ) ;
```ebnf
udp_declaration ::=
```

udp_nonansi_declaration udp_port_declaration { udp_port_declaration }
udp_body
endprimitive [ : udp_identifier ]
| udp_ansi_declaration
udp_body
endprimitive [ : udp_identifier ]
| extern udp_nonansi_declaration
| extern udp_ansi_declaration
| { attribute_instance } primitive udp_identifier ( .* ) ;
{ udp_port_declaration }
udp_body
endprimitive [ : udp_identifier ]
A.5.2 UDP ports
```ebnf
udp_port_list ::= output_port_identifier , input_port_identifier { , input_port_identifier }
udp_declaration_port_list ::= udp_output_declaration , udp_input_declaration { , udp_input_declaration }
udp_port_declaration ::=
```

udp_output_declaration ;
| udp_input_declaration ;
| udp_reg_declaration ;
```ebnf
udp_output_declaration ::=
```

{ attribute_instance } output port_identifier
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1160
Copyright © 2018 IEEE. All rights reserved.
| { attribute_instance } output reg port_identifier [ = constant_expression ]
```ebnf
udp_input_declaration ::= { attribute_instance } input list_of_udp_port_identifiers
udp_reg_declaration ::= { attribute_instance } reg variable_identifier
```

A.5.3 UDP body
```ebnf
udp_body ::= combinational_body | sequential_body
combinational_body ::= table combinational_entry { combinational_entry } endtable
combinational_entry ::= level_input_list : output_symbol ;
sequential_body ::= [ udp_initial_statement ] table sequential_entry { sequential_entry } endtable
udp_initial_statement ::= initial output_port_identifier = init_val ;
init_val ::= 1'b0 | 1'b1 | 1'bx | 1'bX | 1'B0 | 1'B1 | 1'Bx | 1'BX | 1 | 0
sequential_entry ::= seq_input_list : current_state : next_state ;
seq_input_list ::= level_input_list | edge_input_list
level_input_list ::= level_symbol { level_symbol }
edge_input_list ::= { level_symbol } edge_indicator { level_symbol }
edge_indicator ::= ( level_symbol level_symbol ) | edge_symbol
current_state ::= level_symbol
next_state ::= output_symbol | -
output_symbol ::= 0 | 1 | x | X
level_symbol ::= 0 | 1 | x | X | ? | b | B
edge_symbol ::= r | R | f | F | p | P | n | N | *
```

A.5.4 UDP instantiation
```ebnf
udp_instantiation ::= udp_identifier [ drive_strength ] [ delay2 ] udp_instance { , udp_instance } ;
udp_instance ::= [ name_of_instance ] ( output_terminal , input_terminal { , input_terminal } )
```

A.6 Behavioral statements
A.6.1 Continuous assignment and net alias statements
```ebnf
continuous_assign ::=
```

assign [ drive_strength ] [ delay3 ] list_of_net_assignments ;
| assign [ delay_control ] list_of_variable_assignments ;
```ebnf
list_of_net_assignments ::= net_assignment { , net_assignment }
list_of_variable_assignments ::= variable_assignment { , variable_assignment }
net_alias ::= alias net_lvalue = net_lvalue { = net_lvalue } ;
net_assignment ::= net_lvalue = expression
```

Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1161
Copyright © 2018 IEEE. All rights reserved.
A.6.2 Procedural blocks and assignments
```ebnf
initial_construct ::= initial statement_or_null
always_construct ::= always_keyword statement
always_keyword ::= always | always_comb | always_latch | always_ff
final_construct ::= final function_statement
blocking_assignment ::=
```

variable_lvalue = delay_or_event_control expression
| nonrange_variable_lvalue = dynamic_array_new
| [ implicit_class_handle . | class_scope | package_scope ] hierarchical_variable_identifier
select = class_new
| operator_assignment
```ebnf
operator_assignment ::= variable_lvalue assignment_operator expression
assignment_operator ::=
```

= | += | -= | *= | /= | %= | &= | |= | ^= | <<= | >>= | <<<= | >>>=
```ebnf
nonblocking_assignment ::=
```

variable_lvalue <= [ delay_or_event_control ] expression
```ebnf
procedural_continuous_assignment ::=
```

assign variable_assignment
| deassign variable_lvalue
| force variable_assignment
| force net_assignment
| release variable_lvalue
| release net_lvalue
```ebnf
variable_assignment ::= variable_lvalue = expression
```

A.6.3 Parallel and sequential blocks
```ebnf
action_block ::=
```

statement_or_null
| [ statement ] else statement_or_null
```ebnf
seq_block ::=
```

begin [ : block_identifier ] { block_item_declaration } { statement_or_null }
end [ : block_identifier ]
```ebnf
par_block ::=
```

fork [ : block_identifier ] { block_item_declaration } { statement_or_null }
join_keyword [ : block_identifier ]
```ebnf
join_keyword ::= join | join_any | join_none
```

A.6.4 Statements
```ebnf
statement_or_null ::=
```

statement
| { attribute_instance } ;
```ebnf
statement ::= [ block_identifier : ] { attribute_instance } statement_item
statement_item ::=
```

blocking_assignment ;
| nonblocking_assignment ;
| procedural_continuous_assignment ;
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1162
Copyright © 2018 IEEE. All rights reserved.
| case_statement
| conditional_statement
| inc_or_dec_expression ;
| subroutine_call_statement
| disable_statement
| event_trigger
| loop_statement
| jump_statement
| par_block
| procedural_timing_control_statement
| seq_block
| wait_statement
| procedural_assertion_statement
| clocking_drive ;
| randsequence_statement
| randcase_statement
| expect_property_statement
```ebnf
function_statement ::= statement
function_statement_or_null ::=
```

function_statement
| { attribute_instance } ;
```ebnf
variable_identifier_list ::= variable_identifier { , variable_identifier }
```

A.6.5 Timing control statements
```ebnf
procedural_timing_control_statement ::=
```

procedural_timing_control statement_or_null
```ebnf
delay_or_event_control ::=
```

delay_control
| event_control
| repeat ( expression ) event_control
```ebnf
delay_control ::=
```

# delay_value
| # ( mintypmax_expression )
```ebnf
event_control ::=
```

@ hierarchical_event_identifier
| @ ( event_expression )
| @*
| @ (*)
| @ ps_or_hierarchical_sequence_identifier
```ebnf
event_expression31 ::=
```

[ edge_identifier ] expression [ iff expression ]
| sequence_instance [ iff expression ]
| event_expression or event_expression
| event_expression , event_expression
| ( event_expression )
```ebnf
procedural_timing_control ::=
```

delay_control
| event_control
| cycle_delay
```ebnf
jump_statement ::=
```

Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1163
Copyright © 2018 IEEE. All rights reserved.
return [ expression ] ;
| break ;
| continue ;
```ebnf
wait_statement ::=
```

wait ( expression ) statement_or_null
| wait fork ;
| wait_order ( hierarchical_identifier { , hierarchical_identifier } ) action_block
```ebnf
event_trigger ::=
```

-> hierarchical_event_identifier ;
|->> [ delay_or_event_control ] hierarchical_event_identifier ;
```ebnf
disable_statement ::=
```

disable hierarchical_task_identifier ;
| disable hierarchical_block_identifier ;
| disable fork ;
A.6.6 Conditional statements
```ebnf
conditional_statement ::=
```

[ unique_priority ] if ( cond_predicate ) statement_or_null
{else if ( cond_predicate ) statement_or_null }
[ else statement_or_null ]
```ebnf
unique_priority ::= unique | unique0 | priority
cond_predicate ::=
```

expression_or_cond_pattern { &&& expression_or_cond_pattern }
```ebnf
expression_or_cond_pattern ::=
```

expression | cond_pattern
```ebnf
cond_pattern ::= expression matches pattern
```

A.6.7 Case statements
```ebnf
case_statement ::=
```

[ unique_priority ] case_keyword ( case_expression )
case_item { case_item } endcase
| [ unique_priority ] case_keyword (case_expression )matches
case_pattern_item { case_pattern_item } endcase
| [ unique_priority ] case ( case_expression ) inside
case_inside_item { case_inside_item } endcase
```ebnf
case_keyword ::= case | casez | casex
case_expression ::= expression
case_item ::=
```

case_item_expression { , case_item_expression } : statement_or_null
| default [ : ] statement_or_null
```ebnf
case_pattern_item ::=
```

pattern [ &&& expression ] : statement_or_null
| default [ : ] statement_or_null
```ebnf
case_inside_item ::=
```

open_range_list : statement_or_null
| default [ : ] statement_or_null
```ebnf
case_item_expression ::= expression
```

Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1164
Copyright © 2018 IEEE. All rights reserved.
```ebnf
randcase_statement ::=
```

randcase randcase_item { randcase_item } endcase
```ebnf
randcase_item ::= expression : statement_or_null
open_range_list ::= open_value_range { , open_value_range }
open_value_range ::= value_range25
```

A.6.7.1 Patterns
```ebnf
pattern ::=
```

. variable_identifier
| .*
| constant_expression
| tagged member_identifier [ pattern ]
| '{ pattern { , pattern } }
| '{ member_identifier : pattern { , member_identifier : pattern } }
```ebnf
assignment_pattern ::=
```

'{ expression { , expression } }
| '{ structure_pattern_key : expression { , structure_pattern_key : expression } }
| '{ array_pattern_key : expression { , array_pattern_key : expression } }
| '{ constant_expression { expression { , expression } } }
```ebnf
structure_pattern_key ::= member_identifier | assignment_pattern_key
array_pattern_key ::= constant_expression | assignment_pattern_key
assignment_pattern_key ::= simple_type | default
assignment_pattern_expression ::=
```

[ assignment_pattern_expression_type ] assignment_pattern
```ebnf
assignment_pattern_expression_type ::=
```

ps_type_identifier
| ps_parameter_identifier
| integer_atom_type
| type_reference
```ebnf
constant_assignment_pattern_expression32 ::= assignment_pattern_expression
assignment_pattern_net_lvalue ::=
```

'{ net_lvalue {, net_lvalue } }
```ebnf
assignment_pattern_variable_lvalue ::=
```

'{ variable_lvalue {, variable_lvalue } }
A.6.8 Looping statements
```ebnf
loop_statement ::=
```

forever statement_or_null
| repeat ( expression ) statement_or_null
| while ( expression ) statement_or_null
| for ( [ for_initialization ] ; [ expression ] ; [ for_step ] )
statement_or_null
| do statement_or_null while ( expression ) ;
| foreach ( ps_or_hierarchical_array_identifier [ loop_variables ] ) statement
```ebnf
for_initialization ::=
```

list_of_variable_assignments
| for_variable_declaration { , for_variable_declaration }
```ebnf
for_variable_declaration ::=
```

Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1165
Copyright © 2018 IEEE. All rights reserved.
[ var ] data_type variable_identifier = expression { , variable_identifier = expression }14
```ebnf
for_step ::= for_step_assignment { , for_step_assignment }
for_step_assignment ::=
```

operator_assignment
| inc_or_dec_expression
| function_subroutine_call
```ebnf
loop_variables ::= [ index_variable_identifier ] { , [ index_variable_identifier ] }
```

A.6.9 Subroutine call statements
```ebnf
subroutine_call_statement ::=
```

subroutine_call ;
| void ' ( function_subroutine_call ) ;
A.6.10 Assertion statements
```ebnf
assertion_item ::=
```

concurrent_assertion_item
| deferred_immediate_assertion_item
```ebnf
deferred_immediate_assertion_item ::= [ block_identifier : ] deferred_immediate_assertion_statement
procedural_assertion_statement ::=
```

concurrent_assertion_statement
| immediate_assertion_statement
| checker_instantiation
```ebnf
immediate_assertion_statement ::=
```

simple_immediate_assertion_statement
| deferred_immediate_assertion_statement
```ebnf
simple_immediate_assertion_statement ::=
```

simple_immediate_assert_statement
| simple_immediate_assume_statement
| simple_immediate_cover_statement
```ebnf
simple_immediate_assert_statement ::=
```

assert ( expression ) action_block
```ebnf
simple_immediate_assume_statement ::=
```

assume ( expression ) action_block
```ebnf
simple_immediate_cover_statement ::=
```

cover ( expression ) statement_or_null
```ebnf
deferred_immediate_assertion_statement ::=
```

deferred_immediate_assert_statement
| deferred_immediate_assume_statement
| deferred_immediate_cover_statement
```ebnf
deferred_immediate_assert_statement ::=
```

assert #0 ( expression ) action_block
| assert final ( expression ) action_block
```ebnf
deferred_immediate_assume_statement ::=
```

assume #0 ( expression ) action_block
| assume final ( expression ) action_block
```ebnf
deferred_immediate_cover_statement ::=
```

cover #0 ( expression ) statement_or_null
| cover final ( expression ) statement_or_null
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1166
Copyright © 2018 IEEE. All rights reserved.
A.6.11 Clocking block
```ebnf
clocking_declaration ::= [ default ] clocking [ clocking_identifier ] clocking_event ;
```

{ clocking_item }
endclocking [ : clocking_identifier ]
| global clocking [ clocking_identifier ] clocking_event ; endclocking [ : clocking_identifier ]
```ebnf
clocking_event ::=
```

@ identifier
| @ ( event_expression )
```ebnf
clocking_item ::=
```

default default_skew ;
| clocking_direction list_of_clocking_decl_assign ;
| { attribute_instance } assertion_item_declaration
```ebnf
default_skew ::=
```

input clocking_skew
| output clocking_skew
| input clocking_skew output clocking_skew
```ebnf
clocking_direction ::=
```

input [ clocking_skew ]
| output [ clocking_skew ]
| input [ clocking_skew ] output [ clocking_skew ]
| inout
```ebnf
list_of_clocking_decl_assign ::= clocking_decl_assign { , clocking_decl_assign }
clocking_decl_assign ::= signal_identifier [ = expression ]
clocking_skew ::=
```

edge_identifier [ delay_control ]
| delay_control
```ebnf
clocking_drive ::=
```

clockvar_expression <= [ cycle_delay ] expression
```ebnf
cycle_delay ::=
```

## integral_number
| ## identifier
| ## ( expression )
```ebnf
clockvar ::= hierarchical_identifier
clockvar_expression ::= clockvar select
```

A.6.12 Randsequence
```ebnf
randsequence_statement ::= randsequence ( [ production_identifier ] )
```

production { production }
endsequence
```ebnf
production ::= [ data_type_or_void ] production_identifier [ ( tf_port_list ) ] : rs_rule { | rs_rule } ;
rs_rule ::= rs_production_list [ := weight_specification [ rs_code_block ] ]
rs_production_list ::=
```

rs_prod { rs_prod }
| rand join [ ( expression ) ] production_item production_item { production_item }
```ebnf
weight_specification ::=
```

integral_number
| ps_identifier
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1167
Copyright © 2018 IEEE. All rights reserved.
|  ( expression )
```ebnf
rs_code_block ::= { { data_declaration } { statement_or_null } }
rs_prod ::=
```

production_item
| rs_code_block
| rs_if_else
| rs_repeat
| rs_case
```ebnf
production_item ::= production_identifier [ ( list_of_arguments ) ]
rs_if_else ::= if ( expression ) production_item [ else production_item ]
rs_repeat ::= repeat ( expression ) production_item
rs_case ::= case ( case_expression ) rs_case_item { rs_case_item } endcase
rs_case_item ::=
```

case_item_expression { , case_item_expression } : production_item ;
| default [ : ] production_item ;
A.7 Specify section
A.7.1 Specify block declaration
```ebnf
specify_block ::= specify { specify_item } endspecify
specify_item ::=
```

specparam_declaration
| pulsestyle_declaration
| showcancelled_declaration
| path_declaration
| system_timing_check
```ebnf
pulsestyle_declaration ::=
```

pulsestyle_onevent list_of_path_outputs ;
| pulsestyle_ondetect list_of_path_outputs ;
```ebnf
showcancelled_declaration ::=
```

showcancelled list_of_path_outputs ;
| noshowcancelled list_of_path_outputs ;
A.7.2 Specify path declarations
```ebnf
path_declaration ::=
```

simple_path_declaration ;
| edge_sensitive_path_declaration ;
| state_dependent_path_declaration ;
```ebnf
simple_path_declaration ::=
```

parallel_path_description = path_delay_value
| full_path_description = path_delay_value
```ebnf
parallel_path_description ::=
```

( specify_input_terminal_descriptor [ polarity_operator ] => specify_output_terminal_descriptor )
```ebnf
full_path_description ::=
```

( list_of_path_inputs [ polarity_operator ] *> list_of_path_outputs )
```ebnf
list_of_path_inputs ::=
```

Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1168
Copyright © 2018 IEEE. All rights reserved.
specify_input_terminal_descriptor { , specify_input_terminal_descriptor }
```ebnf
list_of_path_outputs ::=
```

specify_output_terminal_descriptor { , specify_output_terminal_descriptor }
A.7.3 Specify block terminals
```ebnf
specify_input_terminal_descriptor ::=
```

input_identifier [ [ constant_range_expression ] ]
```ebnf
specify_output_terminal_descriptor ::=
```

output_identifier [ [ constant_range_expression ] ]
```ebnf
input_identifier ::= input_port_identifier | inout_port_identifier | interface_identifier.port_identifier
output_identifier ::= output_port_identifier | inout_port_identifier | interface_identifier.port_identifier
```

A.7.4 Specify path delays
```ebnf
path_delay_value ::=
```

list_of_path_delay_expressions
| ( list_of_path_delay_expressions )
```ebnf
list_of_path_delay_expressions ::=
```

t_path_delay_expression
| trise_path_delay_expression , tfall_path_delay_expression
| trise_path_delay_expression , tfall_path_delay_expression , tz_path_delay_expression
| t01_path_delay_expression , t10_path_delay_expression , t0z_path_delay_expression ,
tz1_path_delay_expression , t1z_path_delay_expression , tz0_path_delay_expression
| t01_path_delay_expression , t10_path_delay_expression , t0z_path_delay_expression ,
tz1_path_delay_expression , t1z_path_delay_expression , tz0_path_delay_expression ,
t0x_path_delay_expression , tx1_path_delay_expression , t1x_path_delay_expression ,
tx0_path_delay_expression , txz_path_delay_expression , tzx_path_delay_expression
```ebnf
t_path_delay_expression ::= path_delay_expression
trise_path_delay_expression ::= path_delay_expression
tfall_path_delay_expression ::= path_delay_expression
tz_path_delay_expression ::= path_delay_expression
t01_path_delay_expression ::= path_delay_expression
t10_path_delay_expression ::= path_delay_expression
t0z_path_delay_expression ::= path_delay_expression
tz1_path_delay_expression ::= path_delay_expression
t1z_path_delay_expression ::= path_delay_expression
tz0_path_delay_expression ::= path_delay_expression
t0x_path_delay_expression ::= path_delay_expression
tx1_path_delay_expression ::= path_delay_expression
t1x_path_delay_expression ::= path_delay_expression
tx0_path_delay_expression ::= path_delay_expression
txz_path_delay_expression ::= path_delay_expression
tzx_path_delay_expression ::= path_delay_expression
path_delay_expression ::= constant_mintypmax_expression
edge_sensitive_path_declaration ::=
```

parallel_edge_sensitive_path_description = path_delay_value
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1169
Copyright © 2018 IEEE. All rights reserved.
| full_edge_sensitive_path_description = path_delay_value
```ebnf
parallel_edge_sensitive_path_description ::=
```

( [ edge_identifier ] specify_input_terminal_descriptor [ polarity_operator ] =>
( specify_output_terminal_descriptor [ polarity_operator ] : data_source_expression ) )
```ebnf
full_edge_sensitive_path_description ::=
```

( [ edge_identifier ] list_of_path_inputs [ polarity_operator ] *>
( list_of_path_outputs [ polarity_operator ] : data_source_expression ) )
```ebnf
data_source_expression ::= expression
edge_identifier ::= posedge | negedge | edge
state_dependent_path_declaration ::=
```

if ( module_path_expression ) simple_path_declaration
| if ( module_path_expression ) edge_sensitive_path_declaration
| ifnone simple_path_declaration
```ebnf
polarity_operator ::= + | -
```

A.7.5 System timing checks
A.7.5.1 System timing check commands
```ebnf
system_timing_check ::=
```

$setup_timing_check
| $hold_timing_check
| $setuphold_timing_check
| $recovery_timing_check
| $removal_timing_check
| $recrem_timing_check
| $skew_timing_check
| $timeskew_timing_check
| $fullskew_timing_check
| $period_timing_check
| $width_timing_check
| $nochange_timing_check
```ebnf
$setup_timing_check ::=
```

$setup ( data_event , reference_event , timing_check_limit [ , [ notifier ] ] ) ;
```ebnf
$hold_timing_check ::=
```

$hold ( reference_event , data_event , timing_check_limit [ , [ notifier ] ] ) ;
```ebnf
$setuphold_timing_check ::=
```

$setuphold ( reference_event , data_event , timing_check_limit , timing_check_limit
[ , [ notifier ] [ , [ timestamp_condition ] [ , [ timecheck_condition ]
[ , [ delayed_reference ] [ , [ delayed_data ] ] ] ] ] ] ) ;
```ebnf
$recovery_timing_check ::=
```

$recovery ( reference_event , data_event , timing_check_limit [ , [ notifier ] ] ) ;
```ebnf
$removal_timing_check ::=
```

$removal ( reference_event , data_event , timing_check_limit [ , [ notifier ] ] ) ;
```ebnf
$recrem_timing_check ::=
```

$recrem ( reference_event , data_event , timing_check_limit , timing_check_limit
[ , [ notifier ] [ , [ timestamp_condition ] [ , [ timecheck_condition ]
[ , [ delayed_reference ] [ , [ delayed_data ] ] ] ] ] ] ) ;
```ebnf
$skew_timing_check ::=
```

$skew ( reference_event , data_event , timing_check_limit [ , [ notifier ] ] ) ;
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1170
Copyright © 2018 IEEE. All rights reserved.
```ebnf
$timeskew_timing_check ::=
```

$timeskew ( reference_event , data_event , timing_check_limit
[ , [ notifier ] [ , [ event_based_flag ] [ , [ remain_active_flag ] ] ] ] ) ;
```ebnf
$fullskew_timing_check ::=
```

$fullskew ( reference_event , data_event , timing_check_limit , timing_check_limit
[ , [ notifier ] [ , [ event_based_flag ] [ , [ remain_active_flag ] ] ] ] ) ;
```ebnf
$period_timing_check ::=
```

$period ( controlled_reference_event , timing_check_limit [ , [ notifier ] ] ) ;
```ebnf
$width_timing_check ::=
```

$width ( controlled_reference_event , timing_check_limit , threshold [ , [ notifier ] ] ) ;
```ebnf
$nochange_timing_check ::=
```

$nochange ( reference_event , data_event , start_edge_offset , end_edge_offset [ , [ notifier ] ] );
A.7.5.2 System timing check command arguments
```ebnf
timecheck_condition ::= mintypmax_expression
controlled_reference_event ::= controlled_timing_check_event
data_event ::= timing_check_event
delayed_data ::=
```

terminal_identifier
| terminal_identifier [ constant_mintypmax_expression ]
```ebnf
delayed_reference ::=
```

terminal_identifier
| terminal_identifier [ constant_mintypmax_expression ]
```ebnf
end_edge_offset ::= mintypmax_expression
event_based_flag ::= constant_expression
notifier ::= variable_identifier
reference_event ::= timing_check_event
remain_active_flag ::= constant_mintypmax_expression
timestamp_condition ::= mintypmax_expression
start_edge_offset ::= mintypmax_expression
threshold ::= constant_expression
timing_check_limit ::= expression
```

A.7.5.3 System timing check event definitions
```ebnf
timing_check_event ::=
```

[timing_check_event_control] specify_terminal_descriptor [ &&& timing_check_condition ]
```ebnf
controlled_timing_check_event ::=
```

timing_check_event_control specify_terminal_descriptor [ &&& timing_check_condition ]
```ebnf
timing_check_event_control ::=
```

posedge
| negedge
| edge
| edge_control_specifier
```ebnf
specify_terminal_descriptor ::=
```

specify_input_terminal_descriptor
| specify_output_terminal_descriptor
```ebnf
edge_control_specifier ::= edge [ edge_descriptor { , edge_descriptor } ]
```

Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1171
Copyright © 2018 IEEE. All rights reserved.
```ebnf
edge_descriptor33 ::= 01 | 10 | z_or_x zero_or_one | zero_or_one z_or_x
zero_or_one ::= 0 | 1
z_or_x ::= x | X | z | Z
timing_check_condition ::=
```

scalar_timing_check_condition
| ( scalar_timing_check_condition )
```ebnf
scalar_timing_check_condition ::=
```

expression
| ~ expression
| expression == scalar_constant
| expression === scalar_constant
| expression != scalar_constant
| expression !== scalar_constant
```ebnf
scalar_constant ::= 1'b0 | 1'b1 | 1'B0 | 1'B1 | 'b0 | 'b1 | 'B0 | 'B1 | 1 | 0
```

A.8 Expressions
A.8.1 Concatenations
```ebnf
concatenation ::=
```

{ expression { , expression } }
```ebnf
constant_concatenation ::=
```

{ constant_expression { , constant_expression } }
```ebnf
constant_multiple_concatenation ::= { constant_expression constant_concatenation }
module_path_concatenation ::= { module_path_expression { , module_path_expression } }
module_path_multiple_concatenation ::= { constant_expression module_path_concatenation }
multiple_concatenation ::= { expression concatenation }34
streaming_concatenation ::= { stream_operator [ slice_size ] stream_concatenation }
stream_operator ::= >> | <<
slice_size ::= simple_type | constant_expression
stream_concatenation ::= { stream_expression { , stream_expression } }
stream_expression ::= expression [ with [ array_range_expression ] ]
array_range_expression ::=
```

expression
| expression : expression
| expression +: expression
| expression -: expression
```ebnf
empty_unpacked_array_concatenation35 ::= { }
```

A.8.2 Subroutine calls
```ebnf
constant_function_call ::= function_subroutine_call36
tf_call37 ::= ps_or_hierarchical_tf_identifier { attribute_instance } [ ( list_of_arguments ) ]
system_tf_call ::=
```

system_tf_identifier [ ( list_of_arguments ) ]
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1172
Copyright © 2018 IEEE. All rights reserved.
| system_tf_identifier ( data_type [ , expression ] )
| system_tf_identifier ( expression { , [ expression ] } [ , [ clocking_event ] ] )
```ebnf
subroutine_call ::=
```

tf_call
| system_tf_call
| method_call
| [ std :: ] randomize_call
```ebnf
function_subroutine_call ::= subroutine_call
list_of_arguments ::=
```

[ expression ] { , [ expression ] } { , . identifier ( [ expression ] ) }
| . identifier ( [ expression ] ) { , . identifier ( [ expression ] ) }
```ebnf
method_call ::= method_call_root . method_call_body
method_call_body ::=
```

method_identifier { attribute_instance } [ ( list_of_arguments ) ]
| built_in_method_call
```ebnf
built_in_method_call ::=
```

array_manipulation_call
| randomize_call
```ebnf
array_manipulation_call ::=
```

array_method_name { attribute_instance }
[ ( list_of_arguments ) ]
[ with ( expression ) ]
```ebnf
randomize_call ::=
 randomize { attribute_instance }
```

[ ( [ variable_identifier_list | null ] ) ]
[ with [ ( [ identifier_list ] ) ] constraint_block ]38
```ebnf
method_call_root ::= primary | implicit_class_handle
array_method_name ::=
```

method_identifier | unique | and | or | xor
A.8.3 Expressions
```ebnf
inc_or_dec_expression ::=
```

inc_or_dec_operator { attribute_instance } variable_lvalue
| variable_lvalue { attribute_instance } inc_or_dec_operator
```ebnf
conditional_expression ::= cond_predicate ? { attribute_instance } expression : expression
constant_expression ::=
```

constant_primary
| unary_operator { attribute_instance } constant_primary
| constant_expression binary_operator { attribute_instance } constant_expression
| constant_expression ? { attribute_instance } constant_expression : constant_expression
```ebnf
constant_mintypmax_expression ::=
```

constant_expression
| constant_expression : constant_expression : constant_expression
```ebnf
constant_param_expression ::=
```

constant_mintypmax_expression | data_type | $
```ebnf
param_expression ::= mintypmax_expression | data_type | $
constant_range_expression ::=
```

constant_expression
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1173
Copyright © 2018 IEEE. All rights reserved.
| constant_part_select_range
```ebnf
constant_part_select_range ::=
```

constant_range
| constant_indexed_range
```ebnf
constant_range ::= constant_expression : constant_expression
constant_indexed_range ::=
```

constant_expression +: constant_expression
| constant_expression -: constant_expression
```ebnf
expression ::=
```

primary
| unary_operator { attribute_instance } primary
| inc_or_dec_expression
| ( operator_assignment )
| expression binary_operator { attribute_instance } expression
| conditional_expression
| inside_expression
| tagged_union_expression
```ebnf
tagged_union_expression ::=
```

tagged member_identifier [ expression ]
```ebnf
inside_expression ::= expression inside { open_range_list }
value_range ::=
```

expression
| [ expression : expression ]
```ebnf
mintypmax_expression ::=
```

expression
| expression : expression : expression
```ebnf
module_path_conditional_expression ::= module_path_expression ? { attribute_instance }
```

module_path_expression : module_path_expression
```ebnf
module_path_expression ::=
```

module_path_primary
| unary_module_path_operator { attribute_instance } module_path_primary
| module_path_expression binary_module_path_operator { attribute_instance }
module_path_expression
| module_path_conditional_expression
```ebnf
module_path_mintypmax_expression ::=
```

module_path_expression
| module_path_expression : module_path_expression : module_path_expression
```ebnf
part_select_range ::= constant_range | indexed_range
indexed_range ::=
```

expression +: constant_expression
| expression -: constant_expression
```ebnf
genvar_expression ::= constant_expression
```

A.8.4 Primaries
```ebnf
constant_primary ::=
```

primary_literal
| ps_parameter_identifier constant_select
| specparam_identifier [ [ constant_range_expression ] ]
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1174
Copyright © 2018 IEEE. All rights reserved.
| genvar_identifier39
| formal_port_identifier constant_select
| [ package_scope | class_scope ] enum_identifier
| constant_concatenation [ [ constant_range_expression ] ]
| constant_multiple_concatenation [ [ constant_range_expression ] ]
| constant_function_call
| constant_let_expression
| ( constant_mintypmax_expression )
| constant_cast
| constant_assignment_pattern_expression
| type_reference40
| null
```ebnf
module_path_primary ::=
```

number
| identifier
| module_path_concatenation
| module_path_multiple_concatenation
| function_subroutine_call
| ( module_path_mintypmax_expression )
```ebnf
primary ::=
```

primary_literal
| [ class_qualifier | package_scope ] hierarchical_identifier select
| empty_unpacked_array_concatenation
| concatenation [ [ range_expression ] ]
| multiple_concatenation [ [ range_expression ] ]
| function_subroutine_call
| let_expression
| ( mintypmax_expression )
| cast
| assignment_pattern_expression
| streaming_concatenation
| sequence_method_call
| this41
| $42
| null
class_qualifier := [ local ::43 ] [ implicit_class_handle . | class_scope ]
```ebnf
range_expression ::=
```

expression
| part_select_range
```ebnf
primary_literal ::= number | time_literal | unbased_unsized_literal | string_literal
time_literal44 ::=
```

unsigned_number time_unit
| fixed_point_number time_unit
```ebnf
time_unit ::= s | ms | us | ns | ps | fs
implicit_class_handle41 ::= this | super | this . super
bit_select ::= { [ expression ] }
select ::=
```

[ { . member_identifier bit_select } . member_identifier ] bit_select [ [ part_select_range ] ]
```ebnf
nonrange_select ::=
```

Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1175
Copyright © 2018 IEEE. All rights reserved.
[ { . member_identifier bit_select } . member_identifier ] bit_select
```ebnf
constant_bit_select ::= { [ constant_expression ] }
constant_select ::=
```

[ { . member_identifier constant_bit_select } . member_identifier ] constant_bit_select
[ [ constant_part_select_range ] ]
```ebnf
constant_cast ::=
```

casting_type ' ( constant_expression )
```ebnf
constant_let_expression ::= let_expression45
cast ::=
```

casting_type ' ( expression )
A.8.5 Expression left-side values
```ebnf
net_lvalue ::=
```

ps_or_hierarchical_net_identifier constant_select
| { net_lvalue { , net_lvalue } }
| [ assignment_pattern_expression_type ] assignment_pattern_net_lvalue
```ebnf
variable_lvalue ::=
```

[ implicit_class_handle .  | package_scope ] hierarchical_variable_identifier select46
| { variable_lvalue { , variable_lvalue } }
| [ assignment_pattern_expression_type ] assignment_pattern_variable_lvalue
| streaming_concatenation47
```ebnf
nonrange_variable_lvalue ::=
```

[ implicit_class_handle . | package_scope ] hierarchical_variable_identifier nonrange_select
A.8.6 Operators
```ebnf
unary_operator ::=
```

+ | - | ! | ~ | & | ~& | | | ~| | ^ | ~^ | ^~
```ebnf
binary_operator ::=
```

+ | - | * | / | % | == | != | === | !== | ==? | !=? | && | || | **
| < | <= | > | >= | & | | | ^ | ^~ | ~^ | >> | << | >>> | <<<
| -> | <->
```ebnf
inc_or_dec_operator ::= ++ | --
unary_module_path_operator ::=
          ! | ~ | & | ~& | | | ~| | ^ | ~^ | ^~
binary_module_path_operator ::=
          == | != | && | || | & | | | ^ | ^~ | ~^
```

A.8.7 Numbers
```ebnf
number ::=
```

integral_number
| real_number
```ebnf
integral_number ::=
```

decimal_number
| octal_number
| binary_number
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1176
Copyright © 2018 IEEE. All rights reserved.
| hex_number
```ebnf
decimal_number ::=
```

unsigned_number
| [ size ] decimal_base unsigned_number
| [ size ] decimal_base x_digit { _ }
| [ size ] decimal_base z_digit { _ }
```ebnf
binary_number ::= [ size ] binary_base binary_value
octal_number ::= [ size ] octal_base octal_value
hex_number ::= [ size ] hex_base hex_value
sign ::= + | -
size ::= non_zero_unsigned_number
non_zero_unsigned_number33 ::= non_zero_decimal_digit { _ | decimal_digit}
real_number33 ::=
```

fixed_point_number
| unsigned_number [ . unsigned_number ] exp [ sign ] unsigned_number
```ebnf
fixed_point_number33 ::= unsigned_number . unsigned_number
exp ::= e | E
unsigned_number33 ::= decimal_digit { _ | decimal_digit }
binary_value33 ::= binary_digit { _ | binary_digit }
octal_value33 ::= octal_digit { _ | octal_digit }
hex_value33 ::= hex_digit { _ | hex_digit }
decimal_base33 ::= '[s|S]d | '[s|S]D
binary_base33 ::= '[s|S]b | '[s|S]B
octal_base33 ::= '[s|S]o | '[s|S]O
hex_base33 ::= '[s|S]h | '[s|S]H
non_zero_decimal_digit ::= 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9
decimal_digit ::= 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9
binary_digit ::= x_digit | z_digit | 0 | 1
octal_digit ::= x_digit | z_digit | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7
hex_digit ::= x_digit | z_digit | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | a | b | c | d | e | f | A | B | C | D | E | F
x_digit ::= x | X
z_digit ::= z | Z | ?
unbased_unsized_literal ::= '0 | '1 | 'z_or_x 48
```

A.8.8 Strings
```ebnf
string_literal ::= " { Any_ASCII_Characters } "
```

Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1177
Copyright © 2018 IEEE. All rights reserved.
A.9 General
A.9.1 Attributes
```ebnf
attribute_instance ::= (* attr_spec { , attr_spec } *)
attr_spec ::= attr_name [ = constant_expression ]
attr_name ::= identifier
```

A.9.2 Comments
```ebnf
comment ::=
```

one_line_comment
| block_comment
```ebnf
one_line_comment ::= // comment_text \n
block_comment ::= /* comment_text */
comment_text ::= { Any_ASCII_character }
```

A.9.3 Identifiers
```ebnf
array_identifier ::= identifier
block_identifier ::= identifier
bin_identifier ::= identifier
c_identifier49 ::= [ a-zA-Z_ ] { [ a-zA-Z0-9_ ] }
cell_identifier ::= identifier
checker_identifier ::= identifier
class_identifier ::= identifier
class_variable_identifier ::= variable_identifier
clocking_identifier ::= identifier
config_identifier ::= identifier
const_identifier ::= identifier
constraint_identifier ::= identifier
covergroup_identifier ::= identifier
covergroup_variable_identifier ::= variable_identifier
cover_point_identifier ::= identifier
cross_identifier ::= identifier
dynamic_array_variable_identifier ::= variable_identifier
enum_identifier ::= identifier
escaped_identifier ::= \ {any_printable_ASCII_character_except_white_space} white_space
formal_identifier ::= identifier
formal_port_identifier ::= identifier
function_identifier ::= identifier
generate_block_identifier ::= identifier
genvar_identifier ::= identifier
hierarchical_array_identifier ::= hierarchical_identifier
```

Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1178
Copyright © 2018 IEEE. All rights reserved.
```ebnf
hierarchical_block_identifier ::= hierarchical_identifier
hierarchical_event_identifier ::= hierarchical_identifier
hierarchical_identifier ::= [ $root . ] { identifier constant_bit_select . } identifier
hierarchical_net_identifier ::= hierarchical_identifier
hierarchical_parameter_identifier ::= hierarchical_identifier
hierarchical_property_identifier ::= hierarchical_identifier
hierarchical_sequence_identifier ::= hierarchical_identifier
hierarchical_task_identifier ::= hierarchical_identifier
hierarchical_tf_identifier ::= hierarchical_identifier
hierarchical_variable_identifier ::= hierarchical_identifier
identifier ::=
```

simple_identifier
| escaped_identifier
```ebnf
index_variable_identifier ::= identifier
interface_identifier ::= identifier
interface_instance_identifier ::= identifier
inout_port_identifier ::= identifier
input_port_identifier ::= identifier
instance_identifier ::= identifier
library_identifier ::= identifier
member_identifier ::= identifier
method_identifier ::= identifier
modport_identifier ::= identifier
module_identifier ::= identifier
net_identifier ::= identifier
net_type_identifier ::= identifier
output_port_identifier ::= identifier
package_identifier ::= identifier
package_scope ::=
```

package_identifier ::
| $unit ::
```ebnf
parameter_identifier ::= identifier
port_identifier ::= identifier
production_identifier ::= identifier
program_identifier ::= identifier
property_identifier ::= identifier
ps_class_identifier ::= [ package_scope ] class_identifier
ps_covergroup_identifier ::= [ package_scope ] covergroup_identifier
ps_checker_identifier ::= [ package_scope ] checker_identifier
ps_identifier ::= [ package_scope ] identifier
ps_or_hierarchical_array_identifier ::=
```

[ implicit_class_handle . | class_scope | package_scope ] hierarchical_array_identifier
```ebnf
ps_or_hierarchical_net_identifier ::= [ package_scope ] net_identifier | hierarchical_net_identifier
ps_or_hierarchical_property_identifier ::=
```

Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1179
Copyright © 2018 IEEE. All rights reserved.
[ package_scope ] property_identifier
| hierarchical_property_identifier
```ebnf
ps_or_hierarchical_sequence_identifier ::=
```

[ package_scope ] sequence_identifier
| hierarchical_sequence_identifier
```ebnf
ps_or_hierarchical_tf_identifier ::=
```

[ package_scope ] tf_identifier
| hierarchical_tf_identifier
```ebnf
ps_parameter_identifier ::=
```

[ package_scope | class_scope ] parameter_identifier
| { generate_block_identifier [ [ constant_expression ] ] . } parameter_identifier
```ebnf
ps_type_identifier ::= [ local ::43 | package_scope | class_scope ] type_identifier
sequence_identifier ::= identifier
signal_identifier ::= identifier
simple_identifier49 ::= [ a-zA-Z_ ] { [ a-zA-Z0-9_$ ] }
specparam_identifier ::= identifier
system_tf_identifier50 ::= $[ a-zA-Z0-9_$ ]{ [ a-zA-Z0-9_$ ] }
task_identifier ::= identifier
tf_identifier ::= identifier
terminal_identifier ::= identifier
topmodule_identifier ::= identifier
type_identifier ::= identifier
udp_identifier ::= identifier
variable_identifier ::= identifier
```

A.9.4 White space
```ebnf
white_space ::= space | tab | newline | eof
```

A.10 Footnotes (normative)
1)
A package_import_declaration in a module_ansi_header, interface_ansi_header, or program_ansi_header shall be
followed by a parameter_port_list or list_of_port_declarations, or both.
2)
The list_of_port_declarations syntax is explained in 23.2.2.2, which also imposes various semantic restrictions,
e.g., a ref port shall be of a variable type and an inout port shall not be. It shall be illegal to initialize a port that
is not a variable output port or to specify a default value for a port that is not an input port.
3)
A
timeunits_declaration
shall
be
legal
as
a
non_port_module_item,
non_port_interface_item,
non_port_program_item, or package_item only if it repeats and matches a previous timeunits_declaration within
the same time scope.
4)
If the bind_target_scope is an interface_identifier or the bind_target_instance is an interface_instance_identifier,
then the bind_instantiation shall be an interface_instantiation or a checker_instantiation.
5)
It shall be illegal for a program_generate_item to include any item that would be illegal in a program_declaration
outside a program_generate_item.
6)
It shall be illegal for a checker_generate_item to include any item that would be illegal in a checker_declaration
outside a checker_generate_item.
7)
In a parameter_declaration that is a class_item, the parameter keyword shall be a synonym for the
localparam keyword.
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1180
Copyright © 2018 IEEE. All rights reserved.
8)
In any one declaration, only one of protected or local is allowed, only one of rand or randc is allowed,
and static and/or virtual can appear only once.
9)
The open_range_list in a uniqueness_constraint shall contain only expressions that denote scalar or array variables,
as described in 18.5.5.
10) In a data_declaration that is not within a procedural context, it shall be illegal to use the automatic keyword. In
a data_declaration, it shall be illegal to omit the explicit data_type before a list_of_variable_decl_assignments
unless the var keyword is used.
11) It shall be illegal to have an import statement directly within a class scope.
12) A charge strength shall only be used with the trireg keyword. When the vectored or scalared keyword is
used, there shall be at least one packed dimension.
13) When a packed dimension is used with the struct or union keyword, the packed keyword shall also be used.
14) When a type_reference is used in a net declaration, it shall be preceded by a net type keyword; and when it is used
in a variable declaration, it shall be preceded by the var keyword.
15) A type_identifier shall be legal as an enum_base_type if it denotes an integer_atom_type, with which an additional
packed dimension is not permitted, or an integer_vector_type.
16) It shall be legal to declare a void struct_union_member only within tagged unions. It shall be legal to declare a
random_qualifier only within unpacked structures.
17) An expression that is used as the argument in a type_reference shall not contain any hierarchical references or
references to elements of dynamic objects.
18) It shall be legal to omit the constant_param_expression from a param_assignment or the data_type from a
type_assignment only within a parameter_port_list. However, it shall not be legal to omit them from localparam
declarations in a parameter_port_list.
19) In a shallow copy, the expression shall evaluate to an object handle.
20) In packed_dimension, unsized_dimension is permitted only as the sole packed dimension in a DPI import
declaration; see dpi_function_proto and dpi_task_proto.
21) dpi_function_proto return types are restricted to small values, per 35.5.5.
22) Formals of dpi_function_proto and dpi_task_proto cannot use pass-by-reference mode, and class types cannot be
passed at all; see 35.5.6 for a description of allowed types for DPI formal arguments.
23) In a tf_port_item, it shall be illegal to omit the explicit port_identifier except within a function_prototype or
task_prototype.
24) The matches operator shall have higher precedence than the && and || operators.
25) It shall be legal to use the $ primary in an open_value_range or covergroup_value_range of the form [ expression
: $ ] or [ $ : expression ].
26) The result of this expression shall be assignment compatible with an integral type.
27) This expression is restricted as described in 19.5.1.2.
28) This expression is restricted as described in 19.5.
29) The .* token shall appear at most once in a list of port connections.
30) Within an interface_declaration, it shall only be legal for a generate_item to be an interface_or_generate_item.
Within a module_declaration, except when also within an interface_declaration, it shall only be legal for a
generate_item to be a module_or_generate_item. Within a checker_declaration, it shall only be legal for a
generate_item to be a checker_or_generate_item.
31) Parentheses are required when an event expression that contains comma-separated event expressions is passed as an
actual argument using positional binding.
32) In a constant_assignment_pattern_expression, all member expressions shall be constant expressions.
33) Embedded spaces are illegal.
34) In a multiple_concatenation, it shall be illegal for the multiplier not to be a constant_expression unless the type of
the concatenation is string.
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1181
Copyright © 2018 IEEE. All rights reserved.
35) { } shall denote an empty unpacked array concatenation, as described in 10.10, and shall not be used in any other
form of concatenation.
36) In a constant_function_call, all arguments shall be constant_expressions.
37) It shall be illegal to omit the parentheses in a tf_call unless the subroutine is a task, void function, or class method.
If the subroutine is a nonvoid class function method, it shall be illegal to omit the parentheses if the call is directly
recursive.
38) In a randomize_call that is not a method call of an object of class type (i.e., a scope randomize), the optional
parenthesized identifier_list after the keyword with shall be illegal, and the use of null shall be illegal.
39) A genvar_identifier shall be legal in a constant_primary only within a genvar_expression.
40) It shall be legal to use a type_reference constant_primary as the casting_type in a static cast. It shall be illegal for a
type_reference constant_primary to be used with any operators except the equality/inequality and case equality/
inequality operators.
41) implicit_class_handle shall only appear within the scope of a class_declaration or out-of-block method declaration.
42) The $ primary shall be legal only in a select for a queue variable, in an open_value_range,  covergroup_val-
ue_range, integer_covergroup_expression, or as an entire sequence_actual_arg or property_actual_arg.
43) The local:: qualifier shall only appear within the scope of an inline constraint block.
44) The unsigned number or fixed-point number in time_literal shall not be followed by white_space.
45) In a constant_let_expression, all arguments shall be constant_expressions and its right-hand side shall be a
constant_expression itself provided that its formal arguments are treated as constant_primary there.
46) In a variable_lvalue that is assigned within a sequence_match_item any select shall also be a constant_select.
47) A
streaming_concatenation
expression
shall
not
be
nested
within
another
variable_lvalue.
A
streaming_concatenation shall not be the target of the increment or decrement operator nor the target of any
assignment operator except the simple ( = ) or nonblocking assignment ( <= ) operator.
48) The apostrophe ( ' ) in unbased_unsized_literal shall not be followed by white_space.
49) A simple_identifier or c_identifier shall start with an alpha or underscore ( _ ) character, shall have at least one
character, and shall not have any spaces.
50) The $ character in a system_tf_identifier shall not be followed by white_space. A system_tf_identifier shall not be
escaped.
End of file.
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1182
Copyright © 2018 IEEE. All rights reserved.
Annex B
(normative)
Keywords
SystemVerilog reserves the keywords listed in Table B.1.
Table B.1—Reserved keywords
accept_on
alias
always
always_comb
always_ff
always_latch
and
assert
assign
assume
automatic
before
begin
bind
bins
binsof
bit
break
buf
bufif0
bufif1
byte
case
casex
casez
cell
chandle
checker
class
clocking
cmos
config
const
constraint
context
continue
cover
covergroup
coverpoint
cross
deassign
default
defparam
design
disable
dist
do
edge
else
end
endcase
endchecker
endclass
endclocking
endconfig
endfunction
endgenerate
endgroup
endinterface
endmodule
endpackage
endprimitive
endprogram
endproperty
endspecify
endsequence
endtable
endtask
enum
event
eventually
expect
export
extends
extern
final
first_match
for
force
foreach
forever
fork
forkjoin
function
generate
genvar
global
highz0
highz1
if
iff
ifnone
ignore_bins
illegal_bins
implements
implies
import
incdir
include
initial
inout
input
inside
instance
int
integer
interconnect
interface
intersect
join
join_any
join_none
large
let
liblist
library
local
localparam
logic
longint
macromodule
matches
medium
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1183
Copyright © 2018 IEEE. All rights reserved.
Table B.1—Reserved keywords  (continued)
modport
module
nand
negedge
nettype
new
nexttime
nmos
nor
noshowcancelled
not
notif0
notif1
null
or
output
package
packed
parameter
pmos
posedge
primitive
priority
program
property
protected
pull0
pull1
pulldown
pullup
pulsestyle_ondetect
pulsestyle_onevent
pure
rand
randc
randcase
randsequence
rcmos
real
realtime
ref
reg
reject_on
release
repeat
restrict
return
rnmos
rpmos
rtran
rtranif0
rtranif1
s_always
s_eventually
s_nexttime
s_until
s_until_with
scalared
sequence
shortint
shortreal
showcancelled
signed
small
soft
solve
specify
specparam
static
string
strong
strong0
strong1
struct
super
supply0
supply1
sync_accept_on
sync_reject_on
table
tagged
task
this
throughout
time
timeprecision
timeunit
tran
tranif0
tranif1
tri
tri0
tri1
triand
trior
trireg
type
typedef
union
unique
unique0
unsigned
until
until_with
untyped
use
uwire
var
vectored
virtual
void
wait
wait_order
wand
weak
weak0
weak1
while
wildcard
wire
with
within
wor
xnor
xor
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1184
Copyright © 2018 IEEE. All rights reserved.
Annex C
(normative)
Deprecation
C.1 General
This annex identifies constructs that either
—
have been deprecated from SystemVerilog and no longer appear in this standard, or
—
are under consideration for deprecation and might be removed from future versions of this standard.
C.2 Constructs that have been deprecated
C.2.1 PLI TF and ACC routine libraries
IEEE Std 1364-2005 deprecated the Programming Language Interface (PLI) libraries containing the task/
function (TF) and access (ACC) routines that were contained in previous versions of that standard. These
routines were described in Clause 21 through Clause 25, Annex E, and Annex F of IEEE Std 1364-2001.
The text of these deprecated clauses and annexes do not appear in this version of the standard. The text can
be found in IEEE Std 1364-2001.
C.2.2 $sampled with a clocking event argument
IEEE Std 1800-2005 17.7.3 required that an explicit or inferred clocking event argument be provided for the
$sampled assertion system function. In this version of the standard, the semantics of $sampled have been
changed to a form that does not depend on a clocking event. Therefore the syntax for defining the clocking
event argument to $sampled is deprecated and does not appear in this version of the standard.
C.2.3 ended sequence method
IEEE Std 1800-2005 17.7.3 required using the sequence method ended in sequence expressions and the
sequence method triggered in other contexts. Since these two constructs have the same meaning but
mutually exclusive usage contexts, in this version of the standard, the triggered method is allowed to be
used in sequence expressions, and the usage of ended is deprecated and does not appear in this version of
the standard.
C.2.4 vpi_free_object()
The semantics of this VPI routine have been clarified to account for the nature of dynamic data in the
SystemVerilog information model and the concept of handle validity. It has been renamed
vpi_release_handle(), and thus vpi_free_object() has been deprecated.
C.2.5 Data read API
IEEE Std 1800-2009 deprecated the Data Read API that was contained in the previous version of the
standard. These routines were described in Clause 30 and Annex I of IEEE Std 1800-2005. The text of these
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
