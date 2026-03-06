---
title: "Section 12: IEEE Standard for VHDL Language Reference Manual"
document: "VHDL Language Reference Manual"
standard: "IEEE 1076-2019"
domain: "VHDL"
section: "12"
source_txt: "section-12-scope-and-visibility.txt"
source_pdf: "/Users/richarddje/Documents/github/VHDL-LRM-IEEE-1076-2019.pdf"
---

# Section 12: IEEE Standard for VHDL Language Reference Manual

IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
216
Copyright © 2019 IEEE. All rights reserved.
12. Scope and visibility
### 12.1 Declarative region

With two exceptions, a declarative region is a portion of the text of the description. A single declarative
region is formed by the text of each of the following:
a)
An entity declaration, together with a corresponding architecture body
b)
A configuration declaration
c)
A subprogram declaration, together with the corresponding subprogram body
d)
A package declaration together with the corresponding body (if any)
e)
A record type declaration
f)
A component declaration
g)
A block statement
h)
A process statement
i)
A sequential block statement
j)
A loop statement
k)
A block configuration
l)
A component configuration
m)
A for generate statement
n)
The generate statement body of an if or case generate
o)
A protected type declaration, together with the corresponding body
In each of these cases, the declarative region is said to be associated with the corresponding declaration or
statement. A declaration is said to occur immediately within a declarative region if this region is the inner-
most region that encloses the declaration, not counting the declarative region (if any) associated with the
declaration itself.
Certain declarative regions include disjoint parts. Each declarative region is nevertheless considered as a
(logically) continuous portion of the description text. Hence, if any rule defines a portion of text as the text
that extends from some specific point of a declarative region to the end of this region, then this portion is the
corresponding subset of the declarative region (thus, it does not include intermediate declarative items
between the interface declaration and a corresponding body declaration).
In addition to the preceding declarative regions, there is a root declarative region, not associated with a
portion of the text of the description, but encompassing any given primary unit. At the beginning of the
analysis of a given primary unit, there are no declarations whose scopes (see 12.2) are within the root
declarative region. Moreover, the root declarative region associated with any given secondary unit is the root
declarative region of the corresponding primary unit.
There is also a library declarative region associated with each design library (see 13.2). Each library
declarative region has within its scope declarations corresponding to each primary unit contained within the
associated design library.
NOTE—An architecture body, though a declaration, does not occur immediately within any declarative region.
### 12.2 Scope of declarations

For each form of declaration, the language rules define a certain portion of the description text called the
scope of the declaration. The scope of a declaration is also called the scope of any named entity declared by
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
217
Copyright © 2019 IEEE. All rights reserved.
the declaration. Furthermore, if the declaration associates some notation (either an identifier, a character
literal, or an operator symbol) with the named entity, this portion of the text is also called the scope of this
notation. Within the scope of a named entity, and only there, there are places where it is legal to use the
associated notation in order to refer to the named entity. These places are defined by the rules of visibility
and overloading.
The scope of a declaration, except for an architecture body, extends from the beginning of the declaration to
the end of the immediately enclosing declarative region; the scope of an architecture body extends from the
beginning to the end of the architecture body. In either case, this part of the scope of a declaration is called
the immediate scope. Furthermore, for any of the declarations in the following list, the scope of the
declaration extends beyond the immediate scope:
a)
A declaration that occurs immediately within a package declaration
b)
An element declaration in a record type declaration
c)
A formal parameter declaration in a subprogram declaration
d)
A local generic declaration in a component declaration
e)
A local port declaration in a component declaration
f)
A formal generic declaration in an entity declaration, an uninstantiated package declaration, an
uninstantiated subprogram declaration, or an uninstantiated protected type declaration
g)
A formal port declaration in an entity declaration
h)
A declaration that occurs immediately within a protected type declaration
i)
An architecture body
In the absence of a separate subprogram declaration, the subprogram specification given in the subprogram
body acts as the declaration, and rule c) applies also in such a case. In each of these cases except i), the given
declaration occurs immediately within some enclosing declaration, and the scope of the given declaration
extends to the end of the scope of the enclosing declaration.
In addition to the preceding rules, if the scope of any declaration includes the end of the declarative part of a
given block (whether it be an external block defined by a design entity or an internal block defined by a
block statement) then the scope of the declaration extends into a configuration declaration that configures
the given block.
If a component configuration appears as a configuration item immediately within a block configuration that
configures a given block, and if the scope of a given declaration includes the end of the declarative part of
that block, then the scope of the given declaration extends from the beginning to the end of the declarative
region associated with the given component configuration. A similar rule applies to a block configuration
that appears as a configuration item immediately within another block configuration, provided that the
contained block configuration configures an internal block. Furthermore, the scope of a use clause is
similarly extended. Finally, the scope of a library unit contained within a design library is extended along
with the scope of the logical library name corresponding to that design library.
If the scope of any declaration includes the end of the declarative region of the design entity at the root of the
design hierarchy, then the scope extends into a PSL verification unit that is bound to that design entity.
Similarly, if the scope of any declaration includes the end of the declarative region of a design entity bound
to a component instance, then the scope extends into a PSL verification unit that is bound to that component
instance.
NOTE 1—These scope rules apply to all forms of declaration. In particular, they apply also to implicit declarations and
to named primary units.
NOTE 2—The scope of an entity declaration includes an associated architecture body, if any. Thus, the entity name may
be used within the architecture body as the prefix of an expanded name denoting a declaration that occurs immediately
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
218
Copyright © 2019 IEEE. All rights reserved.
within the entity declaration or the architecture body. The scope of an architecture body does not include the
corresponding entity declaration. Thus, the entity cannot use an expanded name to refer to the architecture body nor to
any declaration within the architecture body.
### 12.3 Visibility

The meaning of the occurrence of an identifier at a given place in the text is defined by the visibility rules
and also, in the case of overloaded declarations, by the overloading rules. The identifiers considered in this
subclause include any identifier other than a reserved word or an attribute designator that denotes a
predefined attribute. The places considered in this subclause are those where a lexical element (such as an
identifier) occurs. The overloaded declarations considered in this subclause are those for subprograms and
enumeration literals.
For each identifier and at each place in the text, the visibility rules determine a set of declarations (with this
identifier) that define the possible meanings of an occurrence of the identifier. A declaration is said to be
visible at a given place in the text when, according to the visibility rules, the declaration defines a possible
meaning of this occurrence. The following two cases arise in determining the meaning of such a declaration:
—
The visibility rules determine at most one possible meaning. In such a case, the visibility rules are
sufficient to determine the declaration defining the meaning of the occurrence of the identifier, or in
the absence of such a declaration, to determine that the occurrence is not legal at the given point.
—
The visibility rules determine more than one possible meaning. In such a case, the occurrence of the
identifier is legal at this point if and only if exactly one visible declaration is acceptable for the
overloading rules in the given context or all visible declarations denote the same named entity.
A declaration is visible only within a certain part of its scope; this part starts at the end of the declaration
except in the declaration of a design unit other than a PSL verification unit, a package declaration, or a
protected type declaration, in which case it starts immediately after the reserved word is occurring after the
identifier of the design unit, package declaration, or protected type declaration. This rule applies to both
explicit and implicit declarations.
Visibility is either by selection or direct. A declaration is visible by selection at places that are defined as
follows:
a)
For a primary unit contained in a library: at the place of the suffix in a selected name whose prefix
denotes the library.
b)
For an entity name in a configuration declaration whose entity name is a simple name: at the place of
the simple name, and the context is that of the library WORK.
c)
For an architecture body associated with a given entity declaration: at the place of the block
specification in a block configuration for an external block whose interface is defined by that entity
declaration.
d)
For an architecture body associated with a given entity declaration: at the place of an architecture
identifier (between the parentheses) in the first form of an entity aspect in a binding indication.
e)
For an architecture body associated with a given entity declaration: at the place of an architecture
identifier (between the parentheses) in the second form of an instantiated unit in a component
instantiation statement.
f)
For a declaration given in a package declaration, other than in a package declaration that defines an
uninstantiated package: at the place of the suffix in a selected name whose prefix denotes the
package.
g)
For an element declaration of a given record type declaration: at the place of the suffix in a selected
name whose prefix is appropriate for the type; also at the place of a choice (before the compound
delimiter =>) in a named element association of an aggregate of the type.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
219
Copyright © 2019 IEEE. All rights reserved.
h)
For an element declaration of a given record type declaration: at the place of the record element
simple name in a record element constraint of a record constraint that applies to a type or subtype
that is the given record type or an access type whose designated type is the given record type; also at
the place of a record element simple name in a record element resolution of a record resolution
corresponding to the given record type or a subtype of the given record type.
i)
For a user-defined attribute: at the place of the attribute designator (after the delimiter ') in an
attribute name whose prefix denotes a named entity with which that attribute has been associated.
j)
For a formal parameter declaration of a given subprogram declaration: at the place of the formal part
(before the compound delimiter =>) of a named parameter association element of a corresponding
subprogram call.
k)
For a local generic declaration of a given component declaration: at the place of the formal part
(before the compound delimiter =>) of a named generic association element of a corresponding
component instantiation statement; similarly, at the place of the actual part (after the compound
delimiter =>, if any) of a generic association element of a corresponding binding indication.
l)
For a local port declaration of a given component declaration: at the place of the formal part (before
the compound delimiter =>) of a named port association element of a corresponding component
instantiation statement; similarly, at the place of the actual part (after the compound delimiter =>, if
any) of a port association element of a corresponding binding indication.
m)
For a formal generic declaration of a given entity declaration: at the place of the formal part (before
the compound delimiter =>) of a named generic association element of a corresponding binding
indication; similarly, at the place of the formal part (before the compound delimiter =>) of a generic
association element of a corresponding component instantiation statement when the instantiated unit
is a design entity or a configuration declaration.
n)
For a formal port declaration of a given entity declaration: at the place of the formal part (before the
compound delimiter =>) of a named port association element of a corresponding binding indication;
similarly, at the place of the formal part (before the compound delimiter =>) of a port association
element of a corresponding component instantiation statement when the instantiated unit is a design
entity or a configuration declaration.
o)
For a formal generic declaration or a formal port declaration of a given block statement: at the place
of the formal part (before the compound delimiter =>) of a named association element of a
corresponding generic or port map aspect.
p)
For a formal generic declaration of a given package declaration: at the place of the formal part
(before the compound delimiter =>) of a named association element of a corresponding generic map
aspect.
q)
For a formal generic declaration of a given subprogram declaration: at the place of the formal part
(before the compound delimiter =>) of a named association element of a corresponding generic map
aspect.
r)
For a formal generic type of a given uninstantiated subprogram declaration: at the place of a
signature in a subprogram instantiation declaration in which the uninstantiated subprogram name
denotes the given uninstantiated subprogram declaration.
s)
For a subprogram or alias declared immediately within a given protected type declaration, other than
in a protected type declaration that defines an uninstantiated protected type: at the place of the suffix
in a selected name whose prefix denotes an object of the protected type.
t)
For an alternative label of an if generate statement or a case generate statement: at the place of the
generate specification in a block specification that refers to the generate statement label of the
generate statement.
For a formal generic declaration of a given protected type declaration: at the place of the formal part
(before the compound delimiter =>) of a named association element of a corresponding generic map
aspect.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
220
Copyright © 2019 IEEE. All rights reserved.
Finally, within the declarative region associated with a construct other than a record type declaration or a
protected type, any declaration that occurs immediately within the region and that also occurs textually
within the construct is visible by selection at the place of the suffix of an expanded name whose prefix
denotes the construct. Similarly, within an architecture body, any declaration that occurs immediately within
the architecture body or the corresponding entity declaration is visible by selection at the place of the suffix
of an expanded name whose prefix denotes the entity declaration.
Where it is not visible by selection, a visible declaration is said to be directly visible. A declaration is said to
be directly visible within a certain part of its immediate scope; this part extends to the end of the immediate
scope of the declaration but excludes places where the declaration is hidden as explained in the following
paragraphs. In addition, a declaration occurring immediately within the visible part of a package, other than
an uninstantiated package, can be made directly visible by means of a use clause according to the rules
described in 12.4.
A declaration is said to be hidden within (part of) an inner declarative region if the inner region contains a
homograph of this declaration; the outer declaration is then hidden within the immediate scope of the inner
homograph. Each of two declarations is said to be a homograph of the other if and only if both declarations
have the same designator, and they denote different named entities, and either overloading is allowed for at
most one of the two, or overloading is allowed for both declarations and they have the same parameter and
result type profile (see 4.5.1).
At a place in which a given declaration is visible by selection, every declaration with the same designator as
the given declaration and that would otherwise be directly visible is hidden.
Within the specification of a subprogram, every declaration with the same designator as the subprogram is
hidden. Where hidden in this manner, a declaration is visible neither by selection nor directly.
Two declarations that occur immediately within the same declarative region, other than the declarative
region of a block implied by a component instantiation or the declarative region of a generic-mapped
package or subprogram equivalent to a package instance or a subprogram instance or a protected type
instance, shall not be homographs, unless exactly one of them is the implicit declaration of a predefined
operation or is an implicit alias of such an implicit declaration. In such cases, a predefined operation or alias
thereof is always hidden by the other homograph. Where hidden in this manner, an implicit declaration is
hidden within the entire scope of the other declaration (regardless of which declaration occurs first); the
implicit declaration is visible neither by selection nor directly. For a declarative region of a block implied by
a component instantiation or the declarative region of a generic-mapped package or subprogram equivalent
to a package instance or a subprogram instance, the rules of this paragraph are applied to the corresponding
entity declaration, component declaration, uninstantiated package declaration, uninstantiated subprogram
declaration, or uninstantiated protected type, as appropriate.
A declaration is hidden within a PSL declaration, a PSL directive, or a PSL verification unit if the simple
name of the declaration is a PSL keyword.
Whenever a declaration with a certain identifier is visible from a given point, the identifier and the named
entity (if any) are also said to be visible from that point. Direct visibility and visibility by selection are
likewise defined for character literals and operator symbols. An operator is directly visible if and only if the
corresponding operator declaration is directly visible.
In addition to the aforementioned rules, any declaration that is visible by selection at the end of the
declarative part of a given (external or internal) block is visible by selection in a configuration declaration
that configures the given block.
In addition, any declaration that is directly visible at the end of the declarative part of a given block is
directly visible in a block configuration that configures the given block. This rule holds unless a use clause
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
221
Copyright © 2019 IEEE. All rights reserved.
that makes a homograph of the declaration potentially visible (see 12.4) appears in the corresponding
configuration declaration, and if the scope of that use clause encompasses all or part of those configuration
items. If such a use clause appears, then the declaration will be directly visible within the corresponding
configuration items, except at those places that fall within the scope of the additional use clause. At such
places, neither name will be directly visible.
If a component configuration appears as a configuration item immediately within a block configuration that
configures a given block, and if a given declaration is visible by selection at the end of the declarative part of
that block, then the given declaration is visible by selection from the beginning to the end of the declarative
region associated with the given component configuration. A similar rule applies to a block configuration
that appears as a configuration item immediately within another block configuration, provided that the
contained block configuration configures an internal block.
If a component configuration appears as a configuration item immediately within a block configuration that
configures a given block, and if a given declaration is directly visible at the end of the declarative part of that
block, then the given declaration is visible by selection from the beginning to the end of the declarative
region associated with the given component configuration. A similar rule applies to a block configuration
that appears as a configuration item immediately within another block configuration, provided that the
contained block configuration configures an internal block. Furthermore, the visibility of declarations made
directly visible by a use clause within a block is similarly extended. Finally, the visibility of a logical library
name corresponding to a design library directly visible at the end of a block is similarly extended. The rules
of this paragraph hold unless a use clause that makes a homograph of the declaration potentially visible
appears in the corresponding block configuration, and if the scope of that use clause encompasses all or part
of those configuration items. If such a use clause appears, then the declaration will be directly visible within
the corresponding configuration items, except at those places that fall within the scope of the additional use
clause. At such places, neither name will be directly visible.
NOTE 1—The same identifier, character literal, or operator symbol may occur in different declarations and may thus be
associated with different named entities, even if the scopes of these declarations overlap. Overlap of the scopes of decla-
rations with the same identifier, character literal, or operator symbol can result from overloading of subprograms and of
enumeration literals. Such overlaps can also occur for named entities declared in the visible parts of packages and for
formal generics and ports, record elements, and formal parameters, where there is overlap of the scopes of the enclosing
package declarations, entity declarations, record type declarations, or subprogram declarations. Finally, overlapping
scopes can result from nesting.
NOTE 2—The rules defining immediate scope, hiding, and visibility imply that a reference to an identifier, character lit-
eral, or operator symbol within its own declaration is illegal (except for design units). The identifier, character literal, or
operator symbol hides outer homographs within its immediate scope—that is, from the start of the declaration. On the
other hand, the identifier, character literal, or operator symbol is visible only after the end of the declaration (again,
except for design units). For this reason, all but the last of the following declarations are illegal:
constant K: INTEGER := K*K;             --  Illegal
constant T: T;                          --  Illegal
procedure P (X: P);                     --  Illegal
function Q (X: REAL := Q) return Q;     --  Illegal
procedure R (R: REAL);                  --  Legal (although perhaps confusing)
NOTE 3—A declaration in an uninstantiated package cannot be made visible by selection by referencing it with a
selected name. However, a declaration in an instance of the package can be referenced with a selected name.
NOTE 4—There are circumstances where it is legal for two subprograms declared in the same declarative region to be
homographs. An example is the declaration of the following two subprograms in an uninstantiated package with formal
generic types T1 and T2:
procedure P (X: T1);
procedure P (X: T2);
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
222
Copyright © 2019 IEEE. All rights reserved.
Since T1 and T2 are distinct types, the subprograms are not homographs within the uninstantiated package. If an
instance of the package associates the same actual type with both T1 and T2, then the subprograms are legal homographs
within the instance. However, any call to either of the subprograms in the instance will be ambiguous.
Example:
L1: block
   signal A,B: Bit;
begin
   L2: block
      signal B: Bit;                     --  An inner homograph of B.
   begin
      A <= B after 5 ns;                 --  Means L1.A <= L2.B
      B <= L1.B after 10 ns;             --  Means L2.B <= L1.B
   end block;
   B <= A after 15 ns;                   --  Means L1.B <= L1.A
end block;
NOTE 5—The visibility of declarations within a PSL verification unit is defined in IEEE Std 1850-2010.
### 12.4 Use clauses

A use clause achieves direct visibility of declarations that are visible by selection.
```ebnf
use_clause ::=
```

use selected_name { , selected_name } ;
Each selected name in a use clause identifies one or more declarations that will potentially become directly
visible. If the suffix of the selected name is a simple name other than a type mark, or is a character literal or
operator symbol, then the selected name identifies only the declaration(s) of that simple name, character
literal, or operator symbol contained within the package or library denoted by the prefix of the selected
name.
If the suffix of the selected name is a type mark, then the declaration of the type or subtype denoted by the
type mark is identified. Moreover, the following declarations, if any, that occur immediately within the
package denoted by the prefix of the selected name, are also identified:
—
If the type mark denotes an enumeration type or a subtype of an enumeration type, the enumeration
literals of the base type
—
If the type mark denotes a subtype of a physical type, the units of the base type
—
The implicit declarations of predefined operations for the type that are not hidden by homographs
explicitly declared immediately within the package denoted by the prefix of the selected name
—
The declarations of homographs, explicitly declared immediately within the package denoted by the
prefix of the selected name, that hide implicit declarations of predefined operations for the type
If the suffix is the reserved word all, then the selected name identifies all declarations that are contained
within the package or library denoted by the prefix of the selected name.
It is an error if the prefix of a selected name in a use clause denotes an uninstantiated package.
For each use clause, except a use clause that appears within a context declaration, there is a certain region of
text called the scope of the use clause. This region starts immediately after the use clause. If a use clause is a
declarative item of some declarative region, the scope of the clause extends to the end of the given
declarative region. If a use clause occurs within the context clause of a design unit, the scope of the use
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
223
Copyright © 2019 IEEE. All rights reserved.
clause extends to the end of the root declarative region associated with the given design unit. The scope of a
use clause may additionally extend into a configuration declaration (see 12.2).
In order to determine which declarations are made directly visible at a given place by use clauses, consider
the set of declarations identified by all use clauses whose scopes enclose this place. Any declaration in this
set is a potentially visible declaration. A potentially visible declaration is actually made directly visible
except in the following three cases:
a)
A potentially visible declaration is not made directly visible if the place considered is within the
immediate scope of a homograph of the declaration.
b)
If two potentially visible declarations are homographs and one is explicitly declared and the other is
implicitly declared, then the implicit declaration is not made directly visible.
c)
Potentially visible declarations that have the same designator and that are not covered by case b) are
not made directly visible unless each of them is either an enumeration literal specification or the
declaration of a subprogram.
NOTE 1—These rules require that a declaration that is made directly visible by a use clause cannot hide an otherwise
directly visible declaration. Moreover, an explicitly declared operation has priority over an implicitly declared
homograph of that operation if both are made potentially visible by use clauses.
NOTE 2—If a named entity X declared in package P is made potentially visible within a package Q (e.g., by the
inclusion of the clause "use P.X;" in the context clause of package Q), and the context clause for design unit R includes
the clause "use Q.all;", this does not imply that X will be potentially visible in R. Only those named entities that are
actually declared in package Q will be potentially visible in design unit R (in the absence of any other use clauses).
NOTE 3—A declaration in an uninstantiated package cannot be made potentially or directly visible by a use clause.
However, a declaration in an instance of the package can be made potentially or directly visible by a use clause.
### 12.5 The context of overload resolution

Overloading is defined for names, subprograms, and enumeration literals.
For overloaded entities, overload resolution determines the actual meaning that an occurrence of an
identifier or a character literal has whenever the visibility rules have determined that more than one meaning
is acceptable at the place of this occurrence; overload resolution likewise determines the actual meaning of
an occurrence of an operator or basic operation (see 5.1).
At such a place, all visible declarations are considered. The occurrence is only legal if there is exactly one
interpretation of each constituent of the innermost complete context. Each of the following constructs is a
complete context:
—
A declaration
—
A specification
—
A statement
—
A discrete range used in a constrained array definition, a generate parameter specification, or a loop
parameter specification
—
The expression of a type conversion
—
The expression of a case statement, or a case generate statement
—
The expression following a for generate statement label in an external name
When considering possible interpretations of a complete context, the only rules considered are the syntax
rules, the scope and visibility rules, and the rules of the form as follows:
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
224
Copyright © 2019 IEEE. All rights reserved.
a)
Any rule that requires a name or expression to have a certain type or to have the same type as
another name or expression.
b)
Any rule that requires the type of a name or expression to be a type of a certain class; similarly, any
rule that requires a certain type to be a discrete, integer, floating-point, physical, universal, or
character type.
c)
Any rule that requires a prefix to be appropriate for a certain type.
d)
The rules that require the type of an aggregate or string literal to be determinable solely from the
enclosing complete context. Similarly, the rules that require that the meaning of the prefix of an
attribute shall be determinable independently of the attribute designator and independently of the
fact that it is the prefix of an attribute.
e)
The rules given for the resolution of overloaded subprogram calls; for the implicit conversions of
universal expressions; for the interpretation of discrete ranges with bounds having a universal type;
for the interpretation of an expanded name whose prefix denotes a subprogram; and for a
subprogram named in a subprogram instantiation declaration to denote an uninstantiated
subprogram.
f)
The rules given for the requirements on the return type, the number of formal parameters, and the
types of the formal parameters of the subprogram denoted by the resolution function name (see 4.6).
NOTE 1—If there is only one possible interpretation of an occurrence of an identifier, character literal, operator symbol,
or string, that occurrence denotes the corresponding named entity. However, this condition does not mean that the
occurrence is necessarily legal since other requirements exist that are not considered for overload resolution: for
example, the fact that the expression is static, the parameter modes, conformance rules, the use of named association in
an indexed name, the use of open in an indexed name, the use of a slice as an actual to a function call, and so forth.
NOTE 2—A loop parameter specification is a declaration, and hence a complete context.
NOTE 3—Rules that require certain constructs to have the same parameter and result type profile fall under the
preceding category a). This includes the rule that the actual associated with a formal generic subprogram have a
conforming profile with the formal. The same holds for rules that require lexical conformance of two constructs, since
lexical conformance requires that corresponding names be given the same meaning by the visibility and overloading
rules.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
