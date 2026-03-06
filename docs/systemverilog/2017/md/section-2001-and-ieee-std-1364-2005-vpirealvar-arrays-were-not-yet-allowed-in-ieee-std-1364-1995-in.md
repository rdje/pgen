---
title: "Section 2001: IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language"
document: "SystemVerilog Language Reference Manual"
standard: "IEEE 1800-2017"
domain: "SystemVerilog"
section: "2001"
source_txt: "section-2001-and-ieee-std-1364-2005-vpirealvar-arrays-were-not-yet-allowed-in-ieee-std-1364-1995-in.txt"
source_pdf: "/Users/richarddje/Documents/github/SystemVerilog-LRM-IEEE-1800-2017.pdf"
---

# Section 2001: IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language

IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
963
Copyright © 2018 IEEE. All rights reserved.
represented as vpiRegArray objects, and vpiIntegerVar and vpiTimeVar objects are always non-
array variables (see 37.16).
4)
vpiRealVar can be an array
This object type was allowed to represent an unpacked array of such variables in IEEE Std 1364-
## 2001 and IEEE Std 1364-2005 (vpiRealVar arrays were not yet allowed in IEEE Std 1364-1995). In

IEEE 1800 standards, these are now exclusively represented as vpiRegArray objects (see 37.16).
5)
vpiVariables iterations include vpiReg and vpiRegArray
In all IEEE 1364 standards, vpiReg and vpiRegArray objects were excluded from vpiVariables
iterations, and only accessed instead by iterations on vpiReg (from a scope or vpiRegArray) or
vpiRegArray (from a scope), respectively. In IEEE 1800 standards, they are both included in
vpiVariables iterations (see 37.16).
6)
vpiReg iterations on vpiRegArray include other objects
This is a consequence of vpiRegArray objects being used to represent unpacked arrays of non-
vpiReg elements in IEEE 1800 standards (see 37.16). vpiReg iterations on these array objects can
retrieve array elements that are of type vpiIntegerVar or vpiTimeVar for example, which is not
expected in IEEE Std 1364-2001 and IEEE Std 1364-2005.
7)
vpiRegArray iterations include variable array objects
This is another consequence of vpiRegArray objects being used to represent unpacked arrays of
non-vpiReg elements in IEEE 1800 standards (see 37.16). In IEEE Std 1364-2001 and
IEEE Std 1364-2005, vpiRegArray iterations only included arrays of vpiReg objects, but in
IEEE 1800 standards, this iteration includes arrays of vpiIntegerVar, vpiTimeVar, and
vpiRealVar.
8)
vpiInterfaceDecl iterations allowed on vpiClassDefn objects
The vpiInterfaceDecl iteration (aliased to vpiVirtualInterfaceVar in IEEE Std 1800-2012) was
allowed on vpiClassDefn objects in IEEE Std 1800-2005 and IEEE Std 1800-2009, but this has
been disallowed in IEEE Std 1800-2012. It was deemed to be misleading since vpiClassDefn
objects are lexical-only scopes. This iteration remains allowed for vpiClassTypespec objects, which
can represent active scopes.
9)
vpiInterfaceDecl iterations produce vpiRefObj objects
The vpiInterfaceDecl iteration (aliased to vpiVirtualInterfaceVar in IEEE Std 1800-2012)
returned vpiRefObj objects in IEEE Std 1800-2005 and IEEE Std 1800-2009. This behavior has
been changed to produce vpiVirtualInterfaceVar objects in IEEE Std 1800-2012 in order to match
the aliased iteration type.
#### 36.12.2 VPI Mechanisms to deal with incompatibilities

In order to ease the transition to the latest VPI standard for older applications, capability shall be provided to
emulate the incompatible VPI behaviors where they conflict with the current standard. This allows older
VPI applications dependent on these behaviors to be run unmodified, as long as they are applied only to
designs (or portions of designs) with which they are compatible. This capability is intended only as an
interim measure to allow extra time for applications to be upgraded; it does not provide general emulation of
older behaviors for newer design constructs. For example, it does not allow IEEE 1364 applications to run
on portions of designs requiring IEEE 1800-level simulation capability.
As described in 36.12.2.1 and 36.12.2.2, two mechanisms to support this shall be provided, which can be
used in combination.
##### 36.12.2.1 Mechanism 1: Compile-based binding to a compatibility mode

This mechanism requires recompilation of the VPI application source code and is based on defining a
compiler symbol that binds a particular application to a particular compatibility mode. To use this scheme,
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
964
Copyright © 2018 IEEE. All rights reserved.
one of the following compiler symbols shall be defined prior to compilation of any of the standard VPI
include files in the application source code—either using a “#define” in the source code itself (setting it to
the numeric constant “1”), or defined on the C-compiler command-line:
VPI_COMPATIBILITY_VERSION_1364v1995
VPI_COMPATIBILITY_VERSION_1364v2001
VPI_COMPATIBILITY_VERSION_1364v2005
VPI_COMPATIBILITY_VERSION_1800v2005
VPI_COMPATIBILITY_VERSION_1800v2009
VPI_COMPATIBILITY_VERSION_1800v2012
VPI_COMPATIBILITY_VERSION_1800v2017
No more than one of these symbols shall be defined for a given application, and it shall be consistently
defined for all of its source code that can access any portion of VPI, including callback functions. This
allows all design information to be handled in the same way for a given mode across the entire application.
A compilation error will occur during the processing of vpi_user.h if more than one of the preceding
symbols is defined.
Example:
VPI source code file with a compatibility mode selected:
/* VPI application mytask */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#define VPI_COMPATIBILITY_VERSION_1364v2001 1
#include “vpi_user.h”
#include “sv_vpi_user.h”
#include “my_appl_header.h”
...
...
Alternatively, the same mode selection could be performed by defining the following option on the C-
compiler command line:
-DVPI_COMPATIBILITY_VERSION_1364v2001
When a mode is selected by one of the means above, C-preprocessor constructs in vpi_user.h cause the
following VPI functions to be redefined to mode-specific versions:
vpi_compare_objects
vpi_control
vpi_get
vpi_get_str
vpi_get_value
vpi_handle
vpi_handle_by_index
vpi_handle_by_multi_index
vpi_handle_by_name
vpi_handle_multi
vpi_iterate
vpi_put_value
vpi_register_cb
vpi_scan
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
965
Copyright © 2018 IEEE. All rights reserved.
For example, defining the mode symbol “VPI_COMPATIBILITY_VERSION_1364v2001” as shown above
will cause “vpi_handle” to be redefined as:
vpi_handle_1364v2001
This retargets all calls to “vpi_handle” in the recompiled application to this mode-specific variant,
achieving mode-compatible behavior. See “vpi_compatibility.h” (Annex L) for the complete set of
definitions.
##### 36.12.2.2 Mechanism 2: Selection of default VPI compatibility mode run by host simulator

A means to set the default VPI compatibility mode shall be made available by the simulation provider. This
shall determine the compatibility mode VPI behavior for all applications not using the compile-based
scheme detailed in Mechanism 1. Although VPI applications choosing this mechanism can be run without
modification or recompilation, only one such default mode shall be selectable for a given simulation run.
Additional applications requiring different modes in the same run-time simulation environment shall use the
compile-based mechanism to do so.
#### 36.12.3 Limitations of VPI compatibility mechanisms

When a VPI application uses the compatibility mode mechanism, the application user and application
provider should verify that the design or design partition to which the application is applied is consistent
with the mode, and does not include constructs that are only supported in other modes. If the design contains
unsupported constructs, the behavior of the VPI implementation is undefined. The extent of checking for
consistency between constructs and mode is left to the discretion of the VPI implementation.
In general, VPI users and application developers are strongly encouraged to update their applications to the
latest VPI version as soon as possible. The compatibility mode feature should be used only as a temporary
solution until such upgrades can be completed or become available. It should be expected that older modes
will be phased out as new versions of the standard become available.
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
