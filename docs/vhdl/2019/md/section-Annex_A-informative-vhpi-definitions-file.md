---
title: "Section Annex.A: IEEE Standard for VHDL Language Reference Manual"
document: "VHDL Language Reference Manual"
standard: "IEEE 1076-2019"
domain: "VHDL"
section: "Annex.A"
source_txt: "section-Annex_A-informative-vhpi-definitions-file.txt"
source_pdf: "/Users/richarddje/Documents/github/VHDL-LRM-IEEE-1076-2019.pdf"
---

# Section Annex.A: IEEE Standard for VHDL Language Reference Manual

IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
525
Copyright © 2019 IEEE. All rights reserved.
Annex A
(informative)
VHPI definitions file
A.1 General
The VHPI definitions file, vhpi_def.c, is provided in the IEEE 1076 Open Source Repository. This C
source file contains a definition for the vhpi_is_printable function (see 23.23) and other definitions
described in this annex. The file is informative and is provided as a guide to implementers of VHPI tools.
A.2 VHPICharCodes
An array of strings of graphic characters corresponding to character codes.
Synopsis:
static const char* VHPICharCodes[256] = {
  "NUL",  "SOH",  "STX",  "ETX",  "EOT",  "ENQ",  "ACK",  "BEL",
  "BS",   "HT",   "LF",   "VT",   "FF",   "CR",   "SO" ,  "SI",
  "DLE",  "DC1",  "DC2",  "DC3",  "DC4",  "NAK",  "SYN",  "ETB",
  "CAN",  "EM",   "SUB",  "ESC",  "FSP",  "GSP",  "RSP",  "USP",
  " ",    "!",    "\"",   "#",    "$",    "%",    "&",    "'",
  "(",    ")",    "*",    "+",    ",",    "-",    ".",    "/",
  "0",    "1",    "2",    "3",    "4",    "5",    "6",    "7",
  "8",    "9",    ":",    ";",    "<",    "=",    ">",    "?",
  "@",    "A",    "B",    "C",    "D",    "E",    "F",    "G",
  "H",    "I",    "J",    "K",    "L",    "M",    "N",    "O",
  "P",    "Q",    "R",    "S",    "T",    "U",    "V",    "W",
  "X",    "Y",    "Z",    "[",    "\\",   "]",    "^",    "_",
  "`",    "a",    "b",    "c",    "d",    "e",    "f",    "g",
  "h",    "i",    "j",    "k",    "l",    "m",    "n",    "o",
  "p",    "q",    "r",    "s",    "t",    "u",    "v",    "w",
  "x",    "y",    "z",    "{",    "|",    "}",    "~",    "DEL",
  "C128", "C129", "C130", "C131", "C132", "C133", "C134", "C135",
  "C136", "C137", "C138", "C139", "C140", "C141", "C142", "C143",
  "C144", "C145", "C146", "C147", "C148", "C149", "C150", "C151",
  "C152", "C153", "C154", "C155", "C156", "C157", "C158", "C159",
  " ",    "¡",    "¢",    "£",    "¤",    "¥",    "¦",    "§",
  "¨",    "©",    "ª",    "«",    "¬",    "-",    "®",    "¯",
  "°",    "±",    "²",    "³",    "´",    "µ",    "¶",    "·",
  "¸",    "¹",    "º",    "»",    "¼",    "½",    "¾",    "¿",
  "À",    "Á",    "Â",    "Ã",    "Ä",    "Å",    "Æ",    "Ç",
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
526
Copyright © 2019 IEEE. All rights reserved.
  "È",    "É",    "Ê",    "Ë",    "Ì",    "Í",    "Î",    "Ï",
  "Ð",    "Ñ",    "Ò",    "Ó",    "Ô",    "Õ",    "Ö",    "×",
  "Ø",    "Ù",    "Ú",    "Û",    "Ü",    "Ý",    "Þ",    "ß",
  "à",    "á",    "â",    "ã",    "ä",    "å",    "æ",    "ç",
  "è",    "é",    "ê",    "ë",    "ì",    "í",    "î",    "ï",
  "ð",    "ñ",    "ò",    "ó",    "ô",    "õ",    "ö",    "÷",
  "ø",    "ù",    "ú",    "û",    "ü",    "ý",    "þ",    "ÿ" };
Description:
Each element of the array is a null-terminated string whose value is a printable representation of the
character code that is the index of the element. For character codes representing graphic characters (see
15.2), the string contains just the graphic character. For other character codes, the string contains a
representation in uppercase letters of the enumeration literal of type STD.STANDARD.CHARACTER
whose position number is the character code.
A.3 VHPI_GET_PRINTABLE_STRINGCODE
Gets a string of graphic characters corresponding to a character code.
Synopsis:
#define VHPI_GET_PRINTABLE_STRINGCODE( ch ) VHPICharCodes[unsigned char ch]
Description:
The macro takes as its argument a character code in the range 0 to 255 and substitutes an expression that
uses the character code to index an element of the VHPICharCodes array (see A.2). The type of the
expression is a pointer to a null-terminated string.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
