# `pretty-name`

[![Crates.io](https://img.shields.io/crates/v/pretty-name.svg)](https://crates.io/crates/pretty-name)
[![Documentation](https://docs.rs/pretty-name/badge.svg)](https://docs.rs/pretty-name)
[![License](https://img.shields.io/crates/l/pretty-name.svg)](https://github.com/Nekomaru-PKU/pretty-name#license)

Get the human-friendly name of types, functions, methods, fields, and enum variants in a refactoring-safe way.

## Overview

`pretty-name` provides a set of macros and functions for extracting names of Rust language constructs at compile time. Unlike `stringify!` or `std::any::type_name`, this crate offers:

## Key Features

- **Human-friendly output**: Type names are cleaned to remove module paths (`std::vec::Vec<T>` → `Vec<T>`), lifetime annotations (`&'static str` → `&str`), and other visual clutter.

- **Refactoring-safe**: When you rename items using IDE refactoring tools, the macro calls are automatically updated.

- **Compile-time validation**: All macros check that the referenced items exist. If a referenced identifier, field, method, or variant doesn't exist, you get a compile error instead of a runtime panic.

- **Natural syntax**: No strange custom syntax like `pretty_name!(type i32).

- **`Self` support**: Resolves `Self` into the appropriate type.


## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
pretty-name = "0.1"
```

Or use `cargo add`:

```bash
cargo add pretty-name
```

## Usage

### Type Names

Get human-friendly type names with `pretty_name::type_name<T>()` for types and `pretty_name::type_name_of(value)` for values:

```rust
use pretty_name::{type_name, type_name_of};

// Get type name from a type parameter
assert_eq!(type_name::<Option<i32>>(), "Option<i32>");
assert_eq!(type_name::<&str>(), "&str");
assert_eq!(type_name::<Vec<Box<dyn std::fmt::Debug>>>(), "Vec<Box<dyn Debug>>");

// Get type name from a value
let s = "hello";
assert_eq!(type_name_of(&s), "&str");

let v = vec![1, 2, 3];
assert_eq!(type_name_of(&v), "Vec<i32>");
```

The output removes module paths (e.g., `std::vec::Vec` becomes `Vec`) and lifetime annotations for cleaner, more readable type names.

### Identifier Names

Get the name of local variables, constants, and functions with `pretty_name::of!`:

```rust
fn my_function() {}
let my_variable = 42;

assert_eq!(pretty_name::of!(my_function), "my_function");
assert_eq!(pretty_name::of!(my_variable), "my_variable");
```

The macro validates that the identifier exists in the current scope. If you rename `my_variable` using your IDE's refactoring tools, the macro call will be updated automatically.

### Struct Field Names

Get the name of struct fields with `pretty_name::of_field!`:

```rust
struct MyStruct {
    my_field: i32,
}

assert_eq!(pretty_name::of_field!(MyStruct::my_field), "MyStruct::my_field");
```

This macro resolves `Self` when used inside `impl` blocks and validates that the field exists on the type.

### Method Names

Get the name of methods with `pretty_name::of_method!`:

```rust
struct MyStruct;

impl MyStruct {
    fn my_method(&self) {}
}

assert_eq!(pretty_name::of_method!(MyStruct::my_method), "MyStruct::my_method");
```

The macro resolves `Self` when used inside `impl` blocks and validates that the method exists on the type.

### Enum Variant Names

Get the name of enum variants with `pretty_name::of_variant!`. Supports unit, tuple, and struct variants:

```rust
enum MyEnum {
    UnitVariant,
    TupleVariant(i32, String),
    StructVariant { field: i32 },
}

// Unit variant
assert_eq!(pretty_name::of_variant!(MyEnum::UnitVariant), "MyEnum::UnitVariant");

// Tuple variant - use (..) syntax
assert_eq!(pretty_name::of_variant!(MyEnum::TupleVariant(..)), "MyEnum::TupleVariant");

// Struct variant - use {..} syntax
assert_eq!(pretty_name::of_variant!(MyEnum::StructVariant {..}), "MyEnum::StructVariant");
```

The macro resolves `Self` when used inside `impl` blocks and validates that the variant exists on the enum type.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0)>
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Links

- [Repository](https://github.com/Nekomaru-PKU/pretty-name)
- [Documentation](https://docs.rs/pretty-name)
- [Crates.io](https://crates.io/crates/pretty-name)
