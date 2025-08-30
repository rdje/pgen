#----------------------------------------------------------------------------------------------
# Author	: Richard DJE
#----------------------------------------------------------------------------------------------
package provide	stapack 1.0

proc go_thru {args} {
 getoptions gt_arg $args

 array set gt_so $gt_arg(SO)

 if {[info exists gt_so(help)]} {
  echo "usage: go_thru  <pin_spec> \[--in | --out\] \[--notiarcs\] \[--help\]"
  return {}
 }

 set in_opt 	[info exists gt_so(in)]
 set out_opt 	[info exists gt_so(out)]
 #set options	[expr {$in_opt + $out_opt}]
 if {($in_opt + $out_opt) % 2 == 0} {
  echo "(go_thru) -E- Exactly one switch (in/out) should be provided."
  echo "usage: go_thru  <pin_spec> \[--in | --out\] \[--notiarcs\] \[--help\]"
  return {}
 }

 set direction [expr {$in_opt ? "in" : "out"}]

 set pinarg [lindex $gt_arg(SA) 0]
 if {$pinarg == ""} {
  echo "(go_thru) -E- Missing Pin argument."
  echo "usage: go_thru  <pin_spec> \[--in | --out\] \[--notiarcs\] \[--help\]"
  return {}
 }

 set pinspec [object -pin $pinarg]
 if {$pinspec == ""} {
  echo "(go_thru) -E- Provided argument is not an existing pin object."
  return {}
 }

 set pincell [get_cells -q -o $pinspec]
 if {$pincell == ""} {
  echo "(go_thru) -E- Provided pin has NO cell."
  return {}
 }

 set pincell_name [get_attribute -q $pincell full_name]

 if {![info exists gt_so(notiarcs)]} {
  # Retrieving all cell timing arcs
  set all_tiarcs [get_timing_arcs -o $pincell -fi "is_disabled == false"]
  if {$all_tiarcs == ""} {
   echo "(go_thru) -E- Cell *$pincell_name* has NO non-disabled Timing Arc(s)."
   return {}
  } 

  set x_dir [expr {$direction == "in" ? "from" : "to"}]
  set y_dir [expr {$direction == "in" ? "to"   : "from"}]

  set all_xpins {}

  # Keeping only those otherside x_pins having a timing arc
  # with our $pinspec pin.
  foreach_in_collection mytiarc $all_tiarcs {
   set x_pin	[get_attribute -q $mytiarc "${x_dir}_pin"]
   set y_pin	[get_attribute -q $mytiarc "${y_dir}_pin"]

   if {[compare_collection $pinspec $y_pin] == 0} {
    set all_xpins [add_to_collection -unique $all_xpins $x_pin]
   }
  }

  set right_dirpins [filter_collection $all_xpins  "direction == $direction || direction == inout"]

  # Due to *inout* pins behaviour I prefer filtering-out any occurence of $pinspec here also
  return [remove_from_collection $right_dirpins $pinspec]
 } else {
  # No Timing arcs aware propagation
  set all_xpins [get_pins -o $pincell -fi "direction == $direction || direction == inout"]

  if {$all_xpins == ""} {
   echo "(go_thru) -E- Cell *$pincell_name* has NO ($direction/inout)-pin(s)."
   return {}
  }

  # Due to *inout* pins behaviour I prefer filtering-out any occurence of $pinspec here also
  return [remove_from_collection $all_xpins $pinspec]
 }
}
