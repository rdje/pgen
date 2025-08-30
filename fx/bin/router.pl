#! perl -w -Iperl

require Lispish;

my $confile = qx(cat router.conf);
$confile =~ s/#.*//g;
my $conf = Lispish::HConf(\$confile);

my %routing_area;
my $WIDTH = 41;
my $HEIGHT = 51; 
foreach my $row (0 .. $HEIGHT-1) {
 foreach my $col (0 .. $WIDTH-1) {
  $routing_area{"$row/$col"} = {row=>$row, col=>$col, busy=>0}
 }
}

foreach my $row (0 .. $HEIGHT-1) {
 foreach my $col (0 .. $WIDTH-1) {
  my $cpoint = $routing_area{"$row/$col"};

  $$cpoint{n}  = $$cpoint{row} - 1 >= 0       ? $routing_area{($$cpoint{row} - 1).'/'. $$cpoint{col}     } : undef;
  $$cpoint{s}  = $$cpoint{row} + 1 <  $HEIGHT ? $routing_area{($$cpoint{row} + 1).'/'. $$cpoint{col}     } : undef;
  $$cpoint{e}  = $$cpoint{col} + 1 <  $WIDTH  ? $routing_area{ $$cpoint{row}     .'/'.($$cpoint{col} + 1)} : undef;
  $$cpoint{w}  = $$cpoint{col} - 1 >= 0       ? $routing_area{ $$cpoint{row}     .'/'.($$cpoint{col} - 1)} : undef;

  $$cpoint{ne} = ($$cpoint{row} - 1 >= 0      ) &&  ($$cpoint{col} + 1 <  $WIDTH) ? $routing_area{($$cpoint{row} - 1).'/'. ($$cpoint{col} + 1)} : undef;
  $$cpoint{nw} = ($$cpoint{row} - 1 >= 0      ) &&  ($$cpoint{col} - 1 >= 0     ) ? $routing_area{($$cpoint{row} - 1).'/'. ($$cpoint{col} - 1)} : undef;
  $$cpoint{se} = ($$cpoint{row} + 1 <  $HEIGHT) &&  ($$cpoint{col} + 1 <  $WIDTH) ? $routing_area{($$cpoint{row} + 1).'/'. ($$cpoint{col} + 1)} : undef;
  $$cpoint{sw} = ($$cpoint{row} + 1 <  $HEIGHT) &&  ($$cpoint{col} - 1 >= 0     ) ? $routing_area{($$cpoint{row} + 1).'/'. ($$cpoint{col} - 1)} : undef;
 }
}

foreach (0 .. $HEIGHT-1) {
 point($_, 20)->{busy} = 1;
}

point(30, 18)->{busy} = 1;
point(30, 19)->{busy} = 1;
point(31, 18)->{busy} = 1;
point(32, 18)->{busy} = 1;
point(33, 18)->{busy} = 1;
point(34, 18)->{busy} = 1;
point(35, 18)->{busy} = 1;
point(36, 18)->{busy} = 1;
point(37, 18)->{busy} = 1;
point(38, 18)->{busy} = 1;
point(31, 20)->{busy} = 0;
point(48, 21)->{busy} = 1;
point(48, 22)->{busy} = 1;
point(49, 21)->{busy} = 1;

route(point(27, 19), point(49, 22));
point(27, 19)->{busy} = 2;
point(49, 22)->{busy} = 2;

route_display();

sub point {
my ($r, $c) = @_;

 $routing_area{$r.'/'.$c}
}

sub next_direction {
my ($spoint, $epoint) =  @_;

 my $r_diff = $$spoint{row} - $$epoint{row};
 my $c_diff = $$spoint{col} - $$epoint{col};

 my $r_sens  = $r_diff > 0 ? -1 : ($r_diff < 0 ? 1 : 0);
 my $c_sens  = $c_diff > 0 ? -1 : ($c_diff < 0 ? 1 : 0);

 return 'done' if $r_sens ==  0 && $c_sens ==  0;
 return 'e'    if $r_sens ==  0 && $c_sens ==  1;
 return 'w'    if $r_sens ==  0 && $c_sens == -1;
 return 's'    if $r_sens ==  1 && $c_sens ==  0;
 return 'se'   if $r_sens ==  1 && $c_sens ==  1;
 return 'sw'   if $r_sens ==  1 && $c_sens == -1;
 return 'n'    if $r_sens == -1 && $c_sens ==  0;
 return 'ne'   if $r_sens == -1 && $c_sens ==  1;
 return 'nw'   if $r_sens == -1 && $c_sens == -1;
}

sub route {
my ($spoint, $epoint, $table_type) =  @_;

 return undef unless $spoint;

 $table_type = defined($table_type) ? $table_type : 1;

 my $next_dir = next_direction($spoint, $epoint);
 if($next_dir eq 'done') {
  $$epoint{busy} = 2;

  return 1
 }

 my $alter_idx = -1;
 foreach my $alternative (@{$$conf{$table_type ? 'full_alternatives' : 'limited_alternatives'}{$next_dir}}) {
  ++$alter_idx;

  $table_type = length($next_dir) == 2 && $alter_idx >= 3 ? 0 : $table_type;

  unless (defined $$spoint{$alternative}) {
   print "($table_type)(idx=$alter_idx) next_dir($next_dir)(preferred-alternative)($alternative) of ($$spoint{row}-$$spoint{col}) is OUTSIDE AREA LIMITs, Skipping\n";
   next
  }

  unless ($$spoint{$alternative}{busy} == 1 || $$spoint{$alternative}{busy} == 3) {
   print "($table_type)(idx=$alter_idx) Recursively calling *route* on next_dir($next_dir)(preferred-alternative)($alternative) of ($$spoint{row}-$$spoint{col}).\n";

   $$spoint{busy} = 3;
   my $status = route($$spoint{$alternative}, $epoint, $table_type);
   return undef unless defined $status;
   return 1 if $status;
   print "($table_type)(idx=$alter_idx) next_dir($next_dir)(preferred-alternative)($alternative) of ($$spoint{row}-$$spoint{col}) Led to nowhere, Trying another Alternative, if any.\n";
  } else {
   print "($table_type)(idx=$alter_idx) next_dir($next_dir)(preferred-alternative)($alternative) of ($$spoint{row}-$$spoint{col}) Is BUSY, Skipping\n";
  }
 }
 
 print "($table_type)(idx=$alter_idx) No Way to propagate from next_dir($next_dir) of ($$spoint{row}-$$spoint{col}).\n";
 $$spoint{busy} = 3;
 return 0
}

sub route_display {
 foreach my $row (0 .. $HEIGHT-1) {
  printf "%2d: ", $row;
  foreach my $col (0 .. $WIDTH-1) {
   my $point = $routing_area{$row.'/'.$col};
   print $$point{busy} == 1 ? ' #' : ($$point{busy} == 2 ? ' X' : ($$point{busy} == 3 ? ' *' : '  '))
  }

  print "\n"
 }
}
