#===================================================================
# Copyright (c) 2005-2007 Richard DJE. All rights reserved.
#
# This Perl module is free software, you may redistribute it and/or 
# modify it under the same terms as Perl itself.
#===================================================================
package TableScript;

use File::Glob ':glob';

use HUtils;

my @script_list = bsd_glob(q({).join(',', map {"$_/*.ts"} @{Global->search_path}).q(}), GLOB_BRACE | GLOB_TILDE);

my @table_scripts;
my %table_scripts;
foreach my $ts (@script_list) {
 push @table_scripts, map {$$_[0] => $$_[1]} @{Lispish::multi($ts)};
}

%table_scripts = @table_scripts;

sub node_exec ($$);

sub Run     {
my ($conf, $tinfo, $script) = @_;

 die "(TableScript::Run) -E- Undefined script *$script*," unless defined $table_scripts{$script};

 # Initializing the current script <n>ame<s>pace
 # Reference values are not imported
 foreach (keys %$conf) {$conf->{_script}{namespace}{$_} = $conf->{$_} unless ref $conf->{$_}}

 # Setting the current script name for later use
 unshift @{$conf->{_script}{current}{script}}, $script;

 # Executing $table_scripts{$script}
 $conf->{_tinfo}  = $tinfo;
 node_exec $conf, $_ foreach (@{$table_scripts{$script}});
 my $run_table    = $conf->{_ntinfo};
 delete $conf->{_ntinfo};

 shift @{$conf->{_script}{current}{script}};


 return $run_table
}

sub each_exec {
my ($conf, $args) = @_;

 my $crow=0;
 $conf->{_ntinfo} = [];
 foreach (@{$conf->{_tinfo}}) {
  # One iteration per row

  $conf->{_script}{namespace}{ROW}    = $crow;
  $conf->{_script}{namespace}{MAXCOL} = $#$_;
  $conf->{_lineout}                   = {};
  $conf->{_info}                      = $_;
  $conf->{_store}                     = -1;

  node_exec $conf, $_ foreach (@$args);

  my @negative = sort {$b <=> $a} grep {$_ <  0} keys %{$conf->{_lineout}}; # store
  my @positive = sort {$a <=> $b} grep {$_ >= 0} keys %{$conf->{_lineout}}; # write.*

  $conf->{_ntinfo}[$crow] = [map {$conf->{_lineout}{$_}} @positive, @negative];
  $crow++
 }

 delete $conf->{_lineout};
 delete $conf->{_info};
 delete $conf->{_store};
}

sub unshift_exec {my ($conf, $args) = @_; unshift @{$conf->{_tinfo}}, [map {node_exec $conf, $_} @$args]}
sub push_exec    {my ($conf, $args) = @_; push    @{$conf->{_tinfo}}, [map {node_exec $conf, $_} @$args]}
sub insert_exec  {my ($conf, $args) = @_; splice  @{$conf->{_tinfo}}, node_exec ($conf, $$args[0]), 0, [map {node_exec $conf, $_} @$args[1 .. $#$args]]}
#sub shift_exec   {my ($conf, $args) = @_; shift   @{$conf->{_tinfo}}}
#sub pop_exec     {my ($conf, $args) = @_; pop     @{$conf->{_tinfo}}}

sub call_exec {
my ($conf, $args) = @_;

 my $script = node_exec $conf, $args->[0];
 die "(TableScript)(call_exec) -E- Undefined script *$script*,"   unless defined $table_scripts{$script};
 die "(TableScript)(call_exec) -E- Recursivity is not supported," unless $conf->{_script}{current}{script}[0] ne $script;

 my $ret;
 unshift @{$conf->{_script}{current}{script}}, $script;
 $ret = node_exec $conf, $_ foreach (@{$table_scripts{$script}});
 shift @{$conf->{_script}{current}{script}};

 return $ret
}

sub node_exec ($$) {
my ($conf, $args) = @_;

 return $conf->{_script}{last_output} = $args unless ref $args;

 my $cmdname    = node_exec $conf, $$args[0];
 unshift @{$conf->{_script}{current}{cmd}}, $cmdname;

 $conf->{_script}{last_output} = do {
  if ($cmdname =~ /^(?:then|else|sequence|block|do)$/oi) {
   my $cmd_o; $cmd_o = node_exec $conf, $_ foreach @{$$args[1]}; $cmd_o;
  } else {
   &{"${cmdname}_exec"}($conf, $$args[1])
  }
 };

 shift @{$conf->{_script}{current}{cmd}};
 

 #print uc($cmdname), "-> ($conf->{_script}{last_output})\n";
 return $conf->{_script}{last_output}
}

sub if_exec {
my ($conf, $args) = @_;

 if (ref $$args[0] ? node_exec $conf, $$args[0]  : eval_exec ($conf, [$$args[0]])) {
   die "(TableScript)(if_exec) -E- Undefined 'then' statement error," unless defined $$args[1];
 
   node_exec $conf, $$args[1];
 } elsif(exists $$args[2]) {
   die "(TableScript)(if_exec) -W- Undefined 'else' statement (not a reference)," unless defined $$args[2];

   node_exec $conf, $$args[2]
 }
}

sub unless_exec {
my ($conf, $args) = @_;

 unless (ref $$args[0] ? node_exec $conf, $$args[0]  : eval_exec ($conf, [$$args[0]])) {
   die "(TableScript)(unless_exec) -E- Undefined 'then' statement error," unless defined $$args[1];
  
   node_exec $conf, $$args[1];
 } elsif(exists $$args[2]) {
   die "(TableScript)(unless_exec) -W- Undefined 'else' statement (not a reference)," unless defined $$args[2];

   node_exec $conf, $$args[2]
 }
}

sub write_exec {
my ($conf, $args) = @_;
 my ($column2write, $column2read_or_val, $formatname) = map {node_exec $conf, $_} @$args[0 .. 2];

 #print "write($column2write, $column2read_or_val, $formatname)\n";
# die "(TableScript)(write_exec) -E- Unknown format '$formatname'," unless $conf->{format}{$formatname};

 $column2write        = ref $args->[0] ? $column2write : get_index ($conf, $column2write);

 my $cscript = $conf->{_script}{current}{script}[0];
 # Retrieving option, if any
 my $option = @$args[3 .. $#$args] ? {map {$$_[0] => $$_[1][0]} @$args[3 .. $#$args]} : undef; 

 my $value = do {
	unless (ref $args->[1]) {
          my $column2read      = get_index ($conf, $column2read_or_val);
          my $info_column2read = $conf->{_info}[$column2read];

          die "(TableScript)(write_exec) -W- Value at column '$column2read' not defined ($cscript)," unless defined $info_column2read;
          $info_column2read;
       } else {$column2read_or_val}
 };

 my ($link, $string)       = $value =~ m#((?:internal|external):.+!.+|https?://.+?)@(.+)#o;
 unless($option->{nosubst}) {$value =~ s/^\w+://o unless $value =~ /^(?:internal|external|https?):/o}

 $conf->{_lineout}{$column2write} = {url=> $link, value=> $link ? $string : $value, format=> $formatname}
}

sub write_force_exec {
my ($conf, $args) = @_;

 my ($excel_col, $excel_val, $formatname) = map {node_exec $conf, $_} @$args;
     $excel_col                           = ref $args->[0] ? $excel_col : get_index ($conf, $excel_col);

 $conf->{_lineout}{$excel_col} = {from=>undef, value=>$excel_val, format=>$formatname};
}

sub store_exec {
my ($conf, $args) = @_;

 my $col2read = get_index ($conf, node_exec $conf, $args->[0]);
 die "(TableScript)(store_exec) -W- Column '$col2read' does not exist," unless exists $conf->{_info}[$col2read];
 #print "store_exec: Hello there, <$col2read> --> ($conf->{_info}[$col2read])\n";
 $conf->{_lineout}{$conf->{_store}--} = $conf->{_info}[$col2read];
}

sub linemap_exec {my ($conf, $args) = @_; $conf->{script_index_mapping} = node_exec $conf, $args->[0]}

sub match_exec {
my ($conf, $args) = @_;

 my ($info_col, $regex) = map {node_exec $conf, $_} @$args[0 .. $#$args];
     $info_col          = get_index($conf, $info_col);

 die "(TableScript)(match_exec) -W- Column '$info_col' does not exist," unless exists $conf->{_info}[$info_col];

 $conf->{_info}[$info_col] =~ /$regex/
}

sub eval_exec {
my ($conf, $args) = @_;

 my $eval_arg = node_exec $conf, $$args[0];
 my %option = map {$$_[0] => $$_[1][0]} grep {ref} @$args[1 .. $#$args];

 my $cscript = $conf->{_script}{current}{script}[0];
 #$option{verbose} = 1;
 $option{verbose} && print "=========== Initial =========> EVAL_ARG($eval_arg) ($cscript)\n";
 my @allvars = $eval_arg =~ /\$(\w+)/go;
 foreach my $cvar (@allvars) {
  die "(TableScript)(eval_exec) -E- Undefined variable '$cvar'," unless exists $$conf{_script}{namespace}{$cvar};

  $eval_arg =~ s{\$$cvar}{defined($$conf{_script}{namespace}{$cvar}) ? $$conf{_script}{namespace}{$cvar} : "\$$cvar"}ge;
  $option{verbose} && print "=========== 0 =========> EVAL_ARG($eval_arg)\n";
 }

 $eval_arg =~ s{\@\((\w+)\)}{$conf->{_info}[get_index($conf, $1)]}goe;
 $option{verbose} && print "=========== 1 =========> EVAL_ARG($eval_arg)\n";

 eval $eval_arg
}

sub foreach_exec {
my ($conf, $args) = @_;

 my ($loopvarname) = node_exec $conf, $$args[0];
 my $ret;
 my $cscript = $conf->{_script}{current}{script}[0];
 foreach (0 .. $#{$conf->{_info}}) {
  $conf->{_script}{namespace}{$loopvarname} = $_;
  $ret = node_exec $conf, $$args[1];
 }

 # Remove that variable from the namespace hash table
 delete $conf->{_script}{namespace}{$loopvarname};

 return $ret
}

sub casematch_exec {
my ($conf, $args) = @_;

 my $ccmd    = $conf->{_script}{current}{cmd}[0];

 die "(TableScript)(${ccmd}_exec) -E- At least three arguments are required," unless @$args >= 3;

 my $info_val = $conf->{_info}[my $col = get_index($conf, node_exec $conf, $args->[0])];
 die "(TableScript)(${ccmd}_exec) -E- Undefined value at column #$col," unless defined $info_val;

 my $found = 0;
 my $ret;
 my $idx= 1;
 for (; $idx < $#$args; $idx += 2) {
  my $re = node_exec $conf, $args->[$idx];
  if ($info_val =~ /$re/) {
   $found = 1;
   $ret = node_exec $conf, $args->[$idx + 1];
   last
  }
 }

 return do {
        if    ($found)          {$ret} 
	elsif ($idx == $#$args) {node_exec $conf, $args->[$idx]}
	else  {
	 #print "(TableScript)(${ccmd}_exec) -W- Missing default expression.\n";
         $conf->{_script}{last_output}
	}
 }
}

sub caseval_exec {
my ($conf, $args) = @_;

 my $ccmd    = $conf->{_script}{current}{cmd}[0];

 #die "(TableScript)(${ccmd}_exec) -E- Even number of arguments required," if @$args % 2;

 my $true = 0;
 my $ret;
 my $idx = 0;
 for (; $idx < $#$args; $idx += 2) {
  if (eval_exec($conf, [node_exec $conf, $args->[$idx]])) {
   $true = 1;
   $ret = node_exec $conf, $args->[$idx + 1];
   last
  }
 }

 return do {
        if    ($true)                      {$ret} 
	elsif ($idx == $#$args) {node_exec $conf, $args->[$idx]}
	else  {
	 #print "(TableScript)(${ccmd}_exec) -W- Missing default expression.\n";
         $conf->{_script}{last_output}
	}
 }
}


sub join_exec {
my ($conf, $args) = @_;

 join(node_exec ($conf, $args->[0]), map {node_exec $conf, $_} @$args[1 .. $#$args]); 
}

sub concat_exec {my ($conf, $args) = @_; join_exec($conf, ["", @$args])}

sub col_exec {
my ($conf, $args) = @_;

 my $colidx = get_index($conf, node_exec $conf, $args->[0]);
 warn "(TableScript)($conf->{_script}{current}{cmd}[0]_exec) -W- Undefined value at column #$colidx" unless defined $conf->{_info}[$colidx];

 $conf->{_info}[$colidx]
}

sub expr_exec {
my ($conf, $args) = @_;

 eval_exec($conf, [join " ", map {node_exec $conf, $_} @$args]);
}

sub subst_exec {
my ($conf, $args) = @_;

 my ($regex, $replacement, $target) = map {node_exec $conf, $_} @$args[0 .. 2];

 my $option = defined $args->[3] ? {map {$$_[0] => $$_[1][0]} @$args[3 .. $#$args]} : undef;

 #print "<$target> =~ s{$regex}{$replacement}\n";
 eval '$target'." =~ s{$regex}{$replacement}".($option->{global} ? 'g' : '');

 $target
}

sub gt_exec      {my ($conf, $args) = @_; eval join " > ",  map {node_exec $conf, $_} @$args[0,1]}
sub lt_exec      {my ($conf, $args) = @_; eval join " < ",  map {node_exec $conf, $_} @$args[0,1]}
sub ge_exec      {my ($conf, $args) = @_; eval join " >= ", map {node_exec $conf, $_} @$args[0,1]}
sub le_exec      {my ($conf, $args) = @_; eval join " <= ", map {node_exec $conf, $_} @$args[0,1]}
sub equal_exec   {my ($conf, $args) = @_; eval join " == ", map {node_exec $conf, $_} @$args[0,1]}
sub eq_exec      {my ($conf, $args) = @_; eval join " eq ", map {qq/"$_"/} map {node_exec $conf, $_} @$args[0,1]}
sub ne_exec      {my ($conf, $args) = @_; eval join " ne ", map {qq/"$_"/} map {node_exec $conf, $_} @$args[0,1]}
sub add_exec     {my ($conf, $args) = @_; eval join " + ",  map {node_exec $conf, $_} @$args}
sub sub_exec     {my ($conf, $args) = @_; eval join " - ",  map {node_exec $conf, $_} @$args}
sub mod_exec     {my ($conf, $args) = @_; eval join " % ",  map {node_exec $conf, $_} @$args[0,1]}

sub uc_exec      {my ($conf, $args) = @_; uc node_exec $conf, $args->[0]}
sub lc_exec      {my ($conf, $args) = @_; lc node_exec $conf, $args->[0]}
sub defined_exec {my ($conf, $args) = @_; defined $conf->{_info}[get_index($conf, node_exec $conf, $args->[0])]}

sub get_exec     {my ($conf, $args) = @_; my @arg_list = map {node_exec $conf, $_} @$args; 
	  my $getv = HUtils::avv_get($conf, ['_script', 'namespace', @arg_list]); 

	  #print join " ", "get_exec: ", @arg_list, " -> (" .(defined $getv ? $getv : ""). ")\n";

	  defined $getv ? $getv : ""}

sub set_exec     {my ($conf, $args) = @_; my @arg_list = map {node_exec $conf, $_} @$args;
	my $value = $arg_list[-1]; HUtils::avv_set($conf, ['_script', 'namespace', @arg_list[0 .. $#arg_list-1]], $value); 

	#print join " ", "set_exec: ",@arg_list, "\n";

	$value}

sub sprintf_exec {my ($conf, $args) = @_; sprintf node_exec ($conf, $args->[0]), map {node_exec $conf, $_} @$args[1 .. $#args]}
sub print_exec   {my ($conf, $args) = @_; print node_exec $conf, $args->[0]; $conf->{_script}{last_output}}
sub exit_exec    {my ($conf, $args) = @_; exit}

sub http_exec    {
my ($conf, $args) = @_;

 my ($filename, $label) = map {node_exec $conf, $_} @$args[0,1];

 PPlugin->exec('httplink', $filename).'@'.$label
}

sub get_index {
my ($conf, $data) = @_;

 my $ccmd    = $conf->{_script}{current}{cmd}[0];
 my $cscript = $conf->{_script}{current}{script}[0];


 if ($data =~ /^\$(\w+)$/o) {
  die "(TableScript)(${ccmd}_exec) -E- Sorry but variable '$1' is not defined," unless exists $conf->{_script}{namespace}{$1};

  $data = $conf->{_script}{namespace}{$1};
 } elsif ($data =~ /^([[:alpha:]]\w+)$/o) {
  $data = $conf->{script_index_info}{$conf->{script_index_mapping}}{$1};
  unless (defined $data) {
   die "(TableScript)(${ccmd}_exec) -E- Can't find field '$1' "                   .
                                    "in mapping '$conf->{script_index_mapping}' " .
                                    "for script '$cscript',";
  }
 } elsif ($data !~ /^-?\d+$/o) {
  die "(TableScript)(${ccmd}_exec) -E- Column should either be an integer, an identifier or be a '\$' sign followed by an identifier,";
 }

 $data
}

1;
