---
title: "Section Annex.G: IEEE Standard for VHDL Language Reference Manual"
document: "VHDL Language Reference Manual"
standard: "IEEE 1076-2019"
domain: "VHDL"
section: "Annex.G"
source_txt: "section-Annex_G-informative-guide-to-use-of-standard-packages.txt"
source_pdf: "/Users/richarddje/Documents/github/VHDL-LRM-IEEE-1076-2019.pdf"
---

# Section Annex.G: IEEE Standard for VHDL Language Reference Manual

IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
560
Copyright © 2019 IEEE. All rights reserved.
Annex G
(informative)
Guide to use of standard packages
G.1 Using the MATH_REAL and MATH_COMPLEX packages
G.1.1 General
The information in this clause is intended to be a brief guide to using the MATH_REAL and
MATH_COMPLEX packages, but it is not a normative part of the standard. As a standard, this set of
packages provides a means of building models that interoperate and port to different tools, provided that the
user adheres to a set of guidelines required by the standard and the strict typing imposed by the VHDL
language.
G.1.2 Package bodies for MATH_REAL and MATH_COMPLEX
The collection of files in the IEEE 1076 Open Source Repository that forms part of this standard includes
package bodies for MATH_REAL and MATH_COMPLEX. These package bodies are intended to provide a
guideline for implementors. They are not a normative part of this standard, but suggest ways in which
implementors may implement the MATH_REAL and MATH_COMPLEX packages. Implementors may
also use the package bodies as a guideline to verify their implementation of the packages.
G.1.3 Predefined data types, operators, and precision for MATH_REAL
The MATH_REAL package is built on top of the standard data type (REAL), operators, and precision
requirements for floating-point operations defined in STD.STANDARD.
G.1.4 Use and constraints of pseudo-random number generator in MATH_REAL
The pseudo-random number generator provided with the package is platform independent. In order to
generate a chain of pseudo-random numbers, the seed values shall be set only in the first call to the function.
A different chain of numbers is started every time the seed values are set. If multiple chains of
pseudo-random numbers are required, then different sets of seed values have to be used for every chain.
G.1.5 Precision across different platforms
It is important to note that the math package results may be slightly different on different platforms because
of variations in hardware support for floating-point arithmetic. These differences might not be immediately
apparent to the average VHDL user. However, since most workstations use the IEEE 754 floating-point
format, the variations are likely to be limited in practice.
G.1.6 Handling of overflow/underflow conditions
The detection of underflow/overflow is optional and implementation dependent.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
561
Copyright © 2019 IEEE. All rights reserved.
G.1.7 Testbench for the packages
A non-exhaustive testbench for the packages MATH_REAL and MATH_COMPLEX can be found in the
collection of files in the IEEE 1076 Open Source Repository that forms part of this standard.
G.1.8 Overloading side effect
Note that there is a side effect of adding functions for COMPLEX_POLAR when numerical expressions are
used. Numerical parameters for these functions are ambiguous, unless a qualifier is used to disambiguate
them. For example, SIN((0.0, 0.0)) is ambiguous. One has to say either SIN(COMPLEX'(0.0, 0.0)) or
SIN(COMPLEX_POLAR'(0.0, 0.0)).
G.1.9 Synthesizability of functions
Synthesizability of the functions defined in the mathematical packages is beyond the scope of this standard.
G.2 Using the STD_LOGIC_1164 package
G.2.1 General
This subclause is intended to be a brief guide to using the STD_LOGIC_1164 package. This package
provides a means of building models that interoperate, provided that the user adheres to a set of guidelines
required by the strict typing imposed by the VHDL language.
G.2.2 Value system
The value system in STD_LOGIC_1164 was developed to model a variety of digital device technologies.
The base type of the logic system is named “std_ulogic” where the “u” in the name signifies “unresolved.”
Each of the elements comprising the type have a specified semantic and a commonly used application. In
order for models to properly interoperate, one should interpret the meaning of each of the elements as
provided by the standard.
Value
Name
Usage
'U'
Uninitialized state
Used as a default value
'X'
Forcing unknown
Bus contentions, error conditions, etc.
'0'
Forcing zero
Transistor driven to GND
'1'
Forcing one
Transistor driven to VCC
'Z'
High impedance
3-state buffer outputs
'W'
Weak unknown
Bus terminators
'L'
Weak zero
Pull down resistors
'H'
Weak one
Pull up resistors
'–'
Don’t care
Used for synthesis and advanced modeling
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
562
Copyright © 2019 IEEE. All rights reserved.
G.2.3 Handling strengths
Behavioral modeling techniques rarely require knowledge of the strength of a signal’s value. Therefore, a
number of “strength stripper” functions have been designed to transform 'Z', 'W', 'L', 'H', and '-' into their
corresponding “forcing” strength counterparts.
Once in forcing strength, the model can simply respond to 'X's, '0's, 'l's, and 'U's as the need may arise. This
strength stripping is done by using one of the following functions:
G.2.4 Use of the uninitialized value
The 'U' value is located in the first position of the type. Therefore, any object declared to be of this base type
will be automatically initialized to 'U' unless expressly assigned a default expression.
Uninitialized values were designed to provide a means of detecting system values that have not changed
from their uninitialized state since the time of system initialization. Hence, the logical tables for AND, OR,
NAND, NOR, XOR, XNOR, and NOT have been designed to propagate 'U' states whenever encountered.
The propagation of 'U's through a circuit gives the designer an understanding of where the system has failed
to be properly initialized.
G.2.5 Behavioral modeling for 'U' propagation
For behavioral modeling where 'U' propagation is desired, the function TO_UX01 will provide a reduction
in the state system, as far as the modeler is concerned, thereby easing the modeler’s task.
G.2.6 'U's related to conditional expressions
Case statements, “if” expressions, and selected signal assignments need to separately treat 'U' states and
provide a path for 'U' state propagation in order to propagate 'U's.
G.2.7 Structural modeling with logical tables
The logical tables are designed to generate output values in the range 'U', 'X', '0', and '1'. Therefore, once an
element of the nine-state system passes through any of the logical tables, it will be converted to forcing
strength. If the need arises for a weak or floating strength to be propagated through the remainder of a circuit
or to an output port, then the model developer shall be certain to assign the appropriate value accordingly.
G.2.8 X-handling: assignment of X’s
In assignments, the 'X' and '–' values differ minimally. The value '–', also known as “output don’t care,”
explicitly means that synthesis tools are allowed to generate either a '0' or a '1', whichever leads to minimal
circuitry, whereas 'X' usually appears during transitions or as a result of bus contentions or to flag model
generated internal error conditions, such as in the following waveform assignment:
To_X01 (...)
converts 'L' and 'H' to '0' and '1' respectively. All others are converted to 'X'.
To_UX0 1 (...)
converts 'L' and 'H' to '0' and '1' respectively. 'U's are propagated and all others are
converted to 'X'.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
563
Copyright © 2019 IEEE. All rights reserved.
S <= 'X' after 1 ns, '1' after 5 ns;
where the current value of S becomes indeterminate after 1 ns and then reaches '1' after 5 ns have elapsed.
G.2.9 Modeling with don’t care’s
G.2.9.1 Use of the don’t care state in synthesis models
For synthesis, a VHDL program is a specification of the functionality of a design. VHDL can also be used to
model (in order to simulate) real circuits. The former deals with logical function of the circuit, while the
latter is concerned with function of a circuit from an electrical point of view. The nine-state logic type usage
for synthesis is based on the assumption that the VHDL models will be logical function specifications and,
therefore, attempts to restrict the usage of the logic type to logical function. The motivation for allowing the
user to reference the values 'U' and 'X' (which do not specify the behavior of the circuit to be built, i.e., one
cannot build a circuit which “drives an 'X'”) is to allow such simulation artifacts to remain models for
synthesis for the sake of convenience. By having synthesis remove these references, the user is assuming
only the kind of usage (of 'U' and 'X') that catches error states that should never occur in hardware.
G.2.9.2 Semantics of '-'
In designing the resolution function and the various logic tables in the package body, '–' is almost
exclusively a syntactic shorthand for 'X', provided for compatibility with synthesis tools. This is evident
from the fact that '–' becomes 'X' as soon as it is operated upon and when it is converted to subtype X01 or
UX01. The “output don’t care” value represents either a '1' or a '0' as the output of combinatorial circuitry,
with respect to state encoding in particular.
G.2.10 Resolution function
In digital logic design, there are a number of occasions in which driving outputs of more than one device are
connected together. The most common of which is TRI-STATE® buses in which memory data ports are
connected to each other and to controlling microprocessors.20 Another common case is one in which
multiple drivers are parallel driving a heavily loaded signal path. In each of these cases, the VHDL language
requires that the signals used to interconnect those devices be “resolved” signal types.
Focusing on resolution: when two signals’ values are driving the same “wire,” some resulting value will be
observed on that wire. For example, if two parallel buffers both drive '1' onto a signal, then the signal will be
'1'. If a TRI-STATE driver is in the high-impedance state 'Z' and another driver is in the forcing one '1' state,
then the combination of those two signal values will result in a value of '1' appearing on the wire.
The resolution function built to STD_LOGIC_1164 operates on the principal that weak values dominate
over high-impedance values and forcing values dominate over weak values.
G.2.11 Using STD_ULOGIC vs. STD_LOGIC
In deciding whether to use the resolved signal or unresolved signal type, a number of considerations need to
be made:
a)
Does the simulator run slower when using a resolved type than when using an unresolved type, or is
the simulator optimized for the STD_LOGIC data types?
20TRI-STATE is a registered trademark of National Semiconductor Corporation. This information is given for the convenience of users
of this standard and does not constitute an endorsement by the IEEE of these products. Equivalent products may be used if they can be
shown to lead to the same results.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
564
Copyright © 2019 IEEE. All rights reserved.
b)
How many sources are there for a signal?
Each of these is considered, in order, as follows:
In the absence of other considerations, the choice between an unresolved and a resolved signal type should
depend on whether the signal is intended to have only one source, or whether multiple sources are intended.
In the former case, a scalar signal should be of type STD_ULOGIC, since inadvertent connection of multiple
sources can be detected during analysis or elaboration. Similarly, a vector signal should be of type
STD_ULOGIC_VECTOR, for the same reason. In the latter case a scalar signal should be of type
STD_LOGIC, and a vector signal should be of type STD_LOGIC_VECTOR.
The same considerations apply to ports, regardless of the actual signals to which they are connected. An
input port can be of either an unresolved or a resolved type, as the question of sources is not relevant. An
output or bidirectional port with one internal source should be of type STD_ULOGIC or
STD_ULOGIC_VECTOR. An output or bidirectional port with multiple internal sources should be of type
STD_LOGIC or STD_LOGIC_VECTOR. The values contributed by the internal sources are resolved to
determine the value driven by the port. Since STD_LOGIC is a subtype of STD_ULOGIC, ports and signals
of these types can be interconnected freely. Similarly, since STD_LOGIC_VECTOR is a subtype of
STD_ULOGIC_VECTOR, ports and signals of these vector types can be interconnected freely.
G.3 Notes on the synthesis package functions
G.3.1 General
This subclause provides notes on functions included in the NUMERIC_BIT, NUMERIC_STD,
NUMERIC_BIT_UNSIGNED, and NUMERIC_STD_UNSIGNED packages. Except where otherwise
indicated, notes applying to operations on type UNSIGNED, NUMERIC_BIT, and NUMERIC_STD also
apply to operations on BIT_VECTOR, NUMERIC_BIT_UNSIGNED, STD_ULOGIC_VECTOR, and
NUMERIC_STD_UNSIGNED.
The appearance of a code fragment in this subclause does not require a synthesis tool conforming to this
standard to accept the construct represented by that fragment.
G.3.2 General considerations
G.3.2.1 Mixing SIGNED and UNSIGNED operands
The NUMERIC_BIT and NUMERIC_STD packages do not provide functions for mixing SIGNED and
UNSIGNED operands. To do so would make it necessary to use qualified expressions to disambiguate
commonly occurring forms. For example, with the declarations
variable S: SIGNED (3 downto 0);
variable U: UNSIGNED (4 downto 0);
if the arithmetic and relational functions allowed mixing of SIGNED and UNSIGNED operands, it would be
necessary to rewrite the expressions
S >= "0000"
and
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
565
Copyright © 2019 IEEE. All rights reserved.
U + "1"
as
S >= SIGNED'("0000")
and
U + UNSIGNED'("1")
To apply a binary operation from the NUMERIC_BIT or NUMERIC_STD package to a combination of
SIGNED and UNSIGNED operands, the user must explicitly convert one of the operands to the other type
(see G.3.6.2).
G.3.2.2 Mixing vector and element operands
The packages do not declare functions that combine a vector with an operand that belongs to the element
type of the vector, other than the + and – functions. For example, with the declarations
signal A, B, S: SIGNED(3 downto 0);
signal C: BIT;
a user shall not write
S <= A * B(3);
or
S <= A * C;
or
S <= A / '1';
For the first and third example, a user may write instead
S <= A * B(3 downto 3);
and
S <= A / "1";
For the second example, the user may concatenate C with a 0-length vector
S <= A * (C & "");
G.3.3 Arithmetic operator functions
G.3.3.1 Overflow of maximum negative value
When the SIGNED operand to an abs (function A.1) or unary – (function A.2) function has the maximum
negative value for the number of elements that it has, the result is the maximum negative value of the same
size. This means, for example, that
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
566
Copyright © 2019 IEEE. All rights reserved.
- SIGNED'("1000")
evaluates to
"1000"
Similarly, in functions A.22 and A.25, when the first operand to the / operator has the maximum negative
value for the number of elements that it has, and when the second operand is either an INTEGER with the
value –1 or a SIGNED operand with a value equivalent to –1, the result is the same as the first operand,
rather than its complement:
SIGNED'("1000") / "11111"       evaluates to "1000"
SIGNED'("10000") / (-1)         evaluates to "10000"
To prevent overflow, a user may add an extra bit to the representation. For example, with the declarations
variable DIVIDEND: SIGNED (4 downto 0);
variable DIVISOR: INTEGER range -8 to 7;
variable QUOTIENT: SIGNED (5 downto 0);
one may write
QUOTIENT := (DIVIDEND(4) & DIVIDEND) / DIVISOR;
G.3.3.2 Lack of carry and borrow
When both operands of a binary arithmetic function + or – are either SIGNED or UNSIGNED, the function
returns a value with the same number of elements (bits) as the larger of the two operands. If one operand is
SIGNED or UNSIGNED and the other is INTEGER or NATURAL, the function returns a value with the
same number of elements as the vector operand. Thus, these functions do not return an extra bit to represent
a carry, borrow, or overflow value, nor do they generate a warning if a carry, borrow, or overflow occurs.
The choice not to generate a carry or borrow (and not to generate a warning) makes it easier to represent
counter operations in the VHDL source code via assignments such as
A := A + 1;
or
B <= B - "1";
To obtain the appropriate carry, borrow, or overflow value, a user may add an extra bit to the vector operand.
For example, with the declarations
signal U: UNSIGNED (4 downto 0);
signal S: SIGNED (5 downto 0);
signal SUM: UNSIGNED (5 downto 0);
signal DIFFERENCE: SIGNED (6 downto 0);
one may write
SUM <= ('0' & U) + 1;
DIFFERENCE <= (S(5) & S) - "1";
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
567
Copyright © 2019 IEEE. All rights reserved.
G.3.3.3 Return value for metalogical and high-impedance operands
If an operand to a NUMERIC_STD or NUMERIC_STD_UNSIGNED arithmetic function contains a
metalogical or high-impedance value, the function returns a vector in which every element has the value 'X'.
The function does not report a warning or error.
G.3.4 Relational operator functions
G.3.4.1 Justification of vector operands
The relational operator functions defined in the synthesis packages have a behavior different from the
default behavior defined by this standard for vector types. The default behavior compares the vector
elements left to right after the operands are left-justified, whereas the relational operator functions defined in
the synthesis packages treat their operands as representing binary integers.
Table G.1 compares results for the predefined relational operators applied to BIT_VECTORs with the
relational operators defined in the packages for SIGNED and UNSIGNED values. The results of relational
operators defined in the NUMERIC_BIT_UNSIGNED package for BIT_VECTORs and in the
NUMERIC_STD_UNSIGNED package for STD_ULOGIC_VECTORs are the same as the results for
UNSIGNED.
Table G.1—Relational operators examples
G.3.4.2 Expansion of vector operands compared to integers
When a relational operator compares a SIGNED or UNSIGNED operand value with an INTEGER or
NATURAL value, the function has the effect of converting the SIGNED or UNSIGNED operand to its
equivalent universal integer value and then doing the corresponding comparison of integer values. For
example:
(SIGNED'("111") > -8) = TRUE
and
(UNSIGNED'("111") < 8) = TRUE
That is, the INTEGER value may be larger in magnitude than any value that can be represented by the
number of elements in the SIGNED or UNSIGNED value.
G.3.4.3 Return value for metalogical and high-impedance operands
If an operand to any of the NUMERIC_STD or NUMERIC_STD_UNSIGNED relational operator functions
for =, <, <=, >, or >= contains a metalogical or high-impedance value, the function returns the value FALSE.
Expression
Predefined operation on…
Package operation on…
BIT_VECTOR
UNSIGNED
SIGNED
"001" = "00001"
FALSE
TRUE
TRUE
"001" > "00001"
TRUE
FALSE
FALSE
"100" < "01000"
FALSE
TRUE
TRUE
"010" < "10000"
TRUE
TRUE
FALSE
"100" < "00100"
FALSE
FALSE
TRUE
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
568
Copyright © 2019 IEEE. All rights reserved.
If an operand to the NUMERIC_STD or NUMERIC_STD_UNSIGNED relational operator function /=
contains a metalogical or high-impedance value, the function returns the value TRUE.
G.3.5 Shift functions
G.3.5.1 Multiplication by a power of 2 with remainder
The SHIFT_LEFT function for an UNSIGNED parameter provides for multiplication by a power of
## 2 remaindered by the maximum size of the vector parameter. In particular, if ARG is UNSIGNED and

contains neither metalogical or high-impedance values, and if the integer values fall within the range
allowed for INTEGERs:
TO_INTEGER (SHIFT_LEFT (ARG, COUNT)) =
   TO_INTEGER (ARG) * (2 ** COUNT) rem (2 ** ARG'LENGTH)
G.3.5.2 Division by a power of 2
The SHIFT_RIGHT function for an UNSIGNED parameter provides for division by a power of 2. That is, if
ARG is UNSIGNED and contains neither metalogical or high-impedance values, and if the integer values
fall with the range allowed for INTEGERs:
TO_INTEGER (SHIFT_RIGHT (ARG, COUNT) = TO_INTEGER (ARG) / (2 ** COUNT)
G.3.6 Type conversion functions
G.3.6.1 Overflow in conversion to INTEGER
The TO_INTEGER function does not contain code to check that the SIGNED or UNSIGNED parameter has
an equivalent universal integer value that belongs to the range defined for the INTEGER or NATURAL
subtypes. If TO_INTEGER is called with a parameter value that is too large, the simulation tool may
therefore detect an overflow. A user should avoid applying TO_INTEGER to parameter subtypes for which
the number of elements is greater than the number of bits used to represent INTEGERs in the user’s
simulation and synthesis tools.
G.3.6.2 Conversion between SIGNED and UNSIGNED
The packages do not provide functions for converting directly between the SIGNED and UNSIGNED types.
Such conversions must be performed by the user. There are several ways to convert between SIGNED and
UNSIGNED types. In performing such conversions, a user must determine how to handle any possible
differences in the ranges supported by SIGNED and UNSIGNED objects having the same number of
elements. For example, suppose the VHDL source code contains the declarations
signal S: SIGNED(3 downto 0);
signal BIG_S: SIGNED(4 downto 0);
signal U: UNSIGNED(3 downto 0);
constant S1: SIGNED(3 downto 0) := "1000"; -- equivalent to -8
constant U1: UNSIGNED(3 downto 0) := "1100"; -- equivalent to +12
a)
A user can use a VHDL type conversion to convert one form to another:
S <= SIGNED (U1);   -- U1 (= +12) gets converted to S (= -4)
U <= UNSIGNED (S1); -- S1 (= -8)  gets converted to U (= +8)
b)
A user can add an extra bit to represent the sign when converting from UNSIGNED to SIGNED:
BIG_S <= SIGNED ('0' & U1); -- U1 (= +12) gets converted
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
569
Copyright © 2019 IEEE. All rights reserved.
                            -- to BIG_S (= +12)
c)
Finally, a user can generate an error or warning when the value of one cannot be represented in the
number of elements available in the other:
assert S >= "0000"
   report "Cannot convert negative value."
   severity WARNING;
U <= UNSIGNED (S);
G.3.7 Logical operator functions
G.3.7.1 Application to SIGNED and UNSIGNED
The functions that define the application of the logical operators and, or, nand, nor, xor, and xnor to
SIGNED and UNSIGNED operand values are equivalent to functions that apply the same logical operators
to STD_LOGIC_VECTOR (or STD_ULOGIC_VECTOR) parameters. This equivalence includes the
handling of metalogical and high-impedance element values. That is, for example, if S1 and S2 are SIGNED
values of equal length:
S1 nand S2 = SIGNED (STD_LOGIC_VECTOR (S1) nand STD_LOGIC_VECTOR (S2))
G.3.7.2 Index range of return values
For the functions and, or, nand, nor, xor, and xnor defined in the NUMERIC_STD package, the index
range for the return values has the form “n – 1 downto 0,” where n is the number of elements in the return
value.
In the NUMERIC_BIT package, the corresponding functions are defined implicitly by the type declarations
for the SIGNED and UNSIGNED types, so that the index range of the return values is as defined by this
standard (see 9.2.2).
G.3.8 The STD_MATCH function
The behavior of the STD_MATCH functions in the NUMERIC_STD package differs from that of the
= functions for the same types of parameters. The STD MATCH function compares its parameters element
by element, and treats the value '–' as matching any other STD_ULOGIC value. The = function interprets its
operands, however, as representing the equivalent integer values, and returns TRUE if the equivalent integer
values are equal.
G.4 Using the fixed-point package
G.4.1 General
Fixed point is a step between integer math and floating point. This has the advantage of being almost as fast
as NUMERIC_STD arithmetic, but able to represent numbers that are less than 1.0. A fixed-point number
has an assigned width and an assigned location for the binary point. As long as the number is big enough to
provide enough precision, fixed point is fine for most digital signal processing (DSP) applications. Because
it is based on integer math, it is extremely efficient, as long as the data does not vary too much in magnitude.
The fixed-point package defines two types: “unresolved_ufixed” is the unsigned fixed point, and
“unresolved_sfixed” is the signed fixed point.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
570
Copyright © 2019 IEEE. All rights reserved.
type unresolved_ufixed is array (INTEGER range <>) of STD_ULOGIC;
type unresolved_sfixed is array (INTEGER range <>) of STD_ULOGIC;
There are also aliases of these types, “U_ufixed” and “U_sfixed”. The package defines subtypes, “ufixed”
and “sfixed”, with resolved elements:
subtype ufixed is (resolved) unresolved_ufixed;
subtype sfixed is (resolved) unresolved_sfixed;
Example:
   use ieee.fixed_pkg.all;
   ...
   signal a, b: sfixed (7 downto -6);
   signal c: sfixed (8 downto -6);
begin
   ...
   c <= a + b;
The fixed-point data types define the location of the binary point by using negative indices within a
descending index range. The binary point is assumed to be between the 0 and –1 index. Thus, given a
declaration
signal y: ufixed (4 downto -5)
the data type represents unsigned fixed point, 10 bits wide, with 5 bits after the binary point. Then assigning
y = 6.5 in decimal, or = 00110.10000 in binary, can be written:
y <= "0011010000";
The signed data type uses 2s-complement representation, just like the NUMERIC_STD package.
Any non-null index range is valid. Thus:
signal z: ufixed (-2 downto -3);
signal y: sfixed (3 downto 1);
...
z <= "11";  -- 0.011 = 0.375
y <= "111"; -- 1110.0 = -2
G.4.2 Literals and type conversions
Conversion functions have been created for INTEGER, REAL, SIGNED, and UNSIGNED types. These
conversion functions can be called with two different sets of parameters, one set giving the index bounds of
the result directly, and the other consisting of a single parameter whose index bounds are used. For example,
to convert from a real number to a signed fixed-point result:
a <= to_sfixed (-3.125, 7, -6);
b <= to_sfixed (inp1, b); -- returns "inp1" sized the same as "b"
Likewise, to convert from a real number to an unsigned fixed-point result:
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
571
Copyright © 2019 IEEE. All rights reserved.
y <= to_ufixed (6.5, 4, -5);
where 4 is the upper index, and –5 is the lower index; or similarly:
y <= to_ufixed (6.5, y'high, y'low);
or:
y <= to_ufixed (6.5, y);
The to_signed and to_unsigned conversion functions are also overloaded to take the two forms of
parameters specifying the result bounds. Rounding and saturation rules apply on these functions.
G.4.3 Sizing rules
The data widths in the fixed-point package are designed so that there is no possibility of an overflow. This is
a departure from the NUMERIC_STD model, which simply throws away underflow and overflow bits. The
index range of the result of an operation is defined in Table G.2.
Table G.2—Index range of result of an operation
Example:
Given the unsigned declarations:
signal x: ufixed (7 downto -3);
signal y: ufixed (2 downto -9);
Multiplying x by y gives a result of type ufixed (7+2+1 downto –3+(–9)), or ufixed (10 downto –12).
Given the signed declarations:
Operation
Result range
A + B
Max(A'left, B'left) + 1 downto M(A'right, B'right)
A – B
Max(A'left, B'left) + 1 downto M(A'right, B'right)
A * B
A'left + B'left + 1 downto A'right + B'right
A rem B
M(A'left, B'left) downto M(A’right, B'right)
Signed A/B
A'left – B'right + 1 downto A'right – B'left
Signed A mod B
M(A'left, B'left) downto M(A'right, B'right)
Signed reciprocal(A)
–A'right downto –A'left – 1
abs A
A'left + 1 downto A'right
– A
A'left + 1 downto A'right
Unsigned A/B
A'left – B'right downto A'right – B'left – 1
Unsigned A mod B
B'left downto M(A'right, B'right)
Unsigned reciprocal(A)
–A'right + 1 downto –A'left
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
572
Copyright © 2019 IEEE. All rights reserved.
signal x: sfixed (-1 downto -3);
signal y: sfixed (3 downto 1);
Dividing x by y gives a result of type sfixed (–1–1+1 downto –3–3), or sfixed (–1 downto –6).
It is not necessary to memorize the size rules. Instead, the resize function can be used, or the functions
ufixed_high, ufixed_low, sfixed_high, and sfixed_low can be used to return the bounds of an operand.
Example:
   variable a: sfixed (5 downto -3);
   variable b: sfixed (7 downto -9);
   variable adivb: sfixed (sfixed_high (5, -3, '/', 7, -9)
                           downto sfixed_low (5, -3, '/', 7, -9));
begin
   adivb <= a / b; -- signed fixed-point divide
Alternatively:
signal adivb:
         sfixed (sfixed_high (a'high, a'low, '/', b'high, b'low)
                 downto sfixed_low (a'high, a'low, '/', b'high, b'low));
or:
signal adivb: sfixed(sfixed_high (a, '/', b)
                     downto sfixed_low (a, '/', b));
The resize function can be used to fix the size of the output. However, rounding and saturate rules are
applied:
x <= resize (x * y, x'high, x'low);
The increase in result size can cause problems in some designs, such as an accumulator, that is, a
fixed-width number to which other numbers are added repeatedly. To implement an accumulator in the
fixed-point packages, the resize function can be applied to the result of the addition, or the add_carry
procedure can be used, as follows:
signal ACC: ufixed (7 downto -3);
...
add_carry ( L => ACC, R => X, C_in => '0', Result => ACC, C_out =>
open);
The divide function is defined as follows:
function divide ( l, r: sfixed;
round_style: BOOLEAN := fixed_round_style;
guard_bits: NATURAL := fixed_guard_bits)
return sfixed;
The output is sized with the same rules as the / operator. The function allows the number of guard bits and
the rounding operation to be overridden. Note that the output size is calculated so that overflow is not
possible.
The reciprocal function is defined in a similar manner to the divide function:
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
573
Copyright © 2019 IEEE. All rights reserved.
function reciprocal (arg: ufixed;
                     round_style: BOOLEAN := fixed_round_style;
                     guard_bits: NATURAL := fixed_guard_bits)
                     return ufixed;
This function performs the operation “1/arg”, with the output vector following the sizing rules as previously
noted. The function is very useful for dividing by a constant. For example:
A := B / Cons;
can be rewritten as:
A := B * reciprocal (Cons);
because a multiplier typically uses less logic than a divider, this change can save significant hardware
resources.
G.4.4 Rounding and saturation
Many of the fixed-point operations include parameters to control rounding and saturation behavior. An
example is the resize operation, which may be called as follows:
X <= resize (arg => X + 1,
             left_index => X'high, right_index => X'low,
             overflow_style => fixed_wrap,
             round_style => fixed_truncate );
In the FIXED_PKG package, round_style defaults to fixed_round, which turns on the rounding routines. If
round_style is fixed_truncate, the number is truncated. Rounding returns the representable value that is
nearest the original value before dropping the remainder. If the remainder places the original value exactly
in the middle of two representable values, the one with its least significant bit 0 is returned. The rounding
operation is implemented by examining the least significant bit of the unrounded value and the bits of the
remainder. If the most significant bit of the remainder is 1, and either the least significant bit of the
unrounded value is 1 or any bits other than the most significant of the remainder (or both), then the
unrounded value is rounded up; otherwise it is returned as is. While this has the advantage of maintaining
accuracy, like floating-point round-nearest behavior, it has the disadvantage that all of the bits of the
remainder must be examined to do rounding, increasing the hardware complexity.
In the FIXED_PKG package, overflow_style defaults to fixed_saturate: if the true result is too large to
represent, the returned result is the maximum possible number. The alternative for overflow_style is
fixed_wrap, where the top bits are simply truncated. Unlike in NUMERIC_STD, the sign bit is not
preserved when wrapping. Thus, it is possible to get a positive result when resizing a negative number in this
mode.
Finally, a guard_bits parameter on many operations defaults to the value of fixed_guard_bits, which is 3 in
FIXED_PKG. Guard bits are used in the rounding routines. If guard_bits is 0, rounding is turned off.
Otherwise, the extra bits are added to the end of the numbers in the division and to_real functions to make
the numbers more accurate.
G.4.5 Overloading
The following operations are defined for ufixed:
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
574
Copyright © 2019 IEEE. All rights reserved.
+, –, *, /, rem, mod, =, /=, <, >, >=, <=, sll, srl, rol, ror, sla, sra
The following operations are defined for sfixed:
+, –, *, /, rem, mod, =, /=, <, >, >=, <=, sll, srl, rol, ror, sla, sra, abs, – (unary)
All of the binary operators are overloaded for REAL and INTEGER data types. In the case of a REAL, the
range of the fixed-point number is used to convert the real number into fixed point before the operation is
performed. In the case of an INTEGER, the number is converted into fixed point with the range of
fixed'HIGH downto 0. Thus, the fixed-point operand must be of a format large enough to accommodate the
converted input or a “vector-truncated” warning is produced. In these functions, overflow_style is set to
fixed_saturate.
The overloaded definitions allow, as an example:
signal x: sfixed (4 downto -5);
signal y: real;
...
z := x + y;
In the case where an operation is performed that includes both a fixed-point number and an integer or real,
the sizing rules are modified. For a real number, the real is converted to a fixed-point number that is the
same size as the fixed-point argument. Thus, the preceding example is equivalent to:
z := x + sfixed(y, 4, -5);
result in a type of sfixed (5 downto –5) for z. A similar rule holds for integers.
Shift operators are functionally the same as those for NUMERIC_STD. An arithmetic shift (sra or sla) on an
unsigned number is the same as a logical shift. An arithmetic shift on a signed number is the same as a
logical shift if the number is shifted left, but replicates the sign bit if the number is shifted right.
The scalb function can be used to losslessly multiply or divide any number by a power of two, for example:
   constant half: ufixed (2 downto -2) := "00010"; -- 000.10
   variable two: ufixed (5 downto 0);
   variable someval: ufixed (5 downto -5);
begin
   two := scalb(half, 2); -- returns "00010.", or 2.0
   someval := resize (scalb (half, X), someval'high, someval'low);
All of the standard relational operators are implemented. The operators =, /=, <, >, >=, <= perform in a
similar way to the NUMERIC_STD functions. If values of two different lengths are given, then the inputs
are resized before the comparison is made.
The maximum and minimum functions do a comparison operation and return the appropriate value. These
functions are overloaded for INTEGER and REAL parameters. The sizes of the parameters do not need to
match. The output is resized to the maximum of the left index and minimum of the right index.
The find_leftmost and find_rightmost functions find the leftmost or rightmost occurrence of a given bit
value in a fixed-point number and return the index of the occurrence. The functions are declared as:
function find_leftmost  (arg: ufixed; y: STD_ULOGIC) return INTEGER;
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
575
Copyright © 2019 IEEE. All rights reserved.
function find_rightmost (arg: ufixed; y: STD_ULOGIC) return INTEGER;
and similarly for sfixed parameters. The parameter y can be any STD_ULOGIC value. The functions use the
"?=” operator to compare bits in arg with y, so strength of values is ignored. If the value is not found by the
find_leftmost function, arg'low – 1 is returned. Similarly, if the value is not found by the find_rightmost
function, arg'high + 1 is returned. Note that find_leftmost (arg, '1') for a ufixed parameter or for a positive
sfixed parameter returns the integer log (base 2) of arg.
The To_01, To_X01, To_X01Z, To_UX01, and Is_X functions are similar to the STD_LOGIC_1164 and
NUMERIC_STD functions of the same names.
Most synthesis tools do not support any I/O format other than std_logic_vector and std_logic. Thus,
functions are included to convert between std_logic_vector and ufixed or sfixed, and vice versa, for
example:
uf7_3 <= to_ufixed (slv7, uf7_3'high, uf7_3'low);
and
slv7 <= to_slv (uf7_3);
READ, WRITE, HREAD, HWRITE, OREAD, and OWRITE routines are also defined for fixed-point data
types. A “.” separator is added between the integer part and the fractional part of the fixed-point number.
Therefore the result of to_ufixed (6.5, 4, –5) would be written as "00110.10000". This string can also be read
back into a variable of type ufixed(4 downto –5).
The functions to_string, to_ostring, and to_hstring are also provided. These are very useful in assertion and
report statements, for example:
assert x = y
   report to_string(x) & " /= " & to_string(y)
   severity error;
Alternatively, the numbers can be shown in real format:
assert x = y
   report to_string(to_real(x)) & " /= " & to_string(to_real(y))
   severity error;
In order to provide a measure of compatibility with tools commonly used to define DSP algorithms, the
package provides the To_SFix and To_UFix conversion functions. These functions convert from a
STD_LOGIC_VECTOR value to a ufixed or sfixed value, respectively. The index bounds for the result are
described in terms of the vector length and the number of post-binary-point bits. For example, a DSP tool
might describe an unsigned fixed-point number as ufix[14,10], which specifies a 14-bit word with a 10-bit
fraction. This translates to the unsigned fixed-point type ufixed(3 downto –10). Similarly, sfix[14, 10]
translates to the signed fixed-point type sfixed(3 downto –10).
G.4.6 Package generics
The fixed-point packages are defined by an uninstantiated package with generic constants, as follows:
library IEEE; ...
use IEEE.fixed_float_types.all;
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
576
Copyright © 2019 IEEE. All rights reserved.
package fixed_generic_pkg is
   generic (
      fixed_round_style    : fixed_round_style_type
         := fixed_round;
      fixed_overflow_style : fixed_overflow_style_type
         := fixed_saturate;
      fixed_guard_bits     : NATURAL := 3;
      no_warning           : BOOLEAN := FALSE
      );
   ...
Since it is an uninstantiated package, fixed_generic_pkg cannot be used directly. Rather, it must be
instantiated and the instance used. The library IEEE contains a standard instance, named fixed_pkg,
declared as:
library IEEE;
package fixed_pkg is new IEEE.fixed_generic_pkg
   generic map (
      fixed_round_style    => IEEE.fixed_float_types.fixed_round,
      fixed_overflow_style => IEEE.fixed_float_types.fixed_saturate,
      fixed_guard_bits     => 3,
      no_warning           => FALSE
      );
This is where the actual generics are specified. Note that the user can declare a separate instantiation of the
fixed-point package if different defaults are required. For example, if an application does not require
rounding (because it takes up too much logic), requires wrapping of numbers rather than saturation, requires
no guard bits on divisions, and does not require “metavalue detected” warnings, the package may be
instantiated as follows:
library IEEE;
package my_fixed_pkg is new IEEE.fixed_generic_pkg
   generic map (
      fixed_round_style    => IEEE.fixed_float_types.fixed_truncate,
      fixed_overflow_style => IEEE.fixed_float_types.fixed_wrap,
      fixed_guard_bits     => 0,
      no_warning           => TRUE
      );
This package instance can be analyzed and used in other design units. Note that the ufixed and sfixed types
declared in the different package instances are distinct types, so type conversions may be needed to translate
between them, as shown in the following example:
library IEEE; use IEEE.std_logic_1164.all, IEEE.fixed_pkg.all;
entity sin is
   port (arg: in ufixed (1 downto -16);
         clk, rst: in STD_ULOGIC;
         res: out ufixed (1 downto -11));
end entity sin;
architecture structure of sin is
   component fixed_sin is
      port (arg: in work.my_fixed_pkg.ufixed (1 downto -16);
            clk, rst: in STD_ULOGIC;
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
577
Copyright © 2019 IEEE. All rights reserved.
            res: out work.my_fixed_pkg.ufixed (1 downto -11));
   end component fixed_sin;
begin
   U1: component fixed_sin
      port map (arg => work.my_fixed_pkg.ufixed(arg), -- convert arg
                clk => clk, rst => rst,
                IEEE.fixed_pkg.ufixed (res) => res);
end architecture structure;
G.4.7 Issues
The fixed-point math packages are based on the NUMERIC_STD package and use signed and unsigned
arithmetic from within that package. This makes them highly efficient because the NUMERIC_STD
package is well supported by simulation and synthesis tools.
An ascending index range is treated as an error by the fixed-point routines. Thus, if a number is declared as
ufixed(–1 to 5), an error will occur when the number is operated upon.
String literals also cause problems. For example, in the following:
z <= a + "011011";
the index range of the string literal is defined by VHDL rules to be INTEGER'left to INTEGER'left + 5.
Infeasible index values such as these also cause errors to occur.
Care is required in cases such as the following:
   signal a: sfixed (3 downto -3);
   signal b: sfixed (2 downto -4);
begin
   b <= a;
In this example, the two vectors have the same length, and so the assignment is legal. However, the change
in index range implies a shift in the position of the binary point, thus changing the value represented. For
example, if a represents the value 6.5, after the assignment, b represents the value 3.25. Such direct
assignments are only correct if the index ranges are the same. Otherwise, the resize function should be used.
G.4.8 Catalog of operations
G.4.8.1 Operators
"+"
Adds two fixed-point numbers together, overloaded for REAL and INTEGER. See output sizing rules (see
G.4.3).
"–"
Subtracts fixed-point numbers. Overloaded for REAL and INTEGER. See output sizing rules (see G.4.3).
Unary version (–var1) returns a value that is one bit larger than the input. Note that unary – is only
implemented on objects of type sfixed.
"*"
Multiply two fixed-point numbers together. Overloaded for REAL and INTEGER. See output sizing rules
(G.4.3).
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
578
Copyright © 2019 IEEE. All rights reserved.
"/"
Divides two fixed-point numbers. Overloaded for REAL and INTEGER. See output sizing rules (see G.4.3).
Uses 3 guard bits and rounds the result by default. If this is not the desired functionality, then use the divide
function or modify the package generics.
"abs"
Absolute value. Returns a result one bit larger than the input. The argument and result are both of type sfixed.
"mod"
Modulo. Returns the signed remainder. See output sizing rules (see G.4.3). Overloaded for REAL and
INTEGER.
"rem"
Remainder. Returns the unsigned remainder. See output sizing rules (see G.4.3). Overloaded for REAL and
INTEGER.
"sll"
Shift left logical. Left argument is ufixed or sfixed, right argument is INTEGER. A negative right argument
causes a logical right shift.
"srl"
Shift right logical. Left argument is ufixed or sfixed, right argument is INTEGER. A negative right argument
causes a logical left shift.
"rol"
Rotate logical left. Left argument is ufixed or sfixed, right argument is INTEGER. A negative right argument
causes a rotate right.
"ror"
Rotate logical right. Left argument is ufixed or sfixed, right argument is INTEGER. A negative right argument
causes a rotate left.
"sla"
Shift left arithmetic. Left argument is ufixed or sfixed, right argument is INTEGER. A negative right argument
causes right arithmetic shift. Note that a right arithmetic shift on an sfixed replicates the sign bit. A left shift
does not replicate the least significant bit. Note also that “x sla int” will multiply (or divide) x by a power of 2.
"sra"
Shift right arithmetic. Left argument is ufixed or sfixed, right argument is INTEGER. A negative right
argument causes left arithmetic shift. Note that a right arithmetic shift on an sfixed replicates the sign bit. A
left shift does not replicate the least significant bit. Note that “x sra int” will divide (or multiply) x by a power
of 2.
"="
Equal. Overloaded for REAL and INTEGER. Returns FALSE if any 'X' is found. Integers are converted to
fixed point with to_fixed (arg, max(a'high+1, 0), 0), reals are converted with to_fixed (arg, a'high+1, a'low)
and rounded.
"/="
Not equal. Overloaded for REAL and INTEGER. Returns TRUE if any 'X' is found. Integers are converted to
fixed point with to_fixed (arg, max(a'high+1, 0), 0), reals are converted with to_fixed (arg, a'high+1, a'low)
and rounded.
"<"
Less than. Overloaded for REAL and INTEGER. Returns FALSE if any 'X' is found. Integers are converted to
fixed point with to_fixed (arg, max(a'high+1, 0), 0), reals are converted with to_fixed (arg, a'high+1, a'low)
and rounded.
">"
Greater than. Overloaded for REAL and INTEGER. Returns FALSE if any 'X' is found. Integers are converted
to fixed point with to_fixed (arg, max(a'high+1, 0), 0), reals are converted with to_fixed (arg, a'high+1, a'low)
and rounded.
"<="
Less than or equal. Overloaded for REAL and INTEGER. Returns FALSE if any 'X' is found. Integers are
converted to fixed point with to_fixed (arg, max(a'high+1, 0), 0), reals are converted with to_fixed (arg,
a'high+1, a'low) and rounded.
">="
Greater than or equal. Overloaded for REAL and INTEGER. Returns FALSE if any 'X' is found. Integers are
converted to fixed point with to_fixed (arg, max(a'high+1, 0), 0), reals are converted with to_fixed (arg,
a'high+1, a'low) and rounded.
"?="
Performs an operation similar to the NUMERIC_STD "?=" function, but returns a STD_ULOGIC value.
"?/="
Performs an operation similar to the NUMERIC_STD "?/=" function, but returns a STD_ULOGIC value.
"?<"
Returns 'X' if a metavalue is in either number, '1' if L is less than R, otherwise '0'.
"?<="
Returns 'X' if a metavalue is in either number, '1' if L is less than or equal to R, otherwise '0'.
"?>"
Returns 'X' if a metavalue is in either number, '1' if L is greater than R, otherwise '0'.
"?>="
Returns 'X' if a metavalue is in either number, '1' if L is greater than or equal to R, otherwise '0'.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
579
Copyright © 2019 IEEE. All rights reserved.
G.4.8.2 Functions
"and"
Logical and. Similar to the STD_LOGIC_1164 operators. Binary operators require operands to have the same
index ranges. Index range of the result is the same as those of the operands.
"nand"
Logical nand. Similar to the STD_LOGIC_1164 operators. Binary operators require operands to have the same
index ranges. Index range of the result is the same as those of the operands.
"or"
Logical or. Similar to the STD_LOGIC_1164 operators. Binary operators require operands to have the same
index ranges. Index range of the result is the same as those of the operands.
"nor"
Logical nor. Similar to the STD_LOGIC_1164 operators. Binary operators require operands to have the same
index ranges. Index range of the result is the same as those of the operands.
"xor"
Logical exclusive or. Similar to the STD_LOGIC_1164 operators. Binary operators require operands to have
the same index ranges. Index range of the result is the same as those of the operands.
"xnor"
Logical exclusive nor. Similar to the STD_LOGIC_1164 operators. Binary operators require operands to have
the same index ranges. Index range of the result is the same as those of the operands.
"not"
Logical not. Similar to the STD_LOGIC_1164 operator. Index range of the result is the same as that of the
operand.
find_leftmost
Find leftmost occurrence of a given bit value. Inputs: arg (ufixed or sfixed), y : std_ulogic. Returns
the integer index of the first occurrence of y in the vector arg starting from the left. Arg'low–1 is
returned if y is not found. Note that find_leftmost(arg, '1') for a ufixed parameter or for a positive
sfixed parameter returns the integer log base 2 of the input arg.
find_rightmost
Find rightmost occurrence of a given bit value. Inputs: arg (ufixed or sfixed), y : std_ulogic. Returns
the integer index of the first occurrence of y in the vector arg starting from the right. Arg'high+1 is
returned if y is not found.
divide
Arithmetic divide. Functionally identical to the "/" operator, but with two extra parameters. Inputs:
l, r (both ufixed or sfixed), parameters: guard_bits : NATURAL, round_style :
fixed_round_style_type. See output sizing rules (see G.4.3). Guard bits are extra bits that are added
to the end of the divide routine to maintain precision when rounding. The round style is either
fixed_round or fixed_truncate. If rounding is set to fixed_truncate, then the guard bits are ignored.
reciprocal
Performs a 1/arg function. Inputs: arg (ufixed or sfixed), guard_bits : NATURAL, round_style :
fixed_round_style_type. See output sizing rules (see G.4.3). Guard bits are extra bits that are added
to the end of the divide routine to maintain precision when rounding. The round style is either
fixed_round or fixed_truncate. If rounding is set to fixed_truncate, then the guard bits are ignored.
remainder
Arithmetic remainder. Inputs: l, r (both ufixed or sfixed), parameters: guard_bits : NATURAL,
round_style : fixed_round_style_type. See output sizing rules (see G.4.3). Guard bits are extra bits
that are added to the end of the remainder routine to maintain precision when rounding. The round
style is either fixed_round or fixed_truncate. If rounding is set to fixed_truncate, then the guard bits
are ignored.
modulo
Arithmetic modulo. Inputs: l, r (both ufixed or sfixed), parameters: guard_bits : NATURAL,
round_style : fixed_round_style_type. See output sizing rules (see G.4.3). Guard bits are extra bits
that are added to the end of the remainder routine to maintain precision when rounding. The round
style is either fixed_round or fixed_truncate. If rounding is set to fixed_truncate, then the guard bits
are ignored.
minimum
Returns the minimum of the two input parameters (both either ufixed or sfixed) by performing
a ">" operation.
maximum
Returns the maximum of the two input parameters (both either ufixed or sfixed) by performing
a ">" operation.
std_match
Performs a NUMERIC_STD.STD_MATCH function (allows use of '–' values for the inputs).
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
580
Copyright © 2019 IEEE. All rights reserved.
G.4.8.3 Conversion functions
add_carry
This procedure which takes  two parameters (L and R) as well as a carry in (C_in). It has output
parameters for a carry out (C_OUT) and a result of the same length as the combined width of L
and R. Note that this routine can be used as an accumulator.
scalb
Inputs are of type ufixed or sfixed, with an INTEGER or signed input. The Scalb function moves the
index of the fixed-point number, having the effect of multiplying or dividing by a power of two.
Resize
Changes the size of a ufixed or sfixed (larger or smaller). Inputs: arg (ufixed or sfixed);
left_index and right_index (INTEGER), or size_res (same type as arg). Other parameters:
round_style, saturate_style. Output: resized ufixed or sfixed.
To_ufixed
Converts to the ufixed type.
To_ufixed (std_ulogic_vector)
Inputs: arg (std_ulogic_vector); left_index and
right_index (INTEGER), or size_res (ufixed).
This function converts a std_ulogic_vector to a
ufixed with the same width. A warning is
produced if the width is incorrect.
To_ufixed (unsigned)
Inputs: arg (unsigned); left_index and right_index
(INTEGER), or size_res (ufixed). Other
parameters: overflow_style, round_style.
Converts an unsigned to a ufixed of the same size
with the left_index being arg'length–1 and the
right_index being 0.
To_ufixed (REAL)
Inputs: arg (REAL); left_index and right_index
(INTEGER), or size_res (ufixed). Other
parameters: overflow_style, round_style.
Converts a REAL to a ufixed. If the input is
negative, then an error occurs and 0 is returned.
To_ufixed (INTEGER)
Inputs: arg (NATURAL); left_index and
right_index (INTEGER), or size_res (ufixed).
Other parameters: overflow_style, round_style.
Converts an INTEGER to a ufixed.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
581
Copyright © 2019 IEEE. All rights reserved.
To_sfixed
Converts to the sfixed type.
To_sfixed (std_ulogic_vector)
Inputs: arg (std_ulogic_vector); left_index and
right_index (INTEGER), or size_res (sfixed).
This function converts a std_ulogic_vector to an
sfixed with the same width. A warning is
produced if the width is incorrect.
To_sfixed (signed)
Inputs: arg (signed); left_index and right_index
(INTEGER), or size_res (sfixed). Other
parameters: overflow_style, round_style.
Converts a signed to an sfixed of the same size
with the left_index being arg'length–1 and the
right_index being 0.
To_sfixed (REAL)
Inputs: arg (REAL); left_index and right_index
(INTEGER), or size_res (sfixed). Other
parameters: overflow_style, round_style.
Converts a REAL to an sfixed.
To_sfixed (INTEGER)
Inputs: arg (INTEGER); left_index and
right_index (INTEGER), or size_res (sfixed).
Other parameters: overflow_style, round_style.
Converts a INTEGER to an sfixed.
To_sfixed (ufixed)
Inputs: arg (ufixed). Converts a ufixed into an
sfixed by adding a sign bit.
To_unsigned
Inputs: arg (ufixed); and size (NATURAL), or size_res (unsigned). Other parameters:
round_style, saturate_style. Converts a ufixed to an unsigned. This does not produce a
“vector truncated” warning as the NUMERIC_STD functions do.
To_signed
Inputs: arg (sfixed); and size (NATURAL), or size_res (signed). Other parameters:
round_style, saturate_style. Converts an sfixed to a signed. This does not produce a
“vector truncated” warning as the NUMERIC_STD functions do.
To_real
Inputs: arg (ufixed or sfixed). Converts a fixed-point number to a real number.
To_integer
Inputs: arg (ufixed or sfixed). Other parameters: round_style, saturate_style. Converts a
fixed-point number to an integer.
To_slv
Inputs: arg (ufixed or sfixed). Converts a fixed-point number to a std_logic_vector of the
same length.
To_std_logic_vector
Alias of to_slv.
To_stdlogicvector
Alias of to_slv.
To_sulv
Inputs: arg (ufixed or sfixed). Converts a fixed-point number to a std_ulogic_vector of the
same length.
To_std_ulogic_vector
Alias of to_sulv.
To_stdulogicvector
Alias of to_sulv.
To_01
Inputs s (ufixed or sfixed). Other parameters: XMAP: std_ulogic. Converts metavalues in
the vector S to the XMAP state (defaults to 0).
Is_X
Inputs arg (ufixed or sfixed). Returns a BOOLEAN that is TRUE if there are any
metavalues in the vector arg.
To_x01
Inputs arg (ufixed or sfixed). Converts any metavalues found in the vector arg to be 'X',
'0', or '1'.
To_ux01
Inputs arg (ufixed or sfixed). Converts any metavalues found in the vector arg to be 'U',
'X' , '0', or 1'.
To_x01z
Inputs arg (ufixed or sfixed). Converts any metavalues found in the vector arg to be 'Z',
'X' , '0', or '1'.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
582
Copyright © 2019 IEEE. All rights reserved.
G.4.8.4 Sizing functions
Each of these functions take as a parameter a character that describes the operation to be performed, as
shown in the Table G.3:
Table G.3—Operations described by characters
Character
Operation
'+'
"+"
'–'
"–"
'*'
"*"
'/'
"/", divide
'1'
reciprocal
'M', 'm'
"mod", modulo
'R', 'r'
"rem", remainder
'A', 'a'
"abs"
'N', 'n'
unary "–"
others
index
Ufixed_high
Inputs: left_index, right_index: INTEGER (bounds of the left argument) or size_res: ufixed;
operation: character; left_index2, right_index2: INTEGER (bounds of the left argument) or
size_res2: ufixed. This function is used to compute the high index bound of the result of an unsigned
operation. Any values for the operation character other than those defined in Table G.3 cause the
left_index to be returned.
Ufixed_low
Inputs: left_index, right_index: INTEGER (bounds of the left argument) or size_res: ufixed;
operation: character; left_index2, right_index2: INTEGER (bounds of the left argument) or
size_res2: ufixed. This function is used to compute the low index bound of the result of an unsigned
operation. Any values for the operation character other than those defined in Table G.3 cause the
left_index to be returned.
Sfixed_high
Inputs: left_index, right_index: INTEGER (bounds of the left argument) or size_res: ufixed;
operation: character; left_index2, right_index2: INTEGER (bounds of the left argument) or
size_res2: ufixed. This function is used to compute the high index bound of the result of a signed
operation. Any values for the operation character other than those defined in Table G.3 cause the
left_index to be returned.
Sfixed_low
Inputs: left_index, right_index: INTEGER (bounds of the left argument) or size_res: ufixed;
operation: character; left_index2, right_index2: INTEGER (bounds of the left argument) or
size_res2: ufixed. This function is used to compute the low index bound of the result of a signed
operation. Any values for the operation character other than those defined in Table G.3 cause the
left_index to be returned.
To_ufix
Similar to to_ufixed, but with NATURAL arguments representing the length of the result and the
number of post-binary-point bits. Thus, for example, to_ufix ("00100", 5, 3) = 00.100, or 0.5.
To_sfix
Similar to to_sfixed, but with NATURAL arguments representing the length of the result and the
number of post-binary-point bits. The sign bit is assumed to take an additional place beyond the
specified length. Thus, for example, to_sfix("00100", 4, 3) = 00.100 or 0.5.
Ufix_high
Similar to ufixed_high, but with NATURAL arguments representing the length of the result and the
number of post-binary-point bits.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
583
Copyright © 2019 IEEE. All rights reserved.
G.4.8.5 Textio functions
Ufix_low
Similar to ufixed_low, but with NATURAL arguments representing the length of the result and the
number of post-binary-point bits.
Sfix_high
Similar to sfixed_high, but with NATURAL arguments representing the length of the result and the
number of post-binary-point bits.
Sfix_low
Similar to sfixed_low, but with NATURAL arguments representing the length of the result and the
number of post-binary-point bits.
Write
Similar to the TEXTIO write procedure. Automatically inserts a binary point where needed. If
the range of the input number does not include the 0 index, then the number is extended until it
does before writing.
Read
Similar to the TEXTIO read procedure. If a "." is encountered, then it is tested to check that it
is in the correct place.
Bwrite
Alias for write.
Binary_write
Alias for write.
Bread
Alias for read.
Binary_read
Alias for read.
Owrite
Octal write. The pre- and post-binary-point parts of the number are written separately, with a
binary point between them. Each side is padded to a multiple of 3 bits to form an octal digit.
Oread
Octal read. The number read is interpreted as separate pre- and post-binary-point parts, with an
optional binary point between them. If a "." is encountered, then it is tested to check that it is in
the correct place.
Octal_write
Alias for owrite.
Octal_read
Alias oread.
Hwrite
Hex write. The pre- and post-binary-point parts of the number are written separately, with a
binary point between them. Each side is padded to a multiple of 4 bits to form a hex digit.
Hread
Hex read. The number read is interpreted as separate pre- and post-binary-point parts, with an
optional binary point between them. If a "." is encountered, then it is tested to check that it is in
the correct place.
Hex_write
Alias for hwrite.
Hex_read
Alias for hread.
To_string
Returns a string that can be padded and left or right justified, for example:
assert a = 1.5 report "Result was " & to_string (a) severity
error;
To_bstring
Alias for to_string.
To_binary_string
Alias for to_string.
To_ostring
Similar to to_string, but returns an octal value with a binary point. The padding rules of the
owrite procedure apply to this function.
To_octal_string
Alias for to_ostring.
To_hstring
Similar to to_string, but returns a hex value with a binary point. The padding rules of the hwrite
procedure apply to this function.
To_hex_string
Alias for to_hstring.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
584
Copyright © 2019 IEEE. All rights reserved.
G.5 Using the floating-point package
While floating-point numbers are widely used in software applications, they are less common in custom
hardware. This is because floating-point takes up almost three times the hardware resources of fixed-point
math. The advantage of floating-point, however, is that relative precision is maintained across a wide
dynamic range, whereas fixed-point numbers are limited to a smaller dynamic range with fixed absolute
precision.
G.5.1  Floating-point numbers
Floating-point numbers are well defined by IEEE 754 specifications. Floating point has been used in
processors and intellectual property (IP) for years, and is a well-understood format. The format is a sign
magnitude system, where the sign is processed separately from the magnitude.
There are many concepts in floating-point that make it different from common signed and unsigned number
notations. To illustrate, consider a 32-bit floating-point number:
S   EEEEEEEE FFFFFFFFFFFFFFFFFFFFFFF
## 31  30    23 22                    0

+/- exp.     fraction
Basically, a floating-point number comprises a sign bit (+ or –), a normalized exponent, and a fraction. To
convert this number back into an integer, the following equation can be used:
S * 2**(exponent – exponent_base) * (1.0 + fraction/fraction_base)
From_string
Translates a string (with a binary point in it) to a fixed-point number. Some examples are:
   signal a: ufixed (3 downto -3);
begin
   a <= from_string ("0000.000", a'high, a'low);
   a <= from_string ("0001.000", a);
   a <= from_string ("0000.100"); -- Works only if
                                  -- size is exact.
Note that this is typically not synthesizable, as it uses the STRING type. A synthesizable
alternative is “a <= "0000000";”.
From_bstring
Alias for from_string.
From_binary_string
Alias for from_string.
From_ostring
Same as from_string, but uses octal numbers. The oread padding rules apply in this function.
From_octal_string
Alias for from_ostring.
From_hstring
Same as from_string, but uses hex numbers. The hread padding rules apply in this function.
From_hex_string
Alias for from_hstring.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
585
Copyright © 2019 IEEE. All rights reserved.
where the “exponent_base” is 2**((maximum exponent/2) – 1), and “fraction_base” is the maximum
possible fraction (unsigned) plus one. For example, using a 32-bit representation:
## 0 10000001 101000000000000000000000

= +1 * 2**(129 – 127) * (1.0 + 5242880/8388608) = +1 * 4.0 * 1.625 = 6.5
There are also “denormal numbers,” which are numbers smaller than can be represented in this way. A
denormal number is indicated with an exponent of 0. In this case, the term 1.0 is not added to the scaled
fraction. For example:
## 1 00000000 100000000000000000000000

= –1 * 2**(– 126) * (4194304/8388608) = –1 * 2**(–126) * 0.5 = –2**(–127)
Next, there are several floating-point “constants”:
## 0 00000000 000000000000000000000000 = +0.0

## 1 00000000 000000000000000000000000 = –0 (which = +0)

## 0 11111111 000000000000000000000000 = positive infinity

## 1 11111111 000000000000000000000000 = negative infinity

A number with an infinite (all ones) exponent and anything other than an all-zero fraction is said to be a
NaN, or “Not a Number.” There are two types of NaN: signaling and non-signaling. The floating-point
package defines a NaN with a fraction whose most significant bit is 1 to be a signaling NAN and any other
NaN to be a quiet NaN.
In summary, a floating-point number falls to one of the following classes (or states):
—
nan: Signaling NaN
—
quiet_nan: Quiet NaN
—
neg_inf: Negative infinity
—
neg_normal: Negative normalized non-zero
—
neg_denormal: Negative denormalized
—
neg_zero: –0.0
—
pos_zero: +0.0
—
pos_denormal: Positive denormalized
—
pos_normal: Positive normalized non-zero
—
pos_inf: Positive infinity
—
isx: at least one input is unknown
These states correspond to enumeration values of the type valid_fpstate defined in the package, and are used
to examine and create numbers needed for floating-point operations. The state isx is included to indicate the
presence of one or more metavalues ('X', 'U', and so on) in a floating-point number. Any arithmetic
operation on such a number will return a number with all bits 'X'.
The package also defines functions that return constant values represented in specified sizes:
—
zerofp: +0.0
—
nanfp: Signaling NaN
—
qnanfp: Quiet NaN
—
pos_inffp: Positive infinity
—
neg_inffp: Negative infinity
—
neg_zerofp: –0.0
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
586
Copyright © 2019 IEEE. All rights reserved.
Rounding can take four different forms:
—
round_nearest: Round to nearest
—
round_inf: Round toward positive infinity
—
round_neginf: Round toward negative infinity
—
round_zero: Round toward zero (truncate)
These forms correspond to enumeration values of the type round_type defined in the package
IEEE.fixed_float_types. Parameters of the type control rounding behavior. In the case of rounding to the
nearest value, if the remainder is exactly ½, the result is rounded so that the least significant bit is 0. The
implementation of this form of rounding requires two comparison operations, but they can be consolidated.
Rounding toward negative infinity rounds down, and rounding toward positive infinity rounds up. Rounding
toward zero simply truncates the remainder, with no actual rounding.
G.5.2 Use model
An example of use of the floating-point package is:
   use IEEE.float_pkg.all;
   ...
   signal x, y, z: float (5 downto -10);
begin
   y <= to_float (3.1415, y); -- Uses y for sizing only
   z <= "0011101010101010";   -- 1/3
   x <= z + y;
The package defines three floating-point types:
—
float32: 32-bit IEEE 754 single precision floating-point
—
float64: 64-bit IEEE 754 double precision floating-point
—
float128: 128-bit IEEE 754 extended precision floating-point
The package also allows specification of a custom floating-point width by constraining the float type, as
shown in the preceding example.
The 32-bit floating-point type is defined as follows:
subtype float32 is float (8 downto –23);
A negative index is used to separate the fraction part of the floating-point number from the exponent. The
top bit ('high) is the sign bit, the next bits ('high–1 downto 0) are the exponent, and the bits with negative
indices (–1 downto 'low) are the fraction. Thus, for a 32-bit representation, the number is represented as
follows:
## 0 00000000 00000000000000000000000

## 8 7      0 -1                  -23

± exp.      fraction
where the sign is bit 8, the exponent is contained in bits 7 down to 0 (8 bits, with bit 7 being the most
significant), and the mantissa is contained in bits –1 down to –23 (32 – 8 – 1 = 23 bits, where bit –1 is the
most significant).
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
587
Copyright © 2019 IEEE. All rights reserved.
The negative index format turns out to be a very natural format for the floating-point number, as the fraction
is always assumed to be a number between 1.0 and 2.0 (unless the number is denormalized). Thus, the
implied “1.0” can be assumed on the positive side of the index, and the negative side represents a fraction of
less than one. The format is similar to that used in the fixed-point package, where everything to the right of
the zero index is assumed to be less than 1.0.
Valid values for float_exponent_width and float_fraction_width are 3 or more. Thus, the smallest
(width-wise) number that can be represented is float (3 downto –3), a 7-bit floating-point number.
The base type defined in the package is unresolved_float (aliased to u_float). The type float is a subtype of
unresolved_float, with resolved elements. The operations defined in the package can be used with either
type interchangeably. The subtypes float32, float64, and float128 are subtypes of float with specified index
ranges. The package also defines subtypes unresolved_float32 (aliased to u_float32), unresolved_float64
(aliased to u_float64), and unresolved_float128 (aliased to u_float128) as subtypes of unresolved_float with
specified index ranges.
Operators for all of the standard math and compare operations are defined in this package. In the float_pkg
package, these operators implement all aspects of IEEE floating-point operations. For most designs, full
IEEE support is not necessary. Thus, functions have been created that allow a design to be parameterized,
for example:
x <= add (l => z, r => y,
          denormalize => FALSE, -- turn off denormal numbers
                                -- (default=TRUE)
          check_error => FALSE, -- turn off NaN and overflow checks
                                -- (default=TRUE)
          round_style => round_zero, -- truncate
                                     -- (default=round_nearest)
          guard_bits => 0);     -- extra bits to maintain precision
                                -- (default=3)
The add function performs just like the + operator; however, it allows the user the flexibility needed for
hardware synthesis. Other similar functions are subtract (–), multiply (*), divide (/), modulo (mod), and
remainder (rem). All of these operators and functions assume that both of the inputs are the same width.
Other functions with similar parameters are reciprocal (1/x) and dividebyp2 (divide by a power of 2). The
abs and unary – operators need no parameters, as they only affect the sign of the floating-point number.
Comparison operators work similarly; however there is only one extra parameter for these functions,
namely, the check_error parameter, which allows NaN and infinity testing to be turned off for the
comparison. These functions are called EQ (=), NE (/=), LT (<), GT (>), GE (>=), and LE (<=).
Conversion functions also work in a similar manner. Functions named to_float are available to convert the
types REAL, INTEGER, signed, unsigned, ufixed, and sfixed. All of these functions take as parameters
either the exponent_width and fraction_width, or a size_res input, which uses the input value for its size
only. The functions to_real, to_integer, to_signed, to_unsigned, to_ufixed, and to_sfixed are also overloaded
in the package with both size and size_res inputs. Further, there is a similar resize function to convert from
one float size to another. Note that, as in the fixed_pkg package, an ascending index range (specified with
to) for a float type is illegal.
The package includes a number of functions recommended by IEEE Std 754-2019. They are described in
G.5.4.4.
Two functions, named break_number and normalize, are also provided. Break_number takes a floating-point
number and returns a SIGNED exponent (biased by –1), a ufixed fixed-point fraction, and a std_ulogic sign.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
588
Copyright © 2019 IEEE. All rights reserved.
Normalize takes a SIGNED exponent, a fixed-point fraction, and a sign and returns a floating-point number.
These functions are useful for operating on the fraction of a floating-point number without having to
perform the shifts on every operation.
To_slv (aliased as to_std_logic_vector and to_StdLogicVector) and to_float are used to convert between
std_logic_vector and floating-point types. These may be used on the interfaces of designs. The result of
to_slv is a std_logic_vector with the length of the input floating-point type.
Procedures for reading and writing floating-point numbers are also included in the package. Procedures
read, write, oread, owrite (octal), bread, bwrite (binary), hread, and hwrite (hex) are defined. To_string,
to_ostring, and to_hstring are also provided for string results. Floating-point numbers are written in a format
such as “0:000:000” (for a 7-bit number). They can be read as a simple string of bits, or with a “.” or “:”
separator.
The following example illustrates use of the package:
library IEEE; use IEEE.std_logic_1164.all;
entity xxx is
   port (a, b: in std_logic_vector (31 downto 0);
         sum: out std_logic_vector (31 downto 0);
         clk, reset: in std_ulogic);
end entity xxx;
use IEEE.float_pkg.all;
architecture RTL of xxx is
   signal afp, bfp, sumfp: float32;
begin
   afp <= to_float (a, afp'high, -afp'low); -- SLV to float, with
bounds
   bfp <= to_float (b, bfp); -- SLV to float, using bfp'range
   addreg : process (clk, reset) is
   begin
      if reset = '1' then
         sumfp <= (others => '0');
      elsif rising_edge (clk) then
         sumfp <= afp + bfp;
         -- this is the same as saying:
         --   sumfp <= add (l => afp, r => bfp,
         --                 round_style => round_nearest,
         --                                  -- best, but most hardware
         --                 guard_bits => 3, -- Use 3 guard bits,
         --                                  -- best for round_nearest
         --                 check_error => TRUE,
         --                                  -- NaN processing turned on
         --                 denormalize => TRUE);
         --                                  -- Turn on denormal numbers
      end if;
   end process addreg;
   sum <= to_slv (sumfp);
end architecture xxx;
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
589
Copyright © 2019 IEEE. All rights reserved.
G.5.3 Package generics
Several aspects of floating-point arithmetic can take up a great deal of hardware. Depending on the
application, not all aspects are needed, so the float_generic_pkg package is designed using generic constants
to allow choice among aspects. The float_generic_pkg cannot be used directly, but must first be instantiated
to provide actual values for the generic constants. The declaration of the float_generic_pkg is:
library IEEE; ...
use IEEE.fixed_float_types.all; ...
package float_generic_pkg is
   generic (
      float_exponent_width : NATURAL    := 8;
      float_fraction_width : NATURAL    := 23;
      float_round_style    : round_type := round_nearest;
      float_denormalize    : BOOLEAN    := TRUE;
      float_check_error    : BOOLEAN    := TRUE;
      float_guard_bits     : NATURAL    := 3;
      no_warning           : BOOLEAN    := FALSE;
      package fixed_pkg is new IEEE.fixed_generic_pkg
                             generic map (<>)
      );
   ...
The generic constants are used as follows:
—
float_exponent_width: Default for conversion routines. For example, the value for a 32-bit
floating-point number would be 8.
—
float_fraction_width: Default for conversion routines. For example, the value for a 32-bit
floating-point number would be 23.
—
float_round_style: Specifies the rounding style to be used, as described in G.5.1.
—
float_denormalize: Activates (TRUE) or deactivates (FALSE) use of denormal numbers.
—
float_check_error: Activates (TRUE) or deactivates (FALSE) NaN and infinity processing. With
processing activated, checks are done at the beginning of every operation. If checks have been done
previously, processing does not need to be repeated for each operation.
—
float_guard_bits: Specifies is the number of extra bits used in each operation to maintain precision. If
the number of guard bits is zero, then rounding is automatically turned off.
—
no_warning: Deactivates (TRUE) or activates (FALSE) “metavalue” warnings.
—
fixed_pkg: The package defining fixed-point types for conversion functions.
There is also a standard instantiation, float_pkg, with actual values for the generics, defined as:
library IEEE;
package float_pkg is new IEEE.float_generic_pkg
   generic map (
      float_exponent_width => 8,
      float_fraction_width => 23,
      float_round_style => IEEE.fixed_float_types.round_nearest,
      float_denormalize => TRUE,
      float_check_error => TRUE,
      float_guard_bits => 3,
      no_warning => FALSE,
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
590
Copyright © 2019 IEEE. All rights reserved.
      fixed_pkg => IEEE.fixed_pkg
      );
Note that the user can declare a separate instantiation of the floating-point package if different actual
generics are required. For example, if an application does not require rounding (because it takes up too much
logic), requires 17-bit floating-point numbers with only 5 bits of exponent, does not require denormal
numbers or NaN and infinity processing, and does not require “metavalue detected” warnings, the package
may be instantiated as follows:
library IEEE;
package my_float_pkg is new IEEE.float_generic_pkg
   generic map (
      float_exponent_width => 5,   -- 5 bits of exponent
      float_fraction_width => 11,  -- Default will be
                                   -- float(5 downto -11)
      float_round_style => IEEE.fixed_float_types.round_zero,
                                   -- Truncate, don't round
      float_denormalize => FALSE,  -- no denormal numbers
      float_guard_bits => 0,       -- Unused by round_zero, set to 0
      float_check_error => FALSE,  -- Turn NaN and overflow off
      no_warning => TRUE,          -- turn warningsoff
      fixed_pkg => WORK.my_fixed_pkg
      );
This package instance can be analyzed and used in other design units. Those design units can include a use
clause such as “use work.my_float_pkg.all;” to make the floating-point function visible. Note that the types
declared in the different package instances are distinct types, so type conversions may be needed to translate
between them, as shown in the following example:
use IEEE.float_pkg.all, IEEE.std_logic_1164.all;
entity sin is
   port (arg: in float (5 downto -11);
         clk, rst: in std_ulogic;
         res: out float (5 downto -11));
end entity sin;
architecture structure of sin is
   component float_sin is
      port (arg: in work.my_float_pkg.float (5 downto -11);
            clk, rst: in std_ulogic;
            res: out work.my_float_pkg.float (5 downto -11));
   end component fixed_sin;
begin
   U1: component float_sin
      port map (arg => work.my_float_pkg.float(arg), -- convert arg
                clk => clk, rst => rst,
                IEEE.float_pkg.float (res) => res);
end architecture structure;
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
591
Copyright © 2019 IEEE. All rights reserved.
G.5.4 Catalog of operations
G.5.4.1 Operators
"+"
Add two floating-point numbers together. Overloaded for REAL and INTEGER. In float_pkg, rounding is set
to round_nearest, 3 guard bits are used, and denormal number and NaN processing are turned on. If this is not
the desired functionality, use the add function. Will accept floating-point numbers of any valid width on either
input.
"–"
Subtracts floating-point numbers. Overloaded for REAL and INTEGER. In float_pkg, rounding is set to
round_nearest, 3 guard bits are used, and denormal number and NaN processing are turned on. If this is not the
desired functionality, use the subtract function. Will accept floating-point numbers of any valid width on either
input.
"*"
Multiply two floating-point numbers together. Overloaded for REAL and INTEGER. In float_pkg, rounding is
set to round_nearest, 3 guard bits are used, and denormal number and NaN processing are turned on. If this is
not the desired functionality, use the multiply function. Will accept floating-point numbers of any valid width
on either input.
"/"
Divides two floating-point numbers. Overloaded for REAL and INTEGER. In float_pkg, rounding is set to
round_nearest, 3 guard bits are used, and denormal number and NaN processing are turned on. If this is not the
desired functionality, use the divide function. Will accept floating-point numbers of any valid width on either
input.
"abs"
Absolute value. Changes only the sign bit.
"–"
Unary minus. Changes only the sign bit.
"mod"
Modulo. Overloaded for REAL and INTEGER. In float_pkg, rounding is set to round_nearest, 3 guard bits are
used, and denormal number and NaN processing are turned on. If this is not the desired functionality, use the
modulo function. Will accept floating-point numbers of any valid width on either input.
"rem"
Remainder. Overloaded for REAL and INTEGER. In float_pkg, rounding is set to round_nearest, 3 guard bits
are used, and denormal number and NaN processing are turned on. If this is not the desired functionality, use
the remainder function. Will accept floating-point numbers of any valid width on either input.
"="
Equal. Overloaded for REAL and INTEGER. In float_pkg, NaN processing is turned on. If this is not the
desired functionality, then use the eq function.
"/="
Not equal. Overloaded for REAL and INTEGER. In float_pkg, NaN processing is turned on. If this is not the
desired functionality, then use the ne function.
"<"
Less than. Overloaded for REAL and INTEGER. In float_pkg, NaN processing is turned on. If this is not the
desired functionality, then use the lt function.
">"
Greater than. Overloaded for REAL and INTEGER. In float_pkg, NaN processing is turned on. If this is not
the desired functionality, then use the gt function.
"<="
Less than or equal to. Overloaded for REAL and INTEGER. In float_pkg, NaN processing is turned on. If this
is not the desired functionality, then use the le function.
">="
Greater than or equal to. Overloaded for REAL and INTEGER. In float_pkg, NaN processing is turned on. If
this is not the desired functionality, then use the ge function.
"?="
Similar to "=", but returns a STD_ULOGIC value.
"?/="
Similar to "/=", but returns a STD_ULOGIC value.
"?<"
Similar to "<", but returns a STD_ULOGIC value.
"?>"
Similar to ">", but returns a STD_ULOGIC value.
"?<="
Similar to "<=", but returns a STD_ULOGIC value.
"?>="
Similar to ">=", but returns a STD_ULOGIC value.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
592
Copyright © 2019 IEEE. All rights reserved.
G.5.4.2 Functions
"and"
Logical and. Similar to the STD_LOGIC_1164 operators.
"nand"
Logical nand. Similar to the STD_LOGIC_1164 operators.
"or"
Logical or. Similar to the STD_LOGIC_1164 operators.
"nor"
Logical nor. Similar to the STD_LOGIC_1164 operators.
"xor"
Logical exclusive or. Similar to the STD_LOGIC_1164 operators.
"xnor"
Logical exclusive nor. Similar to the STD_LOGIC_1164 operators.
"not"
Logical not. Similar to the STD_LOGIC_1164 operator.
add
The add function is similar to the "+" operator; however, it allows the user to vary all of the
parameters.
subtract
The subtract function is similar to the "–" operator; however, it allows the user to vary all of the
parameters.
multiply
The multiply function is similar to the "*" operator; however, it allows the user to vary all of the
parameters.
divide
The divide function is similar to the "/" operator; however, it allows the user to vary all of the
parameters.
remainder
The remainder function is similar to the "rem" operator; however, it allows the user to vary all of the
parameters.
modulo
The modulo function is similar to the "mod" operator; however, it allows the user to vary all of the
parameters.
reciprocal
Returns 1/arg. Inputs: l, r: float; round_style: round_type; guard: NATURAL; check_error:
BOOLEAN; denormalize: BOOLEAN. Works similarly to the divide function.
dividebyp2
Divide by a power of two. Inputs: l, r: float; round_style: round_type; guard: NATURAL;
check_error: BOOLEAN; denormalize: BOOLEAN. Takes the exponent from R and multiplies
L by that amount. Returns an error if R is not a power of 2.
mac
Multiply accumulate. Inputs: l, r, c: float; round_style: round_type; guard: NATURAL; check_error:
BOOLEAN; denormalize: BOOLEAN. Performs the function L*R+C. The addition stage is
integrated into the multiplier stage; thus, this operation takes less logic than separate calls to
multiply and add.
sqrt
Square root. Inputs: arg: float; round_style: round_type; guard: NATURAL; check_error:
BOOLEAN; denormalize: BOOLEAN. Returns the square root of arg, as defined by
IEEE Std 754-2019.
Is_negative
Returns TRUE if the floating-point number is negative, or FALSE otherwise.
eq
The eq function is similar to the "=" operator; however, it allows the user to turn NaN processing is
on or off.
ne
The ne function is similar to the "/=" operator; however, it allows the user to turn NaN processing is
on or off.
lt
The lt function is similar to the "<" operator; however, it allows the user to turn NaN processing is
on or off.
gt
The gt function is similar to the ">" operator; however, it allows the user to turn NaN processing is
on or off.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
593
Copyright © 2019 IEEE. All rights reserved.
G.5.4.3 Conversion functions
le
The le function is similar to the "<=" operator; however, it allows the user to turn NaN processing is
on or off.
ge
The ge function is similar to the ">=" operator; however, it allows the user to turn NaN processing is
on or off.
std_match
Same as the NUMERIC_STD std_match function. Overloaded for type float.
maximum
Returns the larger of two numbers.
minimum
Returns the smaller of two numbers.
Resize
Changes the size of a float (larger or smaller). Inputs: arg (float); exponent_width and
fraction_width (NATURAL), or size_res; round_style: round_type; Check_error:
BOOLEAN; denormalize_in: BOOLEAN; denormalize: BOOLEAN. In this function,
denormalize_in is TRUE if the input number can be denormal, and denormalize is TRUE
if the output number can be denormal.
To_slv
Inputs: arg (float). Converts a floating-point number to a std_logic_vector of the same
length.
To_std_logic_vector
Alias for to_slv.
To_stdlogicvector
Alias for to_slv.
To_sulv
Inputs: arg (float). Converts a floating-point number to a std_ulogic_vector of the same
length.
To_std_ulogic_vector
Alias for to_sulv.
To_stdulogicvector
Alias for to_sulv.
To_float
Converts to the float type. The default size returned by these functions is set by
float_exponent_width and float_fraction_width.
To_float (std_logic_vector)
Std_logic_vector to float. Inputs: arg (std_logic_vector);
exponent_width and fraction_width (NATURAL), or
size_res (float).
To_float (INTEGER)
Integer to float. Inputs: arg (INTEGER); exponent_width
and fraction_width (NATURAL), or size_res (float);
round_style: round_type.
To_float (REAL)
Real to float. Inputs: arg (REAL); exponent_width and
fraction_width (NATURAL), or size_res (float);
round_style: round_type; denormalize: BOOLEAN.
To_float(ufixed)
Ufixed to float. Inputs: arg(ufixed); exponent_width and
fraction_width (NATURAL), or size_res (float);
round_style: round_type; denormalize: BOOLEAN.
To_float(sfixed)
Sfixed to float. Inputs: arg(sfixed); exponent_width and
fraction_width (NATURAL), or size_res (float);
round_style: round_type; denormalize: BOOLEAN.
To_float (signed)
Signed to float. Inputs: arg (signed); exponent_width and
fraction_width (NATURAL), or size_res (float);
round_style: round_type.
To_float (unsigned)
Unsigned to float. Inputs: arg (signed); exponent_width and
fraction_width (NATURAL), or size_res (float);
round_style: round_type.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
594
Copyright © 2019 IEEE. All rights reserved.
G.5.4.4 IEEE 754 recommended functions and predicates
To_unsigned
Float to unsigned. Inputs: arg (float); size: NATURAL. Parameters: round_style:
round_type; check_error: BOOLEAN. This does not produce a “vector truncated”
warning as the NUMERIC_STD functions do. Returns a zero if the number is negative.
Returns a saturated value if the input is too big.
To_signed
Float to signed. Inputs: arg (float); size: NATURAL. Parameters: round_style:
round_type; check_error: BOOLEAN. This does not produce a “vector truncated”
warning as the NUMERIC_STD functions do. Returns a saturated value if the number is
too big.
To_ufixed
Float to ufixed. Inputs: arg (float); left_index and right_index (NATURAL), or size_res
(ufixed). Parameters overflow_style: BOOLEAN; round_style: BOOLEAN; check_error:
BOOLEAN; and denormalize: BOOLEAN.
To_sfixed
Float to sfixed. Inputs: arg (float); left_index and right_index (NATURAL), or size_res
(ufixed). Parameters overflow_style: BOOLEAN; round_style: BOOLEAN; check_error:
BOOLEAN; and denormalize: BOOLEAN.
To_real
Float to REAL. Inputs: arg (float). Parameters: check_error: BOOLEAN; denormalize:
BOOLEAN.
To_integer
Float to integer. Inputs: arg (float). Parameters: round_style: round_type; check_error:
BOOLEAN.
realtobits
Inputs: arg (REAL). Converts a real number to a std_ulogic_vector in the same format as a
float64 floating-point number.
bitstoreal
Inputs: arg (std_ulogic_vector). Converts a std_ulogic_vector in the same format as a
float64 floating-point number to a real number.
To_01
Inputs (arg: float). Parameters: xmap: std_ulogic. Converts metavalues in the vector arg to
the xmap state (defaults to '0').
Is_X
Inputs (arg: float). Returns a BOOLEAN which is TRUE if there are any metavalues in the
vector arg.
To_x01
Inputs (arg: float). Converts any metavalues found in the vector arg to be 'X' , and
non-metavalues to '0' or '1'.
To_x01z
Inputs (arg: float). Converts any metavalues other than 'Z' found in the vector arg to be
'X' , and non-metavalues to '0' or '1'.
To_ux01
Inputs (arg: float). Converts any metavalues other than 'U' found in the vector arg to be
'X' , and non-metavalues to '0' or '1'.
Break_number
Procedure to break a floating-point number into its parts. Inputs: arg: float; denormalize:
BOOLEAN; check_error: BOOLEAN. Output: fract: unsigned or ufixed fraction (with a
'1' in the most significant bit); expon: the signed exponent (biased by –1, so add 1 to get
the true exponent); sign: the sign bit.
Normalize
Function to take a fixed-point number and an exponent and return a floating-point number.
Inputs: fract: ufixed; expon: signed (assumed to be biased by –1); sign: std_ulogic.
Parameters: exponent_width and fraction_width (NATURAL), or size_res (float);
round_style: round_type; denormalize: BOOLEAN; nguard: NATURAL. There is also a
version of this function in which fract is an unsigned.
copysign(x, y)
Returns x with the sign of y.
scalb(y, n)
Returns y*(2**n) (where n is an INTEGER or SIGNED) without computing 2**n.
logb(x)
Returns the unbiased exponent of x.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
595
Copyright © 2019 IEEE. All rights reserved.
G.5.4.5 Functions returning constants
For each of the following, parameters are exponent_width and fraction_width, or size_res. The default size
is set by the float_exponent_width and float_fraction_width generics.
G.5.4.6 Textio functions
nextafter(x, y)
Returns the next representable number after x in the direction of y.
finite(x)
BOOLEAN, TRUE if x is not positive or negative infinity
isnan(x)
BOOLEAN, TRUE if x is a signaling or quiet NaN.
unordered(x, y)
BOOLEAN, returns TRUE of either x or y are some type of NaN.
classfp
Find the classification of a floating-point number. Inputs: arg (float). Returns a value of the type
valid_fpstate. Note that IEEE Std 754-2019 recommend the name “class” for this function.
However, the floating-point package calls the function “classfp” to avoid conflict with “class”
as a reserved word in a future extension of VHDL.
zerofp
Returns a floating-point positive zero.
nanfp
Returns a floating-point signaling NaN.
qnanfp
Returns a floating-point quiet NaN.
pos_inffp
Returns a floating-point positive infinity.
neg_inffp
Returns a floating-point negative infinity.
neg_zerofp
Returns a floating-point negative zero (which by definition is equal to a floating-point positive zero).
write
Similar to the TEXTIO write procedure. Automatically inserts a “:” after the sign and the
exponent.
read
Similar to the TEXTIO read procedure. If a decimal point or colon is encountered, then it is
tested to ensure that it is in the correct place.
bwrite
Alias for write.
binary_write
Alias for write.
bread
Alias for read.
binary_read
Alias for read.
owrite
Octal write. If the number of bits is not divisible by three, then padding bits are added.
octal_write
Alias for owrite.
oread
Octal read. If the number of bits to be read is not divisible by three, then the number read is
resized to fit.
octal_read
Alias oread.
hwrite
Hex write. If the number of bits is not divisible by four, then padding bits are added.
hex_write
Alias for hwrite.
hread
Hex read. If the number of bits to be read is not divisible by four, then the number read is
resized to fit.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
596
Copyright © 2019 IEEE. All rights reserved.

hex_read
Alias for hread.
to_string
Returns a string that can be padded and left or right justified, for example:
assert (a = 1.5) report "Result was " & to_string (a)
   severity error;
to_bstring
Alias for to_string.
to_binary_string
Alias for to_string.
to_ostring
Similar to to_string, but returns a padded octal value.
to_octal_string
Alias for to_ostring.
to_hstring
Similar to to_string, but returns a padded hex value.
to_hex_string
Alias for to_hstring.
from_string
Allows translation of a string (with a binary point in it) into a floating-point number, for
example:
   signal a: float (3 downto -3);
begin
   a <= from_string ("0000.000", a'high, -a'low);
   a <= from_string ("0001.000", a);
Note that this is typically not synthesizable (as it uses the type string). An alternative
assignment that is synthesizable is “A <= "0000000";”.
from_bstring
Alias for from_string.
from_binary_string
Alias for from_string.
from_ostring
Same as from_string, but uses octal numbers.
from_octal_string
Alias for from_ostring.
from_hstring
Same as from_string, but uses hex numbers.
from_hex_string
Alias for from_hstring.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
