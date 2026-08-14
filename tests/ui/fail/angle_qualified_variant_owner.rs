/// A generic enum used to verify obsolete angle-qualified variant syntax is rejected.
enum Choice<T> {
    /// A unit variant that would otherwise be valid for this owner.
    Unit,
    /// A payload keeps the generic parameter semantically relevant.
    Value(T),
}

fn main() {
    let _ = pretty_name::of_variant!(<Choice<u32>>::Unit);
}
