# `pretty-name`

[![Crates.io](https://img.shields.io/crates/v/pretty-name.svg)](https://crates.io/crates/pretty-name)
[![Documentation](https://docs.rs/pretty-name/badge.svg)](https://docs.rs/pretty-name)
[![License](https://img.shields.io/crates/l/pretty-name.svg)](https://github.com/Nekomaru-PKU/pretty-name#license)

Get the human-friendly name of types, functions, methods, fields, and enum variants in a refactoring-safe way.

## Overview

`pretty-name` provides macros and functions for extracting readable names of Rust language constructs. Every naming operation returns an opaque value exposing only `Display`; the concrete representation is private. Unlike `stringify!` or `std::any::type_name`, this crate offers:

### Key Features

- **Human-friendly output**: Type names are parsed as Rust syntax so module qualification can be removed structurally (`std::vec::Vec<T>` → `Vec<T>`) without discarding other compiler-emitted type information.

- **Refactoring-safe**: When you rename items using IDE refactoring tools, the macro calls are automatically updated—no more outdated string literals.

- **Full IDE auto-completion support**: Get all your IDE's auto-completion features even inside macros. No more guessing or manual typing.

- **Semantic support for generics, qualified paths, and `Self`**: Preserves lexical module paths, resolves their generic types through the compiler, and resolves `Self` to the concrete type inside `impl` blocks.

- **Catch typos at compile time**: Every referenced item is validated. Misspelled identifiers, fields, methods, or variants trigger compile errors instead of runtime failures.

- **A deliberately narrow value API**: Name values expose `Display` without exposing their concrete type, representation, or unrelated traits such as `Debug`.

- **Natural, idiomatic syntax**: All syntax follows Rust conventions as closely as possible, making the macros feel like native language features.

- **Deferred formatting without global state**: Name values are assembled locally and formatted on demand without a process-wide cache.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
pretty-name = "0.5"
```

Or use `cargo add`:

```bash
cargo add pretty-name
```

## Usage

Naming operations return opaque `Display` values. They can be formatted with `{}`, passed
through `impl Display` APIs, or explicitly materialized with `.to_string()`.

```rust
use pretty_name::*;
use std::fmt::Display;

struct Owner<T> {
    field: T,
}

impl<T> Owner<T> {
    const CONSTANT: u32 = 42;

    fn method<U>(&self) {}
}

enum Choice<T> {
    Unit,
    Tuple(T),
}

fn generic_function<T>() {}

fn forward_name(name: impl Display) -> impl Display { name }

let local_value = 42;

assert_eq!(nameof!(local_value).to_string(), "local_value");
assert_eq!(nameof!(generic_function::<u32>).to_string(), "generic_function<u32>");
assert_eq!(
    forward_name(nameof!(std::array::from_mut::<u32>)).to_string(),
    "std::array::from_mut<u32>");
assert_eq!(nameof_member!(<Owner<u32>>::CONSTANT).to_string(), "Owner<u32>::CONSTANT");
assert_eq!(
    nameof_member!(<Owner<u32>>::method::<String>).to_string(),
    "Owner<u32>::method<String>");
assert_eq!(nameof_member!(<Choice<u32>>::Tuple).to_string(), "Choice<u32>::Tuple");
assert_eq!(nameof_field!(<Owner<u32>>::field).to_string(), "Owner<u32>::field");
assert_eq!(nameof_type!(std::collections::HashMap<String, i32>).to_string(),
    "HashMap<String, i32>");
assert_eq!(type_name::<Option<u32>>().to_string(), "Option<u32>");
assert_eq!(type_name_of_val(&local_value).to_string(), "i32");
```

The four macros use distinct resolution modes:

1. `nameof!` names an ordinary value path, including bindings, constants, statics, and
   free functions. Module qualification, aliases, and a leading `::` are retained as a
   lexical path; explicit type arguments use shortened compiler-resolved names.
2. `nameof_member!` names a value-like member selected through a resolved owner type.
   Members include associated constants, associated functions, methods, and unit or
   tuple enum variants.
3. `nameof_field!` validates an instance field through field-access syntax.
4. `nameof_type!` obtains the compiler-resolved name of a type.

Additional behavior and boundaries:

1. Macros resolve `Self` to the appropriate type when used inside `impl` blocks.
2. A single-identifier owner uses `Type::member`. Qualified or generic owners make
   their input boundary explicit with `<...>`, such as
   `<module::Type<Args>>::member`. The wrapper is omitted from displayed output.
3. Anonymous owner types and qualified-self owner paths remain unsupported.
4. Name trait-provided methods through a concrete implementor or a bounded type
   parameter such as `T::method`. A bare trait declaration is not a resolved owner type.
5. Struct variants are unsupported because their paths are not first-class values.
   Stable Rust validates member resolution rather than the declaration category.
6. Generic values and members require every caller-provided generic argument to be
   written explicitly as a concrete type. Inferred arguments, direct const arguments,
   omitted arguments, and the legacy `::<..>` placeholder are unsupported.
7. A qualified `nameof!` path must also be importable. This rejects type-associated paths
   such as `Type::function`; use `nameof_member!` for associated items and variants.
8. Every validation expression is placed in an uncalled closure. Naming does not invoke
   functions, access fields, evaluate members, construct variants, call `Deref`, or run
   destructors. Local-value validation may establish a temporary shared closure capture
   for borrow checking, but it invokes no user code.
9. Macro-generated validation locally allows warnings so a downstream `deny(warnings)`
   policy does not turn incidental validation warnings, including deprecation, into errors.
   Syntax, resolution, and type errors remain compiler errors.
10. Constructing a name with generic arguments may allocate its argument slice.
    Formatting a type may allocate while parsing and rendering it, and writes through
    the caller-provided formatter.
11. Type names are diagnostic presentation. Rust does not guarantee that compiler type
    names are unique or stable between compiler versions, so they are unsuitable as
    persistent identifiers or serialization keys.
12. Compiler-generated names that are not valid Rust type syntax, such as some closure
    descriptions, are returned unchanged rather than replaced with an error marker.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0)>
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Links

- [Repository](https://github.com/Nekomaru-PKU/pretty-name)
- [Documentation](https://docs.rs/pretty-name)
- [Crates.io](https://crates.io/crates/pretty-name)
