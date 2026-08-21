/// A trait whose declaration cannot serve as a resolved `PrettyName` owner.
trait Named {
    /// A method that can instead be named through an implementor or bounded parameter.
    fn method(&self);
}

fn main() {
    let _ = pretty_name::nameof_member!(Named::method);
}
