/// An owner containing the method referenced through an unsupported qualified-self path.
struct Owner;

impl Owner {
    /// A method that would be valid through the direct owner path.
    fn method(&self) {}
}

/// A trait exposing the owner as an associated type.
trait HasOwner {
    /// The associated owner selected by an implementation.
    type Owner;
}

/// Requests a member through qualified-self syntax outside the named-path contract.
fn name<T: HasOwner<Owner = Owner>>() {
    let _ = pretty_name::of_method!(<<T as HasOwner>::Owner>::method);
}

fn main() {}
