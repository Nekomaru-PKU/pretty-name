/// A const-generic function whose argument cannot be represented by a type-name value.
fn const_generic<const LENGTH: usize>() {}

/// Supplies a direct const argument to the type-only macro grammar.
fn main() {
    let _ = pretty_name::of_function!(const_generic::<16>);
}
