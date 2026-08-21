/// Fixtures used to verify qualified field owners require an explicit boundary.
mod nested {
    /// An owner reached through a module path.
    pub struct Owner {
        /// A field that would be valid with an angle-wrapped owner.
        pub field: u32,
    }
}

/// Omits the required wrapper around a qualified field owner.
fn main() {
    let _ = pretty_name::nameof_field!(nested::Owner::field);
}
