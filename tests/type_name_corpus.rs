/// Declares compiler-description cases as separate tests so any regression identifies
/// the exact Rust type shape that changed.
macro_rules! compiler_description_cases {
    ($($(#[$meta:meta])* $name:ident: $ty:ty => $expected:literal;)+) => {
        $(
            $(#[$meta])*
            #[test]
            fn $name() {
                let actual = pretty_name::type_name::<$ty>().to_string();
                assert_eq!(
                    actual,
                    $expected,
                    "unexpected diagnostic name for `{}`",
                    stringify!($ty));
            }
        )+
    };
}

/// A local const-generic type used to keep its owner path visible in the corpus.
struct Buffer<T, const LENGTH: usize>(std::marker::PhantomData<T>);

compiler_description_cases! {
    /// Verifies a primitive description needs no transformation.
    primitive_i32: i32 => "i32";
    /// Verifies an unsized primitive description needs no transformation.
    unsized_str: str => "str";
    /// Verifies a shared slice reference preserves its structure.
    shared_slice_reference: &[u8] => "&[u8]";
    /// Verifies a mutable unsized reference preserves mutability.
    mutable_str_reference: &mut str => "&mut str";
    /// Verifies nested raw pointers preserve both qualifiers.
    nested_raw_pointer: *const *mut u8 => "*const *mut u8";
    /// Verifies a fixed array preserves its const length.
    fixed_array: [u8; 16] => "[u8; 16]";
    /// Verifies arrays recursively preserve tuple element structure.
    tuple_array: [(i32, bool); 3] => "[(i32, bool); 3]";
    /// Verifies a one-element tuple retains its required trailing comma.
    singleton_tuple: (String,) => "(String,)";
    /// Verifies a standard path is shortened to its final segment.
    standard_string_path: std::string::String => "String";
    /// Verifies nested result, vector, and error paths are shortened independently.
    nested_result: Option<Result<Vec<String>, std::io::Error>>
        => "Option<Result<Vec<String>, Error>>";
    /// Verifies map owners and both generic arguments are shortened independently.
    map_with_nested_value: std::collections::HashMap<String, Vec<i32>>
        => "HashMap<String, Vec<i32>>";
    /// Verifies range types retain their semantic generic argument.
    inclusive_range: std::ops::RangeInclusive<usize> => "RangeInclusive<usize>";
    /// Verifies marker types recursively shorten their argument.
    marker_type: std::marker::PhantomData<std::path::PathBuf>
        => "PhantomData<PathBuf>";
    /// Verifies a local const-generic owner preserves both argument kinds.
    local_const_generic: Buffer<std::vec::Vec<u8>, 32> => "Buffer<Vec<u8>, 32>";
    /// Verifies a bare function preserves borrowed argument lifetimes from the compiler.
    borrowed_function: fn(&str) -> String => "fn(&'_ str) -> String";
    /// Verifies an unsafe ABI-qualified function preserves every qualifier.
    unsafe_abi_function: unsafe extern "C" fn(i32) -> i32
        => "unsafe extern \"C\" fn(i32) -> i32";
    /// Verifies nested function types remain parseable and structurally intact.
    higher_order_function: fn(fn(i32) -> i32) -> fn(i32) -> i32
        => "fn(fn(i32) -> i32) -> fn(i32) -> i32";
    /// Verifies a trait object shortens its trait path.
    debug_trait_object: Box<dyn std::fmt::Debug> => "Box<dyn Debug>";
    /// Verifies multiple trait bounds retain their separators and marker traits.
    sendable_error_trait_object: Box<dyn std::error::Error + Send + Sync>
        => "Box<dyn Error + Send + Sync>";
    /// Verifies an associated-type equality remains attached to its shortened trait.
    iterator_associated_type: Box<dyn Iterator<Item = u8>>
        => "Box<dyn Iterator<Item = u8>>";
}
