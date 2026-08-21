/// An owner used to verify fields cannot carry method-style generic arguments.
struct Owner {
    /// A field that is deliberately given an invalid turbofish.
    field: u32,
}

fn main() {
    let _ = pretty_name::nameof_field!(Owner::field::<u32>);
}
