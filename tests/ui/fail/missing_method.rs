/// An owner without the requested method.
struct Owner;

/// Requests a method that does not exist.
fn main() {
    let _ = pretty_name::of_method!(Owner::missing_method);
}
