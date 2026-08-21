/// Nested fixtures used to validate ordinary named owner paths.
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

    /// An owner whose turbofish exercises lifetime, nested type, and const arguments.
    pub struct ComplexOwner<'a, T, const N: usize> {
        /// A field whose owner requires every supported owner argument category.
        pub field: &'a [T; N],
    }

    impl<'a, T, const N: usize> ComplexOwner<'a, T, N> {
        /// A method referenced through a complex generic owner path.
        pub fn method(&self) {}

        /// A generic method that follows a complex generic owner path.
        pub fn generic<U>(&self) {}
    }

    /// A generic enum referenced through a qualified path.
    pub enum Choice<T> {
        /// A unit variant.
        Unit,
        /// A tuple variant.
        Tuple(T),
    }
}

/// Exercises fields, methods, and variants with qualified generic owners.
fn main() {
    let _ = pretty_name::nameof_field!(<nested::Owner<u32>>::field);
    let _ = pretty_name::nameof_member!(<nested::Owner<u32>>::method);
    let _ = pretty_name::nameof_member!(<nested::Owner<u32>>::generic::<String>);
    let _ = pretty_name::nameof_member!(<nested::Choice<u32>>::Unit);
    let _ = pretty_name::nameof_member!(<nested::Choice<u32>>::Tuple);
    let _ = pretty_name::nameof_field!(
        <nested::ComplexOwner<'static, Vec<Vec<u8>>, 4>>::field);
    let _ = pretty_name::nameof_member!(
        <nested::ComplexOwner<'static, Vec<Vec<u8>>, 4>>::method);
    let _ = pretty_name::nameof_member!(
        <nested::ComplexOwner<'static, Vec<Vec<u8>>, 4>>::generic::<Option<String>>);
    let _ = pretty_name::nameof_member!(<nested::Choice<Vec<Vec<u8>>>>::Tuple);
}
