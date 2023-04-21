use crate::{standard, Command};
use anyhow::Context;
use std::{io::Read, path::PathBuf};

#[derive(Debug, serde::Deserialize)]
pub struct XunleiDaemon {
    port: u32,
    internal: bool,
    download_dir: PathBuf,
}

impl XunleiDaemon {
    pub fn new() -> anyhow::Result<XunleiDaemon> {
        let mut config_file =
            std::fs::File::open(PathBuf::from(standard::SYNOPKG_PKGBASE).join("config.json"))?;
        let mut content = String::new();
        config_file.read_to_string(&mut content)?;
        match serde_json::from_str(&content).context("Failed deserialize to config.json") {
            Ok(daemon) => Ok(daemon),
            Err(_) => Ok(XunleiDaemon {
                port: 5055,
                internal: false,
                download_dir: PathBuf::from(standard::TMP_DOWNLOAD_PATH),
            }),
        }
    }
}

impl Command for XunleiDaemon {
    fn run(&self) -> anyhow::Result<()> {
        Ok(())
    }
}
