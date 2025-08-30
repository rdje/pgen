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

proc is_clock_pin	{object_spec} {
 set pinobject [object -pin $object_spec]

 if {$pinobject == ""} {
  echo "(is_clock_pin) -E- Only pin objects are supported."
  return 0
 } 

 # Get its cell
 set pincell [get_cells -q -o $pinobject]
 if {$pincell == ""} {
  echo "(is_clock_pin) -E- Pin *[get_attribute -q $pinobject full_name]* has NO Cell."
  return 0
 }

 if {[is_gsx0_reg $pincell]} {
  set pinlist	"CLK"
 } elseif {[is_gsx0_lat $pincell]} {
  set pinlist	"C CZ"
 } elseif {[is_gsx0_cgc $pincell]} {
  set pinlist	"CLKIN"
 } else {
  echo "(is_clock_pin) -E- Can't handle this type of cell *[get_attribute -q $pincell ref_name]*."
  return 0
 }

 set pincellname	[get_attribute -q $pincell full_name]
 foreach mypin $pinlist {
  set isapin	[get_pins -q "$pincellname/$mypin"]
  if {$isapin != ""} {
   if {[compare_collection $isapin $pinobject] == 0} {return 1}
  }
 }
 

 # last try before quitting
 # May be the timing_arcs will help
 
 # Retrieve all of the cell timing_arcs
 set alltiarcs	[get_timing_arcs -o $pincell -fi "sense =~ *setup_* || sense =~ *hold_*"]
 if {$alltiarcs == ""} {
  if {[is_gsx0_reg $pincell] || [is_gsx0_lat $pincell] || [is_gsx0_cgc $pincell]} {
   echo "(is_clock_pin) -E- Can't determine this status."
  } else {
   echo "(is_clock_pin) -E- Cell should be either a register or a latch."
  }
  
  return 0
 }

 # Let now get the from pin(s)
 set from_pins	{}
 foreach_in_collection mytiarc $alltiarcs {
  set from_pins [add_to_collection -unique $from_pins [get_attribute -q $mytiarc from_pin]]
 }
 
 foreach_in_collection myfrompin $from_pins {
  if {[compare_collection $myfrompin $pinobject] == 0} {
   return 1
  }
 }

 return 0
}

proc is_gsx0_mux_select_pin  {object_spec}	{
 set pinobject [object -pin $object_spec]

 if {$pinobject == ""} {
  echo "(is_gsx0_mux_select_pin) -E- Only pin objects are supported."
  return 0
 } 

 set pincell	[get_cells -q -o $pinobject]
 if {$pincell == ""} {
  echo "(is_gsx0_mux_select_pin) -E- Pin *[get_attribute -q $pinobject full_name]* has NO cell."
  return 0
 }

 set mux2to1	{(?i)CTGMU4|MU1.*} 
 set mux4to1	{(?i)MU2.*} 

 if {[regexp $mux2to1 [get_attribute -q $pincell ref_name]] || [is_gsx0_clkmux $pincell]} {
  set pinlist	"S EN"
 } elseif {[regexp $mux4to1 [get_attribute -q $pincell ref_name]]} {
  set pinlist	"A B SEL"
 } else {
  return 0
 }


 set pincellname	[get_attribute -q $pincell full_name]
 foreach myselpin $pinlist {
  set piname	"$pincellname/$myselpin"
  set isapin	[get_pins -q $piname]
  if {$isapin != ""} {
   if {[compare_collection $isapin $pinobject] == 0} {return 1}
  }
 }

 return 0
}
