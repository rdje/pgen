#===================================================================
# Primetime Report timing Parser Engine
#
# Copyright (c) 2005-2007 Richard DJE. All rights reserved.
#
# This Perl Module is free software, you may redistribute it and/or 
# modify it under the same terms as Perl itself.
#===================================================================
package PTiming;

use Getopt::Long;
use Digest::MD5;
use File::Path;
use File::Spec;
use File::Glob ':glob';
use File::Copy;

use TableGrep;
use Global;

sub new {
my $d = HUtils::Conf(PathSearch->go('ptiming')); 

 # Temporary Hack !!
 $d->{regexps}{timing_path} =~ s/<$_>/$d->{regexps}{$_}/ foreach qw/slack_re unconstrained_re/;
 $d->{regexps}{$_}          = qr/$d->{regexps}{$_}/ foreach keys %{$d->{regexps}};

 bless $d, ref $_[0] || $_[0]
}

# Accessor routines
sub regexp       {$_[0]{regexps}{$_[1]}}
sub md5_enabled  {$_[0]{md5_enabled}}
sub re_list      {@{$_[0]{re_list}}}

sub read     {
my ($this, $report_file, %option) = @_;

# --output=$x
# --index=i,   to be set only when called from a CGI to retrieve a particular timing path
# --csplit
# --workdir=s, only useful when combined with 'csplit'
# --noverbose
# --split
# --threshold=i

 # Not that clean :(
 my ($slack_re                    ,
     $unconstrained_re            ,
     $timing_path                 ,
     $startpoint_re               ,
     $endpointinfo_re             ,
     $endpoint_re                 ,
     $pathgroup_re                ,
     $pathtype_re                 ,
     $tp_descr_start              ,
     $input_external_delay_re     ,
     $output_external_delay_re    ,
     $data_arrival_time_re        ,
     $data_required_time_re       ,
     $library_sh_time_re          ,
     $clock_uncertainty_re        ,
     $epclk_info_tail_re          ,
     $clock_source_latency_re  
   ) = map {$this->regexp ($_)}  $this->re_list;

 $option{workdir}           ||= "PTIMING";
 my $csplit_test              = $this->regexp ("csplit_test_re");
 my $csplit_test_re           = qr/$csplit_test/o;
 
 if ($option{csplit} && !defined($option{index})) {
  $option{split}              = 1;
  $option{threshold}          = 0;
 }

 my $output_f;
 my $label;
 if ($option{output}) {
  open($output_f, "> $option{output}") || die "(PTiming) -E- Can't open file $option{output} for writing, $!";
  my @label                  = split /\./o, (File::Spec->splitpath($option{output}))[2];
  pop @label if @label > 1;

  $label                     = join '', @label;
 }

 my $split_threshold          = defined($option{threshold}) ? $option{threshold} : Global->split_threshold;

 $option{noverbose} || print "(PTiming) -I- Processing '$report_file'..\n";
 

 my %mapinfo                  = %{TableGrep::IndexOf('ALL', 'reportiming')}; 
 
 my $cwd                      = File::Spec->rel2abs('.');
 my @paths;
 unless ($option{csplit}) {
  unless ($option{index} && $option{split}) {
    my $infile = do {local(@ARGV, $/) = $report_file; <>};
       @paths  = ($infile =~ /$timing_path/g);
    
    # De-allocating $infile memory space
    undef $infile;
  }
  
 } else {
  mkpath $option{workdir};
  chdir $option{workdir};
 
  $option{noverbose} || print "(PTiming) -I- CSPlitting '$report_file'..\n";
  my $cmd    = Global->CSPLIT." 2>/dev/null -s -z -n 10 -f csplit_${label}_  $report_file '".'%Startpoint:\s+%-1'."' '".'/^\s*\(slack\s+\|(\s*Path\s+is\s+unconstrained)\)/+1'."' '{*}'";
  my $status = system($cmd);
 
  $option{noverbose} || print "(PTiming) -I- CSPLIT Done.\n";

  @paths     = $status ? () : sort {$a cmp $b} bsd_glob("csplit_${label}_*");
 }
 
 unless($option{index} && $option{split} || @paths) {
  $option{noverbose} || print "(PTiming) -W- No Timing path found in '$report_file'\n";
  unlink <$option{output}>;
  chdir $cwd if $option{csplit};
  return undef
 }
 

 if (defined $option{index}) {
  unless ($option{csplit}) {
   unless ($option{split}) {
    if ($option{index} =~ /^\d+$/o && $option{index} >= 0) {
     return $paths[$option{index}] if defined $paths[$option{index}];
     $option{noverbose} || print "(PTiming) -E- Timing path #$option{index} can't be located in report file.\n";
     return undef
    } else {
     $option{noverbose} || print "(PTiming) -E- The 'index' argument should be a positive integer.\n";
     return undef
    }
   } else   {
     open(my $f, $report_file); local $/; my $sc = <$f>;

     return $sc
   }

  } else {
   my $data = do {open (my $f, sprintf "csplit_${label}_%010d", $option{index}); local $/; <$f>};
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
  my $filename_4_csplit = $path;
  if ($option{csplit}) {
   $path = do {local(@ARGV, $/) = $path; <>};
   next unless  $path =~ $csplit_test_re;
  }

  my $splitted = 0;       
  my $new_reportfile;
  if ($option{split} && $pathcount > $split_threshold) {
   if ($option{output}) {
    # [2] = file name
    $new_reportfile = File::Spec->catpath($avolume, $newpath, "${afilename}.${path_idx}.$repfile");

    unless ($option{csplit}) {
     open(my $s, "> $new_reportfile") || die "(PTiming) -E- Can't open file '$new_reportfile' for writing, $!";
     print $s $path;
     close $s;
    } else {
       $new_reportfile = File::Spec->rel2abs($filename_4_csplit);
    }

    $splitted = 1;
   }
  }

  my ($startpoint)      = $path =~ /($startpoint_re)/; 
  my ($endpointinfo)    = $path =~ /($endpointinfo_re)/; 
  my ($endpoint)        = $path =~ /($endpoint_re)/; 
  my ($path_group)      = $path =~ /$pathgroup_re/; 
  my ($path_type)       = $path =~ /$pathtype_re/; 
  my ($slack)           = $path =~ /($slack_re)/; 
  my $is_unconstrained  = $path =~ /$unconstrained_re/;
 
  # Removing the path_group star(s) character if any
  $path_group           =~ s/\*//go;
 
  # Startpoint info
  my $spname            ;
  my ($spname_ini)      = ($startpoint =~ /:\s+(\S+)\s+/go);
  my ($spclkname)       = ($startpoint =~ /(\S+)\s*\)\s*$/go);
 
  if ($spname_ini =~ /\//o) {
    $path               =~ /$tp_descr_start/g;
   ($spname)            = ($path =~ /(\Q$spname_ini\E\S*)\s+\(/g);
  } else {
   $spname              = $spname_ini;
  }
 
 
  # Endpoint info
  my $epname            ;
  my ($epname_ini)      = ($endpointinfo =~ /:\s+(\S+)\s+/go);
  my ($epclkname)       = ($endpointinfo =~ /(\S+)\s*\)\s*$/go);
 
  if ($epname_ini =~ /\//o) {
   ($epname)            = ($endpoint =~ /(\S+)/o);
  } else {
   $epname              = $epname_ini;
  }
 
  my $mem          = pos($path);
  pos($path)       = 0;
      $path        =~ /$tp_descr_start/g;
 
  my ($node_range) = $spname && $epname && ($path =~ /\Q$spname\E.+?\Q$epname\E\S*\s+\(.+?\)(?:\D+\d+\.\d+){2,4}\s+\b\w\b/sg); 
  #my ($node_range) = $spname && $epname && ($path =~ /\Q$spname\E.+\Q$epname\E\S*.+?\s+\d+\.\d+\D+\d+\.\d+/sg);
  my  $nodes_list  = path_nodes_xt($node_range);
  pos($path)       = $mem;
  
  # Due to PT strange behaviour, it sometimes adds single quotes around the
  # name of the clock
     $spclkname         =~ /'[^']+'/o && $spclkname =~ s/'//go;
     $epclkname         =~ /'[^']+'/o && $epclkname =~ s/'//go;
 
#   my $spclk_edge_re    = qr/clock\s+\Q$spclkname\E\s+\(\s*(\S+).+?\)/s;
   my $spclk_timedge_re = qr/clock\s+\Q$spclkname\E\s+\(\s*\S+.+?\)(?:\s+\d+\.\d+){2,4}/s;
   my $epclk_timedge_re = qr/clock\s+\Q$epclkname\E\s+\(\s*\S+.+?\)(?:\s+\d+\.\d+){2,4}/s;
   my $spclk_info_re    = qr/(clock\s+\Q$spclkname\E\s+\(.+?\).+\Q$spname\E\S*\s+\(.+?\)(?:\D+\d+\.\d+){2,4}\s+\b\w\b)/s;
   #my $spclk_info_re    = qr/(clock\s+\Q$spclkname\E\s+\(.+?\).+?)(?:clock\s+network\s+delay\s+\(.+?\)\s+\d+\.\d+\D+\d+\.\d+|$clock_source_latency_re)?.+\Q$epname\E\S*\s+/s;
   #my $spclk_info_re    = qr/(clock\s+\Q$spclkname\E\s+\(.+?\)\s+.+?)(?:clock\s+network\s+delay\s+\(.+?\)\s+\d+\.\d+\D+\d+\.\d+|$clock_source_latency_re).+\Q$epname\E\S*.+?\s+/s;
   my $epclk_info_re    = qr/clock\s+\Q$epclkname\E\s+\(.+?\).+?$epclk_info_tail_re/s;
 
  my ($spclk_info)              = $path =~ /$spclk_info_re/; 
  my ($spclk_timedge_info)      = $spclk_info && $spclk_info =~ /($spclk_timedge_re)/;
  my ($spclk_edge)              = $spclk_timedge_info && $spclk_timedge_info =~ /\(\s*(\S+).+?\)/o; 
  my @launch_start_time         = $spclk_timedge_info =~ /\d+\.\d+/go if $spclk_timedge_info;
  my $launch_start_time         = $launch_start_time[-1];
 
  if ($spclkname eq $epclkname) {
    pos($path)  = 0;
    $path       =~ /$data_arrival_time_re/g;
  }
 
  # This /g is necessary. It is used together with this just above /g
  # When both the start and endpoint clock have the same name
  my $cpos                              = pos($path);
  my ($epclk_timedge_info)              = $path =~ /$epclk_timedge_re/g;
  my ($epclk_edge)                      = $epclk_timedge_info && $epclk_timedge_info =~ /\(\s*(\S+).+?\)/o; 
  my @capture_start_time                = $epclk_timedge_info =~ /\d+\.\d+/go if $epclk_timedge_info; 
  my $capture_start_time                = $capture_start_time[-1] if @capture_start_time;
  pos($path) = $cpos;  
 
  my  ($epclk_info)                     = $path =~ /$epclk_info_re/g;
       $epclk_info                      =~ s/$epclk_info_tail_re//o if $epclk_info;
 
  my ($slackv)                          = $slack && ($slack =~ /\s(\S+)\s*$/o);
  my ($arrival_time)                    = $path =~ /$data_arrival_time_re/;
  my ($input_external_delay)            = $path =~ /$input_external_delay_re/; 
  my ($output_external_delay)           = $path =~ /$output_external_delay_re/; 
 
  pos($path)                            = 0;
  my @arrow_points                      = $path =~ /(\S+)\s+\(.+?\)\s*<-/go;  

  my $is_combinational                  = $spname && $epname && ($spname !~ /\//o && $epname !~ /\//o);

  # Clock path information
  my $spclk_nodes                       = path_nodes_xt($spclk_info); 
  my $epclk_nodes                       = path_nodes_xt($epclk_info); 
 
  my $launch_start_data                 = $spclk_nodes && $spclk_nodes->[0];
  my $launch_end_data                   = $spclk_nodes && $spclk_nodes->[-1];
  my $launch_clock_path_startnode       = $spclk_nodes && shift @$launch_start_data;
  my $launch_start_data_delay           = $spclk_nodes && shift @$launch_start_data;
  my $launch_clock_path_endnode         = $spclk_nodes && shift @$launch_end_data;
  my $launch_end_data_delay             = $spclk_nodes && shift @$launch_end_data;
  my $launch_clock_path_delta           = $launch_start_data_delay && $launch_end_data_delay ?  ($launch_end_data_delay - $launch_start_data_delay) : '-';
 
#print "\nlaunch_clock_path_startnode($launch_clock_path_startnode)\n"; 
#print "launch_clock_path_endnode($launch_clock_path_endnode)\n"; 
  my $capture_start_data                = $epclk_nodes && $epclk_nodes->[0];
  my $capture_end_data                  = $epclk_nodes && $epclk_nodes->[-1];
  my $capture_clock_path_startnode      = $epclk_nodes && shift @$capture_start_data;
  my $capture_start_data_delay          = $epclk_nodes && shift @$capture_start_data;
  my $capture_clock_path_endnode        = $epclk_nodes && shift @$capture_end_data;
  my $capture_end_data_delay            = $epclk_nodes && shift @$capture_end_data;
  my $capture_clock_path_delta          = $capture_start_data_delay && $capture_end_data_delay ?  ($capture_end_data_delay - $capture_start_data_delay) : '-';
 
#print "\ncapture_clock_path_startnode($capture_clock_path_startnode)\n"; 
#print "capture_clock_path_endnode($capture_clock_path_endnode)\n"; 

  # mode  corner  spname  spclkname  spclk_edge  epname  epclkname  epclk_edge  slackv/arrival_time iomode filepath path_idx query_string arrival_time propagation_delay launch_start_time capture_start_time
  # input_external_delay output_external_delay startpoint_delay
  
  my $launch_clock_path_md5             = $spclk_nodes && Digest::MD5::md5_hex(map {$$_[0]} @$spclk_nodes) if $this->md5_enabled;
  my $capture_clock_path_md5            = $epclk_nodes && Digest::MD5::md5_hex(map {$$_[0]} @$epclk_nodes) if $this->md5_enabled;
  my $data_path_md5                     = $nodes_list  && Digest::MD5::md5_hex(map {$$_[0]} @$nodes_list ) if $this->md5_enabled;
  
  my $startpoint_edge                   = $nodes_list->[0][2]; 
  my $endpoint_edge                     = $nodes_list->[-1][2]; 

  my $startpoint_delay                  = $nodes_list->[0][1];
  my $propagation_delay                 = defined($startpoint_delay) && defined($arrival_time) ? ($arrival_time - $startpoint_delay): undef;

  my $actual_report_file                = $splitted ? ($option{csplit} ? $new_reportfile : File::Spec->rel2abs($new_reportfile)) : $report_file;
  my $query_string                      = "split=$splitted&pathcount=$pathcount&engine=pt&file=$actual_report_file&path_index=$path_idx";
     $query_string                     .= "&id=" . Digest::MD5::md5_hex($query_string, Global->set('cgi', 'sepc'), Global->md5_encode); 
 
  my @output_line;
  $output_line[$mapinfo{startpoint}]                    = $spname                                   ;
  $output_line[$mapinfo{startpoint_clock}]              = $spclkname                                ;
  $output_line[$mapinfo{startpoint_clock_edge}]         = $spclk_edge                               ;
  $output_line[$mapinfo{endpoint}]                      = $epname                                   ;
  $output_line[$mapinfo{endpoint_clock}]                = $epclkname                                ;
  $output_line[$mapinfo{endpoint_clock_edge}]           = $epclk_edge                               ;
  $output_line[$mapinfo{slack}]                         = defined($slackv)? $slackv : $arrival_time ;
  $output_line[$mapinfo{arrival_time}]                  = $arrival_time                             ;
  $output_line[$mapinfo{propagation_delay}]             = $propagation_delay                        ;
  $output_line[$mapinfo{launch_start_time}]             = $launch_start_time                        ;
  $output_line[$mapinfo{capture_start_time}]            = $capture_start_time                       ;
  $output_line[$mapinfo{input_external_delay}]          = $input_external_delay                     ;
  $output_line[$mapinfo{output_external_delay}]         = $output_external_delay                    ;
  $output_line[$mapinfo{startpoint_delay}]              = $startpoint_delay                         ;
  $output_line[$mapinfo{launch_clock_path_startnode}]   = $launch_clock_path_startnode              ;
  $output_line[$mapinfo{launch_clock_path_endnode}]     = $launch_clock_path_endnode                ;
  $output_line[$mapinfo{launch_clock_path_delta}]       = $launch_clock_path_delta                  ;
  $output_line[$mapinfo{capture_clock_path_startnode}]  = $capture_clock_path_startnode             ;
  $output_line[$mapinfo{capture_clock_path_endnode}]    = $capture_clock_path_endnode               ;
  $output_line[$mapinfo{capture_clock_path_delta}]      = $capture_clock_path_delta                 ;
  $output_line[$mapinfo{path_group}]                    = $path_group                               ;
  $output_line[$mapinfo{path_type}]                     = $path_type                                ;
  $output_line[$mapinfo{reportfile}]                    = $report_file                              ;
  $output_line[$mapinfo{path_index}]                    = $path_idx                                 ;
  $output_line[$mapinfo{query_string}]                  = $query_string                             ;
  $output_line[$mapinfo{launch_clock_path_md5}]         = $launch_clock_path_md5                    ;
  $output_line[$mapinfo{capture_clock_path_md5}]        = $capture_clock_path_md5                   ;
  $output_line[$mapinfo{data_path_md5}]                 = $data_path_md5                            ;
  $output_line[$mapinfo{startpoint_edge}]               = $startpoint_edge                          ;
  $output_line[$mapinfo{endpoint_edge}]                 = $endpoint_edge                            ;
  $output_line[$mapinfo{first_arrow_point}]             = $arrow_points[0]                          ;
  $output_line[$mapinfo{last_arrow_point}]              = $arrow_points[-1]                         ;
 
  # Uninitialized position(s) will be set to '-'
  foreach (keys %mapinfo) {
   $output_line[$mapinfo{$_}] = defined $output_line[$mapinfo{$_}] ? $output_line[$mapinfo{$_}] : '-';
  }
 

  #print {*$OUTFILE} "@output_line\n";
  push @pathout, "@output_line";
  ++$path_idx
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

sub path_nodes_xt {
 my ($str_data) = @_; 

 return undef unless $str_data;
 
 my $node_re = qr#(\S+)\s+\([0-9A-Z_]+\)((?:\D+\d+\.\d+){2,4})\s+(\w)#o;  

 my $index = -1;
 my @matchdata = map {++$index; $index%3 == 1 ? do {
                                                 my @matchdelay = /\d+.\d+/go; 
                                                 $matchdelay[-1]
                                                }

                                              : $_

                     } $str_data =~ /$node_re/g;

 
 my @match_pinport = 
 my $cidx          = 0;
 my @a2d;
 my @atmp;
 foreach (@matchdata) {
#        print "path_nodes_xt CIDX($cidx)\n";
  if ($cidx % 3 == 0 && int($cidx/3)) {
#         print "path_nodes_xt PUSH[@atmp]\n";
   push @a2d, [@atmp];
   @atmp = ()
  }
  
  push @atmp, $_;
  ++$cidx
 }
 
 push @a2d, [@atmp] if @atmp;

 return @a2d > 1 ? [@a2d] : undef;
}


1;
