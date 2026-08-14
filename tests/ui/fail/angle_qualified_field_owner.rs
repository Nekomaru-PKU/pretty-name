/// A generic owner used to verify obsolete angle-qualified field syntax is rejected.
struct Owner<T> {
    /// A field that would otherwise be valid for this owner.
    field: T,
}

fn main() {
    let _ = pretty_name::of_field!(<Owner<u32>>::field);
}
