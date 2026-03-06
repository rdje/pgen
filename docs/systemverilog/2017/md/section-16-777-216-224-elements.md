---
title: "Section 16: IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language"
document: "SystemVerilog Language Reference Manual"
standard: "IEEE 1800-2017"
domain: "SystemVerilog"
section: "16"
source_txt: "section-16-777-216-224-elements.txt"
source_pdf: "/Users/richarddje/Documents/github/SystemVerilog-LRM-IEEE-1800-2017.pdf"
---

# Section 16: IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language

IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
146
Copyright © 2018 IEEE. All rights reserved.
Integer types with predefined widths shall not have packed array dimensions declared. These types are
byte, shortint, int, longint, integer, and time. Although an integer type with a predefined width n
is not a packed array, it matches (see 6.22), and can be selected from as if it were, a packed array type with a
single [n-1:0] dimension.
byte c2;
// same as bit signed [7:0] c2;
integer i1; // same as logic signed [31:0] i1;
#### 7.4.2 Unpacked arrays

Unpacked arrays can be made of any data type. Arrays whose elements are themselves arrays are declared as
multidimensional arrays (see 7.4.5). Unpacked arrays shall be declared by specifying the element address
range(s) after the declared identifier.
Elements of net arrays can be used in the same fashion as a scalar or vector net. Net arrays are useful for
connecting to ports of module instances inside loop generate constructs (see 27.4).
Each fixed-size dimension shall be represented by an address range, such as [1:1024], or a single positive
number to specify the size of a fixed-size unpacked array, as in C. In other words, [size] becomes the
same as [0:size-1].
The following examples declare equivalent size two-dimensional fixed-size arrays of int variables:
int Array[0:7][0:31];
// array declaration using ranges
int Array[8][32];
// array declaration using sizes
The expressions that specify an address range shall be constant integer expressions. The value of the
constant expression can be a positive integer, a negative integer, or zero. It shall be illegal for them to
contain any unknown (x) or high-impedance bits.
Implementations may limit the maximum size of an array, but they shall allow at least
## 16 777 216 (224) elements.

#### 7.4.3 Operations on arrays

The following operations can be performed on all arrays, packed or unpacked. The examples provided with
these rules assume that A and B are arrays of the same shape and type.
—
Reading and writing the array, e.g., A = B
—
Reading and writing a slice of the array, e.g., A[i:j] = B[i:j]
—
Reading and writing a variable slice of the array, e.g., A[x+:c] = B[y+:c]
—
Reading and writing an element of the array, e.g., A[i] = B[i]
—
Equality operations on the array or slice of the array, e.g., A==B, A[i:j] != B[i:j]
The following operations can be performed on packed arrays, but not on unpacked arrays. The examples
provided with these rules assume that A is an array.
—
Assignment from an integer, e.g., A = 8'b11111111;
—
Treatment as an integer in an expression, e.g., (A + 3)
If an unpacked array is declared as signed, then this applies to the individual elements of the array because
the whole array cannot be viewed as a single vector.
See 7.6 for rules for assigning to packed and unpacked arrays.
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
147
Copyright © 2018 IEEE. All rights reserved.
#### 7.4.4 Memories

A one-dimensional array with elements of types reg, logic, or bit is also called a memory. Memory
arrays can be used to model read-only memories (ROMs), random access memories (RAMs), and register
files. An element of the packed dimension in the array is known as a memory element or word.
logic [7:0] mema [0:255]; // declares a memory array of 256 8-bit

// elements. The array indices are 0 to 255
mema[5] = 0;
// Write to word at address 5
data = mema[addr];
// Read word at address indexed by addr
#### 7.4.5 Multidimensional arrays

A multidimensional array is an array of arrays. Multidimensional arrays can be declared by including
multiple dimensions in a single declaration. The dimensions preceding the identifier set the packed
dimensions. The dimensions following the identifier set the unpacked dimensions.
bit [3:0] [7:0] joe [1:10];
// 10 elements of 4 8-bit bytes
// (each element packed into 32 bits)
can be used as follows:
joe[9] = joe[8] + 1; // 4 byte add
joe[7][3:2] = joe[6][1:0]; // 2 byte copy
In a multidimensional declaration, the dimensions declared following the type and before the name
([3:0][7:0] in the preceding declaration) vary more rapidly than the dimensions following the name
([1:10] in the preceding declaration). When referenced, the packed dimensions ([3:0], [7:0]) follow
the unpacked dimensions ([1:10]).
In a list of dimensions, the rightmost one varies most rapidly, as in C. However, a packed dimension varies
more rapidly than an unpacked one.
bit [1:10] v1 [1:5];
// 1 to 10 varies most rapidly; compatible with
memory arrays
bit v2 [1:5] [1:10];
// 1 to 10 varies most rapidly, compatible with C
bit [1:5] [1:10] v3 ;
// 1 to 10 varies most rapidly
bit [1:5] [1:6] v4 [1:7] [1:8];
// 1 to 6 varies most rapidly, followed by
// 1 to 5, then 1 to 8 and then 1 to 7
Multiple packed dimensions can also be defined in stages with typedef.
typedef bit [1:5] bsix;
bsix [1:10] v5; // 1 to 5 varies most rapidly
Multiple unpacked dimensions can also be defined in stages with typedef.
typedef bsix mem_type [0:3];
// array of four 'bsix' elements
mem_type ba [0:7];
// array of eight 'mem_type' elements
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
148
Copyright © 2018 IEEE. All rights reserved.
A subarray is an array that is an element of another array. As in the C language, subarrays are referenced by
omitting indices for one or more array dimensions, always omitting the ones that vary most rapidly.
Omitting indices for all the dimensions references the entire array.
int A[2][3][4], B[2][3][4], C[5][4];
...
A[0][2] = B[1][1];
// assign a subarray composed of four ints
A[1] = B[0];
// assign a subarray composed of three arrays of
// four ints each
A = B;
// assign an entire array
A[0][1] = C[4];
// assign compatible subarray of four ints
A comma-separated list of array declarations can be specified. All arrays in the list shall have the same data
type and the same packed array dimensions.
bit [7:0] [31:0] v7 [1:5] [1:10], v8 [0:255];
// two arrays declared
#### 7.4.6 Indexing and slicing of arrays

An expression can select part of a packed array, or any integer type, which is assumed to be numbered down
to 0.
The term part-select refers to a selection of one or more contiguous bits of a single-dimension packed array.
logic [63:0] data;
logic [7:0] byte2;
byte2 = data[23:16]; // an 8-bit part-select from data
The term slice refers to a selection of one or more contiguous elements of an array.
NOTE—IEEE Std 1364-2005 only permitted a single element of an array to be selected.
A single element of a packed or unpacked array can be selected using an indexed name.
bit [3:0] [7:0] j;
// j is a packed array
byte k;
k = j[2]; // select a single 8-bit element from j
One or more contiguous elements can be selected using a slice name. A slice name of a packed array is a
packed array. A slice name of an unpacked array is an unpacked array.
bit signed [31:0] busA [7:0] ;
// unpacked array of 8 32-bit vectors
int busB [1:0];
// unpacked array of 2 integers
busB = busA[7:6];
// select a 2-vector slice from busA
The size of the part-select or slice shall be constant, but the position can be variable.
int i = bitvec[j +: k];
// k must be constant.
int a[x:y], b[y:z], e;
a = {b[c -: d], e};
// d must be constant
Slices of an array can only apply to one dimension, but other dimensions can have single index values in an
expression.
If an index expression is out of bounds or if any bit in the index expression is x or z, then the index shall be
invalid. Reading from an unpacked array of any kind with an invalid index shall return the value specified in
Table 7-1. Writing to an array with an invalid index shall perform no operation, with the exceptions of
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
149
Copyright © 2018 IEEE. All rights reserved.
writing to element [$+1] of a queue (described in 7.10.1) and creating a new element of an associative array
(described in 7.8.6). Implementations may issue a warning if an invalid index occurs for a read or write
operation on an array.
Access to a packed array with an invalid index is described in 11.5.1.
See 11.5.1 and 11.5.2 for more information on vector and array element selecting and slicing.
### 7.5 Dynamic arrays

A dynamic array is an unpacked array whose size can be set or changed at run time. The default size of an
uninitialized dynamic array is zero. The size of a dynamic array is set by the new constructor or array
assignment, described in 7.5.1 and 7.6, respectively. Dynamic arrays support all variable data types as
element types, including arrays.
Dynamic array dimensions are denoted in the array declaration by [ ]. Any unpacked dimension in an array
declaration may be a dynamic array dimension.
For example:
bit [3:0] nibble[];
// Dynamic array of 4-bit vectors
Table 7-1—Value read from a nonexistent array entry
Type of array
Value read
4-state integral type
'X
2-state integral type
'0
Enumeration
Value specified in this table for the enumeration’s
base type
real, shortreal
0.0
string
""
class
null
interface class
null
event
null
chandle
null
virtual interface
null
Variable-size unpacked array
(dynamic, queue or associative)
Array of size zero (no elements)
Fixed-size unpacked array
Array, all of whose elements have the value
specified in this table for that array’s element type
Unpacked struct
struct, each of whose members has the value
specified in this table for that member’s type, unless
the member has an initial assignment as part of its
declaration (see 7.2.2), in which case the member’s
value shall be as given by its initial assignment
Unpacked union
Value specified in this table for the type of the first
member of the union
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
150
Copyright © 2018 IEEE. All rights reserved.
integer mem[2][];
// Fixed-size unpacked array composed
// of 2 dynamic subarrays of integers
Note that in order for an identifier to represent a dynamic array, it must be declared with a dynamic array
dimension as the leftmost unpacked dimension.
The new[] constructor is used to set or change the size of the array and initialize its elements (see 7.5.1).
The size() built-in method returns the current size of the array (see 7.5.2).
The delete() built-in method clears all the elements yielding an empty array (zero size) (see 7.5.3).
#### 7.5.1 New[ ]

The new constructor sets the size of a dynamic array and initializes its elements. It may appear in place of
the right-hand side expression of variable declaration assignments and blocking procedural assignments
when the left-hand side indicates a dynamic array.
```ebnf
blocking_assignment ::=
```

// from A.6.2
...
|  nonrange_variable_lvalue = dynamic_array_new
...
```ebnf
dynamic_array_new ::= new [ expression ] [ ( expression ) ]
```

// from A.2.4
Syntax 7-3—Dynamic array new constructor syntax (excerpt from Annex A)
[ expression ]:
The desired size of the dynamic array. The type of this operand is longint. It shall be an error if the
value of this operand is negative. If this operand is zero, the array shall become empty.
( expression ):
Optional. An array with which to initialize the dynamic array.
The new constructor follows the SystemVerilog precedence rules. Because both the square brackets [] and
the parenthesis () have the same precedence, the arguments to the new constructor are evaluated left to
right: [ expression ] first, and ( expression ) second.
Dynamic array declarations may include a declaration assignment with the new constructor as the right-hand
side:
int arr1 [][2][3] = new [4];
// arr1 sized to length 4; elements are
// fixed-size arrays and so do not require
// initializing
int arr2 [][] = new [4];
// arr2 sized to length 4; dynamic subarrays
// remain unsized and uninitialized
int arr3 [1][2][] = new [4];
// Error – arr3 is not a dynamic array, though
// it contains dynamic subarrays
Dynamic arrays may be initialized in procedural contexts using the new constructor in blocking
assignments:
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
151
Copyright © 2018 IEEE. All rights reserved.
int arr[2][][];
arr[0] = new [4];
// dynamic subarray arr[0] sized to length 4
arr[0][0] = new [2];
// legal, arr[0][n] created above for n = 0..3
arr[1][0] = new [2];
// illegal, arr[1] not initialized so arr[1][0] does
// not exist
arr[0][] = new [2];
// illegal, syntax error - dimension without
// subscript on left hand side
arr[0][1][1] = new[2];
// illegal, arr[0][1][1] is an int, not a dynamic
// array
In either case, if the new constructor call does not specify an initialization expression, the elements are
initialized to the default value for their type.
The optional initialization expression is used to initialize the dynamic array. When present, it shall be an
array that is assignment compatible with the left-hand-side dynamic array.
int idest[], isrc[3] = '{5, 6, 7};
idest = new [3] (isrc); // set size and array element data values (5, 6, 7)
The size argument need not match the size of the initialization array. When the initialization array’s size is
greater, it is truncated to match the size argument; when it is smaller, the initialized array is padded with
default values to attain the specified size.
int src[3], dest1[], dest2[];
src = '{2, 3, 4};
dest1 = new[2] (src);
// dest1's elements are {2, 3}.
dest2 = new[4] (src);
// dest2's elements are {2, 3, 4, 0}.
This behavior provides a mechanism for resizing a dynamic array while preserving its contents. An existing
dynamic array can be resized by using it both as the left-hand side term and the initialization expression.
integer addr[];
// Declare the dynamic array.
addr = new[100];
// Create a 100-element array.
...
// Double the array size, preserving previous values.
// Preexisting references to elements of addr are outdated.
addr = new[200](addr);
Resizing or reinitializing a previously initialized dynamic array using new is destructive; no preexisting
array data is preserved (unless reinitialized with its old contents—see preceding), and all preexisting
references to array elements become outdated.
#### 7.5.2 Size()

The prototype for the size() method is as follows:
function int size();
The size() method returns the current size of a dynamic array or returns zero if the array has not been
created.
int j = addr.size;
addr = new[ addr.size() * 4 ] (addr);
// quadruple addr array
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
152
Copyright © 2018 IEEE. All rights reserved.
The size dynamic array method is equivalent to $size( addr, 1 ) array query system function (see
20.7).
#### 7.5.3 Delete()

The prototype for the delete() method is as follows:
function void delete();
The delete() method empties the array, resulting in a zero-sized array.
int ab [] = new[ N ];
// create a temporary array of size N
// use ab
ab.delete;
// delete the array contents
$display( "%d", ab.size );
// prints 0
### 7.6 Array assignments

For the purposes of assignment, a packed array is treated as a vector. Any vector expression can be assigned
to any packed array. The packed array bounds of the target packed array do not affect the assignment. A
packed array cannot be directly assigned to an unpacked array without an explicit cast.
Associative arrays are assignment compatible only with associative arrays, as described in 7.9.9. A fixed-
size unpacked array, dynamic array, or queue, or a slice of such an array, shall be assignment compatible
with any other such array or slice if all the following conditions are satisfied:
—
The element types of source and target shall be equivalent.
—
If the target is a fixed-size array or a slice, the source array shall have the same number of elements
as the target.
Here element refers to elements of the slowest-varying array dimension. These elements may themselves be
of some unpacked array type. Consequently, for two arrays to be assignment compatible it is necessary (but
not sufficient) that they have the same number of unpacked dimensions. Assignment compatibility of
unpacked arrays is a weaker condition than type equivalence because it does not require their slowest-
varying dimensions to be of the same unpacked array kind (queue, dynamic, or fixed-size). This weaker
condition applies only to the slowest-varying dimension. Any faster-varying dimensions must meet the
requirements for equivalence (see 6.22.2) for the entire arrays to be assignment compatible.
Assignment shall be done by assigning each element of the source array to the corresponding element of the
target array. Correspondence between elements is determined by the left-to-right order of elements in each
array. For example, if array A is declared as int A[7:0] and array B is declared as int B[1:8], the
assignment A = B; will assign element B[1] to element A[7], and so on. If the target of the assignment is
a queue or dynamic array, it shall be resized to have the same number of elements as the source expression
and assignment shall then follow the same left-to-right element correspondence as previously described.
int A[10:1];
// fixed-size array of 10 elements
int B[0:9];
// fixed-size array of 10 elements
int C[24:1];
// fixed-size array of 24 elements
A = B;
// ok. Compatible type and same size
A = C;
// type check error: different sizes
An array of wires can be assigned to an array of variables, and vice versa, if the source and target arrays’
data types are assignment compatible.
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
153
Copyright © 2018 IEEE. All rights reserved.
logic [7:0] V1[10:1];
logic [7:0] V2[10];
wire [7:0] W[9:0];
// data type is logic [7:0] W[9:0]
assign W = V1;
initial #10 V2 = W;
When a dynamic array or queue is assigned to a fixed-size array, the size of the source array cannot be
determined until run time. An attempt to copy a dynamic array or queue into a fixed-size array target having
a different number of elements shall result in a run-time error and no operation shall be performed. Example
code showing assignment of a dynamic array to a fixed-size array follows.
int A[2][100:1];
int B[] = new[100];
// dynamic array of 100 elements
int C[] = new[8];
// dynamic array of 8 elements
int D [3][][];
// multidimensional array with dynamic subarrays
D[2] = new [2];
// initialize one of D's dynamic subarrays
D[2][0] = new [100];
A[1] = B;
// OK. Both are arrays of 100 ints
A[1] = C;
// type check error: different sizes (100 vs. 8 ints)
A = D[2];
// A[0:1][100:1] and subarray D[2][0:1][0:99] both
// comprise 2 subarrays of 100 ints
Examples showing assignment to a dynamic array follow. (See 7.5.1 for additional assignment examples
involving the dynamic array new constructor).
int A[100:1];
// fixed-size array of 100 elements
int B[];
// empty dynamic array
int C[] = new[8];
// dynamic array of size 8
B = A;
// ok. B has 100 elements
B = C;
// ok. B has 8 elements
The previous last statement is equivalent to:
B = new[ C.size ] (C);
Similarly, the source of an assignment can be a complex expression involving array slices or concatenations.
For example:
string d[1:5] = '{ "a", "b", "c", "d", "e" };
string p[];
p = { d[1:3], "hello", d[4:5] };
The preceding example creates the dynamic array p with contents "a", "b", "c", "hello", "d", "e".
### 7.7 Arrays as arguments to subroutines

Arrays can be passed as arguments to subroutines. The rules that govern array argument passing by value are
the same as for array assignment (see 7.6). When an array argument is passed by value, a copy of the array is
passed to the called subroutine. This is true for all array types: fixed-size, dynamic, queue, or associative.
The rules that govern whether an array actual argument can be associated with a given formal argument are
the same as the rules for whether a source array’s values can be assigned to a destination array (see 7.6). If a
dimension of a formal is unsized (unsized dimensions can occur in dynamic arrays, queues, and formal
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
154
Copyright © 2018 IEEE. All rights reserved.
arguments of import DPI functions), then it matches any size of the actual argument’s corresponding
dimension.
For example, the declaration
task fun(int a[3:1][3:1]);
declares task fun that takes one argument, a two-dimensional array with each dimension of size 3. A call to
fun must pass a two-dimensional array and with the same dimension size 3 for all the dimensions. For
example, given the preceding description for fun, consider the following actuals:
int b[3:1][3:1];
// OK: same type, dimension, and size
int b[1:3][0:2];
// OK: same type, dimension, & size (different ranges)
logic b[3:1][3:1];
// error: incompatible element type
event b[3:1][3:1];
// error: incompatible type
int b[3:1];
// error: incompatible number of dimensions
int b[3:1][4:1];
// error: incompatible size (3 vs. 4)
A subroutine that accepts a fixed-size array can also be passed a dynamic array or queue with compatible
type and equal size.
For example, the declaration
task t( string arr[4:1] );
declares a task that accepts one argument, an array of 4 strings. This task can accept the following actual
arguments:
string b[4:1];
// OK: same type and size
string b[5:2];
// OK: same type and size (different range)
string b[] = new[4]; // OK: same type, number of dimensions, and
// dimension size; requires run-time check
A subroutine that accepts a dynamic array or queue can be passed a dynamic array, queue, or fixed-size
array of a compatible type.
For example, the declaration
task t ( string arr[] );
declares a task that accepts one argument, a dynamic array of strings. This task can accept any
one-dimensional unpacked array of strings or any one-dimensional dynamic array or queue of strings.
The rules that govern dynamic array and queue formal arguments also govern the behavior of unpacked
dimensions of DPI open array formal arguments (see 7.6). DPI open arrays can also have a solitary unsized,
packed dimension (see 34.5.6.1). A dynamic array or queue shall not be passed as an actual argument if the
DPI formal argument has unsized dimensions and an output direction mode.
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
155
Copyright © 2018 IEEE. All rights reserved.
### 7.8 Associative arrays

Dynamic arrays are useful for dealing with contiguous collections of variables whose number changes
dynamically. When the size of the collection is unknown or the data space is sparse, an associative array is a
better option. Associative arrays do not have any storage allocated until it is used, and the index expression
is not restricted to integral expressions, but can be of any type.
An associative array implements a lookup table of the elements of its declared type. The data type to be used
as an index serves as the lookup key and imposes an ordering.
The syntax to declare an associative array is as follows:
data_type array_id [ index_type ];
where
data_type is the data type of the array elements. Can be any type allowed for fixed-size arrays.
array_id is the name of the array being declared.
index_type is the data-type to be used as an index or is *. If * is specified, then the array is indexed by
any integral expression of arbitrary size. An index type restricts the indexing expressions to a
particular type. It shall be illegal for index_type to declare a type.
Examples of associative array declarations are as follows:
integer i_array[*];
// associative array of integer (unspecified
// index)
bit [20:0] array_b[string];
// associative array of 21-bit vector,
// indexed by string
event ev_array[myClass];
// associative array of event indexed by class
// myClass
Array elements in associative arrays are allocated dynamically. An entry for a nonexistent associative array
element shall be allocated when it is used as the target of an assignment or actual to an argument passed by
reference. The associative array maintains the entries that have been assigned values and their relative order
according to the index data type. Associative array elements are unpacked. In other words, other than for
copying or comparing arrays, an individual element must be selected out of the array before it can be used in
most expressions.
#### 7.8.1 Wildcard index type

For example:
int array_name [*];
Associative arrays that specify a wildcard index type have the following properties:
—
The array may be indexed by any integral expression. Because the index expressions may be of
different sizes, the same numerical value can have multiple representations, each of a different size.
SystemVerilog resolves this ambiguity by removing the leading zeros and computing the minimal
length and using that representation for the value.
—
Nonintegral index values are illegal and result in an error.
—
A 4-state index value containing X or Z is invalid.
—
Indexing expressions are self-determined and treated as unsigned.
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
156
Copyright © 2018 IEEE. All rights reserved.
—
A string literal index is automatically cast to a bit vector of equivalent size.
—
The ordering is numerical (smallest to largest).
—
Associative arrays that specify a wildcard index type shall not be used in a foreach loop (see
12.7.3) or with an array manipulation method (see 7.12) that returns an index value or array of
values.
#### 7.8.2 String index

For example:
int array_name [ string ];
Associative arrays that specify a string index have the following properties:
—
Indices can be strings or string literals of any length. Other types are illegal and shall result in a type
check error.
—
An empty string "" index is valid.
—
The ordering is lexicographical (lesser to greater).
#### 7.8.3 Class index

For example:
int array_name [ some_Class ];
Associative arrays that specify a class index have the following properties:
—
Indices can be objects of that particular type or derived from that type. Any other type is illegal and
shall result in a type check error.
—
A null index is valid.
—
The ordering is deterministic but arbitrary.
#### 7.8.4 Integral index

For example:
int array_name1 [ integer ];
typedef bit signed [4:1] SNibble;
int array_name2 [ SNibble ];
typedef bit [4:1] UNibble;
int array_name3 [ UNibble ];
Associative arrays that specify an index of integral data type shall have the following properties:
—
The index expression shall be evaluated in terms of a cast to the index type, except that an implicit
cast from a real or shortreal data type shall be illegal.
—
A 4-state index expression containing X or Z is invalid.
—
The ordering is signed or unsigned numerical, depending on the signedness of the index type.
#### 7.8.5 Other user-defined types

For example:
typedef struct {byte B; int I[*];} Unpkt;
int array_name [ Unpkt ];
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
157
Copyright © 2018 IEEE. All rights reserved.
In general, associative arrays that specify an index of any type have the following properties:
—
Declared indices shall have the equality operator defined for its type to be legal. This includes all of
the dynamically sized types as legal index types. However, real or shortreal data types, or a
type containing a real or shortreal, shall be an illegal index type.
—
An index expression that is or contains X or Z in any of its elements is invalid.
—
An index expression that is or contains an empty value or null for any of its elements does not
make the index invalid.
—
If the relational operator is defined for the index type, the ordering is as defined in the preceding
clauses. If not, the relative ordering of any two entries in such an associative array can vary, even
between successive runs of the same tool. However, the relative ordering shall remain the same
within the same simulation run while no indices have been added or deleted.
#### 7.8.6 Accessing invalid indices

If a read operation uses an index that is a 4-state expression with one or more x or z bits, or an attempt is
made to read a nonexistent entry, then a warning shall be issued and the nonexistent entry value for the array
type shall be returned, as shown in Table 7-1 (see 7.4.6). A user-specified default shall not issue a warning
and returns the value specified in 7.9.11.
If an invalid index is used during a write operation, the write shall be ignored, and a warning shall be issued.
#### 7.8.7 Allocating associative array elements

An entry for a nonexistent associative array element shall be allocated when it is used as the target of an
assignment or actual to an argument passed by reference. Some constructs perform both a read and write
operation as part of a single statement, such as with an increment operation. In those cases, the nonexistent
element shall be allocated with its default or user-specified initial value before any reference to that element.
For example:
int a[int] = '{default:1};
typedef struct { int x=1,y=2; } xy_t;
xy_t b[int];
begin
a[1]++;
b[2].x = 5;
end
Assume the references to a[1] and b[2] are nonexistent elements before these statements execute. Upon
executing a[1]++, a[1] will be allocated and initialized to 1. After the increment, a[1] will be 2. Upon
executing b[2].x = 5, b[2] will be allocated and b[2].x will be 1 and b[2].y will be 2. After executing
the assignment, b[2].x will be updated to 5.
### 7.9 Associative array methods

In addition to the indexing operators, several built-in methods are provided, which allow users to analyze
and manipulate associative arrays, as well as iterate over its indices or keys.
#### 7.9.1 Num() and size()

The syntax for the num() and size() methods is as follows:
function int num();
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
158
Copyright © 2018 IEEE. All rights reserved.
function int size();
The num() and size() methods return the number of entries in the associative array. If the array is empty,
they return 0.
int imem[int];
imem[ 3 ] = 1;
imem[ 16'hffff ] = 2;
imem[ 4'b1000 ] = 3;
$display( "%0d entries\n", imem.num );
// prints "3 entries"
#### 7.9.2 Delete()

The syntax for the delete() method is as follows:
function void delete( [input index] );
where index is an optional index of the appropriate type for the array in question.
If the index is specified, then the delete() method removes the entry at the specified index. If the entry to
be deleted does not exist, the method issues no warning.
If the index is not specified, then the delete() method removes all the elements in the array.
int map[ string ];
map[ "hello" ] = 1;
map[ "sad" ] = 2;
map[ "world" ] = 3;
map.delete( "sad" );
// remove entry whose index is "sad" from "map"
map.delete;
// remove all entries from the associative array "map"
#### 7.9.3 Exists()

The syntax for the exists() method is as follows:
function int exists( input index );
where index is an index of the appropriate type for the array in question.
The exists() function checks whether an element exists at the specified index within the given array. It
returns 1 if the element exists; otherwise, it returns 0.
if ( map.exists( "hello" ))
map[ "hello" ] += 1;
else
map[ "hello" ] = 0;
#### 7.9.4 First()

The syntax for the first() method is as follows:
function int first( ref index );
where index is an index of the appropriate type for the array in question. Associative arrays that specify a
wildcard index type shall not be allowed.
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
159
Copyright © 2018 IEEE. All rights reserved.
The first() method assigns to the given index variable the value of the first (smallest) index in the
associative array. It returns 0 if the array is empty; otherwise, it returns 1.
string s;
if ( map.first( s ) )
$display( "First entry is : map[ %s ] = %0d\n", s, map[s] );
#### 7.9.5 Last()

The syntax for the last() method is as follows:
function int last( ref index );
where index is an index of the appropriate type for the array in question. Associative arrays that specify a
wildcard index type shall not be allowed.
The last() method assigns to the given index variable the value of the last (largest) index in the associative
array. It returns 0 if the array is empty; otherwise, it returns 1.
string s;
if ( map.last( s ) )
$display( "Last entry is : map[ %s ] = %0d\n", s, map[s] );
#### 7.9.6 Next()

The syntax for the next() method is as follows:
function int next( ref index );
where index is an index of the appropriate type for the array in question. Associative arrays that specify a
wildcard index type shall not be allowed.
The next() method finds the smallest index whose value is greater than the given index argument.
If there is a next entry, the index variable is assigned the index of the next entry, and the function returns 1.
Otherwise, the index is unchanged, and the function returns 0.
string s;
if ( map.first( s ) )
do
$display( "%s : %d\n", s, map[ s ] );
while ( map.next( s ) );
#### 7.9.7 Prev()

The syntax for the prev() method is as follows:
function int prev( ref index );
where index is an index of the appropriate type for the array in question. Associative arrays that specify a
wildcard index type shall not be allowed.
The prev() function finds the largest index whose value is smaller than the given index argument. If there
is a previous entry, the index variable is assigned the index of the previous entry, and the function returns 1.
Otherwise, the index is unchanged, and the function returns 0.
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
160
Copyright © 2018 IEEE. All rights reserved.
string s;
if ( map.last( s ) )
do
$display( "%s : %d\n", s, map[ s ] );
while ( map.prev( s ) );
#### 7.9.8 Arguments to traversal methods

The argument that is passed to any of the four associative array traversal methods first(), last(),
next(), and prev() shall be assignment compatible with the index type of the array. If the argument has
an integral type that is smaller than the size of the corresponding array index type, then the function returns
–1 and shall truncate in order to fit into the argument. For example:
string
aa[int];
byte
ix;
int
status;
aa[ 1000 ] = "a";
status = aa.first( ix );
// status is –1
// ix is 232 (least significant 8 bits of 1000)
#### 7.9.9 Associative array assignment

Associative arrays can be assigned only to another associative array of a compatible type and with the same
index type. Other types of arrays cannot be assigned to an associative array, nor can associative arrays be
assigned to other types of arrays, whether fixed-size or dynamic.
Assigning an associative array to another associative array causes the target array to be cleared of any
existing entries, and then each entry in the source array is copied into the target array.
#### 7.9.10 Associative array arguments

Associative arrays can be passed as arguments only to associative arrays of a compatible type and with the
same index type. Other types of arrays, whether fixed-size or dynamic, cannot be passed to subroutines that
accept an associative array as an argument. Likewise, associative arrays cannot be passed to subroutines that
accept other types of arrays.
Passing an associative array by value causes a local copy of the associative array to be created.
#### 7.9.11 Associative array literals

Associative array literals use the '{index:value} syntax with an optional default index. Like all other
arrays, an associative array can be written one entry at a time, or the whole array contents can be replaced
using an array literal.
For example:
// an associative array of strings indexed by 2-state integers,
// default is "hello".
string words [int] = '{default: "hello"};
// an associative array of 4-state integers indexed by strings, default is –1
integer tab [string] = '{"Peter":20, "Paul":22, "Mary":23, default:-1 };
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
161
Copyright © 2018 IEEE. All rights reserved.
If a default value is specified, then reading a nonexistent element shall yield the specified default value, and
no warning shall be issued. Otherwise, the value specified by Table 7-1 (see 7.4.6) shall be returned.
Defining a default value shall not affect the operation of the associative array methods (see 7.9).
### 7.10 Queues

A queue is a variable-size, ordered collection of homogeneous elements. A queue supports constant-time
access to all its elements as well as constant-time insertion and removal at the beginning or the end of the
queue. Each element in a queue is identified by an ordinal number that represents its position within the
queue, with 0 representing the first, and $ representing the last. A queue is analogous to a one-dimensional
unpacked array that grows and shrinks automatically. Thus, like arrays, queues can be manipulated using the
indexing, concatenation, slicing operator syntax, and equality operators.
Queues are declared using the same syntax as unpacked arrays, but specifying $ as the array size. The
maximum size of a queue can be limited by specifying its optional right bound (last index).
Queue values may be written using assignment patterns or unpacked array concatenations (see 10.9, 10.10).
The syntax for declaring queues is shown in Syntax 7-4.
```ebnf
variable_dimension ::=
```

// from A.2.5
unsized_dimension
| unpacked_dimension
| associative_dimension
| queue_dimension
```ebnf
queue_dimension ::= [ $ [ : constant_expression ] ]
```

Syntax 7-4—Declaration of queue dimension (excerpt from Annex A)
constant_expression shall evaluate to a positive integer value.
For example:
byte q1[$];
// A queue of bytes
string names[$] = { "Bob" };
// A queue of strings with one element
integer Q[$] = { 3, 2, 7 };
// An initialized queue of integers
bit q2[$:255];
// A queue whose maximum size is 256 bits
If an initial value is not provided in the declaration, the queue variable is initialized to the empty queue. The
empty queue can be denoted by an empty unpacked array concatenation {}, as described in 10.10.
#### 7.10.1 Queue operators

Queues shall support the same operations that can be performed on unpacked arrays. In addition, queues
shall support the following operations:
—
A queue shall resize itself to accommodate any queue value that is written to it, except that its
maximum size may be bounded as described in 7.10.
—
In a queue slice expression such as Q[a:b], the slice bounds may be arbitrary integral expressions
and, in particular, are not required to be constant expressions.
—
Queues shall support methods as described in 7.10.2.
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
162
Copyright © 2018 IEEE. All rights reserved.
Unlike arrays, the empty queue, {}, is a valid queue and the result of some queue operations. The following
rules govern queue operators:
—
Q[ a : b ] yields a queue with b - a + 1 elements.
•
If a > b, then Q[a:b] yields the empty queue {}.
•
Q[ n : n ] yields a queue with one item, the one at position n. Thus, Q[ n : n ] === {
Q[n] }.
•
If n lies outside Q’s range (n < 0 or n > $), then Q[n:n] yields the empty queue {}.
•
If either a or b are 4-state expressions containing X or Z values, it yields the empty queue {}.
—
Q[ a : b ] where a < 0 is the same as Q[ 0 : b ].
—
Q[ a : b ] where b > $ is the same as Q[ a : $ ].
—
An invalid index value (i.e., a 4-state expression whose value has one or more x or z bits, or a value
that lies outside 0...$) shall cause a read operation to return the value appropriate for a nonexistent
array entry of the queue’s element type (as described in Table 7-1 in 7.4.6).
—
An invalid index (i.e., a 4-state expression with X’s or Z’s, or a value that lies outside 0...$+1) shall
cause a write operation to be ignored and a run-time warning to be issued; however, writing to
Q[$+1] is legal.
—
A queue declared with a right bound using the syntax [$:N] is known as a bounded queue and shall
be limited to have indices not greater than N (its size shall not exceed N+1). The additional rules
governing bounded queues are described in 7.10.5.
#### 7.10.2 Queue methods

In addition to the array operators, queues provide several built-in methods. Assume these declarations for
the examples that follow:
typedef mytype element_t;
// mytype is any legal type for a queue
typedef element_t queue_t[$];
element_t e;
queue_t Q;
int i;
##### 7.10.2.1 Size()

The prototype for the size() method is as follows:
function int size();
The size() method returns the number of items in the queue. If the queue is empty, it returns 0.
for ( int j = 0; j < Q.size; j++ ) $display( Q[j] );
##### 7.10.2.2 Insert()

The prototype of the insert() method is as follows:
function void insert(input integer index, input element_t item);
The insert() method inserts the given item at the specified index position.
If the index argument has any bits with unknown (x/z) value, or is negative, or is greater than the current
size of the queue, then the method call shall have no effect on the queue and may cause a warning to be
issued.
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
163
Copyright © 2018 IEEE. All rights reserved.
NOTE—The index argument is of type integer rather than int so that x/z values in the caller’s actual argument
value can be detected.
##### 7.10.2.3 Delete()

The prototype for the delete() method is as follows:
function void delete( [input integer index] );
where index is an optional index.
If the index is not specified, then the delete() method deletes all the elements in the queue, leaving the
queue empty.
If the index is specified, then the delete() method deletes the item at the specified index position. If the
index argument has any bits with unknown (x/z) value, or is negative, or is greater than or equal to the
current size of the queue, then the method call shall have no effect on the queue and may cause a warning to
be issued.
##### 7.10.2.4 Pop_front()

The prototype of the pop_front() method is as follows:
function element_t pop_front();
The pop_front() method removes and returns the first element of the queue.
If this method is called on an empty queue:
—
Its return value shall be the same as that obtained by attempting to read a nonexistent array element
of the same type as the queue's elements (as described in Table 7-1, in 7.4.6);
—
It shall have no effect on the queue and may cause a warning to be issued.
##### 7.10.2.5 Pop_back()

The prototype of the pop_back() method is as follows:
function element_t pop_back();
The pop_back() method removes and returns the last element of the queue.
If this method is called on an empty queue:
—
Its return value shall be the same as that obtained by attempting to read a nonexistent array element
of the same type as the queue's elements (as described in Table 7-1 in 7.4.6);
—
It shall have no effect on the queue and may cause a warning to be issued.
##### 7.10.2.6 Push_front()

The prototype of the push_front() method is as follows:
function void push_front(input element_t item);
The push_front() method inserts the given element at the front of the queue.
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
164
Copyright © 2018 IEEE. All rights reserved.
##### 7.10.2.7 Push_back()

The prototype of the push_back() method is as follows:
function void push_back(input element_t item);
The push_back() method inserts the given element at the end of the queue.
#### 7.10.3 Persistence of references to elements of a queue

As described in 13.5.2, it is possible for an element of a queue to be passed by reference to a task that
continues to hold the reference while other operations are performed on the queue. Some operations on the
queue shall cause any such reference to become outdated (as defined in 13.5.2). This subclause defines the
situations in which a reference to a queue element shall become outdated.
When any of the queue methods described in 7.10.2 updates a queue, a reference to any existing element that
is not deleted by the method shall not become outdated. All elements that are removed from the queue by the
method shall become outdated references.
When the target of an assignment is an entire queue, references to any element of the original queue shall
become outdated.
As a consequence of this clause, inserting elements in a queue using unpacked array concatenation syntax,
as illustrated in the examples in 7.10.4, will cause all references to any element of the existing queue to
become outdated. Use of the delete, pop_front, and pop_back methods will outdate any reference to the
popped or deleted element, but will leave references to all other elements of the queue unaffected. By
contrast, use of the insert, push_back, and push_front methods on a queue can never give rise to
outdated references (except that insert or push_front on a bounded queue would cause the highest-
numbered element of the queue to be deleted if the new size of the queue were to exceed the queue’s bound).
#### 7.10.4 Updating a queue using assignment and unpacked array concatenation

As described in 7.10, a queue variable may be updated by assignment. Together with unpacked array
concatenation, this offers a flexible alternative to the queue methods described in 7.10.2 when performing
operations on a queue variable.
The following examples show queue assignment operations that exhibit behaviors similar to those of queue
methods. In each case the resulting value of the queue variable shall be the same as if the queue method had
been applied, but any reference to elements of the queue will become outdated by the assignment operation
(see 7.10.3):
int q[$] = { 2, 4, 8 };
int e, pos;
// assignment
// method call yielding the
//
// same value in variable q
// ----------------------------- // -------------------------
q = { q, 6 };
// q.push_back(6)
q = { e, q };
// q.push_front(e)
q = q[1:$];
// void'(q.pop_front()) or q.delete(0)
q = q[0:$-1];
// void'(q.pop_back()) or
// q.delete(q.size-1)
q = { q[0:pos-1], e, q[pos:$] }; // q.insert(pos, e)
q = { q[0:pos], e, q[pos+1:$] }; // q.insert(pos+1, e)
q = {};
// q.delete()
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
165
Copyright © 2018 IEEE. All rights reserved.
Some useful operations that cannot be implemented as a single queue method call are illustrated in the
following examples. As in the preceding examples, assignment to the queue variable outdates any reference
to its elements.
q = q[2:$];
// a new queue lacking the first two items
q = q[1:$-1];
// a new queue lacking the first and last items
#### 7.10.5 Bounded queues

A bounded queue shall not have an element whose index is higher than the queue’s declared upper bound.
Operations on bounded queues shall behave exactly as if the queue were unbounded except that if, after any
operation that writes to a bounded queue variable, that variable has any elements beyond its bound, then all
such out-of-bounds elements shall be discarded and a warning shall be issued.
NOTE—Implementations may meet this requirement in any way that achieves the same result. In particular, they are not
required to write the out-of-bounds elements before discarding them.
### 7.11 Array querying functions

SystemVerilog provides system functions to return information about an array. These are $left, $right,
$low, $high, $increment, $size, $dimensions, and $unpacked_dimensions. These functions are
described in 20.7.
### 7.12 Array manipulation methods

SystemVerilog provides several built-in methods to facilitate array searching, ordering, and reduction.
The general syntax to call these array methods is as follows:
```ebnf
array_method_call ::=
```

expression . array_method_name { attribute_instance } [ ( iterator_argument ) ]
[ with ( expression ) ]
Syntax 7-5—Array method call syntax (not in Annex A)
The optional with clause accepts an expression enclosed in parentheses. In contrast, the with clause used
by the randomize method (see 18.7) accepts a set of constraints enclosed in braces.
If the expression contained in the with clause includes any side effects, the results may be unpredictable.
Array manipulation methods iterate over the array elements, which are then used to evaluate the expression
specified by the with clause. The iterator_argument optionally specifies the name of the variable used by
the with expression to designate the element of the array at each iteration. If it is not specified, the name
item is used by default. The scope for the iterator_argument is the with expression. Specifying an
iterator_argument without also specifying a with clause shall be illegal.
#### 7.12.1 Array locator methods

Array locator methods operate on any unpacked array, including queues, but their return type is a queue.
These locator methods allow searching an array for elements (or their indices) that satisfy a given
expression. Array locator methods traverse the array in an unspecified order.
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
166
Copyright © 2018 IEEE. All rights reserved.
Index locator methods return a queue of int for all arrays except associative arrays, which return a queue of
the same type as the associative index type. Associative arrays that specify a wildcard index type shall not be
allowed.
If no elements satisfy the given expression or the array is empty (in the case of a queue or dynamic array),
then an empty queue is returned. Otherwise, these methods return a queue containing all items that satisfy
the expression. Index locator methods return a queue with the indices of all items that satisfy the expression.
The optional expression specified by the with clause shall evaluate to a Boolean value.
The following locator methods are supported (the with clause is mandatory):
—
find() returns all the elements satisfying the given expression.
—
find_index() returns the indices of all the elements satisfying the given expression.
—
find_first() returns the first element satisfying the given expression.
—
find_first_index() returns the index of the first element satisfying the given expression.
—
find_last() returns the last element satisfying the given expression.
—
find_last_index() returns the index of the last element satisfying the given expression.
The first or last element is defined as being closest to the leftmost or rightmost indexed element,
respectively, except for an associative array, which shall use the element closest to the index returned by the
first or last method for the associative array index type.
For the following locator methods, the with clause (and its expression) may be omitted if the relational
operators (<, >, ==) are defined for the element type of the given array. If a with clause is specified, the
relational operators (<, >, ==) shall be defined for the type of the expression.
—
min() returns the element with the minimum value or whose expression evaluates to a minimum.
—
max() returns the element with the maximum value or whose expression evaluates to a maximum.
—
unique() returns all elements with unique values or whose expression evaluates to a unique value.
The queue returned contains one and only one entry for each of the values found in the array. The
ordering of the returned elements is unrelated to the ordering of the original array.
—
unique_index() returns the indices of all elements with unique values or whose expression
evaluates to a unique value. The queue returned contains one and only one entry for each of the
values found in the array. The ordering of the returned elements is unrelated to the ordering of the
original array. The index returned for duplicate valued entries may be the index for one of the
duplicates.
Examples:
string SA[10], qs[$];
int IA[int], qi[$];
// Find all items greater than 5
qi = IA.find( x ) with ( x > 5 );
qi = IA.find( x );
// shall be an error
// Find indices of all items equal to 3
qi = IA.find_index with ( item == 3 );
// Find first item equal to Bob
qs = SA.find_first with ( item == "Bob" );
// Find last item equal to Henry
qs = SA.find_last( y ) with ( y == "Henry" );
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
167
Copyright © 2018 IEEE. All rights reserved.
// Find index of last item greater than Z
qi = SA.find_last_index( s ) with ( s > "Z" );
// Find smallest item
qi = IA.min;
// Find string with largest numerical value
qs = SA.max with ( item.atoi );
// Find all unique string elements
qs = SA.unique;
// Find all unique strings in lowercase
qs = SA.unique( s ) with ( s.tolower );
#### 7.12.2 Array ordering methods

Array ordering methods reorder the elements of any unpacked array (fixed or dynamically sized) except for
associative arrays.
The prototype for the ordering methods is as follows:
function void ordering_method ( array_type iterator = item );
The following ordering methods are supported:
—
reverse() reverses the order of the elements in the array. Specifying a with clause shall be a com-
piler error.
—
sort() sorts the array in ascending order, optionally using the expression in the with clause. The
with clause (and its expression) is optional when the relational operators (<, >, ==) are defined for
the array element type. If a with clause is specified, the relational operators (<, >, ==) shall be
defined for the type of the expression.
—
rsort() sorts the array in descending order, optionally using the expression in the with clause.
The with clause (and its expression) is optional when the relational operators (<, >, ==) are defined
for the array element type. If a with clause is specified, the relational operators (<, >, ==) shall be
defined for the type of the expression.
—
shuffle() randomizes the order of the elements in the array. Specifying a with clause shall be a
compiler error.
Examples:
string s[] = { "hello", "sad", "world" };
s.reverse;
// s becomes { "world", "sad", "hello" };
int q[$] = { 4, 5, 3, 1 };
q.sort;
// q becomes { 1, 3, 4, 5 }
struct { byte red, green, blue; } c [512];
c.sort with ( item.red );
// sort c using the red field only
c.sort( x ) with ( {x.blue, x.green} );
// sort by blue then green
#### 7.12.3 Array reduction methods

Array reduction methods may be applied to any unpacked array of integral values to reduce the array to a
single value. The expression within the optional with clause is used to specify the values to use in the
reduction. The values produced by evaluating this expression for each array element are used by the
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
168
Copyright © 2018 IEEE. All rights reserved.
reduction method. This is in contrast to the array locator methods (see 7.12.1) where the with clause is used
as a selection criteria.
The prototype for these methods is as follows:
function expression_or_array_type reduction_method (array_type iterator = item);
The method returns a single value of the same type as the array element type or, if specified, the type of the
expression in the with clause. The with clause may be omitted if the corresponding arithmetic or Boolean
reduction operation is defined for the array element type. If a with clause is specified, the corresponding
arithmetic or Boolean reduction operation shall be defined for the type of the expression.
The following reduction methods are supported:
—
sum() returns the sum of all the array elements or, if a with clause is specified, returns the sum of
the values yielded by evaluating the expression for each array element.
—
product() returns the product of all the array elements or, if a with clause is specified, returns the
product of the values yielded by evaluating the expression for each array element.
—
and() returns the bitwise AND ( & ) of all the array elements or, if a with clause is specified,
returns the bitwise AND of the values yielded by evaluating the expression for each array element.
—
or() returns the bitwise OR ( | ) of all the array elements or, if a with clause is specified, returns the
bitwise OR of the values yielded by evaluating the expression for each array element.
—
xor() returns the bitwise XOR ( ^ ) of all the array elements or, if a with clause is specified,
returns the bitwise XOR of the values yielded by evaluating the expression for each array element.
Examples:
byte b[] = { 1, 2, 3, 4 };
int y;
y = b.sum ;
// y becomes 10 => 1 + 2 + 3 + 4
y = b.product ;
// y becomes 24 => 1 * 2 * 3 * 4
y = b.xor with ( item + 4 );
// y becomes 12 => 5 ^ 6 ^ 7 ^ 8
logic [7:0] m [2][2] = '{ '{5, 10}, '{15, 20} };
int y;
y = m.sum with (item.sum with (item)); // y becomes 50 => 5+10+15+20
logic bit_arr [1024];
int y;
y = bit_arr.sum with ( int'(item) );
// forces result to be 32-bit
The last example shows how the result of calling sum on a bit array can be forced to be a 32-bit quantity. By
default, the result of calling sum would be of type logic in this example. Summing the values of 1024 bits
could overflow the result. This overflow can be avoided by using a with clause. When specified, the with
clause is used to determine the type of the result. Casting item to an int in the with clause causes the
array elements to be extended to 32 bits before being summed. The result of calling sum in this example is
## 32 bits since the width of the reduction method result shall be the same as the width of the expression in the

with clause.
#### 7.12.4 Iterator index querying

The expressions used by array manipulation methods sometimes need the actual array indices at each
iteration, not just the array element. The index method of an iterator returns the index value of the specified
dimension. The prototype of the index method is as follows:
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1800-2017
IEEE Standard for SystemVerilog—Unified Hardware Design, Specification, and Verification Language
169
Copyright © 2018 IEEE. All rights reserved.
function int_or_index_type index ( int dimension = 1 );
The array dimensions are numbered as defined in 20.7. The slowest varying is dimension 1. Successively
faster varying dimensions have sequentially higher dimension numbers. If the dimension is not specified, the
first dimension is used by default.
The return type of the index method is an int for all array iterator items except associative arrays, which
return an index of the same type as the associative index type. Associative arrays that specify a wildcard
index type shall not be allowed.
For example:
int arr[];
int q[$];
...
// find all items equal to their position (index)
q = arr.find with ( item == item.index );
Authorized licensed use limited to: Richard DJE. Downloaded on April 22,2021 at 14:18:32 UTC from IEEE Xplore.  Restrictions apply.
