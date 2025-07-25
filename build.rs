#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    
    // Get version from Cargo.toml or environment
    let version = env!("CARGO_PKG_VERSION");
    let version_parts: Vec<&str> = version.split('.').collect();
    
    // Convert version to Windows format (major.minor.patch.build)
    let major: u16 = version_parts.get(0).unwrap_or(&"0").parse().unwrap_or(0);
    let minor: u16 = version_parts.get(1).unwrap_or(&"0").parse().unwrap_or(0);
    let patch: u16 = version_parts.get(2).unwrap_or(&"0").parse().unwrap_or(0);
    let build: u16 = 0; // Build number, can be set to 0
    
    let version_u64 = ((major as u64) << 48) | ((minor as u64) << 32) | ((patch as u64) << 16) | (build as u64);
    
    // Set application information with dynamic version
    res.set_version_info(winres::VersionInfo::PRODUCTVERSION, version_u64);
    res.set_version_info(winres::VersionInfo::FILEVERSION, version_u64);
    
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
