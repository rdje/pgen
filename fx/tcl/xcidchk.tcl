# e-X-tracted -C-lock -I-nsertion -D-elay -CH-ec-K-er
proc	xcidchk	{timing_info {verbose 0}}	{
 
  # I need to create a hash table containing the name of the
  # clock object together with the full_name of the pin/port to
  # which the clock has been applied
  array set clockinfo	{}
  foreach_in_collection myclkobj [get_clocks *] {
    set clkname	[get_attribute -q $myclkobj full_name]

    # Now I should retrieve the pin(s)/port(s) to which it is applied
    set	pinportcol	[get_attribute -q $myclkobj sources]
    if {[sizeof_collection $pinportcol] > 1} {
     puts "\[xcidchk\] -W- Clock $clkname associated with several pin/port."
    }

    array set clockinfo [list  $clkname $pinportcol]
  }
  
  # Checking clock insertion delays for input port to register paths
  set	maxdiff	 0
  set   failed_cnt 0
  set	failed_paths {}
  set ipath_info [lindex $timing_info 0]
  # Append the list of timing_path info from input port to async pins
  if {[llength [lindex $timing_info 3]] == 0} {
    puts "timing_info 3 is EMPTY"
    return
  }

  foreach myel [lindex $timing_info 3] {puts "########$myel#####" ;lappend $ipath_info $myel}

  foreach myinfo $ipath_info {
    # Endpoint point
    set  ep	[lindex $myinfo 1]
    # Endpoint clock
    set  epck	[lindex $myinfo 3]
    # Endpoint clock insertion delay
    set  epckid	[lindex $myinfo 6]

    # Retrieve the cell (register) object
    set epcell	[get_cells -q -o $ep]
    # Retrieve the clock signal
    set epflop_clk [get_pins -q -f "direction == in && is_clock_pin == true" -o $epcell]
    
    # Now let's ask PT to get the timing_path that has the clock source and the above clock-pin
    # as timing_points. We should do TWO get_timing_paths between the clock source and the final
    # clocks pin. One with *-rise_from* and the others with *-fall_from*. Because of them has been
    # used to get the Worst Slack (Fastest path on the Capture clock Path), but we don't which one.
    #
    # So by retrieving two timing_paths with different launching type of edge, we are sure to cover
    # all cases.

    if {$epck == "Undefined"} {
     puts "\[xcidchk\] -W- Skipping '$myinfo'"

     continue
    }

    set	chktip_r	[get_timing_paths -rise_from $clockinfo($epck) -to $epflop_clk]
    set	chktip_f	[get_timing_paths -fall_from $clockinfo($epck) -to $epflop_clk]

    puts "\[xcidchk\] -I- Checking insertion delay for $ep->$epck pair (<= 1ps)"
    set	diff_r	[expr {$epckid - [get_attribute -q $chktip_r arrival]}]
    if {$diff_r > $maxdiff} {set maxdiff $diff_r}

    if {$diff_r <= 0.001} {
     set res_r	""
    } else {
     set res_r  "--> RISE([expr {abs($diff_r)*1000}]ps)"

     incr failed_cnt
     lappend failed_paths	"$myinfo:$res_r" 
    }

    if {$verbose} {puts "\[xcidchk\] -I- rise from clock source: $res_r"}

    set	diff_f	[expr {$epckid - [get_attribute -q $chktip_f arrival]}]
    if {$diff_f > $maxdiff} {set maxdiff $diff_f}
    if {$diff_f <= 0.001} {
     set res_f	""
    } else {
     set res_f  "--> FALL([expr {abs($diff_f)*1000}]ps)"

     incr failed_cnt
     lappend failed_paths	"$myinfo:$res_r" 
    }

    if {$verbose} {puts "\[xcidchk\] -I- fall from clock source: $res_f"}
  }
}
