---
title: "Section 32: IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language"
document: "SystemVerilog Language Reference Manual"
standard: "IEEE 1800-2017"
domain: "SystemVerilog"
section: "32"
source_txt: "section-32-h-12ab-f001.txt"
source_pdf: "/Users/richarddje/Documents/github/SystemVerilog-LRM-IEEE-1800-2017.pdf"
---

# Section 32: IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language

IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
75
Copyright © 2018 IEEE. All rights reserved.
b = 'h 3x;
// yields 03x
c = 'h z3;
// yields zz3
d = 'h 0z3;
// yields 0z3
e = 'h5;
// yields {82{1'b0},3'b101}
f = 'hx;
// yields {85{1'hx}}
g = 'hz;
// yields {85{1'hz}}
end
Example 5: Automatic left padding of constant literal numbers using a single-bit value
logic [15:0] a, b, c, d;
a = '0;
// sets all 16 bits to 0
b = '1;
// sets all 16 bits to 1
c = 'x;
// sets all 16 bits to x
d = 'z;
// sets all 16 bits to z
Example 6: Underscores in literal constant numbers
27_195_000
// unsized decimal 27195000
16'b0011_0101_0001_1111
// 16-bit binary number
## 32 'h 12ab_f001

// 32-bit hexadecimal number
Sized negative literal constant numbers and sized signed literal constant numbers are sign-extended when
assigned to a data object of type logic, regardless of whether the type itself is signed.
The default length of x and z is the same as the default length of an integer.
#### 5.7.2 Real literal constants

The real literal constant numbers shall be represented as described by IEEE Std 754, an IEEE standard for
double-precision floating-point numbers.
Real numbers can be specified in either decimal notation (for example, 14.72) or in scientific notation (for
example, 39e8, which indicates 39 multiplied by 10 to the eighth power). Real numbers expressed with a
decimal point shall have at least one digit on each side of the decimal point.
For example:
1.2
0.1
2394.26331
1.2E12 (the exponent symbol can be e or E)
1.30e-2
0.1e-0
23E10
29E-2
236.123_763_e-12 (underscores are ignored)
The following are invalid forms of real numbers because they do not have at least one digit on each side of
the decimal point:
.12
9.
4.E3
.2e-7
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
76
Copyright © 2018 IEEE. All rights reserved.
The default type for fixed-point format (e.g., 1.2), and exponent format (e.g., 2.0e10) shall be real.
A cast can be used to convert literal real values to the shortreal type (e.g., shortreal'(1.2)). Casting
is described in 6.24.
Real numbers shall be converted to integers by rounding the real number to the nearest integer, rather than
by truncating it. Implicit conversion shall take place when a real number is assigned to an integer. The ties
shall be rounded away from zero. For example:
—
The real numbers 35.7 and 35.5 both become 36 when converted to an integer and 35.2 becomes 35.
—
Converting –1.5 to integer yields –2, converting 1.5 to integer yields 2.
### 5.8 Time literals

Time is written in integer or fixed-point format, followed without a space by a time unit (fs ps ns us ms s).
For example:
2.1ns
40ps
The time literal is interpreted as a realtime value scaled to the current time unit and rounded to the current
time precision.
### 5.9 String literals

A string literal is a sequence of characters enclosed by double quotes ("").
Nonprinting and other special characters are preceded with a backslash.
A string literal shall be contained in a single line unless the newline character is immediately preceded by a
\ (backslash). In this case, the backslash and the newline character are ignored. There is no predefined limit
to the length of a string literal.
Example 1:
$display("Humpty Dumpty sat on a wall. \
Humpty Dumpty had a great fall.");
prints
Humpty Dumpty sat on a wall. Humpty Dumpty had a great fall.
Example 2:
$display("Humpty Dumpty sat on a wall.\n\
Humpty Dumpty had a great fall.");
prints
Humpty Dumpty sat on a wall.
Humpty Dumpty had a great fall.
String literals used as operands in expressions and assignments shall be treated as unsigned integer constants
represented by a sequence of 8-bit ASCII values, with one 8-bit ASCII value representing one character.
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
77
Copyright © 2018 IEEE. All rights reserved.
A string literal can be assigned to an integral type, such as a packed array. If the size differs, it is right
justified. To fully store a string literal, the integral type should be declared with a width equal to the number
of characters in the string multiplied by 8. For example:
byte c1 = "A" ;
bit [7:0] d = "\n" ;
The rules of SystemVerilog assignments shall be followed if the packed array width does not match the
number of characters multiplied by 8. When an integral type is larger than required to hold the string literal
value being assigned, the value is right-justified, and the leftmost bits are padded with zeros, as is done with
nonstring values. If a string literal is larger than the destination integral type, the string is right-justified, and
the leftmost characters are truncated.
For example, to store the 12-character string "Hello world\n" requires a variable 8  12, or 96 bits wide.
bit [8*12:1] stringvar = "Hello world\n";
Alternatively, a multidimensional packed array can be used, with 8-bit subfields, as in:
bit [0:11] [7:0] stringvar = "Hello world\n" ;
A string literal can be assigned to an unpacked array of bytes. If the size differs, it is left justified.
byte c3 [0:12] = "hello world\n" ;
Packed and unpacked arrays are discussed in 7.4.
String literals can also be cast to a packed or unpacked array type, which shall follow the same rules as
assigning a string literal to a packed or unpacked array. Casting is discussed in 6.24.
SystemVerilog also includes a string data type to which a string literal can be assigned. Variables of type
string have arbitrary length; they are dynamically resized to hold any string. String literals are packed
arrays (of a width that is a multiple of 8 bits), and they are implicitly converted to the string type when
assigned to a string type or used in an expression involving string type operands (see 6.16).
String literals stored in vectors can be manipulated using the SystemVerilog operators. The value being
manipulated by the operator is the sequence of 8-bit ASCII values. See 11.10 for operations on string
literals.
#### 5.9.1 Special characters in strings

Certain characters can only be used in string literals when preceded by an introductory character called an
escape character. Table 5-1 lists these characters in the right-hand column, with the escape sequence that
represents the character in the left-hand column.
Table 5-1—Specifying special characters in string literals
Escape string
Character produced by escape string
\n
Newline character
\t
Tab character
\\
\ character
\"
" character
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
78
Copyright © 2018 IEEE. All rights reserved.
### 5.10 Structure literals

Structure literals are structure assignment patterns or pattern expressions with constant member expressions
(see 10.9.2). A structure literal must have a type, which may be either explicitly indicated with a prefix or
implicitly indicated by an assignment-like context (see 10.8).
typedef struct {int a; shortreal b;} ab;
ab c;
c = '{0, 0.0}; // structure literal type determined from
// the left-hand context (c)
Nested braces shall reflect the structure. For example:
ab abarr[1:0] = '{'{1, 1.0}, '{2, 2.0}};
The C-like alternative '{1, 1.0, 2, 2.0} for the preceding example is not allowed.
Structure literals can also use member name and value or use data type and default value (see 10.9.2):
c = '{a:0, b:0.0};
// member name and value for that member
c = '{default:0};
// all elements of structure c are set to 0
d = ab'{int:1, shortreal:1.0};
// data type and default value for all
// members of that type
When an array of structures is initialized, the nested braces shall reflect the array and the structure. For
example:
ab abarr[1:0] = '{'{1, 1.0}, '{2, 2.0}};
Replication operators can be used to set the values for the exact number of members. The inner pair of
braces in a replication is removed.
struct {int X,Y,Z;} XYZ = '{3{1}};
typedef struct {int a,b[4];} ab_t;
int a,b,c;
ab_t v1[1:0] [2:0];
v1 = '{2{'{3{'{a,'{2{b,c}}}}}}};
/* expands to '{ '{3{ '{ a, '{2{ b, c }} } }},
\v
vertical tab
\f
form feed
\a
bell
\ddd
A character specified in 1 to 3 octal_digits (see Syntax 5-2). If fewer than three
characters are used, the following character shall not be an octal_digit. Implementations
may issue an error if the character represented is greater than \377. It shall be illegal for
an octal_digit in an escape sequence to be an x_digit or a z_digit (see Syntax 5-2).
\xdd
A character specified in 1 to 2 hex_digits (see Syntax 5-2). If only one digit is used, the
following character shall not be a hex_digit. It shall be illegal for a hex_digit in an
escape sequence to be an x_digit or a z_digit (see Syntax 5-2).
Table 5-1—Specifying special characters in string literals  (continued)
Escape string
Character produced by escape string
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
79
Copyright © 2018 IEEE. All rights reserved.
'{3{ '{ a, '{2{ b, c }} } }}
} */
/* expands to '{ '{ '{ a, '{2{ b, c }} },
'{ a, '{2{ b, c }} },
'{ a, '{2{ b, c }} }
},
'{ '{ a, '{2{ b, c }} },
'{ a, '{2{ b, c }} },
'{ a, '{2{ b, c }} }
}
} */
/* expands to '{ '{ '{ a, '{ b, c, b, c } },
'{ a, '{ b, c, b, c } },
'{ a, '{ b, c, b, c } }
},
'{ '{ a, '{ b, c, b, c } },
'{ a, '{ b, c, b, c } },
'{ a, '{ b, c, b, c } }
}
} */
### 5.11 Array literals

Array literals are syntactically similar to C initializers, but with the replication operator ( {{}} ) allowed.
int n[1:2][1:3] = '{'{0,1,2},'{3{4}}};
The nesting of braces shall follow the number of dimensions, unlike in C. However, replication operators
can be nested. The inner pair of braces in a replication is removed. A replication expression only operates
within one dimension.
int n[1:2][1:6] = '{2{'{3{4, 5}}}}; // same as
'{'{4,5,4,5,4,5},'{4,5,4,5,4,5}}
Array literals are array assignment patterns or pattern expressions with constant member expressions (see
10.9.1). An array literal must have a type, which may be either explicitly indicated with a prefix or implicitly
indicated by an assignment-like context (see 10.8).
typedef int triple [1:3];
$mydisplay(triple'{0,1,2});
Array literals can also use their index or type as a key and use a default key value (see 10.9.1).
triple b = '{1:1, default:0};
// indices 2 and 3 assigned 0
### 5.12 Attributes

A mechanism is included for specifying properties about objects, statements, and groups of statements in the
SystemVerilog source that can be used by various tools, including simulators, to control the operation or
behavior of the tool. These properties are referred to as attributes. This subclause specifies the syntactic
mechanism used for specifying attributes, without standardizing on any particular attributes.
The syntax for specifying an attribute is shown in Syntax 5-3.
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
80
Copyright © 2018 IEEE. All rights reserved.
```ebnf
attribute_instance ::= (* attr_spec { , attr_spec } *)
```

// from A.9.1
```ebnf
attr_spec ::= attr_name [ = constant_expression ]
attr_name ::= identifier
```

Syntax 5-3—Syntax for attributes (excerpt from Annex A)
An attribute_instance can appear in the SystemVerilog description as a prefix attached to a declaration, a
module item, a statement, or a port connection. It can appear as a suffix to an operator or a function name in
an expression.
The default type of an attribute with no value is bit, with a value of 1. Otherwise, the attribute takes the
type of the expression.
If the same attribute name is defined more than once for the same language element, the last attribute value
shall be used, and a tool can issue a warning that a duplicate attribute specification has occurred.
Nesting of attribute instances is disallowed. It shall be illegal to specify the value of an attribute with a
constant expression that contains an attribute instance.
Refer to Annex A for the syntax of specifying an attribute instance on specific language elements. Several
examples are illustrated below.
Example 1: The following example shows how to attach attributes to a case statement:
(* full_case, parallel_case *)
case (a)
<rest_of_case_statement>
or
(* full_case=1 *)
(* parallel_case=1 *) // Multiple attribute instances also OK
case (a)
<rest_of_case_statement>
or
(* full_case, // no value assigned
parallel_case=1 *)
case (a)
<rest_of_case_statement>
Example 2: To attach the full_case attribute, but not the parallel_case attribute:
(* full_case *) // parallel_case not specified
case (a)
<rest_of_case_statement>
or
(* full_case=1, parallel_case = 0 *)
case (a)
<rest_of_case_statement>
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
81
Copyright © 2018 IEEE. All rights reserved.
Example 3: To attach an attribute to a module definition:
(* optimize_power *)
module mod1 (<port_list>);
or
(* optimize_power=1 *)
module mod1 (<port_list>);
Example 4: To attach an attribute to a module instantiation:
(* optimize_power=0 *)
mod1 synth1 (<port_list>);
Example 5: To attach an attribute to a variable declaration:
(* fsm_state *) logic [7:0] state1;
(* fsm_state=1 *) logic [3:0] state2, state3;
logic [3:0] reg1;
// reg1 does NOT have fsm_state set
(* fsm_state=0 *) logic [3:0] reg2; // nor does reg2
Example 6: To attach an attribute to an operator:
a = b + (* mode = "cla" *) c;
// sets the value for the attribute mode
// to be the string cla.
Example 7: To attach an attribute to a function call:
a = add (* mode = "cla" *) (b, c);
Example 8: To attach an attribute to a conditional operator:
a = b ? (* no_glitch *) c : d;
### 5.13 Built-in methods

SystemVerilog uses a C++ -like class method calling syntax, in which a subroutine is called using the dot
notation (.):
object.task_or_function()
The object uniquely identifies the data on which the subroutine operates. Hence, the method concept is
naturally extended to built-in types in order to add functionality, which traditionally was done via system
tasks or system functions. Unlike system tasks, built-in methods are not prefixed with a $ because they
require no special prefix to avoid collisions with user-defined identifiers. Thus, the method syntax allows
extending the language without the addition of new keywords or the cluttering of the global name space with
system tasks.
Built-in methods, unlike system tasks, cannot be redefined by users via PLI tasks. Thus, only functions that
users should not be allowed to redefine are good candidates for built-in method calls.
In general, a built-in method is preferred over a system task when a particular functionality applies to all
data types or when it applies to a specific data type. For example:
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
82
Copyright © 2018 IEEE. All rights reserved.
dynamic_array.size, associative_array.num, and string.len
These are all similar concepts, but they represent different things. A dynamic array has a size, an associative
array contains a given number of items, and a string has a given length. Using the same system task, such as
$size, for all of them would be less clear and intuitive.
A built-in method can only be associated with a particular data type. Therefore, if some functionality is a
simple side effect (i.e., $stop or $reset) or it operates on no specific data (i.e., $random), then a system
task must be used.
When a subroutine built-in method call specifies no arguments, the empty parenthesis, (), following the
subroutine name is optional. This is also true for subroutines that require arguments, when all arguments
have defaults specified. For a method, this rule allows simple calls to appear as properties of the object or
built-in type. Similar rules are defined for subroutines in 13.5.5.
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
