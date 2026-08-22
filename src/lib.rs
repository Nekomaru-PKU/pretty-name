#![doc = include_str!("../README.md")]

use std::any;
use std::fmt;

/// Private implementation details. It **MUST NOT** be considered a stable public
/// interface and should not be used outside of this crate.
///
/// The contents of this module are subject to change without notice, and any code
/// that depends on them is likely to break.
#[path = "impl.rs"]
#[doc(hidden)]
pub mod __;

/// Returns a shortened diagnostic name for the compiler-resolved type `T`.
///
/// This is a human-readable counterpart to [`std::any::type_name`]. Formatting
/// removes module qualification from parseable type and trait paths while preserving
/// references, pointers, tuples, function signatures, trait bounds, generic arguments,
/// and other type structure. If the compiler description cannot be transformed
/// confidently, it is displayed in full and unchanged.
///
/// The returned value implements [`Display`](fmt::Display) and borrows no caller-owned
/// value. Constructing it does not allocate; formatting it may allocate while parsing
/// and rendering the compiler description.
///
/// # Examples
///
/// ```rust
/// use pretty_name::type_name;
///
/// assert_eq!(type_name::<&str>().to_string(), "&str");
/// assert_eq!(type_name::<Option<i32>>().to_string(), "Option<i32>");
/// assert_eq!(type_name::<Vec<Box<dyn std::fmt::Debug>>>().to_string(), "Vec<Box<dyn Debug>>");
/// ```
///
/// Compiler type descriptions and their shortened forms are intended for presentation.
/// They are not guaranteed to be unique or stable between Rust versions.
pub fn type_name<T: ?Sized>() -> impl 'static + fmt::Display {
    crate::__::TypeName(any::type_name::<T>())
}

/// Returns a shortened diagnostic name for the compiler-resolved type of `value`.
///
/// This is a human-readable counterpart to [`std::any::type_name_of_val`]. Passing
/// `&value` identifies the type of `value`; it does not add that argument borrow to the
/// displayed type. A value which is itself a reference still displays its own reference
/// layer.
///
/// The function inspects only the value's type. The returned [`Display`](fmt::Display)
/// value does not retain the input borrow and may outlive the inspected value.
/// Formatting follows [`type_name`], including its unchanged fallback for compiler
/// descriptions that cannot be transformed confidently.
///
/// # Examples
///
/// ```rust
/// use pretty_name::type_name_of_val;
///
/// let value = vec![1, 2, 3];
/// assert_eq!(type_name_of_val(&value).to_string(), "Vec<i32>");
/// assert_eq!(type_name_of_val(&value.as_slice()).to_string(), "&[i32]");
///
/// let name = {
///     let temporary = String::from("temporary");
///     type_name_of_val(&temporary)
/// };
/// assert_eq!(name.to_string(), "String");
/// ```
pub fn type_name_of_val<T: ?Sized>(value: &T) -> impl fmt::Display + use<T> {
    crate::__::TypeName(any::type_name_of_val(value))
}

/// Returns the call-site spelling of a compiler-validated ordinary value path.
///
/// The path may name a binding, constant, static, or free function. Simple, module-
/// qualified, and leading-`::` paths are supported. The path itself remains lexical:
/// renamed imports and module aliases are displayed exactly as written at the macro call.
/// Type arguments are instead compiler-resolved and shortened with the same formatting
/// as [`type_name`]. The source turbofish is omitted from the displayed name.
///
/// For a generic function, every caller-provided generic argument must be written as a
/// concrete type. Inferred, omitted, or partial arguments, direct const arguments, and
/// the legacy `::<..>` placeholder are unsupported. Const arguments nested inside a
/// resolved type remain supported.
///
/// The macro validates the complete path inside an uncalled closure, so creating the
/// name does not read a binding or static, inspect a constant, or call a function.
/// Type-associated items use [`nameof_member!`](crate::nameof_member) instead.
/// The expansion produces an opaque value implementing [`Display`](fmt::Display).
///
/// # Examples
/// ```rust
/// let my_variable = 42;
/// const MY_CONSTANT: u32 = 42;
/// fn my_function<T>() {}
/// mod nested { pub fn function() {} }
///
/// assert_eq!(pretty_name::nameof!(my_variable).to_string(), "my_variable");
/// assert_eq!(pretty_name::nameof!(MY_CONSTANT).to_string(), "MY_CONSTANT");
/// assert_eq!(pretty_name::nameof!(my_function::<u32>).to_string(), "my_function<u32>");
/// assert_eq!(
///     pretty_name::nameof!(nested::function).to_string(),
///     "nested::function");
/// ```
///
/// Invalid or missing values are rejected at compile time:
///
/// ```compile_fail
/// let _ = pretty_name::nameof!(missing_value);
/// ```
#[macro_export]
macro_rules! nameof {
    ($ident:ident) => {{
        #[allow(warnings, reason = "macro-generated")] {
            let _ = || { let _ = &$ident; };
            $crate::__::ItemName {
                owner: ::core::option::Option::None,
                path: stringify!($ident),
                args: [],
            }
        }
    }};
    ($ident:ident ::<$($arg:ty),+ $(,)?>) => {{
        #[allow(warnings, reason = "macro-generated")] {
            let _ = || { let _ = &$ident::<$($arg),*>; };
            $crate::__::ItemName {
                owner: ::core::option::Option::None,
                path: stringify!($ident),
                args: [$($crate::__::TypeName(::core::any::type_name::<$arg>())),*],
            }
        }
    }};
    ($head:ident :: $($tail:ident)::+ ::<$($arg:ty),+ $(,)?>) => {{
        #[allow(warnings, reason = "macro-generated")] {
            use $head::$($tail)::+ as _;
            let _ = || { let _ = &$head::$($tail)::+::<$($arg),*>; };
            $crate::__::ItemName {
                owner: ::core::option::Option::None,
                path: concat!(stringify!($head), $("::", stringify!($tail)),+),
                args: [$($crate::__::TypeName(::core::any::type_name::<$arg>())),*],
            }
        }
    }};
    (:: $head:ident :: $($tail:ident)::+ ::<$($arg:ty),+ $(,)?>) => {{
        #[allow(warnings, reason = "macro-generated")] {
            use ::$head::$($tail)::+ as _;
            let _ = || { let _ = &::$head::$($tail)::+::<$($arg),*>; };
            $crate::__::ItemName {
                owner: ::core::option::Option::None,
                path: concat!("::", stringify!($head), $("::", stringify!($tail)),+),
                args: [$($crate::__::TypeName(::core::any::type_name::<$arg>())),*],
            }
        }
    }};
    ($head:ident :: $($tail:ident)::+) => {{
        #[allow(warnings, reason = "macro-generated")] {
            use $head::$($tail)::+ as _;
            let _ = || { let _ = &$head::$($tail)::+; };
            $crate::__::ItemName {
                owner: ::core::option::Option::None,
                path: concat!(stringify!($head), $("::", stringify!($tail)),+),
                args: [],
            }
        }
    }};
    (:: $head:ident :: $($tail:ident)::+) => {{
        #[allow(warnings, reason = "macro-generated")] {
            use ::$head::$($tail)::+ as _;
            let _ = || { let _ = &::$head::$($tail)::+; };
            $crate::__::ItemName {
                owner: ::core::option::Option::None,
                path: concat!("::", stringify!($head), $("::", stringify!($tail)),+),
                args: [],
            }
        }
    }};
    ($($invalid:tt)*) => {
        compile_error!(concat!(
            "expected an ordinary value path, ",
            "optionally followed by explicit type arguments"))
    };
}

/// Returns a shortened diagnostic name for a type written in macro syntax.
///
/// This is the macro counterpart to [`type_name`]. It accepts any type expression and
/// resolves aliases, renamed imports, generic parameters, and `Self` to the underlying
/// type selected by the compiler. Formatting then removes module qualification while
/// preserving the remaining type structure.
///
/// The expansion produces the same opaque [`Display`](fmt::Display) value as
/// [`type_name`].
///
/// # Examples
/// ```rust
/// type Bytes = std::vec::Vec<u8>;
/// struct MyGenericStruct<T>(std::marker::PhantomData<T>);
///
/// assert_eq!(pretty_name::nameof_type!(Bytes).to_string(), "Vec<u8>");
/// assert_eq!(
///     pretty_name::nameof_type!(MyGenericStruct<u32>).to_string(),
///     "MyGenericStruct<u32>");
/// ```
///
/// Invalid types are rejected at compile time:
///
/// ```compile_fail
/// let _ = pretty_name::nameof_type!(DefinitelyNotAType);
/// ```
#[macro_export]
macro_rules! nameof_type(($ty:ty) => {{
    #[allow(warnings, reason = "macro-generated")] {
        $crate::__::TypeName(::core::any::type_name::<$ty>())
    }
}});

/// Returns a validated field name with its compiler-resolved owner.
///
/// No owner value is required. The macro type-checks field access inside an uncalled
/// closure, so it neither constructs the owner nor accesses the field at runtime.
/// The displayed owner follows [`type_name`] formatting; aliases, renamed type imports,
/// generic parameters, and `Self` resolve to the type selected by the compiler.
///
/// A single-identifier owner uses `Type::field`. Qualified or generic owners must be
/// wrapped in angle brackets, as in `<module::Type<T>>::field`. The wrapper marks the
/// owner boundary in the macro input and is not included in the displayed name.
///
/// Owners must be named type paths. Anonymous types and qualified-self owner paths are
/// unsupported. Fields must be identifiers, so tuple-field indices are also unsupported.
/// The expansion produces an opaque value implementing [`Display`](fmt::Display).
///
/// # Examples
/// ```rust
/// struct MyStruct {
///     my_field: u32,
/// }
/// struct MyGenericStruct<T> {
///     my_field: T,
/// }
/// assert_eq!(
///     pretty_name::nameof_field!(MyStruct::my_field).to_string(),
///     "MyStruct::my_field");
/// assert_eq!(
///     pretty_name::nameof_field!(<MyGenericStruct<u32>>::my_field).to_string(),
///     "MyGenericStruct<u32>::my_field");
/// ```
///
/// Missing fields and incorrect owner types are rejected at compile time:
///
/// ```compile_fail
/// struct MyStruct;
/// let _ = pretty_name::nameof_field!(MyStruct::missing_field);
/// ```
#[macro_export]
macro_rules! nameof_field {
    ($owner:ident :: $field:ident) => {
        $crate::nameof_field!(<$owner>::$field)
    };
    (<$owner:path> :: $field:ident) => {{
        #[allow(warnings, reason = "macro-generated")] {
            let _ = |obj: $owner| { let _ = &obj.$field; };
            $crate::__::ItemName {
                owner: ::core::option::Option::Some(
                    ::core::any::type_name::<$owner>()),
                path: stringify!($field),
                args: [],
            }
        }
    }};
    ($($invalid:tt)*) => {
        compile_error!(
            "expected `Owner::field` or `<path::to::Owner<Args>>::field`")
    };
}

/// Returns a validated value-like member name with its compiler-resolved owner.
///
/// Members include associated constants, associated functions, methods, and unit or
/// tuple enum variants. No owner value or method receiver is required. The member is
/// resolved inside an uncalled closure and is therefore not evaluated, invoked, or
/// constructed.
///
/// A single-identifier owner uses `Type::member`. Qualified or generic owners must be
/// wrapped in angle brackets, as in `<module::Type<T>>::member`. The wrapper marks the
/// owner boundary in the macro input and is omitted from the displayed name. Aliases,
/// renamed imports, `Self`, and bounded type parameters resolve through the compiler.
/// Trait-provided methods must be named through a concrete implementor or bounded type
/// parameter rather than a bare trait declaration.
///
/// Owners must be named type paths; anonymous and qualified-self owners are unsupported.
/// Generic members must specify every caller-provided generic argument as a concrete
/// type. Inferred, omitted, or partial arguments, direct const arguments, and the legacy
/// `::<..>` placeholder are unsupported. Const arguments nested inside an owner or
/// another resolved type remain supported.
/// The expansion produces an opaque value implementing [`Display`](fmt::Display).
///
/// # Examples
/// ```rust
/// struct Owner<T>(T);
/// impl<T> Owner<T> {
///     const CONSTANT: u32 = 42;
///     fn function() {}
///     fn method<U>(&self) {}
/// }
/// enum Choice {
///     Unit,
///     Tuple(u32),
/// }
/// assert_eq!(
///     pretty_name::nameof_member!(<Owner<u8>>::CONSTANT).to_string(),
///     "Owner<u8>::CONSTANT");
/// assert_eq!(
///     pretty_name::nameof_member!(<Owner<u8>>::function).to_string(),
///     "Owner<u8>::function");
/// assert_eq!(
///     pretty_name::nameof_member!(<Owner<u8>>::method::<u32>).to_string(),
///     "Owner<u8>::method<u32>");
/// assert_eq!(
///     pretty_name::nameof_member!(Choice::Tuple).to_string(),
///     "Choice::Tuple");
/// ```
///
/// Struct variants are rejected because their paths are not first-class values:
///
/// ```compile_fail
/// enum Choice { Struct { value: u32 } }
/// let _ = pretty_name::nameof_member!(Choice::Struct);
/// ```
#[macro_export]
macro_rules! nameof_member {
    ($owner:ident :: $member:ident) => {
        $crate::nameof_member!(<$owner>::$member)
    };
    ($owner:ident :: $member:ident ::<$($arg:ty),+ $(,)?>) => {
        $crate::nameof_member!(<$owner>::$member::<$($arg),*>)
    };
    (<$owner:path> :: $member:ident) => {{
        #[allow(warnings, reason = "macro-generated")] {
            let _ = || <$owner>::$member;
            $crate::__::ItemName {
                owner: ::core::option::Option::Some(
                    ::core::any::type_name::<$owner>()),
                path: stringify!($member),
                args: [],
            }
        }
    }};
    (<$owner:path> :: $member:ident ::<$($arg:ty),+ $(,)?>) => {{
        #[allow(warnings, reason = "macro-generated")] {
            let _ = || <$owner>::$member::<$($arg),*>;
            $crate::__::ItemName {
                owner: ::core::option::Option::Some(
                    ::core::any::type_name::<$owner>()),
                path: stringify!($member),
                args: [$($crate::__::TypeName(::core::any::type_name::<$arg>())),*],
            }
        }
    }};
    ($($invalid:tt)*) => {
        compile_error!(concat!(
            "expected `Owner::member` or `<path::to::Owner<Args>>::member`, ", "optionally followed by explicit type arguments"))
    };
}
