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

/// Splits a named owner path from its final field, method, or variant segment.
///
/// `macro_rules!` cannot bind an owner as `path` or `ty` immediately before `::`, so
/// this implementation-only macro recognizes the path separators and balanced owner
/// turbofish without interpreting the captured owner. The expanded Rust expressions
/// remain responsible for parsing the owner as a type and validating the referenced
/// member.
#[doc(hidden)]
#[macro_export]
macro_rules! __split_member_owner {
    // Terminal field form. Requiring the complete remainder prevents a module or type
    // path segment from being mistaken for the field identifier.
    (@path field [$($owner:tt)+] :: $field:ident) => {{
        let _ = |obj: $($owner)+| { let _ = &obj.$field; };
        $crate::__member_name(
            $crate::type_name::<$($owner)+>(),
            stringify!($field),
            ::std::boxed::Box::new([]))
    }};

    // Terminal method forms retain the existing complete-item validation contract.
    (@path method [$($owner:tt)+] :: $method:ident) => {{
        let _ = &$($owner)+::$method;
        $crate::__member_name(
            $crate::type_name::<$($owner)+>(),
            stringify!($method),
            ::std::boxed::Box::new([]))
    }};

    // Terminal variant forms validate both the selected variant and its requested
    // shape through ordinary Rust patterns.
    (@path variant [$($owner:tt)+] :: $variant:ident) => {{
        let _ = |obj: $($owner)+| match obj {
            $($owner)+::$variant => {},
            _ => {},
        };
        $crate::__member_name(
            $crate::type_name::<$($owner)+>(),
            stringify!($variant),
            ::std::boxed::Box::new([]))
    }};
    (@path variant [$($owner:tt)+] :: $variant:ident (..)) => {{
        let _ = |obj: $($owner)+| match obj {
            $($owner)+::$variant(..) => {},
            _ => {},
        };
        $crate::__member_name(
            $crate::type_name::<$($owner)+>(),
            stringify!($variant),
            ::std::boxed::Box::new([]))
    }};
    (@path variant [$($owner:tt)+] :: $variant:ident { $field:ident, .. }) => {{
        let _ = |obj: $($owner)+| match obj {
            $($owner)+::$variant { $field: _, .. } => {},
            _ => {},
        };
        $crate::__member_name(
            $crate::type_name::<$($owner)+>(),
            stringify!($variant),
            ::std::boxed::Box::new([]))
    }};

    // A turbofish immediately after the accumulated path belongs to its final owner
    // segment, while one after a new segment may instead belong to a generic method.
    (@path $kind:ident [$($owner:tt)+] :: < $($input:tt)*) => {
        $crate::__split_member_owner!(
            @generic $kind owner [$($owner)+] [] [] [] $($input)*)
    };
    (@path $kind:ident [$($owner:tt)+] :: $segment:ident :: < $($input:tt)*) => {
        $crate::__split_member_owner!(
            @generic $kind segment [$($owner)+] [$segment] [] [] $($input)*)
    };

    // A non-terminal identifier is another named segment of the owner path.
    (@path $kind:ident [$($owner:tt)+] :: $segment:ident $($rest:tt)+) => {
        $crate::__split_member_owner!(
            @path $kind [$($owner)+ :: $segment] $($rest)+)
    };

    // Rust lexes adjacent closing generic brackets as `>>`. These two arms consume
    // either one nested level plus the outer turbofish, or two nested levels.
    (@generic $kind:ident $mode:ident
        [$($owner:tt)+] [$($segment:tt)*] [$($args:tt)*] [@] >> $($after:tt)*) => {
        $crate::__split_member_owner!(
            @after_generic $kind $mode
            [$($owner)+] [$($segment)*] [$($args)* >] $($after)*)
    };
    (@generic $kind:ident $mode:ident
        [$($owner:tt)+] [$($segment:tt)*] [$($args:tt)*]
        [@ @ $($depth:tt)*] >> $($rest:tt)*) => {
        $crate::__split_member_owner!(
            @generic $kind $mode
            [$($owner)+] [$($segment)*] [$($args)* >>] [$($depth)*] $($rest)*)
    };

    // A close at depth zero terminates this turbofish. Nested angle brackets are kept
    // verbatim so rustc receives the exact generic arguments written by the caller.
    (@generic $kind:ident $mode:ident
        [$($owner:tt)+] [$($segment:tt)*] [$($args:tt)*] [] > $($after:tt)*) => {
        $crate::__split_member_owner!(
            @after_generic $kind $mode
            [$($owner)+] [$($segment)*] [$($args)*] $($after)*)
    };
    (@generic $kind:ident $mode:ident
        [$($owner:tt)+] [$($segment:tt)*] [$($args:tt)*]
        [$($depth:tt)*] < $($rest:tt)*) => {
        $crate::__split_member_owner!(
            @generic $kind $mode
            [$($owner)+] [$($segment)*] [$($args)* <] [@ $($depth)*] $($rest)*)
    };
    (@generic $kind:ident $mode:ident
        [$($owner:tt)+] [$($segment:tt)*] [$($args:tt)*]
        [@ $($depth:tt)*] > $($rest:tt)*) => {
        $crate::__split_member_owner!(
            @generic $kind $mode
            [$($owner)+] [$($segment)*] [$($args)* >] [$($depth)*] $($rest)*)
    };
    (@generic $kind:ident $mode:ident
        [$($owner:tt)+] [$($segment:tt)*] [$($args:tt)*]
        [$($depth:tt)*] $next:tt $($rest:tt)*) => {
        $crate::__split_member_owner!(
            @generic $kind $mode
            [$($owner)+] [$($segment)*] [$($args)* $next] [$($depth)*] $($rest)*)
    };

    // Owner generics must be followed by another path segment because the macro names
    // a member rather than the owner itself.
    (@after_generic $kind:ident owner
        [$($owner:tt)+] [] [$($args:tt)*] $($after:tt)+) => {
        $crate::__split_member_owner!(
            @path $kind [$($owner)+ :: < $($args)* >] $($after)+)
    };

    // A terminal generic segment is necessarily a method. Parsing its arguments as
    // `ty` here enforces the explicit-type-only generic method contract.
    (@after_generic method segment
        [$($owner:tt)+] [$method:ident] [$($args:tt)*]) => {
        $crate::__split_member_owner!(
            @finish_method [$($owner)+] $method [$($args)*])
    };
    (@after_generic $kind:ident segment
        [$($owner:tt)+] [$segment:ident] [$($args:tt)*] $($after:tt)+) => {
        $crate::__split_member_owner!(
            @path $kind
            [$($owner)+ :: $segment :: < $($args)* >] $($after)+)
    };
    (@finish_method
        [$($owner:tt)+] $method:ident [$($arg:ty),+ $(,)?]) => {{
        let _ = &$($owner)+::$method::<$($arg),*>;
        $crate::__member_name(
            $crate::type_name::<$($owner)+>(),
            stringify!($method),
            ::std::boxed::Box::new([$($crate::type_name::<$arg>()),*]))
    }};

    // Exhausting the input in a non-terminal state gives callers a stable diagnostic
    // instead of exposing this implementation macro's internal matcher states.
    (@path field [$($owner:tt)+] $($invalid:tt)*) => {
        compile_error!("expected a named owner path followed by `::field`")
    };
    (@path method [$($owner:tt)+] $($invalid:tt)*) => {
        compile_error!(
            "expected a named owner path followed by `::method` or `::method::<Types...>`")
    };
    (@path variant [$($owner:tt)+] $($invalid:tt)*) => {
        compile_error!("expected a named owner path followed by a supported variant shape")
    };
    (@generic $kind:ident $mode:ident
        [$($owner:tt)+] [$($segment:tt)*] [$($args:tt)*] [$($depth:tt)*]) => {
        compile_error!("unclosed generic argument list in member owner path")
    };
    (@after_generic $kind:ident $mode:ident
        [$($owner:tt)+] [$($segment:tt)*] [$($args:tt)*]) => {
        compile_error!("a generic field or variant name is not supported")
    };
    (@finish_method [$($owner:tt)+] $method:ident [$($invalid:tt)*]) => {
        compile_error!("generic method arguments must be explicit types")
    };
}

/// Gets a validated field name with its compiler-resolved owner.
///
/// This macro resolves `Self` to the appropriate type when used inside an `impl` block.
///
/// The owner must be a named type path. Qualified and generic owners use ordinary Rust
/// path syntax such as `module::Type::field` and `Type::<T>::field`.
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
///     pretty_name::of_field!(MyGenericStruct::<u32>::my_field).to_string(),
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
    (<$($invalid:tt)*) => {
        compile_error!(
            "angle-qualified owners are unsupported; use `Type::<Args>::field`")
    };
    ($head:ident :: $($rest:tt)+) => {
        $crate::__split_member_owner!(@path field [$head] :: $($rest)+)
    };
    ($($invalid:tt)*) => {
        compile_error!("expected a named owner path followed by `::field`")
    };
}

/// Gets a validated method name with its compiler-resolved owner and type arguments.
///
/// This macro resolves `Self` to the appropriate type when used inside an `impl` block.
///
/// The owner must be a named type path. Qualified and generic owners use ordinary Rust
/// path syntax such as `module::Type::method` and `Type::<T>::method`.
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
///     pretty_name::of_method!(MyGenericStruct::<u32>::my_method).to_string(),
///     "<MyGenericStruct<u32>>::my_method");
/// assert_eq!(
///     pretty_name::of_method!(
///         MyGenericStruct::<u32>::my_generic_method::<String>).to_string(),
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
    (<$($invalid:tt)*) => {
        compile_error!(
            "angle-qualified owners are unsupported; use `Type::<Args>::method`")
    };
    ($head:ident :: $($rest:tt)+) => {
        $crate::__split_member_owner!(@path method [$head] :: $($rest)+)
    };
    ($($invalid:tt)*) => {
        compile_error!(
            "expected a named owner path followed by `::method` or `::method::<Types...>`")
    };
}

/// Gets a validated enum variant name with its compiler-resolved owner.
///
/// This macro resolves `Self` to the appropriate type when used inside an `impl` block.
///
/// This macro supports unit, tuple, and struct variants. The struct form names one
/// field so Rust can distinguish it from unit and tuple variants; a bare `{ .. }`
/// pattern is valid for every variant shape and therefore cannot provide that check.
///
/// The owner must be a named enum path. Qualified and generic owners use ordinary Rust
/// path syntax such as `module::MyEnum::Variant` and `MyEnum::<T>::Variant`.
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
///     pretty_name::of_variant!(MyEnum::StructVariant { field, .. }).to_string(),
///     "<MyEnum>::StructVariant");
/// assert_eq!(
///     pretty_name::of_variant!(MyGenericEnum::<u32>::UnitVariant).to_string(),
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
    (<$($invalid:tt)*) => {
        compile_error!(
            "angle-qualified owners are unsupported; use `Type::<Args>::Variant`")
    };
    ($head:ident :: $($rest:tt)+) => {
        $crate::__split_member_owner!(@path variant [$head] :: $($rest)+)
    };
    ($($invalid:tt)*) => {
        compile_error!("expected a named owner path followed by a supported variant shape")
    };
}
