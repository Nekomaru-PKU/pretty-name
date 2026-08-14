/// Requests a source identifier that does not exist.
fn main() {
    let _ = pretty_name::of_var!(missing_variable);
}
