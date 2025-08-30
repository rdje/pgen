#----------------------------------------------------------------------------------------------
# Author	: Richard DJE
#
# Description	: Routines for identifying gsx0 registers, clock-gating cells, latches,
#		  antenna protection diodes and (de)muxes.
#
#----------------------------------------------------------------------------------------------
# Property of Texas Instruments - For  Unrestricted Internal Use  Only.
# Unauthorized reproduction and/or distribution is strictly prohibited.  This
# product is protected under copyright law and trade secret law as an unpublished
# work. (C) Copyright 2005 Texas Instruments. All rights reserved.
#----------------------------------------------------------------------------------------------
package provide	stapack 1.0

# gsx0 Registers
proc	is_gsx0_reg	{cell_spec} {
 set cell [object -cell $cell_spec]
 if {$cell == ""} {
  echo "(is_gsx0_mux) -E- Only cell full_names/collections are supported !"
  return 0 
 }

 # Let's get the cell ref_name
 set ref_name [get_attribute -q $cell ref_name]

 # Does it match any gsx0 register name ?
 return [regexp {(?i)^(?:DNB|DNN|DTB|DTC|DTN|DTP|TDB|TDC|TDN|TDP|TMC|TMN|TMP|TNB|TNN|SDFF).*|CTGFF4Q} $ref_name]
}


# gsx0 Clock-Gating Cells
proc	is_gsx0_cgc	{cell_spec} {
 set cell [object -cell $cell_spec]
 if {$cell == ""} {
  echo "(is_gsx0_mux) -E- Only cell full_names/collections are supported !"
  return 0 
 }

 # Let's get the cell ref_name
 set ref_name [get_attribute -q $cell ref_name]

 # Does it match any gsx0 Clock Gating Cell name ?
 return [regexp {(?i)^ICG.*} $ref_name]
}


# gsx0 LATCHs
proc	is_gsx0_lat	{cell_spec} {
 set cell [object -cell $cell_spec]
 if {$cell == ""} {
  echo "(is_gsx0_mux) -E- Only cell full_names/collections are supported !"
  return 0 
 }

 # Let's get the cell ref_name
 set ref_name [get_attribute -q $cell ref_name]

 # Does it match any gsx0 Latch name ?
 return [regexp {(?i)^(?:LAH|LAL).*} $ref_name]
}


# gsx0 Antenna Protection Diodes
proc	is_gsx0_apd	{cell_spec} {
 set cell [object -cell $cell_spec]
 if {$cell == ""} {
  echo "(is_gsx0_mux) -E- Only cell full_names/collections are supported !"
  return 0 
 }

 # Let's get the cell ref_name
 set ref_name [get_attribute -q $cell ref_name]

 # Does it match any gsx0 Latch name ?
 return [regexp {(?i)^AP0.*} $ref_name]
}


# gsx0 Mux Cells
proc	is_gsx0_mux	{cell_spec} {
 set cell [object -cell $cell_spec]
 if {$cell == ""} {
  echo "(is_gsx0_mux) -E- Only cell full_names/collections are supported !"
  return 0 
 }

 # Let's get the cell ref_name
 set ref_name [get_attribute -q $cell ref_name]

 # Does it match any gsx0 Multiplexer Cell name ?
 return [regexp {(?i)^(?:DMU|MU|CTGMU|UC43[12]).*} $ref_name]
}

# gsx0 Clock Mux Cells
proc	is_gsx0_clkmux	{cell_spec} {
 set cell [object -cell $cell_spec]
 if {$cell == ""} {
  echo "(is_gsx0_clkmux) -E- Only cell full_names/collections are supported !"
  return 0 
 }

 # Let's get the cell ref_name
 set ref_name [get_attribute -q $cell ref_name]

 # Does it match any gsx0 Multiplexer Cell name ?
 return [regexp {(?i)^UC43[12].*} $ref_name]
}

# GS60 IO PADs
proc	is_gsx0_io	{cell_spec} {
 set cell [object -cell $cell_spec]
 if {$cell == ""} {
  echo "(is_gsx0_io) -E- Only cell full_names/collections are supported !"
  return 0 
 }

 # Let's get the cell ref_name
 set ref_name [get_attribute -q $cell ref_name]

 # Does it match any GS60 IO PADs ?
 return [regexp {(?i)^[BI]Q.*} $ref_name]
}
