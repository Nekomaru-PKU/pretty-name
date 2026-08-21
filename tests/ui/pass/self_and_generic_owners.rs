/// A generic owner used to validate `Self` and generic owner resolution.
struct Owner<T> {
    /// A field referenced through `Self`.
    field: T,
}

impl<T> Owner<T> {
    /// A method referenced through `Self`.
    fn method(&self) {}

    /// A generic method referenced through `Self` with an explicit argument.
    fn generic<U>(&self) {}

    /// Exercises the supported `Self` macro forms inside a generic implementation.
    fn names() {
        let _ = pretty_name::of_type!(Self);
        let _ = pretty_name::of_field!(Self::field);
        let _ = pretty_name::of_method!(Self::method);
        let _ = pretty_name::of_method!(Self::generic::<u32>);
    }
}

/// A trait used to validate method lookup through a bounded type parameter.
trait Named {
    /// A trait-provided method resolved through its implementing owner type.
    fn trait_method(&self);
}

impl<T> Named for Owner<T> {
    fn trait_method(&self) {}
}

/// Exercises a trait-provided method through a resolved generic owner.
fn bounded_owner_name<T: Named>() {
    let _ = pretty_name::of_method!(T::trait_method);
}

/// A generic enum used to validate both supported `Self` variant categories.
enum Choice<T> {
    /// A unit variant referenced through `Self`.
    Unit,
    /// A tuple variant referenced through `Self`.
    Tuple(T),
}

impl<T> Choice<T> {
    /// Exercises unit and tuple variant constructors through `Self`.
    fn names() {
        let _ = pretty_name::of_variant!(Self::Unit);
        let _ = pretty_name::of_variant!(Self::Tuple);
    }
}

/// Instantiates the generic fixtures so their `Self` forms are type-checked.
fn main() {
    Owner::<u32>::names();
    Choice::<u32>::names();
    bounded_owner_name::<Owner<u32>>();
}
