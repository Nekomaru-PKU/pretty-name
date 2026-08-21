/// A generated-API-like conversion bound.
trait Param<T> {}

/// A method argument satisfying the conversion bound for one owner parameter.
struct Argument;

impl Param<u32> for Argument {}

/// An owner modeled after generated APIs with a conversion-heavy method.
struct Owner<T>(std::marker::PhantomData<T>);

impl<T> Owner<T> {
    /// A method whose argument type appears only through its conversion bound.
    fn generated<P>(&self, _argument: P)
    where
        P: Param<T>,
    {}
}

/// Requests a method name while leaving its conversion argument type ambiguous.
fn main() {
    let _ = pretty_name::nameof_member!(<Owner<u32>>::generated);
}
