use pretty_name::{TypeName, type_name, type_name_of_val};

/// Formats a resolved type through the public value API.
fn displayed_type_name<T: ?Sized>() -> String { type_name::<T>().to_string() }

/// Builds owned expected names for tuple-based formatter cases.
macro_rules! names {
    ($($name:literal),+ $(,)?) => { ($(String::from($name)),+) };
}

/// A const-generic owner used to verify that non-type arguments remain intact.
struct ConstGeneric<const LENGTH: usize>;

/// Verifies primitive and unsized primitive names remain unchanged.
#[test]
fn primitives_and_unsized_primitives_keep_their_spelling() {
    assert_eq!(
        (
            displayed_type_name::<i32>(),
            displayed_type_name::<bool>(),
            displayed_type_name::<str>(),
            displayed_type_name::<[i32]>()),
        names!("i32", "bool", "str", "[i32]"));
}

/// Verifies references preserve the compiler's spelling, mutability, and nesting.
#[test]
fn references_preserve_compiler_spelling_and_structure() {
    assert_eq!(
        (
            displayed_type_name::<&i32>(),
            displayed_type_name::<&'static str>(),
            displayed_type_name::<&&&str>(),
            displayed_type_name::<&[i32]>(),
            displayed_type_name::<&mut String>(),
            displayed_type_name::<&mut &str>(),
            displayed_type_name::<&mut str>(),
            displayed_type_name::<&mut [i32]>()),
        names!(
            "&i32",
            "&str",
            "&&&str",
            "&[i32]",
            "&mut String",
            "&mut &str",
            "&mut str",
            "&mut [i32]"));
}

/// Verifies raw pointer kind and nested pointee structure remain visible.
#[test]
fn raw_pointers_preserve_constness_mutability_and_nesting() {
    assert_eq!(
        (
            displayed_type_name::<*const i32>(),
            displayed_type_name::<*mut i32>(),
            displayed_type_name::<*const str>(),
            displayed_type_name::<*mut [u8]>(),
            displayed_type_name::<*const *mut i32>(),
            displayed_type_name::<*const &str>(),
            displayed_type_name::<&*const i32>()),
        names!(
            "*const i32",
            "*mut i32",
            "*const str",
            "*mut [u8]",
            "*const *mut i32",
            "*const &str",
            "&*const i32"));
}

/// Verifies arrays retain their lengths and recursively formatted element types.
#[test]
fn arrays_preserve_lengths_and_nested_elements() {
    assert_eq!(
        (
            displayed_type_name::<[i32; 5]>(),
            displayed_type_name::<[bool; 0]>(),
            displayed_type_name::<&[i32; 3]>(),
            displayed_type_name::<&mut [i32; 5]>(),
            displayed_type_name::<[[i32; 2]; 3]>(),
            displayed_type_name::<[[[u8; 2]; 3]; 4]>(),
            displayed_type_name::<[(i32, bool); 10]>(),
            displayed_type_name::<[(); 5]>()),
        names!(
            "[i32; 5]",
            "[bool; 0]",
            "&[i32; 3]",
            "&mut [i32; 5]",
            "[[i32; 2]; 3]",
            "[[[u8; 2]; 3]; 4]",
            "[(i32, bool); 10]",
            "[(); 5]"));
}

/// Verifies tuples retain arity, nesting, and reference qualifiers.
#[test]
fn tuples_preserve_arity_and_nested_elements() {
    assert_eq!(
        (
            displayed_type_name::<()>(),
            displayed_type_name::<(i32,)>(),
            displayed_type_name::<(i32, String, bool)>(),
            displayed_type_name::<(i32, (String, bool))>(),
            displayed_type_name::<(&str, &[u8])>(),
            displayed_type_name::<(&mut String, &i32)>()),
        names!(
            "()",
            "(i32,)",
            "(i32, String, bool)",
            "(i32, (String, bool))",
            "(&str, &[u8])",
            "(&mut String, &i32)"));
}

/// Verifies generic containers recursively shorten their argument paths.
#[test]
fn generic_containers_shorten_nested_paths() {
    assert_eq!(
        (
            displayed_type_name::<Option<i32>>(),
            displayed_type_name::<Option<&str>>(),
            displayed_type_name::<Result<i32, String>>(),
            displayed_type_name::<Result<(), ()>>(),
            displayed_type_name::<Vec<Vec<Vec<i32>>>>(),
            displayed_type_name::<Option<Result<i32, String>>>(),
            displayed_type_name::<Box<Option<Vec<String>>>>(),
            displayed_type_name::<Vec<Option<&str>>>()),
        names!(
            "Option<i32>",
            "Option<&str>",
            "Result<i32, String>",
            "Result<(), ()>",
            "Vec<Vec<Vec<i32>>>",
            "Option<Result<i32, String>>",
            "Box<Option<Vec<String>>>",
            "Vec<Option<&str>>"));
}

/// Verifies function pointers retain signatures, qualifiers, and nested function types.
#[test]
fn function_pointers_preserve_signature_details() {
    assert_eq!(
        (
            displayed_type_name::<fn()>(),
            displayed_type_name::<fn(i32) -> i32>(),
            displayed_type_name::<fn(i32, String, bool)>(),
            displayed_type_name::<fn(&str) -> String>(),
            displayed_type_name::<fn(&mut i32)>(),
            displayed_type_name::<fn(*const i32) -> *mut i32>(),
            displayed_type_name::<fn() -> fn(i32) -> i32>(),
            displayed_type_name::<fn(fn(i32) -> i32) -> i32>(),
            displayed_type_name::<unsafe fn()>(),
            displayed_type_name::<extern "C" fn(i32) -> i32>(),
            displayed_type_name::<unsafe extern "C" fn(i32)>()),
        names!(
            "fn()",
            "fn(i32) -> i32",
            "fn(i32, String, bool)",
            "fn(&'_ str) -> String",
            "fn(&'_ mut i32)",
            "fn(*const i32) -> *mut i32",
            "fn() -> fn(i32) -> i32",
            "fn(fn(i32) -> i32) -> i32",
            "unsafe fn()",
            "extern \"C\" fn(i32) -> i32",
            "unsafe extern \"C\" fn(i32)"));
}

/// Verifies trait objects retain bounds while shortening every trait path.
#[test]
fn trait_objects_preserve_bounds_and_shorten_paths() {
    assert_eq!(
        (
            displayed_type_name::<Box<dyn std::fmt::Debug>>(),
            displayed_type_name::<&dyn std::fmt::Display>(),
            displayed_type_name::<&mut dyn std::io::Write>(),
            displayed_type_name::<Box<dyn std::fmt::Debug + Send>>(),
            displayed_type_name::<Box<dyn std::fmt::Debug + Send + Sync>>(),
            displayed_type_name::<dyn std::fmt::Debug>(),
            displayed_type_name::<dyn std::fmt::Debug + Send>()),
        names!(
            "Box<dyn Debug>",
            "&dyn Display",
            "&mut dyn Write",
            "Box<dyn Debug + Send>",
            "Box<dyn Debug + Send + Sync>",
            "dyn Debug",
            "dyn Debug + Send"));
}

/// Verifies parenthesized trait inputs and outputs are shortened recursively.
#[test]
fn callable_trait_arguments_and_output_paths_are_shortened() {
    type Callback = dyn Fn(std::vec::Vec<std::string::String>)
        -> std::option::Option<std::path::PathBuf>;

    assert_eq!(displayed_type_name::<Callback>(), "dyn Fn(Vec<String>) -> Option<PathBuf>");
}

/// Verifies associated type bindings are shortened recursively.
#[test]
fn associated_type_binding_paths_are_shortened() {
    type IteratorObject = dyn Iterator<Item = std::vec::Vec<std::string::String>>;

    assert_eq!(displayed_type_name::<IteratorObject>(), "dyn Iterator<Item = Vec<String>>");
}

/// Verifies pointer-like standard containers lose their module qualification.
#[test]
fn smart_pointer_paths_are_shortened() {
    assert_eq!(
        (
            displayed_type_name::<Box<i32>>(),
            displayed_type_name::<Box<str>>(),
            displayed_type_name::<Box<[i32]>>(),
            displayed_type_name::<std::rc::Rc<String>>(),
            displayed_type_name::<std::sync::Arc<String>>(),
            displayed_type_name::<std::cell::RefCell<i32>>()),
        names!(
            "Box<i32>",
            "Box<str>",
            "Box<[i32]>",
            "Rc<String>",
            "Arc<String>",
            "RefCell<i32>"));
}

/// Verifies qualification is removed at every level of a composite type.
#[test]
fn composite_types_shorten_paths_recursively() {
    assert_eq!(
        (
            displayed_type_name::<std::vec::Vec<i32>>(),
            displayed_type_name::<std::string::String>(),
            displayed_type_name::<std::boxed::Box<i32>>(),
            displayed_type_name::<Result<Vec<u8>, std::io::Error>>(),
            displayed_type_name::<std::collections::HashMap<
                std::string::String,
                std::vec::Vec<i32>>>(),
            displayed_type_name::<Vec<Option<Result<Box<dyn std::fmt::Debug>, String>>>>(),
            displayed_type_name::<&[Option<&[(i32, &str)]>]>(),
            displayed_type_name::<fn(Vec<&str>)
                -> Option<Result<i32, Box<dyn std::error::Error>>>>()),
        names!(
            "Vec<i32>",
            "String",
            "Box<i32>",
            "Result<Vec<u8>, Error>",
            "HashMap<String, Vec<i32>>",
            "Vec<Option<Result<Box<dyn Debug>, String>>>",
            "&[Option<&[(i32, &str)]>]",
            "fn(Vec<&'_ str>) -> Option<Result<i32, Box<dyn Error>>>"));
}

/// Verifies const generic arguments survive path shortening unchanged.
#[test]
fn const_generic_arguments_are_preserved() {
    assert_eq!(displayed_type_name::<ConstGeneric<17>>(), "ConstGeneric<17>");
}

/// Verifies marker types retain recursively shortened generic arguments.
#[test]
fn marker_type_arguments_are_shortened() {
    assert_eq!(
        (
            displayed_type_name::<std::marker::PhantomData<i32>>(),
            displayed_type_name::<std::marker::PhantomData<&str>>()),
        names!("PhantomData<i32>", "PhantomData<&str>"));
}

/// Verifies value-based lookup reports the value's type rather than the reference wrapper.
#[test]
fn type_name_of_val_reports_the_borrowed_values_type() {
    let values = vec![1_i32, 2, 3];

    assert_eq!(
        (
            type_name_of_val(&values).to_string(),
            type_name_of_val(&values.as_slice()).to_string()),
        names!("Vec<i32>", "&[i32]"));
}

/// Verifies compiler descriptions outside Rust's type grammar remain informative.
#[test]
fn closure_names_are_preserved_when_they_cannot_be_parsed_as_rust_types() {
    let closure = || 42;

    assert_eq!(
        type_name_of_val(&closure).to_string(),
        std::any::type_name_of_val(&closure));
}

/// Verifies async-block descriptions use the same unchanged fallback as closures.
#[test]
fn async_block_names_are_preserved_when_they_cannot_be_parsed_as_rust_types() {
    let future = async { 42 };

    assert_eq!(
        type_name_of_val(&future).to_string(),
        std::any::type_name_of_val(&future));
}

/// Verifies long pretty-printed types are extracted without the internal alias wrapper.
#[test]
fn long_types_are_extracted_across_pretty_printer_line_wrapping() {
    type LongType = (
        std::collections::HashMap<
            std::string::String,
            Vec<Option<Result<Box<dyn std::error::Error>, std::path::PathBuf>>>>,
        std::collections::BTreeMap<
            std::string::String,
            Vec<Option<Result<Box<dyn std::fmt::Debug>, std::path::PathBuf>>>>);

    let name = displayed_type_name::<LongType>();

    assert!(
        name.starts_with('(')
            && name.ends_with(')')
            && name.contains("HashMap<String")
            && name.contains("BTreeMap<String")
            && !name.contains("__PrettyName"),
        "unexpected formatted type: {name}");
}

/// Verifies the public function returns the opaque display value.
#[test]
fn type_name_returns_type_name_value() {
    let name: TypeName = type_name::<std::num::NonZeroU8>();

    assert_eq!(name.to_string(), "NonZero<u8>");
}

/// Verifies the opaque value uses conventional derived structural debugging.
#[test]
fn type_name_debug_is_structural() {
    let debug = format!("{:?}", type_name::<std::num::NonZeroU8>());

    assert!(
        debug.starts_with("TypeName(") && debug.contains("NonZero"),
        "unexpected debug representation: {debug}");
}

/// Verifies aliases are transparent to compiler-resolved type naming.
#[test]
fn type_aliases_display_the_resolved_type() {
    type Bytes = std::vec::Vec<u8>;

    assert_eq!(displayed_type_name::<Bytes>(), "Vec<u8>");
}

/// Verifies renamed imports are transparent to compiler-resolved type naming.
#[test]
fn renamed_imports_display_the_resolved_type() {
    use std::collections::BTreeMap as OrderedMap;

    assert_eq!(displayed_type_name::<OrderedMap<String, u8>>(), "BTreeMap<String, u8>");
}
