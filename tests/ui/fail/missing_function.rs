/// Requests a function that does not exist.
fn main() {
    let _ = pretty_name::of_function!(missing_function);
}
