#[cfg(target_arch = "x86_64")]
#[derive(rust_embed::RustEmbed)]
#[folder = "libc/x86_64/"]
struct Asset;

#[cfg(target_arch = "aarch64")]
#[derive(rust_embed::RustEmbed)]
#[folder = "libc/aarch64/"]
struct Asset;

pub(crate) fn ld_env(envs: &mut std::collections::HashMap<String, String>) -> anyhow::Result<()> {
    use crate::standard;
    use anyhow::Context;
    const LD_LIBRARY_PATH: &str = "/tmp/libc";
    let libc_path = std::path::Path::new(LD_LIBRARY_PATH);
    if !libc_path.exists() {
        standard::create_dir_all(libc_path, 0o755)?;
    }
    for filename in Asset::iter()
        .map(|v| v.into_owned())
        .collect::<Vec<String>>()
    {
        let file = Asset::get(&filename).context("Failed to get bin asset")?;
        let target_file = libc_path.join(filename);
        standard::write_file(&target_file, file.data, 0o755)?;
    }
    envs.insert(String::from("LD_LIBRARY_PATH"), LD_LIBRARY_PATH.to_string());
    Ok(())
}
