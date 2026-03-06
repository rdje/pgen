---
title: "Section Annex.C: IEEE Standard for VHDL Language Reference Manual"
document: "VHDL Language Reference Manual"
standard: "IEEE 1076-2019"
domain: "VHDL"
section: "Annex.C"
source_txt: "section-Annex_C-informative-syntax-summary.txt"
source_pdf: "/Users/richarddje/Documents/github/VHDL-LRM-IEEE-1076-2019.pdf"
---

# Section Annex.C: IEEE Standard for VHDL Language Reference Manual

IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
530
Copyright © 2019 IEEE. All rights reserved.
Annex C
(informative)
Syntax summary
This annex provides a summary of the syntax for VHDL. Productions are ordered alphabetically by
left-hand nonterminal name. The number listed to the right indicates the clause or subclause where the
production is given.
```ebnf
absolute_pathname ::= . partial_pathname
```

[§8.7]
```ebnf
abstract_literal ::= decimal_literal | based_literal
```

[§ 15.5.1]
```ebnf
access_incomplete_type_definition ::=
```

[§ 5.8.1]
   access access_incomplete_subtype_indication
```ebnf
access_type_definition ::= access subtype_indication
 [§ 5.4.1]
actual_designator ::=
 [§ 6.5.7.1]
       [ inertial ] conditional_expression
   |   signal_name
   |   variable_name
   |   file_name
   |   subtype_indication
| subprogram_name
    |   instantiated_package_name
| open
actual_part ::=
```

[§ 6.5.7.1]
       actual_designator
 |   function_name ( actual_designator )
   |   type_mark ( actual_designator )
```ebnf
adding_operator ::= + | – | &
```

[§ 9.2]
```ebnf
aggregate ::=
```

[§ 9.3.3.1]
    ( element_association { , element_association } )
```ebnf
alias_declaration ::=
  [§ 6.6.1]
   alias alias_designator [ : subtype_indication subtype_indication ] is name [ signature ] ;
alias_designator ::= identifier | character_literal | operator_symbol
```

[§ 6.6.1]
```ebnf
allocator ::=
 [§ 9.3.7]
       new subtype_indication
   |   new qualified_expression
```

Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
531
Copyright © 2019 IEEE. All rights reserved.
```ebnf
architecture_body ::=
```

[§ 3.3.1]
   architecture identifier of entity_name is
       architecture_declarative_part
   begin
       architecture_statement_part
   end [ architecture ] [ architecture_simple_name ] ;
```ebnf
architecture_declarative_part ::=
```

[§ 3.3.2]
    { block_declarative_item }
```ebnf
architecture_statement_part ::=
 [§ 3.3.3]
    { concurrent_statement  }
array_constraint ::=
```

[§ 5.3.2.1]
       index_constraint [ array_element_constraint ]
   |   ( open ) [ array_element_constraint ]
```ebnf
array_element_constraint ::= element_constraint
```

[§ 5.3.2.1]
```ebnf
array_element_resolution ::= resolution_indication
 [§ 6.3]
array_incomplete_type_definition ::=
 [§ 5.8.1]
   array ( array_index_incomplete_type_list )
 of element_incomplete_subtype_indication
array_index_incomplete_type ::=
 [§ 5.8.1]
        index_subtype_definition
  |    index_constraint
   |    unspecified_type_indication
array_index_incomplete_type_list ::=
  [§ 5.8.1]
    array_index_incomplete_type { , array_index_incomplete_type }
array_mode_view_indication ::=
 [§ 6.5.2]
 view ( mode_view_name )  [ of unresolved_array_subtype_indication ]
array_type_definition ::=
   [§ 5.3.2.1]
    unbounded_array_definition | constrained_array_definition
assertion ::=
```

[§ 10.3]
   assert condition
 [ report expression ]
 [ severity expression ]
```ebnf
assertion_statement ::= [ label : ] assertion ;
```

[§ 10.3]
```ebnf
association_element ::=
   [§ 6.5.7.1]
   [ formal_part => ] actual_part
association_list ::=
  [§ 6.5.7.1]
    association_element { , association_element }
attribute_declaration ::=
```

[§ 6.7]
   attribute identifier : type_mark ;
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
532
Copyright © 2019 IEEE. All rights reserved.
```ebnf
attribute_designator ::= attribute_simple_name
```

[§ 8.7]
```ebnf
attribute_name ::=
```

[§ 8.7]
   prefix [ signature ] ' attribute_designator [ ( expression ) ]
```ebnf
attribute_specification ::=
```

[§ 7.2]
   attribute attribute_designator of entity_specification is conditional_expression ;
```ebnf
base ::= integer
```

[§ 15.5.3]
```ebnf
base_specifier ::= B | O | X | UB | UO | UX | SB | SO | SX | D
```

[§ 15.8]
```ebnf
based_integer ::=
```

[§ 15.5.3]
    extended_digit { [ underline ] extended_digit }
```ebnf
based_literal ::=
```

[§ 15.5.3]
    base # based_integer [ . based_integer ] # [ exponent ]
```ebnf
basic_character ::=
```

[§ 15.2]
    basic_graphic_character | format_effector
```ebnf
basic_graphic_character ::=
```

[§ 15.2]
    upper_case_letter | digit | special_character| space_character
```ebnf
basic_identifier ::= letter { [ underline ] letter_or_digit }
```

[§ 15.4.2]
```ebnf
binding_indication ::=
   [§ 7.3.2.1]
   [ use entity_aspect ]
    [ generic_map_aspect ]
    [ port_map_aspect ]
bit_string_literal ::= [ integer ] base_specifier " [ bit_value ] "
```

[§ 15.8]
```ebnf
bit_value ::= graphic_character { [ underline ] graphic_character }
```

[§ 15.8]
```ebnf
block_configuration ::=
```

[§ 3.4.2]
   for block_specification
 { use_clause }
        { configuration_item }
   end for ;
```ebnf
block_declarative_item ::=
```

[§ 3.3.2]
       subprogram_declaration
    |   subprogram_body
    |   subprogram_instantiation_declaration
    |   package_declaration
    |   package_body
    |   package_instantiation_declaration
   |   type_declaration
   |   subtype_declaration
   |   mode_view_declaration
   |   constant_declaration
   |   signal_declaration
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
533
Copyright © 2019 IEEE. All rights reserved.
   |   shared_variable_declaration
   |   file_declaration
   |   alias_declaration
    |   component_declaration
   |   attribute_declaration
   |   attribute_specification
    |   configuration_specification
   |   disconnection_specification
   |   use_clause
    |   group_template_declaration
   |   group_declaration
   |   PSL_Property_Declaration
   |   PSL_Sequence_Declaration
   |   PSL_Clock_Declaration
```ebnf
block_declarative_part ::=
```

[§ 11.2]
    { block_declarative_item }
```ebnf
block_header ::=
```

[§ 11.2]
    [ generic_clause
    [ generic_map_aspect ; ] ]
    [ port_clause
 [ port_map_aspect ; ] ]
```ebnf
block_specification ::=
```

[§ 3.4.2]
       architecture_name
   |   block_statement_label
   |   generate_statement_label [ ( generate_specification ) ]
```ebnf
block_statement ::=
```

[§ 11.2]
   block_label :
 block [ ( guard_condition ) ] [ is ]
  block_header
     block_declarative_part
 begin
  block_statement_part
 end block [ block_label ] ;
```ebnf
block_statement_part ::=
```

[§ 11.2]
    { block_declarative_item }concurrent_statement
```ebnf
case_generate_alternative ::=
  [§ 11.8]
   when [ alternative_label : ] choices =>
 generate_statement_body
case_generate_statement ::=
```

[§ 11.8]
   generate_label :
 case expression generate
  case_generate_alternative
     { case_generate_alternative }
 end generate [ generate_label ] ;
```ebnf
case_statement ::=
  [§ 10.9]
   [ case_label : ]
```

Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
534
Copyright © 2019 IEEE. All rights reserved.
 case [ ? ] expression is
  case_statement_alternative
     { case_statement_alternative }
 end case [ ? ] [ case_label ] ;
```ebnf
case_statement_alternative ::=
```

[§ 10.9]
   when choices =>
sequence_of_statements
```ebnf
character_literal ::= ' graphic_character '
```

[§ 15.6]
```ebnf
choice ::=
```

[§ 9.3.3.1]
       simple_expression
   |   discrete_range
   |   element_simple_name
   |   others
```ebnf
choices ::= choice { | choice }
```

[§ 9.3.3.1]
```ebnf
component_configuration ::=
```

[§ 3.4.3]
   for component_specification
        [ binding_indication ; ]
        { verification_unit_binding_indication ; }
 [ block_configuration ]
   end for ;
```ebnf
component_declaration ::=
```

[§ 6.8]
   component identifier [ is ]
 [ local_generic_clause ]
       [ local_port_clause ]
   end [ component ] [ component_simple_name ] ;
```ebnf
component_instantiation_statement ::=
```

[§ 11.8.1]
   instantiation_label :
 instantiated_unit
  [ generic_map_aspect ]
  [ port_map_aspect ] ;
```ebnf
component_specification ::=
```

[§ 7.3.1]
    instantiation_list : component_name
```ebnf
composite_type_definition ::=
 [§ 5.3.1]
       array_type_definition
    |   record_type_definition
compound_configuration_specification ::=
```

[§ 7.3.1]
   for component_specification binding_indication ;
        verification_unit_binding_indication ;
        { verification_unit_binding_indication ; }
   end for ;
```ebnf
concurrent_assertion_statement ::=
  [§ 11.5]
    [ label : ] [ postponed ] assertion ;
```

Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
535
Copyright © 2019 IEEE. All rights reserved.
```ebnf
concurrent_conditional_signal_assignment ::=
 [§ 11.7]
 target  <=  [ guarded ] [ delay_mechanism ] conditional_waveforms ;
concurrent_procedure_call_statement ::=
 [§ 11.4]
    [ label : ] [ postponed ] procedure_call ;
concurrent_selected_signal_assignment ::=
```

[§ 11.7]
   with expression select [ ? ]
 target <= [ guarded ] [ delay_mechanism ] selected_waveforms ;
```ebnf
concurrent_signal_assignment_statement ::=
  [§ 11.7]
       [ label : ] [ postponed ] concurrent_simple_signal_assignment
   |   [ label : ] [ postponed ] concurrent_conditional_signal_assignment
   |   [ label : ] [ postponed ] concurrent_selected_signal_assignment
concurrent_simple_signal_assignment ::=
  [§ 11.7]
    target <= [ guarded ] [ delay_mechanism ] waveform ;
concurrent_statement ::=
  [§ 11.1]
```

block_statement
|    process_statement
|   concurrent_procedure_call_statement
|   concurrent_assertion_statement
|   concurrent_signal_assignment_statement
|   component_instantiation_statement
|   generate_statement
 |   PSL_PSL_Directive
```ebnf
condition ::= expression
 [§ 9.1]
condition_clause ::= until condition
```

[§ 10.2]
```ebnf
condition_operator ::= ??
 [§ 9.2.1]
conditional_expression ::=
   [§ 9.1]
   expression { when condition else expression }
conditional_or_unaffected_expression ::=
 [§ 9.1]
    expression_or_unaffected { when condition else expression_or_unaffected } [ when condition ]
conditional_signal_assignment ::=
  [§ 10.5.3]
    target  <=  [ delay_mechanism ] conditional_waveforms ;
conditional_waveforms ::=
  [§ 10.5.3]
    waveform when condition
 { else waveform when condition }
 [ else waveform ]
configuration_declaration ::=
```

[§ 3.4.1]
   configuration identifier of entity_name is
        configuration_declarative_part
        { verification_unit_binding_indication ; }
 block_configuration
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
536
Copyright © 2019 IEEE. All rights reserved.
   end [ configuration ] [ configuration_simple_name ] ;
```ebnf
configuration_declarative_item ::=
```

[§ 3.4.1]
       use_clause
   |   attribute_specification
   |   group_declaration
```ebnf
configuration_declarative_part ::=
```

[§ 3.4.1]
    { configuration_declarative_item }
```ebnf
configuration_item ::=
```

[§ 3.4.2]
       block_configuration
    |  component_configuration
```ebnf
configuration_specification ::=
  [§ 7.3.1]
        simple_configuration_specification
    |   compound_configuration_specification
constant_declaration ::=
```

[§ 6.4.2.2]
   constant identifier_list : subtype_indication [ := conditional_expression ] ;
```ebnf
constrained_array_definition ::=
```

[§ 5.3.2.1]
   array index_constraint of element_subtype_indication
```ebnf
constraint ::=
 [§ 6.3]
       range_constraint
   |   array_constraint
   |   record_constraint
context_clause ::= { context_item }
```

[§ 13.4]
```ebnf
context_declaration ::=
```

[§ 13.3]
   context identifier is
    context_clause
 end [ context ] [ context_simple_name ] ;
```ebnf
context_item ::=
```

[§ 13.4]
       library_clause
   |   use_clause
   |   context_reference
```ebnf
context_reference ::=
```

[§ 13.4]
   context selected_name { , selected_name } ;
```ebnf
decimal_literal ::= integer [ . integer ] [ exponent ]
  [§ 15.5.2]
delay_mechanism ::=
 [§ 10.5.2.1]
       transport
   |   [ reject time_expression ] inertial
design_file ::= design_unit { design_unit }
```

[§ 13.1]
```ebnf
design_unit ::= context_clause library_unit
```

[§ 13.1]
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
537
Copyright © 2019 IEEE. All rights reserved.
```ebnf
designator ::= identifier | operator_symbol
```

[§ 4.2.1]
```ebnf
direction ::= to | downto
```

[§ 5.2.1]
```ebnf
disconnection_specification ::=
```

[§ 7.4]
   disconnect guarded_signal_specification after time_expression ;
```ebnf
discrete_range ::= discrete_subtype_indication | range
  [§ 5.3.2.1]
discrete_incomplete_type_definition ::= ( <> )
 [§ 5.8.1]
element_array_mode_view_indication ::=
  [§ 6.5.2]
   view ( mode_view_name )
element_association ::=
   [§ 9.3.3.1]
   [ choices => ] expression
element_constraint ::=
```

[§ 6.3]
       array_constraint
   |   record_constraint
```ebnf
element_declaration ::=
 [§ 5.3.3]
    identifier_list : element_subtype_definition ;
element_mode_indication ::=
  [§ 6.5.2]
       mode
    |   element_mode_view_indication
element_mode_view_indication ::=
   [§ 6.5.2]
        element_record_mode_view_indication
      | element_array_mode_view_indication
element_record_mode_view_indication ::=
 [§ 6.5.2]
 view mode_view_name
element_resolution ::= array_element_resolution | record_resolution
 [§ 6.3]
element_subtype_definition ::= subtype_indication
   [§ 5.3.3]
entity_aspect ::=
```

[§ 7.3.2.2]
       entity entity_name [ ( architecture_identifier ) ]
   |   configuration configuration_name
   |   open
```ebnf
entity_class ::=
```

[§ 7.2]
       entity
   |   architecture
   |   configuration
   |   procedure
   |   function
   |   package
   |   type
   |   subtype
   |   constant
   |   signal
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
538
Copyright © 2019 IEEE. All rights reserved.
   |   variable
   |   component
   |   label
   |   literal
   |   units
   |   group
   |   file
   |   property
   |   sequence
   |   view
```ebnf
entity_class_entry ::= entity_class [ <> ]
```

[§ 6.9]
```ebnf
entity_class_entry_list ::=
```

[§ 6.9]
    entity_class_entry { , entity_class_entry }
```ebnf
entity_declaration ::=
```

[§ 3.2.1]
   entity identifier is
 entity_header
       entity_declarative_part
   [ begin
 entity_statement_part ]
   end [ entity ] [ entity_simple_name ] ;
```ebnf
entity_declarative_item ::=
```

[§ 3.2.3]
       subprogram_declaration
    |   subprogram_body
    |   subprogram_instantiation_declaration
    |   package_declaration
    |   package_body
    |   package_instantiation_declaration
   |   type_declaration
   |   subtype_declaration
   |   mode_view_declaration
   |   constant_declaration
   |   signal_declaration
   |   shared_variable_declaration
   |   file_declaration
   |   alias_declaration
   |   attribute_declaration
   |   attribute_specification
   |   disconnection_specification
   |   use_clause
    |   group_template_declaration
   |   group_declaration
   |   PSL_Property_Declaration
   |   PSL_Sequence_Declaration
   |   PSL_Clock_Declaration
```ebnf
entity_declarative_part ::=
```

[§ 3.2.3]
    { entity_declarative_item }
```ebnf
entity_designator ::= entity_tag [ signature ]
```

[§ 7.2]
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
539
Copyright © 2019 IEEE. All rights reserved.
```ebnf
entity_header ::=
```

[§ 3.2.2]
   [ formal_generic_clause ]
   [ formal_port_clause ]
```ebnf
entity_name_list ::=
```

[§ 7.2]
        entity_designator { , entity_designator }
   |   others
   |   all
```ebnf
entity_specification ::=
```

[§ 7.2]
    entity_name_list : entity_class
```ebnf
entity_statement ::=
```

[§ 3.2.4]
       concurrent_assertion_statement
   |   passive_concurrent_procedure_call_statement
   |   passive_process_statement
   |   PSL_PSL_Directive
```ebnf
entity_statement_part ::=
```

[§ 3.2.4]
    { entity_statement }
```ebnf
entity_tag ::= simple_name | character_literal | operator_symbol
```

[§ 7.2]
```ebnf
enumeration_literal ::= identifier | character_literal
```

[§ 5.2.2.1]
```ebnf
enumeration_type_definition ::=
```

[§ 5.2.2.1]
    ( enumeration_literal { , enumeration_literal } )
```ebnf
exit_statement ::=
```

[§ 10.12]
    [ label : ] exit [ loop_label ] [ when condition ] ;
```ebnf
exponent ::= E [ + ] integer | E – integer
```

[§ 15.5.2]
```ebnf
expression ::=
 [§ 9.1]
        condition_operator primary
   |   logical_expression
expression_or_unaffected ::=
 [§ 9.1]
    expression | unaffected
extended_digit ::= digit | letter
  [§ 15.5.3]
extended_identifier ::= \ graphic_character { graphic_character } \
```

[§ 15.4.3]
```ebnf
external_name ::=
```

[§  8.7]
        external_constant_name
    |   external_signal_name
    |   external_variable_name
```ebnf
external_constant_name ::=
```

[§  8.7]
   << constant external_pathname : interface_type_indication >>
```ebnf
external_signal_name ::=
```

[§  8.7]
   << signal external_pathname : interface_type_indication >>
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
540
Copyright © 2019 IEEE. All rights reserved.
```ebnf
external_variable_name ::=
```

[§  8.7]
   << variable external_pathname : interface_type_indication >>
```ebnf
external_pathname ::=
```

[§  8.7]
       package_pathname
    |   absolute_pathname
   |   relative_pathname
```ebnf
factor ::=
   [§ 9.1]
    unary_expression [ ** unary_expression ]
file_declaration ::=
```

[§ 6.4.2.5]
   file identifier_list : subtype_indication [ file_open_information ] ;
```ebnf
file_incomplete_type_definition ::=
 [§ 5.8.1]
   file of file_incomplete_type_mark
file_logical_name ::= string_expression
  [§ 6.4.2.5]
file_open_information ::=
```

[§ 6.4.2.5]
   [ open file_open_kind_expression ] is file_logical_name
```ebnf
file_type_definition ::=
```

[§ 5.5.1]
  file of type_mark
```ebnf
floating_incomplete_type_definition ::= range <> . <>
```

[§ 5.8.1]
```ebnf
floating_type_definition ::= range_constraint
 [§ 5.2.5.1]
for_generate_statement ::=
```

[§ 11.8]
   generate_label :
 for generate_parameter_specification generate
            generate_statement_body
 end generate [ generate_label ] ;
```ebnf
force_mode ::= in | out
```

[§ 10.5.2.1]
```ebnf
formal_designator ::=
 [§ 6.5.7.1]
       generic_name  [ signature ]
   |   port_name
   |   parameter_name
formal_parameter_list ::= parameter_interface_list
```

[§ 4.2.2.1]
```ebnf
formal_part ::=
 [§ 6.5.7.1]
       formal_designator
   |   function_name ( formal_designator )
   |   type_mark ( formal_designator )
full_type_declaration ::=
  [§ 6.2]
   type identifier is type_definition ;
function_call ::=
   [§ 9.3.4]
```

Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
541
Copyright © 2019 IEEE. All rights reserved.
   function_name [ generic_map_aspect] [ parameter_map_aspect  ]
```ebnf
function_specification ::=
```

[§ 4.2.1]
   [ pure | impure ] function designator
 subprogram_header
 [ [ parameter ] ( formal_parameter_list ) ] return [ return_identifier of ] type_mark
```ebnf
generate_specification ::=
```

[§ 3.4.2]
       static_discrete_range
   |   static_expression
   |   alternative_label
```ebnf
generate_statement ::=
```

[§ 11.8]
       for_generate_statement
   |    if_generate_statement
   |    case_generate_statement
```ebnf
generate_statement_body ::=
```

[§ 11.8]
    [ block_declarative_part
   begin ]
    { concurrent_statement }
 [ end [ alternative_label ] ; ]
```ebnf
generic_clause ::=
 [§ 6.5.6.2]
   generic ( generic_list ) ;
generic_list ::= generic_interface_list
```

[§ 6.5.6.2]
```ebnf
generic_map_aspect ::=
 [§ 6.5.7.2]
   generic map ( generic_association_list )
graphic_character ::=
```

[§ 15.2]
    basic_graphic_character | lower_case_letter | other_special_character
```ebnf
group_constituent ::= name | character_literal
```

[§ 6.10]
```ebnf
group_constituent_list ::= group_constituent { , group_constituent }
```

[§ 6.10]
```ebnf
group_declaration ::=
```

[§ 6.10]
   group identifier : group_template_name ( group_constituent_list ) ;
```ebnf
group_template_declaration ::=
```

[§ 6.9]
   group identifier is ( entity_class_entry_list ) ;
```ebnf
guarded_signal_specification ::=
```

[§ 7.4]
   guarded_signal_list : type_mark
```ebnf
identifier ::= basic_identifier | extended_identifier
```

[§ 15.4.1]
```ebnf
identifier_list ::= identifier { , identifier }
```

[§ 5.3.3]
```ebnf
if_generate_statement ::=
```

[§ 11.8]
   generate_label :
 if [ alternative_label : ] condition generate
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
542
Copyright © 2019 IEEE. All rights reserved.
            generate_statement_body
 { elsif [ alternative_label : ] condition generate
             generate_statement_body }
 [ else [ alternative_label : ] generate
            generate_statement_body ]
 end generate [ generate_label ] ;
```ebnf
if_statement ::=
 [§ 10.8]
   [ if_label : ]
 if condition then
     sequence_of_statements
 { elsif condition then
      sequence_of_statements}
  [ else
```

sequence_of_statements]
  end if [ if_label ] ;
```ebnf
incomplete_subtype_indication ::=
  [§5.8.1]
        subtype_indication
    |   unspecified_type_indication
incomplete_type_declaration ::= type identifier ;
  [§ 5.4.2]
incomplete_type_definition ::=
  [§5.8.1]
        private_incomplete_type_definition
    |   scalar_incomplete_type_definition
    |   discrete_incomplete_type_definition
    |   integer_incomplete_type_definition
    |   physical_incomplete_type_definition
    |   floating_incomplete_type_definition
    |   array_incomplete_type_definition
    |   access_incomplete_type_definition
    |   file_incomplete_type_definition
incomplete_type_mark ::=
  [§5.8.1]
       type_mark
    |   unspecified_type_indication
index_constraint ::= ( discrete_range { , discrete_range } )
 [§ 5.3.2.1]
index_subtype_definition ::= type_mark range <>
 [§ 5.3.2.1]
indexed_name ::= prefix ( expression { , expression } )
```

[§ 8.5]
```ebnf
instantiated_unit ::=
```

[§ 11.8.1]
       [ component ] component_name
   |   entity entity_name [ ( architecture_identifier ) ]
   |   configuration configuration_name
```ebnf
instantiation_list ::=
```

[§ 7.3.1]
       instantiation_label { , instantiation_label }
   |   others
   |   all
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
543
Copyright © 2019 IEEE. All rights reserved.
```ebnf
integer ::= digit { [ underline ] digit }
  [§ 15.5.2]
integer_incomplete_type_definition ::= range <>
  [§ 5.8.1]
integer_type_definition ::= range_constraint
 [§ 5.2.3.1]
interface_constant_declaration ::=
 [§ 6.4.2.2]
   [ constant ] identifier_list : [ in ] interface_type_indication
 [ := static_conditional_expression ]
interface_declaration ::=
  [§ 6.5.1]
       interface_object_declaration
    |   interface_type_declaration
    |   interface_subprogram_declaration
    |   interface_package_declaration
interface_element ::= interface_declaration
  [§ 6.5.6.1]
interface_file_declaration ::=
 [§ 6.5.2]
   file identifier_list : subtype_indication
interface_function_specification ::=
```

[§ 6.5.4]
   [ pure | impure ] function designator
 [ [ parameter ] ( formal_parameter_list ) ] return type_mark
[§ 6.5.3]
```ebnf
interface_list ::=
   [§ 6.5.6.1]
    interface_element { ; interface_element } [ ; ]
interface_object_declaration ::=
 [§ 6.5.2]
        interface_constant_declaration
    |   interface_signal_declaration
    |   interface_variable_declaration
    |   interface_file_declaration
interface_package_declaration ::=
```

[§ 6.5.5]
   package identifier is
 new uninstantiated_package_name interface_package_generic_map_aspect
```ebnf
interface_package_generic_map_aspect ::=
```

[§ 6.5.5]
       generic_map_aspect
   |   generic map ( <> )
   |   generic map ( default )
```ebnf
interface_procedure_specification ::=
```

[§ 6.5.4]
   procedure identifier
 [ [ parameter ] ( formal_parameter_list ) ]
```ebnf
interface_signal_declaration ::=
 [§ 6.5.2]
   [ signal ] identifier_list : signal_mode_indication
interface_subprogram_declaration ::=
 [§ 6.5.4]
    interface_subprogram_specification [ is interface_subprogram_default ]
```

Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
544
Copyright © 2019 IEEE. All rights reserved.
```ebnf
interface_subprogram_default ::= subprogram_name | <>
```

[§ 6.5.4]
```ebnf
interface_subprogram_specification ::=
```

[§ 6.5.4]
    interface_procedure_specification | interface_function_specification
```ebnf
interface_type_declaration ::=
 [§ 6.5.3]
   type identifier [is incomplete_type_definition]
interface_type_indication ::=
 [§ 6.5.2]
    subtype_indication | unspecified_type_indication
interface_variable_declaration ::=
   [§6.5.2]
   [ variable ] identifier_list : [ mode ]  interface_type_indication
 [ := static_conditional_expression ]
iteration_scheme ::=
```

[§ 10.10]
      while condition
   |  for loop_parameter_specification
```ebnf
label ::= identifier
```

[§ 11.8]
```ebnf
letter ::= upper_case_letter | lower_case_letter
```

[§ 15.4.2]
```ebnf
letter_or_digit ::= letter | digit
```

[§ 15.4.2]
```ebnf
library_clause ::= library logical_name_list ;
```

[§ 13.2]
```ebnf
library_unit ::=
```

[§ 13.1]
       primary_unit
    |   secondary_unit
```ebnf
literal ::=
```

[§ 9.3.2]
       numeric_literal
   |   enumeration_literal
    |   string_literal
    |   bit_string_literal
   |   null
```ebnf
logical_expression ::=
```

[§ 9.1]
       relation { and relation }
   |   relation { or relation }
   |   relation { xor relation }
   |   relation [ nand relation ]
   |   relation [ nor relation ]
   |   relation { xnor relation }
```ebnf
logical_name ::= identifier
  [§ 13.2]
logical_name_list ::= logical_name { , logical_name }
```

[§ 13.2]
```ebnf
logical_operator ::= and | or | nand | nor | xor | xnor
```

[§ 9.2.1]
```ebnf
loop_statement ::=
```

[§ 10.10]
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
545
Copyright © 2019 IEEE. All rights reserved.
   [ loop_label : ]
 [ iteration_scheme ] loop
     sequence_of_statements
 end loop [ loop_label ] ;
```ebnf
miscellaneous_operator ::= ** | abs | not
```

[§ 9.2.1]
```ebnf
mode ::= in | out | inout | buffer | linkage
```

[§ 6.5.2]
```ebnf
mode_indication ::=
 [§ 6.5.2]
       simple_mode_indication
    |   record_mode_view_indication
mode_view_declaration ::=
  [§ 6.5.2]
   view identifier of unresolved_record_subtype_indication is
        { mode_view_element_definition }
  end view [ mode_view_simple_name ] ;
mode_view_element_definition ::=
 [§ 6.5.2]
    record_element_list : element_mode_indication ;
mode_view_indication ::=
 [§ 6.5.2]
```

record_mode_view_indication
|   array_mode_view_indication
```ebnf
multiplying_operator ::= * | / | mod | rem
   [§ 9.2.1]
name ::=
```

[ 8.1]
       simple_name
    |   operator_symbol
   |   character_literal
    |   selected_name
   |   indexed_name
   |   slice_name
   |   attribute_name
   |   external_name
```ebnf
next_statement ::=
   [§ 10.11]
    [ label : ] next [ loop_label ] [ when condition ] ;
null_statement ::= [ label : ] null ;
```

[§ 10.14]
```ebnf
numeric_literal ::=
```

[§ 9.3.2]
       abstract_literal
   |   physical_literal
```ebnf
object_declaration ::=
```

[§ 6.4.2.1]
       constant_declaration
   |   signal_declaration
   |   variable_declaration
   |   file_declaration
```ebnf
operator_symbol ::= string_literal
```

[§ 4.2.1]
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
546
Copyright © 2019 IEEE. All rights reserved.
```ebnf
package_body ::=
 [§ 4.8]
   package body package_simple_name is
        package_body_declarative_part
   end [ package body ] [ package_simple_name ] ;
package_body_declarative_item ::=
```

[§ 4.8]
       subprogram_declaration
    |   subprogram_body
    |   subprogram_instantiation_declaration
    |   package_declaration
    |   package_body
    |   package_instantiation_declaration
   |   type_declaration
   |   subtype_declaration
   |   constant_declaration
   |   variable_declaration
   |   file_declaration
   |   alias_declaration
   |   attribute_declaration
   |   attribute_specification
   |   use_clause
    |   group_template_declaration
   |   group_declaration
```ebnf
package_body_declarative_part ::=
```

[§ 4.8]
    { package_body_declarative_item }
```ebnf
package_declaration ::=
```

[§ 4.7]
   package identifier is
        package_header
       package_declarative_part
   end [ package ] [ package_simple_name ] ;
```ebnf
package_declarative_item ::=
```

[§ 4.7]
       subprogram_declaration
    |   subprogram_instantiation_declaration
    |   package_declaration
    |   package_instantiation_declaration
   |   type_declaration
   |   subtype_declaration
   |   mode_view_declaration
   |   constant_declaration
   |   signal_declaration
   |   variable_declaration
   |   file_declaration
   |   alias_declaration
    |   component_declaration
   |   attribute_declaration
   |   attribute_specification
   |   disconnection_specification
   |   use_clause
    |   group_template_declaration
   |   group_declaration
   |   PSL_Property_Declaration
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
547
Copyright © 2019 IEEE. All rights reserved.
   |   PSL_Sequence_Declaration
```ebnf
package_declarative_part ::=
```

[§ 4.7]
    { package_declarative_item }
```ebnf
package_header ::=
```

[§ 4.7]
    [ generic_clause
    [ generic_map_aspect ; ] ]
```ebnf
package_instantiation_declaration ::=
```

[§ 4.9]
   package identifier is new uninstantiated_package_name
 [ generic_map_aspect ] ;
```ebnf
package_pathname ::=
   @ library_logical_name . { package_simple_name . } object_simple_name
   [§  8.7]
parameter_map_aspect ::=
 [§ 9.3.4]
   [ parameter map ] ( parameter_association_list )
parameter_specification ::=
 [§ 10.10]
   identifier in discrete_range
partial_pathname ::= { pathname_element . } object_simple_name
```

[§  8.7]
```ebnf
pathname_element ::=
```

[§  8.7]
       entity_simple_name
   |   component_instantiation_label
   |   block_label
   |   generate_statement_label [ ( static_expression ) ]
   |   package_simple_name
```ebnf
physical_incomplete_type_definition ::= units <>
  [§ 5.8.1]
physical_literal ::= [ abstract_literal ] unit_name
 [§ 5.2.4.1]
physical_type_definition ::=
```

[§ 5.2.4.1]
    range_constraint
 units
  primary_unit_declaration
     { secondary_unit_declaration }
  end units [ physical_type_simple_name ]
```ebnf
plain_return_statement ::=
  [§  10.13]
    [ label : ] return [ when condition ];
port_clause ::=
 [§ 6.5.6.3]
   port ( port_list ) ;
port_list ::= port_interface_list
```

[§ 6.5.6.3]
```ebnf
port_map_aspect ::=
  [§ 6.5.7.3]
   port map ( port_association_list )
prefix ::=
```

[§ 8.1]
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
548
Copyright © 2019 IEEE. All rights reserved.
       name
   |   function_call
```ebnf
primary ::=
```

[§ 9.1]
       name
   |   literal
   |   aggregate
   |   function_call
    |   qualified_expression
    |   type_conversion
   |   allocator
    |   ( conditional_expression )
```ebnf
primary_unit ::=
```

[§ 13.1]
       entity_declaration
    |   configuration_declaration
    |   package_declaration
    |   package_instantiation_declaration
   |   context_declaration
   |   PSL_Verification_Unit
```ebnf
primary_unit_declaration ::= identifier ;
 [§ 5.2.4.1]
private_variable_declaration ::=
 [§ 5.6.2]
   private variable_declaration
private_incomplete_type_definition ::= private
 [§ 5.8.1]
procedure_call ::= procedure_name
  [§ 10.7]
    [ generic_map_aspect ] [ parameter_map_aspect  ]
procedure_call_statement ::= [ label : ] procedure_call ;
```

[§ 10.7]
```ebnf
procedure_specification ::=
```

[§ 4.2.1]
   procedure identifier
 subprogram_header
 [ [ parameter ] ( formal_parameter_list ) ]
```ebnf
process_declarative_item ::=
  [§ 11.3]
       subprogram_declaration
    |   subprogram_body
    |   subprogram_instantiation_declaration
    |   package_declaration
    |   package_body
    |   package instantiation_declaration
   |   type_declaration
   |   subtype_declaration
   |   constant_declaration
   |   variable_declaration
   |   file_declaration
   |   alias_declaration
   |   attribute_declaration
   |   attribute_specification
   |   use_clause
```

Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
549
Copyright © 2019 IEEE. All rights reserved.
    |   group_template_declaration
   |   group_declaration
```ebnf
process_declarative_part ::=
```

[§ 11.3]
    { process_declarative_item }
```ebnf
process_sensitivity_list ::= all | sensitivity_list
```

[§ 11.3]
```ebnf
process_statement ::=
 [§ 11.3]
   [ process_label : ]
 [ postponed ] process [ ( process_sensitivity_list ) ] [ is ]
     process_declarative_part
 begin
     process_statement_part
 end [ postponed ] process [ process_label ] ;
process_statement_part ::=
```

[§ 11.3]
    { sequential_statement }
```ebnf
protected_type_body ::=
```

[§ 5.6.3]
   protected body
        protected_type_body_declarative_part
   end protected body [ protected_type_simple name ]
```ebnf
protected_type_body_declarative_item ::=
```

[§ 5.6.3]
       subprogram_declaration
    |   subprogram_body
    |   subprogram_instantiation_declaration
    |   package_declaration
    |   package_body
    |   package_instantiation_declaration
   |   type_declaration
   |   subtype_declaration
   |   constant_declaration
   |   variable_declaration
   |   file_declaration
   |   alias_declaration
   |   attribute_declaration
   |   attribute_specification
   |   use_clause
    |   group_template_declaration
   |   group_declaration
```ebnf
protected_type_body_declarative_part ::=
```

[§ 5.6.3]
    { protected_type_body_declarative_item }
```ebnf
protected_type_declaration ::=
```

[§ 5.6.2]
   protected
 protected_type_header
       protected_type_declarative_part
   end protected [ protected_type_simple_name ]
```ebnf
protected_type_declarative_item ::=
```

[§ 5.6.2]
 subprogram_declaration
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
550
Copyright © 2019 IEEE. All rights reserved.
    |   subprogram_instantiation_declaration
   |   attribute_specification
   |   use_clause
   |   private_variable_declaration
   |   alias_declaration
```ebnf
protected_type_declarative_part ::=
```

[§ 5.6.2]
    { protected_type_declarative_item }
```ebnf
protected_type_definition ::=
```

[§ 5.6.1]
       protected_type_declaration
    |   protected_type_body
```ebnf
protected_type_header ::=
  [§ 5.6.2]
    [ generic_clause
    [ generic_map_aspect ; ] ]
protected_type_instantiation_definition ::=
   [§ 5.6.4]
   new uninstantiated_protected_type_name [ generic_map_aspect ]
qualified_expression ::=
```

[§ 9.3.5]
       type_mark ' ( expression )
   |   type_mark ' aggregate
   |   type_mark ' (  )
```ebnf
range ::=
 [§ 5.2.1]
       range_attribute_name
   |   simple_range
   |   range_expression
range_constraint ::= range range
```

[§ 5.2.1]
```ebnf
record_constraint ::=
```

[§ 5.3.3]
    ( record_element_constraint { , record_element_constraint } )
```ebnf
record_element_constraint ::= record_element_simple_name element_constraint
```

[§ 5.3.3]
```ebnf
record_element_list ::=
 [§ 6.5.2]
   record_element_simple_name { , record_element_simple_name }
record_element_resolution ::= record_element_simple_name resolution_indication
 [§ 6.3]
record_resolution ::= record_element_resolution { , record_element_resolution }
  [§ 6.3]
record_type_definition ::=
   [§ 5.3.3]
   record
    { element_declaration }
 end record [ record_type_simple_name ]
record_mode_view_indication ::=
  [§ 6.5.2]
   view mode_view_name [ of unresolved_record_subtype_indication ]
relation ::=
 [§ 9.1]
```

Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
551
Copyright © 2019 IEEE. All rights reserved.
    shift_expression [ relational_operator shift_expression ]
```ebnf
relational_operator ::= = | /= | < | <= | > | >= | ?= | ?/= | ?< | ?<= | ?> | ?>=
```

[§ 9.2.1]
```ebnf
relative_pathname ::= { ^ . } partial_pathname
```

[§  8.7]
```ebnf
report_statement ::=
      [§ 10.4]
    [ label : ]
 report expression
  [ severity expression ] ;
resolution_indication ::=
```

[§ 6.3]
   resolution_function_name | ( element_resolution )
```ebnf
return_statement ::=
  [§ 10.13]
```

plain_return_statement
   |   value_return_statement
```ebnf
scalar_incomplete_type_definition ::= <>
   [§ 5.8.1]
scalar_type_definition ::=
 [§ 5.2.1]
        enumeration_type_definition
    |    integer_type_definition
    |    floating_type_definition
    |    physical_type_definition
secondary_unit ::=
```

[§ 13.1]
        architecture_body
    |   package_body
```ebnf
secondary_unit_declaration ::= identifier = physical_literal ;
```

[§ 5.2.4.1]
```ebnf
selected_expressions ::=
```

[§ 10.5.4]
    { expression when choices , }
   expression when choices
```ebnf
selected_force_assignment ::=
```

[§ 10.5.4]
   with expression select [ ? ]
 target <= force [ force_mode ] selected_expressions ;
```ebnf
selected_name ::= prefix . suffix
   [§ 8.4]
selected_signal_assignment ::=
```

[§ 10.5.4]
        selected_waveform_assignment
   |   selected_force_assignment
```ebnf
selected_variable_assignment ::=
  [§ 10.6.3]
   with expression select [ ? ]
 target := selected_expressions ;
selected_waveform_assignment ::=
```

[§ 10.5.4]
   with expression select [ ? ]
 target <= [ delay_mechanism ] selected_waveforms ;
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
552
Copyright © 2019 IEEE. All rights reserved.
```ebnf
selected_waveforms ::=
 [§ 10.5.4]
   { waveform when choices , }
       waveform when choices
sensitivity_clause ::= on sensitivity_list
```

[§ 10.2]
```ebnf
sensitivity_list ::= signal_name { , signal_name }
```

[§ 10.2]
```ebnf
sequence_of_statements ::=
```

[§ 10.1]
    { sequential_statement }
```ebnf
sequential_block_statement ::=
  [§ 10.15]
   [ sequential_block_label : ] block  [ is ]
       sequential_block_declarative_part
   begin
        sequential_block_statement_part
   end [ block ] [ sequential_block_label ] ;
sequential_block_declarative_part ::=
 [§ 10.15]
    { process_declarative_item }
sequential_block_statement_part ::=
 [§ 10.15]
    { sequential_statement }
sequential_statement ::=
```

[§ 10.1]
       wait_statement
   |   assertion_statement
   |   report_statement
   |   signal_assignment_statement
   |   variable_assignment_statement
   |   procedure_call_statement
   |   if_statement
    |   case_statement
    |   loop_statement
   |   next_statement
   |   exit_statement
   |   return_statement
   |   null_statement
   |   sequential_block_statement
```ebnf
shift_expression ::=
 [§ 9.1]
    simple_expression [ shift_operator simple_expression ]
shift_operator ::= sll | srl | sla | sra | rol | ror
```

[§ 9.2.1]
```ebnf
sign ::= + | –
```

[§ 9.2.1]
```ebnf
signal_assignment_statement ::=
```

[§ 10.5.1]
       [ label : ] simple_signal_assignment
    |   [ label : ] conditional_signal_assignment
   |   [ label : ] selected_signal_assignment
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
553
Copyright © 2019 IEEE. All rights reserved.
```ebnf
signal_declaration ::=
 [§ 6.4.2.3]
   signal identifier_list : subtype_indication [ signal_kind ] [ := conditional_expression ] ;
signal_kind ::= register | bus
```

[§ 6.4.2.3]
```ebnf
signal_list ::=
```

[§ 7.4]
       signal_name { , signal_name }
   |   others
   |   all
```ebnf
signature ::= [ [ type_mark { , type_mark } ] [ return type_mark ] ]
```

[§ 4.5.3]
```ebnf
simple_configuration_specification ::=
```

[§ 7.3.1]
   for component_specification binding_indication ;
 [ end for ; ]
```ebnf
simple_expression ::=
```

[§ 9.1]
    [ sign ] term { adding_operator term }
```ebnf
simple_force_assignment ::=
```

[§ 10.5.2.1]
   target <= force [ force_mode ] conditional_or_unaffected_expression ;
```ebnf
simple_mode_indication ::=
  [§ 6.5.2]
    [ mode ]  interface_type_indication [ bus ] [ := static_conditional_expression ]
simple_name ::= identifier
 [§ 8.2]
simple_range ::= simple_expression direction simple_expression
  [§5.2.1]
simple_release_assignment ::=
  [§ 10.5.2.1]
   target <= release [ force_mode ] ;
simple_signal_assignment ::=
```

[§ 10.5.2.1]
        simple_waveform_assignment
   |   simple_force_assignment
   |   simple_release_assignment
```ebnf
simple_waveform_assignment ::=
```

[§ 10.5.2.1]
    target <= [ delay_mechanism ] waveform ;
```ebnf
simple_variable_assignment ::=
```

[§ 10.6.2.1]
    target := conditional_or_unaffected_expression ;
```ebnf
slice_name ::= prefix ( discrete_range )
 [§ 8.6]
string_literal ::= " { graphic_character } "
  [§ 15.7]
subprogram_body ::=
 [§ 4.3]
    subprogram_specification is
 subprogram_declarative_part
  begin
     subprogram_statement_part
  end [ subprogram_kind ] [ designator ] ;
```

Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
554
Copyright © 2019 IEEE. All rights reserved.
```ebnf
subprogram_declaration ::=
```

[§ 4.2.1]
    subprogram_specification ;
```ebnf
subprogram_declarative_item ::=
```

[§ 4.3]
       subprogram_declaration
    |   subprogram_body
    |   subprogram_instantiation_declaration
    |   package_declaration
    |   package_body
    |   package_instantiation_declaration
   |   type_declaration
   |   subtype_declaration
   |   constant_declaration
   |   variable_declaration
   |   file_declaration
   |   alias_declaration
   |   attribute_declaration
   |   attribute_specification
   |   use_clause
    |   group_template_declaration
   |   group_declaration
```ebnf
subprogram_declarative_part ::=
```

[§ 4.3]
    { subprogram_declarative_item }
```ebnf
subprogram_header ::=
  [§ 4.2.1]
   [ generic ( generic_list )
    [ generic_map_aspect ] ]
subprogram_instantiation_declaration ::=
 [§ 4.4]
    subprogram_kind identifier is new uninstantiated_subprogram_name [ signature ]
 [ generic_map_aspect ] ;
subprogram_kind ::= procedure | function
 [§ 4.3]
subprogram_specification ::=
  [§ 4.2.1]
    procedure_specification | function_specification
subprogram_statement_part ::=
```

[§ 4.3]
    { sequential_statement }
```ebnf
subtype_declaration ::=
```

[§ 6.3]
   subtype identifier is subtype_indication ;
```ebnf
subtype_indication ::=
```

[§ 6.3]
    [ resolution_indication ] type_mark [ constraint ]
```ebnf
suffix ::=
```

[§ 8.4]
       simple_name
   |   character_literal
    |   operator_symbol
   |   all
```ebnf
target ::=
```

[§ 10.5.2.1]
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
555
Copyright © 2019 IEEE. All rights reserved.
 name
   |   aggregate
```ebnf
term ::=
```

[§ 9.1]
   factor { multiplying_operator factor }
```ebnf
timeout_clause ::= for time_expression
 [§ 10.2]
tool_directive ::= ` identifier { graphic_character }
  [§ 15.11]
type_conversion ::= type_mark ( expression )
  [§ 9.3.6]
type_declaration ::=
```

[§ 6.2]
       full_type_declaration
    |   incomplete_type_declaration
```ebnf
type_definition ::=
```

[§ 6.2]
       scalar_type_definition
    |   composite_type_definition
    |   access_type_definition
    |   file_type_definition
    |   protected_type_definition
   |   protected_type_instantiation_definition
```ebnf
type_mark ::=
   [§ 6.3]
       type_name
   |   subtype_name
unary_expression ::=
  [§ 9.1]
       primary
   |   abs primary
   |   not primary
   |   unary_logical_operator primary
unary_miscellaneous_operator  ::=    abs  |  not | unary_logical_operator
```

[§ 9.1]
```ebnf
unbounded_array_definition ::=
```

[§ 5.3.2.1]
   array ( index_subtype_definition { , index_subtype_definition } )
 of element_subtype_indication
```ebnf
unspecified_type_indication ::=
 [§ 5.8.1]
   type is incomplete_type_definition
use_clause ::=
 [§ 12.4]
   use selected_name { , selected_name } ;
value_return_statement ::=
   [§ 10.13]
    [ label : ] return conditional_or_unaffected_expression;
variable_assignment_statement ::=
  [§ 10.6.1]
 [ label : ] simple_variable_assignment
    |   [ label : ] selected_variable_assignment
```

Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
556
Copyright © 2019 IEEE. All rights reserved.
```ebnf
variable_declaration ::=
```

[§ 6.4.2.4]
   [ shared ] variable identifier_list : subtype_indication [ generic_map_aspect ]
 [ := conditional_expression ] ;
```ebnf
verification_unit_binding_indication ::=
```

[§ 7.3.4]
   use vunit verification_unit_list
```ebnf
verification_unit_list ::= verification_unit_name { , verification_unit_name }
```

[§ 7.3.4]
```ebnf
wait_statement ::=
  [§ 10.2]
    [ label : ] wait [ sensitivity_clause ] [ condition_clause ] [ timeout_clause ] ;
waveform ::=
```

[§ 10.5.2.1]
       waveform_element { , waveform_element }
   |   unaffected
```ebnf
waveform_element ::=
```

[§ 10.5.2.2]
       value_expression [ after time_expression ]
   |   null [ after time_expression ]
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
