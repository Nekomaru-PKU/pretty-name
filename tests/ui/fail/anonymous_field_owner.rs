/// An owner whose reference type deliberately falls outside the named-path grammar.
struct Owner {
    /// A field reachable through Rust's ordinary reference autoderef.
    field: u32,
}

/// Uses an anonymous reference type where the macro requires a named owner path.
fn main() {
    let _ = pretty_name::of_field!(<&Owner>::field);
}
