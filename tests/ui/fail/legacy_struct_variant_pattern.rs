/// An enum used to verify the former struct-pattern suffix is rejected.
enum Choice {
    /// A struct variant that remains unsupported by the naming macro.
    Struct {
        /// A payload gives the removed syntax a real field name.
        value: u32,
    },
}

/// Uses the removed struct-pattern macro syntax.
fn main() {
    let _ = pretty_name::nameof_member!(Choice::Struct { value, .. });
}
