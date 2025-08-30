proc	c2ctip		{initial_cell final_cell}	{
 # Before Firing up the whole process we should define a new
 # attribute to be used for storing an already_visited status
 # on cells
 define_user_attribute -type int -class cell already_visited

 set return_lst	[c2ctip_ce $initial_cell $final_cell $initial_cell]

 # Removing attribute *already_visited* on all cells on which  it has been set
 remove_user_attribute -q [get_cells -q * -filter "defined(already_visited)"] already_visited

 return $return_lst
}

# This procedure is aimed at walking through the netlist loaded into
# PT runtime database and retrieve all timing paths between two named cells.
proc	c2ctip_ce	{initial_cell final_cell current_cell {initial_pin ""}} {
# upvar $vcells vcells_lst

 # Defining an empty list
 # It will be used for storing result
 # from calls to c2ctip if any
 set tip_lst	{}
 
 # get all pins belonging to the current_cell
 # and loop over the resulting pin collection
 #
 # When not on the top c2ctip call, we should take care of not
 # getting pins not having the same direction than initial_pin
 set filter_option	[expr {$initial_pin != "" ? "-filter \"direction == [get_attribute -q -c pin $initial_pin direction]\"" : ""}]
 echo "\[c2ctip_ce\] -I- filter_option=\[$filter_option\] current_cell=\[$current_cell\]"
 if {$initial_pin != ""} {
  set get_pins_cmdline	"get_pins -q -o $current_cell -filter \"direction == [get_attribute -q -c pin $initial_pin direction]\""
 } else {
  set get_pins_cmdline	"get_pins -q -o $current_cell"
 }

 foreach_in_collection mycellpin [eval $get_pins_cmdline] {
   set c2ctip_pinarg [expr {$initial_pin == "" ? $mycellpin : $initial_pin}]

   echo "\[c2ctip_ce\] -I- Processing pin [get_attribute -q $mycellpin full_name]"

   ##if {$initial_pin != ""} {
   ## # For all c2ctip but the Very first, we will be considering only those
   ## # pins whose direction match that of the 'initial_pin'
   ## set mycellpin_dir	[get_attribute -q $mycellpin direction]
   ## set	initial_pin_dir	[get_attribute -q $initial_pin direction]
   ## if {$mycellpin_dir != $initial_pin_dir} {continue}
   ##}
   
   # For each pin retrieve the connected net, if any
   set cellnet	[get_nets -q -o $mycellpin]
   if {$cellnet == ""} {
    # In cases where the current pin is floating, that is not
    # connected to any net skip the current iteration
    echo "\[c2ctip_ce\] -I- Skipping current iteration pin is dangling"
    echo "\[c2ctip_ce\] [get_attribute -q $mycellpin full_name]"
    continue
   }

   # Having the net, we need the get all connected pins.
   # The actual direction of those pins depends on that of
   # mycellpin in the following way.
   set	actualpin_dir	[expr {[get_attribute -q $mycellpin direction] == "in" ? "out" : "in"}]
   foreach_in_collection myinoutpin [get_pins -q -filter "direction == $actualpin_dir" -o $cellnet] {
    echo "\[c2ctip_ce\] -I- Processing(2) in/out pin [get_attribute -q $myinoutpin full_name]"
    # For each of these input/output pins, get their corresponding cell
    set in_outpin_cell	[get_cells -q -o $myinoutpin]
    echo "\[c2ctip_ce\] -I- The above pin cell is '[get_attribute -q $in_outpin_cell full_name]'"
    
    # In order to speed up the whole process we should skip any cell
    # that has already been visited, that is, that has is *already_visited*
    # property set. It is to be noted that this property is user-defined
    if {[get_attribute -q $in_outpin_cell already_visited] != ""} {
     echo "\[c2ctip_ce\] -I- Skipping current iteration already_visited cell"
     echo "\[c2ctip_ce\] [get_attribute -q $in_outpin_cell full_name]"
     continue
    }

    # Please note that I don't check for the emptiness of $in_outpin_cell
    # because it's unlikely that $in_outpin_cell be empty.
    # In case I am wrong here, patch will be easily added.
    
    # Here we verify if we've reached the $final_cell
    if {[get_attribute -q $in_outpin_cell full_name] == $final_cell} {
     # Got it ! we have another pair (if not the first) of 
     # Driver/Load ($myinoutpin) and Load/Driver ($initial_pin/$mycellpin)
     #
     # before returning to the caller, we need to extract and
     # return the corresponding Timing Path.

     # But we need to take care depending on whether or not we
     # are at the top-level of c2ctip calling stack
     #
     # Important: Without the explicit use of *--include_hierarchical_pins* ('-inc' below)
     #            then hierarchical pins **do not appear** as part of the timing_points
     #            linked to the a given timing_path.
     if {[get_attribute -q $c2ctip_pinarg direction] == "in"} {
      set get_timing_paths_cmd	"get_timing_paths -inc -from $myinoutpin -to $c2ctip_pinarg"
     } else {
      set get_timing_paths_cmd	"get_timing_paths -inc -from $c2ctip_pinarg -to $myinoutpin"
     }

     # The execution of $get_timing_paths_cmd will probably print a bunch of messages like
     # invalid startpoint or endpoint blah, blah, blah,..., but that's normal, since
     # it is likely that $myinoutpin not be a real start/end-point.
     set retip	[eval $get_timing_paths_cmd]

     # What we need to do is to extract the timing_point *point* attribute available in the
     # above timing_path and then locate the startpoint and endpoint pair as used to retrieve
     # the timing_path. From there substracting the arrival_time of the startpoint from that
     # of the endpoint will give us the delay between both.
     #
     # The process just described is done by the following call to intercon_xt()
     # Here please pay some attention. The intercon_xt routine needs to know
     # which one of $myinoutpin and $c2ctip_pinarg is the *from* and which one is
     # the *to*. To accomplish this I just pass the string executed by the above eval 
     # command as argument to *intercon_xt*. Doing so, it will be able to retrieve
     # the appropriate information by capturing the from/to field using *regexp*.
     # Of course it also receives the timing_path structure as its second argument.
     set xtdelay [intercon_xt $get_timing_paths_cmd $retip]

     # The *intercon_xt* just described returns a list of three fields, this list as a whole
     # will be considered as a single entry in $tip_lst list
     #
     # $xtdelay have to be non-empty before pushing it onto $tip_lst 
     if {[llength $xtdelay]} {
      # $xtdelay may be empty mostly because $retip didn't contains any timing_points that is
      # the *points* attribute of $retip timing_path was found to be empty.
      lappend 	tip_lst $xtdelay

      echo "\[c2ctip_ce\] ------------------------------"
      echo "\[c2ctip_ce\] -I- Hourra Found a TIMING PATH"
      echo "\[c2ctip_ce\] ------------------------------"
      echo "\[c2ctip_ce\] [get_attribute -q $myinoutpin full_name]"
      echo "\[c2ctip_ce\] [get_attribute -q $c2ctip_pinarg full_name]"
      echo "\[c2ctip_ce\] ------------------------------"
     }


    } else {
     # It's a pity but we have not reached our the final_cell so we need to
     # continue our search. For this we first have to verify that the 
     # current cell is a combinational cell and then call c2ctip on that 
     # cell if it is the case. Because recursion will happen only on cells 
     # that have their *is_combinational* property set. I hope we can rely
     # on this property for identifying combinational cells :)
     #
     # If not a combinational block then we should skip the current *myinoutpin*
     # iteration.
     if {[get_attribute -q $in_outpin_cell is_combinational]} {
       # Here we go again ! :)
       # Again, depending on the fact that we are at the top of the calling
       # stack or not we will be referencing either $mycellpin or $initial_pin
       # for the last argument of the c2ctip call
       # Here we append the c2ctip call returned value to the tip_lst 
       echo "\[c2ctip_ce\] -I- Recursively calling c2ctip_ce on cell"
       echo "\[c2ctip_ce\] [get_attribute -q $in_outpin_cell full_name]"
       lappend	tip_lst [c2ctip_ce $initial_cell $final_cell $in_outpin_cell $c2ctip_pinarg]
     } else {
       # The current cell is not combinational, so we skip the current *myinoutpin* iteration
       echo "\[c2ctip_ce\] -I- Skipping current iteration cell"
       echo "\[c2ctip_ce\] [get_attribute -q $in_outpin_cell full_name] is not combinational"
       continue
     }
    }
   }
 }

 # Before leaving we have to mark the current cell has visited
 
 if {$initial_pin == ""} {
  echo "\[c2ctip_ce\] -I- Setting *already_visited* on $current_cell"
  set_user_attribute -q -class cell $current_cell already_visited	1
 } else {
  echo "\[c2ctip_ce\] -I- Setting *already_visited* on [get_attribute -q $current_cell full_name]"
  set_user_attribute -q $current_cell already_visited	1
 }

 echo "\[c2ctip_ce\] -I- Leaving cell [get_attribute -q $current_cell full_name]"
 # Returning the possibly empty list of Timing Paths
 # between the initial cell and final cell that pass through the 
 # current cell, if not the initial
 return $tip_lst
}


# Actual interconnect delay extraction routine
proc	intercon_xt	{fromto_info tip} {
 # First we need to extract the *from* and *to* fields
 # from $fromto_info argument
 echo "\[intercon_xt\] -I- \[$fromto_info\]"
 set fromto [regexp -all -inline -- {-(?:from|to)\s+(\w+)} $fromto_info]
 
 echo "\[intercon_xt\] -I- fromto=\[$fromto\]"

 # Both the *from* and *to* just extracted should be pin collection
 # handles/references 
 set from [lindex $fromto 1]
 set to   [lindex $fromto 3]

 # Since the *intercon_xt* routine is called with $tip
 # we need to check if it is not an empty timing_path 
 if {$tip == ""} {return [list [get_attribute -q $from full_name] - [get_attribute -q $to   full_name]]}

 # Now we need to retrieve the timing_point *points* attribute of the $tip
 set tpoint	[get_attribute -q $tip points]
 if {$tpoint == ""} {
  echo "\[intercon_xt\] ---------------------------------------------------------------------------"
  echo "\[intercon_xt\] -W- NO *points* attribute found !!"
  echo "\[intercon_xt\] -W- from='[get_attribute -q $from full_name]'"
  echo "\[intercon_xt\] -W- to  ='[get_attribute -q $to   full_name]'"
  echo "\[intercon_xt\] ---------------------------------------------------------------------------"
  return  [list [get_attribute -q $from full_name] - [get_attribute -q $to   full_name]]
 }

 # Important note: 
 #  Curly braces are to be used to enclose object (pins, ports, clocks, ...)
 #  names containing square brackets '[...]' so as *to prevent* Tcl  from
 #  attempting to do 'command substitution'. The point is that $from and $to
 #  names do respect this rule, but in some cases PT simply remove those quoting
 #  characters '{...}' when no problem may raise from that (I guess).
 #  This is exactly what happens for $pobj_name, which appear not to be quoted 
 #  when needed.
 #  A solution is to use the Tcl *regsub* command for systematically removing
 #  curly braces, if any, before doing any name comparison.
 #  
 #  The version of *regsub* used in PT 2004.06 requires a 'VarName' as last argument.
 regsub -all -- {\{|\}} [get_attribute -q $from full_name] {} m_fromname
 regsub -all -- {\{|\}} [get_attribute -q $to   full_name] {} m_toname
 set name_list	[list $m_fromname $m_toname]
 
 echo "\[intercon_xt\] -I- name_list=\[$name_list\]"
 
 # Now we need to find the $from point from the above timing_point collection
 # and extract its arrival time
 set	count	 0
 set	arrivals {}
 foreach_in_collection mypoint	$tpoint {
  # timing_point objects have an object field referencing the exact type
  # of object that pin or port located at the current point in the $tip
  set point_obj	[get_attribute -q $mypoint object]
  set pobj_name	[get_attribute -q $point_obj full_name]
  echo "\[intercon_xt\] -I- pobj_name=\[$pobj_name\] namelist_val=\[[lindex $name_list $count]\]"

  #set	namelist_val	[lindex $name_list $count]
  #echo "\[intercon_xt\] -I- namelist_val=\[$namelist_val\]"
  
  # Note: When using *lindex* it just skip one-level of braces, if any from the extracted entry
  if {$pobj_name == [lindex $name_list $count]} {
   # Retrieving the current timing_point arrival time
   lappend arrivals	[get_attribute -q $mypoint arrival]

   if {$count} {break} else {incr count}
  } else {
   continue
  }
 }

 # return {$from_name delay_in_between $to_name}
 return [list [get_attribute -q $from full_name] 			\
 	      [expr {[lindex $arrivals 1] - [lindex $arrivals 0]}]	\
	      [get_attribute -q $to   full_name]]
}
