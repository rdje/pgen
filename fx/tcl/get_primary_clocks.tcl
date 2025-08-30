#----------------------------------------------------------------------------------------------
# Author	: Richard DJE
#
# Description	: *get_primary_clocks* goal is to retrieve the list of in/inout port(s)
#		  or out/inout pin(s) of sequential cells belonging to the transitive
#		  fanin of a given pin or port that may reasonably be considered as 
#		  potential primary clock source(s).
#		  
#		  It takes only one argument which is the specification of in/inout pin,
#		  out/inout port. It does not need, thanks to Beatrice Remarks, to be a
#		  clock-pin.
#		  Also Thanks to Beatrice Beta-Testing I added a complete support (I think)
#		  for *inout* pins/ports. These types of object were not previously supported.
#----------------------------------------------------------------------------------------------
# Property of Texas Instruments - For  Unrestricted Internal Use  Only.
# Unauthorized reproduction and/or distribution is strictly prohibited.  This
# product is protected under copyright law and trade secret law as an unpublished
# work. (C) Copyright 2005 Texas Instruments. All rights reserved.
#----------------------------------------------------------------------------------------------
package provide	stapack 1.0

proc   get_primary_clocks  {args} {

 getoptions gpc_arg_s $args

 array set opt_so $gpc_arg_s(SO)

 if {[info exists opt_so(help)]} {
  echo "usage: get_primary_clocks \[--quiet\] \[--debug\] \[--help\] <object_spec>"
  return {}
 }

 set	debug	0
 set	quiet	0

 if {[info exists opt_list(debug)]} {set debug 1}
 if {[info exists opt_list(quiet)]} {set quiet 1}

 set gpc_arg	[lindex $gpc_arg_s(SA) 0]

 array set	get_primary_clocks_visited {}
 array set	get_primary_clocks_warned {}
 set state	"PINPORT_NAME"
 while 1 {
  if {$debug} {echo "current state = \[$state\]"}

  if {$state == "PINPORT_NAME"} {
   # Is it a pin/port ?
   set pinobj  	[get_pins  -q -f "direction == in  || direction == inout" $gpc_arg]
   set portobj 	[get_ports -q -f "direction == out || direction == inout" $gpc_arg]
   set pinports 	[add_to_collection $pinobj $portobj]
   
   set state  "PINPORT_HANDLING"
   continue
  }

  if {$state == "PINPORT_HANDLING"} {
   if {$pinports != ""} {
     # We have pins/ports
     set	returnv	[ce_get_primary_clocks $pinports get_primary_clocks_visited get_primary_clocks_warned opt_so]
     return $returnv
   } else {
    set state  "CELL_NAME"
    continue
   }
  }

  if {$state == "CELL_NAME"} {
   # Is it a cell ?
   set cellcol	[get_cells -q $gpc_arg]
   if {$cellcol != ""} {
    set state  "CELL_COLLECTION"
    continue
   } else {
    # Is it a collection ?
    set cellcol	[get_attribute -q $gpc_arg object_class]
    if {$cellcol != ""} {
     # We do have a collection
     # 
     # But what kind ?
     set state  "WHAT_COLLECTION"
     continue
    } else {
     # Is it a clock ?
     set state "CLOCK_NAME" 
     continue
    }
   }
  }

  if {$state == "CLOCK_NAME"} {
  }
  
  if {$state == "WHAT_COLLECTION"} {
   if {$cellcol == "cell"} {
    set cellcol $gpc_arg
    set state "CELL_COLLECTION"
    continue
   } else {
    echo "\[get_primary_clocks\] -E- Currently only cell collections are supported !"
    return {}
   }
  }

  if {$state == "CELL_COLLECTION"} {
   # So now we should retrieve all of the cell out/inout pins/ports
   set cello	[get_pins -q -fi "direction == out || direction == inout" -o $cellcol]
   set cello	[add_to_collection -unique $cello [get_ports -q -fi "direction == out || direction == inout" -o $cellcol]]

   if {$cello != ""} {
    # Now get all *from* pin/port objects having a timing_arc to those
    # *to* pin/port objects.
    set fromsetiarcs	{}
    foreach_in_collection mytobj	$cello {
      set tiarcs	[get_timing_arcs -fi "is_disabled == false" -to $mytobj]
      set fromsetiarcs	[add_to_collection -unique $fromsetiarcs $tiarcs]
    }

    # Retrieving From in/inout pins/ports
    set fromset	{}
    foreach_in_collection mytiarc $fromsetiarcs {
     set fromset [add_to_collection -unique $fromset [get_attribute -q $mytiarc from_pin]]
    }

    # At this point we should have collected all useable from_pin(s) of the above cell.
    set returnv	{}

    # Is it combinational or sequential ?
    if {[get_attribute -q $cellcol is_combinational]} {
     # Now we will be calling *ce_get_primary_clocks* on each of the $fromset entry
     foreach_in_collection myfrompin $fromset	{
      set returnv [add_to_collection -unique [ce_get_primary_clocks $myfrompin get_primary_clocks_visited get_primary_clocks_warned  opt_so]]
     }
    } else {
     # Based on Synopsys *Object Attributes* an object is either 
     # combinational or sequential.
     #
     # so here we must have a sequential cell.
     #
     set fromset_sz	[sizeof_collection $fromset]
     if {$fromset_sz == 1} {
      # There is only one From_pin having a timing_arc to one of the To_pins, so let's
      # propagate through it by calling *ce_get_primary_clocks*.
      ##foreach_in_collection myfrompin $fromset	{
       set returnv [ce_get_primary_clocks $fromset get_primary_clocks_visited get_primary_clocks_warned   opt_so]
      ##}
     } elseif {$fromset_sz == 0} {
      echo "\[get_primary_clocks\] -W- Sorry but cell '[get_attribute -q $cellcol full_name]' has NO timing_arcs leading to its out/inout pins/ports !"
     } else {
       # Finding all From clock pins/ports, and then call *ce_get_primary_clocks* on each of them
       set from_clk_pins	[filter_collection $fromset "is_clock_pin == true"]
       set from_clk_pins_sz	[sizeof_collection $from_clk_pins]
       if {$from_clk_pins_sz == 0} {
        # Is it a gsx0 register anyway ?
	if {[is_gsx0_reg $cellcol]} {
	 # Y-e-s
	 # In that case, we should have the clock pin be named '[get_attribute -q $cellcol full_name]/CLK'
	 # let's check
	 set check_clkname	"[get_attribute -q $cellcol full_name]/CLK"
	 set clkname		[get_pins -q $check_clkname]
	 if {$clkname != ""} {
	  # I guessed right, let's propagate !
          set returnv [ce_get_primary_clocks $fromset get_primary_clocks_visited get_primary_clocks_warned  opt_so]
	 } else {
          echo "\[get_primary_clocks\] -W- That's weird ! gsx0 register '[get_attribute -q $cellcol full_name]' doesn't have its clock-pin named *CLK* !"
          echo "\[get_primary_clocks\]     Also the clock-pin *is_clock_pin* attribute is not set !!"
	 }
	} elseif {[is_gsx0_cgc $cellcol]} {
	 # Y-e-s
	 # In that case, we should have the clock-gating clock-pin be named '[get_attribute -q $cellcol full_name]/CLKIN'
	 # let's check
	 set check_clkname	"[get_attribute -q $cellcol full_name]/CLKIN"
	 set clkname		[get_pins -q $check_clkname]
	 if {$clkname != ""} {
	  # I guessed right, let's propagate !
          set returnv [ce_get_primary_clocks $fromset get_primary_clocks_visited get_primary_clocks_warned  opt_so]
	 } else {
	  if {!$quiet} {
           echo "\[get_primary_clocks\] -W- That's weird ! gsx0 Clock-Gating Cell '[get_attribute -q $cellcol full_name]' doesn't have its clock-pin named *CLKIN* !"
           echo "\[get_primary_clocks\]     Also the clock-pin *is_clock_pin* attribute is not set !!"
	  }
	 }
	} else {
	 if {!$quiet} {
          echo "\[get_primary_clocks\] -W- Can't propagate through cell '[get_attribute -q $cellcol full_name]' !"
          echo "\[get_primary_clocks\]     More than one timing_arcs from in->out, but no clocks defined on inputs."
          echo "\[get_primary_clocks\]     cell_ref_name=\[[get_attribute -q $cellcol ref_name]\]"
	 }
	}
       } else {
        if {$debug} {echo "\[get_primary_clocks\] -I- Found ($from_clk_pins_sz) input clocks on cell '[get_attribute -q $cellcol full_name]'."}
	foreach_in_collection myfromclk $from_clk_pins	{
         set returnv [add_to_collection -unique $returnv [ce_get_primary_clocks $myfromclk get_primary_clocks_visited get_primary_clocks_warned  opt_so]]
        }
       }
     }
    }

   } else {
     if {!$quiet} {echo "\[get_primary_clocks\] -W- Sorry but cell '[get_attribute -q $cellcol full_name]' has NO  out/inout pins/ports !"}
   }

   # job done.
   return $returnv
  }

 }

  #echo "\[get_primary_clocks\] -E- Only in/inout pins of cell or out/inout ports are supported !"
}


# Core Engine of *get_primary_clocks*
proc	ce_get_primary_clocks	{gpc_arg visited_hash warned_hash optlist}	{
upvar	$optlist	opt_list
upvar	$visited_hash	visitedhash
upvar	$warned_hash	warnedhash

 #default values
 set	debug	0
 set	quiet	0

 if {[info exists opt_list(debug)]} {set debug 1}
 if {[info exists opt_list(quiet)]} {set quiet 1}


 if {$debug} {echo "\[get_primary_clocks\] -I- Entering '[get_attribute -q $gpc_arg full_name]'"}
 # Here I assume that the argument is a pin/port object
 # that is a collection of only one pin/port object.

 # Has $gpc_arg already been visited ?
 if {[info exists visitedhash([get_attribute -q $gpc_arg full_name])]} {
  if {[info exists warnedhash([get_attribute -q $gpc_arg full_name])] == 0} {
   if {!$quiet} {
    echo "\[get_primary_clocks\] -WW- Potential Loop detected on clock-pin [get_attribute -q $gpc_arg full_name]"
    echo "\[get_primary_clocks\] -I-  *set_case_analysis* or *set_disable_timing* may be your friends."
   }
   
   array set warnedhash	[list [get_attribute -q $gpc_arg full_name] 1]
  }

  return {}
 }
 
 # Building *get_fanin* command line
 set getfanin_cmd	"get_fanin -nomuxsel -showmux $gpc_arg"	
 foreach myopt [array names opt_list] {
  append getfanin_cmd " --$myopt"
 }

 # We should call *eval* on that command line
 set clkfanin	[eval $getfanin_cmd]

 # Important: 
 # Some clock-tree may contain some kinda loops. That is
 # a flop1 clock-pin may combinationally depend on its Q1 output
 # and also on the Q2 of flop2, whose clock-pin depends on Q1 and Q2
 # and so on.
 #
 # The way to break those I guess is to use *case_analysis* or
 # *disable_timing*.
 #
 # So for the tool to detect those loop(s) I need to mark $gpc_arg has visited
 # once processed.

 #set_user_attribute -q  $gpc_arg get_primary_clocks_visited	1
 array set visitedhash	[list [get_attribute -q $gpc_arg full_name] 1]

 set clktree_pinsports	{}
 
 if {$clkfanin == ""} {
  # No final pin or port were found in the fanin of $gpc_arg
  if {!$quiet} {
   echo "\[get_primary_clocks\] -W- No sequential output pin(s) and no inport port(s) were found in the fanin of pin [get_attribute -q $gpc_arg full_name]"
  }

  return {}
 } else {
  # Ok we are lucky, let's iterate over this collection and filter out 
  # latch/three-state output pins and retain only registers and sequential
  # cells w/ non-disabled timing_arcs that enable the propagation through them.
  set	goodpins_ports	{}
  foreach_in_collection myobj	$clkfanin {
   if {[info exists visitedhash([get_attribute -q $myobj full_name])]} {continue}

   if {[get_attribute -q $myobj object_class] == "port"} {
    if {[get_attribute -q $myobj direction] == "inout"} {
     if {!$quiet} {
      echo "\[get_primary_clocks\] -I- [get_attribute -q $myobj direction] port '[get_attribute -q $myobj full_name]' is a potential primary clock source."
     }
    } else {
     if {!$quiet} {
      echo "\[get_primary_clocks\] -I- [get_attribute -q $myobj direction]put port '[get_attribute -q $myobj full_name]' is a potential primary clock source."
     }
    }

    set clktree_pinsports [add_to_collection -u $clktree_pinsports $myobj]

    array set visitedhash	[list [get_attribute -q $myobj full_name] 1]
   } else {
    # Since $myobj is a pin we should check if it is the ouput of a latch or three-state.
    # I know, I know, three-state buffer are never used on clock-tree's it's just for completeness
    set pincell	[get_cells -q -o $myobj]
    if {[get_attribute -q $pincell is_positive_level_sensitive] ||
    	[get_attribute -q $pincell is_negative_level_sensitive] ||
	[is_gsx0_lat $pincell]} {
	# We have latch's pin
	if {!$quiet} {echo "\[get_primary_clocks\] -W- Latch '[get_attribute -q $pincell full_name]' found on the clock-tree."}
	continue
    } elseif {[get_attribute -q $pincell is_three_state]} {
     # I can't believe it ! A three state driver on a clock-tree.
     if {!$quiet} {echo "\[get_primary_clocks\] -WW- Three-state driver '[get_attribute -q $pincell full_name]' found on the clock-tree."}
     continue
    } else {
     # So here we are. $myobj is not a latch nor a three-state, so let's verify its timing arcs.
     # In fact we need to retrieve those arcs that end on $myobj and are not disabled
     set myobj_tiarcs	[get_timing_arcs -fi "is_disabled == false" -to $myobj]

     if {$debug} {echo "\[get_primary_clocks\] -I- myobj is [get_attribute -q $myobj full_name]"}

     # Is this set empty ?
     if {$myobj_tiarcs == ""} {
      # We've reached a sequential cell, that is, which has timing_arcs between some 
      # of its inputs or on the same input (for pulse width duration check). But the
      # problem being that there is no timing_arcs to $myobj.
      #
      # As a consequence we need to consider $myobj as a weird potential primary clock source
      if {!$quiet} {echo "\[get_primary_clocks\] -WW- Output pin '[get_attribute -q $myobj full_name]' is a potential primary clock source."}
      set clktree_pinsports [add_to_collection -u $clktree_pinsports $myobj]

      array set visitedhash	[list [get_attribute -q $myobj full_name] 1]
     } else {
      # Ok then, it looks like we do have path(s) to go through.
      # Since we are processing a clock-tree we should only consider timing_arcs
      # starting at a clock-pin and ignore all others.
      set seqcell_clkpins	{}
      foreach_in_collection myseqarc $myobj_tiarcs {
       set seqfrompin	[get_attribute -q $myseqarc from_pin]
       set is_clockpin	[get_attribute -q $seqfrompin is_clock_pin]
       
       if {$debug} {echo "\[get_primary_clocks\] -I- Looping on pin [get_attribute -q $seqfrompin full_name]"}
       
       if {$is_clockpin == "true"} {
        if {$debug} {echo "\[get_primary_clocks\] -I- Clock pin \[$is_clockpin\] : \[[get_attribute -q $seqfrompin full_name]\]"}
        set seqcell_clkpins [add_to_collection -u $seqcell_clkpins $seqfrompin]
       } else {
        if {$debug} {echo "\[get_primary_clocks\] -I- is_clockpin=EMPTY \[$is_clockpin\] : \[[get_attribute -q $seqfrompin full_name]\]"}
       }
      }

      # Normally we should only have no more than one path from a clock-pin to a given
      # output of a sequential, at least on a clock-tree (I guess). 
      set szcol	 [sizeof_collection $seqcell_clkpins]
      if {$szcol > 1} {
       # We have more than one clock-pins through which to propagate. This is a little bit
       # confusing. 
       if {!$quiet} {
        echo "\[get_primary_clocks\] -WW- Sequential cell '[get_attribute -q $pincell full_name]' has more than one input clock for output\
       '[get_attribute -q $myobj full_name]'."
        echo "\[get_primary_clocks\] -I- Skipping fanin processing through cell '[get_attribute -q $pincell full_name]'."
       }
      } elseif {$szcol == 0} {
        # Well, there seems to be no path from a clock-pin to $myobj
	# But let's check whether or not, there is just ONE path leading to $myobj
	# even if it is not from a pin with the *is_clock_pin* property set.
	set myobj_tiarcs_sz	[sizeof_collection $myobj_tiarcs]
        if {$myobj_tiarcs_sz == 1} {
	  # Ok then, so let's propagate through the corresponding *from_pin* since it's
	  # only way out we have
          if {!$quiet} {
	   echo "\[get_primary_clocks\] -WW- No timing_arcs from a clock-pin to output '[get_attribute -q $myobj full_name]' was found."
           echo "\[get_primary_clocks\] -I-  The Culprit is cell '[get_attribute -q $pincell full_name]'."
	   echo "\[get_primary_clocks\]      Cell '[get_attribute -q $pincell full_name]' lacks of input clock-pin definition."
	   echo "\[get_primary_clocks\]      Propagating through it anyway, since there is only one timing_arc leading to '[get_attribute -q $myobj full_name]'."
	  }

	  foreach_in_collection myseqarc $myobj_tiarcs {
          set clktree_pinsports [add_to_collection -u $clktree_pinsports [ce_get_primary_clocks [get_attribute -q $myseqarc from_pin] \
	  									visitedhash warnedhash opt_list]]
	  }
	} else {
	 # There is no timing_arcs from clock-pins leading to $myobj.
	 # But there are MORE THAN ONE path from non-clock-pin to $myobj. This is confusing so I skip this Weird cell
	 if {!$quiet} {
	  echo "\[get_primary_clocks\] -W-  No timing_arcs from clock-pin(s) to output pin '[get_attribute -q $myobj full_name]',"
	  echo "\[get_primary_clocks\]      but more than one ($myobj_tiarcs_sz) from Non-clock-pins. This is confusing so I prefer not propagate"
	  echo "\[get_primary_clocks\]      through cell '[get_attribute -q $pincell full_name]'."
	  echo "\[get_primary_clocks\] -I-  Output pin '[get_attribute -q $myobj full_name]' is identified as a potential primary clock source."
	 }

         set clktree_pinsports [add_to_collection -u $clktree_pinsports $myobj]
	}
      } else {
       # Now we are ready to Recursively call get_primary_clocks on *the* found clock-pin
       # add append the result to $clktree_pinsports
       echo "\[get_primary_clocks\] -I- Propagating through sequential cell '[get_attribute -q $pincell full_name]'."
       set clktree_pinsports [add_to_collection -u $clktree_pinsports [ce_get_primary_clocks $seqcell_clkpins visitedhash warnedhash opt_list]]
      }
     }
    }
   }
  }
 }
 
 return $clktree_pinsports
}
