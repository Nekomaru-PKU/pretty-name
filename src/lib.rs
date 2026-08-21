#![doc = include_str!("../README.md")]

use std::borrow::Cow;
use std::fmt;

use quote::quote;
use syn::visit_mut::{self, VisitMut};
use syn::*;

/// A compiler-resolved type description with human-readable [`Display`](fmt::Display)
/// formatting.
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
/// assert_eq!(format!("{name}"), "HashMap<String, i32>");
/// assert!(format!("{name:?}").starts_with("TypeName("));
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
/// assert_eq!(format!("{name}"), "generic<u32>");
/// assert_eq!(name.to_string(), "generic<u32>");
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

/// A compiler-validated member identifier with its compiler-resolved owner.
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
        if !self.args.is_empty() {
            formatter.write_str("::")?;
        }
        write_generic_arguments(&self.args, formatter)
    }
}

/// Gets the validated source identifier of a local variable or constant.
///
/// This macro checks that the identifier is valid in the current scope. If the identifier
/// is renamed via refactoring tools, the macro call will be updated accordingly.
///
/// # Examples
/// ```rust
/// let my_variable = 42;
/// const MY_CONSTANT: u32 = 42;
/// assert_eq!(pretty_name::of_var!(my_variable).to_string(), "my_variable");
/// assert_eq!(pretty_name::of_var!(MY_CONSTANT).to_string(), "MY_CONSTANT");
/// ```
///
/// Missing identifiers are rejected at compile time:
///
/// ```compile_fail
/// let _ = pretty_name::of_var!(missing_variable);
/// ```
#[macro_export]
macro_rules! of_var {
    ($ident:ident) => {{
        let _ = &$ident;
        $crate::__identifier_name(stringify!($ident))
    }};
}

/// Gets a validated function name with compiler-resolved generic type arguments.
///
/// Module-qualified functions use their ordinary Rust path. Associated functions on a
/// qualified or generic type use an angle-wrapped owner, such as
/// `<module::Owner<T>>::function`. Generic functions must specify every caller-provided
/// generic argument, and every argument must be a concrete type. Inferred arguments,
/// direct const arguments, and the legacy `::<..>` placeholder are intentionally
/// unsupported.
///
/// # Examples
/// ```rust
/// fn my_function() {}
/// fn my_generic_function<T>() {}
/// fn my_generic_function_2args<T, U>() {}
/// assert_eq!(pretty_name::of_function!(my_function).to_string(), "my_function");
/// assert_eq!(
///     pretty_name::of_function!(my_generic_function::<u32>).to_string(),
///     "my_generic_function<u32>");
/// assert_eq!(
///     pretty_name::of_function!(my_generic_function_2args::<u32, String>).to_string(),
///     "my_generic_function_2args<u32, String>");
/// ```
///
/// A generic function cannot omit its type arguments:
///
/// ```compile_fail
/// fn generic<T>() {}
/// let _ = pretty_name::of_function!(generic);
/// ```
///
/// The legacy placeholder is rejected:
///
/// ```compile_fail
/// fn generic<T>() {}
/// let _ = pretty_name::of_function!(generic::<..>);
/// ```
///
/// Inferred type arguments are rejected:
///
/// ```compile_fail
/// fn generic<T>() {}
/// let _ = pretty_name::of_function!(generic::<_>);
/// ```
///
/// Direct const generic arguments are rejected:
///
/// ```compile_fail
/// fn const_generic<const N: usize>() {}
/// let _ = pretty_name::of_function!(const_generic::<16>);
/// ```
///
/// Missing functions are rejected:
///
/// ```compile_fail
/// let _ = pretty_name::of_function!(missing_function);
/// ```
#[macro_export]
macro_rules! of_function {
    ($ident:ident) => {{
        let _ = &$ident;
        $crate::__function_name(
            stringify!($ident),
            ::std::boxed::Box::new([]))
    }};
    ($ident:ident ::<$($arg:ty),+ $(,)?>) => {{
        let _ = &$ident::<$($arg),*>;
        $crate::__function_name(
            stringify!($ident),
            ::std::boxed::Box::new([$($crate::type_name::<$arg>()),*]))
    }};
    (<$owner:path> :: $ident:ident) => {{
        let _ = &<$owner>::$ident;
        $crate::__function_name(
            stringify!($ident),
            ::std::boxed::Box::new([]))
    }};
    (<$owner:path> :: $ident:ident ::<$($arg:ty),+ $(,)?>) => {{
        let _ = &<$owner>::$ident::<$($arg),*>;
        $crate::__function_name(
            stringify!($ident),
            ::std::boxed::Box::new([$($crate::type_name::<$arg>()),*]))
    }};
    ($head:ident :: $($rest:tt)+) => {
        $crate::__qualified_function_name!([$head ::] $($rest)+)
    };
    (:: $head:ident $($rest:tt)*) => {
        $crate::__qualified_function_name!([::] $head $($rest)*)
    };
    ($($invalid:tt)*) => {
        compile_error!(
            "expected a function path, optionally followed by explicit type arguments")
    };
}

/// Finds the final identifier in an ordinary module-qualified function path.
///
/// This implementation-only macro walks forward through `identifier::` segments. A
/// qualified or generic associated owner is deliberately handled by the angle-wrapped
/// public form so this helper never needs to balance or reverse-match generic tokens.
#[doc(hidden)]
#[macro_export]
macro_rules! __qualified_function_name {
    ([$($prefix:tt)*] $ident:ident) => {{
        let _ = &$($prefix)*$ident;
        $crate::__function_name(
            stringify!($ident),
            ::std::boxed::Box::new([]))
    }};
    ([$($prefix:tt)*] $ident:ident ::<$($arg:ty),+ $(,)?>) => {{
        let _ = &$($prefix)*$ident::<$($arg),*>;
        $crate::__function_name(
            stringify!($ident),
            ::std::boxed::Box::new([$($crate::type_name::<$arg>()),*]))
    }};
    ([$($prefix:tt)*] $segment:ident :: $($rest:tt)+) => {
        $crate::__qualified_function_name!([$($prefix)* $segment ::] $($rest)+)
    };
    ([$($prefix:tt)*] $($invalid:tt)*) => {
        compile_error!(
            "expected a module-qualified function path with explicit type arguments, or an angle-wrapped associated owner")
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
/// assert_eq!(pretty_name::of_type!(MyStruct).to_string(), "MyStruct");
/// assert_eq!(
///     pretty_name::of_type!(MyGenericStruct<u32>).to_string(),
///     "MyGenericStruct<u32>");
/// ```
///
/// Invalid types are rejected at compile time:
///
/// ```compile_fail
/// let _ = pretty_name::of_type!(DefinitelyNotAType);
/// ```
#[macro_export]
macro_rules! of_type {
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
///     pretty_name::of_field!(MyStruct::my_field).to_string(),
///     "<MyStruct>::my_field");
/// assert_eq!(
///     pretty_name::of_field!(<MyGenericStruct<u32>>::my_field).to_string(),
///     "<MyGenericStruct<u32>>::my_field");
/// ```
///
/// Missing fields are rejected at compile time:
///
/// ```compile_fail
/// struct MyStruct;
/// let _ = pretty_name::of_field!(MyStruct::missing_field);
/// ```
#[macro_export]
macro_rules! of_field {
    ($owner:ident :: $field:ident) => {
        $crate::of_field!(<$owner>::$field)
    };
    (<$owner:path> :: $field:ident) => {{
        let _ = |obj: $owner| { let _ = &obj.$field; };
        $crate::__member_name(
            $crate::type_name::<$owner>(),
            stringify!($field),
            ::std::boxed::Box::new([]))
    }};
    ($($invalid:tt)*) => {
        compile_error!(
            "expected `Owner::field` or `<qualified::Owner<Args>>::field`")
    };
}

/// Gets a validated method name with its compiler-resolved owner and type arguments.
///
/// This macro resolves `Self` to the appropriate type when used inside an `impl` block.
///
/// A single-identifier owner uses `Type::method`. Qualified or generic owners must be
/// wrapped in angle brackets, as in `<module::Type<T>>::method`.
///
/// A generic method must specify every caller-provided generic argument, and every
/// argument must be a concrete type. Inferred arguments, direct const arguments, and
/// the legacy `::<..>` placeholder are intentionally unsupported.
///
/// # Examples
/// ```rust
/// struct MyStruct;
/// impl MyStruct {
///     fn my_method(&self) {}
///     fn my_generic_method<T>(&self) {}
/// }
/// struct MyGenericStruct<T>(std::marker::PhantomData<T>);
/// impl<T> MyGenericStruct<T> {
///     fn my_method(&self) {}
///     fn my_generic_method<U>(&self) {}
/// }
/// assert_eq!(
///     pretty_name::of_method!(MyStruct::my_method).to_string(),
///     "<MyStruct>::my_method");
/// assert_eq!(
///     pretty_name::of_method!(MyStruct::my_generic_method::<u32>).to_string(),
///     "<MyStruct>::my_generic_method::<u32>");
/// assert_eq!(
///     pretty_name::of_method!(<MyGenericStruct<u32>>::my_method).to_string(),
///     "<MyGenericStruct<u32>>::my_method");
/// assert_eq!(
///     pretty_name::of_method!(
///         <MyGenericStruct<u32>>::my_generic_method::<String>).to_string(),
///     "<MyGenericStruct<u32>>::my_generic_method::<String>");
/// ```
///
/// A generic method cannot omit its type arguments:
///
/// ```compile_fail
/// struct Owner;
/// impl Owner { fn generic<T>(&self) {} }
/// let _ = pretty_name::of_method!(Owner::generic);
/// ```
///
/// The legacy placeholder is rejected:
///
/// ```compile_fail
/// struct Owner;
/// impl Owner { fn generic<T>(&self) {} }
/// let _ = pretty_name::of_method!(Owner::generic::<..>);
/// ```
///
/// Inferred type arguments are rejected:
///
/// ```compile_fail
/// struct Owner;
/// impl Owner { fn generic<T>(&self) {} }
/// let _ = pretty_name::of_method!(Owner::generic::<_>);
/// ```
///
/// Direct const generic arguments are rejected:
///
/// ```compile_fail
/// struct Owner;
/// impl Owner { fn generic<const N: usize>(&self) {} }
/// let _ = pretty_name::of_method!(Owner::generic::<16>);
/// ```
///
/// Missing methods are rejected:
///
/// ```compile_fail
/// struct Owner;
/// let _ = pretty_name::of_method!(Owner::missing_method);
/// ```
#[macro_export]
macro_rules! of_method {
    ($owner:ident :: $method:ident) => {
        $crate::of_method!(<$owner>::$method)
    };
    ($owner:ident :: $method:ident ::<$($arg:ty),+ $(,)?>) => {
        $crate::of_method!(<$owner>::$method::<$($arg),*>)
    };
    (<$owner:path> :: $method:ident) => {{
        let _ = &<$owner>::$method;
        $crate::__member_name(
            $crate::type_name::<$owner>(),
            stringify!($method),
            ::std::boxed::Box::new([]))
    }};
    (<$owner:path> :: $method:ident ::<$($arg:ty),+ $(,)?>) => {{
        let _ = &<$owner>::$method::<$($arg),*>;
        $crate::__member_name(
            $crate::type_name::<$owner>(),
            stringify!($method),
            ::std::boxed::Box::new([$($crate::type_name::<$arg>()),*]))
    }};
    ($($invalid:tt)*) => {
        compile_error!(
            "expected `Owner::method` or `<qualified::Owner<Args>>::method`, optionally followed by explicit type arguments")
    };
}

/// Gets a validated enum variant name with its compiler-resolved owner.
///
/// This macro resolves `Self` to the appropriate type when used inside an `impl` block.
///
/// This macro supports unit and tuple variants through the same bare variant path.
/// Unit variants are values and tuple variants are function-like constructors, so
/// ordinary associated-item resolution validates both without pattern syntax. Struct
/// variants are intentionally unsupported because they are not first-class values.
///
/// A single-identifier owner uses `Enum::Variant`. Qualified or generic owners must be
/// wrapped in angle brackets, as in `<module::Enum<T>>::Variant`.
///
/// # Examples
/// ```rust
/// enum MyEnum {
///     UnitVariant,
///     TupleVariant(u32, String),
/// }
/// enum MyGenericEnum<T> {
///     UnitVariant,
///     Value(T),
/// }
/// assert_eq!(
///     pretty_name::of_variant!(MyEnum::UnitVariant).to_string(),
///     "<MyEnum>::UnitVariant");
/// assert_eq!(
///     pretty_name::of_variant!(MyEnum::TupleVariant).to_string(),
///     "<MyEnum>::TupleVariant");
/// assert_eq!(
///     pretty_name::of_variant!(<MyGenericEnum<u32>>::UnitVariant).to_string(),
///     "<MyGenericEnum<u32>>::UnitVariant");
/// ```
///
/// Struct variants are rejected because their path is not a value or tuple
/// constructor:
///
/// ```compile_fail
/// enum MyEnum { Struct { value: u32 } }
/// let _ = pretty_name::of_variant!(MyEnum::Struct);
/// ```
///
/// The associated item must resolve, but stable Rust does not expose whether that item
/// was declared specifically as a variant. An associated constant or function with
/// the requested name therefore also satisfies this validation.
///
/// Missing associated items are rejected:
///
/// ```compile_fail
/// enum MyEnum { Unit }
/// let _ = pretty_name::of_variant!(MyEnum::Missing);
/// ```
#[macro_export]
macro_rules! of_variant {
    ($owner:ident :: $variant:ident) => {
        $crate::of_variant!(<$owner>::$variant)
    };
    (<$owner:path> :: $variant:ident) => {{
        // Type-check the item without constructing a unit variant whose `Drop`
        // implementation could otherwise run as a naming side effect.
        let _ = || <$owner>::$variant;
        $crate::__member_name(
            $crate::type_name::<$owner>(),
            stringify!($variant),
            ::std::boxed::Box::new([]))
    }};
    ($($invalid:tt)*) => {
        compile_error!(
            "expected `Owner::Variant` or `<qualified::Owner<Args>>::Variant`")
    };
}

/// Constructs an identifier value after an exported macro has validated its input.
///
/// This function is public only so exported macros can use it from downstream crates;
/// it is not a supported construction API.
#[doc(hidden)]
pub fn __identifier_name(ident: &'static str) -> IdentifierName {
    IdentifierName(ident)
}

/// Constructs a function value from a validated identifier and resolved arguments.
///
/// This function is public only so exported macros can use it from downstream crates;
/// it is not a supported construction API.
#[doc(hidden)]
pub fn __function_name(ident: &'static str, args: Box<[TypeName]>) -> FunctionName {
    FunctionName { ident, args }
}

/// Constructs a member value from its resolved owner, validated identifier, and
/// resolved arguments.
///
/// This function is public only so exported macros can use it from downstream crates;
/// it is not a supported construction API.
#[doc(hidden)]
pub fn __member_name(
    owner: TypeName,
    ident: &'static str,
    args: Box<[TypeName]>) -> MemberName {
    MemberName { owner, ident, args }
}

/// Writes a non-empty angle-bracketed argument list.
///
/// An empty slice deliberately writes nothing so the same representation serves plain
/// functions, fields, variants, and non-generic methods.
fn write_generic_arguments(
    args: &[TypeName],
    formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    let Some((first, remaining)) = args.split_first() else {
        return Ok(());
    };

    formatter.write_str("<")?;
    fmt::Display::fmt(first, formatter)?;
    for argument in remaining {
        formatter.write_str(", ")?;
        fmt::Display::fmt(argument, formatter)?;
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
