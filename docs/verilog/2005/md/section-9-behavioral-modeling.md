---
title: "Section 9: Behavioral modeling"
document: "Verilog Hardware Description Language Reference Manual"
standard: "IEEE 1364-2005"
domain: "Verilog"
section: "9"
source_txt: "section-9-behavioral-modeling.txt"
source_pdf: "/Users/richarddje/Documents/github/Verilog-LRM-IEEE-1364-2005.pdf"
---

# Section 9: Behavioral modeling

IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
116
Copyright © 2006 IEEE. All rights reserved.
## 9. Behavioral modeling

The language constructs introduced so far allow hardware to be described at a relatively detailed level.
Modeling a circuit with logic gates and continuous assignments reflects quite closely the logic structure of
the circuit being modeled; however, these constructs do not provide the power of abstraction necessary for
describing complex high-level aspects of a system. The procedural constructs described in this clause are
well suited to tackling problems such as describing a microprocessor or implementing complex timing
checks.
This clause starts with a brief overview of a behavioral model to provide a context for many types of
behavioral statements in the Verilog HDL.
### 9.1 Behavioral model overview

Verilog behavioral models contain procedural statements that control the simulation and manipulate
variables of the data types previously described. These statements are contained within procedures. Each
procedure has an activity flow associated with it.
The activity starts at the control constructs initial and always. Each initial construct and each always
construct starts a separate activity flow. All of the activity flows are concurrent to model the inherent
concurrence of hardware. These constructs are formally described in 9.9.
The following example shows a complete Verilog behavioral model.
module behave;
reg [1:0] a, b;
initial begin
a = 'b1;
b = 'b0;
end
always begin
#50 a = ~a;
end
always begin
#100 b = ~b;
end
endmodule
During simulation of this model, all of the flows defined by the initial and always constructs start together at
simulation time zero. The initial constructs execute once, and the always constructs execute repetitively.
In this model, the reg variables a and b initialize to 1 and 0, respectively, at simulation time zero. The initial
construct is then complete and does not execute again during this simulation run. This initial construct
contains a begin-end block (also called a sequential block) of statements. In this begin-end block, a is
initialized first, followed by b.
The always constructs also start at time zero, but the values of the variables do not change until the times
specified by the delay controls (introduced by #) have elapsed. Thus, reg a inverts after 50 time units and
reg b inverts after 100 time units. Because the always constructs repeat, this model will produce two square
waves. The reg a toggles with a period of 100 time units, and reg b toggles with a period of 200 time units.
The two always constructs proceed concurrently throughout the entire simulation run.
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
117
### 9.2 Procedural assignments

As described in Clause 6, procedural assignments are used for updating reg, integer, time, real, realtime,
and memory data types. There is a significant difference between procedural assignments and continuous
assignments:
—
Continuous assignments drive nets and are evaluated and updated whenever an input operand
changes value.
—
Procedural assignments update the value of variables under the control of the procedural flow
constructs that surround them.
The right-hand side of a procedural assignment can be any expression that evaluates to a value. The left-
hand side shall be a variable that receives the assignment from the right-hand side. The left-hand side of a
procedural assignment can take one of the following forms:
—
reg, integer, real, realtime, or time data type: an assignment to the name reference of one of these
data types.
—
Bit-select of a reg, integer, or time data type: an assignment to a single bit that leaves the other bits
untouched.
—
Part-select of a reg, integer, or time data type: a part-select of one or more contiguous bits that
leaves the rest of the bits untouched.
—
Memory word: a single word of a memory.
—
Concatenation or nested concatenation of any of the above: a concatenation or nested concatenation
of any of the previous four forms. Such specification effectively partitions the result of the right-
hand expression and assigns the partition parts, in order, to the various parts of the concatenation or
nested concatenation.
As described in 5.4, when the right-hand side evaluates to fewer bits than the left-hand side, the right-hand
side value is padded to the size of the left-hand side. If the right-hand side is unsigned, it is padded according
to the rules specified in 5.4.1. If the right-hand side is signed, it is sign-extended.
The Verilog HDL contains two types of procedural assignment statements:
—
Blocking procedural assignment statements
—
Nonblocking procedural assignment statements
Blocking and nonblocking procedural assignment statements specify different procedural flows in
sequential blocks.
#### 9.2.1 Blocking procedural assignments

A blocking procedural assignment statement shall be executed before the execution of the statements that
follow it in a sequential block (see 9.8.1). A blocking procedural assignment statement shall not prevent the
execution of statements that follow it in a parallel block (see 9.8.2).
The syntax for a blocking procedural assignment is given in Syntax 9-1.
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
118
Copyright © 2006 IEEE. All rights reserved.
Syntax 9-1—Syntax for blocking assignments
In this syntax, variable_lvalue is a data type that is valid for a procedural assignment statement, = is the
assignment operator, and delay_or_event_control is the optional intra-assignment timing control. The
control can be either a delay_control (e.g., #6) or an event_control (e.g., @(posedge clk)). The
expression is the right-hand side value that shall be assigned to the left-hand side. If variable_lvalue
requires an evaluation, it shall be evaluated at the time specified by the intra-assignment timing control.
The = assignment operator used by blocking procedural assignments is also used by procedural continuous
assignments and continuous assignments.
For example:
The following examples show blocking procedural assignments:
rega = 0;
rega[3] = 1;
// a bit-select
rega[3:5] = 7;
// a part-select
mema[address] = 8'hff;
// assignment to a mem element
{carry, acc} = rega + regb;
// a concatenation
#### 9.2.2 The nonblocking procedural assignment

The nonblocking procedural assignment allows assignment scheduling without blocking the procedural
flow. The nonblocking procedural assignment statement can be used whenever several variable assignments
within the same time step can be made without regard to order or dependence upon each other.
The syntax for a nonblocking procedural assignment is given in Syntax 9-2.
```ebnf
blocking_assignment ::= (From A.6.2)
```

variable_lvalue = [ delay_or_event_control ] expression
```ebnf
delay_control ::= (From A.6.5)
```

# delay_value
| # ( mintypmax_expression )
```ebnf
delay_or_event_control ::=
```

delay_control
| event_control
| repeat ( expression ) event_control
```ebnf
event_control ::=
```

@ hierarchical_event_identifier
| @ ( event_expression )
| @*
| @ (*)
```ebnf
event_expression ::=
```

expression
| posedge expression
| negedge expression
| event_expression or event_expression
| event_expression , event_expression
```ebnf
variable_lvalue ::= (From A.8.5)
  hierarchical_variable_identifier [ { [ expression ] } [ range_expression ] ]
| { variable_lvalue { , variable_lvalue } }
```

Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
119
Syntax 9-2—Syntax for nonblocking assignments
In this syntax, variable_lvalue is a data type that is valid for a procedural assignment statement, <= is
the nonblocking assignment operator, and delay_or_event_control is the optional intra-assignment
timing control. If variable_lvalue requires an evaluation, it shall be evaluated at the same time as the
expression on the right-hand side. The order of evaluation of the variable_lvalue and the expression on
the right-hand side is undefined if timing control is not specified.
The nonblocking assignment operator is the same operator as the less-than-or-equal-to relational operator.
The interpretation shall be decided from the context in which <= appears. When <= is used in an expression,
it shall be interpreted as a relational operator; and when it is used in a nonblocking procedural assignment, it
shall be interpreted as an assignment operator.
The nonblocking procedural assignments shall be evaluated in two steps as discussed in Clause 11. These
two steps are shown in the following example:
```ebnf
nonblocking_assignment ::= (From A.6.2)
```

variable_lvalue <= [ delay_or_event_control ] expression
```ebnf
delay_control ::= (From A.6.5)
```

# delay_value
| # ( mintypmax_expression )
```ebnf
delay_or_event_control ::=
```

delay_control
| event_control
| repeat ( expression ) event_control
```ebnf
event_control ::=
```

@ hierarchical_event_identifier
| @ ( event_expression )
| @*
| @ (*)
```ebnf
event_expression ::=
```

expression
| posedge expression
| negedge expression
| event_expression or event_expression
| event_expression , event_expression
```ebnf
variable_lvalue ::= (From A.8.5)
  hierarchical_variable_identifier [ { [ expression ] } [ range_expression ] ]
| { variable_lvalue { , variable_lvalue } }
```

Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
120
Copyright © 2006 IEEE. All rights reserved.
Example 1
At the end of the time step means that the nonblocking assignments are the last assignments executed in a
time step—with one exception. Nonblocking assignment events can create blocking assignment events.
These blocking assignment events shall be processed after the scheduled nonblocking events.
Unlike an event or delay control for blocking assignments, the nonblocking assignment does not block the
procedural flow. The nonblocking assignment evaluates and schedules the assignment, but it does not block
the execution of subsequent statements in a begin-end block.
Example 2
As shown in the previous example, the simulator evaluates and schedules assignments for the end of the
current time step and can perform swapping operations with the nonblocking procedural assignments.
module evaluates2 (out);
output out;
reg a, b, c;
initial begin
a = 0;
b = 1;
c = 0;
end
always c = #5 ~c;
always @(posedge c) begin
a <= b; // evaluates, schedules,
b <= a; // and executes in two steps
end
endmodule
At posedge c, the simulator
evaluates the right-hand sides of
the nonblocking assignments and
schedules the assignments of the
new values at the end of the
nonblocking assign update events
(see 11.4).
Step 1:
a = 0
b = 1
Step 2:
When the simulator activates the
nonblocking assign update events,
the simulator updates the left-hand
side of each nonblocking assign-
ment statement.
Nonblocking
assignment
schedules
changes at
time 5
a = 1
b = 0
Assignment
values:
//non_block1.v
module non_block1;
reg a, b, c, d, e, f;
//blocking assignments
initial begin
a = #10 1; // a will be assigned 1 at time 10
b = #2 0; // b will be assigned 0 at time 12
c = #4 1; // c will be assigned 1 at time 16
end
//non-blocking assignments
initial begin
d <= #10 1; // d will be assigned 1 at time 10
e <= #2 0; // e will be assigned 0 at time 2
f <= #4 1; // f will be assigned 1 at time 4
end
endmodule
scheduled
changes at
time 2
e = 0
f = 1
d = 1
scheduled
changes at
time 4
scheduled
changes at
time 10
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
121
Example 3
The order of the execution of distinct nonblocking assignments to a given variable shall be preserved. In
other words, if there is clear ordering of the execution of a set of nonblocking assignments, then the order of
the resulting updates of the destination of the nonblocking assignments shall be the same as the ordering of
the execution (see 11.4.1).
Example 4
module multiple;
reg a;
initial a = 1;
// The assigned value of the reg is determinate
initial begin
a <= #4 0;
// schedules a = 0 at time 4
a <= #4 1;
// schedules a = 1 at time 4
end
// At time 4, a = 1
endmodule
If the simulator executes two procedural blocks concurrently and if these procedural blocks contain
nonblocking assignment operators to the same variable, the final value of that variable is indeterminate. For
example, the value of reg a is indeterminate in the following example:
Example 5
module multiple2;
reg a;
initial a = 1;
initial a <= #4 0;
// schedules 0 at time 4
initial a <= #4 1;
// schedules 1 at time 4
// At time 4, a = ??
// The assigned value of the reg is indeterminate
endmodule
//non_block1.v
module non_block1;
reg a, b;
initial begin
a = 0;
b = 1;
a <= b; // evaluates, schedules, and
b <= a; // executes in two steps
end
initial begin
$monitor ($time, ,"a = %b b = %b", a, b);
#100 $finish;
end
endmodule
The simulator evaluates the right-
hand side of the nonblocking
assignments and schedules the
assignments for the end of the
current time step.
Step 1:
Step 2:
At the end of the current time step,
the simulator updates the left-hand
side of each nonblocking assign-
ment statement.
a = 1
b = 0
assignment values:
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
122
Copyright © 2006 IEEE. All rights reserved.
The fact that two nonblocking assignments targeting the same variable are in different blocks is not by itself
sufficient to make the order of assignments to a variable indeterminate. For example, the value of reg a at
the end of time cycle 16 is determinate in the following example:
Example 6
module multiple3;
reg a;
initial #8 a <= #8 1;  // executed at time 8;
 // schedules an update of 1 at time 16
initial #12 a <= #4 0;  // executed at time 12;
 // schedules an update of 0 at time 16
// Because it is determinate that the update of a to the value 1
// is scheduled before the update of a to the value 0,
// then it is determinate that a will have the value 0
// at the end of time slot 16.
endmodule
The following example shows how the value of i[0] is assigned to r1 and how the assignments are
scheduled to occur after each time delay:
Example 7
module multiple4;
reg r1;
reg [2:0] i;
initial begin
// makes assignments to r1 without cancelling previous assignments
for (i = 0; i <= 5; i = i+1)
r1 <= # (i*10) i[0];
end
endmodule
### 9.3 Procedural continuous assignments

The procedural continuous assignments (using keywords assign and force) are procedural statements that
allow expressions to be driven continuously onto variables or nets. The syntax for these statements is given
in Syntax 9-3.
The left-hand side of the assignment in the assign statement shall be a variable reference or a concatenation
of variables. It shall not be a memory word (array reference) or a bit-select or a part-select of a variable.
In contrast, the left-hand side of the assignment in the force statement can be a variable reference or a net
reference. It can be a concatenation of any of the above. Bit-selects and part-selects of vector variables are
not allowed.
r1
10
20
30
40
50
0
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
123
#### 9.3.1 The assign and deassign procedural statements

The assign procedural continuous assignment statement shall override all procedural assignments to a
variable. The deassign procedural statement shall end a procedural continuous assignment to a variable. The
value of the variable shall remain the same until the variable is assigned a new value through a procedural
assignment or a procedural continuous assignment. The assign and deassign procedural statements allow, for
example, modeling of asynchronous clear/preset on a D-type edge-triggered flip-flop, where the clock is
inhibited when the clear or preset is active.
If the keyword assign is applied to a variable for which there is already a procedural continuous assignment,
then this new procedural continuous assignment shall deassign the variable before making the new
procedural continuous assignment.
For example:
The following example shows a use of the assign and deassign procedural statements in a behavioral
description of a D-type flip-flop with preset and clear inputs:
module dff (q, d, clear, preset, clock);
output q;
input d, clear, preset, clock;
reg q;
always @(clear or preset)
if (!clear)
assign q = 0;
else if (!preset)
assign q = 1;
else
deassign q;
always @(posedge clock)
q = d;
endmodule
```ebnf
net_assignment ::= (From A.6.1)
```

net_lvalue = expression
```ebnf
procedural_continuous_assignments ::= (From A.6.2)
```

assign variable_assignment
| deassign variable_lvalue
| force variable_assignment
| force net_assignment
| release variable_lvalue
| release net_lvalue
```ebnf
variable_assignment ::=
```

variable_lvalue = expression
```ebnf
net_lvalue ::= (From A.8.5)
  hierarchical_net_identifier [ { [ constant_expression ] } [ constant_range_expression ] ]
| { net_lvalue { , net_lvalue } }
variable_lvalue ::=
  hierarchical_variable_identifier [ { [ expression ] } [ range_expression ] ]
| { variable_lvalue { , variable_lvalue } }
```

Syntax 9-3—Syntax for procedural continuous assignments
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
124
Copyright © 2006 IEEE. All rights reserved.
If either clear or preset is low, then the output q will be held continuously to the appropriate constant
value, and a positive edge on the clock will not affect q. When both the clear and preset are high, then
q is deassigned.
#### 9.3.2 The force and release procedural statements

Another form of procedural continuous assignment is provided by the force and release procedural
statements. These statements have a similar effect to the assign-deassign pair, but a force can be applied to
nets as well as to variables. The left-hand side of the assignment can be a variable, a net, a constant bit-select
of a vector net, a part-select of a vector net, or a concatenation. It cannot be a memory word (array reference)
or a bit-select or a part-select of a vector variable.
A force statement to a variable shall override a procedural assignment or an assign procedural continuous
assignment to the variable until a release procedural statement is executed on the variable. When released,
then if the variable does not currently have an active assign procedural continuous assignment, the variable
shall not immediately change value. The variable shall maintain its current value until the next procedural
assignment or procedural continuous assignment to the variable. Releasing a variable that currently has an
active assign procedural continuous assignment shall immediately reestablish that assignment.
A force procedural statement on a net shall override all drivers of the net—gate outputs, module outputs, and
continuous assignments—until a release procedural statement is executed on the net. When released,  the net
shall immediately be assigned the value determined by the drivers of the net.
For example:
module test;
reg a, b, c, d;
wire e;
and and1 (e, a, b, c);
initial begin
$monitor("%d d=%b,e=%b", $stime, d, e);
assign d = a & b & c;
a = 1;
b = 0;
c = 1;
#10;
force d = (a | b | c);
force e = (a | b | c);
#10;
release d;
release e;
#10 $finish;
end
endmodule
Results:
## 0 d=0,e=0

## 10 d=1,e=1

## 20 d=0,e=0

In this example, an and gate instance and1 is “patched” to act like an or gate by a force procedural
statement that forces its output to the value of its ORed inputs, and an assign procedural statement of
ANDed values is “patched” to act like an assign statement of ORed values.
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
125
The right-hand side of a procedural continuous assignment or a force statement can be an expression. This
shall be treated just as a continuous assignment; that is, if any variable on the right-hand side of the
assignment changes, the assignment shall be reevaluated while the assign or force is in effect. For example:
force a = b + f(c) ;
Here, if b changes or c changes, a will be forced to the new value of the expression b+f(c).
### 9.4 Conditional statement

The conditional statement (or if-else statement) is used to make a decision about whether a statement is
executed. Formally, the syntax is given in Syntax 9-4.
Syntax 9-4—Syntax for if statement
If the expression evaluates to true (that is, has a nonzero known value), the first statement shall be executed.
If it evaluates to false (that is, has a zero value or the value is x or z), the first statement shall not execute. If
there is an else statement and expression is false, the else statement shall be executed.
Because the numeric value of the if expression is tested for being zero, certain shortcuts are possible. For
example, the following two statements express the same logic:
if (expression)
if (expression != 0)
Because the else part of an if-else is optional, there can be confusion when an else is omitted from a nested if
sequence. This is resolved by always associating the else with the closest previous if that lacks an else. In the
example below, the else goes with the inner if, as shown by indentation.
if (index > 0)
if (rega > regb)
result = rega;
else
// else applies to preceding if
result = regb;
If that association is not desired, a begin-end block statement shall be used to force the proper association, as
shown below.
if (index > 0) begin
if (rega > regb)
result = rega;
end
else result = regb;
```ebnf
conditional_statement ::= (From A.6.6)
```

if ( expression )
statement_or_null [ else statement_or_null ]
| if_else_if_statement
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
126
Copyright © 2006 IEEE. All rights reserved.
#### 9.4.1 If-else-if construct

The construction in Syntax 9-5 occurs so often that it is worth a brief separate discussion:
Syntax 9-5—Syntax for if-else-if construct
This sequence of if statements (known as an if-else-if construct) is the most general way of writing a
multiway decision. The expressions shall be evaluated in order. If any expression is true, the statement
associated with it shall be executed, and this shall terminate the whole chain. Each statement is either a
single statement or a block of statements.
The last else part of the if-else-if construct handles the none-of-the-above or default case where none of the
other conditions were satisfied. Sometimes there is no explicit action for the default. In that case, the trailing
else statement can be omitted, or it can be used for error checking to catch an impossible condition.
For example:
The following module fragment uses the if-else statement to test the variable index to decide whether one
of three modify_segn regs has to be added to the memory address and which increment is to be added to
the index reg. The first ten lines declare the regs and parameters.
// declare regs and parameters
reg [31:0] instruction, segment_area[255:0];
reg [7:0] index;
reg [5:0] modify_seg1,
modify_seg2,
modify_seg3;
parameter
segment1 = 0,
inc_seg1 = 1,
segment2 = 20, inc_seg2 = 2,
segment3 = 64, inc_seg3 = 4,
data = 128;
// test the index variable
if (index < segment2) begin
instruction = segment_area [index + modify_seg1];
index = index + inc_seg1;
end
else if (index < segment3) begin
instruction = segment_area [index + modify_seg2];
index = index + inc_seg2;
end
else if (index < data) begin
instruction = segment_area [index + modify_seg3];
index = index + inc_seg3;
end
else
instruction = segment_area [index];
```ebnf
if_else_if_statement ::= (From A.6.6)
```

if ( expression ) statement_or_null
{ else if ( expression ) statement_or_null }
[ else statement_or_null ]
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
127
### 9.5 Case statement

The case statement is a multiway decision statement that tests whether an expression matches one of a
number of other expressions and branches accordingly. The case statement has the syntax shown in
Syntax 9-6.
Syntax 9-6—Syntax for case statement
The default statement shall be optional. Use of multiple default statements in one case statement shall be
illegal.
The case expression and the case item expression can be computed at run time; neither expression is
required to be a constant expression.
For example:
A simple example of the use of the case statement is the decoding of reg rega to produce a value for
result as follows:
reg [15:0] rega;
reg [9:0] result;
case (rega)
16'd0: result = 10'b0111111111;
16'd1: result = 10'b1011111111;
16'd2: result = 10'b1101111111;
16'd3: result = 10'b1110111111;
16'd4: result = 10'b1111011111;
16'd5: result = 10'b1111101111;
16'd6: result = 10'b1111110111;
16'd7: result = 10'b1111111011;
16'd8: result = 10'b1111111101;
16'd9: result = 10'b1111111110;
default result = 'bx;
endcase
The case expression given in parentheses shall be evaluated exactly once and before any of the case item
expressions. The case item expressions shall be evaluated and compared in the exact order in which they are
given. If there is a default case item, it is ignored during this linear search. During the linear search, if one of
the case item expressions matches the case expression given in parentheses, then the statement associated
with that case item shall be executed, and the linear search shall terminate. If all comparisons fail and the
default item is given, then the default item statement shall be executed. If the default statement is not given
and all of the comparisons fail, then none of the case item statements shall be executed.
```ebnf
case_statement ::= (From A.6.7)
```

case ( expression )
case_item { case_item } endcase
| casez ( expression )
case_item { case_item } endcase
| casex ( expression )
case_item { case_item } endcase
```ebnf
case_item ::=
```

expression { , expression } : statement_or_null
| default [ : ] statement_or_null
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
128
Copyright © 2006 IEEE. All rights reserved.
Apart from syntax, the case statement differs from the multiway if-else-if construct in two important ways:
a)
The conditional expressions in the if-else-if construct are more general than comparing one expres-
sion with several others, as in the case statement.
b)
The case statement provides a definitive result when there are x and z values in an expression.
In a case expression comparison, the comparison only succeeds when each bit matches exactly with respect
to the values 0, 1, x, and z. As a consequence, care is needed in specifying the expressions in the case
statement. The bit length of all the expressions shall be equal so that exact bitwise matching can be
performed. The length of all the case item expressions, as well as the case expression in the parentheses,
shall be made equal to the length of the longest case expression and case item expression. If any of these
expressions is unsigned, then all of them shall be treated as unsigned. If all of these expressions are signed,
then they shall be treated as signed.
The reason for providing a case expression comparison that handles the x and z values is that it provides a
mechanism for detecting such values and reducing the pessimism that can be generated by their presence.
For example:
Example 1—The following example illustrates the use of a case statement to handle x and z values properly:
case (select[1:2])
2'b00: result = 0;
2'b01: result = flaga;
2'b0x,
2'b0z: result = flaga ? 'bx : 0;
2'b10: result = flagb;
2'bx0,
2'bz0: result = flagb ? 'bx : 0;
default result = 'bx;
endcase
In this example, if select[1] is 0 and flaga is 0, then even if the value of select[2] is x or z, result
should be 0—which is resolved by the third case.
Example 2—The following example shows another way to use a case statement to detect x and z values:
case (sig)
1'bz: $display("signal is floating");
1'bx: $display("signal is unknown");
default: $display("signal is %b", sig);
endcase
#### 9.5.1 Case statement with do-not-cares

Two other types of case statements are provided to allow handling of do-not-care conditions in the case
comparisons. One of these treats high-impedance values (z) as do-not-cares, and the other treats both
high-impedance and unknown (x) values as do-not-cares.
These case statements can be used in the same way as the traditional case statement, but they begin with
keywords casez and casex, respectively.
Do-not-care values (z values for casez, z and x values for casex) in any bit of either the case expression or
the case items shall be treated as do-not-care conditions during the comparison, and that bit position shall
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
129
not be considered. The do-not-care conditions in case expression can be used to control dynamically which
bits should be compared at any time.
The syntax of literal numbers allows the use of the question mark (?) in place of z in these case statements.
This provides a convenient format for specification of do-not-care bits in case statements.
For example:
Example 1—The following is an example of the casez statement. It demonstrates an instruction decode,
where values of the most significant bits select which task should be called. If the most significant bit of ir
is a 1, then the task instruction1 is called, regardless of the values of the other bits of ir.
reg [7:0] ir;
casez (ir)
8'b1???????: instruction1(ir);
8'b01??????: instruction2(ir);
8'b00010???: instruction3(ir);
8'b000001??: instruction4(ir);
endcase
Example 2—The following is an example of the casex statement. It demonstrates an extreme case of how do-
not-care conditions can be dynamically controlled during simulation. In this case, if r = 8'b01100110,
then the task stat2 is called.
reg [7:0] r, mask;
mask = 8'bx0x0x0x0;
casex (r ^ mask)
8'b001100xx: stat1;
8'b1100xx00: stat2;
8'b00xx0011: stat3;
8'bxx010100: stat4;
endcase
#### 9.5.2 Constant expression in case statement

A constant expression can be used for case expression. The value of the constant expression shall be
compared against case item expressions.
For example:
The following example demonstrates the usage by modeling a 3-bit priority encoder:
reg [2:0] encode ;
case (1)
encode[2] : $display("Select Line 2") ;
encode[1] : $display("Select Line 1") ;
encode[0] : $display("Select Line 0") ;
default $display("Error: One of the bits expected ON");
endcase
In this example, the case expression is a constant expression (1). The case items are expressions (bit-selects)
and are compared against the constant expression for a match.
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
130
Copyright © 2006 IEEE. All rights reserved.
### 9.6 Looping statements

There are four types of looping statements. These statements provide a means of controlling the execution of
a statement zero, one, or more times.
forever
Continuously executes a statement.
repeat
Executes a statement a fixed number of times. If the expression evaluates to unknown or
high impedance, it shall be treated as zero, and no statement shall be executed.
while
Executes a statement until an expression becomes false. If the expression starts out false,
the statement shall not be executed at all.
for
Controls execution of its associated statement(s) by a three-step process, as follows:
a)
Executes an assignment normally used to initialize a variable that controls the num-
ber of loops executed.
b)
Evaluates an expression. If the result is zero, the for loop shall exit. If it is not zero,
the for loop shall execute its associated statement(s) and then perform step c). If the
expression evaluates to an unknown or high-impedance value, it shall be treated as
zero.
c)
Executes an assignment normally used to modify the value of the loop-control vari-
able, then repeats step b).
Syntax 9-7 shows the syntax for various looping statements.
Syntax 9-7—Syntax for looping statements
The rest of this subclause presents examples for three of the looping statements. The forever loop should
only be used in conjunction with the timing controls or the disable statement; therefore, this example is
presented in 9.7.2.
For example:
Example 1—Repeat statement: In the following example of a repeat loop, add and shift operators implement
a multiplier:
parameter size = 8, longsize = 16;
reg [size:1] opa, opb;
reg [longsize:1] result;
begin : mult
reg [longsize:1] shift_opa, shift_opb;
shift_opa = opa;
shift_opb = opb;
result = 0;
repeat (size) begin
```ebnf
loop_statement ::= (From A.6.8)
```

forever statement
| repeat ( expression ) statement
| while ( expression ) statement
| for ( variable_assignment ;  expression ; variable_assignment )
statement
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
131
if (shift_opb[1])
result = result + shift_opa;
shift_opa = shift_opa << 1;
shift_opb = shift_opb >> 1;
end
end
Example 2—While statement: The following example counts the number of logic 1 values in rega:
begin : count1s
reg [7:0] tempreg;
count = 0;
tempreg = rega;
while (tempreg) begin
if (tempreg[0])
count = count + 1;
tempreg = tempreg >> 1;
end
end
Example 3—For statement: The for statement accomplishes the same results as the following pseudo-code
that is based on the while loop:
begin
initial_assignment;
while (condition) begin
statement
step_assignment;
end
end
The for loop implements this logic while using only two lines, as shown in the pseudo-code below:
for (initial_assignment; condition; step_assignment)
statement
### 9.7 Procedural timing controls

The Verilog HDL has two types of explicit timing control over when procedural statements can occur. The
first type is a delay control, in which an expression specifies the time duration between initially
encountering the statement and when the statement actually executes. The delay expression can be a
dynamic function of the state of the circuit, but it can be a simple number that separates statement executions
in time. The delay control is an important feature when specifying stimulus waveform descriptions. It is
described in 9.7.1 and 9.7.7.
The second type of timing control is the event expression, which allows statement execution to be delayed
until the occurrence of some simulation event occurring in a procedure executing concurrently with this
procedure. A simulation event can be a change of value on a net or variable (an implicit event) or the
occurrence of an explicitly named event that is triggered from other procedures (an explicit event). Most
often, an event control is a positive or negative edge on a clock signal. Event control is discussed in 9.7.2
through 9.7.7.
The procedural statements encountered so far all execute without advancing simulation time. Simulation
time can advance by one of the following three methods:
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
132
Copyright © 2006 IEEE. All rights reserved.
—
A delay control, which is introduced by the symbol #
—
An event control, which is introduced by the symbol @
—
The wait statement, which operates like a combination of the event control and the while loop
Syntax 9-8 describes timing control in procedural statements.
Syntax 9-8—Syntax for procedural timing control
The gate and net delays also advance simulation time, as discussed in Clause 6. The three procedural timing
control methods are discussed in 9.7.1 through 9.7.7.
#### 9.7.1 Delay control

A procedural statement following the delay control shall be delayed in its execution with respect to the
procedural statement preceding the delay control by the specified delay. If the delay expression evaluates to
an unknown or high-impedance value, it shall be interpreted as zero delay. If the delay expression evaluates
to a negative value, it shall be interpreted as a twos-complement unsigned integer of the same size as a time
variable. Specify parameters are permitted in the delay expression. They can be overridden by SDF
annotation, in which case the expression is reevaluated.
For example:
Example 1—The following example delays the execution of the assignment by 10 time units:
#10 rega = regb;
Example 2—The next three examples provide an expression following the number sign (#). Execution of the
assignment is delayed by the amount of simulation time specified by the value of the expression.
#d rega = regb;
// d is defined as a parameter
#((d+e)/2) rega = regb;
// delay is average of d and e
#regr regr = regr + 1;
// delay is the value in regr
#### 9.7.2 Event control

The execution of a procedural statement can be synchronized with a value change on a net or variable or the
occurrence of a declared event. The value changes on nets and variable can be used as events to trigger
the execution of a statement. This is known as detecting an implicit event. The event can also be based on
```ebnf
delay_control ::= (From A.6.5)
```

# delay_value
| # ( mintypmax_expression )
```ebnf
event_control ::=
```

@ hierarchical_event_identifier
| @ ( event_expression )
| @*
| @ (*)
```ebnf
procedural_timing_control ::=
```

delay_control
| event_control
```ebnf
procedural_timing_control_statement ::=
| procedural_timing_control  statement_or_null
```

Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
133
the direction of the change, that is, toward the value 1 (posedge) or toward the value 0 (negedge). The
behavior of posedge and negedge events is shown in Table 9-1 and can be described as follows:
—
A negedge shall be detected on the transition from 1 to x, z, or 0, and from x or z to 0
—
A posedge shall be detected on the transition from 0 to x, z, or 1, and from x or z to 1
An implicit event shall be detected on any change in the value of the expression. An edge event shall be
detected only on the least significant bit of the expression. A change of value in any operand of the
expression without a change in the result of the expression shall not be detected as an event.
For example:
The following example shows illustrations of edge-controlled statements:
@r rega = regb; // controlled by any value change in the reg r
@(posedge clock) rega = regb; // controlled by posedge on clock
forever @(negedge clock) rega = regb; // controlled by negative edge
#### 9.7.3 Named events

A new data type, in addition to nets and variables, called event can be declared. An identifier declared as an
event data type is called a named event. A named event can be triggered explicitly. It can be used in an event
expression to control the execution of procedural statements in the same manner as event controls described
in 9.7.2. Named events can be made to occur from a procedure. This allows control over the enabling of
multiple actions in other procedures.
An event name shall be declared explicitly before it is used. Syntax 9-9 gives the syntax for declaring
events.
Syntax 9-9—Syntax for event declaration
Table 9-1—Detecting posedge and negedge
From
To
0
1
x
z
0
No edge
posedge
posedge
posedge
1
negedge
No edge
negedge
negedge
x
negedge
posedge
No edge
No edge
z
negedge
posedge
No edge
No edge
```ebnf
event_declaration ::= (From A.2.1.3)
```

event list_of_event_identifiers ;
```ebnf
list_of_event_identifiers ::= (From A.2.3)
```

event_identifier { dimension }
{ , event_identifier { dimension } }
```ebnf
dimension ::= (From A.2.5)
```

[ dimension_constant_expression : dimension_constant_expression ]
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
134
Copyright © 2006 IEEE. All rights reserved.
An event shall not hold any data. The following are the characteristics of a named event:
—
It can be made to occur at any particular time.
—
It has no time duration.
—
Its occurrence can be recognized by using the event control syntax described in 9.7.2.
A declared event is made to occur by the activation of an event triggering statement with the syntax given in
Syntax 9-10. An event is not made to occur by changing the index of an event array in an event control
expression.
Syntax 9-10—Syntax for event trigger
An event-controlled statement (for example, @trig rega = regb;) shall cause simulation of its
containing procedure to wait until some other procedure executes the appropriate event-triggering statement
(for example, -> trig).
Named events and event control give a powerful and efficient means of describing the communication
between, and synchronization of, two or more concurrently active processes. A basic example of this is a
small waveform clock generator that synchronizes control of a synchronous circuit by signalling the
occurrence of an explicit event periodically while the circuit waits for the event to occur.
#### 9.7.4 Event or operator

The logical or of any number of events can be expressed so that the occurrence of any one of the events
triggers the execution of the procedural statement that follows it. The keyword or or a comma character (,) is
used as an event logical or operator. A combination of these can be used in the same event expression.
Comma-separated sensitivity lists shall be synonymous to or-separated sensitivity lists.
For example:
The next two examples show the logical or of two and three events, respectively:
@(trig or enable) rega = regb; // controlled by trig or enable
@(posedge clk_a or posedge clk_b or trig) rega = regb;
The following examples show the use of the comma (,) as an event logical or operator:
always @(a, b, c, d, e)
always @(posedge clk, negedge rstn)
always @(a or b, c, d or e)
#### 9.7.5 Implicit event_expression list

The event_expression list of an event control is a common source of bugs in register transfer level
(RTL) simulations. Users tend to forget to add some of the nets or variables read in the timing control
statement. This is often found when comparing RTL and gate-level versions of a design. The implicit
event_expression, @*, is a convenient shorthand that eliminates these problems by adding all nets and
variables that are read by the statement (which can be a statement group) of a procedural_timing_
control_statement to the event_expression.
```ebnf
event_trigger ::= (From A.6.5)
```

-> hierarchical_event_identifier { [ expression ] } ;
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
135
All net and variable identifiers that appear in the statement will be automatically added to the event
expression with these exceptions:
—
Identifiers that only appear in wait or event expressions.
—
Identifiers that only appear as a hierarchical_variable_identifier in the variable_lvalue of the
left-hand side of assignments.
Nets and variables that appear on the right-hand side of assignments, in function and task calls, in case and
conditional expressions, as an index variable on the left-hand side of assignments, or as variables in case
item expressions shall all be included by these rules.
For example:
Example 1
always @(*)  // equivalent to @(a or b or c or d or f)
y = (a & b) | (c & d) | myfunction(f);
Example 2
always @* begin // equivalent to @(a or b or c or d or tmp1 or tmp2)
tmp1 = a & b;
tmp2 = c & d;
y = tmp1 | tmp2;
end
Example 3
always @* begin  // equivalent to @(b)
@(i) kid = b;  // i is not added to @*
end
Example 4
always @* begin  // equivalent to @(a or b or c or d)
x = a ^ b;
@*             // equivalent to @(c or d)
x = c ^ d;
end
Example 5
always @* begin  // same as @(a or en)
y = 8'hff;
y[a] = !en;
end
Example 6
always @* begin  // same as @(state or go or ws)
next = 4'b0;
case (1'b1)
state[IDLE]: if (go)
next[READ] = 1'b1;
  else
next[IDLE] = 1'b1;
state[READ]:
next[DLY ] = 1'b1;
state[DLY ]: if (!ws) next[DONE] = 1'b1;
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
136
Copyright © 2006 IEEE. All rights reserved.
  else
next[READ] = 1'b1;
state[DONE]:
next[IDLE] = 1'b1;
endcase
end
#### 9.7.6 Level-sensitive event control

The execution of a procedural statement can also be delayed until a condition becomes true. This is
accomplished using the wait statement, which is a special form of event control. The nature of the wait
statement is level-sensitive, as opposed to basic event control (specified by the @ character), which is edge-
sensitive.
The wait statement shall evaluate a condition; and, if it is false, the procedural statements following the wait
statement shall remain blocked until that condition becomes true before continuing. The wait statement has
the form given in Syntax 9-11.
Syntax 9-11—Syntax for wait statement
For example:
The following example shows the use of the wait statement to accomplish level-sensitive event control:
begin
wait (!enable) #10 a = b;
#10 c = d;
end
If the value of enable is 1 when the block is entered, the wait statement will delay the evaluation of the next
statement (#10 a = b;) until the value of enable changes to 0. If enable is already 0 when the begin-
end block is entered, then the assignment “a = b;” is evaluated after a delay of 10 and no additional delay
occurs.
#### 9.7.7 Intra-assignment timing controls

The delay and event control constructs previously described precede a statement and delay its execution. In
contrast, the intra-assignment delay and event controls are contained within an assignment statement and
modify the flow of activity in a different way. This subclause describes the purpose of intra-assignment
timing controls and the repeat timing control that can be used in intra-assignment delays.
An intra-assignment delay or event control shall delay the assignment of the new value to the left-hand side,
but the right-hand expression shall be evaluated before the delay, instead of after the delay. The syntax for
intra-assignment delay and event control is given in Syntax 9-12.
```ebnf
wait_statement ::= (From A.6.5)
```

wait ( expression ) statement_or_null
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
137
The intra-assignment delay and event control can be applied to both blocking assignments and nonblocking
assignments. The repeat event control shall specify an intra-assignment delay of a specified number of
occurrences of an event. If the repeat count literal, or signed reg holding the repeat count, is less than or
equal to 0 at the time of evaluation, the assignment occurs as if there is no repeat construct.
For example:
repeat (-3) @ (event_expression)
// will not execute event_expression.
repeat (a) @ (event_expression)
// if a is assigned -3, it will execute the event_expression
// if a is declared as an unsigned reg, but not if a is signed
This construct is convenient when events have to be synchronized with counts of clock signals.
For example:
Table 9-2 illustrates the philosophy of intra-assignment timing controls by showing the code that could
accomplish the same timing effect without using intra-assignment.
```ebnf
blocking_assignment ::= (From A.6.2)
```

variable_lvalue = [ delay_or_event_control ] expression
```ebnf
nonblocking_assignment ::=
```

variable_lvalue <= [ delay_or_event_control ] expression
```ebnf
delay_control ::= (From A.6.5)
```

# delay_value
| # ( mintypmax_expression )
```ebnf
delay_or_event_control ::=
```

delay_control
| event_control
| repeat ( expression ) event_control
```ebnf
event_control ::=
```

@ hierarchical_event_identifier
| @ ( event_expression )
| @*
| @ (*)
```ebnf
event_expression ::=
```

expression
| posedge expression
| negedge expression
| event_expression or event_expression
| event_expression , event_expression
Syntax 9-12—Syntax for intra-assignment delay and event control
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
138
Copyright © 2006 IEEE. All rights reserved.
The next three examples use the fork-join behavioral construct. All statements between the keywords fork
and join execute concurrently. This construct is described in more detail in 9.8.2.
The following example shows a race condition that could be prevented by using intra-assignment timing
control:
fork
#5 a = b;
#5 b = a;
join
The code in this example samples and sets the values of both a and b at the same simulation time, thereby
creating a race condition. The intra-assignment form of timing control used in the next example prevents this
race condition.
fork
// data swap
a = #5 b;
b = #5 a;
join
Intra-assignment timing control works because the intra-assignment delay causes the values of a and b to be
evaluated before the delay and causes the assignments to be made after the delay. Some existing tools that
implement intra-assignment timing control use temporary storage in evaluating each expression on the right-
hand side.
Intra-assignment waiting for events is also effective. In the following example, the right-hand expressions
are evaluated when the assignment statements are encountered, but the assignments are delayed until the
rising edge of the clock signal:
fork
// data shift
a = @(posedge clk) b;
b = @(posedge clk) c;
join
Table 9-2—Intra-assignment timing control equivalence
Intra-assignment timing control
With intra-assignment construct
Without intra-assignment construct
a = #5 b;
begin
temp = b;
#5 a = temp;
end
a = @(posedge clk) b;
begin
temp = b;
@(posedge clk) a = temp;
end
a = repeat(3)
@(posedge clk) b;
begin
temp = b;
@(posedge clk);
@(posedge clk);
@(posedge clk) a = temp;
end
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
139
The following is an example of a repeat event control as the intra-assignment delay of a nonblocking
assignment:
a <= repeat(5) @(posedge clk) data;
Figure 9-1 illustrates the activities that result from this repeat event control.
In this example, the value of data is evaluated when the assignment is encountered. After five occurrences
of posedge clk, a is assigned the value of data.
The following is an example of a repeat event control as the intra-assignment delay of a procedural
assignment:
a = repeat(num) @(clk) data;
In this example, the value of data is evaluated when the assignment is encountered. After the number of
transitions of clk equals the value of num, a is assigned the value of data.
The following is an example of a repeat event control with expressions containing operations to specify both
the number of event occurrences and the event that is counted:
a <= repeat(a+b) @(posedge phi1 or negedge phi2) data;
In this example, the value of data is evaluated when the assignment is encountered. After the sum of the
positive edges of phi1 and the negative edges of phi2 equals the sum of a and b, a is assigned the value of
data. Even if posedge phi1 and negedge phi2 occurred at the same simulation time, each will be
detected separately.
### 9.8 Block statements

The block statements are a means of grouping statements together so that they act syntactically like a single
statement. There are two types of blocks in the Verilog HDL:
—
Sequential block, also called begin-end block
—
Parallel block, also called fork-join block
The sequential block shall be delimited by the keywords begin and end. The procedural statements in
sequential block shall be executed sequentially in the given order.
clk
data
a
data is evaluated
Figure 9-1—Repeat event control utilizing a clock edge
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
140
Copyright © 2006 IEEE. All rights reserved.
The parallel block shall be delimited by the keywords fork and join. The procedural statements in parallel
block shall be executed concurrently.
#### 9.8.1 Sequential blocks

A sequential block shall have the following characteristics:
—
Statements shall be executed in sequence, one after another.
—
Delay values for each statement shall be treated relative to the simulation time of the execution of
the previous statement.
—
Control shall pass out of the block after the last statement executes.
Syntax 9-13 gives the formal syntax for a sequential block.
Syntax 9-13—Syntax for sequential block
For example:
Example 1—A sequential block enables the following two assignments to have a deterministic result:
begin
areg = breg;
creg = areg;
// creg stores the value of breg
end
The first assignment is performed, and areg is updated before control passes to the second assignment.
Example 2—Delay control can be used in a sequential block to separate the two assignments in time.
begin
areg = breg;
@(posedge clock) creg = areg;
// assignment delayed until
end
// posedge on clock
Example 3—The following example shows how the combination of the sequential block and delay control
can be used to specify a time-sequenced waveform:
parameter d = 50;
// d declared as a parameter and
reg [7:0] r;
// r declared as an 8-bit reg
begin
 // a waveform controlled by sequential delay
```ebnf
seq_block ::= (From A.6.3)
```

begin [ : block_identifier
{ block_item_declaration } ] { statement } end
```ebnf
block_item_declaration ::= (From A.2.8)
```

{ attribute_instance } reg [ signed ] [ range ]  list_of_block_variable_identifiers ;
| { attribute_instance } integer list_of_block_variable_identifiers ;
| { attribute_instance } time list_of_block_variable_identifiers ;
| { attribute_instance } real list_of_block_real_identifiers ;
| { attribute_instance } realtime list_of_block_real_identifiers ;
| { attribute_instance } event_declaration
| { attribute_instance } local_parameter_declaration ;
| { attribute_instance } parameter_declaration ;
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
141
#d r = 'h35;
#d r = 'hE2;
#d r = 'h00;
#d r = 'hF7;
#d -> end_wave; //trigger an event called end_wave
end
#### 9.8.2 Parallel blocks

A parallel block shall have the following characteristics:
—
Statements shall execute concurrently.
—
Delay values for each statement shall be considered relative to the simulation time of entering the
block.
—
Delay control can be used to provide time-ordering for assignments.
—
Control shall pass out of the block when the last time-ordered statement executes.
Syntax 9-14 gives the formal syntax for a parallel block.
Syntax 9-14—Syntax for parallel block
The timing controls in a fork-join block do not have to be ordered sequentially in time.
For example:
The following example codes the waveform description shown in Example 3 of 9.8.1 by using a parallel
block instead of a sequential block. The waveform produced on the reg is exactly the same for both
implementations.
fork
#50 r = 'h35;
#100 r = 'hE2;
#150 r = 'h00;
#200 r = 'hF7;
#250 -> end_wave;
join
#### 9.8.3 Block names

Both sequential and parallel blocks can be named by adding : name_of_block after the keywords begin
or fork. The naming of blocks serves several purposes:
```ebnf
par_block ::= (From A.6.3)
```

fork [ : block_identifier
{ block_item_declaration } ] { statement } join
```ebnf
block_item_declaration ::= (From A.2.8)
```

{ attribute_instance } reg [ signed ] [ range ]  list_of_block_variable_identifiers ;
| { attribute_instance } integer list_of_block_variable_identifiers ;
| { attribute_instance } time list_of_block_variable_identifiers ;
| { attribute_instance } real list_of_block_real_identifiers ;
| { attribute_instance } realtime list_of_block_real_identifiers ;
| { attribute_instance } event_declaration
| { attribute_instance } local_parameter_declaration ;
| { attribute_instance } parameter_declaration ;
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
142
Copyright © 2006 IEEE. All rights reserved.
—
It allows local variables, parameters, and named events to be declared for the block.
—
It allows the block to be referenced in statements such as the disable statement (see 10.3).
All variables shall be static; that is, a unique location exists for all variables, and leaving or entering blocks
shall not affect the values stored in them.
The block names give a means of uniquely identifying all variables at any simulation time.
#### 9.8.4 Start and finish times

Both sequential and parallel blocks have the notion of a start and finish time. For sequential blocks, the start
time is when the first statement is executed, and the finish time is when the last statement has been executed.
For parallel blocks, the start time is the same for all the statements, and the finish time is when the last time-
ordered statement has been executed.
Sequential and parallel blocks can be embedded within each other, allowing complex control structures to be
expressed easily and with a high degree of structure. When blocks are embedded within each other, the
timing of when a block starts and finishes is important. Execution shall not continue to the statement
following a block until the finish time for the block has been reached, that is, until the block has completely
finished executing.
For example:
Example 1—The following example shows the statements from the example in 9.8.2 written in the reverse
order and still producing the same waveform.
fork
#250 -> end_wave;
#200 r = 'hF7;
#150 r = 'h00;
#100 r = 'hE2;
#50 r = 'h35;
join
Example 2—When an assignment is to be made after two separate events have occurred, known as the
joining of events, a fork-join block can be useful.
begin
fork
@Aevent;
@Bevent;
join
areg = breg;
end
The two events can occur in any order (or even at the same simulation time), the fork-join block will
complete, and the assignment will be made. In contrast, if the fork-join block was a begin-end block
and the Bevent occurred before the Aevent, then the block would be waiting for the next Bevent.
Example 3—This example shows two sequential blocks, each of which will execute when its controlling
event occurs. Because the event controls are within a fork-join block, they execute in parallel, and the
sequential blocks can, therefore, also execute in parallel.
fork
@enable_a
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
143
begin
#ta wa = 0;
#ta wa = 1;
#ta wa = 0;
end
@enable_b
begin
#tb wb = 1;
#tb wb = 0;
#tb wb = 1;
end
join
### 9.9 Structured procedures

All procedures in the Verilog HDL are specified within one of the following four statements:
—
initial construct
—
always construct
—
Task
—
Function
The initial and always constructs are enabled at the beginning of a simulation. The initial construct shall
execute only once, and its activity shall cease when the statement has finished. In contrast, the always
construct shall execute repeatedly. Its activity shall cease only when the simulation is terminated. There
shall be no implied order of execution between initial and always constructs. The initial constructs need not
be scheduled and executed before the always constructs. There shall be no limit to the number of initial and
always constructs that can be defined in a module.
Tasks and functions are procedures that are enabled from one or more places in other procedures. Tasks and
functions are described in Clause 10.
#### 9.9.1 Initial construct

The syntax for the initial construct is given in Syntax 9-15.
Syntax 9-15—Syntax for initial construct
For example:
The following example illustrates use of the initial construct for initialization of variables at the start of
simulation.
initial begin
areg = 0; // initialize a reg
for (index = 0; index < size; index = index + 1)
 memory[index] = 0; //initialize memory word
end
```ebnf
initial_construct ::= (From A.6.2)
```

initial statement
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
144
Copyright © 2006 IEEE. All rights reserved.
Another typical usage of the initial construct is specification of waveform descriptions that execute once to
provide stimulus to the main part of the circuit being simulated.
initial begin
inputs = 'b000000;     // initialize at time zero
#10 inputs = 'b011001; // first pattern
#10 inputs = 'b011011; // second pattern
#10 inputs = 'b011000; // third pattern
#10 inputs = 'b001000; // last pattern
end
#### 9.9.2 Always construct

The always construct repeats continuously throughout the duration of the simulation. Syntax 9-16 shows the
syntax for the always construct.
Syntax 9-16—Syntax for always construct
The always construct, because of its looping nature, is only useful when used in conjunction with some form
of timing control. If an always construct has no control for simulation time to advance, it will create a
simulation deadlock condition.
The following code, for example, creates a zero-delay infinite loop:
always areg = ~areg;
Providing a timing control to the above code creates a potentially useful description as shown in the
following:
always #half_period areg = ~areg;
```ebnf
always_construct ::= (From A.6.2)
```

always statement
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
