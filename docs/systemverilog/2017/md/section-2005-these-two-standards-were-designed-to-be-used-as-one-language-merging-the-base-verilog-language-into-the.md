---
title: "Section 2005: IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language"
document: "SystemVerilog Language Reference Manual"
standard: "IEEE 1800-2017"
domain: "SystemVerilog"
section: "2005"
source_txt: "section-2005-these-two-standards-were-designed-to-be-used-as-one-language-merging-the-base-verilog-language-into-the.txt"
source_pdf: "/Users/richarddje/Documents/github/SystemVerilog-LRM-IEEE-1800-2017.pdf"
---

# Section 2005: IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language

IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
39
Copyright © 2018 IEEE. All rights reserved.
—
Direct programming interface (DPI) for interoperation with the C programming language
—
VPI, API, and DPI header files
—
Concurrent assertion formal semantics
—
The formal syntax and semantics of standard delay format (SDF) constructs
—
Informative usage examples
NOTE—An earlier standard, IEEE Std 1800-2009, represented a merger of two previous standards:
IEEE Std 1364™-2005 and IEEE Std 1800-2005. In these previous standards, Verilog® was the base language and
defined a completely self-contained standard. SystemVerilog defined a number of significant extensions to Verilog, but
IEEE Std 1800-2005 was not a self-contained standard; IEEE Std 1800-2005 referred to, and relied on, IEEE Std 1364-
2005. These two standards were designed to be used as one language. Merging the base Verilog language into the
SystemVerilog standard enabled users to have all information regarding syntax and semantics in a single document.2, 3
### 1.4 Special terms

Throughout this standard, the following terms apply:
—
SystemVerilog 3.1a refers to the Accellera SystemVerilog 3.1a Language Reference Manual [B4],
a precursor to IEEE Std 1800-2005.4
—
Verilog refers to IEEE Std 1364-2005 for the Verilog hardware description language (HDL).
—
Language Reference Manual (LRM) refers to the document describing a Verilog or SystemVerilog
standard.
—
Tool refers to a software implementation that reads SystemVerilog source code, such as a logic
simulator.
NOTE—In IEEE Std 1800-2005, SystemVerilog referred to just the extensions to the IEEE 1364-2005 Verilog language
and did not include the Verilog base language.
### 1.5 Conventions used in this standard

This standard is organized into clauses, each of which focuses on a specific area of the language. There are
subclauses within each clause to discuss individual constructs and concepts. The discussion begins with an
introduction and an optional rationale for the construct or the concept, followed by syntax and semantic
descriptions, followed by examples and notes.
The terminology conventions used throughout this standard are as follows:
—
The word shall is used to indicate mandatory requirements strictly to be followed in order to
conform to the standard and from which no deviation is permitted (shall equals is required to).
—
The word should is used to indicate that among several possibilities one is recommended as
particularly suitable, without mentioning or excluding others; or that a certain course of action is
preferred but not necessarily required; or that (in the negative form) a certain course of action is
deprecated but not prohibited (should equals is recommended that).
—
The word may is used to indicate a course of action permissible within the limits of the standard
(may equals is permitted to).
—
The word can is used for statements of possibility and capability, whether material, physical, or
causal (can equals is able to).
2Notes in text, tables, and figures are given for information only and do not contain requirements needed to implement the standard.
3Verilog is a registered trademark of Cadence Design Systems, Inc.
4The numbers in brackets correspond to those of the bibliography in Annex Q.
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
40
Copyright © 2018 IEEE. All rights reserved.
### 1.6 Syntactic description

The main text uses the following conventions:
—
Italicized font when a term is being defined
—
Constant-width font for examples, file names, and references to constants, especially 0, 1, x, and
z values
—
Boldface constant-width font for SystemVerilog keywords, when referring to the actual
keyword
The formal syntax of SystemVerilog is described using Backus-Naur Form (BNF). The following
conventions are used:
—
Lowercase words, some containing embedded underscores, denote syntactic categories. For
example:
—
Boldface-red characters denote reserved keywords, operators, and punctuation marks as a required
part of the syntax. For example:
—
A vertical bar ( | ) that is not in boldface-red separates alternative items. For example:
—
Square brackets ( [ ] ) that are not in boldface-red enclose optional items. For example:
—
Braces ( { } ) that are not in boldface-red enclose a repeated item. The item may appear zero or more
times; the repetitions occur from left to right as with an equivalent left-recursive rule. Thus, the
following two rules are equivalent:
A qualified term in the syntax is a term such as array_identifier for which the “array” portion represents
some semantic intent and the “identifier” term indicates that the qualified term reduces to the “identifier”
term in the syntax. The syntax does not completely define the semantics of such qualified terms; for example
while an identifier that would qualify semantically as an array_identifier is created by a declaration, such
declaration forms are not explicitly described using array_identifier in the syntax.
### 1.7 Use of color in this standard

This standard uses a minimal amount of color to enhance readability. The coloring is not essential and does
not affect the accuracy of this standard when viewed in pure black and white. The places where color is used
are the following:
—
Cross references that are hyperlinked to other portions of this standard are shown in underlined-blue
text (hyperlinking works when this standard is viewed interactively as a PDF file).
—
Syntactic keywords and tokens in the formal language definitions are shown in boldface-red
text.
—
Some figures use a minimal amount of color to enhance readability.
module_declaration
module
=>
;
```ebnf
unary_operator ::=
 + | - | ! | ~ | & | ~& | | | ~| | ^ | ~^ | ^~
function_declaration ::= function [ lifetime ] function_body_declaration
list_of_param_assignments ::= param_assignment { , param_assignment }
list_of_param_assignments ::=
```

param_assignment
| list_of_param_assignments , param_assignment
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
41
Copyright © 2018 IEEE. All rights reserved.
### 1.8 Contents of this standard

A synopsis of the clauses and annexes is presented as a quick reference. All clauses and several of the
annexes are normative parts of this standard. Some annexes are included for informative purposes only.
Part One: Design and Verification Constructs
Clause 1 describes the contents of this standard and the conventions used in this standard.
Clause 2 lists references to other standards that are required in order to implement this standard.
Clause 3 introduces the major building blocks that make up a SystemVerilog design and verification
environment: modules, programs, interfaces, checkers, packages, and configurations. This clause also
discusses primitives, name spaces, the $unit compilation space, and the concept of simulation time.
Clause 4 describes the SystemVerilog simulation scheduling semantics.
Clause 5 describes the lexical tokens used in SystemVerilog source text and their conventions.
Clause 6 describes SystemVerilog data objects and types, including nets and variables, their declaration
syntax and usage rules, and charge strength of the values on nets. This clause also discusses strings and
string methods, enumerated types, user-defined types, constants, data scope and lifetime, and type
compatibility.
Clause 7 describes SystemVerilog compound data types: structures, unions, arrays, including packed and
unpacked arrays, dynamic arrays, associative arrays, and queues. This clause also describes various array
methods.
Clause 8 describes the object-oriented programming capabilities in SystemVerilog. Topics include defining
classes, interface classes, dynamically constructing objects, inheritance and subclasses, data hiding and
encapsulation, polymorphism, and parameterized classes.
Clause 9 describes the SystemVerilog procedural blocks: initial, always, always_comb, always_ff,
always_latch, and final. Sequential and parallel statement grouping, block names, statement labels, and
process control are also described.
Clause 10 describes continuous assignments, blocking and nonblocking procedural assignments, and
procedural continuous assignments.
Clause 11 describes the operators and operands that can be used in expressions.
Clause 12 describes SystemVerilog procedural programming statements, such as decision statements and
looping constructs.
Clause 13 describes tasks and functions, which are subroutines that can be called from more than one place
in a behavioral model.
Clause 14 defines clocking blocks, input and output skews, cycle delays, and default clocking.
Clause 15 describes interprocess communications using event types and event controls, and built-in
semaphore and mailbox classes.
Clause 16 describes immediate and concurrent assertions, properties, sequences, sequence operations,
multiclock sequences, and clock resolution.
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
42
Copyright © 2018 IEEE. All rights reserved.
Clause 17 describes checkers. Checkers allow the encapsulation of assertions and modeling code to create a
single verification entity.
Clause 18 describes generating random numbers, constraining random number generation, dynamically
changing constraints, seeding random number generators (RNGs), and randomized case statement
execution.
Clause 19 describes coverage groups, coverage points, cross coverage, coverage options, and coverage
methods.
Clause 20 describes most of the built-in system tasks and system functions.
Clause 21 describes additional system tasks and system functions that are specific to input/output (I/O)
operations.
Clause 22 describes various compiler directives, including a directive for controlling reserved keyword
compatibility between versions of previous Verilog and SystemVerilog standards.
Part Two: Hierarchy Constructs
Clause 23 describes how hierarchies are created in SystemVerilog using module instances and interface
instances, and port connection rules. This clause also discusses the $root top-level instances, nested
modules, extern modules, identifier search rules, how parameter values can be overridden, and binding
auxiliary code to scopes or instances.
Clause 24 describes the testbench program construct, the elimination of testbench race conditions, and
program control tasks.
Clause 25 describes interface syntax, interface ports, modports, interface subroutines, parameterized
interfaces, virtual interfaces, and accessing objects within interfaces.
Clause 26 describes user-defined packages and the std built-in package.
Clause 27 describes the generate construct and how generated constructs can be used to do conditional or
multiple instantiations of procedural code or hierarchy.
Clause 28 describes the gate- and switch-level primitives and logic strength modeling.
Clause 29 describes how a user-defined primitive (UDP) can be defined and how these primitives are
included in SystemVerilog models.
Clause 30 describes how to specify timing relationships between input and output ports of a module.
Clause 31 describes how timing checks are used in specify blocks to determine whether signals obey the
timing constraints.
Clause 32 describes the syntax and semantics of SDF constructs.
Clause 33 describes how to configure the contents of a design.
Clause 34 describes encryption and decryption of source text regions.
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
