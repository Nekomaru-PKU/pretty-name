# `pretty-name`

[![Crates.io](https://img.shields.io/crates/v/pretty-name.svg)](https://crates.io/crates/pretty-name)
[![Documentation](https://docs.rs/pretty-name/badge.svg)](https://docs.rs/pretty-name)
[![License](https://img.shields.io/crates/l/pretty-name.svg)](https://github.com/Nekomaru-PKU/pretty-name#license)

Get the human-friendly name of types, functions, methods, fields, and enum variants in a refactoring-safe way.

## Overview

`pretty-name` provides macros and functions for extracting readable names of Rust language constructs. Type operations return an opaque value that implements `Display`; the remaining name macros still use their legacy string results while the M1 redesign is completed. Unlike `stringify!` or `std::any::type_name`, this crate offers:

### Key Features

- **Human-friendly output**: Type names are parsed as Rust syntax so module qualification can be removed structurally (`std::vec::Vec<T>` → `Vec<T>`) without discarding other compiler-emitted type information.

- **Refactoring-safe**: When you rename items using IDE refactoring tools, the macro calls are automatically updated—no more outdated string literals.

- **Full IDE auto-completion support**: Get all your IDE's auto-completion features even inside macros. No more guessing or manual typing.

- **Full support for generics, qualified paths, and `Self`**: Works seamlessly with generic types and module-qualified names, and resolves `Self` to the appropriate type when used inside `impl` blocks—handle any type your code needs.

- **Catch typos at compile time**: Every referenced item is validated. Misspelled identifiers, fields, methods, or variants trigger compile errors instead of runtime failures.

- **Evaluation at compile time for common cases**: Macros yield string literals for the most frequently used simple forms, with no runtime cost.

- **Natural, idiomatic syntax**: All syntax follows Rust conventions as closely as possible, making the macros feel like native language features.

- **Deferred type formatting**: Resolved type descriptions are stored without a cache and parsed whenever their `Display` representation is requested.

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

Type operations yield `TypeName`, which can be formatted with `{}` or converted with the standard `.to_string()`. Other macros retain their legacy `&'static str` results until their M1 stages are complete.

| What to get | Syntax | Example |
|-------------|--------|---------|
| **Type names** | | |
| Type name from macro | `pretty_name::of_type!(T)` | `pretty_name::of_type!(Vec<i32>).to_string()` → `"Vec<i32>"` |
| Type name | `type_name::<T>()` | `type_name::<Vec<i32>>().to_string()` → `"Vec<i32>"` |
| Type name from value | `type_name_of_val(val)` | `type_name_of_val(&vec![1]).to_string()` → `"Vec<i32>"` |
| **Variables and constants** | | |
| Variable or constant name | `pretty_name::of_var!(ident)` | `pretty_name::of_var!(my_var)` → `"my_var"` |
| **Functions** | | |
| Function name | `pretty_name::of_function!(ident)` | `pretty_name::of_function!(my_func)` → `"my_func"` |
| Generic function (exclude params) | `pretty_name::of_function!(ident::<..>)` | `pretty_name::of_function!(my_func::<..>)` → `"my_func"` |
| Generic function (include params) | `pretty_name::of_function!(ident::<T, U>)` | `pretty_name::of_function!(my_func::<u32, String>)` → `"my_func::<u32, String>"` |
| **Struct fields** | | |
| Field name | `pretty_name::of_field!(Type::field)` | `pretty_name::of_field!(MyStruct::field)` → `"MyStruct::field"` |
| Field name (on generic type) | `pretty_name::of_field!(<Type<T>>::field)` | `pretty_name::of_field!(<MyStruct<T>>::field)` → `"<MyStruct<T>>::field"` |
| Field name (on qualified type) | `pretty_name::of_field!(<module::Type>::field)` | `pretty_name::of_field!(<my_module::MyStruct>::field)` → `"<my_module::MyStruct>::field"` |
| **Methods** | | |
| Method name | `pretty_name::of_method!(Type::method)` | `pretty_name::of_method!(MyStruct::method)` → `"MyStruct::method"` |
| Method (on generic type) | `pretty_name::of_method!(<Type<T>>::method)` | `pretty_name::of_method!(<MyStruct<T>>::method)` → `"<MyStruct<T>>::method"` |
| Method (on qualified type) | `pretty_name::of_method!(<module::Type>::method)` | `pretty_name::of_method!(<my_module::MyStruct>::method)` → `"<my_module::MyStruct>::method"` |
| Generic method | `pretty_name::of_method!(Type::method::<T>)` | `pretty_name::of_method!(MyStruct::method::<u32>)` → `"MyStruct::method::<u32>"` |
| Generic method (on generic type) | `pretty_name::of_method!(<Type<T>>::method::<U>)` | `pretty_name::of_method!(<MyStruct<T>>::method::<u32>)` → `"<MyStruct<T>>::method::<u32>"` |
| Generic method (on qualified type) | `pretty_name::of_method!(<module::Type>::method::<T>)` | `pretty_name::of_method!(<my_module::MyStruct>::method::<u32>)` → `"<my_module::MyStruct>::method::<u32>"` |
| **Enum variants** | | |
| Unit variant | `pretty_name::of_variant!(Type::Variant)` | `pretty_name::of_variant!(MyEnum::UnitVariant)` → `"MyEnum::UnitVariant"` |
| Tuple variant | `pretty_name::of_variant!(Type::Variant(..))` | `pretty_name::of_variant!(MyEnum::TupleVariant(..))` → `"MyEnum::TupleVariant"` |
| Struct variant | `pretty_name::of_variant!(Type::Variant{..})` | `pretty_name::of_variant!(MyEnum::StructVariant{..})` → `"MyEnum::StructVariant"` |
| Variant (on generic type) | `pretty_name::of_variant!(<Type<T>>::Variant)` | `pretty_name::of_variant!(<MyEnum<u32>>::Variant)` → `"<MyEnum<u32>>::Variant"` |
| Variant (on qualified type) | `pretty_name::of_variant!(<module::Type>::Variant)` | `pretty_name::of_variant!(<my_module::MyEnum>::Variant)` → `"<MyEnum>::Variant"` |

**Notes:**
- Macros resolve `Self` to the appropriate type when used inside `impl` blocks.
- Use `<Type>` syntax for types with qualified paths or generic parameters.
- `of_type!(Type)` always uses semantic type-name resolution. Aliases, renamed imports,
  generic parameters, and `Self` therefore use their compiler-resolved types.
- Type names are intended for diagnostics. Rust does not guarantee that compiler type
  names are unique or stable between compiler versions, so they should not be used as
  persistent identifiers or serialization keys.
- Compiler-generated names that are not valid Rust type syntax, such as some closure
  descriptions, are returned unchanged rather than replaced with an error marker.

**To Get a String Literal:**
Some legacy identifier and compound macro forms still yield string literals, while forms that require compound semantic type resolution return cached `&'static str` values:
- `pretty_name::of_var!(var)` always yields a string literal.
- `pretty_name::of_function!(function)`: If *function* contains a single identifier.
- `pretty_name::of_method!(Type::method)`: If *Type* and *method* are both single identifiers and *Type* is not `Self`.
- `pretty_name::of_field!(Type::field)` and `pretty_name::of_variant!(Type::Variant | Type::Variant(..) | Type::Variant {..})`: If *Type* is a single identifier other than `Self`.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0)>
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Links

- [Repository](https://github.com/Nekomaru-PKU/pretty-name)
- [Documentation](https://docs.rs/pretty-name)
- [Crates.io](https://crates.io/crates/pretty-name)
