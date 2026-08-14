# M1 Design Contract: Verified Diagnostic Names

Status: Frozen by M1 Stage 1; implementation remains staged  
Milestone: `1.0`  
Source plan: [`M1-Plan.md`](M1-Plan.md)

## Purpose

M1 makes `pretty-name` a correctness-first crate for readable diagnostic names.
Every displayed name is composed from information with one of two proven origins:

- A **resolved type** is the compiler description returned by
  `core::any::type_name` or `core::any::type_name_of_val`.
- A **source identifier** is the identifier written at a macro call, captured with
  `stringify!` only after ordinary Rust syntax verifies the referenced item.

These origins are deliberately not interchangeable. Rust provides resolved type
descriptions, but it does not provide stable reflection for the declared names of
variables, functions, fields, methods, or variants. In particular, a type alias is
transparent while a renamed function import retains the identifier used at the macro
call.

The resulting names are presentation for diagnostics. They are neither unique nor
stable identities, and they are not suitable as serialization keys.

## Public value types

M1 has exactly four public value types. Their fields are private, and their intended
representations are:

```rust
#[derive(Debug)]
pub struct TypeName(&'static str);

#[derive(Debug)]
pub struct IdentifierName(&'static str);

#[derive(Debug)]
pub struct FunctionName {
    ident: &'static str,
    args: Box<[TypeName]>,
}

#[derive(Debug)]
pub struct MemberName {
    owner: TypeName,
    ident: &'static str,
    args: Box<[TypeName]>,
}
```

`MemberName` represents fields, methods, and variants. An empty `args` slice represents
a non-generic member without adding another public category.

Each type implements `Display` and derives conventional structural `Debug`. The M1
contract does not add `Copy`, `Clone`, equality, ordering, hashing, iteration, or a
crate-specific common trait. In particular, compound values are not `Copy`.
`ToString` is available through its standard blanket implementation for `Display`.

The documented functions and macros are the only supported construction interface.
The private fields prevent direct struct construction. Any `#[doc(hidden)]` support
needed for an exported macro to cross the downstream-crate privacy boundary is an
unstable implementation detail, not a supported constructor or extension point.

M1 provides no `display()`, `raw()`, `original()`, `inner()`, component accessor,
`From<&str>`, string comparison, or other inspection API. Derived `Debug` may expose
the structural representation, including private field labels, but its text is not a
second name format or a stable contract.

## Public operation inventory

Every currently exported function and macro has one unambiguous M1 disposition:

| Existing operation | Information source | M1 result | M1 disposition |
|---|---|---|---|
| `type_name::<T>()` | Resolved `T` | `TypeName` | Retained with a new return type. |
| `type_name_of_val(&value)` | Resolved type of `value` | `TypeName` | Retained with a new return type. |
| `of_type!(T)` | Resolved `T` | `TypeName` | Retained; every arm resolves the type. |
| `of_var!(ident)` | Validated source `ident` | `IdentifierName` | Retained with a new result type. |
| `of_function!(ident)` | Validated source `ident` | `FunctionName` | Retained for functions whose item type is fully determined. |
| `of_function!(ident::<A, ...>)` | Source `ident`; resolved argument types | `FunctionName` | Retained only for complete explicit type arguments. |
| `of_field!(T::field)` | Resolved owner `T`; source `field` | `MemberName` | Retained with uniform owner formatting. |
| `of_method!(T::method)` | Resolved owner `T`; source `method` | `MemberName` | Retained when the associated item is fully determined. |
| `of_method!(T::method::<A, ...>)` | Resolved owner and argument types; source `method` | `MemberName` | Retained only for complete explicit type arguments. |
| `of_variant!(T::Variant)` and shaped forms | Resolved owner `T`; source `Variant` | `MemberName` | Retained; the requested variant shape is validated. |
| `__with_cache!` | Legacy runtime infrastructure | None | Removed without replacement; it is not part of the supported M1 API. |

No operation retains a lexical type-name arm. Aliases, renamed type imports, generic
parameters, and `Self` therefore behave consistently wherever a type component occurs.

## Display grammar

`Display` has one fixed presentation policy.

### Types

`TypeName` parses a compiler description as Rust type syntax. For a confidently parsed
type it removes module qualification structurally and preserves all other parsed type
information, including:

- References and compiler-emitted lifetimes.
- Reference mutability and raw-pointer qualifiers.
- Arrays, slices, tuple arity, grouping, and parentheses.
- Function qualifiers, ABI, arguments, and return types.
- Trait bounds and associated-type bindings.
- Generic type arguments and const arguments nested in resolved types.
- Qualified-self and other non-module type structure.

Qualification removal is an AST transformation, never a textual search for `::` or a
delimiter-depth scan. A special fast path may not implement weaker semantics.

If the compiler description cannot be parsed and transformed confidently, `Display`
writes the complete original description unchanged. It does not panic, partially
transform the text, discard information, or emit an error marker. The only formatting
error returned is one produced by the destination formatter.

### Identifiers and compound values

`IdentifierName` writes its validated source identifier unchanged.

The compound grammar is:

```text
function
function::<Arg1, Arg2>
<Owner>::member
<Owner>::member::<Arg1, Arg2>
```

Every field, method, and variant uses the same angle-bracketed owner form, including a
simple owner. Arguments are separated by `, ` and never have a trailing comma. Each
owner and argument is formatted through `TypeName`; no source-spelled type exception is
allowed.

For example:

```text
alloc::vec::Vec<crate::model::Record>  ->  Vec<Record>
function::<crate::model::Record>       ->  function::<Record>
crate::model::Owner + field            ->  <Owner>::field
```

Different resolved types may shorten to the same displayed name. M1 intentionally has
no qualification controls because display is presentation rather than identity.

## Macro validation contract

Each macro separates validation from name construction:

1. Ordinary Rust syntax resolves and type-checks the referenced item, owner, arguments,
   and requested shape.
2. `stringify!` captures only the validated source identifier, while `TypeName`
   construction captures every resolved type component.

The validation obligations are:

| Macro | Required compiler validation |
|---|---|
| `of_type!` | The input is a valid type, including supported `?Sized` types. |
| `of_var!` | The binding or constant resolves. |
| `of_function!` | The complete function item resolves. |
| `of_field!` | Field access type-checks for the resolved owner. |
| `of_method!` | The complete associated method item resolves. |
| `of_variant!` | A unit, tuple, or struct pattern matches the requested shape. |

No arm may accept an input by stringifying it without the corresponding compiler check.
Misspelled items, invalid owners, missing generic arguments, and wrong field or variant
shapes must fail during compilation.

## Generic function and method boundary

A function or method generic argument list is supported only when every required,
caller-specifiable argument is a type and every such type is written explicitly:

```rust
of_function!(function::<u32, String>)
of_method!(Owner::method::<u32, String>)
```

The macro validates the complete item, for example with a reference to
`function::<u32, String>`. Free functions and associated methods follow the same rule.

M1 intentionally rejects or does not support:

- Omitted arguments when the generic item type would require inference.
- `_` inference placeholders.
- The legacy `::<..>` placeholder.
- Direct const generic arguments to functions or methods.
- Partial generic argument lists.
- Inference-heavy generated items that cannot form a complete item under this syntax.
- Any unchecked arm that only stringifies a generic item.

Late-bound lifetimes and compiler-synthetic parameters are neither written in the
turbofish nor displayed. Const arguments remain supported when nested inside a resolved
type, such as `Array<u8, 16>`; only direct function and method const arguments are
excluded.

## Ownership, safety, and cost model

`TypeName` and `IdentifierName` borrow compiler or source strings with `'static`
lifetimes. `FunctionName` and `MemberName` own their argument lists as
`Box<[TypeName]>` on every supported compiler channel. This keeps returned values
independent of macro temporaries without exposing argument arity in their public types.

M1 makes the following implementation commitments:

- No naming operation depends on global mutable state, locks, or a call-site cache.
- No string is leaked solely to manufacture a `'static` result.
- No `unsafe` code is expected. Introducing it requires a separate safety proposal and
  documented invariants.
- Construction may allocate for a boxed generic argument list.
- Formatting may allocate while parsing or rendering a type.
- Repeated formatting may repeat parsing and rendering work.

These costs favor one representation and grammar-aware correctness over speculative
optimization. Performance work begins with release-build measurement after the behavior
and correctness corpus are complete.

## Dependency boundary

`syn` is the accepted grammar implementation for parseable compiler descriptions.
Replacing it with a private Rust-type parser would duplicate evolving language work and
would violate the correctness goal that motivated M1.

Stage 2 audits `quote` and `prettyplease` separately. Either dependency remains only if
it materially supports grammar-preserving output; neither may be replaced by heuristic
text rewriting. M1 Stage 1 introduces no new dependency.

## Stage 1 exit record

- Every public naming operation maps to exactly one of the four result types in the
  public operation inventory.
- Every type-bearing position is resolved; there is no lexical type-name exception in
  the contract.
- Every identifier-bearing position has an explicit compiler-validation obligation.
- The `Display` grammar and structural `Debug` role are fixed.
- Generic-function and method exclusions are recorded as intentional limitations.
- No raw inspection API, common name trait, cache abstraction, or other public surface
  without a concrete M1 consumer is included.
