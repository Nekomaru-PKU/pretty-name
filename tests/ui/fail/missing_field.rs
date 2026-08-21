/// An owner without the requested field.
struct Owner;

/// Requests a field that does not exist.
fn main() {
    let _ = pretty_name::nameof_field!(Owner::missing_field);
}
