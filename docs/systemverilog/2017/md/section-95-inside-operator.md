---
title: "Section 95: IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language"
document: "SystemVerilog Language Reference Manual"
standard: "IEEE 1800-2017"
domain: "SystemVerilog"
section: "95"
source_txt: "section-95-inside-operator.txt"
source_pdf: "/Users/richarddje/Documents/github/SystemVerilog-LRM-IEEE-1800-2017.pdf"
---

# Section 95: IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language

IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1295
Copyright © 2018 IEEE. All rights reserved.
#define vpiInsideOp
## 95 /* inside operator */

#define vpiTypeOp
## 81 /* type operator */

#define vpiAssignmentOp
## 82 /* Normal assignment */

/*********************** task/function properties ***********************/
#define vpiOtherFunc
## 6 /* returns other types; for property vpiFuncType */

/* vpiValid and vpiValidUnknown were deprecated in 1800-2009 */
/*********************** value for vpiValid *****************************/
#define vpiValidUnknown
## 2 /* Validity of variable is unknown */

/************************** STRUCTURE DEFINITIONS *************************/
/***************************** structure *****************************/
/**************************** CALLBACK REASONS ****************************/
#define cbStartOfThread       600 /* callback on thread creation */
#define cbEndOfThread         601 /* callback on thread termination */
#define cbEnterThread         602 /* callback on reentering thread */
#define cbStartOfFrame        603 /* callback on frame creation */
#define cbEndOfFrame          604 /* callback on frame exit */
#define cbSizeChange          605 /* callback on array variable size change */
#define cbCreateObj
## 700 /* callback on class object creation */

#define cbReclaimObj
## 701 /* callback on class object reclaimed by

automatic memory management */
#define cbEndOfObject
## 702 /* callback on transient object deletion */

/************************* FUNCTION DECLARATIONS **************************/
/**************************************************************************/
/*************************** Coverage VPI *********************************/
/**************************************************************************/
/* coverage control */
#define vpiCoverageStart          750
#define vpiCoverageStop           751
#define vpiCoverageReset          752
#define vpiCoverageCheck          753
#define vpiCoverageMerge          754
#define vpiCoverageSave           755
/* coverage type properties */
#define vpiAssertCoverage         760
#define vpiFsmStateCoverage       761
#define vpiStatementCoverage      762
#define vpiToggleCoverage         763
/* coverage status properties */
#define vpiCovered                765
#define vpiCoverMax               766 /* preserved for backward compatibility */
#define vpiCoveredMax             766
#define vpiCoveredCount           767
/* assertion-specific coverage status properties */
#define vpiAssertAttemptCovered   770
#define vpiAssertSuccessCovered   771
#define vpiAssertFailureCovered   772
#define vpiAssertVacuousSuccessCovered 773
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1296
Copyright © 2018 IEEE. All rights reserved.
#define vpiAssertDisableCovered
774
#define vpiAssertKillCovered
777
/* FSM-specific coverage status properties */
#define vpiFsmStates              775
#define vpiFsmStateExpression     776
/* FSM handle types */
#define vpiFsm                    758
#define vpiFsmHandle              759
/***************************************************************************/
/***************************** Assertion VPI *******************************/
/***************************************************************************/
/* assertion callback types */
#define cbAssertionStart
606
#define cbAssertionSuccess
607
#define cbAssertionFailure
608
#define cbAssertionVacuousSuccess
657
#define cbAssertionDisabledEvaluation
658
#define cbAssertionStepSuccess
609
#define cbAssertionStepFailure
610
#define cbAssertionLock
661
#define cbAssertionUnlock
662
#define cbAssertionDisable
611
#define cbAssertionEnable
612
#define cbAssertionReset
613
#define cbAssertionKill
614
#define cbAssertionEnablePassAction
645
#define cbAssertionEnableFailAction
646
#define cbAssertionDisablePassAction
647
#define cbAssertionDisableFailAction
648
#define cbAssertionEnableNonvacuousAction
649
#define cbAssertionDisableVacuousAction
650
/* assertion "system" callback types */
#define cbAssertionSysInitialized
615
#define cbAssertionSysOn
616
#define cbAssertionSysOff
617
#define cbAssertionSysKill
631
#define cbAssertionSysLock
659
#define cbAssertionSysUnlock
660
#define cbAssertionSysEnd
618
#define cbAssertionSysReset
619
#define cbAssertionSysEnablePassAction
651
#define cbAssertionSysEnableFailAction
652
#define cbAssertionSysDisablePassAction
653
#define cbAssertionSysDisableFailAction
654
#define cbAssertionSysEnableNonvacuousAction
655
#define cbAssertionSysDisableVacuousAction
656
/* assertion control constants */
#define vpiAssertionLock
645
#define vpiAssertionUnlock
646
#define vpiAssertionDisable
620
#define vpiAssertionEnable
621
#define vpiAssertionReset
622
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1297
Copyright © 2018 IEEE. All rights reserved.
#define vpiAssertionKill
623
#define vpiAssertionEnableStep
624
#define vpiAssertionDisableStep
625
#define vpiAssertionClockSteps
626
#define vpiAssertionSysLock
647
#define vpiAssertionSysUnlock
648
#define vpiAssertionSysOn
627
#define vpiAssertionSysOff
628
#define vpiAssertionSysKill
632
#define vpiAssertionSysEnd
629
#define vpiAssertionSysReset
630
#define vpiAssertionDisablePassAction
633
#define vpiAssertionEnablePassAction
634
#define vpiAssertionDisableFailAction
635
#define vpiAssertionEnableFailAction
636
#define vpiAssertionDisableVacuousAction
637
#define vpiAssertionEnableNonvacuousAction
638
#define vpiAssertionSysEnablePassAction
639
#define vpiAssertionSysEnableFailAction
640
#define vpiAssertionSysDisablePassAction
641
#define vpiAssertionSysDisableFailAction
642
#define vpiAssertionSysEnableNonvacuousAction 643
#define vpiAssertionSysDisableVacuousAction
644
typedef struct t_vpi_assertion_step_info {
   PLI_INT32 matched_expression_count;
   vpiHandle *matched_exprs;                 /* array of expressions */
PLI_INT32 stateFrom, stateTo;             /* identify transition */
} s_vpi_assertion_step_info, *p_vpi_assertion_step_info;
typedef struct t_vpi_attempt_info {
   union {
      vpiHandle failExpr;
      p_vpi_assertion_step_info step;
   } detail;
   s_vpi_time attemptStartTime;     /* Time attempt triggered */
} s_vpi_attempt_info, *p_vpi_attempt_info;
/* typedef for vpi_register_assertion_cb callback function */
typedef PLI_INT32(vpi_assertion_callback_func)(
   PLI_INT32 reason,                /* callback reason */
   p_vpi_time cb_time,              /* callback time */
   vpiHandle assertion,             /* handle to assertion */
   p_vpi_attempt_info info,         /* attempt related information */
   PLI_BYTE8 *user_data             /* user data entered upon registration */
);
vpiHandle vpi_register_assertion_cb(
   vpiHandle assertion,             /* handle to assertion */
   PLI_INT32 reason,                /* reason for which callbacks needed */
   vpi_assertion_callback_func *cb_rtn,
   PLI_BYTE8 *user_data             /* user data to be supplied to cb */
);
#ifdef __cplusplus
}
#endif
#endif
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1298
Copyright © 2018 IEEE. All rights reserved.
Annex N
(normative)
Algorithm for probabilistic distribution functions
N.1 General
This annex lists the C source code for the SystemVerilog probabilistic distribution system functions.
Table N.1 shows the SystemVerilog system function names with their corresponding C functions. See 20.15
for the syntactical definition of these system functions.
The algorithm for these functions is defined by the C code in N.2.
N.2 Source code
/*
*
Algorithm for probabilistic distribution functions.
*
*
IEEE Std 1800-2017 SystemVerilog Unified Hardware Design and Verification Language
*/
#include <limits.h>
static double uniform( long *seed, long start, long end );
static double normal( long *seed, long mean, long deviation);
static double exponential( long *seed, long mean);
static long poisson( long *seed, long mean);
static double chi_square( long *seed, long deg_of_free);
static double t( long *seed, long deg_of_free);
static double erlangian( long *seed, long k, long mean);
long
rtl_dist_chi_square( seed, df )
long *seed;
Table N.1—SystemVerilog to C function cross-listing
SystemVerilog function
 C function
$dist_uniform
 rtl_dist_uniform
$dist_normal
 rtl_dist_normal
$dist_exponential
rtl_dist_exponential
$dist_poisson
rtl_dist_poisson
$dist_chi_square
rtl_dist_chi_square
$dist_t
rtl_dist_t
$dist_erlang
rtl_dist_erlang
$random
rtl_dist_uniform (seed, LONG_MIN, LONG_MAX)
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1299
Copyright © 2018 IEEE. All rights reserved.
long df;
{
double r;
long i;
if(df>0)
{
 r=chi_square(seed,df);
 if(r>=0)
 {
i=(long)(r+0.5);
 }
else
 {
r = -r;
i=(long)(r+0.5);
i = -i;
}
}
 else
{
print_error("WARNING: Chi_square distribution must ",
"have positive degree of freedom\n");
i=0;
}
return (i);
}
long
rtl_dist_erlang( seed, k, mean )
long *seed;
long k, mean;
{
double r;
long i;
if(k>0)
{
r=erlangian(seed,k,mean);
if(r>=0)
{
i=(long)(r+0.5);
}
else
{
r = -r;
i=(long)(r+0.5);
i = -i;
}
}
else
{
 print_error("WARNING: k-stage erlangian distribution ",
"must have positive k\n");
i=0;
}
return (i);
}
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1300
Copyright © 2018 IEEE. All rights reserved.
long
rtl_dist_exponential( seed, mean )
long *seed;
long mean;
{
double r;
long i;
if(mean>0)
{
r=exponential(seed,mean);
if(r>=0)

{
i=(long)(r+0.5);
}
else
{
r = -r;
i=(long)(r+0.5);
i = -i;
}
 }
else
{
print_error("WARNING: Exponential distribution must ",
"have a positive mean\n");
i=0;
}
return (i);
}
long
rtl_dist_normal( seed, mean, sd )
long *seed;
long mean, sd;
{
double r;
long i;
r=normal(seed,mean,sd);
if(r>=0)
{
i=(long)(r+0.5);
}
else
{
r = -r;
i=(long)(r+0.5);
i = -i;
}
return (i);
}
long
rtl_dist_poisson( seed, mean )
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1301
Copyright © 2018 IEEE. All rights reserved.
long *seed;
long mean;
{
long i;
if(mean>0)
{
i=poisson(seed,mean);
}
else
{
print_error("WARNING: Poisson distribution must have a ",
"positive mean\n");
i=0;
}
return (i);
}
long
rtl_dist_t( seed, df )
long *seed;
long df;
{
double r;
long i;
if(df>0)
{
r=t(seed,df);
if(r>=0)
{
i=(long)(r+0.5);
}
else
{
r = -r;
i=(long)(r+0.5);
i = -i;
}
}
else
{
print_error("WARNING: t distribution must have positive ",
"degree of freedom\n");
i=0;
}
return (i);
}
long
rtl_dist_uniform(seed, start, end)
long *seed;
long start, end;
{
double r;
long i;
if (start >= end) return(start);
if (end != LONG_MAX)
{
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1302
Copyright © 2018 IEEE. All rights reserved.
end++;
r = uniform( seed, start, end );
if (r >= 0)
{
i = (long) r;
}
else
{
i = (long) (r-1);
}
if (i<start) i = start;
if (i>=end) i = end-1;
}
else if (start!=LONG_MIN)
{
start--;
r = uniform( seed, start, end) + 1.0;
if (r>=0)
{
i = (long) r;
}
else
{
i = (long) (r-1);
}
if (i<=start) i = start+1;
if (i>end) i = end;
}
else
{
r =(uniform(seed,start,end)+
2147483648.0)/4294967295.0;
r = r*4294967296.0-2147483648.0;
if (r>=0)
{
i = (long) r;
}
else
{
i = (long) (r-1);
}
}
return (i);
}
static double
uniform( seed, start, end )
long *seed, start, end;
{
union u_s
{
float s;
unsigned stemp;
} u;
double d = 0.00000011920928955078125;
double a,b,c;
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1303
Copyright © 2018 IEEE. All rights reserved.
if ((*seed) == 0)
*seed = 259341593;
if (start >= end)
{
a = 0.0;
b = 2147483647.0;
}
else
{
a = (double) start;
b = (double) end;
}
*seed = 69069 * (*seed) + 1;
u.stemp = *seed;
/*
 * This relies on IEEE floating point format
 */
u.stemp = (u.stemp >> 9) | 0x3f800000;
c = (double) u.s;
c = c+(c*d);
c = ((b - a) * (c - 1.0)) + a;
return (c);
}
static double
normal(seed,mean,deviation)
long *seed,mean,deviation;
{
double v1,v2,s;
double log(), sqrt();
s = 1.0;
while((s >= 1.0) || (s == 0.0))
{
v1 = uniform(seed,-1,1);
v2 = uniform(seed,-1,1);
s = v1 * v1 + v2 * v2;
}
s = v1 * sqrt(-2.0 * log(s) / s);
v1 = (double) deviation;
v2 = (double) mean;
return(s * v1 + v2);
}
static double
exponential(seed,mean)
long *seed,mean;
{
double log(),n;
n = uniform(seed,0,1);
if(n != 0)
{
n = -log(n) * mean;
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1304
Copyright © 2018 IEEE. All rights reserved.
}
return(n);
}
static long
poisson(seed,mean)
long *seed,mean;
{
long n;
double p,q;
double exp();
n = 0;
q = -(double)mean;
p = exp(q);
q = uniform(seed,0,1);
while(p < q)
{
n++;
q = uniform(seed,0,1) * q;
}
return(n);
}
static double
chi_square(seed,deg_of_free)
long *seed,deg_of_free;
{
double x;
long k;
if(deg_of_free % 2)
{
x = normal(seed,0,1);
x = x * x;
}
else
{
x = 0.0;
}
for(k = 2; k <= deg_of_free; k = k + 2)
{
x = x + 2 * exponential(seed,1);
}
return(x);
}
static double
t(seed,deg_of_free)
long *seed,deg_of_free;
{
double sqrt(),x;
double chi2 = chi_square(seed,deg_of_free);
double div = chi2 / (double)deg_of_free;
double root = sqrt(div);
x = normal(seed,0,1) / root;
return(x);
}
static double
erlangian(seed,k,mean)
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1305
Copyright © 2018 IEEE. All rights reserved.
long *seed,k,mean;
{
double x,log(),a,b;
long i;
x=1.0;
for(i=1;i<=k;i++)
{
x = x * uniform(seed,0,1);
}
a=(double)mean;
b=(double)k;
x= -a*log(x)/b;
return(x);
}
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1306
Copyright © 2018 IEEE. All rights reserved.
Annex O
(informative)
Encryption/decryption flow
O.1 General
This annex describes a number of scenarios that can be used for IP protection. It also shows how the relevant
pragmas are used to achieve the desired effect of securely protecting, distributing, and decrypting the model.
O.2 Overview
The data to be protected from inappropriate access or from unauthorized modification is placed within a
protect begin-end block. Information in the begin-end block, once encrypted, is also protected.
O.3 Tool vendor secret key encryption system
In the secret key encryption system, the key is tool vendor proprietary and is embedded within the tool itself.
The same key is used for both encryption and decryption. [In the electronic design automation (EDA)
domain, this is the simplest scenario and is roughly equivalent to the historical `protect technique.] It has
the drawback of being completely tool vendor-specific. Using this technique, the IP author can encrypt the
IP, and any IP consumer with appropriate licenses and the same tool vendor can utilize the IP.
O.3.1 Encryption input
The following pragmas are expected when using the tool vendor secret key encryption system. The pragmas
required in the encryption input for use of the secret key encryption system are as follows:
data_keyname=<key name>
Where <key name> is a valid name of a tool’s embedded key.
begin-end
Surrounding the region(s) to be encrypted.
Additional optional pragmas that may be included are as follows:
author=<string>
To embed author name.
author_info=<string>
To embed arbitrary author information.
data_keyowner=<owner identity>This shall be the key owner of the provided name.
data_method=<method-specifier>A method appropriate for the given key name. This may be
necessary if something other than the default number of rounds,
initialization vector, or key width is used.
encoding=<encoding-specifier>
To specify a different encoding.
digest_block
If a message authorization code is desired to validate that the
message has not been modified.
decrypt_license
If the IP author desires a decryption license.
runtime_license
If the IP author desires a run-time license.
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1307
Copyright © 2018 IEEE. All rights reserved.
O.3.2 Encryption output
The encrypting tool should take the input file and copy all cleartext to the corresponding output sections. For
each protect begin-end block, it should generate the following:
begin_protected
To start the protected region.
data_keyowner= <owner identity>
data_keyname=<key name>
data_method=<method-specifier>
encoding=<encoding-specifier>
author=<string>
If provided in the input.
author_info=<string>
If provided in the input.
digest_block
Followed on the next line(s) by the encoded encrypted digest.
data_block
Followed on the next line(s) by the encoded encrypted data
composed of the following:
decrypt_license
encrypt_license
<text found between begin-end>
end_protected
O.4 IP author secret key encryption system
In this mechanism, the IP is encrypted with the public key (of a public/private key pair) of the IP author, and
the decrypting tool will have the IP author’s private key in its secure key database. The IP authors will have
to provide their private keys to the tools’ database so that the tool will be able to decrypt the design.
O.4.1 Encryption input
 The following pragmas are expected when using the IP author secret key encryption system:
data_keyname=< provider’s key name>
begin-end
Surrounding the region(s) to be encrypted.
Additional optional pragmas that may be included are as follows:
author=<string>
To embed author name.
author_info=<string>
To embed arbitrary author information.
data_keyowner=<owner identity>This shall be the key owner of the provided name.
data_method= some_publ_priv_encryption_scheme_name <method-specifier>
A method appropriate for the given key name. This may be
necessary if something other than the default number of rounds,
initialization vector, or key width is used.
encoding=<encoding-specifier> To specify a different encoding.
digest_block
If a message authorization code is desired to validate that the
message has not been modified.
decrypt_license
If the IP author desires a decryption license.
runtime_license
If the IP author desires a run-time license.
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1308
Copyright © 2018 IEEE. All rights reserved.
O.4.2 Encryption output
The encrypting tool should take the input file and copy all cleartext to the corresponding output sections. For
each protect begin-end block, it should generate the following:
begin_protected
To start the protected region.
data_keyowner=<owner identity>
data_keyname=<provider’s key name>
data_method=some_publ_priv_encryption_scheme_name
encoding=<encoding-specifier>
author=<string>
If provided in the input.
author_info=<string>
If provided in the input.
digest_block
Followed on the next line(s) by the encoded encrypted digest.
data_block
Followed on the next line(s) by the encoded encrypted data
composed of the following:
decrypt_license
encrypt_license
<text found between begin-end>
end_protected
O.5 Digital envelopes
In this mechanism, each recipient has a public and private key for an asymmetric encryption algorithm. The
sender encrypts the design using a symmetric key encryption algorithm and then encrypts the symmetric key
using the recipient’s public key. The encrypted symmetric key is recorded in a key_block in the protected
envelope. The recipient is able to recover the symmetric key using the appropriate private key and then
decrypts the design with the symmetric key. This technique permits efficient encryption methods for the
design data, yet secret information is never transmitted without encryption. Digital envelopes can be created
using either tool secret key or IP author secret key protection schemes. The keys for the recipient user or tool
protect the transmission of the symmetric key that encrypts the design data. By using more than one
key_block, a single protected envelope can be decrypted by tools from different vendors and/or different
users.
In the following example, the data_method and data_keyowner/data_keyname are used to encrypt the
data_block. The key to encrypt the data_block can be specified either by a data_keyowner/
data_keyname pair or by a data_decrypt_key pragma expression. In the first case, the encrypting tool
encrypts the data_keyowner and data_keyname pragmas with the key_keymethod/key_keyname and
puts them in the key_block along with data_method. Alternatively, with the data_decrypt_key
pragma, the actual key is provided, which is then encrypted with key_method/key_keyname and stored in
the key_block.
In the first approach, the data_keyowner/data_keyname should also be present with the decrypting tool.
No such dependency exists with the second approach as the key is present in the file itself.
For better security in the first approach, the encrypting tool can actually read the data_keyowner/
data_keyname key and put it in the key_block as data_decrypt_key. This step not only will remove
the dependency mentioned above, but will also protect against the hit-and-trial breaking of the data_block
with the existing keys at the IP user’s end.
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1309
Copyright © 2018 IEEE. All rights reserved.
O.5.1 Encryption input
The following pragmas are expected when using the digital envelopes:
key_keyowner=<owner identity>
key_method=some_encryption_scheme_name
key_keyname=<provider’s key name>
data_keyname=<provider’s key name>
begin-end
Surrounding the region(s) to be encrypted.
Additional optional pragmas that may be included are as follows:
author=<string>
To embed author name.
author_info=<string>
To embed arbitrary author information.
data_keyowner=<owner identity>This shall be the key owner of the provided name.
data_method=<method-specifier>A method appropriate for the given key name. This may be
necessary if something other than the default number of rounds,
initialization vector, or key width is used
encoding=<encoding-specifier>
To specify a different encoding.
digest_block
If a message authorization code is desired to validate that the
message has not been modified.
decrypt_license
If the IP author desires a decryption license.
runtime_license
If the IP author desires a run-time license.
O.5.2 Encryption output
The encrypting tool should take the input file and copy all cleartext to the corresponding output sections. For
each protect begin-end block, it should generate the following:
begin_protected
To start the protected region.
key_keyowner=<owner identity>
key_method=some_encryption_scheme_name
key_keyname=<provider’s key name>
key_block=<encrypted encoded data>
This contains the data_key_owner, data_method, and the symmetric
data_key itself in encrypted form.
encoding=<encoding-specifier>
author=<string>
If provided in the input.
author_info=<string>
If provided in the input.
digest_block
Followed on the next line(s) by the encoded encrypted digest.
data_block
Followed on the next line(s) by the encoded encrypted data
composed of the following:
decrypt_license
encrypt_license
<text found between begin-end>
end_protected
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1310
Copyright © 2018 IEEE. All rights reserved.
Annex P
(informative)
Glossary
For the purposes of this document, the following terms and definitions apply. The IEEE Standards
Dictionary Online should be consulted for terms not defined in this clause. 14
aggregate: A set or collection of singular values, e.g., an aggregate expression, data object, or data type. An
aggregate data type is any unpacked structure, unpacked union, or unpacked array data type. Aggregates
may be copied or compared as a whole, but not typically used in an expression as a whole.
assertion: An assertion statement.
assertion statement: A statement that specifies the verification function to be performed on an underlying
property. An assertion statement is of one of the following kinds:
—
assert, to specify the property as an obligation for the design that is to be checked to verify that the
property holds.
—
assume, to specify the property as an assumption for the environment. Simulators check that the
property holds, while formal tools use the information to generate input stimulus.
—
cover, to monitor the property evaluation for coverage.
—
restrict, to specify the property as a constraint on formal verification computations. Simulators
do not check the property.
The underlying property describes the behavioral criterion that is evaluated by the assertion statement. The
property may be an immediate condition, e.g., that the read_enable and write_enable signals are
mutually exclusive, or it may be a temporal condition, e.g., that if a read_request occurs, then a
read_grant occurs within two clock cycles. An assertion statement is either immediate, for which the
underlying property must be an immediate condition, or concurrent, for which the underlying property may
be either an immediate or a temporal condition. There is no immediate restrict assertion statement.
Assertion statements can generate automatic messages to report that the disposition of the evaluation of the
underlying property is of interest for the kind of the assertion statement, e.g., a failing evaluation disposition
for an assert or assume, or a passing disposition for a cover.
NOTE—SystemVerilog provides special assertion constructs, which are discussed in Clause 16. See 16.2 for a
discussion of assertion statements.
bit-stream data type: Any data type whose values can be represented as a serial stream of bits. To qualify
as a bit-stream data type, each and every bit of the values shall be individually addressable. In other words, a
bit-stream data type can be any data type except for a handle, chandle, real, shortreal, or event.
canonical representation: A data representation format established by convention into which and from
which translations can be made with specialized representations.
constant: Either of two types of constants in SystemVerilog: elaboration constant or run-time constant.
Parameters and local parameters are elaboration constants. Their values are calculated before elaboration is
complete. Elaboration constants can be used to set the range of array types. Run-time constants are variables
that can only be set in an initialization expression using the const qualifier.
14IEEE Standards Dictionary Online is available at: http://dictionary.ieee.org.
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1311
Copyright © 2018 IEEE. All rights reserved.
context imported task: A direct programming interface (DPI) imported task declared with the “context”
property that is capable of calling exported subroutines and capable of accessing SystemVerilog objects via
the SystemVerilog Verification Procedural Interface (VPI) or Programming Language Interface (PLI) calls.
data object: A named entity that has a data value associated with it. Examples of data objects are nets,
variables, and parameters. A data object has a data type that determines which values the data object can
have.
data type: A set of values and a set of operations that can be performed on those values. Examples of data
types are logic, real, and string. Data types can be used to declare data objects or to define user-defined
data types that are constructed from other data types.
direct programming interface (DPI): An interface between SystemVerilog and foreign programming
languages permitting direct function calls from SystemVerilog to foreign code and from foreign code to
SystemVerilog. It has been designed to have low inherent overhead and permit direct exchange of data
between SystemVerilog and foreign code.
disable protocol: A set of conventions for setting, checking, and handling disable status.
dynamic: Having values that can be resized or reallocated at run time. Dynamic arrays, associative arrays,
queues, class handles, and data types that include such data types are dynamic data types.
elaboration: The process of binding together the components that make up a design. These components can
include module instances, primitive instances, interfaces, and the top level of the design hierarchy.
enumerated type: Data types that can declare a data object that can have one of a set of named values. The
numerical equivalents of these values can be specified. Values of an enumerated data type can be easily
referenced or displayed using the enumerated names, as opposed to the enumerated values.
exported task: A SystemVerilog task that is declared in an export declaration and can be enabled from an
imported task.
imported task: A direct programming interface (DPI) foreign code subprogram that can call exported tasks
and can directly or indirectly consume simulation time.
integral: (A) A data type representing integer values. (B) A integer value that may be signed or unsigned,
sliced into smaller integral values, or concatenated into larger values. Syn: vectored value. (C) An
expression of an integral data type. (D) An object of an integral data type.
interface: An encapsulation of the communication between blocks of a design, allowing a smooth migration
from abstract system-level design through successive refinement down to lower level register transfer and
structural views of the design. By encapsulating the communication between blocks, the interface construct
also facilitates design reuse. The inclusion of interface capabilities is one of the major advantages of
SystemVerilog.
Language Reference Manual (LRM): A document describing the syntax, semantics, and usage of a
programming language. SystemVerilog LRM refers to this standard.
open array: A direct programming interface (DPI) array formal argument for which the packed or unpacked
dimension size (or both) is not specified and for which interface routines describe the size of corresponding
actual arguments at run time.
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1312
Copyright © 2018 IEEE. All rights reserved.
packed array: An array where the dimensions are declared before an object name. Packed arrays can have
any number of dimensions. A one-dimensional packed array is the same as a vector width declaration in
IEEE 1364-2005 Verilog. Packed arrays provide a mechanism for subdividing a vector into subfields, which
can be conveniently accessed as array elements. A packed array differs from an unpacked array, in that the
whole array is treated as a single vector for arithmetic operations.
process: A thread of one or more programming statements that can be executed independently of other
programming statements. Each elaborated instance of an initial procedure, always, always_comb,
always_latch, always_ff procedure, or continuous assignment statement in SystemVerilog is a separate
process. These are static processes; their existence is determined by the static instance hierarchy, their
execution begins at the start of simulation, and they cannot be created at run time. SystemVerilog also has
dynamic processes that can be created, stopped, restarted, and destroyed at run time.
signal: An informal term, usually meaning either a variable or net. The context where it is used may imply
further restrictions on allowed types.
singular: An expression, data object, or data type that represents a single value, symbol, or handle. A
singular data type is any data type except an unpacked structure, unpacked union, or unpacked array data
type.
subroutine: An encapsulation of executable code that can be invoked from one or more places. There are
two forms of subroutines, tasks and functions.
unpacked array: An array where the dimensions are declared after an object name. Unpacked arrays are the
same as arrays in IEEE 1364-2005 Verilog and can have any number of dimensions. An unpacked array
differs from a packed array in that the whole array cannot be used for arithmetic operations. Each element
shall be treated separately.
Verification Procedural Interface (VPI): The third generation programming language interface (PLI)
access libraries, providing object-oriented access to SystemVerilog behavioral, structural, assertion, and
coverage objects.
Verilog: The hardware description language (HDL) in IEEE Std 1364-2005.
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
1313
Copyright © 2018 IEEE. All rights reserved.
Annex Q
(informative)
Bibliography
[B1] IEEE Std 1497-2001, IEEE Standard for Standard Delay Format (SDF) for the Electronic Design
Process.15, 16
[B2] IEEE Std 1735-2014, IEEE Recommended Practice for Encryption and Management of Electronic
Design Intellectual Property (IP).
[B3] ISO/IEC 9899:1999, Programming Languages—C.17
[B4] SystemVerilog 3.1a Language Reference Manual, Accellera’s Extensions to Verilog®, 2004.18
15IEEE publications are available from The Institute of Electrical and Electronics Engineers (http://standards.ieee.org/).
16The IEEE standards or products referred to in this clause are trademarks of The Institute of Electrical and Electronics Engineers, Inc.
17ISO/IEC publications are available from the ISO Central Secretariat (http://www.iso.org/). ISO publications are also available in the
United States from the American National Standards Institute (http://www.ansi.org/).
18Available at http://www.eda-twiki.org/sv/SystemVerilog_3.1a.pdf.
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE
standards.ieee.org
Phone: +1 732 981 0060    Fax: +1 732 562 1571
© IEEE
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
