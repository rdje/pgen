#===================================================================
# Copyright (c) 2005-2007 Richard DJE. All rights reserved.
#
# This Perl module is free software, you may redistribute it and/or 
# modify it under the same terms as Perl itself.
#===================================================================
package TcFlow;

use TableGrep;


sub GetSH {
my $lofile = shift;

 die "(TcFlow::GetSH) -E- File '$lofile' is either empty or doesn't exist, " unless -s $lofile;
 
 print "\n(TcFlow::GetSH) -I- Loading '$lofile'..\n";

 my $lofdata	 = qx(cat $lofile);
 my ($setupaths) = $lofdata =~ /=consolidated_setup=\s*(.+)\s*=consolidated_setup_end=/s; 
 my ($holdpaths) = $lofdata =~ /=consolidated_hold=\s*(.+)\s*=consolidated_hold_end=/s; 

 my @setup       = ();
 my @hold        = ();
    @setup       = map {[split /\s+/]} split(/\n/, $setupaths) if $setupaths;
    @hold        = map {[split /\s+/]} split(/\n/, $holdpaths) if $holdpaths;

 print "(TcFlow::GetSH) -I- Done.\n";
 return {setup=>[@setup], hold=>[@hold]}
}

sub Get {
my $lofile = shift;

 die "(TcFlow::Get) -E- File '$lofile' is either empty or doesn't exist, " unless -s $lofile;
 
 print "\n(TcFlow::Get) -I- Loading '$lofile'.. ";

 my   @table       = map {[split /\s+/o]} split(/\n/o, do {local(@ARGV, $/) = $lofile; <>});

 print "Done.\n";
 return \@table
}

sub GetAll {
my ($file_list) = @_;

 my @alltable;

 foreach my $cfile (@$file_list) {
  unless (-s $cfile) {
   warn  "(TcFlow::GetAll) -E- File '$cfile' is either empty or doesn't exist,";
   next
  }

  print "(TcFlow::GetAll) -I- Loading '$cfile'..\n";
  push @alltable, map {[split /\s+/o]} split(/\n/o, do {local(@ARGV, $/) = $cfile; <>})
 }

 print "(TcFlow::GetAll) -I- Done.\n";
 return \@alltable
}

sub Scale {
my ($setup_hold, $scaled_consolidated_setup_hold)  = splice @_, 0, 2;


 (@_ % 2) && die  "(TcFlow::ScaleSlacks) -E- Don't have an Odd number of arguments,";
 
  
 $setup_hold            = GetSH($setup_hold) unless ref $setup_hold;

 my %options            = @_;
 
 my $conf               = $options{conf} && (ref($options{conf}) ? $options{conf} : HUtils::Link($options{conf})) || undef;

 my $interface;
 my $filter;
 unless ($options{nofilter}) {
  $interface	        = $options{interface} || $conf && $conf->{interface}                    || die "(TcFlow::ScaleSlacks) -E- *interface* parameter NOT Defined,";
  $filter               = $options{filter}    || $conf && $conf->{interface_filter}{$interface} || die "(TcFlow::ScaleSlacks) -W- *$interface* interface's filter NOT Defined";
 } else {
  print "(TcFlow::ScaleSlacks) -W- No filter defined.\n"          
 }

 my $target_QC          = $options{target}    || $conf && $conf->{qc_target}                    || die "(TcFlow::ScaleSlacks) -E- No QC target defined,";

 my $any_2_reference_QC = HUtils::Conf($options{scale_file} ? $options{scale_file} : ($conf && $conf->{qc_scale_file} || die "(TcFlow::ScaleSlacks) -E- Unable to locate the QC scale file,"));


 my $slack 	   = TableGrep::IndexOf('slack');
 my $corner 	   = TableGrep::IndexOf('corner');
 my %typath;
 open(WSCALE, ">$scaled_consolidated_setup_hold") || die "(TcFlow::ScaleSlacks) -E- Can't write open file '$scaled_consolidated_setup_hold', ";
 foreach my $type (qw(setup hold)) {
  my $filtpaths    = $options{nofilter} ? $$setup_hold{$type} : TableGrep::Filter($filter, $$setup_hold{$type});
  unless ($options{nofilter} || $filtpaths) {print "(TcFlow::ScaleSlacks) -E- NO '$type' MATCH for filter ** $filter **\n"; next} 

  my @typath	   = @$filtpaths;
  foreach (0 .. $#typath) {
   my $scaling       = $$any_2_reference_QC{$typath[$_][$corner]}/$$any_2_reference_QC{$target_QC};

   unless ($scaling) {
    print "(TcFlow::ScaleSlacks) ($type)($_) -W- Computed Scaling factor in NULL !!\n";
    next
   }
   
   my $actual_factor = $options{divide_by} ? 1/$scaling : $scaling;

   #print "QC($typath[$_][$corner]) --> QC_REF($target_QC) : <$actual_factor>\n";
   push @{$typath[$_]},  $typath[$_][$slack]*$actual_factor;
  }

  print WSCALE "=consolidated_${type}=\n";
  print WSCALE "@$_\n" foreach (@typath);
  print WSCALE "=consolidated_${type}_end=\n";

  $typath{$type} = [@typath]
 }

 close(WSCALE);

 return \%typath;
}


1;
