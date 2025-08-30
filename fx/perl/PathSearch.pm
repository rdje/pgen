#===================================================================
# Copyright (c) 2005-2008 Richard DJE. All rights reserved.
#
# This Perl module is free software, you may redistribute it and/or 
# modify it under the same terms as Perl itself.
#===================================================================
package PathSearch;

use 5.010;

use File::Basename;
use File::Spec;
use File::Find;
use Cwd;

use Global;


sub new {bless {}, ref $_[0] || $_[0]}

sub go  {
my ($this, $basename, $extension, @more_search_dir) = @_;

return undef unless $basename;

state $search_path = do {
           my $top_dir    = Cwd::realpath(File::Spec->catdir((File::Basename::fileparse(Cwd::abs_path($INC{__PACKAGE__.'.pm'})))[1], File::Spec->updir()));
           opendir(my $dir, $top_dir) || die "can't opendir $top_dir: $!";
	   my @dirs = getcwd (); find(sub {push @dirs, $File::Find::name if -d}, $top_dir);
	   \@dirs
          };

 my $base_ext = $basename . ($extension ? ".$extension" : ".conf");
 my %a_dlist  = map {$_, 1} @$search_path, grep -d, @more_search_dir; 
 my @a_dlist  = keys %a_dlist;
 my @results  = map {File::Spec->catfile($_, $base_ext)} grep {-f File::Spec->catfile($_, $base_ext)} @a_dlist;

 @$search_path = @a_dlist;
 
 if (@results) {
  return $results[0]
 } else {
   warn "\n(PathSearch) -W Unable to find file '$base_ext' in \n** @$search_path **, ";
   return undef
 }
}


1;
