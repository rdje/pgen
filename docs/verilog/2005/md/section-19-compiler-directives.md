---
title: "Section 19: Compiler directives"
document: "Verilog Hardware Description Language Reference Manual"
standard: "IEEE 1364-2005"
domain: "Verilog"
section: "19"
source_txt: "section-19-compiler-directives.txt"
source_pdf: "/Users/richarddje/Documents/github/Verilog-LRM-IEEE-1364-2005.pdf"
---

# Section 19: Compiler directives

IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
349
## 19. Compiler directives

All Verilog compiler directives are preceded by the (`) character. This character is called grave accent
(ASCII 0x60). It is different from the character ('), which is the apostrophe character (ASCII 0x27). The
scope of a compiler directive extends from the point where it is processed, across all files processed, to the
point where another compiler directive supersedes it or the processing completes.
This clause describes the following compiler directives:
`begin_keywords
[19.11]
`celldefine
[19.1]
`default_nettype
[19.2]
`define
[19.3]
`else
[19.4]
`elsif
[19.4]
`end_keywords
[19.11]
`endcelldefine
[19.1]
`endif
[19.4]
`ifdef
[19.4]
`ifndef
[19.4]
`include
[19.5]
`line
[19.7]
`nounconnected_drive
[19.9]
`pragma
[19.10]
`resetall
[19.6]
`timescale
[19.8]
`unconnected_drive
[19.9]
`undef
[19.3]
### 19.1 `celldefine and `endcelldefine

The directives `celldefine and `endcelldefine tag modules as cell modules. Cells are used by certain PLI
routines for applications, such as delay calculations. It is advisable to pair each `celldefine with an
`endcelldefine, but it is not required. The latest occurrence of either directive in the source controls whether
modules are tagged as cell modules. More than one of these pairs may appear in a single source description.
These directives may appear anywhere in the source description, but it is recommended that the directives be
specified outside the module definition.
The `resetall directive includes the effects of a `endcelldefine directive.
### 19.2 `default_nettype

The directive `default_nettype controls the net type created for implicit net declarations (see 4.5). It can be
used only outside of module definitions. Multiple `default_nettype directives are allowed. The latest
occurrence of this directive in the source controls the type of nets that will be implicitly declared. Syntax 19-
## 1 contains the syntax of the directive.

When no `default_nettype directive is present or if the `resetall directive is specified, implicit nets are of
type wire. When the `default_nettype is set to none, all nets shall be explicitly declared. If a net is not
explicitly declared, an error is generated.
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
350
Copyright © 2006 IEEE. All rights reserved.
### 19.3 `define and `undef

A text macro substitution facility has been provided so that meaningful names can be used to represent
commonly used pieces of text. For example, in the situation where a constant number is repetitively used
throughout a description, a text macro would be useful in that only one place in the source description would
need to be altered if the value of the constant needed to be changed.
The text macro facility is not affected by the compiler directive `resetall.
#### 19.3.1 `define

The directive `define creates a macro for text substitution. This directive can be used both inside and outside
module definitions. After a text macro is defined, it can be used in the source description by using the (`)
character, followed by the macro name. The compiler shall substitute the text of the macro for the string
`text_macro_name and any actual arguments that follow it. All compiler directives shall be considered
predefined macro names; it shall be illegal to redefine a compiler directive as a macro name.
A text macro can be defined with arguments. This allows the macro to be customized for each use
individually.
The syntax for text macro definitions is given in Syntax 19-2.
Syntax 19-2—Syntax for text macro definition
The macro text can be any arbitrary text specified on the same line as the text macro name. If more than one
line is necessary to specify the text, the newline shall be preceded by a backslash (\). The first newline not
preceded by a backslash shall end the macro text. The newline preceded by a backslash shall be replaced in
the expanded macro with a newline (but without the preceding backslash character).
When formal arguments are used to define a text macro, the scope of the formal argument shall extend up to
the end of the macro text. A formal argument can be used in the macro text in the same manner as an
identifier.
```ebnf
default_nettype_compiler_directive ::=
```

`default_nettype default_nettype_value
```ebnf
default_nettype_value ::= wire | tri | tri0 | tri1 | wand | triand | wor | trior | trireg |
```

uwire | none
Syntax 19-1—Syntax for default_nettype compiler directive
```ebnf
text_macro_definition ::=
```

`define text_macro_name macro_text
```ebnf
text_macro_name ::=
```

text_macro_identifier [ ( list_of_formal_arguments ) ]
```ebnf
list_of_formal_arguments ::=
```

formal_argument_identifier { ,  formal_argument_identifier }
```ebnf
formal_argument_identifier ::=
```

simple_identifier
```ebnf
text_macro_identifier ::= (From A.9.3)
```

identifier
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
351
If formal arguments are used, the list of formal argument names shall be enclosed in parentheses following
the name of the macro. The formal argument names shall be simple_identifiers, separated by commas and
optionally whitespace. The left parenthesis shall follow the text macro name immediately, with no space in
between.
If a one-line comment (that is, a comment specified with the characters //) is included in the text, then the
comment shall not become part of the substituted text. The macro text can be blank, in which case the text
macro is defined to be empty and no text is substituted when the macro is used.
The syntax for using a text macro is given in Syntax 19-3.
Syntax 19-3—Syntax for text macro usage
For a macro without arguments, the text shall be substituted as is for every occurrence of
`text_macro_name. However, a text macro with one or more arguments shall be expanded by substituting
each formal argument with the expression used as the actual argument in the macro usage.
To use a macro defined with arguments, the name of the text macro shall be followed by a list of actual
arguments in parentheses, separated by commas. White space shall be allowed between the text macro name
and the left parenthesis. The number of actual arguments shall match the number of formal arguments.
Once a text macro name has been defined, it can be used anywhere in a source description; that is, there are
no scope restrictions. Text macros can be defined and used interactively.
The text specified for macro text shall not be split across the following lexical tokens:
—
Comments
—
Numbers
—
Strings
—
Identifiers
—
Keywords
—
Operators
For example:
`define wordsize 8
reg [1:`wordsize] data;
//define a nand with variable delay
`define var_nand(dly) nand #dly
`var_nand(2) g121 (q21, n10, n11);
`var_nand(5) g122 (q22, n10, n11);
The following is illegal syntax because it is split across a string:
```ebnf
text_macro_usage ::=
```

`text_macro_identifier [ ( list_of_actual_arguments ) ]
```ebnf
list_of_actual_arguments ::=
```

actual_argument { , actual_argument }
```ebnf
actual_argument ::=
```

expression
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
352
Copyright © 2006 IEEE. All rights reserved.
`define first_half "start of string
$display(`first_half end of string");
Each actual argument is substituted for the corresponding formal argument literally. Therefore, when an
expression is used as an actual argument, the expression will be substituted in its entirety. This may cause an
expression to be evaluated more than once if the formal argument was used more than once in the macro
text. For example:
`define max(a,b)((a) > (b) ? (a) : (b))
n = `max(p+q, r+s) ;
will expand as
n = ((p+q) > (r+s)) ? (p+q) : (r+s) ;
Here, the larger of the two expressions p + q and r + s will be evaluated twice.
The word define is known as a compiler directive keyword, and it is not part of the normal set of keywords.
Thus, normal identifiers in a Verilog HDL source description can be the same as compiler directive
keywords (although this is not recommended). The following problems should be considered:
a)
Text macro names may not be the same as compiler directive keywords.
b)
Text macro names can reuse names being used as ordinary identifiers. For example, signal_name
and `signal_name are different.
c)
Redefinition of text macros is allowed; the latest definition of a particular text macro read by the
compiler prevails when the macro name is encountered in the source text.
The macro text can contain usages of other text macros. Such usages shall be substituted after the original
macro is substituted, not when it is defined. It shall be an error for a macro to expand directly or indirectly to
text containing another usage of itself (a recursive macro).
#### 19.3.2 `undef

The directive `undef shall undefine a previously defined text macro. An attempt to undefine a text macro
that was not previously defined using a `define compiler directive can result in a warning. The syntax for
`undef compiler directive is given in Syntax 19-4.
Syntax 19-4—Syntax for undef compiler directive
An undefined text macro has no value, just as if it had never been defined.
### 19.4 `ifdef, `else, `elsif, `endif, `ifndef

These conditional compilation compiler directives are used to include optionally lines of a Verilog HDL
source description during compilation. The `ifdef compiler directive checks for the definition of a text_
macro_name. If the text_macro_name is defined, then the lines following the `ifdef directive are
included. If the text_macro_name is not defined and an `else directive exists, then this source is compiled.
The `ifndef compiler directive checks for the definition of a text_macro_name. If the text_macro_name
```ebnf
undefine_compiler_directive ::=
```

`undef  text_macro_identifier
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
353
is not defined, then the lines following the `ifndef directive are included. If the text_macro_name is
defined and an `else directive exists, then this source is compiled.
If the `elsif directive exists (instead of the `else), the compiler checks for the definition of the
text_macro_name. If the name exists, the lines following the `elsif directive are included. The `elsif
directive is equivalent to the compiler directive sequence `else `ifdef ... `endif. This directive does not need
a corresponding `endif directive. This directive shall be preceded by an `ifdef or `ifndef directive.
These directives may appear anywhere in the source description.
Situations where the `ifdef, `else, `elsif, `endif, and `ifndef compiler directives may be useful include the
following:
—
Selecting different representations of a module such as behavioral, structural, or switch level
—
Choosing different timing or structural information
—
Selecting different stimulus for a given run
The `ifdef, `else, `elsif, `endif, and `ifndef compiler directives have the syntax shown in Syntax 19-5.
Syntax 19-5—Syntax for conditional compilation directives
The text_macro_identifier is a Verilog HDL identifier. The ifdef_group_of_lines, ifndef_
group_of_lines, elsif_group_of_lines, and the else_group_of_lines are parts of a Verilog
HDL source description. The `else and `elsif compiler directives and all of the groups of lines are optional.
The `ifdef, `else, `elsif, and `endif compiler directives work together in the following manner:
—
When an `ifdef is encountered, the `ifdef text macro identifier is tested to see whether it is defined as
a text macro name using `define within the Verilog HDL source description.
—
If the `ifdef text macro identifier is defined, the `ifdef group of lines is compiled as part of the
description; and if there are `else or `elsif compiler directives, these compiler directives and
corresponding groups of lines are ignored.
—
If the `ifdef text macro identifier has not been defined, the `ifdef group of lines is ignored.
—
If there is an `elsif compiler directive, the `elsif text macro identifier is tested to see whether it is
defined as a text macro name using `define within the Verilog HDL source description.
```ebnf
conditional_compilation_directive ::=
```

ifdef_directive
| ifndef_directive
```ebnf
ifdef_directive ::=
```

`ifdef text_macro_identifier
ifdef_group_of_lines
{ `elsif text_macro_identifier elsif_group_of_lines }
[ `else else_group_of_lines ]
`endif
```ebnf
ifndef_directive ::=
```

`ifndef text_macro_identifier
ifndef_group_of_lines
{ `elsif text_macro_identifier elsif_group_of_lines }
[ `else else_group_of_lines ]
`endif
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
354
Copyright © 2006 IEEE. All rights reserved.
—
If the `elsif text macro identifier is defined, the `elsif group of lines is compiled as part of the
description; and if there are other `elsif or `else compiler directives, the other `elsif or `else
directives and corresponding groups of lines are ignored.
—
If the first `elsif text macro identifier has not been defined, the first `elsif group of lines is ignored.
—
If there are multiple `elsif compiler directives, they are evaluated like the first `elsif compiler
directive in the order they are written in the Verilog HDL source description.
—
If there is an `else compiler directive, the `else group of lines is compiled as part of the description.
The `ifndef, `else, `elsif, and `endif compiler directives work together in the following manner:
—
When an `ifndef is encountered, the `ifndef text macro identifier is tested to see whether it is defined
as a text macro name using `define within the Verilog HDL source description.
—
If the `ifndef text macro identifier is not defined, the `ifndef group of lines is compiled as part of the
description; and if there are `else or `elsif compiler directives, these compiler directives and
corresponding groups of lines are ignored.
—
If the `ifndef text macro identifier is defined, the `ifndef group of lines is ignored.
—
If there is an `elsif compiler directive, the `elsif text macro identifier is tested to see whether it is
defined as a text macro name using `define within the Verilog HDL source description.
—
If the `elsif text macro identifier is defined, the `elsif group of lines is compiled as part of the
description; and if there are other `elsif or `else compiler directives, the other `elsif or `else
directives and corresponding groups of lines are ignored.
—
If the first `elsif text macro identifier has not been defined, the first `elsif group of lines is ignored.
—
If there are multiple `elsif compiler directives, they are evaluated like the first `elsif compiler
directive in the order they are written in the Verilog HDL source description.
—
If there is an `else compiler directive, the `else group of lines is compiled as part of the description.
Although the names of compiler directives are contained in the same name space as text macro names, the
names of compiler directives are considered not to be defined by `ifdef, `ifndef, and `elseif.
Nesting of  `ifdef, `ifndef, `else, `elsif, and `endif compiler directives shall be permitted.
Any group of lines that the compiler ignores shall still follow the Verilog HDL lexical conventions for white
space, comments, numbers, strings, identifiers, keywords, and operators.
For example:
Example 1—The example below shows a simple usage of an `ifdef directive for conditional compilation. If
the identifier behavioral is defined, a continuous net assignment will be compiled in; otherwise, an and
gate will be instantiated.
module and_op (a, b, c);
output a;
input b, c;
`ifdef behavioral
wire a = b & c;
`else
and a1 (a,b,c);
`endif
endmodule
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
355
Example 2—The following example shows usage of nested conditional compilation directive:
module test(out);
output out;
`define wow
`define nest_one
`define second_nest
`define nest_two
`ifdef wow
initial $display("wow is defined");
`ifdef nest_one
initial $display("nest_one is defined");
`ifdef nest_two
initial $display("nest_two is defined");
`else
initial $display("nest_two is not defined");
`endif
`else
initial $display("nest_one is not defined");
`endif
`else
initial $display("wow is not defined");
`ifdef second_nest
initial $display("second_nest is defined");
`else
initial $display("second_nest is not defined");
`endif
`endif
endmodule
Example 3—The following example shows usage of chained nested conditional compilation directives:
module test;
`ifdef first_block
`ifndef second_nest
initial $display("first_block is defined");
`else
initial $display("first_block and second_nest defined");
`endif
`elsif second_block
initial $display("second_block defined, first_block is not");
`else
`ifndef last_result
initial $display("first_block, second_block,"
" last_result not defined.");
`elsif real_last
initial $display("first_block, second_block not defined,"
" last_result and real_last defined.");
`else
initial $display("Only last_result defined!");
`endif
`endif
endmodule
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
356
Copyright © 2006 IEEE. All rights reserved.
### 19.5 `include

The file inclusion (`include) compiler directive is used to insert the entire contents of a source file in another
file during compilation. The result is as though the contents of the included source file appear in place of the
`include compiler directive. The `include compiler directive can be used to include global or commonly
used definitions and tasks without encapsulating repeated code within module boundaries.
Advantages of using the `include compiler directive include the following:
—
Providing an integral part of configuration management
—
Improving the organization of Verilog HDL source descriptions
—
Facilitating the maintenance of Verilog HDL source descriptions
The syntax for the `include compiler directive is given in Syntax 19-6.
The compiler directive `include can be specified anywhere within the Verilog HDL description. The
filename is the name of the file to be included in the source file. The filename can be a full or relative path
name.
Only white space or a comment may appear on the same line as the `include compiler directive.
A file included in the source using the `include compiler directive may contain other `include compiler
directives. The number of nesting levels for include files shall be finite.
For example:
Examples of `include compiler directives are as follows:
`include "parts/count.v"
`include "fileB"
`include "fileB" // including fileB
Implementations may limit the maximum number of levels to which include files can be nested, but the limit
shall be at least 15.
### 19.6 `resetall

When `resetall compiler directive is encountered during compilation, all compiler directives are set to the
default values. This is useful for ensuring that only directives that are desired in compiling a particular
source file are active.
The recommended usage is to place `resetall at the beginning of each source text file, followed immediately
by the directives desired in the file.
It shall be illegal for the `resetall directive to be specified within a module or UDP declaration.
```ebnf
include_compiler_directive ::=
```

`include "filename"
Syntax 19-6—Syntax for include compiler directive
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
357
### 19.7 `line

It is important for Verilog tools to keep track of the filenames of the Verilog source files and the line
numbers in the files. This information can be used for error messages or source code debugging and can be
accessed by the Verilog PLI.
In many cases, however, the Verilog source is preprocessed by some other tool, and the line and file
information of the original source file can be lost because the preprocessor might add additional lines to the
source code file, combine multiple source code lines into one line, concatenate multiple source files, and so
on.
The `line compiler directive can be used to specify the original source code line number and filename. This
allows the location in the original file to be maintained if another process modifies the source. After the
newline number and filename are specified, the compiler can correctly refer to the original source location.
However, a tool is not required to produce `line directives. These directives are not intended to be inserted
manually into the code, although they can be.
The compiler shall maintain the current line number and filename of the file being compiled. The `line
directive shall set the line number and filename of the following line to those specified in the directive. The
directive can be specified anywhere within the Verilog HDL source description. However, only white space
may appear on the same line as the `line directive. Comments are not allowed on the same line as a `line
directive. All parameters in the `line directive are required. The results of this directive are not affected by
the `resetall directive.
The syntax for the `line compiler directive is given in Syntax 19-7.
Syntax 19-7—Syntax for line compiler directive
The number parameter shall be a positive integer that specifies the newline number of the following text
line. The filename parameter shall be a string constant that is treated as the new name of the file. The
filename can also be a full or relative path name. The level parameter shall be 0, 1, or 2. The value 1
indicates that the following line is the first line after an include file has been entered. The value 2 indicates
that the following line is the first line after an include file has been exited. The value 0 indicates any other
line.
For example:
`line 3 "orig.v" 2
// This line is line 3 of orig.v after exiting include file
As the compiler processes the remainder of the file and new files, the line number shall be incremented as
each line is read, and the name shall be updated to the new current file being processed. The line number
shall be reset to 1 at the beginning of each file. When beginning to read include files, the current line and
filename shall be stored for restoration at the termination of the include file. The updated line number and
filename information shall be available for PLI access. The mechanism of library searching is not affected
by the effects of the `line compiler directive.
```ebnf
line_compiler_directive ::=
```

`line number "filename" level
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
358
Copyright © 2006 IEEE. All rights reserved.
### 19.8 `timescale

This directive specifies the time unit and time precision of the modules that follow it. The time unit is the
unit of measurement for time values such as the simulation time and delay values.
To use modules with different time units in the same design, the following timescale constructs are useful:
—
The `timescale compiler directive to specify the unit of measurement for time and precision of time
in the modules in the design
—
The $printtimescale system task to display the time unit and precision of a module
—
The $time and $realtime system functions, the $timeformat system task, and the %t format
specification to specify how time information is reported
The `timescale compiler directive specifies the unit of measurement for time and delay values and the
degree of accuracy for delays in all modules that follow this directive until another `timescale compiler
directive is read. If there is no `timescale specified or it has been reset by a `resetall directive, the time unit
and precision are simulator-specific. It shall be an error if some modules have a `timescale specified and
others do not.
The syntax for the `timescale directive is given in Syntax 19-8.
Syntax 19-8—Syntax for timescale compiler directive
The time_unit argument specifies the unit of measurement for times and delays.
The time_precision argument specifies how delay values are rounded before being used in simulation.
The values used are accurate to within the unit of time specified here, even if there is a smaller
time_precision argument elsewhere in the design. The smallest time_precision argument of all the
`timescale compiler directives in the design determines the precision of the time unit of the simulation.
The time_precision argument shall be at least as precise as the time_unit argument; it cannot specify a
longer unit of time than time_unit.
The integers in these arguments specify an order of magnitude for the size of the value; the valid integers are
1, 10, and 100. The character strings represent units of measurement; the valid character strings are s, ms,
us, ns, ps, and fs.
The units of measurement specified by these character strings are given in Table 19-1.
NOTE—While s, ms, ns, ps and fs are the usual SI unit symbols for second, millisecond, nanosecond, picosecond and
femtosecond, due to lack of the Greek letter μ (mu) in coding character sets, “us” represents the SI unit symbol for
microsecond, properly ms.
```ebnf
timescale_compiler_directive ::=
```

`timescale time_unit / time_precision
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
359
For example:
The following example shows how this directive is used:
`timescale 1 ns / 1 ps
Here, all time values in the modules that follow the directive are multiples of 1 ns because the time_unit
argument is “1 ns.” Delays are rounded to real numbers with three decimal places—or precise to within one
thousandth of a nanosecond—because the time_precision argument is “1 ps,” or one thousandth of a
nanosecond.
Consider the following example:
`timescale 10 us / 100 ns
The time values in the modules that follow this directive are multiples of 10 us because the time_unit
argument is “10 us.” Delays are rounded to within one tenth of a microsecond because the
time_precision argument is “100 ns,” or one tenth of a microsecond.
The following example shows a `timescale directive in the context of a module:
`timescale 10 ns / 1 ns
module test;
reg set;
parameter d = 1.55;
initial begin
#d set = 0;
#d set = 1;
end
endmodule
The `timescale 10 ns / 1 ns compiler directive specifies that the time unit for module test is 10 ns. As a
result, the time values in the module are multiples of 10 ns, rounded to the nearest 1 ns; therefore, the value
stored in parameter d is scaled to a delay of 16 ns. In other words, the value 0 is assigned to reg set at
simulation time 16 ns (1.6 × 10 ns), and the value 1 at simulation time 32 ns.
Parameter d retains its value no matter what timescale is in effect.
Table 19-1—Arguments of time_precision
Character string
Unit of measurement
s
seconds
ms
milliseconds
us
microseconds
ns
nanoseconds
ps
picoseconds
fs
femtoseconds
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
360
Copyright © 2006 IEEE. All rights reserved.
These simulation times are determined as follows:
a)
The value of parameter d is rounded from 1.55 to 1.6 according to the time precision.
b)
The time unit of the module is 10 ns, and the precision is 1 ns; therefore, the delay of parameter d is
scaled from 1.6 to 16.
c)
The assignment of 0 to reg set is scheduled at simulation time 16 ns, and the assignment of 1 at
simulation time 32 ns. The time values are not rounded when the assignments are scheduled.
### 19.9 `unconnected_drive and `nounconnected_drive

All unconnected input ports of a module appearing between the directives `unconnected_drive and
`nounconnected_drive are pulled up or pulled down instead of the normal default.
The directive `unconnected_drive takes one of two arguments—pull1 or pull0. When pull1 is specified, all
unconnected input ports are automatically pulled up. When pull0 is specified, unconnected ports are pulled
down. It is advisable to pair each `unconnected_drive with a `nounconnected_drive, but it is not required.
The latest occurrence of either directive in the source controls what happens to unconnected ports. These
directives shall be specified in pairs outside of the module declarations.
The `resetall directive includes the effects of a `nounconnected_drive directive.
### 19.10 `pragma

The `pragma directive is a structured specification that alters interpretation of the Verilog source. The
specification introduced by this directive is referred to as a pragma. The effect of pragmas other than those
specified in this standard is implementation-specified. The syntax for the `pragma directive is given in
Syntax 19-9.
Syntax 19-9—Syntax for pragma compiler directive
The pragma specification is identified by the pragma_name, which follows the `pragma directive. The
pragma_name is followed by an optional list of pragma_expressions, which qualify the altered
interpretation indicated by the pragma_name. Unless otherwise specified, pragma directives for
pragma_names that are not recognized by an implementation shall have no effect on interpretation of the
Verilog source text.
```ebnf
pragma ::=
```

`pragma pragma_name [ pragma_expression { , pragma_expression } ]
```ebnf
pragma_name ::= simple_identifier
pragma_expression ::=
  pragma_keyword
| pragma_keyword = pragma_value
| pragma_value
pragma_value ::=
  ( pragma_expression { , pragma_expression } )
| number
| string
| identifier
pragma_keyword ::= simple_identifier
```

Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
361
#### 19.10.1 Standard pragmas

The reset and resetall pragmas shall restore the default values and state of pragma_keywords associated
with the affected pragmas. These default values shall be the values that the tool defines before any Verilog
text has been processed. The reset pragma shall reset the state for all pragma_names that appear as
pragma_keywords in the directive. The resetall pragma shall reset the state of all pragma_names recognized
by the implementation.
### 19.11 `begin_keywords, `end_keywords

A pair of directives, `begin_keywords and `end_keywords, can be used to specify what identifiers are
reserved as keywords within a block of source code, based on a specific version of IEEE Std 1364. The
`begin_keywords and `end_keywords directives only specify the set of identifiers that are reserved as
keywords. The directives do not affect the semantics, tokens, and other aspects of the Verilog language.
The syntax of the `begin_keywords and `end_keywords directives is in Syntax 19-10.
Syntax 19-10—Syntax for begin keywords and end keywords compiler directives
Implementations and other standards are permitted to extend the `begin_keywords directive with custom
version specifiers. It shall be an error if an implementation does not recognize the version_specifier used
with the `begin_keywords directive.
The `begin_keywords and `end_keywords directives can only be specified outside of a design element
(module, primitive, or configuration). The `begin_keywords directive affects all source code that
follow the directive, even across source code file boundaries, until the matching `end_keywords directive is
encountered.
Each `begin_keywords directive must be paired with an `end_keywords directive. The pair of directives
define a region of source code to which a specified version_specifier applies.
The `begin_keywords...`end_keywords directive pair can be nested. Each nested pair is stacked so that
when an `end_keywords directive is encountered, the implementation returns to using the version_ specifier
that was in effect prior to the matching `begin_keywords directive.
If no `begin_keywords directive is specified, then the reserved keyword list shall be the implementation’s
default set of keywords. The default set of reserved keywords used by an implementation shall be
implementation dependent. For example, an implementation based on IEEE Std 1364-2005 would most
likely use the 1364-2005 set of reserved keywords as its default, whereas an implementation based on IEEE
Std 1364-2001 would most likely use the 1364-2001 set of reserved keywords as its default.
Implementations may provide other mechanisms for specifying the set of reserved keywords to be used as
the default. One possible use model might be for an implementation to use invocation options to specify its
default set of reserved keywords. Another possible use model might be the use of source file name
extensions for determining a default set of reserved keywords to be used for each source file.
```ebnf
keywords_directive ::= `begin_keywords "version_specifier"
version_specifier ::=
 1364-1995
| 1364-2001
| 1364-2001-noconfig
| 1364-2005
endkeywords_directive ::= `end_keywords
```

Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
362
Copyright © 2006 IEEE. All rights reserved.
The version_specifier "1364-1995" specifies that only the identifiers listed as reserved keywords in IEEE
Std 1364-1995 are considered to be reserved words. These identifiers are listed in Table 19-2.
Table 19-2—IEEE 1364-1995 reserved keywords
always
and
assign
begin
buf
bufif0
bufif1
case
casex
casez
cmos
deassign
default
defparam
disable
edge
else
end
endcase
endmodule
endfunction
endprimitive
endspecify
endtable
endtask
event
for
force
forever
fork
function
highz0
highz1
if
ifnone
initial
inout
input
integer
join
large
macromodule
medium
module
nand
negedge
nmos
nor
not
notif0
notif1
or
output
parameter
pmos
posedge
primitive
pull0
pull1
pullup
pulldown
rcmos
real
realtime
reg
release
repeat
rnmos
rpmos
rtran
rtranif0
rtranif1
scalared
small
specify
specparam
strong0
strong1
supply0
supply1
table
task
time
tran
tranif0
tranif1
tri
tri0
tri1
triand
trior
trireg
vectored
wait
wand
weak0
weak1
while
wire
wor
xnor
xor
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
363
The version_specifier "1364-2001" specifies that only the identifiers listed as reserved keywords in IEEE
Std 1364-2001 are considered to be reserved words. These identifiers are listed in Table 19-3.
Table 19-3—IEEE 1364-2001 reserved keywords
The version_specifier "1364-2001-noconfig" behaves similarly to the "1364-2001" version_specifier,
with the exception that the following identifiers are excluded from the reserved list in Table 19-3:
cell
config
design
endconfig
incdir
include
instance
liblist
library
use
Because these identifiers are not reserved when using the "1364-2001-noconfig" version_specifier, they
may be used as normal Verilog identifiers within the corresponding `begin_keywords...`end_ keywords
region.
always
and
assign
automatic
begin
buf
bufif0
bufif1
case
casex
casez
cell
cmos
config
deassign
default
defparam
design
disable
edge
else
end
endcase
endconfig
endfunction
endgenerate
endmodule
endprimitive
endspecify
endtable
endtask
event
for
force
forever
fork
function
generate
genvar
highz0
highz1
if
ifnone
incdir
include
initial
inout
input
instance
integer
join
large
liblist
library
localparam
macromodule
medium
module
nand
negedge
nmos
nor
noshowcancelled
not
notif0
notif1
or
output
parameter
pmos
posedge
primitive
pull0
pull1
pulldown
pullup
pulsestyle_onevent
pulsestyle_ondetect
rcmos
real
realtime
reg
release
repeat
rnmos
rpmos
rtran
rtranif0
rtranif1
scalared
showcancelled
signed
small
specify
specparam
strong0
strong1
supply0
supply1
table
task
time
tran
tranif0
tranif1
tri
tri0
tri1
triand
trior
trireg
unsigned
use
vectored
wait
wand
weak0
weak1
while
wire
wor
xnor
xor
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
364
Copyright © 2006 IEEE. All rights reserved.
The version_specifier "1364-2005" specifies that only the identifiers listed as reserved keywords in IEEE
Std 1364-2005 are considered to be reserved words. These identifiers are listed in Table 19-4.
Table 19-4—IEEE 1364-2005 reserved keywords
In the example below, it is assumed that the definition of module m1 does not have a `begin_keywords
directive specified prior to the module definition. Without this directive, the set of reserved keywords in
effect for this module shall be the implementation’s default set of reserved keywords.
module m1;
// module definition with no ‘begin_keywords directive
...
endmodule
The following example specifies a `begin_keywords "1364-2001" directive. The source code within the
module uses the identifier uwire as a net name. The `begin_keywords directive would be necessary in this
example if an implementation uses IEEE Std 1364-2005 as its default set of keywords because uwire is a
reserved keyword in this standard. Specifying that the "1364-1995" Verilog keyword lists should be used
would also work with this example.
‘begin_keywords "1364-2001"
// use IEEE Std 1364-2001 Verilog keywords
module m2 (...);
wire [63:0] uwire;
// OK: "uwire" is not a keyword in 1364-2001
...
endmodule
‘end_keywords
always
and
assign
automatic
begin
buf
bufif0
bufif1
case
casex
casez
cell
cmos
config
deassign
default
defparam
design
disable
edge
else
end
endcase
endconfig
endfunction
endgenerate
endmodule
endprimitive
endspecify
endtable
endtask
event
for
force
forever
fork
function
generate
genvar
highz0
highz1
if
ifnone
incdir
include
initial
inout
input
instance
integer
join
large
liblist
library
localparam
macromodule
medium
module
nand
negedge
nmos
nor
noshowcancelled
not
notif0
notif1
or
output
parameter
pmos
posedge
primitive
pull0
pull1
pulldown
pullup
pulsestyle_onevent
pulsestyle_ondetect
rcmos
real
realtime
reg
release
repeat
rnmos
rpmos
rtran
rtranif0
rtranif1
scalared
showcancelled
signed
small
specify
specparam
strong0
strong1
supply0
supply1
table
task
time
tran
tranif0
tranif1
tri
tri0
tri1
triand
trior
trireg
unsigned
use
uwire
vectored
wait
wand
weak0
weak1
while
wire
wor
xnor
xor
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
365
The next example is the same code as the previous example, except that it explicitly specifies that the IEEE
Std 1364-2005 Verilog keywords should be used. This example shall result in an error because uwire is
reserved as a keyword in this standard.
‘begin_keywords "1364-2005"
// use IEEE Std 1364-2005 Verilog keywords
module m2 (...);
wire [63:0] uwire;
// ERROR: "uwire" is a keyword in 1364-2005
...
endmodule
‘end_keywords
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
