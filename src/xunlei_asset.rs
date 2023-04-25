use core::str;
use std::{borrow::Cow, io::Write, ops::Not, path::PathBuf};

use anyhow::Context;

use crate::standard;

#[derive(rust_embed::RustEmbed)]
#[folder = "xunlei/"]
struct Asset;

pub trait Xunlei {
    fn version(&self) -> anyhow::Result<String>;

    fn get(&self, filename: &str) -> anyhow::Result<Cow<[u8]>>;

    fn iter(&self) -> anyhow::Result<Vec<String>>;
}

pub struct XunleiAsset;

impl Xunlei for XunleiAsset {
    fn version(&self) -> anyhow::Result<String> {
        let version_bin = Asset::get("version").context("Failed to get version asset")?;
        let version = std::str::from_utf8(version_bin.data.as_ref())
            .context("Error getting version number!")?;
        Ok(String::from(version))
    }

    fn get(&self, filename: &str) -> anyhow::Result<Cow<[u8]>> {
        let bin = Asset::get(filename).context("Failed to get bin asset")?;
        Ok(bin.data)
    }

    fn iter(&self) -> anyhow::Result<Vec<String>> {
        Ok(Asset::iter()
            .map(|v| v.into_owned())
            .collect::<Vec<String>>())
    }
}

pub struct XunleiLocalAsset {
    tmp: PathBuf,
    filename: String,
}

impl XunleiLocalAsset {
    pub fn new() -> Self {
        let xunlei = Self {
            tmp: PathBuf::from("/tmp/xunlei_bin"),
            filename: format!("nasxunlei-DSM7-{}.spk", standard::SUPPORT_ARCH),
        };
        match xunlei.exestrct_package() {
            Ok(status) => {
                if status.success().not() {
                    log::error!(
                        "[XunleiLocalAsset] There was an error extracting the download package"
                    )
                }
            }
            Err(e) => {
                panic!("{}", e)
            }
        }
        xunlei
    }

    fn exestrct_package(&self) -> anyhow::Result<std::process::ExitStatus> {
        let mut response = ureq::get(&format!("http://down.sandai.net/nas/{}", self.filename))
            .call()?
            .into_reader();
        if self.tmp.exists().not() {
            standard::create_dir_all(&self.tmp, 0o755)?;
        }
        let file_path = self.tmp.join(self.filename.as_str());
        let mut output_file = std::fs::File::create(&file_path)?;
        std::io::copy(&mut response, &mut output_file)?;
        output_file.flush()?;
        drop(output_file);

        let dir = self.tmp.display();
        let filename = self.filename.as_str();
        Ok(std::process::Command::new("sh")
                .arg("-c")
                .arg(format!("tar --wildcards -Oxf $(find {dir} -type f -name {filename} | head -n1) package.tgz | tar --wildcards -xJC {dir} 'bin/bin/*' 'ui/index.cgi' &&
                    mv {dir}/bin/bin/* {dir}/ &&
                    mv {dir}/ui/index.cgi {dir}/xunlei-pan-cli-web &&
                    rm -rf {dir}/bin/bin &&
                    rm -rf {dir}/bin &&
                    rm -rf {dir}/ui &&
                    rm -f {dir}/version_code {dir}/{filename}
                "))
                .spawn()?
                .wait()?
            )
    }
}

impl Xunlei for XunleiLocalAsset {
    fn version(&self) -> anyhow::Result<String> {
        Ok(std::fs::read_to_string(
            PathBuf::from(&self.tmp).join("version"),
        )?)
    }

    fn get(&self, filename: &str) -> anyhow::Result<Cow<[u8]>> {
        let vec = std::fs::read(PathBuf::from(&self.tmp).join(filename))?;
        Ok(std::borrow::Cow::from(vec))
    }

    fn iter(&self) -> anyhow::Result<Vec<String>> {
        let entries = std::fs::read_dir(&self.tmp)?;
        let mut file_names = Vec::new();
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(file_name) = path.file_name() {
                    file_names.push(file_name.to_string_lossy().to_string());
                }
            }
        }
        Ok(file_names)
    }
}
