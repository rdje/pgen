#----------------------------------------------------------------------------------------------
# Author	: Richard DJE
#
# Description	: Routines for retrieving gsx0 registers, clock-gating cells, latches,
#		  antenna protection diodes and (de)muxes.
#
#----------------------------------------------------------------------------------------------
# Property of Texas Instruments - For  Unrestricted Internal Use  Only.
# Unauthorized reproduction and/or distribution is strictly prohibited.  This
# product is protected under copyright law and trade secret law as an unpublished
# work. (C) Copyright 2005 Texas Instruments. All rights reserved.
#----------------------------------------------------------------------------------------------
package provide	stapack 1.0

proc	get_gsx0_xyz	{args} {
 getoptions ggx_args $args

 array set optlist $ggx_args(SO)
 array set optlist $ggx_args(OWV)

 if {[info exists optlist(help)]} {
  echo "usage: get_gsx0_xyz  \[options\] <object_spec>"
  usage_get_gsx0_xyz
 }

 if {![info exists ggx_args(SA)]} {
  echo "(get_gsx0_xyz) -E- The <object_spec> is required"
  usage_get_gsx0_xyz
 }


 if {![info exists optlist(type)]} {
  echo "(get_gsx0_xyz) -E- The *--type* option is required."
  usage_get_gsx0_xyz
 }

 set type	$optlist(type)

 # You may think it's silly to do things that way.
 # The issue being that Filter expressions DO NOT support the Full
 # set of REGEXP regular expression :-(
 set reg_re	 	 "ref_name =~ DNB.*      || ref_name =~ DNN.* || ref_name =~ DTB.* || ref_name =~ DTC.* || ref_name =~ DTN.* || \
 			  ref_name =~ DTP.*      || ref_name =~ TDB.* || ref_name =~ TDC.* || ref_name =~ TDN.* || ref_name =~ TDP.* || \
		  	  ref_name =~ TMC.*      || ref_name =~ TMN.* || ref_name =~ TMP.* || ref_name =~ TNB.* || ref_name =~ TNN.* || \
		  	  ref_name =~ CTGFF4Q.*  || ref_name =~ SDFF.*"


 set scanreg_re	 	 "ref_name =~ TDB.* || ref_name =~ TDC.* || ref_name =~ TDN.* || ref_name =~ TDP.* || \
		  	  ref_name =~ TMC.* || ref_name =~ TMN.* || ref_name =~ TMP.* || ref_name =~ TNB.* || ref_name =~ TNN.*"

 set noscanreg_re	 "ref_name =~ DNB.* || ref_name =~ DNN.* || ref_name =~ DTB.* || ref_name =~ DTC.* || ref_name =~ DTN.* || \
 			  ref_name =~ DTP.* || ref_name =~ CTGFF4Q.*"

 #set filter_exp(reg) 	 [subst {$reg_re}]
 set filter_exp(reg) 	 	$reg_re

 set filter_exp(io) 	 	{ref_name =~ [BIO]Q.*}
 set filter_exp(lat) 	 	{ref_name =~ LAH.* || ref_name =~ LAL.*}
 set filter_exp(cgc) 	 	{ref_name =~ ICG.*}
 set filter_exp(mux) 	 	{ref_name =~ DMU.* || ref_name =~ MU.* || ref_name =~ CTGMU.* || ref_name =~ UC43[12].*}
 set filter_exp(uc43x) 	 	{ref_name =~ UC43[12].*}
 set filter_exp(port) 	 	{object_class == port}
 set filter_exp(apd) 	 	{ref_name =~ AP0.*}
 set filter_exp(scanreg) 	$scanreg_re
 set filter_exp(noscanreg) 	$noscanreg_re
 set filter_exp(regport) 	{$reg_re || object_class == port}
 set filter_exp(cguc43x) 	{ref_name =~ ICG.* || ref_name =~ UC43[12].*}
 set filter_exp(thru)     	"$filter_exp(cguc43x) || $filter_exp(io)"

 if {![info exists filter_exp($type)]} {
  echo "(get_gsx0_xyz) -E- Unknown type '$type'."
  usage_get_gsx0_xyz
 }

 set pinport_col  [lindex $ggx_args(SA) 0]

 set superset	  [get_pins -q -o [get_cells -q -o $pinport_col -re -nocase -fi $filter_exp($type)]]
 set complement	  [remove_from_collection $superset $pinport_col]

 return [remove_from_collection $superset $complement]
}

proc usage_get_gsx0_xyz {} {
 echo "usage: get_gsx0_xyz  --type=<mytype> <object_spec>"
 echo "       mytype = reg     | lat     | thru | cgc | mux  | uc43x | port | io | apd"
 echo "                regport | scanreg | noscanreg  | cguc43x"
 return {}
}
