#[cfg(target_os = "windows")]
fn main() {
    // only build the resource for release builds
    // as calling rc.exe might be slow
    let mut res = winres::WindowsResource::new();
    if cfg!(feature = "elevate") {
        res.set_manifest(
            r#"
    <assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
    <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
        <security>
            <requestedPrivileges>
                <requestedExecutionLevel level="requireAdministrator" uiAccess="false" />
            </requestedPrivileges>
        </security>
    </trustInfo>
    </assembly>
    "#,
        );
    }
    res.set_icon_with_id("icon.ico", "timer_icon");
    if let Err(error) = res.compile() {
        eprint!("{error}");
        std::process::exit(1);
    }
}
