---
title: "Section Annex.C: informative system tasks and functions"
document: "Verilog Hardware Description Language Reference Manual"
standard: "IEEE 1364-2005"
domain: "Verilog"
section: "Annex.C"
source_txt: "section-Annex_C-informative-system-tasks-and-functions.txt"
source_pdf: "/Users/richarddje/Documents/github/Verilog-LRM-IEEE-1364-2005.pdf"
---

# Section Annex.C: informative system tasks and functions

IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
511
Annex C
(informative)
System tasks and functions
The system tasks and functions described in this annex are for informative purposes only and are not part of
this standard.
This annex describes system tasks and functions as companions to the system tasks and functions described
in Clause 17 The system tasks and functions described in this annex may not be available in all
implementations of the Verilog HDL. The following system tasks and functions are described in this annex:
The word tool in this annex refers to an implementation of Verilog HDL, typically a logic simulator.
C.1 $countdrivers
Syntax:
$countdrivers (net, [ net_is_forced, number_of_01x_drivers, number_of_0_drivers,
                       number_of_1_drivers, number_of_x_drivers ] );
The $countdrivers system function is provided to count the number of drivers on a specified net so that bus
contention can be identified.
This system function returns a 0 if there is no more than one driver on the net and returns a 1 otherwise
(indicating contention). The specified net shall be a scalar or a bit-select of a vector net. The number of
arguments to the system function may vary according to how much information is desired.
If additional arguments are supplied to the $countdrivers function, each argument returns the information
described in Table C.1.
$countdrivers
[C.1]
$getpattern
[C.2]
$incsave
[C.8]
$input
[C.3]
$key
[C.4]
$list
[C.5]
$log
[C.6]
$nokey
[C.4]
$nolog
[C.6]
$reset
[C.7]
$reset_count
[C.7]
$reset_value
[C.7]
$restart
[C.8]
$save
[C.8]
$scale
[C.9]
$scope
[C.10]
$showscopes
[C.11]
$showvars
[C.12]
$sreadmemb
[C.13]
$sreadmemh
[C.13]
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
512
Copyright © 2006 IEEE. All rights reserved.
C.2 $getpattern
Syntax:
$getpattern ( mem_element );
The system function $getpattern provides for fast processing of stimulus patterns that have to be
propagated to a large number of scalar inputs. The function reads stimulus patterns that have been loaded
into a memory using the $readmemb or $readmemh system tasks.
Use of this function is limited, however: it may only be used in a continuous assignment statement where the
left-hand side is a concatenation of scalar nets and the argument to the system function is a memory element
reference.
For example:
The following example shows how stimuli stored in a file can be read into a memory using $readmemb and
applied to the circuit one pattern at a time using $getpattern.
The memory in_mem is initialized with the stimulus patterns by the $readmemb task. The integer variable
index selects which pattern is being applied to the circuit. The for loop increments the integer variable
index periodically to sequence the patterns.
module top;
parameter in_width=10,
         patterns=200,
         step=20;
reg [1:in_width] in_mem[1:patterns];
integer index;
// declare scalar inputs
wire i1,i2,i3,i4,i5,i6,i7,i8,i9,i10;
// assign patterns to circuit scalar inputs (a new pattern
// is applied to the circuit each time index changes value)
assign {i1,i2,i3,i4,i5,i6,i7,i8,i9,i10} = $getpattern(in_mem[index]);
initial begin
// read stimulus patterns into memory
$readmemb("patt.mem", in_mem);
Table C.1—Argument return value for $countdriver function
Argument
Return value
net_is_forced
## 1 if net is forced.

## 0 otherwise.

number_of_01x_drivers
An integer representing the number of drivers on the net that are in 0, 1, or x
state. This represents the total number of drivers that are not forced.
number_of_0_drivers
An integer representing the number of drivers on the net that are in 0 state.
number_of_1_drivers
An integer representing the number of drivers on the net that are in 1 state.
number_of_x_drivers
An integer representing the number of drivers on the net that are in x state.
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
513
// step through patterns (each assignment
// to index will drive a new pattern onto the circuit
// inputs from the $getpattern system task specified above
for (index = 1; index <= patterns; index = index + 1)
      #step;
end
// instantiate the circuit module - e.g.,
mod1 cct (o1,o2,o3,o4,o5, i1,i2,i3,i4,i5,i6,i7,i8,i9,i10);
endmodule
C.3 $input
Syntax:
$input ("filename");
The $input system task allows command input text to come from a named file instead of from the terminal.
At the end of the command file, the input is switched back to the terminal.
C.4 $key and $nokey
Syntax:
$key [ ("filename") ] ;
$nokey ;
A key file is created whenever interactive mode is entered for the first time during simulation. The key file
contains all of the text that has been typed in from the standard input. The file also contains information
about asynchronous interrupts.
The $nokey and $key system tasks are used to disable and reenable output to the key file. An optional file
name argument for $key causes the old key file to be closed, a new file to be created, and output to be
directed to the new file.
C.5 $list
Syntax:
$list [ ( hierarchical_name ) ] ;
When invoked without an argument, $list produces a listing of the module, task, function, or named block
that is defined as the current scope setting. If an optional argument is supplied, it shall refer to a specific
module, task, function, or named block, in which case the specified object is listed.
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
514
Copyright © 2006 IEEE. All rights reserved.
C.6 $log and $nolog
Syntax:
$log [ ("filename") ] ;
$nolog ;
A log file contains a copy of all the text that is printed to the standard output. The log file may also contain,
at the beginning of the file, the host command that was used to run the tool.
The $nolog and $log system tasks are used to disable and reenable output to the log file. The $nolog task
disables output to the log file, while the $log task reenables the output. An optional file name argument for
$log causes the old file to be closed, a new log file to be created, and output to be directed to the new log file.
C.7 $reset, $reset_count, and $reset_value
Syntax:
$reset [ ( stop_value [ , reset_value , [ diagnostics_value ] ] ) ] ;
$reset_count ;
$reset_value ;
The $reset system task enables a tool to be reset to its “time 0” state so that processing (e.g., simulation) can
begin again.
The $reset_count system function keeps track of the number of times the tool is reset. The $reset_value
system function returns the value specified by the reset_value argument to the $reset system task. The
$reset_value system function is used to communicate information from before a reset of a tool to the time 0
state to after the reset.
The following are some of the simulation methods that can be employed with this system task and these
system functions:
—
Determine the force statements a design needs to operate correctly, reset the simulation time to 0,
enter these force statements, and start to simulate again.
—
Reset the simulation time to 0 and apply new stimuli.
—
Determine that debug system tasks, such as $monitor and $strobe, are keeping track of the correct
nets or regs, reset the simulation time to 0, and begin simulation again.
The $reset system task tells a tool to return the processing of the design to its logical state at time 0. When a
tool executes the $reset system task, it takes the following actions to stop the process:
a)
Disables all concurrent activity, initiated in either initial or always procedural blocks in the source
description or through interactive mode (disables, for example, all force and assign statements, the
current $monitor system task, and any other active tasks).
b)
Cancels all scheduled simulation events.
After a simulation tool executes the $reset system task, the simulation is in the following state:
—
The simulation time is 0.
—
All regs and nets contain their initial values.
—
The tool begins to execute the first procedural statements in all initial and always blocks.
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
515
The stop_value argument indicates if interactive mode or processing is entered immediately after
resetting of the tool. A value of 0 or no argument causes interactive mode to be entered after resetting the
tool. A nonzero value passed to $reset causes the tool to begin processing immediately.
The reset_value argument is an integer that specifies the value that shall be returned by the $reset_value
system function after the tool is reset. All declared integers return to their initial value after reset, but
entering an integer as this argument allows access to what its value was before the reset with the
$reset_value system function. This argument provides a means of communicating information from before
the reset of a tool to after the reset of the tool.
The diagnostic_value specifies the kind of diagnostic messages a tool displays before it resets the time
to 0. Increasing integer values results in increased information. A value of zero results in no diagnostic
message.
C.8 $save, $restart, and $incsave
Three system tasks $save, $restart, and $incsave work in conjunction with one another to save the complete
state of simulation into a permanent file so that the simulation state can be reloaded at a later time and
processing can continue where it left off.
Syntax:
$save("file_name");
$restart("file_name");
$incsave("incremental_file_name");
All three system tasks take a file name as an argument. The file name has to be supplied as a string enclosed
in quotation marks.
The $save system task saves the complete state into the file specified as an argument.
The $incsave system task saves only what has changed since the last invocation of $save. It is not possible
to do an incremental save on any file other than the one produced by the last $save.
The $restart system task restores a previously saved state from a specified file.
Restarting from an incremental save is similar to restarting from a full save, except that the name of the
incremental save file is specified in the restart command. The full save file on which the incremental save
file was based shall still be present, as it is required for a successful restart. If the full save file has been
changed in any way since the incremental save was performed, errors will result.
The incremental restart is useful for going back in time. If a full save is performed near the beginning of
processing and an incremental save is done at regular intervals, then going back in time is as simple as
restarting from the appropriate file.
For example:
module checkpoint;
initial
#500 $save("save.dat"); // full save
always begin
// incremental save every 10000 units,
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
Std 1364-2005
IEEE STANDARD FOR VERILOG®
516
Copyright © 2006 IEEE. All rights reserved.
// files are recycled every 40000 units
#100000 $incsave("inc1.dat");
#100000 $incsave("inc2.dat");
#100000 $incsave("inc3.dat");
#100000 $incsave("inc4.dat");
end
endmodule
C.9 $scale
Syntax:
$scale ( hierarchical_name ) ;
The $scale function takes a time value from a module with one time unit to be used in a module with a
different time unit. The time value is converted from the time unit of one module to the time unit of the
module that invokes $scale.
C.10 $scope
Syntax:
$scope ( hierarchical_name ) ;
The $scope system task allows a particular level of hierarchy to be specified as the scope for identifying
objects. This task accepts a single argument that shall be the complete hierarchical name of a module, task,
function, or named block. The initial setting of the interactive scope is the first top-level module.
C.11 $showscopes
Syntax:
$showscopes [ ( n ) ];
The $showscopes system task produces a complete list of modules, tasks, functions, and named blocks that
are defined at the current scope level. An optional integer argument can be given to $showscopes. A
nonzero argument value causes all the modules, tasks, functions, and named blocks in or below the current
hierarchical scope to be listed. No argument or a zero value results in only objects at the current scope level
being listed.
C.12 $showvars
Syntax:
$showvars [ ( list_of_variables ) ] ;
The $showvars system task produces status information for reg and net variables, both scalar and vector.
When invoked without arguments, $showvars displays the status of all variables in the current scope. When
invoked with a list of variables, $showvars shows only the status of the specified variables. If the list of
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
IEEE
HARDWARE DESCRIPTION LANGUAGE
Std 1364-2005
Copyright © 2006 IEEE. All rights reserved.
517
variables includes a bit-select or part-select of a reg or net, then the status information for all the bits of that
reg or net are displayed.
C.13 $sreadmemb and $sreadmemh
Syntax:
$sreadmemb ( mem_name , start_address , finish_address , string { , string } ) ;
$sreadmemh ( mem_name , start_address , finish_address , string { , string } ) ;
The system tasks $sreadmemb and $sreadmemh load data into memory mem_name from a character string.
The $sreadmemh and $sreadmemb system tasks take memory data values and addresses as string
arguments. The start and finish addresses indicate the bounds for where the data from strings will be stored
in the memory. These strings take the same format as the strings that appear in the input files passed as
arguments to $readmemb and $readmemh.
Authorized licensed use limited to: Bucknell University. Downloaded on June 12,2014 at 13:56:54 UTC from IEEE Xplore.  Restrictions apply.
