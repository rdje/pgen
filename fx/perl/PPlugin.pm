#===================================================================
# Copyright (c) 2005-2008 Richard DJE. All rights reserved.
#
# This Perl module is free software, you may redistribute it and/or 
# modify it under the same terms as Perl itself.
#===================================================================
package PPlugin;

use 5.010;

use LinkedSpec;


sub new {
my $class = ref $_[0] || $_[0];

state $main_str = do { 
 my $get         = LinkedSpec::get_parser('pplugin');
 
 my $top         = Cwd::realpath(File::Spec->catdir((File::Basename::fileparse(Cwd::abs_path($INC{__PACKAGE__.'.pm'})))[1], File::Spec->updir()));
 my @plugin_list = glob q({).join(',', map {"$_/*.plg"} File::Spec->rel2abs('.'), File::Spec->catdir($top, 'plugin')).q(});

 my @plugins;
 foreach my $cplugin (@plugin_list) {
  my $content = do {local(@ARGV, $/) = $cplugin; <>};
  my $rt      = $get->(\$content); say $@ if $@;
 
  unless ($rt) {
   print "(PPlugin) -W- Issue parsing plugin file '$cplugin'\n";
   next
  }
 
  push @plugins, %$rt;
 }

  my %plugins = @plugins;
  # trying to directly output 
  #   {@plugins} 
  #
  # seems to infact output 
  #   @plugins
  #
  # I really can't explain this behaviour
  # I've seen this many times
  \%plugins
 };


 bless $main_str, $class;
}


sub get  {(ref $_[0] ? $_[0] : __PACKAGE__->new)->{$_[1]}}
sub exec {
my ($this, $autoload_or_subname) = splice @_, 0, 2;

 my ($method) = $autoload_or_subname =~ /(\w+)$/o;
 my $plugin = $this->get($method);
 
 die "(PPlugin::exec) -E- Unknown plugin '$method' (<- $autoload_or_subname)," unless $plugin; 

 goto &$plugin
}

sub AUTOLOAD {__PACKAGE__->new->exec($AUTOLOAD, @_)}
sub DESTROY  {}

1;

