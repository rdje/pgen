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

proc write_clockinfo {args} {
 getoptions wc_arg $args

 array set wc_so  $wc_arg(SO)
 array set wc_owv $wc_arg(OWV)

 if {[info exists wc_so(help)]} {
  echo "usage: write_clockinfo \[--out=<OutputFileName>\] \[--help\]"
  return 
 }

 # Default
 set outputfile  "clockinfo.lof"
 if {[info exists wc_owv(out)]} {set outputfile $wc_owv(out)}
 
 file delete $outputfile

 set clockinfo {}
 array set clockhash {}
 echo "=clockinfo=" >> $outputfile
 foreach_in_collection myclk [get_clocks *] {
  set l_clkinfo		{}

  set clkname 		[get_attribute -q $myclk full_name]
  set period  		[get_attribute -q $myclk period]
  set period  		[expr {$period != "" ? $period : "-"}]

  set propagated_clock 	[get_attribute -q $myclk propagated_clock]
  set propagated_clock  [expr {$propagated_clock != "" ? "propagated" : "ideal"}]

  set setup_uncertainty [get_attribute -q $myclk setup_uncertainty]
  set setup_uncertainty [expr {$setup_uncertainty != "" ? $setup_uncertainty : "-"}]

  set hold_uncertainty 	[get_attribute -q $myclk hold_uncertainty]
  set hold_uncertainty [expr {$hold_uncertainty != "" ? $hold_uncertainty : "-"}]

  set sources	 	[get_attribute -q $myclk sources]

  set sources_info	{}
  foreach_in_collection mysource $sources {
    set source_name 		[get_attribute -q $mysource full_name]
    set source_type 		[get_attribute -q $mysource object_class]
    set source_direction	[get_attribute -q $mysource direction]

    lappend sources_info [list $source_name $source_direction $source_type]
    set clockhash($source_name) $clkname
  }

  # It is not nice but currently I have no other shoice.
  # I assume that there is at most one source clock per clock definition.
  if {$sources_info == ""} {set sources_info "- - -"}

  set masterclks {}
  if {1} {
   # Missing master clock
   #echo "(get_clockinfo) -W- Clock source 'clkname' has no defined Master Clock."
   foreach mysrc_info $sources_info {
    set srcname [lindex $mysrc_info 0]	  
    set srcdir  [lindex $mysrc_info 1]	  
    set srctype [lindex $mysrc_info 2]	  


    if {$srctype == "pin"} {
     echo "\n(get_clockinfo) -I- ($clkname) Searching for potential primary clock(s) leading to pin '$srcname'.."
     if {$srcdir == "in"} {
      # clock source attached on an input.
      # This SHOULD NOT happen. This is an error;
      echo "(get_clockinfo) -W- Wierd. A clock source is attached on *input* pin '$srcname'."
      echo "(get_clockinfo1) -I- Calling get_fir on clk-pin '$srcname'"
      set retv [get_fir --nomuxsel $srcname] 
      query_object $retv
      if {$retv != ""} {
       # Keeping only returned node that are either an input port or and existing clock source.
       foreach_in_collection mygms $retv {
        if {[get_attribute -q $mygms object_class] == "port"} {
         set portname [get_attribute -q $mygms full_name]
         lappend masterclks $portname
        } elseif {[info exists clockhash([get_attribute -q $mygms full_name])]} {
         set name [get_attribute -q $mygms full_name]
         lappend masterclks $clockhash($name) 
        } else {
         echo "(get_clockinfo) -W- Pin '[get_attribute -q $mygms full_name]' is a potential Master clock."
         lappend masterclks [get_attribute -q $mygms full_name]
	}
       }
      } else {
       echo "(get_clockinfo) -W- No potential primary clock(s) seen from input pin '$srcname'."
      }
     } else {
      # clock source attached on an output pin (of a flop, or ...)
      set pincell 	[get_cells -q -o $srcname]
      set pincell_name 	[get_attribute -q $pincell full_name]
      if {$pincell != ""} {
       if {[is_gsx0_reg $pincell]} {
        set clkpin [get_pins -q -fi "is_clock_pin == true && direction == in" -o $pincell]
	query_object $clkpin
	if {$clkpin != ""} {
         echo "(get_clockinfo2) -I- Calling get_fir on clk-pin '[get_attribute  $clkpin full_name]'"
         set retv [get_fir --nomuxsel $clkpin] 
         query_object $retv
         if {$retv != ""} {
          # Keeping only returned node that are either an input port or and existing clock source.
          foreach_in_collection mygms $retv {
	   set mygms_name	[get_attribute $mygms full_name]
           echo "(get_clockinfo2) -I- IN LOOP for '$mygms_name'"
	   if {[get_attribute -q $mygms object_class] == "port"} {
            echo "(get_clockinfo2) -I- Yes Found a port as SOURCE"
            lappend masterclks $mygms_name
	   } elseif {[info exists clockhash($mygms_name)]} {
            echo "(get_clockinfo2) -I- Yes Found a OUTPUT PIN as SOURCE($clockhash($mygms_name))"
            lappend masterclks $clockhash($mygms_name) 
	   } else {
	    echo "################ DEFPOINT($mygms_name) is not recognized ################3"
            echo "(get_clockinfo) -W- Pin '$mygms_name' is a potential Master clock."
            lappend masterclks $mygms_name
	   }
          }
         } else {
          echo "(get_clockinfo) -W- No potential primary clock(s) seen from clock-pin '[get_attribute -q $clkpin full_name]'."
         }
        } else {
          echo "(get_clockinfo) -W- Register '$pincell_name' has NO identifiable clock-pin using attribute."
          echo "(get_clockinfo) -I- Trying '$pincell_name/CLK'.."
	  set isclkpin [object -pin "$pincell_name/CLK"]
          if {$isclkpin != ""} {
           echo "(get_clockinfo) -I- Guessed right."
           echo "(get_clockinfo3) -I- Calling get_fir on clk-pin '[get_attribute -q $isclkpin full_name]'"
           set retv [get_fir --nomuxsel $isclkpin] 
           if {$retv != ""} {
            # Keeping only returned node that are either an input port or and existing clock source.
            foreach_in_collection mygms $retv {
	     if {[get_attribute -q $mygms object_class] == "port"} {
	      set portname [get_attribute -q $mygms full_name]
              lappend masterclks $portname
	     } elseif {[info exists clockhash([get_attribute -q $mygms full_name])]} {
	      set name [get_attribute -q $mygms full_name]
              lappend masterclks $clockhash($name) 
	     } else {
              echo "(get_clockinfo) -W- Pin '[get_attribute -q $mygms full_name]' is a potential Master clock."
              lappend masterclks [get_attribute -q $mygms full_name]
	     }
            }
           } else {
           echo "(get_clockinfo) -W- No potential primary clock(s) seen from clock-pin '[get_attribute -q $isclkpin full_name]'."
           }
          } else {
           echo "(get_clockinfo) -I- No Luck."
	  }
        }
       } else {
        echo "(get_clockinfo) -W- Clock source '$clkname' is not attached to a Register output pin."
        echo "(get_clockinfo)     Corresponding cell is of type '[get_attribute -q $pincell ref_name]'."
        foreach_in_collection osnode [go_thru -in $srcname] {
         if {[is_gsx0_mux_select_pin $osnode]} {
          echo "(get_clockinfo) -I- Skipping mux select/enable pin '[get_attribute -q $osnode full_name]'."
	  continue
	 }

         echo "(get_clockinfo) -I-  FIR of otherside node '[get_attribute -q $osnode full_name]'."
         set fanin_stuff [get_fir --nomuxsel $osnode]
         if {$fanin_stuff != ""} {
           foreach_in_collection mygms $fanin_stuff {
	    if {[get_attribute -q $mygms object_class] == "port"} {
	     set portname [get_attribute -q $mygms full_name]
             lappend masterclks $portname
	    } elseif {[info exists clockhash([get_attribute -q $mygms full_name])]} {
	     set name [get_attribute -q $mygms full_name]
             lappend masterclks $clockhash($name) 
	    } else {
             echo "(get_clockinfo) -W- Pin '[get_attribute -q $mygms full_name]' is a potential Master clock."
             lappend masterclks [get_attribute -q $mygms full_name]
	    }
           }

	 }
	}
       }
      } else {
       echo "(get_clockinfo) -W- Pin 'srcname' has NO cell."
      }
     }
    } else {
     # srctype is a port
     # Nothing to be done in this case
     echo "(get_clockinfo) -I- ($clkname) Searching for potential primary clock(s) leading to pin '$srcname'.."
    }
   }
  }

  lappend l_clkinfo $clkname $period $propagated_clock $setup_uncertainty $hold_uncertainty
  foreach mysrcent $sources_info {
   foreach myinfo $mysrcent {
    lappend l_clkinfo $myinfo
   }
  }

  # potential master w/o slash in their name should appear first
  set w_slash	{}
  set wo_slash	{}
  foreach mymaster $masterclks {
   if {[regexp {/} $mymaster]} {
     lappend w_slash	$mymaster
   } else {
     lappend wo_slash	$mymaster
   }
  }

  set sortedmasters	{}
  push sortedmasters [lsort -dictionary $wo_slash]
  push sortedmasters [lsort -dictionary $w_slash]

  push l_clkinfo $sortedmasters
  #foreach mymaster $masterclks {lappend l_clkinfo $mymaster}
  
  lappend clockinfo $l_clkinfo
 }

 echo [join [lsort -dictionary $clockinfo] \n] >> $outputfile
 echo "=clockinfoend=" >> $outputfile

 foreach myclk [array names clockhash] {
  if {[info exists clockhash($myclk)]} {
  echo "DEFPOINT($myclk) : $clockhash($myclk)"
  } else {
  echo "WEIRD DEFPOINT($myclk) : is not recognized !!!"
  }
 }
}
