/// A generic owner used to verify generic method owners require an explicit boundary.
struct Owner<T>(T);

impl<T> Owner<T> {
    /// A method that would be valid with an angle-wrapped owner.
    fn method(&self) {}
}

/// Omits the required wrapper around a generic method owner.
fn main() {
    let _ = pretty_name::of_method!(Owner::<u32>::method);
}
