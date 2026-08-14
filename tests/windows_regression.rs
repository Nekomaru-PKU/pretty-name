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
