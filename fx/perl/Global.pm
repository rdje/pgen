#===================================================================
# Copyright (c) 2005-2008 Richard DJE. All rights reserved.
#
# This Perl module is free software, you may redistribute it and/or 
# modify it under the same terms as Perl itself.
#===================================================================
package Global;

use 5.010;

use File::Basename;
use Cwd;

use HUtils;


sub new {
my $class = ref $_[0] || $_[0];

 state $global = do {
  my $fx_perl     = (File::Basename::fileparse(Cwd::abs_path($INC{'Global.pm'})))[1];
  my $confile     = File::Spec->catfile($fx_perl, "env.conf");
  -f $confile || die "Can't locate global 'env.conf' configuration file,"; 
  -s $confile || die "File '$confile' is empty,";
  
  my $conf        = HUtils::Conf($confile);
  $conf->{FX_TOP} = Cwd::realpath(File::Spec->catdir($fx_perl, File::Spec->updir()));

  HUtils::Link($conf)
 };

 # Loading requested conf file(s)
 state $just_once = do {
  foreach (HUtils->wantlist($global->{preload_list}{link})) {
   HUtils::avv_get ($global, $_) = HUtils::Link(PathSearch->go($_));
  }

  foreach (HUtils->wantlist($global->{preload_list}{conf})) {
   HUtils::avv_get ($global, $_) = HUtils::Conf(PathSearch->go($_));
  }
 };

 bless $global, $class;
}

sub set :lvalue {HUtils::avv_get (ref $_[0] ? $_[0] : __PACKAGE__->new, [@_[1 .. $#_]])}

sub AUTOLOAD :lvalue  {
my ($fieldname) = $AUTOLOAD =~ /(\w+)$/o;
 
 my $ref = ref $_[0] ? $_[0] : __PACKAGE__->new;

 #warn "(Global) -W- Undefined Variable [env.conf]:$fieldname," unless $ref->set($fieldname, @_[1 .. $#_]);

 $ref->set($fieldname, @_[1 .. $#_])
}

# So that AUTOLOAD won't catch it
sub DESTROY {}

1;
