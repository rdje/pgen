---
title: "Section 83: IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language"
document: "SystemVerilog Language Reference Manual"
standard: "IEEE 1800-2017"
domain: "SystemVerilog"
section: "83"
source_txt: "section-83-accept-on-operator.txt"
source_pdf: "/Users/richarddje/Documents/github/SystemVerilog-LRM-IEEE-1800-2017.pdf"
---

# Section 83: IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language

IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1294
Copyright © 2018 IEEE. All rights reserved.
#define vpiObjId
660
#define vpiDPIPure
665
#define vpiDPIContext
666
#define vpiDPICStr
667
#define vpiDPI
1
#define vpiDPIC
2
#define vpiDPICIdentifier
668
#define vpiIsModPort             669
/******************************** Operators *******************************/
#define vpiImplyOp             50 /* -> implication operator */
#define vpiNonOverlapImplyOp   51 /* |=> nonoverlapped implication */
#define vpiOverlapImplyOp      52 /* |-> overlapped implication operator */
#define vpiAcceptOnOp
## 83 /* accept_on operator */

#define vpiRejectOnOp
## 84 /* reject_on operator */

#define vpiSyncAcceptOnOp
## 85 /* sync_accept_on operator */

#define vpiSyncRejectOnOp
## 86 /* sync_reject_on operator */

#define vpiOverlapFollowedByOp 87 /* overlapped followed_by operator */
#define vpiNonOverlapFollowedByOp 88 /* nonoverlapped followed_by operator */
#define vpiNexttimeOp
## 89 /* nexttime operator */

#define vpiAlwaysOp
## 90 /* always operator */

#define vpiEventuallyOp
## 91 /* eventually operator */

#define vpiUntilOp
## 92 /* until operator */

#define vpiUntilWithOp
## 93 /* until_with operator */

#define vpiUnaryCycleDelayOp   53 /* binary cycle delay (##) operator */
#define vpiCycleDelayOp        54 /* binary cycle delay (##) operator */
#define vpiIntersectOp         55 /* intersection operator */
#define vpiFirstMatchOp        56 /* first_match operator */
#define vpiThroughoutOp        57 /* throughout operator */
#define vpiWithinOp            58 /* within operator */
#define vpiRepeatOp            59 /* [=] nonconsecutive repetition */
#define vpiConsecutiveRepeatOp 60 /* [*] consecutive repetition */
#define vpiGotoRepeatOp        61 /* [->] goto repetition */
#define vpiPostIncOp           62 /* ++ post-increment */
#define vpiPreIncOp            63 /* ++ pre-increment */
#define vpiPostDecOp           64 /* -- post-decrement */
#define vpiPreDecOp            65 /* -- pre-decrement */
#define vpiMatchOp             66 /* match() operator */
#define vpiCastOp              67 /* type'() operator */
#define vpiIffOp               68 /* iff operator */
#define vpiWildEqOp            69 /* ==? operator */
#define vpiWildNeqOp           70 /* !=? operator */
#define vpiStreamLROp          71 /* left-to-right streaming {>>} operator */
#define vpiStreamRLOp          72 /* right-to-left streaming {<<} operator */
#define vpiMatchedOp           73 /* the .matched sequence operation */
#define vpiTriggeredOp
## 74 /* the .triggered sequence operation */

#define vpiAssignmentPatternOp 75 /* '{} assignment pattern */
#define vpiMultiAssignmentPatternOp 76 /* '{n{}} multi assignment pattern */
#define vpiIfOp
## 77 /* if operator */

#define vpiIfElseOp
## 78 /* if–else operator */

#define vpiCompAndOp
## 79 /* Composite and operator */

#define vpiCompOrOp
## 80 /* Composite or operator */

#define vpiImpliesOp
## 94 /* implies operator */

Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
