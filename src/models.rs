use std::io::{Error, ErrorKind, Result};
use std::path::Path;
use std::process::Command;

pub const PUPLOC_URL: &'static str =
    "https://f002.backblazeb2.com/file/tehnokv-www/posts/puploc-with-trees/demo/puploc.bin";

pub fn download(url: &str, path: &Path) -> Result<()> {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .arg("/C")
            .arg(format!("curl.exe {} --output {}", url, path.display()))
            .output()?
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(format!("curl {} --output {}", url, path.display()))
            .output()?
    };

    let status = output.status;

    if status.success() {
        Ok(())
    } else {
        println!("{}", String::from_utf8_lossy(&output.stderr));
        match status.code() {
            Some(code) => Err(Error::new(
                ErrorKind::BrokenPipe,
                format!("error code: {}", code),
            )),
            None => Err(Error::new(ErrorKind::BrokenPipe, "terminated by signal")),
        }
    }
}
pub fn download_puploc(path: &Path) -> Result<()> {
    download(&PUPLOC_URL, path)
}
