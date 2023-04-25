use std::fs::File;
use std::io::copy;
use std::io::Result;
use std::net::TcpStream;

const OUTPUT_FILE: &str = "nasxunlei-DSM7-x86_64.spk";

fn main() -> Result<()> {
    let mut response = get_http_response()?;

    let mut output_file = File::create(OUTPUT_FILE)?;

    copy(&mut response, &mut output_file)?;

    Ok(())
}

fn get_http_response() -> Result<TcpStream> {
    const URL: &str = "http://down.sandai.net/nas/nasxunlei-DSM7-x86_64.spk";
    let request = format!(
        "GET {} HTTP/1.1\r\n\
         Host: down.sandai.net\r\n\
         Connection: close\r\n\
         \r\n",
        URL
    );

    let mut stream = TcpStream::connect("down.sandai.net:80")?;
    std::io::Write::write(&mut stream, request.as_bytes())?;

    Ok(stream)
}
