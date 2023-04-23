pub mod daemon;
pub mod service;
pub mod standard;
pub mod xunlei_asset;
use std::io::Write;

use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug, PartialEq)]
#[structopt(name = "xunlei", version = "3.5.2")]
struct Opt {
    /// Enable debug
    #[structopt(short, long)]
    debug: bool,

    #[structopt(subcommand)]
    cmd: Cmd,
}

#[derive(StructOpt, Debug, PartialEq)]
pub enum Cmd {
    /// Install xunlei
    Install {
        /// Xunlei internal mode
        #[structopt(short, long)]
        internal: bool,
        /// Xunlei web-ui port
        port: Option<u16>,
        /// Xunlei config directory
        #[structopt(short, long)]
        config_path: Option<PathBuf>,
        /// Xunlei download directory
        #[structopt(short, long)]
        download_path: Option<PathBuf>,
    },
    /// Uninstall xunlei
    Uninstall,
    /// Execute xunlei daemon
    Execute {
        /// Xunlei config directory
        #[structopt(short, long)]
        path: Option<PathBuf>,
    },
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();
    init_log(opt.debug);
    match opt.cmd {
        Cmd::Install {
            internal,
            port,
            download_path,
            config_path,
        } => {
            service::XunleiInstall::new(internal, port, download_path, config_path)?.execute()?;
        }
        Cmd::Uninstall => {
            service::XunleiUninstall {}.execute()?;
        }
        Cmd::Execute { path } => {
            daemon::XunleiDaemon::new(path)?.execute()?;
        }
    }
    Ok(())
}

fn init_log(debug: bool) {
    match debug {
        true => std::env::set_var("RUST_LOG", "DEBUG"),
        false => std::env::set_var("RUST_LOG", "INFO"),
    };
    env_logger::builder()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} {}: {}",
                record.level(),
                //Format like you want to: <-----------------
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.args()
            )
        })
        .init();
}

pub trait Running {
    fn execute(&self) -> anyhow::Result<()>;
}
