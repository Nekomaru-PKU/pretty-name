/// A generic owner used to verify angle-wrapped method owners are accepted.
struct Owner<T>(T);

impl<T> Owner<T> {
    /// A method referenced through a generic owner.
    fn method(&self) {}
}

/// Exercises the explicit owner-boundary syntax for a generic method owner.
fn main() {
    let _ = pretty_name::nameof_member!(<Owner<u32>>::method);
}
