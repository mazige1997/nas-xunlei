use std::fs::File;
use std::io::Read;
use std::io::Result;
use std::io::Write;

const OUTPUT_FILE: &str = "/tmp/xunlei_bin/nasxunlei-DSM7-x86_64.spk";

fn main() -> Result<()> {
    let response = ureq::get("http://down.sandai.net/nas/nasxunlei-DSM7-x86_64.spk")
        .call()
        .unwrap();

    let total_size = response
        .header("Content-Length")
        .unwrap()
        .parse::<u64>()
        .unwrap();

    let pb = indicatif::ProgressBar::new(total_size);
    pb.set_style(indicatif::ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
        .unwrap()
        .progress_chars("#>-"));

    let mut output_file = File::create(OUTPUT_FILE)?;

    // std::io::copy(&mut response.into_reader(), &mut output_file)?;
    let mut downloaded = 0;
    let mut buf = [0; 1024];
    let mut reader = response.into_reader();
    loop {
        let n = reader.read(buf.as_mut())?;
        if n == 0 {
            break;
        }
        output_file.write_all(&buf[..n])?;
        let new = std::cmp::min(downloaded + (n as u64), total_size);
        downloaded = new;
        pb.set_position(new);
    }
    pb.finish_with_message("downloaded");

    output_file.flush()?;
    drop(output_file);
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
