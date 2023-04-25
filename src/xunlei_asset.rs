use core::str;
use std::{borrow::Cow, net::TcpStream, path::PathBuf};

use anyhow::Context;

use crate::standard;

#[derive(rust_embed::RustEmbed)]
#[folder = "xunlei/"]
struct Asset;

pub trait Xunlei {
    fn version() -> anyhow::Result<String>;

    fn get(filename: &str) -> anyhow::Result<Cow<[u8]>>;

    fn iter() -> Vec<String>;
}

pub struct XunleiAsset;

impl Xunlei for XunleiAsset {
    fn version() -> anyhow::Result<String> {
        let version_bin = Asset::get("version").context("Failed to get version asset")?;
        let version = std::str::from_utf8(version_bin.data.as_ref())
            .context("Error getting version number!")?;
        Ok(String::from(version))
    }

    fn get(filename: &str) -> anyhow::Result<Cow<[u8]>> {
        let bin = Asset::get(filename).context("Failed to get bin asset")?;
        Ok(bin.data)
    }

    fn iter() -> Vec<String> {
        Asset::iter()
            .map(|v| v.into_owned())
            .collect::<Vec<String>>()
    }
}

pub struct XunleiLocalAsset(PathBuf);

impl XunleiLocalAsset {
    pub fn new() -> Self {
        Self(PathBuf::from(standard::TMP_DOWNLOAD_PATH))
    }

    fn download_package() -> anyhow::Result<TcpStream> {
        let arch = if std::env::consts::ARCH == "x86_64" {
            "x86_64"
        } else if std::env::consts::ARCH == "aarch64" {
            "armv8"
        } else {
            anyhow::bail!("Unsupported CPU architecture")
        };
        let url = format!("http://down.sandai.net/nas/nasxunlei-DSM7-{}.spk", arch);

        let request = format!(
            "GET {} HTTP/1.1\r\n\
             Host: down.sandai.net\r\n\
             Connection: close\r\n\
             \r\n",
            url
        );

        let mut stream = TcpStream::connect("down.sandai.net:80")?;
        std::io::Write::write(&mut stream, request.as_bytes())?;

        Ok(stream)
    }
}

impl Xunlei for XunleiLocalAsset {
    fn version() -> anyhow::Result<String> {
        todo!()
    }

    fn get(filename: &str) -> anyhow::Result<Cow<[u8]>> {
        todo!()
    }

    fn iter() -> Vec<String> {
        todo!()
    }
}
