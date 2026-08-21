/// Fixtures used to verify qualified variant owners require an explicit boundary.
mod nested {
    /// An enum reached through a module path.
    pub enum Choice {
        /// A variant that would be valid with an angle-wrapped owner.
        Unit,
    }
}

/// Omits the required wrapper around a qualified variant owner.
fn main() {
    let _ = pretty_name::nameof_member!(nested::Choice::Unit);
}
