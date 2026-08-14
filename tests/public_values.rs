use pretty_name::{FunctionName, IdentifierName, MemberName, TypeName};

/// An owner used to exercise the public member-name value.
struct Owner {
    /// A field whose semantic existence is checked by the macro.
    field: u32,
}

/// A generic function used to exercise the public function-name value.
fn generic<T>() {}

/// Verifies the identifier macro exposes the documented opaque value contract.
#[test]
fn identifier_name_supports_display_to_string_and_debug() {
    let local_value = 42;
    let name: IdentifierName = pretty_name::of_var!(local_value);

    assert_eq!(format!("{name}"), "local_value");
    assert_eq!(name.to_string(), "local_value");
    assert!(
        format!("{name:?}").starts_with("IdentifierName("),
        "the derived debug representation should identify its value type");
    assert_eq!(local_value, 42);
}

/// Verifies the function macro exposes the documented opaque value contract.
#[test]
fn function_name_supports_display_to_string_and_debug() {
    let name: FunctionName = pretty_name::of_function!(generic::<u32>);

    assert_eq!(format!("{name}"), "generic::<u32>");
    assert_eq!(name.to_string(), "generic::<u32>");
    assert!(
        format!("{name:?}").starts_with("FunctionName {"),
        "the derived debug representation should identify its value type");
}

/// Verifies the member macros expose the documented opaque value contract.
#[test]
fn member_name_supports_display_to_string_and_debug() {
    let owner = Owner { field: 42 };
    let name: MemberName = pretty_name::of_field!(Owner::field);

    assert_eq!(format!("{name}"), "<Owner>::field");
    assert_eq!(name.to_string(), "<Owner>::field");
    assert!(
        format!("{name:?}").starts_with("MemberName {"),
        "the derived debug representation should identify its value type");
    assert_eq!(owner.field, 42);
}

/// Verifies the type functions expose the documented opaque value contract.
#[test]
fn type_name_supports_display_to_string_and_debug() {
    let name: TypeName = pretty_name::type_name::<std::vec::Vec<u32>>();

    assert_eq!(format!("{name}"), "Vec<u32>");
    assert_eq!(name.to_string(), "Vec<u32>");
    assert!(
        format!("{name:?}").starts_with("TypeName("),
        "the derived debug representation should identify its value type");
}
