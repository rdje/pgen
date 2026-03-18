---
title: "Section 5: Expressions"
document: "Verilog Hardware Description Language Reference Manual"
standard: "IEEE 1364-2005"
domain: "Verilog"
section: "5"
source_txt: "section-5-expressions.txt"
source_pdf: "/Users/richarddje/Documents/github/Verilog-LRM-IEEE-1364-2005.pdf"
---

# Section 5: Expressions

IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
41
## 5. Expressions

This clause describes the operators and operands available in the Verilog HDL and how to use them to form
expressions.
An expression is a construct that combines operands with operators to produce a result that is a function of
the values of the operands and the semantic meaning of the operator. Any legal operand, such as a net bit-
select, without any operator is considered an expression. Wherever a value is needed in a Verilog HDL
statement, an expression can be used.
Some statement constructs require an expression to be a constant expression. The operands of a constant
expression consist of constant numbers, strings, parameters, constant bit-selects and part-selects of
parameters, constant function calls (see 10.4.5), and constant system function calls only; but they can use
any of the operators defined in Table 5-1.
Constant system function calls are calls to certain built-in system functions where the arguments are constant
expressions. When used in constant expressions, these function calls shall be evaluated at elaboration time.
The system functions that may be used in constant system function calls are pure functions, i.e., those whose
value depends only on their input arguments and which have no side effects. Specifically, the system
functions allowed in constant expressions are the conversion system functions listed in 17.8 and the
mathematical system functions listed in 17.11.
The data types reg, integer, time, real, and realtime are all variable data types. Descriptions pertaining to
variable usage apply to all of these data types.
An operand can be one of the following:
—
Constant number (including real) or string
—
Parameter (including local and specify parameters)
—
Parameter (not real) bit-select or part-select (including local and specify parameters)
—
Net
—
Net bit-select or part-select
—
reg, integer, or time variable
—
reg, integer, or time variable bit-select or part-select
—
real or realtime variable
—
Array element
—
Array element (not real) bit-select or part-select
—
A call to a user-defined function or system-defined function that returns any of the above
### 5.1 Operators

The symbols for the Verilog HDL operators are similar to those in the C programming language. Table 5-1
lists these operators.
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
42
Copyright © 2006 IEEE. All rights reserved.
#### 5.1.1 Operators with real operands

The operators shown in Table 5-2 shall be legal when applied to real operands. All other operators shall be
considered illegal when used with real operands.
The result of using logical or relational operators on real numbers is a single-bit scalar value.
Table 5-1—Operators in Verilog HDL
{}  {{}}
Concatenation, replication
unary + unary -
Unary operators
+ - * / **
Arithmetic
%
Modulus
> >= < <=
Relational
!
Logical negation
&&
Logical and
||
Logical or
==
Logical equality
!=
Logical inequality
===
Case equality
!==
Case inequality
~
Bitwise negation
&
Bitwise and
|
Bitwise inclusive or
^
Bitwise exclusive or
^~ or ~^
Bitwise equivalence
&
Reduction and
~&
Reduction nand
|
Reduction or
~|
Reduction nor
^
Reduction xor
~^ or ^~
Reduction xnor
<<
Logical left shift
>>
Logical right shift
<<<
Arithmetic left shift
>>>
Arithmetic right shift
? :
Conditional
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
43
Table 5-3 lists operators that shall not be used to operate on real numbers.
See 4.8.1 for more information on use of real numbers.
#### 5.1.2 Operator precedence

The precedence order of the Verilog operators is shown in Table 5-4.
Operators shown on the same row in Table 5-4 shall have the same precedence. Rows are arranged in order
of decreasing precedence for the operators. For example, *, /, and % all have the same precedence, which is
higher than that of the binary + and – operators.
All operators shall associate left to right with the exception of the conditional operator, which shall associate
right to left. Associativity refers to the order in which the operators having the same precedence are
evaluated. Thus, in the following example, B is added to A, and then C is subtracted from the result of A+B.
A + B - C
When operators differ in precedence, the operators with higher precedence shall associate first. In the
following example, B is divided by C (division has higher precedence than addition), and then the result is
added to A.
A + B / C
Parentheses can be used to change the operator precedence.
(A + B) / C
// not the same as A + B / C
Table 5-2—Legal operators for use in real expressions
unary +   unary -
Unary operators
+   -    *   / **
Arithmetic
>   >=   <   <=
Relational
!    &&  ||
Logical
==        !=
Logical equality
?:
Conditional
Table 5-3—Operators not allowed for real expressions
{}   {{}}
Concatenate, replicate
%
Modulus
===   !==
Case equality
~   &   |
^   ^~   ~^
Bitwise
^   ^~   ~^
&   ~&   |   ~|
Reduction
<<   >>   <<<   >>>
Shift
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
44
Copyright © 2006 IEEE. All rights reserved.
#### 5.1.3 Using integer numbers in expressions

Integer numbers can be used as operands in expressions. An integer number can be expressed as
—
An unsized, unbased integer (e.g., 12)
—
An unsized, based integer (e.g., 'd12, 'sd12)
—
A sized, based integer (e.g., 16'd12, 16'sd12)
A negative value for an integer with no base specifier shall be interpreted differently from an integer with a
base specifier. An integer with no base specifier shall be interpreted as a signed value in twos-complement
form. An integer with an unsigned base specifier shall be interpreted as an unsigned value.
For example:
This example shows four ways to write the expression “minus 12 divided by 3.” Note that -12 and -'d12
both evaluate to the same twos-complement bit pattern, but, in an expression, the -'d12 loses its identity as
a signed negative number.
integer IntA;
IntA = -12 / 3;
// The result is -4.
IntA = -'d 12 / 3;
// The result is 1431655761.
IntA = -'sd 12 / 3;
// The result is -4.
IntA = -4'sd 12 / 3;
// -4'sd12 is the negative of the 4-bit
// quantity 1100, which is -4. -(-4) = 4.
// The result is 1.
Table 5-4—Precedence rules for operators
+ - ! ~ & ~& | ~| ^ ~^ ^~ (unary)
Highest precedence
**
* / %
+ - (binary)
 << >> <<< >>>
 < <= > >=
== != === !==
& (binary)
^ ^~ ~^ (binary)
| (binary)
&&
||
?: (conditional operator)
{}  {{}}
Lowest precedence
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
45
#### 5.1.4 Expression evaluation order

The operators shall follow the associativity rules while evaluating an expression as described in 5.1.2.
However, if the final result of an expression can be determined early, the entire expression need not be
evaluated. This is called short-circuiting an expression evaluation.
For example:
reg regA, regB, regC, result ;
result = regA & (regB | regC) ;
If regA is known to be zero, the result of the expression can be determined as zero without evaluating the
subexpression regB | regC.
#### 5.1.5 Arithmetic operators

The binary arithmetic operators are given in Table 5-5.
The integer division shall truncate any fractional part toward zero. For the division or modulus operators, if
the second operand is a zero, then the entire result value shall be x. The modulus operator (for example,
y % z) gives the remainder when the first operand is divided by the second and thus is zero when z divides
y exactly. The result of a modulus operation shall take the sign of the first operand.
If either operand of the power operator is real, then the result type shall be real. The result of the power
operator is unspecified if the first operand is zero and the second operand is nonpositive or if the first
operand is negative and the second operand is not an integral value.
If neither operand of the power operator is real, then the result type shall be determined as outlined in 5.4.1
and 5.5.1. The result value is 'bx if the first operand is zero and the second operand is negative. The result
value is 1 if the second operand is zero.
In all cases, the second operand of the power operator shall be treated as self-determined.
These statements are illustrated in Table 5-6.
Table 5-5—Arithmetic operators defined
a + b
a plus b
a - b
a minus b
a * b
a multiplied by b (or a times b)
a / b
a divided by b
a % b
a modulo b
a ** b
a to the power of b
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
46
Copyright © 2006 IEEE. All rights reserved.
The unary arithmetic operators shall take precedence over the binary operators. The unary operators are
given in Table 5-7.
For the arithmetic operators, if any operand bit value is the unknown value x or the high-impedance value z,
then the entire result value shall be x.
For example:
Table 5-8 gives examples of some modulus and power operations.
Table 5-6—Power operator rules
op1 is
op2 is
negative < –1
–1
zero
1
positive > 1
positive
op1 ** op2
op2 is odd -> –1
op2 is even -> 1
0
1
op1 ** op2
zero
1
1
1
1
1
negative
0
op2 is odd -> –1
op2 is even -> 1
'bx
1
0
Table 5-7—Unary operators defined
+m
Unary plus m (same as m)
-m
Unary minus m
Table 5-8—Examples of modulus and power operators
Expression
Result
Comments
## 10 % 3

1
10/3 yields a remainder of 1.
## 11 % 3

2
11/3 yields a remainder of 2.
## 12 % 3

0
12/3 yields no remainder.
–10 % 3
–1
The result takes the sign of the first operand.
## 11 % –3

2
The result takes the sign of the first operand
–4'd12 % 3
1
–4'd12 is seen as a large positive number that leaves a remainder of 1 when divided by 3.
## 3 ** 2

9
## 3 * 3

## 2 ** 3

8
## 2 * 2 * 2

## 2 ** 0

1
Anything to the zero exponent is 1.
## 0 ** 0

1
Zero to the zero exponent is also 1.
### 2.0 ** –3'sb1

0.5
### 2.0 is real, giving real reciprocal.

## 2 ** –3 'sb1

0
## 2 ** –1 = 1/2. Integer division truncates to zero.

Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
47
#### 5.1.6 Arithmetic expressions with regs and integers

A value assigned to a reg variable or a net shall be treated as an unsigned value unless the reg variable or net
has been explicitly declared to be signed. A value assigned to an integer, real or realtime variable shall be
treated as signed. A value assigned to a time variable shall be treated as unsigned. Signed values, except for
those assigned to real and realtime variables, shall use a twos-complement representation. Values assigned
to real and realtime variables shall use a floating-point representation. Conversions between signed and
unsigned values shall keep the same bit representation; only the interpretation changes.
Table 5-9 lists how arithmetic operators interpret each data type.
For example:
The following example shows various ways to divide “minus twelve by three”—using integer and reg data
types in expressions.
integer intA;
reg [15:0] regA;
reg signed [15:0] regS;
intA = -4'd12;
regA = intA / 3;
// expression result is -4,
// intA is an integer data type, regA is 65532
regA = -4'd12;
// regA is 65524
intA = regA / 3;
// expression result is 21841,
## 0 ** –1

'bx
## 0 ** –1 = 1/0. Integer division by zero is 'bx.

## 9 ** 0.5

3.0
Real square root.
### 9.0 ** (1/2)

1.0
Integer division truncates exponent to zero.
–3.0 ** 2.0
9.0
Defined because real 2.0 is still integral value.
Table 5-9—Data type interpretation by arithmetic operators
Data type
Interpretation
unsigned net
Unsigned
signed net
Signed, twos complement
unsigned reg
Unsigned
signed reg
Signed, twos complement
integer
Signed, twos complement
time
Unsigned
real, realtime
Signed, floating point
Table 5-8—Examples of modulus and power operators  (continued)
Expression
Result
Comments
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
48
Copyright © 2006 IEEE. All rights reserved.
// regA is a reg data type
intA = -4'd12 / 3;
// expression result is 1431655761.
// -4'd12 is effectively a 32-bit reg data type
regA = -12 / 3;
// expression result is -4, -12 is effectively
// an integer data type. regA is 65532
regS = -12 / 3;
// expression result is -4. regS is a signed reg
regS = -4'sd12 / 3; // expression result is 1. -4'sd12 is actually 4.
// The rules for integer division yield 4/3==1.
#### 5.1.7 Relational operators

Table 5-10 lists and defines the relational operators.
An expression using these relational operators shall yield the scalar value 0 if the specified relation is false
or the value 1 if it is true. If either operand of a relational operator contains an unknown (x) or high-
impedance (z) value, then the result shall be a 1-bit unknown value (x).
When one or both operands of a relational expression are unsigned, the expression shall be interpreted as a
comparison between unsigned values. If the operands are of unequal bit lengths, the smaller operand shall be
zero-extended to the size of the larger operand.
When both operands are signed, the expression shall be interpreted as a comparison between signed values.
If the operands are of unequal bit lengths, the smaller operand shall be sign-extended to the size of the larger
operand.
If either operand is a real operand, then the other operand shall be converted to an equivalent real value and
the expression shall be interpreted as a comparison between real values.
All the relational operators shall have the same precedence. Relational operators shall have lower
precedence than arithmetic operators.
For example:
The following examples illustrate the implications of this precedence rule:
a < foo - 1
// this expression is the same as
a < (foo - 1)
// this expression, but . . .
foo - (1 < a)
// this one is not the same as
foo - 1 < a
// this expression
Table 5-10—Definitions of relational operators
a < b
a less than b
a > b
a greater than b
a <= b
a less than or equal to b
a >= b
a greater than or equal to b
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
49
When foo - (1 < a) evaluates, the relational expression evaluates first, and then either zero or one is
subtracted from foo. When foo - 1 < a evaluates, the value of foo operand is reduced by one and then
compared with a.
#### 5.1.8 Equality operators

The equality operators shall rank lower in precedence than the relational operators. Table 5-11 lists and
defines the equality operators.
All four equality operators shall have the same precedence. These four operators compare operands bit for
bit. As with the relational operators, the result shall be 0 if comparison fails and 1 if it succeeds.
If the operands are of unequal bit lengths and if one or both operands are unsigned, the smaller operand shall
be zero-extended to the size of the larger operand. If both operands are signed, the smaller operand shall be
sign-extended to the size of the larger operand.
If either operand is a real operand, then the other operand shall be converted to an equivalent real value, and
the expression shall be interpreted as a comparison between real values.
For the logical equality and logical inequality operators (== and !=), if, due to unknown or high-impedance
bits in the operands, the relation is ambiguous, then the result shall be a 1-bit unknown value (x).
For the case equality and case inequality operators (=== and !==), the comparison shall be done just as it is
in the procedural case statement (see 9.5). Bits that are x or z shall be included in the comparison and shall
match for the result to be considered equal. The result of these operators shall always be a known value,
either 1 or 0.
#### 5.1.9 Logical operators

The operators logical and (&&) and logical or (||) are logical connectives. The result of the evaluation of a
logical comparison shall be 1 (defined as true), 0 (defined as false), or, if the result is ambiguous, the
unknown value (x). The precedence of && is greater than that of ||, and both are lower than relational and
equality operators.
A third logical operator is the unary logical negation operator (!). The negation operator converts a
nonzero or true operand into 0 and a zero or false operand into 1. An ambiguous truth value remains as x.
For example:
Example 1—If reg alpha holds the integer value 237 and beta holds the value zero, then the following
examples perform as described:
regA = alpha && beta;
// regA is set to 0
regB = alpha || beta;
// regB is set to 1
Table 5-11—Definitions of equality operators
a === b
a equal to b, including x and z
a !== b
a not equal to b, including x and z
a == b
a equal to b, result can be unknown
a != b
a not equal to b, result can be unknown
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
50
Copyright © 2006 IEEE. All rights reserved.
Example 2—The following expression performs a logical and of three subexpressions without needing any
parentheses:
a < size-1 && b != c && index != lastone
However, it is recommended for readability purposes that parentheses be used to show very clearly the
precedence intended, as in the following rewrite of this example:
(a < size-1) && (b != c) && (index != lastone)
Example 3—A common use of ! is in constructions like the following:
if (!inword)
In some cases, the preceding construct makes more sense to someone reading the code than this equivalent
construct:
if (inword == 0)
#### 5.1.10 Bitwise operators

The bitwise operators shall perform bitwise manipulations on the operands; that is, the operator shall
combine a bit in one operand with its corresponding bit in the other operand to calculate 1 bit for the result.
Logic Table 5-12 through Table 5-16 show the results for each possible calculation.
Table 5-12—Bitwise binary and operator
&
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
x
x
0
x
x
x
z
0
x
x
x
Table 5-13—Bitwise binary or operator
|
0
1
x
z
0
0
1
x
x
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
x
1
x
x
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
51
When the operands are of unequal bit length, the shorter operand is zero-filled in the most significant bit
positions.
#### 5.1.11 Reduction operators

The unary reduction operators shall perform a bitwise operation on a single operand to produce a single-bit
result. For reduction and, reduction or, and reduction xor operators, the first step of the operation shall apply
the operator between the first bit of the operand and the second using logic Table 5-17 through Table 5-19.
The second and subsequent steps shall apply the operator between the 1-bit result of the prior step and the
next bit of the operand using the same logic table. For reduction nand, reduction nor, and reduction xnor
operators, the result shall be computed by inverting the result of the reduction and, reduction or, and
reduction xor operation, respectively.
Table 5-14—Bitwise binary exclusive or operator
^
0
1
x
z
0
0
1
x
x
1
1
0
x
x
x
x
x
x
x
z
x
x
x
x
Table 5-15—Bitwise binary exclusive nor operator
^~
~^
0
1
x
z
0
1
0
x
x
1
0
1
x
x
x
x
x
x
x
z
x
x
x
x
Table 5-16—Bitwise unary negation operator
~
0
1
1
0
x
x
z
x
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
52
Copyright © 2006 IEEE. All rights reserved.
For example:
Table 5-20 shows the results of applying reduction operators on different operands.
Table 5-17—Reduction unary and operator
&
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
x
x
0
x
x
x
z
0
x
x
x
Table 5-18—Reduction unary or operator
|
0
1
x
z
0
0
1
x
x
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
x
1
x
x
Table 5-19—Reduction unary exclusive or operator
^
0
1
x
z
0
0
1
x
x
1
1
0
x
x
x
x
x
x
x
z
x
x
x
x
Table 5-20—Results of unary reduction operations
Operand
&
~&
|
~|
^
~^
Comments
4'b0000
0
1
0
1
0
1
No bits set
4'b1111
1
0
1
0
0
1
All bits set
4'b0110
0
1
1
0
0
1
Even number of bits set
4'b1000
0
1
1
0
1
0
Odd number of bits set
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
53
#### 5.1.12 Shift operators

There are two types of shift operators: the logical shift operators, << and >>, and the arithmetic shift
operators, <<< and >>>. The left shift operators, << and <<<, shall shift their left operand to the left by the
number by the number of bit positions given by the right operand. In both cases, the vacated bit positions
shall be filled with zeroes. The right shift operators, >> and >>>, shall shift their left operand to the right by
the number of bit positions given by the right operand. The logical right shift shall fill the vacated bit
positions with zeroes. The arithmetic right shift shall fill the vacated bit positions with zeroes if the result
type is unsigned. It shall fill the vacated bit positions with the value of the most significant (i.e., sign) bit of
the left operand if the result type is signed. If the right operand has an x or z value, then the result shall be
unknown. The right operand is always treated as an unsigned number and has no effect on the signedness of
the result. The result signedness is determined by the left-hand operand and the remainder of the expression,
as outlined in 5.5.1.
For example:
Example 1—In this example, the reg result is assigned the binary value 0100, which is 0001 shifted to the
left two positions and zero-filled.
module shift;
reg [3:0] start, result;
initial begin
start = 1;
result = (start << 2);
end
endmodule
Example 2—In this example, the reg result is assigned the binary value 1110, which is 1000 shifted to the
right two positions and sign-filled.
module ashift;
reg signed [3:0] start, result;
initial begin
start = 4'b1000;
result = (start >>> 2);
end
endmodule
#### 5.1.13 Conditional operator

The conditional operator, also known as ternary operator, shall be right associative and shall be constructed
using three operands separated by two operators in the format given in Syntax 5-1.
Syntax 5-1—Syntax for conditional operator
```ebnf
conditional_expression ::= (From A.8.3)
```

expression1 ? { attribute_instance } expression2 : expression3
```ebnf
expression1 ::=
```

expression
```ebnf
expression2 ::=
```

expression
```ebnf
expression3 ::=
```

expression
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
54
Copyright © 2006 IEEE. All rights reserved.
The evaluation of a conditional operator shall begin with a logical equality comparison (see 5.1.8) of
expression1 with zero, termed the condition. If the condition evaluates to false (0), then expression3 shall be
evaluated and used as the result of the conditional expression. If the condition evaluates to true (1), then
expression2 is evaluated and used as the result. If the condition evaluates to an ambiguous value (x or z),
then both expression2 and expression3 shall be evaluated; and their results shall be combined, bit by bit,
using Table 5-21 to calculate the final result unless expression2 or expression3 is real, in which case the
result shall be 0. If the lengths of expression2 and expression3 are different, the shorter operand shall be
lengthened to match the longer and zero-filled from the left (the high-order end).
For example:
The following example of a three-state output bus illustrates a common use of the conditional operator:
wire [15:0] busa = drive_busa ? data : 16'bz;
The bus called data is driven onto busa when drive_busa is 1. If drive_busa is unknown, then an
unknown value is driven onto busa. Otherwise, busa is not driven.
#### 5.1.14 Concatenations

A concatenation is the result of the joining together of bits resulting from one or more expressions. The
concatenation shall be expressed using the brace characters { and }, with commas separating the expressions
within.
Unsized constant numbers shall not be allowed in concatenations. This is because the size of each operand in
the concatenation is needed to calculate the complete size of the concatenation.
For example:
This example concatenates four expressions:
{a, b[3:0], w, 3'b101}
It is equivalent to the following example:
{a, b[3], b[2], b[1], b[0], w, 1'b1, 1'b0, 1'b1}
An operator that can be applied only to concatenations is replication, which is expressed by a concatenation
preceded by a non-negative, non-x and non-z constant expression, called a replication constant, enclosed
together within brace characters, and which indicates a joining together of that many copies of the
Table 5-21—Ambiguous condition results for conditional operator
?:
0
1
x
z
0
0
x
x
x
1
x
1
x
x
x
x
x
x
x
z
x
x
x
x
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
55
concatenation. Unlike regular concatenations, expressions containing replications shall not appear on the
left-hand side of an assignment and shall not be connected to output or inout ports.
This example replicates w four times.
{4{w}} // This yields the same value as {w, w, w, w}
The following examples show illegal replications:
{1'bz{1'b0}}    // illegal
{1'bx{1'b0}}    // illegal
The next example illustrates a replication nested within a concatenation:
{b, {3{a, b}}}
// This yields the same value as
// {b, a, b, a, b, a, b}
A replication operation may have a replication constant with a value of zero. This is useful in parameterized
code. A replication with a zero replication constant is considered to have a size of zero and is ignored. Such
a replication shall appear only within a concatenation in which at least one of the operands of the
concatenation has a positive size.
For example:
parameter P = 32;
// The following is legal for all P from 1 to 32
assign b[31:0] = { {32-P{1’b1}}, a[P-1:0] } ;
// The following is illegal for P=32 because the zero
// replication appears alone within a concatenation
assign c[31:0] = { {{32-P{1’b1}}}, a[P-1:0] }
// The following is illegal for P=32
initial
  $displayb({32-P{1’b1}}, a[P-1:0]);
When a replication expression is evaluated, the operands shall be evaluated exactly once, even if the
replication constant is zero. For example:
result = {4{func(w)}} ;
would be computed as
y = func(w) ;
result = {y, y, y, y} ;
### 5.2 Operands

There are several types of operands that can be specified in expressions. The simplest type is a reference to a
net, variable, or parameter in its complete form; that is, just the name of the net, variable, or parameter is
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
56
Copyright © 2006 IEEE. All rights reserved.
given. In this case, all of the bits making up the net, variable, or parameter value shall be used as the
operand.
If a single bit of a vector net, vector reg, integer, or time variable, or parameter is required, then a bit-select
operand shall be used. A part-select operand shall be used to reference a group of adjacent bits in a vector
net, vector reg, integer, or time variable, or parameter.
An array element or a bit-select or part-select of an array element can be referenced as an operand.
A concatenation of other operands (including nested concatenations) can be specified as an operand. A
function call is an operand.
#### 5.2.1 Vector bit-select and part-select addressing

Bit-selects extract a particular bit from a vector net, vector reg, integer, or time variable, or parameter. The
bit can be addressed using an expression. If the bit-select is out of the address bounds or the bit-select is x or
z, then the value returned by the reference shall be x. A bit-select or part-select of a scalar, or of a variable or
parameter of type real or realtime, shall be illegal.
Several contiguous bits in a vector net, vector reg, integer, or time variable, or parameter can be addressed
and are known as part-selects. There are two types of part-selects, a constant part-select and an indexed part-
select. A constant part-select of a vector reg or net is given with the following syntax:
vect[msb_expr:lsb_expr]
Both msb_expr and lsb_expr shall be constant integer expressions. The first expression has to address a
more significant bit than the second expression.
An indexed part-select of a vector net, vector reg, integer, or time variable, or parameter is given with the
following syntax:
reg [15:0] big_vect;
reg [0:15] little_vect;
   big_vect[lsb_base_expr +: width_expr]
little_vect[msb_base_expr +: width_expr]
   big_vect[msb_base_expr -: width_expr]
little_vect[lsb_base_expr -: width_expr]
The msb_base_expr and lsb_base_expr shall be integer expressions, and the width_expr shall be a
positive constant integer expression. The lsb_base_expr and msb_base_expr can vary at run time. The
first two examples select bits starting at the base and ascending the bit range. The number of bits selected is
equal to the width expression. The second two examples select bits starting at the base and descending the
bit range.
A part-select of any type that addresses a range of bits that are completely out of the address bounds of the
net, reg, integer, time variable, or parameter or a part-select that is x or z shall yield the value x when read
and shall have no effect on the data stored when written. Part-selects that are partially out of range shall,
when read, return x for the bits that are out of range and shall, when written, only affect the bits that are in
range.
For example:
reg [31: 0] big_vect;
reg [0 :31] little_vect;
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
57
reg [63: 0] dword;
integer sel;
big_vect[ 0 +: 8]    // == big_vect[ 7 : 0]
big_vect[15 -: 8]    // == big_vect[15 : 8]
little_vect[ 0 +: 8] // == little_vect[0 : 7]
little_vect[15 -: 8] // == little_vect[8 :15]
dword[8*sel +: 8]    // variable part-select with fixed width
For example:
Example 1—The following example specifies the single bit of acc vector that is addressed by the operand
index:
acc[index]
The actual bit that is accessed by an address is, in part, determined by the declaration of acc. For instance,
each of the declarations of acc shown in the next example causes a particular value of index to access a
different bit:
reg [15:0] acc;
reg [2:17] acc
Example 2—The next example and the bullet items that follow it illustrate the principles of bit addressing.
The code declares an 8-bit reg called vect and initializes it to a value of 4. The list describes how the
separate bits of that vector can be addressed.
reg [7:0] vect;
vect = 4; // fills vect with the pattern 00000100
// msb is bit 7, lsb is bit 0
—
If the value of addr is 2, then vect[addr] returns 1.
—
If the value of addr is out of bounds, then vect[addr] returns x.
—
If addr is 0, 1, or 3 through 7, vect[addr] returns 0.
—
vect[3:0] returns the bits 0100.
—
vect[5:1] returns the bits 00010.
—
vect[expression that returns x] returns x.
—
vect[expression that returns z] returns x.
—
If any bit of addr is x or z, then the value of addr is x.
NOTE 1—Part-select indices that evaluate to x or z may be flagged as a compile time error.
NOTE 2—Bit-select or part-select indices that are outside of the declared range may be flagged as a compile time error.
#### 5.2.2 Array and memory addressing

Declaration of arrays and memories (one-dimensional arrays of reg) are discussed in 4.9. This subclause
discusses array addressing.
For example:
The next example declares a memory of 1024 eight-bit words:
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
58
Copyright © 2006 IEEE. All rights reserved.
reg [7:0] mem_name[0:1023];
The syntax for a memory address shall consist of the name of the memory and an expression for the address,
specified with the following format:
mem_name[addr_expr]
The addr_expr can be any integer expression; therefore, memory indirections can be specified in a single
expression. The next example illustrates memory indirection:
mem_name[mem_name[3]]
In this example, mem_name[3]addresses word three of the memory called mem_name. The value at word
three is the index into mem_name that is used by the memory address mem_name[mem_name[3]]. As with
bit-selects, the address bounds given in the declaration of the memory determine the effect of the address
expression. If the index is out of the address bounds or if any bit in the address is x or z, then the value of the
reference shall be x.
For example:
The next example declares an array of 256-by-256 eight-bit elements and an array 256-by-256-by-8 one-bit
elements:
reg [7:0] twod_array[0:255][0:255];
wire threed_array[0:255][0:255][0:7];
The syntax for access to the array shall consist of the name of the memory or array and an integer expression
for each addressed dimension:
twod_array[addr_expr][addr_expr]
threed_array[addr_expr][addr_expr][addr_expr]
As before, the addr_expr can be any integer expression. The array twod_array accesses a whole 8-bit
vector, while the array threed_array accesses a single bit of the three-dimensional array.
To express bit-selects or part-selects of array elements, the desired word shall first be selected by supplying
an address for each dimension. Once selected, bit-selects and part-selects shall be addressed in the same
manner as net and reg bit-selects and part-selects (see 5.2.1).
For example:
twod_array[14][1][3:0]      // access lower 4 bits of word
twod_array[1][3][6]         // access bit 6 of word
twod_array[1][3][sel]       // use variable bit-select
threed_array[14][1][3:0]    // Illegal
#### 5.2.3 Strings

String operands shall be treated as constant numbers consisting of a sequence of 8-bit ASCII codes, one per
character. Any Verilog HDL operator can manipulate string operands. The operator shall behave as though
the entire string were a single numeric value.
When a variable is larger than required to hold the value being assigned, the contents after the assignment
shall be padded on the left with zeros. This is consistent with the padding that occurs during assignment of
nonstring values.
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
59
For example:
The following example declares a string variable large enough to hold 14 characters and assigns a value to it.
The example then manipulates the string using the concatenation operator.
module string_test;
reg [8*14:1] stringvar;
initial begin
stringvar = "Hello world";
$display("%s is stored as %h", stringvar, stringvar);
stringvar = {stringvar,"!!!"};
$display("%s is stored as %h", stringvar, stringvar);
end
endmodule
The result of simulating the above description is
   Hello world is stored as 00000048656c6c6f20776f726c64
Hello world!!! is stored as 48656c6c6f20776f726c64212121
##### 5.2.3.1 String operations

The common string operations copy, concatenate, and compare are supported by Verilog HDL operators.
Copy is provided by simple assignment. Concatenation is provided by the concatenation operator.
Comparison is provided by the equality operators.
When manipulating string values in vector regs, the regs should be at least 8*n bits (where n is the number
of ASCII characters) in order to preserve the 8-bit ASCII code.
##### 5.2.3.2 String value padding and potential problems

When strings are assigned to variables, the values stored shall be padded on the left with zeros. Padding can
affect the results of comparison and concatenation operations. The comparison and concatenation operators
shall not distinguish between zeros resulting from padding and the original string characters (\0, ASCII
NUL).
For example:
The following example illustrates the potential problem:
reg [8*10:1] s1, s2;
initial begin
s1 = "Hello";
s2 = " world!";
if ({s1,s2} == "Hello world!")
$display("strings are equal");
end
The comparison in this example fails because during the assignment the string variables are padded as
illustrated in the next example:
s1 = 000000000048656c6c6f
s2 = 00000020776f726c6421
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
60
Copyright © 2006 IEEE. All rights reserved.
The concatenation of s1 and s2 includes the zero padding, resulting in the following value:
000000000048656c6c6f00000020776f726c6421
Because the string “Hello world!” contains no zero padding, the comparison fails, as shown in the following
example:
This comparison yields a result of zero, which is equivalent to false.
##### 5.2.3.3 Null string handling

The null string ("") shall be considered equivalent to the ASCII NUL ("\0"), which has a value zero (0),
which is different from a string "0".
000000000048656c6c6f00000020776f726c6421
48656c6c6f20776f726c6421
"Hello"  " world!"
s1
s2
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
61
### 5.3 Minimum, typical, and maximum delay expressions

Verilog HDL delay expressions can be specified as three expressions separated by colons and enclosed by
parentheses. This is intended to represent minimum, typical, and maximum values—in that order. The
syntax is given in Syntax 5-2.
Syntax 5-2—Syntax for mintypmax expression
Verilog HDL models typically specify three values for delay expressions. The three values allow a design to
be tested with minimum, typical, or maximum delay values.
Values expressed in min:typ:max format can be used in expressions. The min:typ:max format can be used
wherever expressions can appear.
For example:
Example 1—This example shows an expression that defines a single triplet of delay values. The minimum
value is the sum of a+d; the typical value is b+e; the maximum value is c+f, as follows:
```ebnf
constant_expression ::= (From A.8.3)
```

constant_primary
| unary_operator { attribute_instance } constant_primary
| constant_expression binary_operator { attribute_instance } constant_expression
| constant_expression ? { attribute_instance } constant_expression
constant_expression
```ebnf
constant_mintypmax_expression ::=
```

constant_expression
| constant_expression : constant_expression : constant_expression
```ebnf
expression ::=
```

primary
| unary_operator { attribute_instance } primary
| expression binary_operator { attribute_instance } expression
| conditional_expression
```ebnf
mintypmax_expression ::=
```

expression
| expression : expression : expression
```ebnf
constant_primary ::= (From A.8.4)
  number
| parameter_identifier [ [ constant_range_expression ] ]
| specparam_identifier [ [ constant_range_expression ] ]
| constant_concatenation
| constant_multiple_concatenation
| constant_function_call
| constant_system_function_call
| ( constant_mintypmax_expression )
| string
primary ::=
```

number
| hierarchical_identifier [ { [ expression ] } [ range_expression ]  ]
| concatenation
| multiple_concatenation
| function_call
| system_function_call
| ( mintypmax_expression )
| string
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
62
Copyright © 2006 IEEE. All rights reserved.
(a:b:c) + (d:e:f)
Example 2—The next example shows a typical expression that is used to specify min:typ:max format
values:
val - (32'd 50: 32'd 75: 32'd 100)
### 5.4 Expression bit lengths

Controlling the number of bits that are used in expression evaluations is important if consistent results are to
be achieved. Some situations have a simple solution; for example, if a bitwise and operation is specified on
two 16-bit regs, then the result is a 16-bit value. However, in some situations, it is not obvious how many
bits are used to evaluate an expression or what size the result should be.
For example, should an arithmetic add of two 16-bit values perform the evaluation using 16 bits, or should
the evaluation use 17 bits in order to allow for a possible carry overflow? The answer depends on the type of
device being modeled and whether that device handles carry overflow. The Verilog HDL uses the bit length
of the operands to determine how many bits to use while evaluating an expression. The bit length rules are
given in 5.4.1. In the case of the addition operator, the bit length of the largest operand, including the left-
hand side of an assignment, shall be used.
For example:
reg [15:0] a, b; // 16-bit regs
reg [15:0] sumA; // 16-bit reg
reg [16:0] sumB; // 17-bit reg
sumA = a + b; // expression evaluates using 16 bits
sumB = a + b; // expression evaluates using 17 bits
#### 5.4.1 Rules for expression bit lengths

The rules governing the expression bit lengths have been formulated so that most practical situations have a
natural solution.
The number of bits of an expression (known as the size of the expression) shall be determined by the
operands involved in the expression and the context in which the expression is given.
A self-determined expression is one where the bit length of the expression is solely determined by the
expression itself—for example, an expression representing a delay value.
A context-determined expression is one where the bit length of the expression is determined by the bit length
of the expression and by the fact that it is part of another expression. For example, the bit size of the right-
hand expression of an assignment depends on itself and the size of the left-hand side.
Table 5-22 shows how the form of an expression shall determine the bit lengths of the results of the
expression. In Table 5-22, i, j, and k represent expressions of an operand, and L(i) represents the bit
length of the operand represented by i.
Multiplication may be performed without losing any overflow bits by assigning the result to something wide
enough to hold it.
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
63
#### 5.4.2 Example of expression bit-length problem

During the evaluation of an expression, interim results shall take the size of the largest operand (in case of an
assignment, this also includes the left-hand side). Care has to be taken to prevent loss of a significant bit
during expression evaluation. The example below describes how the bit lengths of the operands could result
in the loss of a significant bit.
Given the following declarations:
reg [15:0] a, b, answer; // 16-bit regs
the intent is to evaluate the expression
answer = (a + b) >> 1; //will not work properly
where a and b are to be added, which can result in an overflow, and then shifted right by 1 bit to preserve the
carry bit in the 16-bit answer.
A problem arises, however, because all operands in the expression are of a 16-bit width. Therefore, the
expression (a + b) produces an interim result that is only 16 bits wide, thus losing the carry bit before the
evaluation performs the 1-bit right shift operation.
Table 5-22—Bit lengths resulting from self-determined expressions
Expression
Bit length
Comments
Unsized constant numbera
Same as integer
Sized constant number
As given
i op j, where op is:
+  -  *  /  %  &  |  ^  ^~  ~^
max(L(i),L(j))
op i, where op is:
+  -  ~
L(i)
i op j, where op is:
===  !==  ==  !=  >  >=  <  <=
## 1 bit

Operands are sized to max(L(i),L(j))
i op j, where op is:
&&  ||
## 1 bit

All operands are self-determined
op i, where op is:
&  ~&  |  ~|  ^  ~^  ^~ !
## 1 bit

All operands are self-determined
i op j, where op is:
>>   <<   **   >>>   <<<
L(i)
j is self-determined
i ? j : k
max(L(j),L(k))
i is self-determined
{i,...,j}
L(i)+..+L(j)
All operands are self-determined
{i{j,..,k}}
i * (L(j)+..+L(k))
All operands are self-determined
aIf an unsized constant is part of an expression that is longer than 32 bits and if the most significant bit
is unknown (X or x) or three-state (Z or z), the most significant bit is extended up to the size of the
expression. Otherwise, signed constants are sign-extended and unsigned constants are zero-extended.
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
64
Copyright © 2006 IEEE. All rights reserved.
The solution is to force the expression (a + b) to evaluate using at least 17 bits. For example, adding an
integer value of 0 to the expression will cause the evaluation to be performed using the bit size of integers.
The following example will produce the intended result:
answer = (a + b + 0) >> 1; //will work correctly
In the following example:
module bitlength();
   reg [3:0] a,b,c;
   reg [4:0] d;
   initial begin
     a = 9;
     b = 8;
     c = 1;
     $display("answer = %b", c ? (a&b) : d);
     end
   endmodule
the $display statement will display
   answer = 01000
By itself, the expression a&b would have the bit length 4, but because it is in the context of the conditional
expression, which uses the maximum bit length, the expression a&b actually has length 5, the length of d.
#### 5.4.3 Example of self-determined expressions

reg [3:0] a;
reg [5:0] b;
reg [15:0] c;
initial begin
   a = 4'hF;
   b = 6'hA;
   $display("a*b=%h", a*b);// expression size is self-determined
   c = {a**b};           // expression a**b is self-determined
                         // due to concatenation operator {}
       $display("a**b=%h", c);
   c = a**b;             // expression size is determined by c
   $display("c=%h", c);
end
Simulator output for this example:
a*b=16  // 'h96 was truncated to 'h16 since expression size is 6
a**b=1  // expression size is  4 bits (size of a)
c=ac61  // expression size is 16 bits (size of c)
### 5.5 Signed expressions

Controlling the sign of an expression is important if consistent results are to be achieved. In addition to the
rules outlined in 5.5.1 through 5.5.4, two system functions shall be used to handle type casting on
expressions: $signed() and $unsigned(). These functions shall evaluate the input expression and return a
value with the same size and value of the input expression and the type defined by the function:
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
65
$signed - returned value is signed
$unsigned - returned value is unsigned
For example:
reg [7:0] regA, regB;
reg signed [7:0] regS;
regA = $unsigned(-4);     // regA = 8'b11111100
regB = $unsigned(-4'sd4); // regB = 8'b00001100
regS = $signed  (4'b1100); // regS = -4
#### 5.5.1 Rules for expression types

The following are the rules for determining the resulting type of an expression:
—
Expression type depends only on the operands. It does not depend on the left-hand side (if any).
—
Decimal numbers are signed.
—
Based_numbers are unsigned, except where the s notation is used in the base specifier (as in
"4'sd12").
—
Bit-select results are unsigned, regardless of the operands.
—
Part-select results are unsigned, regardless of the operands even if the part-select specifies the entire
vector.
reg [15:0] a;
reg signed [7:0] b;
initial
a = b[7:0]; // b[7:0] is unsigned and therefore zero-extended
—
Concatenate results are unsigned, regardless of the operands.
—
Comparison results (1, 0) are unsigned, regardless of the operands.
—
Reals converted to integers by type coercion are signed
—
The sign and size of any self-determined operand are determined by the operand itself and
independent of the remainder of the expression.
—
For nonself-determined operands, the following rules apply:
—
If any operand is real, the result is real.
—
If any operand is unsigned, the result is unsigned, regardless of the operator.
—
If all operands are signed, the result will be signed, regardless of operator, except when
specified otherwise.
#### 5.5.2 Steps for evaluating an expression

The following are the steps for evaluating an expression:
—
Determine the expression size based upon the standard rules of expression size determination.
—
Determine the sign of the expression using the rules outlined in 5.5.1.
—
Propagate the type and size of the expression (or self-determined subexpression) back down to the
context-determined operands of the expression. In general, any context-determined operand of an
operator shall be the same type and size as the result of the operator. However, there are two
exceptions:
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
66
Copyright © 2006 IEEE. All rights reserved.
—
If the result type of the operator is real and if it has a context-determined operand that is not
real, that operand shall be treated as if it were self-determined and then converted to real just
before the operator is applied.
—
The relational and equality operators have operands that are neither fully self-determined nor
fully context-determined. The operands shall affect each other as if they were context-deter-
mined operands with a result type and size (maximum of the two operand sizes) determined
from them. However, the actual result type shall always be 1 bit unsigned. The type and size of
the operand shall be independent of the rest of the expression and vice versa.
—
When propagation reaches a simple operand as defined in 5.2 (a primary as defined in A.8.4), then
that operand shall be converted to the propagated type and size. If the operand must be extended,
then it shall be sign-extended only if the propagated type is signed.
#### 5.5.3 Steps for evaluating an assignment

The following are the steps for evaluating an assignment:
—
Determine the size of the right-hand side by the standard assignment size determination rules (see
5.4).
—
If needed, extend the size of the right-hand side, performing sign extension if, and only if, the type
of the right-hand side is signed.
#### 5.5.4 Handling X and Z in signed expressions

If a signed operand is to be resized to a larger signed width and the value of the sign bit is X, the resulting
value shall be bit-filled with Xs. If the sign bit of the value is Z, then the resulting value shall be bit-filled
with Zs. If any bit of a signed value is X or Z, then any nonlogical operation involving the value shall result
in the entire resultant value being an X and the type consistent with the expression’s type.
### 5.6 Assignments and truncation

If the width of the right-hand expression is larger than the width of the left-hand side in an assignment, the
MSBs of the right-hand expression will always be discarded to match the size of the left-hand side.
Implementations are not required to warn or report any errors related to assignment size mismatch or
truncation. Truncating the sign bit of a signed expression may change the sign of the result.
For example:
reg      [5:0] a;
reg signed [4:0] b;
initial begin
  a = 8'hff; // After the assignment, a = 6'h3f
  b = 8'hff; // After the assignment, b = 5'h1f
end
 For example:
reg      [0:5] a;
reg signed [0:4] b, c;
initial begin
  a = 8'sh8f; // After the assignment, a = 6'h0f
  b = 8'sh8f; // After the assignment, b = 5'h0f
  c = -113;   // After the assignment, c = 15
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
67
  // 1000_1111 = (-'h71 = -113) truncates to ('h0F = 15)
end
 For example:
reg      [7:0] a;
reg signed [7:0] b;
reg signed [5:0] c, d;
initial begin
  a = 8'hff;
  c = a;      // After the assignment, c = 6'h3f
  b = -113;
  d = b;      // After the assignment, d = 6'h0f
end
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
