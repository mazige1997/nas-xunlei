pub mod daemon;
pub mod service;
pub mod standard;
pub mod xunlei_asset;
use std::io::Write;

use daemon::XunleiDaemon;
use xunlei_asset::XunleiAsset;

fn main() -> anyhow::Result<()> {
    init_log();

    {
        println!("{:?}", XunleiAsset::get_version()?);
    }
    Ok(())
}

fn init_log() {
    match std::env::var("RUST_LOG") {
        Ok(val) => std::env::set_var("RUST_LOG", val),
        Err(_) => std::env::set_var("RUST_LOG", "INFO"),
    }

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

pub trait Command {
    fn run(&self) -> anyhow::Result<()>;
}
