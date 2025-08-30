#===================================================================
# Copyright (c) 2005-2008 Richard DJE. All rights reserved.
#
# This Perl module is free software, you may redistribute it and/or 
# modify it under the same terms as Perl itself.
#===================================================================
package TableGrep;

use 5.010;
use Storable;

use HUtils;
use LinkedSpec;

sub grepconf {state $grepconfv = HUtils::Link(PathSearch->go('tablegrep'))}

sub IndexOf {
my ($fieldname, $map_name) = @_;

 my $grepconf   = grepconf();
 $map_name    //= $grepconf->{DEFAULT};

 die "(TableGrep::IndexOf) -E- Unknown MAP table '$map_name',"                       unless exists($$grepconf{$map_name});
 unless ($fieldname eq 'ALL') {
  die "(TableGrep::IndexOf) -E- Unknown field '$fieldname' of MAP table '$map_name'," unless exists $$grepconf{$map_name}{$fieldname};
 }

 return $fieldname eq 'ALL' ? {%{$$grepconf{$map_name}}} : $$grepconf{$map_name}{$fieldname}
}

sub Map {Storable::dclone(grepconf())}

sub Filter {
#my ($filter, $aoa_ref) = splice @_, 0, 2;

 my %options     = @_[2 .. $#_];

 my $filter_expr = $options{nocompile} ? $_[0] : get_filter($_[0], $options{maptable} || grepconf()->{DEFAULT}, $options{invert});
# print "<$filter_expr>\n";

 my @grep;
 my @n_grep;
 foreach (@{$_[1]}) {
  if (eval $filter_expr) {
   push @grep, $_;
  } else {
   push @n_grep, $_;
  }
 }

 $_[1] = [@n_grep]  if $options{exclusive};

 return @grep ? [@grep] : undef
}

my %cache_string;
my %cache_map;
sub get_filter {
my ($string, $maptable, $invert) = @_;
 
 $cache_string{$string}             //= LinkedSpec::get_parser('tablegrep')->(\$string);
 exit 1 unless $cache_string{$string};

 $cache_map{$string.'::'.$maptable} //=  write_grep_expr($cache_string{$string}, $maptable);

 #return  '@$_ && '.($invert ? "!" : "").'('.$cache_map{$string.'::'.$maptable}.')';
 return  ($invert ? "!" : "").'('.$cache_map{$string.'::'.$maptable}.')';
}

sub write_grep_expr {
my ($wge, $maptable) = @_;

 my $lstring = "";
 $lstring .= ($$_{type} eq 'OR_OP'  ? " || " 								     : (
 	      $$_{type} eq 'AND_OP' ? " && " 								     : (
              $$_{type} eq 'TERM'   ? "\$\$_[".IndexOf($$_{field}, $maptable)."] $$_{sens}~ /".$$_{re}."/o"  : (
	      $$_{type} eq 'STERM'  ? "\$\$_[$$_{field}] $$_{sens}~ /".$$_{re}."/o" 	                     :
                                     '('.write_grep_expr($$_{group}, $maptable).')')))) foreach (@$wge);

 return $lstring
}

1;
