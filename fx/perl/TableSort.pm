#===================================================================
# Copyright (c) 2005-2007 Richard DJE. All rights reserved.
#
# This Perl module is free software, you may redistribute it and/or 
# modify it under the same terms as Perl itself.
#===================================================================
package TableSort;

use TableGrep;
use PPlugin;


sub GenericFilter {
my ($conf, $a2d_ref, $filter_seq) = @_;

 my %option          = @_[3 .. $#_];

 my %filtered;
 my @filter_sequence = ref($filter_seq) ? @$filter_seq : (ref($conf->{$filter_seq}) ? @{$conf->{$filter_seq}} : ($conf->{$filter_seq}));
 my $cfilter         = shift  @filter_sequence;
 
 

 $maptable           = ($option{maptable} && $conf->{maptable}{$option{maptable}}) || 
                                             $conf->{maptable}{$cfilter}           || 
		                             $conf->{maptable}{DEFAULT}            || 
		        $option{maptable}                                          || die "(TableSort::GenericFilter) -E- No default maptable defined,";

#print "### filter_seq<$filter_seq> maptable<$maptable> cfilter<$cfilter>\n";
 
 Recurse($conf->{$cfilter}, sub {my ($info, $filter_exp) = @_; #print "processing =======> <@$info><$filter_exp>(".@$a2d_ref.")\n";
		             my $filteredata;
		             if (ref $filter_exp) {
                              my @args    = @$filter_exp; 
                              my $action  = shift @args;
			      my @varargs;

			      push @varargs, $conf, $info, $a2d_ref, $maptable, \@args;
			      $filteredata = PPlugin->get ("genericfilter_$action")->(@varargs);
			      die "$@, " if $@;

			     } else {
                              my @args;
			      push @args, maptable=>$maptable  if $maptable;
			      push @args, exclusive=>1         if $option{exclusive};
			      push @args, nocompile=>1         if $option{nocompile};

			      #print "before TableGrep::Filter<@args>(".@$a2d_ref.")\n";
		              $filteredata = TableGrep::Filter($filter_exp, $a2d_ref, @args);
			     }

                             print "(TableSort::GenericFilter) -W- No path found for ** ... @$info **\n" unless $filteredata;
			     eval q($filtered{").join(q("}{"), @$info).q("} = $filteredata);
			     #eval q($filtered{").join(q("}{"), @$info).q("} = $filteredata if $filteredata;)
                            });

 my @args = (maptable=>$maptable);
 push @args, exclusive=>1 if $option{exclusive};
 WRecurse(\%filtered, sub {GenericFilter($conf, $_[1], \@filter_sequence, @args)}) if @filter_sequence;

 return \%filtered
}

1;
