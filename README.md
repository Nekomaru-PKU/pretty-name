# `pretty-name`

[![Crates.io](https://img.shields.io/crates/v/pretty-name.svg)](https://crates.io/crates/pretty-name)
[![Documentation](https://docs.rs/pretty-name/badge.svg)](https://docs.rs/pretty-name)
[![License](https://img.shields.io/crates/l/pretty-name.svg)](https://github.com/Nekomaru-PKU/pretty-name#license)

Get the human-friendly name of types, functions, methods, fields, and enum variants in a refactoring-safe way.

## Overview

`pretty-name` provides a set of macros and functions for extracting names of Rust language constructs at compile time. Unlike `stringify!` or `std::any::type_name`, this crate offers:

### Key Features

- **Human-friendly output**: Type names are cleaned to remove module paths (`std::vec::Vec<T>` → `Vec<T>`), lifetime annotations (`&'static str` → `&str`), and other visual clutter for maximum readability.

- **Refactoring-safe**: When you rename items using IDE refactoring tools, the macro calls are automatically updated—no more outdated string literals.

- **Full IDE auto-completion support**: Get all your IDE's auto-completion features even inside macros. No more guessing or manual typing.

- **Full support for generics, qualified paths, and `Self`**: Works seamlessly with generic types, module-qualified names, and `Self` in impl blocks—handle any type your code needs.

- **Catch typos at compile time**: Every referenced item is validated. Misspelled identifiers, fields, methods, or variants trigger compile errors instead of runtime failures.

- **Natural, idiomatic syntax**: All syntax follows Rust conventions as closely as possible, making the macros feel like native language features.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
pretty-name = "0.3"
```

Or use `cargo add`:

```bash
cargo add pretty-name
```

## Usage

| What to get | Syntax | Example | Returns |
|-------------|--------|---------|---------|
| **Type names** | | | |
| Type name | `type_name::<T>()` | `type_name::<Vec<i32>>()` → `"Vec<i32>"` | `&'static str` |
| Type name from value | `type_name_of_val(val)` | `type_name_of_val(&vec![1])` → `"Vec<i32>"` | `&'static str` |
| **Variables and constants** | | | |
| Variable or constant name | `pretty_name::of_var!(ident)` | `pretty_name::of_var!(my_var)` → `"my_var"` | `&'static str` |
| **Functions** | | | |
| Function name | `pretty_name::of_function!(ident)` | `pretty_name::of_function!(my_func)` → `"my_func"` | `&'static str` |
| Generic function (exclude params) | `pretty_name::of_function!(ident::<..>)` | `pretty_name::of_function!(my_func::<..>)` → `"my_func"` | `&'static str` |
| Generic function (include params) | `pretty_name::of_function!(ident::<T, U>)` | `pretty_name::of_function!(my_func::<u32, String>)` → `"my_func::<u32, String>"` | `String` |
| **Struct fields** | | | |
| Field name | `pretty_name::of_field!(Type::field)` | `pretty_name::of_field!(MyStruct::field)` → `"MyStruct::field"` | `String` |
| Field name (on generic type) | `pretty_name::of_field!(<Type<T>>::field)` | `pretty_name::of_field!(<MyStruct<T>>::field)` → `"<MyStruct<T>>::field"` | `String` |
| Field name (on qualified type) | `pretty_name::of_field!(<module::Type>::field)` | `pretty_name::of_field!(<my_module::MyStruct>::field)` → `"<my_module::MyStruct>::field"` | `String` |
| **Methods** | | | |
| Method name | `pretty_name::of_method!(Type::method)` | `pretty_name::of_method!(MyStruct::method)` → `"MyStruct::method"` | `String` |
| Method (on generic type) | `pretty_name::of_method!(<Type<T>>::method)` | `pretty_name::of_method!(<MyStruct<T>>::method)` → `"<MyStruct<T>>::method"` | `String` |
| Method (on qualified type) | `pretty_name::of_method!(<module::Type>::method)` | `pretty_name::of_method!(<my_module::MyStruct>::method)` → `"<my_module::MyStruct>::method"` | `String` |
| Generic method | `pretty_name::of_method!(Type::method::<T>)` | `pretty_name::of_method!(MyStruct::method::<u32>)` → `"MyStruct::method::<u32>"` | `String` |
| Generic method (on generic type) | `pretty_name::of_method!(<Type<T>>::method::<U>)` | `pretty_name::of_method!(<MyStruct<T>>::method::<u32>)` → `"<MyStruct<T>>::method::<u32>"` | `String` |
| Generic method (on qualified type) | `pretty_name::of_method!(<module::Type>::method::<T>)` | `pretty_name::of_method!(<my_module::MyStruct>::method::<u32>)` → `"<my_module::MyStruct>::method::<u32>"` | `String` |
| **Enum variants** | | | |
| Unit variant | `pretty_name::of_variant!(Type::Variant)` | `pretty_name::of_variant!(MyEnum::UnitVariant)` → `"MyEnum::UnitVariant"` | `String` |
| Tuple variant | `pretty_name::of_variant!(Type::Variant(..))` | `pretty_name::of_variant!(MyEnum::TupleVariant(..))` → `"MyEnum::TupleVariant"` | `String` |
| Struct variant | `pretty_name::of_variant!(Type::Variant{..})` | `pretty_name::of_variant!(MyEnum::StructVariant{..})` → `"MyEnum::StructVariant"` | `String` |
| Variant (on generic type) | `pretty_name::of_variant!(Type::<T>::Variant)` | `pretty_name::of_variant!(MyEnum::<u32>::Variant)` → `"<MyEnum<u32>>::Variant"` | `String` |
| Variant (on qualified type) | `pretty_name::of_variant!(<Type>::Variant)` | `pretty_name::of_variant!(<MyEnum<T>>::Variant)` → `"<MyEnum<T>>::Variant"` | `String` |

**Notes:**
- Type names are cleaned to remove module paths (`std::vec::Vec` → `Vec`) and lifetime annotations (`&'static str` → `&str`)
- Macros that return `&'static str` have zero runtime overhead and use caching
- Macros resolve `Self` to the appropriate type when used inside `impl` blocks
- Use `<Type>` syntax for types with qualified paths or generic parameters

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0)>
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Links

- [Repository](https://github.com/Nekomaru-PKU/pretty-name)
- [Documentation](https://docs.rs/pretty-name)
- [Crates.io](https://crates.io/crates/pretty-name)
