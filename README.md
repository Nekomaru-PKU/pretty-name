# `pretty-name`

[![Crates.io](https://img.shields.io/crates/v/pretty-name.svg)](https://crates.io/crates/pretty-name)
[![Documentation](https://docs.rs/pretty-name/badge.svg)](https://docs.rs/pretty-name)
[![License](https://img.shields.io/crates/l/pretty-name.svg)](https://github.com/NekomaruQwQ/pretty-name#license)

Get concise, human-friendly names for Rust types, values, functions, fields, methods, and
variants without maintaining string literals by hand.

`pretty-name` structurally shortens compiler-resolved type names. Its naming macros accept
ordinary Rust syntax, so the compiler and IDE can validate and refactor their references. The
returned values implement `Display` and are intended for diagnostics and other human-readable
output—not stable type identifiers or serialization keys.

## Quick start

Add the crate to your project:

```toml
[dependencies]
pretty-name = "1.0.0"
```

Or run:

```bash
cargo add pretty-name
```

`pretty-name` requires `std`. Then format names like any other `Display` value:

```rust
use pretty_name::{nameof, type_name_of_val};

let numbers = vec![1_i32, 2, 3];

assert_eq!(type_name_of_val(&numbers).to_string(), "Vec<i32>");
assert_eq!(nameof!(numbers).to_string(), "numbers");
```

The examples call `.to_string()` only to make the output easy to assert. You can keep the
returned value for deferred formatting or pass it directly to an API accepting `impl Display`.

## Why use `pretty-name`?

1. **Grammar-aware type shortening.** Compiler type descriptions are parsed as Rust syntax
   with [`syn`](https://docs.rs/syn) before module qualification is removed. Nested type
   structure is preserved, even when valid paths do not follow conventional casing.
2. **Compiler-checked names.** Misspelled bindings, functions, types, fields, methods, and
   variants fail to compile instead of becoming stale strings at runtime.
3. **Refactoring and IDE support.** Macros accept ordinary Rust identifiers and paths, so
   rename refactors and code completion continue to work at each call site.
4. **One API family for common nameable items.** The crate covers local values, constants,
   statics, free functions, types, fields, associated items, methods, and unit or tuple enum
   variants.
5. **Deferred, local formatting.** Constructing a name does not allocate or use global state.
   Formatting a type may allocate while parsing and rendering it.

## API at a glance

| API | Names | Example output |
| --- | --- | --- |
| `type_name::<T>()` | A compiler-resolved type | `HashMap<String, i32>` |
| `nameof_type!(T)` | A compiler-resolved type, using macro syntax | `HashMap<String, i32>` |
| `type_name_of_val(&value)` | The compiler-resolved type of a value | `Vec<i32>` |
| `nameof!(path)` | A binding, constant, static, or free function | `std::f32::consts::PI` |
| `nameof_member!(Owner::member)` | An associated item, method, or unit or tuple variant | `String::with_capacity` |
| `nameof_field!(Owner::field)` | A named instance field | `Range<u32>::start` |

Every API returns a value implementing `Display`. `type_name::<T>()` and `nameof_type!(T)`
produce equivalent names; use whichever form reads better at the call site.

## Naming types

`type_name::<T>()` is a human-friendly counterpart to `std::any::type_name`. It uses the
compiler-resolved type—including aliases, renamed imports, bounded generic parameters, and
`Self` inside `impl` blocks—then removes module qualification structurally.

```rust
use pretty_name::{nameof_type, type_name};
use std::collections::HashMap;

assert_eq!(
    type_name::<HashMap<String, i32>>().to_string(),
    "HashMap<String, i32>");

// Qualified paths produce the same shortened output.
assert_eq!(
    type_name::<std::collections::HashMap<std::string::String, i32>>().to_string(),
    "HashMap<String, i32>");

assert_eq!(
    nameof_type!(HashMap<String, i32>).to_string(),
    "HashMap<String, i32>");
```

`type_name_of_val(&value)` is the corresponding operation for a value. It inspects the
referenced value's type without adding another reference layer to the displayed name. The
returned name does not retain the input borrow and may outlive the value.

```rust
use pretty_name::type_name_of_val;

let numbers = vec![1_i32, 2, 3];
assert_eq!(type_name_of_val(&numbers).to_string(), "Vec<i32>");
```

## Naming values and functions

Use `nameof!(path)` for a local binding, constant, static, or free function. It preserves the
value path as written, including module aliases and a leading `::`. Explicit generic arguments
are displayed using their shortened, compiler-resolved type names.

```rust
use pretty_name::nameof;

let numbers = vec![1_i32, 2, 3];

assert_eq!(nameof!(numbers).to_string(), "numbers");
assert_eq!(
    nameof!(std::f32::consts::PI).to_string(),
    "std::f32::consts::PI");
assert_eq!(
    nameof!(std::array::from_mut::<u32>).to_string(),
    "std::array::from_mut<u32>");
```

## Naming members and fields

Use `nameof_member!(Owner::member)` for an associated constant, associated function, method,
or unit or tuple enum variant. The owner is compiler-resolved and displayed as a shortened
type.

```rust
use pretty_name::nameof_member;

assert_eq!(nameof_member!(u32::MAX).to_string(), "u32::MAX");
assert_eq!(
    nameof_member!(<std::vec::Vec<String>>::push).to_string(),
    "Vec<String>::push");
assert_eq!(
    nameof_member!(<Option<u32>>::Some).to_string(),
    "Option<u32>::Some");
```

A single-identifier owner uses the compact `Owner::member` form. A qualified or generic owner
must make its boundary explicit as `<Owner>::member`. The angle brackets and any source
turbofish are omitted from the displayed name.

Use `nameof_field!(Owner::field)` for a named instance field. It uses the same owner syntax and
validates the field through ordinary field-access syntax.

```rust
use pretty_name::nameof_field;

assert_eq!(
    nameof_field!(<std::ops::Range<u32>>::start).to_string(),
    "Range<u32>::start");
```

Trait-provided methods can be named through a concrete implementor or a bounded type parameter
such as `T::method`. Inside an `impl` block, macros resolve `Self` to the concrete type.

## How names are formatted

### Type formatting is structural

For valid Rust type descriptions, module qualification is removed from the parsed syntax tree
rather than guessed from identifier casing or punctuation. Formatting preserves references,
lifetimes, pointers, arrays, slices, tuples, function signatures, trait bounds, associated-type
bindings, generic type and const arguments, and qualified-self structure.

For example, `std::collections::HashMap<std::string::String, crate::Record>` becomes
`HashMap<String, Record>` without discarding the surrounding generic structure. If a compiler
description cannot be parsed or transformed confidently—as can happen with closure and
async-block descriptions—the complete original description is returned unchanged.

### Item paths are lexical; type positions are resolved

`nameof!` preserves the value path written at the call site, so renamed value and module
imports remain visible. Type-bearing positions use compiler-resolved descriptions, so type
aliases and renamed type imports are transparent. This is why an explicit generic argument is
shortened as a resolved type even though its surrounding function path remains lexical.

## Validation and runtime behavior

Supported macros pair captured identifiers with ordinary Rust expressions that resolve and
type-check the referenced path, owner, field, member, and explicit arguments. Syntax,
name-resolution, privacy, and type errors therefore remain compile-time errors.

Validation expressions live in uncalled closures. Creating a name does not call a function or
method or otherwise evaluate the referenced item. Validating a local value may establish a
temporary shared closure capture for borrow checking, but does not execute user code.

Generated validation locally allows warnings so incidental warnings, including deprecation,
do not become errors under a downstream `deny(warnings)` policy. Command-line force-warn policy
remains authoritative.

Constructing a name does not allocate or consult a process-wide cache. Formatting a type may
allocate while parsing and rendering it, and formatting the same value repeatedly may repeat
that work. The crate requires `std`.

Only the `Display` behavior is part of the return-value contract. Concrete return types,
representations, sizes, and additional trait implementations may change between releases.

## Limitations

1. **Names are presentation, not identity.** Compiler type names are not guaranteed to be
   unique or stable between Rust versions, and shortening can make different resolved types
   look alike. Do not use these names as persistent identifiers or serialization keys.
2. **Generic values and members require complete type arguments.** Every caller-provided
   generic argument must be written explicitly as a type. Inferred, omitted, or partial
   argument lists, direct const arguments, and the legacy `::<..>` placeholder are unsupported.
   Const arguments nested inside a resolved type remain supported.
3. **Owners must be named type paths.** Anonymous owner types and qualified-self owner paths
   are unsupported. A bare trait declaration is not a resolved owner; use a concrete
   implementor or bounded type parameter.
4. **Struct variants are unsupported.** Unit variants and tuple-variant constructors are
   first-class values and work with `nameof_member!`. Struct variant paths are not first-class
   values on stable Rust and cannot use the same validation contract.
5. **Qualified ordinary value paths must be importable.** This keeps type-associated paths
   such as `Type::function` out of `nameof!`; use `nameof_member!` for associated items and
   variants.

## Comparison with similar crates

The best choice depends on whether you need broad item naming, type shortening, generated
schema names, or minimum runtime and dependency cost:

| Tool | Best fit | Important trade-off |
| --- | --- | --- |
| `pretty-name` | Compiler-checked names for resolved types and a broad set of call-site items | Uses `syn`, `quote`, and `prettyplease`, requires `std`, and may allocate while formatting types. In return, shortening follows Rust's grammar and unfamiliar descriptions fall back unchanged. |
| [`nameof`](https://docs.rs/nameof) | Small, dependency-free macros for unqualified item names | Has a narrower item model and does not structurally shorten compiler-resolved composite types. |
| [`disqualified`](https://docs.rs/disqualified) | Lazy, allocation-free, `no_std` type-name shortening | Uses an uppercase-segment heuristic, so unconventional module casing can affect its output; it does not name arbitrary call-site items. |
| [`named_type`](https://docs.rs/named_type) with [`named_type_derive`](https://docs.rs/named_type_derive) | Generated names for structs and enums you control | Requires a derive and does not cover values, functions, fields, methods, or arbitrary composite types. |
| [`field_name`](https://docs.rs/field_name) | Generated field and variant constants for schema metadata | Requires annotating types you control and targets schema metadata rather than general diagnostics. |

## Migrating from 0.5

Version 1.0 is a complete redesign and is not API-compatible with `pretty-name` 0.5. Migrating
from 0.5 requires manual changes. The earlier release remains available on
[crates.io](https://crates.io/crates/pretty-name/0.5.0).

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
