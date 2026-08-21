use std::borrow::Cow;

use std::fmt;

use quote::quote;
use syn::*;
use syn::visit_mut::VisitMut;

pub struct TypeName(pub &'static str);

impl fmt::Display for TypeName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_type_description(self.0, formatter)
    }
}

pub struct ItemName<const N: usize> {
    /// The compiler-resolved owner of a member or field.
    pub owner: Option<&'static str>,
    /// The lexical source path captured after compiler validation.
    pub path: &'static str,
    /// Compiler-resolved generic type arguments in source order.
    pub args: [TypeName; N],
}

impl<const N: usize> fmt::Display for ItemName<N> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(owner) = self.owner {
            write_type_description(owner, formatter)?;
            formatter.write_str("::")?;
        }
        formatter.write_str(self.path)?;
        write_generic_arguments(&self.args, formatter)
    }
}

/// Writes one compiler-resolved type description through the structural formatter.
fn write_type_description(
    description: &str,
    formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.write_str(prettify_type_name(description).as_ref())
}

/// Writes a non-empty angle-bracketed argument list.
///
/// An empty slice deliberately writes nothing so the same representation serves plain
/// values, fields, and non-generic members.
fn write_generic_arguments(
    args: &[TypeName],
    formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    let Some((first, remaining)) = args.split_first() else {
        return Ok(());
    };

    formatter.write_str("<")?;
    fmt::Display::fmt(&first, formatter)?;
    for argument in remaining {
        formatter.write_str(", ")?;
        fmt::Display::fmt(&argument, formatter)?;
    }
    formatter.write_str(">")
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
        if matches!(expr, Expr::Macro(_) | Expr::Verbatim(_)) {
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
        if matches!(ty, Type::Macro(_) | Type::Verbatim(_)) {
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
