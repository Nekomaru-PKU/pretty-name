/// Requests a function that does not exist.
fn main() {
    let _ = pretty_name::nameof!(missing_function);
}
