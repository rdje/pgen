vhdl_file:: I {$|=1}
-> comment                    .push
-> space                      .push
-> library_clause             .push
-> use_clause                 .push
-> entity_declaration         .push
-> architecture_body          .push
-> package_declaration        .push
-> package_body               .push
-> configuration_declaration  .push

LX {return \@vhdl_file}


comment:        /--.*/                         I.return($IMATCH)
space:          /\s+/                          I.return($IMATCH)
dquote_string:  /"(.+?)(?<!")"/                I.return_m       
library_clause: /(?is)\blibrary\s+(.+?)\s*;/   I.return_m       
use_clause:     /(?is)\buse\s+(.+?)\s*;/       I.return_m       

entity_declaration:    /(?i)\bentity\s+(\w+)\s+is\b/ /(?i)\bend\b(?:\s+entity\b)?(?:\s+\w+)?\s*;/
-? push
-> comment               .push
-> dquote_string         .push
-> port_clause           .push
-> entity_declaration[1] .return_a(map {lc} @IMATCH_LIST)

architecture_body: /(?i)\barchitecture\s+(\w+)\s+of\s+(\w+)\s+is\b/ /(?i)\bbegin\b/ /(?i)\bend\b(?:\s+architecture\b)?(?:\s+\w+)?\s*;/
-> comment                              .push
-> dquote_string                        .push
-> space                                .push
-> subprogram_body                      .push
-> subprogram_declaration               .push
-> type_declaration                     .push
-> subtype_declaration                  .push
-> constant_declaration                 .push
-> signal_declaration                   .push
-> variable_declaration                 .push
-> file_declaration                     .push
-> alias_declaration                    .push
-> component_declaration                .push
-> attribute_declaration                .push
-> attribute_specification              .push
-> configuration_specification          .push
-> use_clause                           .push
-> group_template_declaration           .push
-> group_declaration                    .push
-> disconnection_specification          .push

-> architecture_body[1]                 .return ((map {lc} @IMATCH_LIST), \@architecture_body, call(architecture_statement_part))

architecture_statement_part:
-? push
-> comment                                 .push
-> dquote_string                           .push
-> space                                   .push
-> block_statement                         .push
-> process_statement                       .push
#-> concurrent_procedure_call_statement    .push
#-> concurrent_assertion_statement         .push
-> generate_statement                      .push
-> component_instantiation_statement       .push
-> architecture_body[2]                    {return [@architecture_statement_part]}
-> concurrent_signal_assignment_statement  .push


concurrent_signal_assignment_statement: /(?is)(?:\bwith\s+(.+?)\s+select\s+)?(\w+.*?)\s*<=\s*(.+?)\s*;/  I.return (@IMATCH_LIST[-2, -1, 0])
generate_statement: /(?is)(?:\w+\s*:\s*(?:(for|if)\s+(.+?))\s*)?\bgenerate\b/ /(?is)\bend\s+generate\b.*?;/
-? push
-> comment                                 .push
-> dquote_string                           .push
-> block_statement                        .push
-> process_statement                       .push
#-> concurrent_procedure_call_statement    .push
#-> concurrent_assertion_statement         .push
-> generate_statement                      .push
-> component_instantiation_statement       .push
-> generate_statement[1]                   .return_ma
-> concurrent_signal_assignment_statement  .push


# Just avoid having BLOCK_STATEMENT related issues
block_statement: /(?i)(?:(\w+)\s*:\s*)?\bblock\b/ /(?i)\bend\s+block\b.*?;/
-> block_statement
-> comment
-> dquote_string
-> block_statement[1]  .return_m

component_instantiation_statement: /(?i)(\w+)\s*:\s*(?:entity\s+(\S+)(?:\s+\(\s*(\S+)\s*\))?|configuration\s+(\w+)|(?:component\s+)?(\w+))/  /;/
-> comment                                 .push
-> dquote_string                           .push
-> space                                   .push
-> generic_map_aspect                      .push
-> port_map_aspect                         .push
-> component_instantiation_statement[1]    .return_a (map {lc} @IMATCH_LIST)


generic_map_aspect: /(?i)generic\s+map\s*\(/  /\)/
-> comment                .push
-> dquote_string          .push
-> space                  .push
-> association_element    .push
-> generic_map_aspect[1]  .return_a


port_map_aspect: /(?i)port\s+map\s*\(/  /\)/
-> comment                .push
-> dquote_string          .push
-> space                  .push
-> association_element    .push
-> port_map_aspect[1]     .return_a

# The 'port' is to deal w/ generic_map_aspect's association_element's followed
# by a port_map_aspect. I know it is not ** elegant ** but...
association_element: /(?is)(\w+)\s*=>\s*(.+?)(?=\s*(?:,|\)\s*(?:;|port\b)))/  I.return_m

process_statement: /(?i)(?:(\w+)\s*:\s*)?\bprocess\b/  /(?i)\bbegin\b/ /(?is)\bend(?:\s+postponed)?\s+process\b.*?;/  I {
	#print "\nSTART process_statement <@IMATCH_LIST>\n";
	my $pos_begin
}

-> comment                              .push
-> dquote_string                        .push
-> subprogram_body                      .push
-> subprogram_declaration               .push
-> type_declaration                     .push
-> subtype_declaration                  .push
-> constant_declaration                 .push
-> variable_declaration                 .push
-> file_declaration                     .push
-> alias_declaration                    .push
-> attribute_declaration                .push
-> attribute_specification              .push
-> use_clause                           .push
-> group_template_declaration           .push
-> group_declaration                    .push
-> if_endif 
-> case_endcase 
-> loop_endloop
-> process_statement[1]                 {
       #print "\nBEGIN process_statement <@IMATCH_LIST>\n";
	$pos_begin = pos $$STRING
}

-> process_statement[2]                 {
   my $process_statement_part = substr $$STRING, $pos_begin, $LSPOS - $pos_begin - length $LMATCH;
  #print "\nEND process_statement <@IMATCH_LIST>\n";

   return ['?process_statement:', @IMATCH_LIST, \@process_statement, $process_statement_part];
}


component_declaration:    /(?i)\bcomponent\s+(\w+)\s+is\b/ /(?i)\bend\b(?:\s+component\b)?(?:\s+\w+)?\s*;/
-? push
-> comment                  .push
-> dquote_string            .push
-> port_clause              .push
-> component_declaration[1] .return_ma


package_declaration:    /(?i)\bpackage\s+(\w+)\s+is\b/ /(?i)\bend\b(?!\s+component\b)(?:\s+package\b)?(?:\s+\w+)?\s*;/ 
I {
#  say "############## START package_declaration <@IMATCH_LIST> ##############";
}

-> comment                    .push
-> dquote_string              .push
-> space                      .push
-> subprogram_declaration     .push
-> type_declaration           .push
-> subtype_declaration        .push
-> constant_declaration       .push
-> signal_declaration         .push
-> variable_declaration       .push
-> file_declaration           .push
-> alias_declaration          .push
-> component_declaration      .push
-> attribute_declaration      .push
-> attribute_specification    .push
-> disconnection_specification.push
-> use_clause                 .push
-> group_template_declaration .push
-> group_declaration          .push
-> package_declaration[1]        {
#	say "############## END package_declaration <@IMATCH_LIST> ##############\n";
	return ['?package_declaration:', (map {lc} @IMATCH_LIST), \@package_declaration]
}


package_body: /(?i)\bpackage\s+body\s+(\w+)\s+is\b/ /(?i)\bend(?:\s+package\s+body)?(?:\s+\w+)?\s*;/ 
I  {
#  say "##############       START package_body <@IMATCH_LIST>      ###############";
}

-> comment                   .push                    
-> dquote_string             .push             
-> space                     .push                          
-> subprogram_declaration    .push      
-> subprogram_body           .push                
-> type_declaration          .push        
-> subtype_declaration       .push   
-> constant_declaration      .push   
-> variable_declaration      .push   
-> file_declaration          .push               
-> alias_declaration         .push              
-> use_clause                .push     
-> group_template_declaration.push     
-> group_declaration         .push              
-> package_body[1]                {
#  say "##############        END package_body <@IMATCH_LIST>       ##############";
 return ['?package_body:', @IMATCH_LIST, \@package_body]
}


configuration_declaration: /(?i)\bconfiguration\s+(\w+)\s+of\s+(\w+)\s+is\b/  /(?i)\bend\b(?:\s+configuration\b)?(?:\s+(\w+))?\s*;/
-> use_clause                    .push
-> attribute_specification       .push
-> group_declaration             .push
-> block_configuration           .push
-> configuration_declaration[1]  .return_ma

block_configuration: /(?i)\bfor\b(?!\s+generate)/  /(?i)end\s+for\s*;/
-> use_clause
-> block_configuration
-> block_configuration[1]        .return([])

subprogram_declaration:    /(?i)(?:\b(procedure)|(?:\b(?:pure|impure)\s+)?\b(?<ISFUNC>function))\s+(\w+)(\s*\((?:[^\(\)]++|(?-1))+\))?(?(<ISFUNC>)\s*return\s+(\w+))\s*;/ I {
#  say "subprogram_declaration: (@IMATCH_LIST)";
 return ['?subprogram_declaration:', @IMATCH_LIST]
}

subprogram_body:           /(?i)(?:\b(procedure)|(?:\b(?:pure|impure)\s+)?\b(?<ISFUNC>function))\s+(\w+)(\s*\((?:[^\(\)]++|(?-1))+\))?(?(<ISFUNC>)\s*return\s+(\w+))\s+is\b/   /(?i)\bbegin\b/ /(?is)\bend\b.*?;/ 
I {
#  say "subprogram_body: (@IMATCH_LIST)";
 my $pos_begin;
}


-> comment 
-> dquote_string

-> if_endif 
-> case_endcase 
-> loop_endloop

-> subprogram_body
-> subprogram_declaration
-> type_declaration
-> subtype_declaration
-> constant_declaration
-> variable_declaration
-> file_declaration
-> alias_declaration
-> attribute_declaration
-> attribute_specification
-> use_clause
-> group_template_declaration
-> group_declaration

-> subprogram_body[1] {
   # say "BEGIN subprogram_body<$IMATCH_LIST[1]>";
   $pos_begin = pos $$STRING
}

-> subprogram_body[2] {
   my $subprogram_statement_part = substr $$STRING, $pos_begin, $LSPOS - $pos_begin - length $LMATCH;
   my @subprogram_statement_part = grep {length} map {split /^(\s+)/o} split /((?:\s*--.*\s*)+|\s*;\s*)/o, $subprogram_statement_part;

   # say "END subprogram_body<@IMATCH_LIST>";

   return ['?subprogram_body:', @IMATCH_LIST, \@subprogram_statement_part];
}


begin_end:    /(?i)\bbegin\b/ /(?is)\bend\b.*?;/                   -> comment  -> dquote_string -> begin_end     -> if_endif     -> case_endcase -> loop_endloop  -> begin_end[1]    .return([])
if_endif:     /(?i)\bif\b(?!\s+generate)/ /(?is)\bend\s+if\b.*?;/  -> comment  -> dquote_string -> if_endif      -> case_endcase -> loop_endloop                  -> if_endif[1]     .return([])
case_endcase: /(?i)\bcase\b/ /(?is)\bend\s+case\b.*?;/             -> comment  -> dquote_string -> case_endcase  -> if_endif     -> loop_endloop                  -> case_endcase[1] .return([])
loop_endloop: /(?i)\bloop\b/ /(?is)\bend\s+loop\b.*?;/             -> comment  -> dquote_string -> loop_endloop  -> if_endif     -> case_endcase                  -> loop_endloop[1] .return([])
opar_cpar:    /\(/            /\)/                                 -> comment  -> dquote_string -> opar_cpar                                                      -> opar_cpar[1]    .return([])


# interface_declaration:  I {my @idecl}
# -> interface_constant_declaration        {return call(interface_constant_declaration)}     
# -> interface_signal_declaration          {return call(interface_signal_declaration)}
# -> interface_variable_declaration        {return call(interface_variable_declaration)}
# -> interface_file_declaration            {return call(interface_file_declaration)}


generic_clause:   /(?i)\bgeneric\s*\(/ /\)\s*;/
-? push
-> comment                      .push
-> dquote_string                .push
-> interface_signal_declaration .push
-> generic_clause[1]            .return (@generic_clause ? \@generic_clause : undef)


port_clause:   /(?i)\bport\s*\(/ /\)\s*;/
-? push
-> comment                      .push
-> dquote_string                .push
-> interface_signal_declaration .push
-> port_clause[1]               .return (@port_clause ? \@port_clause : undef)


interface_signal_declaration: /(\w+)\s*:\s*(\w+)\s+(\w+)/ /\s*;|\s*(?=\)\s*;)/
-> signal_decl_range                 {push @IMATCH_LIST, call(signal_decl_range)}
-> interface_signal_declaration[1]   {return ['?port_decl:', [@IMATCH_LIST]]}


signal_decl_range: /\(/ /\)/ I {my (@capt, @msi_lsi)}
LS {push @capt, substr $$STRING, $IPOS, $LSPOS - $IPOS - length $LMATCH}
LE {$IPOS = pos $$STRING}

-> opar_cpar              {my $pos1 = pos($$STRING)-1; call(opar_cpar); my $pos2 = pos $$STRING; push @capt, substr $$STRING, $pos1, $pos2-$pos1}
-> downto_or_to           {(my $msi_lsi = join "", @capt) =~ s/^\s+|\n\s*|\s+$//goi; push @msi_lsi, $msi_lsi; @capt = ()} 
-> signal_decl_range[1]   {
   if (@capt) {
    my $msi_lsi = join "", @capt;
    if ($msi_lsi) {$msi_lsi =~ s/^\s+|\n\s*|\s+$//goi; push @msi_lsi, $msi_lsi};
   }

   return @msi_lsi
}


type_declaration:     /(?is)\btype\s+(\w+)\s+is\s+/ /\s*;/  I {
  #print "START TYPE definition (@IMATCH_LIST)\n";
}

-> record_endrecord
-> type_declaration[1]      {
        #print "END TYPE definition (@IMATCH_LIST)\n";
	return ['?type_declaration:',    @IMATCH_LIST, substr $$STRING, $IPOS, $LSPOS - $IPOS - length $LMATCH]}

record_endrecord:   /(?is)\brecord\s.+?\bend\s+record\s+/


subtype_declaration:  /(?is)\bsubtype\s+(\w+)\s+is\s+(.+?)\s*;/                     I.return_m
constant_declaration: /(?is)\bconstant\s+(.+?)\s*:\s*(.+?)(?:\s*:=\s*(.+?))?\s*;/   I {
  my ($identifier_list, $subtype_indication, $expression) = @IMATCH_LIST;

  return [map {['?constant_declaration:', $_, $subtype_indication, $expression]} split /\s*,\s*/o, $identifier_list]
}

variable_declaration: /(?is)\b(?:shared\s+)?variable\s+(.+?)\s*:\s*(.+?)(?:\s*:=\s*(.+?))?\s*;/ I {
  my ($identifier_list, $subtype_indication, $expression) = @IMATCH_LIST;

  return [map {['?variable_declaration:', $_, $subtype_indication, $expression]} split /\s*,\s*/o, $identifier_list]
}

file_declaration: /(?is)\bfile\s+(.+?)\s*:\s*(.+?)\s*;/ I {
  my ($identifier_list, $remainder_info) = @IMATCH_LIST;

  return [map {['?file_declaration:', $_, $remainder_info]} split /\s*,\s*/o, $identifier_list]
}

alias_declaration:          /(?is)\balias\s+(\S+)\s+(.+?)?\bis\s+(\w+)(?:.*?)\s*;/    I.return_m
attribute_declaration:      /(?is)\battribute\s+(\w+)\s*:\s*(.+?)\s*;/                I.return_m
attribute_specification:    /(?is)\battribute\s+(\w+)\s+of\s+(.+?)\s+is\s+(.+?)\s*;/  I.return_m
group_template_declaration: /(?is)group\s+(\w+)\s+is\s+\(\s*(.+?)\s*\)\s*;/           I.return_m
group_declaration:          /(?is)group\s+(\w+)\s*:\s*(\w+)\s*\(\s*(.+?)\s*\)\s*;/    I.return_m

signal_declaration: /(?is)\bsignal\s+(.+?)\s*:\s*(.+?)(?:\s+(register|bus))?(?:\s*:=\s*(.+?))?\s*;/ I {
  my ($identifier_list, @remainder_info) = @IMATCH_LIST;

  return [map {['?signal_declaration:', $_, @remainder_info]} split /\s*,\s*/o, $identifier_list],
}

configuration_specification: /(?is)\bfor\s+(.+?)\s*:\s*(\w+)\s+(.+?)\s*;/ I {
  my ($instantiation_list, @remainder_info) = @IMATCH_LIST;

  return [map {['?configuration_specification:', $_, @remainder_info]} split /\s*,\s*/o, $instantiation_list]
}

downto_or_to: /(?i)\b(?:downto|to)\b/
disconnection_specification: /(?is)disconnection_specification\s+(.+)\s*;/ I.return_m
