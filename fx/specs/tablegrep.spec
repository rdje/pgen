grep::
 -> re_term	{$retv = call(re_term)}
 -> or_op	{$retv = call(or_op)}
 -> and_op	{$retv = call(and_op)}
 -> group	{$retv = call(group)}

I {
# print "\n\n\nGREP: START \n";
 my @internal;
 my $prev_node_type;
}

LX {return @internal ? \@internal : undef}
LS {my $retv}
LE {
 return undef unless $retv;

 if ($prev_node_type && $prev_node_type =~ /_OP/o && $$retv{type} =~ /_OP/o) {
  print "ERROR: Two operators w/o neither a RE_TERM nor a GROUP in between\n";
  exit 1
 }
 
 push @internal, $retv;
 $prev_node_type = $retv->{type}
}
#======== End Of grep ========


group:	/\(/ /\)/
 -> group		{$retv = call(group)}
 -> re_term		{$retv = call(re_term)}
 -> or_op		{$retv = call(or_op)}
 -> and_op		{$retv = call(and_op)}
 -> group[1]		{
#  print "GROUP: ) CLOSING\n";
  unless (@internal) {
   print "\nERROR: ** Empty **  GROUP\n";
   exit 2
  }

  return {type=>'GROUP', group=>\@internal}
 }

I {
# print "GROUP: OPENING ( \n";
 my @internal;
 my $prev_node_type;
}

LS {my $retv}
LE {
 return undef unless $retv;

 if ($prev_node_type && $prev_node_type =~ /_OP/o && $$retv{type} =~ /_OP/o) {
  print "\nERROR: Two operators w/o neither a RE_TERM nor a GROUP in between\n";
  exit 1
 }

 push @internal, $retv;
 $prev_node_type = $retv->{type}
}
#==========


re_term: /((?:\w+|\[\d+\]))\s*([!=])~\s*\/(.+?)(?<!\\)\//
I {
 my ($field, $sens, $re) = @IMATCH_LIST;
# print "RE_TERM#$IMATCH#->($field)($re)\n";

 my ($subscript) =  $field =~ /\[(\d+)\]/o;
 return {type=> defined($subscript) ? 'STERM' : 'TERM', field=>defined($subscript) ? $subscript : $field, sens=>$sens, re=>$re}
}

or_op: /\|\|/		I {
# print "OR_OP#$IMATCH#\n"; 
 return {type=>"OR_OP"}
}

and_op: /\&\&/		I {
# print "AND_OP#$IMATCH#\n"; 
 return {type=>"AND_OP"}
}
