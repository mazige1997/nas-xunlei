use std::ops::Not;

use anyhow::Context;

fn main() {
    println!("{:?}", support().unwrap())
}

fn support() -> bool {
    let child_res = std::process::Command::new("systemctl")
        .arg("--help")
        .output();

    match child_res {
        Ok(output) => {
            if output.status.success() {
                return true;
            }
            log::warn!("[XunleiInstall] Your system does not support systemctl");
            false
        }
        Err(_) => false,
    }
}
