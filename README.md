# `pretty-name`

[![Crates.io](https://img.shields.io/crates/v/pretty-name.svg)](https://crates.io/crates/pretty-name)
[![Documentation](https://docs.rs/pretty-name/badge.svg)](https://docs.rs/pretty-name)
[![License](https://img.shields.io/crates/l/pretty-name.svg)](https://github.com/Nekomaru-PKU/pretty-name#license)

Get concise, human-friendly names for Rust types, functions, methods, fields, and more—with
compiler validation plus refactoring and IDE support.

## Features

1. **Grammar-aware type shortening.** `pretty-name` parses compiler type descriptions as
   Rust syntax with [`syn`](https://docs.rs/syn) before removing module qualification.
   Unlike case- or delimiter-based heuristics, this correctly handles uncommon-looking but
   valid paths such as `PascalCase` module names while preserving nested type structure.
2. **Refactoring-safe names with IDE support.** Macros accept ordinary Rust identifiers and
   paths, so rename refactors and code completion continue to work at each call site.
3. **Compile-time validation.** Misspelled bindings, functions, types, fields, methods, and
   variants are compiler errors rather than stale strings discovered at runtime.
4. **One naming toolkit.** Name local values, constants, statics, free functions, types,
   fields, associated constants and functions, methods, and unit or tuple enum variants.
5. **Resolved generics, qualified paths, and `Self`.** Type-bearing positions use compiler
   resolution, including aliases, renamed imports, bounded generic parameters, and `Self`
   inside `impl` blocks.
6. **Deferred, local formatting.** Every operation returns a lazily formatted value,
   constructed without allocation or global state and rendered only when needed.

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

## Usage

Every operation returns a value implementing `Display`. Keep it for deferred formatting,
format it with `{}`, pass it through an `impl Display` API, or explicitly materialize a
`String` with `.to_string()`.

1. **Name a type with `type_name::<T>()` or `nameof_type!(T)`.** `type_name` is a
   human-friendly counterpart to `std::any::type_name`: it returns the compiler-resolved type
   description with module qualification removed. `nameof_type!` provides equivalent macro
   syntax. Both resolve aliases, renamed imports, generic parameters, and `Self` through the
   compiler.

   ```rust
   use pretty_name::{nameof_type, type_name};
   use std::collections::HashMap;

   let _ = type_name::<HashMap<String, i32>>();
   // -> "HashMap<String, i32>"

   // Qualified paths produce the same shortened output.
   let _ = type_name::<std::collections::HashMap<std::string::String, i32>>();
   // -> "HashMap<String, i32>"

   let _ = nameof_type!(HashMap<String, i32>);
   // -> "HashMap<String, i32>"
   ```

2. **Name the type of a value with `type_name_of_val(&value)`.** This is the
   human-friendly counterpart to `std::any::type_name_of_val`. It inspects the referenced
   value's type without adding another reference layer to the displayed name.

   ```rust
   use pretty_name::type_name_of_val;

   let numbers = vec![1_i32, 2, 3];
   let _ = type_name_of_val(&numbers);
   // -> "Vec<i32>"
   ```

3. **Name a binding, constant, static, or free function with `nameof!(path)`.** The lexical
   value path is preserved, including module aliases and a leading `::`, while explicit
   generic arguments are shortened to their compiler-resolved type names.

   ```rust
   use pretty_name::nameof;

   let numbers = vec![1_i32, 2, 3];
   let _ = nameof!(numbers);
   // -> "numbers"

   let _ = nameof!(std::f32::consts::PI);
   // -> "std::f32::consts::PI"

   let _ = nameof!(std::array::from_mut::<u32>);
   // -> "std::array::from_mut<u32>"
   ```

4. **Name an associated constant, associated function, method, or unit or tuple enum variant
   with `nameof_member!(Owner::member)`.** The owner is compiler-resolved and formatted as a
   type. A single-identifier owner uses the compact form; qualified or generic owners must
   make their boundary explicit with `<...>`. The angle wrapper and source turbofish are
   omitted from the displayed output.

   ```rust
   use pretty_name::nameof_member;

   let _ = nameof_member!(u32::MAX);
   // -> "u32::MAX"

   let _ = nameof_member!(String::with_capacity);
   // -> "String::with_capacity"

   let _ = nameof_member!(<std::vec::Vec<String>>::push);
   // -> "Vec<String>::push"

   let _ = nameof_member!(<Option<u32>>::None);
   // -> "Option<u32>::None"
   let _ = nameof_member!(<Option<u32>>::Some);
   // -> "Option<u32>::Some"
   ```

5. **Name an instance field with `nameof_field!(Owner::field)`.** The macro validates the
   field through field-access syntax, then displays its compiler-resolved owner and field
   name. Qualified or generic owners use the same `<...>` boundary as `nameof_member!`.

   ```rust
   use pretty_name::nameof_field;

   let _ = nameof_field!(<std::ops::Range<u32>>::start);
   // -> "Range<u32>::start"
   ```

## Behavior

### Type formatting is structural

For valid Rust type descriptions, module qualification is removed from the parsed syntax tree
rather than guessed from identifier casing or punctuation. Formatting preserves:

1. References, compiler-emitted lifetimes, mutability, and raw-pointer qualifiers.
2. Arrays, slices, tuple arity, grouping, and parentheses.
3. Function qualifiers, ABI, arguments, and return types.
4. Trait bounds and associated-type bindings.
5. Generic type and const arguments, including nested types.
6. Qualified-self and other non-module type structure.

For example, `std::collections::HashMap<std::string::String, crate::Record>` becomes
`HashMap<String, Record>` without discarding the surrounding generic structure. If parsing or
transformation cannot be completed confidently—as with some closure and async-block
descriptions—the entire compiler description is returned unchanged.

### Source paths and resolved types stay distinct

`nameof!` preserves the value path written at the call site, so renamed value and module
imports remain visible. Type-bearing positions use compiler-resolved descriptions, so type
aliases and renamed type imports are transparent. Explicit generic arguments are therefore
shortened as their resolved types even when the surrounding function path remains lexical.

Macros resolve `Self` to the concrete type inside `impl` blocks. Trait-provided methods can be
named through a concrete implementor or a bounded type parameter such as `T::method`.

### Values implement `Display`

Every operation produces a value implementing `Display`. The concrete return types,
representations, sizes, and additional trait implementations are not part of the compatibility
contract. Callers can keep an inferred value, pass it through an `impl Display` API, or
materialize a `String` with `.to_string()`. A value returned by `type_name_of_val` does not
retain the input borrow and may outlive the inspected value.

## Guarantees

### Referenced items are compiler-validated

Every supported macro form pairs captured identifiers with ordinary Rust syntax that resolves
and type-checks the referenced path, owner, field, member, and explicit arguments. Misspelled
or semantically invalid inputs therefore remain compile-time errors.

### Validation does not execute user code

Every validation expression lives in an uncalled closure. Constructing a name does not call a
function or method, read an associated value, access a field, construct a variant, invoke
`Deref`, or run a destructor. Local-value validation may establish a temporary shared closure
capture for borrow checking, but it invokes no user code.

### Downstream lint policy is isolated

Macro-generated validation locally allows warnings so incidental validation warnings,
including deprecation, do not become errors under a downstream `deny(warnings)` policy.
Syntax, name-resolution, privacy, and type errors remain compiler errors, and command-line
force-warn policy remains authoritative.

### Construction avoids allocation and global state

Explicit generic arguments are stored in a fixed-size inline array, so constructing a name
does not allocate or consult a process-wide cache. Formatting a type may allocate while
parsing and rendering it, repeated formatting may repeat that work, and writes use the
caller-provided formatter.

## Limitations and non-guarantees

1. **Names are diagnostic presentation, not identity.** Rust does not guarantee that compiler
   type names are unique or stable between compiler versions. Shortening can also make
   different resolved types display alike. Do not use these names as persistent identifiers
   or serialization keys.
2. **Generic values and members require complete, concrete type arguments.** Every
   caller-provided generic argument must be written explicitly as a type. Inferred arguments,
   omitted or partial argument lists, direct const arguments, and the legacy `::<..>`
   placeholder are unsupported. Const arguments nested inside a resolved type remain
   supported.
3. **Owners must be named type paths.** Anonymous owner types and qualified-self owner paths
   are unsupported. A bare trait declaration is not a resolved owner; use a concrete
   implementor or bounded type parameter.
4. **Struct variants are unsupported.** Unit variants and tuple-variant constructors are
   first-class values and work with `nameof_member!`; struct variant paths are not first-class
   values on stable Rust, which cannot validate them through the same member-resolution
   contract.
5. **Qualified ordinary value paths must be importable.** This keeps type-associated paths
   such as `Type::function` out of `nameof!`; use `nameof_member!` for associated items and
   variants.
6. **Implementation wrappers are unstable.** The public, doc-hidden `pretty_name::__` module
   must exist for exported macros. Advanced callers may use `TypeName` and `ItemName` directly
   as formatting primitives, including to simplify arbitrary compiler-style descriptions,
   but the module and its concrete types are excluded from compatibility guarantees.

## Comparison with similar crates

The best choice depends on whether you need broad item naming, type shortening, generated
schema names, or minimum runtime and dependency cost:

| Tool | Best fit | Important trade-off |
| --- | --- | --- |
| `pretty-name` | One call-site API for resolved types and validated values, functions, fields, methods, associated items, and variants | Uses `syn`, `quote`, and `prettyplease`; type formatting may allocate, and the crate targets `std`. In return, shortening follows Rust's type grammar and unfamiliar descriptions fall back unchanged. |
| [`nameof`](https://docs.rs/nameof) | Small, dependency-free macros for unqualified binding, type, field, function, and associated-constant names | Returns source-like unqualified names and has a narrower item model; it does not provide structural shortening of compiler-resolved composite types. |
| [`disqualified`](https://docs.rs/disqualified) | Lazy, allocation-free, `no_std` shortening of type names | Its text scanner deliberately uses an uppercase-segment heuristic to retain enum owners. A PascalCase module segment can therefore be mistaken for a type or variant owner; it also does not name arbitrary call-site items. |
| [`named_type`](https://docs.rs/named_type) with [`named_type_derive`](https://docs.rs/named_type_derive) | Generated full and short names for structs and enums you control, including configurable short-name prefixes and suffixes | Requires deriving on the named type and does not cover values, functions, fields, methods, or arbitrary composite types at call sites. |
| [`field_name`](https://docs.rs/field_name) | Generated field and variant constants, complete name arrays, and schema-oriented rename or skip attributes | Requires annotating structs or enums you control; it is aimed at schema metadata rather than general type and item diagnostics. |

In particular, `pretty-name` chooses syntax-aware correctness over the casing conventions
assumed by many lightweight shorteners. Rust normally uses `snake_case` modules and
`PascalCase` types, but those conventions are linted rather than grammatical requirements;
parsing the type prevents valid naming choices from changing the result.

## Pre-1.0 Compatibility

`pretty-name@1.0.0` has been fully rewritten from the pre-1.0 `pretty-name@0.5.0` crate. The new version is a complete redesign with a different API and implementation. It is not compatible with the pre-1.0 version and manual migration is required. The pre-1.0 version is still available on [crates.io](https://crates.io/crates/pretty-name/0.5.0).

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
