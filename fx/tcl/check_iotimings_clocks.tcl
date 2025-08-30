#----------------------------------------------------------------------------------------------
# Author	: Richard DJE
#
# Description	: *check_iotimings_clocks* is a routine aimed at checking timing path endpoint 
#		  clocks for input paths, and startpoint clocks for output paths.
#		  The format used is the one returned by *get_ctinfo*, 
#
#----------------------------------------------------------------------------------------------
# Property of Texas Instruments - For  Unrestricted Internal Use  Only.
# Unauthorized reproduction and/or distribution is strictly prohibited.  This
# product is protected under copyright law and trade secret law as an unpublished
# work. (C) Copyright 2005 Texas Instruments. All rights reserved.
#----------------------------------------------------------------------------------------------
package provide	stapack 1.0


proc	check_iotimings_clocks	{iotimings} {
  array	set	io_timings	$iotimings
  set		tip		$io_timings(tip)

  # Retrieving all declared clock sources
  array	set	clockinfo	{}
  set smallest_period	10000
  foreach_in_collection myclkobj [get_clocks *] {
    set clkname	[get_attribute -q $myclkobj full_name]
    set period	[get_attribute -q $myclkobj period]

    # Now I should retrieve the pin(s)/port(s) to which it is applied
    set	pinportcol	[get_attribute -q $myclkobj sources]
    set	colsize		[sizeof_collection $pinportcol]
    if {$colsize > 1} {
     echo "\[check_iotimings_clocks\] -W- Clock '$clkname' associated with several pins/ports."
    } elseif {$colsize == 0} {
     echo "\[check_iotimings_clocks\] -W- Clock '$clkname' has no generated source(s) (virtual-clock ?)."
    }

    if {$colsize} {
     if {$period < $smallest_period} {
       if {0} {echo "\[check_iotimings_clocks\] -I- New smallest_period(clkname=$clkname): '$period' < '$smallest_period'"}
       set smallest_period $period
     }
    }

    array set clockinfo [list  [get_attribute -q $pinportcol full_name] $clkname]
  }
  
  # Checking for input paths.
  # The Capturing clock should not be 'Undefined'
  # If so, then we have to Check for the type of
  # endpoint we have. 
  # It is a register data pin ?
  # It is a register clock pin ?
  # It is a combinational cell pin ?
  # Or something else ?
  #
  # For all these checks we will be using of one the
  # following routine:
  # - is_gsx0_reg
  # - is_gsx0_cgc
  # - is_gsx0_lat
  echo "\[check_iotimings_clocks\] -I- Input paths Check.."
  foreach myipath [lindex $tip 0] {
   # Retrieve the endpoint clock
   set epclk	[lindex $myipath 3]
   # Skip this iteration in case the epclk is defined
   if {$epclk != "Undefined"} {continue}

   # Here, we need to get the endpoint full_name, and from that its
   # cell .
   set epname		[lindex $myipath 1]
   set epcell		[get_cells -q -o $epname]
   set epcellname	[get_attribute -q $epcell full_name]
   
   # What kind of cell do we have ?
   if {[is_gsx0_reg $epcell]} {
    # A gsx0 register
    #
    # The clock should be something like '$epcellfull_name/CLK'
    # let's check.
    set clockpiname	"[get_attribute -q $epcell full_name]/CLK"
    set clockpin   	[get_pins $clockpiname]
    if {$clockpin != ""} {
     # Y-e-s, let's continue then
     # Calling get_primary_clocks should retrieve the name(s) of its
     # potential primary clock sources
     #
     # Normally there should only one of such potential clock source
     set prim_clocks	[get_primary_clocks $clockpin]
     set prim_clocks_sz	[sizeof_collection $prim_clocks]
     if {$prim_clocks_sz == 1} {
      # There is only one potential primary clock source
      # may be it is not attached to any clock source yet
      #
      # let's check.
      set primname	[get_attribute -q $prim_clocks full_name]
      if {![info exists clockinfo($primname)]} {
       # No clock source is attached to this potential primary source
       #
       # Let's doing it
       if {0} {
        create_clock -p $smallest_period $primname
        # it should be propagated
        set_propagated_clock 	$primname
        echo "\[check_iotimings_clocks\] -I- Created and propagated clock '$primname'."
       }
      }
     } else {
      echo "\[check_iotimings_clocks\] -WW- More than one potential primary clock source for '$clockpiname'."
      query_object $prim_clocks
     }
    } else {
     echo "\[check_iotimings_clocks\] -W- Pin does not exists '$clockpiname'"
    }

   } elseif {[is_gsx0_cgc $epcell]} {
    # A gsx0 clock gating cell
    echo "\[check_iotimings_clocks\] -I- Cell '$epcellname' is a Clock-Gating Cell."
   } elseif {[is_gsx0_lat $epcell]} {
    # A gsx0 latch
    echo "\[check_iotimings_clocks\] -I- Cell '$epcellname' is a latch."
   } elseif {[is_gsx0_apd $epcell]} {
    echo "\[check_iotimings_clocks\] -I- Cell '$epcellname' is an Antenna Protection Diode."
   } elseif {[get_attribute -q $epcell is_combinational] == "true"} {
    echo "\[check_iotimings_clocks\] -I- Cell '$epcellname' is combinational :[get_attribute -q $epcell ref_name]:"
   } else {
    echo "\[check_iotimings_clocks\] -I- Cell '$epcellname' is Unknown :[get_attribute -q $epcell ref_name]:"
   }
  }
}
