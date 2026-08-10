#![doc = include_str!("../README.md")]

mod type_name;
pub use type_name::type_name;
pub use type_name::type_name_of_val;

/// Internal helper macro for caching dynamically generated names process-wide.
///
/// Each invocation owns a cache keyed by the supplied compiler type names. Keying the
/// cache is required because local static items are shared by every monomorphization of
/// a generic function containing the invocation. One result is intentionally leaked per
/// distinct key so callers can keep receiving `&'static str`.
#[doc(hidden)]
#[macro_export]
macro_rules! __with_cache {
    ([$($key:expr),+ $(,)?] => $expr:expr) => {{
        use std::collections::HashMap;
        use std::sync::{LazyLock, RwLock};

        static CACHE: LazyLock<RwLock<HashMap<Vec<&'static str>, &'static str>>> =
            LazyLock::new(|| RwLock::new(HashMap::new()));

        let cache_key = [$($key),+];
        let cached = CACHE
            .read()
            // CONTEXT: A panic cannot invalidate previously inserted immutable strings.
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(cache_key.as_slice())
            .copied();
        if let Some(cached) = cached {
            cached
        } else {
            let result = $expr;
            let mut cache = CACHE
                .write()
                // CONTEXT: A panic cannot invalidate previously inserted immutable strings.
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(cached) = cache.get(cache_key.as_slice()).copied() {
                cached
            } else {
                let result: &'static str = Box::leak(result.into_boxed_str());
                cache.insert(cache_key.to_vec(), result);
                result
            }
        }
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
            [stringify!($ident), $(::std::any::type_name::<$arg>()),*] =>
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
///
/// Simple identifiers preserve source spelling, while generic, qualified, and `Self`
/// types use [`type_name`](crate::type_name). This distinction is observable for type
/// aliases. Call [`type_name`](crate::type_name) directly when semantic resolution is
/// required consistently.
///
/// Invalid simple identifiers are rejected at compile time:
///
/// ```compile_fail
/// let _ = pretty_name::of_type!(DefinitelyNotAType);
/// ```
#[macro_export]
macro_rules! of_type {
    (Self) => {{
        $crate::type_name::<Self>()
    }};
    ($ty:ident) => {{
        let _: ::core::marker::PhantomData<$ty> = ::core::marker::PhantomData;
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
        let _ = |obj: Self| { let _ = &obj.$field; };
        $crate::__with_cache!(
            [::std::any::type_name::<Self>(), stringify!($field)] =>
            format!("{}::{}", $crate::type_name::<Self>(), stringify!($field)))
    }};
    ($ty:ident :: $field:ident) => {{
        let _ = |obj: $ty| { let _ = &obj.$field; };
        concat!(stringify!($ty), "::", stringify!($field))
    }};
    (<$ty:ty> :: $field:ident) => {{
        let _ = |obj: $ty| { let _ = &obj.$field; };
        $crate::__with_cache!(
            [::std::any::type_name::<$ty>(), stringify!($field)] =>
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
        let _ = &Self::$method;
        $crate::__with_cache!(
            [::std::any::type_name::<Self>(), stringify!($method)] =>
            format!("{}::{}", $crate::type_name::<Self>(), stringify!($method)))
    }};
    ($ty:ident :: $method:ident) => {{
        let _ = &$ty::$method;
        concat!(stringify!($ty), "::", stringify!($method))
    }};
    ($ty:ident :: $method:ident ::<$($arg:ty),*>) => {{
        let _ = &$ty::$method::<$($arg),*>;
        $crate::__with_cache!(
            [
                ::std::any::type_name::<$ty>(),
                stringify!($method),
                $(::std::any::type_name::<$arg>()),*
            ] =>
            format!(
                "{}::{}::<{}>",
                $crate::type_name::<$ty>(),
                stringify!($method),
                vec![$($crate::type_name::<$arg>()),*].join(", ")))
    }};

    (<$ty:ty> :: $method:ident) => {{
        let _ = &<$ty>::$method;
        $crate::__with_cache!(
            [::std::any::type_name::<$ty>(), stringify!($method)] =>
            format!("<{}>::{}", $crate::type_name::<$ty>(), stringify!($method)))
    }};
    (<$ty:ty> :: $method:ident ::<$($arg:ty),*>) => {{
        let _ = &<$ty>::$method::<$($arg),*>;
        $crate::__with_cache!(
            [
                ::std::any::type_name::<$ty>(),
                stringify!($method),
                $(::std::any::type_name::<$arg>()),*
            ] =>
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
        let _ = |obj: Self| match obj { Self::$variant => {}, _ => {} };
        $crate::__with_cache!(
            [::std::any::type_name::<Self>(), stringify!($variant)] =>
            format!("{}::{}", $crate::type_name::<Self>(), stringify!($variant)))
    }};
    (Self:: $variant:ident (..)) => {{
        let _ = |obj: Self| match obj { Self::$variant(..) => {}, _ => {} };
        $crate::__with_cache!(
            [::std::any::type_name::<Self>(), stringify!($variant)] =>
            format!("{}::{}", $crate::type_name::<Self>(), stringify!($variant)))
    }};
    (Self:: $variant:ident {..}) => {{
        let _ = |obj: Self| match obj { Self::$variant { .. } => {}, _ => {} };
        $crate::__with_cache!(
            [::std::any::type_name::<Self>(), stringify!($variant)] =>
            format!("{}::{}", $crate::type_name::<Self>(), stringify!($variant)))
    }};

    ($ty:ident :: $variant:ident) => {{
        let _ = |obj: $ty| match obj { $ty::$variant => {}, _ => {} };
        concat!(stringify!($ty), "::", stringify!($variant))
    }};
    ($ty:ident :: $variant:ident (..)) => {{
        let _ = |obj: $ty| match obj { $ty::$variant(..) => {}, _ => {} };
        concat!(stringify!($ty), "::", stringify!($variant))
    }};
    ($ty:ident :: $variant:ident {..}) => {{
        let _ = |obj: $ty| match obj { $ty::$variant { .. } => {}, _ => {} };
        concat!(stringify!($ty), "::", stringify!($variant))
    }};

    (<$ty:ty> :: $variant:ident) => {{
        let _ = |obj: $ty| match obj { <$ty>::$variant => {}, _ => {} };
        $crate::__with_cache!(
            [::std::any::type_name::<$ty>(), stringify!($variant)] =>
            format!("<{}>::{}", $crate::type_name::<$ty>(), stringify!($variant)))
    }};
    (<$ty:ty> :: $variant:ident (..)) => {{
        let _ = |obj: $ty| match obj { <$ty>::$variant(..) => {}, _ => {} };
        $crate::__with_cache!(
            [::std::any::type_name::<$ty>(), stringify!($variant)] =>
            format!("<{}>::{}", $crate::type_name::<$ty>(), stringify!($variant)))
    }};
    (<$ty:ty> :: $variant:ident {..}) => {{
        let _ = |obj: $ty| match obj { <$ty>::$variant { .. } => {}, _ => {} };
        $crate::__with_cache!(
            [::std::any::type_name::<$ty>(), stringify!($variant)] =>
            format!("<{}>::{}", $crate::type_name::<$ty>(), stringify!($variant)))
    }};
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    /// Generic function used to verify that cache entries follow monomorphizations.
    fn generic_function<T>() {}

    /// Gets the name of [`generic_function`] for the caller's concrete type.
    fn generic_function_name<T>() -> &'static str {
        crate::of_function!(generic_function::<T>)
    }

    /// Generic owner used to verify that `Self` cache entries follow monomorphizations.
    struct GenericStruct<T>(PhantomData<T>);

    impl<T> GenericStruct<T> {
        /// Method referenced by the name macro.
        fn method(&self) {}

        /// Gets the method name for the concrete `Self` type.
        fn method_name() -> &'static str {
            crate::of_method!(Self::method)
        }
    }

    /// Gets the source spelling of a possibly unsized generic type parameter.
    fn generic_type_source_name<T: ?Sized>() -> &'static str {
        crate::of_type!(T)
    }

    /// Verifies the existing `Self` behavior for types, fields, and generic methods.
    #[test]
    fn self_macros_resolve_the_concrete_owner() {
        struct MyStruct {
            my_field: u32,
        }
        impl MyStruct {
            /// Method referenced by the name macro.
            fn my_method<T>(&self) {}

            /// Verifies every supported `Self`-based name within its valid scope.
            fn verify_names(&self) {
                assert_eq!(crate::of_type!(Self), "MyStruct");
                assert_eq!(crate::of_field!(Self::my_field), "MyStruct::my_field");
                assert_eq!(crate::of_method!(Self::my_method::<u32>), "MyStruct::my_method::<u32>");
            }
        }

        let my_struct = MyStruct { my_field: 42 };
        my_struct.verify_names();
    }

    /// Verifies one call site caches each concrete generic function name separately.
    #[test]
    fn function_cache_distinguishes_generic_monomorphizations() {
        assert_eq!(
            (generic_function_name::<u8>(), generic_function_name::<u16>()),
            ("generic_function::<u8>", "generic_function::<u16>"));
    }

    /// Verifies a macro invocation shares its cached result between threads.
    #[test]
    fn function_cache_reuses_one_result_across_threads() {
        let first = std::thread::spawn(generic_function_name::<u32>)
            .join()
            .unwrap();
        let second = std::thread::spawn(generic_function_name::<u32>)
            .join()
            .unwrap();

        assert!(std::ptr::eq(first, second));
    }

    /// Verifies a generic `Self` method does not reuse another owner's cached name.
    #[test]
    fn self_cache_distinguishes_generic_monomorphizations() {
        assert_eq!(
            (GenericStruct::<u8>::method_name(), GenericStruct::<u16>::method_name()),
            ("GenericStruct<u8>::method", "GenericStruct<u16>::method"));
    }

    /// Verifies validation does not accidentally impose an implicit `Sized` bound.
    #[test]
    fn simple_type_validation_accepts_unsized_generic_parameters() {
        assert_eq!(generic_type_source_name::<str>(), "T");
    }
}
