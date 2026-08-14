/// A generic function used to reject the legacy placeholder syntax.
fn generic<T>() {}

/// Supplies the unsupported `::<..>` placeholder.
fn main() {
    let _ = pretty_name::of_function!(generic::<..>);
}
