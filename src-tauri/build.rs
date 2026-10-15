fn main() {
    let profile = std::env::var("PROFILE").unwrap_or_default();
    let attrs = if profile == "release" {
        tauri_build::Attributes::new().windows_attributes(
            tauri_build::WindowsAttributes::new().app_manifest(include_str!("tauri-app.manifest")),
        )
    } else {
        tauri_build::Attributes::new()
    };
    tauri_build::try_build(attrs).expect("failed to run tauri-build");
}
