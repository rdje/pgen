---
title: "Section 6: IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language"
document: "SystemVerilog Language Reference Manual"
standard: "IEEE 1800-2017"
domain: "SystemVerilog"
section: "6"
source_txt: "section-6-data-types.txt"
source_pdf: "/Users/richarddje/Documents/github/SystemVerilog-LRM-IEEE-1800-2017.pdf"
---

# Section 6: IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language

IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
83
Copyright © 2018 IEEE. All rights reserved.
6. Data types
### 6.1 General

This clause describes the following:
—
SystemVerilog logic value and strength set
—
Net declarations
—
Singular variable declarations
—
Constants
—
Scope and lifetime of data
—
Type compatibility
—
Type operator and type casting
### 6.2 Data types and data objects

SystemVerilog makes a distinction between an object and its data type. A data type is a set of values and a
set of operations that can be performed on those values. Data types can be used to declare data objects or to
define user-defined data types that are constructed from other data types. A data object is a named entity that
has a data value and a data type associated with it, such as a parameter, a variable, or a net.
### 6.3 Value set

#### 6.3.1 Logic values

The SystemVerilog value set consists of the following four basic values:
0—represents a logic zero or a false condition
1—represents a logic one or a true condition
x—represents an unknown logic value
z—represents a high-impedance state
The values 0 and 1 are logical complements of one another.
When the z value is present at the input of a gate or when it is encountered in an expression, the effect is
usually the same as an x value. Notable exceptions are the metal-oxide semiconductor (MOS) primitives,
which can pass the z value.
The name of this primitive data type is logic. This name can be used to declare objects and to construct
other data types from the 4-state data type.
Several SystemVerilog data types are 4-state types, which can store all four logic values. All bits of 4-state
vectors can be independently set to one of the four basic values. Some SystemVerilog data types are 2-state,
and only store 0 or 1 values in each bit of a vector. Other exceptions are the event type (see 6.17), which has
no storage, and the real types (see 6.12).
#### 6.3.2 Strengths

The language includes strength information in addition to the basic value information for nets. This is
described in detail in Clause 28. The additional strength information associated with bits of a net is not
considered part of the data type.
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
84
Copyright © 2018 IEEE. All rights reserved.
Two types of strengths can be specified in a net declaration, as follows:
—
Charge strength shall only be used when declaring a net of type trireg.
—
Drive strength shall only be used when placing a continuous assignment on a net in the same
statement that declares the net.
Gate declarations can also specify a drive strength. See Clause 28 for more information on gates and for
information on strengths.
##### 6.3.2.1 Charge strength

The charge strength specification shall be used only with trireg nets. A trireg net shall be used to model
charge storage; charge strength shall specify the relative size of the capacitance indicated by one of the
following keywords:
—
small
—
medium
—
large
The default charge strength of a trireg net shall be medium.
A trireg net can model a charge storage node whose charge decays over time. The simulation time of a
charge decay shall be specified in the delay specification for the trireg net (see 28.16.2).
For example:
trireg a;
// trireg net of charge strength medium
trireg (large) #(0,0,50) cap1;
// trireg net of charge strength large
// with charge decay time 50 time units
trireg (small) signed [3:0] cap2;
// signed 4-bit trireg vector of
// charge strength small
##### 6.3.2.2 Drive strength

The drive strength specification allows a continuous assignment to be placed on a net in the same statement
that declares that net. See Clause 10 for more details. Net drive strength properties are described in detail in
Clause 28.
### 6.4 Singular and aggregate types

Data types are categorized as either singular or aggregate. A singular type shall be any data type except an
unpacked structure, unpacked union, or unpacked array (see 7.4 on arrays). An aggregate type shall be any
unpacked structure, unpacked union, or unpacked array data type. A singular variable or expression
represents a single value, symbol, or handle. Aggregate expressions and variables represent a set or
collection of singular values. Integral types (see 6.11.1) are always singular even though they can be sliced
into multiple singular values. The string data type is singular even though a string can be indexed in a
similar way to an unpacked array of bytes.
These categories are defined so that operators and functions can simply refer to these data types as a
collective group. For example, some functions recursively descend into an aggregate variable until reaching
a singular value and then perform an operation on each singular value.
Although a class is a type, there are no variables or expressions of class type directly, only class object
handles that are singular. Therefore, classes need not be categorized in this manner (see Clause 8 on classes).
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
85
Copyright © 2018 IEEE. All rights reserved.
### 6.5 Nets and variables

There are two main groups of data objects: variables and nets. These two groups differ in the way in which
they are assigned and hold values.
A net can be written by one or more continuous assignments, by primitive outputs, or through module ports.
The resultant value of multiple drivers is determined by the resolution function of the net type. A net cannot
be procedurally assigned. If a net on one side of a port is driven by a variable on the other side, a continuous
assignment is implied. A force statement can override the value of a net. When released, the net returns to
the resolved value.
Variables can be written by one or more procedural statements, including procedural continuous
assignments. The last write determines the value. Alternatively, variables can be written by one continuous
assignment or one port.
Variables can be packed or unpacked aggregates of other types (see 7.4 for packed and unpacked types).
Multiple assignments made to independent elements of a variable are examined individually. Independent
elements include different members of a structure or different elements of an array. Each bit in a packed type
is also an independent element. Thus, in an aggregate of packed types, each bit in the aggregate is an
independent element.
An assignment where the left-hand side contains a slice is treated as a single assignment to the entire slice.
Thus, a structure or array can have one element assigned procedurally and another element assigned
continuously. And elements of a structure or array can be assigned with multiple continuous assignments,
provided that each element is covered by no more than a single continuous assignment.
The precise rule is that it shall be an error to have multiple continuous assignments or a mixture of
procedural and continuous assignments writing to any term in the expansion of a written longest static prefix
of a variable (see 11.5.3 for the definition of a longest static prefix).
For example, assume the following structure declaration:
struct {
bit [7:0] A;
bit [7:0] B;
byte C;
} abc;
The following statements are legal assignments to struct abc:
assign abc.C = sel ? 8'hBE : 8'hEF;
not
(abc.A[0],abc.B[0]),
(abc.A[1],abc.B[1]),
(abc.A[2],abc.B[2]),
(abc.A[3],abc.B[3]);
always @(posedge clk) abc.B <= abc.B + 1;
The following additional statements are illegal assignments to struct abc:
// Multiple continuous assignments to abc.C
assign abc.C = sel ? 8'hDE : 8'hED;
// Mixing continuous and procedural assignments to abc.A[3]
always @(posedge clk) abc.A[7:3] <= !abc.B[7:3];
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
86
Copyright © 2018 IEEE. All rights reserved.
For the purposes of the preceding rule, a declared variable initialization or a procedural continuous
assignment is considered a procedural assignment. The force statement overrides the procedural assign
statement, which in turn overrides the normal assignments. A force statement is neither a continuous nor a
procedural assignment.
A continuous assignment shall be implied when a variable is connected to an input port declaration. This
makes assignments to a variable declared as an input port illegal. A continuous assignment shall be implied
when a variable is connected to the output port of an instance. This makes additional procedural or
continuous assignments to a variable connected to the output port of an instance illegal.
Variables cannot be connected to either side of an inout port. Variables can be shared across ports with the
ref port type. See 23.3.3 for more information about ports and port connection rules.
The compiler can issue a warning if a continuous assignment could drive strengths other than St0, St1,
StX, or HiZ to a variable. In any case, automatic type conversion shall be applied to the assignment, and the
strength is lost.
Unlike nets, a variable cannot have an implicit continuous assignment as part of its declaration. An
assignment as part of the declaration of a variable is a variable initialization, not a continuous assignment.
For example:
wire w = vara & varb;
// net with a continuous assignment
logic v = consta & constb;
// variable with initialization
logic vw;
// no initial assignment
assign vw = vara & varb;
// continuous assignment to a variable
real circ;
assign circ = 2.0 * PI * R;
// continuous assignment to a variable
Data shall be declared before they are used, apart from implicit nets (see 6.10).
Within a name space (see 3.13), it shall be illegal to redeclare a name already declared by a net, variable, or
other declaration.
### 6.6 Net types

There are two different kinds of net types: built-in and user-defined. The net types can represent physical
connections between structural entities, such as gates. A net shall not store a value (except for the trireg
net). Instead, its value shall be determined by the values of its drivers, such as a continuous assignment or a
gate. See Clause 10 and Clause 28 for definitions of these constructs. If no driver is connected to a net, its
value shall be high-impedance (z) unless the net is a trireg, in which case it shall hold the previously
driven value.
There are several distinct types of built-in net types, as shown in Table 6-1.
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
87
Copyright © 2018 IEEE. All rights reserved.
#### 6.6.1 Wire and tri nets

The wire and tri nets connect elements. The net types wire and tri shall be identical in their syntax and
functions; two names are provided so that the name of a net can indicate the purpose of the net in that model.
A wire net can be used for nets that are driven by a single gate or continuous assignment. The tri net type
can be used where multiple drivers drive a net.
Logical conflicts from multiple sources of the same strength on a wire or a tri net result in x (unknown)
values.
Table 6-2 is a truth table for resolving multiple drivers on wire and tri nets. It assumes equal strengths for
both drivers. See 28.11 for a discussion of logic strength modeling.
#### 6.6.2 Unresolved nets

The uwire net is an unresolved or unidriver wire and is used to model nets that allow only a single driver.
The uwire type can be used to enforce this restriction. It shall be an error to connect any bit of a uwire net
to more than one driver. It shall be an error to connect a uwire net to a bidirectional terminal of a
bidirectional pass switch.
The port connection rule in 23.3.3.6 enforces this restriction across the net hierarchy or shall issue a warning
if not.
#### 6.6.3 Wired nets

Wired nets are of type wor, wand, trior, and triand and are used to model wired logic configurations.
Wired nets use different truth tables to resolve the conflicts that result when multiple drivers drive the same
net. The wor and trior nets shall create wired or configurations so that when any of the drivers is 1, the
resulting value of the net is 1. The wand and triand nets shall create wired and configurations so that if any
driver is 0, the value of the net is 0.
Table 6-1—Built-in net types
wire
tri
tri0
supply0
wand
triand
tri1
supply1
wor
trior
trireg
uwire
Table 6-2—Truth table for wire and tri nets
wire/tri
0
1
x
z
0
0
x
x
0
1
x
1
x
1
x
x
x
x
x
z
0
1
x
z
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
88
Copyright © 2018 IEEE. All rights reserved.
The net types wor and trior shall be identical in their syntax and functionality. The net types wand and
triand shall be identical in their syntax and functionality. Table 6-3 and Table 6-4 give the truth tables for
wired nets, assuming equal strengths for both drivers. See 28.11 for a discussion of logic strength modeling.
#### 6.6.4 Trireg net

The trireg net stores a value and is used to model charge storage nodes. A trireg net can be in one of two
states, as follows:
Driven state
When at least one driver of a trireg net has a value of 1, 0, or x, the resolved
value propagates into the trireg net and is the driven value of the trireg net.
Capacitive state
When all the drivers of a trireg net are at the high-impedance value (z), the
trireg net retains its last driven value; the high-impedance value does not
propagate from the driver to the trireg.
The strength of the value on the trireg net in the capacitive state can be small, medium, or large,
depending on the size specified in the declaration of the trireg net. The strength of a trireg net in the
driven state can be supply, strong, pull, or weak, depending on the strength of the driver.
For example, Figure 6-1 shows a schematic that includes a trireg net whose size is medium, its driver, and
the simulation results.
Table 6-3—Truth table for wand and triand nets
wand/triand
0
1
x
z
0
0
0
0
0
1
0
1
x
1
x
0
x
x
x
z
0
1
x
z
Table 6-4—Truth table for wor and trior nets
wor/trior
0
1
x
z
0
0
1
x
0
1
1
1
1
1
x
x
1
x
x
z
0
1
x
z
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
89
Copyright © 2018 IEEE. All rights reserved.
Figure 6-1—Simulation values of a trireg and its driver
a)
At simulation time 0, wire a and wire b have a value of 1. A value of 1 with a strong strength
propagates from the and gate through the nmos switches connected to each other by wire c into
trireg net d.
b)
At simulation time 10, wire a changes value to 0, disconnecting wire c from the and gate. When
wire c is no longer connected to the and gate, the value of wire c changes to HiZ. The value of wire
b remains 1 so wire c remains connected to trireg net d through the nmos2 switch. The HiZ value
does not propagate from wire c into trireg net d. Instead, trireg net d enters the capacitive state,
storing its last driven value of 1. It stores the 1 with a medium strength.
##### 6.6.4.1 Capacitive networks

A capacitive network is a connection between two or more trireg nets. In a capacitive network whose trireg
nets are in the capacitive state, logic and strength values can propagate between trireg nets.
For example, Figure 6-2 shows a capacitive network in which the logic value of some trireg nets change the
logic value of other trireg nets of equal or smaller size.
In Figure 6-2, the capacitive strength of trireg_la net is large, trireg_me1 and trireg_me2 are
medium, and trireg_sm is small. Simulation reports the following sequence of events:
a)
At simulation time 0, wire a and wire b have a value of 1. The wire c drives a value of 1 into
trireg_la and trireg_sm; wire d drives a value of 1 into trireg_me1 and trireg_me2.
b)
At simulation time 10, the value of wire b changes to 0, disconnecting trireg_sm and
trireg_me2 from their drivers. These trireg nets enter the capacitive state and store the value 1,
their last driven value.
c)
At simulation time 20, wire c drives a value of 0 into trireg_la.
d)
At simulation time 30, wire d drives a value of 0 into trireg_me1.
e)
At simulation time 40, the value of wire a changes to 0, disconnecting trireg_la and
trireg_me1 from their drivers. These trireg nets enter the capacitive state and store the value 0.
f)
At simulation time 50, the value of wire b changes to 1.
g)
This change of value in wire b connects trireg_sm to trireg_la; these trireg nets have different
sizes and stored different values. This connection causes the smaller trireg net to store the value of
the larger trireg net, and trireg_sm now stores a value of 0.
This change of value in wire b also connects trireg_me1 to trireg_me2; these trireg nets have
the same size and stored different values. The connection causes both trireg_me1 and
trireg_me2 to change value to x.
nmos1
nmos2
wire c
trireg d
wire a
wire b
simulation time
wire a
wire b
wire c
trireg d
1
1
strong 1
strong 1
0
1
HiZ
medium 1
10
0
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
90
Copyright © 2018 IEEE. All rights reserved.
Figure 6-2—Simulation results of a capacitive network
In a capacitive network, charge strengths propagate from a larger trireg net to a smaller trireg net. Figure 6-3
shows a capacitive network and its simulation results.
In Figure 6-3, the capacitive strength of trireg_la is large, and the capacitive strength of trireg_sm is
small. Simulation reports the following results:
a)
At simulation time 0, the values of wire a, wire b, and wire c are 1, and wire a drives a strong 1
into trireg_la and trireg_sm.
b)
At simulation time 10, the value of wire b changes to 0, disconnecting trireg_la and trireg_sm
from wire a. The trireg_la and trireg_sm nets enter the capacitive state. Both trireg nets share
the large charge of trireg_la because they remain connected through tranif1_2.
c)
At simulation time 20, the value of wire c changes to 0, disconnecting trireg_sm from
trireg_la. The trireg_sm no longer shares large charge of trireg_la and now stores a
small charge.
d)
At simulation time 30, the value of wire c changes to 1, connecting the two trireg nets. These trireg
nets now share the same charge.
e)
At simulation time 40, the value of wire c changes again to 0, disconnecting trireg_sm from
trireg_la. Once again, trireg_sm no longer shares the large charge of trireg_la and now
stores a small charge.
40
0
0
0
0
0
1
0
1
trireg_sm
trireg_la
trireg_me2
trireg_me1
wire a
wire b
wire c
wire d
simulation
time
wire a
wire b
wire c
wire d
trireg_la
trireg_sm
trireg_me1
trireg_me2
0
1
1
1
1
1
1
1
1
10
0
1
1
1
1
1
1
1
20
1
0
1
1
1
0
0
1
30
1
0
0
0
0
1
0
1
nmos_1
nmos_2
tranif1_2
50
0
1
0
0
0
0
x
x
tranif1_1
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
91
Copyright © 2018 IEEE. All rights reserved.
Figure 6-3—Simulation results of charge sharing
##### 6.6.4.2 Ideal capacitive state and charge decay

A trireg net can retain its value indefinitely, or its charge can decay over time. The simulation time of
charge decay is specified in the delay specification of the trireg net. See 28.16.2 for charge decay
explanation.
#### 6.6.5 Tri0 and tri1 nets

The tri0 and tri1 nets model nets with resistive pulldown and resistive pullup devices on them. A tri0
net is equivalent to a wire net with a continuous 0 value of pull strength driving it. A tri1 net is equivalent
to a wire net with a continuous 1 value of pull strength driving it.
When no driver drives a tri0 net, its value is 0 with strength pull. When no driver drives a tri1 net, its
value is 1 with strength pull. When there are drivers on a tri0 or tri1 net, the drivers combine with the
strength pull value implicitly driven on the net to determine the net’s value. See 28.11 for a discussion of
logic strength modeling.
Table 6-5 and Table 6-6 are truth tables for modeling multiple drivers of strength strong on tri0 and
tri1 nets. The resulting value on the net has strength strong, unless both drivers are z, in which case the
net has strength pull.
tranif1_2
trireg_sm
simulation
time
wire a
wire b
wire c
tranif1_1
wire a
wire b
trireg_la
trireg_sm
0
strong 1
wire c
strong 1
strong 1
1
1
0
1
large 1
large 1
strong 1
10
20
0
0
small 1
large 1
strong 1
30
1
large 1
large 1
strong 1
0
40
0
0
small 1
large 1
strong 1
trireg_la
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
92
Copyright © 2018 IEEE. All rights reserved.

#### 6.6.6 Supply nets

The supply0 and supply1 nets can be used to model the power supplies in a circuit. These nets shall have
supply strengths.
#### 6.6.7 User-defined nettypes

A user-defined nettype allows users to describe more general abstract values for a wire, including its
resolution function. This nettype is similar to a typedef in some ways, but shall only be used in declaring
a net. It provides a name for a particular data type and optionally an associated resolution function.
The syntax for net type declarations is given in Syntax 6-1.
```ebnf
net_type_declaration ::=
```

// from A.2.1.3
nettype data_type net_type_identifier
[ with [ package_scope | class_scope ] tf_identifier ] ;
| nettype [ package_scope | class_scope ] net_type_identifier net_type_identifier ;
Syntax 6-1—Syntax for net type declarations (excerpt from Annex A)
A net declared with a nettype therefore uses that data type and, if specified, the associated resolution
function. An explicit data type is required for a user-defined nettype.
Table 6-5—Truth table for tri0 net
tri0
0
1
x
z
0
0
x
x
0
1
x
1
x
1
x
x
x
x
x
z
0
1
x
0
Table 6-6—Truth table for tri1 net
tri1
0
1
x
z
0
0
x
x
0
1
x
1
x
1
x
x
x
x
x
z
0
1
x
1
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
93
Copyright © 2018 IEEE. All rights reserved.
Certain restrictions apply to the data type of a net with a user-defined nettype. A valid data type shall be
one of the following:
a)
A 4-state integral type, including a packed array, packed structure or union.
b)
A 2-state integral type, including a packed array, packed structure or union with 2-state data type
members.
c)
A real or shortreal type.
d)
A fixed-size unpacked array, unpacked structure or union, where each element has a valid data type
for a net of a user-defined nettype.
A second form of a nettype declaration is to create another name for an existing nettype.
An atomic net is a net whose value is updated and resolved as a whole. A net declared with a user-defined
nettype is an atomic net. Similarly, a logic net is an atomic net, but a logic vector net is not an atomic
net as each logic element is resolved and updated independently. While an atomic net may have a singular
or aggregate value, each atomic net is intended to describe a single connection point in the design.
The resolution for a user-defined nettype is specified using a SystemVerilog function declaration. If a
resolution function is specified, then when a driver of the net changes value, an update event is scheduled on
the net in the Active (or Reactive) region. When the update event matures, the simulator calls the resolution
function to compute the value of the net from the values of the drivers. The return type of the function shall
match the data type of the nettype. The function shall accept an arbitrary number of drivers, since different
instances of the net could be connected to different numbers of drivers. Any change in the value of one or
more of the drivers shall trigger the evaluation of the resolution function associated with that nettype.
A user-defined resolution function for a net of a user-defined nettype with a data type T shall be a function
with a return type of T and a single input argument whose type is a dynamic array of elements of type T. A
resolution function shall be automatic (or preserve no state information) and have no side effects. A
resolution function shall not resize the dynamic array input argument nor shall it write to any part of the
dynamic array input argument. While a class function method may be used for a resolution function, such
functions shall be class static methods as the method call occurs in a context where no class object is
involved in the call. Parameterized variants of such methods can be created through the use of parameterized
class methods as described in 13.8.
Two different nettypes can use the same data type, but have different resolution functions A nettype may
be declared without a resolution function, in which case it shall be an error for a net of that nettype to have
multiple drivers.
Due to nondeterminism within scheduling regions, if there are multiple driver updates within a scheduling
region, there may be multiple evaluations of the resolution function.
A force statement can override the value of a net of a user-defined nettype. When released, the net
returns to the resolved value.
// user-defined data type T
typedef struct {
real field1;
bit field2;
} T;
// user-defined resolution function Tsum
function automatic T Tsum (input T driver[]);
Tsum.field1 = 0.0;
foreach (driver[i])
Tsum.field1 += driver[i].field1;
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
94
Copyright © 2018 IEEE. All rights reserved.
endfunction
nettype T wT;
// an unresolved nettype wT whose data type is T
// a nettype wTsum whose data type is T and
// resolution function is Tsum
nettype T wTsum with Tsum;
// user-defined data type TR
typedef real TR[5];
// an unresolved nettype wTR whose data type
// is an array of real
nettype TR wTR;
// declare another name nettypeid2 for nettype wTsum
nettype wTsum nettypeid2;
The following example shows how to use a combination of a parameterized class definition with class static
methods to parameterize the data type of a user-defined nettype.
class Base #(parameter p = 1);
typedef struct {
real r;
bit[p-1:0] data;
} T;
static function T Tsum (input T driver[]);
Tsum.r = 0.0;
Tsum.data = 0;
foreach (driver[i])
Tsum.data += driver[i].data;
Tsum.r = $itor(Tsum.data);
endfunction
endclass
typedef Base#(32) MyBaseT;
nettype MyBaseT::T narrowTsum with MyBaseT::Tsum;
typedef Base#(64) MyBaseType;
nettype MyBaseType::T wideTsum with MyBaseType::Tsum;
narrowTsum net1; // data is 32 bits wide
wideTsum net2; // data is 64 bits wide
#### 6.6.8 Generic interconnect

In SystemVerilog it is possible to use net types and configurations to create design models with varying
levels of abstraction. In order to support netlist designs, which primarily specify design element instances
and the net connections between the design elements, SystemVerilog defines a generic form of nets. Such
generic nets allow the separation of the specification of the net connections from the types of the
connections.
A net or port declared as interconnect (an interconnect net or port) indicates a typeless or generic net.
Such nets or ports are only able to express net port and terminal connections and shall not be used in any
procedural context nor in any continuous or procedural continuous assignments. An interconnect net or
port shall not be used in any expression other than a net_lvalue expression in which all nets or ports in the
expression are also interconnect nets. An interconnect array shall be considered valid even if
different bits in the array are resolved to different net types as demonstrated in the following example. It
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
95
Copyright © 2018 IEEE. All rights reserved.
shall be legal to specify a net_alias statement with an interconnect net_lvalue. See 23.3.3.7.1 and
###### 23.3.3.7.2 for port and terminal connection rules for interconnect nets.

package NetsPkg;
nettype real realNet;
endpackage : NetsPkg
module top();
interconnect [0:1] iBus;
lDriver l1(iBus[0]);
rDriver r1(iBus[1]);
rlMod m1(iBus);
endmodule : top
module lDriver(output wire logic out);
endmodule : lDriver
module rDriver
import NetsPkg::*;
(output realNet out);
endmodule : rDriver
module rlMod(input interconnect [0:1] iBus);
lMod l1(iBus[0]);
rMod r1(iBus[1]);
endmodule : rlMod
The following simple example serves to illustrate the usefulness of an interconnect net. The example
contains a top level module (top) that instantiates a stimulus module (driver) and a comparator module
(cmp). This configuration is intended to compare two elements and determine if they are equal. There are
two different versions of the configuration, as described by the two different config blocks: one that works
on real values and one that works on logic values. By using the typeless interconnect net, we can use
the same testbench with both configurations, without having to change anything in the testbench itself. The
interconnect net aBus takes its data type from the type of its connections.
<file lib.map>
library realLib *.svr;
library logicLib *.sv;
config cfgReal;
design logicLib.top;
default liblist realLib logicLib;
endconfig
config cfgLogic;
design logicLib.top;
default liblist logicLib realLib;
endconfig
<file top.sv>
module top();
interconnect [0:3] [0:1] aBus;
logic [0:3] dBus;
driver driverArray[0:3](aBus);
cmp cmpArray[0:3](aBus,rst,dBus);
endmodule : top
<file nets.pkg>
package NetsPkg;
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
96
Copyright © 2018 IEEE. All rights reserved.
nettype real realNet;
endpackage : NetsPkg
<file driver.svr>
module driver
import NetsPkg::*;
#(parameter int delay = 30,
int iterations = 256)
(output realNet [0:1] out);
timeunit 1ns / 1ps;
real outR[1:0];
assign out = outR;
initial begin
outR[0] = 0.0;
outR[1] = 3.3;
for (int i = 0; i < iterations; i++) begin
#delay outR[0] += 0.2;
outR[1] -= 0.2;
end
end
endmodule : driver
<file driver.sv>
module driver #(parameter int delay = 30,
int iterations = 256)
(output wire logic [0:1] out);
timeunit 1ns / 1ps;
logic [0:1] outvar;
assign out = outvar;
initial begin
outvar = '0;
for (int i = 0; i < iterations; i++)
#delay outvar++;
end
endmodule : driver
<file cmp.svr>
module cmp
import NetsPkg::*;
#(parameter real hyst = 0.65)
(input realNet [0:1] inA,
input
logic rst,
output logic out);
timeunit 1ns / 1ps;
real updatePeriod = 100.0;
initial out = 1'b0;
always #updatePeriod begin
if (rst) out <= 1'b0;
else if (inA[0] > inA[1]) out <= 1'b1;
else if (inA[0] < inA[1] - hyst) out <= 1'b0;
end
endmodule : cmp
<file cmp.sv>
module cmp #(parameter real hyst = 0.65)
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
97
Copyright © 2018 IEEE. All rights reserved.
(input wire logic [0:1] inA,
input
logic rst,
output logic out);
initial out = 1'b0;
always @(inA, rst) begin
if (rst) out <= 1'b0;
else if (inA[0] & ~inA[1]) out <= 1'b1;
else out <= 1'b0;
end
endmodule : cmp
### 6.7 Net declarations

The syntax for net declarations is given in Syntax 6-2.
```ebnf
net_declaration12 ::=
```

// from A.2.1.3
net_type [ drive_strength | charge_strength ] [ vectored | scalared ]
data_type_or_implicit [ delay3 ] list_of_net_decl_assignments ;
| net_type_identifier [ delay_control ]
list_of_net_decl_assignments ;
| interconnect implicit_data_type [ # delay_value ]
net_identifier { unpacked_dimension }
[ , net_identifier { unpacked_dimension }] ;
```ebnf
net_type ::=
```

// from A.2.2.1
supply0 | supply1 | tri | triand | trior | trireg | tri0 | tri1 | uwire | wire | wand | wor
```ebnf
drive_strength ::=
```

// from A.2.2.2
( strength0 , strength1 )
| ( strength1 , strength0 )
| ( strength0 , highz1 )
| ( strength1 , highz0 )
| ( highz0 , strength1 )
| ( highz1 , strength0 )
```ebnf
strength0 ::= supply0 | strong0 | pull0 | weak0
strength1 ::= supply1 | strong1 | pull1 | weak1
charge_strength ::= ( small ) | ( medium ) | ( large )
delay3 ::=
```

// from A.2.2.3
# delay_value | # ( mintypmax_expression [ , mintypmax_expression [ , mintypmax_expression ] ] )
```ebnf
delay2 ::= # delay_value | # ( mintypmax_expression [ , mintypmax_expression ] )
delay_value ::=
```

unsigned_number
| real_number
| ps_identifier
| time_literal
| 1step
```ebnf
list_of_net_decl_assignments ::= net_decl_assignment { , net_decl_assignment }
```

// from A.2.3
```ebnf
net_decl_assignment ::= net_identifier { unpacked_dimension } [ = expression ]
```

// from A.2.4

12) A charge strength shall only be used with the trireg keyword. When the vectored or scalared keyword is
used, there shall be at least one packed dimension.
Syntax 6-2—Syntax for net declarations (excerpt from Annex A)
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
98
Copyright © 2018 IEEE. All rights reserved.
#### 6.7.1 Net declarations with built-in net types

Net declarations without assignments and whose nettype is not a user-defined nettype are described in
this subclause. Net declarations with assignments are described in Clause 10.
A net declaration begins with a net type that determines how the values of the nets in the declaration are
resolved. The declaration can include optional information such as delay values, drive or charge strength,
and a data type.
If a set of nets share the same characteristics, they can be declared in the same declaration statement.
Any 4-state data type can be used to declare a net. For example:
trireg (large) logic #(0,0,0) cap1;
typedef logic [31:0] addressT;
wire addressT w1;
wire struct packed { logic ecc; logic [7:0] data; } memsig;
If a data type is not specified in the net declaration or if only a range and/or signing is specified, then the data
type of the net is implicitly declared as logic. For example:
wire w;
// equivalent to "wire logic w;"
wire [15:0] ww;
// equivalent to "wire logic [15:0] ww;"
A net declared as an interconnect net shall:
—
have no data type but may have optional packed or unpacked dimensions;
—
not specify drive_strength or charge_strength;
—
not have assignment expressions;
—
specify at most one delay value.
Certain restrictions apply to the data type of a net. A valid data type for a net shall be one of the following:
a)
A 4-state integral type, including a packed array or packed structure.
b)
A fixed-size unpacked array or unpacked structure, where each element has a valid data type for a
net.
The effect of this recursive definition is that a net is composed entirely of 4-state bits and is treated
accordingly. In addition to a signal value, each bit of a net shall have additional strength information. When
bits of signals combine, the strength and value of the resulting signal shall be determined as described in
28.12.
A lexical restriction applies to the use of the reg keyword in a net or port declaration. A net type keyword
shall not be followed directly by the reg keyword. Thus, the following declarations are in error:
tri reg r;
inout wire reg p;
The reg keyword can be used in a net or port declaration if there are lexical elements between the net type
keyword and the reg keyword.
The default initialization value for a net shall be the value z. Nets with drivers shall assume the output value
of their drivers. The trireg net is an exception. The trireg net shall default to the value x, with the
strength specified in the net declaration (small, medium, or large).
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
99
Copyright © 2018 IEEE. All rights reserved.
As described in 6.6.8, an interconnect net is restricted in terms of its declaration and use. The following
are some examples of legal and illegal interconnect net declarations:
interconnect w1;
// legal
interconnect [3:0] w2;
// legal
interconnect [3:0] w3 [1:0];
// legal
interconnect logic [3:0] w4;
// illegal – data type specified
interconnect #(1,2,3) w5;
// illegal – only one delay permitted
assign w1 = 1;
// illegal – not allowed in a
// continuous assign
initial $display(w1);
// illegal – not allowed in a
// procedural context
#### 6.7.2 Net declarations with user-defined nettypes

A net with a user-defined nettype allows users to describe more general abstract values for a wire. A net
declared with a nettype uses the data type and any associated resolution function for that nettype.
// an unresolved nettype wT whose data type is T
// Refer to example in 6.6.7 for declaration of the data type T
nettype T wT;
// a nettype wTsum whose data type is T and
// resolution function is Tsum
// Refer to example in 6.6.7 for the declaration of Tsum
nettype T wTsum with Tsum;
// a net of unresolved nettype wT
wT w1;
// an array of nets, each net element is of unresolved nettype wT
wT w2[8];
// a net of resolved nettype wTsum and resolution function Tsum
wTsum w3;
// an array of nets, each net is of resolved nettype wTsum
wTsum w4[8];
// user-defined data type TR which is an array of reals
typedef real TR[5];
// an unresolved nettype wTR with data type TR
nettype TR wTR;
// a net with unresolved nettype wTR and data type TR
wTR w5;
// an array of nets, each net has an unresolved nettype wTR
// and data type TR
wTR w6[8];
#### 6.7.3 Initialization of nets with user-defined nettypes

The resolution function for any net of a user-defined nettype shall be activated at time zero at least once.
This activation occurs even for such nets with no drivers or no value changes on drivers at time zero. Since
the actual evaluation of the resolution function is subject to scheduling nondeterminism, no assumptions can
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
100
Copyright © 2018 IEEE. All rights reserved.
be made regarding the state of driven values during the guaranteed call, which may precede or follow any
driver changes at time zero.
The initial value of a net with a user-defined nettype shall be set before any initial or always procedures
are started and before the activation of the guaranteed time zero resolution call. The default initialization
value for a net with a user-defined nettype shall be the default value defined by the data type. Table 6-7
defines the default value for data types of variables if no initializer is provided; those default values shall
also apply to nets of user-defined nettypes for valid data types of a net. For a net with a user-defined
nettype whose data type is a struct type, any initialization expressions for the members within the
struct shall be applied.
NOTE— The default value for a logic net of a user-defined nettype is X. This default means that a bit of a logic
data type in an unresolved user-defined nettype will be X if it has no drivers, not Z. For a net with a resolved
nettype, the value would be determined by the resolution function executed with an empty array of driver values.
### 6.8 Variable declarations

A variable is an abstraction of a data storage element. A variable shall store a value from one assignment to
the next. An assignment statement in a procedure acts as a trigger that changes the value in the data storage
element.
The syntax for variable declarations is given in Syntax 6-3.

```ebnf
data_declaration ::=
```

// from A.2.1.3
[ const ] [ var ] [ lifetime ] data_type_or_implicit list_of_variable_decl_assignments ;10
| type_declaration
...
```ebnf
data_type ::=
```

// from A.2.2.1
integer_vector_type [ signing ] { packed_dimension }
| integer_atom_type [ signing ]
| non_integer_type
| struct_union [ packed [ signing ] ] { struct_union_member { struct_union_member } }
{ packed_dimension }13
| enum [ enum_base_type ] { enum_name_declaration { , enum_name_declaration } }
{ packed_dimension }
| string
| chandle
| virtual [ interface ] interface_identifier [ parameter_value_assignment ] [ . modport_identifier ]
| [ class_scope | package_scope ] type_identifier { packed_dimension }
| class_type
| event
| ps_covergroup_identifier
| type_reference14
```ebnf
integer_type ::= integer_vector_type | integer_atom_type
integer_atom_type ::= byte | shortint | int | longint | integer | time
integer_vector_type ::= bit | logic | reg
non_integer_type ::= shortreal | real | realtime
signing ::= signed | unsigned
simple_type ::= integer_type | non_integer_type | ps_type_identifier | ps_parameter_identifier
data_type_or_void ::= data_type | void
```

Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
101
Copyright © 2018 IEEE. All rights reserved.
```ebnf
variable_decl_assignment ::=
```

// from A.2.4
variable_identifier { variable_dimension } [ = expression ]
| dynamic_array_variable_identifier unsized_dimension { variable_dimension }
[ = dynamic_array_new ]
| class_variable_identifier [ = class_new ]
10) In a data_declaration that is not within a procedural context, it shall be illegal to use the automatic keyword. In
a data_declaration, it shall be illegal to omit the explicit data_type before a list_of_variable_decl_assignments
unless the var keyword is used.
13) When a packed dimension is used with the struct or union keyword, the packed keyword shall also be used.
14) When a type_reference is used in a net declaration, it shall be preceded by a net type keyword; and when it is used
in a variable declaration, it shall be preceded by the var keyword.
Syntax 6-3—Syntax for variable declarations (excerpt from Annex A)
One form of variable declaration consists of a data type followed by one or more instances.
shortint s1, s2[0:9];
Another form of variable declaration begins with the keyword var. The data type is optional in this case. If
a data type is not specified or if only a range and/or signing is specified, then the data type is implicitly
declared as logic.
var byte my_byte;
// equivalent to "byte my_byte;"
var v;
// equivalent to "var logic v;"
var [15:0] vw;
// equivalent to "var logic [15:0] vw;"
var enum bit { clear, error } status;
input var logic data_in;
var reg r;
If a set of variables share the same characteristics, they can be declared in the same declaration statement.
A variable can be declared with an initializer, for example:
int i = 0;
Setting the initial value of a static variable as part of the variable declaration (including static class
members) shall occur before any initial or always procedures are started (also see 6.21 and 10.5 on variable
initialization with static and automatic lifetimes).
NOTE—In IEEE Std 1364-2005, an initialization value specified as part of the declaration was executed as if the
assignment were made from an initial procedure, after simulation has started.
Initial values are not constrained to simple constants; they can include run-time expressions, including
dynamic memory allocation. For example, a static class handle or a mailbox can be created and initialized by
calling its new method (see 15.4.1), or static variables can be initialized to random values by calling the
$urandom system task. This may require a special pre-initial pass at run time.
Table 6-7 contains the default values for variables if no initializer is specified.
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
102
Copyright © 2018 IEEE. All rights reserved.
Nets and variables can be assigned negative values, but only signed types shall retain the significance of the
sign. The byte, shortint, int, integer, and longint types are signed types by default. Other net and
variable types can be explicitly declared as signed. See 11.4.3.1 for a description of how signed and
unsigned nets and variables are treated by certain operators.
### 6.9 Vector declarations

A data object declared as reg, logic, or bit (or as a matching user-defined type or implicitly as logic)
without a range specification shall be considered 1-bit wide and is known as a scalar. A multibit data object
of one of these types shall be declared by specifying a range and is known as a vector. Vectors are packed
arrays of scalars (see 7.4).
#### 6.9.1 Specifying vectors

The range specification ([msb_constant_expression : lsb_constant_expression]) gives addresses to the
individual bits in a multibit reg, logic, or bit vector. The most significant bit, specified by the msb
constant expression, is the left-hand value in the range, and the least significant bit, specified by the lsb
constant expression, is the right-hand value in the range.
Both the msb constant expression and the lsb constant expression shall be constant integer expressions. The
msb and lsb constant expressions (see 11.2.1) may be any integer value—positive, negative, or zero. It shall
be illegal for them to contain any unknown (x) or high-impedance bits. The lsb value may be greater than,
equal to, or less than the msb value.
Vectors shall obey laws of arithmetic modulo-2 to the power n (2n), where n is the number of bits in the
vector. Vectors of reg, logic, and bit types shall be treated as unsigned quantities, unless declared to be
signed or connected to a port that is declared to be signed (see 23.2.2.1 and 23.3.3.8).
Examples:
wand w;
// a scalar "wand" net
Table 6-7—Default variable initial values
Type
Default initial value
4-state integral
'X
2-state integral
'0
real, shortreal
0.0
Enumeration
Base type default initial value
string
"" (empty string)
event
New event
class
null
interface class
null
chandle (Opaque handle)
null
virtual interface
null
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
