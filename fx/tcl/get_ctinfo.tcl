#----------------------------------------------------------------------------------------------
# Author	: Richard DJE
#
# Description	: 
#
#----------------------------------------------------------------------------------------------
# Property of Texas Instruments - For  Unrestricted Internal Use  Only.
# Unauthorized reproduction and/or distribution is strictly prohibited.  This
# product is protected under copyright law and trade secret law as an unpublished
# work. (C) Copyright 2005 Texas Instruments. All rights reserved.
#----------------------------------------------------------------------------------------------
package provide	stapack 1.0

proc	get_ctinfo {args} {
 getoptions gc_args $args

 array set gc_args_so $gc_args(SO)
 
 if {[info exists gc_args_so(help)]} {
  echo "usage: get_ctinfo \[--ilr\] \[--quiet\] \[--help\] \[--debug\]"
  return {}
 }
 

 # Options
 set debug	[info exists gc_args_so(debug)]
 set quiet	[info exists gc_args_so(quiet)]
 set with_ilr	[info exists gc_args_so(ilr)]

 # I need to create a hash table containing the name of 
 # clock objects together with the full_name of the pin/port to
 # which the clock has been applied
 array set clockinfo			{}
 set non_propagated_clock		0
 array set is_nonpropagated_clock	{}
 # Important Note: 
 #    I found that one way to find clock sources that are
 #    not expanded, most probably due to a missing Master Clock 
 #    issue (definition or whatever), the *period* attribute 
 #    is NOT defined, i.e, it is set to the empty string.
 array set not_expanded			{}
 array set has_no_sources		{}
 array set has_one_source		{}
 set	longestname	0
 set 	smallest_period	10000
 foreach_in_collection myclkobj [get_clocks *] {
   set clkname	[get_attribute -q $myclkobj full_name]
   set period	[get_attribute -q $myclkobj period]

   set len	[string length $clkname]
   if {$len > $longestname} {set longestname $len}

   # Is that clock propagated, Because it should be.
   if {[get_attribute $myclkobj propagated_clock] == "false"} {
    if {!$quiet} {echo "\[get_ctinfo\] -W- Clock '$clkname' is not propagated !"}
    incr non_propagated_clock
    array set is_nonpropagated_clock [list $clkname 1]
   }
   

   # Now I should retrieve the pin(s)/port(s) to which it is applied
   set	pinportcol	[get_attribute -q $myclkobj sources]
   set	colsize		[sizeof_collection $pinportcol]
   if {$colsize > 1} {
    if {!$quiet} {echo "\[get_ctinfo\] -W- Clock '$clkname' associated with several pin/port."}
   } elseif {$colsize == 0} {
    if {!$quiet} {echo "\[get_ctinfo\] -W- Clock '$clkname' has no generated source(s) (virtual-clock ?)."}
    array set has_no_sources [list $clkname 1]
   } else {
    array set has_one_source [list $clkname 1]
   }

   if {$colsize} {
    if {$period != "" && $period < $smallest_period} {
      set smallest_period $period
    }
   }

   array set clockinfo [list  $clkname $pinportcol]
 }

 if {!$quiet} {echo "\[get_ctinfo\] -I- Found ($non_propagated_clock) non-propagated clocks."}
 # We should not continue if at least one clock source is not propagated but have sources
 set should_I_quit	0
 foreach myclk [array names clockinfo] {
  # Currently I will be trying to find the primary clocks only when
  # the clock object only has one clock source.
  # Is that clock expanded
  if {[info exists has_one_source($myclk)] && [get_attribute -q -c clock $myclk period] == ""} {
   if {!$quiet} {echo "\[get_ctinfo\] -W- Clock '$myclk' is not expanded !"}

   # So, let's try to identify its potential primary clock source
   # If more than one are found, then I couldn't do anything but
   # report that fact.
   #
   # let's do it by using the cell collection, because *get_primary_clocks* does not
   # currently support output pins/ports as starting point yet.
   set primclock	[get_primary_clocks [get_cells -q -o $clockinfo($myclk)]]
   set primcnt	[sizeof_collection $primclock]
   if {$primcnt == 1} {
    # Got it !
    #
    # So let's defined it (blindy) a clock source and propagate it.
    # before doing just if it is not already attached to a clock
    #
    # I said blindy, because I could have check for the existence
    # of a previous clock declaration on it, but...
    if {1} {
     create_clock -p $smallest_period $primclock
     set primname 	[get_attribute -q $primclock full_name]
     set_propagated_clock	$primname	
     array set clockinfo [list  $primname $primclock]
     if {!$quiet} {echo "\[get_ctinfo\] -I- Attaching a clock source on '[get_attribute -q $primclock full_name]'."}
    }
   } elseif {$primcnt > 1} {
    if {!$quiet} {echo "\[get_ctinfo\] -W- Clock '$myclk' has MORE THAN ONE potential primary clock source !"}
    query_object $primclock
   } else {
    if {!$quiet} {echo "\[get_ctinfo\] -W- Clock '$myclk' has NO potential primary clock source !"}
   }

   if {0} {set	should_I_quit 1}
  }

  if {[info exists is_nonpropagated_clock($myclk)] && ![info exists has_no_sources($myclk)]} {
    if {!$quiet} {echo "\[get_ctinfo\] -I- Propagating clock '$myclk'"}
    set_propagated_clock $myclk
    array unset is_nonpropagated_clock $myclk

    if {0} {set	should_I_quit 1}
  }

 }


 if {$should_I_quit} {return}

 # This hash table is for storing the max/min insertion delay for a specific clock
 array	set	maxmin_cid	{}
 # Initializing that Hash
 foreach myclkname	[array names clockinfo] {
  array	set maxmin_cid [list $myclkname [list 0 0]]
 }
 
 # For each clock object I need to retrieve all of the clock-pins they drive.
 # On all of that set determine the shortest and longest insertion.
 foreach myclkname	[array names clockinfo] {
  if {[info exists is_nonpropagated_clock($myclkname)]} {
    if {$debug} {echo "\[get_ctinfo\] -W- Skipping non-propagated clock '$myclkname'"}
    continue
  }

  if {!$quiet} {echo "\[get_ctinfo\] -I- Retrieving clock-tree insertion delays info for '$myclkname'.."}
  # Calling *get_foilr* to retrieve all registers clock-pins and out/inout ports
  # seen in the (transitive-)fanout of $myclkname. To be retain those returned flops
  # should directly communicate with the outside world using either their data pin 
  # or output.
  
  # Building *get_foilr* command line
  if {$with_ilr} {
   set get_fo_ilr_cmd	"get_foilr  $clockinfo($myclkname)"
  } else {
   set get_fo_ilr_cmd	"get_for  --nomuxsel --showmux $clockinfo($myclkname)"
  }

  foreach myopt [array names gc_args_so] {
   append get_fo_ilr_cmd " --$myopt"
  }
  
  # Executing that command line
  #
  # Of course, of course, the quality of this filtering heavily depends on the quality PT database
  # regarding attributes, but you already know the story.
  set	clkpins_ports	[filter_collection [eval $get_fo_ilr_cmd] "is_clock_pin == true || object_class == port"]

  # For all of these clock-pins get the their insertion delay
  # 
  # As you may already know it, If we are doing a get_timing_paths (report_timing)
  # from a generated clock the value that is returned includes the generated clocks
  # source latency from the primary clock used to create that generated.
  #
  # We will be using either *-rise_to* or *-fall_to* depending on the edge type of the
  # clock-pin.
  set	firstround	1
  set	cid	{}
  foreach_in_collection myclkpin_port $clkpins_ports {
   if {$debug} {echo "\[get_ctinfo\] -I- From '[get_attribute $clockinfo($myclkname) full_name]'"}
   if {[get_attribute -q $myclkpin_port is_rise_edge_triggered_clock_pin] == "true"} {
     if {$debug} {echo "\[get_ctinfo\]     to   '[get_attribute $myclkpin_port full_name]' (Rising Edge)"}
     set cid	[get_timing_paths -from $clockinfo($myclkname) -rise_to $myclkpin_port]
   } elseif {[get_attribute -q $myclkpin_port is_fall_edge_triggered_clock_pin] == "true"} {
     if {$debug} {echo "\[get_ctinfo\]     to   '[get_attribute $myclkpin_port full_name]' (Falling Edge)"}
     set cid	[get_timing_paths -from $clockinfo($myclkname) -fall_to $myclkpin_port]
   } else {
     if {$debug} {echo "\[get_ctinfo\]     to   '[get_attribute $myclkpin_port full_name]' (Port)"}
     set cid	[get_timing_paths -from $clockinfo($myclkname) -to $myclkpin_port]
   }

   # We should not blindy use $cid, it may be empty.
   if {$cid != ""} {
    # Cool, we do have a timing_path.
    #
    # Is it a max, a min ?
    set	cur_max	 [lindex $maxmin_cid($myclkname) 0]
    set	cur_min	 [lindex $maxmin_cid($myclkname) 1]
    
    # Now get the $cid arrival time, which I said above should contain the clock source latency
    # I manually checked and even when the endpoint is port, the *arrival time* report does contain
    # the clock source latency.
    set	cid_arrival	[get_attribute $cid arrival]
    if {$cid_arrival > $cur_max } {
      set cur_max 	$cid_arrival

      if {$firstround} {set cur_min 	$cid_arrival}

      if {$debug} {echo "\[get_ctinfo\] -DEBUGnewmax:$myclkname- New Max = \[$cid_arrival\]"}
    } elseif {$cid_arrival < $cur_min} {
      set cur_min 	$cid_arrival
      if {$debug} {echo "\[get_ctinfo\] -DEBUGnewmin:$myclkname- New min = \[$cid_arrival\]"}
    }
   } else {
    set clkpin_port_dir	 [get_attribute -q $myclkpin_port direction]
    if {!$quiet} {echo [format "\[get_ctinfo\] -W- No timing_path from clock '%s' to %s '%s'." $myclkname \
    									[expr {$clkpin_port_dir == "in" ? "clock-pin" : "port"}] \
    									 [get_attribute -q $myclkpin_port full_name]]}
    continue
   }

   # Storing those max/min numbers back.
   #
   # It's a pity the Tcl *lset* command is not available in PT
   set	maxmin_cid($myclkname)	[list $cur_max $cur_min]
   
   set	firstround	0
  }
 }


 # Getting all these max/min numbers out.
 if {!$quiet} {echo ""}
 array	set	outstr	{}
 foreach myclkname	[array names clockinfo] {
  set	last_max	 [lindex $maxmin_cid($myclkname) 0]
  set	last_min	 [lindex $maxmin_cid($myclkname) 1]
  if {[info exists is_nonpropagated_clock($myclkname)]} {
   if {!$quiet} {echo [format "\[get_ctinfo\]     %-${longestname}s :     -    /    -" $myclkname]}
   array set	outstr [list $myclkname [list "-" "-"]]
  } else {
   set	skew	[expr {($last_max - $last_min)*1000}]
   if {!$quiet} {echo [format "\[get_ctinfo\]     %-${longestname}s : %5.3f ns / %6.2f ps" $myclkname $last_min $skew]}
   array set	outstr [list $myclkname  [list $last_min $skew]]
  }
 }

 # returning the hash info
 return [array get outstr]
}
