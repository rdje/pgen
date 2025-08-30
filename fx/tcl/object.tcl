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

proc object {args} {
 getoptions go_args $args
 array set go_args_so $go_args(SO)

 if {[info exists go_args_so(help)]} {
  echo "usage: object \[--pin\] \[--port\] \[--cell\]\[--quiet\] \[--help\]"
  return {}
 }

 set object_spec [lindex $go_args(SA) 0]

 set check_for_pin  [info exists go_args_so(pin)]
 set check_for_port [info exists go_args_so(port)]
 set check_for_cell [info exists go_args_so(cell)]
 # TIming Path collection
 set check_for_tip  [info exists go_args_so(tip)]
 set quiet	    [info exists go_args_so(quiet)]

 if {($check_for_pin + $check_for_port + $check_for_cell + $check_for_tip) == 0} {
  if {!$quiet} {
   echo "usage: object \[--pin\] \[--port\] \[--cell\] \[--tip\] \[--quiet\] \[--help\]"
   return {}
  }
 }

 if {$check_for_pin} {
  # Is it a pin full_name ?
  set pinobj [get_pins -q $object_spec]
  if {$pinobj != ""} {return $pinobj}
  # So is it a collection ?
  set pinobj [get_attribute -q $object_spec object_class]
  if {$pinobj == "pin"} {return $object_spec}
 }

 if {$check_for_port} {
  # Is it a port full_name ?
  set portobj [get_ports -q $object_spec]
  if {$portobj != ""} {return $portobj}
  # So is it a collection ?
  set portobj [get_attribute -q $object_spec object_class]
  if {$portobj == "port"} {return $object_spec}
 }

 if {$check_for_cell} {
  # Is it a cell full_name ?
  set cellobj [get_cells -q $object_spec]
  if {$cellobj != ""} {return $cellobj}
  # So is it a collection ?
  set cellobj [get_attribute -q $object_spec object_class]
  if {$cellobj == "cell"} {return $object_spec}
 }

 if {$check_for_tip} {
  # So is it a Timing path collection ?
  set tipobj [get_attribute -q $object_spec object_class]
  if {$tipobj == "timing_path"} {return $object_spec}
 }

 return {}
}

