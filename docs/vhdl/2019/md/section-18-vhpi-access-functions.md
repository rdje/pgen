---
title: "Section 18: IEEE Standard for VHDL Language Reference Manual"
document: "VHDL Language Reference Manual"
standard: "IEEE 1076-2019"
domain: "VHDL"
section: "18"
source_txt: "section-18-vhpi-access-functions.txt"
source_pdf: "/Users/richarddje/Documents/github/VHDL-LRM-IEEE-1076-2019.pdf"
---

# Section 18: IEEE Standard for VHDL Language Reference Manual

IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
360
Copyright © 2019 IEEE. All rights reserved.
18. VHPI access functions
### 18.1 General

This clause describes the VHPI functions that are used by VHPI programs to access the information model
of a VHDL model.
### 18.2 Information access functions

#### 18.2.1 General

The VHPI information access functions allow a VHPI program to navigate an association between objects.
The VHPI header file defines enumeration types that contain enumeration constants corresponding to
association roles specified implicitly or explicitly in the information model. The name of each enumeration
constant is the name of the corresponding role prefixed with the letters vhpi.
#### 18.2.2 One-to-one association traversal

The VHPI header file defines the enumeration type vhpiOneToOneT that contains enumeration constants
corresponding to one-to-one association roles.
If the information model includes a one-to-one association that is navigable from a reference class to a target
class, the function vhpi_handle navigates from an object of the reference class to an object of the target
class (see 23.20).
Examples:
Given the information model described by the UML class diagram shown in Figure 1, the following VHPI
program navigates from an object of the compInstStmt class to an object of the designUnit class
using the enumeration constant vhpiDesignUnit.
Figure 1—UML class diagram
void get_binding_info(vhpiHandleT instHdl) {
  char duName[MAXSTR];
  char libName[MAXSTR];
designInstUnit
compInstStmt
packInst
rootInst
designUnit
1
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
361
Copyright © 2019 IEEE. All rights reserved.
  vhpiHandleT duHdl;
  switch (vhpi_get(vhpiKindP, instHdl)) {
  case vhpiCompInstStmtK:
  case vhpiRootInstK:
  case vhpiPackInstK:
    duHdl = vhpi_handle(vhpiDesignUnit, instHdl);
    sprintf (duName, "%s", vhpi_get_str(vhpiUnitNameP, duHdl));
    sprintf(libName, "%s", vhpi_get_str(vhpiLibLogicalNameP, duHdl));
    vhpi_printf("design unit name %s in library %s\n", duName, libName);
    break;
  default:
    break;
  }/* end switch */
}/* get_binding_info() */
Given the information model described by the UML class diagram shown in Figure 2, the following VHPI
program navigates from an object of the waitStmt class to one object of the expr class using the
enumeration constant vhpiCondExpr and to a second object of the expr class using the enumeration
constant vhpiTimeOutExpr.
Figure 2—UML class diagram
vhpiHandleT stmtHdl, condHdl, timeHdl;
if (vhpi_get(vhpiKindP, stmtHdl) == vhpiWaitStmtK) {
  condHdl = vhpi_handle(vhpiCondExpr, stmtHdl);
  timeHdl = vhpi_handle(vhpiTimeOutExpr, stmtHdl);
}
waitStmt
expr
+CondExpr
0..1
+TimeOutExpr
0..1
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
362
Copyright © 2019 IEEE. All rights reserved.
#### 18.2.3 One-to-many association traversal

The VHPI header file defines the enumeration type vhpiOneToManyT that contains enumeration
constants corresponding to one-to-many association roles.
If the information model includes a one-to-many association that is navigable from a reference class to a
target class, the function vhpi_iterator navigates from an object of the reference class to a set of
objects of the target class (see 23.24).
If the information model includes an ordered one-to-many association that is navigable from a reference
class to a target class, the function vhpi_handle_by_index navigates from an object of the reference
class to an object of the target class (see 23.21).
NOTE 1—A VHPI program can use the vhpi_scan function to access the objects referred to by an iterator.
NOTE 2—If the association navigated by the vhpi_iterator function is not an ordered association, the order of
objects returned by applying vhpi_scan to the iterator is not defined.
Example:
vhpiHandleT instHdl, instIter;
/* get all sub-instances of a scope instance */
instIter = vhpi_iterator(vhpiInternalRegions, instHdl);
if (instIter) {
  while (instHdl = vhpi_scan(instIter)) {
    vhpi_printf("found instance %s\n",
                vhpi_get_str(vhpiNameP, instHdl));
  }
}
### 18.3 Property access functions

#### 18.3.1 General

The VHPI property access functions allow a VHPI program to access property values of objects.
The VHPI header file defines enumeration types that contain enumeration constants corresponding to
properties of classes specified in the information model. The name of each enumeration constant is the name
of the corresponding property prefixed with the letters vhpi and suffixed with the uppercase letter P.
#### 18.3.2 Integer and Boolean property access function

The VHPI header file defines the enumeration type vhpiIntPropertyT that contains enumeration
constants corresponding to integer and Boolean properties. The header file defines the type vhpiIntT that
is used to represent values of integer and Boolean properties. The header file defines the integer constant
vhpiFalse that is used to represent the value of a Boolean property that is false and the integer constant
vhpiTrue that is used to represent the value of a Boolean property that is true.
The function vhpi_get accesses an integer or Boolean property of an object (see 23.10).
NOTE—Some properties may legally take on the same value as the constant vhpiUndefined. In such cases, a VHPI
program should use the vhpi_check_error to determine whether a call to vhpi_get resulted in an error.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
363
Copyright © 2019 IEEE. All rights reserved.
#### 18.3.3 String property access function

The VHPI header file defines the enumeration type vhpiStrPropertyT that contains enumeration
constants corresponding to string properties.
The function vhpi_get_str accesses a string property of an object (see 23.17).
NOTE 1—Successive calls to vhpi_get_str may use the same storage for the results. A VHPI program that needs to
save the result of a call to vhpi_get_str should copy the result before subsequent calls to the function. (See
Clause 23.)
NOTE 2—String properties that represent VHDL pathnames and extended identifiers may contain non-letter graphic
characters, such as '\'. VHPI programs that use C string library functions or printf functions to operate on such strings
should take care that the special characters are not interpreted as escape characters by the functions.
#### 18.3.4 Real property access function

The VHPI header file defines the enumeration type vhpiRealPropertyT that contains enumeration
constants corresponding to real properties. The header file defines the type vhpiRealT that is used to
represent values of real properties.
The function vhpi_get_real accesses a real property of an object (see 23.16).
NOTE—A VHPI program should use the vhpi_check_error to determine whether a call to vhpi_get_real
resulted in an error.
#### 18.3.5 Physical property access function

The VHPI header file defines the enumeration type vhpiPhysPropertyT that contains enumeration
constants corresponding to physical properties. The header file defines the struct type vhpiPhysT that is
used to represent values of physical properties. The member high of the struct type represents the most sig-
nificant 32 bits of the position number of a value, and the member low represents the least significant 32 bits
of the position number of the value.
The function vhpi_get_phys accesses a physical property of an object (see 23.15).
NOTE—A VHPI program should use the vhpi_check_error to determine whether a call to vhpi_get_phys
resulted in an error.
### 18.4 Access by name function

If a class in the information model has the vhpiFullNameP property (see 19.4.7), the function
vhpi_handle_by_name (see 23.22) navigates to an object of the class.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
