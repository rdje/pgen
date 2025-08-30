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

proc report_tia {cell_spec} {
 # Is it a cell full_name ?
 set cell [get_cells -q $cell_spec]
 if {$cell == ""} {
  # No
  # Is it a cell collection object ?
  set cell [get_attribute -q $cell_spec object_class]
  if {$cell != "cell"} {
   echo "\[is_gsx0_reg\] -E- Only cell full_name/collection are supported !"
   return 
  }

  set cell $cell_spec
 }

 foreach_in_collection myarc [get_timing_arcs -o $cell] {
  set from_pin 	 	[get_attribute -q $myarc from_pin]
  set to_pin 	 	[get_attribute -q $myarc to_pin]
  set is_disabled 	[get_attribute -q $myarc is_disabled]
  set dMf	 	[get_attribute -q $myarc delay_max_fall]
  set dMr	 	[get_attribute -q $myarc delay_max_rise]
  set dminf	 	[get_attribute -q $myarc delay_min_fall]
  set dminr	 	[get_attribute -q $myarc delay_min_rise]
 
  set is_cellarc  	[get_attribute -q $myarc is_cellarc]
  set is_user_dis 	[get_attribute -q $myarc is_user_disabled]
  set sense	 	[get_attribute -q $myarc sense]
  set when	 	[get_attribute -q $myarc when]
  set mode	 	[get_attribute -q $myarc mode]
 
  echo "[get_attribute -q $from_pin full_name] --> [get_attribute -q $to_pin full_name] : disabled=$is_disabled"
  echo "is_cellarc=\[$is_cellarc\] is_user_dis=\[$is_user_dis\] sense=\[$sense\] when=\[$when\] mode=\[$mode\]"
  echo "dMf=$dMf dMr=$dMr dmf=$dminf dmr=$dminr\n"
 }
}
