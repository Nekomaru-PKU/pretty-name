use fixture::generic as renamed_generic;
use fixture::Owner as RenamedOwner;

/// Fixtures whose source spelling differs from the names used at macro call sites.
mod fixture {
    /// A generic function imported under another name.
    pub fn generic<T>() {}

    /// An owner imported under another name.
    pub struct Owner {
        /// A field referenced through an alias.
        pub field: u32,
    }

    impl Owner {
        /// A method referenced through an alias.
        pub fn method(&self) {}
    }

    /// An enum imported through a type alias.
    pub enum Choice {
        /// A unit variant referenced through an alias.
        Unit,
    }
}

/// A source-level alias whose resolved owner must remain semantically valid.
type OwnerAlias = RenamedOwner;

/// A source-level enum alias used to validate variant lookup.
type ChoiceAlias = fixture::Choice;

/// Exercises aliases and renamed imports accepted by the public macros.
fn main() {
    let value = 42;
    let _ = pretty_name::nameof!(value);
    let _ = pretty_name::nameof!(renamed_generic::<u32>);
    let _ = pretty_name::nameof_type!(OwnerAlias);
    let _ = pretty_name::nameof_field!(OwnerAlias::field);
    let _ = pretty_name::nameof_member!(OwnerAlias::method);
    let _ = pretty_name::nameof_member!(ChoiceAlias::Unit);
}
