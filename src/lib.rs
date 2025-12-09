#![doc = include_str!("../README.md")]

mod type_name;
pub use type_name::type_name;
pub use type_name::type_name_of_val;

/// Internal helper macro for caching string results in thread-local storage.
///
/// This macro wraps an expression that produces a `String` and caches it as a
/// `&'static str` using thread-local `LazyCell`. Each unique macro invocation
/// gets its own cache entry, ensuring zero runtime overhead after first use.
#[doc(hidden)]
#[macro_export]
macro_rules! __with_cache {
    ($expr:expr) => {{
        use std::cell::LazyCell;
        thread_local! {
            static CACHE: LazyCell<&'static str> =
                LazyCell::new(|| {
                    let result = $expr;
                    Box::leak(result.into_boxed_str())
                });
        }
        CACHE.with(|cell| *LazyCell::force(cell))
    }};
}

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

/// Get the name of the given function as a `&'static str`.
///
/// Use a `::<..>` placeholder to exclude generic parameters in the output, see examples.
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
        $crate::__with_cache!(
            format!(
                "{}::<{}>",
                stringify!($ident),
                vec![$($crate::type_name::<$arg>()),*].join(", ")))
    }};
}

/// Get the name of the given type as a `&'static str`.
/// 
/// This macro resolves `Self` to the appropriate type when used inside an `impl` block.
/// 
/// If the given type is a single identifier and is not `Self`, the macro expands to a
/// string literal at compile time. For more complex types, the macro uses runtime type
/// name retrieval with caching.
/// 
/// # Examples
/// ```rust
/// struct MyStruct;
/// struct MyGenericStruct<T>(std::marker::PhantomData<T>);
/// assert_eq!(pretty_name::of_type!(MyStruct), "MyStruct");
/// assert_eq!(pretty_name::of_type!(MyGenericStruct<u32>), "MyGenericStruct<u32>");
/// ```
#[macro_export]
macro_rules! of_type {
    (Self) => {{
        $crate::type_name::<Self>()
    }};
    ($ty:ident) => {{
        stringify!($ty)
    }};
    ($ty:ty) => {{
        $crate::type_name::<$ty>()
    }};
}

/// Get the name of the given struct field like `Type::field` as a `&'static str`.
///
/// This macro resolves `Self` to the appropriate type when used inside an `impl` block.
///
/// By default, this macro expects a simple type identifier like `Type::field`. To use
/// types with qualified path or generic parameters, wrap the type in angle brackets
/// like `<Type<T>>::field` or `<module::Type>::field`.
/// 
/// If the *Type* part is a single identifier and is not `Self`, the macro expands to a
/// string literal at compile time. For more complex types, the macro uses runtime type
/// name retrieval with caching.
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
    (Self:: $field:ident) => {{
        let _ = |obj: $ty| { let _ = &obj.$field; };
        $crate::__with_cache!(
            format!("{}::{}", $crate::type_name::<Self>(), stringify!($field)))
    }};
    ($ty:ident :: $field:ident) => {{
        let _ = |obj: $ty| { let _ = &obj.$field; };
        concat!(stringify!($ty), "::", stringify!($field))
    }};
    (<$ty:ty> :: $field:ident) => {{
        let _ = |obj: $ty| { let _ = &obj.$field; };
        $crate::__with_cache!(
            format!("<{}>::{}", $crate::type_name::<$ty>(), stringify!($field)))
    }};
}

/// Get the name of the given method like `Type::method` as a `&'static str`.
///
/// This macro resolves `Self` to the appropriate type when used inside an `impl` block.
///
/// By default, this macro expects a simple type identifier like `Type::field`. To use
/// types with qualified path or generic parameters, wrap the type in angle brackets
/// like `<Type<T>>::field` or `<module::Type>::field`.
/// 
/// If both the *Type* and *method* parts are single identifiers and the *Type* part is
/// not `Self`, the macro expands to a string literal at compile time. For more complex
/// types, the macro uses runtime type name retrieval with caching.
///
/// Due to implementation limitations, you cannot use the `::<..>` placeholder to exclude
/// generic parameters. Use explicit type arguments instead.
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
    (Self:: $method:ident) => {{
        let _ = &$ty::$method;
        $crate::__with_cache!(
            format!("{}::{}", $crate::type_name::<Self>(), stringify!($method)))
    }};
    ($ty:ident :: $method:ident) => {{
        let _ = &$ty::$method;
        concat!(stringify!($ty), "::", stringify!($method))
    }};
    ($ty:ident :: $method:ident ::<$($arg:ty),*>) => {{
        let _ = &$ty::$method::<$($arg),*>;
        $crate::__with_cache!(
            format!(
                "{}::{}::<{}>",
                $crate::type_name::<$ty>(),
                stringify!($method),
                vec![$($crate::type_name::<$arg>()),*].join(", ")))
    }};

    (<$ty:ty> :: $method:ident) => {{
        let _ = &<$ty>::$method;
        $crate::__with_cache!(
            format!("<{}>::{}", $crate::type_name::<$ty>(), stringify!($method)))
    }};
    (<$ty:ty> :: $method:ident ::<$($arg:ty),*>) => {{
        let _ = &<$ty>::$method::<$($arg),*>;
        $crate::__with_cache!(
            format!(
                "<{}>::{}::<{}>",
                $crate::type_name::<$ty>(),
                stringify!($method),
                vec![$($crate::type_name::<$arg>()),*].join(", ")))
    }};
}

/// Get the name of the given enum variant as a `&'static str`.
///
/// This macro resolves `Self` to the appropriate type when used inside an `impl` block.
///
/// This macros supports both unit variants, tuple variants and struct variants. See
/// examples for syntax for each variant type.
/// 
/// If the *Type* part is a single identifier and is not `Self`, the macro expands to a
/// string literal at compile time. For more complex types, the macro uses runtime type
/// name retrieval with caching.
///
/// This macro currently expects only simple type identifiers.
/// Support for more complex types requires the experimental feature `more_qualified_paths`
/// (issue #86935 <https://github.com/rust-lang/rust/issues/86935>) to be stabilized (or
/// enabled via `#![feature(more_qualified_paths)]` if using a nightly compiler).
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
    (Self:: $variant:ident) => {{
        let _ = |obj| match obj { Self::$variant => {}, _ => {} };
        $crate::__with_cache!(
            format!("{}::{}", $crate::type_name::<Self>(), stringify!($variant)))
    }};
    (Self:: $variant:ident (..)) => {{
        let _ = |obj| match obj { Self::$variant(..) => {}, _ => {} };
        $crate::__with_cache!(
            format!("{}::{}", $crate::type_name::<Self>(), stringify!($variant)))
    }};
    (Self:: $variant:ident {..}) => {{
        let _ = |obj| match obj { Self::$variant { .. } => {}, _ => {} };
        $crate::__with_cache!(
            format!("{}::{}", $crate::type_name::<Self>(), stringify!($variant)))
    }};

    ($ty:ident :: $variant:ident) => {{
        let _ = |obj| match obj { $ty::$variant => {}, _ => {} };
        concat!(stringify!($ty), "::", stringify!($variant))
    }};
    ($ty:ident :: $variant:ident (..)) => {{
        let _ = |obj| match obj { $ty::$variant(..) => {}, _ => {} };
        concat!(stringify!($ty), "::", stringify!($variant))
    }};
    ($ty:ident :: $variant:ident {..}) => {{
        let _ = |obj| match obj { $ty::$variant { .. } => {}, _ => {} };
        concat!(stringify!($ty), "::", stringify!($variant))
    }};

    (<$ty:ty> :: $variant:ident) => {{
        let _ = |obj| match obj { <$ty>::$variant => {}, _ => {} };
        $crate::__with_cache!(
            format!("<{}>::{}", $crate::type_name::<$ty>(), stringify!($variant)))
    }};
    (<$ty:ty> :: $variant:ident (..)) => {{
        let _ = |obj| match obj { <$ty>::$variant(..) => {}, _ => {} };
        $crate::__with_cache!(
            format!("<{}>::{}", $crate::type_name::<$ty>(), stringify!($variant)))
    }};
    (<$ty:ty> :: $variant:ident {..}) => {{
        let _ = |obj| match obj { <$ty>::$variant { .. } => {}, _ => {} };
        $crate::__with_cache!(
            format!("<{}>::{}", $crate::type_name::<$ty>(), stringify!($variant)))
    }};
}
