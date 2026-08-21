#![deny(warnings)]
#![allow(dead_code, non_snake_case)]

/// Deprecated fixtures verify macro-generated validation owns its lint policy.
mod deprecated_fixture {
    /// A deprecated generic function referenced through a qualified path.
    #[deprecated(note = "used to exercise macro lint isolation")]
    pub fn OldFunction<T>() {}

    /// A deprecated type used by the type-name macro.
    #[deprecated(note = "used to exercise macro lint isolation")]
    pub struct OldOwner {
        /// A deprecated field used by the field-name macro.
        #[deprecated(note = "used to exercise macro lint isolation")]
        pub OldField: u32,
    }

    #[allow(deprecated, reason = "the implementation defines the deprecated fixture")]
    impl OldOwner {
        /// A deprecated method used by the member-name macro.
        #[deprecated(note = "used to exercise macro lint isolation")]
        pub fn OldMethod(&self) {}
    }
}

/// Exercises every macro category under a downstream deny-all-warnings policy.
fn main() {
    let _ = pretty_name::nameof!(deprecated_fixture::OldFunction::<u32>);
    let _ = pretty_name::nameof_type!(deprecated_fixture::OldOwner);
    let _ = pretty_name::nameof_field!(<deprecated_fixture::OldOwner>::OldField);
    let _ = pretty_name::nameof_member!(<deprecated_fixture::OldOwner>::OldMethod);
}
