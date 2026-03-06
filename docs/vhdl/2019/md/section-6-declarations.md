---
title: "Section 6: IEEE Standard for VHDL Language Reference Manual"
document: "VHDL Language Reference Manual"
standard: "IEEE 1076-2019"
domain: "VHDL"
section: "6"
source_txt: "section-6-declarations.txt"
source_pdf: "/Users/richarddje/Documents/github/VHDL-LRM-IEEE-1076-2019.pdf"
---

# Section 6: IEEE Standard for VHDL Language Reference Manual

IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
85
Copyright © 2019 IEEE. All rights reserved.
6. Declarations
### 6.1 General

The language defines several kinds of named entities that are declared explicitly or implicitly by
declarations. Each entity’s name is defined by the declaration, either as an identifier or as an operator
symbol or a character literal.
There are several forms of declaration. A declaration is one of the following:
—
A type declaration
—
A subtype declaration
—
An object declaration
—
An interface declaration
—
An alias declaration
—
An attribute declaration
—
A component declaration
—
A group template declaration
—
A group declaration
—
An entity declaration
—
A configuration declaration
—
A subprogram declaration
—
A subprogram instantiation declaration
—
A package declaration
—
A package instantiation declaration
—
A primary unit
—
An architecture body
—
A PSL property declaration
—
A PSL sequence declaration
—
An enumeration literal in an enumeration type definition
—
A primary unit declaration in a physical type definition
—
A secondary unit declaration in a physical type definition
—
An element declaration in a record type definition
—
A parameter specification in a loop statement or a for generate statement
—
An implicit label declaration
—
A logical name in a library clause, other than a library clause that appears within a context
declarative region
For each form of declaration, the language rules define a certain region of text called the scope of the
declaration (see 12.2). Each form of declaration associates an identifier, operator symbol, or character literal
with a named entity. The identifier, operator symbol, or character literal is called the designator of the
declaration. Only within its scope, there are places where it is possible to use the designator to refer to the
associated declared entity; these places are defined by the visibility rules (see 12.3). At such places the
designator is said to be a name of the entity; the name is said to denote the associated entity.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
86
Copyright © 2019 IEEE. All rights reserved.
This clause describes type and subtype declarations, the various kinds of object declarations, alias
declarations, attribute declarations, component declarations, and group and group template declarations. The
other kinds of declarations are described in Clause 3 and Clause 4.
A declaration takes effect through the process of elaboration. Elaboration of declarations is discussed in
Clause 14.
PSL verification units and declarations are described in IEEE Std 1850-2010. It is an error if a property
defined by a PSL property declaration does not conform to the rules for the simple subset of PSL.
### 6.2 Type declarations

A type declaration declares a type. Such a type is called an explicitly declared type.
```ebnf
type_declaration ::=
```

full_type_declaration
|   incomplete_type_declaration
```ebnf
full_type_declaration ::=
```

type identifier is type_definition ;
```ebnf
type_definition ::=
```

scalar_type_definition
|
composite_type_definition
|   access_type_definition
|   file_type_definition
|   protected_type_definition
|   protected_type_instantiation_definition
The types created by the elaboration of distinct type definitions are distinct types. Moreover, they are
distinct from formal generic types of entity declarations, component declarations, and uninstantiated
package and subprogram declarations. The elaboration of the type definition for a scalar type or a partially
constrained or fully constrained composite type creates both a base type and a subtype of the base type.
The simple name declared by a type declaration denotes the declared type, unless the type declaration
declares both a base type and a subtype of the base type, in which case the simple name denotes the subtype
and the base type is anonymous. A type is said to be anonymous if it has no simple name. For explanatory
purposes, this standard sometimes refers to an anonymous type by a pseudo-name, written in italics, and
uses such pseudo-names at places where the syntax normally requires an identifier.
NOTE 1—Two type definitions always define two distinct types, even if they are lexically identical. Thus, the type
definitions in the following two integer type declarations define distinct types:
type A is range 1 to 10;
type B is range 1 to 10;
This applies to type declarations for other classes of types as well.
NOTE 2—The various forms of type definition are described in Clause 5. Examples of type declarations are also given
in that clause.
### 6.3 Subtype declarations

A subtype declaration declares a subtype.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
87
Copyright © 2019 IEEE. All rights reserved.
```ebnf
subtype_declaration ::=
```

subtype identifier is subtype_indication ;
```ebnf
subtype_indication ::=
```

[ resolution_indication ] type_mark [ constraint ]
```ebnf
resolution_indication ::=
```

resolution_function_name
|
( element_resolution )
```ebnf
element_resolution ::= array_element_resolution | record_resolution
array_element_resolution ::= resolution_indication
record_resolution ::= record_element_resolution { , record_element_resolution }
record_element_resolution ::= record_element_simple_name resolution_indication
type_mark ::=
```

type_name
|   subtype_name
```ebnf
constraint ::=
```

range_constraint
|   array_constraint
|   record_constraint
```ebnf
element_constraint ::=
```

array_constraint
|   record_constraint
A type mark denotes a type or a subtype. If a type mark is the name of a type, the type mark denotes this type
and also the corresponding unconstrained subtype. The base type of a type mark is, by definition, the base
type of the type or subtype denoted by the type mark.
A subtype indication defines a subtype of the base type of the type mark.
A subtype defined by a subtype indication with a resolution indication is called a resolved subtype. All other
subtypes are unresolved subtypes. All types are unresolved.
A resolution indication is said to correspond to a subtype, and associates one or more resolution functions
with the subtype to which it corresponds or with subelement subtypes of the subtype to which it
corresponds. A resolution indication that appears in a subtype indication corresponds to the subtype defined
by the subtype indication. For that resolution indication, and any resolution indications nested within it, the
association of resolution functions is specified by the following rules, applied recursively:
—
If a resolution indication is in the form of a resolution function name, then the named resolution
function is associated with the subtype corresponding to the resolution indication.
—
If a resolution indication is in the form that contains an element resolution that is an array element
resolution, then the subtype corresponding to the resolution indication shall be an array subtype. The
array element resolution corresponds to the element subtype of the array subtype.
—
If a resolution indication is in the form that contains an element resolution that is a record resolution,
then the subtype corresponding to the resolution indication shall be a record subtype. For each record
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
88
Copyright © 2019 IEEE. All rights reserved.
element resolution in the record resolution, the record subtype shall have an element with the same
simple name as the record element simple name in the record element resolution, and the resolution
indication immediately following that record element simple name in the record element resolution
corresponds to the element subtype of the element with that simple name in the record subtype.
If a subtype indication includes a resolution indication that associates a resolution function name with a
subtype, then any signal declared to be of that subtype will be resolved, if necessary, by the named function
(see 4.6); for an overloaded function name, the meaning of the function name is determined by context (see
### 4.5 and 12.5). It is an error if the function does not meet the requirements of a resolution function (see 4.6).

The presence of a resolution function indication has no effect on the declarations of objects other than
signals or on the declarations of files, aliases, attributes, or other subtypes.
If the subtype indication does not include a constraint, the subtype is the same as that denoted by the type
mark. The condition imposed by a constraint is the condition obtained after evaluation of the expressions
and ranges forming the constraint. The rules defining compatibility are given for each form of constraint in
the appropriate clause. These rules are such that if a constraint is compatible with a subtype, then the
condition imposed by the constraint cannot contradict any condition already imposed by the subtype on its
values. An error occurs if any check of compatibility fails.
The direction of a discrete subtype indication is the same as the direction of the range constraint that appears
as the constraint of the subtype indication. If no constraint is present, and the type mark denotes a subtype,
the direction of the subtype indication is the same as that of the denoted subtype. If no constraint is present,
and the type mark denotes a type, the direction of the subtype indication is the same as that of the range used
to define the denoted type. The direction of a discrete subtype is the same as the direction of its subtype
indication.
A subtype indication denoting an access type, a file type, or a protected type shall not contain a resolution
function. Furthermore, the only allowable constraint on a subtype indication denoting an access type is an
array constraint (and then only if the designated type is an array type) or a record constraint (and then only if
the designated type is a record type).
A subtype indication denoting a subtype of a file type, a protected type, or a formal generic incomplete type
of an uninstantiated package or subprogram declaration shall not contain a constraint.
NOTE—A subtype declaration does not define a new type.
### 6.4 Objects

#### 6.4.1 General

An object is a named entity that contains (has) a value of a type. An object is one of the following:
—
An object declared by an object declaration (see 6.4.2)
—
A loop or generate parameter (see 10.10 and 11.9)
—
A formal parameter of a subprogram (see 4.2.2)
—
A formal port (see 6.5.6.3 and 11.2)
—
A formal generic constant (see 6.5.6.2 and 11.2)
—
A local port (see 6.8)
—
A local generic constant (see 6.8)
—
An implicit signal GUARD defined by the guard condition of a block statement (see 11.2)
In addition, the following are objects, but are not named entities:
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
89
Copyright © 2019 IEEE. All rights reserved.
—
An implicit signal defined by any of the predefined attributes 'DELAYED, 'STABLE, 'QUIET, and
'TRANSACTION (see 16.2)
—
An element or slice of another object (see 8.4, 8.5, and 8.6)
—
An object designated by a value of an access type (see 5.4.1)
There are four classes of objects: constants, signals, variables, and files. The variable class of objects also
has an additional subclass: shared variables. The class of an explicitly declared object is specified by the
reserved word that shall or may appear at the beginning of the declaration of that object. For a given object
of a composite type, each element of that object is itself an object of the same class and subclass, if any, as
the given object. The value of a composite object is the aggregation of the values of its subelements.
Objects declared by object declarations are available for use within blocks, processes, subprograms, or
packages. Loop and generate parameters are implicitly declared by the corresponding statement and are
available for use only within that statement. Other objects, declared by interface object declarations, create
channels for the communication of values between independent parts of a description.
#### 6.4.2 Object declarations

##### 6.4.2.1 General

An object declaration declares an object of a specified type. Such an object is called an explicitly declared
object.
```ebnf
object_declaration ::=
```

constant_declaration
|   signal_declaration
|   variable_declaration
|   file_declaration
An object declaration is called a single-object declaration if its identifier list has a single identifier; it is
called a multiple-object declaration if the identifier list has two or more identifiers. A multiple-object
declaration is equivalent to a sequence of the corresponding number of single-object declarations. For each
identifier of the list, the equivalent sequence has a single-object declaration formed by this identifier,
followed by a colon and by whatever appears at the right of the colon in the multiple-object declaration; the
equivalent sequence is in the same order as the identifier list.
A similar equivalence applies also for interface object declarations (see 6.5.2).
NOTE—The subelements of a composite declared object are not declared objects.
##### 6.4.2.2 Constant declarations

A constant declaration declares a constant of the specified type. Such a constant is an explicitly declared
constant.
```ebnf
constant_declaration ::= constant identifier_list : subtype_indication [ := conditional_expression ] ;
```

If the assignment symbol “:=” followed by an expression is present in a constant declaration, the expression
specifies the value of the constant; the type of the expression shall be that of the constant. The value of a
constant cannot be modified after the declaration is elaborated.
If the assignment symbol “:=” followed by an expression is not present in a constant declaration, then the
declaration declares a deferred constant. It is an error if such a constant declaration appears anywhere other
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
90
Copyright © 2019 IEEE. All rights reserved.
than in a package declaration. The corresponding full constant declaration, which defines the value of the
constant, shall appear in the body of the package (see 4.8).
Formal parameters of subprograms that are of mode in may be constants, and local and formal generics may
also be constants; the declarations of such objects are discussed in 6.5.2. A loop parameter is a constant
within the corresponding loop (see 10.10); similarly, a generate parameter is a constant within the
corresponding generate statement (see 11.9). A subelement or slice of a constant is a constant.
It is an error if a constant declaration declares a constant that is of a file type, an access type, a protected
type, or a composite type that has a subelement that is of a file, access or protected type.
NOTE 1—The subelements of a composite declared constant are not declared constants.
NOTE 2—A constant may be of a formal generic type. However, for an instance of the enclosing declaration that defines
the formal generic type, a check is required that the actual generic type is neither an access type nor contains a
subelement of an access type. Depending on the implementation, this check may be done during analysis of the
instantiation, or it may be deferred until the design hierarchy is elaborated.
Examples:
constant TOLER: DISTANCE := 1.5 nm;
constant PI: REAL := 3.141592;
constant CYCLE_TIME: TIME := 100 ns;
constant Propagation_Delay: DELAY_LENGTH;  -- A deferred constant.
##### 6.4.2.3 Signal declarations

A signal declaration declares a signal of the specified type. Such a signal is an explicitly declared signal.
```ebnf
signal_declaration ::=
```

signal identifier_list : subtype_indication [ signal_kind ] [ := conditional_expression ] ;
```ebnf
signal_kind ::=  register | bus
```

If a resolution indication appears in the subtype indication in the declaration of a signal or in the declaration
of the subtype used to declare the signal, then each resolution function in the subtype is associated
correspondingly with the declared signal or with a subelement of the declared signal. Such a signal
subelement is called a resolved signal.
If a signal kind appears in a signal declaration, then the signals so declared are guarded signals of the kind
indicated. For a guarded signal that is of a composite type, each subelement is likewise a guarded signal. For
a guarded signal that is of an array type, each slice (see 8.6) is likewise a guarded signal. A guarded signal
may be assigned values under the control of Boolean-valued guard conditions (or guards). When a given
guard becomes FALSE, the drivers of the corresponding guarded signals are implicitly assigned a null
transaction (see 10.5.2.2) to cause those drivers to turn off. A disconnection specification (see 7.4) is used to
specify the time required for those drivers to turn off.
If the signal declaration includes the assignment symbol followed by an expression, it shall be of the same
type as the signal. Such an expression is said to be a default expression. The default expression defines a
default value associated with the signal or, for a composite signal, with each scalar subelement thereof. For a
signal declared to be of a scalar subtype, the value of the default expression is the default value of the signal.
For a signal declared to be of a composite subtype, each scalar of the value of the default expression is the
default value of the corresponding subelement of the signal.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
91
Copyright © 2019 IEEE. All rights reserved.
In the absence of an explicit default expression, an implicit default value is assumed for a signal of a scalar
subtype or for each scalar subelement of a composite signal, each of which is itself a signal of a scalar
subtype. The implicit default value for a signal of a scalar subtype T is defined to be that given by T'LEFT.
It is an error if a signal declaration declares a signal that is of a file type, an access type, a protected type, or
a composite type having a subelement that is of a file type, an access type, or a protected type. It is also an
error if a guarded signal of a scalar type is neither a resolved signal nor a subelement of a resolved signal.
A signal may have one or more sources. For a signal of a scalar type, each source is either a driver (see
14.7.2) or an out, inout, buffer, or linkage port of a component instance or of a block statement with which
the signal is associated. For a signal of a composite type, each composite source is a collection of scalar
sources, one for each scalar subelement of the signal. It is an error if, after the elaboration of a description, a
signal has multiple sources and it is not a resolved signal. It is also an error if, after the elaboration of a
description, a resolved signal has more sources than the number of elements in the index range of the type of
the formal parameter of the resolution function associated with the resolved signal.
If a subelement or slice of a resolved signal of composite type is associated as an actual in (either in a
component instantiation statement, a block statement, or in a binding indication), and if the corresponding
formal is of mode out, inout, buffer, or linkage, then every scalar subelement of that signal shall be
associated exactly once with such a formal in the same port map aspect, and the collection of the
corresponding formal parts taken together constitute one source of the signal. If a resolved signal of
composite type is associated as an actual in a port map aspect, that is equivalent to each of its subelements
being associated in the same port map aspect.
If a subelement of a resolved signal of composite type has a driver in a given process, then every scalar
subelement of that signal shall have a driver in the same process, and the collection of all of those drivers
taken together constitute one source of the signal.
The default value associated with a scalar signal defines the value component of a transaction that is the
initial contents of each driver (if any) of that signal. The time component of the transaction is not defined,
but the transaction is understood to have already occurred by the start of simulation.
Examples:
signal S: STANDARD.BIT_VECTOR (1 to 10);
signal CLK1, CLK2: TIME;
signal OUTPUT: WIRED_OR MULTI_VALUED_LOGIC;
NOTE 1—The term signal is used in this standard to refer to objects declared either by signal declarations or by signal
port declarations (or to subelements, slices, or aliases of such objects). It also refers to the implicit signal GUARD (see
11.2) and to implicit signals defined by the predefined attributes 'DELAYED, 'STABLE, 'QUIET, and
'TRANSACTION. The term port is used to refer to objects declared by port declarations only.
NOTE 2—Signals are given initial values by initializing their drivers. The initial values of drivers are then propagated
through the corresponding net to determine the initial values of the signals that make up the net (see 14.7.3.4).
NOTE 3—The value of a signal is indirectly modified by a signal assignment statement (see 10.5); such assignments
affect the future values of the signal.
NOTE 4—The subelements of a composite, declared signal are not declared signals.
NOTE 5—A signal may be of a formal generic type. Depending on the implementation, various determinations and
checks may be done during analysis of an instance of the enclosing declaration that defines the formal generic type, or
they may be deferred until the design hierarchy is elaborated. These include: determining whether a signal or a
subelement of a signal is resolved, based on the actual generic subtype; determining the implicit default value; checking
that the actual generic type is neither an access type nor contains a subelement of an access type.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
92
Copyright © 2019 IEEE. All rights reserved.
Cross-references: Disconnection specifications, 7.4; disconnection statements, 11.7; guarded assignment,
11.7; guarded blocks, 11.2; guarded targets, 11.7; signal guard, 11.2.
##### 6.4.2.4 Variable declarations

A variable declaration declares a variable of the specified type. Such a variable is an explicitly declared
variable.
```ebnf
variable_declaration ::=
    [ shared ] variable identifier_list : subtype_indication [ generic_map_aspect ]
        [ := conditional_expression ] ;
```

When a generic map aspect is present, the subtype indication shall denote an uninstantiated protected type
declared in a protected type declaration. The generic map aspect, in this case, associates actuals with formal
generics (or member thereof) of the corresponding uninstantiated protected type.
Each variable declaration the subtype indication of which is an uninstantiated protected type is immediately
preceded in the same declaration region by an implicit declaration of the protected type instance defined by
the subtype indication and the generic map aspect, if any, of the variable declaration. The type of the
variables declared by the variable declaration is the implicitly declared protected type. The implicitly
declared protected type does not have a simple name.
It is an error if the subtype denotes an uninstantiated protected type and a formal generic does not get a value
through either association or a default value.
The variable declaration that includes the reserved word shared is a shared variable declaration. A shared
variable declaration declares a shared variable. Shared variables are a subclass of the variable class of
objects. The base type of the subtype indication of a shared variable declaration shall be a protected type or
a composite type with a subelement of a protected type. Variables declared immediately within entity
declarations, architecture bodies, blocks, and generate statements shall be shared variables. Variables
declared immediately within subprograms and processes shall not be shared variables. Variables declared
immediately within a package shall not be shared variables if the package is declared within a subprogram,
process, or protected type body; otherwise, the variables shall be shared variables. Variables declared
immediately within a protected type body shall not be shared variables. Variables that appear in protected
type bodies, other than within subprograms, represent shared data.
If a given variable declaration appears (directly or indirectly) within a protected type body, then the base
type denoted by the subtype indication of the variable declaration shall not be the protected type or a
composite with a subelement of the protected type defined by the protected type body.
If the variable declaration includes the assignment symbol followed by an expression, the expression
specifies an initial value for the declared variable; the type of the expression shall be that of the variable.
Such an expression is said to be an initial value expression. A variable declaration, whether it is a shared
variable declaration or not, the subtype indication of which denotes a protected type or a composite with a
subelement of a protected type shall not have an initial value expression (moreover, it shall not include the
immediately preceding assignment symbol).
If an initial value expression appears in the declaration of a variable, then the initial value of the variable is
determined by that expression each time the variable declaration is elaborated. In the absence of an initial
value expression, a default initial value applies. The default initial value for a variable of a scalar subtype T
is defined to be the value given by T'LEFT. The default initial value of a variable of a composite type is
defined to be the aggregate of the default initial values of all of its scalar subelements, each of which is itself
a variable of a scalar subtype. The default initial value of a variable of an access type is defined to be the
value null for that type.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
93
Copyright © 2019 IEEE. All rights reserved.
NOTE 1—The value of a variable that is not a shared variable is modified by a variable assignment statement (see 10.6);
such assignments take effect immediately.
NOTE 2—The variables declared within a given procedure persist until that procedure completes and returns to the
caller. For procedures that contain wait statements, a variable therefore persists from one point in simulation time to
another, and the value in the variable is thus maintained over time. For processes, which never complete, all variables
persist from the beginning of simulation until the end of simulation.
NOTE 3—The subelements of a composite, declared variable are not declared variables.
NOTE 4—Since the language provides mutual exclusion of accesses to shared data, but not the order of access to such
data by multiple processes in the same simulation cycle, the use of shared variables can be both non-portable and
non-deterministic. For example, consider the following architecture:
architecture UseSharedVariables of SomeEntity is
   subtype ShortRange is INTEGER range -1 to 1;
   type ShortRangeProtected is protected
      procedure Set (V: ShortRange);
      procedure Get (V: out ShortRange);
   end protected;
   type ShortRangeProtected is protected body
      variable Local: ShortRange := 0;
      procedure Set (V: ShortRange) is
      begin
         Local := V;
      end procedure Set;
      procedure Get (V: out ShortRange) is
      begin
         V := Local;
      end procedure Get;
   end protected body;
   shared variable ShortCounter: ShortRangeProtected;
begin
   PROC1: process
      variable V: ShortRange;
   begin
      ShortCounter.Get (V);
      ShortCounter.Set (V+1);
      wait;
   end process PROC1;
   PROC2: process
      variable V: ShortRange;
   begin
      ShortCounter.Get (V);
      ShortCounter.Set (V-1);
      wait;
   end process PROC2;
end architecture UseSharedVariables;
NOTE 5—In particular, the value of ShortCounter after the execution of both processes may not be 0.
NOTE 6—Variables that are not shared variables may have a subtype indication denoting a protected type or a
composite with a subelement of a protected type.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
94
Copyright © 2019 IEEE. All rights reserved.
NOTE 7—A variable, other than a shared variable, may be of a formal generic type. Depending on the implementation,
a default initial value may be determined during analysis of an instance of the enclosing declaration that defines the for-
mal generic type, or determination may be deferred until the design hierarchy is elaborated. A shared variable cannot be
of a formal generic type, since an actual generic type will not be a protected type or a composite with a subelement of a
protected type.
Examples:
architecture A of E is
-- See Clause 5.6.2, Clause 5.6.3, and Clause 5.6.4 for
-- related protected type declarations
    shared variable Counter: SharedCounter;
shared variable addend, augend, result: ComplexNumber;
shared variable ScoreBoard_slv : ScoreBoardPType_slv ;
shared variable ScoreBoard_int : ScoreBoardGenericPType generic map
( ExpectedType    => integer,
ActualType      => integer,
check           => std.standard."="
);
begin
process
        variable INDEX: INTEGER range 0 to 99 := 0;
          -- Initial value is determined by the initial value expression

        variable COUNT: POSITIVE;
          -- Initial value is POSITIVE'LEFT; that is 1

        variable MEMORY: BIT_MATRIX (0 to 7, 0 to 1023);
          -- Initial value is the aggregate of
          -- the initial values of each element

        variable bit_stack: VariableSizeBitArray;
    begin
      . . .
##### 6.4.2.5 File declarations

A file declaration declares a file of the specified type. Such a file is an explicitly declared file.
```ebnf
file_declaration ::=
    file identifier_list : subtype_indication [ file_open_information ] ;
file_open_information ::= [ open file_open_kind_expression ] is file_logical_name
file_logical_name ::=  string_expression
```

Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
95
Copyright © 2019 IEEE. All rights reserved.
The subtype indication of a file declaration shall define a file subtype or a composite containing a file
subtype. If the subtype indication is a composite containing a file subtype, then the file declaration shall not
have the file open information.
If file open information is included in a given file declaration, then the file declared by the declaration is
opened (see 5.5.2) with an implicit call to FILE_OPEN when the file declaration is elaborated (see 14.4.2.5).
This implicit call is to the FILE_OPEN procedure of the first form, and it associates the identifier with the
file parameter F, the file logical name with the External_Name parameter, and the file open kind expression
with the Open_Kind parameter. If a file open kind expression is not included in the file open information of
a given file declaration, then the default value of READ_MODE is used during elaboration of the file
declaration.
If file open information is not included in a given file declaration, then the file declared by the declaration is
not opened when the file declaration is elaborated.
The file logical name shall be an expression of predefined type STRING. The value of this expression is
interpreted as a logical name for a file in the host system environment. An implementation shall provide
some mechanism to associate a file logical name with a host-dependent file. Such a mechanism is not
defined by the language.
The file logical name identifies an external file in the host file system that is associated with the file object.
This association provides a mechanism for either importing data contained in an external file into the design
during simulation or exporting data generated during simulation to an external file.
If multiple file objects are associated with the same external file, and each file object has an access mode
that is read-only (see 5.5.2), then values read from each file object are read from the external file associated
with the file object. The language does not define the order in which such values are read from the external
file, nor does it define whether each value is read once or multiple times (once per file object).
The language does not define the order of and the relationship, if any, between values read from and written
to multiple file objects that are associated with the same external file. An implementation may restrict the
number of file objects that are associated at one time with a given external file.
If a formal subprogram parameter is of the class file, it shall be associated with an actual that has a file type
or a composite with a subelement of a file type.
Examples:
type IntegerFile is file of INTEGER;
file F1: IntegerFile;
   --  No implicit FILE_OPEN is performed during elaboration.
file F2: IntegerFile is "test.dat";
   --  At elaboration, an implicit call is performed:
   --  FILE_OPEN (F2, "test.dat");
   --  The OPEN_KIND parameter defaults to READ_MODE.
file F3: IntegerFile open WRITE_MODE is "test.dat";
   --  At elaboration, an implicit call is performed:
   --  FILE_OPEN (F3, "test.dat", WRITE_MODE);
NOTE 1—All file objects associated with the same external file should be of the same base type.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
96
Copyright © 2019 IEEE. All rights reserved.
NOTE 2—A file cannot be of a formal generic type, since an actual generic type will not be a file type or a composite
with a subelement of a file type.
### 6.5 Interface declarations

#### 6.5.1 General

An interface declaration is an interface object declaration, an interface type declaration, an interface
subprogram declaration, or an interface package declaration.
```ebnf
interface_declaration ::=
        interface_object_declaration
    |   interface_type_declaration
    |   interface_subprogram_declaration
    |   interface_package_declaration
```

#### 6.5.2 Interface object declarations

An interface object declaration declares an interface object of a specified type. Interface objects include
interface constants, interface signals, interface variables, and interface files.
```ebnf
interface_object_declaration ::=
        interface_constant_declaration
    |   interface_signal_declaration
    |   interface_variable_declaration
    |   interface_file_declaration
interface_constant_declaration ::=
    [ constant ] identifier_list : [ in ] interface_type_indication
                                                 [ := static_conditional_expression ]
interface_signal_declaration ::=
    [ signal ] identifier_list : signal_mode_indication
interface_variable_declaration ::=
    [ variable ] identifier_list : [ mode ] interface_type_indication
                                               [ := static_conditional_expression ]
interface_file_declaration ::=
    file identifier_list : subtype_indication
interface_type_indication ::=
```

subtype_indication | unspecified_type_indication
```ebnf
mode_indication ::=
        simple_mode_indication
   |    mode_view_indication
simple_mode_indication ::=
   [    mode ] interface_type_indication [ bus ] [ := static_conditional_expression ]
```

Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
97
Copyright © 2019 IEEE. All rights reserved.
```ebnf
mode ::= in | out | inout | buffer | linkage
mode_view_indication ::=
        record_mode_view_indication
    |   array_mode_view_indication
record_mode_view_indication ::=
    view mode_view_name [ of unresolved_record_subtype_indication ]
```

```ebnf
array_mode_view_indication ::=
    view ( mode_view_name ) [ of  unresolved_array_subtype_indication ]
mode_view_declaration ::=
    view identifier of unresolved_record_subtype_indication is
        { mode_view_element_definition }
    end view [ mode_view_simple_name ] ;
mode_view_element_definition ::=
    record_element_list : element_mode_indication ;
record_element_list ::=
    record_element_simple_name { , record_element_simple_name }
```

```ebnf
element_mode_indication ::=
        mode
    |   element_mode_view_indication
element_mode_view_indication ::=
        element_record_mode_view_indication
    |   element_array_mode_view_indication
element_record_mode_view_indication ::=
    view mode_view_name
```

```ebnf
element_array_mode_view_indication ::=
    view ( mode_view_name )
```

For an interface constant declaration, if no mode is explicitly given, mode in is assumed.
For an interface variable declaration, if no mode is explicitly given, mode in is assumed. For an interface
signal declaration, if a mode indication is a simple mode indication and no mode is explicitly given, mode in
is assumed.
For any named entity that has a formal port or parameter, the type of which is specified as an unspecified
type indication, there is an equivalent construct with a generic list and modified port or parameter list.
For each interface object declaration that contains an unspecified type declaration, there is a corresponding
anonymous formal generic type added to the end of the generic list. If a generic list does not exist, one is
created. The modified port or parameter list contains the same declarations as the original port or parameter
list, except that each unspecified type declaration is replaced by a reference to the corresponding unspecified
formal generic type. The unspecified formal generic type does not have a simple name.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
98
Copyright © 2019 IEEE. All rights reserved.
For an interface object declaration, the subtype indication of which is a formal generic type, the actual
generic type shall comply with the rules for subtype indication for the corresponding interface object.
For an interface constant declaration (other than a formal parameter of the predefined = or /= operator for an
access type) or an interface signal declaration, the subtype indication shall define a subtype that is neither a
file type, an access type, nor a protected type. Moreover, the subtype indication shall not denote a composite
type with a subelement that is of a file type, an access type, or a protected type.
For an interface file declaration, it is an error if the subtype indication does not denote a subtype of a file
type or a composite containing a file type.
If the simple mode indication of an interface signal declaration includes the reserved word bus, then the
signal declared by that interface declaration is a guarded signal of signal kind bus. It is an error if an
interface signal declaration includes both an unspecified type declaration and the reserved word bus.
If an interface constant or variable declaration or the simple mode indication of interface signal declaration
contains a “:=” symbol followed by an expression, the expression is said to be the default expression of the
interface object. The type of a default expression shall be that of the corresponding interface object. It is an
error if a default expression appears in an interface declaration or a simple mode indication of an interface
signal declaration and any of the following conditions hold:
—
The mode is linkage.
—
The interface object is a formal signal parameter.
—
The interface object is a formal variable parameter of mode other than in.
—
The subtype indication of the interface declaration denotes a protected type or a composite type with
a subelement of a protected type.
—
The type of the interface object is specified with an unspecified type indication.
In an interface signal declaration appearing in a port list, the default expression defines the default value(s)
associated with the interface signal or its subelements. In the absence of a default expression, an implicit
default value is assumed for the signal or for each scalar subelement, as defined for signal declarations (see
6.4.2.3). The value, whether implicitly or explicitly provided, is used to determine the initial contents of
drivers, if any, of the interface signal as specified for signal declarations.
In an interface signal declaration, a mode view indication allows each subelement of the composite to have a
different mode. For an interface signal declaration with a mode view indication there is an equivalent
interface signal declaration for each subelement of the composite whose mode corresponds to the mode
specified in the mode view indication and whose type or subtype corresponds to the type or subtype defined
in the composite.
For an array mode view indication the type or subtype of the mode view shall be compatible with the
element type or subtype of the subtype indication.
For a record mode view indication:
—
If the optional subtype indication is not present, an implicit subtype indication is determined from
the corresponding mode view declaration.
—
If the optional subtype indication is present, it specifies the subtype of the interface signal
declaration and it shall be compatible with the type or subtype in the corresponding mode view
declaration.
For an element mode view indication, the type or subtype is determined from the corresponding record
element. For an element array mode view indication, the element type or subtype of each corresponding
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
99
Copyright © 2019 IEEE. All rights reserved.
record element shall be compatible with the type or subtype of the mode view. For an element record mode
view indication, the type or subtype of each corresponding record element shall be compatible with the type
or subtype of the mode view.
A mode view declaration declares a mode view for a composite type or subtype. A mode view specifies a
mode for each subelement of a composite type or subtype. Mode views may be used in an interface signal
declaration or within another mode view declaration.
In a mode view declaration, it is an error if either a resolution function appears in the subtype indication or
the subtype is a resolved subtype. However, the elements of a composite may be resolved subtypes.
The subtype indication of a mode view declaration shall denote an unresolved record type or subtype. For
each record element simple name of the type or subtype, there shall be a record element simple name in the
mode view declaration with the same simple name.
It is an error if the mode of an element mode indication is linkage.
An interface object provides a channel of communication between the environment and a particular portion
of a description. The value of an interface object may be determined by the value of an associated object or
expression in the environment; similarly, the value of an object in the environment may be determined by
the value of an associated interface object. The manner in which such associations are made is described in
6.5.7.
The value of an object is said to be read when one of the following conditions is satisfied:
—
When the object is evaluated, and also (indirectly) when the object is associated with an interface
object of the modes in, inout, or linkage.
—
When the object is a signal and a name denoting the object appears in a sensitivity list in a wait
statement or a process statement.
—
When the object is a signal and the value of any of its predefined attributes 'STABLE, 'QUIET,
'DELAYED, 'TRANSACTION, 'EVENT, 'ACTIVE, 'LAST_EVENT, 'LAST_ACTIVE, or
'LAST_VALUE is read.
—
When one of its subelements is read.
—
When the object is a file and a READ, FILE_STATE, FILE_MODE, FILE_POSITION,
FILE_SIZE, or FILE_CANSEEK operation is performed on the file.
—
When the object is a file of type STD.TEXTIO.TEXT and the procedure STD.TEXTIO.READLINE
is called with the given object associated with the formal parameter F of the given procedure.
The value of an object is said to be updated when one of the following conditions is satisfied:
—
When it is the target of an assignment, and also (indirectly) when the object is associated with an
interface object of the modes out, buffer, inout, or linkage.
—
When a VHPI information model object representing the given object is updated using a call to the
function vhpi_put_value.
—
When the object is a signal and the vhpi_schedule_transaction function is used to
schedule a transaction on a driver of the signal.
—
When one of its subelements is updated.
—
When the object is a file and a WRITE, FLUSH, FILE_REWIND, FILE_SEEK, or
FILE_TRUNCATE operation is performed on the file.
—
When
the
object
is
a
file
of
type
STD.TEXTIO.TEXT
and
the
procedure
STD.TEXTIO.WRITELINE is called with the given object associated with the formal parameter F
of the given procedure.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
100
Copyright © 2019 IEEE. All rights reserved.
It is an error if an object other than a signal, variable, or file object is updated.
The mode or mode view of an interface signal, or an interface variable or constant object has one of the
following modes:
—
in. The value of the interface object is allowed to be read, but it shall not be updated by a simple
waveform assignment, a conditional waveform assignment, a selected waveform assignment, a
concurrent signal assignment, or a variable assignment. Reading an attribute of the interface object is
allowed, unless the interface object is a signal parameter and the attribute is one of 'STABLE,
'QUIET, 'DELAYED, 'TRANSACTION, 'DRIVING, or 'DRIVING_VALUE.
—
out. The value of the interface object is allowed to be updated and, provided it is not a signal
parameter, read. Reading the attributes of the interface object is allowed, unless the interface object
is a signal parameter and the attribute is one of 'STABLE, 'QUIET, 'DELAYED, 'TRANSACTION,
'EVENT, 'ACTIVE, 'LAST_EVENT, 'LAST_ACTIVE, or 'LAST_VALUE.
—
inout or buffer. Reading and updating the value of the interface object is allowed. Reading the
attributes of the interface object, other than the attributes 'STABLE, 'QUIET, 'DELAYED, and
'TRANSACTION of a signal parameter, is also permitted.
—
linkage. Reading and updating the value of the interface object is allowed, but only by appearing as
an actual corresponding to an interface object of mode linkage. No other reading or updating is
permitted.
NOTE 1—A subprogram parameter that is of file type or a composite containing a file type will be declared as a file
parameter.
NOTE 2—Since shared variables are a subclass of variables, a shared variable may be associated as an actual with a
formal of class variable.
NOTE 3—Signal ports of mode linkage are used in the Boundary Scan Description Language (see IEEE Std 1149.1™-
## 2013 [B10]).10

NOTE 4—Interface file objects do not have modes.
NOTE 5—The driving value of a signal port that has no source is the default value of the port (see 14.7.3.2).
NOTE 6—A signal port of mode in may be updated by a force assignment, a release assignment, or a call to
vhpi_put_value   A formal parameter of mode in may not be updated by a call to vhpi_put_value (see 22.5.1).
Example:
-- An entity whose type of the formal signal port A is defined by an
-- unspecified type declaration.
entity E is
  port (
    A : type is private;         -- any type
    B : type is <>               -- a scalar type
  );
-- The equivalent entity with a formal generic type (here designated
-- as anonymous for clarity) and the signal port A whose type is
-- designated by the unnamed formal generic type.
entity E is
  generic (
    type Anonymous1 is private;  -- any type
10The numbers in brackets correspond to those of the bibliography in Annex J.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
101
Copyright © 2019 IEEE. All rights reserved.
    type Anonymous2 is <>        -- a scalar type
  )
  port (
    A : Anonymous1 ;
    B : Anonymous2
  );
#### 6.5.3 Interface type declarations

##### 6.5.3.1 General

An interface type declaration declares an interface type that appears as a formal generic of a generic clause.
```ebnf
interface_type_declaration ::=
 type identifier [ is incomplete_type_definition ]
```

An interface type provides a means for the environment to determine a type to be used for objects in a
particular portion of a description. The set of values and applicable operations for an interface type may be
determined by an associated subtype in the environment. The manner in which such associations are made is
described in 6.5.7. A generic type declared by an interface type declaration without an incomplete type
definition is called an unclassified generic type, otherwise it is called a classified generic type. An
unclassified type denotes a private incomplete type. A classified generic type is any interface type
declaration except a private incomplete type.
##### 6.5.3.2 Array interface type declaration

A formal array type and the associated actual array type shall both be constrained or both be unconstrained.
Both shall have the same dimensionality, the same index types in each dimension, and the same element
types. For a formal constrained array type, the index constraint shall be specified in the form of a type mark,
and the actual array type shall have the same index range as the formal array type.
Example:
package P1 is
generic (
type element_type is private;
-- any type
type index_type is (<>);
-- a discrete type
-- an array type
type array_type is array(index_type) of element_type
);
end package;
entity E is
end entity;
architecture A of E is
package I1 is new P1
generic map (
element_type => bit,
index_type
=> natural,
array_type
=> bit_vector
);
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
102
Copyright © 2019 IEEE. All rights reserved.
begin
end architecture;
If the array index incomplete type is specified with an unspecified type indication, then an implicit formal
unspecified type with the same incomplete type definition is declared immediately before the array interface
type declaration. If the element incomplete subtype indication is specified with an unspecified type
indication, then an implicit formal unspecified type with the same incomplete type definition is declared
immediately before the array interface type declaration.
Example:
package P2 is
    generic (
-- type anonymous is (<>);
-- implicitly declared anonymous generic discrete type
-- type anonymous is private;
-- implicitly declared anonymous unclassified generic type
 type array_type is array(type is (<>)) of type is private
  );
-- example usage of type aliases to create shorter names
 alias index_type   is array_type'INDEX;
  -- alias the implicit type with a name
  alias element_type is array_type'ELEMENT;
  -- alias the implicit type with a name
end package;
architecture A of E is
  package I2 is new P2
    generic map (
      -- anonymous => bit_vector'INDEX,

-- implicitly associated; see Clause 6.5.7.2
      -- anonymous => bit_vector'ELEMENT,

-- implicitly associated; see Clause 6.5.7.2
      array_type =>   bit_vector
    );
begin
end architecture;
##### 6.5.3.3 Access interface type declaration

A formal access type and the associated actual access type shall both have the same designated type.
Example:
package P1 is
generic (
type designated_subtype;                       -- any type
type access_type is access designated_subtype  -- an access type
    );
end package;
entity E is
end entity;
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
103
Copyright © 2019 IEEE. All rights reserved.
architecture A of E is
    package I1 is new P1
    generic map (
        designated_subtype => string,
        access_type        => line
    );
begin
end architecture;
If the access incomplete subtype indication is specified with an unspecified type indication, then an implicit
formal unspecified type with the same incomplete type definition is declared immediately before the array
interface type declaration.
Example:
package P2 is
generic (
-- type anonymous is private;
-- implicitly declared unclassified generic type
type access_type is access type is private
    );
-- example usage of type aliases to create shorter names
   alias designated_subtype is access_type'DESIGNATED_SUBTYPE;
-- alias the implicit type with a name
end package;
architecture A of E is
    package I2 is new P2
    generic map (
        -- anonymous => line'DESIGNATED_SUBTYPE,
        -- implicitly associated; see Clause 6.5.7.2
        access_type => line
    );
begin
end architecture;
##### 6.5.3.4 File interface type declaration

A formal file type and the associated actual file type shall both have the same designated type.
Example:
package P1 is
    generic (
        type designated_subtype;                      -- any type
        type file_type is file of designated_subtype  -- a file type
    );
end package;
entity E is
end entity;
architecture A of E is
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
104
Copyright © 2019 IEEE. All rights reserved.
    package I1 is new P1
    generic map (
        designated_subtype => string,
        file_type          => line
    );
begin
end architecture;
If the file incomplete type mark is specified with an unspecified type indication, then an implicit formal
unspecified type with the same incomplete type definition is declared immediately before the array interface
type declaration.
Example:
package P2 is
    generic (
    -- type anonymous is private;
    -- implicitly declared unclassified generic type
    type file_type is file of type is private
    );
-- example usage of type aliases to create shorter names
    alias designated_subtype is file_type'DESIGNATED_SUBTYPE;
-- alias the implicit type with a name
end package;
architecture A of E is
    package I2 is new P2
        generic map (
        -- anonymous => line'DESIGNATED_SUBTYPE,
        -- implicitly associated; see Clause 6.5.7.2
        file_type => line
    );
begin
end architecture;
NOTE—A type mark in a file interface type declaration may be a formal generic type or have a subelement of a formal
generic type. However, for an instance of the enclosing declaration that defines the formal generic type, a check is
required that the actual generic type is neither an access type, protected type, or file type nor contains a subelement of an
access type, protected type, or file type. Depending on the implementation, this check may be done during analysis of the
instantiation, or it may be deferred until the design hierarchy is elaborated.
#### 6.5.4 Interface subprogram declarations

An interface subprogram declaration declares an interface subprogram that appears in a generic clause.
```ebnf
interface_subprogram_declaration ::=
    interface_subprogram_specification [ is interface_subprogram_default ]
interface_subprogram_specification ::=
    interface_procedure_specification | interface_function_specification
interface_procedure_specification ::=
    procedure designator
```

Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
105
Copyright © 2019 IEEE. All rights reserved.
        [ [ parameter ] ( formal_parameter_list ) ]
```ebnf
interface_function_specification ::=
    [ pure | impure ] function designator
        [ [ parameter ] ( formal_parameter_list ) ] return type_mark
interface_subprogram_default ::= subprogram_name | <>
```

An interface subprogram provides a means for the environment to determine a subprogram to be called in a
particular portion of a description by associating an actual subprogram with the formal interface
subprogram. The manner in which such associations are made is described in 6.5.7.
If an interface subprogram declaration contains an interface subprogram default in the form of a subprogram
name, the subprogram name shall denote a subprogram, and the denoted subprogram and the interface
subprogram shall have conforming profiles (see 4.10).
Within a construct that has an interface subprogram declaration, but does not map it, the name of the given
interface subprogram denotes an undefined subprogram declaration and body. Within a construct that maps
a given interface subprogram declaration, the name of the given interface subprogram denotes the
subprogram specified as the corresponding actual in a generic association list.
#### 6.5.5 Interface package declarations

An interface package declaration declares an interface package that appears in a generic clause.
```ebnf
interface_package_declaration ::=
    package identifier is new uninstantiated_package_name interface_package_generic_map_aspect
interface_package_generic_map_aspect ::=
        generic_map_aspect
    |   generic map ( <> )
    |   generic map ( default )
```

An interface package provides a means for the environment to determine an instance of an uninstantiated
package to be visible in a particular portion of a description by associating an actual instantiated package
with the formal interface package. The manner in which such associations are made is described in 6.5.7.
The uninstantiated package name shall denote an uninstantiated package declared in a package declaration.
The interface package generic map aspect specifies the allowable actual generics of the instantiated package
associated with the formal generic package (see 6.5.7.2), as follows:
—
If the interface package generic map aspect is in the form of a generic map aspect, then the
corresponding actual instantiated package shall have matching actual generics. Matching actual
generics are described in 6.5.7.2.
—
If the interface package generic map aspect is in the form that includes the box (<>) symbol, then the
corresponding actual instantiated package may have any actual generics.
—
If the interface package generic map aspect is in the form that includes the reserved word default,
then every generic of the uninstantiated package shall be either a generic constant with a default
expression or a generic subprogram with an interface subprogram default. The interface package
generic map aspect is equivalent to an implicit interface package generic map aspect containing a
generic map aspect in which each generic of the uninstantiated package is associated with the
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
106
Copyright © 2019 IEEE. All rights reserved.
corresponding default expression or subprogram name implied by the interface subprogram default.
The subprogram implied by an interface subprogram default in the form of a box (<>) symbol is a
subprogram directly visible at the place of the formal generic package declaration.
Within a construct that has an interface subprogram declaration, but does not map it, the name of the given
interface subprogram denotes an undefined subprogram declaration and body. Within a construct that maps
a given interface subprogram declaration, the name of the given interface subprogram denotes the
subprogram specified as the corresponding actual in a generic association list.
#### 6.5.6 Interface lists

##### 6.5.6.1 General

An interface list contains interface declarations.
```ebnf
interface_list ::=
```

interface_element { ; interface_element } [ ; ]
```ebnf
interface_element ::=  interface_declaration
```

A generic interface list consists entirely of interface constant declarations, interface type declarations,
interface subprogram declarations, and interface package declarations. A port interface list consists entirely
of interface signal declarations and interface variable declarations. A parameter interface list may contain
interface constant declarations, interface signal declarations, interface variable declarations, interface file
declarations, or any combination thereof.
A name that denotes an interface declaration in an interface list may appear in a subsequent interface
declaration within the same interface list.
NOTE—The rule mentioned in the previous paragraph makes the following interface lists legal:
entity E is
generic (G1: INTEGER; G2: INTEGER := G1; G3, G4, G5, G6: INTEGER);
port (P1:STRING(G3 to G4); P2: STRING(P1'RANGE); P3: P1'SUBTYPE);
procedure X(Y1, Y2: INTEGER; Y3: INTEGER; Y3 INTEGER range Y1 to Y2;

Y4: Y1'SUBTYPE);
end E;
##### 6.5.6.2 Generic clauses

Generics provide a channel for information to be communicated to a block, a package, a subprogram, or a
protected type from its environment. The following applies to external blocks defined by design entities, to
internal blocks defined by block statements, and to packages, subprograms, and protected types.
```ebnf
generic_clause ::=
    generic ( generic_list ) ;
generic_list ::=  generic_interface_list
```

The generics of a block, a package, a subprogram, or a protected type are defined by a generic interface list.
Each interface element in such a generic interface list declares a formal generic.
The value of a formal generic constant may be specified by the corresponding actual in a generic association
list. If no such actual is specified for a given formal generic constant or subelement or slice thereof (either
because the formal generic is unassociated or because the actual is open), and if a default expression is
specified for that generic constant or subelement or slice thereof, the value of this expression is the value of
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
107
Copyright © 2019 IEEE. All rights reserved.
the generic constant or subelement or slice thereof. It is an error if no actual is specified for a given formal
generic constant and no default expression is present in the corresponding interface element.
The subtype denoted by a generic type is specified by the corresponding actual in a generic association list.
It is an error if no such actual is specified for a given formal generic type (either because the formal generic
is unassociated or because the actual is open).
The subprogram denoted by a generic subprogram may be specified by the corresponding actual in a generic
association list. If no such actual is specified for a given formal generic subprogram (either because the
formal generic is unassociated or because the actual is open), and if an interface subprogram default is
specified for that generic, the subprogram denoted by the generic is determined as follows:
—
If the interface subprogram default is in the form of a subprogram name, then the subprogram
denoted by the generic is the subprogram denoted by the subprogram name.
—
If the interface subprogram default is in the form of a box (<>) symbol, then there shall be a
subprogram directly visible at the place of the generic association list that has the same designator as
the formal and that has a conforming profile to that of the formal; the subprogram denoted by the
generic is the directly visible subprogram.
It is an error if no actual is specified for a given formal generic subprogram and no interface subprogram
default is present in the corresponding interface element. It is an error if the actual subprogram, whether
explicitly associated or associated by default, is impure and the formal generic subprogram is pure.
A call to a formal generic subprogram uses the parameter names and default expressions defined by the
declaration of the formal generic subprogram. Subtype checks and conversions for the association of actual
parameters with formal parameters and for the execution of a return statement from the actual subprogram
use the subtypes defined by the declaration of the actual subprogram.
The instantiated package denoted by a generic package is specified by the corresponding actual in a generic
association list. It is an error if no such actual is specified for a given formal generic package (either because
the formal generic is unassociated or because the actual is open).
Generics may be used to control structural, dataflow, or behavioral characteristics of a block, a package, or a
subprogram, a protected type or may simply be used as documentation. In particular, generics may be used
to specify the size of ports; the number of subcomponents within a block; the timing characteristics of a
block; or even the physical characteristics of a design such as temperature, capacitance, or location.
##### 6.5.6.3 Port clauses

Ports provide channels for dynamic communication between a block and its environment. The following
applies to both external blocks defined by design entities and to internal blocks defined by block statements,
including those equivalent to component instantiation statements and generate statements (see 6.6.1).
```ebnf
port_clause ::=
    port ( port_list ) ;
port_list ::=  port_interface_list
```

The ports of a block are defined by a port interface list. Each interface element in the port interface list
declares a formal port. A formal port shall have an object class that is either signal or variable. A formal
signal port is a formal port the object class of which is signal. A formal variable port is a formal port the
object class of which is variable. If a formal port does not explicitly specify the object class, signal is
assumed. A formal variable port shall either be of a protected type or a composite type with a subelement of
a protected type. Its mode shall be inout.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
108
Copyright © 2019 IEEE. All rights reserved.
To communicate with other blocks, the ports of a block can be associated with signals or shared variables in
the environment in which the block is used. Moreover, signal ports of a block may be associated with an
expression in order to provide these ports with constant driving values or with values derived from signals
and other ports; such ports shall be of mode in. A formal signal port is itself a signal (see 6.4.2.3), and thus,
may be associated as an actual with a formal signal port of an inner block. A formal variable port is a
reference to a shared variable, and may be associated as an actual with a formal variable port of an inner
block. The port, signal, shared variable, or expression associated with a given formal port is called the actual
corresponding to the formal port (see 6.5.7). The actual, if a port, signal, or shared variable shall be denoted
by a static name (see 8.1).
If a formal signal port of mode in is associated with an expression that is not globally static (see 9.4.3) and
the formal is of an unconstrained or partially constrained composite type requiring determination of index
ranges from the actual according to the rules of 5.3.2.2, then the expression shall be one of the following:
—
The name of an object whose subtype is globally static
—
An indexed name whose prefix is one of the members of this list
—
A slice name whose prefix is one of the members of this list and whose discrete range is a globally
static discrete range
—
An aggregate, provided all choices are locally static and all expressions in element associations are
expressions described in this list
—
A function call whose return type mark denotes a globally static subtype
—
A qualified expression or type conversion whose type mark denotes a globally static subtype
—
An expression described in this list and enclosed in parentheses
If the actual part of a given association element for a formal signal port of a block is the reserved word
inertial followed by an expression, or is an expression that is not globally static, then the given association
element is equivalent to association of the port with an anonymous signal implicitly declared in the
declarative region that immediately encloses the block. The signal has the same subtype as the formal signal
port and is the target of an implicit concurrent signal assignment statement of the form
anonymous <= E;
where E is the expression in the actual part of the given association element. The concurrent signal
assignment statement occurs in the same statement part as the block.
After a given description is completely elaborated (see Clause 14), if a formal signal port is associated with
an actual that is itself a port, then the following restrictions apply depending upon the mode (see 6.5.2), if
any, of the formal signal port:
a)
For a formal signal port of mode in the associated actual shall be a port of mode in, out, inout, or
buffer. This restriction applies both to an actual that is associated as a name in the actual part of an
association element and to an actual that is associated as part of an expression in the actual part of an
association element.
b)
For a formal signal port of mode out, the associated actual shall be a port of mode out, inout, or
buffer.
c)
For a formal signal port of mode inout, the associated actual shall be a port of mode out, inout, or
buffer.
d)
For a formal signal port of mode buffer, the associated actual shall be a port of mode out, inout, or
buffer.
e)
For a formal signal port of mode linkage, the associated actual may be a port of any mode.
The actual associated with a formal variable port shall either be a shared variable or a formal variable port of
another design entity, and its type shall be a protected type or a composite type with a subelement of a
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
109
Copyright © 2019 IEEE. All rights reserved.
protected type. The association of an actual with a formal variable port results in the actual passing a
reference to the formal.
A formal signal port, or slice thereof, that is associated with an actual signal port, signal, or expression is
said to be connected. A formal signal port, or slice thereof, that is associated with the reserved word open is
said to be unconnected. It is an error if a formal signal port, or slice thereof, of mode in is either unconnected
or unassociated (see 6.5.7.3) unless its declaration includes a default expression (see 6.5.2). It is an error if a
formal signal port, or slice thereof, of mode other than in is either unconnected or unassociated and its type
is an unconstrained or partially constrained composite type.
A formal variable port, or slice thereof, that is associated with an actual variable port or a shared variable is
said to be connected. A formal variable port, or slice thereof, that is associated with the reserved word open
is said to be unconnected. It is an error if a formal variable port, or slice thereof, is either unconnected or
unassociated.
#### 6.5.7 Association lists

##### 6.5.7.1 General

An association list, other than one appearing in an interface package generic map aspect (see 6.5.5),
establishes correspondences between formal or local generic, port, or parameter names on the one hand and
local or actual names, expressions, subtypes, subprograms, or packages on the other.
```ebnf
association_list ::=
    association_element { , association_element }
association_element ::=
    [ formal_part => ] actual_part
formal_part ::=
        formal_designator
    |   function_name ( formal_designator )
    |   type_mark ( formal_designator )
formal_designator ::=
        generic_name [ signature ]
    |    port_name
    |    parameter_name
actual_part ::=
        actual_designator
    |   function_name ( actual_designator )
    |   type_mark ( actual_designator )
actual_designator ::=
        [ inertial ] conditional_expression
    |    signal_name
    |   variable_name
    |   file_name
    |   subtype_indication
    |   subprogram_name
    |   instantiated_package_name
    |    open
```

Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
110
Copyright © 2019 IEEE. All rights reserved.
Each association element in an association list associates one actual designator with the corresponding
interface element in the interface list. The corresponding interface element is determined either by position
or by name.
An association element is said to be named if the formal designator appears explicitly; otherwise, it is said to
be positional. For a positional association, an actual designator at a given position in an association list
corresponds to the interface element at the same position in the interface list.
Named associations can be given in any order, but if both positional and named associations appear in the
same association list, then all positional associations shall occur first at their normal position. Hence once a
named association is used, the rest of the association list shall use only named associations.
In the following paragraphs, the term actual refers to an actual designator, and the term formal refers to a
formal designator.
The formal part of a named association element may be in the form of a function call, where the single
argument of the function is the formal designator itself, if and only if the formal is an interface object, the
mode of the formal is out, inout, buffer, or linkage, and if the actual is not open. In this case, the function
name shall denote a function whose single parameter is of the type of the formal and whose result is the type
of the corresponding actual. Such a conversion function provides for type conversion in the event that data
flows from the formal to the actual.
Alternatively, the formal part of a named association element may be in the form of a type conversion,
where the expression to be converted is the formal designator itself, if and only if the formal is an interface
object, the mode of the formal is out, inout, buffer, or linkage, and if the actual is not open. In this case, the
base type denoted by the type mark shall be the same as the base type of the corresponding actual. Such a
type conversion provides for type conversion in the event that data flows from the formal to the actual. It is
an error if the type of the formal is not closely related to the type of the actual (see 9.3.6).
The actual part of a (named or positional) association element corresponding to a formal interface object
may have the syntactic form of a function call. This form may be interpreted either as a function call whose
parameter is the actual designator, or as an expression, in which case the entire expression is the actual
designator. The actual part is interpreted as a function call whose parameter is the actual designator if and
only if:
—
The corresponding function declaration has one parameter,
—
The mode of the formal corresponding to the association element is in, inout, or linkage and the
class of the formal is not constant,
—
The function parameter is a signal name or a variable name, and
—
The function name is not preceded by the reserved word inertial.
Otherwise, the entire expression given by the function call is interpreted as the actual designator. In the case
of a function call whose parameter is the actual designator, the type of the function parameter shall be the
type of the actual and the result type shall be the type of the corresponding formal. Such a function call is
interpreted as application of a conversion function that provides for type conversion in the event that data
flows from the actual to the formal.
Alternatively, the actual part of a (named or positional) association element corresponding to a formal
interface object may have the syntactic form of a type conversion. This form may be interpreted either as a
type conversion whose operand is the actual designator, or as an expression, in which case the entire
expression is the actual designator. The actual part is interpreted as a type conversion whose operand is the
actual designator if and only if:
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
111
Copyright © 2019 IEEE. All rights reserved.
—
The mode of the formal corresponding to the association element is in, inout, or linkage, and the
class of the formal is not constant,
—
The operand is a signal name or a variable name, and
—
The type mark is not preceded by the reserved word inertial.
Otherwise, the entire expression given by the type conversion is interpreted as the actual designator. In the
case of a type conversion whose operand is the actual designator, the base type denoted by the type mark
shall be the same as the base type of the corresponding formal. Such a type conversion provides for type
conversion in the event that data flows from the actual to the formal. It is an error if the type of the actual is
not closely related to the type of the formal.
The type of the actual (after applying the conversion function or type conversion, if present in the actual
part) shall be the same as the type of the corresponding formal, if the mode of the formal is in, inout, or
linkage, and if the actual is not open. Similarly, if the mode of the formal is out, inout, buffer, or linkage,
and if the actual is not open, then the type of the formal (after applying the conversion function or type
conversion, if present in the formal part) shall be the same as the corresponding actual.
The association of a formal port of a given composite type with an actual of the same type is equivalent to
the association of each scalar subelement of the formal with the matching subelement of the actual, provided
that no conversion function or type conversion is present in either the actual part or the formal part of the
association element. If a conversion function or type conversion is present, then the entire formal is
considered to be associated with the entire actual.
Similarly, for the association of actuals with corresponding formal subprogram parameters, association of a
formal parameter of a given composite type with an actual of the same type is equivalent to the association
of each scalar subelement of the formal parameter with the matching subelement of the actual. Different
parameter passing mechanisms may be required in each case, but in both cases the associations will have an
equivalent effect. This equivalence applies provided that no actual is accessible by more than one path (see
4.2.2).
A formal interface object shall be either an explicitly declared interface object or member (see 5.1) of such
an interface object. In the former case, such a formal is said to be associated in whole. In the latter cases,
named association shall be used to associate the formal and actual; the subelements of such a formal are said
to be associated individually. Furthermore, every scalar subelement of the explicitly declared interface
object shall be associated exactly once with an actual (thereof) in the same association list, and all such
associations shall appear in a contiguous sequence within that association list. Each association element that
associates a slice subelement (or slice thereof) of an interface object shall identify the formal with a locally
static name.
If an interface element in an interface list includes a default expression for a formal generic constant, for a
formal signal port of any mode other than linkage, or for a formal variable or constant parameter of mode in,
or an interface subprogram default for a formal generic subprogram, then any corresponding association list
need not include an association element for that interface element. For an interface element that is a formal
generic constant, a formal signal port, or a formal variable or constant parameter, if the association element
is not included in the association list, or if the actual is open, then the value of the default expression is used
as the actual expression or signal value in an implicit association element for that interface element. For an
interface element that is a formal generic subprogram, if the association element is not included in the
association list, or if the actual is open, then the subprogram denoted by the formal generic subprogram is
determined by the interface subprogram default as described in 6.5.6.2.
An actual of open counts as the single association allowed for the corresponding formal interface object or
subelement or slice thereof, but does not supply a constant, signal, or variable (as is appropriate to the object
class of the formal) to the formal.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
112
Copyright © 2019 IEEE. All rights reserved.
It is an error if the reserved word inertial appears in an association element other than in.
For any VHDL construct with a port or parameter association list whose corresponding interface object
declaration has a type defined by an unspecified type declaration, there is a corresponding construct with a
generic association list and a port or parameter list. For each formal port or parameter whose type is
specified as an unspecified type declaration, an association element is added to the end of the generic
association list to associate the unspecified formal generic type with the subtype of the actual port or
parameter. It is an error if the subtype of the actual port or parameter is not determinable from the context. If
a generic association list does not already exist, one is created. The original port or parameter list is
unchanged.
Example:
Architecture A of E is
    signal SigA : std_logic_vector(7 downto 0);
    signal SigB : std_logic;
    component C is
    port (
        A : type is private; -- any type
        B : type is <>        -- a scalar type
    );
    end component C;
begin
    -- A component instance whose type of the formal signal port A
    -- is defined by an unspecified type declaration.
    E1 : C
        port map (
            A => SigA;
            B => SigB
        );
    ...
-- The equivalent component declaration and instance.
Architecture A of E is
signal SigA : std_logic_vector(7 downto 0);
signal SigB : std_logic;
-- The equivalent component declaration is formed by the rules
-- of Clause 6.5.2
component C is
generic (
type Anonymous1 is private;   -- any type
type Anonymous2 is <>         -- a scalar type
)
port (
A : Anonymous1;
B : Anonymous2
) ;
end component C;
begin
-- The equivalent component instance has a generic map to
-- associate the subtype of actual port SigA with the formal
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
113
Copyright © 2019 IEEE. All rights reserved.
-- generic type Anonymous. The component instance has the same
-- port map as the original instance.
E1 : C
generic map (
Anonymous1 => SigA'SUBTYPE ;
Anonymous2 => SigB'SUBTYPE
)
port map (
A => SigA  ;
B => SigB
);
    ...
NOTE 1—It is a consequence of these rules that, if an association element is omitted from an association list in order to
make use of the default expression on the corresponding interface element, all subsequent association elements in that
association list will be named associations.
NOTE 2—Although a default expression can appear in an interface element that declares a (local or formal) signal port,
such a default expression is not interpreted as the value of an implicit association element for that port. Instead, the value
of the expression is used to determine the effective value of that port during simulation if the port is left unconnected
(see 14.7.3).
NOTE 3—Named association cannot be used when invoking implicitly defined operators or predefined attributes that
are functions, since the formal parameters of these operators and functions are not named (see 9.2 and 16.2).
NOTE 4—Since information flows only from the actual to the formal when the mode of the formal is in, and since a
function call is itself an expression, the actual associated with a formal of object class constant is never interpreted as a
conversion function or a type conversion converting an actual designator that is an expression. Thus, the following
association element is legal
Param => F (open)
under the conditions that Param is a constant formal and F is a function returning the same base type as that of Param and
having one or more parameters, all of which may be defaulted. It is an error if a conversion function or type conversion
appears in the actual part when the actual designator is open.
##### 6.5.7.2 Generic map aspects

A generic map aspect, other than one appearing in an interface package generic map aspect (see 6.5.5),
associates values, subtypes, subprograms, or instantiated packages with the formal generics of a block, a
package, a subprogram or a protected type. The following applies to external blocks defined by design
entities, to internal blocks defined by block statements, and to packages, subprograms and protected types.
```ebnf
generic_map_aspect ::=
    generic map ( generic_association_list )
```

Both named and positional association are allowed in a generic association list.
The following definitions are used in the remainder of this subclause:
—
The term actual refers to an actual designator that appears in an association element of a generic
association list.
—
The term formal refers to a formal designator that appears in an association element of a generic
association list.
The purpose of a generic map aspect is as follows:
—
A generic map aspect appearing immediately within a binding indication associates actuals with the
formals of the entity declaration implied by the immediately enclosing binding indication.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
114
Copyright © 2019 IEEE. All rights reserved.
—
A generic map aspect appearing immediately within a component instantiation statement associates
actuals with the formals of the component instantiated by the statement.
—
A generic map aspect appearing immediately within a block header associates actuals with the
formals defined by the same block header.
—
A generic map aspect appearing immediately within a package header associates actuals with the
formals defined by the same package header. This applies to a generic map aspect appearing in the
package header of an explicitly declared generic-mapped package or a generic-mapped package that
is equivalent to a package instantiation declaration.
—
A generic map aspect appearing immediately within a subprogram header associates actuals with the
formals defined by the same subprogram header. This applies to a generic map aspect appearing in
the subprogram header of an explicitly declared generic-mapped subprogram, a generic-mapped
subprogram that is equivalent to a subprogram instantiation declaration, or a generic-mapped
subprogram that is implicitly defined when a subprogram call consists of a call to an uninstantiated
subprogram with a generic map aspect.
—
A generic map aspect appearing immediately within a protected type header associates actuals with
the formals defined by the same protected type header. This applies to a generic map aspect
appearing in a protected type header of an explicitly declared generic-mapped protected type, a
generic-mapped protected type that is equivalent to a protected type instantiation declaration, or a
generic-mapped protected type that is implicitly declared when the subtype indication of a variable
declaration, access type definition, or an allocator is an uninstantiated protected type and it has a
generic map aspect.
In each case, for a formal generic constant, it is an error if a scalar formal is associated with more than one
actual, and it is an error if a scalar subelement of any composite formal is associated with more than one
scalar subelement of an actual. Similarly, for a formal generic type, a formal generic subprogram, or a
formal generic package, it is an error if the formal is associated with more than one actual. Thus, it is an
error if two formal generic subprograms have the same designator and the same signature. It is also an error
if a formal generic subprogram has a signature, which is not listed in an interface subprogram declaration for
that designator.
An actual associated with a formal generic constant in a generic map aspect shall be an expression or the
reserved word open. An actual associated with a formal generic type shall be a subtype indication. An actual
associated with a formal generic subprogram shall be a name that denotes a subprogram whose profile
conforms to that of the formal, or the reserved word open. The actual, if a predefined attribute name that
denotes a function, shall be one of the predefined attributes 'IMAGE, 'VALUE, 'POS, 'VAL, 'SUCC, 'PRED,
'LEFTOF, or 'RIGHTOF.
For a formal generic array interface type declaration, if the array index subtype is represented by an implicit
formal generic type, then an implicit association element is added that associates the implicit formal with the
subtype returned when 'INDEX is applied to the actual of the corresponding array type association element.
For a formal generic array interface type declaration, if the element subtype is represented by an implicit
formal generic type, then an implicit association element is added that associates the implicit formal with the
subtype returned when 'ELEMENT is applied to the actual of the corresponding array type association
element.
For a formal generic access interface type declaration, if the designated subtype is represented by an implicit
formal generic type, then an implicit association element is added that associates the implicit formal with the
subtype returned when 'DESIGNATED_SUBTYPE is applied to the actual of the corresponding access type
association element.
For a formal generic file interface type declaration, if the designated subtype is represented by an implicit
formal generic type, then an implicit association element is added that associates the implicit formal with the
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
115
Copyright © 2019 IEEE. All rights reserved.
subtype returned when 'DESIGNATED_SUBTYPE is applied to the actual of the corresponding file type
association element.
An actual associated with a formal generic package in a generic map aspect shall be a name that denotes an
instance of the uninstantiated package named in the formal generic package declaration, as follows:
a)
If the formal generic package declaration includes an interface package generic map aspect in the
form of a generic map aspect, then the generic map aspect of the package instantiation declaration
that declares the instantiated package denoted by the actual shall match the generic map aspect of
the formal generic package declaration. The two generic map aspects match if, for each generic, the
corresponding associated actuals, whether explicit or implicit, match as follows:
—
Two actual generic constants match if they are the same value.
—
Two actual generic types match if they denote the same subtype; that is, if the subtypes denoted
by the two actual generic types have the same base type and the same constraints. Two range
constraints are the same if they have the same bounds and directions. Two array constraints are
the same if they define the same index ranges and the same element subtypes. Two record
constraints are the same if, for each element, the element subtypes are the same.
—
Two actual generic packages match if they denote the same instantiated package.
—
Two actual generic subprograms match if they denote the same subprogram.
b)
If the formal generic package declaration includes an interface package generic map aspect in the
form that includes the box (<>) symbol, then the instantiated package denoted by the actual may be
any instance of the uninstantiated package named in the formal generic package declaration.
c)
If the formal generic package declaration includes an interface package generic map aspect in the
form that includes the reserved word default, then the generic map aspect of the package instantia-
tion declaration that declares the instantiated package denoted by the actual shall match the implicit
generic map aspect defined in 6.5.5.
A formal that is not associated with an actual is said to be an unassociated formal.
NOTE 1—A generic map aspect appearing immediately within a binding indication need not associate every formal
generic constant with an actual. These formals may be left unbound so that, for example, a component configuration
within a configuration declaration may subsequently bind them.
NOTE 2—A local generic (from a component declaration) or formal generic (from a package, a subprogram, a protected
type, a block statement or from the entity declaration of the enclosing design entity) may appear as an actual in a generic
map aspect.
NOTE 3—If a formal generic constant is rebound by an incremental binding indication, the actual expression associated
by the formal generic in the primary binding indication is not evaluated during the elaboration of the description.
Cross-references: Generic clauses, 6.5.6.2.
Example:
Clause 16 defines an uninstantiated package in library IEEE for fixed-point binary numbers, as follows:
package fixed_generic_pkg is
   generic (fixed_round_style: BOOLEAN;
            fixed_overflow_style: BOOLEAN;
            fixed_guard_bits: NATURAL;
            no_warning: BOOLEAN);
   type ufixed is array (INTEGER range <>) of STD_ULOGIC;
   type sfixed is array (INTEGER range <>) of STD_ULOGIC;
   ...
end package fixed_generic_pkg;
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
116
Copyright © 2019 IEEE. All rights reserved.
The package may be instantiated in a design unit as follows:
package fixed_dsp_pkg is new IEEE.fixed_generic_pkg
   generic map (fixed_rounding_style => FALSE,
                fixed_overflow_style => FALSE,
                fixed_guard_bits => 0, no_warning => TRUE);
An uninstantiated package defining complex numbers in which the real and imaginary parts are fixed-point
binary numbers with the same index ranges can be defined as follows:
package fixed_complex_generic_pkg is
generic (
complex_fixed_left, complex_fixed_right: INTEGER;
package complex_fixed_formal_pkg is
new IEEE.fixed_generic_pkg generic map (<>)
);
use complex_fixed_formal_pkg.all;
type complex is record
re, im : sfixed(complex_fixed_left downto complex_fixed_right);
end record;
function "-"  (z : complex ) return complex;
function conj (z : complex ) return complex;
function "+"  (l: complex; r: complex) return complex;
function "-"  (l: complex; r: complex) return complex;
function "*"  (l: complex; r: complex) return complex;
function "/"  (l: complex; r: complex) return complex;
end package fixed_complex_generic_pkg;
This package may be instantiated to use the types and operations defined in fixed_dsp_pkg as follows:
package dsp_complex_pkg is new fixed_complex_generic_pkg
   generic map (
complex_fixed_left => 3,

complex_fixed_right => -12,
complex_fixed_formal_pkg => fixed_dsp_pkg);
A further uninstantiated package defining mathematical operations on fixed-point binary numbers can be
defined as follows:
package fixed_math_generic_pkg is
generic (
package math_fixed_formal_pkg is
new IEEE.fixed_generic_pkg generic map (<>)
);
use math_fixed_formal_pkg.all;
function sqrt (x: sfixed) return sfixed;
function exp  (x: sfixed) return sfixed;
   ...
end package fixed_math_generic_pkg;
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
117
Copyright © 2019 IEEE. All rights reserved.
This package, together with the complex numbers package, can be used to define an uninstantiated package
that provides mathematical operations on complex numbers. Since the mathematical operations and the
complex number representation depend on the fixed-point number package, an instance of the fixed-point
package, together with instances of the mathematical operations and complex numbers packages that refer to
the fixed-point package instance, should be provided to the complex mathematical operations package.
Thus, this package has formal generic packages as follows:
package fixed_complex_math_generic_pkg is
generic (
complex_math_fixed_left,
complex_math_fixed_right: integer;
package complex_math_fixed_formal_pkg is
new IEEE.fixed_generic_pkg generic map (<>);
package fixed_math_formal_pkg is
new fixed_math_generic_pkg

generic map (
math_fixed_formal_pkg =>

complex_math_fixed_formal_pkg
);
package fixed_complex_formal_pkg is
new fixed_complex_generic_pkg
generic map (
complex_fixed_left =>

complex_math_fixed_left,

complex_fixed_right =>

complex_math_fixed_right,

complex_fixed_formal_pkg =>

complex_math_fixed_formal_pkg
)
);

use complex_math_fixed_formal_pkg.all,

 fixed_math_formal_pkg.all,

 fixed_complex_formal_pkg.all;
function "abs" (z: complex) return sfixed;
function arg   (z: complex) return sfixed;
function sqrt  (z: complex) return complex;
...
end package fixed_complex_math_generic_pkg;
The mathematical packages may be instantiated as follows:
package dsp_math_pkg is new fixed_math_generic_pkg
   generic map ( math_fixed_formal_pkg => fixed_dsp_pkg );
package dsp_complex_math_pkg is new fixed_complex_math_generic_pkg
   generic map (complex_math_fixed_left => 3,
                complex_math_fixed_right => 3,
                complex_math_fixed_formal_pkg => fixed_dsp_pkg,
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
118
Copyright © 2019 IEEE. All rights reserved.
                fixed_math_formal_pkg => dsp_math_pkg,
                fixed_complex_formal_pkg => dsp_complex_pkg);
##### 6.5.7.3 Port map aspects

A port map aspect associates signals, shared variables or values with the formal ports of a block. The
following applies to both external blocks defined by design entities and to internal blocks defined by block
statements.
```ebnf
port_map_aspect ::=
    port map ( port_association_list )
```

Both named and positional association are allowed in a port association list.
The following definitions are used in the remainder of this subclause:
—
The term actual refers to an actual designator that appears in an association element of a port
association list.
—
The term formal refers to a formal designator that appears in an association element of a port
association list.
The purpose of a port map aspect is as follows:
—
A port map aspect appearing immediately within a binding indication associates actuals with the
formals of the entity declaration implied by the immediately enclosing binding indication.
Each scalar subelement of every local port of the component instances to which an enclosing
configuration specification or component configuration applies shall be associated as an actual with
at least one formal or with a scalar subelement thereof. The actuals of these associations for a given
local port shall be either the entire local port or any slice (or slice thereof) of the local port. The
actuals in these associations shall be locally static names.
—
A port map aspect appearing immediately within a component instantiation statement associates
actuals with the formals of the component instantiated by the statement.
—
A port map aspect appearing immediately within a block header associates actuals with the formals
defined by the same block header.
In each case, it is an error if a scalar formal is associated with more than one actual, and is an error if a scalar
subelement of any composite formal is associated with more than one scalar subelement of an actual.
An actual associated with a formal signal port in a port map aspect shall be a signal, an expression, or the
reserved word open. An actual associated with a formal variable port in a port map aspect shall be a shared
variable or a formal variable port.
Certain restrictions apply to the actual associated with a formal port in; these restrictions are described in
6.5.6.3.
A formal that is not associated with an actual is said to be an unassociated formal.
entity Buf is
generic (Buf_Delay: TIME := 0 ns);
port (Input_pin: in Bit; Output_pin: out Bit);
end Buf;
architecture DataFlow of Buf is
begin
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
119
Copyright © 2019 IEEE. All rights reserved.
Output_pin <= Input_pin after Buf_Delay;
end DataFlow;
entity Test_Bench is
end Test_Bench;
architecture Structure of Test_Bench is
component Buf is
generic (Comp_Buf_Delay: TIME);
port (Comp_I: in Bit; Comp_O: out Bit);
end component;
-- A binding indication; generic and port map aspects within a
-- binding indication associate actuals (Comp_I, etc.) with
-- formals of the entity declaration (Input_pin, etc.):

for UUT: Buf

use entity Work.Buf(DataFlow)

generic map (Buf_Delay => Comp_Buf_Delay)

port map (Input_pin => Comp_I, Output_pin=> Comp_O);
signal S1,S2: Bit;
begin
-- A component instantiation statement; generic and port map

-- aspects within a component instantiation statement associate

-- actuals (S1, etc.) with the formals of a component

--(Comp_I, etc.):
UUT: Buf

generic map (Comp_Buf_Delay => 50 ns)

port map (Comp_I => S1, Comp_O => S2);
-- A block statement; generic and port map aspects within the
-- block header of a block statement associate actuals (in this
-- case, 4) with the formals defined in the block header:
B: block
generic (G: INTEGER);
generic map (G => 4);
begin
end block;
end Structure;
NOTE—A local port (from a component declaration) or formal port (from a block statement or from the entity
declaration of the enclosing design entity) may appear as an actual in.
Cross-references: Port clauses, 6.5.6.3.
### 6.6 Alias declarations

```ebnf
alias_declaration ::=
    alias alias_designator [ : subtype_indication ] is name [ signature ];
alias_designator ::=  identifier | character_literal | operator_symbol
```

An object alias is an alias whose alias designator denotes an object (i.e., a constant, a variable, a signal, or a
file). A nonobject alias is an alias whose alias designator denotes some named entity other than an object.
An alias can be declared for all named entities except for labels, loop parameters, and generate parameters.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
120
Copyright © 2019 IEEE. All rights reserved.
The alias designator in an alias declaration denotes the named entity specified by the name and, if present,
the signature in the alias declaration. An alias of a signal denotes a signal; an alias of a variable denotes a
variable; an alias of a constant denotes a constant; and an alias of a file denotes a file. Similarly, an alias of a
subprogram (including an operator) denotes a subprogram, an alias of an enumeration literal denotes an
enumeration literal, and so forth.
If the alias designator is a character literal, the name shall denote an enumeration literal. If the alias
designator is an operator symbol, the name shall denote a function, and that function then overloads the
operator symbol. In this latter case, the operator symbol and the function both shall meet the requirements of
4.5.2.
NOTE 1—Since, for example, the alias of a variable is a variable, every reference within this document to a designator
(a name, character literal, or operator symbol) that requires the designator to denote a named entity with certain
characteristics (e.g., to be a variable) allows the designator to denote an alias, so long as the aliased name denotes a
named entity having the required characteristics. This situation holds except where aliases are specifically prohibited.
NOTE 2—The alias of an overloadable named entity is itself overloadable.
#### 6.6.1 Object aliases

The following rules apply to object aliases:
a)
A signature shall not appear in a declaration of an object alias.
b)
If the name is an external name, an indication shall not appear in the alias declaration.
c)
The name shall be a static name (see 8.1) that denotes an object. The base type of the name specified
in an alias declaration shall be the same as the base type of the type mark in the subtype indication
(if the subtype indication is present). When the object denoted by the name is referenced via the alias
defined by the alias declaration, the following rules apply:
1)
If the subtype indication is absent
—
If the alias designator denotes a slice of an object, then the slice of the object is viewed as
if it were of the subtype specified by the slice.
—
If the name is an external name, then the object is viewed as if it were of the subtype
specified in the external name.
—
Otherwise, the object is viewed as if it were of the subtype specified in the declaration of
the object denoted by the name.
2)
If the subtype indication is present and denotes a composite subtype, then the object is viewed
as if it were of the subtype specified by the subtype indication. For each index range, if any, in
the subtype, if the subtype defines the index range, the object is viewed with that index range;
otherwise, the object is viewed with the index range of the object. The view specified by the
subtype shall include a matching element (see 9.2.3) for each element of the object denoted by
the name.
3)
If the subtype indication denotes a scalar subtype, then the object is viewed as if it were of the
subtype specified by the subtype indication; moreover, it is an error if this subtype does not
have the same bounds and direction as the subtype denoted by the object name.
d)
When the prefix of an attribute name denotes the alias defined by the alias declaration, subrules 1),
2), and 3), of rule c) apply.
e)
A reference to an element of an object alias is implicitly a reference to the matching element of the
object denoted by the alias. A reference to a slice of an object alias consisting of the elements e1, e2,
…, en is implicitly a reference to a slice of the object denoted by the alias consisting of the matching
elements corresponding to each of e1 through en.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
121
Copyright © 2019 IEEE. All rights reserved.
#### 6.6.2 Nonobject aliases

The following rules apply to nonobject aliases:
a)
A subtype indication shall not appear in a nonobject alias.
b)
A signature is required if the name denotes a subprogram (including an operator) or enumeration
literal. In this case, the signature is required to match (see 4.5.3) the parameter and result type profile
of exactly one of the subprograms or enumeration literals denoted by the name.
c)
If the name denotes an enumeration type or a subtype of an enumeration type, then one implicit alias
declaration for each of the literals of the base type immediately follows the alias declaration for the
enumeration type; each such implicit declaration has, as its alias designator, the simple name or
character literal of the literal and has, as its name, a name constructed by taking the name of the alias
for the enumeration type or subtype and substituting the simple name or character literal being
aliased for the simple name of the type or subtype. Each implicit alias has a signature that matches
the parameter and result type profile of the literal being aliased.
d)
Alternatively, if the name denotes a subtype of a physical type, then one implicit alias declaration
for each of the units of the base type immediately follows the alias declaration for the physical type;
each such implicit declaration has, as its alias designator, the simple name of the unit and has, as its
name, a name constructed by taking the name of the alias for the subtype of the physical type and
substituting the simple name of the unit being aliased for the simple name of the subtype.
e)
Finally, if the name denotes a type or a subtype, then implicit alias declarations for each predefined
operation for the type immediately follow the explicit alias declaration for the type or subtype and, if
present, any implicit alias declarations for literals or units of the type. Each implicit alias has a
signature that matches the parameter and result type profile of the implicit operation being aliased.
Examples:
variable REAL_NUMBER: BIT_VECTOR (0 to 31);
alias SIGN: BIT is REAL_NUMBER (0);
   -- SIGN is now a scalar (BIT) value
alias MANTISSA: BIT_VECTOR (23 downto 0) is REAL_NUMBER (8 to 31);
   -- MANTISSA is a 24-bit value whose range is 23 downto 0.
   -- Note that the ranges of MANTISSA and REAL_NUMBER (8 to 31)
   -- have opposite directions. A reference to MANTISSA (23 downto 18)
   -- is equivalent to a reference to REAL_NUMBER (8 to 13).
alias EXPONENT: BIT_VECTOR (1 to 7) is REAL_NUMBER (1 to 7);
   -- EXPONENT is a 7-bit value whose range is 1 to 7.
alias STD_BIT         is STD.STANDARD.BIT;  -- explicit alias
-- implicit aliases ...
-- alias '0'    is STD.STANDARD.'0' [return STD.STANDARD.BIT];
-- alias '1'    is STD.STANDARD.'1' [return STD.STANDARD.BIT];
-- alias "and"  is STD.STANDARD."and"  [STD.STANDARD.BIT,
--                                      STD.STANDARD.BIT
--                                         return STD.STANDARD.BIT];
-- alias "or"   is STD.STANDARD."or"   [STD.STANDARD.BIT,
--                                      STD.STANDARD.BIT
--                                         return STD.STANDARD.BIT];
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
122
Copyright © 2019 IEEE. All rights reserved.
-- alias "nand" is STD.STANDARD."nand" [STD.STANDARD.BIT,
--                                      STD.STANDARD.BIT
--                                         return STD.STANDARD.BIT];
-- alias "nor"  is STD.STANDARD."nor"  [STD.STANDARD.BIT,
--                                      STD.STANDARD.BIT
--                                         return STD.STANDARD.BIT];
-- alias "xor"  is STD.STANDARD."xor"  [STD.STANDARD.BIT,
--                                      STD.STANDARD.BIT
--                                         return STD.STANDARD.BIT];
-- alias "xnor" is STD.STANDARD."xnor" [STD.STANDARD.BIT,
--                                      STD.STANDARD.BIT
--                                         return STD.STANDARD.BIT];
-- alias "not"  is STD.STANDARD."not"  [STD.STANDARD.BIT
--                                         return STD.STANDARD.BIT];
-- alias "="    is STD.STANDARD."="    [STD.STANDARD.BIT,
--                                      STD.STANDARD.BIT
--                                         return STD.STANDARD.BOOLEAN];
-- alias "/="   is STD.STANDARD."/="   [STD.STANDARD.BIT,
--                                      STD.STANDARD.BIT
--                                         return STD.STANDARD.BOOLEAN];
-- alias "<"    is STD.STANDARD."<"    [STD.STANDARD.BIT,
--                                      STD.STANDARD.BIT
--                                         return STD.STANDARD.BOOLEAN];
-- alias "<="   is STD.STANDARD."<="   [STD.STANDARD.BIT,
--                                      STD.STANDARD.BIT
--                                         return STD.STANDARD.BOOLEAN];
-- alias ">"    is STD.STANDARD.">"    [STD.STANDARD.BIT,
--                                      STD.STANDARD.BIT
--                                         return STD.STANDARD.BOOLEAN];
-- alias ">="   is STD.STANDARD.">="   [STD.STANDARD.BIT,
--                                      STD.STANDARD.BIT
--                                         return STD.STANDARD.BOOLEAN];
-- alias MINIMUM is STD.STANDARD.MINIMUM [STD.STANDARD.BIT,
--                                        STD.STANDARD.BIT
--                                           return STD.STANDARD.BIT];
-- alias MAXIMUM is STD.STANDARD.MAXIMUM [STD.STANDARD.BIT,
--                                        STD.STANDARD.BIT
--                                           return STD.STANDARD.BIT];
NOTE—An alias of an explicitly declared object is not an explicitly declared object, nor is the alias of a subelement or
slice of an explicitly declared object an explicitly declared object.
### 6.7 Attribute declarations

An attribute is a value, function, type, range, signal, or constant that may be associated with one or more
named entities in a description. There are two categories of attributes: predefined attributes and user-defined
attributes. Predefined attributes provide information about named entities in a description. Clause 16
contains the definition of all predefined attributes. Predefined attributes that are signals shall not be updated.
User-defined attributes are constants of arbitrary type. Such attributes are defined by an attribute declaration.
```ebnf
attribute_declaration ::=
    attribute identifier : type_mark ;
```

Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
123
Copyright © 2019 IEEE. All rights reserved.
The identifier is said to be the designator of the attribute.
It is an error if the type mark denotes an access type, a file type, a protected type, or a composite type with a
subelement that is of an access type, a file type or a protected type. The denoted type or subtype need not be
constrained.
Examples:
type COORDINATE is
record
    X,Y: INTEGER;
      end record;
subtype POSITIVE is INTEGER range 1 to INTEGER'HIGH;
attribute LOCATION: COORDINATE;
attribute PIN_NO: POSITIVE;
### 6.8 Component declarations

A component declaration declares an interface to a virtual design entity that may be used in a component
instantiation statement. A component configuration or a configuration specification can be used to associate
a component instance with a design entity that resides in a library.
```ebnf
component_declaration ::=
    component identifier [ is ]
        [ local_generic_clause ]
        [ local_port_clause ]
    end [ component ] [ component_simple_name ] ;
```

Each interface object in the local generic clause declares a local generic. Each interface object in the local
port clause declares a local port.
If a simple name appears at the end of a component declaration, it shall repeat the identifier of the
component declaration.
### 6.9 Group template declarations

A group template declaration declares a group template, which defines the allowable classes of named
entities that can appear in a group.
```ebnf
group_template_declaration ::=
    group identifier is ( entity_class_entry_list ) ;
entity_class_entry_list ::=
    entity_class_entry { , entity_class_entry }
entity_class_entry ::=  entity_class [ <> ]
```

A group template is characterized by the number of entity class entries and the entity class at each position.
Entity classes are described in 7.2.
An entity class entry that is an entity class defines the entity class that may appear at that position in the
group type. An entity class entry that includes a box (<>) allows zero or more group constituents to appear in
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
124
Copyright © 2019 IEEE. All rights reserved.
this position in the corresponding group declaration; such an entity class entry shall be the last one within the
entity class entry list.
Examples:
group PIN2PIN is (signal, signal);  --  Groups of this type consist of
                                    --  two signals.
group RESOURCE is (label <>);       --  Groups of this type consist of
                                    --  any number of labels.
group DIFF_CYCLES is (group <>);    --  A group of groups.
### 6.10 Group declarations

A group declaration declares a group, a named collection of named entities. Named entities are described in
7.2.
```ebnf
group_declaration ::=
    group identifier : group_template_name ( group_constituent_list ) ;
group_constituent_list ::=  group_constituent { , group_constituent }
group_constituent ::=  name | character_literal
```

It is an error if the class of any group constituent in the group constituent list is not the same as the class
specified by the corresponding entity class entry in the entity class entry list of the group template.
A name that is a group constituent shall not be an attribute name (see 8.7). Moreover, if such a name
contains a prefix, it is an error if the prefix is a function call.
If a group declaration appears within a package body, and a group constituent within that group declaration
is the same as the simple name of the package body, then the group constituent denotes the package
declaration and not the package body. The same rule holds for group declarations appearing within
subprogram bodies containing group constituents with the same designator as that of the enclosing
subprogram body.
If a group declaration contains a group constituent that denotes a variable of an access type, the group
declaration declares a group incorporating the variable itself, and not the designated object, if any.
Examples:
group G1: RESOURCE (L1, L2); --  A group of two labels.
group G2: RESOURCE (L3, L4, L5);           --  A group of three labels.
group C2Q: PIN2PIN (PROJECT.GLOBALS.CK, Q);

--  Groups may associate named

--  entities in different declarative

--  parts (and regions).
group CONSTRAINT1: DIFF_CYCLES (G1, G2);   --  A group of groups.
### 6.11 PSL clock declarations

A PSL clock declaration may occur as an entity declarative item (see 3.2.3) or a block declarative item (see
3.3.2) and applies to certain PSL directives (if any) in the declarative region containing the PSL clock
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
125
Copyright © 2019 IEEE. All rights reserved.
declaration. The PSL clock declaration, if any, that applies to a given PSL directive is the PSL clock
declaration in the innermost declarative region containing both the given directive and a PSL clock
directive. It is an error if more than one PSL clock declaration appears immediately within a given
declarative region.
NOTE—A PSL clock declaration differs from other declarations in VHDL and PSL in that it does not declare a
designator denoting some entity. It is more akin to a VHDL specification in that it associates additional information with
PSL directives within a design. Hence, it is not listed as a declaration in 6.1. Since it is called a declaration in
IEEE Std 1850-2010, it is included in this clause for ease of reference, rather than in Clause 7.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
