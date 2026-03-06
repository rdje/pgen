---
title: "Section 15: IEEE Standard for VHDL Language Reference Manual"
document: "VHDL Language Reference Manual"
standard: "IEEE 1076-2019"
domain: "VHDL"
section: "15"
source_txt: "section-15-lexical-elements.txt"
source_pdf: "/Users/richarddje/Documents/github/VHDL-LRM-IEEE-1076-2019.pdf"
---

# Section 15: IEEE Standard for VHDL Language Reference Manual

IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
256
Copyright © 2019 IEEE. All rights reserved.
15. Lexical elements
### 15.1 General

The text of a description consists of one or more design files. The text of a design file is a sequence of lexical
elements, each composed of characters; the rules of composition are given in this clause.
### 15.2 Character set

The only characters allowed in the text of a VHDL description (except within comments—see 15.9, and
within text treated specially due to the effect of tool directives—see 15.11) are the graphic characters and
format effectors. Each graphic character corresponds to a unique code of the ISO eight-bit coded character
set (ISO/IEC 8859-1:1998) and is represented (visually) by a graphical symbol.
```ebnf
basic_graphic_character ::=
    upper_case_letter | digit | special_character | space_character
graphic_character ::=
    basic_graphic_character | lower_case_letter | other_special_character
basic_character ::=
    basic_graphic_character | format_effector
```

The basic character set is sufficient for writing any description, other than a PSL declaration, a PSL
directive, or a PSL verification unit. The characters included in each of the categories of basic graphic
characters are defined as follows:
—
Uppercase letters
A B C D E F G H I J K L M N O P Q R S T U V W X Y Z À Á Â Ã Ä Å Æ Ç È É Ê Ë Ì Í Î
Ï Ð Ñ Ò Ó Ô Õ Ö Ø Ù Ú Û Ü ÝÞ
—
Digits
## 0 1 2 3 4 5 6 7 8 9

—
Special characters
" # & ' () * + , - . / : ; < = > ? @ [ ] _ ` |
—
The space characters
SPACE11 NBSP12
Format effectors are the ISO/IEC (and ASCII) characters called horizontal tabulation, vertical tabulation,
carriage return, line feed, and form feed.
The characters included in each of the remaining categories of graphic characters are defined as follows:
—
Lowercase letters
11The visual representation of the space is the absence of a graphic symbol. It may be interpreted as a graphic character, a control
character, or both.
12The visual representation of the nonbreaking space is the absence of a graphic symbol. It is used when a line break is to be prevented
in the text as presented.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
257
Copyright © 2019 IEEE. All rights reserved.
a b c d e f g h i j k l m n o p q r s t u v w x y z ß à á â ã ä å æ ç è é ê ë ì í î ï ð ñ ò ó ô
õ ö ø ù ú û ü ý þ ÿ
—
Other special characters
! $ % \ ^ { }  ~ ¡ ¢ £ ¤ ¥ ¦ § ¨ © ª « ¬ ® ¯ ° ± ² ³ ´ µ ¶ · ¸ ¹ º » ¼ ½ ¾ ¿ × ÷ - (soft hyphen)
For each uppercase letter, there is a corresponding lowercase letter; and for each lowercase letter except ÿ
and ß, there is a corresponding uppercase letter. The pairs of corresponding uppercase and lowercase letters
are:
Within a PSL declaration, a PSL directive, or a PSL verification unit, certain of the other special characters
are allowed (see 15.3 and IEEE Std 1850-2010).
NOTE 1—The font design of graphical symbols (for example, whether they are in italic or bold typeface) is not part of
ISO/IEC 8859-1:1998.
NOTE 2—The meanings of the acronyms used in this subclause are as follows: ASCII stands for American Standard
Code for Information Interchange, ISO stands for International Organization for Standardization.
NOTE 3—There are no uppercase equivalents for the characters ß and ÿ.
NOTE 4—The following names are used when referring to special characters:
A
a
B
b
C
c
D
d
E
e
F
f
G
g
H
h
I
i
J
i
K
k
L
l
M
m
N
n
O
o
P
p
Q
q
R
r
S
s
T
t
U
u
V
v
W
w
X
x
Y
y
Z
z
À
à
Á
á
Â
â
Ã
ã
Ä
ä
Å
å
Æ
æ
Ç
ç
È
è
É
é
Ê
ê
Ë
ë
Ì
ì
Í
í
Î
î
Ï
ï
Ð
ð
Ñ
ñ
Ò
ò
Ó
ó
Ô
ô
Õ
õ
Ö
ö
Ø
ø
Ù
ù
Ú
ú
Û
û
Ü
ü
Ý
ý
Þ
þ
Character
Name
"
Quotation mark
£
Pound sign
#
Number sign
¤
Currency sign
&
Ampersand
¥
Yen sign
'
Apostrophe, tick
¦
Broken bar
(
Left parenthesis
§
Paragraph sign, clause sign
)
Right parenthesis
¨
Diaeresis
*
Asterisk, multiply
©
Copyright sign
+
Plus sign
ª
Feminine ordinal indicator
,
Comma
«
Left angle quotation mark
-
Hyphen, minus sign
¬
Not sign
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
258
Copyright © 2019 IEEE. All rights reserved.
### 15.3 Lexical elements, separators, and delimiters

The text of each design unit, apart from text treated specially due to the effect of tool directives (see 15.11),
is a sequence of separate lexical elements. Each lexical element is either a delimiter, an identifier (which
may be a reserved word), an abstract literal, a character literal, a string literal, a bit string literal, a comment,
a lexical element defined for a tool directive, or a lexical element defined in IEEE Std 1850-2010 for a PSL
declaration, a PSL directive, or a PSL verification unit.
In some cases an explicit separator is required to separate adjacent lexical elements (namely when, without
separation, interpretation as a single lexical element is possible). A separator is either a space character
(SPACE or NBSP), a format effector, or the end of a line. A space character (SPACE or NBSP) is a
.
Dot, point, period, full stop
-
Soft hyphena
/
Slash, divide, solidus
®
Registered trade mark sign
:
Colon
¯
Macron
;
Semicolon
°
Ring above, degree sign
<
Less-than sign
±
Plus-minus sign
=
Equals sign
²
Superscript two
>
Greater-than sign
³
Superscript three
_
Underline, low line
´
Acute accent
|
Vertical line, vertical bar
µ
Micro sign
!
Exclamation mark
¶
Pilcrow sign
$
Dollar sign
·
Middle dot
%
Percent sign
¸
Cedilla
?
Question mark
¹
Superscript one
@
Commercial at
º
Masculine ordinal indicator
[
Left square bracket
»
Right angle quotation mark
 \
Backslash, reverse solidus
¼
Vulgar fraction one quarter
]
Right square bracket
½
Vulgar fraction one half
^
Circumflex accent
¾
Vulgar fraction three quarters
`
Grave accent
¿
Inverted question mark
{
Left curly bracket
×
Multiplication sign
}
Right curly bracket
÷
Division sign
~
Tilde
¡
Inverted exclamation mark
¢
Cent sign
aThe soft hyphen is a graphic character that is represented by a graphic symbol identical with, or similar
to, that representing a hyphen, for use when a line break has been established within a word.
Character
Name
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
259
Copyright © 2019 IEEE. All rights reserved.
separator except within an extended identifier, a comment, a string literal, a space character literal, or where
defined to be part of a lexical element in a tool directive.
The end of a line is always a separator. The language does not define what causes the end of a line. However
if, for a given implementation, the end of a line is signified by one or more characters, then these characters
shall be format effectors other than horizontal tabulation. In any case, a sequence of one or more format
effectors other than horizontal tabulation shall cause at least one end-of-line.
One or more separators are allowed between any two adjacent lexical elements, before the first of each
design unit, or after the last lexical element of a design file. At least one separator is required between an
identifier or an abstract literal and an adjacent identifier or abstract literal.
A delimiter is either one of the following special characters (in the basic character set):
&  '  (  )  *  +  ,  - .  /  :  ;  <  =  >  `  | [ ]  ? @
or one of the following compound delimiters, each composed of two or more adjacent special characters:
=>  **  :=  /=  >=  <=  <>  ??  ?=  ?/=  ?<  ?<=  ?>  ?>=  <<  >>
Each of the special characters listed for single character delimiters is a single delimiter except if this
character is used as a character of a compound delimiter or as a character of an extended identifier,
comment, string literal, character literal, or abstract literal.
The remaining forms of lexical elements are described in subclauses of this clause.
NOTE 1—Each lexical element will fit on one line, since the end of a line is a separator. The quotation mark, number
sign, and underline characters, likewise two adjacent hyphens, are not delimiters, but may form part of other lexical
elements.
NOTE 2—The following names are used when referring to compound delimiters:
Delimiter
Name
=>
Arrow
**
Double star, exponentiate
:=
Variable assignment
/=
Inequality (pronounced “not equal”)
>=
Greater than or equal
<=
Less than or equal; signal assignment
<>
Box
??
Condition conversion
?=
Matching equality
?/=
Matching inequality
?<
Matching less than
?<=
Matching less than or equal
?>
Matching greater than
?>=
Matching greater than or equal
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
260
Copyright © 2019 IEEE. All rights reserved.
NOTE 3—PSL macros and preprocessing directives can only be defined and used within PSL verification units. They
cannot appear in PSL declarations or PSL directives embedded in other VHDL code, since they do not occur as part of
the syntax of PSL declarations or PSL directives.
### 15.4 Identifiers

#### 15.4.1 General

Identifiers are used as names and also as reserved words.
```ebnf
identifier ::=  basic_identifier | extended_identifier
```

#### 15.4.2 Basic identifiers

A basic identifier consists only of letters, digits, and underlines.
```ebnf
basic_identifier ::=
```

letter  { [ underline ] letter_or_digit }
```ebnf
letter_or_digit ::=  letter | digit
letter ::=  upper_case_letter | lower_case_letter
```

All characters of a basic identifier are significant, including any underline character inserted between a letter
or digit and an adjacent letter or digit. Basic identifiers differing only in the use of corresponding uppercase
and lowercase letters are considered the same.
Examples:
COUNT    X     c_out        FFT                Decoder
VHSIC    X1    PageCount    STORE_NEXT_ITEM
NOTE—No space (SPACE or NBSP) is allowed within a basic identifier, since a space is a separator.
#### 15.4.3 Extended identifiers

Extended identifiers may contain any graphic character.
```ebnf
extended_identifier ::=
```

\ graphic_character { graphic_character } \
If a backslash is to be used as one of the graphic characters of an extended identifier, it shall be doubled. All
characters of an extended identifier are significant (a doubled backslash counting as one character).
Extended identifiers differing only in the use of corresponding uppercase and lowercase letters are distinct.
Moreover, every extended identifier is distinct from any basic identifier.
<<
Double less than
>>
Double greater than
Delimiter
Name
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
261
Copyright © 2019 IEEE. All rights reserved.
Examples:
\BUS\
\bus\
--  Two different identifiers,

--  neither of which is

--  the reserved word bus.
\a\\b\
--  An identifier containing

--  three characters.
VHDL
\VHDL\
\vhdl\      --  Three distinct identifiers.
### 15.5 Abstract literals

#### 15.5.1 General

There are two classes of abstract literals: real literals and integer literals. A real literal is an abstract literal
that includes a point; an integer literal is an abstract literal without a point. Real literals are the literals of the
type universal_real. Integer literals are the literals of the type universal_integer.
```ebnf
abstract_literal ::=  decimal_literal | based_literal
```

#### 15.5.2 Decimal literals

A decimal literal is an abstract literal expressed in the conventional decimal notation (that is, the base is
implicitly ten).
```ebnf
decimal_literal ::=  integer [ . integer ] [ exponent ]
integer ::=  digit  { [ underline ] digit }
exponent ::=  E [ + ] integer | E – integer
```

An underline character inserted between adjacent digits of a decimal literal does not affect the value of this
abstract literal. The letter E of the exponent, if any, can be written either in lowercase or in uppercase, with
the same meaning.
An exponent indicates the power of 10 by which the value of the decimal literal without the exponent is to be
multiplied to obtain the value of the decimal literal with the exponent. An exponent for an integer literal
shall not have a minus sign.
Examples:
12
0
1E6
123_456
--  Integer literals.
12.0
0.0
### 0.456 3.14159_26 --  Real literals.

1.34E-12
1.0E+6    6.023E+24
--  Real literals

--  with exponents.
NOTE—Leading zeros are allowed. No space (SPACE or NBSP) is allowed in an abstract literal, not even between
constituents of the exponent, since a space is a separator. A zero exponent is allowed for an integer literal.
#### 15.5.3 Based literals

A based literal is an abstract literal expressed in a form that specifies the base explicitly. The base shall be at
least two and at most sixteen.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
262
Copyright © 2019 IEEE. All rights reserved.
```ebnf
based_literal ::=
```

base # based_integer [ . based_integer ] # [ exponent ]
```ebnf
base ::=  integer
based_integer ::=
```

extended_digit { [ underline ] extended_digit }
```ebnf
extended_digit ::=  digit | letter
```

An underline character inserted between adjacent digits of a based literal does not affect the value of this
abstract literal. The base and the exponent, if any, are in decimal notation. The only letters allowed as
extended digits are the letters A through F for the digits 10 through 15. A letter in a based literal (either an
extended digit or the letter E of an exponent) can be written either in lowercase or in uppercase, with the
same meaning.
The conventional meaning of based notation is assumed; in particular the value of each extended digit of a
based literal shall be less than the base. An exponent indicates the power of the base by which the value of
the based literal without the exponent is to be multiplied to obtain the value of the based literal with the
exponent. An exponent for a based integer literal shall not have a minus sign.
Examples:
-- Integer literals of value 255:
2#1111_1111#
16#FF#
016#0FF#
--  Integer literals of value 224:
16#E#E1
2#1110_0000#
--  Real literals of value 4095.0:
16#F.FF#E+2
2#1.1111_1111_111#E11
### 15.6 Character literals

A character literal is formed by enclosing one of the 191 graphic characters (including the space and
nonbreaking space characters) between two apostrophe characters. A character literal has a value that
belongs to a character type.
```ebnf
character_literal ::= ' graphic_character '
```

Examples:
'A'
'*'
'''
' '
### 15.7 String literals

A string literal is formed by a sequence of graphic characters (possibly none) enclosed between two
quotation marks used as string brackets.
```ebnf
string_literal ::=  " { graphic_character } "
```

Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
263
Copyright © 2019 IEEE. All rights reserved.
A string literal has a value that is a sequence of character values corresponding to the graphic characters of
the string literal apart from the quotation mark itself. If a quotation mark value is to be represented in the
sequence of character values, then a pair of adjacent quotation marks shall be written at the corresponding
place within the string literal. (This means that a string literal that includes two adjacent quotation marks is
never interpreted as two adjacent string literals.)
The length of a string literal is the number of character values in the sequence represented. (Each doubled
quotation mark is counted as a single character.)
Examples:
"Setup time is too short"
--  An error message.
""
--  An empty string literal.
" "
"A"   """"
--  Three string literals of length 1.
"Characters such as $, %, and } are allowed in string literals."
NOTE—A string literal fits on one line, since it is a lexical element (see 15.3). Longer sequences of graphic character
values can be obtained by concatenation of string literals. The concatenation operation may also be used to obtain string
literals containing nongraphic character values. The predefined type CHARACTER in package STANDARD specifies
the enumeration literals denoting predefined both graphic and nongraphic characters. Examples of such uses of
concatenation are as follows:
"FIRST PART OF A SEQUENCE OF CHARACTERS " &
"THAT CONTINUES ON THE NEXT LINE"
"Sequence that includes the" & ACK & "control character"
### 15.8 Bit string literals

A bit string literal is formed by a sequence of characters (possibly none) enclosed between two quotation
marks used as bit string brackets, preceded by a base specifier. The bit string literal may also be preceded by
an integer specifying the length of the value represented by the bit string literal.
```ebnf
bit_string_literal ::= [ integer ] base_specifier " [ bit_value ] "
bit_value ::=  graphic_character { [ underline ] graphic_character }
base_specifier ::=  B | O | X | UB | UO | UX | SB | SO | SX | D
```

A graphic character in a bit string literal shall not be an underline character. An underline character inserted
between adjacent graphic characters of a bit string literal does not affect the value of this literal.
If the base specifier is B, UB, or SB, the digits 0 and 1 in the bit value are interpreted as extended digits, and
all other graphic characters are not interpreted as extended digits. If the base specifier is O, UO, or SO, the
digits 0 through 7 in the bit value are interpreted as extended digits, and all other graphic characters are not
interpreted as extended digits. If the base specifier is X, UX, or SX, all digits together with the letters A
through F in the bit value are interpreted as extended digits. If the base specifier is D, all of the graphic
characters in the bit value (not counting underline characters) shall be digits. An extended digit and the base
specifier in a bit string literal can be written either in lowercase or in uppercase, with the same meaning.
A bit string literal has a value that is a string literal. The string literal is formed from the bit value by first
obtaining a simplified bit value, consisting of the bit value with underline characters removed, and then
obtaining an expanded bit value. Finally, the string literal value is obtained by adjusting the expanded bit
value, if required.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
264
Copyright © 2019 IEEE. All rights reserved.
If the base specifier is B, UB, or SB, the expanded bit value is the simplified bit value itself. If the base
specifier is O, UO, or SO (respectively X, UX, or SX), the expanded bit value is the string obtained by
replacing each character of the simplified bit value by a sequence of three (respectively four) characters. For
a character in the simplified bit value that is interpreted as an extended digit, the replacement sequence is as
follows:
For a character in the simplified value that is not interpreted as an extended digit, each character in the
replacement sequence is the same as the character replaced.
If the base specifier is D, the simplified bit value is interpreted as a decimal integer. The expanded bit value
is a string of 0 and 1 digits that is the binary representation of the decimal integer. The number of characters
in the expanded bit value is given by the expression [log2(n)] + 1 , where n is the value of the decimal integer
and if n is greater than 0. If n is equal to 0, the expanded bit value is "0".
The length of a bit string literal is the length of its string literal value. If a bit string literal includes the
integer immediately preceding the base specifier, the length of the bit string literal is the value of the integer.
Otherwise, the length is the number of characters in the expanded bit value.
The string literal value is obtained by adjusting the expanded bit value to the length of the bit string literal, as
follows:
—
If the length is equal to the number of characters in the expanded bit value, the string literal value is
the expanded bit value itself.
—
If the length is greater than the number of characters in the expanded bit value and the base specifier
is B, UB, O, UO, X, UX, or D, the bit string value is obtained by concatenating a string of 0 digits to
Extended digit
Replacement when the base specifier is
O, UO, or SO
Replacement when the base specifier is
X, UX, or SX
0
000
0000
1
001
0001
2
010
0010
3
011
0011
4
100
0100
5
101
0101
6
110
0110
7
111
0111
8
1000
9
1001
A
1010
B
1011
C
1100
D
1101
E
1110
F
1111
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
265
Copyright © 2019 IEEE. All rights reserved.
the left of the expanded bit value. The number of 0 digits in the string is such that the number of
characters in the result of the concatenation is the length of the bit string literal.
—
If the length is greater than the number of characters in the expanded bit value and the base specifier
is SB, SO, or SX, the bit string value is obtained by concatenating to the left of the expanded bit
value a string, each of whose characters is the leftmost character of the expanded bit value. The num-
ber of characters in the string is such that the number of characters in the result of the concatenation
is the length of the bit string literal. It is an error if the bit string literal is an empty string such as
8sx"".
—
If the length is less than the number of characters in the expanded bit value and the base specifier is
B, UB, O, UO, X, UX, or D, the bit string value is obtained by deleting sufficient characters from the
left of the expanded bit value to yield a string whose length is the length of the bit string literal. It is
an error if any of the characters so deleted is other than the digit 0.
—
If the length is less than the number of characters in the expanded bit value and the base specifier is
SB, SO, or SX, the bit string value is obtained by deleting sufficient characters from the left of the
expanded bit value to yield a string whose length is the length of the bit string literal. It is an error if
any of the characters so deleted differs from the leftmost remaining character.
Example:
B"1111_1111_1111" -- Equivalent to the string literal "111111111111".
X"FFF"
-- Equivalent to B"1111_1111_1111".
O"777"
-- Equivalent to B"111_111_111".
X"777"
-- Equivalent to B"0111_0111_0111".
B"XXXX_01LH"
-- Equivalent to the string literal "XXXX01LH"
UO"27"
-- Equivalent to B"010_111"
UO"2C"
-- Equivalent to B"011_CCC"
SX"3W"
-- Equivalent to B"0011_WWWW"
D"35"
-- Equivalent to B"100011"
12UB"X1"
-- Equivalent to B"0000_0000_00X1"
12SB"X1"
-- Equivalent to B"XXXX_XXXX_XXX1"
12UX"F-"
-- Equivalent to B"0000_1111_----"
12SX"F-"
-- Equivalent to B"1111_1111_----"
12D"13"
-- Equivalent to B"0000_0000_1101"
12UX"000WWW"
-- Equivalent to B"WWWW_WWWW_WWWW"
12SX"FFFC00"
-- Equivalent to B"1100_0000_0000"
12SX"XXXX00"
-- Equivalent to B"XXXX_0000_0000"
8D"511"
-- Error
8UO"477"
-- Error
8SX"0FF"
-- Error
8SX"FXX"
-- Error
constant c1: STRING := B"1111_1111_1111";
constant c2: BIT_VECTOR := X"FFF";
type MVL is ('X', '0', '1', 'Z');
type MVL_VECTOR is array (NATURAL range <>) of MVL;
constant c3: MVL_VECTOR := O"777";
assert c1'LENGTH = 12 and c2'LENGTH = 12 and c3 = "111111111";
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
266
Copyright © 2019 IEEE. All rights reserved.
### 15.9 Comments

A comment is either a single-line comment or a delimited comment. A single-line comment starts with two
adjacent hyphens and extends up to the end of the line. A delimited comment starts with a solidus (slash)
character immediately followed by an asterisk character and extends up to the first subsequent occurrence of
an asterisk character immediately followed by a solidus character.
An occurrence of two adjacent hyphens within a delimited comment is not interpreted as the start of a
single-line comment. Similarly, an occurrence of a solidus character immediately followed by an asterisk
character within a single-line comment is not interpreted as the start of a delimited comment. Moreover, an
occurrence of a solidus character immediately followed by an asterisk character within a delimited comment
is not interpreted as the start of a nested delimited comment.
A single-line comment can appear on any line of a VHDL description and may contain any character except
the format effectors vertical tab, carriage return, line feed, and form feed. A delimited comment can start on
any line of a VHDL description and may finish on the same line or any subsequent line.
The presence or absence of comments has no influence on whether a description is legal or illegal.
Furthermore, comments do not influence the execution of a simulation module; their sole purpose is to
enlighten the human reader.
Examples:
-- The last sentence above echoes the Algol 68 report.
end; -- Processing of LINE is complete.
----------- The first two hyphens start the comment.
/* A long comment may be written
    on several consecutive lines */
x := 1; /* Comments /* do not nest */
NOTE 1—Horizontal tabulation can be used in comments, after the starting characters, and is equivalent to one or more
spaces (SPACE characters) (see 15.3).
NOTE 2—Comments may contain characters that, according to 15.2, are non-printing characters. Implementations may
interpret the characters of a comment as members of ISO/IEC 8859-1:1998, or of any other character set; for example,
an implementation may interpret multiple consecutive characters within a comment as single characters of a multi-byte
character set.
### 15.10 Reserved words

The following identifiers are called reserved words and are reserved for significance in the language. For
readability of this standard, the reserved words appear in lowercase boldface.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
267
Copyright © 2019 IEEE. All rights reserved.
A reserved word shall not be used as an explicitly declared identifier.
With a PSL declaration, a PSL directive, or a PSL verification unit, PSL keywords are reserved words (see
IEEE Std 1850-2010). A PSL keyword shall not be used as an identifier to declare a PSL declaration or a
abs
access
after
alias
all
and
architecture
array
assert
assume
attribute
begin
block
body
buffer
bus
case
component
configuration
constant
context
cover
default
disconnect
downto
else
elsif
end
entity
exit
fairness
file
for
force
function
generate
generic
group
guarded
if
impure
in
inertial
inout
is
label
library
linkage
literal
loop
map
mod
nand
new
next
nor
not
null
of
on
open
or
others
out
package
parameter
port
postponed
procedure
process
property
protected
private
pure
range
record
register
reject
release
rem
report
restrict
return
rol
ror
select
sequence
severity
signal
shared
sla
sll
sra
srl
strong
subtype
then
to
transport
type
unaffected
units
until
use
variable
view
vpkg
vmode
vprop
vunit
wait
when
while
with
xnor
xor
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
268
Copyright © 2019 IEEE. All rights reserved.
PSL verification unit. A PSL keyword that is a legal VHDL identifier may be used as an explicitly declared
identifier other than to declare a PSL declaration or a PSL verification unit, but such a declaration is hidden
within a PSL declaration, a PSL directive, or a PSL verification unit (see 12.3).
NOTE 1—Reserved words differing only in the use of corresponding uppercase and lowercase letters are considered as
the same (see 15.4.2). The reserved words range and subtype are also used as the names of predefined attributes.
NOTE 2—An extended identifier whose sequence of characters inside the leading and trailing backslashes is identical to
a reserved word is not a reserved word. For example, \next\ is a legal (extended) identifier and is not the reserved word
next.
NOTE 3—The following reserved words are PSL keywords, that is, reserved identifiers in PSL:
Their use in PSL is defined in IEEE Std 1850-2010. Other PSL keywords, reserved only within PSL declarations, PSL
directives, and PSL verification units, are defined in IEEE Std 1850-2010.
### 15.11 Tool directives

A tool directive directs a tool to analyze, elaborate, execute, or otherwise process a description in a specified
manner. A tool directive starts with a grave accent character and extends up to the end of the line.
```ebnf
tool_directive ::= ` identifier { graphic_character }
```

The identifier determines the form of processing to be performed by the tool. Apart from the standard tool
directives (see Clause 24), the requirements, if any, on the location of a tool directive and on the graphic
characters are implementation defined, as is the effect of the tool directive.
assert
assume
cover
default
fairness
property
restrict
sequence
strong
vmode
vpkg
vprop
vunit
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
