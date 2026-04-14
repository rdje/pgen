# Tcl Language Syntax - Clean Markdown Version

## NAME
**Tcl** - Summary of Tcl language syntax.

## DESCRIPTION
The following rules define the syntax and semantics of the Tcl language.

---

## 1. Scripts and Commands
A Tcl script is a string containing one or more commands.

- Commands are separated by **semicolons** or **newlines**.
- Close brackets (`]`) terminate command substitution unless quoted.

---

## 2. Command Evaluation
A command is evaluated in two steps:

1. The interpreter splits the command into **words** and performs **substitutions**.
2. The first word identifies the command; remaining words are arguments.

Each command may interpret its words differently (as integers, lists, variable names, etc.).

---

## 3. Word Separation
Words are separated by **whitespace**, except:

- Newlines also act as command separators.
- Quoting rules override whitespace splitting.

---

## 4. Double-Quoted Words
If a word begins with `"`, it ends at the next `"`.
Inside double quotes:

- Semicolons, brackets, and whitespace are treated as literal characters.
- **Command**, **variable**, and **backslash** substitutions *do occur*.
- The quotes themselves are not part of the word.

---

## 5. Braced Words
If a word begins with `{`, it ends at the matching `}`.

- Braces **nest**.
- Backslash-newline substitution occurs.
- **No substitutions** occur inside braces (except the newline case).
- The braces are not included in the resulting word.

---

## 6. Command Substitution
If a word contains `[ ... ]`, Tcl performs **command substitution**:

- The interpreter recursively evaluates the script inside the brackets.
- The result of the last command replaces the entire bracketed expression.
- Substitution does **not** occur inside braced words.

---

## 7. Variable Substitution
If a word contains `$`, Tcl performs **variable substitution**.

Forms:

- `$name` - scalar variable
- `$name(index)` - array element
- `${name}` - scalar with arbitrary characters

Notes:

- Substitutions occur unless inside braces.
- Any number of substitutions may appear in a word.

---

## 8. Backslash Substitution
A backslash triggers special processing.

### Special sequences
| Sequence | Meaning |
|---------|---------|
| `\a` | Bell (0x07) |
| `\b` | Backspace (0x08) |
| `\f` | Form feed (0x0C) |
| `\n` | Newline (0x0A) |
| `\r` | Carriage return (0x0D) |
| `\t` | Tab (0x09) |
| `\v` | Vertical tab (0x0B) |
| `\\` | Literal backslash |
| `\ooo` | Octal character code |
| `\xhh` | Hex character code |

### Special case: backslash-newline
A backslash followed by a newline and whitespace is replaced by **a single space**.

This replacement happens **before** parsing, even inside braces.

---

## 9. Comments
A `#` begins a comment **only** when it appears where Tcl expects the first character of a command.

The comment continues to the next newline.

---

## 10. Single-Pass Substitution
Each character is processed **exactly once**.

- Variable substitution inserts the value verbatim.
- Command substitution evaluates the nested script fully; no further substitutions occur afterward.

---

## 11. Substitutions Do Not Change Word Boundaries
Substitutions never split or merge words.

Example:
If a variable expands to a string containing spaces, it still forms **one word**.

---

## COPYRIGHT
- (c) 1993 The Regents of the University of California
- (c) 1994-1996 Sun Microsystems, Inc.
- (c) 1995-1997 Roger E. Critchlow Jr.
