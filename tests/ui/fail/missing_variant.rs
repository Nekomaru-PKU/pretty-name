/// An enum without the requested variant.
enum Choice {
    /// The only valid variant.
    Unit,
}

/// Requests a variant that does not exist.
fn main() {
    let _ = pretty_name::of_variant!(Choice::Missing);
}
