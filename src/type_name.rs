use syn::*;

/// Get the human-friendly type name of given type `T`, removing visual clutter such as
/// full module paths.
/// 
/// # Examples
/// ```rust
/// use pretty_name::type_name;
/// assert_eq!(type_name::<Option<i32>>(), "Option<i32>");
/// assert_eq!(type_name::<&str>(), "&str");
/// assert_eq!(type_name::<Vec<Box<dyn std::fmt::Debug>>>(), "Vec<Box<dyn Debug>>");
/// ```
pub fn type_name<T: ?Sized + 'static>() -> &'static str {
    use std::any::TypeId;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::collections::hash_map::Entry;

    thread_local!(
        static TYPE_NAME_CACHE: RefCell<HashMap<TypeId, &'static str>> =
            RefCell::new(HashMap::new()));

    TYPE_NAME_CACHE.with_borrow_mut(|cache| match cache.entry(TypeId::of::<T>()) {
        Entry::Occupied(entry) =>
            *entry.get(),
        Entry::Vacant(entry) =>
            *entry.insert(type_name_internal::<T>()),
    })
}

/// Get the human-friendly type name of the given value, removing visual clutter such as
/// full module paths.
/// 
/// # Examples
/// ```rust
/// use pretty_name::type_name_of_val;
/// let value = vec![1, 2, 3];
/// assert_eq!(type_name_of_val(&value), "Vec<i32>");
/// ```
pub fn type_name_of_val<T: ?Sized + 'static>(_: &T) -> &'static str {
    type_name::<T>()
}

fn type_name_internal<T: ?Sized + 'static>() -> &'static str {
    let type_name = std::any::type_name::<T>();
    let Ok(mut ty) = syn::parse_str::<Type>(type_name) else {
        return "<error>";
    };

    truncate_type(&mut ty);

    // Use rustfmt to get a nicely formatted type string.
    // rustfmt only accepts full source files, so we wrap the type in a dummy function.
    use quote::quote;
    use rust_format::Formatter as _;
    let format_result =
        rust_format::RustFmt::default()
            .format_tokens(quote!(fn main() -> #ty {}))
            .unwrap_or("<error>".to_string());
    let start = const { "fn main() -> ".len() };
    let end = format_result.len() - const { " {}\r\n".len() };
    Box::leak(
        format_result[start..end]
            .to_owned()
            .into_boxed_str())
}

fn truncate_type(ty: &mut Type) {
    match *ty {
        Type::Infer(_) |
        Type::Macro(_) |
        Type::Never(_) |
        Type::Verbatim(_) => {}

        Type::Array(TypeArray { ref mut elem, .. }) |
        Type::Group(TypeGroup { group_token: _, ref mut elem }) |
        Type::Paren(TypeParen { paren_token: _, ref mut elem }) |
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

        Type::BareFn(ref mut ty) => {
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

fn truncate_path(path: &mut Path) {
    let path_mut = path;
    let path = std::mem::replace(
        path_mut,
        Path {
            leading_colon: None,
            segments: Default::default(),
        });

    let Some(mut last_segment) = path.segments.into_iter().last() else {
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
                truncate_type(input);
            }
            if let ReturnType::Type(_, ref mut output) = args.output {
                truncate_type(output);
            }
        }
    }

    path_mut.leading_colon = None;
    path_mut.segments = Some(last_segment).into_iter().collect();
}

#[cfg(test)]
mod test {
    use super::type_name;

    #[test]
    fn test_type_name() {
        // ===== Primitives =====
        assert_eq!(type_name::<i32>(), "i32");
        assert_eq!(type_name::<bool>(), "bool");

        // ===== Unsized Primitives =====
        assert_eq!(type_name::<str>(), "str");
        assert_eq!(type_name::<[i32]>(), "[i32]");

        // ===== References - Immutable =====
        assert_eq!(type_name::<&i32>(), "&i32");
        assert_eq!(type_name::<&str>(), "&str");
        // Lifetime elision - 'static should be removed
        assert_eq!(type_name::<&'static str>(), "&str");
        // Multiple levels of indirection
        assert_eq!(type_name::<&&&str>(), "&&&str");
        // Reference to unsized slice
        assert_eq!(type_name::<&[i32]>(), "&[i32]");

        // ===== References - Mutable =====
        assert_eq!(type_name::<&mut String>(), "&mut String");
        assert_eq!(type_name::<&mut &str>(), "&mut &str");
        assert_eq!(type_name::<&mut str>(), "&mut str");
        assert_eq!(type_name::<&mut [i32]>(), "&mut [i32]");

        // ===== Raw Pointers =====
        assert_eq!(type_name::<*const i32>(), "*const i32");
        assert_eq!(type_name::<*mut i32>(), "*mut i32");
        assert_eq!(type_name::<*const str>(), "*const str");
        assert_eq!(type_name::<*mut [u8]>(), "*mut [u8]");
        // Nested raw pointers
        assert_eq!(type_name::<*const *mut i32>(), "*const *mut i32");
        // Mixed pointer types
        assert_eq!(type_name::<*const &str>(), "*const &str");
        assert_eq!(type_name::<&*const i32>(), "&*const i32");

        // ===== Arrays =====
        assert_eq!(type_name::<[i32; 5]>(), "[i32; 5]");
        assert_eq!(type_name::<[bool; 0]>(), "[bool; 0]");
        assert_eq!(type_name::<&[i32; 3]>(), "&[i32; 3]");
        assert_eq!(type_name::<&mut [i32; 5]>(), "&mut [i32; 5]");
        // Nested arrays
        assert_eq!(type_name::<[[i32; 2]; 3]>(), "[[i32; 2]; 3]");
        assert_eq!(type_name::<[[[u8; 2]; 3]; 4]>(), "[[[u8; 2]; 3]; 4]");
        // Array of tuples
        assert_eq!(type_name::<[(i32, bool); 10]>(), "[(i32, bool); 10]");

        // ===== Tuples =====
        assert_eq!(type_name::<()>(), "()");
        assert_eq!(type_name::<(i32,)>(), "(i32,)");
        assert_eq!(type_name::<(i32, String, bool)>(), "(i32, String, bool)");
        // Nested tuples
        assert_eq!(type_name::<(i32, (String, bool))>(), "(i32, (String, bool))");
        // Tuples with references to unsized
        assert_eq!(type_name::<(&str, &[u8])>(), "(&str, &[u8])");
        assert_eq!(type_name::<(&mut String, &i32)>(), "(&mut String, &i32)");

        // ===== Generic Containers =====
        assert_eq!(type_name::<Option<i32>>(), "Option<i32>");
        assert_eq!(type_name::<Option<&str>>(), "Option<&str>");
        assert_eq!(type_name::<Result<i32, String>>(), "Result<i32, String>");
        assert_eq!(type_name::<Result<(), ()>>(), "Result<(), ()>");
        assert_eq!(type_name::<Vec<i32>>(), "Vec<i32>");
        assert_eq!(type_name::<std::collections::HashMap<String, i32>>(), "HashMap<String, i32>");
        assert_eq!(type_name::<std::collections::BTreeMap<String, i32>>(), "BTreeMap<String, i32>");

        // ===== Function Pointers =====
        assert_eq!(type_name::<fn()>(), "fn()");
        assert_eq!(type_name::<fn(i32) -> i32>(), "fn(i32) -> i32");
        assert_eq!(type_name::<fn(i32, String, bool)>(), "fn(i32, String, bool)");
        assert_eq!(type_name::<fn(&str) -> String>(), "fn(&str) -> String");
        assert_eq!(type_name::<fn(&mut i32)>(), "fn(&mut i32)");
        assert_eq!(type_name::<fn(*const i32) -> *mut i32>(), "fn(*const i32) -> *mut i32");
        // Function returning function
        assert_eq!(type_name::<fn() -> fn(i32) -> i32>(), "fn() -> fn(i32) -> i32");
        assert_eq!(type_name::<fn(fn(i32) -> i32) -> i32>(), "fn(fn(i32) -> i32) -> i32");
        // Unsafe and extern functions
        assert_eq!(type_name::<unsafe fn()>(), "unsafe fn()");
        assert_eq!(type_name::<extern "C" fn(i32) -> i32>(), "extern \"C\" fn(i32) -> i32");
        assert_eq!(type_name::<unsafe extern "C" fn(i32)>(), "unsafe extern \"C\" fn(i32)");

        // ===== Trait Objects =====
        assert_eq!(type_name::<Box<dyn std::fmt::Debug>>(), "Box<dyn Debug>");
        assert_eq!(type_name::<&dyn std::fmt::Display>(), "&dyn Display");
        assert_eq!(type_name::<&mut dyn std::io::Write>(), "&mut dyn Write");
        assert_eq!(type_name::<Box<dyn std::fmt::Debug + Send>>(), "Box<dyn Debug + Send>");
        assert_eq!(type_name::<Box<dyn std::fmt::Debug + Send + Sync>>(), "Box<dyn Debug + Send + Sync>");
        assert_eq!(type_name::<dyn std::fmt::Debug>(), "dyn Debug");
        assert_eq!(type_name::<dyn std::fmt::Debug + Send>(), "dyn Debug + Send");

        // ===== Smart Pointers =====
        assert_eq!(type_name::<Box<i32>>(), "Box<i32>");
        assert_eq!(type_name::<Box<str>>(), "Box<str>");
        assert_eq!(type_name::<Box<[i32]>>(), "Box<[i32]>");
        assert_eq!(type_name::<std::rc::Rc<String>>(), "Rc<String>");
        assert_eq!(type_name::<std::sync::Arc<String>>(), "Arc<String>");
        assert_eq!(type_name::<std::cell::RefCell<i32>>(), "RefCell<i32>");

        // ===== Nested Generic Types =====
        assert_eq!(type_name::<Vec<Vec<String>>>(), "Vec<Vec<String>>");
        assert_eq!(type_name::<Vec<Vec<Vec<i32>>>>(), "Vec<Vec<Vec<i32>>>");
        assert_eq!(type_name::<Option<Result<i32, String>>>(), "Option<Result<i32, String>>");
        assert_eq!(type_name::<Box<Option<Vec<String>>>>(), "Box<Option<Vec<String>>>");
        assert_eq!(type_name::<Option<Box<dyn std::fmt::Debug>>>(), "Option<Box<dyn Debug>>");
        assert_eq!(type_name::<Vec<Option<&str>>>(), "Vec<Option<&str>>");

        // ===== Composite Structures =====
        assert_eq!(type_name::<(Option<i32>, Result<String, ()>)>(), "(Option<i32>, Result<String, ()>)");
        assert_eq!(type_name::<&[(i32, String)]>(), "&[(i32, String)]");
        assert_eq!(type_name::<[(Option<i32>, &str); 5]>(), "[(Option<i32>, &str); 5]");
        assert_eq!(type_name::<std::collections::HashMap<String, Vec<i32>>>(), "HashMap<String, Vec<i32>>");
        assert_eq!(type_name::<&[Option<Result<i32, String>>]>(), "&[Option<Result<i32, String>>]");
        // Function pointers in containers
        assert_eq!(type_name::<Vec<fn(i32) -> i32>>(), "Vec<fn(i32) -> i32>");
        assert_eq!(type_name::<Option<fn() -> String>>(), "Option<fn() -> String>");

        // ===== Path Simplification =====
        assert_eq!(type_name::<std::vec::Vec<i32>>(), "Vec<i32>");
        assert_eq!(type_name::<std::string::String>(), "String");
        assert_eq!(type_name::<std::boxed::Box<i32>>(), "Box<i32>");
        // Nested qualified paths
        assert_eq!(type_name::<Result<Vec<u8>, std::io::Error>>(), "Result<Vec<u8>, Error>");
        assert_eq!(type_name::<std::collections::HashMap<std::string::String, std::vec::Vec<i32>>>(), "HashMap<String, Vec<i32>>");

        // ===== Extreme Nesting & Combinations =====
        assert_eq!(type_name::<Vec<Option<Result<Box<dyn std::fmt::Debug>, String>>>>(), "Vec<Option<Result<Box<dyn Debug>, String>>>");
        assert_eq!(type_name::<&[Option<&[(i32, &str)]>]>(), "&[Option<&[(i32, &str)]>]");
        assert_eq!(type_name::<fn(Vec<&str>) -> Option<Result<i32, Box<dyn std::error::Error>>>>(), "fn(Vec<&str>) -> Option<Result<i32, Box<dyn Error>>>");

        // ===== Edge Cases =====
        assert_eq!(type_name::<[(); 5]>(), "[(); 5]");
        assert_eq!(type_name::<std::marker::PhantomData<i32>>(), "PhantomData<i32>");
        assert_eq!(type_name::<std::marker::PhantomData<&str>>(), "PhantomData<&str>");
    }
}
