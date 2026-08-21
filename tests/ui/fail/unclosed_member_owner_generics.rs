/// A generic owner used to verify an unterminated angle wrapper is diagnosed.
struct Owner<T> {
    /// A field that would be valid after a closed owner wrapper.
    field: T,
}

fn main() {
    let _ = pretty_name::of_field!(<Owner<u32>::field);
}
