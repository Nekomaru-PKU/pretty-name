#![doc = include_str!("../README.md")]

use std::borrow::Cow;

use std::any;
use std::fmt;

use quote::quote;
use syn::*;
use syn::visit_mut::VisitMut;

/// A compiler-validated diagnostic name with human-readable [`Display`](fmt::Display)
/// formatting.
///
/// A name contains either one compiler-resolved type description or a validated source
/// identifier with an optional resolved owner and explicit generic type arguments. Its
/// representation remains private because displayed names are diagnostic presentation,
/// not stable identities or reflection data.
///
/// # Examples
///
/// ```rust
/// use pretty_name::{PrettyName, nameof, nameof_member, nameof_type};
///
/// struct Owner;
/// impl Owner { fn function<T>() {} }
/// let local_value = 42;
///
/// let value: PrettyName = nameof!(local_value);
/// let member: PrettyName = nameof_member!(Owner::function::<u32>);
/// let ty: PrettyName = nameof_type!(std::collections::HashMap<String, i32>);
///
/// assert_eq!(value.to_string(), "local_value");
/// assert_eq!(member.to_string(), "Owner::function<u32>");
/// assert_eq!(ty.to_string(), "HashMap<String, i32>");
/// assert!(format!("{member:?}").starts_with("PrettyName("));
/// ```
pub struct PrettyName(PrettyNameRepresentation);

/// The private semantic components used to format one [`PrettyName`].
enum PrettyNameRepresentation {
    /// One complete compiler-resolved type description.
    Type(&'static str),
    /// A validated identifier and its optional resolved type owner and arguments.
    Item {
        /// The compiler-resolved owner of a member or field.
        owner: Option<&'static str>,
        /// The source identifier captured after compiler validation.
        ident: &'static str,
        /// Compiler-resolved generic type arguments in source order.
        args: Box<[&'static str]>,
    },
}

impl PrettyName {
    /// Constructs an arbitrary resolved-type representation for focused formatter tests.
    #[cfg(test)]
    pub fn from_type_description(description: &'static str) -> Self {
        PrettyName(PrettyNameRepresentation::Type(description))
    }
}

impl fmt::Debug for PrettyName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("PrettyName(")?;
        fmt::Display::fmt(self, formatter)?;
        formatter.write_str(")")
    }
}

impl fmt::Display for PrettyName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            PrettyNameRepresentation::Type(description) => {
                write_type_description(description, formatter)
            }
            PrettyNameRepresentation::Item { owner, ident, args } => {
                if let Some(owner) = owner {
                    write_type_description(owner, formatter)?;
                    formatter.write_str("::")?;
                }
                formatter.write_str(ident)?;
                write_generic_arguments(args, formatter)
            }
        }
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
/// assert_eq!(type_name::<&str>().to_string(), "&str");
/// assert_eq!(type_name::<Option<i32>>().to_string(), "Option<i32>");
/// assert_eq!(type_name::<Vec<Box<dyn std::fmt::Debug>>>().to_string(), "Vec<Box<dyn Debug>>");
/// ```
pub fn type_name<T: ?Sized>() -> PrettyName {
    PrettyName(PrettyNameRepresentation::Type(any::type_name::<T>()))
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
pub fn type_name_of_val<T: ?Sized>(value: &T) -> PrettyName {
    PrettyName(PrettyNameRepresentation::Type(any::type_name_of_val(value)))
}

/// Gets the validated source identifier of an ordinary value path.
///
/// The path may name a binding, constant, static, function, or other value-like item.
/// Generic arguments must be explicit concrete types. The referenced value is resolved
/// inside an uncalled closure and is therefore not evaluated.
///
/// # Examples
/// ```rust
/// let my_variable = 42;
/// const MY_CONSTANT: u32 = 42;
/// fn my_function<T>() {}
/// mod nested { pub fn function() {} }
///
/// assert_eq!(pretty_name::nameof!(my_variable).to_string(), "my_variable");
/// assert_eq!(pretty_name::nameof!(MY_CONSTANT).to_string(), "MY_CONSTANT");
/// assert_eq!(pretty_name::nameof!(my_function::<u32>).to_string(), "my_function<u32>");
/// assert_eq!(pretty_name::nameof!(nested::function).to_string(), "function");
/// ```
///
/// Missing values are rejected at compile time:
///
/// ```compile_fail
/// let _ = pretty_name::nameof!(missing_value);
/// ```
#[macro_export]
macro_rules! nameof {
    (@__qualified [$($prefix:tt)*] $ident:ident) => {{
        let _ = || { let _ = &$($prefix)*$ident; };
        $crate::__item_name(
            ::core::option::Option::None,
            stringify!($ident),
            ::std::boxed::Box::new([]))
    }};
    (@__qualified [$($prefix:tt)*] $ident:ident ::<$($arg:ty),+ $(,)?>) => {{
        let _ = || { let _ = &$($prefix)*$ident::<$($arg),*>; };
        $crate::__item_name(
            ::core::option::Option::None,
            stringify!($ident),
            ::std::boxed::Box::new([$($crate::__resolved_type_name::<$arg>()),*]))
    }};
    (@__qualified [$($prefix:tt)*] $segment:ident :: $($rest:tt)+) => {
        $crate::nameof!(@__qualified [$($prefix)* $segment ::] $($rest)+)
    };
    (@__qualified [$($prefix:tt)*] $($invalid:tt)*) => {
        compile_error!(
            "expected an ordinary value path, optionally followed by explicit type arguments")
    };
    ($ident:ident) => {{
        let _ = || { let _ = &$ident; };
        $crate::__item_name(
            ::core::option::Option::None,
            stringify!($ident),
            ::std::boxed::Box::new([]))
    }};
    ($ident:ident ::<$($arg:ty),+ $(,)?>) => {{
        let _ = || { let _ = &$ident::<$($arg),*>; };
        $crate::__item_name(
            ::core::option::Option::None,
            stringify!($ident),
            ::std::boxed::Box::new([$($crate::__resolved_type_name::<$arg>()),*]))
    }};
    ($head:ident :: $($rest:tt)+) => {
        $crate::nameof!(@__qualified [$head ::] $($rest)+)
    };
    (:: $head:ident $($rest:tt)*) => {
        $crate::nameof!(@__qualified [::] $head $($rest)*)
    };
    ($($invalid:tt)*) => {
        compile_error!(
            "expected an ordinary value path, optionally followed by explicit type arguments")
    };
}

/// Gets a diagnostic name for the compiler-resolved type.
///
/// This macro resolves aliases, renamed imports, generic parameters, and `Self` to the
/// underlying type selected by the compiler.
///
/// # Examples
/// ```rust
/// struct MyStruct;
/// struct MyGenericStruct<T>(std::marker::PhantomData<T>);
/// assert_eq!(pretty_name::nameof_type!(MyStruct).to_string(), "MyStruct");
/// assert_eq!(
///     pretty_name::nameof_type!(MyGenericStruct<u32>).to_string(),
///     "MyGenericStruct<u32>");
/// ```
///
/// Invalid types are rejected at compile time:
///
/// ```compile_fail
/// let _ = pretty_name::nameof_type!(DefinitelyNotAType);
/// ```
#[macro_export]
macro_rules! nameof_type {
    ($ty:ty) => {{
        $crate::type_name::<$ty>()
    }};
}

/// Gets a validated field name with its compiler-resolved owner.
///
/// This macro resolves `Self` to the appropriate type when used inside an `impl` block.
///
/// A single-identifier owner uses `Type::field`. Qualified or generic owners must be
/// wrapped in angle brackets, as in `<module::Type<T>>::field`. The wrapper makes the
/// owner boundary explicit while preserving ordinary Rust syntax and name resolution.
///
/// # Examples
/// ```rust
/// struct MyStruct {
///     my_field: u32,
/// }
/// struct MyGenericStruct<T> {
///     my_field: T,
/// }
/// assert_eq!(
///     pretty_name::nameof_field!(MyStruct::my_field).to_string(),
///     "MyStruct::my_field");
/// assert_eq!(
///     pretty_name::nameof_field!(<MyGenericStruct<u32>>::my_field).to_string(),
///     "MyGenericStruct<u32>::my_field");
/// ```
///
/// Missing fields are rejected at compile time:
///
/// ```compile_fail
/// struct MyStruct;
/// let _ = pretty_name::nameof_field!(MyStruct::missing_field);
/// ```
#[macro_export]
macro_rules! nameof_field {
    ($owner:ident :: $field:ident) => {
        $crate::nameof_field!(<$owner>::$field)
    };
    (<$owner:path> :: $field:ident) => {{
        let _ = |obj: $owner| { let _ = &obj.$field; };
        $crate::__item_name(
            ::core::option::Option::Some($crate::__resolved_type_name::<$owner>()),
            stringify!($field),
            ::std::boxed::Box::new([]))
    }};
    ($($invalid:tt)*) => {
        compile_error!(
            "expected `Owner::field` or `<path::to::Owner<Args>>::field`")
    };
}

/// Gets a validated value-like member name with its compiler-resolved owner.
///
/// Members include associated constants, associated functions, methods, and unit or
/// tuple enum variants. The member is resolved inside an uncalled closure and is
/// therefore not evaluated or invoked.
///
/// A single-identifier owner uses `Type::member`. Qualified or generic owners must be
/// wrapped in angle brackets, as in `<module::Type<T>>::member`. The wrapper identifies
/// the owner in macro input but is omitted from the displayed name.
///
/// A generic member must specify every caller-provided generic argument, and every
/// argument must be a concrete type. Inferred arguments, direct const arguments, and
/// the legacy `::<..>` placeholder are intentionally unsupported.
///
/// # Examples
/// ```rust
/// struct Owner;
/// impl Owner {
///     const CONSTANT: u32 = 42;
///     fn function() {}
///     fn method<T>(&self) {}
/// }
/// enum Choice {
///     Unit,
///     Tuple(u32),
/// }
/// assert_eq!(
///     pretty_name::nameof_member!(Owner::CONSTANT).to_string(),
///     "Owner::CONSTANT");
/// assert_eq!(
///     pretty_name::nameof_member!(Owner::function).to_string(),
///     "Owner::function");
/// assert_eq!(
///     pretty_name::nameof_member!(Owner::method::<u32>).to_string(),
///     "Owner::method<u32>");
/// assert_eq!(
///     pretty_name::nameof_member!(Choice::Tuple).to_string(),
///     "Choice::Tuple");
/// ```
///
/// Struct variants are rejected because their paths are not first-class values:
///
/// ```compile_fail
/// enum Choice { Struct { value: u32 } }
/// let _ = pretty_name::nameof_member!(Choice::Struct);
/// ```
#[macro_export]
macro_rules! nameof_member {
    ($owner:ident :: $member:ident) => {
        $crate::nameof_member!(<$owner>::$member)
    };
    ($owner:ident :: $member:ident ::<$($arg:ty),+ $(,)?>) => {
        $crate::nameof_member!(<$owner>::$member::<$($arg),*>)
    };
    (<$owner:path> :: $member:ident) => {{
        let _ = || <$owner>::$member;
        $crate::__item_name(
            ::core::option::Option::Some($crate::__resolved_type_name::<$owner>()),
            stringify!($member),
            ::std::boxed::Box::new([]))
    }};
    (<$owner:path> :: $member:ident ::<$($arg:ty),+ $(,)?>) => {{
        let _ = || <$owner>::$member::<$($arg),*>;
        $crate::__item_name(
            ::core::option::Option::Some($crate::__resolved_type_name::<$owner>()),
            stringify!($member),
            ::std::boxed::Box::new([$($crate::__resolved_type_name::<$arg>()),*]))
    }};
    ($($invalid:tt)*) => {
        compile_error!(
            "expected `Owner::member` or `<path::to::Owner<Args>>::member`, optionally followed by explicit type arguments")
    };
}

/// Returns the compiler description used to format a resolved type component.
#[doc(hidden)]
pub fn __resolved_type_name<T: ?Sized>() -> &'static str {
    any::type_name::<T>()
}

/// Constructs a name after an exported macro has validated its source path.
///
/// This function is public only so exported macros can use it from downstream crates;
/// it is not a supported construction API.
#[doc(hidden)]
pub fn __item_name(
    owner: Option<&'static str>,
    ident: &'static str,
    args: Box<[&'static str]>) -> PrettyName {
    PrettyName(PrettyNameRepresentation::Item { owner, ident, args })
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
    args: &[&str],
    formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    let Some((first, remaining)) = args.split_first() else {
        return Ok(());
    };

    formatter.write_str("<")?;
    write_type_description(first, formatter)?;
    for argument in remaining {
        formatter.write_str(", ")?;
        write_type_description(argument, formatter)?;
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

#[cfg(test)]
mod tests;
