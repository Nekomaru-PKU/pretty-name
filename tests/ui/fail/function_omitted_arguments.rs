/// A generic function that cannot produce a concrete item without its argument.
fn generic<T>() {}

/// Omits a generic function's required type argument.
fn main() {
    let _ = pretty_name::of_function!(generic);
}
