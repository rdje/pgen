---
title: "Section 31: IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language"
document: "SystemVerilog Language Reference Manual"
standard: "IEEE 1800-2017"
domain: "SystemVerilog"
section: "31"
source_txt: "section-31-timing-checks.txt"
source_pdf: "/Users/richarddje/Documents/github/SystemVerilog-LRM-IEEE-1800-2017.pdf"
---

# Section 31: IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language

IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
863
Copyright © 2018 IEEE. All rights reserved.
31. Timing checks
### 31.1 General

This clause describes the following:
—
Stability timing checks
—
Clock and control timing checks
—
Edge control specifiers
—
Notifiers
—
Enabling timing checks
—
Vectors in timing checks
—
Negative timing checks
### 31.2 Overview

Timing checks can be placed in specify blocks to verify the timing performance of a design by making sure
critical events occur within given time limits. The syntax for system timing checks is given in Syntax 31-1.
```ebnf
system_timing_check ::=
```

// from A.7.5.1
$setup_timing_check
| $hold_timing_check
| $setuphold_timing_check
| $recovery_timing_check
| $removal_timing_check
| $recrem_timing_check
| $skew_timing_check
| $timeskew_timing_check
| $fullskew_timing_check
| $period_timing_check
| $width_timing_check
| $nochange_timing_check
```ebnf
$setup_timing_check ::=
```

// from A.7.5.1
$setup ( data_event , reference_event , timing_check_limit [ , [ notifier ] ] ) ;
```ebnf
$hold_timing_check ::=
```

$hold ( reference_event , data_event , timing_check_limit [ , [ notifier ] ] ) ;
```ebnf
$setuphold_timing_check ::=
```

$setuphold ( reference_event , data_event , timing_check_limit , timing_check_limit
[ , [ notifier ] [ , [ timestamp_condition ] [ , [ timecheck_condition ]
[ , [ delayed_reference ] [ , [ delayed_data ] ] ] ] ] ] ) ;
```ebnf
$recovery_timing_check ::=
```

$recovery ( reference_event , data_event , timing_check_limit [ , [ notifier ] ] ) ;
```ebnf
$removal_timing_check ::=
```

$removal ( reference_event , data_event , timing_check_limit [ , [ notifier ] ] ) ;
```ebnf
$recrem_timing_check ::=
```

$recrem ( reference_event , data_event , timing_check_limit , timing_check_limit
[ , [ notifier ] [ , [ timestamp_condition ] [ , [ timecheck_condition ]
[ , [ delayed_reference ] [ , [ delayed_data ] ] ] ] ] ] ) ;
```ebnf
$skew_timing_check ::=
```

Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
864
Copyright © 2018 IEEE. All rights reserved.
$skew ( reference_event , data_event , timing_check_limit [ , [ notifier ] ] ) ;
```ebnf
$timeskew_timing_check ::=
```

$timeskew ( reference_event , data_event , timing_check_limit
[ , [ notifier ] [ , [ event_based_flag ] [ , [ remain_active_flag ] ] ] ] ) ;
```ebnf
$fullskew_timing_check ::=
```

$fullskew ( reference_event , data_event , timing_check_limit , timing_check_limit
[ , [ notifier ] [ , [ event_based_flag ] [ , [ remain_active_flag ] ] ] ] ) ;
```ebnf
$period_timing_check ::=
```

$period ( controlled_reference_event , timing_check_limit [ , [ notifier ] ] ) ;
```ebnf
$width_timing_check ::=
```

$width ( controlled_reference_event , timing_check_limit , threshold [ , [ notifier ] ] ) ;
```ebnf
$nochange_timing_check ::=
```

$nochange ( reference_event , data_event , start_edge_offset , end_edge_offset [ , [ notifier ] ] ) ;
Syntax 31-1—Syntax for system timing checks (excerpt from Annex A)
The syntax for time check conditions and timing check events is given in Syntax 31-2.
```ebnf
timecheck_condition ::= mintypmax_expression
```

// from A.7.5.2
```ebnf
controlled_reference_event ::= controlled_timing_check_event
data_event ::= timing_check_event
delayed_data ::=
```

terminal_identifier
| terminal_identifier [ constant_mintypmax_expression ]
```ebnf
delayed_reference ::=
```

terminal_identifier
| terminal_identifier [ constant_mintypmax_expression ]
```ebnf
end_edge_offset ::= mintypmax_expression
event_based_flag ::= constant_expression
notifier ::= variable_identifier
reference_event ::= timing_check_event
remain_active_flag ::= constant_mintypmax_expression
timestamp_condition ::= mintypmax_expression
start_edge_offset ::= mintypmax_expression
threshold ::= constant_expression
timing_check_limit ::= expression
timing_check_event ::=
```

// from A.7.5.3
[timing_check_event_control] specify_terminal_descriptor [ &&& timing_check_condition ]
```ebnf
controlled_timing_check_event ::=
```

timing_check_event_control specify_terminal_descriptor [ &&& timing_check_condition ]
```ebnf
timing_check_event_control ::=
```

posedge
| negedge
| edge
| edge_control_specifier
```ebnf
specify_terminal_descriptor ::=
```

specify_input_terminal_descriptor
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
865
Copyright © 2018 IEEE. All rights reserved.
| specify_output_terminal_descriptor
```ebnf
edge_control_specifier ::= edge [ edge_descriptor { , edge_descriptor } ]
edge_descriptor33 ::= 01 | 10 | z_or_x zero_or_one | zero_or_one z_or_x
zero_or_one ::= 0 | 1
z_or_x ::= x | X | z | Z
timing_check_condition ::=
```

scalar_timing_check_condition
| ( scalar_timing_check_condition )
```ebnf
scalar_timing_check_condition ::=
```

expression
| ~ expression
| expression == scalar_constant
| expression === scalar_constant
| expression != scalar_constant
| expression !== scalar_constant
```ebnf
scalar_constant ::= 1'b0 | 1'b1 | 1'B0 | 1'B1 | 'b0 | 'b1 | 'B0 | 'B1 | 1 | 0
```

33) Embedded spaces are illegal.
Syntax 31-2—Syntax for time check conditions and timing check events (excerpt from Annex A)
For ease of presentation, the timing checks are divided into two groups. The first group of timing checks are
described in terms of stability time windows:
$setup
$hold
$setuphold
$recovery
$removal
$recrem
The timing checks in the second group check clock and control signals and are described in terms of the
difference in time between two events (the $nochange check involves three events):
$skew
$timeskew
$fullskew
$width
$period
$nochange
Although they begin with a $, timing checks are not system tasks. The leading $ is present because of
historical reasons, and timing checks shall not be confused with system tasks. In particular, no system task
can appear in a specify block, and no timing check can appear in procedural code.
Some timing checks can accept negative limit values. This topic is discussed in detail in 31.9.
All timing checks have both a reference event and a data event, and Boolean conditions can be associated
with each. Some of the checks have two signal arguments, one of which is the reference event and the other
is the data event. Other checks have only one signal argument, and the reference and data events are derived
from it. Reference events and data events shall only be detected by timing checks when their associated
conditions are true. See 31.7 for more information about conditions in timing checks.
Timing check evaluation is based upon the times of two events, called the timestamp event and the
timecheck event. A transition on the timestamp event signal causes the simulator to record (stamp) the time
of transition for future use in evaluating the timing check. A transition on the timecheck event signal causes
the simulator to actually evaluate the timing check to determine whether a violation has taken place.
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
866
Copyright © 2018 IEEE. All rights reserved.
For some checks, the reference event is always the timestamp event, and the data event is always the
timecheck event; while for other checks the reverse is true. And for yet other checks, the decision about
which is the timestamp and which is the timecheck event is based upon factors that are discussed in greater
detail in 31.3 and 31.4.
Every timing check can include an optional notifier that toggles whenever the timing check detects a
violation. The model can use the notifier to make behavior a function of timing check violations. Notifiers
are discussed in greater detail in 31.6.
Like expressions for module path delays, timing check limit values are constant expressions that can include
specparams.
### 31.3 Timing checks using a stability window

The following timing checks are discussed in this subclause:
$setup
$hold
$setuphold
$recovery
$removal
$recrem
These checks accept two signals, the reference event and the data event, and define a time window with
respect to one signal while checking the time of transition of the other signal with respect to the window. In
general, they all perform the following steps:
a)
Define a time window with respect to the reference signal using the specified limit or limits.
b)
Check the time of transition of the data signal with respect to the time window.
c)
Report a timing violation if the data signal transitions within the time window.
#### 31.3.1 $setup

The $setup timing check syntax is shown in Syntax 31-3.
```ebnf
$setup_timing_check ::=
```

// from A.7.5.1
$setup ( data_event , reference_event , timing_check_limit [ , [ notifier ] ] ) ;
```ebnf
data_event ::= timing_check_event
```

// from A.7.5.2
```ebnf
notifier ::= variable_identifier
reference_event ::= timing_check_event
timing_check_limit ::= expression
```

Syntax 31-3—Syntax for $setup (excerpt from Annex A)
Table 31-1 defines the $setup timing check.
Table 31-1—$setup arguments
Argument
Description
data_event
Timestamp event
reference_event
Timecheck event
limit
Non-negative constant expression
notifier (optional)
Variable (see 31.6)
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
867
Copyright © 2018 IEEE. All rights reserved.
The data event is usually a data signal, while the reference event is usually a clock signal.
The end points of the time window are determined as follows:
(beginning of time window) = (timecheck time) - limit
(end of time window) = (timecheck time)
The $setup timing check reports a timing violation in the following case:
(beginning of time window) < (timestamp time) < (end of time window)
The end points of the time window are not part of the violation region. When the limit is zero, the $setup
check shall never issue a violation.
#### 31.3.2 $hold

The $hold timing check syntax is shown in Syntax 31-4.
```ebnf
$hold_timing_check ::=
```

// from A.7.5.1
$hold ( reference_event , data_event , timing_check_limit [ , [ notifier ] ] ) ;
```ebnf
data_event ::= timing_check_event
```

// from A.7.5.2
```ebnf
notifier ::= variable_identifier
reference_event ::= timing_check_event
timing_check_limit ::= expression
```

Syntax 31-4—Syntax for $hold (excerpt from Annex A)
Table 31-2 defines the $hold timing check.
The data event is usually a data signal, while the reference event is usually a clock signal.
The end points of the time window are determined as follows:
(beginning of time window) = (timestamp time)
(end of time window) = (timestamp time) + limit
The $hold timing check reports a timing violation in the following case:
(beginning of time window) <= (timecheck time) < (end of time window)
Table 31-2—$hold arguments
Argument
Description
reference_event
Timestamp event
data_event
Timecheck event
limit
Non-negative constant expression
notifier (optional)
Variable (see 31.6)
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
868
Copyright © 2018 IEEE. All rights reserved.
Only the end of the time window is not part of the violation region. When the limit is zero, the $hold check
shall never issue a violation.
#### 31.3.3 $setuphold

The $setuphold timing check syntax is shown in Syntax 31-5.
```ebnf
$setuphold_timing_check ::=
```

// from A.7.5.1
$setuphold ( reference_event , data_event , timing_check_limit , timing_check_limit
[ , [ notifier ] [ , [ timestamp_condition ] [ , [ timecheck_condition ]
[ , [ delayed_reference ] [ , [ delayed_data ] ] ] ] ] ] ) ;
```ebnf
timecheck_condition ::= mintypmax_expression
```

// from A.7.5.2
```ebnf
data_event ::= timing_check_event
delayed_data ::=
```

terminal_identifier
| terminal_identifier [ constant_mintypmax_expression ]
```ebnf
delayed_reference ::=
```

terminal_identifier
| terminal_identifier [ constant_mintypmax_expression ]
```ebnf
notifier ::= variable_identifier
reference_event ::= timing_check_event
timestamp_condition ::= mintypmax_expression
timing_check_limit ::= expression
```

Syntax 31-5—Syntax for $setuphold (excerpt from Annex A)
Table 31-3 defines the $setuphold timing check.
The $setuphold timing check can accept negative limit values. This is discussed in greater detail in 31.9.
Table 31-3—$setuphold arguments
Argument
Description
reference_event
Timecheck or timestamp event when setup limit is positive
Timestamp event when setup limit is negative
data_event
Timecheck or timestamp event when hold limit is positive
Timestamp event when hold limit is negative
setup_limit
Constant expression
hold_limit
Constant expression
notifier (optional)
Variable (see 31.6)
timestamp_condition (optional)
Timestamp condition for negative timing checks
timecheck_condition (optional)
Timecheck condition for negative timing checks
delayed_reference (optional)
Delayed reference signal for negative timing checks
delayed_data (optional)
Delayed data signal for negative timing checks
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
869
Copyright © 2018 IEEE. All rights reserved.
The data event is usually a data signal, while the reference event is usually a clock signal.
When both the setup limit and the hold limit are positive, either the reference event or the data event can be
the timecheck event. It shall depend upon which occurs first in the simulation.
When either the setup limit or the hold limit is negative, the restriction becomes as follows:
setup_limit + hold_limit > (simulation unit of precision)
The $setuphold timing check combines the functionality of the $setup and $hold timing checks into a
single timing check. Therefore, the invocation
$setuphold( posedge clk, data, tSU, tHLD );
is equivalent in functionality to the following, if tSU and tHLD are not negative:
$setup( data, posedge clk, tSU );
$hold( posedge clk, data, tHLD );
When both setup and hold limits are positive and the data event occurs first, the end points of the time
window are determined as follows:
(beginning of time window) = (timecheck time) - limit
(end of time window) = (timecheck time)
And the $setuphold timing check reports a timing violation in the following case:
(beginning of time window) < (timestamp time) <= (end of time window)
Only the beginning of the time window is not part of the violation region. The $setuphold check shall
report a timing violation when the reference and data events occur simultaneously.
When both setup and hold limits are positive and the data event occurs second, the end points of the time
window are determined as follows:
beginning of time window) = (timestamp time)
(end of time window) = (timestamp time) + limit
And the $setuphold timing check reports a timing violation in the following case:
(beginning of time window) <= (timecheck time) < (end of time window)
Only the end of the time window is not part of the violation region. The $setuphold check shall report a
timing violation when the reference and data events occur simultaneously.
When both limits are zero, the $setuphold check shall never issue a violation.
#### 31.3.4 $removal

The $removal timing check syntax is shown in Syntax 31-6.
```ebnf
$removal_timing_check ::=
```

// from A.7.5.1
$removal ( reference_event , data_event , timing_check_limit [ , [ notifier ] ] ) ;
```ebnf
data_event ::= timing_check_event
```

// from A.7.5.2
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
870
Copyright © 2018 IEEE. All rights reserved.
```ebnf
notifier ::= variable_identifier
reference_event ::= timing_check_event
timing_check_limit ::= expression
```

Syntax 31-6—Syntax for $removal (excerpt from Annex A)
Table 31-4 defines the $removal timing check.
The reference event is usually a control signal like clear, reset, or set, while the data event is usually a clock
signal.
The end points of the time window are determined as follows:
(beginning of time window) = (timecheck time) - limit
(end of time window) = (timecheck time)
The $removal timing check reports a timing violation in the following case:
(beginning of time window) < (timestamp time) < (end of time window)
The end points of the time window are not part of the violation region. When the limit is zero, the $removal
check shall never issue a violation.
#### 31.3.5 $recovery

The $recovery timing check syntax is shown in Syntax 31-7.
```ebnf
$recovery_timing_check ::=
```

// from A.7.5.1
$recovery ( reference_event , data_event , timing_check_limit [ , [ notifier ] ] ) ;
```ebnf
data_event ::= timing_check_event
```

// from A.7.5.2
```ebnf
notifier ::= variable_identifier
reference_event ::= timing_check_event
timing_check_limit ::= expression
```

Syntax 31-7—Syntax for $recovery (excerpt from Annex A)
Table 31-4—$removal arguments
Argument
Description
reference_event
Timecheck event
data_event
Timestamp event
limit
Non-negative constant expression
notifier (optional)
Variable (see 31.6)
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
871
Copyright © 2018 IEEE. All rights reserved.
Table 31-5 defines the $recovery timing check.
The reference event is usually a control signal like clear, reset, or set, while the data event is usually a clock
signal.
The end points of the time window are determined as follows:
(beginning of time window) = (timestamp time)
(end of time window) = (timestamp time) + limit
The $recovery timing check reports a timing violation in the following case:
(beginning of time window) <= (timecheck time) < (end of time window)
Only the end of the time window is not part of the violation region. When the limit is zero, the $recovery
check shall never issue a violation.
#### 31.3.6 $recrem

The $recrem timing check syntax is shown in Syntax 31-8.
```ebnf
$recrem_timing_check ::=
```

// from A.7.5.1
$recrem ( reference_event , data_event , timing_check_limit , timing_check_limit
[ , [ notifier ] [ , [ timestamp_condition ] [ , [ timecheck_condition ]
[ , [ delayed_reference ] [ , [ delayed_data ] ] ] ] ] ] ) ;
```ebnf
timecheck_condition ::= mintypmax_expression
```

// from A.7.5.2
```ebnf
data_event ::= timing_check_event
delayed_data ::=
```

terminal_identifier
| terminal_identifier [ constant_mintypmax_expression ]
```ebnf
delayed_reference ::=
```

terminal_identifier
| terminal_identifier [ constant_mintypmax_expression ]
```ebnf
notifier ::= variable_identifier
reference_event ::= timing_check_event
timestamp_condition ::= mintypmax_expression
timing_check_limit ::= expression
```

Syntax 31-8—Syntax for $recrem (excerpt from Annex A)
Table 31-5—$recovery arguments
Argument
Description
reference_event
Timestamp event
data_event
Timecheck event
limit
Non-negative constant expression
notifier (optional)
Variable (see 31.6)
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
872
Copyright © 2018 IEEE. All rights reserved.
Table 31-6 defines the $recrem timing check.
The $recrem timing check can accept negative limit values. This is discussed in greater detail in 31.9.
When both the removal limit and the recovery limit are positive, either the reference event or the data event
can be the timecheck event. It shall depend upon which occurs first in the simulation.
When either the removal limit or the recovery limit is negative, the restriction becomes as follows:
removal_limit + recovery_limit > (simulation unit of precision)
The $recrem timing check combines the functionality of the $removal and $recovery timing checks into
a single timing check. Therefore, the invocation
$recrem( posedge clear, posedge clk, tREC, tREM );
is equivalent in functionality to the following, if tREC and tREM are not negative:
$removal( posedge clear, posedge clk, tREM );
$recovery( posedge clear, posedge clk, tREC );
When both removal and recovery limits are positive and the data event occurs first, the end points of the
time window are determined as follows:
(beginning of time window) = (timecheck time) - limit
(end of time window) = (timecheck time)
And the $recrem timing check reports a timing violation in the following case:
(beginning of time window) < (timestamp time) <= (end of time window)
Only the beginning of the time window is not part of the violation region. The $recrem check shall report a
timing violation when the reference and data events occur simultaneously.
Table 31-6—$recrem arguments
Argument
Description
reference_event
Timecheck or timestamp event when removal limit is positive
Timestamp event when removal limit is negative
data_event
Timecheck or timestamp event when recovery limit is positive
Timestamp event when recovery limit is negative
recovery_limit
Constant expression
removal_limit
Constant expression
notifier (optional)
Variable (see 31.6)
timestamp_condition (optional)
Timestamp condition for negative timing checks
timecheck_condition (optional)
Timecheck condition for negative timing checks
delayed_reference (optional)
Delayed reference signal for negative timing checks
delayed_data (optional)
Delayed data signal for negative timing checks
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
873
Copyright © 2018 IEEE. All rights reserved.
When both removal and recovery limits are positive and the data event occurs second, the end points of the
time window are determined as follows:
(beginning of time window) = (timestamp time)
(end of time window) = (timestamp time) + limit
And the $recrem timing check reports a timing violation in the following case:
(beginning of time window) <= (timecheck time) < (end of time window)
Only the end of the time window is not part of the violation region. The $recrem check shall report a timing
violation when the reference and data events occur simultaneously.
When both limits are zero, the $recrem check shall never issue a violation.
### 31.4 Timing checks for clock and control signals

The following timing checks are discussed in this subclause:
$skew
$timeskew
$fullskew
$period
$width
$nochange
These checks accept one or two signals and verify that transitions on them are never separated by more than
the limit. For checks specifying only one signal, the reference event and data event are derived from that one
signal. In general, these checks all perform the following steps:
a)
Determine the elapsed time between two events.
b)
Compare the elapsed time to the specified limit.
c)
Report a timing violation if the elapsed time violates the limit.
The skew checks have two different violation detection mechanisms, event-based and timer-based.
Event-based skew checking is performed only when a signal transitions, while timer-based skew checking
takes place as soon as the simulation time equal to the skew limit has elapsed.
The $nochange check involves three events rather than two.
#### 31.4.1 $skew

The $skew timing check syntax is shown in Syntax 31-9.
```ebnf
$skew_timing_check ::=
```

// from A.7.5.1
$skew ( reference_event , data_event , timing_check_limit [ , [ notifier ] ] ) ;
```ebnf
data_event ::= timing_check_event
```

// from A.7.5.2
```ebnf
notifier ::= variable_identifier
reference_event ::= timing_check_event
timing_check_limit ::= expression
```

Syntax 31-9—Syntax for $skew (excerpt from Annex A)
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
874
Copyright © 2018 IEEE. All rights reserved.
Table 31-7 defines the $skew timing check.
The $skew timing check reports a violation in the following case:
(timecheck time) - (timestamp time) > limit
Simultaneous transitions on the reference and data signals shall not cause $skew to report a timing violation,
even when the skew limit value is zero.
The $skew timing check is event-based; it is evaluated only after a data event. If there is never a data event
(i.e., the data event is infinitely late), the $skew timing check shall never be evaluated, and no timing
violation shall ever be reported. In contrast, the $timeskew and $fullskew checks are timer-based by
default, and they should be used if violation reports are absolutely required and the data event can be very
late or even absent altogether. These checks are discussed in 31.4.2 and 31.4.3.
$skew shall wait indefinitely for the data event once it has detected a reference event, and it shall not report
a timing violation until the data event takes place. A second consecutive reference event shall cancel the old
wait for the data event and begin a new one.
After a reference event, the $skew timing check shall never stop checking data events for a timing violation.
$skew shall report timing violations for all data events occurring beyond the limit after a reference event.
#### 31.4.2 $timeskew

The syntax for $timeskew is shown in Syntax 31-10.
```ebnf
$timeskew_timing_check ::=
```

// from A.7.5.1
$timeskew ( reference_event , data_event , timing_check_limit
[ , [ notifier ] [ , [ event_based_flag ] [ , [ remain_active_flag ] ] ] ] ) ;
```ebnf
data_event ::= timing_check_event
```

// from A.7.5.2
```ebnf
event_based_flag ::= constant_expression
notifier ::= variable_identifier
reference_event ::= timing_check_event
remain_active_flag ::= constant_mintypmax_expression
timing_check_limit ::= expression
```

Syntax 31-10—Syntax for $timeskew (excerpt from Annex A)
Table 31-7—$skew arguments
Argument
Description
reference_event
Timestamp event
data_event
Timecheck event
limit
Non-negative constant expression
notifier (optional)
Variable (see 31.6)
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
875
Copyright © 2018 IEEE. All rights reserved.
Table 31-8 defines the $timeskew timing check arguments.
The $timeskew timing check reports a violation only in the following case:
(timecheck time) - (timestamp time) > limit
Simultaneous transitions on the reference and data signals shall not cause $timeskew to report a timing
violation, even when the skew limit value is zero. $timeskew shall also not report a violation if a new
timestamp event occurs exactly at the expiration of the time limit.
The default behavior for $timeskew is timer-based. A violation shall be reported immediately upon an
elapse of time after the reference event equal to the limit, and the check shall become dormant and report no
more violations (even in response to data events) until after the next reference event. However, if a data
event occurs within the limit, then a violation shall not be reported, and the check shall become dormant
immediately. This check shall also become dormant if it detects a conditioned reference event when its
condition is false and the remain_active_flag is not set.
The $timeskew check’s default timer-based behavior can be altered to event-based using the
event_based_flag. It behaves like the $skew check when both the event_based_flag and the
remain_active_flag are set. The $timeskew check behaves like the $skew check when only the
event_based_flag is set, except that it becomes dormant after reporting the first violation or if it detects a
conditioned reference event when its condition is false.
For example, see Figure 31-1.
$timeskew (posedge CP &&& MODE, negedge CPN, 50, , event_based_flag,
remain_active_flag);
Table 31-8—$timeskew arguments
Argument
Description
reference_event
Timestamp event
data_event
Timecheck event
limit
Non-negative constant expression
notifier (optional)
Variable (see 31.6)
event_based_flag (optional)
Constant expression
remain_active_flag (optional)
Constant expression
MODE
CP
A
50
F
CPN
C
D
E
B
G
H
I
J
Figure 31-1—Sample $timeskew
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
876
Copyright © 2018 IEEE. All rights reserved.
Case 1: event_based_flag not set, remain_active_flag not set.
After the first reference event on CP at A, a violation is reported at B as soon as 50 time units have passed,
turning the $timeskew check dormant, and no further violations are reported.
Case 2: event_based_flag set, remain_active_flag not set.
After the first reference event on CP at A, the negative transition on CPN at point C causes a timing violation,
turning the $timeskew check dormant, and no further violations are reported. The second reference event at
F occurs while MODE is false; therefore, the $timeskew check remains dormant.
Case 3: event_based_flag set, remain_active_flag set.
After the first reference event on CP at A, the first three negative transitions on CPN at points C, D, and E
cause timing violations. The second reference event at F occurs while MODE is false, but because the
remain_active_flag is set, the $timeskew check remains active. Therefore, additional violations are
reported at G, H, I, and J. In other words, all negative transitions on CPN cause violations, which is identical
to $skew behavior.
Case 4: event_based_flag not set, remain_active_flag set.
For the waveform depicted in Figure 31-1, $timeskew has the same behavior in Case 4 as in Case 1. The
difference between the two cases is illustrated by the waveform in Figure 31-2.
Although the reference event on CP at F occurs while MODE is false, it does not turn the $timeskew check
dormant because the remain_active_flag is set. A violation will hence be reported at time B, whereas for
Case 1, where the remain_active_flag is not set, the $timeskew check would turn dormant at F, and no
violation would be reported.
#### 31.4.3 $fullskew

The syntax for $fullskew is shown in Syntax 31-11.
```ebnf
$fullskew_timing_check ::=
```

// from A.7.5.1
$fullskew ( reference_event , data_event , timing_check_limit , timing_check_limit
[ , [ notifier ] [ , [ event_based_flag ] [ , [ remain_active_flag ] ] ] ] ) ;
```ebnf
data_event ::= timing_check_event
```

// from A.7.5.2
```ebnf
event_based_flag ::= constant_expression
notifier ::= variable_identifier
```

MODE
CP
A
50
F
CPN
C
D
E
B
G
Figure 31-2—Sample $timeskew with remain_active_flag set
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
877
Copyright © 2018 IEEE. All rights reserved.
```ebnf
reference_event ::= timing_check_event
remain_active_flag ::= constant_mintypmax_expression
timing_check_limit ::= expression
```

Syntax 31-11—Syntax for $fullskew (excerpt from Annex A)
Table 31-9 defines the $fullskew timing check arguments.
$fullskew is similar to $timeskew except that the reference and data events can transition in either order.
The first limit is the maximum time by which the data event should follow the reference event. The second
limit is the maximum time by which the reference event should follow the data event.
The reference event is the timestamp event, and the data event is the timecheck event when the reference
event precedes the data event. The data event is the timestamp event, and the reference event is the
timecheck event when the data event precedes the reference event.
The $fullskew timing check reports a violation only in the following case, where limit is set to limit1
when the reference event transitions first and set to limit2 when the data event transitions first:
(timecheck time) - (timestamp time) > limit
Simultaneous transitions on the reference and data signals shall not cause $fullskew to report a timing
violation, even when the skew limit value is zero. $fullskew shall also not report a violation if a new
timestamp event occurs exactly at the expiration of the time limit.
The default behavior for $fullskew is timer-based (event_based_flag not set). A violation shall be reported
immediately upon elapse of the time limit after the timestamp event if a timecheck event does not occur in
this time, turning the timing check dormant. However, if a timecheck event does occur within the time limit,
then no violation is reported, and the timing check turns dormant immediately.
A reference event or data event is a timestamp event and starts a new timing window, unless it is a
timecheck event occurring within the time limit after a preceding timestamp event, in which case it turns the
timing check dormant, as previously stated.
In the timer-based mode, a second timestamp event that occurs within the time limit starts a new timing
window that replaces the first one, unless the second timestamp event has an associated condition whose
value is false. In such a case, the behavior of $fullskew depends on the remain_active_flag. If the flag is
Table 31-9—$fullskew arguments
Argument
Description
reference_event
Timestamp or timecheck event
data_event
Timestamp or timecheck event
limit 1
Non-negative constant expression
limit 2
Non-negative constant expression
notifier (optional)
Variable (see 31.6)
event_based_flag (optional)
Constant expression
remain_active_flag (optional)
Constant expression
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
878
Copyright © 2018 IEEE. All rights reserved.
set, then the second timestamp event is simply ignored. If the flag is not set and if the timing check is active,
then the timing check turns dormant.
The $fullskew check’s default timer-based behavior can be altered to event-based using the
event_based_flag. In this mode, $fullskew is similar to $skew in that a violation is reported not upon
elapse of the time limit after the timestamp event (as in timer-based mode), but rather if a timecheck event
occurs after the time limit. Such an event ends the first timing window and immediately begins a new timing
window, where it acts as the timestamp event of the new window. A timecheck event within the time limit
ends the timing window and turns the timing check dormant, and no violation is reported.
In the event-based mode, a second timestamp event that occurs before a timecheck event has occurred starts
a new timing window that replaces the first one, unless the second timestamp event has an associated
condition whose value is false. In such a case, the behavior of $fullskew depends on the
remain_active_flag. If the flag is set, then the second timestamp event is simply ignored. If the flag is not set
and if the timing check is active, then the timing check turns dormant.
In both the timer-based and event-based modes, if the timestamp event has no condition or has a true
condition and if the timing check is dormant, then the timing check is activated.
For example, see Figure 31-3.
$fullskew (posedge CP &&& MODE, negedge CPN, 50, 70,, event_based_flag,
remain_active_flag);

Case 1: event_based_flag not set.
The transition at A of CP while MODE is true begins a wait for a negative transition on CPN, and a violation is
reported at B as soon as a period of time equal to 50 time units has passed. This resets the check and readies
it for the next active transition.
A negative transition on CPN occurs next at C, beginning a wait for a positive transition on CP while MODE is
true. At D, a time equal to 70 time units has passed without a positive edge on CP while MODE is true;
therefore, a violation is reported, and the check is again reset to await the next active transition.
A transition on CPN at E also results in a timing violation, as does the transition at F, because even though CP
transitions, MODE is no longer true. Transitions at G and H also result in timing violations, but not the
transition at I because it is followed by a positive transition on CP while MODE is true.
MODE
CP
50
J
70
D
70
C
E
F
G
H
I
CPN
A B
Figure 31-3—Sample $fullskew
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
879
Copyright © 2018 IEEE. All rights reserved.
Case 2: event_based_flag set.
The transition at A of CP while MODE is true begins a wait for a negative transition on CPN, and a violation is
reported at C on CPN because it occurs beyond the 50 time unit limit. This transition at C also begins a wait
of 70 time units for a positive transition on CP while MODE is true. But for transitions on CPN at C through H,
there is no positive transition on CP while MODE is true; therefore, no timing violations are reported. The
transition at I on CPN begins a wait of 70 time units, and this is satisfied by the positive transition on CP at J
while MODE is true.
Although the waveform in this particular example does not show the role of the remain_active_flag, it
should be recognized that this flag has a vital role in determining the behavior of the $fullskew timing
check, just as it does for the $timeskew timing check.
#### 31.4.4 $width

The $width timing check syntax is shown in Syntax 31-12.
```ebnf
$width_timing_check ::=
```

// from A.7.5.1
$width ( controlled_reference_event , timing_check_limit , threshold [ , [ notifier ] ] ) ;
```ebnf
controlled_reference_event ::= controlled_timing_check_event
```

// from A.7.5.2
```ebnf
notifier ::= variable_identifier
threshold ::= constant_expression
timing_check_limit ::= expression
```

Syntax 31-12—Syntax for $width (excerpt from Annex A)
Table 31-10 defines the $width timing check.
The $width timing check monitors the width of signal pulses by measuring the time from the timestamp
event to the timecheck event. Because a data event is not passed to $width, it is derived from the reference
event, as follows:
data event = reference event signal with opposite edge
Because of the way the data event is derived for $width, an edge triggered event has to be passed as the
reference event. A compilation error shall occur if the reference event is not an edge specification.
Table 31-10—$width arguments
Argument
Description
reference_event
Timestamp edge triggered event
data_event (implicit)
Timecheck edge triggered event
limit
Non-negative constant expression
threshold (optional)
Non-negative constant expression
notifier (optional)
Variable (see 31.6)
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
880
Copyright © 2018 IEEE. All rights reserved.
While the $width timing check can be defined in terms of a time window, it is simpler to express it as the
difference between the timecheck and timestamp times. The $width timing check reports a violation in the
following case:
threshold < (timecheck time) - (timestamp time) < limit
The pulse width has to be greater than or equal to limit in order to avoid a timing violation, but no violation
is reported for glitches smaller than the threshold.
The threshold argument shall be included if the notifier argument is required. It is permissible to not specify
both the threshold and notifier arguments, making the default value for the threshold zero. If the notifier is
present, a non-null value for the threshold shall also be present. Here is a legal $width check when the
notifier is required and the threshold is not:
$width (posedge clk, 6, 0, ntfr_reg);
The data event and the reference event shall never occur at the same simulation time because these events
are triggered by opposite transitions.
The following example demonstrates some examples of legal and illegal calls:
// Legal Calls
$width ( negedge clr, lim );
$width ( negedge clr, lim, thresh, notif );
$width ( negedge clr, lim, 0, notif );
// Illegal Calls
$width ( negedge clr, lim, , notif );
$width ( negedge clr, lim, notif );
#### 31.4.5 $period

The $period timing check syntax is shown in Syntax 31-13.
```ebnf
$period_timing_check ::=
```

// from A.7.5.1
$period ( controlled_reference_event , timing_check_limit [ , [ notifier ] ] ) ;
```ebnf
controlled_reference_event ::= controlled_timing_check_event
```

// from A.7.5.2
```ebnf
notifier ::= variable_identifier
timing_check_limit ::= expression
```

Syntax 31-13—Syntax for $period (excerpt from Annex A)
Table 31-11 defines the $period timing check.
Table 31-11—$period arguments
Argument
Description
reference_event
Timestamp edge triggered event
data_event (implicit)
Timecheck edge triggered event
limit
Non-negative constant expression
notifier (optional)
Variable (see 31.6)
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
881
Copyright © 2018 IEEE. All rights reserved.
Because the data event is not passed as an argument to $period, it is derived from the reference event, as
follows:
data event = reference event signal with the same edge
Because of the way the data event is derived for $period, an edge triggered event shall be passed as the
reference event. A compilation error shall occur if the reference event is not an edge specification.
While the $period timing check can be defined in terms of a time window, it is simpler to express it as the
difference between the timecheck and timestamp times. The $period timing check reports a violation in the
following case:
(timecheck time) - (timestamp time) < limit
#### 31.4.6 $nochange

The $nochange syntax is shown in Syntax 31-14.
```ebnf
$nochange_timing_check ::=
```

// from A.7.5.1
$nochange ( reference_event , data_event , start_edge_offset , end_edge_offset [ , [ notifier ] ] );
```ebnf
data_event ::= timing_check_event
```

// from A.7.5.2
```ebnf
end_edge_offset ::= mintypmax_expression
notifier ::= variable_identifier
reference_event ::= timing_check_event
start_edge_offset ::= mintypmax_expression
```

Syntax 31-14—Syntax for $nochange (excerpt from Annex A)
Table 31-12 defines the $nochange timing check arguments.
The $nochange timing check reports a timing violation if the data event occurs during the specified level of
the control signal (the reference event). The reference event can be specified with the posedge or the
negedge keyword, but the edge-control specifiers (see 31.5) cannot be used.
The start edge and end edge offsets can expand or shrink the timing violation region, which is defined by the
duration of the reference event signal after the edge. For example, if the reference event is a posedge, then
the duration is the period during which the reference signal is high. A positive offset for start edge extends
the region by starting the timing violation region earlier; a negative offset for start edge shrinks the region by
starting the region later. Similarly, a positive offset for the end edge extends the timing violation region by
Table 31-12—$nochange arguments
Argument
Description
reference_event
Edge triggered timestamp and/or timecheck event
data_event
Timestamp or timecheck event
start_edge_offset
Constant expression
end_edge_offset
Constant expression
notifier (optional)
Variable (see 31.6)
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
882
Copyright © 2018 IEEE. All rights reserved.
ending it later, while a negative offset for the end edge shrinks the region by ending it earlier. If both the
offsets are zero, the size of the region shall not change.
Unlike other timing checks, $nochange involves three, rather than two, transitions. The leading edge of the
reference event defines the beginning of the time window, while the trailing edge of the reference event
defines the end of the time window. A violation results if the data event occurs anytime within the time
window.
The end points of the time window are determined as follows:
(beginning of time window) = (leading reference edge time) - start_edge_offset
(end of time window) = (trailing reference edge time) + end_edge_offset
The $nochange timing check reports a timing violation in the following case:
(beginning of time window) < (data event time) < (end of time window)
The end points of the time window are not included. The values of start_edge_offset and
end_edge_offset play a significant role in determining which signal, the reference event or the data
event, is the timestamp or timecheck event.
For example:
$nochange( posedge clk, data, 0, 0) ;
In this example, the $nochange timing check shall report a violation if the data signal changes while clk
is high. It shall not be a violation if posedge clk and a transition on data occur simultaneously.
### 31.5 Edge-control specifiers

The edge-control specifiers can be used to control events in timing checks based on specific edge transitions
between 0, 1, and x. Syntax 31-15 shows the syntax for edge-control specifiers.
```ebnf
edge_control_specifier ::= edge [ edge_descriptor { , edge_descriptor } ]
```

// from A.7.5.3
```ebnf
edge_descriptor33 ::= 01 | 10 | z_or_x zero_or_one | zero_or_one z_or_x
zero_or_one ::= 0 | 1
z_or_x ::= x | X | z | Z
```

33) Embedded spaces are illegal.
Syntax 31-15—Syntax for edge-control specifier (excerpt from Annex A)
Edge-control specifiers contain the keyword edge followed by a square-bracketed list of from one to six
pairs of edge transitions between 0, 1, and x, as follows:
01
Transition from 0 to 1
0x
Transition from 0 to x
10
Transition from 1 to 0
1x
Transition from 1 to x
x0
Transition from x to 0
x1
Transition from x to 1
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
883
Copyright © 2018 IEEE. All rights reserved.
Edge transitions involving z are treated the same way as edge transitions involving x.
The posedge and negedge keywords can be used as a shorthand for certain edge-control specifiers. For
example, the construct
posedge clr
is equivalent to the following:
edge[01, 0x, x1] clr
Similarly, the construct
negedge clr
is the same as the following:
edge[10, x0, 1x] clr
However, edge-control specifiers offer the flexibility to declare edge transitions other than posedge and
negedge.
### 31.6 Notifiers: user-defined responses to timing violations

Timing check notifiers detect timing check violations behaviorally and, therefore, take an action as soon as a
violation occurs. Such notifiers can be used to print an informative error message describing the violation or
to propagate an x value at the output of the device that reported the violation.
The notifier is a variable, declared in the module where timing check tasks are invoked, that is passed as the
last argument to a system timing check. Whenever a timing violation occurs, the timing check updates the
value of the notifier.
The notifier is an optional argument to all system timing checks and can be omitted from the timing check
call without adversely affecting its operation.
Table 31-13 shows how the notifier values are toggled when timing violations occur.
Example 1:
$setup( data, posedge clk, 10, notifier ) ;
$width( posedge clk, 16, 0, notifier ) ;
Table 31-13—Notifier value responses to timing violations
BEFORE violation
 AFTER violation
x
Either 0 or 1
0
1
1
0
z
z
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
884
Copyright © 2018 IEEE. All rights reserved.
Example 2: Consider a more complex example of how to use notifiers in a behavioral model. The following
example uses a notifier to set the D flip-flop output to x when a timing violation occurs in an edge-sensitive
UDP:
primitive posdff_udp(q, clock, data, preset, clear, notifier);
output q; reg q;
input clock, data, preset, clear, notifier;
table
//clock data
p c notifier state
q
//-------------------------------------
 r
0
## 1 1

?
:
?
: 0 ;
 r
1
## 1 1

?
:
?
: 1 ;
 p
1
? 1
?
:
1
: 1 ;
 p
0
## 1 ?

?
:
0
: 0 ;
 n
?
? ?
?
:
?
: - ;
 ?
*
? ?
?
:
?
: - ;
 ?
?
## 0 1

?
:
?
: 1 ;
 ?
?
* 1
?
:
1
: 1 ;
 ?
?
## 1 0

?
:
?
: 0 ;
 ?
?
## 1 *

?
:
0
: 0 ;
 ?
?
? ?
*
:
?
: x ;// At any notifier event

 // output x
endtable
endprimitive
module dff(q, qbar, clock, data, preset, clear);
output q, qbar;
input clock, data, preset, clear;
reg notifier;
and (enable, preset, clear);
not (qbar, ffout);
buf (q, ffout);
posdff_udp (ffout, clock, data, preset, clear, notifier);
specify
// Define timing check specparam values
specparam tSU = 10, tHD = 1, tPW = 25, tWPC = 10, tREC = 5;
// Define module path delay rise and fall min:typ:max values
specparam tPLHc = 4:6:9 , tPHLc = 5:8:11;
specparam tPLHpc = 3:5:6 , tPHLpc = 4:7:9;
// Specify module path delays
(clock *> q,qbar) = (tPLHc, tPHLc);
(preset,clear *> q,qbar) = (tPLHpc, tPHLpc);
// Setup time : data to clock, only when preset and clear are 1
$setup(data, posedge clock &&& enable, tSU, notifier);
// Hold time: clock to data, only when preset and clear are 1
$hold(posedge clock, data &&& enable, tHD, notifier);
// Clock period check
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
885
Copyright © 2018 IEEE. All rights reserved.
$period(posedge clock, tPW, notifier);
// Pulse width : preset, clear
$width(negedge preset, tWPC, 0, notifier);
$width(negedge clear, tWPC, 0, notifier);
// Recovery time: clear or preset to clock
$recovery(posedge preset, posedge clock, tREC, notifier);
$recovery(posedge clear, posedge clock, tREC, notifier);
endspecify
endmodule
NOTE—This model applies to edge-sensitive UDPs only; for level-sensitive models, an additional UDP for x
propagation has to be generated.
### 31.7 Enabling timing checks with conditioned events

A construct called a conditioned event ties the occurrence of timing checks to the value of a conditioning
signal. Syntax 31-16 shows the syntax for controlled timing check events.
```ebnf
timing_check_event ::=
```

// from A.7.5.3
[timing_check_event_control] specify_terminal_descriptor [ &&& timing_check_condition ]
```ebnf
controlled_timing_check_event ::=
```

timing_check_event_control specify_terminal_descriptor [ &&& timing_check_condition ]
```ebnf
timing_check_event_control ::=
```

posedge
| negedge
| edge
| edge_control_specifier
```ebnf
specify_terminal_descriptor ::=
```

specify_input_terminal_descriptor
| specify_output_terminal_descriptor
```ebnf
timing_check_condition ::=
```

scalar_timing_check_condition
| ( scalar_timing_check_condition )
```ebnf
scalar_timing_check_condition ::=
```

expression
| ~ expression
| expression == scalar_constant
| expression === scalar_constant
| expression != scalar_constant
| expression !== scalar_constant
```ebnf
scalar_constant ::= 1'b0 | 1'b1 | 1'B0 | 1'B1 | 'b0 | 'b1 | 'B0 | 'B1 | 1 | 0
```

Syntax 31-16—Syntax for controlled timing check events (excerpt from Annex A)
The comparisons used in the condition can be deterministic, as in ===, !==, ~, or no operation, or
nondeterministic, as in == or !=. When comparisons are deterministic, an x value on the conditioning signal
shall not enable the timing check. For nondeterministic comparisons, an x on the conditioning signal shall
enable the timing check.
The conditioning signal shall be a scalar net; if a vector net or an expression resulting in a multibit value is
used, then the LSB of the vector net or the expression value is used.
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
886
Copyright © 2018 IEEE. All rights reserved.
If more than one conditioning signal is required for conditioning timing checks, appropriate logic shall be
combined in a separate signal outside the specify block, which can be used as the conditioning signal.
Example 1:To illustrate the difference between conditioned and unconditioned timing check events,
consider the following example with unconditioned timing check:
$setup( data, posedge clk, 10 );
Here, a setup timing check shall occur every time there is a positive edge on the signal clk.
To trigger the setup check on the positive edge on the signal clk only when the signal clr is high, rewrite
the command as
$setup( data, posedge clk &&& clr, 10 ) ;
Example 2: This example shows two ways to trigger the same timing check as in Example 1 (on the positive
clk edge) only when clr is low. The second method uses the === operator, which makes the comparison
deterministic.
$setup( data, posedge clk &&& (~clr), 10 ) ;
$setup( data, posedge clk &&& (clr===0), 10 );
Example 3: To perform the previous sample setup check on the positive clk edge only when clr and set
are high, add the following statement outside the specify block:
and new_gate( clr_and_set, clr, set );
Then add the condition to the timing check using the signal clr_and_set as follows:
$setup( data, posedge clk &&& clr_and_set, 10 );
### 31.8 Vector signals in timing checks

Either or both signals in a timing check can be a vector. This shall be interpreted as a single timing check
where the transition of one or more bits of a vector is considered a single transition of that vector.
For example:
module DFF (Q, CLK, DAT);
input CLK;
input [7:0] DAT;
output [7:0] Q;
always @(posedge clk)
Q = DAT;
specify
$setup (DAT, posedge CLK, 10);
endspecify
endmodule
If DAT transitions from 'b00101110 to 'b01010011 at time 100 and if CLK transitions from 0 to 1 at time
105, then the $setup timing check shall still only report a single timing violation.
Simulators may provide an option causing vectors in timing checks to result in the creation of multiple
single-bit timing checks. For timing checks with only a single signal, such as $period or $width, a vector
of width N results in N unique timing checks. For timing checks with two signals, such as $setup, $hold,
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
887
Copyright © 2018 IEEE. All rights reserved.
$setuphold, $skew, $timeskew, $fullskew, $recovery, $removal, $recrem, and $nochange,
where M and N are the widths of the signals, the result is M*N unique timing checks. If there is a notifier, all
the timing checks trigger that notifier.
With such an option enabled, the preceding example yields six timing violation because 6 bits of DAT
transitioned.
### 31.9 Negative timing checks

Both the $setuphold and $recrem timing checks can accept negative values when the negative timing
check option is enabled. The behavior of these two timing checks is identical with respect to negative values.
The descriptions in this subclause are for the $setuphold timing check, but apply equally to the $recrem
timing check.
The setup and hold timing check values define a timing violation window with respect to the reference
signal edge during which the data shall remain constant. Any change of the data during the specified
window causes a timing violation. The timing violation is reported, and through the notifier variable, other
actions can take place in the model, such as forcing the output of a flip-flop to X when it detects a timing
violation.
A positive value for both setup and hold times implies this violation window straddles the reference signal
shown in Figure 31-4.
A negative hold or setup time means the violation window is shifted to either before or after the reference
edge. This can happen in a real device because of disparate internal device delays between the internal clock
and data signal paths. These internal device delays are illustrated in Figure 31-5 showing how significant
differences in these delays can cause negative setup or hold values.
clock
data
..........Setup time (+)
..........Hold Time (+)
Figure 31-4—Data constraint interval, positive setup/hold
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
888
Copyright © 2018 IEEE. All rights reserved.
#### 31.9.1 Requirements for accurate simulation

In order to accurately model negative value timing checks, the following requirements apply:
a)
A timing violation shall be triggered if the signal changes in the violation window, exclusive of the
end points. Violation windows smaller than two units of simulation precision cannot yield timing
violations.
b)
The value of the latched data shall be the one that is stable during the violation window, again,
exclusive of the end points.
To facilitate these modeling requirements, delayed copies of the data and reference signals are generated in
the timing checks, and these are used internally for timing check evaluation at run time. The setup and hold
times used internally are adjusted to shift the violation window and make it overlap the reference signal.
Delayed data and reference signals can be declared within the timing check so they can be used in the
model’s functional implementation for more accurate simulation. If no delayed signals are declared in the
D1
D2
Seq.
Elem.
data
clock
output
ASIC Cell
clock
data
..........Setup time (+)
..........Hold Time (-)
Negative Setup time (D2>D1)
clock
data
..........Setup time (-)
..........Hold Time (+)
Negative Hold time (D1>D2)
Figure 31-5—Data constraint interval, negative setup/hold
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
889
Copyright © 2018 IEEE. All rights reserved.
timing check and if a negative setup or hold value is present, then implicit delayed signals are created.
Because implicit delayed signals cannot be used in defining model behavior, such a model can possibly
behave incorrectly.
Example 1:
$setuphold(posedge CLK, DATA, -10, 20);
Implicit delayed signals shall be created for CLK and DATA, but it shall not be possible to access them. The
$setuphold check shall be properly evaluated, but functional behavior shall not always be accurate. The
old DATA value shall be incorrectly clocked in if DATA transitions between posedge CLK and 10 time units
later.
Example 2:
$setuphold(posedge CLK, DATA1, -10, 20);
$setuphold(posedge CLK, DATA2, -15, 18);
Implicit delayed signals shall be created for CLK, DATA1, and DATA2, one for each. Even though CLK is
referenced in two different timing checks, only one implicit delayed signal is created, and it is used for both
timing checks.
Example 3: If a given signal has a delayed signal in some timing checks but not in others, the delayed signal
shall be used in both cases:
$setuphold(posedge CLK, DATA1, -10, 20,,,, del_CLK, del_DATA1);
$setuphold(posedge CLK, DATA2, -15, 18);
Explicit delayed signals of del_CLK and del_DATA1 are created for CLK and DATA1, while an implicit
delayed signal is created for DATA2. In other words, CLK has only one delayed signal created for it,
del_CLK, rather than one explicit delayed signal for the first check and another implicit delayed signal for
the second check.
The delayed versions of the signals, whether implicit or explicit, shall be used in the $setup, $hold,
$setuphold, $recovery, $removal, $recrem, $width, $period, and $nochange timing checks; and
these checks shall have their limits adjusted accordingly so that the notifier shall be toggled at the proper
moment. If the adjusted limit becomes less than or equal to 0, the limit shall be set to 0, and the simulator
shall issue a warning.
The delayed versions of the signals shall not be used for the $skew, $fullskew, and $timeskew timing
checks because it can possibly result in the reversal of the order of signal transitions. This causes the
notifiers for these timing checks to toggle at the wrong time relative to the rest of the model, perhaps
resulting in transitions to X due to a timing check violation being cancelled. This issue shall be addressed in
the model, possibly by using separate notifiers for these checks.
It is possible for a set of negative timing check values to be mutually inconsistent and produce no solution
for the delay values of delayed signals. In these situations, the simulator shall issue a warning. The
inconsistency shall be resolved by changing the smallest negative limit value to 0 and recalculating the
delays for the delayed signals, and this shall be repeated until a solution is reached. This procedure shall
always produce a solution because in the worst case all negative limit values become 0 and no delayed
signals are needed.
The delayed timing check signals are only actually delayed when negative limit values are present. If a
timing check signal becomes delayed by more than the propagation delay from that signal to an output, that
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
890
Copyright © 2018 IEEE. All rights reserved.
output shall take longer than its propagation delay to change. It shall instead transition at the same time that
the delayed timing check signal changes. Thus, the output shall behave as if its specify path delay were equal
to the delay applied to the timing check signal. This situation can only arise when unique setup/hold or
removal/recovery times are given for each edge of the data signal.
For example:
(CLK = Q) = 6;
$setuphold (posedge CLK, posedge D, -3,  8, , , , dCLK, dD);
$setuphold (posedge CLK, negedge D, -7, 13, , , , dCLK, dD);
The setup time of -7 (the larger in absolute value of -3 and -7) creates a delay of 7 for dCLK; therefore,
output Q shall not change until 7 time units after a positive edge on CLK, rather than the 6 time units given in
the specify path.
#### 31.9.2 Conditions in negative timing checks

Conditions can be associated with both the reference and data signals by using the &&& operator; but when
either the setup or hold time is negative, the conditions need to be paired with reference and data signals in a
more flexible way. This example illustrates why.
This pair of $setup and $hold checks works together to provide the same check as a single $setuphold:
$setup (data, clk &&& cond1, tsetup, ntfr);
$hold (clk, data &&& cond1, thold, ntfr);
clk is the timecheck event for the $setup check, while data is the timecheck event for the $hold check.
This cannot be represented in a single $setuphold check; therefore, additional arguments are provided to
make this possible. These arguments are timestamp_condition and timecheck_condition, and they
immediately follow the notifier (see 31.3.3). The following $setuphold check is equivalent to the separate
$setup and $hold checks shown above:
$setuphold( clk, data, tsetup, thold, ntfr, , cond1);
The timestamp_condition argument is null, while the timecheck_condition argument is cond1.
The timestamp_condition and timecheck_condition arguments are associated with either the reference or
data signals based on which delayed version of these signals occurs first. timestamp_condition is associated
with the delayed signal that transitions first, while timecheck_condition is associated with the delayed signal
that transitions second.
Delayed signals are only created for the reference and data signals and not for any condition signals
associated with them. Therefore, timestamp_condition and timecheck_condition are not implicitly delayed
by the simulator. Delayed condition signals for the timestamp_condition and timecheck_condition fields can
be created by making them a function of the delayed signals.
For example:
assign TE_cond_D = (dTE !== 1'b1);
assign TE_cond_TI = (dTE !== 1'b0);
assign DXTI_cond  = (dTI !==   dD);
specify
  $setuphold(posedge CP, D, -10,  20, notifier, ,TE_cond_D,  dCP, dD);
  $setuphold(posedge CP, TI, 20, -10, notifier, ,TE_cond_TI, dCP, dTI);
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
891
Copyright © 2018 IEEE. All rights reserved.
  $setuphold(posedge CP, TE, -4,   8, notifier, ,DXTI_cond,  dCP, dTE);
endspecify
The assign statements create condition signals that are functions of the delayed signals. Creating delayed
signal conditions synchronizes the conditions with the delayed versions of the reference and data signals
used to perform the checks.
The first $setuphold has a negative setup time; therefore, the timecheck condition TE_cond_D is
associated with data signal D. The second $setuphold has a negative hold time; therefore, the timecheck
condition TE_cond_TI is associated with reference signals CP. The third $setuphold has a negative setup
time; therefore, the timecheck condition DXTI_cond is associated with data signal TE.
The violation windows for the example are shown in Figure 31-6.
These are the delay values calculated for the delayed signals:
dCP     10.01
dD       0.00
dTI     20.02
dTE      2.02
Use of delayed signals in creating the signals for the timestamp_condition and timecheck_condition
arguments is not required, but it is usually closer to actual device behavior.
#### 31.9.3 Notifiers in negative timing checks

Because the reference and data signals are delayed internally, the detection of the timing violation is also
delayed. Notifier variables in negative timing checks shall be toggled when the timing check detects a
timing violation, which occurs when the delayed signals as measured by the adjusted timing check values
are in violation, not when the undelayed signals at the model inputs as measured by the original timing
check values are in violation.
#### 31.9.4 Option behavior

As already mentioned, the ability of simulators to handle negative values in $setuphold and $recrem
timing checks shall be enabled with an invocation option. It is possible models written to accept negative
timing check values with delayed reference and/or delayed data signals can be run without this invocation
option enabled. In this circumstance, the delayed reference and data signals become copies of the original
reference and data signals. The same occurs if an invocation option turning off all timing checks is used.
D
TE
TI
480
CP
508
490
520
510
500
504
Figure 31-6—Timing check violation windows
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
892
Copyright © 2018 IEEE. All rights reserved.
32. Backannotation using the standard delay format
### 32.1 General

This clause describes the following:
—
Standard delay format (SDF) annotator
—
Mapping SDF constructs to SystemVerilog
—
Multiple annotations
—
Multiple SDF files
—
Pulse limit annotation
—
SDF to SystemVerilog value mapping
—
The $sdf_annotate SDF file reader
### 32.2 Overview

SDF files contain timing values for specify path delays, specparam values, timing check constraints, and
interconnect delays. SDF files can also contain other information in addition to simulation timing, but these
need not concern SystemVerilog simulation. The timing values in SDF files usually come from application-
specific integrated circuit (ASIC) delay calculation tools that take advantage of connectivity, technology,
and layout geometry information.
SystemVerilog backannotation is the process by which timing values from the SDF file update specify path
delays, specparam values, timing constraint values, and interconnect delays.
All this information is covered further in IEEE Std 1497™-2001 [B1].
### 32.3 The SDF annotator

The term SDF annotator refers to any tool capable of backannotating SDF data to a SystemVerilog
simulator. It shall issue a warning for any data it is unable to annotate.
An SDF file can contain many constructs that are not related to specify path delays, specparam values,
timing check constraint values, or interconnect delays. An example is any construct in the TIMINGENV
section of the SDF file. All constructs unrelated to SystemVerilog timing shall be ignored without any
warnings issued.
Any SystemVerilog timing value for which the SDF file does not provide a value shall not be modified
during the backannotation process, and its prebackannotation value shall be unchanged.
### 32.4 Mapping of SDF constructs to SystemVerilog

SDF timing values appear within a CELL declaration, which can contain one or more of DELAY,
TIMINGCHECK, and LABEL sections. The DELAY section contains propagation delay values for specify paths
and interconnect delays. The TIMINGCHECK section contains timing check constraint values. The LABEL
section contains new values for specparams. Backannotation into SystemVerilog is done by matching SDF
constructs to the corresponding SystemVerilog declarations and then replacing the existing SystemVerilog
timing values with those from the SDF file.
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
893
Copyright © 2018 IEEE. All rights reserved.
#### 32.4.1 Mapping of SDF delay constructs to SystemVerilog declarations

When annotating DELAY constructs that are not interconnect delays (covered in 32.4.4), the SDF annotator
looks for specify paths where the names and conditions match. When annotating TIMINGCHECK constructs,
the SDF annotator looks for timing checks of the same type where the names and conditions match.
Table 32-1 shows which SystemVerilog structures can be annotated by each SDF construct in the DELAY
section.
In the following example, the source SDF signal sel matches the source SystemVerilog signal, and the
destination SDF signal zout also matches the destination SystemVerilog signal. Therefore, the rise/fall
times of 1.3 and 1.7 are annotated to the specify path.
SDF file:
(IOPATH sel zout (1.3) (1.7))
SystemVerilog specify path:
(sel => zout) = 0;
A conditional IOPATH delay between two ports shall annotate only to SystemVerilog specify paths between
those same two ports with the same condition. In the following example, the rise/fall times of 1.3 and 1.7
are annotated only to the second specify path:
SDF file:
(COND mode (IOPATH sel zout (1.3) (1.7)))
Table 32-1—Mapping of SDF delay constructs to SystemVerilog declarations
SDF construct
SystemVerilog annotated structure
(PATHPULSE...
Conditional and nonconditional specify path pulse limits
(PATHPULSEPERCENT...
Conditional and nonconditional specify path pulse limits
(IOPATH...
Conditional and nonconditional specify path delays/pulse limits
(IOPATH (RETAIN...
Conditional and nonconditional specify path delays/pulse limits,
RETAIN may be ignored
(COND (IOPATH...
Conditional specify path delays/pulse limits
(COND (IOPATH (RETAIN...
Conditional specify path delays/pulse limits, RETAIN may be ignored
(CONDELSE (IOPATH...
ifnone
(CONDELSE (IOPATH (RETAIN...
ifnone, RETAIN may be ignored
(DEVICE...
All specify paths to module outputs. If no specify paths, all primitives
driving module outputs.
(DEVICE port_instance...
If port_instance is a module instance, all specify paths to module
outputs. If no specify paths, all primitives driving module outputs. If
port_instance is a module instance output, all specify paths to that
module output. If no specify path, all primitives driving that module
output.
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
894
Copyright © 2018 IEEE. All rights reserved.
SystemVerilog specify paths:
if (!mode) (sel => zout) = 0;
if (mode) (sel => zout) = 0;
A nonconditional IOPATH delay between two ports shall annotate to all SystemVerilog specify paths
between those same two ports. In the following example, the rise/fall times of 1.3 and 1.7 are annotated
to both specify paths:
SDF file:
(IOPATH sel zout (1.3) (1.7))
SystemVerilog specify paths:
if (!mode) (sel => zout) = 0;
if (mode) (sel => zout) = 0;
#### 32.4.2 Mapping of SDF timing check constructs to SystemVerilog

Table 32-2 shows which SystemVerilog timing checks are annotated to by each type of SDF timing check.
v1 is the first value of a timing check, v2 is the second value, while x indicates no value is annotated.
The reference and data signals of timing checks can have logical condition expressions and edges associated
with them. An SDF timing check with no conditions or edges on any of its signals shall match all
corresponding SystemVerilog timing checks regardless of whether conditions are present. In the following
example, the SDF timing check shall annotate to all the SystemVerilog timing checks:
SDF file:
(SETUPHOLD data clk (3) (4))
Table 32-2—Mapping of SDF timing check constructs to SystemVerilog
SDF timing check
Annotated SystemVerilog timing checks
(SETUP v1...
$setup(v1), $setuphold(v1,x)
(HOLD v1...
$hold(v1), $setuphold(x,v1)
(SETUPHOLD v1 v2...
$setup(v1), $hold(v2), $setuphold(v1,v2)
(RECOVERY v1...
$recovery(v1), $recrem(v1,x)
(REMOVAL v1...
$removal(v1), $recrem(x,v1)
(RECREM v1 v2...
$recovery(v1), $removal(v2), $recrem(v1,v2)
(SKEW v1...
$skew(v1), $timeskew(v1)
(BIDIRECTSKEW v1 v2...
$fullskew(v1,v2)
(WIDTH v1...
$width(v1,x)
(PERIOD v1...
$period(v1)
(NOCHANGE v1 v2...
$nochange(v1,v2)
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
895
Copyright © 2018 IEEE. All rights reserved.
SystemVerilog timing checks:
$setuphold (posedge clk &&&
mode, data, 1, 1, ntfr);
$setuphold (negedge clk &&& !mode, data, 1, 1, ntfr);
$setuphold (edge clk, data, 1, 1, ntfr);
When conditions and/or edges are associated with the signals in an SDF timing check, then they shall match
those in any corresponding SystemVerilog timing check before annotation shall happen. In the following
example, the SDF timing check shall annotate to the first SystemVerilog timing check, but not the second
and the third:
SDF file:
(SETUPHOLD data (posedge clk) (3) (4))
SystemVerilog timing checks:
$setuphold (posedge clk &&&
mode, data, 1, 1, ntfr); // Annotated
$setuphold (negedge clk &&& !mode, data, 1, 1, ntfr); // Not annotated
$setuphold (edge clk, data, 1, 1, ntfr);
// Not annotated
Here, the SDF timing check shall not annotate to any of the SystemVerilog timing checks.
SDF file:
(SETUPHOLD data (COND !mode (posedge clk)) (3) (4))
SystemVerilog timing checks:
$setuphold (posedge clk &&&
mode, data, 1, 1, ntfr); // Not annotated
$setuphold (negedge clk &&& !mode, data, 1, 1, ntfr); // Not annotated
$setuphold (edge clk, data, 1, 1, ntfr);
// Not annotated
#### 32.4.3 SDF annotation of specparams

The SDF LABEL construct annotates to specparams. Any expression containing one or more specparams is
reevaluated when annotated to from an SDF file.
The following example shows SDF LABEL constructs annotating to specparams in a SystemVerilog module.
The specparams are used in procedural delays to control when the clock transitions. The SDF LABEL
construct annotates the values of dhigh and dlow, thereby setting the period and duty cycle of the clock.
SDF file:
(LABEL
(ABSOLUTE
(dhigh 60)
(dlow 40)))
SystemVerilog file:
module clock(clk);
output clk;
reg clk;
specparam dhigh=0, dlow=0;
initial clk = 0;
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
896
Copyright © 2018 IEEE. All rights reserved.
always
begin
#dhigh clk = 1;
// Clock remains low for time dlow
// before transitioning to 1
#dlow  clk = 0;
// Clock remains high for time dhigh
// before transitioning to 0
end
endmodule
The following example shows a specparam in an expression of a specify path. The SDF LABEL construct can
be used to change the value of the specparam and cause reevaluation of the expression.
specparam cap = 0;
...
specify
(A => Z) = 1.4 * cap + 0.7;
endspecify
#### 32.4.4 SDF annotation of interconnect delays

SDF interconnect delay annotation differs from annotation of other constructs previously described in that
there exists no corresponding SystemVerilog declaration to which to annotate. In SystemVerilog simulation,
interconnect delays are an abstraction that represents the signal propagation delay from an output or inout
module port to an input or inout module port. The INTERCONNECT construct includes a source, a load, and
delay values, while the PORT and NETDELAY constructs include only a load and delay values. Interconnect
delays can only be annotated between module ports, never between primitive pins. Table 32-3 shows how
the SDF interconnect constructs in the DELAY section are annotated.
Interconnect delays can be annotated to both single source and multisource nets.
When annotating a PORT construct, the SDF annotator shall search for the port and, if it exists, shall annotate
an interconnect delay to that port that shall represent the delay from all sources on the net to that port.
When annotating a NETDELAY construct, the SDF annotator shall check to see if it is annotating to a port or a
net. If it is a port, then the SDF annotator shall annotate an interconnect delay to that port. If it is a net, then
it shall annotate an interconnect delay to all load ports connected to that net. If the port or net has more than
one source, then the delay shall represent the delay from all sources. NETDELAY delays can only be
annotated to input or inout module ports or to nets.
In the case of multisource nets, unique delays can be annotated between each source/load pair using the
INTERCONNECT construct. When annotating this construct, the SDF annotator shall find the source port and
the load port; and if both exist, it shall annotate an interconnect delay between the two. If the source port is
not found or if the source port and the load port are not actually on the same net, then a warning is issued,
but the delay to the load port is annotated anyway. If this happens for a load port that is part of a multisource
Table 32-3—SDF annotation of interconnect delays
SDF construct
SystemVerilog annotated structure
(PORT...
Interconnect delay
(NETDELAY a
aOnly OVI SDF version 1.0, 2.0, and 2.1 and IEEE SDF version 4.0.
Interconnect delay
(INTERCONNECT...
Interconnect delay
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
897
Copyright © 2018 IEEE. All rights reserved.
net, then the delay is treated as if it were the delay from all source ports, which is the same as the annotation
behavior for a PORT delay. Source ports shall be output or inout ports, while load ports shall be input or inout
ports.
Interconnect delays share many of the characteristics of specify path delays. The same rules for specify path
delays for filling in missing delays and pulse limits also apply for interconnect delays. Interconnect delays
have 12 transition delays, and unique reject and error pulse limits are associated with each of the 12. An
unlimited number of future schedules are permitted.
In a SystemVerilog module, a reference to an annotated port, wherever it occurs, whether in $monitor and
$display statements or in expressions, shall provide the delayed signal value. A reference to the source
shall yield the undelayed signal value, while a reference to the load shall yield the delayed signal value. In
general, references to the signal value hierarchically before the load shall yield the undelayed signal value,
while references to the signal at or hierarchically after the load shall yield the delayed signal value. An
annotation to a hierarchical port shall affect all connected ports at higher or lower hierarchical levels,
depending on the direction of annotation. An annotation from a source port shall be interpreted as being
from all sources hierarchically higher or lower than that source port.
Up-hierarchy annotations shall be properly handled. This situation arises when the load is hierarchically
above the source. The delay to all ports that are hierarchically above the load or that connect to the net at
points hierarchically above the load is the same as the delay to that load.
Down-hierarchy annotation shall also be properly handled. This situation arises when the source is
hierarchically above the load. The delay to the load is interpreted as being from all ports that are at or above
the source or that connect to the net at points hierarchically above the source.
Hierarchically overlapping annotations are permitted. This occurs when annotations to or from the same port
take place at different hierarchical levels and, therefore, do not correspond to the same hierarchical subset of
ports. In the following example, the first INTERCONNECT statement annotates to all ports of the net that are
at or hierarchically within i53/selmode, while the second annotates to a smaller subset of ports, only those
at or hierarchically within i53/u21/in:
(INTERCONNECT i14/u5/out i53/selmode (1.43) (2.17))
(INTERCONNECT i14/u5/out i53/u21/in  (1.58) (1.92))
Overlapping annotations can occur in many different ways, particularly on multisource/multiload nets, and
SDF annotation shall properly resolve all the interactions.
### 32.5 Multiple annotations

SDF annotation is an ordered process. The constructs from the SDF file are annotated in their order of
occurrence. In other words, annotation of an SDF construct can be changed by annotation of a subsequent
construct that either modifies (INCREMENT) or overwrites (ABSOLUTE) it. These do not have to be the
same construct. The following example first annotates pulse limits to an IOPATH and then annotates the
entire IOPATH, thereby overwriting the pulse limits that were just annotated:
(DELAY
(ABSOLUTE
(PATHPULSE A Z (2.1) (3.4))
(IOPATH A Z (3.5) (6.1))
Overwriting the pulse limits can be avoided by using empty parentheses to hold the current values of the
pulse limits:
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
898
Copyright © 2018 IEEE. All rights reserved.
(DELAY
(ABSOLUTE
(PATHPULSE A Z (2.1) (3.4))
(IOPATH A Z ((3.5) () ()) ((6.1) () ()) )
The preceding annotation can be simplified into a single statement as follows:
(DELAY
(ABSOLUTE
(IOPATH A Z ((3.5) (2.1) (3.4)) ((6.1) (2.1) (3.4)) )
A PORT annotation followed by an INTERCONNECT annotation to the same load shall cause only the delay
from the INTERCONNECT source to be affected. For the following net with three sources and a single load,
the delay from all sources except i13/out remains 6:
(DELAY
(ABSOLUTE
(PORT i15/in (6))
(INTERCONNECT i13/out i15/in (5))
An INTERCONNECT annotation followed by a PORT annotation shall cause the INTERCONNECT annotation to
be overwritten. Here, the delays from all sources to the load shall become 6:
(DELAY
(ABSOLUTE
(INTERCONNECT i13/out i15/in (5))
(PORT i15/in (6))
### 32.6 Multiple SDF files

More than one SDF file can be annotated. Each call to the $sdf_annotate task annotates the design with
timing information from an SDF file. Annotated values either modify (INCREMENT) or overwrite
(ABSOLUTE) values from earlier SDF files. Different regions of a design can be annotated from different
SDF files by specifying the region’s hierarchy scope as the second argument to $sdf_annotate.
### 32.7 Pulse limit annotation

For SDF annotation of delays (not timing constraints), the default values annotated for pulse limits shall be
calculated using the percentage settings for the reject and error limits. By default, these limits are 100%, but
they can be modified through invocation options. For example, assuming invocation options have set the
reject limit to 40% and the error limit to 80%, the following SDF construct shall annotate a delay of 5, a reject
limit of 2, and an error limit of 4:
(DELAY
(ABSOLUTE
(IOPATH A Z (5))
Given that the specify path delay was originally 0, the following annotation results in a delay of 5 and pulse
limits of 0:
(DELAY
(ABSOLUTE
(IOPATH A Z ((5) () ()) )
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
899
Copyright © 2018 IEEE. All rights reserved.
Annotations in INCREMENT mode can result in pulse limits less than 0, in which case they shall be adjusted
to 0. For example, if the specify path pulse limits were both 3, the following annotation results in a 0 value
for both pulse limits:
(DELAY
(INCREMENT
(IOPATH A Z (() (-4) (-5)) )
There are two SDF constructs that annotate only to pulse limits, PATHPULSE and PATHPULSEPERCENT.
They do not affect the delay. When PATHPULSE sets the pulse limits to values greater than the delay,
SystemVerilog shall exhibit the same behavior as if the pulse limits had been set equal to the delay.
### 32.8 SDF to SystemVerilog delay value mapping

SystemVerilog specify paths and interconnects can have unique delays for up to 12 state transitions (see
30.5.1). All other constructs, such as gate primitives and continuous assignments, can have only three state
transition delays (see 28.16).
For SystemVerilog specify path and interconnect delays, the number of transition delay values provided by
SDF might be less than 12.
Table 32-4 shows how fewer than 12 SDF delays are extended to be 12 delays. The SystemVerilog
transition types are shown down the left-hand side, while the number of SDF delays provided is shown
across the top. The SDF values are given the names v1 through v12.
For other delays that can have at most three values, the expansion of less than three SDF delays into three
SystemVerilog delays is covered in Table 28-9. More than three SDF delays are reduced to three
SystemVerilog delays by simply ignoring the extra delays. The delay to the X-state is created from the
minimum of the other three delays.
Table 32-4—SDF to SystemVerilog delay value mapping
SystemVerilog
transition
Number of SDF delay values provided
## 1 value

## 2 values

## 3 values

## 6 values

## 12 values

## 0 –> 1

 v1
 v1
v1
v1
v1
## 1 –> 0

 v1
v2
v2
 v2
 v2
## 0 –> z

v1
v1
v3
v3
v3
z –> 1
v1
v1
v1
v4
v4
## 1 –> z

v1
v2
v3
v5
v5
z –> 0
v1
v2
v2
 v6
 v6
## 0 –> x

v1
v1
min(v1,v3)
min(v1,v3)
v7
x –> 1
 v1
 v1
 v1
max(v1,v4)
v8
## 1 –> x

v1
v2
min(v2,v3)
min(v2,v5)
v9
x –> 0
v1
v2
v2
max(v2,v6)
v10
x –> z
v1
max(v1,v2)
v3
max(v3,v5)
v11
z –> x
v1
min(v1,v2)
min(v1,v2)
min(v4,v6)
v12
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
900
Copyright © 2018 IEEE. All rights reserved.
### 32.9 Loading timing data from an SDF file

The syntax for the $sdf_annotate system task is shown in Syntax 32-1.
```ebnf
sdf_annotate_task ::=
```

$sdf_annotate ( sdf_file [ , [ module_instance ] [ , [ "config_file" ]
[ , [ "log_file" ] [ , [ "mtm_spec" ] [ , [ "scale_factors" ] [ , [ "scale_type" ] ] ] ] ] ] ] ) ;
Syntax 32-1—Syntax for $sdf_annotate system task (not in Annex A)
The $sdf_annotate system task reads timing data from an SDF file into a specified region of the design.
sdf_file—is an expression that is a string literal, string data type, or an integral data type containing a
character string that names the file to be opened.
module_instance—is an optional argument specifying the scope to which to annotate the information in the
SDF file. The SDF annotator uses the hierarchy level of the specified instance for running the annotation.
Array indices are permitted. If the module_instance is not specified, the SDF annotator uses the module
containing the call to the $sdf_annotate system task as the module_instance for annotation.
config_file—is an optional character string argument providing the name of a configuration file. Information
in this file can be used to provide detailed control over many aspects of annotation.
log_file—is an optional character string argument providing the name of the log file used during SDF
annotation. Each individual annotation of timing data from the SDF file results in an entry in the log file.
mtm_spec—is an optional character string argument specifying which member of the min/typ/max triples
shall be annotated. The legal values for this string are described in Table 32-5. This overrides any MTM_SPEC
keywords in the configuration file.
scale_factors—is an optional character string argument specifying the scale factors to be used while
annotating timing values. For example, "1.6:1.4:1.2" causes minimum values to be multiplied by 1.6,
typical values by 1.4, and maximum values by 1.2. The default values are 1.0:1.0:1.0. The
scale_factors argument overrides any SCALE_FACTORS keywords in the configuration file.
scale_type—is an optional character string argument specifying how the scale factors should be applied to
the min/typ/max triples. The legal values for this string are shown in Table 32-6. This overrides any
SCALE_TYPE keywords in the configuration file.
Table 32-5—mtm_spec argument
Keyword
Description
MAXIMUM
Annotates the maximum value
MINIMUM
Annotates the minimum value
TOOL_CONTROL (default)
Annotates the value as selected by the simulator
TYPICAL
Annotates the typical value
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
901
Copyright © 2018 IEEE. All rights reserved.
Table 32-6—scale_type argument
Keyword
Description
FROM_MAXIMUM
Applies scale factors to maximum value
FROM_MINIMUM
Applies scale factors to minimum value
FROM_MTM (default)
Applies scale factors to min/typ/max values
FROM_TYPICAL
Applies scale factors to typical value
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
