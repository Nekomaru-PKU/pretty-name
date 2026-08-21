/// An owner used to verify malformed method suffixes reach the public grammar fallback.
struct Owner;

impl Owner {
    /// A method whose call-like suffix is deliberately invalid.
    fn method(&self) {}
}

fn main() {
    let _ = pretty_name::of_method!(Owner::method());
}
