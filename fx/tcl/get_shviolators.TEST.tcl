#----------------------------------------------------------------------------------------------
# Author	: Richard DJE
#
# Description	: get_shviolators
#
#----------------------------------------------------------------------------------------------
# Property of Texas Instruments - For  Unrestricted Internal Use  Only.
# Unauthorized reproduction and/or distribution is strictly prohibited.  This
# product is protected under copyright law and trade secret law as an unpublished
# work. (C) Copyright 2005 Texas Instruments. All rights reserved.
#----------------------------------------------------------------------------------------------
package provide	stapack 1.0

proc	get_shviolators	{args} {
 getoptions gs_arg $args

 array set gs_arg_so $gs_arg(SO)
 array set gs_arg_so $gs_arg(OWV)

 if {[info exists gs_arg_so(help)]} {
  echo "usage: get_shviolators \[--nviolators=<ViolatorsCount>\] \[--nworst=<NworstValue>\] \[-out=<Outputfile>\] \[--slack=<SlackValue>\]"
  echo "                       \[--skip-group=<GroupToSkip>\]* \[--noslack>\] \[--auto\] \[--show\] \[--quiet\] \[--debug\] \[--help\]"
  return {}
 }

 # Default values
 set slack	0
 set nworst	1
 set nviolators	10000
 set output	"shviolators.lof"

 if {[info exists gs_arg_so(slack)]}       {set slack      $gs_arg_so(slack)}
 if {[info exists gs_arg_so(nworst)]}      {set nworst     $gs_arg_so(nworst)}
 if {[info exists gs_arg_so(out)]}         {set output     $gs_arg_so(out)}
 if {[info exists gs_arg_so(nviolators)]}  {set nviolators $gs_arg_so(nviolators)}

 set auto [info exists gs_arg_so(auto)]

 # Some group sometimes cause PT to crash when asked to retrieve
 # all violating paths relative to that group, by indicating a big
 # value as --max_paths, that is >= 1000
 set nothem {}
 if {[info exists gs_arg_so(skip-group)]} {
  set toignore {}
  foreach skipgrp $gs_arg_so(skip-group) {
   # Here push IS MANDATORY
   push toignore [split $skipgrp ,]
  }

  foreach mygroup $toignore {
   set nothem [add_to_collection -unique $nothem [get_path_group -q $mygroup]]
  }
 }

 set quiet	[info exists gs_arg_so(quiet)]
 set show	[info exists gs_arg_so(show)]
 set noslack	[info exists gs_arg_so(noslack)]


 set setup_slacksum	0
 set setup_wns		0
 set setup_count	0
 set setup_uncount	0
 set setup_i_uncount	0
 set setup_o_uncount	0
 set setup_int_uncount	0
 set setup_c_uncount	0
 set setup_v_i_count	0
 set setup_v_o_count	0
 set setup_v_int_count	0
 set setup_v_c_count	0
 set setup_m_i_count	0
 set setup_m_o_count	0
 set setup_m_int_count	0
 set setup_m_c_count	0

 set hold_slacksum	0
 set hold_wns		0
 set hold_count		0
 set hold_uncount	0
 set hold_i_uncount	0
 set hold_o_uncount	0
 set hold_int_uncount	0
 set hold_c_uncount	0
 set hold_v_i_count	0
 set hold_v_o_count	0
 set hold_v_int_count	0
 set hold_v_c_count	0
 set hold_m_i_count	0
 set hold_m_o_count	0
 set hold_m_int_count	0
 set hold_m_c_count	0

 array set	setup	{}
 array set	hold	{}

 foreach_in_collection mygroup [remove_from_collection [get_path_groups *] $nothem] {
  set grpname	[get_attribute -q $mygroup full_name]

  if {!$quiet} {echo "(get_shviolators) -I- Processing group '$grpname'.."}

  if {$auto && ![info exists gs_arg_so(nviolators)]} {
   #Now I should retrieve the pin(s)/port(s) to which it is applied
   set	clksrcs	[get_attribute  -c clock $grpname sources]
   set	colsize		[sizeof_collection $clksrcs]
   if {$colsize > 1} {
    echo "(get_shviolators) -W- Clock '$grpname' is associated with several pins/ports."
   } elseif {$colsize == 0} {
    echo "(get_shviolators) -W- Clock '$grpname' has no (generated) source(s) (virtual-clock ?)."
   }

   set maxep	0
   foreach_in_collection mysrc $clksrcs {
    set sz [sizeof_collection [get_for -quiet -latch $mysrc]]
    if {$sz > $maxep} {set maxep $sz}
   }

   set nviolators [expr {$maxep ? $maxep : 1}]
   if {!$quiet} {echo "(get_shviolators) -I- *nviolators* set to *$nviolators*"}
  }
  
  if {$noslack} {
   array set setup	[get_tpinfo [get_timing_paths -group $grpname -max_paths $nviolators -delay_type "max"  -nworst $nworst]]
   array set hold	[get_tpinfo [get_timing_paths -group $grpname -max_paths $nviolators -delay_type "min"  -nworst $nworst]]
  } else {
   array set setup	[get_tpinfo [get_timing_paths -group $grpname -max_paths $nviolators -delay_type "max"  -slack_lesser_than $slack -nworst $nworst]]
   array set hold	[get_tpinfo [get_timing_paths -group $grpname -max_paths $nviolators -delay_type "min"  -slack_lesser_than $slack -nworst $nworst]]
  }


  set setup_input($grpname)		$setup(i_violated)
  set setup_output($grpname)		$setup(o_violated)
  set setup_internal($grpname)		$setup(int_violated)
  set setup_combinational($grpname)	$setup(c_violated)
  
  set setup_i_noconstraint($grpname)	$setup(i_noconstraint)
  set setup_o_noconstraint($grpname)	$setup(o_noconstraint)
  set setup_int_noconstraint($grpname)	$setup(int_noconstraint)
  set setup_c_noconstraint($grpname)	$setup(c_noconstraint)

  set setup_i_meet($grpname)		$setup(i_meet)
  set setup_o_meet($grpname)		$setup(o_meet)
  set setup_int_meet($grpname)		$setup(int_meet)
  set setup_c_meet($grpname)		$setup(c_meet)

  set setup_nopath($grpname)		$setup(nopath)
 
  set hold_input($grpname)		$hold(i_violated)
  set hold_output($grpname)		$hold(o_violated)
  set hold_internal($grpname)		$hold(int_violated)
  set hold_combinational($grpname)	$hold(c_violated)

  set hold_i_noconstraint($grpname)	$hold(i_noconstraint)
  set hold_o_noconstraint($grpname)	$hold(o_noconstraint)
  set hold_int_noconstraint($grpname)	$hold(int_noconstraint)
  set hold_c_noconstraint($grpname)	$hold(c_noconstraint)

  set hold_i_meet($grpname)		$hold(i_meet)
  set hold_o_meet($grpname)		$hold(o_meet)
  set hold_int_meet($grpname)		$hold(int_meet)
  set hold_c_meet($grpname)		$hold(c_meet)

  set hold_nopath($grpname)		$hold(nopath)

  incr setup_uncount			$setup(uncount)
  incr setup_i_uncount			$setup(i_uncount)
  incr setup_o_uncount			$setup(o_uncount)
  incr setup_int_uncount		$setup(int_uncount)
  incr setup_c_uncount			$setup(c_uncount)

  incr setup_v_i_count                  $setup(i_violated_count)
  incr setup_v_o_count                  $setup(o_violated_count)
  incr setup_v_int_count                $setup(int_violated_count)
  incr setup_v_c_count                  $setup(c_violated_count)
 
  incr setup_m_i_count                  $setup(i_meet_count)
  incr setup_m_o_count                  $setup(o_meet_count)
  incr setup_m_int_count                $setup(int_meet_count)
  incr setup_m_c_count                  $setup(c_meet_count)
 
  incr hold_uncount			$hold(uncount)
  incr hold_i_uncount			$hold(i_uncount)
  incr hold_o_uncount			$hold(o_uncount)
  incr hold_int_uncount			$hold(int_uncount)
  incr hold_c_uncount			$hold(c_uncount)

  incr hold_v_i_count                   $hold(i_violated_count)
  incr hold_v_o_count                   $hold(o_violated_count)
  incr hold_v_int_count                 $hold(int_violated_count)
  incr hold_v_c_count                   $hold(c_violated_count)

  incr hold_m_i_count                   $hold(i_meet_count)
  incr hold_m_o_count                   $hold(o_meet_count)
  incr hold_m_int_count                 $hold(int_meet_count)
  incr hold_m_c_count                   $hold(c_meet_count)

  set setup_slacksum			[expr {$setup_slacksum	+ $setup(slacksum)}]
  set hold_slacksum			[expr {$hold_slacksum	+ $hold(slacksum)}]
  incr setup_count			$setup(count)
  incr hold_count			$hold(count)
  if {$setup(wns) < $setup_wns} 	{set setup_wns $setup(wns)}
  if {$hold(wns)  < $hold_wns}  	{set hold_wns  $hold(wns)}

  array set setup {}
  array set hold  {}
 }


 if {[file exists $output]} {file delete $output}

 set outputd {}
 array set alreadyseen	{}
 foreach maxormin {setup hold} {
  foreach type {input output internal combinational i_noconstraint o_noconstraint int_noconstraint c_noconstraint i_meet o_meet int_meet c_meet nopath} {
   if {!$quiet} {echo "(get_shviolators) -I- writing '$type' paths for '$maxormin'.."} 
   set arrayname	${maxormin}_${type}
   lappend outputd "=$arrayname="
   foreach grp [array names $arrayname] {
    set data	[set [subst {$arrayname}]($grp)]
    if {[llength $data] == 0} {echo "(get_shviolators) -W- No violating '$type' paths for '$maxormin' for '$grp'."}

    foreach ent  $data {
      set datconcat  [join $ent ,]
      if {![info exists alreadyseen($datconcat)]} {
       lappend outputd $datconcat
       set alreadyseen($datconcat)  1
      } else {
       if {$show} {echo "-SHOW- ($datconcat)($grp)"}
       continue
      }
    }
   }

   lappend outputd "=${arrayname}_end="
  }
 }
 
 foreach maxormin {setup hold} {
  if {!$quiet} {echo "(get_shviolators) -I- writing summary for '$maxormin'.."} 
  lappend outputd "=${maxormin}_summary="
  set data	{}
  foreach type {wns count slacksum v_i_count v_o_count v_int_count v_c_count uncount i_uncount o_uncount int_uncount c_uncount m_i_count m_o_count m_int_count m_c_count} {
   lappend data [set ${maxormin}_${type}]
  }

  lappend outputd [join $data ,]
  lappend outputd "=${maxormin}_summary_end="
 }

 echo [join $outputd \n] > $output
}
