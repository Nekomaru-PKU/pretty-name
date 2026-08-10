mod fixtures {
    /// A constant used to verify constant identifier names.
    pub const NAMED_CONSTANT: u32 = 42;

    /// A plain function used to verify identifier-only function names.
    pub fn plain_function() {}

    /// A generic function used to verify explicit and omitted type arguments.
    pub fn generic_function<T>() {}

    /// A two-argument generic function used to verify argument formatting.
    pub fn generic_pair<T, U>() {}

    /// Gets the name of [`generic_function`] for the caller's concrete type.
    pub fn generic_function_name<T>() -> &'static str {
        pretty_name::of_function!(generic_function::<T>)
    }

    /// Gets the source spelling of a possibly unsized generic type parameter.
    pub fn generic_type_source_name<T: ?Sized>() -> &'static str {
        pretty_name::of_type!(T)
    }

    /// A non-generic owner used to verify literal and `Self`-based macro forms.
    pub struct PlainOwner {
        /// A field referenced by field-name macros.
        pub field: u32,
    }

    impl PlainOwner {
        /// A method referenced by method-name macros.
        pub fn method(&self) {}

        /// A generic method referenced by method-name macros.
        pub fn generic_method<T>(&self) {}

        /// Gets this owner's type name through `Self`.
        pub fn self_type_name() -> &'static str {
            pretty_name::of_type!(Self)
        }

        /// Gets this owner's field name through `Self`.
        pub fn self_field_name() -> &'static str {
            pretty_name::of_field!(Self::field)
        }

        /// Gets this owner's method name through `Self`.
        pub fn self_method_name() -> &'static str {
            pretty_name::of_method!(Self::method)
        }

        /// Gets this owner's generic method name through `Self`.
        pub fn self_generic_method_name<T>() -> &'static str {
            pretty_name::of_method!(Self::generic_method::<T>)
        }
    }

    /// A generic owner used to verify qualified macro forms and cache keys.
    pub struct GenericOwner<T> {
        /// A field referenced by qualified field-name macros.
        pub value: T,
    }

    impl<T> GenericOwner<T> {
        /// A method referenced by qualified method-name macros.
        pub fn method(&self) {}

        /// A generic method referenced by qualified method-name macros.
        pub fn generic_method<U>(&self) {}

        /// Gets the method name for this concrete `Self` type.
        pub fn self_method_name() -> &'static str {
            pretty_name::of_method!(Self::method)
        }
    }

    /// An enum containing every supported variant shape.
    pub enum SimpleEnum {
        /// A unit variant used by variant-name macros.
        Unit,
        /// A tuple variant used by variant-name macros.
        Tuple(u32),
        /// A struct variant used by variant-name macros.
        Struct {
            /// A payload used to make the struct shape concrete.
            value: u32,
        },
    }

    /// A generic enum used to verify qualified and `Self`-based variant names.
    pub enum GenericEnum<T> {
        /// A unit variant used by variant-name macros.
        Unit,
        /// A tuple variant used by variant-name macros.
        Tuple(T),
        /// A struct variant used by variant-name macros.
        Struct {
            /// A payload used to make the struct shape concrete.
            value: T,
        },
    }

    impl<T> GenericEnum<T> {
        /// Gets the unit variant name through `Self`.
        pub fn self_unit_name() -> &'static str {
            pretty_name::of_variant!(Self::Unit)
        }

        /// Gets the tuple variant name through `Self`.
        pub fn self_tuple_name() -> &'static str {
            pretty_name::of_variant!(Self::Tuple(..))
        }

        /// Gets the struct variant name through `Self`.
        pub fn self_struct_name() -> &'static str {
            pretty_name::of_variant!(Self::Struct {..})
        }
    }
}

use fixtures::{
    GenericEnum,
    GenericOwner,
    NAMED_CONSTANT,
    PlainOwner,
    SimpleEnum,
    generic_function,
    generic_function_name,
    generic_pair,
    generic_type_source_name,
    plain_function,
};

/// Verifies variable and constant identifiers retain their source spelling.
#[test]
fn variable_macro_returns_variable_and_constant_names() {
    let local_variable = 7;

    assert_eq!(
        (
            pretty_name::of_var!(local_variable),
            pretty_name::of_var!(NAMED_CONSTANT)),
        ("local_variable", "NAMED_CONSTANT"));
}

/// Verifies the identifier-only function form returns a string literal name.
#[test]
fn function_macro_returns_plain_function_name() {
    assert_eq!(pretty_name::of_function!(plain_function), "plain_function");
}

/// Verifies the placeholder function form validates generics without displaying them.
#[test]
fn function_macro_omits_generic_arguments_for_the_placeholder_form() {
    assert_eq!(pretty_name::of_function!(generic_function::<..>), "generic_function");
}

/// Verifies explicit generic function arguments are shortened and comma-separated.
#[test]
fn function_macro_formats_explicit_generic_arguments() {
    assert_eq!(
        pretty_name::of_function!(generic_pair::<std::vec::Vec<u8>, std::string::String>),
        "generic_pair::<Vec<u8>, String>");
}

/// Verifies one function macro call site keeps monomorphized cache entries separate.
#[test]
fn function_cache_distinguishes_generic_monomorphizations() {
    assert_eq!(
        (generic_function_name::<u8>(), generic_function_name::<u16>()),
        ("generic_function::<u8>", "generic_function::<u16>"));
}

/// Verifies one function macro call site reuses its allocation across threads.
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

/// Verifies simple type identifiers preserve aliases while semantic forms resolve them.
#[test]
fn type_macro_distinguishes_source_spelling_from_semantic_resolution() {
    /// A local alias used to expose the macro's source-spelling behavior.
    type IntegerAlias = u32;

    assert_eq!(
        (
            pretty_name::of_type!(IntegerAlias),
            pretty_name::of_type!(Option<IntegerAlias>)),
        ("IntegerAlias", "Option<u32>"));
}

/// Verifies qualified generic type paths are semantically shortened.
#[test]
fn type_macro_shortens_qualified_generic_paths() {
    assert_eq!(
        pretty_name::of_type!(std::collections::HashMap<std::string::String, i32>),
        "HashMap<String, i32>");
}

/// Verifies `Self` resolves to the concrete owner from an integration crate.
#[test]
fn type_macro_resolves_self_to_the_concrete_owner() {
    assert_eq!(PlainOwner::self_type_name(), "PlainOwner");
}

/// Verifies identifier validation does not impose an implicit `Sized` bound.
#[test]
fn type_macro_accepts_unsized_generic_parameters() {
    assert_eq!(generic_type_source_name::<str>(), "T");
}

/// Verifies the simple field form returns its source-spelled owner and field.
#[test]
fn field_macro_returns_simple_owner_and_field_name() {
    let owner = PlainOwner { field: 42 };

    assert_eq!(
        (pretty_name::of_field!(PlainOwner::field), owner.field),
        ("PlainOwner::field", 42));
}

/// Verifies qualified generic field owners are semantically shortened.
#[test]
fn field_macro_shortens_qualified_generic_owner() {
    let owner = GenericOwner { value: 42_u32 };

    assert_eq!(
        (pretty_name::of_field!(<fixtures::GenericOwner<u32>>::value), owner.value),
        ("<GenericOwner<u32>>::value", 42));
}

/// Verifies the `Self` field form resolves the concrete owner.
#[test]
fn field_macro_resolves_self_to_the_concrete_owner() {
    assert_eq!(PlainOwner::self_field_name(), "PlainOwner::field");
}

/// Verifies the simple method form returns its source-spelled owner and method.
#[test]
fn method_macro_returns_simple_owner_and_method_name() {
    let owner = PlainOwner { field: 42 };
    owner.method();

    assert_eq!(pretty_name::of_method!(PlainOwner::method), "PlainOwner::method");
}

/// Verifies explicit generic method arguments are shortened.
#[test]
fn method_macro_formats_explicit_generic_arguments() {
    let owner = PlainOwner { field: 42 };
    owner.generic_method::<std::string::String>();

    assert_eq!(
        pretty_name::of_method!(PlainOwner::generic_method::<std::vec::Vec<u8>>),
        "PlainOwner::generic_method::<Vec<u8>>");
}

/// Verifies qualified generic owners and method arguments are shortened together.
#[test]
fn method_macro_shortens_qualified_owner_and_generic_arguments() {
    let owner = GenericOwner { value: 42_u32 };
    owner.method();
    owner.generic_method::<String>();

    assert_eq!(
        (
            pretty_name::of_method!(<fixtures::GenericOwner<u32>>::method),
            pretty_name::of_method!(
                <fixtures::GenericOwner<u32>>::generic_method::<std::string::String>)),
        (
            "<GenericOwner<u32>>::method",
            "<GenericOwner<u32>>::generic_method::<String>"));
}

/// Verifies both non-generic and generic `Self` method forms resolve their owner.
#[test]
fn method_macro_resolves_self_and_generic_arguments() {
    assert_eq!(
        (
            PlainOwner::self_method_name(),
            PlainOwner::self_generic_method_name::<std::vec::Vec<u8>>()),
        ("PlainOwner::method", "PlainOwner::generic_method::<Vec<u8>>"));
}

/// Verifies `Self` method cache entries remain separate across owner monomorphizations.
#[test]
fn self_method_cache_distinguishes_generic_monomorphizations() {
    assert_eq!(
        (
            GenericOwner::<u8>::self_method_name(),
            GenericOwner::<u16>::self_method_name()),
        ("GenericOwner<u8>::method", "GenericOwner<u16>::method"));
}

/// Verifies simple unit, tuple, and struct variants return the same owner format.
#[test]
fn variant_macro_supports_every_simple_variant_shape() {
    let unit_name = match SimpleEnum::Unit {
        SimpleEnum::Unit => pretty_name::of_variant!(SimpleEnum::Unit),
        _ => unreachable!(),
    };
    let tuple_value = match SimpleEnum::Tuple(7) {
        SimpleEnum::Tuple(value) => value,
        _ => unreachable!(),
    };
    let struct_value = match (SimpleEnum::Struct { value: 9 }) {
        SimpleEnum::Struct { value } => value,
        _ => unreachable!(),
    };

    assert_eq!(
        (
            unit_name,
            pretty_name::of_variant!(SimpleEnum::Tuple(..)),
            pretty_name::of_variant!(SimpleEnum::Struct {..}),
            tuple_value,
            struct_value),
        (
            "SimpleEnum::Unit",
            "SimpleEnum::Tuple",
            "SimpleEnum::Struct",
            7,
            9));
}

/// Verifies stable generic variant forms support qualification and source-spelled aliases.
#[test]
fn variant_macro_supports_stable_generic_variant_forms() {
    /// An alias that gives tuple and struct variants a stable simple path.
    type GenericU32 = GenericEnum<u32>;

    let unit_name = match GenericEnum::<u32>::Unit {
        GenericEnum::Unit => {
            pretty_name::of_variant!(<fixtures::GenericEnum<u32>>::Unit)
        }
        _ => unreachable!(),
    };
    let tuple_value = match GenericEnum::Tuple(7_u32) {
        GenericEnum::Tuple(value) => value,
        _ => unreachable!(),
    };
    let struct_value = match (GenericEnum::Struct { value: 9_u32 }) {
        GenericEnum::Struct { value } => value,
        _ => unreachable!(),
    };

    assert_eq!(
        (
            unit_name,
            pretty_name::of_variant!(GenericU32::Tuple(..)),
            pretty_name::of_variant!(GenericU32::Struct {..}),
            tuple_value,
            struct_value),
        (
            "<GenericEnum<u32>>::Unit",
            "GenericU32::Tuple",
            "GenericU32::Struct",
            7,
            9));
}

/// Verifies `Self` supports every variant shape for one concrete owner.
#[test]
fn variant_macro_supports_every_self_variant_shape() {
    assert_eq!(
        (
            GenericEnum::<u32>::self_unit_name(),
            GenericEnum::<u32>::self_tuple_name(),
            GenericEnum::<u32>::self_struct_name()),
        (
            "GenericEnum<u32>::Unit",
            "GenericEnum<u32>::Tuple",
            "GenericEnum<u32>::Struct"));
}

/// Verifies `Self` variant cache entries remain separate across owner monomorphizations.
#[test]
fn self_variant_cache_distinguishes_generic_monomorphizations() {
    assert_eq!(
        (
            GenericEnum::<u8>::self_unit_name(),
            GenericEnum::<u16>::self_unit_name()),
        ("GenericEnum<u8>::Unit", "GenericEnum<u16>::Unit"));
}
