fn main() {
    let is_windows = std::env::var("CARGO_CFG_TARGET_OS").ok()
        .is_some_and(|v| v == "windows");
    if is_windows && std::path::Path::new("resources.rc").exists() {
        embed_resource::compile("resources.rc", embed_resource::NONE);
    }
}
