#-------------------------------------------------------------------------------------------
# Author	: Richard DJE
#
# Description	: *get_fanin* and *get_fanout* are there for retrieving all pin/port seen in
#		  the transitive fanin/fanout of a provided pin/port argument.
#		  
#		  The only purpose of these routine is to propagate through combinational 
#		  logic and stop either on sequential cells in/out-put pins at in/out-put 
#		  ports depending on the routine being called.
#
#-------------------------------------------------------------------------------------------
# Property of Texas Instruments - For  Unrestricted Internal Use  Only.
# Unauthorized reproduction and/or distribution is strictly prohibited.  This
# product is protected under copyright law and trade secret law as an unpublished
# work. (C) Copyright 2005 Texas Instruments. All rights reserved.
#-------------------------------------------------------------------------------------------
package provide	stapack 1.0

proc	get_fanin	{args} {
 getoptions gf_args $args

 array set optlist $gf_args(SO)
 array set optlist $gf_args(OWV)

 if {[info exists optlist(help)] || [llength $gf_args(SA)] == 0} {
  echo "usage: get_fanin \[options\]* <pinport_object_spec>"
  echo "       options =  --notiarcs | --showmux | --nomuxsel | --tpoints=<ListName> | --maxlevel=<MaxLevel>"
  echo "                  --flush    | --unique-disable       | --quiet    | --debug | --help"
  return {}
 }
 
 if {[info exists optlist(tpoints)]} {
  upvar $optlist(tpoints) uplist
  set	optlist(tpoints)  uplist
 }

 get_fanx [lindex $gf_args(SA) 0] in optlist
}

proc	get_fanout	{args} {
 getoptions gf_args $args
 
 array set optlist $gf_args(SO)
 array set optlist $gf_args(OWV)

 if {[info exists optlist(help)] || [llength $gf_args(SA)] == 0} {
  echo "usage: get_fanout \[options\]* <pinport_object_spec>"
  echo "       options =  --notiarcs | --showmux | --nomuxsel | --tpoints=<ListName> | --maxlevel=<MaxLevel>"
  echo "                  --flush    | --unique-disable       | --quiet    | --debug | --help"
  return {}
 }

 if {[info exists optlist(tpoints)]} {
  upvar $optlist(tpoints) uplist
  set	optlist(tpoints)  uplist
 }

 get_fanx [lindex $gf_args(SA) 0] out optlist
}

proc	get_fanx	{pinorport direction optlist} {
upvar	$optlist opt_list


 # Options
 set debug	[info exists opt_list(debug)]
 set quiet	[info exists opt_list(quiet)]

 if {![regexp "^(?:in|out)$" $direction]} {
  if {!$quiet} {echo "\[get_fanx\] -E- Only supported directions are *in* and *out*"}
  return  {}
 }


 array set get_fanx_visited {}

 if {$debug} {echo "\[get_fan$direction\] -I- <<<<<<<<<<<<< \[$pinorport\] \[$direction\] >>>>>>>>>>>>>>"}

 set ydir	[expr {$direction == "in" ? "out" : "in"}]
 set pin_port	[filter_collection [object -pin -port $pinorport] "direction == inout 				     || \
 								   (object_class == pin				   ) || \
								   (object_class == port && direction == $ydir     )"]
 if {$pin_port != ""} {

  if {[info exists opt_list(maxlevel)]} {
   if {[info exists opt_list(tpout)]} {file delete $opt_list(tpout)}

   set	returnv	[mlce_get_fanx $pin_port get_fanx_visited $direction opt_list ""]

  } else {
   if {[info exists opt_list(tpoints)]} {
    upvar $opt_list(tpoints)	uplist
    set	 opt_list(tpoints)	uplist

    lappend opt_list(curtp)	[get_attribute -q $pin_port full_name]
   }

   set	returnv	[ce_get_fanx $pin_port get_fanx_visited $direction opt_list]

   if {[info exists opt_list(tpoints)]} {
    # Here the PUSH command is important.
    # unlike the Tcl Lappend, the home-made
    # PUSH command behaves like is perl conterpart.
    if {$returnv != {}} {
     push uplist $opt_list(gtp)
     set opt_list(gtp) {}
    }
   }
  }
 } else {
  #echo "(get_fan$direction) -E- Only $direction/inout pin and $ydir/inout port objects are supported."
  echo "(get_fan$direction) -E- Only in/out/inout pin and $ydir/inout port objects are supported."
  set	returnv {}
 }

 return $returnv
}

# Max Level ce_get_fanx
proc	mlce_get_fanx	{pinorport  getfanx_visited sense optlist indent} {
upvar 1 $optlist		opt_list
upvar 1 $getfanx_visited 	getfanxvisited

set debug	[info exists opt_list(debug)]

 if {[info exists opt_list(tpoints)]} {
  upvar $opt_list(tpoints)	uplist
  set	 opt_list(tpoints)	uplist
  
  lappend opt_list(curtp)	[get_attribute -q $pinorport full_name]
 }

 if {0} {echo "$indent (mlce_get_fanx)-E-([get_attribute -q $pinorport full_name])"}
 set curun	   [ce_get_fanx $pinorport getfanxvisited $sense opt_list]

 # Using PUSH instead of Lappend is mandatory. Things won't work otherwise.
 if {[info exists opt_list(tpoints)] && $curun != {}} {
  if {[info exists opt_list(flush)]} {push uplist	$opt_list(gtp)}

  #if {[info exists opt_list(tpout)]} {
  # echo $opt_list(gtp) >> $opt_list(tpout)
  #}

  set opt_list(gtp) {}
 }

 if {0} {echo "$indent (mlce_get_fanx) -I- Filtering .."}
 set combinational_set	{}
 set returnv		{}


 # From $curun get all connected cells.
 # From this cells set keep only combinational, assuming this attribute is well set.
 # From this combinational cells set retrieve all connected pins (--> $acombpin_set).
 # Now to extract all sequential pins and ports out of $curun in just one go, I only need
 # to substract $acombpin_set from $curun.
 #
 # Similarly, all returned combinational pins are retrieved by substracting $returnv from $curun.
 set acombpin_set       [get_pins -q -o [get_cells -q -fi "is_combinational == true" -o $curun]]
 set returnv		[remove_from_collection $curun $acombpin_set]
 set combinational_set	[remove_from_collection $curun $returnv]

 if {0} {echo "$indent (mlce_get_fanx) -I- Done."}
 # We simply return if no more combinational pin were found at this level
 if {$combinational_set == ""} {
  if {0} {echo "$indent (mlce_get_fanx)-L-([get_attribute -q $pinorport full_name])"}
  return $returnv
 }

 # Otherwise, we should iterate on each of them, recursively calling *mlce_get_fanx*
 # mlce_get_fanx returned value is either an empty collection or a collection of
 # sequential element pins or interface ports.
 set newindent	"$indent          | "
 echo $newindent
 foreach_in_collection mycombpin $combinational_set {
    set returnv [add_to_collection -u $returnv [mlce_get_fanx $mycombpin getfanxvisited $sense opt_list $newindent]]
 }

 # Once we processed all of these combinational pins, $returnv contains all sequential pins/ports
 # found at this level in the addition of those found at lower levels when recursively calling
 # *mlce_get_fanx*
 if {0} {echo "$indent (mlce_get_fanx)-L-([get_attribute -q $pinorport full_name])"}
 return $returnv
}


proc	ce_get_fanx	{pinorport  getfanx_visited sense optlist}	{
upvar 1 $optlist		opt_list
upvar 1 $getfanx_visited 	getfanxvisited

 if {[info exists opt_list(maxlevel)]} {
  set maxlevel $opt_list(maxlevel)
  if {![info exists opt_list(curlevel)]} {set curlevel 1
  } else {
   set curlevel $opt_list(curlevel)
  }
 }

 # Default values
 set tiarcs_enabled	1
 if {[info exists opt_list(notiarcs)] || [info exists maxlevel]} 	{set tiarcs_enabled 0}

 set debug 	    [info exists opt_list(debug)]
 set nomux_sel	    [info exists opt_list(nomuxsel)]
 set quiet	    [info exists opt_list(quiet)]
 set show_mux	    [info exists opt_list(showmux)]
 set unique 	    [expr {[info exists opt_list(unique-disable)] ? 0 : 1}]


 # Initializing $fanx_col collection variable
 set	fanx_col	{}

 # No need to continue if $pinorport has already been visited
 set pinorport_name	[get_attribute  -q $pinorport full_name]
 if {$unique && [info exists getfanxvisited($pinorport_name)]} {
  if {[info exists opt_list(tpoints)]} {pop opt_list(curtp)}

  return $fanx_col
 }

 # For dealing with I/O pins/ports we should mark the $pinorport as VISITED otherwise
 # loops may appear.
 if {$unique} {set getfanxvisited($pinorport_name) 1}

 # Retrieving all nets connected to $pinorport
 #
 # Important Note: 
 # I struggled a while before finding that the point I was missing was the
 # necessary use I had to do regarding options *-top_net_of_hierarchical_group*
 # and *-segments*, for more info please read the *get_nets* command man page
 # specially section discussing these two aforementioned options.
 # Without this two options the get_nets was always returning nets only at
 # lower level of hierarchy ignoring the top level one.
 # Y-e-s-s-s !
 set	nets	[get_nets -q -top -seg -o $pinorport]
 
 set	direction	$sense

 if {$nets != ""} {
  # The collection ain't empty
  if {$debug} {echo "(ce_get_fanx) -I- *$pinorport_name* \
  is connected to net *[get_attribute -q $nets full_name]* direction($direction)"}

  # Now we should get all connected pins/ports
  # of the right direction (y), that is opposite to that
  # of $pinorport (x)
  if {$debug} {echo "(ce_get_fanx) -I- Now we should get all connected pins/ports of the right direction (y)"}

  set ydir	[expr {$direction == "in" ? "out" : "in"}]
  
  # 
  # The *-leaf* option is MANDATORY otherwise the WON'T get the right set of pins.
  # It will force the get_pins command to Cross module boundaries so as to retrieve
  # leaf cell pins connected to this net.
  #
  set ypins	[remove_from_collection [get_pins -leaf -q -f "direction == $ydir || direction == inout" -o $nets] $pinorport]
  
  if {$debug} {
   foreach_in_collection mypin $ypins {
    echo "(ce_get_fanx) -I- ypin([get_attribute -q $mypin full_name]) : Direction([get_attribute -q $ypins direction])"
   }
  }

  if {$ypins != ""} {
   # There are (y)pins to play with

   if {$debug} {echo "(ce_get_fanx) -I- There are (y)pins to play with"}
   foreach_in_collection myypin $ypins {
    set myypin_name  [get_attribute -q $myypin full_name]
    if {$debug} {echo "(ce_get_fanx) -I- \[Iterating on Y-pins\] (y-$ydir/inout)pin *$myypin_name* iteration"}

    # Before trying to get the associated cell we need to first check
    # if we already processed that (y)pin. just skip that loop in case
    # it was visited already.
    if {$unique && [info exists getfanxvisited($myypin_name)]} {continue}

    # Now for each of those (y)pins get the associated cell
    if {$debug} {echo "(ce_get_fanx) -I- Now for each of those (y)pins get the associated cell"}

    set cell	[get_cells -q -o $myypin]

    if {$cell == ""} {
     echo "(ce_get_fanx) -I- Pin *$myypin_name* has NO cell."
     if {$unique} {set getfanxvisited($myypin_name)	1}
     continue
    }

    if {$debug} {echo "(ce_get_fanx) -I- We found cell *[get_attribute -q $cell full_name]*"}

    # Is that cell combinational ?
    if {[get_attribute -q $cell is_combinational]} {
     if {$debug} {echo "(ce_get_fanx) -I- Yes, more fun to come, We have a combinational cell"}
     # Yes, more fun to come
     # Now let's retrieve all of its (x)pins, that is pins
     # having the same direction as that of $pinorport
      
     if {$tiarcs_enabled} {
      # Well, here I shouldn't bindly propagate through the cell.
      # I should check for the availability of timing_arcs between
      # the given $myypin and any of $cell (x)pins, and that those
      # timing_arcs are not disabled, and also not *getfanxvisited*.
      set celltiarcs_dir	[expr {$ydir == "out" ? "-to" : "-from"}]
      set getiarcs_cmd	"get_timing_arcs -q -fi \"is_disabled == false\" -o $cell"
      set celltiars	[eval $getiarcs_cmd]
      
      # There exist cells w/o any timing arcs, the *getfanxvisited* attribute
      # might also have filtered all them out, or for whatever raisons.
      if {$celltiars != ""} {
       # We are lucky here, so let's continue
       #
       # Now I have to retrieve all (x)pins defined in $celltiars
       set celltiarcs_xpins	{}
       if {$debug} {echo "(ce_get_fanx) -I- Retrieving (X-$direction)pins from celltiars timing_arcs collection"}

       foreach_in_collection myxpinarcs $celltiars {
        set right_dir	[expr {$direction == "in" ? "from_pin" : "to_pin"}]
	set ypin_dir	[expr {$direction == "in" ? "to_pin"   : "from_pin"}]
	set ypin_name	[get_attribute -q [get_attribute -q $myxpinarcs $ypin_dir] full_name]
	if {$ypin_name == $myypin_name} {
         set celltiarcs_xpins [add_to_collection -unique $celltiarcs_xpins [get_attribute -q $myxpinarcs $right_dir]] 
	}
       }

       if {$debug} {
        query_object $celltiarcs_xpins
       }

       if {[sizeof_collection $celltiarcs_xpins] == 0} {
        if {$debug} {echo "(ce_get_fanx) -I- No '$right_dir' pins found in Celltiars of cell '[get_attribute -q $cell full_name]' linked to pin \
        '[get_attribute -q $myypin full_name]'!"}
       }

       if {$show_mux} {
        if {[is_gsx0_mux $cell]} {echo "(ce_get_fanx) -I.MUX- Processing '[get_attribute -q $cell full_name]'."}
       }

       # There is no need to check for emptyness of $celltiarcs_xpins
       #
       # Now that we have all the (x)pins to propagate through all of them.
       if {[info exists opt_list(tpoints)]} {lappend opt_list(curtp)	$myypin_name}

       foreach_in_collection myxpin $celltiarcs_xpins {
        # This is for handling I/O pins/ports Loop issue.
        if {$unique && [info exists getfanxvisited([get_attribute -q $myxpin full_name])]} {
	 if {$debug} {echo "(ce_get_fanx) -I- ([get_attribute -q $myxpin direction])-pin '[get_attribute -q $myxpin full_name]' Already Seen. Skipping."}
	 continue
	}
	
	# skipping this pin in case *--nomuxsel* is on and it is mux select/enable pin
	if {$nomux_sel} {
         if {[is_gsx0_mux_select_pin $myxpin]} {
	  # Logic for not propagating through multiplexers selection/enable pin(s)
	  if {$show_mux} {echo "(ce_get_fanx) -I.MUXSELPIN- Pin '[get_attribute -q $myxpin full_name]' is a Mux select/enable pin. Skipping."}
	  continue
	 }
	}

        # Foreach each of those pins, recursively call ce_get_fanx and append 
        # the result to the $fanx_col collection variable
        if {$debug} {
	 echo "(ce_get_fanx) -I- \[Recursion on X-pins\] Foreach each of those (x-$direction)pins, recursively call ce_get_fanx"
         echo "(ce_get_fanx)      with myxpin '[get_attribute -q $myxpin full_name]' pin"
	}

        if {$debug} {echo "(ce_get_fanx) -I- Object Class is '[get_attribute -q $myxpin object_class]'"}
        if {[info exists opt_list(tpoints)]} {
	 lappend opt_list(curtp)	[get_attribute -q $myxpin full_name]
	}

        #
        # Infact *From_pin* and *To_pin* of timing_arcs may be either PIN or PORT.
        # I was previously thinking that timing_arcs from/to pin were only PINs, 
        # I was wrong. Sigh!
        #
        if {[get_attribute -q $myxpin object_class] == "port"} {
	 if {[info exists opt_list(tpoints)]} {
	  if {[info exists opt_list(tpout)]} {
	   echo "{$opt_list(curtp)}" >> $opt_list(tpout)
  	  } else {
	   lappend opt_list(gtp)	$opt_list(curtp)
  	  }
         }

         if {[info exists opt_list(tpoints)]} {pop opt_list(curtp)}

         set	fanx_col	[add_to_collection -unique $fanx_col $myxpin]
        } else {
         set	fanx_col	[add_to_collection -unique $fanx_col [ce_get_fanx $myxpin getfanxvisited $sense opt_list]]
        }
       }

       if {[info exists opt_list(tpoints)]} {pop opt_list(curtp)}

       # Now we need to mark the current (y)pin as visited, for avoiding triggering
       # recursion when seeing it next time.
       # array set getfanxvisited	[list [get_attribute -q $myypin full_name] 1]
      } else {
       # We found no non-disabled timing arcs to/from pin $myypin
       set arcdir	[expr {$ydir == "out" ? "to" : "from"}]
       if {!$quiet} {echo "\[get_fan$direction\] -I- No *non-disabled* timing arcs $arcdir pin *$myypin_name* !"}

       # Adding pin to the list of collected nodes
       set	fanx_col	[add_to_collection -unique $fanx_col $myypin]

       # continue <<< Useless
      }

      # Moved the above command here
      if {$unique} {set getfanxvisited($myypin_name) 1}
     } else {
      # Here we have the old-fashion propagation mode, which totally ignore the timing_arcs concept
      #
      # let's retrieve all (x)pins/port + inout pins/ports of current cell
      set xpinports	[remove_from_collection [get_pins -q -fi "direction == $direction || direction == inout" -o $cell] $myypin]
      
      if {[info exists maxlevel]} {
       if {$curlevel == $maxlevel} {
        set	fanx_col	[add_to_collection -unique $fanx_col $xpinports]


        if {[info exists opt_list(tpoints)]} {
	 # Pushing $myypin
         lappend opt_list(curtp)	$myypin_name

         foreach_in_collection myxpinport $xpinports {
          lappend opt_list(curtp)	[get_attribute -q $myxpinport full_name]
  #echo "-myxpinport-ReachedLevel-[get_attribute -q $myxpinport full_name]"
	  if {[info exists opt_list(tpout)]} {
	   echo "{$opt_list(curtp)}" >> $opt_list(tpout)
  	  } else {
	   lappend opt_list(gtp)	$opt_list(curtp)
  	  }

          pop opt_list(curtp)
 	 }

	 # Poping $myypin
         pop opt_list(curtp)
        }

        # One never Know :)
        if {$unique} {set getfanxvisited($myypin_name) 1}

	continue
       }
      }

      if {$xpinports != ""} {
        if {$show_mux} {
         if {[is_gsx0_mux $cell]} {echo "(ce_get_fanx) -I.MUX- Processing '[get_attribute -q $cell full_name]'."}
        }

        if {[info exists opt_list(tpoints)]} {lappend opt_list(curtp)	$myypin_name}

        foreach_in_collection myxpinport $xpinports {
	 # This is for handling I/O pins/ports Loop issue.
         if {$unique && [info exists getfanxvisited([get_attribute -q $myxpinport full_name])]} {continue}
 	 
	 if {$nomux_sel} {
          if {[is_gsx0_mux_select_pin $myxpinport]} {
	   # Logic for not propagating through multiplexers selection/enable pin(s)
	   if {$show_mux} {echo "(ce_get_fanx) -I.MUXSELPIN- Pin '[get_attribute -q $myxpinport full_name]' is a Mux select/enable pin. Skipping."}
	   continue
	  }
	 }
	 
         # Foreach each of those pins, recursively call *ce_get_fanx* and append 
         # the result to the $fanx_col collection variable
         if {$debug} {
	  echo "(ce_get_fanx) -I- \[Recursion on X-pins\] Foreach each of those (x-$direction)pins, recursively call ce_get_fanx"
          echo "(ce_get_fanx)      with myxpin '[get_attribute -q $myxpinport full_name]' pin"
          echo "(ce_get_fanx) -I- Object Class is '[get_attribute -q $myxpinport object_class]'"
	 }

         if {[info exists opt_list(tpoints)]} {
	  lappend opt_list(curtp)	[get_attribute -q $myxpinport full_name]
	 }

         #
         # Infact *From_pin* and *To_pin* of timing_arcs may be either PIN or PORT.
         # I was previously thinking that timing_arcs from/to pin were only PINs, 
         # I was wrong. Sigh!
         #
         if {[get_attribute -q $myxpinport object_class] == "port"} {
	  if {[info exists opt_list(tpoints)]} {
	   if {[info exists opt_list(tpout)]} {
	   echo "{$opt_list(curtp)}" >> $opt_list(tpout)
  	   } else {
	    lappend opt_list(gtp)	$opt_list(curtp)
  	   }
	  }

          if {[info exists opt_list(tpoints)]} {pop opt_list(curtp)}

          set	fanx_col	[add_to_collection -unique $fanx_col $myxpinport]
         } else {
	  if {[info exists maxlevel]} {set opt_list(curlevel)  [expr {$curlevel + 1}]}
          set	fanx_col	[add_to_collection -unique $fanx_col [ce_get_fanx $myxpinport getfanxvisited $sense opt_list]]
	  if {[info exists maxlevel]} {set opt_list(curlevel)  $curlevel}
         }
	}

        if {[info exists opt_list(tpoints)]} {pop opt_list(curtp)}

        # Now we need to mark the current (y)pin as visited, for avoiding triggering
        # recursion when seeing it next time.
        # array set getfanxvisited	[list [get_attribute -q $myypin full_name] 1]
      } else {
       if {!$quiet} {echo "\[get_fan$direction\] -I- No ${direction}put pin found on cell '[get_attribute -q $cell full_name]'"}
       # continue <<< Useless 
      }

      # Moved the above command here
      if {$unique} {set getfanxvisited($myypin_name) 1}
     }
    } else {
     if {$debug && [get_attribute -q $cell is_sequential]}  {echo "(ce_get_fanx) -I- Yes, more fun to come, We have a sequential cell"}
     if {$debug && ![get_attribute -q $cell is_sequential]} {echo "(ce_get_fanx) -I- Yes, more fun to come, We have an Unknown type of cell"}
     # We can't go anywhere, it looks like we found a sequential
     # cell (y)pin. $myypin is a kind of relative start/endpoint
     # we need to log that fact

     ##if {[get_attribute -q $myypin getfanxvisited] != ""} {continue}
     if {$unique && [info exists getfanxvisited($myypin_name)]} {continue}

     if {$debug} {
      echo "(ce_get_fanx) -I- Yes, more fun to come, appending *$myypin_name* pin to fanx_col collection"
      echo "(ce_get_fanx) -I- pinappend *$myypin_name*"
     }

     if {[info exists opt_list(tpoints)]} {
      lappend opt_list(curtp)	$myypin_name

      if {[info exists opt_list(tpout)]} {
       echo "{$opt_list(curtp)}" >> $opt_list(tpout)
      } else {
       lappend opt_list(gtp)	$opt_list(curtp)
      }
  #echo "-sequential-$myypin_name lencurtp=([llength $opt_list(curtp)])"
     }

     set	fanx_col	[add_to_collection -unique $fanx_col $myypin]

     if {[info exists opt_list(tpoints)]} {pop opt_list(curtp)}

     ##set_user_attribute -q $myypin getfanxvisited	1
     if {$unique} {set getfanxvisited([get_attribute -q $myypin full_name]) 1}

     # Now go and see the next $myxpin
     # continue <<< Useless
    }
   }

   # Now let's go and play with (x)ports, if any
  } else {
   if {$debug} {echo "(ce_get_fanx) -I- No $ydir and no inout pin(s) connected to net [get_attribute -q $nets full_name]"}
  }

  # Let's try to find (x)ports
  if {$debug} {echo "(ce_get_fanx) -I- Let's try to find (x)ports"}

  set	xports	[remove_from_collection [get_ports -q -f "direction == $direction || direction == inout" -o $nets] $pinorport]

  if {$debug} {query_object $xports}

  if {$xports != ""} {
   # We do have connected (x)ports
   if {$debug} {echo "(ce_get_fanx) -I- We do have connected (x)ports"}

   foreach_in_collection myxport $xports {
    # For each of those xport, just append its to $fanx_col collection
    # variable

    ##if {[get_attribute -q $myxport getfanxvisited] != ""} {continue}
    if {$unique && [info exists getfanxvisited([get_attribute -q $myxport full_name])]} {
     if {$debug} {echo "(ce_get_fanx) -I- (x)port alreday seen '[get_attribute -q $myxport full_name]'. Skipping that iteration"}
     continue
    }

    if {$debug} {
     echo "(ce_get_fanx) -I- For each of those xport, just append its to fanx_col collection variable "
     echo "(ce_get_fanx) -I- portappend [get_attribute -q $myxport full_name]"
    }

    if {[info exists opt_list(tpoints)]} {
     lappend opt_list(curtp)	[get_attribute -q $myxport full_name]
     if {[info exists opt_list(tpout)]} {
      echo "{$opt_list(curtp)}" >> $opt_list(tpout)
     } else {
      lappend opt_list(gtp)	$opt_list(curtp)
     }
    }

    set	fanx_col	[add_to_collection -unique $fanx_col $myxport]

    if {[info exists opt_list(tpoints)]} {pop opt_list(curtp)}

    ##set_user_attribute -q $myxport getfanxvisited	1
    if {$unique} {set getfanxvisited([get_attribute -q $myxport full_name]) 1}
   }

  }

  if {[info exists opt_list(tpoints)]} {pop opt_list(curtp)}

  # Now that we did all our job for the current
  # ce_get_fanx instance we can return the value
  # of $fanx_col variable to the caller
  return $fanx_col
 } else {
  # Recursion stops here.
  if {$debug} {echo "(ce_get_fanx) -W- No nets connected on '$pinorport_name'"}

  if {[info exists opt_list(tpoints)]} {pop opt_list(curtp)}

  return $fanx_col
 }
}
