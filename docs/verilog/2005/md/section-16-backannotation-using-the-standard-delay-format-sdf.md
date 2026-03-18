---
title: "Section 16: Backannotation using the standard delay format (SDF)"
document: "Verilog Hardware Description Language Reference Manual"
standard: "IEEE 1364-2005"
domain: "Verilog"
section: "16"
source_txt: "section-16-backannotation-using-the-standard-delay-format-sdf.txt"
source_pdf: "/Users/richarddje/Documents/github/Verilog-LRM-IEEE-1364-2005.pdf"
---

# Section 16: Backannotation using the standard delay format (SDF)

IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
269
## 16. Backannotation using the standard delay format (SDF)

SDF files contain timing values for specify path delays, specparam values, timing check constraints, and
interconnect delays. SDF files can also contain other information in addition to simulation timing, but these
need not concern Verilog simulation. The timing values in SDF files usually come from application-specific
integrated circuit (ASIC) delay calculation tools that take advantage of connectivity, technology, and layout
geometry information.
Verilog backannotation is the process by which timing values from the SDF file update specify path delays,
specparam values, timing constraint values, and interconnect delays.
All this information is covered further in IEEE Std 1497™-2001 [B1]9.
### 16.1 The SDF annotator

The term SDF annotator refers to any tool capable of backannotating SDF data to a Verilog simulator. It
shall report a warning for any data it is unable to annotate.
An SDF file can contain many constructs that are not related to specify path delays, specparam values,
timing check constraint values, or interconnect delays. An example is any construct in the TIMINGENV
section of the SDF file. All constructs unrelated to Verilog timing shall be ignored without any warnings
issued.
Any Verilog timing value for which the SDF file does not provide a value shall not be modified during the
backannotation process, and its prebackannotation value shall be unchanged.
### 16.2 Mapping of SDF constructs to Verilog

SDF timing values appear within a CELL declaration, which can contain one or more of DELAY,
TIMINGCHECK, and LABEL sections. The DELAY section contains propagation delay values for specify paths
and interconnect delays. The TIMINGCHECK section contains timing check constraint values. The LABEL
section contains new values for specparams. Backannotation into Verilog is done by matching SDF
constructs to the corresponding Verilog declarations and then replacing the existing Verilog timing values
with those from the SDF file.
#### 16.2.1 Mapping of SDF delay constructs to Verilog declarations

When annotating DELAY constructs that are not interconnect delays (covered in 16.2.3), the SDF annotator
looks for specify paths where the names and conditions match. When annotating TIMINGCHECK constructs,
the SDF annotator looks for timing checks of the same type where the names and conditions match.
Table 16-1 shows which Verilog structures can be annotated by each SDF construct in the DELAY section.
9The numbers in brackets correspond to those of the bibliography in Annex I.
Table 16-1—Mapping of SDF delay constructs to Verilog declarations
SDF construct
Verilog annotated structure
(PATHPULSE...
Conditional and nonconditional specify path pulse limits
(PATHPULSEPERCENT...
Conditional and nonconditional specify path pulse limits
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
270
Copyright © 2006 IEEE. All rights reserved.
In the following example, the source SDF signal sel matches the source Verilog signal, and the destination
SDF signal zout also matches the destination Verilog signal. Therefore, the rise/fall times of 1.3 and 1.7
are annotated to the specify path.
SDF file:
(IOPATH sel zout (1.3) (1.7))
Verilog specify path:
(sel => zout) = 0;
A conditional IOPATH delay between two ports shall annotate only to Verilog specify paths between those
same two ports with the same condition. In the following example, the rise/fall times of 1.3 and 1.7 are
annotated only to the second specify path:
SDF file:
(COND mode (IOPATH sel zout (1.3) (1.7)))
Verilog specify paths:
if (!mode) (sel => zout) = 0;
if (mode) (sel => zout) = 0;
A nonconditional IOPATH delay between two ports shall annotate to all Verilog specify paths between those
same two ports. In the following example, the rise/fall times of 1.3 and 1.7 are annotated to both specify
paths:
SDF file:
(IOPATH sel zout (1.3) (1.7))
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
If port_instance is a module instance, all specify paths to module out-
puts. If no specify paths, all primitives driving module outputs. If
port_instance is a module instance output, all specify paths to that mod-
ule output. If no specify path, all primitives driving that module output.
Table 16-1—Mapping of SDF delay constructs to Verilog declarations  (continued)
SDF construct
Verilog annotated structure
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
271
Verilog specify paths:
if (!mode) (sel => zout) = 0;
if (mode) (sel => zout) = 0;
#### 16.2.2 Mapping of SDF timing check constructs to Verilog

Table 16-2 shows which Verilog timing checks are annotated to by each type of SDF timing check. v1 is the
first value of a timing check, v2 is the second value, while x indicates no value is annotated.
The reference and data signals of timing checks can have logical condition expressions and edges associated
with them. An SDF timing check with no conditions or edges on any of its signals shall match all
corresponding Verilog timing checks regardless of whether conditions are present. In the following
example, the SDF timing check shall annotate to all the Verilog timing checks:
SDF file:
(SETUPHOLD data clk (3) (4))
Verilog timing checks:
$setuphold (posedge clk &&&  mode, data, 1, 1, ntfr);
$setuphold (negedge clk &&& !mode, data, 1, 1, ntfr);
When conditions and/or edges are associated with the signals in an SDF timing check, then they shall match
those in any corresponding Verilog timing check before annotation shall happen. In the following example,
the SDF timing check shall annotate to the first Verilog timing check, but not the second:
Table 16-2—Mapping of SDF timing check constructs to Verilog
SDF timing check
Annotated Verilog timing checks
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
$skew(v1)
(TIMESKEW v1...a
aNot part of current SDF standard
$timeskew(v1)
(FULLSKEW v1 v2...a
$fullskew(v1,v2)
(WIDTH v1...
$width(v1,x)
(PERIOD v1...
$period(v1)
(NOCHANGE v1 v2...
$nochange(v1,v2)
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
272
Copyright © 2006 IEEE. All rights reserved.
SDF file:
(SETUPHOLD data (posedge clk) (3) (4))
Verilog timing checks:
$setuphold (posedge clk &&&  mode, data, 1, 1, ntfr); // Annotated
$setuphold (negedge clk &&& !mode, data, 1, 1, ntfr); // Not annotated
Here, the SDF timing check shall not annotate to any of the Verilog timing checks:
SDF file:
(SETUPHOLD data (COND !mode (posedge clk)) (3) (4))
Verilog timing checks:
$setuphold (posedge clk &&&  mode, data, 1, 1, ntfr); // Not annotated
$setuphold (negedge clk &&& !mode, data, 1, 1, ntfr); // Not annotated
#### 16.2.3 SDF annotation of specparams

The SDF LABEL construct annotates to specparams. Any expression containing one or more specparams is
reevaluated when annotated to from an SDF file.
The following example shows SDF LABEL constructs annotating to specparams in a Verilog module. The
specparams are used in procedural delays to control when the clock transitions. The SDF LABEL construct
annotates the values of dhigh and dlow, thereby setting the period and duty cycle of the clock.
SDF file:
(LABEL
(ABSOLUTE
(dhigh 60)
(dlow 40)))
Verilog file:
module clock(clk);
output clk;
reg clk;
specparam dhigh=0, dlow=0;
initial clk = 0;
always
begin
#dhigh clk = 1;
// Clock remains low for time dlow
// before transitioning to 1
#dlow  clk = 0;
// Clock remains high for time dhigh
// before transitioning to 0
end;
endmodule
The following example shows a specparam in an expression of a specify path. The SDF LABEL construct can
be used to change the value of the specparam and cause reevaluation of the expression.
specparam cap = 0;
...
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
273
specify
(A => Z) = 1.4 * cap + 0.7;
endspecify
#### 16.2.4 SDF annotation of interconnect delays

SDF interconnect delay annotation differs from annotation of other constructs described above in that there
exists no corresponding Verilog declaration to which to annotate. In Verilog simulation, interconnect delays
are an abstraction that represents the signal propagation delay from an output or inout module port to an
input or inout module port. The INTERCONNECT construct includes a source, a load, and delay values, while
the PORT and NETDELAY constructs include only a load and delay values. Interconnect delays can only be
annotated between module ports, never between primitive pins. Table 16-3 shows how the SDF interconnect
constructs in the DELAY section are annotated.
Interconnect delays can be annotated to both single source and multisource nets.
When annotating a PORT construct, the SDF annotator shall search for the port and. if it exists, shall annotate
an interconnect delay to that port that shall represent the delay from all sources on the net to that port.
When annotating a NETDELAY construct, the SDF annotator shall check to see if it is annotating to a port or a
net. If it is a port, then the SDF annotator shall annotate an interconnect delay to that port. If it is a net, then
it shall annotate an interconnect delay to all load ports connected to that net. If the port or net has more than
one source, then the delay shall represent the delay from all sources. NETDELAY delays can only be
annotated to input or inout module ports or to nets.
In the case of multisource nets, unique delays can be annotated between each source/load pair using the
INTERCONNECT construct. When annotating this construct, the SDF annotator shall find the source port and
the load port; and if both exist, it shall annotate an interconnect delay between the two. If the source port is
not found or if the source port and the load port are not actually on the same net, then a warning message is
issued, but the delay to the load port is annotated anyway. If this happens for a load port that is part of a
multisource net, then the delay is treated as if it were the delay from all source ports, which is the same as the
annotation behavior for a PORT delay. Source ports shall be output or inout ports, while load ports shall be
input or inout ports.
Interconnect delays share many of the characteristics of specify path delays. The same rules for specify path
delays for filling in missing delays and pulse limits also apply for interconnect delays. Interconnect delays
have twelve transition delays, and unique reject and error pulse limits are associated with each of the twelve.
An unlimited number of future schedules are permitted.
In a Verilog module, a reference to an annotated port, wherever it occurs, whether in $monitor and
$display statements or in expressions, shall provide the delayed signal value. A reference to the source
Table 16-3—SDF annotation of interconnect delays
SDF construct
Verilog annotated structure
(PORT...
Interconnect delay
(NETDELAY a
aOnly OVI SDF version 1.0, 2.0, and 2.1 and IEEE SDF version 4.0
Interconnect delay
(INTERCONNECT...
Interconnect delay
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
274
Copyright © 2006 IEEE. All rights reserved.
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
### 16.3 Multiple annotations

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
(DELAY
(ABSOLUTE
(PATHPULSE A Z (2.1) (3.4))
(IOPATH A Z ((3.5) () ()) ((6.1) () ()) )
The above annotation can be simplified into a single statement like this:
(DELAY
(ABSOLUTE
(IOPATH A Z ((3.5) (2.1) (3.4)) ((6.1) (2.1) (3.4)) )
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
275
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
### 16.4 Multiple SDF files

More than one SDF file can be annotated. Each call to the $sdf_annotate task annotates the design with
timing information from an SDF file. Annotated values either modify (INCREMENT) or overwrite
(ABSOLUTE) values from earlier SDF files. Different regions of a design can be annotated from different
SDF files by specifying the region’s hierarchy scope as the second argument to $sdf_annotate.
### 16.5 Pulse limit annotation

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
Annotations in INCREMENT mode can result in pulse limits less than 0, in which case they shall be adjusted
to 0. For example, if the specify path pulse limits were both 3, the following annotation results in a 0 value
for both pulse limits:
(DELAY
(INCREMENT
(IOPATH A Z (() (-4) (-5)) )
There are two SDF constructs that annotate only to pulse limits, PATHPULSE and PATHPULSEPERCENT.
They do not affect the delay. When PATHPULSE sets the pulse limits to values greater than the delay, Verilog
shall exhibit the same behavior as if the pulse limits had been set equal to the delay.
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
276
Copyright © 2006 IEEE. All rights reserved.
### 16.6 SDF to Verilog delay value mapping

Verilog specify paths and interconnects can have unique delays for up to twelve state transitions (see
14.3.1). All other constructs, such as gate primitives and continuous assignments, can have only three state
transition delays (see 7.14).
For Verilog specify path and interconnect delays, the number of transition delay values provided by SDF
might be less than twelve.
Table 16-4 shows how fewer than twelve SDF delays are extended to be twelve delays. The Verilog
transition types are shown down the left-hand side, while the number of SDF delays provided is shown
across the top. The SDF values are given the names v1 through v12.
For other delays that can have at most three values, the expansion of less than three SDF delays into three
Verilog delays is covered in Table 7-9. More than three SDF delays are reduced to three Verilog delays by
simply ignoring the extra delays. The delay to the X-state is created from the minimum of the other three
delays.
Table 16-4—SDF to Verilog delay value mapping
Verilog transition
Number of SDF delay values provided
## 1 value

## 2 values

## 3 values

## 6 values

## 12 values

## 0 -> 1

 v1
 v1
v1
v1
v1
## 1 -> 0

 v1
v2
v2
 v2
 v2
## 0 -> z

v1
v1
v3
v3
v3
z -> 1
v1
v1
v1
v4
v4
## 1 -> z

v1
v2
v3
v5
v5
z -> 0
v1
v2
v2
 v6
 v6
## 0 -> x

v1
v1
min(v1,v3)
min(v1,v3)
v7
x -> 1
 v1
 v1
 v1
max(v1,v4)
v8
## 1 -> x

v1
v2
min(v2,v3)
min(v2,v5)
v9
x -> 0
v1
v2
v2
max(v2,v6)
v10
x -> z
v1
max(v1,v2)
v3
max(v3,v5)
v11
z -> x
v1
min(v1,v2)
min(v1,v2)
min(v4,v6)
v12
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
