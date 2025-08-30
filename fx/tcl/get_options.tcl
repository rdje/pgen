#-------------------------------------------------------------------------------------------
# Author	: Richard DJE
#
# Description	: Routines for retrieving command options and arguments
#-------------------------------------------------------------------------------------------
# Property of Texas Instruments - For  Unrestricted Internal Use  Only.
# Unauthorized reproduction and/or distribution is strictly prohibited.  This
# product is protected under copyright law and trade secret law as an unpublished
# work. (C) Copyright 2005 Texas Instruments. All rights reserved.
#-------------------------------------------------------------------------------------------
package provide	stapack 1.0

proc getoptions {getopt arglist} {
 upvar 1 $getopt 	go_o


 set simpleopt		{^--?([a-zA-Z0-9_-]+)$}
 set option_eq_val	{^--?([a-zA-Z0-9_-]+)=(.+)$}

 # simple options hash
 array set so_a	{}

 # options w/ value hash
 array set owv_a	{}

 foreach myarg $arglist {
  set	capstr	{}
  set	optname	{}
  set	optionval	{}
  if {[regexp -- $simpleopt $myarg capstr optname]} {
   if {[info exists so_a($optname)]} {
    incr so_a($optname)
   } else {
    set so_a($optname)	1
   }

   if {[info procs $optname] != {}} {$optname}
  } elseif {[regexp -- $option_eq_val $myarg capstr optname optionval]} {
   lappend owv_a($optname) $optionval

  } else {
   # Simple arguments, i.e non-options, are directly appended
   lappend go_o(SA)  $myarg
  }
 }

 array set go_o [list SO  [array get so_a]]
 array set go_o [list OWV [array get owv_a]]
}


proc getopt_so {getopt arglist} {
 upvar 1 $getopt 	go_o

 set simpleopt		{^--?([a-zA-Z0-9_-]+)$}

 foreach myarg $arglist {
  set	capstr	{}
  set	optname	{}
  if {[regexp -- $simpleopt $myarg capstr optname]} {
   set go_o($optname) 1
  }
 }
}

