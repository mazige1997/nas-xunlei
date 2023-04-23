use crate::{standard, Running};
use anyhow::Context;
use rouille::cgi::CgiRun;
use std::{
    collections::HashMap,
    io::Read,
    path::{Path, PathBuf},
    process::Stdio,
};

#[derive(Debug, serde::Deserialize)]
pub struct XunleiDaemon {
    port: u32,
    internal: bool,
    download_path: PathBuf,
    #[serde(skip_serializing)]
    config_path: PathBuf,
}

impl XunleiDaemon {
    pub fn new(config_path: Option<PathBuf>) -> anyhow::Result<XunleiDaemon> {
        let config_path: PathBuf =
            config_path.unwrap_or(PathBuf::from(standard::DEFAULT_CONFIG_PATH));
        let mut config_file =
            std::fs::File::open(PathBuf::from(standard::SYNOPKG_PKGBASE).join("config.json"))?;
        let mut content = String::new();
        config_file.read_to_string(&mut content)?;
        match serde_json::from_str::<XunleiDaemon>(&content)
            .context("Failed deserialize to config.json")
        {
            Ok(mut daemon) => {
                daemon.config_path = config_path;
                Ok(daemon)
            }
            Err(_) => Ok(XunleiDaemon {
                port: 5055,
                internal: false,
                download_path: PathBuf::from(standard::TMP_DOWNLOAD_PATH),
                config_path: config_path,
            }),
        }
    }

    fn run_backend(&self) -> anyhow::Result<std::process::Child> {
        log::info!("[XunleiDaemon] Start Xunlei Engine");
        standard::create_dir_all(&Path::new(standard::SYNOPKG_VAR), 0o755)?;
        let child_process = std::process::Command::new(standard::LAUNCHER_EXE)
            .args([
                format!("-launcher_listen={}", standard::LAUNCHER_SOCK),
                format!("-pid={}", standard::PID_FILE),
                format!("-logfile={}", standard::LAUNCH_LOG_FILE),
            ])
            .current_dir(standard::SYNOPKG_PKGDEST)
            .envs(self.envs())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("failed to spawn child process");
        let child_pid = child_process.id() as libc::pid_t;
        let parent_pgid = unsafe { libc::getpgid(libc::getpid()) };
        unsafe {
            if libc::setpgid(child_pid, parent_pgid) < 0 {
                panic!("failed to set child process group ID");
            }
        }
    
        Ok(child_process)
    }

    fn run_ui(&self) -> ! {
        log::info!("[XunleiDaemon] Start Xunlei Engine UI");
        let envs = self.envs();
        rouille::start_server("localhost:8000", move |request| {
            let path = request.raw_url();
            if path == "/"
                || path.starts_with("/webman/")
                || path.starts_with("/webman/3rdparty/pan-xunlei-com")
            {
                let mut cmd = std::process::Command::new(standard::SYNOPKG_CLI_WEB);
                cmd.current_dir(standard::SYNOPKG_PKGDEST);
                cmd.envs(&envs);

                return cmd.start_cgi(request).unwrap();
            } else if path == "/webman/login.cgi" {
                return rouille::Response::text(r#"{"SynoToken":""}"#)
                .with_unique_header("Content-Type", "application/json; charset=utf-8")
                .with_status_code(200)
            }
            rouille::Response::empty_404()
        });
    }

    fn envs(&self) -> HashMap<String, String> {
        let mut envs = HashMap::new();
        envs.insert(
            String::from("DriveListen"),
            String::from(standard::SOCK_FILE),
        );
        envs.insert(
            String::from("OS_VERSION"),
            format!(
                "dsm {}.{}-{}",
                standard::SYNOPKG_DSM_VERSION_MAJOR,
                standard::SYNOPKG_DSM_VERSION_MINOR,
                standard::SYNOPKG_DSM_VERSION_BUILD
            ),
        );
        envs.insert(String::from("HOME"), self.config_path.display().to_string());
        envs.insert(
            String::from("ConfigPath"),
            self.config_path.display().to_string(),
        );
        envs.insert(
            String::from("DownloadPATH"),
            self.download_path.display().to_string(),
        );
        envs.insert(
            String::from("SYNOPKG_DSM_VERSION_MAJOR"),
            String::from(standard::SYNOPKG_DSM_VERSION_MAJOR),
        );
        envs.insert(
            String::from("SYNOPKG_DSM_VERSION_MINOR"),
            String::from(standard::SYNOPKG_DSM_VERSION_MINOR),
        );
        envs.insert(
            String::from("SYNOPKG_DSM_VERSION_BUILD"),
            String::from(standard::SYNOPKG_DSM_VERSION_BUILD),
        );

        envs.insert(
            String::from("SYNOPKG_PKGDEST"),
            String::from(standard::SYNOPKG_PKGDEST),
        );
        envs.insert(
            String::from("SYNOPKG_PKGNAME"),
            String::from(standard::SYNOPKG_PKGNAME),
        );
        envs.insert(
            String::from("SVC_CWD"),
            String::from(standard::SYNOPKG_PKGDEST),
        );

        envs.insert(String::from("PID_FILE"), String::from(standard::PID_FILE));
        envs.insert(String::from("ENV_FILE"), String::from(standard::ENV_FILE));
        envs.insert(String::from("LOG_FILE"), String::from(standard::LOG_FILE));
        envs.insert(
            String::from("LAUNCH_LOG_FILE"),
            String::from(standard::LAUNCH_LOG_FILE),
        );
        envs.insert(
            String::from("LAUNCH_PID_FILE"),
            String::from(standard::LAUNCH_PID_FILE),
        );
        envs.insert(String::from("INST_LOG"), String::from(standard::INST_LOG));
        envs.insert(String::from("GIN_MODE"), String::from("release"));
        envs
    }
}

impl Running for XunleiDaemon {
    fn run(&self) -> anyhow::Result<()> {
        let backend_process = self.run_backend()?;
        self.run_ui();
        Ok(())
    }
}
