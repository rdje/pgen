#===================================================================
# Copyright (c) 2005-2007 Richard DJE. All rights reserved.
#
# This Perl module is free software, you may redistribute it and/or 
# modify it under the same terms as Perl itself.
#===================================================================
package Reportiming;


my $slack_re	     	  = qr/slack\s+\(\s*(?:VIOLATED|MET).+/o;
my $slackv_re	     	  = qr/\s(\S+)\s*$/o;
my $unconstrained_re   	  = qr/\(\s*Path\s+is\s+unconstrained\)/o;
my $timing_path		  = qr/Startpoint:.+?(?:$slack_re|$unconstrained_re)/so;
my $startpoint_re   	  = qr/Startpoint:\s+\S+\s+\(.+\)/o;
my $endpointinfo_re       = qr/Endpoint:\s+\S+\s+\(.+\)/o;
my $endpoint_re     	  = qr#\S+\s+\(\S+\)[^/]+?(?=data arrival time)#o;
my $pathgroup_re     	  = qr/Path Group:.+/o;
my $pathtype_re     	  = qr/Path Type:.+/o;
my $tp_descr_start	  = qr/-+/o;
my $xpname_ini_re	  = qr/:\s+(\S+)\s+/o;
my $xpclkname_re	  = qr/(\S+)\s*\)\s*$/o;
my $data_arrival_time_re  = qr/data\s+arrival\s+time\s+(\S+)/o;
my $data_required_time_re = qr/data\s+required\s+time\s+(\S+)/o;
my $library_sh_time_re    = qr/library\s+(?:setup|hold)\s+time\s+(\S+)/o;
my $clock_uncertainty_re  = qr/clock\s+uncertainty\s+(\S+)/o;
my $xpclk_edge		  = qr/\(\s*(\S+).+\)/o;
my $epname_re		  = qr/(\S+)/o;


# $path_num
# -1 return all paths
# 0  return only path #0
# 1  return only path #1
# etc..
#
sub Read {
my ($reportfile, $path_num) = splice @_, 0, 2;

 die "(Reportiming::Read) -E- Need an Even number of arguments," if @_ % 2;

 -s $reportfile || die "(Reportiming::Read) -E- report file '$reportfile' is either empty or doesn't exist,";

 my %optional = @_;

 my $outfile   = $optional{outfile};
 my $mode      = $optional{mode}    || "-";
 my $corner    = $optional{corner}  || "-";
 my $iomode    = $optional{iomode}  || "-";

 my @clock_paths;

 if (!$optional{clock_path} && $path_num < 0 && !$outfile) {
  print "(Reportiming::Read) -I- Please provide the output file name.\n";
  return undef
 }
 
 my $indata = qx(cat $reportfile);
 my @paths = ($indata =~ /$timing_path/g);
 
 # De-allocating $indata memory space
 undef $indata;
 
 unless (@paths) {
  print "(Reportiming::Read) -W- No Timing path found in '$reportfile'\n";

  return undef
 }

 my $OUTFILE;
 if (!$optional{clock_path} && $path_num < 0) {
  open OUTF, "> $outfile" || die "(Reportiming::Read) -E- Cant open file $outfile for writing, ";
  $OUTFILE = *OUTF{IO};
 }
 
 my @outdata;
 my $path_idx=0;
 my @points;
 while(my $path = shift @paths) {
  if($path_num > 0 && $path_num != $path_idx) {
   ++$path_idx;
   next
  }

  my ($startpoint)	= ($path =~ /($startpoint_re)/); 
  my ($endpointinfo)  	= ($path =~ /($endpointinfo_re)/); 
  my ($endpoint)  	= ($path =~ /($endpoint_re)/); 
  my ($pathgroup)  	= ($path =~ /($pathgroup_re)/); 
  my ($pathtype)  	= ($path =~ /($pathtype_re)/); 
  my ($slack)	  	= ($path =~ /($slack_re)/); 
 
  # Startpoint info
  my $spname	 	;
  my ($spname_ini) 	= ($startpoint =~ /$xpname_ini_re/g);
  my ($spclkname) 	= ($startpoint =~ /$xpclkname_re/g);
 
  if ($spname_ini =~ /\//o) {
    $path 		=~ /$tp_descr_start/g;
   ($spname)	 	= ($path =~ /(\Q$spname_ini\E\S*)\s+\(/g);
  } else {
   $spname	 	= $spname_ini;
  }
 
 
  # Endpoint info
  my $epname	 	;
  my ($epname_ini) 	= ($endpointinfo =~ /$xpname_ini_re/g);
  my ($epclkname) 	= ($endpointinfo =~ /$xpclkname_re/g);
 
  if ($epname_ini =~ /\//o) {
   ($epname)	 	= ($endpoint =~ /$epname_re/);
  } else {
   $epname	 	= $epname_ini;
  }
  
 
  my $nodes_list;
  if ($path_num >= 0) {
   pos($path) 	= 0;
   $path 	=~ /$tp_descr_start/g;
   my ($node_range) = ($path =~ /\Q$spname\E(?:.|\n)+\Q$epname\E.*/g);
   $nodes_list      = path_nodes_xt($node_range);
  }
  
  # Due to PT strange behaviour, it sometime add single quote around the
  # name of the clock
  $spclkname 	=~ /'[^']+'/o && $spclkname =~ s/'//g;
  $epclkname 	=~ /'[^']+'/o && $epclkname =~ s/'//g;
 
  my $spclk_info_re	= qr/clock\s+\Q$spclkname\E\s+\(.+?\)\s+.+?clock\s+network\s+delay\s+\(.+?\)\s+\d+\.\d+\D+\d+\.\d+/s;
  my $epclk_info_re	= qr/clock\s+\Q$epclkname\E\s+\(.+?\)\s+.+?clock\s+(?:network\s+delay\s+\(.+?\)\s+\d+\.\d+\D+\d+\.\d+|reconvergence|uncertainty)/s;
 
  my ($spclk_info)	= ($path =~ /($spclk_info_re)/);
  if ($spclkname eq $epclkname) {
    pos($path) 	= 0;
    $path	=~ /$data_arrival_time_re/g;
  }
 
  my  ($epclk_info)	= ($path =~ /$epclk_info_re/g);
 
  my ($spclk_edge)	= $spclk_info && ($spclk_info =~ /$xpclk_edge/);
  my ($epclk_edge)	= $epclk_info && ($epclk_info =~ /$xpclk_edge/);
 
  $spclk_edge		= $spclk_edge ? $spclk_edge : "-";
  $epclk_edge		= $epclk_edge ? $epclk_edge : "-";
 
  $spclkname 		= $spclk_info ? $spclkname : "-";
  $epclkname 		= $epclk_info ? $epclkname : "-";
  
 
  unless ($optional{clock_path}) {
   # Slack info
   my ($slackv)		= $slack && ($slack =~ /$slackv_re/);
 
   pos($path) = 0;
   my ($arrival_time)       = $path =~ /$data_arrival_time_re/;
   my ($required_time)      = $path =~ /$data_required_time_re/;
   my ($library_sh_time)    = $path =~ /$library_sh_time_re/;
   my ($clock_uncertainty)  = $path =~ /$clock_uncertainty_re/;
 
   unless ($path_num >= 0) {
                      # mode  corner  spname  spclkname  spclk_edge  epname  epclkname  epclk_edge        slackv/datv       iomode filepath    path_idx";
    my $query_string = "file=$reportfile&path_index=$path_idx";
    print {*$OUTFILE} "$mode $corner $spname $spclkname $spclk_edge $epname $epclkname $epclk_edge ". ($slackv || $arrival_time)." $iomode $reportfile $path_idx $query_string $arrival_time\n"

   } elsif ($path_num == $path_idx) {
      return {info => {mode       		=> $mode,
         	       corner     		=> $corner,
         	       iomode     		=> $iomode,
         	       startpoint	 	=> $spname,
            	       startpoint_clock	 	=> $spclkname,
            	       startpoint_clock_edge 	=> $spclk_edge,
            	       endpoint	 		=> $epname,
            	       endpoint_clock	 	=> $epclkname,
            	       endpoint_clock_edge 	=> $epclk_edge,
            	       slack	 		=> ($slackv || $datv),
            	       reportfile		=> $reportfile,
         	       path_index 		=> $path_idx,
         	       query_string 		=> $query_string,
         	       arrival_time		=> $arrival_time,
         	       required_time		=> $required_time,
         	       library_setup_hold_time  => $library_sh_time,
         	       clock_uncertainty	=> $clock_uncertainty,
                      },

              points                 => $nodes_list,
              path                   => $path
           } 
   }
  } else {
   # Clock path information
   my $spclk_nodes	= path_nodes_xt($spclk_info); 
   my $epclk_nodes	= path_nodes_xt($epclk_info); 

   push @clock_paths, {startpoint_clock_path  => $spclk_info,
                       startpoint_clock_nodes => $spclk_nodes,
                       endpoint_clock_path    => $epclk_info,
                       endpoint_clock_nodes   => $epclk_nodes,
            	       reportfile	      => $reportfile,
		       path_index	      => $path_idx
   		      }
  }
  
  ++$path_idx
 }

 unless ($path_num >= 0) {
  unless ($optional{clock_path}) {
   print {*$OUTFILE} "\n";
   close *$OUTFILE;
  } else {
   return \@clock_paths
  }
 } else {
  # path w/ $path_num as index not found
  return  undef
 }

}


sub path_nodes_xt {
 my ($str_data) = @_; 

 return undef unless $str_data;

 my $node_re = qr#(\S+)\s+\(\s*[\w\d_]+\s*\)\D+\d+\.\d+\D+(\d+\.\d+)#o;  

 my $ELEMT_COUNT = 2;
 my $cidx        = 0;
 my @a2d;
 my @atmp;
 foreach ($str_data =~ /$node_re/g) {
  if ($cidx % 2 == 0 && int($cidx/2)) {
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
