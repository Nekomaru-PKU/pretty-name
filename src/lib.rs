#![doc = include_str!("../README.md")]

mod type_name;

pub use type_name::type_name;

/// Get the human-friendly type name of the given value, removing visual clutter such as
/// full module paths.
/// 
/// # Examples
/// ```rust
/// use pretty_name::type_name_of;
/// let value = vec![1, 2, 3];
/// assert_eq!(type_name_of(&value), "Vec<i32>");
/// ```
pub fn type_name_of<T: ?Sized>(_: &T) -> String {
    type_name::<T>()
}

/// Get the name of the given identifier.
/// 
/// This macro can be used to get the name of local variables, contants and functions.
/// 
/// This macro checks that the identifier is valid in the current scope. If the identifier
/// is renamed via refactoring tools, the macro call will be updated accordingly.
/// 
/// # Examples
/// ```rust
/// fn my_function() {}
/// let my_variable = 42;
/// assert_eq!(pretty_name::of!(my_function), "my_function");
/// assert_eq!(pretty_name::of!(my_variable), "my_variable");
/// ```
#[macro_export]
macro_rules! of {
    ($ident:ident) => {{
        let _ = &$ident;
        stringify!($ident)
    }};
}

/// Get the name of the given struct field.
/// 
/// This macro resolves `Self` to the appropriate type when used inside an `impl` block.
/// 
/// This macro checks that the field exists on the given type. If either the type or field
/// is renamed via refactoring tools, the macro call will be updated accordingly.
/// 
/// # Examples
/// ```rust
/// struct MyStruct {
///     my_field: i32,
/// }
/// assert_eq!(pretty_name::of_field!(MyStruct::my_field), "MyStruct::my_field");
/// ```
#[macro_export]
macro_rules! of_field {
    ($ty:ident :: $field:ident) => {{
        let _ = |obj: $ty| {
            let _ = &obj.$field;
        };
        format!("{}::{}", $crate::type_name::<$ty>(), stringify!($field))
    }};
}

/// Get the name of the given method.
/// 
/// This macro resolves `Self` to the appropriate type when used inside an `impl` block.
/// 
/// This macro checks that the method exists on the given type. If either the type or method
/// is renamed via refactoring tools, the macro call will be updated accordingly.
/// 
/// # Examples
/// ```rust
/// struct MyStruct;
/// impl MyStruct {
///     fn my_method(&self) {}
/// }
/// assert_eq!(pretty_name::of_method!(MyStruct::my_method), "MyStruct::my_method");
/// ```
#[macro_export]
macro_rules! of_method {
    ($ty:ident :: $method:ident) => {{
        let _ = &$ty::$method;
        format!("{}::{}", $crate::type_name::<$ty>(), stringify!($method))
    }};
}

/// Get the name of the given enum variant.
/// 
/// This macro resolves `Self` to the appropriate type when used inside an `impl` block.
/// 
/// This macros supports both unit variants, tuple variants and struct variants. See
/// examples for syntax for each variant type.
/// 
/// This macro checks that the variant exists on the given enum type. If either the type or
/// variant is renamed via refactoring tools, the macro call will be updated accordingly.
/// 
/// # Examples
/// ```rust
/// enum MyEnum {
///     UnitVariant,
///     TupleVariant(i32, String),
///     StructVariant { field: i32 },
/// }
/// assert_eq!(pretty_name::of_variant!(MyEnum::UnitVariant), "MyEnum::UnitVariant");
/// assert_eq!(pretty_name::of_variant!(MyEnum::TupleVariant(..)), "MyEnum::TupleVariant");
/// assert_eq!(pretty_name::of_variant!(MyEnum::StructVariant {..}), "MyEnum::StructVariant");
/// ```
#[macro_export]
macro_rules! of_variant {
    ($ty:ident :: $variant:ident) => {{
        let _ = |obj| match obj { $ty::$variant => {}, _ => {} };
        format!("{}::{}", $crate::type_name::<$ty>(), stringify!($variant))
    }};
    ($ty:ident :: $variant:ident (..)) => {{
        let _ = |obj| match obj { $ty::$variant(..) => {}, _ => {} };
        format!("{}::{}", $crate::type_name::<$ty>(), stringify!($variant))
    }};
    ($ty:ident :: $variant:ident {..}) => {{
        let _ = |obj| match obj { $ty::$variant { .. } => {}, _ => {} };
        format!("{}::{}", $crate::type_name::<$ty>(), stringify!($variant))
    }};
}
