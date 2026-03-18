---
title: "Section 14: Specify blocks"
document: "Verilog Hardware Description Language Reference Manual"
standard: "IEEE 1364-2005"
domain: "Verilog"
section: "14"
source_txt: "section-14-specify-blocks.txt"
source_pdf: "/Users/richarddje/Documents/github/Verilog-LRM-IEEE-1364-2005.pdf"
---

# Section 14: Specify blocks

IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
211
## 14. Specify blocks

Two types of HDL constructs are often used to describe delays for structural models such as ASIC cells.
They are as follows:
—
Distributed delays, which specify the time it takes events to propagate through gates and nets inside
the module (see 7.14)
—
Module path delays, which describe the time it takes an event at a source (input port or inout port) to
propagate to a destination (output port or inout port)
This clause describes how paths are specified in a module and how delays are assigned to these paths.
### 14.1 Specify block declaration

A block statement called the specify block is the vehicle for describing paths between a source and a
destination and for assigning delays to these paths. The syntax for specify blocks is shown in Syntax 14-1.
Syntax 14-1—Syntax for specify block
The specify block shall be bounded by the keywords specify and endspecify, and it shall appear inside a
module declaration. The specify block can be used to perform the following tasks:
—
Describe various paths across the module.
—
Assign delays to those paths.
—
Perform timing checks to ensure that events occurring at the module inputs satisfy the timing
constraints of the device described by the module (see Clause 15).
The paths described in the specify block, called module paths, pair a signal source with a signal destination.
The source may be unidirectional (an input port) or bidirectional (an inout port) and is referred to as the
module path source. Similarly, the destination may be unidirectional (an output port) or bidirectional (an
inout port) and is referred to as the module path destination.
For example:
specify
specparam tRise_clk_q = 150, tFall_clk_q = 200;
specparam tSetup = 70;
(clk => q) = (tRise_clk_q, tFall_clk_q);
$setup(d, posedge clk, tSetup);
endspecify
```ebnf
specify_block ::= (From A.7.1)
```

specify { specify_item } endspecify
```ebnf
specify_item ::=
```

specparam_declaration
| pulsestyle_declaration
| showcancelled_declaration
| path_declaration
| system_timing_check
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
212
Copyright © 2006 IEEE. All rights reserved.
The first two lines following the keyword specify declare specify parameters, which are discussed in 4.10.3.
The line following the declarations of specify parameters describes a module path and assigns delays to that
module path. The specify parameters determine the delay assigned to the module path. Specifying module
paths is presented in 14.2. Assigning delays to module paths is discussed in 14.3. The line preceding the
keyword endspecify instantiates one of the system timing checks, which are discussed further in Clause 15.
### 14.2 Module path declarations

There are two steps required to set up module path delays in a specify block:
a)
Describe the module paths.
b)
Assign delays to those paths (see 14.3).
The syntax of the module path declaration is described in Syntax 14-2.
Syntax 14-2—Syntax for module path declaration
A module path may be described as a simple path, an edge-sensitive path, or a state-dependent path. A
module path shall be defined inside a specify block as a connection between a source signal and a
destination signal. Module paths can connect any combination of vectors and scalars.
For example:
Figure 14-1 illustrates a circuit with module path delays. More than one source (A, B, C, and D) may have a
module path to the same destination (Q), and different delays may be specified for each input to output path.
```ebnf
path_declaration ::= (From A.7.2)
```

simple_path_declaration ;
| edge_sensitive_path_declaration ;
| state_dependent_path_declaration ;
MODULE PATHS:
from A to Q
from B to Q
from C to Q
from D to Q
= module path delay
n
A
B
C
D
Q
22
10
12
18
Figure 14-1—Module path delays
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
213
#### 14.2.1 Module path restrictions

Module paths have the following restrictions:
—
The module path source shall be a net that is connected to a module input port or inout port.
—
The module path destination shall be a net or variable that is connected to a module output port or
inout port.
—
The module path destination shall have only one driver inside the module.
#### 14.2.2 Simple module paths

The syntax for specifying a simple module path is given in Syntax 14-3.
Syntax 14-3—Syntax for simple module path
Simple paths can be declared in one of two forms:
—
Source *> destination
—
Source => destination
The symbols *> and => each represent a different kind of connection between the module path source and
the module path destination. The operator *> establishes a full connection between source and destination.
The operator => establishes a parallel connection between source and destination. See 14.2.5 for a
description of full connection and parallel connection paths.
```ebnf
simple_path_declaration ::= (From A.7.2)
```

parallel_path_description = path_delay_value
| full_path_description = path_delay_value
```ebnf
parallel_path_description ::=
```

( specify_input_terminal_descriptor [ polarity_operator ] =>
specify_output_terminal_descriptor )
```ebnf
full_path_description ::=
```

( list_of_path_inputs [ polarity_operator ] *> list_of_path_outputs )
```ebnf
list_of_path_inputs ::=
```

specify_input_terminal_descriptor { , specify_input_terminal_descriptor }
```ebnf
list_of_path_outputs ::=
```

specify_output_terminal_descriptor { , specify_output_terminal_descriptor }
```ebnf
specify_input_terminal_descriptor ::= (From A.7.3)
```

input_identifier [ [ constant_range_expression ] ]
```ebnf
specify_output_terminal_descriptor ::=
```

output_identifier [ [ constant_range_expression ] ]
```ebnf
input_identifier ::=
```

input_port_identifier | inout_port_identifier
```ebnf
output_identifier ::=
```

output_port_identifier | inout_port_identifier
```ebnf
polarity_operator ::= (From A.7.4)
```

+ | -
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
214
Copyright © 2006 IEEE. All rights reserved.
For example:
The following three examples illustrate valid simple module path declarations:
(A => Q) = 10;
(B => Q) = (12);
(C, D *> Q) = 18;
#### 14.2.3 Edge-sensitive paths

When a module path is described using an edge transition at the source, it is called an edge-sensitive path.
The edge-sensitive path construct is used to model the timing of input-to-output delays, which only occur
when a specified edge occurs at the source signal.
The syntax of the edge-sensitive path declaration is shown in Syntax 14-4.
Syntax 14-4—Syntax for edge-sensitive path declaration
The edge identifier may be one of the keywords posedge or negedge, associated with an input terminal
descriptor, which may be any input port or inout port. If a vector port is specified as the input terminal
descriptor, the edge transition shall be detected on the least significant bit. If the edge transition is not
specified, the path shall be considered active on any transition at the input terminal.
An edge-sensitive path may be specified with full connections (*>) or parallel connections (=>). For parallel
connections (=>), the destination shall be any scalar output or inout port or the bit-select of a vector output
or inout port. For full connections (*>), the destination shall be a list of one or more of the vector or scalar
output and inout ports, and bit-selects or part-selects of vector output and inout ports. See 14.2.5 for a
description of parallel paths and full connection paths.
The data source expression is an arbitrary expression, which serves as a description of the flow of data to the
path destination. This arbitrary data path description does not affect the actual propagation of data or events
through the model; how an event at the data path source propagates to the destination depends on the internal
logic of the module. The polarity operator describes whether the data path is inverting or noninverting.
For example:
Example 1—The following example demonstrates an edge-sensitive path declaration with a positive polarity
operator:
( posedge clock => ( out +: in ) ) = (10, 8);
```ebnf
edge_sensitive_path_declaration ::= (From A.7.4)
```

parallel_edge_sensitive_path_description = path_delay_value
| full_edge_sensitive_path_description = path_delay_value
```ebnf
parallel_edge_sensitive_path_description ::=
```

( [ edge_identifier ] specify_input_terminal_descriptor =>
( specify_output_terminal_descriptor [ polarity_operator ] : data_source_expression ) )
```ebnf
full_edge_sensitive_path_description ::=
```

( [ edge_identifier ] list_of_path_inputs *>
( list_of_path_outputs [ polarity_operator ] : data_source_expression ) )
```ebnf
data_source_expression ::=
```

expression
```ebnf
edge_identifier ::=
```

posedge | negedge
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
215
In this example, at the positive edge of clock, a module path extends from clock to out using a rise delay
of 10 and a fall delay of 8. The data path is from in to out, and in is not inverted as it propagates to out.
Example 2—The following example demonstrates an edge-sensitive path declaration with a negative
polarity operator:
( negedge clock[0] => ( out -: in ) ) = (10, 8);
In this example, at the negative edge of clock[0], a module path extends from clock[0] to out using a
rise delay of 10 and a fall delay of 8. The data path is from in to out, and in is inverted as it propagates to
out.
Example 3—The following example demonstrates an edge-sensitive path declaration with no edge identifier:
( clock => ( out : in ) ) = (10, 8);
In this example, at any change in clock, a module path extends from clock to out.
#### 14.2.4 State-dependent paths

A state-dependent path makes it possible to assign a delay to a module path that affects signal propagation
delay through the path only if specified conditions are true.
A state-dependent path description includes the following items:
—
A conditional expression that, when evaluated true, enables the module path
—
A module path description
—
A delay expression that applies to the module path
The syntax for the state-dependent path declaration is shown in Syntax 14-5.
Syntax 14-5—Syntax for state-dependent paths
##### 14.2.4.1 Conditional expression

The operands in the conditional expression shall be constructed from the following:
—
Scalar or vector module input ports or inout ports or their bit-selects or part-selects
—
Locally defined variables or nets or their bit-selects or part-selects
—
Compile time constants (constant numbers and specify parameters)
Table 14-1 contains a list of valid operators that may be used in conditional expressions.
A conditional expression shall evaluate to true (1) for the state-dependent path to be assigned a delay value.
If the conditional expression evaluates to x or z, it shall be treated as true. If the conditional expression
evaluates to multiple bits, the least significant bit shall represent the result. The conditional expression can
have any number of operands and operators.
```ebnf
state_dependent_path_declaration ::= (From A.7.4)
```

if ( module_path_expression ) simple_path_declaration
| if ( module_path_expression ) edge_sensitive_path_declaration
| ifnone simple_path_declaration
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
216
Copyright © 2006 IEEE. All rights reserved.
##### 14.2.4.2 Simple state-dependent paths

If the path description of a state-dependent path is a simple path, then it is called a simple state-dependent
path. The simple path description is discussed in 14.2.2.
For example:
Example 1—The following example uses state-dependent paths to describe the timing of an XOR gate.
module XORgate (a, b, out);
input a, b;
output out;
xor x1 (out, a, b);
specify
specparam noninvrise = 1, noninvfall = 2;
specparam invertrise = 3, invertfall = 4;
if (a) (b => out) = (invertrise, invertfall);
if (b) (a => out) = (invertrise, invertfall);
if (~a)(b => out) = (noninvrise, noninvfall);
if (~b)(a => out) = (noninvrise, noninvfall);
endspecify
endmodule
In this example, the first two state-dependent paths describe a pair of output rise and fall delay times when
the XOR gate (x1) inverts a changing input. The last two state-dependent paths describe another pair of
output rise and fall delay times when the XOR gate buffers a changing input.
Example 2—The following example models a partial ALU. The state-dependent paths specify different
delays for different ALU operations.
module ALU (o1, i1, i2, opcode);
input [7:0] i1, i2;
input [2:1] opcode;
Table 14-1—List of valid operators in state-dependent path delay expression
Operator
Description
Operator
Description
~
bitwise negation
&
reduction and
&
bitwise and
|
reduction or
|
bitwise or
^
reduction xor
^
bitwise xor
~&
reduction nand
^~ ~^
bitwise xnor
~|
reduction nor
==
logical equality
^~ ~^
reduction xnor
!=
logical inequality
{}
concatenation
&&
logical and
{ {} }
replication
||
logical or
?:
conditional
!
logical not
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
217
output [7:0] o1;
//functional description omitted
specify
// add operation
if (opcode == 2'b00) (i1,i2 *> o1) = (25.0, 25.0);
// pass-through i1 operation
if (opcode == 2'b01) (i1 => o1) = (5.6, 8.0);
// pass-through i2 operation
if (opcode == 2'b10) (i2 => o1) = (5.6, 8.0);
// delays on opcode changes
(opcode *> o1) = (6.1, 6.5);
endspecify
endmodule
In the preceding example, the first three path declarations declare paths extending from operand inputs i1
and i2 to the o1 output. The delays on these paths are assigned to operations on the basis of the operation
specified by the inputs on opcode. The last path declaration declares a path from the opcode input to the o1
output.
##### 14.2.4.3 Edge-sensitive state-dependent paths

If the path description of a state-dependent path describes an edge-sensitive path, then the state-dependent
path is called an edge-sensitive state-dependent path. The edge-sensitive paths are discussed in 14.2.3.
Different delays can be assigned to the same edge-sensitive path as long as the following criteria are met:
—
The edge, condition, or both make each declaration unique.
—
The port is referenced in the same way in all path declarations (entire port, bit-select, or part-select).
For example:
Example 1
if ( !reset && !clear )
( posedge clock => ( out +: in ) ) = (10, 8) ;
In this example, if the positive edge of clock occurs when reset and clear are low, a module path
extends from clock to out using a rise delay of 10 and a fall delay of 8.
Example 2—The following example shows two edge-sensitive path declarations, each of which has a unique
edge:
specify
( posedge clk => ( q[0] : data ) ) = (10, 5);
( negedge clk => ( q[0] : data ) ) = (20, 12);
endspecify
Example 3—The following example shows two edge-sensitive path declarations, each of which has a unique
condition:
specify
if (reset)
( posedge clk => ( q[0] : data ) ) = (15, 8);
if (!reset && cntrl)
( posedge clk => ( q[0] : data ) ) = (6, 2);
endspecify
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
218
Copyright © 2006 IEEE. All rights reserved.
Example 4—The two state-dependent path declarations shown below are not legal because even though they
have different conditions, the destinations are not specified in the same way: the first destination is a part-
select, the second is a bit-select.
specify
if (reset)
(posedge clk => (q[3:0]:data)) = (10,5);
if (!reset)
(posedge clk => (q[0]:data)) = (15,8);
endspecify
##### 14.2.4.4 The ifnone condition

The ifnone keyword is used to specify a default state-dependent path delay when all other conditions for the
path are false. The ifnone condition shall specify the same module path source and destination as the state-
dependent module paths. The following rules apply to module paths specified with the ifnone condition:
—
Only simple module paths may be described with an ifnone condition.
—
The state-dependent paths that correspond to the ifnone path may be either simple module paths or
edge-sensitive paths.
—
If there are no corresponding state-dependent module paths to the ifnone module path, then the
ifnone module path shall be treated the same as an unconditional simple module path.
—
It is illegal to specify both an ifnone condition for a module path and an unconditional simple
module path for the same module path.
For example:
Example 1—The following are valid state-dependent path combinations:
if (C1) (IN => OUT) = (1,1);
ifnone (IN => OUT) = (2,2);
// add operation
if (opcode == 2'b00) (i1,i2 *> o1) = (25.0, 25.0);
// pass-through i1 operation
if (opcode == 2'b01) (i1 => o1) = (5.6, 8.0);
// pass-through i2 operation
if (opcode == 2'b10) (i2 => o1) = (5.6, 8.0);
// all other operations
ifnone (i2 => o1) = (15.0, 15.0);
(posedge CLK => (Q +: D)) = (1,1);
ifnone (CLK => Q) = (2,2);
Example 2—The following module path description combination is illegal because it combines a state-
dependent path using an ifnone condition and an unconditional path for the same module path:
if (a) (b => out) = (2,2);
if (b) (a => out) = (2,2);
ifnone (a => out) = (1,1);
(a => out) = (1,1);
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
219
#### 14.2.5 Full connection and parallel connection paths

The operator *> shall be used to establish a full connection between source and destination. In a full
connection, every bit in the source shall connect to every bit in the destination. The module path source need
not have the same number of bits as the module path destination.
The full connection can handle most types of module paths because it does not restrict the size or number of
source signals and destination signals. The following situations require the use of full connections:
—
To describe a module path between a vector and a scalar
—
To describe a module path between vectors of different sizes
—
To describe a module path with multiple sources or multiple destinations in a single statement (see
14.2.6)
The operator => shall be used to establish a parallel connection between source and destination. In a parallel
connection, each bit in the source shall connect to one corresponding bit in the destination. Parallel module
paths can be created only between sources and destinations that contain the same number of bits.
Parallel connections are more restrictive than full connections. They only connect one source to one
destination, where each signal contains the same number of bits. Therefore, a parallel connection may only
be used to describe a module path between two vectors of the same size. Because scalars are 1 bit wide,
either *> or => may be used to set up bit-to-bit connections between two scalars.
For example:
Example 1—Figure 14-2 illustrates how a parallel connection differs from a full connection between two
4-bit vectors.
 Parallel module path
0
1
2
3
0
1
2
3
Input bits                   Output bits
0
1
2
3
0
1
2
3
Input bits                   Output bits
N = number of bits = 4
Number of paths = N =
Use
=>
to define path
4
bit-to-bit connections
Full module path
Number of paths = N * N =
Use
to define path
16
bit-to-vector connections
*>
Figure 14-2—Difference between parallel and full connection paths
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
220
Copyright © 2006 IEEE. All rights reserved.
Example 2—The following example shows module paths for a 2:1 multiplexor with two 8-bit inputs and
one 8-bit output:
module mux8 (in1, in2, s, q) ;
output [7:0] q;
input [7:0] in1, in2;
input s;
// Functional description omitted ...
specify
(in1 => q) = (3, 4) ;
(in2 => q) = (2, 3) ;
(s *> q) = 1;
endspecify
endmodule
The module path from s to q uses a full connection (*>) because it connects a scalar source—the 1-bit
select line—to a vector destination—the 8-bit output bus. The module paths from both input lines in1 and
in2 to q use a parallel connection (=>) because they set up parallel connections between two 8-bit buses.
#### 14.2.6 Declaring multiple module paths in a single statement

Multiple module paths may be described in a single statement by using the symbol *> to connect a comma-
separated list of sources to a comma-separated list of destinations. When describing multiple module paths
in one statement, the lists of sources and destinations may contain a mix of scalars and vectors of any size.
The connection in a multiple module path declaration is always a full connection.
For example:
(a, b, c *> q1, q2) = 10;
is equivalent to the following six individual module path assignments:
(a *> q1) = 10 ;
(b *> q1) = 10 ;
(c *> q1) = 10 ;
(a *> q2) = 10 ;
(b *> q2) = 10 ;
(c *> q2) = 10 ;
#### 14.2.7 Module path polarity

The polarity of a module path is an arbitrary specification indicating whether the direction of a signal
transition is inverted as it propagates from the input to the output. This arbitrary polarity description does not
affect the actual propagation of data or events through the model; how a rise or a fall at the source
propagates to the destination depends on the internal logic of the module.
Module paths may specify any of three polarities:
—
Unknown polarity
—
Positive polarity
—
Negative polarity
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
221
##### 14.2.7.1 Unknown polarity

By default, module paths shall have unknown polarity; that is, a transition at the path source may propagate
to the destination in an unpredictable way, as follows:
—
A rise at the source may cause a rise transition, a fall transition, or no transition at the destination.
—
A fall at the source may cause a rise transition, a fall transition, or no transition at the destination.
A module path specified either as a full connection or as a parallel connection, but without a polarity
operator + or -, shall be treated as a module path with unknown polarity.
For example:
// Unknown polarity
(In1 => q) = In_to_q ;
(s
*> q) = s_to_q ;
##### 14.2.7.2 Positive polarity

For module paths with positive polarity, any transition at the source may cause the same transition at the
destination, as follows:
—
A rise at the source may cause either a rise transition or no transition at the destination.
—
A fall at the source may cause either a fall transition or no transition at the destination.
A module path with positive polarity shall be specified by prefixing the + polarity operator to => or *>.
For example:
// Positive polarity
(In1 +=> q) = In_to_q ;
(s
+*> q) = s_to_q ;
##### 14.2.7.3 Negative polarity

For module paths with negative polarity, any transition at the source may cause the opposite transition at the
destination, as follows:
—
A rise at the source may cause either a fall transition or no transition at the destination.
—
A fall at the source may cause either a rise transition or no transition at the destination.
A module path with negative polarity shall be specified by prefixing the - polarity operator to => or *>.
For example:
// Negative polarity
(In1 -=> q) = In_to_q ;
(s  -*> q) = s_to_q ;
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
222
Copyright © 2006 IEEE. All rights reserved.
### 14.3 Assigning delays to module paths

The delays that occur at the module outputs where paths terminate shall be specified by assigning delay
values to the module path descriptions. The syntax for specifying delay values is shown in Syntax 14-6.
Syntax 14-6—Syntax for path delay value
In module path delay assignments, a module path description (see 14.2) is specified on the left-hand side,
and one or more delay values are specified on the right-hand side. The delay values may be optionally
enclosed in a pair of parentheses. There may be one, two, three, six, or twelve delay values assigned to a
module path, as described in 14.3.1. The delay values shall be constant expressions containing literals or
specparams, and there may be a delay expression of the form min:typ:max.
For example:
specify
// Specify Parameters
specparam tRise_clk_q = 45:150:270, tFall_clk_q=60:200:350;
specparam tRise_Control = 35:40:45, tFall_control=40:50:65;
// Module Path Assignments
(clk => q) = (tRise_clk_q, tFall_clk_q);
(clr, pre *> q) = (tRise_control, tFall_control);
endspecify
In the example above, the specify parameters declared following the specparam keyword specify values for
the module path delays. The module path assignments assign those module path delays to the module paths.
#### 14.3.1 Specifying transition delays on module paths

Each path delay expression may be a single value—representing the typical delay—or a colon-separated list
of three values—representing a minimum, typical, and maximum delay, in that order. If the path delay
expression results in a negative value, it shall be treated as zero. Table 14-2 describes how different path
delay values shall be associated with various transitions. The path delay expression names refer to the names
used in Syntax 14-6.
```ebnf
path_delay_value ::= (From A.7.4)
```

list_of_path_delay_expressions
| ( list_of_path_delay_expressions )
```ebnf
list_of_path_delay_expressions ::=
```

t_path_delay_expression
| trise_path_delay_expression , tfall_path_delay_expression
| trise_path_delay_expression , tfall_path_delay_expression , tz_path_delay_expression
| t01_path_delay_expression , t10_path_delay_expression , t0z_path_delay_expression ,
tz1_path_delay_expression , t1z_path_delay_expression , tz0_path_delay_expression
| t01_path_delay_expression , t10_path_delay_expression , t0z_path_delay_expression ,
tz1_path_delay_expression , t1z_path_delay_expression , tz0_path_delay_expression ,
t0x_path_delay_expression , tx1_path_delay_expression , t1x_path_delay_expression ,
tx0_path_delay_expression , txz_path_delay_expression , tzx_path_delay_expression
```ebnf
t_path_delay_expression ::=
```

path_delay_expression
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
223
For example:
// one expression specifies all transitions
(C => Q) = 20;
(C => Q) = 10:14:20;
// two expressions specify rise and fall delays
specparam tPLH1 = 12, tPHL1 = 25;
specparam tPLH2 = 12:16:22, tPHL2 = 16:22:25;
(C => Q) = ( tPLH1, tPHL1 ) ;
(C => Q) = ( tPLH2, tPHL2 ) ;
// three expressions specify rise, fall, and z transition delays
specparam tPLH1 = 12, tPHL1 = 22, tPz1 = 34;
specparam tPLH2 = 12:14:30, tPHL2 = 16:22:40, tPz2 = 22:30:34;
(C => Q) = (tPLH1, tPHL1, tPz1);
(C => Q) = (tPLH2, tPHL2, tPz2);
// six expressions specify transitions to/from 0, 1, and z
specparam t01 = 12, t10 = 16, t0z = 13,
 tz1 = 10, t1z = 14, tz0 = 34 ;
(C => Q) = ( t01, t10, t0z, tz1, t1z, tz0) ;
specparam  T01 = 12:14:24, T10 = 16:18:20, T0z = 13:16:30 ;
specparam  Tz1 = 10:12:16, T1z = 14:23:36, Tz0 = 15:19:34 ;
(C => Q) = ( T01, T10, T0z, Tz1, T1z, Tz0) ;
// twelve expressions specify all transition delays explicitly
specparam t01=10, t10=12, t0z=14, tz1=15, t1z=29, tz0=36,
 t0x=14, tx1=15, t1x=15, tx0=14, txz=20, tzx=30 ;
(C => Q) = (t01, t10, t0z, tz1, t1z, tz0,
t0x, tx1, t1x, tx0, txz, tzx) ;
Table 14-2—Associating path delay expressions with transitions
Number of path delay expressions specified
Transitions
1
2
3
6
12
## 0 -> 1

t
trise
trise
t01
t01
## 1 -> 0

t
tfall
tfall
t10
t10
## 0 -> z

t
trise
tz
t0z
t0z
z -> 1
t
trise
trise
tz1
tz1
## 1 -> z

t
tfall
tz
t1z
t1z
z -> 0
t
tfall
tfall
tz0
tz0
## 0 -> x

*
*
*
*
t0x
x -> 1
*
*
*
*
tx1
## 1 -> x

*
*
*
*
t1x
x -> 0
*
*
*
*
tx0
x -> z
*
*
*
*
txz
z -> x
*
*
*
*
tzx
* See 14.3.2.
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
224
Copyright © 2006 IEEE. All rights reserved.
#### 14.3.2 Specifying x transition delays

If the x transition delays are not explicitly specified, the calculation of delay values for x transitions is based
on the following two pessimistic rules:
—
Transitions from a known state to x shall occur as quickly as possible; that is, the shortest possible
delay shall be used for any transition to x.
—
Transitions from x to a known state shall take as long as possible; that is, the longest possible delay
shall be used for any transition from x.
Table 14-3 presents the general algorithm for calculating delay values for x transitions along with specific
examples. The following two groups of x transitions are represented in the table:
a)
Transition from a known state s to x:  s -> x
b)
Transition from x to a known state s:  x -> s
Table 14-3—Calculating delays for x transitions
X transition
Delay value
General algorithm
s -> x
minimum (s -> other known signals)
x -> s
maximum (other known signals -> s)
Specific transitions
## 0 -> x

minimum (0 -> z delay, 0 -> 1 delay)
## 1 -> x

minimum (1 -> z delay, 1 -> 0 delay)
z -> x
minimum (z -> 1 delay, z -> 0 delay)
x -> 0
maximum (z -> 0 delay, 1 -> 0 delay)
x -> 1
maximum (z -> 1 delay, 0 -> 1 delay)
x -> z
maximum (1 -> z delay, 0 -> z delay)
Usage:     (C => Q)  =  (5, 12, 17, 10, 6, 22) ;
## 0 -> x

minimum (17, 5) = 5
## 1 -> x

minimum (6, 12) = 6
z -> x
minimum (10, 22) = 10
x -> 0
maximum (22, 12) = 22
x -> 1
maximum (10, 5) = 10
x -> z
maximum (6, 17) = 17
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
225
#### 14.3.3 Delay selection

The simulator shall determine the proper delay to use when a specify path output must be scheduled to
transition. There may be specify paths to the output from more than one input, and the simulator must decide
which specify path to use.
The simulator shall do this by first determining which specify paths to the output are active. Active specify
paths are those whose input has transitioned most recently in time, and either they have no condition or their
conditions are true. In the presence of simultaneous input transitions, it is possible for many specify paths to
an output to be simultaneously active.
Once the active specify paths are identified, a delay must be selected from among them. This is done by
comparing the correct delay for the specific transition being scheduled from each specify path and choosing
the smallest.
For example:
Example 1
(A => Y) = (6, 9);
(B => Y) = (5, 11);
For a Y transition from 0 to 1, if A transitioned more recently than B, a delay of 6 will be chosen. But if B
transitioned more recently than A, a delay of 5 will be chosen. And if, the last time they transitioned, A and B
did so simultaneously, then the smallest of the two rise delays would be chosen, which is the rise delay from
B of 5. The fall delay from A of 9 would be chosen if Y was instead to transition from 1 to 0.
Example 2
if (MODE < 5) (A => Y) = (5, 9);
if (MODE < 4) (A => Y) = (4, 8);
if (MODE < 3) (A => Y) = (6, 5);
if (MODE < 2) (A => Y) = (3, 2);
if (MODE < 1) (A => Y) = (7, 7);
Anywhere from zero to five of these specify paths might be active depending upon the value of MODE. For
instance, when MODE is 2, the first three specify paths are active. A rise transition would select a delay of 4
because that is the smallest rise delay among the first three. A fall transition would select a delay of 5
because that is the smallest fall delay among the first three.
### 14.4 Mixing module path delays and distributed delays

If a module contains module path delays and distributed delays (delays on primitive instances within the
module), the larger of the two delays for each path shall be used.
For example:
Example 1—Figure 14-3 illustrates a simple circuit modeled with a combination of distributed delays and
path delays (only the D input to Q output path is illustrated). Here, the delay on the module path from input
D to output Q is 22, while the sum of the distributed delays is 0 + 1 = 1. Therefore, a transition on Q caused
by a transition on D will occur 22 time units after the transition on D.
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
226
Copyright © 2006 IEEE. All rights reserved.
Example 2—In Figure 14-4, the delay on the module path from D to Q is 22, but the distributed delays along
that module path now add up to 10 + 20 = 30. Therefore, an event on Q caused by an event on D will occur
## 30 time units after the event on D.

### 14.5 Driving wired logic

Module path output nets shall not have more than one driver within the module. Therefore, wired logic is not
allowed at module path outputs.
Figure 14-5 illustrates a violation of this wired-output rule and a method of avoiding the rule violation.
A
B
C
D
Q
1
0
0
22
  = distributed delay
n
= module path delay
n
Figure 14-3—Module path delays longer than distributed delays
A
B
C
D
Q
  = distributed delay
20
10
10
n
22
= module path delay
n
Figure 14-4—Module path delays shorter than distributed delays
E
F
G
H
S
(a)
(b)
E
F
G
H
S
Figure 14-5—Legal and illegal module paths
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
227
In Figure 14-5 (a), any module path to S is illegal because the path destination has two drivers.
Assuming signal S in Figure 14-5 (a) is a wired and, this limitation can be circumvented by replacing wired
logic with gated logic to create a single driver to the output. Figure 14-5 (b) shows how adding a third and
gate—the shaded gate—solves the problem for the module in Figure 14-5 (a).
The example in Figure 14-6 is also illegal. In this example, when the outputs Q and R are wired together, it
creates a condition where both paths have multiple drivers from within the same module.
Although multiple output drivers to a path destination are prohibited inside the same module, they are
allowed outside the module. The example in Figure 14-7 is legal because Q and R each have only one driver
within the module in which the module paths are specified.
A
B
C
D
R
Q
Figure 14-6—Illegal module paths
A
B
C
D
E
F
G
H
R
Q
Figure 14-7—Legal module paths
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
228
Copyright © 2006 IEEE. All rights reserved.
### 14.6 Detailed control of pulse filtering behavior

Two consecutive scheduled transitions closer together in time than the module path delay is deemed a pulse.
By default, pulses on a module path output are rejected. Consecutive transitions cannot be closer together
than the module path delay, and this is known as the inertial delay model of pulse propagation.
Pulse width ranges control how to handle a pulse presented at a module path output. They are as follows:
—
A pulse width range for which a pulse shall be rejected
—
A pulse width range for which a pulse shall be allowed to propagate to the path destination
—
A pulse width range for which a pulse shall generate a logic x on the path destination
Two pulse limit values define the pulse width ranges associated with each module path transition delay. The
pulse limit values are called the error limit and the reject limit. The error limit shall always be at least as
large as the reject limit. Pulses greater than or equal to the error limit pass unfiltered. Pulses less than the
error limit but greater than or equal to the reject limit are filtered to X. Pulses less than the reject limit are
rejected, and no pulse emerges. By default, both the error limit and the reject limit are set equal to the delay.
These default values yield full inertial pulse behavior, rejecting all pulses smaller than the delay.
In Figure 14-8, the rise delay from input A to output Y is 7, and the fall delay is 9. By default, the error limit
and the reject limit for the rise delay are both 7. The error limit and the reject limit for the fall delay are both
## 9. The pulse limits associated with the delay forming the trailing edge of the pulse determine whether and

how the pulse should be filtered. Waveform Y' shows the waveform resulting from no pulse filtering. The
width of the pulse is 2, which is less than the reject limit for the rise delay of 7; therefore, the pulse is filtered
as shown in waveform Y.
There are three ways to modify the pulse limits from their default values. First, the Verilog language
provides the PATHPULSE$ specparam to modify the pulse limits from their default values. Second,
invocation options can specify percentages applying to all module path delays to form the corresponding
error limits and reject limits. Third, SDF annotation can individually annotate the error limit and reject limit
of each module path transition delay.
(A =>Y ) = 7, 9;

pulse width = 4
// Pulse considered
// at module path output
pulse width = 4
Y
Y’
A
// Pulse is filtered
// Module path
// delay for a buffer
Figure 14-8—Example of pulse filtering
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
229
#### 14.6.1 Specify block control of pulse limit values

Pulse limit values may be set from within the specify block with the PATHPULSE$ specparam. The syntax
for using PATHPULSE$ to specify the reject limit and error limit values is given in Syntax 14-7.
If only the reject limit value is specified, it shall apply to both the reject limit and the error limit.
The reject limit and error limit may be specified for a specific module path. When no module path is
specified, the reject limit and error limit shall apply to all module paths defined in a module. If both
path-specific PATHPULSE$ specparams and a nonpath-specific PATHPULSE$ specparam appear in the
same module, then the path-specific specparams shall take precedence for the specified paths.
The module path input terminals and output terminals shall conform to the rules for module path inputs and
outputs, with the following restriction: the terminals may not be a bit-select or part-select of a vector.
When a module path declaration declares multiple paths, the PATHPULSE$ specparam shall only be
specified for the first path input terminal and the first path output terminal. The reject limit and error limit
specified shall apply to all other paths in the multiple path declaration. A PATHPULSE$ specparam that
specifies anything other than the first path input and path output terminals shall be ignored.
For example:
In the following example, the path (clk=>q) acquires a reject limit of 2 and an error limit of 9, as defined
by the first PATHPULSE$ declaration. The paths (clr*>q) and (pre*>q) receive a reject limit of 0 and
an error limit of 4, as specified by the second PATHPULSE$ declaration. The path (data=>q) is not
explicitly defined in any of the PATHPULSE$ declarations; therefore, it acquires reject and error limit of 3,
as defined by the last PATHPULSE$ declaration.
specify
(clk => q) = 12;
(data => q) = 10;
(clr, pre *> q) = 4;
specparam
PATHPULSE$clk$q = (2,9),
PATHPULSE$clr$q = (0,4),
PATHPULSE$ = 3;
endspecify
```ebnf
pulse_control_specparam ::= (From A.2.4)
```

PATHPULSE$ = ( reject_limit_value [ , error_limit_value ] )
| PATHPULSE$specify_input_terminal_descriptor$specify_output_terminal_descriptor
= ( reject_limit_value [ , error_limit_value ] )
```ebnf
error_limit_value ::=
```

limit_value
```ebnf
reject_limit_value ::=
```

limit_value
```ebnf
limit_value ::=
```

constant_mintypmax_expression
Syntax 14-7—Syntax for PATHPULSE$ pulse control
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
230
Copyright © 2006 IEEE. All rights reserved.
#### 14.6.2 Global control of pulse limit values

Two invocation options can specify percentages applying globally to all module path transition delays. The
error limit invocation option specifies the percentage of each module path transition delay used for its error
limit value. The reject limit invocation option specifies the percentage of each module path transition delay
used for its reject limit value. The percentage values shall be an integer between 0 and 100.
The default values for both the reject and error limit invocation options are 100%. When neither option is
present, then 100% of each module transition delay is used as the reject and error limits.
It is an error if the error limit percentage is smaller than the reject limit percentage. In such cases, the error
limit percentage is set equal to the reject limit percentage.
When both PATHPULSE$ and global pulse limit invocation options are present, the PATHPULSE$
values shall take precedence.
#### 14.6.3 SDF annotation of pulse limit values

SDF annotation can be used to specify the pulse limit values of module path transition delays. Clause 16
describes this in greater detail.
When PATHPULSE$, global pulse limit invocation options, and SDF annotation of pulse limit values are
present, SDF annotation values shall take precedence.
#### 14.6.4 Detailed pulse control capabilities

The default style of pulse filtering behavior has two drawbacks. First, pulse filtering to the X state may be
insufficiently pessimistic with an X state duration too short to be useful. Second, unequal delays can result in
pulse rejection whenever the trailing edge precedes the leading edge, leaving no indication that a pulse was
rejected. This subclause introduces more detailed pulse control capabilities.
##### 14.6.4.1 On-event versus on-detect pulse filtering

When an output pulse must be filtered to X, greater pessimism can be expressed if the module path output
transitions immediately to X (on-detect) instead of at the already scheduled transition time of the leading
edge of the pulse (on-event).
The on-event method of pulse filtering to X is the default. When an output pulse must be filtered to X, the
leading edge of the pulse becomes a transition to X, and the trailing edge becomes a transition from X. The
times of transition of the edges do not change.
Just like on-event, the on-detect method of pulse filtering changes the leading edge of the pulse into a
transition to X and the trailing edge to a transition from X, but the time of the leading edge is changed to
occur immediately upon detection of the pulse.
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
231
Figure 14-9 illustrates this behavior using a simple buffer with asymmetric rise/fall times and both the reject
limits and error limits equal to 0. An output waveform is shown for both on-detect and on-event approaches.
On-detect versus on-event behavior can be selected in two different ways. First, one may be selected
globally for all module path outputs through use of the on-detect or on-event invocation option. Second, one
may be selected locally through use of specify block pulse style declarations.
The syntax for pulse style declarations is shown in Syntax 14-8.
Syntax 14-8—Syntax for pulse style declarations
It is an error if a module path output appears in a pulse style declaration after it has already appeared in a
module path declaration.
The pulse style invocation options take precedence over pulse style specify block declarations.
```ebnf
pulsestyle_declaration ::= (From A.7.1)
```

pulsestyle_onevent list_of_path_outputs ;
| pulsestyle_ondetect list_of_path_outputs ;
in
rise/fall
4/6
out
in
## 12 14

18
10
out (on-event)
(default)
out (on-detect)
Figure 14-9—On-detect versus on-event
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
232
Copyright © 2006 IEEE. All rights reserved.
##### 14.6.4.2 Negative pulse detection

When the delays to a module path output are unequal, it is possible for the trailing edge of a pulse to be
scheduled for a time earlier than the schedule time of the leading edge, yielding a pulse with a negative
width. Under normal operation, if the schedule for a trailing pulse edge is earlier than the schedule for a
leading pulse edge, then the leading edge is cancelled. No transition takes place when the initial and final
states of the pulse are the same, leaving no indication a schedule was ever present.
Negative pulses can be indicated with the X state by use of the showcancelled style of behavior. When the
trailing edge of a pulse would be scheduled before the leading edge, this style causes the leading edge to be
scheduled to X and the trailing edge to be scheduled from X. With on-event pulse style, the schedule to X
replaces the leading edge schedule. With on-detect pulse style, the schedule to X is made immediately upon
detection of the negative pulse.
Showcancelled behavior can be enabled in two different ways. First, it may be enabled globally for all
module path outputs through use of the showcancelled and noshowcancelled invocation options. Second, it
may be enabled locally through use of specify block showcancelled declarations.
The syntax for showcancelled declarations is shown in Syntax 14-9.
Syntax 14-9—Syntax for showcancelled declarations
It is an error if a module path output appears in a showcancelled declaration after it has already appeared in
a module path declaration. The showcancelled invocation options take precedence over the showcancelled
specify block declarations.
```ebnf
showcancelled_declaration ::= (From A.7.1)
```

showcancelled list_of_path_outputs ;
| noshowcancelled list_of_path_outputs ;
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
233
The showcancelled behavior is illustrated in Figure 14-10, which shows a narrow pulse presented at the
input to a buffer with unequal rise/fall delays. This causes the trailing edge of the pulse to be scheduled
earlier than leading edge. The leading edge of the input pulse schedules an output event 6 units later at the
point marked by A. The pulse trailing edge occurs one time unit later, which schedules an output event 4
units later marked by point B. This second schedule on the output is for a time prior to the already existing
schedule for the leading output pulse edge.
The output waveform is shown for three different operating modes. The first waveform shows the default
behavior with showcancelled behavior not enabled and with the default on-event style. The second
waveform shows showcancelled behavior in conjunction with on-event. The last waveform shows
showcancelled behavior in conjunction with on-detect.
in
(in=>out)=(4,6);
out
in
out (default)

15
16
## 10 11

B
A
out (showcancelled with on-event)
out (showcancelled with on-detect)
Figure 14-10—Current event cancellation problem and correction
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
234
Copyright © 2006 IEEE. All rights reserved.
This same situation can also arise with nearly simultaneous input transitions, which is defined as two inputs
transitioning closer together in time than the difference in their respective delays to the output. Figure 14-11
shows waveforms for a two-input NAND gate where initially A is high and B is low. B transitions 0->1 at
time 10, causing a 1->0 output schedule at time 24. A transitions 1->0 at time 12, causing a 0->1 schedule
at time 22. Arrows mark the output transitions caused by the transitions on inputs A and B.
The output waveform is shown for three different operating modes. The first waveform shows the default
behavior with showcancelled behavior not enabled and with the default on-event style. The second shows
showcancelled behavior in conjunction with on-event. The third shows showcancelled behavior in
conjunction with on-detect.
out (default)
24
.
10
22
12
A
B
(A=>Q) = 10;
(B=>Q) = 14;
out (showcancelled with on-event)
out (showcancelled with on-detect)
Figure 14-11—NAND gate with nearly simultaneous input switching where one event
is scheduled prior to another that has not matured
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
235
One drawback of the on-event style with showcancelled behavior is that as the output pulse edges draw
closer together, the duration of the resulting X state becomes smaller. Figure 14-12 illustrates how the
on-detect style solves this problem.
For example:
Example 1
specify
(a=>out)=(2,3);
(b =>out)=(3,4);
endspecify
Because no pulse style or showcancelled declarations appear within the specify block, the compiler applies
the default modes of on-event and noshowcancelled.
Example 2
specify
(a=>out)=(2,3);
showcancelled out;
(b =>out)=(3,4);
endspecify
out (default)
10
A
B

(A=>Q) = 10
(B=>Q) = 14
14
24
out (showcancelled with on-event)
out (showcancelled with on-detect)
Figure 14-12—NAND gate with nearly simultaneous input switching with output event
scheduled at same time
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
236
Copyright © 2006 IEEE. All rights reserved.
This showcancelled declaration is in error because it follows use of out in a module path declaration. It
would be contradictory for out to have noshowcancelled behavior from input a, but showcancelled behavior
from input b.
Example 3—Both these specify blocks produce the same result. Outputs out and out_b are both declared
showcancelled and on-detect.
specify
showcancelled out;
pulsestyle_ondetect out;
(a => out) = (2,3);
(b => out) = (4,5);
showcancelled out_b;
pulsestyle_ondetect out_b;
(a => out_b) = (3,4);
(b => out_b) = (5,6);
endspecify
specify
showcancelled out,out_b;
pulsestyle_ondetect out,out_b;
(a => out) = (2,3);
(b => out) = (4,5);
(a => out_b) = (3,4);
(b => out_b) = (5,6);
endspecify
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
