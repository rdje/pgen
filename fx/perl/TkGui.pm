#===================================================================
# Copyright (c) 2005-2007 Richard DJE. All rights reserved.
#
# This Perl module is free software, you may redistribute it and/or 
# modify it under the same terms as Perl itself.
#===================================================================
package TkGui;

require Lispish;

my $descr = {
 spec => {

 sub_gui_list => sub {
my ($descr, $STRING) = @_; 
my $IMATCH = $$info{match}; 
my $IINDEX = $$info{index}; 
my $IPOS  = pos $$STRING;

my @sub_guis; 

 while (1) {
  my $minfo = LinkedRE::or($STRING, $$descr{gdata}{sub_gui_list});
  unless($minfo) {
  return {@sub_guis};
  }

  my $LMATCH = $$minfo{match};
  my $LINDEX = $$minfo{index};
  my $LSPOS  = pos $$STRING;
  
  
  
  
   if($$minfo{index} == 0) {
    push @sub_guis, &{$$descr{spec}{sub_gui}}($descr, $STRING, $minfo)
   } elsif ($$minfo{index} == 1) {
    next
   }

  
 }
 },

 sub_gui => sub {
my ($descr, $STRING, $info) = @_; 
my $IMATCH = $$info{match}; 
my $IINDEX = $$info{index}; 
my $IPOS  = pos $$STRING;


 my ($subgui_name) = $IMATCH =~ /(\S+)/;
# print "Found a SUB GUI entry point <$subgui_name>\n";
; 

 while (1) {
  my $minfo = LinkedRE::or($STRING, $$descr{gdata}{sub_gui});
  unless($minfo) {
   return undef
  }

  my $LMATCH = $$minfo{match};
  my $LINDEX = $$minfo{index};
  my $LSPOS  = pos $$STRING;
  
  
  
  
   if($$minfo{index} == 0) {
    &{$$descr{spec}{curlyb}}($descr, $STRING, $minfo)
   } elsif ($$minfo{index} == 1) {
    return ($subgui_name => '('.substr($$STRING, $IPOS, $LSPOS - $IPOS - 1).')')
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
   return undef
  }

  my $LMATCH = $$minfo{match};
  my $LINDEX = $$minfo{index};
  my $LSPOS  = pos $$STRING;
  
  
  
  
   if($$minfo{index} == 0) {
    &{$$descr{spec}{curlyb}}($descr, $STRING, $minfo)
   } elsif ($$minfo{index} == 1) {
    next
   } elsif ($$minfo{index} == 2) {
    return
   }

  
 }
 },

 comment => sub {
my ($descr, $STRING, $info) = @_; 
my $IMATCH = $$info{match}; 
my $IINDEX = $$info{index}; 
my $IPOS  = pos $$STRING;


 },

 _main_ => sub {
my ($descr, $STRING) = @_; 

 while (1) {
  my $minfo = LinkedRE::or($STRING, $$descr{gdata}{_main_});
  return undef unless $minfo;

  
  if($$minfo{index} == 0) {
   &{$$descr{spec}{sub_gui}}($descr, $STRING, $minfo)
  } elsif ($$minfo{index} == 1) {
   &{$$descr{spec}{curlyb}}($descr, $STRING, $minfo)
  } elsif ($$minfo{index} == 2) {
   &{$$descr{spec}{comment}}($descr, $STRING, $minfo)
  }
 }
 }
},

 gdata => {
 sub_gui	=> qr/(?-xism:(?-xism:\{)(?{$pos=0})|(?-xism:\})(?{$pos=1}))/o,
 sub_gui_list	=> qr/(?-xism:(?-xism:\S+\s+\{)(?{$pos=0})|(?-xism:#.*\n)(?{$pos=1}))/o,
 _main_	=> qr/(?-xism:(?-xism:\S+\s+\{)(?{$pos=0})|(?-xism:\{)(?{$pos=1})|(?-xism:#.*\n)(?{$pos=2}))/o,
 curlyb	=> qr/(?-xism:(?-xism:\{)(?{$pos=0})|(?-xism:#.*\n)(?{$pos=1})|(?-xism:\})(?{$pos=2}))/o
 }
};


sub Get {
my ($guifile) = @_;

 -s $guifile || die "(TkGui::Get) -E- File '$guifile' is either empty or does not exist,";

 my $file_content = do {local(@ARGV, $/) = $guifile; <>};
 my $gui_data     = &{$descr->{spec}{sub_gui_list}}($descr, \$file_content);
 my @gui_data     = map {$_ => Lispish::single(\$gui_data->{$_})} keys %$gui_data;

 return {@gui_data}
}


1;
