/// An enum used to verify malformed variant suffixes reach the public grammar fallback.
enum Choice {
    /// A unit variant whose call-like suffix is deliberately invalid.
    Unit,
}

fn main() {
    let _ = pretty_name::nameof_member!(Choice::Unit());
}
