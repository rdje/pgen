---
title: "Section 1: IEEE Standard for VHDL Language Reference Manual"
document: "VHDL Language Reference Manual"
standard: "IEEE 1076-2019"
domain: "VHDL"
section: "1"
source_txt: "section-1-overview.txt"
source_pdf: "/Users/richarddje/Documents/github/VHDL-LRM-IEEE-1076-2019.pdf"
---

# Section 1: IEEE Standard for VHDL Language Reference Manual

IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
14
Copyright © 2019 IEEE. All rights reserved.
IEEE Standard for VHDL Language
Reference Manual
1. Overview
### 1.1 Scope

This standard defines the syntax and semantics of the VHSIC Hardware Description Language (VHDL).
The acronym VHSIC (Very High Speed Integrated Circuits) in the language’s name comes from the U.S.
government program that funded early work on the standard.
### 1.2 Purpose

VHDL is a formal notation intended for use in all phases of the creation of electronic systems. Since it is
both machine and human readable, it supports the design, development, verification, synthesis, and testing
of hardware designs; the communication of hardware design data; and the maintenance, modification, and
procurement of hardware. This document is intended for the implementers of tools supporting the language
and for advanced users of the language.
### 1.3 Structure and terminology of this standard

#### 1.3.1 General

This standard is organized into clauses, each of which focuses on some particular area of the language.
Within each clause, individual constructs or concepts are discussed in each subclause.
Each subclause describing a specific construct begins with an introductory paragraph. Next, the syntax of
the construct is described using one or more grammatical productions.
A set of paragraphs describing the meaning and restrictions of the construct in narrative form then follow.
In this document, the word shall is used to indicate a mandatory requirement. The word should is used to
indicate a recommendation. The word may is used to indicate a permissible action. The word can is used for
statements of possibility and capability.
Machine readable elements that are normatively referenced are available in the IEEE 1076 Open Source
Repository (see Clause 2).
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
15
Copyright © 2019 IEEE. All rights reserved.
Finally, each clause may end with examples, notes, and references to other pertinent clauses.
#### 1.3.2 Syntactic description

The form of a VHDL description is described by means of context-free syntax using a simple variant of the
Backus-Naur form (BNF); in particular:
a)
Lowercase words in roman font, some containing embedded underlines, are used to denote syntactic
categories, for example:
      formal_port_list
Whenever the name of a syntactic category is used, apart from the syntax rules themselves, spaces
take the place of underlines [thus, “formal port list” would appear in the narrative description when
referring to the syntactic category in item a)].
b)
Boldface words are used to denote reserved words, for example:
      array
Reserved words shall be used only in those places indicated by the syntax.
c)
```ebnf
A production consists of a left-hand side, the symbol “::=” (which is read as “can be replaced by”),
```

and a right-hand side. The left-hand side of a production is always a syntactic category; the
right-hand side is a replacement rule. The meaning of a production is a textual-replacement rule: any
occurrence of the left-hand side may be replaced by an instance of the right-hand side.
d)
A vertical bar (|) separates alternative items on the right-hand side of a production unless it occurs
immediately after an opening brace, in which case it stands for itself, as follows:
```ebnf
      letter_or_digit ::= letter | digit
      choices  ::=  choice { | choice }
```

In the first instance, an occurrence of “letter_or_digit” can be replaced by either “letter” or “digit.”
In the second case, “choices” can be replaced by a list of “choice,” separated by vertical bars [see
item f) for the meaning of braces].
e)
Square brackets [ ] enclose optional items on the right-hand side of a production; thus, the following
two productions are equivalent:
```ebnf
      return_statement ::= return [ expression ] ;
      return_statement ::= return ; | return expression ;
```

Note, however, that the initial and terminal square brackets in the right-hand side of the production
for signatures (see 4.5.3) are part of the syntax of signatures and do not indicate that the entire
right-hand side is optional.
f)
Braces { } enclose a repeated item or items on the right-hand side of a production. The items may
appear zero or more times; the repetitions occur from left to right as with an equivalent left-recursive
rule. Thus, the following two productions are equivalent:
```ebnf
      term ::= factor { multiplying_operator factor }
      term ::= factor | term multiplying_operator factor
```

g)
If the name of any syntactic category starts with an italicized part, it is equivalent to the category
name without the italicized part. The italicized part is intended to convey some semantic
information. For example, type_name and subtype_name are both syntactically equivalent to name
alone.
h)
The term simple_name is used for any occurrence of an identifier that already denotes some
declared entity.
#### 1.3.3 Semantic description

The meaning and restrictions of a particular construct are described with a set of narrative rules immediately
following the syntactic productions. In these rules, an italicized term indicates the definition of that term,
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
16
Copyright © 2019 IEEE. All rights reserved.
and identifiers appearing entirely in uppercase letters refer to definitions in package STANDARD (see
16.3).
The following terms are used in these semantic descriptions with the following meanings:
erroneous: The condition described represents an ill-formed description; however, implementations are not
required to detect and report this condition. Conditions are deemed erroneous only when it is impossible in
general to detect the condition during the processing of the language.
error: The condition described represents an ill-formed description; implementations are required to detect
the condition and report an error to the user of the tool.
illegal: A synonym for “error.”
legal: The condition described represents a well-formed description.
#### 1.3.4 Front matter, examples, notes, references, and annexes

Prior to this subclause are several pieces of introductory material; following Clause 24 are some annexes and
an index. The front matter, annexes (except Annex B), and index serve to orient and otherwise aid the user
of this standard, but are not part of the definition of VHDL; Annex B, however, is normative.
Some clauses of this standard contain examples, notes, and cross-references to other clauses of the standard;
these parts always appear at the end of a clause. Examples are meant to illustrate the possible forms of the
construct described. Illegal examples are italicized. Notes are meant to emphasize consequences of the rules
described in the clause or elsewhere. In order to distinguish notes from the other narrative portions of this
standard, notes are set as enumerated paragraphs in a font smaller than the rest of the text. Cross-references
are meant to guide the user to other relevant clauses of the standard. Examples, notes, and cross-references
are not part of the definition of the language.
#### 1.3.5 Incorporation of Property Specification Language

VHDL incorporates the simple subset of the Property Specification Language (PSL) as an embedded
language for formal specification of the behavior of a VHDL description. PSL is defined by
IEEE Std 1850™-2010.2 All PSL constructs that appear in a VHDL description shall conform to the VHDL
flavor of PSL. Within this standard, reference is made to syntactic rules of PSL. Each such reference has the
italicized prefix PSL_ and corresponds to the syntax rule in IEEE Std 1850-2010 with the same name but
without the prefix.
### 1.4 Word usage

The word shall indicates mandatory requirements strictly to be followed in order to conform to the standard
and from which no deviation is permitted (shall equals is required to).3, 4
The word should indicates that among several possibilities one is recommended as particularly suitable,
without mentioning or excluding others; or that a certain course of action is preferred but not necessarily
required (should equals is recommended that).
2Information on references can be found in Clause 2.
## 3 The use of the word must is deprecated and cannot be used when stating mandatory requirements, must is used only to describe

unavoidable situations.
## 4 The use of will is deprecated and cannot be used when stating mandatory requirements, will is only used in statements of fact.

Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
17
Copyright © 2019 IEEE. All rights reserved.
The word may is used to indicate a course of action permissible within the limits of the standard (may equals
is permitted to).
The word can is used for statements of possibility and capability, whether material, physical, or causal (can
equals is able to).
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
