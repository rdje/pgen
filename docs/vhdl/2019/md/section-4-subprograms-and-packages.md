---
title: "Section 4: IEEE Standard for VHDL Language Reference Manual"
document: "VHDL Language Reference Manual"
standard: "IEEE 1076-2019"
domain: "VHDL"
section: "4"
source_txt: "section-4-subprograms-and-packages.txt"
source_pdf: "/Users/richarddje/Documents/github/VHDL-LRM-IEEE-1076-2019.pdf"
---

# Section 4: IEEE Standard for VHDL Language Reference Manual

IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
32
Copyright © 2019 IEEE. All rights reserved.
4. Subprograms and packages
### 4.1 General

Subprograms define algorithms for computing values or exhibiting behavior. They may be used as
computational resources to convert between values of different types, to define the resolution of output
values driving a common signal, or to define portions of a process. Packages provide a means of defining
these and other resources in a way that allows different design units or different parts of a given design unit
to share the same declarations.
There are two forms of subprograms: procedures and functions. A procedure call is a statement; a function
call is an expression and returns a value. Certain functions, designated pure functions, return the same value
each time they are called with the same values as actual parameters; the remainder, impure functions, may
return a different value each time they are called, even when multiple calls have the same actual parameter
values. For the purposes of determining purity, the return identifier of a function is considered a parameter.
In addition, impure functions can update objects outside of their scope and can access a broader class of
values than can pure functions. The definition of a subprogram can be given in two parts: a subprogram
declaration defining its calling conventions, and a subprogram body defining its execution.
Packages may also be defined in two parts. A package declaration defines the visible contents of a package;
a package body provides hidden details. In particular, a package body contains the bodies of any
subprograms declared in the package declaration.
### 4.2 Subprogram declarations

#### 4.2.1 General

A subprogram declaration declares a procedure or a function, as indicated by the appropriate reserved word.
```ebnf
subprogram_declaration ::=
```

subprogram_specification ;
```ebnf
subprogram_specification ::=
```

procedure_specification | function_specification
```ebnf
procedure_specification ::=
```

procedure designator
subprogram_header
[ [ parameter ] ( formal_parameter_list ) ]
```ebnf
function_specification ::=
```

[ pure | impure ] function designator
subprogram_header
[ [ parameter ] ( formal_parameter_list ) ] return [ return_identifier of ] type_mark
```ebnf
subprogram_header ::=
```

[ generic ( generic_list )
[ generic_map_aspect ] ]
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
33
Copyright © 2019 IEEE. All rights reserved.
```ebnf
designator ::=  identifier  |  operator_symbol
operator_symbol ::=  string_literal
```

The specification of a procedure specifies its designator, its generics (if any), and its formal parameters (if
any). The specification of a function specifies its designator, its generics (if any), its formal parameters (if
any), its return identifier (if any), the subtype of the returned value (the result subtype), and whether or not
the function is pure. A function is impure if its specification contains the reserved word impure; otherwise,
it is said to be pure. A procedure designator is always an identifier. A function designator is either an
identifier or an operator symbol. A designator that is an operator symbol is used for the overloading of an
operator (see 4.5.2). The sequence of characters represented by an operator symbol shall be an operator
belonging to one of the classes of operators defined in 9.2. Extra spaces are not allowed in an operator
symbol, and the case of letters is not significant.
For a function that includes a return identifier, an implicit subtype declaration, whose name is the return
identifier, is created as the first declaration in the subprogram declarative part. The properties of the
implicitly declared subtype denoted by the return identifier shall be determined from the immediate context
of the function call, as follows:
a)
If the function call is the expression following the assignment symbol ":=" in a constant, variable, or
signal declaration, then the subtype denoted by the return identifier is the subtype of the
corresponding object declaration.
b)
If the function call is the value expression in an assignment statement whose target is not an
aggregate, then the subtype denoted by the return identifier is the subtype of the target of the
assignment statement.
c)
If the function call is the actual in an association element, then the subtype denoted by the return
identifier is the subtype of the corresponding formal.
d)
If the function call is the formal in an association element, then the subtype denoted by the return
identifier is the subtype of the corresponding actual.
e)
If the function call is the operand in a qualified expression, then the subtype denoted by the return
identifier is the subtype defined by the type mark of the qualified expression.
It is an error if a function has a return identifier and
—
    the function is called from a context other than the ones above or
—
    the subtype of the return identifier is a unconstrained or partially constrained composite subtype.
If the subprogram header is empty, the subprogram declared by a subprogram declaration is called a simple
subprogram. If the subprogram header contains the reserved word generic, a generic list, and no generic
map aspect, the subprogram is called an uninstantiated subprogram. If the subprogram header contains the
reserved word generic, a generic list, and a generic map aspect, the subprogram is called a generic-mapped
subprogram. A subprogram declared with a generic list in which every generic declaration has a default, and
with no generic map aspect, is considered to be an uninstantiated subprogram, not a generic-mapped
subprogram with default associations for all of the generic declarations. A generic list in a subprogram
declaration is equivalent to a generic clause containing that generic list (see 6.5.6.2).
An uninstantiated subprogram without a generic map aspect shall not be called, except as a recursive call
within the body of the uninstantiated subprogram. Moreover, an uninstantiated subprogram shall not be used
as a resolution function or used as a conversion function in an association list.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
34
Copyright © 2019 IEEE. All rights reserved.
It is an error if the result subtype of a function denotes either a file type, a protected type, or a composite type
with a subelement of either a file type or a protected type. Moreover, it is an error if the result subtype of a
pure function denotes an access type or a subtype that has a subelement of an access type.
NOTE 1—All subprograms can be called recursively. In the case of an instantiated subprogram, a reference to the
uninstantiated subprogram within the uninstantiated subprogram is interpreted as a reference to the instance (see 4.4).
Hence, the subprogram can be called recursively using the name of the uninstantiated subprogram. The effect is a
recursive call of the instance.
NOTE 2—The restrictions on pure functions are enforced even when the function appears within a protected type. That
is, pure functions whose body appears in the protected type body will not directly reference variables declared
immediately within the declarative region associated with the protected type. However, impure functions and procedures
whose bodies appear in the protected type body may make such references. Such references are made only when the
referencing subprogram has exclusive access to the declarative region associated with the protected type.
NOTE 3—The rule stating equivalence of a generic list in a subprogram header to a generic clause containing the
generic list ensures that the generic list conforms to the same rules as a generic clause. A subprogram header is not
defined to contain a generic clause directly, since that would introduce a semicolon into the syntax of a subprogram
header.
#### 4.2.2 Formal parameters

##### 4.2.2.1 Formal parameter lists

The formal parameter list in a subprogram specification defines the formal parameters of the subprogram.
```ebnf
formal_parameter_list ::=  parameter_interface_list
```

Formal parameters of subprograms may be constants, variables, signals, or files. In the first three cases, the
mode of a parameter determines how a given formal parameter is accessed within the subprogram. The
mode of a formal parameter, together with its class, also determines how such access is implemented. In the
fourth case, that of files, the parameters have no mode.
For those parameters with modes, the only modes that are allowed for formal parameters of a procedure or
an impure function are in, inout, and out. If the mode is in and no object class is explicitly specified,
constant is assumed. If the mode is inout or out, and no object class is explicitly specified, variable is
assumed.
For those parameters with modes, the only mode that is allowed for formal parameters of a pure function is
the mode in (whether this mode is specified explicitly or implicitly). The object class shall be constant,
signal, or file. If no object class is explicitly given, constant is assumed.
In a subprogram call, the actual designator (see 6.5.7.1) associated with a formal parameter of class signal
shall be a name denoting a signal. The actual designator associated with a formal of class variable shall be a
name denoting a variable. The actual designator associated with a formal of class constant shall be an
expression. The actual designator associated with a formal of class file shall be a name denoting a file.
NOTE 1—Variable class parameters of access and protected types are allowed for procedures and impure functions.
NOTE 2—Attributes of an actual are never passed into a subprogram. References to an attribute of a formal parameter
are legal only if that formal has such an attribute. Such references retrieve the value of the attribute associated with the
formal.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
35
Copyright © 2019 IEEE. All rights reserved.
##### 4.2.2.2 Constant and variable parameters

For parameters of class constant or variable, only the values of the actual or formal are transferred into or
out of the subprogram call. The manner of such transfers, and the accompanying access privileges that are
granted for constant and variable parameters, are described in this subclause.
For a non-foreign subprogram having a parameter of a scalar type or an access type, or for a subprogram
decorated with the 'FOREIGN attribute defined in package STANDARD for which the attribute value is of
the form described in 20.2.4, the parameter is passed by copy. At the start of each call, if the mode is in or
inout, the value of the actual parameter is copied into the associated formal parameter; it is an error if, after
applying any conversion function or type conversion present in the actual part of the applicable association
element (see 6.5.7.1), the value of the actual parameter does not belong to the subtype denoted by the
subtype indication of the formal. After completion of the subprogram body, if the mode is inout or out and
the associated actual parameter is not forced, the value of the formal parameter is copied back into the
associated actual parameter; it is similarly an error if, after applying any conversion function or type
conversion present in the formal part of the applicable association element, the value of the formal
parameter does not belong to the subtype denoted by the subtype indication of the actual.
For a non-foreign subprogram having a parameter whose type is an array or record, an implementation may
pass parameter values by copy, as for scalar types. In that case, after completion of the subprogram body, if
the mode is inout or out, the value of each subelement of the formal parameter is only copied back to the
corresponding subelement of the associated actual parameter if the subelement of the associated actual
parameter is not forced. If a parameter of mode out is passed by copy, then the range of each index position
of the actual parameter is copied in, and likewise for its subelements or slices. Alternatively, an
implementation may achieve these effects by reference; that is, by arranging that every use of the formal
parameter (to read or update its value) be treated as a use of the associated actual parameter throughout the
execution of the subprogram call. The language does not define which of these two mechanisms is to be
adopted for parameter passing, nor whether different calls to the same subprogram are to use the same
mechanism. The execution of a subprogram is erroneous if its effect depends on which mechanism is
selected by the implementation.
 If a subprogram has a parameter whose type is a protected type or a composite type with a subelement of a
protected type, then the parameter shall be passed by reference and the mode shall be inout.
For a formal parameter of a composite subtype, the index ranges of the formal, if it is an array, and of any
array subelements, are determined as specified in 5.3.2.2. For a formal parameter of mode in or inout, it is
an error if the value of the associated actual parameter (after application of any conversion function or type
conversion present in the actual part) does not contain a matching subelement for each subelement of the
formal. It is also an error if the value of each subelement of the actual (after applying any conversion
function or type conversion present in the actual part) does not belong to the subtype of the corresponding
subelement of the formal. If the formal parameter is of mode out or inout, it is also an error if, at the end of
the subprogram call, the value of each subelement of the formal (after applying any conversion function or
type conversion present in the formal part) does not belong to the subtype of the corresponding subelement
of the actual.
NOTE 1—For parameters of array and record types, the parameter passing rules imply that if no actual parameter of
such a type is accessible by more than one path, then the effect of a subprogram call is the same whether or not the
implementation uses copying for parameter passing. If, however, there are multiple access paths to such a parameter (for
example, if another formal parameter is associated with the same actual parameter), then the value of the formal is
undefined after updating the actual other than by updating the formal. A description using such an undefined value is
erroneous.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
36
Copyright © 2019 IEEE. All rights reserved.
NOTE 2—The value of an actual associated with a formal variable parameter of mode out is not copied into the formal
parameter. Rather, the formal parameter is initialized based on its declared type, regardless of whether the
implementation chooses to pass the parameter by copy or by reference. When a formal variable parameter of mode out is
read, the current value of the formal parameter is read.
##### 4.2.2.3 Signal parameters

For a formal parameter of class signal, references to the signal, the driver of the signal, or both, are passed
into the subprogram call.
For a signal parameter of mode in or inout, the actual signal is associated with the corresponding formal
signal parameter at the start of each call. Thereafter, during the execution of the subprogram body, a
reference to the formal signal parameter within an expression is equivalent to a reference to the actual signal.
It is an error if signal-valued attributes 'STABLE, 'QUIET, 'TRANSACTION, and 'DELAYED of formal
signal parameters of any mode are read within a subprogram.
A process statement contains a driver for each actual signal associated with a formal signal parameter of
mode out or inout in a subprogram call. Similarly, a subprogram contains a driver for each formal signal
parameter of mode out or inout declared in its subprogram specification.
For a signal parameter of mode inout or out, the driver of an actual signal is associated with the
corresponding driver of the formal signal parameter at the start of each call. Thereafter, during the execution
of the subprogram body, an assignment to the driver of a formal signal parameter is equivalent to an
assignment to the driver of the actual signal.
If an actual signal is associated with a signal parameter of any mode, the actual shall be denoted by a static
signal name. It is an error if a conversion function or type conversion appears in either the formal part or the
actual part of an association element that associates an actual signal with a formal signal parameter.
If an actual signal is associated with a signal parameter of mode in or inout, and if the type of the formal is a
scalar type, then it is an error if the subtype of the actual is not compatible with the subtype of the formal.
Similarly, if an actual signal is associated with a signal parameter of mode out or inout, and if the type of the
actual is a scalar type, then it is an error if the subtype of the formal is not compatible with the subtype of the
actual.
For a formal parameter of a composite subtype, the index ranges of the formal, if it is an array, and of any
array subelements, are determined as specified in 5.3.2.2. It is an error if the actual signal does not contain a
matching subelement for each subelement of the formal. It is also an error if the mode of the formal is in or
inout and if the value of each scalar subelement of the actual does not belong to the subtype of the
corresponding subelement of the formal.
A formal signal parameter is a guarded signal if and only if it is associated with an actual signal that is a
guarded signal. It is an error if the declaration of a formal signal parameter includes the reserved word bus
(see 6.5.2).
NOTE—It is a consequence of the preceding rules that a procedure with an out or inout signal parameter called by a
process does not have to complete in order for any assignments to that signal parameter within the procedure to take
effect. Assignments to the driver of a formal signal parameter are equivalent to assignments directly to the actual driver
contained in the process calling the procedure.
##### 4.2.2.4 File parameters

For parameters of class file, references to the actual file are passed into the subprogram. No particular
parameter-passing mechanism is defined by the language, but a reference to the formal parameter shall be
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
37
Copyright © 2019 IEEE. All rights reserved.
equivalent to a reference to the actual parameter. It is an error if an association element associates an actual
with a formal parameter of a file type or a composite type with a subelement of a file type and that
association element contains a conversion function or type conversion. It is also an error if a formal of a file
type is associated with an actual that is not of a file type. Similarly, it is an error if a formal of a composite
type with a subelement of a file type is associated with an actual that is not of a composite type with a
subelement of a file type.
At the beginning of a given subprogram call, a file parameter is open (see 5.5.2) if and only if the actual file
object associated with the given parameter in a given subprogram call is also open. Similarly, at the
beginning of a given subprogram call, both the access mode of and external file associated with (see 5.5.2)
an open file parameter are the same as, respectively, the access mode of and the external file associated with
the actual file object associated with the given parameter in the subprogram call.
At the completion of the execution of a given subprogram call, the actual file object associated with a given
file parameter is open if and only if the formal parameter is also open. Similarly, at the completion of the
execution of a given subprogram call, the access mode of and the external file associated with an open actual
file object associated with a given file parameter are the same as, respectively, the access mode of and the
external file associated with the associated formal parameter.
### 4.3 Subprogram bodies

A subprogram body specifies the execution of a subprogram.
```ebnf
subprogram_body ::=
```

subprogram_specification is
subprogram_declarative_part
begin
subprogram_statement_part
end [ subprogram_kind ] [ designator ];
```ebnf
subprogram_declarative_part ::=
```

{ subprogram_declarative_item }
```ebnf
subprogram_declarative_item ::=
```

subprogram_declaration
|   subprogram_body
    |
subprogram_instantiation_declaration
    |   package_declaration
    |   package_body
    |   package_instantiation_declaration
    |   type_declaration
    |   subtype_declaration
    |   constant_declaration
    |   variable_declaration
    |
file_declaration
    |   alias_declaration
    |   attribute_declaration
    |   attribute_specification
    |   use_clause
    |   group_template_declaration
    |   group_declaration
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
38
Copyright © 2019 IEEE. All rights reserved.
```ebnf
subprogram_statement_part ::=
```

{ sequential_statement }
```ebnf
subprogram_kind ::=  procedure | function
```

The declaration of a subprogram is optional. In the absence of such a declaration, the subprogram
specification of the subprogram body acts as the declaration. For each subprogram declaration, there shall be
a corresponding body. If both a declaration and a body are given, the subprogram specification of the body
shall lexically conform (see 4.10) to the subprogram specification of the declaration. Furthermore, both the
declaration and the body shall occur immediately within the same declarative region (see 12.1).
If a subprogram kind appears at the end of a subprogram body, it shall repeat the reserved word given in the
subprogram specification. If a designator appears at the end of a subprogram body, it shall repeat the
designator of the subprogram.
It is an error if a subprogram declarative part declares a shared variable (see 6.4.2.4).
A foreign subprogram is one that is decorated with the attribute 'FOREIGN, defined in package
STANDARD (see 16.3). The STRING value of the attribute may specify implementation-dependent
information about the foreign subprogram. Foreign subprograms may have non-VHDL implementations. An
implementation may place restrictions on the appearance of a generic list and a generic map aspect in the
declaration of a foreign subprogram. An implementation may also place restrictions on the allowable modes,
classes, and types of the formal parameters to a foreign subprogram; such restrictions may include
restrictions on the number and allowable order of the parameters.
Excepting foreign subprograms, the algorithm performed by a subprogram is defined by the sequence of
statements that appears in the subprogram statement part. For a foreign subprogram, the algorithm
performed is implementation defined.
The execution of a subprogram body, other than an uninstantiated subprogram body, is invoked by a
subprogram call. For this execution, after establishing the association between the formal and actual
parameters, the sequence of statements of the body is executed if the subprogram is not a foreign
subprogram; otherwise, an implementation-defined action occurs. Upon completion of the body or
implementation-dependent action, if exclusive access to an object of a protected type was granted during
elaboration of the declaration of the subprogram (see 14.6), the exclusive access is rescinded. Then, return is
made to the caller (and any necessary copying back of formal to actual parameters occurs).
A process or a subprogram is said to be a parent of a given subprogram S if that process or subprogram
contains a procedure call or function call for S or for a parent of S. An instantiated subprogram is a parent of
a given subprogram S if the uninstantiated subprogram of which the instantiated subprogram is an instance
is a parent of S.
An explicit signal is a signal other than an implicit signal GUARD and other than one of the implicit signals
defined by the predefined attributes 'DELAYED, 'STABLE, 'QUIET, or 'TRANSACTION. The explicit
ancestor of an implicit signal is found as follows. The implicit signal GUARD has no explicit ancestor. An
explicit ancestor of an implicit signal defined by the predefined attributes 'DELAYED, 'STABLE, 'QUIET,
or 'TRANSACTION is the signal found by recursively examining the prefix of the attribute. If the prefix
denotes an explicit signal, a slice, or a member (see Clause 5) of an explicit signal, then that is the explicit
ancestor of the implicit signal. Otherwise, if the prefix is one of the implicit signals defined by the
predefined attributes 'DELAYED, 'STABLE, 'QUIET, or 'TRANSACTION, this rule is recursively applied.
If the prefix is an implicit signal GUARD, then the signal has no explicit ancestor.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
39
Copyright © 2019 IEEE. All rights reserved.
If a pure function subprogram is a parent of a given procedure and if that procedure contains a reference to
an explicitly declared signal or variable object, or a slice, or subelement (or slice thereof) of an explicit
signal, then that object shall be declared within the declarative region formed by the function (see 12.1) or
within the declarative region formed by the procedure; this rule also holds for the explicit ancestor, if any, of
an implicit signal and also for the implicit signal GUARD. If a pure function is the parent of a given
procedure, then that procedure shall not contain a reference to an explicitly declared file object (see 6.4.2.5)
or to a shared variable (see 6.4.2.4).
Similarly, if a pure function subprogram contains a reference to an explicitly declared signal or variable
object, or a slice (or slice thereof) of an explicit signal, then that object shall be declared within the
declarative region formed by the function; this rule also holds for the explicit ancestor, if any, of an implicit
signal and also for the implicit signal GUARD. A pure function shall not contain a reference to an explicitly
declared file object.
A pure function shall not be the parent of an impure function.
The rules of the preceding three paragraphs apply to all pure function subprograms. For pure functions that
are not foreign subprograms, violations of any of these rules are errors. However, since implementations
cannot in general check that such rules hold for pure function subprograms that are foreign subprograms, a
description calling pure foreign function subprograms not adhering to these rules is erroneous.
Example:
—
The declaration of a foreign function subprogram:
package P is
function F return INTEGER;

attribute FOREIGN of F: function is
"implementation-dependent information";
end package P;
NOTE 1—It follows from the visibility rules that a subprogram declaration will be given if a call of the subprogram
occurs textually before the subprogram body, and that such a declaration will occur before the call itself.
NOTE 2—The preceding rules concerning pure function subprograms, together with the fact that pure function
parameters are of mode in, imply that a pure function has no effect other than the computation of the returned value.
Thus, a pure function with static inputs may be invoked either during elaboration or simulation without impacting other
objects in the description.
NOTE 3—VHDL does not define the parameter-passing mechanisms for foreign subprograms.
NOTE 4—The declarative parts and statement parts of subprograms decorated with the 'FOREIGN attribute are subject
to special elaboration rules (see 14.4.1 and 14.6).
NOTE 5—A pure function subprogram will not reference a shared variable. This prohibition exists because a shared
variable cannot be declared in a subprogram declarative part and a pure function cannot reference any variable declared
outside of its declarative region.
NOTE 6—A subprogram containing a wait statement will not have a parent that is a subprogram declared within either a
protected type declaration or a protected type body.
### 4.4 Subprogram instantiation declarations

A subprogram instantiation declaration defines an instance of an uninstantiated subprogram. The instance is
called an instantiated subprogram.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
40
Copyright © 2019 IEEE. All rights reserved.
```ebnf
subprogram_instantiation_declaration ::=
```

subprogram_kind designator is new uninstantiated_subprogram_name [ signature ] [
generic_map_aspect ] ;
The uninstantiated subprogram name shall denote an uninstantiated subprogram declared in a subprogram
declaration. The signature, if present, shall match the parameter and result type profile of exactly one
subprogram denoted by the name, in which case the subprogram instantiation declaration defines an instance
of the uninstantiated subprogram whose parameter and result type profile is matched by the signature. The
subprogram kind shall repeat the reserved word used in the declaration of the uninstantiated subprogram.
The generic map aspect, if present, optionally associates a single actual with each formal generic (or
member thereof) in the corresponding subprogram declaration. Each formal generic (or member thereof)
shall be associated at most once. The generic map aspect is described in 6.5.7.2.
The subprogram instantiation declaration is equivalent to a subprogram declaration and a subprogram body
that jointly define a generic-mapped subprogram. The designator of the generic-mapped subprogram
declaration and subprogram body is the designator of the subprogram instantiation declaration. The
generic-mapped subprogram declaration and subprogram body have the generic list of the uninstantiated
subprogram declaration, the generic map aspect of the subprogram instantiation declaration, and the
parameter list and return type (if appropriate) of the uninstantiated subprogram declaration. The generic-
mapped subprogram body has the declarations and statements of the uninstantiated subprogram body. The
meaning of any identifier appearing anywhere in the generic-mapped subprogram declaration or subprogram
body is that associated with the corresponding occurrence of the identifier in the subprogram instantiation
declaration, the uninstantiated subprogram declaration, or the uninstantiated subprogram body, respectively,
except that an identifier that denotes the uninstantiated subprogram denotes, instead, the generic-mapped
subprogram.
If the subprogram instantiation declaration occurs immediately within an enclosing package declaration, the
generic-mapped subprogram body occurs at the end of the package body corresponding to the enclosing
package declaration. If there is no such body, then there is implicitly a package body corresponding to the
enclosing package declaration, and that implicit body contains the generic-mapped subprogram body. If the
subprogram instantiation declaration occurs immediately within an enclosing protected type declaration, the
generic-mapped subprogram body occurs at the end of the protected type body corresponding to the
enclosing protected type declaration.
NOTE—If two uninstantiated subprograms have the same name and have parameter and result type profiles that include
formal generic types of the uninstantiated subprograms, in addition to other types, a signature can be used to distinguish
between the uninstantiated subprograms, since the formal generic types are made visible by selection in the signatures.
### 4.5 Subprogram overloading

#### 4.5.1 General

Two formal parameter lists are said to have the same parameter type profile if and only if they have the same
number of parameters, and if at each parameter position the corresponding parameters either have the same
base type, or one or both have an unspecified type indication. Two subprograms are said to have the same
parameter and result type profile if and only if both have the same parameter type profile, and if either both
are functions with the same result base type or neither of the two is a function.
A given subprogram designator can be used to designate multiple subprograms. The subprogram designator
is then said to be overloaded; the designated subprograms are also said to be overloaded and to overload
each other. If two subprograms overload each other, one of them can hide the other only if both subprograms
have the same parameter and result type profile.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
41
Copyright © 2019 IEEE. All rights reserved.
A call to an overloaded subprogram is ambiguous (and therefore is an error) if the name of the subprogram,
the number of parameter associations, the types and order of the actual parameters, the names of the formal
parameters (if named associations are used), and the result type (for functions) are not sufficient to identify
exactly one (overloaded) subprogram.
Similarly, a reference to an overloaded resolution function name in a subtype indication is ambiguous (and
is therefore an error) if the name of the function, the number of formal parameters, the result type, and the
relationships between the result type and the types of the formal parameters (as defined in 4.6) are not
sufficient to identify exactly one (overloaded) subprogram specification.
Examples:
—
Declarations of overloaded subprograms:
procedure Dump (F: inout Text; Value: Integer);
procedure Dump (F: inout Text; Value: String);
procedure Check (Setup: Time; signal D: Data; signal C: Clock);
procedure Check (Hold: Time; signal C: Clock; signal D: Data);
—
Calls to overloaded subprograms:
Dump (Sys_Output, 12);
Dump (Sys_Error, "Actual output does not match expected output");
Check (Setup=>10 ns, D=>DataBus, C=>Clk1);
Check (Hold=>5 ns, D=>DataBus, C=>Clk2);
Check (15 ns, DataBus, Clk);
   --  Ambiguous if the base type of DataBus is the same type
   --  as the base type of Clk.
NOTE 1—The notion of parameter and result type profile does not include parameter names, parameter classes,
parameter modes, parameter subtypes, or default expressions or their presence or absence.
NOTE 2—Ambiguities may (but need not) arise when actual parameters of the call of an overloaded subprogram are
themselves overloaded function calls, literals, or aggregates. Ambiguities may also (but need not) arise when several
overloaded subprograms belonging to different packages are visible. These ambiguities can usually be solved in two
ways: qualified expressions can be used for some or all actual parameters and for the result, if any; or the name of the
subprogram can be expressed more explicitly as an expanded name (see 8.4). Further, ambiguities may (but need not)
arise when the declarations of overloaded subprograms in an uninstantiated declaration have parameter and result type
profiles that involve different formal generic types of the uninstantiated declaration. If the declaration is instantiated with
the same actual type associated with the formals, the resulting overloaded subprograms in the instance may have the
same parameter and result type profile. Such ambiguities cannot be solved.
#### 4.5.2 Operator overloading

The declaration of a function whose designator is an operator symbol is used to overload an operator. The
sequence of characters of the operator symbol shall be one of the operators in the operator classes defined in
9.2.
The subprogram specification of a unary operator shall have a single parameter, unless the subprogram
specification is a method (see 5.6.2) of a protected type. In this latter case, the subprogram specification
shall have no parameters. The subprogram specification of a binary operator shall have two parameters,
unless the subprogram specification is a method of a protected type, in which case, the subprogram
specification shall have a single parameter. If the subprogram specification of a binary operator has two
parameters, for each use of this operator, the first parameter is associated with the left operand, and the
second parameter is associated with the right operand.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
42
Copyright © 2019 IEEE. All rights reserved.
For each of the operators “+”, “–”, “and”, “or”, “xor”, “nand”, “nor” and “xnor”, overloading is allowed
both as a unary operator and as a binary operator.
NOTE 1—Overloading of the equality operator does not affect the selection of choices in a case statement in a selected
signal assignment statement, nor does it affect the propagation of signal values.
NOTE 2—A user-defined operator that has the same designator as a short-circuit operator (i.e., a user-defined operator
that overloads the short-circuit operator) is not invoked in a short-circuit manner. Specifically, calls to the user-defined
operator always evaluate both arguments prior to the execution of the function.
NOTE 3—Functions that overload operator symbols may also be called using function call notation rather than operator
notation. This statement is also true of the predefined operators themselves.
Examples:
type MVL is ('0', '1', 'Z', 'X');
type MVL_Vector is array (Natural range <>) of MVL;
function "and" (Left, Right: MVL) return MVL;
function "or" (Left, Right: MVL) return MVL;
function "not" (Value: MVL) return MVL;
function "xor" (Right: MVL_Vector) return MVL;
signal Q,R,S,T: MVL;
signal V: MVL_Vector(0 to 3);
Q <= 'X' or '1';
R <= "or" ('0','Z');
S <= (Q and R) or not S;
T <= xor V;
#### 4.5.3 Signatures

A signature distinguishes between overloaded subprograms and overloaded enumeration literals based on
their parameter and result type profiles. A signature can be used in a subprogram instantiation declaration,
generic map aspect, attribute name, entity designator, or alias declaration.
```ebnf
signature ::=  [ [ type_mark { , type_mark } ] [ return type_mark ] ]
```

(Note that the initial and terminal brackets are part of the syntax of signatures and do not indicate that the
entire right-hand side of the production is optional.) A signature is said to match the parameter and the result
type profile of a given subprogram if, and only if, all of the following conditions hold:
—
The number of type marks prior to the reserved word return, if any, matches the number of formal
parameters of the subprogram.
—
At each parameter position, the base type denoted by the type mark of the signature is either the same
as the base type of the corresponding formal parameter of the subprogram or the subtype indication
of the formal parameter is an unspecified type indication.
—
If the reserved word return is present, the subprogram is a function and the base type of the type
mark following the reserved word in the signature is the same as the base type of the return type of
the function, or the reserved word return is absent and the subprogram is a procedure.
Similarly, a signature is said to match the parameter and result type profile of a given enumeration literal if
the signature matches the parameter and result type profile of the subprogram equivalent to the enumeration
literal defined in 5.2.2.1.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
43
Copyright © 2019 IEEE. All rights reserved.
Example:
attribute BuiltIn of "or" [MVL, MVL return MVL]: function is TRUE;
-- Because of the presence of the signature, this attribute
-- specification decorates only the "or" function defined in
-- Clause 4.5.2.
attribute Mapping of JMP [return OpCode]: literal is "001";
### 4.6 Resolution functions

A resolution function is a function that defines how the values of multiple sources of a given signal are to be
resolved into a single value for that signal. Resolution functions are associated with signals that require
resolution by including the name of the resolution function in the declaration of the signal or in the
declaration of the subtype of the signal. A signal with an associated resolution function is called a resolved
signal (see 6.4.2.3).
A resolution function shall be a pure function other than an uninstantiated function (see 4.2.1); moreover, it
shall have a single input parameter of class constant that is a one-dimensional, unconstrained or partially
constrained array with an undefined index range and whose element type is that of the associated subtype or
subelement subtype in the subtype indication in which the name of the resolution function appears. The
resolution function name shall not be an attribute name (see 8.7). The type of the return value of the function
shall also be that of the associated subtype or subelement subtype in the subtype indication in which the
name of the resolution function appears. Errors occur at the place of the subtype indication containing the
name of the resolution function if any of these checks fail (see 6.3).
The resolution function associated with a resolved signal determines the resolved value of the signal as a
function of the collection of inputs from its multiple sources. If a resolved signal is of a composite type, and
if subelements of that type also have associated resolution functions, such resolution functions have no
effect on the process of determining the resolved value of the signal. It is an error if a resolved signal has
more connected sources than the number of elements in the index type of the unconstrained array type used
to define the parameter of the corresponding resolution function.
Resolution functions are implicitly invoked during each simulation cycle in which corresponding resolved
signals are active (see 14.7.3.1). Each time a resolution function is invoked, it is passed an array value, each
element of which is determined by a corresponding source of the resolved signal, but excluding those
sources that are drivers whose values are determined by null transactions (see 10.5.2.2). Such drivers are
said to be off. For certain invocations (specifically, those involving the resolution of sources of a signal
declared with the signal kind bus), a resolution function may thus be invoked with an input parameter that is
a null array; this occurs when all sources of the bus are drivers, and they are all off. In such a case, the
resolution function returns a value representing the value of the bus when no source is driving it.
Example:
function WIRED_OR (Inputs: BIT_VECTOR) return BIT is
constant FloatValue: BIT := '0';
begin
if Inputs'Length = 0 then--  This is a bus whose drivers are all
off.
return FloatValue;
else
for I in Inputs'Range loop
if Inputs(I) = '1' then
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
44
Copyright © 2019 IEEE. All rights reserved.
return '1';
end if;
end loop;
return '0';
end if;
end function WIRED_OR;
### 4.7 Package declarations

A package declaration defines the interface to a package. The scope of a declaration within a package can be
extended to other design units or to other parts of the design unit containing the package declaration.
```ebnf
package_declaration ::=
```

package identifier is
package_header
package_declarative_part
end [ package ] [ package_simple_name ] ;
```ebnf
package_header ::=
```

[ generic_clause
[ generic_map_aspect ; ] ]
```ebnf
package_declarative_part ::=
```

{ package_declarative_item }
```ebnf
package_declarative_item ::=
```

subprogram_declaration
    |   subprogram_instantiation_declaration
    |   package_declaration
    |   package_instantiation_declaration
    |   type_declaration
    |  subtype_declaration
    |   mode_view_declaration
    |   constant_declaration
    |   signal_declaration
    |   shared_variable_declaration
|   file_declaration
|   alias_declaration
|   component_declaration
|   attribute_declaration
|   attribute_specification
|   disconnection_specification
|   use_clause
|   group_template_declaration
|   group_declaration
|   PSL_Property_Declaration
|  PSL_Sequence_Declaration
If a simple name appears at the end of the package declaration, it shall repeat the identifier of the package
declaration.
If the package header is empty, the package declared by a package declaration is called a simple package. If
the package header contains a generic clause and no generic map aspect, the package is called an
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
45
Copyright © 2019 IEEE. All rights reserved.
uninstantiated package. If the package header contains both a generic clause and a generic map aspect, the
package is called a generic-mapped package. A package declared with a generic clause in which every
generic declaration has a default, and with no generic map aspect, is considered to be an uninstantiated
package, not a generic-mapped package with default associations for all of the generic declarations.
If a package declarative item is a full type declaration whose type definition is a protected type definition,
then that protected type definition shall not be a protected type body.
Items declared immediately within a simple or a generic-mapped package declaration become visible by
selection within a given design unit wherever the name of that package is visible in the given unit. Such
items may also be made directly visible by an appropriate use clause (see 12.4). Items declared immediately
within an uninstantiated package declaration cannot be made visible outside of the package.
For a package declaration that appears in a subprogram body, a process statement, or a protected type body,
it is an error if a shared variable is declared in the package declarative part. Moreover, it is an error if a signal
declaration, a disconnection specification, or a PSL declaration appears as a package declarative item of
such a package declaration.
NOTE—Not all packages will have a package body. In particular, a package body is unnecessary if no subprograms,
deferred constants, or protected type definitions are declared in the package declaration.
Examples:
—
A package declaration that needs no package body:
package TimeConstants is
constant tPLH: Time := 10 ns;
constant tPHL: Time := 12 ns;
constant tPLZ: Time := 7 ns;
constant tPZL: Time := 8 ns;
constant tPHZ: Time := 8 ns;
constant tPZH: Time := 9 ns;
end TimeConstants;
—
A package declaration that needs a package body:
package TriState is
type Tri is ('0', '1', 'Z', 'E');
function BitVal (Value: Tri) return Bit;
function TriVal (Value: Bit) return Tri;
type TriVector is array (Natural range <>) of Tri;
function Resolve (Sources: TriVector) return Tri;
end package TriState;
### 4.8 Package bodies

A package body defines the bodies of subprograms and the values of deferred constants declared in the
interface to the package.
```ebnf
package_body ::=
```

package body package_simple_name is
package_body_declarative_part
end [ package body ] [ package_simple_name ] ;
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
46
Copyright © 2019 IEEE. All rights reserved.
```ebnf
package_body_declarative_part ::=
```

{ package_body_declarative_item }
```ebnf
package_body_declarative_item ::=
```

subprogram_declaration
|   subprogram_body
|   subprogram_instantiation_declaration
    |   package_declaration
    |
package_body
    |
package_instantiation_declaration
    |
type_declaration
    |
subtype_declaration
    |
constant_declaration
|
shared_variable_declaration
    |
file_declaration
    |
alias_declaration
|   attribute_declaration
|   attribute_specification
|   use_clause
    |   group_template_declaration
    |   group_declaration
The simple name at the start of a package body shall repeat the package identifier. If a simple name appears
at the end of the package body, it shall be the same as the identifier in the package declaration.
A package body that is not a library unit shall appear immediately within the same declarative region as the
corresponding package declaration and textually subsequent to that package declaration.
For a package body that appears in a subprogram body, a process statement or a protected type body, it is an
error if a shared variable is declared in the package declarative part.
In addition to subprogram body and constant declarative items, a package body may contain certain other
declarative items to facilitate the definition of the bodies of subprograms declared in the interface. Items
declared in the body of a package cannot be made visible outside of the package body.
If a given package declaration contains a deferred constant declaration (see 6.4.2.2), then a constant
declaration with the same identifier shall appear as a declarative item in the corresponding package body.
This object declaration is called the full declaration of the deferred constant. The subtype indication given in
the full declaration shall lexically conform to that given in the deferred constant declaration.
Within a package declaration that contains the declaration of a deferred constant, and within the body of that
package (before the end of the corresponding full declaration), the use of a name that denotes the deferred
constant is only allowed in the default expression for a local generic, local port, or formal parameter. The
result of evaluating an expression that references a deferred constant before the elaboration of the
corresponding full declaration is not defined by the language.
Example:
package body TriState is
function BitVal (Value: Tri) return Bit is
constant Bits : Bit_Vector := "0100";
begin
return Bits(Tri'Pos(Value));
end;
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
47
Copyright © 2019 IEEE. All rights reserved.
    function TriVal (Value: Bit) return Tri is
    begin

return Tri'Val(Bit'Pos(Value));
    end;
function Resolve (Sources: TriVector) return Tri is
        variable V: Tri := 'Z';
    begin

for i in Sources'Range loop

if Sources(i) /= 'Z' then

if V = 'Z' then

V := Sources(i);
             else

return 'E';
end if;

end if;
end loop;
return V;
end;
end package body TriState;
### 4.9 Package instantiation declarations

A package instantiation declaration defines an instance of an uninstantiated package. The instance is called
an instantiated package.
```ebnf
package_instantiation_declaration ::=
```

package identifier is new uninstantiated_package_name
[ generic_map_aspect ] ;
The uninstantiated package name shall denote an uninstantiated package declared in a package declaration.
The generic map aspect, if present, optionally associates a single actual with each formal generic (or
member thereof) in the corresponding package declaration. Each formal generic (or member thereof) shall
be associated at most once. The generic map aspect is described in 6.5.7.2.
The package instantiation declaration is equivalent to declaration of a generic-mapped package, consisting
of a package declaration and possibly a corresponding package body. The simple name of the generic-
mapped package declaration is the identifier of the package instantiation declaration. The generic-mapped
package declaration has the generic clause of the uninstantiated package declaration, the generic map aspect
of the package instantiation declaration, and the declarations of the uninstantiated package declaration. The
package body corresponding to the generic-mapped package is present if the uninstantiated package has a
package body. In that case, the simple name of the generic-mapped package body is the identifier of the
package instantiation declaration, and the declarations of the generic-mapped package body are the
declarations of the uninstantiated package body. The meaning of any identifier appearing anywhere in the
generic-mapped package declaration or package body is that associated with the corresponding occurrence
of the identifier in the package instantiation declaration, the uninstantiated package declaration, or the
uninstantiated package body, respectively, except that an identifier that denotes the uninstantiated package
denotes, instead, the generic-mapped package.
If the package instantiation declaration occurs immediately within an enclosing package declaration and the
uninstantiated package has a package body, the generic-mapped package body occurs at the end of the
package body corresponding to the enclosing package declaration. If there is no such body, then there is
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
48
Copyright © 2019 IEEE. All rights reserved.
implicitly a package body corresponding to the enclosing package declaration, and that implicit body
contains the generic-mapped package body.
### 4.10 Conformance rules

Whenever the language rules either require or allow the specification of a given subprogram to be provided
in more than one place, the following variations are allowed at each place:
—
A numeric literal can be replaced by a different numeric literal if and only if both have the same
value.
—
A simple name can be replaced by an expanded name in which this simple name is the suffix if, and
only if, at both places the meaning of the simple name is given by the same declaration.
Two subprogram specifications are said to lexically conform if, apart from comments and the preceding
allowed variations, both specifications are formed by the same sequence of lexical elements and if
corresponding lexical elements are given the same meaning by the visibility rules.
Lexical conformance is likewise defined for subtype indications in deferred constant declarations. Two
subprogram declarations are said to have conforming profiles if and only if both are procedures or both are
functions, the parameter and result type profiles of the subprograms are the same and, at each parameter
position, the corresponding parameters have the same class and mode.
NOTE 1—A simple name can be replaced by an expanded name even if the simple name is itself the prefix of a selected
name. For example, Q.R can be replaced by P.Q.R if Q is declared immediately within P.
NOTE 2—The subprogram specification of an impure function is never lexically conformant to a subprogram
specification of a pure function.
NOTE 3—The following specifications do not lexically conform since they are not formed by the same sequence of
lexical elements:
procedure P (X,Y: INTEGER)
procedure P (X: INTEGER; Y: INTEGER)
procedure P (X,Y: in INTEGER)
NOTE 4—Conformance of profiles is required for formal and actual generic subprograms (see 6.5.4).
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
