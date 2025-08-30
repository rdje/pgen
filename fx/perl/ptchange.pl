#!/apps/perl/5.8.3/bin/perl -w

# usage: ptchange.pl <InputPtFile> <OuputClockFile> [OutputPtFile]

my $delimiter		= qr/\s\\\n\s|\s/o;
my $spep_def		= qr#\{?[\w/\[\]\.]+\}?#o; 
my $spep_spec		= qr#$delimiter?$spep_def#o; 
my $spep_list		= qr#$spep_spec+#o; 
my $spep_obj_list	= qr#\{$spep_list\}#o; 
my $get_cmds		= qr#get_pins|get_ports|get_clocks#o; 
my $spep_obj_list_spec	= qr#\[$get_cmds $delimiter $spep_obj_list\]#xo; 
my $from_statement	= qr#$delimiter -from $delimiter $spep_obj_list_spec#xo;
my $to_statement	= qr#$delimiter -to $delimiter $spep_obj_list_spec#xo;
my $through_statement	= qr#$delimiter -through     $delimiter $spep_obj_list_spec#xo;
my $set_false_path	= qr#set_false_path#o;
my $misc_op		= qr#$delimiter-(?:setup|hold|rise|fall|reset_path)#o;
my $sfp_re	    	= qr#$set_false_path
	  	             $misc_op*
			     $from_statement?
			     $through_statement*
			     $to_statement?
	  	            #mxo;

my $ftt_statement	= qr#$delimiter -(?:from|to|through) $delimiter $spep_obj_list_spec#xo;
my $sfp_start_capture	= qr#$set_false_path $misc_op*#xo;
my $ftt_start_capture	= qr#$delimiter -(?:from|to|through) $delimiter#xo;
my $spep_obj_list_spec_start_capture	= qr#\[$get_cmds $delimiter#xo; 

-s $ARGV[0] || die "[ptchange] -E- Input file $ARGV[0] doesn't exists !";
my $istr=qx(cat $ARGV[0]);

# Removing leading './' character from all pins.
# After this step ports will be those w/o slashe(s)
# in their name.
$istr =~ s/\.\///g;

# Any pins w/o' slashe(s) in their names should be refered to as ports.
$istr =~ s/(\[\s*)get_pins(\s+[\\\{][^\/]+?\}\])/$1get_ports$2/gi;



#--------------------------!!!!!!!!!!!!!!!!!!!!-----------------------
# THESE ARE TEMPORARY CHANGES, TO BE REMOVED WHEN DEALING ONLY WITH
# ORIGINAL ILMS
#--------------------------!!!!!!!!!!!!!!!!!!!!-----------------------
# Need for UMA ILM func mode
$istr =~ s/omap_MSCLKDSPCTS\b/CLK/ig;

# Needed for OMAP ILM HOM mode
# $istr =~ s/(\[\s*)get_clocks(\s+\{?MSPENSTROBEP\b\}?)/$1get_ports$2/ig;
# $istr =~ s/(\[\s*)get_clocks(\s+\{?MSPENSTROBES\b\}?)/$1get_ports$2/ig;
# $istr =~ s/(\[\s*)get_clocks(\s+\{?MSPENSTROBE2S\b\}?)/$1get_ports$2/ig;
# $istr =~ s/(\[\s*)get_clocks(\s+\{?PIDDLDQS\b\}?)/$1get_ports$2/ig;
# $istr =~ s/(\[\s*)get_clocks(\s+\{?PIDDUDQS\b\}?)/$1get_ports$2/ig;

# Needed for OMAP SAM_SYNCSCAL mode
$istr =~ s/^#(\s*create_)/$1/igm; # un-commenting commented create_* commmands
$istr =~ s/(\[\s*get_clocks\s+\{?)shared_rhea_pstrobe\b/$1MSPENSTROBES/ig;
$istr =~ s/(\[\s*get_clocks\s+\{?)private_rhea_pstrobe\b/$1MSPENSTROBEP/ig;
$istr =~ s/(\[\s*get_clocks\s+\{?)shared_rhea_pstrobe2\b/$1MSPENSTROBE2S/ig;
$istr =~ s/(\[\s*get_clocks\s+\{?)fclk_pad\b/$1PIFLCLKBFIN/ig;



#----- The following change it a huge one -----
# Extracting set_false_path commands and also remenbering start/end pairs
# for post-processing purposes.
my @spos = ();
push @spos, 0;
while($istr =~ /$sfp_re/g) {
 push @spos, $-[0], $+[0];
 push @m_sfp_list, $&
}

push @spos, length($istr);


# Identify wrongly formatted set_false_path commands
# and preparing stuffs for correcting those problems.
my $i=0;
my @new_sfp_list;
foreach my $sfp (@m_sfp_list) {
 #----------------------------------------------------------
 # Capturing the start of the set_false_path statement
 # including all $misc_op stuffs
 #----------------------------------------------------------
 my ($mysfp_start)=($sfp =~ /($sfp_start_capture)/m);

 my @all_ftt_objlist=($sfp =~ /($ftt_statement)/mg);
 my @concat_all_ftt=();
 foreach my $ftt (@all_ftt_objlist) {
  my $j=0;
  my ($myftt_start)=($ftt =~ /($ftt_start_capture)/m);

  my ($ftt_spep_obj_list_spec) = ($ftt =~ /($spep_obj_list_spec)/m);
  my ($ftt_sols_start)=($ftt_spep_obj_list_spec =~ /($spep_obj_list_spec_start_capture)/m);
  my ($ftt_spep_obj_list)=($ftt_spep_obj_list_spec =~ /($spep_obj_list)/m);
  
  my @new_ftt=();

  $ftt_spep_obj_list =~ s/\{|\}//g;
  my @objlist = ($ftt_spep_obj_list =~ /($spep_def)/mg);

  my $w_slash = grep /\//, @objlist;
  if($w_slash != scalar(@objlist) && $w_slash != 0) {
   # Changes need to be applied here
   my @w_slash =grep /\//, @objlist;
   my @wo_slash=grep !/\//, @objlist;
   
   #print "Preparing Changes for [SFP#$i][FTT#$j]\n";
   
   push @new_ftt, $myftt_start, [$ftt_sols_start, [@w_slash], [@wo_slash]];
   push @concat_all_ftt, [@new_ftt];
  } else {
   push @concat_all_ftt, $ftt;
  }
 
  $j++
 }

 push @new_sfp_list, [$mysfp_start, @concat_all_ftt];
 $i++
}


# Correcting not correctly formatted set_false_paths commands
# In some cases a given set_false_path command will be split into
# several others.
my $c=0;
my @last_sfp_list=map {&ftt_setup($_, $c++); &allcombs($_)} @new_sfp_list;


my $outputfile_name=(defined($ARGV[2]) ? $ARGV[2] : "ptchange-of.txt");
open(OUTFILE, "> $outputfile_name") || die "Can't open '$outputfile_name' for writing.";
my $new_filecontent = &sfp_reinsert(\@spos, \@last_sfp_list, $istr);
print OUTFILE $new_filecontent;
close(OUTFILE);

# There is one other thing to do before quitting
#
# We need to extract all *get_clocks* statements argument and drive a '.tcl' script to be sourced
# just before applying the modified constraint file.
# The purpose of this file is to create most, if not all, clocks needed at the boundary of the design
my $getclocks_statement = qr#get_clocks$delimiter\{?\w+\}?#o;
my @all_getclocks_statements = ($new_filecontent =~ /($getclocks_statement)/img);
my %uniquify_clocklist;
 foreach (map { /get_clocks/g; /(\w+)/g; $1} @all_getclocks_statements) {
   $uniquify_clocklist{$_} = 1 unless exists $uniquify_clocklist{$_}; 
 }

my  @uniquify_clocklist = keys %uniquify_clocklist;

if (scalar(@uniquify_clocklist)) {
 # Generating the Tcl script to be sourced later.
 open(TCLSCRIPT, "> $ARGV[1]") || die "Can't open '$ARGV[1]' for writing,";
 print TCLSCRIPT <<END_OF_SCRIPT;
foreach mypotential_clk [list @uniquify_clocklist] {
 if {[get_ports -q \$mypotential_clk -fi "direction == in"] != ""} {
  echo "(ilm2xls) -I- Defining clock '\$mypotential_clk'"
  create_clock -p 3.56 \$mypotential_clk
  set_propagated_clock \$mypotential_clk
 }
}
END_OF_SCRIPT
 
 close(TCLSCRIPT);
}

# Done. :)



sub	allcombs {
my ($aref, $index)=@_;
my @combs;
 
 $index = 0 unless defined($index);

 my $lastindex=scalar(@$aref) -1;
 my $is_arrayref=(ref($aref->[$index]) eq "ARRAY");

 if($index+1 <= $lastindex) {
   my $lowlevel_combs=&allcombs($aref, $index+1);
   if($is_arrayref) {
    foreach my $val (@{$aref->[$index]}) {
      foreach (@$lowlevel_combs) {
        push @combs, $val.$_;
      }
    }
   } else {
    foreach (@$lowlevel_combs) {
      push @combs, ($aref->[$index]).$_;
    }
   }
 } else {
  # Recursion stops here.
  push @combs, ($is_arrayref ? @{$aref->[$index]} : $aref->[$index]);
 }

 return [@combs]
}

sub ftt_setup {
 my ($concat, $index)=@_;

 #print "[ftt_setup] Processing SFP#$index\n";
 my $i=0;
 foreach my $ent (@$concat) {
  #print "[ftt_setup] Looping on SFP#$index entry#$i\n";
  if(ref($ent) eq "ARRAY") {
   #print "[ftt_setup] Splitting one object list for SFP#$index\n";
   $concat->[$i] = [
    		    $ent->[0].$ent->[1][0].'{'.join(" \\\n ", map { /\[|\]/ ? "{$_}": $_} @{$ent->[1][1]}).'}'.']',
		    # For Ports I should take care of replacing the *get_pins* by *get_ports* if any.
    		    $ent->[0].($ent->[1][0] =~ s/get_pins/get_ports/i, $ent->[1][0]).
		    		'{'.join(" \\\n ", map { /\[|\]/ ? "{$_}": $_} @{$ent->[1][2]}).'}'.']',
   		   ]
  }

  $i++
 }
}

sub	sfp_reinsert {
my ($spos, $last_sfp_list, $orig_string)=@_;

 my %spos=(@$spos);
 my $new_string="";

 foreach my $start (sort {$a <=> $b} keys %spos) {
  $new_string .= substr($orig_string, $start, $spos{$start}-$start);

  print "\n" unless ($spos{$start}-$start);

  my $nxt_sfp=shift @$last_sfp_list;
  next unless $nxt_sfp;

  $new_string .= join("\n", @$nxt_sfp);
 }

 return $new_string;
}
