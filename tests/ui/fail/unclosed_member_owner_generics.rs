/// A generic owner used to verify an unterminated owner turbofish is diagnosed.
struct Owner<T> {
    /// A field that would be valid after a closed owner turbofish.
    field: T,
}

fn main() {
    let _ = pretty_name::of_field!(Owner::<u32);
}
