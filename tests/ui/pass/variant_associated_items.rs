/// A non-enum owner used to exercise supported member categories.
struct Owner;

impl Owner {
    /// An associated constant accepted by the member macro.
    const VALUE: Self = Self;

    /// An associated function accepted by the member macro.
    fn constructor() -> Self { Self }
}

/// Verifies associated constants and functions are both first-class members.
fn main() {
    let _ = pretty_name::nameof_member!(Owner::VALUE);
    let _ = pretty_name::nameof_member!(Owner::constructor);
}
