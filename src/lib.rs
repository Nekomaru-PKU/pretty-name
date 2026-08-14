#![doc = include_str!("../README.md")]

mod name;
mod type_name;
pub use name::FunctionName;
pub use name::IdentifierName;
pub use name::MemberName;
pub use type_name::TypeName;
pub use type_name::type_name;
pub use type_name::type_name_of_val;

#[doc(hidden)]
pub use name::function_name as __function_name;
#[doc(hidden)]
pub use name::identifier_name as __identifier_name;
#[doc(hidden)]
pub use name::member_name as __member_name;

/// Gets the validated source identifier of a local variable or constant.
///
/// This macro checks that the identifier is valid in the current scope. If the identifier
/// is renamed via refactoring tools, the macro call will be updated accordingly.
///
/// # Examples
/// ```rust
/// let my_variable = 42;
/// const MY_CONSTANT: u32 = 42;
/// assert_eq!(pretty_name::of_var!(my_variable).to_string(), "my_variable");
/// assert_eq!(pretty_name::of_var!(MY_CONSTANT).to_string(), "MY_CONSTANT");
/// ```
///
/// Missing identifiers are rejected at compile time:
///
/// ```compile_fail
/// let _ = pretty_name::of_var!(missing_variable);
/// ```
#[macro_export]
macro_rules! of_var {
    ($ident:ident) => {{
        let _ = &$ident;
        $crate::__identifier_name(stringify!($ident))
    }};
}

/// Gets a validated function name with compiler-resolved generic type arguments.
///
/// Non-generic functions use the identifier-only form. Generic functions must specify
/// every caller-provided generic argument, and every argument must be a concrete type.
/// Inferred arguments, direct const arguments, and the legacy `::<..>` placeholder are
/// intentionally unsupported.
///
/// # Examples
/// ```rust
/// fn my_function() {}
/// fn my_generic_function<T>() {}
/// fn my_generic_function_2args<T, U>() {}
/// assert_eq!(pretty_name::of_function!(my_function).to_string(), "my_function");
/// assert_eq!(
///     pretty_name::of_function!(my_generic_function::<u32>).to_string(),
///     "my_generic_function::<u32>");
/// assert_eq!(
///     pretty_name::of_function!(my_generic_function_2args::<u32, String>).to_string(),
///     "my_generic_function_2args::<u32, String>");
/// ```
///
/// A generic function cannot omit its type arguments:
///
/// ```compile_fail
/// fn generic<T>() {}
/// let _ = pretty_name::of_function!(generic);
/// ```
///
/// The legacy placeholder is rejected:
///
/// ```compile_fail
/// fn generic<T>() {}
/// let _ = pretty_name::of_function!(generic::<..>);
/// ```
///
/// Inferred type arguments are rejected:
///
/// ```compile_fail
/// fn generic<T>() {}
/// let _ = pretty_name::of_function!(generic::<_>);
/// ```
///
/// Direct const generic arguments are rejected:
///
/// ```compile_fail
/// fn const_generic<const N: usize>() {}
/// let _ = pretty_name::of_function!(const_generic::<16>);
/// ```
///
/// Missing functions are rejected:
///
/// ```compile_fail
/// let _ = pretty_name::of_function!(missing_function);
/// ```
#[macro_export]
macro_rules! of_function {
    ($ident:ident) => {{
        let _ = &$ident;
        $crate::__function_name(
            stringify!($ident),
            ::std::boxed::Box::new([]))
    }};
    ($ident:ident ::<$($arg:ty),+ $(,)?>) => {{
        let _ = &$ident::<$($arg),*>;
        $crate::__function_name(
            stringify!($ident),
            ::std::boxed::Box::new([$($crate::type_name::<$arg>()),*]))
    }};
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
/// assert_eq!(pretty_name::of_type!(MyStruct).to_string(), "MyStruct");
/// assert_eq!(
///     pretty_name::of_type!(MyGenericStruct<u32>).to_string(),
///     "MyGenericStruct<u32>");
/// ```
///
/// Invalid types are rejected at compile time:
///
/// ```compile_fail
/// let _ = pretty_name::of_type!(DefinitelyNotAType);
/// ```
#[macro_export]
macro_rules! of_type {
    ($ty:ty) => {{
        $crate::type_name::<$ty>()
    }};
}

/// Gets a validated field name with its compiler-resolved owner.
///
/// This macro resolves `Self` to the appropriate type when used inside an `impl` block.
///
/// By default, this macro expects a simple type identifier like `Type::field`. To use
/// types with qualified path or generic parameters, wrap the type in angle brackets
/// like `<Type<T>>::field` or `<module::Type>::field`.
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
///     pretty_name::of_field!(MyStruct::my_field).to_string(),
///     "<MyStruct>::my_field");
/// assert_eq!(
///     pretty_name::of_field!(<MyGenericStruct<u32>>::my_field).to_string(),
///     "<MyGenericStruct<u32>>::my_field");
/// ```
///
/// Missing fields are rejected at compile time:
///
/// ```compile_fail
/// struct MyStruct;
/// let _ = pretty_name::of_field!(MyStruct::missing_field);
/// ```
#[macro_export]
macro_rules! of_field {
    (Self:: $field:ident) => {{
        let _ = |obj: Self| { let _ = &obj.$field; };
        $crate::__member_name(
            $crate::type_name::<Self>(),
            stringify!($field),
            ::std::boxed::Box::new([]))
    }};
    ($ty:ident :: $field:ident) => {{
        let _ = |obj: $ty| { let _ = &obj.$field; };
        $crate::__member_name(
            $crate::type_name::<$ty>(),
            stringify!($field),
            ::std::boxed::Box::new([]))
    }};
    (<$ty:ty> :: $field:ident) => {{
        let _ = |obj: $ty| { let _ = &obj.$field; };
        $crate::__member_name(
            $crate::type_name::<$ty>(),
            stringify!($field),
            ::std::boxed::Box::new([]))
    }};
}

/// Gets a validated method name with its compiler-resolved owner and type arguments.
///
/// This macro resolves `Self` to the appropriate type when used inside an `impl` block.
///
/// By default, this macro expects a simple type identifier like `Type::method`. To use
/// types with qualified path or generic parameters, wrap the type in angle brackets
/// like `<Type<T>>::method` or `<module::Type>::method`.
///
/// A generic method must specify every caller-provided generic argument, and every
/// argument must be a concrete type. Inferred arguments, direct const arguments, and
/// the legacy `::<..>` placeholder are intentionally unsupported.
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
/// assert_eq!(
///     pretty_name::of_method!(MyStruct::my_method).to_string(),
///     "<MyStruct>::my_method");
/// assert_eq!(
///     pretty_name::of_method!(MyStruct::my_generic_method::<u32>).to_string(),
///     "<MyStruct>::my_generic_method::<u32>");
/// assert_eq!(
///     pretty_name::of_method!(<MyGenericStruct<u32>>::my_method).to_string(),
///     "<MyGenericStruct<u32>>::my_method");
/// assert_eq!(
///     pretty_name::of_method!(
///         <MyGenericStruct<u32>>::my_generic_method::<String>).to_string(),
///     "<MyGenericStruct<u32>>::my_generic_method::<String>");
/// ```
///
/// A generic method cannot omit its type arguments:
///
/// ```compile_fail
/// struct Owner;
/// impl Owner { fn generic<T>(&self) {} }
/// let _ = pretty_name::of_method!(Owner::generic);
/// ```
///
/// The legacy placeholder is rejected:
///
/// ```compile_fail
/// struct Owner;
/// impl Owner { fn generic<T>(&self) {} }
/// let _ = pretty_name::of_method!(Owner::generic::<..>);
/// ```
///
/// Inferred type arguments are rejected:
///
/// ```compile_fail
/// struct Owner;
/// impl Owner { fn generic<T>(&self) {} }
/// let _ = pretty_name::of_method!(Owner::generic::<_>);
/// ```
///
/// Direct const generic arguments are rejected:
///
/// ```compile_fail
/// struct Owner;
/// impl Owner { fn generic<const N: usize>(&self) {} }
/// let _ = pretty_name::of_method!(Owner::generic::<16>);
/// ```
///
/// Missing methods are rejected:
///
/// ```compile_fail
/// struct Owner;
/// let _ = pretty_name::of_method!(Owner::missing_method);
/// ```
#[macro_export]
macro_rules! of_method {
    (Self:: $method:ident) => {{
        let _ = &Self::$method;
        $crate::__member_name(
            $crate::type_name::<Self>(),
            stringify!($method),
            ::std::boxed::Box::new([]))
    }};
    ($ty:ident :: $method:ident) => {{
        let _ = &$ty::$method;
        $crate::__member_name(
            $crate::type_name::<$ty>(),
            stringify!($method),
            ::std::boxed::Box::new([]))
    }};
    ($ty:ident :: $method:ident ::<$($arg:ty),+ $(,)?>) => {{
        let _ = &$ty::$method::<$($arg),*>;
        $crate::__member_name(
            $crate::type_name::<$ty>(),
            stringify!($method),
            ::std::boxed::Box::new([$($crate::type_name::<$arg>()),*]))
    }};

    (<$ty:ty> :: $method:ident) => {{
        let _ = &<$ty>::$method;
        $crate::__member_name(
            $crate::type_name::<$ty>(),
            stringify!($method),
            ::std::boxed::Box::new([]))
    }};
    (<$ty:ty> :: $method:ident ::<$($arg:ty),+ $(,)?>) => {{
        let _ = &<$ty>::$method::<$($arg),*>;
        $crate::__member_name(
            $crate::type_name::<$ty>(),
            stringify!($method),
            ::std::boxed::Box::new([$($crate::type_name::<$arg>()),*]))
    }};
}

/// Gets a validated enum variant name with its compiler-resolved owner.
///
/// This macro resolves `Self` to the appropriate type when used inside an `impl` block.
///
/// This macro supports unit, tuple, and struct variants. See the examples for each
/// variant shape's syntax.
///
/// To use a qualified or generic owner type, wrap the type in angle brackets like
/// `<module::MyEnum>::Variant` or `<MyEnum<T>>::Variant`. These forms work on stable Rust.
///
/// # Examples
/// ```rust
/// enum MyEnum {
///     UnitVariant,
///     TupleVariant(u32, String),
///     StructVariant { field: u32 },
/// }
/// enum MyGenericEnum<T> {
///     UnitVariant,
///     Value(T),
/// }
/// assert_eq!(
///     pretty_name::of_variant!(MyEnum::UnitVariant).to_string(),
///     "<MyEnum>::UnitVariant");
/// assert_eq!(
///     pretty_name::of_variant!(MyEnum::TupleVariant(..)).to_string(),
///     "<MyEnum>::TupleVariant");
/// assert_eq!(
///     pretty_name::of_variant!(MyEnum::StructVariant {..}).to_string(),
///     "<MyEnum>::StructVariant");
/// assert_eq!(
///     pretty_name::of_variant!(<MyGenericEnum<u32>>::UnitVariant).to_string(),
///     "<MyGenericEnum<u32>>::UnitVariant");
/// ```
///
/// The requested variant shape is checked at compile time:
///
/// ```compile_fail
/// enum MyEnum { Tuple(u32) }
/// let _ = pretty_name::of_variant!(MyEnum::Tuple);
/// ```
///
/// Missing variants are rejected:
///
/// ```compile_fail
/// enum MyEnum { Unit }
/// let _ = pretty_name::of_variant!(MyEnum::Missing);
/// ```
#[macro_export]
macro_rules! of_variant {
    (Self:: $variant:ident) => {{
        let _ = |obj: Self| match obj { Self::$variant => {}, _ => {} };
        $crate::__member_name(
            $crate::type_name::<Self>(),
            stringify!($variant),
            ::std::boxed::Box::new([]))
    }};
    (Self:: $variant:ident (..)) => {{
        let _ = |obj: Self| match obj { Self::$variant(..) => {}, _ => {} };
        $crate::__member_name(
            $crate::type_name::<Self>(),
            stringify!($variant),
            ::std::boxed::Box::new([]))
    }};
    (Self:: $variant:ident {..}) => {{
        let _ = |obj: Self| match obj { Self::$variant { .. } => {}, _ => {} };
        $crate::__member_name(
            $crate::type_name::<Self>(),
            stringify!($variant),
            ::std::boxed::Box::new([]))
    }};

    ($ty:ident :: $variant:ident) => {{
        let _ = |obj: $ty| match obj { $ty::$variant => {}, _ => {} };
        $crate::__member_name(
            $crate::type_name::<$ty>(),
            stringify!($variant),
            ::std::boxed::Box::new([]))
    }};
    ($ty:ident :: $variant:ident (..)) => {{
        let _ = |obj: $ty| match obj { $ty::$variant(..) => {}, _ => {} };
        $crate::__member_name(
            $crate::type_name::<$ty>(),
            stringify!($variant),
            ::std::boxed::Box::new([]))
    }};
    ($ty:ident :: $variant:ident {..}) => {{
        let _ = |obj: $ty| match obj { $ty::$variant { .. } => {}, _ => {} };
        $crate::__member_name(
            $crate::type_name::<$ty>(),
            stringify!($variant),
            ::std::boxed::Box::new([]))
    }};

    (<$ty:ty> :: $variant:ident) => {{
        let _ = |obj: $ty| match obj { <$ty>::$variant => {}, _ => {} };
        $crate::__member_name(
            $crate::type_name::<$ty>(),
            stringify!($variant),
            ::std::boxed::Box::new([]))
    }};
    (<$ty:ty> :: $variant:ident (..)) => {{
        let _ = |obj: $ty| match obj { <$ty>::$variant(..) => {}, _ => {} };
        $crate::__member_name(
            $crate::type_name::<$ty>(),
            stringify!($variant),
            ::std::boxed::Box::new([]))
    }};
    (<$ty:ty> :: $variant:ident {..}) => {{
        let _ = |obj: $ty| match obj { <$ty>::$variant { .. } => {}, _ => {} };
        $crate::__member_name(
            $crate::type_name::<$ty>(),
            stringify!($variant),
            ::std::boxed::Box::new([]))
    }};
}
