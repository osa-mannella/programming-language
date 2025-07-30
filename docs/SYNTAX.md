# Mirrow Language Syntax

## Variables

### Declaration

```mirrow
let x = 5   // immutable, always
let y = 10  // everything is immutable by default
```

### Naming Rules

- Letters, numbers, and underscores allowed.
- Cannot start with a number.
- Case-sensitive (`value` != `Value`).

### Best Practices

```mirrow
let userName = "Alice"
let counter = 0
```

---

## Functions

```mirrow
func greet(name) {
    "Hello " + name
}

let adder = fn (a, b) => a + b
```

- Last expression returned implicitly.
- Automatic currying:

```mirrow
func add3(a, b, c) { a + b + c }
let add1 = add3(1)
let add1and2 = add1(2)
print(add1and2(3)) // 6
```

### Reflection

```mirrow
func greet(name) { "Hello " + name }
let meta = #greet
print(meta.name)    // "greet"
print(meta.params)  // ["name"]
print(meta.arity)   // 1
```

---

## Structs

Structs in Mirrow are **lightweight dynamic objects**. They are created using key-value syntax and are **immutable**. Fields cannot be modified after creation. To "modify" a struct, a new one is created with the desired changes.

```mirrow
let user = { name = "Alice", age = 30 }
print(user.name) // "Alice"
```

### Updating Structs

Because structs are immutable, updates return a **new struct**:

```mirrow
let updatedUser = user { age = 31 }
print(updatedUser.age) // 31
```

### Pattern Matching

Struct fields can be destructured directly:

```mirrow
let person = { name = "Alice", age = 25 }
match person {
    { name, age } -> IO.print($"Name: {name}, Age: {age}")
    _ -> IO.print("No match")
}
```

### Reflection

Structs support runtime reflection:

```mirrow
let keys = keys(user)     // returns ["name", "age"]
let hasAge = has(user, "age") // true
```

---

## Pattern Matching

```mirrow
let x = 3
match x {
    1 -> print("one")
    2 -> print("two")
    _ -> print("other")
}
```

- Supports structural patterns:

```mirrow
let person = { name = "Alice", age = 25 }
match person {
    {name, age} -> print("Name: " + name + ", Age: " + age)
    _ -> print("No match")
}
```

---

## Collections

### Lists

```mirrow
let numbers = [1, 2, 3]
IO.print(numbers[0]) // 1
```

#### Built-in helpers:

- `append(list, value)` → returns new list with value appended.
- `map(list, fn)` → returns transformed list.
- `filter(list, fn)` → filters list by predicate.
- `reduce(list, fn, initial)` → folds list.

### Objects (Maps)

```mirrow
let user = { name = "Alice", age = 30 }
IO.print(user.name)
```

---

## String Interpolation

```mirrow
let name = "Alice"
let greeting = $"Hello {name}, welcome!"
```

- `$"...{expr}..."` interpolates expressions at runtime.

---

## Comments

```mirrow
// single-line comment
/* multi-line
   comment */
```

---

## Modules & Imports

- File-based imports like Python.

```mirrow
import "math"
import "./utils"
```

- Entry point is `main()` when running a file.
- REPL supported.

---

## IO & Side Effects

Side effects isolated under `IO` global:

```mirrow
IO.print("Hello")
IO.write("file.txt", "content")
```

- Core runtime defines IO operations.

---

## Operators

### Precedence

- `*`, `/` bind tighter than `+`, `-`.
- `=` for assignment is right-associative.

### Types

- Arithmetic: `+ - * / %`
- Comparison: `== != > < >= <=`
- Logic: `&& || !`

---

## Error Handling

```mirrow
throw "Something went wrong"
try {
    riskyOperation()
} catch (err) {
    IO.print("Error: " + err)
}
```

---

## Concurrency

- Planned **Future** type (like JS Promises).
- Will support async constructs:

```mirrow
let f = async fetchData()
await f
```
