#===================================================================
# Copyright (c) 2005-2007 Richard DJE. All rights reserved.
#
# This Perl module is free software, you may redistribute it and/or 
# modify it under the same terms as Perl itself.
#===================================================================
package HLinkSubst;

use LinkedRE;

my $descr = {
 spec => {

 substitute_top => sub {
my ($descr, $STRING) = @_; 
my $IMATCH = $$info{match}; 
my $IINDEX = $$info{index}; 
my $IPOS  = pos $$STRING;

my $retv; my @word_items; 

 while (1) {
  my $minfo = LinkedRE::or($STRING, $$descr{gdata}{substitute_top});
  unless($minfo) {
  return @word_items ? \@word_items : undef;
  }

  my $LMATCH = $$minfo{match};
  my $LINDEX = $$minfo{index};
  my $LSPOS  = pos $$STRING;
  
  
  
  
   if($$minfo{index} == 0) {
    $retv = &{$$descr{spec}{substitute_statement2}}($descr, $STRING, $minfo);
   } elsif ($$minfo{index} == 1) {
    $retv = &{$$descr{spec}{curlyb}}($descr, $STRING, $minfo)
   } elsif ($$minfo{index} == 2) {
    $retv = &{$$descr{spec}{raw_string}}($descr, $STRING, $minfo)
   } elsif ($$minfo{index} == 3) {
    print "(HLinkSubst) -E- Dangling closing bracket\n"; exit 1
   }

  push @word_items, $retv
 }
 },

 substitute_statement2 => sub {
my ($descr, $STRING, $info) = @_; 
my $IMATCH = $$info{match}; 
my $IINDEX = $$info{index}; 
my $IPOS  = pos $$STRING;

 

 while (1) {
  my $minfo = LinkedRE::or($STRING, $$descr{gdata}{substitute_statement2});
  unless($minfo) {
  print "(HLinkSubst) -E- Unmatched closing bracket\n"; exit 2;
  }

  my $LMATCH = $$minfo{match};
  my $LINDEX = $$minfo{index};
  my $LSPOS  = pos $$STRING;
  
  
  
  
   if($$minfo{index} == 0) {
    &{$$descr{spec}{substitute_statement2}}($descr, $STRING, $minfo)
   } elsif ($$minfo{index} == 1) {
    &{$$descr{spec}{curlyb}}($descr, $STRING, $minfo)
   } elsif ($$minfo{index} == 2) {
    return \(my $capt = substr($$STRING, $IPOS, $LSPOS - $IPOS - 1))
   }

  
 }
 },

 curlyb => sub {
my ($descr, $STRING, $info) = @_; 
my $IMATCH = $$info{match}; 
my $IINDEX = $$info{index}; 
my $IPOS  = pos $$STRING;

 

 while (1) {
  my $minfo = LinkedRE::or($STRING, $$descr{gdata}{curlyb});
  unless($minfo) {
  print "(HLinkSubst) -E- Unmatched closing brace\n"; exit 2;
  }

  my $LMATCH = $$minfo{match};
  my $LINDEX = $$minfo{index};
  my $LSPOS  = pos $$STRING;
  
  
  
  
   if($$minfo{index} == 0) {
    &{$$descr{spec}{curlyb}}($descr, $STRING, $minfo)
   } elsif ($$minfo{index} == 1) {
    return substr($$STRING, $IPOS-1, $LSPOS - $IPOS + 1)
   }

  
 }
 },

 raw_string => sub {
my ($descr, $STRING, $info) = @_; 
my $IMATCH = $$info{match}; 
my $IINDEX = $$info{index}; 
my $IPOS  = pos $$STRING;

return $IMATCH;
 },

 _main_ => sub {
my ($descr, $STRING) = @_; 

 while (1) {
  my $minfo = LinkedRE::or($STRING, $$descr{gdata}{_main_});
  return undef unless $minfo;

  
  if($$minfo{index} == 0) {
   &{$$descr{spec}{substitute_statement2}}($descr, $STRING, $minfo)
  } elsif ($$minfo{index} == 1) {
   &{$$descr{spec}{curlyb}}($descr, $STRING, $minfo)
  } elsif ($$minfo{index} == 2) {
   &{$$descr{spec}{raw_string}}($descr, $STRING, $minfo)
  }
 }
 }
},

 gdata => {
 substitute_top	=> qr/(?-xism:(?-xism:(?<!\\)\[)(?{$pos=0})|(?-xism:\{)(?{$pos=1})|(?-xism:(\\(?:\[|\])|[^\{\}\[\]])+)(?{$pos=2})|(?-xism:(?<!\\)\])(?{$pos=3}))/o,
 substitute_statement2	=> qr/(?-xism:(?-xism:(?<!\\)\[)(?{$pos=0})|(?-xism:\{)(?{$pos=1})|(?-xism:(?<!\\)\])(?{$pos=2}))/o,
 _main_	=> qr/(?-xism:(?-xism:(?<!\\)\[)(?{$pos=0})|(?-xism:\{)(?{$pos=1})|(?-xism:(\\(?:\[|\])|[^\{\}\[\]])+)(?{$pos=2}))/o,
 curlyb	=> qr/(?-xism:(?-xism:\{)(?{$pos=0})|(?-xism:\})(?{$pos=1}))/o
 }
};


sub Get {&{$descr->{spec}{substitute_top}}($descr, $_[0])}

1;
