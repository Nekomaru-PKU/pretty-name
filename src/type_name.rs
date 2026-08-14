use std::borrow::Cow;
use std::fmt;

use quote::quote;
use syn::visit_mut::{self, VisitMut};
use syn::*;

/// A compiler-resolved type description with human-readable [`Display`] formatting.
///
/// The stored compiler description remains private and is parsed only when the value is
/// formatted. If the description is unfamiliar to the formatter, it is displayed
/// unchanged instead of being partially shortened.
///
/// # Examples
///
/// ```rust
/// use pretty_name::type_name;
///
/// let name = type_name::<std::collections::HashMap<String, i32>>();
/// assert_eq!(name.to_string(), "HashMap<String, i32>");
/// ```
#[derive(Debug)]
pub struct TypeName(&'static str);

impl fmt::Display for TypeName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(prettify_type_name(self.0).as_ref())
    }
}

/// Gets a diagnostic name for the compiler-resolved type `T`.
///
/// Formatting removes module qualification from parseable Rust type paths while
/// preserving the remaining type structure. Compiler descriptions outside the
/// supported grammar are displayed unchanged.
///
/// # Examples
///
/// ```rust
/// use pretty_name::type_name;
///
/// assert_eq!(type_name::<Option<i32>>().to_string(), "Option<i32>");
/// assert_eq!(type_name::<&str>().to_string(), "&str");
/// assert_eq!(
///     type_name::<Vec<Box<dyn std::fmt::Debug>>>().to_string(),
///     "Vec<Box<dyn Debug>>");
/// ```
pub fn type_name<T: ?Sized>() -> TypeName {
    TypeName(core::any::type_name::<T>())
}

/// Gets a diagnostic name for the compiler-resolved type of `value`.
///
/// Passing a reference inspects the referenced value's type rather than adding another
/// reference layer to the description.
///
/// # Examples
///
/// ```rust
/// use pretty_name::type_name_of_val;
///
/// let value = vec![1, 2, 3];
/// assert_eq!(type_name_of_val(&value).to_string(), "Vec<i32>");
/// assert_eq!(type_name_of_val(&value.as_slice()).to_string(), "&[i32]");
/// ```
pub fn type_name_of_val<T: ?Sized>(value: &T) -> TypeName {
    TypeName(core::any::type_name_of_val(value))
}

/// Produces a shortened type description or borrows the original when transformation
/// cannot be completed confidently.
fn prettify_type_name(type_name: &str) -> Cow<'_, str> {
    let Ok(mut ty) = syn::parse_str::<Type>(type_name) else {
        return Cow::Borrowed(type_name);
    };

    let mut shortener = TypeQualificationShortener::default();
    shortener.visit_type_mut(&mut ty);
    if shortener.encountered_unparsed_syntax {
        return Cow::Borrowed(type_name);
    }

    let Ok(file) = syn::parse2::<File>(quote!(type __PrettyName = #ty;)) else {
        return Cow::Borrowed(type_name);
    };
    let formatted = prettyplease::unparse(&file);
    let Some(pretty_name) = extract_pretty_type(&formatted) else {
        return Cow::Borrowed(type_name);
    };

    Cow::Owned(pretty_name.to_owned())
}

/// Extracts the right-hand type from the private alias used for pretty-printing.
///
/// Whitespace around `=` is deliberately ignored because the pretty-printer may wrap a
/// deeply nested type onto the next line.
fn extract_pretty_type(formatted: &str) -> Option<&str> {
    let (declaration, pretty_name) = formatted.split_once('=')?;
    if declaration.trim() != "type __PrettyName" {
        return None;
    }

    pretty_name.trim().strip_suffix(';')
}

/// Removes module qualification from type and trait paths reached through Syn's type
/// grammar.
#[derive(Default)]
struct TypeQualificationShortener {
    /// Records syntax that Syn preserved without interpreting, requiring unchanged
    /// fallback for the complete compiler description.
    encountered_unparsed_syntax: bool,
}

impl VisitMut for TypeQualificationShortener {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        if matches!(expr, Expr::Verbatim(_)) {
            self.encountered_unparsed_syntax = true;
            return;
        }

        visit_mut::visit_expr_mut(self, expr);
    }

    fn visit_trait_bound_mut(&mut self, bound: &mut TraitBound) {
        visit_mut::visit_trait_bound_mut(self, bound);
        if !retain_final_path_segment(&mut bound.path) {
            self.encountered_unparsed_syntax = true;
        }
    }

    fn visit_type_mut(&mut self, ty: &mut Type) {
        if matches!(ty, Type::Verbatim(_)) {
            self.encountered_unparsed_syntax = true;
            return;
        }

        visit_mut::visit_type_mut(self, ty);
    }

    fn visit_type_param_bound_mut(&mut self, bound: &mut TypeParamBound) {
        if matches!(bound, TypeParamBound::Verbatim(_)) {
            self.encountered_unparsed_syntax = true;
            return;
        }

        visit_mut::visit_type_param_bound_mut(self, bound);
    }

    fn visit_type_path_mut(&mut self, type_path: &mut TypePath) {
        visit_mut::visit_type_path_mut(self, type_path);
        if !shorten_type_path(type_path) {
            self.encountered_unparsed_syntax = true;
        }
    }
}

/// Shortens a type path while retaining the trait and associated segments of a
/// qualified-self projection.
fn shorten_type_path(type_path: &mut TypePath) -> bool {
    let Some(qself) = type_path.qself.as_mut() else {
        return retain_final_path_segment(&mut type_path.path);
    };

    if qself.position == 0 {
        return !type_path.path.segments.is_empty();
    }

    let position = qself.position;
    let mut segments: Vec<_> = std::mem::take(&mut type_path.path.segments)
        .into_iter()
        .collect();
    if position > segments.len() {
        return false;
    }

    let associated_segments = segments.split_off(position);
    let Some(trait_segment) = segments.pop() else {
        return false;
    };

    type_path.path.leading_colon = None;
    type_path.path.segments = std::iter::once(trait_segment)
        .chain(associated_segments)
        .collect();
    qself.position = 1;
    true
}

/// Retains a path's final segment after its nested generic arguments have already been
/// visited.
fn retain_final_path_segment(path: &mut Path) -> bool {
    let Some(last_segment) = std::mem::take(&mut path.segments).into_iter().next_back()
    else {
        return false;
    };

    path.leading_colon = None;
    path.segments = std::iter::once(last_segment).collect();
    true
}

#[cfg(test)]
mod tests {
    use super::TypeName;

    /// Verifies compiler-emitted lifetimes survive qualification removal.
    #[test]
    fn display_preserves_reference_lifetimes() {
        assert_eq!(TypeName("&'named crate::model::Record").to_string(), "&'named Record");
    }

    /// Verifies qualified-self projections retain their owner, trait, and associated
    /// type structure.
    #[test]
    fn display_preserves_qualified_self_projections() {
        assert_eq!(
            TypeName(
                "<crate::model::Record as crate::traits::HasItem>::Item").to_string(),
            "<Record as HasItem>::Item");
    }

    /// Verifies associated bounds are traversed through Syn's generic-argument grammar.
    #[test]
    fn display_shortens_associated_type_constraints() {
        assert_eq!(
            TypeName(
                "dyn crate::traits::Outer<Item: crate::fmt::Display>").to_string(),
            "dyn Outer<Item: Display>");
    }

    /// Verifies descriptions outside Rust's type grammar remain completely unchanged.
    #[test]
    fn display_preserves_unparseable_descriptions() {
        let description = "crate::module::{closure@src/lib.rs:1:1}";

        assert_eq!(TypeName(description).to_string(), description);
    }
}
