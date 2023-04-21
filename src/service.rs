extern crate libc;

use std::{env, path::PathBuf};

use anyhow::{Context, Ok};
use rand::Rng;

use crate::standard;
use crate::xunlei_asset::XunleiAsset;
use crate::Command;

#[derive(serde::Serialize)]
pub struct Install {
    #[serde(skip_serializing)]
    description: &'static str,
    port: u32,
    internal: bool,
    #[serde(skip_serializing)]
    uid: u32,
    #[serde(skip_serializing)]
    gid: u32,
    download_dir: PathBuf,
}

impl Install {
    pub fn new() -> Self {
        let port = env::var("XUNLEI_PORT")
            .ok()
            .and_then(|port| port.parse::<u32>().ok())
            .unwrap_or(5051);

        let internal = env::var("XUNLEI_INTERNAL")
            .ok()
            .and_then(|internal| internal.parse::<bool>().ok())
            .unwrap_or(false);

        let uid = env::var("UID")
            .ok()
            .and_then(|uid| uid.parse::<u32>().ok())
            .unwrap();

        let gid: u32 = env::var("GID")
            .ok()
            .and_then(|gid| gid.parse::<u32>().ok())
            .unwrap();

        let _config_path = env::var("XUNLEI_DOWNLOAD_DIR")
            .ok()
            .and_then(|_config_path: String| _config_path.parse::<PathBuf>().ok())
            .unwrap_or(PathBuf::from("/etc/xunlei"));

        let download_dir = env::var("XUNLEI_DOWNLOAD_DIR")
            .ok()
            .and_then(|download_dir| download_dir.parse::<PathBuf>().ok())
            .unwrap_or(PathBuf::from(standard::TMP_DOWNLOAD_PATH));
        Self {
            description: "Thunder remote download service",
            port,
            internal,
            uid,
            gid,
            download_dir,
        }
    }

    fn config(&self) -> anyhow::Result<()> {
        log::info!("Configuration");
        log::info!("WebUI port: {}", self.port);
        log::info!("Download Directory: {}", self.download_dir.display());
        let config_json = serde_json::to_vec(&self).context("Failed serialize to config.json")?;
        let config_path = PathBuf::from(standard::SYNOPKG_PKGBASE).join("config.json");
        if let Some(parent) = config_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let mut options = std::fs::OpenOptions::new();
        options.write(true);
        options.create(true);
        std::os::unix::prelude::OpenOptionsExt::mode(&mut options, 0o666);
        let mut file = options.open(config_path).context("Failed to create file")?;
        std::io::Write::write_all(&mut file, &config_json).context("Failed to write to file")?;
        Ok(())
    }

    fn extract(&self) -> anyhow::Result<()> {
        log::info!("Installing...");

        // /var/packages/pan-xunlei-com/target
        let target_dir = PathBuf::from(standard::SYNOPKG_PKGBASE).join("target");
        // /var/packages/pan-xunlei-com/target/host
        let host_dir = PathBuf::from(&target_dir).join("host");
        // /var/packages/pan-xunlei-com/xunlei
        let start_endpoint = PathBuf::from(standard::SYNOPKG_PKGBASE).join(standard::APP_NAME);

        standard::create_dir_all(&target_dir, 0o755)?;

        log::info!("Installing xunlei...");
        for file in XunleiAsset::iter() {
            let filename = file.as_ref();
            let target_filepath = target_dir.join(filename);

            let data = XunleiAsset::get(filename)?;
            standard::write_file(&target_filepath, data, 0o755)?;
            log::info!("dump file: {}", target_filepath.display())
        }

        standard::set_permissions(standard::SYNOPKG_PKGBASE, self.uid, self.gid).context(
            format!(
                "Failed to set permission: {}, PUID:{}, GUID:{}",
                standard::SYNOPKG_PKGBASE,
                self.uid,
                self.gid
            ),
        )?;

        standard::set_permissions(target_dir.to_str().unwrap(), self.uid, self.gid).context(
            format!(
                "Failed to set permission: {}, PUID:{}, GUID:{}",
                target_dir.display(),
                self.uid,
                self.gid
            ),
        )?;

        standard::set_permissions(host_dir.to_str().unwrap(), self.uid, self.gid).context(
            format!(
                "Failed to set permission: {}, PUID:{}, GUID:{}",
                target_dir.display(),
                self.uid,
                self.gid
            ),
        )?;

        let sys_info_path = host_dir.join(standard::SYNO_INFO_PATH);
        standard::create_dir_all(&sys_info_path.parent().unwrap().to_path_buf(), 0o755)?;

        let mut rb = vec![0u8; 32];
        rand::thread_rng().fill(&mut rb[..]);
        let rs = hex::encode(&rb[..]).chars().take(7).collect::<String>();
        standard::write_file(
            &sys_info_path,
            std::borrow::Cow::Borrowed(format!("unique=\"synology_{}_720+\"", rs).as_bytes()),
            0o644,
        )?;

        let syno_authenticate_path = host_dir.join(standard::SYNO_AUTHENTICATE_PATH);
        standard::create_dir_all(
            &syno_authenticate_path.parent().unwrap().to_path_buf(),
            0o755,
        )?;

        standard::write_file(
            &syno_authenticate_path,
            std::borrow::Cow::Borrowed(String::from("#!/usr/bin/env sh\necho OK").as_bytes()),
            0o755,
        )?;

        // symlink
        unsafe {
            let source_sys_info_path = std::ffi::CString::new(sys_info_path.display().to_string())?;
            let target_sys_info_path = std::ffi::CString::new(standard::SYNO_INFO_PATH)?;
            if libc::symlink(source_sys_info_path.as_ptr(), target_sys_info_path.as_ptr()) != 0 {
                anyhow::bail!(std::io::Error::last_os_error());
            }

            let source_syno_authenticate_path =
                std::ffi::CString::new(syno_authenticate_path.display().to_string())?;
            let target_syno_authenticate_path = std::ffi::CString::new(standard::SYNO_INFO_PATH)?;
            if libc::symlink(
                source_syno_authenticate_path.as_ptr(),
                target_syno_authenticate_path.as_ptr(),
            ) != 0
            {
                anyhow::bail!(std::io::Error::last_os_error());
            }
        }

        let exepath = std::env::current_exe()?;
        if exepath.exists() {
            log::info!("[extract] {}", start_endpoint.display());
            if std::fs::copy(exepath, start_endpoint)? == 0 {
                log::error!("description Failed to copy the execution file. the length is 0")
            }
        }

        log::info!("Installation completed");

        Ok(())
    }

    fn systemctl(&self) -> anyhow::Result<()> {
        let systemctl_unit = format!(
            r#"[Unit]
                Description={}
                After=network.target network-online.target
                Requires=network-online.target
                
                [Service]
                Type=simple
                ExecStart=/var/packages/pan-xunlei-com/xunlei run
                LimitNOFILE=1024
                LimitNPROC=512
                User={}
                
                [Install]
                WantedBy=multi-user.target"#,
            self.description, self.uid
        );

        standard::write_file(
            &PathBuf::from(standard::SYSTEMCTL_UNIT_FILE),
            std::borrow::Cow::Borrowed(systemctl_unit.as_bytes()),
            0o666,
        )?;

        systemctl(["daemon-reload"])?;
        systemctl(["enable", standard::APP_NAME])?;
        systemctl(["start", standard::APP_NAME])?;
        Ok(())
    }
}

impl Command for Install {
    fn run(&self) -> anyhow::Result<()> {
        self.extract()?;
        self.config()?;
        self.systemctl()
    }
}

pub struct Uninstall;

impl Uninstall {
    pub fn remove_service_file(&self) -> anyhow::Result<()> {
        std::fs::remove_file(PathBuf::from(standard::SYSTEMCTL_UNIT_FILE))?;
        log::info!("Uninstall systemctl service");
        Ok(())
    }

    pub fn remove_package(&self) -> anyhow::Result<()> {
        std::fs::remove_dir_all(PathBuf::from(standard::SYNOPKG_PKGBASE))?;
        log::info!("Uninstall xunlei package");
        Ok(())
    }
}

impl Command for Uninstall {
    fn run(&self) -> anyhow::Result<()> {
        systemctl(["disable", standard::APP_NAME])?;
        systemctl(["stop", standard::APP_NAME])?;
        self.remove_service_file()?;
        self.remove_package()?;
        systemctl(["daemon-reload"])?;
        Ok(())
    }
}

fn systemctl<I, S>(args: I) -> anyhow::Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr> + std::convert::AsRef<std::ffi::OsStr>,
{
    log::info!("Operating the systemctl service");
    let output = std::process::Command::new("systemctl")
        .args(args)
        .output()?;
    println!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}
