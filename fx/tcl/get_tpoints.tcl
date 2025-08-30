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

proc get_tpoints {args} {
 getoptions gt_arg $args

 array set gt_so  $gt_arg(SO)

 if {[info exists gt_so(help)]} {
  echo "usage: get_tpoints <TimingPath> \[--out=<OutputFile>\] \[--help\]"
  return {}
 }

 set tp	 [lindex $gt_arg(SA) 0]
 if {$tp == ""} {
  echo "(get_tpoints) -E- Missing timing_path argument."
  echo "usage: get_tpoints <TimingPath> \[--out=<OutputFile>\] \[--help\]"
  return {}
 }

 array set gt_owv $gt_arg(OWV)

 set tpfile	""
 if {[info exists gt_owv(out)]} {
  set tpfile  $gt_owv(out)
  file delete $tpfile
 }

 set nodelist {}
 foreach_in_collection mypoint [get_attribute -q $tp points] {
  set myobj	[get_attribute -q $mypoint object]

  set cellref	{}
  if {[get_attribute -q $myobj object_class] == "pin"} {
   set objcell	[get_cells -o $myobj]
   if {$objcell == ""} {
    echo "(get_tpoints) -E.NOPINCELL- ([get_attribute -q $myobj full_name])"
    return {}
   } else {
    set cellref	 	[get_attribute -q $objcell ref_name]
   }
  } else {
   set cellref	 	"-"
  }
  
  set name	 	[get_attribute -q $myobj full_name]
  set direction 	[get_attribute -q $myobj direction]
  set object_class 	[get_attribute -q $myobj object_class]

  lappend nodelist [list $name $direction $object_class $cellref]
 }

 
 if {$tpfile != ""} {echo [join $nodelist \n] >> $tpfile}

 return $nodelist
}
