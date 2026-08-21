/// An owner used to verify malformed field suffixes reach the public grammar fallback.
struct Owner {
    /// A field whose call-like suffix is deliberately invalid.
    field: u32,
}

fn main() {
    let _ = pretty_name::nameof_field!(Owner::field());
}
