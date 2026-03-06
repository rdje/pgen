---
title: "Section 7: IEEE Standard for VHDL Language Reference Manual"
document: "VHDL Language Reference Manual"
standard: "IEEE 1076-2019"
domain: "VHDL"
section: "7"
source_txt: "section-7-specifications.txt"
source_pdf: "/Users/richarddje/Documents/github/VHDL-LRM-IEEE-1076-2019.pdf"
---

# Section 7: IEEE Standard for VHDL Language Reference Manual

IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
126
Copyright © 2019 IEEE. All rights reserved.
7. Specifications
### 7.1 General

This clause describes specifications, which may be used to associate additional information with a VHDL
description. A specification associates additional information with a named entity that has been previously
declared. There are three kinds of specifications: attribute specifications, configuration specifications, and
disconnection specifications.
A specification always relates to named entities that already exist; thus a given specification shall either
follow or (in certain cases) be contained within the declaration of the entity to which it relates. Furthermore,
a specification shall always appear either immediately within the same declarative part as that in which the
declaration of the named entity appears, or (in the case of specifications that relate to design units or the
interface objects of design units, subprograms, or block statements) immediately within the declarative part
associated with the declaration of the design unit, subprogram body, or block statement.
### 7.2 Attribute specification

An attribute specification associates a user-defined attribute with one or more named entities and defines the
value of that attribute for those entities. The attribute specification is said to decorate the named entity.
```ebnf
attribute_specification ::=
    attribute attribute_designator of entity_specification is conditional_expression ;
entity_specification ::=
    entity_name_list : entity_class
entity_class ::=
        entity
    |   architecture
    |   configuration
    |   procedure
    |   function
    |   package
    |   type
    |   subtype
    |   constant
    |   signal
    |   variable
    |   component
    |   label
    |   literal
    |   units
    |   group
    |   file
    |   property
    |   sequence
    |   view
entity_name_list ::=
        entity_designator { , entity_designator }
    |   others
```

Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
127
Copyright © 2019 IEEE. All rights reserved.
    |   all
```ebnf
entity_designator ::=  entity_tag [ signature ]
entity_tag ::=  simple_name | character_literal | operator_symbol
```

The attribute designator shall denote an attribute. The entity name list identifies those named entities, both
implicitly and explicitly defined, that inherit the attribute, described as follows:
—
If a list of entity designators is supplied, then the attribute specification applies to the named entities
that are denoted by those designators and are of the specified class. It is an error if any entity
designator denotes no named entity of the specified class.
—
If the reserved word others is supplied, then the attribute specification applies to named entities of
the specified class that are declared in the immediately enclosing declarative part, provided that each
such entity is not explicitly named in the entity name list of a previous attribute specification for the
given attribute.
—
If the reserved word all is supplied, then the attribute specification applies to all named entities of the
specified class that are declared in the immediately enclosing declarative part.
An attribute specification with the entity name list others or all for a given entity class that appears in a
declarative part shall be the last such specification for the given attribute for the given entity class in that
declarative part. It is an error if a named entity in the specified entity class is declared in a given declarative
part following such an attribute specification.
If a name in an entity name list denotes a subprogram or package, it denotes the subprogram declaration or
package declaration. Subprogram and package bodies cannot be decorated.
An entity designator that denotes an alias of an object is required to denote the entire object, not a member of
an object.
The entity tag of an entity designator containing a signature shall denote the name of one or more
subprograms or enumeration literals. In this case, the signature shall match (see 4.5.3) the parameter and
result type profile of exactly one subprogram or enumeration literal in the current declarative part: the
enclosing attribute specification then decorates that subprogram or enumeration literal.
The expression specifies the value of this attribute for each of the named entities inheriting the attribute as a
result of this attribute specification. The type of the expression in the attribute specification shall be the same
as (or implicitly convertible to) the type mark in the corresponding attribute declaration. If the entity name
list denotes an entity declaration, architecture body, configuration declaration, or an uninstantiated package
that is declared as a design unit, then the expression is required to be locally static (see 9.4.1). Similarly, if
the entity name list denotes a subprogram and the attribute designator denotes the 'FOREIGN attribute
defined in package STANDARD, then the expression is required to be locally static.
An attribute specification for an attribute of an entity declaration, an architecture, a configuration, or a
package shall appear immediately within the declarative part of that declaration. Similarly, an attribute
specification for an attribute of an interface object of a design unit, subprogram, block statement, or package
shall appear immediately within the declarative part of that design unit, subprogram, block statement, or
package.
An attribute specification for an attribute of a procedure, a function, a type, a subtype, an object (i.e., a
constant, a file, a signal, or a variable), a component, literal, unit name, group, property, sequence, or a
labeled entity shall appear within the declarative part in which that procedure, function, type, subtype,
object, component, literal, unit name, group, property, sequence, or label, respectively, is explicitly or
implicitly declared.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
128
Copyright © 2019 IEEE. All rights reserved.
For a given named entity, the value of a user-defined attribute of that entity is the value specified in an
attribute specification for that attribute of that entity.
It is an error if a given attribute is associated more than once with a given named entity. Similarly, it is an
error if two different attributes with the same simple name (whether predefined or user-defined) are both
associated with a given named entity.
An entity designator that is a character literal is used to associate an attribute with one or more character
literals. An entity designator that is an operator symbol is used to associate an attribute with one or more
overloaded operators.
If the entity tag is overloaded and the entity designator does not contain a signature, all named entities
already declared in the current declarative part and matching the specification are decorated.
If an attribute specification appears, it shall follow the declaration of the named entity with which the
attribute is associated, and it shall precede all references to that attribute of that named entity. Attribute
specifications are allowed for all user-defined attributes, but are not allowed for predefined attributes.
An attribute specification may reference a named entity by using an alias for that entity in the entity name
list, but such a reference counts as the single attribute specification that is allowed for a given attribute and
therefore prohibits a subsequent specification that uses the declared name of the entity (or any other alias) as
the entity designator.
An attribute specification whose entity designator contains no signature and identifies an overloaded subpro-
gram or enumeration literal has the effect of associating that attribute with each of the designated overloaded
subprograms or enumeration literals declared within that declarative part.
Examples:
attribute PIN_NO of CIN: signal is 10;
attribute PIN_NO of COUT: signal is 5;
attribute LOCATION of ADDER1: label is (10,15);
attribute LOCATION of others: label is (25,77);
attribute CAPACITANCE of all: signal is 15 pF;
attribute IMPLEMENTATION of G1: group is "74LS152";
attribute RISING_DELAY of C2Q: group is 7.2 ns;
NOTE 1—A given named entity E will be decorated with the user-defined attribute A if and only if an attribute
specification for the value of attribute A exists in the same declarative part as the declaration of E. In the absence of such
a specification, an attribute name of the form E'A is illegal.
NOTE 2—A user-defined attribute is associated with the named entity denoted by the name specified in an attribute
specification, not with the name itself. Hence, an attribute of an object can be referenced by using an alias for that object
rather than the declared name of the object as the prefix of the attribute name, and the attribute referenced in such a way
is the same attribute (and therefore has the same value) as the attribute referenced by using the declared name of the
object as the prefix.
NOTE 3—A user-defined attribute of a port, signal, variable, or constant of some composite type is an attribute of the
entire port, signal, variable, or constant, not of its elements. If it is necessary to associate an attribute with each element
of some composite object, then the attribute itself can be declared to be of a composite type such that for each element of
the object, there is a corresponding element of the attribute.
NOTE 4—User-defined attributes represent local information only and cannot be used to pass information from one
description to another. For instance, assume some signal X in an architecture body has some attribute A. Further, assume
that X is associated with some local port L of component C. C in turn is associated with some design entity E(B), and L
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
129
Copyright © 2019 IEEE. All rights reserved.
is associated with E’s formal port P. Neither L nor P has attributes with the simple name A, unless such attributes are
supplied via other attribute specifications; in this latter case, the values of P'A and X'A are not related in any way.
NOTE 5—The local ports and generics of a component declaration cannot be decorated, since component declarations
lack a declarative part.
NOTE 6—If an attribute specification applies to an overloadable named entity, then declarations of additional named
entities with the same simple name are allowed to occur in the current declarative part unless the aforementioned
attribute specification has as its entity name list either of the reserved words others or all.
NOTE 7—Attribute specifications supplying either of the reserved words others or all never apply to the interface
objects of design units, block statements, or subprograms.
NOTE 8—An attribute specification supplying either of the reserved words others or all may apply to none of the
named entities in the current declarative part, in the event that none of the named entities in the current declarative part
meet all of the requirements of the attribute specification.
NOTE 9—An enumeration literal is of class literal, not function.
### 7.3 Configuration specification

#### 7.3.1 General

A configuration specification associates binding information with component labels representing instances
of a given component declaration.
```ebnf
configuration_specification ::=
        simple_configuration_specification
    |   compound_configuration_specification
simple_configuration_specification ::=
    for component_specification binding_indication ;
        [ end for ; ]
compound_configuration_specification ::=
    for component_specification binding_indication ;
        verification_unit_binding_indication ;
        { verification_unit_binding_indication ; }
    end for ;
component_specification ::=
    instantiation_list : component_name
instantiation_list ::=
        instantiation_label { , instantiation_label }
    |    others
    |    all
```

The instantiation list identifies those component instances with which binding information is to be
associated, defined as follows:
—
If a list of instantiation labels is supplied, then the configuration specification applies to the
corresponding component instances. Such labels shall be (implicitly) declared within the
immediately enclosing declarative part. It is an error if these component instances are not instances
of the component declaration named in the component specification. It is also an error if any of the
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
130
Copyright © 2019 IEEE. All rights reserved.
labels denote a component instantiation statement whose corresponding instantiated unit does not
name a component.
—
If the reserved word others is supplied, then the configuration specification applies to instances of
the specified component declaration whose labels are (implicitly) declared in the immediately
enclosing declarative part, provided that each such component instance is not explicitly named in the
instantiation list of a previous configuration specification. This rule applies only to those component
instantiation statements whose corresponding instantiated units name components.
—
If the reserved word all is supplied, then the configuration specification applies to all instances of the
specified component declaration whose labels are (implicitly) declared in the immediately enclosing
declarative part. This rule applies only to those component instantiation statements whose
corresponding instantiated units name components.
A configuration specification with the instantiation list others or all for a given component name that
appears in a declarative part shall be the last such specification for the given component name in that
declarative part.
The elaboration of a configuration specification results in the association of binding information with the
labels identified by the instantiation list. A label that has binding information associated with it, specified by
a binding indication, is said to be bound. It is an error if the elaboration of a configuration specification
results in the association of binding information with a component label that is already bound, unless the
binding indication in the configuration specification is an incremental binding indication (see 7.3.2.1). It is
also an error if the elaboration of a configuration specification containing an incremental binding indication
results in the association of binding information with a component label that is already incrementally bound.
NOTE—A configuration specification supplying either of the reserved words others or all may apply to none of the
component instances in the current declarative part. This is the case when none of the component instances in the current
declarative part meet all of the requirements of the given configuration specification.
#### 7.3.2 Binding indication

##### 7.3.2.1 General

A binding indication associates instances of a component with a particular design entity. It may also
associate actuals with formals declared in the entity declaration.
```ebnf
binding_indication ::=
    [ use entity_aspect ]
    [ generic_map_aspect ]
    [ port_map_aspect ]
```

The entity aspect of a binding indication, if present, identifies the design entity with which the instances of a
component are associated. If present, the generic map aspect of a binding indication identifies the
expressions, subtypes, subprograms, or instantiated packages to be associated with formal generics in the
entity declaration. Similarly, the port map aspect of a binding indication identifies the signals or values to be
associated with formal ports in the entity declaration.
When a binding indication is used in an explicit configuration specification, it is an error if the entity aspect
is absent.
A binding indication appearing in a component configuration shall have an entity aspect unless the block
corresponding to the block configuration in which the given component configuration appears has one or
more configuration specifications that together configure all component instances denoted in the given
component configuration. The binding indications appearing in these configuration specifications are the
corresponding primary binding indications. A binding indication need not have an entity aspect; in that case,
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
131
Copyright © 2019 IEEE. All rights reserved.
either or both of a generic map aspect or a port map aspect shall be present in the binding indication. Such a
binding indication is an incremental binding indication. An incremental binding indication is used to
incrementally rebind the ports and generic constants of the denoted instance(s) under the following
conditions:
—
For each formal generic constant appearing in the generic map aspect of the incremental binding
indication and denoting a formal generic constant that is unassociated or associated with open in any
of the primary binding indications, the given formal generic constant is bound to the actual with
which it is associated in the generic map aspect of the incremental binding indication.
—
For each formal generic constant appearing in the generic map aspect of the incremental binding
indication and denoting a formal generic constant that is associated with an actual other than open in
one of the primary binding indications, the given formal generic constant is rebound to the actual
with which it is associated in the generic map aspect of the incremental binding indication. That is,
the association given in the primary binding indication has no effect for the given instance.
—
For each formal port appearing in the port map aspect of the incremental binding indication and
denoting a formal port that is unassociated or associated with open in any of the primary binding
indications, the given formal port is bound to the actual with which it is associated in the port map
aspect of the incremental binding indication.
It is an error if a formal port appears in the port map aspect of the incremental binding indication and it is a
formal port that is associated with an actual other than open in one of the primary binding indications.
If the generic map aspect or port map aspect of a primary binding indication is not present, then the default
rules as described in 7.3.3 apply.
It is an error if an explicit entity aspect in an incremental binding indication does not adhere to any of the fol-
lowing rules:
—
If the entity aspect in the corresponding primary binding indication is of the first form (fully bound),
as specified in 7.3.2.2, then the entity aspect in the incremental binding indication shall also be of
the first form and shall denote the same entity declaration as that of the primary binding indication.
An architecture name shall be specified in the incremental binding indication if and only if the
primary binding indication also identifies an architecture name; in this case, the architecture name in
the incremental binding indication shall denote the same architecture name as that of the primary
binding indication.
—
If the entity aspect in the primary binding indication is of the second form (that is, identifying a
configuration), then the entity aspect of the incremental binding indication shall be of the same form
and shall denote the same configuration declaration as that of the primary binding indication.
NOTE 1—The third form (open) of an entity aspect does not apply to incremental binding indications as this form
cannot include either a generic map aspect or a port map aspect and incremental binding indications will contain at least
one of these aspects.
NOTE 2—The entity aspect of an incremental binding indication in a component configuration is optional.
NOTE 3—The presence of an incremental binding indication will never cause the default rules of 7.3.3 to be applied.
Examples:
entity AND_GATE is
generic (I1toO, I2toO: DELAY_LENGTH := 4 ns);
port (I1, I2: in BIT := '1'; O: out BIT);
end entity AND_GATE;
entity XOR_GATE is
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
132
Copyright © 2019 IEEE. All rights reserved.
generic (I1toO, I2toO: DELAY_LENGTH := 4 ns);
port (I1, I2: in BIT; O: out BIT);
end entity XOR_GATE;
package MY_GATES is
component AND_GATE is
generic (I1toO, I2toO: DELAY_LENGTH := 4 ns);
port (I1, I2: in BIT := '1'; O: out BIT);
end component AND_GATE;
component XOR_GATE is
 generic (I1toO, I2toO: DELAY_LENGTH := 4 ns);

 port (I1, I2: in BIT; O: out BIT);
end component XOR_GATE;
end package MY_GATES;
entity Half_Adder is
port (X, Y: in BIT; Sum, Carry: out BIT);
end entity Half_Adder;
use WORK.MY_GATES.all;
architecture Structure of Half_Adder is
   for L1: XOR_GATE use
      entity WORK.XOR_GATE(Behavior)    --  The primary binding
         generic map (3 ns, 3 ns)       --  indication for instance L1.
         port map (I1 => I1, I2 => I2, O => O);
   for L2: AND_GATE use
      entity WORK.AND_GATE(Behavior)    --  The primary binding
         generic map (3 ns, 4 ns)       --  indication for instance L2.
         port map (I1, open, O);
begin
   L1: XOR_GATE port map (X, Y, Sum);
   L2: AND_GATE port map (X, Y, Carry);
end architecture Structure;
use WORK.GLOBAL_SIGNALS.all;
configuration Different of Half_Adder is
   for Structure
      for L1: XOR_GATE
         generic map (2.9 ns, 3.6 ns);  --  The incremental binding
      end for;                          --  indication of L1; rebinds
                                        --  its generics.
      for L2: AND_GATE
         generic map (2.8 ns, 3.25 ns)  --  The incremental binding
         port map (I2 => Tied_High);    --  indication of L2; rebinds
      end for;                          --  its generics and binds
                                        --  its open port.
   end for;
end configuration Different;
##### 7.3.2.2 Entity aspect

An entity aspect identifies a particular design entity to be associated with instances of a component. An
entity aspect may also specify that such a binding is to be deferred.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
133
Copyright © 2019 IEEE. All rights reserved.
```ebnf
entity_aspect ::=
```

entity entity_name [ ( architecture_identifier ) ]
    |
configuration configuration_name
    |
open
The first form of entity aspect identifies a particular entity declaration and (optionally) a corresponding
architecture body. If no architecture identifier appears, then the immediately enclosing binding indication is
said to imply the design entity whose interface is defined by the entity declaration denoted by the entity
name and whose body is defined by the default binding rules for architecture identifiers (see 7.3.3). If an
architecture identifier appears, then the immediately enclosing binding indication is said to imply the design
entity consisting of the entity declaration denoted by the entity name together with an architecture body
associated with the entity declaration; the architecture identifier defines a simple name that is used during
the elaboration of a design hierarchy to select the appropriate architecture body. In either case, the
corresponding component instances are said to be fully bound.
At the time of the analysis of an entity aspect of the first form, the library unit corresponding to the entity
declaration denoted by the entity name is required to exist; moreover, the design unit containing the entity
aspect depends on the denoted entity declaration. If the architecture identifier is also present, the library unit
corresponding to the architecture identifier is required to exist only if the binding indication is part of a
component configuration containing explicit block configurations or explicit component configurations;
only in this case does the design unit containing the entity aspect also depend on the denoted architecture
body. In any case, the library unit corresponding to the architecture identifier is required to exist at the time
that the design entity implied by the enclosing binding indication is bound to the component instance
denoted by the component configuration or configuration specification containing the binding indication; if
the library unit corresponding to the architecture identifier was required to exist during analysis, it is an error
if the architecture identifier does not denote the same library unit as that denoted during analysis. The library
unit corresponding to the architecture identifier, if it exists, shall be an architecture body associated with the
entity declaration denoted by the entity name.
The second form of entity aspect identifies a design entity indirectly by identifying a configuration. In this
case, the entity aspect is said to imply the design entity at the root of the design hierarchy that is defined by
the configuration denoted by the configuration name.
At the time of the analysis of an entity aspect of the second form, the library unit corresponding to the
configuration name is required to exist. The design unit containing the entity aspect depends on the
configuration denoted by the configuration name.
The third form of entity aspect is used to specify that the identification of the design entity is to be deferred.
In this case, the immediately enclosing binding indication is said to not imply any design entity.
Furthermore, the immediately enclosing binding indication shall not include a generic map aspect or a port
map aspect.
#### 7.3.3 Default binding indication

In certain circumstances, a default binding indication will apply in the absence of an explicit binding
indication. The default binding indication consists of a default entity aspect, together with a default generic
map aspect and a default port map aspect, as appropriate.
If no visible entity declaration has the same simple name as that of the instantiated component, then the
default entity aspect is open. A visible entity declaration is the first entity declaration, if any, in the
following list:
a)
An entity declaration that has the same simple name as that of the instantiated component and that is
directly visible (see 12.3),
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
134
Copyright © 2019 IEEE. All rights reserved.
b)
An entity declaration that has the same simple name as that of the instantiated component and that
would be directly visible in the absence of a directly visible (see 12.3) component declaration with
the same simple name as that of the entity declaration, or
c)
An entity declaration denoted by L.C, where L is the target library and C is the simple name of the
instantiated component. The target library is the library logical name of the library containing the
design unit in which the component C is declared.
These visibility checks are made at the point of the absent explicit binding indication that causes the default
binding indication to apply.
Otherwise, the default entity aspect is of the form
entity entity_name ( architecture_identifier )
where the entity name is the simple name of the instantiated component, and the architecture identifier is the
same as the simple name of the most recently analyzed architecture body associated with the entity
declaration. If this rule is applied either to a binding indication contained within a configuration
specification or to a component configuration that does not contain an explicit inner block configuration,
then the architecture identifier is determined during elaboration of the design hierarchy containing the
binding indication. Likewise, if a component instantiation statement contains an instantiated unit containing
the reserved word entity but does not contain an explicitly specified architecture identifier, this rule is
applied during the elaboration of the design hierarchy containing a component instantiation statement. In all
other cases, this rule is applied during analysis of the binding indication.
It is an error if there is no architecture body associated with the entity declaration denoted by an entity name
that is the simple name of the instantiated component.
The default binding indication includes a default generic map aspect if the design entity implied by the entity
aspect contains formal generics. The default generic map aspect associates each local generic in the
corresponding component instantiation (if any) with a formal of the same simple name. It is an error if such
a formal does not exist or if its mode and type are not appropriate for such an association. Any remaining
unassociated formals are associated with the actual designator open.
The default binding indication includes a default port map aspect if the design entity implied by the entity
aspect contains formal ports. The default port map aspect associates each local port in the corresponding
component instantiation (if any) with a formal of the same simple name. It is an error if such a formal does
not exist or if its mode and type are not appropriate for such an association. Any remaining unassociated
formals are associated with the actual designator open.
If an explicit binding indication lacks a generic map aspect, and if the design entity implied by the entity
aspect contains formal generics, then the default generic map aspect is assumed within that binding
indication. Similarly, if an explicit binding indication lacks a port map aspect, and the design entity implied
by the entity aspect contains formal ports, then the default port map aspect is assumed within that binding
indication.
#### 7.3.4 Verification unit binding indication

A verification unit binding indication binds one or more PSL verification units to the design entity bound to
a component instance.
```ebnf
verification_unit_binding_dication ::=
    use vunit verification_unit_list
verification_unit_list ::= verification_unit_name { , verification_unit_name }
```

Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
135
Copyright © 2019 IEEE. All rights reserved.
Each name in a verification unit list shall denote a PSL verification unit (see 13.1 and IEEE Std 1850-2010).
It is an error if a PSL verification unit bound to a design entity by a configuration specification, whether
explicit or implicit, is explicitly bound by its declaration (see IEEE Std 1850-2010). It is an error if a
verification unit binding indication is specified for a component instance that is unbound or that is bound by
a binding indication that has an entity aspect of the third form (open).
### 7.4 Disconnection specification

A disconnection specification defines the time delay to be used in the implicit disconnection of drivers of a
guarded signal within a guarded signal assignment.
```ebnf
disconnection_specification ::=
    disconnect guarded_signal_specification after time_expression ;
guarded_signal_specification ::=
    guarded_signal_list : type_mark
signal_list ::=
        signal_name { , signal_name }
    |   others
    |   all
```

Each signal name in a signal list in a guarded signal specification shall be a locally static name that denotes
a guarded signal (see 6.4.2.3). Each guarded signal shall be an explicitly declared signal or member of such
a signal.
If a signal name in the guarded signal specification denotes a declared signal or a slice thereof, then the type
mark in the specification shall be the same as the type mark in the subtype indication of the signal
declaration (see 6.4.2.3).
If a signal name in the guarded signal specification denotes a slice of an array in of a composite signal, then
the type mark in the specification shall be the same as the type mark in the subtype indication of the
declaration of the array subelement.
If a signal name in the guarded signal specification denotes an array element of a composite signal, then the
type mark in the specification shall be the same as the type mark of the element subtype indication in the
declaration of the array type.
If a signal name in the guarded signal specification denotes a record element of a composite signal, then the
type mark shall be the same as the type mark of the element subtype indication in the declaration of the
record type.
Each signal shall either be declared in the declarative part enclosing the disconnection specification or be a
member of a signal declared in that declarative part.
Subject to the aforementioned rules, a disconnection specification applies to the drivers of a guarded signal
S specified with type mark T under the following circumstances:
—
For a scalar signal S, if an explicit or implicit disconnection specification of the form
disconnect S: T after time_expression;
exists, then this disconnection specification applies to the drivers of S.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
136
Copyright © 2019 IEEE. All rights reserved.
—
For a composite signal S, an explicit or implicit disconnection specification of the form
disconnect S: T after time_expression;
is equivalent to a series of implicit disconnection specifications, one for each scalar subelement of
the signal S. Each disconnection specification in the series is created as follows: it has, as its single
signal name in its signal list, a unique scalar subelement of S. Its type mark is the same as the type of
the same scalar subelement of S. Its time expression is the same as that of the original disconnection
specification.
The characteristics of the disconnection specification shall be such that each implicit disconnection
specification in the series is a legal disconnection specification.
—
If the signal list in an explicit or implicit disconnection specification contains more than one signal
name, the disconnection specification is equivalent to a series of disconnection specifications, one
for each signal name in the signal list. Each disconnection specification in the series is created as
follows: It has, as its single signal name in its signal list, a unique member of the signal list from the
original disconnection specification. Its type mark and time expression are the same as those in the
original disconnection specification.
The characteristics of the disconnection specification shall be such that each implicit disconnection
specification in the series is a legal disconnection specification.
—
An explicit disconnection specification of the form
disconnect others: T after time_expression;
is equivalent to an implicit disconnection specification where the reserved word others is replaced
with a signal list comprised of the simple names of those guarded signals that are declared signals
declared in the enclosing declarative part, whose type mark is the same as T, and that do not
otherwise have an explicit disconnection specification applicable to its drivers; the remainder of the
disconnection specification is otherwise unchanged. If there are no guarded signals in the enclosing
declarative part whose type mark is the same as T and that do not otherwise have an explicit
disconnection specification applicable to its drivers, then the preceding disconnection specification
has no effect.
The characteristics of the explicit disconnection specification shall be such that the implicit
disconnection specification, if any, is a legal disconnection specification.
—
An explicit disconnection specification of the form
disconnect all: T after time_expression;
is equivalent to an implicit disconnection specification where the reserved word all is replaced with
a signal list comprised of the simple names of those guarded signals that are declared signals
declared in the enclosing declarative part and whose type mark is the same as T; the remainder of the
disconnection specification is otherwise unchanged. If there are no guarded signals in the enclosing
declarative part whose type mark is the same as T, then the preceding disconnection specification
has no effect.
The characteristics of the explicit disconnection specification shall be such that the implicit
disconnection specification, if any, is a legal disconnection specification.
A disconnection specification with the signal list others or all for a given type that appears in a declarative
part shall be the last such specification for the given type in that declarative part. It is an error if a guarded
signal of the given type is declared in a given declarative part following such a disconnection specification.
The time expression in a disconnection specification shall be static and shall evaluate to a nonnegative
value.
It is an error if more than one disconnection specification applies to drivers of the same signal.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
137
Copyright © 2019 IEEE. All rights reserved.
If, by the aforementioned rules, no disconnection specification applies to the drivers of a guarded, scalar
signal S whose type mark is T (including a scalar subelement of a composite signal), then the following
default disconnection specification is implicitly assumed:
disconnect S : T after 0 ns;
A disconnection specification that applies to the drivers of a guarded signal S is the applicable disconnection
specification for the signal S.
Thus the implicit disconnection delay for any guarded signal is always defined, either by an explicit
disconnection specification or by an implicit one.
NOTE 1—A disconnection specification supplying either the reserved words others or all may apply to none of the
guarded signals in the current declarative part, in the event that none of the guarded signals in the current declarative part
meet all of the requirements of the disconnection specification.
NOTE 2—Since disconnection specifications are based on declarative parts, not on declarative regions, ports declared in
an entity declaration cannot be referenced by a disconnection specification in a corresponding architecture body.
Cross-references: Disconnection statements, in 11.7; guarded assignment, 11.7; guarded blocks, in 11.2;
guarded signals, in 6.4.2.3; guarded targets, in 11.7; signal guard, in 11.2.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
