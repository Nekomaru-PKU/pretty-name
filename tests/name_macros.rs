use std::sync::atomic::{AtomicUsize, Ordering};

mod fixtures {
    /// A constant used to verify constant identifier names.
    pub const NAMED_CONSTANT: u32 = 42;

    /// A plain function used to verify identifier-only function names.
    pub fn plain_function() {}

    /// A generic function used to verify explicit type arguments.
    pub fn generic_function<T>() {}

    /// A two-argument generic function used to verify argument formatting.
    pub fn generic_pair<T, U>() {}

    /// Gets the name of [`generic_function`] for the caller's concrete type.
    pub fn generic_function_name<T>() -> impl std::fmt::Display {
        pretty_name::nameof!(generic_function::<T>)
    }

    /// Gets the resolved name of a possibly unsized generic type parameter.
    pub fn generic_type_name<T: ?Sized>() -> impl std::fmt::Display {
        pretty_name::nameof_type!(T)
    }

    /// A non-generic owner used to verify literal and `Self`-based macro forms.
    pub struct PlainOwner {
        /// A field referenced by field-name macros.
        pub field: u32,
    }

    impl PlainOwner {
        /// An associated constant referenced by member-name macros.
        pub const CONSTANT: u32 = 42;

        /// A method referenced by method-name macros.
        pub fn method(&self) {}

        /// A generic method referenced by method-name macros.
        pub fn generic_method<T>(&self) {}

        /// Gets this owner's type name through `Self`.
        pub fn self_type_name() -> impl std::fmt::Display {
            pretty_name::nameof_type!(Self)
        }

        /// Gets this owner's field name through `Self`.
        pub fn self_field_name() -> impl std::fmt::Display {
            pretty_name::nameof_field!(Self::field)
        }

        /// Gets this owner's method name through `Self`.
        pub fn self_method_name() -> impl std::fmt::Display {
            pretty_name::nameof_member!(Self::method)
        }

        /// Gets this owner's generic method name through `Self`.
        pub fn self_generic_method_name<T>() -> impl std::fmt::Display {
            pretty_name::nameof_member!(Self::generic_method::<T>)
        }
    }

    /// A trait used to verify methods resolve through bounded owner parameters.
    pub trait Named {
        /// A trait-provided method referenced through its implementing type.
        fn trait_method(&self);
    }

    impl Named for PlainOwner {
        fn trait_method(&self) {}
    }

    /// Gets a trait-provided method name through a resolved bounded owner parameter.
    pub fn trait_method_name<T: Named>() -> impl std::fmt::Display {
        pretty_name::nameof_member!(T::trait_method)
    }

    /// A generic owner used to verify named-path macro forms.
    pub struct GenericOwner<T> {
        /// A field referenced by qualified field-name macros.
        pub value: T,
    }

    impl<T> GenericOwner<T> {
        /// An associated function referenced through a qualified generic owner.
        pub fn associated_function() {}

        /// A generic associated function referenced through a qualified generic owner.
        pub fn generic_associated_function<U>() {}

        /// A method referenced by qualified method-name macros.
        pub fn method(&self) {}

        /// A generic method referenced by qualified method-name macros.
        pub fn generic_method<U>(&self) {}

        /// Gets the method name for this concrete `Self` type.
        pub fn self_method_name() -> impl std::fmt::Display {
            pretty_name::nameof_member!(Self::method)
        }
    }

    /// An enum containing every supported variant category.
    pub enum SimpleEnum {
        /// A unit variant used by variant-name macros.
        Unit,
        /// A tuple variant used by variant-name macros.
        Tuple(u32),
    }

    /// A generic enum used to verify qualified and `Self`-based variant names.
    pub enum GenericEnum<T> {
        /// A unit variant used by variant-name macros.
        Unit,
        /// A tuple variant used by variant-name macros.
        Tuple(T),
    }

    impl<T> GenericEnum<T> {
        /// Gets the unit variant name through `Self`.
        pub fn self_unit_name() -> impl std::fmt::Display {
            pretty_name::nameof_member!(Self::Unit)
        }

        /// Gets the tuple-constructor variant name through `Self`.
        pub fn self_tuple_name() -> impl std::fmt::Display {
            pretty_name::nameof_member!(Self::Tuple)
        }
    }
}

/// Counts destructor calls caused by constructing [`DroppingEnum`] values.
static VARIANT_DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Counts user-defined dereferences caused by field validation.
static FIELD_DEREF_COUNT: AtomicUsize = AtomicUsize::new(0);

/// An owner that makes field-access autoderef observable if validation ever runs.
struct DerefOwner(PlainOwner);

impl std::ops::Deref for DerefOwner {
    type Target = PlainOwner;

    fn deref(&self) -> &Self::Target {
        FIELD_DEREF_COUNT.fetch_add(1, Ordering::Relaxed);
        &self.0
    }
}

/// An enum whose destructor makes accidental variant construction observable.
enum DroppingEnum {
    /// A unit variant referenced only for its source identifier.
    Unit,
}

impl Drop for DroppingEnum {
    fn drop(&mut self) { VARIANT_DROP_COUNT.fetch_add(1, Ordering::Relaxed); }
}

use fixtures::{
    GenericEnum,
    GenericOwner,
    NAMED_CONSTANT,
    PlainOwner,
    SimpleEnum,
    generic_function as renamed_generic_function,
    generic_function_name,
    generic_pair,
    generic_type_name,
    plain_function,
    trait_method_name,
};

/// Builds owned expected names for tuple-based macro cases.
macro_rules! names {
    ($($name:literal),+ $(,)?) => { ($(String::from($name)),+) };
}

/// Verifies variable and constant identifiers retain their source spelling.
#[test]
fn variable_macro_returns_variable_and_constant_names() {
    let local_variable = 7;

    assert_eq!(
        (
            pretty_name::nameof!(local_variable).to_string(),
            pretty_name::nameof!(NAMED_CONSTANT).to_string()),
        names!("local_variable", "NAMED_CONSTANT"));
}

/// Verifies the identifier-only function form returns a display value.
#[test]
fn function_macro_returns_plain_function_name() {
    assert_eq!(
        pretty_name::nameof!(plain_function).to_string(),
        "plain_function");
}

/// Verifies explicit generic function arguments are shortened and comma-separated.
#[test]
fn function_macro_formats_explicit_generic_arguments() {
    assert_eq!(
        pretty_name::nameof!(
            generic_pair::<std::vec::Vec<u8>, std::string::String>).to_string(),
        "generic_pair<Vec<u8>, String>");
}

/// Verifies module qualification is validated and retained lexically.
#[test]
fn function_macro_accepts_a_module_qualified_function() {
    assert_eq!(
        pretty_name::nameof!(fixtures::plain_function).to_string(),
        "fixtures::plain_function");
}

/// Verifies a qualified generic function retains its path and resolves type arguments.
#[test]
fn function_macro_accepts_a_module_qualified_generic_function() {
    assert_eq!(
        pretty_name::nameof!(
            fixtures::generic_pair::<std::vec::Vec<u8>, String>).to_string(),
        "fixtures::generic_pair<Vec<u8>, String>");
}

/// Verifies an absolute function path retains its leading root marker.
#[test]
fn function_macro_accepts_an_absolute_generic_function_path() {
    assert_eq!(
        pretty_name::nameof!(::std::mem::drop::<u32>).to_string(),
        "::std::mem::drop<u32>");
}

/// Verifies a renamed module stays lexical while generic arguments stay resolved.
#[test]
fn function_macro_preserves_a_renamed_module_path() {
    use fixtures as renamed_fixtures;

    assert_eq!(
        pretty_name::nameof!(renamed_fixtures::generic_function::<String>).to_string(),
        "renamed_fixtures::generic_function<String>");
}

/// Verifies meaningful module paths distinguish otherwise identical function names.
#[test]
fn function_macro_distinguishes_standard_functions_by_module_path() {
    assert_eq!(
        (
            pretty_name::nameof!(std::array::from_mut::<u8>).to_string(),
            pretty_name::nameof!(std::slice::from_mut::<u8>).to_string()),
        names!("std::array::from_mut<u8>", "std::slice::from_mut<u8>"));
}

/// Verifies associated constants use the member owner's display form.
#[test]
fn member_macro_accepts_an_associated_constant() {
    assert_eq!(
        pretty_name::nameof_member!(PlainOwner::CONSTANT).to_string(),
        "PlainOwner::CONSTANT");
}

/// Verifies a generic associated function retains its resolved owner.
#[test]
fn member_macro_accepts_an_angle_wrapped_associated_function() {
    assert_eq!(
        pretty_name::nameof_member!(
            <fixtures::GenericOwner::<std::vec::Vec<u8>>>::associated_function).to_string(),
        "GenericOwner<Vec<u8>>::associated_function");
}

/// Verifies associated-function type arguments use the member display grammar.
#[test]
fn member_macro_accepts_an_angle_wrapped_generic_associated_function() {
    assert_eq!(
        pretty_name::nameof_member!(
            <fixtures::GenericOwner<u32>>::generic_associated_function::<
                std::vec::Vec<u8>>).to_string(),
        "GenericOwner<u32>::generic_associated_function<Vec<u8>>");
}

/// Verifies a renamed import is validated semantically while retaining its source name.
#[test]
fn function_macro_preserves_a_renamed_import_identifier() {
    assert_eq!(
        pretty_name::nameof!(renamed_generic_function::<u32>).to_string(),
        "renamed_generic_function<u32>");
}

/// Verifies one function macro call site resolves each monomorphization independently.
#[test]
fn function_values_distinguish_generic_monomorphizations() {
    assert_eq!(
        (
            generic_function_name::<u8>().to_string(),
            generic_function_name::<u16>().to_string()),
        names!("generic_function<u8>", "generic_function<u16>"));
}

/// Verifies a function name owns its arguments and can be formatted after construction.
#[test]
fn function_values_can_be_stored_for_later_formatting() {
    let name = generic_function_name::<std::vec::Vec<u32>>();

    assert_eq!(name.to_string(), "generic_function<Vec<u32>>");
}

/// Verifies simple and compound type forms both resolve aliases.
#[test]
fn type_macro_resolves_aliases_consistently() {
    /// A local alias used to verify semantic resolution.
    type IntegerAlias = u32;

    assert_eq!(
        (
            pretty_name::nameof_type!(IntegerAlias).to_string(),
            pretty_name::nameof_type!(Option<IntegerAlias>).to_string()),
        ("u32".to_owned(), "Option<u32>".to_owned()));
}

/// Verifies qualified generic type paths are semantically shortened.
#[test]
fn type_macro_shortens_qualified_generic_paths() {
    assert_eq!(
        pretty_name::nameof_type!(
            std::collections::HashMap<std::string::String, i32>).to_string(),
        "HashMap<String, i32>");
}

/// Verifies `Self` resolves to the concrete owner from an integration crate.
#[test]
fn type_macro_resolves_self_to_the_concrete_owner() {
    assert_eq!(PlainOwner::self_type_name().to_string(), "PlainOwner");
}

/// Verifies semantic resolution does not impose an implicit `Sized` bound.
#[test]
fn type_macro_accepts_unsized_generic_parameters() {
    assert_eq!(generic_type_name::<str>().to_string(), "str");
}

/// Verifies the simple field form resolves and uniformly qualifies its owner.
#[test]
fn field_macro_returns_simple_owner_and_field_name() {
    let owner = PlainOwner { field: 42 };

    assert_eq!(
        (pretty_name::nameof_field!(PlainOwner::field).to_string(), owner.field),
        ("PlainOwner::field".to_owned(), 42));
}

/// Verifies qualified generic field owners are semantically shortened.
#[test]
fn field_macro_shortens_qualified_generic_owner() {
    let owner = GenericOwner { value: 42_u32 };

    assert_eq!(
        (
            pretty_name::nameof_field!(
                <fixtures::GenericOwner::<u32>>::value).to_string(),
            owner.value),
        ("GenericOwner<u32>::value".to_owned(), 42));
}

/// Verifies field owners resolve through type aliases rather than source spelling.
#[test]
fn field_macro_resolves_an_owner_alias() {
    /// An owner alias whose source spelling must not leak into semantic output.
    type OwnerAlias = PlainOwner;

    assert_eq!(
        pretty_name::nameof_field!(OwnerAlias::field).to_string(),
        "PlainOwner::field");
}

/// Verifies the `Self` field form resolves the concrete owner.
#[test]
fn field_macro_resolves_self_to_the_concrete_owner() {
    assert_eq!(PlainOwner::self_field_name().to_string(), "PlainOwner::field");
}

/// Verifies field validation type-checks autoderef without calling user code.
#[test]
fn field_macro_does_not_run_deref_during_validation() {
    FIELD_DEREF_COUNT.store(0, Ordering::Relaxed);
    let name = pretty_name::nameof_field!(DerefOwner::field);

    assert_eq!(
        (name.to_string(), FIELD_DEREF_COUNT.load(Ordering::Relaxed)),
        (String::from("DerefOwner::field"), 0));
}

/// Verifies the simple method form resolves and uniformly qualifies its owner.
#[test]
fn method_macro_returns_simple_owner_and_method_name() {
    let owner = PlainOwner { field: 42 };
    owner.method();

    assert_eq!(
        pretty_name::nameof_member!(PlainOwner::method).to_string(),
        "PlainOwner::method");
}

/// Verifies explicit generic method arguments are shortened.
#[test]
fn method_macro_formats_explicit_generic_arguments() {
    let owner = PlainOwner { field: 42 };
    owner.generic_method::<std::string::String>();

    assert_eq!(
        pretty_name::nameof_member!(
            PlainOwner::generic_method::<std::vec::Vec<u8>>).to_string(),
        "PlainOwner::generic_method<Vec<u8>>");
}

/// Verifies qualified generic owners and method arguments are shortened together.
#[test]
fn method_macro_shortens_qualified_owner_and_generic_arguments() {
    let owner = GenericOwner { value: 42_u32 };
    owner.method();
    owner.generic_method::<String>();

    assert_eq!(
        (
            pretty_name::nameof_member!(
                <fixtures::GenericOwner<u32>>::method).to_string(),
            pretty_name::nameof_member!(
                <fixtures::GenericOwner<u32>>::generic_method::<
                    std::string::String>).to_string()),
        names!(
            "GenericOwner<u32>::method",
            "GenericOwner<u32>::generic_method<String>"));
}

/// Verifies method owners resolve through type aliases rather than source spelling.
#[test]
fn method_macro_resolves_an_owner_alias() {
    /// An owner alias whose source spelling must not leak into semantic output.
    type OwnerAlias = PlainOwner;

    assert_eq!(
        pretty_name::nameof_member!(OwnerAlias::method).to_string(),
        "PlainOwner::method");
}

/// Verifies trait methods use the concrete bounded owner rather than a trait declaration.
#[test]
fn method_macro_resolves_a_bounded_owner_parameter() {
    assert_eq!(
        trait_method_name::<PlainOwner>().to_string(),
        "PlainOwner::trait_method");
}

/// Verifies both non-generic and generic `Self` method forms resolve their owner.
#[test]
fn method_macro_resolves_self_and_generic_arguments() {
    assert_eq!(
        (
            PlainOwner::self_method_name().to_string(),
            PlainOwner::self_generic_method_name::<std::vec::Vec<u8>>().to_string()),
        names!(
            "PlainOwner::method",
            "PlainOwner::generic_method<Vec<u8>>"));
}

/// Verifies `Self` method values retain their resolved owner monomorphization.
#[test]
fn self_method_values_distinguish_generic_monomorphizations() {
    assert_eq!(
        (
            GenericOwner::<u8>::self_method_name().to_string(),
            GenericOwner::<u16>::self_method_name().to_string()),
        names!("GenericOwner<u8>::method", "GenericOwner<u16>::method"));
}

/// Verifies simple unit and tuple variants use the same bare syntax and owner format.
#[test]
fn variant_macro_supports_unit_and_tuple_variants() {
    let unit_name = match SimpleEnum::Unit {
        SimpleEnum::Unit => pretty_name::nameof_member!(SimpleEnum::Unit),
        _ => unreachable!(),
    };
    let tuple_value = match SimpleEnum::Tuple(7) {
        SimpleEnum::Tuple(value) => value,
        _ => unreachable!(),
    };
    assert_eq!(
        (
            unit_name.to_string(),
            pretty_name::nameof_member!(SimpleEnum::Tuple).to_string(),
            tuple_value),
        (
            String::from("SimpleEnum::Unit"),
            String::from("SimpleEnum::Tuple"),
            7));
}

/// Verifies validation does not construct and drop a unit-variant value at runtime.
#[test]
fn variant_macro_does_not_evaluate_the_referenced_variant() {
    VARIANT_DROP_COUNT.store(0, Ordering::Relaxed);
    let name = pretty_name::nameof_member!(DroppingEnum::Unit);

    assert_eq!(
        (name.to_string(), VARIANT_DROP_COUNT.load(Ordering::Relaxed)),
        (String::from("DroppingEnum::Unit"), 0));
}

/// Verifies generic variant constructors support qualification and source-spelled aliases.
#[test]
fn variant_macro_supports_generic_variant_constructors() {
    /// An alias that gives the tuple variant a simple owner path.
    type GenericU32 = GenericEnum<u32>;

    let unit_name = match GenericEnum::<u32>::Unit {
        GenericEnum::Unit => {
            pretty_name::nameof_member!(<fixtures::GenericEnum<u32>>::Unit)
        }
        _ => unreachable!(),
    };
    let tuple_value = match GenericEnum::Tuple(7_u32) {
        GenericEnum::Tuple(value) => value,
        _ => unreachable!(),
    };
    assert_eq!(
        (
            unit_name.to_string(),
            pretty_name::nameof_member!(GenericU32::Tuple).to_string(),
            tuple_value),
        (
            String::from("GenericEnum<u32>::Unit"),
            String::from("GenericEnum<u32>::Tuple"),
            7));
}

/// Verifies `Self` supports both variant categories for one concrete owner.
#[test]
fn variant_macro_supports_unit_and_tuple_variants_through_self() {
    assert_eq!(
        (
            GenericEnum::<u32>::self_unit_name().to_string(),
            GenericEnum::<u32>::self_tuple_name().to_string()),
        names!(
            "GenericEnum<u32>::Unit",
            "GenericEnum<u32>::Tuple"));
}

/// Verifies `Self` variant values retain their resolved owner monomorphization.
#[test]
fn self_variant_values_distinguish_generic_monomorphizations() {
    assert_eq!(
        (
            GenericEnum::<u8>::self_unit_name().to_string(),
            GenericEnum::<u16>::self_unit_name().to_string()),
        names!("GenericEnum<u8>::Unit", "GenericEnum<u16>::Unit"));
}
