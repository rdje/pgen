#===================================================================
# Magma Report timing Parser Engine
#
# Copyright (c) 2005-2007 Richard DJE. All rights reserved.
#
# This Perl Module is free software, you may redistribute it and/or 
# modify it under the same terms as Perl itself.
#===================================================================
package MagmaTiming;

use Getopt::Long;
use Digest::MD5;
use File::Path;
use File::Spec;
use File::Glob ':glob';
use File::Copy;

use TableGrep;
use Global;

sub new {
my $d = HUtils::Conf(PathSearch->go('magmatiming')); 

 $d->{regexps}{$_}   = qr/$d->{regexps}{$_}/ foreach keys %{$d->{regexps}};

 bless $d, ref $_[0] || $_[0]
}

# Accessor routines
sub regexp   {$_[0]{regexps}{$_[1]}}

sub read {
my ($this, $report_file, %option) = @_;

# --output=s
# --index=i
# --csplit
# --workdir=s, only useful when combined with 'csplit'
# --noverbose
# --split
# --threshold=i

 my ($separator, $timing_path) = map {$this->regexp ($_)} qw/separator timing_path/;


 if ($option{csplit}) {
  $option{split}              = 1;
  $option{threshold}          = 0;
 }

 my $output_f;
 my $label;
 if ($option{output} && !defined($option{index})) {
  open($output_f, "> $option{output}") || die "(MagmaTiming) -E- Can't open file $option{output} for writing, $!";
  my @label                  = split /\./o, (File::Spec->splitpath($option{output}))[2];
  pop @label if @label > 1;

  $label                     = join '', @label;
 }

 my $split_threshold          = defined($option{threshold}) ? $option{threshold} : Global->split_threshold;

 $option{noverbose} || print "(MagmaTiming) -I- Processing '$report_file'..\n";
 
 my %mapinfo                  = %{TableGrep::IndexOf('ALL', 'reportiming')}; 
 
 my $cwd                      = File::Spec->rel2abs('.');
 my @paths;
 unless ($option{csplit}) {
  my $infile = do {local(@ARGV, $/) = $report_file; <>};
     @paths  = ($infile =~ /$timing_path/g);
  
  # De-allocating $infile memory space
  undef $infile;
  
 } else {
  mkpath($option{workdir});
  chdir $option{workdir};
 
  print "(MagmaTiming) -I- Splitting file '$report_file' ..\n";
  my $cmd    = Global->CSPLIT." 2>/dev/null -s -z -n 4 -f csplit_${label}_  $report_file '".'%Start\s%-1'."' '".'/Start\s/-2'."' '{*}'";
  my $status = system($cmd);
 
  $option{noverbose} || print "(PTiming) -I- CSPLIT Done.\n";

  @paths     = $status ? () : sort {$a cmp $b} bsd_glob("csplit_${label}_*");
 }

 unless(@paths) {
  $option{noverbose} || print "(MagmaTiming) -W- No Timing path found in '$report_file'\n";
  unlink <$option{output}>;
  chdir $cwd if $option{csplit};
  return undef
 }
 
 
 if (defined $option{index}) {
  unless ($option{csplit}) {
   unless ($option{split}) {
    if ($option{index} =~ /^\d+$/o && $option{index} >= 0) {
     return $paths[$option{index}] if defined $paths[$option{index}];
     $option{noverbose} || print "(MagmaTiming) -E- Timing path #$option{index} can't be located in report file.\n";
     return undef
    } else {
     $option{noverbose} || print "(MagmaTiming) -E- The 'index' argument should be a positive integer.\n";
     return undef
    }
   } else {
     open(my $f, $report_file); 
     local $/ = undef;
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
     open(my $s, "> $new_reportfile") || die "(MagmaTiming) -E- Can't open file '$new_reportfile' for writing, $!";
     print $s $path;
     close $s;
    } else {
      $new_reportfile = File::Spec->rel2abs($filename_4_csplit);
    }

    $splitted = 1;
   }
  }

  my %dispatch;
  foreach (split /$separator/o, $path) {
   my ($section)       = /(\bStart\b|Reference arrival time|Clock path|Data path|Reference clock path)/o; 
 
   # This only happen at the end of $path. Unuseful text separated by $separator
   next unless defined $section;
   $section            =~ s/\s+/_/go; $section = lc($section);
   $dispatch{$section} = $_
  }
 
 
  my %path_info;
  my $actual_report_file    = $splitted ? ($option{csplit} ? $new_reportfile : File::Spec->rel2abs($new_reportfile)) : $report_file;
  $path_info{reportfile}    = $report_file;
  $path_info{path_index}    = $path_idx;
  $path_info{query_string}  = "split=$splitted&pathcount=$pathcount&engine=magma&file=$actual_report_file&path_index=$path_idx";
  $path_info{query_string} .= "&id=" . Digest::MD5::md5_hex($path_info{query_string}, Global->set('cgi', 'sepc'), Global->md5_encode); 

  foreach (keys %dispatch) {
   my $secinfo = $dispatch{$_};
 
   #print "Processing Path#$path_idx section ($_)\n";
   if (/clock_path|data_path|reference_clock_path/o) {
        $secinfo                      =~ /-{3,}\s*?\n/go;
    my ($clockinfo)                   = $secinfo =~ /clock:(.+?)\s*\n/gco;
    my  @clockinfo; @clockinfo        = split /\s+/, $clockinfo if $clockinfo;
    my ($clockname, $startime, $edge) = @clockinfo[0, 2, $#clockinfo]; 
    
    $path_info{($_ eq 'clock_path' ? 'start'  : 'end')    . "point_clock"}      = $clockname if $clockname;
    $path_info{($_ eq 'clock_path' ? 'start'  : 'end')    . "point_clock_edge"} = $edge      if $edge;
    $path_info{($_ eq 'clock_path' ? 'launch' : 'capture'). "_start_time"}      = $startime  if $startime;
 
    $path_info{"_".$_."_points"}           = [map {[split /\s+/]} split(/\n/, substr($secinfo, pos($secinfo), length($secinfo) - pos($secinfo)))];
 
   } elsif ($_ eq 'reference_arrival_time') {
     ($path_info{launch_clock_path_delta}) = $secinfo =~ /Clock\s+path\s+delay\D+(\d+)/o;
     ($path_info{propagation_delay})       = $secinfo =~ /Data\s+path\s+delay\D+(\d+)/o;
     ($path_info{arrival_time})            = $secinfo =~ /End-of-path\s+arrival\s+time\D+(\d+)/o;
     
     $path_info{startpoint_delay}          = $path_info{launch_clock_path_delta};
     $path_info{path_type}                 = $secinfo =~ /(?:Setup|Recovery)\s+time/o ? 'max' : 'min';
 
     #print "############# $path_idx##############\n$secinfo\n@@@@@@@@@@@@@@@@@@ $path_idx @@@@@@@@@@@@@@@@@@\n";
   } elsif ($_ eq 'start') {
     ($path_info{slack})                   = $secinfo =~ /Path\s+slack\s+(-?\d+)/o;
   }
  }
 
  
  my $size_clock_path_points                        = @{$path_info{_clock_path_points}}                    if $path_info{_clock_path_points};
  my @reverse_path_info_clock_path_points           = reverse @{$path_info{_clock_path_points}}            if $path_info{_clock_path_points};
 
  my $size_reference_clock_path_points              = @{$path_info{_reference_clock_path_points}}          if $path_info{_reference_clock_path_points};
  my @reverse_path_info_reference_clock_path_points = reverse @{$path_info{_reference_clock_path_points}}  if $path_info{_reference_clock_path_points};
 
  my $size_data_path_points                         = @{$path_info{_data_path_points}}                     if $path_info{_data_path_points};
  my @reverse_path_info_data_path_points            = reverse @{$path_info{_data_path_points}}             if $path_info{_data_path_points};
 
  $path_info{launch_clock_path_startnode}           = $path_info{_clock_path_points}[0][0]                 if $size_clock_path_points;
  $path_info{launch_clock_path_endnode}             = $reverse_path_info_clock_path_points[0][0]           if $size_clock_path_points           && $size_clock_path_points           >= 2;
  $path_info{capture_clock_path_startnode}          = $path_info{_reference_clock_path_points}[0][0]       if $size_reference_clock_path_points;
  $path_info{capture_clock_path_endnode}            = $reverse_path_info_reference_clock_path_points[0][0] if $size_reference_clock_path_points && $size_reference_clock_path_points >= 2;
 
  if ($path_info{capture_clock_path_startnode} && $path_info{capture_clock_path_endnode}) {
   $path_info{capture_clock_path_delta}             = $reverse_path_info_reference_clock_path_points[0][3] - $path_info{_reference_clock_path_points}[0][3];
  }
 
  $path_info{startpoint}                            = $path_info{launch_clock_path_endnode} || (defined $size_data_path_points ? $path_info{_data_path_points}[0][0] : undef);
  $path_info{endpoint}                              = $size_data_path_points && $size_data_path_points >= 2 ? $reverse_path_info_data_path_points[0][0] : undef;
 
  $path_info{startpoint}                            =~ s/:.*//o if $path_info{startpoint};
  $path_info{endpoint}                              =~ s/:.*//o if $path_info{endpoint};
 
  $path_info{launch_clock_path_md5}                 = Digest::MD5::md5_hex(map {$$_[0]} @{$path_info{_clock_path_points}})           if $size_clock_path_points;
  $path_info{capture_clock_path_md5}                = Digest::MD5::md5_hex(map {$$_[0]} @{$path_info{_reference_clock_path_points}}) if $size_reference_clock_path_points;
  $path_info{data_path_md5}                         = Digest::MD5::md5_hex(map {$$_[0]} @{$path_info{_data_path_points}})            if $size_data_path_points;
 
  # For mimicking PTiming 
  $path_info{slack}                                 = defined $path_info{slack} ? $path_info{slack} : $path_info{arrival_time};
 
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
#  open(my $output_f, "> $option{output}") || die "(MagmaTiming) -E- Can't open file $option{output} for writing, $!";

  print $output_f join("\n", @pathout)."\n";
#  close $output_f
 } else {
  print STDOUT join("\n", @pathout)."\n"
 }

 #print {*$OUTFILE} "\n";
 #close *$OUTFILE;

  if ($option{csplit}) {
   chdir $cwd; 
  }
}
 
1;
