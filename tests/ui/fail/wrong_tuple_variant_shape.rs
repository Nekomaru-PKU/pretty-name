/// An enum whose struct variant is requested as a tuple variant.
enum Choice {
    /// A struct variant that requires a named-field pattern.
    Struct {
        /// A field that makes the requested shape unambiguous.
        value: u32,
    },
}

/// Requests a struct variant with the tuple-variant macro form.
fn main() {
    let _ = pretty_name::of_variant!(Choice::Struct(..));
}
