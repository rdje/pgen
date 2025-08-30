package LispML;

use 5.010;


our @ISA = PPlugin;

use HUtils;
use Storable;

# static/class method
sub new {
my $desc = $_[1] ? {_desc=>Lispish::multi ($_[1])} : {}; 

 $desc->{_conf} = init (Lispish::multi (PathSearch->go ("lispml"))); 

 bless $desc, $_[0];
}


sub init {
my $cf = shift;

 my $crail = {};
 $$crail{entity_info} = {map {$$_[1][0] => [grep {defined && length} @{$$_[1]}[1 .. $#{$$_[1]}]]} grep {$$_[0] eq 'entity'} @$cf};

 # Recursively substituing entities' values starting with '$', aka, entity references
 entref_subst($crail); 
  

 $$crail{attribute_info} = {map {
                             my $v = {map {m/:/o; $` => $' eq '-' ? 0 : $'} map {m/^\$(\S+)/o ? entrefdo_subst ($crail, $1) : $_} @{$$_[1]}[1 .. $#{$$_[1]}]};

		             if (ref $$_[1][0]) {
                              map {lc $_ => $v} map {m/^\$(\S+)/o ? entrefdo_subst ($crail, $1) : $_} @{$$_[1][0][1]}
		             } else {
                              lc $$_[1][0] => $v
		             }
	                    } grep {$$_[0] eq 'attlist'} @$cf};



 $$crail{alias_info} = {map {$$_[1][0] => $$_[1][1]} grep {$$_[0] eq 'alias'} @$cf};
 $crail
}

sub entref_subst   {my $cr = $_[0]; HUtils::WRecurse ($$cr{entity_info}, sub {my ($ks, $vs) = @_; [map {m/^\$(\S+)/o ? entrefdo_subst ($cr, $1) : $_} @$vs]})}
sub entrefdo_subst {map {m/^\$(\S+)/o ? entrefdo_subst ($_[0], $1) : $_} @{$_[0]{entity_info}{$_[1]}}}


# intance methods
sub ctype    : lvalue {$_[0]{_ctype}}
sub v_indent : lvalue {$_[0]{_indent}}

sub attributes        {$_[0]{_conf}{attribute_info}{$_[1]}}
sub alias             {$_[0]{_conf}{alias_info}{$_[1]}}
sub indent            {$_[0]->v_indent += $_[1] || 2}
sub dedent            {$_[0]->v_indent -= $_[1] || 2}
sub insert_indent     {" " x $_[0]->v_indent . $_[1]}
sub desc              {wantarray ? @{$_[0]{_desc}} : $_[0]{_desc}}

sub render_html       {join '', map {$_[0]->gnode ($_)} grep {$$_[0] eq ($_[1] || 'html')} $_[0]->desc}

# A generic node has:
# - attributes, i.e, options
# - sub-gnodes
sub gnode {
my ($this, $cnode) = @_;

 return $cnode unless ref $cnode;

 $this->{_ctype}= $cnode->[0];

 my $alias      = $this->alias($cnode->[0]);

 $cnode         = $this->subst_expand ({ARGV=>$cnode->[1]}, $alias) if $alias;

 my $opt_spec   = $this->attributes($cnode->[0]);
 my %opt        = $this->get_options ($cnode->[1], $opt_spec);
 my $attributes = join " ", map {qq/$_="$opt{$_}"/} grep {defined $opt_spec->{$_}} sort {$a cmp $b}  keys %opt;

 my $content    =  $opt{ARGV} ? join "", map {$this->indent; my $o = $this->gnode($_); $this->dedent; $o} @{$opt{ARGV}} : "";

 my $o          = $this->insert_indent ("<".uc($cnode->[0]).($attributes ? " $attributes" : "").">\n"). 
                   $content.
                  ($content && $this->insert_indent ("</".uc($cnode->[0]).">\n") || "");
 

 $o
}


sub subst_expand {
my ($this, $context, $gnode) = @_;

 my $raw_info = {};
 my $clone = Storable::dclone($gnode);
 Lispish::substitute ($context, $clone,  sub {my ($c, $v, $r) = @_;
  return $v unless $v =~ m/^\$(\S+)/o;
  my $subst_v = HUtils::avv_get($c, [split /\//o, $1]);

  $$raw_info{"$r"} = $r if defined $subst_v  && ref $subst_v;

  defined $subst_v ? (ref $subst_v ? {v=>$subst_v} : $subst_v) : $v;
 });

 foreach (values %$raw_info) {
  @$_ = map {ref $_ eq 'HASH' ? @{$$_{v}} : $_} @$_;
 }

 $clone
}


sub get_options    {
my ($this, $argv, $opt_spec) = @_;

 return unless $argv;

 my $opt = {};
 my @argv = map {m/=/o ? ($`, $') : $_} @$argv;
 while (my $v = shift @argv) {
  if ((my $m) = $v =~ /^--?(\w[\w-]*)/o) {
   
   # mc   = Match Count
   # mopt = Matched OPTions
   my ($mc, @mopt) = 0;
   foreach (keys %$opt_spec) {
    if (/^$m/) {
     ++$mc;
     push @mopt, $_;
    } 
   }

   if ($mc == 1) {
    $m = $mopt[-1];
    if ($opt_spec->{$m}) {
     # w/ value
     $v = shift @argv;
     unless ($opt->{$m}) {
      $opt->{$m} = $v;
     } elsif (ref $opt->{$m}) {
      push @{$opt->{$m}}, $v; 
     } else {
      push @{$opt->{$m} = [$opt->{$m}]}, $v;
     }
    } else {
     # w/o value
     $opt->{$m}++
    }
   } elsif ($mc == 0) {
    # Option not to be handled at this level, so propagate it to sub-comman(s)
    push @{$opt->{ARGV}}, $v  
   } else {
    # Multiple matches !!
    print "-E- Ambiguous option '$m', matches (".(join ", ", map {"--$_"} @mopt).")\n";
    print "-I- Please check command *$this->{_ctype}*\n";
    exit 1;
   }
  } else {
   push @{$opt->{ARGV}}, $v 
  }
 }

 return wantarray ? %$opt : $opt;
}

1;
