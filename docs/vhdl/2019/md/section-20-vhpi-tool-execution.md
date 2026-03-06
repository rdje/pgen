---
title: "Section 20: IEEE Standard for VHDL Language Reference Manual"
document: "VHDL Language Reference Manual"
standard: "IEEE 1076-2019"
domain: "VHDL"
section: "20"
source_txt: "section-20-vhpi-tool-execution.txt"
source_pdf: "/Users/richarddje/Documents/github/VHDL-LRM-IEEE-1076-2019.pdf"
---

# Section 20: IEEE Standard for VHDL Language Reference Manual

IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
414
Copyright © 2019 IEEE. All rights reserved.
20. VHPI tool execution
### 20.1 General

This clause describes the way in which foreign models and applications interact with a VHPI tool and the
way in which the tool executes VHDL and foreign models. A foreign model is a design entity whose
architecture is decorated with the 'FOREIGN attribute in the form described in this clause, or a subprogram
similarly decorated. A foreign application is a VHPI program that does not correspond to design entities or
subprograms declared in the VHDL model.
The VHPI supports various execution phases of a VHDL tool. Each phase is identified by a value of the
enumeration type vhpiPhaseT (see Annex B). A VHPI program determines the current phase of the
VHDL tool by calling the VHPI routine vhpi_get (see 23.10) supplying the value vhpiPhaseP as the
first parameter and NULL as the second parameter. The return value of vhpi_get is one of the values of
vhpiPhaseT.
In temporal order, the VHDL tool execution phases are:
a)
vhpiRegistrationPhase: indicates the tool has begun executing
b)
vhpiAnalysisPhase: The analysis of a design file is occurring
c)
vhpiElaborationPhase: The static elaboration of a design hierarchy is occurring
d)
vhpiInitializationPhase: The initialization of an elaborated design hierarchy is occurring
e)
vhpiSimulationPhase: The execution of an elaborated and initialized design hierarchy is
occurring
f)
vhpiSavePhase: The current state of a VHDL model is being saved for possible restart
g)
vhpiRestartPhase: A previously saved VHDL model is being restarted from the point of its
save
h)
vhpiResetPhase: A VHDL model is being restarted from the state it was in at the end of
initialization
i)
vhpiTerminationPhase: The tool is terminating
NOTE—If a tool does not support a given phase and a VHPI program attempts to register a callback with the callback
reason being the start or end of the phase, the vhpi_register_cb function raises an error indicating that the callback
reason is not implemented.
### 20.2 Registration phase

#### 20.2.1 General

The registration phase involves the following steps:
a)
Foreign models, applications, and libraries of foreign models are registered
b)
Each registered and enabled vhpiCbStartOfTool callback is executed
The registration phase is complete when all registered and enabled vhpiCbStartOfTool callbacks have
returned to the VHDL tool. During the registration phase, a call to vhpi_get(vhpiPhaseP, NULL)
returns vhpiRegistrationPhase.
Before a VHPI program can gain access to the internals of a VHDL tool, the program shall register itself
with the tool. Through either of two registration mechanisms described in 20.2.2 and 20.2.3, or through
decoration of a foreign model with the 'FOREIGN attribute in the form of a standard direct binding (see
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
415
Copyright © 2019 IEEE. All rights reserved.
20.2.4.3), the tool is supplied with the identity of one or more elaboration, execution, or registration
functions in a VHPI program. These functions shall be provided to the tool as entry points in one or more
object libraries. The format of the object libraries and whether the object libraries are statically or
dynamically bound to the tool are not specified by this standard. Each registration function shall be of the
type vhpiRegistrationFctT defined in Annex B.
Prior to the start of processing of any VHDL model by the tool, all registration functions of registered
libraries of foreign models and registration functions of selected registered foreign applications are invoked.
The manner in which registered foreign applications are selected is not defined by this standard. All such
calls to the registration functions shall terminate prior to the tool continuing its execution.
During the registration phase, the only parts of the information model defined by this standard that are
available are the objects of the tool and argv classes. It is an error if a registration function attempts to
access other parts of the information model during the registration phase.
A registration function may register callbacks. It is not possible for any VHPI callbacks (see Clause 21) to
occur prior to the completion of execution of all registration functions; in particular, registration shall be
complete before the vhpiCbStartOfTool callback (see 21.3.7.2) can occur.
A tool shall bind an elaboration, execution, or registration function prior to acquiring a pointer to the
function or calling the function. A tool is not required to bind such a function immediately upon registration.
It is an error if the tool cannot locate an entry point denoted by an elaboration or execution or registration
function name.
It is an error if a given foreign model, identified by a unique combination of object library name and model
name, is registered more than once by any of the mechanisms defined in this standard.
A foreign application may be registered multiple times with different registration functions. It is an error if a
given foreign application, identified by a unique combination of object library name and application name,
is registered more than once with the same registration function name by any of the mechanisms defined in
this standard.
A library of foreign models may be registered multiple times with different registration functions. It is an
error if a given library of foreign models, identified by an object library name, is registered more than once
with the same registration function name by any of the mechanisms defined in this standard.
The registration of a VHPI program with a given invocation of a tool does not persist beyond termination of
that invocation of the tool.
NOTE 1—A foreign model for which there is no corresponding VHDL architecture or subprogram decorated with the
'FOREIGN attribute may be registered. However, it will have no effect on the design since neither its elaboration
function (for a foreign architecture) nor its execution function can be invoked.
NOTE 2—The registration functions are the only entry points in an object library for a foreign application or library of
foreign models that need to be externally visible. Entry points for local elaboration and execution functions can be made
known to the tool as a consequence of resolving symbols referenced by the registration functions.
#### 20.2.2 Registration using a tabular registry

A tabular registry is a text file containing the registration information for foreign models and applications.
Any number of registry files can be passed to a VHDL tool; the mechanism for identifying the files to be
passed to a tool is not specified by this standard.
Each entry in the file defines the registration of one foreign model or application, or one library of foreign
models. Each entry occupies one line of the file and is a sequence of identifiers separated by one or more
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
416
Copyright © 2019 IEEE. All rights reserved.
space (SPACE or NBSP) characters. Blank lines, containing either no characters or only space characters,
and comments may also appear in the file. Space characters preceding an entry in the file or a comment are
ignored. Space characters following an entry in the file are ignored. A comment begins with the characters
“--” and continues to the end of the line containing the beginning of the comment.
```ebnf
tabular_registry_file ::= { tabular_registry_entry }
tabular_registry_entry ::=
        foreign_architecture_registry
      | foreign_subprogram_registry
      | foreign_application_registry
      | library_registry
foreign_architecture_registry ::=
      object_library_name model_name vhpiArchF elaboration_specifier execution_function_name
foreign_subprogram_registry ::=
        object_library_name model_name vhpiFuncF null execution_specifier
      | object_library_name model_name vhpiProcF null execution_specifier
foreign_application_registry ::=
      object_library_name application_name vhpiAppF registration_function_name null
library_registry ::=
      object_library_name null vhpiLibF registration_function_name null
object_library_name ::= C_identifier | extended_identifier
model_name ::= C_identifier | extended_identifier
application_name ::= C_identifier | extended_identifier
elaboration_specifier ::= elaboration_function_name | null
elaboration_function_name ::= C_identifier
execution_specifier ::= execution_function_name | null
execution_function_name ::= C_identifier
registration_function_name ::= C_identifier
```

An object library name denotes a logical name for an object library containing one or more entry points for
elaboration, execution, or registration functions. An object library name may or may not be case sensitive,
depending on the host environment. The mapping between an object library logical name and a host physical
object library is not defined by this standard. It is an error if the host system cannot locate the physical object
library identified by an object library name.
A model name is an identifier that, jointly with the object library name, shall uniquely identify a foreign
model. An application name is an identifier that, jointly with the object library name, shall uniquely identify
a foreign application.
An elaboration function name, execution function name, or registration function name denotes an entry
point in the library denoted by the immediately preceding object library name. An elaboration specifier of
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
417
Copyright © 2019 IEEE. All rights reserved.
null indicates that no elaboration function is required for the foreign model. An execution specifier of null
in a foreign subprogram registry is equivalent to an execution function name that is the same as the
immediately preceding model name.
A C identifier is formed from a contiguous sequence of graphical characters according to the rules for
forming identifiers in ISO/IEC 9899:2018. The reserved words in a tabular registry entry, vhpiArchF,
vhpiFuncF, vhpiProcF, vhpiAppF, vhpiLibF, and null, are case sensitive and shall be written using the
combination of uppercase and lowercase letters shown in this standard.
For each entry in the file, the foreign model, foreign application, or library of foreign models whose
registration is defined by the entry is registered with the tool reading the tabular registry.
NOTE 1—This standard does not define a default name or location for any tabular registry file.
NOTE 2—A model name or application name alone is not sufficient to uniquely identify a model or application.
Different models or applications may have the same model or application names, provided they can be distinguished by
different object library names.
NOTE 3—A C identifier that denotes a C function name is the same as the name of the C function defined in the C
source code. If an implementation modifies such a name during machine code generation, for example, by prefixing it
with an underline character, such modification is not reflected in the use of the name in a tabular registry entry.
Examples:
An example tabular registry:
-- registration of a foreign architecture:
myLib orgate vhpiArchF elab_or_gate init_or_gate
-- registration of a foreign function:
myLib myfunc vhpiFuncF null sim_myfunc
-- registration of a foreign application:
myApp appl vhpiAppF register_myapp null
-- registration of a library of models:
myLib null vhpiLibF register_lib null
An example registration function for the preceding table:
void register_lib() {
  for each model  the library
    vhpi_register_foreignf(...);
}
#### 20.2.3 Registration using registration functions

A VHPI program can register a foreign model or application using the vhpi_register_foreignf
function (see 23.30). The function shall be called during the registration phase of tool execution directly or
indirectly from a registration function of a previously registered foreign application or library of foreign
models.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
418
Copyright © 2019 IEEE. All rights reserved.
#### 20.2.4 Foreign attribute for foreign models

##### 20.2.4.1 General

The value of the 'FOREIGN attribute defined in package STANDARD decorating an architecture or a
subprogram may be a string of the form described in this subclause (20.2.4). The value of the attribute is
used to identify the VHPI program that implements the foreign model.
The value of the 'FOREIGN attribute for a foreign model is a sequence of identifiers separated by one or
more space (SPACE or NBSP) characters. Space characters, if any, preceding or following the sequence of
identifiers are ignored.
```ebnf
foreign_attribute_value ::=
      standard_direct_binding | standard_direct_binding
```

NOTE 1—The expression in an attribute specification for the 'FOREIGN attribute is required to be locally static (see
7.2). Nonetheless, analysis of a design unit containing a 'FOREIGN attribute specification does not require interpretation
of the value of the attribute at the time of analysis.
NOTE 2—An implementation may, as part of elaboration of a 'FOREIGN attribute specification whose value is of the
form described in this subclause (20.2.4), perform certain checks, for example, that the C library exists or that the for-
eign model implementation functions exists.
NOTE 3—The object library name for a foreign model need not be the same as the logical name of the VHDL library
containing the architecture or subprogram decorated with the 'FOREIGN attribute.
##### 20.2.4.2 Standard indirect binding

```ebnf
standard_indirect_binding ::=
      VHPI object_library_name model_name
```

The object library name and model name are described in 20.2.2. The reserved word VHPI in a standard
indirect binding is case sensitive and shall be written using uppercase letters.
A foreign attribute value in the form of a standard indirect binding specifies sufficient information for the
tool to register a foreign model, but not to identify elaboration or execution functions for the foreign model.
Identification of functions shall be specified separately using one of the mechanisms described in 20.2.2 or
20.2.3. A VHDL design entity or subprogram decorated with the 'FOREIGN attribute in the form of a
standard indirect binding is implemented by the elaboration and execution functions, as appropriate,
identified using the same object library name and model name as those that occur in the attribute value.
It is an error if, upon completion of registration, no execution function is specified corresponding to a
foreign model for which standard indirect binding is specified in the value of a 'FOREIGN attribute.
NOTE—It is permissible for no elaboration function to be specified corresponding to a foreign architecture for which
standard indirect binding is specified.
Example:
The following are analyzed into library foreignmodels:
package PACKSHELL is

component C_AND

port (P1, P2: in BIT; P3: out: BIT);
 end component;
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
419
Copyright © 2019 IEEE. All rights reserved.

procedure MYPROC (signal F1: out BIT; constant F2: in INTEGER);

attribute FOREIGN of MYPROC: procedure is "VHPI foreignC myCproc";

function MYFUNC (signal F1: in BIT) return INTEGER;

attribute FOREIGN of MYFUNC: function is "VHPI foreignC myCfunc";
end package PACKSHELL;
entity C_AND is

port (P1, P2: in BIT; P3: out: bit);
end C_AND;
architecture MY_C_GATE of C_AND is

attribute FOREIGN of MY_C_GATE: architecture is
            "VHPI foreignC myCarch";
begin
end architecture MY_C_GATE;
The following refer to declarations in the foreignmodels library:
library FOREIGNMODELS;
use FOREIGNMODELS.PACKSHELL.all;
entity TOP is
end TOP;
architecture MY_VHDL of TOP is

constant VAL: INTEGER:= 0;

signal S1, S2, S3: BIT;
begin

U1: C_AND (S1, S2, S3);

MYPROC (S1, VAL);

process (S1)

variable VA: INTEGER := VAL;

begin

VA := MYFUNC (S1);

end process;
end MY_VHDL;
##### 20.2.4.3 Standard direct binding

```ebnf
standard_direct_binding ::=
      standard_direct_architecture_binding | standard_direct_subprogram_binding
standard_direct_architecture_binding ::=
      VHPIDIRECT object_library_specifier elaboration_specifier execution_function_name
standard_direct_subprogram_binding ::=
      VHPIDIRECT object_library_specifier execution_specifier
object_library_specifier ::= object_library_path | null
object_library_path ::=
      graphic_character { graphic_character }
```

A foreign attribute value in the form of a standard direct binding specifies sufficient information for the tool
to register a foreign model and to identify elaboration and execution functions, as required, for the foreign
model. If the foreign model is a design entity, the standard direct binding shall take the form of a standard
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
420
Copyright © 2019 IEEE. All rights reserved.
direct architecture binding; otherwise, the standard direct binding shall take the form of a standard direct
subprogram binding.
An object library specifier denotes a physical name for an object library containing one or more entry points
for elaboration or execution functions.
An object library path may or may not be case sensitive, depending on the host environment. If a space
character (SPACE or NBSP) is to be used as one of the graphic characters of an object library path, it shall
be preceded by a backslash character (the combination of the backslash and space character counting as just
the space character). If a backslash is to be used as one of the graphic characters of an extended literal, it
shall be doubled (a doubled backslash counting as just one backslash). A host system interprets an object
library path in a manner not defined by this standard to locate a physical object library. It is an error if the
host system cannot locate the physical object library identified by an object library path.
An object library specifier of null indicates that a physical object library is to be determined in an
implementation defined manner. It is an error if an object library specifier of null is used and the host system
cannot locate the physical object library.
The reserved words VHPIDIRECT and null in a standard direct binding are case sensitive and shall be
written using uppercase and lowercase letters, respectively, as shown in this standard.
The elaboration specifier, execution specifier, and execution function name are described in 20.2.2. An
execution specifier of null in a standard direct subprogram binding is equivalent to an execution function
name that is the same as the designator of the subprogram decorated with the foreign attribute value using
the same combination of uppercase and lowercase letters that occur in the subprogram declaration for the
subprogram, if present, or the subprogram body otherwise.
NOTE—A host system may interpret an object library path by appending an implementation-dependent file-name
extension, such as “.so” or “.dll,” to derive a file pathname. It is recommended that a file-name extension in an object
library path be omitted so that an implementation can append an extension that is appropriate for the host environment.
### 20.3 Analysis phase

The analysis phase involves the following steps:
a)
Each registered and enabled vhpiCbStartOfAnalysis callback is executed.
b)
One or more design files are analyzed. The manner in which the design files to be analyzed are
specified to the tool is not specified by this standard.
c)
Each registered and enabled vhpiCbEndOfAnalysis callback is executed.
During
the
analysis
phase,
a
call
to
vhpi_get(vhpiPhaseP,
NULL)
returns
vhpiAnalysisPhase.
### 20.4 Elaboration phase

#### 20.4.1 General

The elaboration phase involves the following steps:
a)
Each registered and enabled vhpiCbStartOfElaboration callback is executed.
b)
The design hierarchy is elaborated, as described in 14.2 through 14.5. This may involve invocation
of elaboration functions, if any, for registered foreign architectures.
c)
Each registered and enabled vhpiCbEndOfElaboration callback is executed.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
421
Copyright © 2019 IEEE. All rights reserved.
During
the
elaboration
phase,
a
call
to
vhpi_get(vhpiPhaseP,
NULL)
returns
vhpiElaborationPhase.
An elaboration function shall conform to the rules for a callback function (see Clause 21). It is invoked by
the tool in the same way as a vhpiCbStartOfElaboration callback. The reason member of the
callback data structure passed to the elaboration function has the value vhpiCbStartOfElaboration.
The obj member of the callback data structure passed to the elaboration function contains a handle that
refers to an object of class designUnitInst that represents an instance of the foreign architecture
corresponding to the elaboration function. The value of the user_data member of the structure is not
specified by this standard.
It is an error if an elaboration function accesses the design hierarchy information model other than as
follows:
—
To access objects navigable from the object of class designUnitInst, representing the instance
of the foreign architecture body, passed to the elaboration function.
—
To use the vhpi_create function to create a foreign process, a driver, or a driver collection.
—
To use the vhpi_put_value function to set the initial value of an elaborated signal within the
instance of the corresponding foreign architecture or of an elaborated port of mode out, inout, or
buffer of the instance of the corresponding foreign architecture.
NOTE—At the time an elaboration function is invoked, the entire design hierarchy might not have been completely
elaborated. Thus, objects that ultimately will be accessible by navigating from the object passed to the elaboration
function might not yet exist.
#### 20.4.2 Dynamic elaboration

Dynamic elaboration of a foreign subprogram (see 14.6) involves invocation of the execution function of the
foreign subprogram. Dynamic elaboration of a foreign subprogram may occur during the elaboration,
initialization, or simulation phases of tool execution
An execution function of a foreign subprogram shall conform to the rules for a callback function (see
Clause 21). It is invoked by the tool in the same way as a vhpiCbStartOfSubpCall callback. The
reason member of the callback data structure passed to the elaboration function has the value
vhpiCbStartOfSubpCall. The obj member of the callback data structure passed to the elaboration
function contains a handle that refers to an object of class subpCall that represents an instance of the call
to the subprogram corresponding to the execution function. The value of the user_data member of the
structure is not specified by this standard.
An execution function of a foreign subprogram may obtain handles to objects representing the elaborated
formal parameters and their associated actual parameters. Such handles may become invalid upon
completion of the subprogram call. A VHPI program that relies upon such a handle remaining valid after the
execution function has returned is erroneous.
Parameters of a foreign subprogram implemented by a VHPI execution function are passed either by copy or
by references, as described in 4.2.2. An execution function may use the vhpi_get_value function to
read the value of a formal parameter of mode in or inout, and may use the vhpi_put_value function to
write the value of a formal parameter of mode out or inout. An execution function may use the
vhpi_schedule_transaction function to schedule a transaction on a driver for a formal signal
parameter of mode out or inout.
It is an error if the execution function for a foreign function does not provide a result for the function call
represented by the object referred to by the obj member of the callback data structure. The mechanism for
the execution to provide the result is described in 22.5.5.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
422
Copyright © 2019 IEEE. All rights reserved.
NOTE—An implementation may, in some cases, be able to statically elaborate parts of interface declarations in a
concurrent procedure call statement that invokes a foreign subprogram. In such cases, handles to objects representing the
elaborated declarations may remain valid between invocations of the subprogram.
### 20.5 Initialization phase

The initialization phase involves initializing the design hierarchy, as described in 14.7.5.2. This may involve
invocation of execution functions for registered foreign architectures. During the initialization phase, a call
to vhpi_get(vhpiPhaseP, NULL) returns vhpiInitializationPhase.
An execution function of a foreign architecture shall conform to the rules for a callback function (see
Clause 21). It is invoked by the tool in the same way as a vhpiCbStartOfInitialization callback.
The reason member of the callback data structure passed to the elaboration function has the value
vhpiCbStartOfInitialization. The obj member of the callback data structure passed to the
execution function contains a handle that refers to an object of class compInstStmt that represents an
instance of the foreign architecture corresponding to the execution function. The value of the user_data
member of the structure is not specified by this standard.
An execution function of a foreign architecture may access any part of the design hierarchy information
model.
NOTE—An execution function of a foreign architecture may register callbacks that occur in later phases of tool
execution. Memory allocated by the execution function may be referred to in the user_data member of callback data
structures used to register such callbacks.
### 20.6 Simulation phase

The simulation phase involves execution of simulation cycles, including execution of registered and enabled
vhpiCbStartOfSimulation and vhpiCbEndOfSimulation callbacks, as described in 14.7.5.3.
During
the
simulation
phase,
a
call
to
vhpi_get(vhpiPhaseP,
NULL)
returns
vhpiSimulationPhase.
### 20.7 Save phase

A tool may allow a user or a VHPI program to request that the current state of a VHDL model be saved for
possible restart. The manner by which such a request is made is not specified by this standard. If a VHPI
program makes such a request, the tool shall enter the save phase of tool execution either at the end of the
initialization phase, if the request was made before the end of the initialization phase, or at the end of the
current simulation cycle otherwise.
The save phase involves the following steps:
a)
The tool performs some actions, not specified by this standard, to save the current state of the VHDL
model, which includes the time of the next simulation cycle, Tn.
b)
Each registered and enabled vhpiCbStartOfSave callback is executed.
c)
Each registered and enabled vhpiCbEndOfSave callback is executed.
During the save phase, a call to vhpi_get(vhpiPhaseP, NULL) returns vhpiSavePhase.
A VHPI program may register vhpiCbStartOfSave and/or vhpiCbEndOfSave callbacks. During
execution of such callbacks, the VHPI program may use the vhpi_put_data (see 23.27) function to
include data as part of the saved state. The VHPI program may also register vhpiCbStartOfRestart
and/or vhpiCbEndOfRestart callbacks. During the save phase, the tool shall save registration of such
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
423
Copyright © 2019 IEEE. All rights reserved.
callbacks and restore the registration in such a manner that the callbacks can be invoked upon a subsequent
restart using the saved state.
NOTE 1—A tool may automatically save part or all of the state of a VHPI program. The flag bits of the value of the
AutomaticRestore property of the tool class specify the parts of the state that the tool automatically saves.
Depending on which flag bits are set, a VHPI program may need to save information about its handles, callbacks, and
private data using the vhpi_put_data function.
NOTE
2—A
VHPI
program
that
uses
vhpi_put_data
to
save
its
state
should
register
a
vhpiCbStartOfRestart or vhpiCbEndOfRestart callback and write to the user_data member of the
callback data structure the value of the identification number used to save state. The callback function, when invoked,
should read the identification number from the user_data member of the callback data structure it is passed and use
the id value in calls to the vhpi_get_data function to restore the state.
NOTE 3—If a user interrupts the save phase, through some implementation-defined means, the current state of the
model might not be correctly saved. It might not be possible to restart execution of the model using the saved state.
### 20.8 Restart phase

A tool may allow a user or a VHPI program to request that execution of a VHDL model be restarted from a
previously saved state. The manner by which such a request is made is not specified by this standard. If a
VHPI program makes such a request, the tool shall enter the restart phase of tool execution either at the end
of the initialization phase, if the request was made before the end of the initialization phase, or at the end of
the current simulation cycle otherwise.
The restart phase involves the following steps:
a)
The tool performs some actions, not specified by this standard, to restore the previously saved state
of the VHDL model, including the time of the next simulation cycle, Tn. The manner in which the
saved state is identified to the tool is not specified by this standard.
b)
Each registered and enabled vhpiCbStartOfRestart callback is executed.
c)
Each registered and enabled vhpiCbEndOfRestart callback is executed.
During the restart phase, a call to vhpi_get(vhpiPhaseP, NULL) returns vhpiRestartPhase.
After completion of the restart phase, the tool enters the simulation phase, commencing with a new
simulation cycle.
NOTE 1—A tool may automatically restore part or all of the state of a VHPI program. The flag bits of the value of the
AutomaticRestore property of the tool class specify the parts of the state that the tool automatically restores.
Depending on which flag bits are set, a VHPI program may need to reacquire handles, reregister callbacks, and restore
private data using the vhpi_get_data function.
NOTE 2—Upon entering the simulation phase from the restart phase, the tool does not execute any
vhpiCbStartOfSimulation callbacks.
### 20.9 Reset phase

A tool may allow a user or a VHPI program to request that execution of a VHDL model be reset to the
beginning of the initialization phase. The manner by which such a request is made is not specified by this
standard. If a VHPI program makes such a request, the tool shall enter the reset phase of tool execution
either at the end of the initialization phase, if the request was made before the end of the initialization phase,
or at the end of the current simulation cycle otherwise.
The reset phase involves the following steps:
a)
Each registered and enabled vhpiCbStartOfReset callback is executed.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
424
Copyright © 2019 IEEE. All rights reserved.
b)
All callbacks except vhpiCbEndOfReset callbacks are removed.
c)
The projected output waveform of each driver is reset to its initial contents.
d)
The current time, Tc, is reset to be 0 ns.
e)
Each registered and enabled vhpiCbEndOfReset callback is executed.
During the reset phase, a call to vhpi_get(vhpiPhaseP, NULL) returns vhpiResetPhase. After
completion of the reset phase, the tool enters the initialization phase.
A handle, acquired before the reset phase, that refers to a static object, remains valid during and after the
reset phase. A handle, acquired before the reset phase, that refers to a dynamic object, may become invalid
during or after the reset phase.
NOTE—A VHPI program that allows for reset should register a vhpiCbStartOfReset callback that releases
resources and saves information about callbacks that are to be reinstated after reset. It should also register a
vhpiCbEndOfReset callback that reregisters the callbacks that are to be reinstated.
### 20.10 Termination phase

The termination phase involves executing each registered and enabled vhpiCbEndOfTool callback.
When all such callbacks have returned to the tool, the tool may terminate. No further VHPI operations may
be called. During the termination phase, a call to vhpi_get(vhpiPhaseP,
NULL) returns
vhpiTerminationPhase.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
