#[cfg(windows)]
fn main() {
    use std::io::Write;
    
    let mut res = winres::WindowsResource::new();
    
    // 设置应用程序信息
    res.set_version_info(winres::VersionInfo::PRODUCTVERSION, 0x00010000);
    res.set_version_info(winres::VersionInfo::FILEVERSION, 0x00010000);
    
    // 设置图标（如果有的话）
    // res.set_icon("icon.ico");
    
    // 编译资源
    if let Err(e) = res.compile() {
        eprintln!("Warning: Failed to compile Windows resources: {}", e);
        // 不要让构建失败，只是警告
    }
}

#[cfg(not(windows))]
fn main() {
    // 在非Windows平台上什么都不做
}
