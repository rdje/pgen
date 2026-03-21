#===================================================================
# Copyright (c) 2005-2008 Richard DJE. All rights reserved.
#
# This Perl module is free software, you may redistribute it and/or
# modify it under the same terms as Perl itself.
#===================================================================
package LinkedRE;
use re 'eval';

sub or {
my ($stref, $oredRE) = @_;

 return undef unless $$stref =~ /(?{my $pos=0})$oredRE/gcp;
 return {index=>$pos, match      => ${^MATCH},
	              match_list => [grep {defined} map {eval "\$$_"} 1 .. scalar @+],
		      match_hash => {%+}
	};
}

sub oredRE {
my $REs	        = [@_];
my $ored        = join '|', map {"$$REs[$_]\(?{\$pos=$_}\)"} 0 .. $#$REs;

 return qr/$ored/;
}


1;
