use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::{LazyLock, RwLock};

use quote::quote;
use syn::*;

/// Process-wide cache from compiler-generated type names to their pretty forms.
///
/// The compiler names are already `&'static str`, and each owned pretty form is leaked
/// once so the public API can retain its historical `&'static str` return type.
static TYPE_NAME_CACHE: LazyLock<RwLock<HashMap<&'static str, &'static str>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Get the human-friendly type name of given type `T`.
///
/// Note that you can also use the `pretty_name::of_type!(T)` macro, which expands to a
/// string literal at compile time if `T` is a simple type identifier, and expands to a
/// call to this function otherwise.
///
/// The returned name is intended for diagnostics rather than persistent identity. Rust
/// does not guarantee the exact format or uniqueness of [`std::any::type_name`]. If a
/// compiler-generated name is not valid Rust type syntax, this function returns that
/// original name instead of replacing it with an opaque error marker.
///
/// # Examples
/// ```rust
/// use pretty_name::type_name;
/// assert_eq!(type_name::<Option<i32>>(), "Option<i32>");
/// assert_eq!(type_name::<&str>(), "&str");
/// assert_eq!(type_name::<Vec<Box<dyn std::fmt::Debug>>>(), "Vec<Box<dyn Debug>>");
/// ```
pub fn type_name<T: ?Sized>() -> &'static str {
    let full_name = std::any::type_name::<T>();
    let cached = TYPE_NAME_CACHE
        .read()
        // CONTEXT: A panic cannot invalidate previously inserted immutable strings.
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get(full_name)
        .copied();
    if let Some(cached) = cached {
        return cached;
    }

    let pretty_name = prettify_type_name(full_name);
    let mut cache = TYPE_NAME_CACHE
        .write()
        // CONTEXT: A panic cannot invalidate previously inserted immutable strings.
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    match cache.entry(full_name) {
        Entry::Occupied(entry) => entry.get(),
        Entry::Vacant(entry) => {
            let pretty_name = match pretty_name {
                Cow::Borrowed(name) => name,
                Cow::Owned(name) => Box::leak(name.into_boxed_str()),
            };
            entry.insert(pretty_name)
        }
    }
}

/// Get the human-friendly type name of the given value.
///
/// Note that even if the value is a reference, you should pass a reference to it to get
/// the correct type name.
///
/// # Examples
/// ```rust
/// use pretty_name::type_name_of_val;
/// let value = vec![1, 2, 3];
/// assert_eq!(type_name_of_val(&value), "Vec<i32>");
/// assert_eq!(type_name_of_val(&value.as_slice()), "&[i32]");
/// ```
pub fn type_name_of_val<T: ?Sized>(_: &T) -> &'static str {
    type_name::<T>()
}

/// Produces a shortened type name or borrows the original when it cannot be parsed.
///
/// The fallback keeps compiler-generated names such as closure descriptions useful and
/// prevents formatter implementation details from becoming runtime failure paths.
fn prettify_type_name(type_name: &'static str) -> Cow<'static, str> {
    let Ok(mut ty) = syn::parse_str::<Type>(type_name) else {
        return Cow::Borrowed(type_name);
    };

    truncate_type(&mut ty);

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
/// Whitespace around `=` is deliberately ignored because pretty-printers may wrap a
/// deeply nested type onto the next line.
fn extract_pretty_type(formatted: &str) -> Option<&str> {
    let (declaration, pretty_name) = formatted.split_once('=')?;
    if declaration.trim() != "type __PrettyName" {
        return None;
    }

    pretty_name.trim().strip_suffix(';')
}

/// Removes lifetimes and module qualification recursively from a parsed type.
fn truncate_type(ty: &mut Type) {
    match *ty {
        Type::Infer(_) |
        Type::Macro(_) |
        Type::Never(_) |
        Type::Verbatim(_) => {}

        Type::Array(TypeArray { ref mut elem, .. }) |
        Type::Group(TypeGroup { ref mut elem, .. }) |
        Type::Paren(TypeParen { ref mut elem, .. }) |
        Type::Ptr(TypePtr { ref mut elem, .. }) |
        Type::Slice(TypeSlice { ref mut elem, .. }) => truncate_type(elem),

        Type::Reference(TypeReference {
            ref mut lifetime,
            ref mut elem,
            ..
        }) => {
            *lifetime = None;
            truncate_type(elem);
        }

        Type::Path(ref mut ty) => truncate_path(&mut ty.path),

        Type::FnPtr(ref mut ty) => {
            for input in ty.inputs.iter_mut() {
                truncate_type(&mut input.ty);
            }

            if let ReturnType::Type(_, ref mut ty) = ty.output {
                truncate_type(ty.as_mut());
            }
        }

        Type::ImplTrait(ref mut ty) => {
            for bound in ty.bounds.iter_mut() {
                if let &mut TypeParamBound::Trait(ref mut trt) = bound {
                    truncate_path(&mut trt.path);
                }
            }
        }

        Type::TraitObject(ref mut ty) => {
            for bound in ty.bounds.iter_mut() {
                if let &mut TypeParamBound::Trait(ref mut trt) = bound {
                    truncate_path(&mut trt.path);
                }
            }
        }

        Type::Tuple(ref mut ty) => {
            for elem in ty.elems.iter_mut() {
                truncate_type(elem);
            }
        }

        _ => { /* non_exhaustive variants */ }
    }
}

/// Retains a path's final segment and shortens types nested in its arguments.
fn truncate_path(path: &mut Path) {
    let path_mut = path;
    let path = std::mem::replace(
        path_mut,
        Path {
            leading_colon: None,
            segments: Default::default(),
        });

    let Some(mut last_segment) = path.segments.into_iter().next_back() else {
        path_mut.leading_colon = None;
        path_mut.segments = Default::default();
        return;
    };

    match last_segment.arguments {
        PathArguments::None => {}
        PathArguments::AngleBracketed(ref mut args) => {
            for arg in args.args.iter_mut() {
                match *arg {
                    GenericArgument::Type(ref mut ty) => truncate_type(ty),
                    GenericArgument::AssocType(ref mut ty) => {
                        truncate_type(&mut ty.ty)
                    }
                    _ => {}
                }
            }
        }
        PathArguments::Parenthesized(ref mut args) => {
            for input in args.inputs.iter_mut() {
                truncate_type(&mut input.ty);
            }
            if let ReturnType::Type(_, ref mut output) = args.output {
                truncate_type(output);
            }
        }
    }

    path_mut.leading_colon = None;
    path_mut.segments = Some(last_segment).into_iter().collect();
}
