/// Nested fixtures used to require the angle-bracketed qualified-owner grammar.
mod nested {
    /// A generic owner referenced through a qualified path.
    pub struct Owner<T> {
        /// A field referenced through a qualified path.
        pub field: T,
    }

    impl<T> Owner<T> {
        /// A method referenced through a qualified path.
        pub fn method(&self) {}

        /// A generic method referenced through a qualified path.
        pub fn generic<U>(&self) {}
    }

    /// A generic enum referenced through a qualified path.
    pub enum Choice<T> {
        /// A unit variant.
        Unit,
        /// A tuple variant.
        Tuple(T),
        /// A struct variant.
        Struct {
            /// A payload that makes the struct variant non-empty.
            value: T,
        },
    }
}

/// Exercises fields, methods, and variants with qualified generic owners.
fn main() {
    let _ = pretty_name::of_field!(<nested::Owner<u32>>::field);
    let _ = pretty_name::of_method!(<nested::Owner<u32>>::method);
    let _ = pretty_name::of_method!(<nested::Owner<u32>>::generic::<String>);
    let _ = pretty_name::of_variant!(<nested::Choice<u32>>::Unit);
    let _ = pretty_name::of_variant!(<nested::Choice<u32>>::Tuple(..));
    let _ = pretty_name::of_variant!(<nested::Choice<u32>>::Struct { value, .. });
}
