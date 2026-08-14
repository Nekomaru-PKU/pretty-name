/// An owner with a generic method.
struct Owner;

impl Owner {
    /// A generic method that cannot produce a concrete item without its argument.
    fn generic<T>(&self) {}
}

/// Omits a generic method's required type argument.
fn main() {
    let _ = pretty_name::of_method!(Owner::generic);
}
