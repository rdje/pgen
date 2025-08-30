package RTLUtils;

use 5.010;
use re 'eval';

use File::Spec;
use File::Path;
use Cwd qw(chdir);

use HUtils;
use Table;
use LinkedSpec;

our @ISA = 'PPlugin';

sub new {bless {}, $_[0]}

sub drive_entity    ($$$%) {drive_entity_component (@_[0 .. 2], component=>0, @_[3 .. $#_])}
sub drive_component ($$$%) {drive_entity_component (@_[0 .. 2], component=>1, @_[3 .. $#_])}

sub string_align ($) {
my ($strlst) = @_;

 my $maxlen = (sort {$b <=> $a} map {length} @$strlst)[0];
 return {map {$_ => $_." " x ($maxlen - length)} @$strlst}
}

sub string_realign ($$) {
my ($alignedset, $newitems) = @_; 

 return  string_align ([keys %$alignedset, @$newitems])
}


sub drive_entity_component {
my ($conf, $modules, $mod, %option) = @_;

 my @alignthem;
 my $alignthem;
 #unless ($modules->{$mod}{aligned_ports}) {
  HUtils::Recurse($modules->{$mod}{port}{list}, sub {push @alignthem, map {$$_[0]} @{$_[1]}});
  $modules->{$mod}{aligned_ports} = $alignthem = string_align \@alignthem;

  my @align_direction; 
  HUtils::Recurse($modules->{$mod}{port}{list}, sub {push @align_direction, map {$$_[1]} @{$_[1]}});
  my $align_direction = string_align \@align_direction; 

 #}


 my $entity_or_component = $option{component} ? "COMPONENT" : "ENTITY";

 print STDOUT "-- Driving module *$mod*\n" if $option{debug};

 # Retrieving the current file handle
 my $c_fh = select;

 my ($entity, $inmem, $inmemory);
 unless ($option{component})  {
  if ($option{each}) {
   mkpath "Entities";
   open ($entity, "> Entities/${mod}_e.vhd") || die "-E- Can't write entity of *$mod*,";
   select $entity;
  }
 } elsif ($option{in_memory}) {
   open ($inmem, '>', \$inmemory) || die "-E- Can't write component of *$mod*,";
   select $inmem;
 }


 print PPlugin->exec('add_header_n_context_clause', $conf, %option) unless $option{component};

 print "$entity_or_component  $mod  IS";

 if ($option{component} && $option{generic}) {
  $alignthem = RTLUtils::string_realign ($alignthem, [keys %{$option{generic}}]);

  print "\n  GENERIC (\n";

  my @generic  = map {"    $$alignthem{$_} : INTEGER;"}  keys %{$option{generic}};
  $generic[-1] =~ s/;$//o;
  print join "\n",  @generic;

  print "\n  );";
 }

 print "\n  PORT (\n";
 my $grp;
 my $once;
 HUtils::Recurse($modules->{$mod}{port}{list}, sub {my ($info, $portlst) = @_;
   print ";\n" if $once && @$portlst;
   print "\n"  if $grp && $grp ne $info->[$#$info];

   # We have a Single slot/bit signal if field #3 (MSI) if Undefined/Zero
   if (@$portlst) {
     my @actualist = $option{sort} ? sort {$a->[0] cmp $b->[0]} @$portlst : @$portlst;
     print join(";\n", map {
                           if ($option{template_info}) {
                            # Gathering information for generating the corresponding template
                            push  @{$option{template_info}{$mod}}, [$$_[0], $$_[1], !$$_[3] ? 1 : $$_[3] - $$_[4] + 1];
                           }

			   "    $$alignthem{$$_[0]} : $$align_direction{$$_[1]} ".
                          $$_[2].(defined $$_[3] ? "($$_[3] DOWNTO $$_[4])" : '')
                         } @actualist);
   } else {
   
    print "-- (RTLUtils) ########################> NO PORT LIST $entity_or_component <$mod>\n";
   }

   ++$once;
   $grp = $info->[$#$info];
 }); 

 print "\n  );\n";
 print "END $entity_or_component  $mod;\n\n";

 select $c_fh;
 return $inmemory if $option{in_memory};
}

sub _drive_instances ($$$$%) {
my ($conf, $modules, $of_module, $inst_info, %options) = @_;

  my @align_port;
  my $align_port;
  my @align_net;
  my $align_net;
  HUtils::Recurse($modules->{$of_module}{port}{list}, sub {push @align_port, map {$$_[0]} @{$_[1]}});
  $align_port = string_align \@align_port;
  HUtils::Recurse($inst_info, sub {push @align_net, $_[1] eq '__open__' ? 'open' : $_[1]});
  $align_net  = string_align \@align_net;

  print "-- ===== Instance(s) of module $of_module =====\n";
  foreach my $cinstance (sort {$a cmp $b} keys %$inst_info) {
   print "$cinstance  :  $of_module\n";
   print "  PORT MAP (\n";

   
   my @a2d;
   HUtils::Recurse($modules->{$of_module}{port}{list}, sub {my ($info, $portlst) = @_;
     # Ports are divided into groups and groups are separated by an empty line
     push @a2d, [map {my $p = $$_[0]; my $direction = $$modules{$of_module}{port}{$p}{direction}; my $n = $$inst_info{$cinstance}{$p};
	              [$$align_port{$p}."  =>  ".$$align_net{$n eq '__open__' ? 'open' : $n}, 
	              "-- $direction".(defined $$modules{$of_module}{port}{$p}{msi} ? ($direction eq 'IN' ? "   " : "  ").(join ':', map {$$modules{$of_module}{port}{$p}{$_}} qw/msi lsi/) : "")]
	             } @$portlst];
   }); 

   my $grp_index = 0;
   foreach my $ctable (@a2d) {
    print "\n" if $grp_index;
    my $is_last_grp = $grp_index == $#a2d;

    my $conindex = -1;
    print join "\n", map {' ' x 4 . $_} map {++$conindex; my $sep = $is_last_grp && $conindex == $#$ctable ? '   ' : ' , '; join $sep, @$_} @$ctable;

    ++$grp_index
   }

   print "\n  );\n\n\n";
  }
}

sub drive_instances_in ($$$%) {
my ($conf, $modules, $in_module, %options) = @_;

 print "\n\n";
 HUtils::KeyGrep($$modules{$in_module}, qr/hierarchy\s+\S+/o, sub {
		my $cm = $_[0][-1]; 

		return unless $cm =~ /$options{connect}/ && (!$options{unconnect} || $cm !~ /$options{unconnect}/); 
		_drive_instances $conf, $modules, $cm, $_[1], %options
 });
}

sub drive_macroblock (@) {
my ($conf, $modules, $top, %options) = @_;

# --connect=sub-block-selection-RE
# --unconnect=sub-block-rejection-RE
# --entity=0|1
# --macroname=Name
# --sort=0|1

 
 # Building the full connectivity structure
 my %full_connectivity;
 my %netinfo;
 HUtils::Recurse($$modules{$top}{hierarchy}, sub {my ($info, $topnet) = @_;
   my ($cmodule, $cinstance, $cport) = @$info;

   unless (defined $$modules{$cmodule}) {
    print "-W- Module *$cmodule* was not seen, skipping.\n";

    $$conf{module_not_seen}{$cmodule} = 1;
    return
   }

   # This is NEEDED due to very annoying side-effect of Perl Autovivication Perl feature, yet VERY POWERFUL !
   return if $$conf{module_not_seen}{$cmodule};

   # Only connectivity between selected blocks will be generated
   #return   unless $cmodule =~ /$options{connect}/ && (!$options{unconnect} || $cmodule !~ /$options{unconnect}/);

   # This corresponds to dangling ports, like in   .fooport (),
   return if     $topnet eq '__open__';

   my $type_of_count;
   # Needed for when no (non)-selected read this net 
   $netinfo{$topnet}{selected_info}{input_count}     ||= 0;
   $netinfo{$topnet}{non_selected_info}{input_count} ||= 0;

   if ($cmodule =~ /$options{connect}/ && (!$options{unconnect} || $cmodule !~ /$options{unconnect}/)) {
     $type_of_count =  "selected_module_count";

     $$conf{debug} && print "=[$top]===<$options{macroname}>====YES========== IS SELECTED <$cmodule> <$cinstance> <$cport> <$$modules{$cmodule}{port}{$cport}{direction}>\n";

     # Incrementing the number of INPUT or OUTPUT pin of the selected module connected to this net
     $netinfo{$topnet}{selected_info}{lc($$modules{$cmodule}{port}{$cport}{direction})."put_count"}++;
   } else {
     $type_of_count =  "non_selected_module_count";

     # Incrementing the number of INPUT or OUTPUT pin of the selected module connected to this net
     $netinfo{$topnet}{non_selected_info}{lc($$modules{$cmodule}{port}{$cport}{direction})."put_count"}++;

     $$conf{debug} && print "=[$top]===<$options{macroname}>====NO========== ISNOT SELECTED <$cmodule> <$cinstance> <$cport> <$$modules{$cmodule}{port}{$cport}{direction}>\n";
   }

   $netinfo{$topnet}{$type_of_count}++;
   $netinfo{$topnet}{connection_count}++;



   push @{$full_connectivity{$topnet}}, {module=>$cmodule, instance=>$cinstance, port=>$cport, direction=>$$modules{$cmodule}{port}{$cport}{direction}};
 });

 if ($$conf{debug}) {
   foreach (sort {$a cmp $b} keys %full_connectivity) {
    print "======= CONNECTIVITY ($_) : ==========\n";
    foreach (@{$full_connectivity{$_}}) {
     while (my ($k, $v) = each %$_) {print "\t<$k-$v>"}
     print "\n"
    }
    print "\n"
   }
 }


 # Nets that are to be considered as ==PORTS== of the macro to be created are those
 # + Only Selected Modules' pins is(are) connected
 #    - #module == 1: 
 #      connected to ONLY ONE pin.
 #      In that case the port to be created will have the same direction as that pin        ===> INPUT or OUTPUT
 #    
 #    - #module > 1:
 #      connected to ONLY INPUT pins                                                        ===> INPUT
 #
 # + Also connected to non selected Modules
 #    - #module > 1:
 #      + connected to ONLY INPUT pins of the Selected moduless                             ===> INPUT
 #      + connected to INPUTs and and OUTPUT (no tri-state) pins of the Selected modules    ===> an INTERNAL net (for the selected) + an OUTPUT port (for the non selected) 
 #      
 #
 #my %ports; foreach my $k (sort {$a cmp $b} keys %full_connectivity) {my $v = $full_connectivity{$k}; $ports{$k} = $v->[0]{direction} if @$v == 1 || @$v == grep {$$_{direction} eq "IN"} @$v}
 #my @nets = grep {!$ports{$_}}  keys %full_connectivity;


 my %ports;
 my %output_internal_nets;
 my @nets;
 foreach my $cnet (sort {$a cmp $b} keys %full_connectivity) {
  # Nets that are not connected to selected module(s) should be ignored
  unless ($netinfo{$cnet}{selected_module_count}) {
   $$conf{debug} && print "#[$top]###<$options{macroname}>######################## NOT CONNECTED to selected <$cnet>\n";
   next
  }

   $$conf{debug} && print "%[$top]%%%<$options{macroname}>%%%%%%%%%%%%%%%%%%%%%%%% CONNECTED to selected <$cnet>\n";
  # If all of these connected modules take this net as INPUT then we should declare it as an INPUT port 
  if ($netinfo{$cnet}{selected_module_count} == $netinfo{$cnet}{selected_info}{input_count}) {
   $ports{$cnet} = "IN";

  } else {
   # This net is driven by one selected module

   # This net should be seen as an output if it is connected to at least one input pin of a non-selected module
   # or if it is an output port of the top level module
   if ($netinfo{$cnet}{non_selected_info}{input_count} || ($$modules{$top}{port}{$cnet} && $$modules{$top}{port}{$cnet}{direction} eq "OUT")) {
    $ports{$cnet} = "OUT";

    # An internal net needs to be created if this net is also read in by at least one other selected module
    $output_internal_nets{$cnet} = 1; # FIXME NOT IMPLEMENTED YET
   } else {
    # We have a simple internal net
    push @nets, $cnet;
   }
  }
 }



 # Interface generation
 #
 # When drive_macroblock is called via drive_architecture  we have ($options{macroname} eq $top) == TRUE
 # So there is should not be any interface change.
 $modules->{$options{macroname}}{port}{list}{default} = [map {my $p = $_; [$p, map {$_ eq '-'         ? (defined $$modules{$top}{net}{$p}{msi} ? "STD_LOGIC_VECTOR" : "STD_LOGIC") : 
		                                                                  ($_ eq 'direction' ? $ports{$p}                                                         : 
											               $$modules{$top}{net}{$p}{$_})} qw/direction - msi lsi/]}
                                                               sort {$a cmp $b} keys %ports] unless $options{macroname} eq $top; 


 drive_entity $conf, $modules, $options{macroname}, each=>1  unless $options{macroname} eq $top;

 my $cf = select;
 open (my $mb, "> Entities/$options{macroname}".($options{entity} || "_a").".vhd") || die "-E- Can't write architecture of *$options{macroname}*,";
 select $mb;

 print PPlugin->exec('add_header_n_context_clause', $conf, %options);

 print "ARCHITECTURE $options{macroname}_arch OF $options{macroname} IS\n";
 foreach (sort {$a cmp $b} keys %{$modules->{$top}{hierarchy}}) {
  drive_component $conf, $modules, $_, %options if $options{macroname} eq $top || /$options{connect}/ && (!$options{unconnect} || !/$options{unconnect}/);
 }

 # Local signals' generation
 my $netalign = string_align \@nets;
 print join(";\n", map {"SIGNAL  $$netalign{$_}  :  ".(defined $$modules{$top}{net}{$_}{msi} ? "STD_LOGIC_VECTOR($$modules{$top}{net}{$_}{msi} DOWNTO $$modules{$top}{net}{$_}{lsi})" : "STD_LOGIC")} @nets), ";\n\n";

 print "BEGIN\n";
 drive_instances_in $conf, $modules, $top, %options;
 print "END ARCHITECTURE $options{macroname}_arch;\n";

 select $cf;
}

sub drive_architecture ($$$%) {drive_macroblock @_, macroname=> $_[2], entity=>0}


my $p;
my $m;
sub verilog_parse ($$%) {
my ($conf, $vf, %options) = @_;


 print  "-I- Processing $vf ..\n"             unless $options{resume};
 print  "-I- Resuming processing of $vf ..\n" if     $options{resume};

 open (my $cf, "filepp ".($$conf{_verilog_info_}{includirectories} || "")." -kc '`' -mp '`' -pb $vf 2>&1 |"); local $/; my $v = <$cf>; $v =~ s/\/\/.*//go; $v =~ s/\/\*.*?\*\///sgo;

 my $mi;
 foreach (@{Table::list2table([$v =~ /(\S+):(\d+):\s+include\s+file\s+"(.+?)"\s+not\s+found/gio], 3)}) {
  print "-E- Missing include file *$$_[2]*\n";

  $$conf{_verilog_info_}{missing_includes}{$$_[0]}{$$_[1]}{$$_[2]} = 1;
  $$conf{_verilog_info_}{files_in_trouble}{$vf} = 1;

  ++$mi
 }

 if ($mi) {
  print "-I- Postponing processing of *$vf*\n";
  return
 }

 $$conf{_current_file}     = $vf;

 my $ms       = qr/\b(?:macro)?module\s+\w+\s*\(.*?\)\s*;/so;                            # 0
 my $me       = qr/\bendmodule\b/so;                                                     # 1
 my $sx       = qr/\b(?:input|output|inout|reg|wire|assign|integer|parameter)\b.+?;/so;  # 2
 my $function = qr/\bfunction\b.+?\bendfunction\b/so;                                    # 3
 my $task     = qr/\btask\b.+?\bendtask\b/so;                                            # 4
 my $initial  = qr/\binitial\s*begin\b/o;                                                # 5
 my $alwaysat = qr/\balways\s*\@\(.+?\)\s*begin\b/so;                                    # 6
 my $ob       = qr/\{/o;                                                                 # 7
 my $cb       = qr/\}/o;                                                                 # 8
 my $op       = qr/\(/o;                                                                 # 9
 my $cp       = qr/\)/o;                                                                 # 10
 my $begin    = qr/\bbegin\b/o;                                                          # 11
 my $end      = qr/\bend\b/o;                                                            # 12
 my $specify  = qr/\bspecify\b.+?\bendspecify\b/so;                                      # 13
 # Builtin Primitive/Gate instantiation
 my $pinst    = qr/$$conf{vp}.+?;/so;                                                    # 14
 # Module/UDP instantiation                                                            
 my $minst    = qr/\b\w+(?:\s*#\(.+?\))?(?:\s*\w+?)?\s*\(.*?\)\s*;/so;                   # 15

 my $highlev  = qr/$ms       (?{$p = 0})   |
                   $me       (?{$p = 1})   |
                   $sx       (?{$p = 2})   |
                   $function (?{$p = 3})   |
                   $task     (?{$p = 4})   |
                   $initial  (?{$p = 5})   |
                   $alwaysat (?{$p = 6})   |
                   $ob       (?{$p = 7})   |
                   $op       (?{$p = 9})   |
                   $begin    (?{$p = 11})  |     
                   $specify  (?{$p = 13})  |
                   $pinst    (?{$p = 14})  |     
                   $minst    (?{$p = 15})
                  /xo;

 my $ocb      = qr/$ob    (?{$p = 0})| $cb  (?{$p = 8})/xo;  # open/closing brace
 my $ocp      = qr/$op    (?{$p = 0})| $cp  (?{$p = 10})/xo; # open/closing parenthesis
 my $ocsb     = qr/$begin (?{$p = 0})| $end (?{$p = 12})/xo; # open/closing sequential block


 vkw_init ($conf);
 vp_init  ($conf);


 # [0] = code ref
 # [1] = openin & closing RE
 my $ocsym    = sub {while (1) {return unless $v =~ /$_[1]/g; return if $p; &{$_[0]}}};

 my $ret;
 my $pos;
 my $type;
 while (1) {
  last unless $v =~ /$highlev/go;

  do {add_verilog_simple_info($conf, $&); next}                      if $p == 0;  # macromodule|module
  do {add_verilog_simple_info($conf, $&); next}                      if $p == 2;  # input|output|inout|reg|wire|assign|integer|parameter
  do {$$conf{_verilog_function}{($& =~ /(\w+)\s*;/o)[0]} = 1; next}  if $p == 3;  # function
  next                                                               if $p == 4;  # task
  do {$pos = pos $v; &$ocsym($ocsym, $ocsb)}                         if $p == 5;  # initial + sequential block
  do {$pos = pos($v) - length $&; &$ocsym($ocsym, $ocsb); always_at($conf, substr ($v, $pos, pos($v) - $pos -3))}  if $p == 6;  # always  + sequential block
  do {$pos = pos $v; &$ocsym($ocsym, $ocb)}                          if $p == 7;  # opening brace
  do {$pos = pos $v; &$ocsym($ocsym, $ocp)}                          if $p == 9;  # opening parenthesis
  do {$pos = pos $v; &$ocsym($ocsym, $ocsb)}                         if $p == 11; # opening sequential block
  do {gate_primitive_instantiation ($conf, $&); next}                if $p == 14; # gate/primitive instantiation
  do {module_udp_instantiation     ($conf, $&); next}                if $p == 15; # module/udp instantiation

  do {# ENDMODULE
   
   # ASSIGNs final handling
   # ARCs definition ==> Combinational
   my %output;
   HUtils::Recurse($conf->{module}{$$conf{_current_module}}{assign}, sub {my ($i) = @_;
    my ($current_in, $current_out) = reverse @$i;

    $output{$current_out} = 1;
    
    my $assign_module_name = $$conf{_current_module}."_assign_".$current_out;
    my $msi                = $conf->{module}{$$conf{_current_module}}{net}{$current_in}{size} !~ /-\?-/o ? $conf->{module}{$$conf{_current_module}}{net}{$current_in}{size} - 1 : '-?-';

    if ($$conf{debug}) {
    print "=================================> ASSIGN:  PLEASE CHECK Module<$$conf{_current_module}> file<$$conf{_current_file}> Current_in<$current_in> --> <$current_out>\n" unless
     $conf->{module}{$$conf{_current_module}}{net}{$current_in}{size}
    }

    $conf->{module}{$assign_module_name}{port}{$current_in} = {direction => "IN", 
                                                               msi       => $msi,
                                                               lsi       => 0
                                                              };

    # Model the (current_in ---> current_out) Timing Arc in the current *assign*
    $conf->{module}{$assign_module_name}{arcs}{$current_in} = $current_out;

    # Connecting port $current_in of instance $assign_module_name."_inst" of module $assign_module_name
    $conf->{module}{$$conf{_current_module}}{hierarchy}{$assign_module_name}{$assign_module_name."_inst"}{$current_in} = $current_in;

    # Adding the port to its interface
    push @{$conf->{module}{$assign_module_name}{port}{list}{default}}, [$_, "IN", defined $msi ? "STD_LOGIC_VECTOR" : "STD_LOGIC", $msi, 0];
   });

   foreach (sort {$a cmp $b} keys %output) {
     my $assign_module_name = "$$conf{_current_module}_assign_$_";
     my $msi                = $conf->{module}{$$conf{_current_module}}{net}{$_}{size} !~ /-\?-/o ? $conf->{module}{$$conf{_current_module}}{net}{$_}{size} - 1 : '-?-';

     $conf->{module}{$assign_module_name}{port}{$_} = {direction => "OUT", 
                                                       msi       => $msi,
                                                       lsi       => 0
                                                      };

    # Connecting port $_ of instance $assign_module_name."_inst" of module $assign_module_name
    $conf->{module}{$$conf{_current_module}}{hierarchy}{$assign_module_name}{$assign_module_name."_inst"}{$_} = $_;

    # Adding the port to its interface
    push @{$conf->{module}{$assign_module_name}{port}{list}{default}}, [$_, "OUT", defined $msi ? "STD_LOGIC_VECTOR" : "STD_LOGIC", $msi, 0];

    $conf->{module}{$$conf{_current_module}."_assign_".$_}{is_sequential} = 0;                                                  
   }

   # ALWAYS final handling
   my $always_idx = 0;
   foreach my $type (keys %{$conf->{module}{$$conf{_current_module}}{always}}) {
     my $tinfo              = $conf->{module}{$$conf{_current_module}}{always}{$type}; 
     my $always_module_name = "$$conf{_current_module}_always_$type".$always_idx;

     # Defining ports
     foreach my $dir (keys %$tinfo) {
      foreach (@{$$tinfo{$dir}}) {
       my $msi       = $conf->{module}{$$conf{_current_module}}{net}{$_}{size} !~ /-\?-/o ? $conf->{module}{$$conf{_current_module}}{net}{$_}{size} - 1 : '-?-';;
       my $direction = $dir eq 'outputs' ? "OUT" : 'IN';

       $conf->{module}{$always_module_name}{port}{$_} = {direction => $direction, 
                                                         msi       => $msi,
                                                         lsi       => 0
                                                        }; 

       # Connecting port $_ of instance $always_module_name."_inst" of module $always_module_name
       $conf->{module}{$$conf{_current_module}}{hierarchy}{$always_module_name}{$always_module_name."_inst"}{$_} = $_;

       # Adding the port to its interface
       push @{$conf->{module}{$always_module_name}{port}{list}{default}}, [$_, $direction, defined $msi ? "STD_LOGIC_VECTOR" : "STD_LOGIC", $msi, 0];
      }
     }


     if ($type eq 'combi') {
      # Defining Timing Arcs from all inputs to all outputs
      $conf->{module}{$always_module_name}{arcs} = {map {my $ci = $_; map {$ci => $_} @{$$tinfo{outputs}}} @{$$tinfo{inputs}}};
      $conf->{module}{$always_module_name}{is_sequential} = 0;
     } else {
      $conf->{module}{$always_module_name}{is_sequential} = 1;
     }

     ++$always_idx
   }

   undef $conf->{module}{$$conf{_current_module}}{assign};
   $$conf{_verilog_function}      = {};
   $$conf{_verilog_top_regmemory} = {};
  }                                             if $p == 1; # ENDMODULE

 } # WHILE

}


sub add_verilog_simple_info {
my ($conf, $info) = @_;

 local $_ = $info;

 my ($itype) = /(\w+)/o;

 my $bus_re  = qr/\[(.+?):(.+?)\]/o;
 if ($itype =~ /input|output|inout/o) {
  my @bus_info    = map {value_substitute($conf, $_)} grep {defined} m/$bus_re/;
  m/$itype\s*$bus_re?/; my @signalist  = map {(m/(\w+)/o)[0]} split /\s*,\s*/o, $';
  
  my @port_info   = $$conf{dirmap}{$itype};
  push @port_info, @bus_info ? ("STD_LOGIC_VECTOR", @bus_info): ("STD_LOGIC", 0, 0);

  my @msi_lsi = @bus_info ? (msi=> $bus_info[0], lsi=> $bus_info[1]) : (msi=>0, lsi=>0);
  #  0      1       2    3   4     5
  # port direction type MSI LSI RTL_file ...
  foreach (@signalist) {
   push @{$conf->{module}{$$conf{_current_module}}{port}{list}{default}}, [$_, @port_info, Cwd::abs_path($$conf{_current_file})];
   $$conf{module}{$$conf{_current_module}}{port}{$_} = {connex=>0, map {$$conf{portinfo_map}{$_} => $port_info[$$conf{portinfo_map}{field}{$_}]} 0 .. 2};
   my @undefined = grep {!defined} @msi_lsi;
   $$conf{module}{$$conf{_current_module}}{net}{$_}  = {connex=>0, size=> @undefined ? '-?-' : (@bus_info ? $bus_info[0] - $bus_info[1] + 1 : 1), @msi_lsi};
  }

 } elsif ($itype =~ /reg|wire/o) {
  my @bus_info  = map {value_substitute($conf, $_)} grep {defined} m/$bus_re/;
  m/$itype\s*$bus_re?/; my @net_list  = map {my ($n) = /(\w+)/o; $$conf{_verilog_top_regmemory}{$n} = 1 if /\[/o; $n} split /\s*,\s*/o, $';

  my @msi_lsi = @bus_info ? (msi=> $bus_info[0], lsi=> $bus_info[1]) : (msi=>0, lsi=>0);

  my @undefined = grep {!defined} @msi_lsi;
  $$conf{module}{$$conf{_current_module}}{net}{$_} = {size=> @undefined ? '-?-' : (@bus_info ? $bus_info[0] - $bus_info[1] + 1 : 1), connex=>0, @msi_lsi} foreach (@net_list);

 } elsif ($itype eq 'parameter') {
  $$conf{_verilog_param}{$$_[0]} = $$_[1] foreach (@{Table::list2table([map {/\s*=\s*/o; my @t = ($`, $'); $t[0] =~ /(\S+)$/o, $t[1]} map {split /\s*,\s*/o} /\bparameter\s+(.+?)\s*;/gso], 2)});
  parameter_substitute($conf);

 } elsif ($itype eq 'module') {
    $$conf{_verilog_param}   = {};
   ($$conf{_current_module}) = /module\s+(\w+)/o;

   print "\tMODULE $$conf{_current_module}\n" if $$conf{show};

 } elsif ($itype eq 'assign') {
  # The LHS of an assign may be a concatenation {...}
  my ($a_lhs)  = /assign\s*((?s).+?)\s*=\s*/o;
  my  $a_rhs   = parameter_remove($conf, $'); 

  my $rhs_info = Table::list2table([$a_rhs =~ /(?<!'|\.)\b([a-z]\w*)(?:\[(.*?)\])?/igo], 2);
  # Processing subscript content of reg memories in RHS, if any
  my @more_inp = keys %{{map  {$_ => 1} 
                         map  {($$_[1] =~ /(?<!'|\.)\b([a-z]\w*)(?:\[.*?\])?/igo)} 
                         grep {defined $$_[1] && $$conf{_verilog_local_regmemory}{$$_[0]}} @$rhs_info}};

  foreach (grep {!$$conf{_verilog_function}{$_} && $$conf{module}{$$conf{_current_module}}{net}{$_}} @more_inp, map {$$_[0]} @$rhs_info) {
   # All potential LHS share the same set of RHS nets
   foreach my $clhs (grep {$$conf{module}{$$conf{_current_module}}{net}{$_}} $a_lhs =~ /(?<!'|\.)\b([a-z]\w*)(?:\[.*?\])?/igo) {
    $conf->{module}{$$conf{_current_module}}{assign}{$clhs}{$_} = 1 
   }
  }
 }
}

sub always_at {
my ($conf, $m) = @_;

 $m = parameter_remove($conf, $m);

 #print "ALWAYS-AT =$m=\n";
 my %is_seq;
(my $sensitivity = ($m =~ /\@\((.+?)\)\s*begin/so)[0]) =~ s/\b(?:posedge|negedge|or)\b/$is_seq{$&} = 1; ""/igoe;
 my %sensitivity = map {$_ => 1} $sensitivity =~ /(\w+)(?:\[.+?\])?/go;
 my @sensitivity = keys %sensitivity;


 my %local_reg   = map {m/(?:\[.+?\])?/o; map {my ($n) = /(\w+)/o; $$conf{_verilog_local_regmemory}{$n} = 1 if /\[/o; $n => 1} split /\s*,\s*/o, $'} $m =~ /\breg\b(.+?);/gso;

 my $o           = qr/\b(?:if|case[zx]?|for)\s*\(/o;
 my $c           = qr/\)/o;
 if ($is_seq{posedge} || $is_seq{negedge}) {
  # Get "if ()" and "case[zx]? ()" constructs in case we have a sequential ALWAYS
  my @ifcase_conditions;
  while ($m =~ /$o/g) {
   my $is_for = $& =~ /for/o;
   my $pos1   = (my $pos  = pos($m)) - length $&;
   closing_re ($o, $c, $m);
   push @ifcase_conditions, substr $m, $pos, pos($m) - $pos -1 unless $is_for;

   substr $m, $pos1, pos($m) - $pos1, "";
  }

  my $lhs_rhs     = Table::list2table([$m =~ /(\w+)(?=(?:\[.*?\])?\s*<=((?s).+?);)/go], 2);
  my @seq_outputs = grep {!$local_reg{$_} && $$conf{module}{$$conf{_current_module}}{net}{$_}} keys %{{map {$$_[0] => 1} @$lhs_rhs}};
  my $rhs_info    = Table::list2table([join(" ", map {$$_[1] => 1} @$lhs_rhs) =~ /(?<!'|\.)\b([a-z]\w*)(?:\[(.*?)\])?/igo], 2);
  # Processing subscript content of reg memories in RHS, if any
  my @more_inputs = keys %{{map  {$_ => 1} 
                            map  {($$_[1] =~ /(?<!'|\.)\b([a-z]\w*)(?:\[.*?\])?/igo)} 
                            grep {defined $$_[1] && ($$conf{_verilog_local_regmemory}{$$_[0]} || $$conf{_verilog_top_regmemory}{$$_[0]})} @$rhs_info}};

  my %seq_outputs = map  {$_ => 1} @seq_outputs; 
  my @ctrl_inputs = keys %{{map {$_ => 1} join(" ", @ifcase_conditions) =~ /(?<!'|\.)\b([a-z]\w*)(?:\[.*?\])?/igo}};
  my @data_inputs = keys %{{map {$$_[0] => 1} @$rhs_info}};
  my @inputs      = grep {!$local_reg{$_} && !$seq_outputs{$_} && !$$conf{_verilog_function}{$_} && $$conf{module}{$$conf{_current_module}}{net}{$_}} 
                      keys %{{map {$_ => 1} @ctrl_inputs, @data_inputs, @more_inputs, @sensitivity}};

  print "ALWAYS-AT SEQUENTIAL:\nINPUTs<@inputs>\nENDPOINTs/OUTPUTs<@seq_outputs>\n\n"  if $$conf{debug};

  $conf->{module}{$$conf{_current_module}}{always}{seq}{inputs}  = \@inputs;
  $conf->{module}{$$conf{_current_module}}{always}{seq}{outputs} = \@seq_outputs;

 } else {

  while ($m =~ /$o/g) {
   my $pos = pos($m) - length $&;
   closing_re ($o, $c, $m);
   substr $m, $pos, pos($m) - $pos, "";
  }

  my $lhs_rhs       = Table::list2table([$m =~ /(\w+)(?:\[.*?\])?\s*=(?!=)((?s).+?);/go], 2);
  my %arhs          = map {$_ => 1} grep {!$sensitivity{$_}} join(" ", map {$$_[1] => 1} @$lhs_rhs) =~ /(?<!'|\.)\b([a-z]\w*)(?:\[.*?\])?/igo;
  my @combi_outputs = grep {!$local_reg{$_} && $$conf{module}{$$conf{_current_module}}{net}{$_} && !$arhs{$_}} keys %{{map {$$_[0] => 1} @$lhs_rhs}};
  print "ALWAYS-AT COMBINATIONAL:\nINPUTs<@sensitivity>\nOUTPUTs<@combi_outputs)\n\n"  if $$conf{debug};

  $conf->{module}{$$conf{_current_module}}{always}{combi}{inputs}  = \@sensitivity;
  $conf->{module}{$$conf{_current_module}}{always}{combi}{outputs} = \@combi_outputs;
 }

 $$conf{_verilog_local_regmemory} = {};
}

sub parameter_substitute ($)  {my ($conf)     = @_; my $h = $$conf{_verilog_param}; $h->{$_} = value_substitute($conf, $_) foreach (keys %$h)}
sub parameter_remove     ($$) {my ($conf, $v) = @_; my $h = $$conf{_verilog_param}; my $p = join('|', keys %$h); $v =~ s/$p//g if $p; $v}

sub value_substitute  {
my ($conf, $vs) = @_;

 my $h = $$conf{_verilog_param};
 my $identifier = qr/(?<!'|")\b[a-z]\w*/io;
 die unless defined $vs;
 while ($vs =~ $identifier) {
   my $start_string = $vs;

   $vs =~ s/$identifier/vsubstitute($conf, $h, $&)/gioe;

   last if $start_string eq $vs;
 }

  $vs =~ s/\d+'d(\d+)/$1/ogi;
  $vs =~ /'/o ? $vs : eval $vs;
}

sub vsubstitute  {
my ($conf, $h, $capt) = @_;

 unless (defined $h->{$capt}) {
  print "-W- String *$capt* is not defined\n" unless $$h{__already_seen}{$capt};
  $$h{__already_seen}{$capt}++;

  $capt;
 } else {
  $h->{$capt}
 }
}


sub closing_re   {
my ($ore, $cre) = @_;

 while (1) {
  return unless $_[2] =~ /(?{my $c = 0})(?:$ore(?{$c=0})|$cre(?{$c=1}))/g; 
  return if $c; 
  closing_re (@_)
 }
}

sub vkw_init  {$_[0]{vkw} = eval q{qr/\b(?:}.join('|', @{$_[0]{vkw}}).q{)\b/o} unless $_[0]{vkw_init_done}; $_[0]{vkw_init_done} = 1}
sub vp_init   {$_[0]{vp}  = eval q{qr/\b(?:}.join('|', @{$_[0]{vp}}) .q{)\b/o} unless $_[0]{vp_init_done};  $_[0]{vp_init_done}  = 1}

sub gate_primitive_instantiation  {}

sub module_udp_instantiation  {
my ($conf, $v) = @_;

 my ($module, $instance, $portmap) = $v =~ /(\w+).*?(\w+)\s*\((.*?)\)\s*;/so;
 print "FILE($$conf{_current_file}) MODULE ($module) INSTANCE ($instance)\n" if $$conf{debug};

 my %mapairs = $portmap =~ /\.(\w+)\s*\((.*?)\)(?=\s*,\s*\.|\s*$)/go;
 my $assindex = 0;
 my %updated_mapairs;
 foreach (keys %mapairs) {
  $mapairs{$_} =~ s/\s//go; 
  
  unless ($mapairs{$_}) {
   $updated_mapairs{$_} = '__open__';

  } elsif ($mapairs{$_} !~ /^\w+$/o) {
   add_verilog_simple_info($conf, "wire  $instance"."_$assindex;");
   add_verilog_simple_info($conf, "assign $instance"."_$assindex = $mapairs{$_};");
   $updated_mapairs{$_} =  "$instance"."_$assindex";

  } else {
   $updated_mapairs{$_} =  $mapairs{$_};
  }

  $assindex++
 }

 $$conf{module}{$$conf{_current_module}}{hierarchy}{$module}{$instance} = {%updated_mapairs};
}


sub vhdl_file   {['?analyzed_vhdl_file:', File::Spec->rel2abs($_[1]), $_[0]->vhdl_parse($_[0]->slurp($_[1]))]}
sub vhdl_string {$_[0]->vhdl_parse($_[1])}

sub vhdl_parse {
my ($this, $data) = @_;

 state $vhdl_parser = do {$this->{vhdl_parser} = LinkedSpec::get_parser('vhdl')};

 $data ? $this->{vhdl_parser}->(\$data) : undef;
}

1;
