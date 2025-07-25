#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    
    // Set application information
    res.set_version_info(winres::VersionInfo::PRODUCTVERSION, 0x00010000);
    res.set_version_info(winres::VersionInfo::FILEVERSION, 0x00010000);
    
    // Set icon
    res.set_icon("icon.ico");
    
    // Compile resources
    if let Err(e) = res.compile() {
        eprintln!("Warning: Failed to compile Windows resources: {}", e);
        // Don't fail the build, just warn
    }
}

#[cfg(not(windows))]
fn main() {
    // For Mac platform, icon is handled through bundle configuration
    // Other build-time tasks can be executed here
    println!("cargo:rerun-if-changed=icon.icns");
    println!("cargo:rerun-if-changed=icon.png");
}
