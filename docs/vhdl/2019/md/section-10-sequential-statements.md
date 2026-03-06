---
title: "Section 10: IEEE Standard for VHDL Language Reference Manual"
document: "VHDL Language Reference Manual"
standard: "IEEE 1076-2019"
domain: "VHDL"
section: "10"
source_txt: "section-10-sequential-statements.txt"
source_pdf: "/Users/richarddje/Documents/github/VHDL-LRM-IEEE-1076-2019.pdf"
---

# Section 10: IEEE Standard for VHDL Language Reference Manual

IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
176
Copyright © 2019 IEEE. All rights reserved.
10. Sequential statements
### 10.1 General

The various forms of sequential statements are described in this clause. Sequential statements are used to
define algorithms for the execution of a subprogram or process; they execute in the order in which they
appear.
```ebnf
sequence_of_statements ::=
    { sequential_statement }
sequential_statement ::=
        wait_statement
    |   assertion_statement
    |   report_statement
    |   signal_assignment_statement
    |   variable_assignment_statement
    |   procedure_call_statement
    |   if_statement
    |   case_statement
    |   loop_statement
    |   next_statement
    |   exit_statement
    |   return_statement
    |   null_statement
    |   sequential_block_statement
```

All sequential statements may be labeled. Such labels are implicitly declared at the beginning of the
declarative part of the innermost enclosing process statement or subprogram body.
### 10.2 Wait statement

The wait statement causes the suspension of a process statement or a procedure.
```ebnf
wait_statement ::=
    [ label : ] wait [ sensitivity_clause ] [ condition_clause ] [ timeout_clause ] ;
sensitivity_clause ::=  on sensitivity_list
sensitivity_list ::=  signal_name { , signal_name }
condition_clause ::=  until condition
timeout_clause ::=  for time_expression
```

The sensitivity clause defines the sensitivity set of the wait statement, which is the set of signals to which the
wait statement is sensitive. Each signal name in the sensitivity list identifies a given signal as a member of
the sensitivity set. Each signal name in the sensitivity list shall be a static signal name, and each name shall
denote a signal for which reading is permitted. If no sensitivity clause appears, the sensitivity set is
constructed according to the following (recursive) rule:
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
177
Copyright © 2019 IEEE. All rights reserved.
The sensitivity set is initially empty. For each primary in the condition of the condition clause, if the primary
is
—
A simple name that denotes a signal, add the longest static prefix of the name to the sensitivity set.
—
An expanded name that denotes a signal, add the longest static prefix of the name to the sensitivity
set.
—
A selected name whose prefix denotes a signal, add the longest static prefix of the name to the
sensitivity set.
—
An indexed name whose prefix denotes a signal, add the longest static prefix of the name to the
sensitivity set and apply this rule to all expressions in the indexed name.
—
A slice name whose prefix denotes a signal, add the longest static prefix of the name to the
sensitivity set and apply this rule to any expressions appearing in the discrete range of the slice
name.
—
An attribute name, if the designator denotes a signal attribute, add the longest static prefix of the
name of the implicit signal denoted by the attribute name to the sensitivity set; otherwise, apply this
rule to the prefix of the attribute name.
—
An aggregate, apply this rule to every expression appearing after the choices and the =>, if any, in
every element association.
—
A function call, apply this rule to every actual designator in every parameter association.
—
An actual designator of open in a parameter association, do not add to the sensitivity set.
—
A qualified expression, apply this rule to the expression or aggregate qualified by the type mark, as
appropriate.
—
A type conversion, apply this rule to the expression type converted by the type mark.
—
A parenthesized expression, apply this rule to the expression enclosed within the parentheses.
—
Otherwise, do not add to the sensitivity set.
This rule is also used to construct the sensitivity sets of the wait statements in the equivalent process
statements for concurrent procedure call statements (11.4), concurrent assertion statements (11.5), and
concurrent signal assignment statements (11.7). Furthermore, this rule is used to construct the sensitivity list
of an implicit wait statement in a process statement whose process sensitivity list is the reserved word all
(11.3).
If a signal name that denotes a signal of a composite type appears in a sensitivity list, the effect is as if the
name of each scalar subelement of that signal appears in the list.
The condition clause specifies a condition that shall be met for the process to continue execution. If no
condition clause appears, the condition clause until TRUE is assumed.
The timeout clause specifies the maximum amount of time the process will remain suspended at this wait
statement. If no timeout clause appears, the timeout clause for (STD.STANDARD.TIME'HIGH –
STD.STANDARD.NOW) is assumed. It is an error if the time expression in the timeout clause evaluates to
a negative value.
The execution of a wait statement causes the time expression to be evaluated to determine the timeout
interval. It also causes the execution of the corresponding process statement to be suspended, where the
corresponding process statement is the one that either contains the wait statement or is the parent (see 4.3) of
the procedure that contains the wait statement. The suspended process will resume, at the latest, immediately
after the timeout interval has expired.
The suspended process also resumes as a result of an event occurring on any signal in the sensitivity set of
the wait statement. If such an event occurs, the condition in the condition clause is evaluated. If the value of
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
178
Copyright © 2019 IEEE. All rights reserved.
the condition is FALSE, the process suspends again. Such repeated suspension does not involve the
recalculation of the timeout interval.
It is an error if a wait statement appears in a function subprogram or in a procedure that has a parent that is a
function subprogram. Furthermore, it is an error if a wait statement appears in an explicit process statement
that includes a sensitivity list or in a procedure that has a parent that is such a process statement. Finally, it is
an error if a wait statement appears within any subprogram whose body is declared within a protected type
body, or within any subprogram that has a parent whose body is declared within a protected type body.
Example:
type Arr is array (1 to 5) of BOOLEAN;
function F (P: BOOLEAN) return BOOLEAN;
signal S: Arr;
signal l, r: INTEGER range 1 to 5;
--  The following two wait statements have the same meaning:
wait until F(S(3)) and (S(l) or S(r));
wait on S(3), S, l, r until F(S(3)) and (S(l) or S(r));
NOTE 1—The wait statement wait until Clk = '1'; has semantics identical to
loop
wait on Clk;
exit when Clk = '1';
end loop;
because of the rules for the construction of the default sensitivity clause. These same rules imply that wait until TRUE;
has semantics identical to wait;
NOTE 2—The conditions that cause a wait statement to resume execution of its enclosing process may no longer hold at
the time the process resumes execution if the enclosing process is a postponed process.
NOTE 3—The rule for the construction of the default sensitivity set implies that if a function call appears in a condition
clause and the called function is an impure function, then any signals that are accessed by the function but that are not
passed through the association list of the call are not added to the default sensitivity set for the condition by virtue of the
appearance of the function call in the condition.
### 10.3 Assertion statement

An assertion statement checks that a specified condition is true and reports an error if it is not.
```ebnf
assertion_statement ::=  [ label : ] assertion ;
assertion ::=
    assert condition
        [ report expression ]
        [ severity expression ]
```

If the report clause is present, it shall include an expression of predefined type STRING that specifies a
message to be reported. If the severity clause is present, it shall specify an expression of predefined type
SEVERITY_LEVEL that specifies the severity level of the assertion.
The report clause specifies a message string to be included in error messages generated by the assertion. In
the absence of a report clause for a given assertion, the string “Assertion violation.” is the default value for
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
179
Copyright © 2019 IEEE. All rights reserved.
the message string. The severity clause specifies a severity level associated with the assertion. In the
absence of a severity clause for a given assertion, the default value of the severity level is ERROR.
Execution of an assertion statement consists of evaluation of the Boolean expression specifying the
condition. If the expression results in the value FALSE, then an assertion violation is said to occur. When an
assertion violation occurs, the report and severity clause expressions of the corresponding assertion, if
present, are evaluated. The specified message string and severity level (or the corresponding default values,
if not specified) are then used to construct an error message.
The error message consists of at least the following:
a)
An indication that this message is from an assertion
b)
The value of the severity level
c)
The value of the message string
d)
The name of the design unit (see 13.1) containing the assertion
A line feed (LF) format effector occurring as an element of the message string is interpreted by the
implementation as signifying the end of a line. The implementation shall transform the LF into the
implementation-defined representation of the end of a line.
An implementation should continue execution of a model after occurrence of an assertion violation in which
the severity level is NOTE, WARNING, or ERROR.
NOTE 1—An implementation may choose whether or not to continue execution of a model after occurrence of assertion
violations with various severity levels. It may also give tool users ability to control simulator actions for assertions of
various severity levels via mechanisms not specified by this standard.
NOTE 2—The inadvertent insertion of a semicolon between the condition and the reserved word report in an assertion
statement does not cause an error. Rather, it causes the statement to be parsed as an assertion statement with no report or
severity clause, followed by a report statement.
### 10.4 Report statement

A report statement displays a message.
```ebnf
report_statement ::=
    [ label : ]
        report expression
        [ severity expression ] ;
```

The report statement expression shall be of the predefined type STRING. The string value of this
expression is included in the message generated by the report statement. If the severity clause is present, it
shall specify an expression of predefined type SEVERITY_LEVEL. The severity clause specifies a severity
level associated with the report. In the absence of a severity clause for a given report, the default value of the
severity level is NOTE.
Execution of a report statement consists of the evaluation of the report expression and severity clause
expression, if present. The specified message string and severity level (or corresponding default, if the
severity level is not specified) are then used to construct a report message.
The report message consists of at least the following:
a)
An indication that this message is from a report statement
b)
The value of the severity level
c)
The value of the message string
d)
The name of the design unit containing the report statement
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
180
Copyright © 2019 IEEE. All rights reserved.
An LF format effector occurring as an element of the message string is interpreted by the implementation as
signifying the end of a line. The implementation shall transform the LF into the implementation-defined
representation of the end of a line.
An implementation should continue execution of a model after displaying a report message in which the
severity level is NOTE, WARNING, or ERROR.
NOTE—An implementation may choose whether or not to continue execution of a model after execution of report
statements with various severity levels. It may also give tool users ability to control simulator actions for report
statements of various severity levels via mechanisms not specified by this standard.
Example:
report "Entering process P";
   -- A report statement with default severity NOTE.
report "Setup or Hold violation; outputs driven to 'X'"
   severity warning;
   -- Another report statement; severity is specified.
### 10.5 Signal assignment statement

#### 10.5.1 General

A signal assignment statement modifies the projected output waveforms contained in the drivers of one or
more signals (see 14.7.2), schedules a force for one or more signals, or schedules release of one or more
signals (see 14.7.3).
```ebnf
signal_assignment_statement ::=
        [ label : ] simple_signal_assignment
    |   [ label : ] conditional_signal_assignment
    |   [ label : ] selected_signal_assignment
```

#### 10.5.2 Simple signal assignments

##### 10.5.2.1 General

```ebnf
simple_signal_assignment ::=
        simple_waveform_assignment
    |   simple_force_assignment
    |   simple_release_assignment
simple_waveform_assignment ::=
    target <= [ delay_mechanism ] waveform ;
simple_force_assignment ::=
    target <= force [ force_mode ] conditional_or_unaffected_expression ;
simple_release_assignment ::=
    target <= release [ force_mode ] ;
force_mode ::= in | out
delay_mechanism ::=
```

Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
181
Copyright © 2019 IEEE. All rights reserved.
        transport
    |   [ reject time_expression ] inertial
```ebnf
target ::=
        name
    |   aggregate
waveform ::=
        waveform_element { , waveform_element }
    |   unaffected
```

If the target of the signal assignment statement is a name, then the name shall denote a signal. For a simple
waveform assignment, the base type of the value component of each transaction produced by a waveform
element on the right-hand side shall be the same as the base type of the signal denoted by the target. This
form of signal assignment assigns right-hand side values to the drivers associated with a single (scalar or
composite) signal. For a simple force assignment, the base type of the expression on the right-hand side shall
be the same as the base type of the signal denoted by the target. This form of signal assignment schedules
either a driving-value force or an effective-value force for a single signal, with the expression value being the
driving force value or effective force value, respectively. A simple release assignment schedules a
driving-value release or an effective-value release for a single signal.
If the target of the signal assignment statement is in the form of an aggregate, then the type of the aggregate
shall be determinable from the context, excluding the aggregate itself but including the fact that the type of
the aggregate shall be a composite type. Furthermore, the expression in each element association of the
aggregate shall be a locally static name that denotes a signal. For a simple waveform assignment, the base
type of the value component of each transaction produced by a waveform element on the right-hand side
shall be the same as the base type of the aggregate. This form of signal assignment assigns slices or
subelements of the right-hand side values to the drivers associated with the signal named as the
corresponding slice subelement of the aggregate. It is an error if the target of a simple force assignment or a
simple release assignment is in the form of an aggregate.
If the target of a signal assignment statement is in the form of an aggregate, and if the expression in an
element association of that aggregate is a signal name that denotes a given signal, then the given signal and
each subelement thereof (if any) are said to be identified by that element association as targets of the
assignment statement. It is an error if a given signal or any subelement thereof is identified as a target by
more than one element association in such an aggregate. Furthermore, it is an error if an element association
in such an aggregate contains an others choice, or if the element association contains a choice that is a
discrete range and an expression of a type other than the aggregate type.
The right-hand side of a simple waveform assignment may optionally specify a delay mechanism. A delay
mechanism consisting of the reserved word transport specifies that the delay associated with the first
waveform element is to be construed as transport delay. Transport delay is characteristic of hardware
devices (such as transmission lines) that exhibit nearly infinite frequency response: any pulse is transmitted,
no matter how short its duration. If no delay mechanism is present, or if a delay mechanism including the
reserved word inertial is present, the delay is construed to be inertial delay. Inertial delay is characteristic of
switching circuits: a pulse whose duration is shorter than the switching time of the circuit will not be
transmitted, or in the case that a pulse rejection limit is specified, a pulse whose duration is shorter than that
limit will not be transmitted.
Every inertially delayed signal assignment has a pulse rejection limit. If the delay mechanism specifies
inertial delay, and if the reserved word reject followed by a time expression is present, then the time
expression specifies the pulse rejection limit. In all other cases, the pulse rejection limit is specified by the
time expression associated with the first waveform element.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
182
Copyright © 2019 IEEE. All rights reserved.
It is an error if the pulse rejection limit for any inertially delayed signal assignment statement is either
negative or greater than the time expression associated with the first waveform element.
A simple waveform assignment or simple force assignment from an expression that evaluates to the special
value of unaffected has the same effect as replacing the given assignment with a null statement (not an
assignment with a null waveform element).
The right-hand side of a simple force assignment or a simple release assignment may optionally specify a
force mode. A force mode consisting of the reserved word in specifies that an effective-value force or an
effective-value release is to be scheduled, and a force mode consisting of the reserved word out specifies
that a driving-value force or a driving-value release is to be scheduled.
If the right-hand side of a simple force assignment or a simple release assignment does not specify a force
mode, then a default force mode is used, as follows:
—
If the target is a signal port or signal parameter of mode in, a force mode of in is used.
—
If the target is a signal port of mode out, inout, or buffer, or a signal parameter of mode out or
inout, a force mode of out is used.
—
If the target is not a signal port or a signal parameter, a force mode of in is used.
It is an error if a force mode of out is specified and the target is a signal port of mode in. It is an error if a
force mode of in is specified and the target is a port of mode out.
It is an error if a simple force assignment schedules a driving value force or an effective value force for a
member of a resolved composite signal.
NOTE—If a right-hand side value expression is either a numeric literal or an attribute that yields a result of type
universal_integer or universal_real, then an implicit type conversion is performed.
Examples:
-- Assignments using inertial delay:
-- The following three assignments are equivalent to each other:
Output_pin <= Input_pin after 10 ns;
Output_pin <= inertial Input_pin after 10 ns;
Output_pin <= reject 10 ns inertial Input_pin after 10 ns;
-- Assignments with a pulse rejection limit less than the time
-- expression:
Output_pin <= reject 5 ns inertial Input_pin after 10 ns;
Output_pin <=
reject 5 ns inertial Input_pin after 10 ns,
not Input_pin after 20 ns;
-- Assignments using transport delay:
Output_pin <= transport Input_pin after 10 ns;
Output_pin <=
transport Input_pin after 10 ns,
not Input_pin after 20 ns;
-- Their equivalent assignments:
Output_pin <=
reject 0 ns inertial Input_pin after 10 ns;
Output_pin <=
reject 0 ns inertial Input_pin after 10 ns,
not Input_pin after 20 ns;
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
183
Copyright © 2019 IEEE. All rights reserved.
##### 10.5.2.2 Executing a simple assignment statement

The effect of execution of a simple waveform assignment statement is defined in terms of its effect upon the
projected output waveforms (see 14.7.2) representing the current and future values of drivers of signals.
```ebnf
waveform_element ::=
```

value_expression [ after time_expression ]

|
null [ after time_expression ]
The future behavior of the driver(s) for a given target is defined by transactions produced by the evaluation
of waveform elements in the waveform of a simple waveform assignment statement. The first form of
waveform element is used to specify that the driver is to assign a particular value to the target at the specified
time. The second form of waveform element is used to specify that the driver of the signal is to be turned off,
so that it (at least temporarily) stops contributing to the value of the target. This form of waveform element
is called a null waveform element. It is an error if the target of a simple waveform assignment statement
containing a null waveform element is not a guarded signal or an aggregate of guarded signals.
The base type of the time expression in each waveform element shall be the predefined physical type TIME
as defined in package STANDARD. If the after clause of a waveform element is not present, then an
implicit “after 0 ns” is assumed. It is an error if the time expression in a waveform element evaluates to a
negative value.
Evaluation of a waveform element produces a single transaction. The time component of the transaction is
determined by the current time added to the value of the time expression in the waveform element. For the
first form of waveform element, the value component of the transaction is determined by the value
expression in the waveform element. For the second form of waveform element, the value component is not
defined by the language, but it is defined to be of the type of the target. A transaction produced by the
evaluation of the second form of waveform element is called a null transaction.
For the execution of a simple waveform assignment statement whose target is of a scalar type, the waveform
on its right-hand side is first evaluated. Evaluation of a waveform consists of the evaluation of each
waveform element in the waveform. Thus, the evaluation of a waveform results in a sequence of
transactions, where each transaction corresponds to one waveform element in the waveform. These
transactions are called new transactions. It is an error if the sequence of new transactions is not in ascending
order with respect to time. It is also an error if the value of any value expression in the waveform does not
belong to the subtype of the target.
The sequence of transactions is then used to update the projected output waveform representing the current
and future values of the driver associated with the simple waveform assignment statement. Updating a
projected output waveform consists of the deletion of zero or more previously computed transactions (called
old transactions) from the projected output waveform and the addition of the new transactions, as follows:
a)
All old transactions that are projected to occur at or after the time at which the earliest new
transaction is projected to occur are deleted from the projected output waveform.
b)
The new transactions are then appended to the projected output waveform in the order of their
projected occurrence.
If the initial delay is inertial delay according to the definitions of 10.5.2.1, the projected output waveform is
further modified as follows:
1)
All of the new transactions are marked.
2)
An old transaction is marked if the time at which it is projected to occur is less than the time at
which the first new transaction is projected to occur minus the pulse rejection limit.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
184
Copyright © 2019 IEEE. All rights reserved.
3)
For each remaining unmarked, old transaction, the old transaction is marked if it immediately
precedes a marked transaction and its value component is the same as that of the marked transaction.
4)
The transaction that determines the current value of the driver is marked.
5)
All unmarked transactions (all of which are old transactions) are deleted from the projected output
waveform.
For the purposes of marking transactions, any two successive null transactions in a projected output
waveform are considered to have the same value component.
The execution of a simple waveform assignment statement whose target is of a composite type proceeds in a
similar fashion, except that the evaluation of the waveform results in one sequence of transactions for each
scalar subelement of the type of the target. Each such sequence consists of transactions whose value portions
are determined by the values of the same scalar subelement of the value expressions in the waveform, and
whose time portion is determined by the time expression corresponding to that value expression. Each such
sequence is then used to update the projected output waveform of the driver of the matching subelement of
the target. This applies both to a target that is the name of a signal of a composite type and to a target that is
in the form of an aggregate.
For the execution of a simple force assignment whose target is of a scalar type, the expression on its
right-hand side is first evaluated. It is an error if the value of the expression does not belong to the subtype of
the target. The value of the expression is then used to schedule a driving-value force or an effective-value
force.
The execution of a simple force assignment whose target is of a composite type proceeds in a similar
fashion, except that the evaluation of the expression results in one value for each scalar subelement of the
type of the target. Each such value is then used to schedule a driving-value force or an effective-value force
of the matching subelement of the target.
For the execution of a simple release assignment whose target is of a scalar type, a driving-value release or
an effective-value release is scheduled for the target. The execution of a simple release assignment whose
target is of a composite type proceeds in a similar fashion, except that a driving-value release or an
effective-value release is scheduled for each scalar subelement of the target.
It is an error if the target of a simple force assignment or a simple release assignment is a member of a
resolved composite signal.
If a given procedure is declared by a declarative item that is not contained within a process statement, and if
a simple waveform assignment statement appears in that procedure, then the target of the simple waveform
assignment shall be a formal parameter of the given procedure or of a parent of that procedure, or an
aggregate of such formal parameters. Similarly, if a given procedure is declared by a declarative item that is
not contained within a process statement, and if a signal is associated with an inout or out mode signal
parameter in a subprogram call within that procedure, then the signal so associated shall be a formal
parameter of the given procedure or of a parent of that procedure.
NOTE 1—These rules require that the driver affected by a simple waveform assignment statement is always statically
determinable if the simple waveform assignment appears within a given process (including the case in which it appears
within a procedure that is declared within the given process). In this case, the affected driver is the one defined by the
process; otherwise, the simple waveform assignment will appear within a procedure, and the affected driver is the one
passed to the procedure along with a signal parameter of that procedure. Simple force assignments and simple release
assignments, on the other hand, do not involve drivers. Hence, the target of such an assignment occurring in a procedure
not contained with a process statement need not be a signal parameter of the procedure.
NOTE 2—Overloading the operator "=" has no effect on the updating of a projected output waveform.
NOTE 3—Consider a signal assignment statement of the form:
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
185
Copyright © 2019 IEEE. All rights reserved.
T <= reject tr inertial e1 after t1 { , ei after ti }
NOTE 4—The following relations hold:
NOTE 5—and
NOTE 6—Note that, if tr = 0 ns, then the waveform editing is identical to that for transport-delayed assignment; and if tr
= t1, the waveform is identical to that for the statement
T <= e1 after t1 { , ei after ti }
NOTE 7—Consider the following signal assignment in some process:
S <= reject 15 ns inertial 12 after 20 ns, 18 after 41 ns;
where S is a signal of some integer type.
NOTE 8—Assume that at the time this signal assignment is executed, the driver of S in the process has the following
contents (the first entry is the current driving value):
NOTE 9—(The times given are relative to the current time.) The updating of the projected output waveform proceeds as
follows:
—
The driver is truncated at 20 ns. The driver now contains the following pending transactions:
—
The new waveforms are added to the driver. The driver now contains the following pending transactions:
—
All new transactions are marked, as well as those old transactions that occur at less than the time of the first new
waveform (20 ns) less the rejection limit (15 ns). The driver now contains the following pending transactions
(marked transactions are in bold type):
1
2
2
12
5
8
NOW
+3 ns
+12 ns
+13 ns
+20 ns
+42 ns
1
2
2
12
NOW
+3 ns
+12 ns
+13 ns
1
2
2
12
12
18
NOW
+3 ns
+12 ns
+13 ns
+20 ns
+41 ns
1
2
2
12
12
18
NOW
+3 ns
+12 ns
+13 ns
+20 ns
+41 ns
## 0 ns

tr
t1
≤
≤
## 0 ns

ti
ti
1
+
<
≤
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
186
Copyright © 2019 IEEE. All rights reserved.
—
Each remaining unmarked transaction is marked if it immediately precedes a marked transaction and has the
same value as the marked transaction. The driver now contains the following pending transactions:
—
The transaction that determines the current value of the driver is marked, and all unmarked transactions are then
deleted. The final driver contents are then as follows, after clearing the markings:
#### 10.5.3 Conditional signal assignments

The conditional signal assignment represents an equivalent if statement that assigns values to signals
```ebnf
conditional_signal_assignment ::=
      target  <=  [ delay_mechanism ] conditional_waveforms ;
conditional_waveforms ::=
      waveform when condition
      { else waveform when condition }
      [ else waveform ]
```

The delay mechanism for a conditional waveform assignment statement is discussed in 10.5.2.1.
For a given conditional signal assignment, there is an equivalent sequential statement with the same
meaning. If the conditional signal assignment is of the form:
target <= delay_mechanism
          waveform1   when condition1   else
          waveform2   when condition2   else
             ·
             ·
             ·
          waveformN-1 when conditionN-1 else
          waveformN   when conditionN;
then the equivalent sequential statement is of the form:
if condition1 then
   target <= delay_mechanism waveform1;
elsif condition2 then
   target <= delay_mechanism waveform2;
      ·
      ·
      ·
elsif conditionN-1 then
   target <= delay_mechanism waveformN-1;
elsif conditionN then
   target <= delay_mechanism waveformN;
1
2
2
12
12
18
NOW
+3 ns
+12 ns
+13 ns
+20 ns
+41 ns
1
2
12
12
18
NOW
+3 ns
+13 ns
+20 ns
+41 ns
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
187
Copyright © 2019 IEEE. All rights reserved.
end if;
If the conditional signal assignment is of the form:
target <= delay_mechanism
          waveform1   when condition1   else
          waveform2   when condition2   else
             ·
             ·
             ·
          waveformN-1 when conditionN-1 else
          waveformN;
then the equivalent sequential statement is of the form:
if condition1 then
   target <= delay_mechanism waveform1;
elsif condition2 then
   target <= delay_mechanism waveform2;
      ·
      ·
      ·
elsif conditionN-1 then
   target <= delay_mechanism waveformN-1;
else
   target <= delay_mechanism waveformN;
end if;
The characteristics of the target, waveforms, expressions, and conditions in the conditional assignment
statement shall be such that the equivalent sequential statement is a legal statement.
If a label appears on the signal assignment statement containing the conditional signal assignment, then the
same label appears on the equivalent sequential statement. If a delay mechanism appears in a conditional
waveform assignment, then the same delay mechanism appears in every simple waveform assignment
statement in the equivalent sequential statement.
Example:
S <= unaffected when Input_pin = S'Driving_Value else
     Input_pin after Buffer_Delay;
#### 10.5.4 Selected signal assignments

The selected signal assignment represents an equivalent case statement that assigns values to signals or that
forces or releases signals.
```ebnf
selected_signal_assignment ::=
```

selected_waveform_assignment
|
selected_force_assignment
```ebnf
selected_waveform_assignment ::=
```

with expression select [ ? ]
target <= [ delay_mechanism ] selected_waveforms ;
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
188
Copyright © 2019 IEEE. All rights reserved.
```ebnf
selected_waveforms ::=
```

{ waveform when choices , }

waveform when choices
```ebnf
selected_force_assignment ::=
```

with expression select [ ? ]

target <= force [ force_mode ] selected_expressions ;
```ebnf
selected_expressions ::=
```

{ expression when choices , }
expression when choices
The delay mechanism for a selected waveform assignment statement is discussed in 10.5.2.1.
For a given selected signal assignment, there is an equivalent sequential statement with the same meaning. If
the selected signal assignment is of the form:
with expression select
   target <= delay_mechanism waveform1   when choice_list1,
                             waveform2   when choice_list2,
                                ·
                                ·
                                ·
                             waveformN-1 when choice_listN-1,
                             waveformN   when choice_listN;
then the equivalent sequential statement is of the form:
case expression is
   when choice_list1 =>
      target <= delay_mechanism waveform1;
   when choice_list2 =>
      target <= delay_mechanism waveform2;
         ·
         ·
         ·
   when choice_listN-1 =>
      target <= delay_mechanism waveformN-1;
   when choice_listN =>
      target <= delay_mechanism waveformN;
end case;
If the selected signal assignment is of the form:
with expression select
   target <= force expression1   when choice_list1,
                   expression2   when choice_list2,
                      ·
                      ·
                      ·
                   expressionN-1 when choice_listN-1,
                   expressionN   when choice_listN;
then the equivalent sequential statement is of the form:
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
189
Copyright © 2019 IEEE. All rights reserved.
case expression is
   when choice_list1 =>
      target <= force expression1;
   when choice_list2 =>
      target <= force expression2;
         ·
         ·
         ·
   when choice_listN-1 =>
      target <= force expressionN-1;
   when choice_listN =>
      target <= force expressionN;
end case;
If a selected signal assignment statement includes the question mark delimiter, then the equivalent
sequential statement includes a question mark delimiter after both occurrences of the reserved word case;
otherwise the equivalent sequential statement does not include the question mark delimiters.
The characteristics of the select expression, the target, the waveforms, the expressions, and the choices in the
selected assignment statement shall be such that the equivalent sequential statement is a legal statement.
If a label appears on the signal assignment statement containing the selected signal assignment, then the
same label appears on the equivalent sequential statement. If a delay mechanism appears in a selected
waveform assignment, then the same delay mechanism appears in every simple waveform assignment
statement in the equivalent sequential statement.
### 10.6 Variable assignment statement

#### 10.6.1 General

A variable assignment statement replaces the current value of a variable with a new value specified by an
expression. The named variable and the right-hand side expression shall be of the same type.
```ebnf
variable_assignment_statement ::=
        [ label : ] simple_variable_assignment
      | [ label : ] selected_variable_assignment
```

#### 10.6.2 Simple variable assignments

##### 10.6.2.1 General

```ebnf
simple_variable_assignment ::=
      target := conditional_or_unaffected_expression ;
```

If the target of the variable assignment statement is a name, then the name shall denote a variable, and the
base type of the expression on the right-hand side shall be the same as the base type of the variable denoted
by that name. It is an error if the type of the target is a protected type or a composite of protected type. This
form of variable assignment assigns the right-hand side value to a single (scalar or composite) variable.
If the target of the variable assignment statement is in the form of an aggregate, then the type of the
aggregate shall be determinable from the context, excluding the aggregate itself but including the fact that
the type of the aggregate shall be a composite type. The base type of the expression on the right-hand side
shall be the same as the base type of the aggregate. Furthermore, the expression in each element association
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
190
Copyright © 2019 IEEE. All rights reserved.
of the aggregate shall be a locally static name that denotes a variable. This form of variable assignment
assigns each subelement or slice of the right-hand side value to the variable named as the corresponding
subelement or slice of the aggregate.
If the target of a variable assignment statement is in the form of an aggregate, and if the locally static name
in an element association of that aggregate denotes a given variable or denotes another variable of which the
given variable is a subelement or slice, then the element association is said to identify the given variable as a
target of the assignment statement. It is an error if a given variable is identified as a target by more than one
element association in such an aggregate. Furthermore, it is an error if an element association in such an
aggregate contains an others choice, or if the element association contains a choice that is a discrete range
and an expression of a type other than the aggregate type.
For the execution of a variable assignment whose target is a variable name, the variable name and the
expression are first evaluated. A check is then made that the value of the expression belongs to the subtype
of the variable, except in the case of a variable that is of a composite type (in which case the assignment
involves a subtype conversion). Finally, each subelement of the variable that is not forced is updated with
the corresponding subelement of the expression. A design is erroneous if it depends on the order of
evaluation of the target and source expressions of an assignment statement.
The execution of a variable assignment whose target is in the form of an aggregate proceeds in a similar
fashion, except that each of the names in the aggregate is evaluated, and a subtype check is performed for
each subelement or slice of the right-hand side value that corresponds to one of the names in the aggregate.
For each variable denoted by a name corresponding to a subelement or slice of the right-hand side value,
each subelement of the variable that is not forced is updated with the corresponding subelement of the
subelement or slice of the right-hand side value.
An error occurs if the aforementioned subtype checks fail.
A simple variable assignment from an expression that evaluates to the special value of unaffected has the
same effect as replacing the given assignment by a null statement (not an assignment of a null value).
NOTE 1—If the right-hand side is either a numeric literal or an attribute that yields a result of type universal integer or
universal real, then an implicit type conversion is performed.
NOTE 2—For a variable assignment whose target is a name, no subelement of the target can be of a protected type.
NOTE 3—For a variable assignment whose target is in the form of an aggregate, no element of the target can be of a
protected type, nor can any subelement of any element of the target be of a protected type.
NOTE 4—The value of a composite variable or of any element or slice of a composite variable is considered to have
changed if any of the subelements of the variable, element, or slice changes value.
##### 10.6.2.2 Composite variable assignments

If the target of an assignment statement is a name denoting a composite variable (including a slice), the
value assigned to the target is implicitly converted to the subtype of the composite variable; the result of this
subtype conversion becomes the new value of the composite variable.
This means that the new value of each element of the composite variable is specified by the matching
element (see 9.2.3) in the corresponding composite value obtained by evaluation of the expression. The
subtype conversion checks that for each element of the composite variable there is a matching element in the
composite value, and vice versa. An error occurs if this check fails.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
191
Copyright © 2019 IEEE. All rights reserved.
#### 10.6.3 Selected variable assignments

The selected variable assignment represents an equivalent case statement that assigns values to variables.
```ebnf
selected_variable_assignment ::=
```

with expression select [ ? ]
target := selected_expressions ;
For a given selected variable assignment, there is an equivalent sequential statement with the same meaning.
If the selected variable assignment is of the form:
with expression select
   target := expression1   when choice_list1,
             expression2   when choice_list2,
                ·
                ·
                ·
             expressionN-1 when choice_listN-1,
             expressionN   when choice_listN;
then the equivalent sequential statement is of the form:
case expression is
   when choice_list1 =>
      target := expression1;
   when choice_list2 =>
      target := expression2;
         ·
         ·
         ·
   when choice_listN-1 =>
      target := expressionN-1;
   when choice_listN =>
      target := expressionN;
end case;
If a selected variable assignment statement includes the question mark delimiter, then the equivalent
sequential statement includes a question mark delimiter after both occurrences of the reserved word case;
otherwise the equivalent sequential statement does not include the question mark delimiters.
The characteristics of the select expression, the expressions, and the choices in the selected assignment
statement shall be such that the equivalent sequential statement is a legal statement.
If a label appears on the variable assignment statement containing the selected variable assignment, then the
same label appears on the equivalent sequential statement.
### 10.7 Procedure call statement

A procedure call invokes the execution of a procedure body.
```ebnf
procedure_call_statement ::=  [ label : ] procedure_call ;
```

Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
192
Copyright © 2019 IEEE. All rights reserved.
```ebnf
procedure_call ::=  procedure_name
```

[ generic_map_aspect ] [ parameter_map_aspect ]
The procedure name specifies the procedure body to be invoked. It is an error if the procedure name denotes
an uninstantiated procedure and the procedure call does not have a generic map aspect. The generic map
aspect, if present, specifies the association of actual generics with formal generics of the procedure. The
parameter map aspect, if present, specifies the association of actual parameters with formal parameters of
the procedure.
For each formal parameter of a procedure, a procedure call shall specify exactly one corresponding actual
parameter. This actual parameter is specified either explicitly, by an association element (other than the
actual open) in the association list or, in the absence of such an association element, by a default expression
(see 6.5.2).
For each formal generic of a procedure, a procedure call shall specify exactly one corresponding actual
generic. This actual generic is specified either explicitly, by an association element (other than the actual
open) in the association list, or in the absence of such an association element, by a default.
Each uninstantiated procedure call with a generic map aspect is replaced by an implicit subprogram
instantiation declaration and a call to the implicitly defined subprogram instance. The implicit subprogram
instance is declared prior to the uninstantiated procedure call in the declaration region that immediately
encloses the procedure call. The call to the uninstantiated procedure is equivalent to a call to the implicitly
instantiated procedure with the same parameters that were specified in the uninstantiated procedure call. The
implicitly instantiated procedure does not have a designator that can be referenced separately from the
procedure call. Hence, ambiguity cannot be introduced by the existence of an implicit procedure instance.
Execution of a procedure call includes evaluation of the actual parameter expressions specified in the call
and evaluation of the default expressions associated with formal parameters of the procedure that do not
have actual parameters associated with them. In both cases, the resulting value shall belong to the subtype of
the associated formal parameter. (If the formal parameter is of an unconstrained or partially constrained
composite type, then any undefined index ranges of subelements of the formal parameter are determined as
described 5.3.2.2.) The procedure body is executed using the actual parameter values and default expression
values as the values of the corresponding formal parameters.
NOTE—A call to a formal generic procedure uses the parameter names and default expressions defined in the formal
generic procedure declaration, and the parameter subtypes of the associated actual generic procedure.
### 10.8 If statement

An if statement selects for execution one or none of the enclosed sequences of statements, depending on the
value of one or more corresponding conditions.
```ebnf
if_statement ::=
```

[ if_label : ]
if condition then
sequence_of_statements
{ elsif condition then
sequence_of_statements}

[ else
sequence_of_statements]

end if [ if_label ] ;
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
193
Copyright © 2019 IEEE. All rights reserved.
```ebnf
sequential_statement_body ::=
    [ sequential_statement_declarative_part
    begin ]
    sequence_of_statements
sequential_statement_declarative_part ::=
    { process_declarative_item }
```

If a label appears at the end of an if statement, it shall repeat the if label.
For the execution of an if statement, the condition specified after if and any conditions specified after elsif
are evaluated in succession (treating a final else as elsif TRUE then) until one evaluates to TRUE or all con-
ditions are evaluated and yield FALSE. If one condition evaluates to TRUE, then the corresponding
sequence of statements is executed; otherwise, none of the sequences of statements is executed.
### 10.9 Case statement

A case statement selects for execution one of a number of alternative sequences of statements; the chosen
alternative is defined by the value of an expression.
```ebnf
case_statement ::=
```

[ case_label : ]

case [ ? ] expression is

case_statement_alternative

{ case_statement_alternative }
end case [ ? ] [ case_label ] ;
```ebnf
case_statement_alternative ::=
    when choices =>
        sequence_of_statements
```

A case statement shall include the question mark delimiter either in both places, in which case the case
statement is called a matching case statement, or in neither place, in which case the case statement is called
an ordinary case statement.
The expression shall be of a discrete type or of a one-dimensional array type whose element base type is a
character type. This type shall be determined by applying the rules of 12.5 to the expression considered as a
complete context, using the rule that the expression shall be of a discrete type or a one-dimensional character
array type (in particular, the type of the case expression shall be determinable independently of the type of
the case statement choices.). It is an error if the type of the expression in a matching case statement is other
than BIT, STD_ULOGIC, or a one-dimensional array type whose element type is BIT or STD_ULOGIC.
Each choice in a case statement alternative shall be of the same type as the expression; the list of choices
specifies for which values of the expression the alternative is chosen.
For an ordinary case statement, or for a matching case statement in which the expression is of type BIT or an
array type whose element type is BIT, if the expression is the name of an object whose subtype is locally
static, whether a scalar type or an array type, then each value of the subtype shall be represented once and
only once in the set of choices of the case statement, and no other value is allowed; this rule is likewise
applied if the expression is a qualified expression or type conversion whose type mark denotes a locally
static subtype, or if the expression is a call to a function whose return type mark denotes a locally static
subtype, or if the expression is an expression described in this paragraph and enclosed in parentheses.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
194
Copyright © 2019 IEEE. All rights reserved.
For a matching case statement in which the expression is of type STD_ULOGIC, or an array type whose
element type is STD_ULOGIC, if the expression is the name of an object whose subtype is locally static,
whether a scalar type or an array type, then each value of the subtype, other than the scalar value '–' or an
array value containing '–' as an element, shall be represented once and only once in the set of choices of the
case statement. A value is represented by a choice if application of the predefined matching equality
operator to the value and the choice gives the result '1'. It is an error if a choice does not represent a value of
the subtype other than the scalar value '–' or an array value containing '–' as an element. This rule is likewise
applied if the expression is a qualified expression or type conversion whose type mark denotes a locally
static subtype, or if the expression is a call to a function whose return type mark denotes a locally static
subtype, or if the expression is an expression described in this paragraph and enclosed in parentheses.
If the expression is of a one-dimensional character array type and is not described by either of the preceding
two paragraphs, then the values of all of the choices, except the others choice, if present, shall be of the
same length. Moreover, for an ordinary case statement, or for a matching case statement in which the
expression is of an array type whose element type is BIT, each value of the (base) type of the expression
shall be represented once and only once in the set of choices, and no other value is allowed. For a matching
case statement in which the expression is of an array type whose element type is STD_ULOGIC, each value
of the (base) type of the expression, other than an array value containing '–' as an element, shall be repre-
sented (as defined in the preceding paragraph) once and only once in the set of choices of the case statement.
It is an error if a choice does not represent a value of the (base) type of the expression other than an array
value containing '–' as an element. In all cases, it is an error if the value of the expression is not of the same
length as the values of the choices. If there is only one choice and that choice is others, then the value of the
expression may be of any length.
For other forms of expression in an ordinary case statement or in a matching case statement in which the
expression is of type BIT, each value of the (base) type of the expression shall be represented once and only
once in the set of choices, and no other value is allowed. For other forms of expression in a matching case
statement in which the expression is of type STD_ULOGIC, each value of the (base) type of the expression,
other than the scalar value '–', shall be represented once and only once in the set of choices of the case
statement. It is an error if a choice does not represent a value of the (base) type of the expression other than
the scalar value '–'.
All simple expressions and discrete ranges given as choices in a case statement shall be locally static. A
choice defined by a discrete range stands for all values in the corresponding range. The choice others is only
allowed for the last alternative and as its only choice; it stands for all values (possibly none) not given in the
choices of previous alternatives. An element simple name (see 9.3.3.1) is not allowed as a choice of a case
statement alternative. For a matching case statement in which the expression is of type STD_ULOGIC, or an
array type whose element type is STD_ULOGIC, it is an error if application of the predefined matching
equality operator to the values of any two distinct choices other than the choice others gives the result '1'.
If a label appears at the end of a case statement, it shall repeat the case label.
The execution of a case statement consists of the evaluation of the expression followed by the execution of
the chosen sequence of statements. A sequence of statements in a given ordinary case statement alternative
is the chosen sequence of statements if and only if the expression “E = V” evaluates to TRUE, where “E” is
the expression, “V” is the value of one of the choices of the given case statement alternative (if a choice is a
discrete range, then this latter condition is fulfilled when V is an element of the discrete range), and the
operator “=” in the expression is the predefined “=” operator on the base type of E. A sequence of statements
in a given matching case statement alternative is the chosen sequence of statements if and only if the
condition “E ?= V” evaluates to TRUE or '1', where “E” and “V” are similarly defined and the operator “?=”
is the predefined “?=” operator on the base type of E.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
195
Copyright © 2019 IEEE. All rights reserved.
For a matching case statement in which the expression is of type STD_ULOGIC, or an array type whose
element type is STD_ULOGIC, it is an error if the value of the expression is the scalar value '–' or an array
value containing '–' as an element.
NOTE 1—The execution of a case statement chooses one and only one alternative, since the choices are exhaustive and
mutually exclusive. A qualified expression whose type mark denotes a locally static subtype can often be used as the
expression of a case statement to limit the number of choices that need be explicitly specified.
NOTE 2—An others choice is required in a case statement if the type of the expression is the type universal_integer (for
example, if the expression is an integer literal), since this is the only way to cover all values of the type
universal_integer.
NOTE 3—Overloading the operator “=” has no effect on the semantics of ordinary case statement execution. Similarly,
overloading the operator “?=” has no effect on the semantics of matching case statement execution.
NOTE 4—An others choice is generally required in a matching case statement in which the expression is of type
STD_ULOGIC, or an array type whose element type is STD_ULOGIC, since explicit choice values cannot be written to
represent metalogical values of the expression. (Application of the predefined matching equality operator with a
metalogical operand value gives the result 'X'.) Such expression values, which will nonetheless be represented by a
choice, are represented by the others choice.
### 10.10 Loop statement

A loop statement includes a sequence of statements that is to be executed repeatedly, zero or more times.
```ebnf
loop_statement ::=
    [ loop_label : ]
        [ iteration_scheme ] loop
            sequence_of_statements
        end loop [ loop_label ] ;
iteration_scheme ::=
        while condition
    |   for loop_parameter_specification
parameter_specification ::=
    identifier in discrete_range
```

If a label appears at the end of a loop statement, it shall repeat the label at the beginning of the loop
statement.
Execution of a loop statement is complete when the loop is left as a consequence of the completion of the
iteration scheme (see the following), if any, or the execution of a next statement, an exit statement, or a
return statement.
A loop statement without an iteration scheme specifies repeated execution of the sequence of statements.
For a loop statement with a while iteration scheme, the condition is evaluated before each execution of the
sequence of statements; if the value of the condition is TRUE, the sequence of statements is executed; if
FALSE, the iteration scheme is said to be complete and the execution of the loop statement is complete.
For a loop statement with a for iteration scheme, the loop parameter specification is the declaration of the
loop parameter with the given identifier. The loop parameter is an object whose type is the base type of the
discrete range. Within the sequence of statements, the loop parameter is a constant. Hence, a loop parameter
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
196
Copyright © 2019 IEEE. All rights reserved.
is not allowed as the target of an assignment statement. Similarly, the loop parameter shall not be given as an
actual corresponding to a formal of mode out or inout in an association list.
For the execution of a loop with a for iteration scheme, the discrete range is first evaluated. If the discrete
range is a null range, the iteration scheme is said to be complete and the execution of the loop statement is
therefore complete; otherwise, the sequence of statements is executed once for each value of the discrete
range (subject to the loop not being left as a consequence of the execution of a next statement, an exit
statement, or a return statement), after which the iteration scheme is said to be complete. Prior to each such
iteration, the corresponding value of the discrete range is assigned to the loop parameter. These values are
assigned in left-to-right order.
NOTE—A loop may be left as the result of the execution of a next statement if the loop is nested inside of an outer loop
and the next statement has a loop label that denotes the outer loop.
### 10.11 Next statement

A next statement is used to complete the execution of one of the iterations of an enclosing loop statement
(called loop in the following text). The completion is conditional if the statement includes a condition.
```ebnf
next_statement ::=
    [ label : ] next [ loop_label ] [ when condition ] ;
```

A next statement with a loop label is only allowed within the labeled loop and applies to that loop; a next
statement without a loop label is only allowed within a loop and applies only to the innermost enclosing loop
(whether labeled or not).
For the execution of a next statement, the condition, if present, is first evaluated. The current iteration of the
loop is terminated if the value of the condition is TRUE or if there is no condition.
### 10.12 Exit statement

An exit statement is used to complete the execution of an enclosing loop statement (called loop in the fol-
lowing text). The completion is conditional if the statement includes a condition.
```ebnf
exit_statement ::=
      [ label : ] exit [ loop_label ] [ when condition ] ;
```

An exit statement with a loop label is only allowed within the labeled loop and applies to that loop; an exit
statement without a loop label is only allowed within a loop and applies only to the innermost enclosing loop
(whether labeled or not).
For the execution of an exit statement, the condition, if present, is first evaluated. Exit from the loop then
takes place if the value of the condition is TRUE or if there is no condition.
### 10.13 Return statement

A return statement is used to complete the execution of the innermost enclosing function or procedure body.
```ebnf
return_statement ::=
```

plain_return_statement
    |   value_return_statement
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
197
Copyright © 2019 IEEE. All rights reserved.
```ebnf
 plain_return_statement ::=
    [ label : ] return [ when condition ];
value_return_statement ::=
    [ label : ] return conditional_or_unaffected_expression;
```

A return statement is only allowed within the body of a function or procedure, and it applies to the innermost
enclosing function or procedure.
A return statement appearing in a procedure body shall be a plain return statement. For the execution of the
return plain statement, its condition, if present, is first evaluated. If there is a condition and its value is
FALSE, the return statement completes and the execution of the subprogram continues. Otherwise, the
execution of both the return statement and the enclosing procedure is completed.
A return statement appearing in a function body shall be a value return statement with a conditional or
unaffected expression. For the execution of a return value statement, the conditional or unaffected
expression is first evaluated. If its value is unaffected, the return statement completes and the execution of
the function continues. Otherwise, its value defines the result returned by the function. The type of this
expression shall be the base type of the type mark given after the reserved word return in the specification
of the function. An error occurs at the place of the return statement if the conversion to the result subtype
fails. Otherwise, the execution of both the return statement and the enclosing function is completed. It is an
error if the execution of a function completes by any means other than the execution of a return statement
whose conditional or unaffected expression evaluates to a value other than unaffected.
NOTE—If the expression is either a numeric literal, or an attribute that yields a result of type universal_integer or
universal_real, then an implicit conversion of the result is performed.
### 10.14 Null statement

A null statement performs no action.
```ebnf
null_statement ::=
    [ label : ] null ;
```

The execution of the null statement has no effect other than to pass on to the next statement.
NOTE—The null statement can be used to specify explicitly that no action is to be performed when certain conditions
are true, although it is never mandatory for this (or any other) purpose. This is particularly useful in conjunction with the
case statement, in which all possible values of the case expression will be covered by choices; for certain choices, it may
be that no action is required.
### 10.15 Sequential block statement

A sequential block statement encloses a sequence of sequential statements. Sequential block statements may
be nested.
```ebnf
sequential_block_statement ::=
    [ sequential_block_label : ] block  [ is ]
        sequential_block_declarative_part
    begin
        sequential_block_statement_part
    end [ block ] [ sequential_block_label ] ;
```

Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
198
Copyright © 2019 IEEE. All rights reserved.
```ebnf
sequential_block_declarative_part ::=
    { process_declarative_item }
sequential_block_statement_part ::=
```

{ sequential_statement }
If a label appears at the end of a sequential block statement, it shall repeat the sequential block label.
The execution of a sequential block statement consists of the elaboration of the sequential block declarative
part followed by the execution of the statements in the sequential block statement part.
It is an error if a variable declaration in a process declarative part declares a shared variable.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
