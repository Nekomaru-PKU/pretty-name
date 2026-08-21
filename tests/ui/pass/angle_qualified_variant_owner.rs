/// A generic enum used to verify angle-wrapped variant owners are accepted.
enum Choice<T> {
    /// A unit variant referenced through a generic owner.
    Unit,
    /// A payload keeps the generic parameter semantically relevant.
    Value(T),
}

/// Exercises the explicit owner-boundary syntax for a generic variant owner.
fn main() {
    let _ = pretty_name::nameof_member!(<Choice::<u32>>::Unit);
    let _ = pretty_name::nameof_member!(<Choice<u32>>::Value);
}
