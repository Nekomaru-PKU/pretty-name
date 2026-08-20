# Design: Verified Diagnostic Names

This document explains the principles and trade-offs behind `pretty-name`. The public
API documentation and tests define the exact supported interface and behavior; this
document records why that interface is designed as it is.

## Purpose

`pretty-name` produces readable names for diagnostics. Every displayed name is
composed from information with one of two origins:

1. A **resolved type** is the compiler description returned by
   `core::any::type_name` or `core::any::type_name_of_val`.
2. A **source identifier** is the identifier written at a macro call, captured with
   `stringify!` only after ordinary Rust syntax verifies the referenced item.

These origins are deliberately not interchangeable. Rust provides resolved type
descriptions, but it does not provide stable reflection for the declared names of
variables, functions, fields, methods, or variants. A type alias is therefore
transparent, while a renamed function import retains the identifier used at the macro
call.

The resulting names are diagnostic presentation. They are neither unique nor stable
identities, and they are unsuitable as serialization keys.

## Public value model

The crate exposes four opaque name categories:

| Value | Semantic contents |
|---|---|
| `TypeName` | One compiler-resolved type description. |
| `IdentifierName` | One compiler-validated source identifier. |
| `FunctionName` | A validated function identifier and its resolved generic type arguments. |
| `MemberName` | A resolved owner type, a validated field, method, or variant identifier, and any resolved method type arguments. |

Each value implements `Display` and derives structural `Debug`. The standard
`ToString` blanket implementation provides an owned `String` when needed. The values
do not expose their stored strings or components because the displayed representation
is diagnostic output rather than an identity or reflection interface.

Fields remain private, and the documented functions and macros are the supported
construction interface. Public `#[doc(hidden)]` helpers exist only so exported macros
can construct values across a downstream crate's privacy boundary. They are not
supported constructors or extension points.

`MemberName` covers fields, methods, and variants because all three share the same
display structure. An empty generic-argument slice represents a non-generic member
without adding public categories.

## Information sources

Every operation has an explicit mapping from resolved types and source identifiers to
its result:

| Operation | Resolved type components | Source identifier components | Result |
|---|---|---|---|
| `type_name::<T>()` | `T` | None | `TypeName` |
| `type_name_of_val(&value)` | The type of `value` | None | `TypeName` |
| `of_type!(T)` | `T` | None | `TypeName` |
| `of_var!(value)` | None | `value` | `IdentifierName` |
| `of_function!(function)` | None | `function` | `FunctionName` |
| `of_function!(function::<A, ...>)` | `A, ...` | `function` | `FunctionName` |
| `of_field!(T::field)` | `T` | `field` | `MemberName` |
| `of_method!(T::method)` | `T` | `method` | `MemberName` |
| `of_method!(T::method::<A, ...>)` | `T`, `A, ...` | `method` | `MemberName` |
| `of_variant!(T::Variant)` and shaped forms | `T` | `Variant` | `MemberName` |

Aliases, renamed type imports, generic parameters, and `Self` behave consistently in
every type-bearing position because the compiler supplies their resolved types.

## Display policy

`Display` applies one predictable presentation policy. It shortens type paths for
readability while preserving type structure and composes identifiers with canonical
Rust-like punctuation.

### Types

`TypeName` parses a compiler description as Rust type syntax. When parsing succeeds,
the formatter removes module qualification structurally and preserves the remaining
type information, including:

1. References, compiler-emitted lifetimes, and mutability.
2. Raw-pointer qualifiers.
3. Arrays, slices, tuple arity, grouping, and parentheses.
4. Function qualifiers, ABI, arguments, and return types.
5. Trait bounds and associated-type bindings.
6. Generic type arguments and const arguments nested in resolved types.
7. Qualified-self and other non-module type structure.

Qualification removal is an AST transformation rather than a textual search for `::`
or a delimiter-depth scan. Every formatting path must preserve the same structural
guarantees.

If a compiler description cannot be parsed and transformed confidently, `Display`
writes the complete original description unchanged. It does not panic, partially
transform the text, discard information, or emit an error marker. The only formatting
error returned is one produced by the destination formatter.

### Identifiers and compound values

`IdentifierName` writes its validated source identifier unchanged. Compound values use
the following grammar:

```text
function
function::<Arg1, Arg2>
<Owner>::member
<Owner>::member::<Arg1, Arg2>
```

Fields, methods, and variants use the same angle-bracketed owner form, including for a
simple owner. Arguments are separated by `, ` without a trailing comma. Every owner
and generic argument is formatted through `TypeName`.

For example:

```text
alloc::vec::Vec<crate::model::Record>  ->  Vec<Record>
function::<crate::model::Record>       ->  function::<Record>
crate::model::Owner + field            ->  <Owner>::field
```

Different resolved types may shorten to the same displayed name. This ambiguity is
acceptable because display output is presentation rather than identity.

## Macro validation

Each macro separates compiler validation from name construction:

1. Ordinary Rust syntax resolves and type-checks the referenced item, owner,
   arguments, and requested shape.
2. `stringify!` captures only the validated source identifier, while `TypeName`
   construction captures every resolved type component.

The compiler validation obligations are:

| Macro | Required validation |
|---|---|
| `of_type!` | The input is a valid type, including supported `?Sized` types. |
| `of_var!` | The binding or constant resolves. |
| `of_function!` | The complete function item resolves. |
| `of_field!` | Field access type-checks for the resolved owner. |
| `of_method!` | The complete associated method item resolves. |
| `of_variant!` | A unit, tuple, or struct pattern matches the requested shape. |

No macro arm may accept an identifier without the corresponding compiler check.
Misspelled items, invalid owners, incomplete generic arguments, and incorrect field or
variant shapes fail during compilation.

### Member owner grammar

Member owners are named Rust paths written with ordinary path and turbofish syntax:

```rust
of_field!(Type::field)
of_method!(module::Type::method)
of_method!(Type::<OwnerArgs>::method::<MethodArgs>)
of_variant!(Enum::<Args>::Variant)
```

`Self`, aliases, and bounded type parameters are valid owner paths because rustc can
resolve them as types. Angle-qualified, qualified-self, and anonymous owner types such
as `<Type<Args>>::member`, `<T as Trait>::member`, and `<&T>::member` are unsupported.
A trait-provided method is named through its concrete implementor or a bounded type
parameter such as `T::method`; a bare trait declaration is not a resolved owner type.

An implementation-only declarative macro partitions a named path from its final member
segment. It recognizes path separators and balanced owner turbofish tokens only to
preserve the caller's token sequence. Rustc then reparses the owner in a type position
and validates the emitted field access, method item, or variant pattern. This token
partitioning does not interpret compiler-generated type descriptions and therefore
does not weaken the grammar-aware `TypeName` formatting contract.

The struct-variant form is `T::Variant { field, .. }` and requires one real field name.
Rust accepts a bare `T::Variant { .. }` pattern for unit and tuple variants as well, so
the named field is necessary to validate the requested struct shape.

### Generic functions and methods

A function or method generic argument list is supported only when every required,
caller-specifiable argument is a type and every such type is written explicitly:

```rust
of_function!(function::<u32, String>)
of_method!(Owner::method::<u32, String>)
```

The macro validates the complete function or associated-method item. The supported
grammar therefore excludes:

1. Omitted arguments when the item requires inference.
2. `_` inference placeholders.
3. `..` placeholders.
4. Direct const generic arguments to functions or methods.
5. Partial generic argument lists.
6. Items whose complete type cannot be determined from explicit type arguments.

Late-bound lifetimes and compiler-synthetic parameters are neither written in the
turbofish nor displayed. Const arguments remain supported when nested inside a resolved
type, such as `Array<u8, 16>`; only direct function and method const arguments fall
outside the supported input grammar.

## Ownership, safety, and cost model

`TypeName` and `IdentifierName` borrow compiler or source strings with `'static`
lifetimes. `FunctionName` and `MemberName` own their generic argument lists as
`Box<[TypeName]>`. A boxed slice gives compound values a lifetime-independent,
arity-independent representation without exposing generic arity in their public types.

The implementation follows these constraints:

1. Naming operations do not depend on global mutable state, locks, or call-site
   caches.
2. Strings are not leaked to manufacture `'static` results.
3. The implementation uses no `unsafe` code. Introducing it requires a separate safety
   argument and documented invariants.
4. Constructing a compound value may allocate its boxed generic argument list.
5. Formatting a type may allocate while parsing and rendering.
6. Formatting the same value repeatedly may repeat that parsing and rendering work.

These costs favor one clear representation and grammar-aware correctness. Performance
changes must preserve displayed names, validation behavior, and unchanged fallback for
unfamiliar compiler descriptions.

## Dependency choices

The formatter relies on established Rust syntax tooling rather than maintaining a
private subset of Rust's evolving type grammar:

1. `syn` parses compiler descriptions and exposes the type AST used for structural path
   shortening.
2. `quote` converts the transformed type AST into a temporary type alias accepted by
   the formatter.
3. `prettyplease` renders that alias with grammar-preserving punctuation and spacing.

Replacing any of these dependencies must retain the structural transformation,
complete unchanged fallback, and formatting guarantees. Compile time, binary size, and
formatting cost are relevant measurements, but they do not justify a weaker parser or
partial transformation.

## Extension boundaries

The public API stays focused on readable diagnostic presentation:

1. Name values do not provide raw-string or component accessors, string comparison, or
   conversion from arbitrary strings.
2. The crate does not define a shared public name trait because the standard `Display`
   and `ToString` interfaces cover the common behavior.
3. Display qualification is fixed; callers cannot treat shortened output as a unique
   identity.
4. Caching is an application-level composition choice and cannot change naming or
   validation semantics.
5. Support for additional macro input forms must retain complete rustc validation and
   resolved type components.
