/// An enum whose tuple variant is requested as a struct variant.
enum Choice {
    /// A tuple variant that has no named `value` field.
    Tuple(u32),
}

/// Requests a tuple variant with the named-field struct-variant macro form.
fn main() {
    let _ = pretty_name::of_variant!(Choice::Tuple { value, .. });
}
