/// An owner whose similarly named item is a method rather than a field.
struct Owner;

impl Owner {
    /// A method deliberately passed to the field macro.
    fn item(&self) {}
}

/// Requests a method with field syntax.
fn main() {
    let _ = pretty_name::of_field!(Owner::item);
}
