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
| `of_function!(module::function)` | None | `function` | `FunctionName` |
| `of_function!(<T>::function::<A, ...>)` | `A, ...` | `function` | `FunctionName` |
| `of_field!(T::field)` | `T` | `field` | `MemberName` |
| `of_method!(T::method)` | `T` | `method` | `MemberName` |
| `of_method!(T::method::<A, ...>)` | `T`, `A, ...` | `method` | `MemberName` |
| `of_variant!(T::Variant)` | `T` | `Variant` | `MemberName` |

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
function<Arg1, Arg2>
<Owner>::member
<Owner>::member::<Arg1, Arg2>
```

Fields, methods, and variants use the same angle-bracketed owner form, including for a
simple owner. Arguments are separated by `, ` without a trailing comma. Every owner
and generic argument is formatted through `TypeName`.

Function generic arguments omit the source turbofish separator to match the function
item style produced by `core::any::type_name_of_val`. Generic methods retain `::`
because their identifier is composed after an explicit owner path.

For example:

```text
alloc::vec::Vec<crate::model::Record>  ->  Vec<Record>
function::<crate::model::Record>       ->  function<Record>
crate::model::Owner + field            ->  <Owner>::field
```

Different resolved types may shorten to the same displayed name. This ambiguity is
acceptable because display output is presentation rather than identity.

## Macro validation

Each macro separates compiler validation from name construction:

1. Ordinary Rust syntax resolves and type-checks the referenced item, owner, and
   explicit arguments.
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
| `of_variant!` | The associated value or function-like constructor resolves. |

No macro arm may accept an identifier without the corresponding compiler check.
Misspelled items, invalid owners, incomplete generic arguments, and incorrect fields
fail during compilation.

### Member owner grammar

Member owners are named Rust paths with an explicit boundary. A single identifier can
use the compact form, while qualified or generic owners are wrapped in `<...>`:

```rust
of_field!(Type::field)
of_method!(<module::Type>::method)
of_method!(<Type<OwnerArgs>>::method::<MethodArgs>)
of_variant!(<module::Enum<Args>>::Variant)
```

The wrapper is macro input syntax composed entirely from ordinary Rust tokens. It lets
`macro_rules!` capture the owner directly as `path`, so no token partitioner or
procedural macro is required. It also leaves the member identifier visible to editor
completion and refactoring tools.

`Self`, aliases, and bounded type parameters remain valid compact owners because rustc
can resolve them as types. Angle-wrapped simple owners are accepted as well. A
trait-provided method is named through its concrete implementor or a bounded type
parameter such as `T::method`; a bare trait declaration is not a resolved owner type.
Qualified-self and anonymous owner types such as `<<T as Trait>::Owner>::member` and
`<&T>::member` are unsupported because they do not satisfy the named `path` contract.

### Function path grammar

Module-qualified free functions use their ordinary Rust path because only the final
identifier needs to be separated:

```rust
of_function!(module::function)
of_function!(module::function::<u32>)
```

A small declarative helper walks forward through `identifier::` module segments and
captures the final function identifier. It does not inspect or balance generic owner
tokens. Associated functions on qualified or generic types use the same explicit owner
boundary as members:

```rust
of_function!(<module::Type<u32>>::function)
of_function!(<module::Type<u32>>::function::<String>)
```

Only the final source identifier and any resolved function type arguments are stored;
the associated owner is validation context and is not part of `FunctionName` output.

### Variant boundary

`of_variant!` supports unit variants and tuple variants through the same bare path:

```rust
of_variant!(Enum::Unit)
of_variant!(<module::Enum<u32>>::Tuple)
```

A unit variant is a value and a tuple variant is a function-like constructor, so an
ordinary associated-item expression validates both without pattern-shaped macro
syntax. Struct variants are intentionally unsupported because their paths are not
first-class values.

The validation expression is placed inside an uncalled closure. Rustc still resolves
and type-checks its body, but naming a unit variant cannot construct a temporary or run
that enum's `Drop` implementation.

Stable Rust does not expose an item's declaration category through this expression.
An associated constant or function with the requested name therefore satisfies the
same check. This is an explicit semantic boundary: the macro guarantees that the owner
and associated item resolve, not that compiler reflection identified an enum variant.

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
