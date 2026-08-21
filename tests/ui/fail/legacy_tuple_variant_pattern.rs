/// An enum used to verify the former tuple-pattern suffix is rejected.
enum Choice {
    /// A tuple variant that is now named through its bare constructor path.
    Tuple(u32),
}

/// Uses the removed tuple-pattern macro syntax.
fn main() {
    let _ = pretty_name::nameof_member!(Choice::Tuple(..));
}
