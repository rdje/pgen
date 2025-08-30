#===================================================================
# Copyright (c) 2005-2007 Richard DJE. All rights reserved.
#
# This Perl module is free software, you may redistribute it and/or 
# modify it under the same terms as Perl itself.
#===================================================================
package Table;

sub vsplit {
my ($table, $splitspec, %option) = @_;

 if (ref($splitspec) eq 'ARRAY') {
  my @tablelist;
  foreach my $cl (@$table) {
   push @tablelist, list2table($cl, $splitspec);
  }

  my @otablelist;
  foreach my $ct (@tablelist) {
   my $ti = 0;
   foreach my $cl (@$ct) {
    push @{$otablelist[$ti]}, $cl;
    ++$ti;
   }
  }

  return \@otablelist
 }
}

sub vmerge {
my $table_list = [@_];

 my $max_height = (sort {$b <=> $a} map {scalar(@$_)} @$table_list)[0];
 # All lines of a given table should have the same length
 my @width_list = map {scalar(@{$_->[0]})} @$table_list;

 my @omerge;
 foreach my $cl (0 .. $max_height-1) {
  my @omline;
  foreach my $ct (0 .. $#$table_list) {                                                            #              WHY THIS ?
   # Tables with empty entries, i.e, lines will be ignored.                                                  vvvvvvvvvvvvvvvvvvvv
   push @omline, (defined $table_list->[$ct][$cl] ? @{$table_list->[$ct][$cl]} : (undef) x $width_list[$ct]) if $width_list[$ct]; 
  }

  push @omerge, [@omline];
 }

 return \@omerge
}

sub list2table {
my ($list, $width) = @_;

 my @table;

 unless (ref($width) eq 'ARRAY') {
  push @{$table[$_/$width]}, $$list[$_] foreach 0 .. $#$list;
 } else {
  foreach my $widx (0 .. $#$width) {
   my $lower_bound = $widx ? eval(join("+", @$width[0 .. $widx-1])) : 0;
   #print "<".$lower_bound."> <".($lower_bound + $width->[$widx] - 1).">\n";
   my $lob = $lower_bound;
   my $upb = $lower_bound + $width->[$widx] - 1;

   if ($lob <= $#$list) {
    $upd = $upb <= $#$list ? $upd : $#$list;

    push @table, [@$list[$lob  .. $upb]];
   }

   #print "list2table=($lob)($upd): <".join('> <', @$list[$lob  .. $upd]).">\n";

   ++$widx;
  }
 }

 #print "################################################# SEP #####################################33\n";
 return [@table]
}

sub iterate {
my ($table, $tcode) = @_;

 foreach my $cr (0 .. $#$table) {
  foreach my $cc (0 .. $#{$table->[$cr]}) {
   $tcode->($table->[$cr][$cc]);
  }
 }

 $table
}

1;
