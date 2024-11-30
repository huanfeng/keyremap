fn main() {
    // 仅在 Windows 平台上添加链接参数
    if cfg!(target_os = "windows") {
        println!("cargo:rustc-link-arg=assets/app.res");
    }
}