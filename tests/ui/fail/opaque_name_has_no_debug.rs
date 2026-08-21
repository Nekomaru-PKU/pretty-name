/// Verifies an opaque name exposes `Display` without also exposing `Debug`.
fn main() {
    let name = pretty_name::nameof_type!(u32);
    let _ = format!("{name:?}");
}
