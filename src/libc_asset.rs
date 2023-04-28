#[cfg(target_arch = "x86_64")]
#[derive(rust_embed::RustEmbed)]
#[folder = "libc/x86_64/"]
struct Asset;

#[cfg(target_arch = "aarch64")]
#[derive(rust_embed::RustEmbed)]
#[folder = "libc/aarch64/"]
struct Asset;

pub(crate) fn ld_env(envs: &mut std::collections::HashMap<String, String>) -> anyhow::Result<()> {
    use anyhow::Context;
    #[cfg(target_arch = "x86_64")]
    const LD: &str = "ld-linux-x86-64.so.2";
    #[cfg(target_arch = "aarch64")]
    const LD: &str = "ld-linux-aarch64.so.1";
    const LD_PATH: &str = "/lib";
    const LD_LIBRARY_PATH: &str = "/tmp/libc";
    let libc_path = std::path::Path::new(LD_LIBRARY_PATH);
    if !libc_path.exists() {
        crate::standard::create_dir_all(libc_path, 0o755)?;
    }
    for filename in Asset::iter()
        .map(|v| v.into_owned())
        .collect::<Vec<String>>()
    {
        let file = Asset::get(&filename).context("Failed to get bin asset")?;
        let target_file = libc_path.join(filename);
        crate::standard::write_file(&target_file, file.data, 0o755)?;
    }
    let ld_path = std::path::Path::new(LD_PATH).join(LD);
    if ld_path.exists() {
        std::fs::remove_file(&ld_path)?;
    }
    unsafe {
        let source_path = std::ffi::CString::new(
            std::path::Path::new(LD_LIBRARY_PATH)
                .join(LD)
                .display()
                .to_string(),
        )?;
        let target_path = std::ffi::CString::new(ld_path.display().to_string())?;
        if libc::symlink(source_path.as_ptr(), target_path.as_ptr()) != 0 {
            anyhow::bail!(std::io::Error::last_os_error());
        }
    }
    envs.insert(String::from("LD_LIBRARY_PATH"), LD_LIBRARY_PATH.to_string());
    Ok(())
}
