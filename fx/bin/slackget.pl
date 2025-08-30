#! /apps/perl/5.8.3/bin/perl -w -I/home/rdje/perl/modules


use Storable;
use HUtils;


my @command_list      = qw(help quit exit history);

my $slacks = retrieve("SETUP_Jan28.slacks");#MERGEDNS_handoff_Jan19_DROUTE_ECO3_netlist_starxt_1.hash");#SETUP.slacks.FIRST");
#HUtils::recurse($slacks, \&display);

TOP: while (1) {
 print "PIN> "; my $pin = <>;
 chomp $pin;
 next unless $pin;

 # Stripping Leading/trailing SPACEs, if any.
 $pin =~ s/^\s+|\s+$//g;

 unless ($pin =~ /\//) {
  my $is_a_command = is_command($pin);

  unless ($is_a_command) {
   print "Error: Unknown command '$pin'\n";
   next
  } elsif ($is_a_command == 2) {
   print "Warning: Ambigious command '$pin', try 'help'\n";
   next
  }

  if ('help' =~ /^\Q$pin\E/) {
   print " ".join("\n ", sort {$a cmp $b} @command_list)."\n";
   next
  }

  last if 'quit' =~ /^\Q$pin\E/;
  last if 'exit' =~ /^\Q$pin\E/;
  
  if ('history' =~ /^\Q$pin\E/) {
   next unless @history;
   print join("\n ", @history)."\n";
   next
  }
 }
 
 my $hashent = "\$\$slacks{'".join("'}{'", split(/\//, $pin))."'}";

 do {
  print "Error: Unknown pin '$pin'\n"; 
  next TOP;
 } unless eval "exists $hashent";
 
 my $hashtype = eval $hashent;

 if (ref($hashtype) eq 'HASH') {
  print "Warning: '$pin' is *not* a pin\n";
  next
 }
 
 print " ".eval($hashent)."\n";
 push @history, $pin;
}

print "Bye.\n";

sub is_command {
 my @match_list;
 foreach (@command_list) {
  push @match_list, $_ if /\Q$_[0]\E/;
 }

 return @match_list == 1 ? 1 : @match_list > 1 ? 2 : 0;
}
