# M1 Plan: Verified Diagnostic Names

Status: Design accepted; implementation not started  
Target: `1.0`  
Scope: A correctness-first redesign of type, identifier, function, member, and variant names

## Milestone outcome

M1 establishes `pretty-name` as a crate for readable diagnostic names that combine two sources of information:

- **Resolved types** come from the compiler description returned by `core::any::type_name`.
- **Source identifiers** come from the identifier written at the macro call after ordinary Rust code verifies that the referenced item exists and has the expected shape.

The public result is an opaque value that implements `Display`. Formatting a name with `{}` produces the shortened human-readable representation, while the standard `ToString` blanket implementation provides an explicitly owned `String` when needed.

M1 prioritizes, in order:

1. Correct names and uncompromised compiler validation.
2. Robust handling of Rust syntax and unfamiliar compiler descriptions.
3. A small, predictable public API.
4. Performance measured after the behavior and feature set are correct.

M1 does not promise zero allocation, lock-free operation, `no_std`, a dependency-free implementation, minimum binary size, or compatibility with earlier releases. Any public claim must be backed by the completed implementation and its tests.

## Semantic contract

### Resolved types

Every type-bearing component is obtained through the compiler rather than from the spelling passed to a macro. This rule applies uniformly to:

- Direct type names.
- Owner types of fields, methods, and variants.
- Type arguments of functions and methods.
- Type aliases and renamed imports.
- Generic type parameters and `Self` after monomorphization.
- Const arguments nested inside a resolved type, such as `[u8; 16]` or `Array<u8, 16>`.

A type alias is therefore transparent. If `Alias` names `crate::model::Record`, `of_type!(Alias)` describes the resolved `Record` type rather than preserving the token `Alias`.

`core::any::type_name` is diagnostic input, not reflection, stable identity, or a serialization key. Its output may be incomplete or may change with a compiler release. Such a change is not a `pretty-name` defect unless the crate corrupts, loses, or misrepresents the received description.

### Source identifiers

Rust does not provide stable reflection for the declared names of variables, functions, fields, methods, or variants. These components are captured with `stringify!` only after a real Rust expression, item reference, field access, or pattern validates the identifier.

A renamed function import therefore displays the identifier written at the macro call. This is intentional: the function component is a source identifier, while its generic arguments remain resolved types.

### Composition matrix

| Operation | Resolved type components | Source identifier components |
|---|---|---|
| `of_type!(T)` | `T` | None |
| `of_var!(value)` | None | `value` |
| `of_function!(function::<A>)` | `A` | `function` |
| `of_field!(T::field)` | `T` | `field` |
| `of_method!(T::method::<A>)` | `T`, `A` | `method` |
| `of_variant!(T::Variant)` | `T` | `Variant` |

There are no lexical or source-spelled type exceptions in M1.

## Public value contract

M1 uses a small set of category-specific, opaque value types. Their intended private representation is:

```rust
pub struct TypeName(&'static str);

pub struct IdentifierName(&'static str);

pub struct FunctionName {
    ident: &'static str,
    args: Box<[TypeName]>,
}

pub struct MemberName {
    owner: TypeName,
    ident: &'static str,
    args: Box<[TypeName]>,
}
```

`MemberName` represents fields, methods, and variants. Empty `args` distinguish non-generic members naturally without adding more public types. Public entry points construct these values; callers cannot construct arbitrary names.

The public surface is deliberately narrow:

- Every name type implements `Display`.
- Every name type derives `Debug` conventionally.
- Compound values own their generic argument list as `Box<[TypeName]>` on every compiler channel.
- Compound values do not implement `Copy`.
- No crate-specific common trait is introduced in M1.
- No `display()`, `raw()`, `original()`, `inner()`, component accessor, `From<&str>`, string comparison, or similar inspection API is provided.
- `.to_string()` is the standard fluent conversion for callers that need `String`.

Derived `Debug` is structural developer diagnostics, not a second name format. For example, it may expose `TypeName("crate::model::Record")` or private field labels such as `ident` and `args`. Its exact text is not a stable identity or serialization contract.

### Display grammar

`Display` uses one fixed M1 presentation policy:

- A type name omits module qualification from parseable type paths.
- Every other parsed type component is preserved, including references, lifetimes emitted by the compiler, mutability, raw-pointer qualifiers, tuple arity, arrays, slices, function signatures, trait bounds, associated-type bindings, and const arguments.
- A function is written as `ident` or `ident::<Args...>`.
- Every field, method, and variant uses the uniform qualified-owner form `<Owner>::ident` or `<Owner>::ident::<Args...>`.
- Arguments are separated by `, ` with no trailing comma.

For example:

```text
alloc::vec::Vec<crate::model::Record>  ->  Vec<Record>
function::<crate::model::Record>       ->  function::<Record>
<crate::model::Owner>::field           ->  <Owner>::field
```

Short names are diagnostic presentation, not unique identities. Two different resolved types may shorten to the same output. Qualification controls and other formatting customization are postponed until after `1.0`.

## Type-description parsing contract

Correctness depends on recognizing Rust type structure accurately. M1 therefore uses `syn` or an equivalently rigorous Rust grammar implementation for type descriptions that can be parsed as Rust syntax.

The following approaches are explicitly prohibited, including as future performance optimizations:

- Delimiter-depth scanning presented as parsing.
- Searching for the last `::` or similar path heuristics.
- Ad-hoc token deletion based on surrounding punctuation.
- A separate fast path whose behavior is weaker than the grammar-aware path.

If a compiler description cannot be parsed confidently, `Display` writes that description unchanged. It must not panic, emit an opaque error marker, silently drop text, or partially transform an uncertain shape. Compound values may still compose an unchanged type component with their validated source identifiers. The only formatting error returned is an error from the destination formatter.

`TypeName` stores the compiler's `&'static str` and defers parsing and shortening until `Display` is invoked. The implementation may allocate while parsing or rendering. Repeated formatting may repeat that work. These costs are accepted until measurement demonstrates a real problem.

### Dependency policy

`syn` is an accepted correctness dependency. Maintaining a private subset of Rust's evolving type grammar would duplicate substantial work and recreate the failure mode this crate is intended to avoid, especially for generated APIs such as `windows-rs`.

`quote` and `prettyplease` are not automatically required or prohibited. Stage 2 must retain either dependency only when it contributes to grammar-preserving output that would otherwise require fragile custom logic. Compile time, binary size, and formatting cost are relevant measurements, but they do not override correctness.

## Macro validation contract

Name production and validation are separate responsibilities inside each macro:

```text
ordinary Rust syntax
    -> rustc verifies the item, owner, argument count, argument kinds, and shape

stringify!(ident) plus TypeName construction
    -> produces the source identifier and resolved type components
```

No macro arm may weaken validation merely to accept more inputs. If rustc cannot form the referenced item with the supported explicit syntax, `pretty-name` does not claim to name it.

Fields, methods, and variants accept named owner paths written with ordinary Rust path syntax:

```rust
of_field!(Type::field)
of_method!(module::Type::method)
of_method!(Type::<OwnerArgs>::method::<MethodArgs>)
of_variant!(Enum::<Args>::Variant)
```

`Self`, aliases, and bounded type parameters are supported owner paths. Extra angle-qualified owners, qualified-self owners, and anonymous owner types are not supported; this excludes `<Type<Args>>::member`, `<T as Trait>::member`, and `<&T>::member`. Trait-provided methods are named through their concrete implementor or a bounded type parameter such as `T::method`, because a bare trait declaration is not a resolved owner type.

An implementation-only `macro_rules!` token partitioner separates the final member from the named owner path. It recognizes separators and balances owner turbofish tokens without deriving semantic information. Rustc then reparses the preserved owner in a type position and validates the emitted field access, method item, or variant pattern. This macro-input partitioning is distinct from the prohibited heuristic parsing of compiler-generated type descriptions.

### Generic functions and methods

M1 supports function and method generic arguments only when every required, caller-specifiable generic argument is a type and is written explicitly.

```rust
of_function!(function::<u32, String>)
of_method!(Owner::method::<u32, String>)
```

Validation forms the complete function item or associated method item, such as `let _ = &function::<u32, String>;`. The same rule applies to free functions and associated methods.

The following forms are unsupported:

- Omitting type arguments and relying on inference to determine the function item.
- `_` placeholders in the generic argument list.
- The existing `::<..>` placeholder.
- Direct const generic arguments such as `function::<16>`.
- Generic items whose item type cannot be fully determined through the supported explicit type arguments, including inference-heavy generated methods.
- Any unchecked arm that merely stringifies a generic item.

Late-bound lifetimes and compiler-synthetic parameters cannot normally be written in a turbofish and are not displayed as function arguments. A function or method that cannot be validated under this contract is outside M1. An external macro may resolve a domain-specific call and compose `pretty-name` values without weakening this crate's guarantees.

Direct const generic arguments are excluded only from function and method argument lists. They remain supported wherever they occur inside a resolved type description.

### Validation requirements by macro

| Macro | Required rustc validation | Result type |
|---|---|---|
| `of_type!` | The input is a valid type, including `?Sized` forms where supported | `TypeName` |
| `of_var!` | The binding or constant resolves | `IdentifierName` |
| `of_function!` | The complete function item resolves under the generic contract | `FunctionName` |
| `of_field!` | Field access type-checks for the owner | `MemberName` |
| `of_method!` | The complete associated method item resolves | `MemberName` |
| `of_variant!` | The unit, tuple, or struct variant pattern has the requested shape | `MemberName` |

Misspelled items, wrong variant shapes, missing generic arguments, unsupported const arguments, and invalid owner types must fail during compilation.

## Ownership and runtime policy

`Box<[TypeName]>` gives every compound name one owned, lifetime-independent representation. This deliberately accepts construction-time allocation for generic argument lists instead of exposing const-generic arity in the public type or creating compiler-channel-dependent behavior.

M1 removes the legacy process-wide and call-site caches. The core design does not require global state, locking, cache poisoning recovery, or intentionally leaked result strings. In particular, `Box::leak` or an equivalent operation must not be used solely to manufacture a `'static` lifetime.

Caching is composition rather than core naming semantics. An optional helper based on an established caching crate may be considered after the core API is correct, but it is not an M1 requirement and must not change displayed names or validation behavior.

No `unsafe` code is expected. Any proposal to introduce it requires a separate safety contract and justification.

## Toolchain policy

Rust 1.85, the first release supporting Edition 2024, is the initial baseline, not a promise to avoid clearer language or library features. The implementation should prefer clarity and maintainability over preserving that baseline. Raising the required stable version, or using nightly when it materially improves the design, is acceptable after an explicit review.

Nightly must not be introduced merely to avoid an ordinary owned value, and the crate must not acquire different name semantics across toolchains.

`std`, allocation, and third-party dependencies are acceptable. `no_std` and a dependency-free build may be revisited after `1.0` if they fall out naturally without fragmenting behavior.

## Linear implementation stages

Each stage begins only after the preceding exit criteria are satisfied. Tests for behavior introduced by a stage are part of that stage rather than deferred wholesale to the end.

### Stage 1: Freeze semantics and the public surface

Tasks:

- Encode the resolved-type and source-identifier vocabulary in crate-level design notes.
- Confirm the four opaque public value types and their private fields.
- Confirm the fixed `Display` grammar and derived `Debug` behavior.
- Inventory every existing public function and macro against the new result types.
- Record the generic-function exclusions as intentional limitations.

Exit criteria:

- Every public operation maps unambiguously to the semantic contract.
- No lexical type-name arm or raw inspection API remains in the design.
- The public API contains no abstraction without a concrete M1 consumer.

### Stage 2: Build the grammar-aware `TypeName` formatter

Tasks:

- Make `type_name::<T>()`, `type_name_of_val`, and `of_type!` return `TypeName`.
- Parse compiler descriptions with `syn`.
- Remove module qualification structurally while preserving every other parsed component.
- Preserve unparseable compiler descriptions unchanged.
- Derive `Debug` and implement `Display` without a public raw accessor.
- Audit whether `quote` and `prettyplease` are necessary for lossless output.

Correctness coverage includes primitives, aliases, imports, `Self`, references, raw pointers, arrays, slices, tuples, function types, trait objects, associated bindings, nested generics, consts nested in types, closures, async blocks, and unfamiliar or malformed diagnostic input.

Exit criteria:

- No supported type shape loses semantic information other than intentionally removed module qualification.
- Unsupported descriptions round-trip unchanged without panic.
- No heuristic parser or weaker fast path exists.

### Stage 3: Introduce opaque compound values

Tasks:

- Add `IdentifierName`, `FunctionName`, and `MemberName` with private fields.
- Store generic type arguments in `Box<[TypeName]>` for every compiler channel.
- Implement the fixed compound `Display` grammar.
- Derive conventional structural `Debug`.
- Keep construction internal to the crate and its exported macros.

Exit criteria:

- Values can be returned and stored without borrowed temporary data.
- Zero, one, and many arguments format with canonical punctuation.
- The only public ownership conversion is standard `.to_string()`.

### Stage 4: Rebuild macros around strict rustc validation

Tasks:

- Rebuild every macro so validation uses real Rust syntax before capturing a source identifier.
- Apply one explicit-type-argument rule to functions and associated methods.
- Resolve all owner and argument types through `TypeName`.
- Accept only named member-owner paths written with ordinary path and turbofish syntax.
- Preserve IDE completion where possible without weakening validation.
- Reject placeholders, inferred arguments, direct const generic arguments, and wrong item shapes with compile-time errors.

Exit criteria:

- Invalid identifiers and shapes fail to compile in every macro category.
- Functions and associated methods enforce the same generic guarantee.
- Alias, import, generic-parameter, and `Self` behavior agrees across all macros.
- No supported form relies on `use` as a substitute for a fully resolved function item.

### Stage 5: Remove legacy runtime and compatibility infrastructure

Tasks:

- Remove process-wide and per-call-site caches, locks, and cache keys.
- Remove leaked strings and `&'static str` compatibility paths.
- Remove the `::<..>` placeholder and lexical simple-type shortcuts.
- Remove documentation and tests for old pointer identity, cache behavior, and compile-time string results.
- Simplify modules and dependencies only where doing so preserves the accepted behavior.

Exit criteria:

- No public operation returns a legacy string solely for compatibility.
- No naming operation requires global mutable state or synchronization.
- No allocation or dependency claim exceeds what the implementation proves.

### Stage 6: Complete the correctness corpus

Use complementary test layers:

- Focused unit tests for each formatter transformation and unchanged fallback.
- Integration tests for every public function, value type, and macro category.
- Compile-pass tests for supported aliases, imports, `Self`, generic owners, qualified owners, and fully explicit generic functions and methods.
- Compile-fail tests for misspellings, wrong field and variant shapes, omitted generic arguments, `_`, `::<..>`, direct const generic arguments, and inference-heavy unsupported methods.
- Rustdoc tests for normal `Display`, `.to_string()`, and derived `Debug` use.
- A broad compiler-description corpus with individually diagnosable cases.

`windows-rs` is a primary regression target rather than an anecdote. The suite must include representative generated module and type shapes that heuristic name crates mishandle. At least one realistic fixture should exercise the exact failure family that motivated this redesign.

Tests should normally demonstrate one behavior each. Table-driven formatter cases are appropriate when each row remains individually identifiable; compile-fail cases should remain separately named so a failure identifies the broken guarantee immediately.

Exit criteria:

- The supported behavior matrix has positive and negative coverage.
- Every known `windows-rs` regression case passes without a heuristic exception.
- Unknown compiler descriptions cannot cause a naming panic or silent loss.

### Stage 7: Stabilize documentation, positioning, and measured costs

Tasks:

- Rewrite the README and rustdoc around correctness and the two-source semantic model.
- Document boxed generic arguments, unsupported direct const arguments, and inference-heavy generic limitations prominently.
- State that compiler type descriptions and derived `Debug` output are diagnostic and unstable.
- Audit the public API again and remove anything without a demonstrated user.
- Compare neighboring crates with concrete, reproducible cases, including `windows-rs`, rather than broad superiority claims.
- Measure construction, parsing, formatting, repeated formatting, and explicit `.to_string()` costs in release builds after correctness is complete.

Performance measurements characterize the implementation; they do not become README guarantees automatically. Binary size and dependency count may be reported for context but are not release gates. No migration guide or compatibility shim is required: downstream users should treat `1.0` as a new crate with new aims despite the coincidentally reused package name.

Exit criteria:

- README and rustdoc describe only implemented, tested behavior.
- Ecosystem comparisons are specific, fair, and reproducible.
- The minimal public surface is accepted for `1.0`.
- Tests and Clippy pass on the supported toolchain.

## Explicitly deferred features

The following are outside M1 and may be reconsidered only after the `1.0` core is correct:

- Direct const generic arguments to functions or methods.
- Domain-specific resolution for inference-heavy generated APIs.
- Formatting options, qualification depth, and alternate presentation styles.
- Raw or component accessors and string-like behavior.
- A shared public name trait.
- Declaration-side or enclosing-function discovery.
- Built-in or exported caching helpers.
- `no_std`, dependency-free, or stable/nightly storage variants.
- Compatibility helpers for pre-`1.0` behavior.

## Milestone risks and responses

| Risk | Response |
|---|---|
| Compiler descriptions change | Treat them as diagnostic input, preserve unparseable text, and maintain a compiler-description corpus. |
| Grammar-aware parsing costs more | Accept the cost for correctness, then measure it after the feature set is stable. |
| `syn` cannot parse a compiler-generated description | Return that description unchanged instead of guessing. |
| Macro syntax accepts an item it cannot fully validate | Reject the form; never introduce an unchecked fallback. |
| Boxed argument lists allocate | Document the allocation and revisit it only with evidence and without changing semantics. |
| Shortened names collide | Document that `Display` is presentation rather than identity. |
| IDE completion regresses | Test representative editor-facing forms and prefer native-looking macro syntax where guarantees remain intact. |
| Comparisons become promotional rather than factual | Tie every comparison to a reproducible input and observed output. |

## M1 definition of done

M1 is complete when:

- Every type-bearing component is a resolved type with no lexical exception.
- Every source identifier is validated by ordinary Rust syntax.
- Generic functions and methods require every supported type argument explicitly and reject unsupported generic forms.
- Public names are opaque values implementing `Display` and derived `Debug`.
- Compound generic arguments use `Box<[TypeName]>` consistently.
- Type shortening is grammar-aware, and uncertain input is preserved unchanged.
- No heuristic scanner, global name cache, synchronization primitive, or lifetime-elimination leak remains.
- The complete positive, negative, and `windows-rs` correctness corpus passes.
- README and rustdoc state all limitations and make no unverified guarantees.
- The public API has been minimized and accepted as the new `1.0` contract.
