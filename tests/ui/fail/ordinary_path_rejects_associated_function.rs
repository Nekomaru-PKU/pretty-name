/// An owner whose associated function belongs in `nameof_member!`.
struct Owner;

impl Owner {
    /// An associated function deliberately passed to the ordinary-path macro.
    fn function() {}
}

/// Verifies import validation rejects a type-associated path in `nameof!`.
fn main() {
    let _ = pretty_name::nameof!(Owner::function);
}
