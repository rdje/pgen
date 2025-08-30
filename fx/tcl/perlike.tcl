#----------------------------------------------------------------------------------------------
# Author	: Richard DJE
#
# Description	: The name of the file says it all.
#
#----------------------------------------------------------------------------------------------
# Property of Texas Instruments - For  Unrestricted Internal Use  Only.
# Unauthorized reproduction and/or distribution is strictly prohibited.  This
# product is protected under copyright law and trade secret law as an unpublished
# work. (C) Copyright 2005 Texas Instruments. All rights reserved.
#----------------------------------------------------------------------------------------------
package provide	stapack 1.0

proc push {args} {
 getoptions p_args $args
 
 array set p_args_so $p_args(SO)

 if {[info exists p_args_so(help)]} {
  echo "usage: push <TclistName> <ListoPush> \[-help\]"
  return ""
 }

 if {[llength $p_args(SA)] != 2} {
  echo "(push) -E- Exactly Two positional arguments should be provided."
  echo "usage: push <TclistName> <ListoPush> \[-help\]"
  return ""
 }

 set listname	[lindex $p_args(SA) 0]
 upvar $listname uplist
 foreach myent [lindex  $p_args(SA) 1] {
  lappend  uplist $myent
 }
}

proc pop {args} {
 getoptions p_args $args
 
 array set p_args_so $p_args(SO)

 if {[info exists p_args_so(help)]} {
  echo "usage: pop <TclistName> \[-help\]"
  return ""
 }

 if {[llength $p_args(SA)] != 1} {
  echo "(pop) -E- Exactly One positional argument should be provided."
  echo "usage: pop <TclistName> \[-help\]"
  return ""
 }

 
 set listname	[lindex $p_args(SA) 0]
 upvar $listname uplist
 set lastdata [lindex  $uplist end]


 # Deleting the last entry of that list
 set uplist [lrange $uplist  0 end-1]


 return $lastdata
}


proc shift {args} {
 getoptions s_args $args
 
 array set s_args_so $s_args(SO)

 if {[info exists s_args_so(help)]} {
  echo "usage: shift <TclistName>\[-help\]"
 }
 
 if {[llength $s_args(SA)] != 1} {
  echo "(shift) -E- Exactly One positional arguments should be provided."
  echo "usage: shift <TclistName>\[-help\]"
  return ""
 }

 set listname	[lindex $s_args(SA) 0]
 upvar $listname uplist
 set firstdata [lindex  $uplist 0]

 # Deleting the first entry of that list
 set uplist [lrange $uplist  1 end]

 return $firstdata
}

proc unshift {args} {
 getoptions u_args $args
 
 array set u_args_so $u_args(SO)

 if {[info exists u_args_so(help)]} {
  echo "usage: unshift <TclistName> <ValueToUnShift>\[-help\]"
  return ""
 }

 if {[llength $u_args(SA)] != 2} {
  echo "(unshift) -E- Exactly Two positional arguments should be provided."
  echo "usage: unshift <TclistName> <ValueToUnShift>\[-help\]"
  return ""
 }

 set listname	[lindex $u_args(SA) 0]
 upvar $listname uplist

 set  newlist  [lindex  $u_args(SA) 1]
 foreach myelem $uplist {
  lappend newlist $myelem
 }

 set uplist $newlist
}


proc reverse {args} {
 getoptions r_args $args
 
 array set r_args_so $r_args(SO)
 
 if {[info exists r_args_so(help)]} {
  echo "usage: reverse <Tclist>\[-help\]"
 }
 
 if {[llength $r_args(SA)] != 1} {
  echo "(reverse) -E- Exactly One positional arguments should be provided."
  echo "usage: reverse <Tclist>\[-help\]"
  return ""
 }

 
 set listname	[lindex $r_args(SA) 0]
 upvar $listname uplist
 
 set reversed	{}
 set size	[llength $uplist]
 set idx	0
 while {$idx < $size} {
  set rev_idx	[expr {$size -1 - $idx}]
  lappend reversed [lindex $uplist $rev_idx]
  incr idx
 }

 return $reversed
}

