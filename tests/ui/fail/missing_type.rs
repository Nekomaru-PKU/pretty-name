/// Requests a type that does not exist.
fn main() {
    let _ = pretty_name::nameof_type!(MissingType);
}
