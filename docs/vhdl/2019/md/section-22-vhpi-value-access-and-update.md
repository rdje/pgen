---
title: "Section 22: IEEE Standard for VHDL Language Reference Manual"
document: "VHDL Language Reference Manual"
standard: "IEEE 1076-2019"
domain: "VHDL"
section: "22"
source_txt: "section-22-vhpi-value-access-and-update.txt"
source_pdf: "/Users/richarddje/Documents/github/VHDL-LRM-IEEE-1076-2019.pdf"
---

# Section 22: IEEE Standard for VHDL Language Reference Manual

IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
439
Copyright © 2019 IEEE. All rights reserved.
22. VHPI value access and update
### 22.1 General

This clause describes the data structures and operations provided in the VHPI for reading and updating
values of objects in a VHDL model.
### 22.2 Value structures and types

#### 22.2.1 General

The VHPI header file (see Annex B) defines a number of data types that are used by VHPI function. They
are described in this subclause (22.2).
It is an error if a VHPI program uses a given type described in this clause to represent a VHDL scalar type,
and there are position numbers in the scalar type that exceed the range of position numbers that can be
represented in the given type.
#### 22.2.2 vhpiEnumT and vhpiSmallEnumT

A value of type vhpiEnumT or vhpiSmallEnumT represents a value of a VHDL enumeration type. A
value of type vhpiEnumT shall be represented with at least 32 bits, and a value of type vhpiSmallE-
numT shall be represented with at least 8 bits. The value represented by a given value of either type is an
enumeration value whose position number is the given value, interpreted as an unsigned binary number.
#### 22.2.3 vhpiIntT and vhpiLongIntT

A value of type vhpiIntT or vhpiLongIntT represents a value of a VHDL integer type. A value of
type vhpiIntT shall be represented with at least 32 bits, and a value of type vhpiLongIntT shall be
represented with at least 64 bits. The value represented by a given value of either type is the given value,
interpreted as a signed twos-complement binary number.
#### 22.2.4 vhpiCharT

A value of type vhpiCharT represents a value of a VHDL character type. The value shall be represented
with at least 8 bits. The value represented by a given value of type vhpiCharT is a character value whose
position number is the given value, interpreted as an unsigned binary number.
#### 22.2.5 vhpiRealT

A value of type vhpiRealT represents a value of a VHDL floating-point type. The value shall be
represented with at least 64 bits. The value represented by a given value of type vhpiRealT is the given
value, interpreted according to the chosen representation for floating-point types (see 5.2.5.1).
#### 22.2.6 vhpiPhysT and vhpiSmallPhysT

A value of type vhpiPhysT is called a physical structure and represents a value of a physical type. The
position number of a physical structure is the signed integer represented by the concatenation of the high
and low members of the physical structure to form a 64-bit twos-complement binary number, with the
high member as the most significant part and the low member as the least significant part.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
440
Copyright © 2019 IEEE. All rights reserved.
A value of type vhpiSmallPhysT also represents a value of a physical type. The value shall be
represented with at least 32 bits. The position number of the value of type vhpiSmallPhysT is the value
interpreted as a signed twos-complement binary number.
If a physical structure or value of type vhpiSmallPhysT occurs as part of a value structure or as an
element of an array pointed to by a value structure, its position number determines the value represented by
the value structure or value of type vhpiSmallPhysT, as described in 22.2.8. Otherwise, the physical
structure or value of type vhpiSmallPhysT represents a value of a physical type. The value is the product
of the position number of the physical structure or value of type vhpiSmallPhysT and a unit determined
from the context in which the physical structure or value of type vhpiSmallPhysT occurs.
#### 22.2.7 vhpiTimeT

A value of type vhpiTimeT is called time structure and represents a time value. The position number of a
time structure is the signed integer represented by the concatenation of the high and low members of the
time structure to form a 64-bit twos-complement binary number, with the high member as the most
significant part and the low member as the least significant part.
If a time structure occurs as part of a value structure or as an element of an array pointed to by a value
structure, its position number determines the value represented by the value structure, as described in 22.2.8.
Otherwise, the time structure represents a value of type TIME defined in package STANDARD. The value
is the product of the position number of the time structure and the resolution limit of the tool.
NOTE—A
VHPI
program
can
determine
the
resolution
limit
with
the
function
call
vhpi_get_phys(vhpiResolutionLimit, NULL).
#### 22.2.8 vhpiValueT

A value of type vhpiValueT is called a value structure and represents a scalar value, a one-dimensional
array of scalar values, or a value of any type represented in an implementation-defined internal
representation.
The format member of a value structure specifies the format of the value structure, that is, a value of type
vhpiFormatT that determines how the value is represented. The value member of the value structure is
a union that contains the value in the appropriate representation. The following formats are specified by this
standard:
vhpiBinStrVal
The value structure represents a scalar value. The position number of the scalar value
is represented in the str member of the value member using a pointer to a string
of binary digit characters interpreted as a binary number.
vhpiOctStrVal
The value structure represents a scalar value. The position number of the scalar value
is represented in the str member of the value member using a pointer to a string
of octal digit characters interpreted as an octal number.
vhpiDecStrVal
The value structure represents a scalar value. The position number of the scalar value
is represented in the str member of the value member using a pointer to a string
of decimal digit characters interpreted as a decimal number.
vhpiHexStrVal
The value structure represents a scalar value. The position number of the scalar value
is represented in the str member of the value member using a pointer to a string
of hexadecimal digit characters interpreted as a hexadecimal number.
vhpiEnumVal
The value structure represents an enumeration value. The enumeration value is
represented in the enumv member of the value member using a value of type
vhpiEnumT.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
441
Copyright © 2019 IEEE. All rights reserved.
vhpiSmallEnumVal
The value structure represents an enumeration value. The enumeration value is
represented in the smallenumv member of the value member using a value of
type vhpiSmallEnumT.
vhpiIntVal
The value structure represents an integer value. The integer value is represented in
the intg member of the value member using a value of type vhpiIntT.
vhpiLongIntVal
The value structure represents an integer value. The integer value is represented in
the longintg member of the value member using a value of type
vhpiLongIntT.
vhpiLogicVal
The value structure represents a logic value of type STD_ULOGIC or STD_LOGIC
defined in the package IEEE.STD_LOGIC_1164. The logic value is represented in
the enumv member of the value member using a value of type vhpiEnumT.
vhpiRealVal
The value structure represents a floating-point value. The floating-point value is
represented in the real member of the value member using a value of type
vhpiRealT.
vhpiStrVal
The value structure represents a string of characters. The string is represented in the
str member of the value member using a pointer to a null-terminated array of
characters.
vhpiCharVal
The value structure represents a character value. The character value is represented in
the ch member of the value member using a value of type vhpiCharT.
vhpiTimeVal
The value structure represents a time value. The time value is represented in the
time member of the value member using a time structure.
vhpiPhysVal
The value structure represents a physical value. The physical value is represented in
the phys member of the value member using a physical structure.
vhpiSmallPhysVal
The value structure represents a physical value. The physical value is represented in
the smallphys member of the value member using a value of type
vhpiSmallPhysT.
vhpiObjTypeVal
This format is used by a VHPI program to specify that the tool provide the value of
an object in a format that is appropriate for the type of the object (see 22.4).
vhpiPtrVal
The value structure represents an access value. The access value is represented in the
ptr member of the value member using a pointer.
vhpiEnumVecVal
The value structure represents a one-dimensional array of enumeration values. The
array value is represented in the enumvs member of the value member using a
pointer to an array of values of type vhpiEnumT.
vhpiSmallEnumVecVal
The value structure represents a one-dimensional array of enumeration values. The
array value is represented in the smallenumvs member of the value member
using a pointer to an array of values of type vhpiSmallEnumT.
vhpiIntVecVal
The value structure represents a one-dimensional array of integer values. The array
value is represented in the intgs member of the value member using a pointer to
an array of values of type vhpiIntT.
vhpiLongIntVecVal
The value structure represents a one-dimensional array of integer values. The array
value is represented in the longintgs member of the value member using a
pointer to an array of values of type vhpiLongIntT.
vhpiLogicVecVal
The value structure represents a one-dimensional array of logic values of type
STD_ULOGIC or STD_LOGIC defined in the package IEEE.STD_LOGIC_1164.
The array value is represented in the enumvs member of the value member using a
pointer to an array of values of type vhpiEnumT.
vhpiRealVecVal
The value structure represents a one-dimensional array of floating-point values. The
array value is represented in the reals member of the value member using a
pointer to an array of values of type vhpiRealT.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
442
Copyright © 2019 IEEE. All rights reserved.
An implementation may specify further formats and the way in which values are represented for those
formats.
If a value structure is used by a VHPI program as an argument to the vhpi_get_value function and the
format of the value structure specifies an array, string, or internal representation, the VHPI program shall set
the bufSize member of the value structure to the number of bytes of storage allocated by the VHPI
program for storage of the value (see 23.19).
If the format of a value structure used to represent a value specifies an array or string representation, the
numElems member of the value structure specifies the number of elements in the array or string
representation of the value represented by the value structure. If the value is represented as a string, the
number of elements excludes the null termination character of the string.
If the format of a value structure used to represent a value specifies a physical type or time type
representation, the unit member of the value structure specifies a unit of the physical or time type. The
position number of the value represented by the value structure is the product of the position number of the
unit and the position number of the physical or time structure or value of type vhpiSmallPhysT used to
represent the value.
NOTE 1—A VHPI program that allocates buffer storage for a string to be written by a call to the vhpi_get_value
function should allow storage for the null termination character. The value written to the bufSize member of the value
structure should be at least one more than the length of the string.
NOTE 2—The vhpiRawDataVal format allows a VHPI program to read the value of an object without requiring the
tool to reformat the value. An implementation may allow a VHPI program to read the value of an object in its internal
representation and subsequently to set the value of an object of the same type using the value, thus avoiding the
performance impact of reformatting.
### 22.3 Reading object values

A VHPI program may read the value of certain objects in the design hierarchy information model using the
vhpi_get_value function (see 23.19). The objects for which it is legal to read the value are:
—
An object of class name
—
An object of class driver
—
An object of class transaction
—
An object of class port
vhpiTimeVecVal
The value structure represents a one-dimensional array of time values. The array
value is represented in the times member of the value member using a pointer to
an array of time structures.
vhpiPhysVecVal
The value structure represents a one-dimensional array of physical values. The array
value is represented in the physs member of the value member using a pointer to
an array of physical structures.
vhpiSmallPhysVecVal
The value structure represents a one-dimensional array of physical values. The array
value is represented in the smallphyss member of the value member using a
pointer to an array of values of type vhpiSmallPhysT.
vhpiPtrVecVal
The value structure represents a one-dimensional array of access values. The array
value is represented in the ptrs member of the value member using a pointer to an
array of pointers.
vhpiRawDataVal
The value structure represents a value in the ptr member of the value member
using a pointer to an implementation defined internal representation.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
443
Copyright © 2019 IEEE. All rights reserved.
—
An object of class literal
—
An object of class expr for which the Staticness property has the value
vhpiLocallyStatic or vhpiGloballyStatic
It is an error if a VHPI program uses the vhpi_get_value function to read the value of an object whose
type is other than a scalar type or a one-dimensional array type whose element type is a scalar type, unless
the format specified in the value structure is vhpiRawDataVal or an implementation-defined format (see
22.2.8). Furthermore, it is an error if a VHPI program uses the vhpi_get_value function to read the
value of an object of class name that does not represent a locally static name.
The effect of reading the value of a given object of class aliasDecl is the same as the effect of reading
the value of the target object of the aliasedName association with the given object as the reference object.
A VHPI program may read the value of an object during the elaboration phase provided the object has been
elaborated. A VHPI program may read the value of a formal parameter of a subprogram provided the formal
parameter has been dynamically elaborated as part of a call to the subprogram. A VHPI program may read
the value of an object during the initialization and simulation phases.
For an object of class constant, variable, or driver, or for an object of class signal other than an
object of class outPort, an object of class portDecl representing a port of mode out or an object of
class sigParamDecl representing a signal parameter of mode out, the vhpi_get_value function
yields the current value of the VHDL object represented by the object. For an object of class outPort or
an object of class portDecl representing a port of mode out, the vhpi_get_value function yields the
driving value of the VHDL object represented by the object. For an object of class sigParamDecl
representing a signal parameter of mode out, the vhpi_get_value function yields the driving value of
the driver for the signal parameter. For an object of class transaction, the vhpi_get_value function
yields the value component of the transaction represented by the object.
For an object of class file, if the file is open, the vhpi_get_value function yields a string whose value
is the file logical name. Otherwise, the vhpi_get_value function raises an error with severity
vhpiWarning.
For an object of class literal, the vhpi_get_value function returns the value of the literal
represented by the object.
For an object of class expr, the vhpi_get_value function returns the value of the expression
represented by the object.
NOTE 1—A VHPI program can read the value of an object of composite type by navigating associations in the
information model to acquire handles to subelements for which reading the value using the vhpi_get_value
function is legal.
NOTE 2—A VHPI program can, as an alternative to using the vhpi_get_value function, read the value of an object
representing a literal by reading the IntVal, RealVal, PhysVal, or StrVal property, as appropriate, of the object.
### 22.4 Formatting values

For each type of object whose value can be read using the vhpi_get_value function, there is a native
format, defined as follows.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
444
Copyright © 2019 IEEE. All rights reserved.
If a VHPI program calls the vhpi_get_value function with the format member of the value structure
set to vhpiObjTypeVal, the function yields the value of the object formatted using the native format and
updates the format member with the value of type vhpiFormatT corresponding to the native format
used. For types for which there is more than one native format, the function may return the value in either
format, provided the range of position numbers in the type or element type (as appropriate) is representable
in the format.
A tool shall support reading of the value of an object using the native format of the object, the
vhpiObjTypeVal format, and the vhpiRawDataVal format. An implementation may also support
reading of the value of an object using other formats.
Object type
Native format
Any integer type
vhpiIntVal or vhpiLongIntVal
Any enumeration type other than CHARACTER, or the
type STD_LOGIC or STD_ULOGIC defined in
IEEE.STD_LOGIC_1164
vhpiEnumVal or vhpiSmallEnumVal
CHARACTER
vhpiCharVal
STD_LOGIC or STD_ULOGIC defined in
IEEE.STD_LOGIC_1164
vhpiLogicVal
Any physical type other than TIME
vhpiPhysVal or vhpiSmallPhysVal
TIME
vhpiTimeVal
Any floating-point type
vhpiRealVal
Any access type
vhpiPtrVal
Any one-dimensional array type whose element type is
an integer type
vhpiIntVecVal or vhpiLongIntVecVal
Any one-dimensional array type whose element type is
an enumeration type other than CHARACTER or the type
STD_LOGIC or STD_ULOGIC defined in
IEEE.STD_LOGIC_1164
vhpiEnumVecVal or vhpiSmallEnumVecVal
Any one-dimensional array type whose element type is
CHARACTER
vhpiStrVal
Any one-dimensional array type whose element type is
STD_LOGIC or STD_ULOGIC defined in
IEEE.STD_LOGIC_1164
vhpiLogicVecVal
Any one-dimensional array type whose element type is
any physical type other than TIME
vhpiPhysVecVal or vhpiSmallPhysVecVal
Any one-dimensional array type whose element type is
TIME
vhpiTimeVecVal
Any one-dimensional array type whose element type is
any floating-point type
vhpiRealVecVal
Any one-dimensional array type whose element type is
any access type
vhpiPtrVecVal
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
445
Copyright © 2019 IEEE. All rights reserved.
### 22.5 Updating object values

#### 22.5.1 General

A VHPI program may update the value of certain objects in the design hierarchy information model using
the vhpi_put_value function (see 23.28). The objects for which it is legal to update the value are:
—
An object of one of the following subclasses of objDecl: genericDecl, sigDecl, varDecl,
portDecl, sigParamDecl, or varParamDecl
—
An object of class aliasDecl whose target object of the aliasedName association is an object
for which it is legal to update the value
—
An object of one of the following subclasses of prefixedName: indexedName, sliceName,
or selectedName, provided the target object of the prefix association is an object for which it
is legal to update the value
—
An object of class derefObj
—
An object of class driver
—
An object of class port
—
An object of class funcCall
The effect of a call to the vhpi_put_value function to update an object of class genericDecl is not
specified by this standard.
The effect of updating the value of a given object of class aliasDecl is the same as the effect of updating
the value of the target object of the aliasedName association with the given object as the reference object.
A VHPI program may use the vhpi_put_value function to update the value of the following objects
during the elaboration phase provided the object to be updated has been elaborated or created:
—
A signal or port of a foreign architecture
—
A variable that is elaborated as part of elaboration of a shared variable, of a protected type or of a
foreign architecture
—
A driver created using the vhpi_create function
—
The return value of a foreign function
A VHPI program may update the value of an object during the initialization and simulation phases. A VHPI
program may update the value of a formal parameter of a subprogram provided the formal parameter is of
mode out or inout and has been dynamically elaborated as part of a call to the subprogram. It is an error if a
VHPI program updates the value of a formal parameter of mode in.
The VHPI header file defines the enumeration type vhpiPutValueModeT with enumeration constants
corresponding to update modes as follows:
vhpiDeposit
The value of an object is updated, with no propagation of signal values.
vhpiDepositPropagate
The value of an object is updated, and, if the object is a signal on a net, the updated
value is propagated to other signals on the net.
vhpiForce
An object is forced to a given value, with no propagation of signal values.
vhpiForcePropagate
An object is forced to a given value, and, if the object is a signal on a net, the updated
value is propagated to other signals on the net.
vhpiRelease
The forcing of an object is released.
vhpiSizeConstraint
The constraint of the type of an object is set.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
446
Copyright © 2019 IEEE. All rights reserved.
For objects of class other than signal, the effect of an update with update mode
vhpiDepositPropagate is the same as an update with update model vhpiDeposit, and the effect of
an update with update mode vhpiForcePropagate is the same as an update with update model
vhpiForce.
If the vhpi_put_value function is called with an update mode of vhpiRelease, no value structure is
required, and the value of the value_p argument is ignored.
It is an error if a VHPI program uses the vhpi_put_value function to update the value of an object
whose type is other than a scalar type or a one-dimensional array type whose element type is a scalar type,
unless the format specified in the value structure is vhpiRawDataVal or the update mode is
vhpiRelease. Furthermore, it is an error if a VHPI program uses the vhpi_put_value function to
update the value of an object of class name that does not represent a locally static name.
#### 22.5.2 updating an object of class variable

A call to the vhpi_put_value function to update the value of an object of class variable shall use an
update mode of vhpiDeposit, vhpiDepositPropage, vhpiForce, or vhpiForcePropagate.
A call to the vhpi_put_value function to update the value of an object of class variable with an
update mode of vhpiForce or vhpiForcePropagate causes the variable represented by the object to
become forced and to be updated with the value represented by the value structure provided to the
vhpi_put_value function. The value of a variable that is forced is not updated by a variable assignment
statement or by association as an actual parameter with a formal variable parameter. The variable remains
forced until a subsequent update with an update mode of vhpiRelease, which causes the variable to be
released, that is, no longer to be forced.
Subelements of a variable of composite type may be separately forced. If a variable of composite type is
forced, all of its subelements are forced. If a variable of composite type is released, all of the subelements of
the variable are released.
For a formal variable parameter, if the parameter is passed by reference, forcing or releasing the formal
parameter causes the actual parameter to be forced or released, respectively, and forcing or releasing the
actual parameter causes the formal parameter to be forced or released, respectively. Otherwise, if the
parameter is passed by copy, forcing or releasing the formal parameter has no effect on whether the actual
parameter is forced or released, and forcing or releasing the actual parameter has no effect on whether the
formal parameter is forced or released.
A call to the vhpi_put_value function to update the value of an object of class variable with an
update mode of vhpiDeposit or vhpiDepositPropagate causes the variable represented by the
object to be updated with the value represented by the value structure provided to the vhpi_put_value
function, provided the variable is not forced.
NOTE—If a forced variable is updated with an update mode of vhpiDeposit or vhpiDepositPropagate, the
update has no effect.
#### 22.5.3 updating an object of class signal

A call to the vhpi_put_value function to update the value of an object of class signal shall use an
update mode of vhpiDeposit, vhpiDepositPropage, vhpiForce, vhpiForcePropagate, or
vhpiRelease.
A call to the vhpi_put_value function to update the value of one of the following objects:
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
447
Copyright © 2019 IEEE. All rights reserved.
—
An object of class portDecl representing a port of mode out
—
An object of class sigParamDecl representing a signal parameter of mode out
—
An object of class outPort
causes the driving value of the signal represented by the object to be updated; a call to update an object of
class signal other than one of the object described in the preceding list causes the effective value of the
signal represented by the object to be updated.
A call to the vhpi_put_value function to update the driving value of a signal with an update mode of
vhpiForce causes the signal to become driving-value forced. The variable containing the driving value of
the signal is updated with the value represented by the value structure provided to the vhpi_put_value
function. Similarly, a call to the vhpi_put_value function to update the effective value of a signal with
an update mode of vhpiForce causes the signal to become effective-value forced. The variable containing
the current value of the signal is updated with the value represented by the value structure provided to the
vhpi_put_value function.
A call to the vhpi_put_value function to update the driving value of a signal with an update mode of
vhpiForcePropagate schedules a driving-value force for the signal, with the driving force value for
the signal being the value represented by the value structure provided to the vhpi_put_value function.
The effect is to cause the signal to become driving-value forced during the next signal update phase of a
simulation cycle (see 14.7.3). Similarly, a call to the vhpi_put_value function to update the effective
value of a signal with an update mode of vhpiForcePropagate schedules an effective-value force for
the signal, with the effective force value for the signal being the value represented by the value structure
provided to the vhpi_put_value function. The effect is to cause the signal to become effective-value
forced during the next signal update phase of a simulation cycle.
If more than one driving-value force or more than one effective-value force is scheduled for a given signal
before that signal is updated, the effect is not specified by this standard.
A signal that is driving-value forced remains so until a subsequent update of the signal with an update mode
of vhpiRelease, which causes the signal to be driving-value released, that is, no longer to be driving-
value forced, or until the signal becomes driving-value released during the signal update phase of a
simulation cycle. Similarly, a signal that is effective-value forced remains so until a subsequent update of the
signal with an update mode of vhpiRelease, which causes the signal to be effective-value released, that
is, no longer to be effective-value forced, or until the signal becomes effective-value released during the
signal update phase of a simulation cycle.
Subelements of a signal of composite type may be separately forced. If a signal of composite type is forced,
all of its subelements are forced. If a signal of composite type is released, all of the subelements of the signal
are released.
A call to the vhpi_put_value function to update the driving value of a signal with an update mode of
vhpiDeposit causes the variable containing the driving value of the signal to be updated with the value
represented by the value structure provided to the vhpi_put_value function, provided the signal is not
driving-value forced. Similarly, a call to the vhpi_put_value function to update the effective value of a
signal with an update mode of vhpiDeposit causes the variable containing the current value of the signal
to be updated with the value represented by the value structure provided to the vhpi_put_value
function, provided the signal is not effective-value forced.
A call to the vhpi_put_value function to update the driving value of a signal with an update mode of
vhpiDepositPropagate schedules a driving-value deposit for the signal, with the driving deposit
value for the signal being the value represented by the value structure provided to the vhpi_put_value
function. The effect is to update the variable containing the driving value of the signal during the next signal
update phase of a simulation cycle (see 14.7.3). Similarly, a call to the vhpi_put_value function to
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
448
Copyright © 2019 IEEE. All rights reserved.
update the effective value of a signal with an update mode of vhpiDepositPropagate schedules an
effective-value deposit for the signal, with the effective deposit value for the signal being the value
represented by the value structure provided to the vhpi_put_value function. The effect is to update the
variable containing the current value of the signal during the next signal update phase of a simulation cycle.
If more than one driving-value deposit or more than one effective-value deposit is scheduled for a given
signal before that signal is updated, the effect is not specified by this standard.
NOTE—If both a deposit and a force are scheduled for a given signal, the force takes precedence over the deposit.
Furthermore, if a forced signal is updated with an update mode of vhpiDeposit, the update has no effect.
#### 22.5.4 Updating an object of class driver

A call to the vhpi_put_value function to update the value of an object of class driver shall use an
update mode of vhpiDeposit, vhpiDepositPropage, vhpiForce, vhpiForcePropagate, or
vhpiRelease.
A call to the vhpi_put_value function to update the value of an object of class driver with an update
mode of vhpiForce causes the driver represented by the object to become forced. The variable containing
the current value of the driver is updated with the value represented by the value structure provided to the
vhpi_put_value function.
A call to the vhpi_put_value function to update the value of an object of class driver with an update
mode of vhpiForcePropagate schedules a force for the driver represented by the object, with the force
value for the driver being the value represented by the value structure provided to the vhpi_put_value
function. The effect is to cause the driver to become forced during the next signal update phase of a
simulation cycle (see 14.7.3).
If more than one force is scheduled for a given driver before that driver is updated, the effect is not specified
by this standard.
A driver that is forced remains so until a subsequent update of the driver with an update mode of
vhpiRelease, which causes the driver to be released, that is, no longer to be forced.
A call to the vhpi_put_value function to update the value of an object of class driver with an update
mode of vhpiDeposit causes the variable containing the current value of the driver represented by the
object to be updated with the value represented by the value structure provided to the vhpi_put_value
function, provided the driver is not forced.
A call to the vhpi_put_value function to update the value of an object of class driver with an update
mode of vhpiDepositPropagate schedules a deposit for the driver represented by the object, with the
deposit value for the driver being the value represented by the value structure provided to the
vhpi_put_value function. The effect is to update the variable containing the current value of the driver
during the next signal update phase of a simulation cycle (see 14.7.3).
If more than one deposit is scheduled for a given driver before that driver is updated, the effect is not
specified by this standard.
NOTE—If both a deposit and a force are scheduled for a given driver, the force takes precedence over the deposit.
Furthermore, if a forced driver is updated with an update mode of vhpiDeposit, the update has no effect.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
449
Copyright © 2019 IEEE. All rights reserved.
#### 22.5.5 Updating an object of class funcCall

For an object of class funcCall representing a function call to a foreign function, the execution function
of the foreign function shall define the result returned by the function call.
If the result subtype of the function is an unconstrained type, the execution function shall set the constraint
of the object of class funcCall using the vhpi_put_value function with an update mode of
vhpiSizeConstraint, and subsequently use a call or calls to the vhpi_put_value function to
define the result. For the call to the vhpi_put_value function that sets the constraint, the numElems
member of the value structure is the number of elements in the result array. Other members of the value
structure are ignored.
If the result subtype of the function is a type for which values can be represented in a single value structure,
the execution function may define the result using a single call to the vhpi_put_value function to
update the object of class funcCall. If the result subtype of the function is a one-dimensional array type
whose element type is a scalar type, the execution function may define the result using a single call to the
vhpi_put_value function to update the object of class funcCall, or may define the result using
multiple calls to the vhpi_put_value function, as described in the following paragraph.
If the result subtype of the function is a type for which values cannot be represented in a single value
structure, the execution function shall define the result using multiple calls to the vhpi_put_value
function. The execution function shall navigate associations from the object of class funcCall to objects
of class name that represent elements of the result for which values can be represented in a single value
structure, and call the vhpi_put_value function for each such object to update the value of the element
represented by the object.
A call to the vhpi_put_value function to define the result shall use an update mode of vhpiDeposit,
vhpiDepositPropage, vhpiForce, or vhpiForcePropagate. The effect, in each case, is to
update the object immediately. A call to the vhpi_put_value function with update mode
vhpiRelease to define the result has no effect.
If the result subtype of the function is a composite type, it is an error if the call or calls to the
vhpi_put_value function that define the result before the execution function returns do not define the
values of all elements of the result.
An implementation may allow a VHPI program to update the value of an object of class funcCall
representing a function call to a function other than a foreign function; the effect is not specified by this
standard.
### 22.6 Scheduling transactions on drivers

A VHPI program may schedule a transaction on a driver or transactions on drivers in a collection of drivers
using the vhpi_schedule_transaction function (see 23.34). The effect of scheduling a transaction
on a driver is to modify the projected output waveform of the driver according to the rules described in
10.5.2.2. The value provided for each driver in a value structure to the vhpi_schedule_transaction
function is used as the value component of a transaction assigned to the driver. The time component of the
transaction and the delay mechanism are determined as described in 23.34.
If the value_p argument provided to the vhpi_schedule_transaction function is NULL, a null
transaction is scheduled for the driver, or for each driver in the collection, as appropriate, represented by the
object referred to by the handle provided in the drivHdl argument. It is an error if a null transaction is
scheduled for a driver that is not a driver for a guarded signal. The effect of scheduling a null transaction on
a driver defined by a sequential assignment statement or using the function vhpi_create is described in
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
450
Copyright © 2019 IEEE. All rights reserved.
10.5.2.2. The effect of using the vhpi_schedule_transaction function to schedule a null
transaction on a driver defined by a concurrent signal assignment statement is not specified by this standard.
If the value_p argument is not NULL, it shall point to a value structure or an array of value structures that
are used to specify values of transactions. The number of value structures is specified by the numValues
argument. In certain cases, a single value structure shall be provided, with the numValues argument being
1, as follows:
—
If the drivHdl argument is a handle that refers to an object of class driver representing a driver
for a scalar signal, the value structure shall represent a scalar value that can legally be assigned to the
driver, and that value is used as the value of the transaction for the driver.
—
If the drivHdl argument is a handle that refers to an object of class driver representing a driver
for a resolved signal of an array type whose element type is a scalar type, the value structure shall
represent an array of scalar values that can legally be assigned to the signal, and that value is used as
the value of the transaction for the driver.
—
If the drivHdl argument is a handle that refers to an object of class driverCollection
representing a collection of drivers for elements of a signal of an array type whose element type is a
scalar type, the value structure shall represent an array of scalar values, each of which can legally be
assigned to an element of the signal. There shall be as many elements in the array as there are
members of the collection. The value of an element of the array with a given index is used as the
value of the transaction for the driver in the collection with the given index.
In other cases, an array of value structures shall be provided and is used as follows.
For a given driver, either represented by an object of class driver referred to by the handle provided as the
drivHdl argument or in a collection of drivers represented by an object of class driverCollection
referred to by that handle, the type of the signal driven by the driver is referred to as the driver type. For
certain subelements of the driver type, and for the driver type itself, the value or values represented by a
subarray of one or more contiguous value structures or by the entire array of value structures are formed into
a transaction subvalue of the type of the subelement or of the driver type, respectively. The transaction
subvalue for the driver type is used as the value of the transaction for the given driver.
For a subelement that is a scalar record element, the transaction subvalue is formed from the value
represented by a single value structure. That value shall be a scalar value that can legally be assigned to a
signal of the type of the scalar record element.
For a subelement that is an array whose element type is a scalar type, the transaction subvalue is formed
from the value represented by a single value structure. That value shall be an array of scalar values, each of
which can legally be assigned to a signal of the element type of the subelement. There shall be as many
elements in the array as there are elements in the subelement.
For a subelement or a driver type that is an array whose element type is other than a scalar type, the
transaction subvalue is formed from the concatenation of distinct subarrays corresponding to each element
of the array. The subarrays occur contiguously in the array of value structures in the same order as elements
in the array and are concatenated in that order to form the transaction subvalue for the array.
For a subelement or driver type that is a record, the transaction subvalue is formed from the concatenation of
distinct subarrays corresponding to each element of the record. The subarrays occur contiguously in the
array of value structures in the same order as the order in which the elements are declared in the record type
definition for the type of the subelement or driver type, as appropriate, and are concatenated in that order to
form the transaction subvalue for the array.
If the drivHdl argument is a handle that refers to an object of class driver, the array of value structures
is used to form the transaction subvalue for the driver type of the driver represented by the object, and the
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
451
Copyright © 2019 IEEE. All rights reserved.
transaction subvalue is used as the value of the transaction for the driver. It is an error if the number of value
structures is insufficient to form the transaction subvalue.
If the drivHdl argument is a handle that refers to an object of class driverCollection, a transaction
subvalue is formed from a distinct subarray for each member of the collection represented by the object. The
subarrays occur contiguously in the array of value structures in the same order as the order in which the
members occur in the collection. The transaction subvalue for each member is used as the value of the
transaction for the member. It is an error if the number of value structures is insufficient to form the
transaction subvalues.
NOTE—An object of class driver represents a driver for a basic signal.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
