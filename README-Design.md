# Design: Verified Diagnostic Names

This document records the principles and trade-offs behind `pretty-name`. Public API
documentation and tests define the supported interface and exact behavior; this document
explains why that interface is shaped as it is.

## Documentation principles

Design and public documentation follow these additional rules:

1. Avoid tables with three or more columns. Prose, focused lists, and paired examples are
   easier to read when a design has caveats or asymmetrical cases.
2. Public documentation demonstrates supported use. Inputs which happen to compile but do
   not express the intended semantics are recorded here instead of advertised as API forms.
3. Avoid side effects in naming operations. Any unavoidable side effect must be documented
   where users choose the operation and in this design document.

## Purpose

`pretty-name` produces readable names for diagnostics. Every displayed component comes from
one of two sources:

1. A **resolved type** is the compiler description returned by `core::any::type_name` or
   `core::any::type_name_of_val`.
2. A **source identifier** is the identifier written at a macro call. It is captured with
   `stringify!` only alongside ordinary Rust syntax that verifies the referenced item.

These sources are deliberately different. Rust exposes resolved type descriptions, but it
does not expose stable reflection for the declared names of values, functions, fields,
methods, or variants. A type alias is consequently transparent, while a renamed value import
retains the identifier written at the call site.

Names are diagnostic presentation rather than unique or stable identities. They must not be
used as serialization keys.

## Public value model

Every operation returns one opaque `PrettyName`. A single result type keeps generic code and
glob imports simple while the private representation preserves the semantic difference
between a resolved type and a composed item name.

The private representation has two cases:

1. A type stores one compiler-resolved type description.
2. An item stores an optional resolved owner type, one compiler-validated source identifier,
   and zero or more resolved generic argument types.

`PrettyName` implements `Display` and formats its semantic components on demand. It implements
`Debug` explicitly as `PrettyName(<display output>)`, which exposes no private representation
details. The private representation intentionally does not implement `Debug`.

Fields remain private. Documented functions and macros are the supported construction
interface. Public `#[doc(hidden)]` functions exist only because exported macros must construct
values across a downstream crate's privacy boundary; they are not supported constructors or
extension points.

## Information sources

The functions use only resolved type information:

1. `type_name::<T>()` and `nameof_type!(T)` store the compiler description of `T`.
2. `type_name_of_val(&value)` stores the compiler description of the value's type.

The macros for values and members combine source identifiers with resolved types:

1. `nameof!(path)` stores the final source identifier. Explicit generic arguments are stored
   as resolved types.
2. `nameof_member!(Owner::member)` additionally stores the resolved owner type.
3. `nameof_field!(Owner::field)` stores the resolved owner type and field identifier.

Aliases, renamed type imports, generic parameters, and `Self` behave consistently in every
type-bearing position because rustc supplies their resolved types.

## Display policy

`Display` applies one predictable presentation policy. It shortens type paths for readability
while preserving type structure, then composes item components with Rust-like punctuation.

### Types

Compiler descriptions are parsed as Rust type syntax. When parsing succeeds, the formatter
removes module qualification structurally while preserving:

1. References, compiler-emitted lifetimes, and mutability.
2. Raw-pointer qualifiers.
3. Arrays, slices, tuple arity, grouping, and parentheses.
4. Function qualifiers, ABI, arguments, and return types.
5. Trait bounds and associated-type bindings.
6. Generic type and const arguments nested in resolved types.
7. Qualified-self and other non-module type structure.

Qualification removal is an AST transformation rather than a textual search for `::` or a
delimiter-depth scan. Every formatting path must preserve the same structural guarantees.

If a compiler description cannot be parsed and transformed confidently, `Display` writes the
complete original description unchanged. It does not panic, partially transform the text,
discard information, or emit an error marker. The only formatting error returned is one from
the destination formatter.

### Values and members

Composed names use this grammar:

```text
identifier
identifier<Arg1, Arg2>
Owner::member
Owner<OwnerArg>::member<MemberArg>
```

An owner is formatted as an ordinary type without wrapping the entire type in angle brackets.
Generic arguments omit the source turbofish separator. This matches the function-item style
produced by `core::any::type_name_of_val`:

```text
crate::model::Owner<crate::model::Record>  ->  Owner<Record>
function::<crate::model::Record>           ->  function<Record>
Owner<Record> + method::<String>            ->  Owner<Record>::method<String>
```

Different resolved types can shorten to the same displayed name. This ambiguity is acceptable
because display output is presentation rather than identity.

## Macro validation

Each macro separates compiler validation from name construction:

1. Ordinary Rust syntax resolves and type-checks the referenced path, owner, field, and
   explicit arguments inside an uncalled closure.
2. `stringify!` captures the final validated source identifier.
3. Compiler type descriptions capture the resolved owner and generic type arguments.

No supported macro arm accepts an identifier without the corresponding compiler check.
Misspelled items, invalid owners, incomplete generic arguments, and incorrect fields therefore
fail during compilation.

### Four resolution modes

`nameof_type!` treats its complete input as a type.

`nameof!` treats its input as an ordinary value path. It supports bindings, constants,
statics, and free functions, optionally through module segments and with explicit generic type
arguments. Only the final source identifier is displayed.

Rust's expression namespace also allows some inputs whose spelling suggests a type boundary.
For example, `nameof!(Type::function)`, `nameof!(Enum::Unit)`, and
`nameof!(Enum::Tuple)` may compile. They are intentionally classified as unexpected usage:
`nameof!` treats the leading identifiers as ordinary path segments and consequently omits the
owner. Supported associated-item and variant spelling uses `nameof_member!`.

`nameof_member!` treats its first non-`<` owner path as a resolved type. Its member umbrella
includes associated constants, associated functions, methods, unit variants, and tuple-variant
constructors. Stable Rust validates that the associated item resolves; it does not expose the
item's declaration category. Struct variants remain unsupported because their paths are not
first-class values.

`nameof_field!` validates instance field access for a resolved owner type. This provides the
field-specific compiler check which associated-item syntax cannot express.

### Owner boundaries

A one-identifier owner uses the compact form. Qualified or generic owners are wrapped in
`<...>` to make the type boundary unambiguous to `macro_rules!`:

```rust
nameof_member!(Type::method)
nameof_member!(<module::Type>::method)
nameof_member!(<Type<OwnerArgs>>::method::<MemberArgs>)
nameof_field!(<module::Type<Args>>::field)
```

The wrapper exists only in macro input and is never part of displayed output. It lets a
declarative macro capture the owner directly as `path`, without a token partitioner or
procedural macro, while leaving the member visible to completion and refactoring tools.

`Self`, aliases, and bounded type parameters are valid compact owners because rustc resolves
them as types. Angle-wrapped simple owners are accepted as well. A trait-provided method is
named through its concrete implementor or a bounded type parameter such as `T::method`; a bare
trait declaration is not a resolved owner type. Qualified-self and anonymous owners such as
`<<T as Trait>::Owner>::member` and `<&T>::member` are outside the named-`path` contract.

### Generic arguments

Value and member generic arguments are supported only when every required,
caller-specifiable argument is a type and all such types are written explicitly:

```rust
nameof!(function::<u32, String>)
nameof_member!(Owner::method::<u32, String>)
```

Validation applies to the complete value or associated item. The grammar therefore excludes:

1. Omitted arguments when the item requires inference.
2. `_` inference placeholders.
3. `..` placeholders.
4. Direct const generic arguments to values or members.
5. Partial generic argument lists.
6. Items whose complete type cannot be determined from explicit type arguments.

Late-bound lifetimes and compiler-synthetic parameters are neither written nor displayed.
Const arguments remain supported when nested inside a resolved type, such as
`Array<u8, 16>`; only direct value and member const arguments are outside the input grammar.

## Side-effect policy

Naming should inspect compiler-visible structure without executing user code. Every validation
expression is placed inside an uncalled closure, so rustc resolves and type-checks it while the
closure body never runs:

1. `nameof!` does not read a value, call a function, or construct a tuple variant.
2. `nameof_member!` does not evaluate an associated value, invoke a function or method,
   construct a variant, or run a destructor.
3. `nameof_field!` does not access a field or trigger an autoderef because its field expression
   is never evaluated.
4. Local-value validation can establish a temporary compiler-visible shared borrow or closure
   capture. This invokes no user code; in particular, taking a direct reference does not call
   `Deref`.

Some implementation effects are unavoidable and are part of the contract:

1. Constructing a name with non-empty generic arguments allocates its boxed argument slice.
2. Formatting a type may allocate while parsing, transforming, and rendering its syntax.
3. `Display` writes to the caller-provided formatter, so destination-specific write behavior
   and formatting errors remain observable.
4. Allocation failure follows Rust's ordinary allocator-failure behavior.

Any future operation that must execute user code or introduce another side effect requires
explicit documentation both here and on the relevant public API.

## Ownership, safety, and cost model

Resolved compiler descriptions and source identifiers have `'static` lifetimes. The private
item representation owns its variable-length generic argument list as `Box<[&'static str]>`.
This direct representation avoids one allocation and one pointer indirection compared with
boxing the entire private enum while keeping `PrettyName` lifetime-independent and
arity-independent.

The implementation follows these constraints:

1. Naming does not depend on global mutable state, locks, or call-site caches.
2. Strings are not leaked to manufacture `'static` results.
3. The implementation uses no `unsafe` code. Introducing it requires a separate safety
   argument and documented invariants.
4. Constructing a non-generic name does not allocate.
5. Constructing a generic item name may allocate its argument slice.
6. Formatting a type may allocate; repeated formatting may repeat parsing and rendering work.

These costs favor a clear semantic representation and grammar-aware correctness. Performance
changes must preserve displayed names, validation behavior, and unchanged fallback for
unfamiliar compiler descriptions.

## Dependency choices

The formatter relies on established Rust syntax tooling instead of maintaining a private
subset of Rust's evolving type grammar:

1. `syn` parses compiler descriptions and exposes the type AST used for structural path
   shortening.
2. `quote` converts the transformed type AST into a temporary type alias accepted by the
   formatter.
3. `prettyplease` renders that alias with grammar-preserving punctuation and spacing.

Replacing a dependency must retain structural transformation, complete unchanged fallback,
and formatting guarantees. Compile time, binary size, and runtime cost remain relevant
measurements, but they do not justify a weaker parser or partial transformation.

## Extension boundaries

The public API stays focused on readable diagnostic presentation:

1. `PrettyName` does not expose raw strings, semantic components, string comparison, or
   construction from arbitrary strings.
2. The crate does not define a public name trait because `Display` and `ToString` cover the
   shared behavior.
3. Display qualification is fixed; callers cannot treat shortened output as a unique identity.
4. Caching is an application-level composition choice and cannot change validation semantics.
5. Additional macro forms must retain complete rustc validation and resolved type components.
