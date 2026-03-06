---
title: "Section 11: IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language"
document: "SystemVerilog Language Reference Manual"
standard: "IEEE 1800-2017"
domain: "SystemVerilog"
section: "11"
source_txt: "section-11-operators-and-expressions.txt"
source_pdf: "/Users/richarddje/Documents/github/SystemVerilog-LRM-IEEE-1800-2017.pdf"
---

# Section 11: IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language

IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
254
Copyright © 2018 IEEE. All rights reserved.
11. Operators and expressions
### 11.1 General

This clause describes the following:
—
Expression semantics
—
Operations on expressions
—
Operator precedence
—
Operand size extension rules
—
Signed and unsigned operation rules
—
Bit and part-select operations and longest static prefix
—
Bit-stream operations
### 11.2 Overview

This clause describes the operators and operands available in SystemVerilog and how to use them to form
expressions.
An expression is a construct that combines operands with operators to produce a result that is a function of
the values of the operands and the semantic meaning of the operator. Any legal operand, such as a net bit-
select, without any operator is considered an expression. Wherever a value is needed in a SystemVerilog
statement, an expression can be used.
An operand can be one of the following:
—
Constant literal number, including real literals
—
String literal
—
Parameter, including local and specify parameters
—
Parameter bit-select or part-select, including local and specify parameters
—
Net (see 6.7)
—
Net bit-select or part-select
—
Variable (see 6.8)
—
Variable bit-select or part-select
—
Structure, either packed or unpacked
—
Structure member
—
Packed structure bit-select or part-select
—
Union, packed, unpacked, or tagged
—
Union member
—
Packed union bit-select or part-select
—
Array, either packed or unpacked
—
Packed array bit-select, part-select, element, or slice
—
Unpacked array element bit-select or part-select, element, or slice
—
A call to a user-defined function, system-defined function, or method that returns any of the above
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
255
Copyright © 2018 IEEE. All rights reserved.
#### 11.2.1 Constant expressions

Some statement constructs require an expression to be a constant expression. The operands of a constant
expression consist of constant numbers, strings, parameters, constant bit-selects and part-selects of
parameters, constant function calls (see 13.4.3), and constant system function calls only. Constant
expressions can use any of the operators defined in Table 11-1.
Constant system function calls are calls to certain built-in system functions where the arguments meet
conditions outlined in this subclause. When used in constant expressions, these function calls shall be
evaluated at elaboration time. The system functions that may be used in constant system function calls are
pure functions, i.e., those whose value depends only on their input arguments and that have no side effects.
Certain built-in system functions where the arguments are constant expressions are constant system function
calls. Specifically, these are the conversion system functions listed in 20.5, the mathematical system
functions listed in 20.8, and the bit vector system functions listed in 20.9.
The data query system functions listed in 20.6 and the array query system functions listed in 20.7 are
normally also constant system function calls even when their arguments are not constant. See those
subclauses for the conditions under which these query system function calls are considered to be constant
expressions.
#### 11.2.2 Aggregate expressions

Unpacked structure and array data objects, as well as unpacked structure and array constructors, can all be
used as aggregate expressions. A multi-element slice of an unpacked array can also be used as an aggregate
expression.
Aggregate expressions can be copied in an assignment, through a port, or as an argument to a subroutine.
Aggregate expressions can also be compared with equality or inequality operators.
If the two operands of a comparison operator are aggregate expressions, they shall be of equivalent type as
defined in 6.22.2. Assignment compatibility of aggregate expressions is defined in 6.22.3 and, for arrays, in
7.6.
### 11.3 Operators

The symbols for the SystemVerilog operators are similar to those in the C programming language.
Syntax 11-1 and Table 11-1 list these operators.

```ebnf
assignment_operator ::=
```

// from A.6.2
= | += | -= | *= | /= | %= | &= | |= | ^= | <<= | >>= | <<<= | >>>=
```ebnf
conditional_expression ::=
```

// from A.8.3
cond_predicate ? { attribute_instance } expression : expression
```ebnf
unary_operator ::=
```

// from A.8.6
+ | - | ! | ~ | & | ~& | | | ~| | ^ | ~^ | ^~
```ebnf
binary_operator ::=
```

+ | - | * | / | % | == | != | === | !== | ==? | !=? | && | || | **
| < | <= | > | >= | & | | | ^ | ^~ | ~^ | >> | << | >>> | <<<
| -> | <->
```ebnf
inc_or_dec_operator ::= ++ | --
stream_operator ::= >> | <<
```

// from A.8.1
Syntax 11-1—Operator syntax (excerpt from Annex A)
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
256
Copyright © 2018 IEEE. All rights reserved.

#### 11.3.1 Operators with real operands

Table 11-1 shows what operators may be applied to real operands.
The result of using logical or relational operators or the inside operator on real operands shall be a
single-bit scalar value.
Table 11-1—Operators and data types
Operator token
Name
Operand data types
=
Binary assignment operator
Any
+=
-=
/=
*=
Binary arithmetic assignment operators
Integral, real, shortreal
%=
Binary arithmetic modulus assignment operator
Integral
&=
|=
^=
Binary bitwise assignment operators
Integral
>>= <<=
Binary logical shift assignment operators
Integral
>>>= <<<=
Binary arithmetic shift assignment operators
Integral
?:
Conditional operator
Any
+
-
Unary arithmetic operators
Integral, real, shortreal
!
Unary logical negation operator
Integral, real, shortreal
~
&
~&
|
~|
^
~^
^~
Unary logical reduction operators
Integral
+
-
*
/
**
Binary arithmetic operators
Integral, real, shortreal
%
Binary arithmetic modulus operator
Integral
&
|
^
^~
~^
Binary bitwise operators
Integral
>> <<
Binary logical shift operators
Integral
>>> <<<
Binary arithmetic shift operators
Integral
&&
||
–> <–>
Binary logical operators
Integral, real, shortreal
<
<=
>
>=
Binary relational operators
Integral, real, shortreal
===
!==
Binary case equality operators
Any except real and
shortreal
==
!=
Binary logical equality operators
Any
==?
!=?
Binary wildcard equality operators
Integral
++
--
Unary increment, decrement operators
Integral, real, shortreal
inside
Binary set membership operator
Singular for the left operand
dista
aThe dist operator is described in 16.14.2 and 18.5.4.
Binary distribution operator
Integral
{}
{{}}
Concatenation, replication operators
Integral
{<<{}} {>>{}}
Stream operators
Integral
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
257
Copyright © 2018 IEEE. All rights reserved.
For other operators, if any operand, except before the ? in the conditional operator, is real, the result is
real. Otherwise, if any operand, except before the ? in the conditional operator, is shortreal, the result is
shortreal.
Real operands can also be used in the following expressions:
str.realval // structure or union member
realarray[intval] // array element
See 6.12.1 for more information on use of real numbers.
#### 11.3.2 Operator precedence

Operator precedence and associativity are listed in Table 11-2. The highest precedence is listed first.
Operators shown on the same row in Table 11-2 shall have the same precedence. Rows are arranged in order
of decreasing precedence for the operators. For example, *, /, and % all have the same precedence, which is
higher than that of the binary + and – operators.
All operators shall associate left to right with the exception of the conditional (?:), implication (->), and
equivalence (<->) operators, which shall associate right to left. Associativity refers to the order in which the
operators having the same precedence are evaluated. Thus, in the following example, B is added to A, and
then C is subtracted from the result of A+B.
Table 11-2—Operator precedence and associativity
Operator
Associativity
Precedence
()
[]
::
.

Left
Highest
+
-
!
~
&
~&
|
~|
^
~^
^~
++
-- (unary)

**
Left
*
/
%
Left
+
- (binary)
Left
<<
>>
<<<
>>>
Left
<
<=
>
>=
inside
dist
Left
==
!=
===
!==
==?
!=?
Left
& (binary)
Left
^
~^
^~ (binary)
Left
| (binary)
Left
&&
Left
||
Left
?: (conditional operator)
Right
–> <–>
Right
=
+=
-=
*=
/=
%=
&=
^=
|=
<<=
>>=
<<<=
>>>=
:=
:/
<=
None
{}
{{}}
Concatenation
Lowest
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
258
Copyright © 2018 IEEE. All rights reserved.
A + B - C
When operators differ in precedence, the operators with higher precedence shall associate first. In the
following example, B is divided by C (division has higher precedence than addition), and then the result is
added to A.
A + B / C
Parentheses can be used to change the operator precedence.
(A + B) / C
// not the same as A + B / C
#### 11.3.3 Using integer literals in expressions

Integer literals can be used as operands in expressions. An integer literal can be expressed as the following:
—
An unsized, unbased integer (e.g., 12)
—
An unsized, based integer (e.g., 'd12, 'sd12)
—
A sized, based integer (e.g., 16'd12, 16'sd12)
See 5.7.1 for integer literal syntax.
A negative value for an integer with no base specifier shall be interpreted differently from an integer with a
base specifier. An integer with no base specifier shall be interpreted as a signed value in two’s-complement
form. An integer with an unsigned base specifier shall be interpreted as an unsigned value.
The following example shows four ways to write the expression “minus 12 divided by 3.” Note that -12 and
-'d12 both evaluate to the same two’s-complement bit pattern, but, in an expression, the -'d12 loses its
identity as a signed negative number.
int IntA;
IntA = -12 / 3;
// The result is -4
IntA = -'d 12 / 3;
// The result is 1431655761
IntA = -'sd 12 / 3;
// The result is -4
IntA = -4'sd 12 / 3;
// -4'sd12 is the negative of the 4-bit
// quantity 1100, which is -4. -(-4) = 4
// The result is 1
#### 11.3.4 Operations on logic (4-state) and bit (2-state) types

Operators may be applied to 2-state values or to a mixture of 2-state and 4-state values. The result is the
same as if all values were treated as 4-state values. In most cases, if all operands are 2-state, the result is in
the 2-state value set. The only exceptions involve operators that produce an x result for operands in the
2-state value set (e.g., division by zero).
int n = 8, zero = 0;
int res = 'b01xz | n;
// res gets 'b11xz coerced to int, or 'b1100
int sum = n + n;
// sum gets 16 coerced to int, or 16
int sumx = 'x + n;
// sumx gets 'x coerced to int, or 0
int div2 = n/zero + n;
// div2 gets 'x coerced to int, or 0
integer div4 = n/zero + n;
// div4 gets 'x
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
259
Copyright © 2018 IEEE. All rights reserved.
#### 11.3.5 Operator expression short circuiting

The operators shall follow the associativity rules while evaluating an expression as described in 11.3.2.
Some operators (&&, ||, ->, and ?:) shall use short-circuit evaluation; in other words, some of their
operand expressions shall not be evaluated if their value is not required to determine the final value of the
operation. The detailed short-circuiting behavior of each of these operators is described in its corresponding
subclause (11.4.7 and 11.4.11). All other operators shall not use short-circuit evaluation—all of their
operand expressions are always evaluated. When short circuiting occurs, any side effects or runtime errors
that would have occurred due to evaluation of the short-circuited operand expression shall not occur.
For example:
logic regA, regB, regC, result ;
function logic myFunc(logic x);
...
endfunction
result = regA & (regB | myFunc(regC)) ;
Even if regA is known to be zero, the subexpression (regB | myFunc(regC)) will be evaluated and any
side effects caused by calling myFunc(regC) will occur.
Note that implementations are free to optimize by omitting evaluation of subexpressions as long as the
simulation behavior (including side effects) is as if the standard rules were followed.
#### 11.3.6 Assignment within an expression

An expression can include a blocking assignment, provided it does not have a timing control. These
blocking assignments shall be enclosed in parentheses to avoid common mistakes such as using a=b for
a==b or using a|=b for a!=b.
if ((a=b)) b = (a+=1);
a = (b = (c = 5));
The semantics of such an assignment expression is that of a function that evaluates the right-hand side, casts
the right-hand side to the left-hand data type, stacks it, updates the left-hand side, and returns the stacked
value. The data type of the value that is returned is the data type of the left-hand side. If the left-hand side is
a concatenation, then the data type of the value that is returned shall be an unsigned integral data type whose
bit length is the sum of the length of its operands.
It shall be illegal to include an assignment operator in an event expression, in an expression within a
procedural continuous assignment, or in an expression that is not within a procedural statement.
### 11.4 Operator descriptions

#### 11.4.1 Assignment operators

In addition to the simple assignment operator, =, SystemVerilog includes the C assignment operators and
special bitwise assignment operators: +=, -=, *=, /=, %=, &=, |=, ^=, <<=, >>=, <<<=, and >>>=. An
assignment operator is semantically equivalent to a blocking assignment, with the exception that any
left-hand index expression is only evaluated once. For example:
a[i]+=2; // same as a[i] = a[i] +2;
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
260
Copyright © 2018 IEEE. All rights reserved.
#### 11.4.2 Increment and decrement operators

SystemVerilog includes the C increment and decrement assignment operators ++i, --i, i++, and i--.
These do not need parentheses when used in expressions. These increment and decrement assignment
operators behave as blocking assignments.
The ordering of assignment operations relative to any other operation within an expression is undefined. An
implementation can warn whenever a variable is both written and read-or-written within an integral
expression or in other contexts where an implementation cannot guarantee order of evaluation. For example:
i = 10;
j = i++ + (i = i - 1);
After execution, the value of j can be 18, 19, or 20 depending upon the relative ordering of the increment
and the assignment statements.
The increment and decrement operators, when applied to real operands, increment or decrement the operand
by 1.0.
#### 11.4.3 Arithmetic operators

The binary arithmetic operators are given in Table 11-3.
The integer division shall truncate any fractional part toward zero. For the division or modulus operators, if
the second operand is a zero, then the entire result value shall be x. The modulus operator (for example,
a % b) gives the remainder when the first operand is divided by the second and thus is zero when b divides
a exactly. The result of a modulus operation shall take the sign of the first operand.
If either operand of the power operator is real, then the result type shall be real (see 11.3.1). The result of the
power operator is unspecified if the first operand is zero and the second operand is nonpositive or if the first
operand is negative and the second operand is not an integral value.
If neither operand of the power operator is real, then the result type shall be determined as outlined in 11.6.1
and 11.8.1. The result value is 'x if the first operand is zero and the second operand is negative. The result
value is 1 if the second operand is zero.
In all cases, the second operand of the power operator shall be treated as self-determined.
These statements are illustrated in Table 11-4.
Table 11-3—Arithmetic operators defined
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
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
261
Copyright © 2018 IEEE. All rights reserved.
The unary arithmetic operators shall take precedence over the binary operators. The unary operators are
given in Table 11-5.
For the arithmetic operators, if any operand bit value is the unknown value x or the high-impedance value z,
then the entire result value shall be x.
Table 11-6 gives examples of some modulus and power operations.
Table 11-4—Power operator rules
op1 is
negative < -1
op1 is
-1
op1 is
zero
op1 is
1
op1 is
positive > 1
op2 is positive
op1 ** op2
op2 is odd -> -1
op2 is even -> 1
0
1
op1 ** op2
op2 is zero
1
1
1
1
1
op2 is negative
0
op2 is odd -> -1
op2 is even -> 1
'x
1
0
Table 11-5—Unary operators defined
+m
Unary plus m (same as m)
-m
Unary minus m
Table 11-6—Examples of modulus and power operators
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
The result takes the sign of the first operand.
–4'd12 % 3
1
–4'd12 is seen as a large positive number that leaves a remainder of 1 when divided by 3.
## 3 ** 2

9
## 3  3

## 2 ** 3

8
## 2  2  2

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

## 0 ** –1

'x
## 0 ** –1 = 1/0. Integer division by zero is 'x.

## 9 ** 0.5

3.0
Real square root.
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
262
Copyright © 2018 IEEE. All rights reserved.
##### 11.4.3.1 Arithmetic expressions with unsigned and signed types

Nets and variables can be explicitly declared as unsigned or signed. The byte, shortint, int, integer,
and longint data types are signed by default. Other data types are unsigned by default.
A value assigned to an unsigned variable or net shall be treated as an unsigned value. A value assigned to a
signed variable or net shall be treated as signed. Signed values, except for those assigned to real variables,
shall use a two’s-complement representation. Values assigned to real variables shall use a floating-point
representation. Conversions between signed and unsigned values shall keep the same bit representation;
only the interpretation changes.
Table 11-7 lists how arithmetic operators interpret each data type.
The following example shows various ways to divide “minus twelve by three”—using integer and logic
variables in expressions.
integer intS;
var logic [15:0] U;
var logic signed [15:0] S;
intS = -4'd12;
U = intS / 3;
// expression result is -4,
// intS is an integer data type, U is 65532
U = -4'd12;
// U is 65524
intS = U / 3;
// expression result is 21841,
// U is a logic data type
intS = -4'd12 / 3;
// expression result is 1431655761.
// -4'd12 is effectively a 32-bit logic data type
U = -12 / 3;
// expression result is -4, -12 is effectively
// an integer data type. U is 65532
S = -12 / 3;
// expression result is -4. S is a signed logic
### 9.0 ** (1/2)

1.0
Integer division truncates exponent to zero.
–3.0 ** 2.0
9.0
Defined because real 2.0 is still integral value.
Table 11-7—Data type interpretation by arithmetic operators
Data type
Interpretation
Unsigned net
Unsigned
Signed net
Signed, two’s-complement
Unsigned variable
Unsigned
Signed variable
Signed, two’s-complement
Real variable
Signed, floating point
Table 11-6—Examples of modulus and power operators  (continued)
Expression
Result
Comments
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
263
Copyright © 2018 IEEE. All rights reserved.
S = -4'sd12 / 3;
// expression result is 1. -4'sd12 is actually 4.
// The rules for integer division yield 4/3==1
#### 11.4.4 Relational operators

Table 11-8 lists and defines the relational operators.
An expression using these relational operators shall yield the scalar value 0 if the specified relation is false
or the value 1 if it is true. If either operand of a relational operator contains an unknown (x) or
high-impedance (z) value, then the result shall be a 1-bit unknown value (x).
When one or both operands of a relational expression are unsigned, the expression shall be interpreted as a
comparison between unsigned values. If the operands are of unequal bit lengths, the smaller operand shall be
zero-extended to the size of the larger operand.
When both operands are signed, the expression shall be interpreted as a comparison between signed values.
If the operands are of unequal bit lengths, the smaller operand shall be sign-extended to the size of the larger
operand. See 11.8.2 for more information.
If either operand is a real operand, then the other operand shall be converted to an equivalent real value and
the expression shall be interpreted as a comparison between real values.
All the relational operators shall have the same precedence. Relational operators shall have lower
precedence than arithmetic operators.
The following examples illustrate the implications of this precedence rule:
a < b - 1
// this expression is the same as
a < (b - 1)
// this expression, but . . .
b - (1 < a)
// this one is not the same as
b - 1 < a
// this expression
When b - (1 < a) evaluates, the relational expression evaluates first, and then either zero or one is
subtracted from b. When b - 1 < a evaluates, the value of b operand is reduced by one and then
compared with a.
#### 11.4.5 Equality operators

The equality operators shall rank lower in precedence than the relational operators. Table 11-9 lists and
defines the equality operators.
Table 11-8—Definitions of relational operators
a < b
a less than b
a > b
a greater than b
a <= b
a less than or equal to b
a >= b
a greater than or equal to b
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
264
Copyright © 2018 IEEE. All rights reserved.
All four equality operators shall have the same precedence. These four operators compare operands bit for
bit. As with the relational operators, the result shall be 0 if comparison fails and 1 if it succeeds.
When one or both operands are unsigned, the expression shall be interpreted as a comparison between
unsigned values. If the operands are of unequal bit lengths, the smaller operand shall be zero-extended to the
size of the larger operand.
When both operands are signed, the expression shall be interpreted as a comparison between signed values.
If the operands are of unequal bit lengths, the smaller operand shall be sign-extended to the size of the larger
operand. See 11.8.2 for more information.
If either operand is a real operand, then the other operand shall be converted to an equivalent real value, and
the expression shall be interpreted as a comparison between real values.
The logical equality (or case equality) operator is a legal operation if either operand is a class handle or the
literal null, and one of the operands is assignment compatible with the other. The logical equality (or case
equality) operator is a legal operation if either operand is a chandle or the literal null. In both cases, the
operator compares the values of the class handles, interface class handles, or chandles.
For the logical equality and logical inequality operators (== and !=), if, due to unknown or high-impedance
bits in the operands, the relation is ambiguous, then the result shall be a 1-bit unknown value (x).
For the case equality and case inequality operators (=== and !==), the comparison shall be done just as it is
in the procedural case statement (see 12.5). Bits that are x or z shall be included in the comparison and shall
match for the result to be considered equal. The result of these operators shall always be a known value,
either 1 or 0.
#### 11.4.6 Wildcard equality operators

The wildcard equality operators shall have the same precedence as the equality operators. Table 11-10 lists
and defines the wildcard equality operators.
The wildcard equality operator (==?) and inequality operator (!=?) treat X and Z values in a given bit
position of their right operand as a wildcard. X and Z values in the left operand are not treated as wildcards.
Table 11-9—Definitions of equality operators
a === b
a equal to b, including x and z
a !== b
a not equal to b, including x and z
a == b
a equal to b, result can be unknown
a != b
a not equal to b, result can be unknown
Table 11-10—Wildcard equality and wildcard inequality operators
Operator
Usage
Description
==?
a ==? b
a equals b, X and Z values in b act as wildcards
!=?
a !=? b
a does not equal b, X and Z values in b act as wildcards
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
