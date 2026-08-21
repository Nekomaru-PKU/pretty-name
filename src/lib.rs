#![doc = include_str!("../README.md")]

use std::any;
use std::fmt;

/// Private implementation details. It **MUST NOT** be considered a stable public
/// interface and should not be used outside of this crate.
///
/// The contents of this module are subject to change without notice, and any code
/// that depends on them is likely to break.
#[path ="impl.rs"]
#[doc(hidden)]
pub mod __;

/// Gets a diagnostic name for the compiler-resolved type `T`.
///
/// Formatting removes module qualification from parseable Rust type paths while
/// preserving the remaining type structure. Compiler descriptions outside the
/// supported grammar are displayed unchanged.
///
/// Returns an opaque wrapper that implements [`Display`](fmt::Display) and may
/// outlive `T`.
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
pub fn type_name<T: ?Sized>() -> impl 'static + fmt::Display {
    crate::__::TypeName(any::type_name::<T>())
}

/// Gets a diagnostic name for the compiler-resolved type of `value`.
///
/// Formatting removes module qualification from parseable Rust type paths while
/// preserving the remaining type structure. Compiler descriptions outside the
/// supported grammar are displayed unchanged.
///
/// Passing a reference inspects the referenced value's type rather than adding
/// another reference layer to the description.
///
/// Returns an opaque wrapper that implements [`Display`](fmt::Display) and may
/// outlive `T`.
///
/// # Examples
///
/// ```rust
/// use pretty_name::type_name_of_val;
///
/// let value = vec![1, 2, 3];
/// assert_eq!(type_name_of_val(&value).to_string(), "Vec<i32>");
/// assert_eq!(type_name_of_val(&value.as_slice()).to_string(), "&[i32]");
/// ```
pub fn type_name_of_val<T: ?Sized>(value: &T) -> impl fmt::Display + use<T> {
    crate::__::TypeName(any::type_name_of_val(value))
}

/// Gets the validated lexical path of an ordinary value.
///
/// The path may name a binding, constant, static, function, or other value-like item.
/// Generic arguments must be explicit concrete types. The referenced value is resolved
/// inside an uncalled closure and is therefore not evaluated. Qualified paths retain
/// their source spelling, while explicit generic arguments use compiler-resolved type
/// names. A qualified path must also be importable, which keeps associated items in the
/// [`nameof_member!`](crate::nameof_member) macro's resolution mode.
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
/// Missing values are rejected at compile time:
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

/// Gets a diagnostic name for the compiler-resolved type.
///
/// This macro resolves aliases, renamed imports, generic parameters, and `Self` to the
/// underlying type selected by the compiler.
///
/// # Examples
/// ```rust
/// struct MyStruct;
/// struct MyGenericStruct<T>(std::marker::PhantomData<T>);
/// assert_eq!(pretty_name::nameof_type!(MyStruct).to_string(), "MyStruct");
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

/// Gets a validated field name with its compiler-resolved owner.
///
/// This macro resolves `Self` to the appropriate type when used inside an `impl` block.
///
/// A single-identifier owner uses `Type::field`. Qualified or generic owners must be
/// wrapped in angle brackets, as in `<module::Type<T>>::field`. The wrapper makes the
/// owner boundary explicit while preserving ordinary Rust syntax and name resolution.
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
/// Missing fields are rejected at compile time:
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
                    $crate::__resolved_type_name::<$owner>()),
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

/// Gets a validated value-like member name with its compiler-resolved owner.
///
/// Members include associated constants, associated functions, methods, and unit or
/// tuple enum variants. The member is resolved inside an uncalled closure and is
/// therefore not evaluated or invoked.
///
/// A single-identifier owner uses `Type::member`. Qualified or generic owners must be
/// wrapped in angle brackets, as in `<module::Type<T>>::member`. The wrapper identifies
/// the owner in macro input but is omitted from the displayed name.
///
/// A generic member must specify every caller-provided generic argument, and every
/// argument must be a concrete type. Inferred arguments, direct const arguments, and
/// the legacy `::<..>` placeholder are intentionally unsupported.
///
/// # Examples
/// ```rust
/// struct Owner;
/// impl Owner {
///     const CONSTANT: u32 = 42;
///     fn function() {}
///     fn method<T>(&self) {}
/// }
/// enum Choice {
///     Unit,
///     Tuple(u32),
/// }
/// assert_eq!(
///     pretty_name::nameof_member!(Owner::CONSTANT).to_string(),
///     "Owner::CONSTANT");
/// assert_eq!(
///     pretty_name::nameof_member!(Owner::function).to_string(),
///     "Owner::function");
/// assert_eq!(
///     pretty_name::nameof_member!(Owner::method::<u32>).to_string(),
///     "Owner::method<u32>");
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
                    $crate::__resolved_type_name::<$owner>()),
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
                    $crate::__resolved_type_name::<$owner>()),
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
