pub mod daemon;
pub mod service;
pub mod standard;
pub mod xunlei_asset;
use std::io::Write;
use std::process::Stdio;

extern crate rouille;

use rouille::cgi::CgiRun;
use std::env;
use std::io;
use std::process::Command;

use crate::daemon::XunleiDaemon;

fn main() {
    println!("Now listening on localhost:8000");
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

pub trait Running {
    fn run(&self) -> anyhow::Result<()>;
}
