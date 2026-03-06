---
title: "Section 9: IEEE Standard for VHDL Language Reference Manual"
document: "VHDL Language Reference Manual"
standard: "IEEE 1076-2019"
domain: "VHDL"
section: "9"
source_txt: "section-9-expressions.txt"
source_pdf: "/Users/richarddje/Documents/github/VHDL-LRM-IEEE-1076-2019.pdf"
---

# Section 9: IEEE Standard for VHDL Language Reference Manual

IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
148
Copyright © 2019 IEEE. All rights reserved.
9. Expressions
### 9.1 General

An expression is a formula that defines the computation of a value.
```ebnf
conditional_or_unaffected_expression ::=
    expression_or_unaffected { when condition else expression_or_unaffected } [ when condition ]
expression_or_unaffected ::=
    expression | unaffected
conditional_expression ::=
    expression { when condition else expression }
expression ::=
        condition_operator primary
    |   logical_expression
logical_expression ::=
        relation { and relation }
    |   relation { or relation }
    |   relation { xor relation }
    |   relation [ nand relation ]
    |   relation [ nor relation ]
    |   relation { xnor relation }
relation ::=
    shift_expression [ relational_operator shift_expression ]
shift_expression ::=
    simple_expression [ shift_operator simple_expression ]
simple_expression ::=
    [ sign ] term { adding_operator term }
term ::=
    factor { multiplying_operator factor }
factor ::=
    unary_expression [ ** unary_expression ]
unary_expression ::=
        primary
    |   abs primary
    |   not primary
    |   unary_logical_operator primary
primary ::=
        name
    |   literal
    |   aggregate
    |   function_call
```

Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
149
Copyright © 2019 IEEE. All rights reserved.
    |   qualified_expression
    |   type_conversion
    |   allocator
    |   ( conditional_expression )
```ebnf
condition ::=  expression
```

Each primary has a value and a type. The only names allowed as primaries are attributes that yield values
and names denoting objects or values. In the case of names denoting objects other than objects of file types
or protected types, the value of the primary is the value of the object. In the case of names denoting either
file objects or objects of protected types, the value of the primary is the entity denoted by the name.
The type of an expression depends only upon the types of its operands and on the operators applied; for an
overloaded operand or operator, the determination of the operand type, or the identification of the
overloaded operator, depends on the context (see 12.5). For each predefined operator, the operand and result
types are given in the following subclause.
All of the candidate result expressions in a conditional expression shall have the same base type. The value
of a conditional expression is the value of the expression preceding the first condition that evaluates to
TRUE, if any. Otherwise, the value is determined from the final, unconditional expression.
Each of the expressions in a conditional or unaffected expression either is a candidate result expression of a
common base type or is the reserved word unaffected. For the purpose of evaluation, a conditional or
unaffected expression the last expression of which has a condition is implicitly completed by a final
unconditional choice of unaffected. The conditional or unaffected expression evaluates to the choice
preceding the first condition that evaluates to TRUE, if any. Otherwise, the value is determined from the
final (possibly implicitly added) unconditional expression.
NOTE 1—The syntax for an expression involving logical operators allows a sequence of binary and, or, xor, or xnor
operators (whether predefined or user-defined), since the corresponding predefined operations are associative. For the
binary operators nand and nor (whether predefined or user-defined), however, such a sequence is not allowed, since the
corresponding predefined operations are not associative.
NOTE 2—PSL extends the grammar of VHDL expressions to allow PSL expressions, PSL built-in function calls, and
PSL union expressions as subexpressions. Such extended expressions can only appear in a VHDL description within
PSL declarations and PSL directives, or in a verification unit.
### 9.2 Operators

#### 9.2.1 General

The operators that may be used in expressions are defined as follows. Each operator belongs to a class of
operators, all of which have the same precedence level; the classes of operators are listed in order of
increasing precedence.
```ebnf
condition_operator     ::=     ??
logical_operator  ::=  and  |  or  |  nand  |  nor  |  xor  |  xnor
relational_operator ::=  =  |  /=  |  <  |  <=  |  >  |  >=  |  ?=  |  ?/=  |  ?<  |  ?<=  |  ?>  |  ?>=
shift_operator  ::=  sll  |  srl  |  sla  |  sra  |  rol  |  ror
adding_operator  ::=  +  |  –  |  &
```

Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
150
Copyright © 2019 IEEE. All rights reserved.
```ebnf
sign  ::=  +  |  –
multiplying_operator  ::=  *  |  /  |  mod  |  rem
binary_miscellaneous_operator ::= **
unary_miscellaneous_operator  ::=    abs  |  not | unary_logical_operator
```

Operators of higher precedence are associated with their operands before operators of lower precedence.
Where the language allows a sequence of operators, operators with the same precedence level are associated
with their operands in textual order, from left to right. The precedence of an operator is fixed and cannot be
changed by the user, but parentheses can be used to control the association of operators and operands.
In general, operands in an expression are evaluated before being associated with operators. For certain
operations, however, the right-hand operand is evaluated if and only if the left-hand operand has a certain
value. These operations are called short-circuit operations. The binary logical operations and, or, nand, and
nor defined for operands of types BIT and BOOLEAN are all short-circuit operations; furthermore, these
are the only short-circuit operations.
Every predefined operator and every predefined MINIMUM and MAXIMUM operation is a pure function
(see 4.2.1). No predefined operators have named formal parameters; therefore, named association (see
6.5.7.1) cannot be used when invoking a predefined operator.
NOTE—The predefined operators for the standard types are declared in package STANDARD as shown in 16.3.
#### 9.2.2 Logical operators

The binary logical operators and, or, nand, nor, xor, and xnor, and the unary logical operator not are
defined for predefined types BIT and BOOLEAN. They are also defined for any one-dimensional array type
whose element type is BIT or BOOLEAN.
For the binary operators and, or, nand, nor, xor, and xnor, the operands shall both be of the same base
type, or one operand shall be of a scalar type and the other operand shall be a one-dimensional array whose
element type is the scalar type. The result type is the same as the base type of the operands if both operands
are scalars of the same base type or both operands are arrays, or the same as the base type of the array
operand if one operand is a scalar and the other operand is an array.
If both operands are one-dimensional arrays, the operands shall be arrays of the same length, the operation is
performed on matching elements of the arrays, and the result is an array with the same index range as the left
operand. If one operand is a scalar and the other operand is a one-dimensional array, the operation is
performed on the scalar operand with each element of the array operand. The result is an array with the same
index range as the array operand.
For the unary operator not, the result type is the same as the base type of the operand. If the operand is a
one-dimensional array, the operation is performed on each element of the operand, and the result is an array
with the same index range as the operand.
The effects of the logical operators are defined in the following tables. The symbol T represents TRUE for
type BOOLEAN, '1' for type BIT; the symbol F represents FALSE for type BOOLEAN, '0' for type BIT.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
151
Copyright © 2019 IEEE. All rights reserved.
For the short-circuit operations and, or, nand, and nor on types BIT and BOOLEAN, the right operand is
evaluated only if the value of the left operand is not sufficient to determine the result of the operation. For
operations and and nand, the right operand is evaluated only if the value of the left operand is T; for
operations or and nor, the right operand is evaluated only if the value of the left operand is F.
The unary logical operators and, or, nand, nor, xor, and xnor are referred to as logical reduction operators.
The logical reduction operators are predefined for any one-dimensional array type whose element type is
BIT or BOOLEAN. The result type for the logical reduction operators is the same as the element type of the
operand.
The values returned by the logical reduction operators are defined as follows. In the remainder of this
subclause, the values of their arguments are referred to as R.
—
The and operator returns a value that is the logical and of the elements of R. That is, if R is a null
array, the return value is '1' if the element type of R is BIT or TRUE if the element type of R is
BOOLEAN. Otherwise, the return value is the result of a binary and operation. The left argument of
the binary and operation is the leftmost element of R. The right argument of the binary and
operation is the result of a unary and operation with the argument being the rightmost (R'LENGTH
– 1) elements of R.
—
The or operator returns a value that is the logical or of the elements of R. That is, if R is a null array,
the return value is '0' if the element type of R is BIT or FALSE if the element type of R is
BOOLEAN. Otherwise, the return value is the result of a binary or operation. The left argument of
the binary or operation is the leftmost element of R. The right argument of the binary or operation is
the result of a unary or operation with the argument being the rightmost (R'LENGTH – 1) elements
of R.
—
The xor operator returns a value that is the logical exclusive-or of the elements of R. That is, if R is
a null array, the return value is '0' if the element type of R is BIT or FALSE if the element type of R
is BOOLEAN. Otherwise, the return value is the result of a binary xor operation. The left argument
of the binary xor operation is the leftmost element of R. The right argument of the binary xor
operation is the result of a unary xor operation with the argument being the rightmost (R'LENGTH –
1) elements of R.
A
T
T
F
F
B
T
F
T
F
A and B
T
F
F
F
A
T
T
F
F
B
T
F
T
F
A or B
T
T
T
F
A
T
T
F
F
B
T
F
T
F
A xor B
F
T
T
F
A
T
T
F
F
B
T
F
T
F
A nand B
F
T
T
T
A
T
T
F
F
B
T
F
T
F
A nor B
F
F
F
T
A
T
T
F
F
B
T
F
T
F
A xnor B
T
F
F
T
A
T
F
not A
F
T
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
152
Copyright © 2019 IEEE. All rights reserved.
—
The nand operator returns a value that is the negated logical and of the elements of R. That is, the
return value is the result of a not operation. The argument of the not operation is the result of a
unary and operation with the argument being R.
—
The nor operator returns a value that is the negated logical or of the elements of R. That is, the
return value is the result of a not operation. The argument of the not operation is the result of a
unary or operation with the argument being R.
—
The xnor operator returns a value that is the negated logical exclusive-or of the elements of R. That
is, the return value is the result of a not operation. The argument of the not operation is the result of
a unary xor operation with the argument being R.
#### 9.2.3 Relational operators

Relational operators include tests for equality, inequality, and ordering of operands. The operands of each
relational operator shall be of the same type. The result type of each ordinary relational operator (=, /=, <,
<=, >, and >=) is the predefined type BOOLEAN. The result type of each matching relational operator (?=,
?/=, ?<, ?<=, ?>, and ?>=) is the same as the type of the operands (for scalar operands) or the element type of
the operands (for array operands).
Operator
Operation
Operand type
Result type
=
Equality
Any type, other than a file
type, a protected type, or a
composite type that contains a
file type or a protected type
BOOLEAN
/=
Inequality
Any type, other than a file
type in a protected type, or a
composite type that contains a
file type or a protected type
BOOLEAN
<
<=
>
>=
Ordering
Any scalar type or array type
BOOLEAN
?=
Matching
equality
BIT or
STD_ULOGIC
Same type
Any one-dimensional array
type whose element type is
BIT or STD_ULOGIC
The element
type
?/=
Matching
inequality
BIT or STD_ULOGIC
Same type
Any one-dimensional array
type whose element type is
BIT or STD_ULOGIC
The element
type
?<
?<=
?>
?>=
Matching
ordering
BIT or STD_ULOGIC
Same type
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
153
Copyright © 2019 IEEE. All rights reserved.
The equality and inequality operators (= and /=) are defined for all types other than file types, protected
types and types that contain a file type or protected type. The equality operator returns the value TRUE if the
two operands are equal and returns the value FALSE otherwise. The inequality operator returns the value
FALSE if the two operands are equal and returns the value TRUE otherwise.
Two scalar values of the same type are equal if and only if the values are the same. Two composite values of
the same type are equal if and only if for each element of the left operand there is a matching element of the
right operand and vice versa, and the values of matching elements are equal, as given by the predefined
equality operator for the element type. In particular, two null arrays of the same type are always equal. Two
values of an access type are equal if and only if they both designate the same object or they both are equal to
the null value for the access type.
For two record values, matching elements are those that have the same element identifier. For two
one-dimensional array values, matching elements are those (if any) whose index values match in the
following sense: the left bounds of the index ranges are defined to match; if two elements match, the
elements immediately to their right are also defined to match. For two multidimensional array values,
matching elements are those whose indices match in successive positions.
The ordinary ordering operators are defined for any scalar type and for any scalar array type. A scalar array
is a one-dimensional array whose elements are of a scalar type. Likewise a discrete array is a one-
dimensional array whose elements are of a discrete type. Each operator returns TRUE if the corresponding
relation is satisfied; otherwise, the operator returns FALSE.
For scalar types, ordering is defined in terms of the relative values. For scalar array types, the relation < (less
than) is defined such that the left operand is less than the right operand if and only if:
a)
either, the left operand is a null array and the right operand is a non-null array, or
b)
both operands are non-null arrays, and one of the following conditions is satisfied:
—   The leftmost element of the left operand is less than that of the right, or
—   The leftmost element of the left operand is equal to that of the right, and the tail of the left
operand is less than that of the right operand (the tail consists of the remaining elements to the
right (A’LENGTH – 1) elements and can be null).
The relation <= (less than or equal) for scalar array types is defined to be the inclusive disjunction of the
results of the < and = operators for the same two operands. The relations > (greater than) and >= (greater
than or equal) are defined to be the complements of the <= and < operators, respectively, for the same two
operands.
The matching relational operators are predefined for the predefined type BIT and for the type
STD_ULOGIC defined in package STD_LOGIC_1164. For operands of type BIT, each matching relational
operator returns '1' if the corresponding ordinary relational operator applied to the operands returns TRUE,
and returns the value '0' otherwise.
For the matching ordering operators applied to operands of type STD_ULOGIC, if either operand is the
value '–', an error is reported in a manner equivalent to execution of the following assertion statement (see
10.3):
assert FALSE
   report "STD_LOGIC_1164: '-' operand for matching ordering operator"
   severity ERROR;
For operands of type STD_ULOGIC, the value returned by the matching equality operator is defined in the
following table:
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
154
Copyright © 2019 IEEE. All rights reserved.
For operands of type STD_ULOGIC, the value returned by the matching ordering operator ?< is defined in
the following table:
For operands of type STD_ULOGIC, the value returned by the matching inequality operator is defined to be
the result of applying the not operator to the result of applying the ?= operator to the operands. The value
returned by the matching ordering operator ?<= is defined to be the result of applying the binary or operator
to the results of applications of the ?< and ?= operators to the operands. The value returned by the matching
ordering operator ?> is the result of applying the not operator to the result of applying the ?<= operator to
the operands. The value returned by the matching ordering operator ?>= is the result of applying the not
operator to the result of applying the ?< operator to the operands. In each case, the not and or operators are
those declared in the package IEEE.STD_LOGIC_1164.
?=
Right operand
Left operand
'U'
'X'
'0'
'1'
'Z'
'W'
'L'
'H'
'–'
'U'
'U'
'U'
'U'
'U'
'U'
'U'
'U'
'U'
'1'
'X'
'U'
'X'
'X'
'X'
'X'
'X'
'X'
'X'
'1'
'0'
'U'
'X'
'1'
'0'
'X'
'X'
'1'
'0'
'1'
'1'
'U'
'X'
'0'
'1'
'X'
'X'
'0'
'1'
'1'
'Z'
'U'
'X'
'X'
'X'
'X'
'X'
'X'
'X'
'1'
'W'
'U'
'X'
'X'
'X'
'X'
'X'
'X'
'X'
'1'
'L'
'U'
'X'
'1'
'0'
'X'
'X'
'1'
'0'
'1'
'H'
'U'
'X'
'0'
'1'
'X'
'X'
'0'
'1'
'1'
'–'
'1'
'1'
'1'
'1'
'1'
'1'
'1'
'1'
'1'
?<
Right operand
Left operand
'U'
'X'
'0'
'1'
'Z'
'W'
'L'
'H'
'–'
'U'
'U'
'U'
'U'
'U'
'U'
'U'
'U'
'U'
'X'
'X'
'U'
'X'
'X'
'X'
'X'
'X'
'X'
'X'
'X'
'0'
'U'
'X'
'0'
'1'
'X'
'X'
'0'
'1'
'X'
'1'
'U'
'X'
'0'
'0'
'X'
'X'
'0'
'0'
'X'
'Z'
'U'
'X'
'X'
'X'
'X'
'X'
'X'
'X'
'X'
'W'
'U'
'X'
'X'
'X'
'X'
'X'
'X'
'X'
'X'
'L'
'U'
'X'
'0'
'1'
'X'
'X'
'0'
'1'
'X'
'H'
'U'
'X'
'0'
'0'
'X'
'X'
'0'
'0'
'X'
'–'
'X'
'X'
'X'
'X'
'X'
'X'
'X'
'X'
'X'
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
155
Copyright © 2019 IEEE. All rights reserved.
The matching equality and matching inequality operators are also defined for any one-dimensional array
type whose element type is BIT or STD_ULOGIC. The operands shall be arrays of the same length. The
matching equality operator for the element type is applied to matching elements of the operands to form an
intermediate array of type BIT_VECTOR (in the case of operands whose element type is BIT) or
STD_ULOGIC_VECTOR (in the case of operands whose element type is STD_ULOGIC). The result of the
matching equality operator applied to the operands is then the result of applying the unary and operator to
the intermediate array. The result of the matching inequality operator is the result of applying the not
operator to the result of applying the unary and operator to the intermediate array. In each case, the not and
and operators are either the predefined operators or those declared in the package IEEE.STD_LOGIC_1164,
as appropriate.
#### 9.2.4 Shift operators

The shift operators sll, srl, sla, sra, rol, and ror are defined for any one-dimensional array type whose
element type is either of the predefined types BIT or BOOLEAN.
The index range of the return value of each shift operator is the same as the index range of the left operand.
The values returned by the shift operators are defined as follows. In the remainder of this subclause, the val-
ues of their leftmost arguments are referred to as L and the values of their rightmost arguments are referred
to as R.
—
The sll operator returns a value that is L logically shifted left by R index positions. That is, if R is 0
or if L is a null array, the return value is L. Otherwise, a basic shift operation replaces L with a value
that is the result of a concatenation whose left argument is the rightmost (L'LENGTH – 1) elements
of L and whose right argument is T'LEFT, where T is the element type of L. If R is positive, this
basic shift operation is repeated R times to form the result. If R is negative, then the return value is
the value of the expression L srl –R.
—
The srl operator returns a value that is L logically shifted right by R index positions. That is, if R is 0
or if L is a null array, the return value is L. Otherwise, a basic shift operation replaces L with a value
that is the result of a concatenation whose right argument is the leftmost (L'LENGTH – 1) elements
of L and whose left argument is T'LEFT, where T is the element type of L. If R is positive, this basic
shift operation is repeated R times to form the result. If R is negative, then the return value is the
value of the expression L sll –R.
—
The sla operator returns a value that is L arithmetically shifted left by R index positions. That is, if R
is 0 or if L is a null array, the return value is L. Otherwise, a basic shift operation replaces L with a
Operator
Operation
Left operand type
Right operand type
Result type
sll
Shift left
logical
Any one-dimensional array type whose
element type is BIT or BOOLEAN
INTEGER
Same as left
srl
Shift right
logical
Any one-dimensional array type whose
element type is BIT or BOOLEAN
INTEGER
Same as left
sla
Shift left
arithmetic
Any one-dimensional array type whose
element type is BIT or BOOLEAN
INTEGER
Same as left
sra
Shift right
arithmetic
Any one-dimensional array type whose
element type is BIT or BOOLEAN
INTEGER
Same as left
rol
Rotate left
logical
Any one-dimensional array type whose
element type is BIT or BOOLEAN
INTEGER
Same as left
ror
Rotate right
logical
Any one-dimensional array type whose
element type is BIT or BOOLEAN
INTEGER
Same as left
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
156
Copyright © 2019 IEEE. All rights reserved.
value that is the result of a concatenation whose left argument is the rightmost (L'LENGTH – 1)
elements of L and whose right argument is L(L'RIGHT). If R is positive, this basic shift operation is
repeated R times to form the result. If R is negative, then the return value is the value of the
expression L sra –R.
—
The sra operator returns a value that is L arithmetically shifted right by R index positions. That is, if
R is 0 or if L is a null array, the return value is L. Otherwise, a basic shift operation replaces L with
a value that is the result of a concatenation whose right argument is the leftmost (L'LENGTH – 1)
elements of L and whose left argument is L(L'LEFT). If R is positive, this basic shift operation is
repeated R times to form the result. If R is negative, then the return value is the value of the
expression L sla –R.
—
The rol operator returns a value that is L rotated left by R index positions. That is, if R is 0 or if L is
a null array, the return value is L. Otherwise, a basic rotate operation replaces L with a value that is
the result of a concatenation whose left argument is the rightmost (L'LENGTH – 1) elements of L
and whose right argument is L(L'LEFT). If R is positive, this basic rotate operation is repeated R
times to form the result. If R is negative, then the return value is the value of the expression L ror –
R.
—
The ror operator returns a value that is L rotated right by R index positions. That is, if R is 0 or if L
is a null array, the return value is L. Otherwise, a basic rotate operation replaces L with a value that
is the result of a concatenation whose right argument is the leftmost (L'LENGTH – 1) elements of L
and whose left argument is L(L'RIGHT). If R is positive, this basic rotate operation is repeated R
times to form the result. If R is negative, then the return value is the value of the expression L rol –
R.
NOTE 1—The logical operators may be overloaded, for example, to disallow negative integers as the second argument.
NOTE 2—The subtype of the result of a shift operator is the same as that of the left operand.
#### 9.2.5 Adding operators

The adding operators + and – are predefined for any numeric type and have their conventional mathematical
meaning. The concatenation operator & is predefined for any one-dimensional array type.
For concatenation, there are three mutually exclusive cases, as follows:
a)
If both operands are one-dimensional arrays of the same type, the result of the concatenation is a
one-dimensional array of this same type whose length is the sum of the lengths of its operands, and
Operator
Operation
Left operand type
Right operand type
Result type
+
Addition
Any numeric type
Same type
Same type
–
Subtraction
Any numeric type
Same type
Same type
&
Concatenation
Any one-dimensional
array type
Same array type
Same array type
Any one-dimensional
array type
The element type
Same array type
The element type
Any one-dimensional
array type
Same array type
The element type
The element type
Any one-dimensional
array type
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
157
Copyright © 2019 IEEE. All rights reserved.
whose elements consist of the elements of the left operand (in left-to-right order) followed by the
elements of the right operand (in left-to-right order).
If both operands are null arrays, then the result of the concatenation is the right operand. Otherwise,
the direction and bounds of the result are determined as follows: Let S be the index subtype of the
base type of the result. The direction of the result of the concatenation is the direction of S, and the
left bound of the result is S'LEFT.
b)
If one of the operands is a one-dimensional array and the type of the other operand is the element
type of this aforementioned one-dimensional array, the result of the concatenation is given by the
rules in case a), using in place of the other operand an implicit array having this operand as its only
element. Both the left and right bounds of the index subtype of this implicit array is S'LEFT, and the
direction of the index subtype of this implicit array is the direction of S, where S is the index subtype
of the base type of the result.
c)
If both operands are of the same type and it is the element type of some one-dimensional array type,
the type of the result is this one-dimensional array type. In this case, each operand is treated as the
one element of an implicit array, and the result of the concatenation is determined as in case a). The
bounds and direction of the index subtypes of the implicit arrays are determined as in the case of the
implicit array in case b).
In all cases, it is an error if either bound of the index range of the result does not belong to the index subtype
of the type of the result, unless the result is a null array. It is also an error if any element of the result does
not belong to the element subtype of the type of the result.
Examples:
subtype BYTE is BIT_VECTOR (7 downto 0);
type MEMORY is array (Natural range <>) of BYTE;
--  The following concatenation accepts two BIT_VECTORs and returns
--  a BIT_VECTOR [case a)]:
constant ZERO: BYTE := "0000" & "0000";
--  The next two examples show that the same expression can represent
--  either case a) or case c), depending on the context of
--  the expression.
--  The following concatenation accepts two BIT_VECTORS and returns
--  a BIT_VECTOR [case a)]:
constant C1: BIT_VECTOR := ZERO & ZERO;
--  The following concatenation accepts two BIT_VECTORs and returns
--  a MEMORY [case c)]:
constant C2: MEMORY := ZERO & ZERO;
--  The following concatenation accepts a BIT_VECTOR and a MEMORY,
--  returning a MEMORY [case b)]:
constant C3: MEMORY := ZERO & C2;
--  The following concatenation accepts a MEMORY and a BIT_VECTOR,
--  returning a MEMORY [case b)]:
constant C4: MEMORY := C2 & ZERO;
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
158
Copyright © 2019 IEEE. All rights reserved.
--  The following concatenation accepts two MEMORYs and returns
--  a MEMORY [case a)]:
constant C5: MEMORY := C2 & C3;
type R1 is range 0 to 7;
type R2 is range 7 downto 0;
type T1 is array (R1 range <>) of Bit;
type T2 is array (R2 range <>) of Bit;
subtype S1 is T1(R1);
subtype S2 is T2(R2);
constant K1: S1 := (others => '0');
constant K2: T1 := K1(1 to 3) & K1(3 to 4);  --  K2'Left = 0
                                             --     and K2'Right = 4
constant K3: T1 := K1(5 to 7) & K1(1 to 2);  --  K3'Left = 0
                                             --     and K3'Right = 4
constant K4: T1 := K1(2 to 1) & K1(1 to 2);  --  K4'Left = 0
                                             --     and K4'Right = 1
constant K5: S2 := (others => '0');
constant K6: T2 := K5(3 downto 1) & K5(4 downto 3);
-- K6'Left = 7

-- and K6'Right = 3
constant K7: T2 := K5(7 downto 5) & K5(2 downto 1);
-- K7'Left = 7
-- and K7'Right = 3
constant K8: T2 := K5(1 downto 2) & K5(2 downto 1);
-- K8'Left = 7
-- and K8'Right = 6
NOTE 1—For a given concatenation whose operands are of the same type, there may be visible more than one array
type that could be the result type according to the rules of case c). The concatenation is ambiguous and therefore an error
if, using the overload resolution rules of 4.5 and 12.5, the type of the result is not uniquely determined.
NOTE 2—Additionally, for a given concatenation, there may be visible array types that allow both case a) and case c) to
apply. The concatenation is again ambiguous and therefore an error if the overload resolution rules cannot be used to
determine a result type uniquely.
#### 9.2.6 Sign operators

Signs + and – are predefined for any numeric type and have their conventional mathematical meaning: they
respectively represent the identity and negation functions. For each of these unary operators, the operand
and the result have the same type.
Operator
Operation
Operand type
Result type
+
Identity
Any numeric type
Same type
–
Negation
Any numeric type
Same type
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
159
Copyright © 2019 IEEE. All rights reserved.
NOTE—Because of the relative precedence of signs + and – in the grammar for expressions, a signed operand will not
follow a multiplying operator, the exponentiating operator **, or the operators abs and not. For example, the syntax
does not allow the following expressions:
A/+B           --  An illegal expression.
A**-B          --  An illegal expression.
However, these expressions may be rewritten legally as follows:
A /(+B)         --  A legal expression.
A ** (-B)      --  A legal expression.
#### 9.2.7 Multiplying operators

The operators * and / are predefined for any integer and any floating-point type and have their conventional
mathematical meaning; the operators mod and rem are predefined for any integer type. For each of these
operators, the operands and the result are of the same type.
Integer division and remainder are defined by the following relation:
A = (A/B) * B + (A rem B)
where (A rem B) has the sign of A and an absolute value less than the absolute value of B. Integer division
satisfies the following identity:
(–A)/B = – (A/B) = A/(–B)
The result of the modulus operation is such that (A mod B) has the sign of B and an absolute value less than
the absolute value of B; in addition, for some integer value N, this result shall satisfy the relation:
A = B * N + (A mod B)
In addition to the preceding table, the multiplying operators are predefined for any physical type.
Operator
Operation
Left operand type
Right operand type
Result type
*
Multiplication
Any integer type
Same type
Same type
Any floating-point type
Same type
Same type
/
Division
Any integer type
Same type
Same type
Any floating-point type
Same type
Same type
mod
Modulus
Any integer type
Same type
Same type
rem
Remainder
Any integer type
Same type
Same type
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
160
Copyright © 2019 IEEE. All rights reserved.
Multiplication of a value P of a physical type Tp by a value I of type INTEGER is equivalent to the
following computation:
Tp'Val( Tp'Pos(P) * I )
Multiplication of a value P of a physical type Tp by a value F of type REAL is equivalent to the following
computation:
Tp'Val( INTEGER( REAL( Tp'Pos(P) ) * F ))
Division of a value P of a physical type Tp by a value I of type INTEGER is equivalent to the following
computation:
Tp'Val( Tp'Pos(P) / I )
Division of a value P of a physical type Tp by a value F of type REAL is equivalent to the following
computation:
Tp'Val( INTEGER( REAL( Tp'Pos(P) ) / F ))
Division of a value P of a physical type Tp by a value P2 of the same physical type is equivalent to the
following computation:
Tp'Pos(P) / Tp'Pos(P2)
The computation of P mod P2, where P and P2 are values of a physical type Tp, is equivalent to the
following computation:
Tp'Val(Tp'Pos(P) mod Tp'Pos(P2))
The computation of P rem P2, where P and P2 are values of a physical type Tp, is equivalent to the
following computation:
Tp'Val(Tp'Pos(P) rem Tp'Pos(P2))
Operator
Operation
Left operand type
Right operand type
Result type
*
Multiplication
Any physical type
INTEGER
Same as left
Any physical type
REAL
Same as left
INTEGER
Any physical type
Same as right
REAL
Any physical type
Same as right
/
Division
Any physical type
INTEGER
Same as left
Any physical type
REAL
Same as left
Any physical type
The same type
Universal integer
mod
Modulus
Any physical type
Same type
Same type
rem
Remainder
Any physical type
Same type
Same type
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
161
Copyright © 2019 IEEE. All rights reserved.
Examples:
## 5      rem    3       =  2

## 5      mod    3       =  2

(-5)     rem    3       = -2
(-5)     mod    3       =  1
(-5)     rem  (-3)      = -2
(-5)     mod  (-3)      = -2
## 5      rem  (-3)      =  2

## 5      mod  (-3)      = -1

## 5 ns   rem    3 ns    = 2 ns

## 5 ns   mod    3 ns    = 2 ns

(-5 ns)  rem    3 ns    = -2 ns
(-5 ns)  mod    3 ns    = 1 ns
## 1 ns   mod    300 ps  = 100 ps

(-1 ns)  mod    300 ps  = 200 ps
NOTE—Because of the precedence rules (see 9.2.1), the expression “–5 rem 2” is interpreted as “–(5 rem 2)” and not as
“(–5) rem 2.”
#### 9.2.8 Miscellaneous operators

The unary operator abs is predefined for any numeric type.
The exponentiating operator ** is predefined for each integer type and for each floating-point type. In either
case the right operand, called the exponent, is of the predefined type INTEGER.
Exponentiation with an integer exponent is equivalent to repeated multiplication of the left operand by itself
for a number of times indicated by the absolute value of the exponent and from left to right; if the exponent
is negative, then the result is the reciprocal of that obtained with the absolute value of the exponent.
Exponentiation with a negative exponent is only allowed for a left operand of a floating-point type.
Exponentiation by a zero exponent results in the value one. Exponentiation of a value of a floating-point
type is approximate.
#### 9.2.9 Condition operator

The unary operator ?? is predefined for type BIT defined in package STANDARD (see 16.3).
Operator
Operation
Operand type
Result type
abs
Absolute value
Any numeric type
Same numeric type
Operator
Operation
Left operand type
Right operand type
Result type
**
Exponentiation
Any integer type
INTEGER
Same as left
Any floating-point type
INTEGER
Same as left
Operator
Operation
Operand type
Result type
??
Condition conversion
BIT
BOOLEAN
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
162
Copyright © 2019 IEEE. All rights reserved.
Conversion of a value of type BIT converts '1' to TRUE and '0' to FALSE. The conversion operator may be
overloaded for other types.
In certain circumstances, the condition operator is implicitly applied to an expression that occurs as a
condition in any of the following places:
—
After until in the condition clause of a wait statement (see 10.2)
—
After assert in an assertion, either in an assertion statement (see 10.3) or in a concurrent assertion
statement (see 11.5)
—
After if or elsif in an if statement (see 10.8)
—
After while in a while iteration scheme of a loop statement (see 10.10)
—
After when in a next statement (see 10.11)
—
After when in an exit statement (see 10.12)
—
After when in a return statement (see 10.13)
—
After when in a conditional signal assignment statement (see 10.5.3)
—
After when in a concurrent conditional signal assignment statement (see 11.7)
—
After when in a conditional expression (see 9.1)
—
After when in a conditional or unaffected expression (see 9.1)
—
After if or elsif in an if generate statement (see 11.9)
—
In a guard condition in a block statement (see 11.2)
—
In a Boolean expression in a PSL declaration or a PSL directive
The condition operator implicitly applied, if any, is either the predefined operator for type BIT or an
overloaded operator, determined as follows. If, without overload resolution (see 12.5), the expression is of
type BOOLEAN defined in package STANDARD, or if, assuming a rule requiring the expression to be of
type BOOLEAN defined in package STANDARD, overload resolution can determine at least one
interpretation of each constituent of the innermost complete context including the expression, then the
condition operator is not applied. Otherwise, the condition operator is implicitly applied, and the type of the
expression with the implicit application shall be BOOLEAN defined in package STANDARD.
Example:
use IEEE.STD_LOGIC_1164.all;
signal S: STD_ULOGIC;
assert S;  -- implicit conversion applied
NOTE 1—The condition operator is not implicitly applied if there is at least one interpretation of the expression as being
of type BOOLEAN. If overload resolution yields more than one such interpretation, the expression is of type
BOOLEAN but ambiguous. In cases where the condition operator is implicitly applied to the expression, overload
resolution may yield multiple interpretations, in which case the expression is ambiguous. The expression is only legal if
there is exactly one interpretation of type BOOLEAN without the condition operator, or failing that, one interpretation of
type BOOLEAN with the condition operator.
NOTE 2—The condition operator is defined for type STD_ULOGIC defined in package STD_LOGIC_1164 (see 16.7).
Conversion of a value of type STD_ULOGIC converts '1' and 'H' to TRUE and all other values to FALSE.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
163
Copyright © 2019 IEEE. All rights reserved.
### 9.3 Operands

#### 9.3.1 General

The operands in an expression include names (that denote objects, values, or attributes that result in a value),
literals, aggregates, function calls, qualified expressions, type conversions, and allocators. In addition, an
expression enclosed in parentheses may be an operand in an expression. Names are defined in 8.1; the other
kinds of operands are defined in 9.3.2 through 9.3.7.
#### 9.3.2 Literals

A literal is either a numeric literal, an enumeration literal, a string literal, a bit string literal, or the literal
null.
```ebnf
literal ::=
```

numeric_literal
|    enumeration_literal
|    string_literal
    |    bit_string_literal
    |    null
```ebnf
numeric_literal ::=
```

abstract_literal
    |   physical_literal
Numeric literals include literals of the abstract types universal_integer and universal_real, as well as literals
of physical types. Abstract literals are defined in 15.5; physical literals are defined in 5.2.4.1.
Enumeration literals are literals of enumeration types. They include both identifiers and character literals.
Enumeration literals are defined in 5.2.2.1.
String and bit string literals are representations of one-dimensional arrays of characters. The type of a string
or bit string literal shall be determinable solely from the context in which the literal appears, excluding the
literal itself but using the fact that the type of the literal shall be a one-dimensional array of a character type.
The lexical structure of string and bit string literals is defined in Clause 15.
For a non-null array value represented by either a string or bit string literal, the direction and bounds of the
index range of the array value are determined according to the rules for positional array aggregates, where
the number of elements in the aggregate is equal to the length (see 15.7 and 15.8) of the string or bit string
literal. For a null array value represented by either a string or bit string literal, the direction and leftmost
bound of the index range of the array value are determined as follows: the direction and nominal leftmost
bound of the index range of the array value are determined as in the non-null case. If there is a value to the
left of the nominal leftmost bound (given by the 'LEFTOF attribute), then the leftmost bound is the nominal
leftmost bound, and the rightmost bound is the value to the left of the nominal leftmost bound. Otherwise,
the leftmost bound is the value to the right of the nominal leftmost bound, and the rightmost bound is the
nominal leftmost bound.
For a null array value represented by either a string or bit string literal, it is an error if the base type of the
index subtype of the array type does not have at least two values.
The character literals corresponding to the graphic characters contained within a string literal or a bit string
literal shall be visible at the place of the string literal.
The literal null represents the null access value for any access type.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
164
Copyright © 2019 IEEE. All rights reserved.
Evaluation of a literal yields the corresponding value.
Examples:
3.14159_26536     --  A literal of type universal_real.
## 5280              --  A literal of type universal_integer.

### 10.7 ns           --  A literal of a physical type.

O"4777"           --  A bit string literal.
"54LS281"         --  A string literal.
""                --  A string literal representing a null array.
#### 9.3.3 Aggregates

##### 9.3.3.1 General

An aggregate is a basic operation (see 5.1) that combines one or more values into a composite value of a
record or array type.
```ebnf
aggregate ::=
```

( element_association { , element_association } )
```ebnf
element_association ::=
```

[ choices => ] expression
```ebnf
choices ::=  choice { | choice }
choice ::=
```

simple_expression
|   discrete_range
|   element_simple_name
    |   others
Each element association associates an expression with elements (possibly none). An element association is
said to be named if the elements are specified explicitly by choices; otherwise, it is said to be positional. For
a positional association, each element is implicitly specified by position in the textual order of the elements
in the corresponding type declaration.
Both named and positional associations can be used in the same aggregate, with all positional associations
appearing first (in textual order) and all named associations appearing next (in any order, except that it is an
error if any associations follow an others association). Aggregates containing a single element association
shall always be specified using named association in order to distinguish them from parenthesized
expressions.
An element association with a choice that is an element simple name is only allowed in a record aggregate.
An element association with a choice that is a simple expression or a discrete range is only allowed in an
array aggregate: a simple expression specifies the element at the corresponding index value, whereas a
discrete range specifies the elements at each of the index values in the range. Except as described in 9.3.3.3,
the discrete range, and, in particular, the direction specified or implied by the discrete range, has no
significance other than to define the set of choices implied by the discrete range. An element association
with the choice others is allowed in either an array aggregate or a record aggregate if the association appears
last and has this single choice; it specifies all remaining elements, if any.
Each element of the value defined by an aggregate shall be represented once and only once in the aggregate.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
165
Copyright © 2019 IEEE. All rights reserved.
The type of an aggregate shall be determinable solely from the context in which the aggregate appears,
excluding the aggregate itself but using the fact that the type of the aggregate shall be a composite type. The
type of an aggregate in turn determines the required type for each of its elements.
##### 9.3.3.2 Record aggregates

If the type of an aggregate is a record type, the element names given as choices shall denote elements of that
record type. If the choice others is given as a choice of a record aggregate, it shall represent at least one ele-
ment. An element association with more than one choice, or with the choice others, is only allowed if the
elements specified are all of the same type. The expression of an element association shall have the type of
the associated record elements.
A record aggregate is evaluated as follows. The expressions given in the element associations are evaluated
in an order (or lack thereof) not defined by the language. The expression of a named association is evaluated
once for each associated element. A check is made that the value of each element of the aggregate belongs to
the subtype of this element. It is an error if this check fails.
##### 9.3.3.3 Array aggregates

For an aggregate of a one-dimensional array type, each choice shall specify values of the index type, and the
expression of each element association shall be of either the element type or the type of the aggregate. If the
type of the expression of an element association is the type of the aggregate, then either the element
association shall be positional or the choice shall be a discrete range.
For an element association with a choice that is a discrete range and an expression of the element type of the
aggregate, the value of the expression is the element at each index value in the range.
For an element association with a choice that is a discrete range and an expression of the type of the
aggregate, each element of the value of the expression is the value of the element of the aggregate at the
matching index value in the range. The matching index value for an element of the value of the expression is
determined as follows: the leftmost element of the value matches the left bound of the range; if an element
matches an index value, the element immediately to its right matches the index value immediately to the
right in the range. It is an error if the length of the discrete range differs from the length of the value of the
expression.
For a positional association with an expression of the element type of the aggregate, the expression specifies
one element of the aggregate value. For a positional association with an expression of the type of the
aggregate, the expression specifies a number of matching elements (see 9.2.3) of the aggregate value given
by the length of the value of the expression.
An aggregate of an n-dimensional array type, where n is greater than 1, is written as a one-dimensional
aggregate in which the index subtype of the aggregate is given by the first index position of the array type,
and the expression specified for each element association is an (n–1)-dimensional array or array aggregate,
which is called a subaggregate. A string or bit string literal is allowed as a subaggregate in the place of any
aggregate of a one-dimensional array of a character type.
Apart from a final element association with the single choice others, the rest (if any) of the element
associations of an array aggregate shall be either all positional or all named. A named association of an array
aggregate is allowed to have a choice that is not locally static, or likewise a choice that is a null range, only
if the aggregate includes a single element association and this element association has a single choice. An
others choice is locally static if the applicable index constraint is locally static.
The index range of an array aggregate that has an others choice shall be determinable from the context. That
is, an array aggregate with an others choice shall appear only in one of the following contexts:
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
166
Copyright © 2019 IEEE. All rights reserved.
a)
As an actual associated with a formal parameter, formal generic, or formal port (or member thereof),
where either the formal (or the member) is declared to be of a fully constrained array subtype, or the
formal designator is a slice name
b)
As the default expression defining the default initial value of a port declared to be of a fully
constrained array subtype
c)
As the default expression for a generic constant declared to be of a fully constrained array subtype
d)
As the result expression of a function, where the corresponding function result type is a fully
constrained array subtype
e)
As a value expression in an assignment statement, where the target is a declared object (or member
thereof), and either the subtype of the target is a fully constrained array subtype or the target is a
slice name
f)
As the expression defining the initial value of a constant or variable object, where that object is
declared to be of a fully constrained array subtype
g)
As the expression defining the default values of signals in a signal declaration, where the
corresponding subtype is a fully constrained array subtype
h)
As the expression defining the value of an attribute in an attribute specification, where that attribute
is declared to be of a fully constrained array subtype
i)
As the operand of a qualified expression whose type mark denotes a fully constrained array subtype
j)
As a choice in a case statement whose expression is of a one-dimensional character array type and is
one of the following:
—
The name of an object whose subtype is locally static, in which case the index range of the
aggregate is the index range of the subtype of the object
—
An indexed name whose prefix is one of the members of this list and whose indexing
expressions are locally static expressions, in which case the index range of the aggregate is the
index range of the element subtype of the prefix
—
A slice name whose prefix is one of the members of this list and whose discrete range is a
locally static discrete range, in which case the index range of the aggregate is the discrete range
—
A function call whose return type mark denotes a locally static subtype, in which case the
index range of the aggregate is the index range of the subtype denoted by the return type mark
—
A qualified expression or type conversion whose type mark denotes a locally static subtype, in
which case the index range of the aggregate is the index range of the subtype denoted by the
type mark
—
An expression described in this list and enclosed in parentheses, in which case the index range
of the aggregate is the index range of the subtype defined for the enclosed expression
In each case, the applicable index constraint is locally static.
k)
As a subaggregate nested within an aggregate, where that aggregate itself appears in one of these
contexts
The direction of the index range of an array that does not have an others choice are determined as follows:
—
If the aggregate appears in one of the contexts in the preceding list, then the direction of the index
range of the aggregate is that of the corresponding fully constrained array subtype, or that of the
range of the corresponding slice name, as appropriate.
—
If the aggregate does not appear in one of the contexts in the preceding list and an element
association in the aggregate has a choice that is a discrete range and an expression that is of the type
of the aggregate, then the direction of the index range of the aggregate is that of the discrete range.
—
Otherwise, the direction of the index range of the aggregate is that of the index subtype of the base
type of the aggregate.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
167
Copyright © 2019 IEEE. All rights reserved.
The bounds of an array that does not have an others choice are determined as follows. For an aggregate that
has named associations, the leftmost and rightmost bounds are determined by the direction of the index
range of the aggregate and the smallest and largest choices given. For a positional aggregate, the leftmost
bound is determined by the applicable index constraint if the aggregate appears in one of the contexts in the
preceding list; otherwise, the leftmost bound is given by S'LEFT where S is the index subtype of the base
type of the array. In either case, the rightmost bound is determined by the direction of the index range and
the number of elements.
It is an error if the direction of the index range of an aggregate is determined by the context, and an element
association has a choice that is a discrete range and an expression that is of the type of the aggregate, and the
direction of the discrete range differs from that of the index range of the aggregate. If an aggregate has a
given element association with a choice that is a discrete range and an expression that is of the type of the
aggregate, then it is an error if any other element association has a choice that is a discrete range whose
direction differs from that of the choice of the given element association.
The evaluation of an array aggregate that is not a subaggregate proceeds in two steps. First, the choices of
this aggregate and of its subaggregates, if any, are evaluated in some order (or lack thereof) that is not
defined by the language. Second, the expressions of the element associations of the array aggregate are
evaluated in some order that is not defined by the language; the expression of a named association in which
the expression is of the element type of the aggregate is evaluated once for each associated element. The
evaluation of a subaggregate consists of this second step (the first step is omitted since the choices have
already been evaluated).
For the evaluation of an aggregate that is not a null array, a check is made that the index values defined by
choices belong to the corresponding index subtypes, and also that the value of each element of the aggregate
belongs to the subtype of this element. For a multidimensional aggregate of dimension n, a check is made
that all (n–1)-dimensional subaggregates have the same bounds. It is an error if any one of these checks fails.
#### 9.3.4 Function calls

A function call invokes the execution of a function body. The call specifies the name of the function to be
invoked, the actual generics, if any, to be associated with the formal generics of a function, and the actual
parameters, if any, to be associated with the formal parameters of the function. Execution of the function
body results in a value of the type declared to be the result type in the declaration of the invoked function.
```ebnf
function_call ::=
```

function_name [ generic_map_aspect] [ parameter_map_aspect  ]
```ebnf
parameter_map_aspect ::=
```

[ parameter map ] ( parameter_association_list )
For each formal generic of a function, a function call shall specify exactly one corresponding actual generic.
This actual generic is specified either explicitly, by an association element (other than the actual part open)
in the association list, or in the absence of such an association element, by a default.
For each formal parameter of a function, a function call shall specify exactly one corresponding actual
parameter. This actual parameter is specified either explicitly by an association element (other than the
actual part open) in the association list or in the absence of such an association element, by a default
expression.
It is an error if the function name denotes an uninstantiated function and the function call does not have a
generic map aspect.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
168
Copyright © 2019 IEEE. All rights reserved.
Each uninstantiated function call with a generic map aspect is replaced by an implicit subprogram
instantiation declaration and a call to the implicitly defined subprogram instance. The implicit subprogram
instance is declared prior to the uninstantiated function call in the declaration region that immediately
encloses and/or includes the function call. The call to the uninstantiated function is equivalent to a call to the
implicitly instantiated function with the same parameters that were specified in the uninstantiated function
call. The implicitly instanced function does not have a designator that can be referenced separately from the
function call. Hence, ambiguity cannot be introduced by the existence of an implicit function instance.
Evaluation of a function call includes evaluation of the actual parameter expressions specified in the call and
evaluation of the default expressions associated with formal parameters of the function that do not have
actual parameters associated with them. In both cases, the resulting value shall belong to the subtype of the
associated formal parameter. (If the formal parameter is of an unconstrained or partially constrained
composite type, then any undefined index ranges of subelements of the formal parameter are determined as
described in 5.3.2.2.) The function body is executed using the actual parameter values and default
expression values as the values of the corresponding formal parameters.
NOTE—If a name (including one used as a prefix) has an interpretation both as a function call and an indexed name,
then the innermost complete context is used to disambiguate the name. If, after applying this rule, there is not exactly
one interpretation of the name, then the name is ambiguous (see 12.5).
A call to a formal generic function uses the parameter names and default expressions defined in the formal
generic function declaration and the parameter subtypes and result subtype of the associated actual generic
function.
#### 9.3.5 Qualified expressions

A qualified expression is a basic operation (see 5.1) that is used to explicitly state the type, and possibly the
subtype, of an operand that is an expression or an aggregate.
```ebnf
qualified_expression ::=
```

type_mark ' ( expression )
|   type_mark ' aggregate
    |   type_mark ' (  )
For qualified expressions with an operand, the operand shall have the same type as the base type of the type
mark. The value of such a qualified expression is the value of the operand. The evaluation of such a qualified
expression evaluates the operand and converts it to the subtype denoted by the type mark.
A qualified expression without an operand defines a literal of a composite type or subtype with no elements.
The value of such a qualified expression is an empty value of the type or subtype denoted by the type mark.
NOTE—Whenever the type of an enumeration literal or aggregate is not known from the context, a qualified expression
can be used to state the type explicitly.
#### 9.3.6 Type conversions

A type conversion provides for explicit conversion between closely related types.
```ebnf
type_conversion ::=  type_mark ( expression )
```

The target type of a type conversion is the base type of the type mark, and the target subtype of a type
conversion is the type or subtype denoted by the type mark. The type of the operand of a type conversion
shall be determined by applying the rules of 12.5 to the operand considered as a complete context. (In
particular, the type of the operand shall be determinable independent of the target type). Furthermore, the
operand of a type conversion is not allowed to be the literal null, an allocator, an aggregate, or a string
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
169
Copyright © 2019 IEEE. All rights reserved.
literal. An expression enclosed by parentheses is allowed as the operand of a type conversion only if the
expression alone is allowed.
If the type mark denotes a subtype, conversion consists of conversion to the target type followed by a check
that the result of the conversion belongs to the subtype.
In certain cases, an implicit subtype conversion is performed. A subtype conversion involves a type
conversion in which the target subtype is the subtype to which the operand is converted and the target type is
the base type of the target subtype.
Explicit type conversions are allowed between closely related types. In particular, a type is closely related to
itself. Other types are closely related only under the following conditions:
—
Abstract numeric types—Any abstract numeric type is closely related to any other abstract numeric
type.
—
Array types—Two array types are closely related if and only if the types have the same
dimensionality and the element types are closely related
—
Record types—Two record types are closely related if and only if each element has a matching
element (see 9.2.3) in the other type and the corresponding types for matching elements are closely
related.
No other types are closely related.
In a type conversion where the target type is an abstract numeric type, the operand can be of any integer or
floating-point type. The value of the operand is converted to the target type, which shall also be an integer or
floating-point type. The conversion of a floating-point value to an integer type rounds to the nearest integer;
if the value is halfway between two integers, rounding may be up or down.
In the case of conversions between numeric types, it is an error if the result of the conversion fails to satisfy
a constraint imposed by the type mark.
In a type conversion where the target type is an array type, the following rules apply:
If the target subtype is an array type or subtype for which the index ranges are not defined, then, for each
index position, the index range of the result is determined as follows:
If the index type of the operand and the index type of the target type are not closely related, then the
direction and nominal left bound of the index range of the result are the direction and left bound,
respectively, of the corresponding index subtype of the target type. For a non-null range, the left bound of
the index range is the nominal left bound, and the right bound is determined by the number of values in the
corresponding index range of the operand. For a null range, if there is a value to the left of the nominal left
bound (given by the 'LEFTOF attribute), then the left bound is the nominal left bound, and the right bound is
the value to the left of the nominal left bound; otherwise, the left bound is the value to the right of the
nominal left bound, and the right bound is the nominal left bound. For either a non-null or a null range, it is
an error if the base type of the corresponding index subtype of the target type does not include sufficient
values for the index range of the result.
If the index type of the operand and the index type of the target type are closely related, then the bounds of
the index range of the result are obtained by converting the bounds of the index range of the operand to the
index type of the target type, and the direction of the index range of the result is the direction of the index
type of the operand.
If the target subtype is an array subtype for which the index ranges are defined, then the bounds of the result
are those imposed by the target subtype.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
170
Copyright © 2019 IEEE. All rights reserved.
In either case, the value of each element of the result is that of the matching element of the operand (see
9.2.3) converted to the element subtype of the target subtype.
In the case of conversions between array types, if the target subtype is an array type for which the index
ranges are not defined, then, for each index position, a check is made that the bounds of the result belong to
the corresponding index subtype of the target type. If the target subtype is an array subtype for which the
index ranges are defined, a check is made that for each element of the operand there is a matching element of
the target subtype, and vice versa. It is an error if any of these checks fail.
In a type or subtype conversion where the target type is a record type, the value of each element of the result
is that of the matching element of the operand (see 9.2.3) converted to the subtype of the element of the
result.
In certain cases, an implicit type conversion will be performed. An implicit conversion of an operand of type
universal_integer to another integer type, or of an operand of type universal_real to another floating-point
type, can only be applied if the operand is either a numeric literal or an attribute, or if the operand is an
expression consisting of the division of a value of a physical type by a value of the same type; such an
operand is called a convertible universal operand. An implicit conversion of a convertible universal operand
is applied if and only if the innermost complete context determines a unique (numeric) target type for the
implicit conversion, and there is no legal interpretation of this context without this conversion.
NOTE—Two array types may be closely related even if corresponding index positions have different
directions.
#### 9.3.7 Allocators

The evaluation of an allocator creates an object and yields an access value that designates the object.
```ebnf
allocator ::=
```

new subtype_indication [ generic map aspect ]
|
new qualified_expression
The type of the object created by an allocator is the base type of the type mark given in either the subtype
indication or the qualified expression. For an allocator with a subtype indication, the initial value of the
created object is the same as the default initial value for an explicitly declared variable of the designated
subtype. For an allocator with a qualified expression, this expression defines the initial value of the created
object.
The type of the access value returned by an allocator shall be determinable solely from the context, but using
the fact that the value returned is of an access type having the named designated type.
The only allowed form of constraint in the subtype indication of an allocator is an array constraint or a
record constraint. If an allocator includes a subtype indication and if the type of the object created is an array
type or a record type, then the subtype indication shall denote a fully constrained subtype. A subtype
indication that is part of an allocator shall not include a resolution indication.
If the type of the created object is an array type or a record type, then the created object is always fully
constrained. If the allocator includes a subtype indication, the created object is constrained by the subtype. If
the allocator includes a qualified expression, the created object is constrained by the bounds of the initial
value defined by that expression. For other types, the subtype of the created object is the subtype defined by
the subtype of the access type definition.
When a generic map aspect is present, the subtype indication shall denote an uninstantiated protected type
declared in a protected type declaration. The generic map aspect, in this case, optionally associates a single
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
171
Copyright © 2019 IEEE. All rights reserved.
actual with each formal generic (or member thereof) in the corresponding protected type declaration. Each
formal generic (or member thereof) shall be associated at most once. The generic map aspect is described in
6.5.7.2.
Each allocator, the subtype indication of which is an uninstantiated protected type, is preceded in the
declarative region that immediately encloses the allocator by an implicit declaration of the protected type
instance defined by the subtype indication and the generic map aspect, if any, of the allocator. The type of
the allocator is the implicitly declared protected type. The implicitly declared protected type does not have a
simple name.
It is an error if the subtype for an allocator denotes an uninstantiated protected type and a formal generic
does not get a value through either association or a default value. It is an error if a generic map aspect is
present and the subtype indication is not an uninstantiated protected type.
For the evaluation of an allocator, the elaboration of the subtype indication or the evaluation of the qualified
expression is first performed. The new object is then created, and the object is then assigned its initial value.
Finally, an access value that designates the created object is returned.
Any object created by the evaluation of an allocator shall remain allocated for as long as this object or one of
its subelements is accessible directly or indirectly; that is, as long as it can be denoted by some name.
An implementation shall deallocate the storage occupied by an object created by an allocator, once this
object is no longer designated by any access value. Deallocation may be delayed by the implementation.
Examples:
new NODE                        -- Takes on default initial value.
new NODE'(15 ns, null)          -- Initial value is specified.
new NODE'(Delay => 5  ns,
          \Next\=> Stack)       -- Initial value is specified.
new BIT_VECTOR'("00110110")     -- Constrained by initial value.
new STRING (1 to 10)            -- Constrained by index constraint.
new STRING                      -- Illegal: required to be constrained.
-- Example using a generic map aspect with new:
process
  -- Using types defined in 5.4.1
    variable SB_Ptr_slv     : SB_Ptr_Type_slv ;
    variable SB_Generic_Ptr : SB_Generic_Ptr_Type ;
    variable SB_Ptr_int     : SB_Ptr_Type_int ;
  begin
    . . .
    SB_Ptr_slv := new ScoreBoardPType_slv ;

    SB_Generic_Ptr := new ScoreBoardGenericPType generic map (
            ExpectedType    => std_logic_vector,
            ActualType      => std_logic_vector,
            check           => std_match
        ) ;

    SB_Ptr_slv := new SB_Ptr_Type_int'DESIGNATED_SUBTYPE ;
    . . .
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
172
Copyright © 2019 IEEE. All rights reserved.
### 9.4 Static expressions

#### 9.4.1 General

Certain expressions are said to be static. Similarly, certain discrete ranges are said to be static, and the type
marks of certain subtypes are said to denote static subtypes.
There are two categories of static expression. Certain forms of expression can be evaluated during the
analysis of the design unit in which they appear; such an expression is said to be locally static. Certain forms
of expression can be evaluated as soon as the design hierarchy in which they appear is elaborated; such an
expression is said to be globally static.
#### 9.4.2 Locally static primaries

An expression is said to be locally static if and only if every operator in the expression denotes an implicitly
defined operator or an operator defined in one of the packages STD_LOGIC_1164, NUMERIC_BIT,
NUMERIC_STD, NUMERIC_BIT_UNSIGNED, or NUMERIC_STD_UNSIGNED in library IEEE, and if
every primary in the expression is a locally static primary, where a locally static primary is defined to be one
of the following:
a)
A literal of any type other than type TIME
b)
A constant (other than a deferred constant) explicitly declared by a constant declaration with a
locally static subtype or with an unconstrained or partially constrained composite subtype for which
the applicable constraints are locally static, and initialized with a locally static expression
c)
A formal generic constant of a generic-mapped subprogram or package (whether explicitly declared
or equivalent to a subprogram or package instance, respectively), declared with a locally static
subtype and for which the associated actual is a locally static expression
d)
An alias whose aliased name (given in the corresponding alias declaration) is a locally static primary
and for which the subtype with which the aliased object is viewed is a locally static subtype
e)
A function call whose function name denotes an implicitly defined operation or an operation defined
in
one
of
the
packages
STD_LOGIC_1164,
NUMERIC_BIT,
NUMERIC_STD,
NUMERIC_BIT_UNSIGNED, or NUMERIC_STD_UNSIGNED in library IEEE and whose actual
parameters are each locally static expressions
f)
A predefined attribute that is a value, other than the predefined attributes 'INSTANCE_NAME and
'PATH_NAME, and whose prefix is either a locally static subtype or is an object name that is of a
locally static subtype
g)
A predefined attribute that is a function, other than the predefined attribute 'VALUE with a prefix
whose base type is the predefined type TIME, and other than the predefined attributes 'EVENT,
'ACTIVE,
'LAST_EVENT,
'LAST_ACTIVE,
'LAST_VALUE,
'DRIVING,
and
'DRIVING_VALUE, whose prefix is either a locally static subtype or is an object that is of a locally
static subtype, and whose actual parameter (if any) is a locally static expression and other than the
predefined attribute 'REFLECT.
h)
A user-defined attribute whose value is defined by a locally static expression
i)
A qualified expression whose type mark denotes a locally static subtype and whose operand is a
locally static expression
j)
A type conversion whose type mark denotes a locally static subtype and whose expression is a
locally static expression
k)
A locally static expression enclosed in parentheses
l)
An array aggregate in which all expressions in element associations are locally static expressions, all
simple expressions in choices are locally static expressions, all discrete ranges in choices are locally
static discrete ranges, and the others choice, if present, is locally static
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
173
Copyright © 2019 IEEE. All rights reserved.
m)
A record aggregate in which all expressions in element associations are locally static expressions
n)
An indexed name whose prefix is a locally static primary and whose index expressions are all
locally static expressions
o)
A slice name whose prefix is a locally static primary and whose discrete range is a locally static
discrete range
p)
A selected name whose prefix is a locally static primary
A locally static range is either a range of the second form (see 5.2.1) whose bounds are locally static
expressions, or a range of the first form whose prefix denotes either a locally static subtype or an object that
is of a locally static subtype or a range of the third form the range expression of which is a locally static
expression. A locally static range constraint is a range constraint whose range is locally static. A locally
static scalar subtype is either a scalar base type or a scalar subtype formed by imposing on a locally static
subtype a locally static range constraint. A locally static discrete range is either a locally static subtype or a
locally static range.
A locally static index constraint is an index constraint for which each index subtype of the corresponding
array type is locally static and in which each discrete range is locally static. A locally static array constraint
is an array constraint with a locally static index constraint and, if the array element constraint is present, a
locally static array element constraint. A locally static array subtype is a fully constrained array subtype
formed by imposing on an unconstrained array type a locally static array constraint. The unconstrained array
type shall have a locally static index subtype for each index position and a locally static index subtype for
each index position of each array subelement, if any. A locally static record constraint is a record constraint
with a locally static constraint in each record element constraint. A locally static record subtype is a fully
constrained record type whose elements are all of locally static subtypes, or a fully constrained record
subtype formed by imposing on an unconstrained record type a locally static record constraint. The
unconstrained record type shall have a locally static index subtype for each index position of each array
subelement, if any. A locally static access subtype is a subtype denoting an access type. A locally static file
subtype is a subtype denoting a file type. A locally static formal generic type is a formal generic type of an
explicit block statement or of a generic-mapped subprogram or package (whether explicitly declared or
equivalent to a subprogram or package instance, respectively) for which the associated actual is a locally
static subtype.
A locally static subtype is either a locally static scalar subtype, a locally static array subtype, a locally static
record subtype, a locally static access subtype, a locally static file subtype, or a locally static formal generic
type.
#### 9.4.3 Globally static primaries

An expression is said to be globally static if and only if every operator in the expression denotes a pure
function and every primary in the expression is a globally static primary, where a globally static primary is a
primary that, if it denotes an object or a function, does not denote a dynamically elaborated named entity
(see 14.6) and is one of the following:
a)
A literal of type TIME
b)
A locally static primary
c)
A generic constant declared with a globally static subtype
d)
A generate parameter
e)
A constant (including a deferred constant) explicitly declared by a constant declaration with a
globally static subtype or with an unconstrained or partially constrained composite subtype for
which the applicable constraints are globally static
f)
An alias whose aliased name (given in the corresponding alias declaration) is a globally static
primary
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
174
Copyright © 2019 IEEE. All rights reserved.
g)
An array aggregate, if and only if
1)
All expressions in its element associations are globally static expressions, and
2)
All ranges in its element associations are globally static ranges
h)
A record aggregate, if and only if all expressions in its element associations are globally static
expressions
i)
A function call whose function name denotes a pure function and whose actual parameters are each
globally static expressions
j)
A predefined attribute that is one of 'SIMPLE_NAME, 'INSTANCE_NAME, or 'PATH_NAME
k)
A predefined attribute that is a value, other than the predefined attributes 'SIMPLE_NAME,
'INSTANCE_NAME, and 'PATH_NAME, whose prefix is appropriate for a globally static attribute
l)
A predefined attribute that is a function, other than the predefined attributes 'EVENT, 'ACTIVE,
'LAST_EVENT, 'LAST_ACTIVE, 'LAST_VALUE, 'DRIVING, and 'DRIVING_VALUE, whose
prefix is appropriate for a globally static attribute, and whose actual parameter (if any) is a globally
static expression and other than the predefined attribute 'REFLECT
m)
A user-defined attribute whose value is defined by a globally static expression
n)
A qualified expression whose type mark denotes a globally static subtype and whose operand is a
globally static expression
o)
A type conversion whose type mark denotes a globally static subtype and whose expression is a
globally static expression
p)
An allocator of the first form (see 9.3.7) whose subtype indication denotes a globally static subtype
q)
An allocator of the second form whose qualified expression is a globally static expression
r)
A globally static expression enclosed in parentheses
s)
A subelement or a slice of a globally static primary, provided that any index expressions are globally
static expressions and any discrete ranges used in slice names are globally static discrete ranges
A prefix is appropriate for a globally static attribute if it denotes a signal, a constant, a type or subtype, a
globally static function call, a variable that is not of an access type, or a variable of an access type whose
designated subtype is fully constrained.
A globally static range is either a range of the second form (see 5.2.1) whose bounds are globally static
expressions, or a range of the first form whose prefix is appropriate for a globally static attribute, or a range
of the third form the range expression of which is a globally static expression. A globally static range
constraint is a range constraint whose range is globally static. A globally static scalar subtype is either a
scalar base type or a scalar subtype formed by imposing on a globally static subtype a globally static range
constraint. A globally static discrete range is either a globally static subtype or a globally static range.
A globally static index constraint is an index constraint for which each index subtype of the corresponding
array type is globally static and in which each discrete range is globally static. A globally static array
constraint is an array constraint with a globally static index constraint and, if the array element constraint is
present, a globally static array element constraint. A globally static array subtype is a fully constrained array
subtype formed by imposing on an unconstrained array type a globally static array constraint. A globally
static record constraint is a record constraint with a globally static constraint in each record element
constraint. A globally static record subtype is a fully constrained record type whose elements are all of
globally static subtypes, or a fully constrained record subtype formed by imposing on an unconstrained
record type a globally static record constraint. A globally static access subtype is a subtype denoting an
access type. A globally static file subtype is a subtype denoting a file type. A globally static formal generic
type is a formal generic type of a block statement (including an implied block statement representing a
component instance or a bound design entity) or of a generic-mapped subprogram or package (whether
explicitly declared or equivalent to a subprogram or package instance, respectively) for which the associated
actual is a globally static subtype.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
175
Copyright © 2019 IEEE. All rights reserved.
A globally static subtype is either a globally static scalar subtype, a globally static array subtype, a globally
static record subtype, a globally static access subtype, a globally static file subtype, or a globally static
formal generic type.
NOTE 1—An expression that is required to be a static expression shall either be a locally static expression or a globally
static expression. Similarly, a range, a range constraint, a scalar subtype, a discrete range, an index constraint, an array
constraint, an array subtype, a record constraint, or a record subtype that is required to be static shall either be locally
static or globally static.
NOTE 2—The rules for globally static expressions imply that a declared constant or a generic may be initialized with an
expression that is not globally static, for example, with a call to an impure function. The resulting constant value may be
globally static, even though its initial value expression is not. Only interface constant, variable, and signal declarations
require that their initial value expressions be static expressions.
### 9.5 Universal expressions

A universal_expression is either an expression that delivers a result of type universal_integer or one that
delivers a result of type universal_real.
The same operations are predefined for the type universal_integer as for any integer type. The same
operations are predefined for the type universal_real as for any floating-point type. In addition, these
operations include the following multiplication and division operators:
The accuracy of the evaluation of a universal expression of type universal_real is at least as good as the
accuracy of evaluation of expressions of the most precise predefined floating-point type supported by the
implementation, apart from universal_real itself.
For the evaluation of an operation of a universal expression, the following rules apply. If the result is of type
universal_integer, then the values of the operands and the result shall lie within the range of the integer type
with the widest range provided by the implementation, excluding type universal_integer itself. If the result
is of type universal_real, then the values of the operands and the result shall lie within the range of the
floating-point type with the widest range provided by the implementation, excluding type universal_real
itself.
NOTE—The predefined operators for the universal types are declared in package STANDARD as shown in 16.3.
Operator
Operation
Left operand type
Right operand
type
Result type
*
Multiplication
Universal real
Universal integer
Universal real
Universal integer
Universal real
Universal real
/
Division
Universal real
Universal integer
Universal real
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
