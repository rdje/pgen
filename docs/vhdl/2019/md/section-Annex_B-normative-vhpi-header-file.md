---
title: "Section Annex.B: IEEE Standard for VHDL Language Reference Manual"
document: "VHDL Language Reference Manual"
standard: "IEEE 1076-2019"
domain: "VHDL"
section: "Annex.B"
source_txt: "section-Annex_B-normative-vhpi-header-file.txt"
source_pdf: "/Users/richarddje/Documents/github/VHDL-LRM-IEEE-1076-2019.pdf"
---

# Section Annex.B: IEEE Standard for VHDL Language Reference Manual

IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
527
Copyright © 2019 IEEE. All rights reserved.
Annex B
(normative)
VHPI header file
B.1 General
The VHPI header file, vhpi_user.h, shall be included by a VHPI tool. A tool provider should provide
the header file with the tool.
Several definitions in the VHPI header file are marked as deprecated. They are included for compatibility
with earlier versions of the VHPI than that defined by this standard. VHPI programs that conform to this
standard should not use definitions so marked. The function vhpi_get_foreign_info, which is
marked as deprecated, is defined to be the same as the vhpi_get_foreignf_info function. The
deprecated function will be removed in a future revision of this standard.
The content of vhpi_user.h is provided in the IEEE 1076 Open Source Repository.
B.2 Macros for sensitivity-set bitmaps
B.2.1 General
The macros for manipulating sensitivity-set bitmaps, defined in the header file, are described in this
subclause (B.2).
The definitions of the macros in the header file invoke functions defined in the file vhpi_sens.c
(provided in the IEEE 1076 Open Source Repository). A tool provider may replace the definitions with
implementation-specific definitions that have the effect described in this subclause (B.2). Such definitions
may invoke implementation-defined functions or may be in the form of in-line code.
B.2.2 VHPI_SENS_ZERO
Clears a sensitivity-set bitmap.
Synopsis:
VHPI_SENS_ZERO(sens)
Description:
The argument sens is a pointer to a sensitivity-set bitmap. The macro clears all of the bits in the
sensitivity-set bitmap to 0.
B.2.3 VHPI_SENS_SET
Sets a bit in a sensitivity-set bitmap.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
528
Copyright © 2019 IEEE. All rights reserved.
Synopsis:
VHPI_SENS_SET(obj, sens)
Description:
The argument obj is an integer representing the index of a signal in a sensitivity set, and the argument
sens is a pointer to a sensitivity-set bitmap. The macro sets to 1 the bit in the sensitivity-set bitmap
corresponding to the signal with the given index.
B.2.4 VHPI_SENS_CLR
Clears a bit in a sensitivity-set bitmap.
Synopsis:
VHPI_SENS_CLR(obj, sens)
Description:
The argument obj is an integer representing the index of a signal in a sensitivity set, and the argument
sens is a pointer to a sensitivity-set bitmap. The macro clears to 0 the bit in the sensitivity-set bitmap
corresponding to the signal with the given index.
B.2.5 VHPI_SENS_ISSET
Determines whether a specific bit in a sensitivity-set bitmap is set.
Synopsis:
VHPI_SENS_ISSET(obj, sens)
Description:
The argument obj is an integer representing the index of a signal in a sensitivity set, and the argument
sens is a pointer to a sensitivity-set bitmap. The macro yields an integer that is the value of the bit in the
sensitivity-set bitmap corresponding to the signal with the given index.
B.2.6 VHPI_SENS_FIRST
Determines whether any bit in a sensitivity-set bitmap is set.
Synopsis:
VHPI_SENS_FIRST(sens)
Description:
The argument sens is a pointer to a sensitivity-set bitmap. If any of the bits in the sensitivity-set bitmap
corresponding to signals in a sensitivity set is 1, the macro yields an integer that is the least index of the
signals for which the corresponding bit is set. Otherwise, the macro yields the value vhpiUndefined.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
529
Copyright © 2019 IEEE. All rights reserved.
B.3 Implementation-specific extensions
A tool provider may provide implementation-defined functionality in addition to that described by this
standard. Where such functionality requires declarations in the vhpi_user.h header file, those
declarations shall be provided by definitions of the following macros:
The macros shall be defined before compilation of the vhpi_user.h file and shall be defined in such a
way that their instantiation in the vhpi_user.h file results in legal C declarations.
The range of enumeration values from 1000 to 2000, inclusive, of enumeration constants of types
vhpiClassKindT,
vhpiOneToOneT,
vhpiOneToManyT,
vhpiIntPropertyT,
vhpiStrPropertyT, vhpiRealPropertyT, and vhpiPhysPropertyT are reserved and shall not
be used for implementation defined functionality.
VHPIEXTEND_VAL_FORMATS
Enumeration constants for implementation-defined value formats.
VHPIEXTEND_CLASSES
Enumeration constants for implementation-defined classes.
VHPIEXTEND_ONE_METHODS
Enumeration constants for implementation-defined one-to-one
associations.
VHPIEXTEND_MANY_METHODS
Enumeration constants for implementation-defined one-to-many
associations.
VHPIEXTEND_T_PROPERTIES
Enumeration constants for implementation-defined integer properties.
VHPIEXTEND_STR_PROPERTIES
Enumeration constants for implementation-defined string properties.
VHPIEXTEND_REAL_PROPERTIES
Enumeration constants for implementation-defined real properties.
VHPIEXTEND_PHYS_PROPERTIES
Enumeration constants for implementation-defined physical properties.
VHPIEXTEND_ATTR
Enumeration constants for implementation-defined attribute kinds.
VHPIEXTEND_CONTROL
Enumeration constants for implementation-defined control actions.
VHPIEXTEND_FUNCTIONS
Prototypes for implementation-defined functions.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
