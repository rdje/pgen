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

proc	get_c2pcon {args} {
 getoptions gc_arg $args

 array set gc_arg_so 	$gc_arg(SO)
 array set gc_arg_owv 	$gc_arg(OWV)


 if {[info exists gc_arg_so(help)]} {
  echo "usage: get_c2pcon --cell=<CellName> \[--output=<OutputFile>\] \[--quiet\] \[--help\]"
  return {}
 }

 # Default Values
 set quiet	0
 set cellioconn	"cellioconn.txt"
 set mycell	""

 if {[info exists gc_arg_so(quiet)   ]} {set quiet	1}
 if {[info exists gc_arg_owv(output)]} {set cellioconn	$gc_arg_owv(output)}
 if {![info exists gc_arg_owv(cell)]} {
   echo "\[get_c2pcon\] -E- The cell name have to be provided by means of the --cell option."
   echo "usage: get_c2pcon --cell=<CellName> \[--output=<OutputFile>\] \[--quiet\] \[--help\]"
   return {}
 } else {
  set mycell	$gc_arg_owv(cell)
 }

 # Is it a cell ?
 set cellcol	[get_cells -q $mycell]
 if {$cellcol != ""} {
  file delete $cellioconn
  
  set	inpincnt	0
  foreach_in_collection mypin [get_pins -fi "direction == in" -o $cellcol] {
   incr	inpincnt
   set	portlst {}
   foreach_in_collection inpad	[filter_collection [get_fanin $mypin] "object_class == port"] {
    lappend portlst	[get_attribute -q $inpad full_name]
   }
  
   echo [format "-i- , \[%d\] , %s , (%s)" $inpincnt [get_attribute $mypin full_name] [join $portlst ,]] >> $cellioconn
  }
  
  set	outpincnt	$inpincnt
  foreach_in_collection mypin [get_pins -fi "direction == out" -o $cellcol] {
   incr	outpincnt
   set	portlst {}
   foreach_in_collection outpad	[filter_collection [get_fanout $mypin] "object_class == port"] {
    lappend portlst	[get_attribute -q $outpad full_name]
   }
  
   echo [format "-o-, \[%d\] , %s , (%s)" $outpincnt [get_attribute $mypin full_name] [join $portlst ,]] >> $cellioconn
  }
 } else {
  if {!$quiet} {echo "\[get_c2pcon\] -E- Sorry but only full_name of existing cells are supported"}
 }
}
