/// A generic function whose complete type argument list is explicit.
fn generic_pair<T, U>() {}

/// An owner with a generic method whose complete type argument list is explicit.
struct Owner;

impl Owner {
    /// A generic method with two caller-provided type arguments.
    fn generic_pair<T, U>(&self) {}
}

/// Exercises fully explicit generic function and method forms.
fn main() {
    let _ = pretty_name::nameof!(generic_pair::<Vec<u8>, String>);
    let _ = pretty_name::nameof_member!(Owner::generic_pair::<Vec<u8>, String>);
}
