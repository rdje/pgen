#===================================================================
# Copyright (c) 2005-2008 Richard DJE. All rights reserved.
#
# This Perl module is free software, you may redistribute it and/or 
# modify it under the same terms as Perl itself.
#===================================================================
package HUtils;

use 5.010;

use HLinkSubst;
use Lispish;
use PathSearch;
use TableGrep;
use PPlugin;


sub new {
 my ($class) = @_;
 
 my $r = do {unless (my $rt = ref $_[1]) {
            Read (@_[1 .. $#_])
           } elsif ($rt eq 'HASH') {
                   $rt
           } else {
            die "-E- Only Hash-Ref are supported,"
           }
         };

 bless $r, $class
}


sub Link {
my ($larg) = @_;

 return undef unless $larg;

 my $linko = ref($larg) ? $larg : Read($larg);

 WRecurse($linko, [\&hlink, $linko])
}

sub Conf {Read(@_)}

sub Read {
my $hread = shift;

 return undef unless $hread;

 my $ALLREs;
 my @codes;

 unless (ref $hread) {
  -s $hread || die "(HUtils::Read) -E- File '$hread' is either empty or does not exit,";

  my $hreadata = do {open (my $f, $hread); local $/; <$f>};
  $hread       = \$hreadata;
 }
  
 if (@_) {
  die "(HRead) -E- Need an Odd number of arguments, " if @_ % 2; 
  my %recodepairs = @_;
  my $pos = 0;
  $ALLREs = LinkedRE::oredRE(map {$codes[$pos++] = $recodepairs{$_}; qr/^$_$/} keys %recodepairs);
 }
 
 my @hread;
 foreach my $lite (@{Lispish::multi($hread)}) {
  if (ref($lite)) {
   my $minfo = $ALLREs ? LinkedRE::or(\$$lite[0], $ALLREs) : undef;

   push @hread, %{$minfo ? &{$codes[$$minfo{index}]}($lite) : Format($lite)}
  } else {
   push @hread, $lite=>1;
  }
 }

 return {@hread}
}

sub recurse {Recurse (@_)}
sub Recurse {
my ($self, $coderef, %opt) = @_;

 my @path = $opt{path} && @{$opt{path}} ?  @{$opt{path}} : ();

 return $self if !$self || $opt{hashseen}{$self};
 $opt{hashseen}{$self} = 1;

 my $haskey = 0;
 foreach my $ck (sort {$a cmp $b} keys %$self) {
  ++$haskey;

  push @path, $ck;

  if (ref($$self{$ck}) && "$$self{$ck}" =~ /HASH\(/o) {
   $opt{path}              = [@path];

   if (@path > 80) {
    say "\n(Recurse) -E HTree too deep (>80) issue?";
    exit 1;
   }

   Recurse($$self{$ck}, $coderef, %opt)
  } else {
   my @args = ([@path], $$self{$ck});

   my $code;
   if (ref($coderef) eq 'ARRAY') {
    my @code = @$coderef;
    $code = shift @code;
    push @args, @code;
   } else {
    $code = $coderef;
   }

   &$code(@args)

  }
  
  pop @path;
 }

 unless ($haskey) {&$coderef ([@path])}

 $self
}

sub wrecurse {WRecurse (@_)}
sub WRecurse {
my ($self, $coderef, %opt) = @_;

 my @path = $opt{path} && @{$opt{path}} ?  @{$opt{path}} : ();

 return $self if !$self || $opt{hashseen}{$self};
 $opt{hashseen}{$self} = 1;

 my $haskey = 0;
 foreach my $ck (keys %$self) {
  ++$haskey;
  push @path, $ck;

  if (ref($$self{$ck}) && "$$self{$ck}" =~ /HASH\(/o) {
   $opt{path}              = [@path];

   if (@path > 80) {
    say "\n(WRecurse) -E HTree too deep (>80) issue?";
    exit 1;
   }

   WRecurse($$self{$ck}, $coderef, %opt)
  } else {
   my @args = ([@path], $$self{$ck});

   my $code;
   if (ref($coderef) eq 'ARRAY') {
    my @code = @$coderef;
    $code    = shift @code;
    push @args, @code;
   } else {
    $code = $coderef;
   }

   # Force a list context, and then take the first returned value
   # Acts as a global workaround when playing w/   * glob() *
   ($$self{$ck}) = &$code(@args)
  }
  
  pop @path;
 }

 unless ($haskey) {$_[0] = &$coderef ([@path])}

 $self
}

sub Each  {
my ($self, $keyre, $coderef, $node_id, $level_id) = @_;
 $node_id  = 0 unless defined $node_id;
 $level_id = 0 unless defined $level_id;

 foreach (sort {$a cmp $b} keys %$self) {
  $coderef->($_, $self->{$_}, $node_id++, $level_id+1) if m/$keyre/;
  $node_id = Each ($self->{$_}, $keyre, $coderef, $node_id, $level_id+1) if ref $self->{$_} eq 'HASH';
 }

 $node_id;
}

sub eitable {my @table; Each ($_[0], qr/./, sub {my ($k, $v, $ni, $li) = @_; push @table, [(' ') x $li, (ref($v) eq 'HASH' ? 'node:' : 'leaf:').$k]}); \@table}
sub itable  {my @table; Each ($_[0], qr/./, sub {my ($k, $v, $ni, $li) = @_; push @table, [(' ') x $li, $k]; }); \@table}

sub Format {my @a = hformat(shift); return {@a}}
sub hformat {
my $mainref = shift;

 die "(HUtils::Format) -E- Argument is a not a Reference, " unless ref $mainref;
 
 my $entrycount = @{$mainref->[1]};
 my $refcount   = grep {ref} @{$mainref->[1]};

 #print "Processing ($mainref->[0])($entrycount)($refcount)\n";
 if ($entrycount == $refcount) {
   # All Refs                               -> A => {...}
   $mainref->[0] => {map {hformat($_)} @{$mainref->[1]}}
 } elsif ($entrycount == 1) {
   # No Ref => just one scalar               -> A => B
   $mainref->[0] => $mainref->[1][0]
 } else {
   # Mix Ref(s) and scalar(s) / All (>1) scalars -> A => [...]
   $mainref->[0] => [map {ref($_) ? {hformat($_)} : $_} @{$mainref->[1]}]
 }
}

sub Write {
my ($levelname, $href, $fhandle, $indent) = @_;

 $indent = $indent || "";
 print {*$fhandle} "$indent($levelname\n";

 foreach my $k (keys %$href) {
  if (ref($$href{$k}) eq 'HASH') {

   Write($k, $$href{$k}, $fhandle, $indent." ");

  } else {
   print {*$fhandle} "$indent ($k @{[ref($$href{$k}) ? @{$$href{$k}} : $$href{$k}]})\n"
  }
 }

 print {*$fhandle} "$indent)\n";
}


sub hlink {
my ($info, $entv, $linko) = @_;

 if (ref $entv) {
  [map {ref($_) ? WRecurse($_, [\&hlink, $linko]) : hlink_subsitute($_, $linko)} @$entv]
 } else {
  hlink_subsitute($entv, $linko)
 }
}

sub hlink_subsitute{
my ($val, $linko, $as_link) = @_;

 $as_link ||= undef;

 # Dunno how to substitute a Reference
 return $val if ref $val;


 if ($val =~ /^seto:\/\/(\S+)\/?$/) {
  hlink_subsitute(hlink_seto($1, $linko), $linko)
 } else {
  my $hlo = HLinkSubst::Get(\$val);
  return $val unless $hlo;
  return  hlink_subsitute_now(join("", @$hlo), $linko, $as_link) unless grep {ref} @$hlo;

  return $val unless $hlo;
  #join("", map {ref($_) ? hlink_subsitute($$_, $linko, 'as_link') : $_} @$hlo)
  my @joina = map {ref($_) ? hlink_subsitute($$_, $linko, 'as_link') : $_} @$hlo;
  return @joina == 1 ? $joina[0] : join("", @joina)
 }
}

sub hlink_seto {
my ($hs, $linko) = @_;

 my @components   = split(/\//, $hs);
 my $confile_base = shift @components;
 my $inlocal      = $confile_base =~ /^local$/;
 my $conf_ent     = ($inlocal ? q($$linko) : q($$conf)).q/{qq(/.join(q/)}{qq(/, @components).q/)}/;

 my $conf         = Link(PathSearch->go($confile_base)) unless $inlocal;
 
 $hs =~ s/($confile_base)/\[$1\]/;
 eval "exists $conf_ent" || die "(HUtils::Link) -E- Undefined entry '$hs',";
 
 return eval $conf_ent
}

sub hlink_subsitute_now {
my ($hsn, $linko, $as_link) = @_;

#print "hlink_subsitute_now: hsn<$hsn>\n";

 # It is recommended not to use SPACE inside potential hash KEY
 return $hsn if $hsn =~ /\s/;

 # If it's not a link, just return
 return $hsn unless defined $as_link;

 #print "hlink_subsitute_now: AS_LINK hsn<$hsn>\n";
 my @components  = split(/\//, $hsn);
 my $conf_ent    = q/$$linko{q(/.join(q/)}{q(/, @components).q/)}/;
 my $subst_val   = eval $conf_ent;

 if (defined $subst_val) {
  return hlink_subsitute($subst_val, $linko)
 } else {
  #print "(HUtils::Link) -W- Undefined entry '$hsn'\n";
  return "[$hsn]"
 }
}

sub merge {my ($self, $rhs) = @_; __PACKAGE__->new($rhs)->recurse(sub {$self->avv_set(@_)}); $self}

sub Merge {my ($lhs, $rhs) = @_;
 Recurse($rhs, sub {my ($info, $scalar) = @_; eval q/$lhs->{qq(/.join(q/)}{qq(/,  @$info).q/)} = $scalar if defined $scalar/}) if $rhs;

 return $lhs
}

sub GenericFilter {
my ($conf, $a2d_ref, $filter_seq) = @_;

 my %option          = @_[3 .. $#_];

 my %filtered;
 my @filter_sequence = ref($filter_seq) ? @$filter_seq : (ref($conf->{$filter_seq}) ? @{$conf->{$filter_seq}} : $conf->{$filter_seq});
 my $cfilter         = shift  @filter_sequence;
 
 

 $maptable           = ($option{maptable} && $conf->{maptable}{$option{maptable}}) || 
                                             $conf->{maptable}{$cfilter}           || 
		                             $conf->{maptable}{DEFAULT}            || 
		        $option{maptable}                                          || die "(HUtils::GenericFilter) -E- No default maptable defined,";

#print "### filter_seq<$filter_seq> maptable<$maptable> cfilter<$cfilter>\n";
 
 Recurse($conf->{$cfilter}, sub {my ($info, $filter_exp) = @_; #print "processing =======> <@$info><$filter_exp>(".@$a2d_ref.")\n";
		             my $filteredata;
		             if (ref $filter_exp) {
                              my @args    = @$filter_exp; 
                              my $action  = shift @args;
			      my @varargs;

			      push @varargs, $conf, $info, $a2d_ref, $maptable, \@args;
			      $filteredata = PPlugin->exec("genericfilter_$action", @varargs);
			      die "$@, " if $@;

			     } else {
                              my @args = %option;
			      push @args, maptable=>$maptable  if $maptable;
			      #push @args, exclusive=>1         if $option{exclusive};
			      #push @args, nocompile=>1         if $option{nocompile};

			      #print "before TableGrep::Filter<@args>(".@$a2d_ref.")\n";
		              $filteredata = TableGrep::Filter($filter_exp, $a2d_ref, @args);
			     }

                             print "(HUtils::GenericFilter) -W- No path found for ** ... @$info **\n" unless $filteredata;
			     avv_set (\%filtered, $info, $filteredata);
                            });

 my @args = (maptable=>$maptable, %option);
 #push @args, exclusive=>1 if $option{exclusive};
 WRecurse(\%filtered, sub {GenericFilter($conf, $_[1], \@filter_sequence, @args)}) if @filter_sequence;

 return \%filtered
}


sub keygrep {KeyGrep (@_[0, 2, 1, 3])}
sub KeyGrep {
my ($self, $kgrep_re_or_code, $coderef, @opt) = @_;

 my $cha = $opt[0] && ref($opt[0]) eq 'HASH' ? shift @opt : {};
 my %opt = @opt;

 if (!$self || $cha->{self_seen}{$self}) {
	 #   say "(KeyGrep) -W- HASH already seen '$self', ignoring this call";
  return $self
 }

 $cha->{self_seen}{$self} = 1;

 my @path = $opt{path} && @{$opt{path}} ?  @{$opt{path}} : ();

 foreach (sort {$a cmp $b} keys %$self) {#  say "\n[$self]-HAS------> ($_) -------PROPERTY\n" if /cell|table|row/io;
  push @path, $_;

  if (ref($kgrep_re_or_code) ne 'CODE' ? "@path" =~ /$kgrep_re_or_code/ : &$kgrep_re_or_code([@path], $self)) {
   my @args = ([@path], $$self{$_});

   my $code;
   if (ref($coderef) eq 'ARRAY') {
    my @code = @$coderef;
    $code = shift @code;
    push @args, @code;
   } else {
    $code = $coderef;
   }

   &$code(@args)

  } elsif (ref($$self{$_}) && "$$self{$_}" =~ /HASH\(/o) {

   if (@path > 80) {
	   #   say "(KeyGrep) -E HTree too deep (>80) issue?, returning";
    return undef;
   }

   $opt{path}            = [@path];
   KeyGrep($$self{$_}, $kgrep_re_or_code, $coderef, $cha, %opt)
  }
  
  pop @path;
 }

 $self
}

sub grep {Grep (@_)} 
sub Grep {my ($self, $grepre, $grepcode) = @_; Recurse($self, sub {$grepcode->(@_) if "@{$_[0]}" =~ /$grepre/})}

sub Key2Table {
my ($hash) = @_;

 my @keylist; 
 my $pos=0;
 my %imap;
 my $max_width = 0;
 Recurse($hash, sub {my ($info) = @_;
        $max_width = @$info if @$info > $max_width;

	push @keylist, [@$info];
	eval q/$imap{qq(/.join(q/)}{qq(/, @$info).q/)} = $pos++/
 });

 return {table=>\@keylist, imap=>\%imap, count=>$pos, width=> $max_width}
}

sub GetKeyIndex {my ($k2t, $keys) = @_; eval q/$k2t->{imap}{qq(/.join(q/)}{qq(/, @$keys).q/)}/}

sub display {Print (@_)}
sub Print   {Recurse($_[0], sub {print "<@{$_[0]}>  {".(defined $_[1] ? $_[1] : '-UNDEFINED-')."}\n"})}

sub avv_set          {avv_get ($_[0], map {ref $_ ? @$_ : $_} @_[1 .. $#_-1]) = $_[-1] if defined $_[-1]}
sub avv_get  :lvalue {my $r = eval q/\$_[0]{qq(/.join(q/)}{qq(/,  map {ref $_ ? @$_ : $_} @_[1 .. $#_]).q/)}/; $$r}
sub avv_push         {eval q/push @{$_[0]{qq(/.join(q/)}{qq(/,   map {ref $_ ? @$_ : $_} @_[1 .. $#_-1]).q/)}}, $_[-1]/}

sub wantlist {ref $_[-1] ? (ref($_[-1]) eq 'ARRAY' ? @{$_[-1]} : undef) : $_[-1]}
1;
