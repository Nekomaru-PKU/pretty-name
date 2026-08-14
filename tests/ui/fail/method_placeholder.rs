/// An owner with a generic method.
struct Owner;

impl Owner {
    /// A generic method used to reject the legacy placeholder syntax.
    fn generic<T>(&self) {}
}

/// Supplies the unsupported `::<..>` placeholder.
fn main() {
    let _ = pretty_name::of_method!(Owner::generic::<..>);
}
