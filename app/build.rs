//! Embeds the app icon (and the version from Cargo.toml) into the Windows exe.

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rerun-if-changed=../assets/icons/balance.ico");
        let mut res = winresource::WindowsResource::new();
        res.set_icon("../assets/icons/balance.ico");
        res.compile().expect("failed to embed Windows resources");
    }
}
