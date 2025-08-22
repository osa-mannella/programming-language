# n Bytecode v1

## 1. FILE STRUCTURE

```
[HEADER]
[CONSTANT TABLE]
[FUNCTION TABLE]
[ENUM TABLE]
[INSTRUCTION STREAM]
```

## 2. HEADER (8 bytes)

- Magic number (2 bytes) : "NB"
- Version (uint16) : e.g., 1
- Flags (uint16) : reserved for future use

**Example:**

```
4D 49 52 42 00 01 00 00
```

## 3. CONSTANT TABLE

- Count (uint16)
- For each constant:
  - Type (uint8)
    0 = String
    1 = Number (double)
    2 = Boolean
    3 = Null
  - Length (uint16) [strings only]
  - Value (raw bytes)

**Example constants:**

```
count = 2
0: string "Hi"
1: number 42
```

## 4. FUNCTION TABLE

- Count (uint16)
- For each function:
  - Name index (uint16) : index in constants, or 0xFFFF for anonymous
  - Arg count (uint8)
  - Local count (uint8)
  - Offset (uint32) : byte offset into instruction stream

**Example function table:**

```
function 0 → name="hello", argc=0, locals=0, offset=12
```

## 5. ENUM TABLE

- **Count** (uint16)
- For each enum:

  - **Name index** (uint16) : index in constant table (the enum’s name)
  - **Variant count** (uint8) : number of variants
  - **Descriptor offset** (uint32) : byte offset into the variant‐descriptor region

### Variant Descriptor Region

Following all enum entries, a flat list of variant descriptors for each enum in order:

- For each variant:

  - **Variant name index** (uint16)
  - **Field count** (uint8)

## 6. INSTRUCTION STREAM

A flat sequence of opcodes and operands. Each instruction is encoded as:

- **Opcode** (1 byte, uint8) — selects the operation
- **Operands** — zero or more fields immediately following the opcode, in the order shown below

  - **uint8** (1 byte)
  - **uint16** (2 bytes, little-endian)
  - **uint32** (4 bytes, little-endian)

## 7. INSTRUCTIONS (v1)

### Constants & Variables

- `0x01` LOAD_CONST index
- `0x02` LOAD_GLOBAL index
- `0x03` STORE_GLOBAL index
- `0x04` LOAD_LOCAL index
- `0x05` STORE_LOCAL index

### Arithmetic & Logic

- `0x10` ADD
- `0x11` SUB
- `0x12` MUL
- `0x13` DIV
- `0x14` EQUAL
- `0x15` LESS
- `0x16` GREATER

### Control Flow

- `0x20` JUMP offset
- `0x21` JUMP_IF_FALSE offset

### Functions

- `0x30` CALL index argc
- `0x31` CALL_GLOBAL index argc
- `0x32` RETURN

### Stack

- `0x40` POP
- `0x41` DUP

### Misc

- `0xFE` PRINT ; temporary debug
- `0xFF` HALT

## EXAMPLE

**Source:**

```n
func hello() {
    print("Hi")
}
hello()
```

**Constants:**

```
0: "Hi"
```

**Functions:**

```
0: hello offset=12 argc=0 locals=0
```

**Instructions:**

```
; hello() @ offset 12
12: LOAD_CONST 0
14: CALL_GLOBAL 0 1 ; print, 1 arg
17: RETURN

; main
20: CALL 0 0 ; hello()
23: HALT
```
