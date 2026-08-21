/// A generic owner used to verify associated functions require an explicit boundary.
struct Owner<T>(T);

impl<T> Owner<T> {
    /// An associated function that would be valid with an angle-wrapped owner.
    fn create() -> Self { panic!("the function item is never called") }
}

/// Omits the required wrapper around a generic associated-function owner.
fn main() {
    let _ = pretty_name::of_function!(Owner::<u32>::create);
}
