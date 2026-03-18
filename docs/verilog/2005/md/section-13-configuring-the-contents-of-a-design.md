---
title: "Section 13: Configuring the contents of a design"
document: "Verilog Hardware Description Language Reference Manual"
standard: "IEEE 1364-2005"
domain: "Verilog"
section: "13"
source_txt: "section-13-configuring-the-contents-of-a-design.txt"
source_pdf: "/Users/richarddje/Documents/github/Verilog-LRM-IEEE-1364-2005.pdf"
---

# Section 13: Configuring the contents of a design

IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
199
## 13. Configuring the contents of a design

### 13.1 Introduction

To facilitate both the sharing of Verilog designs between designers and/or design groups and the
repeatability of the exact contents of a given simulation (or other tool) session, the concept of configurations
is used in the Verilog language. A configuration is simply an explicit set of rules to specify the exact source
description to be used to represent each instance in a design. The operation of selecting a source
representation for an instance is referred to as binding the instance.
The example below shows a simple configuration problem.
For example:
file top.v
file adder.v
file adder.vg
module top();
module adder(...);
module adder(...);
adder a1(...);
// rtl adder
// gate-level adder
adder a2(...);
// description
// description
endmodule
...
...
endmodule
endmodule
Consider using the rtl adder description in adder.v for instance a1 in module top and the gate-level
adder description in adder.vg for instance a2. In order to specify this particular set of instance bindings
and to avoid having to change the source description to specify a new set, a configuration can be used.
config cfg1; // specify rtl adder for top.a1, gate-level adder for top.a2
design rtlLib.top;
default liblist rtlLib;
instance top.a2 liblist gateLib;
endconfig
The elements of a config are explained in subsequent subclauses, but this simple example illustrates some
important points about configs. As evidenced by the config-endconfig syntax, the config is a design
element, similar to a module, which exists in the Verilog name space. The config contains a set of rules that
are applied when searching for a source description to bind to a particular instance of the design.
A Verilog design description starts with a top-level module (or modules) (see 12.1.1). From this module’s
source description, the instantiated modules (or children) are found, then the source descriptions for the
module definitions of these subinstances shall be located, and so on until every instance in the design is
mapped to a source description.
#### 13.1.1 Library notation

In order to map a Verilog instance to a source description, the concept of a symbolic library, which is simply
a logical collection of design elements (such as modules, primitives, or configs), can be used. These design
elements can be referred to as cells. The cell name shall be the same as the name of the module/primitive/
config being processed. Syntax 13-1 specifies a cell from a given library.
Syntax 13-1—Syntax for cell
```ebnf
library_cell ::=
```

[library_identifier.]cell_identifier[:config]
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
200
Copyright © 2006 IEEE. All rights reserved.
This notation gives a symbolic method of referring to source descriptions; the method of mapping source
descriptions into libraries is shown in greater detail in 13.2.1. The optional :config extension shall be used
explicitly to refer to a config in the case where a config has the same name as a module/primitive.
For the purposes of this example, suppose the files top.v and adder.v (i.e., the RTL descriptions) have
been mapped into the library rtlLib and the file adder.vg (i.e., the gate-level description of the adder)
has been mapped into the library gateLib. The actual mechanism for mapping source descriptions to
libraries is detailed in 13.2.
#### 13.1.2 Basic configuration elements

The design statement in config cfg1 of the first example of 13.1 specifies the top-level module in the
design and what source description is to be used. In this example, the rtlLib.top notation indicates the
top-level module description shall be taken from rtlLib. Because top.v and adder.v were mapped to
this library, the actual description for the module is known to come from top.v.
The default statement coupled with the liblist clause specifies, by default, all subinstances of top (i.e.,
top.a1 and top.a2) shall be taken from rtlLib, which means the descriptions in top.v and adder.v,
which were mapped to this library, shall be used. For a basic design, which can be completely rtl, this can
be sufficient to specify completely the binding for the entire design. However, here the top.a2 instance of
adder to the gate-level description shall be bound.
The instance statement specifies, for the particular instance top.a2, the source description shall be taken
from gateLib. The instance statement overrides the default rule for this particular instance. Because
adder.vg was mapped to gateLib, this statement dictates the gate-level description in adder.vg be used
for instance top.a2.
### 13.2 Libraries

As mentioned in the previous subclause, a library is a logical collection of cells that are mapped to particular
source description files. The symbolic lib.cell[:config] notation supports the separate compilation of
source files by providing a file-system-independent name to refer to source descriptions when instances in a
design are bound. It also allows multiple tools, which can have different invocation use models, to share the
same configuration.
#### 13.2.1 Specifying libraries—the library map file

When parsing a source description file (or files), the parser shall first read the library mapping information
from a predefined file prior to reading any source files. The name of this file and the mechanism for reading
it shall be tool-specific, but all compliant tools shall provide a mechanism to specify one or more library
map files to be used for a particular invocation of the tool. If multiple map files are specified, then they shall
be read in the order in which they are specified.
For the purposes of this discussion, assume the existence of a file named lib.map in the current working
directory, which is automatically read by the parser prior to parsing any source files specified on the
command line. The syntax for declaring a library in the library map file is shown in Syntax 13-2.
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
201
Library map file details
1—file_path_spec uses file-system-specific notation to specify an absolute or relative path to a particular file or set of
files. The following shortcuts/wildcards can be used:
?
single character wildcard (matches any single character)
*
multiple character wildcard (matches any number of characters in a directory/file name)
...
hierarchical wildcard (matches any number of hierarchical directories)
..
specifies the parent directory
.
specifies the directory containing the lib.map
Paths that end in / shall include all files in the specified directory. Identical to /*.
Paths that do not begin with / are relative to the directory in which the current lib.map file is located.
2—The paths ./*.v and *.v are identical, and both specify all files with a .v suffix in the current directory.
Any file encountered by the compiler that does not match any library’s file_path_spec shall by default be
compiled into a library named work.
To perform the library mapping discussed in the example in 13.1, use the following library definitions in the
lib.map file:
library rtlLib *.v;                // matches all files in the current directory with a .v suffix
library gateLib ./*.vg;      // matches all files in the current directory with a .vg suffix
##### 13.2.1.1 File path resolution

If a file name potentially matches multiple file path specifications, the path specifications shall be resolved
in the following order:
a)
File path specifications that end with an explicit filename
b)
File path specifications that end with a wildcarded filename
c)
File path specifications that end with a directory
If a file name matches path specifications in multiple library definitions (after the above resolution rules
have been applied), it shall be an error.
Using these rules with the library definitions in the lib.map file, all source files encountered by the parser/
compiler can be mapped to a unique library. Once the source descriptions have been mapped to libraries, the
cells defined in those libraries are available for binding.
```ebnf
library_text ::= (From A.1.1)
```

{ library_description }
```ebnf
library_description ::=
```

library_declaration
| include_statement
| config_declaration
```ebnf
library_declaration ::=
```

library library_identifier file_path_spec [ { , file_path_spec } ]
[ -incdir file_path_spec { , file_path_spec } ] ;
```ebnf
include_statement ::=
```

include file_path_spec ;
Syntax 13-2—Syntax for declaring library in library map file
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
202
Copyright © 2006 IEEE. All rights reserved.
NOTE—Tool implementers may find it convenient to provide a command-line argument to explicitly specify the library
into which the file being parsed is to be mapped, which shall override any library definitions in the lib.map file. If these
libraries do not exist in the lib.map file, they can only be accessed via an explicit config.
If multiple cells with the same name map to the same library, then the LAST cell encountered shall be written
to the library. This is to support a “separate-compile” use model (see 13.4.3), where it is assumed that
encountering a cell after it has previously been compiled is intended to be a recompiling of the cell. In the
case where multiple modules with the same name are mapped to the same library in a single invocation of
the compiler, then a warning message shall be issued.
#### 13.2.2 Using multiple library map files

In addition to specifying library mapping information, a lib.map file can also include references to other
lib.map files. The include command is used to insert the entire contents of a library map file in another file
during parsing. The result is as though the contents of the included map file appear in place of the include
command.
The syntax of a lib.map file is limited to library specifications, include statements, and standard Verilog
comment syntax. Syntax 13-3 shows the syntax for the include command.
Syntax 13-3—Syntax for include command
If the file path specification, whether in an include or library statement, describes a relative path, it shall be
relative to the location of the file that contains the file path. Library providers shall include a local library
map file in addition to the source contents of the library. Individual users can then simply include the
provider’s library map file in their own map file to gain access to the contents of the provided library.
#### 13.2.3 Mapping source files to libraries

For each cell definition encountered during parsing/compiling, the name of the source file being parsed is
compared to the file path specifications of the library declarations in all of the library map files being used.
The cell is mapped into the library whose file path specification matches the source file name.
### 13.3 Configurations

As mentioned in the introduction of this clause, a configuration is simply a set of rules to apply when
searching for library cells to which to bind instances. The syntax for configurations is shown in 13.3.1.
#### 13.3.1 Basic configuration syntax

The configuration syntax is shown in Syntax 13-4.
##### 13.3.1.1 Design statement

The design statement names the library and cell of the top-level module or modules in the design hierarchy
configured by the config. There shall be one and only one design statement, but multiple top-level modules
can be listed in the design statement. The cell or cells identified cannot be configurations themselves. It is
possible the design identified can have the same name as configs, however.
The design statement shall appear before any config rule statements in the config.
```ebnf
include_statement ::= (From A.1.1)
```

include file_path_spec ;
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
203
If the library identifier is omitted, then the library that contains the config shall be used to search for the cell.
##### 13.3.1.2 The default clause

The syntax for the default clause is specified in Syntax 13-5.
Syntax 13-5—Syntax for default clause
The default clause selects all instances that do not match a more specific selection clause. The use
expansion clause (see 13.3.1.6) cannot be used with a default selection clause. For other expansion clauses,
there cannot be more than one default clause that specifies the expansion clause.
For simple design configurations, it might be sufficient to specify a default liblist (see 13.3.1.5).
##### 13.3.1.3 The instance clause

The instance clause is used to specify the specific instance to which the expansion clause shall apply. The
syntax for the instance clause is specified in Syntax 13-6.
Syntax 13-6—Syntax for instance clause
The instance name associated with the instance clause is a Verilog hierarchical name, starting at the top-
level module of the config (i.e., the name of the cell in the design statement).
```ebnf
config_declaration ::= (From A.1.5)
```

config config_identifier ;
design_statement
{config_rule_statement}
endconfig
```ebnf
design_statement ::=
```

design { [library_identifier.]cell_identifier } ;
```ebnf
config_rule_statement ::=
```

default_clause liblist_clause ;
| inst_clause liblist_clause ;
| inst_clause use_clause  ;
| cell_clause liblist_clause ;
| cell_clause use_clause ;
Syntax 13-4—Syntax for configuration
```ebnf
default_clause ::= (From A.1.5)
```

default
```ebnf
inst_clause ::= (From A.1.5)
```

instance inst_name
```ebnf
inst_name ::=
```

topmodule_identifier{.instance_identifier}
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
204
Copyright © 2006 IEEE. All rights reserved.
##### 13.3.1.4 The cell clause

The cell selection clause names the cell to which it applies. The syntax for the cell clause is specified in
Syntax 13-7.
If the optional library name is specified, then the selection rule applies to any instance that is bound or is
under consideration for being bound to the selected library and cell. It is an error if a library name is
included in a cell selection clause and the corresponding expansion clause is a library list expansion clause.
##### 13.3.1.5 The liblist clause

The liblist clause defines an ordered set of libraries to be searched to find the current instance. The syntax
for the liblist clause is specified in Syntax 13-8.
Syntax 13-8—Syntax for liblist clause
liblists are inherited hierarchically downward as instances are bound. When searching for a cell to bind to
the current unbound instance, and in the absence of an applicable binding expansion clause, the specified
library list is searched in the specified order.
The current library list is selected by the selection clauses. If no library list clause is selected or if the
selected library list is empty, then the library list contains the single name that is the library in which the cell
containing the unbound instance is found (i.e., the parent cell’s library).
##### 13.3.1.6 The use clause

The use clause specifies a specific binding for the selected cell. The syntax for the use clause is specified in
Syntax 13-9.
Syntax 13-9—Syntax for use clause
A use clause can only be used in conjunction with an instance or cell selection clause. It specifies the exact
library and cell to which a selected cell or instance is bound.
The use clause has no effect on the current value of the library list. It can be common in practice to specify
multiple config rule statements, one of which specifies a binding and the other of which specifies a library
list.
```ebnf
cell_clause ::= (From A.1.5)
```

cell [ library_identifier.]cell_identifier
Syntax 13-7—Syntax for cell clause
```ebnf
liblist_clause ::= (From A.1.5)
```

liblist { library_identifier }
```ebnf
use_clause ::= (From A.1.5)
```

use [library_identifier.]cell_identifier[:config]
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
205
If the lib.cell to which the use clause refers is a config that has the same name as a module/primitive in the
same library, then the optional :config suffix can be added to the lib.cell to specify the config
explicitly.
If the library name is omitted, the library shall be inherited from the parent cell.
NOTE—The binding statement can create situations where the unbound instance’s module name and the cell name to
which it is bound are different.
#### 13.3.2 Hierarchical configurations

For situations where it is desirable to specify a special set of configuration rules for a subsection of a design,
it is possible to bind a particular instance directly to a configuration using the binding clause:
instance top.a1.foo use lib1.foo:config;
// bind to the config foo in library lib1
specifies the instance top.a1.foo is to be replaced with the design hierarchy specified by the configuration
lib1.foo:config. The design statement in lib1.foo:config shall specify the actual binding for the
instance top.a1.foo, and the rules specified in the config shall determine the configuration of all other
subinstances under top.a1.foo.
It shall be an error for an instance clause to specify a hierarchical path to an instance that occurs within a
hierarchy specified by another config.
config bot;
design lib1.bot;
default liblist lib1 lib2;
instance bot.a1 liblist lib3;
endconfig
config top;
design lib1.top;
default liblist lib2 lib1;
instance top.bot use lib1.bot:config;
instance top.bot.a1 liblist lib4;
// ERROR - cannot set liblist for top.bot.a1 from this config
endconfig
### 13.4 Using libraries and configs

This subclause describes potential use models for referencing configs on the command line. It is included for
clarification purposes.
The traditional Verilog simulation use model takes a file-based approach, where the source descriptions for
all cells in the design are specified on the command line for each invocation of the tool. With the advent of
compiled-code simulators, the configuration mechanism shall also support a use model that allows for the
source files to be precompiled and then for the precompiled design objects to be referenced on the command
line. This subclause explains how configurations can be used in both of these scenarios.
#### 13.4.1 Precompiling in a single-pass use model

The single-pass use model is the traditional use model with which most users are familiar. In this use model,
all of the source description files shall be provided to the simulator via the command line, and only these
source descriptions can be used to bind cell instances in the current design. A precompiling strategy in this
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
206
Copyright © 2006 IEEE. All rights reserved.
scenario actually parses every cell description provided on the command line and maps it into the library
without regard to whether the cell actually is used in the design. The tool can optionally check to see
whether the cell already exists in the library and, if it is up-to-date (i.e., the source description has not
changed since the last time the cell was compiled), can skip recompiling the cell. After all cells on the
command line have been compiled, then the tool can locate the top-level cell (discussed in Clause 12) and
proceed down the hierarchy, binding each instance as it is encountered in the hierarchy.
NOTE—With this use model, it is not necessary for library objects to persist from one tool invocation to another
(although for performance considerations it is recommended they do).
#### 13.4.2 Elaboration-time compiling in a single-pass use model

An alternate strategy that can be used with a single-pass tool is to parse the source files only to find the top-
level module(s), without actually compiling anything into the library during this scanning process. Once the
top-level module(s) has been found, then it can be compiled into the library, and the tool can proceed down
the hierarchy, only compiling the source descriptions necessary to bind the design successfully. Based on the
binding rules in place, only the source files that match the current library specification need to be parsed to
find the current cell’s source description to compile. As with the precompiled single-pass use model, it is not
necessary for library cells to persist from one invocation to another using this strategy.
#### 13.4.3 Precompiling using a separate compilation tool

When using a separate compilation tool, it is essential that library cells persist, and the compiled forms shall,
therefore, exist somewhere in the file system. The exact format and location for holding these compiled
forms shall be vendor- or tool-specific. Using this separate compiler strategy, the source descriptions shall
be parsed and compiled into the library using one or more invocations of the compiler tool. The only
restriction is that all cells in a design shall be precompiled prior to binding the design (typically via an
invocation of a separate tool). Using this strategy, the tool that actually does the binding only needs to be
told the top-level module(s) of the design to be bound, and then it shall use the precompiled form of the cell
description(s) from the library to determine the subinstances and descend hierarchically down the design,
binding each cell as it is located.
#### 13.4.4 Command line considerations

In each of the three preceding strategies, either the binding rules can be specified via a config, or the default
rules (from the library map file) can be used. In the single-pass use models, the config can be specified by
including its source description file on the command line. In the case where the config includes a design
statement, then the specified cell shall be the top-level module, regardless of the presence of any
uninstantiated cells in the rest of the source files. When using a separate compilation tool, the tool that
actually does the binding only needs to be given the lib.cell specification for the top-level cell(s) and/or the
config to be used. In this strategy, the config itself shall also be precompiled.
### 13.5 Configuration examples

Consider the following set of source descriptions:
file top.v
module top(...);
...
adder a1(...);
adder a2(...);
endmodule
module foo(...);
... // rtl
endmodule
file adder.v
module adder(...);
... // rtl
foo f1(...);
foo f2(...);
endmodule
module foo(...);
... // rtl
endmodule
file adder.vg
module adder(...);
... // gate-level
foo f1(...);
foo f2(...);
endmodule
module foo(...);
... // gate-level
endmodule
file lib.map
library rtlLib top.v;
library aLib adder.*;
library gateLib
adder.vg;
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
207
All of the examples in this subclause shall assume the top.v, adder.v and adder.vg files get compiled
with the given lib.map file. This yields the following library structure:
rtlLib.top // from top.v
rtlLib.foo // from top.v
aLib.adder // from adder.v
aLib.foo // rtl from adder.v
gateLib.adder // from adder.vg
gateLib.foo // from adder.vg
#### 13.5.1 Default configuration from library map file

With no configuration, the libraries are searched according to the library declaration order in the library map
file. In other words, all instances of module adder shall use aLib.adder (because aLib is the first library
specified that contains a cell named adder), and all instances of module foo shall use rtlLib.foo
(because rtlLib is the first library that contains foo).
#### 13.5.2 Using default clause

To always use the foo definition from file adder.v, use the following simple configuration:
config cfg1;
design rtlLib.top ;
default liblist aLib rtlLib;
endconfig
The default liblist statement overrides the library search order in the lib.map file; therefore, aLib is
always searched before rtlLib. Because the gateLib library is not included in the liblist, the gate-
level descriptions of adder and foo shall not be used.
To use the gate-level representations of adder and foo, add to the config as follows:
config cfg2;
design rtlLib.top ;
default liblist gateLib aLib rtlLib;
endconfig
This shall cause the gate representation always to be taken before the rtl representation, using the module
definitions for adder and foo from adder.vg. The rtl view of top shall be taken because there is no gate
representation available.
#### 13.5.3 Using cell clause

To modify the config to use the rtl view of adder and the gate-level representation of foo from gateLib,
use the following:
config cfg3;
design rtlLib.top ;
default liblist aLib rtlLib;
cell foo use gateLib.foo;
endconfig
The cell clause selects all cells named foo and explicitly binds them to the gate representation in gateLib.
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
208
Copyright © 2006 IEEE. All rights reserved.
#### 13.5.4 Using instance clause

To modify the config so the top.a1 adder (and its descendants) use the gate representation and the
top.a2 adder (and its descendants), use the rtl representation from aLib:
config cfg4
design rtlLib.top ;
default liblist gateLib rtlLib;
instance top.a2 liblist aLib;
endconfig
Because the liblist is inherited, all of the descendants of top.a2 inherit its liblist from the instance selection
clause.
#### 13.5.5 Using hierarchical config

Now suppose all this work has only been on the adder module by itself and a config that uses the
rtlLib.foo cell for f1, and the gateLib.foo cell for f2 has already been developed. Then use the
following:
config cfg5;
design aLib.adder;
default liblist gateLib aLib;
instance adder.f1 liblist rtlLib;
endconfig
To use this configuration cfg5 for the top.a2 instance of adder and take the full default aLib adder for
the top.a1 instance, use the following config:
config cfg6;
design rtlLib.top;
default liblist aLib rtlLib;
instance top.a2 use work.cfg5:config ;
endconfig
The binding clause specifies the work.cfg5:config configuration is to be used to resolve the bindings of
instance top.a2 and its descendants. It is the design statement in config cfg5 that defines the exact binding
for the top.a2 instance itself. The rest of cfg5 defines the rules to bind the descendants of top.a2. Notice
the instance clause in cfg5 is relative to its own top-level module, adder.
### 13.6 Displaying library binding information

It shall be possible to display the actual library binding information for module instances during simulation.
The format specifier %l or %L shall print out the library.cell binding information for the module
instance containing the display (or other textual output) command. This is similar to the %m format specifier,
which prints out the hierarchical path name of the module containing it.
It shall also be able to use VPI to display the binding information. The following VPI properties shall exist
for objects of type vpiModule:
—
vpiLibrary—the library name into which the module was compiled
—
vpiCell—the name of the cell bound to the module instance
—
vpiConfig—the library.cell name of the config controlling the binding of the module
instance
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
209
These properties shall be of string type, similar to the vpiName and vpiFullName properties.
### 13.7 Library mapping examples

In the absence of a configuration, it is possible to perform basic control of the library searching order when
binding a design.
When a config is used, the config overrides the rules specified in this subclause.
#### 13.7.1 Using the command line to control library searching

In the absence of a configuration, it shall be necessary for all compliant tools to provide a mechanism of
specifying a library search order on the command line that overrides the default order from the library map
file. This mechanism shall include specification of library names only, with the definitions of these libraries
to be taken from the library map file.
NOTE—It is recommended all compliant tools use “-L <library_name>” to specify this search order.
#### 13.7.2 File path specification examples

For example:
Given the following set of files:
/proj/lib1/rtl/a.v
/proj/lib2/gates/a.v
/proj/lib1/rtl/b.v
/proj/lib2/gates/b.v
From the /proj library, the following absolute file_path_specs are resolved as shown:
/proj/lib*/*/a.v =/proj/lib1/rtl/a.v, /proj/lib2/gates/a.v
.../a.v =/proj/lib1/rtl/a.v, /proj/lib2/gates/a.v
/proj/.../b.v =/proj/lib1/rtl/b.v, /proj/lib2/gates/b.v
.../rtl/*.v =/proj/lib1/rtl/a.v, /proj/lib1/rtl/b.v
From the /proj/lib1 directory, the following relative file_path_specs are resolved as shown:
../lib2/gates/*.v = /proj/lib2/gates/a.v, /proj/lib2/gates/b.v
./rtl/?.v = /proj/lib1/rtl/a.v, /proj/lib1/rtl/b.v
./rtl/ = /proj/lib1/rtl/a.v, /proj/lib1/rtl/b.v
#### 13.7.3 Resolving multiple path specifications

For example:
library lib1 "/proj/lib1/foo*.v";
library lib2 "/proj/lib1/foo.v";
library lib3 "../lib1/";
library lib4 "/proj/lib1/*ver.v";
When evaluated from the directory /proj/tb directory, the following source files shall map into the
specified library:
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
210
Copyright © 2006 IEEE. All rights reserved.
../lib1/foobar.v - lib1
// potentially matches lib1 and lib3. Because lib1
includes  a filename and lib3 only specifies a directory; lib1 takes
precedence
/proj/lib1/foo.v - lib2
// takes precedence over lib1 and lib3 path specifications
/proj/lib1/bar.v -
lib3
/proj/lib1/barver.v -
lib4 // takes precedence over lib3 path specification
/proj/lib1/foover.v -
ERROR // matches lib1 and lib4
/test/tb/tb.v -
work // does not match any library specifications.
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
