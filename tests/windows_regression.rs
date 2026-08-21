/// A local stand-in for the generated Windows namespaces that originally exposed
/// partial path shortening.
///
/// The nonstandard module casing deliberately matches `windows-rs` output so the
/// regression keeps its original path shape without depending on that crate.
#[allow(non_snake_case, reason = "mirrors generated windows-rs namespaces")]
mod windows {
    /// Mirrors the generated Windows Foundation namespace.
    pub mod Foundation {
        /// Mirrors a generated event handler whose arguments retain nested type paths.
        pub struct TypedEventHandler<Sender, Arguments>(
            /// Retains both generic arguments without imposing representation requirements.
            pub std::marker::PhantomData<(Sender, Arguments)>);
    }

    /// Mirrors the generated Windows ApplicationModel namespace.
    pub mod ApplicationModel {
        /// Mirrors the generated Windows activation namespace.
        pub mod Activation {
            /// A generated activation argument used as a nested generic argument.
            pub struct IActivatedEventArgs;
        }

        /// Mirrors the generated Windows core-application namespace.
        pub mod Core {
            /// A generated application view used as a nested generic argument.
            pub struct CoreApplicationView;
        }
    }
}

/// The generated WinRT event-handler type that exposed partial path shortening in the
/// original implementation.
type ActivatedHandler = windows::Foundation::TypedEventHandler<
    windows::ApplicationModel::Core::CoreApplicationView,
    windows::ApplicationModel::Activation::IActivatedEventArgs>;

/// Verifies every generated `windows-rs` path is shortened recursively instead of
/// preserving qualifications inside the outer generic type.
#[test]
fn generated_windows_event_handler_shortens_every_type_path() {
    assert_eq!(
        pretty_name::type_name::<ActivatedHandler>().to_string(),
        "TypedEventHandler<CoreApplicationView, IActivatedEventArgs>");
}
