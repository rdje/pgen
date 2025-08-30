#===================================================================
# Copyright (c) 2005-2008 Richard DJE. All rights reserved.
#
# This Perl module is free software, you may redistribute it and/or 
# modify it under the same terms as Perl itself.
#===================================================================
package LibReader;

use 5.010;

use LinkedSpec;

sub Get     {LinkedSpec::get_parser('lib_reader')->($_[0])}
sub Read    {my $none_idx = 0; HFormat(Get(@_), \$none_idx)}
sub HFormat {
my ($libnode, $none_idx) = @_;

 if (ref $libnode->[0]) {
  my @lvdata;
  foreach (@$libnode) {push @lvdata, %{HFormat($_, $none_idx)}}
  return {@lvdata}
 } else {
  my $isgroup = $libnode->[0] eq 'GROUP';
  my $lvkey   = $isgroup ? $libnode->[1].':'.($libnode->[2] || '_none'.$$none_idx++.'_')  : $libnode->[1];
  return {$lvkey => $isgroup ? HFormat($libnode->[3], $none_idx) : $libnode->[2]}
 }
}

sub Recurse {
my ($libnode, $nodeid, $nodecode) = @_;

 if (ref $libnode->[0]) {Recurse($_, [@$nodeid], $nodecode) foreach (@$libnode)}
 else {
#print "#@$libnode#\n";
   my $nmatch  = grep {$nodeid->[$_] eq '*' || $nodeid->[$_] eq $libnode->[$_]} (0 .. 1);

#   print "nmatch[$nmatch] nodeid<@$nodeid>\n";
   if ($nmatch == 2) {
    my @newinfo = @$libnode;
    shift @newinfo;
    $nodecode->(@newinfo)
   } elsif ($libnode->[0] eq 'GROUP') {#print "Recursion on $libnode->[0]/$libnode->[1]/$libnode->[2]\n";
    Recurse($libnode->[3], $nodeid, $nodecode)
   }
 }
}

1;
