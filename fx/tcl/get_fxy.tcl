#----------------------------------------------------------------------------------------------
# Author	: Richard DJE
#
# Description	: get_for get_fir
#----------------------------------------------------------------------------------------------
package provide	stapack 1.0


proc get_for	{args} {
 getoptions gf_args_s $args

 array set gf_args_so $gf_args_s(SO)
 array set gf_args_owv $gf_args_s(OWV)

 if {[info exists gf_args_so(help)]} {
  echo  "usage: get_for \[options\] <pinport_object_spec>"
  echo  "     options = \[--nomuxsel\] \[--latch\] \[--unknown\] \[--thrulatch\] \[--tpoints=<ListName>\]"
  echo  "               \[--notiarcs\] \[-showmux\] \[--unique=ArrayName\] \[--quiet\] \[--help\] \[--debug\]"
  return {}
 }

 set gf_obj [lindex $gf_args_s(SA) 0]


 # Options
 set debug	[info exists gf_args_so(debug)]
 set quiet	[info exists gf_args_so(quiet)]
 set latch	[info exists gf_args_so(latch)]
 set unknown	[info exists gf_args_so(unknown)]
 set thrulatch	[info exists gf_args_so(thrulatch)]
 
 if {0} {echo "(get_for) -I- Entering w/ =$args="}

 set so_opts	""
 foreach myopt [array names gf_args_so] {
  append so_opts " --$myopt"
 }

 if {[info exists gf_args_owv(tpoints)]} {
  upvar $gf_args_owv(tpoints)	uplist
  set	gf_args_owv(tpoints)	uplist
 }

 foreach myopt [array names gf_args_owv] {
  if {$myopt != "unique"} {
   append so_opts " --$myopt=$gf_args_owv($myopt)"
  }
 }

 set opiniport [filter_collection [object -pin -port $gf_obj] "direction == inout || 								      \
 							       (object_class == pin && direction == out) || (object_class == port && direction == in)"]
 set	reglist	{}
 if {$opiniport != ""} {
   if {![info exists gf_args_owv(unique)]} {
   foreach_in_collection myobj $opiniport {
    array set uniqueness [list [get_attribute $myobj full_name]	1]
   }

   } else {
    upvar $gf_args_owv(unique) uniqueness
    if {[info exists uniqueness([get_attribute $opiniport full_name])]} {
     echo  "(get_for) -W- LOOP on *[get_attribute $opiniport full_name]*"
     return {}
    } else {
     array set uniqueness [list [get_attribute $opiniport full_name]	1]
    }
   }

   set get_fanout_cmd	"get_fanout $opiniport $so_opts"
   set	retobj	{}
   foreach_in_collection myobj $opiniport {
    if {!$quiet} {echo "(get_for) -I- Calling get_fanout on '[get_attribute -q $myobj full_name]'."}
    set retobj [add_to_collection -u $retobj [eval $get_fanout_cmd]]
   }

   # Retrieving ports, if any.
   if {0} {echo "(get_for) -I- Retrieving PORTs"}
   set ports	[filter_collection $retobj "object_class == port"]
   set reglist 	[add_to_collection -u $reglist $ports]

   # Retrieving registers' pins, if any.
   if {0} {echo "(get_for) -I- Retrieving REGs"}
   set regpins	[get_gsx0_xyz -type=reg $retobj]
   set reglist 	[add_to_collection -u $reglist $regpins]


   if {0} {echo "(get_for) -I- Retrieving LATs"}
   set latpins	[get_gsx0_xyz -type=lat $retobj]
   if {$latch && !$thrulatch} {set reglist	[add_to_collection -u $reglist $latpins]}

   if {0} {
    # Retrieving latches' pins, if any.
    foreach_in_collection mylatpin $latpins {
     echo "(get_for) -W- Latch pin '[get_attribute -q $mylatpin full_name]' on the path."
    }
   }

   # Recursively handling CGC and UC43X, if any.
   if {0} {echo "(get_for) -I- Retrieving CGUC43Xs"}
   set thrupins	 [get_gsx0_xyz -type=thru $retobj]
   if {$thrulatch} {set thrupins [add_to_collection -u $thrupins $latpins]}

   foreach_in_collection mythru  $thrupins {
    # Retrieving all otherside pin(s) of this cell
    set gothru_cmd	"go_thru -out $mythru $so_opts"
    set otherside_pins [eval $gothru_cmd]

    if {$otherside_pins != ""} {
     if {!$quiet} {echo "(get_for) -I- Propagating through pin '[get_attribute -q $mythru full_name]'."}
     foreach_in_collection myotherside $otherside_pins {
      if {!$quiet} {echo "(get_for) -I- Recursing through other-side pin '[get_attribute -q $mythru full_name]'."}
      set get_for_cmd	"get_for $myotherside $so_opts --unique=uniqueness"
      set reglist [add_to_collection -u $reglist [eval $get_for_cmd]]
     }
    } else {
     if {!$quiet} {echo "(get_for) -I- Can't Propagate through pin '[get_attribute -q $mythru full_name]'."}
    }
   }

   # Unknown pins elements, if any
   set tobe_removed	$ports
   set tobe_removed   	[add_to_collection $tobe_removed $regpins]
   set tobe_removed    	[add_to_collection $tobe_removed $latpins]
   set tobe_removed    	[add_to_collection $tobe_removed $thrupins]
   
   set unknown_cells	[remove_from_collection $retobj $tobe_removed]
   if {$unknown} 	{set reglist	[add_to_collection -u $reglist $unknown_cells]}

   if {!$quiet && !$unknown} {
    foreach_in_collection myunknown $unknown_cells {
     echo "(get_for) -WW- **Unidentified** sequential cell '[get_attribute -q [get_cells -q -o $myunknown] full_name]' on the path."
     echo "(get_for) -WW- Pin name     = '[get_attribute -q $myunknown  full_name]'"
     echo "(get_for) -WW- Cell refname = '[get_attribute -q [get_cells -q -o $myunknown] ref_name]'"
    }
   }


   if {0} {echo "(get_for) -I- Leaving =$args="}
   # Job Done.
   return $reglist
 } else {
  if {!$quiet} {echo "(get_for) -E- Only output pin and in/inout port are supported (full_name/collection)!"}
  return {}
 }
}

proc get_fir	{args} {
 getoptions gf_args_s $args


 array set gf_args_so  $gf_args_s(SO)
 array set gf_args_owv $gf_args_s(OWV)

 if {[info exists gf_args_so(help)]} {
  echo  "usage: get_fir \[--quiet\] \[--notiarcs\] \[--latch\] \[--unknown\] \[--thrulatch\] \[--nomuxsel\] \[-showmux\] \[--tpoints=<ListName>\]  \[--unique=ArrayName\] \[--help\] \[--debug\] <pinport_object_spec>"
  return {}
 }

 set gf_obj [lindex $gf_args_s(SA) 0]

 # Options
 set debug	[info exists gf_args_so(debug)]
 set quiet	[info exists gf_args_so(quiet)]
 set latch	[info exists gf_args_so(latch)]
 set unknown	[info exists gf_args_so(unknown)]
 set thrulatch	[info exists gf_args_so(thrulatch)]
 
 set so_opts	"--nomuxsel"
 foreach myopt [array names gf_args_so] {
  append so_opts " --$myopt"
 }

 if {[info exists gf_args_owv(tpoints)]} {
  upvar $gf_args_owv(tpoints)	uplist
  set	gf_args_owv(tpoints)	uplist
 }

 foreach myopt [array names gf_args_owv] {
  if {$myopt != "unique"} {
   append so_opts " --$myopt=$gf_args_owv($myopt)"
  }
 }

 set ispinport [filter_collection [object -pin -port $gf_obj] "direction == inout || 								     \
 							      (object_class == pin && direction == in) || (object_class == port && direction == out)"]

 if {$ispinport != ""} {
  if {![info exists gf_args_owv(unique)]} {
   foreach_in_collection myobj $ispinport {
    array set uniqueness [list [get_attribute $myobj full_name]	1]
   }
  } else {
   upvar $gf_args_owv(unique) uniqueness
   if {[info exists uniqueness([get_attribute $ispinport full_name])]} {
    echo  "(get_fir) -W- LOOP on *[get_attribute $ispinport full_name]*"
    return {}
   } else {
    array set uniqueness [list [get_attribute $ispinport full_name]	1]
   }
  }

  set get_fanin_cmd "get_fanin $ispinport $so_opts"
  set fanin_objs	{} 
  foreach_in_collection myobj $ispinport {
   if {!$quiet} {echo "(get_fir) -I- Calling get_fanin on '[get_attribute -q $myobj full_name]'."}
   set fanin_objs [add_to_collection -u $fanin_objs [eval $get_fanin_cmd]]
  }
  
  set fanin_objs_list	{}
  
  # Retrieving ports, if any.
  set ports	        [filter_collection $fanin_objs "object_class == port"]
  set fanin_objs_list 	[add_to_collection -u $fanin_objs_list $ports]

  # Retrieving registers' pins, if any.
  set regpins	        [get_gsx0_xyz -type=reg $fanin_objs]
  set fanin_objs_list 	[add_to_collection -u $fanin_objs_list $regpins]

  # Retrieving latch's pins, if any.
  set latpins	               [get_gsx0_xyz -type=lat $fanin_objs]
  if {$latch && !$thrulatch}   {set fanin_objs_list 	[add_to_collection -u $fanin_objs_list $latpins]}

  # Retrieving thru's pins, if any.
  set thrupins	        [get_gsx0_xyz -type=thru $fanin_objs]
  if {$thrulatch}       {set thrupins [add_to_collection -u $thrupins $latpins]}

  foreach_in_collection mythru  $thrupins {
   # Retrieving all otherside pin(s) of this cell
   set gothru_cmd	"go_thru -in $mythru $so_opts"
   set otherside_pins [eval $gothru_cmd]

   if {$otherside_pins != ""} {
     if {!$quiet} {echo "(get_fir) -I- Propagating through pin '[get_attribute -q $mythru full_name]'."}
     foreach_in_collection myotherside $otherside_pins {
      if {[is_gsx0_mux_select_pin $myotherside]} {
       echo "(get_fir) -I- Skipping mux select/enable pin '[get_attribute -q $myotherside full_name]'."
       continue
      }

      echo "(get_fir) -I- Recursing through other-side pin '[get_attribute $myotherside full_name]'."
      set get_fir_cmd	"get_fir $myotherside $so_opts --unique=uniqueness"
      set fanin_objs_list [add_to_collection -u $fanin_objs_list [eval $get_fir_cmd]]

     }
   } else {
    if {!$quiet} {echo "(get_fir) -I- Can't Propagate through pin '[get_attribute -q $mythru full_name]'."}
   }
  }

  # Unknown pins elements, if any
  set tobe_removed   	[add_to_collection $ports           $regpins]
  set tobe_removed    	[add_to_collection $tobe_removed    $latpins]
  set tobe_removed    	[add_to_collection $tobe_removed    $thrupins]
  
  set unknown_pins	[remove_from_collection $fanin_objs $tobe_removed]
  if $unknown 	        {set fanin_objs_list	[add_to_collection -u $fanin_objs_list $unknown_pins]}

  if {!$quiet && !$unknown} {
   foreach_in_collection myunknown $unknown_pins {
    echo "(get_fir) -WW- **Unidentified** sequential cell '[get_attribute -q [get_cells -q -o $myunknown] full_name]' on the path."
    echo "(get_fir) -WW- Pin name     = '[get_attribute -q $myunknown  full_name]'"
    echo "(get_fir) -WW- Cell refname = '[get_attribute -q [get_cells -q -o $myunknown] ref_name]'"
   }
  }

  return $fanin_objs_list
 } else {
  echo "(get_fir) -E- Only input pin and out/inout port objects are supported."
 }
}






proc get_foilr  {args} {
 getoptions gf_args_s $args


 # building *get_for* command line
 array set gf_args_so $gf_args_s(SO)

 if {[info exists gf_args_so(help)]} {
  echo  "usage: get_foilr \[--quiet\] \[--notiarcs\] \[--nomuxsel\] \[-showmux\] \[--help\] \[--debug\] <pinport_object_spec>"
  return {}
 }

 set gf_obj [lindex $gf_args_s(SA) 0]

 # Options
 set debug	[info exists gf_args_so(debug)]
 
 set get_for_cmd	"get_for $gf_obj"
 foreach myopt [array names gf_args_so] {
  append get_for_cmd " --$myopt"
 }

 # Executing the get_for command line via the call
 set  retv	[eval $get_for_cmd]

 if {$retv == ""} {
  if {![info exists gf_args_so(quiet)]} {echo "\[get_foilr\] -W- No fanout-node(s) found !"}
  return {}
 }

 # Building *get_fanout/get_fanin* option list
 set get_fanx_opts	""
 foreach myopt [array names gf_args_so] {
  append get_fanx_opts " --$myopt"
 }

 # Retrieving ports
 set	endpoints	[filter_collection $retv "object_class == port"]

 # Retrieving registers clock pin only, because *get_for* may have stopped
 # on registers data pins.
 set	onlypins	[filter_collection $retv "object_class == pin"]

 # We will be considering Only Clock-pins
 # I could have used the above filter command also to retrieve pins that are
 # also clock_pin, but since the $retv is a mix collection, that is, may
 # contain pins and ports, using the *is_clock_pin* property on port
 # generates an Error.
 set clkpins		[filter_collection $onlypins "is_clock_pin == true"]

 foreach_in_collection myclkpin  $clkpins {
  if {$debug} {echo "\n\[get_foilr\] -I- Processing clk-pin '[get_attribute -q $myclkpin full_name]'."}
  # Foreach of these clock pins,  Retrieve the associated cell and cell data-pin(s)
  # Normally there should be just one data-pin
  set	mycell		[get_cells -q -o $myclkpin]
  set	mycellname	[get_attribute -q $mycell full_name]
  set	datapin_s	[get_pins -q -fi "direction == in && is_data_pin == true" -o $mycell]
  set	datapin_ssize	[sizeof_collection $datapin_s]
  if {$datapin_ssize > 1} {
   if {$debug} {echo "\[get_foilr\] -W- Register '$mycellname' has more than one ($datapin_ssize) data pins."}
  } elseif {$datapin_ssize == 0} {
   if {$debug} {echo "\[get_foilr\] -W- Register '$mycellname' has No data pin."}
  }

  # Also retrieve the cell output pins
  set	outputpins	[get_pins -q -fi "direction == out" -o $mycell]

  set	iports	{}
  foreach_in_collection mydatapin $datapin_s {
   # Retrieve the fanin Nodes of that data-pin(s)
   if {$debug} {echo "\[get_foilr\] -I- Processing data-pin '[get_attribute -q $mydatapin full_name]'."}
   set get_fanin_cmd	"get_fanin $mydatapin $get_fanx_opts"
   set	iports	[add_to_collection -u $iports [filter_collection [eval $get_fanin_cmd] "object_class == port"]]
  }

  # Also get the fanout Nodes of the output pin(s)
  set	oports	{}
  foreach_in_collection myoutpin $outputpins {
    if {$debug} {echo "\[get_foilr\] -I- Processing output-pin '[get_attribute -q $myoutpin full_name]'."}
    set get_fanout_cmd	"get_fanout $myoutpin $get_fanx_opts"
    set oports	[add_to_collection -u $oports [filter_collection [eval $get_fanout_cmd] "object_class == port"]]
  }

  if {$debug} {
   if {$iports != ""} {
    echo "\[get_foilr\] -W- Register '$mycellname' data pin(s) have port(s) fanin-node(s)."
   } else {
    echo "\[get_foilr\] -W- Register '$mycellname' data pin(s) '[get_attribute -q $datapin_s full_name]' have No port fanin-node."
   }

   if {$oports != ""} {
    echo "\[get_foilr\] -W- Register '$mycellname' output-pin(s) have port(s) fanout-node(s)."
   } else {
    echo "\[get_foilr\] -W- Register '$mycellname' output-pin(s) have No port fanout-node."
   }
  }

  # In either of $iports and $oports is non-empty than this register is for sure an interface logic register
  # controlled by the clock source identified by $gf_obj
  if {$iports != "" || $oports != ""} {
    if {$debug} {echo "\[get_foilr\] -W- Register '$mycellname' is an Interface Logic Register."}
    set  endpoints	[add_to_collection -u $endpoints $myclkpin]
  } else {
    if {$debug} {echo "\[get_foilr\] -W- Register '$mycellname' is NOT an Interface Logic Register."}
  }
 }

 # Job Done.
 return $endpoints
}
