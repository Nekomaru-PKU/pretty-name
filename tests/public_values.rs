use std::fmt::Display;

/// An owner used to exercise field and member name values.
struct Owner {
    /// A field whose semantic existence is checked by the macro.
    field: u32,
}

impl Owner {
    /// A method whose semantic existence is checked by the macro.
    fn method(&self) {}
}

/// A generic function used to exercise function name values.
fn generic<T>() {}

/// Passes an opaque name through an ordinary `Display`-bounded API unchanged.
fn forward_name(name: impl Display) -> impl Display { name }

/// Returns a macro-produced name without exposing its concrete representation.
fn returned_name() -> impl Display { pretty_name::nameof!(generic::<u32>) }

/// Verifies macro values expose lazy display and explicit string materialization.
#[test]
fn macro_names_support_the_display_contract() {
    let local_value = 42;
    let name = forward_name(pretty_name::nameof!(local_value));

    assert_eq!(format!("{name}"), "local_value");
    assert_eq!(name.to_string(), "local_value");
    assert_eq!(returned_name().to_string(), "generic<u32>");
    assert_eq!(local_value, 42);
}

/// Verifies every macro category returns the same erased concrete type.
#[test]
fn all_macro_categories_share_one_opaque_type() {
    let owner = Owner { field: 42 };
    owner.method();
    let names = [
        pretty_name::nameof!(generic::<u32>),
        pretty_name::nameof_type!(u32),
        pretty_name::nameof_field!(Owner::field),
        pretty_name::nameof_member!(Owner::method),
    ];

    assert_eq!(
        names.map(|name| name.to_string()),
        ["generic<u32>", "u32", "Owner::field", "Owner::method"]);
    assert_eq!(owner.field, 42);
}

/// Verifies the public functions expose only the same usable `Display` contract.
#[test]
fn type_name_functions_support_the_display_contract() {
    let value = std::vec::Vec::<u32>::new();
    let by_type = forward_name(pretty_name::type_name::<std::vec::Vec<u32>>());
    let by_value = forward_name(pretty_name::type_name_of_val(&value));

    assert_eq!(format!("{by_type}"), "Vec<u32>");
    assert_eq!(by_value.to_string(), "Vec<u32>");
}
