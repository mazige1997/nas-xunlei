fn main() {
    let child = std::process::Command::new("systemctl")
        .arg("--help")
        .output()
        .unwrap();
    let status = child.status;
    println!("{:?}", status.success())
}
