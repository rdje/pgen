sub gd_check {
my ($checkinfo, $cursection, $data0, $data1) = @_;

my @gd_msks=();
my $numre=qr/^-?\d+(?:\.\d+)?$/o;

 # Build line identifiers and associate them w/ their line index
 return [@gd_msks] unless exists $checkinfo->{lineids}{$cursection};
 
 my $lineids   = $checkinfo->{lineids}{$cursection};
 my @id_colist = ref($lineids) ? @$lineids : ($lineids);

 my $joinstr = $checkinfo->{joinstring}{$cursection} || '_';
 
 # We need to precisely handle the case where multiple LOF line share the
 # same Primary KEY, i.e, Line ID.
 #
 # When Removing/Adding, All lines w/ the same Primary Key should be remove
 # and not anly one of them
 #
 # We will compare a line between DATA0 et DATA1 ONLY and ONLY if that line 
 # DOES NOT share its lineID, with others.
 #
 my %lid2pathset;
 my %idset_unique;
 my @idx;
 my @data = ($data0, $data1);
 foreach my $dx (0 .. 1) {
  my $indx=0;
  foreach my $lref (@{$data[$dx]}) {
   #print "lref=(@$lref)";
   my @selected_cols = map {$$lref[$_]} @id_colist;
   #print "selected_cols=(", join(" ; ", @selected_cols), ")\n";
   my $idstr = join($joinstr, @selected_cols);
   push @{$idx[$dx]{$idstr}}, $indx; 

   push @{$lid2pathset{$dx}{$idstr}}, $lref;

   # Also make use of this step to uniquify all identifiers into the same big set.
   $idset_unique{$idstr} = 1;

   $indx++
  }
 }

 # split identifiers into *common*, *data0* only and *data1* only subsets.
 my %subset;
 foreach my $curid (keys %idset_unique) {
  if (exists $idx[1]{$curid} && exists $idx[0]{$curid}) {
   $subset{common}{$curid} = 1;
  } elsif (! exists $idx[1]{$curid} && exists $idx[0]{$curid}) {
   $subset{data0}{$curid} = 1;
  } else {
   $subset{data1}{$curid} = 1;
  }
 }

 # Check will be performed only if $cursection has a *numbers* entry
 if (exists $checkinfo->{numbers}{$cursection}) {
  # Only the *common* IDs may have some of their columns checked.
  my @chknum_colist = keys %{$checkinfo->{numbers}{$cursection}};
  foreach my $cid (keys %{$subset{common}}) {

   # A given lineID SHOULD be associated to ONLY one Line in EACH data0/data1
   # for the Check to make sense
   
   # Retrieve the number of corresponding paths for this ID for both sets
   my $bad = 0;
   foreach (0 .. 1) {
     my $cnt = @{$lid2pathset{$_}{$cid}};
     unless ($cnt == 1) {
      print "(a2xhs) -W- LineID *$cid* is **NOT** unique ($cnt) in LOF file #$_\n";
      $bad++
     }
   }

   if ($bad) {
    print "(a2xhs) -I- Ignoring LineID **$cid**\n";
    next
   }
   
   # get the line index of each ID in their respective $data(x).
   my @lindex;
   foreach (0 .. 1) {$lindex[$_] = $idx[$_]{$cid}[0]}
   
   foreach my $col (@chknum_colist) {
    # get the data to be compared
    my $v0 	= $data[0][$lindex[0]][$col];
    my $v1	= $data[1][$lindex[1]][$col];

    # vx should have number format
    next unless $v0 =~ $numre &&  $v1 =~ $numre;

    # take their difference
    my $diff	= ($v1 - $v0);


    if ($checkinfo->{numbers}{$cursection}{$col} !~ /-/) {
     # Check is based on a tolerance (%) of data0 column value(x).
     my $tol	= $checkinfo->{numbers}{$cursection}{$col};
     my $margin	= ($v0 * $tol)/100;

     unless (abs($diff) <= $margin) {
      $gd_msks[$lindex[0]][$col] = $checkinfo->{diff}{$diff > 0 ? 'above' : 'below'}
     }
    } else {
     # binary check, that is, Columns either match or don't match.
     # In this case that the only information that matter;
     if ($diff) {
      $gd_msks[$lindex[0]][$col] = $checkinfo->{diff}{notequal}
     }
    }
   }
  }
 }


 # $subset{data0} lines need to be removed from $data0 in order to get $data1.
 #
 # Of course all lines SHOULD have the same number of columns.
 # for the following instruction to be ok.
 # 
 # Check if the reference data is not empty
 if (exists $$data0[0]) {
  my $maxcol = scalar(@{$$data0[0]}) -1;
  foreach my $lid (keys %{$subset{data0}}) {
   # get the line index in $data0.
   my $lindex = $idx[0]{$lid}[0];

   # All lines having the same LineID will be removed
   #
   # All columns on that line should have a *removed* mask.
   foreach (0 .. $maxcol) {$gd_msks[$lindex][$_] = {}}; # $checkinfo->{diff}{removed}
  }
 }

 # $subset{data1} lines need to be added to $data0 in order to get $data1.
 #
 # Of course all lines SHOULD in both $data0 and $data1 should have the same 
 # number of columns for the following to apply.
 foreach my $lid (keys %{$subset{data1}}) {
  # get the line index in $data1.
  my $lindex = $idx[1]{$lid}[0];

  # Get the array at $lindex in $data1 and push it as a reference
  # into $data0. Added lines will appear at the bottom of $data0.
  push @$data0, [@{$$data1[$lindex]}];

  # All columns on that line should have a *added* mask.
  # All lines having the same LineID will be added
  #
  my $maxindex=@{$$data0[0]} - 1;
  foreach (0 .. $maxindex) {$gd_msks[$#$data0][$_] = []}; # $checkinfo->{diff}{added}
 }

 return [@gd_msks]
}


#
#
# (checkinfo
#  (numbers
#   (sec0 (s_c00) (s_c01 µ01) ...)
#   (sec1 (s_c10) (s_c11 µ01) ...)
#    ...                
#   (secp (s_cp0) (s_cp1 µ01) ...)
#  )
#  
#  (lineids
#   (sec0 l_c00 l_c01 l_c02 ...)
#   (sec1 l_c10 l_c11 l_c12 ...)
#    ...
#   (secm l_cm0 l_cm1 l_cm2 ...)
#  )
#
#  (diff
#   (notequal	"notequal_format")
#   (above	"above_format")
#   (below	"below_format")
#   (added	"added_line_format")
#   (removed	"removed_line_format")
#  )


sub checkinfo {
my ($c_arg) = @_;

my @checkinfo;

 unless(ref($c_arg)) {
  print "(checkinfo) -E- 'c_arg' is not a reference.\n";
  return undef
 }

 foreach (@{$$c_arg[1]}) {
  unless(ref) {
   print "(checkinfo) -E- Badly formatted section.\n";
   return undef
  }

  my ($entryname) = ($$_[0] =~ /(\S+)/);
  push @checkinfo, $entryname => &{'checkinfo_'.$entryname}($_)
 }

 return {@checkinfo}
}

sub checkinfo_numbers {
my ($cn_arg) = @_;

my @checkinfo_numbers;

 unless(ref($cn_arg)) {
  print "(checkinfo-numbers) -E- 'cn_arg' is not a reference.\n";
  return undef
 }

 foreach (@{$$cn_arg[1]}) {
  unless(ref) {
   print "(checkinfo-numbers) -E- Badly formatted.\n";
   return undef
  }

  push @checkinfo_numbers, &checkinfo_numbers_sec($_)
 }

 return {@checkinfo_numbers}
}

sub checkinfo_numbers_sec {
my ($cns_arg) = @_;

my @checkinfo_numbers_sec;

 unless(ref($cns_arg)) {
  print "(checkinfo-numbers-sec) -E- 'cns_arg' is not a reference.\n";
  return undef
 }

 my ($secname) = ($$cns_arg[0] =~ /(\S+)/);
 foreach my $coldesc (@{$$cns_arg[1]}) {
  if(ref($coldesc)) {
   print "(checkinfo-numbers-sec) -E- Badly formatted.\n";
   return undef
  }

  my @splitcoldesc = split(/\s+/, $coldesc);
  unless(@splitcoldesc) {
   print "(checkinfo-numbers-sec) -E- Column description should have one or two numbers provided.\n";
   return undef
  }

  if (@splitcoldesc == 1) {
   push @checkinfo_numbers_sec, $splitcoldesc[0] => undef
  } else {
   push @checkinfo_numbers_sec, @splitcoldesc[0 .. 1]
  }
 }

 return $secname => {@checkinfo_numbers_sec}
}

sub checkinfo_lineids {
my ($cl_arg) = @_;

my @checkinfo_lineids;

 unless(ref($cl_arg)) {
  print "(checkinfo-lineids) -E- 'cl_arg' is not a reference.\n";
  return undef
 }

 foreach (@{$$cl_arg[1]}) {
  if(ref) {
   print "(checkinfo-lineids) -E- Badly formatted.\n";
   return undef
  }

  my @splitline = split(/\s+/);
  push @checkinfo_lineids, $splitline[0] => [@splitline[1 .. $#splitline]]
 }

 return {@checkinfo_lineids}
}

sub checkinfo_joinstring {
 return &simple_hash(shift @_)
}

sub checkinfo_diff {
my ($cd_arg) = @_;

my @checkinfo_diff;

 unless(ref($cd_arg)) {
  print "(checkinfo-diff) -E- 'cl_arg' is not a reference.\n";
  return undef
 }

 foreach (@{$$cd_arg[1]}) {
  if(ref) {
    push @checkinfo_diff, &checkinfo_addrem($_)
  } else {
   my ($formatid) = /(\S+)/;
   my @formatval = map {s/"//g; $_} /"(.+?)"/g;

   if (@formatval == 1) {
    push @checkinfo_diff, $formatid => $formatval[0]
   } else {
    push @checkinfo_diff, $formatid => [@formatval]
   }
  }
 }

 return {@checkinfo_diff}
}

sub checkinfo_addrem {
my ($ca_arg) = @_;

my @mapout;

 unless(ref($ca_arg)) {
  print "(checkinfo_addrem) -E- 'ca_arg' is not a reference.\n";
  return undef
 }

 my ($levelname) = ($$ca_arg[0] =~ /(\S+)/);
 foreach (@{$$ca_arg[1]}) {
  if(ref) {
   print "(checkinfo-$levelname) -E- Badly formatted.\n";
   return undef
  }

  my ($secid, $scriptname) = /(\S+)/g;

  push @mapout, $secid => $scriptname
 }

 return $levelname => {@mapout}
}

sub checkinfo_aio_labels {
my ($ca_arg) = @_;

my @checkinfo_aio;

 unless(ref($ca_arg)) {
  print "(checkinfo-aio) -E- 'ca_arg' is not a reference.\n";
  return undef
 }

 foreach (@{$$ca_arg[1]}) {
  unless(ref) {
   print "(checkinfo-aio) -E- Badly formatted.\n";
   return undef
  }

  push @checkinfo_aio, &checkinfo_aio_sec($_)
 }

 return {@checkinfo_aio}
}

sub checkinfo_aio_sec {
my ($cas_arg) = @_;

my @checkinfo_aio_sec;

 unless(ref($cas_arg)) {
  print "(checkinfo-aio-sec) -E- 'cas_arg' is not a reference.\n";
  return undef
 }

 my ($secname) = ($$cas_arg[0] =~ /(\S+)/);
 foreach my $coldesc (@{$$cas_arg[1]}) {
  if(ref($coldesc)) {
   print "(checkinfo-aio-sec) -E- Badly formatted.\n";
   return undef
  }

  my @splitcoldesc = split(/\s+/, $coldesc);
  unless(@splitcoldesc == 2) {
   print "(checkinfo-aio-sec) -E- Column description should have exactly 2 fields, with no space in their value.\n";
   return undef
  }

  push @checkinfo_aio_sec, @splitcoldesc[0 .. 1]
 }

 return $secname => {@checkinfo_aio_sec}
}

sub checkinfo_uniquify {
my ($cu_arg) = @_;
 
my %uniquify_info;

 unless(ref($cu_arg)) {
  print "(a2xhs)(checkinfo-uniquify) -E- 'cu_arg' argument is not a reference.\n";
  exit 1
 }

 unless(ref($$cu_arg[1])) {
   print "(a2xhs)(checkinfo-uniquify) -E0- Wrongly formatted description.\n";
   exit 1
 }

 foreach my $secdes (@{$$cu_arg[1]}) {
  if(ref($secdes)) {
   print "(a2xhs)(checkinfo-uniquify) -E1- Wrongly formatted description.\n";
   exit 1
  }

  my ($secname) = ($secdes =~ /(\S+)/);
  my @EVAL0 = ($secdes =~ /"(.*?)(?!<\\)"/g);
  my @EVAL  = map {s/\\(\(|\)|\\|")/$1/g; $_} @EVAL0; 

  if ($secname eq 'DEFAULT' && length(@EVAL) != 1) {
   print "(a2xhs)(checkinfo-uniquify) -E- 'DEFAULT' should have exactly *one* RE.\n";
   exit 1
  }

  $uniquify_info{$secname} = shift @EVAL
 }

 return {%uniquify_info}

}
1
