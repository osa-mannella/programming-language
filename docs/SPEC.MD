# n Language

Here I will quickly denote what is needed for the byte code setup

## Bytecode Spec

### Miscellaneous Specifications
The byte code will initially be made up of two passes, one will collect function definitions (to be expanded later on) and constants, the next will generate the instructions for the given ASTNodes it comes across. The byte code will consist of an array of different instructions, most of which simple in nature. Given the dynamic nature of functions they will be handled in a special way, they will be parsed on first pass where they will be the forefront of the byte code within the header, in addition their name will be stored inside a temporary transient hash map. Upon the second pass when a function is being compiled it will work as so: It will look up the name of the function in the hash map which will give the respective index of the function inside the header. We will then set the current byte offset as the indicated offset of the function per the signature. This way we optimize for runtime where we can go about exact jumps around the byte code that have been predefined.

With respect to the VM and scoping we will use a simple 2 dimensional array system in order to create a stack frame system at runtime. The compilation step shouldn’t be complicated by this as it is the interpreter which handles the STORE_LOCAL byte code instruction.

## Bytecode Specifics

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
LOAD_ARG 0x03 <count>
CALL 0x04 <index>
RETURN 0x05
LOAD_CONST 0x06 <index>
ADD 0x10
SUB 0x11
DIV 0x12
MUL 0x13
EQUAL 0x14
LESS 0x15
GREATER 0x16
JUMP 0x20 <index>
JUMP_IF_FALSE 0x21 <index>
JUMP_IF_TRUE 0x22 <index>
POP 0x30
DUP 0x31
HALT 0x32

The initial byte code set is simple in nature but should be ample in order to begin writing an initial byte code set for n that will be expandable in the future. There are some OPCODEs I have omitted, a CALL_GLOBAL OPCODE may be in effect in a later version in order to allow for maximum runtime speed for calling native global functions instead of relying on a runtime fallback system.

## Interpreter

With respect to the n language interpreter I want to try and keep it lean to begin, which likely says very little given such has likely been the thought of everyone who has ever decided upon writing a programming language of a fair scope. Nonetheless needless to say the VM interpreter will be entirely stack based opposed to a register based approach given the logistics of a register based approach would not provide a meaningful benefit for n. We will use a stack frame for variables and store their array index useize, revisit the line above with respect to mapping the variables to array indexes, that is what we will store in the stack frame. When we see a referenced variable we will do 2 O(n) checks on the last entry in our stackframe (to be clear our stack frame will be a 2 dimensional) array and our first entry. The first would be the global score and the last would be the local scope. That way scoping will work appropriately.

A lot of the instructions are pretty self explanatory, for example POP will pop an item off our runtime stack, DUP will duplicate it, HALT terminates the program, GREATER checks whether the 2nd last item on the stack is greater than the last item on the stack, same for LESS but measures if it is less than. Equal of course checks if the last 2 items on the stack are equal and pushes the result onto the stack (as do the former comparative expressions I mentioned), MUL will multiply the last 2 and push the product onto the stack, DIV same thing but for the quotient, SUB same thing but for the difference and ADD same thing but for the sum. LOAD_CONST will refer to the compiled constant table, nothing too complex there. Return will pop the current stack frame, it will also refer to the last entry in the return pointer vector we created with the opcode offset of the position where the function was called. STORE_VAR will create an entry in the current stack frame, to be clear the index after STORE_VAR is going to be the index of the actual value.

LOAD_ARG is a specialized instruction for function parameter binding. It takes a count parameter specifying the number of arguments to pop from the stack. These arguments are then bound to local variable indices 0 through count-1 in the current stack frame. This ensures each function gets a fresh parameter space starting from index 0, resolving variable scoping issues with nested functions. Arguments are processed in reverse order (last argument popped becomes parameter 0) to maintain proper parameter ordering from the function call.
