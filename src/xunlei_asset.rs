use std::borrow::Cow;

use anyhow::Context;

#[derive(rust_embed::RustEmbed)]
#[folder = "xunlei/"]
struct Asset;

pub struct XunleiAsset;

impl XunleiAsset {
    pub fn get_version() -> anyhow::Result<String> {
        let version_bin = Asset::get("version").context("Failed to get version asset")?;
        let version = std::str::from_utf8(version_bin.data.as_ref())
            .context("Error getting version number!")?;
        Ok(String::from(version))
    }

    pub fn get(filename: &str) -> anyhow::Result<Cow<[u8]>> {
        let bin = Asset::get(filename).context("Failed to get version asset")?;
        Ok(bin.data)
    }

    pub fn iter() -> impl Iterator<Item = std::borrow::Cow<'static, str>> {
        Asset::iter()
    }
}
