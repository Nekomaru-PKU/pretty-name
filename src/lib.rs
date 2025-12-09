#![doc = include_str!("../README.md")]

mod type_name;
pub use type_name::type_name;
pub use type_name::type_name_of_val;

/// Get the name of the given local variable or constant as a string literal.
/// 
/// This macro checks that the identifier is valid in the current scope. If the identifier
/// is renamed via refactoring tools, the macro call will be updated accordingly.
/// 
/// # Examples
/// ```rust
/// let my_variable = 42;
/// const MY_CONSTANT: u32 = 42;
/// assert_eq!(pretty_name::of_var!(my_variable), "my_variable");
/// assert_eq!(pretty_name::of_var!(MY_CONSTANT), "MY_CONSTANT");
/// ```
#[macro_export]
macro_rules! of_var {
    ($ident:ident) => {{
        let _ = &$ident;
        stringify!($ident)
    }};
}

/// Get the name of the given function.
/// 
/// Use `::<..>` syntax to exclude generic parameters in the output, see examples.
/// 
/// This macro produces a string literal if the function has no generic parameters,
/// or the generic parameters are explicitly excluded with `::<..>`. Otherwise, it
/// produces a [`String`].
/// 
/// This macro checks that the identifier is valid in the current scope. If the identifier
/// is renamed via refactoring tools, the macro call will be updated accordingly.
/// 
/// # Examples
/// ```rust
/// fn my_function() {}
/// fn my_generic_function<T>() {}
/// fn my_generic_function_2args<T, U>() {}
/// assert_eq!(pretty_name::of_function!(my_function), "my_function");
/// assert_eq!(pretty_name::of_function!(my_generic_function::<..>), "my_generic_function");
/// assert_eq!(pretty_name::of_function!(my_generic_function::<u32>), "my_generic_function::<u32>");
/// assert_eq!(pretty_name::of_function!(my_generic_function_2args::<..>), "my_generic_function_2args");
/// assert_eq!(pretty_name::of_function!(my_generic_function_2args::<u32, String>), "my_generic_function_2args::<u32, String>");
/// ```
#[macro_export]
macro_rules! of_function {
    // IMPLEMENTATION NOTE:
    //   - The $ident arm magically handles auto-completion for the other arms,
    //     especially for the $ident::<..> arm.
    //   - The $ident::<..> arm adopts an unusual approach for identifier validation
    //     by using `use $ident;`. This works because functions can be imported, but
    //     lacks auto-completion support in VSCode and other editors. This means that
    //     currently we cannot use this approach for the general case.
    ($ident:ident) => {{
        let _ = &$ident;
        stringify!($ident)
    }};
    ($ident:ident ::<..>) => {{
        #[allow(unused)] use $ident;
        stringify!($ident)
    }};
    ($ident:ident ::<$($arg:ty),*>) => {{
        let _ = &$ident::<$($arg),*>;
        format!(
            "{}::<{}>",
            stringify!($ident),
            vec![$($crate::type_name::<$arg>()),*].join(", "))
    }};
}

/// Get the name of the given struct field like `Type::field` as a [`String`].
/// 
/// This macro resolves `Self` to the appropriate type when used inside an `impl` block.
/// 
/// By default, this macro expects a simple type identifier like `Type::field`. To use
/// types with qualified path or generic parameters, wrap the type in angle brackets
/// like `<Type<T>>::field` or `<module::Type>::field`.
/// 
/// This macro checks that the field exists on the given type. If either the type or field
/// is renamed via refactoring tools, the macro call will be updated accordingly.
/// 
/// # Examples
/// ```rust
/// struct MyStruct {
///     my_field: u32,
/// }
/// struct MyGenericStruct<T> {
///     my_field: T,
/// }
/// assert_eq!(pretty_name::of_field!(MyStruct::my_field), "MyStruct::my_field");
/// assert_eq!(pretty_name::of_field!(<MyGenericStruct<u32>>::my_field), "<MyGenericStruct<u32>>::my_field");
/// ```
#[macro_export]
macro_rules! of_field {
    ($ty:ident :: $field:ident) => {{
        let _ = |obj: $ty| { let _ = &obj.$field; };
        format!("{}::{}", $crate::type_name::<$ty>(), stringify!($field))
    }};
    (<$ty:ty> :: $field:ident) => {{
        let _ = |obj: $ty| { let _ = &obj.$field; };
        format!("<{}>::{}", $crate::type_name::<$ty>(), stringify!($field))
    }};
}

/// Get the name of the given method as a [`String`].
/// 
/// This macro resolves `Self` to the appropriate type when used inside an `impl` block.
/// 
/// By default, this macro expects a simple type identifier like `Type::field`. To use
/// types with qualified path or generic parameters, wrap the type in angle brackets
/// like `<Type<T>>::field` or `<module::Type>::field`.
/// 
/// This macro checks that the method exists on the given type. If either the type or method
/// is renamed via refactoring tools, the macro call will be updated accordingly.
/// 
/// Due to implementation limitations, you cannot use `::<..>` syntax to exclude generic
/// parameters. Use explicit type arguments instead.
/// 
/// # Examples
/// ```rust
/// struct MyStruct;
/// impl MyStruct {
///     fn my_method(&self) {}
///     fn my_generic_method<T>(&self) {}
/// }
/// struct MyGenericStruct<T>(std::marker::PhantomData<T>);
/// impl<T> MyGenericStruct<T> {
///     fn my_method(&self) {}
///     fn my_generic_method<U>(&self) {}
/// }
/// assert_eq!(pretty_name::of_method!(MyStruct::my_method), "MyStruct::my_method");
/// assert_eq!(pretty_name::of_method!(MyStruct::my_generic_method::<u32>), "MyStruct::my_generic_method::<u32>");
/// assert_eq!(pretty_name::of_method!(<MyGenericStruct<u32>>::my_method), "<MyGenericStruct<u32>>::my_method");
/// assert_eq!(pretty_name::of_method!(<MyGenericStruct<u32>>::my_generic_method::<String>), "<MyGenericStruct<u32>>::my_generic_method::<String>");
/// ```
#[macro_export]
macro_rules! of_method {
    ($ty:ident :: $method:ident) => {{
        let _ = &$ty::$method;
        format!("{}::{}", $crate::type_name::<$ty>(), stringify!($method))
    }};
    ($ty:ident :: $method:ident ::<$($arg:ty),*>) => {{
        let _ = &$ty::$method::<$($arg),*>;
        format!(
            "{}::{}::<{}>",
            $crate::type_name::<$ty>(),
            stringify!($method),
            vec![$($crate::type_name::<$arg>()),*].join(", "))
    }};

    (<$ty:ty> :: $method:ident) => {{
        let _ = &<$ty>::$method;
        format!("<{}>::{}", $crate::type_name::<$ty>(), stringify!($method))
    }};
    (<$ty:ty> :: $method:ident ::<$($arg:ty),*>) => {{
        let _ = &<$ty>::$method::<$($arg),*>;
        format!(
            "<{}>::{}::<{}>",
            $crate::type_name::<$ty>(),
            stringify!($method),
            vec![$($crate::type_name::<$arg>()),*].join(", "))
    }};
}

/// Get the name of the given enum variant as a [`String`].
/// 
/// This macro resolves `Self` to the appropriate type when used inside an `impl` block.
/// 
/// This macros supports both unit variants, tuple variants and struct variants. See
/// examples for syntax for each variant type.
/// 
/// This macro checks that the variant exists on the given enum type. If either the type or
/// variant is renamed via refactoring tools, the macro call will be updated accordingly.
/// 
/// This macro currently expects only simple type identifiers like `Type::field`.
/// Support for more complex types requires the experimental feature `more_qualified_paths`
/// (issue #86935 <https://github.com/rust-lang/rust/issues/86935>) to be stabilized.
/// 
/// # Examples
/// ```rust
/// enum MyEnum {
///     UnitVariant,
///     TupleVariant(u32, String),
///     StructVariant { field: u32 },
/// }
/// assert_eq!(pretty_name::of_variant!(MyEnum::UnitVariant), "MyEnum::UnitVariant");
/// assert_eq!(pretty_name::of_variant!(MyEnum::TupleVariant(..)), "MyEnum::TupleVariant");
/// assert_eq!(pretty_name::of_variant!(MyEnum::StructVariant {..}), "MyEnum::StructVariant");
/// ```
#[macro_export]
macro_rules! of_variant {
    ($ty:ident $(::<$($ty_arg:ty),*>)? :: $variant:ident) => {{
        let _ = |obj| match obj { $ty $(::<$($ty_arg),*>)?::$variant => {}, _ => {} };
        format!("{}::{}", $crate::type_name::<$ty $(::<$($ty_arg),*>)?>(), stringify!($variant))
    }};
    ($ty:ident $(::<$($ty_arg:ty),*>)? :: $variant:ident (..)) => {{
        let _ = |obj| match obj { $ty $(::<$($ty_arg),*>)?::$variant(..) => {}, _ => {} };
        format!("{}::{}", $crate::type_name::<$ty $(::<$($ty_arg),*>)?>(), stringify!($variant))
    }};
    ($ty:ident $(::<$($ty_arg:ty),*>)? :: $variant:ident {..}) => {{
        let _ = |obj| match obj { $ty $(::<$($ty_arg),*>)?::$variant { .. } => {}, _ => {} };
        format!("{}::{}", $crate::type_name::<$ty $(::<$($ty_arg),*>)?>(), stringify!($variant))
    }};

    (<$ty:ty> :: $variant:ident) => {{
        let _ = |obj| match obj { <$ty>::$variant => {}, _ => {} };
        format!("<{}>::{}", $crate::type_name::<$ty>(), stringify!($variant))
    }};
    (<$ty:ty> :: $variant:ident (..)) => {{
        let _ = |obj| match obj { <$ty>::$variant(..) => {}, _ => {} };
        format!("<{}>::{}", $crate::type_name::<$ty>(), stringify!($variant))
    }};
    (<$ty:ty> :: $variant:ident {..}) => {{
        let _ = |obj| match obj { <$ty>::$variant { .. } => {}, _ => {} };
        format!("<{}>::{}", $crate::type_name::<$ty>(), stringify!($variant))
    }};
}
