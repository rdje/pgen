#----------------------------------------------------------------------------------------------
# Author	: Richard DJE
#
# Description	: get_tpinfo
#
#----------------------------------------------------------------------------------------------
# Property of Texas Instruments - For  Unrestricted Internal Use  Only.
# Unauthorized reproduction and/or distribution is strictly prohibited.  This
# product is protected under copyright law and trade secret law as an unpublished
# work. (C) Copyright 2005 Texas Instruments. All rights reserved.
#----------------------------------------------------------------------------------------------
package provide	stapack 1.0

proc get_tpinfo	{args} {
 getoptions gt_arg $args

 array set gt_arg_so $gt_arg(SO)

 if {[info exists gt_arg_so(help)] || ![info exists gt_arg(SA)]} {
  echo "usage: get_tpinfo <TimingPaths_Collection> \[--unique\] \[--quiet\] \[--debug\] \[--help\]"
  return {}
 }

 set quiet	[info exists gt_arg_so(quiet)]
 set debug	[info exists gt_arg_so(debug)]

 if {$debug} {echo "(get_tpinfo) Entering"}
 
 set tpaths	[lindex $gt_arg(SA) 0]

 set info_section		""
 set info_i			{}
 set info_i_uncons		{}
 set info_i_violated		{}
 set info_i_meet		{}
 set info_o			{}
 set info_o_uncons		{}
 set info_o_violated		{}
 set info_o_meet		{}
 set info_int			{}
 set info_int_uncons		{}
 set info_int_violated		{}
 set info_int_meet		{}
 set info_c			{}
 set info_c_uncons		{}
 set info_c_violated		{}
 set info_c_meet		{}

 set nopath			{}

 array set maxpath		{}
 array set worst_path		{}
 array set allpaths		{}

 array set already_seen		{}
 
 set worstslack			0
 set slacksum			0
 set uncount			0
 set i_uncount			0
 set o_uncount			0
 set int_uncount		0
 set c_uncount			0

 # Violating counts
 set i_violated_count		0
 set o_violated_count		0
 set int_violated_count		0
 set c_violated_count		0

 # Paths that meet their constraints
 set i_meet_count		0
 set o_meet_count		0
 set int_meet_count		0
 set c_meet_count		0
 
 foreach_in_collection mytp $tpaths {
  #-----------------------------------------------------
  # Note:
  # Start/End Points and Start/End point Clocks fields
  # are Collection handles/references.
  #-----------------------------------------------------
 
  # Start Point
  set	sp		[get_attribute $mytp startpoint]
  set	sptype		[get_attribute $sp object_class]
  set	spname		[expr {$sp != "" ? [get_attribute $sp full_name] : "Unknown"}]
 
  # In some cases we might want to filter-out any timing paths starting at a clock input port
  
  # Start Point Clock
  set 	spclk		[get_attribute -q $mytp startpoint_clock]
  # If the startpoint clock name is defined. it may be the *input default clock*
  # automatically used by PT, or user-defined (create_clock)
  set  spclkn		[get_attribute $spclk full_name]
  set	spclkname	[expr {$spclk != "" && ![regexp {(?i)\(input\s+port\s+clock\)} $spclkn] ? $spclkn : "undefined"}]
  
  # Start Point Input delay Value
  set 	spidv_p		[get_attribute -q $mytp startpoint_input_delay_value]
  set 	spidv		[expr {$spidv_p != "" ? $spidv_p : "-"}]

  # Data Path Delay
  set	dpd		[get_attribute $mytp arrival]
  set	slack		[get_attribute $mytp slack]
 
  # End Point
  set	ep		[get_attribute $mytp endpoint]
  set	eptype		[get_attribute $ep object_class]
  set	epname		[expr {$ep != "" ? [get_attribute $ep full_name] : "Unknown"}]
 
  # End Point Clock
  set	epclk		[get_attribute -q $mytp endpoint_clock]
  set	epclkname	[expr {$epclk != "" ? [get_attribute $epclk full_name] : "undefined"}]
  
  # End Point Input delay Value
  set	epodv_p		[get_attribute -q $mytp endpoint_output_delay_value]
  set 	epodv		[expr { $epodv_p != "" ? $epodv_p : "-"}]

  # Filtering TIMING PATHs  With NO information
  if {$spclkname == "undefined" && $epclkname == "undefined" && $spname == $epname && [get_attribute $sp object_class] == "port"} {
   set ucv		[get_attribute -q $sp user_case_value]
   lappend nopath	[list $spname [get_attribute -q $sp direction] [expr {$ucv != "" ? $ucv : "-"}]]
   continue
  }

  set	spclklat 	[expr {$spclk != "" ? [get_attribute -q $mytp startpoint_clock_latency] : 0.0}]
  set	epclklat 	[expr {$epclk != "" ? [get_attribute -q $mytp endpoint_clock_latency]   : 0.0}]
 
  #----------------------------------------------------------------------------------
  # Dispatching the TP info depending on the object class of the startpoint and endpoint.
  #----------------------------------------------------------------------------------
  if {$sptype == "port" && $eptype == "pin"} {
   set info_section 	info_i
  } elseif {$sptype == "pin" && $eptype == "port"} {
   set info_section	info_o
  } elseif {$sptype == "pin" && $eptype == "pin"} {
   set info_section	info_int
  } else { 
   # port --> port , i.e, combinational paths
   set info_section	info_c
  }
 
  set pathtype 	[expr {$info_section == "info_i" ? "I" : "O"}]
  set tpentry	[list $spname $epname $spclkname $epclkname $dpd $spclklat $epclklat 				   \
  		[get_attribute -q $sp direction] [get_attribute -q $ep direction] $pathtype $slack $spidv $epodv ]  

  if {[info exists gt_arg_so(unique)]} {
   # This temp var and the following 'info exists' are for filtering any
   # duplicate entries, Yes such entries exist.
   set	tmpvar	[join  $tpentry ,]
   if {![info exists already_seen($tmpvar)]} {
    # Here is a new tpentry
    set already_seen($tmpvar)	1
    lappend $info_section $tpentry
   
    # Keeping the Worst path for each $startpoint and $endpoint_clock triplet
    # for all paths
    # I added this logic because I noticed that for port 2 pin paths, sometimes one can
    # see more than one path returned by *get_timing_paths*, so you need to have access
    # to the worst of these paths.
    # I did not see such a thing for pin 2 port paths.
    if {[info exists maxpath($spname,$epname,$spclkname,$epclkname)]} {
      if {[expr {$maxpath($spname,$epname,$spclkname,$epclkname) < $dpd}]} {
       # Keeping only the Worst path
       set maxpath($spname,$epname,$spclkname,$epclkname)		$dpd
       set worst_path($spname,$epname,$spclkname,$epclkname)	$tpentry
      }
    } else {
     set maxpath($spname,$epname,$spclkname,$epclkname)		$dpd
     set worst_path($spname,$epname,$spclkname,$epclkname)	$tpentry
    }
   }
  } else {
   if {[regexp {^-} $slack]} {
    # Constrainted but Violating path

    if {$info_section == "info_i"}   {
     set info_section	info_i_violated
     incr	i_violated_count
    }

    if {$info_section == "info_o"}   {
     set info_section	info_o_violated
     incr	o_violated_count
    }

    if {$info_section == "info_int"}   {
     set info_section	info_int_violated
     incr	int_violated_count
    }

    if {$info_section == "info_c"}   {
     set info_section	info_c_violated
     incr	c_violated_count
    }

    # Worst negative slack and Total
    if {$slack < $worstslack} {set worstslack $slack}
    set slacksum  [expr {$slacksum + $slack}]

    if {$debug} {echo "slack($slack) slacksum($slacksum) worstslack($worstslack)"}
   } elseif {[regexp {(?i)infinity} $slack]} {
    # Non constrainted path

    incr	uncount

    if {$info_section == "info_i"}   {
     set info_section	info_i_uncons
     incr	i_uncount
    }

    if {$info_section == "info_o"}   {
     set info_section	info_o_uncons
     incr	o_uncount
    }

    if {$info_section == "info_int"}   {
     set info_section	info_int_uncons
     incr	int_uncount
    }

    if {$info_section == "info_c"}   {
     set info_section	info_c_uncons
     incr	c_uncount
    }
   } else {
    # Constrainted and positive slack path
    if {$info_section == "info_i"}   {
     set info_section	info_i_meet
     incr	i_meet_count
    }

    if {$info_section == "info_o"}   {
     set info_section	info_o_meet
     incr	o_meet_count
    }

    if {$info_section == "info_int"}   {
     set info_section	info_int_meet
     incr	int_meet_count
    }

    if {$info_section == "info_c"}   {
     set info_section	info_c_meet
     incr	c_meet_count
    }
   }

   lappend $info_section $tpentry
  }

  # All returned paths for a given $startpoint/$endpoint_clock pair are kept here
  lappend allpaths($spname,$epname,$spclkname,$epclkname)	$tpentry	
 }


 # Preparing output data
 array set retinfo   [list i_violated 	  	$info_i_violated]
 array set retinfo   [list o_violated 	  	$info_o_violated]
 array set retinfo   [list int_violated 	$info_int_violated]
 array set retinfo   [list c_violated	  	$info_c_violated]

 array set retinfo   [list i_meet 	  	$info_i_meet]
 array set retinfo   [list o_meet 	  	$info_o_meet]
 array set retinfo   [list int_meet 		$info_int_meet]
 array set retinfo   [list c_meet	  	$info_c_meet]

 array set retinfo   [list i_noconstraint 	$info_i_uncons]
 array set retinfo   [list o_noconstraint 	$info_o_uncons]
 array set retinfo   [list int_noconstraint 	$info_int_uncons]
 array set retinfo   [list c_noconstraint 	$info_c_uncons]

 array set retinfo   [list nopath	  	$nopath]

 array set retinfo   [list slacksum	  	$slacksum]
 # Violating constrainted paths count
 array set retinfo   [list count	  	[expr {$i_violated_count + $o_violated_count + $int_violated_count + $c_violated_count}]]
 array set retinfo   [list wns		  	$worstslack]

 # Unconstrained paths count 
 array set retinfo   [list uncount	  	$uncount]
 array set retinfo   [list i_uncount	  	$i_uncount]
 array set retinfo   [list o_uncount	  	$o_uncount]
 array set retinfo   [list int_uncount	  	$int_uncount]
 array set retinfo   [list c_uncount	  	$c_uncount]

 array set retinfo   [list i_violated_count	$i_violated_count]
 array set retinfo   [list o_violated_count	$o_violated_count]
 array set retinfo   [list int_violated_count	$int_violated_count]
 array set retinfo   [list c_violated_count	$c_violated_count]

 array set retinfo   [list i_meet_count		$i_meet_count]
 array set retinfo   [list o_meet_count		$o_meet_count]
 array set retinfo   [list int_meet_count	$int_meet_count]
 array set retinfo   [list c_meet_count		$c_meet_count]

 array set retinfo   [list maxpath 	  	[array get maxpath]]
 array set retinfo   [list worst_path     	[array get worst_path]]
 array set retinfo   [list allpaths 	  	[array get allpaths]]

 return [array get retinfo]
}
