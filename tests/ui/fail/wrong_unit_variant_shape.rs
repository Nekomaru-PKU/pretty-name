/// An enum whose tuple variant is requested as a unit variant.
enum Choice {
    /// A tuple variant that requires tuple-pattern syntax.
    Tuple(u32),
}

/// Requests a tuple variant with the unit-variant macro form.
fn main() {
    let _ = pretty_name::of_variant!(Choice::Tuple);
}
