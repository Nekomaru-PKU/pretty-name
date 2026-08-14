/// A generic owner used to verify obsolete angle-qualified method syntax is rejected.
struct Owner<T>(T);

impl<T> Owner<T> {
    /// A method that would otherwise be valid for this owner.
    fn method(&self) {}
}

fn main() {
    let _ = pretty_name::of_method!(<Owner<u32>>::method);
}
