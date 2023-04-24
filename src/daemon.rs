use crate::{standard, Running};
use anyhow::Context;
use std::{
    collections::HashMap,
    io::Read,
    ops::Not,
    path::{Path, PathBuf},
    process::Stdio,
};

#[derive(Debug, serde::Deserialize)]
pub struct XunleiDaemon {
    port: u16,
    internal: bool,
    download_path: PathBuf,
    #[serde(skip_serializing)]
    config_path: Option<PathBuf>,
}

impl XunleiDaemon {
    pub fn new(config_path: Option<PathBuf>) -> anyhow::Result<XunleiDaemon> {
        let config_path: PathBuf = config_path.unwrap_or(PathBuf::from(standard::SYNOPKG_PKGBASE));
        let mut config_file = std::fs::File::open(config_path.join(standard::CONFIG_FILE_NAME))?;
        let mut content = String::new();
        config_file.read_to_string(&mut content)?;
        match serde_json::from_str::<XunleiDaemon>(&content)
            .context("Failed deserialize to config.json")
        {
            Ok(mut daemon) => {
                daemon.config_path = Some(config_path);
                Ok(daemon)
            }
            Err(_) => Ok(XunleiDaemon {
                port: 5055,
                internal: false,
                download_path: PathBuf::from(standard::TMP_DOWNLOAD_PATH),
                config_path: Some(config_path),
            }),
        }
    }

    fn run_backend(envs: HashMap<String, String>) -> anyhow::Result<std::process::Child> {
        log::info!("[XunleiDaemon] Start Xunlei Engine");
        standard::create_dir_all(&Path::new(standard::SYNOPKG_VAR), 0o755)?;
        let child_process = std::process::Command::new(standard::LAUNCHER_EXE)
            .args([
                format!("-launcher_listen={}", standard::LAUNCHER_SOCK),
                format!("-pid={}", standard::PID_FILE),
                format!("-logfile={}", standard::LAUNCH_LOG_FILE),
            ])
            .current_dir(standard::SYNOPKG_PKGDEST)
            .envs(&envs)
            // Join the parent process group by default
            .spawn()
            .expect("failed to spawn child process");
        let child_pid = child_process.id() as libc::pid_t;
        log::info!("[XunleiDaemon] Backend pid: {}", child_pid);
        Ok(child_process)
    }

    fn run_ui(internal: bool, port: u16, envs: HashMap<String, String>) {
        log::info!("[XunleiDaemon] Start Xunlei Engine UI");
        let address = match internal {
            true => "127.0.0.1",
            false => "0.0.0.0",
        };
        rouille::start_server(format!("{}:{}", address, port), move |request| {
            rouille::router!(request,
                (GET) ["/webman/login.cgi"] => {
                    let mut  json = HashMap::new();
                    json.insert("SynoToken", "");
                    rouille::Response::json(&json)
                    .with_additional_header("Content-Type", "application/json; charset=utf-8")
                    .with_status_code(200)
                 },
                (GET) ["/"] => {
                    rouille::Response::redirect_307("/webman/3rdparty/pan-xunlei-com/index.cgi/")
                },
                (GET) ["/webman/"] => {
                    rouille::Response::redirect_307("/webman/3rdparty/pan-xunlei-com/index.cgi/")
                },
                (GET) ["/webman/3rdparty/pan-xunlei-com"] => {
                    rouille::Response::redirect_307("/webman/3rdparty/pan-xunlei-com/index.cgi/")
                 },
                _ => {
                    let mut cmd = std::process::Command::new(standard::SYNOPKG_CLI_WEB);
                    // cmd.current_dir(standard::SYNOPKG_PKGDEST);

                    cmd.envs(&envs)
                    .env("SERVER_SOFTWARE", "rust")
                    .env("SERVER_PROTOCOL", "HTTP/1.1")
                    .env("HTTP_HOST", &request.remote_addr().to_string())
                    .env("GATEWAY_INTERFACE", "CGI/1.1")
                    .env("REQUEST_METHOD", request.method())
                    .env("QUERY_STRING", &request.raw_query_string())
                    .env("REQUEST_URI", &request.raw_url())
                    .env("PATH_INFO", &request.url())
                    .env("SCRIPT_NAME", ".")
                    .env("SCRIPT_FILENAME", &request.url())
                    .env("SERVER_PORT", port.to_string())
                    .env("REMOTE_ADDR", request.remote_addr().to_string())
                    .env("SERVER_NAME", request.remote_addr().to_string())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::inherit())
                    .stdin(Stdio::piped());

                    for ele in request.headers() {
                        let k = ele.0.to_uppercase();
                        let v = ele.1;
                        if k == "PROXY" {
                            continue
                        }
                        if v.is_empty().not() {
                            cmd.env(format!("HTTP_{}", k), v);
                        }
                    }

                    if request.header("Content-Type").unwrap_or_default().is_empty().not() {
                        cmd.env(
                            "CONTENT_TYPE",
                            &request.header("Content-Type").unwrap(),
                        );
                    }

                    if request.header("content-type").unwrap_or_default().is_empty().not() {
                        cmd.env(
                            "CONTENT_TYPE",
                            &request.header("content-type").unwrap(),
                        );
                    }

                    if request.header("Content-Length").unwrap_or_default().is_empty().not() {
                        cmd.env(
                            "CONTENT_LENGTH",
                            &request.header("Content-Length").unwrap(),
                        );
                    }

                    let mut child = cmd.spawn().unwrap();

                    if let Some(mut body) = request.data() {
                        std::io::copy(&mut body, child.stdin.as_mut().unwrap()).unwrap();
                    }

                    let response = {
                        let mut stdout = std::io::BufReader::new(child.stdout.unwrap());

                        let mut headers = Vec::new();
                        let mut status_code = 200;
                        for header in std::io::BufRead::lines(stdout.by_ref()) {
                            let header = header.unwrap();
                            if header.is_empty() {
                                break;
                            }

                            let (header, val) = header.split_once(':').unwrap();
                            let val = &val[1..];

                            if header == "Status" {
                                status_code = val[0..3]
                                    .parse()
                                    .expect("Status returned by CGI program is invalid");
                            } else {
                                headers.push((header.to_owned().into(), val.to_owned().into()));
                            }
                        }
                        rouille::Response {
                            status_code,
                            headers,
                            data: rouille::ResponseBody::from_reader(stdout),
                            upgrade: None,
                        }
                    };

                    response
                }
            )
        });
    }

    pub fn envs(&self) -> HashMap<String, String> {
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
        envs.insert(
            String::from("HOME"),
            self.config_path.clone().unwrap().display().to_string(),
        );
        envs.insert(
            String::from("ConfigPath"),
            self.config_path.clone().unwrap().display().to_string(),
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
    fn execute(&self) -> anyhow::Result<()> {
        use std::thread::{Builder, JoinHandle};
        let envs = self.envs();
        let mut backend_process = XunleiDaemon::run_backend(envs.clone())?;
        let backend_thread: JoinHandle<_> = Builder::new()
            .name("backend".to_string())
            .spawn(move || {
                let status = backend_process
                    .wait()
                    .expect("Failed to wait child process");
                if status.success() {
                    log::info!("[XunleiDaemon] Xunlei backend has exit(0)")
                }
            })
            .expect("Failed to start backend thread");

        let internal = self.internal;
        let port = self.port;
        let ui_thread: JoinHandle<_> = Builder::new()
            .name("ui".to_string())
            .spawn(move || {
                XunleiDaemon::run_ui(internal, port, envs);
            })
            .expect("Failed to start ui thread");

        // waiting... child thread exit
        ui_thread.join().expect("Failed to join ui_thread");
        backend_thread
            .join()
            .expect("Failed to join backend_thread");

        Ok(())
    }
}
