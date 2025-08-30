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

proc get_ucobj {args} {
 getoptions gu_args $args
 array set gu_args_so $gu_args(SO)
 array set gu_args_owv $gu_args(OWV)

 if {[info exists $gu_args_so(help)]} {
  echo "usage: get_ucobj \[--points=<PointsVar>\] \[--quiet\] \[--help\] <InportObjectSpec>"
  return {}
 }

 set inclk [filter_collection [object -port [lindex $gu_args(SA) 0]] "direction == in || direction == inout"]

 if {$inclk == ""} {
  if {![info exists $go_args_so(quiet)]} {
   echo "usage: get_ucobj \[--points=<PointsVar>\] \[--quiet\] \[--help\] <InportObjectSpec>"
   return {}
  }
 }

 set tpoints_en	 [info exits gu_args_owv(points)]

 set get_for_cmd	"get_for $inclk"
 if {$tpoints_en} {append get_for_cmd " --tpoints=mytpoints"} 

 set inclk_for [eval $get_for_cmd]

 if {$tpoints_en} {
  upvar 1 ${$gu_args_owv(points)} gu_points
  foreach myl1 $mytpoints {
   set newl2 {}
   foreach myl2 $myl1 {
    set newl3 {}
    array set visited {}
    foreach myl3 $myl2 {
     set obj [object -pin -port $myl3]
     set objcell [get_cells -q -o $obj]

     set objcell_name [get_attribute -q $objcell full_name]

     if {[info exists visited($objcell_name)]} {
      lappend newl3 "$myl3 [get_attribute -q $obj object_class] [get_attribute -q $obj direction]"
      continue
     } elseif {[is_gsx0_mux $objcell]} {
      lappend newl3 [gu_muxhandler $obj]
     } elseif {[is_gsx0_cgc $objcell]} {
      lappend newl3 [gu_cgchandler $obj]
     } else {
      lappend newl3 "$myl3 [get_attribute -q $obj object_class] [get_attribute -q $obj direction]"
     }

     set visited($objcell_name) 1
    }

    lappend newl2 $newl3
   }

   lappend gu_points $newl2
  }
 }

 return $inclk_for
}

proc gu_muxhandler {obj} {
}

proc gu_cgchandler {obj} {
}

proc write_cases	{args} {
 getoptions wu_arg $args

 array set wu_so $wu_arg(SO)

 if {[info exists wu_so(help)]} {
  echo "usage: write_cases  \[--cv | --user\] \[--out=CaseFile\] \[--help\]"
  return {}
 }

 array set wu_owv $wu_arg(OWV)

 set cv_opt 	[info exists wu_so(cv)]
 set ucv_opt 	[info exists wu_so(user)]

 if {($cv_opt + $ucv_opt) == 2} {
  echo "(write_cases) -E- Exactly one switch (cv/user) should be provided."
  echo "usage: write_cases  \[--cv | --user\] \[--out=CaseFile\] \[--help\]"
  return {}
 }

 set right_attribute	[expr {$cv_opt ? "case_value" : "user_case_value"}]

 set ucfile	"ucv_file.lof"
 if {[info exists wu_owv(out)]} {
  set ucfile	$wu_owv(out)
 }

 file delete $ucfile

 set sortedlist {}
 set portcases [get_ports -fi "defined($right_attribute)" *]
 if {$portcases != ""} {
  echo "=port=" >> $ucfile
  foreach_in_collection mypcase $portcases {
   lappend sortedlist [write_ucnode [get_attribute -q $mypcase full_name]]
  }

  echo [join [lsort -increasing -dictionary $sortedlist] \n] >> $ucfile

  echo "=portend=" >> $ucfile
 }

 set sortedlist {}
 set pincases [get_pins -hi -fi "defined($right_attribute)" *]
 if {$pincases != ""} {
  echo "=pin=" >> $ucfile
  set lv	0
  foreach_in_collection mypcase $pincases {
   set retdata [write_ucnode [get_attribute -q $mypcase full_name]]
   set chk1st  [lindex $retdata 0]
   if {$chk1st != "@"} {lappend sortedlist $retdata}

   incr lv
  }


  echo [join [lsort -increasing -dictionary $sortedlist] \n] >> $ucfile

  echo "=pinend=" >> $ucfile
 }
}

proc write_ucnode {path2obj}	{

 set iscell [object -cell $path2obj]
 if {$iscell != ""} {
  # We have a cell.
  #
  # If it's hierarchical get its ref_name , otherwise pop-out the last 
  # part of the name and recursively call *write_ucnode* on the reminder
  if {[get_attribute -q $iscell is_hierarchical]} {
   set outstr {}
   push outstr	[get_attribute -q $iscell ref_name]
   push outstr	$path2obj

   return $outstr
  } else {
   # I need to filter TIE-OFFs out
   set refname [get_attribute -q $iscell ref_name]
   if {[regexp {(?i)^TO.*} $refname]} {
    return "@ @"
   }

   set splitpath [split $path2obj /]
   set leafname [pop splitpath]

   if {[llength $splitpath] == 0} {
    # non-hierarchical cells instanciated right below
    # the top level module
    set outstr {}
    push outstr [get_attribute -q [current_design] full_name]
    push outstr	"-"
    push outstr	$path2obj

    return $outstr
   }

   set retv [write_ucnode [join $splitpath /]]
   if {[llength $retv] > 2} {
    set lasthier	[pop	retv]
    append lasthier	"/$leafname"
    push	retv	$lasthier
   } else {
    push retv "$leafname"
   }

   return $retv
  }
 } elseif {[object -pin -port $path2obj] == ""} {
  #
  # This is a LOST Hierarchical Cell.
  # probably due to the flattening during synthesis
  # of one of its parents module.
  # 
  # I do exactly the step required when dealing w/ a true non-hierarchical cell
  # (cf. just above)
  set splitpath [split $path2obj /]
  set leafname [pop splitpath]

  set retv [write_ucnode [join $splitpath /]]
  if {[llength $retv] > 2} {
   set lasthier	[pop	retv]
   append lasthier	"/$leafname"
   push	retv	$lasthier
  } else {
   push retv "$leafname"
  }

  return $retv
 }


  # We have a pin/port
  set isport [object -port $path2obj]
  if {$isport != ""} {
   set outlist {}
   push outlist [get_attribute -q [current_design] full_name]
   push outlist "-"
   push outlist $path2obj
   push outlist [get_attribute -q $isport direction]
   set ucv	[get_attribute -q $isport user_case_value]
   push outlist [expr {$ucv != "" ? $ucv : "-"}]
   
   return $outlist
  } else {
   # No. with have a pin
   set splitpath	[split $path2obj /]
   set leafname		[pop splitpath]

   set retv	[write_ucnode [join $splitpath /]]
   
   if {[llength $retv] > 2} {
    set lasthier	[pop	retv]
    append lasthier	"/$leafname"
    push	retv	$lasthier
   } else {
    push retv "$leafname"
   }

   push retv 	[get_attribute -q -c pin $path2obj direction]
   set ucv	[get_attribute -q -c pin $path2obj user_case_value]
   push retv	[expr {$ucv != "" ? $ucv : "-"}]

   return $retv
  }

}
