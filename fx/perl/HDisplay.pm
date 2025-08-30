#===================================================================
# Copyright (c) 2005-2007 Richard DJE. All rights reserved.
#
# This Perl module is free software, you may redistribute it and/or 
# modify it under the same terms as Perl itself.
#===================================================================
package HDisplay;

use Tcl::Tk;


sub View {
my ($href, $hlist, $path) = @_;

 my @path = $path ? (ref($path) eq 'ARRAY' ?  @$path : $path) : qw(/);
 
 @path && $hlist->add(join('.', @path), -text=>$path[$#path]);
 foreach (sort {$a cmp $b} keys %$href) {
  push @path, $_;

  if (ref($$href{$_}) eq 'HASH') {
   View($$href{$_}, $hlist, \@path);
  } else {
   $hlist->add(join('.', @path), -text=>$path[$#path])
  }

  pop @path;
 }
}


1;
