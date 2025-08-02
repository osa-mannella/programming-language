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
let updatedUser = user <- { age = 31 }
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

## Pipeline Operator (`|>`) and Error Propagation (`let!`)

### Pipeline Operator (`|>`)

The pipeline operator `|>` allows you to chain function calls, passing the result of one operation into the next. This is especially useful when working with functions that return `Maybe` or `Result` types, enabling fluent error handling and functional programming styles.

```mirrow
let result = getUser("osa")
    |> parseUser
    |> validateUser
```

This is equivalent to:

```mirrow
let result = validateUser(parseUser(getUser("osa")))
```

If any function in the chain returns an error (`Err`) or absence (`None`), the pipeline will propagate that value.

#### Built-in Pipeline Helpers

- `map` applies a function to the inner value (if present).
- `flatMap` chains another function that returns a `Maybe` or `Result`.
- `unwrapOr` provides a fallback/default value.
- `mapError` transforms an error value.

#### Example

```mirrow
readFile("config.json")
    |> Result.map(parseConfig)
    |> Result.flatMap(connectDatabase)
    |> Result.unwrapOr(defaultDB)
```

### Error Propagation with `let!`

The `let!` binding is used within functions that return a `Result` or `Maybe` type. It simplifies error handling by automatically propagating errors or absence, letting you write clear, linear code without manual matching.

```mirrow
func setup() -> Result<Database, SetupError> {
    let! fileData = readFile("config.json")   // returns Result<String, IOError>
    let! config   = parseConfig(fileData)      // returns Result<Config, ParseError>
    connectDatabase(config)                    // returns Result<Database, SetupError>
}
```

If any step returns an error (`Err`) or absence (`None`), the function immediately returns that error/absence.

#### Equivalent Expanded Form

```mirrow
func setup() -> Result<Database, SetupError> {
    match readFile("config.json") {
        Ok(fileData) =>
            match parseConfig(fileData) {
                Ok(config) => connectDatabase(config)
                Err(e) => return Err(e)
            }
        Err(e) => return Err(e)
    }
}
```

#### Rules

- `let!` may only be used inside functions returning `Result` or `Maybe`.
- Using `let!` elsewhere will produce a compile or runtime error.

#### Example with Maybe

```mirrow
func middleName(user) -> Maybe<String> {
    let! name = user.name         // Maybe<String>
    let! parts = splitName(name)  // Maybe<[String]>
    parts[1]
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
