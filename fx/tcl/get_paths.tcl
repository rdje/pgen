#-------------------------------------------------------------------------------------------
# Author	: Richard DJE
#
# Description	: get_paths
#-------------------------------------------------------------------------------------------
# Property of Texas Instruments - For  Unrestricted Internal Use  Only.
# Unauthorized reproduction and/or distribution is strictly prohibited.  This
# product is protected under copyright law and trade secret law as an unpublished
# work. (C) Copyright 2005 Texas Instruments. All rights reserved.
#-------------------------------------------------------------------------------------------
package provide	stapack 1.0

proc	get_paths	{args} {
 getoptions gp_args $args

 array set optlist $gp_args(SO)
 array set optlist $gp_args(OWV)

 if {[info exists optlist(help)]} {
  echo "usage: get_paths --from=<from_spec> --to=<to_spec> \[--out=<OutputFile>\] \[--help\]"
  return {}
 }

 if {[info exists optlist(from)] + [info exists optlist(to)] != 2} {
  echo "(get_paths) -E- The --from and --to options are required."
  echo "usage: get_paths --from=<from_spec> --to=<to_spec> \[--out=<OutputFile>\] \[--help\]"
  return {}
 }

 set from_spec	[object -pin -port $optlist(from)]
 set to_spec	[object -pin -port $optlist(to)]
 if {$from_spec == ""} {
  echo "(get_paths) -E- The *from_spec* seems not to be valid."
  echo "usage: get_paths --from=<from_spec> --to=<to_spec> \[--out=<OutputFile>\] \[--help\]"
  return {}
 } elseif {$to_spec == ""} {
  echo "(get_paths) -E- The *to_spec* seems not to be valid."
  echo "usage: get_paths --from=<from_spec> --to=<to_spec> \[--out=<OutputFile>\] \[--help\]"
  return {}
 }

 echo "(get_paths) F:[get_attribute $from_spec full_name] T:[get_attribute $to_spec full_name]"
 
 set gfo [get_fanout -maxlevel=100 $from_spec]
 set bsz [sizeof_collection $gfo]
 set asz [sizeof_collection [remove_from_collection $gfo $to_spec]]
 if {$asz < $bsz} {
  echo "(get_paths) Found  T:[get_attribute $to_spec full_name] BSZ=($bsz) ASZ=($asz)"
 } else {
  echo "(get_paths) BSZ=($bsz) ASZ=($asz)"
  foreach_in_collection myseqpin $gfo {
   set cell [get_cells -o $myseqpin]
   if {[is_gsx0_cgc $cell] || [is_gsx0_clkmux $cell]} {
    set gthru	[go_thru -out --notiarcs $myseqpin]
    foreach_in_collection  mythru $gthru {
     get_paths -from=$mythru -to=$to_spec 
    }
   }
  }
 }

}
