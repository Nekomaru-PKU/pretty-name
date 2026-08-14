/// An owner with a generic method.
struct Owner;

impl Owner {
    /// A generic method used to reject inferred macro arguments.
    fn generic<T>(&self) {}
}

/// Supplies `_` where the macro contract requires a concrete type.
fn main() {
    let _ = pretty_name::of_method!(Owner::generic::<_>);
}
