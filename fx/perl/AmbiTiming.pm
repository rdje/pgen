#===================================================================
# Ambit BuildGates Report timing Parser Engine
#
# Copyright (c) 2007 Richard DJE. All rights reserved.
#
# This Perl Module is free software, you may redistribute it and/or 
# modify it under the same terms as Perl itself.
#===================================================================
package AmbiTiming;

use Getopt::Long;
use Digest::MD5;
use File::Path;
use File::Spec;
use File::Glob ':glob';
use File::Copy;

use HUtils;
use TableGrep;
use Table;
use Global;


sub new {
my $d = HUtils::Conf(PathSearch->go('ambitiming')); 

 $d->{regexps}{$_}     = qr/$d->{regexps}{$_}/ foreach keys %{$d->{regexps}};

 bless $d, ref $_[0] || $_[0]
}

# Accessor routines
sub regexp          {$_[0]{regexps}{$_[1]}}
sub rise_o_fall     {$_[1] && $_[0]{rise_o_fall}{$_[1]}}


sub read {
my ($this, $report_file, %option) = @_; 

# --output=s
# --index=i
# --csplit
# --workdir=s, only useful when combined with 'csplit'
# --noverbose
# --split
# --threshold=i

 if ($option{csplit}) {
  $option{split}              = 1;
  $option{threshold}          = 0;
 }

 my $output_f;
 my $label;
 if ($option{output} && !defined($option{index})) {
  open($output_f, "> $option{output}") || die "(AmbiTiming) -E- Can't open file $option{output} for writing, $!";
  my @label                  = split /\./o, (File::Spec->splitpath($option{output}))[2];
  pop @label if @label > 1;

  $label                     = join '', @label;
 }

 $split_threshold             = defined($option{threshold}) ? $option{threshold} : Global->split_threshold;

 $option{noverbose} || print "(AmbiTiming) -I- Processing '$report_file'..\n";
 
 my %mapinfo                  = %{TableGrep::IndexOf('ALL', 'reportiming')}; 
 
 my $cwd                      = File::Spec->rel2abs('.');
 my @paths;
 unless ($option{csplit}) {
  my $infile  = do {local(@ARGV, $/) = $report_file; <>};
     my $tpre = $this->regexp ('timing_path_re');
     @paths   = $infile =~ /$tpre/go;
  
  # De-allocating $infile memory space
  undef $infile;
  
 } else {
  mkpath($option{workdir});
  chdir $option{workdir};
 
  print "(AmbiTiming) -I- Splitting file '$report_file' ..\n";
  my $cmd    = Global->CSPLIT." 2>/dev/null -s -z -n 4 -f csplit_${label}_  $report_file '".'%\<path\s+[0-9]+:%-1'."' '".'/\<path\s+[0-9]+:/-1'."' '{*}'";
  my $status = system($cmd);
 
  $option{noverbose} || print "(PTiming) -I- CSPLIT Done.\n";

  @paths     = $status ? () : sort {$a cmp $b} bsd_glob ("csplit_${label}_*");
 }

 unless(@paths) {
  $option{noverbose} || print "(AmbiTiming) -W- No Timing path found in '$report_file'\n";
  unlink <$option{output}>;
  chdir $cwd if $option{csplit};
  return undef
 }
 
 
 if (defined $option{index}) {
  unless ($option{csplit}) {
   unless ($option{split}) {
    if ($option{index} =~ /^\d+$/o && $option{index} >= 0) {
     return $paths[$option{index}] if defined $paths[$option{index}];
     $option{noverbose} || print "(AmbiTiming) -E- Timing path #$option{index} can't be located in report file.\n";
     return undef
    } else {
     $option{noverbose} || print "(AmbiTiming) -E- The 'index' argument should be a positive integer.\n";
     return undef
    }
   } else {
     open(my $f, $report_file); 
     local $/;
     return <$f>
   }
  } else {
   my $data = do {local(@ARGV, $/) = sprintf("csplit_${label}_%04d", $option{index}); <>};
   unlink <csplit_${label}_*>;
   chdir $cwd;
   return $data;
  }
 }

 my ($avolume, $newpath, $afilename) = File::Spec->splitpath($option{output});
 my $repfile                         = (File::Spec->splitpath($report_file))[2];

 my @pathout;
 my $pathcount = @paths;
 my @outdata;
 my $path_idx=0;
 my ($se_clock_re, $se_point_re, $slack_re, $timing_data_re, $path_type_re) = map {$this->regexp ($_)} qw/se_clock_re se_point_re slack_re timing_data_re path_type_re/;   

 while(my $path = shift @paths) {
  my $filename_4_csplit;
  if ($option{csplit}) {
   $filename_4_csplit = $path;
   $path = do {local(@ARGV, $/) = $path; <>};
  }

  my $splitted = 0;       
  my $new_reportfile;
  if ($option{split} && $pathcount > $split_threshold) {
   if ($option{output}) {
    # [2] = file name
    $new_reportfile = File::Spec->catpath($avolume, $newpath, "${afilename}.${path_idx}.$repfile");

    unless ($option{csplit}) {
     open(my $s, "> $new_reportfile") || die "(AmbiTiming) -E- Can't open file '$new_reportfile' for writing, $!";
     print $s $path;
     close $s;
    } else {
      $new_reportfile = File::Spec->rel2abs($filename_4_csplit);
    }

    $splitted = 1;
   }
  }


  my %path_info;
  my $actual_report_file    = $splitted ? ($option{csplit} ? $new_reportfile : File::Spec->rel2abs($new_reportfile)) : $report_file;
  $path_info{reportfile}    = $report_file;
  $path_info{path_index}    = $path_idx;
  $path_info{query_string}  = "split=$splitted&pathcount=$pathcount&engine=ambit&file=$actual_report_file&path_index=$path_idx";
  $path_info{query_string} .= "&id=" . Digest::MD5::md5_hex($path_info{query_string}, Global->set('cgi', 'sepc'), Global->md5_encode); 

  # SE Points
   $path_info{lc $$_[0]."point"} = $$_[1] foreach @{Table::list2table ([$path =~ /$se_point_re/go], 2)};

  # SE Clocks
  my $se_clocks = Table::list2table ([$path =~ m/$se_clock_re/go], 3);
  my $launch  = (grep {$$_[1] && $$_[1] eq 'launch'} @$se_clocks)[0];
  my $capture = (grep {$$_[1] && $$_[1] ne 'launch'} @$se_clocks)[0];

  (@path_info{qw/startpoint_clock startpoint_clock_edge/}) = @$launch[0, 2]  if $launch;
  (@path_info{qw/endpoint_clock   endpoint_clock_edge/})   = @$capture[0, 2] if $capture;

  $path_info{$_} = $this->rise_o_fall ($path_info{$_}) foreach qw/startpoint_clock_edge endpoint_clock_edge/;

  # Slack
  ($path_info{slack}) = ($path =~ /$slack_re/g)[0];
  # For mimicking PTiming 
  #$path_info{slack}                                 = defined $path_info{slack} ? $path_info{slack} : $path_info{arrival_time};
 
  
  my ($timing_data) = $path =~ $timing_data_re;
  
  # Path_type 
  $path_info{path_type} = $timing_data =~ $path_type_re ? 'max' : 'min';

  my @output_line;
  # Unitialized position(s) will be set to '-'
  foreach (keys %mapinfo) {
   $output_line[$mapinfo{$_}] = defined $path_info{$_} ? $path_info{$_} : '-';
  }
 
  #print {*$OUTFILE} "@output_line\n";
  push @pathout, "@output_line";
  ++$path_idx;
 }
 
 if ($option{output}) {
  print $output_f join("\n", @pathout)."\n";
 } else {
  print STDOUT join("\n", @pathout)."\n"
 }

  if ($option{csplit}) {
   chdir $cwd; 
  }
}


1;
