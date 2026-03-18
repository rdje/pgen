---
title: "Section 12: Hierarchical structures"
document: "Verilog Hardware Description Language Reference Manual"
standard: "IEEE 1364-2005"
domain: "Verilog"
section: "12"
source_txt: "section-12-hierarchical-structures.txt"
source_pdf: "/Users/richarddje/Documents/github/Verilog-LRM-IEEE-1364-2005.pdf"
---

# Section 12: Hierarchical structures

IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
163
## 12. Hierarchical structures

The Verilog HDL supports a hierarchical hardware description structure by allowing modules to be
embedded within other modules. Higher level modules create instances of lower level modules and
communicate with them through input, output, and bidirectional ports. These module input/output (I/O)
ports can be scalar or vector.
As an example of a module hierarchy, consider a system consisting of printed circuit boards (PCBs). The
system would be represented as the top-level module and would create instances of modules that represent
the boards. The board modules would, in turn, create instances of modules that represent integrated circuits
(ICs), and the ICs could, in turn, create instances of modules such as flip-flops, muxes, and alus.
To describe a hierarchy of modules, the user provides textual definitions of the various modules. Each
module definition stands alone; the definitions are not nested. Statements within the module definitions
create instances of other modules, thus describing the hierarchy.
### 12.1 Modules

This subclause gives the formal syntax for a module definition and then gives the syntax for module
instantiation, along with an example of a module definition and a module instantiation.
A module definition shall be enclosed between the keywords module and endmodule. The identifier
following the keyword module shall be the name of the module being defined. The optional list of
parameter definitions shall specify an ordered list of the parameters for the module. The optional list of ports
or port declarations shall specify an ordered list of the ports for the module. The order used in defining the
list of parameters in the module_parameter_port_list and in the list of ports can be significant when
instantiating the module (see 12.2.2.1 and 12.3.5). The identifiers in this list shall be declared in input,
output, and inout statements within the module definition. Ports declared in the list of port declarations shall
not be redeclared within the body of the module. The module items define what constitutes a module, and
they include many different types of declarations and definitions, many of which have already been
introduced.
The keyword macromodule can be used interchangeably with the keyword module to define a module. An
implementation may choose to treat module definitions beginning with the macromodule keyword
differently.
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
164
Copyright © 2006 IEEE. All rights reserved.
Syntax 12-1—Syntax for module
```ebnf
module_declaration ::= (From A.1.2)
```

{ attribute_instance } module_keyword module_identifier [ module_parameter_port_list ]
list_of_ports ; { module_item }
endmodule
| { attribute_instance } module_keyword module_identifier [ module_parameter_port_list ]
[ list_of_port_declarations ] ; { non_port_module_item }
endmodule
```ebnf
module_keyword ::= module | macromodule
module_parameter_port_list ::= (From A.1.3
```

# ( parameter_declaration { , parameter_declaration } )
```ebnf
list_of_ports ::= ( port { , port } )
list_of_port_declarations ::= ( port_declaration { , port_declaration } ) | ( )
port ::= [ port_expression ] | . port_identifier ( [ port_expression ] )
port_expression ::= port_reference | { port_reference { , port_reference } }
port_reference ::= port_identifier [ [ constant_range_expression ] ]
port_declaration ::= {attribute_instance} inout_declaration
| {attribute_instance} input_declaration
| {attribute_instance} output_declaration
module_item ::=  (From A.1.4)
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
module_or_generate_item_declaration ::=
  net_declaration
| reg_declaration
| integer_declaration
| real_declaration
| time_declaration
| realtime_declaration
| event_declaration
| genvar_declaration
| task_declaration
| function_declaration
non_port_module_item ::=
  module_or_generate_item
| generate_region
| specify_block
| { attribute_instance } parameter_declaration ;
| { attribute_instance } specparam_declaration
parameter_override ::= defparam list_of_defparam_assignments ;
```

Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
165
See 12.3 for the definitions of ports.
#### 12.1.1 Top-level modules

Top-level modules are modules that are included in the source text, but do not appear in any module
instantiation statement, as described in 12.1.2. This applies even if the module instantiation appears in a
generate block that is not itself instantiated (see 12.4). A model shall contain at least one top-level module.
#### 12.1.2 Module instantiation

Instantiation allows one module to incorporate a copy of another module into itself. Module definitions do
not nest. In other words, one module definition shall not contain the text of another module definition within
its module-endmodule keyword pair. A module definition nests another module by instantiating it. The
module instantiation statement creates one or more named instances of a defined module.
For example, a counter module might instantiate a D flip-flop module to create multiple instances of the
flip-flop.
Syntax 12-2 gives the syntax for specifying instantiations of modules.
Syntax 12-2—Syntax for module instantiation
The instantiations of modules can contain a range specification. This allows an array of instances to be
created. The array of instances is described in 7.1. The syntax and semantics of arrays of instances defined
for gates and primitives apply for modules as well.
One or more module instances (identical copies of a module) can be specified in a single module
instantiation statement.
```ebnf
module_instantiation ::= (From A.4.1)
```

module_identifier [ parameter_value_assignment ]
module_instance { , module_instance } ;
```ebnf
parameter_value_assignment ::=
```

# ( list_of_parameter_assignments )
```ebnf
list_of_parameter_assignments ::=
```

ordered_parameter_assignment { , ordered_parameter_assignment }
| named_parameter_assignment   { , named_parameter_assignment }
```ebnf
ordered_parameter_assignment ::=
```

expression
```ebnf
named_parameter_assignment ::=
```

. parameter_identifier ( [ mintypmax_expression ] )
```ebnf
module_instance ::=
```

name_of_module_instance ( [ list_of_port_connections ] )
```ebnf
name_of_module_instance ::=
```

module_instance_identifier [ range ]
```ebnf
list_of_port_connections ::=
```

ordered_port_connection { , ordered_port_connection }
| named_port_connection { , named_port_connection }
```ebnf
ordered_port_connection ::=
```

{ attribute_instance } [ expression ]
```ebnf
named_port_connection ::=
```

{ attribute_instance } . port_identifier ( [ expression ] )
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
166
Copyright © 2006 IEEE. All rights reserved.
The list of port connections shall be provided only for modules defined with ports. The parentheses,
however, are always required. When a list of port connections is given using the ordered port connection
method, the first element in the list shall connect to the first port declared in the module, the second to the
second port, and so on. See 12.3 for a more detailed discussion of ports and port connection rules.
A connection can be a simple reference to a variable or a net identifier, an expression, or a blank. An
expression can be used for supplying a value to a module input port. A blank port connection shall represent
the situation where the port is not to be connected.
When connecting ports by name, an unconnected port can be indicated either by omitting it in the port list or
by providing no expression in the parentheses [i.e., .port_name ()].
For example:
Example 1—The following example illustrates a circuit (the lower level module) being driven by a simple
waveform description (the higher level module) where the circuit module is instantiated inside the waveform
module:
// Lower level module:
// module description of a nand flip-flop circuit
module ffnand (q, qbar, preset, clear);
output q, qbar;
//declares 2 circuit output nets
input preset, clear;
//declares 2 circuit input nets
// declaration of two nand gates and their interconnections
nand g1 (q, qbar, preset),
 g2 (qbar, q, clear);
endmodule
// Higher level module:
// a waveform description for the nand flip-flop
module ffnand_wave;
wire out1, out2;
//outputs from the circuit
reg in1, in2;
//variables to drive the circuit
parameter d = 10;
// instantiate the circuit ffnand, name it "ff",
// and specify the IO port interconnections
ffnand ff(out1, out2, in1, in2);
// define the waveform to stimulate the circuit
initial begin
#d in1 = 0; in2 = 1;
#d in1 = 1;
#d in2 = 0;
#d in2 = 1;
end
endmodule
Example 2—The following example creates two instances of the flip-flop module ffnand defined in
Example 1. It connects only to the q output in one instance and only to the qbar output in the other instance.
// a waveform description for testing
// the nand flip-flop, without the output ports
module ffnand_wave;
reg in1, in2; //variables to drive the circuit
parameter d = 10;
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
167
// make two copies of the circuit ffnand
// ff1 has qbar unconnected, ff2 has q unconnected
ffnand ff1(out1, , in1, in2),

ff2(.qbar(out2), .clear(in2), .preset(in1), .q());
// ff3(.q(out3),.clear(in1),,,); is illegal
// define the waveform to stimulate the circuit
initial begin
#d in1 = 0; in2 = 1;
#d in1 = 1;
#d in2 = 0;
#d in2 = 1;
end
endmodule
### 12.2 Overriding module parameter values

There are two different ways that parameters can be defined. The first is the module_parameter_port_list
(see 12.1), and the second is as a module_item (see 4.10). A module declaration can contain parameter
definitions of either or both types or can contain no parameter definitions.
A module parameter can have a type specification and a range specification. The effect of parameter
overrides on a parameter’s type and range shall be in accordance with the following rules:
—
A parameter declaration with no type or range specification shall default to the type and range of the
final override value assigned to the parameter.
—
A parameter with a range specification, but with no type specification, shall be the range of the
parameter declaration and shall be unsigned. An override value shall be converted to the type and
range of the parameter.
—
A parameter with a type specification, but with no range specification, shall be of the type specified.
An override value shall be converted to the type of the parameter. A signed parameter shall default
to the range of the final override value assigned to the parameter.
—
A parameter with a signed type specification and with a range specification shall be signed and shall
be the range of its declaration. An override value shall be converted to the type and range of the
parameter.
For example:
module generic_fifo
     #(parameter MSB=3, LSB=0, DEPTH=4)
                              //These parameters can be overridden
   (input [MSB:LSB] in,
    input clk, read, write, reset,
    output [MSB:LSB] out,
    output full, empty );

   localparam FIFO_MSB = DEPTH*MSB;
   localparam FIFO_LSB = LSB;
         // These parameters are local, and cannot be overridden.
         // They can be affected by altering the public parameters
         // above, and the module will work correctly.
   reg [FIFO_MSB:FIFO_LSB] fifo;
   reg [LOG2(DEPTH):0] depth;
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
168
Copyright © 2006 IEEE. All rights reserved.
   always @(posedge clk or reset) begin
    casex ({read,write,reset})
      // implementation of fifo
    endcase
   end
 endmodule
There are two ways to alter nonlocal parameter values: the defparam statement, which allows assignment to
parameters using their hierarchical names, and the module instance parameter value assignment, which
allows values to be assigned in line during module instantiation. If a defparam assignment conflicts with a
module instance parameter, the parameter in the module will take the value specified by the defparam. The
module instance parameter value assignment comes in two forms, by ordered list or by name. The next two
subclauses describe these two methods.
There are two kinds of parameter declarations. The first kind of parameter declaration has a type and/or
range qualification, and the second does not. When an untyped and unranged parameter’s value is
overridden, the parameter takes on the size and type of the override.
When a typed and/or ranged parameter is overridden, the new value is converted to the type and size of the
destination and assigned to that parameter.
For example:
module foo(a,b);
   real r1,r2;
   parameter [2:0] A = 3'h2;
   parameter B = 3'h2;
   initial begin
      r1 = A;
      r2 = B;
      $display("r1 is %f r2 is %f",r1,r2);
   end
endmodule // foo
module bar;
   wire a,b;
   defparam f1.A = 3.1415;
   defparam f1.B = 3.1415;
   foo f1(a,b);
endmodule // bar
Parameter A is a typed and/or ranged parameter; therefore, when its value is redefined, the parameter retains
its original type and sign. Therefore, the defparam of f1.A with the value 3.1415 is performed by
converting the floating point number 3.1415 into a fixed-point number 3, and then the low 3 bits of 3 are
assigned to A.
Parameter B is not a typed and/or ranged parameter; therefore, when its value is redefined, the parameter
type and range take on the type and range of the new value. Therefore, the defparam of f1.B with the value
### 3.1415 replaces B’s current value of 3'h2 with the floating point number 3.1415.

#### 12.2.1 defparam statement

Using the defparam statement, parameter values can be changed in any module instance throughout the
design using the hierarchical name of the parameter. See 12.5 for hierarchical names.
However, a defparam statement in a hierarchy in or under a generate block instance (see 12.4) or an array of
instances (see 7.1 and 12.1.2) shall not change a parameter value outside that hierarchy.
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
169
Each instantiation of a generate block is considered to be a separate hierarchy scope. Therefore, this rule
implies that a defparam statement in a generate block may not target a parameter in another instantiation of
the same generate block, even when the other instantiation is created by the same loop generate construct.
For example, the following code is not allowed:
genvar i;
generate
  for (i = 0; i < 8; i = i + 1) begin : somename
    flop my_flop(in[i], in1[i], out1[i]);
    defparam somename[i+1].my_flop.xyz = i ;
  end
endgenerate
Similarly, a defparam statement in one instance of an array of instances may not target a parameter in
another instance of the array.
The expression on the right-hand side of the defparam assignments shall be a constant expression involving
only numbers and references to parameters. The referenced parameters (on the right-hand side of the
defparam) shall be declared in the same module as the defparam statement.
The defparam statement is particularly useful for grouping all of the parameter value override assignments
together in one module.
In the case of multiple defparams for a single parameter, the parameter takes the value of the last defparam
statement encountered in the source text. When defparams are encountered in multiple source files, e.g.,
found by library searching, the defparam from which the parameter takes its value is undefined.
For example:
module top;
reg clk;
reg [0:4] in1;
reg [0:9] in2;
wire [0:4] o1;
wire [0:9] o2;
vdff m1 (o1, in1, clk);
vdff m2 (o2, in2, clk);
endmodule
module vdff (out, in, clk);
parameter size = 1, delay = 1;
input [0:size-1] in;
input clk;
output [0:size-1] out;
reg [0:size-1] out;
always @(posedge clk)
# delay out = in;
endmodule
module annotate;
defparam
top.m1.size = 5,
top.m1.delay = 10,
top.m2.size = 10,
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
170
Copyright © 2006 IEEE. All rights reserved.
top.m2.delay = 20;
endmodule
The module annotate has the defparam statement, which overrides size and delay parameter values for
instances m1 and m2 in the top-level module top. The modules top and annotate would both be
considered top-level modules.
#### 12.2.2 Module instance parameter value assignment

An alternative method for assigning values to parameters within module instances is to use one of the two
forms of module instance parameter value assignment. They are assignment by ordered list and assignment
by name. The two types of module instance parameter value assignment shall not be mixed; parameter
assignments to a particular module instance shall be entirely by order or entirely by name.
Module instance parameter value assignment by ordered list is similar in appearance to the assignment of
delay values to gate instances, and assignment by name is similar to connecting module ports by name. It
supplies values for particular instances of a module to any parameters that have been specified in the
definition of that module.
A parameter declared in a named block, task, or function can only be directly redefined using a defparam
statement. However, if the parameter value is dependent on a second parameter, then redefining the second
parameter will update the value of the first parameter as well (see 12.2.3).
##### 12.2.2.1 Parameter value assignment by ordered list

The order of the assignments in the module instance parameter value assignment by ordered list shall follow
the order of declaration of the parameters within the module. It is not necessary to assign values to all of the
parameters within a module when using this method. However, it is not possible to skip over a parameter.
Therefore, to assign values to a subset of the parameters declared within a module, the declarations of the
parameters that make up this subset shall precede the declarations of the remaining parameters. An
alternative is to assign values to all of the parameters, but to use the default value (the same value assigned
in the declaration of the parameter within the module definition) for those parameters that do not need new
values.
For example:
Consider the following example, where the parameters within module instances mod_a, mod_c, and mod_d
are changed during instantiation:
module tb1;
  wire [9:0] out_a, out_d;
  wire [4:0] out_b, out_c;
  reg  [9:0] in_a, in_d;
  reg  [4:0] in_b, in_c;
  reg        clk;
  // testbench clock & stimulus generation code ...

  // Four instances of vdff with parameter value assignment
  // by ordered list
  // mod_a has new parameter values size=10 and delay=15
  // mod_b has default parameters (size=5, delay=1)
  // mod_c has one default size=5 and one new delay=12
  //   In order to change the value of delay,
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
171
  //   it is necessary to specify the (default) value of size as well.
  // mod_d has a new parameter value size=10.
  //   delay retains its default value
  vdff #(10,15) mod_a (.out(out_a), .in(in_a), .clk(clk));
  vdff          mod_b (.out(out_b), .in(in_b), .clk(clk));
  vdff #( 5,12) mod_c (.out(out_c), .in(in_c), .clk(clk));
  vdff #(10)    mod_d (.out(out_d), .in(in_d), .clk(clk));
endmodule
module vdff (out, in, clk);
  parameter size=5, delay=1;
  output [size-1:0] out;
  input  [size-1:0] in;
  input             clk;
  reg    [size-1:0] out;
  always @(posedge clk)
    #delay out = in;
endmodule
Local parameters cannot be overridden; therefore, they are not considered part of the ordered list for
parameter value assignment. In the following example, addr_width will be assigned the value 12, and
data_width will be assigned the value 16. mem_size will not be explicitly assigned a value due to the
ordered list, but will have the value 4096 due to its declaration expression.
module my_mem (addr, data);
parameter addr_width = 16;
localparam mem_size = 1 << addr_width;
parameter data_width = 8;
...
endmodule
module top;
...
my_mem #(12, 16) m(addr,data);
endmodule
##### 12.2.2.2 Parameter value assignment by name

Parameter assignment by name consists of explicitly linking the parameter name and its new value. The
name of the parameter shall be the name specified in the instantiated module.
It is not necessary to assign values to all of the parameters within a module when using this method. Only
parameters that are assigned new values need to be specified.
The parameter expression is optional so that the instantiating module can document the existence of a
parameter without assigning anything to it. The parentheses are required, and in this case the parameter
retains its default value. Once a parameter is assigned a value, there shall not be another assignment to this
parameter name.
Consider the following example, where both parameters of mod_a and only one parameter of mod_c and
mod_d are changed during instantiation:
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
172
Copyright © 2006 IEEE. All rights reserved.
module tb2;
  wire [9:0] out_a, out_d;
  wire [4:0] out_b, out_c;
  reg  [9:0] in_a, in_d;
  reg  [4:0] in_b, in_c;
  reg        clk;
  // testbench clock & stimulus generation code ...
  // Four instances of vdff with parameter value assignment by name
  // mod_a has new parameter values size=10 and delay=15
  // mod_b has default parameters (size=5, delay=1)
  // mod_c has one default size=5 and one new delay=12
  // mod_d has a new parameter value size=10.
  //   delay retains its default value
  vdff #(.size(10),.delay(15)) mod_a (.out(out_a),.in(in_a),.clk(clk));
  vdff                         mod_b (.out(out_b),.in(in_b),.clk(clk));
  vdff #(.delay(12))           mod_c (.out(out_c),.in(in_c),.clk(clk));
  vdff #(.delay( ),.size(10) ) mod_d (.out(out_d),.in(in_d),.clk(clk));
endmodule
module vdff (out, in, clk);
  parameter size=5, delay=1;
  output [size-1:0] out;
  input  [size-1:0] in;
  input             clk;
  reg    [size-1:0] out;
  always @(posedge clk)
    #delay out = in;
endmodule
It shall be legal to instantiate modules using different types of parameter redefinition in the same top-level
module. Consider the following example, where the parameters of mod_a are changed using parameter
redefinition by ordered list and the second parameter of mod_c is changed using parameter redefinition by
name during instantiation:
module tb3;
  // declarations & code
  // legal mixture of instance with positional parameters and
  // another instance with named parameters
  vdff #(10, 15)     mod_a (.out(out_a), .in(in_a), .clk(clk));
  vdff               mod_b (.out(out_b), .in(in_b), .clk(clk));
  vdff #(.delay(12)) mod_c (.out(out_c), .in(in_c), .clk(clk));
endmodule
It shall be illegal to instantiate any module using a mixture of parameter redefinitions by order and by name
as shown in the instantiation of mod_a below:
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
173
// mod_a instance with ILLEGAL mixture of parameter assignments
vdff #(10, .delay(15)) mod_a (.out(out_a), .in(in_a), .clk(clk));
#### 12.2.3 Parameter dependence

A parameter (for example, memory_size) can be defined with an expression containing another parameter
(for example, word_size). However, overriding a parameter, whether by a defparam statement or in a
module instantiation statement, effectively replaces the parameter definition with the new expression.
Because memory_size depends on the value of word_size, a modification of word_size changes the
value of memory_size. For example, in the following parameter declaration, an update of word_size,
whether by defparam statement or in an instantiation statement for the module that defined these
parameters, automatically updates memory_size. If memory_size is updated due to either a defparam or
an instantiation statement, then it will take on that value, regardless of the value of word_size.
parameter
    word_size = 32,
    memory_size = word_size * 4096;
### 12.3 Ports

Ports provide a means of interconnecting a hardware description consisting of modules and primitives. For
example, module A can instantiate module B, using port connections appropriate to module A. These port
names can differ from the names of the internal nets and variables specified in the definition of module B.
#### 12.3.1 Port definition

The syntax for ports and a list of ports is given in Syntax 12-3.
```ebnf
list_of_ports ::= (From A.1.3)
```

( port { , port } )
```ebnf
list_of_port_declarations ::=
```

( port_declaration { , port_declaration } )
| ( )
```ebnf
port ::=
```

[ port_expression ]
| . port_identifier ( [ port_expression ] )
```ebnf
port_expression ::=
```

port_reference
| { port_reference { , port_reference } }
```ebnf
port_reference ::=
```

port_identifier [ [ constant_range_expression ] ]
```ebnf
port_declaration ::=
```

{attribute_instance} inout_declaration
| {attribute_instance} input_declaration
| {attribute_instance} output_declaration
Syntax 12-3—Syntax for port
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
174
Copyright © 2006 IEEE. All rights reserved.
#### 12.3.2 List of ports

The port reference for each port in the list of ports at the top of each module declaration can be one of the
following:
—
A simple identifier or escaped identifier
—
A bit-select of a vector declared within the module
—
A part-select of a vector declared within the module
—
A concatenation of any of the above
The port expression is optional because ports can be defined that do not connect to anything internal to the
module. Once a port has been defined, there shall not be another port definition with this same name.
The first type of module port, with only a port_expression, is an implicit port. The second type is the
explicit port. This explicitly specifies the port_identifier used for connecting module instance ports by
name (see 12.3.6) and the port_expression that contains identifiers declared inside the module as
described in 12.3.3. Named port connections shall not be used for implicit ports unless the
port_expression is a simple identifier or escaped identifier, which shall be used as the port name.
#### 12.3.3 Port declarations

Each port_identifier in a port_expression in the list of ports for the module declaration shall also be declared
in the body of the module as one of the following port declarations: input, output, or inout (bidirectional).
This is in addition to any other data type declaration for a particular port— for example, a reg or wire. The
syntax for port declarations is given in Syntax 12-4.
If a port declaration includes a net or variable type, then the port is considered completely declared, and it is
an error for the port to be declared again in a variable or net data type declaration. Because of this, all other
aspects of the port shall be declared in such a port declaration, including the signed and range definitions if
needed.
If a port declaration does not include a net or variable type, then the port can be again declared in a net or
variable declaration. If the net or variable is declared as a vector, the range specification between the two
declarations of a port shall be identical. Once a name is used in a port declaration, it shall not be declared
again in another port declaration or in a data type declaration.
```ebnf
inout_declaration ::= (From A.2.1.2)
```

inout [ net_type ] [ signed ] [ range ] list_of_port_identifiers
```ebnf
input_declaration ::=
```

input [ net_type ] [ signed ] [ range ] list_of_port_identifiers
```ebnf
output_declaration ::=
```

output [ net_type ] [ signed ] [ range ]
list_of_port_identifiers
| output reg [ signed ] [ range ]
list_of_variable_port_identifiers
| output output_variable_type
list_of_variable_port_identifiers
```ebnf
list_of_port_identifiers ::= (From A.2.3)
```

port_identifier { , port_identifier }
Syntax 12-4—Syntax for port declarations
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
175
Implementations may limit the maximum number of ports in a module definition, but the limit shall be at
least 256.
For example:
input  aport;    // First declaration - okay.
input  aport;    // Error - multiple declaration, port declaration
output aport;    // Error - multiple declaration, port declaration
The signed attribute can be attached either to a port declaration or the corresponding net or reg declaration
or to both. If either the port or the net/reg is declared as signed, then the other shall also be considered
signed.
Implicit nets shall be considered unsigned. Nets connected to ports without an explicit net declaration shall
be considered unsigned, unless the port is declared as signed.
For example:
module test(a,b,c,d,e,f,g,h);
input [7:0] a;
 // no explicit declaration - net is unsigned
input [7:0] b;
input signed [7:0] c;
input signed [7:0] d;  // no explicit net declaration - net is signed
output [7:0] e;
 // no explicit declaration - net is unsigned
output [7:0] f;
output signed [7:0] g;
output signed [7:0] h;  // no explicit net declaration - net is signed
wire signed [7:0] b;
// port b inherits signed attribute from net decl.
wire [7:0] c;
// net  c inherits signed attribute from port
reg signed [7:0] f;
// port f inherits signed attribute from reg decl.
reg [7:0] g;
// reg  g inherits signed attribute from port
endmodule
module complex_ports ({c,d}, .e(f));
// Nets {c,d} receive the first port bits.
// Name 'f' is declared inside the module.
// Name 'e' is defined outside the module.
// Can't use named port connections of first port.
module split_ports (a[7:4], a[3:0]);
// First port  is upper 4 bits of 'a'.
// Second port is lower 4 bits of 'a'.
// Can't use named port connections because
// of part-select port 'a'.
module same_port (.a(i), .b(i));
         // Name 'i' is declared inside the module as an inout port.
         // Names 'a' and 'b' are defined for port connections.
module renamed_concat (.a({b,c}), f, .g(h[1]));
 // Names 'b', 'c', 'f', 'h' are defined inside the module.
 // Names 'a', 'f', 'g' are defined for port connections.
 // Can use named port connections.
module same_input (a,a);
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
176
Copyright © 2006 IEEE. All rights reserved.
input a;              // This is legal. The inputs are tied together.
module mixed_direction (.p({a, e}));
input a;              // p contains both input and output directions.
output e;
#### 12.3.4 List of ports declarations

An alternate syntax that minimizes the duplication of data can be used to specify the ports of a module. Each
module shall be declared either entirely with the list of ports syntax as described in 12.3.2 or entirely using
the list_of_port_declarations as described in this subclause.
Each declared port provides the complete information about the port. The port’s direction, width, net, or
variable type and whether the port is signed or unsigned are completely described. The same syntax for
input, inout, and output declarations is used in the module header as would be used for the list of port style
declaration, except the list_of_port_declarations is included in the module header rather than separately
(after the ; that terminates the module header).
For example:
As an example, the module named test given in the previous example could alternatively be declared as
follows:
module test (
   input [7:0] a,
   input signed [7:0] b, c, d,
// Multiple ports that share all
// attributes can be declared together.
   output [7:0] e,
// Every attribute of the declaration
// must be in the one declaration.
   output reg signed [7:0] f, g,
   output signed   [7:0] h) ;
   // It is illegal to redeclare any ports of
   // the module in the body of the module.
endmodule
The port_reference type of module port declaration shall not be done using list_of_port_declarations style
of module declarations. Also ports declared using the list_of_port_declarations shall only be simple
identifiers or escaped identifiers. They shall not be bit-selects, part-selects, or concatenations (as in the
example complex_ports); nor can ports be split (as in the example split_ports); nor can they be named
ports (as in the example same_port).
Designs may freely mix modules declared using each syntax; hence implementations desiring the above
special cases of port declaration can be done using the first list_of_ports syntax.
#### 12.3.5 Connecting module instance ports by ordered list

One method of making the connection between the port expressions listed in a module instantiation and the
ports declared within the instantiated module is the ordered list; that is, the port expressions listed for the
module instance shall be in the same order as the ports listed in the module declaration.
For example:
The following example illustrates a top-level module (topmod) that instantiates a second module (modB).
Module modB has ports that are connected by an ordered list. The connections made are as follows:
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
177
—
Port wa in the modB definition connects to the bit-select v[0] in the topmod module.
—
Port wb connects to v[3].
—
Port c connects to w.
—
Port d connects to v[4].
In the modB definition, ports wa and wb are declared as inouts while ports c and d are declared as input.
module topmod;
wire [4:0] v;
wire a,b,c,w;
modB b1 (v[0], v[3], w, v[4]);
endmodule
module modB (wa, wb, c, d);
inout wa, wb;
input c, d;
tranif1
g1 (wa, wb, cinvert);
not #(2, 6)
n1 (cinvert, int);
and #(6, 5)
g2 (int, c, d);
endmodule
During simulation of the b1 instance of modb, the and gate g2 activates first to produce a value on int. This
value triggers the not gate n1 to produce output on cinvert, which then activates the tranif1 gate g1.
#### 12.3.6 Connecting module instance ports by name

The second way to connect module ports consists of explicitly linking the two names for each side of the
connection: the port declaration name from the module declaration to the expression, i.e., the name used in
the module declaration, followed by the name used in the instantiating module. This compound name is then
placed in the list of module connections. The port name shall be the name specified in the module
declaration. The port name cannot be a bit-select, a part-select, or a concatenation of ports. If the module
port declaration was implicit, the port_expression shall be a simple identifier or escaped identifier,
which shall be used as the port name. If the module port declaration was explicit, the explicit name is used as
the name of port.
The port expression can be any valid expression.
The port expression is optional so that the instantiating module can document the existence of the port
without connecting it to anything. The parentheses are required.
The two types of module port connections shall not be mixed; connections to the ports of a particular module
instance shall be all by order or all by name.
For example:
Example 1—In the following example, the instantiating module connects its signals topA and topB to the
ports In1 and Out defined by the module ALPHA. At least one port provided by ALPHA is unused; it is named
In2. There could be other unused ports not mentioned in the instantiation.
ALPHA instance1 (.Out(topB),.In1(topA),.In2());
Example 2—This example defines the modules modB and topmod, and then topmod instantiates modB using
ports connected by name.
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
178
Copyright © 2006 IEEE. All rights reserved.
module topmod;
wire [4:0] v;
wire a,b,c,w;
modB b1 (.wb(v[3]),.wa(v[0]),.d(v[4]),.c(w));
endmodule
module modB(wa, wb, c, d);
inout wa, wb;
input c, d;
tranif1
g1(wa, wb, cinvert);
not #(6, 2)
n1(cinvert, int);
and #(5, 6)
g2(int, c, d);
endmodule
Because these connections are made by name, the order in which they appear is irrelevant.
Multiple module instance port connections are not allowed, e.g., the following example is illegal:
Example 3—This example shows illegal port connections.
module test;
  a ia (.i (a), .i (b), // illegal connection of input port twice.
        .o (c), .o (d), // illegal connection of output port twice.
        .e (e), .e (f)); // illegal connection of inout port twice.
endmodule
#### 12.3.7 Real numbers in port connections

The real data type shall not be directly connected to a port. It shall be connected indirectly, as shown in the
following example. The system functions $realtobits and $bitstoreal shall be used for passing the bit
patterns across module ports. (See 17.8 for a description of these system tasks.)
For example:
module driver (net_r);
output net_r;
real r;
wire [64:1] net_r = $realtobits(r);
endmodule
module receiver (net_r);
input net_r;
wire [64:1] net_r;
real r;
initial assign r = $bitstoreal(net_r);
endmodule
#### 12.3.8 Connecting dissimilar ports

A port of a module can be viewed as providing a link or connection between two items (e.g., nets, regs,
expressions)—one internal to the module instance and one external to the module instance.
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
179
Examination of the port connection rules described in 12.3.9 will show that the item receiving the value
through the port (the internal item for inputs, the external item for outputs) shall be a structural net
expression. The item that provides the value can be any expression.
A port that is declared as input (output) but used as an output (input) or inout may be coerced to inout. If not
coerced to inout, a warning has to be issued.
#### 12.3.9 Port connection rules

The rules in 12.3.9.1 through 12.3.9.3 shall govern the way module ports are declared and the way they are
interconnected.
##### 12.3.9.1 Rule 1

An input or inout port shall be of type net.
##### 12.3.9.2 Rule 2

Each port connection shall be a continuous assignment of source to sink, where one connected item shall be
a signal source and the other shall be a signal sink. The assignment shall be a continuous assignment from
source to sink for input or output ports. The assignment is a nonstrength reducing transistor connection for
inout ports. Only nets or structural net expressions shall be the sinks in an assignment.
A structural net expression is a port expression whose operands can be the following:
—
A scalar net
—
A vector net
—
A constant bit-select of a vector net
—
A part-select of a vector net
—
A concatenation of structural net expressions
The following external items shall not be connected to the output or inout ports of modules:
—
Variables
—
Expressions other than the following:
—
A scalar net
—
A vector net
—
A constant bit-select of a vector net
—
A part-select of a vector net
—
A concatenation of the expressions listed above
##### 12.3.9.3 Rule 3

If the net on either side of a port has the net type uwire, a warning shall be issued if the nets are not merged
into a single net, as described in 12.3.10.
#### 12.3.10 Net types resulting from dissimilar port connections

When different net types are connected through a module port, the nets on both sides of the port can take on
the same type. The resulting net type can be determined as shown in Table 12-1. In the table, external net
means the net specified in the module instantiation, and internal net means the net specified in the module
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
180
Copyright © 2006 IEEE. All rights reserved.
definition. The net whose type is used is said to be the dominating net. The net whose type is changed is said
to be the dominated net. It is permissible to merge the dominating and dominated nets into a single net,
whose type shall be that of the dominating net. The resulting net is called the simulated net, and the
dominated net is called a collapsed net.
The simulated net shall take the delay specified for the dominating net. If the dominating net is of the type
trireg, any strength value specified for the trireg net shall apply to the simulated net.
##### 12.3.10.1 Net type resolution rule

When the two nets connected by a port are of different net type, the resulting single net can be assigned one
of the following:
—
The dominating net type if one of the two nets is dominating, or
—
The net type external to the module
When a dominating net type does not exist, the external net type shall be used.
##### 12.3.10.2 Net type table

Table 12-1 shows the net type dictated by net type resolution rule.
Table 12-1—Net types resulting from dissimilar port connections
Internal
net
External net
wire,
tri
wand,
triand
wor,
trior
trireg
tri0
tri1
uwire
supply0
supply1
wire, tri
ext
ext
ext
ext
ext
ext
ext
ext
ext
wand, triand
int
ext
ext
warn
ext
warn
ext
warn
ext
warn
ext
warn
ext
ext
wor, trior
int
ext
warn
ext
ext
warn
ext
warn
ext
warn
ext
warn
ext
ext
trireg
int
ext
warn
ext
warn
ext
ext
ext
ext
warn
ext
ext
tri0
int
ext
warn
ext
warn
int
ext
ext
warn
ext
warn
ext
ext
tri1
int
ext
warn
ext
warn
int
ext
warn
ext
ext
warn
ext
ext
uwire
int
int
warn
int
warn
int
warn
int
warn
int
warn
ext
ext
ext
supply0
int
int
int
int
int
int
int
ext
ext warn
supply1
int
int
int
int
int
int
int
ext warn
ext
KEY:
ext = The external net type shall be used.
int = The internal net type shall be used.
warn = A warning shall be issued.
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
181
The simulated net shall take the net type specified in the table and the delay specified for that net. If the
simulated net selected is a trireg, any strength value specified for the trireg net applies to the simulated net.
#### 12.3.11 Connecting signed values via ports

The sign attribute shall not cross hierarchy. In order to have the signed type cross hierarchy, the signed
keyword must be used in the object’s declaration at the different levels of hierarchy. Any expressions on a
port shall be treated as any other expression in an assignment. It shall be typed, sized, and evaluated, and the
resulting value assigned to the object on the other side of the port using the same rules as an assignment.
### 12.4 Generate constructs

Generate constructs are used to either conditionally or multiply instantiate generate blocks into a model. A
generate block is a collection of one or more module items. A generate block may not contain port
declarations, parameter declarations, specify blocks, or specparam declarations. All other module items,
including other generate constructs, are allowed in a generate block. Generate constructs provide the ability
for parameter values to affect the structure of the model. They also allow for modules with repetitive
structure to be described more concisely, and they make recursive module instantiation possible.
There are two kinds of generate constructs: loops and conditionals. Loop generate constructs allow a single
generate block to be instantiated into a model multiple times. Conditional generate constructs, which
include if-generate and case-generate constructs, instantiate at most one generate block from a set of
alternative generate blocks. The term generate scheme refers to the method for determining which or how
many generate blocks are instantiated. It includes the conditional expressions, case alternatives, and loop
control statements that appear in a generate construct.
Generate schemes are evaluated during elaboration of the model. Elaboration occurs after parsing the HDL
and before simulation; and it involves expanding module instantiations, computing parameter values,
resolving hierarchical names (see 12.5), establishing net connectivity and in general preparing the model for
simulation. Although generate schemes use syntax that is similar to behavioral statements, it is important to
recognize that they do not execute at simulation time. They are evaluated at elaboration time, and the result
is determined before simulation begins. Therefore, all expressions in generate schemes shall be constant
expressions, deterministic at elaboration time. For more details on elaboration, see 12.8.
The elaboration of a generate construct results in zero or more instances of a generate block. An instance of
a generate block is similar in some ways to an instance of a module. It creates a new level of hierarchy. It
brings the objects, behavioral constructs, and module instances within the block into existence. These
constructs act the same as they would if they were in a module brought into existence with a module
instantiation, except that object declarations from the enclosing scope can be referenced directly (see 12.7).
Names in instantiated named generate blocks can be referenced hierarchically as described in 12.5.
The keywords generate and endgenerate may be used in a module to define a generate region. A generate
region is a textual span in the module description where generate constructs may appear. Use of generate
regions is optional. There is no semantic difference in the module when a generate region is used. A parser
may choose to recognize the generate region to produce different error messages for misused generate
construct keywords. Generate regions do not nest, and they may only occur directly within a module. If the
generate keyword is used, it shall be matched by an endgenerate keyword.
The syntax for generate constructs is given in Syntax 12-5.
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
182
Copyright © 2006 IEEE. All rights reserved.
Syntax 12-5—Syntax for generate constructs
```ebnf
module_or_generate_item ::= (From A.1.4)
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
generate_region ::= (From A.4.2)
```

generate { module_or_generate_item } endgenerate
```ebnf
genvar_declaration ::=
```

genvar list_of_genvar_identifiers ;
```ebnf
list_of_genvar_identifiers ::=
```

genvar_identifier { , genvar_identifier }
```ebnf
loop_generate_construct ::=
```

for ( genvar_initialization ; genvar_expression ; genvar_iteration )
    generate_block
```ebnf
genvar_initialization ::=
```

genvar_identifier = constant_expression
```ebnf
genvar_expression ::=
  genvar_primary
| unary_operator { attribute_instance } genvar_primary
| genvar_expression binary_operator { attribute_instance } genvar_expression
| genvar_expression ? { attribute_instance } genvar_expression : genvar_expression
genvar_iteration ::=
```

genvar_identifier = genvar_expression
```ebnf
genvar_primary ::=
  constant_primary
| genvar_identifier
conditional_generate_construct ::=
  if_generate_construct
| case_generate_construct
if_generate_construct ::=
```

if ( constant_expression ) generate_block_or_null
    [ else generate_block_or_null ]
```ebnf
case_generate_construct ::=
```

case ( constant_expression )
    case_generate_item { case_generate_item } endcase
```ebnf
case_generate_item ::=
  constant_expression { , constant_expression } : generate_block_or_null
| default [ : ] generate_block_or_null
generate_block ::=
  module_or_generate_item
| begin [ : generate_block_identifier ] { module_or_generate_item } end
generate_block_or_null ::=
  generate_block | ;
```

Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
183
#### 12.4.1 Loop generate constructs

A loop generate construct permits a generate block to be instantiated multiple times using syntax that is
similar to a for loop statement. The loop index variable shall be declared in a genvar declaration prior to its
use in a loop generate scheme.
The genvar is used as an integer during elaboration to evaluate the generate loop and create instances of the
generate block, but it does not exist at simulation time. A genvar shall not be referenced anywhere other
than in a loop generate scheme.
Both the initialization and iteration assignments in the loop generate scheme shall assign to the same
genvar. The initialization assignment shall not reference the loop index variable on the right-hand side.
Within the generate block of a loop generate construct, there is an implicit localparam declaration. This is
an integer parameter that has the same name and type as the loop index variable, and its value within each
instance of the generate block is the value of the index variable at the time the instance was elaborated. This
parameter can be used anywhere within the generate block that a normal parameter with an integer value can
be used. It can be referenced with a hierarchical name.
Because this implicit localparam has the same name as the genvar, any reference to this name inside the
loop generate block will be a reference to the localparam, not to the genvar. As a consequence, it is not
possible to have two nested loop generate constructs that use the same genvar.
Generate blocks in loop generate constructs can be named or unnamed, and they can consist of only one
item, which need not be surrounded by begin/end keywords. Even if the begin/end keywords are absent, it
is still a generate block, which, like all generate blocks, comprises a separate scope and a new level of
hierarchy when it is instantiated.
If the generate block is named, it is a declaration of an array of generate block instances. The index values in
this array are the values assumed by the genvar during elaboration. This can be a sparse array because the
genvar values do not have to form a contiguous range of integers. The array is considered to be declared
even if the loop generate scheme resulted in no instances of the generate block. If the generate block is not
named, the declarations within it cannot be referenced using hierarchical names other than from within the
hierarchy instantiated by the generate block itself.
It shall be an error if the name of a generate block instance array conflicts with any other declaration,
including any other generate block instance array. It shall be an error if the loop generate scheme does not
terminate. It shall be an error if a genvar value is repeated during the evaluation of the loop generate
scheme. It shall be an error if any bit of the genvar is set to x or z during the evaluation of the loop generate
scheme.
For example:
Example 1—Examples of legal and illegal generate loops
module mod_a;
genvar i;
// "generate",  "endgenerate" keywords are not required
for (i=0; i<5; i=i+1) begin:a
  for (i=0; i<5; i=i+1) begin:b
  ...               // error -- using "i" as loop index for
  ...               // two nested generate loops
  end
end
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
184
Copyright © 2006 IEEE. All rights reserved.
endmodule
------
module mod_b;
genvar i;
reg a;
for (i=1; i<0; i=i+1) begin: a
  ...               // error -- "a" conflicts with name of reg "a"
end
endmodule
------
module mod_c;
genvar i;
for (i=1; i<5; i=i+1) begin: a
  ...
end
for (i=10; i<15; i=i+1) begin: a
  ...              // error -- "a" conflicts with name of previous
  ...              // loop even though indices are unique
end
endmodule
Example 2—A parameterized gray-code–to–binary-code converter module using a loop to generate
continuous assignments
module gray2bin1 (bin, gray);
  parameter SIZE = 8;      // this module is parameterizable
  output [SIZE-1:0] bin;
  input  [SIZE-1:0] gray;

  genvar i;
  generate
    for (i=0; i<SIZE; i=i+1) begin:bit
      assign bin[i] = ^gray[SIZE-1:i];
            // i refers to the implicitly defined localparam whose
            // value in each instance of the generate block is
            // the value of the genvar when it was elaborated.
    end
  endgenerate
endmodule
The models in Example 3 and Example 4 are parameterized modules of ripple adders using a loop to
generate Verilog gate primitives. Example 3 uses a two-dimensional net declaration outside of the generate
loop to make the connections between the gate primitives while Example 4 makes the net declaration inside
of the generate loop to generate the wires needed to connect the gate primitives for each iteration of the loop.
Example 3—Generated ripple adder with two-dimensional net declaration outside of the generate loop
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
185
module addergen1 (co, sum, a, b, ci);
  parameter SIZE = 4;
  output [SIZE-1:0] sum;
  output            co;
  input  [SIZE-1:0] a, b;
  input             ci;
  wire   [SIZE  :0] c;
  wire   [SIZE-1:0] t [1:3];
  genvar            i;

  assign c[0] = ci;

  // Hierarchical gate instance names are:
  // xor gates: bit[0].g1 bit[1].g1 bit[2].g1 bit[3].g1
  //            bit[0].g2 bit[1].g2 bit[2].g2 bit[3].g2
  // and gates: bit[0].g3 bit[1].g3 bit[2].g3 bit[3].g3
  //            bit[0].g4 bit[1].g4 bit[2].g4 bit[3].g4
  // or  gates: bit[0].g5 bit[1].g5 bit[2].g5 bit[3].g5
  // Generated instances are connected with
  // multidimensional nets t[1][3:0] t[2][3:0] t[3][3:0]
  // (12 nets total)

    for(i=0; i<SIZE; i=i+1) begin:bit
      xor g1 ( t[1][i],    a[i],    b[i]);
      xor g2 (  sum[i], t[1][i],    c[i]);
      and g3 ( t[2][i],    a[i],    b[i]);
      and g4 ( t[3][i], t[1][i],    c[i]);
      or  g5 (  c[i+1], t[2][i], t[3][i]);
    end

  assign co = c[SIZE];
endmodule
Example 4—Generated ripple adder with net declaration inside of the generate loop
module addergen1 (co, sum, a, b, ci);
  parameter SIZE = 4;
  output [SIZE-1:0] sum;
  output            co;
  input  [SIZE-1:0] a, b;
  input             ci;
  wire   [SIZE  :0] c;

  genvar            i;

  assign c[0] = ci;

  // Hierarchical gate instance names are:
  // xor gates: bit[0].g1 bit[1].g1 bit[2].g1 bit[3].g1
  //            bit[0].g2 bit[1].g2 bit[2].g2 bit[3].g2
  // and gates: bit[0].g3 bit[1].g3 bit[2].g3 bit[3].g3
  //            bit[0].g4 bit[1].g4 bit[2].g4 bit[3].g4
  // or  gates: bit[0].g5 bit[1].g5 bit[2].g5 bit[3].g5
  // Gate instances are connected with nets named:
  //            bit[0].t1 bit[1].t1 bit[2].t1 bit[3].t1
  //            bit[0].t2 bit[1].t2 bit[2].t2 bit[3].t2
  //            bit[0].t3 bit[1].t3 bit[2].t3 bit[3].t3

Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
186
Copyright © 2006 IEEE. All rights reserved.
    for(i=0; i<SIZE; i=i+1) begin:bit
      wire   t1, t2, t3;

      xor g1 (     t1, a[i], b[i]);
      xor g2 ( sum[i],   t1, c[i]);
      and g3 (     t2, a[i], b[i]);
      and g4 (     t3,   t1, c[i]);
      or  g5 ( c[i+1],   t2,   t3);
    end

  assign co = c[SIZE];
endmodule
The hierarchical generate block instance names in a multilevel generate loop are shown in Example 5. For
each block instance created by the generate loop, the generate block identifier for the loop is indexed by
adding the “[genvar value]” to the end of the generate block identifier. These names can be used in
hierarchical path names (see 12.5).
Example 5—A multilevel generate loop
parameter SIZE = 2;
genvar i, j, k, m;
generate
  for (i=0; i<SIZE; i=i+1) begin:B1     // scope B1[i]
    M1 N1();                   // instantiates B1[i].N1
    for (j=0; j<SIZE; j=j+1) begin:B2   // scope B1[i].B2[j]
      M2 N2();                 // instantiates B1[i].B2[j].N2
      for (k=0; k<SIZE; k=k+1) begin:B3 // scope B1[i].B2[j].B3[k]
        M3 N3();               // instantiates B1[i].B2[j].B3[k].N3
      end
    end
    if (i>0) begin:B4                   // scope B1[i].B4
      for (m=0; m<SIZE; m=m+1) begin:B5 // scope B1[i].B4.B5[m]
        M4 N4();               // instantiates B1[i].B4.B5[m].N4
      end
    end
  end
endgenerate
  // Some examples of hierarchical names for the module instances:
  // B1[0].N1              B1[1].N1
  // B1[0].B2[0].N2        B1[0].B2[1].N2
  // B1[0].B2[0].B3[0].N3  B1[0].B2[0].B3[1].N3
  // B1[0].B2[1].B3[0].N3
  // B1[1].B4.B5[0].N4     B1[1].B4.B5[1].N4
#### 12.4.2 Conditional generate constructs

The conditional generate constructs, if-generate and case-generate, select at most one generate block from a
set of alternative generate blocks based on constant expressions evaluated during elaboration. The selected
generate block, if any, is instantiated into the model.
Generate blocks in conditional generate constructs can be named or unnamed, and they may consist of only
one item, which need not be surrounded by begin/end keywords. Even if the begin/end keywords are
absent, it is still a generate block, which, like all generate blocks, comprises a separate scope and a new level
of hierarchy when it is instantiated.
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
187
Because at most one of the alternative generate blocks is instantiated, it is permissible for there to be more
than one block with the same name within a single conditional generate construct. It is not permissible for
any of the named generate blocks to have the same name as generate blocks in any other conditional or loop
generate construct in the same scope, even if the blocks with the same name are not selected for
instantiation. It is not permissible for any of the named generate blocks to have the same name as any other
declaration in the same scope, even if that block is not selected for instantiation.
If the generate block selected for instantiation is named, then this name declares a generate block instance
and is the name for the scope it creates. Normal rules for hierarchical naming apply. If the generate block
selected for instantiation is not named, it still creates a scope; but the declarations within it cannot be
referenced using hierarchical names other than from within the hierarchy instantiated by the generate block
itself.
If a generate block in a conditional generate construct consists of only one item that is itself a conditional
generate construct and if that item is not surrounded by begin/end keywords, then this generate block is not
treated as a separate scope. The generate construct within this block is said to be directly nested. The
generate blocks of the directly nested construct are treated as if they belong to the outer construct. Therefore,
they can have the same name as the generate blocks of the outer construct, and they cannot have the same
name as any declaration in the scope enclosing the outer construct (including other generate blocks in other
generate constructs in that scope). This allows complex conditional generate schemes to be expressed
without creating unnecessary levels of generate block hierarchy.
The most common use of this would be to create an if-else-if generate scheme with any number of else-if
clauses, all of which can have generate blocks with the same name because only one will be selected for
instantiation. It is permissible to combine if-generate and case-generate constructs in the same complex
generate scheme. Direct nesting applies only to conditional generate constructs nested in conditional
generate constructs. It does not apply in any way to loop generate constructs.
Example 1
module test;
parameter p = 0, q = 0;
wire a, b, c;
//---------------------------------------------------------
// Code to either generate a u1.g1 instance or no instance.
// The u1.g1 instance of one of the following gates:
// (and, or, xor, xnor) is generated if
// {p,q} == {1,0}, {1,2}, {2,0}, {2,1}, {2,2}, {2, default}
//---------------------------------------------------------
if (p == 1)
  if (q == 0)
    begin : u1          // If p==1 and q==0, then instantiate
      and g1(a, b, c); // AND with hierarchical name test.u1.g1
    end
  else if (q == 2)
    begin : u1          // If p==1 and q==2, then instantiate
      or  g1(a, b, c); // OR with hierarchical name test.u1.g1
    end
                   // "else" added to end "if (q == 2)" statement
  else ;                                  // If p==1 and q!=0 or 2, then no instantiation
else if (p == 2)
  case (q)
  0, 1, 2:
    begin : u1         // If p==2 and q==0,1, or 2, then instantiate
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
188
Copyright © 2006 IEEE. All rights reserved.
      xor g1(a, b, c);// XOR with hierarchical name test.u1.g1
          end
  default:
    begin : u1          // If p==2 and q!=0,1, or 2, then instantiate
      xnor g1(a, b, c);// XNOR with hierarchical name test.u1.g1
    end
  endcase
endmodule
This generate construct will select at most one of the generate blocks named u1. The hierarchical name of
the gate instantiation in that block would be test.u1.g1. When nesting if-generate constructs, the else
always belongs to the nearest if construct.
NOTE—As in the example above, an else with a null generate block can be inserted to make a subsequent else belong to
an outer if construct. begin/end keywords can also be used to disambiguate. However, this would violate the criteria for
direct nesting, and an extra level of generate block hierarchy would be created.
Conditional generate constructs make it possible for a module to contain an instantiation of itself. The same
can be said of loop generate constructs, but it is more easily done with conditional generates. With proper
use of parameters, the resulting recursion can be made to terminate, resulting in a legitimate model
hierarchy. Because of the rules for determining top-level modules, a module containing an instantiation of
itself will not be a top-level module.
Example 2—An implementation of a parameterized multiplier module
module multiplier(a,b,product);
parameter a_width = 8, b_width = 8;
localparam product_width = a_width+b_width;
                  // cannot be modified directly with the defparam
                  // statement or the module instance statement #
input  [a_width-1:0] a;
input  [b_width-1:0] b;
output [product_width-1:0] product;
generate
  if((a_width < 8) || (b_width < 8)) begin: mult
    CLA_multiplier #(a_width,b_width) u1(a, b, product);
    // instantiate a CLA  multiplier
  end
  else begin: mult
    WALLACE_multiplier #(a_width,b_width) u1(a, b, product);
    // instantiate a Wallace-tree  multiplier
  end
endgenerate
// The hierarchical instance name is mult.u1
endmodule
Example 3—Generate with a case to handle widths less than 3
generate
  case (WIDTH)
    1: begin: adder              // 1-bit adder implementation
         adder_1bit x1(co, sum, a, b, ci);
       end
    2: begin: adder              // 2-bit adder implementation
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
189
         adder_2bit x1(co, sum, a, b, ci);
       end
    default:
       begin: adder              // others - carry look-ahead adder
         adder_cla #(WIDTH) x1(co, sum, a, b, ci);
       end
  endcase
// The hierarchical instance name is adder.x1
endgenerate
Example 4—A module of memory dimm
module dimm(addr, ba, rasx, casx, csx, wex, cke, clk, dqm, data, dev_id);
  parameter [31:0] MEM_WIDTH = 16, MEM_SIZE  = 8; // in mbytes
  input [10:0] addr;
  input        ba, rasx, casx, csx, wex, cke, clk;
  input [ 7:0] dqm;
  inout [63:0] data;
  input [ 4:0] dev_id;
  genvar       i;
    case ({MEM_SIZE, MEM_WIDTH})
      {32'd8, 32'd16}: // 8Meg x 16 bits wide
      begin: memory
        for (i=0; i<4; i=i+1) begin:word
          sms_08b216t0 p(.clk(clk), .csb(csx), .cke(cke),.ba(ba),
                       .addr(addr), .rasb(rasx), .casb(casx),
                       .web(wex), .udqm(dqm[2*i+1]), .ldqm(dqm[2*i]),
                       .dqi(data[15+16*i:16*i]), .dev_id(dev_id));
        // The hierarchical instance names are memory.word[3].p,
        // memory.word[2].p, memory.word[1].p, memory.word[0].p,
        // and the task memory.read_mem
        end
        task read_mem;
         input  [31:0] address;
          output [63:0] data;
          begin                 // call read_mem in sms module
            word[3].p.read_mem(address, data[63:48]);
            word[2].p.read_mem(address, data[47:32]);
            word[1].p.read_mem(address, data[31:16]);
            word[0].p.read_mem(address, data[15: 0]);
          end
        endtask
      end
      {32'd16, 32'd8}: // 16Meg x 8 bits wide
      begin: memory
        for (i=0; i<8; i=i+1) begin:byte
          sms_16b208t0 p(.clk(clk), .csb(csx), .cke(cke),.ba(ba),
                       .addr(addr), .rasb(rasx), .casb(casx),
                       .web(wex), .dqm(dqm[i]),
                       .dqi(data[7+8*i:8*i]), .dev_id(dev_id));
        // The hierarchical instance names are memory.byte[7].p,
        // memory.byte[6].p, ... , memory.byte[1].p, memory.byte[0].p,
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
190
Copyright © 2006 IEEE. All rights reserved.
        // and the task memory.read_mem
        end
        task read_mem;
          input  [31:0] address;
          output [63:0] data;
          begin                 // call read_mem in sms module
            byte[7].p.read_mem(address, data[63:56]);
            byte[6].p.read_mem(address, data[55:48]);
            byte[5].p.read_mem(address, data[47:40]);
            byte[4].p.read_mem(address, data[39:32]);
            byte[3].p.read_mem(address, data[31:24]);
            byte[2].p.read_mem(address, data[23:16]);
            byte[1].p.read_mem(address, data[15: 8]);
            byte[0].p.read_mem(address, data[ 7: 0]);
          end
        endtask
      end
      // Other memory cases ...
    endcase
endmodule
#### 12.4.3 External names for unnamed generate blocks

Although an unnamed generate block has no name that can be used in a hierarchical name, it needs to have a
name by which external interfaces can refer to it. A name will be assigned for this purpose to each unnamed
generate block as described in the next paragraph.
Each generate construct in a given scope is assigned a number. The number will be 1 for the construct that
appears textually first in that scope and will increase by 1 for each subsequent generate construct in that
scope. All unnamed generate blocks will be given the name “genblk<n>” where <n> is the number assigned
to its enclosing generate construct. If such a name would conflict with an explicitly declared name, then
leading zeroes are added in front of the number until the name does not conflict.
NOTE—Each generate construct is assigned its number as described in the previous paragraph even if it does not
contain any unnamed generate bocks.
For example:
module top;
  parameter genblk2 = 0;
  genvar i;
  // The following generate block is implicitly named genblk1
  if (genblk2) reg a;  // top.genblk1.a
  else                  reg b;  // top.genblk1.b
  // The following generate block is implicitly named genblk02
  // as genblk2 is already a declared identifier
  if (genblk2) reg a;  // top.genblk02.a
  else                  reg b;  // top.genblk02.b
  // The following generate block would have been named genblk3
  // but is explicitly named g1
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
191
  for (i = 0; i < 1; i = i + 1) begin : g1    // block name
    // The following generate block is implicitly named genblk1
    // as the first nested scope inside of g1
    if (1)     reg a;  // top.g1[0].genblk1.a
  end
  // The following generate block is implicitly named genblk4 since
  // it belongs to the fourth generate construct in scope "top".
  // The previous generate block would have been
  // named genblk3 if it had not been explicitly named g1
  for (i = 0; i < 1; i = i + 1)
    // The following generate block is implicitly named genblk1
    // as the first nested generate block in genblk4
    if (1)     reg a;  // top.genblk4[0].genblk1.a

  // The following generate block is implicitly named genblk5
  if (1)       reg a;  // top.genblk5.a
endmodule
### 12.5 Hierarchical names

Every identifier in a Verilog HDL description shall have a unique hierarchical path name. The hierarchy of
modules and the definition of items such as tasks and named blocks within the modules shall define these
names. The hierarchy of names can be viewed as a tree structure, where each module instance, generate
block instance, task, function, or named begin-end or fork-join block defines a new hierarchical level, or
scope, in a particular branch of the tree.
A design description contains one or more top-level modules (see 12.1.1). Each such module forms the top
of a name hierarchy. This root or these parallel root modules make up one or more hierarchies in a design
description or description. Inside any module, each module instance (including an arrayed instance),
generate block instance, task definition, function definition, and named begin-end or fork-join block shall
define a new branch of the hierarchy. Named blocks within named blocks and within tasks and functions
shall create new branches. Unnamed generate blocks are exceptions. They create branches that are visible
only from within the block and within any hierarchy instantiated by the block. See 12.4.3 for a discussion of
unnamed generate blocks.
Each node in the hierarchical name tree shall be a separate scope with respect to identifiers. A particular
identifier can be declared at most once in any scope. See 12.7 for a discussion of scope rules and 4.11 for a
discussion of name spaces.
Any named Verilog object or hierarchical name reference can be referenced uniquely in its full form by
concatenating the names of the modules, module instance names, generate blocks, tasks, functions, or
named blocks that contain it. The period character shall be used to separate each of the names in the
hierarchy, except for escaped identifiers embedded in the hierarchical name reference, which are followed
by separators composed of white space and a period-character. The complete path name to any object shall
start at a top-level (root) module. This path name can be used from any level in the hierarchy or from a
parallel hierarchy. The first node name in a path name can also be the top of a hierarchy that starts at the
level where the path is being used (which allows and enables downward referencing of items). Objects
declared in automatic tasks and functions are exceptions and cannot be accessed by hierarchical name
references. Objects declared in unnamed generate blocks are also exceptions. They can be referenced by
hierarchical names only from within the block and within any hierarchy instantiated by the block.
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
192
Copyright © 2006 IEEE. All rights reserved.
Names in a hierarchical path name that refer to instance arrays or loop generate blocks may be followed
immediately by a constant expression in square brackets. This expression selects a particular instance of the
array and is, therefore, called an instance select. The expression shall evaluate to one of the legal index
values of the array. If the array name is not the last path element in the hierarchical name, the instance select
expression is required.
The syntax for hierarchical path names is given in Syntax 12-6.
Syntax 12-6—Syntax for hierarchical path names
For example:
Example 1—The code in this example defines a hierarchy of module instances and named blocks.
module mod (in);
module cct (stim1, stim2);
input in;
input stim1, stim2;
always @(posedge in) begin : keep
// instantiate mod
reg hold;
mod amod(stim1), bmod(stim2);
hold = in;
endmodule
end
endmodule
module wave;
reg stim1, stim2;
cct a(stim1, stim2); // instantiate cct
initial begin :wave1
#100
fork :innerwave
reg hold;
join
#150
begin
stim1 = 0;
end
end
endmodule
Figure 12-1 illustrates the hierarchy implicit in this Verilog code.
```ebnf
escaped_identifier ::= (From A.9.3)
```

\ {Any_ASCII_character_except_white_space} white_space
```ebnf
hierarchical_identifier ::=
```

{ identifier [ [ constant_expression ] ] . } identifier
```ebnf
identifier ::=
  simple_identifier
| escaped_identifier
simple_identifiera ::= [ a-zA-Z_ ] { [ a-zA-Z0-9_$ ] }
white_space ::= (From A.9.4)
```

space | tab | newline | eofb
aA simple_identifier shall start with an alpha or underscore (_) character, shall have at least one character,
and shall not have any spaces.
bEnd of file.
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
193
Figure 12-2 is a list of the hierarchical forms of the names of all the objects defined in the code.
Hierarchical name referencing allows free data access to any object from any level in the hierarchy. If the
unique hierarchical path name of an item is known, its value can be sampled or changed from anywhere
within the description.
Example 2—The next example shows how a pair of named blocks can refer to items declared within each
other.
begin
fork :mod_1
reg x;
mod_2.x = 1;
join
fork :mod_2
reg x;
mod_1.x = 0;
join
end
### 12.6 Upwards name referencing

The name of a module or module instance is sufficient to identify the module and its location in the
hierarchy. A lower level module can reference items in a module above it in the hierarchy. Variables can be
wave1
  a
amod
bmod
keep
keep
innerwave
wave
Figure 12-1—Hierarchy in a model
wave
wave.a.bmod
wave.stim1
wave.a.bmod.in
wave.stim2
wave.a.bmod.keep
wave.a
wave.a.bmod.keep.hold
wave.a.stim1
wave.wave1
wave.a.stim2
wave.wave1.innerwave
wave.a.amod
wave.wave1.innerwave.hold
wave.a.amod.in
wave.a.amod.keep
wave.a.amod.keep.hold
Figure 12-2—Hierarchical path names in a model
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
194
Copyright © 2006 IEEE. All rights reserved.
referenced if the name of the higher level module or its instance name is known. For tasks, functions, named
blocks, and generate blocks, Verilog shall look in the enclosing module for the name until it is found or until
the root of the hierarchy is reached. It shall only search in higher enclosing modules for the name, not
instances.
The syntax for an upward reference is given in Syntax 12-7.
Syntax 12-7—Syntax for upward name referencing
Upward name references can also be done with names of the form
scope_name.item_name
where scope_name is either a module instance name or a generate block name. A name of this form shall be
resolved as follows:
a)
Look in the current scope for a scope named scope_name. If not found and the current scope is not
the module scope, look for the name in the enclosing scope, repeating as necessary until the name is
found or the module scope is reached. If still not found, proceed to step b). Otherwise, this name ref-
erence shall be treated as a downward reference from the scope in which the name is found.
b)
Look in the parent module’s outermost scope for a scope named scope_name. If found, the item
name shall be resolved from that scope.
c)
Repeat step b), going up the hierarchy.
There is an exception to these rules for hierarchical names on the left-hand side of defparam statements.
See 12.8 for details.
For example:
In this example, there are four modules, a, b, c, and d. Each module contains an integer i. The highest level
modules in this segment of a model hierarchy are a and d. There are two copies of module b because module
a and d instantiate b. There are four copies of c.i because each of the two copies of b instantiates c twice.
module a;
integer i;
b
a_b1();
endmodule
```ebnf
upward_name_reference ::=
```

module_identifier.item_name
```ebnf
item_name ::=
```

function_identifier
| block_identifier
| net_identifier
| parameter_identifier
| port_identifier
| task_identifier
| variable_identifier
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
195
module b;
integer i;
c
b_c1(), b_c2();
initial
// downward path references two copies of i:
#10 b_c1.i = 2;
// a.a_b1.b_c1.i, d.d_b1.b_c1.i
endmodule
module c;
integer i;
initial begin
// local name references four copies of i:
i = 1;
// a.a_b1.b_c1.i, a.a_b1.b_c2.i,
// d.d_b1.b_c1.i, d.d_b1.b_c2.i
b.i = 1;
// upward path references two copies of i:
// a.a_b1.i, d.d_b1.i
end
endmodule
module d;
integer i;
b d_b1();
initial begin
// full path name references each copy of i
a.i = 1;
d.i = 5;
a.a_b1.i = 2;
d.d_b1.i = 6;
a.a_b1.b_c1.i = 3;
d.d_b1.b_c1.i = 7;
a.a_b1.b_c2.i = 4;
d.d_b1.b_c2.i = 8;
end
endmodule
### 12.7 Scope rules

The following elements define a new scope in Verilog:
—
Modules
—
Tasks
—
Functions
—
Named blocks
—
Generate blocks
An identifier shall be used to declare only one item within a scope. This rule means it is illegal to declare
two or more variables that have the same name, or to name a task the same as a variable within the same
module, or to give a gate instance the same name as the name of the net connected to its output. For generate
blocks, this rule applies regardless of whether the generate block is instantiated. An exception to this is made
for generate blocks in a conditional generate construct. See 12.4.3 for a discussion of naming conditional
generate blocks.
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
196
Copyright © 2006 IEEE. All rights reserved.
If an identifier is referenced directly (without a hierarchical path) within a task, function, named block, or
generate block, it shall be declared either within the task, function, named block, or generate block locally or
within a module, task, function, named block, or generate block that is higher in the same branch of the
name tree that contains the task, function, named block, or generate block. If it is declared locally, then the
local item shall be used; if not, the search shall continue upward until an item by that name is found or until
a module boundary is encountered. If the item is a variable, it shall stop at a module boundary; if the item is
a task, function, named block, or generate block, it continues to search higher level modules until found.
This fact means that tasks and functions can use and modify the variables within the containing module by
name, without going through their ports.
If an identifier is referenced with a hierarchical name, the path can start with a module name, instance name,
task, function, named block, or named generate block. The names shall be searched first at the current level
and then in higher level modules until found. Because both module names and instance names can be used,
precedence is given to instance names if there is a module named the same as an instance name.
Because of the upward searching, path names that are not strictly on a downward path can be used.
For example:
Example 1—In Figure 12-3, each rectangle represents a local scope. The scope available to upward
searching extends outward to all containing rectangles—with the boundary of the module A as the outer
limit. Thus block G can directly reference identifiers in F, E, and A; it cannot directly reference identifiers in
H, B, C, and D.
Example 2—The following example shows how variables can be accessed directly or with hierarchical
names:
task t;
reg s;
begin : b
  reg r;
block B
task C
func D
task E
block F
block G
block H
module A
Scopes available
to block G
Scopes not
available to
block G
Figure 12-3—Scopes available to upward name referencing
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
197
  t.b.r = 0;// These three lines access the same variable r
  b.r = 0;
  r = 0;
  t.s = 0;// These two lines access the same variable s
  s = 0;
end
endtask
### 12.8 Elaboration

Elaboration is the process that occurs between parsing and simulation. It binds modules to module
instances, builds the model hierarchy, computes parameter values, resolves hierarchical names, establishes
net connectivit, and prepares all of this for simulation. With the addition of generate constructs, the order in
which these tasks occur becomes significant.
#### 12.8.1 Order of elaboration

Because of generate constructs, the model hierarchy can depend on parameter values. Because defparam
statements can alter parameter values from almost anywhere in the hierarchy, the result of elaboration can be
ambiguous when generate constructs are involved. The final model hierarchy can depend on the order in
which defparams and generate constructs are evaluated.
The following algorithm defines an order that produces the correct hierarchy:
a)
A list of starting points is initialized with the list of top-level modules.
b)
The hierarchy below each starting point is expanded as much as possible without elaborating gener-
ate constructs. All parameters encountered during this expansion are given their final values by
applying initial values, parameter overrides, and defparam statements.
In other words, any defparam statement whose target can be resolved within the hierarchy elabo-
rated so far must have its target resolved and its value applied. defparam statements whose target
cannot be resolved are deferred until the next iteration of this step. Because no defparam inside the
hierarchy below a generate construct is allowed to refer to a parameter outside the generate con-
struct, it is possible for parameters to get their final values before going to step c).
c)
Each generate construct encountered in step b) is revisited, and the generate scheme is evaluated.
The resulting generate block instantiations make up the new list of starting points. If the new list of
starting points is not empty, go to step b).
#### 12.8.2 Early resolution of hierarchical names

In order to comply with this algorithm, hierarchical names in some defparam statements will need to be
resolved prior to the full elaboration of the hierarchy. It is possible that when elaboration is complete, rules
for name resolution would dictate that a hierarchical name in a defparam statement would have resolved
differently had early resolution not been required. This could result in a situation where an identical
hierarchical name in some other statement in the same scope would resolve differently from the one in the
defparam statement. Below is an example of a design that has this problem:
module m;
  m1 n();
endmodule
module m1;
  parameter p = 2;
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
198
Copyright © 2006 IEEE. All rights reserved.
  defparam m.n.p = 1;
  initial $display(m.n.p);
  generate
    if (p == 1) begin : m
      m2 n();
    end
  endgenerate
endmodule

module m2;
  parameter p = 3;
endmodule
In this example, the defparam must be evaluated before the conditional generate is elaborated. At this point
in elaboration, the name resolves to parameter p in module mid1, and this parameter is used in the generate
scheme. The result of the defparam is to set that parameter to 1; therefore, the generate condition is true.
After the hierarchy below the generate construct is elaborated, the rules for hierarchical name resolution
would dictate that the name should have resolved to parameter p in module mid2. In fact, the identical
name in the $display statement will resove to that other parameter.
It shall be an error if a hierarchical name in a defparam is resolved before the hierarchy is completely
elaborated and that name would resolve differently once the model is completely elaborated.
This situation will occur very rarely. In order to cause the error, there has to be a named generate block that
has the same name as one of the scopes in its full hierarchical name. Furthermore, there have to be two
instances with the same name, one in the generate block and one in the other scope with the same name as
the generate block. Then, inside these instances there have to be parameters with the same name. If this
problem occurs, it can be easily fixed by changing the name of the generate block.
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
