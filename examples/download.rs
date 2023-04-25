use std::fs::File;
use std::io::copy;
use std::io::Result;
use std::io::Write;

const OUTPUT_FILE: &str = "/tmp/xunlei_bin/nasxunlei-DSM7-x86_64.spk";

fn main() -> Result<()> {
    // let mut response = ureq::get("http://down.sandai.net/nas/nasxunlei-DSM7-x86_64.spk")
    //     .call()
    //     .unwrap()
    //     .into_reader();

    // let mut output_file = File::create(OUTPUT_FILE)?;

    // copy(&mut response, &mut output_file)?;

    // output_file.flush()?;
    // drop(output_file);
    let filename = "nasxunlei-DSM7-x86_64.spk";
    let dir = "/tmp/xunlei_bin";
    std::process::Command::new("sh")
    .arg("-c")
    .arg(format!("
                tar --wildcards -Oxf $(find {dir} -type f -name {filename} | head -n1) package.tgz | tar --wildcards -xJC {dir} 'bin/bin/*' 'ui/index.cgi' &&
                mv {dir}/bin/bin/* {dir}/ &&
                mv {dir}/ui/index.cgi {dir}/xunlei-pan-cli-web &&
                rm -rf {dir}/bin/bin &&
                rm -rf {dir}/bin &&
                rm -rf {dir}/ui &&
                rm -f {dir}/version_code {dir}/{filename}
                "))
                .spawn()?
                .wait()?;

    Ok(())
}
