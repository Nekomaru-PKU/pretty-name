use pretty_name::PrettyName;

/// An owner used to exercise the public member-name value.
struct Owner {
    /// A field whose semantic existence is checked by the macro.
    field: u32,
}

/// A generic function used to exercise the public function-name value.
fn generic<T>() {}

/// Verifies an ordinary value name exposes the shared opaque value contract.
#[test]
fn identifier_name_supports_display_to_string_and_debug() {
    let local_value = 42;
    let name: PrettyName = pretty_name::nameof!(local_value);

    assert_eq!(format!("{name}"), "local_value");
    assert_eq!(name.to_string(), "local_value");
    assert_eq!(format!("{name:?}"), "PrettyName(local_value)");
    assert_eq!(local_value, 42);
}

/// Verifies a generic function name uses the shared opaque value contract.
#[test]
fn function_name_supports_display_to_string_and_debug() {
    let name: PrettyName = pretty_name::nameof!(generic::<u32>);

    assert_eq!(format!("{name}"), "generic<u32>");
    assert_eq!(name.to_string(), "generic<u32>");
    assert_eq!(format!("{name:?}"), "PrettyName(generic<u32>)");
}

/// Verifies a member name uses the shared opaque value contract.
#[test]
fn member_name_supports_display_to_string_and_debug() {
    let owner = Owner { field: 42 };
    let name: PrettyName = pretty_name::nameof_field!(Owner::field);

    assert_eq!(format!("{name}"), "Owner::field");
    assert_eq!(name.to_string(), "Owner::field");
    assert_eq!(format!("{name:?}"), "PrettyName(Owner::field)");
    assert_eq!(owner.field, 42);
}

/// Verifies the type functions expose the documented opaque value contract.
#[test]
fn type_name_supports_display_to_string_and_debug() {
    let name: PrettyName = pretty_name::type_name::<std::vec::Vec<u32>>();

    assert_eq!(format!("{name}"), "Vec<u32>");
    assert_eq!(name.to_string(), "Vec<u32>");
    assert_eq!(format!("{name:?}"), "PrettyName(Vec<u32>)");
}
