---
title: "Section 3: Lexical conventions"
document: "Verilog Hardware Description Language Reference Manual"
standard: "IEEE 1364-2005"
domain: "Verilog"
section: "3"
source_txt: "section-3-lexical-conventions.txt"
source_pdf: "/Users/richarddje/Documents/github/Verilog-LRM-IEEE-1364-2005.pdf"
---

# Section 3: Lexical conventions

IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
8
Copyright © 2006 IEEE. All rights reserved.
## 3. Lexical conventions

This clause describes the lexical tokens used in Verilog HDL source text and their conventions.
### 3.1 Lexical tokens

Verilog HDL source text files shall be a stream of lexical tokens. A lexical token shall consist of one or more
characters. The layout of tokens in a source file shall be free format; that is, spaces and newlines shall not be
syntactically significant other than being token separators, except for escaped identifiers (see 3.7.1).
The types of lexical tokens in the language are as follows:
—
White space
—
Comment
—
Operator
—
Number
—
String
—
Identifier
—
Keyword
### 3.2 White space

White space shall contain the characters for spaces, tabs, newlines, and formfeeds. These characters shall be
ignored except when they serve to separate other lexical tokens. However, blanks and tabs shall be
considered significant characters in strings (see 3.6).
### 3.3 Comments

The Verilog HDL has two forms to introduce comments. A one-line comment shall start with the two
characters // and end with a newline. A block comment shall start with /* and end with */. Block
comments shall not be nested. The one-line comment token // shall not have any special meaning in a block
comment.
### 3.4 Operators

Operators are single-, double-, or triple-character sequences and are used in expressions. Clause 5 discusses
the use of operators in expressions.
Unary operators shall appear to the left of their operand. Binary operators shall appear between their
operands. A conditional operator shall have two operator characters that separate three operands.
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
9
### 3.5 Numbers

Constant numbers can be specified as integer constants (defined in 3.5.1) or real constants.
Syntax 3-1—Syntax for integer and real numbers
```ebnf
number ::=  (From A.8.7)
```

decimal_number
| octal_number
| binary_number
| hex_number
| real_number
```ebnf
real_numbera ::=
```

unsigned_number . unsigned_number
| unsigned_number [ . unsigned_number ] exp [ sign ] unsigned_number
```ebnf
exp ::= e | E
decimal_number ::=
```

unsigned_number
| [ size ] decimal_base unsigned_number
| [ size ] decimal_base x_digit { _ }
| [ size ] decimal_base z_digit { _ }
```ebnf
binary_number ::=
```

[ size ] binary_base binary_value
```ebnf
octal_number ::=
```

[ size ] octal_base octal_value
```ebnf
hex_number ::=
```

[ size ] hex_base hex_value
```ebnf
sign ::= + | -
size ::= non_zero_unsigned_number
non_zero_unsigned_numbera ::= non_zero_decimal_digit { _ | decimal_digit}
unsigned_numbera ::= decimal_digit { _ | decimal_digit }
binary_valuea ::= binary_digit { _ | binary_digit }
octal_valuea ::= octal_digit { _ | octal_digit }
hex_valuea ::= hex_digit { _ | hex_digit }
decimal_basea ::= '[s|S]d | '[s|S]D
binary_basea ::= '[s|S]b |  '[s|S]B
octal_basea::= '[s|S]o | '[s|S]O
hex_basea ::= '[s|S]h | '[s|S]H
non_zero_decimal_digit ::= 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9
decimal_digit ::= 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9
binary_digit ::= x_digit | z_digit | 0 | 1
octal_digit ::= x_digit | z_digit | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7
hex_digit ::=
```

x_digit | z_digit | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9
| a | b | c | d | e | f | A | B | C | D | E | F
```ebnf
x_digit ::= x | X
z_digit ::= z | Z | ?
```

aEmbedded spaces are illegal.
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
10
Copyright © 2006 IEEE. All rights reserved.
#### 3.5.1 Integer constants

Integer constants can be specified in decimal, hexadecimal, octal, or binary format.
There are two forms to express integer constants. The first form is a simple decimal number, which shall be
specified as a sequence of digits 0 through 9, optionally starting with a plus or minus unary operator. The
second form specifies a based constant, which shall be composed of up to three tokens—an optional size
constant, an apostrophe character (', ASCII 0x27) followed by a base format character, and the digits
representing the value of the number. It shall be legal to macro-substitute these three tokens.
The first token, a size constant, shall specify the size of the constant in terms of its exact number of bits. It
shall be specified as a nonzero unsigned decimal number. For example, the size specification for two
hexadecimal digits is 8 because one hexadecimal digit requires 4 bits.
The second token, a base_format, shall consist of a case-insensitive letter specifying the base for the
number, optionally preceded by the single character s (or S) to indicate a signed quantity, preceded by the
apostrophe character. Legal base specifications are d, D, h, H, o, O, b, or B for the bases decimal,
hexadecimal, octal, and binary, respectively.
The apostrophe character and the base format character shall not be separated by any white space.
The third token, an unsigned number, shall consist of digits that are legal for the specified base format. The
unsigned number token shall immediately follow the base format, optionally preceded by white space. The
hexadecimal digits a to f shall be case insensitive.
Simple decimal numbers without the size and the base format shall be treated as signed integers, whereas the
numbers specified with the base format shall be treated as signed integers if the s designator is included or
as unsigned integers if the base format only is used. The s designator does not affect the bit pattern
specified, only its interpretation.
A plus or minus operator preceding the size constant is a unary plus or minus operator. A plus or minus
operator between the base format and the number is an illegal syntax.
Negative numbers shall be represented in twos-complement form.
An x represents the unknown value in hexadecimal, octal, and binary constants. A z represents the high-
impedance value. See 4.1 for a discussion of the Verilog HDL value set. An x shall set 4 bits to unknown in
the hexadecimal base, 3 bits in the octal base, and 1 bit in the binary base. Similarly, a z shall set 4 bits,
## 3 bits, and 1 bit, respectively, to the high-impedance value.

If the size of the unsigned number is smaller than the size specified for the constant, the unsigned number
shall be padded to the left with zeros. If the leftmost bit in the unsigned number is an x or a z, then an x or a
z shall be used to pad to the left, respectively. If the size of the unsigned number is larger than the size
specified for the constant, the unsigned number shall be truncated from the left.
The number of bits that make up an unsized number (which is a simple decimal number or a number without
the size specification) shall be at least 32. Unsized unsigned constants where the high-order bit is unknown
(X or x) or three-state (Z or z) shall be extended to the size of the expression containing the constant.
NOTE—In IEEE Std 1364-1995, in unsized constants where the high-order bit is unknown or three-state, the x or z was
only extended to 32 bits.
The use of x and z in defining the value of a number is case insensitive.
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
11
When used in a number, the question-mark (?) character is a Verilog HDL alternative for the z character. It
sets 4 bits to the high-impedance value in hexadecimal numbers, 3 bits in octal, and 1 bit in binary. The
question mark can be used to enhance readability in cases where the high-impedance value is a do-not-care
condition. See the discussion of casez and casex in 9.5.1. The question-mark character is also used in user-
defined primitive (UDP) state tables. See Table 8-1 in 8.1.6.
In a decimal constant, the unsigned number token shall not contain any x, z, or ? digits, unless there is
exactly one digit in the token, indicating that every bit in the decimal constant is x or z.
The underscore character (_) shall be legal anywhere in a number except as the first character. The
underscore character is ignored. This feature can be used to break up long numbers for readability purposes.
For example:
Example 1—Unsized constant numbers
659

// is a decimal number
'h 837FF // is a hexadecimal number
'o7460
// is an octal number
4af
// is illegal (hexadecimal format requires 'h)
Example 2—Sized constant numbers
4'b1001
// is a 4-bit binary number
## 5 'D 3

// is a 5-bit decimal number
3'b01x
// is a 3-bit number with the least
// significant bit unknown
12'hx
// is a 12-bit unknown number
16'hz
// is a 16-bit high-impedance number
Example 3—Using sign with constant numbers
## 8 'd -6

// this is illegal syntax
-8 'd 6
// this defines the two's complement of 6,
 // held in 8 bits—equivalent to -(8'd 6)
## 4 'shf

// this denotes the 4-bit number '1111', to
// be interpreted as a 2's complement number,
// or '-1'. This is equivalent to -4'h 1
-4 'sd15
// this is equivalent to -(-4'd 1), or '0001'
16'sd?
  // the same as 16'sbz
Example 4—Automatic left padding
reg [11:0] a, b, c, d;
initial begin
a = 'h x;
// yields xxx
b = 'h 3x;
// yields 03x
c = 'h z3;
// yields zz3
d = 'h 0z3;
// yields 0z3
end
reg [84:0]      e, f, g;
e = 'h5;        // yields {82{1'b0},3'b101}
f = 'hx;        // yields {85{1'hx}}
g = 'hz;        // yields {85{1'hz}}
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
12
Copyright © 2006 IEEE. All rights reserved.
Example 5—Using underscore character in numbers
27_195_000
16'b0011_0101_0001_1111
## 32 'h 12ab_f001

Sized negative constant numbers and sized signed constant numbers are sign-extended when assigned to a
reg data type, regardless of whether the reg itself is signed.
The default length of x and z is the same as the default length of an integer.
#### 3.5.2 Real constants

The real constant numbers shall be represented as described by IEEE Std 754-1985, an IEEE standard for
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
#### 3.5.3 Conversion

Real numbers shall be converted to integers by rounding the real number to the nearest integer, rather than
by truncating it. Implicit conversion shall take place when a real number is assigned to an integer. The ties
shall be rounded away from zero. For example:
—
The real numbers 35.7 and 35.5 both become 36 when converted to an integer and 35.2 becomes 35.
—
Converting –1.5 to integer yields –2, converting 1.5 to integer yields 2.
### 3.6 Strings

A string is a sequence of characters enclosed by double quotes ("") and contained on a single line. Strings
used as operands in expressions and assignments shall be treated as unsigned integer constants represented
by a sequence of 8-bit ASCII values, with one 8-bit ASCII value representing one character.
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
13
#### 3.6.1 String variable declaration

String variables are variables of reg type (see 4.2) with width equal to the number of characters in the string
multiplied by 8.
For example:
To store the 12-character string "Hello world!" requires a reg 8 * 12, or 96 bits wide.
reg [8*12:1] stringvar;
initial begin
 stringvar = "Hello world!";
end
#### 3.6.2 String manipulation

Strings can be manipulated using the Verilog HDL operators. The value being manipulated by the operator
is the sequence of 8-bit ASCII values.
For example:
module string_test;
reg [8*14:1] stringvar;
initial begin
stringvar = "Hello world";
$display("%s is stored as %h", stringvar,stringvar);
stringvar = {stringvar,"!!!"};
$display("%s is stored as %h", stringvar,stringvar);
end
endmodule
The output is as follows:
Hello world is stored as 00000048656c6c6f20776f726c64
Hello world!!! is stored as 48656c6c6f20776f726c64212121
When a variable is larger than required to hold a string value being assigned, the value is right-justified, and
the leftmost bits are .padded with zeros, as is done with nonstring values. If a string is larger than the
destination string variable, the string is right-justified, and the leftmost characters are truncated.
#### 3.6.3 Special characters in strings

Certain characters can only be used in strings when preceded by an introductory character called an escape
character. Table 3-1 lists these characters in the right-hand column, with the escape sequence that represents
the character in the left-hand column.
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
14
Copyright © 2006 IEEE. All rights reserved.
### 3.7 Identifiers, keywords, and system names

An identifier is used to give an object a unique name so it can be referenced. An identifier is either a simple
identifier or an escaped identifier (see 3.7.1). A simple identifier shall be any sequence of letters, digits,
dollar signs ($), and underscore characters (_).
The first character of a simple identifier shall not be a digit or $; it can be a letter or an underscore.
Identifiers shall be case sensitive.
For example:
shiftreg_a
busa_index
error_condition
merge_ab
_bus3
n$657
Implementations may set a limit on the maximum length of identifiers, but the limit shall be at least
## 1024 characters. If an identifier exceeds the implementation-specified length limit, an error shall be

reported.
#### 3.7.1 Escaped identifiers

Escaped identifiers shall start with the backslash character (\) and end with white space (space, tab,
newline). They provide a means of including any of the printable ASCII characters in an identifier (the
decimal values 33 through 126, or 21 through 7E in hexadecimal).
Neither the leading backslash character nor the terminating white space is considered to be part of the
identifier. Therefore, an escaped identifier \cpu3 is treated the same as a nonescaped identifier cpu3.
For example:
\busa+index
\-clock
\***error-condition***
\net1/\net2
\{a,b}
\a*(b+c)
Table 3-1—Specifying special characters in string
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
\ddd
A character specified in 1–3 octal digits (0 ≤ d ≤ 7).
If less than three characters are used, the following character shall not be an octal digit.
Implementations may issue an error if the character represented is greater than \377.
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
15
#### 3.7.2 Keywords

Keywords are predefined nonescaped identifiers that are used to define the language constructs. A Verilog
HDL keyword preceded by an escape character is not interpreted as a keyword.
All keywords are defined in lowercase only. Annex B gives a list of all defined keywords.
#### 3.7.3 System tasks and functions

The dollar sign ($) introduces a language construct that enables development of user-defined system tasks
and functions. System constructs are not design semantics, but refer to simulator functionality. A name
following the $ is interpreted as a system task or a system function.
The syntax for a system task/function is given in Syntax 3-2.
Syntax 3-2—Syntax for system tasks and functions
The $identifier system task/function can be defined in three places:
—
A standard set of $identifier system tasks and functions, as defined in Clause 17 and Clause 18.
—
Additional $identifier system tasks and functions defined using the PLI, as described in Clause 20.
—
Additional $identifier system tasks and functions defined by software implementations.
Any valid identifier, including keywords already in use in contexts other than this construct, can be used as a
system task/function name. The system tasks and functions described in Clause 17 and Clause 18 are part of
this standard. Additional system tasks and functions with the $identifier construct are not part of this
standard.
For example:
$display ("display a message");
$finish;
#### 3.7.4 Compiler directives

The ` character (the ASCII value 0x60, called grave accent) introduces a language construct used to
implement compiler directives. The compiler behavior dictated by a compiler directive shall take effect as
soon as the compiler reads the directive. The directive shall remain in effect for the rest of the compilation
unless a different compiler directive specifies otherwise. A compiler directive in one description file can,
therefore, control compilation behavior in multiple description files.
```ebnf
system_task_enable ::= (From A.6.9)
```

system_task_identifier [ ( [ expression ] { , [ expression ] } ) ] ;
```ebnf
system_function_call ::= (From A.8.2)
```

system_function_identifier [ ( expression { , expression } ) ]
```ebnf
system_function_identifiera ::= (From A.9.3)
```

$[ a-zA-Z0-9_$ ]{ [ a-zA-Z0-9_$ ] }
```ebnf
system_task_identifiera ::=
```

$[ a-zA-Z0-9_$ ]{ [ a-zA-Z0-9_$ ] }
aThe dollar sign ($) in a system_function_identifier or system_task_identifier shall not be followed by white
space. A system_function_identifier or system_task_identifier shall not be escaped.
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
16
Copyright © 2006 IEEE. All rights reserved.
The `identifier compiler directive construct can be defined in two places:
—
A standard set of `identifier compiler directives defined in Clause 19.
—
Additional `identifier compiler directives defined by software implementations.
Any valid identifier, including keywords already in use in contexts other than this construct, can be used as a
compiler directive name. The compiler directives described in Clause 19 are part of this standard. Additional
compiler directives with the `identifier construct are not part of this standard.
For example:
`define wordsize 8
### 3.8 Attributes

With the proliferation of tools other than simulators that use Verilog HDL as their source, a mechanism is
included for specifying properties about objects, statements, and groups of statements in the HDL source
that can be used by various tools, including simulators, to control the operation or behavior of the tool.
These properties shall be referred to as attributes. This subclause specifies the syntactic mechanism that
shall be used for specifying attributes, without standardizing on any particular attributes.
The syntax for specifying an attribute is shown in Syntax 3-3.
Syntax 3-3—Syntax for attributes
An attribute_instance can appear in the Verilog description as a prefix attached to a declaration, a
module item, a statement, or a port connection. It can appear as a suffix to an operator or a Verilog function
name in an expression.
If a value is not specifically assigned to the attribute, then its value shall be 1. If the same attribute name is
defined more than once for the same language element, the last attribute value shall be used; and a tool can
give a warning that a duplicate attribute specification has occurred.
Nesting of attribute instances is disallowed. It shall be illegal to specify the value of an attribute with a
constant expression that contains an attribute instance.
#### 3.8.1 Examples

Example 1—The following example shows how to attach attributes to a case statement:
(* full_case, parallel_case *)
case (foo)
<rest_of_case_statement>
or
```ebnf
attribute_instance ::= (From A.9.1)
```

(* attr_spec { , attr_spec } *)
```ebnf
attr_spec ::=
```

attr_name [ = constant_expression ]
```ebnf
attr_name ::=
```

identifier
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
17
(* full_case=1 *)
(* parallel_case=1 *) // Multiple attribute instances also OK
case (foo)
<rest_of_case_statement>
or
(* full_case, // no value assigned
parallel_case=1 *)
case (foo)
<rest_of_case_statement>
Example 2—To attach the full_case attribute, but not the parallel_case attribute:
(* full_case *) // parallel_case not specified
case (foo)
<rest_of_case_statement>
or
(* full_case=1, parallel_case = 0 *)
case (foo)
<rest_of_case_statement>
Example 3—To attach an attribute to a module definition:
(* optimize_power *)
module mod1 (<port_list>);
or
(* optimize_power=1 *)
module mod1 (<port_list>);
Example 4—To attach an attribute to a module instantiation:
(* optimize_power=0 *)
mod1 synth1 (<port_list>);
Example 5—To attach an attribute to a reg declaration:
(* fsm_state *) reg [7:0] state1;
(* fsm_state=1 *) reg [3:0] state2, state3;
reg [3:0] reg1;  // this reg does NOT have fsm_state set
(* fsm_state=0 *) reg [3:0] reg2;  // nor does this one
Example 6—To attach an attribute to an operator:
a = b + (* mode = "cla" *) c;
This sets the value for the attribute mode to be the string cla.
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
18
Copyright © 2006 IEEE. All rights reserved.
Example 7—To attach an attribute to a Verilog function call:
a = add (* mode = "cla" *) (b, c);
Example 8—To attach an attribute to a conditional operator:
a = b ? (* no_glitch *) c : d;
#### 3.8.2 Syntax

The syntax for legal statements with attributes is shown in Syntax 3-4 through Syntax 3-9.
The syntax for module declaration attributes is given in Syntax 3-4.
Syntax 3-4—Syntax for module declaration attributes
The syntax for port declaration attributes is given in Syntax 3-5.
Syntax 3-5—Syntax for port declaration attributes
```ebnf
module_declaration ::= (From A.1.2)
```

{ attribute_instance } module_keyword module_identifier
[ module_parameter_port_list ] list_of_ports ;
{ module_item }
endmodule
        | { attribute_instance } module_keyword module_identifier
[ module_parameter_port_list ] [ list_of_port_declarations ] ;
{ non_port_module_item }
endmodule
```ebnf
port_declaration ::= (From A.1.3)
```

{attribute_instance} inout_declaration
| {attribute_instance} input_declaration
| {attribute_instance} output_declaration
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
19
The syntax for module item attributes is given in Syntax 3-6.
Syntax 3-6—Syntax for module item attributes
The syntax for function port, task, and block attributes is given in Syntax 3-7.
Syntax 3-7—Syntax for function port, task, and block attributes
```ebnf
module_item ::= (From A.1.4)
  port_declaration ;
| non_port_module_item
module_or_generate_item ::=
  { attribute_instance } module_or_generate_item_declaration
| { attribute_instance } local_parameter_declaration ;
| { attribute_instance } parameter_override
| { attribute_instance } continuous_assign
| { attribute_instance } gate_instantiation
| { attribute_instance } udp_instantiation
| { attribute_instance } module_instantiation
| { attribute_instance } initial_construct
| { attribute_instance } always_construct
| { attribute_instance } loop_generate_construct
| { attribute_instance } conditional_generate_construct
non_port_module_item ::=
  module_or_generate_item
| generate_region
| specify_block
| { attribute_instance } parameter_declaration ;
| { attribute_instance } specparam_declaration
function_port_list ::= (From A.2.6)
```

{attribute_instance} input_declaration { , {attribute_instance } input_declaration}
```ebnf
task_item_declaration ::= (From A.2.7)
```

block_item_declaration
| { attribute_instance } input_declaration ;
| { attribute_instance } output_declaration ;
| { attribute_instance } inout_declaration ;
```ebnf
task_port_item ::=
```

{ attribute_instance } input_declaration
| { attribute_instance } output_declaration
| { attribute_instance } inout_declaration
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
20
Copyright © 2006 IEEE. All rights reserved.
The syntax for port connection attributes is given in Syntax 3-8.
Syntax 3-8—Syntax for port connection attributes
The syntax for udp attributes is given in Syntax 3-9.
Syntax 3-9—Syntax for udp attributes
```ebnf
ordered_port_connection ::= (From A.4.1)
```

{ attribute_instance } [ expression ]
```ebnf
named_port_connection ::=
```

{ attribute_instance } . port_identifier ( [ expression ] )
```ebnf
udp_declaration ::= (From A.5.1)
```

{ attribute_instance }  primitive udp_identifier ( udp_port_list ) ;
udp_port_declaration { udp_port_declaration }
udp_body
endprimitive
| { attribute_instance } primitive udp_identifier ( udp_declaration_port_list ) ;
udp_body
endprimitive
```ebnf
udp_output_declaration ::= (From A.5.2)
```

{ attribute_instance } output port_identifier
| { attribute_instance } output reg port_identifier [ = constant_expression ]
```ebnf
udp_input_declaration ::=
```

{ attribute_instance } input list_of_port_identifiers
```ebnf
udp_reg_declaration ::=
```

{ attribute_instance } reg variable_identifier
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
