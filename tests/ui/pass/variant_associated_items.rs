/// A non-enum owner used to pin the associated-item validation boundary.
struct Owner;

impl Owner {
    /// An associated constant accepted by the same path check as a unit variant.
    const VALUE: Self = Self;

    /// An associated function accepted by the same path check as a tuple constructor.
    fn constructor() -> Self { Self }
}

/// Verifies stable Rust validation intentionally checks resolution, not declaration kind.
fn main() {
    let _ = pretty_name::of_variant!(Owner::VALUE);
    let _ = pretty_name::of_variant!(Owner::constructor);
}
