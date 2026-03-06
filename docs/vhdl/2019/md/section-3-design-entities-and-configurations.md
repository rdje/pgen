---
title: "Section 3: IEEE Standard for VHDL Language Reference Manual"
document: "VHDL Language Reference Manual"
standard: "IEEE 1076-2019"
domain: "VHDL"
section: "3"
source_txt: "section-3-design-entities-and-configurations.txt"
source_pdf: "/Users/richarddje/Documents/github/VHDL-LRM-IEEE-1076-2019.pdf"
---

# Section 3: IEEE Standard for VHDL Language Reference Manual

IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
19
Copyright © 2019 IEEE. All rights reserved.
3. Design entities and configurations
### 3.1 General

The design entity is the primary hardware abstraction in VHDL. It represents a portion of a hardware design
that has well-defined inputs and outputs and performs a well-defined function. A design entity may
represent an entire system, a subsystem, a board, a chip, a macro-cell, a logic gate, or any level of abstraction
in between. A configuration can be used to describe how design entities are put together to form a complete
design.
A design entity may be described in terms of a hierarchy of blocks, each of which represents a portion of the
whole design. The top-level block in such a hierarchy is the design entity itself; such a block is an external
block that resides in a library and may be used as a component of other designs. Nested blocks in the
hierarchy are internal blocks, defined by block statements (see 11.2).
A design entity may also be described in terms of interconnected components. Each component of a design
entity may be bound to a lower-level design entity in order to define the structure or behavior of that
component. Successive decomposition of a design entity into components, and binding those components to
other design entities that may be decomposed in like manner, results in a hierarchy of design entities
representing a complete design. Such a collection of design entities is called a design hierarchy. The
bindings necessary to identify a design hierarchy can be specified in a configuration of the top-level entity in
the hierarchy.
This clause describes the way in which design entities and configurations are defined. A design entity is
defined by an entity declaration together with a corresponding architecture body. A configuration is defined
by a configuration declaration.
### 3.2 Entity declarations

#### 3.2.1 General

An entity declaration defines the interface between a given design entity and the environment in which it is
used. It may also specify declarations and statements that are part of the design entity. A given entity
declaration may be shared by many design entities, each of which has a different architecture. Thus, an
entity declaration can potentially represent a class of design entities, each with the same interface.
```ebnf
entity_declaration ::=
```

entity identifier is
entity_header
entity_declarative_part
[ begin entity_statement_part ]
end [ entity ] [ entity_simple_name ] ;
The entity header and entity declarative part consist of declarative items that pertain to each design entity
whose interface is defined by the entity declaration. The entity statement part, if present, consists of
concurrent statements that are present in each such design entity.
If a simple name appears at the end of an entity declaration, it shall repeat the identifier of the entity
declaration.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
20
Copyright © 2019 IEEE. All rights reserved.
#### 3.2.2 Entity header

The entity header declares objects used for communication between a design entity and its environment.
```ebnf
entity_header ::=
```

[ formal_generic_clause ]
[ formal_port_clause ]
The generic list in the formal generic clause defines generics whose associated actuals may be determined
by the environment (see 6.5.6.2). The port list in the formal port clause defines the input and output ports of
the design entity (see 6.5.6.3).
In certain circumstances, the names of generics and ports declared in the entity header become visible
outside of the design entity (see 12.2 and 12.3).
Examples:
—
An entity declaration with port declarations only:
entity Full_Adder is
port (X, Y, Cin: in Bit; Cout, Sum: out Bit);
end Full_Adder;
—
An entity declaration with generic declarations also:
entity AndGate is
generic (N: Natural := 2)
port(
inputs: in Bit_Vector (1 to N);
result: out Bit);
end entity AndGate;
—
An entity declaration with neither:
entity TestBench is
end TestBench;
#### 3.2.3 Entity declarative part

The entity declarative part of a given entity declaration declares items that are common to all design entities
whose interfaces are defined by the given entity declaration.
```ebnf
entity_declarative_part ::=
```

{ entity_declarative_item }
```ebnf
entity_declarative_item ::=
```

subprogram_declaration
|   subprogram_body
|   subprogram_instantiation_declaration
|   package_declaration

|   package_body
|   package_instantiation_declaration
|   type_declaration
|   subtype_declaration
|   mode_view_declaration
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
21
Copyright © 2019 IEEE. All rights reserved.
|   constant_declaration
|
signal_declaration
|
shared_variable_declaration
|   file_declaration
|   alias_declaration
|   attribute_declaration
|   attribute_specification
|   disconnection_specification
|   use_clause
|   group_template_declaration
|   group_declaration
|   PSL_Property_Declaration
|   PSL_Sequence_Declaration
|   PSL_Clock_Declaration
Names declared by declarative items in the entity declarative part of a given entity declaration are visible
within the bodies of corresponding design entities, as well as within certain portions of a corresponding
configuration declaration.
The various kinds of declaration are described in Clause 6, and the various kinds of specification are
described in Clause 7. The use clause, which makes externally defined names visible within the block, is
described in Clause 12.
Example:
—
An entity declaration with entity declarative items:
entity ROM is
port (
Addr: in Word;
Data: out Word;
Sel: in Bit);
type Instruction is array (1 to 5) of Natural;
type Program is array (Natural range <>) of Instruction;
use Work.OpCodes.all, Work.RegisterNames.all;
constant ROM_Code: Program :=
(
(STM, R14, R12, 12, R13),
(LD,  R7,  32,  0,  R1 ),
(BAL, R14, 0,   0,  R7 ),
·
·    --  etc.
·
) ;
end ROM;
NOTE—The entity declarative part of a design entity whose corresponding architecture is decorated with the 'FOREIGN
attribute is subject to special elaboration rules. See 14.4.1.9
#### 3.2.4 Entity statement part

The entity statement part contains concurrent statements that are common to each design entity with this
interface.
9Notes in text, tables, and figures are given for information only and do not contain requirements needed to implement the standard.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
22
Copyright © 2019 IEEE. All rights reserved.
```ebnf
entity_statement_part ::=
```

{ entity_statement }
```ebnf
entity_statement ::=
```

concurrent_assertion_statement
|   passive_concurrent_procedure_call_statement

|   passive_process_statement

|   PSL_Directive
It is an error if any statements other than concurrent assertion statements, concurrent procedure call
statements, process statements, or PSL directives appear in the entity statement part. All entity statements
shall be passive (see 11.3). Such statements may be used to monitor the operating conditions or
characteristics of a design entity.
Example:
—
An entity declaration with statements:
entity Latch is
port (
Din:    in Word;
Dout: out Word;
Load: in Bit;
Clk:  in Bit );
constant Setup: Time := 12 ns;
constant PulseWidth: Time := 50 ns;
use Work.TimingMonitors.all;
begin
assert Clk='1' or Clk'Delayed'Stable (PulseWidth);
CheckTiming (Setup, Din, Load, Clk);
end;
NOTE—The entity statement part of a design entity whose corresponding architecture is decorated with the 'FOREIGN
attribute is subject to special elaboration rules. See 14.5.1.
### 3.3 Architecture bodies

#### 3.3.1 General

An architecture body defines the body of a design entity. It specifies the relationships between the inputs and
outputs of a design entity and may be expressed in terms of structure, dataflow, or behavior. Such
specifications may be partial or complete.
```ebnf
architecture_body ::=
```

architecture identifier of entity_name is
architecture_declarative_part
begin
architecture_statement_part
end [ architecture ] [ architecture_simple_name ] ;
The identifier defines the simple name of the architecture body; this simple name distinguishes architecture
bodies associated with the same entity declaration.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
23
Copyright © 2019 IEEE. All rights reserved.
The entity name identifies the name of the entity declaration that defines the interface of this design entity.
For a given design entity, both the entity declaration and the associated architecture body shall reside in the
same library.
If a simple name appears at the end of an architecture body, it shall repeat the identifier of the architecture
body.
More than one architecture body may exist corresponding to a given entity declaration. Each declares a
different body with the same interface; thus, each together with the entity declaration represents a different
design entity with the same interface.
NOTE—Two architecture bodies that are associated with different entity declarations may have the same simple name,
even if both architecture bodies (and the corresponding entity declarations) reside in the same library.
#### 3.3.2 Architecture declarative part

The architecture declarative part contains declarations of items that are available for use within the block
defined by the design entity.
```ebnf
architecture_declarative_part ::=
```

{ block_declarative_item }
```ebnf
block_declarative_item ::=
```

subprogram_declaration
|   subprogram_body
|   subprogram_instantiation_declaration
|   package_declaration
|
package_body
|   package_instantiation_declaration
|   type_declaration
|   subtype_declaration
|   mode_view_declaration
|   constant_declaration
|   signal_declaration
|   shared_variable_declaration
|   file_declaration
|   alias_declaration
|   component_declaration
|   attribute_declaration
|   attribute_specification
|   configuration_specification
|   disconnection_specification
|   use_clause
|   group_template_declaration
|   group_declaration
|   PSL_Property_Declaration
|   PSL_Sequence_Declaration
|   PSL_Clock_Declaration
The various kinds of declaration are described in Clause 6, and the various kinds of specification are
described in Clause 7. The use clause, which makes externally defined names visible within the block, is
described in Clause 12.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
24
Copyright © 2019 IEEE. All rights reserved.
NOTE—The declarative part of an architecture decorated with the 'FOREIGN attribute is subject to special elaboration
rules. See 14.4.1.
#### 3.3.3 Architecture statement part

The architecture statement part contains statements that describe the internal organization and/or operation
of the block defined by the design entity.
```ebnf
architecture_statement_part ::=
```

{ concurrent_statement }
All of the statements in the architecture statement part are concurrent statements, which execute
asynchronously with respect to one another. The various kinds of concurrent statements are described in
Clause 11.
Examples:
—
A body of entity Full_Adder:
architecture DataFlow of Full_Adder is
signal A,B: Bit;
begin
A <= X xor Y;
B <= A and Cin;
Sum <= A xor Cin;
Cout <= B or (X and Y);
end architecture DataFlow;
—
A body of entity TestBench:
library Test;
use Test.Components.all;
architecture Structure of TestBench is
component Full_Adder
port (X, Y, Cin: Bit; Cout, Sum: out Bit);
end component;
signal A,B,C,D,E,F,G: Bit;
signal OK: Boolean;
begin
UUT:
Full_Adder port map (A,B,C,D,E);
Generator:
AdderTest  port map (A,B,C,F,G);
Comparator:
AdderCheck port map (D,E,F,G,OK);
end Structure;
—
A body of entity AndGate:
 architecture Behavior of AndGate is
begin
process (Inputs)
variable Temp: Bit;
begin
Temp := '1';
for i in Inputs'Range loop
if Inputs(i) = '0' then
Temp := '0';
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
25
Copyright © 2019 IEEE. All rights reserved.
exit;
end if;
end loop;
Result <= Temp after 10 ns;
end process;
end Behavior;
NOTE—The statement part of an architecture decorated with the 'FOREIGN attribute is subject to special elaboration
rules. See 14.5.1.
### 3.4 Configuration declarations

#### 3.4.1 General

The binding of component instances to design entities is performed by configuration specifications (see 7.3);
such specifications appear in the declarative part of the block in which the corresponding component
instances are created. In certain cases, however, it may be appropriate to leave unspecified the binding of
component instances in a given block and to defer such specification until later. A configuration declaration
provides the mechanism for specifying such deferred bindings.
```ebnf
configuration_declaration ::=
```

configuration identifier of entity_name is
configuration_declarative_part
{ verification_unit_binding_indication ; }
block_configuration
end [ configuration ] [ configuration_simple_name ] ;
```ebnf
configuration_declarative_part ::=
```

{ configuration_declarative_item }
```ebnf
configuration_declarative_item ::=
```

use_clause
|   attribute_specification

|   group_declaration
The entity name identifies the name of the entity declaration that defines the design entity at the root of the
design hierarchy.
If a simple name appears at the end of a configuration declaration, it shall repeat the identifier of the
configuration declaration.
A verification unit binding indication in a configuration declaration binds one or more PSL verification units
to the design entity at the root of the design hierarchy. Verification unit binding indications are described   in
7.3.4.
NOTE 1—A configuration declaration achieves its effect entirely through elaboration (see Clause 14). There are no
behavioral semantics associated with a configuration declaration.
NOTE 2—A given configuration may be used in the definition of another, more complex configuration.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
26
Copyright © 2019 IEEE. All rights reserved.
Examples:
—
An architecture of a microprocessor:
architecture Structure_View of Processor is
component ALU port ( ··· ); end component;
component MUX port ( ··· ); end component;
component Latch port ( ··· ); end component
begin
A1: ALU port map ( ··· );
M1: MUX port map ( ··· );
M2: MUX port map ( ··· );
M3: MUX port map ( ··· );
L1: Latch port map ( ··· );
L2: Latch port map ( ··· );
end Structure_View;
—
A configuration of the microprocessor:
library TTL, Work;
configuration V4_27_87 of Processor is
use Work.all;
for Structure_View
for A1: ALU
use configuration TTL.SN74LS181;
end for;
for M1,M2,M3: MUX
use entity Multiplex4 (Behavior);
end for;
for all: Latch
-- use defaults
end for;
end for;
end configuration V4_27_87;
#### 3.4.2 Block configuration

A block configuration defines the configuration of a block. Such a block is either an internal block defined
by a block statement or an external block defined by a design entity. If the block is an internal block, the
defining block statement is either an explicit block statement or an implicit block statement that is itself
defined by a generate statement.
```ebnf
block_configuration ::=
```

for block_specification
{ use_clause }
{ configuration_item }
end for ;
```ebnf
block_specification ::=
```

architecture_name
|   block_statement_label
|   generate_statement_label [ ( generate_specification ) ]
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
27
Copyright © 2019 IEEE. All rights reserved.
```ebnf
generate_specification ::=
```

static_discrete_range
|   static_expression
|   alternative_label
```ebnf
configuration_item ::=
```

block_configuration
|   component_configuration
The block specification identifies the internal or external block to which this block configuration applies.
If a block configuration appears immediately within a configuration declaration, then the block specification
of that block configuration shall be an architecture name, and that architecture name shall denote a design
entity body whose interface is defined by the entity declaration denoted by the entity name of the enclosing
configuration declaration.
If a block configuration appears immediately within a component configuration, then the corresponding
components shall be fully bound (see 7.3.2.2), the block specification of that block configuration shall be an
architecture name, and that architecture name shall denote the same architecture body as that to which the
corresponding components are bound.
If a block configuration appears immediately within another block configuration, then the block
specification of the contained block configuration shall be a block statement or generate statement label, and
the label shall denote a block statement or generate statement that is contained immediately within the block
denoted by the block specification of the containing block configuration.
If the scope of a declaration (see 12.2) includes the end of the declarative part of a block corresponding to a
given block configuration, then the scope of that declaration extends to each configuration item contained in
that block configuration, with the exception of block configurations that configure external blocks.
Similarly, if a declaration is visible (either directly or by selection) at the end of the declarative part of a
block corresponding to a given block configuration, then the declaration is visible in each configuration item
contained in that block configuration, with the exception of block configurations that configure external
blocks. Additionally, if a given declaration is a homograph of a declaration that a use clause in the block
configuration makes potentially directly visible, then the given declaration is not directly visible in the block
configuration or any of its configuration items. See 12.3.
For any name that is the label of a block statement appearing immediately within a given block, a
corresponding block configuration may appear as a configuration item immediately within a block
configuration corresponding to the given block. For any collection of names that are labels of instances of
the same component appearing immediately within a given block, a corresponding component configuration
may appear as a configuration item immediately within a block configuration corresponding to the given
block.
For any name that is the label of a generate statement immediately within a given block, one or more
corresponding block configurations may appear as configuration items immediately within a block
configuration corresponding to the given block. Such block configurations apply to implicit blocks
generated by that generate statement. If such a block configuration contains a generate specification that is a
static discrete range, then the block configuration applies to those implicit block statements that are
generated for the specified range of values of the corresponding generate parameter; the discrete range has
no significance other than to define the set of generate statement parameter values implied by the discrete
range. If such a block configuration contains a generate specification that is a static expression, then the
block configuration applies only to the implicit block statement generated for the specified value of the
corresponding generate parameter. If such a block configuration contains a generate specification that is an
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
28
Copyright © 2019 IEEE. All rights reserved.
alternative label, then the block configuration applies only to the implicit block generated for the generate
statement body following the alternative label in the generate statement, if and only if the condition after the
alternative label evaluates to TRUE (for an if generate statement) or the case generate alternative containing
the alternative label is the chosen alternative (for a case generate statement). If no generate specification
appears in such a block configuration, then it applies to exactly one of the following sets of blocks:
—
All implicit blocks (if any) generated by the corresponding generate statement, if and only if the
corresponding generate statement is a for generate statement.
—
The implicit block generated by the corresponding generate statement, if and only if the
corresponding generate statement is an if generate statement and if the first condition after if
evaluates to TRUE.
—
No implicit or explicit blocks, if and only if the corresponding generate statement is an if generate
statement and the first condition after if evaluates to FALSE.
If the block specification of a block configuration contains a generate statement label, and if this label
contains a generate specification, then:
—
If the generate specification is a discrete range or an expression, then it is an error if the generate
statement denoted by the generate statement label is not a for generate statement. Moreover, for a
generate specification that is a discrete range, it is an error if the type of the discrete range is not the
same as the type of the discrete range of the generate parameter specification and if any value in the
range does not belong to the discrete range of the generate parameter specification. Similarly, for a
generate specification that is an expression, it is an error if the type of the expression is not the same
as the type of the discrete range of the generate parameter specification and if the value of the
expression does not belong to the discrete range of the generate parameter specification.
—
If the generate specification is an alternative label, then it is an error if the generate statement
denoted by the generate statement label is not an if generate statement that includes the alternative
label or a case generate statement that includes the alternative label.
If the block specification of a block configuration contains a generate statement label that denotes an if
generate statement, and if the first condition after if has an alternative label, then it is an error if the generate
statement label does not contain a generate specification that is an alternative label. Similarly, if the block
specification of a block configuration contains a generate statement label that denotes a case generate
statement, then it is an error if the generate statement label does not contain a generate specification that is
an alternative label.
Within a given block configuration, whether implicit or explicit, an implicit block configuration is assumed
to appear for any block statement that appears within the block corresponding to the given block
configuration, if no explicit block configuration appears for that block statement. Similarly, an implicit
component configuration is assumed to appear for each component instance that appears within the block
corresponding to the given block configuration, if no explicit component configuration appears for that
instance. Such implicit configuration items are assumed to appear following all explicit configuration items
in the block configuration.
It is an error if, in a given block configuration, more than one configuration item is defined for the same
block or component instance.
NOTE 1—As a result of the rules described in the preceding paragraphs and in Clause 12, a simple name that is visible
by selection at the end of the declarative part of a given block is also visible by selection within any configuration item
contained in a corresponding block configuration. If such a name is directly visible at the end of the given block
declarative part, it will likewise be directly visible in the corresponding configuration items, unless a use clause for a
different declaration with the same simple name appears in the corresponding configuration declaration, and the scope of
that use clause encompasses all or part of those configuration items. If such a use clause appears, then the name will be
directly visible within the corresponding configuration items except at those places that fall within the scope of the
additional use clause (at which places neither name will be directly visible).
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
29
Copyright © 2019 IEEE. All rights reserved.
NOTE 2—If an implicit configuration item is assumed to appear within a block configuration, that implicit
configuration item will never contain explicit configuration items.
NOTE 3—If the block specification in a block configuration specifies a generate statement label, and if this label
contains a generate specification that is a discrete range, then the direction specified or implied by the discrete range has
no significance other than to define, together with the bounds of the range, the set of generate statement parameter values
denoted by the range. Thus, the following two block configurations are equivalent:
for Adders(31 downto 0) ··· end for;
for Adders(0 to 31) ··· end for;
NOTE 4—A block configuration is allowed to appear immediately within a configuration declaration only if the entity
declaration denoted by the entity name of the enclosing configuration declaration has associated architectures.
Furthermore, the block specification of the block configuration will denote one of these architectures.
Examples:
—
A block configuration for a design entity:
for ShiftRegStruct
-- An architecture name.
-- Configuration items
-- for blocks and components
-- within ShiftRegStruct.
end for;
—
A block configuration for a block statement:
for B1
-- A block label.
-- Configuration items
-- for blocks and components
-- within block B1.
end for;
#### 3.4.3 Component configuration

A component configuration defines the configuration of one or more component instances in a
corresponding block.
```ebnf
component_configuration ::=
```

for component_specification

[ binding_indication ; ]

{ verification_unit_binding_indication ; }

[ block_configuration ]
    end for;
The component specification (see 7.3) identifies the component instances to which this component
configuration applies. A component configuration that appears immediately within a given block
configuration applies to component instances that appear immediately within the corresponding block.
It is an error if two component configurations apply to the same component instance.
If the component configuration contains a binding indication (see 7.3.2), then the component configuration
implies a configuration specification for the component instances to which it applies. This implicit
configuration specification has the same component specification and binding indication as that of the
component configuration.
If a given component instance is unbound in the corresponding block, then any explicit component
configuration for that instance that does not contain an explicit binding indication will contain an implicit,
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
30
Copyright © 2019 IEEE. All rights reserved.
default binding indication (see 7.3.3). Similarly, if a given component instance is unbound in the
corresponding block, then any implicit component configuration for that instance will contain an implicit,
default binding indication.
A verification unit binding indication in a configuration declaration binds one or more PSL verification units
to the instance of the design entity bound to the component instances identified by the component
specification. Verification unit binding indications are described in 7.3.4.
It is an error if a component configuration contains an explicit block configuration and the component
configuration does not bind all identified component instances to the same design entity.
Within a given component configuration, whether implicit or explicit, an implicit block configuration is
assumed for the design entity to which the corresponding component instance is bound, if no explicit block
configuration appears and if the corresponding component instance is fully bound.
Examples:
—
A component configuration with binding indication:
for all: IOPort
use entity StdCells.PadTriState4 (DataFlow)
port map (Pout=>A, Pin=>B, IO=>Dir, Vdd=>Pwr, Gnd=>Gnd);
end for;
—
A component configuration containing block configurations:
for D1: DSP
for DSP_STRUCTURE
-- binding specified in design entity or else defaults.
for Filterer
-- Configuration items for filtering components.
end for;
for Processor
-- Configuration items for processing components.
end for;
end for;
end for;
NOTE—The requirement that all component instances corresponding to a block configuration be bound to the same
design entity makes the following configuration illegal:
architecture A of E is
component C is end component C;
for L1: C use entity E1(X);
for L2: C use entity E2(X);
begin
L1: C;
L2: C;
end architecture A;
configuration Illegal of Work.E is
for A
for all: C

for X
-- Does not apply to the same design entity

-- in all instances of C.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
31
Copyright © 2019 IEEE. All rights reserved.
               ···

end for;     -- X

end for; -- C
end for; -- A
end configuration Illegal;
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
