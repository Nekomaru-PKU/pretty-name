/// A generic function used to reject inferred macro arguments.
fn generic<T>() {}

/// Supplies `_` where the macro contract requires a concrete type.
fn main() {
    let _ = pretty_name::nameof!(generic::<_>);
}
