/// An enum used to verify struct variants are not first-class constructor values.
enum Choice {
    /// A struct variant that cannot be named through a bare value path.
    Struct {
        /// A payload prevents the variant from behaving like a unit value.
        value: u32,
    },
}

/// Requests an intentionally unsupported struct variant.
fn main() {
    let _ = pretty_name::of_variant!(Choice::Struct);
}
