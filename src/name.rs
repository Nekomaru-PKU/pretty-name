use std::fmt;

use crate::TypeName;

/// A compiler-validated source identifier.
///
/// Formatting writes the identifier exactly as it appeared at the macro call after
/// ordinary Rust syntax confirmed that it resolves.
///
/// # Examples
///
/// ```rust
/// let local_value = 42;
/// let name: pretty_name::IdentifierName = pretty_name::of_var!(local_value);
///
/// assert_eq!(format!("{name}"), "local_value");
/// assert_eq!(name.to_string(), "local_value");
/// assert!(format!("{name:?}").starts_with("IdentifierName("));
/// ```
#[derive(Debug)]
pub struct IdentifierName(&'static str);

impl fmt::Display for IdentifierName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

/// A validated function identifier with compiler-resolved generic type arguments.
///
/// # Examples
///
/// ```rust
/// fn generic<T>() {}
///
/// let name: pretty_name::FunctionName = pretty_name::of_function!(generic::<u32>);
/// assert_eq!(format!("{name}"), "generic::<u32>");
/// assert_eq!(name.to_string(), "generic::<u32>");
/// assert!(format!("{name:?}").starts_with("FunctionName {"));
/// ```
#[derive(Debug)]
pub struct FunctionName {
    /// The identifier written at the macro call.
    ident: &'static str,
    /// The compiler-resolved generic type arguments in source order.
    args: Box<[TypeName]>,
}

impl fmt::Display for FunctionName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.ident)?;
        write_generic_arguments(&self.args, formatter)
    }
}

/// A validated field, method, or variant identifier with its compiler-resolved owner.
///
/// # Examples
///
/// ```rust
/// struct Owner {
///     field: u32,
/// }
///
/// let name: pretty_name::MemberName = pretty_name::of_field!(Owner::field);
/// assert_eq!(format!("{name}"), "<Owner>::field");
/// assert_eq!(name.to_string(), "<Owner>::field");
/// assert!(format!("{name:?}").starts_with("MemberName {"));
/// ```
#[derive(Debug)]
pub struct MemberName {
    /// The compiler-resolved owner type.
    owner: TypeName,
    /// The identifier written at the macro call.
    ident: &'static str,
    /// The compiler-resolved method type arguments in source order.
    args: Box<[TypeName]>,
}

impl fmt::Display for MemberName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("<")?;
        fmt::Display::fmt(&self.owner, formatter)?;
        formatter.write_str(">::")?;
        formatter.write_str(self.ident)?;
        write_generic_arguments(&self.args, formatter)
    }
}

/// Constructs an identifier value after an exported macro has validated its input.
///
/// This function is re-exported only as hidden macro infrastructure and is not a
/// supported construction API.
pub fn identifier_name(ident: &'static str) -> IdentifierName {
    IdentifierName(ident)
}

/// Constructs a function value from a validated identifier and resolved arguments.
///
/// This function is re-exported only as hidden macro infrastructure and is not a
/// supported construction API.
pub fn function_name(ident: &'static str, args: Box<[TypeName]>) -> FunctionName {
    FunctionName { ident, args }
}

/// Constructs a member value from its resolved owner, validated identifier, and
/// resolved arguments.
///
/// This function is re-exported only as hidden macro infrastructure and is not a
/// supported construction API.
pub fn member_name(
    owner: TypeName,
    ident: &'static str,
    args: Box<[TypeName]>) -> MemberName {
    MemberName { owner, ident, args }
}

/// Writes a non-empty argument list with canonical turbofish punctuation.
///
/// An empty slice deliberately writes nothing so the same representation serves plain
/// functions, fields, variants, and non-generic methods.
fn write_generic_arguments(
    args: &[TypeName],
    formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    let Some((first, remaining)) = args.split_first() else {
        return Ok(());
    };

    formatter.write_str("::<")?;
    fmt::Display::fmt(first, formatter)?;
    for argument in remaining {
        formatter.write_str(", ")?;
        fmt::Display::fmt(argument, formatter)?;
    }
    formatter.write_str(">")
}

#[cfg(test)]
mod tests {
    use super::{function_name, identifier_name, member_name};
    use crate::type_name;

    /// An owner used to verify compound member formatting.
    struct Owner;

    /// Verifies identifier values preserve validated source spelling.
    #[test]
    fn identifier_display_preserves_source_spelling() {
        assert_eq!(identifier_name("local_value").to_string(), "local_value");
    }

    /// Verifies functions without arguments omit turbofish punctuation.
    #[test]
    fn function_display_omits_empty_arguments() {
        assert_eq!(function_name("function", Box::new([])).to_string(), "function");
    }

    /// Verifies a single function argument uses canonical turbofish punctuation.
    #[test]
    fn function_display_formats_one_argument() {
        assert_eq!(
            function_name("function", Box::new([type_name::<u32>()])).to_string(),
            "function::<u32>");
    }

    /// Verifies multiple function arguments use canonical separators without a trailing
    /// comma.
    #[test]
    fn function_display_formats_many_arguments() {
        assert_eq!(
            function_name(
                "function",
                Box::new([type_name::<std::vec::Vec<u8>>(), type_name::<String>()]))
            .to_string(),
            "function::<Vec<u8>, String>");
    }

    /// Verifies fields use the uniform angle-bracketed owner grammar.
    #[test]
    fn member_display_formats_resolved_owner() {
        assert_eq!(
            member_name(type_name::<Owner>(), "field", Box::new([])).to_string(),
            "<Owner>::field");
    }

    /// Verifies method arguments follow the same punctuation as function arguments.
    #[test]
    fn member_display_formats_arguments() {
        assert_eq!(
            member_name(
                type_name::<Owner>(),
                "method",
                Box::new([type_name::<String>()]))
            .to_string(),
            "<Owner>::method::<String>");
    }
}
