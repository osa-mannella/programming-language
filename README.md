# Mirrow

A web-first functional programming language built for modern backend development.

## Overview

Mirrow prioritizes immutability, safety, and readable code while remaining practical for building APIs and web services. Designed for developers who want functional programming benefits without sacrificing productivity.

## Key Features

- **Immutable by default** - All data is immutable, eliminating entire classes of bugs
- **Elegant error handling** - `let!` operator for clean error propagation with `Result` and `Maybe` types
- **Pipeline operators** - Compose operations naturally with `|>` for readable data transformations
- **Pattern matching** - Destructure data with powerful match expressions
- **Side effect isolation** - All I/O operations explicitly marked under the `IO` namespace

## Quick Example

```mirrow
func processUsers(ids) {
  let! users = ids
    |> map(getUser)
    |> collectResults
  users |> filter(fn(u) -> u.active)
}

func getUser(id) {
  let! userData = IO.readFile($"users/{id}.json")
  let! parsed = parseJSON(userData)
  validateUser(parsed)
}
```

## Getting Started

Mirrow is in early development. Follow this repository for updates as we build toward an initial release.

## Architecture

- **Runtime**: Built in Rust for performance and safety
- **Execution**: Custom bytecode VM with single-threaded async/await
- **Package Management**: JSON configuration with GitHub-based dependencies

## Status

🚧 **Early Development** - Core language design and bytecode specification complete. VM implementation in progress.
