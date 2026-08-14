/// An owner with a const-generic method.
struct Owner;

impl Owner {
    /// A const-generic method whose argument cannot be represented by a type-name value.
    fn const_generic<const LENGTH: usize>(&self) {}
}

/// Supplies a direct const argument to the type-only macro grammar.
fn main() {
    let _ = pretty_name::of_method!(Owner::const_generic::<16>);
}
