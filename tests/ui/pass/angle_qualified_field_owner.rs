/// A generic owner used to verify angle-wrapped field owners are accepted.
struct Owner<T> {
    /// A field referenced through a generic owner.
    field: T,
}

/// Exercises the explicit owner-boundary syntax for a generic field owner.
fn main() {
    let _ = pretty_name::nameof_field!(<Owner<u32>>::field);
}
