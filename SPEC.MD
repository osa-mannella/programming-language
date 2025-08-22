n Language

Here I will quickly denote what is needed for the byte code setup

Bytecode Spec

Miscellaneous Specifications
The byte code will initially be made up of two passes, one will collect function definitions (to be expanded later on) and constants, the next will generate the instructions for the given ASTNodes it comes across. The byte code will consist of an array of different instructions, most of which simple in nature. Given the dynamic nature of functions they will be handled in a special way, they will be parsed on first pass where they will be the forefront of the byte code within the header, in addition their name will be stored inside a temporary transient hash map. Upon the second pass when a function is being compiled it will work as so: It will look up the name of the function in the hash map which will give the respective index of the function inside the header. We will then set the current byte offset as the indicated offset of the function per the signature. This way we optimize for runtime where we can go about exact jumps around the byte code that have been predefined.

With respect to the VM and scoping we will use a simple 2 dimensional array system in order to create a stack frame system at runtime. The compilation step shouldn’t be complicated by this as it is the interpreter which handles the STORE_LOCAL byte code instruction.

Bytecode Specifics

Below is a table outlining the respective byte code instructions inside of n

With respect to indexes the index is defined by its placement inside the constant/function etc. table.

That is with an exception for STORE_VAR and LOAD_VAR which reference a runtime only variable table. At compile time the variable name is parsed and added to a temporary table in order to keep these dynamic indexes smaller. Given the complexity of this I will provide an example 
`let x = 42`
 LOAD_CONST 0 // Load 42 onto the runtime stack
STORE_VAR 0 // The identifier x is given an entry in a compile time table as 0. Later if that same identifier is referenced the same number is used. We then use an array at runtime and the encoded number from compile time is used as our index

In short: In a linear order at compile time our variable names are mapped to usize’s which are used at runtime as array indexes.

// Result: O(1) array access at runtime instead of O(1) hashmap lookup
// Plus smaller bytecode (no string constants for variable names)
STORE_VAR 0x01 <index>
LOAD_VAR 0x02 <index>
CALL 0x03 <index>
RETURN 0x04
LOAD_CONST 0x05 <index>
ADD 0x10
SUB 0x11
DIV 0x12
MUL 0x13
EQUAL 0x14
LESS 0x15
GREATER 0x16
JUMP 0x20
JUMP_IF_FALSE 0x21
JUMP_IF_TRUE 0x22
POP 0x30
DUP 0x31
HALT 0x32

The initial byte code set is simple in nature but should be ample in order to begin writing an initial byte code set for n that will be expandable in the future. There are some OPCODEs I have omitted, a CALL_GLOBAL OPCODE may be in effect in a later version in order to allow for maximum runtime speed for calling native global functions instead of relying on a runtime fallback system.
