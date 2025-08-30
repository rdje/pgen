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

proc	get_iotimings	{args} {
 getoptions gi_arg $args

 array set gi_arg_so $gi_arg(SO)
 array set gi_arg_so $gi_arg(OWV)

 if {[info exists gi_arg_so(help)]} {
  echo "usage: get_iotimings \[--type=<max|min>\] \[--slack=<SlackValue>\] \[--quiet\] \[--debug\] \[--help\]"
  return {}
 }

 # Default is not to print messages
 set	verbose	1
 if {[info exists gi_arg_so(quiet)]} {set verbose 0}

 set type	"max"
 if {[info exists gi_arg_so(type)]} {set type	$gi_arg_so(type)}

 set debug   [info exists gi_arg_so(debug)]

 set slack	0
 if {[info exists gi_arg_so(slack)]} {set slack	$gi_arg_so(slack)}

 # Intializing the Interface ports Timing Paths hash Table
 array	set	ports_tp_hash	{}
 
 if {$verbose} {echo "(get_iotimings) -I- Finding all input ports associated with a clock object.."}

 # Get the name of all input ports associated with Clock Object
 set	input_clks	{}
 foreach_in_collection myclk [get_clocks *] {
  foreach_in_collection mypinorport [get_attribute -q $myclk sources] {
   if {[get_attribute -q $mypinorport object_class] == "port"} {
    # I Guess there is no need to check for the direction :)
    set inputclkname	[get_attribute -q $mypinorport full_name]
    lappend input_clks 	$inputclkname
    if {$debug} {echo "(get_iotimings) -I- Input port '$inputclkname' is a clock."}
   }
  }
 }
 
 # Looping over all ports of the *current_design*
 if {$verbose} {echo "(get_iotimings) -I- Retrieving all ports timing paths.."}

 foreach_in_collection myport [get_ports *] {
   # For each port retrieves Timimg_Paths (TP) starting (input) from it or ending (output) at it
   set from_or_to	[expr {[get_attribute -q $myport direction] == "in" ? "-from" : "-to"}]
   set myport_name	[get_attribute -q $myport full_name]
   
   #----------------------------------------------------------------------
   # We should *filter* out any ports (not only inputs) associated with a Clock Object
   #----------------------------------------------------------------------
   if {[lsearch -regexp $input_clks $myport_name] != -1} {
    if {$verbose} {echo "(get_iotimings) -W- Timing paths from input port '$myport_name' won't be extracted !"}

    continue
   }
 

   set from_or_to_tp	[get_timing_paths -delay_type $type -nw 1 -max 1 -slack_lesser_than $slack $from_or_to $myport]
   # Needs to take care of INOUT perticularities --> one get_timing_path command for both -from/-to
   set inoutp {}
   if {[get_attribute -q $myport direction] == "inout"} {
    set to_or_from	[expr {$from_or_to == "-from" ? "-to" : "-from"}]
    set inoutp		[get_timing_paths -delay_type $type -nw 1 -max 1 -slack_lesser_than $slack $to_or_from $myport]
    set from_or_to_tp	[add_to_collection -unique $from_or_to_tp $inoutp]
   }

   array set ports_tp_hash	[list $myport_name $from_or_to_tp]
 }
 
 
 set i_meet			{}
 set o_meet			{}
 set int_meet		        {}
 set c_meet			{}
 
 set i_violated		        {}
 set o_violated		        {}
 set int_violated		{}
 set c_violated		        {}

 set i_noconstraint		{}
 set o_noconstraint		{}
 set int_noconstraint	        {}
 set c_noconstraint		{}

 set nopath		        {}
 set mpath		        {}
 set worst_path	        	{}
 set apaths 		        {}

 # Looping over all ports' name
 foreach myportname [lsort [array names ports_tp_hash]] {
 if {$debug} {echo "(get_iotimings) -I- Extracting timing paths information of port '$myportname'.."}
 
 #---------------------------------------
 # Retrieving the TPs' list of that port
 #---------------------------------------
  set	port_tps	$ports_tp_hash($myportname)
  
  if {$port_tps == {}} {
   # In case there is no Timing Path(s) involving that port
   # just log it and skip this iteration
    set ucv	[get_attribute -q -c port $myportname user_case_value]
   lappend nopath	[list $myportname [get_attribute -q -c port $myportname direction] [expr {$ucv != "" ? $ucv : "-"}]]
   continue
  }

  # Extracting the current port timings paths information
  array set tpinfo		[get_tpinfo $port_tps]

  # The PUSH command is mandatory here.
  push	i_meet			$tpinfo(i_meet)
  push  o_meet			$tpinfo(o_meet) 
  push  int_meet		$tpinfo(int_meet)
  push  c_meet			$tpinfo(c_meet)

  push	i_violated		$tpinfo(i_violated)
  push  o_violated		$tpinfo(o_violated) 
  push  int_violated		$tpinfo(int_violated)
  push  c_violated		$tpinfo(c_violated)

  push	i_noconstraint		$tpinfo(i_noconstraint)
  push  o_noconstraint		$tpinfo(o_noconstraint) 
  push  int_noconstraint	$tpinfo(int_noconstraint)
  push  c_noconstraint		$tpinfo(c_noconstraint)

  push  nopath			$tpinfo(nopath)

  push  mpath			$tpinfo(maxpath)
  push  worst_path		$tpinfo(worst_path)
  push  apaths 			$tpinfo(allpaths)
 }
 
 
 array set maxpath	$mpath		
 array set worst_paths	$worst_path		
 array set allpaths	$apaths		

#  # Fixing  multi-occurence problem
#  # Better doing this for all three types of paths
#  set fixed_info_i	{}
#  port2pin_fix $input fixed_info_i worst_paths
# 
#  set fixed_info_o	{}
#  port2pin_fix $output fixed_info_o worst_paths
# 
#  set fixed_info_combi	{}
#  port2pin_fix $combinational fixed_info_combi worst_paths

 # preparing data to be return as a Hash table.
# array set	retv	[list tip 		[list $fixed_info_i $fixed_info_o $fixed_info_combi]]
 array set	retv	[list i_meet		$i_meet]
 array set	retv	[list o_meet		$o_meet]
 array set	retv	[list int_meet		$int_meet]
 array set	retv	[list c_meet		$c_meet]

 array set	retv	[list i_violated	$i_violated]
 array set	retv	[list o_violated	$o_violated]
 array set	retv	[list int_violated	$int_violated]
 array set	retv	[list c_violated	$c_violated]

 array set	retv	[list i_noconstraint	$i_noconstraint]
 array set	retv	[list o_noconstraint	$o_noconstraint]
 array set	retv	[list int_noconstraint	$int_noconstraint]
 array set	retv	[list c_noconstraint	$c_noconstraint]

 array set	retv	[list nopath		$nopath]

 array set	retv	[list allpaths 		[array get allpaths]]

 return [array get retv]
}


# proc	port2pin_fix	{input2pin newinputpaths theworstpath} {
#  upvar 1 $newinputpaths new_inputpaths
#  upvar 1 $theworstpath  worst_paths
# 
#  array set	already_seen  		{}
#  array set	multiple_occurrence	{}
# 
#  
#  foreach myipath $input2pin {
#   # For each $startpoint/$endpoint_clock pair that is a port 2 pin, get the worst path.
#   # index #0 points to 'spname', index #3 points to 'epclkname'
#   set l_spname		[lindex $myipath 0]
#   set l_epname		[lindex $myipath 1]
#   set l_spclkname	[lindex $myipath 2]
#   set l_epclkname	[lindex $myipath 3]
#   if {![info exists already_seen($l_spname,$l_epname,$l_spclkname,$l_epclkname)]} {
#    lappend	new_inputpaths	$worst_paths($l_spname,$l_epname,$l_spclkname,$l_epclkname)
#    set	already_seen($l_spname,$l_epname,$l_spclkname,$l_epclkname)	1
#   } else {
#    set multiple_occurrence($l_spname,$l_epname,$l_spclkname,$l_epclkname) 1
#   }
#  }
# 
# }
