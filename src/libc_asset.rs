#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
#[derive(rust_embed::RustEmbed)]
#[folder = if cfg!(target_arch = "aarch64") {
    "libc/aarch64"
} else {
    "libc/x86_64"
}]
pub struct Asset;