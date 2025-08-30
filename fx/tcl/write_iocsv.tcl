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

proc	write_iocsv	{args} {
 getoptions wi_arg $args

 array set wi_arg_so 	$wi_arg(SO)
 array set wi_arg_so 	$wi_arg(OWV)
 array set wi_arg_owv 	$wi_arg(OWV)


 if {[info exists wi_arg_so(help)]} {
  echo "usage: write_iocsv \[--out=<OutputFile>\] \[--type=<max|min>\]  \[--slack=<SlackValue>\] \[--quiet\] \[--debug\] \[--help\]"
  return {}
 }

 # Default Values
 set verbose	1
 set output	"csvinfo.csv"

 if {[info exists wi_arg_so(quiet)]} {set verbose  0}
 if {[info exists wi_arg_owv(out)]}  {set output   $wi_arg_owv(out)}

 # Building other commands options list
 set option_list	""
 foreach myopt [array names wi_arg_so] {
  if {[info exists wi_arg_owv($myopt)]} {
   append option_list " --$myopt=$wi_arg_owv($myopt)"
  } else {
   append option_list " --$myopt"
  }
 }

 # Let's get the Clock-Tree info
 # set get_ctinfo_cmd	"get_ctinfo $option_list"
 # array 	set	ctinfo	[eval $get_ctinfo_cmd]

 # Then assign the content of *iotimings* to an hash table
 set get_iotimings_cmd	"get_iotimings $option_list"
 array	set	ioghash	[eval $get_iotimings_cmd]

 # That's just a list of list, it is not structured as a hash table
 # set	tip		$ioghash(tip)

 
 # Don't forget to delete the file if it already exists.
 if {[file exists $output]} {file delete $output}
 
 #------------------------------------
 # Setting the Header of the CSV file
 #------------------------------------
 # set	csv_header	"# StartPoint EndPoint SPClock EPClock DataPathDelay  SPClock_Latency EPClock_Latency StartPoint_direction EndPoint_direction"
 # echo  $csv_header >> $output
 
 # Driving ioinfo
 # set ioentries	{}
 if {$verbose} {echo "(write_iocsv) -I- Generating '$output'.."}

 set i_meet		$ioghash(i_meet)
 set o_meet		$ioghash(o_meet)
 set int_meet		$ioghash(int_meet)
 set c_meet		$ioghash(c_meet)
 
 set i_violated		$ioghash(i_violated)
 set o_violated		$ioghash(o_violated)
 set int_violated	$ioghash(int_violated)
 set c_violated		$ioghash(c_violated)

 set i_noconstraint	$ioghash(i_noconstraint)
 set o_noconstraint	$ioghash(o_noconstraint)
 set int_noconstraint	$ioghash(int_noconstraint)
 set c_noconstraint	$ioghash(c_noconstraint)

 set	nopath		$ioghash(nopath)

 set fcontent {}
 foreach dir {i o int c} {
  foreach type {meet violated noconstraint} {
   set secname	${dir}_$type
   lappend fcontent "=$secname="
   foreach paths [set [subst {$secname}]] {lappend fcontent [join $paths ,]}
   lappend fcontent "=${secname}_end="
  }
 }

 lappend fcontent "=nopath="
 foreach mycsv $nopath {lappend fcontent "[join $mycsv ,]"}
 lappend fcontent "=nopath_end="

 # set lv	 0
 # foreach mylst	$tip  {
 #  set type [expr {$lv ? "_o" : "_i"}]

 #  echo ":ioinfo$type-begin" >> $output
 #  
 #  foreach mycsv $mylst {lappend ioentries [join $mycsv ,]}
 #  echo [join $ioentries \n] >> $output

 #  echo ":ioinfo$type-end" >> $output

 #  set ioentries	{}
 #  incr lv

 #  if {$lv == 2} {break}
 # }
 # 
 # # Driving the combinational part
 # echo ":combi-begin" >> $output
 # foreach mycsv [lindex $tip 2] {
 #  echo "[join $mycsv ,]" >> $output
 # }
 # echo ":combi-end" >> $output


 echo [join $fcontent \n] > $output
}

