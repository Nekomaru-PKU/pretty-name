use std::fmt;
use std::fmt::Write as _;

use super::{__function_name, __identifier_name, __member_name, type_name, TypeName};

/// An owner used to verify compound member formatting.
struct Owner;

/// A destination that proves `Display` forwards formatter failures to its caller.
struct RefusingWriter;

impl fmt::Write for RefusingWriter {
    fn write_str(&mut self, _value: &str) -> fmt::Result { Err(fmt::Error) }
}

/// Verifies identifier values preserve validated source spelling.
#[test]
fn identifier_display_preserves_source_spelling() {
    assert_eq!(__identifier_name("local_value").to_string(), "local_value");
}

/// Verifies functions without arguments omit generic punctuation.
#[test]
fn function_display_omits_empty_arguments() {
    assert_eq!(__function_name("function", Box::new([])).to_string(), "function");
}

/// Verifies a single function argument uses the compact function-name grammar.
#[test]
fn function_display_formats_one_argument() {
    assert_eq!(
        __function_name("function", Box::new([type_name::<u32>()])).to_string(),
        "function<u32>");
}

/// Verifies multiple function arguments use canonical separators without a trailing
/// comma.
#[test]
fn function_display_formats_many_arguments() {
    assert_eq!(
        __function_name(
            "function",
            Box::new([type_name::<std::vec::Vec<u8>>(), type_name::<String>()]))
        .to_string(),
        "function<Vec<u8>, String>");
}

/// Verifies fields use the uniform angle-bracketed owner grammar.
#[test]
fn member_display_formats_resolved_owner() {
    assert_eq!(
        __member_name(type_name::<Owner>(), "field", Box::new([])).to_string(),
        "<Owner>::field");
}

/// Verifies method arguments retain Rust's associated-item separator.
#[test]
fn member_display_formats_arguments() {
    assert_eq!(
        __member_name(
            type_name::<Owner>(),
            "method",
            Box::new([type_name::<String>()]))
        .to_string(),
        "<Owner>::method::<String>");
}

/// Verifies a plain qualified path loses only its module qualification.
#[test]
fn display_shortens_a_plain_qualified_path() {
    assert_eq!(TypeName("crate::model::Record").to_string(), "Record");
}

/// Verifies nested generic arguments are transformed recursively.
#[test]
fn display_shortens_nested_qualified_paths() {
    assert_eq!(
        TypeName(
            "std::collections::HashMap<std::string::String, crate::model::Record>")
        .to_string(),
        "HashMap<String, Record>");
}

/// Verifies leading global qualification is removed with module qualification.
#[test]
fn display_removes_a_leading_global_qualifier() {
    assert_eq!(TypeName("::crate_name::Record").to_string(), "Record");
}

/// Verifies compiler-emitted lifetimes survive qualification removal.
#[test]
fn display_preserves_reference_lifetimes() {
    assert_eq!(TypeName("&'named crate::model::Record").to_string(), "&'named Record");
}

/// Verifies qualified-self projections retain their owner, trait, and associated type
/// structure.
#[test]
fn display_preserves_qualified_self_projections() {
    assert_eq!(
        TypeName(
            "<crate::model::Record as crate::traits::HasItem>::Item").to_string(),
        "<Record as HasItem>::Item");
}

/// Verifies associated bounds are traversed through Syn's generic-argument grammar.
#[test]
fn display_shortens_associated_type_constraints() {
    assert_eq!(
        TypeName(
            "dyn crate::traits::Outer<Item: crate::fmt::Display>").to_string(),
        "dyn Outer<Item: Display>");
}

/// Verifies descriptions outside Rust's type grammar remain completely unchanged.
#[test]
fn display_preserves_unparseable_descriptions() {
    let description = "crate::module::{closure@src/lib.rs:1:1}";

    assert_eq!(TypeName(description).to_string(), description);
}

/// Verifies empty unknown input is preserved rather than panicking or inventing a name.
#[test]
fn display_preserves_an_empty_description() {
    assert_eq!(TypeName("").to_string(), "");
}

/// Verifies malformed generic input is preserved byte-for-byte.
#[test]
fn display_preserves_an_unclosed_generic_description() {
    let description = "crate::module::Wrapper<crate::model::Record";

    assert_eq!(TypeName(description).to_string(), description);
}

/// Verifies macro type tokens trigger whole-description fallback because Syn does not
/// expose their internal type grammar to the visitor.
#[test]
fn display_preserves_macro_types_without_partial_shortening() {
    let description = "crate::types::Wrapper<crate::type_macro!(crate::model::Record)>";

    assert_eq!(TypeName(description).to_string(), description);
}

/// Verifies macro const expressions trigger whole-description fallback for the same
/// opaque-token reason as macro types.
#[test]
fn display_preserves_macro_const_arguments_without_partial_shortening() {
    let description = "crate::types::Buffer<{ crate::length!() }>";

    assert_eq!(TypeName(description).to_string(), description);
}

/// Verifies destination failures remain ordinary formatting errors after parsing and
/// shortening complete.
#[test]
fn display_propagates_destination_errors() {
    let mut writer = RefusingWriter;

    assert!(write!(&mut writer, "{}", TypeName("crate::model::Record")).is_err());
}
