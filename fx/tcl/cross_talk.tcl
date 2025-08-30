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

proc cross_talk {{fileout "cross_talk.log"}} {

 array set cell2domain	{}
 array set domain2clkpins {}
 array set source2clk {}
 # for all clock domain, retrieve all driven flops ans associate them with that clock
 foreach_in_collection myclk [get_clocks *] {
  set clkname		[get_attribute -q $myclk full_name]
  set clksources	[get_attribute -q $myclk sources]

  set	l_domain2clkpins	{}
  foreach_in_collection mysrc $clksources {
   set	source2clk([get_attribute -q $mysrc full_name]) $clkname
   # Get all fanout pins belonging to registers  or fanout ports
   echo "(cross_talk) -I- Retrieving Fanout nodes of pin *[get_attribute -q $mysrc full_name]* ($clkname)."
   set clkpinodes	{}
   foreach_in_collection mypinode [filter_collection [get_for -quiet $mysrc] "object_class == pin"]	{
    if {[is_clock_pin $mypinode]} {
     set clkpinodes [add_to_collection -unique $clkpinodes $mypinode]
    }
   }

   set l_domain2clkpins	[add_to_collection -unique $l_domain2clkpins $clkpinodes]

   foreach_in_collection mynode  $clkpinodes {
    # For each of this node get the cell and associate it with $clkname
    #
    # But only if it is a pin
    if {[get_attribute -q $mynode object_class] == "pin"} {
     set node_cellname	[get_attribute -q [get_cells -q -o $mynode] full_name]
     if {![info exists cell2domain($node_cellname)]} {
      echo "(cross_talk) -W- Cell *$node_cellname* put in domain ($clkname)"
      set cell2domain($node_cellname)	$clkname
     } else {
      echo "(cross_talk) -W- More than one domain for cell *$node_cellname* : 1=(cell2domain($node_cellname)) 2=($clkname)"
      #set cell2domain($node_cellname)	$clkname
     }
    } else {
     echo "(cross_talk) -I- Port *[get_attribute -q $mynode full_name]* in the fanout of '$clkname'"
    }
   }
  }

  set	domain2clkpins($clkname)	$l_domain2clkpins
 }


 file delete $fileout
 foreach mydomain [array names domain2clkpins] {
  echo "(cross_talk) -I- Processing domain *$mydomain*"
  foreach_in_collection mynode $domain2clkpins($mydomain) {
   # Get the fanout register of that node and also get their domain.
   # It case differ from the current then log that fact.
   set otherside_nodes	[go_thru -out $mynode]
   if {$otherside_nodes != ""} {
    foreach_in_collection myosn	$otherside_nodes {
     set nodefanout_reg	[get_for $myosn]

     # Skip this Other-side node in case it is a starting point of a clock definition
     if {[info exists source2clk([get_attribute -q $myosn full_name])]} {
      continue
     }

     if {$nodefanout_reg != ""} {
      foreach_in_collection myfn $nodefanout_reg {
       # Considering only pin (--> flop pins)
       if {[get_attribute -q $myfn object_class] == "pin"} {
        set  myfncell	[get_cells -q -o $myfn]
	if {$myfncell != ""} {
	 # Get the domain of that flop and check
	 set fncellname	[get_attribute -q $myfncell full_name]
	 if {![info exists cell2domain($fncellname)]} {
	  echo "(cross_talk) -E- Cell *$fncellname* can't be found in 'cell2domain' hash. It ain't attached to any existing domain."
	  continue
	 }
	 
	 set capture_domain	$cell2domain($fncellname)
	 if {$mydomain != $capture_domain} {
	  echo "(cross_talk) -CROSS- $mydomain:[get_attribute -q $myosn full_name] > $capture_domain:[get_attribute -q $myfn full_name]" >> $fileout
	 }
	} else {
         echo "(cross_talk) -I- Pin *[get_attribute -q $myfn full_name]* Has NO Cell."
	}
       } else {
        echo "(cross_talk) -I- Pin *[get_attribute -q $myosn full_name]* Has port *[get_attribute -q $myfn full_name]* in its fanout."
       }
      }
     } else {
      echo "(cross_talk) -W- Pin *[get_attribute -q $myosn full_name]* Has no fanout regs/ports."
     }
    }
   } else {
    echo "(cross_talk) -W- No other-side Node for pin *[get_attribute -q $mynode full_name]*"
   }

  }
 }
}

