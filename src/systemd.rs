extern crate libc;

use std::ops::Not;
use std::path::Path;
use std::path::PathBuf;

use anyhow::{Context, Ok};
use rand::Rng;

use crate::standard;
use crate::xunlei_asset::Xunlei;
use crate::xunlei_asset::XunleiAsset;
use crate::Config;
use crate::Running;

pub struct XunleiInstall {
    description: &'static str,
    internal: bool,
    port: u16,
    download_path: PathBuf,
    config_path: PathBuf,
    uid: u32,
    gid: u32,
}

impl From<Config> for XunleiInstall {
    fn from(config: Config) -> Self {
        let uid = unsafe { libc::getuid() };
        let gid = unsafe { libc::getgid() };
        Self {
            description: "Thunder remote download service",
            internal: config.internal,
            port: config.port,
            download_path: config.download_path,
            config_path: config.config_path,
            uid,
            gid,
        }
    }
}

impl XunleiInstall {
    fn config(&self) -> anyhow::Result<()> {
        log::info!("[XunleiInstall] Configuration in progress");
        log::info!("[XunleiInstall] WebUI port: {}", self.port);

        if self.download_path.is_dir().not() {
            std::fs::create_dir_all(&self.download_path)?;
        } else if self.download_path.is_file() {
            return Err(anyhow::anyhow!("Download path must be a directory"));
        }

        if self.config_path.is_dir().not() {
            std::fs::create_dir_all(&self.config_path)?;
        } else if self.config_path.is_file() {
            return Err(anyhow::anyhow!("Config path must be a directory"));
        }
        log::info!(
            "[XunleiInstall] Config directory: {}",
            self.config_path.display()
        );
        log::info!(
            "[XunleiInstall] Download directory: {}",
            self.download_path.display()
        );
        log::info!("[XunleiInstall] Configuration completed");
        Ok(())
    }

    fn extract(&self) -> anyhow::Result<()> {
        log::info!("[XunleiInstall] Installing in progress");

        // /var/packages/pan-xunlei-com/target
        let target_dir = PathBuf::from(standard::SYNOPKG_PKGDEST);
        // /var/packages/pan-xunlei-com/target/host
        let host_dir = PathBuf::from(standard::SYNOPKG_HOST);
        // /var/packages/pan-xunlei-com/xunlei
        let start_endpoint = PathBuf::from(standard::SYNOPKG_PKGBASE).join(standard::APP_NAME);

        standard::create_dir_all(&target_dir, 0o755)?;

        let xunlei = XunleiAsset {};
        for file in xunlei.iter()? {
            let filename = file.as_str();
            let target_filepath = target_dir.join(filename);
            let data = xunlei.get(filename).context("Read data failure")?;
            standard::write_file(&target_filepath, data, 0o755)?;
            log::info!("[XunleiInstall] Install to: {}", target_filepath.display());
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

        // path: /var/packages/pan-xunlei-com/target/host/etc/synoinfo.conf
        let syno_info_path = PathBuf::from(format!(
            "{}{}",
            host_dir.display(),
            standard::SYNO_INFO_PATH
        ));
        standard::create_dir_all(
            &syno_info_path.parent().context(format!(
                "the path: {} parent not exists",
                syno_info_path.display()
            ))?,
            0o755,
        )?;
        let mut byte_arr = vec![0u8; 32];
        rand::thread_rng().fill(&mut byte_arr[..]);
        let hex_string = byte_arr
            .iter()
            .map(|u| format!("{:02x}", *u as u32))
            .collect::<String>()
            .chars()
            .take(7)
            .collect::<String>();
        standard::write_file(
            &syno_info_path,
            std::borrow::Cow::Borrowed(
                format!("unique=\"synology_{}_720+\"", hex_string).as_bytes(),
            ),
            0o644,
        )?;

        // path: /var/packages/pan-xunlei-com/target/host/usr/syno/synoman/webman/modules/authenticate.cgi
        let syno_authenticate_path = PathBuf::from(format!(
            "{}{}",
            host_dir.display(),
            standard::SYNO_AUTHENTICATE_PATH
        ));
        standard::create_dir_all(
            &syno_authenticate_path.parent().context(format!(
                "directory path: {} not exists",
                syno_authenticate_path.display()
            ))?,
            0o755,
        )?;
        standard::write_file(
            &syno_authenticate_path,
            std::borrow::Cow::Borrowed(String::from("#!/usr/bin/env sh\necho OK").as_bytes()),
            0o755,
        )?;

        // symlink
        unsafe {
            if !Path::new(standard::SYNO_INFO_PATH).exists() {
                let source_sys_info_path =
                    std::ffi::CString::new(syno_info_path.display().to_string())?;
                let target_sys_info_path = std::ffi::CString::new(standard::SYNO_INFO_PATH)?;
                if libc::symlink(source_sys_info_path.as_ptr(), target_sys_info_path.as_ptr()) != 0
                {
                    anyhow::bail!(std::io::Error::last_os_error());
                }
            }

            if !Path::new(standard::SYNO_AUTHENTICATE_PATH).exists() {
                let source_syno_authenticate_path =
                    std::ffi::CString::new(syno_authenticate_path.display().to_string())?;
                let target_syno_authenticate_path =
                    std::ffi::CString::new(standard::SYNO_AUTHENTICATE_PATH)?;

                if libc::symlink(
                    source_syno_authenticate_path.as_ptr(),
                    target_syno_authenticate_path.as_ptr(),
                ) != 0
                {
                    anyhow::bail!(std::io::Error::last_os_error());
                }
            }
        }

        let exe_path = std::env::current_exe()?;
        if exe_path.exists() {
            if std::fs::copy(exe_path, &start_endpoint)? == 0 {
                log::error!("description Failed to copy the execution file. the length is 0")
            } else {
                log::info!("[XunleiInstall] Install to: {}", start_endpoint.display());
            }
        }

        log::info!("[XunleiInstall] Installation completed");
        Ok(())
    }

    fn systemctl(&self) -> anyhow::Result<()> {
        let internal = if self.internal { "-i" } else { "" };
        let systemctl_unit = format!(
            r#"[Unit]
                Description={}
                After=network.target network-online.target
                Requires=network-online.target
                
                [Service]
                Type=simple
                ExecStart=/var/packages/pan-xunlei-com/xunlei execute {} -p {} -d {} -c {}
                LimitNOFILE=1024
                LimitNPROC=512
                User={}
                
                [Install]
                WantedBy=multi-user.target"#,
            self.description,
            internal,
            self.port,
            self.download_path.display(),
            self.config_path.display(),
            self.uid
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

impl Running for XunleiInstall {
    fn execute(&self) -> anyhow::Result<()> {
        self.config()?;
        self.extract()?;
        self.systemctl()
    }
}

pub struct XunleiUninstall;

impl XunleiUninstall {
    pub fn remove_service_file(&self) -> anyhow::Result<()> {
        let path = PathBuf::from(standard::SYSTEMCTL_UNIT_FILE);
        if path.exists() {
            std::fs::remove_file(path)?;
            log::info!("[XunleiUninstall] Uninstall xunlei service");
        }
        Ok(())
    }

    pub fn remove_package(&self) -> anyhow::Result<()> {
        let path = PathBuf::from(standard::SYNOPKG_PKGBASE);
        if path.exists() {
            std::fs::remove_dir_all(path)?;
            log::info!("[XunleiUninstall] Uninstall xunlei package");
        }

        Ok(())
    }
}

impl Running for XunleiUninstall {
    fn execute(&self) -> anyhow::Result<()> {
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
    let output = std::process::Command::new("systemctl")
        .args(args)
        .output()?;
    let status = output.status;
    if status.success().not() {
        log::error!(
            "[systemctl] {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}
